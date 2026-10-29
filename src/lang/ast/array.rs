//! Lists: literals, indexing, slicing and list operations.

use crate::builtins;
use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::lang::ast::Expr;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

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

pub struct Array<'a> {
    pub node: &'a Expr,
}

impl<'a> Array<'a> {
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
            other => unreachable!("{other} is not a list"),
        }
    }

    /// Renders the expression as source.
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

    pub(crate) fn evaluate_slice(
        &mut self,
        object: &Expr,
        start: Option<&Expr>,
        end: Option<&Expr>,
        scope: &mut Scope,
    ) -> Result<Value> {
        let base = self.evaluate(object, scope)?;

        let start = match start {
            Some(start) => Some(self.evaluate(start, scope)?),
            None => None,
        };
        let end = match end {
            Some(end) => Some(self.evaluate(end, scope)?),
            None => None,
        };

        Ok(slice(&base, start.as_ref(), end.as_ref()))
    }

    pub(crate) fn evaluate_index(
        &mut self,
        object: &Expr,
        index: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let base = self.evaluate(object, scope)?;
        let key = self.evaluate(index, scope)?;
        self.read_index(&base, &key, scope)
    }

    pub(crate) fn read_index(
        &mut self,
        base: &Value,
        key: &Value,
        scope: &mut Scope,
    ) -> Result<Value> {
        match base {
            Value::List(items) => {
                let index = key.to_number();
                let index = builtins::slice_index(index, items.len());
                Ok(items.get(index).cloned().unwrap_or(Value::Undefined))
            }

            Value::String(string) => {
                let index = builtins::slice_index(key.to_number(), string.chars().count());
                Ok(string
                    .chars()
                    .nth(index)
                    .map(|character| Value::String(character.to_string()))
                    .unwrap_or(Value::Undefined))
            }

            Value::Class(name) => {
                let Some(class) = self.state.class(name) else {
                    return Ok(Value::Undefined);
                };

                let instances = class.instances.clone();
                self.track(NodeKey::class(name));

                let found = match key {
                    Value::Number(index) => instances.get(*index as usize).cloned(),
                    other => {
                        let wanted = other.to_string();
                        instances
                            .iter()
                            .find(|instance| instance.as_str() == wanted)
                            .cloned()
                    }
                };

                Ok(match found {
                    Some(id) => Value::Object(id),
                    None => Value::Null,
                })
            }

            Value::Object(_) => self.read_property(base, &key.to_string(), scope),

            _ => Ok(Value::Undefined),
        }
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
                Ok(Value::Number(length as f64))
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
                self.list_query(items, property, &values, &receiver)
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
                let value = self.read_index(&base, &key, scope)?;
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
        let value = self.read_property(&base, property, scope)?;
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

    /// The read-only list operations, shared by lists and class instance lists.
    pub(crate) fn list_query(
        &mut self,
        items: Vec<Value>,
        property: &str,
        arguments: &[Value],
        receiver: &str,
    ) -> Result<Value> {
        let predicate = arguments.first().cloned();

        match property {
            "find" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("find expects a function"));
                };

                for item in items {
                    if self
                        .invoke(&function, std::slice::from_ref(&item))?
                        .truthy()
                    {
                        return Ok(item);
                    }
                }

                Ok(Value::Null)
            }

            "filter" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("filter expects a function"));
                };

                let mut matched = Vec::new();

                for item in items {
                    if self
                        .invoke(&function, std::slice::from_ref(&item))?
                        .truthy()
                    {
                        matched.push(item);
                    }
                }

                Ok(Value::List(matched))
            }

            "map" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("map expects a function"));
                };

                let mut mapped = Vec::new();

                for item in items {
                    mapped.push(self.invoke(&function, &[item])?);
                }

                Ok(Value::List(mapped))
            }

            "flatMap" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("flatMap expects a function"));
                };

                let mut mapped = Vec::new();

                for item in items {
                    match self.invoke(&function, &[item])? {
                        Value::List(values) => mapped.extend(values),
                        value => mapped.push(value),
                    }
                }

                Ok(Value::List(mapped))
            }

            "includes" | "contains" => {
                let wanted = predicate.unwrap_or(Value::Undefined);
                Ok(Value::Bool(
                    items.iter().any(|item| self.deep_equal(item, &wanted)),
                ))
            }

            "indexOf" => {
                let wanted = predicate.unwrap_or(Value::Undefined);
                let found = items.iter().position(|item| self.deep_equal(item, &wanted));
                Ok(Value::Number(
                    found.map(|index| index as f64).unwrap_or(-1.0),
                ))
            }

            "join" => {
                let separator = predicate.map(|value| value.to_string()).unwrap_or_default();
                let rendered: Vec<String> = items.iter().map(|item| item.to_string()).collect();
                Ok(Value::String(rendered.join(&separator)))
            }

            "every" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("every expects a function"));
                };

                for item in items {
                    if !self
                        .invoke(&function, std::slice::from_ref(&item))?
                        .truthy()
                    {
                        return Ok(Value::Bool(false));
                    }
                }

                Ok(Value::Bool(true))
            }

            "some" | "any" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("some expects a function"));
                };

                for item in items {
                    if self
                        .invoke(&function, std::slice::from_ref(&item))?
                        .truthy()
                    {
                        return Ok(Value::Bool(true));
                    }
                }

                Ok(Value::Bool(false))
            }

            "reduce" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("reduce expects a function"));
                };

                let mut items = items.into_iter();

                let mut accumulator = match arguments.get(1) {
                    Some(initial) => initial.clone(),
                    None => match items.next() {
                        Some(first) => first,
                        None => return Ok(Value::Undefined),
                    },
                };

                for item in items {
                    accumulator = self.invoke(&function, &[accumulator, item])?;
                }

                Ok(accumulator)
            }

            // Lists sort in place through `call_list_method`; a class population
            // has no stored list to write, so its sort only returns the order.
            "sort" => Ok(Value::List(self.sort_items(items, predicate.as_ref())?)),

            "slice" => {
                let length = items.len();
                let start = arguments
                    .first()
                    .map(|value| builtins::slice_index(value.to_number(), length))
                    .unwrap_or(0);
                let end = arguments
                    .get(1)
                    .map(|value| builtins::slice_index(value.to_number(), length))
                    .unwrap_or(length);

                Ok(Value::List(items[start.min(end)..end].to_vec()))
            }

            "reverse" => {
                let mut reversed = items;
                reversed.reverse();
                Ok(Value::List(reversed))
            }

            "concat" => {
                let mut joined = items;

                for argument in arguments {
                    match argument {
                        Value::List(other) => joined.extend(other.iter().cloned()),
                        other => joined.push(other.clone()),
                    }
                }

                Ok(Value::List(joined))
            }

            "length" => Ok(Value::Number(items.len() as f64)),

            _ => Err(Error::type_error(format!(
                "{receiver}.{property} is not a function"
            ))),
        }
    }

    /// Orders items stably. A comparator function places each item, in input
    /// order, before the first already placed item it compares below zero;
    /// without one, items order by their string forms.
    fn sort_items(
        &mut self,
        mut items: Vec<Value>,
        comparator: Option<&Value>,
    ) -> Result<Vec<Value>> {
        let Some(Value::Function(function)) = comparator else {
            // Renders each string form once rather than on every comparison;
            // the sort is still stable.
            items.sort_by_cached_key(|item| item.to_string());
            return Ok(items);
        };

        let mut ordered: Vec<Value> = Vec::with_capacity(items.len());

        for item in items {
            let mut index = ordered.len();

            for (position, placed) in ordered.iter().enumerate() {
                let comparison = self.invoke(function, &[item.clone(), placed.clone()])?;

                if comparison.to_number() < 0.0 {
                    index = position;
                    break;
                }
            }

            ordered.insert(index, item);
        }

        Ok(ordered)
    }

    /// Stores a mutated list back where it came from and wakes its dependents.
    fn write_back(&mut self, target: &Expr, value: Value, scope: &mut Scope) -> Result<()> {
        match target {
            Expr::Identifier(name) => {
                if scope.assign(name, value.clone()) {
                    return Ok(());
                }

                let changed = self.assign(name, value);
                let key = NodeKey::variable(name.clone());
                self.propagate_change(&key, changed)?;
                Ok(())
            }

            Expr::Member { object, property } => {
                let base = self.evaluate(object, scope)?;

                if let Value::Object(id) = base {
                    let changed = self.assign_property(&id, property, value);
                    let key = NodeKey::property(&id, property);
                    self.propagate_change(&key, changed)?;
                } else if target.version_view() {
                    return Err(Error::type_error("State versions are read-only"));
                }

                Ok(())
            }

            other if other.version_view() => Err(Error::type_error("State versions are read-only")),
            _ => Ok(()),
        }
    }
}

pub fn slice(base: &Value, start: Option<&Value>, end: Option<&Value>) -> Value {
    match base {
        Value::String(string) => Value::String(builtins::string_slice(
            string,
            start.map(Value::to_number).unwrap_or(0.0),
            end.map(Value::to_number),
        )),

        Value::List(items) => {
            let length = items.len();
            let from = start
                .map(|value| builtins::slice_index(value.to_number(), length))
                .unwrap_or(0);
            let to = end
                .map(|value| builtins::slice_index(value.to_number(), length))
                .unwrap_or(length);

            Value::List(items[from.min(to)..to].to_vec())
        }

        _ => Value::Undefined,
    }
}
