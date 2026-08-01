//! `a = 1` — a standing declaration filed under the variable's own name.
//! Mirrors `ref/src/nuc/VARIABLE.js`.

use indexmap::IndexSet;

use crate::error::Result;
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{Expr, Stmt};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

impl Runtime {
    pub(crate) fn assign_variable(
        &mut self,
        name: String,
        value: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let key = NodeKey::new(name.clone());
        let value = &self.freeze(value, scope)?;
        let statement = Stmt::Assign {
            target: Expr::Identifier(name.clone()),
            value: value.clone(),
        };

        if self.is_imperative() {
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

    /// A name bound in an enclosing scope — a parameter or a loop variable —
    /// which is written directly and never filed in the graph.
    pub(crate) fn assign_local(
        &mut self,
        name: String,
        value: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
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

    pub(crate) fn set_variable(&mut self, name: &str, value: Value) {
        let before = self.state.variables.get(name).cloned();
        self.transaction.record_variable(name, before);
        self.state.variables.insert(name.to_string(), value);
    }
}
