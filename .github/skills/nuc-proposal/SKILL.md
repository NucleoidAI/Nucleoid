---
name: nuc-proposal
description: Draft or revise a Nucleoid NUC and prototype behavior-preserving syntax. Use for proposal preparation, not acceptance evaluation.
---

# Preparing a NUC proposal

When coordinating a NUC revision, launch a fresh `nuc-proposal` agent using
[its profile](../../agents/nuc-proposal.agent.md) and this skill. If named
custom agents are unavailable, give both documents to a fresh general-purpose
agent. Inside that agent, execute the steps below; do not spawn another agent.

Follow `.github/copilot-instructions.md` and `docs/nuc-0001.md`.
Work on exactly one proposal revision in the caller's isolated candidate.

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

## Draft and prototype

Draft or revise the NUC using its required format and sections. Trace implemented
claims to `nucleoid.spec.md`; draft claims do not override the accepted language.
Keep dependencies already accepted and do not combine pending proposals.

Implement the candidate syntax where needed, then rewrite applicable cases in
`nucleoid.spec.md`, `tests/reference.md`, and the `nuc` blocks in `README.md`
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
proposal), accepted baseline revision and path, candidate worktree, changed
files, development checks, and explicit blockers. Freeze the candidate after
handoff. The caller starts a fresh `nuc-evaluate` agent with this handoff and
a fresh report path; do not launch it yourself. The caller owns integration
and evaluation status/index updates. The proposal agent must not commit, push,
or merge.

## Record rejected outcomes (caller only)

After the independent evaluator returns `Rejected`, the coordinating caller
must record and commit the rejected proposal documentation and its index/outcome
metadata. Rejected proposals are not exempt from commits. Follow the
[shared rejection workflow](../../instructions/proposals.instructions.md#record-rejected-outcomes-caller-only).

Commit only rejection documentation, never the rejected candidate's language
changes or session baseline/report artifacts. Preserve existing accepted NUC
text and unrelated work. Proposal and evaluation agents must not perform this
caller-owned commit.
