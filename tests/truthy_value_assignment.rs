mod common;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::{generate, generate_all};
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let result = common::check_with_count(
        runtime,
        &common::Case {
            title: "truthy value assignment regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
    result
}

fn error(runtime: &mut Runtime, source: &str) -> (ErrorKind, String, Option<Position>) {
    let error = runtime.run(source).unwrap_err();
    (error.kind(), error.message(), error.position())
}

fn assignment(target: Expr, value: Expr) -> Stmt {
    Stmt::TruthyValueAssign { target, value }
}

fn dependencies(runtime: &Runtime, condition: &str) -> Vec<String> {
    let key = NodeKey::conditional(&parse_expression(condition).unwrap(), None);
    let node = runtime.graph.retrieve(&key).unwrap();
    assert_eq!(runtime.graph.shape_dependencies(&key).count(), 0);
    node.dependencies.iter().map(ToString::to_string).collect()
}

#[test]
fn the_statement_reuses_and_and_value_assignment_tokens() {
    for source in ["slot&&:=1", "slot && := 1", "slot && /* gap */ := 1"] {
        let tokens: Vec<_> = tokenize(source)
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .filter(|token| !matches!(token, Token::Newline | Token::Eof))
            .collect();
        assert_eq!(
            tokens,
            [
                Token::Identifier("slot".to_string()),
                Token::AndAnd,
                Token::ColonAssign,
                Token::Number(1.0),
            ]
        );
        assert_eq!(
            parse(source).unwrap(),
            [assignment(
                Expr::Identifier("slot".to_string()),
                Expr::Number(1.0)
            )]
        );
    }
}

#[test]
fn grouped_paths_contextual_names_and_property_receivers_match_value_assignment() {
    for target in [
        "slot",
        "(slot)",
        "box.item",
        "(box).item",
        "((box.item))",
        "this.item",
        "self.item",
        "$Order.item",
        "lambda",
        "suppose",
        "assert",
        "repeat",
        "while",
        "until",
        "enum",
        "defer",
    ] {
        let ordinary = parse(&format!("{target} := source")).unwrap();
        let [
            Stmt::ValueAssign {
                target: path,
                value,
            },
        ] = ordinary.as_slice()
        else {
            panic!("{ordinary:?}");
        };
        assert_eq!(
            parse(&format!("{target} &&:= source")).unwrap(),
            [assignment(path.clone(), value.clone())]
        );
    }
    for target in ["box.answer", "this.answer", "self.answer", "$Order.answer"] {
        let ordinary = parse(&format!("{target} := .base")).unwrap();
        let [
            Stmt::ValueAssign {
                target: path,
                value,
            },
        ] = ordinary.as_slice()
        else {
            panic!("{ordinary:?}");
        };
        assert_eq!(
            parse(&format!("{target} &&:= .base")).unwrap(),
            [assignment(path.clone(), value.clone())]
        );
    }
}

#[test]
fn rendering_preserves_the_distinct_action_and_round_trips() {
    for (source, rendered) in [
        ("slot && := source + 1", "slot&&:=source+1"),
        ("box.item &&:= c ? 1 : 2", "box.item&&:=(c?1:2)"),
        ("slot &&:= first && last", "slot&&:=first&&last"),
        ("slot &&:= first ?? last", "slot&&:=(first??last)"),
        ("slot &&:= other = 3", "slot&&:=other=3"),
    ] {
        let statements = parse(source).unwrap();
        assert_eq!(generate(&statements[0]), rendered);
        assert_eq!(parse(rendered).unwrap(), statements);
    }
    let action = parse("slot &&:= source").unwrap();
    assert_eq!(generate_all(&action), "slot&&:=source");
    for source in ["slot := source", "slot ??:= source", "slot &&= source"] {
        assert_ne!(
            NodeKey::block(&action, None),
            NodeKey::block(&parse(source).unwrap(), None)
        );
    }
    let source = "def consume(slot = 1):\n    slot &&:= 5\n    return slot";
    let statements = parse(source).unwrap();
    assert_eq!(parse(&generate(&statements[0])).unwrap(), statements);
}

#[test]
fn only_the_exact_statement_head_pair_gains_a_new_reading() {
    for source in [
        "result = slot &&:= 1",
        "f(slot &&:= 1)",
        "[slot &&:= 1]",
        "(slot &&:= 1)",
        "return slot &&:= 1",
        "if slot &&:= 1: pass",
        "slot &&:= other &&:= 1",
        "slot &&:= (other &&:= 1)",
        "slot &&:= item -> item &&:= 1",
        "slot &&:= lambda item: item &&:= 1",
        "class Slot(value &&:= 1): pass",
        "def consume(value &&:= 1): pass",
        "slot ||:= 1",
        "slot and:= 1",
        "slot &&\n:= 1",
    ] {
        assert!(Runtime::check(source).is_err(), "{source}");
    }
    for source in ["slot &&:= 1", "slot && := 1"] {
        assert!(parse_expression(source).is_err(), "{source}");
    }
    assert_eq!(
        parse_expression("slot && other := 1").unwrap(),
        parse_expression("slot && other").unwrap()
    );
    for source in [
        "slot && other",
        "slot and other",
        "slot && (other && last)",
        "slot &&= other",
        "slot ||= other",
        "slot := other",
        "slot ??:= other",
    ] {
        assert!(parse(source).is_ok(), "{source}");
    }
}

#[test]
fn pinned_negative_diagnostics_and_ignored_interpolation_suffixes_are_unchanged() {
    for (source, message, line, column) in [
        ("x ??= 1", "Unexpected '='", 1, 7),
        ("x = a ??= b", "Unexpected '='", 1, 11),
        ("x = a ??? b", "Unexpected '?'", 1, 11),
        ("x = c ?? 1 : 2", "Unexpected ':'", 1, 14),
        ("x := y := 1", "Unexpected ':='", 1, 11),
        ("x = 1\nif x:=2: pass", "Expected ':' but found ':='", 2, 5),
    ] {
        assert_eq!(
            error(&mut Runtime::new(), source),
            (
                ErrorKind::Syntax,
                message.to_string(),
                Some(Position::new(line, column))
            ),
            "{source}"
        );
    }
    checked(
        &mut Runtime::new(),
        r#"slot = 4
assert(`${slot:=missing &&:= 7}`, "4")
assert(`${slot && 2 := missing}`, "2")
assert(`${slot ?? missing := missing}`, "4")
assert(`${slot:=missing ??:= 7}`, "4")
assert(`${slot if missing}`, "4")
assert(slot, 4)"#,
    );
    assert!(parse(r#"text = `${slot &&:= 1}`"#).is_err());
}

#[test]
fn unsupported_targets_and_path_bounds_match_ordinary_value_assignment() {
    for target in [
        "xs[0]", "xs[0:1]", "f()", "this", "$Order", "(a+b)", "1", "(a&&b)",
    ] {
        let (kind, message, _) = error(&mut Runtime::new(), &format!("{target} &&:= 1"));
        assert_eq!(kind, ErrorKind::Syntax, "{target}");
        assert_eq!(message, "Invalid value assignment target", "{target}");
    }
    for target in [
        format!("x{}", ".p".repeat(127)),
        format!("x{}", ".p".repeat(128)),
        format!("x{}", ".p".repeat(129)),
        format!("(x{}){}", ".p".repeat(100), ".p".repeat(28)),
        format!("(x{}){}", ".p".repeat(100), ".p".repeat(29)),
        format!("{}x{}", "(".repeat(70), ")".repeat(70)),
    ] {
        let ordinary = Runtime::check(&format!("{target} := 1"));
        let guarded = Runtime::check(&format!("{target} &&:= 1"));
        assert_eq!(ordinary.is_ok(), guarded.is_ok(), "{target}");
        if let (Err(first), Err(second)) = (ordinary, guarded) {
            assert_eq!(first.kind(), second.kind(), "{target}");
            assert_eq!(first.message(), second.message(), "{target}");
        }
    }
}

#[test]
fn assertion_traversal_preserves_rhs_coverage() {
    let source = "slot = 1\nslot &&:= assert(2 + 3, 5)";
    assert_eq!(
        common::assertions::protected_assertions(&parse(source).unwrap()),
        ["assert(2+3,5)"]
    );
    checked(&mut Runtime::new(), source);
}

#[test]
fn every_falsy_kind_skips_without_state_graph_history_or_identity_changes() {
    for value in ["null", "false", "0", "-0", r#""""#, "Number.NaN"] {
        let mut runtime = Runtime::new();
        runtime.run(&format!("slot = {value}")).unwrap();
        let state = format!("{:?}", runtime.state);
        let graph = format!("{:?}", runtime.graph);
        assert_eq!(
            runtime.run("slot &&:= missing").unwrap(),
            Value::Null,
            "{value}"
        );
        assert_eq!(format!("{:?}", runtime.state), state, "{value}");
        assert_eq!(format!("{:?}", runtime.graph), graph, "{value}");
    }
    let mut runtime = Runtime::new();
    runtime.run("box = {}").unwrap();
    let state = format!("{:?}", runtime.state);
    let graph = format!("{:?}", runtime.graph);
    assert_eq!(runtime.run("box.absent &&:= missing").unwrap(), Value::Null);
    assert_eq!(format!("{:?}", runtime.state), state);
    assert_eq!(format!("{:?}", runtime.graph), graph);
}

#[test]
fn every_truthy_value_kind_takes_the_existing_action() {
    for value in [
        "true",
        "1",
        "-1",
        r#""0""#,
        r#""false""#,
        "[]",
        "[null]",
        "{}",
        "item -> item",
        "Date(0)",
        "/x/",
        "Number",
    ] {
        let mut runtime = Runtime::new();
        runtime.run(&format!("slot = {value}")).unwrap();
        assert_eq!(
            runtime.run("slot &&:= 9").unwrap(),
            Value::Number(9.0),
            "{value}"
        );
        checked(
            &mut runtime,
            "assert(slot, 9)\nassert((why slot)[0].from, [])",
        );
    }
}

#[test]
fn rhs_effects_and_extra_arguments_run_once_only_on_a_take() {
    checked(
        &mut Runtime::new(),
        r#"count = 0
def bump():
    count := count + 1
    return count
slot = 1
slot &&:= Math.abs(-3, bump(), bump())
assert([slot, count], [3, 2])
slot = 0
slot &&:= Math.abs(-4, bump(), missing)
assert([slot, count], [0, 2])
slot &&:= bump()
assert(count, 2)
slot = true
slot &&:= bump()
assert([slot, count], [3, 3])"#,
    );
}

#[test]
fn top_level_takes_file_literals_and_replace_only_the_taken_law() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"source = 2
slot = source + 1
slot &&:= 9
assert(slot, 9)
assert((why slot)[0].rule, "slot = 9")
assert((why slot)[0].state, "stated")
assert((why slot)[0].from, [])
source = 7
assert(slot, 9)
assert(slot@previous, 3)
assert(slot@last, 9)"#,
    );
    let key = NodeKey::variable("slot");
    assert!(
        runtime
            .graph
            .retrieve(&key)
            .unwrap()
            .dependencies
            .is_empty()
    );
    assert_eq!(runtime.graph.shape_dependencies(&key).count(), 0);
}

#[test]
fn skips_preserve_live_laws_and_scope_assignment_bookkeeping() {
    let mut runtime = Runtime::new();
    runtime
        .run("source = 0\nslot = source\nbox = {}\nbox.item = source")
        .unwrap();
    let state = format!("{:?}", runtime.state);
    let graph = format!("{:?}", runtime.graph);
    runtime
        .run("slot &&:= missing\nbox.item &&:= missing")
        .unwrap();
    assert_eq!(format!("{:?}", runtime.state), state);
    assert_eq!(format!("{:?}", runtime.graph), graph);
    checked(
        &mut runtime,
        "source = 4\nassert(slot, 4)\nassert(box.item, 4)\nassert(slot@previous, 0)\nassert(box.item@previous, 0)",
    );
    checked(
        &mut Runtime::new(),
        "slot = 0\n{\n    slot &&:= missing\n    { slot := 9 }\n}\nassert(slot, 9)",
    );
}

#[test]
fn guard_owner_rhs_calls_and_structural_reads_do_not_escape_to_enclosing_rules() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"class Item(n):
    this.n = n
    def amount() => this.n * factor
factor = 2
item = Item(3)
target = {answer: 1}
paid = false
def replacement() => Item.map(each -> each.amount()).sum() + (why factor).length
if paid: target.answer &&:= replacement()
paid = true
assert(target.answer, 7)
factor = 9
item.n = 5
assert(target.answer, 7)
target.answer = 2
assert(target.answer, 2)
paid = false
paid = true
assert(target.answer, 46)
other = {answer: 3}
target = other
assert(other.answer, 3)"#,
    );
    assert_eq!(dependencies(&runtime, "paid"), ["paid"]);
}

#[test]
fn skipped_actions_add_no_guard_or_rhs_subscription_to_enclosing_rules() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"paid = false
source = 4
slot = 0
if paid: slot &&:= source
paid = true
assert(slot, 0)
slot = 2
assert(slot, 2)
source = 9
assert(slot, 2)
paid = false
paid = true
assert(slot, 9)"#,
    );
    assert_eq!(dependencies(&runtime, "paid"), ["paid"]);
}

#[test]
fn branch_actions_carry_out_without_replacing_the_existing_law() {
    checked(
        &mut Runtime::new(),
        r#"source = 2
slot = source + 1
enabled = false
if enabled: slot &&:= 12
enabled = true
assert(slot, 12)
source = 8
assert(slot, 9)
assert((why slot)[0].rule, "slot = source+1")
slot &&:= 0
assert(slot, 0)
source = 11
assert(slot, 0)"#,
    );
}

#[test]
fn local_parameters_loops_methods_and_receivers_keep_existing_contexts() {
    checked(
        &mut Runtime::new(),
        r#"def consume(slot, replacement):
    slot &&:= replacement
    return slot
assert(consume(1, 4), 4)
assert(consume(0, 4), 0)
base = 1
slot = base
for item of "ab": slot &&:= item
assert(slot, "b")
base = 8
assert(slot, 8)
class Box:
    def init(n):
        this.base = n
        this.answer = 1
        this.answer &&:= .base + 1
    def consume():
        this.answer &&:= .base + 2
        return .answer
box = Box(3)
assert(box.answer, 4)
assert(box.consume(), 5)
box.answer = 0
assert(box.consume(), 0)"#,
    );
}

#[test]
fn class_rule_walkers_keep_instance_context_without_a_reactive_guard() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"class Order(qty, paid):
    this.qty = qty
    this.paid = paid
    this.taken = 1
if $Order.paid:
    $Order.taken &&:= $Order.qty
small = Order(2, true)
large = Order(5, false)
assert(small.taken, 2)
large.paid = true
assert(large.taken, 5)
large.qty = 9
assert(large.taken, 5)
large.taken = 0
large.paid = false
large.paid = true
assert(large.taken, 0)
last = Order(7, true)
assert(last.taken, 7)"#,
    );
}

#[test]
fn owner_resolution_precedes_rhs_effects_and_constructors_keep_target_names() {
    checked(
        &mut Runtime::new(),
        r#"class Box
first = Box()
first.item = 1
second = Box()
second.item = 2
route = first
def move():
    route := second
    return 9
route.item &&:= move()
assert(first.item, 9)
assert(second.item, 2)
assert(route == second, true)
class Item(n): this.n = n
selected = 1
selected &&:= Item(n=3)
assert(selected.id, "selected")
assert(selected.n, 3)
first.item &&:= Item(n=5)
assert(first.item.id, "first.item")
assert(first.item.n, 5)
selected = 0
selected &&:= Item(n=9)
assert(Item.length, 2)"#,
    );
}

#[test]
fn nullish_guards_do_not_poison_later_actions_or_disable_rhs_settlement() {
    checked(
        &mut Runtime::new(),
        r#"nullable = null
slot = 1
slot &&:= nullable + 1
assert(slot, null)
nullable = 5
assert(slot, null)
slot &&:= missing
assert(slot, null)
slot = 1
slot &&:= nullable.value
assert(slot, 5)
box = {}
slot &&:= box.absent
assert(slot, null)
done = 0
{
    slot &&:= missing
    box.absent &&:= missing
    done := 7
}
assert(done, 7)"#,
    );
}

#[test]
fn target_validation_and_read_only_errors_apply_even_to_skipped_targets() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Box\nbox = Box()\nnumber = 0\nslot = 0")
        .unwrap();
    for (source, kind, message) in [
        (
            "unbound &&:= 7",
            ErrorKind::Reference,
            "unbound is not defined",
        ),
        (
            "missing.item &&:= 7",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "box.inner.item &&:= 7",
            ErrorKind::Reference,
            "box.inner is not defined",
        ),
        (
            "number.item &&:= 7",
            ErrorKind::Type,
            "Cannot assign 'item' on number",
        ),
        (
            "box.value &&:= 7",
            ErrorKind::Type,
            "Cannot use 'value' as a property",
        ),
        (
            "slot@previous &&:= 7",
            ErrorKind::Type,
            "State versions are read-only",
        ),
        (
            "$Box.item &&:= 7",
            ErrorKind::Type,
            "Cannot use ':=' on a class-level property",
        ),
        (
            "Box.item &&:= 7",
            ErrorKind::Type,
            "Cannot use ':=' on a class-level property",
        ),
        (
            "$Missing.item &&:= 7",
            ErrorKind::Reference,
            "Missing is not defined",
        ),
        (
            "this.item &&:= 7",
            ErrorKind::Reference,
            "this is not defined",
        ),
    ] {
        assert_eq!(
            error(&mut runtime, source),
            (kind, message.to_string(), Some(Position::new(1, 1))),
            "{source}"
        );
    }
    runtime.run("old = box@last").unwrap();
    checked(
        &mut runtime,
        "old.item := 0\nold.item &&:= missing\nassert(box.item, 0)\nold.item = 1\nold.item &&:= 7\nassert(box.item, 7)\nslot &&:= missing\nassert(slot, 0)",
    );
}

#[test]
fn failed_taken_rhs_and_single_dependent_propagation_roll_back_and_recover() {
    let mut runtime = Runtime::new();
    runtime.run("slot = 1\ncount = 0\ndef fail():\n    count := count + 1\n    throw \"FAILED\"\nif slot > 8: throw \"LIMIT\"").unwrap();
    for (source, kind, message) in [
        ("slot &&:= fail()", ErrorKind::Thrown, "FAILED"),
        ("slot &&:= 9", ErrorKind::Thrown, "LIMIT"),
        (
            "slot &&:= missing",
            ErrorKind::Reference,
            "missing is not defined",
        ),
    ] {
        let (found, text, _) = error(&mut runtime, source);
        assert_eq!((found, text.as_str()), (kind, message));
        checked(
            &mut runtime,
            "assert(slot, 1)\nassert(count, 0)\nassert(slot@last, 1)",
        );
    }
    checked(
        &mut runtime,
        "slot &&:= 4\nassert(slot, 4)\nassert(slot@previous, 1)",
    );
}

#[test]
fn caught_errors_hypothetical_actions_and_deferred_bodies_restore_context() {
    checked(
        &mut Runtime::new(),
        r#"slot = 1
caught = false
try:
    slot &&:= missing
catch error:
    caught = true
assert(caught, true)
assert(slot, 1)
result = suppose:
    slot &&:= 5
    slot
assert(result, 5)
assert(slot, 1)
def finish():
    defer:
        slot &&:= 7
    return 2
assert(finish(), 2)
assert(slot, 7)"#,
    );
}

#[test]
fn runtime_depth_failures_match_the_taken_action_and_recover() {
    let mut runtime = Runtime::new();
    runtime
        .run("slot = 1\ndeep = 0\nrepeat 400: deep = [deep]")
        .unwrap();
    let ordinary = error(&mut runtime, "slot := deep");
    assert_eq!(ordinary.0, ErrorKind::Type);
    assert_eq!(ordinary.1, "Maximum expression depth exceeded");
    assert_eq!(error(&mut runtime, "slot &&:= deep"), ordinary);
    runtime
        .run("slot = 0\ndef recurse(n) => recurse(n + 1)")
        .unwrap();
    runtime.run("slot &&:= recurse(0)").unwrap();
    runtime.run("slot = 1").unwrap();
    let ordinary = error(&mut runtime, "slot := recurse(0)");
    assert_eq!(ordinary.0, ErrorKind::Type);
    assert_eq!(error(&mut runtime, "slot &&:= recurse(0)"), ordinary);
    checked(&mut runtime, "slot &&:= 3\nassert(slot, 3)");
}

#[test]
fn exactly_two_paired_document_cases_keep_six_ordered_assertions_and_string_returns() {
    for (document, old_count, title, assertions) in [
        (
            include_str!("../nucleoid.spec.md"),
            897,
            "Nucleoid writes truthy values with one-shot assignments",
            [
                "assert(slot,9)",
                "assert(slot,9)",
                "assert(empty,0)",
                "assert(active,5)",
                "assert(text,\"\")",
                "assert(input,7)",
            ],
        ),
        (
            include_str!("reference.md"),
            174,
            "Nucleoid keeps truthy assignment guards isolated",
            [
                "assert(gate.target,3)",
                "assert(gate.target,5)",
                "assert(gate.target,12)",
                "assert(gate.target,9)",
                "assert(gate.target,0)",
                "assert(gate.target,0)",
            ],
        ),
    ] {
        let cases = common::cases(document);
        assert_eq!(cases.len(), old_count + 1);
        let case = cases.last().unwrap();
        assert_eq!(case.title, title);
        assert_eq!(cases.iter().filter(|case| case.title == title).count(), 1);
        assert_eq!(case.expected.as_deref(), Some("1"));
        assert_eq!(
            common::assertions::protected_assertions(&parse(&case.source).unwrap()),
            assertions
        );
        common::check_with_assertions(case, 6).unwrap();
    }
}
