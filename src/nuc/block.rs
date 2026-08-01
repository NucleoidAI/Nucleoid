//! `BLOCK` — `{ ... }`, a run of statements filed as one declaration and
//! re-evaluated whenever anything it read changes. Mirrors
//! `ref/src/nuc/BLOCK.js`, which likewise keeps its statements on the node and
//! runs them in a scope of their own.

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{Expr, Stmt, collect_assigned_names, collect_read_roots};
use crate::nuc::{Nuc, Outcome};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::state::DeclarationKey;
use crate::value::{ObjectId, Value};

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
    /// Set while running: where the block is filed, and the instance whose
    /// rules it belongs to. A block declared against a class is kept on the
    /// class instead and files nothing of its own.
    key: Option<NodeKey>,
    instance: Option<ObjectId>,
}

impl Block {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Block {
            statements,
            key: None,
            instance: None,
        }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let statement = Stmt::Block(self.statements.clone());

        if let Some(class) = runtime.class_reference(&statement, scope) {
            // A rule states something about every instance, so it cannot be
            // built from the properties of one particular instance.
            if declares_class_rule(&self.statements) && self.reads_an_instance(runtime, scope) {
                return Err(Error::syntax(
                    "Cannot define class declaration in non-class block",
                ));
            }

            let key = DeclarationKey::block(&self.statements);
            runtime.declare_on_class(&class, key, statement)?;
            return Ok(Outcome::null());
        }

        self.instance = scope.instance().cloned();
        self.key = Some(NodeKey::block(&self.statements, self.instance.as_ref()));

        runtime.push_tracking(false);
        scope.push();
        runtime.enter_imperative();
        let outcome = runtime.execute_all(&self.statements, scope);
        runtime.leave_imperative();
        scope.pop();
        let dependencies = runtime.pop_tracking();

        Ok(Outcome {
            flow: outcome?,
            dependencies,
        })
    }

    pub fn graph(&self, runtime: &mut Runtime, dependencies: IndexSet<NodeKey>) -> Result<()> {
        let Some(key) = &self.key else {
            return Ok(());
        };

        runtime.file(
            key,
            NodeKind::Block,
            Some(Nuc::Block(self.clone())),
            dependencies,
            self.instance.clone(),
        )
    }

    /// Whether the block reads a named instance from the state, rather than only
    /// its own locals and the type it is stating a rule about.
    fn reads_an_instance(&self, runtime: &Runtime, scope: &Scope) -> bool {
        let mut assigned = Vec::new();
        for statement in &self.statements {
            collect_assigned_names(statement, &mut assigned);
        }

        let mut roots = Vec::new();
        for statement in &self.statements {
            collect_read_roots(statement, &mut roots);
        }

        roots.iter().any(|root| {
            !assigned.contains(root)
                && !scope.has(root)
                && matches!(runtime.state.variable(root), Some(Value::Object(_)))
        })
    }
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
