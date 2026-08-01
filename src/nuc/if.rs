//! `IF` — a standing condition, re-evaluated whenever anything it tests
//! changes. Mirrors `ref/src/nuc/IF.js`, which likewise keeps the condition and
//! both branches on the node; `ref`'s `IF$CLASS.js` and `IF$INSTANCE.js` are
//! the two arms at the top of [`If::run`].

use indexmap::IndexSet;

use crate::error::Result;
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{Expr, Stmt};
use crate::lang::evaluation::Flow;
use crate::nuc::Outcome;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

pub struct If {
    pub condition: Expr,
    pub consequent: Vec<Stmt>,
    pub alternate: Option<Box<Stmt>>,
    key: Option<NodeKey>,
    instance: Option<ObjectId>,
}

impl If {
    pub fn new(condition: Expr, consequent: Vec<Stmt>, alternate: Option<Box<Stmt>>) -> Self {
        If {
            condition,
            consequent,
            alternate,
            key: None,
            instance: None,
        }
    }

    /// `.value` in a condition means the value it has now, so the stored rule
    /// keeps it rather than re-reading it on every run.
    pub fn before(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<()> {
        // A condition kept on a class is re-run per instance and is not frozen,
        // so leave it alone until that is ruled out.
        if runtime.class_reference(&self.statement(), scope).is_some() {
            return Ok(());
        }

        self.condition = runtime.freeze(&self.condition, scope)?;
        Ok(())
    }

    fn statement(&self) -> Stmt {
        Stmt::If {
            condition: self.condition.clone(),
            consequent: self.consequent.clone(),
            alternate: self.alternate.clone(),
        }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let statement = self.statement();

        if let Some(class) = runtime.class_reference(&statement, scope) {
            let key = format!("if({})", self.condition);
            runtime.declare_on_class(&class, key, statement)?;
            return Ok(Outcome::null());
        }

        self.instance = scope.instance().cloned();
        self.key = Some(match &self.instance {
            Some(instance) => NodeKey::new(format!("if({})@{instance}", self.condition)),
            None => NodeKey::new(format!("if({})", self.condition)),
        });

        runtime.push_tracking(false);
        runtime.enter_imperative();
        let outcome = self.branch(runtime, scope);
        runtime.leave_imperative();
        let dependencies = runtime.pop_tracking();

        Ok(Outcome {
            flow: outcome?,
            dependencies,
        })
    }

    fn branch(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Flow> {
        let saved = std::mem::take(&mut runtime.undefined_read);
        let test = runtime.evaluate(&self.condition, scope);
        let unresolved = runtime.undefined_read;
        runtime.undefined_read = saved;
        let test = test?;

        // While a class-level rule still reads something undefined it stands
        // aside, rather than deciding on a value that is not there yet.
        if unresolved && scope.instance().is_some() {
            return Ok(Flow::Normal(Value::Null));
        }

        // Branch bodies run in the surrounding scope: only `{ }` introduces
        // locals, so an assignment in a branch is a state assignment.
        if test.truthy() {
            runtime.execute_all(&self.consequent, scope)
        } else if let Some(alternate) = &self.alternate {
            match alternate.as_ref() {
                Stmt::Block(statements) => runtime.execute_all(statements, scope),
                other => runtime.execute(other, scope),
            }
        } else {
            Ok(Flow::Normal(Value::Null))
        }
    }

    pub fn graph(&self, runtime: &mut Runtime, dependencies: IndexSet<NodeKey>) -> Result<()> {
        let Some(key) = &self.key else {
            return Ok(());
        };

        runtime.file(
            key,
            NodeKind::If,
            Some(self.statement()),
            dependencies,
            self.instance.clone(),
        )
    }
}
