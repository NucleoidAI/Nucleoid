---
name: nuc-evaluate
description: Independently evaluate a completed Nucleoid NUC candidate with mandatory behavior/integrity checks and the applicable lexer-token policy, not drafting or repairs.
---

# Evaluating a NUC proposal

When coordinating a NUC revision, wait for the proposal agent's completed
handoff, then launch a fresh `nuc-evaluate` agent using
[its profile](../../agents/nuc-evaluate.agent.md) and this skill. If named
custom agents are unavailable, give both documents to a fresh general-purpose
agent. Inside that agent, execute the steps below; do not spawn another agent.

Follow `.github/copilot-instructions.md` and `docs/nuc-0001.md`.
Evaluate exactly one completed revision, independently of the proposal agent.

## Verify the handoff

Require the proposal path and handed-off Git blob revision, accepted baseline
Git revision and immutable JSON artifact, candidate worktree, and fresh report
artifact path in the session workspace. Require the confirmed proposal
category and verify that its header matches the handoff and
[category registry](../../../docs/nuc-0001.md#proposal-categories).
Require the preserved execution mode and runtime applicability, including
reason and evidence for a genuinely uncomparable existing runtime feature.
Independently verify applicability and that evidence; missing comparison
artifacts do not prove impossibility. A Runtime features proposal needs an
explicit branch even in single mode.
Confirm that the current proposal blob matches the handoff and the baseline
matches the accepted revision. Missing prerequisites block evaluation; never
invent measurements or guess an uncertain category.

Treat candidate sources, generated exports, and the baseline as read-only.
Only new session reports and normal ignored build outputs may be written.
Do not repair code, rewrite cases or assertions, regenerate exports, replace
the baseline, or change proposal status/index metadata. A source change after
handoff requires a fresh proposal/evaluation pair, not an in-place repair.

## Validate independently

Run the smallest affected executable suites and the proposal gate regressions,
`cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings`.
Validate relevant dataset and exported-test consistency without regeneration.
Ensure `UPDATE_DATASET` and `UPDATE_SPEC_TESTS` are unset for evaluation.
Report failures explicitly rather than editing the candidate to make them pass.

Independently verify backward compatibility with accepted language syntax.
Run unchanged accepted-syntax counterparts from the pinned Git revision and
supplied extra-baseline sources under the candidate parser and runtime; check
their protected assertions, explicit expected returns and executed coverage.
Existing supported forms must keep their parsing and runtime semantics, even
when optional shorter forms or runtime features are added. A compatibility
failure rejects the proposal regardless of token savings or a passing gate.

The checked corpus is `nucleoid.spec.md`, `tests/reference.md`, and the
`nuc` blocks in `README.md` and `docs/examples.md`. Generated Rust and
synthesized training records are not scored.

## Measure and decide

The **token non-regression gate** requires each compared case to use the same
or fewer lexer tokens than its accepted baseline.
Single-proposal requests always skip token comparison, regardless of category.
Only continuous cycles apply the
[NUC 1 matrix](../../../docs/nuc-0001.md#proposal-evaluation):

- Syntax enhancement, Utility enhancement, and Reasoning enhancement: required.
- Runtime features: required for an existing comparable feature; skipped
  entirely for a brand-new feature; skipped for a genuinely uncomparable existing
  feature with explicit reason and evidence.
- Documentation: skipped.

Every mode still requires valid immutable accepted-corpus inputs, behavior,
case integrity, syntax compatibility, development checks, and outcome recording.
When tokens are skipped, genuinely new behavior cases may lack accepted-syntax
counterparts, but must execute assertions or explicit expected returns with
full assertion coverage. Pre-existing identities, canonical assertions, returns,
coverage, and legacy forms remain protected; never invent before-counts.

From the supplied candidate worktree, set:

- `NUC_PROPOSAL_BASELINE` to the supplied immutable baseline.
- `NUC_PROPOSAL` to the repository-relative `docs/nuc-XXXX.md` path.
- `NUC_PROPOSAL_REPORT` to the fresh session report path.
- `NUC_PROPOSAL_MODE` to the caller's `single` or `continuous` mode; explicitly
  use `single` for a one-shot request. Omitted mode defaults to `continuous`
  for existing open-ended invocations, not for explicit single requests.
- For Runtime features, `NUC_PROPOSAL_RUNTIME` to `new_feature`,
  `existing_comparable`, or `existing_uncomparable`; the last also requires
  nonblank `NUC_PROPOSAL_RUNTIME_REASON` and `NUC_PROPOSAL_RUNTIME_EVIDENCE`.
  Unset runtime inputs for other categories and reason/evidence for other
  runtime branches; inconsistent or unknown inputs are errors, not skips.
- `NUC_PROPOSAL_EXTRA_BASELINES` to the original extra-baseline source file
  when the artifact includes additional accepted-syntax cases.

Run the deterministic evaluator in every mode; only its token portion is
optional:

```console
cargo test --quiet --test proposals evaluate_proposal -- --ignored --nocapture
```

When comparison is required, the metric is `tokenize(case.source.trim())`
from the runtime lexer, including `Newline`, `Indent`, and `Dedent`, excluding
only `Eof`. It is not an LLM token
count. The gate protects case identities, canonical assertions, expected
returns, and executed assertion coverage.

- **Accepted:** all behavior/integrity and independent checks pass, accepted
  syntax remains backward compatible, no required case or baseline is missing,
  and either the required comparison has every paired case `same`/`less` or
  token checking was legitimately skipped under the policy.
- **Rejected:** any case is `more` when token comparison is required, or the
  candidate changes protected assertions/results/coverage, breaks accepted
  syntax compatibility, or fails behavior or development checks.
- **Blocked:** a required baseline or other evaluation prerequisite is absent.

Never offset a required-gate increase with savings elsewhere or decide from an
aggregate. Invalid/corrupt baseline artifacts and bad policy inputs are errors,
never successful skips; no behavior or compatibility failure may be suppressed.
The report records actual mode/category/runtime applicability and rationale,
behavior checks, and `token_check.status`: `ran` includes metric and measured
cases, while `skipped` includes the reason and no measured-case list. A skipped
comparison is not zero measured cases or a passing token gate.
Read the saved report and verify that its proposal revision still matches the
handoff. If no report was produced, state that and the actual error explicitly.

## Return the evidence

Return the proposal and baseline revisions, execution mode, confirmed category,
runtime applicability and rationale, decision, report path, independent
checks including syntax compatibility, and actual ran/skipped status with
reason. When tokens ran, include counts of `less`/`same`/`more`; include each
increased, missing, unpaired, invalid or syntax-incompatible case with its
before/after counts or reason. Keep the candidate and baseline unchanged.
The caller owns integration and status/index updates;
outcome metadata does not trigger another pair. No commits, pushes, or merges.

## Continuous-cycle handoff (caller only)

The evaluation worker stops after returning its evidence. It never records
outcome metadata, commits, integrates a candidate, or starts another cycle.

The caller preserves the proposal request's
[execution mode](../nuc-proposal/SKILL.md#choose-execution-mode-caller-only).
A single-proposal request does not enter a continuous cycle: finish its normal
evaluation and outcome recording, then stop without starting another proposal
or running the token non-regression gate (token comparison). Independent
behavior/integrity, accepted-corpus baseline, compatibility, and development
checks still apply.

When `/nuc-evaluate` coordinates an active continuous cycle, the caller follows
the [shared rejection workflow](../../instructions/proposals.instructions.md#record-rejected-outcomes-caller-only).
For `Rejected`, commit only the NUC rejection documentation and directly
related index/status metadata, using explicit paths and a path-limited commit.
Verify the committed file list; never apply or commit the rejected candidate's
runtime, executable-document, reference, dataset, or generated-export changes.
Keep implementation changes isolated and preserve unrelated staged and
unstaged work. Baselines and reports remain in the session workspace.

After recording the outcome, follow the
[continuous-cycle rule](../nuc-proposal/SKILL.md#continue-with-the-next-nuc-caller-only)
without requesting confirmation to record outcomes or continue. For uncertain
category additions, selections, updates, or assignments, workers return the
question to the caller, who asks the user for confirmation under the proposal skill's
[classification rule](../nuc-proposal/SKILL.md#classify-every-proposal).
Otherwise use the existing rules and verified inputs; a missing prerequisite
or other blocker pauses the cycle with an explicit report, not a question
or invented measurement.
Required tool approvals and user stop/pause instructions still apply.
