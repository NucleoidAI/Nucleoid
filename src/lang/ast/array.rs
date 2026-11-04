//! Lists: literals, indexing, slicing and list operations.

use std::cmp::Ordering;
use std::sync::Arc;

use crate::builtins;
use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::lang::ast::operator::compare;
use crate::lang::ast::{Expr, Function};
use crate::number;
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

    pub(crate) fn evaluate_slice(
        &mut self,
        object: &Expr,
        start: Option<&Expr>,
        end: Option<&Expr>,
        scope: &mut Scope,
    ) -> Result<Value> {
        // A class's instances are read before the bounds, as `Name.slice()`
        // reads them before its arguments, and sliced like a list.
        let base = match self.evaluate(object, scope)? {
            Value::Class(name) => match self.population(&name) {
                Some(instances) => Value::List(instances),
                None => Value::Class(name),
            },
            base => base,
        };

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

    pub(crate) fn evaluate_index(
        &mut self,
        object: &Expr,
        index: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let base = self.evaluate(object, scope)?;
        let key = self.evaluate(index, scope)?;
        Ok(self.read_index(&base, &key))
    }

    pub(crate) fn read_index(&mut self, base: &Value, key: &Value) -> Value {
        match base {
            Value::List(items) => {
                let index = key.to_number();
                let index = builtins::slice_index(index, items.len());
                items.get(index).cloned().unwrap_or(Value::Undefined)
            }

            Value::String(string) => {
                let index = builtins::slice_index(key.to_number(), string.chars().count());
                string
                    .chars()
                    .nth(index)
                    .map_or(Value::Undefined, |character| {
                        Value::String(character.to_string())
                    })
            }

            Value::Class(name) => {
                let Some(class) = self.state.class(name) else {
                    return Value::Undefined;
                };

                let instances = class.instances.clone();
                self.track(&NodeKey::class(name));

                let found = match key {
                    Value::Number(index) => instances.get(number::to_usize(*index)).cloned(),
                    other => {
                        let wanted = other.to_string();
                        instances
                            .iter()
                            .find(|instance| instance.as_str() == wanted)
                            .cloned()
                    }
                };

                match found {
                    Some(id) => Value::Object(id),
                    None => Value::Null,
                }
            }

            Value::Object(_) => self.read_property(base, &key.to_string()),

            _ => Value::Undefined,
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

    /// The read-only list operations, shared by lists and class instance lists.
    pub(crate) fn list_query(
        &mut self,
        items: Vec<Value>,
        property: &str,
        arguments: &[Value],
        receiver: &str,
        source: ListSource,
    ) -> Result<Value> {
        let predicate = arguments.first().cloned();

        match property {
            "find" | "findLast" => {
                let function = expect_function(predicate, property)?;
                let found = self.find_position(&function, &items, property == "findLast")?;
                Ok(found
                    .and_then(|position| items.into_iter().nth(position))
                    .unwrap_or(Value::Null))
            }
            "findIndex" | "findLastIndex" => {
                let function = expect_function(predicate, property)?;
                let found = self.find_position(&function, &items, property == "findLastIndex")?;
                Ok(position_number(found))
            }
            "at" => Ok(list_at(items, predicate.as_ref(), source)),
            // A class's instance list as a list value. A list already is one,
            // so plain lists keep raising `<list>.list is not a function`.
            "list" | "all" if source == ListSource::Population => Ok(Value::List(items)),
            "filter" => self.filter_items(items, &expect_function(predicate, "filter")?),
            "count" => self.count_items(&items, &expect_function(predicate, "count")?),
            "map" => self.map_items(items, &expect_function(predicate, "map")?, false),
            "flatMap" => self.map_items(items, &expect_function(predicate, "flatMap")?, true),
            "includes" | "contains" | "indexOf" | "lastIndexOf" => {
                Ok(self.search_items(&items, property, predicate))
            }
            "join" => Ok(join_items(&items, predicate)),
            "every" => {
                let function = expect_function(predicate, "every")?;
                Ok(Value::Bool(!self.any_item_is(&items, &function, false)?))
            }
            "some" | "any" => {
                let function = expect_function(predicate, "some")?;
                Ok(Value::Bool(self.any_item_is(&items, &function, true)?))
            }
            "reduce" => {
                let function = expect_function(predicate, "reduce")?;
                self.reduce_items(items, &function, arguments.get(1))
            }
            "sum" | "max" | "min" | "distinct" | "mode" => {
                self.aggregate_items(items, property, predicate)
            }
            "maxBy" | "minBy" => self.extreme_item(items, property, predicate),
            // Lists sort in place through `call_list_method`; a class population
            // has no stored list to write, so its sort only returns the order.
            "sort" => Ok(Value::List(self.sort_items(items, predicate.as_ref())?)),
            "slice" => Ok(Value::List(slice_items(
                &items,
                arguments.first(),
                arguments.get(1),
            ))),
            "reverse" => {
                let mut reversed = items;
                reversed.reverse();
                Ok(Value::List(reversed))
            }
            "concat" => Ok(Value::List(concat_items(items, arguments))),
            "length" => Ok(Value::Number(number::from_usize(items.len()))),
            _ => Err(Error::type_error(format!(
                "{receiver}.{property} is not a function"
            ))),
        }
    }

    fn filter_items(&mut self, items: Vec<Value>, function: &Arc<Function>) -> Result<Value> {
        let mut matched = Vec::new();

        for item in items {
            if self.invoke(function, std::slice::from_ref(&item))?.truthy() {
                matched.push(item);
            }
        }

        Ok(Value::List(matched))
    }

    fn count_items(&mut self, items: &[Value], function: &Arc<Function>) -> Result<Value> {
        let mut matched = 0_usize;

        for item in items {
            if self.invoke(function, std::slice::from_ref(item))?.truthy() {
                matched += 1;
            }
        }

        Ok(Value::Number(number::from_usize(matched)))
    }

    fn map_items(
        &mut self,
        items: Vec<Value>,
        function: &Arc<Function>,
        flatten: bool,
    ) -> Result<Value> {
        let mut mapped = Vec::new();

        for item in items {
            match self.invoke(function, &[item])? {
                Value::List(values) if flatten => mapped.extend(values),
                value => mapped.push(value),
            }
        }

        Ok(Value::List(mapped))
    }

    fn search_items(&self, items: &[Value], property: &str, predicate: Option<Value>) -> Value {
        let wanted = predicate.unwrap_or(Value::Undefined);
        let matches = |item: &Value| self.deep_equal(item, &wanted);

        match property {
            "indexOf" => position_number(items.iter().position(matches)),
            "lastIndexOf" => position_number(items.iter().rposition(matches)),
            _ => Value::Bool(items.iter().any(matches)),
        }
    }

    fn any_item_is(
        &mut self,
        items: &[Value],
        function: &Arc<Function>,
        wanted: bool,
    ) -> Result<bool> {
        for item in items {
            if self.invoke(function, std::slice::from_ref(item))?.truthy() == wanted {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn reduce_items(
        &mut self,
        items: Vec<Value>,
        function: &Arc<Function>,
        initial: Option<&Value>,
    ) -> Result<Value> {
        let mut items = items.into_iter();

        let mut accumulator = match initial {
            Some(initial) => initial.clone(),
            None => match items.next() {
                Some(first) => first,
                None => return Ok(Value::Undefined),
            },
        };

        for item in items {
            accumulator = self.invoke(function, &[accumulator, item])?;
        }

        Ok(accumulator)
    }

    fn aggregate_items(
        &mut self,
        items: Vec<Value>,
        property: &str,
        predicate: Option<Value>,
    ) -> Result<Value> {
        let selector = optional_selector(predicate, property)?;

        if property == "sum" {
            return self.sum_items(items, selector.as_ref());
        }

        let values = self.select_items(items, selector.as_ref())?;

        Ok(match property {
            "distinct" => Value::List(self.distinct_values(values)),
            "mode" => self.mode_value(values),
            _ => builtins::math(property, &values)?,
        })
    }

    fn extreme_item(
        &mut self,
        items: Vec<Value>,
        property: &str,
        predicate: Option<Value>,
    ) -> Result<Value> {
        let selector = expect_function(predicate, property)?;
        let wanted = if property == "maxBy" {
            Ordering::Greater
        } else {
            Ordering::Less
        };
        let mut extreme: Option<(Value, Value)> = None;
        let mut unordered = false;

        for item in items {
            let key = self.invoke(&selector, std::slice::from_ref(&item))?;

            if compare(&key, &key).is_none() {
                unordered = true;
            } else if extreme
                .as_ref()
                .is_none_or(|(_, kept)| compare(&key, kept) == Some(wanted))
            {
                extreme = Some((item, key));
            }
        }

        Ok(match extreme {
            Some((item, _)) if !unordered => item,
            _ => Value::Undefined,
        })
    }

    /// Calls `function` with each item, first to last or last to first, and
    /// returns the position of the first item it accepts. Later items are
    /// not visited, so their reads are not dependencies.
    fn find_position(
        &mut self,
        function: &Arc<Function>,
        items: &[Value],
        from_end: bool,
    ) -> Result<Option<usize>> {
        let candidates = items.iter().enumerate();

        if from_end {
            self.first_accepted(function, candidates.rev())
        } else {
            self.first_accepted(function, candidates)
        }
    }

    fn first_accepted<'v>(
        &mut self,
        function: &Arc<Function>,
        candidates: impl Iterator<Item = (usize, &'v Value)>,
    ) -> Result<Option<usize>> {
        for (position, item) in candidates {
            if self.invoke(function, std::slice::from_ref(item))?.truthy() {
                return Ok(Some(position));
            }
        }

        Ok(None)
    }
    fn sum_items(&mut self, items: Vec<Value>, selector: Option<&Arc<Function>>) -> Result<Value> {
        let mut total = 0.0;
        let mut undefined = false;

        for item in items {
            let value = match selector {
                Some(function) => self.invoke(function, &[item])?,
                None => item,
            };
            undefined |= value.is_undefined();
            total += value.to_number();
        }

        Ok(if undefined {
            Value::Undefined
        } else {
            Value::Number(total)
        })
    }

    fn select_items(
        &mut self,
        items: Vec<Value>,
        selector: Option<&Arc<Function>>,
    ) -> Result<Vec<Value>> {
        let Some(function) = selector else {
            return Ok(items);
        };

        items
            .into_iter()
            .map(|item| self.invoke(function, &[item]))
            .collect()
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
            items.sort_by_cached_key(ToString::to_string);
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

fn expect_function(predicate: Option<Value>, label: &str) -> Result<Arc<Function>> {
    match predicate {
        Some(Value::Function(function)) => Ok(function),
        _ => Err(Error::type_error(format!("{label} expects a function"))),
    }
}

fn list_at(items: Vec<Value>, index: Option<&Value>, source: ListSource) -> Value {
    let index = index.map_or(f64::NAN, Value::to_number);
    relative_index(index, items.len())
        .and_then(|position| items.into_iter().nth(position))
        .unwrap_or_else(|| source.missing())
}

fn join_items(items: &[Value], separator: Option<Value>) -> Value {
    let separator = separator.map(|value| value.to_string()).unwrap_or_default();
    let rendered: Vec<String> = items.iter().map(ToString::to_string).collect();
    Value::String(rendered.join(&separator))
}

fn concat_items(items: Vec<Value>, arguments: &[Value]) -> Vec<Value> {
    let mut joined = items;

    for argument in arguments {
        match argument {
            Value::List(other) => joined.extend(other.iter().cloned()),
            other => joined.push(other.clone()),
        }
    }

    joined
}

fn optional_selector(predicate: Option<Value>, query: &str) -> Result<Option<Arc<Function>>> {
    match predicate {
        None => Ok(None),
        Some(Value::Function(function)) => Ok(Some(function)),
        Some(_) => Err(Error::type_error(format!("{query} expects a function"))),
    }
}

/// A found position as a number, or `-1` when nothing was found.
fn position_number(position: Option<usize>) -> Value {
    Value::Number(position.map_or(-1.0, number::from_usize))
}

/// Resolves the index of `at` the way JavaScript does: the number is
/// truncated, NaN reads as zero and a negative index counts from the end.
/// Unlike bracket indexing, an index outside the list is never clamped into
/// it, so `None` means there is no such position.
fn relative_index(index: f64, length: usize) -> Option<usize> {
    let index = if index.is_nan() { 0.0 } else { index.trunc() };
    let position = if index < 0.0 {
        index + number::from_usize(length)
    } else {
        index
    };

    (0.0..number::from_usize(length))
        .contains(&position)
        .then_some(number::to_usize(position))
}

#[must_use]
pub fn slice(base: &Value, start: Option<&Value>, end: Option<&Value>) -> Value {
    match base {
        Value::String(string) => Value::String(builtins::string_slice(
            string,
            start.map_or(0.0, Value::to_number),
            end.map(Value::to_number),
        )),

        Value::List(items) => Value::List(slice_items(items, start, end)),

        _ => Value::Undefined,
    }
}

/// The items from `start` up to `end`, with the bound rules that ranges and
/// `slice` share: an omitted start or end is the start or the end of the list,
/// a negative bound counts from the end, every bound is clamped to the list,
/// and a start at or after the end gives no items.
fn slice_items(items: &[Value], start: Option<&Value>, end: Option<&Value>) -> Vec<Value> {
    let length = items.len();
    let bound = |value: &Value| builtins::slice_index(value.to_number(), length);
    let start = start.map_or(0, bound);
    let end = end.map_or(length, bound);

    items
        .get(start.min(end)..end)
        .map(<[Value]>::to_vec)
        .unwrap_or_default()
}
