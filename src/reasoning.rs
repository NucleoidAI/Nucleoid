use indexmap::{IndexMap, IndexSet};

use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::nuc::Nuc;
use crate::nuc::property::Owner;
use crate::runtime::Runtime;
use crate::value::{ObjectId, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Why,
    Affects,
}

impl Stage {
    pub fn from_name(name: &str) -> Option<Stage> {
        match name {
            "why" => Some(Stage::Why),
            "affects" => Some(Stage::Affects),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Why => "why",
            Stage::Affects => "affects",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Stated,
    Derived,
}

impl StepState {
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

impl Selection {
    pub fn keys(&self) -> Vec<NodeKey> {
        self.steps.iter().map(|step| step.node.clone()).collect()
    }
}

impl Runtime {
    pub(crate) fn selection_of(&mut self, keys: Vec<NodeKey>) -> Selection {
        let mut steps = Vec::new();

        for key in keys {
            if self.is_observer(&key) {
                continue;
            }

            if let Some(step) = self.step(&key) {
                steps.push(step);
            }
        }

        Selection { steps }
    }

    pub(crate) fn model_keys(&self) -> Vec<NodeKey> {
        self.graph.keys().cloned().collect()
    }

    pub(crate) fn apply_stage(&mut self, stage: Stage, selection: &Selection) -> Selection {
        let mut reached: IndexSet<NodeKey> = IndexSet::new();
        let mut pending: Vec<NodeKey> = Vec::new();

        for step in &selection.steps {
            if stage == Stage::Why {
                reached.insert(step.node.clone());
            }
            pending.push(step.node.clone());
        }

        while let Some(current) = pending.pop() {
            let Some(node) = self.graph.retrieve(&current) else {
                continue;
            };

            let next: Vec<NodeKey> = match stage {
                Stage::Why => node.dependencies.iter().cloned().collect(),
                Stage::Affects => self.graph.dependents_in_order(&current),
            };

            for key in next {
                if reached.insert(key.clone()) {
                    pending.push(key);
                }
            }
        }

        self.selection_of(reached.into_iter().collect())
    }

    fn is_observer(&self, key: &NodeKey) -> bool {
        self.graph
            .retrieve(key)
            .and_then(|node| node.node.as_ref())
            .map(is_reasoning)
            .unwrap_or(false)
    }

    fn step(&mut self, key: &NodeKey) -> Option<Step> {
        let node = self.graph.retrieve(key)?;

        if node.kind == NodeKind::Pending {
            return None;
        }

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
        self.track(key.clone());

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
