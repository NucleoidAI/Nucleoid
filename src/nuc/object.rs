//! Creating an instance and running its constructor. Mirrors
//! `ref/src/nuc/OBJECT.js`.

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{Expr, Parameter, Stmt};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::state::ClassData;
use crate::value::{ObjectData, ObjectId, Value};

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

impl Runtime {
    pub(crate) fn create_instance(
        &mut self,
        class_name: &str,
        arguments: &[Expr],
        id: ObjectId,
        scope: &mut Scope,
    ) -> Result<Value> {
        let Some(class) = self.state.class(class_name) else {
            return Err(Error::not_defined(class_name));
        };

        let shape = ClassShape::of(class);

        let mut values = Vec::new();
        for argument in arguments {
            values.push(self.evaluate(argument, scope)?);
        }

        if self.transaction.needs_object(&id) {
            let before = self.state.object(&id).cloned();
            self.transaction.record_object(&id, before);
        }

        let mut data = ObjectData::new(Some(class_name.to_string()));
        data.properties
            .insert("id".to_string(), Value::String(id.to_string()));
        self.state.objects.insert(id.clone(), data);

        if self.transaction.needs_class(class_name) {
            let before = self.state.class(class_name).cloned();
            self.transaction.record_class(class_name, before);
        }

        if let Some(data) = self.state.class_mut(class_name) {
            if !data.instances.contains(&id) {
                data.instances.push(id.clone());
            }
        }

        let key = NodeKey::new(id.to_string());
        self.register(&key, NodeKind::Object, None, IndexSet::new(), None)?;

        self.run_constructor(&shape, &values, &id)?;

        for declaration in self.declarations_for(class_name) {
            self.apply_declaration(&declaration.statement, &id)?;
        }

        self.propagate(&NodeKey::new(format!("${class_name}")))?;

        Ok(Value::Object(id))
    }

    fn run_constructor(
        &mut self,
        class: &ClassShape,
        arguments: &[Value],
        id: &ObjectId,
    ) -> Result<()> {
        if let Some(parent) = &class.parent {
            if class.constructor.is_empty() {
                if let Some(parent) = self.state.class(parent).map(ClassShape::of) {
                    self.run_constructor(&parent, arguments, id)?;
                }
            }
        }

        if class.constructor.is_empty() && class.parameters.is_empty() {
            return Ok(());
        }

        let mut scope = Scope::new();
        scope.set_this(Some(id.clone()));

        for (index, parameter) in class.parameters.iter().enumerate() {
            let value = arguments.get(index).cloned().unwrap_or(Value::Null);
            scope.declare(parameter.name.clone(), value);
        }

        let statements = class.constructor.clone();
        self.push_tracking(true);
        let result = self.execute_all(&statements, &mut scope);
        self.pop_tracking();
        result?;

        Ok(())
    }
}
