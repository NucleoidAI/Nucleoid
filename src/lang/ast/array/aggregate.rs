use std::cmp::Ordering;
use std::sync::Arc;

use super::query::{expect_function, optional_selector};
use crate::error::{Error, Result};
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
pub(super) fn is_unordered(key: &Value) -> bool {
    compare(key, key).is_none()
}

#[must_use]
fn sorted_statistic(values: &[Value], statistic: impl FnOnce(&[f64]) -> f64) -> Value {
    if values.iter().any(Value::is_undefined) {
        return Value::Undefined;
    }

    let mut numbers: Vec<f64> = values.iter().map(Value::to_number).collect();

    if numbers.iter().any(|number| number.is_nan()) {
        return Value::Number(f64::NAN);
    }

    numbers.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    Value::Number(statistic(&numbers))
}

#[must_use]
fn median(numbers: &[f64]) -> f64 {
    match numbers.split_at_checked(numbers.len() / 2) {
        Some((lower, upper)) => match (lower.last(), upper.first()) {
            (Some(low), Some(high)) if lower.len() == upper.len() => (low + high) / 2.0,
            (_, Some(center)) => *center,
            (_, None) => f64::NAN,
        },
        None => f64::NAN,
    }
}

#[must_use]
fn population_variance(values: &[Value]) -> Option<f64> {
    if values.iter().any(Value::is_undefined) {
        return None;
    }

    let numbers: Vec<f64> = values.iter().map(Value::to_number).collect();
    let count = number::from_usize(numbers.len());
    let mean = numbers.iter().fold(0.0, |total, value| total + value) / count;
    let squares = numbers.iter().fold(0.0, |total, value| {
        let deviation = value - mean;
        total + deviation * deviation
    });

    Some(squares / count)
}

fn quantile_rank(rank: Option<&Value>) -> Result<f64> {
    match rank {
        Some(Value::Number(rank)) if (0.0..=1.0).contains(rank) => Ok(*rank),
        _ => Err(Error::type_error("quantile expects a number from 0 to 1")),
    }
}

#[must_use]
fn interpolate(numbers: &[f64], rank: f64) -> f64 {
    let position = number::from_usize(numbers.len().saturating_sub(1)) * rank;
    let lower = number::to_usize(position);
    let fraction = position - number::from_usize(lower);

    match numbers.get(lower..) {
        Some([low, high, ..]) if !number::equal(fraction, 0.0) && !number::equal(*low, *high) => {
            (1.0 - fraction) * low + fraction * high
        }
        Some([low, ..]) => *low,
        Some([]) | None => f64::NAN,
    }
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
            "product" => return self.product_items(items, selector.as_ref()),
            "average" => return self.average_items(items, selector.as_ref()),
            _ => {}
        }

        let values = self.select_items(items, selector.as_ref())?;

        Ok(match property {
            "distinct" => Value::List(self.distinct_values(values)),
            "mode" => self.mode_value(values),
            "median" => sorted_statistic(&values, median),
            "variance" => population_variance(&values).map_or(Value::Undefined, Value::Number),
            "stdev" => population_variance(&values)
                .map_or(Value::Undefined, |variance| Value::Number(variance.sqrt())),
            _ => builtins::math(property, &values)?,
        })
    }

    pub(super) fn quantile_items(
        &mut self,
        items: Vec<Value>,
        arguments: &[Value],
    ) -> Result<Value> {
        let rank = quantile_rank(arguments.first())?;
        let selector = optional_selector(arguments.get(1).cloned(), "quantile")?;
        let values = self.select_items(items, selector.as_ref())?;
        let quantile = sorted_statistic(&values, |numbers| interpolate(numbers, rank));
        Ok(quantile)
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

            if is_unordered(&key) {
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
        self.fold_numbers(items, selector, 0.0, |total, value| total + value)
    }

    fn product_items(
        &mut self,
        items: Vec<Value>,
        selector: Option<&Arc<Function>>,
    ) -> Result<Value> {
        self.fold_numbers(items, selector, 1.0, |total, value| total * value)
    }

    fn fold_numbers(
        &mut self,
        items: Vec<Value>,
        selector: Option<&Arc<Function>>,
        start: f64,
        combine: impl Fn(f64, f64) -> f64,
    ) -> Result<Value> {
        let mut total = start;
        let mut undefined = false;

        for item in items {
            let value = match selector {
                Some(function) => self.invoke(function, &[item])?,
                None => item,
            };
            undefined |= value.is_undefined();
            total = combine(total, value.to_number());
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
