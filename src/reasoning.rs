mod rules;
mod traversal;

use std::collections::HashSet;

use crate::error::{Error, Result};
use crate::graph::{GraphNode, NodeKey, NodeKind, ShapeKey};
use crate::lang::ast::SetOperation;
use crate::nuc::Nuc;
use crate::runtime::Runtime;
use crate::transaction::{Origin, Start};
use crate::value::Value;
use rules::held;

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

    pub(crate) fn combined(mut self, operation: SetOperation, other: &Selection) -> Selection {
        let members = other.members();
        self.steps
            .retain(|step| operation.keeps(members.contains(&step.node)));
        self
    }

    fn members(&self) -> HashSet<&NodeKey> {
        self.steps.iter().map(|step| &step.node).collect()
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

    pub(crate) fn combined(self, operation: SetOperation, other: &Selection) -> Self {
        let members = other.members();
        self.retained(|step, _| operation.keeps(members.contains(&step.node)))
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
