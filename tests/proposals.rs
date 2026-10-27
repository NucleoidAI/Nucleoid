//! Deterministic NUC checks with optional, independently evidenced advisory scores.
//!
//! The ignored entry points record an immutable baseline and compare a rewritten
//! corpus. Ordinary tests supply policy inputs directly, without mutating the
//! process environment.

mod common;
#[path = "proposals/strategy.rs"]
mod strategy;

use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use indexmap::IndexMap;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

const METRIC: &str = "nucleoid-lexer-with-layout-v1";
const SCORE_METRIC: &str = "nuc-advisory-usefulness-evaluator-time-v1";
const SCORE_FORMULA: &str = "useful: 1 - 0.1 * t / (t + 1), t in seconds; neutral: 0; rejected: -1";
const TIMING_SCOPE: &str = "elapsed deterministic evaluator work: input validation, source loading, \
    legacy/candidate checks, and blob stability verification; excludes compilation, external \
    suites/Clippy, proposal/model time, score calculation, and report writing";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExecutionMode {
    Continuous,
    Single,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Decision {
    Accepted,
    Rejected,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TokenChange {
    Less,
    Same,
    More,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Comparison {
    id: CaseId,
    before: usize,
    after: usize,
    change: TokenChange,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum TokenCheck {
    Ran {
        metric: String,
        cases: Vec<Comparison>,
    },
    Skipped {
        reason: TokenSkipReason,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BehaviorChecks {
    legacy_cases_passed: usize,
    candidate_cases_passed: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UsefulnessRubric {
    criterion_id: String,
    proposal_path: String,
    baseline_revision: String,
    criterion: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AssessorRole {
    IndependentEvaluator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Assessor {
    role: AssessorRole,
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum IndependentCheck {
    Passed { evidence: String },
    Failed { evidence: String },
    Unresolved { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndependentChecks {
    behavior_integrity: IndependentCheck,
    syntax_compatibility: IndependentCheck,
    development_suites: IndependentCheck,
    formatting: IndependentCheck,
    clippy: IndependentCheck,
    export_consistency: IndependentCheck,
}

impl IndependentChecks {
    fn entries(&self) -> [(&'static str, &IndependentCheck); 6] {
        [
            ("behavior_integrity", &self.behavior_integrity),
            ("syntax_compatibility", &self.syntax_compatibility),
            ("development_suites", &self.development_suites),
            ("formatting", &self.formatting),
            ("clippy", &self.clippy),
            ("export_consistency", &self.export_consistency),
        ]
    }

    fn eligibility(&self) -> IndependentEligibility {
        let checks = self.entries();
        if checks
            .iter()
            .any(|(_, check)| matches!(check, IndependentCheck::Failed { .. }))
        {
            IndependentEligibility::Failed
        } else if checks
            .iter()
            .any(|(_, check)| matches!(check, IndependentCheck::Unresolved { .. }))
        {
            IndependentEligibility::Unresolved
        } else {
            IndependentEligibility::Passed
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum UsefulnessJudgment {
    Demonstrated { evidence: String },
    NoDemonstratedBenefit { evidence: String },
    Unknown { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UsefulnessAssessment {
    criterion_id: String,
    rubric_revision: String,
    proposal_path: String,
    baseline_revision: String,
    proposal_revision: String,
    assessor: Assessor,
    checks: IndependentChecks,
    usefulness: UsefulnessJudgment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScoringArtifact<T> {
    path: PathBuf,
    revision: String,
    input: T,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScoringEvidence {
    rubric: Option<ScoringArtifact<UsefulnessRubric>>,
    assessment: Option<ScoringArtifact<UsefulnessAssessment>>,
}

#[derive(Default)]
struct ScoringPaths<'a> {
    rubric: Option<&'a Path>,
    rubric_revision: Option<&'a str>,
    assessment: Option<&'a Path>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum IndependentEligibility {
    NotAssessed,
    Passed,
    Failed,
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RewardBasis {
    GateRejected,
    IndependentChecksFailed,
    DemonstratedUsefulness,
    NoDemonstratedBenefit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum UnscoredReason {
    Blocked,
    MissingRubric,
    MissingAssessment,
    IndependentChecksUnresolved,
    UnknownUsefulness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum AdvisoryReward {
    Scored {
        basis: RewardBasis,
        base_reward: i8,
        applied_cost_adjustment: f64,
        final_reward: f64,
    },
    Unscored {
        reason: UnscoredReason,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum ReportUsefulness {
    Demonstrated,
    NoDemonstratedBenefit,
    Unscored { reason: UnscoredReason },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvaluationTime {
    elapsed_seconds: f64,
    scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdvisoryScore {
    metric: String,
    formula: String,
    evaluation_time: EvaluationTime,
    independent_check_eligibility: IndependentEligibility,
    usefulness: ReportUsefulness,
    usefulness_basis: ScoringEvidence,
    reward: AdvisoryReward,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "strategy::SavedEvaluation")]
struct EvaluationReport {
    #[serde(flatten)]
    gate: Report,
    advisory_score: AdvisoryScore,
    strategy_learning: strategy::Attribution,
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
    #[error("invalid advisory scoring input: {0}")]
    Scoring(String),
    #[error("invalid proposal strategy input: {0}")]
    Strategy(String),
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

fn full_git_revision(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn scoring_text(field: &str, value: &str) -> Result<(), GateError> {
    if value.trim().is_empty() {
        return Err(GateError::Scoring(format!("{field} must be nonblank")));
    }
    Ok(())
}

fn scoring_binding(field: &str, actual: &str, expected: &str) -> Result<(), GateError> {
    if actual != expected {
        return Err(GateError::Scoring(format!(
            "{field} mismatch: expected {expected:?}, found {actual:?}"
        )));
    }
    Ok(())
}

fn scoring_blob(root: &Path, path: &Path) -> Result<String, GateError> {
    let path = path.to_str().ok_or_else(|| {
        GateError::Scoring("scoring artifact paths must be valid Unicode".to_string())
    })?;
    // Rust canonicalizes Windows paths to verbatim paths, which Git cannot open.
    let git_path = match path.strip_prefix(r"\\?\UNC\") {
        Some(unc) => format!(r"\\{unc}"),
        None => path.strip_prefix(r"\\?\").unwrap_or(path).to_string(),
    };
    Ok(
        git(root, &["hash-object", "--no-filters", "--", &git_path])?
            .trim()
            .to_string(),
    )
}

impl<T> ScoringArtifact<T> {
    fn verify_stable(&self, root: &Path) -> Result<(), GateError> {
        scoring_binding(
            "scoring artifact blob (input changed during evaluation)",
            &scoring_blob(root, &self.path)?,
            &self.revision,
        )
    }
}

fn read_scoring_artifact<T: DeserializeOwned>(
    root: &Path,
    path: &Path,
) -> Result<ScoringArtifact<T>, GateError> {
    if !path.is_absolute() || path.extension().is_none_or(|extension| extension != "json") {
        return Err(GateError::Scoring(
            "use an absolute session path to a JSON scoring artifact".to_string(),
        ));
    }
    let canonical = fs::canonicalize(path).map_err(|source| GateError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let canonical_root = fs::canonicalize(root).map_err(|source| GateError::Io {
        path: root.to_path_buf(),
        source,
    })?;
    if canonical.starts_with(canonical_root) {
        return Err(GateError::Scoring(
            "scoring inputs belong outside the candidate repository".to_string(),
        ));
    }
    if canonical
        .extension()
        .is_none_or(|extension| extension != "json")
    {
        return Err(GateError::Scoring(
            "the resolved scoring artifact must also be a JSON path".to_string(),
        ));
    }
    let revision = scoring_blob(root, &canonical)?;
    let artifact = ScoringArtifact {
        input: serde_json::from_str(&read(&canonical)?)?,
        path: canonical,
        revision,
    };
    artifact.verify_stable(root)?;
    Ok(artifact)
}

impl ScoringEvidence {
    fn load(
        root: &Path,
        paths: ScoringPaths<'_>,
        baseline_revision: &str,
        proposal: &Proposal,
    ) -> Result<Self, GateError> {
        let (rubric_path, rubric_revision) = match (paths.rubric, paths.rubric_revision) {
            (None, None) if paths.assessment.is_none() => return Ok(Self::default()),
            (Some(path), Some(revision)) if full_git_revision(revision) => (path, revision),
            _ => {
                return Err(GateError::Scoring(
                    "scoring requires NUC_PROPOSAL_RUBRIC and its caller-frozen full Git blob \
                     NUC_PROPOSAL_RUBRIC_REVISION; an assessment cannot stand alone"
                        .to_string(),
                ));
            }
        };
        let rubric = read_scoring_artifact(root, rubric_path)?;
        scoring_binding(
            "caller-frozen rubric revision",
            &rubric.revision,
            rubric_revision,
        )?;
        let evidence = Self {
            rubric: Some(rubric),
            assessment: paths
                .assessment
                .map(|path| read_scoring_artifact(root, path))
                .transpose()?,
        };
        evidence.validate(baseline_revision, proposal, &[])?;
        Ok(evidence)
    }

    fn validate(
        &self,
        baseline_revision: &str,
        proposal: &Proposal,
        issues: &[Issue],
    ) -> Result<(), GateError> {
        let Some(rubric) = &self.rubric else {
            if self.assessment.is_some() {
                return Err(GateError::Scoring(
                    "an assessment requires a caller-frozen rubric".to_string(),
                ));
            }
            return Ok(());
        };
        if !full_git_revision(&rubric.revision) {
            return Err(GateError::Scoring(
                "rubric revision must be a full Git blob ID".to_string(),
            ));
        }
        let criterion = &rubric.input;
        scoring_text("rubric.criterion_id", &criterion.criterion_id)?;
        scoring_text("rubric.criterion", &criterion.criterion)?;
        scoring_binding(
            "rubric.baseline_revision",
            &criterion.baseline_revision,
            baseline_revision,
        )?;
        scoring_binding(
            "rubric.proposal_path",
            &criterion.proposal_path.replace('\\', "/"),
            &proposal.path.replace('\\', "/"),
        )?;
        let Some(artifact) = &self.assessment else {
            return Ok(());
        };
        let assessment = &artifact.input;
        scoring_binding(
            "assessment.criterion_id",
            &assessment.criterion_id,
            &criterion.criterion_id,
        )?;
        scoring_binding(
            "assessment.rubric_revision",
            &assessment.rubric_revision,
            &rubric.revision,
        )?;
        scoring_binding(
            "assessment.baseline_revision",
            &assessment.baseline_revision,
            baseline_revision,
        )?;
        scoring_binding(
            "assessment.proposal_path",
            &assessment.proposal_path.replace('\\', "/"),
            &proposal.path.replace('\\', "/"),
        )?;
        if !full_git_revision(&assessment.proposal_revision) {
            return Err(GateError::Scoring(
                "assessment.proposal_revision must be the full handed-off Git blob ID".to_string(),
            ));
        }
        scoring_binding(
            "assessment.proposal_revision",
            &assessment.proposal_revision,
            &proposal.revision,
        )?;
        scoring_text("assessment.assessor.id", &assessment.assessor.id)?;
        for (name, check) in assessment.checks.entries() {
            let value = match check {
                IndependentCheck::Passed { evidence } | IndependentCheck::Failed { evidence } => {
                    evidence
                }
                IndependentCheck::Unresolved { reason } => reason,
            };
            scoring_text(&format!("assessment.checks.{name} evidence/reason"), value)?;
        }
        let value = match &assessment.usefulness {
            UsefulnessJudgment::Demonstrated { evidence }
            | UsefulnessJudgment::NoDemonstratedBenefit { evidence } => evidence,
            UsefulnessJudgment::Unknown { reason } => reason,
        };
        scoring_text("assessment.usefulness evidence/reason", value)?;
        for issue in issues {
            let (name, check) = match issue {
                Issue::MissingCase { .. }
                | Issue::ChangedAssertions { .. }
                | Issue::ChangedReturn { .. }
                | Issue::InvalidCase { .. } => {
                    ("behavior_integrity", &assessment.checks.behavior_integrity)
                }
                Issue::InvalidLegacyCase { .. } | Issue::ChangedLegacyAssertions { .. } => (
                    "syntax_compatibility",
                    &assessment.checks.syntax_compatibility,
                ),
                Issue::MoreTokens { .. } | Issue::UnpairedCase { .. } => continue,
            };
            if matches!(check, IndependentCheck::Passed { .. }) {
                return Err(GateError::Scoring(format!(
                    "assessment.checks.{name} claims passed but contradicts gate issue {issue:?}"
                )));
            }
        }
        Ok(())
    }

    fn verify_stable(&self, root: &Path) -> Result<(), GateError> {
        if let Some(rubric) = &self.rubric {
            rubric.verify_stable(root)?;
        }
        if let Some(assessment) = &self.assessment {
            assessment.verify_stable(root)?;
        }
        Ok(())
    }
}

fn optional_scoring_path(variable: &'static str) -> Result<Option<PathBuf>, GateError> {
    match env::var_os(variable) {
        None => Ok(None),
        Some(value) if !value.is_empty() => Ok(Some(PathBuf::from(value))),
        Some(_) => Err(GateError::Scoring(format!(
            "{variable} was supplied but is empty; unset absent inputs"
        ))),
    }
}

fn read_scoring_evidence(
    root: &Path,
    baseline_revision: &str,
    proposal: &Proposal,
) -> Result<ScoringEvidence, GateError> {
    let rubric = optional_scoring_path("NUC_PROPOSAL_RUBRIC")?;
    let rubric_revision = optional_environment("NUC_PROPOSAL_RUBRIC_REVISION")?;
    let assessment = optional_scoring_path("NUC_PROPOSAL_ASSESSMENT")?;
    ScoringEvidence::load(
        root,
        ScoringPaths {
            rubric: rubric.as_deref(),
            rubric_revision: rubric_revision.as_deref(),
            assessment: assessment.as_deref(),
        },
        baseline_revision,
        proposal,
    )
}

fn advisory_score(report: &Report, evidence: ScoringEvidence, elapsed: Duration) -> AdvisoryScore {
    advisory_score_at_seconds(report, evidence, elapsed.as_secs_f64())
}

fn advisory_score_at_seconds(
    report: &Report,
    evidence: ScoringEvidence,
    seconds: f64,
) -> AdvisoryScore {
    let assessment = evidence.assessment.as_ref().map(|artifact| &artifact.input);
    let eligibility = assessment.map_or(IndependentEligibility::NotAssessed, |assessment| {
        assessment.checks.eligibility()
    });
    let usefulness = match assessment {
        Some(assessment) => match assessment.usefulness {
            UsefulnessJudgment::Demonstrated { .. } => ReportUsefulness::Demonstrated,
            UsefulnessJudgment::NoDemonstratedBenefit { .. } => {
                ReportUsefulness::NoDemonstratedBenefit
            }
            UsefulnessJudgment::Unknown { .. } => ReportUsefulness::Unscored {
                reason: UnscoredReason::UnknownUsefulness,
            },
        },
        None => ReportUsefulness::Unscored {
            reason: if evidence.rubric.is_some() {
                UnscoredReason::MissingAssessment
            } else {
                UnscoredReason::MissingRubric
            },
        },
    };
    let result = match report.decision {
        Decision::Blocked => Err(UnscoredReason::Blocked),
        Decision::Rejected => Ok((RewardBasis::GateRejected, -1)),
        Decision::Accepted => match (eligibility, assessment) {
            (IndependentEligibility::Failed, _) => Ok((RewardBasis::IndependentChecksFailed, -1)),
            (IndependentEligibility::Unresolved, _) => {
                Err(UnscoredReason::IndependentChecksUnresolved)
            }
            (_, None) if evidence.rubric.is_none() => Err(UnscoredReason::MissingRubric),
            (_, None) => Err(UnscoredReason::MissingAssessment),
            (_, Some(assessment)) => match assessment.usefulness {
                UsefulnessJudgment::Demonstrated { .. } => {
                    Ok((RewardBasis::DemonstratedUsefulness, 1))
                }
                UsefulnessJudgment::NoDemonstratedBenefit { .. } => {
                    Ok((RewardBasis::NoDemonstratedBenefit, 0))
                }
                UsefulnessJudgment::Unknown { .. } => Err(UnscoredReason::UnknownUsefulness),
            },
        },
    };
    let reward = match result {
        Ok((basis, base_reward)) => {
            let applied_cost_adjustment = if base_reward == 1 {
                -0.1 * (seconds / (seconds + 1.0))
            } else {
                0.0
            };
            AdvisoryReward::Scored {
                basis,
                base_reward,
                applied_cost_adjustment,
                final_reward: f64::from(base_reward) + applied_cost_adjustment,
            }
        }
        Err(reason) => AdvisoryReward::Unscored { reason },
    };
    AdvisoryScore {
        metric: SCORE_METRIC.to_string(),
        formula: SCORE_FORMULA.to_string(),
        evaluation_time: EvaluationTime {
            elapsed_seconds: seconds,
            scope: TIMING_SCOPE.to_string(),
        },
        independent_check_eligibility: eligibility,
        usefulness,
        usefulness_basis: evidence,
        reward,
    }
}

fn time_evaluation(
    work: impl FnOnce() -> Result<(Report, ScoringEvidence, strategy::Attribution), GateError>,
) -> Result<EvaluationReport, GateError> {
    let started = Instant::now();
    let (gate, evidence, strategy_learning) = work()?;
    evidence.validate(&gate.baseline_revision, &gate.proposal, &gate.issues)?;
    strategy_learning.validate_binding(&gate)?;
    let elapsed = started.elapsed();
    Ok(EvaluationReport {
        advisory_score: advisory_score(&gate, evidence, elapsed),
        gate,
        strategy_learning,
    })
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
    let evaluation = time_evaluation(|| {
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
        let scoring = read_scoring_evidence(root, &baseline.revision, &proposal)?;
        let strategy_learning =
            strategy::read_attribution(root, &baseline.revision, &proposal, &policy)?;
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
        scoring.verify_stable(root)?;
        strategy_learning.verify_stable(root)?;
        Ok((report, scoring, strategy_learning))
    })?;
    let report = &evaluation.gate;
    let path = artifact("NUC_PROPOSAL_REPORT")?;
    write_new(&path, &evaluation)?;
    println!("Policy: {}", serde_json::to_string(&report.policy)?);
    println!(
        "Advisory score: {}",
        serde_json::to_string(&evaluation.advisory_score)?
    );
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

#[test]
#[ignore = "caller only: select and pin a strategy receipt BEFORE drafting"]
fn select_proposal_strategy() -> Result<(), GateError> {
    strategy::select_command()
}

#[test]
#[ignore = "independent evaluator only: learn from an existing pinned attributed report"]
fn learn_proposal_strategy() -> Result<(), GateError> {
    strategy::learn_command()
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
const FIXTURE_PROPOSAL_REVISION: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const FIXTURE_RUBRIC_REVISION: &str = "cccccccccccccccccccccccccccccccccccccccc";

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
        revision: FIXTURE_PROPOSAL_REVISION.to_string(),
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

fn fixture_scoring_evidence() -> ScoringEvidence {
    let passed = || IndependentCheck::Passed {
        evidence: "independent regression fixture command and passing result".to_string(),
    };
    ScoringEvidence {
        rubric: Some(ScoringArtifact {
            path: env::temp_dir().join("regression-rubric.json"),
            revision: FIXTURE_RUBRIC_REVISION.to_string(),
            input: UsefulnessRubric {
                criterion_id: "observable-benefit".to_string(),
                proposal_path: "docs\\nuc-9999.md".to_string(),
                baseline_revision: FIXTURE_REVISION.to_string(),
                criterion: "demonstrate the caller's specified observable benefit".to_string(),
            },
        }),
        assessment: Some(ScoringArtifact {
            path: env::temp_dir().join("regression-assessment.json"),
            revision: FIXTURE_PROPOSAL_REVISION.to_string(),
            input: UsefulnessAssessment {
                criterion_id: "observable-benefit".to_string(),
                rubric_revision: FIXTURE_RUBRIC_REVISION.to_string(),
                proposal_path: "docs/nuc-9999.md".to_string(),
                baseline_revision: FIXTURE_REVISION.to_string(),
                proposal_revision: FIXTURE_PROPOSAL_REVISION.to_string(),
                assessor: Assessor {
                    role: AssessorRole::IndependentEvaluator,
                    id: "independent-regression-assessor".to_string(),
                },
                checks: IndependentChecks {
                    behavior_integrity: passed(),
                    syntax_compatibility: passed(),
                    development_suites: passed(),
                    formatting: passed(),
                    clippy: passed(),
                    export_consistency: passed(),
                },
                usefulness: UsefulnessJudgment::Demonstrated {
                    evidence: "independent regression fixture observation against the criterion"
                        .to_string(),
                },
            },
        }),
    }
}

fn fixture_scored_report() -> Report {
    let cases = [fixture("same", "a = 1\nassert(a, 1)")];
    verdict(&cases, &cases)
}

fn fixture_score(report: &Report, evidence: ScoringEvidence, elapsed: Duration) -> AdvisoryScore {
    evidence
        .validate(&report.baseline_revision, &report.proposal, &report.issues)
        .unwrap();
    advisory_score(report, evidence, elapsed)
}

fn scored_values(score: &AdvisoryScore) -> (RewardBasis, i8, f64, f64) {
    match score.reward {
        AdvisoryReward::Scored {
            basis,
            base_reward,
            applied_cost_adjustment,
            final_reward,
        } => (basis, base_reward, applied_cost_adjustment, final_reward),
        AdvisoryReward::Unscored { reason } => panic!("expected a scored reward, not {reason:?}"),
    }
}

fn check_slots(checks: &mut IndependentChecks) -> [&mut IndependentCheck; 6] {
    [
        &mut checks.behavior_integrity,
        &mut checks.syntax_compatibility,
        &mut checks.development_suites,
        &mut checks.formatting,
        &mut checks.clippy,
        &mut checks.export_consistency,
    ]
}

#[test]
fn independently_demonstrated_usefulness_and_no_benefit_are_distinct() {
    let report = fixture_scored_report();
    let useful = fixture_score(&report, fixture_scoring_evidence(), Duration::from_secs(1));
    assert_eq!(
        scored_values(&useful),
        (RewardBasis::DemonstratedUsefulness, 1, -0.05, 0.95)
    );
    assert_eq!(
        useful.independent_check_eligibility,
        IndependentEligibility::Passed
    );
    let mut neutral = fixture_scoring_evidence();
    neutral.assessment.as_mut().unwrap().input.usefulness =
        UsefulnessJudgment::NoDemonstratedBenefit {
            evidence: "independent fixture comparison found no demonstrated benefit".to_string(),
        };
    assert_eq!(
        scored_values(&fixture_score(&report, neutral, Duration::MAX)),
        (RewardBasis::NoDemonstratedBenefit, 0, 0.0, 0.0)
    );
    assert_eq!(report.decision, Decision::Accepted);
}

#[test]
fn missing_and_unknown_usefulness_are_explicitly_unscored_not_zero() {
    let report = fixture_scored_report();
    let mut rubric_only = fixture_scoring_evidence();
    rubric_only.assessment = None;
    let mut unknown = fixture_scoring_evidence();
    unknown.assessment.as_mut().unwrap().input.usefulness = UsefulnessJudgment::Unknown {
        reason: "independent fixture observation could not establish usefulness".to_string(),
    };
    for (evidence, expected) in [
        (ScoringEvidence::default(), UnscoredReason::MissingRubric),
        (rubric_only, UnscoredReason::MissingAssessment),
        (unknown, UnscoredReason::UnknownUsefulness),
    ] {
        let score = fixture_score(&report, evidence, Duration::from_secs(2));
        assert!(matches!(score.reward, AdvisoryReward::Unscored { reason } if reason == expected));
        let json = serde_json::to_value(score).unwrap();
        assert_eq!(json["reward"]["status"], "unscored");
        assert_eq!(json["usefulness"]["status"], "unscored");
        for field in ["base_reward", "applied_cost_adjustment", "final_reward"] {
            assert!(json["reward"].get(field).is_none());
        }
        assert_eq!(json["evaluation_time"]["elapsed_seconds"], 2.0);
    }
}

#[test]
fn genuine_rejection_wins_over_claimed_usefulness_and_blocked_stays_unscored() {
    let rejected = verdict(
        &[fixture("more", "a = 1\nassert(a, 1)")],
        &[fixture("more", "a = (1)\nassert(a, 1)")],
    );
    assert_eq!(rejected.decision, Decision::Rejected);
    for evidence in [fixture_scoring_evidence(), ScoringEvidence::default()] {
        assert_eq!(
            scored_values(&fixture_score(&rejected, evidence, Duration::from_secs(1))),
            (RewardBasis::GateRejected, -1, 0.0, -1.0)
        );
    }
    let without_evidence = fixture_score(&rejected, ScoringEvidence::default(), Duration::ZERO);
    assert!(matches!(
        without_evidence.usefulness,
        ReportUsefulness::Unscored {
            reason: UnscoredReason::MissingRubric
        }
    ));
    let blocked = verdict(
        &[fixture("original", "assert(1, 1)")],
        &[
            fixture("original", "assert(1, 1)"),
            fixture("new", "assert(2, 2)"),
        ],
    );
    assert_eq!(blocked.decision, Decision::Blocked);
    let score = fixture_score(&blocked, fixture_scoring_evidence(), Duration::from_secs(1));
    assert!(matches!(
        score.reward,
        AdvisoryReward::Unscored {
            reason: UnscoredReason::Blocked
        }
    ));
    assert_eq!(score.evaluation_time.elapsed_seconds, 1.0);
}

#[test]
fn every_independent_check_requires_evidenced_success_before_positive_reward() {
    let report = fixture_scored_report();
    for slot in 0..6 {
        for failed in [false, true] {
            let mut evidence = fixture_scoring_evidence();
            let checks = &mut evidence.assessment.as_mut().unwrap().input.checks;
            *check_slots(checks).into_iter().nth(slot).unwrap() = if failed {
                IndependentCheck::Failed {
                    evidence: "independent fixture command failed with an observed diagnostic"
                        .to_string(),
                }
            } else {
                IndependentCheck::Unresolved {
                    reason: "independent fixture command has not completed".to_string(),
                }
            };
            let score = fixture_score(&report, evidence, Duration::ZERO);
            if failed {
                assert_eq!(
                    scored_values(&score),
                    (RewardBasis::IndependentChecksFailed, -1, 0.0, -1.0)
                );
                assert_eq!(
                    score.independent_check_eligibility,
                    IndependentEligibility::Failed
                );
            } else {
                assert!(matches!(
                    score.reward,
                    AdvisoryReward::Unscored {
                        reason: UnscoredReason::IndependentChecksUnresolved
                    }
                ));
            }
            assert_eq!(report.decision, Decision::Accepted);
        }
    }
    let mut evidence = fixture_scoring_evidence();
    let assessment = &mut evidence.assessment.as_mut().unwrap().input;
    assessment.checks.clippy = IndependentCheck::Failed {
        evidence: "independent fixture Clippy failed".to_string(),
    };
    assessment.checks.development_suites = IndependentCheck::Unresolved {
        reason: "independent fixture suite remains unresolved".to_string(),
    };
    assessment.usefulness = UsefulnessJudgment::Unknown {
        reason: "usefulness is unknown; a known failure must remain visible".to_string(),
    };
    assert_eq!(
        scored_values(&fixture_score(&report, evidence, Duration::ZERO)).0,
        RewardBasis::IndependentChecksFailed
    );
}

#[test]
fn usefulness_cost_discount_is_bounded_and_only_applied_to_positive_reward() {
    let report = fixture_scored_report();
    let mut last_reward = 1.0;
    for elapsed in [
        Duration::ZERO,
        Duration::from_nanos(1),
        Duration::from_millis(500),
        Duration::from_secs(1),
        Duration::from_secs(100),
        Duration::MAX,
    ] {
        let score = fixture_score(&report, fixture_scoring_evidence(), elapsed);
        let (_, base, adjustment, reward) = scored_values(&score);
        assert_eq!(base, 1);
        assert!(reward.is_finite() && (0.9..=1.0).contains(&reward));
        assert!((-0.1..=0.0).contains(&adjustment));
        assert!(reward <= last_reward);
        assert_eq!(reward, 1.0 + adjustment);
        assert_eq!(score.evaluation_time.elapsed_seconds, elapsed.as_secs_f64());
        last_reward = reward;
    }
    assert_eq!(
        scored_values(&fixture_score(
            &report,
            fixture_scoring_evidence(),
            Duration::ZERO
        ))
        .3,
        1.0
    );
}

#[test]
fn advisory_scores_preserve_the_complete_verdict_and_token_policy_matrix() {
    let before = [fixture("paired", "a = 1\nassert(a, 1)")];
    let after = [fixture("paired", "a = (1)\nassert(a, 1)")];
    for (policy, skip) in policy_matrix() {
        let gate = verdict_with_policy(&before, &after, policy);
        let original = serde_json::to_value(&gate).unwrap();
        let score = fixture_score(&gate, fixture_scoring_evidence(), Duration::ZERO);
        assert_eq!(scored_values(&score).1, if skip.is_some() { 1 } else { -1 });
        let evaluation = EvaluationReport {
            gate,
            advisory_score: score,
            strategy_learning: strategy::Attribution::NotConfigured,
        };
        let mut json = serde_json::to_value(evaluation).unwrap();
        json.as_object_mut().unwrap().remove("advisory_score");
        json.as_object_mut().unwrap().remove("strategy_learning");
        assert_eq!(json, original);
    }
}

#[test]
fn scoring_provenance_mismatches_and_blank_judgments_are_errors() {
    let report = fixture_scored_report();
    let corruptions: &[fn(&mut ScoringEvidence)] = &[
        |e| e.rubric.as_mut().unwrap().input.criterion_id = " \n ".to_string(),
        |e| e.rubric.as_mut().unwrap().input.criterion.clear(),
        |e| e.rubric.as_mut().unwrap().revision = "not-a-blob".to_string(),
        |e| e.rubric.as_mut().unwrap().input.proposal_path = "docs/nuc-0001.md".to_string(),
        |e| {
            e.rubric.as_mut().unwrap().input.baseline_revision =
                FIXTURE_PROPOSAL_REVISION.to_string()
        },
        |e| e.assessment.as_mut().unwrap().input.criterion_id = "another-criterion".to_string(),
        |e| e.assessment.as_mut().unwrap().input.rubric_revision = FIXTURE_REVISION.to_string(),
        |e| {
            e.assessment.as_mut().unwrap().input.baseline_revision =
                FIXTURE_RUBRIC_REVISION.to_string()
        },
        |e| e.assessment.as_mut().unwrap().input.proposal_path = "docs/nuc-0001.md".to_string(),
        |e| {
            e.assessment.as_mut().unwrap().input.proposal_revision =
                FIXTURE_RUBRIC_REVISION.to_string()
        },
        |e| e.assessment.as_mut().unwrap().input.proposal_revision = "short".to_string(),
        |e| e.assessment.as_mut().unwrap().input.assessor.id = " \t ".to_string(),
        |e| {
            e.assessment.as_mut().unwrap().input.usefulness = UsefulnessJudgment::Demonstrated {
                evidence: " ".to_string(),
            };
        },
        |e| {
            e.assessment.as_mut().unwrap().input.usefulness =
                UsefulnessJudgment::NoDemonstratedBenefit {
                    evidence: String::new(),
                };
        },
        |e| {
            e.assessment.as_mut().unwrap().input.usefulness = UsefulnessJudgment::Unknown {
                reason: " ".to_string(),
            };
        },
        |e| e.rubric = None,
    ];
    for corrupt in corruptions {
        let mut evidence = fixture_scoring_evidence();
        corrupt(&mut evidence);
        assert!(matches!(
            evidence.validate(&report.baseline_revision, &report.proposal, &report.issues),
            Err(GateError::Scoring(_))
        ));
    }
    for slot in 0..6 {
        for check in [
            IndependentCheck::Passed {
                evidence: " ".to_string(),
            },
            IndependentCheck::Failed {
                evidence: String::new(),
            },
            IndependentCheck::Unresolved {
                reason: "\n".to_string(),
            },
        ] {
            let mut evidence = fixture_scoring_evidence();
            *check_slots(&mut evidence.assessment.as_mut().unwrap().input.checks)
                .into_iter()
                .nth(slot)
                .unwrap() = check;
            assert!(matches!(
                evidence.validate(&report.baseline_revision, &report.proposal, &[]),
                Err(GateError::Scoring(_))
            ));
        }
    }
}

#[test]
fn scoring_json_rejects_malformed_unknown_and_contradictory_fields() {
    let evidence = fixture_scoring_evidence();
    let rubric = serde_json::to_value(&evidence.rubric.unwrap().input).unwrap();
    for field in [
        "criterion_id",
        "criterion",
        "baseline_revision",
        "proposal_path",
    ] {
        let mut missing = rubric.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<UsefulnessRubric>(missing).is_err());
    }
    let mut extra = rubric;
    extra["reward"] = 1.into();
    assert!(serde_json::from_value::<UsefulnessRubric>(extra).is_err());
    assert!(serde_json::from_str::<UsefulnessRubric>("{").is_err());
    assert!(
        serde_json::from_str::<UsefulnessRubric>(
            r#"{"criterion_id":"a","criterion_id":"b","criterion":"x","proposal_path":"x","baseline_revision":"x"}"#
        )
        .is_err()
    );
    let assessment = serde_json::to_value(&evidence.assessment.unwrap().input).unwrap();
    let corruptions: &[fn(&mut serde_json::Value)] = &[
        |a| a["assessor"]["role"] = "proposer".into(),
        |a| a["assessor"]["trusted"] = true.into(),
        |a| a["checks"]["clippy"]["status"] = "waived".into(),
        |a| a["checks"]["clippy"]["reason"] = "contradicts passed evidence".into(),
        |a| {
            a["checks"]["clippy"]
                .as_object_mut()
                .unwrap()
                .remove("evidence");
        },
        |a| {
            a["checks"]
                .as_object_mut()
                .unwrap()
                .remove("syntax_compatibility");
        },
        |a| a["usefulness"]["status"] = "accepted".into(),
        |a| a["usefulness"]["reason"] = "contradicts a demonstrated judgment".into(),
        |a| {
            a["usefulness"].as_object_mut().unwrap().remove("evidence");
        },
        |a| a["elapsed_seconds"] = 0.into(),
        |a| {
            a.as_object_mut().unwrap().remove("proposal_revision");
        },
        |a| a["rubric_revision"] = serde_json::Value::Null,
    ];
    for corrupt in corruptions {
        let mut input = assessment.clone();
        corrupt(&mut input);
        assert!(serde_json::from_value::<UsefulnessAssessment>(input).is_err());
    }
}

#[test]
fn a_passed_assessment_cannot_contradict_known_gate_integrity_or_legacy_failure() {
    let before = [fixture("paired", "a = 1\nassert(a, 1)")];
    let baseline = capture(FIXTURE_REVISION.to_string(), &before).unwrap();
    let integrity = verdict(&before, &[fixture("paired", "a = 1\nassert(1, 1)")]);
    let legacy = compare(
        &baseline,
        fixture_proposal(),
        required_policy(),
        &[fixture("paired", "assert(")],
        &before,
    )
    .unwrap();
    for (gate, legacy_failure) in [(integrity, false), (legacy, true)] {
        assert_eq!(gate.decision, Decision::Rejected);
        let mut evidence = fixture_scoring_evidence();
        assert!(matches!(
            evidence.validate(&gate.baseline_revision, &gate.proposal, &gate.issues),
            Err(GateError::Scoring(_))
        ));
        let checks = &mut evidence.assessment.as_mut().unwrap().input.checks;
        let check = if legacy_failure {
            &mut checks.syntax_compatibility
        } else {
            &mut checks.behavior_integrity
        };
        *check = IndependentCheck::Failed {
            evidence: "independent fixture reproduced the gate failure".to_string(),
        };
        assert_eq!(
            scored_values(&fixture_score(&gate, evidence, Duration::ZERO)).1,
            -1
        );
    }
}

#[test]
fn scoring_paths_require_a_frozen_pin_and_never_silently_drop_supplied_inputs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = Path::new("not-created-scoring-input.json");
    for paths in [
        ScoringPaths {
            rubric: Some(path),
            ..ScoringPaths::default()
        },
        ScoringPaths {
            rubric_revision: Some(FIXTURE_RUBRIC_REVISION),
            ..ScoringPaths::default()
        },
        ScoringPaths {
            assessment: Some(path),
            ..ScoringPaths::default()
        },
        ScoringPaths {
            rubric: Some(path),
            rubric_revision: Some(""),
            assessment: None,
        },
    ] {
        assert!(matches!(
            ScoringEvidence::load(root, paths, FIXTURE_REVISION, &fixture_proposal()),
            Err(GateError::Scoring(_))
        ));
    }
    assert!(matches!(
        ScoringEvidence::load(
            root,
            ScoringPaths {
                rubric: Some(path),
                rubric_revision: Some(FIXTURE_RUBRIC_REVISION),
                assessment: None,
            },
            FIXTURE_REVISION,
            &fixture_proposal(),
        ),
        Err(GateError::Scoring(_))
    ));
    assert!(
        ScoringEvidence::load(
            root,
            ScoringPaths::default(),
            FIXTURE_REVISION,
            &fixture_proposal()
        )
        .unwrap()
        .rubric
        .is_none()
    );
}

#[test]
fn actual_timing_wrapper_reports_integrated_results_without_timing_input_or_error_fallback() {
    let outside = Instant::now();
    let evaluation = time_evaluation(|| {
        Ok((
            fixture_scored_report(),
            fixture_scoring_evidence(),
            strategy::Attribution::NotConfigured,
        ))
    })
    .unwrap();
    let measured = evaluation.advisory_score.evaluation_time.elapsed_seconds;
    assert!(measured >= 0.0 && measured <= outside.elapsed().as_secs_f64());
    let json = serde_json::to_value(evaluation).unwrap();
    assert_eq!(json["decision"], "accepted");
    assert_eq!(json["advisory_score"]["metric"], SCORE_METRIC);
    assert_eq!(json["advisory_score"]["formula"], SCORE_FORMULA);
    assert_eq!(
        json["advisory_score"]["evaluation_time"]["scope"],
        TIMING_SCOPE
    );
    assert_eq!(json["advisory_score"]["reward"]["base_reward"], 1);
    assert!(matches!(
        time_evaluation(|| Err(GateError::Input(
            "early infrastructure/input failure".to_string()
        ))),
        Err(GateError::Input(_))
    ));
}

#[test]
fn scoring_artifacts_are_external_pinned_stable_and_create_once() -> Result<(), GateError> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory =
        env::temp_dir().join(format!("nucleoid-score-regression-{}", std::process::id()));
    fs::create_dir(&directory).map_err(|source| GateError::Io {
        path: directory.clone(),
        source,
    })?;
    let rubric_path = directory.join("rubric.json");
    let assessment_path = directory.join("assessment.json");
    assert!(matches!(
        read_scoring_artifact::<UsefulnessRubric>(root, &rubric_path),
        Err(GateError::Io { .. })
    ));
    assert!(matches!(
        read_scoring_artifact::<UsefulnessRubric>(root, &root.join("Cargo.toml")),
        Err(GateError::Scoring(_))
    ));
    let template = fixture_scoring_evidence();
    write_new(&rubric_path, &template.rubric.unwrap().input)?;
    let pin = scoring_blob(root, &rubric_path)?;
    let mut assessment = template.assessment.unwrap().input;
    assessment.rubric_revision = pin.clone();
    write_new(&assessment_path, &assessment)?;
    assert!(matches!(
        write_new(&assessment_path, &assessment),
        Err(GateError::Io { .. })
    ));
    let evaluation = time_evaluation(|| {
        let gate = fixture_scored_report();
        let evidence = ScoringEvidence::load(
            root,
            ScoringPaths {
                rubric: Some(&rubric_path),
                rubric_revision: Some(&pin),
                assessment: Some(&assessment_path),
            },
            &gate.baseline_revision,
            &gate.proposal,
        )?;
        evidence.verify_stable(root)?;
        Ok((gate, evidence, strategy::Attribution::NotConfigured))
    })?;
    assert_eq!(scored_values(&evaluation.advisory_score).1, 1);
    let report_path = directory.join("regression-report.json");
    write_new(&report_path, &evaluation)?;
    let saved: serde_json::Value = serde_json::from_str(&read(&report_path)?)?;
    assert_eq!(
        saved["advisory_score"]["usefulness_basis"]["rubric"]["revision"],
        pin
    );
    assert_eq!(
        saved["advisory_score"]["usefulness_basis"]["assessment"]["input"]["proposal_revision"],
        FIXTURE_PROPOSAL_REVISION
    );
    assert!(matches!(
        write_new(&report_path, &evaluation),
        Err(GateError::Io { .. })
    ));
    fs::write(&rubric_path, "{}").map_err(|source| GateError::Io {
        path: rubric_path.clone(),
        source,
    })?;
    assert!(matches!(
        evaluation
            .advisory_score
            .usefulness_basis
            .verify_stable(root),
        Err(GateError::Scoring(_))
    ));
    assert!(
        ScoringEvidence::load(
            root,
            ScoringPaths {
                rubric: Some(&rubric_path),
                rubric_revision: Some(&pin),
                assessment: Some(&assessment_path),
            },
            FIXTURE_REVISION,
            &fixture_proposal(),
        )
        .is_err()
    );
    fs::remove_dir_all(&directory).map_err(|source| GateError::Io {
        path: directory,
        source,
    })?;
    Ok(())
}
