use super::Parser;
use crate::error::Result;
use crate::lang::ast::lexer::Token;
use crate::lang::ast::{Expr, Stmt};

impl Parser {
    pub(super) fn starts_predicate(&self, subject: &Expr) -> bool {
        matches!(
            (subject, self.peek(), self.peek_at(1)),
            (Expr::ClassRef(_), Token::Identifier(is), Token::Identifier(_)) if is == "is"
        ) && self.ends_line_statement_at(2)
    }

    pub(super) fn predicate(&mut self, subject: &Expr) -> Result<Stmt> {
        self.advance();
        match self.advance() {
            Token::Identifier(property) => Ok(Stmt::Assign {
                target: Expr::Member {
                    object: Box::new(subject.clone()),
                    property,
                },
                value: Expr::Bool(true),
            }),
            other => Err(self.error(format!("Expected a name but found {}", other.describe()))),
        }
    }
}
