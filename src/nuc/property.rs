//! `person.age = 30` — a declaration filed under `<object id>.<property>`.
//! Mirrors `ref/src/nuc/PROPERTY.js`; the `$INSTANCE` case in `ref` is the
//! branch here that reaches an instance through a class-level rule.

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{Expr, Stmt};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectData, ObjectId, Value};

impl Runtime {
    pub(crate) fn assign_property(
        &mut self,
        object: ObjectId,
        property: String,
        value: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        if property == "value" {
            return Err(Error::type_error("Cannot use 'value' as a property"));
        }

        let key = NodeKey::property(&object, &property);
        let value = &self.freeze(value, scope)?;
        let statement = Stmt::Assign {
            target: Expr::Member {
                object: Box::new(Expr::ObjectRef(object.to_string())),
                property: property.clone(),
            },
            value: value.clone(),
        };

        if self.is_imperative() && self.instantiation(value).is_none() {
            let (evaluated, _) = self.evaluate_tracked(value, scope, Some(&key))?;
            self.set_property(&object, &property, evaluated.clone());
            self.propagate(&key)?;
            return Ok(evaluated);
        }

        if let Some((class, arguments)) = self.instantiation(value) {
            let id = ObjectId::from(format!("{object}.{property}"));
            let created = self.create_instance(&class, &arguments, id, scope)?;
            self.register(
                &key,
                NodeKind::Object,
                Some(statement),
                IndexSet::new(),
                Some(object.clone()),
            )?;
            self.set_property(&object, &property, created.clone());
            self.propagate(&key)?;
            return Ok(created);
        }

        let (evaluated, dependencies) = self.evaluate_tracked(value, scope, Some(&key))?;

        self.register(
            &key,
            NodeKind::Property,
            Some(statement),
            dependencies,
            Some(object.clone()),
        )?;
        self.set_property(&object, &property, evaluated.clone());
        self.propagate(&key)?;

        Ok(evaluated)
    }

    pub(crate) fn set_property(&mut self, object: &ObjectId, property: &str, value: Value) {
        if !self.state.objects.contains_key(object) {
            self.transaction.record_object(object, None);
            self.state
                .objects
                .insert(object.clone(), ObjectData::new(None));
        }

        let before = self.state.property(object, property).cloned();
        self.transaction.record_property(object, property, before);

        if let Some(data) = self.state.object_mut(object) {
            data.properties.insert(property.to_string(), value);
        }
    }
}
