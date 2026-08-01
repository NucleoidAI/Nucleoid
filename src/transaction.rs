use indexmap::IndexSet;
use std::sync::Arc;

use crate::ast::Function;
use crate::graph::{Graph, GraphNode, NodeKey};
use crate::state::{ClassData, State};
use crate::value::{ObjectData, ObjectId, Value};

/// The previous contents of one slot, recorded before it was overwritten.
#[derive(Debug, Clone)]
enum Undo {
    Variable {
        name: String,
        before: Option<Value>,
    },
    Property {
        object: ObjectId,
        property: String,
        before: Option<Value>,
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
}

/// Identifies one slot, so a slot is recorded once however often it is written.
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
/// Only the first write to a slot is recorded: that is the value the run began
/// with, which is what rolling back has to restore. Later writes to the same
/// slot cost nothing, which matters because a class is rewritten once per
/// instance it gains.
#[derive(Debug, Clone, Default)]
pub struct Transaction {
    entries: Vec<Undo>,
    recorded: IndexSet<Slot>,
    active: bool,
}

impl Transaction {
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
    pub fn needs_class(&self, name: &str) -> bool {
        self.active && !self.recorded.contains(&Slot::Class(name.to_string()))
    }

    pub fn needs_node(&self, key: &NodeKey) -> bool {
        self.active && !self.recorded.contains(&Slot::Node(key.clone()))
    }

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
        while self.entries.len() > mark {
            let Some(entry) = self.entries.pop() else {
                break;
            };

            apply(entry, state, graph);
        }

        self.recorded.clear();
    }

    pub fn is_active(&self) -> bool {
        self.active
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

    pub fn rollback(&mut self, state: &mut State, graph: &mut Graph) {
        while let Some(entry) = self.entries.pop() {
            apply(entry, state, graph);
        }

        self.recorded.clear();
        self.active = false;
    }
}

fn apply(entry: Undo, state: &mut State, graph: &mut Graph) {
    match entry {
        Undo::Variable { name, before } => match before {
            Some(value) => {
                state.variables.insert(name, value);
            }
            None => {
                state.variables.shift_remove(&name);
            }
        },
        Undo::Property {
            object,
            property,
            before,
        } => {
            if let Some(data) = state.objects.get_mut(&object) {
                match before {
                    Some(value) => {
                        data.properties.insert(property, value);
                    }
                    None => {
                        data.properties.shift_remove(&property);
                    }
                }
            }
        }
        Undo::Object { id, before } => match before {
            Some(data) => {
                state.objects.insert(id, data);
            }
            None => {
                state.objects.shift_remove(&id);
            }
        },
        Undo::Class { name, before } => match before {
            Some(data) => {
                state.classes.insert(name, data);
            }
            None => {
                state.classes.shift_remove(&name);
            }
        },
        Undo::Function { name, before } => match before {
            Some(function) => {
                state.functions.insert(name, function);
            }
            None => {
                state.functions.shift_remove(&name);
            }
        },
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
    }
}
