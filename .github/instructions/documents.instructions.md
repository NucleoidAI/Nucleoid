---
applyTo: "**"
---

# Specification, executable documents, and datasets

`nucleoid.spec.md` is authoritative. Where the synthesized sources or `docs/` disagree with it, the main reference wins.

`nucleoid.spec.md`, the synthesized sources in `dataset/`, `docs/`, the generated JSONL and the Rust crate (`Cargo.toml`, `src/`) must stay in sync. Every change to one must be propagated to the others as part of the same change.

`dataset/` contains the synthesized Markdown sources and the Hugging Face JSONL publication of `nucleoid.spec.md` and those sources. The JSONL is rendered, never hand-edited: change the documents and regenerate with `UPDATE_DATASET=1 cargo test --test dataset`.

The crate executes its normative documents rather than restating them: `tests/nucleoid.spec.rs` exports `dataset/nucleoid.spec.jsonl` as individually named tests that embed their own code and expected return, while `tests/docs.rs` runs `tests/reference.md` (the executable form of `docs/README.md`) and the ```nuc blocks in `README.md` and `docs/examples.md`. Synthesized use cases are not runtime tests. `tests/dataset.rs` only renders and validates the Hugging Face dataset from `nucleoid.spec.md` and `dataset/nucleoid.spec.synth.*.md`, failing when committed JSONL differs from its source documents. Adding a case to a normative executable document adds a test; adding a synthesized case only updates the fine-tuning dataset.

A case whose assertions sit on a branch that is never taken proves nothing, so the suites also compare `Runtime::assertions_run()` against the number of `assert` calls in the source and fail when fewer ran.
