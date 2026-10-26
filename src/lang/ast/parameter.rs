//! Scalar formal defaults are data, not expressions or captured environments.

use std::fmt;

use super::Parameter;
use crate::value::Value;

impl fmt::Debug for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parameter = f.debug_struct("Parameter");
        parameter
            .field("name", &self.name)
            .field("type_name", &self.type_name);
        // Retain accepted AST debug seals for every legacy formal. New
        // metadata is visible when present, not an invented legacy field.
        if self.default.is_some() {
            parameter.field("default", &self.default);
        }
        parameter.finish()
    }
}

/// The immutable scalar literals permitted in a parenthesized formal list.
/// The parser accepts only finite numbers, preserving their signed-zero bits.
#[derive(Debug, Clone)]
pub enum ParameterDefault {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
}

impl PartialEq for ParameterDefault {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => left.to_bits() == right.to_bits(),
            (Self::String(left), Self::String(right)) => left == right,
            _ => false,
        }
    }
}

impl ParameterDefault {
    fn value(&self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Bool(value) => Value::Bool(*value),
            Self::Number(value) => Value::Number(*value),
            Self::String(value) => Value::String(value.clone()),
        }
    }
}

impl Parameter {
    /// Resolve omission before declaring the fresh Scope slot. Supplied Null,
    /// Undefined and falsy values are ordinary values, never omission markers.
    pub(crate) fn effective_value(&self, supplied: Option<Value>) -> Value {
        supplied.unwrap_or_else(|| {
            self.default
                .as_ref()
                .map_or(Value::Null, ParameterDefault::value)
        })
    }
}

impl fmt::Display for ParameterDefault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => f.write_str("null"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::Number(value) if *value == 0.0 && value.is_sign_negative() => f.write_str("-0"),
            Self::Number(value) => f.write_str(&crate::value::format_number(*value)),
            Self::String(value) => {
                f.write_str("\"")?;
                for character in value.chars() {
                    // Use the existing quoted-string lexer's escapes, not
                    // JSON's unsupported Unicode escape spelling.
                    match character {
                        '"' => f.write_str("\\\"")?,
                        '\\' => f.write_str("\\\\")?,
                        '\n' => f.write_str("\\n")?,
                        '\r' => f.write_str("\\r")?,
                        '\t' => f.write_str("\\t")?,
                        '\0' => f.write_str("\\0")?,
                        other => write!(f, "{other}")?,
                    }
                }
                f.write_str("\"")
            }
        }
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;
        if let Some(type_name) = &self.type_name {
            write!(f, ":{type_name}")?;
        }
        if let Some(default) = &self.default {
            write!(f, "={default}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_absence_uses_the_default() {
        let parameter = Parameter {
            name: "value".to_string(),
            type_name: None,
            default: Some(ParameterDefault::Number(12.0)),
        };
        assert_eq!(parameter.effective_value(None), Value::Number(12.0));
        for supplied in [
            Value::Null,
            Value::Undefined,
            Value::Bool(false),
            Value::Number(0.0),
            Value::String(String::new()),
            Value::List(Vec::new()),
        ] {
            assert_eq!(parameter.effective_value(Some(supplied.clone())), supplied);
        }
        let plain = Parameter {
            default: None,
            ..parameter
        };
        assert_eq!(plain.effective_value(None), Value::Null);
    }

    #[test]
    fn absent_default_metadata_preserves_the_accepted_ast_debug_representation() {
        let mut parameter = Parameter {
            name: "value".to_string(),
            type_name: None,
            default: None,
        };
        assert_eq!(
            format!("{parameter:?}"),
            r#"Parameter { name: "value", type_name: None }"#
        );
        parameter.default = Some(ParameterDefault::Number(12.0));
        assert!(format!("{parameter:?}").contains("default: Some(Number(12.0))"));
    }
}
