//! `DELETE` — removing a variable, a property, an object or a class-level rule,
//! and clearing whatever depended on it. Mirrors `ref/src/nuc/DELETE.js`
//! together with `DELETE$VARIABLE.js` and `DELETE$OBJECT.js`, which are the
//! arms of the match in `Runtime::delete`.

use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::lang::ast::Expr;
use crate::nuc::Outcome;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::state::DeclarationKey;
use crate::value::{ObjectId, Value};

#[derive(Debug, Clone)]
pub struct Delete {
    pub target: Expr,
}

impl Delete {
    pub fn new(target: Expr) -> Self {
        Delete { target }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let value = runtime.delete(&self.target, scope)?;
        Ok(Outcome::value(value))
    }
}

impl Runtime {
    /// Shared by the statement above and by `delete` used as an expression.
    pub(crate) fn delete(&mut self, expression: &Expr, scope: &mut Scope) -> Result<Value> {
        match expression {
            Expr::Identifier(name) => {
                let key = NodeKey::variable(name.clone());

                let Some(value) = self.state.variable(name).cloned() else {
                    return Ok(Value::Bool(false));
                };

                if let Value::Object(id) = &value {
                    self.delete_object(id)?;
                }

                self.remove_variable(name);
                self.remove_node(&key);
                self.cascade_removal(&key)?;

                Ok(Value::Bool(true))
            }

            Expr::Member { object, property } => {
                if let Expr::ClassRef(class) = object.as_ref() {
                    if scope.instance().is_none() {
                        return self.delete_declaration(class, property);
                    }
                }

                let base = self.evaluate(object, scope)?;

                let Value::Object(id) = base else {
                    return Ok(Value::Bool(false));
                };

                if self.state.property(&id, property).is_none() {
                    return Ok(Value::Bool(false));
                }

                self.remove_property(&id, property);

                let key = NodeKey::property(&id, property);
                self.remove_node(&key);
                self.cascade_removal(&key)?;

                Ok(Value::Bool(true))
            }

            Expr::Index { .. } => {
                let value = self.evaluate(expression, scope)?;

                let Value::Object(id) = value else {
                    return Ok(Value::Bool(false));
                };

                self.delete_object(&id)?;

                let key = NodeKey::object(&id);
                self.remove_node(&key);

                if self.state.has_variable(id.as_str()) {
                    self.remove_variable(id.as_str());
                }

                self.cascade_removal(&key)?;

                Ok(Value::Bool(true))
            }

            other => Err(Error::syntax(format!("Cannot delete {other}"))),
        }
    }

    /// Removes a class-level rule and the values it produced.
    fn delete_declaration(&mut self, class: &str, property: &str) -> Result<Value> {
        let key = DeclarationKey::property(class, property);

        let Some(data) = self.state.class(class).cloned() else {
            return Err(Error::not_defined(class));
        };

        if !data.declarations.contains_key(&key) {
            return Ok(Value::Bool(false));
        }

        self.update_class(class, |data| {
            data.declarations.shift_remove(&key);
        });

        for instance in data.instances {
            let node = NodeKey::property(&instance, property);
            self.remove_node(&node);

            if self.state.property(&instance, property).is_some() {
                self.assign_property(&instance, property, Value::Null);
            }
        }

        Ok(Value::Bool(true))
    }

    /// Removes an instance, refusing while it still carries properties.
    fn delete_object(&mut self, id: &ObjectId) -> Result<()> {
        let Some(data) = self.state.object(id).cloned() else {
            return Ok(());
        };

        let remaining = data
            .properties
            .iter()
            .filter(|(name, _)| name.as_str() != "id")
            .count();

        if remaining > 0 {
            return Err(Error::type_error(format!("Cannot delete object '{id}'")));
        }

        let data_class = data.class.clone();

        if let Some(class_name) = &data_class {
            self.update_class(class_name, |class| {
                class.instances.retain(|instance| instance != id);
            });
        }

        self.remove_object(id);

        if let Some(class_name) = data_class {
            self.propagate(&NodeKey::class(&class_name))?;
        }

        Ok(())
    }
}
