//! String sources extend only For's snapshot construction. Ordinary regression
//! and sealed-document checks need neither Git nor session artifacts.

mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;
#[path = "support/string_iteration_cases.rs"]
mod string_cases;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Stmt, parse};
use nucleoid::lang::evaluation::Flow;
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "String iteration regression".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
    value
}

#[test]
fn nine_sealed_pairs_preserve_all_controls_assertions_return_and_sixteen_traversals() {
    let documents = string_cases::documents();
    let mut previous = HashMap::new();
    let mut totals = (0, 0, 0);
    let mut forms = [0; 3];
    assert_eq!(string_cases::SEALED.len(), 9);
    for sealed in string_cases::SEALED {
        let cases = &documents
            .iter()
            .find(|(document, _)| *document == sealed.document)
            .unwrap()
            .1;
        let matching = cases
            .iter()
            .enumerate()
            .filter(|(_, case)| case.title == sealed.title)
            .collect::<Vec<_>>();
        let [(position, case)] = matching.as_slice() else {
            panic!("expected exactly one case: {}", sealed.title);
        };
        if let Some(before) = previous.insert(sealed.document, *position) {
            assert!(before < *position, "{}: relative order", sealed.title);
        }
        assert_eq!(case.expected.as_deref(), sealed.expected);
        let original_source = string_cases::documented_source(sealed);
        let (rewritten, edits) = string_cases::rewrite(&original_source);
        assert_eq!(edits, sealed.rewrites, "{}", sealed.title);
        assert_eq!(case.source.trim(), rewritten, "{}", sealed.title);
        let old = parse(&original_source).unwrap();
        let new = parse(&case.source).unwrap();
        assert_ne!(new, old, "map callbacks and For ASTs intentionally differ");
        let assertions = common::assertions::protected_assertions(&old);
        assert_eq!(assertions.len(), sealed.assertions);
        assert_eq!(common::assertions::protected_assertions(&new), assertions);
        for line in case.source.lines() {
            if let Some(header) = line.trim_start().strip_prefix("for ") {
                let form = if header.contains(" of ") {
                    0
                } else if header.contains(" in ") {
                    1
                } else {
                    2
                };
                forms[form] += 1;
            }
        }
        let original = common::Case {
            title: sealed.title.to_string(),
            source: sealed.accepted.to_string(),
            expected: sealed.expected.map(str::to_string),
        };
        let mut runtimes = [Runtime::new(), Runtime::new()];
        let mut results = Vec::new();
        for (runtime, source) in runtimes.iter_mut().zip([&original, *case]) {
            results.push(
                common::check_with_count(runtime, source, sealed.assertions)
                    .unwrap_or_else(|reason| panic!("{}: {reason}", sealed.title)),
            );
            assert_eq!(runtime.assertions_run(), sealed.assertions);
        }
        string_cases::assert_observations(&runtimes[1], &runtimes[0], sealed);
        if sealed.expected.is_some() {
            assert_eq!(results[1], results[0]);
        }
        totals.0 += sealed.assertions;
        totals.1 += usize::from(sealed.expected.is_some());
        totals.2 += edits;
    }
    assert_eq!(totals, (27, 1, 16));
    assert_eq!(forms, [11, 3, 2], "of, in, implicit-it");
}

#[test]
fn scalar_order_matches_split_indexing_and_length_without_normalization_or_trimming() {
    // Non-ASCII literals distinguish scalars from bytes, UTF-16 and graphemes.
    // The language's existing \0 escape covers embedded NUL without new syntax.
    for literal in [
        r#""""#,
        r#""cab""#,
        r#""Aé🙂é""#,
        r#""👩‍💻""#,
        r#"" \0🙂\t ""#,
    ] {
        for header in ["for character of text:", "for character in text:"] {
            let source = format!(
                "text = {literal}\nseen = []\n{header}\n    seen.push(character)\n\
                 assert(seen, text.split(\"\"))\nassert(seen.length, text.length)\n\
                 assert(seen.join(\"\"), text)"
            );
            let mut runtime = Runtime::new();
            checked(&mut runtime, &source, None);
            let Value::String(text) = runtime.state.variable("text").unwrap() else {
                panic!("expected a String source");
            };
            let scalars = text
                .chars()
                .map(|scalar| Value::String(scalar.to_string()))
                .collect::<Vec<_>>();
            assert_eq!(runtime.state.variable("seen"), Some(&Value::List(scalars)));
        }
    }
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
text = "Aé🙂é👩‍💻\0"
seen = []
for text:
    seen.push(it)
assert(seen, ["A", "é", "🙂", "e", "́", "👩", "‍", "💻", "\0"])
assert(seen.length, 9)
assert(text[2], "🙂")
assert(text[4], "́")
assert(text[6], "‍")
assert(text[8], "\0")
"#,
        None,
    );
}

#[test]
fn every_legacy_suite_and_explicit_expression_route_accepts_string_sources() {
    for expression in [
        "text",
        "(text)",
        "holder.text",
        "texts[0]",
        "supply()",
        "\"ab\"",
        "\"a\" + \"b\"",
        "text || \"fallback\"",
    ] {
        for keyword in ["of", "in"] {
            for body in [
                ": seen.push(character)",
                ": { seen.push(character) }",
                ":\n    seen.push(character)",
            ] {
                let source = format!(
                    "text = \"ab\"\nholder = {{text: text}}\ntexts = [text]\n\
                     def supply() => text\nseen = []\nfor character {keyword} {expression}{body}\n\
                     assert(seen, [\"a\", \"b\"])\nassert(text, \"ab\")"
                );
                checked(&mut Runtime::new(), &source, None);
            }
        }
    }
    // Both aliases still lower to exactly the same public For node.
    let explicit = parse("for character of holder.text:\n    seen.push(character)").unwrap();
    assert!(matches!(explicit.as_slice(), [Stmt::For { .. }]));
    assert_eq!(
        explicit,
        parse("for character in holder.text:\n    seen.push(character)").unwrap()
    );
}

#[test]
fn source_calls_and_assignment_expressions_execute_once_before_stable_snapshots() {
    for keyword in ["of", "in"] {
        let source = format!(
            r#"
class Source
source = Source()
source.calls = 0
source.text = "abc"
def supply():
    source.calls += 1
    return source.text
seen = []
for character {keyword} supply():
    seen.push(character)
    source.text = "extended"
assert(source.calls, 1)
assert(seen, ["a", "b", "c"])
assert(source.text, "extended")
source.text = "xy"
assert(seen, ["a", "b", "c"])
assert(source.calls, 1)
for character {keyword} (source.text = "pq"):
    seen.push(character)
    source.text = "changed"
assert(seen, ["a", "b", "c", "p", "q"])
assert(source.text, "changed")
assert(source.calls, 1)
"#
        );
        checked(&mut Runtime::new(), &source, None);
    }
}

#[test]
fn string_bindings_shadow_and_restore_same_name_sources_parameters_and_nested_it() {
    checked(
        &mut Runtime::new(),
        r#"
it = "ab"
digits = "12"
seen = []
for it:
    seen.push(it)
    for digits:
        seen.push(it)
    seen.push(it)
assert(seen, ["a", "1", "2", "a", "b", "1", "2", "b"])
assert(it, "ab")
text = "xy"
for text of text:
    seen.push(text)
assert(text, "xy")
assert(seen, ["a", "1", "2", "a", "b", "1", "2", "b", "x", "y"])
def first(it):
    for it:
        return it
    return "empty"
assert(first("pq"), "p")
assert(first(""), "empty")
assert(it, "ab")
"#,
        None,
    );
}

#[test]
fn method_and_constructor_loops_keep_this_and_self_and_independent_receivers() {
    checked(
        &mut Runtime::new(),
        r#"
class Reader:
    def init(text):
        this.letters = []
        for character of text:
            self.letters.push(character)
    def append(text):
        for text:
            this.letters.push(it)
        return self.letters
    def first(text):
        for character in text:
            return [character, this.letters, self.letters]
        return []
first = Reader("ab")
second = Reader("CD")
assert(first.letters, ["a", "b"])
assert(second.letters, ["C", "D"])
assert(first.append("xy"), ["a", "b", "x", "y"])
assert(second.letters, ["C", "D"])
assert(first.first("pq"), ["p", ["a", "b", "x", "y"], ["a", "b", "x", "y"]])
assert(first.first(""), [])
assert(second.append(""), ["C", "D"])
"#,
        None,
    );
}

#[test]
fn early_bare_and_absent_returns_and_empty_sources_keep_existing_for_flow() {
    checked(
        &mut Runtime::new(),
        r#"
seen = []
def choose(text):
    for text:
        seen.push(it)
        if it == "b":
            return it
    return "empty"
def bare(text):
    for character of text:
        return
    return "empty"
def noResult(text):
    for character in text:
        character
def nested(text):
    for text:
        for character of "12":
            return it + character
    return "empty"
assert(choose("abc"), "b")
assert(seen, ["a", "b"])
assert(choose(""), "empty")
assert(bare("ab"), null)
assert(bare(""), "empty")
assert(noResult("ab"), null)
assert(noResult(""), null)
assert(nested("ab"), "a1")
assert(nested(""), "empty")
"#,
        None,
    );
    for source in [
        "for character of \"ab\":\n    character",
        "for character in \"\":\n    throw \"empty body executed\"",
        "text = \"\"\nfor text:\n    throw \"empty body executed\"",
    ] {
        checked(&mut Runtime::new(), source, Some("null"));
    }
}

#[test]
fn strings_do_not_broaden_primitive_lists_or_change_registered_object_snapshots() {
    checked(
        &mut Runtime::new(),
        r#"
class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
plain = Object()
values = ["ab", second, 99, first, null, {amount: 9}, second, plain, true]
seen = []
for it of values:
    seen.push(it.amount)
assert(seen, [5, 2, 5])
for it in "ab".split(""):
    throw "primitive split item visited"
primitives = ["ab", "", 1, true, null, {amount: 9}, plain]
for primitives:
    throw "primitive list item visited"
assert(seen, [5, 2, 5])
for it of Entry:
    seen.push(it.amount)
    Entry()
assert(seen, [5, 2, 5, 2, 5])
assert(Entry.length, 4)
"#,
        None,
    );
}

#[test]
fn nonstring_noniterables_and_callable_values_keep_exact_errors_and_rollback() {
    for setup in [
        "broken = null",
        "broken = 7",
        "broken = true",
        "broken = {}",
        "broken = Object()",
        "broken = Entry()",
        "broken = Date(0)",
        "broken = /ab/",
        "broken = value => value",
        "def broken():\n    Entry()\n    return \"ab\"",
    ] {
        for header in ["for it of broken:", "for it in broken:", "for broken:"] {
            let mut runtime = Runtime::new();
            runtime
                .run(&format!("class Entry\nsaved = 7\n{setup}"))
                .unwrap();
            let before = runtime.run("Entry.length").unwrap();
            let error = runtime
                .run(&format!("saved = 99\n{header}\n    saved = 100"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Type, "{setup}");
            assert_eq!(error.message(), "broken is not iterable", "{setup}");
            assert_eq!(error.position(), Some(Position::new(2, 1)));
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
            assert_eq!(runtime.run("Entry.length").unwrap(), before);
        }
    }
}

#[test]
fn uncaught_errors_restore_the_complete_runtime_then_allow_declarative_recovery() {
    for (body, kind, message) in [
        ("missing", ErrorKind::Reference, "missing is not defined"),
        ("this.text", ErrorKind::Reference, "this is not defined"),
        ("self.text", ErrorKind::Reference, "this is not defined"),
        ("throw \"stop\"", ErrorKind::Thrown, "stop"),
    ] {
        for header in ["for it of text:", "for it in text:", "for text:"] {
            let mut actual = Runtime::new();
            let mut expected = Runtime::new();
            let setup = "text = \"abc\"\nit = \"outer\"\nseen = [\"saved\"]\nsaved = 7";
            actual.run(setup).unwrap();
            expected.run(setup).unwrap();
            let error = actual
                .run(&format!(
                    "saved = 99\n{header}\n    seen.push(it)\n    {body}"
                ))
                .unwrap_err();
            assert_eq!(error.kind(), kind);
            if kind == ErrorKind::Thrown {
                assert_eq!(error.thrown_value(), Some(&Value::string(message)));
            } else {
                assert_eq!(error.message(), message);
            }
            assert_eq!(error.position(), Some(Position::new(2, 1)));
            runtime_equivalence::assert_equivalent(&actual, None, &expected, None);
            // An untouched control proves the full rollback above. For later
            // allocations, give the accepted split/map control the same failed
            // write: rollback does not rewind future sequence allocation.
            let counterpart = expected
                .run(&format!(
                    "saved = 99\ntext.split(\"\").map(it => {{\n    seen.push(it)\n    {body}\n}})"
                ))
                .unwrap_err();
            assert_eq!(counterpart, error);
            runtime_equivalence::assert_equivalent(&actual, None, &expected, None);
            let recovery = r#"
assert(seen, ["saved"])
assert(saved, 7)
assert(it, "outer")
for character of "ok":
    seen.push(character)
assert(seen, ["saved", "o", "k"])
input = 2
output = input * 3
input = 4
assert(output, 12)
"#;
            checked(&mut actual, recovery, None);
            checked(&mut expected, recovery, None);
            runtime_equivalence::assert_equivalent(&actual, None, &expected, None);
        }
    }
    let mut runtime = Runtime::new();
    runtime.run("saved = 7").unwrap();
    let error = runtime
        .run("saved = 99\nfor it of missing:\n    pass")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
}

#[test]
fn direct_for_nodes_restore_scope_receivers_and_mode_on_all_string_exits() {
    for text in ["ab", "", "\0🙂e\u{301}"] {
        for body in ["pass", "return it", "return", "missing", "throw \"stop\""] {
            for header in ["for text:", "for it of text:", "for it in text:"] {
                let mut runtime = Runtime::new();
                runtime
                    .run("class Receiver\nreceiver = Receiver()")
                    .unwrap();
                let receiver = ObjectId::named("receiver");
                let mut scope = Scope::new();
                scope.push();
                scope.declare("it", Value::string("outer"));
                scope.declare("text", Value::string(text));
                scope.set_this(Some(receiver.clone()));
                scope.set_instance(Some(receiver.clone()));
                let statements = parse(&format!("{header}\n    {body}")).unwrap();
                let mut node = Nuc::convert(&mut runtime, &mut scope, &statements[0]).unwrap();
                let result = node.run(&mut runtime, &mut scope);
                if text.is_empty() || body == "pass" {
                    assert!(matches!(result.unwrap().flow, Flow::Normal(Value::Null)));
                } else {
                    match (body, result) {
                        ("return it", Ok(outcome)) => assert!(matches!(
                            outcome.flow,
                            Flow::Return(Value::String(ref value))
                                if value == &text.chars().next().unwrap().to_string()
                        )),
                        ("return", Ok(outcome)) => {
                            assert!(matches!(outcome.flow, Flow::Return(Value::Null)));
                        }
                        ("missing", Err(error)) => {
                            assert_eq!(error.kind(), ErrorKind::Reference);
                            assert_eq!(error.message(), "missing is not defined");
                        }
                        ("throw \"stop\"", Err(error)) => {
                            assert_eq!(error.thrown_value(), Some(&Value::string("stop")));
                        }
                        _ => panic!("unexpected direct For result: {header} {body}"),
                    }
                }
                assert_eq!(scope.depth(), 2);
                assert_eq!(scope.retrieve("it"), Some(&Value::string("outer")));
                assert_eq!(scope.retrieve("text"), Some(&Value::string(text)));
                assert_eq!(scope.this(), Some(&receiver));
                assert_eq!(scope.instance(), Some(&receiver));
                checked(
                    &mut runtime,
                    "input = 2\noutput = input * 3\ninput = 4\nassert(output, 12)",
                    None,
                );
            }
        }
    }
}

#[test]
fn completed_loops_leave_no_standing_graph_or_iteration_local_and_do_not_replay() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
text = "ab"
it = "outer"
seen = []
for text:
    seen.push(it)
assert(seen, ["a", "b"])
text = "cd"
assert(seen, ["a", "b"])
for character of text:
    seen.push(character)
assert(seen, ["a", "b", "c", "d"])
assert(it, "outer")
"#,
        None,
    );
    assert!(!runtime.graph.contains(&NodeKey::variable("character")));
    assert!(
        runtime
            .graph
            .keys()
            .all(|key| !key.as_str().starts_with("for "))
    );
    assert!(runtime.state.variable("character").is_none());
}

fn fingerprint(bytes: impl IntoIterator<Item = u8>) -> u64 {
    bytes.into_iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

#[test]
fn all_642_historical_measured_programs_remain_byte_exact_with_future_tails_allowed() {
    for ((document, cases), (name, count, seal)) in string_cases::documents().into_iter().zip([
        ("spec", 433, 0x4265_f191_d569_760d),
        ("reference", 85, 0x04c7_51aa_3972_0d5c),
        ("readme", 34, 0x42f4_c74d_ed9d_8031),
        ("examples", 90, 0x34e0_8735_f827_8b7a),
    ]) {
        assert_eq!(document, name);
        assert!(cases.len() >= count);
        let protected = cases
            .iter()
            .take(count)
            .flat_map(|case| {
                [
                    case.title.as_str(),
                    case.source.as_str(),
                    case.expected.as_deref().unwrap_or(""),
                ]
            })
            .flat_map(|field| field.bytes().chain([0]));
        assert_eq!(
            fingerprint(protected),
            seal,
            "{document}: historical sources"
        );
    }
}

#[test]
fn all_1121_previous_synthesized_records_remain_byte_exact_with_future_tails_allowed() {
    let historical = [
        ("01", 45, 0x1f62_fc2c_18fd_bab7),
        ("02", 47, 0x6583_1a57_9dc1_20ef),
        ("03", 46, 0x5b96_ef50_0df1_3c3f),
        ("04", 45, 0x671e_1aa0_bdf4_f366),
        ("05", 45, 0xb45e_a284_4f1b_28ec),
        ("06", 49, 0x31db_a2d2_9d46_d284),
        ("07", 49, 0xc1ce_32a8_f859_ae98),
        ("08", 49, 0xff62_ab4c_af80_69da),
        ("09", 49, 0x3115_1148_c249_c3fc),
        ("10", 49, 0xc7a8_8eae_ab52_4b62),
        ("11", 49, 0x0df5_cbc7_f1dc_2b80),
        ("12", 49, 0xf902_0ab5_093d_12e7),
        ("13", 49, 0xed7f_e837_a2db_b4b5),
        ("14", 49, 0xea9c_1755_eadf_a501),
        ("15", 49, 0xe4bc_6006_a8c3_2313),
        ("16", 48, 0xb5c9_21cd_0c2b_111f),
        ("17", 49, 0x8675_4bf5_476b_d12e),
        ("18", 49, 0x7799_abc0_643e_0ab4),
        ("19", 51, 0x03e9_28d8_0609_2393),
        ("20", 48, 0xfb37_d596_1b7b_aae6),
        ("21", 120, 0x8978_8ae3_ca5f_9f08),
        ("22", 5, 0xd492_bb31_ff05_4791),
        ("23", 5, 0xa0f9_de1a_17b4_8e6c),
        ("24", 4, 0x911b_aa6b_0a59_211a),
        ("25", 4, 0x6971_3174_1a7a_2ae8),
        ("26", 4, 0x6720_a4fd_0211_18e1),
        ("27", 4, 0xee46_987e_aa50_d746),
        ("28", 4, 0x7b14_c662_c185_defa),
        ("29", 4, 0x4f34_1b1d_e74d_6962),
        ("30", 4, 0xc601_55fb_844f_7d16),
    ];
    let mut total = 0;
    for (number, count, seal) in historical {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("dataset/nucleoid.synth.{number}.jsonl"));
        let source = fs::read_to_string(path).unwrap().replace("\r\n", "\n");
        assert!(source.lines().count() >= count);
        let prefix = source
            .split_inclusive('\n')
            .take(count)
            .flat_map(str::bytes);
        assert_eq!(
            fingerprint(prefix),
            seal,
            "synth-{number}: historical fields"
        );
        total += count;
    }
    assert_eq!(total, 1_121);
}

#[test]
fn four_new_canonical_records_keep_all_fields_without_becoming_normative_runtime_tests() {
    let source = include_str!("../dataset/nucleoid.synth.31.jsonl").replace("\r\n", "\n");
    assert!(source.lines().count() >= 4, "allow future appended records");
    assert_eq!(
        fingerprint(source.split_inclusive('\n').take(4).flat_map(str::bytes)),
        0x84b6_6e3e_a9a1_7303,
        "preserve all four IDs, descriptions, programs and expected returns"
    );
}
