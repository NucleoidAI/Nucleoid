//! Function bodies and how they are entered.

use std::sync::Arc;

use crate::error::{Error, Result};
use crate::lang::ast::{Expr, Function as FunctionDecl, FunctionBody, Parameter};
use crate::lang::evaluation::Flow;
use crate::nuc::object::ConstructorCall;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

pub(crate) const SELECTOR_PARAMETER: &str = "";

#[derive(Debug)]
pub struct Function<'a> {
    pub node: &'a Expr,
}

impl FunctionDecl {
    #[must_use]
    pub(crate) fn selector(path: Vec<String>) -> Arc<Self> {
        let body = path.into_iter().fold(
            Expr::Identifier(SELECTOR_PARAMETER.to_string()),
            |object, property| Expr::Member {
                object: Box::new(object),
                property,
            },
        );
        let selector = Arc::new(FunctionDecl {
            name: None,
            parameters: vec![Parameter {
                name: SELECTOR_PARAMETER.to_string(),
                type_name: None,
                default: None,
                rest: false,
            }],
            body: FunctionBody::Expression(Box::new(body)),
        });
        debug_assert!(selector.selector_path().is_some());
        selector
    }

    #[must_use]
    pub(crate) fn selector_path(&self) -> Option<&Expr> {
        let [parameter] = self.parameters.as_slice() else {
            return None;
        };
        let unnamed = self.name.is_none()
            && parameter.name == SELECTOR_PARAMETER
            && parameter.type_name.is_none()
            && parameter.default.is_none()
            && !parameter.rest;
        if !unnamed {
            return None;
        }
        let FunctionBody::Expression(body) = &self.body else {
            return None;
        };
        let mut node = body.as_ref();
        let mut members = 0_usize;
        while let Expr::Member { object, .. } = node {
            node = object;
            members += 1;
        }
        let rooted = matches!(node, Expr::Identifier(name) if name == SELECTOR_PARAMETER);
        (rooted && members > 0).then_some(body)
    }
}

impl<'a> Function<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Function { node }
    }

    pub fn resolve(&self) -> Result<Value> {
        let Expr::Function(function) = self.node else {
            return Err(Error::type_error(format!(
                "{} is not a function",
                self.node
            )));
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

        let mut arguments = arguments.iter().cloned().map(Some);
        for parameter in &function.parameters {
            let value = parameter.bind_value(&mut arguments);
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
