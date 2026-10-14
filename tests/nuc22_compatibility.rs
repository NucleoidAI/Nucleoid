//! Read-only NUC22 preparation checks: pinned syntax and sealed RHS emulations.
//! No candidate token measurements, acceptance decisions or reports.

mod common;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use indexmap::IndexMap;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{BinaryOp, Expr, Stmt, parse, parse_expression};
use nucleoid::{Runtime, Value};
use serde::Deserialize;

const ACCEPTED_REVISION: &str = "ed45e444f20ccad1a40134d37d83d04c2db34fdb";
const DOCUMENTS: [(Document, usize); 4] = [
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseId {
    document: Document,
    title: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtectedCase {
    id: CaseId,
    tokens: usize,
    assertions: Vec<String>,
    expected: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Baseline {
    metric: String,
    revision: String,
    cases: Vec<ProtectedCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Extra {
    document: Document,
    title: String,
    source: String,
    expected: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum CallForm {
    Three,
    MissingAll,
    MissingEnd,
    MissingWeight,
    Extra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Coercion {
    Number,
    Raw,
}

struct Rhs {
    target: &'static str,
    arguments: [&'static str; 3],
    form: CallForm,
    coercion: Coercion,
}

impl Rhs {
    const fn three(target: &'static str, arguments: [&'static str; 3]) -> Self {
        Self::with_form(target, arguments, CallForm::Three)
    }

    const fn raw(target: &'static str, arguments: [&'static str; 3]) -> Self {
        Self {
            target,
            arguments,
            form: CallForm::Three,
            coercion: Coercion::Raw,
        }
    }

    const fn with_form(target: &'static str, arguments: [&'static str; 3], form: CallForm) -> Self {
        Self {
            target,
            arguments,
            form,
            coercion: Coercion::Number,
        }
    }

    fn sources(&self) -> (String, String) {
        let [start, end, weight] = self.arguments;
        let before = match self.coercion {
            Coercion::Number => format!(
                "{} = Number({start}) + (Number({end}) - Number({start})) * Number({weight})",
                self.target
            ),
            Coercion::Raw => {
                format!("{} = {start} + ({end} - {start}) * {weight}", self.target)
            }
        };
        let arguments = match self.form {
            CallForm::Three => self.arguments.join(", "),
            CallForm::MissingAll => String::new(),
            CallForm::MissingEnd => start.to_string(),
            CallForm::MissingWeight => [start, end].join(", "),
            CallForm::Extra => format!("{start}, {end}, {weight}, 99"),
        };
        (before, format!("{} = Math.lerp({arguments})", self.target))
    }
}

struct Registration {
    document: Document,
    title: &'static str,
    rewrites: &'static [Rhs],
}

impl Registration {
    fn id(&self) -> CaseId {
        CaseId {
            document: self.document,
            title: self.title.to_string(),
        }
    }
}

// Exact sealed identities/order and complete argument spellings.
// Raw arithmetic is authorized only for numeric-only class/deferred programs:
// Number(undefined-property) would destroy their protected null deferral.
const ADDITIONS: [Registration; 8] = [
    Registration {
        document: Document::Spec,
        title: "Nucleoid keeps numeric interpolation reactive to all three inputs",
        rewrites: &[Rhs::three("position", ["start", "end", "weight"])],
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid preserves numeric coercion while interpolating values",
        rewrites: &[
            Rhs::three("numeric", ["\" 2 \"", "\"10\"", "\"0.25\""]),
            Rhs::three("boolean", ["false", "true", "true"]),
            Rhs::three("nil", ["null", "4", "0.5"]),
            Rhs::three("empty", ["[]", "4", "0.5"]),
            Rhs::three("singleton", ["[\"2\"]", "[\"10\"]", "[\"0.25\"]"]),
            Rhs::three("date", ["Date(0)", "Date(10)", "0.5"]),
            Rhs::three("invalid", ["\"invalid\"", "10", "0.5"]),
        ],
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid preserves ordered special values and omitted interpolation inputs",
        rewrites: &[
            Rhs::three("left", ["2", "10", "0"]),
            Rhs::three("right", ["2", "10", "1"]),
            Rhs::three("equal", ["4", "4", "2"]),
            Rhs::three("reversed", ["10", "2", "0.25"]),
            Rhs::three("extrapolated", ["2", "10", "2"]),
            Rhs::three("positive", ["0", "Number.POSITIVE_INFINITY", "0.5"]),
            Rhs::three("negative", ["0", "Number.NEGATIVE_INFINITY", "0.5"]),
            Rhs::three("zero_infinite", ["0", "Number.POSITIVE_INFINITY", "0"]),
            Rhs::three(
                "equal_infinite",
                [
                    "Number.POSITIVE_INFINITY",
                    "Number.POSITIVE_INFINITY",
                    "0.5",
                ],
            ),
            Rhs::three("nan_start", ["Number.NaN", "10", "0.5"]),
            Rhs::three("nan_end", ["2", "Number.NaN", "0.5"]),
            Rhs::three("nan_weight", ["2", "10", "Number.NaN"]),
            Rhs::with_form(
                "missing_all",
                ["Number.NaN", "Number.NaN", "Number.NaN"],
                CallForm::MissingAll,
            ),
            Rhs::with_form(
                "missing_end",
                ["2", "Number.NaN", "Number.NaN"],
                CallForm::MissingEnd,
            ),
            Rhs::with_form(
                "missing_weight",
                ["2", "10", "Number.NaN"],
                CallForm::MissingWeight,
            ),
            Rhs::with_form("extra", ["2", "10", "0.5"], CallForm::Extra),
        ],
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid applies numeric interpolation to live class rules",
        rewrites: &[Rhs::raw(
            "$Blend.position",
            ["$Blend.start", "$Blend.end", "$Blend.weight"],
        )],
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid preserves deferred reads and rollback for numeric interpolation",
        rewrites: &[
            Rhs::raw("position", ["sensor.start", "end", "weight"]),
            Rhs::raw("    position", ["missing", "end", "weight"]),
        ],
    },
    Registration {
        document: Document::Reference,
        title: "Numeric interpolation keeps endpoints and weights live",
        rewrites: &[Rhs::three("position", ["start", "end", "weight"])],
    },
    Registration {
        document: Document::Readme,
        title: "Numeric interpolation blends live values",
        rewrites: &[Rhs::three("mixed", ["start", "end", "weight"])],
    },
    Registration {
        document: Document::Examples,
        title: "Numeric interpolation keeps an animated position live",
        rewrites: &[Rhs::three(
            "coordinate",
            ["origin", "destination", "progress"],
        )],
    },
];

fn git(root: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("--no-pager")
        .args(arguments)
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

fn binary_operands(expression: &Expr, expected: BinaryOp) -> (&Expr, &Expr) {
    let Expr::Binary {
        operator,
        left,
        right,
    } = expression
    else {
        panic!("expected ordered binary arithmetic: {expression}");
    };
    assert_eq!(*operator, expected);
    (left, right)
}

fn numeric_argument(expression: &Expr, coercion: Coercion) -> &Expr {
    match coercion {
        Coercion::Raw => expression,
        Coercion::Number => {
            let Expr::Call { callee, arguments } = expression else {
                panic!("accepted emulation requires explicit Number coercion");
            };
            assert_eq!(callee.as_ref(), &Expr::Identifier("Number".to_string()));
            let [argument] = arguments.as_slice() else {
                panic!("accepted Number coercion has exactly one argument");
            };
            argument
        }
    }
}

fn lerp_arguments(expression: &Expr) -> &[Expr] {
    let Expr::Call { callee, arguments } = expression else {
        panic!("expected Math.lerp call: {expression}");
    };
    let Expr::Member { object, property } = callee.as_ref() else {
        panic!("expected static Math member: {callee}");
    };
    assert_eq!(object.as_ref(), &Expr::Identifier("Math".to_string()));
    assert_eq!(property, "lerp");
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
        panic!("accepted RHS must be a single assignment");
    };
    let [
        Stmt::Assign {
            target: after_target,
            value: after_value,
        },
    ] = after.as_slice()
    else {
        panic!("candidate RHS must be a single assignment");
    };
    assert_eq!(before_target, after_target);
    let (start, scaled) = binary_operands(before_value, BinaryOp::Add);
    let (delta, weight) = binary_operands(scaled, BinaryOp::Multiply);
    let (end, repeated_start) = binary_operands(delta, BinaryOp::Subtract);
    assert_eq!(start, repeated_start, "accepted start spelling changed");
    let accepted = [start, end, weight].map(|argument| numeric_argument(argument, rhs.coercion));
    let candidate = lerp_arguments(after_value);
    let nan = parse_expression("Number.NaN").unwrap();
    let retained = match rhs.form {
        CallForm::Three => 3,
        CallForm::MissingAll => {
            assert!(accepted.iter().all(|argument| **argument == nan));
            0
        }
        CallForm::MissingEnd => {
            assert_eq!(*accepted[1], nan);
            assert_eq!(*accepted[2], nan);
            1
        }
        CallForm::MissingWeight => {
            assert_eq!(*accepted[2], nan);
            2
        }
        CallForm::Extra => {
            assert_eq!(candidate.len(), 4);
            assert_eq!(candidate[3], Expr::Number(99.0));
            3
        }
    };
    if !matches!(rhs.form, CallForm::Extra) {
        assert_eq!(candidate.len(), retained);
    }
    assert_eq!(
        candidate[..retained].iter().collect::<Vec<_>>(),
        accepted[..retained],
        "retained argument expressions/order changed"
    );
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
        "every registered RHS must occur once: {seen:?}"
    );
    assert_eq!(
        rewritten,
        candidate.trim(),
        "{}: only registered RHS API substitutions may change",
        registration.title
    );
    rewrites.len()
}

/// Sealed additions append per document; ordinary metadata verifies the exact
/// historical prefix from shipped sources, independently of Git availability.
fn snapshot_ids(document: Document, source: &str) -> Vec<CaseId> {
    let (_, count) = DOCUMENTS
        .iter()
        .find(|(registered, _)| *registered == document)
        .expect("document has declared historical metadata");
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

/// Small metadata-only controls, not copied programs or measured cases.
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

fn compare_emulation_results(legacy: &Execution, candidate: &Execution) {
    // The eight sealed programs return numbers/null, not anonymous IDs.
    match (&legacy.value, &candidate.value) {
        (Value::Number(before), Value::Number(after)) => {
            assert_eq!(before.to_bits(), after.to_bits())
        }
        (Value::Null, Value::Null) | (Value::Undefined, Value::Undefined) => {}
        other => panic!("unexpected sealed emulation return pair: {other:?}"),
    }
    assert_eq!(legacy.runtime.graph.len(), candidate.runtime.graph.len());
    for node in legacy.runtime.graph.nodes() {
        assert!(
            !node.key.as_str().contains("obj:"),
            "unconstrained anonymous key is not comparable"
        );
        let other = candidate.runtime.graph.retrieve(&node.key).unwrap();
        assert_eq!(node.kind, other.kind);
        assert_eq!(node.dependencies, other.dependencies);
        assert_eq!(node.dependents, other.dependents);
        // Stored runnable/source ASTs intentionally differ at registered RHSs.
    }
}

#[test]
fn document_metadata_retains_closed_wire_names_paths_snapshot_counts_and_order() {
    for ((document, count), (wire_name, path, expected_count)) in DOCUMENTS.into_iter().zip([
        ("spec", "nucleoid.spec.md", 293),
        ("reference", "tests/reference.md", 59),
        ("readme", "README.md", 10),
        ("examples", "docs/examples.md", 65),
    ]) {
        assert_eq!(document.path(), path);
        assert_eq!(count, expected_count);
        let ids = snapshot_ids(document, document.source());
        assert_eq!(ids.len(), count);
        let additions: Vec<_> = ADDITIONS
            .iter()
            .filter(|registration| registration.document == document)
            .map(Registration::id)
            .collect();
        assert_eq!(&ids[ids.len() - additions.len()..], additions);
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
        assert!(
            snapshot
                .iter()
                .all(|id| id.title != "Future corpus addition")
        );
    }
}

#[test]
fn historical_metadata_rejects_short_duplicate_misordered_and_renamed_registrations() {
    for (document, count) in DOCUMENTS {
        let snapshot = snapshot_ids(document, document.source());
        assert_eq!(snapshot.len(), count);
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
        executable.with_file_name(format!("nuc22-metadata-no-git-{}", std::process::id()));
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
fn unknown_document_variants_fields_and_missing_required_inputs_fail_explicitly() {
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
    ] {
        let error = serde_json::from_str::<Extra>(input).unwrap_err();
        assert!(
            error
                .to_string()
                .contains(&format!("missing field `{field}`")),
            "{error}"
        );
    }
}

#[test]
fn registered_rhs_mappings_preserve_order_number_coercion_raw_exceptions_and_arity() {
    assert_eq!(
        ADDITIONS.each_ref().map(|case| case.rewrites.len()),
        [1, 7, 16, 1, 2, 1, 1, 1]
    );
    assert_eq!(
        ADDITIONS
            .iter()
            .flat_map(|case| case.rewrites)
            .filter(|rhs| rhs.coercion == Coercion::Raw)
            .count(),
        3
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
    let legacy = "# Number(start) is only prose\ntext = \"Number(start)\"\nposition = Number(start) + (Number(end) - Number(start)) * Number(weight)\nassert(position, 6)";
    let candidate = "# Number(start) is only prose\ntext = \"Number(start)\"\nposition = Math.lerp(start, end, weight)\nassert(position, 6)";
    assert_eq!(audit_rhs_changes(&ADDITIONS[0], legacy, candidate), 1);
}

#[test]
#[should_panic(expected = "only registered RHS API substitutions may change")]
fn rhs_audit_rejects_an_unregistered_assertion_body_change() {
    audit_rhs_changes(
        &ADDITIONS[0],
        "position = Number(start) + (Number(end) - Number(start)) * Number(weight)\nassert(position, 6)",
        "position = Math.lerp(start, end, weight)\nassert(position, position)",
    );
}

#[test]
fn consolidated_reference_keeps_the_registered_lerp_example_in_sync() {
    let reference = include_str!("../docs/README.md");
    let (_, section) = reference
        .split_once("### Numeric interpolation keeps endpoints and weights live\n\n```\n")
        .expect("consolidated builtin reference contains the registered example");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == ADDITIONS[5].title)
        .expect("registered reference case");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 22](nuc-0022.md)"));
}

#[test]
#[ignore = "requires caller-sealed NUC22_COMPAT_BASELINE and NUC22_COMPAT_EXTRAS"]
fn pinned_corpus_preserves_legacy_syntax_and_registered_interpolation_emulations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let baseline: Baseline = serde_json::from_str(
        &fs::read_to_string(env::var_os("NUC22_COMPAT_BASELINE").expect("sealed baseline path"))
            .unwrap(),
    )
    .unwrap();
    let extras: Vec<Extra> = serde_json::from_str(
        &fs::read_to_string(env::var_os("NUC22_COMPAT_EXTRAS").expect("sealed extras path"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(baseline.metric, "nucleoid-lexer-with-layout-v1");
    assert_eq!(baseline.revision, ACCEPTED_REVISION);
    assert_eq!(git(root, &["rev-parse", "HEAD"]).trim(), ACCEPTED_REVISION);
    assert_eq!(extras.len(), ADDITIONS.len());
    let mut legacy = IndexMap::new();
    let mut candidate = IndexMap::new();
    for (document, _) in DOCUMENTS {
        let path = document.path();
        let accepted_path = format!("{ACCEPTED_REVISION}:{path}");
        collect_cases(&mut legacy, document, &git(root, &["show", &accepted_path]));
        collect_cases(
            &mut candidate,
            document,
            &fs::read_to_string(root.join(path)).unwrap(),
        );
    }
    assert_eq!(legacy.len(), 419, "all pinned originals are required");
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
    assert_eq!(baseline.cases.len(), 427);
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
    for case in &baseline.cases {
        assert!(case.tokens > 0, "sealed accepted counts must be present");
        let before = legacy.get(&case.id).unwrap();
        let after = candidate.get(&case.id).unwrap();
        let before_ast = parse(&before.source).unwrap();
        let after_ast = parse(&after.source).unwrap();
        if originals.contains(&case.id) {
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
        } else {
            let registration = ADDITIONS
                .iter()
                .find(|registration| registration.id() == case.id)
                .expect("unregistered addition");
            for rhs in registration.rewrites {
                validate_rhs_mapping(rhs);
            }
            rhs_changes += audit_rhs_changes(registration, &before.source, &after.source);
        }
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
        if !originals.contains(&case.id) {
            compare_emulation_results(&before_execution, &after_execution);
        }
        protected += case.assertions.len();
        expected_returns += usize::from(case.expected.is_some());
    }
    assert_eq!(unchanged, 419);
    assert_eq!(rhs_changes, 30);
    assert_eq!(protected, 1566);
    assert_eq!(expected_returns, 17);
    assert_eq!(executed, [1566, 1566]);
    println!(
        "427 pairs (419 untouched pinned originals + 8 sealed emulations) preserve per-document identities/order, {protected} ordered canonical assertions, {expected_returns} explicit expected returns and executed coverage {executed:?}. All 419 original sources/ASTs/canonical forms are equal. Exactly {rhs_changes} registered RHS calls changed, with explicit Number coercion or the sealed numeric-only RAW exceptions, retained argument order, prescribed omissions/extra, layouts/comments/assertions/types, numeric outputs and named dependency edges protected. No candidate token measurements or acceptance evaluation ran."
    );
}
