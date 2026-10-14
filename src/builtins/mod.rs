pub mod date;

use crate::error::{Error, Result};
use crate::value::{Value, format_number};

/// A name that resolves to a built-in rather than to state.
///
/// A closed enum keeps name resolution and call dispatch consistent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Global {
    Math,
    Date,
    Number,
    String,
    Boolean,
    Object,
    Class,
    List,
    Function,
    Json,
    Regex,
}

impl Global {
    /// The built-in a bare name refers to, if any.
    pub fn from_name(name: &str) -> Option<Global> {
        Some(match name {
            "Math" => Global::Math,
            "Date" => Global::Date,
            "Number" => Global::Number,
            "String" => Global::String,
            "Boolean" => Global::Boolean,
            "Object" => Global::Object,
            "Class" => Global::Class,
            "List" => Global::List,
            "Function" => Global::Function,
            "JSON" => Global::Json,
            "RegExp" => Global::Regex,
            _ => return None,
        })
    }

    /// The built-in a name in call position refers to. `Array` is callable as
    /// another spelling of `List`, but is not a name that resolves on its own,
    /// so it is not a [`Global::from_name`].
    pub fn from_callee(name: &str) -> Option<Global> {
        match name {
            "Array" => Some(Global::List),
            other => Global::from_name(other),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Global::Math => "Math",
            Global::Date => "Date",
            Global::Number => "Number",
            Global::String => "String",
            Global::Boolean => "Boolean",
            Global::Object => "Object",
            Global::Class => "Class",
            Global::List => "List",
            Global::Function => "Function",
            Global::Json => "JSON",
            Global::Regex => "RegExp",
        }
    }
}

pub fn is_global(name: &str) -> bool {
    Global::from_name(name).is_some()
}

/// `Math.<name>(...)`
pub fn math(name: &str, arguments: &[Value]) -> Result<Value> {
    if arguments.iter().any(Value::is_undefined) {
        return Ok(Value::Undefined);
    }

    let first = arguments.first().map(Value::to_number).unwrap_or(f64::NAN);

    let result = match name {
        "abs" => first.abs(),
        "ceil" => first.ceil(),
        "floor" => first.floor(),
        "round" => {
            // Compare the fraction directly: host `round` can lose precision
            // just below 0.5. Truncation also preserves negative zero.
            let integer = first.trunc();
            let fraction = first - integer;
            if fraction >= 0.5 {
                integer + 1.0
            } else if fraction < -0.5 {
                integer - 1.0
            } else {
                integer
            }
        }
        "trunc" => first.trunc(),
        "sqrt" => first.sqrt(),
        "cbrt" => first.cbrt(),
        "sign" => {
            if first > 0.0 {
                1.0
            } else if first < 0.0 {
                -1.0
            } else {
                first
            }
        }
        "exp" => first.exp(),
        "log" => first.ln(),
        "log2" => first.log2(),
        "log10" => first.log10(),
        "sin" => first.sin(),
        "cos" => first.cos(),
        "tan" => first.tan(),
        "atan" => first.atan(),
        "asin" => first.asin(),
        "acos" => first.acos(),
        "pow" => {
            let exponent = arguments.get(1).map(Value::to_number).unwrap_or(f64::NAN);
            first.powf(exponent)
        }
        "atan2" => {
            let second = arguments.get(1).map(Value::to_number).unwrap_or(f64::NAN);
            first.atan2(second)
        }
        "max" | "min" => numeric_extremum(arguments.iter().map(Value::to_number), name == "max"),
        "clamp" => {
            let lower = arguments.get(1).map(Value::to_number).unwrap_or(f64::NAN);
            let upper = arguments.get(2).map(Value::to_number).unwrap_or(f64::NAN);
            // Ordered min(max(...), ...), not f64::clamp: NaN and reversed
            // bounds have defined results, and signed zeros use the same folds.
            let bounded = numeric_extremum([first, lower], true);
            numeric_extremum([bounded, upper], false)
        }
        "random" => return Err(Error::type_error("Math.random is not supported")),
        "PI" => std::f64::consts::PI,
        "E" => std::f64::consts::E,
        _ => return Err(Error::type_error(format!("Math.{name} is not a function"))),
    };

    Ok(Value::Number(result))
}

/// The existing Math min/max fold, shared without changing its NaN or zero rules.
fn numeric_extremum(numbers: impl IntoIterator<Item = f64>, maximum: bool) -> f64 {
    let initial = if maximum {
        f64::NEG_INFINITY
    } else {
        f64::INFINITY
    };

    numbers.into_iter().fold(initial, |extreme, number| {
        if extreme.is_nan() || number.is_nan() {
            f64::NAN
        } else if maximum {
            extreme.max(number)
        } else {
            extreme.min(number)
        }
    })
}

/// `Math.<name>` read as a value rather than called.
pub fn math_constant(name: &str) -> Option<Value> {
    Some(match name {
        "PI" => Value::Number(std::f64::consts::PI),
        "E" => Value::Number(std::f64::consts::E),
        "LN2" => Value::Number(std::f64::consts::LN_2),
        "LN10" => Value::Number(std::f64::consts::LN_10),
        "SQRT2" => Value::Number(std::f64::consts::SQRT_2),
        _ => return None,
    })
}

/// `Number.<name>` read as a value.
pub fn number_constant(name: &str) -> Option<Value> {
    Some(match name {
        "MAX_INTEGER" | "MAX_SAFE_INTEGER" => Value::Number(9_007_199_254_740_991.0),
        "MIN_INTEGER" | "MIN_SAFE_INTEGER" => Value::Number(-9_007_199_254_740_991.0),
        "MAX_VALUE" => Value::Number(f64::MAX),
        "MIN_VALUE" => Value::Number(f64::MIN_POSITIVE),
        "POSITIVE_INFINITY" => Value::Number(f64::INFINITY),
        "NEGATIVE_INFINITY" => Value::Number(f64::NEG_INFINITY),
        "EPSILON" => Value::Number(f64::EPSILON),
        "NaN" => Value::Number(f64::NAN),
        _ => return None,
    })
}

/// `Number.<name>(...)`
pub fn number(name: &str, arguments: &[Value]) -> Result<Value> {
    let first = arguments.first().cloned().unwrap_or(Value::Undefined);

    Ok(match name {
        "parseFloat" | "parseInt" => {
            let text = first.to_string();
            let trimmed = text.trim();
            let parsed = if name == "parseInt" {
                trimmed
                    .split(['.', 'e', 'E'])
                    .next()
                    .unwrap_or("")
                    .parse::<f64>()
            } else {
                trimmed.parse::<f64>()
            };
            Value::Number(parsed.unwrap_or(f64::NAN))
        }
        "isInteger" => Value::Bool(matches!(first, Value::Number(number) if number.fract() == 0.0)),
        "isNaN" => Value::Bool(first.to_number().is_nan()),
        "isFinite" => Value::Bool(first.to_number().is_finite()),
        _ => {
            return Err(Error::type_error(format!(
                "Number.{name} is not a function"
            )));
        }
    })
}

/// `String.<name>(...)`
pub fn string(name: &str, arguments: &[Value]) -> Result<Value> {
    match name {
        "fromCharCode" => {
            let mut output = String::new();

            for argument in arguments {
                let code = argument.to_number();
                if let Some(character) = u32::try_from(code as i64).ok().and_then(char::from_u32) {
                    output.push(character);
                }
            }

            Ok(Value::String(output))
        }
        _ => Err(Error::type_error(format!(
            "String.{name} is not a function"
        ))),
    }
}

/// String instance methods.
pub fn string_method(receiver: &str, name: &str, arguments: &[Value]) -> Result<Value> {
    let first = arguments.first().unwrap_or(&Value::Undefined);

    Ok(match name {
        "lower" | "toLowerCase" => Value::String(receiver.to_lowercase()),
        "upper" | "toUpperCase" => Value::String(receiver.to_uppercase()),
        "trim" | "strip" => Value::String(receiver.trim().to_string()),
        "charAt" => {
            let index = first.to_number() as usize;
            Value::String(
                receiver
                    .chars()
                    .nth(index)
                    .map(|c| c.to_string())
                    .unwrap_or_default(),
            )
        }
        "charCodeAt" => {
            let index = first.to_number() as usize;
            match receiver.chars().nth(index) {
                Some(character) => Value::Number(character as u32 as f64),
                None => Value::Number(f64::NAN),
            }
        }
        "indexOf" | "find" => {
            let needle = first.to_string();
            match receiver.find(&needle) {
                Some(index) => Value::Number(receiver[..index].chars().count() as f64),
                None => Value::Number(-1.0),
            }
        }
        "includes" | "contains" => Value::Bool(receiver.contains(&first.to_string())),
        "startsWith" => Value::Bool(receiver.starts_with(&first.to_string())),
        "endsWith" => Value::Bool(receiver.ends_with(&first.to_string())),
        "split" => {
            let separator = first.to_string();
            let parts: Vec<Value> = if separator.is_empty() {
                receiver
                    .chars()
                    .map(|character| Value::String(character.to_string()))
                    .collect()
            } else {
                receiver
                    .split(separator.as_str())
                    .map(|part| Value::String(part.to_string()))
                    .collect()
            };
            Value::List(parts)
        }
        "replace" => {
            let from = first.to_string();
            let to = arguments.get(1).unwrap_or(&Value::Undefined).to_string();
            Value::String(receiver.replacen(from.as_str(), to.as_str(), 1))
        }
        "repeat" => Value::String(string_repeat(
            receiver,
            first.to_number().max(0.0) as usize,
        )?),
        "substring" | "slice" | "substr" => {
            let end = arguments
                .get(1)
                .filter(|value| !value.is_nullish())
                .map(Value::to_number);
            Value::String(string_slice(receiver, first.to_number(), end))
        }
        "toString" => Value::String(receiver.to_string()),
        _ => {
            return Err(Error::type_error(format!(
                "{receiver}.{name} is not a function"
            )));
        }
    })
}

fn string_repeat(receiver: &str, count: usize) -> Result<String> {
    if receiver.is_empty() || count == 0 {
        return Ok(String::new());
    }

    let capacity = receiver
        .len()
        .checked_mul(count)
        .ok_or_else(|| Error::type_error("Repeated string is too large"))?;
    let mut output = String::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_| Error::type_error("Repeated string is too large"))?;

    for _ in 0..count {
        output.push_str(receiver);
    }

    Ok(output)
}

/// Slices by Unicode scalar indices without allocating a character buffer.
pub(crate) fn string_slice(receiver: &str, start: f64, end: Option<f64>) -> String {
    let length = receiver.chars().count();
    let start = slice_index(start, length);
    let end = end.map(|end| slice_index(end, length)).unwrap_or(length);

    receiver
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

/// Resolves a possibly negative index against a length, the way slicing does.
pub fn slice_index(index: f64, length: usize) -> usize {
    if index < 0.0 {
        let from_end = length as f64 + index;
        from_end.max(0.0) as usize
    } else {
        (index as usize).min(length)
    }
}

/// Renders a number the way the runtime prints it.
pub fn number_to_string(number: f64) -> String {
    format_number(number)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::lang::ast::{Expr, Function, FunctionBody};
    use crate::{ErrorKind, ObjectId};
    use regex::Regex;

    fn values() -> Vec<Value> {
        vec![
            Value::Undefined,
            Value::Null,
            Value::Bool(false),
            Value::Bool(true),
            Value::Number(-0.0),
            Value::Number(0.0),
            Value::Number(-3.5),
            Value::Number(7.25),
            Value::Number(f64::MIN_POSITIVE),
            Value::Number(f64::MAX),
            Value::Number(f64::NEG_INFINITY),
            Value::Number(f64::INFINITY),
            Value::Number(f64::NAN),
            Value::String(String::new()),
            Value::String("  4.25  ".to_string()),
            Value::String("invalid".to_string()),
            Value::List(vec![]),
            Value::List(vec![Value::String("4".to_string())]),
            Value::List(vec![Value::Undefined]),
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::Object(ObjectId::named("probe")),
            Value::Class("Probe".to_string()),
            Value::Date(12),
            Value::Regex(Arc::new(Regex::new("probe").unwrap())),
            Value::Function(Arc::new(Function {
                name: None,
                parameters: vec![],
                body: FunctionBody::Expression(Box::new(Expr::Null)),
            })),
        ]
    }

    fn ordered_composition(value: &Value, lower: &Value, upper: &Value) -> Value {
        let bounded = math("max", &[value.clone(), lower.clone()]).unwrap();
        math("min", &[bounded, upper.clone()]).unwrap()
    }

    fn assert_same(actual: &Value, expected: &Value) {
        assert!(actual.same_value(expected), "{actual:?} != {expected:?}");
        if let (Value::Number(actual), Value::Number(expected)) = (actual, expected) {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn clamp_matches_ordered_composition_for_every_value_kind_without_panicking() {
        let values = values();
        for value in &values {
            for lower in &values {
                for upper in &values {
                    let actual =
                        math("clamp", &[value.clone(), lower.clone(), upper.clone()]).unwrap();
                    let expected = ordered_composition(value, lower, upper);
                    assert_same(&actual, &expected);
                }
            }
        }
    }

    #[test]
    fn clamp_handles_finite_fractional_coerced_equal_and_reversed_inputs() {
        for (arguments, expected) in [
            (
                [Value::number(12), Value::number(0), Value::number(10)],
                10.0,
            ),
            (
                [Value::number(-3), Value::number(0), Value::number(10)],
                0.0,
            ),
            (
                [
                    Value::number(1.25),
                    Value::number(-0.5),
                    Value::number(0.75),
                ],
                0.75,
            ),
            ([Value::number(3), Value::number(4), Value::number(4)], 4.0),
            ([Value::number(5), Value::number(10), Value::number(0)], 0.0),
            (
                [Value::string(" 7 "), Value::number(0), Value::number(5)],
                5.0,
            ),
            (
                [Value::number(3), Value::string(" 4 "), Value::number(5)],
                4.0,
            ),
            (
                [Value::number(7), Value::number(0), Value::string("5")],
                5.0,
            ),
            ([Value::Null, Value::number(-1), Value::number(1)], 0.0),
            (
                [Value::Bool(true), Value::Bool(false), Value::number(2)],
                1.0,
            ),
            (
                [Value::List(vec![]), Value::number(1), Value::number(3)],
                1.0,
            ),
            (
                [
                    Value::List(vec![Value::string("4")]),
                    Value::number(0),
                    Value::number(3),
                ],
                3.0,
            ),
            ([Value::Date(12), Value::number(0), Value::Date(10)], 10.0),
        ] {
            assert_same(
                &math("clamp", &arguments).unwrap(),
                &Value::number(expected),
            );
        }
    }

    #[test]
    fn clamp_signed_zero_bits_match_both_ordered_extremum_folds() {
        let numbers = [-0.0, 0.0, -1.0, 1.0, f64::NEG_INFINITY, f64::INFINITY];
        for value in numbers {
            for lower in numbers {
                for upper in numbers {
                    let [value, lower, upper] = [value, lower, upper].map(Value::Number);
                    let expected = ordered_composition(&value, &lower, &upper);
                    assert_same(&math("clamp", &[value, lower, upper]).unwrap(), &expected);
                }
            }
        }
    }

    #[test]
    fn clamp_omissions_are_nan_and_extras_are_ignored_but_included_in_undefined_gate() {
        let arguments = [Value::number(3), Value::number(0), Value::number(5)];
        for length in 0..3 {
            let actual = math("clamp", &arguments[..length]).unwrap();
            assert_same(&actual, &Value::Number(f64::NAN));
        }
        for extra in values() {
            let mut supplied = arguments.to_vec();
            supplied.push(extra.clone());
            let expected = if extra.is_undefined() {
                Value::Undefined
            } else {
                Value::number(3)
            };
            assert_same(&math("clamp", &supplied).unwrap(), &expected);
            supplied.push(Value::number(99));
            assert_same(&math("clamp", &supplied).unwrap(), &expected);
        }
        for position in 0..5 {
            let mut supplied = vec![Value::number(3); 5];
            supplied[position] = Value::Undefined;
            assert_same(&math("clamp", &supplied).unwrap(), &Value::Undefined);
        }
    }

    #[test]
    fn existing_extrema_keep_variadic_seeds_coercion_nan_and_zero_rules() {
        let values = values();
        for (name, initial) in [("max", f64::NEG_INFINITY), ("min", f64::INFINITY)] {
            assert_same(&math(name, &[]).unwrap(), &Value::Number(initial));
            for left in &values {
                for right in &values {
                    let left_number = left.to_number();
                    let right_number = right.to_number();
                    let expected = if left.is_undefined() || right.is_undefined() {
                        Value::Undefined
                    } else if left_number.is_nan() || right_number.is_nan() {
                        Value::Number(f64::NAN)
                    } else if name == "max" {
                        Value::Number(left_number.max(right_number))
                    } else {
                        Value::Number(left_number.min(right_number))
                    };
                    assert_same(
                        &math(name, &[left.clone(), right.clone()]).unwrap(),
                        &expected,
                    );
                }
            }
            for position in 0..4 {
                let mut arguments = vec![Value::number(1); 4];
                arguments[position] = Value::Number(f64::NAN);
                assert_same(&math(name, &arguments).unwrap(), &Value::Number(f64::NAN));
            }
        }
    }

    #[test]
    fn existing_math_dispatch_rounding_constants_and_errors_are_unchanged() {
        for (name, input, expected) in [
            ("abs", -2.0, 2.0),
            ("ceil", 1.25, 2.0),
            ("floor", 1.25, 1.0),
            ("round", 0.499_999_999_999_999_94, 0.0),
            ("round", 0.5, 1.0),
            ("round", -0.5, -0.0),
            ("trunc", -0.25, -0.0),
            ("sqrt", 4.0, 2.0),
            ("cbrt", 8.0, 2.0),
            ("sign", -0.0, -0.0),
            ("exp", 0.0, 1.0),
            ("log", 1.0, 0.0),
            ("log2", 1.0, 0.0),
            ("log10", 1.0, 0.0),
            ("sin", 0.0, 0.0),
            ("cos", 0.0, 1.0),
            ("tan", 0.0, 0.0),
            ("atan", 0.0, 0.0),
            ("asin", 0.0, 0.0),
            ("acos", 1.0, 0.0),
        ] {
            assert_same(
                &math(name, &[Value::number(input)]).unwrap(),
                &Value::number(expected),
            );
            assert_same(
                &math(name, &[Value::number(input), Value::Number(f64::NAN)]).unwrap(),
                &Value::number(expected),
            );
            assert_same(&math(name, &[]).unwrap(), &Value::Number(f64::NAN));
        }
        assert_same(
            &math("pow", &[Value::number(2), Value::number(3)]).unwrap(),
            &Value::number(8),
        );
        assert_same(
            &math("atan2", &[Value::number(0), Value::number(1)]).unwrap(),
            &Value::number(0),
        );
        for name in ["pow", "atan2"] {
            assert_same(
                &math(name, &[Value::number(2)]).unwrap(),
                &Value::Number(f64::NAN),
            );
        }
        for (name, expected) in [("PI", std::f64::consts::PI), ("E", std::f64::consts::E)] {
            assert_same(&math(name, &[]).unwrap(), &Value::Number(expected));
            assert_same(&math_constant(name).unwrap(), &Value::Number(expected));
        }
        assert!(math_constant("clamp").is_none());
        assert!(math_constant("max").is_none());
        for (name, message) in [
            ("random", "Math.random is not supported"),
            ("unknown", "Math.unknown is not a function"),
        ] {
            let error = math(name, &[]).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Type);
            assert_eq!(error.message(), message);
            assert_same(&math(name, &[Value::Undefined]).unwrap(), &Value::Undefined);
        }
    }
}
