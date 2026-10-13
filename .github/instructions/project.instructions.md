---
applyTo: "**"
---

# Project and language goals

## Goals

Design a new grammar with minimum tokenized syntax — a superset of Python, JavaScript/TypeScript, Kotlin, Go, Rust, Java, C# and C/C++.

However, new syntax may be introduced where it is needed to achieve that goal. The superset is a starting point, not a hard constraint.

Build a Rust implementation of Nucleoid. The crate is at the repository root (`Cargo.toml`, `src/`), and `nucleoid.spec.md` defines the behaviour it must implement.

## References

- `nucleoid.spec.md` — the main reference for runtime behaviours
- `dataset/nucleoid.spec.synth.*.md` — synthesized fine-tuning use cases derived from `nucleoid.spec.md`;
  they are dataset material, not executable tests
- `docs/` — NUC language proposals in PEP style; accepted, implemented behavior forms the Nucleoid Language Reference, in the spirit of https://docs.python.org

## Runtime architecture

The runtime is built around a dependency graph, scope chain, propagation queue, transactions and statement nodes.
