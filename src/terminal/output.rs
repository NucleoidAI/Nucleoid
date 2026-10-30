use indexmap::IndexSet;
use nucleoid::{Runtime, Value};
use serde_json::Value as Json;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum View {
    Text,
    Json,
    Table,
}

impl View {
    pub const ALL: [Self; 3] = [Self::Text, Self::Json, Self::Table];

    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Json => "JSON",
            Self::Table => "Table",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Text => Self::Json,
            Self::Json => Self::Table,
            Self::Table => Self::Text,
        }
    }
}

pub(super) struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl TableData {
    fn from_json(value: &Json) -> Self {
        match value {
            Json::Array(values) if !values.is_empty() && values.iter().all(Json::is_object) => {
                let headers: IndexSet<String> = values
                    .iter()
                    .filter_map(Json::as_object)
                    .flat_map(|object| object.keys().cloned())
                    .collect();
                if headers.is_empty() {
                    return Self::indexed(values);
                }
                let rows = values
                    .iter()
                    .map(|value| {
                        headers
                            .iter()
                            .map(|key| value.get(key).map(cell).unwrap_or_default())
                            .collect()
                    })
                    .collect();
                Self {
                    headers: headers.into_iter().collect(),
                    rows,
                }
            }
            Json::Array(values) => Self::indexed(values),
            Json::Object(properties) => Self {
                headers: vec!["Property".into(), "Value".into()],
                rows: properties
                    .iter()
                    .map(|(key, value)| vec![visible(key, false), cell(value)])
                    .collect(),
            },
            value => Self {
                headers: vec!["Value".into()],
                rows: vec![vec![cell(value)]],
            },
        }
    }

    fn indexed(values: &[Json]) -> Self {
        Self {
            headers: vec!["Index".into(), "Value".into()],
            rows: values
                .iter()
                .enumerate()
                .map(|(index, value)| vec![index.to_string(), cell(value)])
                .collect(),
        }
    }
}

pub(super) enum Output {
    Value {
        text: String,
        structured: Result<(String, TableData), String>,
    },
    Message(String),
    Error(String),
}

impl Output {
    fn value(runtime: &Runtime, value: &Value) -> Self {
        let structured = runtime
            .serialize_json(value)
            .map_err(|error| error.to_string())
            .and_then(|json| serde_json::from_str::<Json>(&json).map_err(|error| error.to_string()))
            .and_then(|json| {
                serde_json::to_string_pretty(&json)
                    .map(|pretty| (pretty, TableData::from_json(&json)))
                    .map_err(|error| error.to_string())
            });
        Self::Value {
            text: visible(&value.to_string(), true),
            structured,
        }
    }

    pub fn text(&self, view: View) -> &str {
        match self {
            Self::Value {
                structured: Ok((json, _)),
                ..
            } if view != View::Text => json,
            Self::Value {
                structured: Err(error),
                ..
            } if view != View::Text => error,
            Self::Value { text, .. } | Self::Message(text) | Self::Error(text) => text,
        }
    }

    pub fn table(&self) -> Option<&TableData> {
        match self {
            Self::Value {
                structured: Ok((_, table)),
                ..
            } => Some(table),
            _ => None,
        }
    }

    pub fn is_error(&self, view: View) -> bool {
        matches!(self, Self::Error(_))
            || (view != View::Text
                && matches!(
                    self,
                    Self::Value {
                        structured: Err(_),
                        ..
                    }
                ))
    }
}

pub(super) struct Entry {
    pub source: String,
    pub output: Output,
    pub diagnostics: String,
}

impl Entry {
    pub fn execute(runtime: &mut Runtime, source: String, graph: bool) -> Self {
        let output = match runtime.run(&source) {
            Ok(value) => Output::value(runtime, &value),
            Err(error) => Output::Error(visible(&error.to_string(), true)),
        };
        let mut diagnostics = Vec::new();
        for failure in runtime.take_assertions() {
            diagnostics.push(format!(
                "assertion failed: expected {}, got {}",
                failure.expected, failure.actual
            ));
        }
        if graph {
            diagnostics.push(format!("logic graph ({} nodes)", runtime.graph.len()));
            for node in runtime.graph.nodes() {
                diagnostics.push(format!("  {} [{}]", node.key, node.kind));
                for (label, names) in [
                    (
                        "reads",
                        node.dependencies
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>(),
                    ),
                    (
                        "observes",
                        runtime
                            .graph
                            .shape_dependencies(&node.key)
                            .map(ToString::to_string)
                            .collect(),
                    ),
                    (
                        "updates",
                        node.dependents.iter().map(ToString::to_string).collect(),
                    ),
                ] {
                    if !names.is_empty() {
                        diagnostics.push(format!("    {label}: {}", names.join(", ")));
                    }
                }
            }
        }
        Self {
            source,
            output,
            diagnostics: visible(&diagnostics.join("\n"), true),
        }
    }
}

fn cell(value: &Json) -> String {
    match value {
        Json::String(text) => visible(text, false),
        value => value.to_string(),
    }
}

pub(super) fn visible(text: &str, multiline: bool) -> String {
    text.chars()
        .flat_map(|character| {
            if character.is_control() && !(multiline && character == '\n') {
                character.escape_default().collect::<Vec<_>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn object_rows_union_columns_without_losing_null_or_missing_values() {
        let table = TableData::from_json(&json!([
            {"name": "Ada", "score": null},
            {"name": "Lin", "active": true}
        ]));
        assert_eq!(table.headers, ["name", "score", "active"]);
        assert_eq!(table.rows[0], ["Ada", "null", ""]);
        assert_eq!(table.rows[1], ["Lin", "", "true"]);
    }

    #[test]
    fn tables_preserve_mixed_nested_scalar_and_empty_results() {
        for value in [
            json!([1, {"nested": [2, 3]}, null]),
            json!([{}, {}]),
            json!([]),
        ] {
            let table = TableData::from_json(&value);
            assert_eq!(table.headers, ["Index", "Value"]);
            assert_eq!(table.rows.len(), value.as_array().unwrap().len());
        }
        let object = TableData::from_json(&json!({"line": "a\nb", "nested": {"x": 1}}));
        assert_eq!(object.headers, ["Property", "Value"]);
        assert_eq!(object.rows[0], ["line", "a\\nb"]);
        assert_eq!(object.rows[1], ["nested", "{\"x\":1}"]);
        assert_eq!(TableData::from_json(&json!(false)).rows, [vec!["false"]]);
    }

    #[test]
    fn results_resolve_objects_and_freeze_them_before_later_mutations() {
        let mut runtime = Runtime::new();
        let entry = Entry::execute(&mut runtime, "item = {count: 1}\n[item]".into(), false);
        runtime.run("item.count = 2").unwrap();
        let table = entry.output.table().unwrap();
        assert_eq!(table.headers, ["count"]);
        assert_eq!(table.rows, [vec!["1.0"]]);
        assert!(entry.output.text(View::Json).contains("1.0"));
    }

    #[test]
    fn serialization_failures_assertions_and_runtime_errors_stay_visible() {
        let mut runtime = Runtime::new();
        let cycle = Entry::execute(
            &mut runtime,
            "item = {}\nitem.self = item\nitem".into(),
            false,
        );
        assert!(!cycle.output.is_error(View::Text));
        assert!(cycle.output.is_error(View::Json));
        assert!(cycle.output.text(View::Json).contains("circular"));
        let assertions = Entry::execute(&mut runtime, "assert(1, 2)".into(), true);
        assert!(assertions.diagnostics.contains("assertion failed"));
        assert!(assertions.diagnostics.contains("logic graph"));
        assert!(runtime.take_assertions().is_empty());
        let error = Entry::execute(&mut runtime, "value = (".into(), false);
        assert!(error.output.is_error(View::Text));
    }

    #[test]
    fn terminal_control_sequences_are_displayed_not_executed() {
        assert_eq!(visible("a\u{1b}[2J\r\nb", true), "a\\u{1b}[2J\\r\nb");
        assert_eq!(cell(&json!("a\tb\nc")), "a\\tb\\nc");
    }
}
