//! The runtime: what holds the state and the graph, and drives one statement
//! after another through its four execution phases.

use indexmap::IndexSet;
use std::collections::HashSet;

use crate::error::{Error, Result};
use crate::graph::{Graph, NodeKey};
use crate::lang::ast::Stmt;
use crate::lang::evaluation::{Flow, Tracking, TrackingMode};
use crate::nuc::Nuc;
use crate::nuc::expression::is_assertion;
use crate::scope::Scope;
use crate::stack::Stack;
use crate::state::State;
use crate::statement;
use crate::transaction::Transaction;
use crate::value::{ObjectId, Value};

/// A failed `assert` recorded during a run.
#[derive(Debug, Clone)]
pub struct AssertionFailure {
    pub actual: Value,
    pub expected: Value,
}

pub struct Runtime {
    pub state: State,
    pub graph: Graph,
    pub(crate) transaction: Transaction,
    pub(crate) stack: Stack,
    pub(crate) assertions: Vec<AssertionFailure>,
    pub(crate) assertions_run: usize,
    pub(crate) tracking: Tracking,
    pub(crate) depth: usize,
    /// Set when an expression reads something that is null, which makes the
    /// assignment it feeds store null rather than a coerced value.
    pub(crate) null_read: bool,
    /// Greater than zero while running a `for` body, where statements are
    /// carried out once instead of being filed as standing declarations.
    imperative: usize,
    /// Set when an expression reads a property that has never been defined.
    pub(crate) undefined_read: bool,
    /// The instances whose class-level rules are being applied, innermost last.
    /// Functions called from a rule see the same instance for `$Class`.
    pub(crate) instances: Vec<ObjectId>,
    /// Names being removed. While a deletion propagates, reading one of these
    /// is undefined rather than an error, so dependents settle to null instead
    /// of aborting the transaction.
    pub(crate) deleted: IndexSet<NodeKey>,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime::new()
    }
}

/// How deep statements, propagation and expression evaluation may nest before
/// the runtime gives up. A library cannot rely on the caller's stack, so this
/// is reported as an error rather than overflowing.
pub(crate) const MAX_DEPTH: usize = 192;

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            state: State::new(),
            graph: Graph::new(),
            transaction: Transaction::new(),
            stack: Stack::new(),
            assertions: Vec::new(),
            assertions_run: 0,
            tracking: Tracking::default(),
            depth: 0,
            null_read: false,
            imperative: 0,
            undefined_read: false,
            instances: Vec::new(),
            deleted: IndexSet::new(),
        }
    }

    /// Runs a program. Everything it changes is one transaction: if a statement
    /// raises, the state is left exactly as it was.
    pub fn run(&mut self, source: &str) -> Result<Value> {
        let program = statement::compile(source)?;
        let mut scope = Scope::new();

        self.transaction.start();
        self.deleted.clear();

        let mut value = Value::Null;
        let mut failure = None;

        for (index, statement) in program.statements.iter().enumerate() {
            let position = program.positions.get(index).copied();

            match self.execute(statement, &mut scope) {
                Ok(Flow::Return(returned)) => {
                    value = returned;
                    break;
                }
                Ok(Flow::Normal(produced)) => {
                    if !is_assertion(statement) {
                        value = produced;
                    }
                }
                Err(error) => {
                    // The statement that was running is where the reader has to
                    // look, so an error without a place of its own gets this one.
                    failure = Some(match position {
                        Some(position) => error.or_at(position),
                        None => error,
                    });
                    break;
                }
            }
        }

        match failure {
            None => {
                self.transaction.commit();
                Ok(value)
            }
            Some(error) => {
                self.transaction.rollback(&mut self.state, &mut self.graph);
                Err(error)
            }
        }
    }

    /// Parses without running, reporting the first syntax error.
    pub fn check(source: &str) -> Result<()> {
        statement::compile(source).map(|_| ())
    }

    pub fn take_assertions(&mut self) -> Vec<AssertionFailure> {
        std::mem::take(&mut self.assertions)
    }

    /// How many `assert` calls actually ran. A case whose assertions sit on a
    /// branch that was never taken proves nothing, and this is how the suites
    /// notice.
    pub fn assertions_run(&self) -> usize {
        self.assertions_run
    }

    /// Serializes a runtime value as JSON, resolving object references through
    /// the current state.
    pub fn serialize_json(&self, value: &Value) -> Result<String> {
        let value = self.json_value(value, &mut HashSet::new())?;

        serde_json::to_string(&value)
            .map_err(|error| Error::type_error(format!("Cannot serialize value as JSON: {error}")))
    }

    fn json_value(
        &self,
        value: &Value,
        ancestors: &mut HashSet<ObjectId>,
    ) -> Result<serde_json::Value> {
        let value = match value {
            Value::Undefined | Value::Null => serde_json::Value::Null,
            Value::Bool(value) => serde_json::Value::Bool(*value),
            Value::Number(value) => serde_json::Number::from_f64(*value)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Value::String(value) => serde_json::Value::String(value.clone()),
            Value::List(values) => serde_json::Value::Array(
                values
                    .iter()
                    .map(|value| self.json_value(value, ancestors))
                    .collect::<Result<Vec<_>>>()?,
            ),
            Value::Object(id) => {
                if !ancestors.insert(id.clone()) {
                    return Err(Error::type_error("Converting circular structure to JSON"));
                }

                let mut properties = serde_json::Map::new();

                if let Some(object) = self.state.object(id) {
                    for (name, value) in &object.properties {
                        if !value.is_undefined() {
                            properties.insert(name.clone(), self.json_value(value, ancestors)?);
                        }
                    }
                }

                ancestors.remove(id);
                serde_json::Value::Object(properties)
            }
            Value::Class(name) => serde_json::Value::String(name.clone()),
            Value::Date(millis) => serde_json::Value::String(millis.to_string()),
            Value::Regex(regex) => serde_json::Value::String(regex.as_str().to_string()),
            Value::Function(_) => serde_json::Value::String("[function]".to_string()),
        };

        Ok(value)
    }

    pub fn clear(&mut self) {
        self.state.clear();
        self.graph.clear();
        self.assertions.clear();
    }

    // -- statement execution -------------------------------------------------

    pub(crate) fn execute_all(&mut self, statements: &[Stmt], scope: &mut Scope) -> Result<Flow> {
        let mut last = Value::Null;

        for statement in statements {
            match self.execute(statement, scope)? {
                Flow::Return(value) => return Ok(Flow::Return(value)),
                Flow::Normal(value) => {
                    if !is_assertion(statement) {
                        last = value;
                    }
                }
            }
        }

        Ok(Flow::Normal(last))
    }

    pub(crate) fn execute(&mut self, statement: &Stmt, scope: &mut Scope) -> Result<Flow> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(Error::type_error("Maximum statement depth exceeded"));
        }

        let result = Nuc::convert(self, scope, statement)
            .and_then(|mut node| self.process(&mut node, scope));
        self.depth -= 1;
        result
    }

    /// Runs a node the graph already holds, because something it reads changed.
    ///
    /// The node does not have to be built from source again — that is the point
    /// of the graph holding the node rather than the statement it was written
    /// as — but it is still the same four phases, and the same depth limit.
    pub(crate) fn rerun(&mut self, node: &mut Nuc, scope: &mut Scope) -> Result<Flow> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(Error::type_error("Maximum statement depth exceeded"));
        }

        let result = self.process(node, scope);
        self.depth -= 1;
        result
    }

    /// Prepares the node's expressions, executes it, files its dependencies,
    /// then wakes whatever was reading what it wrote.
    pub(crate) fn process(&mut self, node: &mut Nuc, scope: &mut Scope) -> Result<Flow> {
        let (outcome, shapes) = self.with_shape_tracking(TrackingMode::Inherited, |runtime| {
            node.before(runtime, scope)?;
            node.run(runtime, scope)
        })?;

        node.graph_observed(self, outcome.dependencies, shapes)?;
        node.after(self)?;
        self.drain_pending()?;

        Ok(outcome.flow)
    }

    // -- imperative depth ----------------------------------------------------
    //
    // Inside a `for` body, and inside the branch a standing `if` takes, a
    // statement is carried out once rather than filed as a declaration of its
    // own.

    pub(crate) fn is_imperative(&self) -> bool {
        self.imperative > 0
    }

    pub(crate) fn enter_imperative(&mut self) {
        self.imperative += 1;
    }

    pub(crate) fn leave_imperative(&mut self) {
        self.imperative -= 1;
    }

    /// Steps back out to declarative execution, for a statement that is a
    /// declaration wherever it happens to have been triggered from.
    pub(crate) fn suspend_imperative(&mut self) -> usize {
        std::mem::take(&mut self.imperative)
    }

    pub(crate) fn restore_imperative(&mut self, depth: usize) {
        self.imperative = depth;
    }
}
