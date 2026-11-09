use super::Parser;
use super::is_assignment::Items;
use crate::error::{Error, Result};
use crate::lang::ast::lexer::{EXPECTED_VERSION_SELECTOR, Token};
use crate::lang::ast::{Expr, VersionSelector};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Rejection {
    Character,
    Selector,
}

impl Rejection {
    fn message(self) -> &'static str {
        match self {
            Rejection::Character => "Unexpected character '@'",
            Rejection::Selector => EXPECTED_VERSION_SELECTOR,
        }
    }

    fn offset(self) -> usize {
        match self {
            Rejection::Character => 0,
            Rejection::Selector => 1,
        }
    }
}

impl Parser {
    pub(super) fn at_variable(&mut self) -> Result<Expr> {
        let at = self.position;
        match self.variable_after(at) {
            Some(name) if self.items != Items::Plain => {
                let variable = Expr::Identifier(name.to_string());
                self.position += 2;
                self.at_variables.entry(at).or_insert(Rejection::Character);
                Ok(variable)
            }
            _ => Err(self.error(Rejection::Character.message())),
        }
    }

    pub(super) fn starts_at_variable(&self, offset: usize) -> bool {
        self.variable_after(self.position + offset)
            .is_some_and(|name| VersionSelector::from_name(name).is_none())
    }

    pub(super) fn record_value_start(&mut self) {
        if self.check(&Token::At) {
            self.at_variables.insert(self.position, Rejection::Selector);
        }
    }

    pub(super) fn enter_variables(&mut self) -> Items {
        let items = self.items;
        if items == Items::Plain {
            self.items = Items::Variables;
        }
        items
    }

    pub(super) fn at_variable_since(&self, start: usize) -> Option<(usize, Rejection)> {
        debug_assert!(start <= self.position);
        let (&at, &rejection) = self.at_variables.range(start..).next()?;
        (at < self.position).then_some((at, rejection))
    }

    pub(super) fn reject_at_variable(&self, (at, rejection): (usize, Rejection)) -> Result<Expr> {
        let position = self
            .tokens
            .get(at + rejection.offset())
            .map_or_else(|| self.here(), |spanned| spanned.position);
        Err(Error::syntax(rejection.message()).at(position))
    }

    fn variable_after(&self, at: usize) -> Option<&str> {
        let [mark, next] = self.tokens.get(at..at + 2)? else {
            return None;
        };
        let adjacent = mark.token == Token::At
            && next.position.line == mark.position.line
            && next.position.column == mark.position.column + 1;
        match &next.token {
            Token::Identifier(name) if adjacent => Some(name),
            _ => None,
        }
    }
}
