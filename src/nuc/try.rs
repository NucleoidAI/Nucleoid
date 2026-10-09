//! `try`/`catch`.
//!
//! A caught failure restores only the changes made by the failing body.
//! Pending work and deletion tracking are restored before entering the catch.
//! This statement executes immediately rather than being filed as a declaration.

use crate::error::Result;
use crate::lang::ast::Stmt;
use crate::nuc::{Outcome, throw};
use crate::runtime::Runtime;
use crate::scope::Scope;

#[derive(Debug, Clone)]
pub struct Try {
    pub body: Vec<Stmt>,
    pub parameter: String,
    pub catch: Vec<Stmt>,
}

impl Try {
    pub fn new(body: Vec<Stmt>, parameter: String, catch: Vec<Stmt>) -> Self {
        Try {
            body,
            parameter,
            catch,
        }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let savepoint = runtime.savepoint();

        match runtime.execute_all(&self.body, scope) {
            Ok(flow) => Ok(Outcome::flow(flow)),
            Err(error) => {
                runtime.rollback_to(savepoint);

                scope.push();
                scope.declare(self.parameter.clone(), throw::caught(&error));
                let result = runtime.execute_all(&self.catch, scope);
                scope.pop();

                Ok(Outcome::flow(result?))
            }
        }
    }
}
