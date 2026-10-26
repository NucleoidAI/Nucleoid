---
name: nuc-proposal
description: Draft or revise a Nucleoid NUC and prototype its changes. Use for proposal preparation, not acceptance evaluation.
---

# Preparing a NUC proposal

When coordinating a NUC revision, launch a fresh `nuc-proposal` agent using
[its profile](../../agents/nuc-proposal.agent.md) and this skill. If named
custom agents are unavailable, give both documents to a fresh general-purpose
agent. Inside that agent, execute the steps below; do not spawn another agent.

Follow `.github/copilot-instructions.md` and `docs/nuc-0001.md`.
Work on exactly one proposal revision in the caller's isolated candidate.

## Choose execution mode (caller only)

`/nuc-proposal` defaults to continuous mode for open-ended proposal requests.
If the user requests one specific NUC by number or path, one concrete feature,
or a particular revision, use single-proposal mode instead. Complete that
proposal's preparation, independent evaluation, and outcome recording, then
stop; do not select or start another proposal unless the user explicitly
requests continuation.

Preserve this mode through evaluation, including coordination through
`/nuc-evaluate`. Do not run reinforcement-learning policy checks for a
single-proposal request. Immutable baselines, syntax compatibility, per-case
token acceptance, and required development checks still apply in both modes.

## Classify every proposal

Give every new and existing NUC exactly one `Category` header from the
[proposal category registry](../../../docs/nuc-0001.md#proposal-categories),
including Draft, Accepted, Rejected, Active, and Superseded documents.
Category describes the proposal's purpose; it does not replace `Type` or
change its status.

Use the user's explicit classification when supplied. Whenever a category is
added, selected, or updated, the coordinating caller must ask the user for
confirmation if anything is uncertain, including its meaning, automatic-selection
eligibility, or which proposals belong to it. Ask before proceeding, not after
guessing. Workers return the question to the caller. This clarification also
applies during continuous cycles.

The user may add categories in the future. Every category addition requires
reviewing all existing NUCs, across every status, and reclassifying proposals
that fit the new category more accurately. Keep unaffected classifications,
`Type`, `Status`, and historical outcomes unchanged. Synchronize the registry,
affected proposal headers, and both category and numerical indexes through the
normal proposal/evaluation process. Ask for confirmation on uncertain moves;
do not invent categories or retain stale classifications without review.

## Select an unspecified enhancement

When asked to choose a language enhancement without a specific feature or
category, the coordinating caller makes one unbiased random draw with equal
probability across the registry's automatically selectable enhancement
categories. The current four enhancement categories each have a 25% chance;
`Documentation` is not part of this automatic language-enhancement draw.

Read the current registry before drawing; do not hard-code its size or retain
the former 50/50 syntax/runtime split. For example, populate `$categories`
from the eligible registry entries and use
`Get-Random -Minimum 0 -Maximum $categories.Count` to select an index.
Choose one bounded enhancement in that category; do not redraw to favor a
preferred category or substitute deterministic alternation for randomness.

Select before recording baselines or delegating preparation. Supply the
selected category, requested change, and a brief selection rationale to the
proposal agent, which must not redraw. Include the category and rationale in
its handoff and write the same category in the proposal header. Explicit
feature/category requests take precedence, and process or documentation-only
revisions are not randomized.

Every category retains the same baseline and acceptance requirements. New
measured cases still need pre-executed behavior-equivalent accepted-syntax
programs; missing baselines block preparation rather than bypass the gate.

## Confirm the inputs

Require the proposal path, confirmed category and requested change, accepted
Git revision, immutable baseline JSON artifact, and candidate worktree.
Confirm that the baseline belongs to that accepted revision before making
runtime or executable document edits. Missing inputs stop preparation, not
justify assumptions.

New measured cases need behavior-equivalent accepted-syntax programs with the
same document, title, canonical assertions, and expected return. They must
already have been executed by the accepted runtime and included through
`NUC_PROPOSAL_EXTRA_BASELINES` when recording the baseline. If a case is not
paired, return its required baseline to the caller before proceeding. Never
overwrite a baseline or derive before-counts from the candidate lexer.

## Preserve syntax compatibility

Always preserve backward compatibility with previously accepted Nucleoid
syntax, for both syntax improvements and runtime features. Existing supported
forms must retain their parsing and runtime semantics. New syntax is additive
and optional; shorter spellings never replace or disable the existing forms.
Do not remove or reinterpret accepted syntax, or introduce keyword or grammar
collisions that break existing programs.

Keep representative legacy forms documented and covered by regression tests.
Run unchanged accepted-syntax counterparts from the pinned Git revision and
the supplied extra-baseline sources through the candidate parser and runtime.
Verify their protected assertions, explicit expected returns and executed
assertion coverage alongside the candidate forms.

Include compatibility-check results in the handoff. Resolve any known syntax
compatibility break before declaring the candidate ready for evaluation;
token savings or new functionality never justify a breaking syntax change.

## Draft and prototype

Draft or revise the NUC using its required format and sections. Trace implemented
claims to `nucleoid.spec.md`; draft claims do not override the accepted language.
Keep dependencies already accepted and do not combine pending proposals.

Implement the candidate change where needed,
then rewrite applicable cases in `nucleoid.spec.md`, `tests/reference.md`, and
the `nuc` blocks in `README.md`
and `docs/examples.md`. Preserve case identities, assertion order and canonical
assertions, expected returns, and executed assertion coverage. Do not remove
cases, weaken assertions, replace them with tautologies, or bypass branches.
A process-only proposal needs no artificial code or case rewrite.

Keep affected reference and synthesized examples synchronized. Edit source
documents, never generated JSONL or exported Rust tests. When their sources
change, regenerate using `UPDATE_DATASET=1 cargo test --test dataset` and
`UPDATE_SPEC_TESTS=1 cargo build`.

## Check and hand off

Run the affected development tests, `cargo fmt --check`, and
`cargo clippy --all-targets -- -D warnings`. Fix feature-coupled problems in
the candidate, without changing the baseline or caller work.

Do not run the ignored `evaluate_proposal` entry point, produce an acceptance
report, or mark the proposal Accepted or Rejected. Those belong to the
independent evaluator.

Return the proposal path and Git blob revision (`git hash-object` on the
proposal), accepted baseline revision and path, confirmed proposal category
and selection rationale when applicable, candidate worktree, changed files,
development checks, and explicit blockers. Freeze the candidate after handoff.
The caller starts a fresh `nuc-evaluate` agent with this handoff and
a fresh report path; do not launch it yourself. The caller owns integration
and evaluation status/index updates. The proposal agent must not commit, push,
or merge.

## Record rejected outcomes (caller only)

After the independent evaluator returns `Rejected`, the coordinating caller
must record and commit the rejected proposal documentation and its index/outcome
metadata. Rejected proposals are not exempt from commits. Follow the
[shared rejection workflow](../../instructions/proposals.instructions.md#record-rejected-outcomes-caller-only).

Commit only rejection documentation, never the rejected candidate's language
changes or session baseline/report artifacts. Keep implementation changes in
the isolated candidate; stage explicit proposal/index paths, use a path-limited
commit, and verify its file list. Preserve existing accepted NUC text and
unrelated staged and unstaged work. Proposal and evaluation agents must not
perform this caller-owned commit.

## Continue with the next NUC (caller only)

This section applies only in continuous mode. A single-proposal request stops
after its outcome under the [execution-mode rule](#choose-execution-mode-caller-only).

After a new NUC receives a final `Accepted` or `Rejected` decision, the
coordinating caller must finish its outcome before starting the next cycle:

- **Accepted:** integrate the unchanged accepted candidate, record status/index
  metadata, and complete the authorized Git commit.
- **Rejected:** record and commit only rejection documentation and index/outcome
  metadata; leave rejected language changes isolated.

Immediately start another fresh "purpose a NUC" cycle without waiting for a
new user prompt. Repeat sequentially in the active Copilot session until the
user stops, pauses or redirects the work, the session ends, or a resource limit
is reached. A `Blocked` decision, failed outcome recording, or missing required
input pauses continuation; report the blocker instead of bypassing it.

Continuous cycles do not ask for confirmation before recording an outcome or
continuing, including when coordinated through `/nuc-evaluate`. The exception
is an uncertain proposal category: the caller asks the user under
[the classification rule](#classify-every-proposal) before assigning it.
Workers return that uncertainty to the caller. Use the existing selection
rules and verified inputs. For other missing prerequisites or blockers,
report them and pause without a question; do not invent inputs or bypass
checks or required tool approvals.

Each cycle chooses a new NUC number and one bounded enhancement under the
existing selection rule. Pin the current verified accepted-language revision,
not a rejected candidate; create fresh immutable baselines, an isolated
candidate and report paths. Launch fresh proposal and evaluation agents, and
perform the required Rust improvement pass when applicable. Never reuse
workers across cycles or combine pending proposals.

Use actual independent token measurements and behavior/compatibility results
as feedback for the next enhancement. Preserve earlier artifacts and all
acceptance, compatibility and version-protection rules. This is iterative
proposal/evaluation feedback, not model training or model-weight updates.

Only the caller continues the loop; proposal and evaluation workers still stop
at their single-proposal handoffs. Starting a distinct next NUC is not recursive
reevaluation of status/index metadata. Instruction-file edits alone do not
complete a NUC or start this loop, and no filesystem watcher or background
service is implied.
