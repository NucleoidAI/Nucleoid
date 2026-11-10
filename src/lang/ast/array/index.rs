use super::ListSource;
use crate::error::Result;
use crate::graph::NodeKey;
use crate::lang::ast::Expr;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;
use crate::{builtins, number};

pub(super) fn list_at(items: Vec<Value>, index: Option<&Value>, source: ListSource) -> Value {
    let index = index.map_or(f64::NAN, Value::to_number);
    relative_index(index, items.len())
        .and_then(|position| items.into_iter().nth(position))
        .unwrap_or_else(|| source.missing())
}

/// A found position as a number, or `-1` when nothing was found.
pub(super) fn position_number(position: Option<usize>) -> Value {
    Value::Number(position.map_or(-1.0, number::from_usize))
}

pub(super) fn integer_or_infinity(number: f64) -> f64 {
    if number.is_nan() { 0.0 } else { number.trunc() }
}

/// Resolves the index of `at` the way JavaScript does: the number is
/// truncated, NaN reads as zero and a negative index counts from the end.
/// Unlike bracket indexing, an index outside the list is never clamped into
/// it, so `None` means there is no such position.
fn relative_index(index: f64, length: usize) -> Option<usize> {
    let index = integer_or_infinity(index);
    let position = if index < 0.0 {
        index + number::from_usize(length)
    } else {
        index
    };

    (0.0..number::from_usize(length))
        .contains(&position)
        .then_some(number::to_usize(position))
}

#[must_use]
pub fn slice(base: &Value, start: Option<&Value>, end: Option<&Value>) -> Value {
    match base {
        Value::String(string) => Value::String(builtins::string_slice(
            string,
            start.map_or(0.0, Value::to_number),
            end.map(Value::to_number),
        )),

        Value::List(items) => Value::List(slice_items(items, start, end)),

        _ => Value::Undefined,
    }
}

/// The items from `start` up to `end`, with the bound rules that ranges and
/// `slice` share: an omitted start or end is the start or the end of the list,
/// a negative bound counts from the end, every bound is clamped to the list,
/// and a start at or after the end gives no items.
pub(super) fn slice_items(
    items: &[Value],
    start: Option<&Value>,
    end: Option<&Value>,
) -> Vec<Value> {
    let length = items.len();
    let bound = |value: &Value| builtins::slice_index(value.to_number(), length);
    let start = start.map_or(0, bound);
    let end = end.map_or(length, bound);

    items
        .get(start.min(end)..end)
        .map(<[Value]>::to_vec)
        .unwrap_or_default()
}

impl Runtime {
    pub(crate) fn evaluate_slice(
        &mut self,
        object: &Expr,
        start: Option<&Expr>,
        end: Option<&Expr>,
        scope: &mut Scope,
    ) -> Result<Value> {
        // A class's instances are read before the bounds, as `Name.slice()`
        // reads them before its arguments, and sliced like a list.
        let base = match self.evaluate(object, scope)? {
            Value::Class(name) => match self.population(&name) {
                Some(instances) => Value::List(instances),
                None => Value::Class(name),
            },
            base => base,
        };

        let start = match start {
            Some(start) => Some(self.evaluate(start, scope)?),
            None => None,
        };
        let end = match end {
            Some(end) => Some(self.evaluate(end, scope)?),
            None => None,
        };

        Ok(slice(&base, start.as_ref(), end.as_ref()))
    }

    pub(crate) fn evaluate_index(
        &mut self,
        object: &Expr,
        index: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let base = self.evaluate(object, scope)?;
        let key = self.evaluate(index, scope)?;
        Ok(self.read_index(&base, &key))
    }

    pub(crate) fn read_index(&mut self, base: &Value, key: &Value) -> Value {
        match base {
            Value::List(items) => {
                let index = key.to_number();
                let index = builtins::slice_index(index, items.len());
                items.get(index).cloned().unwrap_or(Value::Undefined)
            }

            Value::String(string) => {
                let index = builtins::slice_index(key.to_number(), string.chars().count());
                string
                    .chars()
                    .nth(index)
                    .map_or(Value::Undefined, |character| {
                        Value::String(character.to_string())
                    })
            }

            Value::Class(name) => {
                let Some(class) = self.state.class(name) else {
                    return Value::Undefined;
                };

                let instances = class.instances.clone();
                self.track(&NodeKey::class(name));

                let found = match key {
                    Value::Number(index) => instances.get(number::to_usize(*index)).cloned(),
                    other => {
                        let wanted = other.to_string();
                        instances
                            .iter()
                            .find(|instance| instance.as_str() == wanted)
                            .cloned()
                    }
                };

                match found {
                    Some(id) => Value::Object(id),
                    None => Value::Null,
                }
            }

            Value::Object(_) => self.read_property(base, &key.to_string()),

            _ => Value::Undefined,
        }
    }
}
