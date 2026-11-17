use super::{ExpressionContext, Parser, furthest};
use crate::error::{Error, Result};
use crate::lang::ast::lexer::Token;
use crate::lang::ast::{Expr, Function};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Sections {
    Accepted,
    Read,
}

fn section_operator(token: &Token) -> bool {
    matches!(
        token,
        Token::Equal
            | Token::NotEqual
            | Token::StrictEqual
            | Token::StrictNotEqual
            | Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual
            | Token::Star
            | Token::Percent
            | Token::StarStar
            | Token::AndAnd
            | Token::OrOr
            | Token::QuestionQuestion
    )
}

impl Parser {
    pub(super) fn attempt_sections<T>(
        &mut self,
        parse: &impl Fn(&mut Self) -> Result<T>,
        accepted: Error,
    ) -> Result<T> {
        if !self.mentions_sections() {
            return Err(accepted);
        }
        *self = Parser {
            sections: Sections::Read,
            ..Parser::new(std::mem::take(&mut self.tokens))
        };
        self.attempt_selves(parse)
            .map_err(|sectioned| furthest(accepted, sectioned))
    }

    fn mentions_sections(&self) -> bool {
        self.mentions_templates()
            || self.tokens.windows(2).any(|pair| {
                matches!(pair, [opening, operator]
                    if matches!(opening.token, Token::LeftParen | Token::Comma)
                        && section_operator(&operator.token))
            })
    }

    pub(super) fn starts_section(&self) -> bool {
        self.sections == Sections::Read && section_operator(self.peek())
    }

    pub(super) fn section(&mut self) -> Result<Expr> {
        self.nested_expression_in(ExpressionContext::Ordinary, Self::section_function)
    }

    fn section_function(&mut self) -> Result<Expr> {
        debug_assert!(self.operand.is_none());
        let items = self.leave_items();
        self.operand = Some(Function::unnamed_parameter());
        let body = self.without_instance(|parser| parser.expression_in(parser.context));
        let unread = self.operand.take();
        self.items = items;
        let body = body?;
        debug_assert!(unread.is_none());
        Ok(Expr::Function(Function::section(body)))
    }
}

#[cfg(test)]
mod tests;
