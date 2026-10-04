#![allow(clippy::approx_constant)]

//! The synthesized JSONL records run as one embedded test per behavior.
//!
//! Each JSONL file is its own module. Every generated test creates one stateful
//! runner, then passes each top-level statement to a separate `run` call.

mod common;

include!(concat!(env!("OUT_DIR"), "/synth.rs"));

/// The JSONL export contains the expected number of synthesized behaviors.
#[test]
fn every_record_is_exported() {
    assert_eq!(
        GENERATED, 1007,
        "the synth dataset changed; update this count deliberately"
    );
}
