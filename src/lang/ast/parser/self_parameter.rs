use std::sync::Arc;

use super::class::{self, CONSTRUCTOR};
use super::{ParameterList, Parser};
use crate::error::Result;
use crate::lang::ast::Function;
use crate::lang::ast::lexer::{Keyword, Spanned, Token};

const PYTHON_CONSTRUCTOR: &str = "__init__";

const RECEIVER_WORDS: [&str; 2] = ["self", "this"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Selves {
    Accepted,
    Dropped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReceiverMethod {
    Constructor,
    Ordinary,
}

fn receiver_window(tokens: &[Spanned], index: usize) -> bool {
    let token = |offset: usize| tokens.get(index + offset).map(|spanned| &spanned.token);
    token(0) == Some(&Token::Keyword(Keyword::Def))
        && matches!(token(1), Some(Token::Identifier(_)))
        && token(2) == Some(&Token::LeftParen)
        && token(3) == Some(&Token::Keyword(Keyword::This))
        && matches!(token(4), Some(Token::Comma | Token::RightParen))
}

fn may_hold_receiver(template: &str) -> bool {
    template.contains(Keyword::Def.as_str())
        && RECEIVER_WORDS.iter().any(|word| template.contains(word))
}

impl Parser {
    pub(super) fn mentions_self_parameters(&self) -> bool {
        self.tokens.iter().enumerate().any(|(index, spanned)| {
            receiver_window(&self.tokens, index)
                || matches!(&spanned.token, Token::Template(template) if may_hold_receiver(template))
        })
    }

    pub(super) fn receiver_method(&self) -> Option<ReceiverMethod> {
        if self.selves == Selves::Accepted || !receiver_window(&self.tokens, self.position) {
            return None;
        }
        Some(match self.peek_at(1) {
            Token::Identifier(name) if name == CONSTRUCTOR || name == PYTHON_CONSTRUCTOR => {
                ReceiverMethod::Constructor
            }
            _ => ReceiverMethod::Ordinary,
        })
    }

    pub(super) fn receiver_declaration(&mut self, method: ReceiverMethod) -> Result<Arc<Function>> {
        let spelling = self.function_keyword()?;
        let written = self.function_name()?;
        let parameters = self.receiver_parameters(method == ReceiverMethod::Constructor)?;
        let body = self.declaration_body(spelling)?;
        let name = match method {
            ReceiverMethod::Constructor => CONSTRUCTOR.to_string(),
            ReceiverMethod::Ordinary => written,
        };
        Ok(class::declared_method(name, parameters, body))
    }

    fn receiver_parameters(&mut self, initializing: bool) -> Result<ParameterList> {
        debug_assert!(
            self.check(&Token::LeftParen) && self.peek_at(1) == &Token::Keyword(Keyword::This)
        );
        self.position += 2;
        self.eat(&Token::Comma);
        self.parameter_items(initializing)
    }
}

#[cfg(test)]
mod tests;
