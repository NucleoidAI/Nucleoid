//! Function bodies and how they are entered. Mirrors
//! `ref/src/lang/ast/Function.js`.

use std::sync::Arc;

use crate::error::{Error, Result};
use crate::lang::ast::{Expr, Function as FunctionDecl, FunctionBody};
use crate::lang::evaluation::Flow;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::state::ClassData;
use crate::value::{ObjectId, Value};

pub struct Function<'a> {
    pub node: &'a Expr,
}

impl<'a> Function<'a> {
    pub fn new(node: &'a Expr) -> Self {
        Function { node }
    }

    pub fn resolve(&self) -> Result<Value> {
        let Expr::Function(function) = self.node else {
            unreachable!("Function only wraps Expr::Function")
        };

        Ok(Value::Function(function.clone()))
    }
}

impl Runtime {
    pub(crate) fn invoke(
        &mut self,
        function: &Arc<FunctionDecl>,
        arguments: &[Value],
    ) -> Result<Value> {
        self.invoke_with_this(function, arguments, None)
    }

    pub(crate) fn invoke_with_this(
        &mut self,
        function: &Arc<FunctionDecl>,
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

    /// `super(...)`, which runs the parent constructor against the same object.
    pub(crate) fn call_super(&mut self, arguments: &[Expr], scope: &mut Scope) -> Result<Value> {
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

    fn run_super(&mut self, class: &ClassData, arguments: &[Value], this: &ObjectId) -> Result<()> {
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
}
