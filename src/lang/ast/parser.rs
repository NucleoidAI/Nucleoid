use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

#[cfg(test)]
use super::lexer::tokenize;
use super::lexer::{
    Keyword, Spanned, SyntaxOptions, Token, excerpt, interpolation_end, tokenize_with_options,
};
use crate::error::{Error, Position, Result};
use crate::lang::ast::{
    BinaryOp, ClassDecl, Expr, Function, FunctionBody, LogicalOp, Noun, ObjectEntry, Parameter,
    ParameterDefault, Quantifier, Stmt, TemplatePart, UnaryOp, VersionSelector,
};
use crate::reasoning::Stage;

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
    let tokens = tokenize_with_options(source, options)?;
    let mut parser = Parser::new(tokens);
    parser.options = options;
    parser.program()
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
    parse_expression_in(source, ExpressionContext::Ordinary, options)
}

fn parse_expression_in(
    source: &str,
    context: ExpressionContext,
    options: SyntaxOptions,
) -> Result<Expr> {
    let tokens = tokenize_with_options(source, options)?;
    let mut parser = Parser::new(tokens);
    parser.options = options;
    parser.skip_separators();
    parser.expression_in(context)
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

impl Target {
    fn declaration(self, value: &Expr) -> Stmt {
        let object = Box::new(value.clone());
        Stmt::Assign {
            target: Expr::Identifier(self.name),
            value: match self.read {
                Read::Key(property) => Expr::Member { object, property },
                Read::Index(index) => Expr::Index {
                    object,
                    index: Box::new(Expr::Number(f64::from(index))),
                },
            },
        }
    }
}

impl Parser {
    fn new(tokens: Vec<Spanned>) -> Self {
        Parser {
            tokens,
            position: 0,
            depth: 0,
            context: ExpressionContext::Ordinary,
            options: SyntaxOptions::default(),
            python_conditions: HashMap::new(),
            suppose_premises: HashMap::new(),
            while_conditions: HashMap::new(),
        }
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

    fn program(&mut self) -> Result<Program> {
        let mut statements = Vec::new();
        let mut positions = Vec::new();

        loop {
            self.skip_separators();

            if self.check(&Token::Eof) {
                break;
            }

            let position = self.here();
            self.statement(&mut statements)?;
            positions.resize(statements.len(), position);
        }

        Ok(Program {
            statements,
            positions,
        })
    }

    /// Parses the body introduced by a `:`, accepting both an indented block and
    /// a single statement written on the same line.
    fn suite(&mut self) -> Result<Vec<Stmt>> {
        self.expect(&Token::Colon)?;
        self.suite_body()
    }

    /// Shares the existing body parser with conditional and exception clauses
    /// whose real indentation has already qualified for colon omission.
    fn suite_body(&mut self) -> Result<Vec<Stmt>> {
        if self.eat(&Token::Newline) || self.check(&Token::Indent) {
            self.skip_newlines();
            self.expect(&Token::Indent)?;

            let mut statements = Vec::new();

            loop {
                self.skip_newlines();

                if self.check(&Token::Dedent) || self.check(&Token::Eof) {
                    break;
                }

                self.statement(&mut statements)?;
            }

            self.eat(&Token::Dedent);
            Ok(statements)
        } else {
            let mut statements = Vec::new();
            self.statement(&mut statements)?;

            while self.eat(&Token::Semicolon) {
                if matches!(self.peek(), Token::Newline | Token::Eof) {
                    break;
                }
                self.statement(&mut statements)?;
            }

            Ok(statements)
        }
    }

    /// Parses the statements of a `{ ... }` block.
    fn braced(&mut self) -> Result<Vec<Stmt>> {
        self.expect(&Token::LeftBrace)?;

        let mut statements = Vec::new();

        loop {
            self.skip_separators();

            if self.eat(&Token::RightBrace) {
                break;
            }

            if self.check(&Token::Eof) {
                return Err(self.error("Unterminated block"));
            }

            self.statement(&mut statements)?;
        }

        Ok(statements)
    }

    fn statement(&mut self, statements: &mut Vec<Stmt>) -> Result<()> {
        self.depth += 1;

        if self.depth > MAX_NESTING {
            self.depth -= 1;
            return Err(self.error("Statements are nested too deeply"));
        }

        let depth = self.depth;
        let result = self.destructuring().and_then(|declarations| {
            match declarations {
                Some(declarations) => statements.extend(declarations),
                None => statements.push(self.statement_inner()?),
            }
            Ok(())
        });
        debug_assert_eq!(self.depth, depth);
        self.depth -= 1;
        result
    }

    fn destructuring(&mut self) -> Result<Option<Vec<Stmt>>> {
        if !self.starts_statement() {
            return Ok(None);
        }
        let Some(pattern) = self.object_pattern().or_else(|| self.unpacking_pattern()) else {
            return Ok(None);
        };

        let start = self.position;
        self.position += pattern.length;
        let value = match self.nested_expression(Self::destructured_value) {
            Ok(value) if self.ends_destructuring() => value,
            _ => {
                self.position = start;
                return Ok(None);
            }
        };

        if let Some(target) = pattern.targets.get(MAX_DESTRUCTURING_TARGETS) {
            return Err(Error::syntax("Destructuring has too many targets").at(target.position));
        }

        Ok(Some(
            pattern
                .targets
                .into_iter()
                .map(|target| target.declaration(&value))
                .collect(),
        ))
    }

    fn destructured_value(&mut self) -> Result<Expr> {
        self.nested_expression_in(ExpressionContext::Ordinary, Self::conditional)
    }

    fn starts_statement(&self) -> bool {
        self.position
            .checked_sub(1)
            .and_then(|previous| self.tokens.get(previous))
            .is_none_or(|previous| {
                matches!(
                    previous.token,
                    Token::Newline
                        | Token::Semicolon
                        | Token::Indent
                        | Token::Dedent
                        | Token::LeftBrace
                        | Token::Colon
                )
            })
    }

    fn ends_destructuring(&self) -> bool {
        matches!(
            self.peek(),
            Token::Newline
                | Token::Semicolon
                | Token::Dedent
                | Token::RightBrace
                | Token::Eof
                | Token::Keyword(Keyword::Else | Keyword::Catch)
        )
    }

    fn target_at(&self, offset: usize, name: &str, read: Read) -> Target {
        Target {
            name: name.to_string(),
            read,
            position: self
                .tokens
                .get(self.position + offset)
                .map_or_else(|| self.here(), |spanned| spanned.position),
        }
    }

    fn object_pattern(&self) -> Option<Pattern> {
        if !matches!(
            (self.peek(), self.peek_at(1)),
            (Token::LeftBrace, Token::Identifier(_))
        ) {
            return None;
        }
        let mut targets = Vec::new();
        let mut offset = 1;
        loop {
            match (
                self.peek_at(offset),
                self.peek_at(offset + 1),
                self.peek_at(offset + 2),
            ) {
                (Token::RightBrace, _, _) if !targets.is_empty() => break,
                (Token::Identifier(key), Token::Colon, Token::Identifier(name)) => {
                    targets.push(self.target_at(offset + 2, name, Read::Key(key.clone())));
                    offset += 3;
                    match (self.peek_at(offset), self.peek_at(offset + 1)) {
                        (Token::Comma, _) => offset += 1,
                        (Token::RightBrace, _) | (Token::Identifier(_), Token::Colon) => {}
                        _ => return None,
                    }
                }
                (Token::Identifier(name), Token::Comma | Token::RightBrace, _) => {
                    targets.push(self.target_at(offset, name, Read::Key(name.clone())));
                    offset += 1;
                    if self.peek_at(offset) == &Token::Comma {
                        offset += 1;
                    }
                }
                _ => return None,
            }
        }
        (self.peek_at(offset + 1) == &Token::Assign).then_some(Pattern {
            targets,
            length: offset + 2,
        })
    }

    fn unpacking_pattern(&self) -> Option<Pattern> {
        if !matches!(
            (self.peek(), self.peek_at(1)),
            (Token::Identifier(_), Token::Comma)
        ) {
            return None;
        }
        let mut targets = Vec::new();
        let mut offset = 0;
        loop {
            match self.peek_at(offset) {
                Token::Identifier(name) => {
                    let index = u32::try_from(targets.len()).ok()?;
                    targets.push(self.target_at(offset, name, Read::Index(index)));
                    offset += 1;
                    match self.peek_at(offset) {
                        Token::Comma => offset += 1,
                        Token::Assign => break,
                        _ => return None,
                    }
                }
                Token::Assign => break,
                _ => return None,
            }
        }
        Some(Pattern {
            targets,
            length: offset + 1,
        })
    }

    fn statement_inner(&mut self) -> Result<Stmt> {
        if let Some(condition) = self.while_condition() {
            let body = self.suite()?;
            return Ok(Stmt::While { condition, body });
        }

        match self.peek().clone() {
            Token::Keyword(Keyword::If) => self.if_statement(),
            Token::Keyword(Keyword::For) => self.for_statement(),
            Token::Keyword(Keyword::Class) => self.class_statement(),
            Token::Keyword(Keyword::Def) => self.function_statement(),
            Token::Keyword(Keyword::Try) => self.try_statement(),
            Token::Keyword(Keyword::Pass) => {
                self.advance();
                Ok(Stmt::Pass)
            }
            Token::Keyword(Keyword::Return) => self.return_statement(),
            Token::Keyword(Keyword::Throw) => {
                self.advance();
                Ok(Stmt::Throw(self.expression()?))
            }
            Token::Keyword(Keyword::Delete) => {
                self.advance();
                Ok(Stmt::Delete(self.expression()?))
            }
            Token::LeftBrace => Ok(Stmt::Block(self.braced()?)),
            Token::Identifier(name) if self.is_type_declaration() => {
                let type_name = self.declared_type()?;
                Ok(Stmt::Declaration { name, type_name })
            }
            Token::Identifier(_) if self.is_assertion_statement_at(0) => {
                self.advance();
                self.advance();
                // An ordinary call has an outer expression level in addition
                // to each argument's level. Preserve that same nesting budget.
                self.nested_expression(Self::assertion_expression)
                    .map(Stmt::Expression)
            }
            _ => {
                let expression = self.expression()?;

                if self.check(&Token::ColonAssign) {
                    return self.value_assignment(expression);
                }

                if let Some(statement) = self.suppose_statement(&expression)? {
                    return Ok(statement);
                }

                Ok(simple_statement(expression))
            }
        }
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        self.advance();

        if matches!(
            self.peek(),
            Token::Newline | Token::Semicolon | Token::Dedent | Token::RightBrace | Token::Eof
        ) {
            Ok(Stmt::Return(None))
        } else {
            Ok(Stmt::Return(Some(self.expression()?)))
        }
    }

    fn value_assignment(&mut self, target: Expr) -> Result<Stmt> {
        self.check_path_target(&target, "Invalid value assignment target")?;
        self.advance();
        let value = self.expression()?;
        Ok(Stmt::ValueAssign { target, value })
    }

    fn suppose_statement(&mut self, expression: &Expr) -> Result<Option<Stmt>> {
        if !heads_suppose(expression) {
            return Ok(None);
        }
        let target = match expression {
            Expr::Assign { target, .. } => Some(target.as_ref().clone()),
            _ => None,
        };

        let mut body = Vec::new();
        if !self.check(&Token::Colon) {
            let Some(premise) = self.suppose_premise() else {
                return Ok(None);
            };
            body.push(premise);
        }
        body.extend(self.suite()?);

        Ok(Some(Stmt::Suppose { target, body }))
    }

    fn suppose_premise(&mut self) -> Option<Stmt> {
        if self.starts_colon_statement_at(0)
            || matches!(
                self.peek(),
                Token::Newline | Token::Semicolon | Token::Dedent | Token::RightBrace | Token::Eof
            )
        {
            return None;
        }

        let attempt = (self.position, self.depth);
        let outcome = if let Some(outcome) = self.suppose_premises.get(&attempt) {
            outcome.clone()
        } else {
            let outcome = self.attempt_suppose_premise();
            self.suppose_premises.insert(attempt, outcome.clone());
            outcome
        };
        let SupposePremise { premise, end } = outcome?;
        self.position = end;
        Some(simple_statement(premise))
    }

    fn attempt_suppose_premise(&mut self) -> Option<SupposePremise> {
        let start = self.position;
        let outcome = self.clause_before_colon().map(|premise| SupposePremise {
            premise,
            end: self.position,
        });
        self.position = start;
        outcome
    }

    fn clause_before_colon(&mut self) -> Option<Expr> {
        let clause = self.clause_expression().ok()?;
        let owns_colon = self.check(&Token::Colon)
            && !matches!(
                self.position
                    .checked_sub(1)
                    .and_then(|previous| self.tokens.get(previous))
                    .map(|spanned| &spanned.token),
                Some(Token::Identifier(name)) if name == "lambda"
            );
        owns_colon.then_some(clause)
    }

    fn assertion_expression(&mut self) -> Result<Expr> {
        // Keep the same argument AST as parenthesized assert(...).
        // Evaluation is still the ordinary helper, not a constructor.
        let actual = self.argument(ArgumentContext::PotentialConstructor)?;
        self.expect(&Token::Comma)?;
        let expected = self.argument(ArgumentContext::PotentialConstructor)?;
        if self.check(&Token::Comma) {
            return Err(self.error("Assertion statements expect exactly two arguments"));
        }
        Ok(Expr::Call {
            callee: Box::new(Expr::Identifier("assert".to_string())),
            arguments: vec![actual, expected],
        })
    }

    /// Distinguishes `a: int` (a declaration) from a labelled suite.
    fn is_type_declaration(&self) -> bool {
        self.is_type_declaration_at(0)
    }

    /// Whether a complete `name: Type` declaration, which is also a class
    /// field, starts at `offset`: a name, its colon and one type name before
    /// a statement boundary.
    fn is_type_declaration_at(&self, offset: usize) -> bool {
        matches!(self.peek_at(offset), Token::Identifier(_))
            && self.peek_at(offset + 1) == &Token::Colon
            && matches!(self.peek_at(offset + 2), Token::Identifier(_))
            && matches!(
                self.peek_at(offset + 3),
                Token::Newline | Token::Semicolon | Token::Dedent | Token::Eof
            )
    }

    /// Whether NUC 45's `assert: actual, expected` statement starts at
    /// `offset`.
    fn is_assertion_statement_at(&self, offset: usize) -> bool {
        matches!(self.peek_at(offset), Token::Identifier(name) if name == "assert")
            && self.peek_at(offset + 1) == &Token::Colon
    }

    /// Whether a statement led by a name and its colon starts at `offset`:
    /// exactly the declaration or assertion guard that the statement parser
    /// itself uses. These are the only statements a name and a colon begin,
    /// and statement loops accept them directly after an expression
    /// statement.
    fn starts_colon_statement_at(&self, offset: usize) -> bool {
        self.is_type_declaration_at(offset) || self.is_assertion_statement_at(offset)
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::If))?;
        self.condition_clause()
    }

    /// Explicit `else if` and qualified direct alternatives share the entire
    /// condition, suite and recursive alternate parser.
    fn condition_clause(&mut self) -> Result<Stmt> {
        let condition = self.clause_expression()?;
        // Explicit colons retain every accepted suite and diagnostic. Only a
        // real indented body can select the optional conditional spelling.
        let consequent = if self.check(&Token::Colon) || !self.has_indented_body(0) {
            self.suite()?
        } else {
            self.suite_body()?
        };

        let mut alternate = None;
        let checkpoint = self.position;
        self.skip_newlines();

        if self.eat_keyword(Keyword::Else) {
            if self.check(&Token::Keyword(Keyword::If)) {
                alternate = Some(Box::new(self.if_statement()?));
            } else if self.has_direct_condition_suite() {
                alternate = Some(Box::new(self.condition_clause()?));
            } else {
                let body = if self.has_indented_body(0) {
                    self.suite_body()?
                } else {
                    self.suite()?
                };
                alternate = Some(Box::new(Stmt::Block(body)));
            }
        } else {
            self.position = checkpoint;
        }

        Ok(Stmt::If {
            condition,
            consequent,
            alternate,
        })
    }

    /// Qualify a nonempty, balanced header without consuming tokens or parsing
    /// an expression. Only a top-level colon or real body layout may end it;
    /// colons in calls, slices and object literals still belong to the header,
    /// as does each top-level conditional's colon after its `?`.
    fn has_direct_condition_suite(&self) -> bool {
        let mut delimiters = Vec::new();
        let mut conditionals = 0usize;
        for (offset, spanned) in self.tokens.iter().skip(self.position).enumerate() {
            match &spanned.token {
                token @ (Token::LeftParen | Token::LeftBracket | Token::LeftBrace) => {
                    delimiters.push(token);
                }
                token @ (Token::RightParen | Token::RightBracket | Token::RightBrace) => {
                    if !matches!(
                        (delimiters.pop(), token),
                        (Some(Token::LeftParen), Token::RightParen)
                            | (Some(Token::LeftBracket), Token::RightBracket)
                            | (Some(Token::LeftBrace), Token::RightBrace)
                    ) {
                        return false;
                    }
                }
                Token::Question if delimiters.is_empty() => conditionals += 1,
                Token::Identifier(_)
                    if delimiters.is_empty() && self.quantifier_at(offset).is_some() =>
                {
                    conditionals += 1;
                }
                Token::Colon if delimiters.is_empty() && conditionals > 0 => conditionals -= 1,
                Token::Colon if delimiters.is_empty() => {
                    return offset > 0 && self.has_indented_suite(offset);
                }
                Token::Newline | Token::Indent if delimiters.is_empty() => {
                    return offset > 0 && self.has_indented_body(offset);
                }
                Token::Semicolon | Token::ColonAssign | Token::Dedent | Token::Eof
                    if delimiters.is_empty() =>
                {
                    return false;
                }
                _ => {}
            }
        }
        false
    }

    fn while_condition(&mut self) -> Option<Expr> {
        if !matches!(self.peek(), Token::Identifier(name) if name == "while")
            || self.starts_colon_statement_at(1)
        {
            return None;
        }

        let attempt = (self.position, self.depth);
        let outcome = if let Some(outcome) = self.while_conditions.get(&attempt) {
            outcome.clone()
        } else {
            let outcome = self.attempt_while_condition();
            self.while_conditions.insert(attempt, outcome.clone());
            outcome
        };
        let LoopCondition { condition, colon } = outcome?;
        self.position = colon;
        Some(condition)
    }

    fn attempt_while_condition(&mut self) -> Option<LoopCondition> {
        let start = self.position;
        self.advance();
        let outcome = self
            .clause_before_colon()
            .filter(|condition| !heads_suppose(condition))
            .map(|condition| LoopCondition {
                condition,
                colon: self.position,
            });
        self.position = start;
        outcome
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::For))?;

        if let Some((key, value)) = self.entry_names() {
            let iterable = self.clause_expression()?;
            let body = self.suite()?;
            return Ok(Stmt::ForEntries {
                key,
                value,
                iterable,
                body,
            });
        }

        let variable = if matches!(self.peek(), Token::Identifier(_)) && self.has_indented_suite(1)
        {
            // Leave the named source and its complete suite to the existing
            // parsers. Nonqualifying prefixes retain the explicit-loop path.
            "it".to_string()
        } else {
            let variable = match self.advance() {
                Token::Identifier(name) => name,
                other => {
                    return Err(
                        self.error(format!("Expected a name but found {}", other.describe()))
                    );
                }
            };

            if !self.eat_keyword(Keyword::Of) && !self.eat_keyword(Keyword::In) {
                return Err(self.error("Expected 'of' in for statement"));
            }

            variable
        };

        let iterable = self.clause_expression()?;
        let body = self.suite()?;

        Ok(Stmt::For {
            variable,
            iterable,
            body,
        })
    }

    fn entry_names(&mut self) -> Option<(String, String)> {
        let (Token::Identifier(key), Token::Comma, Token::Identifier(value)) =
            (self.peek(), self.peek_at(1), self.peek_at(2))
        else {
            return None;
        };
        if !matches!(self.peek_at(3), Token::Keyword(Keyword::Of | Keyword::In)) {
            return None;
        }
        let names = (key.clone(), value.clone());
        self.position += 4;
        Some(names)
    }

    /// Recognizes only the real-indented colon path already accepted by
    /// `suite()`, without consuming tokens or skipping header separators.
    fn has_indented_suite(&self, colon_offset: usize) -> bool {
        matches!(self.peek_at(colon_offset), Token::Colon)
            && self.has_indented_body(colon_offset + 1)
    }

    /// Qualify the complete layout prefix before consuming any of it. A
    /// same-line semicolon is not a newline, even if an indent follows later.
    fn has_indented_body(&self, offset: usize) -> bool {
        matches!(self.peek_at(offset), Token::Newline | Token::Indent)
            && self
                .tokens
                .iter()
                .skip(self.position + offset)
                .find(|spanned| !matches!(spanned.token, Token::Newline | Token::Semicolon))
                .is_some_and(|spanned| spanned.token == Token::Indent)
    }

    fn try_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::Try))?;
        // Explicit colons keep the original suite path and diagnostics.
        let body = if self.check(&Token::Colon) || !self.has_indented_body(0) {
            self.suite()?
        } else {
            self.suite_body()?
        };
        self.skip_newlines();

        if !self.eat_keyword(Keyword::Catch) {
            return Err(self.error("Expected 'catch' after 'try'"));
        }

        let parameter = match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                name
            }
            Token::LeftParen => {
                self.advance();
                let name = match self.advance() {
                    Token::Identifier(name) => name,
                    other => {
                        return Err(
                            self.error(format!("Expected a name but found {}", other.describe()))
                        );
                    }
                };
                self.expect(&Token::RightParen)?;
                name
            }
            _ => "error".to_string(),
        };

        let catch = if self.check(&Token::Colon) || !self.has_indented_body(0) {
            self.suite()?
        } else {
            self.suite_body()?
        };

        Ok(Stmt::Try {
            body,
            parameter,
            catch,
        })
    }

    /// Offset of the delimiter after a complete bare list of at least two
    /// ordinary identifiers with optional identifier annotations, ending in an
    /// arrow or suite colon. Inspect without consuming tokens, allocating names
    /// or skipping layout; the caller checks a block's real indentation.
    fn bare_multiple_parameter_end(&self) -> Option<usize> {
        let mut offset = 0;
        let mut multiple = false;
        loop {
            if !matches!(self.peek_at(offset), Token::Identifier(_)) {
                return None;
            }
            offset += 1;
            if matches!(
                (self.peek_at(offset), self.peek_at(offset + 1)),
                (Token::Colon, Token::Identifier(_))
            ) {
                offset += 2;
            }
            match self.peek_at(offset) {
                delimiter if multiple && (delimiter.is_arrow() || delimiter == &Token::Colon) => {
                    return Some(offset);
                }
                Token::Comma => {
                    multiple = true;
                    offset += 1;
                }
                _ => return None,
            }
        }
    }

    /// Scan a complete uninterrupted run without consuming or allocating names.
    /// Each caller must still qualify its own delimiter and body layout.
    fn comma_free_parameter_end(&self) -> Option<usize> {
        let mut end = 0;
        while matches!(self.peek_at(end), Token::Identifier(_)) {
            end += 1;
        }
        (end >= 2).then_some(end)
    }

    /// Keep the arrow guard arrow-only; blocks need a genuine indented suite.
    fn comma_free_arrow_end(&self) -> Option<usize> {
        self.comma_free_parameter_end()
            .filter(|&end| self.peek_at(end).is_arrow())
    }

    fn comma_free_block_end(&self) -> Option<usize> {
        self.comma_free_parameter_end()
            .filter(|&end| self.has_indented_suite(end))
    }

    /// Only used after complete lookahead has qualified an untyped name.
    /// In particular, leave a final suite colon for `suite()`, not `parameter()`.
    fn untyped_parameter(&mut self) -> Result<Parameter> {
        match self.advance() {
            Token::Identifier(name) => Ok(Parameter {
                name,
                type_name: None,
                default: None,
                rest: false,
            }),
            other => Err(self.error(format!(
                "Expected a parameter but found {}",
                other.describe()
            ))),
        }
    }

    fn function_statement(&mut self) -> Result<Stmt> {
        self.function_declaration().map(Stmt::Function)
    }

    fn function_declaration(&mut self) -> Result<Arc<Function>> {
        self.expect(&Token::Keyword(Keyword::Def))?;

        let name = match self.advance() {
            Token::Identifier(name) => name,
            other => {
                return Err(self.error(format!("Expected a name but found {}", other.describe())));
            }
        };

        let parameters = self.named_parameters()?;
        let body = self.named_body()?;

        Ok(Arc::new(Function {
            name: Some(name),
            parameters,
            body: FunctionBody::Block(body),
        }))
    }

    fn named_parameters(&mut self) -> Result<Vec<Parameter>> {
        // Either arrow spelling, `=>` or `->`, ends a bare arrow header.
        let bare_arrow = matches!(self.peek(), Token::Identifier(_))
            && (self.peek_at(1).is_arrow()
                || (self.peek_at(1) == &Token::Colon
                    && matches!(self.peek_at(2), Token::Identifier(_))
                    && self.peek_at(3).is_arrow()));
        let bare_multiple_end = self
            .bare_multiple_parameter_end()
            .filter(|&end| self.peek_at(end).is_arrow() || self.has_indented_suite(end));
        let bare_block = matches!(self.peek(), Token::Identifier(_)) && self.has_indented_suite(1);
        let bare_typed_block = matches!(
            (
                self.peek(),
                self.peek_at(1),
                self.peek_at(2),
                self.peek_at(3)
            ),
            (
                Token::Identifier(_),
                Token::Colon,
                Token::Identifier(_),
                Token::Colon
            )
        ) && self.has_indented_suite(3);
        let comma_free_end = self
            .comma_free_arrow_end()
            .or_else(|| self.comma_free_block_end());
        if let Some(end) = comma_free_end {
            (0..end).map(|_| self.untyped_parameter()).collect()
        } else if let Some(end) = bare_multiple_end {
            self.bare_parameters(self.position + end)
        } else if bare_arrow || bare_typed_block {
            Ok(vec![self.parameter()?])
        } else if bare_block {
            Ok(vec![self.untyped_parameter()?])
        } else {
            // Do not skip separators or consume other prefixes: preserve the
            // parenthesized parameter parser's diagnostics and positions.
            self.parameters()
        }
    }

    fn bare_parameters(&mut self, delimiter_position: usize) -> Result<Vec<Parameter>> {
        let mut parameters = Vec::new();
        loop {
            let parameter = if self.position + 1 == delimiter_position
                && matches!(self.peek_at(1), Token::Colon)
            {
                // A final untyped parameter's colon belongs to suite(),
                // not parameter()'s annotation parser.
                self.untyped_parameter()?
            } else {
                self.parameter()?
            };
            parameters.push(parameter);
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        // Leave the complete list's arrow or suite colon for the body.
        Ok(parameters)
    }

    fn named_body(&mut self) -> Result<Vec<Stmt>> {
        if !self.eat_arrow() {
            return self.suite();
        }
        let indented = self.has_indented_body(0);
        if indented {
            self.skip_newlines();
            self.expect(&Token::Indent)?;
            self.skip_newlines();
        }
        let expression = self.expression()?;
        if indented {
            self.skip_newlines();
            self.expect(&Token::Dedent)?;
        }
        Ok(vec![Stmt::Return(Some(expression))])
    }

    fn parameter(&mut self) -> Result<Parameter> {
        let name = match self.advance() {
            Token::Identifier(name) => name,
            other => {
                return Err(self.error(format!(
                    "Expected a parameter but found {}",
                    other.describe()
                )));
            }
        };

        let type_name = if self.eat(&Token::Colon) {
            match self.advance() {
                Token::Identifier(type_name) => Some(type_name),
                other => {
                    return Err(
                        self.error(format!("Expected a type but found {}", other.describe()))
                    );
                }
            }
        } else {
            None
        };

        Ok(Parameter {
            name,
            type_name,
            default: None,
            rest: false,
        })
    }

    /// Only `parameters()` calls this: bare signature parsing is unchanged.
    fn parameter_default(&mut self) -> Result<ParameterDefault> {
        let negative = if self.eat(&Token::Minus) {
            Some(true)
        } else if self.eat(&Token::Plus) {
            Some(false)
        } else {
            None
        };
        let default = match self.peek().clone() {
            Token::Number(number) => {
                if !number.is_finite() {
                    return Err(self.error("Parameter default must be a finite number"));
                }
                self.advance();
                ParameterDefault::Number(if negative == Some(true) {
                    -number
                } else {
                    number
                })
            }
            Token::Str(value) if negative.is_none() => {
                self.advance();
                ParameterDefault::String(value)
            }
            Token::Keyword(Keyword::True | Keyword::False) if negative.is_none() => {
                let value = self.eat_keyword(Keyword::True);
                if !value {
                    self.advance();
                }
                ParameterDefault::Bool(value)
            }
            Token::Keyword(Keyword::Null) if negative.is_none() => {
                self.advance();
                ParameterDefault::Null
            }
            _ => return Err(self.error("Expected a scalar literal parameter default")),
        };
        if !matches!(self.peek(), Token::Comma | Token::RightParen) {
            return Err(self.error("Expected ',' or ')' after parameter default"));
        }
        Ok(default)
    }

    /// Recognize adjacent dots only here, without changing tokenization of
    /// any existing source or adding rest syntax to bare signatures.
    fn starts_rest_parameter(&self) -> bool {
        let start = self.here();
        (0..3).all(|offset| {
            self.tokens
                .get(self.position + offset)
                .is_some_and(|spanned| {
                    spanned.token == Token::Dot
                        && spanned.position == Position::new(start.line, start.column + offset)
                })
        })
    }

    fn parameters(&mut self) -> Result<Vec<Parameter>> {
        self.expect(&Token::LeftParen)?;

        let mut parameters: Vec<Parameter> = Vec::new();

        while !self.check(&Token::RightParen) {
            let rest = self.starts_rest_parameter();
            if rest {
                self.position += 3;
            }
            let mut parameter = self.parameter()?;
            parameter.rest = rest;
            if rest && parameters.iter().any(|fixed| fixed.name == parameter.name) {
                return Err(self.error(format!(
                    "Rest parameter '{}' duplicates a fixed parameter",
                    parameter.name
                )));
            }
            if self.eat(&Token::Assign) {
                if rest {
                    return Err(self.error("Rest parameter cannot have a default"));
                }
                parameter.default = Some(self.parameter_default()?);
            }
            parameters.push(parameter);

            if !self.eat(&Token::Comma) {
                break;
            }
            if rest && !self.check(&Token::RightParen) {
                return Err(self.error("Rest parameter must be last"));
            }
        }

        self.expect(&Token::RightParen)?;
        Ok(parameters)
    }

    fn class_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::Class))?;

        let name = match self.advance() {
            Token::Identifier(name) => name,
            other => {
                return Err(self.error(format!("Expected a name but found {}", other.describe())));
            }
        };

        let parameters = if self.check(&Token::LeftParen) {
            self.parameters()?
        } else {
            Vec::new()
        };

        if matches!(self.peek(), Token::Identifier(word) if word == "extends") {
            return self.extends_class(name, parameters);
        }

        if self.ends_bodyless_header() {
            return Ok(Stmt::Class(empty_class(name, None, parameters)));
        }

        self.expect(&Token::Colon)?;

        // `class Student: Person` names a parent on the same line.
        let parent = self.colon_parent().map(str::to_owned);

        if parent.is_some() {
            self.advance();
        }

        let mut declaration = empty_class(name, parent, parameters);
        self.class_body(&mut declaration)?;
        Ok(Stmt::Class(declaration))
    }

    fn extends_class(&mut self, name: String, parameters: Vec<Parameter>) -> Result<Stmt> {
        self.advance();

        let Token::Identifier(parent) = self.peek().clone() else {
            return Err(self.error(format!(
                "Expected a parent class name but found {}",
                self.peek().describe()
            )));
        };
        self.advance();

        if self.ends_bodyless_header() {
            return Ok(Stmt::Class(empty_class(name, Some(parent), parameters)));
        }

        self.expect(&Token::Colon)?;

        if self.colon_parent().is_some() {
            return Err(self.error(format!("Class '{name}' already extends '{parent}'")));
        }

        let mut declaration = empty_class(name, Some(parent), parameters);
        self.class_body(&mut declaration)?;
        Ok(Stmt::Class(declaration))
    }

    fn ends_bodyless_header(&self) -> bool {
        // Leave the terminator for the enclosing suite or block. An indent
        // after separators still requires a colon; skip_separators would
        // otherwise turn a misspelled class body into separate statements.
        matches!(
            self.peek(),
            Token::Newline | Token::Semicolon | Token::Dedent | Token::RightBrace | Token::Eof
        ) && self
            .tokens
            .iter()
            .skip(self.position)
            .find(|spanned| !matches!(spanned.token, Token::Newline | Token::Semicolon))
            .is_none_or(|spanned| spanned.token != Token::Indent)
    }

    fn colon_parent(&self) -> Option<&str> {
        match (self.peek(), self.peek_at(1)) {
            (Token::Identifier(parent), Token::Newline | Token::Indent | Token::Eof) => {
                Some(parent)
            }
            _ => None,
        }
    }

    fn class_body(&mut self, declaration: &mut ClassDecl) -> Result<()> {
        self.skip_newlines();

        if self.eat(&Token::Indent) {
            self.class_members(declaration)?;
            self.eat(&Token::Dedent);
        } else if !matches!(self.peek(), Token::Newline | Token::Eof) {
            self.statement(&mut declaration.constructor)?;
        }

        Ok(())
    }

    fn class_members(&mut self, declaration: &mut ClassDecl) -> Result<()> {
        loop {
            self.skip_newlines();

            if self.check(&Token::Dedent) || self.check(&Token::Eof) {
                return Ok(());
            }

            match self.peek().clone() {
                Token::Keyword(Keyword::Pass) => {
                    self.advance();
                }
                Token::Keyword(Keyword::Def) => {
                    declaration.methods.push(self.function_declaration()?);
                }
                Token::Identifier(field) if self.is_type_declaration() => {
                    let type_name = self.declared_type()?;
                    declaration.fields.push(Parameter {
                        name: field,
                        type_name: Some(type_name),
                        default: None,
                        rest: false,
                    });
                }
                _ => self.statement(&mut declaration.constructor)?,
            }
        }
    }

    fn declared_type(&mut self) -> Result<String> {
        self.advance();
        self.advance();
        match self.advance() {
            Token::Identifier(type_name) => Ok(type_name),
            other => Err(self.error(format!("Expected a type but found {}", other.describe()))),
        }
    }

    fn expression(&mut self) -> Result<Expr> {
        self.expression_in(ExpressionContext::Ordinary)
    }

    /// An `if`/`else`/`for` header, subscript start or conditional consequent:
    /// its top-level `:` belongs to the enclosing construct, never to a
    /// contextual `lambda`.
    fn clause_expression(&mut self) -> Result<Expr> {
        self.expression_in(ExpressionContext::Clause)
    }

    fn expression_in(&mut self, context: ExpressionContext) -> Result<Expr> {
        self.nested_expression_in(context, Self::expression_inner)
    }

    /// Runs one nested `parse` in `context`, then restores the enclosing
    /// context whether or not it succeeded.
    fn nested_expression_in(
        &mut self,
        context: ExpressionContext,
        parse: impl FnOnce(&mut Self) -> Result<Expr>,
    ) -> Result<Expr> {
        let outer = std::mem::replace(&mut self.context, context);
        let expression = self.nested_expression(parse);
        self.context = outer;
        expression
    }

    fn nested_expression(&mut self, parse: impl FnOnce(&mut Self) -> Result<Expr>) -> Result<Expr> {
        self.depth += 1;

        if self.depth > MAX_NESTING {
            self.depth -= 1;
            return Err(self.error("Expressions are nested too deeply"));
        }

        let depth = self.depth;
        let result = parse(self);
        debug_assert_eq!(self.depth, depth);
        self.depth -= 1;
        result
    }

    fn expression_inner(&mut self) -> Result<Expr> {
        let left = self.conditional()?;

        let operator = match self.peek() {
            Token::Assign => None,
            Token::PlusAssign => Some(CompoundOp::Arithmetic(BinaryOp::Add)),
            Token::MinusAssign => Some(CompoundOp::Arithmetic(BinaryOp::Subtract)),
            Token::StarAssign => Some(CompoundOp::Arithmetic(BinaryOp::Multiply)),
            Token::StarStarAssign => Some(CompoundOp::Arithmetic(BinaryOp::Power)),
            Token::SlashAssign => Some(CompoundOp::Arithmetic(BinaryOp::Divide)),
            Token::PercentAssign => Some(CompoundOp::Arithmetic(BinaryOp::Modulo)),
            Token::AndAndAssign => Some(CompoundOp::Logical(LogicalOp::And)),
            Token::OrOrAssign => Some(CompoundOp::Logical(LogicalOp::Or)),
            _ => return Ok(left),
        };

        if operator.is_some() {
            self.check_compound_target(&left)?;
        }

        self.advance();
        let value = self.expression_in(self.context)?;
        let value = match operator {
            Some(operator) => {
                let left = Box::new(left.clone());
                let right = Box::new(value);
                match operator {
                    CompoundOp::Arithmetic(operator) => Expr::Binary {
                        operator,
                        left,
                        right,
                    },
                    CompoundOp::Logical(operator) => Expr::Logical {
                        operator,
                        left,
                        right,
                    },
                }
            }
            None => value,
        };
        Ok(Expr::Assign {
            target: Box::new(left),
            value: Box::new(value),
        })
    }

    /// Only clone bounded, plain paths. A computed target containing another
    /// compound assignment could otherwise double its tree at each lowering.
    fn check_compound_target(&self, target: &Expr) -> Result<()> {
        self.check_path_target(target, "Invalid compound assignment target")
    }

    fn check_path_target(&self, mut target: &Expr, invalid: &str) -> Result<()> {
        let mut parts = 0;
        while let Expr::Member { object, .. }
        | Expr::Version {
            reference: object, ..
        } = target
        {
            parts += 1;
            if parts > MAX_CHAIN {
                return Err(self.error("Expression has too many parts"));
            }
            target = object;
        }

        match target {
            Expr::Identifier(_) => Ok(()),
            Expr::ClassRef(_) | Expr::This if parts > 0 => Ok(()),
            _ => Err(self.error(invalid)),
        }
    }

    /// `condition ? consequent : alternate` or Python's
    /// `consequent if condition else alternate`: one right-associative level
    /// above assignment and below the reasoning pipeline. Both spellings build
    /// the same node. The consequent after `?` is a full expression whose
    /// following `:` belongs to the conditional; the alternate continues in
    /// the enclosing context.
    fn conditional(&mut self) -> Result<Expr> {
        let operand = self.pipeline()?;
        let operand = self.coalescing(operand)?;

        let (condition, consequent) = if self.eat(&Token::Question) {
            let consequent = self.clause_expression()?;
            self.expect(&Token::Colon)?;
            (operand, consequent)
        } else if let Some(condition) = self.python_condition() {
            (condition, operand)
        } else {
            return Ok(operand);
        };

        let alternate = self.nested_expression(Self::conditional)?;
        Ok(Expr::Conditional {
            condition: Box::new(condition),
            consequent: Box::new(consequent),
            alternate: Box::new(alternate),
        })
    }

    /// Qualifies a Python-style conditional after a complete operand: `if`,
    /// one condition,
    /// and an immediately following `else`, which is consumed. Any other
    /// outcome consumes nothing, so the statement parser still reads `if` as
    /// an adjacent statement with its original positions and diagnostics.
    /// Template interpolations keep ignoring a trailing `if`.
    fn python_condition(&mut self) -> Option<Expr> {
        if self.context == ExpressionContext::Interpolation
            || self.peek() != &Token::Keyword(Keyword::If)
        {
            return None;
        }

        let attempt = (self.position, self.depth);
        let outcome = if let Some(outcome) = self.python_conditions.get(&attempt) {
            outcome.clone()
        } else {
            let outcome = self.attempt_python_condition();
            self.python_conditions.insert(attempt, outcome.clone());
            outcome
        };
        let PythonCondition { condition, end } = outcome?;
        self.position = end;
        Some(condition)
    }

    /// Parses one Python-style condition after `if` without keeping any
    /// position, depth or context change.
    fn attempt_python_condition(&mut self) -> Option<PythonCondition> {
        let start = self.position;
        self.advance();
        let condition =
            self.nested_expression_in(ExpressionContext::Clause, Self::coalescing_expression);
        let outcome = match condition {
            Ok(condition) if self.eat_keyword(Keyword::Else) => Some(PythonCondition {
                condition,
                end: self.position,
            }),
            _ => None,
        };
        self.position = start;
        outcome
    }

    fn coalescing_expression(&mut self) -> Result<Expr> {
        let operand = self.pipeline()?;
        self.coalescing(operand)
    }

    fn coalescing(&mut self, first: Expr) -> Result<Expr> {
        let mut lefts = Vec::new();
        let mut last = first;

        while self.eat(&Token::QuestionQuestion) {
            if lefts.len() >= MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let next = self.pipeline()?;
            lefts.push(std::mem::replace(&mut last, next));
        }

        Ok(lefts
            .into_iter()
            .rev()
            .fold(last, |right, left| Expr::Logical {
                operator: LogicalOp::Coalesce,
                left: Box::new(left),
                right: Box::new(right),
            }))
    }

    fn pipeline(&mut self) -> Result<Expr> {
        let mut left = self.logical_or()?;

        let mut chain = 0;

        while self.check(&Token::Pipe) {
            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            if let Expr::Identifier(name) = &left {
                if name == "model" {
                    left = Expr::Model;
                }
            }

            left = self.stage(left)?;
        }

        Ok(left)
    }

    fn stage(&mut self, source: Expr) -> Result<Expr> {
        if let Token::Identifier(name) = self.peek() {
            let stage = match name.as_str() {
                "diff" => Some(Stage::Diff),
                other => Stage::from_name(other),
            };
            if let Some(stage) = stage {
                self.advance();
                return Ok(Expr::Reason {
                    stage,
                    source: Box::new(source),
                });
            }

            if name == "depth" && self.peek_at(1) == &Token::LeftParen {
                self.position += 2;
                let limit = self.expression()?;
                self.expect(&Token::RightParen)?;
                return Ok(Expr::Depth {
                    source: Box::new(source),
                    limit: Box::new(limit),
                });
            }

            return Err(Error::type_error(format!("{name} is not an operation")));
        }

        Err(self.error("Expected an operation"))
    }

    fn logical_or(&mut self) -> Result<Expr> {
        let mut left = self.logical_and()?;

        let mut chain = 0;

        while self.check(&Token::OrOr) || self.check(&Token::Keyword(Keyword::Or)) {
            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let right = self.logical_and()?;
            left = Expr::Logical {
                operator: LogicalOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn logical_and(&mut self) -> Result<Expr> {
        let mut left = self.equality()?;

        let mut chain = 0;

        while self.check(&Token::AndAnd) || self.check(&Token::Keyword(Keyword::And)) {
            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let right = self.equality()?;
            left = Expr::Logical {
                operator: LogicalOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut left = self.relational()?;

        let mut chain = 0;

        loop {
            let operator = match self.peek() {
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                Token::StrictEqual => BinaryOp::StrictEqual,
                Token::StrictNotEqual => BinaryOp::StrictNotEqual,
                _ => break,
            };

            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let right = self.relational()?;
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn relational(&mut self) -> Result<Expr> {
        let mut left = self.additive()?;

        let mut chain = 0;

        loop {
            let operator = match self.peek() {
                Token::Less => BinaryOp::Less,
                Token::LessEqual => BinaryOp::LessEqual,
                Token::Greater => BinaryOp::Greater,
                Token::GreaterEqual => BinaryOp::GreaterEqual,
                _ => break,
            };

            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let right = self.additive()?;
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn additive(&mut self) -> Result<Expr> {
        let mut left = self.multiplicative()?;

        let mut chain = 0;

        loop {
            let operator = match self.peek() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                _ => break,
            };

            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let right = self.multiplicative()?;
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.unary()?;

        let mut chain = 0;

        loop {
            let operator = match self.peek() {
                Token::Star => BinaryOp::Multiply,
                Token::Slash => BinaryOp::Divide,
                Token::Percent => BinaryOp::Modulo,
                _ => break,
            };

            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let right = self.unary()?;
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr> {
        if let Token::Identifier(name) = self.peek() {
            if let Some(stage) = Stage::from_name(name) {
                // An ordinary object value wins before the next explicit
                // field, even on the right of an operator or an arrow.
                let field_boundary = self.context == ExpressionContext::ObjectValue
                    && matches!(stage, Stage::Stated | Stage::Derived)
                    && matches!(self.peek_at(1), Token::Identifier(_))
                    && self.peek_at(2) == &Token::Colon;
                if self.starts_operand(1) && !field_boundary {
                    self.advance();
                    let operand = self.nested_expression(Self::unary)?;

                    return Ok(Expr::Reason {
                        stage,
                        source: Box::new(operand),
                    });
                }
            }
        }

        if let Some(noun) = self.noun_query() {
            self.position += 2;
            let operand = self.nested_expression(Self::unary)?;

            return Ok(Expr::Of {
                noun,
                operand: Box::new(operand),
            });
        }

        let operator = match self.peek() {
            Token::Bang | Token::Keyword(Keyword::Not) => UnaryOp::Not,
            Token::Minus => UnaryOp::Negate,
            Token::Plus => UnaryOp::Plus,
            Token::Keyword(Keyword::TypeOf) => UnaryOp::TypeOf,
            _ => return self.exponent(),
        };

        self.advance();
        let operand = self.nested_expression(Self::unary)?;

        Ok(Expr::Unary {
            operator,
            operand: Box::new(operand),
        })
    }

    fn exponent(&mut self) -> Result<Expr> {
        let base = self.postfix()?;
        if !self.eat(&Token::StarStar) {
            return Ok(base);
        }
        let exponent = self.nested_expression(Self::unary)?;
        Ok(Expr::Binary {
            operator: BinaryOp::Power,
            left: Box::new(base),
            right: Box::new(exponent),
        })
    }

    /// The noun of a `NOUN of OPERAND` query starting here (NUC 75): a
    /// noun-named list query, then `of`. Accepted syntax gives `of` no other
    /// place in an expression; a `for` header takes its loop name before any
    /// expression starts. A template interpolation keeps ignoring trailing
    /// tokens, so `${sum of xs}` still reads `sum`, as quantifiers and
    /// `lambda` stay names there.
    fn noun_query(&self) -> Option<Noun> {
        if self.context == ExpressionContext::Interpolation
            || self.peek_at(1) != &Token::Keyword(Keyword::Of)
        {
            return None;
        }

        match self.peek() {
            Token::Identifier(name) => Noun::from_name(name),
            _ => None,
        }
    }

    fn starts_operand(&self, offset: usize) -> bool {
        matches!(
            self.peek_at(offset),
            Token::Identifier(_) | Token::ClassRef(_)
        )
    }

    fn postfix(&mut self) -> Result<Expr> {
        let mut expression = self.primary()?;
        let mut chain = 0;

        loop {
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many parts"));
            }

            match self.peek() {
                Token::At => {
                    self.advance();
                    expression = Expr::Version {
                        reference: Box::new(expression),
                        selector: self.version_selector()?,
                    };
                }
                Token::Dot => {
                    self.advance();
                    expression = Expr::Member {
                        object: Box::new(expression),
                        property: self.member_property()?,
                    };
                }
                Token::LeftBracket => {
                    self.advance();
                    expression = self.subscript(expression)?;
                }
                Token::LeftParen => {
                    self.advance();
                    // Only direct names can construct classes. Even a
                    // reserved helper name may name a class on the existing
                    // assignment-instantiation route; actual call routing
                    // decides whether a bare assignment is a label.
                    let context = if matches!(&expression, Expr::Identifier(_)) {
                        ArgumentContext::PotentialConstructor
                    } else {
                        ArgumentContext::Ordinary
                    };
                    let arguments = self.arguments(context)?;
                    expression = Expr::Call {
                        callee: Box::new(expression),
                        arguments,
                    };
                }
                _ => break,
            }
        }

        Ok(expression)
    }

    fn version_selector(&mut self) -> Result<VersionSelector> {
        let selector = match self.peek() {
            Token::VersionIndex(index) => VersionSelector::Index(*index),
            Token::Identifier(name) if name == "first" => VersionSelector::First,
            Token::Identifier(name) if name == "last" => VersionSelector::Last,
            Token::Identifier(name) if name == "previous" => VersionSelector::Previous,
            _ => {
                return Err(self.error(
                    "Expected a non-negative integer, 'first', 'last', or 'previous' after '@'",
                ));
            }
        };
        self.advance();
        Ok(selector)
    }

    fn member_property(&mut self) -> Result<String> {
        match self.advance() {
            Token::Identifier(name) => Ok(name),
            Token::Keyword(keyword) => Ok(keyword.as_str().to_string()),
            other => Err(self.error(format!(
                "Expected a property but found {}",
                other.describe()
            ))),
        }
    }

    fn subscript(&mut self, object: Expr) -> Result<Expr> {
        if self.eat(&Token::Colon) {
            let end = if self.check(&Token::RightBracket) {
                None
            } else {
                Some(Box::new(self.expression()?))
            };
            self.expect(&Token::RightBracket)?;
            return Ok(Expr::Slice {
                object: Box::new(object),
                start: None,
                end,
            });
        }

        let index = self.clause_expression()?;

        if self.eat(&Token::Colon) {
            let end = if self.check(&Token::RightBracket) {
                None
            } else {
                Some(Box::new(self.expression()?))
            };
            self.expect(&Token::RightBracket)?;
            return Ok(Expr::Slice {
                object: Box::new(object),
                start: Some(Box::new(index)),
                end,
            });
        }

        self.expect(&Token::RightBracket)?;

        Ok(Expr::Index {
            object: Box::new(object),
            index: Box::new(index),
        })
    }

    fn arguments(&mut self, context: ArgumentContext) -> Result<Vec<Expr>> {
        let mut arguments = Vec::new();

        while !self.check(&Token::RightParen) {
            let length = self.selector_length();
            let argument = if self.starts_selector(length) {
                self.nested_expression(|parser| parser.selector(length))?
            } else {
                self.argument(context)?
            };
            arguments.push(argument);

            if !self.eat(&Token::Comma) {
                break;
            }
        }

        self.expect(&Token::RightParen)?;
        Ok(arguments)
    }

    fn selector_length(&self) -> usize {
        let mut length = 0;
        while length < MAX_CHAIN
            && self.peek_at(2 * length) == &Token::Dot
            && matches!(
                self.peek_at(2 * length + 1),
                Token::Identifier(_) | Token::Keyword(_)
            )
        {
            length += 1;
        }
        length
    }

    fn starts_selector(&self, length: usize) -> bool {
        length == MAX_CHAIN
            || (length > 0 && matches!(self.peek_at(2 * length), Token::Comma | Token::RightParen))
    }

    fn selector(&mut self, length: usize) -> Result<Expr> {
        if length == MAX_CHAIN {
            return Err(Error::syntax("Expression has too many parts")
                .at(self.position_at(2 * (MAX_CHAIN - 1))));
        }
        let mut path = Vec::with_capacity(length);
        for _ in 0..length {
            self.advance();
            path.push(self.member_property()?);
        }
        debug_assert!(matches!(self.peek(), Token::Comma | Token::RightParen));
        Ok(Expr::Function(Function::selector(path)))
    }

    fn position_at(&self, offset: usize) -> Position {
        self.tokens
            .get(self.position + offset)
            .map_or_else(|| self.here(), |spanned| spanned.position)
    }

    fn argument(&mut self, context: ArgumentContext) -> Result<Expr> {
        let bare_label =
            matches!(self.peek(), Token::Identifier(_)) && self.peek_at(1) == &Token::Assign;
        let argument = self.expression()?;
        // Grouping is normally erased by primary(). Preserve it only when
        // erasing it could turn an explicit assignment into a constructor
        // label. Compound assignments need the same escape. All other
        // arguments, including member calls, keep their existing AST shape.
        if matches!(context, ArgumentContext::PotentialConstructor)
            && !bare_label
            && matches!(&argument, Expr::Assign { target, .. }
                if matches!(target.as_ref(), Expr::Identifier(_)))
        {
            Ok(Expr::Unary {
                operator: UnaryOp::Positional,
                operand: Box::new(argument),
            })
        } else {
            Ok(argument)
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        // `@` is only additive postfix syntax. At an operand boundary retain
        // the accepted diagnostic and its original character position.
        if self.check(&Token::At) {
            return Err(self.error("Unexpected character '@'"));
        }
        if self.context != ExpressionContext::Interpolation {
            if let Some(quantifier) = self.quantifier_at(0) {
                return self.quantified(quantifier);
            }
        }
        // `name => ...` or `name -> ...`. Check before taking the name so
        // that ordinary identifier operands are not cloned here.
        if matches!(self.peek(), Token::Identifier(_)) && self.peek_at(1).is_arrow() {
            let parameter = self.untyped_parameter()?;
            self.advance();
            return self.arrow_body(vec![parameter]);
        }

        // `lambda a, b: ...`
        if let Some(count) = self.lambda_parameter_count() {
            return self.lambda(count);
        }

        // `(a, b) => ...` or `(a, b) -> ...`
        if self.check(&Token::LeftParen) && self.is_arrow_parameters() {
            let parameters = self.parameters()?;
            self.expect_arrow()?;
            return self.arrow_body(parameters);
        }

        match self.advance() {
            Token::Number(number) => Ok(Expr::Number(number)),
            Token::Str(string) => Ok(Expr::String(string)),
            Token::Template(template) => self.template(&template),
            Token::Regex(pattern) => Ok(Expr::Regex(pattern)),
            Token::Identifier(name) => Ok(Expr::Identifier(name)),
            Token::ClassRef(name) => Ok(Expr::ClassRef(name)),
            Token::Keyword(Keyword::True) => Ok(Expr::Bool(true)),
            Token::Keyword(Keyword::False) => Ok(Expr::Bool(false)),
            Token::Keyword(Keyword::Null) => Ok(Expr::Null),
            Token::Keyword(Keyword::This) => Ok(Expr::This),
            Token::Keyword(Keyword::New) => self.nested_expression(Self::primary),
            Token::Keyword(Keyword::Delete) => {
                let operand = self.nested_expression(Self::unary)?;
                Ok(Expr::Delete(Box::new(operand)))
            }
            Token::Keyword(Keyword::Super) => {
                // Only an immediate `(` introduces arguments, not separators.
                let arguments = if self.eat(&Token::LeftParen) {
                    self.arguments(ArgumentContext::PotentialConstructor)?
                } else {
                    Vec::new()
                };
                Ok(Expr::Super(arguments))
            }
            Token::Keyword(Keyword::Function) => self.function_literal(),
            Token::LeftParen => {
                let expression = self.expression()?;
                self.expect(&Token::RightParen)?;
                Ok(expression)
            }
            Token::LeftBracket => self.list_literal(),
            Token::LeftBrace => self.object_literal(),
            other => Err(self.error(format!("Unexpected {}", other.describe()))),
        }
    }

    fn function_literal(&mut self) -> Result<Expr> {
        let parameters = self.parameters()?;
        let body = self.braced()?;
        Ok(Expr::Function(Arc::new(Function {
            name: None,
            parameters,
            body: FunctionBody::Block(body),
        })))
    }

    fn list_literal(&mut self) -> Result<Expr> {
        let mut items = Vec::new();

        while !self.check(&Token::RightBracket) {
            items.push(self.expression()?);

            if !self.eat(&Token::Comma) {
                break;
            }
        }

        self.expect(&Token::RightBracket)?;
        Ok(Expr::List(items))
    }

    fn object_literal(&mut self) -> Result<Expr> {
        let mut entries = Vec::new();

        loop {
            self.skip_newlines();

            if self.eat(&Token::RightBrace) {
                break;
            }

            entries.push(self.object_entry()?);

            self.skip_newlines();

            if !self.eat(&Token::Comma) {
                self.skip_newlines();
                // A complete ordinary value wins before considering the next
                // explicit field. Keep shorthand and key parsing unchanged.
                if matches!(
                    self.peek(),
                    Token::Str(_) | Token::Identifier(_) | Token::Number(_)
                ) && self.peek_at(1) == &Token::Colon
                {
                    continue;
                }
                self.expect(&Token::RightBrace)?;
                break;
            }
        }

        Ok(object_expression(entries))
    }

    fn object_entry(&mut self) -> Result<ObjectEntry> {
        if self.starts_rest_parameter() {
            self.position += 3;
            let source = self.expression_in(ExpressionContext::ObjectValue)?;
            return Ok(ObjectEntry::Spread(source));
        }

        let (key, is_identifier) = match self.advance() {
            Token::Str(key) => (key, false),
            Token::Identifier(key) => (key, true),
            Token::Number(number) => (crate::value::format_number(number), false),
            other => {
                return Err(self.error(format!("Expected a key but found {}", other.describe())));
            }
        };

        let mut offset = 0;
        while matches!(self.peek_at(offset), Token::Newline | Token::Semicolon) {
            offset += 1;
        }
        let value =
            if is_identifier && matches!(self.peek_at(offset), Token::Comma | Token::RightBrace) {
                Expr::Identifier(key.clone())
            } else {
                self.expect(&Token::Colon)?;
                self.expression_in(ExpressionContext::ObjectValue)?
            };
        Ok(ObjectEntry::Property(key, value))
    }

    fn arrow_body(&mut self, parameters: Vec<Parameter>) -> Result<Expr> {
        let body = if self.check(&Token::LeftBrace) {
            FunctionBody::Block(self.braced()?)
        } else {
            FunctionBody::Expression(Box::new(self.expression_in(self.context)?))
        };

        Ok(Expr::Function(Arc::new(Function {
            name: None,
            parameters,
            body,
        })))
    }

    /// The parameter count of a `lambda` expression starting here, qualified
    /// without consuming tokens or allocating names. Only an ordinary
    /// identifier spelled `lambda`, then nothing or comma-separated ordinary
    /// names, then a `:` qualifies; `lambda` is not reserved.
    ///
    /// The name stays an ordinary identifier wherever accepted syntax owns
    /// that colon: a clause header, subscript start, conditional consequent or
    /// template interpolation; a single name in an object value, where
    /// `lambda next:` is a comma-free property boundary; and a single name
    /// whose colon starts a statement, which statement loops accept directly
    /// after the expression ending at `lambda`: a typed declaration
    /// (`lambda x: int`) or an assertion statement (`lambda assert: x, 1`).
    fn lambda_parameter_count(&self) -> Option<usize> {
        if matches!(
            self.context,
            ExpressionContext::Clause | ExpressionContext::Interpolation
        ) || !matches!(self.peek(), Token::Identifier(name) if name == "lambda")
        {
            return None;
        }

        let mut count = 0;
        let mut colon = 1;
        if self.peek_at(colon) != &Token::Colon {
            loop {
                if !matches!(self.peek_at(colon), Token::Identifier(_)) {
                    return None;
                }
                count += 1;
                colon += 1;
                match self.peek_at(colon) {
                    Token::Comma => colon += 1,
                    Token::Colon => break,
                    _ => return None,
                }
            }
        }

        // A single name and its colon keep every accepted reading: the next
        // comma-free property, or a declaration or assertion statement.
        let accepted_colon = count == 1
            && (self.context == ExpressionContext::ObjectValue
                || self.starts_colon_statement_at(1));
        (!accepted_colon).then_some(count)
    }

    /// Parses a lambda that [`Self::lambda_parameter_count`] qualified. It
    /// lowers to exactly the anonymous `(names) => (expression)` function:
    /// untyped parameters and an expression body, where braces are an object.
    fn lambda(&mut self, count: usize) -> Result<Expr> {
        self.advance();
        let mut parameters = Vec::with_capacity(count);
        for index in 0..count {
            if index > 0 {
                self.expect(&Token::Comma)?;
            }
            parameters.push(self.untyped_parameter()?);
        }
        self.expect(&Token::Colon)?;
        let body = self.expression_in(self.context)?;

        Ok(Expr::Function(Arc::new(Function {
            name: None,
            parameters,
            body: FunctionBody::Expression(Box::new(body)),
        })))
    }

    fn quantifier_at(&self, offset: usize) -> Option<Quantifier> {
        Quantifier::starting(
            self.peek_at(offset),
            self.peek_at(offset + 1),
            self.peek_at(offset + 2),
        )
    }

    fn quantified(&mut self, quantifier: Quantifier) -> Result<Expr> {
        self.advance();
        let binder = match self.advance() {
            Token::Identifier(binder) => binder,
            other => {
                return Err(self.error(format!("Expected a name but found {}", other.describe())));
            }
        };
        self.advance();
        let collection = self.clause_expression()?;
        self.expect(&Token::Colon)?;
        let condition = self.expression_in(self.context)?;

        Ok(Expr::Quantified {
            quantifier,
            binder,
            collection: Box::new(collection),
            condition: Box::new(condition),
        })
    }

    /// Looks ahead past a balanced `( ... )` for an arrow, which distinguishes
    /// an arrow parameter list from a parenthesised expression.
    fn is_arrow_parameters(&self) -> bool {
        let mut depth = 0usize;
        let mut offset = 0usize;

        loop {
            match self.peek_at(offset) {
                Token::LeftParen | Token::LeftBracket => depth += 1,
                Token::RightParen | Token::RightBracket => {
                    depth -= 1;
                    if depth == 0 {
                        return self.peek_at(offset + 1).is_arrow();
                    }
                }
                Token::Eof => return false,
                _ => {}
            }

            offset += 1;
        }
    }

    fn template(&mut self, raw: &str) -> Result<Expr> {
        let origin = self.previous_position();
        let mut parts = Vec::new();
        let mut literal = String::new();
        let mut characters = raw.char_indices().peekable();

        while let Some((index, character)) = characters.next() {
            if character == '\\' {
                if let Some((_, escaped)) = characters.next() {
                    literal.push(match escaped {
                        'n' => '\n',
                        't' => '\t',
                        other => other,
                    });
                }
                continue;
            }

            if character != '$' || characters.peek().map(|(_, next)| *next) != Some('{') {
                literal.push(character);
                continue;
            }

            let start = index + 2;
            let Some(end) = self.interpolation_close(raw, start, origin)? else {
                return Err(self.error("Unterminated template expression"));
            };

            if !literal.is_empty() {
                parts.push(TemplatePart::Literal(std::mem::take(&mut literal)));
            }

            parts.push(TemplatePart::Expression(self.interpolation(
                raw,
                start..end,
                origin,
            )?));

            while let Some((offset, _)) = characters.peek() {
                if *offset <= end {
                    characters.next();
                } else {
                    break;
                }
            }
        }

        if !literal.is_empty() {
            parts.push(TemplatePart::Literal(literal));
        }

        Ok(Expr::Template(parts))
    }

    fn previous_position(&self) -> Position {
        self.position
            .checked_sub(1)
            .and_then(|previous| self.tokens.get(previous))
            .map_or_else(|| self.here(), |spanned| spanned.position)
    }

    fn interpolation_close(
        &self,
        raw: &str,
        start: usize,
        origin: Position,
    ) -> Result<Option<usize>> {
        let body = raw.get(start..).unwrap_or_default();
        if !self.options.c_style_comments {
            return Ok(default_interpolation_end(body).map(|offset| start + offset));
        }
        Ok(interpolation_end(body, self.options)
            .map_err(|error| template_error(error, origin, excerpt(raw, 0..start)))?
            .map(|offset| start + offset))
    }

    fn interpolation(&self, raw: &str, span: Range<usize>, origin: Position) -> Result<Expr> {
        let prefix = excerpt(raw, 0..span.start);
        // Trailing tokens after an interpolated expression are ignored,
        // so a top-level `lambda` keeps its accepted name reading here
        // and a trailing Python-style `if` stays ignored.
        parse_expression_in(
            excerpt(raw, span),
            ExpressionContext::Interpolation,
            self.options,
        )
        .map_err(|error| {
            if self.options.c_style_comments {
                template_error(error, origin, prefix)
            } else {
                error
            }
        })
    }
}

fn default_interpolation_end(body: &str) -> Option<usize> {
    // Preserve the accepted default interpolation boundary rule.
    let mut depth = 1usize;
    for (offset, candidate) in body.char_indices() {
        match candidate {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(offset);
                }
            }
            _ => {}
        }
    }
    None
}

/// Secondary parsers report relative positions; opt-in diagnostics retain the
/// location in the caller's original template, including Unicode and newlines.
fn template_error(error: Error, origin: Position, prefix: &str) -> Error {
    let Some(relative) = error.position() else {
        return error;
    };
    let newlines = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let column = match prefix.rsplit_once('\n') {
        Some((_, tail)) => tail.chars().count() + 1,
        None => origin.column + 1 + prefix.chars().count(),
    };
    error.at(Position::new(
        origin.line + newlines + relative.line - 1,
        if relative.line == 1 {
            column + relative.column - 1
        } else {
            relative.column
        },
    ))
}

fn empty_class(name: String, parent: Option<String>, parameters: Vec<Parameter>) -> ClassDecl {
    ClassDecl {
        name,
        parent,
        parameters,
        fields: Vec::new(),
        constructor: Vec::new(),
        methods: Vec::new(),
    }
}

fn object_expression(entries: Vec<ObjectEntry>) -> Expr {
    if entries
        .iter()
        .any(|entry| matches!(entry, ObjectEntry::Spread(_)))
    {
        return Expr::SpreadObject(entries);
    }
    Expr::ObjectLiteral(
        entries
            .into_iter()
            .filter_map(|entry| match entry {
                ObjectEntry::Property(key, value) => Some((key, value)),
                ObjectEntry::Spread(_) => None,
            })
            .collect(),
    )
}

fn simple_statement(expression: Expr) -> Stmt {
    match expression {
        Expr::Assign { target, value } => Stmt::Assign {
            target: *target,
            value: *value,
        },
        other => Stmt::Expression(other),
    }
}

fn names_suppose(expression: &Expr) -> bool {
    matches!(expression, Expr::Identifier(name) if name == "suppose")
}

fn heads_suppose(expression: &Expr) -> bool {
    match expression {
        Expr::Assign { value, .. } => names_suppose(value),
        _ => names_suppose(expression),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::ast::generator::{generate, generate_all};

    #[test]
    fn direct_condition_lookahead_requires_balanced_header_and_real_suite_without_consuming() {
        for (header, qualifies) in [
            ("ready:\n    pass", true),
            ("not ready and available:\n    pass", true),
            ("items[1:3] == [2, 3]:\n    pass", true),
            ("choose({ready: items[:]}, (x: int) => x):\n    pass", true),
            ("({ready: {nested: true}}).ready:\n    pass", true),
            ("(\n ready\n):\n    pass", true),
            ("ready: # header\n\n# blank\n    # body\n    pass", true),
            ("ready:\r\n\tpass", true),
            ("ready:\n;\n    pass", true),
            ("ready:\n    ;\n    pass", true),
            ("ready\n    pass", true),
            ("items[1:3] == [2, 3]\n    pass", true),
            ("choose({ready: items[:]}, (x: int) => x)\r\n\tpass", true),
            ("ready # header\n\n# blank\n    # body\n    pass", true),
            ("ready\n;\n    pass", true),
            ("ready\n    ;\n    pass", true),
            // A top-level conditional expression's colon pairs with its `?`.
            ("ready ? on : off:\n    pass", true),
            ("ready ? on : off\n    pass", true),
            ("a ? b ? c : d : e:\n    pass", true),
            ("a ? b : c ? d : e\n    pass", true),
            ("(ready ? on : off):\n    pass", true),
            ("on if ready else off:\n    pass", true),
            ("ready ? (x: int) => x : off:\n    pass", true),
            // Qualification is not expression validation. Real parsing must
            // report these malformed expressions rather than fall back.
            ("ready +:\n    pass", true),
            ("():\n    pass", true),
            ("pass:\n    pass", true),
            ("ready +\n    pass", true),
            ("ready ? on:\n    pass", true),
            ("ready ?\n    pass", true),
            ("(ready ? on):\n    pass", true),
            (":\n    pass", false),
            ("\n    pass", false),
            ("ready: pass", false),
            ("ready: { pass }", false),
            ("ready { pass }", false),
            ("ready:;\n    pass", false),
            ("ready:\npass", false),
            ("ready:\n# no body", false),
            ("ready:\n    # no body", false),
            ("ready:", false),
            ("ready", false),
            ("ready;\n    pass", false),
            ("ready\npass", false),
            ("ready\n    # no body", false),
            ("(ready]\n    pass", false),
            ("{ready: true\n    pass", false),
            ("ready\n:\n    pass", false),
            ("ready; other:\n    pass", false),
            ("ready # boundary\nother:\n    pass", false),
            ("items[:]\nother:\n    pass", false),
            ("(ready]:\n    pass", false),
            ("[ready):\n    pass", false),
            ("{ready: true]:\n    pass", false),
            ("ready):\n    pass", false),
            ("(ready:\n    pass", false),
            ("{ready: true:\n    pass", false),
            ("ready ? on : off: pass", false),
            ("ready ? on : off", false),
            ("ready ? on : off:", false),
            ("ready ? on", false),
            ("ready ? on : off; other:\n    pass", false),
        ] {
            let mut parser = Parser::new(tokenize(&format!("else {header}")).unwrap());
            parser.expect(&Token::Keyword(Keyword::Else)).unwrap();
            let position = parser.position;
            let tokens = parser.tokens.clone();
            assert_eq!(parser.has_direct_condition_suite(), qualifies, "{header}");
            assert_eq!(parser.position, position, "{header}");
            assert_eq!(parser.depth, 0, "{header}");
            assert_eq!(parser.tokens, tokens, "{header}");
        }
    }

    #[test]
    fn direct_condition_lookahead_pairs_quantifier_colons_without_consuming() {
        for (header, qualifies) in [
            ("all v in xs: v > 0:\n    pass", true),
            ("all v in xs: v > 0\n    pass", true),
            ("any v in xs: v ? on : off:\n    pass", true),
            ("ready ? all v in xs: v : off:\n    pass", true),
            ("all t in ts: any w in ws: w == t:\n    pass", true),
            ("all v in ready ? xs : ys: v\n    pass", true),
            ("all v in xs: v ?? false:\n    pass", true),
            ("(all v in xs: v):\n    pass", true),
            ("all v in xs:\n    pass", true),
            ("all v in xs: v: pass", false),
            ("all v in xs: v", false),
            ("all v in xs: v;\n    pass", false),
            ("all v of xs: v:\n    pass", false),
            ("all: v\n    pass", false),
        ] {
            let mut parser = Parser::new(tokenize(&format!("else {header}")).unwrap());
            parser.expect(&Token::Keyword(Keyword::Else)).unwrap();
            let position = parser.position;
            let tokens = parser.tokens.clone();
            assert_eq!(parser.has_direct_condition_suite(), qualifies, "{header}");
            assert_eq!(parser.position, position, "{header}");
            assert_eq!(parser.depth, 0, "{header}");
            assert_eq!(parser.tokens, tokens, "{header}");
        }
    }

    #[test]
    fn direct_condition_lookahead_stops_at_a_top_level_value_assignment() {
        for (header, qualifies) in [
            ("ready := 1:\n    pass", false),
            ("ready := 1\n    pass", false),
            ("x := ready ? 1 : 2:\n    pass", false),
            ("f(x := 1):\n    pass", true),
            ("ready:\n    x := 1", true),
            ("ready\n    x := 1", true),
        ] {
            let mut parser = Parser::new(tokenize(&format!("else {header}")).unwrap());
            parser.expect(&Token::Keyword(Keyword::Else)).unwrap();
            let position = parser.position;
            let tokens = parser.tokens.clone();
            assert_eq!(parser.has_direct_condition_suite(), qualifies, "{header}");
            assert_eq!(parser.position, position, "{header}");
            assert_eq!(parser.depth, 0, "{header}");
            assert_eq!(parser.tokens, tokens, "{header}");
        }
    }

    #[test]
    fn direct_condition_lookahead_reuses_immediate_indent_guard_and_stops_at_layout() {
        let mut parser = Parser::new(tokenize("ready:\n    pass").unwrap());
        assert!(parser.has_direct_condition_suite());
        parser.tokens.remove(2);
        assert_eq!(parser.peek_at(2), &Token::Indent);
        assert!(parser.has_direct_condition_suite());
        parser.tokens.insert(
            2,
            Spanned {
                token: Token::Semicolon,
                position: Position::new(1, 7),
            },
        );
        assert!(!parser.has_direct_condition_suite());
        assert_eq!(parser.position, 0);
        for layout in [
            vec![Token::Newline, Token::Indent],
            vec![Token::Indent],
            vec![Token::Newline, Token::Semicolon, Token::Indent],
        ] {
            parser.tokens = std::iter::once(Token::Identifier("ready".into()))
                .chain(layout)
                .chain([Token::Keyword(Keyword::Pass), Token::Eof])
                .map(|token| Spanned {
                    token,
                    position: Position::new(1, 1),
                })
                .collect();
            let tokens = parser.tokens.clone();
            assert!(parser.has_direct_condition_suite());
            assert_eq!(parser.position, 0);
            assert_eq!(parser.depth, 0);
            assert_eq!(parser.tokens, tokens);
        }
        for boundary in [
            Token::Newline,
            Token::Semicolon,
            Token::Indent,
            Token::Dedent,
            Token::Eof,
        ] {
            let qualifies = boundary == Token::Indent;
            parser.tokens = [
                Token::Identifier("ready".into()),
                boundary,
                Token::Colon,
                Token::Newline,
                Token::Indent,
                Token::Keyword(Keyword::Pass),
                Token::Eof,
            ]
            .into_iter()
            .map(|token| Spanned {
                token,
                position: Position::new(1, 1),
            })
            .collect();
            let tokens = parser.tokens.clone();
            assert_eq!(parser.has_direct_condition_suite(), qualifies);
            assert_eq!(parser.position, 0);
            assert_eq!(parser.tokens, tokens);
        }
    }

    #[test]
    fn comma_free_arrow_lookahead_qualifies_the_whole_prefix_without_consuming() {
        for (header, end) in [
            ("left right => left", Some(2)),
            ("left right factor => left", Some(3)),
            ("left\tright=>left", Some(2)),
            ("left => left", None),
            ("left, right => left", None),
            ("left: int, right => left", None),
            ("left: int right => left", None),
            ("left right: int => left", None),
            ("left right, third => left", None),
            ("left, right third => left", None),
            ("left right:\n    return left", None),
            ("left right\n=> left", None),
            ("left # boundary\nright => left", None),
            ("left right; => left", None),
            ("left this => left", None),
            ("left right", None),
        ] {
            let mut parser = Parser::new(tokenize(&format!("def f {header}")).unwrap());
            parser.expect(&Token::Keyword(Keyword::Def)).unwrap();
            assert_eq!(parser.advance(), Token::Identifier("f".to_string()));
            let original = parser.position;
            assert_eq!(parser.comma_free_arrow_end(), end, "{header}");
            assert_eq!(parser.position, original, "{header}");
            assert_eq!(parser.peek(), &Token::Identifier("left".to_string()));
        }
    }

    #[test]
    fn comma_free_block_lookahead_requires_the_whole_header_and_real_suite_without_consuming() {
        for (header, end) in [
            ("left right:\n    return left", Some(2)),
            ("left right factor:\n    return left", Some(3)),
            ("left\tright:\r\n\treturn left", Some(2)),
            ("same same:\n    return same", Some(2)),
            (
                "type value _private A1 why affects model:\n    pass",
                Some(7),
            ),
            (
                "left right: # header\n\n# blank\n    # body\n    pass",
                Some(2),
            ),
            ("left right:\n;\n    pass", Some(2)),
            ("left right:\n    ;\n    pass", Some(2)),
            ("left right:\n    int => left", Some(2)),
            ("left:\n    pass", None),
            ("left, right:\n    pass", None),
            ("left: int, right:\n    pass", None),
            ("left: int right:\n    pass", None),
            ("left right: int:\n    pass", None),
            ("left: int right: int:\n    pass", None),
            ("left right, third:\n    pass", None),
            ("left, right third:\n    pass", None),
            ("left right,:\n    pass", None),
            ("left right third, fourth:\n    pass", None),
            ("left right => left", None),
            ("left right", None),
            ("left right:", None),
            ("left right:\n", None),
            ("left right:\n    # no body", None),
            ("left right:\nreturn left", None),
            ("left right:\n;\nreturn left", None),
            ("left right:;\n    pass", None),
            ("left right: return left", None),
            ("left right: { return left }", None),
            ("left right:\n{ return left }", None),
            ("left right { return left }", None),
            ("left right\n:\n    pass", None),
            ("left\nright:\n    pass", None),
            ("left right;:\n    pass", None),
            ("left; right:\n    pass", None),
            ("left right # boundary\n:\n    pass", None),
            ("left # boundary\nright:\n    pass", None),
            ("left this:\n    pass", None),
            ("left self:\n    pass", None),
            ("left null:\n    pass", None),
            ("left $Right:\n    pass", None),
            ("left 1:\n    pass", None),
            ("left \"right\":\n    pass", None),
            ("left = 1 right:\n    pass", None),
            ("left right = 1:\n    pass", None),
            ("left ...right:\n    pass", None),
            ("left [right]:\n    pass", None),
            ("left {right}:\n    pass", None),
            ("left right -> int:\n    pass", None),
            ("this right:\n    pass", None),
            ("$Left right:\n    pass", None),
            ("1 right:\n    pass", None),
            ("(left right):\n    pass", None),
            (":\n    pass", None),
        ] {
            let mut parser = Parser::new(tokenize(&format!("def f {header}")).unwrap());
            parser.expect(&Token::Keyword(Keyword::Def)).unwrap();
            assert_eq!(parser.advance(), Token::Identifier("f".to_string()));
            let position = parser.position;
            let tokens = parser.tokens.clone();
            assert_eq!(parser.comma_free_block_end(), end, "{header}");
            assert_eq!(parser.position, position, "{header}");
            assert_eq!(parser.depth, 0, "{header}");
            assert_eq!(parser.tokens, tokens, "{header}");
        }
    }

    #[test]
    fn lambda_lookahead_qualifies_whole_headers_by_context_without_consuming() {
        use ExpressionContext::{Clause, Interpolation, ObjectValue, Ordinary};
        for (source, context, count) in [
            ("lambda: 1", Ordinary, Some(0)),
            ("lambda: value", Ordinary, Some(0)),
            ("lambda x: x + 1", Ordinary, Some(1)),
            ("lambda x: x.name", Ordinary, Some(1)),
            ("lambda x: x)", Ordinary, Some(1)),
            ("lambda x: x }", Ordinary, Some(1)),
            ("lambda x: int: x", Ordinary, Some(1)),
            ("lambda x: 1", Ordinary, Some(1)),
            ("lambda a, b: a", Ordinary, Some(2)),
            ("lambda a, b, c: a", Ordinary, Some(3)),
            ("lambda a,\tb : a", Ordinary, Some(2)),
            // A single name, a colon and one name before a statement boundary
            // is an expression followed by a typed declaration.
            ("lambda x: x", Ordinary, None),
            ("lambda x: int\n", Ordinary, None),
            ("lambda x: int; other", Ordinary, None),
            ("lambda x: int # comment", Ordinary, None),
            ("lambda a, b: int", Ordinary, Some(2)),
            ("lambda: int", Ordinary, Some(0)),
            // NUC45: a single `assert` and its colon start an assertion
            // statement after the expression, whatever follows the colon.
            ("lambda assert: x, 1", Ordinary, None),
            ("lambda assert: 1, 1", Ordinary, None),
            ("lambda assert: x", Ordinary, None),
            ("lambda assert: int", Ordinary, None),
            ("lambda assert:", Ordinary, None),
            ("lambda assert : x, 1", Ordinary, None),
            ("lambda assert: x, 1", ObjectValue, None),
            ("lambda assert: x, 1", Clause, None),
            ("lambda assert, b: 1", Ordinary, Some(2)),
            ("lambda a, assert: 1", Ordinary, Some(2)),
            ("lambda Assert: x, 1", Ordinary, Some(1)),
            ("lambda asserts: x, 1", Ordinary, Some(1)),
            // NUC44: one name and a colon in an object value is a property.
            ("lambda x: 1", ObjectValue, None),
            ("lambda x: x", ObjectValue, None),
            ("lambda: 1", ObjectValue, Some(0)),
            ("lambda a, b: 1", ObjectValue, Some(2)),
            // Clause headers, subscript starts, conditional consequents and
            // interpolations own it.
            ("lambda: 1", Clause, None),
            ("lambda x: 1", Clause, None),
            ("lambda a, b: 1", Clause, None),
            ("lambda: 1", Interpolation, None),
            ("lambda x: 1", Interpolation, None),
            ("lambda a, b: 1", Interpolation, None),
            ("lambda", Ordinary, None),
            ("lambda x", Ordinary, None),
            ("lambda x, : x", Ordinary, None),
            ("lambda x,: x", Ordinary, None),
            ("lambda , x: x", Ordinary, None),
            ("lambda x y: x", Ordinary, None),
            ("lambda x = 1: x", Ordinary, None),
            ("lambda (x): x", Ordinary, None),
            ("lambda [x]: x", Ordinary, None),
            ("lambda *x: x", Ordinary, None),
            ("lambda this: 1", Ordinary, None),
            ("lambda self: 1", Ordinary, None),
            ("lambda null: 1", Ordinary, None),
            ("lambda $Type: 1", Ordinary, None),
            ("lambda 1: 1", Ordinary, None),
            ("lambda \"x\": 1", Ordinary, None),
            ("lambda => 1", Ordinary, None),
            ("lambda -> 1", Ordinary, None),
            ("lambda(x)", Ordinary, None),
            ("lambda.x", Ordinary, None),
            ("lambda[0]", Ordinary, None),
            ("lambda = 1", Ordinary, None),
            ("lambda\n: 1", Ordinary, None),
            ("lambda x\n: 1", Ordinary, None),
            ("lambda; x: 1", Ordinary, None),
            ("Lambda: 1", Ordinary, None),
            ("lambdas: 1", Ordinary, None),
            ("x: 1", Ordinary, None),
        ] {
            let mut parser = Parser::new(tokenize(source).unwrap());
            parser.context = context;
            let tokens = parser.tokens.clone();
            assert_eq!(
                parser.lambda_parameter_count(),
                count,
                "{source} in {context:?}"
            );
            assert_eq!(parser.position, 0, "{source}");
            assert_eq!(parser.depth, 0, "{source}");
            assert_eq!(parser.tokens, tokens, "{source}");
        }
    }

    #[test]
    fn noun_query_lookahead_qualifies_a_noun_and_of_by_context_without_consuming() {
        use ExpressionContext::{Clause, Interpolation, ObjectValue, Ordinary};
        for (source, context, noun) in [
            ("sum of xs", Ordinary, Some(Noun::Sum)),
            ("max of xs", Ordinary, Some(Noun::Max)),
            ("min of xs", Ordinary, Some(Noun::Min)),
            ("mode of xs", Ordinary, Some(Noun::Mode)),
            ("mode of", Ordinary, Some(Noun::Mode)),
            ("sum of of", Ordinary, Some(Noun::Sum)),
            ("sum of xs", Clause, Some(Noun::Sum)),
            ("sum of xs", ObjectValue, Some(Noun::Sum)),
            // An interpolation ignores trailing tokens, so a leading noun
            // stays the name it was at the accepted revision.
            ("sum of xs", Interpolation, None),
            ("sum", Ordinary, None),
            ("sum = 1", Ordinary, None),
            ("sum.of", Ordinary, None),
            ("sum(of)", Ordinary, None),
            ("sum\nof xs", Ordinary, None),
            ("sum in xs", Ordinary, None),
            ("count of xs", Ordinary, None),
            ("distinct of xs", Ordinary, None),
            ("sort of xs", Ordinary, None),
            ("Sum of xs", Ordinary, None),
            ("sums of xs", Ordinary, None),
            ("$sum of xs", Ordinary, None),
            ("\"sum\" of xs", Ordinary, None),
            ("of xs", Ordinary, None),
        ] {
            let mut parser = Parser::new(tokenize(source).unwrap());
            parser.context = context;
            let tokens = parser.tokens.clone();
            assert_eq!(parser.noun_query(), noun, "{source} in {context:?}");
            assert_eq!(parser.position, 0, "{source}");
            assert_eq!(parser.depth, 0, "{source}");
            assert_eq!(parser.tokens, tokens, "{source}");
        }
    }

    #[test]
    fn noun_queries_take_a_unary_operand_while_loop_headers_keep_their_names() {
        let mut parser = Parser::new(tokenize("sum of xs.map(f) + 1").unwrap());
        let expression = parser.expression().unwrap();
        assert_eq!(parser.depth, 0);
        assert_eq!(parser.context, ExpressionContext::Ordinary);
        assert_eq!(expression.to_string(), "sum of xs.map(f)+1");
        let Expr::Binary { left, .. } = &expression else {
            panic!("the sum is the left operand: {expression:?}");
        };
        assert!(matches!(
            left.as_ref(),
            Expr::Of {
                noun: Noun::Sum,
                operand,
            } if matches!(operand.as_ref(), Expr::Call { .. })
        ));
        assert_eq!(
            generate_all(&parse("for sum of xs:\n    pass").unwrap()),
            "for sum of xs"
        );
        assert_eq!(
            generate_all(&parse("for k, max of rec:\n    pass").unwrap()),
            "for k,max of rec"
        );
        assert_eq!(
            generate_all(&parse("for x of mode of rows:\n    pass").unwrap()),
            "for x of mode of rows"
        );
    }

    #[test]
    fn colon_statement_lookahead_uses_the_statement_parser_guards() {
        for (source, offset, starts) in [
            ("value: int", 0, true),
            ("value: int\nnext", 0, true),
            ("value: int; next", 0, true),
            ("value: int # comment", 0, true),
            ("assert: value, 1", 0, true),
            ("assert : value, 1", 0, true),
            ("assert: int", 0, true),
            ("assert: value", 0, true),
            ("assert:", 0, true),
            ("lambda x: int", 1, true),
            ("lambda assert: x, 1", 1, true),
            ("value: 1", 0, false),
            ("value: int other", 0, false),
            ("value: int }", 0, false),
            ("value: $Type", 0, false),
            ("value", 0, false),
            ("value, other: int", 0, false),
            ("Assert: value, 1", 0, false),
            ("asserts: value, 1", 0, false),
            ("assert(value, 1)", 0, false),
            ("assert = 1", 0, false),
            (": int", 0, false),
            ("this: int", 0, false),
            ("lambda x: 1", 1, false),
            ("lambda x: x + 1", 1, false),
        ] {
            let parser = Parser::new(tokenize(source).unwrap());
            assert_eq!(
                parser.starts_colon_statement_at(offset),
                starts,
                "{source} at {offset}"
            );
            assert_eq!(parser.position, 0, "{source}");
        }
        // The `assert:` branch of the statement parser uses the same guard.
        let statements = parse("assert: value, 1\nassert : value, 1").unwrap();
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0], statements[1]);
        assert_eq!(statements[0], parse("assert(value, 1)").unwrap()[0]);
        // The declaration guard checks the leading name and its colon itself,
        // so neither its branches nor the lambda scan recheck them.
        for (source, declares) in [
            ("value: int", true),
            ("value: int\nnext", true),
            ("value other int", false),
            ("value = int", false),
            ("1: int", false),
            ("this: int", false),
        ] {
            let parser = Parser::new(tokenize(source).unwrap());
            assert_eq!(parser.is_type_declaration_at(0), declares, "{source}");
        }
        assert_eq!(
            parse("value: int").unwrap(),
            [Stmt::Declaration {
                name: "value".into(),
                type_name: "int".into(),
            }]
        );
        let statements = parse("class Holder:\n    value: int").unwrap();
        let [Stmt::Class(class)] = statements.as_slice() else {
            panic!("one class with a typed field");
        };
        assert_eq!(class.constructor, []);
        let [field] = class.fields.as_slice() else {
            panic!("one typed field");
        };
        assert_eq!(field.name, "value");
        assert_eq!(field.type_name.as_deref(), Some("int"));
    }

    #[test]
    fn python_conditions_consume_nothing_unless_else_follows_and_reuse_outcomes() {
        for (source, qualifies) in [
            ("a if b else c", true),
            ("a if b elif c", true),
            ("a if not b or c else d", true),
            ("a if lambda else c", true),
            ("a if x => x else c", true),
            ("a if (b if c else d) else e", true),
            ("a if b: c", false),
            ("a if b\n    c", false),
            ("a if b; c", false),
            ("a if b c else d", false),
            ("a if b", false),
            ("a if (b else c", false),
            ("a if lambda: 1 else c", false),
            ("a if b ? c : d else e", false),
            ("a if b if c else d else e", false),
            ("a if b = 1 else c", false),
            ("a if", false),
        ] {
            let mut parser = Parser::new(tokenize(source).unwrap());
            parser.advance();
            let start = parser.position;
            let tokens = parser.tokens.clone();
            let first = parser.python_condition();
            assert_eq!(first.is_some(), qualifies, "{source}");
            if !qualifies {
                assert_eq!(parser.position, start, "{source}");
            }
            assert_eq!(parser.depth, 0, "{source}");
            assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
            let recorded = parser.python_conditions.len();
            assert!(
                parser.python_conditions.contains_key(&(start, 0)),
                "{source}"
            );
            let end = parser.position;
            parser.position = start;
            assert_eq!(parser.python_condition(), first, "{source}");
            assert_eq!(parser.position, end, "{source}");
            assert_eq!(parser.python_conditions.len(), recorded, "{source}");
            assert_eq!(parser.tokens, tokens, "{source}");
        }

        let mut parser = Parser::new(tokenize("a if b else c").unwrap());
        parser.advance();
        parser.context = ExpressionContext::Interpolation;
        assert_eq!(parser.python_condition(), None);
        assert_eq!(parser.position, 1);
        assert!(parser.python_conditions.is_empty());
    }

    #[test]
    fn suppose_premises_consume_nothing_unless_a_colon_follows_and_reuse_outcomes() {
        for (source, premise) in [
            ("suppose reading = 95: alarm", Some("reading=95")),
            ("suppose not ready: y", Some("!ready")),
            ("suppose x = c ? 1 : 2: x", Some("x=(c?1:2)")),
            (
                "suppose ok = all v in xs: v > 0: ok",
                Some("ok=(all v in xs:v>0)"),
            ),
            ("suppose Item(5): Item.length", Some("Item(5)")),
            ("suppose $Order.fee = 0: fees", Some("$Order.fee=0")),
            ("suppose delete x: y", Some("delete x")),
            ("suppose reading = 95\n    reading = 50", None),
            ("suppose reading = 95 { reading = 50 }", None),
            ("suppose f = lambda: 1", None),
            ("suppose x.lambda: 1", None),
            ("suppose a b: c", None),
            ("suppose (reading: 95", None),
        ] {
            let mut parser = Parser::new(tokenize(source).unwrap());
            parser.advance();
            let start = parser.position;
            let tokens = parser.tokens.clone();
            let first = parser.suppose_premise();
            assert_eq!(first.as_ref().map(generate).as_deref(), premise, "{source}");
            if first.is_some() {
                assert_eq!(parser.peek(), &Token::Colon, "{source}");
            } else {
                assert_eq!(parser.position, start, "{source}");
            }
            assert_eq!(parser.depth, 0, "{source}");
            assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
            let recorded = parser.suppose_premises.len();
            assert!(
                parser.suppose_premises.contains_key(&(start, 0)),
                "{source}"
            );
            let end = parser.position;
            parser.position = start;
            assert_eq!(parser.suppose_premise(), first, "{source}");
            assert_eq!(parser.position, end, "{source}");
            assert_eq!(parser.suppose_premises.len(), recorded, "{source}");
            assert_eq!(parser.tokens, tokens, "{source}");
        }

        for source in [
            "suppose x: int",
            "suppose assert: x, 1",
            "suppose",
            "suppose\nx",
            "suppose; x",
        ] {
            let mut parser = Parser::new(tokenize(source).unwrap());
            parser.advance();
            assert_eq!(parser.suppose_premise(), None, "{source}");
            assert_eq!(parser.position, 1, "{source}");
            assert!(parser.suppose_premises.is_empty(), "{source}");
        }
    }

    #[test]
    fn lambdas_and_thin_arrows_lower_to_the_existing_anonymous_function_ast() {
        for (alias, canonical) in [
            ("lambda: 1", "() => (1)"),
            ("lambda value: value * 2", "(value) => (value * 2)"),
            (
                "lambda left, right: left + right",
                "(left, right) => (left + right)",
            ),
            ("lambda: {x: 1}", "() => ({x: 1})"),
            ("lambda: lambda: 1", "() => (() => (1))"),
            ("lambda x: x = 2", "(x) => (x = 2)"),
            ("value -> value * 2", "value => value * 2"),
            ("(value: int) -> value", "(value: int) => value"),
            ("(value = 1) -> value", "(value = 1) => value"),
            ("value -> { return value }", "value => { return value }"),
            ("() -> 1", "() => 1"),
        ] {
            let alias_ast = parse_expression(alias).unwrap();
            assert_eq!(alias_ast, parse_expression(canonical).unwrap(), "{alias}");
            assert!(matches!(alias_ast, Expr::Function(_)), "{alias}");
        }
        let mut parser = Parser::new(tokenize("{ a: lambda: 1, b: 2 }").unwrap());
        parser.skip_separators();
        parser.expression().unwrap();
        assert_eq!(parser.context, ExpressionContext::Ordinary);
        assert_eq!(parser.depth, 0);
    }

    #[test]
    fn comma_free_block_layout_uses_exactly_the_existing_suite_guard() {
        let mut parser = Parser::new(tokenize("left right:\n    pass").unwrap());
        assert_eq!(parser.comma_free_block_end(), Some(2));
        // The existing suite parser also accepts an immediate genuine Indent.
        parser.tokens.remove(3);
        assert_eq!(parser.peek_at(3), &Token::Indent);
        assert_eq!(parser.comma_free_block_end(), Some(2));
        parser.tokens.insert(
            3,
            Spanned {
                token: Token::Semicolon,
                position: Position::new(1, 12),
            },
        );
        assert_eq!(parser.comma_free_block_end(), None);
        assert_eq!(parser.position, 0);
    }

    #[test]
    fn comma_free_blocks_leave_the_final_colon_and_complete_named_block_unchanged() {
        for count in [2, 3, 129, 1024] {
            let names = (0..count).map(|i| format!("p{i}")).collect::<Vec<_>>();
            let candidate = format!("def f {}:\n    int => p0\n    return p0", names.join(" "));
            let accepted = format!("def f {}:\n    int => p0\n    return p0", names.join(", "));
            let actual = parse(&candidate).unwrap();
            let expected = parse(&accepted).unwrap();
            assert_eq!(actual, expected);
            assert_eq!(generate_all(&actual), generate_all(&expected));
            let [Stmt::Function(function)] = actual.as_slice() else {
                panic!("one complete named block required");
            };
            assert_eq!(function.name.as_deref(), Some("f"));
            assert_eq!(function.parameters.len(), count);
            for (parameter, name) in function.parameters.iter().zip(&names) {
                assert_eq!(&parameter.name, name);
                assert_eq!(parameter.type_name, None);
            }
            assert!(matches!(&function.body, FunctionBody::Block(body) if body.len() == 2));
        }
    }

    #[test]
    fn compound_assignments_lower_to_identical_assignment_and_binary_asts() {
        for operator in ["+", "-", "*", "**", "/", "%"] {
            for target in [
                "amount",
                "meter.reading",
                "parent.meter.reading",
                "$Scale.factor",
                "this.reading",
                "self.reading",
                "(amount)",
                "(this).reading",
                "($Scale).factor",
            ] {
                for value in [
                    "2",
                    "-3",
                    "+3",
                    "1 + 2 * 3",
                    "1 - 2 - 3",
                    "step.value",
                    "first = second",
                    "first or second",
                ] {
                    let compact = format!("{target} {operator}= {value}");
                    let expanded = format!("{target} = {target} {operator} ({value})");
                    assert_eq!(
                        parse_expression(&compact).unwrap(),
                        parse_expression(&expanded).unwrap(),
                        "{compact}"
                    );
                    let compact_ast = parse(&compact).unwrap();
                    let expanded_ast = parse(&expanded).unwrap();
                    assert_eq!(compact_ast, expanded_ast, "{compact}");
                    assert_eq!(
                        generate_all(&compact_ast),
                        generate_all(&expanded_ast),
                        "{compact}"
                    );
                }
            }
        }
    }

    #[test]
    fn compound_assignments_keep_assignment_precedence_and_right_associativity() {
        for (compact, expanded) in [
            (
                "first += second *= third -= 1",
                "first = first + (second = second * (third = third - 1))",
            ),
            ("outer = inner += 1", "outer = (inner = inner + 1)"),
            ("first += second > 2", "first = first + (second > 2)"),
            (
                "def combined(a, b) => a += b *= 2",
                "def combined(a, b) => a = a + (b = b * 2)",
            ),
            (
                "class Meter:\n    def adjust(delta) => self.reading += delta",
                "class Meter:\n    def adjust(delta) => self.reading = self.reading + delta",
            ),
            (
                "callback = number => number %= 3",
                "callback = number => number = number % 3",
            ),
            (
                "if active:\n    amount += step\nelse:\n    amount -= step",
                "if active:\n    amount = amount + step\nelse:\n    amount = amount - step",
            ),
            (
                "for item of items:\n    item.count += 1",
                "for item of items:\n    item.count = item.count + 1",
            ),
            (
                "assert((amount += 2), 4)",
                "assert((amount = amount + 2), 4)",
            ),
            (
                "text = `value: ${amount += 2}`",
                "text = `value: ${amount = amount + 2}`",
            ),
            (
                "first **= second **= 2",
                "first = first ** (second = second ** 2)",
            ),
            (
                "def raise(current, power) => current **= power",
                "def raise(current, power) => current = current ** power",
            ),
            (
                "for item of items:\n    item.count **= 2",
                "for item of items:\n    item.count = item.count ** 2",
            ),
            (
                "text = `value: ${amount **= 2}`",
                "text = `value: ${amount = amount ** 2}`",
            ),
        ] {
            let compact_ast = parse(compact).unwrap();
            let expanded_ast = parse(expanded).unwrap();
            assert_eq!(compact_ast, expanded_ast, "{compact}");
            assert_eq!(
                generate_all(&compact_ast),
                generate_all(&expanded_ast),
                "{compact}"
            );
        }
        assert_eq!(
            generate_all(&parse("amount += step").unwrap()),
            "amount=amount+step"
        );
        assert_eq!(
            generate_all(&parse("amount **= step").unwrap()),
            "amount=amount**step"
        );
        assert_eq!(
            generate_all(&parse("amount **= step + 1").unwrap()),
            "amount=amount**(step+1)"
        );
    }

    #[test]
    fn logical_compounds_lower_to_existing_logical_and_assignment_nodes() {
        for (symbol, operator) in [("&&", LogicalOp::And), ("||", LogicalOp::Or)] {
            assert_eq!(
                parse_expression(&format!("target {symbol}= rhs")).unwrap(),
                Expr::Assign {
                    target: Box::new(Expr::Identifier("target".into())),
                    value: Box::new(Expr::Logical {
                        operator,
                        left: Box::new(Expr::Identifier("target".into())),
                        right: Box::new(Expr::Identifier("rhs".into())),
                    }),
                }
            );
            for target in [
                "enabled",
                "gate.enabled",
                "parent.gate.enabled",
                "$Gate.enabled",
                "this.enabled",
                "self.enabled",
                "(enabled)",
                "(this).enabled",
                "($Gate).enabled",
                "(parent.gate).enabled",
            ] {
                for rhs in [
                    "false",
                    "7",
                    "-3",
                    "+3",
                    "!other",
                    "first || second && third",
                    "first or second and third",
                    "first + second * 3 > 0",
                    "source.value",
                    "first = second",
                    "first += 2",
                    "model |> why",
                    "/=/",
                ] {
                    let compact = format!("{target} {symbol}= {rhs}");
                    let expanded = format!("{target} = {target} {symbol} ({rhs})");
                    assert_eq!(
                        parse_expression(&compact).unwrap(),
                        parse_expression(&expanded).unwrap(),
                        "{compact}"
                    );
                    let compact_ast = parse(&compact).unwrap();
                    let expanded_ast = parse(&expanded).unwrap();
                    assert_eq!(compact_ast, expanded_ast, "{compact}");
                    assert_eq!(
                        generate_all(&compact_ast),
                        generate_all(&expanded_ast),
                        "{compact}"
                    );
                }
            }
        }
    }

    #[test]
    fn logical_compounds_preserve_grouping_right_associativity_and_contexts() {
        for (compact, expanded) in [
            ("a &&= b || c", "a = a && (b || c)"),
            ("a ||= b && c", "a = a || (b && c)"),
            ("a &&= b ||= c += 2", "a = a && (b = b || (c = c + 2))"),
            (
                "a ||= b *= c &&= false",
                "a = a || (b = b * (c = c && false))",
            ),
            (
                "outer = inner ||= fallback",
                "outer = (inner = inner || fallback)",
            ),
            (
                "def combined(a, b) => a ||= b &&= 9",
                "def combined(a, b) => a = a || (b = b && 9)",
            ),
            (
                "class Gate:\n    def allow(fallback) => self.enabled ||= fallback",
                "class Gate:\n    def allow(fallback) => self.enabled = self.enabled || fallback",
            ),
            (
                "callback = value => value &&= allowed",
                "callback = value => value = value && allowed",
            ),
            (
                "if active:\n    ready ||= fallback\nelse:\n    ready &&= allowed",
                "if active:\n    ready = ready || fallback\nelse:\n    ready = ready && allowed",
            ),
            (
                "for gate of gates:\n    gate.enabled ||= fallback",
                "for gate of gates:\n    gate.enabled = gate.enabled || fallback",
            ),
            (
                "assert((enabled ||= fallback), true)",
                "assert((enabled = enabled || fallback), true)",
            ),
            (
                "text = `ready: ${enabled &&= allowed}`",
                "text = `ready: ${enabled = enabled && allowed}`",
            ),
            (
                "enabled &&= first or second",
                "enabled = enabled and (first or second)",
            ),
            (
                "enabled ||= first and second",
                "enabled = enabled or (first and second)",
            ),
        ] {
            let compact_ast = parse(compact).unwrap();
            let expanded_ast = parse(expanded).unwrap();
            assert_eq!(compact_ast, expanded_ast, "{compact}");
            assert_eq!(
                generate_all(&compact_ast),
                generate_all(&expanded_ast),
                "{compact}"
            );
        }
        assert_eq!(
            generate_all(&parse("enabled ||= fallback").unwrap()),
            "enabled=enabled||fallback"
        );
        assert_eq!(
            generate_all(&parse("ready &&= allowed").unwrap()),
            "ready=ready&&allowed"
        );
        for legacy in [
            "get().enabled = true",
            "(first = second).enabled = true",
            "items[0].enabled = false",
        ] {
            assert!(parse(legacy).is_ok(), "{legacy}");
        }
    }

    #[test]
    fn compound_assignments_reject_non_path_targets_before_cloning() {
        for target in [
            "1",
            "true",
            "null",
            "$Scale",
            "this",
            "self",
            "first + second",
            "first and second",
            "items[0]",
            "items[0].count",
            "items[:]",
            "get().count",
            "(first = second)",
            "(first += 1)",
            "(first += 1).count",
            "(first ||= 1)",
            "(first &&= 1).count",
            "(first => first)",
            "model |> why",
            "first ** second",
            "(first **= 1)",
        ] {
            for operator in ["+=", "-=", "*=", "**=", "/=", "%=", "&&=", "||="] {
                let source = format!("{target} {operator} 2");
                let error = parse(&source).unwrap_err();
                assert_eq!(error.kind(), crate::ErrorKind::Syntax, "{source}");
                assert_eq!(
                    error.message(),
                    "Invalid compound assignment target",
                    "{source}"
                );
                assert!(error.position().is_some(), "{source}");
            }
        }
    }

    #[test]
    fn compound_target_limits_include_parenthesized_member_chains() {
        for operator in ["+=", "-=", "*=", "**=", "/=", "%=", "&&=", "||="] {
            let allowed = format!(
                "(root{}){} {operator} 1",
                ".next".repeat(64),
                ".next".repeat(64)
            );
            assert!(parse(&allowed).is_ok(), "{operator}");

            let excessive = format!(
                "(root{}){} {operator} 1",
                ".next".repeat(64),
                ".next".repeat(65)
            );
            let error = parse(&excessive).unwrap_err();
            assert_eq!(
                error.message(),
                "Expression has too many parts",
                "{operator}"
            );
        }
    }

    #[test]
    fn exponentiation_binds_tighter_than_products_and_groups_to_the_right() {
        for (compact, grouped) in [
            ("a ** b ** c", "a ** (b ** c)"),
            ("a * b ** c", "a * (b ** c)"),
            ("a ** b * c", "(a ** b) * c"),
            ("a / b ** c % d", "a / (b ** c) % d"),
            ("a + b ** c - d", "a + (b ** c) - d"),
            ("a ** b > c", "(a ** b) > c"),
            ("a ** b ?? c", "(a ** b) ?? c"),
            ("a ** b && c", "(a ** b) && c"),
            ("a ? b ** c : d ** e", "a ? (b ** c) : (d ** e)"),
            ("-a ** b", "-(a ** b)"),
            ("+a ** b", "+(a ** b)"),
            ("!a ** b", "!(a ** b)"),
            ("not a ** b", "not (a ** b)"),
            ("typeof a ** b", "typeof (a ** b)"),
            ("delete a ** b", "delete (a ** b)"),
            ("why a ** b", "(a ** b) |> why"),
            ("new a ** b", "a ** b"),
            ("a ** -b", "a ** (-b)"),
            ("a ** -b ** c", "a ** (-(b ** c))"),
            ("a ** !b", "a ** (!b)"),
            ("a ** typeof b", "a ** (typeof b)"),
            ("a.b ** c[0]", "(a.b) ** (c[0])"),
            ("f(a) ** g(b)", "(f(a)) ** (g(b))"),
            ("a@first ** 2", "(a@first) ** 2"),
            ("x => x ** 2", "x => (x ** 2)"),
            ("lambda x: x ** 2", "(x) => (x ** 2)"),
        ] {
            assert_eq!(
                parse_expression(compact).unwrap(),
                parse_expression(grouped).unwrap(),
                "{compact}"
            );
        }
        assert_eq!(
            parse_expression("a ** b").unwrap(),
            Expr::Binary {
                operator: BinaryOp::Power,
                left: Box::new(Expr::Identifier("a".into())),
                right: Box::new(Expr::Identifier("b".into())),
            }
        );
    }

    #[test]
    fn power_rendering_reads_back_as_the_same_expression() {
        for (source, rendered) in [
            ("x ** 2", "x**2"),
            ("x ** 2 ** 3", "x**2**3"),
            ("(x ** 2) ** 3", "(x**2)**3"),
            ("-x ** 2", "-x**2"),
            ("(-x) ** 3", "(-x)**3"),
            ("(!x) ** 2", "(!x)**2"),
            ("(typeof x) ** 2", "(typeof x)**2"),
            ("x ** -y", "x**-y"),
            ("x ** (1 + 1)", "x**(1+1)"),
            ("x ** (y > 1)", "x**(y>1)"),
            ("(2 * x) ** 2", "(2*x)**2"),
            ("2 * x ** 2", "2*x**2"),
            ("x ** 2 * 2", "x**2*2"),
            ("(a || b) ** 2", "(a||b)**2"),
            ("x ** (a && b)", "x**(a&&b)"),
            ("(a ?? b) ** 2", "(a??b)**2"),
            ("x ** (a ?? b)", "x**(a??b)"),
            ("(c ? a : b) ** 2", "(c?a:b)**2"),
            ("x ** (c ? a : b)", "x**(c?a:b)"),
            ("(why x) ** 2", "(x|>why)**2"),
            ("2 ** (x |> why)", "2**(x|>why)"),
            ("(x = 2) ** 2", "(x=2)**2"),
            ("x ** (y = 2)", "x**(y=2)"),
            ("(delete x.y) ** 2", "(delete x.y)**2"),
            ("x ** delete y", "x**delete y"),
            ("((a = 1) => a) ** 2", "((a=1)=>a)**2"),
            ("2 ** ((a = 1) => a)", "2**((a=1)=>a)"),
            ("x.y ** z[0]", "x.y**z[0]"),
            ("f(x) ** g(y)", "f(x)**g(y)"),
            ("(all v in xs: v) ** 2", "(all v in xs:v)**2"),
        ] {
            let expression = parse_expression(source).unwrap();
            assert_eq!(expression.to_string(), rendered, "{source}");
            assert_eq!(parse_expression(rendered).unwrap(), expression, "{source}");
        }
        let literal = Expr::Binary {
            operator: BinaryOp::Power,
            left: Box::new(Expr::Number(-3.0)),
            right: Box::new(Expr::Number(-2.0)),
        };
        assert_eq!(literal.to_string(), "(-3)**-2");
        assert_eq!(
            generate_all(&parse("area = side ** 2 * depth").unwrap()),
            "area=side**2*depth"
        );
    }

    #[test]
    fn power_chains_share_the_expression_nesting_budget() {
        let assigned = |count: usize| format!("x = 1{}", " ** 1".repeat(count));
        assert!(parse(&assigned(61)).is_ok());
        let error = parse(&assigned(62)).unwrap_err();
        assert_eq!(error.message(), "Expressions are nested too deeply");
        assert_eq!(error.position(), Some(Position::new(1, 315)));
        let bare = |count: usize| format!("1{}", " ** 1".repeat(count));
        assert!(parse(&bare(62)).is_ok());
        let error = parse(&bare(63)).unwrap_err();
        assert_eq!(error.message(), "Expressions are nested too deeply");
        assert_eq!(error.position(), Some(Position::new(1, 316)));
    }

    #[test]
    fn named_expression_bodies_lower_to_single_return_blocks() {
        for expression in [
            "number + 1 * factor",
            "number = factor",
            "{value: number, scaled: number * factor}",
            "value => value * 2",
            "(value: int) => { return value * 2 }",
            "why number",
        ] {
            let compact = parse(&format!(
                "def calculate(number: int, factor) => {expression}"
            ))
            .unwrap();
            let expanded = parse(&format!(
                "def calculate(number: int, factor):\n    return {expression}"
            ))
            .unwrap();

            assert_eq!(compact, expanded, "{expression}");
            assert_eq!(generate_all(&compact), generate_all(&expanded));

            let [Stmt::Function(function)] = compact.as_slice() else {
                panic!("expected one named function");
            };
            let FunctionBody::Block(body) = &function.body else {
                panic!("named expression bodies must use block execution");
            };

            assert!(matches!(body.as_slice(), [Stmt::Return(Some(_))]));
            assert_eq!(
                generate_all(body),
                format!("return {}", parse_expression(expression).unwrap())
            );
        }
    }

    #[test]
    fn named_expression_methods_share_block_lowering() {
        let compact = parse(
            r"
class Gauge:
    def init(amount: int) => this.amount = amount
    def reading() => self.amount
",
        )
        .unwrap();
        let expanded = parse(
            r"
class Gauge:
    def init(amount: int):
        return this.amount = amount
    def reading():
        return self.amount
",
        )
        .unwrap();

        assert_eq!(compact, expanded);
        assert_eq!(generate_all(&compact), generate_all(&expanded));

        let [Stmt::Class(class)] = compact.as_slice() else {
            panic!("expected one class");
        };
        assert_eq!(class.methods.len(), 2);
        assert!(class.methods.iter().all(|method| matches!(
            &method.body,
            FunctionBody::Block(body) if matches!(body.as_slice(), [Stmt::Return(Some(_))])
        )));
    }

    #[test]
    fn named_expression_bodies_allow_parenthesized_continuations() {
        let compact =
            parse("def double(number: int) => (\n    number *\n    2\n)\nother = 3").unwrap();
        let expanded = parse("def double(number: int):\n    return number * 2\nother = 3").unwrap();

        assert_eq!(compact, expanded);
        assert_eq!(generate_all(&compact), generate_all(&expanded));
    }

    #[test]
    fn while_conditions_qualify_only_colon_headers_and_reuse_outcomes() {
        for (source, condition) in [
            ("while n < 3:\n    n = n + 1", Some("n<3")),
            ("while n < 3: n = n + 1", Some("n<3")),
            ("while (ready): pass", Some("ready")),
            ("while -n: pass", Some("-n")),
            ("while [n]: pass", Some("[n]")),
            ("while {}: pass", Some("{}")),
            ("while x if y else z: pass", Some("(y?x:z)")),
            ("while all v in xs: v > 0: pass", Some("(all v in xs:v>0)")),
            ("while ready ? on : off: pass", Some("(ready?on:off)")),
            ("while x => lambda + 1: pass", Some("function()")),
            ("while [lambda: 1]: pass", Some("[function()]")),
            ("while while: pass", Some("while")),
            ("while ready: tick()", Some("ready")),
            ("while ready: 1", Some("ready")),
            ("while n < 3 {\n    n = n + 1\n}", None),
            ("while n < 3\n    n = n + 1", None),
            ("while lambda: 1", None),
            ("while ready + lambda: 1", None),
            ("while not lambda: 1", None),
            ("while ready ? on : lambda: 1", None),
            ("while x => lambda: 1", None),
            ("while gauge.lambda: tick()", None),
            ("while lambda x: x", None),
            ("while x if y: z", None),
            ("while all v in xs: v", None),
            ("while: int", None),
            ("while(4)", None),
            ("while = 1", None),
            ("while", None),
            ("while\n    ready: int", None),
            ("while ready\n: tick()", None),
            ("while @first: tick()", None),
        ] {
            let mut parser = Parser::new(tokenize(source).unwrap());
            let start = parser.position;
            let tokens = parser.tokens.clone();
            let first = parser.while_condition();
            assert_eq!(
                first.as_ref().map(ToString::to_string).as_deref(),
                condition,
                "{source}"
            );
            if first.is_some() {
                assert_eq!(parser.peek(), &Token::Colon, "{source}");
            } else {
                assert_eq!(parser.position, start, "{source}");
            }
            assert_eq!(parser.depth, 0, "{source}");
            assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
            let recorded = parser.while_conditions.len();
            assert!(
                parser.while_conditions.contains_key(&(start, 0)),
                "{source}"
            );
            let end = parser.position;
            parser.position = start;
            assert_eq!(parser.while_condition(), first, "{source}");
            assert_eq!(parser.position, end, "{source}");
            assert_eq!(parser.while_conditions.len(), recorded, "{source}");
            assert_eq!(parser.tokens, tokens, "{source}");
        }

        for source in [
            "while ready: tick",
            "while ready: tick;",
            "while assert: ready, true",
            "value = 1",
        ] {
            let mut parser = Parser::new(tokenize(source).unwrap());
            assert_eq!(parser.while_condition(), None, "{source}");
            assert_eq!(parser.position, 0, "{source}");
            assert_eq!(parser.depth, 0, "{source}");
            assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
            assert!(parser.while_conditions.is_empty(), "{source}");
        }
    }

    #[test]
    fn nested_unqualified_while_headers_are_attempted_once_per_position() {
        let levels = 16;
        let mut source = String::new();
        for level in 0..levels {
            source.push_str(&"    ".repeat(level));
            source.push_str("while (function() {\n");
        }
        source.push_str(&"    ".repeat(levels));
        source.push_str("ready\n");
        for level in (0..levels).rev() {
            source.push_str(&"    ".repeat(level));
            source.push_str("})\n");
        }

        let mut parser = Parser::new(tokenize(&source).unwrap());
        let program = parser.program().unwrap();

        assert_eq!(program.statements.len(), 1);
        assert!(matches!(
            &program.statements[0],
            Stmt::Expression(Expr::Call { callee, .. })
                if matches!(callee.as_ref(), Expr::Identifier(name) if name == "while")
        ));
        assert!(parser.while_conditions.len() <= levels * 2);
    }

    #[test]
    fn while_conditions_leave_suppose_heads_and_value_assignments_to_the_accepted_parser() {
        for (source, condition) in [
            ("while suppose: x", None),
            ("while suppose:\n    x = 1", None),
            ("while x = suppose: x", None),
            ("while x = (suppose): x", None),
            ("while point.x = suppose: x", None),
            ("while (a) = suppose: a", None),
            ("while [a] = suppose: a", None),
            ("while (suppose): pass", None),
            ("while (x = suppose): pass", None),
            ("while suppose x > 1: x", None),
            ("while x = suppose y: y", None),
            ("while := 1", None),
            ("while x := 1", None),
            ("while x:= 1", None),
            ("while suppose > 1: pass", Some("suppose>1")),
            ("while x == suppose: pass", Some("x==suppose")),
            ("while x = suppose + 1: pass", Some("x=suppose+1")),
            ("while x = y = suppose: pass", Some("x=y=suppose")),
            ("while not suppose: pass", Some("!suppose")),
            ("while [suppose]: pass", Some("[suppose]")),
            ("while x: y := 1", Some("x")),
            ("while x > 0: x := x - 1", Some("x>0")),
        ] {
            let mut parser = Parser::new(tokenize(source).unwrap());
            let first = parser.while_condition();
            assert_eq!(
                first.as_ref().map(ToString::to_string).as_deref(),
                condition,
                "{source}"
            );
            if first.is_some() {
                assert_eq!(parser.peek(), &Token::Colon, "{source}");
            } else {
                assert_eq!(parser.position, 0, "{source}");
            }
            assert_eq!(parser.depth, 0, "{source}");
            assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
        }
    }
}
