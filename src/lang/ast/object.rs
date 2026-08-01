//! Object literals, and the structural comparison objects are matched by.
//! Mirrors `ref/src/lang/ast/Object.js`; the comparison stands in for
//! `ref/src/lib/deep.js`, which has no module of its own here because it needs
//! the state to resolve object identities.

use indexmap::IndexMap;

use crate::error::Result;
use crate::lang::ast::Expr;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectData, ObjectId, Value};

pub struct Object<'a> {
    pub node: &'a Expr,
}

impl<'a> Object<'a> {
    pub fn new(node: &'a Expr) -> Self {
        Object { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        let Expr::ObjectLiteral(entries) = self.node else {
            unreachable!("Object only wraps Expr::ObjectLiteral")
        };

        runtime.evaluate_object_literal(entries, scope)
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

    /// Creates an object that belongs to no class, used for object literals and
    /// `Object()`. Such objects are skipped by `for ... of`.
    pub(crate) fn create_anonymous(&mut self, properties: IndexMap<String, Value>) -> Value {
        let id = ObjectId::anonymous();
        let mut data = ObjectData::new(None);
        data.properties = properties;

        self.insert_object(id.clone(), data);

        Value::Object(id)
    }

    /// Deep equality, which is what `assert` compares with. Objects match on
    /// their contents, and `null` matches a property that was never set.
    pub(crate) fn deep_equal(&self, left: &Value, right: &Value) -> bool {
        self.equal_within(left, right, 0)
    }

    fn equal_within(&self, left: &Value, right: &Value, depth: usize) -> bool {
        if depth > 64 {
            return false;
        }

        match (left, right) {
            (Value::Undefined | Value::Null, Value::Undefined | Value::Null) => true,

            (Value::Object(left), Value::Object(right)) => {
                if left == right {
                    return true;
                }

                let (Some(left), Some(right)) = (self.state.object(left), self.state.object(right))
                else {
                    return false;
                };

                let left_keys = defined_keys(left);
                let right_keys = defined_keys(right);

                if left_keys.len() != right_keys.len() {
                    return false;
                }

                left_keys.iter().all(|key| {
                    match (left.properties.get(*key), right.properties.get(*key)) {
                        (Some(left), Some(right)) => self.equal_within(left, right, depth + 1),
                        _ => false,
                    }
                })
            }

            (Value::List(left), Value::List(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right.iter())
                        .all(|(left, right)| self.equal_within(left, right, depth + 1))
            }

            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Date(left), Value::Date(right)) => left == right,
            (Value::Class(left), Value::Class(right)) => left == right,

            (Value::Date(millis), Value::Number(number))
            | (Value::Number(number), Value::Date(millis)) => *millis as f64 == *number,

            _ => false,
        }
    }
}

/// The property names an object actually holds a value for.
fn defined_keys(object: &ObjectData) -> Vec<&String> {
    object
        .properties
        .iter()
        .filter(|(_, value)| !value.is_undefined())
        .map(|(name, _)| name)
        .collect()
}
