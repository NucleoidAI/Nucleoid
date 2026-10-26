//! A version selects a stored Value, not a member of the current value.

use crate::builtins;
use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::history::HistoryKey;
use crate::lang::ast::{Expr, VersionSelector};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

pub struct Version<'a> {
    pub node: &'a Expr,
}

impl<'a> Version<'a> {
    pub fn new(node: &'a Expr) -> Self {
        Self { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        let Expr::Version {
            reference,
            selector,
        } = self.node
        else {
            unreachable!("Version only wraps Expr::Version")
        };
        runtime.read_version(reference, *selector, scope)
    }
}

impl Runtime {
    fn read_version(
        &mut self,
        reference: &Expr,
        selector: VersionSelector,
        scope: &mut Scope,
    ) -> Result<Value> {
        // Receiver/key evaluation still tracks dependencies and side effects,
        // but the CURRENT slot's nullishness cannot poison a valid old value.
        let null = std::mem::take(&mut self.null_read);
        let undefined = std::mem::take(&mut self.undefined_read);
        let result = self.version_of_slot(reference, selector, scope);
        self.null_read = null;
        self.undefined_read = undefined;
        let value = result?;
        self.note_nullish(&value);
        self.undefined_read |= value.is_undefined();
        Ok(value)
    }

    fn version_of_slot(
        &mut self,
        reference: &Expr,
        selector: VersionSelector,
        scope: &mut Scope,
    ) -> Result<Value> {
        let key = match reference {
            Expr::Identifier(name) => {
                if scope.has(name) {
                    return Ok(scope
                        .version(name, selector)
                        .cloned()
                        .unwrap_or(Value::Undefined));
                }
                let dependency = NodeKey::variable(name);
                // An existing history observer can outlive variable storage
                // (for example when `def` replaces it). Defer that missing
                // slot during cascades just as deletion defers other reads,
                // while a new direct namespace read is still not a slot.
                let deferred = self.deleted.contains(&dependency)
                    || self.stack.is_draining()
                        && self.graph.retrieve(&dependency).is_some_and(|node| {
                            node.dependents
                                .iter()
                                .any(|dependent| self.stack.is_running(dependent))
                        });
                if !self.state.has_variable(name)
                    && !deferred
                    && (self.state.has_class(name)
                        || self.state.has_function(name)
                        || builtins::is_global(name))
                {
                    return Err(version_reference_error());
                }
                self.track(dependency);
                HistoryKey::Variable(name.clone())
            }
            Expr::Member { object, property } if property != "value" => {
                let base = self.evaluate(object, scope)?;
                return self.property_version(base, property, selector);
            }
            Expr::Index { object, index } => {
                // Evaluate both exactly once, in the ordinary receiver/key order.
                let base = self.evaluate(object, scope)?;
                let key = self.evaluate(index, scope)?;
                return self.property_version(base, &key.to_string(), selector);
            }
            _ => return Err(version_reference_error()),
        };
        Ok(self
            .state
            .version(&key, selector)
            .cloned()
            .unwrap_or(Value::Undefined))
    }

    fn property_version(
        &mut self,
        base: Value,
        property: &str,
        selector: VersionSelector,
    ) -> Result<Value> {
        match base {
            Value::Object(id) => {
                self.track(NodeKey::property(&id, property));
                Ok(self
                    .state
                    .version(&HistoryKey::Property(id, property.to_string()), selector)
                    .cloned()
                    .unwrap_or(Value::Undefined))
            }
            Value::Null | Value::Undefined => Ok(Value::Undefined),
            _ => Err(version_reference_error()),
        }
    }
}

fn version_reference_error() -> Error {
    Error::type_error("State versions require a variable or object property")
}
