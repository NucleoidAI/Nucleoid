use crate::error::{Error, Result};
use crate::lang::ast::{Expr, Stmt};
use crate::nuc::{self, MAX_ITERATIONS, Outcome};
use crate::runtime::Runtime;
use crate::scope::Scope;

#[derive(Debug, Clone)]
pub struct RepeatUntil {
    statements: Vec<Stmt>,
    condition: Expr,
}

impl RepeatUntil {
    #[must_use]
    pub(crate) fn new(statements: Vec<Stmt>, condition: Expr) -> Self {
        RepeatUntil {
            statements,
            condition,
        }
    }

    pub(crate) fn run(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        for _ in 0..MAX_ITERATIONS {
            if let Some(returned) = nuc::run_iteration(runtime, scope, &self.statements)? {
                return Ok(returned);
            }
            if self.holds(runtime, scope)? {
                return Ok(Outcome::null());
            }
        }

        Err(Error::type_error("Loop did not terminate"))
    }

    fn holds(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<bool> {
        let (test, _) = runtime.evaluate_condition(&self.condition, scope)?;
        Ok(test.truthy())
    }
}
