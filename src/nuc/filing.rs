use indexmap::IndexSet;

use super::Nuc;
use crate::error::{Error, Result};
use crate::graph::{GraphNode, NodeKey, NodeKind, ShapeKey};
use crate::runtime::Runtime;
use crate::value::ObjectId;

impl Runtime {
    /// Files a node under `key` and wires the edges to everything it read.
    ///
    /// Checks new edges before calling [`Runtime::register`] or
    /// [`Runtime::replace`], then wires dependencies with [`Runtime::direct`].
    pub(crate) fn file(
        &mut self,
        key: &NodeKey,
        kind: NodeKind,
        nuc: Option<Nuc>,
        dependencies: IndexSet<NodeKey>,
        instance: Option<ObjectId>,
    ) -> Result<()> {
        self.file_observed(key, kind, nuc, dependencies, instance, IndexSet::new())
    }

    pub(crate) fn file_observed(
        &mut self,
        key: &NodeKey,
        kind: NodeKind,
        nuc: Option<Nuc>,
        dependencies: IndexSet<NodeKey>,
        instance: Option<ObjectId>,
        shapes: IndexSet<ShapeKey>,
    ) -> Result<()> {
        let existing = self.graph.retrieve(key).cloned();
        let was_visible = existing
            .as_ref()
            .is_some_and(|node| node.kind != NodeKind::Pending && !self.graph.is_observer(key));

        self.check_new_edges(key, &dependencies, nuc.as_ref(), existing.as_ref())?;

        let sequence = match &existing {
            Some(existing) if self.stack.is_running(key) => existing.sequence,
            _ => self.graph.next_sequence(),
        };
        let mut node = GraphNode::new(key.clone(), kind, sequence);
        node.node = nuc;
        node.instance = instance;
        node.dependencies.clone_from(&dependencies);

        match &existing {
            Some(existing) => self.replace(node, existing),
            None => self.register(node),
        }

        for dependency in &dependencies {
            if dependency == key {
                continue;
            }

            self.direct(dependency, key);
        }

        self.replace_shape_dependencies(key, shapes);

        let is_visible = kind != NodeKind::Pending && !self.graph.is_observer(key);
        let shape_changed = !self.stack.is_running(key)
            || was_visible != is_visible
            || existing.as_ref().is_none_or(|previous| {
                previous.kind != kind || !previous.dependencies.iter().eq(dependencies.iter())
            });

        if (was_visible || is_visible) && shape_changed {
            self.queue_shape_change(&ShapeKey::Node(key.clone()));

            let mut sources = IndexSet::new();
            if was_visible {
                if let Some(existing) = &existing {
                    sources.extend(existing.dependencies.iter().cloned());
                }
            }
            if is_visible {
                sources.extend(dependencies);
            }
            for source in sources {
                self.queue_shape_change(&ShapeKey::Node(source));
            }

            if was_visible != is_visible {
                self.queue_shape_change(&ShapeKey::Model);
            }
        }

        Ok(())
    }

    /// A fixed anchor may close a cycle only when every other edge on it is
    /// one too; any other new edge may close none.
    fn check_new_edges(
        &self,
        key: &NodeKey,
        dependencies: &IndexSet<NodeKey>,
        nuc: Option<&Nuc>,
        existing: Option<&GraphNode>,
    ) -> Result<()> {
        for dependency in dependencies {
            if dependency == key {
                continue;
            }

            let anchored = nuc.is_some_and(|nuc| nuc.anchors_on(dependency));

            // An edge that is already there was checked when it was made, and
            // re-checking costs a walk of everything downstream — which is the
            // whole graph, on every re-evaluation. An anchor that has become an
            // ordinary read is checked again.
            let unchanged = existing.is_some_and(|node| {
                node.dependencies.contains(dependency) && (anchored || !node.anchors_on(dependency))
            });

            if unchanged {
                continue;
            }

            let cycle = if anchored {
                self.graph.reaches_through_ordinary(key, dependency)
            } else {
                self.graph.reaches(key, dependency)
            };

            if cycle {
                return Err(Error::type_error("Circular Dependency"));
            }
        }

        Ok(())
    }

    fn replace_shape_dependencies(&mut self, key: &NodeKey, mut shapes: IndexSet<ShapeKey>) {
        shapes.shift_remove(&ShapeKey::Node(key.clone()));
        let previous: Vec<ShapeKey> = self.graph.shape_dependencies(key).cloned().collect();

        for source in previous {
            if !shapes.contains(&source) && self.graph.remove_shape_dependency(&source, key) {
                self.transaction
                    .record_shape_dependent_removed(&source, key);
            }
        }

        for source in shapes {
            if self.graph.add_shape_dependency(&source, key) {
                self.transaction.record_shape_dependent_added(&source, key);
            }
        }
    }

    /// Puts a node in the graph under its own key.
    pub(crate) fn register(&mut self, node: GraphNode) {
        if self.transaction.needs_node(&node.key) {
            self.transaction.record_node(&node.key, None);
        }

        self.graph.insert(node);
    }

    /// Puts a node where one already stood, so that a
    /// redeclaration keeps the edges pointing at it.
    pub(crate) fn replace(&mut self, mut node: GraphNode, existing: &GraphNode) {
        let key = node.key.clone();

        if self.transaction.needs_node(&key) {
            self.transaction.record_node(&key, Some(existing.clone()));
        }

        node.dependents.clone_from(&existing.dependents);

        // Drop edges from dependencies this declaration no longer has.
        for previous in &existing.dependencies {
            if !node.dependencies.contains(previous) {
                let removed = self
                    .graph
                    .get_mut(previous)
                    .is_some_and(|source| source.dependents.shift_remove(&key));

                if removed {
                    self.transaction.record_dependent_removed(previous, &key);
                }
            }
        }

        self.graph.insert(node);
    }

    /// Wires one edge, from what was read to what read it. A
    /// name that has not been defined yet still gets a node, so the edge is
    /// there waiting when it is.
    pub(crate) fn direct(&mut self, source: &NodeKey, target: &NodeKey) {
        if !self.graph.contains(source) {
            let sequence = self.graph.next_sequence();
            let pending = GraphNode::new(source.clone(), NodeKind::Pending, sequence);
            self.transaction.record_node(source, None);
            self.graph.insert(pending);
        }

        let added = self
            .graph
            .get_mut(source)
            .is_some_and(|node| node.dependents.insert(target.clone()));

        if added {
            self.transaction.record_dependent_added(source, target);
        }
    }

    /// Puts a placeholder where a node stood, keeping its dependents so they
    /// are still woken when the name is defined again.
    pub(crate) fn remove_node(&mut self, key: &NodeKey) {
        if let Some(node) = self.graph.retrieve(key).cloned() {
            let was_visible = node.kind != NodeKind::Pending && !self.graph.is_observer(key);
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
            placeholder.dependents = node.dependents;
            self.graph.insert(placeholder);
            self.replace_shape_dependencies(key, IndexSet::new());

            self.queue_shape_change(&ShapeKey::Node(key.clone()));
            if was_visible {
                for source in node.dependencies {
                    self.queue_shape_change(&ShapeKey::Node(source));
                }
                self.queue_shape_change(&ShapeKey::Model);
            }
        }
    }
}
