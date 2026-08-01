//! Lists: the literal, indexing, slicing and the list operations. Mirrors
//! `ref/src/lang/ast/Array.js`.

use crate::builtins;
use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::lang::ast::Expr;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

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

    /// `Array.generate(scope)` — `ref` renders the elements rather than the
    /// node, since a list is built from its parts.
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
        self.read_index(&base, &key)
    }

    fn read_index(&mut self, base: &Value, key: &Value) -> Result<Value> {
        match base {
            Value::List(items) => {
                let index = key.to_number();
                let index = builtins::slice_index(index, items.len());
                Ok(items.get(index).cloned().unwrap_or(Value::Undefined))
            }

            Value::String(string) => {
                let characters: Vec<char> = string.chars().collect();
                let index = builtins::slice_index(key.to_number(), characters.len());
                Ok(characters
                    .get(index)
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

            Value::Object(id) => {
                let property = key.to_string();
                self.track(NodeKey::property(id, &property));
                Ok(self
                    .state
                    .property(id, &property)
                    .cloned()
                    .unwrap_or(Value::Undefined))
            }

            _ => Ok(Value::Undefined),
        }
    }

    pub(crate) fn call_list_method(
        &mut self,
        object: &Expr,
        mut items: Vec<Value>,
        property: &str,
        arguments: &[Expr],
        scope: &mut Scope,
    ) -> Result<Value> {
        match property {
            "push" | "append" => {
                let values = self.evaluate_all(arguments, scope)?;
                items.extend(values);
                let length = items.len();
                self.write_back(object, Value::List(items), scope)?;
                Ok(Value::Number(length as f64))
            }

            "pop" => {
                let popped = items.pop().unwrap_or(Value::Undefined);
                self.write_back(object, Value::List(items), scope)?;
                Ok(popped)
            }

            "shift" => {
                let removed = if items.is_empty() {
                    Value::Undefined
                } else {
                    items.remove(0)
                };
                self.write_back(object, Value::List(items), scope)?;
                Ok(removed)
            }

            _ => {
                let values = self.evaluate_all(arguments, scope)?;
                let receiver = object.path().unwrap_or_else(|| object.to_string());
                self.list_query(items, property, &values, &receiver)
            }
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

            "sort" => {
                let mut sorted = items;

                match predicate {
                    Some(Value::Function(function)) => {
                        // A comparator sort, kept simple with insertion order.
                        let mut ordered: Vec<Value> = Vec::with_capacity(sorted.len());

                        for item in sorted {
                            let mut index = ordered.len();

                            for (position, placed) in ordered.iter().enumerate() {
                                let comparison =
                                    self.invoke(&function, &[item.clone(), placed.clone()])?;

                                if comparison.to_number() < 0.0 {
                                    index = position;
                                    break;
                                }
                            }

                            ordered.insert(index, item);
                        }

                        Ok(Value::List(ordered))
                    }
                    _ => {
                        sorted.sort_by_key(|item| item.to_string());
                        Ok(Value::List(sorted))
                    }
                }
            }

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

    /// Stores a mutated list back where it came from and wakes its dependents.
    fn write_back(&mut self, target: &Expr, value: Value, scope: &mut Scope) -> Result<()> {
        match target {
            Expr::Identifier(name) => {
                if scope.assign(name, value.clone()) {
                    return Ok(());
                }

                self.assign(name, value);
                let key = NodeKey::variable(name.clone());
                self.propagate(&key)?;
                Ok(())
            }

            Expr::Member { object, property } => {
                let base = self.evaluate(object, scope)?;

                if let Value::Object(id) = base {
                    self.assign_property(&id, property, value);
                    let key = NodeKey::property(&id, property);
                    self.propagate(&key)?;
                }

                Ok(())
            }

            _ => Ok(()),
        }
    }
}

pub fn slice(base: &Value, start: Option<&Value>, end: Option<&Value>) -> Value {
    match base {
        Value::String(string) => {
            let characters: Vec<char> = string.chars().collect();
            let length = characters.len();
            let from = start
                .map(|value| builtins::slice_index(value.to_number(), length))
                .unwrap_or(0);
            let to = end
                .map(|value| builtins::slice_index(value.to_number(), length))
                .unwrap_or(length);

            Value::String(characters[from.min(to)..to].iter().collect())
        }

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
