use std::fmt;

use super::shown::{Items, Mode, Shown};
use crate::expression::visit_children;
use crate::lang::ast::{Expr, UnaryOp};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Found {
    Nothing,
    Words,
    ItemWords,
}

fn found_words(value: &Expr) -> Found {
    let mut found = Found::Nothing;
    let mut pending = vec![(value, true)];
    while let Some((expression, chain)) = pending.pop() {
        match expression {
            Expr::Unary {
                operator: UnaryOp::Word,
                ..
            } if chain => found = Found::Words,
            Expr::Unary {
                operator: UnaryOp::Word,
                ..
            } => return Found::ItemWords,
            Expr::Conditional {
                condition,
                consequent,
                alternate,
            } if chain => pending.extend([
                (condition.as_ref(), false),
                (consequent.as_ref(), true),
                (alternate.as_ref(), true),
            ]),
            expression => visit_children(expression, &mut |child| pending.push((child, false))),
        }
    }
    found
}

pub(super) fn branch_words(value: &Expr) -> bool {
    let mut pending = vec![value];
    while let Some(expression) = pending.pop() {
        match expression {
            Expr::Unary {
                operator: UnaryOp::Word,
                ..
            } => return true,
            Expr::Conditional {
                consequent,
                alternate,
                ..
            } => pending.extend([consequent.as_ref(), alternate.as_ref()]),
            _ => {}
        }
    }
    false
}

pub(crate) struct WordValue<'a> {
    value: &'a Expr,
    items: Items,
}

impl<'a> WordValue<'a> {
    pub(crate) fn of(value: &'a Expr) -> Option<Self> {
        let items = match found_words(value) {
            Found::Nothing => return None,
            Found::Words => Items::Plain,
            Found::ItemWords => Items::Words,
        };
        Some(WordValue { value, items })
    }
}

impl fmt::Display for WordValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown = Shown {
            expression: self.value,
            mode: Mode::Position(self.items),
        };
        fmt::Display::fmt(&shown, f)
    }
}
