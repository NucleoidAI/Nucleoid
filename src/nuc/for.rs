//! `for x of ...` — carried out once over what is there now, rather than filed
//! as a standing declaration. Mirrors `ref/src/nuc/FOR.js`.

use crate::error::{Error, Result};
use crate::lang::ast::{Expr, Stmt};
use crate::lang::evaluation::Flow;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

impl Runtime {
    pub(crate) fn run_for(
        &mut self,
        variable: &str,
        iterable: &Expr,
        body: &[Stmt],
        scope: &mut Scope,
    ) -> Result<Flow> {
        let items = self.iterate(iterable, scope)?;

        for item in items {
            scope.push();
            scope.declare(variable.to_string(), item);
            self.enter_imperative();
            let result = self.execute_all(body, scope);
            self.leave_imperative();
            scope.pop();

            if let Flow::Return(value) = result? {
                return Ok(Flow::Return(value));
            }
        }

        Ok(Flow::Normal(Value::Null))
    }

    /// The objects a `for ... of` walks: instances of a class, or the instances
    /// found in a list. Plain values in a list are skipped.
    fn iterate(&mut self, iterable: &Expr, scope: &mut Scope) -> Result<Vec<Value>> {
        let value = self.evaluate(iterable, scope)?;

        Ok(match value {
            Value::Class(name) => self
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
                    Value::Object(id) => self
                        .state
                        .object(id)
                        .and_then(|object| object.class.as_ref())
                        .is_some(),
                    _ => false,
                })
                .collect(),
            _other => {
                return Err(Error::type_error(format!(
                    "{other} is not iterable",
                    other = iterable
                )));
            }
        })
    }
}
