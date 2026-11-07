//! Lists: literals, indexing, slicing and list operations.

mod aggregate;
mod index;
mod query;

use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::lang::ast::Expr;
use crate::number;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectId, Value};
pub use index::slice;

/// A list receiver resolved once, including side-effecting owners/keys, so
/// writing a result back never evaluates the receiver again.
pub(crate) enum ListTarget {
    /// A real property of an object, the only writable place.
    Property(ObjectId, String),
    /// A list read from a selected state version. History is never written.
    ReadOnly,
    /// A `.value` snapshot, or a member of something that is not an object:
    /// there is no place to write.
    Detached,
}

/// What a read-only list query runs over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListSource {
    /// A list value.
    List,
    /// A class's instance list, in population order.
    Population,
}

impl ListSource {
    /// What `at` returns for a position outside the receiver: the same value
    /// bracket indexing returns there, `list[9]` or `Name[9]`.
    fn missing(self) -> Value {
        match self {
            ListSource::List => Value::Undefined,
            ListSource::Population => Value::Null,
        }
    }
}

#[derive(Debug)]
pub struct Array<'a> {
    pub node: &'a Expr,
}

impl<'a> Array<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Array { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        match self.node {
            Expr::List(items) => runtime.evaluate_list(items, scope),
            Expr::Index { object, index } => runtime.evaluate_index(object, index, scope),
            Expr::Slice { object, start, end } => {
                runtime.evaluate_slice(object, start.as_deref(), end.as_deref(), scope)
            }
            other => Err(Error::type_error(format!("{other} is not a list"))),
        }
    }

    /// Renders the expression as source.
    #[must_use]
    pub fn generate(&self) -> String {
        self.node.to_string()
    }
}

impl Runtime {
    pub(crate) fn evaluate_list(&mut self, items: &[Expr], scope: &mut Scope) -> Result<Value> {
        let mut values = Vec::with_capacity(items.len());

        for item in items {
            values.push(self.evaluate(item, scope)?);
        }

        Ok(Value::List(values))
    }

    /// The instances of class `name`, in population order, read as a
    /// dependency on the population. `None` when no such class is declared.
    pub(crate) fn population(&mut self, name: &str) -> Option<Vec<Value>> {
        let instances = self
            .state
            .class(name)?
            .instances
            .iter()
            .cloned()
            .map(Value::Object)
            .collect();
        self.track(&NodeKey::class(name));
        Some(instances)
    }

    /// A method on a list value. `push`, `append`, `pop`, `shift` and `sort`
    /// write their result back to the receiver; the rest are read-only queries.
    pub(crate) fn call_list_method(
        &mut self,
        object: &Expr,
        mut items: Vec<Value>,
        property: &str,
        arguments: &[Expr],
        scope: &mut Scope,
        target: Option<ListTarget>,
    ) -> Result<Value> {
        match property {
            "push" | "append" => {
                let values = self.evaluate_all(arguments, scope)?;
                items.extend(values);
                let length = items.len();
                self.write_list(object, Value::List(items), scope, target)?;
                Ok(Value::Number(number::from_usize(length)))
            }

            "pop" => {
                let popped = items.pop().unwrap_or(Value::Undefined);
                self.write_list(object, Value::List(items), scope, target)?;
                Ok(popped)
            }

            "shift" => {
                let removed = if items.is_empty() {
                    Value::Undefined
                } else {
                    items.remove(0)
                };
                self.write_list(object, Value::List(items), scope, target)?;
                Ok(removed)
            }

            // Sorts in place, as JavaScript does. History and detached
            // receivers have no place to write, so they are sorted as copies.
            "sort" => {
                let values = self.evaluate_all(arguments, scope)?;
                let sorted = self.sort_items(items, values.first())?;

                if !matches!(target, Some(ListTarget::ReadOnly | ListTarget::Detached)) {
                    self.write_list(object, Value::List(sorted.clone()), scope, target)?;
                }

                Ok(Value::List(sorted))
            }

            _ => {
                let values = self.evaluate_all(arguments, scope)?;
                let receiver = object.path().unwrap_or_else(|| object.to_string());
                self.list_query(items, property, &values, &receiver, ListSource::List)
            }
        }
    }

    pub(crate) fn version_list_receiver(
        &mut self,
        expression: &Expr,
        scope: &mut Scope,
    ) -> Result<(Value, ListTarget)> {
        match expression {
            Expr::Member { object, property } if property != "value" => {
                self.member_list_receiver(object, property, scope, ListTarget::ReadOnly)
            }
            Expr::Index { object, index } => {
                let base = self.evaluate(object, scope)?;
                let key = self.evaluate(index, scope)?;
                let value = self.read_index(&base, &key);
                let target = match base {
                    Value::Object(id) => ListTarget::Property(id, key.to_string()),
                    _ => ListTarget::ReadOnly,
                };
                Ok((value, target))
            }
            _ => Ok((self.evaluate(expression, scope)?, ListTarget::ReadOnly)),
        }
    }

    /// Reads the receiver of a `sort` once. A member resolves to its object's
    /// property, so the sorted list is written without evaluating the
    /// receiver again; a `.value` snapshot is detached. Other receivers keep
    /// the ordinary write-back, which stores names and ignores the rest.
    pub(crate) fn sort_receiver(
        &mut self,
        expression: &Expr,
        scope: &mut Scope,
    ) -> Result<(Value, Option<ListTarget>)> {
        let Expr::Member { object, property } = expression else {
            return Ok((self.evaluate(expression, scope)?, None));
        };

        if property == "value" {
            let value = self.evaluate(expression, scope)?;
            return Ok((value, Some(ListTarget::Detached)));
        }

        let (value, target) =
            self.member_list_receiver(object, property, scope, ListTarget::Detached)?;
        Ok((value, Some(target)))
    }

    /// Reads `object.property` with the object evaluated once. A property of
    /// an object is the place to write; any other owner resolves to `otherwise`.
    fn member_list_receiver(
        &mut self,
        object: &Expr,
        property: &str,
        scope: &mut Scope,
        otherwise: ListTarget,
    ) -> Result<(Value, ListTarget)> {
        let base = self.evaluate(object, scope)?;
        let value = self.read_property(&base, property);
        let target = match base {
            Value::Object(id) => ListTarget::Property(id, property.to_string()),
            _ => otherwise,
        };
        Ok((value, target))
    }

    fn write_list(
        &mut self,
        target: &Expr,
        value: Value,
        scope: &mut Scope,
        resolved: Option<ListTarget>,
    ) -> Result<()> {
        match resolved {
            Some(ListTarget::ReadOnly) => Err(Error::type_error("State versions are read-only")),
            Some(ListTarget::Detached) => Ok(()),
            Some(ListTarget::Property(id, property)) => {
                let changed = self.assign_property(&id, &property, value);
                self.propagate_change(&NodeKey::property(&id, &property), changed)
            }
            None => self.write_back(target, value, scope),
        }
    }
}
