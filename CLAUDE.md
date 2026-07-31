# Nucleoid

Nucleoid is a next-gen Logic Programming Language focused on LLMs.

## Goal

Design a new grammar with minimum tokenized syntax — a superset of Python, JavaScript/TypeScript, Kotlin, Go, Rust, Java, C# and C/C++.

However, new syntax may be introduced where it is needed to achieve that goal. The superset is a starting point, not a hard constraint.

## References

- `nucleoid.spec.md` — the main reference for runtime behaviours
- `synth/` — synthesized use cases, all derived from `nucleoid.spec.md`
- `docs/` — the Nucleoid Language Reference documentation, written as NUC documents in PEP style, in the spirit of https://docs.python.org
- `ref/` — technical reference (the archived JS implementation)

`nucleoid.spec.md` is authoritative. Where `synth/` or `docs/` disagrees with it, the main reference wins.

`nucleoid.spec.md`, `synth/` and `docs/` must stay in sync. Every change to one must be propagated to the others as part of the same change.

`ref/` is **frozen**. Read it for reference, never modify it.
