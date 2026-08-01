use indexmap::{IndexMap, IndexSet};
use std::fmt;

use crate::lang::ast::{Expr, Stmt};
use crate::lang::estree::generator::generate_all;
use crate::value::ObjectId;

/// The name a statement is filed under in the dependency graph.
///
/// `ref/src/nuc/NODE.js` keys a node by `key.toString()` and each `$nuc`
/// builder formats its own — so the scheme lives in a dozen places there. Here
/// it lives only in the constructors below, and nothing else builds a key by
/// hand: two spellings of the same thing have to collapse onto one node, and
/// that only holds if one piece of code decides the spelling.
///
/// Variables and functions use their own name, properties use
/// `<object id>.<property>`, a class uses `$<name>`, and control-flow
/// statements use their rendered source, qualified by the instance whose rules
/// they belong to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeKey(String);

impl NodeKey {
    /// A top-level name: `a`.
    pub fn variable(name: impl Into<String>) -> Self {
        NodeKey(name.into())
    }

    /// A named function: `f`. The same shape as a variable, deliberately — a
    /// call depends on the name, however that name came to be defined.
    pub fn function(name: impl Into<String>) -> Self {
        NodeKey(name.into())
    }

    /// A property of one object: `person1.age`.
    pub fn property(object: &ObjectId, property: &str) -> Self {
        NodeKey(format!("{object}.{property}"))
    }

    /// A type: `$Person`. Reading the instances of a class depends on this, so
    /// creating or deleting an instance wakes whatever counted them.
    pub fn class(name: &str) -> Self {
        NodeKey(format!("${name}"))
    }

    /// One instance, under its own id.
    pub fn object(id: &ObjectId) -> Self {
        NodeKey(id.to_string())
    }

    /// A standing condition: `if(a>1)`, or `if(a>1)@person1` for the copy that
    /// belongs to one instance.
    pub fn conditional(condition: &Expr, instance: Option<&ObjectId>) -> Self {
        NodeKey(qualify(format!("if({condition})"), instance))
    }

    /// A standing block, keyed by the source of its statements.
    pub fn block(statements: &[Stmt], instance: Option<&ObjectId>) -> Self {
        NodeKey(qualify(
            format!("block({})", generate_all(statements)),
            instance,
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A statement applied to every instance of a class needs one node per
/// instance, or they would overwrite each other.
fn qualify(key: String, instance: Option<&ObjectId>) -> String {
    match instance {
        Some(instance) => format!("{key}@{instance}"),
        None => key,
    }
}

impl fmt::Display for NodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
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

    /// The node filed under a key, if there is one. `graph.retrieve` in
    /// `ref/src/graph.js`.
    pub fn retrieve(&self, key: &NodeKey) -> Option<&GraphNode> {
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

    /// Empties the graph, keeping the sequence counter so that keys filed after
    /// a clear still sort after the ones before it.
    pub fn clear(&mut self) {
        self.nodes.clear();
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
