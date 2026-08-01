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
}

/// An undo log for one `run`. Everything a statement touches is recorded so an
/// exception leaves the state exactly as it was before the run started.
#[derive(Debug, Clone, Default)]
pub struct Transaction {
    entries: Vec<Undo>,
    active: bool,
}

impl Transaction {
    pub fn new() -> Self {
        Transaction::default()
    }

    pub fn start(&mut self) {
        self.entries.clear();
        self.active = true;
    }

    pub fn commit(&mut self) {
        self.entries.clear();
        self.active = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn record_variable(&mut self, name: &str, before: Option<Value>) {
        if self.active {
            self.entries.push(Undo::Variable {
                name: name.to_string(),
                before,
            });
        }
    }

    pub fn record_property(&mut self, object: &ObjectId, property: &str, before: Option<Value>) {
        if self.active {
            self.entries.push(Undo::Property {
                object: object.clone(),
                property: property.to_string(),
                before,
            });
        }
    }

    pub fn record_object(&mut self, id: &ObjectId, before: Option<ObjectData>) {
        if self.active {
            self.entries.push(Undo::Object {
                id: id.clone(),
                before,
            });
        }
    }

    pub fn record_class(&mut self, name: &str, before: Option<ClassData>) {
        if self.active {
            self.entries.push(Undo::Class {
                name: name.to_string(),
                before,
            });
        }
    }

    pub fn record_function(&mut self, name: &str, before: Option<Arc<Function>>) {
        if self.active {
            self.entries.push(Undo::Function {
                name: name.to_string(),
                before,
            });
        }
    }

    pub fn record_node(&mut self, key: &NodeKey, before: Option<GraphNode>) {
        if self.active {
            self.entries.push(Undo::Node {
                key: key.clone(),
                before,
            });
        }
    }

    pub fn rollback(&mut self, state: &mut State, graph: &mut Graph) {
        while let Some(entry) = self.entries.pop() {
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
            }
        }

        self.active = false;
    }
}
