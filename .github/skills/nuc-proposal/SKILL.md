---
name: nuc-proposal
description: Draft or revise a Nucleoid NUC and prototype syntax improvements or runtime features. Use for proposal preparation, not acceptance evaluation.
---

# Preparing a NUC proposal

When coordinating a NUC revision, launch a fresh `nuc-proposal` agent using
[its profile](../../agents/nuc-proposal.agent.md) and this skill. If named
custom agents are unavailable, give both documents to a fresh general-purpose
agent. Inside that agent, execute the steps below; do not spawn another agent.

Follow `.github/copilot-instructions.md` and `docs/nuc-0001.md`.
Work on exactly one proposal revision in the caller's isolated candidate.

## Select an unspecified enhancement

When asked to choose a language enhancement without a specific feature or
category, the coordinating caller makes one unbiased random draw:

- **50% language syntax improvement:** simplify syntax or reduce lexer tokens
  while preserving behavior.
- **50% new language runtime feature:** add a runtime capability while
  preserving existing behavior.

Use a tool-backed draw, such as `Get-Random -Minimum 0 -Maximum 2`, with `0`
selecting syntax improvement and `1` selecting a runtime feature. Choose one
bounded enhancement in that category; do not redraw to favor a preferred
category or substitute deterministic alternation for randomness.

Within the selected category, pursue all-around language development:

- Review the current specification and recent NUCs to identify unmet needs and
  avoid repeatedly extending the same narrow feature family.
- Consider gaps across logic and reasoning, reactive dependencies, functions
  and scope, control flow, collections and strings, data modeling, and
  diagnostics. Prefer underrepresented areas when user value is comparable.
- Weigh distinct user value, broader expressiveness, and API complexity.
  Do not repeatedly choose math helpers or other closely related utility
  variants merely for easy implementation, baselines, or token savings.
- Keep math eligible when it fills a clear gap. Broaden coverage without a
  math ban, fixed rotation, quota, or category redraw; preserve every
  compatibility and acceptance requirement.

Select before recording baselines or delegating preparation. Supply the
selected category, requested change, and a brief selection rationale explaining
the unmet need and fit with recent coverage to the proposal agent, which must
not redraw. Include the category and rationale in its handoff. Explicit
feature/category requests take precedence, and process or documentation-only
revisions are not randomized.

Runtime features retain the same baseline and acceptance requirements. New
measured cases still need pre-executed behavior-equivalent accepted-syntax
programs; missing baselines block preparation rather than bypass the gate.

## Confirm the inputs

Require the proposal path and requested change, accepted Git revision,
immutable baseline JSON artifact, and candidate worktree. Confirm that the
baseline belongs to that accepted revision before making runtime or executable
document edits. Missing inputs stop preparation, not justify assumptions.

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

Implement the candidate syntax improvement or runtime feature where needed,
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
proposal), accepted baseline revision and path, selected enhancement category
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

Continuous cycles are non-interactive, including when coordinated through
`/nuc-evaluate`. Neither the caller nor workers ask questions, present choice
forms, or wait for confirmation before recording an outcome or continuing.
Use the existing selection rules and verified inputs. If a required input
cannot be obtained or another blocker occurs, report it and pause without a
question; do not invent inputs or bypass checks or required tool approvals.

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
