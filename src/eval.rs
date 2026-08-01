use indexmap::IndexMap;
use regex::Regex;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::ast::{BinaryOp, Expr, Function, FunctionBody, LogicalOp, TemplatePart, UnaryOp};
use crate::builtins;
use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::runtime::{AssertionFailure, Flow, MAX_DEPTH, Runtime};
use crate::scope::Scope;
use crate::value::{ObjectData, ObjectId, Value};

impl Runtime {
    pub(crate) fn evaluate(&mut self, expression: &Expr, scope: &mut Scope) -> Result<Value> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(Error::type_error("Maximum expression depth exceeded"));
        }

        let result = self.evaluate_inner(expression, scope);
        self.depth -= 1;
        result
    }

    fn evaluate_inner(&mut self, expression: &Expr, scope: &mut Scope) -> Result<Value> {
        match expression {
            Expr::Null => Ok(Value::Null),
            Expr::Bool(bool) => Ok(Value::Bool(*bool)),
            Expr::Number(number) => Ok(Value::Number(*number)),
            Expr::String(string) => Ok(Value::String(string.clone())),

            Expr::Regex(pattern) => match Regex::new(pattern) {
                Ok(regex) => Ok(Value::Regex(Arc::new(regex))),
                Err(error) => Err(Error::syntax(format!(
                    "Invalid regular expression: {error}"
                ))),
            },

            Expr::Template(parts) => {
                let mut output = String::new();

                for part in parts {
                    match part {
                        TemplatePart::Literal(literal) => output.push_str(literal),
                        TemplatePart::Expression(expression) => {
                            let value = self.evaluate(expression, scope)?;
                            output.push_str(&value.to_string());
                        }
                    }
                }

                Ok(Value::String(output))
            }

            Expr::List(items) => {
                let mut values = Vec::with_capacity(items.len());

                for item in items {
                    values.push(self.evaluate(item, scope)?);
                }

                Ok(Value::List(values))
            }

            Expr::ObjectLiteral(entries) => {
                let mut properties = IndexMap::new();

                for (key, value) in entries {
                    properties.insert(key.clone(), self.evaluate(value, scope)?);
                }

                Ok(self.create_anonymous(properties))
            }

            Expr::Function(function) => Ok(Value::Function(function.clone())),

            Expr::Identifier(name) => self.read_identifier(name, scope),

            Expr::ClassRef(name) => match scope.instance() {
                Some(instance) => Ok(Value::Object(instance.clone())),
                None => {
                    if self.state.classes.contains_key(name) {
                        Ok(Value::Class(name.clone()))
                    } else {
                        Err(Error::not_defined(name))
                    }
                }
            },

            Expr::ObjectRef(id) => Ok(Value::Object(ObjectId::from(id.clone()))),

            Expr::This => match scope.this().or_else(|| scope.instance()) {
                Some(id) => Ok(Value::Object(id.clone())),
                None => Err(Error::reference("this is not defined")),
            },

            Expr::Super(arguments) => {
                let mut values = Vec::new();
                for argument in arguments {
                    values.push(self.evaluate(argument, scope)?);
                }

                let Some(this) = scope.this().cloned() else {
                    return Err(Error::reference("super is not defined"));
                };

                let parent = self
                    .state
                    .object(&this)
                    .and_then(|object| object.class.clone())
                    .and_then(|class| {
                        self.state
                            .class(&class)
                            .and_then(|data| data.parent.clone())
                    });

                if let Some(parent) = parent {
                    if let Some(class) = self.state.class(&parent).cloned() {
                        self.run_super(&class, &values, &this)?;
                    }
                }

                Ok(Value::Null)
            }

            Expr::Member { object, property } => {
                if property == "value" {
                    return self.read_value(object, scope);
                }

                let base = self.evaluate(object, scope)?;
                self.read_property(&base, property, scope)
            }

            Expr::Index { object, index } => {
                let base = self.evaluate(object, scope)?;
                let key = self.evaluate(index, scope)?;
                self.read_index(&base, &key)
            }

            Expr::Slice { object, start, end } => {
                let base = self.evaluate(object, scope)?;

                let start = match start {
                    Some(start) => Some(self.evaluate(start, scope)?),
                    None => None,
                };
                let end = match end {
                    Some(end) => Some(self.evaluate(end, scope)?),
                    None => None,
                };

                Ok(slice(&base, start.as_ref(), end.as_ref()))
            }

            Expr::Call { callee, arguments } => self.call(callee, arguments, scope),

            Expr::Unary { operator, operand } => {
                if *operator == UnaryOp::TypeOf {
                    // `$Person` is the class itself; `Person` is its instances.
                    if let Expr::ClassRef(_) = operand.as_ref() {
                        return Ok(Value::Class("Class".to_string()));
                    }

                    let value = self.evaluate(operand, scope)?;
                    return Ok(type_of(&value));
                }

                let value = self.evaluate(operand, scope)?;

                Ok(match operator {
                    UnaryOp::Not => Value::Bool(!value.truthy()),
                    UnaryOp::Negate => Value::Number(-value.to_number()),
                    UnaryOp::Plus => Value::Number(value.to_number()),
                    UnaryOp::TypeOf => unreachable!("handled above"),
                })
            }

            Expr::Logical {
                operator,
                left,
                right,
            } => {
                let left = self.evaluate(left, scope)?;

                match operator {
                    LogicalOp::And => {
                        if left.truthy() {
                            self.evaluate(right, scope)
                        } else {
                            Ok(left)
                        }
                    }
                    LogicalOp::Or => {
                        if left.truthy() {
                            Ok(left)
                        } else {
                            self.evaluate(right, scope)
                        }
                    }
                }
            }

            Expr::Binary {
                operator,
                left,
                right,
            } => {
                let left = self.evaluate(left, scope)?;
                let right = self.evaluate(right, scope)?;
                binary(*operator, &left, &right)
            }

            Expr::Assign { target, value } => self.assign_expression(target, value, scope),

            Expr::Delete(operand) => self.delete(operand, scope),
        }
    }

    // -- reads ---------------------------------------------------------------

    fn read_identifier(&mut self, name: &str, scope: &mut Scope) -> Result<Value> {
        if let Some(value) = scope.get(name) {
            let value = value.clone();
            self.note_nullish(&value);
            return Ok(value);
        }

        if let Some(value) = self.state.variables.get(name).cloned() {
            self.track(NodeKey::new(name.to_string()));
            self.note_nullish(&value);
            return Ok(value);
        }

        if self.state.classes.contains_key(name) {
            return Ok(Value::Class(name.to_string()));
        }

        if let Some(function) = self.state.functions.get(name).cloned() {
            return Ok(Value::Function(function));
        }

        if builtins::is_global(name) {
            return Ok(Value::Class(name.to_string()));
        }

        let key = NodeKey::new(name.to_string());

        if self.deleted.contains(&key) {
            self.track(key);
            self.undefined_read = true;
            return Ok(Value::Undefined);
        }

        Err(Error::not_defined(name))
    }

    fn read_property(&mut self, base: &Value, property: &str, scope: &mut Scope) -> Result<Value> {
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
        if name == "Class" && property == "length" {
            return Ok(Value::Number(self.state.classes.len() as f64));
        }

        if name == "Number" {
            if let Some(value) = builtins::number_constant(property) {
                return Ok(value);
            }
        }

        if name == "Math" {
            if let Some(value) = builtins::math_constant(property) {
                return Ok(value);
            }
        }

        if let Some(class) = self.state.class(name) {
            if property == "length" {
                let length = class.instances.len();
                self.track(NodeKey::new(format!("${name}")));
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

    fn read_index(&mut self, base: &Value, key: &Value) -> Result<Value> {
        match base {
            Value::List(items) => {
                let index = key.to_number();
                let index = builtins::slice_index(index, items.len());
                Ok(items.get(index).cloned().unwrap_or(Value::Undefined))
            }

            Value::String(string) => {
                let characters: Vec<char> = string.chars().collect();
                let index = builtins::slice_index(key.to_number(), characters.len());
                Ok(characters
                    .get(index)
                    .map(|character| Value::String(character.to_string()))
                    .unwrap_or(Value::Undefined))
            }

            Value::Class(name) => {
                let Some(class) = self.state.class(name) else {
                    return Ok(Value::Undefined);
                };

                let instances = class.instances.clone();
                self.track(NodeKey::new(format!("${name}")));

                let found = match key {
                    Value::Number(index) => instances.get(*index as usize).cloned(),
                    other => {
                        let wanted = other.to_string();
                        instances
                            .iter()
                            .find(|instance| instance.as_str() == wanted)
                            .cloned()
                    }
                };

                Ok(match found {
                    Some(id) => Value::Object(id),
                    None => Value::Null,
                })
            }

            Value::Object(id) => {
                let property = key.to_string();
                self.track(NodeKey::property(id, &property));
                Ok(self
                    .state
                    .property(id, &property)
                    .cloned()
                    .unwrap_or(Value::Undefined))
            }

            _ => Ok(Value::Undefined),
        }
    }

    // -- calls ---------------------------------------------------------------

    fn call(&mut self, callee: &Expr, arguments: &[Expr], scope: &mut Scope) -> Result<Value> {
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

        if let Some(Value::Function(function)) = scope.get(name).cloned() {
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
            return self.create_instance(name, arguments, id, scope);
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
                return date_from(&values);
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
                        return date_static(property, &values);
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
                date_method(millis, property, &values)
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
            return date_static(property, &values);
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

    fn call_list_method(
        &mut self,
        object: &Expr,
        mut items: Vec<Value>,
        property: &str,
        arguments: &[Expr],
        scope: &mut Scope,
    ) -> Result<Value> {
        match property {
            "push" | "append" => {
                let values = self.evaluate_all(arguments, scope)?;
                items.extend(values);
                let length = items.len();
                self.write_back(object, Value::List(items), scope)?;
                Ok(Value::Number(length as f64))
            }

            "pop" => {
                let popped = items.pop().unwrap_or(Value::Undefined);
                self.write_back(object, Value::List(items), scope)?;
                Ok(popped)
            }

            "shift" => {
                let removed = if items.is_empty() {
                    Value::Undefined
                } else {
                    items.remove(0)
                };
                self.write_back(object, Value::List(items), scope)?;
                Ok(removed)
            }

            _ => {
                let values = self.evaluate_all(arguments, scope)?;
                let receiver = object.path().unwrap_or_else(|| object.to_string());
                self.list_query(items, property, &values, &receiver)
            }
        }
    }

    /// The read-only list operations, shared by lists and class instance lists.
    fn list_query(
        &mut self,
        items: Vec<Value>,
        property: &str,
        arguments: &[Value],
        receiver: &str,
    ) -> Result<Value> {
        let predicate = arguments.first().cloned();

        match property {
            "find" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("find expects a function"));
                };

                for item in items {
                    if self
                        .invoke(&function, std::slice::from_ref(&item))?
                        .truthy()
                    {
                        return Ok(item);
                    }
                }

                Ok(Value::Null)
            }

            "filter" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("filter expects a function"));
                };

                let mut matched = Vec::new();

                for item in items {
                    if self
                        .invoke(&function, std::slice::from_ref(&item))?
                        .truthy()
                    {
                        matched.push(item);
                    }
                }

                Ok(Value::List(matched))
            }

            "map" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("map expects a function"));
                };

                let mut mapped = Vec::new();

                for item in items {
                    mapped.push(self.invoke(&function, &[item])?);
                }

                Ok(Value::List(mapped))
            }

            "includes" | "contains" => {
                let wanted = predicate.unwrap_or(Value::Undefined);
                Ok(Value::Bool(
                    items.iter().any(|item| self.deep_equal(item, &wanted)),
                ))
            }

            "indexOf" => {
                let wanted = predicate.unwrap_or(Value::Undefined);
                let found = items.iter().position(|item| self.deep_equal(item, &wanted));
                Ok(Value::Number(
                    found.map(|index| index as f64).unwrap_or(-1.0),
                ))
            }

            "join" => {
                let separator = predicate.map(|value| value.to_string()).unwrap_or_default();
                let rendered: Vec<String> = items.iter().map(|item| item.to_string()).collect();
                Ok(Value::String(rendered.join(&separator)))
            }

            "every" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("every expects a function"));
                };

                for item in items {
                    if !self
                        .invoke(&function, std::slice::from_ref(&item))?
                        .truthy()
                    {
                        return Ok(Value::Bool(false));
                    }
                }

                Ok(Value::Bool(true))
            }

            "some" | "any" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("some expects a function"));
                };

                for item in items {
                    if self
                        .invoke(&function, std::slice::from_ref(&item))?
                        .truthy()
                    {
                        return Ok(Value::Bool(true));
                    }
                }

                Ok(Value::Bool(false))
            }

            "reduce" => {
                let Some(Value::Function(function)) = predicate else {
                    return Err(Error::type_error("reduce expects a function"));
                };

                let mut items = items.into_iter();

                let mut accumulator = match arguments.get(1) {
                    Some(initial) => initial.clone(),
                    None => match items.next() {
                        Some(first) => first,
                        None => return Ok(Value::Undefined),
                    },
                };

                for item in items {
                    accumulator = self.invoke(&function, &[accumulator, item])?;
                }

                Ok(accumulator)
            }

            "sort" => {
                let mut sorted = items;

                match predicate {
                    Some(Value::Function(function)) => {
                        // A comparator sort, kept simple with insertion order.
                        let mut ordered: Vec<Value> = Vec::with_capacity(sorted.len());

                        for item in sorted {
                            let mut index = ordered.len();

                            for (position, placed) in ordered.iter().enumerate() {
                                let comparison =
                                    self.invoke(&function, &[item.clone(), placed.clone()])?;

                                if comparison.to_number() < 0.0 {
                                    index = position;
                                    break;
                                }
                            }

                            ordered.insert(index, item);
                        }

                        Ok(Value::List(ordered))
                    }
                    _ => {
                        sorted.sort_by_key(|item| item.to_string());
                        Ok(Value::List(sorted))
                    }
                }
            }

            "slice" => {
                let length = items.len();
                let start = arguments
                    .first()
                    .map(|value| builtins::slice_index(value.to_number(), length))
                    .unwrap_or(0);
                let end = arguments
                    .get(1)
                    .map(|value| builtins::slice_index(value.to_number(), length))
                    .unwrap_or(length);

                Ok(Value::List(items[start.min(end)..end].to_vec()))
            }

            "reverse" => {
                let mut reversed = items;
                reversed.reverse();
                Ok(Value::List(reversed))
            }

            "concat" => {
                let mut joined = items;

                for argument in arguments {
                    match argument {
                        Value::List(other) => joined.extend(other.iter().cloned()),
                        other => joined.push(other.clone()),
                    }
                }

                Ok(Value::List(joined))
            }

            "length" => Ok(Value::Number(items.len() as f64)),

            _ => Err(Error::type_error(format!(
                "{receiver}.{property} is not a function"
            ))),
        }
    }

    /// Stores a mutated list back where it came from and wakes its dependents.
    fn write_back(&mut self, target: &Expr, value: Value, scope: &mut Scope) -> Result<()> {
        match target {
            Expr::Identifier(name) => {
                if scope.assign(name, value.clone()) {
                    return Ok(());
                }

                self.set_variable(name, value);
                let key = NodeKey::new(name.clone());
                self.propagate(&key)?;
                Ok(())
            }

            Expr::Member { object, property } => {
                let base = self.evaluate(object, scope)?;

                if let Value::Object(id) = base {
                    self.set_property(&id, property, value);
                    let key = NodeKey::property(&id, property);
                    self.propagate(&key)?;
                }

                Ok(())
            }

            _ => Ok(()),
        }
    }

    pub(crate) fn invoke(
        &mut self,
        function: &Arc<Function>,
        arguments: &[Value],
    ) -> Result<Value> {
        self.invoke_with_this(function, arguments, None)
    }

    fn invoke_with_this(
        &mut self,
        function: &Arc<Function>,
        arguments: &[Value],
        this: Option<ObjectId>,
    ) -> Result<Value> {
        let mut scope = Scope::new();
        // A second frame keeps function locals out of the state.
        scope.push();
        scope.set_this(this);
        // A function called from a class-level rule keeps seeing that instance,
        // so `$Class` inside a predicate means the same thing as outside it.
        scope.set_instance(self.instances.last().cloned());

        for (index, parameter) in function.parameters.iter().enumerate() {
            let value = arguments.get(index).cloned().unwrap_or(Value::Null);
            scope.declare(parameter.name.clone(), value);
        }

        match &function.body {
            FunctionBody::Expression(expression) => self.evaluate(expression, &mut scope),
            FunctionBody::Block(statements) => {
                let flow = self.execute_all(statements, &mut scope)?;

                Ok(match flow {
                    Flow::Return(value) => value,
                    Flow::Normal(_) => Value::Null,
                })
            }
        }
    }

    fn run_super(
        &mut self,
        class: &crate::state::ClassData,
        arguments: &[Value],
        this: &ObjectId,
    ) -> Result<()> {
        let mut scope = Scope::new();
        scope.push();
        scope.set_this(Some(this.clone()));

        for (index, parameter) in class.parameters.iter().enumerate() {
            let value = arguments.get(index).cloned().unwrap_or(Value::Null);
            scope.declare(parameter.name.clone(), value);
        }

        let statements = class.constructor.clone();
        self.execute_all(&statements, &mut scope)?;
        Ok(())
    }

    fn assign_expression(
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
                    self.set_property(&id, property, evaluated.clone());
                    let key = NodeKey::property(&id, property);
                    self.propagate(&key)?;
                }

                Ok(evaluated)
            }

            other => Err(Error::syntax(format!("Cannot assign to {other}"))),
        }
    }

    pub(crate) fn evaluate_all(
        &mut self,
        expressions: &[Expr],
        scope: &mut Scope,
    ) -> Result<Vec<Value>> {
        let mut values = Vec::with_capacity(expressions.len());

        for expression in expressions {
            values.push(self.evaluate(expression, scope)?);
        }

        Ok(values)
    }

    /// Creates an object that belongs to no class, used for object literals and
    /// `Object()`. Such objects are skipped by `for ... of`.
    fn create_anonymous(&mut self, properties: IndexMap<String, Value>) -> Value {
        let id = ObjectId::from(next_id());
        let mut data = ObjectData::new(None);
        data.properties = properties;

        self.transaction.record_object(&id, None);
        self.state.objects.insert(id.clone(), data);

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

/// A fresh identifier for an object created without a variable name.
fn next_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("obj:{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn type_of(value: &Value) -> Value {
    match value {
        Value::Object(_) => Value::Class("Object".to_string()),
        Value::Class(_) => Value::Class("List".to_string()),
        Value::List(_) => Value::Class("List".to_string()),
        Value::Function(_) => Value::Class("Function".to_string()),
        Value::Number(_) => Value::String("number".to_string()),
        Value::String(_) => Value::String("string".to_string()),
        Value::Bool(_) => Value::String("boolean".to_string()),
        Value::Date(_) => Value::Class("Date".to_string()),
        Value::Regex(_) => Value::Class("RegExp".to_string()),
        Value::Null => Value::String("null".to_string()),
        Value::Undefined => Value::String("undefined".to_string()),
    }
}

fn slice(base: &Value, start: Option<&Value>, end: Option<&Value>) -> Value {
    match base {
        Value::String(string) => {
            let characters: Vec<char> = string.chars().collect();
            let length = characters.len();
            let from = start
                .map(|value| builtins::slice_index(value.to_number(), length))
                .unwrap_or(0);
            let to = end
                .map(|value| builtins::slice_index(value.to_number(), length))
                .unwrap_or(length);

            Value::String(characters[from.min(to)..to].iter().collect())
        }

        Value::List(items) => {
            let length = items.len();
            let from = start
                .map(|value| builtins::slice_index(value.to_number(), length))
                .unwrap_or(0);
            let to = end
                .map(|value| builtins::slice_index(value.to_number(), length))
                .unwrap_or(length);

            Value::List(items[from.min(to)..to].to_vec())
        }

        _ => Value::Undefined,
    }
}

fn date_from(arguments: &[Value]) -> Result<Value> {
    match arguments.first() {
        None => Ok(Value::Date(builtins::date::now())),
        Some(Value::Number(millis)) => Ok(Value::Date(*millis as i64)),
        Some(Value::Date(millis)) => Ok(Value::Date(*millis)),
        Some(Value::String(text)) => match builtins::date::parse(text) {
            Some(millis) => Ok(Value::Date(millis)),
            None => Err(Error::type_error(format!("Invalid date '{text}'"))),
        },
        Some(other) => Err(Error::type_error(format!(
            "Cannot create a date from {}",
            other.type_name()
        ))),
    }
}

fn date_static(name: &str, arguments: &[Value]) -> Result<Value> {
    match name {
        "now" => Ok(Value::Number(builtins::date::now() as f64)),
        "parse" => {
            let text = arguments
                .first()
                .cloned()
                .unwrap_or(Value::Undefined)
                .to_string();

            Ok(match builtins::date::parse(&text) {
                Some(millis) => Value::Number(millis as f64),
                None => Value::Number(f64::NAN),
            })
        }
        "UTC" => Ok(Value::Number(builtins::date::now() as f64)),
        _ => Err(Error::type_error(format!("Date.{name} is not a function"))),
    }
}

fn date_method(millis: i64, name: &str, _arguments: &[Value]) -> Result<Value> {
    Ok(match name {
        "getTime" | "valueOf" => Value::Number(millis as f64),
        "getYear" => Value::Number(builtins::date::get_year(millis) as f64),
        "getFullYear" => Value::Number(builtins::date::get_full_year(millis) as f64),
        "getMonth" => Value::Number(builtins::date::get_month(millis) as f64),
        "getDate" => Value::Number(builtins::date::get_date(millis) as f64),
        "getDay" => Value::Number(builtins::date::get_day(millis) as f64),
        "getHours" => Value::Number(builtins::date::get_hours(millis) as f64),
        "getMinutes" => Value::Number(builtins::date::get_minutes(millis) as f64),
        "getSeconds" => Value::Number(builtins::date::get_seconds(millis) as f64),
        "toDateString" => Value::String(builtins::date::to_date_string(millis)),
        "toISOString" | "toJSON" => Value::String(builtins::date::to_iso_string(millis)),
        "toString" => Value::String(builtins::date::to_string(millis)),
        _ => {
            return Err(Error::type_error(format!("Date.{name} is not a function")));
        }
    })
}

fn binary(operator: BinaryOp, left: &Value, right: &Value) -> Result<Value> {
    use BinaryOp::*;

    // A read of something undefined makes the whole expression undefined, which
    // an assignment then stores as null.
    if matches!(operator, Add | Subtract | Multiply | Divide | Modulo)
        && (left.is_undefined() || right.is_undefined())
    {
        return Ok(Value::Undefined);
    }

    Ok(match operator {
        Add => match (left, right) {
            (Value::String(_), _) | (_, Value::String(_)) => {
                Value::String(format!("{left}{right}"))
            }
            _ => Value::Number(left.to_number() + right.to_number()),
        },
        Subtract => Value::Number(left.to_number() - right.to_number()),
        Multiply => Value::Number(left.to_number() * right.to_number()),
        Divide => Value::Number(left.to_number() / right.to_number()),
        Modulo => Value::Number(left.to_number() % right.to_number()),

        Equal => Value::Bool(loose_equal(left, right)),
        NotEqual => Value::Bool(!loose_equal(left, right)),
        StrictEqual => Value::Bool(left == right),
        StrictNotEqual => Value::Bool(left != right),

        Less | LessEqual | Greater | GreaterEqual => {
            let ordering = match (left, right) {
                (Value::String(left), Value::String(right)) => {
                    Some(left.as_str().cmp(right.as_str()))
                }
                _ => left.to_number().partial_cmp(&right.to_number()),
            };

            let Some(ordering) = ordering else {
                return Ok(Value::Bool(false));
            };

            Value::Bool(match operator {
                Less => ordering.is_lt(),
                LessEqual => ordering.is_le(),
                Greater => ordering.is_gt(),
                GreaterEqual => ordering.is_ge(),
                _ => unreachable!("only comparisons reach here"),
            })
        }
    })
}

fn loose_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Null | Value::Undefined, Value::Null | Value::Undefined) => true,
        (Value::Null | Value::Undefined, _) | (_, Value::Null | Value::Undefined) => false,
        (Value::String(left), Value::String(right)) => left == right,
        (Value::Object(left), Value::Object(right)) => left == right,
        (Value::Class(left), Value::Class(right)) => left == right,
        (Value::List(left), Value::List(right)) => left == right,
        (Value::Bool(left), Value::Bool(right)) => left == right,
        _ => left.to_number() == right.to_number(),
    }
}
