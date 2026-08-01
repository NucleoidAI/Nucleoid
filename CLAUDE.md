# Nucleoid

Nucleoid is a next-gen Logic Programming Language focused on LLMs.

## Goals

Design a new grammar with minimum tokenized syntax — a superset of Python, JavaScript/TypeScript, Kotlin, Go, Rust, Java, C# and C/C++.

However, new syntax may be introduced where it is needed to achieve that goal. The superset is a starting point, not a hard constraint.

Build a Rust implementation of Nucleoid. The crate is at the repository root (`Cargo.toml`, `src/`), and `nucleoid.spec.md` defines the behaviour it must implement.

## References

- `nucleoid.spec.md` — the main reference for runtime behaviours
- `synth/` — synthesized use cases, all derived from `nucleoid.spec.md`
- `docs/` — the Nucleoid Language Reference documentation, written as NUC documents in PEP style, in the spirit of https://docs.python.org
- `ref/` — technical reference for the Rust implementation (the archived JS implementation)

Use `ref/` for **how** the runtime is built — dependency graph, scope chain, stack, transactions, statement node types. Do not use it for **what** the language does: its error types and built-in names diverge from the spec deliberately.

Port `ref/`'s structure, not its dynamism. `ref/` is untyped JavaScript; the Rust crate must model the same concepts with concrete types — enums for closed sets (node kinds, runtime values, error kinds), dedicated structs for graph nodes, scopes, stack frames and transactions, newtypes for identifiers and node keys, and a typed error enum returned through `Result`. No stringly-typed maps or dynamic catch-all value as the internal representation.

Prefer popular, well-maintained crates over hand-rolled infrastructure — lexing/parsing, error types and diagnostics, graph storage, ordered maps, numerics, serialization, the CLI. Pick the mainstream choice for each slot, one crate per slot, and note why when adding it to `Cargo.toml`. Nucleoid's own runtime semantics stay hand-written, and third-party representations are wrapped in the crate's own types rather than leaking through the interpreter.

## Open source

Nucleoid is published as open source under Apache-2.0, so everything is public-facing and follows mainstream Rust community practice rather than local invention: default `cargo fmt`, `cargo clippy` clean with warnings denied in CI, library in `src/lib.rs` with a thin binary, integration tests in `tests/`, examples in `examples/`, complete `Cargo.toml` metadata (`description`, `license`, `repository`, `readme`, `keywords`, `categories`, `rust-version`) before publishing, and semantic versioning on the public API. When a convention exists, default to it and flag any deliberate divergence.

`nucleoid.spec.md` is authoritative. Where `synth/`, `docs/` or `ref/` disagrees with it, the main reference wins.

`nucleoid.spec.md`, `synth/`, `docs/` and the Rust crate (`Cargo.toml`, `src/`) must stay in sync. Every change to one must be propagated to the others as part of the same change.

`ref/` is **frozen**. Read it for reference, never modify it.
