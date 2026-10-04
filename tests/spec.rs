//! A committed Rust export of every normative JSONL record.
//!
//! Each test embeds its description, Nucleoid source and expected return. This
//! gives failures BDD-style behavior names without Gherkin or runtime document
//! parsing. The final test keeps the committed export equal to the JSONL.
//!
//! ```text
//! cargo test --test spec                       # every behaviour
//! cargo test --test spec detects_a_circular    # one of them
//! cargo test --test spec -- --list             # what there is
//! ```

mod common;

include!("spec_cases/mod.rs");

/// Runs one embedded dataset record. Called by its individual Rust test.
fn case(id: &str, description: &str, code: &str, expected: Option<&str>) {
    let case = common::Case {
        title: description.to_string(),
        source: code.to_string(),
        expected: expected.map(str::to_string),
    };

    if let Err(reason) = common::check(&case) {
        panic!(
            "{id}: {description}
  {reason}"
        );
    }
}

/// The committed tests must not drift from the JSONL that generated them.
#[test]
fn committed_tests_match_the_dataset() {
    assert_eq!(
        include_str!("spec_cases/mod.rs").replace("\r\n", "\n"),
        include_str!(concat!(env!("OUT_DIR"), "/spec_cases.rs")),
        "tests/spec_cases/mod.rs is stale; regenerate with \
         UPDATE_SPEC_TESTS=1 cargo build"
    );
}
