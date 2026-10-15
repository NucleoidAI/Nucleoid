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
        "fract" => {
            let floor = first.floor();
            first - floor
        }
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
            numeric_clamp(first, lower, upper)
        }
        "saturate" => numeric_clamp(first, 0.0, 1.0),
        "smoothstep" => {
            let upper = arguments.get(1).map(Value::to_number).unwrap_or(f64::NAN);
            let value = arguments.get(2).map(Value::to_number).unwrap_or(f64::NAN);
            // `first` is the lower bound, not clamp's value-first argument.
            // Keep normalization and the cubic as separate ordered operations.
            let numerator = value - first;
            let denominator = upper - first;
            let normalize = numerator / denominator;
            let t = numeric_clamp(normalize, 0.0, 1.0);
            let square = t * t;
            let scaled = 2.0 * t;
            let falloff = 3.0 - scaled;
            square * falloff
        }
        "smootherstep" => {
            let upper = arguments.get(1).map(Value::to_number).unwrap_or(f64::NAN);
            let value = arguments.get(2).map(Value::to_number).unwrap_or(f64::NAN);
            // Match the accepted normalization and quintic expression's order,
            // including its rounding; do not clamp the polynomial's result.
            let numerator = value - first;
            let denominator = upper - first;
            let normalize = numerator / denominator;
            let t = numeric_clamp(normalize, 0.0, 1.0);
            let square = t * t;
            let cube = square * t;
            let scaled = 6.0 * t;
            let offset = scaled - 15.0;
            let weighted = t * offset;
            let polynomial = weighted + 10.0;
            cube * polynomial
        }
        "lerp" => {
            let end = arguments.get(1).map(Value::to_number).unwrap_or(f64::NAN);
            let weight = arguments.get(2).map(Value::to_number).unwrap_or(f64::NAN);
            // Preserve ordinary subtraction, multiplication, then addition.
            // No endpoint shortcut, clamping, reordering or fused operation.
            let delta = end - first;
            let scaled = delta * weight;
            first + scaled
        }
        "inverseLerp" => {
            let end = arguments.get(1).map(Value::to_number).unwrap_or(f64::NAN);
            let value = arguments.get(2).map(Value::to_number).unwrap_or(f64::NAN);
            let numerator = value - first;
            let denominator = end - first;
            numerator / denominator
        }
        "random" => return Err(Error::type_error("Math.random is not supported")),
        "PI" => std::f64::consts::PI,
        "E" => std::f64::consts::E,
        _ => return Err(Error::type_error(format!("Math.{name} is not a function"))),
    };

    Ok(Value::Number(result))
}

/// Ordered min(max(...), ...), not f64::clamp: retain the accepted folds for
/// NaN, reversed bounds and signed zeros, without allocating runtime values.
fn numeric_clamp(value: f64, lower: f64, upper: f64) -> f64 {
    let bounded = numeric_extremum([value, lower], true);
    numeric_extremum([bounded, upper], false)
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
    use crate::lang::ast::operator::binary;
    use crate::lang::ast::{BinaryOp, Expr, Function, FunctionBody};
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

    fn ordered_interpolation(start: &Value, end: &Value, weight: &Value) -> Value {
        if [start, end, weight].into_iter().any(Value::is_undefined) {
            return Value::Undefined;
        }
        let [start, end, weight] =
            [start, end, weight].map(|value| Value::Number(value.to_number()));
        let delta = binary(BinaryOp::Subtract, &end, &start).unwrap();
        let scaled = binary(BinaryOp::Multiply, &delta, &weight).unwrap();
        binary(BinaryOp::Add, &start, &scaled).unwrap()
    }

    fn ordered_smoothing(lower: &Value, upper: &Value, value: &Value) -> Value {
        if [lower, upper, value].into_iter().any(Value::is_undefined) {
            return Value::Undefined;
        }
        let [lower, upper, value] =
            [lower, upper, value].map(|value| Value::Number(value.to_number()));
        let numerator = binary(BinaryOp::Subtract, &value, &lower).unwrap();
        let denominator = binary(BinaryOp::Subtract, &upper, &lower).unwrap();
        let normalize = binary(BinaryOp::Divide, &numerator, &denominator).unwrap();
        let t = ordered_composition(&normalize, &Value::number(0), &Value::number(1));
        let square = binary(BinaryOp::Multiply, &t, &t).unwrap();
        let scaled = binary(BinaryOp::Multiply, &Value::number(2), &t).unwrap();
        let falloff = binary(BinaryOp::Subtract, &Value::number(3), &scaled).unwrap();
        binary(BinaryOp::Multiply, &square, &falloff).unwrap()
    }

    fn assert_smoothing_same(actual: &Value, expected: &Value) {
        if matches!(expected, Value::Number(number) if number.is_nan()) {
            assert!(matches!(actual, Value::Number(number) if number.is_nan()));
        } else {
            assert_same(actual, expected);
        }
    }

    fn ordered_quintic_smoothing(lower: &Value, upper: &Value, value: &Value) -> Value {
        if [lower, upper, value].into_iter().any(Value::is_undefined) {
            return Value::Undefined;
        }
        let [lower, upper, value] =
            [lower, upper, value].map(|value| Value::Number(value.to_number()));
        let numerator = binary(BinaryOp::Subtract, &value, &lower).unwrap();
        let denominator = binary(BinaryOp::Subtract, &upper, &lower).unwrap();
        let normalize = binary(BinaryOp::Divide, &numerator, &denominator).unwrap();
        let t = ordered_composition(&normalize, &Value::number(0), &Value::number(1));
        let square = binary(BinaryOp::Multiply, &t, &t).unwrap();
        let cube = binary(BinaryOp::Multiply, &square, &t).unwrap();
        let scaled = binary(BinaryOp::Multiply, &Value::number(6), &t).unwrap();
        let offset = binary(BinaryOp::Subtract, &scaled, &Value::number(15)).unwrap();
        let weighted = binary(BinaryOp::Multiply, &t, &offset).unwrap();
        let polynomial = binary(BinaryOp::Add, &weighted, &Value::number(10)).unwrap();
        binary(BinaryOp::Multiply, &cube, &polynomial).unwrap()
    }

    fn ordered_inverse_interpolation(start: &Value, end: &Value, value: &Value) -> Value {
        let numerator = binary(BinaryOp::Subtract, value, start).unwrap();
        let denominator = binary(BinaryOp::Subtract, end, start).unwrap();
        binary(BinaryOp::Divide, &numerator, &denominator).unwrap()
    }

    #[test]
    fn inverse_lerp_matches_independent_binary_controls_for_every_value_kind_and_position() {
        let values = values();
        for start in &values {
            for end in &values {
                for value in &values {
                    assert_smoothing_same(
                        &math("inverseLerp", &[start.clone(), end.clone(), value.clone()]).unwrap(),
                        &ordered_inverse_interpolation(start, end, value),
                    );
                }
            }
        }
    }

    #[test]
    fn inverse_lerp_edge_grid_keeps_nonfinite_overflow_subnormal_and_zero_composition_bits() {
        let one = 1.0_f64.to_bits();
        let half = 0.5_f64.to_bits();
        let numbers = [
            -f64::MAX,
            -1e16,
            -1.0,
            -f64::MIN_POSITIVE,
            -f64::from_bits(1),
            -0.0,
            0.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            f64::EPSILON,
            0.1,
            0.125,
            0.25,
            f64::from_bits(half - 1),
            0.5,
            f64::from_bits(half + 1),
            0.75,
            f64::from_bits(one - 1),
            1.0,
            f64::from_bits(one + 1),
            1e16,
            f64::MAX,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NAN,
            f64::from_bits(0x7ff8_0000_0000_0042),
            f64::from_bits(0xfff8_0000_0000_0042),
        ];
        for start in numbers {
            for end in numbers {
                for value in numbers {
                    let arguments = [start, end, value].map(Value::Number);
                    assert_smoothing_same(
                        &math("inverseLerp", &arguments).unwrap(),
                        &ordered_inverse_interpolation(&arguments[0], &arguments[1], &arguments[2]),
                    );
                }
            }
        }
        for (arguments, expected) in [
            ([0.0, 8.0, 2.0], 0.25),
            ([0.0, 8.0, -8.0], -1.0),
            ([0.0, 8.0, 16.0], 2.0),
            ([10.0, 2.0, 8.0], 0.25),
            ([2.0, 2.0, 2.0], f64::NAN),
            ([2.0, 2.0, 3.0], f64::INFINITY),
            ([2.0, 2.0, 1.0], f64::NEG_INFINITY),
            ([-f64::MAX, f64::MAX, 0.0], 0.0),
            ([-f64::MAX, f64::MAX, f64::MAX], f64::NAN),
            ([1e16, 1e16 + 2.0, 1e16], 0.0),
        ] {
            assert_smoothing_same(
                &math("inverseLerp", &arguments.map(Value::Number)).unwrap(),
                &Value::Number(expected),
            );
        }
    }

    #[test]
    fn inverse_lerp_full_bit_triples_match_ordered_subtraction_then_division() {
        let mut bits = 0x38fa_1c70_8e25_d49b_u64;
        for _ in 0..4_096 {
            let arguments = std::array::from_fn::<_, 3, _>(|_| {
                bits = bits
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                Value::Number(f64::from_bits(bits))
            });
            assert_smoothing_same(
                &math("inverseLerp", &arguments).unwrap(),
                &ordered_inverse_interpolation(&arguments[0], &arguments[1], &arguments[2]),
            );
        }
    }

    #[test]
    fn inverse_lerp_missing_inputs_and_unconverted_extras_keep_the_supplied_undefined_gate() {
        let arguments = [Value::number(0), Value::number(8), Value::number(2)];
        for length in 0..3 {
            assert_smoothing_same(
                &math("inverseLerp", &arguments[..length]).unwrap(),
                &Value::Number(f64::NAN),
            );
        }
        for extra in values() {
            let mut supplied = arguments.to_vec();
            let expected = if extra.is_undefined() {
                Value::Undefined
            } else {
                Value::number(0.25)
            };
            supplied.push(extra);
            assert_smoothing_same(&math("inverseLerp", &supplied).unwrap(), &expected);
            supplied.push(Value::number(99));
            assert_smoothing_same(&math("inverseLerp", &supplied).unwrap(), &expected);
        }
        for position in 0..6 {
            let mut supplied = vec![Value::number(0.5); 6];
            supplied[position] = Value::Undefined;
            assert_same(&math("inverseLerp", &supplied).unwrap(), &Value::Undefined);
        }
        assert_same(
            &math("inverseLerp", &[Value::Undefined]).unwrap(),
            &Value::Undefined,
        );
        assert!(math_constant("inverseLerp").is_none());
    }

    #[test]
    fn fract_matches_independent_floor_subtraction_for_every_value_kind() {
        for value in values() {
            let floor = math("floor", std::slice::from_ref(&value)).unwrap();
            let expected = binary(BinaryOp::Subtract, &value, &floor).unwrap();
            assert_smoothing_same(
                &math("fract", std::slice::from_ref(&value)).unwrap(),
                &expected,
            );
        }
    }

    #[test]
    fn fract_preserves_negative_rounding_subnormal_nonfinite_and_zero_bits() {
        let one = 1.0_f64.to_bits();
        let half = 0.5_f64.to_bits();
        for number in [
            -f64::MAX,
            -9_007_199_254_740_991.0,
            -4_503_599_627_370_495.5,
            -3.0,
            -2.75,
            -1.25,
            -1.0,
            -f64::from_bits(one - 1),
            -0.5,
            -1e-20,
            -f64::MIN_POSITIVE,
            -f64::from_bits(1),
            -0.0,
            0.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            f64::EPSILON,
            0.1,
            f64::from_bits(half - 1),
            0.5,
            f64::from_bits(half + 1),
            f64::from_bits(one - 1),
            1.0,
            f64::from_bits(one + 1),
            1.25,
            4_503_599_627_370_495.5,
            9_007_199_254_740_991.0,
            f64::MAX,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NAN,
            f64::from_bits(0x7ff8_0000_0000_0042),
            f64::from_bits(0xfff8_0000_0000_0042),
        ] {
            let value = Value::Number(number);
            let floor = math("floor", std::slice::from_ref(&value)).unwrap();
            let expected = binary(BinaryOp::Subtract, &value, &floor).unwrap();
            assert_smoothing_same(&math("fract", &[value]).unwrap(), &expected);
        }
        for (input, expected) in [(-1.25, 0.75), (-1e-20, 1.0), (2.75, 0.75)] {
            assert_same(
                &math("fract", &[Value::Number(input)]).unwrap(),
                &Value::Number(expected),
            );
        }
    }

    #[test]
    fn fract_full_bit_samples_match_the_accepted_arithmetic_control() {
        let mut bits = 0x36fa_1c70_8e25_d49b_u64;
        for _ in 0..4_096 {
            bits = bits
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let value = Value::Number(f64::from_bits(bits));
            let floor = math("floor", std::slice::from_ref(&value)).unwrap();
            let expected = binary(BinaryOp::Subtract, &value, &floor).unwrap();
            assert_smoothing_same(&math("fract", &[value]).unwrap(), &expected);
        }
    }

    #[test]
    fn fract_omission_and_all_extra_values_keep_the_existing_undefined_gate() {
        assert_smoothing_same(&math("fract", &[]).unwrap(), &Value::Number(f64::NAN));
        let values = values();
        for value in &values {
            let floor = math("floor", std::slice::from_ref(value)).unwrap();
            let fraction = binary(BinaryOp::Subtract, value, &floor).unwrap();
            for extra in &values {
                let expected = if value.is_undefined() || extra.is_undefined() {
                    &Value::Undefined
                } else {
                    &fraction
                };
                let supplied = [value.clone(), extra.clone(), Value::number(99)];
                assert_smoothing_same(&math("fract", &supplied[..2]).unwrap(), expected);
                assert_smoothing_same(&math("fract", &supplied).unwrap(), expected);
            }
        }
        for position in 0..5 {
            let mut supplied = vec![Value::number(0.375); 5];
            supplied[position] = Value::Undefined;
            assert_same(&math("fract", &supplied).unwrap(), &Value::Undefined);
        }
        assert!(math_constant("fract").is_none());
    }

    #[test]
    fn smootherstep_matches_independent_ordered_binary_controls_for_every_value_kind() {
        let values = values();
        for lower in &values {
            for upper in &values {
                for value in &values {
                    assert_smoothing_same(
                        &math(
                            "smootherstep",
                            &[lower.clone(), upper.clone(), value.clone()],
                        )
                        .unwrap(),
                        &ordered_quintic_smoothing(lower, upper, value),
                    );
                }
            }
        }
    }

    #[test]
    fn smootherstep_keeps_nonfinite_overflow_subnormal_boundary_and_zero_composition_bits() {
        let one = 1.0_f64.to_bits();
        let half = 0.5_f64.to_bits();
        let numbers = [
            -f64::MAX,
            -1e16,
            -1.0,
            -f64::MIN_POSITIVE,
            -f64::from_bits(1),
            -0.0,
            0.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            f64::EPSILON,
            0.1,
            0.125,
            0.25,
            f64::from_bits(half - 1),
            0.5,
            f64::from_bits(half + 1),
            0.75,
            0.9,
            f64::from_bits(one - 3),
            f64::from_bits(one - 1),
            1.0,
            f64::from_bits(one + 1),
            1e16,
            f64::MAX,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NAN,
            f64::from_bits(0x7ff8_0000_0000_0042),
            f64::from_bits(0xfff8_0000_0000_0042),
        ];
        for lower in numbers {
            for upper in numbers {
                for value in numbers {
                    let arguments = [lower, upper, value].map(Value::Number);
                    assert_smoothing_same(
                        &math("smootherstep", &arguments).unwrap(),
                        &ordered_quintic_smoothing(&arguments[0], &arguments[1], &arguments[2]),
                    );
                }
            }
        }
    }

    #[test]
    fn smootherstep_retains_quintic_rounding_overshoot_and_unvalidated_bound_order() {
        for (arguments, expected) in [
            ([0.0, 8.0, 1.0], 0.01605224609375),
            ([0.0, 8.0, 2.0], 0.103515625),
            ([0.0, 8.0, 3.0], 0.27520751953125),
            ([0.0, 8.0, 4.0], 0.5),
            ([0.0, 8.0, 6.0], 0.896484375),
            ([0.0, 8.0, -2.0], 0.0),
            ([0.0, 8.0, 10.0], 1.0),
            ([8.0, 0.0, 2.0], 0.896484375),
            ([2.0, 2.0, 1.0], 0.0),
            ([2.0, 2.0, 3.0], 1.0),
            ([2.0, 2.0, 2.0], f64::NAN),
            ([-f64::MAX, f64::MAX, 0.0], 0.0),
            ([-f64::MAX, f64::MAX, f64::MAX], f64::NAN),
            ([0.0, 1.0, 0.1], f64::from_bits(0x3f81_87e7_c06e_19bb)),
            (
                [0.0, 1.0, f64::from_bits(1.0_f64.to_bits() - 1)],
                f64::from_bits(0x3ff0_0000_0000_0006),
            ),
        ] {
            assert_smoothing_same(
                &math("smootherstep", &arguments.map(Value::Number)).unwrap(),
                &Value::Number(expected),
            );
        }
    }

    #[test]
    fn smootherstep_omissions_are_nan_and_unconverted_extras_keep_the_undefined_gate() {
        let arguments = [Value::number(0), Value::number(8), Value::number(2)];
        for length in 0..3 {
            assert_smoothing_same(
                &math("smootherstep", &arguments[..length]).unwrap(),
                &Value::Number(f64::NAN),
            );
        }
        for extra in values() {
            let mut supplied = arguments.to_vec();
            let expected = if extra.is_undefined() {
                Value::Undefined
            } else {
                Value::number(0.103515625)
            };
            supplied.push(extra);
            assert_smoothing_same(&math("smootherstep", &supplied).unwrap(), &expected);
            supplied.push(Value::number(99));
            assert_smoothing_same(&math("smootherstep", &supplied).unwrap(), &expected);
        }
        for position in 0..6 {
            let mut supplied = vec![Value::number(0.5); 6];
            supplied[position] = Value::Undefined;
            assert_same(&math("smootherstep", &supplied).unwrap(), &Value::Undefined);
        }
        assert_same(
            &math("smootherstep", &[Value::Undefined]).unwrap(),
            &Value::Undefined,
        );
        assert!(math_constant("smootherstep").is_none());
    }

    #[test]
    fn smoothstep_matches_ordered_normalization_extrema_and_cubic_for_every_value_kind() {
        let values = values();
        for lower in &values {
            for upper in &values {
                for value in &values {
                    assert_smoothing_same(
                        &math("smoothstep", &[lower.clone(), upper.clone(), value.clone()])
                            .unwrap(),
                        &ordered_smoothing(lower, upper, value),
                    );
                }
            }
        }
    }

    #[test]
    fn smoothstep_preserves_rounding_overflow_nonfinite_subnormal_and_zero_composition_bits() {
        let one = 1.0_f64.to_bits();
        let half = 0.5_f64.to_bits();
        let numbers = [
            -f64::MAX,
            -1e16,
            -1.0,
            -f64::MIN_POSITIVE,
            -f64::from_bits(1),
            -0.0,
            0.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            f64::EPSILON,
            f64::from_bits(half - 1),
            0.5,
            f64::from_bits(half + 1),
            f64::from_bits(one - 3),
            f64::from_bits(one - 1),
            1.0,
            f64::from_bits(one + 1),
            1e16,
            f64::MAX,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NAN,
            f64::from_bits(0x7ff8_0000_0000_0042),
            f64::from_bits(0xfff8_0000_0000_0042),
        ];
        for lower in numbers {
            for upper in numbers {
                for value in numbers {
                    let arguments = [lower, upper, value].map(Value::Number);
                    assert_smoothing_same(
                        &math("smoothstep", &arguments).unwrap(),
                        &ordered_smoothing(&arguments[0], &arguments[1], &arguments[2]),
                    );
                }
            }
        }
    }

    #[test]
    fn smoothstep_uses_lower_upper_value_order_without_equal_or_reversed_bound_validation() {
        for (arguments, expected) in [
            ([0.0, 8.0, 2.0], 0.15625),
            ([0.0, 8.0, 6.0], 0.84375),
            ([0.0, 8.0, -2.0], 0.0),
            ([0.0, 8.0, 10.0], 1.0),
            ([8.0, 0.0, 2.0], 0.84375),
            ([2.0, 2.0, 1.0], 0.0),
            ([2.0, 2.0, 3.0], 1.0),
            ([2.0, 2.0, 2.0], f64::NAN),
            ([-f64::MAX, f64::MAX, 0.0], 0.0),
            ([-f64::MAX, f64::MAX, f64::MAX], f64::NAN),
        ] {
            assert_smoothing_same(
                &math("smoothstep", &arguments.map(Value::Number)).unwrap(),
                &Value::Number(expected),
            );
        }
    }

    #[test]
    fn smoothstep_omissions_are_nan_and_every_supplied_extra_retains_the_undefined_gate() {
        let arguments = [Value::number(0), Value::number(8), Value::number(2)];
        for length in 0..3 {
            assert_smoothing_same(
                &math("smoothstep", &arguments[..length]).unwrap(),
                &Value::Number(f64::NAN),
            );
        }
        for extra in values() {
            let mut supplied = arguments.to_vec();
            let expected = if extra.is_undefined() {
                Value::Undefined
            } else {
                Value::number(0.15625)
            };
            supplied.push(extra);
            assert_smoothing_same(&math("smoothstep", &supplied).unwrap(), &expected);
            supplied.push(Value::number(99));
            assert_smoothing_same(&math("smoothstep", &supplied).unwrap(), &expected);
        }
        for position in 0..6 {
            let mut supplied = vec![Value::number(0.5); 6];
            supplied[position] = Value::Undefined;
            assert_same(&math("smoothstep", &supplied).unwrap(), &Value::Undefined);
        }
        assert_same(
            &math("smoothstep", &[Value::Undefined]).unwrap(),
            &Value::Undefined,
        );
        assert!(math_constant("smoothstep").is_none());
    }

    #[test]
    fn lerp_matches_ordered_binary_arithmetic_for_every_value_kind_without_panicking() {
        let values = values();
        for start in &values {
            for end in &values {
                for weight in &values {
                    assert_same(
                        &math("lerp", &[start.clone(), end.clone(), weight.clone()]).unwrap(),
                        &ordered_interpolation(start, end, weight),
                    );
                }
            }
        }
    }

    #[test]
    fn lerp_preserves_ordered_rounding_overflow_cancellation_nonfinite_and_zero_bits() {
        let numbers = [
            -f64::MAX,
            -1e16,
            -1.0,
            -f64::MIN_POSITIVE,
            -f64::from_bits(1),
            -0.0,
            0.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            f64::EPSILON,
            1.0,
            1.000_000_000_000_000_2,
            1e16,
            f64::MAX,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NAN,
        ];
        for start in numbers {
            for end in numbers {
                for weight in numbers {
                    let arguments = [start, end, weight].map(Value::Number);
                    assert_same(
                        &math("lerp", &arguments).unwrap(),
                        &ordered_interpolation(&arguments[0], &arguments[1], &arguments[2]),
                    );
                }
            }
        }
    }

    #[test]
    fn lerp_keeps_finite_fractional_extrapolated_equal_and_reversed_results() {
        for (arguments, expected) in [
            ([0.0, 10.0, 0.25], 2.5),
            ([2.0, 10.0, 0.25], 4.0),
            ([2.0, 18.0, 0.5], 10.0),
            ([2.0, 18.0, -0.5], -6.0),
            ([2.0, 18.0, 1.5], 26.0),
            ([4.0, 4.0, 2.0], 4.0),
            ([10.0, 2.0, 0.25], 8.0),
        ] {
            assert_same(
                &math("lerp", &arguments.map(Value::Number)).unwrap(),
                &Value::Number(expected),
            );
        }
    }

    #[test]
    fn lerp_omissions_are_nan_and_all_supplied_extras_remain_in_the_undefined_gate() {
        let arguments = [Value::number(2), Value::number(10), Value::number(0.5)];
        for length in 0..3 {
            assert_same(
                &math("lerp", &arguments[..length]).unwrap(),
                &Value::Number(f64::NAN),
            );
        }
        for extra in values() {
            let mut supplied = arguments.to_vec();
            let expected = if extra.is_undefined() {
                Value::Undefined
            } else {
                Value::number(6)
            };
            supplied.push(extra);
            assert_same(&math("lerp", &supplied).unwrap(), &expected);
            supplied.push(Value::number(99));
            assert_same(&math("lerp", &supplied).unwrap(), &expected);
        }
        for position in 0..5 {
            let mut supplied = vec![Value::number(2); 5];
            supplied[position] = Value::Undefined;
            assert_same(&math("lerp", &supplied).unwrap(), &Value::Undefined);
        }
        assert!(math_constant("lerp").is_none());
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
    fn saturate_matches_unit_clamp_and_ordered_folds_for_every_value_kind() {
        let lower = Value::number(0);
        let upper = Value::number(1);
        for value in values() {
            let actual = math("saturate", std::slice::from_ref(&value)).unwrap();
            let ordered = ordered_composition(&value, &lower, &upper);
            let clamped = math("clamp", &[value, lower.clone(), upper.clone()]).unwrap();
            assert_same(&actual, &ordered);
            assert_same(&actual, &clamped);
        }
    }

    #[test]
    fn saturate_preserves_boundary_subnormal_nonfinite_and_zero_bits_from_accepted_folds() {
        let one = 1.0_f64.to_bits();
        for number in [
            -f64::MAX,
            -1.0,
            -f64::MIN_POSITIVE,
            -f64::from_bits(1),
            -0.0,
            0.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            0.375,
            f64::from_bits(one - 1),
            1.0,
            f64::from_bits(one + 1),
            f64::MAX,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NAN,
            f64::from_bits(0x7ff8_0000_0000_0042),
            f64::from_bits(0xfff8_0000_0000_0042),
        ] {
            let value = Value::Number(number);
            let expected = ordered_composition(&value, &Value::number(0), &Value::number(1));
            assert_same(
                &math("saturate", std::slice::from_ref(&value)).unwrap(),
                &expected,
            );
            assert_same(
                &math("clamp", &[value, Value::number(0), Value::number(1)]).unwrap(),
                &expected,
            );
        }
    }

    #[test]
    fn saturate_omission_is_nan_and_every_supplied_extra_retains_the_undefined_gate() {
        assert_same(
            &math("saturate", &[]).unwrap(),
            &math("clamp", &[]).unwrap(),
        );
        assert_same(&math("saturate", &[]).unwrap(), &Value::Number(f64::NAN));
        let values = values();
        for value in &values {
            for extra in &values {
                let expected = if value.is_undefined() || extra.is_undefined() {
                    Value::Undefined
                } else {
                    ordered_composition(value, &Value::number(0), &Value::number(1))
                };
                let supplied = [value.clone(), extra.clone(), Value::number(99)];
                assert_same(&math("saturate", &supplied[..2]).unwrap(), &expected);
                assert_same(&math("saturate", &supplied).unwrap(), &expected);
                assert_same(
                    &math(
                        "clamp",
                        &[
                            value.clone(),
                            Value::number(0),
                            Value::number(1),
                            extra.clone(),
                            Value::number(99),
                        ],
                    )
                    .unwrap(),
                    &expected,
                );
            }
        }
        for position in 0..5 {
            let mut supplied = vec![Value::number(0.375); 5];
            supplied[position] = Value::Undefined;
            assert_same(&math("saturate", &supplied).unwrap(), &Value::Undefined);
        }
        assert!(math_constant("saturate").is_none());
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
