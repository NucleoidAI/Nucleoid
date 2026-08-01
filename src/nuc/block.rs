//! `{ ... }` — a run of statements filed as one declaration, re-evaluated
//! whenever anything it read changes. Mirrors `ref/src/nuc/BLOCK.js`.

use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{Expr, Stmt, collect_assigned_names, collect_read_roots};
use crate::lang::estree::generator::generate_all;
use crate::lang::evaluation::Flow;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

impl Runtime {
    pub(crate) fn declare_block(
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

            let key = format!("block({})", generate_all(statements));
            self.declare_on_class(&class, key, statement.clone())?;
            return Ok(Flow::Normal(Value::Null));
        }

        let rendered = generate_all(statements);
        let key = match scope.instance() {
            Some(instance) => NodeKey::new(format!("block({rendered})@{instance}")),
            None => NodeKey::new(format!("block({rendered})")),
        };

        let instance = scope.instance().cloned();

        self.push_tracking(false);
        scope.push();
        self.enter_imperative();
        let outcome = self.execute_all(statements, scope);
        self.leave_imperative();
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
