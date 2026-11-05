use indexmap::IndexSet;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::graph::{Graph, GraphNode, NodeKey, ShapeKey};
use crate::history::{History, HistoryKey};
use crate::lang::ast::Function;
use crate::runtime::Runtime;
use crate::stack::Stack;
use crate::state::{ClassData, State};
use crate::value::{ObjectData, ObjectId, Value};

/// A caught failure must undo its queued effects as well as its writes.
pub(crate) struct Savepoint {
    mark: usize,
    stack: Stack,
    deleted: IndexSet<NodeKey>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Start<T> {
    Unchanged,
    Absent,
    Held(T),
}

impl<T> Start<T> {
    fn of(before: Option<T>) -> Self {
        match before {
            Some(value) => Start::Held(value),
            None => Start::Absent,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Origin<'t> {
    variables: HashMap<&'t str, Option<&'t Value>>,
    properties: HashMap<(&'t str, &'t str), (usize, Option<&'t Value>)>,
    objects: HashMap<&'t str, (usize, Option<&'t ObjectData>)>,
    nodes: HashMap<&'t str, Option<&'t GraphNode>>,
    shapes: HashMap<(&'t ShapeKey, &'t str), bool>,
    observers: HashSet<&'t str>,
}

impl<'t> Origin<'t> {
    fn record(&mut self, index: usize, entry: &'t Undo) {
        match entry {
            Undo::Variable { name, before } => {
                self.variables
                    .entry(name.as_str())
                    .or_insert(before.as_ref());
            }
            Undo::Property {
                object,
                property,
                before,
            } => {
                self.properties
                    .entry((object.as_str(), property.as_str()))
                    .or_insert((index, before.as_ref()));
            }
            Undo::PropertyRemoved {
                object,
                property,
                before,
                ..
            } => {
                self.properties
                    .entry((object.as_str(), property.as_str()))
                    .or_insert((index, Some(before)));
            }
            Undo::Object { id, before } => {
                self.objects
                    .entry(id.as_str())
                    .or_insert((index, before.as_ref()));
            }
            Undo::Node { key, before } => {
                self.nodes.entry(key.as_str()).or_insert(before.as_ref());
            }
            Undo::ShapeDependentAdded { source, dependent } => {
                self.shape(source, dependent, false);
            }
            Undo::ShapeDependentRemoved { source, dependent } => {
                self.shape(source, dependent, true);
            }
            Undo::HistoryLength { .. }
            | Undo::HistoryRemoved { .. }
            | Undo::Class { .. }
            | Undo::Function { .. }
            | Undo::DependentAdded { .. }
            | Undo::DependentRemoved { .. } => {}
        }
    }

    fn shape(&mut self, source: &'t ShapeKey, dependent: &'t NodeKey, present: bool) {
        if source.observes_graph() {
            self.shapes
                .entry((source, dependent.as_str()))
                .or_insert(present);
        }
    }

    fn settle(mut self) -> Self {
        self.observers = self
            .shapes
            .iter()
            .filter(|(_, present)| **present)
            .map(|((_, dependent), _)| *dependent)
            .collect();
        self
    }

    #[must_use]
    pub(crate) fn node(&self, key: &NodeKey) -> Start<&'t GraphNode> {
        self.nodes
            .get(key.as_str())
            .map_or(Start::Unchanged, |before| Start::of(*before))
    }

    #[must_use]
    pub(crate) fn observed(&self, key: &NodeKey) -> bool {
        self.observers.contains(key.as_str())
    }

    #[must_use]
    pub(crate) fn variable(&self, name: &str) -> Start<&'t Value> {
        self.variables
            .get(name)
            .map_or(Start::Unchanged, |before| Start::of(*before))
    }

    #[must_use]
    pub(crate) fn property(&self, object: &str, property: &str) -> Start<&'t Value> {
        let own = self.properties.get(&(object, property)).copied();
        let whole = self
            .objects
            .get(object)
            .map(|(index, data)| (*index, data.and_then(|data| data.properties.get(property))));
        let before = match (own, whole) {
            (Some((first, value)), Some((second, _))) if first < second => value,
            (_, Some((_, value))) | (Some((_, value)), None) => value,
            (None, None) => return Start::Unchanged,
        };
        Start::of(before)
    }
}

/// The previous contents of one slot, recorded before it was overwritten.
#[derive(Debug, Clone)]
enum Undo {
    /// An append needs only a length, not a copy of all retained values.
    HistoryLength {
        key: HistoryKey,
        length: usize,
    },
    /// Move deleted histories into the log, so delete/recreate and nested
    /// savepoints restore the original lifetime without copying its values.
    HistoryRemoved {
        key: HistoryKey,
        history: History,
    },
    Variable {
        name: String,
        before: Option<Value>,
    },
    Property {
        object: ObjectId,
        property: String,
        before: Option<Value>,
    },
    PropertyRemoved {
        object: ObjectId,
        property: String,
        index: usize,
        before: Value,
    },
    Object {
        id: ObjectId,
        before: Option<ObjectData>,
    },
    Class {
        name: String,
        before: Option<ClassData>,
    },
    Function {
        name: String,
        before: Option<Arc<Function>>,
    },
    Node {
        key: NodeKey,
        before: Option<GraphNode>,
    },
    /// One edge, undone on its own. Recording the whole node instead would copy
    /// its list of dependents, which is what a widely-read name accumulates.
    DependentAdded {
        source: NodeKey,
        dependent: NodeKey,
    },
    DependentRemoved {
        source: NodeKey,
        dependent: NodeKey,
    },
    ShapeDependentAdded {
        source: ShapeKey,
        dependent: NodeKey,
    },
    ShapeDependentRemoved {
        source: ShapeKey,
        dependent: NodeKey,
    },
}

/// Identifies one slot, so an overwritten slot is recorded once however often it is written.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Slot {
    Variable(String),
    Property(ObjectId, String),
    Object(ObjectId),
    Class(String),
    Function(String),
    Node(NodeKey),
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

fn apply(entry: Undo, state: &mut State, graph: &mut Graph) {
    match entry {
        Undo::HistoryLength { key, length } => state.truncate_history(&key, length),
        Undo::HistoryRemoved { key, history } => state.restore_history(key, history),
        Undo::Variable { name, before } => state.restore_variable(name, before),
        Undo::Property {
            object,
            property,
            before,
        } => state.restore_property(&object, property, before),
        Undo::PropertyRemoved {
            object,
            property,
            index,
            before,
        } => state.restore_removed_property(&object, property, index, before),
        Undo::Object { id, before } => state.restore_object(id, before),
        Undo::Class { name, before } => state.restore_class(name, before),
        Undo::Function { name, before } => state.restore_function(name, before),
        Undo::Node { key, before } => match before {
            Some(node) => {
                graph.insert(node);
            }
            None => {
                graph.remove(&key);
            }
        },
        Undo::DependentAdded { source, dependent } => {
            if let Some(node) = graph.get_mut(&source) {
                node.dependents.shift_remove(&dependent);
            }
        }
        Undo::DependentRemoved { source, dependent } => {
            if let Some(node) = graph.get_mut(&source) {
                node.dependents.insert(dependent);
            }
        }
        Undo::ShapeDependentAdded { source, dependent } => {
            graph.remove_shape_dependency(&source, &dependent);
        }
        Undo::ShapeDependentRemoved { source, dependent } => {
            graph.add_shape_dependency(&source, &dependent);
        }
    }
}
