//! Calls. Mirrors `ref/src/lang/ast/Call.js`.

use indexmap::IndexMap;

use crate::builtins;
use crate::builtins::Global;
use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::lang::ast::Expr;
use crate::nuc::object::Object;
use crate::runtime::{AssertionFailure, Runtime};
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

pub struct Call<'a> {
    pub node: &'a Expr,
}

impl<'a> Call<'a> {
    pub fn new(node: &'a Expr) -> Self {
        Call { node }
    }

    /// The name being called — `Call.function` in `ref`.
    pub fn function(&self) -> Option<&'a Expr> {
        match self.node {
            Expr::Call { callee, .. } => Some(callee),
            _ => None,
        }
    }

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
            other => unreachable!("{other} is not a call"),
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

        if let Value::Function(function) = value {
            let values = self.evaluate_all(arguments, scope)?;
            return self.invoke(&function, &values);
        }

        Err(Error::type_error(format!("{callee} is not a function")))
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

        if let Some(Value::Function(function)) = scope.retrieve(name).cloned() {
            let values = self.evaluate_all(arguments, scope)?;
            return self.invoke(&function, &values);
        }

        if let Some(function) = self.state.function(name).cloned() {
            self.track(NodeKey::function(name));
            let values = self.evaluate_all(arguments, scope)?;
            return self.invoke(&function, &values);
        }

        if self.state.has_class(name) {
            let id = ObjectId::anonymous();
            let object = Object::new(id, name.to_string(), arguments.to_vec());
            return object.run(self, scope);
        }

        match Global::from_callee(name) {
            Some(Global::Object) => {
                let properties = IndexMap::new();
                return Ok(self.create_anonymous(properties));
            }
            Some(Global::Boolean) => {
                let values = self.evaluate_all(arguments, scope)?;
                let value = values.first().cloned().unwrap_or(Value::Undefined);
                return Ok(Value::Bool(value.truthy()));
            }
            Some(Global::String) => {
                let values = self.evaluate_all(arguments, scope)?;
                let value = values.first().cloned().unwrap_or(Value::Undefined);
                return Ok(Value::String(value.to_string()));
            }
            Some(Global::Number) => {
                let values = self.evaluate_all(arguments, scope)?;
                let value = values.first().cloned().unwrap_or(Value::Undefined);
                return Ok(Value::Number(value.to_number()));
            }
            Some(Global::Date) => {
                let values = self.evaluate_all(arguments, scope)?;
                return builtins::date::construct(&values);
            }
            Some(Global::List) => {
                let values = self.evaluate_all(arguments, scope)?;
                return Ok(Value::List(values));
            }
            _ => {}
        }

        if let Some(Value::Function(function)) = self.state.variable(name).cloned() {
            let values = self.evaluate_all(arguments, scope)?;
            return self.invoke(&function, &values);
        }

        Err(Error::not_defined(name))
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

        let receiver = self.evaluate(object, scope)?;

        match &receiver {
            Value::Class(name) => {
                let name = name.clone();
                self.call_class_method(&name, property, arguments, scope)
            }

            Value::List(items) => {
                let items = items.clone();
                self.call_list_method(object, items, property, arguments, scope)
            }

            Value::String(string) => {
                let string = string.clone();
                let values = self.evaluate_all(arguments, scope)?;
                builtins::string_method(&string, property, &values)
            }

            Value::Date(millis) => {
                let millis = *millis;
                let values = self.evaluate_all(arguments, scope)?;
                builtins::date::method(millis, property, &values)
            }

            Value::Regex(regex) => {
                let regex = regex.clone();
                let values = self.evaluate_all(arguments, scope)?;
                let subject = values.first().cloned().unwrap_or(Value::Undefined);

                match property {
                    "test" => Ok(Value::Bool(regex.is_match(&subject.to_string()))),
                    _ => Err(Error::type_error(format!("{property} is not a function"))),
                }
            }

            Value::Object(id) => {
                let id = id.clone();
                let values = self.evaluate_all(arguments, scope)?;
                self.call_object_method(&id, property, &values)
            }

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
        arguments: &[Value],
    ) -> Result<Value> {
        let method = self
            .state
            .object(id)
            .and_then(|object| object.class.clone())
            .and_then(|class| self.state.class(&class).cloned())
            .and_then(|class| class.methods.get(property).cloned());

        if let Some(method) = method {
            return self.invoke_with_this(&method, arguments, Some(id.clone()));
        }

        if let Some(Value::Function(function)) = self.state.property(id, property).cloned() {
            return self.invoke(&function, arguments);
        }

        if property == "toString" {
            return Ok(Value::String(id.to_string()));
        }

        Err(Error::type_error(format!(
            "{id}.{property} is not a function"
        )))
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

        let Some(class) = self.state.class(name).cloned() else {
            return Err(Error::not_defined(name));
        };

        let instances: Vec<Value> = class.instances.iter().cloned().map(Value::Object).collect();
        self.track(NodeKey::class(name));

        let values = self.evaluate_all(arguments, scope)?;
        self.list_query(instances, property, &values, name)
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
        let values = match global {
            Global::Math | Global::Number | Global::String | Global::Date => {
                self.evaluate_all(arguments, scope)?
            }
            _ => return Ok(None),
        };

        Ok(Some(match global {
            Global::Math => builtins::math(property, &values)?,
            Global::Number => builtins::number(property, &values)?,
            Global::String => builtins::string(property, &values)?,
            Global::Date => builtins::date::statics(property, &values)?,
            _ => unreachable!("filtered above"),
        }))
    }
}
