pub mod date;
mod numeric;
mod text;

use crate::error::{Error, Result};
use crate::value::{Value, format_number};
pub use numeric::{math, math_constant};
pub(crate) use text::string_slice;
pub use text::{slice_index, string, string_method};

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
    #[must_use]
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
    #[must_use]
    pub fn from_callee(name: &str) -> Option<Global> {
        match name {
            "Array" => Some(Global::List),
            other => Global::from_name(other),
        }
    }

    #[must_use]
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

#[must_use]
pub fn is_global(name: &str) -> bool {
    Global::from_name(name).is_some()
}

/// `Number.<name>` read as a value.
#[must_use]
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

/// Renders a number the way the runtime prints it.
#[must_use]
pub fn number_to_string(number: f64) -> String {
    format_number(number)
}

#[cfg(test)]
mod tests;
