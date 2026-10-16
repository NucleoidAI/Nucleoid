//! Literal text replacement with embedded, Git/session-independent controls.

mod common;

use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

use nucleoid::builtins::string_method;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Ast, Expr, Function, FunctionBody, parse, parse_expression};
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};
use regex::Regex;
use serde::Deserialize;

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "replaceAll regression".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count);
    value
}

fn run(source: &str) -> Runtime {
    let mut runtime = Runtime::new();
    checked(&mut runtime, source, None);
    runtime
}

fn dependencies<'a>(runtime: &'a Runtime, name: &str) -> Vec<&'a str> {
    runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap()
        .dependencies
        .iter()
        .map(NodeKey::as_str)
        .collect()
}

fn documents() -> [(&'static str, Vec<common::Case>); 4] {
    [
        ("spec", common::cases(include_str!("../nucleoid.spec.md"))),
        ("reference", common::cases(include_str!("reference.md"))),
        (
            "readme",
            common::snippets(include_str!("../README.md"), "README.md"),
        ),
        (
            "examples",
            common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
        ),
    ]
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedCase {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

// Exact caller-supplied counterparts executed by the accepted runtime before
// any prototype. Ordinary tests embed them rather than discovering a session.
const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid replaces every literal string match",
    "source": "# Nucleoid replaces every literal string match\n\ntext = \"one|two|one\"\njoined = text.split(\"|\").join(\"/\")\noverlap = \"aaaaa\".split(\"aa\").join(\"X\")\nliteral = \"a.b.a\".split(\".\").join(\"$&\")\nnoMatch = \"abc\".split(\"z\").join(\"\")\nassert(joined, \"one/two/one\")\nassert(overlap, \"XXa\")\nassert(literal, \"a$&b$&a\")\nassert(noMatch, \"abc\")\nassert(\"a.a.a\".replace(\".\", \"/\"), \"a/a.a\")\nassert(text, \"one|two|one\")\njoined\n",
    "expected": "\"one/two/one\""
  },
  {
    "document": "spec",
    "title": "Nucleoid replaces empty string matches at Unicode boundaries",
    "source": "# Nucleoid replaces empty string matches at Unicode boundaries\n\ndef replaceEvery(text, search, replacement):\n    if search == \"\":\n        if text == \"\":\n            return replacement\n        return replacement + text.split(\"\").join(replacement) + replacement\n    return text.split(search).join(replacement)\naccent = String.fromCharCode(233)\nsymbol = String.fromCharCode(128512)\ntext = accent + symbol\nempty = replaceEvery(\"\", \"\", \"|\")\ninserted = replaceEvery(text, \"\", \"|\")\nunchanged = replaceEvery(text, \"\", \"\")\nlegacy = replaceEvery(text, \"\", \"|\")\nassert(empty, \"|\")\nassert(inserted, \"|\" + accent + \"|\" + symbol + \"|\")\nassert(inserted, legacy)\nassert(unchanged, text)\nassert(text, accent + symbol)\n",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid keeps literal replacement reactive",
    "source": "# Nucleoid keeps literal replacement reactive\n\ntext = \"red|blue|red\"\nsearch = \"|\"\nreplacement = \"/\"\nnormalized = text.split(search).join(replacement)\nfrozen = normalized.value\nassert(normalized, \"red/blue/red\")\nreplacement = \"-\"\nassert(normalized, \"red-blue-red\")\ntext = \"blue|green\"\nassert(normalized, \"blue-green\")\nsearch = \"blue\"\nassert(normalized, \"-|green\")\nreplacement = \"cyan\"\nassert(normalized, \"cyan|green\")\nassert(frozen, \"red/blue/red\")\nassert(text, \"blue|green\")\n",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid replaces literal strings in class rules",
    "source": "# Nucleoid replaces literal strings in class rules\n\nclass Page(title: str):\n    this.title = title\n$Page.slug = $Page.title.split(\" \").join(\"-\")\nfirst = Page(\"One Two One\")\nsecond = Page(\"Three Four\")\nslugs = Page.map(page => page.slug)\nassert(first.slug, \"One-Two-One\")\nassert(slugs, [\"One-Two-One\", \"Three-Four\"])\nfirst.title = \"Fresh Page\"\nassert(first.slug, \"Fresh-Page\")\nassert(slugs, [\"Fresh-Page\", \"Three-Four\"])\n$Page.slug = $Page.title.lower().split(\" \").join(\"_\")\nassert(first.slug, \"fresh_page\")\nassert(second.slug, \"three_four\")\nassert(slugs, [\"fresh_page\", \"three_four\"])\n",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back literal replacement argument failures",
    "source": "# Nucleoid rolls back literal replacement argument failures\n\nsource = \"a|b\"\ndef fail():\n    source = \"changed|changed\"\n    throw \"stop\"\ntry:\n    source.split(\"|\").join(fail())\ncatch error:\n    assert(error, \"stop\")\nassert(source, \"a|b\")\nrecovered = source.split(\"|\").join(\"/\")\nassert(recovered, \"a/b\")\n",
    "expected": null
  },
  {
    "document": "reference",
    "title": "Nucleoid normalizes live text records",
    "source": "# Nucleoid normalizes live text records\n\nlabels = [\"first name\", \"last name\", \"first name\"]\nnormalized = labels.map(label => label.split(\" \").join(\"_\"))\nassert(normalized, [\"first_name\", \"last_name\", \"first_name\"])\nlabels.push(\"display name\")\nassert(normalized, [\"first_name\", \"last_name\", \"first_name\", \"display_name\"])\nlabels = [\"given name\", \"last name\"]\nassert(normalized, [\"given_name\", \"last_name\"])\n",
    "expected": null
  },
  {
    "document": "readme",
    "title": "Literal replacements normalize every occurrence",
    "source": "source = \"todo:one|todo:two|todo:one\"\nnormalized = source.split(\"todo:\").join(\"\")\nassert(normalized, \"one|two|one\")\nsource = \"todo:three\"\nassert(normalized, \"three\")\nassert(source, \"todo:three\")\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Literal replacement preserves frozen text normalizers",
    "source": "text = \"archive::notes::today\"\ndef normalize(value) => value.split(\"::\").join(\"/\")\nfrozen = normalize.value\nlive = normalize(text)\nsnapshot = frozen(text)\nassert(live, \"archive/notes/today\")\nassert(snapshot, \"archive/notes/today\")\ndef normalize(value) => value.split(\"::\").join(\"-\")\nassert(live, \"archive-notes-today\")\nassert(snapshot, \"archive/notes/today\")\ntext = \"archive::notes\"\nassert(live, \"archive-notes\")\nassert(snapshot, \"archive/notes\")\n",
    "expected": null
  }
]"###;

const REWRITES: &[&[(&str, &str)]] = &[
    &[
        (
            "text.split(\"|\").join(\"/\")",
            "text.replaceAll(\"|\", \"/\")",
        ),
        (
            "\"aaaaa\".split(\"aa\").join(\"X\")",
            "\"aaaaa\".replaceAll(\"aa\", \"X\")",
        ),
        (
            "\"a.b.a\".split(\".\").join(\"$&\")",
            "\"a.b.a\".replaceAll(\".\", \"$&\")",
        ),
        (
            "\"abc\".split(\"z\").join(\"\")",
            "\"abc\".replaceAll(\"z\", \"\")",
        ),
    ],
    &[
        (
            "empty = replaceEvery(\"\", \"\", \"|\")",
            "empty = \"\".replaceAll(\"\", \"|\")",
        ),
        (
            "inserted = replaceEvery(text, \"\", \"|\")",
            "inserted = text.replaceAll(\"\", \"|\")",
        ),
        (
            "unchanged = replaceEvery(text, \"\", \"\")",
            "unchanged = text.replaceAll(\"\", \"\")",
        ),
    ],
    &[(
        "text.split(search).join(replacement)",
        "text.replaceAll(search, replacement)",
    )],
    &[
        (
            "$Page.title.split(\" \").join(\"-\")",
            "$Page.title.replaceAll(\" \", \"-\")",
        ),
        (
            "$Page.title.lower().split(\" \").join(\"_\")",
            "$Page.title.lower().replaceAll(\" \", \"_\")",
        ),
    ],
    &[
        (
            "source.split(\"|\").join(fail())",
            "source.replaceAll(\"|\", fail())",
        ),
        (
            "source.split(\"|\").join(\"/\")",
            "source.replaceAll(\"|\", \"/\")",
        ),
    ],
    &[(
        "label.split(\" \").join(\"_\")",
        "label.replaceAll(\" \", \"_\")",
    )],
    &[(
        "source.split(\"todo:\").join(\"\")",
        "source.replaceAll(\"todo:\", \"\")",
    )],
    &[
        (
            "value.split(\"::\").join(\"/\")",
            "value.replaceAll(\"::\", \"/\")",
        ),
        (
            "value.split(\"::\").join(\"-\")",
            "value.replaceAll(\"::\", \"-\")",
        ),
    ],
];

#[test]
fn exact_eight_counterparts_change_only_sixteen_operations_and_execute_all_forty_assertions() {
    let sealed: Vec<SealedCase> = serde_json::from_str(ACCEPTED).unwrap();
    let documents = documents();
    assert_eq!(sealed.len(), 8);
    let mut totals = (0, 0, 0);
    for (index, sealed) in sealed.iter().enumerate() {
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == sealed.document)
            .unwrap()
            .1;
        let matches = cases
            .iter()
            .enumerate()
            .filter(|(_, case)| case.title == sealed.title)
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "{}", sealed.title);
        let (position, case) = matches[0];
        assert_eq!(position, [383, 384, 385, 386, 387, 77, 28, 83][index]);
        assert_eq!(case.expected, sealed.expected);
        let mut rewritten = sealed.source.clone();
        for &(old, new) in REWRITES[index] {
            assert_eq!(rewritten.matches(old).count(), 1, "{old}");
            rewritten = rewritten.replacen(old, new, 1);
        }
        assert_eq!(
            case.source.trim(),
            rewritten.trim(),
            "non-operation source bytes"
        );
        let control_ast = parse(&sealed.source).unwrap();
        let actual_ast = parse(&case.source).unwrap();
        assert_ne!(
            control_ast, actual_ast,
            "runtime-call RHSs intentionally differ"
        );
        let assertions = common::assertions::protected_assertions(&control_ast);
        assert_eq!(assertions.len(), [6, 5, 7, 7, 3, 3, 3, 6][index]);
        assert_eq!(
            common::assertions::protected_assertions(&actual_ast),
            assertions
        );
        let control = common::Case {
            title: sealed.title.clone(),
            source: sealed.source.clone(),
            expected: sealed.expected.clone(),
        };
        let mut runtimes = [Runtime::new(), Runtime::new()];
        for (runtime, form) in runtimes.iter_mut().zip([case, &control]) {
            common::check_with_count(runtime, form, assertions.len())
                .unwrap_or_else(|reason| panic!("{}: {reason}", sealed.title));
            assert_eq!(runtime.assertions_run(), assertions.len());
        }
        let outputs: &[&str] = match index {
            0 => &["text", "joined", "overlap", "literal", "noMatch"],
            1 => &[
                "accent",
                "symbol",
                "text",
                "empty",
                "inserted",
                "unchanged",
                "legacy",
            ],
            2 => &["text", "search", "replacement", "normalized", "frozen"],
            3 => &["first", "second", "slugs"],
            4 => &["source", "recovered"],
            5 => &["labels", "normalized"],
            6 => &["source", "normalized"],
            7 => &["text", "live", "snapshot"],
            _ => unreachable!(),
        };
        // Compare protected observable outputs, not deliberately changed
        // function bodies or independent function Arc identities.
        for name in outputs {
            assert_eq!(
                runtimes[0]
                    .serialize_json(runtimes[0].state.variable(name).unwrap())
                    .unwrap(),
                runtimes[1]
                    .serialize_json(runtimes[1].state.variable(name).unwrap())
                    .unwrap(),
                "{name}",
            );
        }
        totals.0 += assertions.len();
        totals.1 += usize::from(sealed.expected.is_some());
        totals.2 += REWRITES[index].len();
    }
    assert_eq!(totals, (40, 1, 16));
}

#[test]
fn empty_search_helper_body_and_actual_legacy_call_are_byte_exact() {
    let controls: Vec<SealedCase> = serde_json::from_str(ACCEPTED).unwrap();
    let candidate = &documents()[0].1[384].source;
    let helper = |source: &str| {
        source
            .split_once("def replaceEvery")
            .unwrap()
            .1
            .split_once("accent = ")
            .unwrap()
            .0
            .to_string()
    };
    assert_eq!(helper(candidate), helper(&controls[1].source));
    assert_eq!(
        candidate
            .matches("legacy = replaceEvery(text, \"\", \"|\")")
            .count(),
        1
    );
    let runtime = run(candidate);
    assert_eq!(
        runtime.state.variable("legacy"),
        runtime.state.variable("inserted")
    );
}

// Length-delimited FNV-1a regression seals from a read-only audit of the
// pinned 8577859 snapshot using tests/common. These are not lexer counts or
// security digests. All 571 historical cases retain even boundary whitespace.
fn seal_field(hash: &mut u64, bytes: &[u8]) {
    for byte in (bytes.len() as u64).to_le_bytes().iter().chain(bytes) {
        *hash = (*hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
}

fn history_signatures(cases: &[common::Case]) -> [u64; 4] {
    let mut hashes = [0xcbf2_9ce4_8422_2325; 4];
    for case in cases {
        for text in [&case.title, &case.source, &format!("{:?}", case.expected)] {
            seal_field(&mut hashes[0], text.as_bytes());
        }
        let ast = parse(&case.source).unwrap();
        seal_field(&mut hashes[1], format!("{ast:?}").as_bytes());
        for assertion in common::assertions::protected_assertions(&ast) {
            seal_field(&mut hashes[2], assertion.as_bytes());
        }
        seal_field(
            &mut hashes[3],
            nucleoid::lang::ast::generator::generate_all(&ast).as_bytes(),
        );
    }
    hashes
}

#[test]
fn all_571_parent_sources_keep_exact_boundaries_order_comments_asts_canonical_assertions_returns() {
    let expected = [
        (
            383,
            [
                15188015017421260451,
                6297702563039077619,
                134657377144635479,
                17046315188128762997,
            ],
        ),
        (
            77,
            [
                9194139556706992802,
                12605047110061484984,
                3326160933199036696,
                18249000666025700349,
            ],
        ),
        (
            28,
            [
                11369271977386378935,
                10484175198589423031,
                18269221152758357673,
                3093834341845860914,
            ],
        ),
        (
            83,
            [
                256389272184628546,
                12133003150586287149,
                17673460078947350630,
                8622323037974767122,
            ],
        ),
    ];
    let mut total = 0;
    let mut clock_programs = 0;
    for ((document, cases), (count, signatures)) in documents().into_iter().zip(expected) {
        assert!(cases.len() >= count);
        assert_eq!(
            history_signatures(&cases[..count]),
            signatures,
            "{document}"
        );
        total += count;
        clock_programs += cases[..count]
            .iter()
            .filter(|case| case.source.contains("Date.now()") || case.source.contains("Date()"))
            .count();
    }
    assert_eq!(total, 571);
    // Their source/AST/canonical assertions are protected above; wall-clock
    // values are deliberately not compared between separate executions.
    assert_eq!(clock_programs, 4);
}

#[test]
fn literal_matches_are_non_overlapping_left_to_right_and_do_not_revisit_insertions() {
    for (receiver, search, replacement, expected) in [
        ("", "x", "Y", ""),
        ("abc", "z", "Y", "abc"),
        ("abc", "abc", "Y", "Y"),
        ("abcabc", "abc", "Y", "YY"),
        ("aaaaa", "aa", "X", "XXa"),
        ("ababa", "aba", "X", "Xba"),
        ("aaa", "a", "aa", "aaaaaa"),
        ("aaa", "a", "", ""),
        ("a.b.a", ".", "$&", "a$&b$&a"),
        ("a$a", "$", "\\1$`$'$&", "a\\1$`$'$&a"),
        ("a\0a\0", "\0", "$1\\n", "a$1\\na$1\\n"),
        ("a\\a", "\\", "$$", "a$$a"),
    ] {
        assert_eq!(
            string_method(
                receiver,
                "replaceAll",
                &[Value::string(search), Value::string(replacement)]
            )
            .unwrap(),
            Value::string(expected),
        );
    }
}

#[test]
fn empty_search_uses_scalars_not_bytes_utf16_or_graphemes() {
    for (receiver, replacement, expected) in [
        ("", "|", "|"),
        ("", "", ""),
        ("ab", "|", "|a|b|"),
        ("é😀", "|", "|é|😀|"),
        ("e\u{301}", "|", "|e|\u{301}|"),
        ("\0é", "\\$", "\\$\0\\$é\\$"),
        ("é😀", "", "é😀"),
    ] {
        assert_eq!(
            string_method(
                receiver,
                "replaceAll",
                &[Value::string(""), Value::string(replacement)]
            )
            .unwrap(),
            Value::string(expected),
        );
    }
}

#[test]
fn regex_and_function_values_are_stringified_not_patterns_or_callbacks() {
    run(r#"
assert("/a+//a+/".replaceAll(/a+/, "x"), "xx")
assert("aaa".replaceAll(/a+/, "x"), "aaa")
assert("[function]|[function]".replaceAll(value => missing, "x"), "x|x")
assert("a|a".replaceAll("a", value => missing), "[function]|[function]")
"#);
}

#[test]
fn every_value_variant_and_numeric_edge_uses_the_existing_replace_coercion() {
    let values = [
        (Value::Undefined, "undefined"),
        (Value::Null, "null"),
        (Value::Bool(false), "false"),
        (Value::Bool(true), "true"),
        (Value::Number(-0.0), "0"),
        (Value::number(-3.5), "-3.5"),
        (Value::number(f64::NAN), "NaN"),
        (Value::number(f64::INFINITY), "Infinity"),
        (Value::number(f64::NEG_INFINITY), "-Infinity"),
        (Value::string(""), ""),
        (Value::string("$&\\1"), "$&\\1"),
        (Value::List(vec![]), ""),
        (
            Value::List(vec![Value::Null, Value::Undefined]),
            "null,undefined",
        ),
        (
            Value::List(vec![
                Value::number(1),
                Value::Bool(true),
                Value::string("x"),
            ]),
            "1,true,x",
        ),
        (Value::Object(ObjectId::named("probe")), "[object probe]"),
        (Value::Class("Probe".to_string()), "class Probe"),
        (Value::Date(0), "Thu Jan 01 1970 00:00:00 GMT+0000"),
        (Value::Regex(Arc::new(Regex::new("a+").unwrap())), "/a+/"),
        (
            Value::Function(Arc::new(Function {
                name: None,
                parameters: vec![],
                body: FunctionBody::Expression(Box::new(Expr::Null)),
            })),
            "[function]",
        ),
    ];
    for (value, spelling) in values {
        assert_eq!(value.to_string(), spelling);
        assert_eq!(
            string_method("x|x", "replaceAll", &[Value::string("x"), value.clone()]).unwrap(),
            Value::string(format!("{spelling}|{spelling}")),
        );
        assert_eq!(
            string_method(
                &format!("{spelling}|{spelling}"),
                "replaceAll",
                &[value.clone(), Value::string("Q")]
            )
            .unwrap(),
            Value::string("Q|Q"),
        );
        assert_eq!(
            string_method("x", "replace", &[Value::string("x"), value.clone()]).unwrap(),
            Value::string(spelling),
        );
        assert_eq!(
            string_method(spelling, "replace", &[value, Value::string("Q")]).unwrap(),
            Value::string("Q"),
        );
    }
}

#[test]
fn omitted_and_literal_values_do_not_manufacture_null_or_undefined_reads() {
    run(r#"
assert("undefined|undefined".replaceAll(), "undefined|undefined")
assert("a|a".replaceAll("a"), "undefined|undefined")
assert("abc".replaceAll(), "abc")
assert("null|null".replaceAll(null, "x"), "x|x")
assert("a|a".replaceAll("a", null), "null|null")
assert("a|a".replaceAll("a", true), "true|true")
assert("1|1".replaceAll(1, 2), "2|2")
literal = "null|null".replaceAll(null, "x")
assert(literal, "x|x")
defaulted = "a|a".replaceAll("a")
assert(defaulted, "undefined|undefined")
"#);
    assert_eq!(
        string_method(
            "x",
            "replaceAll",
            &[
                Value::string("x"),
                Value::string("Q"),
                Value::Undefined,
                Value::Null
            ]
        )
        .unwrap(),
        Value::string("Q"),
    );
}

#[test]
fn receiver_and_all_supplied_expressions_execute_once_in_order_before_the_operation() {
    run(r#"
log = Object()
log.order = 0
def receiver():
    log.order = log.order.value * 10 + 1
    return "a|a"
def search():
    log.order = log.order.value * 10 + 2
    return "|"
def replacement():
    log.order = log.order.value * 10 + 3
    return "/"
def extra():
    log.order = log.order.value * 10 + 4
    return null
result = receiver().replaceAll(search(), replacement(), extra(), log.order = log.order.value * 10 + 5)
assert(result, "a/a")
assert(log.order, 12345)
noMatch = "".replaceAll("x", "y", log.order = 6)
assert(noMatch, "")
assert(log.order, 6)
"#);
}

#[test]
fn receiver_and_earlier_argument_values_survive_later_argument_mutations() {
    run(r#"
box = Object()
box.text = "a|a"
box.search = "|"
box.replacement = "/"
result = box.text.replaceAll(box.search, box.replacement, box.text = "later", box.search = "a", box.replacement = "-")
assert(result, "a/a")
assert(box.text, "later")
assert(box.search, "a")
assert(box.replacement, "-")
"#);
}

#[test]
fn argument_failures_keep_precedence_positions_and_rollback() {
    for (expression, kind, message) in [
        (
            r#""a".replaceAll(missing, Math.random())"#,
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            r#""a".replaceAll("a", Math.random(), missing)"#,
            ErrorKind::Type,
            "Math.random is not supported",
        ),
        (
            r#""a".replaceAll("a", "b", missing)"#,
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            r#""".replaceAll("x", "y", missing)"#,
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            r#"String.replaceAll("a", "b", missing)"#,
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            r#"String.replaceAll("a", "b")"#,
            ErrorKind::Type,
            "String.replaceAll is not a function",
        ),
        (
            r#"(1).replaceAll(missing)"#,
            ErrorKind::Type,
            "number.replaceAll is not a function",
        ),
        (
            r#"missing.replaceAll(Math.random())"#,
            ErrorKind::Reference,
            "missing is not defined",
        ),
    ] {
        let mut runtime = Runtime::new();
        let error = runtime
            .run(&format!("\nanswer = {expression}"))
            .unwrap_err();
        assert_eq!(error.kind(), kind, "{expression}");
        assert_eq!(error.message(), message, "{expression}");
        assert_eq!(error.position(), Some(Position::new(2, 1)), "{expression}");
        assert!(runtime.state.variable("answer").is_none());
    }
}

#[test]
fn failure_stops_later_effects_and_restores_state_and_graph_before_recovery() {
    let mut runtime = run(r#"
box = Object()
box.text = "a|a"
box.calls = 0
def fail():
    box.calls = box.calls.value + 1
    box.text = "changed"
    throw "REPLACEMENT_ABORT"
result = box.text.replaceAll("|", "/")
"#);
    let state = format!("{:?}", runtime.state);
    let keys = runtime.graph.keys().cloned().collect::<Vec<_>>();
    let reads = dependencies(&runtime, "result")
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let error = runtime
        .run(r#"result = box.text.replaceAll("|", fail(), box.calls = 99)"#)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Thrown);
    assert_eq!(
        error.thrown_value(),
        Some(&Value::string("REPLACEMENT_ABORT"))
    );
    assert_eq!(error.position(), Some(Position::new(1, 1)));
    assert_eq!(format!("{:?}", runtime.state), state);
    assert_eq!(runtime.graph.keys().cloned().collect::<Vec<_>>(), keys);
    assert_eq!(dependencies(&runtime, "result"), reads);
    checked(
        &mut runtime,
        r#"
assert(box.calls, 0)
assert(box.text, "a|a")
assert(result, "a/a")
box.text = "b|b"
assert(result, "b/b")
"#,
        None,
    );
}

#[test]
fn real_null_and_undefined_reads_defer_and_recover_without_changing_literal_defaults() {
    let mut runtime = run(r#"
class Probe
probe = Probe()
nullable = null
literal = "null|null".replaceAll(null, "x")
read = "null|null".replaceAll(nullable, "x")
missing = "undefined|undefined".replaceAll(probe.search, "x")
legacyMissing = "undefined".replace(probe.search, "x")
assert(literal, "x|x")
assert(read, null)
assert(missing, "x|x")
assert(legacyMissing, "x")
nullable = "null"
assert(read, "x|x")
probe.search = "undefined"
assert(missing, "x|x")
probe.search = null
assert(missing, null)
probe.search = "z"
assert(missing, "undefined|undefined")
"#);
    assert!(dependencies(&runtime, "read").contains(&"nullable"));
    assert!(dependencies(&runtime, "missing").contains(&"probe.search"));
    checked(
        &mut runtime,
        r#"
delete probe.search
assert(missing, "x|x")
assert(legacyMissing, "x")
probe.search = "undefined"
assert(missing, "x|x")
"#,
        None,
    );
}

#[test]
fn actual_parameter_reads_follow_the_same_null_handling_as_first_only_replace() {
    run(r#"
def normalize(value, search, replacement) => value.replaceAll(search, replacement)
def legacy(value, search, replacement) => value.replace(search, replacement)
actual = normalize("null", null, "x")
control = legacy("null", null, "x")
assert(actual, null)
assert(actual, control)
actualMissing = normalize("undefined", "undefined")
controlMissing = legacy("undefined", "undefined")
assert(actualMissing, controlMissing)
assert(actualMissing, null)
"#);
}

#[test]
fn nullish_receivers_skip_arguments_and_preserve_dependency_driven_recovery() {
    run(r#"
class Probe
probe = Probe()
log = Object()
log.calls = 0
result = probe.text.replaceAll("|", "/", log.calls = log.calls.value + 1)
assert(result, null)
assert(log.calls, 0)
probe.text = "a|a"
assert(result, "a/a")
assert(log.calls, 1)
probe.text = null
assert(result, null)
assert(log.calls, 1)
probe.text = "b|b"
assert(result, "b/b")
assert(log.calls, 2)
"#);
    let mut runtime = Runtime::new();
    assert_eq!(
        runtime.run(r#"null.replaceAll(missing)"#).unwrap(),
        Value::Undefined
    );
}

#[test]
fn unused_extra_reads_keep_dependencies_null_tracking_and_later_errors() {
    let runtime = run(r#"
extra = 1
result = "a|a".replaceAll("|", "/", extra)
empty = "".replaceAll("x", "y", extra)
assert(result, "a/a")
assert(empty, "")
extra = null
assert(result, null)
assert(empty, null)
extra = 2
assert(result, "a/a")
assert(empty, "")
"#);
    assert_eq!(dependencies(&runtime, "result"), ["extra"]);
    assert_eq!(dependencies(&runtime, "empty"), ["extra"]);
}

#[test]
fn live_search_replacement_and_receiver_reads_are_replaced_not_accumulated() {
    let mut runtime = run(r#"
class Label(text):
    this.text = text
first = Label("one|one")
second = Label("two|two")
selected = first
search = "|"
replacement = "/"
normalized = selected.text.replaceAll(search, replacement)
assert(normalized, "one/one")
"#);
    assert_eq!(
        dependencies(&runtime, "normalized"),
        ["selected", "first.text", "search", "replacement"]
    );
    checked(
        &mut runtime,
        r#"
selected = second
assert(normalized, "two/two")
first.text = "obsolete"
assert(normalized, "two/two")
"#,
        None,
    );
    assert_eq!(
        dependencies(&runtime, "normalized"),
        ["selected", "second.text", "search", "replacement"]
    );
    assert!(!runtime.graph.reaches(
        &NodeKey::property(&ObjectId::named("first"), "text"),
        &NodeKey::variable("normalized"),
    ));
    checked(
        &mut runtime,
        r#"
replacement = "-"
assert(normalized, "two-two")
search = "two"
assert(normalized, "-|-")
normalized = "fixed"
replacement = "new"
second.text = "changed"
assert(normalized, "fixed")
"#,
        None,
    );
    assert!(dependencies(&runtime, "normalized").is_empty());
}

#[test]
fn property_and_class_rules_keep_member_binding_deletion_and_recovery() {
    let runtime = run(r#"
class Label:
    def normalize(search, replacement) => self.text.replaceAll(search, replacement)
$Label.normalized = $Label.text.replaceAll(" ", "-")
first = Label()
second = Label()
assert(first.normalized, null)
first.text = "one two one"
second.text = "three four"
assert(first.normalized, "one-two-one")
assert(second.normalized, "three-four")
assert(first.normalize(" ", "_"), "one_two_one")
delete first.text
assert(first.normalized, null)
first.text = "fresh label"
assert(first.normalized, "fresh-label")
assert(second.normalized, "three-four")
$Label.normalized = $Label.text.upper().replaceAll(" ", "_")
assert(first.normalized, "FRESH_LABEL")
assert(second.normalized, "THREE_FOUR")
box = Object()
box.source = "a|a"
box.result = box.source.replaceAll("|", "/")
box.source = "b|b"
assert(box.result, "b/b")
"#);
    for name in ["first", "second"] {
        let id = ObjectId::named(name);
        assert!(runtime.graph.reaches(
            &NodeKey::property(&id, "text"),
            &NodeKey::property(&id, "normalized")
        ));
    }
}

#[test]
fn live_and_frozen_function_definitions_keep_live_inputs_and_within_runtime_identity() {
    let runtime = run(r#"
text = "a::b::a"
separator = "/"
def normalize(value) => value.replaceAll("::", separator)
alias = normalize
frozen = normalize.value
frozenAlias = frozen
live = alias(text)
saved = frozen(text)
snapshot = live.value
assert(frozenAlias(text), "a/b/a")
def normalize(value) => value.replaceAll("::", "-")
assert(live, "a-b-a")
assert(saved, "a/b/a")
separator = "_"
assert(saved, "a_b_a")
assert(snapshot, "a/b/a")
text = "c::d"
assert(live, "c-d")
assert(saved, "c_d")
assert(frozenAlias(text), "c_d")
"#);
    assert!(dependencies(&runtime, "snapshot").is_empty());
    assert!(
        runtime
            .graph
            .reaches(&NodeKey::function("normalize"), &NodeKey::variable("live"))
    );
    assert!(
        !runtime
            .graph
            .reaches(&NodeKey::function("normalize"), &NodeKey::variable("saved"))
    );
    let (Value::Function(frozen), Value::Function(alias)) = (
        runtime.state.variable("frozen").unwrap(),
        runtime.state.variable("frozenAlias").unwrap(),
    ) else {
        panic!("frozen function values")
    };
    assert!(Arc::ptr_eq(frozen, alias));
}

#[test]
fn failed_redefinition_restores_the_live_text_rule_then_allows_recovery() {
    let runtime = run(r#"
text = "a|a"
def normalize(value) => value.replaceAll("|", "/")
result = normalize(text)
try:
    def normalize(value):
        throw "BAD_NORMALIZER"
catch error:
    assert(error, "BAD_NORMALIZER")
assert(result, "a/a")
text = "b|b"
assert(result, "b/b")
def normalize(value) => value.replaceAll("|", "-")
assert(result, "b-b")
"#);
    assert!(runtime.graph.reaches(
        &NodeKey::function("normalize"),
        &NodeKey::variable("result")
    ));
}

#[test]
fn ordinary_scope_and_noncapture_and_caller_this_are_unchanged() {
    let runtime = run(r#"
def normalize(value, search, replacement):
    local = value.replaceAll(search, replacement)
    return local
assert(normalize("a|b", "|", "/"), "a/b")
"#);
    let control = run(r#"
def normalize(value, search, replacement):
    local = value.replace(search, replacement)
    return local
assert(normalize("a|b", "|", "/"), "a/b")
"#);
    // Preserve the accepted assignment/scope behavior, including any global
    // assignment effects; this method does not introduce lexical-local fixes.
    assert_eq!(
        runtime.state.variable("local"),
        control.state.variable("local")
    );
    let mut runtime = Runtime::new();
    let error = runtime
        .run(
            r#"
def factory(search) => value => value.replaceAll(search, "-")
factory("|")("a|a")
"#,
        )
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "search is not defined");
    let mut scope = Scope::new();
    scope.push();
    scope.declare("text", Value::string("a|a"));
    scope.declare("needle", Value::string("|"));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    let expression = parse_expression(r#"text.replaceAll(needle, "/")"#).unwrap();
    let value = Ast::convert(&expression)
        .resolve(&mut runtime, &mut scope)
        .unwrap();
    assert_eq!(value, Value::string("a/a"));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.retrieve("text"), Some(&Value::string("a|a")));
    assert_eq!(scope.retrieve("needle"), Some(&Value::string("|")));
}

#[test]
fn custom_global_local_object_and_instance_names_keep_old_dispatch() {
    run(r#"
def replaceAll(value) => value + 1
assert(replaceAll(2), 3)
namespace = Object()
namespace.replaceAll = replaceAll
assert(namespace.replaceAll(3), 4)
class Handler(bias):
    this.bias = bias
    def replaceAll(value) => self.bias + value
handler = Handler(10)
assert(handler.replaceAll(2), 12)
def apply(replaceAll) => replaceAll(2)
assert(apply(value => value * 3), 6)
namespace.replaceAll = 9
assert(namespace.replaceAll, 9)
replaceAll = 7
assert(replaceAll, 7)
assert("a|a".replaceAll("|", "/"), "a/a")
"#);
}

#[test]
fn shadowed_string_namespaces_keep_ordinary_global_and_parameter_lookup() {
    run(r#"
String = Object()
String.replaceAll = value => value + 1
assert(String.replaceAll(2), 3)
def useNamespace(String) => String.replaceAll(3)
assert(useNamespace(String), 4)
assert("a|a".replaceAll("|", "/"), "a/a")
"#);
}

#[test]
fn native_properties_static_namespace_and_aliases_do_not_gain_callable_forms() {
    run(r#"
native = "abc".replaceAll
legacy = "abc".replace
assert(native, null)
assert(legacy, null)
class Holder
holder = Holder()
holder.replaceAll = value => value + 2
assert(holder.replaceAll(3), 5)
def callNative():
    native = "abc".replaceAll
    return native("a", "b")
assert(callNative(), null)
"#);
    for (expression, message) in [
        (
            r#""a".replaceall("a", "b")"#,
            "a.replaceall is not a function",
        ),
        (
            r#""a".replace_all("a", "b")"#,
            "a.replace_all is not a function",
        ),
        (
            r#"String.replaceAll("a", "b")"#,
            "String.replaceAll is not a function",
        ),
    ] {
        let error = Runtime::new().run(expression).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), message);
    }
}

#[test]
fn legacy_replace_split_join_indexing_and_other_string_methods_remain_unchanged() {
    run(r#"
assert("a.a.a".replace(".", "/"), "a/a.a")
assert("ab".replace("", "|"), "|ab")
assert("".replace("", "|"), "|")
assert("a|a".split("|").join("/"), "a/a")
assert("".split(""), [])
accent = String.fromCharCode(233)
symbol = String.fromCharCode(128512)
text = accent + symbol
assert(text[0], accent)
assert(text[1], symbol)
assert(text.split(""), [accent, symbol])
assert(text.length, 2)
assert("  Ab  ".trim().lower(), "ab")
assert("ab".upper(), "AB")
assert("abc".includes("b"), true)
assert("abc".startsWith("a"), true)
assert("abc".endsWith("c"), true)
assert("ab".repeat(2), "abab")
assert("abc".slice(1), "bc")
"#);
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrainingRecord {
    id: String,
    description: String,
    code: String,
    #[serde(rename = "return")]
    expected: Option<String>,
}

#[test]
fn five_new_self_contained_training_records_development_execute_without_becoming_normative() {
    let records: Vec<TrainingRecord> = include_str!("../dataset/nucleoid.synth.23.jsonl")
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(records.len(), 5);
    for (index, record) in records.into_iter().enumerate() {
        assert_eq!(record.id, format!("synth-23-{:04}", index + 1));
        assert!(!record.description.trim().is_empty());
        let mut runtime = Runtime::new();
        checked(&mut runtime, &record.code, record.expected.as_deref());
    }
}

#[test]
fn native_cli_files_compare_literal_input_and_output_bytes_including_true_eof() {
    for (index, (source, success, stdout, stderr)) in [
        (
            r#""one|two|one".replaceAll("|", "/")"#,
            true,
            "\"one/two/one\"\n",
            "",
        ),
        (r#""aaaaa".replaceAll("aa", "X")"#, true, "\"XXa\"\n", ""),
        (r#""".replaceAll("", "|")"#, true, "\"|\"\n", ""),
        (r#""é😀".replaceAll("", "|")"#, true, "\"|é|😀|\"\n", ""),
        (
            r#""a.a".replaceAll(".", "$&\\1")"#,
            true,
            "\"a$&\\\\1a\"\n",
            "",
        ),
        (
            "\"a\0a\".replaceAll(\"a\", \"$&\")",
            true,
            "\"$&\\u0000$&\"\n",
            "",
        ),
        (r#""a.a.a".replace(".", "/")"#, true, "\"a/a.a\"\n", ""),
        (r#""undefined".replaceAll()"#, true, "\"undefined\"\n", ""),
        (
            r#"String.replaceAll("a", "b")"#,
            false,
            "",
            "TypeError: String.replaceAll is not a function (line 1, column 1)\n",
        ),
        (
            r#"(1).replaceAll(missing)"#,
            false,
            "",
            "TypeError: number.replaceAll is not a function (line 1, column 1)\n",
        ),
        (
            r#""a".replaceAll("a", "b", missing)"#,
            false,
            "",
            "ReferenceError: missing is not defined (line 1, column 1)\n",
        ),
        (
            r#""a".replaceAll("#,
            false,
            "",
            "SyntaxError: Unexpected end of input (line 1, column 16)\n",
        ),
        (
            "\"a\".replaceAll(\n",
            false,
            "",
            "SyntaxError: Unexpected end of input (line 2, column 1)\n",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!(
                "nuc42-replace-all-{}-{index}.nuc",
                std::process::id()
            ));
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap()
            .write_all(source.as_bytes())
            .unwrap();
        assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg("--json")
            .arg(&path)
            .stdin(Stdio::null())
            .output();
        let cleanup = fs::remove_file(&path);
        let output = output.unwrap();
        cleanup.unwrap();
        assert_eq!(output.status.success(), success, "{source}: {output:?}");
        assert_eq!(output.stdout, stdout.as_bytes(), "{source}");
        assert_eq!(output.stderr, stderr.as_bytes(), "{source}");
    }
}

fn stdin_cli(source: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn native_stdin_recovers_after_errors_and_keeps_inherited_repl_newline_framing() {
    let banner = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";
    let source = "source = \"a|a\"\nresult = source.replaceAll(\"|\", \"/\")\nString.replaceAll(\"a\", \"b\")\nsource = \"b|b\"\nresult";
    assert!(!source.ends_with('\n'));
    let output = stdin_cli(source);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stderr,
        b"TypeError: String.replaceAll is not a function (line 1, column 1)\n"
    );
    assert_eq!(
        output.stdout,
        format!("{banner}\"a|a\"\n\"a/a\"\n\"b|b\"\n\"b/b\"\n").as_bytes()
    );
    // Files above test actual unterminated EOF. Interactive stdin still
    // appends its inherited newline; this revision does not repair framing.
    for source in [r#""a".replaceAll("#, "\"a\".replaceAll(\n"] {
        let output = stdin_cli(source);
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, banner.as_bytes());
        assert_eq!(
            output.stderr,
            b"SyntaxError: Unexpected end of input (line 2, column 1)\n"
        );
    }
}
