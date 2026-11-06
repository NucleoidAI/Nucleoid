//! Names and paths: bare identifiers and member expressions are the same kind
//! of read at different depths.

use crate::builtins;
use crate::builtins::Global;
use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::Expr;
use crate::lang::ast::function::SELECTOR_PARAMETER;
use crate::lang::evaluation::TrackingMode;
use crate::number;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

#[derive(Debug)]
pub struct Identifier<'a> {
    pub node: &'a Expr,
}

impl<'a> Identifier<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Identifier { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        match self.node {
            Expr::Identifier(name) => runtime.read_identifier(name, scope),
            Expr::ClassRef(name) => runtime.read_class_ref(name, scope),
            Expr::ObjectRef(id) => Ok(Value::Object(ObjectId::from(id.clone()))),
            Expr::This => read_this(scope),
            Expr::Member { object, property } => runtime.read_member(object, property, scope),
            other => Err(Error::type_error(format!("{other} is not an identifier"))),
        }
    }
}

impl Runtime {
    pub(crate) fn read_identifier(&mut self, name: &str, scope: &Scope) -> Result<Value> {
        match self.read_bound(name, scope) {
            Some(value) => Ok(value),
            None => self.read_missing_identifier(name),
        }
    }

    pub(crate) fn read_word(&mut self, word: &str, scope: &Scope) -> Value {
        self.read_bound(word, scope).unwrap_or_else(|| {
            self.track(&NodeKey::variable(word));
            Value::String(word.to_string())
        })
    }

    fn read_bound(&mut self, name: &str, scope: &Scope) -> Option<Value> {
        if let Some(value) = scope.retrieve(name) {
            let value = value.clone();
            self.note_nullish(&value);
            return Some(value);
        }

        if let Some(value) = self.state.variable(name).cloned() {
            self.track(&NodeKey::variable(name));
            self.note_nullish(&value);
            return Some(value);
        }

        if self.state.has_class(name) {
            return Some(Value::Class(name.to_string()));
        }

        if let Some(function) = self.state.function(name).cloned() {
            self.track(&NodeKey::function(name));
            return Some(Value::Function(function));
        }

        builtins::is_global(name).then(|| Value::Class(name.to_string()))
    }

    pub(crate) fn read_missing_identifier(&mut self, name: &str) -> Result<Value> {
        let key = NodeKey::variable(name);
        // Only prior dependents may defer a deleted name in later cascades.
        let deferred = self.deleted.contains(&key)
            || self.stack.is_draining()
                && self.graph.retrieve(&key).is_some_and(|node| {
                    node.kind == NodeKind::Pending
                        && node
                            .dependents
                            .iter()
                            .any(|dependent| self.stack.is_running(dependent))
                });

        if deferred {
            self.track(&key);
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
        Ok(self.read_property(&base, property))
    }

    pub(crate) fn read_property(&mut self, base: &Value, property: &str) -> Value {
        match base {
            Value::Object(id) => {
                self.track(&NodeKey::property(id, property));

                let value = self
                    .state
                    .property(id, property)
                    .cloned()
                    .unwrap_or(Value::Undefined);

                self.note_nullish(&value);

                if value.is_undefined() {
                    self.undefined_read = true;
                }

                value
            }

            Value::Class(name) => self.read_class_property(name, property),

            Value::String(string) => match property {
                "length" => Value::Number(number::from_usize(string.chars().count())),
                _ => Value::Undefined,
            },

            Value::List(items) => match property {
                "length" => Value::Number(number::from_usize(items.len())),
                _ => Value::Undefined,
            },

            _ => Value::Undefined,
        }
    }

    fn read_class_property(&mut self, name: &str, property: &str) -> Value {
        match Global::from_name(name) {
            Some(Global::Class) if property == "length" => {
                self.track(&NodeKey::class("Class"));
                return Value::Number(number::from_usize(self.state.class_count()));
            }
            Some(Global::Number) => {
                if let Some(value) = builtins::number_constant(property) {
                    return value;
                }
            }
            Some(Global::Math) => {
                if let Some(value) = builtins::math_constant(property) {
                    return value;
                }
            }
            _ => {}
        }

        if let Some(class) = self.state.class(name) {
            if property == "length" {
                let length = class.instances.len();
                self.track(&NodeKey::class(name));
                return Value::Number(number::from_usize(length));
            }
        }

        Value::Undefined
    }

    /// `x.value` reads the current value without recording a dependency. The
    /// path must already be defined.
    fn read_value(&mut self, object: &Expr, scope: &mut Scope) -> Result<Value> {
        let saved = self.null_read;
        let value = self.with_tracking(TrackingMode::Isolated, |runtime| {
            runtime.evaluate(object, scope)
        });
        self.null_read = saved;

        let (value, _) = value?;

        if value.is_undefined() {
            let name = match object {
                Expr::Identifier(name) if name == SELECTOR_PARAMETER => ".value".to_string(),
                _ => object.path().unwrap_or_else(|| object.to_string()),
            };
            return Err(Error::not_defined(name));
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
        if matches!(target, Expr::Version { .. }) {
            return Err(Error::type_error("State versions are read-only"));
        }
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
                    let changed = self.assign_property(&id, property, evaluated.clone());
                    let key = NodeKey::property(&id, property);
                    self.propagate_change(&key, changed)?;
                }

                Ok(evaluated)
            }

            other => Err(Error::syntax(format!("Cannot assign to {other}"))),
        }
    }
}

fn read_this(scope: &Scope) -> Result<Value> {
    match scope.this().or_else(|| scope.instance()) {
        Some(id) => Ok(Value::Object(id.clone())),
        None => Err(Error::reference("this is not defined")),
    }
}
