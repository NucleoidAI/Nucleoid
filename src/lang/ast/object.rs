//! Object literals, and the structural comparison objects are matched by.
//! Comparison uses the runtime state to resolve object identities.

mod equality;

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::error::{Error, Result};
use crate::graph::{NodeKey, ShapeKey};
use crate::lang::ast::{Expr, ObjectEntry};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectData, ObjectId, Value};

#[derive(Debug)]
pub struct Object<'a> {
    pub node: &'a Expr,
}

impl<'a> Object<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Object { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        match self.node {
            Expr::ObjectLiteral(entries) => runtime.evaluate_object_literal(entries, scope),
            Expr::SpreadObject(entries) => runtime.evaluate_spread_object(entries, scope),
            other => Err(Error::type_error(format!("{other} is not an object"))),
        }
    }
}

impl Runtime {
    pub(crate) fn evaluate_object_literal(
        &mut self,
        entries: &[(String, Expr)],
        scope: &mut Scope,
    ) -> Result<Value> {
        let mut properties = IndexMap::new();

        for (key, value) in entries {
            properties.insert(key.clone(), self.evaluate(value, scope)?);
        }

        Ok(self.create_anonymous(properties))
    }

    fn evaluate_spread_object(
        &mut self,
        entries: &[ObjectEntry],
        scope: &mut Scope,
    ) -> Result<Value> {
        let keyed: HashMap<&str, usize> = entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                ObjectEntry::Property(key, _) => Some((key.as_str(), index)),
                ObjectEntry::Spread(_) => None,
            })
            .collect();
        let mut properties = IndexMap::new();

        for (index, entry) in entries.iter().enumerate() {
            match entry {
                ObjectEntry::Property(key, value) => {
                    properties.insert(key.clone(), self.evaluate(value, scope)?);
                }
                ObjectEntry::Spread(source) => {
                    let value = self.evaluate(source, scope)?;
                    let replaced = |key: &str| keyed.get(key).is_some_and(|&last| last > index);
                    self.spread_entries(source, &value, &mut properties, replaced)?;
                }
            }
        }

        Ok(self.create_anonymous(properties))
    }

    fn spread_entries(
        &mut self,
        source: &Expr,
        value: &Value,
        properties: &mut IndexMap<String, Value>,
        replaced: impl Fn(&str) -> bool,
    ) -> Result<()> {
        let id = match value {
            Value::Object(id) => id,
            Value::Null | Value::Undefined => return Ok(()),
            Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::List(_)
            | Value::Class(_)
            | Value::Date(_)
            | Value::Regex(_)
            | Value::Function(_) => {
                return Err(Error::type_error(format!("{source} is not an object")));
            }
        };

        self.track_shape(&ShapeKey::Entries(id.clone()));

        let entries: Vec<(String, Value)> = self
            .state
            .object(id)
            .map(|object| {
                object
                    .properties
                    .iter()
                    .map(|(key, entry)| (key.clone(), entry.clone()))
                    .collect()
            })
            .unwrap_or_default();

        for (key, entry) in entries {
            let slot = NodeKey::property(id, &key);
            if self.declares(&slot) {
                continue;
            }
            if !replaced(&key) {
                self.track(&slot);
            }
            if !entry.is_undefined() {
                properties.insert(key, entry);
            }
        }

        Ok(())
    }

    /// Creates an object that belongs to no class, used for object literals and
    /// `Object()`. Such objects are skipped by `for ... of`.
    pub(crate) fn create_anonymous(&mut self, properties: IndexMap<String, Value>) -> Value {
        let id = ObjectId::anonymous();
        let mut data = ObjectData::new(None);
        data.properties = properties;

        self.insert_object(id.clone(), data);

        Value::Object(id)
    }
}

#[cfg(test)]
mod tests;
