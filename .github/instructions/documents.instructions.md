---
applyTo: "**"
---

# Specification, executable documents, and datasets

`nucleoid.spec.md` is authoritative. Where synthesized training records or `docs/` disagree with it, the main reference wins.

`nucleoid.spec.md`, the training records in `dataset/`, the language reference in `docs/`, and the Rust crate (`Cargo.toml`, `src/`) must stay consistent. Propagate affected behavior changes to the relevant records and references in the same change.

`dataset/` must contain only `.jsonl` files. Do not add Markdown, README files, source documents, or subdirectories there. Do not create duplicate Markdown datasets or relocate them elsewhere: `docs/dataset/` must not exist. Keep the normative `nucleoid.spec.md` at the repository root.

`dataset/nucleoid.spec.jsonl` is generated from `nucleoid.spec.md`, never hand-edited. Regenerate that export with `UPDATE_DATASET=1 cargo test --test dataset`; regenerate its Rust tests with `UPDATE_SPEC_TESTS=1 cargo build` when necessary.

`dataset/nucleoid.synth.*.jsonl` and `dataset/arc.jsonl` are the canonical, self-contained fine-tuning records, maintained directly as JSONL. Do not regenerate them from Markdown or create Markdown source copies. Preserve their stable IDs, descriptions, programs, expected results, and ARC annotations when updating them. Dataset metadata belongs in existing project documentation, not a duplicate dataset card or case corpus.

The crate executes its normative documents rather than restating them: `tests/nucleoid.spec.rs` exports `dataset/nucleoid.spec.jsonl` as individually named tests that embed their own code and expected return, while `tests/docs.rs` runs `tests/reference.md` (the executable form of `docs/README.md`) and the ```nuc blocks in `README.md` and `docs/examples.md`. Synthesized use cases are not normative runtime tests. `tests/dataset.rs` checks the generated specification export against its source and validates the standalone synthesized and ARC JSONL records, failing on invalid records, duplicate Markdown sources, or non-JSONL entries in `dataset/`. Adding a case to a normative executable document adds a test; adding a synthesized case only updates the fine-tuning dataset.

A case whose assertions sit on a branch that is never taken proves nothing, so the suites also compare `Runtime::assertions_run()` against the number of `assert` calls in the source and fail when fewer ran.
