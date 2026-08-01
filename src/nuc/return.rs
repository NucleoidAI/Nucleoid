//! `return` — ends the enclosing block with a value. Mirrors
//! `ref/src/nuc/RETURN.js`.

use crate::error::Result;
use crate::lang::ast::Expr;
use crate::lang::evaluation::Flow;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

impl Runtime {
    pub(crate) fn run_return(
        &mut self,
        expression: Option<&Expr>,
        scope: &mut Scope,
    ) -> Result<Flow> {
        let value = match expression {
            Some(expression) => self.evaluate(expression, scope)?,
            None => Value::Null,
        };

        Ok(Flow::Return(value))
    }
}
