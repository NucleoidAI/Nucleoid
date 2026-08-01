//! `if (...) { ... }` — a standing condition, re-evaluated whenever anything it
//! tests changes. Mirrors `ref/src/nuc/IF.js`; the `$CLASS` and `$INSTANCE`
//! variants in `ref` are the two branches at the top of [`Runtime::declare_if`].

use crate::error::Result;
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::Stmt;
use crate::lang::evaluation::Flow;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

impl Runtime {
    pub(crate) fn declare_if(&mut self, statement: &Stmt, scope: &mut Scope) -> Result<Flow> {
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
        self.enter_imperative();
        let outcome = self.run_if(statement, scope);
        self.leave_imperative();
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
}
