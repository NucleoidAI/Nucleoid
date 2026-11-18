//! `try`/`catch`.
//!
//! A caught failure restores only the changes made by the failing body.
//! Pending work and deletion tracking are restored before entering the catch.
//! This statement executes immediately rather than being filed as a declaration.

use crate::error::{Error, Result};
use crate::lang::ast::Stmt;
use crate::lang::evaluation::Flow;
use crate::nuc::{Outcome, throw};
use crate::runtime::Runtime;
use crate::scope::Scope;

#[derive(Debug, Clone)]
pub struct Try {
    pub body: Vec<Stmt>,
    pub parameter: String,
    pub catch: Vec<Stmt>,
    pub finally: Option<Vec<Stmt>>,
}

impl Try {
    #[must_use]
    pub fn new(body: Vec<Stmt>, parameter: String, catch: Vec<Stmt>) -> Self {
        Try {
            body,
            parameter,
            catch,
            finally: None,
        }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let savepoint = runtime.savepoint();

        let pending = match runtime.execute_all(&self.body, scope) {
            Ok(flow) => Ok(flow),
            Err(error) => {
                runtime.rollback_to(savepoint);
                self.recover(runtime, scope, &error)
            }
        };

        let Some(finally) = &self.finally else {
            return pending.map(Outcome::flow);
        };
        match runtime.execute_all(finally, scope)? {
            returned @ Flow::Return(_) => Ok(Outcome::flow(returned)),
            Flow::Normal(_) => pending.map(Outcome::flow),
        }
    }

    fn recover(&self, runtime: &mut Runtime, scope: &mut Scope, error: &Error) -> Result<Flow> {
        let savepoint = self.finally.is_some().then(|| runtime.savepoint());

        scope.push();
        scope.declare(self.parameter.clone(), throw::caught(error));
        let result = runtime.execute_all(&self.catch, scope);
        scope.pop();

        if let (Err(_), Some(savepoint)) = (&result, savepoint) {
            runtime.rollback_to(savepoint);
        }
        result
    }
}
