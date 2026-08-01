//! Shared harness for the case files written in the language itself.
//!
//! Each case is separated by `---`, titled by its first comment line, and may
//! end with a `# return:` comment giving the value the program evaluates to.

// Each integration test compiles this module on its own, so not every
// binary uses every helper.
#![allow(dead_code)]

use nucleoid::value::{ObjectId, Value};
use nucleoid::{Runtime, state::State};

pub struct Case {
    pub title: String,
    pub source: String,
    pub expected: Option<String>,
}

pub fn cases(document: &str) -> Vec<Case> {
    // Case files may arrive with either line ending.
    let document = document.replace("\r\n", "\n");

    let body = document
        .as_str()
        .split("```")
        .nth(1)
        .expect("cases live in one fenced block");

    body.split("\n---\n")
        .filter(|block| !block.trim().is_empty())
        .map(|block| {
            let title = block
                .lines()
                .map(str::trim)
                .find(|line| line.starts_with('#'))
                .unwrap_or("untitled")
                .trim_start_matches('#')
                .trim()
                .to_string();

            let expected = block.lines().map(str::trim).find_map(|line| {
                line.strip_prefix("# return:")
                    .map(|rest| rest.trim().to_string())
            });

            let source = block
                .lines()
                .filter(|line| !line.trim().starts_with("# return:"))
                .collect::<Vec<_>>()
                .join("\n");

            Case {
                title,
                source,
                expected,
            }
        })
        .collect()
}

/// Projects a runtime value into JSON so it can be compared with the literal
/// written in the spec.
fn to_json(state: &State, value: &Value) -> serde_json::Value {
    match value {
        Value::Undefined | Value::Null => serde_json::Value::Null,
        Value::Bool(bool) => serde_json::Value::Bool(*bool),
        Value::Number(number) => serde_json::Number::from_f64(*number)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(string) => serde_json::Value::String(string.clone()),
        Value::List(items) => {
            serde_json::Value::Array(items.iter().map(|item| to_json(state, item)).collect())
        }
        Value::Object(id) => object_json(state, id),
        Value::Class(name) => serde_json::Value::String(name.clone()),
        Value::Date(millis) => serde_json::Value::String(millis.to_string()),
        Value::Regex(regex) => serde_json::Value::String(regex.as_str().to_string()),
        Value::Function(_) => serde_json::Value::String("[function]".to_string()),
    }
}

fn object_json(state: &State, id: &ObjectId) -> serde_json::Value {
    let mut map = serde_json::Map::new();

    if let Some(object) = state.object(id) {
        for (name, value) in &object.properties {
            if value.is_undefined() {
                continue;
            }
            map.insert(name.clone(), to_json(state, value));
        }
    }

    serde_json::Value::Object(map)
}

/// Compares against the spec's expected value, treating `[UUID]` as a wildcard.
fn matches(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
        (_, serde_json::Value::String(text)) if text == "[UUID]" => true,
        (serde_json::Value::Object(actual), serde_json::Value::Object(expected)) => {
            actual.len() == expected.len()
                && expected.iter().all(|(key, expected)| {
                    actual
                        .get(key)
                        .map(|actual| matches(actual, expected))
                        .unwrap_or(false)
                })
        }
        (serde_json::Value::Array(actual), serde_json::Value::Array(expected)) => {
            actual.len() == expected.len()
                && actual
                    .iter()
                    .zip(expected.iter())
                    .all(|(actual, expected)| matches(actual, expected))
        }
        (serde_json::Value::Number(actual), serde_json::Value::Number(expected)) => {
            actual.as_f64() == expected.as_f64()
        }
        _ => actual == expected,
    }
}

pub fn check(case: &Case) -> Result<(), String> {
    let mut runtime = Runtime::new();

    let value = runtime
        .run(&case.source)
        .map_err(|error| format!("{error}"))?;

    let expected_assertions = case
        .source
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .map(|line| line.matches("assert(").count())
        .sum::<usize>();

    if runtime.assertions_run() < expected_assertions {
        return Err(format!(
            "only {} of {} assertions ran; the rest are on a branch that was never taken",
            runtime.assertions_run(),
            expected_assertions
        ));
    }

    let failures = runtime.take_assertions();

    if !failures.is_empty() {
        let rendered: Vec<String> = failures
            .iter()
            .map(|failure| format!("expected {}, got {}", failure.expected, failure.actual))
            .collect();
        return Err(format!(
            "{} assertion(s): {}",
            failures.len(),
            rendered.join("; ")
        ));
    }

    if let Some(expected) = &case.expected {
        let expected: serde_json::Value = serde_json::from_str(expected)
            .map_err(|error| format!("spec return value is not JSON: {error}"))?;
        let actual = to_json(&runtime.state, &value);

        if !matches(&actual, &expected) {
            return Err(format!("returned {actual}, expected {expected}"));
        }
    }

    Ok(())
}

/// Runs every case in a document, reporting how many pass.
pub fn run(name: &str, document: &str) {
    let cases = cases(document);
    let mut failures = Vec::new();

    let trace = std::env::var_os("NUCLEOID_TRACE").is_some();

    for case in &cases {
        if trace {
            println!("  running {}", case.title);
        }

        if let Err(reason) = check(case) {
            failures.push((case.title.clone(), reason));
        }
    }

    let passed = cases.len() - failures.len();
    println!(
        "
{name}: {passed}/{} cases pass",
        cases.len()
    );

    for (title, reason) in &failures {
        println!(
            "  FAIL {title}
       {reason}"
        );
    }

    assert!(
        failures.is_empty(),
        "{} of {} {name} cases fail",
        failures.len(),
        cases.len()
    );
}
