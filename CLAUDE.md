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

`nucleoid.spec.md` is authoritative. Where `synth/`, `docs/` or `ref/` disagrees with it, the main reference wins.

`nucleoid.spec.md`, `synth/`, `docs/` and the Rust crate (`Cargo.toml`, `src/`) must stay in sync. Every change to one must be propagated to the others as part of the same change.

`ref/` is **frozen**. Read it for reference, never modify it.
