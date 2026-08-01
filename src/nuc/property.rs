//! `PROPERTY` — `person.age = 30`, filed under `<object id>.<property>`, and
//! `$Person.mortal = true`, which states the same thing about every instance of
//! a type. Mirrors `ref/src/nuc/PROPERTY.js`; `ref`'s `PROPERTY$CLASS.js` and
//! `PROPERTY$INSTANCE.js` are the two arms of [`Owner`] here.

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{Expr, Stmt};
use crate::lang::evaluation::Flow;
use crate::nuc::Outcome;
use crate::nuc::object::Object;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::state::DeclarationKey;
use crate::value::ObjectId;

/// What the property is being set on.
pub enum Owner {
    Object(ObjectId),
    /// A type, so the assignment is a rule that holds for every instance.
    Class(String),
}

pub struct Property {
    pub owner: Owner,
    pub name: String,
    pub value: Expr,
    kind: Option<NodeKind>,
}

impl Property {
    pub fn new(owner: Owner, name: String, value: Expr) -> Self {
        Property {
            owner,
            name,
            value,
            kind: None,
        }
    }

    pub fn key(&self) -> Option<NodeKey> {
        match &self.owner {
            Owner::Object(id) => Some(NodeKey::property(id, &self.name)),
            Owner::Class(_) => None,
        }
    }

    /// A rule about a type keeps its expression as written — it is re-run for
    /// every instance, so there is no one value to freeze it to.
    pub fn before(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<()> {
        if let Owner::Object(_) = self.owner {
            self.value = runtime.freeze(&self.value, scope)?;
        }
        Ok(())
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        if self.name == "value" {
            return Err(Error::type_error("Cannot use 'value' as a property"));
        }

        match &self.owner {
            Owner::Class(class) => {
                let class = class.clone();

                // A rule is checked where it is written, not when an instance
                // finally arrives to run it.
                runtime.check_rule_references(&self.value, scope)?;

                let statement = Stmt::Assign {
                    target: Expr::Member {
                        object: Box::new(Expr::ClassRef(class.clone())),
                        property: self.name.clone(),
                    },
                    value: self.value.clone(),
                };

                let key = DeclarationKey::property(&class, &self.name);
                runtime.declare_on_class(&class, key, statement)?;

                Ok(Outcome::null())
            }

            Owner::Object(object) => {
                let object = object.clone();
                let key = NodeKey::property(&object, &self.name);

                if runtime.is_imperative() && runtime.instantiation(&self.value).is_none() {
                    let (evaluated, _) =
                        runtime.evaluate_tracked(&self.value, scope, Some(&key))?;
                    runtime.assign_property(&object, &self.name, evaluated.clone());
                    self.kind = None;
                    return Ok(Outcome::value(evaluated));
                }

                if let Some((class, arguments)) = runtime.instantiation(&self.value) {
                    let id = ObjectId::nested(&object, &self.name);
                    let instance = Object::new(id, class, arguments);
                    let created = instance.run(runtime, scope)?;
                    runtime.assign_property(&object, &self.name, created.clone());
                    self.kind = Some(NodeKind::Object);
                    return Ok(Outcome::value(created));
                }

                let (evaluated, dependencies) =
                    runtime.evaluate_tracked(&self.value, scope, Some(&key))?;
                runtime.assign_property(&object, &self.name, evaluated.clone());
                self.kind = Some(NodeKind::Property);

                Ok(Outcome {
                    flow: Flow::Normal(evaluated),
                    dependencies,
                })
            }
        }
    }

    pub fn graph(&self, runtime: &mut Runtime, dependencies: IndexSet<NodeKey>) -> Result<()> {
        let (Some(kind), Owner::Object(object)) = (self.kind, &self.owner) else {
            return Ok(());
        };

        let statement = Stmt::Assign {
            target: Expr::Member {
                object: Box::new(Expr::ObjectRef(object.to_string())),
                property: self.name.clone(),
            },
            value: self.value.clone(),
        };

        runtime.file(
            &NodeKey::property(object, &self.name),
            kind,
            Some(statement),
            dependencies,
            Some(object.clone()),
        )
    }

    pub fn after(&self, runtime: &mut Runtime) -> Result<()> {
        match self.key() {
            Some(key) => runtime.propagate(&key),
            None => Ok(()),
        }
    }
}
