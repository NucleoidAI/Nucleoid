---
applyTo: "**"
---

# Rust development

Model the runtime with concrete types — enums for closed sets (node kinds, runtime values, error kinds), dedicated structs for graph nodes, scopes, stack frames and transactions, newtypes for identifiers and node keys, and a typed error enum returned through `Result`. No stringly-typed maps or dynamic catch-all value as the internal representation.

Prefer popular, well-maintained crates over hand-rolled infrastructure — lexing/parsing, error types and diagnostics, graph storage, ordered maps, numerics, serialization, the CLI. Pick the mainstream choice for each slot, one crate per slot, and note why when adding it to `Cargo.toml`. Nucleoid's own runtime semantics stay hand-written, and third-party representations are wrapped in the crate's own types rather than leaking through the interpreter.

## Open source

Nucleoid is published as open source under Apache-2.0, so everything is public-facing and follows mainstream Rust community practice rather than local invention: default `cargo fmt`, `cargo clippy` clean with warnings denied in CI, library in `src/lib.rs` with a thin binary, integration tests in `tests/`, examples in `examples/`, complete `Cargo.toml` metadata (`description`, `license`, `repository`, `readme`, `keywords`, `categories`, `rust-version`) before publishing, and semantic versioning on the public API. When a convention exists, default to it and flag any deliberate divergence.
