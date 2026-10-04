//! The synthesized JSONL records run as one embedded test per behavior.
//!
//! Each JSONL file is its own module, and every generated test contains its
//! complete record: BDD-style names without Gherkin or Markdown parsing.

mod common;

include!(concat!(env!("OUT_DIR"), "/synth.rs"));

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

/// The JSONL export contains the expected number of synthesized behaviors.
#[test]
fn every_record_is_exported() {
    assert_eq!(
        GENERATED, 1007,
        "the synth dataset changed; update this count deliberately"
    );
}
