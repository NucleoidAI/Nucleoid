//! Runs the behaviours stated in `docs/reference.md`, kept executable here.

mod common;

#[test]
fn reference() {
    common::run("reference", include_str!("reference.md"));
}
