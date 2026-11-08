use std::sync::Arc;

use super::ListSource;
use super::aggregate::{concat_items, join_items};
use super::index::{list_at, position_number, slice_items};
use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::lang::ast::{Expr, Function};
use crate::number;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

pub(super) fn expect_function(predicate: Option<Value>, label: &str) -> Result<Arc<Function>> {
    match predicate {
        Some(Value::Function(function)) => Ok(function),
        _ => Err(Error::type_error(format!("{label} expects a function"))),
    }
}

pub(super) fn optional_selector(
    predicate: Option<Value>,
    query: &str,
) -> Result<Option<Arc<Function>>> {
    match predicate {
        None => Ok(None),
        Some(Value::Function(function)) => Ok(Some(function)),
        Some(_) => Err(Error::type_error(format!("{query} expects a function"))),
    }
}

impl Runtime {
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
            "sum" | "average" | "median" | "max" | "min" | "distinct" | "mode" => {
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

    pub(super) fn select_items(
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
    pub(super) fn sort_items(
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
    pub(super) fn write_back(
        &mut self,
        target: &Expr,
        value: Value,
        scope: &mut Scope,
    ) -> Result<()> {
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
                    self.write_property(&id, property, value)?;
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
