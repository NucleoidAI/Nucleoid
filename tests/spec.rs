//! A committed Rust export of every normative JSONL record.
//!
//! Each test creates one stateful runner, then passes every top-level Nucleoid
//! statement to a separate `run` call. This mirrors the reference suite while
//! preserving blocks and the specification's `# return:` expectations.
//!
//! ```text
//! cargo test --test spec                       # every behaviour
//! cargo test --test spec detects_a_circular    # one of them
//! cargo test --test spec -- --list             # what there is
//! ```

mod common;

use common::runner;

include!("spec_cases/mod.rs");

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
