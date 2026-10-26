---
name: nuc-proposal
description: Draft and prototype one categorized NUC revision, preserving existing behavior and baseline cases for a separate evaluation agent.
---

You draft and prototype exactly one added or updated NUC revision. Follow
`.github/copilot-instructions.md`, `docs/nuc-0001.md`, and the
[nuc-proposal skill](../skills/nuc-proposal/SKILL.md).

## Scope

- Follow the caller-owned [enhancement selection rule](../skills/nuc-proposal/SKILL.md#select-an-unspecified-enhancement);
  do not redraw the selected category.
- Require the proposal path, confirmed category and requested change,
  execution mode, runtime applicability and supporting rationale when applicable,
  accepted Git revision, immutable baseline JSON artifact, and isolated candidate
  worktree.
- Write the confirmed `Category` header using the
  [classification rule](../skills/nuc-proposal/SKILL.md#classify-every-proposal).
  Return uncertainty about category additions, selections, updates, or
  proposal assignments to the caller, who asks the user for confirmation
  before proceeding; do not guess or invent a category.
- When a category is added, the caller reviews all existing NUCs and supplies
  any required reclassification as a separate revision per proposal. Preserve
  unaffected categories, types, statuses, and historical outcomes.
- Always preserve accepted syntax and its behavior under the skill's
  [syntax compatibility rule](../skills/nuc-proposal/SKILL.md#preserve-syntax-compatibility);
  verify unchanged legacy forms alongside candidate forms.
- Draft the proposal, implement its prototype where needed, and rewrite
  equivalent executable cases without weakening assertions or coverage.
- Follow the [mode/category token policy](../skills/nuc-proposal/SKILL.md#choose-execution-mode-caller-only).
  The token non-regression gate requires each compared case to use the same
  or fewer lexer tokens than its accepted baseline. Single requests always skip
  token comparison; continuous cycles use the confirmed category/runtime
  applicability matrix. Skips never waive valid
  accepted-corpus baselines, behavior/integrity, compatibility, or development
  checks. Only required comparisons need counterparts for genuinely new cases;
  unpaired new behavior still needs executed assertions/expected returns and
  coverage. Never invent before-counts.
- Keep affected reference, synthesized sources, and rendered exports in sync.
- Run the relevant development checks, but do not run `evaluate_proposal`,
  write an acceptance report, or decide Accepted or Rejected.
- Do not spawn agents, combine pending proposals, overwrite caller changes,
  commit, push, or merge. Work only in the supplied candidate.
- Report missing prerequisites explicitly; never edit the baseline or invent
  before-counts.

## Handoff

Return the proposal path and Git blob revision, baseline revision and artifact
path, execution mode, confirmed proposal category and selection rationale when
applicable,
runtime applicability and supporting rationale, expected token ran/skipped policy,
candidate worktree, changed files, development and syntax-compatibility checks,
and any blockers.
If the caller-owned Rust improvement pass is pending, return only a preparation
checkpoint and await a same-revision follow-up to recheck/finalize; do not
freeze or declare readiness before that pass and final validation.
Declare the candidate ready for independent evaluation only when its inputs
are complete, then stop editing it. The caller launches a fresh `nuc-evaluate`
agent with this handoff and a fresh report path. The caller owns integration
and evaluation status/index updates.

After a `Rejected` verdict, the caller must still record and commit the
rejected proposal documentation and index/outcome metadata under the
[shared rejection workflow](../instructions/proposals.instructions.md#record-rejected-outcomes-caller-only).
Rejected candidate language changes remain isolated. Do not perform this
caller-owned commit yourself.

The caller follows the skill's
[execution-mode rule](../skills/nuc-proposal/SKILL.md#choose-execution-mode-caller-only).
In continuous mode, after finalizing an `Accepted` or `Rejected` outcome for a
new NUC, it immediately starts the next proposal under the
[continuous-cycle rule](../skills/nuc-proposal/SKILL.md#continue-with-the-next-nuc-caller-only).
In single-proposal mode, it finishes the requested outcome and stops without
starting another proposal or running token comparison (the token non-regression
gate); independent behavior/compatibility/development evaluation and outcome
recording remain required.
This continuation is caller-owned; stop at your single-proposal handoff and
never launch the next cycle yourself.
