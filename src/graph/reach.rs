use indexmap::IndexSet;

use super::{Graph, NodeKey, NodeKind, ShapeKey};

impl Graph {
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

    /// Whether a queued statement, this one included, is upstream of this one
    /// along a path that crosses an edge other than a fixed anchor. Statements
    /// that are upstream of each other only through fixed anchors, the only
    /// edges that may close a cycle, need not wait for each other. Each node
    /// is visited at most twice: before and after its path crosses such an
    /// edge.
    #[must_use]
    pub(crate) fn waits_beyond_anchors(
        &self,
        key: &NodeKey,
        candidates: &IndexSet<NodeKey>,
    ) -> bool {
        let mut seen = IndexSet::new();
        let mut pending = Vec::new();

        self.upstream_into(key, false, &mut pending);

        while let Some((current, crossed)) = pending.pop() {
            if crossed && candidates.contains(current) {
                return true;
            }

            if seen.insert((current, crossed)) {
                self.upstream_into(current, crossed, &mut pending);
            }
        }

        false
    }

    fn dependencies_into<'a>(&'a self, key: &NodeKey, pending: &mut Vec<&'a NodeKey>) {
        if let Some(node) = self.nodes.get(key) {
            pending.extend(&node.dependencies);
        }

        self.shape_sources_into(key, pending);
    }

    /// The upstream neighbours of `key`, each with whether the path to it has
    /// crossed an edge other than a fixed anchor. Structural observations are
    /// never fixed anchors.
    fn upstream_into<'a>(
        &'a self,
        key: &NodeKey,
        crossed: bool,
        pending: &mut Vec<(&'a NodeKey, bool)>,
    ) {
        if let Some(node) = self.nodes.get(key) {
            pending.extend(
                node.dependencies
                    .iter()
                    .map(|dependency| (dependency, crossed || !node.anchors_on(dependency))),
            );
        }

        let mut sources = Vec::new();
        self.shape_sources_into(key, &mut sources);
        pending.extend(sources.into_iter().map(|source| (source, true)));
    }

    /// The nodes whose structure `key` observes.
    fn shape_sources_into<'a>(&'a self, key: &NodeKey, pending: &mut Vec<&'a NodeKey>) {
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
                ShapeKey::Entries(_) => {}
            }
        }
    }

    /// Whether `reader` reads `source` only at fixed history positions of its
    /// own class receiver.
    fn anchored(&self, source: &NodeKey, reader: &NodeKey) -> bool {
        self.nodes
            .get(reader)
            .is_some_and(|node| node.anchors_on(source))
    }

    /// Whether `from` reaches `to` along dependent edges of which at least one
    /// is an ordinary read, so that a new fixed-anchor edge from `to` back to
    /// `from` would close a cycle through an ordinary read. Each node is
    /// visited at most twice: before and after its path crosses one.
    #[must_use]
    pub(crate) fn reaches_through_ordinary(&self, from: &NodeKey, to: &NodeKey) -> bool {
        let mut seen = IndexSet::new();
        let mut pending = vec![(from, false)];

        while let Some((current, ordinary)) = pending.pop() {
            if ordinary && current == to {
                return true;
            }

            if !seen.insert((current, ordinary)) {
                continue;
            }

            if let Some(node) = self.nodes.get(current) {
                for dependent in &node.dependents {
                    let crossed = ordinary || !self.anchored(current, dependent);
                    pending.push((dependent, crossed));
                }
            }
        }

        false
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
