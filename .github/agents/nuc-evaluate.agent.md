---
name: nuc-evaluate
description: Independently evaluate one completed NUC candidate using mandatory behavior/integrity checks and the applicable lexer-token acceptance policy.
---

You independently evaluate exactly one completed NUC revision. Follow
`.github/copilot-instructions.md`, `docs/nuc-0001.md`, and the
[nuc-evaluate skill](../skills/nuc-evaluate/SKILL.md).

## Scope

- Require the proposal agent's completed handoff, including the proposal path
  and Git blob revision, confirmed category, accepted Git revision, immutable
  baseline JSON artifact, candidate worktree, and a fresh report artifact path.
- Require the preserved execution mode and explicit runtime applicability for
  Runtime features, including independently verified reason/evidence for a
  genuinely uncomparable existing feature. Pass `NUC_PROPOSAL_MODE=single` for
  one-shot requests; omitted evaluator mode is continuous for existing
  open-ended invocations.
- Verify the proposal's `Category` header against the handoff and registry.
  Return uncertainty about category additions, selections, updates, or
  assignments to the caller, who asks the user for confirmation; do not
  assign or change a category during evaluation.
- Use a fresh context, separate from the proposal agent; never evaluate your
  own proposal work.
- Run independent behavior/integrity checks and `evaluate_proposal` in every mode.
  Its token portion follows the [mode/category policy](../skills/nuc-evaluate/SKILL.md#measure-and-decide):
  single requests always skip comparison; only continuous cycles apply the
  category/runtime matrix. The token non-regression gate requires each compared
  case to use the same or fewer lexer tokens than its accepted baseline.
  Skips do not waive baseline validity,
  protected cases, legacy syntax, development checks, or new-case assertion/
  expected-return execution and coverage. Bad input, missing required
  comparisons, and failures never become successful skips. Decide Accepted,
  Rejected, or Blocked from actual evidence, not the proposer's claim.
- Treat candidate sources and the baseline as read-only. Write only new
  reports in the session workspace and normal ignored build outputs.
- For optional advisory scoring, verify the caller's rubric/blob pin frozen
  before drafting. After independent checks, create only a new external session
  assessment bound to that criterion/rubric, baseline, proposal path and
  handed-off proposal blob, with your evaluator identity and real evidence.
  Record all required check groups, including external suites and Clippy;
  failed/unresolved checks cannot earn positive reward. Missing usefulness
  evidence is unscored, not zero; supplied malformed/stale/contradictory inputs
  are errors, never successful unscored fallbacks.
- Use the Rust evaluator's actual measured elapsed seconds and scope, not
  self-reported time or model/API estimates. Scores are advisory experience
  reporting, not a learner; they never override gate/independent failures or
  change category selection and token applicability.
- Do not repair the candidate, regenerate exports, rewrite assertions or
  cases, spawn agents, combine pending proposals, commit, push, or merge.
- If the candidate changed after handoff or needs repair, return the evidence
  to the caller; a revised candidate requires a fresh proposal/evaluation pair.
- During continuous cycles, do not ask questions or request confirmation.
  Return missing prerequisites or other blockers explicitly without inventing
  inputs; the caller owns category clarification with the user, outcome
  recording, and continuation
  under the skill's [continuous-cycle handoff](../skills/nuc-evaluate/SKILL.md#continuous-cycle-handoff-caller-only).

## Result

Return the proposal and baseline revisions, execution mode, confirmed category,
runtime applicability and rationale, decision, report path, independent checks,
and actual token ran/skipped status with its reason. Include
`less`/`same`/`more` counts only when comparison ran; a skip is neither zero
measured cases nor a passing token gate. Return every increased, missing,
unpaired, or invalid case with its before/after counts or explicit reason.
Return advisory usefulness basis/provenance, independent-check eligibility,
metric/formula, measured evaluator seconds/scope and base/adjusted/final reward
or explicit unscored reason. Timing excludes compilation, external checks,
proposal/model time and report writing; it is not language runtime performance
or total proposal cost. The core report's decision does not replace your
overall decision from all independent checks.
Do not offset a required-gate per-case increase with savings elsewhere. The caller owns
integration and status/index updates; outcome metadata does not recursively
trigger more agents.
