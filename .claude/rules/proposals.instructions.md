---
applyTo: "**"
---

# Language proposals

Every added or updated `docs/nuc-XXXX.md` requires a fresh `nuc-proposal`
agent followed by a separate fresh `nuc-evaluate` agent for that revision,
including process-only changes. Follow their profiles in `.claude/agents/`,
their respective `SKILL.md` files in `.claude/skills/`, and the process in
`docs/nuc-0001.md`.

The proposal agent drafts and prototypes; it cannot decide acceptance. The
evaluation agent independently checks the frozen handoff and measures tokens
when required by the execution-mode policy; it cannot rewrite the candidate.
Only the caller orchestrates the two stages and decides the outcome under the
[automatic acceptance rule](#automatic-acceptance-caller-only): every proposal
that compiles, passes every test and stays backward compatible is accepted,
whatever the policy checks found, and those policy results are recorded in its
documentation.

Record an immutable accepted-syntax baseline before rewriting the runtime or
executable documents. Evaluate each proposal in its own candidate worktree;
do not reuse agents or combine pending proposals. Preserve case identities,
canonical assertions, expected results, and executed assertion coverage.
When token comparison is required, new cases need behavior-equivalent
accepted-syntax baselines. A legitimate skip permits genuinely new behavior
without a counterpart, but still executes assertions/explicit expected returns
and verifies coverage. Pre-existing cases and legacy syntax stay protected.

`tests/proposals.rs` counts tokens with the runtime lexer, including layout
markers and excluding `Eof`. When required, the token policy passes only if
every paired case uses the same or fewer tokens; any increase fails it,
regardless of savings elsewhere, and missing required comparisons fail it too.
A failed token policy is a recorded policy result, not a rejection. All modes
require valid immutable accepted-corpus inputs and behavior/integrity,
compatibility, and development checks.
Keep reports in the session workspace and do not apply rejected candidates.
Evaluation outcome metadata does not recursively trigger another agent.

## Automatic acceptance (caller only)

Every NUC is accepted when its frozen candidate meets all three **hard
criteria**, regardless of the policy checks (the evaluator's overall decision,
the token non-regression gate, counterpart fairness, independent behavior
findings, advisory scores and an ARC solver run). The independent evaluator
verifies them and reports each one explicitly, and the caller re-checks them
on the integrated tree before committing:

1. **No compile errors.** `cargo build --all-targets` succeeds, and so does the
   warning-denied `cargo clippy --all-targets -- -D warnings` that the
   repository treats as a build error; `cargo fmt --check` passes.
2. **Every test passes.** The full `cargo test` passes on the candidate: every
   existing test is present, unweakened and passing, including the core gate's
   replay of pinned accepted sources and supplied extra baselines, and every
   new test passes. Never edit an existing test's assertions, pinned
   diagnostics or sealed case bytes to make a candidate pass; count-only pins
   that follow from added cases may change.
3. **Always backward compatible.** Every input the accepted parser accepts,
   including forms that fail at run time, keeps its parse and behavior; every
   pre-existing case keeps its identity, canonical assertions, expected return
   and executed coverage; and no accepted output changes. Only inputs that were
   errors at the accepted revision may change, as the proposal documents.

Then decide:

- **Accepted:** the hard criteria pass and every policy check passes (the
  evaluator's overall decision is Accepted, or an
  [ARC-requested](#arc-requested-proposals-caller-only) or
  [user-requested](#user-requested-proposals-caller-only) basis applies).
- **Automatic-Accepted:** the hard criteria pass but a policy check failed. This
  adds the status `Automatic-Accepted` to NUC 1's status list. It is an accepted
  status: integrate the unchanged frozen candidate exactly as for Accepted, and
  its behavior becomes accepted language behavior.
- **Rejected:** any hard criterion fails. Record the failing criterion as the
  reason under the [rejection workflow](#record-rejected-outcomes-caller-only).
- **Blocked:** a prerequisite is missing, so the hard criteria cannot be
  verified; the NUC stays Draft and the cycle pauses.

Record the policy results in the documentation of every Accepted and
Automatic-Accepted NUC: in its Evaluation section, the evaluator's overall
decision and findings, the token check status with per-case counts when it ran,
any ARC run evidence and the advisory score, plus the hard-criteria evidence.
For Automatic-Accepted, state in the Abstract and in both NUC index entries
(`Automatic-Accepted: <failed policy check>`) which policy check failed, and
repeat it in the outcome commit body. Never repair a candidate in place to meet
a hard criterion; a revision needs a fresh proposal/evaluation pair.

A previously Rejected NUC may be revisited under this rule. Port its isolated
candidate onto the current accepted revision with a fresh single-mode
proposal/evaluation pair and the Rust improvement pass; if the ported candidate
meets every hard criterion, record it Automatic-Accepted (or Accepted when the
new evaluation passes every policy check), keeping its original evaluation and
adding the new one. If a later accepted NUC already implements its behavior,
record it Automatic-Accepted without integrating code and name that NUC. If a
hard criterion fails, for example because accepted tests pin its syntax as an
error, it stays Rejected and the outcome metadata states why. Commit each
revisited NUC separately.

## Advisory usefulness and cost evidence

Follow [NUC 1's advisory scoring policy](../../docs/nuc-0001.md#advisory-usefulness-and-evaluation-cost).
When scoring is requested, the caller freezes a create-once usefulness rubric
outside the repository and pins its full Git blob before drafting. Pass the
unchanged criterion, proposal/baseline binding, path and pin through preparation
and evaluation. The proposer does not assess its own benefit.

After independently checking the frozen candidate, the evaluator may create a
new session assessment binding its identity, criterion ID/rubric blob, accepted
baseline, proposal path and handed-off proposal blob. Require real nonblank
criterion evidence and explicit results/evidence for behavior/integrity, syntax
compatibility, development suites, formatting, Clippy and export consistency.
Known failures or unresolved checks never earn positive reward. Acceptance,
line counts, extra assertions, token skipping and proposer claims do not
demonstrate usefulness. Missing evidence is explicitly unscored, not neutral
zero; malformed/stale/contradictory supplied inputs are errors, never fallbacks.

The existing Rust evaluator measures its deterministic work with `Instant`;
exclude compilation, external checks, proposal/model time and report writing.
Return its raw seconds/scope, usefulness basis, eligibility, formula and
base/adjusted reward or unscored reason. Scoring never changes deterministic
gate results, token applicability or the independent overall decision; preserve
all mode/category/compatibility safeguards. Scoring alone reports experience;
the separate opt-in category-conditioned UCB1 strategy learner below uses the
verified reward without training LLM weights or changing this score formula.

## Reward-driven strategy learning

Follow [NUC 1](../../docs/nuc-0001.md#reward-driven-proposal-strategies) and
the [callable command/schema contract](../../CONTRIBUTING.md#reward-driven-proposal-strategies).
The fixed ordered strategies are `reuse_first` (accepted primitives/helpers
and a minimal additive implementation) and `evidence_first` (a concrete
unsupported/failing case or counterexample, then a minimal preserving
fix/extension). Both keep the requested feature/category and all mandatory
baselines/checks. The learner never reweights categories: unspecified category
selection stays uniform, while explicit requests and mode/stop rules prevail.

When configured, the caller actually runs `select_proposal_strategy` before
drafting/dispatch, using its latest retained immutable policy head or explicit
empty-history genesis. Freeze policy and receipt raw full blob pins, unique
trial ID, proposal path, accepted baseline, category, mode and chosen strategy.
The fresh proposer verifies and executes the selected approach and hands off
concrete consumption evidence and unchanged attribution. Never backfill old
reports, reselect after seeing a candidate, or silently reset invalid state.

The independent evaluator preserves the pinned policy/receipt into the actual
`evaluate_proposal` report. After checking that saved report and the complete
independent outcome, it pins the report and actually invokes the separate
`learn_proposal_strategy` command with a fresh external checkpoint path.
Reports precede checkpoints, avoiding circular hashes or multi-output pretend
success. The existing reward is validated against score/evidence and report
provenance; it is not supplied as a self-reported number.

Only genuinely scored attributed outcomes count: useful with the unchanged
time discount, assessed neutral 0, or known core/external rejection -1.
Blocked/unresolved/unscored/unknown/missing evidence and unattributed outcomes
never increment trials. `all_attributed_trials` includes both modes, even
explicit single Documentation trials; it never authorizes continuation.
Absent learning config is explicitly `not_configured`; partial, malformed,
stale, duplicate or mismatched supplied inputs are errors, never fallbacks.
Deduplicate trial IDs and actual evaluated proposal blobs, not just report
bytes/timing. Preserve all acceptance, token, compatibility and version rules.

The evaluator may write only new external session reports/checkpoints, never
candidate sources or prior artifacts. Return the new checkpoint path/raw pin
or explicit skip/error; the caller retains one head and all ancestors,
receipts, reports and scoring evidence for subsequent validation/selection.
Use that updated head for the next authorized proposal, or keep the old head
on a legitimate skip. A learner error pauses the caller as a process blocker
without changing the already-recorded gate/independent decision.

This is a single-owner immutable checkpoint chain, not concurrent/global-head
protection or cryptographic evaluator authentication/proof of pre-draft timing.
The category-conditioned two-arm bandit is real reward-driven strategy
learning, not LLM-weight training or full multi-step RL. Cold starts, small
samples, rubric/reward proxies and noisy partial timing limit conclusions;
do not claim improved proposal quality from fixtures or one trial.

## Execution mode and token policy

The **token non-regression gate** requires each compared case to use the same
or fewer lexer tokens than its accepted baseline.
`/nuc-proposal` defaults to continuous mode for open-ended requests.
Explicit single-NUC, feature, and revision requests always skip token comparison,
independently evaluate behavior/compatibility/development, record the outcome,
then stop. Preserve the caller's mode through `/nuc-evaluate`; pass
`NUC_PROPOSAL_MODE=single` explicitly. An omitted evaluator mode is `continuous`
for existing open-ended invocations.

Only continuous cycles apply the
[NUC 1 matrix](../../docs/nuc-0001.md#proposal-evaluation):

- Syntax enhancement, Utility enhancement, and Reasoning enhancement: required.
- Runtime features: required for an existing comparable feature; skipped entirely
  for a brand-new feature; skipped for a genuinely uncomparable existing feature
  only with explicit reason and evidence.
- Documentation: skipped.

Read the proposal's actual registered Category header. Runtime proposals require
explicit new/existing-comparable/existing-uncomparable applicability, including
in single mode; validate uncomparable reason/evidence independently.
Bad inputs, invalid/corrupt baselines, behavior or syntax failures, and required
token increases never become successful skips. Missing comparisons do not prove
uncomparability. Report the actual mode/category/applicability and whether the
check ran or was skipped with its reason; a skip has no measured-case counts
and is not a passing token gate.

## Enhancement selection

Every NUC, including existing and rejected proposals, has one `Category`
header from the extensible
[category registry](../../docs/nuc-0001.md#proposal-categories).
Keep its category and index entries consistent without changing its `Type`
or status. Only the user adds categories or resolves uncertainty. Whenever a
category is added, selected, or updated, the caller must ask the user for
confirmation before proceeding if its meaning, automatic-selection eligibility,
or proposal assignments are unclear, including during continuous cycles.

Every category addition also triggers a review of all existing NUCs across
every status. Reclassify proposals that fit the new category more accurately,
ask about uncertain moves, and synchronize their headers and both NUC indexes.
Preserve unaffected categories, types, statuses, and historical outcomes;
the normal independent proposal/evaluation requirements still apply.

Follow the caller-owned
[selection rule](../skills/nuc-proposal/SKILL.md#select-an-unspecified-enhancement).
Unspecified enhancements use equal random selection across the registry's
eligible categories, currently Syntax enhancement, Runtime features, Utility
enhancement, and Reasoning enhancement. Documentation is classified but is
not automatically selected for a language enhancement.
Explicit feature/category requests still take precedence.

## Detailed proposal workflow

Follow the process in `docs/nuc-0001.md`.
`nucleoid.spec.md` remains authoritative for accepted runtime behavior.

Every added or updated `docs/nuc-XXXX.md` is a proposal revision. Before
finishing an agent task that changes a NUC, launch a fresh `nuc-proposal`
agent and then an independent fresh `nuc-evaluate` agent for each changed
proposal. Do not reuse either agent for another role, proposal, or later
revision. This applies to process and documentation-only NUCs too; they still
undergo independent behavior, integrity, compatibility, and development checks.

Pin the accepted Git revision and record its token baseline before editing
runtime behavior or executable cases. Give the proposer its requested change,
proposal path and confirmed category, execution mode and runtime applicability
with supporting rationale, baseline revision and immutable JSON artifact,
and candidate worktree. When configured, also supply the pre-draft strategy
receipt and policy paths/raw pins, trial identity and actionable approach.
Use `.claude/agents/nuc-proposal.agent.md` and
`.claude/skills/nuc-proposal/SKILL.md` for drafting, prototyping, equivalent
case rewrites, synchronization, and development checks. It must not run the
acceptance gate or mark the proposal Accepted, Automatic-Accepted or Rejected.

Complete the caller-owned Rust improvement pass, when required, before the
final frozen handoff. After the proposer returns the proposal's Git blob
revision and completed handoff, freeze the candidate and launch the evaluator with that handoff,
baseline, worktree, and a fresh report path. Use
`.claude/agents/nuc-evaluate.agent.md` and
`.claude/skills/nuc-evaluate/SKILL.md`. The evaluator independently runs
checks and `evaluate_proposal`, whose token portion follows the mode/category
policy; it cannot repair sources, regenerate exports,
or change the baseline. It returns the policy decision together with explicit
evidence for each hard criterion; the caller decides the status under the
[automatic acceptance rule](#automatic-acceptance-caller-only), which also
covers [ARC-requested](#arc-requested-proposals-caller-only) and
[user-requested](#user-requested-proposals-caller-only) proposals.

If named custom agents are unavailable, give the corresponding profile,
skill, and inputs to separate fresh general-purpose agents. Neither role
spawns more agents; the caller coordinates the handoff.

Evaluate proposals separately. Agents that rewrite shared files need separate
candidate worktrees; serialize integration of accepted changes. A substantive
revision, repair, or rebase requires a new proposal/evaluation pair. Never
discard another user's changes when rejecting a candidate.

Rewrite the executable source documents, not generated Rust or JSONL. Preserve
case identities, canonical assertions, expected returns, and assertion coverage.
When the gate is required, new cases need behavior-equivalent accepted-syntax
baselines, executed before the candidate runtime is changed. Legitimately
skipped comparisons permit unpaired new behavior, never weakened old cases.
Replay pinned accepted sources and original extra-baseline sources in all modes.

Use `tests/proposals.rs`, not estimates or LLM token usage. It counts the
trimmed case source using the runtime lexer, including `Newline`, `Indent`,
and `Dedent`, excluding only `Eof`. When token comparison is required, every
paired case must use the same or fewer tokens for the token policy to pass. Any
increase fails it, even if other cases save tokens, and missing comparisons
fail it too; the caller records that policy result and applies the automatic
acceptance rule. All modes require passing behavior/integrity, syntax
compatibility, and development checks. Skipping token comparison never
suppresses those failures.

Keep baseline and report artifacts in the session workspace, not the
repository. Report the decision and actual ran/skipped token-check status;
include per-case before/after counts only when comparison ran. Apply
candidate language changes only after an Accepted or Automatic-Accepted
outcome, keeping the specification,
runtime, reference, synthesized sources, and rendered exports synchronized.
Outcome metadata, such as updating a status or index after evaluation, belongs
to that evaluation and must not recursively launch another agent.

## Outcome commit messages (caller only)

- Every commit that records a NUC outcome, Accepted, Automatic-Accepted or
  Rejected, has the subject line `NUC <number> <Title>`, using the proposal's
  `Title` header verbatim, for example `NUC 57 Thin Arrows and Python-Style
  Lambdas`.
- This applies to single and continuous `/nuc-proposal` outcomes, including
  cycles coordinated through `/nuc-evaluate`, NUCs that `/arc-purposal`
  requests and revisited Rejected NUCs.
- State the outcome in the body: `Accepted`, `Automatic-Accepted` or
  `Rejected`, its basis, the hard-criteria evidence, every failed policy check
  for Automatic-Accepted, and for a rejection the failing hard criterion that
  the rejection rules below require.

## Record rejected outcomes (caller only)

- When a candidate fails a hard criterion of the [automatic acceptance rule](#automatic-acceptance-caller-only), the coordinating caller must record and commit the rejected proposal documentation and its index/outcome metadata. An evaluator's overall `Rejected` decision alone no longer rejects a NUC whose candidate meets every hard criterion; that NUC is Automatic-Accepted.
- For a new proposal, record `Status: Rejected`, the failing hard criterion as the rejection reason, the evaluator's policy results, actual token-check ran/skipped status (per-case before/after counts only when run), and update its NUC index entries.
- For a rejected revision of an already accepted or active NUC, preserve its accepted text and status; record only the rejected revision's outcome metadata rather than replacing it with the candidate.
- Commit only rejection documentation and directly related index/status metadata. Never apply or commit rejected runtime, executable-document, reference, synthesized-data or generated-export changes. Baseline and report artifacts stay in the session workspace.
- Keep rejected implementation changes isolated in the candidate worktree; do not apply its full diff merely to record the NUC.
- Stage explicit proposal/index paths and use a path-limited commit. Verify the resulting commit contains only the rejection documentation and directly related metadata. Never use repository-wide staging or `git commit -a`, or include unrelated files already staged.
- Preserve unrelated staged and unstaged changes. If the rejection documentation is already committed, verify and report that commit instead of creating an empty duplicate.
- Proposal and evaluation agents must not commit, push or merge; the rejection commit belongs solely to the coordinating caller, who pulls before committing and pushes right after under the [git synchronization policy](git.instructions.md).

## ARC-requested proposals (caller only)

`/arc-purposal <puzzle-id>` may request one NUC when the puzzle's
`arc-dataset` handoff requires a Nucleoid enhancement. Prepare and evaluate it
as a single-proposal request: fresh proposal and evaluation agents, a pinned
baseline, an isolated candidate that changes nothing under `arc/`, the Rust
improvement pass when required, and no token comparison. Hold its outcome until
the puzzle's one solver run against the frozen candidate, as the
[arc-purposal skill](../skills/arc-purposal/SKILL.md) describes, then decide
under the [automatic acceptance rule](#automatic-acceptance-caller-only):

- A failed hard criterion: Rejected, whatever the run shows.
- Otherwise, `bun start <puzzle-id>` exits with `0`, an exact match: Accepted,
  regardless of the evaluator's decision.
- Otherwise, any other exit status, a timeout at the skill's 30-minute limit,
  or no run: Accepted when the evaluator's overall decision is Accepted, and
  Automatic-Accepted when it is not.

This exact-match basis covers only the NUC that the same invocation requested,
frozen before its run and integrated unchanged; every other NUC follows the
automatic acceptance rule directly. Record the basis, the evaluator's decision
and report, and the run's evidence with the outcome, and never hide the
evaluator's failures. The run is scored once: never rerun it, or revise the
dataset or candidate from its output, to reach a pass.

## User-requested proposals (caller only)

A NUC the user introduces manually, by requesting that specific NUC by number
or path, one concrete feature, or a particular revision, is a single-proposal
request. Prepare and evaluate it as usual: fresh proposal and evaluation
agents, a pinned baseline, an isolated candidate, the Rust improvement pass
when required, and no token comparison. The evaluator also runs the full
`cargo test` suite against the frozen candidate. Then decide under the
[automatic acceptance rule](#automatic-acceptance-caller-only):

- The candidate meets every hard criterion (it compiles without error, every
  existing test and every new test passes, and it is backward compatible):
  Accepted, regardless of the evaluator's decision.
- A failed hard criterion, such as a compile error, a failing or unrun test, or
  a compatibility break: Rejected.
- No evaluation result: Blocked; the NUC stays Draft.

Existing tests are the accepted baseline's suites and executable cases,
including the core gate's replay of pinned accepted sources. They have not
passed when the core gate reports a protected-case issue or the evaluator
finds an existing test removed or weakened. New tests are the cases and
suites the candidate adds.

This user-request basis covers only NUCs the user requested directly, not
those that continuous cycles select or `/arc-purposal` requests; every other
NUC follows the automatic acceptance rule directly. Integrate the frozen
candidate unchanged. Record the basis, the
evaluator's decision, findings and report, and the test results with the
outcome, and never hide the evaluator's failures. Never repair a failing
candidate in place to reach a pass; a revision needs a fresh
proposal/evaluation pair.

## Continuous proposal cycles (caller only)

`/nuc-proposal` defaults to continuous mode for open-ended requests. A request
for one specific NUC, concrete feature, or particular revision uses
single-proposal mode: finish its preparation, independent evaluation, and
outcome recording, then stop unless the user explicitly requests continuation.
Follow the [execution-mode rule](../skills/nuc-proposal/SKILL.md#choose-execution-mode-caller-only),
including through `/nuc-evaluate`. Do not run the token non-regression gate
for a single-proposal request: that means skipping token comparison,
not independent behavior/integrity, accepted-corpus baseline, compatibility,
development evaluation, or outcome recording.

In continuous mode, after finalizing and committing an `Accepted`,
`Automatic-Accepted` or `Rejected` outcome for a new NUC, the caller
immediately starts a distinct new proposal under
the [continuous-cycle rule](../skills/nuc-proposal/SKILL.md#continue-with-the-next-nuc-caller-only).
Run cycles sequentially with fresh agents, baselines, candidate worktrees and
reports, preserving the selection, compatibility and acceptance requirements.
Only the caller continues; workers stop at their handoffs. This does not
recursively reevaluate the completed NUC's outcome metadata. Stop or pause
under the conditions defined by the skill rather than bypassing a blocker.

Continuous cycles, including those coordinated through `/nuc-evaluate`, do not
request confirmation to record an outcome or start the next NUC. An uncertain
category addition, selection, or update is the exception: workers return it
to the caller, who asks the user for confirmation before proceeding.
Use the existing selection rules and verified inputs;
a rejection is not a reason to interrupt continuation. For other missing
prerequisites or blockers, report them and pause without asking a question.
Never invent baselines, bypass checks or required tool approvals, or ignore
a user stop.

This is an agent-session policy for Claude Code and Copilot, not a filesystem
watcher or GitHub Actions trigger. Do not claim that an edit outside an agent
session launched an agent.
