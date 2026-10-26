//! Per-instance history anchors in ordinary class rules and conditionals.

mod common;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::parse;
use nucleoid::value::ObjectId;
use nucleoid::{ErrorKind, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let assertions = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "class history regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        assertions,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"))
}

fn error(runtime: &mut Runtime, source: &str, kind: ErrorKind, message: &str) {
    let error = runtime.run(source).unwrap_err();
    assert_eq!(error.kind(), kind, "{source}: {error}");
    assert_eq!(error.message(), message, "{source}: {error}");
    assert!(error.position().is_some(), "{source}: source diagnostic");
}

#[test]
fn confirmed_tag_and_shift_uses_each_slots_first_recorded_value_without_repeated_decrements() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Obj(x: int, y: int, orientation: int):
    this.x = x
    this.y = y
    this.orientation = orientation
before = Obj(10, 20, 180)
before.x = 70
before.y = 80
if $Obj.orientation == 180:
    $Obj.tag = "l4214"
    $Obj.x = $Obj.x@first - 1
    $Obj.y = $Obj.y@first - 1
after = Obj(30, 40, 180)
inactive = Obj(50, 60, 0)
assert([before.x, before.y, before.tag], [9, 19, "l4214"])
assert([after.x, after.y, after.tag], [29, 39, "l4214"])
assert([inactive.x, inactive.y, inactive.tag], [50, 60, null])
assert(before.x@first, 10)
assert(before.x@0, 10)
assert(after.x@first, 30)
before.orientation = 0
before.orientation = 180
after.orientation = 180
assert([before.x, before.y], [9, 19])
assert([after.x, after.y], [29, 39])
inactive.x = 90
inactive.orientation = 180
assert([inactive.x, inactive.y, inactive.tag], [49, 59, "l4214"])
before.x = 100
assert(before.x, 9)
assert(before.x@previous, 100)
assert(after.x@2, null)
"#,
    );
    let instance = ObjectId::named("before");
    let conditional = runtime
        .graph
        .nodes()
        .find(|node| {
            node.key.as_str().starts_with("if(") && node.instance.as_ref() == Some(&instance)
        })
        .unwrap();
    for property in ["orientation", "x", "y"] {
        assert!(
            conditional
                .dependencies
                .contains(&NodeKey::property(&instance, property))
        );
    }
}

#[test]
fn ordinary_class_rules_allow_fixed_self_anchors_but_keep_non_self_history_readers_live() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Counter(seed):
    this.x = seed
    this.x = seed + 10
    this.y = seed * 2
before = Counter(10)
step = 1
$Counter.x = $Counter.x@first - step
$Counter.y = $Counter["y"]@0 - step
$Counter.latest = $Counter.x@last
$Counter.prior = $Counter.x@previous
assert([before.x, before.y, before.latest, before.prior], [9, 19, 9, 20])
step = 2
assert([before.x, before.y, before.latest, before.prior], [8, 18, 8, 9])
$Counter.x = $Counter.x@1 - step
after = Counter(30)
assert([before.x, after.x, after.y], [18, 38, 58])
assert([after.x@first, after.x@0, after.x@1], [30, 30, 40])
step = 3
assert([before.x, after.x, before.y, after.y], [17, 37, 17, 57])
assert([before.latest, after.latest, before.prior, after.prior], [17, 37, 18, 38])
"#,
    );
    let instance = ObjectId::named("before");
    let node = runtime
        .graph
        .retrieve(&NodeKey::property(&instance, "x"))
        .unwrap();
    assert!(node.dependencies.contains(&NodeKey::variable("step")));
    assert!(
        !node
            .dependencies
            .contains(&NodeKey::property(&instance, "x"))
    );
}

#[test]
fn missing_self_history_defers_without_seeding_and_recovers_after_slot_recreation() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Pending(orientation):
    this.orientation = orientation
if $Pending.orientation == 180:
    $Pending.tag = "l4214"
    $Pending.x = $Pending.x@first - 1
item = Pending(180)
original = item.x@first
assert(item.tag, "l4214")
assert(item.x, null)
assert(original, null)
"#,
    );
    let instance = ObjectId::named("item");
    assert!(runtime.state.property(&instance, "x").is_none());
    checked(
        &mut runtime,
        r#"
item.x = 10
assert(item.x, 9)
assert(item.x@first, 10)
assert(item.x@previous, 10)
assert(original, 10)
delete item.x
assert(item.x, null)
assert(item.x@first, null)
assert(original, null)
"#,
    );
    assert!(runtime.state.property(&instance, "x").is_none());
    checked(
        &mut runtime,
        r#"
item.x = 40
assert(item.x, 39)
assert(item.x@first, 40)
assert(item.x@0, 40)
assert(item.x@previous, 40)
assert(original, 40)
item.orientation = 180
assert(item.x, 39)
assert(item.x@2, null)
"#,
    );
}

#[test]
fn moving_conditional_history_keeps_accepted_effects_while_first_anchors_stay_stable() {
    let mut runtime = Runtime::new();
    let result = checked(
        &mut runtime,
        r#"
class Counter(seed):
    this.x = seed
    this.enabled = false
if $Counter.enabled:
    $Counter.x = $Counter.x@last - 1
item = Counter(10)
item.enabled = true
assert(item.x, 9)
item.enabled = false
item.enabled = true
assert(item.x, 8)
assert(item.x@first, 10)
assert(item.x@previous, 9)
[item.x, item.x@first, item.x@previous]
"#,
    );
    assert_eq!(
        result,
        Value::List(vec![Value::number(8), Value::number(10), Value::number(9)])
    );

    for selector in ["first", "0"] {
        let mut runtime = Runtime::new();
        checked(
            &mut runtime,
            &format!(
                r#"
class Counter(seed):
    this.x = seed
    this.enabled = false
if $Counter.enabled:
    $Counter.x = $Counter.x@{selector} - 1
item = Counter(10)
item.enabled = true
assert(item.x, 9)
item.enabled = false
item.enabled = true
assert(item.x, 9)
assert(item.x@first, 10)
assert(item.x@previous, 10)
assert(item.x@2, null)
"#
            ),
        );
    }
}

#[test]
fn rules_declared_before_any_instance_and_inherited_receivers_keep_separate_anchors() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Base(seed):
    this.x = seed
    this.orientation = 180
class Child: Base
    def init(seed):
        super(seed)
step = 1
$Base.x = $Base.x@0 - step
if $Base.orientation == 180:
    $Base.tag = "l4214"
    $Base.x = $Base.x@first - step
parent = Base(10)
child = Child(30)
assert([parent.x, child.x], [9, 29])
assert([parent.tag, child.tag], ["l4214", "l4214"])
assert([parent.x@first, child.x@first], [10, 30])
step = 2
assert([parent.x, child.x], [8, 28])
child.orientation = 0
child.orientation = 180
assert(child.x, 28)
assert(child.x@3, null)
"#,
    );
}

#[test]
fn unavailable_fixed_index_waits_for_external_history_not_a_synthetic_self_write() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Sample(seed):
    this.x = seed
    this.enabled = true
if $Sample.enabled:
    $Sample.x = $Sample.x@1 - 1
item = Sample(10)
assert(item.x, 10)
assert(item.x@first, 10)
assert(item.x@1, null)
item.enabled = true
assert(item.x, 10)
assert(item.x@1, null)
item.x = 20
assert(item.x, 19)
assert(item.x@first, 10)
assert(item.x@1, 20)
item.enabled = false
item.enabled = true
assert(item.x, 19)
assert(item.x@3, null)
"#,
    );
}

#[test]
fn failed_anchor_evaluations_unwind_and_pending_rules_keep_other_history_dependencies() {
    for conditional in [false, true] {
        let assignment = |value: &str| {
            if conditional {
                format!("if $Sample.enabled:\n    $Sample.x = {value}")
            } else {
                format!("$Sample.x = {value}")
            }
        };
        let mut runtime = Runtime::new();
        checked(
            &mut runtime,
            r#"
class Sample:
    this.enabled = true
input = {delta: 1}
def fail(item):
    missing = item.x@first
    throw "ANCHOR_FAILURE"
item = Sample()
"#,
        );
        let failed = assignment("fail($Sample)").replace('\n', "\n    ");
        checked(
            &mut runtime,
            &format!("try:\n    {failed}\ncatch error:\n    assert(error, \"ANCHOR_FAILURE\")"),
        );
        assert!(
            runtime
                .state
                .class("Sample")
                .unwrap()
                .declarations
                .is_empty()
        );

        let anchored = assignment("$Sample.x@first - input.delta@last");
        checked(&mut runtime, &anchored);
        checked(
            &mut runtime,
            "assert(item.x, null)\ninput.delta = 2\nassert(item.x, null)\nassert(item.x@first, null)",
        );
        let instance = ObjectId::named("item");
        assert!(runtime.state.property(&instance, "x").is_none());
        let node = if conditional {
            runtime.graph.nodes().find(|node| {
                node.key.as_str().starts_with("if(") && node.instance.as_ref() == Some(&instance)
            })
        } else {
            runtime.graph.retrieve(&NodeKey::property(&instance, "x"))
        }
        .unwrap();
        let Some(Value::Object(input)) = runtime.state.variable("input") else {
            panic!("input must retain its object identity");
        };
        assert!(
            node.dependencies
                .contains(&NodeKey::property(input, "delta"))
        );

        // An instance write replaces an ordinary property rule, so redeclare
        // it before checking recovery in both execution modes.
        checked(&mut runtime, "item.x = 20");
        checked(&mut runtime, &anchored);
        checked(
            &mut runtime,
            "assert(item.x, 18)\nassert(item.x@first, 20)\ninput.delta = 3\nassert(item.x, 17)\nassert(item.x@previous, 18)\nassert(item.x@3, null)",
        );
    }
}

#[test]
fn missing_guard_and_missing_ordinary_class_self_rule_do_not_create_history() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Empty
step = 1
$Empty.x = $Empty.x@first - step
if $Empty.orientation == 180:
    $Empty.y = $Empty.y@first - step
item = Empty()
assert(item.x, null)
assert(item.x@first, null)
assert(item.y, null)
assert(item.y@first, null)
step = 2
assert(item.x@first, null)
item.y = 20
assert(item.y, 20)
item.orientation = 180
assert(item.y, 18)
assert(item.y@first, 20)
item.orientation = 0
item.orientation = 180
assert(item.y, 18)
"#,
    );
    assert!(
        runtime
            .state
            .property(&ObjectId::named("item"), "x")
            .is_none()
    );
    // An explicit instance assignment still overrides its property rule.
    checked(
        &mut runtime,
        "item.x = 40\nstep = 3\nassert(item.x, 40)\n$Empty.x = $Empty.x@first - step\nassert(item.x, 37)",
    );
}

#[test]
fn null_first_values_and_per_instance_value_snapshots_keep_their_accepted_meanings() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Sample(seed):
    this.x = seed
    this.enabled = false
$Sample.saved = $Sample.x@first
$Sample.frozen = $Sample.x@last.value
one = Sample(10)
two = Sample(30)
empty = Sample(null)
one.x = 20
empty.x = 50
if $Sample.enabled:
    $Sample.x = $Sample.x@first - 1
one.enabled = true
two.enabled = true
empty.enabled = true
assert([one.x, two.x, empty.x], [9, 29, null])
assert([one.saved, two.saved, empty.saved], [10, 30, null])
assert([one.frozen, two.frozen, empty.frozen], [10, 30, null])
assert(one.x@previous, 20)
assert(empty.x@first, null)
assert(empty.x@previous, 50)
one.enabled = true
assert(one.x, 9)
"#,
    );
}

#[test]
fn explicit_self_snapshots_stay_detached_and_missing_snapshots_keep_their_error() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Sample(seed):
    this.x = seed
step = 1
$Sample.x = $Sample.x@last.value - step
one = Sample(10)
two = Sample(30)
assert([one.x, two.x], [9, 29])
step = 2
assert([one.x, two.x], [8, 28])
assert([one.x@first, two.x@first], [10, 30])
class Empty
$Empty.x = $Empty.x@first.value
"#,
    );
    error(
        &mut runtime,
        "missing = Empty()",
        ErrorKind::Reference,
        "$Empty.x@first is not defined",
    );
    assert!(!runtime.state.has_object(&ObjectId::named("missing")));
}

#[test]
fn helpers_and_computed_receivers_preserve_fixed_and_moving_history_semantics() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Sample(seed):
    this.x = seed
    this.enabled = true
    this.key = "x"
def original(item) => item[item.key]@first
if $Sample.enabled:
    $Sample.x = original($Sample) - 1
one = Sample(10)
two = Sample(30)
assert([one.x, two.x], [9, 29])
one.enabled = true
assert(one.x, 9)
"#,
    );
    for selector in ["last", "previous"] {
        let mut runtime = Runtime::new();
        let first = if selector == "last" { 19 } else { 9 };
        let second = if selector == "last" { 18 } else { 19 };
        checked(
            &mut runtime,
            &format!(
                r#"
class Counter(seed):
    this.x = seed
    this.x = seed + 10
    this.enabled = false
    this.key = "x"
def receiver(item) => item
def moving(item) => receiver(item)[item.key]@{selector}
if $Counter.enabled:
    $Counter.x = moving($Counter) - 1
item = Counter(10)
item.enabled = true
assert(item.x, {first})
item.enabled = false
item.enabled = true
assert(item.x, {second})
assert(item.x@first, 10)
assert(item.x@1, 20)
assert(item.x@2, {first})
assert(item.x@previous, {first})
assert(item.x@3, {second})
assert(item.x@4, null)
"#
            ),
        );
    }
}

#[test]
fn live_class_cycles_and_non_self_history_cycles_still_fail_before_instances_exist() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, "class Sample\nstep = 1");
    for source in [
        "$Sample.x = $Sample.x + 1",
        "$Sample.x = $Sample.x@last - 1",
        "$Sample.x = $Sample.x@previous - 1",
        "$Sample.x = $Sample.x@first + $Sample.x",
        "$Sample.x += $Sample.x@first",
        "$Sample.x = $Sample.y@first\n$Sample.y = $Sample.x + 1",
        "$Sample.x = $Sample.x@first + $Sample.y\n$Sample.y = $Sample.x",
        "$Sample.x = [$Sample.x@first, $Sample.x@last]",
    ] {
        error(&mut runtime, source, ErrorKind::Type, "Circular Dependency");
        assert!(
            runtime
                .state
                .class("Sample")
                .unwrap()
                .declarations
                .is_empty()
        );
    }
    checked(
        &mut runtime,
        "$Sample.x = $Sample.x@first - step\nitem = Sample()\nassert(item.x@first, null)",
    );
}

#[test]
fn ordinary_variable_object_and_non_self_class_history_forms_are_unchanged() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
value = 1
value = value@last + 1
record = {x: 10}
record.x = record.x@last + 1
assert(value, 2)
assert(record.x, 11)
class Sample(seed):
    this.x = seed
$Sample.first = $Sample.x@first
$Sample.second = $Sample.x@1
$Sample.latest = $Sample.x@last
$Sample.prior = $Sample.x@previous
item = Sample(20)
assert([item.first, item.second, item.latest, item.prior], [20, null, 20, null])
item.x = 40
assert([item.first, item.second, item.latest, item.prior], [20, 40, 40, 20])
delete item.x
assert([item.first, item.second, item.latest, item.prior], [null, null, null, null])
item.x = 60
assert([item.first, item.second, item.latest, item.prior], [60, null, 60, null])
"#,
    );
}

#[test]
fn failed_runs_restore_class_rules_graph_and_histories_before_later_anchor_recovery() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Sample(seed):
    this.x = seed
    this.enabled = true
step = 1
$Sample.x = $Sample.x@first - step
item = Sample(10)
assert(item.x, 9)
"#,
    );
    let key = NodeKey::property(&ObjectId::named("item"), "x");
    let dependencies = runtime.graph.retrieve(&key).unwrap().dependencies.clone();
    error(
        &mut runtime,
        "step = 5\n$Sample.x = $Sample.x@first - step * 2\nfuture = Sample(30)\nmissing",
        ErrorKind::Reference,
        "missing is not defined",
    );
    assert!(!runtime.state.has_variable("future"));
    assert!(!runtime.state.has_object(&ObjectId::named("future")));
    assert_eq!(
        runtime.graph.retrieve(&key).unwrap().dependencies,
        dependencies
    );
    checked(
        &mut runtime,
        "assert(step, 1)\nassert(item.x, 9)\nassert(item.x@first, 10)\nassert(item.x@2, null)\nstep = 2\nassert(item.x, 8)\nnext = Sample(40)\nassert(next.x, 38)",
    );
}

#[test]
fn caught_failures_restore_deleted_slots_rule_templates_and_anchor_observers() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Sample(seed):
    this.x = seed
    this.enabled = true
if $Sample.enabled:
    $Sample.x = $Sample.x@first - 1
item = Sample(10)
original = item.x@first
try:
    delete item.x
    item.x = 50
    $Sample.extra = $Sample.x@first
    throw "RESTORE_ANCHOR"
catch error:
    assert(error, "RESTORE_ANCHOR")
assert(item.x, 9)
assert(item.x@first, 10)
assert(item.x@previous, 10)
assert(item.x@2, null)
assert(item.extra, null)
assert(original, 10)
try:
    $Sample.x = $Sample.x@last - 1
catch error:
    assert(error, TypeError("Circular Dependency"))
item.enabled = true
assert(item.x, 9)
delete item.x
item.x = 30
assert(item.x, 29)
assert(item.x@first, 30)
assert(original, 30)
"#,
    );
    assert_eq!(runtime.state.class("Sample").unwrap().declarations.len(), 1);
}

#[test]
fn previous_history_in_a_predeclared_conditional_preserves_values_and_history_order() {
    let mut runtime = Runtime::new();
    let result = checked(
        &mut runtime,
        r#"
class Sample(seed):
    this.x = seed
    this.x = seed + 10
    this.enabled = false
if $Sample.enabled:
    $Sample.x = $Sample.x@previous - 1
item = Sample(10)
assert(item.x, 20)
assert(item.x@previous, 10)
item.enabled = true
assert(item.x, 9)
assert(item.x@previous, 20)
item.enabled = false
item.enabled = true
assert(item.x, 19)
assert(item.x@first, 10)
assert(item.x@1, 20)
assert(item.x@2, 9)
assert(item.x@previous, 9)
assert(item.x@3, 19)
assert(item.x@4, null)
[item.x, item.x@first, item.x@previous]
"#,
    );
    assert_eq!(
        result,
        Value::List(vec![Value::number(19), Value::number(10), Value::number(9)])
    );
    checked(
        &mut runtime,
        r#"
class Missing(seed):
    this.x = seed
    this.enabled = false
if $Missing.enabled:
    $Missing.x = $Missing.x@previous - 1
missing = Missing(10)
missing.enabled = true
assert(missing.x, null)
assert(missing.x@first, 10)
assert(missing.x@previous, 10)
missing.enabled = false
missing.enabled = true
assert(missing.x, 9)
assert(missing.x@previous, null)
assert(missing.x@2, 9)
"#,
    );
}

#[test]
fn class_history_destinations_remain_read_only_and_failed_writes_leave_no_rule() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Sample:
    this.x = 10
    this.items = [1, 2]
    this.enabled = true
item = Sample()
"#,
    );
    for source in [
        "$Sample.x@first = 4",
        "$Sample.x@0 += 4",
        "delete $Sample.x@first",
        "if $Sample.enabled:\n    $Sample.items@first.push(3)",
        "if $Sample.enabled:\n    $Sample.x@first = 4",
    ] {
        error(
            &mut runtime,
            source,
            ErrorKind::Type,
            "State versions are read-only",
        );
        checked(
            &mut runtime,
            "assert(item.x, 10)\nassert(item.x@first, 10)\nassert(item.x@1, null)\nassert(item.items, [1, 2])\nassert(item.items@1, null)",
        );
        assert!(
            runtime
                .state
                .class("Sample")
                .unwrap()
                .declarations
                .is_empty()
        );
    }
}
