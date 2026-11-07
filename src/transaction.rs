mod origin;
mod undo;

use indexmap::IndexSet;
use std::sync::Arc;

use crate::graph::{Graph, GraphNode, NodeKey, ShapeKey};
use crate::history::{History, HistoryKey};
use crate::lang::ast::Function;
use crate::runtime::Runtime;
use crate::stack::Stack;
use crate::state::{ClassData, State};
use crate::value::{ObjectData, ObjectId, Value};
pub(crate) use origin::{Origin, Start};
use undo::{Slot, Undo, apply};

/// A caught failure must undo its queued effects as well as its writes.
pub(crate) struct Savepoint {
    mark: usize,
    stack: Stack,
    deleted: IndexSet<NodeKey>,
}

/// An undo log for one `run`. Everything a statement touches is recorded so an
/// exception leaves the state exactly as it was before the run started.
///
/// State and graph writes are logged here. Runtime savepoints also preserve
/// pending propagation and temporary deletion tracking.
///
/// Only the first write to a slot is recorded, except that every addition and
/// removal of a property is recorded so rolling back also restores key order:
/// the first write holds the value the run began with, which is what rolling
/// back has to restore. Later overwrites of the same slot cost nothing, which
/// matters because a class is rewritten once per instance it gains.
#[derive(Debug, Clone, Default)]
pub struct Transaction {
    entries: Vec<Undo>,
    recorded: IndexSet<Slot>,
    active: bool,
    hypotheses: Vec<usize>,
}

impl Transaction {
    #[must_use]
    pub fn new() -> Self {
        Transaction::default()
    }

    pub fn start(&mut self) {
        self.entries.clear();
        self.recorded.clear();
        self.active = true;
    }

    pub fn commit(&mut self) {
        self.entries.clear();
        self.recorded.clear();
        self.active = false;
    }

    /// Whether this slot still needs its before-image kept.
    fn first_write(&mut self, slot: Slot) -> bool {
        self.active && self.recorded.insert(slot)
    }

    /// Whether a before-image is still wanted, so a caller can avoid cloning one
    /// that would be thrown away.
    #[must_use]
    pub fn needs_class(&self, name: &str) -> bool {
        self.active && !self.recorded.contains(&Slot::Class(name.to_string()))
    }

    #[must_use]
    pub fn needs_node(&self, key: &NodeKey) -> bool {
        self.active && !self.recorded.contains(&Slot::Node(key.clone()))
    }

    #[must_use]
    pub fn needs_object(&self, id: &ObjectId) -> bool {
        self.active && !self.recorded.contains(&Slot::Object(id.clone()))
    }

    /// Opens a savepoint. What follows can be undone on its own, which is how a
    /// `try` puts back only what its body changed.
    ///
    /// Slots recorded before the mark are forgotten, so anything written inside
    /// is recorded again with the value it had here.
    pub fn mark(&mut self) -> usize {
        self.recorded.clear();
        self.entries.len()
    }

    /// Undoes everything recorded since a mark, leaving earlier entries for the
    /// enclosing transaction.
    pub fn rollback_to(&mut self, mark: usize, state: &mut State, graph: &mut Graph) {
        debug_assert!(mark <= self.entries.len());
        while self.entries.len() > mark {
            let Some(entry) = self.entries.pop() else {
                break;
            };

            apply(entry, state, graph);
        }

        self.recorded.clear();
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    fn begin_hypothesis(&mut self, mark: usize) {
        debug_assert!(mark <= self.entries.len());
        self.hypotheses.push(mark);
    }

    fn end_hypothesis(&mut self, mark: usize) {
        debug_assert_eq!(self.hypotheses.last(), Some(&mark));
        self.hypotheses.pop();
    }

    #[must_use]
    pub(crate) fn hypotheses(&self) -> &[usize] {
        &self.hypotheses
    }

    #[must_use]
    pub(crate) fn origin(&self) -> Option<Origin<'_>> {
        let &mark = self.hypotheses.last()?;
        debug_assert!(mark <= self.entries.len());
        let mut origin = Origin::default();
        for (index, entry) in self.entries.iter().enumerate().skip(mark) {
            origin.record(index, entry);
        }
        Some(origin.settle())
    }

    pub(crate) fn record_history_append(&mut self, key: &HistoryKey, length: usize) {
        if self.active {
            self.entries.push(Undo::HistoryLength {
                key: key.clone(),
                length,
            });
        }
    }

    pub(crate) fn record_history_removal(&mut self, key: HistoryKey, history: History) {
        if self.active {
            self.entries.push(Undo::HistoryRemoved { key, history });
        }
    }

    pub fn record_variable(&mut self, name: &str, before: Option<Value>) {
        if self.first_write(Slot::Variable(name.to_string())) {
            self.entries.push(Undo::Variable {
                name: name.to_string(),
                before,
            });
        }
    }

    pub fn record_property(&mut self, object: &ObjectId, property: &str, before: Option<Value>) {
        if self.first_write(Slot::Property(object.clone(), property.to_string())) {
            self.entries.push(Undo::Property {
                object: object.clone(),
                property: property.to_string(),
                before,
            });
        }
    }

    pub(crate) fn record_property_added(&mut self, object: &ObjectId, property: &str) {
        if self.active {
            self.recorded
                .insert(Slot::Property(object.clone(), property.to_string()));
            self.entries.push(Undo::Property {
                object: object.clone(),
                property: property.to_string(),
                before: None,
            });
        }
    }

    pub(crate) fn record_property_removed(
        &mut self,
        object: &ObjectId,
        property: &str,
        index: usize,
        before: Value,
    ) {
        if self.active {
            self.recorded
                .insert(Slot::Property(object.clone(), property.to_string()));
            self.entries.push(Undo::PropertyRemoved {
                object: object.clone(),
                property: property.to_string(),
                index,
                before,
            });
        }
    }

    pub fn record_object(&mut self, id: &ObjectId, before: Option<ObjectData>) {
        if self.first_write(Slot::Object(id.clone())) {
            self.entries.push(Undo::Object {
                id: id.clone(),
                before,
            });
        }
    }

    pub fn record_class(&mut self, name: &str, before: Option<ClassData>) {
        if self.first_write(Slot::Class(name.to_string())) {
            self.entries.push(Undo::Class {
                name: name.to_string(),
                before,
            });
        }
    }

    pub fn record_function(&mut self, name: &str, before: Option<Arc<Function>>) {
        if self.first_write(Slot::Function(name.to_string())) {
            self.entries.push(Undo::Function {
                name: name.to_string(),
                before,
            });
        }
    }

    /// Records an edge that was just added, so rollback can take it out again.
    pub fn record_dependent_added(&mut self, source: &NodeKey, dependent: &NodeKey) {
        if self.active {
            self.entries.push(Undo::DependentAdded {
                source: source.clone(),
                dependent: dependent.clone(),
            });
        }
    }

    /// Records an edge that was just removed.
    pub fn record_dependent_removed(&mut self, source: &NodeKey, dependent: &NodeKey) {
        if self.active {
            self.entries.push(Undo::DependentRemoved {
                source: source.clone(),
                dependent: dependent.clone(),
            });
        }
    }

    pub fn record_node(&mut self, key: &NodeKey, before: Option<GraphNode>) {
        if self.first_write(Slot::Node(key.clone())) {
            self.entries.push(Undo::Node {
                key: key.clone(),
                before,
            });
        }
    }

    pub(crate) fn record_shape_dependent_added(&mut self, source: &ShapeKey, dependent: &NodeKey) {
        if self.active {
            self.entries.push(Undo::ShapeDependentAdded {
                source: source.clone(),
                dependent: dependent.clone(),
            });
        }
    }

    pub(crate) fn record_shape_dependent_removed(
        &mut self,
        source: &ShapeKey,
        dependent: &NodeKey,
    ) {
        if self.active {
            self.entries.push(Undo::ShapeDependentRemoved {
                source: source.clone(),
                dependent: dependent.clone(),
            });
        }
    }

    pub fn rollback(&mut self, state: &mut State, graph: &mut Graph) {
        while let Some(entry) = self.entries.pop() {
            apply(entry, state, graph);
        }

        self.recorded.clear();
        self.active = false;
    }
}

impl Runtime {
    pub(crate) fn savepoint(&mut self) -> Savepoint {
        Savepoint {
            mark: self.transaction.mark(),
            stack: self.stack.clone(),
            deleted: self.deleted.clone(),
        }
    }

    pub(crate) fn rollback_to(&mut self, savepoint: Savepoint) {
        self.transaction
            .rollback_to(savepoint.mark, &mut self.state, &mut self.graph);
        self.stack = savepoint.stack;
        self.deleted = savepoint.deleted;
    }

    pub(crate) fn hypothesis_savepoint(&mut self) -> Savepoint {
        let savepoint = self.savepoint();
        self.transaction.begin_hypothesis(savepoint.mark);
        savepoint
    }

    pub(crate) fn rollback_hypothesis(&mut self, savepoint: Savepoint) {
        self.transaction.end_hypothesis(savepoint.mark);
        self.rollback_to(savepoint);
    }
}
