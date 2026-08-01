use std::sync::Arc;

use crate::ast::{
    BinaryOp, ClassDecl, Expr, Function, FunctionBody, LogicalOp, Parameter, Stmt, TemplatePart,
    UnaryOp,
};
use crate::error::{Error, Position, Result};
use crate::lexer::{Keyword, Spanned, Token, tokenize};

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
        let condition = self.expression()?;
        let consequent = self.suite()?;

        let mut alternate = None;
        let checkpoint = self.position;
        self.skip_newlines();

        if self.eat_keyword(Keyword::Else) {
            if self.check(&Token::Keyword(Keyword::If)) {
                alternate = Some(Box::new(self.if_statement()?));
            } else {
                alternate = Some(Box::new(Stmt::Block(self.suite()?)));
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

    fn for_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::For))?;

        let variable = match self.advance() {
            Token::Identifier(name) => name,
            other => {
                return Err(self.error(format!("Expected a name but found {}", other.describe())));
            }
        };

        if !self.eat_keyword(Keyword::Of) && !self.eat_keyword(Keyword::In) {
            return Err(self.error("Expected 'of' in for statement"));
        }

        let iterable = self.expression()?;
        let body = self.suite()?;

        Ok(Stmt::For {
            variable,
            iterable,
            body,
        })
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

    fn function_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::Def))?;

        let name = match self.advance() {
            Token::Identifier(name) => name,
            other => {
                return Err(self.error(format!("Expected a name but found {}", other.describe())));
            }
        };

        let parameters = self.parameters()?;
        let body = self.suite()?;

        Ok(Stmt::Function(Arc::new(Function {
            name: Some(name),
            parameters,
            body: FunctionBody::Block(body),
        })))
    }

    fn parameters(&mut self) -> Result<Vec<Parameter>> {
        self.expect(&Token::LeftParen)?;

        let mut parameters = Vec::new();

        while !self.check(&Token::RightParen) {
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

            parameters.push(Parameter { name, type_name });

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
        self.depth += 1;

        if self.depth > MAX_NESTING {
            self.depth -= 1;
            return Err(self.error("Expressions are nested too deeply"));
        }

        let result = self.expression_inner();
        self.depth -= 1;
        result
    }

    fn expression_inner(&mut self) -> Result<Expr> {
        let left = self.logical_or()?;

        if self.check(&Token::Assign) {
            self.advance();
            let value = self.expression()?;
            return Ok(Expr::Assign {
                target: Box::new(left),
                value: Box::new(value),
            });
        }

        Ok(left)
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
        let operator = match self.peek() {
            Token::Bang | Token::Keyword(Keyword::Not) => UnaryOp::Not,
            Token::Minus => UnaryOp::Negate,
            Token::Plus => UnaryOp::Plus,
            Token::Keyword(Keyword::TypeOf) => UnaryOp::TypeOf,
            _ => return self.postfix(),
        };

        self.advance();
        let operand = self.unary()?;

        Ok(Expr::Unary {
            operator,
            operand: Box::new(operand),
        })
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
            Token::Keyword(Keyword::New) => self.primary(),
            Token::Keyword(Keyword::Delete) => {
                let operand = self.unary()?;
                Ok(Expr::Delete(Box::new(operand)))
            }
            Token::Keyword(Keyword::Super) => {
                self.expect(&Token::LeftParen)?;
                let arguments = self.arguments()?;
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

            let key = match self.advance() {
                Token::Str(key) => key,
                Token::Identifier(key) => key,
                Token::Number(number) => crate::value::format_number(number),
                other => {
                    return Err(
                        self.error(format!("Expected a key but found {}", other.describe()))
                    );
                }
            };

            self.expect(&Token::Colon)?;
            let value = self.expression()?;
            entries.push((key, value));

            self.skip_newlines();

            if !self.eat(&Token::Comma) {
                self.skip_newlines();
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
