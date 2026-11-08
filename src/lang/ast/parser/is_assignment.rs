use super::Parser;
use super::expression::PathTarget;
use crate::error::Result;
use crate::lang::ast::lexer::{Keyword, Token};
use crate::lang::ast::{Expr, Stmt, UnaryOp};

fn begins_expression(token: &Token) -> bool {
    matches!(
        token,
        Token::Identifier(_)
            | Token::ClassRef(_)
            | Token::Number(_)
            | Token::Str(_)
            | Token::Template(_)
            | Token::Regex(_)
            | Token::LeftParen
            | Token::LeftBracket
            | Token::LeftBrace
            | Token::Bang
            | Token::Minus
            | Token::Plus
            | Token::Keyword(
                Keyword::True
                    | Keyword::False
                    | Keyword::Null
                    | Keyword::This
                    | Keyword::Super
                    | Keyword::New
                    | Keyword::Delete
                    | Keyword::Function
                    | Keyword::Not
                    | Keyword::TypeOf
            )
    )
}

fn word(value: Expr) -> Expr {
    Expr::Unary {
        operator: UnaryOp::Word,
        operand: Box::new(value),
    }
}

impl Parser {
    pub(super) fn starts_is_assignment(&self, target: &Expr) -> bool {
        self.options.is_assignment
            && matches!(self.peek(), Token::Identifier(name) if name == "is")
            && begins_expression(self.peek_at(1))
            && PathTarget::of(target) == PathTarget::Plain
    }

    pub(super) fn is_assignment(&mut self, target: Expr) -> Result<Stmt> {
        self.advance();
        let start = self.position;
        let value = self.nested_expression(Self::expression)?;
        let bare = self.position == start + 1 && matches!(value, Expr::Identifier(_));
        let statement = self.expression_tail(Expr::Assign {
            target: Box::new(target),
            value: Box::new(value),
        })?;
        Ok(match statement {
            Stmt::Assign {
                target: path,
                value: name,
            } if bare => Stmt::Assign {
                target: path,
                value: word(name),
            },
            statement => statement,
        })
    }
}
