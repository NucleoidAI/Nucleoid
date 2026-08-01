//! `FOR` — walked once over what is there now, rather than filed as a standing
//! declaration. Mirrors `ref/src/nuc/FOR.js`, which likewise keeps the loop
//! variable, the list and the body on the node and steps through the list.

use crate::error::{Error, Result};
use crate::lang::ast::{Expr, Stmt};
use crate::lang::evaluation::Flow;
use crate::nuc::Outcome;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct For {
    pub variable: String,
    pub array: Expr,
    pub statements: Vec<Stmt>,
}

impl For {
    pub fn new(variable: String, array: Expr, statements: Vec<Stmt>) -> Self {
        For {
            variable,
            array,
            statements,
        }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let items = self.iterate(runtime, scope)?;

        for item in items {
            scope.push();
            scope.declare(self.variable.clone(), item);
            runtime.enter_imperative();
            let result = runtime.execute_all(&self.statements, scope);
            runtime.leave_imperative();
            scope.pop();

            if let Flow::Return(value) = result? {
                return Ok(Outcome::flow(Flow::Return(value)));
            }
        }

        Ok(Outcome::null())
    }

    /// The objects the loop walks: instances of a class, or the instances found
    /// in a list. Plain values in a list are skipped.
    fn iterate(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Vec<Value>> {
        let value = runtime.evaluate(&self.array, scope)?;

        Ok(match value {
            Value::Class(name) => runtime
                .state
                .class(&name)
                .map(|class| class.instances.clone())
                .unwrap_or_default()
                .into_iter()
                .map(Value::Object)
                .collect(),
            Value::List(items) => items
                .into_iter()
                .filter(|item| match item {
                    Value::Object(id) => runtime
                        .state
                        .object(id)
                        .and_then(|object| object.class.as_ref())
                        .is_some(),
                    _ => false,
                })
                .collect(),
            _ => {
                return Err(Error::type_error(format!("{} is not iterable", self.array)));
            }
        })
    }
}
