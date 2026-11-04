//! Literal values.

use regex::Regex;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::lang::ast::Expr;
use crate::runtime::MAX_DEPTH;
use crate::value::Value;

#[derive(Debug)]
pub struct Literal<'a> {
    pub node: &'a Expr,
}

impl<'a> Literal<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Literal { node }
    }

    pub fn resolve(&self) -> Result<Value> {
        match self.node {
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
            other => Err(Error::type_error(format!("{other} is not a literal"))),
        }
    }

    /// Renders the literal as source.
    #[must_use]
    pub fn generate(&self) -> String {
        self.node.to_string()
    }
}

/// The literal expression for a value, where one exists. This is how a frozen
/// `.value` read is written back into a stored declaration.
#[must_use]
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

pub(crate) fn snapshot(value: &Value) -> Result<Expr> {
    let mut pending = vec![(value, 1)];
    while let Some((current, depth)) = pending.pop() {
        if depth > MAX_DEPTH {
            return Err(Error::type_error("Maximum expression depth exceeded"));
        }
        if let Value::List(items) = current {
            pending.extend(items.iter().map(|item| (item, depth + 1)));
        }
    }
    Ok(constant(value))
}

pub(crate) fn constant(value: &Value) -> Expr {
    match value {
        Value::Undefined | Value::Null => Expr::Null,
        Value::Bool(bool) => Expr::Bool(*bool),
        Value::Number(number) => Expr::Number(*number),
        Value::String(string) => Expr::String(string.clone()),
        Value::List(items) => Expr::List(items.iter().map(constant).collect()),
        Value::Object(id) => Expr::ObjectRef(id.to_string()),
        Value::Class(name) => Expr::Identifier(name.clone()),
        Value::Date(millis) => Expr::Call {
            callee: Box::new(Expr::Identifier("Date".to_string())),
            arguments: vec![Expr::Number(crate::number::from_i64(*millis))],
        },
        Value::Regex(regex) => Expr::Regex(regex.as_str().to_string()),
        Value::Function(function) => Expr::Function(function.clone()),
    }
}
