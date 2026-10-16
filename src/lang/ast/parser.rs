use std::sync::Arc;

use super::lexer::{Keyword, Spanned, Token, tokenize};
use crate::error::{Error, Position, Result};
use crate::lang::ast::{
    BinaryOp, ClassDecl, Expr, Function, FunctionBody, LogicalOp, Parameter, Stmt, TemplatePart,
    UnaryOp,
};
use crate::reasoning::Stage;

/// A parsed program: its statements, and where each top-level statement began.
/// The positions are what places a runtime error in the source.
pub struct Program {
    pub statements: Vec<Stmt>,
    pub positions: Vec<Position>,
}

pub fn parse_program(source: &str) -> Result<Program> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);
    parser.program()
}

pub fn parse(source: &str) -> Result<Vec<Stmt>> {
    Ok(parse_program(source)?.statements)
}

pub fn parse_expression(source: &str) -> Result<Expr> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);
    parser.skip_separators();
    let expression = parser.expression()?;
    Ok(expression)
}

/// How deeply expressions and blocks may nest. Recursive descent uses the
/// stack, so the limit is what keeps a pathological source from overflowing it.
const MAX_NESTING: usize = 64;

/// How many operands one chain of operators may have. Chains are built
/// iteratively so they escape `MAX_NESTING`, but they still produce a deep tree
/// that later walks recurse over.
const MAX_CHAIN: usize = 128;

/// Compound syntax selects an existing operator, never a new runtime path.
enum CompoundOp {
    Arithmetic(BinaryOp),
    Logical(LogicalOp),
}

struct Parser {
    tokens: Vec<Spanned>,
    position: usize,
    depth: usize,
}

impl Parser {
    fn new(tokens: Vec<Spanned>) -> Self {
        Parser {
            tokens,
            position: 0,
            depth: 0,
        }
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.position)
            .map(|spanned| &spanned.token)
            .unwrap_or(&Token::Eof)
    }

    fn peek_at(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.position + offset)
            .map(|spanned| &spanned.token)
            .unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self
            .tokens
            .get(self.position)
            .map(|spanned| spanned.token.clone())
            .unwrap_or(Token::Eof);

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
            .map(|spanned| spanned.position)
            .unwrap_or_else(|| Position::new(1, 1))
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

            positions.push(self.here());
            statements.push(self.statement()?);
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

    /// Shares the existing body parser with a final `else` whose real
    /// indentation has already qualified for colon omission.
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

                statements.push(self.statement()?);
            }

            self.eat(&Token::Dedent);
            Ok(statements)
        } else {
            let mut statements = vec![self.statement()?];

            while self.eat(&Token::Semicolon) {
                if matches!(self.peek(), Token::Newline | Token::Eof) {
                    break;
                }
                statements.push(self.statement()?);
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

            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt> {
        self.depth += 1;

        if self.depth > MAX_NESTING {
            self.depth -= 1;
            return Err(self.error("Statements are nested too deeply"));
        }

        let result = self.statement_inner();
        self.depth -= 1;
        result
    }

    fn statement_inner(&mut self) -> Result<Stmt> {
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
            Token::Keyword(Keyword::Return) => {
                self.advance();

                if matches!(
                    self.peek(),
                    Token::Newline
                        | Token::Semicolon
                        | Token::Dedent
                        | Token::RightBrace
                        | Token::Eof
                ) {
                    Ok(Stmt::Return(None))
                } else {
                    Ok(Stmt::Return(Some(self.expression()?)))
                }
            }
            Token::Keyword(Keyword::Throw) => {
                self.advance();
                Ok(Stmt::Throw(self.expression()?))
            }
            Token::Keyword(Keyword::Delete) => {
                self.advance();
                Ok(Stmt::Delete(self.expression()?))
            }
            Token::LeftBrace => Ok(Stmt::Block(self.braced()?)),
            Token::Identifier(name)
                if self.peek_at(1) == &Token::Colon && self.is_type_declaration() =>
            {
                self.advance();
                self.advance();
                let type_name = match self.advance() {
                    Token::Identifier(type_name) => type_name,
                    other => {
                        return Err(
                            self.error(format!("Expected a type but found {}", other.describe()))
                        );
                    }
                };
                Ok(Stmt::Declaration { name, type_name })
            }
            Token::Identifier(name) if name == "assert" && self.peek_at(1) == &Token::Colon => {
                self.advance();
                self.advance();
                // An ordinary call has an outer expression level in addition
                // to each argument's level. Preserve that same nesting budget.
                self.nested_expression(Self::assertion_expression)
                    .map(Stmt::Expression)
            }
            _ => {
                let expression = self.expression()?;

                match expression {
                    Expr::Assign { target, value } => Ok(Stmt::Assign {
                        target: *target,
                        value: *value,
                    }),
                    other => Ok(Stmt::Expression(other)),
                }
            }
        }
    }

    fn assertion_expression(&mut self) -> Result<Expr> {
        let actual = self.expression()?;
        self.expect(&Token::Comma)?;
        let expected = self.expression()?;
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
        matches!(self.peek_at(2), Token::Identifier(_))
            && matches!(
                self.peek_at(3),
                Token::Newline | Token::Semicolon | Token::Dedent | Token::Eof
            )
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::If))?;
        self.condition_clause()
    }

    /// Explicit `else if` and qualified direct alternatives share the entire
    /// condition, suite and recursive alternate parser.
    fn condition_clause(&mut self) -> Result<Stmt> {
        let condition = self.expression()?;
        let consequent = self.suite()?;

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
    /// an expression. Only its top-level colon may introduce the real suite;
    /// colons in calls, slices and object literals still belong to the header.
    fn has_direct_condition_suite(&self) -> bool {
        let mut delimiters = Vec::new();
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
                Token::Colon if delimiters.is_empty() => {
                    return offset > 0 && self.has_indented_suite(offset);
                }
                Token::Newline | Token::Semicolon | Token::Indent | Token::Dedent | Token::Eof
                    if delimiters.is_empty() =>
                {
                    return false;
                }
                _ => {}
            }
        }
        false
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::For))?;

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

        let iterable = self.expression()?;
        let body = self.suite()?;

        Ok(Stmt::For {
            variable,
            iterable,
            body,
        })
    }

    /// Recognizes only the real-indented colon path already accepted by
    /// suite(), without consuming tokens or skipping header separators.
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
        let body = self.suite()?;
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

        let catch = self.suite()?;

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
                Token::Arrow | Token::Colon if multiple => return Some(offset),
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
            .filter(|&end| matches!(self.peek_at(end), Token::Arrow))
    }

    fn comma_free_block_end(&self) -> Option<usize> {
        self.comma_free_parameter_end()
            .filter(|&end| self.has_indented_suite(end))
    }

    /// Only used after complete lookahead has qualified an untyped name.
    /// In particular, leave a final suite colon for suite(), not parameter().
    fn untyped_parameter(&mut self) -> Parameter {
        let Token::Identifier(name) = self.advance() else {
            unreachable!("lookahead checked the untyped parameter identifier")
        };
        Parameter {
            name,
            type_name: None,
        }
    }

    fn function_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::Def))?;

        let name = match self.advance() {
            Token::Identifier(name) => name,
            other => {
                return Err(self.error(format!("Expected a name but found {}", other.describe())));
            }
        };

        let bare_arrow = matches!(
            (
                self.peek(),
                self.peek_at(1),
                self.peek_at(2),
                self.peek_at(3)
            ),
            (Token::Identifier(_), Token::Arrow, _, _)
                | (
                    Token::Identifier(_),
                    Token::Colon,
                    Token::Identifier(_),
                    Token::Arrow
                )
        );
        let bare_multiple_end = self.bare_multiple_parameter_end().filter(|&end| {
            matches!(self.peek_at(end), Token::Arrow) || self.has_indented_suite(end)
        });
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
        let parameters = if let Some(end) = comma_free_end {
            (0..end).map(|_| self.untyped_parameter()).collect()
        } else if let Some(end) = bare_multiple_end {
            let delimiter_position = self.position + end;
            let mut parameters = Vec::new();
            loop {
                let parameter = if self.position + 1 == delimiter_position
                    && matches!(self.peek_at(1), Token::Colon)
                {
                    // A final untyped parameter's colon belongs to suite(),
                    // not parameter()'s annotation parser.
                    self.untyped_parameter()
                } else {
                    self.parameter()?
                };
                parameters.push(parameter);
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
            // Leave the complete list's arrow or suite colon for the body.
            parameters
        } else if bare_arrow || bare_typed_block {
            vec![self.parameter()?]
        } else if bare_block {
            vec![self.untyped_parameter()]
        } else {
            // Do not skip separators or consume other prefixes: preserve the
            // parenthesized parameter parser's diagnostics and positions.
            self.parameters()?
        };
        let body = if self.eat(&Token::Arrow) {
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
            vec![Stmt::Return(Some(expression))]
        } else {
            self.suite()?
        };

        Ok(Stmt::Function(Arc::new(Function {
            name: Some(name),
            parameters,
            body: FunctionBody::Block(body),
        })))
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

        Ok(Parameter { name, type_name })
    }

    fn parameters(&mut self) -> Result<Vec<Parameter>> {
        self.expect(&Token::LeftParen)?;

        let mut parameters = Vec::new();

        while !self.check(&Token::RightParen) {
            parameters.push(self.parameter()?);

            if !self.eat(&Token::Comma) {
                break;
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

        if matches!(
            self.peek(),
            Token::Newline | Token::Semicolon | Token::Dedent | Token::RightBrace | Token::Eof
        ) {
            // Leave the terminator for the enclosing suite or block. An indent
            // after separators still requires a colon; skip_separators would
            // otherwise turn a misspelled class body into separate statements.
            let following = self
                .tokens
                .iter()
                .skip(self.position)
                .find(|spanned| !matches!(spanned.token, Token::Newline | Token::Semicolon));
            if following.is_none_or(|spanned| spanned.token != Token::Indent) {
                return Ok(Stmt::Class(ClassDecl {
                    name,
                    parent: None,
                    parameters,
                    fields: Vec::new(),
                    constructor: Vec::new(),
                    methods: Vec::new(),
                }));
            }
        }

        self.expect(&Token::Colon)?;

        // `class Student: Person` names a parent on the same line.
        let mut parent = None;

        if let Token::Identifier(candidate) = self.peek().clone() {
            if matches!(self.peek_at(1), Token::Newline | Token::Indent | Token::Eof) {
                self.advance();
                parent = Some(candidate);
            }
        }

        let mut fields = Vec::new();
        let mut constructor = Vec::new();
        let mut methods = Vec::new();

        self.skip_newlines();

        if self.eat(&Token::Indent) {
            loop {
                self.skip_newlines();

                if self.check(&Token::Dedent) || self.check(&Token::Eof) {
                    break;
                }

                match self.peek().clone() {
                    Token::Keyword(Keyword::Pass) => {
                        self.advance();
                    }
                    Token::Keyword(Keyword::Def) => {
                        let Stmt::Function(function) = self.function_statement()? else {
                            unreachable!("function_statement always returns Stmt::Function")
                        };
                        methods.push(function);
                    }
                    Token::Identifier(field)
                        if self.peek_at(1) == &Token::Colon && self.is_type_declaration() =>
                    {
                        self.advance();
                        self.advance();
                        let type_name = match self.advance() {
                            Token::Identifier(type_name) => type_name,
                            other => {
                                return Err(self.error(format!(
                                    "Expected a type but found {}",
                                    other.describe()
                                )));
                            }
                        };
                        fields.push(Parameter {
                            name: field,
                            type_name: Some(type_name),
                        });
                    }
                    _ => constructor.push(self.statement()?),
                }
            }

            self.eat(&Token::Dedent);
        } else if !matches!(self.peek(), Token::Newline | Token::Eof) {
            constructor.push(self.statement()?);
        }

        Ok(Stmt::Class(ClassDecl {
            name,
            parent,
            parameters,
            fields,
            constructor,
            methods,
        }))
    }

    fn expression(&mut self) -> Result<Expr> {
        self.nested_expression(Self::expression_inner)
    }

    fn nested_expression(&mut self, parse: fn(&mut Self) -> Result<Expr>) -> Result<Expr> {
        self.depth += 1;

        if self.depth > MAX_NESTING {
            self.depth -= 1;
            return Err(self.error("Expressions are nested too deeply"));
        }

        let result = parse(self);
        self.depth -= 1;
        result
    }

    fn expression_inner(&mut self) -> Result<Expr> {
        let left = self.pipeline()?;

        let operator = match self.peek() {
            Token::Assign => None,
            Token::PlusAssign => Some(CompoundOp::Arithmetic(BinaryOp::Add)),
            Token::MinusAssign => Some(CompoundOp::Arithmetic(BinaryOp::Subtract)),
            Token::StarAssign => Some(CompoundOp::Arithmetic(BinaryOp::Multiply)),
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
        let value = self.expression()?;
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
    fn check_compound_target(&self, mut target: &Expr) -> Result<()> {
        let mut parts = 0;
        while let Expr::Member { object, .. } = target {
            parts += 1;
            if parts > MAX_CHAIN {
                return Err(self.error("Expression has too many parts"));
            }
            target = object;
        }

        match target {
            Expr::Identifier(_) => Ok(()),
            Expr::ClassRef(_) | Expr::This if parts > 0 => Ok(()),
            _ => Err(self.error("Invalid compound assignment target")),
        }
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

            let stage = self.stage()?;
            left = Expr::Reason {
                stage,
                source: Box::new(left),
            };
        }

        Ok(left)
    }

    fn stage(&mut self) -> Result<Stage> {
        if let Token::Identifier(name) = self.peek() {
            if let Some(stage) = Stage::from_name(name) {
                self.advance();
                return Ok(stage);
            }

            let name = name.clone();
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
                if self.starts_operand(1) {
                    self.advance();
                    let operand = self.nested_expression(Self::unary)?;

                    return Ok(Expr::Reason {
                        stage,
                        source: Box::new(operand),
                    });
                }
            }
        }

        let operator = match self.peek() {
            Token::Bang | Token::Keyword(Keyword::Not) => UnaryOp::Not,
            Token::Minus => UnaryOp::Negate,
            Token::Plus => UnaryOp::Plus,
            Token::Keyword(Keyword::TypeOf) => UnaryOp::TypeOf,
            _ => return self.postfix(),
        };

        self.advance();
        let operand = self.nested_expression(Self::unary)?;

        Ok(Expr::Unary {
            operator,
            operand: Box::new(operand),
        })
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
                Token::Dot => {
                    self.advance();
                    let property = match self.advance() {
                        Token::Identifier(name) => name,
                        Token::Keyword(keyword) => keyword.as_str().to_string(),
                        other => {
                            return Err(self.error(format!(
                                "Expected a property but found {}",
                                other.describe()
                            )));
                        }
                    };
                    expression = Expr::Member {
                        object: Box::new(expression),
                        property,
                    };
                }
                Token::LeftBracket => {
                    self.advance();
                    expression = self.subscript(expression)?;
                }
                Token::LeftParen => {
                    self.advance();
                    let arguments = self.arguments()?;
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

        let index = self.expression()?;

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

    fn arguments(&mut self) -> Result<Vec<Expr>> {
        let mut arguments = Vec::new();

        while !self.check(&Token::RightParen) {
            arguments.push(self.expression()?);

            if !self.eat(&Token::Comma) {
                break;
            }
        }

        self.expect(&Token::RightParen)?;
        Ok(arguments)
    }

    fn primary(&mut self) -> Result<Expr> {
        // `name => ...`
        if let Token::Identifier(name) = self.peek().clone() {
            if self.peek_at(1) == &Token::Arrow {
                self.advance();
                self.advance();
                return self.arrow_body(vec![Parameter {
                    name,
                    type_name: None,
                }]);
            }
        }

        // `(a, b) => ...`
        if self.check(&Token::LeftParen) && self.is_arrow_parameters() {
            let parameters = self.parameters()?;
            self.expect(&Token::Arrow)?;
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
                    self.arguments()?
                } else {
                    Vec::new()
                };
                Ok(Expr::Super(arguments))
            }
            Token::Keyword(Keyword::Function) => {
                let parameters = self.parameters()?;
                let body = self.braced()?;
                Ok(Expr::Function(Arc::new(Function {
                    name: None,
                    parameters,
                    body: FunctionBody::Block(body),
                })))
            }
            Token::LeftParen => {
                let expression = self.expression()?;
                self.expect(&Token::RightParen)?;
                Ok(expression)
            }
            Token::LeftBracket => {
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
            Token::LeftBrace => self.object_literal(),
            other => Err(self.error(format!("Unexpected {}", other.describe()))),
        }
    }

    fn object_literal(&mut self) -> Result<Expr> {
        let mut entries = Vec::new();

        loop {
            self.skip_newlines();

            if self.eat(&Token::RightBrace) {
                break;
            }

            let (key, is_identifier) = match self.advance() {
                Token::Str(key) => (key, false),
                Token::Identifier(key) => (key, true),
                Token::Number(number) => (crate::value::format_number(number), false),
                other => {
                    return Err(
                        self.error(format!("Expected a key but found {}", other.describe()))
                    );
                }
            };

            // Do not consume lookahead: malformed explicit fields must keep
            // their original missing-colon diagnostics and positions.
            let mut offset = 0;
            while matches!(self.peek_at(offset), Token::Newline | Token::Semicolon) {
                offset += 1;
            }
            let value = if is_identifier
                && matches!(self.peek_at(offset), Token::Comma | Token::RightBrace)
            {
                Expr::Identifier(key.clone())
            } else {
                self.expect(&Token::Colon)?;
                self.expression()?
            };
            entries.push((key, value));

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

        Ok(Expr::ObjectLiteral(entries))
    }

    fn arrow_body(&mut self, parameters: Vec<Parameter>) -> Result<Expr> {
        let body = if self.check(&Token::LeftBrace) {
            FunctionBody::Block(self.braced()?)
        } else {
            FunctionBody::Expression(Box::new(self.expression()?))
        };

        Ok(Expr::Function(Arc::new(Function {
            name: None,
            parameters,
            body,
        })))
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
                        return self.peek_at(offset + 1) == &Token::Arrow;
                    }
                }
                Token::Eof => return false,
                _ => {}
            }

            offset += 1;
        }
    }

    fn template(&mut self, raw: &str) -> Result<Expr> {
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
            let mut depth = 1usize;
            let mut end = None;

            for (offset, candidate) in raw[start..].char_indices() {
                match candidate {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(start + offset);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let Some(end) = end else {
                return Err(self.error("Unterminated template expression"));
            };

            if !literal.is_empty() {
                parts.push(TemplatePart::Literal(std::mem::take(&mut literal)));
            }

            parts.push(TemplatePart::Expression(parse_expression(
                &raw[start..end],
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::ast::generator::generate_all;

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
            // Qualification is not expression validation. Real parsing must
            // report these malformed expressions rather than fall back.
            ("ready +:\n    pass", true),
            ("():\n    pass", true),
            ("pass:\n    pass", true),
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
        for boundary in [
            Token::Newline,
            Token::Semicolon,
            Token::Indent,
            Token::Dedent,
            Token::Eof,
        ] {
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
            assert!(!parser.has_direct_condition_suite());
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
        for operator in ["+", "-", "*", "/", "%"] {
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
        ] {
            for operator in ["+=", "-=", "*=", "/=", "%=", "&&=", "||="] {
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
        for operator in ["+=", "-=", "*=", "/=", "%=", "&&=", "||="] {
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
            r#"
class Gauge:
    def init(amount: int) => this.amount = amount
    def reading() => self.amount
"#,
        )
        .unwrap();
        let expanded = parse(
            r#"
class Gauge:
    def init(amount: int):
        return this.amount = amount
    def reading():
        return self.amount
"#,
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
}
