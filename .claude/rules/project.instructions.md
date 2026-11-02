---
applyTo: "**"
---

# Project and language goals

## Environment-file access

Never read `.env` files. This includes direct reads, file-content searches,
bulk reads, and indirect access through commands, scripts, or tools. Exclude
these files from content searches and use documented variable names or
placeholders instead.

## Goals

Design a minimal core grammar with minimum tokenized syntax, while aspiring to
be a superset of programming languages, including Python, JavaScript/TypeScript,
Kotlin, Go, Rust, Java, C# and C/C++. These are examples, not a fixed boundary.
This is a goal, not a claim of full compatibility with every language.

Welcome additive cross-language aliases and equivalent familiar source forms,
such as the already accepted `this`/`self` and `=>`/`->`, under the Language
compatibility category. They need not reduce or preserve lexer-token counts;
new syntax and token kinds may be introduced when appropriate. Keep every
accepted form and meaning. The superset aspiration is not a hard constraint.

Build a Rust implementation of Nucleoid. The crate is at the repository root (`Cargo.toml`, `src/`), and `nucleoid.spec.md` defines the behaviour it must implement.

## References

- `nucleoid.spec.md` — the main reference for runtime behaviours
- `dataset/nucleoid.synth.*.jsonl` — canonical synthesized fine-tuning records derived from `nucleoid.spec.md`;
  they are dataset material, not executable tests
- `arc/src/instruct_dataset/` — ARC instruction examples, maintained within the ARC project
- `dataset/` — `.jsonl` files only, with no duplicate Markdown sources or dataset cards elsewhere; do not recreate `docs/dataset/`
- `docs/` — NUC language proposals in PEP style; accepted, implemented behavior forms the Nucleoid Language Reference, in the spirit of https://docs.python.org

## Runtime architecture

The runtime is built around a dependency graph, scope chain, propagation queue, transactions and statement nodes.
