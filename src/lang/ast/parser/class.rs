use std::sync::Arc;

use super::{ParameterList, Parser};
use crate::error::Result;
use crate::lang::ast::lexer::{Keyword, Token};
use crate::lang::ast::{ClassDecl, Expr, Function, FunctionBody, Parameter, Stmt};

fn class_declaration(
    name: String,
    parent: Option<String>,
    ParameterList {
        parameters,
        initializers,
    }: ParameterList,
) -> ClassDecl {
    ClassDecl {
        name,
        parent,
        parameters,
        fields: Vec::new(),
        constructor: initializers,
        methods: Vec::new(),
    }
}

pub(super) fn initializer(name: &str) -> Stmt {
    Stmt::Assign {
        target: Expr::Member {
            object: Box::new(Expr::This),
            property: name.to_string(),
        },
        value: Expr::Identifier(name.to_string()),
    }
}

impl Parser {
    pub(super) fn class_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::Class))?;

        let name = match self.advance() {
            Token::Identifier(name) => name,
            other => {
                return Err(self.error(format!("Expected a name but found {}", other.describe())));
            }
        };

        let parameters = if self.check(&Token::LeftParen) {
            self.constructor_parameters()?
        } else {
            ParameterList::default()
        };

        if matches!(self.peek(), Token::Identifier(word) if word == "extends") {
            return self.extends_class(name, parameters);
        }

        if self.ends_bodyless_header() {
            return Ok(Stmt::Class(class_declaration(name, None, parameters)));
        }

        self.expect(&Token::Colon)?;

        // `class Student: Person` names a parent on the same line.
        let parent = self.colon_parent().map(str::to_owned);

        if parent.is_some() {
            self.advance();
        }

        let mut declaration = class_declaration(name, parent, parameters);
        self.class_body(&mut declaration)?;
        Ok(Stmt::Class(declaration))
    }

    fn extends_class(&mut self, name: String, parameters: ParameterList) -> Result<Stmt> {
        self.advance();

        let Token::Identifier(parent) = self.peek().clone() else {
            return Err(self.error(format!(
                "Expected a parent class name but found {}",
                self.peek().describe()
            )));
        };
        self.advance();

        if self.ends_bodyless_header() {
            return Ok(Stmt::Class(class_declaration(
                name,
                Some(parent),
                parameters,
            )));
        }

        self.expect(&Token::Colon)?;

        if self.colon_parent().is_some() {
            return Err(self.error(format!("Class '{name}' already extends '{parent}'")));
        }

        let mut declaration = class_declaration(name, Some(parent), parameters);
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
                    declaration.methods.push(self.method_declaration()?);
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

    fn method_declaration(&mut self) -> Result<Arc<Function>> {
        let adoptable = matches!(self.peek_at(1), Token::Identifier(name) if name == "init")
            && self.peek_at(2) == &Token::LeftParen;
        if !adoptable {
            return self.function_declaration();
        }
        let name = self.function_name()?;
        let ParameterList {
            parameters,
            initializers: mut body,
        } = self.constructor_parameters()?;
        body.extend(self.named_body()?);
        Ok(Arc::new(Function {
            name: Some(name),
            parameters,
            body: FunctionBody::Block(body),
        }))
    }

    pub(super) fn declared_type(&mut self) -> Result<String> {
        self.advance();
        self.advance();
        match self.advance() {
            Token::Identifier(type_name) => Ok(type_name),
            other => Err(self.error(format!("Expected a type but found {}", other.describe()))),
        }
    }
}
