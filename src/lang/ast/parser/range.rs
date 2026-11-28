use super::postfix::expected_property;
use super::{Parser, furthest};
use crate::error::{Error, Position, Result};
use crate::lang::ast::BinaryOp;
use crate::lang::ast::lexer::{Spanned, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Ranges {
    Accepted,
    Read,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Reservation {
    Open,
    Ranges,
}

fn touches(first: &Spanned, second: &Spanned) -> bool {
    second.position.line == first.position.line
        && second.position.column == first.position.column + 1
}

fn operator(window: &[Spanned]) -> Option<BinaryOp> {
    let [first, second, third] = window else {
        return None;
    };
    let operator = match (&first.token, &second.token, &third.token) {
        (Token::Dot, Token::Dot, Token::Less) => BinaryOp::Range,
        (Token::Dot, Token::Dot, Token::Assign) => BinaryOp::RangeInclusive,
        _ => return None,
    };
    (touches(first, second) && touches(second, third)).then_some(operator)
}

fn mentions_range(spanned: &Spanned) -> bool {
    matches!(&spanned.token, Token::Template(raw) if raw.contains("..<") || raw.contains("..="))
}

fn reserved_range(count: &[Spanned]) -> Option<&Spanned> {
    let mut depth = 0usize;
    for (index, spanned) in count.iter().enumerate() {
        match spanned.token {
            Token::LeftParen | Token::LeftBracket | Token::LeftBrace => depth += 1,
            Token::RightParen | Token::RightBracket | Token::RightBrace => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
        let window = count.get(index..index + 3).unwrap_or_default();
        if depth == 0 && operator(window).is_some() {
            return window.last();
        }
    }
    None
}

impl Parser {
    pub(super) fn attempt_ranges<T>(
        &mut self,
        parse: &impl Fn(&mut Self) -> Result<T>,
        accepted: Error,
        line_breaks: Vec<Position>,
    ) -> Result<T> {
        if !self.mentions_ranges() {
            return Err(accepted);
        }
        *self = Parser {
            line_breaks,
            ranges: Ranges::Read,
            ..Parser::new(std::mem::take(&mut self.tokens))
        };
        self.attempt_lines(parse)
            .map_err(|ranged| furthest(accepted, ranged))
    }

    fn mentions_ranges(&self) -> bool {
        self.tokens
            .windows(3)
            .any(|window| operator(window).is_some())
            || self.tokens.iter().any(mentions_range)
    }

    pub(super) fn range_operator(&self) -> Option<BinaryOp> {
        if self.ranges == Ranges::Accepted || self.reservation == Reservation::Ranges {
            return None;
        }
        self.tokens
            .get(self.position..self.position + 3)
            .and_then(operator)
    }

    pub(super) fn reserved<T>(&mut self, parse: impl FnOnce(&mut Self) -> T) -> T {
        self.reserving(Reservation::Ranges, parse)
    }

    pub(super) fn unreserved<T>(&mut self, parse: impl FnOnce(&mut Self) -> T) -> T {
        self.reserving(Reservation::Open, parse)
    }

    fn reserving<T>(&mut self, reservation: Reservation, parse: impl FnOnce(&mut Self) -> T) -> T {
        let outer = std::mem::replace(&mut self.reservation, reservation);
        let parsed = parse(self);
        self.reservation = outer;
        parsed
    }

    pub(super) fn reserve_count(&self, start: usize) -> Result<()> {
        if self.ranges == Ranges::Accepted {
            return Ok(());
        }
        let count = self.tokens.get(start..self.position).unwrap_or_default();
        match reserved_range(count) {
            Some(third) => Err(Error::syntax(expected_property(&Token::Dot)).at(third.position)),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests;
