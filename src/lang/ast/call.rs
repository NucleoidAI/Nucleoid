//! Calls. Mirrors `ref/src/lang/ast/Call.js`.

use indexmap::IndexMap;

use crate::builtins;
use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::lang::ast::Expr;
use crate::lang::ast::object::next_id;
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

        if let Some(function) = self.state.functions.get(name).cloned() {
            self.track(NodeKey::new(name.to_string()));
            let values = self.evaluate_all(arguments, scope)?;
            return self.invoke(&function, &values);
        }

        if self.state.classes.contains_key(name) {
            let id = ObjectId::from(next_id());
            let object = Object::new(id, name.to_string(), arguments.to_vec());
            return object.run(self, scope);
        }

        match name {
            "Object" => {
                let properties = IndexMap::new();
                return Ok(self.create_anonymous(properties));
            }
            "Boolean" => {
                let values = self.evaluate_all(arguments, scope)?;
                let value = values.first().cloned().unwrap_or(Value::Undefined);
                return Ok(Value::Bool(value.truthy()));
            }
            "String" => {
                let values = self.evaluate_all(arguments, scope)?;
                let value = values.first().cloned().unwrap_or(Value::Undefined);
                return Ok(Value::String(value.to_string()));
            }
            "Number" => {
                let values = self.evaluate_all(arguments, scope)?;
                let value = values.first().cloned().unwrap_or(Value::Undefined);
                return Ok(Value::Number(value.to_number()));
            }
            "Date" => {
                let values = self.evaluate_all(arguments, scope)?;
                return builtins::date::construct(&values);
            }
            "List" | "Array" => {
                let values = self.evaluate_all(arguments, scope)?;
                return Ok(Value::List(values));
            }
            _ => {}
        }

        if let Some(Value::Function(function)) = self.state.variables.get(name).cloned() {
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
            if !scope.has(name) && !self.state.variables.contains_key(name) {
                match name.as_str() {
                    "Math" => {
                        let values = self.evaluate_all(arguments, scope)?;
                        return builtins::math(property, &values);
                    }
                    "Number" => {
                        let values = self.evaluate_all(arguments, scope)?;
                        return builtins::number(property, &values);
                    }
                    "String" => {
                        let values = self.evaluate_all(arguments, scope)?;
                        return builtins::string(property, &values);
                    }
                    "Date" => {
                        let values = self.evaluate_all(arguments, scope)?;
                        return builtins::date::statics(property, &values);
                    }
                    _ => {}
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
        if name == "Date" {
            let values = self.evaluate_all(arguments, scope)?;
            return builtins::date::statics(property, &values);
        }

        if name == "Math" {
            let values = self.evaluate_all(arguments, scope)?;
            return builtins::math(property, &values);
        }

        if name == "Number" {
            let values = self.evaluate_all(arguments, scope)?;
            return builtins::number(property, &values);
        }

        if name == "String" {
            let values = self.evaluate_all(arguments, scope)?;
            return builtins::string(property, &values);
        }

        let Some(class) = self.state.class(name).cloned() else {
            return Err(Error::not_defined(name));
        };

        let instances: Vec<Value> = class.instances.iter().cloned().map(Value::Object).collect();
        self.track(NodeKey::new(format!("${name}")));

        let values = self.evaluate_all(arguments, scope)?;
        self.list_query(instances, property, &values, name)
    }
}
