use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};

use indexmap::IndexSet;

use super::rules::rule_of;
use super::{Direction, End, Selection, Step, StepState};
use crate::graph::{NodeKey, ShapeKey};
use crate::runtime::Runtime;

#[derive(Debug, Default)]
struct Region {
    reached: HashMap<NodeKey, Vec<NodeKey>>,
}

impl Region {
    fn back_from(&self, target: &NodeKey) -> HashSet<NodeKey> {
        let start = self.reached.contains_key(target).then_some(target);
        self.trace_back(start).into_iter().cloned().collect()
    }

    fn origins(&self, steps: &[Step]) -> HashSet<NodeKey> {
        steps
            .iter()
            .filter(|step| self.reached.get(&step.node).is_some_and(Vec::is_empty))
            .map(|step| step.node.clone())
            .collect()
    }

    fn outcomes(&self, steps: &[Step]) -> HashSet<NodeKey> {
        let sources = steps.iter().flat_map(|step| self.sources(&step.node));
        let resting = self.trace_back(sources);
        steps
            .iter()
            .filter(|step| !resting.contains(&step.node))
            .map(|step| step.node.clone())
            .collect()
    }

    fn trace_back<'region>(
        &'region self,
        keys: impl IntoIterator<Item = &'region NodeKey>,
    ) -> HashSet<&'region NodeKey> {
        let mut traced: HashSet<&NodeKey> = HashSet::new();
        let mut returning: Vec<&NodeKey> = Vec::new();
        for key in keys {
            if traced.insert(key) {
                returning.push(key);
            }
        }

        while let Some(current) = returning.pop() {
            for source in self.sources(current) {
                if traced.insert(source) {
                    returning.push(source);
                }
            }
        }

        debug_assert!(traced.iter().all(|key| self.reached.contains_key(*key)));
        traced
    }

    fn sources(&self, key: &NodeKey) -> &[NodeKey] {
        self.reached.get(key).map(Vec::as_slice).unwrap_or_default()
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

    pub(super) fn ends(&mut self, steps: &[Step], end: End) -> HashSet<NodeKey> {
        let region = self.region(steps, Direction::Dependents);
        match end {
            End::Origins => region.origins(steps),
            End::Outcomes => region.outcomes(steps),
        }
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
