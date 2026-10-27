//! Controlled regression data only: no independent assessment of a real NUC,
//! historical-report import, paid calls, or live proposal-quality measurement.

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

const GENESIS: &str = r#"{
  "algorithm": "nuc-category-strategy-ucb1-v1",
  "reward_metric": "nuc-advisory-usefulness-evaluator-time-v1",
  "learning_scope": "all_attributed_trials",
  "history": []
}"#;

const CATEGORIES: [Category; 5] = [
    Category::Syntax,
    Category::Runtime,
    Category::Utility,
    Category::Reasoning,
    Category::Documentation,
];

fn genesis() -> Policy {
    serde_json::from_str(GENESIS).unwrap()
}

fn synthetic_observation(policy: &mut Policy, category: Category, reward: f64) {
    let index = policy.history.len() + 1;
    let parent_revision = format!("{index:040x}");
    policy.history.push(Observation {
        parent: ArtifactRef {
            path: env::temp_dir().join(format!("test-only-parent-{index}.json")),
            revision: parent_revision.clone(),
        },
        selection: ScoringArtifact {
            path: env::temp_dir().join(format!("test-only-selection-{index}.json")),
            revision: format!("{:040x}", index + 100),
            input: Selection {
                algorithm: Algorithm::CategoryStrategyUcb1,
                policy_revision: parent_revision,
                trial_id: format!("controlled-statistics-test-{index}"),
                proposal_path: "docs/nuc-9999.md".to_string(),
                baseline_revision: FIXTURE_REVISION.to_string(),
                category,
                mode: ExecutionMode::Single,
                strategy: policy.select(category).unwrap(),
            },
        },
        evaluation: ArtifactRef {
            path: env::temp_dir().join(format!("test-only-report-{index}.json")),
            revision: format!("{:040x}", index + 200),
        },
        proposal_revision: format!("{:040x}", index + 300),
        raw_reward: reward,
    });
}

struct TestSession {
    directory: PathBuf,
    root: PathBuf,
    artifacts: PathBuf,
}

fn io(path: &Path, work: std::io::Result<()>) -> Result<(), GateError> {
    work.map_err(|source| GateError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Clone, Copy)]
enum Scenario {
    Useful,
    Neutral,
    CoreRejected,
    ExternalFailed,
    Blocked,
    Unresolved,
    UnknownUsefulness,
    MissingAssessment,
    MissingRubric,
}

impl TestSession {
    fn new() -> Result<Self, GateError> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let directory = env::temp_dir().join(format!(
            "nucleoid-strategy-controlled-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let root = directory.join("candidate");
        let artifacts = directory.join("test-artifacts");
        io(&root, fs::create_dir_all(root.join("docs")))?;
        io(&artifacts, fs::create_dir(&artifacts))?;
        git(&root, &["init", "--quiet"])?;
        let registry = root.join("docs/nuc-0001.md");
        io(
            &registry,
            fs::write(&registry, include_str!("../../docs/nuc-0001.md")),
        )?;
        Ok(Self {
            directory,
            root,
            artifacts,
        })
    }

    fn save<T: Serialize + DeserializeOwned>(
        &self,
        name: &str,
        input: &T,
    ) -> Result<ScoringArtifact<T>, GateError> {
        let path = self.artifacts.join(format!("{name}.json"));
        write_new(&path, input)?;
        load_pinned(&self.root, &path, &scoring_blob(&self.root, &path)?)
    }

    fn initial(&self) -> Result<ScoringArtifact<Policy>, GateError> {
        self.save("test-genesis", &genesis())
    }

    fn selection(
        &self,
        parent: &ScoringArtifact<Policy>,
        id: &str,
        category: Category,
        mode: ExecutionMode,
    ) -> Result<Inputs, GateError> {
        let selection = select_to_file(
            &self.root,
            parent,
            Trial {
                id: format!("controlled-test-{id}"),
                proposal_path: "docs\\nuc-9999.md".to_string(),
                baseline_revision: FIXTURE_REVISION.to_string(),
                category,
                mode,
            },
            &self.artifacts.join(format!("{id}-selection.json")),
        )?;
        Ok(Inputs {
            policy: parent.clone(),
            selection,
        })
    }

    fn proposal(&self, id: &str, category: Category) -> Result<Proposal, GateError> {
        let label = serde_json::to_string(&category)?;
        let label: String = serde_json::from_str(&label)?;
        let path = self.root.join("docs/nuc-9999.md");
        io(
            &path,
            fs::write(
                &path,
                format!(
                    "# Controlled regression fixture {id}\n\n```\nNUC: 9999\n\
                     Title: Controlled regression fixture {id}\nType: Process\nStatus: Draft\n\
                     Category: {label}\n```\n"
                ),
            ),
        )?;
        read_proposal(&self.root, "docs/nuc-9999.md".to_string())
    }

    fn report(
        &self,
        id: &str,
        inputs: Option<&Inputs>,
        scenario: Scenario,
    ) -> Result<ScoringArtifact<EvaluationReport>, GateError> {
        let (category, mode) = inputs
            .map_or((Category::Syntax, ExecutionMode::Continuous), |inputs| {
                (inputs.selection.input.category, inputs.selection.input.mode)
            });
        let policy = EvaluationPolicy {
            category,
            mode,
            runtime: if category == Category::Runtime {
                RuntimeApplicability::ExistingComparable
            } else {
                RuntimeApplicability::NotApplicable
            },
        };
        let before = [fixture("controlled-paired", "assert(1, 1)")];
        let after = match scenario {
            Scenario::CoreRejected => vec![fixture("controlled-paired", "assert(2, 2)")],
            Scenario::Blocked => vec![
                fixture("controlled-paired", "assert(1, 1)"),
                fixture("controlled-new", "assert(2, 2)"),
            ],
            _ => vec![fixture("controlled-paired", "assert(1, 1)")],
        };
        let mut gate = verdict_with_policy(&before, &after, policy);
        // The pre-draft receipt exists before this controlled candidate is written.
        gate.proposal = self.proposal(id, category)?;
        let mut evidence = fixture_scoring_evidence();
        let rubric = evidence.rubric.as_mut().unwrap();
        rubric.input.proposal_path = gate.proposal.path.clone();
        let rubric = self.save(&format!("{id}-test-rubric"), &rubric.input)?;
        let assessment = &mut evidence.assessment.as_mut().unwrap().input;
        assessment.proposal_revision = gate.proposal.revision.clone();
        assessment.rubric_revision = rubric.revision.clone();
        assessment.proposal_path = gate.proposal.path.clone();
        assessment.assessor.id =
            "controlled-test-data-not-a-live-independent-evaluator".to_string();
        match scenario {
            Scenario::Neutral => {
                assessment.usefulness = UsefulnessJudgment::NoDemonstratedBenefit {
                    evidence: "controlled test: independently assessed neutral fixture".to_string(),
                };
            }
            Scenario::CoreRejected => {
                assessment.checks.behavior_integrity = IndependentCheck::Failed {
                    evidence: "controlled test: protected assertion changed".to_string(),
                };
            }
            Scenario::ExternalFailed => {
                assessment.checks.clippy = IndependentCheck::Failed {
                    evidence: "controlled test: external failure".to_string(),
                };
                assessment.usefulness = UsefulnessJudgment::Unknown {
                    reason: "controlled test: failure still earns -1 with unknown usefulness"
                        .to_string(),
                };
            }
            Scenario::Unresolved => {
                assessment.checks.development_suites = IndependentCheck::Unresolved {
                    reason: "controlled test: independent suite unresolved".to_string(),
                };
            }
            Scenario::UnknownUsefulness => {
                assessment.usefulness = UsefulnessJudgment::Unknown {
                    reason: "controlled test: unknown criterion evidence".to_string(),
                };
            }
            _ => {}
        }
        let assessment = if matches!(
            scenario,
            Scenario::MissingAssessment | Scenario::MissingRubric
        ) {
            None
        } else {
            Some(self.save(&format!("{id}-test-assessment"), assessment)?)
        };
        let evidence = if matches!(scenario, Scenario::MissingRubric) {
            ScoringEvidence::default()
        } else {
            ScoringEvidence {
                rubric: Some(rubric),
                assessment,
            }
        };
        let attribution = inputs.map_or(Attribution::NotConfigured, Inputs::attribution);
        let evaluation = time_evaluation(|| {
            evidence.verify_stable(&self.root)?;
            attribution.verify_stable(&self.root)?;
            Ok((gate, evidence, attribution))
        })?;
        self.save(&format!("{id}-test-evaluation"), &evaluation)
    }

    fn update(
        &self,
        id: &str,
        inputs: &Inputs,
        evaluation: &ScoringArtifact<EvaluationReport>,
    ) -> Result<ScoringArtifact<Policy>, GateError> {
        let output = self.artifacts.join(format!("{id}-test-checkpoint.json"));
        let result = learn_from_report(&self.root, Some(inputs), evaluation, &output)?;
        let UpdateOutcome::Updated { path, revision } = result else {
            panic!("expected a controlled scored update, got {result:?}");
        };
        load_policy(&self.root, &path, &revision)
    }

    fn step(
        &self,
        parent: &ScoringArtifact<Policy>,
        id: &str,
        category: Category,
        mode: ExecutionMode,
        scenario: Scenario,
    ) -> Result<ScoringArtifact<Policy>, GateError> {
        let inputs = self.selection(parent, id, category, mode)?;
        let evaluation = self.report(id, Some(&inputs), scenario)?;
        self.update(id, &inputs, &evaluation)
    }
}

impl Drop for TestSession {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn standard_genesis_reproduces_caller_cold_start_without_an_exception() {
    let policy = genesis();
    assert!(policy.history.is_empty());
    for category in CATEGORIES {
        assert_eq!(policy.select(category).unwrap(), Strategy::ReuseFirst);
    }
    let artifact = ScoringArtifact {
        path: env::temp_dir().join("controlled-genesis.json"),
        revision: "fee02dd919b6f0f69f63297b0db3af04178a66d5".to_string(),
        input: policy,
    };
    let selected = Selection::new(
        &artifact,
        Trial {
            id: "nuc-0001-strategy-learner-r1".to_string(),
            proposal_path: "docs\\nuc-0001.md".to_string(),
            baseline_revision: "58e6452756b2ef5b8bdc2a536a7622c0bc6956fa".to_string(),
            category: Category::Documentation,
            mode: ExecutionMode::Single,
        },
    )
    .unwrap();
    assert_eq!(selected.strategy, Strategy::ReuseFirst);
    assert_eq!(selected.policy_revision, artifact.revision);
    assert!(
        Strategy::ReuseFirst
            .instructions()
            .contains("accepted primitives/helpers")
    );
    assert!(
        Strategy::EvidenceFirst
            .instructions()
            .contains("counterexample")
    );
    assert_eq!(
        serde_json::to_value(&artifact.input).unwrap(),
        serde_json::from_str::<serde_json::Value>(GENESIS).unwrap()
    );
}

#[test]
fn exploration_ties_ucb_math_and_reward_dependent_choices() {
    for reward in [-1.0, 0.0, 1.0] {
        let mut policy = genesis();
        synthetic_observation(&mut policy, Category::Syntax, reward);
        assert_eq!(
            policy.select(Category::Syntax).unwrap(),
            Strategy::EvidenceFirst
        );
    }
    for (first, second, expected) in [
        (1.0, -1.0, Strategy::ReuseFirst),
        (-1.0, 1.0, Strategy::EvidenceFirst),
        (0.0, 0.0, Strategy::ReuseFirst),
    ] {
        let mut policy = genesis();
        synthetic_observation(&mut policy, Category::Syntax, first);
        synthetic_observation(&mut policy, Category::Syntax, second);
        assert_eq!(policy.select(Category::Syntax).unwrap(), expected);
        let stats = policy.stats(Category::Syntax).unwrap();
        let expected_ucb = (first + 1.0) / 2.0 + (2.0 * 2.0_f64.ln()).sqrt();
        assert!((stats.arms[0].ucb(2).unwrap() - expected_ucb).abs() < 1e-14);
        assert_eq!(stats.trials, 2);
    }
    let mut policy = genesis();
    for _ in 0..8 {
        synthetic_observation(&mut policy, Category::Syntax, 0.0);
    }
    let stats = policy.stats(Category::Syntax).unwrap();
    assert_eq!(stats.arms[0].trials, 4);
    assert_eq!(stats.arms[1].trials, 4);
    assert_eq!(stats.arms[0].mean, 0.5);
    assert_eq!(stats.arms[1].mean, 0.5);
    assert_eq!(
        policy.select(Category::Syntax).unwrap(),
        Strategy::ReuseFirst
    );
    let unbalanced = CategoryStats {
        trials: 10,
        arms: [
            ArmStats {
                trials: 9,
                mean: 0.8,
            },
            ArmStats {
                trials: 1,
                mean: 0.5,
            },
        ],
    };
    assert_eq!(unbalanced.select().unwrap(), Strategy::EvidenceFirst);
}

#[test]
fn categories_are_independent_and_neutral_is_an_observation_not_absence() {
    for category in CATEGORIES {
        let mut policy = genesis();
        synthetic_observation(&mut policy, category, 0.0);
        let stats = policy.stats(category).unwrap();
        assert_eq!(stats.trials, 1);
        assert_eq!(stats.arms[0].mean, 0.5);
        for other in CATEGORIES {
            assert_eq!(
                policy.select(other).unwrap(),
                if category == other {
                    Strategy::EvidenceFirst
                } else {
                    Strategy::ReuseFirst
                }
            );
        }
    }
}

#[test]
fn numeric_and_counter_validity_never_uses_nonfinite_ucb_sentinels() {
    for reward in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.01, 1.01] {
        assert!(normalize(reward).is_err());
        let mut policy = genesis();
        synthetic_observation(&mut policy, Category::Syntax, reward);
        assert!(policy.validate().is_err());
    }
    for count in [MAX_TRIALS, u64::MAX] {
        assert!(next_count(count).is_err());
        let mut stats = CategoryStats {
            trials: count,
            arms: [ArmStats::default(); 2],
        };
        assert!(stats.observe(Strategy::ReuseFirst, 0.0).is_err());
    }
    for arm in [
        ArmStats {
            trials: 0,
            mean: 0.0,
        },
        ArmStats {
            trials: 2,
            mean: 0.5,
        },
        ArmStats {
            trials: 1,
            mean: f64::NAN,
        },
        ArmStats {
            trials: 1,
            mean: f64::INFINITY,
        },
        ArmStats {
            trials: 1,
            mean: -0.1,
        },
        ArmStats {
            trials: 1,
            mean: 1.1,
        },
    ] {
        assert!(arm.ucb(1).is_err());
    }
    assert!(
        ArmStats {
            trials: 1,
            mean: 0.0
        }
        .ucb(MAX_TRIALS + 1)
        .is_err()
    );
    assert_eq!(normalize(-1.0).unwrap(), 0.0);
    assert_eq!(normalize(0.0).unwrap(), 0.5);
    assert_eq!(normalize(1.0).unwrap(), 1.0);
}

#[test]
fn real_saved_rewards_change_later_choices_after_reload() -> Result<(), GateError> {
    for (first, second, expected) in [
        (
            Scenario::Useful,
            Scenario::ExternalFailed,
            Strategy::ReuseFirst,
        ),
        (
            Scenario::ExternalFailed,
            Scenario::Useful,
            Strategy::EvidenceFirst,
        ),
        (Scenario::Neutral, Scenario::Neutral, Strategy::ReuseFirst),
    ] {
        let session = TestSession::new()?;
        let initial = session.initial()?;
        let initial_bytes = read(&initial.path)?;
        let first = session.step(
            &initial,
            "first",
            Category::Documentation,
            ExecutionMode::Single,
            first,
        )?;
        assert_eq!(
            first.input.select(Category::Documentation)?,
            Strategy::EvidenceFirst
        );
        let second = session.step(
            &first,
            "second",
            Category::Documentation,
            ExecutionMode::Continuous,
            second,
        )?;
        let reloaded = load_policy(&session.root, &second.path, &second.revision)?;
        let next = session.selection(
            &reloaded,
            "next",
            Category::Documentation,
            ExecutionMode::Single,
        )?;
        assert_eq!(next.selection.input.strategy, expected);
        assert_eq!(second.input.history.len(), 2);
        assert_eq!(
            second.input.select(Category::Runtime)?,
            Strategy::ReuseFirst
        );
        assert_eq!(read(&initial.path)?, initial_bytes);
    }
    Ok(())
}

#[test]
fn all_attributed_scope_learns_positive_neutral_and_both_negative_bases_in_both_modes()
-> Result<(), GateError> {
    for mode in [ExecutionMode::Single, ExecutionMode::Continuous] {
        for (scenario, base) in [
            (Scenario::Useful, 1),
            (Scenario::Neutral, 0),
            (Scenario::CoreRejected, -1),
            (Scenario::ExternalFailed, -1),
        ] {
            let session = TestSession::new()?;
            let parent = session.initial()?;
            let inputs = session.selection(&parent, "trial", Category::Syntax, mode)?;
            let evaluation = session.report("trial", Some(&inputs), scenario)?;
            assert_eq!(scored_values(&evaluation.input.advisory_score).1, base);
            assert_eq!(
                evaluation.input.gate.policy.skip_reason(),
                (mode == ExecutionMode::Single).then_some(TokenSkipReason::SingleProposal)
            );
            let next = session.update("trial", &inputs, &evaluation)?;
            assert_eq!(next.input.history.len(), 1);
            assert_eq!(
                next.input.history[0].raw_reward,
                scored_values(&evaluation.input.advisory_score).3
            );
            assert_eq!(next.input.stats(Category::Syntax)?.trials, 1);
        }
    }
    Ok(())
}

#[test]
fn blocked_missing_unknown_and_unresolved_rewards_do_not_write_or_increment()
-> Result<(), GateError> {
    for scenario in [
        Scenario::Blocked,
        Scenario::Unresolved,
        Scenario::UnknownUsefulness,
        Scenario::MissingAssessment,
        Scenario::MissingRubric,
    ] {
        let session = TestSession::new()?;
        let parent = session.initial()?;
        let before = read(&parent.path)?;
        let inputs = session.selection(
            &parent,
            "unscored",
            Category::Syntax,
            ExecutionMode::Continuous,
        )?;
        let evaluation = session.report("unscored", Some(&inputs), scenario)?;
        let output = session.artifacts.join("must-not-exist.json");
        assert!(matches!(
            learn_from_report(&session.root, Some(&inputs), &evaluation, &output)?,
            UpdateOutcome::Unscored(_)
        ));
        assert!(!output.exists());
        assert_eq!(read(&parent.path)?, before);
        assert_eq!(parent.input.stats(Category::Syntax)?.trials, 0);
    }
    Ok(())
}

#[test]
fn legacy_unattributed_reports_cannot_be_retroactively_assigned() -> Result<(), GateError> {
    let session = TestSession::new()?;
    let parent = session.initial()?;
    let inputs = session.selection(
        &parent,
        "later",
        Category::Syntax,
        ExecutionMode::Continuous,
    )?;
    let original = session.report("legacy", None, Scenario::Useful)?;
    let mut json = serde_json::to_value(&original.input)?;
    assert_eq!(json["strategy_learning"]["status"], "not_configured");
    json.as_object_mut().unwrap().remove("strategy_learning");
    let path = session
        .artifacts
        .join("legacy-report-without-learning-field.json");
    write_new(&path, &json)?;
    let legacy: ScoringArtifact<EvaluationReport> =
        load_pinned(&session.root, &path, &scoring_blob(&session.root, &path)?)?;
    let output = session.artifacts.join("no-retroactive-learning.json");
    assert_eq!(
        learn_from_report(&session.root, None, &legacy, &output)?,
        UpdateOutcome::Unattributed
    );
    assert!(learn_from_report(&session.root, Some(&inputs), &legacy, &output).is_err());
    assert!(!output.exists());
    assert!(load_inputs(&session.root, InputPaths::default())?.is_none());
    Ok(())
}

#[test]
fn attributed_reports_require_exact_pre_draft_inputs_and_create_once_outputs()
-> Result<(), GateError> {
    let session = TestSession::new()?;
    let parent = session.initial()?;
    let inputs = session.selection(
        &parent,
        "owned",
        Category::Documentation,
        ExecutionMode::Single,
    )?;
    let report = session.report("owned", Some(&inputs), Scenario::Useful)?;
    assert_eq!(report.input.strategy_learning, inputs.attribution());
    let output = session.artifacts.join("owned-checkpoint.json");
    assert!(learn_from_report(&session.root, None, &report, &output).is_err());
    assert!(!output.exists());
    let first = learn_from_report(&session.root, Some(&inputs), &report, &output)?;
    assert!(matches!(first, UpdateOutcome::Updated { .. }));
    let preserved = read(&output)?;
    assert!(learn_from_report(&session.root, Some(&inputs), &report, &output).is_err());
    assert_eq!(read(&output)?, preserved);
    assert!(
        session
            .selection(
                &parent,
                "owned",
                Category::Documentation,
                ExecutionMode::Single
            )
            .is_err()
    );
    assert!(external_output(&session.root, &session.root.join("forbidden.json")).is_err());
    assert!(external_output(&session.root, Path::new("relative.json")).is_err());
    assert!(external_output(&session.root, &session.artifacts.join("not-json.txt")).is_err());
    Ok(())
}

#[test]
fn stale_receipts_and_trial_or_actual_revision_duplicates_are_errors() -> Result<(), GateError> {
    let session = TestSession::new()?;
    let parent = session.initial()?;
    let inputs = session.selection(&parent, "first", Category::Syntax, ExecutionMode::Single)?;
    let report = session.report("same-candidate", Some(&inputs), Scenario::Useful)?;
    let head = session.update("first", &inputs, &report)?;
    assert!(inputs.selection.input.validate_for(&head).is_err());
    assert!(
        session
            .selection(&head, "first", Category::Syntax, ExecutionMode::Single)
            .is_err()
    );
    let new_inputs = session.selection(
        &head,
        "different-trial",
        Category::Syntax,
        ExecutionMode::Single,
    )?;
    // Same actual proposal blob, but a different trial, report/assessment and elapsed time.
    let mut rerun: EvaluationReport = serde_json::from_str(&read(&report.path)?)?;
    rerun.strategy_learning = new_inputs.attribution();
    rerun.advisory_score = advisory_score(
        &rerun.gate,
        rerun.advisory_score.usefulness_basis.clone(),
        Duration::from_secs(2),
    );
    let rerun = session.save("distinct-rerun-report", &rerun)?;
    assert_ne!(rerun.revision, report.revision);
    assert_eq!(
        rerun.input.gate.proposal.revision,
        report.input.gate.proposal.revision
    );
    let output = session.artifacts.join("no-duplicate.json");
    assert!(learn_from_report(&session.root, Some(&new_inputs), &rerun, &output).is_err());
    assert!(!output.exists());
    Ok(())
}

#[test]
fn incomplete_supplied_learning_configuration_is_never_an_unconfigured_fallback()
-> Result<(), GateError> {
    let session = TestSession::new()?;
    let parent = session.initial()?;
    let inputs = session.selection(
        &parent,
        "config",
        Category::Syntax,
        ExecutionMode::Continuous,
    )?;
    for bits in 1..15 {
        let paths = InputPaths {
            policy: (bits & 1 != 0).then_some(parent.path.as_path()),
            policy_revision: (bits & 2 != 0).then_some(parent.revision.as_str()),
            selection: (bits & 4 != 0).then_some(inputs.selection.path.as_path()),
            selection_revision: (bits & 8 != 0).then_some(inputs.selection.revision.as_str()),
        };
        assert!(load_inputs(&session.root, paths).is_err());
    }
    for pin in ["", "not-a-pin", FIXTURE_REVISION] {
        assert!(
            load_inputs(
                &session.root,
                InputPaths {
                    policy: Some(&parent.path),
                    policy_revision: Some(pin),
                    selection: Some(&inputs.selection.path),
                    selection_revision: Some(&inputs.selection.revision),
                },
            )
            .is_err()
        );
    }
    assert!(
        load_inputs(
            &session.root,
            InputPaths {
                policy: Some(&parent.path),
                policy_revision: Some(&parent.revision),
                selection: Some(&inputs.selection.path),
                selection_revision: Some(&inputs.selection.revision),
            },
        )?
        .is_some()
    );
    for (trial, category, checkpoint) in [
        (Some("stray"), None, None),
        (None, Some("Documentation"), None),
        (
            None,
            None,
            Some(session.artifacts.join("stray-output.json")),
        ),
    ] {
        assert!(
            validate_hints(&session.root, None, trial, category, checkpoint.as_deref()).is_err()
        );
    }
    for (trial, category, checkpoint) in [
        (Some("wrong-trial"), None, None),
        (Some(""), None, None),
        (None, Some("Documentation"), None),
        (None, Some("unknown"), None),
        (None, None, Some(session.root.join("forbidden.json"))),
    ] {
        assert!(
            validate_hints(
                &session.root,
                Some(&inputs),
                trial,
                category,
                checkpoint.as_deref()
            )
            .is_err()
        );
    }
    validate_hints(
        &session.root,
        Some(&inputs),
        Some(&inputs.selection.input.trial_id),
        Some("Syntax enhancement"),
        Some(&session.artifacts.join("fresh-output.json")),
    )?;
    assert!(
        load_inputs(
            &session.root,
            InputPaths {
                policy: Some(&parent.path),
                policy_revision: Some(&parent.revision),
                selection: Some(&inputs.selection.path),
                selection_revision: Some(FIXTURE_REVISION),
            },
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn changed_input_blobs_are_errors_and_old_policy_cannot_be_an_output() -> Result<(), GateError> {
    let session = TestSession::new()?;
    let parent = session.initial()?;
    let inputs = session.selection(
        &parent,
        "pinned",
        Category::Documentation,
        ExecutionMode::Single,
    )?;
    let report = session.report("pinned", Some(&inputs), Scenario::Neutral)?;
    let before = read(&parent.path)?;
    assert!(learn_from_report(&session.root, Some(&inputs), &report, &parent.path).is_err());
    assert_eq!(read(&parent.path)?, before);
    // Deliberate corruption of disposable test data, never a real session input.
    io(
        &inputs.selection.path,
        fs::write(&inputs.selection.path, "{}"),
    )?;
    assert!(inputs.attribution().verify_stable(&session.root).is_err());
    let output = session.artifacts.join("no-changed-input-update.json");
    assert!(learn_from_report(&session.root, Some(&inputs), &report, &output).is_err());
    assert!(!output.exists());
    assert_eq!(read(&parent.path)?, before);
    Ok(())
}

#[test]
fn positive_json_roundoff_tolerance_does_not_change_exact_neutral_or_negative_rewards() {
    let scored = |base, adjustment, reward| AdvisoryReward::Scored {
        basis: match base {
            1 => RewardBasis::DemonstratedUsefulness,
            0 => RewardBasis::NoDemonstratedBenefit,
            _ => RewardBasis::GateRejected,
        },
        base_reward: base,
        applied_cost_adjustment: adjustment,
        final_reward: reward,
    };
    let expected = scored(1, -0.05, 0.95);
    assert!(validate_reward(&scored(1, -0.05, 0.95 + f64::EPSILON), &expected).is_ok());
    assert!(validate_reward(&scored(1, -0.05, 0.9500001), &expected).is_err());
    for raw in [f64::NAN, f64::INFINITY, -1.1, 1.1] {
        assert!(validate_reward(&scored(1, -0.05, raw), &expected).is_err());
    }
    assert!(validate_reward(&scored(0, 0.0, f64::EPSILON), &scored(0, 0.0, 0.0)).is_err());
    assert!(
        validate_reward(
            &scored(-1, 0.0, -1.0 + f64::EPSILON),
            &scored(-1, 0.0, -1.0)
        )
        .is_err()
    );
}

#[test]
fn malformed_unknown_and_out_of_range_policy_and_selection_inputs_fail() {
    let policy = serde_json::to_value(genesis()).unwrap();
    for field in ["algorithm", "reward_metric", "learning_scope", "history"] {
        let mut missing = policy.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<Policy>(missing).is_err());
    }
    for (field, value) in [
        ("algorithm", "other-learner"),
        ("reward_metric", "other-score"),
        ("learning_scope", "continuous_only"),
        ("statistics", "untrusted-counters"),
    ] {
        let mut invalid = policy.clone();
        invalid[field] = value.into();
        assert!(serde_json::from_value::<Policy>(invalid).is_err());
    }
    assert!(serde_json::from_str::<Policy>("{").is_err());
    assert!(
        serde_json::from_str::<Policy>(
            &GENESIS.replace("\"history\": []", "\"history\": [], \"history\": []")
        )
        .is_err()
    );
    let mut policy = genesis();
    synthetic_observation(&mut policy, Category::Documentation, 0.0);
    let json = serde_json::to_value(&policy).unwrap();
    for value in ["NaN", "Infinity", "1e400"] {
        let text = serde_json::to_string(&json)
            .unwrap()
            .replace("\"raw_reward\":0.0", &format!("\"raw_reward\":{value}"));
        assert!(serde_json::from_str::<Policy>(&text).is_err());
    }
    for value in [-1.1, 1.1] {
        let mut invalid = json.clone();
        invalid["history"][0]["raw_reward"] = value.into();
        assert!(
            serde_json::from_value::<Policy>(invalid)
                .unwrap()
                .validate()
                .is_err()
        );
    }
    let selection = serde_json::to_value(&policy.history[0].selection.input).unwrap();
    for (field, value) in [
        ("algorithm", "other"),
        ("strategy", "invented_arm"),
        ("category", "invented_category"),
        ("mode", "other"),
        ("reward", "self-reported"),
    ] {
        let mut invalid = selection.clone();
        invalid[field] = value.into();
        assert!(serde_json::from_value::<Selection>(invalid).is_err());
    }
    for path in [
        "",
        "../docs/nuc-0001.md",
        "docs/nuc-1.md",
        "C:\\docs\\nuc-0001.md",
    ] {
        assert!(proposal_identity(path).is_err());
    }
    assert_eq!(
        proposal_identity("docs\\nuc-0001.md").unwrap(),
        "docs/nuc-0001.md"
    );
}

#[test]
fn tampered_saved_scores_attribution_provenance_and_core_policy_are_errors() -> Result<(), GateError>
{
    let session = TestSession::new()?;
    let parent = session.initial()?;
    let inputs = session.selection(
        &parent,
        "valid",
        Category::Documentation,
        ExecutionMode::Single,
    )?;
    let evaluation = session.report("valid", Some(&inputs), Scenario::Useful)?;
    let original = serde_json::to_value(&evaluation.input)?;
    let changes: &[fn(&mut serde_json::Value)] = &[
        |r| r["advisory_score"]["reward"]["final_reward"] = 0.123.into(),
        |r| r["advisory_score"]["reward"]["base_reward"] = 0.into(),
        |r| r["advisory_score"]["metric"] = "other".into(),
        |r| r["advisory_score"]["formula"] = "other".into(),
        |r| r["advisory_score"]["evaluation_time"]["elapsed_seconds"] = (-1.0).into(),
        |r| r["advisory_score"]["evaluation_time"]["scope"] = "self-reported".into(),
        |r| r["advisory_score"]["independent_check_eligibility"] = "failed".into(),
        |r| {
            r["advisory_score"]["usefulness_basis"]["assessment"]["input"]["assessor"]["id"] =
                "tampered".into()
        },
        |r| {
            r["strategy_learning"]["inputs"]["selection"]["input"]["strategy"] =
                "evidence_first".into()
        },
        |r| {
            r["strategy_learning"]["inputs"]["selection"]["input"]["trial_id"] =
                "post-hoc-trial".into()
        },
        |r| r["strategy_learning"]["inputs"]["selection"]["revision"] = FIXTURE_REVISION.into(),
        |r| r["strategy_learning"]["inputs"]["policy"]["revision"] = FIXTURE_REVISION.into(),
        |r| r["proposal"]["path"] = "docs/nuc-0001.md".into(),
        |r| r["proposal"]["revision"] = FIXTURE_REVISION.into(),
        |r| r["baseline_revision"] = FIXTURE_PROPOSAL_REVISION.into(),
        |r| r["policy"]["mode"] = "continuous".into(),
        |r| r["proposal"]["category"] = "Syntax enhancement".into(),
        |r| r["decision"] = "rejected".into(),
        |r| r["token_check"]["reason"] = "documentation".into(),
    ];
    for (index, change) in changes.iter().enumerate() {
        let mut corrupted = original.clone();
        change(&mut corrupted);
        let path = session
            .artifacts
            .join(format!("tampered-test-report-{index}.json"));
        write_new(&path, &corrupted)?;
        let report: ScoringArtifact<EvaluationReport> =
            load_pinned(&session.root, &path, &scoring_blob(&session.root, &path)?)?;
        let output = session
            .artifacts
            .join(format!("no-tampered-update-{index}.json"));
        assert!(
            learn_from_report(&session.root, Some(&inputs), &report, &output).is_err(),
            "corruption {index}"
        );
        assert!(!output.exists());
    }
    let mut extra = original;
    extra["self_reported_reward"] = 1.into();
    assert!(serde_json::from_value::<EvaluationReport>(extra).is_err());
    let mut impossible: EvaluationReport = serde_json::from_str(&read(&evaluation.path)?)?;
    impossible.gate.issues.push(Issue::MoreTokens {
        id: CaseId {
            document: Document::Spec,
            title: "comparison-issue-in-a-skipped-report".to_string(),
        },
    });
    impossible.gate.decision = Decision::Rejected;
    impossible.advisory_score = advisory_score(
        &impossible.gate,
        impossible.advisory_score.usefulness_basis.clone(),
        Duration::ZERO,
    );
    assert!(validate_report(&session.root, &impossible).is_err());
    let mut impossible: EvaluationReport = serde_json::from_str(&read(&evaluation.path)?)?;
    impossible.gate.behavior.legacy_cases_passed = 0;
    assert!(validate_report(&session.root, &impossible).is_err());
    session.proposal("changed-after-evaluation", Category::Documentation)?;
    let output = session.artifacts.join("no-stale-final-proposal.json");
    assert!(learn_from_report(&session.root, Some(&inputs), &evaluation, &output).is_err());
    assert!(!output.exists());
    Ok(())
}

#[test]
fn saved_token_comparisons_match_successfully_checked_paired_cases() -> Result<(), GateError> {
    let session = TestSession::new()?;
    let parent = session.initial()?;
    let inputs = session.selection(
        &parent,
        "comparison-counts",
        Category::Syntax,
        ExecutionMode::Continuous,
    )?;
    let evaluation = session.report("comparison-counts", Some(&inputs), Scenario::Useful)?;
    assert!(validate_report(&session.root, &evaluation.input)?.is_some());
    for (name, omit) in [("omitted", true), ("invented", false)] {
        let mut corrupted: EvaluationReport = serde_json::from_str(&read(&evaluation.path)?)?;
        let TokenCheck::Ran { cases, .. } = &mut corrupted.gate.token_check else {
            panic!("continuous Syntax trial must run token comparison");
        };
        assert_eq!(cases.len(), 1);
        assert_eq!(corrupted.gate.behavior.candidate_cases_passed, 1);
        if omit {
            cases.clear();
        } else {
            cases.push(Comparison {
                id: CaseId {
                    document: Document::Spec,
                    title: "never-checked".to_string(),
                },
                before: 1,
                after: 1,
                change: TokenChange::Same,
            });
        }
        let corrupted = session.save(&format!("{name}-test-comparisons"), &corrupted)?;
        let output = session
            .artifacts
            .join(format!("no-{name}-test-checkpoint.json"));
        let result = learn_from_report(&session.root, Some(&inputs), &corrupted, &output);
        assert!(
            matches!(&result, Err(GateError::Strategy(reason))
                if reason == "saved token comparisons do not account for checked candidate cases"),
            "{name} comparisons must not produce an observation: {result:?}"
        );
        assert!(!output.exists());
    }
    let before = [fixture("paired", "assert(1, 1)")];
    let after = [fixture("paired", "assert(1, 2)")];
    let rejected = verdict(&before, &after);
    assert_eq!(rejected.decision, Decision::Rejected);
    assert_eq!(rejected.behavior.candidate_cases_passed, 0);
    assert!(comparisons(&rejected).is_empty());
    validate_gate(&rejected)?;
    Ok(())
}

#[test]
fn checkpoint_reload_validates_parent_receipt_report_and_reward_not_cached_counters()
-> Result<(), GateError> {
    let session = TestSession::new()?;
    let initial = session.initial()?;
    let head = session.step(
        &initial,
        "chain",
        Category::Utility,
        ExecutionMode::Continuous,
        Scenario::Neutral,
    )?;
    let changes: &[fn(&mut Policy)] = &[
        |p| p.history[0].raw_reward = 0.5,
        |p| p.history[0].parent.revision = FIXTURE_REVISION.to_string(),
        |p| p.history[0].selection.input.trial_id = "fabricated-history".to_string(),
        |p| p.history[0].evaluation.revision = FIXTURE_REVISION.to_string(),
        |p| p.history[0].proposal_revision = FIXTURE_REVISION.to_string(),
        |p| p.history[0].selection.input.strategy = Strategy::EvidenceFirst,
        |p| p.history.push(p.history[0].clone()),
    ];
    for (index, change) in changes.iter().enumerate() {
        let mut corrupted = head.input.clone();
        change(&mut corrupted);
        let artifact = session.save(&format!("tampered-test-state-{index}"), &corrupted)?;
        assert!(load_policy(&session.root, &artifact.path, &artifact.revision).is_err());
    }
    assert!(load_policy(&session.root, &head.path, FIXTURE_REVISION).is_err());
    assert_eq!(
        load_policy(&session.root, &head.path, &head.revision)?.input,
        head.input
    );
    Ok(())
}

#[test]
fn attribution_leaves_every_core_scoring_and_token_verdict_matrix_entry_unchanged()
-> Result<(), GateError> {
    let before = [fixture("paired", "a = 1\nassert(a, 1)")];
    let after = [fixture("paired", "a = (1)\nassert(a, 1)")];
    for (policy, _) in policy_matrix() {
        let gate = verdict_with_policy(&before, &after, policy.clone());
        let original = serde_json::to_value(&gate)?;
        let score = fixture_score(&gate, fixture_scoring_evidence(), Duration::from_secs(1));
        let score_json = serde_json::to_value(&score)?;
        let state = ScoringArtifact {
            path: env::temp_dir().join("controlled-matrix-policy.json"),
            revision: FIXTURE_RUBRIC_REVISION.to_string(),
            input: genesis(),
        };
        let receipt = Selection::new(
            &state,
            Trial {
                id: "controlled-matrix".to_string(),
                proposal_path: gate.proposal.path.clone(),
                baseline_revision: gate.baseline_revision.clone(),
                category: policy.category,
                mode: policy.mode,
            },
        )?;
        let attribution = Inputs {
            policy: state,
            selection: ScoringArtifact {
                path: env::temp_dir().join("controlled-matrix-selection.json"),
                revision: FIXTURE_PROPOSAL_REVISION.to_string(),
                input: receipt,
            },
        }
        .attribution();
        attribution.validate_binding(&gate)?;
        let evaluation = EvaluationReport {
            gate,
            advisory_score: score,
            strategy_learning: attribution,
        };
        let mut combined = serde_json::to_value(evaluation)?;
        assert_eq!(
            combined
                .as_object_mut()
                .unwrap()
                .remove("advisory_score")
                .unwrap(),
            score_json
        );
        combined
            .as_object_mut()
            .unwrap()
            .remove("strategy_learning");
        assert_eq!(combined, original);
    }
    Ok(())
}
