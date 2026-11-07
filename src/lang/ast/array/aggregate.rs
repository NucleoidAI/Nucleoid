use std::cmp::Ordering;
use std::sync::Arc;

use super::query::{expect_function, optional_selector};
use crate::error::Result;
use crate::lang::ast::operator::{binary, compare};
use crate::lang::ast::{BinaryOp, Function};
use crate::runtime::Runtime;
use crate::value::Value;
use crate::{builtins, number};

pub(super) fn join_items(items: &[Value], separator: Option<Value>) -> Value {
    let separator = separator.map(|value| value.to_string()).unwrap_or_default();
    let rendered: Vec<String> = items.iter().map(ToString::to_string).collect();
    Value::String(rendered.join(&separator))
}

pub(super) fn concat_items(items: Vec<Value>, arguments: &[Value]) -> Vec<Value> {
    let mut joined = items;

    for argument in arguments {
        match argument {
            Value::List(other) => joined.extend(other.iter().cloned()),
            other => joined.push(other.clone()),
        }
    }

    joined
}

#[must_use]
fn median_value(values: &[Value]) -> Value {
    if values.iter().any(Value::is_undefined) {
        return Value::Undefined;
    }

    let mut numbers: Vec<f64> = values.iter().map(Value::to_number).collect();

    if numbers.iter().any(|number| number.is_nan()) {
        return Value::Number(f64::NAN);
    }

    numbers.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

    let median = match numbers.split_at_checked(numbers.len() / 2) {
        Some((lower, upper)) => match (lower.last(), upper.first()) {
            (Some(low), Some(high)) if lower.len() == upper.len() => (low + high) / 2.0,
            (_, Some(center)) => *center,
            (_, None) => f64::NAN,
        },
        None => f64::NAN,
    };

    Value::Number(median)
}

impl Runtime {
    pub(super) fn aggregate_items(
        &mut self,
        items: Vec<Value>,
        property: &str,
        predicate: Option<Value>,
    ) -> Result<Value> {
        let selector = optional_selector(predicate, property)?;

        match property {
            "sum" => return self.sum_items(items, selector.as_ref()),
            "average" => return self.average_items(items, selector.as_ref()),
            _ => {}
        }

        let values = self.select_items(items, selector.as_ref())?;

        Ok(match property {
            "distinct" => Value::List(self.distinct_values(values)),
            "mode" => self.mode_value(values),
            "median" => median_value(&values),
            _ => builtins::math(property, &values)?,
        })
    }

    pub(super) fn extreme_item(
        &mut self,
        items: Vec<Value>,
        property: &str,
        predicate: Option<Value>,
    ) -> Result<Value> {
        let selector = expect_function(predicate, property)?;
        let wanted = if property == "maxBy" {
            Ordering::Greater
        } else {
            Ordering::Less
        };
        let mut extreme: Option<(Value, Value)> = None;
        let mut unordered = false;

        for item in items {
            let key = self.invoke(&selector, std::slice::from_ref(&item))?;

            if compare(&key, &key).is_none() {
                unordered = true;
            } else if extreme
                .as_ref()
                .is_none_or(|(_, kept)| compare(&key, kept) == Some(wanted))
            {
                extreme = Some((item, key));
            }
        }

        Ok(match extreme {
            Some((item, _)) if !unordered => item,
            _ => Value::Undefined,
        })
    }
    fn sum_items(&mut self, items: Vec<Value>, selector: Option<&Arc<Function>>) -> Result<Value> {
        let mut total = 0.0;
        let mut undefined = false;

        for item in items {
            let value = match selector {
                Some(function) => self.invoke(function, &[item])?,
                None => item,
            };
            undefined |= value.is_undefined();
            total += value.to_number();
        }

        Ok(if undefined {
            Value::Undefined
        } else {
            Value::Number(total)
        })
    }

    fn average_items(
        &mut self,
        items: Vec<Value>,
        selector: Option<&Arc<Function>>,
    ) -> Result<Value> {
        let count = Value::Number(number::from_usize(items.len()));
        let total = self.sum_items(items, selector)?;
        binary(BinaryOp::Divide, &total, &count)
    }
}
