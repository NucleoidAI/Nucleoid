use indexmap::IndexMap;

use super::Selection;
use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::nuc::Nuc;
use crate::nuc::property::Owner;
use crate::runtime::Runtime;
use crate::transaction::{Origin, Start};
use crate::value::{ObjectId, Value};

pub(super) fn held<'t>(origin: &Origin<'t>, key: &NodeKey, kind: NodeKind) -> Start<&'t Value> {
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

pub(super) fn rule_of(node: Option<&Nuc>, key: &NodeKey) -> String {
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

impl Runtime {
    pub(super) fn class_rule(
        &self,
        instance: &ObjectId,
        node: Option<&Nuc>,
        key: &NodeKey,
    ) -> String {
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

    pub(super) fn node_value(&self, key: &NodeKey, kind: NodeKind) -> Value {
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
