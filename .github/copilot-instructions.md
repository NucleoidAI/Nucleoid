# Nucleoid

Nucleoid is a next-gen Logic Programming Language focused on LLMs.

## Goals

Design a new grammar with minimum tokenized syntax — a superset of Python, JavaScript/TypeScript, Kotlin, Go, Rust, Java, C# and C/C++.

However, new syntax may be introduced where it is needed to achieve that goal. The superset is a starting point, not a hard constraint.

Build a Rust implementation of Nucleoid. The crate is at the repository root (`Cargo.toml`, `src/`), and `nucleoid.spec.md` defines the behaviour it must implement.

## References

- `nucleoid.spec.md` — the main reference for runtime behaviours
- `dataset/nucleoid.spec.synth.*.md` — synthesized fine-tuning use cases derived from `nucleoid.spec.md`;
  they are dataset material, not executable tests
- `docs/` — NUC language proposals in PEP style; accepted, implemented behavior forms the Nucleoid Language Reference, in the spirit of https://docs.python.org

The runtime is built around a dependency graph, scope chain, propagation queue, transactions and statement nodes.

Model the runtime with concrete types — enums for closed sets (node kinds, runtime values, error kinds), dedicated structs for graph nodes, scopes, stack frames and transactions, newtypes for identifiers and node keys, and a typed error enum returned through `Result`. No stringly-typed maps or dynamic catch-all value as the internal representation.

Prefer popular, well-maintained crates over hand-rolled infrastructure — lexing/parsing, error types and diagnostics, graph storage, ordered maps, numerics, serialization, the CLI. Pick the mainstream choice for each slot, one crate per slot, and note why when adding it to `Cargo.toml`. Nucleoid's own runtime semantics stay hand-written, and third-party representations are wrapped in the crate's own types rather than leaking through the interpreter.

## Open source

Nucleoid is published as open source under Apache-2.0, so everything is public-facing and follows mainstream Rust community practice rather than local invention: default `cargo fmt`, `cargo clippy` clean with warnings denied in CI, library in `src/lib.rs` with a thin binary, integration tests in `tests/`, examples in `examples/`, complete `Cargo.toml` metadata (`description`, `license`, `repository`, `readme`, `keywords`, `categories`, `rust-version`) before publishing, and semantic versioning on the public API. When a convention exists, default to it and flag any deliberate divergence.

`nucleoid.spec.md` is authoritative. Where the synthesized sources or `docs/` disagree with it, the main reference wins.

`nucleoid.spec.md`, the synthesized sources in `dataset/`, `docs/`, the generated JSONL and the Rust crate (`Cargo.toml`, `src/`) must stay in sync. Every change to one must be propagated to the others as part of the same change.

`dataset/` contains the synthesized Markdown sources and the Hugging Face JSONL publication of `nucleoid.spec.md` and those sources. The JSONL is rendered, never hand-edited: change the documents and regenerate with `UPDATE_DATASET=1 cargo test --test dataset`.

The crate executes its normative documents rather than restating them: `tests/nucleoid.spec.rs` exports `dataset/nucleoid.spec.jsonl` as individually named tests that embed their own code and expected return, while `tests/docs.rs` runs `tests/reference.md` (the executable form of `docs/README.md`) and the ```nuc blocks in `README.md` and `docs/examples.md`. Synthesized use cases are not runtime tests. `tests/dataset.rs` only renders and validates the Hugging Face dataset from `nucleoid.spec.md` and `dataset/nucleoid.spec.synth.*.md`, failing when committed JSONL differs from its source documents. Adding a case to a normative executable document adds a test; adding a synthesized case only updates the fine-tuning dataset.

A case whose assertions sit on a branch that is never taken proves nothing, so the suites also compare `Runtime::assertions_run()` against the number of `assert` calls in the source and fail when fewer ran.

## Language proposals

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

## Leave the world better than you found it

When adding a feature, review the code you touch and apply current Rust best practices where needed. Adopt improved idioms and methods incrementally, including ownership and borrowing, concrete types, typed errors, and maintainable APIs, so the codebase improves continuously.

Keep improvements focused on touched code and directly related helpers, proportional to the feature. Preserve intended behavior, respect public API semantic versioning, and cover affected behavior with appropriate tests. Avoid unrelated rewrites, and follow the existing `cargo fmt` and warning-denied `cargo clippy` conventions.

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

This is a Copilot-session policy, not a filesystem watcher or GitHub Actions
trigger. Do not claim that an edit outside a Copilot session launched an agent.
