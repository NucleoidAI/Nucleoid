//! Shared harness for the case files written in the language itself.
//!
//! Each case is separated by `---`, titled by its first comment line, and may
//! end with a `# return:` comment giving the value the program evaluates to.

// Each integration test compiles this module on its own, so not every
// binary uses every helper.
#![allow(dead_code)]

use nucleoid::Runtime;
use std::cell::RefCell;
use std::rc::Rc;

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

/// Every fenced `nuc` block in a prose document, titled by the bold claim
/// above it.
///
/// Deliberately the same rule as `snippet_titles` in `build.rs`; the two are
/// checked against each other by the title lookup and by each suite's coverage
/// test. `name` is the document the blocks came from, for the failure message.
pub fn snippets(document: &str, name: &str) -> Vec<Case> {
    let document = document.replace("\r\n", "\n");

    let mut cases: Vec<Case> = Vec::new();
    let mut claim = String::new();
    let mut source: Option<String> = None;

    for line in document.lines() {
        let trimmed = line.trim();

        if let Some(collected) = &mut source {
            if trimmed.starts_with("```") {
                let title = if claim.is_empty() {
                    format!("snippet {}", cases.len() + 1)
                } else {
                    claim.clone()
                };

                cases.push(Case {
                    title,
                    source: std::mem::take(collected),
                    expected: None,
                });

                source = None;
            } else {
                collected.push_str(line);
                collected.push('\n');
            }

            continue;
        }

        if trimmed.len() > 4 && trimmed.starts_with("**") && trimmed.ends_with("**") {
            claim = trimmed
                .trim_matches('*')
                .trim()
                .trim_end_matches('.')
                .to_string();
        }

        if trimmed.starts_with("```nuc") {
            source = Some(String::new());
        }
    }

    assert!(source.is_none(), "unterminated ```nuc block in {name}");

    cases
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

#[derive(Debug)]
pub struct TestValue {
    json: serde_json::Value,
    number: Option<f64>,
}

impl PartialEq for TestValue {
    fn eq(&self, other: &Self) -> bool {
        match (self.number, other.number) {
            (Some(actual), Some(expected)) => actual == expected,
            _ => matches(&self.json, &other.json),
        }
    }
}

impl PartialEq<serde_json::Value> for TestValue {
    fn eq(&self, other: &serde_json::Value) -> bool {
        matches(&self.json, other)
    }
}

impl PartialEq<bool> for TestValue {
    fn eq(&self, other: &bool) -> bool {
        matches(&self.json, &serde_json::Value::Bool(*other))
    }
}

impl PartialEq<i32> for TestValue {
    fn eq(&self, other: &i32) -> bool {
        self.number == Some(f64::from(*other))
    }
}

impl PartialEq<i64> for TestValue {
    fn eq(&self, other: &i64) -> bool {
        self.number == Some(*other as f64)
    }
}

impl PartialEq<f64> for TestValue {
    fn eq(&self, other: &f64) -> bool {
        self.number == Some(*other)
    }
}

impl PartialEq<&str> for TestValue {
    fn eq(&self, other: &&str) -> bool {
        matches(&self.json, &serde_json::Value::String((*other).to_string()))
    }
}

pub fn check(case: &Case) -> Result<(), String> {
    let mut runtime = Runtime::new();
    check_with(&mut runtime, case).map(|_| ())
}

fn check_with(runtime: &mut Runtime, case: &Case) -> Result<nucleoid::Value, String> {
    let assertions_before = runtime.assertions_run();
    let value = runtime
        .run(&case.source)
        .map_err(|error| format!("{error}"))?;

    let expected_assertions = case
        .source
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .map(|line| line.matches("assert(").count())
        .sum::<usize>();

    let assertions_run = runtime.assertions_run() - assertions_before;

    if assertions_run < expected_assertions {
        return Err(format!(
            "only {} of {} assertions ran; the rest are on a branch that was never taken",
            assertions_run, expected_assertions
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
        let actual = runtime
            .serialize_json(&value)
            .map_err(|error| error.to_string())
            .and_then(|json| {
                serde_json::from_str(&json)
                    .map_err(|error| format!("runtime returned invalid JSON: {error}"))
            })?;

        if !matches(&actual, &expected) {
            return Err(format!("returned {actual}, expected {expected}"));
        }
    }

    Ok(value)
}

/// Creates a stateful `run` function for one generated behavior test.
pub fn runner() -> impl FnMut(&str) -> TestValue {
    let mut runtime = Runtime::new();

    move |source| run_source(&mut runtime, source)
}

/// Creates stateful success and error runners that share one runtime.
pub fn runners() -> (impl FnMut(&str) -> TestValue, impl FnMut(&str) -> TestValue) {
    let runtime = Rc::new(RefCell::new(Runtime::new()));
    let success_runtime = Rc::clone(&runtime);

    let success = move |source: &str| run_source(&mut success_runtime.borrow_mut(), source);
    let failure = move |source: &str| {
        let mut runtime = runtime.borrow_mut();

        match runtime.run(source) {
            Ok(value) => panic!("expected an error, got {}", value),
            Err(error) => match error.thrown_value().cloned() {
                Some(value) => test_value(&runtime, value),
                None => TestValue {
                    json: serde_json::Value::String(format!(
                        "{}: {}",
                        error.kind(),
                        error.message()
                    )),
                    number: None,
                },
            },
        }
    };

    (success, failure)
}

fn run_source(runtime: &mut Runtime, source: &str) -> TestValue {
    let expected = source.lines().map(str::trim).find_map(|line| {
        line.strip_prefix("# return:")
            .map(|rest| rest.trim().to_string())
    });
    let source = source
        .lines()
        .filter(|line| !line.trim().starts_with("# return:"))
        .collect::<Vec<_>>()
        .join("\n");
    let case = Case {
        title: "inline statement".to_string(),
        source,
        expected,
    };

    let value = check_with(runtime, &case).unwrap_or_else(|reason| panic!("{reason}"));
    test_value(runtime, value)
}

fn test_value(runtime: &Runtime, value: nucleoid::Value) -> TestValue {
    let json = runtime
        .serialize_json(&value)
        .unwrap_or_else(|error| panic!("{error}"));

    TestValue {
        json: serde_json::from_str(&json).expect("the runtime serializes valid JSON"),
        number: match value {
            nucleoid::Value::Number(number) => Some(number),
            _ => None,
        },
    }
}

/// Runs the case with this title, for the generated per-behaviour tests.
///
/// The case is looked up by title rather than by position, so that `build.rs`
/// and [`cases`] disagreeing about where one case ends and the next begins
/// fails loudly here instead of quietly running the wrong source.
pub fn run_case(documents: &[Vec<Case>], document: usize, title: &str) {
    let cases = documents
        .get(document)
        .unwrap_or_else(|| panic!("no document {document}"));

    let mut matching = cases.iter().filter(|case| case.title == title);

    let case = matching
        .next()
        .unwrap_or_else(|| panic!("no case titled {title:?}"));

    // `build.rs` refuses a document with repeated titles, so reaching here
    // means the two disagree about where cases begin — in which case running
    // the first match would silently test the wrong source.
    assert!(
        matching.next().is_none(),
        "more than one case titled {title:?}"
    );

    if let Err(reason) = check(case) {
        panic!(
            "{title}
  {reason}"
        );
    }
}

/// How many cases the harness found, to compare with how many tests were
/// generated for them.
pub fn total(documents: &[Vec<Case>]) -> usize {
    documents.iter().map(Vec::len).sum()
}
