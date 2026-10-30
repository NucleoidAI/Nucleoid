//! The specification export and standalone JSONL training data in `dataset/`.
//!
//! Only `nucleoid.spec.jsonl` is generated from Markdown. Regenerate it with
//!
//! ```text
//! UPDATE_DATASET=1 cargo test --test dataset
//! ```
//!
//! Synthesized records are maintained directly as JSONL, without
//! duplicate Markdown sources. They are training data, not normative tests.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use nucleoid::Runtime;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// One case as the dataset publishes it.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Record {
    id: String,
    description: String,
    code: String,
    /// `return` is a Rust keyword, so the field is named for what it holds and
    /// renamed when reading and writing JSONL.
    #[serde(rename = "return")]
    returns: Option<String>,
}

const SPEC_DOCUMENT: &str = include_str!("../nucleoid.spec.md");

const SPEC_RECORDS: usize = 489;
const SYNTH_RECORDS: usize = 1_159;
const SYNTH_19_RECORDS: usize = 51;
const SYNTH_21_RECORDS: usize = 120;
const SYNTH_22_RECORDS: usize = 5;
const SYNTH_23_RECORDS: usize = 5;
const SYNTH_24_RECORDS: usize = 4;
const SYNTH_25_RECORDS: usize = 4;
const SYNTH_26_RECORDS: usize = 4;
const SYNTH_27_RECORDS: usize = 4;
const SYNTH_28_RECORDS: usize = 4;
const SYNTH_29_RECORDS: usize = 4;
const SYNTH_30_RECORDS: usize = 4;
const SYNTH_31_RECORDS: usize = 4;
const SYNTH_33_RECORDS: usize = 4;
const SYNTH_34_RECORDS: usize = 4;
const SYNTH_35_RECORDS: usize = 4;
const SYNTH_36_RECORDS: usize = 4;
const SYNTH_37_RECORDS: usize = 4;
const SYNTH_38_RECORDS: usize = 4;
const SYNTH_39_RECORDS: usize = 4;
const SYNTH_40_RECORDS: usize = 6;

#[test]
fn dataset_contains_only_jsonl_files() {
    assert!(
        !path("docs/dataset").exists(),
        "do not recreate duplicate Markdown dataset sources in docs/dataset/"
    );
    for entry in fs::read_dir(path("dataset")).expect("dataset/ must be readable") {
        let entry = entry.expect("a dataset directory entry must be readable");
        let path = entry.path();
        assert!(
            entry
                .file_type()
                .expect("a dataset entry's type must be readable")
                .is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == "jsonl"),
            "dataset/ must contain only .jsonl files; found {}",
            path.display()
        );
    }
}

#[test]
fn synthesized_records_keep_their_identifiers_and_fields() {
    let sets = synthesized_sets();
    assert!(!sets.is_empty(), "dataset/ must contain synthesized JSONL");
    let mut total = 0;

    for (number, records) in &sets {
        check_record_fields(records, &format!("synth-{number}"));
        total += records.len();
    }
    assert_eq!(
        total, SYNTH_RECORDS,
        "update the training corpus size deliberately"
    );
    for (number, expected) in [
        ("19", SYNTH_19_RECORDS),
        ("21", SYNTH_21_RECORDS),
        ("22", SYNTH_22_RECORDS),
        ("23", SYNTH_23_RECORDS),
        ("24", SYNTH_24_RECORDS),
        ("25", SYNTH_25_RECORDS),
        ("26", SYNTH_26_RECORDS),
        ("27", SYNTH_27_RECORDS),
        ("28", SYNTH_28_RECORDS),
        ("29", SYNTH_29_RECORDS),
        ("30", SYNTH_30_RECORDS),
        ("31", SYNTH_31_RECORDS),
        ("33", SYNTH_33_RECORDS),
        ("34", SYNTH_34_RECORDS),
        ("35", SYNTH_35_RECORDS),
        ("36", SYNTH_36_RECORDS),
        ("37", SYNTH_37_RECORDS),
        ("38", SYNTH_38_RECORDS),
        ("39", SYNTH_39_RECORDS),
        ("40", SYNTH_40_RECORDS),
    ] {
        let (_, records) = sets
            .iter()
            .find(|(found, _)| found == number)
            .expect("the protected synthesized split must remain present");
        assert_eq!(records.len(), expected, "synth-{number}: stable split size");
    }
}

/// Keep regeneration and validation together so parallel tests cannot read
/// the specification export while it is being rewritten.
#[test]
fn the_spec_dataset_matches_the_document() {
    let rendered = render_jsonl(spec_records());
    let output = "dataset/nucleoid.spec.jsonl";
    let path = path(output);

    if std::env::var_os("UPDATE_DATASET").is_some() {
        fs::write(&path, &rendered).unwrap_or_else(|error| {
            panic!("{output} cannot be regenerated from nucleoid.spec.md: {error}")
        });
    }

    let committed = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{output} cannot be read: {error}"));
    assert_eq!(
        committed.replace("\r\n", "\n"),
        rendered,
        "{output} differs from nucleoid.spec.md; regenerate with \
         UPDATE_DATASET=1 cargo test --test dataset"
    );
    check_record_fields(&parse_jsonl::<Record>(&committed, &path), "spec");
}

fn check_record_fields(records: &[Record], prefix: &str) {
    assert!(!records.is_empty(), "{prefix}: dataset must not be empty");
    for (index, record) in records.iter().enumerate() {
        let id = &record.id;
        assert_eq!(
            *id,
            format!("{prefix}-{:04}", index + 1),
            "{prefix}: stable record id"
        );
        assert!(
            !record.description.trim().is_empty(),
            "{id}: description must not be empty"
        );
        assert!(
            !record.code.trim().is_empty(),
            "{id}: program must not be empty"
        );
        if let Some(expected) = &record.returns {
            serde_json::from_str::<serde_json::Value>(expected)
                .unwrap_or_else(|error| panic!("{id}: return must be JSON: {error}"));
        }
    }
}

/// Every normative specification record must parse. Synthesized records are
/// training data and deliberately do not participate in runtime testing.
#[test]
fn every_spec_record_is_nucleoid() {
    let mut records = 0;
    for case in common::cases(SPEC_DOCUMENT) {
        let code = code(&case);

        if let Err(error) = Runtime::check(&code) {
            panic!("{} does not parse: {error}\n{code}", case.title);
        }

        records += 1;
    }

    // Two empty specification representations would agree and prove nothing.
    assert_eq!(
        records, SPEC_RECORDS,
        "update the specification case count deliberately when one is added"
    );
}

fn spec_records() -> Vec<Record> {
    common::cases(SPEC_DOCUMENT)
        .iter()
        .enumerate()
        .map(|(index, case)| Record {
            id: format!("spec-{:04}", index + 1),
            description: case.title.clone(),
            code: code(case),
            returns: case.expected.clone(),
        })
        .collect()
}

fn render_jsonl<T: Serialize>(records: impl IntoIterator<Item = T>) -> String {
    let mut out = String::new();

    for record in records {
        out.push_str(&serde_json::to_string(&record).expect("a record serialises"));
        out.push('\n');
    }

    out
}

fn synthesized_sets() -> Vec<(String, Vec<Record>)> {
    let mut files: Vec<_> = fs::read_dir(path("dataset"))
        .expect("dataset/ must be readable")
        .map(|entry| {
            entry
                .expect("a dataset directory entry must be readable")
                .path()
        })
        .filter(|file| {
            file.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("nucleoid.synth.") && name.ends_with(".jsonl"))
        })
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|file| {
            let number = file
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.strip_prefix("nucleoid.synth."))
                .expect("synthesized JSONL has a numbered filename")
                .to_string();
            assert!(
                !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()),
                "{}: synthesized dataset identifiers must be numbered",
                file.display()
            );
            (number, read_jsonl(&file))
        })
        .collect()
}

fn read_jsonl<T: DeserializeOwned>(file: &Path) -> Vec<T> {
    let source = fs::read_to_string(file)
        .unwrap_or_else(|error| panic!("{} cannot be read: {error}", file.display()));
    parse_jsonl(&source, file)
}

fn parse_jsonl<T: DeserializeOwned>(source: &str, file: &Path) -> Vec<T> {
    source
        .lines()
        .enumerate()
        .map(|(index, line)| {
            serde_json::from_str(line).unwrap_or_else(|error| {
                panic!(
                    "{}:{}: invalid JSONL record: {error}",
                    file.display(),
                    index + 1
                )
            })
        })
        .collect()
}

#[test]
fn jsonl_records_accept_crlf_and_preserve_fields() {
    let source = concat!(
        r#"{"id":"synth-01-0001","description":"a nullable return","code":"i = 1\ni","return":null}"#,
        "\r\n",
        r#"{"id":"synth-01-0002","description":"a JSON return","code":"i = 2\ni","return":"2"}"#,
        "\r\n",
    );
    let records = parse_jsonl::<Record>(source, Path::new("nucleoid.synth.01.jsonl"));

    assert_eq!(records.len(), 2);
    check_record_fields(&records, "synth-01");
    assert_eq!(records[0].code, "i = 1\ni");
    assert!(records[0].returns.is_none());
    assert_eq!(records[1].returns.as_deref(), Some("2"));
}

#[test]
#[should_panic(expected = "blank.jsonl:2: invalid JSONL record")]
fn jsonl_records_reject_blank_lines_with_context() {
    parse_jsonl::<u8>("1\n \t\n2\n", Path::new("blank.jsonl"));
}

#[test]
#[should_panic(expected = "malformed.jsonl:2: invalid JSONL record")]
fn jsonl_records_reject_invalid_json_with_context() {
    parse_jsonl::<u8>("1\nnot JSON\n", Path::new("malformed.jsonl"));
}

#[test]
#[should_panic(expected = "unknown.jsonl:1: invalid JSONL record: unknown field `extra`")]
fn jsonl_records_reject_unknown_fields_with_context() {
    parse_jsonl::<Record>(
        r#"{"id":"spec-0001","description":"a record","code":"i = 1","return":null,"extra":true}"#,
        Path::new("unknown.jsonl"),
    );
}

#[test]
#[should_panic(expected = "synth-02: stable record id")]
fn record_fields_reject_ids_from_another_synthesized_file() {
    let records = parse_jsonl::<Record>(
        r#"{"id":"synth-01-0001","description":"a record","code":"i = 1","return":null}"#,
        Path::new("nucleoid.synth.02.jsonl"),
    );
    check_record_fields(&records, "synth-02");
}

/// The program as the suites run it, without the comment that titles it — that
/// comment becomes the `description` field, and a model trained on this should
/// not learn to write it back.
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
