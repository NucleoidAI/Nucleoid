//! Runs the synthesized use cases, which are derived from `nucleoid.spec.md`.

mod common;

#[test]
fn synth_01() {
    common::run(
        "synth 01",
        include_str!("../synth/nucleoid.spec.synth.01.md"),
    );
}

#[test]
fn synth_02() {
    common::run(
        "synth 02",
        include_str!("../synth/nucleoid.spec.synth.02.md"),
    );
}
