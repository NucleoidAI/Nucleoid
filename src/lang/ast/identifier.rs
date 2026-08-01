//! Names and paths. Mirrors `ref/src/lang/ast/Identifier.js`, which — as here —
//! covers both a bare identifier and a member expression, since the two are the
//! same kind of read at different depths.

use crate::builtins;
use crate::builtins::Global;
use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::lang::ast::Expr;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

pub struct Identifier<'a> {
    pub node: &'a Expr,
}

impl<'a> Identifier<'a> {
    pub fn new(node: &'a Expr) -> Self {
        Identifier { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        match self.node {
            Expr::Identifier(name) => runtime.read_identifier(name, scope),
            Expr::ClassRef(name) => runtime.read_class_ref(name, scope),
            Expr::ObjectRef(id) => Ok(Value::Object(ObjectId::from(id.clone()))),
            Expr::This => runtime.read_this(scope),
            Expr::Member { object, property } => runtime.read_member(object, property, scope),
            other => unreachable!("{other} is not an identifier"),
        }
    }
}

impl Runtime {
    pub(crate) fn read_identifier(&mut self, name: &str, scope: &mut Scope) -> Result<Value> {
        if let Some(value) = scope.retrieve(name) {
            let value = value.clone();
            self.note_nullish(&value);
            return Ok(value);
        }

        if let Some(value) = self.state.variable(name).cloned() {
            self.track(NodeKey::variable(name));
            self.note_nullish(&value);
            return Ok(value);
        }

        if self.state.has_class(name) {
            return Ok(Value::Class(name.to_string()));
        }

        if let Some(function) = self.state.function(name).cloned() {
            return Ok(Value::Function(function));
        }

        if builtins::is_global(name) {
            return Ok(Value::Class(name.to_string()));
        }

        let key = NodeKey::variable(name);

        if self.deleted.contains(&key) {
            self.track(key);
            self.undefined_read = true;
            return Ok(Value::Undefined);
        }

        Err(Error::not_defined(name))
    }

    pub(crate) fn read_class_ref(&mut self, name: &str, scope: &mut Scope) -> Result<Value> {
        match scope.instance() {
            Some(instance) => Ok(Value::Object(instance.clone())),
            None => {
                if self.state.has_class(name) {
                    Ok(Value::Class(name.to_string()))
                } else {
                    Err(Error::not_defined(name))
                }
            }
        }
    }

    pub(crate) fn read_this(&mut self, scope: &mut Scope) -> Result<Value> {
        match scope.this().or_else(|| scope.instance()) {
            Some(id) => Ok(Value::Object(id.clone())),
            None => Err(Error::reference("this is not defined")),
        }
    }

    pub(crate) fn read_member(
        &mut self,
        object: &Expr,
        property: &str,
        scope: &mut Scope,
    ) -> Result<Value> {
        if property == "value" {
            return self.read_value(object, scope);
        }

        let base = self.evaluate(object, scope)?;
        self.read_property(&base, property, scope)
    }

    pub(crate) fn read_property(
        &mut self,
        base: &Value,
        property: &str,
        scope: &mut Scope,
    ) -> Result<Value> {
        match base {
            Value::Object(id) => {
                self.track(NodeKey::property(id, property));

                let value = self
                    .state
                    .property(id, property)
                    .cloned()
                    .unwrap_or(Value::Undefined);

                self.note_nullish(&value);

                if value.is_undefined() {
                    self.undefined_read = true;
                }

                Ok(value)
            }

            Value::Class(name) => self.read_class_property(name, property, scope),

            Value::String(string) => Ok(match property {
                "length" => Value::Number(string.chars().count() as f64),
                _ => Value::Undefined,
            }),

            Value::List(items) => Ok(match property {
                "length" => Value::Number(items.len() as f64),
                _ => Value::Undefined,
            }),

            Value::Undefined | Value::Null => Ok(Value::Undefined),

            _ => Ok(Value::Undefined),
        }
    }

    fn read_class_property(
        &mut self,
        name: &str,
        property: &str,
        _scope: &mut Scope,
    ) -> Result<Value> {
        match Global::from_name(name) {
            Some(Global::Class) if property == "length" => {
                return Ok(Value::Number(self.state.class_count() as f64));
            }
            Some(Global::Number) => {
                if let Some(value) = builtins::number_constant(property) {
                    return Ok(value);
                }
            }
            Some(Global::Math) => {
                if let Some(value) = builtins::math_constant(property) {
                    return Ok(value);
                }
            }
            _ => {}
        }

        if let Some(class) = self.state.class(name) {
            if property == "length" {
                let length = class.instances.len();
                self.track(NodeKey::class(name));
                return Ok(Value::Number(length as f64));
            }
        }

        Ok(Value::Undefined)
    }

    /// `x.value` reads the current value without recording a dependency. The
    /// path must already be defined.
    fn read_value(&mut self, object: &Expr, scope: &mut Scope) -> Result<Value> {
        let saved = self.null_read;
        self.push_tracking(true);
        let value = self.evaluate(object, scope);
        self.pop_tracking();
        self.null_read = saved;

        let value = value?;

        if value.is_undefined() {
            return Err(Error::not_defined(
                object.path().unwrap_or_else(|| object.to_string()),
            ));
        }

        Ok(value)
    }

    /// An assignment used as an expression, which writes a local or a property
    /// without filing a standing declaration.
    pub(crate) fn assign_expression(
        &mut self,
        target: &Expr,
        value: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let evaluated = self.evaluate(value, scope)?;

        match target {
            Expr::Identifier(name) => {
                if !scope.assign(name, evaluated.clone()) {
                    scope.declare(name.clone(), evaluated.clone());
                }
                Ok(evaluated)
            }

            Expr::Member { object, property } => {
                let base = self.evaluate(object, scope)?;

                if let Value::Object(id) = base {
                    self.assign_property(&id, property, evaluated.clone());
                    let key = NodeKey::property(&id, property);
                    self.propagate(&key)?;
                }

                Ok(evaluated)
            }

            other => Err(Error::syntax(format!("Cannot assign to {other}"))),
        }
    }
}
