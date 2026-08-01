//! `FUNCTION` — a named definition, filed in the graph so that redefining it
//! re-evaluates everything that calls it. Mirrors `ref/src/nuc/FUNCTION.js`,
//! which is likewise a `NODE` with nothing added: what matters is that the name
//! has a node at all, so calls can depend on it.

use indexmap::IndexSet;
use std::sync::Arc;

use crate::error::Result;
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::Function as FunctionDecl;
use crate::nuc::Outcome;
use crate::runtime::Runtime;
use crate::scope::Scope;

pub struct Function {
    pub function: Arc<FunctionDecl>,
}

impl Function {
    pub fn new(function: Arc<FunctionDecl>) -> Self {
        Function { function }
    }

    pub fn key(&self) -> Option<NodeKey> {
        self.function.name.clone().map(NodeKey::new)
    }

    pub fn run(&mut self, runtime: &mut Runtime, _scope: &mut Scope) -> Result<Outcome> {
        if let Some(name) = &self.function.name {
            let before = runtime.state.functions.get(name).cloned();
            runtime.transaction.record_function(name, before);
            runtime
                .state
                .functions
                .insert(name.clone(), self.function.clone());
        }

        Ok(Outcome::null())
    }

    pub fn graph(&self, runtime: &mut Runtime) -> Result<()> {
        let Some(key) = self.key() else {
            return Ok(());
        };

        runtime.register(&key, NodeKind::Function, None, IndexSet::new(), None)
    }

    pub fn after(&self, runtime: &mut Runtime) -> Result<()> {
        match self.key() {
            Some(key) => runtime.propagate(&key),
            None => Ok(()),
        }
    }
}
