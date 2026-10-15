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
evaluation agent independently checks the frozen handoff and measures tokens;
it cannot rewrite the candidate. Only the caller orchestrates the two stages.

Record an immutable accepted-syntax baseline before rewriting the runtime or
executable documents. Evaluate each proposal in its own candidate worktree;
do not reuse agents or combine pending proposals. Preserve case identities,
canonical assertions, expected results, and executed assertion coverage.
New cases require behavior-equivalent accepted-syntax baselines.

`tests/proposals.rs` counts tokens with the runtime lexer, including layout
markers and excluding `Eof`. Accept only if every case uses the same or fewer
tokens and behavior checks pass; reject if any case uses more. Savings in
another case cannot offset an increase. Missing baselines block acceptance.
Keep reports in the session workspace and do not apply rejected candidates.
Evaluation outcome metadata does not recursively trigger another agent.

## Detailed proposal workflow

Follow the process in `docs/nuc-0001.md`.
`nucleoid.spec.md` remains authoritative for accepted runtime behavior.

Every added or updated `docs/nuc-XXXX.md` is a proposal revision. Before
finishing a Copilot task that changes a NUC, launch a fresh `nuc-proposal`
agent and then an independent fresh `nuc-evaluate` agent for each changed
proposal. Do not reuse either agent for another role, proposal, or later
revision. This applies to process and documentation-only NUCs too; unchanged
executable cases can demonstrate equal token usage.

Pin the accepted Git revision and record its token baseline before editing
runtime behavior or executable cases. Give the proposer its requested change,
proposal path, baseline revision and immutable JSON artifact, and candidate
worktree. Use `.github/agents/nuc-proposal.agent.md` and
`.github/skills/nuc-proposal/SKILL.md` for drafting, prototyping, equivalent
case rewrites, synchronization, and development checks. It must not run the
acceptance gate or mark the proposal Accepted or Rejected.

After the proposer returns the proposal's Git blob revision and completed
handoff, freeze the candidate and launch the evaluator with that handoff,
baseline, worktree, and a fresh report path. Use
`.github/agents/nuc-evaluate.agent.md` and
`.github/skills/nuc-evaluate/SKILL.md`. The evaluator independently runs
checks and `evaluate_proposal`; it cannot repair sources, regenerate exports,
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
New cases need behavior-equivalent accepted-syntax baselines, executed before
the candidate runtime is changed.

Use `tests/proposals.rs`, not estimates or LLM token usage. It counts the
trimmed case source using the runtime lexer, including `Newline`, `Indent`,
and `Dedent`, excluding only `Eof`. Accept only when every paired case uses
the same or fewer tokens and all behavior checks pass. Any case using more
tokens rejects the proposal, even if other cases save tokens. Missing
baselines block acceptance.

Keep baseline and report artifacts in the session workspace, not the
repository. Report the decision and per-case before/after counts. Apply
candidate language changes only after acceptance, keeping the specification,
runtime, reference, synthesized sources, and rendered exports synchronized.
Outcome metadata, such as updating a status or index after evaluation, belongs
to that evaluation and must not recursively launch another agent.

## Record rejected outcomes (caller only)

- After the independent evaluator's overall `Rejected` decision, the coordinating caller must record and commit the rejected proposal documentation and its index/outcome metadata.
- For a new proposal, record `Status: Rejected`, the rejection reason and actual per-case before/after counts, and update its NUC index entries.
- For a rejected revision of an already accepted or active NUC, preserve its accepted text and status; record only the rejected revision's outcome metadata rather than replacing it with the candidate.
- Commit only rejection documentation and directly related index/status metadata. Never apply or commit rejected runtime, executable-document, reference, synthesized-data or generated-export changes. Baseline and report artifacts stay in the session workspace.
- Keep rejected implementation changes isolated in the candidate worktree; do not apply its full diff merely to record the NUC.
- Stage explicit proposal/index paths and use a path-limited commit. Verify the resulting commit contains only the rejection documentation and directly related metadata. Never use repository-wide staging or `git commit -a`, or include unrelated files already staged.
- Preserve unrelated staged and unstaged changes. If the rejection documentation is already committed, verify and report that commit instead of creating an empty duplicate.
- Proposal and evaluation agents must not commit, push or merge; the rejection commit belongs solely to the coordinating caller and does not authorize an automatic push.

## Continuous proposal cycles (caller only)

After finalizing and committing an `Accepted` or `Rejected` outcome for a new
NUC, the coordinating caller immediately starts a distinct new proposal under
the [continuous-cycle rule](../skills/nuc-proposal/SKILL.md#continue-with-the-next-nuc-caller-only).
Run cycles sequentially with fresh agents, baselines, candidate worktrees and
reports, preserving the selection, compatibility and acceptance requirements.
Only the caller continues; workers stop at their handoffs. This does not
recursively reevaluate the completed NUC's outcome metadata. Stop or pause
under the conditions defined by the skill rather than bypassing a blocker.

Continuous cycles, including those coordinated through `/nuc-evaluate`, are
non-interactive. Neither the caller nor workers ask questions or request
confirmation to record an outcome or start the next NUC. Use the existing
selection rules and verified inputs; a rejection is not a reason to interrupt
continuation. If a required input cannot be obtained or another blocking
condition occurs, report it and pause without asking a question. Never invent
baselines, bypass checks or required tool approvals, or ignore a user stop.

This is a Copilot-session policy, not a filesystem watcher or GitHub Actions
trigger. Do not claim that an edit outside a Copilot session launched an agent.
