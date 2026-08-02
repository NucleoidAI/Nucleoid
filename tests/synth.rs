//! The synthesized use cases, which are derived from `nucleoid.spec.md`, run as
//! one test per behaviour.
//!
//! Each document is its own module, so a failure says which synth set it came
//! from. The documents are discovered by `build.rs` from `synth/`, so a new set
//! is picked up by dropping the file in — see there for why.

mod common;

use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/synth_documents.rs"));

static DOCUMENTS: LazyLock<Vec<Vec<common::Case>>> = LazyLock::new(|| {
    SYNTH_DOCUMENTS
        .iter()
        .map(|(_, _, text)| common::cases(text))
        .collect()
});

include!(concat!(env!("OUT_DIR"), "/synth.rs"));

/// Runs the case with this title. Called by each generated test.
fn case(document: usize, title: &str) {
    common::run_case(&DOCUMENTS, document, title);
}

/// Every case in every synth document has a test of its own.
#[test]
fn every_case_is_covered() {
    assert_eq!(
        common::total(&DOCUMENTS),
        GENERATED,
        "build.rs generated {GENERATED} tests for the cases in synth/"
    );
}
