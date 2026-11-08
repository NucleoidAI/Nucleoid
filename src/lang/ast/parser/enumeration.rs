use indexmap::IndexMap;

use super::{Parser, ScalarLiteral};
use crate::error::Result;
use crate::lang::ast::lexer::Token;
use crate::lang::ast::{EnumMember, Expr, ParameterDefault, Stmt};

const KEYWORD: &str = "enum";

const COLON_STATEMENT_NAMES: [&str; 3] = ["assert", "lambda", "suppose"];

impl Parser {
    pub(super) fn enum_header(&self) -> Option<String> {
        let starts = !self.in_class
            && matches!(self.peek(), Token::Identifier(word) if word == KEYWORD)
            && self.peek_at(2) == &Token::Colon
            && !self.is_type_declaration_at(1);
        match self.peek_at(1) {
            Token::Identifier(name)
                if starts && !COLON_STATEMENT_NAMES.contains(&name.as_str()) =>
            {
                Some(name.clone())
            }
            _ => None,
        }
    }

    pub(super) fn enum_declaration(&mut self, name: String) -> Result<Stmt> {
        self.position += 3;
        let members = if matches!(self.peek(), Token::Newline | Token::Indent) {
            self.enum_block(&name)?
        } else {
            self.enum_line(&name)?
        };
        Ok(Stmt::Assign {
            target: Expr::Identifier(name),
            value: Expr::Enum(
                members
                    .into_iter()
                    .map(|(member, value)| EnumMember {
                        name: member,
                        value,
                    })
                    .collect(),
            ),
        })
    }

    fn enum_line(&mut self, name: &str) -> Result<IndexMap<String, ParameterDefault>> {
        let mut members = IndexMap::new();
        loop {
            self.enum_member(name, &mut members)?;
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        if !self.ends_line_statement() {
            self.expect(&Token::Comma)?;
        }
        if members.len() < 2 {
            return Err(self.error(format!(
                "Enum '{name}' needs at least two members on one line"
            )));
        }
        Ok(members)
    }

    fn enum_block(&mut self, name: &str) -> Result<IndexMap<String, ParameterDefault>> {
        while self.eat(&Token::Newline) {}
        self.expect(&Token::Indent)?;
        let mut members = IndexMap::new();
        loop {
            while self.eat(&Token::Newline) {}
            if matches!(self.peek(), Token::Dedent | Token::Eof) {
                break;
            }
            self.enum_member(name, &mut members)?;
            if !matches!(self.peek(), Token::Newline | Token::Dedent | Token::Eof) {
                self.expect(&Token::Newline)?;
            }
        }
        self.eat(&Token::Dedent);
        Ok(members)
    }

    fn enum_member(
        &mut self,
        name: &str,
        members: &mut IndexMap<String, ParameterDefault>,
    ) -> Result<()> {
        let member = match self.peek() {
            Token::Identifier(member) => member.clone(),
            other => {
                return Err(self.error(format!("Expected a name but found {}", other.describe())));
            }
        };
        if members.contains_key(&member) {
            return Err(self.error(format!("Enum '{name}' already has member '{member}'")));
        }
        self.advance();
        let value = if self.eat(&Token::Assign) {
            self.enum_value()?
        } else {
            ParameterDefault::String(member.clone())
        };
        members.insert(member, value);
        Ok(())
    }

    fn enum_value(&mut self) -> Result<ParameterDefault> {
        match self.scalar_literal() {
            ScalarLiteral::Value(value) => Ok(value),
            ScalarLiteral::NotFinite => {
                Err(self.error("Enum member value must be a finite number"))
            }
            ScalarLiteral::Missing => {
                Err(self.error("Expected a scalar literal enum member value"))
            }
        }
    }
}
