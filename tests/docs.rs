//! Executable behaviors from the language reference, README, and examples.
//!
//! `build.rs` gives every documented behavior a named Rust test while this
//! file keeps document loading, coverage checks, and cross-document checks in
//! one integration-test crate.

mod common;

use std::sync::LazyLock;

const EXAMPLES: &str = include_str!("../docs/examples.md");
const REFERENCE: &str = include_str!("../docs/README.md");

/// Reference sections that are syntax tables rather than runnable examples.
const WITHOUT_EXAMPLES: &[&str] = &["Syntax Summary"];

static DOCUMENTS: LazyLock<Vec<Vec<common::Case>>> = LazyLock::new(|| {
    vec![
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(EXAMPLES, "docs/examples.md"),
    ]
});

include!(concat!(env!("OUT_DIR"), "/docs.rs"));

fn case(document: usize, title: &str) {
    common::run_case(&DOCUMENTS, document, title);
}

#[test]
fn every_documented_behavior_is_covered() {
    assert!(
        DOCUMENTS.iter().all(|cases| !cases.is_empty()),
        "every executable document must contain at least one behavior"
    );
    assert_eq!(
        common::total(&DOCUMENTS),
        GENERATED,
        "build.rs generated {GENERATED} tests for the documented behaviors"
    );
}

#[test]
fn example_sections_follow_the_reference() {
    let documented: Vec<String> = sections(REFERENCE)
        .into_iter()
        .filter(|section| !WITHOUT_EXAMPLES.contains(&section.as_str()))
        .collect();

    assert!(
        !documented.is_empty(),
        "no numbered sections found in docs/README.md"
    );
    assert_eq!(
        sections(EXAMPLES),
        documented,
        "docs/examples.md must follow the sections of docs/README.md"
    );
}

fn sections(document: &str) -> Vec<String> {
    document
        .lines()
        .filter_map(|line| line.trim().strip_prefix("## "))
        .filter_map(|heading| heading.split_once(". "))
        .filter(|(number, _)| number.chars().all(|character| character.is_ascii_digit()))
        .map(|(_, name)| name.trim().to_string())
        .collect()
}
