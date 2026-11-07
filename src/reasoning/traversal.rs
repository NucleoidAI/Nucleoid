use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};

use indexmap::IndexSet;

use super::rules::rule_of;
use super::{Direction, Selection, Step, StepState};
use crate::graph::{NodeKey, ShapeKey};
use crate::runtime::Runtime;

#[derive(Debug, Default)]
struct Region {
    reached: HashMap<NodeKey, Vec<NodeKey>>,
}

impl Region {
    fn back_from(&self, target: &NodeKey) -> HashSet<NodeKey> {
        let mut kept: HashSet<&NodeKey> = HashSet::new();
        let mut returning: Vec<&NodeKey> = Vec::new();
        if let Some((key, _)) = self.reached.get_key_value(target) {
            kept.insert(key);
            returning.push(key);
        }

        while let Some(current) = returning.pop() {
            for source in self.reached.get(current).into_iter().flatten() {
                if kept.insert(source) {
                    returning.push(source);
                }
            }
        }

        debug_assert!(kept.iter().all(|key| self.reached.contains_key(*key)));
        kept.into_iter().cloned().collect()
    }
}

impl Runtime {
    pub(super) fn reaching(
        &mut self,
        steps: &[Step],
        target: &NodeKey,
        direction: Direction,
    ) -> HashSet<NodeKey> {
        self.track_shape(&ShapeKey::Node(target.clone()));
        if !self
            .graph
            .retrieve(target)
            .is_some_and(|node| self.selectable(node))
        {
            return HashSet::new();
        }
        self.region(steps, direction).back_from(target)
    }

    fn region(&mut self, steps: &[Step], direction: Direction) -> Region {
        let mut region = Region::default();
        let mut pending: Vec<NodeKey> = Vec::new();

        for step in steps {
            if let Entry::Vacant(entry) = region.reached.entry(step.node.clone()) {
                pending.push(entry.key().clone());
                entry.insert(Vec::new());
            }
        }

        while let Some(current) = pending.pop() {
            self.track_shape(&ShapeKey::Node(current.clone()));

            for key in self.neighbors(&current, direction) {
                if self.graph.is_observer(&key) {
                    continue;
                }

                match region.reached.entry(key) {
                    Entry::Occupied(mut sources) => sources.get_mut().push(current.clone()),
                    Entry::Vacant(entry) => {
                        pending.push(entry.key().clone());
                        entry.insert(vec![current.clone()]);
                    }
                }
            }
        }

        region
    }

    pub(super) fn traverse(&mut self, selection: &Selection, direction: Direction) -> Selection {
        let mut reached: IndexSet<NodeKey> = IndexSet::new();
        let mut pending: Vec<NodeKey> = Vec::new();

        for step in &selection.steps {
            if direction == Direction::Dependencies {
                reached.insert(step.node.clone());
            }
            pending.push(step.node.clone());
        }

        while let Some(current) = pending.pop() {
            self.track_shape(&ShapeKey::Node(current.clone()));

            for key in self.neighbors(&current, direction) {
                if self.graph.is_observer(&key) {
                    continue;
                }

                if reached.insert(key.clone()) {
                    pending.push(key);
                }
            }
        }

        self.selection_of(reached.into_iter().collect())
    }

    pub(super) fn distances(
        &self,
        input: &Selection,
        direction: Direction,
    ) -> HashMap<NodeKey, usize> {
        let mut shortest: HashMap<NodeKey, usize> = HashMap::new();
        let mut queued: HashSet<NodeKey> = HashSet::new();
        let mut pending: VecDeque<(NodeKey, usize)> = VecDeque::new();

        for step in &input.steps {
            if direction == Direction::Dependencies {
                shortest.insert(step.node.clone(), 0);
            }
            if queued.insert(step.node.clone()) {
                pending.push_back((step.node.clone(), 0));
            }
        }

        while let Some((current, distance)) = pending.pop_front() {
            for key in self.neighbors(&current, direction) {
                if self.graph.is_observer(&key) || shortest.contains_key(&key) {
                    continue;
                }

                shortest.insert(key.clone(), distance + 1);
                if queued.insert(key.clone()) {
                    pending.push_back((key, distance + 1));
                }
            }
        }

        shortest
    }

    fn neighbors(&self, key: &NodeKey, direction: Direction) -> Vec<NodeKey> {
        match direction {
            Direction::Dependencies => self
                .graph
                .retrieve(key)
                .map(|node| node.dependencies.iter().cloned().collect())
                .unwrap_or_default(),
            Direction::Dependents => self.graph.dependents_in_order(key),
        }
    }

    pub(super) fn step(&mut self, key: &NodeKey) -> Option<Step> {
        let node = self
            .graph
            .retrieve(key)
            .filter(|node| self.selectable(node))?;

        let from: Vec<NodeKey> = node.dependencies.iter().cloned().collect();
        let governed = node.instance.clone();
        let state = if from.is_empty() && governed.is_none() {
            StepState::Stated
        } else {
            StepState::Derived
        };
        let rule = match &governed {
            Some(instance) => self.class_rule(instance, node.node.as_ref(), key),
            None => rule_of(node.node.as_ref(), key),
        };
        let kind = node.kind;

        let value = self.node_value(key, kind);
        self.track(key);

        Some(Step {
            node: key.clone(),
            value,
            rule,
            state,
            from,
        })
    }
}
