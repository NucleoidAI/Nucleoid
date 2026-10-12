---
name: nuc-proposal
description: Rewrite one NUC proposal's executable cases and accept only behavior-preserving per-case lexer token counts that do not increase.
---

You evaluate exactly one added or updated NUC revision. Follow
`.github/copilot-instructions.md` and `docs/nuc-0001.md`. Do not spawn more
agents or evaluate another proposal.

## Inputs and isolation

Require a proposal path, an accepted Git revision, an immutable baseline JSON
artifact, a candidate worktree, and a fresh report artifact path. If an input
is missing, stop and report the blocker rather than assume acceptance.

Read the proposal and its requirements. Work only on its candidate; do not
combine other pending proposals, overwrite caller changes, commit, push,
or merge. A process-only proposal needs no artificial code rewrite.

## Rewrite and preserve behavior

Implement the proposed syntax in the candidate runtime where needed, then
rewrite applicable cases in `nucleoid.spec.md`, `tests/reference.md`, and the
`nuc` blocks in `README.md` and `docs/examples.md` using the updated syntax.
Preserve every baseline case's document and title, canonical assertions,
expected returns, and executed assertion coverage. Do not remove, rename,
weaken, reorder, or replace assertions with tautologies to save tokens.

New cases must have behavior-equivalent programs in the accepted syntax with
the same case identity and assertions. Prepare these before changing the
runtime, execute them with the accepted runtime, and include them through
`NUC_PROPOSAL_EXTRA_BASELINES` when recording the baseline. Never derive a
before-count from the candidate lexer or edit an existing baseline artifact.
If the accepted language cannot express a baseline, report that the proposal
is blocked; do not invent a token comparison.

Keep directly affected reference and synthesized examples synchronized.
Regenerate JSONL with `UPDATE_DATASET=1 cargo test --test dataset`, and
export normative tests with `UPDATE_SPEC_TESTS=1 cargo build`. Never
hand-edit either generated representation.

## Measure and decide

Use the ignored `evaluate_proposal` entry point in `tests/proposals.rs` with
the supplied baseline and report paths. Follow the commands in
`CONTRIBUTING.md`. Count lexer tokens, including line and block markers but
excluding `Eof`; this is not an LLM tokenizer metric.

Run the affected executable suites, formatting, and warning-denied clippy.
The gate must preserve canonical assertions and return expectations, execute
all assertions, and pair every case with its baseline.

- **Accepted:** every case is `same` or `less`, and all behavior checks pass.
- **Rejected:** any case is `more`, or the rewrite changes or fails its tests.
- **Blocked:** a required baseline or other evaluation prerequisite is absent.

Never offset an increase with savings elsewhere or approve from a total.
Process and documentation-only proposals may be accepted with all cases
unchanged. Leave rejected candidate runtime and corpus changes out of the
accepted worktree; preserve the proposal and report for the caller.

Return the proposal revision, baseline revision, decision, report path,
counts of `less`/`same`/`more` cases, and every increased, missing, unpaired,
or invalid case with its before/after counts or explicit reason. The caller
owns integration and status/index updates; those outcome updates must not
launch another evaluation recursively.
