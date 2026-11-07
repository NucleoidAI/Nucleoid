use std::sync::Arc;

use crate::graph::{Graph, GraphNode, NodeKey, ShapeKey};
use crate::history::{History, HistoryKey};
use crate::lang::ast::Function;
use crate::state::{ClassData, State};
use crate::value::{ObjectData, ObjectId, Value};

/// The previous contents of one slot, recorded before it was overwritten.
#[derive(Debug, Clone)]
pub(super) enum Undo {
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
pub(super) enum Slot {
    Variable(String),
    Property(ObjectId, String),
    Object(ObjectId),
    Class(String),
    Function(String),
    Node(NodeKey),
}

pub(super) fn apply(entry: Undo, state: &mut State, graph: &mut Graph) {
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
