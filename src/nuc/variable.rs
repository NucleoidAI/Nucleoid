//! `VARIABLE` — `a = 1`, a standing declaration filed under the variable's own
//! name. Mirrors `ref/src/nuc/VARIABLE.js`.

use indexmap::IndexSet;

use crate::error::Result;
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::Expr;
use crate::lang::evaluation::Flow;
use crate::nuc::object::Object;
use crate::nuc::{Nuc, Outcome};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::ObjectId;

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: Expr,
    /// How the node should be filed, decided while running: `Object` when the
    /// value turned out to be an instantiation, and nothing at all while
    /// running imperatively, where a statement is carried out rather than
    /// declared.
    kind: Option<NodeKind>,
}

impl Variable {
    pub fn new(name: String, value: Expr) -> Self {
        Variable {
            name,
            value,
            kind: None,
        }
    }

    pub fn key(&self) -> NodeKey {
        NodeKey::variable(self.name.clone())
    }

    pub fn before(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<()> {
        self.value = runtime.freeze(&self.value, scope)?;
        Ok(())
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let key = self.key();

        if runtime.is_imperative() {
            let (evaluated, _) = runtime.evaluate_tracked(&self.value, scope, Some(&key))?;
            runtime.assign(&self.name, evaluated.clone());
            self.kind = None;
            return Ok(Outcome::value(evaluated));
        }

        // `a = Person(...)` names the new instance after what it is assigned to.
        if let Some((class, arguments)) = runtime.instantiation(&self.value) {
            let object = Object::new(ObjectId::named(self.name.clone()), class, arguments);
            let created = object.run(runtime, scope)?;
            runtime.assign(&self.name, created.clone());
            self.kind = Some(NodeKind::Object);
            return Ok(Outcome::value(created));
        }

        let (evaluated, dependencies) = runtime.evaluate_tracked(&self.value, scope, Some(&key))?;
        runtime.assign(&self.name, evaluated.clone());
        self.kind = Some(NodeKind::Variable);

        Ok(Outcome {
            flow: Flow::Normal(evaluated),
            dependencies,
        })
    }

    pub fn graph(&self, runtime: &mut Runtime, dependencies: IndexSet<NodeKey>) -> Result<()> {
        let Some(kind) = self.kind else {
            return Ok(());
        };

        runtime.file(
            &self.key(),
            kind,
            Some(Nuc::Variable(self.clone())),
            dependencies,
            None,
        )
    }

    pub fn after(&self, runtime: &mut Runtime) -> Result<()> {
        runtime.propagate(&self.key())
    }
}
