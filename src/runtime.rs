use indexmap::IndexSet;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::ast::{ClassDecl, Expr, FunctionBody, Stmt, TemplatePart};
use crate::error::{Error, Result};
use crate::graph::{Graph, GraphNode, NodeKey, NodeKind};
use crate::parser::parse_program;
use crate::scope::Scope;
use crate::state::{ClassData, Declaration, State};
use crate::transaction::Transaction;
use crate::value::{ObjectData, ObjectId, Value};

/// A failed `assert` recorded during a run.
#[derive(Debug, Clone)]
pub struct AssertionFailure {
    pub actual: Value,
    pub expected: Value,
}

/// How a statement finished: normally, or by returning out of its block.
pub(crate) enum Flow {
    Normal(Value),
    Return(Value),
}

/// The parts of a class a constructor needs, lifted out so creating an instance
/// does not copy the list of instances the class already has.
struct ClassShape {
    parent: Option<String>,
    parameters: Vec<crate::ast::Parameter>,
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

#[derive(Debug, Clone, Default)]
struct Tracking {
    keys: IndexSet<NodeKey>,
    barrier: bool,
}

pub struct Runtime {
    pub state: State,
    pub graph: Graph,
    pub(crate) transaction: Transaction,
    pub(crate) assertions: Vec<AssertionFailure>,
    pub(crate) assertions_run: usize,
    tracking: Vec<Tracking>,
    running: IndexSet<NodeKey>,
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
    /// Statements waiting to be re-evaluated, ordered by when they were
    /// declared, and the set of what is already waiting. Propagation is a queue
    /// rather than recursion, so the length of a dependency chain is not limited
    /// by the stack.
    pending: BinaryHeap<Reverse<(u64, NodeKey)>>,
    queued: IndexSet<NodeKey>,
    draining: bool,
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

/// How many statements one change may re-evaluate before the runtime decides it
/// is not settling. Cycles are refused when they are declared, so this is a
/// backstop that should never be reached — it is set low enough to fail quickly
/// rather than appear to hang.
const MAX_PROPAGATION: usize = 100_000;

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            state: State::new(),
            graph: Graph::new(),
            transaction: Transaction::new(),
            assertions: Vec::new(),
            assertions_run: 0,
            tracking: Vec::new(),
            running: IndexSet::new(),
            depth: 0,
            null_read: false,
            imperative: 0,
            undefined_read: false,
            instances: Vec::new(),
            deleted: IndexSet::new(),
            pending: BinaryHeap::new(),
            queued: IndexSet::new(),
            draining: false,
        }
    }

    /// Runs a program. Everything it changes is one transaction: if a statement
    /// raises, the state is left exactly as it was.
    pub fn run(&mut self, source: &str) -> Result<Value> {
        let program = parse_program(source)?;
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
        parse_program(source).map(|_| ())
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

            Stmt::Throw(expression) => {
                let value = self.evaluate(expression, scope)?;
                Err(Error::thrown(value))
            }

            Stmt::Return(expression) => {
                let value = match expression {
                    Some(expression) => self.evaluate(expression, scope)?,
                    None => Value::Null,
                };
                Ok(Flow::Return(value))
            }

            Stmt::Delete(expression) => {
                let value = self.delete(expression, scope)?;
                Ok(Flow::Normal(value))
            }

            Stmt::Function(function) => {
                if let Some(name) = &function.name {
                    let before = self.state.functions.get(name).cloned();
                    self.transaction.record_function(name, before);
                    self.state.functions.insert(name.clone(), function.clone());

                    let key = NodeKey::new(name.clone());
                    self.register(&key, NodeKind::Function, None, IndexSet::new(), None)?;
                    self.propagate(&key)?;
                }
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
            } => {
                let mark = self.transaction.mark();

                match self.execute_all(body, scope) {
                    Ok(flow) => Ok(flow),
                    Err(error) => {
                        // A caught exception still puts back what the failing
                        // branch changed, and nothing else.
                        self.transaction
                            .rollback_to(mark, &mut self.state, &mut self.graph);

                        scope.push();
                        scope.declare(parameter.clone(), error_value(&error));
                        let result = self.execute_all(catch, scope);
                        scope.pop();
                        result
                    }
                }
            }

            Stmt::If { .. } => self.declare_if(statement, scope),

            Stmt::Block(statements) => self.declare_block(statement, statements, scope),

            Stmt::For {
                variable,
                iterable,
                body,
            } => self.run_for(variable, iterable, body, scope),
        }
    }

    // -- assignment ----------------------------------------------------------

    fn assign(&mut self, target: &Expr, value: &Expr, scope: &mut Scope) -> Result<Value> {
        let target = self.resolve_target(target, scope)?;

        match target {
            Target::Local(name) => {
                let saved = self.null_read;
                self.null_read = false;
                self.push_tracking(false);
                let evaluated = self.evaluate(value, scope);
                let dependencies = self.pop_tracking();
                let evaluated = self.settle(evaluated?);
                self.null_read = saved;

                for dependency in dependencies {
                    self.track(dependency);
                }

                if !scope.assign(&name, evaluated.clone()) {
                    scope.declare(name, evaluated.clone());
                }

                Ok(evaluated)
            }

            Target::Variable(name) => {
                let key = NodeKey::new(name.clone());
                let value = &self.freeze(value, scope)?;
                let statement = Stmt::Assign {
                    target: Expr::Identifier(name.clone()),
                    value: value.clone(),
                };

                if self.imperative > 0 {
                    let (evaluated, _) = self.evaluate_tracked(value, scope, Some(&key))?;
                    self.set_variable(&name, evaluated.clone());
                    self.propagate(&key)?;
                    return Ok(evaluated);
                }

                if let Some((class, arguments)) = self.instantiation(value) {
                    let id = ObjectId::from(name.clone());
                    let created = self.create_instance(&class, &arguments, id, scope)?;
                    self.register(
                        &key,
                        NodeKind::Object,
                        Some(statement),
                        IndexSet::new(),
                        None,
                    )?;
                    self.set_variable(&name, created.clone());
                    self.propagate(&key)?;
                    return Ok(created);
                }

                let (evaluated, dependencies) = self.evaluate_tracked(value, scope, Some(&key))?;

                self.register(
                    &key,
                    NodeKind::Variable,
                    Some(statement),
                    dependencies,
                    None,
                )?;
                self.set_variable(&name, evaluated.clone());
                self.propagate(&key)?;

                Ok(evaluated)
            }

            Target::Property { object, property } => {
                if property == "value" {
                    return Err(Error::type_error("Cannot use 'value' as a property"));
                }

                let key = NodeKey::property(&object, &property);
                let value = &self.freeze(value, scope)?;
                let statement = Stmt::Assign {
                    target: Expr::Member {
                        object: Box::new(Expr::ObjectRef(object.to_string())),
                        property: property.clone(),
                    },
                    value: value.clone(),
                };

                if self.imperative > 0 && self.instantiation(value).is_none() {
                    let (evaluated, _) = self.evaluate_tracked(value, scope, Some(&key))?;
                    self.set_property(&object, &property, evaluated.clone());
                    self.propagate(&key)?;
                    return Ok(evaluated);
                }

                if let Some((class, arguments)) = self.instantiation(value) {
                    let id = ObjectId::from(format!("{object}.{property}"));
                    let created = self.create_instance(&class, &arguments, id, scope)?;
                    self.register(
                        &key,
                        NodeKind::Object,
                        Some(statement),
                        IndexSet::new(),
                        Some(object.clone()),
                    )?;
                    self.set_property(&object, &property, created.clone());
                    self.propagate(&key)?;
                    return Ok(created);
                }

                let (evaluated, dependencies) = self.evaluate_tracked(value, scope, Some(&key))?;

                self.register(
                    &key,
                    NodeKind::Property,
                    Some(statement),
                    dependencies,
                    Some(object.clone()),
                )?;
                self.set_property(&object, &property, evaluated.clone());
                self.propagate(&key)?;

                Ok(evaluated)
            }

            Target::ClassProperty { class, property } => {
                if property == "value" {
                    return Err(Error::type_error("Cannot use 'value' as a property"));
                }

                // A rule is checked where it is written, not when an instance
                // finally arrives to run it.
                self.check_rule_references(value, scope)?;

                let statement = Stmt::Assign {
                    target: Expr::Member {
                        object: Box::new(Expr::ClassRef(class.clone())),
                        property: property.clone(),
                    },
                    value: value.clone(),
                };

                self.declare_on_class(&class, format!("${class}.{property}"), statement)?;
                Ok(Value::Null)
            }
        }
    }

    /// Freezes the `.value` reads inside a statement, leaving its shape alone.
    fn freeze_statement(&mut self, statement: &Stmt, scope: &mut Scope) -> Result<Stmt> {
        Ok(match statement {
            Stmt::If {
                condition,
                consequent,
                alternate,
            } => Stmt::If {
                condition: self.freeze(condition, scope)?,
                consequent: consequent.clone(),
                alternate: alternate.clone(),
            },
            other => other.clone(),
        })
    }

    /// Replaces every `x.value` read with the value it has right now, so the
    /// stored declaration keeps that value instead of following `x` later.
    fn freeze(&mut self, expression: &Expr, scope: &mut Scope) -> Result<Expr> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(Error::type_error("Maximum expression depth exceeded"));
        }

        let result = self.freeze_inner(expression, scope);
        self.depth -= 1;
        result
    }

    fn freeze_inner(&mut self, expression: &Expr, scope: &mut Scope) -> Result<Expr> {
        Ok(match expression {
            Expr::Member { object, property } if property == "value" => {
                let value = self.evaluate(expression, scope)?;
                literal(&value).unwrap_or_else(|| Expr::Member {
                    object: object.clone(),
                    property: property.clone(),
                })
            }
            Expr::Member { object, property } => Expr::Member {
                object: Box::new(self.freeze(object, scope)?),
                property: property.clone(),
            },
            Expr::Binary {
                operator,
                left,
                right,
            } => Expr::Binary {
                operator: *operator,
                left: Box::new(self.freeze(left, scope)?),
                right: Box::new(self.freeze(right, scope)?),
            },
            Expr::Logical {
                operator,
                left,
                right,
            } => Expr::Logical {
                operator: *operator,
                left: Box::new(self.freeze(left, scope)?),
                right: Box::new(self.freeze(right, scope)?),
            },
            Expr::Unary { operator, operand } => Expr::Unary {
                operator: *operator,
                operand: Box::new(self.freeze(operand, scope)?),
            },
            Expr::Call { callee, arguments } => {
                let mut frozen = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    frozen.push(self.freeze(argument, scope)?);
                }
                Expr::Call {
                    callee: Box::new(self.freeze(callee, scope)?),
                    arguments: frozen,
                }
            }
            Expr::Index { object, index } => Expr::Index {
                object: Box::new(self.freeze(object, scope)?),
                index: Box::new(self.freeze(index, scope)?),
            },
            other => other.clone(),
        })
    }

    /// Recognises `Class(...)` on the right of an assignment, which names the
    /// new instance after what it is assigned to.
    fn instantiation(&self, value: &Expr) -> Option<(String, Vec<Expr>)> {
        let Expr::Call { callee, arguments } = value else {
            return None;
        };

        let Expr::Identifier(name) = callee.as_ref() else {
            return None;
        };

        if self.state.classes.contains_key(name) {
            Some((name.clone(), arguments.clone()))
        } else {
            None
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

    /// Renders a path for an error message, preferring the source spelling.
    pub(crate) fn describe(&self, expression: &Expr, _scope: &Scope) -> String {
        expression.path().unwrap_or_else(|| expression.to_string())
    }

    // -- declarations that re-run --------------------------------------------

    fn declare_if(&mut self, statement: &Stmt, scope: &mut Scope) -> Result<Flow> {
        let Stmt::If { condition, .. } = statement else {
            unreachable!("declare_if is only called with Stmt::If")
        };

        if let Some(class) = self.class_reference(statement, scope) {
            let key = format!("if({condition})");
            self.declare_on_class(&class, key, statement.clone())?;
            return Ok(Flow::Normal(Value::Null));
        }

        // `.value` in a condition means the value it has now, so the stored
        // rule keeps it rather than re-reading it on every run.
        let statement = &self.freeze_statement(statement, scope)?;

        let Stmt::If { condition, .. } = statement else {
            unreachable!("freeze_statement keeps the statement kind")
        };

        let key = match scope.instance() {
            Some(instance) => NodeKey::new(format!("if({condition})@{instance}")),
            None => NodeKey::new(format!("if({condition})")),
        };

        let instance = scope.instance().cloned();
        self.push_tracking(false);
        self.imperative += 1;
        let outcome = self.run_if(statement, scope);
        self.imperative -= 1;
        let dependencies = self.pop_tracking();
        let flow = outcome?;

        self.register(
            &key,
            NodeKind::If,
            Some(statement.clone()),
            dependencies,
            instance,
        )?;

        Ok(flow)
    }

    fn run_if(&mut self, statement: &Stmt, scope: &mut Scope) -> Result<Flow> {
        let Stmt::If {
            condition,
            consequent,
            alternate,
        } = statement
        else {
            unreachable!("run_if is only called with Stmt::If")
        };

        let saved = std::mem::take(&mut self.undefined_read);
        let test = self.evaluate(condition, scope);
        let unresolved = self.undefined_read;
        self.undefined_read = saved;
        let test = test?;

        // While a class-level rule still reads something undefined it stands
        // aside, rather than deciding on a value that is not there yet.
        if unresolved && scope.instance().is_some() {
            return Ok(Flow::Normal(Value::Null));
        }

        // Branch bodies run in the surrounding scope: only `{ }` introduces
        // locals, so an assignment in a branch is a state assignment.
        if test.truthy() {
            self.execute_all(consequent, scope)
        } else if let Some(alternate) = alternate {
            match alternate.as_ref() {
                Stmt::Block(statements) => self.execute_all(statements, scope),
                other => self.execute(other, scope),
            }
        } else {
            Ok(Flow::Normal(Value::Null))
        }
    }

    fn declare_block(
        &mut self,
        statement: &Stmt,
        statements: &[Stmt],
        scope: &mut Scope,
    ) -> Result<Flow> {
        if let Some(class) = self.class_reference(statement, scope) {
            // A rule states something about every instance, so it cannot be
            // built from the properties of one particular instance.
            if declares_class_rule(statements) && self.reads_an_instance(statements, scope) {
                return Err(Error::syntax(
                    "Cannot define class declaration in non-class block",
                ));
            }

            let key = format!("block({})", render_statements(statements));
            self.declare_on_class(&class, key, statement.clone())?;
            return Ok(Flow::Normal(Value::Null));
        }

        let rendered = render_statements(statements);
        let key = match scope.instance() {
            Some(instance) => NodeKey::new(format!("block({rendered})@{instance}")),
            None => NodeKey::new(format!("block({rendered})")),
        };

        let instance = scope.instance().cloned();

        self.push_tracking(false);
        scope.push();
        self.imperative += 1;
        let outcome = self.execute_all(statements, scope);
        self.imperative -= 1;
        scope.pop();
        let dependencies = self.pop_tracking();
        let flow = outcome?;

        self.register(
            &key,
            NodeKind::Block,
            Some(statement.clone()),
            dependencies,
            instance,
        )?;

        Ok(flow)
    }

    fn run_for(
        &mut self,
        variable: &str,
        iterable: &Expr,
        body: &[Stmt],
        scope: &mut Scope,
    ) -> Result<Flow> {
        let items = self.iterate(iterable, scope)?;

        for item in items {
            scope.push();
            scope.declare(variable.to_string(), item);
            self.imperative += 1;
            let result = self.execute_all(body, scope);
            self.imperative -= 1;
            scope.pop();

            if let Flow::Return(value) = result? {
                return Ok(Flow::Return(value));
            }
        }

        Ok(Flow::Normal(Value::Null))
    }

    /// The objects a `for ... of` walks: instances of a class, or the instances
    /// found in a list. Plain values in a list are skipped.
    fn iterate(&mut self, iterable: &Expr, scope: &mut Scope) -> Result<Vec<Value>> {
        let value = self.evaluate(iterable, scope)?;

        Ok(match value {
            Value::Class(name) => self
                .state
                .class(&name)
                .map(|class| class.instances.clone())
                .unwrap_or_default()
                .into_iter()
                .map(Value::Object)
                .collect(),
            Value::List(items) => items
                .into_iter()
                .filter(|item| match item {
                    Value::Object(id) => self
                        .state
                        .object(id)
                        .and_then(|object| object.class.as_ref())
                        .is_some(),
                    _ => false,
                })
                .collect(),
            _other => {
                return Err(Error::type_error(format!(
                    "{other} is not iterable",
                    other = iterable
                )));
            }
        })
    }

    // -- class declarations --------------------------------------------------

    /// Reports a name a class-level rule reads that nothing has defined.
    fn check_rule_references(&self, value: &Expr, scope: &Scope) -> Result<()> {
        let mut roots = Vec::new();
        collect_roots(value, &mut roots);

        for root in roots {
            let known = scope.has(&root)
                || self.state.variables.contains_key(&root)
                || self.state.classes.contains_key(&root)
                || self.state.functions.contains_key(&root)
                || crate::builtins::is_global(&root);

            if !known {
                return Err(Error::not_defined(root));
            }
        }

        Ok(())
    }

    /// Whether a block reads a named instance from the state, rather than only
    /// its own locals and the type it is stating a rule about.
    fn reads_an_instance(&self, statements: &[Stmt], scope: &Scope) -> bool {
        let mut assigned = Vec::new();
        for statement in statements {
            collect_assigned_names(statement, &mut assigned);
        }

        let mut roots = Vec::new();
        for statement in statements {
            collect_read_roots(statement, &mut roots);
        }

        roots.iter().any(|root| {
            !assigned.contains(root)
                && !scope.has(root)
                && matches!(self.state.variables.get(root), Some(Value::Object(_)))
        })
    }

    /// The class a statement declares against, if it mentions `$Class` outside
    /// of an instance scope.
    fn class_reference(&self, statement: &Stmt, scope: &Scope) -> Option<String> {
        if scope.instance().is_some() {
            return None;
        }

        let mut found = None;
        find_class_reference_statement(statement, &mut found);

        found.filter(|name| self.state.classes.contains_key(name))
    }

    fn declare_on_class(&mut self, class: &str, key: String, statement: Stmt) -> Result<()> {
        // `$Class.property = Other()` creates one object that every instance
        // shares, rather than one per instance.
        let statement = match &statement {
            Stmt::Assign { target, value } => match (target, self.instantiation(value)) {
                (
                    Expr::Member {
                        object,
                        property: name,
                    },
                    Some((instantiated, arguments)),
                ) if matches!(object.as_ref(), Expr::ClassRef(_)) => {
                    let id = ObjectId::from(format!("${class}.{name}"));
                    let mut scope = Scope::new();
                    self.create_instance(&instantiated, &arguments, id.clone(), &mut scope)?;

                    Stmt::Assign {
                        target: target.clone(),
                        value: Expr::ObjectRef(id.to_string()),
                    }
                }
                _ => statement.clone(),
            },
            other => other.clone(),
        };

        self.check_class_cycle(class, &statement)?;

        let sequence = self.graph.next_sequence();

        let Some(data) = self.state.class(class) else {
            return Err(Error::not_defined(class));
        };

        let before = data.clone();
        let instances = data.instances.clone();

        self.transaction.record_class(class, Some(before));

        if let Some(data) = self.state.class_mut(class) {
            data.declarations.shift_remove(&key);
            data.declarations.insert(
                key.clone(),
                Declaration {
                    key,
                    statement: statement.clone(),
                    sequence,
                },
            );
        }

        for instance in instances {
            self.apply_declaration(&statement, &instance)?;
        }

        Ok(())
    }

    /// Rejects a class-level rule that would make two properties of the same
    /// class depend on each other, before any instance exists to reveal it.
    fn check_class_cycle(&self, class: &str, statement: &Stmt) -> Result<()> {
        let Stmt::Assign { target, value } = statement else {
            return Ok(());
        };

        let Expr::Member { object, property } = target else {
            return Ok(());
        };

        if !matches!(object.as_ref(), Expr::ClassRef(name) if name == class) {
            return Ok(());
        }

        let Some(data) = self.state.class(class) else {
            return Ok(());
        };

        let mut pending = Vec::new();
        collect_class_properties(value, class, &mut pending);

        let mut seen: Vec<String> = Vec::new();

        while let Some(current) = pending.pop() {
            if &current == property {
                return Err(Error::type_error("Circular Dependency"));
            }

            if seen.contains(&current) {
                continue;
            }

            seen.push(current.clone());

            if let Some(declaration) = data.declarations.get(&format!("${class}.{current}")) {
                if let Stmt::Assign { value, .. } = &declaration.statement {
                    collect_class_properties(value, class, &mut pending);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn apply_declaration(
        &mut self,
        statement: &Stmt,
        instance: &ObjectId,
    ) -> Result<()> {
        let mut scope = Scope::new();
        scope.set_instance(Some(instance.clone()));

        // A class-level statement is a declaration wherever the instantiation
        // that triggered it happened to be written.
        let nested = std::mem::take(&mut self.imperative);
        self.instances.push(instance.clone());
        self.push_tracking(true);
        let result = self.execute(statement, &mut scope);
        self.pop_tracking();
        self.instances.pop();
        self.imperative = nested;

        result.map(|_| ())
    }

    fn define_class(&mut self, declaration: &ClassDecl) -> Result<()> {
        let existing = self.state.class(&declaration.name).cloned();
        self.transaction
            .record_class(&declaration.name, existing.clone());

        let mut data = ClassData::new(declaration.name.clone());
        data.parent = declaration.parent.clone();
        data.parameters = declaration.parameters.clone();
        data.constructor = declaration.constructor.clone();

        for method in &declaration.methods {
            if let Some(name) = &method.name {
                data.methods.insert(name.clone(), method.clone());
            }
        }

        // A redeclared class keeps its instances and class-level declarations.
        if let Some(existing) = existing {
            data.instances = existing.instances;
            data.declarations = existing.declarations;

            if data.parameters.is_empty() && data.constructor.is_empty() {
                if let Some(initializer) = data.methods.get("init").cloned() {
                    data.parameters = initializer.parameters.clone();

                    if let FunctionBody::Block(body) = &initializer.body {
                        data.constructor = body.clone();
                    }
                }
            }
        }

        if data.constructor.is_empty() {
            if let Some(initializer) = data.methods.get("init").cloned() {
                data.parameters = initializer.parameters.clone();

                if let FunctionBody::Block(body) = &initializer.body {
                    data.constructor = body.clone();
                }
            }
        }

        self.state
            .classes
            .insert(declaration.name.clone(), data.clone());

        let key = NodeKey::new(format!("${}", declaration.name));
        self.register(&key, NodeKind::Class, None, IndexSet::new(), None)?;

        Ok(())
    }

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

        // Only the parts a constructor needs, so the class's instance list is
        // not copied once per instance.
        let shape = ClassShape {
            parent: class.parent.clone(),
            parameters: class.parameters.clone(),
            constructor: class.constructor.clone(),
        };

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

    pub(crate) fn declarations_for(&self, class_name: &str) -> Vec<Declaration> {
        let mut declarations = Vec::new();
        let mut chain = Vec::new();
        let mut current = Some(class_name.to_string());

        while let Some(name) = current {
            let Some(class) = self.state.class(&name) else {
                break;
            };
            chain.push(class);
            current = class.parent.clone();
        }

        for class in chain.into_iter().rev() {
            declarations.extend(class.declarations_in_order());
        }

        declarations.sort_by_key(|declaration| declaration.sequence);
        declarations
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

    // -- graph bookkeeping ---------------------------------------------------

    fn register(
        &mut self,
        key: &NodeKey,
        kind: NodeKind,
        statement: Option<Stmt>,
        dependencies: IndexSet<NodeKey>,
        instance: Option<ObjectId>,
    ) -> Result<()> {
        let existing = self.graph.get(key).cloned();

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

        if self.transaction.needs_node(key) {
            self.transaction.record_node(key, existing.clone());
        }

        let sequence = self.graph.next_sequence();
        let mut node = GraphNode::new(key.clone(), kind, sequence);
        node.statement = statement;
        node.instance = instance;
        node.dependencies = dependencies.clone();

        if let Some(existing) = &existing {
            node.dependents = existing.dependents.clone();

            // Drop edges from dependencies this declaration no longer has.
            for previous in &existing.dependencies {
                if !dependencies.contains(previous) {
                    let removed = self
                        .graph
                        .get_mut(previous)
                        .is_some_and(|source| source.dependents.shift_remove(key));

                    if removed {
                        self.transaction.record_dependent_removed(previous, key);
                    }
                }
            }
        }

        self.graph.insert(node);

        for dependency in &dependencies {
            if dependency == key {
                continue;
            }

            if !self.graph.contains(dependency) {
                let sequence = self.graph.next_sequence();
                let pending = GraphNode::new(dependency.clone(), NodeKind::Pending, sequence);
                self.transaction.record_node(dependency, None);
                self.graph.insert(pending);
            }

            let added = self
                .graph
                .get_mut(dependency)
                .is_some_and(|source| source.dependents.insert(key.clone()));

            if added {
                self.transaction.record_dependent_added(dependency, key);
            }
        }

        Ok(())
    }

    /// Notes that everything reading `key` is now out of date. The work is done
    /// by whoever is draining the queue, which keeps propagation flat however
    /// long the chain is.
    pub(crate) fn propagate(&mut self, key: &NodeKey) -> Result<()> {
        for dependent in self.graph.dependents_in_order(key) {
            self.enqueue(dependent);
        }

        if self.draining {
            return Ok(());
        }

        self.draining = true;
        let result = self.drain();
        self.draining = false;

        if result.is_err() {
            self.pending.clear();
            self.queued.clear();
        }

        result
    }

    fn enqueue(&mut self, key: NodeKey) {
        // A statement does not re-run itself: what it writes while running is
        // its own doing, not news to it.
        if self.running.contains(&key) {
            return;
        }

        let sequence = self
            .graph
            .get(&key)
            .map(|node| node.sequence)
            .unwrap_or(u64::MAX);

        if self.queued.insert(key.clone()) {
            self.pending.push(Reverse((sequence, key)));
        }
    }

    /// Re-evaluates what is waiting, in the order it was declared, until nothing
    /// is left. A statement re-run here may queue more work, which this same
    /// loop picks up.
    fn drain(&mut self) -> Result<()> {
        // Once each. A statement that writes something it also reads would
        // otherwise wake itself for as long as it kept changing.
        let mut settled: IndexSet<NodeKey> = IndexSet::new();
        let mut steps = 0usize;

        while let Some(Reverse((_, dependent))) = self.pending.pop() {
            self.queued.swap_remove(&dependent);

            steps += 1;

            if steps > MAX_PROPAGATION {
                return Err(Error::type_error("Propagation did not settle"));
            }

            if !settled.insert(dependent.clone()) {
                continue;
            }

            let Some(node) = self.graph.get(&dependent).cloned() else {
                continue;
            };

            let Some(statement) = node.statement.clone() else {
                continue;
            };

            self.running.insert(dependent.clone());

            let mut scope = Scope::new();
            scope.set_instance(node.instance.clone());

            let nested = std::mem::take(&mut self.imperative);
            if let Some(instance) = &node.instance {
                self.instances.push(instance.clone());
            }
            self.push_tracking(true);
            let result = self.execute(&statement, &mut scope);
            self.pop_tracking();
            if node.instance.is_some() {
                self.instances.pop();
            }
            self.imperative = nested;

            self.running.shift_remove(&dependent);
            result?;
        }

        Ok(())
    }

    // -- state writes --------------------------------------------------------

    pub(crate) fn set_variable(&mut self, name: &str, value: Value) {
        let before = self.state.variables.get(name).cloned();
        self.transaction.record_variable(name, before);
        self.state.variables.insert(name.to_string(), value);
    }

    pub(crate) fn set_property(&mut self, object: &ObjectId, property: &str, value: Value) {
        if !self.state.objects.contains_key(object) {
            self.transaction.record_object(object, None);
            self.state
                .objects
                .insert(object.clone(), ObjectData::new(None));
        }

        let before = self.state.property(object, property).cloned();
        self.transaction.record_property(object, property, before);

        if let Some(data) = self.state.object_mut(object) {
            data.properties.insert(property.to_string(), value);
        }
    }

    pub(crate) fn note_nullish(&mut self, value: &Value) {
        if value.is_null() {
            self.null_read = true;
        }
    }

    /// Applies the null and undefined rules to a value about to be stored.
    fn settle(&self, value: Value) -> Value {
        if value.is_undefined() || self.null_read {
            Value::Null
        } else {
            value
        }
    }

    // -- dependency tracking -------------------------------------------------

    pub(crate) fn push_tracking(&mut self, barrier: bool) {
        self.tracking.push(Tracking {
            keys: IndexSet::new(),
            barrier,
        });
    }

    pub(crate) fn pop_tracking(&mut self) -> IndexSet<NodeKey> {
        self.tracking
            .pop()
            .map(|frame| frame.keys)
            .unwrap_or_default()
    }

    /// Records a read. Reads reach every enclosing frame up to a barrier, so an
    /// enclosing block depends on whatever its statements read.
    pub(crate) fn track(&mut self, key: NodeKey) {
        for frame in self.tracking.iter_mut().rev() {
            frame.keys.insert(key.clone());

            if frame.barrier {
                break;
            }
        }
    }

    pub(crate) fn evaluate_tracked(
        &mut self,
        expression: &Expr,
        scope: &mut Scope,
        exclude: Option<&NodeKey>,
    ) -> Result<(Value, IndexSet<NodeKey>)> {
        let saved = self.null_read;
        self.null_read = false;
        self.push_tracking(false);
        let value = self.evaluate(expression, scope);
        let mut dependencies = self.pop_tracking();
        let value = self.settle(value?);
        self.null_read = saved;

        if let Some(exclude) = exclude {
            dependencies.shift_remove(exclude);
        }

        // Reads made while evaluating still belong to any enclosing declaration.
        for dependency in &dependencies {
            self.track(dependency.clone());
        }

        Ok((value, dependencies))
    }

    // -- deletion ------------------------------------------------------------

    pub(crate) fn delete(&mut self, expression: &Expr, scope: &mut Scope) -> Result<Value> {
        match expression {
            Expr::Identifier(name) => {
                let key = NodeKey::new(name.clone());

                let Some(value) = self.state.variables.get(name).cloned() else {
                    return Ok(Value::Bool(false));
                };

                if let Value::Object(id) = &value {
                    self.delete_object(id)?;
                }

                self.transaction.record_variable(name, Some(value));
                self.state.variables.shift_remove(name);
                self.remove_node(&key);
                self.cascade_removal(&key)?;

                Ok(Value::Bool(true))
            }

            Expr::Member { object, property } => {
                if let Expr::ClassRef(class) = object.as_ref() {
                    if scope.instance().is_none() {
                        return self.delete_declaration(class, property);
                    }
                }

                let base = self.evaluate(object, scope)?;

                let Value::Object(id) = base else {
                    return Ok(Value::Bool(false));
                };

                let before = self.state.property(&id, property).cloned();

                if before.is_none() {
                    return Ok(Value::Bool(false));
                }

                self.transaction.record_property(&id, property, before);

                if let Some(data) = self.state.object_mut(&id) {
                    data.properties.shift_remove(property);
                }

                let key = NodeKey::property(&id, property);
                self.remove_node(&key);
                self.cascade_removal(&key)?;

                Ok(Value::Bool(true))
            }

            Expr::Index { .. } => {
                let value = self.evaluate(expression, scope)?;

                let Value::Object(id) = value else {
                    return Ok(Value::Bool(false));
                };

                self.delete_object(&id)?;

                let key = NodeKey::new(id.to_string());
                self.remove_node(&key);

                if self.state.variables.get(id.as_str()).is_some() {
                    let before = self.state.variables.get(id.as_str()).cloned();
                    self.transaction.record_variable(id.as_str(), before);
                    self.state.variables.shift_remove(id.as_str());
                }

                self.cascade_removal(&key)?;

                Ok(Value::Bool(true))
            }

            other => Err(Error::syntax(format!("Cannot delete {other}"))),
        }
    }

    /// Removes a class-level rule and the values it produced.
    fn delete_declaration(&mut self, class: &str, property: &str) -> Result<Value> {
        let key = format!("${class}.{property}");

        let Some(data) = self.state.class(class).cloned() else {
            return Err(Error::not_defined(class));
        };

        if !data.declarations.contains_key(&key) {
            return Ok(Value::Bool(false));
        }

        self.transaction.record_class(class, Some(data.clone()));

        if let Some(data) = self.state.class_mut(class) {
            data.declarations.shift_remove(&key);
        }

        for instance in data.instances {
            let node = NodeKey::property(&instance, property);
            self.remove_node(&node);

            if self.state.property(&instance, property).is_some() {
                self.set_property(&instance, property, Value::Null);
            }
        }

        Ok(Value::Bool(true))
    }

    /// Removes an instance, refusing while it still carries properties.
    fn delete_object(&mut self, id: &ObjectId) -> Result<()> {
        let Some(data) = self.state.object(id).cloned() else {
            return Ok(());
        };

        let remaining = data
            .properties
            .iter()
            .filter(|(name, _)| name.as_str() != "id")
            .count();

        if remaining > 0 {
            return Err(Error::type_error(format!("Cannot delete object '{id}'")));
        }

        let data_class = data.class.clone();

        if let Some(class_name) = &data_class {
            if let Some(class) = self.state.class(class_name).cloned() {
                self.transaction.record_class(class_name, Some(class));
            }

            if let Some(class) = self.state.class_mut(class_name) {
                class.instances.retain(|instance| instance != id);
            }
        }

        self.transaction.record_object(id, Some(data));
        self.state.objects.shift_remove(id);

        if let Some(class_name) = data_class {
            self.propagate(&NodeKey::new(format!("${class_name}")))?;
        }

        Ok(())
    }

    fn remove_node(&mut self, key: &NodeKey) {
        if let Some(node) = self.graph.get(key).cloned() {
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
        }
    }

    /// Re-runs everything that read what was just removed. Each dependent finds
    /// the name undefined, settles to null, and passes that on to its own
    /// dependents, so a whole chain clears.
    fn cascade_removal(&mut self, key: &NodeKey) -> Result<()> {
        self.deleted.insert(key.clone());
        let outermost = !self.draining;
        let result = self.propagate(key);

        // Only the run that actually drained the queue is finished with it; a
        // nested deletion leaves the name undefined until the outer drain ends.
        if outermost {
            self.deleted.shift_remove(key);
        }

        result
    }
}

/// `assert` is a test helper, so it does not become the program's result.
fn is_assertion(statement: &Stmt) -> bool {
    let Stmt::Expression(Expr::Call { callee, .. }) = statement else {
        return false;
    };

    matches!(callee.as_ref(), Expr::Identifier(name) if name == "assert")
}

/// Whether any statement states a rule about a type, as `$Class.property = ...`.
fn declares_class_rule(statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| match statement {
        Stmt::Assign { target, .. } => matches!(
            target,
            Expr::Member { object, .. } if matches!(object.as_ref(), Expr::ClassRef(_))
        ),
        Stmt::Block(nested) => declares_class_rule(nested),
        Stmt::If {
            consequent,
            alternate,
            ..
        } => {
            declares_class_rule(consequent)
                || alternate
                    .as_ref()
                    .is_some_and(|statement| declares_class_rule(std::slice::from_ref(statement)))
        }
        _ => false,
    })
}

/// The bare names a statement assigns to.
fn collect_assigned_names(statement: &Stmt, names: &mut Vec<String>) {
    match statement {
        Stmt::Assign {
            target: Expr::Identifier(name),
            ..
        } => names.push(name.clone()),
        Stmt::Block(nested) => {
            for statement in nested {
                collect_assigned_names(statement, names);
            }
        }
        Stmt::If {
            consequent,
            alternate,
            ..
        } => {
            for statement in consequent {
                collect_assigned_names(statement, names);
            }
            if let Some(alternate) = alternate {
                collect_assigned_names(alternate, names);
            }
        }
        _ => {}
    }
}

/// The leftmost names a statement reads.
fn collect_read_roots(statement: &Stmt, roots: &mut Vec<String>) {
    match statement {
        Stmt::Assign { target, value } => {
            if let Expr::Member { object, .. } = target {
                collect_roots(object, roots);
            }
            collect_roots(value, roots);
        }
        Stmt::Expression(expression) | Stmt::Throw(expression) | Stmt::Delete(expression) => {
            collect_roots(expression, roots)
        }
        Stmt::Return(Some(expression)) => collect_roots(expression, roots),
        Stmt::Block(nested) => {
            for statement in nested {
                collect_read_roots(statement, roots);
            }
        }
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            collect_roots(condition, roots);
            for statement in consequent {
                collect_read_roots(statement, roots);
            }
            if let Some(alternate) = alternate {
                collect_read_roots(alternate, roots);
            }
        }
        _ => {}
    }
}

fn collect_roots(expression: &Expr, roots: &mut Vec<String>) {
    match expression {
        Expr::Identifier(name) => roots.push(name.clone()),
        Expr::Member { object, .. } | Expr::Slice { object, .. } => collect_roots(object, roots),
        Expr::Index { object, index } => {
            collect_roots(object, roots);
            collect_roots(index, roots);
        }
        Expr::Call { callee, arguments } => {
            collect_roots(callee, roots);
            for argument in arguments {
                collect_roots(argument, roots);
            }
        }
        Expr::Unary { operand, .. } => collect_roots(operand, roots),
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            collect_roots(left, roots);
            collect_roots(right, roots);
        }
        Expr::List(items) => {
            for item in items {
                collect_roots(item, roots);
            }
        }
        Expr::ObjectLiteral(entries) => {
            for (_, value) in entries {
                collect_roots(value, roots);
            }
        }
        Expr::Template(parts) => {
            for part in parts {
                if let TemplatePart::Expression(expression) = part {
                    collect_roots(expression, roots);
                }
            }
        }
        Expr::Assign { target, value } => {
            collect_roots(target, roots);
            collect_roots(value, roots);
        }
        _ => {}
    }
}

/// The `$Class.property` names an expression reads.
fn collect_class_properties(expression: &Expr, class: &str, found: &mut Vec<String>) {
    match expression {
        Expr::Member { object, property } => {
            if matches!(object.as_ref(), Expr::ClassRef(name) if name == class) {
                found.push(property.clone());
            } else {
                collect_class_properties(object, class, found);
            }
        }
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            collect_class_properties(left, class, found);
            collect_class_properties(right, class, found);
        }
        Expr::Unary { operand, .. } => collect_class_properties(operand, class, found),
        Expr::Call { callee, arguments } => {
            collect_class_properties(callee, class, found);
            for argument in arguments {
                collect_class_properties(argument, class, found);
            }
        }
        Expr::Index { object, index } => {
            collect_class_properties(object, class, found);
            collect_class_properties(index, class, found);
        }
        _ => {}
    }
}

/// The literal expression for a value, where one exists.
fn literal(value: &Value) -> Option<Expr> {
    Some(match value {
        Value::Null | Value::Undefined => Expr::Null,
        Value::Bool(bool) => Expr::Bool(*bool),
        Value::Number(number) => Expr::Number(*number),
        Value::String(string) => Expr::String(string.clone()),
        Value::Object(id) => Expr::ObjectRef(id.to_string()),
        _ => return None,
    })
}

/// The value a `catch` binds. Positions are deliberately left out: this text is
/// part of the language's behaviour and is compared against in programs.
fn error_value(error: &Error) -> Value {
    match error.thrown_value() {
        Some(value) => value.clone(),
        None => Value::String(format!("{}: {}", error.kind().as_str(), error.message())),
    }
}

fn render_statements(statements: &[Stmt]) -> String {
    statements
        .iter()
        .map(render_statement)
        .collect::<Vec<_>>()
        .join(";")
}

fn render_statement(statement: &Stmt) -> String {
    match statement {
        Stmt::Assign { target, value } => format!("{target}={value}"),
        Stmt::Expression(expression) => expression.to_string(),
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            let alternate = alternate
                .as_ref()
                .map(|statement| format!("else{{{}}}", render_statement(statement)))
                .unwrap_or_default();
            format!(
                "if({condition}){{{}}}{alternate}",
                render_statements(consequent)
            )
        }
        Stmt::Block(statements) => format!("{{{}}}", render_statements(statements)),
        Stmt::Return(Some(expression)) => format!("return {expression}"),
        Stmt::Return(None) => "return".to_string(),
        Stmt::Throw(expression) => format!("throw {expression}"),
        Stmt::Delete(expression) => format!("delete {expression}"),
        Stmt::Class(declaration) => format!("class {}", declaration.name),
        Stmt::Function(function) => match &function.name {
            Some(name) => format!("def {name}"),
            None => "def".to_string(),
        },
        Stmt::For {
            variable, iterable, ..
        } => format!("for {variable} of {iterable}"),
        Stmt::Try { .. } => "try".to_string(),
        Stmt::Declaration { name, type_name } => format!("{name}:{type_name}"),
        Stmt::Pass => "pass".to_string(),
    }
}

fn find_class_reference_statement(statement: &Stmt, found: &mut Option<String>) {
    if found.is_some() {
        return;
    }

    match statement {
        Stmt::Assign { target, value } => {
            find_class_reference(target, found);
            find_class_reference(value, found);
        }
        Stmt::Expression(expression) | Stmt::Throw(expression) | Stmt::Delete(expression) => {
            find_class_reference(expression, found)
        }
        Stmt::Return(Some(expression)) => find_class_reference(expression, found),
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            find_class_reference(condition, found);
            for statement in consequent {
                find_class_reference_statement(statement, found);
            }
            if let Some(alternate) = alternate {
                find_class_reference_statement(alternate, found);
            }
        }
        Stmt::Block(statements) => {
            for statement in statements {
                find_class_reference_statement(statement, found);
            }
        }
        _ => {}
    }
}

fn find_class_reference(expression: &Expr, found: &mut Option<String>) {
    if found.is_some() {
        return;
    }

    match expression {
        Expr::ClassRef(name) => *found = Some(name.clone()),
        Expr::Member { object, .. } => find_class_reference(object, found),
        Expr::Index { object, index } => {
            find_class_reference(object, found);
            find_class_reference(index, found);
        }
        Expr::Slice { object, start, end } => {
            find_class_reference(object, found);
            if let Some(start) = start {
                find_class_reference(start, found);
            }
            if let Some(end) = end {
                find_class_reference(end, found);
            }
        }
        Expr::Call { callee, arguments } => {
            find_class_reference(callee, found);
            for argument in arguments {
                find_class_reference(argument, found);
            }
        }
        Expr::Unary { operand, .. } => find_class_reference(operand, found),
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            find_class_reference(left, found);
            find_class_reference(right, found);
        }
        Expr::List(items) => {
            for item in items {
                find_class_reference(item, found);
            }
        }
        Expr::ObjectLiteral(entries) => {
            for (_, value) in entries {
                find_class_reference(value, found);
            }
        }
        Expr::Template(parts) => {
            for part in parts {
                if let TemplatePart::Expression(expression) = part {
                    find_class_reference(expression, found);
                }
            }
        }
        Expr::Assign { target, value } => {
            find_class_reference(target, found);
            find_class_reference(value, found);
        }
        Expr::Function(function) => {
            if let FunctionBody::Expression(body) = &function.body {
                find_class_reference(body, found);
            }
        }
        _ => {}
    }
}
