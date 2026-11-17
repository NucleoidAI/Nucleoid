use std::mem;

use super::Parser;
use super::expression::PathTarget;
use crate::error::{Error, Result};
use crate::lang::ast::Expr;
use crate::lang::ast::lexer::{Keyword, Spanned, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Dots {
    Accepted,
    Receivers,
    Instances,
}

impl Dots {
    pub(super) fn probed(self) -> Self {
        match self {
            Dots::Accepted => Dots::Accepted,
            Dots::Receivers | Dots::Instances => Dots::Instances,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum Receiver {
    Absent,
    Instance,
    Path(Expr),
    Unreadable,
}

impl Receiver {
    fn of(target: &Expr) -> Self {
        let Expr::Member { object, .. } = target else {
            return Receiver::Absent;
        };
        match PathTarget::of(target) {
            PathTarget::Plain => Receiver::Path(object.as_ref().clone()),
            PathTarget::TooManyParts => Receiver::Unreadable,
            PathTarget::Invalid => Receiver::Absent,
        }
    }
}

fn receiver_dot(tokens: &[Spanned], index: usize) -> bool {
    let token = |offset: usize| tokens.get(index + offset).map(|spanned| &spanned.token);
    token(0) == Some(&Token::Dot)
        && matches!(token(1), Some(Token::Identifier(_) | Token::Keyword(_)))
        && index
            .checked_sub(1)
            .and_then(|previous| tokens.get(previous))
            .is_none_or(|previous| previous.token.precedes_regex())
}

impl Parser {
    pub(super) fn mentions_receivers(&self) -> bool {
        (0..self.tokens.len()).any(|index| receiver_dot(&self.tokens, index))
    }

    pub(super) fn mentions_templates(&self) -> bool {
        self.tokens
            .iter()
            .any(|spanned| matches!(spanned.token, Token::Template(_)))
    }

    fn reads_at(&self, index: usize) -> bool {
        debug_assert!(self.dots != Dots::Accepted || matches!(self.receiver, Receiver::Absent));
        let reads = match self.receiver {
            Receiver::Absent => false,
            Receiver::Instance => true,
            Receiver::Path(_) | Receiver::Unreadable => self.dots == Dots::Receivers,
        };
        reads && receiver_dot(&self.tokens, index)
    }

    pub(super) fn reads_receiver(&self) -> bool {
        self.reads_at(self.primary_head())
    }

    fn primary_head(&self) -> usize {
        let news = self.tokens.get(self.position..).map_or(0, |rest| {
            rest.iter()
                .take_while(|spanned| spanned.token == Token::Keyword(Keyword::New))
                .count()
        });
        self.position + news
    }

    pub(super) fn reads_behind(&self) -> bool {
        self.position
            .checked_sub(1)
            .is_some_and(|dot| self.reads_at(dot))
    }

    pub(super) fn receiver_read(&mut self) -> Result<Expr> {
        debug_assert!(self.reads_behind());
        let object = match &self.receiver {
            Receiver::Instance => Expr::This,
            Receiver::Path(path) => path.clone(),
            Receiver::Absent | Receiver::Unreadable => {
                let dot = self.previous_position();
                return Err(Error::syntax("Expression has too many parts").at(dot));
            }
        };
        let property = self.member_property()?;
        Ok(Expr::Member {
            object: Box::new(object),
            property,
        })
    }

    pub(super) fn receiving(
        &mut self,
        target: &Expr,
        parse: impl FnOnce(&mut Self) -> Result<Expr>,
    ) -> Result<Expr> {
        let receiver = match (self.dots, target) {
            (Dots::Accepted, _) => Receiver::Absent,
            (Dots::Receivers | Dots::Instances, Expr::Identifier(_)) => self.variable_receiver(),
            (Dots::Receivers | Dots::Instances, _) => Receiver::of(target),
        };
        self.with_receiver(receiver, parse)
    }

    fn variable_receiver(&self) -> Receiver {
        match self.receiver {
            Receiver::Instance => Receiver::Instance,
            Receiver::Absent | Receiver::Path(_) | Receiver::Unreadable => Receiver::Absent,
        }
    }

    pub(super) fn variable_value<T>(&mut self, parse: impl FnOnce(&mut Self) -> T) -> T {
        let receiver = self.variable_receiver();
        self.with_receiver(receiver, parse)
    }

    pub(super) fn with_instance<T>(&mut self, parse: impl FnOnce(&mut Self) -> T) -> T {
        if self.dots == Dots::Accepted || !matches!(self.receiver, Receiver::Absent) {
            return parse(self);
        }
        self.with_receiver(Receiver::Instance, parse)
    }

    pub(super) fn holds_instance(&self) -> bool {
        matches!(self.receiver, Receiver::Instance)
    }

    pub(super) fn without_instance<T>(&mut self, parse: impl FnOnce(&mut Self) -> T) -> T {
        if self.holds_instance() {
            return self.without_receiver(parse);
        }
        parse(self)
    }

    fn without_receiver<T>(&mut self, parse: impl FnOnce(&mut Self) -> T) -> T {
        self.with_receiver(Receiver::Absent, parse)
    }

    pub(super) fn without_reads<T>(&mut self, parse: impl FnOnce(&mut Self) -> T) -> T {
        let mode = mem::replace(&mut self.dots, Dots::Accepted);
        let parsed = self.without_receiver(parse);
        self.dots = mode;
        parsed
    }

    fn with_receiver<T>(&mut self, receiver: Receiver, parse: impl FnOnce(&mut Self) -> T) -> T {
        let outer = mem::replace(&mut self.receiver, receiver);
        let parsed = parse(self);
        self.receiver = outer;
        parsed
    }

    pub(super) fn without_head_read<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        if self.check(&Token::Dot) {
            self.without_receiver(parse)
        } else {
            parse(self)
        }
    }

    pub(super) fn check_target_start(&self, start: usize) -> Result<()> {
        let led_by_dot = self
            .tokens
            .get(start)
            .is_some_and(|spanned| spanned.token == Token::Dot);
        if self.dots == Dots::Accepted || !led_by_dot {
            return Ok(());
        }
        let position = self
            .tokens
            .get(start + 1)
            .map_or_else(|| self.here(), |spanned| spanned.position);
        Err(Error::syntax("Unexpected '.'").at(position))
    }
}

#[cfg(test)]
mod tests;
