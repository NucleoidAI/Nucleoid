//! For loops execute once over a source snapshot rather than being filed as
//! standing declarations.

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
    pub value: Option<String>,
    pub array: Expr,
    pub statements: Vec<Stmt>,
}

impl For {
    #[must_use]
    pub fn new(variable: String, array: Expr, statements: Vec<Stmt>) -> Self {
        For {
            variable,
            value: None,
            array,
            statements,
        }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        match &self.value {
            None => {
                let items = self.iterate(runtime, scope)?;
                self.repeat(runtime, scope, items, |scope, item| {
                    scope.declare(self.variable.clone(), item);
                })
            }
            Some(value) => {
                let entries = self.entries(runtime, scope)?;
                self.repeat(runtime, scope, entries, |scope, (key, entry)| {
                    scope.declare(self.variable.clone(), Value::String(key));
                    scope.declare(value.clone(), entry);
                })
            }
        }
    }

    fn repeat<T>(
        &self,
        runtime: &mut Runtime,
        scope: &mut Scope,
        snapshot: Vec<T>,
        bind: impl Fn(&mut Scope, T),
    ) -> Result<Outcome> {
        for item in snapshot {
            scope.push();
            bind(scope, item);
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

    fn entries(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Vec<(String, Value)>> {
        let Value::Object(id) = runtime.evaluate(&self.array, scope)? else {
            return Err(Error::type_error(format!(
                "{} is not an object",
                self.array
            )));
        };

        Ok(runtime
            .state
            .object(&id)
            .map(|object| {
                object
                    .properties
                    .iter()
                    .filter(|(_, value)| !value.is_undefined())
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Snapshot class instances, registered instances in a list, or a string's
    /// Unicode scalars. Plain values in a list are still skipped.
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
            Value::String(text) => text
                .chars()
                .map(|character| Value::String(character.to_string()))
                .collect(),
            _ => {
                return Err(Error::type_error(format!("{} is not iterable", self.array)));
            }
        })
    }
}
