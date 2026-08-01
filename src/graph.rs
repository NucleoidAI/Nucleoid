use indexmap::{IndexMap, IndexSet};
use std::fmt;

use crate::ast::Stmt;
use crate::value::ObjectId;

/// The name a statement is filed under in the dependency graph.
///
/// Variables use their own name, properties use `<object id>.<property>`, and
/// control-flow statements use their rendered source so that redeclaring the
/// same condition replaces the previous node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeKey(String);

impl NodeKey {
    pub fn new(key: impl Into<String>) -> Self {
        NodeKey(key.into())
    }

    pub fn property(object: &ObjectId, property: &str) -> Self {
        NodeKey(format!("{object}.{property}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Splits `a.b.c` into (`a.b`, `c`).
    pub fn split_last(&self) -> Option<(&str, &str)> {
        self.0.rsplit_once('.')
    }
}

impl fmt::Display for NodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for NodeKey {
    fn from(value: &str) -> Self {
        NodeKey(value.to_string())
    }
}

impl From<String> for NodeKey {
    fn from(value: String) -> Self {
        NodeKey(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// A placeholder for something referenced before it was defined.
    Pending,
    Variable,
    Property,
    Object,
    Class,
    Function,
    If,
    Block,
}

impl NodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            NodeKind::Pending => "pending",
            NodeKind::Variable => "variable",
            NodeKind::Property => "property",
            NodeKind::Object => "object",
            NodeKind::Class => "class",
            NodeKind::Function => "function",
            NodeKind::If => "if",
            NodeKind::Block => "block",
        }
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A statement filed in the graph, together with the edges that decide when it
/// is re-evaluated.
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub key: NodeKey,
    pub kind: NodeKind,
    pub statement: Option<Stmt>,
    /// The instance a class-level declaration was instantiated for.
    pub instance: Option<ObjectId>,
    pub dependencies: IndexSet<NodeKey>,
    pub dependents: IndexSet<NodeKey>,
    pub sequence: u64,
}

impl GraphNode {
    pub fn new(key: NodeKey, kind: NodeKind, sequence: u64) -> Self {
        GraphNode {
            key,
            kind,
            statement: None,
            instance: None,
            dependencies: IndexSet::new(),
            dependents: IndexSet::new(),
            sequence,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Graph {
    nodes: IndexMap<NodeKey, GraphNode>,
    sequence: u64,
}

impl Graph {
    pub fn new() -> Self {
        Graph::default()
    }

    pub fn next_sequence(&mut self) -> u64 {
        self.sequence += 1;
        self.sequence
    }

    pub fn get(&self, key: &NodeKey) -> Option<&GraphNode> {
        self.nodes.get(key)
    }

    pub fn get_mut(&mut self, key: &NodeKey) -> Option<&mut GraphNode> {
        self.nodes.get_mut(key)
    }

    pub fn contains(&self, key: &NodeKey) -> bool {
        self.nodes.contains_key(key)
    }

    pub fn insert(&mut self, node: GraphNode) -> Option<GraphNode> {
        self.nodes.insert(node.key.clone(), node)
    }

    pub fn remove(&mut self, key: &NodeKey) -> Option<GraphNode> {
        self.nodes.shift_remove(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &NodeKey> {
        self.nodes.keys()
    }

    /// Every statement filed in the graph, in the order the keys were first
    /// created. This is the logic graph: what the runtime knows, and what it
    /// will re-evaluate when something changes.
    pub fn nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.nodes.values()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The dependents of a node, ordered by the sequence in which they were
    /// declared. A redeclared statement takes a fresh sequence, so it runs last.
    pub fn dependents_in_order(&self, key: &NodeKey) -> Vec<NodeKey> {
        let Some(node) = self.nodes.get(key) else {
            return Vec::new();
        };

        let mut dependents: Vec<(u64, NodeKey)> = node
            .dependents
            .iter()
            .filter_map(|dependent| {
                self.nodes
                    .get(dependent)
                    .map(|node| (node.sequence, dependent.clone()))
            })
            .collect();

        dependents.sort_by_key(|(sequence, _)| *sequence);
        dependents.into_iter().map(|(_, key)| key).collect()
    }

    /// Whether `from` can reach `to` by following dependent edges, which is what
    /// makes a new edge circular.
    pub fn reaches(&self, from: &NodeKey, to: &NodeKey) -> bool {
        let mut seen: IndexSet<NodeKey> = IndexSet::new();
        let mut pending = vec![from.clone()];

        while let Some(current) = pending.pop() {
            if &current == to {
                return true;
            }

            if !seen.insert(current.clone()) {
                continue;
            }

            if let Some(node) = self.nodes.get(&current) {
                pending.extend(node.dependents.iter().cloned());
            }
        }

        false
    }
}
