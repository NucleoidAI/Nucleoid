//! Runs `nucleoid.spec.md`, the authoritative description of the language.

mod common;

#[test]
fn spec() {
    common::run("spec", include_str!("../nucleoid.spec.md"));
}
