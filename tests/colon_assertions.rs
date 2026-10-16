//! Colon-led checks lower to the existing assertion helper without a runtime API.

mod common;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_program};
use nucleoid::{ErrorKind, Position, Runtime, Value};

/// Test-fixture transformation only: each assertion occupies a complete line.
fn compact(control: &str) -> String {
    control
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            match trimmed
                .strip_prefix("assert(")
                .and_then(|arguments| arguments.strip_suffix(')'))
            {
                Some(arguments) => {
                    let indent = &line[..line.len() - trimmed.len()];
                    format!("{indent}assert: {arguments}")
                }
                None => line.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn equivalent(candidate: &str, control: &str) {
    let candidate_ast = parse(candidate).unwrap_or_else(|error| panic!("{candidate}\n{error}"));
    let control_ast = parse(control).unwrap_or_else(|error| panic!("{control}\n{error}"));
    assert_eq!(candidate_ast, control_ast, "{candidate}");
    assert_eq!(generate_all(&candidate_ast), generate_all(&control_ast));
    assert_eq!(
        common::assertions::protected_assertions(&candidate_ast),
        common::assertions::protected_assertions(&control_ast)
    );
}

fn check_pair(control: &str, assertions: usize, expected: Option<&str>) -> [Runtime; 2] {
    check_pair_runs(control, assertions, assertions, expected)
}

fn check_pair_runs(
    control: &str,
    protected: usize,
    executed: usize,
    expected: Option<&str>,
) -> [Runtime; 2] {
    let candidate = compact(control);
    assert_ne!(candidate, control, "fixture must exercise the new spelling");
    equivalent(&candidate, control);
    assert_eq!(
        parse_program(&candidate).unwrap().positions,
        parse_program(control).unwrap().positions,
        "line-leading statements retain their positions"
    );
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let mut results = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([candidate.as_str(), control]) {
        assert_eq!(
            common::assertions::protected_assertions(&parse(source).unwrap()).len(),
            protected,
            "{source}"
        );
        let case = common::Case {
            title: "paired colon assertion regression".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        };
        let result = common::check_with_count(runtime, &case, protected).unwrap();
        assert_eq!(runtime.assertions_run(), executed, "{source}");
        results.push(runtime.serialize_json(&result).unwrap());
    }
    assert_eq!(results[0], results[1]);
    runtimes
}

#[test]
fn exact_call_ast_and_canonical_generator_are_unchanged() {
    let statement = parse("assert: 1 + 2, 3").unwrap().remove(0);
    assert_eq!(statement, parse("assert(1 + 2, 3)").unwrap().remove(0));
    let Stmt::Expression(Expr::Call { callee, arguments }) = statement else {
        panic!("must lower to the existing expression statement");
    };
    assert_eq!(*callee, Expr::Identifier("assert".to_string()));
    assert_eq!(arguments.len(), 2);
    assert_eq!(generate_all(&parse("assert: 1, 1").unwrap()), "assert(1,1)");
    for (candidate, control) in [
        ("assert : 1, 1", "assert(1, 1)"),
        ("assert\t:\t1, 1", "assert(1, 1)"),
        ("assert: 1, 1; assert(2, 2)", "assert(1, 1); assert(2, 2)"),
        ("if true: assert: 1, 1", "if true: assert(1, 1)"),
        ("{ assert: 1, 1 }", "{ assert(1, 1) }"),
    ] {
        equivalent(candidate, control);
    }
}

#[test]
fn literals_lists_objects_templates_and_full_expressions_keep_their_values() {
    check_pair(
        r#"amount = 3
label = "sensor"
record = {amount: amount, label: label}
assert(null, null)
assert(true, true)
assert("a,b", "a,b")
assert('a,b', 'a,b')
assert(`sensor,${amount}`, "sensor,3")
assert([record.amount, label], [3, "sensor"])
assert({kind: label, valid: true}, {kind: "sensor", valid: true})
assert((amount = 4), 4)
assert(record.amount, 3)
record.amount"#,
        9,
        // An assignment expression in an argument retains the accepted
        // runtime's non-propagating behavior; statement assignments above
        // and in the reasoning regressions exercise live propagation.
        Some("3"),
    );
}

#[test]
fn reasoning_reads_do_not_install_assertion_nodes_or_shape_observers() {
    let mut runtimes = check_pair(
        "source = 2\ntotal = source + 1\n\
         assert(total, 3)\nassert((affects source).length, 1)\n\
         assert((why total)[1].node, \"source\")\n\
         source = 6\nassert(total, 7)\nassert((why total)[1].holds, 6)\ntotal",
        5,
        Some("7"),
    );
    for runtime in &mut runtimes {
        assert_eq!(runtime.graph.len(), 2);
        assert!(
            !runtime
                .graph
                .keys()
                .any(|key| key.as_str().contains("assert"))
        );
        for node in runtime.graph.nodes() {
            assert_eq!(runtime.graph.shape_dependencies(&node.key).count(), 0);
        }
        let count = runtime.assertions_run();
        runtime.run("source = 10").unwrap();
        assert_eq!(runtime.run("total").unwrap(), Value::Number(11.0));
        assert_eq!(
            runtime.assertions_run(),
            count,
            "checks must not react again"
        );
    }
    assert!(runtimes[0].graph.keys().eq(runtimes[1].graph.keys()));
    let key = NodeKey::variable("total");
    assert_eq!(
        runtimes[0].graph.retrieve(&key).unwrap().dependencies,
        runtimes[1].graph.retrieve(&key).unwrap().dependencies
    );
}

#[test]
fn scopes_functions_receivers_returns_and_snapshots_are_shared() {
    check_pair_runs(
        r#"def check_name(name):
    saved = name.value
    assert(saved, "draft")
    return {name: name, initial: saved}
packet = check_name("draft")
assert(packet.name, "draft")
assert(packet.initial, "draft")
class Meter:
    def init(amount):
        this.amount = amount
    def check(expected):
        assert(self.amount, expected)
        return this.amount
meter = Meter(4)
saved = meter.amount.value
assert(meter.check(4), 4)
meter.amount = 7
assert(meter.amount, 7)
assert(saved, "draft")
saved"#,
        7,
        8,
        // Reusing this name after the function's frozen local retains the
        // inherited "draft" result in both forms; do not repair the runtime.
        Some("\"draft\""),
    );
    check_pair(
        "class Meter:\n    def init(amount):\n        this.amount = amount\n\
         meter = Meter(4)\nstartup = meter.amount.value\n\
         assert(meter.amount, 4)\nmeter.amount = 7\n\
         assert(meter.amount, 7)\nassert(startup, 4)\nstartup",
        3,
        Some("4"),
    );
}

#[test]
fn blocks_braced_arrows_loops_and_conditionals_execute_all_checks() {
    check_pair(
        r#"callback = value => {
    assert(value, 3)
    return value
}
callback(3)
{
    local = 3
    assert(local, 3)
}
class Item
item = Item()
item.reading = 3
for entry in [item]:
    assert(entry.reading, 3)
active = true
if active:
    assert(active, true)"#,
        4,
        None,
    );
}

#[test]
fn inherited_nested_control_limitations_still_fail_complete_coverage_in_both_forms() {
    let control = r#"def identity(value):
    if value > 0:
        assert(value, 3)
        return value
    return 0
callback = value => {
    assert(value, 3)
    return value
}
{
    local = identity(3)
    assert(local, 3)
}
for item in [3]:
    assert(callback(item), 3)
assert(identity(0), 0)"#;
    let candidate = compact(control);
    equivalent(&candidate, control);
    for source in [candidate.as_str(), control] {
        assert_eq!(
            common::assertions::protected_assertions(&parse(source).unwrap()).len(),
            5
        );
        let mut runtime = Runtime::new();
        assert_eq!(runtime.run(source).unwrap(), Value::Null);
        assert_eq!(runtime.assertions_run(), 3);
        assert!(runtime.take_assertions().is_empty());
        let error = check_source(source).unwrap_err();
        assert!(error.contains("only 3 of 5 assertions ran"), "{error}");
    }
}

#[test]
fn arguments_execute_once_left_to_right() {
    check_pair(
        r#"log = []
def note(label):
    log.push(label)
    return 7
assert(note("actual"), note("expected"))
assert(log, ["actual", "expected"])
assert(log.length, 2)"#,
        3,
        None,
    );
}

#[test]
fn failures_counters_and_result_exclusion_are_the_existing_helper_behavior() {
    for source in ["17\nassert: [1, 2], [1, 3]", "17\nassert([1, 2], [1, 3])"] {
        let mut runtime = Runtime::new();
        assert_eq!(runtime.run(source).unwrap(), Value::Number(17.0));
        assert_eq!(runtime.assertions_run(), 1);
        let failures = runtime.take_assertions();
        assert_eq!(failures.len(), 1);
        assert_eq!(
            failures[0].actual,
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
        );
        assert_eq!(
            failures[0].expected,
            Value::List(vec![Value::Number(1.0), Value::Number(3.0)])
        );
        assert!(runtime.take_assertions().is_empty());
        assert_eq!(runtime.run("assert(1, 1)").unwrap(), Value::Null);
        assert_eq!(runtime.run("assert: 2, 2").unwrap(), Value::Null);
        assert_eq!(runtime.assertions_run(), 3);
        assert!(runtime.graph.is_empty());
    }
}

#[test]
fn operand_errors_stop_later_arguments_and_roll_back_the_statement() {
    for (candidate, control) in [
        (
            "assert: missing, note(\"later\")",
            "assert(missing, note(\"later\"))",
        ),
        (
            "assert: (state = 2), missing",
            "assert((state = 2), missing)",
        ),
        ("assert: 1, missing", "assert(1, missing)"),
    ] {
        equivalent(candidate, control);
        let mut errors = Vec::new();
        for source in [candidate, control] {
            let mut runtime = Runtime::new();
            runtime
                .run("state = 1\nlog = []\ndef note(label):\n    log.push(label)\n    return 1")
                .unwrap();
            let error = runtime.run(source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Reference);
            assert_eq!(error.message(), "missing is not defined");
            assert_eq!(error.position(), Some(Position::new(1, 1)));
            errors.push(error);
            assert_eq!(runtime.assertions_run(), 0);
            assert!(runtime.take_assertions().is_empty());
            assert_eq!(runtime.run("state").unwrap(), Value::Number(1.0));
            assert_eq!(runtime.run("log.length").unwrap(), Value::Number(0.0));
        }
        assert_eq!(errors[0], errors[1]);
    }
}

#[test]
fn caught_errors_rollback_and_recovered_dependencies_are_unchanged() {
    check_pair(
        r#"state = 1
total = state + 1
message = ""
try:
    state = 2
    failed = missing
catch error:
    message = error
    assert(error, ReferenceError("missing is not defined"))
assert(state, 1)
assert(total, 2)
assert(message, ReferenceError("missing is not defined"))
state = 4
assert(total, 5)
total"#,
        5,
        Some("5"),
    );
}

#[test]
fn bindings_parameters_fields_and_object_keys_named_assert_keep_their_meaning() {
    check_pair(
        r#"assert = 12
def identity(assert: int) => assert
def checked(assert):
    assert(assert, 4)
    return assert
record = {assert: "field", reading: 4}
class Box:
    assert: int
box = Box()
box.assert = 9
assert(identity(4), 4)
assert(checked(4), 4)
assert(record.assert, "field")
assert(box.assert, 9)
assert(assert, 12)"#,
        6,
        None,
    );
    equivalent(
        "callback = (assert: int) => assert\nrecord = {assert: 4}",
        "callback = (assert: int) => assert\nrecord = {assert: 4}",
    );
}

#[test]
fn typed_declarations_keep_priority_and_the_inherited_missing_definition_error() {
    for source in ["assert: int", "assert: int\n", "assert: int;"] {
        let program = parse_program(source).unwrap();
        assert_eq!(
            program.statements,
            [Stmt::Declaration {
                name: "assert".to_string(),
                type_name: "int".to_string()
            }]
        );
        assert_eq!(program.positions, [Position::new(1, 1)]);
        let error = Runtime::new().run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "Missing definition");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
    }
    let statements = parse("if true:\n    assert: Custom\nnext = 1").unwrap();
    let Stmt::If { consequent, .. } = &statements[0] else {
        panic!("if statement")
    };
    assert!(
        matches!(&consequent[0], Stmt::Declaration { name, type_name }
        if name == "assert" && type_name == "Custom")
    );
    // RightBrace is deliberately not one of is_type_declaration's boundaries.
    let error = Runtime::check("{ assert: int }").unwrap_err();
    assert_eq!(error.message(), "Expected ',' but found '}'");
}

#[test]
fn direct_assert_helper_is_not_shadowed_by_a_binding_or_parameter() {
    check_pair(
        "assert = 12\ndef check(assert):\n    assert(assert, 4)\n    return assert\n\
         result = check(4)\nassert(result, 4)\nassert(assert, 12)",
        3,
        None,
    );
}

#[test]
fn ordinary_call_argument_counts_and_expression_uses_are_not_restricted() {
    let mut runtime = Runtime::new();
    runtime
        .run("assert()\nassert(1)\nassert(1, 1, 99)\nresult = assert(2, 2)")
        .unwrap();
    assert_eq!(runtime.assertions_run(), 4);
    let failures = runtime.take_assertions();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].actual, Value::Number(1.0));
    assert_eq!(failures[0].expected, Value::Undefined);
    equivalent("assert(1, 1,)", "assert(1, 1)");
    equivalent("assert (1, 1)", "assert(1, 1)");
    for source in [
        "result = assert: 1, 1",
        "return assert: 1, 1",
        "Assert: 1, 1",
        "verify: 1, 1",
        "assert 1, 1",
        "record.assert: 1, 1",
    ] {
        assert!(Runtime::check(source).is_err(), "{source}");
    }
}

#[test]
fn malformed_forms_have_precise_diagnostics_without_new_layout_continuation() {
    for (source, message, position) in [
        ("assert:", "Unexpected end of input", Position::new(1, 8)),
        (
            "assert: 1",
            "Expected ',' but found end of input",
            Position::new(1, 10),
        ),
        (
            "assert: 1 2",
            "Expected ',' but found number 2",
            Position::new(1, 11),
        ),
        (
            "assert: 1,",
            "Unexpected end of input",
            Position::new(1, 11),
        ),
        (
            "assert: 1, 1,",
            "Assertion statements expect exactly two arguments",
            Position::new(1, 13),
        ),
        (
            "assert: 1, 1, 2",
            "Assertion statements expect exactly two arguments",
            Position::new(1, 13),
        ),
        (
            "assert: 1,\n1",
            "Unexpected end of line",
            Position::new(2, 1),
        ),
        (
            "assert:\n    1, 1",
            "Unexpected end of line",
            Position::new(2, 5),
        ),
        (
            "assert:\n# no continuation\n1, 1",
            "Unexpected end of line",
            Position::new(3, 1),
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(position), "{source}");
    }
}

#[test]
fn unchanged_legacy_errors_retain_their_physical_positions() {
    for (source, message, position) in [
        ("assert(1,, 2)", "Unexpected ','", Position::new(1, 12)),
        (
            "assert(1, 2",
            "Expected ')' but found end of input",
            Position::new(1, 12),
        ),
        ("value: int", "Missing definition", Position::new(1, 1)),
    ] {
        let error = Runtime::new().run(source).unwrap_err();
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(position), "{source}");
    }
}

#[test]
fn ordinary_whitespace_uses_existing_tokens_not_a_keyword_or_new_layout() {
    assert_eq!(
        tokenize("assert: 1, 1")
            .unwrap()
            .into_iter()
            .map(|token| token.token)
            .collect::<Vec<_>>(),
        [
            Token::Identifier("assert".to_string()),
            Token::Colon,
            Token::Number(1.0),
            Token::Comma,
            Token::Number(1.0),
            Token::Eof,
        ]
    );
    for newline in ["\n", "\r\n"] {
        let candidate = format!("assert \t: 1, 1{newline}assert(2, 2)");
        equivalent(&candidate, "assert(1, 1)\nassert(2, 2)");
    }
}

#[test]
fn statement_and_argument_nesting_limits_match_calls_at_the_exact_boundary() {
    let mut successes = 0;
    let mut failures = 0;
    for depth in 0..=70 {
        for operand in [
            format!("{}1{}", "(".repeat(depth), ")".repeat(depth)),
            format!("{}1", "+".repeat(depth)),
            format!("{}1{}", "[".repeat(depth), "]".repeat(depth)),
        ] {
            let candidate = format!("assert: {operand}, 1");
            let control = format!("assert({operand}, 1)");
            match (parse(&candidate), parse(&control)) {
                (Ok(candidate), Ok(control)) => {
                    assert_eq!(candidate, control);
                    successes += 1;
                }
                (Err(candidate), Err(control)) => {
                    assert_eq!(candidate.kind(), ErrorKind::Syntax);
                    assert_eq!(candidate.message(), "Expressions are nested too deeply");
                    assert_eq!(candidate.message(), control.message());
                    failures += 1;
                }
                _ => panic!("different nesting budget at depth {depth}: {operand}"),
            }
        }
        let candidate = format!("{}assert: 1, 1{}", "{".repeat(depth), "}".repeat(depth));
        let control = format!("{}assert(1, 1){}", "{".repeat(depth), "}".repeat(depth));
        match (parse(&candidate), parse(&control)) {
            (Ok(candidate), Ok(control)) => assert_eq!(candidate, control),
            (Err(candidate), Err(control)) => assert_eq!(candidate.message(), control.message()),
            _ => panic!("different statement nesting budget at depth {depth}"),
        }
    }
    assert!(successes > 0 && failures > 0);
}

#[test]
fn operator_and_postfix_chains_share_the_original_limits() {
    for count in 120..=132 {
        for operand in [
            format!("1{}", " + 1".repeat(count)),
            format!("record{}", ".field".repeat(count)),
        ] {
            let candidate = parse(&format!("assert: {operand}, 1"));
            let control = parse(&format!("assert({operand}, 1)"));
            match (candidate, control) {
                (Ok(candidate), Ok(control)) => assert_eq!(candidate, control),
                (Err(candidate), Err(control)) => {
                    assert_eq!(candidate.message(), control.message())
                }
                _ => panic!("different chain budget for {operand}"),
            }
        }
    }
}

fn check_source(source: &str) -> Result<(), String> {
    common::check(&common::Case {
        title: "coverage regression".to_string(),
        source: source.to_string(),
        expected: None,
    })
}

#[test]
fn untaken_new_and_spaced_old_assertions_cannot_escape_coverage() {
    for source in [
        "if false:\n    assert: 1, 1",
        "if false:\n    assert (1, 1)",
        "def unused():\n    assert: 1, 1",
        "callback = value => { assert: value, value }",
    ] {
        let error = check_source(source).unwrap_err();
        assert!(
            error.contains("only 0 of 1 assertions ran"),
            "{source}\n{error}"
        );
    }
}

#[test]
fn coverage_ignores_literal_comment_and_key_text_but_visits_nested_function_and_template_asts() {
    for source in [
        "text = \"assert( and assert:\"\n# assert( assert:\nrecord = {assert: 1}",
        "assert: assert(1, 1), null",
        "text = `${assert(1, 1)}`",
        "def used():\n    assert: 1, 1\nused()",
        "callback = value => { assert: value, value }\ncallback(3)",
        "{ if true: assert: 1, 1 }",
    ] {
        check_source(source).unwrap_or_else(|error| panic!("{source}\n{error}"));
    }
    let statements = parse("assert: assert(1, 1), null").unwrap();
    assert_eq!(
        common::assertions::protected_assertions(&statements),
        ["assert(assert(1,1),null)", "assert(1,1)"]
    );
}

#[test]
fn snippet_return_metadata_preserves_source_and_checks_the_actual_value() {
    let document = "**A check.**\n```nuc\n# A check\n7\nassert: 1, 1\n# return: 7\n```\n";
    let cases = common::snippets(document, "fixture");
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].expected.as_deref(), Some("7"));
    assert_eq!(cases[0].source, "# A check\n7\nassert: 1, 1\n");
    common::check(&cases[0]).unwrap();
    let old = "**An old snippet.**\n```nuc\n\n7\n\n```\n";
    assert_eq!(common::snippets(old, "fixture")[0].source, "\n7\n\n");
}

#[test]
fn four_self_contained_training_records_execute_their_actual_checks_and_returns() {
    for line in include_str!("../dataset/nucleoid.synth.26.jsonl").lines() {
        let record: serde_json::Value = serde_json::from_str(line).unwrap();
        let case = common::Case {
            title: record["id"].as_str().unwrap().to_string(),
            source: record["code"].as_str().unwrap().to_string(),
            expected: record["return"].as_str().map(str::to_string),
        };
        common::check(&case).unwrap_or_else(|error| panic!("{}: {error}", case.title));
    }
}
