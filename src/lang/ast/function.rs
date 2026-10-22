//! Function bodies and how they are entered.

use std::sync::Arc;

use crate::error::{Error, Result};
use crate::lang::ast::{Expr, Function as FunctionDecl, FunctionBody};
use crate::lang::evaluation::Flow;
use crate::nuc::object::ConstructorCall;
use crate::runtime::Runtime;
use crate::scope::Scope;
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
        // Only labeled calls pin a parent before values run. Positional calls
        // must resolve it after any argument-side class redeclaration.
        if arguments
            .iter()
            .any(|argument| argument.named_argument().is_some())
        {
            if let Ok((this, Some(parent))) = self.super_target(scope) {
                if self.state.has_class(&parent) {
                    let call = self.prepare_constructor(&parent, arguments, scope)?;
                    self.run_constructor(&parent, call, &this)?;
                    return Ok(Value::Null);
                }
            }
        }

        // Preserve argument evaluation before receiver/parent resolution for
        // positional calls and calls without a receiver or parent.
        let mut values = Vec::new();
        for argument in arguments {
            values.push(self.evaluate(argument, scope)?);
        }

        let (this, parent) = self.super_target(scope)?;

        if let Some(parent) = parent {
            if self.state.has_class(&parent) {
                self.run_constructor(&parent, ConstructorCall::Positional(values), &this)?;
            }
        }

        Ok(Value::Null)
    }

    pub(crate) fn super_target(&self, scope: &Scope) -> Result<(ObjectId, Option<String>)> {
        let this = scope
            .this()
            .cloned()
            .ok_or_else(|| Error::reference("super is not defined"))?;
        let class = scope.constructor_class().map(str::to_string).or_else(|| {
            self.state
                .object(&this)
                .and_then(|object| object.class.clone())
        });
        let parent = class.and_then(|class| {
            self.state
                .class(&class)
                .and_then(|data| data.parent.clone())
        });

        Ok((this, parent))
    }
}
