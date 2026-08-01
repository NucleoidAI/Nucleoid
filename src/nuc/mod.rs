//! The statement kinds the runtime files in the dependency graph, one module
//! per kind, mirroring `ref/src/nuc`.
//!
//! `ref` splits each kind again by the context it runs in — `IF.js`,
//! `IF$CLASS.js`, `IF$INSTANCE.js` — because JavaScript dispatches on the
//! constructed subclass. Here the context is a field on [`crate::scope::Scope`]
//! and a [`crate::graph::NodeKind`], so the `$CLASS` and `$INSTANCE` variants
//! are branches inside the module named after the kind.
//!
//! `ALIAS.js`, `LET.js` and `REFERENCE.js` have no counterpart: they are
//! JavaScript declaration forms that `nucleoid.spec.md` does not have.
//!
//! This module itself corresponds to `ref/src/nuc/NODE.js`, the registration of
//! a statement and its edges.

pub mod block;
pub mod class;
pub mod delete;
pub mod expression;
pub mod function;
pub mod object;
pub mod property;
pub mod variable;

#[path = "for.rs"]
pub mod r#for;
#[path = "if.rs"]
pub mod r#if;
#[path = "return.rs"]
pub mod r#return;
pub mod throw;

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{GraphNode, NodeKey, NodeKind};
use crate::lang::ast::Stmt;
use crate::runtime::Runtime;
use crate::value::ObjectId;

impl Runtime {
    /// Files a statement in the graph under `key` and wires up its edges,
    /// refusing an edge that would close a cycle.
    pub(crate) fn register(
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

    /// Replaces a node with a placeholder, keeping its dependents so they are
    /// still woken when the name is defined again.
    pub(crate) fn remove_node(&mut self, key: &NodeKey) {
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
}
