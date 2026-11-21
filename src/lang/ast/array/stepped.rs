use std::num::NonZeroUsize;

use super::index::integer_or_infinity;
use crate::builtins;
use crate::error::{Error, Result};
use crate::lang::ast::Expr;
use crate::number;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

const STEP_ERROR: &str = "Slice step must be a nonzero number";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Step {
    direction: Direction,
    stride: NonZeroUsize,
}

impl Step {
    fn of(step: Option<&Value>) -> Result<Self> {
        let amount = step.map_or(1.0, Value::to_number);
        let stride = NonZeroUsize::new(number::to_usize(amount.abs()))
            .ok_or_else(|| Error::type_error(STEP_ERROR))?;

        let direction = if amount < 0.0 {
            Direction::Backward
        } else {
            Direction::Forward
        };

        Ok(Step { direction, stride })
    }

    fn select<T, C: FromIterator<T>>(
        self,
        items: Vec<T>,
        start: Option<&Value>,
        end: Option<&Value>,
    ) -> C {
        let length = items.len();
        let stride = self.stride.get();

        match self.direction {
            Direction::Forward => {
                let bound = |value: &Value| builtins::slice_index(value.to_number(), length);
                let first = start.map_or(0, bound);
                let stop = end.map_or(length, bound);
                items
                    .into_iter()
                    .take(stop)
                    .skip(first)
                    .step_by(stride)
                    .collect()
            }
            Direction::Backward => {
                let bound = |value: &Value| past_backward_bound(value.to_number(), length);
                let past_first = start.map_or(length, bound);
                let past_stop = end.map_or(0, bound);
                items
                    .into_iter()
                    .take(past_first)
                    .skip(past_stop)
                    .rev()
                    .step_by(stride)
                    .collect()
            }
        }
    }
}

fn past_backward_bound(index: f64, length: usize) -> usize {
    let size = number::from_usize(length);
    let counted = if index < 0.0 { size + index } else { index };
    number::to_usize((integer_or_infinity(counted) + 1.0).max(0.0).min(size))
}

fn stepped(base: Value, start: Option<&Value>, end: Option<&Value>, step: Step) -> Value {
    match base {
        Value::List(items) => Value::List(step.select(items, start, end)),
        Value::String(string) => {
            let characters: Vec<char> = string.chars().collect();
            Value::String(step.select(characters, start, end))
        }
        _ => Value::Undefined,
    }
}

impl Runtime {
    pub(crate) fn evaluate_stepped_slice(
        &mut self,
        object: &Expr,
        start: Option<&Expr>,
        end: Option<&Expr>,
        step: Option<&Expr>,
        scope: &mut Scope,
    ) -> Result<Value> {
        let base = self.slice_base(object, scope)?;
        let start = self.slice_part(start, scope)?;
        let end = self.slice_part(end, scope)?;
        let step = Step::of(self.slice_part(step, scope)?.as_ref())?;
        Ok(stepped(base, start.as_ref(), end.as_ref(), step))
    }
}
