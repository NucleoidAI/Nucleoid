use indexmap::{IndexMap, IndexSet};
use std::fmt;

use crate::lang::ast::generator::generate_all;
use crate::lang::ast::{Expr, Stmt};
use crate::nuc::Nuc;
use crate::value::ObjectId;

/// The name a statement is filed under in the dependency graph.
///
/// Key formatting lives in the constructors below so that two spellings of
/// the same thing collapse onto one node.
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
    #[must_use]
    pub fn property(object: &ObjectId, property: &str) -> Self {
        NodeKey(format!("{object}.{property}"))
    }

    /// A type: `$Person`. Reading the instances of a class depends on this, so
    /// creating or deleting an instance wakes whatever counted them.
    #[must_use]
    pub fn class(name: &str) -> Self {
        NodeKey(format!("${name}"))
    }

    /// One instance, under its own id.
    #[must_use]
    pub fn object(id: &ObjectId) -> Self {
        NodeKey(id.to_string())
    }

    /// A standing condition: `if(a>1)`, or `if(a>1)@person1` for the copy that
    /// belongs to one instance.
    #[must_use]
    pub fn conditional(condition: &Expr, instance: Option<&ObjectId>) -> Self {
        NodeKey(qualify(format!("if({condition})"), instance))
    }

    /// A standing block, keyed by the source of its statements.
    #[must_use]
    pub fn block(statements: &[Stmt], instance: Option<&ObjectId>) -> Self {
        NodeKey(qualify(
            format!("block({})", generate_all(statements)),
            instance,
        ))
    }

    #[must_use]
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

/// Graph structure observed by a reasoning expression, separately from values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShapeKey {
    /// A node's declaration, dependencies, dependents, or presence.
    Node(NodeKey),
    /// The membership of the model, excluding pending nodes and observers.
    Model,
}

impl fmt::Display for ShapeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShapeKey::Node(key) => key.fmt(f),
            ShapeKey::Model => f.write_str("model"),
        }
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
    #[must_use]
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
///
/// `node` holds the runnable statement rather than its source, avoiding
/// recompilation on every propagation.
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub key: NodeKey,
    pub kind: NodeKind,
    pub node: Option<Nuc>,
    /// The instance a class-level declaration was instantiated for.
    pub instance: Option<ObjectId>,
    pub dependencies: IndexSet<NodeKey>,
    pub dependents: IndexSet<NodeKey>,
    pub sequence: u64,
}

impl GraphNode {
    #[must_use]
    pub fn new(key: NodeKey, kind: NodeKind, sequence: u64) -> Self {
        GraphNode {
            key,
            kind,
            node: None,
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
    shape_dependencies: IndexMap<NodeKey, IndexSet<ShapeKey>>,
    shape_dependents: IndexMap<ShapeKey, IndexSet<NodeKey>>,
    sequence: u64,
}

impl Graph {
    #[must_use]
    pub fn new() -> Self {
        Graph::default()
    }

    pub fn next_sequence(&mut self) -> u64 {
        self.sequence += 1;
        self.sequence
    }

    /// The node filed under a key, if there is one.
    #[must_use]
    pub fn retrieve(&self, key: &NodeKey) -> Option<&GraphNode> {
        self.nodes.get(key)
    }

    pub fn get_mut(&mut self, key: &NodeKey) -> Option<&mut GraphNode> {
        self.nodes.get_mut(key)
    }

    #[must_use]
    pub fn contains(&self, key: &NodeKey) -> bool {
        self.nodes.contains_key(key)
    }

    pub fn insert(&mut self, node: GraphNode) -> Option<GraphNode> {
        self.nodes.insert(node.key.clone(), node)
    }

    pub fn remove(&mut self, key: &NodeKey) -> Option<GraphNode> {
        let sources: Vec<ShapeKey> = self.shape_dependencies(key).cloned().collect();
        for source in sources {
            self.remove_shape_dependency(&source, key);
        }

        self.nodes.shift_remove(key)
    }

    /// Empties the graph, keeping the sequence counter so that keys filed after
    /// a clear still sort after the ones before it.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.shape_dependencies.clear();
        self.shape_dependents.clear();
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

    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Structural dependencies do not join the model's ordinary value edges.
    pub fn shape_dependencies(&self, key: &NodeKey) -> impl Iterator<Item = &ShapeKey> {
        self.shape_dependencies.get(key).into_iter().flatten()
    }

    pub(crate) fn shape_dependents(&self, source: &ShapeKey) -> impl Iterator<Item = &NodeKey> {
        self.shape_dependents.get(source).into_iter().flatten()
    }

    pub(crate) fn add_shape_dependency(&mut self, source: &ShapeKey, target: &NodeKey) -> bool {
        let added = self
            .shape_dependents
            .entry(source.clone())
            .or_default()
            .insert(target.clone());

        if added {
            self.shape_dependencies
                .entry(target.clone())
                .or_default()
                .insert(source.clone());
        }

        added
    }

    pub(crate) fn remove_shape_dependency(&mut self, source: &ShapeKey, target: &NodeKey) -> bool {
        let removed = self
            .shape_dependents
            .get_mut(source)
            .is_some_and(|dependents| dependents.shift_remove(target));

        if removed {
            if self
                .shape_dependents
                .get(source)
                .is_some_and(IndexSet::is_empty)
            {
                self.shape_dependents.shift_remove(source);
            }

            if let Some(dependencies) = self.shape_dependencies.get_mut(target) {
                dependencies.shift_remove(source);
                if dependencies.is_empty() {
                    self.shape_dependencies.shift_remove(target);
                }
            }
        }

        removed
    }

    pub(crate) fn is_observer(&self, key: &NodeKey) -> bool {
        self.shape_dependencies.contains_key(key)
            || self
                .nodes
                .get(key)
                .and_then(|node| node.node.as_ref())
                .is_some_and(crate::reasoning::is_reasoning)
    }

    /// The dependents of a node, ordered by the sequence in which they were
    /// declared. A redeclared statement takes a fresh sequence, so it runs last.
    #[must_use]
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
                    .map(|entry| (entry.sequence, dependent.clone()))
            })
            .collect();

        dependents.sort_by_key(|(sequence, _)| *sequence);
        dependents
            .into_iter()
            .map(|(_, dependent)| dependent)
            .collect()
    }

    /// Whether any queued statement is upstream of this one, including through
    /// a dependency that has not been queued yet.
    pub(crate) fn depends_on_any(&self, key: &NodeKey, candidates: &IndexSet<NodeKey>) -> bool {
        let mut seen = IndexSet::new();
        let mut pending = Vec::new();

        self.dependencies_into(key, &mut pending);

        while let Some(current) = pending.pop() {
            if candidates.contains(current) {
                return true;
            }

            if seen.insert(current) {
                self.dependencies_into(current, &mut pending);
            }
        }

        false
    }

    fn dependencies_into<'a>(&'a self, key: &NodeKey, pending: &mut Vec<&'a NodeKey>) {
        if let Some(node) = self.nodes.get(key) {
            pending.extend(&node.dependencies);
        }

        for source in self.shape_dependencies(key) {
            match source {
                ShapeKey::Node(node) => pending.push(node),
                ShapeKey::Model => pending.extend(
                    self.nodes
                        .values()
                        .filter(|node| {
                            node.kind != NodeKind::Pending && !self.is_observer(&node.key)
                        })
                        .map(|node| &node.key),
                ),
            }
        }
    }

    /// Whether `from` can reach `to` by following dependent edges, which is what
    /// makes a new edge circular.
    #[must_use]
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
