//! The JSONL dataset in `dataset/`, held equal to the documents it comes from.
//!
//! The dataset is published to Hugging Face, so it is not written by hand: it
//! is rendered from `nucleoid.spec.md` and `synth/`, and this suite fails when
//! the committed files are not what those documents render to. Regenerate with
//!
//! ```text
//! UPDATE_DATASET=1 cargo test --test dataset
//! ```
//!
//! which is the only supported way to change anything under `dataset/`.

mod common;

use std::fs;
use std::path::PathBuf;

use nucleoid::Runtime;
use serde::Serialize;

/// One case as the dataset publishes it.
#[derive(Serialize)]
struct Record {
    id: String,
    source: String,
    title: String,
    code: String,
    returns: Option<String>,
}

include!(concat!(env!("OUT_DIR"), "/synth_documents.rs"));

/// A case document and the JSONL file rendered from it.
struct Set {
    /// Prefix of the record ids, so an id says where it came from.
    prefix: String,
    /// The document, as its path is quoted in the records.
    document: String,
    output: String,
    text: &'static str,
}

/// The spec, then every synth set `build.rs` found in `synth/`.
fn sets() -> Vec<Set> {
    let mut sets = vec![Set {
        prefix: "spec".to_string(),
        document: "nucleoid.spec.md".to_string(),
        output: "dataset/spec.jsonl".to_string(),
        text: include_str!("../nucleoid.spec.md"),
    }];

    for (module, path, text) in SYNTH_DOCUMENTS {
        let number = module.trim_start_matches("synth_");

        sets.push(Set {
            prefix: format!("synth-{number}"),
            document: (*path).to_string(),
            output: format!("dataset/synth.{number}.jsonl"),
            text,
        });
    }

    sets
}

/// The dataset is what the documents say, so it is rendered rather than
/// edited. A file that differs from its document fails here.
#[test]
fn the_dataset_matches_the_documents() {
    let updating = std::env::var_os("UPDATE_DATASET").is_some();
    let mut stale: Vec<String> = Vec::new();

    for set in &sets() {
        let rendered = render(set);
        let path = path(&set.output);

        if updating {
            fs::write(&path, &rendered).expect("the dataset must be writable");
            continue;
        }

        match fs::read_to_string(&path) {
            Ok(committed) if committed.replace("\r\n", "\n") == rendered => {}
            Ok(_) => stale.push(format!(
                "{} is not what {} renders to",
                set.output, set.document
            )),
            Err(error) => stale.push(format!("{}: {error}", set.output)),
        }
    }

    assert!(
        stale.is_empty(),
        "the dataset is out of date with the documents it comes from:\n  {}\n\
         regenerate with `UPDATE_DATASET=1 cargo test --test dataset`",
        stale.join("\n  ")
    );
}

/// A dataset of programs that do not parse would be worse than none. Every
/// record is checked, which also catches a rendering that mangles the source.
#[test]
fn every_record_is_nucleoid() {
    let mut records = 0;

    for set in &sets() {
        for case in common::cases(set.text) {
            let code = code(&case);

            if let Err(error) = Runtime::check(&code) {
                panic!("{} does not parse: {error}\n{code}", case.title);
            }

            records += 1;
        }
    }

    // Two empty datasets would agree with each other and prove nothing.
    assert_eq!(
        records, 1185,
        "the documents hold 1185 cases; update this count deliberately when one is added"
    );
}

/// Renders a document as JSONL, one record per line.
fn render(set: &Set) -> String {
    let mut out = String::new();

    for (index, case) in common::cases(set.text).iter().enumerate() {
        let record = Record {
            id: format!("{}-{:04}", set.prefix, index + 1),
            source: set.document.clone(),
            title: case.title.clone(),
            code: code(case),
            returns: case.expected.clone(),
        };

        out.push_str(&serde_json::to_string(&record).expect("a record serialises"));
        out.push('\n');
    }

    out
}

/// The program as the suites run it, without the comment that titles it — the
/// title is a field of its own, and a model trained on this should not learn to
/// write it back.
fn code(case: &common::Case) -> String {
    let mut lines: Vec<&str> = case.source.lines().collect();

    // The same line `cases` took the title from.
    if let Some(position) = lines.iter().position(|line| line.trim().starts_with('#')) {
        lines.remove(position);
    }

    lines.join("\n").trim().to_string()
}

fn path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}
