use crate::error::{Error, Result};
use crate::lang::ast::{Expr, Stmt};
use crate::nuc::{self, MAX_ITERATIONS, Outcome};
use crate::runtime::Runtime;
use crate::scope::Scope;

#[derive(Debug, Clone)]
pub struct While {
    pub condition: Expr,
    pub statements: Vec<Stmt>,
}

impl While {
    #[must_use]
    pub fn new(condition: Expr, statements: Vec<Stmt>) -> Self {
        While {
            condition,
            statements,
        }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let mut iterations = 0;

        while self.holds(runtime, scope)? {
            if iterations == MAX_ITERATIONS {
                return Err(Error::type_error("Loop did not terminate"));
            }
            iterations += 1;

            if let Some(returned) = nuc::run_iteration(runtime, scope, &self.statements)? {
                return Ok(returned);
            }
        }

        Ok(Outcome::null())
    }

    fn holds(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<bool> {
        let (test, _) = runtime.evaluate_condition(&self.condition, scope)?;
        Ok(test.truthy())
    }
}
