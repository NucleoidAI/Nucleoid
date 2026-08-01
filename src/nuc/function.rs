//! A named function definition, which is filed in the graph so that
//! redefining it re-evaluates everything that calls it. Mirrors
//! `ref/src/nuc/FUNCTION.js`.

use indexmap::IndexSet;
use std::sync::Arc;

use crate::error::Result;
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::Function;
use crate::runtime::Runtime;

impl Runtime {
    pub(crate) fn declare_function(&mut self, function: &Arc<Function>) -> Result<()> {
        let Some(name) = &function.name else {
            return Ok(());
        };

        let before = self.state.functions.get(name).cloned();
        self.transaction.record_function(name, before);
        self.state.functions.insert(name.clone(), function.clone());

        let key = NodeKey::new(name.clone());
        self.register(&key, NodeKind::Function, None, IndexSet::new(), None)?;
        self.propagate(&key)
    }
}
