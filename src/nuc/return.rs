//! Returning a value from the enclosing block.

use crate::error::Result;
use crate::lang::ast::Expr;
use crate::lang::evaluation::Flow;
use crate::nuc::Outcome;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Return {
    pub statement: Option<Expr>,
}

impl Return {
    pub fn new(statement: Option<Expr>) -> Self {
        Return { statement }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let value = match &self.statement {
            Some(expression) => runtime.evaluate(expression, scope)?,
            None => Value::Null,
        };

        Ok(Outcome::flow(Flow::Return(value)))
    }
}
