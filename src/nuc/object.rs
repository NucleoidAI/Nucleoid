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
struct ClassShape {
    name: String,
    parent: Option<String>,
    parameters: Vec<Parameter>,
    constructor: Vec<Stmt>,
}

impl ClassShape {
    fn of(class: &ClassData) -> Self {
        ClassShape {
            name: class.name.clone(),
            parent: class.parent.clone(),
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

        let mut values = Vec::new();
        for argument in &self.arguments {
            values.push(runtime.evaluate(argument, scope)?);
        }

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

        runtime.run_constructor(&self.class, &values, &self.id)?;

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
    pub(crate) fn run_constructor(
        &mut self,
        name: &str,
        arguments: &[Value],
        this: &ObjectId,
    ) -> Result<()> {
        let mut class = self
            .state
            .class(name)
            .map(ClassShape::of)
            .ok_or_else(|| Error::not_defined(name))?;

        while class.constructor.is_empty() {
            let Some(parent) = class
                .parent
                .as_ref()
                .and_then(|name| self.state.class(name))
            else {
                break;
            };
            class = ClassShape::of(parent);
        }

        if class.constructor.is_empty() && class.parameters.is_empty() {
            return Ok(());
        }

        let mut scope = Scope::new();
        scope.set_this(Some(this.clone()));
        scope.set_constructor_class(class.name);

        for (index, parameter) in class.parameters.iter().enumerate() {
            let value = arguments.get(index).cloned().unwrap_or(Value::Null);
            scope.declare(parameter.name.clone(), value);
        }

        self.with_tracking(TrackingMode::Isolated, |runtime| {
            runtime.execute_all(&class.constructor, &mut scope)
        })?;

        Ok(())
    }
}
