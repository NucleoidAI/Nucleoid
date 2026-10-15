//! The specification export and standalone JSONL training data in `dataset/`.
//!
//! Only `nucleoid.spec.jsonl` is generated from Markdown. Regenerate it with
//!
//! ```text
//! UPDATE_DATASET=1 cargo test --test dataset
//! ```
//!
//! Synthesized and ARC records are maintained directly as JSONL, without
//! duplicate Markdown sources. They are training data, not normative tests.

mod common;

use std::collections::HashSet;
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

const SPEC_RECORDS: usize = 338;
const SYNTH_RECORDS: usize = 1_081;
const SYNTH_19_RECORDS: usize = 51;
const SYNTH_21_RECORDS: usize = 118;

type Grid = Vec<Vec<u8>>;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ArcTask {
    Core,
    Curve,
    Line,
    Recolor,
    MirrorHorizontal,
    MirrorVertical,
    Rotate,
    Translate,
    Crop,
    Upscale,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ArcOrigin {
    Source,
    Synthetic,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ArcSource {
    file: String,
    case: usize,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ArcInstance {
    instance_name: String,
    input_instance: Grid,
    output_instance: Grid,
    input_code: String,
    output_value: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ArcRecord {
    id: String,
    task: ArcTask,
    origin: ArcOrigin,
    source: Option<ArcSource>,
    input_matrix: Grid,
    output_matrix: Grid,
    instances: Vec<ArcInstance>,
    normalizations: Vec<String>,
    description: String,
    code: String,
    #[serde(rename = "return")]
    returns: String,
}

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
    for (number, expected) in [("19", SYNTH_19_RECORDS), ("21", SYNTH_21_RECORDS)] {
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

#[test]
fn arc_records_preserve_their_grid_and_annotation_contract() {
    let records = arc_records();
    let mut ids = HashSet::new();
    let mut sources = HashSet::new();
    let mut source_instances = 0;
    let mut synthetic = 0;

    assert_eq!(records.len(), 109, "nine source scenes and 100 new cases");

    for record in &records {
        let id = &record.id;
        assert!(id.starts_with("arc-") && ids.insert(id), "{id}");
        assert!(!record.description.is_empty(), "{id}: missing description");
        assert!(!record.code.contains("# arc:"), "{id}: metadata in code");
        check_instance_masks(record);
        assert_eq!(
            serde_json::from_str::<Grid>(&record.returns)
                .unwrap_or_else(|error| panic!("{id}: return must be a JSON grid: {error}")),
            record.output_matrix,
            "{id}: return must match output_matrix"
        );
        assert!(
            record.code.contains(&format!(
                "input_matrix = {}",
                serde_json::to_string(&record.input_matrix).unwrap()
            )),
            "{id}: program must declare its input grid"
        );
        Runtime::check(&record.code).unwrap_or_else(|error| panic!("{id} does not parse: {error}"));

        match record.origin {
            ArcOrigin::Source => {
                let source = record
                    .source
                    .as_ref()
                    .unwrap_or_else(|| panic!("{id}: source scene must declare provenance"));
                let expected_file = match record.task {
                    ArcTask::Core => "dataset.core.json",
                    ArcTask::Curve => "dataset.curve.json",
                    ArcTask::Line => "dataset.line.json",
                    _ => panic!("{id}: unexpected source task"),
                };
                assert_eq!(source.file, expected_file, "{id}");
                assert!((1..=3).contains(&source.case), "{id}: source scene index");
                assert!(sources.insert((&source.file, source.case)), "{id}");
                source_instances += record.instances.len();
            }
            ArcOrigin::Synthetic => {
                synthetic += 1;
                assert!(record.source.is_none(), "{id}: synthetic provenance");
            }
        }

        let mut instance_names = HashSet::new();
        for instance in &record.instances {
            assert!(
                instance_names.insert(&instance.instance_name),
                "{id}: duplicate instance name"
            );
            assert_eq!(
                instance
                    .output_value
                    .get("id")
                    .and_then(|value| value.as_str()),
                Some(instance.instance_name.as_str()),
                "{id}: instance id must match its name"
            );
            assert!(
                record.code.contains(&instance.input_code),
                "{id}: instance code must be part of the program"
            );
        }
    }

    assert_eq!(sources.len(), 9);
    assert_eq!(source_instances, 23);
    assert_eq!(synthetic, 100);
}

fn check_grid(grid: &[Vec<u8>], context: &str) {
    let height = grid.len();
    assert!(
        (1..=30).contains(&height),
        "{context}: grid height {height} must be in 1..=30"
    );
    let width = grid[0].len();
    assert!(
        (1..=30).contains(&width),
        "{context}: grid width {width} must be in 1..=30"
    );

    for (row, values) in grid.iter().enumerate() {
        assert_eq!(
            values.len(),
            width,
            "{context}[{row}]: row width must match {width}"
        );
        for (column, &color) in values.iter().enumerate() {
            assert!(
                color <= 9,
                "{context}[{row}][{column}]: color {color} must be in 0..=9"
            );
        }
    }
}

fn check_instance_mask(mask: &[Vec<u8>], scene: &[Vec<u8>], context: &str) {
    check_grid(mask, context);
    assert_eq!(
        mask.len(),
        scene.len(),
        "{context}: mask height must match scene"
    );
    assert_eq!(
        mask[0].len(),
        scene[0].len(),
        "{context}: mask width must match scene"
    );

    for (row, (mask_row, scene_row)) in mask.iter().zip(scene).enumerate() {
        for (column, (&color, &scene_color)) in mask_row.iter().zip(scene_row).enumerate() {
            assert!(
                color == 0 || color == scene_color,
                "{context}[{row}][{column}]: color {color} must be 0 or scene color {scene_color}"
            );
        }
    }
}

fn check_instance_masks(record: &ArcRecord) {
    for (scene, input) in [(&record.input_matrix, true), (&record.output_matrix, false)] {
        let direction = if input { "input" } else { "output" };
        check_grid(scene, &format!("{}: {direction}_matrix", record.id));
        let mut combined = (record.origin == ArcOrigin::Source)
            .then(|| vec![vec![0; scene[0].len()]; scene.len()]);

        for instance in &record.instances {
            let mask = if input {
                &instance.input_instance
            } else {
                &instance.output_instance
            };
            let context = format!(
                "{}: {} {direction}_instance",
                record.id, instance.instance_name
            );

            // Validate dimensions before indexing the combined scene. Even an
            // all-zero mask must be rectangular and have the scene's size.
            check_instance_mask(mask, scene, &context);
            if let Some(combined) = &mut combined {
                for (row, values) in mask.iter().enumerate() {
                    for (column, &color) in values.iter().enumerate() {
                        if color != 0 {
                            assert!(
                                combined[row][column] == 0 || combined[row][column] == color,
                                "{context}[{row}][{column}]: conflicting instance pixels"
                            );
                            combined[row][column] = color;
                        }
                    }
                }
            }
        }

        if let Some(combined) = combined {
            assert_eq!(
                combined, *scene,
                "{}: complete {direction} instance masks",
                record.id
            );
        }
    }
}

#[test]
fn arc_grids_accept_palette_and_size_boundaries() {
    check_grid(&[vec![0]], "arc-boundary: input_matrix");
    check_grid(&vec![vec![9; 30]; 30], "arc-boundary: output_matrix");
}

#[test]
#[should_panic(expected = "arc-invalid: input_matrix: grid height 0 must be in 1..=30")]
fn arc_grids_reject_empty_grids_with_context() {
    check_grid(&[], "arc-invalid: input_matrix");
}

#[test]
#[should_panic(expected = "arc-invalid: output_matrix: grid width 0 must be in 1..=30")]
fn arc_grids_reject_empty_rows_with_context() {
    check_grid(&[vec![]], "arc-invalid: output_matrix");
}

#[test]
#[should_panic(expected = "arc-invalid: input_matrix: grid height 31 must be in 1..=30")]
fn arc_grids_reject_excessive_height_with_context() {
    check_grid(&vec![vec![0]; 31], "arc-invalid: input_matrix");
}

#[test]
#[should_panic(expected = "arc-invalid: output_matrix: grid width 31 must be in 1..=30")]
fn arc_grids_reject_excessive_width_with_context() {
    check_grid(&[vec![0; 31]], "arc-invalid: output_matrix");
}

#[test]
#[should_panic(expected = "arc-invalid: input_matrix[1]: row width must match 1")]
fn arc_grids_identify_ragged_rows() {
    check_grid(&[vec![1], vec![1, 2]], "arc-invalid: input_matrix");
}

#[test]
#[should_panic(expected = "arc-invalid: output_matrix[1][0]: color 10 must be in 0..=9")]
fn arc_grids_identify_invalid_pixels() {
    check_grid(&[vec![0], vec![10]], "arc-invalid: output_matrix");
}

#[test]
#[should_panic(
    expected = "arc-core-source-0001: obj6068 input_instance: mask height must match scene"
)]
fn arc_source_masks_check_height_before_composition() {
    let mut record = arc_records().remove(0);
    record.instances[0]
        .input_instance
        .push(vec![9; record.input_matrix[0].len()]);
    check_instance_masks(&record);
}

#[test]
#[should_panic(
    expected = "arc-core-source-0001: obj6068 output_instance: mask width must match scene"
)]
fn arc_source_masks_check_width_before_composition() {
    let mut record = arc_records().remove(0);
    for row in &mut record.instances[0].output_instance {
        row.push(9);
    }
    check_instance_masks(&record);
}

#[test]
#[should_panic(
    expected = "arc-core-source-0001: obj6068 input_instance[0][0]: color 9 must be 0 or scene color 0"
)]
fn arc_source_masks_identify_pixels_that_disagree_with_the_scene() {
    let mut record = arc_records().remove(0);
    record.instances[0].input_instance[0][0] = 9;
    check_instance_masks(&record);
}

#[test]
#[should_panic(expected = "arc-core-source-0001: complete input instance masks")]
fn arc_source_masks_must_still_cover_the_complete_scene() {
    let mut record = arc_records().remove(0);
    record.instances[0].input_instance[1][6] = 0;
    check_instance_masks(&record);
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

fn arc_records() -> Vec<ArcRecord> {
    read_jsonl(&path("dataset/arc.jsonl"))
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
