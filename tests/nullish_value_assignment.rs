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
            title: "nullish value assignment regression".to_string(),
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

fn identifier(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn assignment(target: Expr, value: Expr) -> Stmt {
    Stmt::NullishValueAssign { target, value }
}

fn rule_dependencies(runtime: &Runtime, condition: &str) -> Vec<String> {
    let key = NodeKey::conditional(&parse_expression(condition).unwrap(), None);
    let node = runtime.graph.retrieve(&key).unwrap();
    assert_eq!(runtime.graph.shape_dependencies(&key).count(), 0);
    node.dependencies.iter().map(ToString::to_string).collect()
}

#[test]
fn the_operator_reuses_two_existing_token_kinds() {
    for source in ["slot??:=1", "slot ?? := 1", "slot ?? /* gap */ := 1"] {
        let tokens: Vec<_> = tokenize(source)
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .filter(|token| !matches!(token, Token::Newline | Token::Eof))
            .collect();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("slot".to_string()),
                Token::QuestionQuestion,
                Token::ColonAssign,
                Token::Number(1.0),
            ],
            "{source}"
        );
        assert_eq!(
            parse(source).unwrap(),
            vec![assignment(identifier("slot"), Expr::Number(1.0))]
        );
    }
}

#[test]
fn paths_grouping_and_contextual_names_keep_their_statement_readings() {
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
    ] {
        let nullish = format!("{target} ??:= source");
        let ordinary = parse(&format!("{target} := source")).unwrap();
        let [Stmt::ValueAssign { target, value }] = ordinary.as_slice() else {
            panic!("{ordinary:?}");
        };
        assert_eq!(
            parse(&nullish).unwrap(),
            vec![assignment(target.clone(), value.clone())]
        );
    }
    let statements = parse("if paid: slot ??:= source\n{ box.item ??:= 2 }").unwrap();
    let Stmt::If { consequent, .. } = &statements[0] else {
        panic!("{statements:?}");
    };
    assert_eq!(
        consequent,
        &vec![assignment(identifier("slot"), identifier("source"))]
    );
    assert!(
        matches!(&statements[1], Stmt::Block(body) if matches!(body.as_slice(), [Stmt::NullishValueAssign { .. }]))
    );
}

#[test]
fn rendering_distinguishes_actions_and_round_trips_new_statements() {
    for (source, rendered) in [
        ("slot ?? := source + 1", "slot??:=source+1"),
        ("box.item ??:= c ? 1 : 2", "box.item??:=(c?1:2)"),
        ("slot ??:= first ?? last", "slot??:=(first??last)"),
        ("slot ??:= other = 3", "slot??:=other=3"),
    ] {
        let statements = parse(source).unwrap();
        assert_eq!(generate(&statements[0]), rendered);
        assert_eq!(parse(rendered).unwrap(), statements);
    }
    let nullish = parse("slot ??:= source").unwrap();
    let unconditional = parse("slot := source").unwrap();
    let law = parse("slot = source").unwrap();
    assert_eq!(generate_all(&nullish), "slot??:=source");
    assert_ne!(
        NodeKey::block(&nullish, None),
        NodeKey::block(&unconditional, None)
    );
    assert_ne!(NodeKey::block(&nullish, None), NodeKey::block(&law, None));
    let source = "def fill(slot = null):\n    slot ??:= 5\n    return slot";
    let statements = parse(source).unwrap();
    assert_eq!(parse(&generate(&statements[0])).unwrap(), statements);
}

#[test]
fn assertion_traversal_enters_the_new_rhs_without_losing_coverage() {
    let source = "slot = null\nslot ??:= assert(2 + 3, 5)";
    assert_eq!(
        common::assertions::protected_assertions(&parse(source).unwrap()),
        vec!["assert(2+3,5)"]
    );
    checked(&mut Runtime::new(), source);
}

#[test]
fn only_the_statement_head_can_terminate_a_coalescing_expression() {
    for source in [
        "result = slot ??:= 1",
        "f(slot ??:= 1)",
        "[slot ??:= 1]",
        "(slot ??:= 1)",
        "return slot ??:= 1",
        "if slot ??:= 1: pass",
        "slot ??:= other ??:= 1",
        "slot ??:= (other ??:= 1)",
        "slot ??:= item -> item ??:= 1",
        "slot ??:= lambda item: item ??:= 1",
        "class Slot(value ??:= 1): pass",
        "def fill(value ??:= 1): pass",
    ] {
        assert!(Runtime::check(source).is_err(), "{source}");
    }
    for source in ["slot ??:= 1", "slot ?? := 1", "slot ??= 1"] {
        assert!(parse_expression(source).is_err(), "{source}");
    }
    assert_eq!(parse_expression("slot := 1").unwrap(), identifier("slot"));
    assert_eq!(
        parse_expression("slot ?? 1 := 2").unwrap(),
        parse_expression("slot ?? 1").unwrap()
    );
}

#[test]
fn old_negative_positions_and_ignored_template_suffixes_stay_intact() {
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
assert(`${slot:=missing}`, "4")
assert(`${slot ?? missing := missing}`, "4")
assert(`${slot:=missing ??:= 7}`, "4")
assert(`${slot if missing}`, "4")
assert(`${slot lambda missing}`, "4")
assert(slot, 4)"#,
    );
    assert!(parse(r#"text = `${slot ??:= 1}`"#).is_err());
}

#[test]
fn invalid_targets_and_path_limits_are_shared_with_value_assignment() {
    for target in [
        "xs[0]", "xs[0:1]", "f()", "this", "$Order", "(a+b)", "1", "(a??b)",
    ] {
        let source = format!("{target} ??:= 1");
        let (kind, message, _) = error(&mut Runtime::new(), &source);
        assert_eq!(kind, ErrorKind::Syntax, "{source}");
        assert_eq!(message, "Invalid value assignment target", "{source}");
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
        let nullish = Runtime::check(&format!("{target} ??:= 1"));
        assert_eq!(ordinary.is_ok(), nullish.is_ok(), "{target}");
        if let (Err(first), Err(second)) = (ordinary, nullish) {
            assert_eq!(first.kind(), second.kind(), "{target}");
            assert_eq!(first.message(), second.message(), "{target}");
        }
    }
}

#[test]
fn every_non_nullish_kind_skips_without_changing_state_or_graph() {
    for value in [
        "false",
        "0",
        "-0",
        r#""""#,
        "Number.NaN",
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
        let state = format!("{:?}", runtime.state);
        let graph = format!("{:?}", runtime.graph);
        assert!(runtime.run("slot ??:= missing").is_ok(), "{value}");
        assert_eq!(format!("{:?}", runtime.state), state, "{value}");
        assert_eq!(format!("{:?}", runtime.graph), graph, "{value}");
    }
}

#[test]
fn null_and_missing_properties_take_a_constant_without_guard_deferral() {
    checked(
        &mut Runtime::new(),
        r#"slot = null
slot ??:= 7
assert(slot, 7)
box = {}
box.item ??:= 8
assert(box.item, 8)
assert((why slot)[0].state, "stated")
assert((why slot)[0].from, [])
assert((why box.item)[0].from, [])
assert(slot@last, 7)
assert(box.item@last, 8)"#,
    );
}

#[test]
fn a_nullable_rhs_keeps_the_ordinary_assignment_settling_rules() {
    checked(
        &mut Runtime::new(),
        r#"nullable = null
ordinary := nullable + 1
slot = null
slot ??:= nullable + 1
assert(ordinary, null)
assert(slot, null)
nullable = 5
assert(slot, null)
slot ??:= nullable + 1
assert(slot, 6)
slot = null
slot ??:= nullable.value
assert(slot, 5)
box = {}
slot = null
slot ??:= box.absent
assert(slot, null)
slot ??:= 9
assert(slot, 9)"#,
    );
}

#[test]
fn the_rhs_runs_once_only_when_the_target_is_nullish() {
    checked(
        &mut Runtime::new(),
        r#"count = 0
def bump():
    count := count + 1
    return count
slot = null
slot ??:= bump()
assert([slot, count], [1, 1])
slot ??:= bump()
assert([slot, count], [1, 1])
slot ??:= missing
assert(count, 1)
slot = null
slot ??:= bump()
assert([slot, count], [2, 2])
slot = false
slot ??:= bump()
assert([slot, count], [false, 2])"#,
    );
}

#[test]
fn skipping_preserves_standing_declarations_histories_and_dependencies() {
    let mut runtime = Runtime::new();
    runtime
        .run("source = 2\nslot = source * 3\nbox = {}\nbox.item = source + 1")
        .unwrap();
    let state = format!("{:?}", runtime.state);
    let graph = format!("{:?}", runtime.graph);
    runtime
        .run("slot ??:= missing\nbox.item ??:= missing")
        .unwrap();
    assert_eq!(format!("{:?}", runtime.state), state);
    assert_eq!(format!("{:?}", runtime.graph), graph);
    checked(
        &mut runtime,
        "source = 4\nassert(slot, 12)\nassert(box.item, 5)\nassert(slot@previous, 6)\nassert(box.item@previous, 3)",
    );
}

#[test]
fn a_skipped_name_does_not_change_enclosing_scope_assignment_bookkeeping() {
    checked(
        &mut Runtime::new(),
        "slot = 1\n{\n    slot ??:= missing\n    { slot := 9 }\n}\nassert(slot, 9)",
    );
}

#[test]
fn taking_a_top_level_action_stores_a_fact_without_fallback_subscriptions() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"source = 4
slot = null
slot ??:= source
assert(slot, 4)
assert((why slot)[0].rule, "slot = 4")
assert((why slot)[0].from, [])
source = 9
assert(slot, 4)
assert(slot@previous, null)
assert(slot@last, 4)"#,
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
fn guard_target_and_called_code_are_isolated_from_a_standing_enclosing_if() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"class Item(n):
    this.n = n
    def amount() => this.n * factor
factor = 2
item = Item(3)
target = {}
paid = false
def fallback() => Item.map(each -> each.amount()).sum() + (why factor).length
if paid: target.answer ??:= fallback()
paid = true
assert(target.answer, 7)
factor = 9
item.n = 5
assert(target.answer, 7)
target.answer = null
assert(target.answer, null)
paid = false
paid = true
assert(target.answer, 46)
other = {}
target = other
assert(other.answer == 46, false)"#,
    );
    assert_eq!(rule_dependencies(&runtime, "paid"), ["paid"]);
}

#[test]
fn skipped_actions_add_no_guard_or_structural_subscription_to_enclosing_rules() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"paid = false
source = 4
slot = 8
if paid: slot ??:= source
paid = true
assert(slot, 8)
slot = null
assert(slot, null)
source = 9
assert(slot, null)
paid = false
paid = true
assert(slot, 9)"#,
    );
    assert_eq!(rule_dependencies(&runtime, "paid"), ["paid"]);
}

#[test]
fn imperative_bodies_local_parameters_and_methods_execute_once_per_entry() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"def fill(slot, fallback):
    slot = null
    slot ??:= fallback
    return slot
assert(fill(0, 4), 4)
def keep(slot, fallback):
    slot ??:= fallback
    return slot
def ordinary(slot, fallback):
    slot := fallback
    return slot
assert(keep(null, 4), ordinary(null, 4))
assert(keep(0, missing.value if false else 9), 0)"#,
    );
    checked(
        &mut runtime,
        r#"base = null
slot = base
for item of "ab":
    slot ??:= item
assert(slot, "a")
base = 8
assert(slot, 8)"#,
    );
    checked(
        &mut runtime,
        r#"class Box:
    def init(n): this.item ??:= n
    def fill(n):
        this.item ??:= n
        return this.item
box = Box(null)
assert(box.fill(5), 5)
assert(box.fill(9), 5)
assert(box.item, 5)"#,
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
if $Order.paid:
    $Order.taken ??:= $Order.qty
small = Order(2, true)
large = Order(5, false)
assert(small.taken, 2)
large.paid = true
assert(large.taken, 5)
large.qty = 9
assert(large.taken, 5)
large.taken = null
assert(large.taken, null)
large.paid = false
large.paid = true
assert(large.taken, 9)
large.taken = 0
large.paid = false
large.paid = true
assert(large.taken, 0)
last = Order(7, true)
assert(last.taken, 7)"#,
    );
}

#[test]
fn target_owner_resolution_still_precedes_rhs_effects() {
    checked(
        &mut Runtime::new(),
        r#"class Box
first = Box()
second = Box()
route = first
def move():
    route := second
    return 9
route.item ??:= move()
assert(first.item, 9)
assert(second.item == 9, false)
assert(route == second, true)"#,
    );
}

#[test]
fn taken_constructors_retain_target_naming_and_named_argument_binding() {
    checked(
        &mut Runtime::new(),
        r#"class Item(n): this.n = n
class Box
slot = null
slot ??:= Item(n=3)
assert(slot.id, "slot")
assert(slot.n, 3)
slot ??:= Item(n=9)
assert(slot.n, 3)
holder = Box()
holder.item ??:= Item(n=5)
assert(holder.item.id, "holder.item")
assert(holder.item.n, 5)
holder.item ??:= Item(n=9)
assert(Item.length, 2)"#,
    );
}

#[test]
fn property_receivers_and_instance_shorthand_share_the_existing_rhs_scope() {
    checked(
        &mut Runtime::new(),
        r#"box = {base: 5}
box.answer ??:= .base * 2
assert(box.answer, 10)
box.base = 9
assert(box.answer, 10)
class Item:
    def init(n):
        this.base = n
        this.answer ??:= .base + 1
    def fill():
        this.answer ??:= .base + 2
        return .answer
item = Item(3)
assert(item.answer, 4)
item.answer = null
assert(item.fill(), 5)
assert(item.fill(), 5)"#,
    );
    for (nullish, ordinary) in [
        ("box.answer ??:= .base", "box.answer := .base"),
        ("this.answer ??:= .base", "this.answer := .base"),
    ] {
        let [Stmt::ValueAssign { target, value }] = &parse(ordinary).unwrap()[..] else {
            panic!("{ordinary}");
        };
        assert_eq!(
            parse(nullish).unwrap(),
            vec![assignment(target.clone(), value.clone())]
        );
    }
}

#[test]
fn extra_argument_effects_and_guard_independence_match_the_taken_action() {
    checked(
        &mut Runtime::new(),
        r#"count = 0
def bump():
    count := count + 1
    return count
slot = null
slot ??:= Math.abs(-3, bump(), bump())
assert(slot, 3)
assert(count, 2)
slot ??:= Math.abs(-4, bump(), missing)
assert(slot, 3)
assert(count, 2)
ordinary := Math.abs(-3, bump(), bump())
assert(ordinary, slot)
assert(count, 4)"#,
    );
}

#[test]
fn missing_names_invalid_storage_and_class_boundaries_raise_ordinary_errors() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Box\nbox = Box()\nnumber = 1\nslot = null")
        .unwrap();
    for (source, kind, message) in [
        (
            "unbound ??:= 7",
            ErrorKind::Reference,
            "unbound is not defined",
        ),
        (
            "missing.item ??:= 7",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "box.inner.item ??:= 7",
            ErrorKind::Reference,
            "box.inner is not defined",
        ),
        (
            "number.item ??:= 7",
            ErrorKind::Type,
            "Cannot assign 'item' on number",
        ),
        (
            "box.value ??:= 7",
            ErrorKind::Type,
            "Cannot use 'value' as a property",
        ),
        (
            "slot@previous ??:= 7",
            ErrorKind::Type,
            "State versions are read-only",
        ),
        (
            "$Box.item ??:= 7",
            ErrorKind::Type,
            "Cannot use ':=' on a class-level property",
        ),
        (
            "Box.item ??:= 7",
            ErrorKind::Type,
            "Cannot use ':=' on a class-level property",
        ),
        (
            "$Missing.item ??:= 7",
            ErrorKind::Reference,
            "Missing is not defined",
        ),
        (
            "this.item ??:= 7",
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
    checked(&mut runtime, "slot ??:= 8\nassert(slot, 8)");
}

#[test]
fn failed_taken_rhs_and_single_dependent_propagation_roll_back_and_recover() {
    let mut runtime = Runtime::new();
    runtime
        .run("slot = null\ncount = 0\ndef fail():\n    count := count + 1\n    throw \"FAILED\"\nif slot > 8: throw \"LIMIT\"")
        .unwrap();
    for (source, kind, message) in [
        ("slot ??:= fail()", ErrorKind::Thrown, "FAILED"),
        ("slot ??:= 9", ErrorKind::Thrown, "LIMIT"),
        (
            "slot ??:= missing",
            ErrorKind::Reference,
            "missing is not defined",
        ),
    ] {
        let (found, text, _) = error(&mut runtime, source);
        assert_eq!((found, text.as_str()), (kind, message));
        checked(
            &mut runtime,
            "assert(slot, null)\nassert(count, 0)\nassert(slot@last, null)",
        );
    }
    checked(
        &mut runtime,
        "slot ??:= 4\nassert(slot, 4)\nassert(slot@previous, null)",
    );
}

#[test]
fn caught_errors_and_suppose_restore_the_guard_and_action_context() {
    checked(
        &mut Runtime::new(),
        r#"slot = null
caught = false
try:
    slot ??:= missing
catch error:
    caught = true
assert(caught, true)
assert(slot, null)
result = suppose:
    slot ??:= 5
    slot
assert(result, 5)
assert(slot, null)
slot ??:= 7
assert(slot, 7)"#,
    );
}

#[test]
fn deep_non_nullish_values_skip_without_literal_conversion() {
    let mut runtime = Runtime::new();
    runtime.run("deep = 0\nrepeat 400: deep = [deep]").unwrap();
    runtime.run("deep ??:= missing").unwrap();
    checked(&mut runtime, "assert(deep.length, 1)");
    runtime.run("slot = null").unwrap();
    let ordinary = error(&mut runtime, "slot := deep");
    assert_eq!(ordinary.0, ErrorKind::Type);
    assert_eq!(ordinary.1, "Maximum expression depth exceeded");
    assert_eq!(error(&mut runtime, "slot ??:= deep"), ordinary);
    checked(
        &mut runtime,
        "assert(slot, null)\nslot ??:= 6\nassert(slot, 6)",
    );
}

#[test]
fn recursive_rhs_failure_leaves_the_action_usable_afterward() {
    let mut runtime = Runtime::new();
    runtime
        .run("slot = null\ndef recurse(n) => recurse(n + 1)")
        .unwrap();
    let ordinary = error(&mut runtime, "slot := recurse(0)");
    assert_eq!(ordinary.0, ErrorKind::Type);
    assert_eq!(error(&mut runtime, "slot ??:= recurse(0)"), ordinary);
    checked(&mut runtime, "slot ??:= 3\nassert(slot, 3)");
}

#[test]
fn exactly_the_two_paired_document_cases_are_appended_with_six_assertions_each() {
    for (document, old_count, title) in [
        (
            include_str!("../nucleoid.spec.md"),
            896,
            "Nucleoid fills nullish values with one-shot assignments",
        ),
        (
            include_str!("reference.md"),
            170,
            "Nucleoid keeps nullish assignment guards isolated",
        ),
    ] {
        let cases = common::cases(document);
        assert_eq!(
            cases.len(),
            old_count
                + 6
                + usize::from(title == "Nucleoid fills nullish values with one-shot assignments")
        );
        assert_eq!(cases.get(old_count).unwrap().title, title);
        assert_eq!(cases.iter().filter(|case| case.title == title).count(), 1);
        let case = cases.get(old_count).unwrap();
        assert_eq!(case.expected.as_deref(), Some("1"));
        let assertions = common::assertions::protected_assertions(&parse(&case.source).unwrap());
        assert_eq!(assertions.len(), 6);
        common::check_with_assertions(case, 6).unwrap();
    }
}
