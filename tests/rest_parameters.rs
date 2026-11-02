//! Final rest formals reuse the fixed/default binder without changing legacy
//! syntax, constructor label validation, invocation identity or transactions.

mod common;

use std::sync::Arc;

use nucleoid::lang::ast::generator::{generate, generate_all};
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_expression};
use nucleoid::{ErrorKind, ObjectId, Runtime, Value};

fn checked(source: &str) -> Runtime {
    let assertions = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let mut runtime = Runtime::new();
    common::check_with_count(
        &mut runtime,
        &common::Case {
            title: "rest parameters".to_string(),
            source: source.to_string(),
            expected: None,
        },
        assertions,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run(), assertions, "{source}");
    runtime
}

#[test]
fn parenthesized_routes_capture_only_surplus_values_and_preserve_fixed_defaults() {
    for definition in [
        "def collect(head = 1, ...tail: Unresolved) => [head, tail]",
        "def collect(head = 1, ...tail: Unresolved) ->\n    [head, tail]",
        "def collect(head = 1, ...tail: Unresolved): return [head, tail]",
        "def collect(\n    head = 1,\n    ...tail: Unresolved,\n):\n    return [head, tail]",
        "collect = (head = 1, ...tail: Unresolved) => [head, tail]",
        "collect = (head = 1, ...tail: Unresolved) -> { return [head, tail] }",
        "collect = function(head = 1, ...tail: Unresolved) { return [head, tail] }",
    ] {
        checked(&format!(
            r#"
{definition}
assert(collect(), [1, []])
assert(collect(0), [0, []])
assert(collect(1, 2, 3), [1, [2, 3]])
assert(collect(false, "", [4, 5]), [false, ["", [4, 5]]])
"#
        ));
    }
    checked(
        r#"
def collect(...items) => items
def mixed(first = 1, required, ...items) => [first, required, items]
def repeated(value = 1, value = 2, ...items) => [value@0, value@1, value, items]
alias = collect
holder = {call: collect}
callbacks = [collect]
def apply(callback) => callback(4, 5)
def factory() => (...items) => items
returned = factory()
assert(collect(), [])
assert(collect(null).value, [null])
assert(mixed().value, [1, null, []])
assert(mixed(2, 3, 4), [2, 3, [4]])
assert(repeated(), [1, 2, 2, []])
assert(alias(1, 2), [1, 2])
assert(holder.call(2, 3), [2, 3])
assert(callbacks[0](3, 4), [3, 4])
assert(apply(collect), [4, 5])
assert(returned(5, 6), [5, 6])
assert([2, 3].map((item, ...extras) => [item, extras]), [[2, []], [3, []]])
assert([2, 3].reduce((sum, ...items) => sum + items[0], 0), 5)
def fixed(first = 1, second) => [first, second]
def bare first second -> [first, second]
assert(fixed().value, [1, null])
assert(fixed(2, 3, 4), [2, 3])
assert(bare(2, 3, 4), [2, 3])
"#,
    );
    let mut runtime = checked(
        "class Empty\nbox = Empty()\ndef kinds(...values) => values.map(value => typeof value)",
    );
    assert_eq!(
        runtime
            .run("kinds(box.missing, null, false, 0, [])")
            .unwrap(),
        Value::List(vec![
            Value::String("undefined".to_string()),
            Value::String("null".to_string()),
            Value::String("boolean".to_string()),
            Value::String("number".to_string()),
            Value::Class("List".to_string()),
        ])
    );
}

#[test]
fn fresh_lists_shadow_state_start_at_version_zero_and_do_not_create_closures() {
    checked(
        r#"
tail = [100]
def history(...tail):
    tail.push(9)
    return [tail@0, tail@last, tail@previous, tail@2]
first = history(1, 2).value
second = history().value
third = history().value
assert(first, [[1, 2], [1, 2, 9], [1, 2], null])
assert(second, [[], [9], [], null])
assert(third, [[], [9], [], null])
assert(tail, [100])
original = [3, 4]
def mutate(...tail):
    tail.push(8)
    return tail
assert(mutate(original), [[3, 4], 8])
assert(original, [3, 4])
shared = {level: 1}
def update(...tail):
    tail[0].level = 4
    return tail.length
assert(update(shared), 1)
assert(shared.level, 4)
def factory(...tail) => () => tail
callback = factory(7, 8)
assert(callback(), [100])
state = 1
def attempt(...tail):
    try:
        tail.push(9)
        state = 2
        throw "LOCAL_REST"
    catch error:
        state = state
    return [tail@0, tail, state]
assert(attempt(1), [[1], [1, 9], 1])
assert(state, 1)
class Packet(...tail):
    this.initial = tail@0
    tail.push(9)
    this.latest = tail@last
    def observe(...tail):
        tail.push(this.initial.length)
        return [tail@0, tail@last]
one = Packet(1, 2)
two = Packet()
assert(one.initial, [1, 2])
assert(one.latest, [1, 2, 9])
assert(two.initial, [])
assert(two.latest, [9])
assert(one.observe(3), [[3], [3, 2]])
assert(two.observe(), [[], [0]])
assert(tail, [100])
"#,
    );
}

#[test]
fn constructors_bind_fixed_labels_and_positional_rest_once_across_init_inheritance_and_super() {
    checked(
        r#"
class Packet(head = 1, ...tail):
    this.head = head
    this.tail = tail
    def read(...extra) => [self.head, extra]
empty = Packet()
positional = Packet(2, 3, 4)
named = Packet(head=5)
nested = new Packet(6, [7, 8])
assert(empty.head, 1)
assert(empty.tail, [])
assert(positional.tail, [3, 4])
assert(named.head, 5)
assert(named.tail, [])
assert(nested.tail, [[7, 8]])
assert(positional.read(9, 10), [2, [9, 10]])
class Pair(first = 1, second = 2, ...tail):
    this.values = [first, second, tail]
mixed = Pair(4, second=7)
namedPair = Pair(second=8)
assert(mixed.values, [4, 7, []])
assert(namedPair.values, [1, 8, []])
class Initialized:
    def init(head = 3, ...tail):
        this.head = head
        this.tail = tail
class Inherited: Initialized
    pass
class Skipped(...ignored): Initialized
    pass
class Child: Initialized
    def init(...items):
        super(10, 20, 30)
        this.items = items
class NamedChild: Initialized
    super(head=8)
initialized = Initialized(4, 5, 6)
inherited = Inherited(7, 8, 9)
skipped = Skipped(head=11)
child = Child(40, 50)
namedChild = NamedChild()
assert(initialized.tail, [5, 6])
assert(inherited.tail, [8, 9])
assert(skipped.head, 11)
assert(skipped.tail, [])
assert(child.head, 10)
assert(child.tail, [20, 30])
assert(child.items, [40, 50])
assert(namedChild.head, 8)
assert(namedChild.tail, [])
record = {packet: Packet(1, 2, 3)}
items = [Packet(2, 3)]
assert(record.packet.tail, [2, 3])
assert(items[0].tail, [3])
assert(Packet(3, 4).tail, [4])
class Holder
holder = Holder()
$Holder.packet = Packet(4, 5, 6)
assert(holder.packet.tail, [5, 6])
class Shell(...items)
shell = Shell(1, 2)
assert(shell, {id: "shell"})
"#,
    );
}

#[test]
fn argument_order_single_evaluation_and_receiver_and_constructor_selection_are_unchanged() {
    let mut runtime = checked(
        r#"
events = {trace: []}
def mark(value):
    events.trace.push(value)
    return value
def collect(head, ...tail) => [head, tail]
class Packet(head, ...tail):
    this.head = head
    this.tail = tail
"#,
    );
    assert_eq!(
        runtime.run("collect(mark(1), mark(2), mark(3))").unwrap(),
        Value::List(vec![
            Value::Number(1.0),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ])
    );
    runtime
        .run("packet = Packet(mark(4), mark(5), mark(6))\nnamed = Packet(head=mark(7))")
        .unwrap();
    assert_eq!(
        runtime.run("events.trace").unwrap(),
        Value::List(
            (1..=7)
                .map(|value| Value::Number(f64::from(value)))
                .collect()
        )
    );
    assert_eq!(
        runtime.run("packet.tail").unwrap(),
        Value::List(vec![Value::Number(5.0), Value::Number(6.0)])
    );
    assert_eq!(runtime.run("named.tail").unwrap(), Value::List(Vec::new()));

    for (call, expected) in [
        ("Packet(head=replace())", "[2, []]"),
        ("Packet(replace(), 3, 4)", "[2, 3, 4]"),
    ] {
        checked(&format!(
            r#"
class Packet(head, ...tail):
    this.values = [head, tail]
def replace():
    class Packet(...all):
        this.values = all
    return 2
first = {call}
assert(first.values, {expected})
second = Packet(5, 6)
assert(second.values, [5, 6])
"#
        ));
    }
    let mut runtime = checked(
        r#"
class Meter(seed):
    this.seed = seed
    def read(...items) => [this.seed, items]
first = Meter(1)
second = Meter(2)
def swap():
    first = second
    return 3
"#,
    );
    // Observe the call directly: effectful list-valued assignments retain
    // their accepted settlement/re-evaluation behavior.
    assert_eq!(
        runtime.run("first.read(swap(), 4)").unwrap(),
        Value::List(vec![
            Value::Number(1.0),
            Value::List(vec![Value::Number(3.0), Value::Number(4.0)]),
        ])
    );
    assert_eq!(
        runtime.run("first").unwrap(),
        runtime.run("second").unwrap()
    );
    checked(
        r#"
'use imperative'
a = b = 0
def assignments(...items) => items
assert(assignments(b=5, a=6), [5, 6])
assert([a, b], [6, 5])
"#,
    );
}

#[test]
fn invalid_new_signatures_and_rest_labels_fail_without_weakening_legacy_diagnostics() {
    for (signature, message) in [
        ("...tail, next", "Rest parameter must be last"),
        ("...tail, ...other", "Rest parameter must be last"),
        ("...tail = 1", "Rest parameter cannot have a default"),
        ("...tail = []", "Rest parameter cannot have a default"),
        (
            "...tail: Marker = 1",
            "Rest parameter cannot have a default",
        ),
        ("...", "Expected a parameter but found ')'"),
        (
            "tail, ...tail",
            "Rest parameter 'tail' duplicates a fixed parameter",
        ),
        (". . .tail", "Expected a parameter but found '.'"),
    ] {
        for source in [
            format!("def invalid({signature}) => 1"),
            format!("callback = ({signature}) -> 1"),
            format!("callback = function({signature}) {{ return 1 }}"),
            format!("class Invalid({signature}):\n    pass"),
            format!("class Invalid:\n    def init({signature}) => 1"),
        ] {
            let error = Runtime::check(&source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            assert_eq!(error.message(), message, "{source}");
        }
    }
    for source in [
        "def invalid ...tail => 1",
        "def invalid head, ...tail => 1",
        "def invalid head ...tail => 1",
        "def invalid ...tail:\n    return 1",
        "callback = ...tail => 1",
        "callback = lambda ...tail: 1",
        "collect(...items)",
    ] {
        assert_eq!(
            Runtime::check(source).unwrap_err().kind(),
            ErrorKind::Syntax
        );
    }
    let mut runtime = checked(
        r#"
class Packet(head = 1, ...tail):
    this.tail = tail
def fail() => missing
class Inherited: Packet
    pass
class Caller: Packet
    def retry(...items) => super(tail=fail())
caller = Caller()
"#,
    );
    for (call, message) in [
        (
            "Packet(tail=fail())",
            "Rest parameter 'tail' cannot be a named constructor argument for Packet",
        ),
        (
            "Packet(1, 2, tail=fail())",
            "Rest parameter 'tail' cannot be a named constructor argument for Packet",
        ),
        (
            "Inherited(tail=fail())",
            "Rest parameter 'tail' cannot be a named constructor argument for Packet",
        ),
        (
            "caller.retry()",
            "Rest parameter 'tail' cannot be a named constructor argument for Packet",
        ),
        (
            "Packet(unknown=fail())",
            "Unknown constructor argument 'unknown' for Packet",
        ),
        (
            "Packet(1, head=fail())",
            "Duplicate constructor argument 'head' for Packet",
        ),
        (
            "Packet(head=1, fail())",
            "Positional constructor argument after named arguments for Packet",
        ),
    ] {
        let error = runtime.run(call).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{call}");
        assert_eq!(error.message(), message, "{call}");
        assert!(runtime.state.class("Packet").unwrap().instances.is_empty());
    }
}

#[test]
fn canonical_rest_metadata_round_trips_without_changing_legacy_debug_or_graph_labels() {
    for source in [
        "(head = 1, ...tail: Marker) => [head, tail]",
        "function(...tail) { return tail }",
    ] {
        let expression = parse_expression(source).unwrap();
        let canonical = expression.to_string();
        assert!(canonical.contains("...tail"));
        assert_eq!(
            parse_expression(&canonical).unwrap(),
            expression,
            "{canonical}"
        );
    }
    for source in [
        "def collect(...tail) => tail",
        "class Packet(...tail):\n    this.tail = tail",
        "class Packet:\n    def init(...tail):\n        this.tail = tail",
        "class Packet(head = 1):\n    this.head = head\n    def read(...tail) => tail",
        "class Packet(...tail)",
    ] {
        let statements = parse(source).unwrap();
        let canonical = generate(&statements[0]);
        assert!(canonical.contains("...tail"));
        assert_eq!(parse(&canonical).unwrap(), statements, "{canonical}");
    }
    let legacy = parse("def old(value) => value\nclass Old(value)").unwrap();
    assert_eq!(generate_all(&legacy), "def old;class Old");
    let Stmt::Function(function) = &legacy[0] else {
        panic!()
    };
    assert_eq!(
        format!("{:?}", function.parameters[0]),
        r#"Parameter { name: "value", type_name: None }"#
    );
    let defaulted = parse_expression("(value = 1) => value").unwrap();
    assert_eq!(defaulted.to_string(), "(value=1)=>value");
    assert!(!format!("{defaulted:?}").contains("rest"));
    let Expr::Function(rest) = parse_expression("(...tail) => tail").unwrap() else {
        panic!()
    };
    assert!(rest.parameters[0].rest);
    assert!(format!("{rest:?}").contains("rest: true"));
}

#[test]
fn live_frozen_and_historical_rest_functions_keep_identity_reactivity_and_rollback() {
    let mut runtime = checked(
        r#"
base = 2
def collect(...tail) => base + tail.reduce((sum, item) => sum + item, 0)
live = collect
frozen = collect.value
stored = collect.value
stored = (...tail) => tail.length
old = stored@first
observed = live(1, 2)
assert(observed, 5)
assert(frozen(1, 2), 5)
assert(old(1, 2), 5)
assert(stored(1, 2), 2)
def collect(head = 10, ...tail) => base + head + tail.length
assert(observed, 4)
assert(frozen(1, 2), 5)
base = 4
assert(observed, 6)
assert(old(1, 2), 7)
"#,
    );
    let before = runtime
        .graph
        .nodes()
        .map(|node| (node.key.clone(), node.dependencies.clone()))
        .collect::<Vec<_>>();
    let function = runtime.state.function("collect").unwrap().clone();
    let error = runtime
        .run("def collect(...tail) => missing + tail.length")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert!(Arc::ptr_eq(
        runtime.state.function("collect").unwrap(),
        &function
    ));
    let after = runtime
        .graph
        .nodes()
        .map(|node| (node.key.clone(), node.dependencies.clone()))
        .collect::<Vec<_>>();
    assert_eq!(before, after);
    assert_eq!(runtime.run("observed").unwrap(), Value::Number(6.0));
    assert_eq!(runtime.run("old(1, 2)").unwrap(), Value::Number(7.0));
}

#[test]
fn class_redefinition_and_failed_rest_construction_restore_metadata_state_histories_and_graph() {
    let mut runtime = checked(
        r#"
saved = {amount: 5}
derived = saved.amount * 2
class Packet(...tail):
    this.tail = tail
    def read(...items) => [this.tail, items]
first = Packet(1, 2)
class Packet(head = 3, ...tail):
    this.tail = [head, tail]
    def read(...items) => items
second = Packet(4, 5)
try:
    class Packet(...tail):
        this.tail = []
        def read(...items) => 99
    temporary = Packet(8, 9)
    throw "RESTORE_REST"
catch error:
    assert(error, "RESTORE_REST")
third = Packet()
assert(first.tail, [1, 2])
assert(second.tail, [4, [5]])
assert(third.tail, [3, []])
assert(first.read(6, 7), [6, 7])
assert(Packet.length, 3)
class Failing(...tail):
    saved.amount = tail.length
    this.tail = tail
    throw "ABORT_REST"
"#,
    );
    let class = runtime.state.class("Packet").unwrap().clone();
    assert!(!class.parameters[0].rest);
    assert!(class.parameters[1].rest);
    assert!(class.methods["read"].parameters[0].rest);
    let before = runtime
        .graph
        .nodes()
        .map(|node| (node.key.clone(), node.dependencies.clone()))
        .collect::<Vec<_>>();
    let error = runtime.run("failed = Failing(1, 2, 3)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Thrown);
    assert_eq!(error.message(), "ABORT_REST");
    let after = runtime
        .graph
        .nodes()
        .map(|node| (node.key.clone(), node.dependencies.clone()))
        .collect::<Vec<_>>();
    assert_eq!(before, after);
    assert!(!runtime.state.has_object(&ObjectId::named("failed")));
    assert!(runtime.state.class("Failing").unwrap().instances.is_empty());
    assert!(runtime.state.class("Failing").unwrap().parameters[0].rest);
    assert_eq!(
        runtime.state.class("Packet").unwrap().parameters,
        class.parameters
    );
    assert_eq!(runtime.run("saved.amount").unwrap(), Value::Number(5.0));
    assert_eq!(
        runtime.run("saved.amount@last").unwrap(),
        Value::Number(5.0)
    );
    assert_eq!(runtime.run("derived").unwrap(), Value::Number(10.0));
}
