//! Deterministic, per-case token gate for NUC proposal agents.
//!
//! The ignored entry points record an immutable baseline and compare a rewritten
//! corpus. Ordinary tests exercise the gate without environment variables.

mod common;

use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Command;

use indexmap::IndexMap;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, FunctionBody, Stmt, TemplatePart, parse};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const METRIC: &str = "nucleoid-lexer-with-layout-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Document {
    Spec,
    Reference,
    Readme,
    Examples,
}

impl Document {
    const ALL: [Self; 4] = [Self::Spec, Self::Reference, Self::Readme, Self::Examples];

    fn path(self) -> &'static str {
        match self {
            Self::Spec => "nucleoid.spec.md",
            Self::Reference => "tests/reference.md",
            Self::Readme => "README.md",
            Self::Examples => "docs/examples.md",
        }
    }

    fn cases(self, source: &str) -> Vec<SourceCase> {
        let cases = match self {
            Self::Spec | Self::Reference => common::cases(source),
            Self::Readme | Self::Examples => common::snippets(source, self.path()),
        };
        cases
            .into_iter()
            .map(|case| SourceCase {
                id: CaseId {
                    document: self,
                    title: case.title.clone(),
                },
                case,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseId {
    document: Document,
    title: String,
}

impl fmt::Display for CaseId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} :: {}", self.document.path(), self.title)
    }
}

struct SourceCase {
    id: CaseId,
    case: common::Case,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AdditionalBaseline {
    document: Document,
    title: String,
    source: String,
    expected: Option<String>,
}

impl From<AdditionalBaseline> for SourceCase {
    fn from(baseline: AdditionalBaseline) -> Self {
        Self {
            id: CaseId {
                document: baseline.document,
                title: baseline.title.clone(),
            },
            case: common::Case {
                title: baseline.title,
                source: baseline.source,
                expected: baseline.expected,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MeasuredCase {
    id: CaseId,
    tokens: usize,
    assertions: Vec<String>,
    expected: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Baseline {
    metric: String,
    revision: String,
    cases: Vec<MeasuredCase>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Decision {
    Accepted,
    Rejected,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TokenChange {
    Less,
    Same,
    More,
}

#[derive(Debug, Serialize)]
struct Comparison {
    id: CaseId,
    before: usize,
    after: usize,
    change: TokenChange,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Issue {
    MissingCase { id: CaseId },
    UnpairedCase { id: CaseId },
    ChangedAssertions { id: CaseId },
    ChangedReturn { id: CaseId },
    InvalidCase { id: CaseId, reason: String },
    MoreTokens { id: CaseId },
}

#[derive(Debug, Serialize)]
struct Proposal {
    path: String,
    revision: String,
}

#[derive(Debug, Serialize)]
struct Report {
    proposal: Proposal,
    metric: String,
    baseline_revision: String,
    decision: Decision,
    cases: Vec<Comparison>,
    issues: Vec<Issue>,
}

#[derive(Debug, Error)]
enum GateError {
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("git {arguments}: {reason}")]
    Git { arguments: String, reason: String },
    #[error("{id}: {reason}")]
    Case { id: CaseId, reason: String },
    #[error("invalid baseline: {0}")]
    Baseline(String),
    #[error("invalid proposal: {0}")]
    Proposal(String),
    #[error("set {0} before running the proposal gate")]
    MissingEnvironment(&'static str),
    #[error("{variable}: {source}")]
    Environment {
        variable: &'static str,
        source: env::VarError,
    },
    #[error("proposal evaluation is {decision:?}; see {report}")]
    Verdict { decision: Decision, report: PathBuf },
}

fn measure(source: &SourceCase) -> Result<MeasuredCase, GateError> {
    let code = source.case.source.trim();
    let tokens = tokenize(code).map_err(|error| GateError::Case {
        id: source.id.clone(),
        reason: error.to_string(),
    })?;
    let statements = parse(code).map_err(|error| GateError::Case {
        id: source.id.clone(),
        reason: error.to_string(),
    })?;
    let mut assertions = Vec::new();
    for statement in &statements {
        statement_assertions(statement, &mut assertions);
    }
    common::check_with_assertions(&source.case, assertions.len()).map_err(|reason| {
        GateError::Case {
            id: source.id.clone(),
            reason,
        }
    })?;

    Ok(MeasuredCase {
        id: source.id.clone(),
        tokens: tokens
            .iter()
            .filter(|spanned| spanned.token != Token::Eof)
            .count(),
        assertions,
        expected: source.case.expected.clone(),
    })
}

fn statement_assertions(statement: &Stmt, assertions: &mut Vec<String>) {
    match statement {
        Stmt::Assign { target, value } => {
            expression_assertions(target, assertions);
            expression_assertions(value, assertions);
        }
        Stmt::Expression(expression) | Stmt::Throw(expression) | Stmt::Delete(expression) => {
            expression_assertions(expression, assertions);
        }
        Stmt::If {
            condition,
            consequent,
            alternate,
        } => {
            expression_assertions(condition, assertions);
            for statement in consequent {
                statement_assertions(statement, assertions);
            }
            if let Some(statement) = alternate {
                statement_assertions(statement, assertions);
            }
        }
        Stmt::Block(body) => {
            for statement in body {
                statement_assertions(statement, assertions);
            }
        }
        Stmt::Class(class) => {
            for statement in &class.constructor {
                statement_assertions(statement, assertions);
            }
            for method in &class.methods {
                body_assertions(&method.body, assertions);
            }
        }
        Stmt::Function(function) => body_assertions(&function.body, assertions),
        Stmt::For { iterable, body, .. } => {
            expression_assertions(iterable, assertions);
            for statement in body {
                statement_assertions(statement, assertions);
            }
        }
        Stmt::Return(Some(expression)) => expression_assertions(expression, assertions),
        Stmt::Try { body, catch, .. } => {
            for statement in body.iter().chain(catch) {
                statement_assertions(statement, assertions);
            }
        }
        Stmt::Return(None) | Stmt::Declaration { .. } | Stmt::Pass => {}
    }
}

fn body_assertions(body: &FunctionBody, assertions: &mut Vec<String>) {
    match body {
        FunctionBody::Expression(expression) => expression_assertions(expression, assertions),
        FunctionBody::Block(statements) => {
            for statement in statements {
                statement_assertions(statement, assertions);
            }
        }
    }
}

fn expression_assertions(expression: &Expr, assertions: &mut Vec<String>) {
    match expression {
        Expr::Call { callee, arguments } => {
            if matches!(callee.as_ref(), Expr::Identifier(name) if name == "assert") {
                assertions.push(expression.to_string());
            }
            expression_assertions(callee, assertions);
            for argument in arguments {
                expression_assertions(argument, assertions);
            }
        }
        Expr::Member { object, .. } => expression_assertions(object, assertions),
        Expr::Version { reference, .. } => expression_assertions(reference, assertions),
        Expr::Index { object, index } => {
            expression_assertions(object, assertions);
            expression_assertions(index, assertions);
        }
        Expr::Slice { object, start, end } => {
            expression_assertions(object, assertions);
            for bound in start.iter().chain(end) {
                expression_assertions(bound, assertions);
            }
        }
        Expr::Unary { operand, .. } | Expr::Delete(operand) => {
            expression_assertions(operand, assertions);
        }
        Expr::Binary { left, right, .. }
        | Expr::Logical { left, right, .. }
        | Expr::Assign {
            target: left,
            value: right,
        } => {
            expression_assertions(left, assertions);
            expression_assertions(right, assertions);
        }
        Expr::Super(items) | Expr::List(items) => {
            for item in items {
                expression_assertions(item, assertions);
            }
        }
        Expr::ObjectLiteral(entries) => {
            for (_, value) in entries {
                expression_assertions(value, assertions);
            }
        }
        Expr::Function(function) => body_assertions(&function.body, assertions),
        Expr::Template(parts) => {
            for part in parts {
                if let TemplatePart::Expression(expression) = part {
                    expression_assertions(expression, assertions);
                }
            }
        }
        Expr::Reason { source, .. } => expression_assertions(source, assertions),
        Expr::Null
        | Expr::Bool(_)
        | Expr::Number(_)
        | Expr::String(_)
        | Expr::Regex(_)
        | Expr::Identifier(_)
        | Expr::ClassRef(_)
        | Expr::ObjectRef(_)
        | Expr::This
        | Expr::Model => {}
    }
}

fn capture(revision: String, sources: &[SourceCase]) -> Result<Baseline, GateError> {
    if sources.is_empty() {
        return Err(GateError::Baseline("the corpus is empty".to_string()));
    }
    let mut identifiers = HashSet::new();
    let mut cases = Vec::new();
    for source in sources {
        if !identifiers.insert(source.id.clone()) {
            return Err(GateError::Baseline(format!("duplicate case {}", source.id)));
        }
        cases.push(measure(source)?);
    }
    Ok(Baseline {
        metric: METRIC.to_string(),
        revision,
        cases,
    })
}

fn compare(
    baseline: &Baseline,
    proposal: Proposal,
    sources: &[SourceCase],
) -> Result<Report, GateError> {
    if baseline.metric != METRIC || baseline.revision.is_empty() || baseline.cases.is_empty() {
        return Err(GateError::Baseline(
            "expected a revision and a nonempty corpus using the current metric".to_string(),
        ));
    }
    let mut remaining = IndexMap::new();
    for case in &baseline.cases {
        if remaining.insert(case.id.clone(), case).is_some() {
            return Err(GateError::Baseline(format!("duplicate case {}", case.id)));
        }
    }
    let mut identifiers = HashSet::new();
    let mut cases = Vec::new();
    let mut issues = Vec::new();

    for source in sources {
        if !identifiers.insert(source.id.clone()) {
            return Err(GateError::Case {
                id: source.id.clone(),
                reason: "duplicate case".to_string(),
            });
        }
        let before = remaining.shift_remove(&source.id);
        let after = match measure(source) {
            Ok(case) => case,
            Err(error) => {
                issues.push(Issue::InvalidCase {
                    id: source.id.clone(),
                    reason: error.to_string(),
                });
                continue;
            }
        };
        let Some(before) = before else {
            issues.push(Issue::UnpairedCase {
                id: source.id.clone(),
            });
            continue;
        };
        if before.assertions != after.assertions {
            issues.push(Issue::ChangedAssertions {
                id: source.id.clone(),
            });
        }
        if before.expected != after.expected {
            issues.push(Issue::ChangedReturn {
                id: source.id.clone(),
            });
        }
        let change = match after.tokens.cmp(&before.tokens) {
            std::cmp::Ordering::Less => TokenChange::Less,
            std::cmp::Ordering::Equal => TokenChange::Same,
            std::cmp::Ordering::Greater => {
                issues.push(Issue::MoreTokens {
                    id: source.id.clone(),
                });
                TokenChange::More
            }
        };
        cases.push(Comparison {
            id: source.id.clone(),
            before: before.tokens,
            after: after.tokens,
            change,
        });
    }
    for (id, _) in remaining {
        issues.push(Issue::MissingCase { id });
    }
    let decision = if issues.is_empty() {
        Decision::Accepted
    } else if issues
        .iter()
        .all(|issue| matches!(issue, Issue::UnpairedCase { .. }))
    {
        Decision::Blocked
    } else {
        Decision::Rejected
    };
    Ok(Report {
        proposal,
        metric: METRIC.to_string(),
        baseline_revision: baseline.revision.clone(),
        decision,
        cases,
        issues,
    })
}

fn add_baselines(
    baseline: &mut Baseline,
    additional: Vec<AdditionalBaseline>,
) -> Result<(), GateError> {
    let mut identifiers: HashSet<CaseId> =
        baseline.cases.iter().map(|case| case.id.clone()).collect();
    for additional in additional {
        let source = SourceCase::from(additional);
        if !identifiers.insert(source.id.clone()) {
            return Err(GateError::Baseline(format!("duplicate case {}", source.id)));
        }
        let measured = measure(&source)?;
        if measured.assertions.is_empty() && measured.expected.is_none() {
            return Err(GateError::Case {
                id: source.id,
                reason: "a new baseline needs assertions or an expected return".to_string(),
            });
        }
        baseline.cases.push(measured);
    }
    Ok(())
}

fn git(root: &Path, arguments: &[&str]) -> Result<String, GateError> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()
        .map_err(|source| GateError::Io {
            path: root.join(".git"),
            source,
        })?;
    if !output.status.success() {
        return Err(GateError::Git {
            arguments: arguments.join(" "),
            reason: format!(
                "{}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        });
    }
    String::from_utf8(output.stdout).map_err(|error| GateError::Git {
        arguments: arguments.join(" "),
        reason: error.to_string(),
    })
}

fn read(path: &Path) -> Result<String, GateError> {
    fs::read_to_string(path).map_err(|source| GateError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn artifact(variable: &'static str) -> Result<PathBuf, GateError> {
    env::var_os(variable)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or(GateError::MissingEnvironment(variable))
}

fn write_new<T: Serialize>(path: &Path, value: &T) -> Result<(), GateError> {
    let output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| GateError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    serde_json::to_writer_pretty(output, value)?;
    Ok(())
}

fn read_proposal(root: &Path, path: String) -> Result<Proposal, GateError> {
    let file = Path::new(&path);
    let number = file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.strip_prefix("nuc-"));
    if file.parent() != Some(Path::new("docs"))
        || file.extension().is_none_or(|extension| extension != "md")
        || number.is_none_or(|number| {
            number.len() != 4 || !number.bytes().all(|byte| byte.is_ascii_digit())
        })
    {
        return Err(GateError::Proposal(
            "use the repository-relative path of a docs/nuc-XXXX.md file".to_string(),
        ));
    }
    let revision = git(root, &["hash-object", "--", &path])?.trim().to_string();
    Ok(Proposal { path, revision })
}

#[test]
#[ignore = "set NUC_PROPOSAL_BASELINE before recording the accepted syntax"]
fn record_baseline() -> Result<(), GateError> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let reference = match env::var("NUC_PROPOSAL_BASE_REF") {
        Ok(reference) => reference,
        Err(env::VarError::NotPresent) => "HEAD".to_string(),
        Err(source) => {
            return Err(GateError::Environment {
                variable: "NUC_PROPOSAL_BASE_REF",
                source,
            });
        }
    };
    let revision = git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{reference}^{{commit}}"),
        ],
    )?
    .trim()
    .to_string();
    git(
        root,
        &[
            "diff",
            "--quiet",
            &revision,
            "--",
            "src",
            "Cargo.toml",
            "Cargo.lock",
        ],
    )
    .map_err(|error| GateError::Baseline(format!(
        "record with the baseline runtime before editing it, or in its own Git worktree: {error}"
    )))?;

    let mut sources = Vec::new();
    for document in Document::ALL {
        let text = git(root, &["show", &format!("{revision}:{}", document.path())])?;
        sources.extend(document.cases(&text));
    }
    let mut baseline = capture(revision, &sources)?;
    if let Some(path) = env::var_os("NUC_PROPOSAL_EXTRA_BASELINES") {
        let additional: Vec<AdditionalBaseline> = serde_json::from_str(&read(Path::new(&path))?)?;
        add_baselines(&mut baseline, additional)?;
    }
    let path = artifact("NUC_PROPOSAL_BASELINE")?;
    write_new(&path, &baseline)?;
    println!(
        "Recorded {} cases in {}",
        baseline.cases.len(),
        path.display()
    );
    Ok(())
}

#[test]
#[ignore = "set NUC_PROPOSAL_BASELINE, NUC_PROPOSAL, and NUC_PROPOSAL_REPORT"]
fn evaluate_proposal() -> Result<(), GateError> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let baseline: Baseline = serde_json::from_str(&read(&artifact("NUC_PROPOSAL_BASELINE")?)?)?;
    let proposal_path = env::var("NUC_PROPOSAL").map_err(|source| GateError::Environment {
        variable: "NUC_PROPOSAL",
        source,
    })?;
    if proposal_path.is_empty() {
        return Err(GateError::MissingEnvironment("NUC_PROPOSAL"));
    }
    let proposal = read_proposal(root, proposal_path)?;
    let mut sources = Vec::new();
    for document in Document::ALL {
        sources.extend(document.cases(&read(&root.join(document.path()))?));
    }
    let report = compare(&baseline, proposal, &sources)?;
    let current = read_proposal(root, report.proposal.path.clone())?;
    if current.revision != report.proposal.revision {
        return Err(GateError::Proposal(
            "the proposal changed during evaluation; evaluate the new revision".to_string(),
        ));
    }
    let path = artifact("NUC_PROPOSAL_REPORT")?;
    write_new(&path, &report)?;
    println!(
        "{:?}: {} less, {} same, {} more; {} issues. Report: {}",
        report.decision,
        report
            .cases
            .iter()
            .filter(|case| case.change == TokenChange::Less)
            .count(),
        report
            .cases
            .iter()
            .filter(|case| case.change == TokenChange::Same)
            .count(),
        report
            .cases
            .iter()
            .filter(|case| case.change == TokenChange::More)
            .count(),
        report.issues.len(),
        path.display()
    );
    if report.decision != Decision::Accepted {
        return Err(GateError::Verdict {
            decision: report.decision,
            report: path,
        });
    }
    Ok(())
}

fn fixture(title: &str, source: &str) -> SourceCase {
    SourceCase {
        id: CaseId {
            document: Document::Spec,
            title: title.to_string(),
        },
        case: common::Case {
            title: title.to_string(),
            source: source.to_string(),
            expected: None,
        },
    }
}

fn verdict(before: &[SourceCase], after: &[SourceCase]) -> Report {
    let baseline = capture("baseline-revision".to_string(), before).unwrap();
    compare(&baseline, fixture_proposal(), after).unwrap()
}

fn fixture_proposal() -> Proposal {
    Proposal {
        path: "docs/nuc-9999.md".to_string(),
        revision: "proposal-revision".to_string(),
    }
}

#[test]
fn equal_tokens_are_accepted() {
    let cases = [fixture("same", "a = 1\nassert(a, 1)")];
    let report = verdict(&cases, &cases);
    assert_eq!(report.decision, Decision::Accepted);
    assert_eq!(report.cases[0].change, TokenChange::Same);
    assert_eq!(report.cases[0].before, 10);
}

#[test]
fn fewer_tokens_are_accepted() {
    let report = verdict(
        &[fixture("less", "a = (1)\nassert(a, 1)")],
        &[fixture("less", "a = 1\nassert(a, 1)")],
    );
    assert_eq!(report.decision, Decision::Accepted);
    assert_eq!(report.cases[0].change, TokenChange::Less);
    assert_eq!(report.cases[0].before - report.cases[0].after, 2);
}

#[test]
fn one_increase_is_rejected_even_when_another_case_saves_more() {
    let report = verdict(
        &[
            fixture("more", "a = 1\nassert(a, 1)"),
            fixture("less", "a = ((1))\nassert(a, 1)"),
        ],
        &[
            fixture("more", "a = (1)\nassert(a, 1)"),
            fixture("less", "a = 1\nassert(a, 1)"),
        ],
    );
    assert_eq!(report.decision, Decision::Rejected);
    assert_eq!(report.cases[0].change, TokenChange::More);
    assert_eq!(report.cases[1].change, TokenChange::Less);
}

#[test]
fn layout_tokens_count_but_eof_does_not() {
    let case = fixture("layout", "if true:\n    assert(1, 1)");
    assert_eq!(measure(&case).unwrap().tokens, 12);
}

#[test]
fn missing_and_renamed_cases_are_rejected() {
    let before = [fixture("original", "assert(1, 1)")];
    assert_eq!(verdict(&before, &[]).decision, Decision::Rejected);
    assert_eq!(
        verdict(&before, &[fixture("renamed", "assert(1, 1)")]).decision,
        Decision::Rejected
    );
}

#[test]
fn new_cases_require_an_old_syntax_baseline() {
    let before = [fixture("original", "assert(1, 1)")];
    let after = [
        fixture("original", "assert(1, 1)"),
        fixture("new", "assert(2, 2)"),
    ];
    let report = verdict(&before, &after);
    assert_eq!(report.decision, Decision::Blocked);
    assert!(matches!(report.issues[0], Issue::UnpairedCase { .. }));
    assert_eq!(verdict(&after, &after).decision, Decision::Accepted);
}

#[test]
fn additional_baselines_pair_new_cases_using_the_accepted_runtime() {
    let mut baseline = capture(
        "revision".to_string(),
        &[fixture("original", "assert(1, 1)")],
    )
    .unwrap();
    add_baselines(
        &mut baseline,
        vec![AdditionalBaseline {
            document: Document::Spec,
            title: "new".to_string(),
            source: "a = (2)\nassert(a, 2)".to_string(),
            expected: None,
        }],
    )
    .unwrap();
    let report = compare(
        &baseline,
        fixture_proposal(),
        &[
            fixture("original", "assert(1, 1)"),
            fixture("new", "a = 2\nassert(a, 2)"),
        ],
    )
    .unwrap();
    assert_eq!(report.decision, Decision::Accepted);
    assert_eq!(report.cases[1].change, TokenChange::Less);
}

#[test]
fn additional_baselines_need_observable_behavior() {
    let mut baseline = capture(
        "revision".to_string(),
        &[fixture("original", "assert(1, 1)")],
    )
    .unwrap();
    assert!(
        add_baselines(
            &mut baseline,
            vec![AdditionalBaseline {
                document: Document::Spec,
                title: "empty".to_string(),
                source: "pass".to_string(),
                expected: None,
            }]
        )
        .is_err()
    );
}

#[test]
fn assertions_cannot_be_removed_or_weakened() {
    let before = [fixture("assertions", "a = 1\nassert(a, 1)")];
    for source in ["a = 1", "a = 1\nassert(1, 1)"] {
        let report = verdict(&before, &[fixture("assertions", source)]);
        assert_eq!(report.decision, Decision::Rejected);
        assert!(matches!(report.issues[0], Issue::ChangedAssertions { .. }));
    }
}

#[test]
fn return_expectations_cannot_change() {
    let mut before = fixture("return", "1");
    before.case.expected = Some("1".to_string());
    let after = fixture("return", "1");
    let report = verdict(&[before], &[after]);
    assert_eq!(report.decision, Decision::Rejected);
    assert!(matches!(report.issues[0], Issue::ChangedReturn { .. }));
}

#[test]
fn incorrect_and_unexecuted_assertions_are_rejected() {
    let before = [fixture("coverage", "assert(1, 1)")];
    for source in ["assert(1, 2)", "if false:\n    assert (1, 1)", "assert("] {
        let report = verdict(&before, &[fixture("coverage", source)]);
        assert_eq!(report.decision, Decision::Rejected);
        assert!(matches!(report.issues[0], Issue::InvalidCase { .. }));
    }
}

#[test]
fn baseline_snapshots_round_trip_and_reject_duplicate_cases() {
    let cases = [fixture("snapshot", "assert(1, 1)")];
    let baseline = capture("revision".to_string(), &cases).unwrap();
    let json = serde_json::to_string(&baseline).unwrap();
    let restored = serde_json::from_str(&json).unwrap();
    assert_eq!(
        compare(&restored, fixture_proposal(), &cases)
            .unwrap()
            .decision,
        Decision::Accepted
    );
    assert!(
        capture(
            "revision".to_string(),
            &[
                fixture("duplicate", "assert(1, 1)"),
                fixture("duplicate", "assert(1, 1)")
            ]
        )
        .is_err()
    );
}
