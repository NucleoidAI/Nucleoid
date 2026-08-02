//! `nucleoid.spec.md`, the authoritative description of the language, run as
//! one test per behaviour.
//!
//! The cases are written in Nucleoid rather than in Rust, so the tests that run
//! them are generated from the document by `build.rs` — see there for why. The
//! effect is the same as `ref/src/test/nucleoid.spec.js`: every behaviour is
//! named, reported and filtered on its own.
//!
//! ```text
//! cargo test --test spec                       # every behaviour
//! cargo test --test spec detects_a_circular    # one of them
//! cargo test --test spec -- --list             # what there is
//! ```

mod common;

use std::sync::LazyLock;

static DOCUMENTS: LazyLock<Vec<Vec<common::Case>>> =
    LazyLock::new(|| vec![common::cases(include_str!("../nucleoid.spec.md"))]);

include!(concat!(env!("OUT_DIR"), "/spec.rs"));

/// Runs the case with this title. Called by each generated test.
fn case(document: usize, title: &str) {
    common::run_case(&DOCUMENTS, document, title);
}

/// Every case in the document has a test of its own. A case that quietly
/// stopped being run would otherwise look like a passing suite.
#[test]
fn every_case_is_covered() {
    assert_eq!(
        common::total(&DOCUMENTS),
        GENERATED,
        "build.rs generated {GENERATED} tests for the cases in nucleoid.spec.md"
    );
}
