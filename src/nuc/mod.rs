//! Executable statement kinds, each with its own module and type.
//!
//! [`Nuc`] is the closed enum that dispatches each statement's execution.
//!
//! Each kind follows the same four-phase lifecycle:
//!
//! | phase | what it does |
//! | --- | --- |
//! | [`Nuc::before`] | prepare the node's expressions — freezing `.value` reads |
//! | [`Nuc::run`] | carry the statement out, reporting what it read |
//! | [`Nuc::graph`] | file the node and wire its edges |
//! | [`Nuc::after`] | wake whatever read what the node just wrote |
//!
//! [`Scope`] carries local and instance context, while
//! [`NodeKind`](crate::graph::NodeKind) classifies statements filed in the
//! dependency graph.

pub mod block;
pub mod class;
pub mod delete;
pub mod expression;
pub mod function;
pub mod object;
pub mod property;
pub mod suppose;
pub mod throw;
pub mod r#try;
pub mod value_assign;
pub mod variable;

#[path = "for.rs"]
pub mod r#for;
#[path = "if.rs"]
pub mod r#if;
#[path = "let.rs"]
pub mod r#let;
#[path = "return.rs"]
pub mod r#return;
#[path = "while.rs"]
pub mod r#while;

mod filing;

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{NodeKey, ShapeKey};
use crate::lang::ast::{Expr, Stmt};
use crate::lang::evaluation::{Flow, TrackingMode};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

use block::Block;
use class::Class;
use delete::Delete;
use expression::Expression;
use r#for::For;
use function::Function;
use r#if::If;
use r#let::Let;
use property::{Owner, Property};
use r#return::Return;
use suppose::Suppose;
use throw::Throw;
use r#try::Try;
use value_assign::ValueAssign;
use variable::Variable;
use r#while::While;

/// What [`Nuc::run`] produced: the value or `return` it ended with, and the
/// dependency keys recorded during execution.
#[derive(Debug)]
pub struct Outcome {
    pub flow: Flow,
    pub dependencies: IndexSet<NodeKey>,
}

impl Outcome {
    #[must_use]
    pub fn value(value: Value) -> Self {
        Outcome {
            flow: Flow::Normal(value),
            dependencies: IndexSet::new(),
        }
    }

    #[must_use]
    pub fn null() -> Self {
        Outcome::value(Value::Null)
    }

    #[must_use]
    pub fn flow(flow: Flow) -> Self {
        Outcome {
            flow,
            dependencies: IndexSet::new(),
        }
    }
}

/// A statement in the executable form stored by the graph.
#[derive(Debug, Clone)]
pub enum Nuc {
    Let(Let),
    Variable(Variable),
    Property(Property),
    Class(Class),
    Block(Block),
    If(If),
    For(For),
    While(While),
    Function(Function),
    Return(Return),
    Throw(Throw),
    Delete(Delete),
    Expression(Expression),
    Try(Try),
    /// `a: int` — a type without a definition.
    Declaration,
    Pass,
    Suppose(Suppose),
    ValueAssign(ValueAssign),
}

impl Nuc {
    /// Builds a statement node, using its scope to distinguish local,
    /// variable and property assignments.
    pub fn convert(runtime: &mut Runtime, scope: &mut Scope, statement: &Stmt) -> Result<Nuc> {
        Ok(match statement {
            Stmt::Assign { target, value } => {
                Nuc::assignment(runtime, scope, target, value.clone())?
            }
            Stmt::ValueAssign { target, value } => {
                Nuc::value_assignment(runtime, scope, target, value.clone())?
            }
            Stmt::Expression(expression) => Nuc::Expression(Expression::new(expression.clone())),
            Stmt::If {
                condition,
                consequent,
                alternate,
            } => Nuc::If(If::new(
                condition.clone(),
                consequent.clone(),
                alternate.clone(),
            )),
            Stmt::Block(statements) => Nuc::Block(Block::new(statements.clone())),
            Stmt::Class(declaration) => Nuc::Class(Class::new(declaration.clone())),
            Stmt::Function(function) => Nuc::Function(Function::new(function.clone())),
            Stmt::For {
                variable,
                iterable,
                body,
            } => Nuc::For(For::new(variable.clone(), iterable.clone(), body.clone())),
            Stmt::ForEntries {
                key,
                value,
                iterable,
                body,
            } => Nuc::For(For {
                value: Some(value.clone()),
                ..For::new(key.clone(), iterable.clone(), body.clone())
            }),
            Stmt::While { condition, body } => {
                Nuc::While(While::new(condition.clone(), body.clone()))
            }
            Stmt::Return(value) => Nuc::Return(Return::new(value.clone())),
            Stmt::Throw(exception) => Nuc::Throw(Throw::new(exception.clone())),
            Stmt::Delete(target) => Nuc::Delete(Delete::new(target.clone())),
            Stmt::Try {
                body,
                parameter,
                catch,
            } => Nuc::Try(Try::new(body.clone(), parameter.clone(), catch.clone())),
            Stmt::Declaration { .. } => Nuc::Declaration,
            Stmt::Pass => Nuc::Pass,
            Stmt::Suppose { target, body } => {
                Nuc::Suppose(Suppose::new(target.clone(), body.clone()))
            }
        })
    }

    /// Which kind of assignment this is. Only names already bound as locals —
    /// parameters, loop variables — stay local; everything else is a state
    /// assignment, so a block can define what the statements after it depend on.
    fn assignment(
        runtime: &mut Runtime,
        scope: &mut Scope,
        target: &Expr,
        value: Expr,
    ) -> Result<Nuc> {
        Ok(match target {
            Expr::Version { .. } => return Err(Error::type_error("State versions are read-only")),
            Expr::Identifier(name) => {
                if scope.has(name) || scope.assigned_by_enclosing(name) {
                    Nuc::Let(Let::new(name.clone(), value))
                } else {
                    scope.record_assignment(name);
                    Nuc::Variable(Variable::new(name.clone(), value))
                }
            }

            Expr::Member { object, property } => {
                if let Expr::ClassRef(class) = object.as_ref() {
                    if scope.instance().is_some_and(|instance| {
                        runtime.property_rule_is_overridden(class, instance, property)
                    }) {
                        return Ok(Nuc::Pass);
                    }
                    if scope.instance().is_none() {
                        if !runtime.state.has_class(class) {
                            return Err(Error::not_defined(class));
                        }
                        return Ok(Nuc::Property(Property::new(
                            Owner::Class(class.clone()),
                            property.clone(),
                            value,
                        )));
                    }
                }

                let base = runtime.evaluate(object, scope)?;

                match base {
                    Value::Object(id) => {
                        let assignment = Property::new(Owner::Object(id), property.clone(), value);
                        Nuc::Property(match object.as_ref() {
                            Expr::ClassRef(class) => assignment.with_receiver(class),
                            _ => assignment,
                        })
                    }
                    Value::Class(class) => {
                        Nuc::Property(Property::new(Owner::Class(class), property.clone(), value))
                    }
                    Value::Undefined | Value::Null => {
                        return Err(Error::not_defined(object.describe()));
                    }
                    other => {
                        return Err(Error::type_error(format!(
                            "Cannot assign '{property}' on {}",
                            other.type_name()
                        )));
                    }
                }
            }

            Expr::This => return Err(Error::type_error("Cannot assign to 'this'")),

            other => return Err(Error::syntax(format!("Cannot assign to {other}"))),
        })
    }

    fn value_assignment(
        runtime: &mut Runtime,
        scope: &mut Scope,
        target: &Expr,
        value: Expr,
    ) -> Result<Nuc> {
        let (assignment, _) = runtime.with_tracking(TrackingMode::Isolated, |runtime| {
            Nuc::assignment(runtime, scope, target, value)
        })?;

        match assignment {
            Nuc::Property(Property {
                owner: Owner::Class(_),
                ..
            }) => Err(Error::type_error(
                "Cannot use ':=' on a class-level property",
            )),
            Nuc::Pass => Ok(Nuc::Pass),
            assignment => Ok(Nuc::ValueAssign(ValueAssign::new(assignment))),
        }
    }

    /// Prepares expressions before execution.
    pub fn before(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<()> {
        match self {
            Nuc::Variable(node) => node.before(runtime, scope),
            Nuc::Property(node) => node.before(runtime, scope),
            Nuc::If(node) => node.before(runtime, scope),
            _ => Ok(()),
        }
    }

    /// Executes the statement and records its dependencies.
    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        match self {
            Nuc::Let(node) => node.run(runtime, scope),
            Nuc::Variable(node) => node.run(runtime, scope),
            Nuc::Property(node) => node.run(runtime, scope),
            Nuc::Class(node) => node.run(runtime, scope),
            Nuc::Block(node) => node.run(runtime, scope),
            Nuc::If(node) => node.run(runtime, scope),
            Nuc::For(node) => node.run(runtime, scope),
            Nuc::While(node) => node.run(runtime, scope),
            Nuc::Function(node) => node.run(runtime, scope),
            Nuc::Return(node) => node.run(runtime, scope),
            Nuc::Throw(node) => node.run(runtime, scope),
            Nuc::Delete(node) => node.run(runtime, scope),
            Nuc::Expression(node) => node.run(runtime, scope),
            Nuc::Try(node) => node.run(runtime, scope),
            Nuc::Declaration => Err(Error::reference("Missing definition")),
            Nuc::Pass => Ok(Outcome::null()),
            Nuc::Suppose(node) => node.run(runtime, scope),
            Nuc::ValueAssign(node) => node.run(runtime, scope),
        }
    }

    /// Files the node and wires edges to its dependencies. Kinds that hold no
    /// standing declaration do nothing.
    pub fn graph(&self, runtime: &mut Runtime, dependencies: IndexSet<NodeKey>) -> Result<()> {
        self.graph_observed(runtime, dependencies, IndexSet::new())
    }

    pub(crate) fn graph_observed(
        &self,
        runtime: &mut Runtime,
        dependencies: IndexSet<NodeKey>,
        shapes: IndexSet<ShapeKey>,
    ) -> Result<()> {
        match self {
            Nuc::Variable(node) => node.graph_observed(runtime, dependencies, shapes),
            Nuc::Property(node) => node.graph_observed(runtime, dependencies, shapes),
            Nuc::Class(node) => node.graph(runtime),
            Nuc::Block(node) => node.graph_observed(runtime, dependencies, shapes),
            Nuc::If(node) => node.graph_observed(runtime, dependencies, shapes),
            Nuc::Function(node) => node.graph(runtime),
            Nuc::ValueAssign(node) => node.graph(runtime),
            _ => Ok(()),
        }
    }

    /// Whether this node reads `source` only at fixed history positions of its
    /// own class receiver. Only such edges may close a cycle.
    pub(crate) fn anchors_on(&self, source: &NodeKey) -> bool {
        matches!(self, Nuc::Property(property) if property.anchors_on(source))
    }

    /// Wakes everything that read what this node wrote.
    /// A kind whose key nothing can read has nothing to wake.
    pub fn after(&self, runtime: &mut Runtime) -> Result<()> {
        match self {
            Nuc::Variable(node) => node.after(runtime),
            Nuc::Property(node) => node.after(runtime),
            Nuc::Class(node) => node.after(runtime),
            Nuc::Function(node) => node.after(runtime),
            Nuc::ValueAssign(node) => node.after(runtime),
            _ => Ok(()),
        }
    }
}
