//! `RETURN` — ends the enclosing block with a value. Mirrors
//! `ref/src/nuc/RETURN.js`, which is likewise a node that only carries the
//! statement it returns; the stack is what acts on it.

use crate::error::Result;
use crate::lang::ast::Expr;
use crate::lang::evaluation::Flow;
use crate::nuc::Outcome;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

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
