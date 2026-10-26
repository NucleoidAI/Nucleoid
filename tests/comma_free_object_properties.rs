//! Optional explicit-property separators, with self-contained accepted controls.
//! Ordinary behavior tests need neither Git nor proposal-session artifacts.

mod common;
#[allow(dead_code)]
#[path = "support/conditional_headers.rs"]
mod conditional_headers;

#[path = "../build_statements.rs"]
mod build_statements;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, FunctionBody, Stmt, parse, parse_expression, parse_program};
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Counterpart {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

// Exact V3 counterparts, actually executed before the immutable baseline and
// candidate creation. V1/V2 failed preparation inputs are deliberately not used.
const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid constructs comma-free explicit records",
    "source": "# Nucleoid constructs comma-free explicit records\n\namount = 4\nunit = \"C\"\nreading = {amount: amount, unit: unit, active: true}\nassert(reading.amount, 4)\nassert(reading.unit, \"C\")\nassert(reading.active, true)\namount = 9\nassert(reading.amount, 9)\nunit = \"F\"\nassert(reading.unit, \"F\")\nreading.unit\n",
    "expected": "\"F\""
  },
  {
    "document": "spec",
    "title": "Nucleoid preserves comma-free record keys and replacement order",
    "source": "# Nucleoid preserves comma-free record keys and replacement order\n\nseed = 3\nrecord = {\"state\": \"open\", 1: \"first\", \"1\": \"latest\", copy: seed, copy: seed + 1}\nassert(record.state, \"open\")\nassert(record[\"1\"], \"latest\")\nassert(record.copy, 4)\nseed = 8\nassert(record.copy, 9)\nrecord.state = \"closed\"\nassert(record.state, \"closed\")\n",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid nests comma-free records without changing list or call separators",
    "source": "# Nucleoid nests comma-free records without changing list or call separators\n\ndef pair(left, right) => left + right\nfirst = 2\npacket = {meta: {kind: \"sample\", ready: true}, data: [{reading: first, unit: \"C\"}, {reading: pair(first, 3), unit: \"K\"}], tag: \"batch\"}\nassert(packet.meta.kind, \"sample\")\nassert(packet.data[0].reading, 2)\nassert(packet.data[1].reading, 5)\nassert(packet.data[1].unit, \"K\")\nfirst = 7\nassert(packet.data[0].reading, 7)\nassert(packet.data[1].reading, 10)\n",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid preserves scope and snapshots in comma-free records",
    "source": "# Nucleoid preserves scope and snapshots in comma-free records\n\ndef package(amount):\n    return {reading: amount, initial: amount.value, label: \"sample\"}\nsource = 3\nitem = package(source)\nview = {current: source, saved: source.value, packed: item.reading}\nassert(item.reading, 3)\nassert(view.current, 3)\nassert(view.saved, 3)\nsource = 6\nassert(item.reading, 6)\nassert(view.current, 6)\nassert(view.saved, 3)\nassert(view.packed, 6)\n",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back failed comma-free record values",
    "source": "# Nucleoid rolls back failed comma-free record values\n\nstate = 1\nrecord = {status: \"old\", marker: state}\nmessage = \"\"\ntry:\n    state = 2\n    record = {status: \"new\", marker: missing, status: \"last\"}\ncatch error:\n    message = error\nassert(message, ReferenceError(\"missing is not defined\"))\nassert(state, 1)\nassert(record.status, \"old\")\nassert(record.marker, 1)\nstate = 3\nassert(record.marker, 3)\nrecord.status\n",
    "expected": "\"old\""
  },
  {
    "document": "reference",
    "title": "Nucleoid builds compact routing records",
    "source": "# Nucleoid builds compact routing records\n\ndef lane(urgent):\n    if urgent: return \"fast\"\n    return \"normal\"\nkind = \"invoice\"\nurgent = false\nsummary = {kind: kind, lane: lane(urgent), active: true}\nassert(summary.kind, \"invoice\")\nassert(summary.lane, \"normal\")\nurgent = true\nassert(summary.lane, \"fast\")\nkind = \"alert\"\nassert(summary.kind, \"alert\")\n",
    "expected": null
  },
  {
    "document": "readme",
    "title": "Explicit object fields omit redundant commas",
    "source": "# Explicit object fields omit redundant commas\n\nname = \"sensor\"\nreading = 4\nrecord = {name: name, reading: reading, unit: \"C\"}\nassert(record.name, \"sensor\")\nassert(record.reading, 4)\nreading = 9\nassert(record.reading, 9)\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Compact records keep class rules independent",
    "source": "# Compact records keep class rules independent\n\nclass Ticket\nfirst = Ticket()\nsecond = Ticket()\nfirst.amount = 2\nsecond.amount = 5\n$Ticket.payload = {reading: $Ticket.amount, category: \"sample\", ready: true}\nassert(first.payload.reading, 2)\nassert(second.payload.reading, 5)\nfirst.amount = 7\nassert(first.payload.reading, 7)\nassert(second.payload.reading, 5)\nassert(first.payload.ready, true)\n",
    "expected": null
  }
]"###;

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

/// Only these explicit-property boundaries occur in the eight sealed fixtures.
/// This is not a lexer, general comma remover or source-to-source framework.
fn counterpart(source: &str) -> (String, usize) {
    let mut source = source.to_string();
    let mut sites = 0;
    for boundary in [
        ", unit:",
        ", active:",
        ", 1:",
        ", \"1\":",
        ", copy:",
        ", ready:",
        ", data:",
        ", tag:",
        ", initial:",
        ", label:",
        ", saved:",
        ", packed:",
        ", marker:",
        ", status:",
        ", lane:",
        ", reading:",
        ", category:",
    ] {
        sites += source.matches(boundary).count();
        source = source.replace(boundary, &boundary[1..]);
    }
    (source, sites)
}

fn equivalent(actual: &str, comma: &str) -> Vec<Stmt> {
    let actual = parse(actual).unwrap_or_else(|error| panic!("{actual}\n{error}"));
    let comma = parse(comma).unwrap_or_else(|error| panic!("{comma}\n{error}"));
    assert_eq!(actual, comma);
    assert_eq!(generate_all(&actual), generate_all(&comma));
    assert_eq!(
        common::assertions::protected_assertions(&actual),
        common::assertions::protected_assertions(&comma)
    );
    actual
}

fn checked(runtime: &mut Runtime, source: &str, count: usize, expected: Option<&str>) -> Value {
    assert_eq!(
        common::assertions::protected_assertions(&parse(source).unwrap()).len(),
        count,
        "{source}"
    );
    common::check_with_count(
        runtime,
        &common::Case {
            title: "explicit-property separator regression".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"))
}

fn check_pair(actual: &str, comma: &str, count: usize, expected: Option<&str>) -> [Runtime; 2] {
    equivalent(actual, comma);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, source) in runtimes.iter_mut().zip([actual, comma]) {
        checked(runtime, source, count, expected);
    }
    runtimes
}

#[test]
fn exact_eight_v3_pairs_remove_only_twenty_four_field_commas_and_run_forty_assertions() {
    let fixtures: Vec<Counterpart> = serde_json::from_str(ACCEPTED).unwrap();
    assert_eq!(fixtures.len(), 8);
    let documents = documents();
    let mut totals = (0, 0, 0, 0);
    for (index, fixture) in fixtures.iter().enumerate() {
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == fixture.document)
            .unwrap()
            .1;
        let matching = cases
            .iter()
            .enumerate()
            .filter(|(_, case)| case.title == fixture.title)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1);
        let (position, case) = matching[0];
        assert_eq!(position, [393, 394, 395, 396, 397, 79, 30, 85][index]);
        assert_eq!(case.expected, fixture.expected);
        let (rewritten, sites) = counterpart(&fixture.source);
        assert_eq!(sites, [2, 4, 5, 4, 3, 2, 2, 2][index]);
        assert_eq!(case.source.trim(), rewritten.trim());
        assert_eq!(fixture.source.len() - rewritten.len(), sites);
        let count = [5, 5, 6, 7, 5, 4, 3, 5][index];
        check_pair(
            case.source.trim(),
            fixture.source.trim(),
            count,
            fixture.expected.as_deref(),
        );
        assert_eq!(
            parse_program(case.source.trim()).unwrap().positions,
            parse_program(fixture.source.trim()).unwrap().positions
        );
        totals.0 += 1;
        totals.1 += sites;
        totals.2 += count;
        totals.3 += usize::from(fixture.expected.is_some());
    }
    assert_eq!(totals, (8, 24, 40, 2));
}

#[test]
fn all_587_historical_programs_keep_original_byte_seals_with_only_authorized_conditional_colons() {
    let mut restored = 0;
    for ((name, cases), (count, expected)) in documents().into_iter().zip([
        (393, 0xf5a53a46a6f7ec80_u64),
        (79, 0x7926adbcd843199c),
        (30, 0x27fc1f8f174166e3),
        (85, 0xc505be4d30545616),
    ]) {
        assert!(cases.len() > count);
        let mut seal = 0xcbf29ce484222325_u64;
        for case in &cases[..count] {
            // NUC49 changes only these exact eight older programs. Restore
            // only their qualified header colons for the ORIGINAL byte seal,
            // checking the exact edit counts, inverse, full AST and positions.
            // Every other historical byte, title, order and return stays sealed.
            let original = conditional_headers::historical_source(name, &case.title, &case.source);
            restored += usize::from(original != case.source);
            for field in [
                &case.title,
                &original,
                case.expected.as_deref().unwrap_or(""),
            ] {
                for byte in field.bytes().chain([0xff]) {
                    seal = (seal ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                }
            }
        }
        assert_eq!(seal, expected, "{name}: accepted historical program bytes");
    }
    assert_eq!(restored, 8);
}

#[test]
fn existing_keys_keep_the_full_ordered_ast_and_quoted_comma_canonical_form() {
    let source = r#"{alpha: 1 "quoted": 2 1e2: 3 .5: 4 alpha: 5}"#;
    let expression = parse_expression(source).unwrap();
    assert_eq!(
        expression,
        Expr::ObjectLiteral(vec![
            ("alpha".to_string(), Expr::Number(1.0)),
            ("quoted".to_string(), Expr::Number(2.0)),
            ("100".to_string(), Expr::Number(3.0)),
            ("0.5".to_string(), Expr::Number(4.0)),
            ("alpha".to_string(), Expr::Number(5.0)),
        ])
    );
    let canonical = r#"{"alpha":1,"quoted":2,"100":3,"0.5":4,"alpha":5}"#;
    assert_eq!(expression.to_string(), canonical);
    assert_eq!(parse_expression(canonical).unwrap(), expression);
    let comma = r#"{alpha: 1, "quoted": 2, 1e2: 3, .5: 4, alpha: 5}"#;
    assert_eq!(parse_expression(comma).unwrap(), expression);
    assert_eq!(
        tokenize(source)
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        tokenize(comma)
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .filter(|token| *token != Token::Comma)
            .collect::<Vec<_>>()
    );
}

#[test]
fn mixed_commas_trailing_commas_and_only_inherited_layout_are_optional() {
    for (actual, comma) in [
        (
            "record = {a: 1 b: 2, c: 3 d: 4,}",
            "record = {a: 1, b: 2, c: 3, d: 4,}",
        ),
        (
            "record = {;a: 1;\nb: 2;;, c: 3,\n;}",
            "record = {;a: 1;\n,b: 2;;, c: 3,\n;}",
        ),
        (
            "record = ({\n    a: 1\n    \"b\": 2;\n    3: 4,\n})",
            "record = ({\n    a: 1,\n    \"b\": 2;,\n    3: 4,\n})",
        ),
        (
            "record = {a: 1 # next field\nb: 2}",
            "record = {a: 1 # next field\n,b: 2}",
        ),
        (
            "record = {a: 1\t'b': 2 \"c\": 3}",
            "record = {a: 1,\t'b': 2, \"c\": 3}",
        ),
    ] {
        for newline in ["\n", "\r\n"] {
            equivalent(
                &actual.replace('\n', newline),
                &comma.replace('\n', newline),
            );
        }
    }
}

#[test]
fn shorthand_qualification_and_commas_around_shorthand_are_not_relaxed() {
    for (actual, comma) in [
        (
            "record = {amount, reading: 2 unit: 3, label, done: true extra: false}",
            "record = {amount, reading: 2, unit: 3, label, done: true, extra: false}",
        ),
        (
            "record = {reading: 1 unit: 2, tail}",
            "record = {reading: 1, unit: 2, tail}",
        ),
        (
            "record = {amount;\n, reading: 2 unit: 3, label;\n}",
            "record = {amount;\n, reading: 2, unit: 3, label;\n}",
        ),
    ] {
        equivalent(actual, comma);
    }
    for source in [
        "record = {amount unit: 2}",
        "record = {reading: 1 unit}",
        "record = {reading: 1 unit, done: true}",
        "record = {reading: 1 unit; done: true}",
        "record = {reading: 1 unit\n}",
        "record = {amount\n: 1}",
    ] {
        assert!(Runtime::check(source).is_err(), "{source}");
    }
}

#[test]
fn the_whole_original_expression_wins_before_any_next_explicit_field() {
    for value in [
        "source + offset * 2",
        "source = other = 3",
        "source && other || backup",
        "items[1:3].length",
        "factory(1, 2)(3).amount",
        "n => n + 1",
        "(left, right) => left + right",
        "function(n) { return {reading: n, unit: \"C\"} }",
        "(n => ({reading: n, unit: \"C\"}))(2).reading",
        "{nested: {reading: source, unit: \"C\"}, tag: \"ok\"}",
        "new Box(2).amount",
        "delete target.item",
        "/[:{}]/.test(\"x\")",
        "`tag:${source}`",
    ] {
        let actual = format!("record = {{first: {value} second: 2}}");
        let comma = format!("record = {{first: {value}, second: 2}}");
        let statements = equivalent(&actual, &comma);
        let [
            Stmt::Assign {
                value: Expr::ObjectLiteral(entries),
                ..
            },
        ] = statements.as_slice()
        else {
            panic!("expected the entire record assignment: {actual}");
        };
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].1, parse_expression(value).unwrap());
        assert_eq!(entries[1], ("second".to_string(), Expr::Number(2.0)));
    }
    check_pair(
        "source = 2\ndef twice(n) => n * 2\nrecord = {reading: twice(source) + 1 done: true}\nassert(record.reading, 5)\nsource = 4\nassert(record.reading, 9)",
        "source = 2\ndef twice(n) => n * 2\nrecord = {reading: twice(source) + 1, done: true}\nassert(record.reading, 5)\nsource = 4\nassert(record.reading, 9)",
        2,
        None,
    );
}

#[test]
fn statement_blocks_and_named_or_anonymous_bodies_keep_existing_disambiguation() {
    for (actual, comma) in [
        (
            "def pack(n) => {reading: n unit: \"C\"}",
            "def pack(n) => {reading: n, unit: \"C\"}",
        ),
        (
            "callback = n => ({reading: n unit: \"C\"})",
            "callback = n => ({reading: n, unit: \"C\"})",
        ),
        (
            "callback = n => { return {reading: n unit: \"C\"} }",
            "callback = n => { return {reading: n, unit: \"C\"} }",
        ),
        (
            "if ({active: true ready: true}).active:\n    pass",
            "if ({active: true, ready: true}).active:\n    pass",
        ),
    ] {
        equivalent(actual, comma);
    }
    let Expr::Function(function) = parse_expression("n => { n }").unwrap() else {
        panic!("expected an anonymous function");
    };
    assert_eq!(
        function.body,
        FunctionBody::Block(vec![Stmt::Expression(Expr::Identifier("n".to_string()))])
    );
    assert_eq!(
        parse("{ n }").unwrap(),
        [Stmt::Block(vec![Stmt::Expression(Expr::Identifier(
            "n".to_string()
        ))])]
    );
}

#[test]
fn failed_guards_keep_exact_observed_errors_tokens_positions_and_no_partial_execution() {
    // Observed with this task's untouched accepted native executable.
    for (source, message, line, column) in [
        (
            "record = { amount other }",
            "Expected ':' but found identifier 'other'",
            1,
            19,
        ),
        (
            "record = { amount\n: amount }",
            "Expected ':' but found end of line",
            1,
            12,
        ),
        (
            "record = { amount:\namount }",
            "Unexpected end of line",
            2,
            1,
        ),
        (
            "record = {\n    amount: amount,\n}",
            "Expected a key but found an indented block",
            2,
            5,
        ),
        ("record = { amount: }", "Unexpected '}'", 1, 21),
        (
            "record = { amount: 1,, next: 2 }",
            "Expected a key but found ','",
            1,
            24,
        ),
        (
            "record = { amount: 1 next }",
            "Expected '}' but found identifier 'next'",
            1,
            22,
        ),
        (
            "record = { amount: 1 next\n: 2 }",
            "Expected '}' but found identifier 'next'",
            1,
            22,
        ),
        (
            "record = { amount: 1 true: 2 }",
            "Expected '}' but found keyword 'true'",
            1,
            22,
        ),
        (
            "record = { amount: 1 `next`: 2 }",
            "Expected '}' but found a template string",
            1,
            22,
        ),
        (
            "record = { amount: 1 [next]: 2 }",
            "Expected '}' but found ':'",
            1,
            28,
        ),
        (
            "record = { amount: 1 next.key: 2 }",
            "Expected '}' but found identifier 'next'",
            1,
            22,
        ),
        (
            "record = { amount: 1 $Next: 2 }",
            "Expected '}' but found '$Next'",
            1,
            22,
        ),
        (
            "record = { amount: 1 -2: 2 }",
            "Expected '}' but found ':'",
            1,
            24,
        ),
        ("record = { amount: 1, next:: 2 }", "Unexpected ':'", 1, 30),
        ("items = [1 2]", "Expected ']' but found number 2", 1, 12),
        ("take(1 2)", "Expected ')' but found number 2", 1, 8),
        (
            "def f(left right) => left",
            "Expected ')' but found identifier 'right'",
            1,
            12,
        ),
        ("{ amount: 1 next: 2 }", "Unexpected ':'", 1, 11),
        (
            "arrow = amount => { amount: 1 next: 2 }",
            "Unexpected ':'",
            1,
            29,
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(
            error.position(),
            Some(Position::new(line, column)),
            "{source}"
        );
        let mut runtime = Runtime::new();
        runtime.run("saved = 7").unwrap();
        let error = runtime.run(&format!("saved = 3\n{source}")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(Position::new(line + 1, column)));
        assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
    }
    let error = Runtime::check("record = { amount: 1 next:: 2 }").unwrap_err();
    assert_eq!(error.message(), "Unexpected ':'");
    assert_eq!(error.position(), Some(Position::new(1, 29)));
}

#[test]
fn every_duplicate_value_runs_in_order_and_aliases_keep_their_identity() {
    let setup = r#"
amount = 3
events = {trace: ""}
payload = {count: 1}
alias = payload
def mark(label, reading):
    events.trace = events.trace + label
    return reading
"#;
    let checks = r#"
assert(events.trace, "ABCDE")
assert(record.first, 4)
assert(record["1"], 3)
assert(record.amount, 3)
record.nested.count = 9
assert(alias.count, 9)
assert(payload.count, 9)
"#;
    let actual = format!(
        "{setup}\nrecord = {{first: mark(\"A\", 1) 1: mark(\"B\", 2) \"1\": mark(\"C\", 3) first: mark(\"D\", 4) nested: payload, amount, last: mark(\"E\", 5)}}\n{checks}"
    );
    let comma = format!(
        "{setup}\nrecord = {{first: mark(\"A\", 1), 1: mark(\"B\", 2), \"1\": mark(\"C\", 3), first: mark(\"D\", 4), nested: payload, amount, last: mark(\"E\", 5)}}\n{checks}"
    );
    for runtime in check_pair(&actual, &comma, 6, None) {
        let Value::Object(id) = runtime.state.variable("record").unwrap() else {
            panic!("expected the existing object value");
        };
        assert_eq!(
            runtime
                .state
                .object(id)
                .unwrap()
                .properties
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["first", "1", "nested", "amount", "last"]
        );
    }
    for source in [
        "record = {entry: missing entry: 7}",
        "record = {entry: missing, entry: 7}",
    ] {
        let mut runtime = Runtime::new();
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert!(!runtime.graph.contains(&NodeKey::variable("record")));
    }
}

#[test]
fn actual_dependencies_snapshots_and_rollback_keep_the_existing_edges() {
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (actual, comma) in [
        (
            "amount = 2\nalternate = 7\nenabled = true\nlive = {reading: amount unit: \"C\"}\nfrozen = {reading: amount.value unit: \"C\"}\nchosen = {reading: enabled and amount or alternate unit: \"C\"}\nderived = live.reading * 2",
            "amount = 2\nalternate = 7\nenabled = true\nlive = {reading: amount, unit: \"C\"}\nfrozen = {reading: amount.value, unit: \"C\"}\nchosen = {reading: enabled and amount or alternate, unit: \"C\"}\nderived = live.reading * 2",
        ),
        ("amount = 5\nenabled = false", "amount = 5\nenabled = false"),
        (
            "try:\n    amount = 8\n    live = {reading: alternate unit: \"K\"}\n    missing\ncatch error:\n    assert(error, ReferenceError(\"missing is not defined\"))",
            "try:\n    amount = 8\n    live = {reading: alternate, unit: \"K\"}\n    missing\ncatch error:\n    assert(error, ReferenceError(\"missing is not defined\"))",
        ),
        ("alternate = 9\namount = 6", "alternate = 9\namount = 6"),
    ] {
        equivalent(actual, comma);
        for (runtime, source) in runtimes.iter_mut().zip([actual, comma]) {
            let count = usize::from(source.contains("assert("));
            checked(runtime, source, count, None);
        }
        for name in ["live", "frozen", "chosen"] {
            let key = NodeKey::variable(name);
            assert_eq!(
                runtimes[0].graph.retrieve(&key).unwrap().dependencies,
                runtimes[1].graph.retrieve(&key).unwrap().dependencies
            );
        }
        // Anonymous IDs are runtime-local. Check the complete derived edge
        // list against the actual live object's identity in each runtime.
        for runtime in &runtimes {
            let Value::Object(id) = runtime.state.variable("live").unwrap() else {
                panic!("expected the live record");
            };
            let expected = [NodeKey::variable("live"), NodeKey::property(id, "reading")];
            assert!(
                runtime
                    .graph
                    .retrieve(&NodeKey::variable("derived"))
                    .unwrap()
                    .dependencies
                    .iter()
                    .eq(expected.iter())
            );
        }
    }
    for runtime in &mut runtimes {
        checked(
            runtime,
            "assert(amount, 6)\nassert(live.reading, 6)\nassert(live.unit, \"C\")\nassert(frozen.reading, 2)\nassert(chosen.reading, 9)\nassert(derived, 12)",
            6,
            None,
        );
        let live = runtime.graph.retrieve(&NodeKey::variable("live")).unwrap();
        assert!(live.dependencies.contains(&NodeKey::variable("amount")));
        assert!(!live.dependencies.contains(&NodeKey::variable("alternate")));
        let frozen = runtime
            .graph
            .retrieve(&NodeKey::variable("frozen"))
            .unwrap();
        assert!(!frozen.dependencies.contains(&NodeKey::variable("amount")));
        let chosen = runtime
            .graph
            .retrieve(&NodeKey::variable("chosen"))
            .unwrap();
        assert!(chosen.dependencies.contains(&NodeKey::variable("enabled")));
        assert!(
            chosen
                .dependencies
                .contains(&NodeKey::variable("alternate"))
        );
        assert!(!chosen.dependencies.contains(&NodeKey::variable("amount")));
        assert_eq!(runtime.assertions_run(), 7);
    }
}

#[test]
fn parameters_receiver_scopes_frozen_functions_and_noncapture_are_unchanged() {
    let setup = "amount = 100\npayload = {count: 1}\nalias = payload\n";
    let checks = r#"
local = pack(7, payload)
frozen = pack.value
box = Box(8)
received = box.pack(9)
assert(local.reading, 7)
assert(local.initial, 7)
assert(received.reading, 9)
assert(received.receiver.id, "box")
assert(received.receiver.amount, 8)
received.payload.count = 3
assert(alias.count, 3)
assert(local.payload.count, 3)
amount = 200
assert(local.reading, 7)
assert(received.reading, 9)
"#;
    let actual = format!(
        "{setup}\ndef pack(amount, payload) => {{reading: amount initial: amount.value payload: payload}}\nclass Box:\n    def init(amount):\n        this.amount = amount\n    def pack(amount):\n        return {{reading: amount receiver: self payload: alias}}\n{checks}"
    );
    let comma = format!(
        "{setup}\ndef pack(amount, payload) => {{reading: amount, initial: amount.value, payload: payload}}\nclass Box:\n    def init(amount):\n        this.amount = amount\n    def pack(amount):\n        return {{reading: amount, receiver: self, payload: alias}}\n{checks}"
    );
    let mut runtimes = check_pair(&actual, &comma, 9, None);
    for runtime in &mut runtimes {
        checked(
            runtime,
            "def pack(amount, payload) => {reading: amount * 2, initial: amount.value, payload: payload}\nassert(pack(3, payload).reading, 6)\nassert(frozen(3, payload).reading, 3)",
            2,
            None,
        );
    }
    for fields in [
        "{reading: amount next: candidate}",
        "{reading: amount, next: candidate}",
    ] {
        let source = format!(
            "def factory(amount):\n    return candidate => ({fields})\ncallback = factory(3)\ncallback(4)"
        );
        let error = Runtime::new().run(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "amount is not defined");
        assert_eq!(error.position(), Some(Position::new(4, 1)));
    }
}

#[test]
fn computed_string_writes_error_members_and_queued_receiver_failures_stay_inherited() {
    for fields in [
        r#"{1: "first" "1": "latest" state: "open"}"#,
        r#"{1: "first", "1": "latest", state: "open"}"#,
    ] {
        let mut runtime = Runtime::new();
        runtime.run(&format!("record = {fields}")).unwrap();
        let error = runtime.run("record[\"1\"] = \"manual\"").unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), "Cannot assign to record[\"1\"]");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        checked(
            &mut runtime,
            "assert(record[\"1\"], \"latest\")\ntry:\n    missing\ncatch error:\n    assert(error, ReferenceError(\"missing is not defined\"))\n    assert(error.message, null)",
            3,
            None,
        );
        let error = runtime.run("record.value = 3").unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), "Cannot use 'value' as a property");
    }
    for fields in [
        "{reading: live unit: \"C\"}",
        "{reading: live, unit: \"C\"}",
    ] {
        let source = format!(
            r#"class Router:
    def init(prefix):
        this.prefix = prefix
    def choose(status, fallback):
        if status == "blocked":
            return this.prefix + "hold"
        else if status == "ready":
            return this.prefix + "send"
        else if fallback:
            return this.prefix + "later"
        else:
            return this.prefix + "skip"
router = Router("a:")
status = "ready"
live = router.choose(status, false)
snapshot = live.value
assert(live, "a:send")
assert(snapshot, "a:send")
record = {fields}
router.prefix = "b:"
"#
        );
        let mut runtime = Runtime::new();
        runtime.run("kept = 7").unwrap();
        let error = runtime.run(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "this is not defined");
        assert_eq!(error.position(), Some(Position::new(20, 1)));
        assert_eq!(runtime.assertions_run(), 2);
        assert!(runtime.take_assertions().is_empty());
        assert!(runtime.state.class("Router").is_none());
        assert_eq!(runtime.run("kept").unwrap(), Value::Number(7.0));
        assert_eq!(runtime.graph.len(), 1);
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    id: String,
    description: String,
    code: String,
    #[serde(rename = "return")]
    expected: Option<String>,
}

fn check_export(source: &str, count: usize, expected: Option<&str>) {
    let parts = build_statements::statements(source);
    let split = parts
        .iter()
        .flat_map(|part| parse(part).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(parse(source).unwrap(), split, "{source}\n{parts:#?}");
    assert_eq!(generate_all(&parse(source).unwrap()), generate_all(&split));
    let mut runtime = Runtime::new();
    let mut last = Value::Null;
    for part in &parts {
        last = runtime
            .run(part)
            .unwrap_or_else(|error| panic!("{part}\n{error}"));
    }
    assert_eq!(runtime.assertions_run(), count);
    assert!(runtime.take_assertions().is_empty());
    if let Some(expected) = expected {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&runtime.serialize_json(&last).unwrap())
                .unwrap(),
            serde_json::from_str::<serde_json::Value>(expected).unwrap()
        );
    }
}

#[test]
fn whole_programs_and_actual_statement_exports_run_all_eight_pairs() {
    let fixtures: Vec<Counterpart> = serde_json::from_str(ACCEPTED).unwrap();
    for (fixture, count) in fixtures.iter().zip([5, 5, 6, 7, 5, 4, 3, 5]) {
        let (actual, _) = counterpart(&fixture.source);
        for source in [&actual, &fixture.source] {
            check_export(source, count, fixture.expected.as_deref());
        }
    }
}

#[test]
fn generated_spec_records_retain_full_asts_assertions_returns_and_exported_execution() {
    let records = include_str!("../dataset/nucleoid.spec.jsonl")
        .lines()
        .map(|line| serde_json::from_str::<Record>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 445);
    let fixtures: Vec<Counterpart> = serde_json::from_str(ACCEPTED).unwrap();
    for ((fixture, record), count) in fixtures[..5]
        .iter()
        .zip(&records[393..])
        .zip([5, 5, 6, 7, 5])
    {
        assert_eq!(record.description, fixture.title);
        assert_eq!(record.expected, fixture.expected);
        equivalent(&record.code, &fixture.source);
        check_export(&record.code, count, record.expected.as_deref());
    }
}

#[test]
fn column_one_multiline_exporter_splitting_is_not_silently_repaired() {
    let actual = "record = ({\nfirst: 1\nsecond: 2\n})";
    let comma = "record = ({\nfirst: 1,\nsecond: 2\n})";
    equivalent(actual, comma);
    let mut errors = Vec::new();
    for source in [actual, comma] {
        let parts = build_statements::statements(source);
        assert!(parts.len() > 1);
        errors.push(Runtime::check(&parts[0]).unwrap_err());
    }
    assert_eq!(errors[0], errors[1]);
    assert_eq!(errors[0].message(), "Expected a key but found end of input");
}

#[test]
fn new_canonical_training_shard_is_self_contained_and_development_executed() {
    let records = include_str!("../dataset/nucleoid.synth.25.jsonl")
        .lines()
        .map(|line| serde_json::from_str::<Record>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 4);
    for ((index, record), count) in records.iter().enumerate().zip([5, 5, 4, 5]) {
        assert_eq!(record.id, format!("synth-25-{:04}", index + 1));
        assert!(!record.description.is_empty());
        checked(
            &mut Runtime::new(),
            &record.code,
            count,
            record.expected.as_deref(),
        );
        check_export(&record.code, count, record.expected.as_deref());
    }
}

#[test]
fn flat_fields_make_progress_without_bypassing_nested_operator_or_postfix_limits() {
    let fields = (0..4096)
        .map(|index| format!("key{index}: {index}"))
        .collect::<Vec<_>>();
    let actual = format!("record = {{{}}}", fields.join(" "));
    let comma = format!("record = {{{}}}", fields.join(", "));
    let ast = equivalent(&actual, &comma);
    let Stmt::Assign {
        value: Expr::ObjectLiteral(entries),
        ..
    } = &ast[0]
    else {
        panic!("expected a flat record");
    };
    assert_eq!(entries.len(), 4096);
    let layout = ";\n".repeat(2048);
    equivalent(
        &format!("record = {{first: 1{layout}second: 2}}"),
        &format!("record = {{first: 1{layout},second: 2}}"),
    );
    let valid = "record = {first: 1 second: {inner: 2 ready: true} third: 3}";
    for length in 0..valid.len() {
        let _ = Runtime::check(&valid[..length]);
    }
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            for (value, message) in [
                (
                    format!("{}0{}", "{next: ".repeat(200), "}".repeat(200)),
                    "Expressions are nested too deeply",
                ),
                (
                    format!("source{}", " + 1".repeat(4096)),
                    "Expression has too many operands",
                ),
                (
                    format!("source{}", ".next".repeat(4096)),
                    "Expression has too many parts",
                ),
            ] {
                let errors = [
                    Runtime::check(&format!("record = {{first: {value} second: 2}}")).unwrap_err(),
                    Runtime::check(&format!("record = {{first: {value}, second: 2}}")).unwrap_err(),
                ];
                assert_eq!(errors[0], errors[1]);
                assert_eq!(errors[0].kind(), ErrorKind::Syntax);
                assert_eq!(errors[0].message(), message);
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

fn stdin(source: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(source).unwrap();
    child.wait_with_output().unwrap()
}

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

#[test]
fn native_batch_and_stdin_preserve_utf8_nul_numbers_framing_and_existing_buffering() {
    let directory = std::env::temp_dir().join(format!(
        "nucleoid-explicit-properties-{}",
        std::process::id()
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("program.nuc");
    for newline in ["\n", "\r\n"] {
        for ending in ["", "\n", "\n\n"] {
            for text in [r#""café😀\0""#, "\"café😀\0\""] {
                for separator in [" ", ", "] {
                    let source = format!(
                        "record = {{text: {text}{separator}reading: -1.25}}\nrecord.text{ending}"
                    )
                    .replace('\n', newline);
                    fs::write(&path, &source).unwrap();
                    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
                        .arg("--json")
                        .arg(&path)
                        .output()
                        .unwrap();
                    assert!(output.status.success(), "{output:?}");
                    assert_eq!(output.stdout, "\"café😀\\u0000\"\n".as_bytes());
                    assert_eq!(output.stderr, b"");
                    let source = format!("if true:\n    record = {{text: {text}{separator}reading: -1.25}}\nrecord.text{ending}").replace('\n', newline);
                    let output = stdin(source.as_bytes());
                    assert!(output.status.success(), "{output:?}");
                    assert_eq!(
                        output.stdout,
                        format!("{BANNER}\"café😀\\u0000\"\n").as_bytes()
                    );
                    assert_eq!(output.stderr, b"");
                }
            }
        }
    }
    for separator in [" ", ", "] {
        let output =
            stdin(format!("({{reading: -1.25{separator}unit: \"C\"}}).reading\n").as_bytes());
        assert!(output.status.success());
        assert_eq!(output.stdout, format!("{BANNER}-1.25\n").as_bytes());
        assert_eq!(output.stderr, b"");
    }
    fs::remove_file(path).unwrap();
    fs::remove_dir(directory).unwrap();
}

#[test]
fn native_file_eof_and_inherited_stdin_newline_errors_remain_distinct() {
    let directory = std::env::temp_dir().join(format!(
        "nucleoid-explicit-property-errors-{}",
        std::process::id()
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("program.nuc");
    for separator in [" ", ", "] {
        let source = format!("record = {{reading: 1{separator}unit:");
        fs::write(&path, &source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg("--json")
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"");
        let message = String::from_utf8(output.stderr).unwrap();
        assert!(
            message.starts_with("SyntaxError: Unexpected end of input (line 1, column "),
            "{message}"
        );
        let output = stdin(source.as_bytes());
        assert!(output.status.success());
        assert_eq!(output.stdout, BANNER.as_bytes());
        assert_eq!(
            output.stderr,
            b"SyntaxError: Unexpected end of line (line 2, column 1)\n"
        );
    }
    fs::remove_file(path).unwrap();
    fs::remove_dir(directory).unwrap();
}
