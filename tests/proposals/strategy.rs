//! Session-local bandit learning, not category selection or model-weight training.
//!
//! A single owner retains every immutable input and advances one checkpoint chain.
//! Blob pins detect changed inputs; they do not authenticate an evaluator, prove
//! wall-clock pre-draft timing, or provide a concurrent/global-head lock.

use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Algorithm {
    #[serde(rename = "nuc-category-strategy-ucb1-v1")]
    CategoryStrategyUcb1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum RewardMetric {
    #[serde(rename = "nuc-advisory-usefulness-evaluator-time-v1")]
    AdvisoryUsefulnessEvaluatorTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LearningScope {
    AllAttributedTrials,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Strategy {
    ReuseFirst,
    EvidenceFirst,
}

impl Strategy {
    // This order is part of the algorithm, including cold-start and UCB ties.
    const ALL: [Self; 2] = [Self::ReuseFirst, Self::EvidenceFirst];

    fn index(self) -> usize {
        match self {
            Self::ReuseFirst => 0,
            Self::EvidenceFirst => 1,
        }
    }

    fn instructions(self) -> &'static str {
        match self {
            Self::ReuseFirst => {
                "Prioritize accepted primitives/helpers; implement the requested feature with a \
                 minimal additive extension. Retain every mandatory baseline and check."
            }
            Self::EvidenceFirst => {
                "Begin with a concrete unsupported/failing use case or counterexample for the \
                 requested feature, then implement the minimal preserving fix/extension. Retain \
                 every mandatory baseline and check."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactRef {
    path: PathBuf,
    revision: String,
}

impl<T> From<&ScoringArtifact<T>> for ArtifactRef {
    fn from(artifact: &ScoringArtifact<T>) -> Self {
        Self {
            path: artifact.path.clone(),
            revision: artifact.revision.clone(),
        }
    }
}

impl ArtifactRef {
    fn verify_stable(&self, root: &Path) -> Result<(), GateError> {
        binding(
            "artifact blob changed",
            &scoring_blob(root, &self.path)?,
            &self.revision,
        )
    }

    fn load<T: DeserializeOwned>(&self, root: &Path) -> Result<ScoringArtifact<T>, GateError> {
        load_pinned(root, &self.path, &self.revision)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    algorithm: Algorithm,
    policy_revision: String,
    trial_id: String,
    proposal_path: String,
    baseline_revision: String,
    category: Category,
    mode: ExecutionMode,
    strategy: Strategy,
}

struct Trial {
    id: String,
    proposal_path: String,
    baseline_revision: String,
    category: Category,
    mode: ExecutionMode,
}

impl Selection {
    fn new(policy: &ScoringArtifact<Policy>, trial: Trial) -> Result<Self, GateError> {
        let selection = Self {
            algorithm: policy.input.algorithm,
            policy_revision: policy.revision.clone(),
            trial_id: trial.id,
            proposal_path: trial.proposal_path,
            baseline_revision: trial.baseline_revision,
            category: trial.category,
            mode: trial.mode,
            strategy: policy.input.select(trial.category)?,
        };
        selection.validate_for(policy)?;
        Ok(selection)
    }

    fn validate(&self) -> Result<(), GateError> {
        text("trial_id", &self.trial_id)?;
        proposal_identity(&self.proposal_path)?;
        revision("selection.policy_revision", &self.policy_revision)?;
        revision("selection.baseline_revision", &self.baseline_revision)
    }

    fn validate_for(&self, policy: &ScoringArtifact<Policy>) -> Result<(), GateError> {
        self.validate()?;
        binding("selection parent", &self.policy_revision, &policy.revision)?;
        if self.algorithm != policy.input.algorithm
            || self.strategy != policy.input.select(self.category)?
        {
            return Err(invalid(
                "receipt does not reproduce the selected policy strategy",
            ));
        }
        if policy
            .input
            .history
            .iter()
            .any(|observation| observation.selection.input.trial_id == self.trial_id)
        {
            return Err(invalid("trial already observed in this checkpoint chain"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    parent: ArtifactRef,
    selection: ScoringArtifact<Selection>,
    evaluation: ArtifactRef,
    proposal_revision: String,
    raw_reward: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    algorithm: Algorithm,
    reward_metric: RewardMetric,
    learning_scope: LearningScope,
    history: Vec<Observation>,
}

// Counts are derived, never trusted serialized sufficient statistics. Every
// integer converted to f64 must remain exactly representable.
const MAX_TRIALS: u64 = (1 << 53) - 1;

fn next_count(value: u64) -> Result<u64, GateError> {
    value
        .checked_add(1)
        .filter(|value| *value <= MAX_TRIALS)
        .ok_or_else(|| invalid("trial counter exceeds the exact numeric range"))
}

fn normalize(reward: f64) -> Result<f64, GateError> {
    if !reward.is_finite() || !(-1.0..=1.0).contains(&reward) {
        return Err(invalid("raw reward must be finite and in [-1, 1]"));
    }
    Ok((reward + 1.0) / 2.0)
}

#[derive(Debug, Default, Clone, Copy)]
struct ArmStats {
    trials: u64,
    mean: f64,
}

impl ArmStats {
    fn ucb(self, category_trials: u64) -> Result<f64, GateError> {
        if self.trials == 0
            || self.trials > category_trials
            || category_trials > MAX_TRIALS
            || !self.mean.is_finite()
            || !(0.0..=1.0).contains(&self.mean)
        {
            return Err(invalid("invalid UCB statistics"));
        }
        let value = self.mean + (2.0 * (category_trials as f64).ln() / self.trials as f64).sqrt();
        if !value.is_finite() {
            return Err(invalid("nonfinite UCB value"));
        }
        Ok(value)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct CategoryStats {
    trials: u64,
    arms: [ArmStats; 2],
}

impl CategoryStats {
    fn observe(&mut self, strategy: Strategy, raw: f64) -> Result<(), GateError> {
        let normalized = normalize(raw)?;
        let category_trials = next_count(self.trials)?;
        let arm = &mut self.arms[strategy.index()];
        let arm_trials = next_count(arm.trials)?;
        let mean = arm.mean + (normalized - arm.mean) / arm_trials as f64;
        if !mean.is_finite() || !(0.0..=1.0).contains(&mean) {
            return Err(invalid("invalid observed mean"));
        }
        *arm = ArmStats {
            trials: arm_trials,
            mean,
        };
        self.trials = category_trials;
        Ok(())
    }

    fn select(self) -> Result<Strategy, GateError> {
        if let Some(strategy) = Strategy::ALL
            .into_iter()
            .find(|strategy| self.arms[strategy.index()].trials == 0)
        {
            return Ok(strategy);
        }
        let mut selected = Strategy::ALL[0];
        let mut best = self.arms[selected.index()].ucb(self.trials)?;
        for strategy in Strategy::ALL.into_iter().skip(1) {
            let value = self.arms[strategy.index()].ucb(self.trials)?;
            if value > best {
                selected = strategy;
                best = value;
            }
        }
        Ok(selected)
    }
}

impl Policy {
    fn validate(&self) -> Result<(), GateError> {
        let mut categories = HashMap::<Category, CategoryStats>::new();
        let mut trials = HashSet::new();
        let mut evaluated = HashSet::new();
        let mut reports = HashSet::new();
        for observation in &self.history {
            let selection = &observation.selection.input;
            selection.validate()?;
            revision("parent.revision", &observation.parent.revision)?;
            revision("selection blob", &observation.selection.revision)?;
            revision("evaluation blob", &observation.evaluation.revision)?;
            revision("evaluated proposal blob", &observation.proposal_revision)?;
            binding(
                "observation parent",
                &selection.policy_revision,
                &observation.parent.revision,
            )?;
            if !trials.insert(&selection.trial_id)
                || !evaluated.insert(&observation.proposal_revision)
                || !reports.insert(&observation.evaluation.revision)
            {
                return Err(invalid(
                    "duplicate trial, evaluated proposal revision, or report",
                ));
            }
            let stats = categories.entry(selection.category).or_default();
            if stats.select()? != selection.strategy {
                return Err(invalid(
                    "history strategy does not reproduce chronological selection",
                ));
            }
            stats.observe(selection.strategy, observation.raw_reward)?;
        }
        Ok(())
    }

    fn stats(&self, category: Category) -> Result<CategoryStats, GateError> {
        let mut stats = CategoryStats::default();
        for observation in &self.history {
            if observation.selection.input.category == category {
                stats.observe(observation.selection.input.strategy, observation.raw_reward)?;
            }
        }
        Ok(stats)
    }

    fn select(&self, category: Category) -> Result<Strategy, GateError> {
        self.validate()?;
        self.stats(category)?.select()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AttributedInputs {
    policy: ArtifactRef,
    selection: ScoringArtifact<Selection>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Attribution {
    #[default]
    NotConfigured,
    Attributed {
        inputs: Box<AttributedInputs>,
    },
}

impl Attribution {
    pub(super) fn validate_binding(&self, gate: &Report) -> Result<(), GateError> {
        let Self::Attributed { inputs } = self else {
            return Ok(());
        };
        let receipt = &inputs.selection.input;
        receipt.validate()?;
        revision("selection blob", &inputs.selection.revision)?;
        binding(
            "receipt parent",
            &receipt.policy_revision,
            &inputs.policy.revision,
        )?;
        binding(
            "receipt proposal path",
            &proposal_identity(&receipt.proposal_path)?,
            &proposal_identity(&gate.proposal.path)?,
        )?;
        binding(
            "receipt accepted baseline",
            &receipt.baseline_revision,
            &gate.baseline_revision,
        )?;
        if receipt.category != gate.proposal.category
            || receipt.category != gate.policy.category
            || receipt.mode != gate.policy.mode
        {
            return Err(invalid(
                "receipt category/mode differs from evaluated proposal",
            ));
        }
        Ok(())
    }

    pub(super) fn verify_stable(&self, root: &Path) -> Result<(), GateError> {
        if let Self::Attributed { inputs } = self {
            inputs.policy.verify_stable(root)?;
            inputs.selection.verify_stable(root)?;
        }
        Ok(())
    }
}

struct Inputs {
    policy: ScoringArtifact<Policy>,
    selection: ScoringArtifact<Selection>,
}

impl Inputs {
    fn attribution(&self) -> Attribution {
        Attribution::Attributed {
            inputs: Box::new(AttributedInputs {
                policy: (&self.policy).into(),
                selection: self.selection.clone(),
            }),
        }
    }
}

#[derive(Default)]
struct InputPaths<'a> {
    policy: Option<&'a Path>,
    policy_revision: Option<&'a str>,
    selection: Option<&'a Path>,
    selection_revision: Option<&'a str>,
}

fn load_inputs(root: &Path, paths: InputPaths<'_>) -> Result<Option<Inputs>, GateError> {
    let (policy, policy_revision, selection, selection_revision) = match (
        paths.policy,
        paths.policy_revision,
        paths.selection,
        paths.selection_revision,
    ) {
        (None, None, None, None) => return Ok(None),
        (Some(p), Some(pr), Some(s), Some(sr)) => (p, pr, s, sr),
        _ => {
            return Err(invalid(
                "learning requires policy AND selection paths with both frozen full blob pins",
            ));
        }
    };
    let policy = load_policy(root, policy, policy_revision)?;
    let selection: ScoringArtifact<Selection> = load_pinned(root, selection, selection_revision)?;
    selection.input.validate_for(&policy)?;
    policy.verify_stable(root)?;
    Ok(Some(Inputs { policy, selection }))
}

fn environment_inputs(root: &Path) -> Result<Option<Inputs>, GateError> {
    let policy = optional_scoring_path("NUC_STRATEGY_POLICY")?;
    let policy_revision = optional_environment("NUC_STRATEGY_POLICY_REVISION")?;
    let selection = optional_scoring_path("NUC_STRATEGY_SELECTION")?;
    let selection_revision = optional_environment("NUC_STRATEGY_SELECTION_REVISION")?;
    let inputs = load_inputs(
        root,
        InputPaths {
            policy: policy.as_deref(),
            policy_revision: policy_revision.as_deref(),
            selection: selection.as_deref(),
            selection_revision: selection_revision.as_deref(),
        },
    )?;
    let trial = optional_environment("NUC_STRATEGY_TRIAL_ID")?;
    let category = optional_environment("NUC_STRATEGY_CATEGORY")?;
    let checkpoint = optional_scoring_path("NUC_STRATEGY_CHECKPOINT")?;
    validate_hints(
        root,
        inputs.as_ref(),
        trial.as_deref(),
        category.as_deref(),
        checkpoint.as_deref(),
    )?;
    Ok(inputs)
}

fn validate_hints(
    root: &Path,
    inputs: Option<&Inputs>,
    trial: Option<&str>,
    category: Option<&str>,
    checkpoint: Option<&Path>,
) -> Result<(), GateError> {
    let Some(inputs) = inputs else {
        return if trial.is_some() || category.is_some() || checkpoint.is_some() {
            Err(invalid(
                "strategy variables supplied without complete pinned attribution",
            ))
        } else {
            Ok(())
        };
    };
    if let Some(trial) = trial {
        binding(
            "preserved trial identity",
            trial,
            &inputs.selection.input.trial_id,
        )?;
    }
    if let Some(category) = category {
        if Category::from_label(category)? != inputs.selection.input.category {
            return Err(invalid("preserved category differs from receipt"));
        }
    }
    if let Some(path) = checkpoint {
        external_output(root, path)?;
    }
    Ok(())
}

pub(super) fn read_attribution(
    root: &Path,
    baseline_revision: &str,
    proposal: &Proposal,
    policy: &EvaluationPolicy,
) -> Result<Attribution, GateError> {
    let Some(inputs) = environment_inputs(root)? else {
        return Ok(Attribution::NotConfigured);
    };
    let receipt = &inputs.selection.input;
    binding(
        "receipt baseline",
        &receipt.baseline_revision,
        baseline_revision,
    )?;
    binding(
        "receipt proposal",
        &proposal_identity(&receipt.proposal_path)?,
        &proposal_identity(&proposal.path)?,
    )?;
    if receipt.mode != policy.mode || receipt.category != proposal.category {
        return Err(invalid(
            "receipt category/mode does not match evaluator inputs",
        ));
    }
    Ok(inputs.attribution())
}

// A flat, strict wire view avoids serde's deny_unknown_fields/flatten conflict.
// Older unconfigured reports deserialize as unattributed; they cannot be backfilled.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SavedEvaluation {
    proposal: Proposal,
    baseline_revision: String,
    policy: EvaluationPolicy,
    token_check: TokenCheck,
    behavior: BehaviorChecks,
    decision: Decision,
    issues: Vec<Issue>,
    advisory_score: AdvisoryScore,
    #[serde(default)]
    strategy_learning: Attribution,
}

impl From<SavedEvaluation> for EvaluationReport {
    fn from(saved: SavedEvaluation) -> Self {
        Self {
            gate: Report {
                proposal: saved.proposal,
                baseline_revision: saved.baseline_revision,
                policy: saved.policy,
                token_check: saved.token_check,
                behavior: saved.behavior,
                decision: saved.decision,
                issues: saved.issues,
            },
            advisory_score: saved.advisory_score,
            strategy_learning: saved.strategy_learning,
        }
    }
}

fn validate_gate(gate: &Report) -> Result<(), GateError> {
    revision("report proposal blob", &gate.proposal.revision)?;
    revision("report baseline", &gate.baseline_revision)?;
    proposal_identity(&gate.proposal.path)?;
    let (runtime, reason, evidence) = match &gate.policy.runtime {
        RuntimeApplicability::NotApplicable => (None, None, None),
        RuntimeApplicability::NewFeature => (Some("new_feature"), None, None),
        RuntimeApplicability::ExistingComparable => (Some("existing_comparable"), None, None),
        RuntimeApplicability::ExistingUncomparable { reason, evidence } => (
            Some("existing_uncomparable"),
            Some(reason.as_str()),
            Some(evidence.as_str()),
        ),
    };
    let policy = EvaluationPolicy::from_inputs(
        gate.proposal.category,
        PolicyInputs {
            mode: Some(match gate.policy.mode {
                ExecutionMode::Single => "single",
                ExecutionMode::Continuous => "continuous",
            }),
            runtime,
            reason,
            evidence,
        },
    )?;
    if policy != gate.policy {
        return Err(invalid("saved report policy contradicts proposal"));
    }
    match (&gate.token_check, policy.skip_reason()) {
        (TokenCheck::Skipped { reason }, Some(expected)) if *reason == expected => {
            if gate
                .issues
                .iter()
                .any(|issue| matches!(issue, Issue::MoreTokens { .. } | Issue::UnpairedCase { .. }))
            {
                return Err(invalid(
                    "skipped token check cannot contain comparison-only issues",
                ));
            }
        }
        (TokenCheck::Ran { metric, cases }, None) if metric == METRIC => {
            // Every successfully checked candidate is either paired and measured
            // or explicitly unpaired; failed candidates have no comparison.
            let unpaired = gate
                .issues
                .iter()
                .filter(|issue| matches!(issue, Issue::UnpairedCase { .. }))
                .count();
            if cases.len().checked_add(unpaired) != Some(gate.behavior.candidate_cases_passed) {
                return Err(invalid(
                    "saved token comparisons do not account for checked candidate cases",
                ));
            }
            let mut ids = HashSet::new();
            for case in cases {
                let expected = match case.after.cmp(&case.before) {
                    std::cmp::Ordering::Less => TokenChange::Less,
                    std::cmp::Ordering::Equal => TokenChange::Same,
                    std::cmp::Ordering::Greater => TokenChange::More,
                };
                let has_issue = gate
                    .issues
                    .iter()
                    .any(|issue| matches!(issue, Issue::MoreTokens { id } if *id == case.id));
                if case.before == 0
                    || case.after == 0
                    || case.id.title.trim().is_empty()
                    || case.change != expected
                    || has_issue != (expected == TokenChange::More)
                    || !ids.insert(&case.id)
                {
                    return Err(invalid("inconsistent saved token comparison"));
                }
            }
            if gate.issues.iter().any(|issue| {
                matches!(issue, Issue::MoreTokens { id } if !cases.iter().any(|case| {
                    case.id == *id && case.change == TokenChange::More
                }))
            }) {
                return Err(invalid(
                    "token increase issue lacks its measured comparison",
                ));
            }
        }
        _ => return Err(invalid("saved report token policy mismatch")),
    }
    let expected_decision = if gate.issues.is_empty() {
        Decision::Accepted
    } else if gate
        .issues
        .iter()
        .all(|issue| matches!(issue, Issue::UnpairedCase { .. }))
    {
        Decision::Blocked
    } else {
        Decision::Rejected
    };
    if gate.decision != expected_decision {
        return Err(invalid("saved core verdict contradicts issues"));
    }
    if gate.decision == Decision::Accepted
        && (gate.behavior.legacy_cases_passed == 0 || gate.behavior.candidate_cases_passed == 0)
    {
        return Err(invalid(
            "accepted report cannot have an empty checked corpus",
        ));
    }
    Ok(())
}

fn validate_report(root: &Path, evaluation: &EvaluationReport) -> Result<Option<f64>, GateError> {
    let gate = &evaluation.gate;
    validate_gate(gate)?;
    evaluation.strategy_learning.validate_binding(gate)?;
    evaluation.strategy_learning.verify_stable(root)?;
    let score = &evaluation.advisory_score;
    let seconds = score.evaluation_time.elapsed_seconds;
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(invalid(
            "saved evaluator seconds must be finite and nonnegative",
        ));
    }
    Duration::try_from_secs_f64(seconds)
        .map_err(|_| invalid("saved evaluator seconds exceed Duration's numeric range"))?;
    let evidence = &score.usefulness_basis;
    evidence.validate(&gate.baseline_revision, &gate.proposal, &gate.issues)?;
    let actual_evidence = ScoringEvidence::load(
        root,
        ScoringPaths {
            rubric: evidence.rubric.as_ref().map(|value| value.path.as_path()),
            rubric_revision: evidence
                .rubric
                .as_ref()
                .map(|value| value.revision.as_str()),
            assessment: evidence
                .assessment
                .as_ref()
                .map(|value| value.path.as_path()),
        },
        &gate.baseline_revision,
        &gate.proposal,
    )?;
    if serde_json::to_value(&actual_evidence)? != serde_json::to_value(evidence)? {
        return Err(invalid(
            "saved scoring evidence differs from its pinned artifacts",
        ));
    }
    let reward = match score.reward {
        AdvisoryReward::Scored { final_reward, .. } => {
            normalize(final_reward)?;
            Some(final_reward)
        }
        AdvisoryReward::Unscored { .. } => None,
    };
    let mut expected = advisory_score_at_seconds(gate, actual_evidence, seconds);
    validate_reward(&score.reward, &expected.reward)?;
    // Decimal JSON round trips can differ by a couple of f64 ulps with the
    // existing serde_json configuration. Only the positive floating calculation
    // admits that bounded tolerance; neutral/rejection and all metadata are exact.
    expected.reward = score.reward.clone();
    if serde_json::to_value(&expected)? != serde_json::to_value(score)? {
        return Err(invalid(
            "saved advisory score does not reproduce the existing verifier formula",
        ));
    }
    Ok(reward)
}

fn validate_reward(actual: &AdvisoryReward, expected: &AdvisoryReward) -> Result<(), GateError> {
    let matches = match (actual, expected) {
        (
            AdvisoryReward::Scored {
                basis,
                base_reward,
                applied_cost_adjustment,
                final_reward,
            },
            AdvisoryReward::Scored {
                basis: expected_basis,
                base_reward: expected_base,
                applied_cost_adjustment: expected_adjustment,
                final_reward: expected_final,
            },
        ) => {
            basis == expected_basis
                && base_reward == expected_base
                && if *base_reward == 1 {
                    applied_cost_adjustment.is_finite()
                        && (-0.1..=0.0).contains(applied_cost_adjustment)
                        && (0.9..=1.0).contains(final_reward)
                        && (applied_cost_adjustment - expected_adjustment).abs()
                            <= 2.0 * f64::EPSILON
                        && (final_reward - expected_final).abs() <= 2.0 * f64::EPSILON
                } else {
                    applied_cost_adjustment == expected_adjustment && final_reward == expected_final
                }
        }
        (AdvisoryReward::Unscored { reason }, AdvisoryReward::Unscored { reason: expected }) => {
            reason == expected
        }
        _ => false,
    };
    if !matches {
        return Err(invalid(
            "saved reward differs from the existing verifier formula",
        ));
    }
    Ok(())
}

fn load_policy(root: &Path, path: &Path, pin: &str) -> Result<ScoringArtifact<Policy>, GateError> {
    let policy: ScoringArtifact<Policy> = load_pinned(root, path, pin)?;
    policy.input.validate()?;
    for (index, observation) in policy.input.history.iter().enumerate() {
        let parent: ScoringArtifact<Policy> = observation.parent.load(root)?;
        let prefix = Policy {
            history: policy.input.history[..index].to_vec(),
            ..policy.input.clone()
        };
        if parent.input != prefix {
            return Err(invalid(
                "checkpoint parent is not the exact preceding history",
            ));
        }
        let selection: ScoringArtifact<Selection> =
            ArtifactRef::from(&observation.selection).load(root)?;
        if selection != observation.selection {
            return Err(invalid(
                "checkpoint receipt differs from its pinned selection",
            ));
        }
        selection.input.validate_for(&parent)?;
        let evaluation: ScoringArtifact<EvaluationReport> = observation.evaluation.load(root)?;
        let expected = Inputs {
            policy: parent,
            selection,
        }
        .attribution();
        if evaluation.input.strategy_learning != expected {
            return Err(invalid(
                "history report is not attributed to this parent/receipt",
            ));
        }
        let raw = validate_report(root, &evaluation.input)?
            .ok_or_else(|| invalid("history contains an unscored observation"))?;
        if raw != observation.raw_reward
            || evaluation.input.gate.proposal.revision != observation.proposal_revision
        {
            return Err(invalid(
                "history reward/evaluated revision differs from saved report",
            ));
        }
    }
    policy.verify_stable(root)?;
    Ok(policy)
}

fn select_to_file(
    root: &Path,
    policy: &ScoringArtifact<Policy>,
    trial: Trial,
    output: &Path,
) -> Result<ScoringArtifact<Selection>, GateError> {
    let selection = Selection::new(policy, trial)?;
    let output = external_output(root, output)?;
    policy.verify_stable(root)?;
    write_new(&output, &selection)?;
    load_pinned(root, &output, &scoring_blob(root, &output)?)
}

#[derive(Debug, PartialEq, Eq)]
enum UpdateOutcome {
    Updated { path: PathBuf, revision: String },
    Unattributed,
    Unscored(UnscoredReason),
}

fn learn_from_report(
    root: &Path,
    inputs: Option<&Inputs>,
    evaluation: &ScoringArtifact<EvaluationReport>,
    output: &Path,
) -> Result<UpdateOutcome, GateError> {
    let output = external_output(root, output)?;
    let raw = validate_report(root, &evaluation.input)?;
    let gate = &evaluation.input.gate;
    let current = read_proposal(root, gate.proposal.path.clone())?;
    binding(
        "actual final proposal blob",
        &current.revision,
        &gate.proposal.revision,
    )?;
    if current.category != gate.proposal.category {
        return Err(invalid(
            "actual final proposal category differs from report",
        ));
    }
    evaluation.verify_stable(root)?;
    if evaluation.input.strategy_learning == Attribution::NotConfigured {
        if inputs.is_some() {
            return Err(invalid(
                "cannot retroactively attribute an unconfigured/legacy report",
            ));
        }
        return Ok(UpdateOutcome::Unattributed);
    }
    let inputs = inputs.ok_or_else(|| invalid("attributed report requires its pinned inputs"))?;
    inputs.selection.input.validate_for(&inputs.policy)?;
    if evaluation.input.strategy_learning != inputs.attribution() {
        return Err(invalid(
            "report attribution differs from supplied policy/selection",
        ));
    }
    if inputs
        .policy
        .input
        .history
        .iter()
        .any(|observation| observation.proposal_revision == gate.proposal.revision)
    {
        return Err(invalid(
            "actual evaluated proposal revision already observed",
        ));
    }
    let Some(raw_reward) = raw else {
        let AdvisoryReward::Unscored { reason } = evaluation.input.advisory_score.reward else {
            unreachable!("validated reward is either scored or explicitly unscored");
        };
        return Ok(UpdateOutcome::Unscored(reason));
    };
    let mut next = inputs.policy.input.clone();
    next.history.push(Observation {
        parent: (&inputs.policy).into(),
        selection: inputs.selection.clone(),
        evaluation: evaluation.into(),
        proposal_revision: gate.proposal.revision.clone(),
        raw_reward,
    });
    next.validate()?;
    inputs.attribution().verify_stable(root)?;
    evaluation.verify_stable(root)?;
    write_new(&output, &next)?;
    Ok(UpdateOutcome::Updated {
        path: output.clone(),
        revision: scoring_blob(root, &output)?,
    })
}

pub(super) fn select_command() -> Result<(), GateError> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if optional_environment("NUC_STRATEGY_SELECTION_REVISION")?.is_some()
        || optional_scoring_path("NUC_STRATEGY_CHECKPOINT")?.is_some()
    {
        return Err(invalid(
            "unset the old selection pin/checkpoint output before creating a new pre-draft receipt",
        ));
    }
    let policy = load_policy(
        root,
        &artifact("NUC_STRATEGY_POLICY")?,
        &required_environment("NUC_STRATEGY_POLICY_REVISION")?,
    )?;
    let category = Category::from_label(&required_environment("NUC_STRATEGY_CATEGORY")?)?;
    let mode = read_policy(category)?.mode;
    let baseline: Baseline = serde_json::from_str(&read(&artifact("NUC_PROPOSAL_BASELINE")?)?)?;
    baseline_cases(&baseline)?;
    // Resolve the accepted commit, not a candidate-derived or fabricated before-count.
    sources_at_revision(root, &baseline.revision)?;
    let selection = select_to_file(
        root,
        &policy,
        Trial {
            id: required_environment("NUC_STRATEGY_TRIAL_ID")?,
            proposal_path: required_environment("NUC_PROPOSAL")?,
            baseline_revision: baseline.revision,
            category,
            mode,
        },
        &artifact("NUC_STRATEGY_SELECTION")?,
    )?;
    println!(
        "Pre-draft selection: {}",
        serde_json::to_string(&selection)?
    );
    println!(
        "Proposer must consume: {}",
        selection.input.strategy.instructions()
    );
    Ok(())
}

pub(super) fn learn_command() -> Result<(), GateError> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let inputs = environment_inputs(root)?;
    let evaluation: ScoringArtifact<EvaluationReport> = load_pinned(
        root,
        &artifact("NUC_PROPOSAL_REPORT")?,
        &required_environment("NUC_PROPOSAL_REPORT_REVISION")?,
    )?;
    let baseline: Baseline = serde_json::from_str(&read(&artifact("NUC_PROPOSAL_BASELINE")?)?)?;
    baseline_cases(&baseline)?;
    binding(
        "report accepted baseline",
        &evaluation.input.gate.baseline_revision,
        &baseline.revision,
    )?;
    let proposal = read_proposal(root, required_environment("NUC_PROPOSAL")?)?;
    binding(
        "report proposal path",
        &proposal_identity(&evaluation.input.gate.proposal.path)?,
        &proposal_identity(&proposal.path)?,
    )?;
    if read_policy(proposal.category)? != evaluation.input.gate.policy {
        return Err(invalid(
            "report mode/category/runtime differs from preserved evaluator inputs",
        ));
    }
    let result = learn_from_report(
        root,
        inputs.as_ref(),
        &evaluation,
        &artifact("NUC_STRATEGY_CHECKPOINT")?,
    )?;
    println!("Strategy learning: {result:?}");
    Ok(())
}

fn invalid(message: impl Into<String>) -> GateError {
    GateError::Strategy(message.into())
}

fn text(field: &str, value: &str) -> Result<(), GateError> {
    if value.trim().is_empty() {
        return Err(invalid(format!("{field} must be nonblank")));
    }
    Ok(())
}

fn revision(field: &str, value: &str) -> Result<(), GateError> {
    if !full_git_revision(value) {
        return Err(invalid(format!(
            "{field} must be a full Git blob/commit ID"
        )));
    }
    Ok(())
}

fn binding(field: &str, actual: &str, expected: &str) -> Result<(), GateError> {
    if actual != expected {
        return Err(invalid(format!(
            "{field} mismatch: expected {expected:?}, found {actual:?}"
        )));
    }
    Ok(())
}

fn proposal_identity(path: &str) -> Result<String, GateError> {
    let normalized = path.replace('\\', "/");
    let number = normalized
        .strip_prefix("docs/nuc-")
        .and_then(|name| name.strip_suffix(".md"));
    if number
        .is_none_or(|number| number.len() != 4 || !number.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(invalid(
            "use the repository-relative docs/nuc-XXXX.md proposal path",
        ));
    }
    Ok(normalized)
}

fn load_pinned<T: DeserializeOwned>(
    root: &Path,
    path: &Path,
    pin: &str,
) -> Result<ScoringArtifact<T>, GateError> {
    revision("frozen artifact pin", pin)?;
    let artifact = read_scoring_artifact(root, path)?;
    binding("frozen artifact pin", &artifact.revision, pin)?;
    Ok(artifact)
}

fn external_output(root: &Path, path: &Path) -> Result<PathBuf, GateError> {
    if !path.is_absolute() || path.extension().is_none_or(|extension| extension != "json") {
        return Err(invalid("use an absolute external session JSON output path"));
    }
    let parent = path
        .parent()
        .ok_or_else(|| invalid("missing output parent"))?;
    let parent = fs::canonicalize(parent).map_err(|source| GateError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let canonical_root = fs::canonicalize(root).map_err(|source| GateError::Io {
        path: root.to_path_buf(),
        source,
    })?;
    if parent.starts_with(canonical_root) {
        return Err(invalid(
            "learner outputs belong outside the candidate repository",
        ));
    }
    let output = parent.join(
        path.file_name()
            .ok_or_else(|| invalid("missing output filename"))?,
    );
    match fs::symlink_metadata(&output) {
        Ok(_) => {
            return Err(invalid(
                "output already exists; use a fresh create-once JSON path",
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(GateError::Io {
                path: output,
                source,
            });
        }
    }
    Ok(output)
}

fn required_environment(variable: &'static str) -> Result<String, GateError> {
    let value = optional_environment(variable)?.ok_or(GateError::MissingEnvironment(variable))?;
    text(variable, &value)?;
    Ok(value)
}

#[path = "strategy_tests.rs"]
mod tests;
