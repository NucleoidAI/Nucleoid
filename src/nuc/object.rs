//! `OBJECT` — an instance: created, given its constructor's properties, then
//! handed every rule its class states. Mirrors `ref/src/nuc/OBJECT.js`.
//!
//! Like `ref`, this is never a statement of its own. It is what an assignment
//! turns into when its value is an instantiation, which is the job
//! `ref/src/lang/$nuc/$ASSIGNMENT.js` does there and [`crate::nuc::Nuc`] does
//! here.

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{Expr, Parameter, Stmt};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::state::ClassData;
use crate::value::{ObjectData, ObjectId, Value};

pub struct Object {
    pub id: ObjectId,
    pub class: String,
    pub arguments: Vec<Expr>,
}

/// The parts of a class a constructor needs, lifted out so creating an instance
/// does not copy the list of instances the class already has.
struct ClassShape {
    parent: Option<String>,
    parameters: Vec<Parameter>,
    constructor: Vec<Stmt>,
}

impl ClassShape {
    fn of(class: &ClassData) -> Self {
        ClassShape {
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
        let Some(class) = runtime.state.class(&self.class) else {
            return Err(Error::not_defined(&self.class));
        };

        let shape = ClassShape::of(class);

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

        self.constructor(runtime, &shape, &values)?;

        for declaration in runtime.declarations_for(&self.class) {
            runtime.apply_declaration(&declaration.statement, &self.id)?;
        }

        self.after(runtime)?;

        Ok(Value::Object(self.id.clone()))
    }

    /// `OBJECT.graph()` — files the instance so the class's own node can reach
    /// it.
    fn graph(&self, runtime: &mut Runtime) -> Result<()> {
        runtime.file(&self.key(), NodeKind::Object, None, IndexSet::new(), None)
    }

    /// Wakes everything that reads the class, since it now has one more
    /// instance.
    fn after(&self, runtime: &mut Runtime) -> Result<()> {
        runtime.propagate(&NodeKey::class(&self.class))
    }

    fn constructor(
        &self,
        runtime: &mut Runtime,
        class: &ClassShape,
        arguments: &[Value],
    ) -> Result<()> {
        if let Some(parent) = &class.parent {
            if class.constructor.is_empty() {
                if let Some(parent) = runtime.state.class(parent).map(ClassShape::of) {
                    self.constructor(runtime, &parent, arguments)?;
                }
            }
        }

        if class.constructor.is_empty() && class.parameters.is_empty() {
            return Ok(());
        }

        let mut scope = Scope::new();
        scope.set_this(Some(self.id.clone()));

        for (index, parameter) in class.parameters.iter().enumerate() {
            let value = arguments.get(index).cloned().unwrap_or(Value::Null);
            scope.declare(parameter.name.clone(), value);
        }

        let statements = class.constructor.clone();
        runtime.push_tracking(true);
        let result = runtime.execute_all(&statements, &mut scope);
        runtime.pop_tracking();
        result?;

        Ok(())
    }
}
