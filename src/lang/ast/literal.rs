//! Literals. Mirrors `ref/src/lang/ast/Literal.js`.

use regex::Regex;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::lang::ast::Expr;
use crate::value::Value;

pub fn evaluate(expression: &Expr) -> Result<Value> {
    match expression {
        Expr::Null => Ok(Value::Null),
        Expr::Bool(bool) => Ok(Value::Bool(*bool)),
        Expr::Number(number) => Ok(Value::Number(*number)),
        Expr::String(string) => Ok(Value::String(string.clone())),
        Expr::Regex(pattern) => match Regex::new(pattern) {
            Ok(regex) => Ok(Value::Regex(Arc::new(regex))),
            Err(error) => Err(Error::syntax(format!(
                "Invalid regular expression: {error}"
            ))),
        },
        other => unreachable!("{other} is not a literal"),
    }
}

/// The literal expression for a value, where one exists. This is how a frozen
/// `.value` read is written back into a stored declaration.
pub fn of(value: &Value) -> Option<Expr> {
    Some(match value {
        Value::Null | Value::Undefined => Expr::Null,
        Value::Bool(bool) => Expr::Bool(*bool),
        Value::Number(number) => Expr::Number(*number),
        Value::String(string) => Expr::String(string.clone()),
        Value::Object(id) => Expr::ObjectRef(id.to_string()),
        _ => return None,
    })
}
