use crate::error::{Error, Result};
use crate::lang::ast::{Expr, Stmt};
use crate::nuc::{self, MAX_ITERATIONS, Outcome};
use crate::number;
use crate::runtime::Runtime;
use crate::scope::Scope;

#[derive(Debug, Clone)]
pub struct Repeat {
    count: Expr,
    statements: Vec<Stmt>,
}

impl Repeat {
    #[must_use]
    pub(crate) fn new(count: Expr, statements: Vec<Stmt>) -> Self {
        Repeat { count, statements }
    }

    pub(crate) fn run(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let iterations = self.iterations(runtime, scope)?;

        for _ in 0..iterations {
            if let Some(returned) = nuc::run_iteration(runtime, scope, &self.statements)? {
                return Ok(returned);
            }
        }

        Ok(Outcome::null())
    }

    fn iterations(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<usize> {
        let (count, _) = runtime.evaluate_condition(&self.count, scope)?;
        let iterations = number::repetitions(count.to_number());
        if iterations > MAX_ITERATIONS {
            return Err(Error::type_error("Repeat count is too large"));
        }
        Ok(iterations)
    }
}
