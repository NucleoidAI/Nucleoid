use std::cmp::Ordering;

use super::aggregate::is_unordered;
use super::query::expect_function;
use crate::error::Result;
use crate::lang::ast::operator::compare;
use crate::runtime::Runtime;
use crate::value::Value;

#[must_use]
fn text(key: &Value) -> Option<&str> {
    match key {
        Value::String(text) => Some(text),
        _ => None,
    }
}

#[must_use]
fn without_negative_zero(number: f64) -> f64 {
    if number == 0.0 { 0.0 } else { number }
}

#[must_use]
fn by_text(mut keyed: Vec<(Value, Value)>) -> Vec<Value> {
    debug_assert!(keyed.iter().all(|(key, _)| text(key).is_some()));
    keyed.sort_by(|(left, _), (right, _)| text(left).cmp(&text(right)));
    keyed.into_iter().map(|(_, item)| item).collect()
}

#[must_use]
fn by_number(keyed: Vec<(Value, Value)>) -> Vec<Value> {
    debug_assert!(
        keyed
            .iter()
            .all(|(key, _)| text(key).is_none() && !is_unordered(key))
    );
    let mut numbered: Vec<(f64, Value)> = keyed
        .into_iter()
        .map(|(key, item)| (without_negative_zero(key.to_number()), item))
        .collect();
    numbered.sort_by(|(left, _), (right, _)| left.total_cmp(right));
    numbered.into_iter().map(|(_, item)| item).collect()
}

#[must_use]
fn by_placement(keyed: Vec<(Value, Value)>) -> Vec<Value> {
    let mut placed: Vec<(Value, Value)> = Vec::with_capacity(keyed.len());

    for (key, item) in keyed {
        let index = placed
            .iter()
            .position(|(other, _)| compare(&key, other).is_some_and(Ordering::is_lt))
            .unwrap_or(placed.len());
        placed.insert(index, (key, item));
    }

    placed.into_iter().map(|(_, item)| item).collect()
}

#[must_use]
fn ordered(keyed: Vec<(Value, Value)>) -> Vec<Value> {
    let texts = keyed.iter().filter(|(key, _)| text(key).is_some()).count();

    if texts == 0 {
        by_number(keyed)
    } else if texts == keyed.len() {
        by_text(keyed)
    } else {
        by_placement(keyed)
    }
}

impl Runtime {
    pub(super) fn sort_by_key(
        &mut self,
        items: Vec<Value>,
        predicate: Option<Value>,
    ) -> Result<Value> {
        let selector = expect_function(predicate, "sortBy")?;
        let mut keyed = Vec::with_capacity(items.len());
        let mut unordered = false;

        for item in items {
            let key = self.invoke(&selector, std::slice::from_ref(&item))?;
            unordered |= is_unordered(&key);
            keyed.push((key, item));
        }

        Ok(if unordered {
            Value::Undefined
        } else {
            Value::List(ordered(keyed))
        })
    }
}

#[cfg(test)]
mod tests;
