//! `THROW` — raises a value, which rolls the transaction back to wherever it is
//! caught. Mirrors `ref/src/nuc/THROW.js`.

use crate::error::{Error, Result};
use crate::lang::ast::Expr;
use crate::nuc::Outcome;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Throw {
    pub exception: Expr,
}

impl Throw {
    pub fn new(exception: Expr) -> Self {
        Throw { exception }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let value = runtime.evaluate(&self.exception, scope)?;
        Err(Error::thrown(value))
    }
}

/// The value a `catch` binds. Positions are deliberately left out: this text is
/// part of the language's behaviour and is compared against in programs.
pub(crate) fn caught(error: &Error) -> Value {
    match error.thrown_value() {
        Some(value) => value.clone(),
        None => Value::String(format!("{}: {}", error.kind().as_str(), error.message())),
    }
}
