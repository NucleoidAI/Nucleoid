pub mod date;

use crate::error::{Error, Result};
use crate::value::{Value, format_number};

/// A name that resolves to a built-in rather than to state.
///
/// A closed set, so an enum: `ref` compares strings at each use, which is why
/// `Math` reaching one call site and not another is a bug it cannot see. Here
/// the compiler checks that every site handles the same set.
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
        "round" => (first + 0.5).floor(),
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
        "max" => arguments
            .iter()
            .map(Value::to_number)
            .fold(f64::NEG_INFINITY, f64::max),
        "min" => arguments
            .iter()
            .map(Value::to_number)
            .fold(f64::INFINITY, f64::min),
        "random" => return Err(Error::type_error("Math.random is not supported")),
        "PI" => std::f64::consts::PI,
        "E" => std::f64::consts::E,
        _ => return Err(Error::type_error(format!("Math.{name} is not a function"))),
    };

    Ok(Value::Number(result))
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
    let characters: Vec<char> = receiver.chars().collect();
    let first = arguments.first().cloned().unwrap_or(Value::Undefined);

    Ok(match name {
        "lower" | "toLowerCase" => Value::String(receiver.to_lowercase()),
        "upper" | "toUpperCase" => Value::String(receiver.to_uppercase()),
        "trim" | "strip" => Value::String(receiver.trim().to_string()),
        "charAt" => {
            let index = first.to_number() as usize;
            Value::String(
                characters
                    .get(index)
                    .map(|c| c.to_string())
                    .unwrap_or_default(),
            )
        }
        "charCodeAt" => {
            let index = first.to_number() as usize;
            match characters.get(index) {
                Some(character) => Value::Number(*character as u32 as f64),
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
            let to = arguments
                .get(1)
                .cloned()
                .unwrap_or(Value::Undefined)
                .to_string();
            Value::String(receiver.replacen(from.as_str(), to.as_str(), 1))
        }
        "repeat" => Value::String(receiver.repeat(first.to_number().max(0.0) as usize)),
        "substring" | "slice" | "substr" => {
            let start = slice_index(first.to_number(), characters.len());
            let end = match arguments.get(1) {
                Some(value) if !value.is_nullish() => {
                    slice_index(value.to_number(), characters.len())
                }
                _ => characters.len(),
            };
            Value::String(characters[start.min(end)..end].iter().collect())
        }
        "toString" => Value::String(receiver.to_string()),
        _ => {
            return Err(Error::type_error(format!(
                "{receiver}.{name} is not a function"
            )));
        }
    })
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
