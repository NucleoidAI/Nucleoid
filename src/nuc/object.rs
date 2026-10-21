//! Instance creation, constructor execution and class-level rule application.
//!
//! Class calls create instances through this module, whether used in an
//! assignment or evaluated as expressions.

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{ConstructorBinding, Expr, Parameter, Stmt};
use crate::lang::evaluation::TrackingMode;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::state::ClassData;
use crate::value::{ObjectData, ObjectId, Value};

#[derive(Debug, Clone)]
pub struct Object {
    pub id: ObjectId,
    pub class: String,
    pub arguments: Vec<Expr>,
}

/// The parts of a class a constructor needs, lifted out so creating an instance
/// does not copy the list of instances the class already has.
pub(crate) struct ClassShape {
    name: String,
    parameters: Vec<Parameter>,
    constructor: Vec<Stmt>,
}

/// Legacy calls select the constructor after evaluating their expressions,
/// exactly as before. Named calls select and validate a signature first, and
/// keep that same signature/body if an RHS re-declares the class.
pub(crate) enum ConstructorCall {
    Positional(Vec<Value>),
    Named {
        constructor: ClassShape,
        values: Vec<Value>,
    },
}

impl ClassShape {
    fn of(class: &ClassData) -> Self {
        ClassShape {
            name: class.name.clone(),
            parameters: class.parameters.clone(),
            constructor: class.constructor.clone(),
        }
    }
}

impl Object {
    pub fn new(id: ObjectId, class: String, arguments: Vec<Expr>) -> Self {
        Object {
            id,
            class,
            arguments,
        }
    }

    pub fn key(&self) -> NodeKey {
        NodeKey::object(&self.id)
    }

    pub fn run(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        if !runtime.state.has_class(&self.class) {
            return Err(Error::not_defined(&self.class));
        }

        let arguments = runtime.prepare_constructor(&self.class, &self.arguments, scope)?;

        let mut data = ObjectData::new(Some(self.class.clone()));
        data.properties
            .insert("id".to_string(), Value::String(self.id.to_string()));
        runtime.insert_object(self.id.clone(), data);

        let id = self.id.clone();
        runtime.update_class(&self.class, |class| {
            if !class.instances.contains(&id) {
                class.instances.push(id);
            }
        });

        self.graph(runtime)?;

        runtime.run_constructor(&self.class, arguments, &self.id)?;

        for declaration in runtime.declarations_for(&self.class) {
            runtime.apply_declaration(&declaration.statement, &self.id)?;
        }

        self.after(runtime)?;

        Ok(Value::Object(self.id.clone()))
    }

    /// Files the instance so the class's own node can reach it.
    fn graph(&self, runtime: &mut Runtime) -> Result<()> {
        runtime.file(&self.key(), NodeKind::Object, None, IndexSet::new(), None)
    }

    /// Wakes everything that reads the class, since it now has one more
    /// instance.
    fn after(&self, runtime: &mut Runtime) -> Result<()> {
        runtime.propagate(&NodeKey::class(&self.class))
    }
}

impl Runtime {
    /// Follow precisely the existing nearest-nonempty-body rule, even for a
    /// bodyless class that declares parameters. Binding mode travels with the
    /// selected constructor, not with the originally called subclass.
    fn effective_constructor(&self, name: &str) -> Result<&ClassData> {
        let mut class = self
            .state
            .class(name)
            .ok_or_else(|| Error::not_defined(name))?;
        while class.constructor.is_empty() {
            let Some(parent) = class
                .parent
                .as_ref()
                .and_then(|name| self.state.class(name))
            else {
                break;
            };
            class = parent;
        }
        Ok(class)
    }

    pub(crate) fn constructor_binding(&self, name: &str) -> Option<ConstructorBinding> {
        self.effective_constructor(name)
            .ok()
            .map(|class| class.constructor_binding)
    }

    /// Assignment instantiation deliberately retains its existing routing,
    /// which differs from ordinary calls when a function/local shadows a class.
    pub(crate) fn instantiation_binding(&self, expression: &Expr) -> ConstructorBinding {
        if let Expr::Call { callee, .. } = expression {
            if let Expr::Identifier(name) = callee.as_ref() {
                return self.constructor_binding(name).unwrap_or_default();
            }
        }
        ConstructorBinding::Positional
    }

    pub(crate) fn call_constructor_binding(
        &self,
        expression: &Expr,
        scope: &Scope,
    ) -> ConstructorBinding {
        match expression {
            Expr::Call { callee, .. } => {
                if let Expr::Identifier(name) = callee.as_ref() {
                    if !matches!(
                        name.as_str(),
                        "assert" | "ReferenceError" | "TypeError" | "SyntaxError"
                    ) && !scope.has(name)
                        && !self.state.has_function(name)
                    {
                        return self.constructor_binding(name).unwrap_or_default();
                    }
                }
            }
            Expr::Super(_) => {
                if let Ok((_, Some(parent))) = self.super_target(scope) {
                    return self.constructor_binding(&parent).unwrap_or_default();
                }
            }
            _ => {}
        }
        ConstructorBinding::Positional
    }

    pub(crate) fn prepare_constructor(
        &mut self,
        name: &str,
        arguments: &[Expr],
        scope: &mut Scope,
    ) -> Result<ConstructorCall> {
        let class = self.effective_constructor(name)?;
        if class.constructor_binding == ConstructorBinding::Positional {
            return self
                .evaluate_all(arguments, scope)
                .map(ConstructorCall::Positional);
        }

        let mut seen = IndexSet::new();
        let mut bindings = Vec::with_capacity(arguments.len());
        for argument in arguments {
            let Expr::Assign { target, value } = argument else {
                return Err(Error::type_error(format!(
                    "Constructor {} requires named arguments",
                    class.name
                )));
            };
            let Expr::Identifier(name) = target.as_ref() else {
                return Err(Error::type_error(format!(
                    "Constructor {} requires named arguments",
                    class.name
                )));
            };
            let Some(index) = class
                .parameters
                .iter()
                .position(|parameter| parameter.name == *name)
            else {
                return Err(Error::type_error(format!(
                    "Unknown constructor argument '{name}' for {}",
                    class.name
                )));
            };
            if !seen.insert(index) {
                return Err(Error::type_error(format!(
                    "Duplicate constructor argument '{name}' for {}",
                    class.name
                )));
            }
            bindings.push((index, value.as_ref()));
        }

        // Invalid bindings need no owned snapshot, but valid calls must pin
        // the selected signature/body before evaluating any RHS.
        let constructor = ClassShape::of(class);
        let mut values = vec![Value::Null; constructor.parameters.len()];
        for (index, expression) in bindings {
            values[index] = self.evaluate(expression, scope)?;
        }
        Ok(ConstructorCall::Named {
            constructor,
            values,
        })
    }

    pub(crate) fn run_constructor(
        &mut self,
        name: &str,
        call: ConstructorCall,
        this: &ObjectId,
    ) -> Result<()> {
        let (class, arguments) = match call {
            ConstructorCall::Positional(values) => {
                (ClassShape::of(self.effective_constructor(name)?), values)
            }
            ConstructorCall::Named {
                constructor,
                values,
            } => (constructor, values),
        };

        if class.constructor.is_empty() && class.parameters.is_empty() {
            return Ok(());
        }

        let mut scope = Scope::new();
        scope.set_this(Some(this.clone()));
        scope.set_constructor_class(class.name);

        let mut arguments = arguments.into_iter();
        for parameter in class.parameters {
            let value = arguments.next().unwrap_or(Value::Null);
            scope.declare(parameter.name, value);
        }

        self.with_tracking(TrackingMode::Isolated, |runtime| {
            runtime.execute_all(&class.constructor, &mut scope)
        })?;

        Ok(())
    }
}
