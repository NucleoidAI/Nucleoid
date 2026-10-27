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

## Assess advisory usefulness (optional)

Follow [NUC 1's scoring policy](../../../docs/nuc-0001.md#advisory-usefulness-and-evaluation-cost)
and the [JSON schemas](../../../CONTRIBUTING.md#advisory-usefulnesscost-scoring).
When supplied, require the caller-owned rubric and its full Git blob pin frozen
before drafting. Verify its criterion ID, proposal path, accepted baseline and
original pin; do not replace the criterion after seeing the candidate.

After the independent checks above, you may create a **new session assessment**
outside the repository. This is permitted report writing, not candidate repair.
Bind it to the rubric blob/criterion ID, accepted baseline revision, proposal
path and exact handed-off proposal blob; identify your independent evaluator
session. Record all six check groups (behavior/integrity, syntax compatibility,
development suites, formatting, Clippy, export consistency) as `passed` or
`failed` with actual commands/results, or `unresolved` with a nonblank reason.
The core gate does not run Clippy or external checks. Missing evidence is not
passed; a known failure must be recorded even when usefulness is unknown.

Independently assess the frozen criterion as `demonstrated` or
`no_demonstrated_benefit` with real nonblank observations, or `unknown` with a
reason. Acceptance, changed-line/case/assertion counts, token skipping and
proposer claims are not demonstrated usefulness. Do not copy a proposer's
self-assessment into an independent result. The role/session declaration
records provenance, not cryptographic proof of independence; you remain
responsible for genuine checks and evidence.

Leave optional scoring variables unset when evidence is unavailable; legacy
evaluation remains valid and reports unscored usefulness, not zero. A pinned
rubric alone is also valid and unscored. Supplied malformed, stale, contradictory
or provenance-mismatched inputs must surface explicit errors; never drop them
and retry as a successful unscored fallback. A claim of passed integrity/legacy
checks that contradicts the core gate is an input error, not usable evidence.

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
- Optionally, `NUC_PROPOSAL_RUBRIC` to the absolute external session JSON path
  and `NUC_PROPOSAL_RUBRIC_REVISION` to its caller-frozen full Git blob ID
  (`git hash-object --no-filters`). They must be supplied together.
- Optionally, `NUC_PROPOSAL_ASSESSMENT` to your new absolute external assessment
  JSON path; this requires the pinned rubric. Confirm its proposal blob still
  matches the frozen handoff. Unset absent optional variables, not empty strings.

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

The report's `advisory_score` separately records usefulness status/basis,
input blobs, independent-check eligibility, raw elapsed evaluator seconds and
scope, formula/metric, and reward or explicit unscored reason. A genuine core
rejection scores -1 regardless of usefulness; a blocked core result is unscored.
With a passed core, independent failure scores -1, unresolved checks remain
unscored, and only independently valid demonstrated usefulness earns
`1 - 0.1 * t / (t + 1)` in `[0.9, 1]`; assessed no benefit is neutral 0.
Missing/unknown usefulness is unscored, not zero. There is no token-savings
bonus or token-skip cost credit.

`t` is measured by `Instant` around actual deterministic evaluator work:
input validation, source loading, legacy/candidate checks and blob stability.
It excludes compilation, external suites/Clippy, proposal/model time, score
calculation and report writing. Do not describe this nondeterministic time as
language runtime performance, total proposal cost or model/API spending.
Early infrastructure/input failures produce no fabricated measured report.
The top-level `decision` is still the deterministic core gate's result, not a
replacement for your overall independent decision. External failures still
reject and missing independent prerequisites still block, regardless of score.
Scoring never changes category selection, acceptance or token policy; no
learner, adaptive strategy or model training is implemented.

## Return the evidence

Return the proposal and baseline revisions, execution mode, confirmed category,
runtime applicability and rationale, decision, report path, independent
checks including syntax compatibility, and actual ran/skipped status with
reason. When tokens ran, include counts of `less`/`same`/`more`; include each
increased, missing, unpaired, invalid or syntax-incompatible case with its
before/after counts or reason. Keep the candidate and baseline unchanged.
Return the advisory metric/formula, usefulness basis and rubric/assessment
provenance, independent-check eligibility, raw elapsed seconds/scope, and
base/adjusted/final reward or the legitimate unscored reason. Disclose timing
exclusions and missing evidence instead of manufacturing scores.
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
