//! Deterministic behavior, integrity, and optional token checks for NUCs.
//!
//! The ignored entry points record an immutable baseline and compare a rewritten
//! corpus. Ordinary tests supply policy inputs directly, without mutating the
//! process environment.

mod common;

use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Command;

use indexmap::IndexMap;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const METRIC: &str = "nucleoid-lexer-with-layout-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExecutionMode {
    Continuous,
    Single,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Category {
    #[serde(rename = "Syntax enhancement")]
    Syntax,
    #[serde(rename = "Runtime features")]
    Runtime,
    #[serde(rename = "Utility enhancement")]
    Utility,
    #[serde(rename = "Reasoning enhancement")]
    Reasoning,
    Documentation,
}

impl Category {
    fn from_label(label: &str) -> Result<Self, GateError> {
        match label {
            "Syntax enhancement" => Ok(Self::Syntax),
            "Runtime features" => Ok(Self::Runtime),
            "Utility enhancement" => Ok(Self::Utility),
            "Reasoning enhancement" => Ok(Self::Reasoning),
            "Documentation" => Ok(Self::Documentation),
            _ => Err(GateError::Proposal(format!(
                "unsupported Category {label:?}; confirm its registry and policy before evaluation"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RuntimeApplicability {
    NotApplicable,
    NewFeature,
    ExistingComparable,
    ExistingUncomparable { reason: String, evidence: String },
}

#[derive(Default)]
struct PolicyInputs<'a> {
    mode: Option<&'a str>,
    runtime: Option<&'a str>,
    reason: Option<&'a str>,
    evidence: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct EvaluationPolicy {
    mode: ExecutionMode,
    category: Category,
    runtime: RuntimeApplicability,
}

impl EvaluationPolicy {
    fn from_inputs(category: Category, inputs: PolicyInputs<'_>) -> Result<Self, GateError> {
        let mode = match inputs.mode {
            None | Some("continuous") => ExecutionMode::Continuous,
            Some("single") => ExecutionMode::Single,
            Some(value) => {
                return Err(GateError::Input(format!(
                    "NUC_PROPOSAL_MODE must be continuous or single, not {value:?}"
                )));
            }
        };
        let runtime = match (category, inputs.runtime) {
            (Category::Runtime, Some("new_feature")) => RuntimeApplicability::NewFeature,
            (Category::Runtime, Some("existing_comparable")) => {
                RuntimeApplicability::ExistingComparable
            }
            (Category::Runtime, Some("existing_uncomparable")) => {
                RuntimeApplicability::ExistingUncomparable {
                    reason: required_rationale("NUC_PROPOSAL_RUNTIME_REASON", inputs.reason)?,
                    evidence: required_rationale("NUC_PROPOSAL_RUNTIME_EVIDENCE", inputs.evidence)?,
                }
            }
            (Category::Runtime, _) => {
                return Err(GateError::Input(
                    "Runtime features requires NUC_PROPOSAL_RUNTIME: new_feature, \
                     existing_comparable, or existing_uncomparable"
                        .to_string(),
                ));
            }
            (_, None) => RuntimeApplicability::NotApplicable,
            (_, Some(_)) => {
                return Err(GateError::Input(
                    "NUC_PROPOSAL_RUNTIME is only valid for Runtime features".to_string(),
                ));
            }
        };
        if !matches!(runtime, RuntimeApplicability::ExistingUncomparable { .. })
            && (inputs.reason.is_some() || inputs.evidence.is_some())
        {
            return Err(GateError::Input(
                "runtime reason/evidence are only valid for existing_uncomparable".to_string(),
            ));
        }
        Ok(Self {
            mode,
            category,
            runtime,
        })
    }

    fn skip_reason(&self) -> Option<TokenSkipReason> {
        if self.mode == ExecutionMode::Single {
            return Some(TokenSkipReason::SingleProposal);
        }
        match self.category {
            Category::Documentation => Some(TokenSkipReason::Documentation),
            Category::Runtime => match self.runtime {
                RuntimeApplicability::NewFeature => Some(TokenSkipReason::NewRuntimeFeature),
                RuntimeApplicability::ExistingUncomparable { .. } => {
                    Some(TokenSkipReason::UncomparableRuntimeFeature)
                }
                RuntimeApplicability::ExistingComparable | RuntimeApplicability::NotApplicable => {
                    None
                }
            },
            Category::Syntax | Category::Utility | Category::Reasoning => None,
        }
    }
}

fn required_rationale(variable: &str, value: Option<&str>) -> Result<String, GateError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| GateError::Input(format!("{variable} needs a nonblank rationale")))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TokenSkipReason {
    SingleProposal,
    Documentation,
    NewRuntimeFeature,
    UncomparableRuntimeFeature,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MeasuredCase {
    id: CaseId,
    tokens: usize,
    assertions: Vec<String>,
    expected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    InvalidLegacyCase { id: CaseId, reason: String },
    ChangedLegacyAssertions { id: CaseId },
    MoreTokens { id: CaseId },
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum TokenCheck {
    Ran {
        metric: String,
        cases: Vec<Comparison>,
    },
    Skipped {
        reason: TokenSkipReason,
    },
}

#[derive(Debug, Serialize)]
struct BehaviorChecks {
    legacy_cases_passed: usize,
    candidate_cases_passed: usize,
}

#[derive(Debug, Serialize)]
struct Proposal {
    path: String,
    revision: String,
    category: Category,
}

#[derive(Debug, Serialize)]
struct Report {
    proposal: Proposal,
    baseline_revision: String,
    policy: EvaluationPolicy,
    token_check: TokenCheck,
    behavior: BehaviorChecks,
    decision: Decision,
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
    #[error("invalid evaluation input: {0}")]
    Input(String),
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

fn check_behavior(source: &SourceCase) -> Result<Vec<String>, GateError> {
    let statements = parse(source.case.source.trim()).map_err(|error| GateError::Case {
        id: source.id.clone(),
        reason: error.to_string(),
    })?;
    let assertions = common::assertions::protected_assertions(&statements);
    common::check_with_assertions(&source.case, assertions.len()).map_err(|reason| {
        GateError::Case {
            id: source.id.clone(),
            reason,
        }
    })?;
    Ok(assertions)
}

fn count_tokens(source: &SourceCase) -> Result<usize, GateError> {
    let tokens = tokenize(source.case.source.trim()).map_err(|error| GateError::Case {
        id: source.id.clone(),
        reason: error.to_string(),
    })?;
    Ok(tokens
        .iter()
        .filter(|spanned| spanned.token != Token::Eof)
        .count())
}

fn measure(source: &SourceCase) -> Result<MeasuredCase, GateError> {
    Ok(MeasuredCase {
        id: source.id.clone(),
        tokens: count_tokens(source)?,
        assertions: check_behavior(source)?,
        expected: source.case.expected.clone(),
    })
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
    let baseline = Baseline {
        metric: METRIC.to_string(),
        revision,
        cases,
    };
    baseline_cases(&baseline)?;
    Ok(baseline)
}

fn baseline_cases(baseline: &Baseline) -> Result<IndexMap<CaseId, &MeasuredCase>, GateError> {
    if baseline.metric != METRIC
        || !matches!(baseline.revision.len(), 40 | 64)
        || !baseline
            .revision
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || baseline.cases.is_empty()
    {
        return Err(GateError::Baseline(
            "expected a full Git revision and a nonempty corpus using the current metric"
                .to_string(),
        ));
    }
    let mut cases = IndexMap::new();
    for case in &baseline.cases {
        if case.id.title.trim().is_empty()
            || case.tokens == 0
            || case
                .assertions
                .iter()
                .any(|assertion| assertion.trim().is_empty())
        {
            return Err(GateError::Baseline(format!(
                "invalid metadata for {}",
                case.id
            )));
        }
        if let Some(expected) = &case.expected {
            serde_json::from_str::<serde_json::Value>(expected).map_err(|error| {
                GateError::Baseline(format!("invalid return for {}: {error}", case.id))
            })?;
        }
        if cases.insert(case.id.clone(), case).is_some() {
            return Err(GateError::Baseline(format!("duplicate case {}", case.id)));
        }
    }
    Ok(cases)
}

fn check_legacy(
    baseline: &Baseline,
    sources: &[SourceCase],
    issues: &mut Vec<Issue>,
) -> Result<usize, GateError> {
    let mut remaining = baseline_cases(baseline)?;
    let mut passed = 0;
    for source in sources {
        let before = remaining.shift_remove(&source.id).ok_or_else(|| {
            GateError::Baseline(format!("missing or duplicate accepted case {}", source.id))
        })?;
        if before.expected != source.case.expected {
            return Err(GateError::Baseline(format!(
                "return metadata differs from the accepted source for {}",
                source.id
            )));
        }
        match check_behavior(source) {
            Ok(assertions) => {
                passed += 1;
                if before.assertions != assertions {
                    issues.push(Issue::ChangedLegacyAssertions {
                        id: source.id.clone(),
                    });
                }
            }
            Err(error) => issues.push(Issue::InvalidLegacyCase {
                id: source.id.clone(),
                reason: error.to_string(),
            }),
        }
    }
    if let Some((id, _)) = remaining.first() {
        return Err(GateError::Baseline(format!(
            "no accepted source supplied for {id}; include the original extra baselines"
        )));
    }
    Ok(passed)
}

fn compare(
    baseline: &Baseline,
    proposal: Proposal,
    policy: EvaluationPolicy,
    legacy_sources: &[SourceCase],
    sources: &[SourceCase],
) -> Result<Report, GateError> {
    if proposal.category != policy.category {
        return Err(GateError::Input(
            "policy category differs from the proposal header".to_string(),
        ));
    }
    let mut remaining = baseline_cases(baseline)?;
    let mut issues = Vec::new();
    let mut behavior = BehaviorChecks {
        legacy_cases_passed: check_legacy(baseline, legacy_sources, &mut issues)?,
        candidate_cases_passed: 0,
    };
    let mut token_check = match policy.skip_reason() {
        Some(reason) => TokenCheck::Skipped { reason },
        None => TokenCheck::Ran {
            metric: METRIC.to_string(),
            cases: Vec::new(),
        },
    };
    let mut identifiers = HashSet::new();
    for source in sources {
        if source.id.title.trim().is_empty()
            || source.id.title != source.case.title
            || !identifiers.insert(source.id.clone())
        {
            return Err(GateError::Case {
                id: source.id.clone(),
                reason: "invalid or duplicate case identity".to_string(),
            });
        }
        let before = remaining.shift_remove(&source.id);
        let assertions = match check_behavior(source) {
            Ok(assertions) => {
                behavior.candidate_cases_passed += 1;
                assertions
            }
            Err(error) => {
                issues.push(Issue::InvalidCase {
                    id: source.id.clone(),
                    reason: error.to_string(),
                });
                continue;
            }
        };
        let Some(before) = before else {
            if matches!(token_check, TokenCheck::Ran { .. }) {
                issues.push(Issue::UnpairedCase {
                    id: source.id.clone(),
                });
            } else if assertions.is_empty() && source.case.expected.is_none() {
                issues.push(Issue::InvalidCase {
                    id: source.id.clone(),
                    reason: "a new case needs assertions or an expected return".to_string(),
                });
            }
            continue;
        };
        if before.assertions != assertions {
            issues.push(Issue::ChangedAssertions {
                id: source.id.clone(),
            });
        }
        if before.expected != source.case.expected {
            issues.push(Issue::ChangedReturn {
                id: source.id.clone(),
            });
        }
        if let TokenCheck::Ran { cases, .. } = &mut token_check {
            let after = count_tokens(source)?;
            let change = match after.cmp(&before.tokens) {
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
                after,
                change,
            });
        }
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
        baseline_revision: baseline.revision.clone(),
        policy,
        token_check,
        behavior,
        decision,
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
    baseline_cases(baseline)?;
    Ok(())
}

fn git(root: &Path, arguments: &[&str]) -> Result<String, GateError> {
    let output = Command::new("git")
        .arg("--no-pager")
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

fn optional_environment(variable: &'static str) -> Result<Option<String>, GateError> {
    match env::var(variable) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(source) => Err(GateError::Environment { variable, source }),
    }
}

fn read_policy(category: Category) -> Result<EvaluationPolicy, GateError> {
    let mode = optional_environment("NUC_PROPOSAL_MODE")?;
    let runtime = optional_environment("NUC_PROPOSAL_RUNTIME")?;
    let reason = optional_environment("NUC_PROPOSAL_RUNTIME_REASON")?;
    let evidence = optional_environment("NUC_PROPOSAL_RUNTIME_EVIDENCE")?;
    EvaluationPolicy::from_inputs(
        category,
        PolicyInputs {
            mode: mode.as_deref(),
            runtime: runtime.as_deref(),
            reason: reason.as_deref(),
            evidence: evidence.as_deref(),
        },
    )
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

fn header_category(source: &str, registry: &str) -> Result<Category, GateError> {
    let mut lines = source.lines();
    lines
        .find(|line| line.trim_start().starts_with("```"))
        .filter(|line| line.trim() == "```")
        .ok_or_else(|| {
            GateError::Proposal("missing or invalid opening NUC header fence".to_string())
        })?;
    let mut header = Vec::new();
    loop {
        let line = lines
            .next()
            .ok_or_else(|| GateError::Proposal("missing closing NUC header fence".to_string()))?;
        if line.trim_start().starts_with("```") {
            if line.trim() != "```" {
                return Err(GateError::Proposal(
                    "invalid closing NUC header fence".to_string(),
                ));
            }
            break;
        }
        header.push(line);
    }
    if !header.iter().any(|line| line.starts_with("NUC:")) {
        return Err(GateError::Proposal(
            "the first fenced block must be the NUC header".to_string(),
        ));
    }
    let mut labels = header
        .iter()
        .filter_map(|line| line.strip_prefix("Category:").map(str::trim));
    let label = labels
        .next()
        .ok_or_else(|| GateError::Proposal("missing Category header".to_string()))?;
    if labels.next().is_some() {
        return Err(GateError::Proposal("duplicate Category header".to_string()));
    }
    let category = Category::from_label(label)?;
    let entries = registry
        .split_once("## Proposal Categories")
        .map(|(_, section)| section.split("\n## ").next().unwrap_or(section))
        .ok_or_else(|| GateError::Proposal("missing proposal category registry".to_string()))?;
    let registrations = entries
        .lines()
        .filter_map(|line| line.trim().strip_prefix('|'))
        .filter_map(|row| row.split('|').next())
        .filter(|entry| entry.trim() == label)
        .count();
    if registrations != 1 {
        return Err(GateError::Proposal(format!(
            "Category {label:?} must occur exactly once in the NUC 1 registry"
        )));
    }
    Ok(category)
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
    let category = header_category(
        &read(&root.join(&path))?,
        &read(&root.join("docs/nuc-0001.md"))?,
    )?;
    if git(root, &["hash-object", "--", &path])?.trim() != revision {
        return Err(GateError::Proposal(
            "the proposal changed while reading its header".to_string(),
        ));
    }
    Ok(Proposal {
        path,
        revision,
        category,
    })
}

fn sources_at_revision(root: &Path, revision: &str) -> Result<Vec<SourceCase>, GateError> {
    let resolved = git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{revision}^{{commit}}"),
        ],
    )?;
    if resolved.trim() != revision {
        return Err(GateError::Baseline(
            "the baseline revision must be the full accepted commit ID".to_string(),
        ));
    }
    let mut sources = Vec::new();
    for document in Document::ALL {
        let text = git(root, &["show", &format!("{revision}:{}", document.path())])?;
        sources.extend(document.cases(&text));
    }
    Ok(sources)
}

fn extra_baselines() -> Result<Vec<AdditionalBaseline>, GateError> {
    match env::var_os("NUC_PROPOSAL_EXTRA_BASELINES") {
        Some(path) => Ok(serde_json::from_str(&read(Path::new(&path))?)?),
        None => Ok(Vec::new()),
    }
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

    let sources = sources_at_revision(root, &revision)?;
    let mut baseline = capture(revision, &sources)?;
    add_baselines(&mut baseline, extra_baselines()?)?;
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
    baseline_cases(&baseline)?;
    let proposal_path = env::var("NUC_PROPOSAL").map_err(|source| GateError::Environment {
        variable: "NUC_PROPOSAL",
        source,
    })?;
    if proposal_path.is_empty() {
        return Err(GateError::MissingEnvironment("NUC_PROPOSAL"));
    }
    let proposal = read_proposal(root, proposal_path)?;
    let policy = read_policy(proposal.category)?;
    let mut legacy_sources = sources_at_revision(root, &baseline.revision)?;
    legacy_sources.extend(extra_baselines()?.into_iter().map(SourceCase::from));
    let mut sources = Vec::new();
    for document in Document::ALL {
        sources.extend(document.cases(&read(&root.join(document.path()))?));
    }
    let report = compare(&baseline, proposal, policy, &legacy_sources, &sources)?;
    let current = read_proposal(root, report.proposal.path.clone())?;
    if current.revision != report.proposal.revision {
        return Err(GateError::Proposal(
            "the proposal changed during evaluation; evaluate the new revision".to_string(),
        ));
    }
    let path = artifact("NUC_PROPOSAL_REPORT")?;
    write_new(&path, &report)?;
    println!("Policy: {}", serde_json::to_string(&report.policy)?);
    match &report.token_check {
        TokenCheck::Ran { cases, .. } => println!(
            "Token check ran: {} less, {} same, {} more",
            cases
                .iter()
                .filter(|case| case.change == TokenChange::Less)
                .count(),
            cases
                .iter()
                .filter(|case| case.change == TokenChange::Same)
                .count(),
            cases
                .iter()
                .filter(|case| case.change == TokenChange::More)
                .count(),
        ),
        TokenCheck::Skipped { reason } => println!("Token check skipped: {reason:?}"),
    }
    println!(
        "{:?}; {} issues. Report: {}",
        report.decision,
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
    verdict_with_policy(before, after, required_policy())
}

const FIXTURE_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn required_policy() -> EvaluationPolicy {
    EvaluationPolicy::from_inputs(Category::Syntax, PolicyInputs::default()).unwrap()
}

fn verdict_with_policy(
    before: &[SourceCase],
    after: &[SourceCase],
    policy: EvaluationPolicy,
) -> Report {
    let baseline = capture(FIXTURE_REVISION.to_string(), before).unwrap();
    let mut proposal = fixture_proposal();
    proposal.category = policy.category;
    compare(&baseline, proposal, policy, before, after).unwrap()
}

fn comparisons(report: &Report) -> &[Comparison] {
    match &report.token_check {
        TokenCheck::Ran { cases, .. } => cases,
        TokenCheck::Skipped { .. } => panic!("this test requires an actual token comparison"),
    }
}

fn fixture_proposal() -> Proposal {
    Proposal {
        path: "docs/nuc-9999.md".to_string(),
        revision: "proposal-revision".to_string(),
        category: Category::Syntax,
    }
}

#[test]
fn equal_tokens_are_accepted() {
    let cases = [fixture("same", "a = 1\nassert(a, 1)")];
    let report = verdict(&cases, &cases);
    assert_eq!(report.decision, Decision::Accepted);
    assert_eq!(comparisons(&report)[0].change, TokenChange::Same);
    assert_eq!(comparisons(&report)[0].before, 10);
}

#[test]
fn fewer_tokens_are_accepted() {
    let report = verdict(
        &[fixture("less", "a = (1)\nassert(a, 1)")],
        &[fixture("less", "a = 1\nassert(a, 1)")],
    );
    assert_eq!(report.decision, Decision::Accepted);
    assert_eq!(comparisons(&report)[0].change, TokenChange::Less);
    assert_eq!(
        comparisons(&report)[0].before - comparisons(&report)[0].after,
        2
    );
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
    assert_eq!(comparisons(&report)[0].change, TokenChange::More);
    assert_eq!(comparisons(&report)[1].change, TokenChange::Less);
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
        FIXTURE_REVISION.to_string(),
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
        required_policy(),
        &[
            fixture("original", "assert(1, 1)"),
            fixture("new", "a = (2)\nassert(a, 2)"),
        ],
        &[
            fixture("original", "assert(1, 1)"),
            fixture("new", "a = 2\nassert(a, 2)"),
        ],
    )
    .unwrap();
    assert_eq!(report.decision, Decision::Accepted);
    assert_eq!(comparisons(&report)[1].change, TokenChange::Less);
}

#[test]
fn additional_baselines_need_observable_behavior() {
    let mut baseline = capture(
        FIXTURE_REVISION.to_string(),
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
fn additional_baselines_reject_invalid_metadata_before_recording() {
    let cases = [fixture("original", "assert(1, 1)")];
    for title in ["", " \n "] {
        let mut baseline = capture(FIXTURE_REVISION.to_string(), &cases).unwrap();
        let result = add_baselines(
            &mut baseline,
            vec![AdditionalBaseline {
                document: Document::Spec,
                title: title.to_string(),
                source: "assert(2, 2)".to_string(),
                expected: None,
            }],
        );
        assert!(
            matches!(result, Err(GateError::Baseline(_))),
            "accepted invalid baseline title {title:?}"
        );
    }
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
    let baseline = capture(FIXTURE_REVISION.to_string(), &cases).unwrap();
    let legacy_json = serde_json::json!({
        "metric": "nucleoid-lexer-with-layout-v1",
        "revision": FIXTURE_REVISION,
        "cases": [{
            "id": {"document": "spec", "title": "snapshot"},
            "tokens": 6,
            "assertions": ["assert(1,1)"],
            "expected": null
        }]
    });
    let restored: Baseline = serde_json::from_value(legacy_json.clone()).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), legacy_json);
    assert_eq!(serde_json::to_value(&baseline).unwrap(), legacy_json);
    assert_eq!(
        compare(
            &restored,
            fixture_proposal(),
            required_policy(),
            &cases,
            &cases
        )
        .unwrap()
        .decision,
        Decision::Accepted
    );
    assert!(
        capture(
            FIXTURE_REVISION.to_string(),
            &[
                fixture("duplicate", "assert(1, 1)"),
                fixture("duplicate", "assert(1, 1)")
            ]
        )
        .is_err()
    );
}

#[test]
#[ignore = "set NUC_PROPOSAL_BASELINE to replay an accepted artifact without evaluating a proposal"]
fn replay_accepted_baseline_without_evaluation() -> Result<(), GateError> {
    let json: serde_json::Value =
        serde_json::from_str(&read(&artifact("NUC_PROPOSAL_BASELINE")?)?)?;
    let baseline: Baseline = serde_json::from_value(json.clone())?;
    assert_eq!(serde_json::to_value(&baseline)?, json);

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = sources_at_revision(root, &baseline.revision)?;
    sources.extend(extra_baselines()?.into_iter().map(SourceCase::from));
    let mut issues = Vec::new();
    let passed = check_legacy(&baseline, &sources, &mut issues)?;
    assert!(issues.is_empty(), "{issues:#?}");
    assert_eq!(passed, baseline.cases.len());
    println!("Replayed {passed} accepted cases; legacy schema and canonical assertions match.");
    Ok(())
}

fn policy_matrix() -> Vec<(EvaluationPolicy, Option<TokenSkipReason>)> {
    let categories = [
        (Category::Syntax, None, None),
        (Category::Utility, None, None),
        (Category::Reasoning, None, None),
        (
            Category::Documentation,
            None,
            Some(TokenSkipReason::Documentation),
        ),
        (
            Category::Runtime,
            Some("new_feature"),
            Some(TokenSkipReason::NewRuntimeFeature),
        ),
        (Category::Runtime, Some("existing_comparable"), None),
        (
            Category::Runtime,
            Some("existing_uncomparable"),
            Some(TokenSkipReason::UncomparableRuntimeFeature),
        ),
    ];
    let mut policies = Vec::new();
    for mode in ["continuous", "single"] {
        for (category, runtime, continuous_skip) in categories {
            let uncomparable = runtime == Some("existing_uncomparable");
            let policy = EvaluationPolicy::from_inputs(
                category,
                PolicyInputs {
                    mode: Some(mode),
                    runtime,
                    reason: uncomparable.then_some("the existing observation is nondeterministic"),
                    evidence: uncomparable.then_some("accepted spec entry and offline trace"),
                },
            )
            .unwrap();
            let skip = if mode == "single" {
                Some(TokenSkipReason::SingleProposal)
            } else {
                continuous_skip
            };
            policies.push((policy, skip));
        }
    }
    policies
}

fn skipped_policies() -> impl Iterator<Item = EvaluationPolicy> {
    policy_matrix()
        .into_iter()
        .filter_map(|(policy, skip)| skip.map(|_| policy))
}

#[test]
fn absent_mode_preserves_the_continuous_default() {
    let policy = required_policy();
    assert_eq!(policy.mode, ExecutionMode::Continuous);
    assert_eq!(policy.skip_reason(), None);
}

#[test]
fn complete_mode_category_matrix_enforces_required_token_increases() {
    let before = [fixture("paired", "a = 1\nassert(a, 1)")];
    let after = [fixture("paired", "a = (1)\nassert(a, 1)")];
    for (policy, skip) in policy_matrix() {
        assert_eq!(policy.skip_reason(), skip);
        let report = verdict_with_policy(&before, &after, policy.clone());
        assert_eq!(report.policy, policy);
        assert_eq!(report.behavior.legacy_cases_passed, 1);
        assert_eq!(report.behavior.candidate_cases_passed, 1);
        match skip {
            None => {
                assert_eq!(report.decision, Decision::Rejected, "{policy:?}");
                assert_eq!(comparisons(&report)[0].change, TokenChange::More);
                assert!(matches!(report.issues[0], Issue::MoreTokens { .. }));
            }
            Some(reason) => {
                assert_eq!(report.decision, Decision::Accepted, "{policy:?}");
                assert!(
                    matches!(report.token_check, TokenCheck::Skipped { reason: actual } if actual == reason)
                );
                let json = serde_json::to_value(&report).unwrap();
                assert_eq!(json["token_check"]["status"], "skipped");
                assert!(json["token_check"].get("cases").is_none());
                assert!(json["token_check"].get("metric").is_none());
                assert!(json.get("cases").is_none());
            }
        }
    }
}

#[test]
fn new_behavior_needs_a_comparison_only_when_the_matrix_requires_it() {
    let before = [fixture("paired", "assert(1, 1)")];
    for (policy, skip) in policy_matrix() {
        let mut new_return = fixture("new return", "2");
        new_return.case.expected = Some("2".to_string());
        let after = [
            fixture("paired", "assert(1, 1)"),
            fixture("new assertion", "assert(2, 2)"),
            new_return,
        ];
        let report = verdict_with_policy(&before, &after, policy.clone());
        assert_eq!(report.behavior.candidate_cases_passed, 3);
        if skip.is_some() {
            assert_eq!(report.decision, Decision::Accepted, "{policy:?}");
            assert!(report.issues.is_empty());
        } else {
            assert_eq!(report.decision, Decision::Blocked, "{policy:?}");
            assert_eq!(report.issues.len(), 2);
            assert!(
                report
                    .issues
                    .iter()
                    .all(|issue| matches!(issue, Issue::UnpairedCase { .. }))
            );
        }
    }
}

#[test]
fn runtime_noncomparison_requires_validated_reason_and_evidence() {
    let policy = EvaluationPolicy::from_inputs(
        Category::Runtime,
        PolicyInputs {
            mode: Some("continuous"),
            runtime: Some("existing_uncomparable"),
            reason: Some("  no deterministic accepted-syntax observation  "),
            evidence: Some("  accepted spec entry and observed trace  "),
        },
    )
    .unwrap();
    assert_eq!(
        policy.runtime,
        RuntimeApplicability::ExistingUncomparable {
            reason: "no deterministic accepted-syntax observation".to_string(),
            evidence: "accepted spec entry and observed trace".to_string(),
        }
    );
    let json = serde_json::to_value(policy).unwrap();
    assert_eq!(json["category"], "Runtime features");
    assert_eq!(json["runtime"]["kind"], "existing_uncomparable");
    assert_eq!(
        json["runtime"]["evidence"],
        "accepted spec entry and observed trace"
    );
}

#[test]
fn invalid_policy_inputs_never_select_a_skip() {
    for mode in [
        Some(""),
        Some("Single"),
        Some("ignore"),
        Some(" continuous "),
    ] {
        assert!(
            EvaluationPolicy::from_inputs(
                Category::Documentation,
                PolicyInputs {
                    mode,
                    ..PolicyInputs::default()
                },
            )
            .is_err()
        );
    }
    for mode in [None, Some("single")] {
        for runtime in [None, Some(""), Some("new"), Some("not_applicable")] {
            assert!(
                EvaluationPolicy::from_inputs(
                    Category::Runtime,
                    PolicyInputs {
                        mode,
                        runtime,
                        ..PolicyInputs::default()
                    },
                )
                .is_err()
            );
        }
        for runtime in [Some("new_feature"), Some("existing_comparable")] {
            assert!(
                EvaluationPolicy::from_inputs(
                    Category::Documentation,
                    PolicyInputs {
                        mode,
                        runtime,
                        ..PolicyInputs::default()
                    },
                )
                .is_err()
            );
        }
        for (reason, evidence) in [
            (None, None),
            (Some("reason"), None),
            (None, Some("trace")),
            (Some(" \n "), Some("trace")),
            (Some("reason"), Some(" \t ")),
        ] {
            assert!(
                EvaluationPolicy::from_inputs(
                    Category::Runtime,
                    PolicyInputs {
                        mode,
                        runtime: Some("existing_uncomparable"),
                        reason,
                        evidence,
                    },
                )
                .is_err()
            );
        }
    }
    for (category, runtime) in [
        (Category::Syntax, None),
        (Category::Documentation, None),
        (Category::Runtime, Some("new_feature")),
        (Category::Runtime, Some("existing_comparable")),
    ] {
        assert!(
            EvaluationPolicy::from_inputs(
                category,
                PolicyInputs {
                    mode: Some("single"),
                    runtime,
                    reason: Some("a stale reason must not silently select a skip"),
                    evidence: None,
                },
            )
            .is_err()
        );
    }
}

#[test]
fn category_comes_from_the_actual_registered_proposal_header() {
    let registry = include_str!("../docs/nuc-0001.md");
    let source = "```\nNUC: 99\nCategory: Utility enhancement\n```\n\
                  ## Format example\n```\nCategory: Documentation\n```";
    assert_eq!(
        header_category(source, registry).unwrap(),
        Category::Utility
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(
        read_proposal(root, "docs/nuc-0001.md".to_string())
            .unwrap()
            .category,
        Category::Documentation,
    );
    for invalid in [
        "",
        "```\nNUC: 99\nCategory: Documentation",
        "```\nNUC: 99\n```",
        "```\nCategory: Documentation\n```",
        "```\nNUC: 99\nCategory: Documentation\nCategory: Utility enhancement\n```",
        "```\nNUC: 99\nCategory: Other\n```",
    ] {
        assert!(header_category(invalid, registry).is_err(), "{invalid:?}");
    }
    assert!(header_category(source, "## Proposal Categories\n| Documentation | No |").is_err());
    assert!(
        header_category(
            source,
            "## Proposal Categories\n| Utility enhancement | Yes |\n| Utility enhancement | Yes |"
        )
        .is_err()
    );
}

#[test]
fn category_parsing_uses_complete_line_fences_not_inline_backticks() {
    let registry = include_str!("../docs/nuc-0001.md");
    let source = "# NUC 99 -- Header fences\n\
                  Inline opener ```\n\
                  NUC: 99\n\
                  Category: Documentation\n\
                  Inline closer ```\n\
                  ```\n\
                  NUC: 99\n\
                  Category: Syntax enhancement\n\
                  ```";
    let crlf = source.replace('\n', "\r\n");
    for source in [source, crlf.as_str()] {
        assert_eq!(header_category(source, registry).unwrap(), Category::Syntax);
    }
    for invalid in [
        "```\nNUC: 99\nCategory: Documentation\nInline closing ```",
        "```\nNUC: 99\nCategory: Documentation\n```not-a-closing-fence\n```",
        "````\nNUC: 99\nCategory: Documentation\n```",
    ] {
        assert!(header_category(invalid, registry).is_err(), "{invalid:?}");
    }
}

#[test]
fn a_policy_cannot_override_the_proposals_category() {
    let cases = [fixture("paired", "assert(1, 1)")];
    let baseline = capture(FIXTURE_REVISION.to_string(), &cases).unwrap();
    let policy =
        EvaluationPolicy::from_inputs(Category::Documentation, PolicyInputs::default()).unwrap();
    assert!(compare(&baseline, fixture_proposal(), policy, &cases, &cases).is_err());
}

#[test]
fn skipped_comparison_preserves_all_existing_integrity_checks() {
    for policy in skipped_policies() {
        let mut original = fixture("protected", "a = 1\nassert(a, 1)\nassert(2, 2)\na");
        original.case.expected = Some("1".to_string());
        let before = [original];
        for source in [
            "a = 1\nassert(a, 1)\na",
            "a = 1\nassert(1, 1)\nassert(2, 2)\na",
            "a = 1\nassert(2, 2)\nassert(a, 1)\na",
            "assert(1, 2)",
            "if false:\n    assert(1, 1)",
            "assert(",
        ] {
            let mut after = fixture("protected", source);
            after.case.expected = Some("1".to_string());
            assert_eq!(
                verdict_with_policy(&before, &[after], policy.clone()).decision,
                Decision::Rejected,
                "{policy:?}: {source}",
            );
        }
        for expected in [None, Some("2"), Some("not JSON")] {
            let mut after = fixture("protected", &before[0].case.source);
            after.case.expected = expected.map(str::to_string);
            assert_eq!(
                verdict_with_policy(&before, &[after], policy.clone()).decision,
                Decision::Rejected,
            );
        }
        for after in [vec![], vec![fixture("renamed", "assert(1, 1)")]] {
            assert_eq!(
                verdict_with_policy(&before, &after, policy.clone()).decision,
                Decision::Rejected,
            );
        }
    }
}

#[test]
fn skipped_new_cases_still_execute_observable_behavior_and_coverage() {
    let before = [fixture("paired", "assert(1, 1)")];
    for policy in skipped_policies() {
        for (source, expected) in [
            ("pass", None),
            ("assert(1, 2)", None),
            ("if false:\n    assert(1, 1)", None),
            ("assert(", None),
            ("2", Some("1")),
            ("2", Some("not JSON")),
        ] {
            let mut new = fixture("new", source);
            new.case.expected = expected.map(str::to_string);
            let report = verdict_with_policy(
                &before,
                &[fixture("paired", "assert(1, 1)"), new],
                policy.clone(),
            );
            assert_eq!(report.decision, Decision::Rejected, "{policy:?}: {source}");
            assert!(matches!(report.issues[0], Issue::InvalidCase { .. }));
        }
    }
}

#[test]
fn skipped_comparison_does_not_hide_broken_legacy_syntax_or_behavior() {
    let before = [fixture("paired", "a = 1\nassert(a, 1)")];
    let baseline = capture(FIXTURE_REVISION.to_string(), &before).unwrap();
    for policy in skipped_policies() {
        for source in [
            "assert(",
            "a = 2\nassert(a, 1)",
            "if false:\n    assert(1, 1)",
        ] {
            let mut proposal = fixture_proposal();
            proposal.category = policy.category;
            let report = compare(
                &baseline,
                proposal,
                policy.clone(),
                &[fixture("paired", source)],
                &before,
            )
            .unwrap();
            assert_eq!(report.decision, Decision::Rejected);
            assert!(matches!(report.issues[0], Issue::InvalidLegacyCase { .. }));
        }
    }
}

#[test]
fn corrupt_or_incomplete_baselines_cannot_be_successful_skips() {
    let before = [fixture("paired", "assert(1, 1)")];
    let baseline = capture(FIXTURE_REVISION.to_string(), &before).unwrap();
    let mut corrupt = Vec::new();
    let mut changed = baseline.clone();
    changed.metric = "unknown".to_string();
    corrupt.push(changed);
    for revision in [
        "",
        "not-a-revision",
        "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
    ] {
        let mut changed = baseline.clone();
        changed.revision = revision.to_string();
        corrupt.push(changed);
    }
    let mut changed = baseline.clone();
    changed.cases.clear();
    corrupt.push(changed);
    let mut changed = baseline.clone();
    changed.cases.push(changed.cases[0].clone());
    corrupt.push(changed);
    let mut changed = baseline.clone();
    changed.cases[0].tokens = 0;
    corrupt.push(changed);
    let mut changed = baseline.clone();
    changed.cases[0].expected = Some("not JSON".to_string());
    corrupt.push(changed);
    let mut changed = baseline.clone();
    changed.cases[0].id.title = "not the accepted case".to_string();
    corrupt.push(changed);
    for policy in skipped_policies() {
        for corrupt in &corrupt {
            let mut proposal = fixture_proposal();
            proposal.category = policy.category;
            assert!(compare(corrupt, proposal, policy.clone(), &before, &before).is_err());
        }
        let mut altered = baseline.clone();
        altered.cases[0].assertions = vec!["assert(2, 2)".to_string()];
        let mut proposal = fixture_proposal();
        proposal.category = policy.category;
        let report = compare(&altered, proposal, policy, &before, &before).unwrap();
        assert_eq!(report.decision, Decision::Rejected);
        assert!(matches!(
            report.issues[0],
            Issue::ChangedLegacyAssertions { .. }
        ));
    }
    for json in [
        "{}",
        r#"{"metric":"nucleoid-lexer-with-layout-v1","revision":"abc","cases":null}"#,
        r#"{"metric":"nucleoid-lexer-with-layout-v1","revision":"abc","cases":[],"ignore":true}"#,
    ] {
        assert!(serde_json::from_str::<Baseline>(json).is_err());
    }
}

#[test]
fn skipped_comparison_still_requires_every_accepted_case_and_extra_source() {
    let before = [
        fixture("original", "assert(1, 1)"),
        fixture("extra", "assert(2, 2)"),
    ];
    let baseline = capture(FIXTURE_REVISION.to_string(), &before).unwrap();
    for policy in skipped_policies() {
        let mut incomplete = baseline.clone();
        incomplete.cases.pop();
        for (baseline, legacy) in [(&incomplete, before.as_slice()), (&baseline, &before[..1])] {
            let mut proposal = fixture_proposal();
            proposal.category = policy.category;
            assert!(compare(baseline, proposal, policy.clone(), legacy, &before).is_err());
        }
        let mut proposal = fixture_proposal();
        proposal.category = policy.category;
        assert!(
            compare(
                &baseline,
                proposal,
                policy,
                &before,
                &[
                    fixture("original", "assert(1, 1)"),
                    fixture("original", "assert(1, 1)")
                ],
            )
            .is_err()
        );
    }
}
