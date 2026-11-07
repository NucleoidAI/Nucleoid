use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};

use indexmap::{IndexMap, IndexSet};

use crate::error::{Error, Result};
use crate::graph::{GraphNode, NodeKey, NodeKind, ShapeKey};
use crate::nuc::Nuc;
use crate::nuc::property::Owner;
use crate::runtime::Runtime;
use crate::transaction::{Origin, Start};
use crate::value::{ObjectId, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Why,
    Affects,
    Stated,
    Derived,
    Diff,
}

impl Stage {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Stage> {
        match name {
            "why" => Some(Stage::Why),
            "affects" => Some(Stage::Affects),
            "stated" => Some(Stage::Stated),
            "derived" => Some(Stage::Derived),
            _ => None,
        }
    }

    pub(crate) fn direction(self) -> Option<Direction> {
        match self {
            Stage::Why => Some(Direction::Dependencies),
            Stage::Affects => Some(Direction::Dependents),
            Stage::Stated | Stage::Derived | Stage::Diff => None,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Why => "why",
            Stage::Affects => "affects",
            Stage::Stated => "stated",
            Stage::Derived => "derived",
            Stage::Diff => "diff",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Stated,
    Derived,
}

impl StepState {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            StepState::Stated => "stated",
            StepState::Derived => "derived",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Step {
    pub node: NodeKey,
    pub value: Value,
    pub rule: String,
    pub state: StepState,
    pub from: Vec<NodeKey>,
}

#[derive(Debug, Clone, Default)]
pub struct Selection {
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    Dependencies,
    Dependents,
}

fn filtered(selection: &Selection, state: StepState) -> Selection {
    // The source already observed every step, including excluded ones.
    Selection {
        steps: selection
            .steps
            .iter()
            .filter(|step| step.state == state)
            .cloned()
            .collect(),
    }
}

impl Selection {
    #[must_use]
    pub fn keys(&self) -> Vec<NodeKey> {
        self.steps.iter().map(|step| step.node.clone()).collect()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Ranked {
    selection: Selection,
    distances: Vec<usize>,
    direction: Direction,
}

impl Ranked {
    pub(crate) fn unwalked(selection: Selection) -> Self {
        let distances = vec![0; selection.steps.len()];
        Ranked {
            selection,
            distances,
            direction: Direction::Dependencies,
        }
    }

    pub(crate) fn within(self, limit: usize) -> Self {
        self.retained(|_, distance| distance <= limit)
    }

    pub(crate) fn into_selection(self) -> Selection {
        self.selection
    }

    fn filtered(self, state: StepState) -> Self {
        self.retained(|step, _| step.state == state)
    }

    fn retained(self, keep: impl Fn(&Step, usize) -> bool) -> Self {
        debug_assert_eq!(self.selection.steps.len(), self.distances.len());
        let (steps, distances) = self
            .selection
            .steps
            .into_iter()
            .zip(self.distances)
            .filter(|(step, distance)| keep(step, *distance))
            .unzip();
        Ranked {
            selection: Selection { steps },
            distances,
            direction: self.direction,
        }
    }
}

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
    pub(crate) fn selection_of(&mut self, keys: Vec<NodeKey>) -> Selection {
        let mut steps = Vec::new();

        for key in keys {
            self.track_shape(&ShapeKey::Node(key.clone()));

            if let Some(step) = self.step(&key) {
                steps.push(step);
            }
        }

        Selection { steps }
    }

    pub(crate) fn model_keys(&self) -> Vec<NodeKey> {
        self.graph
            .nodes()
            .filter(|node| self.selectable(node))
            .map(|node| node.key.clone())
            .collect()
    }

    fn selectable(&self, node: &GraphNode) -> bool {
        node.kind != NodeKind::Pending && !self.graph.is_observer(&node.key)
    }

    pub(crate) fn apply_stage(&mut self, stage: Stage, selection: &Selection) -> Result<Selection> {
        Ok(match stage {
            Stage::Stated => filtered(selection, StepState::Stated),
            Stage::Derived => filtered(selection, StepState::Derived),
            Stage::Diff => {
                let origin = self.origin()?;
                Selection {
                    steps: selection
                        .steps
                        .iter()
                        .filter(|step| self.changed(&origin, step))
                        .cloned()
                        .collect(),
                }
            }
            Stage::Why => self.traverse(selection, Direction::Dependencies),
            Stage::Affects => self.traverse(selection, Direction::Dependents),
        })
    }

    pub(crate) fn apply_ranked(&mut self, stage: Stage, ranked: Ranked) -> Result<Ranked> {
        Ok(match stage {
            Stage::Stated => ranked.filtered(StepState::Stated),
            Stage::Derived => ranked.filtered(StepState::Derived),
            Stage::Diff => {
                let origin = self.origin()?;
                ranked.retained(|step, _| self.changed(&origin, step))
            }
            Stage::Why => self.walk(&ranked.selection, Direction::Dependencies),
            Stage::Affects => self.walk(&ranked.selection, Direction::Dependents),
        })
    }

    fn origin(&self) -> Result<Origin<'_>> {
        self.transaction
            .origin()
            .ok_or_else(|| Error::type_error("Cannot use 'diff' outside suppose"))
    }

    fn changed(&self, origin: &Origin<'_>, step: &Step) -> bool {
        let Some(kind) = self.start_kind(origin, &step.node) else {
            return true;
        };
        match held(origin, &step.node, kind) {
            Start::Unchanged => false,
            Start::Absent => !step.value.is_null(),
            Start::Held(value) => !value.same_value(&step.value),
        }
    }

    fn start_kind(&self, origin: &Origin<'_>, key: &NodeKey) -> Option<NodeKind> {
        let kind = match origin.node(key) {
            Start::Held(node) if !node.node.as_ref().is_some_and(is_reasoning) => node.kind,
            Start::Held(_) | Start::Absent => return None,
            Start::Unchanged => self.graph.retrieve(key)?.kind,
        };
        (kind != NodeKind::Pending && !origin.observed(key)).then_some(kind)
    }

    fn walk(&mut self, input: &Selection, direction: Direction) -> Ranked {
        let selection = self.traverse(input, direction);
        let shortest = self.distances(input, direction);
        debug_assert!(
            selection
                .steps
                .iter()
                .all(|step| shortest.contains_key(&step.node))
        );
        let distances = selection
            .steps
            .iter()
            .map(|step| shortest.get(&step.node).copied().unwrap_or_default())
            .collect();
        Ranked {
            selection,
            distances,
            direction,
        }
    }

    pub(crate) fn apply_path(
        &mut self,
        mut selection: Selection,
        target: &NodeKey,
        direction: Direction,
    ) -> Selection {
        let kept = self.reaching(&selection.steps, target, direction);
        selection.steps.retain(|step| kept.contains(&step.node));
        selection
    }

    pub(crate) fn ranked_path(&mut self, ranked: Ranked, target: &NodeKey) -> Ranked {
        let kept = self.reaching(&ranked.selection.steps, target, ranked.direction);
        ranked.retained(|step, _| kept.contains(&step.node))
    }

    fn reaching(
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

    fn traverse(&mut self, selection: &Selection, direction: Direction) -> Selection {
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

    fn distances(&self, input: &Selection, direction: Direction) -> HashMap<NodeKey, usize> {
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

    fn step(&mut self, key: &NodeKey) -> Option<Step> {
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

    fn class_rule(&self, instance: &ObjectId, node: Option<&Nuc>, key: &NodeKey) -> String {
        let class = self
            .state
            .object(instance)
            .and_then(|data| data.class.clone());

        match (class, node) {
            (Some(class), Some(Nuc::Property(property))) => {
                format!("${}.{} = {}", class, property.name, property.value)
            }
            _ => rule_of(node, key),
        }
    }

    fn node_value(&self, key: &NodeKey, kind: NodeKind) -> Value {
        match kind {
            NodeKind::Variable | NodeKind::Object => self
                .state
                .variable(key.as_str())
                .cloned()
                .unwrap_or(Value::Null),
            NodeKind::Property => {
                let text = key.as_str();

                match text.rsplit_once('.') {
                    Some((object, property)) => self
                        .state
                        .property(&ObjectId::from(object), property)
                        .cloned()
                        .unwrap_or(Value::Null),
                    None => Value::Null,
                }
            }
            _ => Value::Null,
        }
    }

    pub(crate) fn materialize(&mut self, selection: &Selection) -> Value {
        let values = selection
            .steps
            .iter()
            .map(|step| {
                let mut properties: IndexMap<String, Value> = IndexMap::new();
                properties.insert("node".to_string(), Value::string(step.node.to_string()));
                properties.insert("holds".to_string(), step.value.clone());
                properties.insert("rule".to_string(), Value::string(step.rule.clone()));
                properties.insert("state".to_string(), Value::string(step.state.as_str()));
                properties.insert(
                    "from".to_string(),
                    Value::List(
                        step.from
                            .iter()
                            .map(|key| Value::string(key.to_string()))
                            .collect(),
                    ),
                );

                self.create_anonymous(properties)
            })
            .collect();

        Value::List(values)
    }

    pub(crate) fn reference_key(&mut self, name: &str) -> Result<NodeKey> {
        let key = NodeKey::variable(name);

        if self.graph.contains(&key) {
            return Ok(key);
        }

        Err(Error::not_defined(name))
    }
}

#[must_use]
pub fn is_reasoning(node: &Nuc) -> bool {
    let value = match node {
        Nuc::Variable(variable) => &variable.value,
        Nuc::Let(binding) => &binding.value,
        Nuc::Property(property) => &property.value,
        Nuc::Expression(expression) => &expression.tokens,
        _ => return false,
    };

    value.contains_reasoning()
}

fn held<'t>(origin: &Origin<'t>, key: &NodeKey, kind: NodeKind) -> Start<&'t Value> {
    match kind {
        NodeKind::Variable | NodeKind::Object => origin.variable(key.as_str()),
        NodeKind::Property => key
            .as_str()
            .rsplit_once('.')
            .map_or(Start::Unchanged, |(object, property)| {
                origin.property(object, property)
            }),
        NodeKind::Pending
        | NodeKind::Class
        | NodeKind::Function
        | NodeKind::If
        | NodeKind::Block => Start::Absent,
    }
}

fn rule_of(node: Option<&Nuc>, key: &NodeKey) -> String {
    let Some(node) = node else {
        return key.to_string();
    };

    match node {
        Nuc::Variable(variable) => format!("{} = {}", variable.name, variable.value),
        Nuc::Let(binding) => format!("{} = {}", binding.name, binding.value),
        Nuc::Property(property) => match &property.owner {
            Owner::Object(id) => format!("{}.{} = {}", id, property.name, property.value),
            Owner::Class(class) => format!("${}.{} = {}", class, property.name, property.value),
        },
        _ => key.to_string(),
    }
}
