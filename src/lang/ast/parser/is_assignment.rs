use super::expression::PathTarget;
use super::{ExpressionContext, Parser};
use crate::error::Result;
use crate::lang::ast::lexer::{Keyword, SyntaxOptions, Token};
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

fn is_word(start: usize, end: usize, value: &Expr) -> bool {
    end == start + 1 && matches!(value, Expr::Identifier(_))
}

fn without_words(mut target: Expr) -> Expr {
    let mut pending = vec![&mut target];
    while let Some(expression) = pending.pop() {
        match expression {
            Expr::Unary {
                operator: UnaryOp::Word,
                operand,
            } => {
                let name = std::mem::replace(operand.as_mut(), Expr::Null);
                *expression = name;
            }
            Expr::Conditional {
                consequent,
                alternate,
                ..
            } => {
                pending.push(consequent.as_mut());
                pending.push(alternate.as_mut());
            }
            _ => {}
        }
    }
    target
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Branches {
    Plain,
    Words,
}

impl Branches {
    fn of(options: SyntaxOptions) -> Self {
        if options.conditional_words {
            Branches::Words
        } else {
            Branches::Plain
        }
    }

    pub(super) fn mark(self, start: usize, end: usize, branch: Expr) -> Expr {
        if self == Branches::Words && is_word(start, end, &branch) {
            word(branch)
        } else {
            branch
        }
    }

    pub(super) fn unmark(self, target: Expr) -> Expr {
        match self {
            Branches::Plain => target,
            Branches::Words => without_words(target),
        }
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
        let branches = Branches::of(self.options);
        let value = self.nested_expression(|parser| {
            parser.nested_expression_in(ExpressionContext::Ordinary, |value| {
                value.assignment_expression(branches)
            })
        })?;
        let bare = is_word(start, self.position, &value);
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
