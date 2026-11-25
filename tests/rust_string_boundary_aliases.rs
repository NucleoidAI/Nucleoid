mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::Command;

use nucleoid::builtins;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde::Deserialize;

const SPEC_TITLES: [&str; 2] = [
    "Nucleoid tests string boundaries with Rust spellings",
    "Nucleoid keeps Rust boundary rules live and rolls back failed receiver updates",
];
const REFERENCE_TITLE: &str = "Nucleoid classifies live strings with Rust boundary names";
const README_TITLE: &str = "Rust boundary methods keep file flags live";
const EXAMPLES_TITLE: &str = "Rust boundary methods keep a document desk's selection live";

fn documents() -> [Vec<common::Case>; 4] {
    [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ]
}

fn calls_alias(source: &str) -> bool {
    let tokens: Vec<Token> = tokenize(source)
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    tokens.windows(3).any(|window| {
        window[0] == Token::Dot
            && matches!(&window[1], Token::Identifier(name) if name == "starts_with" || name == "ends_with")
            && window[2] == Token::LeftParen
    })
}

fn checked(runtime: &mut Runtime, source: &str, expected: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "Rust string boundary regression".to_string(),
            source: source.to_string(),
            expected: Some(expected.to_string()),
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
    value
}

fn dependencies(runtime: &Runtime, name: &str) -> Vec<String> {
    runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap()
        .dependencies
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn only_five_appended_document_cases_call_the_aliases_and_execute_every_assertion() {
    let documents = documents();
    let titles: Vec<&str> = SPEC_TITLES
        .into_iter()
        .chain([REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE])
        .collect();
    let mut assertions = 0;
    for case in documents.iter().flatten() {
        let new = titles.contains(&case.title.as_str());
        assert_eq!(calls_alias(&case.source), new, "{}", case.title);
        if new {
            let count =
                common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            assert!(case.expected.is_some(), "{} declares a return", case.title);
            let mut runtime = Runtime::new();
            common::check_with_count(&mut runtime, case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assert_eq!(runtime.assertions_run(), count, "{}", case.title);
            assertions += count;
        }
    }
    for title in titles {
        assert_eq!(
            documents
                .iter()
                .flatten()
                .filter(|case| case.title == title)
                .count(),
            1,
            "{title} is appended once"
        );
    }
    assert_eq!(documents.iter().map(Vec::len).sum::<usize>(), 1310);
    assert_eq!(assertions, 22);
    println!("Executed five new document cases, 22 assertions and five explicit returns.");
}

#[test]
fn the_language_reference_contains_the_executable_reference_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Rust string boundary method aliases\n\n```\n")
        .unwrap();
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .unwrap();
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(case.expected.as_deref(), Some("[true,true]"));
}

#[test]
fn every_legacy_boundary_name_and_string_receiver_route_still_works() {
    checked(
        &mut Runtime::new(),
        r#"
text = "report.csv"
assert([text.starts_with("report"), text.startsWith("report"), text.startswith("report")], [true, true, true])
assert([text.ends_with(".csv"), text.endsWith(".csv"), text.endswith(".csv")], [true, true, true])
record = {name: "café.csv"}
assert(record.name.starts_with("café"), true)
names = ["skip", "last.csv"]
assert(names[1].ends_with(".csv"), true)
def name() => "draft.txt"
assert(name().starts_with("draft"), true)
[text.startsWith("Report"), text.startswith("Report"), text.endsWith(".CSV"), text.endswith(".CSV")]
"#,
        "[false,false,false,false]",
    );
}

#[test]
fn the_public_string_method_uses_the_same_query_arms_and_text_conversion() {
    for (alias, spelling, receiver, arguments, expected) in [
        (
            "starts_with",
            "startsWith",
            "café.csv",
            vec![Value::String("café".to_string())],
            true,
        ),
        (
            "ends_with",
            "endsWith",
            "café.csv",
            vec![Value::String(".CSV".to_string())],
            false,
        ),
        (
            "starts_with",
            "startsWith",
            "e\u{301}",
            vec![Value::String("é".to_string())],
            false,
        ),
        (
            "ends_with",
            "endsWith",
            "undefined",
            vec![Value::Undefined],
            true,
        ),
        ("starts_with", "startsWith", "undefined", vec![], true),
        (
            "starts_with",
            "startsWith",
            "12 files",
            vec![Value::Number(12.0)],
            true,
        ),
        (
            "ends_with",
            "endsWith",
            "reportnull",
            vec![Value::Null, Value::String("ignored".to_string())],
            true,
        ),
        (
            "ends_with",
            "endsWith",
            "a.csv,.txt",
            vec![Value::List(vec![
                Value::String(".csv".to_string()),
                Value::String(".txt".to_string()),
            ])],
            true,
        ),
        (
            "starts_with",
            "startsWith",
            "",
            vec![Value::String(String::new())],
            true,
        ),
    ] {
        assert_eq!(
            builtins::string_method(receiver, alias, &arguments).unwrap(),
            Value::Bool(expected),
            "{receiver}.{alias}"
        );
        assert_eq!(
            builtins::string_method(receiver, spelling, &arguments).unwrap(),
            Value::Bool(expected),
            "{receiver}.{spelling}"
        );
    }
    assert_eq!(
        builtins::string("fromCharCode", &[Value::Number(65.0)]).unwrap(),
        Value::String("A".to_string())
    );
    for name in ["starts_with", "ends_with"] {
        let error = builtins::string(name, &[]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), format!("String.{name} is not a function"));
    }
}

#[test]
fn receiver_then_all_arguments_evaluate_once_in_written_order() {
    let functions = r#"
log = []
def receiver():
    log.push("receiver")
    return "prefix.csv"
def argument(value):
    log.push(value)
    return value
"#;
    for (alias, affix) in [("starts_with", "pre"), ("ends_with", ".csv")] {
        checked(
            &mut Runtime::new(),
            &format!(
                "{functions}[receiver().{alias}(argument(\"{affix}\"), argument(\"extra\"), argument(\"last\")), log]"
            ),
            &format!("[true,[\"receiver\",\"{affix}\",\"extra\",\"last\"]]"),
        );
        let mut runtime = Runtime::new();
        runtime.run(functions).unwrap();
        let error = runtime
            .run(&format!(
                "receiver().{alias}(argument(\"{affix}\"), missing, argument(\"last\"))"
            ))
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        checked(&mut runtime, "log", "[]");
    }
}

#[test]
fn nullish_receivers_skip_arguments_but_null_and_missing_affixes_are_text() {
    checked(
        &mut Runtime::new(),
        r#"
log = []
value = null
record = {}
assert(value.starts_with(log.push(1), missing), undefined)
assert(record.absent.ends_with(log.push(2), missing), undefined)
assert(undefined.ends_with(log.push(3)), undefined)
assert(log, [])
assert(["null".starts_with(null), "undefined".ends_with()], [true, true])
log
"#,
        "[]",
    );
}

#[test]
fn properties_user_functions_and_instance_methods_keep_their_own_routing() {
    checked(
        &mut Runtime::new(),
        r#"
record = {starts_with: x -> x + 1, ends_with: x -> x + 2}
def starts_with(x) => x + 3
class Marker:
    def starts_with(x) => x + 4
    def ends_with(x) => x + 5
marker = Marker()
assert([record.starts_with(1), record.ends_with(1), starts_with(1), marker.starts_with(1), marker.ends_with(1)], [2, 3, 4, 5, 6])
assert(["abc".starts_with, "abc".ends_with], [undefined, undefined])
empty = {}
assert([empty.starts_with(missing), empty.ends_with(missing)], [undefined, undefined])
[record.ends_with(2), marker.starts_with(2)]
"#,
        "[4,6]",
    );
}

#[test]
fn nonstring_static_assignment_and_unknown_name_errors_keep_their_formats() {
    for (source, message, line) in [
        (
            "xs = [\"a\"]\nxs.starts_with(\"a\")",
            "xs.starts_with is not a function",
            2,
        ),
        (
            "(5).ends_with(\"5\")",
            "number.ends_with is not a function",
            1,
        ),
        (
            "class Tag\nTag.starts_with(\"a\")",
            "Tag.starts_with is not a function",
            2,
        ),
        (
            "String.starts_with(\"a\")",
            "String.starts_with is not a function",
            1,
        ),
        (
            "String.ends_with(\"a\")",
            "String.ends_with is not a function",
            1,
        ),
        (
            "text = \"abc\"\ntext.starts_with = 1",
            "Cannot assign 'starts_with' on string",
            2,
        ),
        (
            "text = \"abc\"\ntext.ends_with = 1",
            "Cannot assign 'ends_with' on string",
            2,
        ),
        (
            "\"abc\".Starts_with(\"a\")",
            "abc.Starts_with is not a function",
            1,
        ),
    ] {
        let error = Runtime::new().run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(Position::new(line, 1)), "{source}");
        assert_eq!(
            error.to_string(),
            format!("TypeError: {message} (line {line}, column 1)"),
            "{source}"
        );
    }
}

#[test]
fn live_declarations_track_all_arguments_roll_back_and_route_a_replaced_receiver() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
text = "draft.csv"
prefix = "draft"
suffix = ".csv"
ignored = 0
head = text.starts_with(prefix, ignored)
tail = text.ends_with(suffix, ignored)
assert([head, tail], [true, true])
[head, tail]
"#,
        "[true,true]",
    );
    assert_eq!(
        dependencies(&runtime, "head"),
        ["text", "prefix", "ignored"]
    );
    assert_eq!(
        dependencies(&runtime, "tail"),
        ["text", "suffix", "ignored"]
    );
    checked(
        &mut runtime,
        "prefix = \"notes\"\nsuffix = \".txt\"\nassert([head, tail], [false, false])\ntext = \"notes.txt\"\nignored = 9\nassert([head, tail], [true, true])\n[head, tail]",
        "[true,true]",
    );
    let error = runtime.run("text = 5").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.starts_with is not a function");
    checked(
        &mut runtime,
        r#"
assert([text, head, tail], ["notes.txt", true, true])
try:
    prefix = "other"
    suffix = ".bad"
    throw "UNDO"
catch error:
    assert(error, "UNDO")
assert([prefix, suffix, head, tail], ["notes", ".txt", true, true])
[head, tail]
"#,
        "[true,true]",
    );
    checked(
        &mut runtime,
        "text = {starts_with: x -> true, ends_with: x -> false}\nassert([head, tail], [true, false])\ntext = \"notes.txt\"\n[head, tail]",
        "[true,true]",
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(source: &str) -> Self {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("rust-string-boundaries-{}.nuc", std::process::id()));
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap()
            .write_all(source.as_bytes())
            .unwrap();
        Self(path)
    }
}

impl Drop for SourceFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).unwrap();
    }
}

#[test]
fn syntax_explanations_and_cli_graphs_retain_the_written_alias_names() {
    for (source, rendered) in [
        (
            "head = text.starts_with(prefix, ignored)",
            "head=text.starts_with(prefix,ignored)",
        ),
        (
            "$File.table = $File.name.ends_with(suffix)",
            "$File.table=$File.name.ends_with(suffix)",
        ),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), rendered);
    }
    let source = r#"
name = "report.csv"
head = name.starts_with("report")
tail = name.ends_with(".csv")
if name.ends_with(".csv"):
    flag = true
[(why head)[0].rule, (why tail)[0].rule, head, tail]
"#;
    let expected =
        r#"["head = name.starts_with(\"report\")","tail = name.ends_with(\".csv\")",true,true]"#;
    checked(&mut Runtime::new(), source, expected);
    let file = SourceFile::new(source);
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(&file.0)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with(expected), "{stdout}");
    assert!(
        stdout.contains("head [variable]\n    reads    name"),
        "{stdout}"
    );
    assert!(
        stdout.contains("tail [variable]\n    reads    name"),
        "{stdout}"
    );
    assert!(stdout.contains("name.ends_with"), "{stdout}");
    assert!(
        !stdout.contains("startsWith") && !stdout.contains("endsWith"),
        "{stdout}"
    );
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SynthRecord {
    id: String,
    description: String,
    code: String,
    #[serde(rename = "return")]
    expected: String,
}

#[test]
fn all_three_new_synthesized_records_execute_assertions_and_expected_returns() {
    let mut assertions = 0;
    let records: Vec<SynthRecord> = include_str!("../dataset/nucleoid.synth.94.jsonl")
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(records.len(), 3);
    for (index, record) in records.iter().enumerate() {
        assert_eq!(record.id, format!("synth-94-{:04}", index + 1));
        assert!(!record.description.trim().is_empty());
        assert!(calls_alias(&record.code), "{}", record.id);
        let mut runtime = Runtime::new();
        checked(&mut runtime, &record.code, &record.expected);
        assert!(runtime.assertions_run() > 0, "{}", record.id);
        assertions += runtime.assertions_run();
    }
    assert_eq!(assertions, 8);
    println!(
        "Executed three new synthesized records, eight assertions and three explicit returns."
    );
}
