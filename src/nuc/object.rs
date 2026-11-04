//! Instance creation, constructor execution and class-level rule application.
//!
//! Class calls create instances through this module, whether used in an
//! assignment or evaluated as expressions.

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{Expr, Parameter, Stmt};
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

/// Purely positional calls retain delayed constructor selection. Calls with
/// labels validate and pin one signature/body before evaluating any value.
pub(crate) enum ConstructorCall {
    Positional(Vec<Value>),
    Bound {
        constructor: ClassShape,
        values: Vec<Option<Value>>,
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

/// A validated source expression and the fixed slot it fills. Surplus
/// positional arguments have no fixed slot; a rest formal collects them.
struct ArgumentBinding<'a> {
    slot: Option<usize>,
    expression: &'a Expr,
}

fn bind_arguments<'a>(
    class: &ClassData,
    arguments: &'a [Expr],
) -> Result<Vec<ArgumentBinding<'a>>> {
    let mut seen = IndexSet::new();
    let mut bindings = Vec::with_capacity(arguments.len());
    let mut positional = 0;
    let mut named = false;
    for argument in arguments {
        if let Some((name, value)) = argument.named_argument() {
            named = true;
            let Some((index, parameter)) = class
                .parameters
                .iter()
                .enumerate()
                .find(|(_, parameter)| parameter.name == name)
            else {
                return Err(Error::type_error(format!(
                    "Unknown constructor argument '{name}' for {}",
                    class.name
                )));
            };
            if parameter.rest {
                return Err(Error::type_error(format!(
                    "Rest parameter '{name}' cannot be a named constructor argument for {}",
                    class.name
                )));
            }
            if !seen.insert(index) {
                return Err(Error::type_error(format!(
                    "Duplicate constructor argument '{name}' for {}",
                    class.name
                )));
            }
            bindings.push(ArgumentBinding {
                slot: Some(index),
                expression: value,
            });
        } else {
            if named {
                return Err(Error::type_error(format!(
                    "Positional constructor argument after named arguments for {}",
                    class.name
                )));
            }
            let slot = class
                .parameters
                .get(positional)
                .filter(|parameter| !parameter.rest)
                .map(|_| positional);
            if let Some(slot) = slot {
                seen.insert(slot);
            }
            bindings.push(ArgumentBinding {
                slot,
                expression: argument,
            });
            positional += 1;
        }
    }
    Ok(bindings)
}

impl Object {
    #[must_use]
    pub fn new(id: ObjectId, class: String, arguments: Vec<Expr>) -> Self {
        Object {
            id,
            class,
            arguments,
        }
    }

    #[must_use]
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
    /// bodyless class that declares parameters. Labels refer to the selected
    /// constructor's signature, not the originally called subclass's header.
    fn effective_constructor(&self, name: &str) -> Result<&ClassData> {
        let mut class = self
            .state
            .class(name)
            .ok_or_else(|| Error::not_defined(name))?;
        for ancestor in self.state.lineage(Some(class)) {
            class = ancestor;
            if !class.constructor.is_empty() {
                break;
            }
        }
        Ok(class)
    }

    /// Assignment instantiation deliberately retains its existing routing,
    /// which differs from ordinary calls when a function/local shadows a class.
    pub(crate) fn is_instantiation(&self, expression: &Expr) -> bool {
        if let Expr::Call { callee, .. } = expression {
            if let Expr::Identifier(name) = callee.as_ref() {
                return self.state.has_class(name);
            }
        }
        false
    }

    pub(crate) fn is_constructor_call(&self, expression: &Expr, scope: &Scope) -> bool {
        match expression {
            Expr::Call { callee, .. } => {
                if let Expr::Identifier(name) = callee.as_ref() {
                    if !matches!(
                        name.as_str(),
                        "assert" | "ReferenceError" | "TypeError" | "SyntaxError"
                    ) && !scope.has(name)
                        && !self.state.has_function(name)
                    {
                        return self.state.has_class(name);
                    }
                }
            }
            Expr::Super(_) => {
                if let Ok((_, Some(parent))) = self.super_target(scope) {
                    return self.state.has_class(&parent);
                }
            }
            _ => {}
        }
        false
    }

    /// Static class-rule reference checks must not turn an invalid binding
    /// into a missing-RHS-reference error before construction gets to validate.
    pub(crate) fn check_constructor_bindings(
        &self,
        expression: &Expr,
        scope: &Scope,
        assignment_instantiation: bool,
    ) -> Result<()> {
        if !expression.has_named_arguments() {
            return Ok(());
        }
        match expression {
            Expr::Call { callee, arguments }
                if (assignment_instantiation && self.is_instantiation(expression))
                    || self.is_constructor_call(expression, scope) =>
            {
                if let Expr::Identifier(name) = callee.as_ref() {
                    bind_arguments(self.effective_constructor(name)?, arguments)?;
                }
            }
            Expr::Super(arguments) if self.is_constructor_call(expression, scope) => {
                if let (_, Some(parent)) = self.super_target(scope)? {
                    bind_arguments(self.effective_constructor(&parent)?, arguments)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn prepare_constructor(
        &mut self,
        name: &str,
        arguments: &[Expr],
        scope: &mut Scope,
    ) -> Result<ConstructorCall> {
        let class = self.effective_constructor(name)?;
        if !arguments
            .iter()
            .any(|argument| argument.named_argument().is_some())
        {
            return self
                .evaluate_all(arguments, scope)
                .map(ConstructorCall::Positional);
        }

        let bindings = bind_arguments(class, arguments)?;

        // Invalid bindings need no owned snapshot, but valid calls must pin
        // the selected signature/body before evaluating any RHS.
        let constructor = ClassShape::of(class);
        let has_rest = constructor
            .parameters
            .last()
            .is_some_and(|parameter| parameter.rest);
        // None is an absent key, not a supplied Null or Undefined value.
        // Keep only fixed holes, followed by raw positional extras, so the
        // shared binder constructs the rest list exactly once.
        let fixed_count = constructor.parameters.len() - usize::from(has_rest);
        let mut values = vec![None; fixed_count];
        for binding in bindings {
            let value = self.evaluate(binding.expression, scope)?;
            if let Some(slot) = binding.slot {
                let entry = values
                    .get_mut(slot)
                    .ok_or_else(|| Error::type_error("Constructor argument has no parameter"))?;
                *entry = Some(value);
            } else if has_rest {
                values.push(Some(value));
            }
        }
        Ok(ConstructorCall::Bound {
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
            ConstructorCall::Positional(values) => (
                ClassShape::of(self.effective_constructor(name)?),
                values.into_iter().map(Some).collect(),
            ),
            ConstructorCall::Bound {
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
            let value = parameter.bind_value(&mut arguments);
            scope.declare(parameter.name, value);
        }

        self.with_tracking(TrackingMode::Isolated, |runtime| {
            runtime.execute_all(&class.constructor, &mut scope)
        })?;

        Ok(())
    }
}
