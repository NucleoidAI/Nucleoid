use std::collections::{HashMap, HashSet};

use super::undo::Undo;
use crate::graph::{GraphNode, NodeKey, ShapeKey};
use crate::value::{ObjectData, Value};

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
    pub(super) fn record(&mut self, index: usize, entry: &'t Undo) {
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

    pub(super) fn settle(mut self) -> Self {
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
