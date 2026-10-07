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
- `docs/` — the Nucleoid Language Reference documentation, written as NUC documents in PEP style, in the spirit of https://docs.python.org

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
