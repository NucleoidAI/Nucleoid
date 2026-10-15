//! Complete runtime snapshots modulo fresh anonymous object identities.
//!
//! ObjectId::anonymous uses one process-global counter, so two runtimes cannot
//! share its absolute numbers. Intern only typed ObjectId roots, preserve their
//! aliasing and nested paths, and compare every remaining state/graph field.

use std::collections::HashMap;

use nucleoid::value::ObjectId;
use nucleoid::{Runtime, Value};
use serde_json::json;

#[derive(Default)]
struct Identities {
    roots: HashMap<String, String>,
    objects: Vec<ObjectId>,
}

impl Identities {
    fn root(&mut self, root: &str) -> String {
        let next = self.roots.len();
        self.roots
            .entry(root.to_string())
            .or_insert_with(|| format!("anonymous:{next}"))
            .clone()
    }

    fn typed(&mut self, debug: &str) -> String {
        let mut result = String::new();
        let mut remaining = debug;
        while let Some((before, after)) = remaining.split_once("ObjectId(\"") {
            result.push_str(before);
            result.push_str("ObjectId(\"");
            if let Some(tail) = after.strip_prefix("obj:") {
                let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
                if digits > 0 && matches!(tail.as_bytes().get(digits), Some(b'"' | b'.')) {
                    let end = 4 + digits;
                    let object = ObjectId::named(&after[..after.find('"').unwrap()]);
                    if !self.objects.contains(&object) {
                        self.objects.push(object);
                    }
                    result.push_str(&self.root(&after[..end]));
                    remaining = &after[end..];
                    continue;
                }
            }
            remaining = after;
        }
        result.push_str(remaining);
        result
    }

    fn key(&mut self, key: &str) -> String {
        if let Some(tail) = key.strip_prefix("obj:") {
            let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
            if digits > 0 && (tail.len() == digits || tail.as_bytes()[digits] == b'.') {
                let end = 4 + digits;
                return format!("{}{}", self.root(&key[..end]), &key[end..]);
            }
        }
        if let Some((source, instance)) = key.rsplit_once('@') {
            return format!("{source}@{}", self.key(instance));
        }
        key.to_string()
    }

    fn json_ids(&mut self, value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Array(values) => {
                for value in values {
                    self.json_ids(value);
                }
            }
            serde_json::Value::Object(fields) => {
                if let Some(serde_json::Value::String(id)) = fields.get_mut("id") {
                    *id = self.key(id);
                }
                for value in fields.values_mut() {
                    self.json_ids(value);
                }
            }
            _ => {}
        }
    }
}

fn snapshot(runtime: &Runtime, value: Option<&Value>) -> serde_json::Value {
    let mut ids = Identities::default();
    ids.typed(&format!("{:?}", runtime.state));
    let mut state = runtime.state.clone();
    for object in ids.objects.clone() {
        if let Some(data) = state.object_mut(&object) {
            // Class instances also store their identity as a string property.
            // Rename only valid self-ID metadata, never arbitrary string data
            // or a mismatched/corrupted `id` property.
            if data.properties.get("id") == Some(&Value::String(object.to_string())) {
                data.properties
                    .insert("id".to_string(), Value::String(ids.key(object.as_str())));
            }
        }
    }
    let state = ids.typed(&format!("{state:?}"));
    let graph = runtime
        .graph
        .nodes()
        .map(|node| {
            json!({
                "key": ids.key(node.key.as_str()),
                "kind": node.kind.as_str(),
                "instance": ids.typed(&format!("{:?}", node.instance)),
                "sequence": node.sequence,
                "node": ids.typed(&format!("{:?}", node.node)),
                "dependencies": node.dependencies.iter().map(|key| ids.key(key.as_str())).collect::<Vec<_>>(),
                "dependents": node.dependents.iter().map(|key| ids.key(key.as_str())).collect::<Vec<_>>(),
                "shapes": runtime.graph.shape_dependencies(&node.key).map(|key| ids.key(&key.to_string())).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    let returned = value.map(|value| ids.typed(&format!("{value:?}")));
    let serialized = value.map(|value| {
        let mut value = serde_json::from_str(&runtime.serialize_json(value).unwrap()).unwrap();
        ids.json_ids(&mut value);
        value
    });
    json!({
        "state": state,
        "graph": graph,
        "returned": returned,
        "serialized": serialized,
        "assertions_run": runtime.assertions_run(),
    })
}

pub fn assert_equivalent(
    actual: &Runtime,
    actual_value: Option<&Value>,
    expected: &Runtime,
    expected_value: Option<&Value>,
) {
    assert_eq!(
        snapshot(actual, actual_value),
        snapshot(expected, expected_value)
    );
}
