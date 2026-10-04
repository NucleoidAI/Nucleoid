//! A committed Rust copy of every behaviour in `nucleoid.spec.md`.
//!
//! Each test embeds its Nucleoid source in `spec_cases/mod.rs`, like the individual
//! cases in `ref/src/test/nucleoid.spec.js`. The final synchronization test
//! keeps those committed cases identical to the authoritative document.
//!
//! ```text
//! cargo test --test spec                       # every behaviour
//! cargo test --test spec detects_a_circular    # one of them
//! cargo test --test spec -- --list             # what there is
//! ```

mod common;

struct SpecCase {
    title: &'static str,
    source: &'static str,
    expected: Option<&'static str>,
}

include!("spec_cases/mod.rs");

/// Runs one committed use case. Called by its individual Rust test.
fn case(index: usize) {
    let committed = &CASES[index];
    let case = common::Case {
        title: committed.title.to_string(),
        source: committed.source.to_string(),
        expected: committed.expected.map(str::to_string),
    };

    if let Err(reason) = common::check(&case) {
        panic!(
            "{}
  {reason}",
            committed.title
        );
    }
}

/// The editable Rust cases must not drift from the authoritative specification.
#[test]
fn committed_cases_match_the_specification() {
    let documented = common::cases(include_str!("../nucleoid.spec.md"));

    assert_eq!(documented.len(), CASES.len(), "case count differs");

    for (documented, committed) in documented.iter().zip(CASES) {
        assert_eq!(documented.title, committed.title, "case title differs");
        assert_eq!(
            documented.source, committed.source,
            "{} source differs",
            committed.title
        );
        assert_eq!(
            documented.expected.as_deref(),
            committed.expected,
            "{} expected return differs",
            committed.title
        );
    }
}
