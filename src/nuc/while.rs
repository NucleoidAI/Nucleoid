use crate::error::{Error, Result};
use crate::lang::ast::{Expr, Stmt};
use crate::lang::evaluation::Flow;
use crate::nuc::Outcome;
use crate::runtime::Runtime;
use crate::scope::Scope;

const MAX_ITERATIONS: usize = 100_000;

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

            runtime.enter_imperative();
            let result = runtime.execute_all(&self.statements, scope);
            runtime.leave_imperative();

            if let Flow::Return(value) = result? {
                return Ok(Outcome::flow(Flow::Return(value)));
            }
        }

        Ok(Outcome::null())
    }

    fn holds(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<bool> {
        let (test, _) = runtime.evaluate_condition(&self.condition, scope)?;
        Ok(test.truthy())
    }
}
