---
applyTo: "**"
---

# Language proposals

Every added or updated `docs/nuc-XXXX.md` requires a fresh `nuc-proposal`
agent followed by a separate fresh `nuc-evaluate` agent for that revision,
including process-only changes. Follow their profiles in `.github/agents/`,
their respective `SKILL.md` files in `.github/skills/`, and the process in
`docs/nuc-0001.md`.

The proposal agent drafts and prototypes; it cannot decide acceptance. The
evaluation agent independently checks the frozen handoff and measures tokens
when required by the execution-mode policy; it cannot rewrite the candidate.
Only the caller orchestrates the two stages.

Record an immutable accepted-syntax baseline before rewriting the runtime or
executable documents. Evaluate each proposal in its own candidate worktree;
do not reuse agents or combine pending proposals. Preserve case identities,
canonical assertions, expected results, and executed assertion coverage.
When token comparison is required, new cases need behavior-equivalent
accepted-syntax baselines. A legitimate skip permits genuinely new behavior
without a counterpart, but still executes assertions/explicit expected returns
and verifies coverage. Pre-existing cases and legacy syntax stay protected.

`tests/proposals.rs` counts tokens with the runtime lexer, including layout
markers and excluding `Eof`. When required, accept only if every paired case
uses the same or fewer tokens; reject any increase, regardless of savings
elsewhere. Missing required comparisons block acceptance. All modes require
valid immutable accepted-corpus inputs and behavior/integrity, compatibility,
and development checks.
Keep reports in the session workspace and do not apply rejected candidates.
Evaluation outcome metadata does not recursively trigger another agent.

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
all mode/category/compatibility safeguards. This is advisory experience
reporting, not a learned proposal-selection or training mechanism.

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
finishing a Copilot task that changes a NUC, launch a fresh `nuc-proposal`
agent and then an independent fresh `nuc-evaluate` agent for each changed
proposal. Do not reuse either agent for another role, proposal, or later
revision. This applies to process and documentation-only NUCs too; they still
undergo independent behavior, integrity, compatibility, and development checks.

Pin the accepted Git revision and record its token baseline before editing
runtime behavior or executable cases. Give the proposer its requested change,
proposal path and confirmed category, execution mode and runtime applicability
with supporting rationale, baseline revision and immutable JSON artifact,
and candidate worktree. Use `.github/agents/nuc-proposal.agent.md` and
`.github/skills/nuc-proposal/SKILL.md` for drafting, prototyping, equivalent
case rewrites, synchronization, and development checks. It must not run the
acceptance gate or mark the proposal Accepted or Rejected.

Complete the caller-owned Rust improvement pass, when required, before the
final frozen handoff. After the proposer returns the proposal's Git blob
revision and completed handoff, freeze the candidate and launch the evaluator with that handoff,
baseline, worktree, and a fresh report path. Use
`.github/agents/nuc-evaluate.agent.md` and
`.github/skills/nuc-evaluate/SKILL.md`. The evaluator independently runs
checks and `evaluate_proposal`, whose token portion follows the mode/category
policy; it cannot repair sources, regenerate exports,
or change the baseline. Only it returns the acceptance decision.

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
paired case must use the same or fewer tokens. Any increase rejects the
proposal, even if other cases save tokens; missing comparisons block acceptance.
All modes require passing behavior/integrity, syntax compatibility, and
development checks. Skipping token comparison never suppresses those failures.

Keep baseline and report artifacts in the session workspace, not the
repository. Report the decision and actual ran/skipped token-check status;
include per-case before/after counts only when comparison ran. Apply
candidate language changes only after acceptance, keeping the specification,
runtime, reference, synthesized sources, and rendered exports synchronized.
Outcome metadata, such as updating a status or index after evaluation, belongs
to that evaluation and must not recursively launch another agent.

## Record rejected outcomes (caller only)

- After the independent evaluator's overall `Rejected` decision, the coordinating caller must record and commit the rejected proposal documentation and its index/outcome metadata.
- For a new proposal, record `Status: Rejected`, the rejection reason, actual token-check ran/skipped status (per-case before/after counts only when run), and update its NUC index entries.
- For a rejected revision of an already accepted or active NUC, preserve its accepted text and status; record only the rejected revision's outcome metadata rather than replacing it with the candidate.
- Commit only rejection documentation and directly related index/status metadata. Never apply or commit rejected runtime, executable-document, reference, synthesized-data or generated-export changes. Baseline and report artifacts stay in the session workspace.
- Keep rejected implementation changes isolated in the candidate worktree; do not apply its full diff merely to record the NUC.
- Stage explicit proposal/index paths and use a path-limited commit. Verify the resulting commit contains only the rejection documentation and directly related metadata. Never use repository-wide staging or `git commit -a`, or include unrelated files already staged.
- Preserve unrelated staged and unstaged changes. If the rejection documentation is already committed, verify and report that commit instead of creating an empty duplicate.
- Proposal and evaluation agents must not commit, push or merge; the rejection commit belongs solely to the coordinating caller and does not authorize an automatic push.

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

In continuous mode, after finalizing and committing an `Accepted` or `Rejected`
outcome for a new NUC, the caller immediately starts a distinct new proposal under
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

This is a Copilot-session policy, not a filesystem watcher or GitHub Actions
trigger. Do not claim that an edit outside a Copilot session launched an agent.
