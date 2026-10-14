//! NUC24 development compatibility: real Git sources and sealed paired inputs.
//! No candidate corpus token measurements, proposal evaluation or reports.

mod common;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use indexmap::IndexMap;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Parameter, Stmt, parse};
use nucleoid::{ObjectId, Runtime, Value};
use serde::{Deserialize, Deserializer};

const ACCEPTED_REVISION: &str = "0eb374bed5cbe5bb37168cc1f523e61654b8bf61";
const DOCUMENTS: [(Document, usize, usize); 4] = [
    (Document::Spec, 298, 303),
    (Document::Reference, 60, 61),
    (Document::Readme, 11, 12),
    (Document::Examples, 66, 67),
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

    fn snapshot_count(self) -> usize {
        DOCUMENTS
            .iter()
            .find(|(document, _, _)| *document == self)
            .expect("closed document has registered counts")
            .2
    }

    // Protect the entire recent tail, not just this proposal's final identity.
    fn protected_tail(self) -> &'static [&'static str] {
        match self {
            Self::Spec => &[
                "Nucleoid keeps bare single parameter named arrows reactive",
                "Nucleoid preserves callable references for bare single parameter named arrows",
                "Nucleoid preserves bare single parameter methods and initializers",
                "Nucleoid keeps bare single parameter binding and explicit alternatives equivalent",
                "Nucleoid rolls back failed bare single parameter arrow redefinitions",
                "Nucleoid keeps bare single parameter block definitions reactive",
                "Nucleoid preserves callable references for bare single parameter block definitions",
                "Nucleoid preserves bare single parameter block methods and initializers",
                "Nucleoid keeps bare single parameter block binding and explicit alternatives equivalent",
                "Nucleoid rolls back failed bare single parameter block redefinitions",
                "Nucleoid keeps bare typed single parameter named arrows reactive",
                "Nucleoid preserves callable references for bare typed single parameter named arrows",
                "Nucleoid preserves bare typed single parameter methods and initializers",
                "Nucleoid keeps bare typed single parameter binding and explicit alternatives equivalent",
                "Nucleoid rolls back failed bare typed single parameter arrow redefinitions",
                "Nucleoid keeps numeric clamping reactive to all three inputs",
                "Nucleoid preserves numeric coercion while clamping values",
                "Nucleoid preserves nonfinite equal reversed and omitted clamp bounds",
                "Nucleoid applies numeric clamping to live class rules",
                "Nucleoid preserves deferred reads and rollback for numeric clamping",
                "Nucleoid keeps numeric interpolation reactive to all three inputs",
                "Nucleoid preserves numeric coercion while interpolating values",
                "Nucleoid preserves ordered special values and omitted interpolation inputs",
                "Nucleoid applies numeric interpolation to live class rules",
                "Nucleoid preserves deferred reads and rollback for numeric interpolation",
                "Nucleoid keeps unit-interval saturation reactive to its input",
                "Nucleoid preserves numeric coercion while saturating values",
                "Nucleoid preserves special values and omitted unit saturation inputs",
                "Nucleoid applies unit-interval saturation to live class rules",
                "Nucleoid preserves deferred reads and rollback for unit saturation",
                "Nucleoid keeps bare typed single parameter block definitions reactive",
                "Nucleoid preserves callable references for bare typed single parameter block definitions",
                "Nucleoid preserves bare typed single parameter block methods and initializers",
                "Nucleoid keeps bare typed single parameter block binding and explicit alternatives equivalent",
                "Nucleoid rolls back failed bare typed single parameter block redefinitions",
            ],
            Self::Reference => &[
                "Bare single parameter named arrows",
                "Bare single parameter indented block definitions",
                "Bare typed single parameter named arrows",
                "Numeric clamping keeps bounds live",
                "Numeric interpolation keeps endpoints and weights live",
                "Unit-interval saturation keeps live fractions bounded",
                "Bare typed single parameter indented block definitions",
            ],
            Self::Readme => &[
                "Single input named arrows omit parameter parentheses",
                "Single input block definitions omit parameter parentheses",
                "Typed single input named arrows omit parameter parentheses",
                "Numeric clamping keeps values within live bounds",
                "Numeric interpolation blends live values",
                "Unit-interval saturation normalizes live progress",
                "Typed block definitions preserve live bounded progress",
            ],
            Self::Examples => &[
                "Single parameter block definitions preserve live inputs",
                "Typed single parameter named arrows preserve live inputs",
                "Numeric clamping bounds live inventory levels",
                "Numeric interpolation keeps an animated position live",
                "Unit-interval saturation bounds live animation opacity",
                "Typed block definitions preserve both live return paths",
            ],
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

// A missing expected field is not interchangeable with explicit null.
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
    headers: &'static [&'static str],
    outputs: &'static [NamedNumeric],
    nodes: &'static [&'static str],
    // CLI observations, not invented normative return annotations.
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

const ADDITIONS: [Registration; 8] = [
    Registration {
        document: Document::Spec,
        title: "Nucleoid keeps bare typed single parameter block definitions reactive",
        headers: &["def scaled(amount: int):", "def scaled(amount: int):"],
        outputs: &[NamedNumeric::Variable("result")],
        nodes: &["base", "scaled", "result"],
        native_stdout: "null\n",
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid preserves callable references for bare typed single parameter block definitions",
        headers: &[
            "def change(amount: ArbitraryType):",
            "def change(amount: int):",
        ],
        outputs: &[
            NamedNumeric::Variable("first"),
            NamedNumeric::Variable("old"),
        ],
        nodes: &["base", "change", "live", "snapshot", "first", "old"],
        native_stdout: "4.0\n",
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid preserves bare typed single parameter block methods and initializers",
        headers: &[
            "    def init(amount: int):",
            "    def reading(offset: int):",
        ],
        outputs: &[NamedNumeric::Property("gauge", "amount")],
        nodes: &["$Gauge", "gauge", "gauge.amount"],
        native_stdout: "6.0\n",
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid keeps bare typed single parameter block binding and explicit alternatives equivalent",
        headers: &[
            "def identity(amount: ArbitraryType):",
            "def text(label: str):",
            "def ignored(amount: int):",
        ],
        outputs: &[NamedNumeric::Variable("amount")],
        nodes: &[
            "amount", "identity", "text", "ignored", "bare", "typed", "arrow", "pair", "unit",
        ],
        native_stdout: "null\n",
    },
    Registration {
        document: Document::Spec,
        title: "Nucleoid rolls back failed bare typed single parameter block redefinitions",
        headers: &["def reading(amount: int):", "    def reading(amount: int):"],
        outputs: &[NamedNumeric::Variable("total")],
        nodes: &["source", "reading", "total"],
        native_stdout: "4.0\n",
    },
    Registration {
        document: Document::Reference,
        title: "Bare typed single parameter indented block definitions",
        headers: &["def double(amount: int):"],
        outputs: &[],
        nodes: &["double"],
        native_stdout: "null\n",
    },
    Registration {
        document: Document::Readme,
        title: "Typed block definitions preserve live bounded progress",
        headers: &["def normalize(amount: ArbitraryType):"],
        outputs: &[NamedNumeric::Variable("progress")],
        nodes: &["ratio", "normalize", "progress"],
        native_stdout: "-0.5\n",
    },
    Registration {
        document: Document::Examples,
        title: "Typed block definitions preserve both live return paths",
        headers: &["def total(amount: int):"],
        outputs: &[NamedNumeric::Variable("invoice")],
        nodes: &["fee", "total", "invoice"],
        native_stdout: "3.0\n",
    },
];

fn ids(document: Document, source: &str) -> Vec<CaseId> {
    document
        .cases(source)
        .into_iter()
        .map(|case| CaseId {
            document,
            title: case.title,
        })
        .collect()
}

fn unique_ids(ids: &[CaseId]) {
    assert_eq!(
        ids.iter().collect::<HashSet<_>>().len(),
        ids.len(),
        "duplicate historical identity, including after the protected prefix"
    );
}

fn shipped_prefix(document: Document) -> Vec<CaseId> {
    let mut all = ids(document, document.source());
    unique_ids(&all);
    let prefix = all
        .get(..document.snapshot_count())
        .expect("shipped historical prefix is complete");
    let tail = document.protected_tail();
    let start = prefix.len().checked_sub(tail.len()).unwrap();
    assert_eq!(
        prefix[start..]
            .iter()
            .map(|id| id.title.as_str())
            .collect::<Vec<_>>(),
        tail,
        "protected full historical tail identity/order changed"
    );
    all.truncate(document.snapshot_count());
    all
}

/// Exact include_str! prefix; no Git, runtime history, baseline or status input.
fn snapshot_ids(document: Document, source: &str) -> Vec<CaseId> {
    let mut all = ids(document, source);
    unique_ids(&all);
    let prefix = all
        .get(..document.snapshot_count())
        .expect("historical source is too short");
    assert_eq!(
        prefix,
        shipped_prefix(document),
        "historical prefix identity/order changed"
    );
    all.truncate(document.snapshot_count());
    all
}

// Only metadata negative controls; never substitute these dummy sources in the audit.
fn metadata_source(document: Document, ids: &[CaseId]) -> String {
    assert!(ids.iter().all(|id| id.document == document));
    match document {
        Document::Spec | Document::Reference => {
            let body = ids
                .iter()
                .map(|id| format!("# {}\n\n42", id.title))
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");
            format!("```\n{body}\n```\n")
        }
        Document::Readme | Document::Examples => ids
            .iter()
            .map(|id| format!("**{}**\n\n```nuc\n42\n```\n", id.title))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn metadata_rejected(document: Document, ids: &[CaseId], reason: &str) {
    let source = metadata_source(document, ids);
    let failure = std::panic::catch_unwind(|| snapshot_ids(document, &source))
        .expect_err("invalid historical metadata must fail");
    let message = failure
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| failure.downcast_ref::<&str>().copied())
        .unwrap();
    assert!(message.contains(reason), "{message}");
}

/// Actual lexer positions identify headers, not matching text inside literals.
/// This inspects syntax, never counts corpus tokens.
fn eligible_headers(legacy: &str) -> IndexMap<usize, String> {
    let lines: Vec<_> = legacy.lines().collect();
    let tokens = tokenize(legacy).unwrap();
    let mut eligible = IndexMap::new();
    for (index, window) in tokens.windows(9).enumerate() {
        let [
            definition,
            name,
            left,
            parameter,
            annotation_colon,
            annotation,
            right,
            suite_colon,
            body,
        ] = window
        else {
            unreachable!("nine-token window");
        };
        if definition.token != Token::Keyword(Keyword::Def)
            || !matches!(name.token, Token::Identifier(_))
            || left.token != Token::LeftParen
            || !matches!(parameter.token, Token::Identifier(_))
            || annotation_colon.token != Token::Colon
            || !matches!(annotation.token, Token::Identifier(_))
            || right.token != Token::RightParen
            || suite_colon.token != Token::Colon
            || !matches!(body.token, Token::Newline | Token::Indent)
            || window[..8]
                .iter()
                .any(|token| token.position.line != definition.position.line)
            || !tokens
                .iter()
                .skip(index + 8)
                .find(|token| !matches!(token.token, Token::Newline | Token::Semicolon))
                .is_some_and(|token| token.token == Token::Indent)
        {
            continue;
        }
        let line = definition.position.line - 1;
        let mut spelling: Vec<char> = lines[line].chars().collect();
        assert_eq!(spelling[left.position.column - 1], '(');
        assert_eq!(spelling[right.position.column - 1], ')');
        spelling[left.position.column - 1] = ' ';
        spelling.remove(right.position.column - 1);
        assert!(
            eligible
                .insert(line, spelling.into_iter().collect())
                .is_none()
        );
    }
    eligible
}

fn audit_header_changes(registration: &Registration, legacy: &str, candidate: &str) -> usize {
    let legacy = legacy.trim();
    let candidate = candidate.trim();
    let lines: Vec<_> = legacy.lines().collect();
    let eligible = eligible_headers(legacy);
    let mut next = 0;
    let rewritten = lines
        .iter()
        .enumerate()
        .map(|(line, before)| {
            if registration.headers.get(next) == Some(before) {
                if let Some(after) = eligible.get(&line) {
                    next += 1;
                    return after.as_str();
                }
            }
            *before
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        next,
        registration.headers.len(),
        "every registered actual typed-block header must occur in exact order"
    );
    assert_eq!(
        rewritten, candidate,
        "{}: only registered typed-block delimiter pairs may change",
        registration.title
    );
    next
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

fn numeric_bits_match(before: &Value, after: &Value) {
    match (before, after) {
        (Value::Number(before), Value::Number(after)) => {
            assert_eq!(before.to_bits(), after.to_bits())
        }
        (Value::Null, Value::Null) | (Value::Undefined, Value::Undefined) => {}
        other => panic!("unexpected numeric/null result pair: {other:?}"),
    }
}

fn compare_named_results(registration: &Registration, before: &Execution, after: &Execution) {
    // Every non-assert normal result replaces the run result, even Undefined.
    // Top-level asserts do not replace it. Do not infer a generic last-value rule.
    numeric_bits_match(&before.value, &after.value);
    assert_eq!(before.runtime.graph.len(), after.runtime.graph.len());
    for &output in registration.outputs {
        numeric_bits_match(output.value(&before.runtime), output.value(&after.runtime));
        let key = output.key();
        let before = before.runtime.graph.retrieve(&key).unwrap();
        let after = after.runtime.graph.retrieve(&key).unwrap();
        assert_eq!(before.kind, after.kind, "{key}");
        assert_eq!(before.dependencies, after.dependencies, "{key}");
        assert_eq!(before.dependents, after.dependents, "{key}");
    }
    for name in registration.nodes {
        let key = NodeKey::variable(*name);
        let before = before.runtime.graph.retrieve(&key).unwrap();
        let after = after.runtime.graph.retrieve(&key).unwrap();
        assert_eq!(before.kind, after.kind, "{key}");
        assert_eq!(before.dependencies, after.dependencies, "{key}");
        assert_eq!(before.dependents, after.dependents, "{key}");
    }
    for execution in [before, after] {
        for annotation in ["ArbitraryType", "int", "str"] {
            assert!(
                !execution
                    .runtime
                    .graph
                    .contains(&NodeKey::variable(annotation))
            );
        }
    }
}

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

#[derive(Debug)]
struct NativeFile(PathBuf);

impl NativeFile {
    fn new(path: PathBuf, source: &str) -> io::Result<Self> {
        let writer = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        let file = Self(path);
        // Close the writer before guard cleanup, including on write errors.
        {
            let mut writer = writer;
            writer.write_all(source.as_bytes())?;
        }
        assert_eq!(fs::read(&file.0)?, source.as_bytes());
        Ok(file)
    }
}

impl Drop for NativeFile {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

fn native_case(directory: &Path, case: &common::Case, stdout: &str, index: usize, form: &str) {
    let file = NativeFile::new(
        directory.join(format!(
            "nuc24-development-{}-{index}-{form}.nuc",
            std::process::id()
        )),
        &case.source,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
        .arg(&file.0)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success(), "{}: {output:?}", case.title);
    assert_eq!(output.stdout, stdout.as_bytes(), "{}", case.title);
    assert!(output.stderr.is_empty(), "{}: {output:?}", case.title);
    // Actual assertion coverage comes from checked_execution, not CLI success.
}

#[test]
fn native_fixture_creation_preserves_existing_files_and_removes_only_owned_paths() {
    let path = env::temp_dir().join(format!(
        "nuc24-development-{}-ownership-control.nuc",
        std::process::id()
    ));
    let source = "assert(1, 1)";
    let file = NativeFile::new(path.clone(), source).unwrap();
    let error = NativeFile::new(path.clone(), "replacement").unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
    drop(file);
    assert_eq!(
        fs::metadata(&path).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn document_metadata_retains_closed_wire_names_paths_exact_shipped_prefixes_and_order() {
    for ((document, _, count), (wire_name, path, expected_count)) in DOCUMENTS.into_iter().zip([
        ("spec", "nucleoid.spec.md", 303),
        ("reference", "tests/reference.md", 61),
        ("readme", "README.md", 12),
        ("examples", "docs/examples.md", 67),
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
fn exact_historical_prefix_metadata_permits_unique_future_append_only_cases() {
    for (document, _, count) in DOCUMENTS {
        let source = document.source();
        let extended = match document {
            Document::Spec | Document::Reference => {
                let (body, closing) = source.rsplit_once("```").unwrap();
                format!("{body}\n---\n\n# Future corpus addition\n\n42\n```{closing}")
            }
            Document::Readme | Document::Examples => {
                format!("{source}\n**Future corpus addition**\n\n```nuc\n42\n```\n")
            }
        };
        assert_eq!(
            document.cases(&extended).len(),
            document.cases(source).len() + 1
        );
        assert_eq!(snapshot_ids(document, &extended), shipped_prefix(document));
        assert_eq!(snapshot_ids(document, &extended).len(), count);
    }
}

#[test]
fn full_input_metadata_rejects_short_duplicate_misordered_and_renamed_prefixes() {
    for (document, _, count) in DOCUMENTS {
        let snapshot = shipped_prefix(document);
        assert_eq!(
            snapshot_ids(document, &metadata_source(document, &snapshot)),
            snapshot
        );
        metadata_rejected(
            document,
            &snapshot[..count - 1],
            "historical source is too short",
        );
        let mut duplicate = snapshot.clone();
        duplicate[1] = duplicate[0].clone();
        metadata_rejected(document, &duplicate, "duplicate historical identity");
        let mut appended_duplicate = snapshot.clone();
        appended_duplicate.push(snapshot[0].clone());
        metadata_rejected(
            document,
            &appended_duplicate,
            "duplicate historical identity",
        );
        let mut future_duplicates = snapshot.clone();
        let future = CaseId {
            document,
            title: "Future duplicate".to_string(),
        };
        future_duplicates.extend([future.clone(), future]);
        metadata_rejected(
            document,
            &future_duplicates,
            "duplicate historical identity",
        );
        for pair in [(0, 1), (count - 2, count - 1)] {
            let mut misordered = snapshot.clone();
            misordered.swap(pair.0, pair.1);
            metadata_rejected(
                document,
                &misordered,
                "historical prefix identity/order changed",
            );
        }
        for index in [0, count - 1] {
            let mut renamed = snapshot.clone();
            renamed[index].title.push_str(" renamed");
            metadata_rejected(
                document,
                &renamed,
                "historical prefix identity/order changed",
            );
        }
    }
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
        serde_json::from_str::<CaseId>(r#"{"document":"spec","title":"case","fallback":true}"#).unwrap_err(),
        serde_json::from_str::<Extra>(r#"{"document":"spec","title":"case","source":"42","expected":null,"fallback":true}"#).unwrap_err(),
        serde_json::from_str::<ProtectedCase>(r#"{"id":{"document":"spec","title":"case"},"tokens":1,"assertions":[],"expected":null,"fallback":true}"#).unwrap_err(),
        serde_json::from_str::<Baseline>(r#"{"metric":"nucleoid-lexer-with-layout-v1","revision":"pinned","cases":[],"fallback":true}"#).unwrap_err(),
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
fn all_fourteen_registered_headers_preserve_complete_typed_metadata_and_delimiters_only() {
    assert_eq!(
        ADDITIONS.each_ref().map(|case| case.headers.len()),
        [2, 2, 2, 3, 2, 1, 1, 1]
    );
    for registration in &ADDITIONS {
        for header in registration.headers {
            let header = header.trim_start();
            let legacy = format!("{header}\n    pass");
            let eligible = eligible_headers(&legacy);
            let candidate = format!("{}\n    pass", eligible.get(&0).unwrap());
            let before = parse(&legacy).unwrap();
            let after = parse(&candidate).unwrap();
            assert_eq!(before, after);
            assert_eq!(generate_all(&before), generate_all(&after));
            let [Stmt::Function(function)] = before.as_slice() else {
                panic!("registered header must define one named function");
            };
            let [
                Parameter {
                    type_name: Some(_), ..
                },
            ] = function.parameters.as_slice()
            else {
                panic!("registered header must retain exactly one typed parameter");
            };
        }
    }
}

#[test]
fn registered_header_audit_rejects_body_assertion_annotation_layout_and_alternative_changes() {
    let registration = &ADDITIONS[0];
    let legacy = "def scaled(amount: int):\n    return amount + base\ndef scaled(amount: int):\n    return amount * base\nassert(scaled(3), 12)\ndef typed(amount: int):\n    return amount + 2";
    let candidate = legacy.replace("def scaled(amount: int):", "def scaled amount: int:");
    assert_eq!(audit_header_changes(registration, legacy, &candidate), 2);
    for changed in [
        candidate.replace("assert(scaled(3), 12)", "assert(scaled(3), scaled(3))"),
        candidate.replace("return amount + base", "return amount"),
        candidate.replace("def scaled amount: int:", "def scaled amount: str:"),
        candidate.replace("    return amount * base", "  return amount * base"),
        candidate.replace("def typed(amount: int):", "def typed amount: int:"),
        candidate.replacen("def scaled amount: int:", "def scaled(amount: int):", 1),
    ] {
        assert!(
            std::panic::catch_unwind(|| audit_header_changes(registration, legacy, &changed))
                .is_err()
        );
    }
    let missing = legacy.replacen("def scaled(amount: int):", "def other(amount: int):", 1);
    assert!(
        std::panic::catch_unwind(|| audit_header_changes(registration, &missing, &candidate))
            .is_err()
    );
}

#[test]
fn registered_header_audit_uses_real_tokens_not_comments_strings_or_template_text() {
    let registration = Registration {
        document: Document::Spec,
        title: "syntax control",
        headers: &["def f(amount: int):"],
        outputs: &[],
        nodes: &[],
        native_stdout: "null\n",
    };
    let legacy = "# def f(amount: int):\ntext = \"def f(amount: int):\"\ntemplate = `example\ndef f(amount: int):\n    return amount`\ndef f(amount: int):\n    return amount\ndef typed(amount: int):\n    return amount";
    let (unchanged, actual) = legacy.rsplit_once("\ndef f(amount: int):").unwrap();
    let candidate = format!("{unchanged}\ndef f amount: int:{actual}");
    assert_eq!(audit_header_changes(&registration, legacy, &candidate), 1);
    assert_eq!(parse(legacy).unwrap(), parse(&candidate).unwrap());
}

#[test]
fn protected_returns_and_assertion_coverage_use_the_same_normative_checked_execution() {
    let case = common::Case {
        title: "checked typed block".to_string(),
        source: "def double amount: int:\n    return amount * 2\nassert(double(3), 6)\ndouble(4)"
            .to_string(),
        expected: Some("8".to_string()),
    };
    let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
    let execution = checked_execution(&case, count);
    assert_eq!(execution.value, Value::Number(8.0));
    assert_eq!(execution.runtime.assertions_run(), 1);
    for (source, expected, assertions) in [
        ("assert(1, 1)\n2", "3", 1),
        ("if false:\n    assert(1, 2)\n3", "3", 1),
        ("assert(1, 2)\n3", "3", 1),
    ] {
        let case = common::Case {
            title: "invalid checked execution".to_string(),
            source: source.to_string(),
            expected: Some(expected.to_string()),
        };
        assert!(std::panic::catch_unwind(|| checked_execution(&case, assertions)).is_err());
    }
    checked_execution(
        &common::Case {
            title: "normative UUID matcher".to_string(),
            source: "class Item\nItem()".to_string(),
            expected: Some(r#"{"id":"[UUID]"}"#.to_string()),
        },
        0,
    );
    assert!(
        std::panic::catch_unwind(|| checked_execution(
            &common::Case {
                title: "UUID wildcard does not invent extra fields".to_string(),
                source: "class Item\nItem()".to_string(),
                expected: Some(r#"{"id":"[UUID]","type":"Item"}"#.to_string()),
            },
            0,
        ))
        .is_err()
    );
}

#[test]
fn consolidated_reference_keeps_the_exact_registered_typed_block_example_in_sync() {
    let reference = include_str!("../docs/README.md");
    let (_, section) = reference
        .split_once("### Bare typed single parameter indented block definitions\n\n```\n")
        .expect("consolidated reference contains the paired example");
    let (example, _) = section.split_once("```").unwrap();
    let case = Document::Reference
        .cases(Document::Reference.source())
        .into_iter()
        .find(|case| case.title == ADDITIONS[5].title)
        .unwrap();
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 24](nuc-0024.md)"));
}

fn without_history<'a>(command: &'a mut Command, unavailable: &Path) -> &'a mut Command {
    command
        .env("PATH", "")
        .env("GIT_DIR", unavailable)
        .env("GIT_OBJECT_DIRECTORY", unavailable.join("objects"))
        .env("GIT_COMMON_DIR", unavailable)
        .env("GIT_ALTERNATE_OBJECT_DIRECTORIES", "")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
}

#[test]
fn ordinary_complete_binary_is_hermetic_without_git_history() {
    const CHILD: &str = "NUC24_HERMETIC_CHILD";
    if env::var_os(CHILD).is_some() {
        assert_eq!(env::var_os("PATH").unwrap(), "");
        assert!(!Path::new(&env::var_os("GIT_DIR").unwrap()).exists());
        return; // Run every ordinary test, including this one, without recursion.
    }
    let executable = env::current_exe().unwrap();
    let unavailable = executable.with_file_name(format!("nuc24-no-history-{}", std::process::id()));
    assert!(!unavailable.exists());
    // Resolve an absolute executable for the negative control before emptying PATH.
    // No history is read to obtain the Git installation directory.
    let installation = git(Path::new(env!("CARGO_MANIFEST_DIR")), &["--exec-path"]);
    let absolute_git = ["git.exe", "git"]
        .into_iter()
        .map(|name| Path::new(installation.trim()).join(name))
        .find(|path| path.is_file())
        .expect("absolute Git executable for negative control")
        .canonicalize()
        .unwrap();
    let negative = without_history(&mut Command::new(absolute_git), &unavailable)
        .args(["--no-pager", "rev-parse", "--verify", "HEAD"])
        .current_dir(executable.parent().unwrap())
        .output()
        .unwrap();
    assert_eq!(negative.status.code(), Some(128), "{negative:?}");
    let output = without_history(&mut Command::new(&executable), &unavailable)
        .env(CHILD, "1")
        .current_dir(executable.parent().unwrap())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ordinary complete no-history binary failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("0 failed; 1 ignored"));
}

#[test]
#[ignore = "requires caller-sealed NUC24_COMPAT_BASELINE and NUC24_COMPAT_EXTRAS"]
fn pinned_corpus_preserves_legacy_syntax_and_registered_typed_block_headers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let baseline_path =
        PathBuf::from(env::var_os("NUC24_COMPAT_BASELINE").expect("sealed baseline path"));
    let extras_path =
        PathBuf::from(env::var_os("NUC24_COMPAT_EXTRAS").expect("sealed extras path"));
    let baseline: Baseline =
        serde_json::from_str(&fs::read_to_string(&baseline_path).unwrap()).unwrap();
    let extras: Vec<Extra> =
        serde_json::from_str(&fs::read_to_string(extras_path).unwrap()).unwrap();
    assert_eq!(baseline.metric, Metric::LexerWithLayout);
    assert_eq!(baseline.revision, ACCEPTED_REVISION);
    assert_eq!(git(root, &["rev-parse", "HEAD"]).trim(), ACCEPTED_REVISION);
    assert_eq!(extras.len(), ADDITIONS.len());
    let mut legacy = IndexMap::new();
    let mut candidate = IndexMap::new();
    for (document, original_count, _) in DOCUMENTS {
        let source = git(
            root,
            &["show", &format!("{ACCEPTED_REVISION}:{}", document.path())],
        );
        assert_eq!(document.cases(&source).len(), original_count);
        collect_cases(&mut legacy, document, &source);
        collect_cases(
            &mut candidate,
            document,
            &fs::read_to_string(root.join(document.path())).unwrap(),
        );
    }
    assert_eq!(
        legacy.len(),
        435,
        "all actual accepted Git originals are required"
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
    assert_eq!(baseline.cases.len(), 443);
    assert_eq!(legacy.len(), baseline.cases.len());
    assert_eq!(candidate.len(), baseline.cases.len());
    assert!(
        legacy.keys().eq(baseline.cases.iter().map(|case| &case.id)),
        "baseline order changed"
    );
    for (document, _, count) in DOCUMENTS {
        let expected: Vec<_> = baseline
            .cases
            .iter()
            .filter(|case| case.id.document == document)
            .map(|case| &case.id)
            .collect();
        assert_eq!(expected.len(), count);
        assert_eq!(
            snapshot_ids(document, document.source())
                .iter()
                .collect::<Vec<_>>(),
            expected
        );
        for sources in [&legacy, &candidate] {
            assert_eq!(
                sources
                    .keys()
                    .filter(|id| id.document == document)
                    .collect::<Vec<_>>(),
                expected,
                "{document:?}: per-document identity/order changed"
            );
        }
    }
    let mut protected = 0;
    let mut expected_returns = 0;
    let mut executed = [0, 0];
    let mut headers = 0;
    let mut unchanged = 0;
    let mut native = 0;
    for case in &baseline.cases {
        assert!(case.tokens > 0, "sealed accepted counts must be present");
        let before = legacy.get(&case.id).unwrap();
        let after = candidate.get(&case.id).unwrap();
        let before_ast = parse(&before.source).unwrap();
        let after_ast = parse(&after.source).unwrap();
        // All 443, not just originals, keep the complete tree and canonical form.
        assert_eq!(before_ast, after_ast, "{:?}: full AST changed", case.id);
        assert_eq!(
            generate_all(&before_ast),
            generate_all(&after_ast),
            "{:?}: canonical generator changed",
            case.id
        );
        let registration = if originals.contains(&case.id) {
            assert_eq!(
                before.source, after.source,
                "{:?}: original extracted bytes changed",
                case.id
            );
            unchanged += 1;
            None
        } else {
            let registration = ADDITIONS
                .iter()
                .find(|registration| registration.id() == case.id)
                .expect("unregistered addition");
            headers += audit_header_changes(registration, &before.source, &after.source);
            Some(registration)
        };
        for (source, statements) in [(before, &before_ast), (after, &after_ast)] {
            assert_eq!(
                source.expected, case.expected,
                "{:?}: explicit return changed",
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
            "{:?}: same-execution coverage changed",
            case.id
        );
        for (total, count) in executed.iter_mut().zip(counts) {
            *total += count;
        }
        if let Some(registration) = registration {
            compare_named_results(registration, &before_execution, &after_execution);
            let directory = baseline_path.parent().unwrap();
            native_case(
                directory,
                before,
                registration.native_stdout,
                native,
                "accepted",
            );
            native_case(
                directory,
                after,
                registration.native_stdout,
                native,
                "candidate",
            );
            native += 1;
        }
        protected += case.assertions.len();
        expected_returns += usize::from(case.expected.is_some());
    }
    assert_eq!(unchanged, 435);
    assert_eq!(headers, 14);
    assert_eq!(protected, 1652);
    assert_eq!(expected_returns, 17);
    assert_eq!(executed, [1652, 1652]);
    assert_eq!(native, 8);
    println!(
        "443 actual-source pairs (435 byte-unchanged Git originals + 8 sealed additions) preserve exact identities/per-document order 303/61/12/67, all full ASTs/canonical generators, {protected} ordered protected assertions, {expected_returns} explicit expected returns and same-execution coverage {executed:?}. Exactly {headers} registered typed-block delimiter pairs changed with every other line byte retained. Numeric output/result bits and bounded named graph dependencies/dependents match for all additions; all 16 exact-byte ordinary native runs match accepted CLI observations and their temporary files were removed. No anonymous raw IDs, candidate corpus token measurements, proposal evaluation or reports."
    );
}
