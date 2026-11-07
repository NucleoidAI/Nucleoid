mod class;
mod control;
mod destructuring;
mod expression;
mod function;
mod postfix;
mod precedence;
mod primary;
mod statement;
mod template;

use std::collections::HashMap;

use super::lexer::{Keyword, Spanned, SyntaxOptions, Token, tokenize_with_options};
use crate::error::{Error, Position, Result};
use crate::lang::ast::{BinaryOp, Expr, LogicalOp, Parameter, Stmt};

/// A parsed program: its statements, and where each top-level statement began.
/// The positions are what places a runtime error in the source.
#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub positions: Vec<Position>,
}

pub fn parse_program(source: &str) -> Result<Program> {
    parse_program_with_options(source, SyntaxOptions::default())
}

/// Parses a program with explicit, opt-in source forms.
pub fn parse_program_with_options(source: &str, options: SyntaxOptions) -> Result<Program> {
    Parser::from_source(source, options)?.attempt(Parser::program)
}

pub fn parse(source: &str) -> Result<Vec<Stmt>> {
    Ok(parse_program(source)?.statements)
}

pub fn parse_with_options(source: &str, options: SyntaxOptions) -> Result<Vec<Stmt>> {
    Ok(parse_program_with_options(source, options)?.statements)
}

pub fn parse_expression(source: &str) -> Result<Expr> {
    parse_expression_with_options(source, SyntaxOptions::default())
}

pub fn parse_expression_with_options(source: &str, options: SyntaxOptions) -> Result<Expr> {
    Parser::from_source(source, options)?
        .attempt(|parser| parser.leading_expression(ExpressionContext::Ordinary))
}

fn parse_interpolation(source: &str, options: SyntaxOptions, words: bool) -> Result<Expr> {
    let mut parser = Parser {
        words,
        ..Parser::from_source(source, options)?
    };
    parser.leading_expression(ExpressionContext::Interpolation)
}

fn furthest(accepted: Error, retried: Error) -> Error {
    let further = match (accepted.position(), retried.position()) {
        (Some(first), Some(second)) => (second.line, second.column) > (first.line, first.column),
        _ => false,
    };
    if further { retried } else { accepted }
}

/// How deeply expressions and blocks may nest. Recursive descent uses the
/// stack, so the limit is what keeps a pathological source from overflowing it.
const MAX_NESTING: usize = 64;

/// How many operands one chain of operators may have. Chains are built
/// iteratively so they escape `MAX_NESTING`, but they still produce a deep tree
/// that later walks recurse over.
const MAX_CHAIN: usize = 128;

const MAX_DESTRUCTURING_TARGETS: usize = 128;

/// Compound syntax selects an existing operator, never a new runtime path.
enum CompoundOp {
    Arithmetic(BinaryOp),
    Logical(LogicalOp),
}

#[derive(Clone, Copy)]
enum ArgumentContext {
    PotentialConstructor,
    Ordinary,
}

/// What may legally follow the expression being parsed. Contextual spellings
/// consult it so that a following `:` keeps its accepted owner. Operands,
/// assignment values, conditional alternates and arrow or lambda bodies
/// inherit it; an expression nested in brackets, braces or call arguments
/// selects its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpressionContext {
    /// Statements, grouping, list items, call arguments and slice ends.
    Ordinary,
    /// An object-literal value, where a following `name:` starts the next
    /// comma-free property (NUC 44).
    ObjectValue,
    /// An `if`, `else` or `for` header, a subscript start (an index or slice
    /// start), a conditional's consequent or a Python-style condition, whose
    /// top-level `:` or `else` belongs to the enclosing construct.
    Clause,
    /// A template interpolation, whose trailing tokens are ignored, so a
    /// top-level `lambda` and a Python-style `if` keep their accepted readings.
    Interpolation,
}

struct Parser {
    tokens: Vec<Spanned>,
    position: usize,
    depth: usize,
    context: ExpressionContext,
    options: SyntaxOptions,
    words: bool,
    enclosed: bool,
    /// Python-style conditional attempts by `if` token position and nesting
    /// depth, which together decide one: the qualified condition and the
    /// position after its `else`, or `None`. A re-parse of the same source
    /// reuses an outcome instead of parsing the condition again.
    python_conditions: HashMap<(usize, usize), Option<PythonCondition>>,
    suppose_premises: HashMap<(usize, usize), Option<SupposePremise>>,
    while_conditions: HashMap<(usize, usize), Option<LoopCondition>>,
}

/// A qualified Python-style condition and the position after its `else`.
#[derive(Clone)]
struct PythonCondition {
    condition: Expr,
    end: usize,
}

#[derive(Clone)]
struct SupposePremise {
    premise: Expr,
    end: usize,
}

#[derive(Clone)]
struct LoopCondition {
    condition: Expr,
    colon: usize,
}

#[derive(Clone, Copy)]
enum FunctionSpelling {
    Def,
    Function,
}

#[derive(Default)]
struct ParameterList {
    parameters: Vec<Parameter>,
    initializers: Vec<Stmt>,
}

struct Pattern {
    targets: Vec<Target>,
    length: usize,
}

struct Target {
    name: String,
    read: Read,
    position: Position,
}

enum Read {
    Key(String),
    Index(u32),
}

impl Parser {
    fn new(tokens: Vec<Spanned>) -> Self {
        Parser {
            tokens,
            position: 0,
            depth: 0,
            context: ExpressionContext::Ordinary,
            options: SyntaxOptions::default(),
            words: false,
            enclosed: false,
            python_conditions: HashMap::new(),
            suppose_premises: HashMap::new(),
            while_conditions: HashMap::new(),
        }
    }

    fn from_source(source: &str, options: SyntaxOptions) -> Result<Self> {
        Ok(Parser {
            options,
            ..Parser::new(tokenize_with_options(source, options)?)
        })
    }

    fn attempt<T>(&mut self, parse: impl Fn(&mut Self) -> Result<T>) -> Result<T> {
        let accepted = match parse(self) {
            Ok(parsed) => return Ok(parsed),
            Err(error) => error,
        };
        if !self.mentions_words() {
            return Err(accepted);
        }
        *self = Parser {
            options: self.options,
            words: true,
            ..Parser::new(std::mem::take(&mut self.tokens))
        };
        parse(self).map_err(|retried| furthest(accepted, retried))
    }

    fn mentions_words(&self) -> bool {
        self.tokens.iter().any(|spanned| match &spanned.token {
            Token::Identifier(name) => name == "is",
            Token::Template(_) => true,
            _ => false,
        })
    }

    fn leading_expression(&mut self, context: ExpressionContext) -> Result<Expr> {
        self.skip_separators();
        self.expression_in(context)
    }

    fn within<T>(
        &mut self,
        enclosed: bool,
        parse: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let outer = std::mem::replace(&mut self.enclosed, enclosed);
        let result = parse(self);
        self.enclosed = outer;
        result
    }

    fn word_comparison(&self) -> Option<BinaryOp> {
        let qualifies = self.words
            && self.enclosed
            && matches!(self.peek(), Token::Identifier(name) if name == "is")
            && !(self.context == ExpressionContext::ObjectValue
                && self.peek_at(1) == &Token::Colon);
        qualifies.then(|| match self.peek_at(1) {
            Token::Keyword(Keyword::Not) => BinaryOp::IsNot,
            _ => BinaryOp::Is,
        })
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.position)
            .map_or(&Token::Eof, |spanned| &spanned.token)
    }

    fn peek_at(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.position + offset)
            .map_or(&Token::Eof, |spanned| &spanned.token)
    }

    fn advance(&mut self) -> Token {
        let token = self
            .tokens
            .get(self.position)
            .map_or(Token::Eof, |spanned| spanned.token.clone());

        if self.position < self.tokens.len() {
            self.position += 1;
        }

        token
    }

    /// Where the parser is looking.
    fn here(&self) -> Position {
        self.tokens
            .get(self.position)
            .or_else(|| self.tokens.last())
            .map_or_else(|| Position::new(1, 1), |spanned| spanned.position)
    }

    /// A syntax error placed where the parser is looking.
    fn error(&self, message: impl Into<String>) -> Error {
        Error::syntax(message).at(self.here())
    }

    fn check(&self, token: &Token) -> bool {
        self.peek() == token
    }

    fn eat(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token: &Token) -> Result<()> {
        if self.eat(token) {
            Ok(())
        } else {
            Err(self.error(format!(
                "Expected {} but found {}",
                token.describe(),
                self.peek().describe()
            )))
        }
    }

    /// Consumes either arrow spelling, `=>` or `->`.
    fn eat_arrow(&mut self) -> bool {
        if self.peek().is_arrow() {
            self.position += 1;
            true
        } else {
            false
        }
    }

    /// Requires either arrow spelling, naming the canonical one when absent.
    fn expect_arrow(&mut self) -> Result<()> {
        if self.eat_arrow() {
            Ok(())
        } else {
            self.expect(&Token::Arrow)
        }
    }

    fn eat_keyword(&mut self, keyword: Keyword) -> bool {
        if self.peek() == &Token::Keyword(keyword) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn skip_separators(&mut self) {
        while matches!(
            self.peek(),
            Token::Newline | Token::Semicolon | Token::Indent | Token::Dedent
        ) {
            self.position += 1;
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline | Token::Semicolon) {
            self.position += 1;
        }
    }
}

#[cfg(test)]
mod tests;
