//! The runtime: what holds the state and the graph, and drives one statement
//! after another. Mirrors `ref/src/runtime.js`.
//!
//! Each statement kind is carried out by the module named after it in
//! [`crate::nuc`]; each expression kind by the module named after it in
//! [`crate::lang::ast`].

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{Graph, NodeKey};
use crate::lang::ast::{Expr, Stmt};
use crate::lang::evaluation::{Flow, Tracking};
use crate::nuc::expression::is_assertion;
use crate::nuc::throw;
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

/// Where an assignment writes to.
enum Target {
    Local(String),
    Variable(String),
    Property {
        object: ObjectId,
        property: String,
    },
    /// A class-level declaration such as `$Person.mortal`.
    ClassProperty {
        class: String,
        property: String,
    },
}

pub struct Runtime {
    pub state: State,
    pub graph: Graph,
    pub(crate) transaction: Transaction,
    pub(crate) stack: Stack,
    pub(crate) assertions: Vec<AssertionFailure>,
    pub(crate) assertions_run: usize,
    pub(crate) tracking: Vec<Tracking>,
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
            tracking: Vec::new(),
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

    pub fn clear(&mut self) {
        self.state.clear();
        self.graph = Graph::new();
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

        let result = self.execute_inner(statement, scope);
        self.depth -= 1;
        result
    }

    /// Dispatches to the module named after the statement kind, matching
    /// `ref/src/nuc`.
    fn execute_inner(&mut self, statement: &Stmt, scope: &mut Scope) -> Result<Flow> {
        match statement {
            Stmt::Pass => Ok(Flow::Normal(Value::Null)),

            Stmt::Declaration { .. } => Err(Error::reference("Missing definition")),

            Stmt::Expression(expression) => {
                let value = self.evaluate(expression, scope)?;
                Ok(Flow::Normal(value))
            }

            Stmt::Assign { target, value } => {
                let assigned = self.assign(target, value, scope)?;
                Ok(Flow::Normal(assigned))
            }

            Stmt::Throw(expression) => self.run_throw(expression, scope),

            Stmt::Return(expression) => self.run_return(expression.as_ref(), scope),

            Stmt::Delete(expression) => {
                let value = self.delete(expression, scope)?;
                Ok(Flow::Normal(value))
            }

            Stmt::Function(function) => {
                self.declare_function(function)?;
                Ok(Flow::Normal(Value::Null))
            }

            Stmt::Class(declaration) => {
                self.define_class(declaration)?;
                Ok(Flow::Normal(Value::Null))
            }

            Stmt::Try {
                body,
                parameter,
                catch,
            } => self.run_try(body, parameter, catch, scope),

            Stmt::If { .. } => self.declare_if(statement, scope),

            Stmt::Block(statements) => self.declare_block(statement, statements, scope),

            Stmt::For {
                variable,
                iterable,
                body,
            } => self.run_for(variable, iterable, body, scope),
        }
    }

    /// `try`/`catch`. `ref` has no node for this — it is the one statement kind
    /// here that is run rather than filed, because a caught failure has to put
    /// back exactly what the failing branch changed and nothing else.
    fn run_try(
        &mut self,
        body: &[Stmt],
        parameter: &str,
        catch: &[Stmt],
        scope: &mut Scope,
    ) -> Result<Flow> {
        let mark = self.transaction.mark();

        match self.execute_all(body, scope) {
            Ok(flow) => Ok(flow),
            Err(error) => {
                self.transaction
                    .rollback_to(mark, &mut self.state, &mut self.graph);

                scope.push();
                scope.declare(parameter.to_string(), throw::caught(&error));
                let result = self.execute_all(catch, scope);
                scope.pop();
                result
            }
        }
    }

    // -- assignment ----------------------------------------------------------

    fn assign(&mut self, target: &Expr, value: &Expr, scope: &mut Scope) -> Result<Value> {
        match self.resolve_target(target, scope)? {
            Target::Local(name) => self.assign_local(name, value, scope),
            Target::Variable(name) => self.assign_variable(name, value, scope),
            Target::Property { object, property } => {
                self.assign_property(object, property, value, scope)
            }
            Target::ClassProperty { class, property } => {
                self.assign_class_property(class, property, value, scope)
            }
        }
    }

    fn resolve_target(&mut self, target: &Expr, scope: &mut Scope) -> Result<Target> {
        match target {
            Expr::Identifier(name) => {
                // Only names already bound as locals — parameters, loop
                // variables — stay local. Everything else is a state
                // assignment, so a block can define what the statements after
                // it depend on.
                if scope.has(name) || scope.assigned_by_enclosing(name) {
                    Ok(Target::Local(name.clone()))
                } else {
                    scope.record_assignment(name);
                    Ok(Target::Variable(name.clone()))
                }
            }

            Expr::Member { object, property } => {
                if let Expr::ClassRef(class) = object.as_ref() {
                    if scope.instance().is_none() {
                        if !self.state.classes.contains_key(class) {
                            return Err(Error::not_defined(class));
                        }
                        return Ok(Target::ClassProperty {
                            class: class.clone(),
                            property: property.clone(),
                        });
                    }
                }

                let base = self.evaluate(object, scope)?;

                match base {
                    Value::Object(id) => Ok(Target::Property {
                        object: id,
                        property: property.clone(),
                    }),
                    Value::Undefined | Value::Null => {
                        Err(Error::not_defined(self.describe(object, scope)))
                    }
                    Value::Class(class) => Ok(Target::ClassProperty {
                        class,
                        property: property.clone(),
                    }),
                    _ => Err(Error::type_error(format!(
                        "Cannot assign '{property}' on {}",
                        base.type_name()
                    ))),
                }
            }

            Expr::This => Err(Error::type_error("Cannot assign to 'this'")),

            other => Err(Error::syntax(format!("Cannot assign to {other}"))),
        }
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
