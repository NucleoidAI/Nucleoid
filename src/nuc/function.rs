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

#[derive(Debug, Clone)]
pub struct Function {
    pub function: Arc<FunctionDecl>,
}

impl Function {
    pub fn new(function: Arc<FunctionDecl>) -> Self {
        Function { function }
    }

    pub fn key(&self) -> Option<NodeKey> {
        self.function.name.clone().map(NodeKey::function)
    }

    pub fn run(&mut self, runtime: &mut Runtime, _scope: &mut Scope) -> Result<Outcome> {
        if let Some(name) = &self.function.name {
            runtime.insert_function(name, self.function.clone());
        }

        Ok(Outcome::null())
    }

    pub fn graph(&self, runtime: &mut Runtime) -> Result<()> {
        let Some(key) = self.key() else {
            return Ok(());
        };

        runtime.file(&key, NodeKind::Function, None, IndexSet::new(), None)
    }

    pub fn after(&self, runtime: &mut Runtime) -> Result<()> {
        match self.key() {
            Some(key) => runtime.propagate(&key),
            None => Ok(()),
        }
    }
}
