use indexmap::IndexMap;
use regex::Regex;
use std::fmt;
use std::sync::Arc;

use crate::lang::ast::Function;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(pub String);

impl ObjectId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for ObjectId {
    fn from(value: &str) -> Self {
        ObjectId(value.to_string())
    }
}

impl From<String> for ObjectId {
    fn from(value: String) -> Self {
        ObjectId(value)
    }
}

/// A runtime value.
///
/// `Undefined` is distinct from `Null`: it marks a read of something that has
/// never been defined, which makes a dependent assignment evaluate to `Null`
/// rather than raising.
#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<Value>),
    Object(ObjectId),
    Class(String),
    Date(i64),
    Regex(Arc<Regex>),
    Function(Arc<Function>),
}

impl Value {
    pub fn number(value: impl Into<f64>) -> Self {
        Value::Number(value.into())
    }

    pub fn string(value: impl Into<String>) -> Self {
        Value::String(value.into())
    }

    pub fn is_undefined(&self) -> bool {
        matches!(self, Value::Undefined)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_nullish(&self) -> bool {
        matches!(self, Value::Null | Value::Undefined)
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Undefined | Value::Null => false,
            Value::Bool(bool) => *bool,
            Value::Number(number) => *number != 0.0 && !number.is_nan(),
            Value::String(string) => !string.is_empty(),
            Value::Date(_) | Value::Object(_) | Value::Class(_) => true,
            Value::List(_) | Value::Regex(_) | Value::Function(_) => true,
        }
    }

    /// Numeric coercion following the spec's arithmetic: `null` reads as zero,
    /// booleans as 0/1, and anything non-numeric as NaN.
    pub fn to_number(&self) -> f64 {
        match self {
            Value::Undefined => f64::NAN,
            Value::Null => 0.0,
            Value::Bool(bool) => {
                if *bool {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Number(number) => *number,
            Value::Date(millis) => *millis as f64,
            Value::String(string) => {
                let trimmed = string.trim();
                if trimmed.is_empty() {
                    0.0
                } else {
                    trimmed.parse::<f64>().unwrap_or(f64::NAN)
                }
            }
            Value::List(list) => match list.as_slice() {
                [] => 0.0,
                [single] => single.to_number(),
                _ => f64::NAN,
            },
            Value::Object(_) | Value::Class(_) | Value::Regex(_) | Value::Function(_) => f64::NAN,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Undefined => "undefined",
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Object(_) => "object",
            Value::Class(_) => "class",
            Value::Date(_) => "date",
            Value::Regex(_) => "regex",
            Value::Function(_) => "function",
        }
    }
}

/// Formats a float the way the runtime renders numbers: integral values without
/// a trailing `.0`, everything else with the shortest round-tripping form.
pub fn format_number(number: f64) -> String {
    if number.is_nan() {
        return "NaN".to_string();
    }
    if number.is_infinite() {
        return if number > 0.0 {
            "Infinity"
        } else {
            "-Infinity"
        }
        .to_string();
    }
    if number == 0.0 {
        return "0".to_string();
    }
    format!("{number}")
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undefined => f.write_str("undefined"),
            Value::Null => f.write_str("null"),
            Value::Bool(bool) => write!(f, "{bool}"),
            Value::Number(number) => f.write_str(&format_number(*number)),
            Value::String(string) => f.write_str(string),
            Value::List(list) => {
                let rendered: Vec<String> = list.iter().map(|item| item.to_string()).collect();
                f.write_str(&rendered.join(","))
            }
            Value::Object(id) => write!(f, "[object {id}]"),
            Value::Class(name) => write!(f, "class {name}"),
            Value::Date(millis) => write!(f, "{}", crate::builtins::date::to_string(*millis)),
            Value::Regex(regex) => write!(f, "/{}/", regex.as_str()),
            Value::Function(_) => f.write_str("[function]"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::List(left), Value::List(right)) => left == right,
            (Value::Object(left), Value::Object(right)) => left == right,
            (Value::Class(left), Value::Class(right)) => left == right,
            (Value::Date(left), Value::Date(right)) => left == right,
            (Value::Regex(left), Value::Regex(right)) => left.as_str() == right.as_str(),
            (Value::Function(left), Value::Function(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }
}

/// An object in the state: either a class instance or a plain object literal.
#[derive(Debug, Clone, Default)]
pub struct ObjectData {
    pub class: Option<String>,
    pub properties: IndexMap<String, Value>,
}

impl ObjectData {
    pub fn new(class: Option<String>) -> Self {
        ObjectData {
            class,
            properties: IndexMap::new(),
        }
    }
}
