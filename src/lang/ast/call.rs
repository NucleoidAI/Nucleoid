//! Function, method and constructor calls.

use indexmap::IndexMap;

use crate::builtins;
use crate::builtins::Global;
use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::lang::ast::Expr;
use crate::lang::ast::array::ListSource;
use crate::nuc::object::Object;
use crate::runtime::{AssertionFailure, Runtime};
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

#[derive(Debug)]
pub struct Call<'a> {
    pub node: &'a Expr,
}

impl<'a> Call<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Call { node }
    }

    /// The callee expression.
    #[must_use]
    pub fn function(&self) -> Option<&'a Expr> {
        match self.node {
            Expr::Call { callee, .. } => Some(callee),
            _ => None,
        }
    }

    #[must_use]
    pub fn arguments(&self) -> &'a [Expr] {
        match self.node {
            Expr::Call { arguments, .. } | Expr::Super(arguments) => arguments,
            _ => &[],
        }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        match self.node {
            Expr::Call { callee, arguments } => runtime.call(callee, arguments, scope),
            Expr::Super(arguments) => runtime.call_super(arguments, scope),
            // Exactly the route of the method call `OPERAND.NOUN()`.
            Expr::Of { noun, operand } => runtime.call_method(operand, noun.as_str(), &[], scope),
            other => Err(Error::type_error(format!("{other} is not a call"))),
        }
    }
}

impl Runtime {
    pub(crate) fn call(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
        scope: &mut Scope,
    ) -> Result<Value> {
        if let Expr::Identifier(name) = callee {
            return self.call_named(name, arguments, scope);
        }

        if let Expr::Member { object, property } = callee {
            return self.call_method(object, property, arguments, scope);
        }

        let value = self.evaluate(callee, scope)?;

        self.call_value(callee, value, arguments, scope)
    }

    fn call_value(
        &mut self,
        callee: impl std::fmt::Display,
        value: Value,
        arguments: &[Expr],
        scope: &mut Scope,
    ) -> Result<Value> {
        match value {
            Value::Function(function) => {
                let values = self.evaluate_all(arguments, scope)?;
                self.invoke(&function, &values)
            }
            Value::Null | Value::Undefined => {
                self.note_nullish(&value);
                self.undefined_read |= value.is_undefined();
                Ok(Value::Undefined)
            }
            _ => Err(Error::type_error(format!("{callee} is not a function"))),
        }
    }

    fn call_named(&mut self, name: &str, arguments: &[Expr], scope: &mut Scope) -> Result<Value> {
        if name == "assert" {
            let values = self.evaluate_all(arguments, scope)?;
            self.assertions_run += 1;
            let actual = values.first().cloned().unwrap_or(Value::Undefined);
            let expected = values.get(1).cloned().unwrap_or(Value::Undefined);

            if !self.deep_equal(&actual, &expected) {
                self.assertions.push(AssertionFailure { actual, expected });
            }

            return Ok(Value::Null);
        }

        if matches!(name, "ReferenceError" | "TypeError" | "SyntaxError") {
            let values = self.evaluate_all(arguments, scope)?;
            let message = values
                .first()
                .cloned()
                .unwrap_or(Value::String(String::new()))
                .to_string();
            return Ok(Value::String(format!("{name}: {message}")));
        }

        if let Some(value) = scope.retrieve(name).cloned() {
            return self.call_value(name, value, arguments, scope);
        }

        if let Some(function) = self.state.function(name).cloned() {
            self.track(&NodeKey::function(name));
            let values = self.evaluate_all(arguments, scope)?;
            return self.invoke(&function, &values);
        }

        if self.state.has_class(name) {
            let id = ObjectId::anonymous();
            let object = Object::new(id, name.to_string(), arguments.to_vec());
            return object.run(self, scope);
        }

        if let Some(value) = self.construct_global(name, arguments, scope)? {
            return Ok(value);
        }

        if let Some(value) = self.state.variable(name).cloned() {
            self.track(&NodeKey::variable(name));
            return self.call_value(name, value, arguments, scope);
        }

        let value = self.read_missing_identifier(name)?;
        self.call_value(name, value, arguments, scope)
    }

    fn construct_global(
        &mut self,
        name: &str,
        arguments: &[Expr],
        scope: &mut Scope,
    ) -> Result<Option<Value>> {
        Ok(Some(match Global::from_callee(name) {
            Some(Global::Object) => self.create_anonymous(IndexMap::new()),
            Some(Global::Boolean) => Value::Bool(self.first_argument(arguments, scope)?.truthy()),
            Some(Global::String) => {
                Value::String(self.first_argument(arguments, scope)?.to_string())
            }
            Some(Global::Number) => {
                Value::Number(self.first_argument(arguments, scope)?.to_number())
            }
            Some(Global::Date) => builtins::date::construct(&self.evaluate_all(arguments, scope)?)?,
            Some(Global::List) => Value::List(self.evaluate_all(arguments, scope)?),
            _ => return Ok(None),
        }))
    }

    fn first_argument(&mut self, arguments: &[Expr], scope: &mut Scope) -> Result<Value> {
        let values = self.evaluate_all(arguments, scope)?;
        Ok(values.into_iter().next().unwrap_or(Value::Undefined))
    }

    fn call_method(
        &mut self,
        object: &Expr,
        property: &str,
        arguments: &[Expr],
        scope: &mut Scope,
    ) -> Result<Value> {
        // Static calls on the built-in namespaces.
        if let Expr::Identifier(name) = object {
            if !scope.has(name) && !self.state.has_variable(name) {
                if let Some(global) = Global::from_name(name) {
                    if let Some(value) = self.call_global(global, property, arguments, scope)? {
                        return Ok(value);
                    }
                }
            }
        }

        let (receiver, target) = if object.version_view() {
            let (value, target) = self.version_list_receiver(object, scope)?;
            (value, Some(target))
        } else if property == "sort" {
            self.sort_receiver(object, scope)?
        } else {
            (self.evaluate(object, scope)?, None)
        };

        match receiver {
            Value::Class(name) => self.call_class_method(&name, property, arguments, scope),

            Value::List(items) => {
                self.call_list_method(object, items, property, arguments, scope, target)
            }

            Value::String(string) => {
                let values = self.evaluate_all(arguments, scope)?;
                builtins::string_method(&string, property, &values)
            }

            Value::Date(millis) => {
                let values = self.evaluate_all(arguments, scope)?;
                builtins::date::method(millis, property, &values)
            }

            Value::Regex(regex) => {
                let values = self.evaluate_all(arguments, scope)?;
                let subject = values.first().cloned().unwrap_or(Value::Undefined);

                match property {
                    "test" => Ok(Value::Bool(regex.is_match(&subject.to_string()))),
                    _ => Err(Error::type_error(format!("{property} is not a function"))),
                }
            }

            Value::Object(id) => self.call_object_method(&id, property, arguments, scope),

            // A method on something not yet defined leaves the expression
            // undefined, so the assignment it feeds stores null and re-runs
            // once the receiver exists.
            Value::Undefined | Value::Null => Ok(Value::Undefined),

            other => Err(Error::type_error(format!(
                "{}.{property} is not a function",
                other.type_name()
            ))),
        }
    }

    fn call_object_method(
        &mut self,
        id: &ObjectId,
        property: &str,
        arguments: &[Expr],
        scope: &mut Scope,
    ) -> Result<Value> {
        let method = self
            .state
            .object(id)
            .and_then(|object| object.class.clone())
            .and_then(|class| self.state.class(&class).cloned())
            .and_then(|class| class.methods.get(property).cloned());

        if let Some(method) = method {
            let values = self.evaluate_all(arguments, scope)?;
            return self.invoke_with_this(&method, &values, Some(id.clone()));
        }

        if property == "toString" && self.state.property(id, property).is_none() {
            self.evaluate_all(arguments, scope)?;
            return Ok(Value::String(id.to_string()));
        }

        let value = self.read_property(&Value::Object(id.clone()), property);
        self.call_value(format_args!("{id}.{property}"), value, arguments, scope)
    }

    fn call_class_method(
        &mut self,
        name: &str,
        property: &str,
        arguments: &[Expr],
        scope: &mut Scope,
    ) -> Result<Value> {
        if let Some(global) = Global::from_name(name) {
            if let Some(value) = self.call_global(global, property, arguments, scope)? {
                return Ok(value);
            }
        }

        let Some(instances) = self.population(name) else {
            return Err(Error::not_defined(name));
        };

        let values = self.evaluate_all(arguments, scope)?;
        self.list_query(instances, property, &values, name, ListSource::Population)
    }

    /// A static call on a built-in namespace: `Math.max(...)`, `Date.now()`.
    /// `None` when the namespace has no statics, so the caller can carry on.
    fn call_global(
        &mut self,
        global: Global,
        property: &str,
        arguments: &[Expr],
        scope: &mut Scope,
    ) -> Result<Option<Value>> {
        Ok(Some(match global {
            Global::Math => builtins::math(property, &self.evaluate_all(arguments, scope)?)?,
            Global::Number => builtins::number(property, &self.evaluate_all(arguments, scope)?)?,
            Global::String => builtins::string(property, &self.evaluate_all(arguments, scope)?)?,
            Global::Date => {
                builtins::date::statics(property, &self.evaluate_all(arguments, scope)?)?
            }
            _ => return Ok(None),
        }))
    }
}
