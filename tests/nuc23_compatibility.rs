//! Read-only NUC23 compatibility checks using real pinned sources and sealed
//! clamp emulations. No candidate token counts, acceptance gate or reports.

mod common;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use indexmap::IndexMap;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{Expr, Stmt, parse};
use nucleoid::{ObjectId, Runtime, Value};
use serde::{Deserialize, Deserializer};

const ACCEPTED_REVISION: &str = "fe25c9b4036cd94e120cf3f6c8b56c606ece4328";
const DOCUMENTS: [(Document, usize); 4] = [
    (Document::Spec, 298),
    (Document::Reference, 60),
    (Document::Readme, 11),
    (Document::Examples, 66),
];
const ORIGINAL_DOCUMENTS: [(Document, usize); 4] = [
    (Document::Spec, 293),
    (Document::Reference, 59),
    (Document::Readme, 10),
    (Document::Examples, 65),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Document {
    Spec,
    Reference,
    Readme,
    Examples,
}

impl Document {
    fn path(self) -> &'static str {
        match self {
            Self::Spec => "nucleoid.spec.md",
            Self::Reference => "tests/reference.md",
            Self::Readme => "README.md",
            Self::Examples => "docs/examples.md",
        }
    }

    fn cases(self, source: &str) -> Vec<common::Case> {
        match self {
            Self::Spec | Self::Reference => common::cases(source),
            Self::Readme | Self::Examples => common::snippets(source, self.path()),
        }
    }

    fn source(self) -> &'static str {
        match self {
            Self::Spec => include_str!("../nucleoid.spec.md"),
            Self::Reference => include_str!("reference.md"),
            Self::Readme => include_str!("../README.md"),
            Self::Examples => include_str!("../docs/examples.md"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
enum Metric {
    #[serde(rename = "nucleoid-lexer-with-layout-v1")]
    LexerWithLayout,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseId {
    document: Document,
    title: String,
}

// Null means no explicit return; a missing field is not a sealed input.
fn required_expected<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtectedCase {
    id: CaseId,
    tokens: usize,
    assertions: Vec<String>,
    #[serde(deserialize_with = "required_expected")]
    expected: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Baseline {
    metric: Metric,
    revision: String,
    cases: Vec<ProtectedCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Extra {
    document: Document,
    title: String,
    source: String,
    #[serde(deserialize_with = "required_expected")]
    expected: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum CallForm {
    Unit(&'static str),
    Omitted,
    Extra {
        value: &'static str,
        extra: &'static str,
    },
}

struct Rhs {
    target: &'static str,
    form: CallForm,
}

impl Rhs {
    const fn unit(target: &'static str, value: &'static str) -> Self {
        Self {
            target,
            form: CallForm::Unit(value),
        }
    }

    fn sources(&self) -> (String, String) {
        let (before, after) = match self.form {
            CallForm::Unit(value) => (format!("{value}, 0, 1"), value.to_string()),
            CallForm::Omitted => (String::new(), String::new()),
            CallForm::Extra { value, extra } => (
                format!("{value}, 0, 1, {extra}"),
                format!("{value}, {extra}"),
            ),
        };
        (
            format!("{} = Math.clamp({before})", self.target),
            format!("{} = Math.saturate({after})", self.target),
        )
    }
}

#[derive(Debug, Clone, Copy)]
enum NamedNumeric {
    Variable(&'static str),
    Property(&'static str, &'static str),
}

impl NamedNumeric {
    fn key(self) -> NodeKey {
        match self {
            Self::Variable(name) => NodeKey::variable(name),
            Self::Property(object, property) => {
                NodeKey::property(&ObjectId::named(object), property)
            }
        }
    }

    fn value(self, runtime: &Runtime) -> &Value {
        match self {
            Self::Variable(name) => runtime.state.variable(name),
            Self::Property(object, property) => {
                runtime.state.property(&ObjectId::named(object), property)
            }
        }
        .unwrap_or_else(|| panic!("missing explicit named output {self:?}"))
    }
}

struct Registration {
    document: Document,
    title: &'static str,
    rewrites: &'static [Rhs],
    outputs: &'static [NamedNumeric],
    // Actual accepted CLI observations, not normative return expectations.
    native_stdout: &'static str,
}

impl Registration {
    fn id(&self) -> CaseId {
        CaseId {
            document: self.document,
            title: self.title.to_string(),
        }
    }
}

// Exact sealed identities/order, RHS spellings, retained arguments and finite
// native observations. Independent min/max assertions are never substitutions.
const ADDITIONS: [Registration; 8] = [
    Registration {
        document: Document::Spec,
        title: "Nucleoid keeps unit-interval saturation reactive to its input",
        rewrites: &[Rhs::unit("bounded", "reading")],
        outputs: &[NamedNumeric::Variable("bounded")],
        native_stdout: "-1.0\n",
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid preserves numeric coercion while saturating values",
        rewrites: &[
            Rhs::unit("from_text", "text"),
            Rhs::unit("empty", "\"\""),
            Rhs::unit("from_false", "false"),
            Rhs::unit("from_true", "true"),
            Rhs::unit("from_empty_list", "[]"),
            Rhs::unit("from_list", "[0.625]"),
            Rhs::unit("bad_list", "[0, 1]"),
            Rhs::unit("bad_text", "\"not a number\""),
            Rhs::unit("literal_null", "null"),
            Rhs::unit("from_date", "Date(1)"),
        ],
        outputs: &[
            NamedNumeric::Variable("from_text"),
            NamedNumeric::Variable("empty"),
            NamedNumeric::Variable("from_false"),
            NamedNumeric::Variable("from_true"),
            NamedNumeric::Variable("from_empty_list"),
            NamedNumeric::Variable("from_list"),
            NamedNumeric::Variable("bad_list"),
            NamedNumeric::Variable("bad_text"),
            NamedNumeric::Variable("literal_null"),
            NamedNumeric::Variable("from_date"),
        ],
        native_stdout: "1.0\n",
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid preserves special values and omitted unit saturation inputs",
        rewrites: &[
            Rhs::unit("positive", "1 / 0"),
            Rhs::unit("negative", "-1 / 0"),
            Rhs::unit("invalid", "0 / 0"),
            Rhs::unit("positive_zero", "0"),
            Rhs::unit("negative_zero", "-0"),
            Rhs {
                target: "omitted",
                form: CallForm::Omitted,
            },
            Rhs {
                target: "extra",
                form: CallForm::Extra {
                    value: "0.5",
                    extra: "99",
                },
            },
            Rhs::unit("tiny", "Number.MIN_VALUE"),
        ],
        outputs: &[
            NamedNumeric::Variable("positive"),
            NamedNumeric::Variable("negative"),
            NamedNumeric::Variable("invalid"),
            NamedNumeric::Variable("positive_zero"),
            NamedNumeric::Variable("negative_zero"),
            NamedNumeric::Variable("omitted"),
            NamedNumeric::Variable("extra"),
            NamedNumeric::Variable("tiny"),
        ],
        native_stdout: "2.2250738585072014e-308\n",
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid applies unit-interval saturation to live class rules",
        rewrites: &[Rhs::unit("$Level.bounded", "$Level.reading")],
        outputs: &[
            NamedNumeric::Property("first", "bounded"),
            NamedNumeric::Property("second", "bounded"),
        ],
        native_stdout: "2.0\n",
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid preserves deferred reads and rollback for unit saturation",
        rewrites: &[
            Rhs::unit("bounded", "sensor.reading"),
            Rhs::unit("    bounded", "missing"),
        ],
        outputs: &[NamedNumeric::Variable("bounded")],
        native_stdout: "2.0\n",
    },
    Registration {
        document: Document::Reference,
        title: "Unit-interval saturation keeps live fractions bounded",
        rewrites: &[Rhs::unit("fraction", "reading")],
        outputs: &[NamedNumeric::Variable("fraction")],
        native_stdout: "1.4\n",
    },
    Registration {
        document: Document::Readme,
        title: "Unit-interval saturation normalizes live progress",
        rewrites: &[Rhs::unit("progress", "ratio")],
        outputs: &[NamedNumeric::Variable("progress")],
        native_stdout: "-0.5\n",
    },
    Registration {
        document: Document::Examples,
        title: "Unit-interval saturation bounds live animation opacity",
        rewrites: &[Rhs::unit("$Fade.opacity", "$Fade.level")],
        outputs: &[NamedNumeric::Property("fade", "opacity")],
        native_stdout: "-1.0\n",
    },
];

fn git(root: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("--no-pager")
        .args(arguments)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn collect_cases(
    destination: &mut IndexMap<CaseId, common::Case>,
    document: Document,
    source: &str,
) {
    for case in document.cases(source) {
        let id = CaseId {
            document,
            title: case.title.clone(),
        };
        assert!(
            destination.insert(id, case).is_none(),
            "duplicate case identity"
        );
    }
}

fn math_arguments<'a>(expression: &'a Expr, method: &str) -> &'a [Expr] {
    let Expr::Call { callee, arguments } = expression else {
        panic!("expected static Math call: {expression}");
    };
    let Expr::Member { object, property } = callee.as_ref() else {
        panic!("expected Math member: {callee}");
    };
    assert_eq!(object.as_ref(), &Expr::Identifier("Math".to_string()));
    assert_eq!(property, method);
    arguments
}

fn validate_rhs_mapping(rhs: &Rhs) {
    let (before, after) = rhs.sources();
    let before = parse(before.trim_start()).unwrap();
    let after = parse(after.trim_start()).unwrap();
    let [
        Stmt::Assign {
            target: before_target,
            value: before_value,
        },
    ] = before.as_slice()
    else {
        panic!("accepted RHS must be one assignment");
    };
    let [
        Stmt::Assign {
            target: after_target,
            value: after_value,
        },
    ] = after.as_slice()
    else {
        panic!("candidate RHS must be one assignment");
    };
    assert_eq!(before_target, after_target);
    let accepted = math_arguments(before_value, "clamp");
    let candidate = math_arguments(after_value, "saturate");
    match rhs.form {
        CallForm::Omitted => {
            assert!(accepted.is_empty());
            assert!(candidate.is_empty());
        }
        CallForm::Unit(_) | CallForm::Extra { .. } => {
            let extra = usize::from(matches!(rhs.form, CallForm::Extra { .. }));
            assert_eq!(accepted.len(), 3 + extra);
            assert_eq!(candidate.len(), 1 + extra);
            assert_eq!(accepted[1], Expr::Number(0.0));
            assert_eq!(accepted[2], Expr::Number(1.0));
            assert_eq!(candidate[0], accepted[0], "first expression changed");
            if extra == 1 {
                assert_eq!(candidate[1], accepted[3], "extra expression/order changed");
            }
        }
    }
}

fn audit_rhs_changes(registration: &Registration, legacy: &str, candidate: &str) -> usize {
    let rewrites: Vec<_> = registration.rewrites.iter().map(Rhs::sources).collect();
    let mut seen = vec![0; rewrites.len()];
    let rewritten = legacy
        .trim()
        .lines()
        .map(|line| {
            for (index, (before, after)) in rewrites.iter().enumerate() {
                if line == before {
                    seen[index] += 1;
                    return after.as_str();
                }
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        seen.iter().all(|count| *count == 1),
        "every registered RHS must occur exactly once: {seen:?}"
    );
    assert_eq!(
        rewritten,
        candidate.trim(),
        "{}: only registered RHS API substitutions may change",
        registration.title
    );
    rewrites.len()
}

/// Validate the exact shipped historical prefix without Git, while checking
/// uniqueness across the whole input, including later appended identities.
fn snapshot_ids(document: Document, source: &str) -> Vec<CaseId> {
    let (_, count) = DOCUMENTS
        .iter()
        .find(|(registered, _)| *registered == document)
        .expect("document has declared snapshot metadata");
    let mut ids: Vec<_> = document
        .cases(source)
        .into_iter()
        .map(|case| CaseId {
            document,
            title: case.title,
        })
        .collect();
    let prefix = ids.get(..*count).unwrap_or_else(|| {
        panic!("{document:?}: historical source is too short for {count} cases")
    });
    assert_eq!(
        ids.iter().collect::<HashSet<_>>().len(),
        ids.len(),
        "duplicate historical identity"
    );
    let additions: Vec<_> = ADDITIONS
        .iter()
        .filter(|registration| registration.document == document)
        .map(Registration::id)
        .collect();
    let tail = prefix
        .len()
        .checked_sub(additions.len())
        .expect("registered additions fit the historical prefix");
    assert_eq!(
        &prefix[tail..],
        additions,
        "registered historical tail identity/order changed"
    );
    ids.truncate(*count);
    ids
}

/// Metadata-only negative controls, never substitute sources for the Git audit.
fn metadata_source(document: Document, ids: &[CaseId]) -> String {
    assert!(ids.iter().all(|id| id.document == document));
    match document {
        Document::Spec | Document::Reference => {
            let cases = ids
                .iter()
                .map(|id| format!("# {}\n\n42", id.title))
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");
            format!("```\n{cases}\n```\n")
        }
        Document::Readme | Document::Examples => ids
            .iter()
            .map(|id| format!("**{}**\n\n```nuc\n42\n```\n", id.title))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn assert_metadata_rejected(document: Document, ids: &[CaseId], reason: &str) {
    let source = metadata_source(document, ids);
    let failure = std::panic::catch_unwind(|| snapshot_ids(document, &source))
        .expect_err("invalid historical metadata must fail");
    let message = failure
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| failure.downcast_ref::<&str>().copied())
        .expect("metadata assertion has a textual failure");
    assert!(message.contains(reason), "{message}");
}

struct Execution {
    runtime: Runtime,
    value: Value,
}

fn checked_execution(case: &common::Case, protected: usize) -> Execution {
    let mut runtime = Runtime::new();
    let value = common::check_with_count(&mut runtime, case, protected)
        .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
    assert_eq!(runtime.assertions_run(), protected, "{}", case.title);
    Execution { runtime, value }
}

fn assert_numeric_bits(before: &Value, after: &Value) {
    match (before, after) {
        (Value::Number(before), Value::Number(after)) => {
            assert_eq!(before.to_bits(), after.to_bits());
        }
        (Value::Null, Value::Null) | (Value::Undefined, Value::Undefined) => {}
        other => panic!("unexpected numeric/null result pair: {other:?}"),
    }
}

fn compare_emulation_results(
    registration: &Registration,
    legacy: &Execution,
    candidate: &Execution,
) {
    // These eight finite last-statement returns are not forced to null by assert.
    assert_numeric_bits(&legacy.value, &candidate.value);
    assert_eq!(legacy.runtime.graph.len(), candidate.runtime.graph.len());
    for &output in registration.outputs {
        assert_numeric_bits(
            output.value(&legacy.runtime),
            output.value(&candidate.runtime),
        );
        let key = output.key();
        let before = legacy.runtime.graph.retrieve(&key).unwrap();
        let after = candidate.runtime.graph.retrieve(&key).unwrap();
        assert_eq!(before.kind, after.kind, "{key}");
        assert_eq!(before.dependencies, after.dependencies, "{key}");
        assert_eq!(before.dependents, after.dependents, "{key}");
        // Runnable/source ASTs intentionally differ; no anonymous IDs compared.
    }
}

fn native_case(case: &common::Case, stdout: &str, index: usize, form: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!(
            "nuc23-compatibility-{}-{index}-{form}.nuc",
            std::process::id()
        ));
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap()
        .write_all(case.source.as_bytes())
        .unwrap();
    assert_eq!(fs::read(&path).unwrap(), case.source.as_bytes());
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
        .arg(&path)
        .stdin(Stdio::null())
        .output();
    let cleanup = fs::remove_file(&path);
    let output = output.unwrap();
    cleanup.unwrap();
    assert!(output.status.success(), "{}: {output:?}", case.title);
    assert_eq!(output.stdout, stdout.as_bytes(), "{}", case.title);
    assert!(output.stderr.is_empty(), "{}: {output:?}", case.title);
    // Exact coverage is proved by checked_execution, not native exit status.
}

#[test]
fn document_metadata_retains_closed_wire_names_paths_snapshot_counts_and_order() {
    for ((document, count), (wire_name, path, expected_count)) in DOCUMENTS.into_iter().zip([
        ("spec", "nucleoid.spec.md", 298),
        ("reference", "tests/reference.md", 60),
        ("readme", "README.md", 11),
        ("examples", "docs/examples.md", 66),
    ]) {
        assert_eq!(document.path(), path);
        assert_eq!(count, expected_count);
        assert_eq!(snapshot_ids(document, document.source()).len(), count);
        let id: CaseId = serde_json::from_value(serde_json::json!({
            "document": wire_name, "title": "protected title",
        }))
        .unwrap();
        assert_eq!(
            id,
            CaseId {
                document,
                title: "protected title".to_string()
            }
        );
        let extra: Extra = serde_json::from_value(serde_json::json!({
            "document": wire_name, "title": "protected title", "source": "42", "expected": "42",
        }))
        .unwrap();
        assert_eq!(extra.document, document);
        assert_eq!(extra.title, "protected title");
        assert_eq!(extra.source, "42");
        assert_eq!(extra.expected.as_deref(), Some("42"));
    }
}

#[test]
fn historical_metadata_is_independent_of_forward_corpus_additions() {
    for (document, count) in DOCUMENTS {
        let current = document.source();
        let snapshot = snapshot_ids(document, current);
        let extended = match document {
            Document::Spec | Document::Reference => {
                let (body, closing) = current.rsplit_once("```").unwrap();
                format!("{body}\n---\n\n# Future corpus addition\n\n42\n```{closing}")
            }
            Document::Readme | Document::Examples => {
                format!("{current}\n**Future corpus addition**\n\n```nuc\n42\n```\n")
            }
        };
        assert_eq!(
            document.cases(&extended).len(),
            document.cases(current).len() + 1
        );
        assert_eq!(snapshot_ids(document, &extended), snapshot);
        assert_eq!(snapshot.len(), count);
    }
}

#[test]
fn historical_metadata_rejects_short_duplicate_misordered_and_renamed_registrations() {
    for (document, count) in DOCUMENTS {
        let snapshot = snapshot_ids(document, document.source());
        assert_eq!(
            snapshot_ids(document, &metadata_source(document, &snapshot)),
            snapshot
        );
        assert_metadata_rejected(
            document,
            &snapshot[..count - 1],
            "historical source is too short",
        );
        let mut duplicate = snapshot.clone();
        duplicate[1] = duplicate[0].clone();
        assert_metadata_rejected(document, &duplicate, "duplicate historical identity");
        let mut appended_duplicate = snapshot.clone();
        appended_duplicate.push(snapshot[0].clone());
        assert_metadata_rejected(
            document,
            &appended_duplicate,
            "duplicate historical identity",
        );
        let mut misordered = snapshot.clone();
        misordered.swap(count - 2, count - 1);
        assert_metadata_rejected(
            document,
            &misordered,
            "registered historical tail identity/order changed",
        );
        let mut renamed = snapshot;
        renamed[count - 1].title.push_str(" renamed");
        assert_metadata_rejected(
            document,
            &renamed,
            "registered historical tail identity/order changed",
        );
    }
}

#[test]
fn ordinary_metadata_is_hermetic_without_git_history() {
    let executable = env::current_exe().unwrap();
    let unavailable =
        executable.with_file_name(format!("nuc23-metadata-no-git-{}", std::process::id()));
    assert!(!unavailable.exists());
    let output = Command::new(&executable)
        .args([
            "document_metadata_retains_closed_wire_names_paths_snapshot_counts_and_order",
            "--exact",
            "--nocapture",
        ])
        .current_dir(executable.parent().unwrap())
        .env("PATH", "")
        .env("GIT_DIR", &unavailable)
        .env("GIT_OBJECT_DIRECTORY", unavailable.join("objects"))
        .env("GIT_COMMON_DIR", &unavailable)
        .env("GIT_ALTERNATE_OBJECT_DIRECTORIES", "")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "no-Git ordinary metadata subprocess failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
}

#[test]
fn unknown_wire_variants_fields_and_missing_sealed_inputs_fail_explicitly() {
    for document in ["unknown", "Spec", "docs/examples.md"] {
        let id = format!(r#"{{"document":"{document}","title":"case"}}"#);
        let extra =
            format!(r#"{{"document":"{document}","title":"case","source":"42","expected":null}}"#);
        for error in [
            serde_json::from_str::<CaseId>(&id).unwrap_err(),
            serde_json::from_str::<Extra>(&extra).unwrap_err(),
        ] {
            assert!(
                error
                    .to_string()
                    .contains(&format!("unknown variant `{document}`")),
                "{error}"
            );
        }
    }
    assert!(
        serde_json::from_str::<Metric>("\"estimated\"")
            .unwrap_err()
            .to_string()
            .contains("unknown variant `estimated`")
    );
    for error in [
        serde_json::from_str::<CaseId>(
            r#"{"document":"spec","title":"case","fallback":"spec"}"#,
        )
        .unwrap_err(),
        serde_json::from_str::<Extra>(
            r#"{"document":"spec","title":"case","source":"42","expected":null,"fallback":true}"#,
        )
        .unwrap_err(),
        serde_json::from_str::<Baseline>(
            r#"{"metric":"nucleoid-lexer-with-layout-v1","revision":"pinned","cases":[],"fallback":true}"#,
        )
        .unwrap_err(),
        serde_json::from_str::<ProtectedCase>(
            r#"{"id":{"document":"spec","title":"case"},"tokens":1,"assertions":[],"expected":null,"fallback":true}"#,
        )
        .unwrap_err(),
    ] {
        assert!(error.to_string().contains("unknown field `fallback`"), "{error}");
    }
    for (input, field) in [
        (
            r#"{"document":"spec","source":"42","expected":null}"#,
            "title",
        ),
        (
            r#"{"document":"spec","title":"case","expected":null}"#,
            "source",
        ),
        (
            r#"{"document":"spec","title":"case","source":"42"}"#,
            "expected",
        ),
    ] {
        let error = serde_json::from_str::<Extra>(input).unwrap_err();
        assert!(
            error
                .to_string()
                .contains(&format!("missing field `{field}`")),
            "{error}"
        );
    }
    let error = serde_json::from_str::<ProtectedCase>(
        r#"{"id":{"document":"spec","title":"case"},"tokens":1,"assertions":[]}"#,
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("missing field `expected`"),
        "{error}"
    );
    for (input, field) in [
        (r#"{"revision":"pinned","cases":[]}"#, "metric"),
        (
            r#"{"metric":"nucleoid-lexer-with-layout-v1","cases":[]}"#,
            "revision",
        ),
        (
            r#"{"metric":"nucleoid-lexer-with-layout-v1","revision":"pinned"}"#,
            "cases",
        ),
    ] {
        let error = serde_json::from_str::<Baseline>(input).unwrap_err();
        assert!(
            error
                .to_string()
                .contains(&format!("missing field `{field}`")),
            "{error}"
        );
    }
}

#[test]
fn registered_rhs_mappings_preserve_all_operands_extras_omissions_and_control_lines() {
    assert_eq!(
        ADDITIONS.each_ref().map(|case| case.rewrites.len()),
        [1, 10, 8, 1, 2, 1, 1, 1]
    );
    for registration in &ADDITIONS {
        for rhs in registration.rewrites {
            validate_rhs_mapping(rhs);
        }
        let legacy = registration
            .rewrites
            .iter()
            .map(|rhs| rhs.sources().0)
            .collect::<Vec<_>>()
            .join("\n");
        let candidate = registration
            .rewrites
            .iter()
            .map(|rhs| rhs.sources().1)
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            audit_rhs_changes(registration, &legacy, &candidate),
            registration.rewrites.len()
        );
    }
    let legacy = "# Math.clamp(reading, 0, 1) is prose\ntext = \"Math.clamp(reading, 0, 1)\"\nbounded = Math.clamp(reading, 0, 1)\nassert(1 / bounded, 1 / Math.min(Math.max(reading, 0), 1))";
    let candidate = "# Math.clamp(reading, 0, 1) is prose\ntext = \"Math.clamp(reading, 0, 1)\"\nbounded = Math.saturate(reading)\nassert(1 / bounded, 1 / Math.min(Math.max(reading, 0), 1))";
    assert_eq!(audit_rhs_changes(&ADDITIONS[0], legacy, candidate), 1);
}

#[test]
#[should_panic(expected = "only registered RHS API substitutions may change")]
fn rhs_audit_rejects_weakened_assertions_even_with_the_same_call_count() {
    audit_rhs_changes(
        &ADDITIONS[0],
        "bounded = Math.clamp(reading, 0, 1)\nassert(bounded, 0)",
        "bounded = Math.saturate(reading)\nassert(bounded, bounded)",
    );
}

#[test]
#[should_panic(expected = "only registered RHS API substitutions may change")]
fn rhs_audit_rejects_rewriting_an_independent_min_max_zero_control() {
    audit_rhs_changes(
        &ADDITIONS[0],
        "bounded = Math.clamp(reading, 0, 1)\nassert(1 / bounded, 1 / Math.min(Math.max(reading, 0), 1))",
        "bounded = Math.saturate(reading)\nassert(1 / bounded, 1 / Math.saturate(reading))",
    );
}

#[test]
fn rhs_audit_rejects_missing_or_repeated_registered_changes_and_wrong_extra_order() {
    let registration = &ADDITIONS[0];
    let (legacy, candidate) = registration.rewrites[0].sources();
    assert!(
        std::panic::catch_unwind(|| audit_rhs_changes(registration, "reading = 1", &candidate))
            .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| {
            audit_rhs_changes(
                registration,
                &format!("{legacy}\n{legacy}"),
                &format!("{candidate}\n{candidate}"),
            )
        })
        .is_err()
    );
    let registration = &ADDITIONS[2];
    let legacy = registration
        .rewrites
        .iter()
        .map(|rhs| rhs.sources().0)
        .collect::<Vec<_>>()
        .join("\n");
    let candidate = registration
        .rewrites
        .iter()
        .map(|rhs| rhs.sources().1)
        .collect::<Vec<_>>()
        .join("\n");
    for changed in [
        candidate.replace("Math.saturate(0.5, 99)", "Math.saturate(99, 0.5)"),
        candidate.replace("Math.saturate()", "Math.saturate(Number.NaN)"),
    ] {
        assert!(
            std::panic::catch_unwind(|| audit_rhs_changes(registration, &legacy, &changed))
                .is_err()
        );
    }
}

#[test]
fn consolidated_reference_keeps_the_exact_registered_saturation_example_in_sync() {
    let reference = include_str!("../docs/README.md");
    let (_, section) = reference
        .split_once("### Unit-interval saturation keeps live fractions bounded\n\n```\n")
        .expect("consolidated reference contains the registered example");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == ADDITIONS[5].title)
        .expect("registered reference case");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 23](nuc-0023.md)"));
}

#[test]
#[ignore = "requires caller-sealed NUC23_COMPAT_BASELINE and NUC23_COMPAT_EXTRAS"]
fn pinned_corpus_preserves_legacy_syntax_and_registered_unit_clamp_emulations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let baseline: Baseline = serde_json::from_str(
        &fs::read_to_string(env::var_os("NUC23_COMPAT_BASELINE").expect("sealed baseline path"))
            .unwrap(),
    )
    .unwrap();
    let extras: Vec<Extra> = serde_json::from_str(
        &fs::read_to_string(env::var_os("NUC23_COMPAT_EXTRAS").expect("sealed extras path"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(baseline.metric, Metric::LexerWithLayout);
    assert_eq!(baseline.revision, ACCEPTED_REVISION);
    assert_eq!(git(root, &["rev-parse", "HEAD"]).trim(), ACCEPTED_REVISION);
    assert_eq!(extras.len(), ADDITIONS.len());
    let mut legacy = IndexMap::new();
    let mut candidate = IndexMap::new();
    for (document, count) in ORIGINAL_DOCUMENTS {
        let accepted_path = format!("{ACCEPTED_REVISION}:{}", document.path());
        let source = git(root, &["show", &accepted_path]);
        assert_eq!(document.cases(&source).len(), count);
        collect_cases(&mut legacy, document, &source);
        collect_cases(
            &mut candidate,
            document,
            &fs::read_to_string(root.join(document.path())).unwrap(),
        );
    }
    assert_eq!(
        legacy.len(),
        427,
        "all actual pinned originals are required"
    );
    let originals: HashSet<_> = legacy.keys().cloned().collect();
    for (extra, registration) in extras.into_iter().zip(&ADDITIONS) {
        let id = CaseId {
            document: extra.document,
            title: extra.title.clone(),
        };
        assert_eq!(
            id,
            registration.id(),
            "sealed addition identity/order changed"
        );
        assert!(
            legacy
                .insert(
                    id,
                    common::Case {
                        title: extra.title,
                        source: extra.source,
                        expected: extra.expected,
                    }
                )
                .is_none(),
            "duplicate extra identity"
        );
    }
    assert_eq!(baseline.cases.len(), 435);
    assert_eq!(legacy.len(), baseline.cases.len());
    assert_eq!(candidate.len(), baseline.cases.len());
    assert!(
        legacy.keys().eq(baseline.cases.iter().map(|case| &case.id)),
        "baseline order changed"
    );
    for (document, count) in DOCUMENTS {
        let expected: Vec<_> = baseline
            .cases
            .iter()
            .filter(|case| case.id.document == document)
            .map(|case| &case.id)
            .collect();
        assert_eq!(expected.len(), count);
        for sources in [&legacy, &candidate] {
            assert_eq!(
                sources
                    .keys()
                    .filter(|id| id.document == document)
                    .collect::<Vec<_>>(),
                expected,
                "{document:?}: identity/order changed"
            );
        }
    }
    let mut protected = 0;
    let mut expected_returns = 0;
    let mut executed = [0, 0];
    let mut rhs_changes = 0;
    let mut unchanged = 0;
    let mut native = 0;
    for case in &baseline.cases {
        assert!(case.tokens > 0, "sealed accepted counts must be present");
        let before = legacy.get(&case.id).unwrap();
        let after = candidate.get(&case.id).unwrap();
        let before_ast = parse(&before.source).unwrap();
        let after_ast = parse(&after.source).unwrap();
        let registration = if originals.contains(&case.id) {
            assert_eq!(
                before.source, after.source,
                "{:?}: original source changed",
                case.id
            );
            assert_eq!(before_ast, after_ast, "{:?}: legacy AST changed", case.id);
            assert_eq!(
                generate_all(&before_ast),
                generate_all(&after_ast),
                "{:?}: legacy canonical form changed",
                case.id
            );
            unchanged += 1;
            None
        } else {
            let registration = ADDITIONS
                .iter()
                .find(|registration| registration.id() == case.id)
                .expect("unregistered addition");
            for rhs in registration.rewrites {
                validate_rhs_mapping(rhs);
            }
            rhs_changes += audit_rhs_changes(registration, &before.source, &after.source);
            Some(registration)
        };
        for (source, statements) in [(before, &before_ast), (after, &after_ast)] {
            assert_eq!(
                source.expected, case.expected,
                "{:?}: expected return changed",
                case.id
            );
            assert_eq!(
                common::assertions::protected_assertions(statements),
                case.assertions,
                "{:?}: ordered canonical assertions changed",
                case.id
            );
        }
        let before_execution = checked_execution(before, case.assertions.len());
        let after_execution = checked_execution(after, case.assertions.len());
        let counts = [
            before_execution.runtime.assertions_run(),
            after_execution.runtime.assertions_run(),
        ];
        assert_eq!(
            counts[0], counts[1],
            "{:?}: executed coverage changed",
            case.id
        );
        for (total, count) in executed.iter_mut().zip(counts) {
            *total += count;
        }
        if let Some(registration) = registration {
            compare_emulation_results(registration, &before_execution, &after_execution);
            native_case(before, registration.native_stdout, native, "accepted");
            native_case(after, registration.native_stdout, native, "candidate");
            native += 1;
        }
        protected += case.assertions.len();
        expected_returns += usize::from(case.expected.is_some());
    }
    assert_eq!(unchanged, 427);
    assert_eq!(rhs_changes, 25);
    assert_eq!(protected, 1612);
    assert_eq!(expected_returns, 17);
    assert_eq!(executed, [1612, 1612]);
    assert_eq!(native, 8);
    println!(
        "435 pairs (427 untouched actual Git originals + 8 sealed clamp emulations) preserve per-document identities/order, {protected} ordered canonical assertions, {expected_returns} explicit expected returns and same-execution coverage {executed:?}. All 427 original sources/full ASTs/generator canonical forms are equal. Exactly {rhs_changes} registered RHS calls changed, retaining operand/extra order, prescribed omission, layouts/comments/assertions/literal types and independent min/max zero controls. Numeric result/output bits and explicit named node dependencies/dependents match for all eight additions. All 16 exact-byte ordinary native case executions match accepted observations; no anonymous global IDs, candidate tokens, acceptance evaluation or reports were compared/generated."
    );
}
