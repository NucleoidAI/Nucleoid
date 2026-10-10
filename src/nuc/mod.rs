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
//! [`Scope`] carries local and instance context, while [`NodeKind`] classifies
//! statements filed in the dependency graph.

pub mod block;
pub mod class;
pub mod delete;
pub mod expression;
pub mod function;
pub mod object;
pub mod property;
pub mod throw;
pub mod r#try;
pub mod variable;

#[path = "for.rs"]
pub mod r#for;
#[path = "if.rs"]
pub mod r#if;
#[path = "let.rs"]
pub mod r#let;
#[path = "return.rs"]
pub mod r#return;

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{GraphNode, NodeKey, NodeKind, ShapeKey};
use crate::lang::ast::{Expr, Stmt};
use crate::lang::evaluation::Flow;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

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
use throw::Throw;
use r#try::Try;
use variable::Variable;

/// What [`Nuc::run`] produced: the value or `return` it ended with, and the
/// dependency keys recorded during execution.
pub struct Outcome {
    pub flow: Flow,
    pub dependencies: IndexSet<NodeKey>,
}

impl Outcome {
    pub fn value(value: Value) -> Self {
        Outcome {
            flow: Flow::Normal(value),
            dependencies: IndexSet::new(),
        }
    }

    pub fn null() -> Self {
        Outcome::value(Value::Null)
    }

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
    Function(Function),
    Return(Return),
    Throw(Throw),
    Delete(Delete),
    Expression(Expression),
    Try(Try),
    /// `a: int` — a type without a definition.
    Declaration,
    Pass,
}

impl Nuc {
    /// Builds a statement node, using its scope to distinguish local,
    /// variable and property assignments.
    pub fn convert(runtime: &mut Runtime, scope: &mut Scope, statement: &Stmt) -> Result<Nuc> {
        Ok(match statement {
            Stmt::Assign { target, value } => {
                Nuc::assignment(runtime, scope, target, value.clone())?
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
                        Nuc::Property(Property::new(Owner::Object(id), property.clone(), value))
                    }
                    Value::Class(class) => {
                        Nuc::Property(Property::new(Owner::Class(class), property.clone(), value))
                    }
                    Value::Undefined | Value::Null => {
                        return Err(Error::not_defined(runtime.describe(object, scope)));
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
            Nuc::Function(node) => node.run(runtime, scope),
            Nuc::Return(node) => node.run(runtime, scope),
            Nuc::Throw(node) => node.run(runtime, scope),
            Nuc::Delete(node) => node.run(runtime, scope),
            Nuc::Expression(node) => node.run(runtime, scope),
            Nuc::Try(node) => node.run(runtime, scope),
            Nuc::Declaration => Err(Error::reference("Missing definition")),
            Nuc::Pass => Ok(Outcome::null()),
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
            _ => Ok(()),
        }
    }

    /// Wakes everything that read what this node wrote.
    /// A kind whose key nothing can read has nothing to wake.
    pub fn after(&self, runtime: &mut Runtime) -> Result<()> {
        match self {
            Nuc::Variable(node) => node.after(runtime),
            Nuc::Property(node) => node.after(runtime),
            Nuc::Class(node) => node.after(runtime),
            Nuc::Function(node) => node.after(runtime),
            _ => Ok(()),
        }
    }
}

impl Runtime {
    /// Files a node under `key` and wires the edges to everything it read.
    ///
    /// Checks new edges before calling [`Runtime::register`] or
    /// [`Runtime::replace`], then wires dependencies with [`Runtime::direct`].
    pub(crate) fn file(
        &mut self,
        key: &NodeKey,
        kind: NodeKind,
        nuc: Option<Nuc>,
        dependencies: IndexSet<NodeKey>,
        instance: Option<ObjectId>,
    ) -> Result<()> {
        self.file_observed(key, kind, nuc, dependencies, instance, IndexSet::new())
    }

    pub(crate) fn file_observed(
        &mut self,
        key: &NodeKey,
        kind: NodeKind,
        nuc: Option<Nuc>,
        dependencies: IndexSet<NodeKey>,
        instance: Option<ObjectId>,
        shapes: IndexSet<ShapeKey>,
    ) -> Result<()> {
        let existing = self.graph.retrieve(key).cloned();
        let was_visible = existing
            .as_ref()
            .is_some_and(|node| node.kind != NodeKind::Pending && !self.graph.is_observer(key));

        for dependency in &dependencies {
            if dependency == key {
                continue;
            }

            // An edge that is already there was checked when it was made, and
            // re-checking costs a walk of everything downstream — which is the
            // whole graph, on every re-evaluation.
            let unchanged = existing
                .as_ref()
                .is_some_and(|node| node.dependencies.contains(dependency));

            if unchanged {
                continue;
            }

            if self.graph.reaches(key, dependency) {
                return Err(Error::type_error("Circular Dependency"));
            }
        }

        let sequence = match &existing {
            Some(existing) if self.stack.is_running(key) => existing.sequence,
            _ => self.graph.next_sequence(),
        };
        let mut node = GraphNode::new(key.clone(), kind, sequence);
        node.node = nuc;
        node.instance = instance;
        node.dependencies = dependencies.clone();

        match &existing {
            Some(existing) => self.replace(node, existing),
            None => self.register(node),
        }

        for dependency in &dependencies {
            if dependency == key {
                continue;
            }

            self.direct(dependency, key);
        }

        self.replace_shape_dependencies(key, shapes);

        let is_visible = kind != NodeKind::Pending && !self.graph.is_observer(key);
        let shape_changed = !self.stack.is_running(key)
            || was_visible != is_visible
            || existing.as_ref().is_none_or(|node| {
                node.kind != kind || !node.dependencies.iter().eq(dependencies.iter())
            });

        if (was_visible || is_visible) && shape_changed {
            self.queue_shape_change(&ShapeKey::Node(key.clone()));

            let mut sources = IndexSet::new();
            if was_visible {
                if let Some(existing) = &existing {
                    sources.extend(existing.dependencies.iter().cloned());
                }
            }
            if is_visible {
                sources.extend(dependencies);
            }
            for source in sources {
                self.queue_shape_change(&ShapeKey::Node(source));
            }

            if was_visible != is_visible {
                self.queue_shape_change(&ShapeKey::Model);
            }
        }

        Ok(())
    }

    fn replace_shape_dependencies(&mut self, key: &NodeKey, mut shapes: IndexSet<ShapeKey>) {
        shapes.shift_remove(&ShapeKey::Node(key.clone()));
        let previous: Vec<ShapeKey> = self.graph.shape_dependencies(key).cloned().collect();

        for source in previous {
            if !shapes.contains(&source) && self.graph.remove_shape_dependency(&source, key) {
                self.transaction
                    .record_shape_dependent_removed(&source, key);
            }
        }

        for source in shapes {
            if self.graph.add_shape_dependency(&source, key) {
                self.transaction.record_shape_dependent_added(&source, key);
            }
        }
    }

    /// Puts a node in the graph under its own key.
    pub(crate) fn register(&mut self, node: GraphNode) {
        if self.transaction.needs_node(&node.key) {
            self.transaction.record_node(&node.key, None);
        }

        self.graph.insert(node);
    }

    /// Puts a node where one already stood, so that a
    /// redeclaration keeps the edges pointing at it.
    pub(crate) fn replace(&mut self, mut node: GraphNode, existing: &GraphNode) {
        let key = node.key.clone();

        if self.transaction.needs_node(&key) {
            self.transaction.record_node(&key, Some(existing.clone()));
        }

        node.dependents = existing.dependents.clone();

        // Drop edges from dependencies this declaration no longer has.
        for previous in &existing.dependencies {
            if !node.dependencies.contains(previous) {
                let removed = self
                    .graph
                    .get_mut(previous)
                    .is_some_and(|source| source.dependents.shift_remove(&key));

                if removed {
                    self.transaction.record_dependent_removed(previous, &key);
                }
            }
        }

        self.graph.insert(node);
    }

    /// Wires one edge, from what was read to what read it. A
    /// name that has not been defined yet still gets a node, so the edge is
    /// there waiting when it is.
    pub(crate) fn direct(&mut self, source: &NodeKey, target: &NodeKey) {
        if !self.graph.contains(source) {
            let sequence = self.graph.next_sequence();
            let pending = GraphNode::new(source.clone(), NodeKind::Pending, sequence);
            self.transaction.record_node(source, None);
            self.graph.insert(pending);
        }

        let added = self
            .graph
            .get_mut(source)
            .is_some_and(|node| node.dependents.insert(target.clone()));

        if added {
            self.transaction.record_dependent_added(source, target);
        }
    }

    /// Puts a placeholder where a node stood, keeping its dependents so they
    /// are still woken when the name is defined again.
    pub(crate) fn remove_node(&mut self, key: &NodeKey) {
        if let Some(node) = self.graph.retrieve(key).cloned() {
            let was_visible = node.kind != NodeKind::Pending && !self.graph.is_observer(key);
            self.transaction.record_node(key, Some(node.clone()));

            for dependency in &node.dependencies {
                let removed = self
                    .graph
                    .get_mut(dependency)
                    .is_some_and(|source| source.dependents.shift_remove(key));

                if removed {
                    self.transaction.record_dependent_removed(dependency, key);
                }
            }

            let sequence = self.graph.next_sequence();
            let mut placeholder = GraphNode::new(key.clone(), NodeKind::Pending, sequence);
            placeholder.dependents = node.dependents.clone();
            self.graph.insert(placeholder);
            self.replace_shape_dependencies(key, IndexSet::new());

            self.queue_shape_change(&ShapeKey::Node(key.clone()));
            if was_visible {
                for source in node.dependencies {
                    self.queue_shape_change(&ShapeKey::Node(source));
                }
                self.queue_shape_change(&ShapeKey::Model);
            }
        }
    }
}
