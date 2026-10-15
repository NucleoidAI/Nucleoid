//! Ordered cubic easing through the unchanged static Math call path.
//!
//! Only normative documents and focused handwritten fixtures are executed.
//! These tests need neither Git history nor external proposal/session inputs.

mod common;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Ast, Stmt, parse, parse_expression};
use nucleoid::lang::evaluation::Flow;
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value, statement};

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let protected = common::assertions::protected_assertions(&parse(source).unwrap());
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "Math.smoothstep regression".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        },
        protected.len(),
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, protected.len());
    value
}

fn run(source: &str) -> Runtime {
    let mut runtime = Runtime::new();
    checked(&mut runtime, source, None);
    runtime
}

fn pair(template: &str, candidate: &str, accepted: &str) -> [Runtime; 2] {
    let sources = [
        template.replace("__SMOOTHSTEP__", candidate),
        template.replace("__SMOOTHSTEP__", accepted),
    ];
    assert_ne!(sources[0], sources[1], "the controls must be distinct");
    let statements = sources.each_ref().map(|source| parse(source).unwrap());
    assert_ne!(statements[0], statements[1]);
    let protected = statements
        .each_ref()
        .map(|statements| common::assertions::protected_assertions(statements));
    assert_eq!(protected[0], protected[1]);
    sources.each_ref().map(|source| run(source))
}

fn assert_dependencies(runtime: &Runtime, key: NodeKey, expected: &[&str]) {
    let node = runtime.graph.retrieve(&key).unwrap();
    assert_eq!(
        node.dependencies
            .iter()
            .map(NodeKey::as_str)
            .collect::<Vec<_>>(),
        expected
    );
}

fn assert_numeric_parity(left: &Runtime, right: &Runtime, name: &str) {
    let (Some(Value::Number(left)), Some(Value::Number(right))) =
        (left.state.variable(name), right.state.variable(name))
    else {
        panic!("{name} must be numeric in both controls");
    };
    if right.is_nan() {
        assert!(left.is_nan(), "{name}: {left} is not NaN");
    } else {
        assert_eq!(left.to_bits(), right.to_bits(), "{name}");
    }
}

#[test]
fn three_live_inputs_and_ignored_extras_keep_argument_order_and_transitive_behavior() {
    let mut runtimes = pair(
        r#"
lower = 0
upper = 8
value = 2
extra = 99
unit = Math.saturate((value - lower) / (upper - lower))
eased = __SMOOTHSTEP__
assert(unit, 0.25)
assert(eased, 0.15625)
value = 4
assert(eased, 0.5)
upper = 16
assert(eased, 0.15625)
lower = -8
assert(eased, 0.5)
extra = Number.NaN
assert(eased, 0.5)
value = -16
assert(eased, 0)
value = 24
assert(eased, 1)
upper = -32
assert(eased, 0)
value = -20
assert(eased, 0.5)
"#,
        "Math.smoothstep(lower, upper, value, extra)",
        "Math.saturate(unit * unit * (3 - 2 * unit), extra)",
    );
    for (runtime, dependencies) in runtimes.iter_mut().zip([
        ["lower", "upper", "value", "extra"].as_slice(),
        ["unit", "extra"].as_slice(),
    ]) {
        assert_dependencies(runtime, NodeKey::variable("eased"), dependencies);
        let trace = runtime.run("why eased").unwrap();
        let trace: serde_json::Value =
            serde_json::from_str(&runtime.serialize_json(&trace).unwrap()).unwrap();
        assert_eq!(trace[0]["from"], serde_json::json!(dependencies));
        assert_eq!(trace[0]["holds"], 0.5);
        for input in ["lower", "upper", "value", "extra"] {
            assert!(
                runtime
                    .graph
                    .reaches(&NodeKey::variable(input), &NodeKey::variable("eased")),
                "{input}"
            );
        }
    }
}

#[test]
fn intrinsic_coercion_and_lower_upper_value_order_cover_language_values() {
    run(r#"
assert(Math.smoothstep(0, 8, 2), 0.15625)
assert(Math.smoothstep(" 0 ", "8", "2"), 0.15625)
assert(Math.smoothstep(false, true, true), 1)
assert(Math.smoothstep(null, 1, 0.5), 0.5)
assert(Math.smoothstep(0, null, 1), 1)
assert(Math.smoothstep(0, 1, null), 0)
assert(Math.smoothstep("", "1", "0.5"), 0.5)
assert(Math.smoothstep([], ["8"], ["6"]), 0.84375)
assert(Math.smoothstep(Date(0), Date(8), Date(4)), 0.5)
assert(Number.isNaN(Math.smoothstep("invalid", 1, 0.5)), true)
assert(Number.isNaN(Math.smoothstep(0, "invalid", 0.5)), true)
assert(Number.isNaN(Math.smoothstep(0, 1, "invalid")), true)
assert(Number.isNaN(Math.smoothstep([0, 1], 1, 0.5)), true)
assert(Number.isNaN(Math.smoothstep(0, {}, 0.5)), true)
assert(Number.isNaN(Math.smoothstep(0, 1, /probe/)), true)
assert(Number.isNaN(Math.smoothstep(amount => amount, 1, 0.5)), true)
class Probe
assert(Number.isNaN(Math.smoothstep(0, Probe, 0.5)), true)
assert(Number.isNaN(Math.smoothstep(0, 1, Probe())), true)
"#);
}

#[test]
fn omission_nonfinite_equal_reversed_and_overflow_results_have_no_shortcuts() {
    run(r#"
assert(Number.isNaN(Math.smoothstep()), true)
assert(Number.isNaN(Math.smoothstep(0)), true)
assert(Number.isNaN(Math.smoothstep(0, 1)), true)
assert(Math.smoothstep(0, 1, 0.5, 99), 0.5)
assert(Math.smoothstep(0, 1, 0.5, Number.NaN, "invalid", {}, null), 0.5)
assert(Math.smoothstep(0, 1, 0.5, [0, 1], Date(99), /probe/, amount => amount), 0.5)
assert(Math.smoothstep(8, 0, 2), 0.84375)
assert(Math.smoothstep(2, 2, 1), 0)
assert(Math.smoothstep(2, 2, 3), 1)
assert(Number.isNaN(Math.smoothstep(2, 2, 2)), true)
assert(Math.smoothstep(0, 1, Number.POSITIVE_INFINITY), 1)
assert(Math.smoothstep(0, 1, Number.NEGATIVE_INFINITY), 0)
assert(Math.smoothstep(0, Number.POSITIVE_INFINITY, 1), 0)
assert(Number.isNaN(Math.smoothstep(0, Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY)), true)
assert(Number.isNaN(Math.smoothstep(Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY, 0)), true)
assert(Number.isNaN(Math.smoothstep(Number.NaN, 1, 0.5)), true)
assert(Number.isNaN(Math.smoothstep(0, Number.NaN, 0.5)), true)
assert(Number.isNaN(Math.smoothstep(0, 1, Number.NaN)), true)
assert(Math.smoothstep(-Number.MAX_VALUE, Number.MAX_VALUE, 0), 0)
assert(Number.isNaN(Math.smoothstep(-Number.MAX_VALUE, Number.MAX_VALUE, Number.MAX_VALUE)), true)
"#);
}

#[test]
fn boundary_subnormal_zero_and_repeated_nan_updates_match_accepted_composition_bits() {
    let mut runtimes = pair(
        r#"
lower = 0
upper = 1
value = -0
unit = Math.saturate((value - lower) / (upper - lower))
eased = __SMOOTHSTEP__
reciprocal = 1 / eased
assert(reciprocal, 1 / (unit * unit * (3 - 2 * unit)))
"#,
        "Math.smoothstep(lower, upper, value)",
        "unit * unit * (3 - 2 * unit)",
    );
    for source in [
        "value = 0",
        "value = -0",
        "lower = -0",
        "value = Number.MIN_VALUE / 2",
        "value = -Number.MIN_VALUE / 2",
        "value = 0.5 - Number.EPSILON / 4",
        "value = 0.5 + Number.EPSILON / 2",
        "value = 1 - Number.EPSILON / 2",
        "value = 1",
        "value = 1 + Number.EPSILON",
        "value = Number.POSITIVE_INFINITY",
        "value = Number.NEGATIVE_INFINITY",
        "value = Number.NaN",
        "value = Number.NaN",
        "value = 0.25",
        "lower = 1\nupper = 0\nvalue = 0.25",
        "lower = 2\nupper = 2\nvalue = 2",
        "value = 1",
        "value = 3",
        "lower = -Number.MAX_VALUE\nupper = Number.MAX_VALUE\nvalue = 0",
        "value = Number.MAX_VALUE",
        "lower = 0\nupper = 8\nvalue = 2\nassert(eased, 0.15625)",
    ] {
        for runtime in &mut runtimes {
            checked(runtime, source, None);
        }
        for name in ["unit", "eased", "reciprocal"] {
            assert_numeric_parity(&runtimes[0], &runtimes[1], name);
        }
    }
}

#[test]
fn every_expression_and_callback_runs_once_left_to_right_including_ignored_extras() {
    run(r#"
class Log
log = Log()
log.order = 0
eased = Math.smoothstep(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3,
    log.order = log.order.value * 10 + 4,
    log.order = log.order.value * 10 + 5
)
assert(log.order, 12345)
assert(eased, 1)
invalid = Math.smoothstep(Number.NaN, 1, 0.5, log.order = 8)
assert(Number.isNaN(invalid), true)
assert(log.order, 8)
deferred = Math.smoothstep(log.lower, 1, 0.5, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.lower = 0
assert(deferred, 0.5)
assert(log.order, 7)
log.order = 0
mapped = [1, 2, 3].map(amount => Math.smoothstep(0, 4, amount, log.order = log.order.value * 10 + amount))
assert(mapped, [0.15625, 0.5, 0.84375])
assert(log.order, 123)
"#);
}

#[test]
fn supplied_undefined_in_each_input_and_extra_recovers_after_creation_and_deletion() {
    let runtime = run(r#"
class Sensor
sensor = Sensor()
first = Math.smoothstep(sensor.lower, 8, 2)
second = Math.smoothstep(0, sensor.upper, 2)
third = Math.smoothstep(0, 8, sensor.sample)
fourth = Math.smoothstep(0, 8, 2, sensor.extra)
fifth = Math.smoothstep(0, 8, 2, 99, sensor.extra)
assert(first, null)
assert(second, null)
assert(third, null)
assert(fourth, null)
assert(fifth, null)
sensor.lower = 0
sensor.upper = 8
sensor.sample = 2
sensor.extra = Number.NaN
assert(first, 0.15625)
assert(second, 0.15625)
assert(third, 0.15625)
assert(fourth, 0.15625)
assert(fifth, 0.15625)
delete sensor.lower
delete sensor.upper
delete sensor.sample
delete sensor.extra
assert(first, null)
assert(second, null)
assert(third, null)
assert(fourth, null)
assert(fifth, null)
sensor.lower = 0
sensor.upper = 8
sensor.sample = 2
sensor.extra = {}
assert(first, 0.15625)
assert(second, 0.15625)
assert(third, 0.15625)
assert(fourth, 0.15625)
assert(fifth, 0.15625)
"#);
    for (name, property) in [
        ("first", "lower"),
        ("second", "upper"),
        ("third", "sample"),
        ("fourth", "extra"),
        ("fifth", "extra"),
    ] {
        assert!(
            runtime
                .graph
                .retrieve(&NodeKey::variable(name))
                .unwrap()
                .dependencies
                .contains(&NodeKey::property(&ObjectId::named("sensor"), property))
        );
    }
}

#[test]
fn literal_null_conversion_does_not_override_live_assignment_and_parameter_contexts() {
    let mut runtimes = pair(
        r#"
lower = 0
upper = 8
value = 2
unit = Math.saturate((value - lower) / (upper - lower))
eased = __SMOOTHSTEP__
assert(eased, 0.15625)
lower = null
assert(eased, null)
lower = 0
assert(eased, 0.15625)
upper = null
assert(eased, null)
upper = 8
assert(eased, 0.15625)
value = null
assert(eased, null)
value = 6
assert(eased, 0.84375)
"#,
        "Math.smoothstep(lower, upper, value)",
        "unit * unit * (3 - 2 * unit)",
    );
    for runtime in &mut runtimes {
        checked(
            runtime,
            r#"
assert(Math.smoothstep(null, 1, 0.5), 0.5)
assert(Math.smoothstep(0, 1, null), 0)
assert(Math.smoothstep(0, 1, 0.5, null), 0.5)
extra = null
observed = Math.smoothstep(0, 1, 0.5, extra)
assert(observed, null)
extra = 99
assert(observed, 0.5)
extra = null
assert(observed, null)
extra = 99
assert(observed, 0.5)
def smooth lower: Bound, upper: Bound, value: Input => Math.smoothstep(lower, upper, value)
def control lower: Bound, upper: Bound, value: Input => Math.saturate((value - lower) / (upper - lower)) * Math.saturate((value - lower) / (upper - lower)) * (3 - 2 * Math.saturate((value - lower) / (upper - lower)))
missing = smooth()
old_missing = control()
fewer = smooth(0, 1)
old_fewer = control(0, 1)
parameter = smooth(null, 1, 0.5)
old_parameter = control(null, 1, 0.5)
assert(missing, null)
assert(old_missing, null)
assert(fewer, null)
assert(old_fewer, null)
assert(parameter, null)
assert(old_parameter, null)
assert(smooth(0, 8, 2, 99), 0.15625)
assert(control(0, 8, 2, 99), 0.15625)
"#,
            None,
        );
    }
}

#[test]
fn strict_later_arguments_keep_typed_positions_rollback_and_no_phantom_graphs() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 7
lower = 0
upper = 8
value = 2
eased = Math.smoothstep(lower, upper, value)
try:
    eased = Math.smoothstep(log.order = 9, upper, value, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 7)
assert(eased, 0.15625)
try:
    Math.smoothstep(log.absent, upper, value, missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
try:
    Math.smoothstep(lower, upper, value, Math.random())
catch error:
    assert(error, TypeError("Math.random is not supported"))
try:
    Math.smoothstep(lower, upper, value, log.absent, Math.random())
catch error:
    assert(error, TypeError("Math.random is not supported"))
"#);
    for source in [
        "Math.smoothstep(missing, 8, 2)",
        "Math.smoothstep(0, missing, 2)",
        "Math.smoothstep(0, 8, missing)",
        "Math.smoothstep(0, 8, 2, missing)",
        "Math.smoothstep(log.absent, 8, 2, missing)",
    ] {
        let keys = runtime.graph.keys().cloned().collect::<Vec<_>>();
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.graph.keys().cloned().collect::<Vec<_>>(), keys);
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
    let error = runtime
        .run("log.order = 9\nMath.smoothstep(0, 8, 2, missing)")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    checked(
        &mut runtime,
        "assert(log.order, 7)\nassert(eased, 0.15625)\nvalue = 6\nassert(eased, 0.84375)\neased",
        Some("0.84375"),
    );
    assert_dependencies(
        &runtime,
        NodeKey::variable("eased"),
        &["lower", "upper", "value"],
    );
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
}

#[test]
fn class_rules_methods_and_initializers_keep_instances_deletion_and_failed_rule_rollback() {
    let runtimes = pair(
        r#"
lower = 0
upper = 8
class Curve(sample):
    this.sample = sample
    def reading lower, upper => Math.smoothstep(lower, upper, self.sample)
$Curve.unit = Math.saturate(($Curve.sample - lower) / (upper - lower))
$Curve.eased = __SMOOTHSTEP__
first = Curve(2)
second = Curve(6)
assert(first.eased, 0.15625)
assert(second.eased, 0.84375)
assert(first.reading(0, 8), 0.15625)
first.sample = 4
assert(first.eased, 0.5)
upper = 16
assert(first.eased, 0.15625)
assert(second.eased, 0.31640625)
delete first.sample
assert(first.eased, null)
assert(second.eased, 0.31640625)
first.sample = 8
assert(first.eased, 0.5)
try:
    $Curve.eased = Math.smoothstep(missing, upper, $Curve.sample)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(first.eased, 0.5)
first.sample = 4
assert(first.eased, 0.15625)
class Initial:
    def init sample => this.eased = Math.smoothstep(0, 8, sample)
initialized = Initial(2)
assert(initialized.eased, 0.15625)
assert(initialized.id, "initialized")
"#,
        "Math.smoothstep(lower, upper, $Curve.sample)",
        "$Curve.unit * $Curve.unit * (3 - 2 * $Curve.unit)",
    );
    for (index, runtime) in runtimes.iter().enumerate() {
        for (object, other) in [("first", "second"), ("second", "first")] {
            let dependencies = &runtime
                .graph
                .retrieve(&NodeKey::property(&ObjectId::named(object), "eased"))
                .unwrap()
                .dependencies;
            let own = if index == 0 { "sample" } else { "unit" };
            assert!(dependencies.contains(&NodeKey::property(&ObjectId::named(object), own)));
            assert!(!dependencies.contains(&NodeKey::property(&ObjectId::named(other), own)));
        }
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
}

#[test]
fn intrinsic_conversion_does_not_invoke_or_track_shadowed_number_or_saturate() {
    let runtime = run(r#"
def Number(value) => 999
assert(Number("2"), 999)
text = "2"
eased = Math.smoothstep("0", "8", text)
assert(eased, 0.15625)
text = "6"
assert(eased, 0.84375)
def numeric(Number):
    return Math.smoothstep("0", "8", "4")
assert(numeric(null), 0.5)
"#);
    assert_dependencies(&runtime, NodeKey::variable("eased"), &["text"]);
    let runtime = run(r#"
Number = 7
eased = Math.smoothstep("0", "8", "2")
assert(eased, 0.15625)
def saturate(value) => 999
assert(saturate(0.25), 999)
assert(Math.saturate(0.25), 0.25)
assert(Math.smoothstep("0", "8", "2"), 0.15625)
class Namespace:
    def saturate(value) => 999
namespace = Namespace()
assert(namespace.saturate(0.25), 999)
def numeric(saturate):
    return Math.smoothstep("0", "8", "2")
assert(numeric(value => 999), 0.15625)
"#);
    assert_dependencies(&runtime, NodeKey::variable("eased"), &[]);
    assert!(!runtime.graph.contains(&NodeKey::variable("Math.saturate")));
}

#[test]
fn math_shadowing_ordinary_members_and_unsupported_method_reads_retain_lookup() {
    run(r#"
assert(Math.smoothstep(0, 8, 2), 0.15625)
reference = Math.smoothstep
old_reference = Math.saturate
indirect = reference(0, 8, 2)
assert(reference, null)
assert(old_reference, null)
assert(indirect, null)
class Namespace:
    def smoothstep(lower, upper, value):
        return lower + upper + value
    def saturate(value) => 999
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
holder.namespace = namespace
assert(holder.Math(2), 6)
assert(holder.namespace.smoothstep(2, 3, 4), 9)
holder.Math = namespace
assert(holder.Math.smoothstep(2, 3, 4), 9)
assert(holder.Math.saturate(0.5), 999)
def scoped(Math):
    return Math.smoothstep(2, 3, 4)
assert(scoped(namespace), 9)
assert(scoped(null), null)
assert(Math.smoothstep(0, 8, 2), 0.15625)
Math = namespace
assert(Math.saturate(0.5), 999)
assert(Math.smoothstep(2, 3, 4), 9)
"#);
    run(r#"
Math = 7
try:
    Math.smoothstep(0, 8, 2)
catch error:
    assert(error, TypeError("number.smoothstep is not a function"))
"#);
}

#[test]
fn typed_wrappers_live_reads_noncallable_arguments_and_failed_redefinitions_recover() {
    let runtimes = pair(
        r#"
lower = 0
upper = 8
input = 2
extra = 99
def ease amount: ArbitraryType => __SMOOTHSTEP__
eased = ease(input)
assert(eased, 0.15625)
input = 6
assert(eased, 0.84375)
upper = 16
assert(eased, 0.31640625)
extra = 12
assert(eased, 0.31640625)
try:
    def ease amount: ArbitraryType => Math.smoothstep(missing, upper, amount)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(eased, 0.31640625)
input = 8
assert(eased, 0.5)
assert(ease(4), 0.15625)
def invoke(operation, amount):
    return operation(amount)
assert(invoke(ease, 4), 0.15625)
assert(invoke(null, 4), null)
try:
    invoke(7, 4)
catch error:
    assert(error, TypeError("operation is not a function"))
"#,
        "Math.smoothstep(lower, upper, amount, extra)",
        "Math.saturate(Math.saturate((amount - lower) / (upper - lower)) * Math.saturate((amount - lower) / (upper - lower)) * (3 - 2 * Math.saturate((amount - lower) / (upper - lower))), extra)",
    );
    for runtime in runtimes {
        let node = runtime.graph.retrieve(&NodeKey::variable("eased")).unwrap();
        for key in ["ease", "input", "lower", "upper", "extra"] {
            assert!(node.dependencies.contains(&NodeKey::variable(key)), "{key}");
        }
        assert!(!runtime.graph.contains(&NodeKey::variable("ArbitraryType")));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
}

#[test]
fn inherited_noncapture_and_parameter_dependent_local_failures_remain_paired() {
    let accepted = "Math.saturate((value - lower) / (upper - lower)) * Math.saturate((value - lower) / (upper - lower)) * (3 - 2 * Math.saturate((value - lower) / (upper - lower)))";
    let template = "def factory(lower, upper) => value => __SMOOTHSTEP__\ncallback = factory(0, 1)\ncallback(0.5)";
    let sources = [
        template.replace("__SMOOTHSTEP__", "Math.smoothstep(lower, upper, value)"),
        template.replace("__SMOOTHSTEP__", accepted),
    ];
    assert_ne!(sources[0], sources[1]);
    let errors = sources
        .each_ref()
        .map(|source| Runtime::new().run(source).unwrap_err());
    assert_eq!(errors[0], errors[1]);
    assert_eq!(errors[0].kind(), ErrorKind::Reference);
    assert_eq!(errors[0].message(), "lower is not defined");
    assert_eq!(errors[0].position(), Some(Position::new(3, 1)));

    let mut runtimes = pair(
        "lower = 0\nupper = 8\ndef ease(value):\n    temporary = __SMOOTHSTEP__\n    return temporary\nresult = ease(2)\nassert(result, 0.15625)",
        "Math.smoothstep(lower, upper, value)",
        accepted,
    );
    for runtime in &mut runtimes {
        let error = runtime.run("upper = 16").unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "value is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        checked(runtime, "assert(upper, 8)\nassert(result, 0.15625)", None);
        assert!(!runtime.graph.contains(&NodeKey::variable("value")));
    }
}

#[test]
fn application_rule_failure_rolls_back_the_input_and_preserves_later_propagation() {
    let runtimes = pair(
        r#"
lower = 0
upper = 8
value = 2
unit = Math.saturate((value - lower) / (upper - lower))
eased = __SMOOTHSTEP__
if eased > 0.8:
    throw "TOO_HIGH"
try:
    value = 6
catch error:
    assert(error, "TOO_HIGH")
assert(value, 2)
assert(unit, 0.25)
assert(eased, 0.15625)
value = 4
assert(eased, 0.5)
"#,
        "Math.smoothstep(lower, upper, value)",
        "unit * unit * (3 - 2 * unit)",
    );
    assert_dependencies(
        &runtimes[0],
        NodeKey::variable("eased"),
        &["lower", "upper", "value"],
    );
    assert_dependencies(&runtimes[1], NodeKey::variable("eased"), &["unit"]);
}

#[test]
fn existing_math_functions_constants_and_exact_errors_remain_available() {
    run(r#"
assert(Math.clamp(12, 0, 10), 10)
assert(Math.clamp(5, 10, 0), 0)
assert(Math.lerp(2, 10, 0.5), 6)
assert(Math.saturate(0.25), 0.25)
assert(Math.max(), Number.NEGATIVE_INFINITY)
assert(Math.min(), Number.POSITIVE_INFINITY)
assert(Number.isNaN(Math.max(1, Number.NaN, 2)), true)
assert(Number.isNaN(Math.min(1, Number.NaN, 2)), true)
assert(Math.round(0.49999999999999994), 0)
assert(1 / Math.round(-0.5), Number.NEGATIVE_INFINITY)
assert(Math.PI > 3, true)
assert(Math.E > 2, true)
try:
    Math.unknown()
catch error:
    assert(error, TypeError("Math.unknown is not a function"))
try:
    Math.random()
catch error:
    assert(error, TypeError("Math.random is not supported"))
class Sensor
sensor = Sensor()
unknown = Math.unknown(sensor.absent)
random = Math.random(sensor.absent)
assert(unknown, null)
assert(random, null)
"#);
}

#[test]
fn ordinary_source_call_and_node_apis_preserve_caller_scope_frames_on_success_and_error() {
    let source = "Math.smoothstep(lower, upper, value)";
    Runtime::check(source).unwrap();
    let program = statement::compile(source).unwrap();
    assert_eq!(program.statements, parse(source).unwrap());
    assert_eq!(program.positions, [Position::new(1, 1)]);
    let [Stmt::Expression(expression)] = program.statements.as_slice() else {
        panic!("the intrinsic must remain an ordinary call");
    };
    let Ast::Call(call) = Ast::convert(expression) else {
        panic!("no special AST representation is needed");
    };
    assert_eq!(
        call.function().unwrap().path().as_deref(),
        Some("Math.smoothstep")
    );
    assert_eq!(call.arguments().len(), 3);
    assert_eq!(
        Ast::convert(expression).generate(),
        "Math.smoothstep(lower,upper,value)"
    );
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.push();
    scope.declare("lower", Value::number(0));
    scope.declare("upper", Value::number(8));
    scope.declare("value", Value::number(2));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    assert_eq!(
        call.resolve(&mut runtime, &mut scope).unwrap(),
        Value::number(0.15625)
    );
    let mut node = Nuc::convert(&mut runtime, &mut scope, &program.statements[0]).unwrap();
    node.before(&mut runtime, &mut scope).unwrap();
    let outcome = node.run(&mut runtime, &mut scope).unwrap();
    assert!(matches!(outcome.flow, Flow::Normal(Value::Number(0.15625))));
    node.graph(&mut runtime, outcome.dependencies).unwrap();
    node.after(&mut runtime).unwrap();
    let failed = parse_expression("Math.smoothstep(lower, upper, value, missing)").unwrap();
    let error = Ast::convert(&failed)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.retrieve("value"), Some(&Value::number(2)));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    checked(
        &mut runtime,
        "input = 2\noutput = Math.smoothstep(0, 8, input)\ninput = 4\nassert(output, 0.5)",
        None,
    );
}

#[test]
fn eight_normative_identities_keep_relative_order_controls_assertions_and_explicit_returns() {
    let documents = [
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
    ];
    let expected = [
        (
            "spec",
            "Nucleoid keeps cubic smoothstep reactive to all three inputs",
            16,
            Some("0.5"),
        ),
        (
            "spec",
            "Nucleoid preserves numeric coercion while smoothing values",
            10,
            None,
        ),
        (
            "spec",
            "Nucleoid preserves special values and omitted smoothstep inputs",
            24,
            None,
        ),
        (
            "spec",
            "Nucleoid applies cubic smoothstep to live class rules",
            19,
            None,
        ),
        (
            "spec",
            "Nucleoid preserves deferred reads and rollback for cubic smoothstep",
            17,
            Some("0.15625"),
        ),
        (
            "reference",
            "Nucleoid preserves cubic smoothstep relationships",
            4,
            None,
        ),
        (
            "readme",
            "Cubic smoothstep keeps input changes live",
            3,
            None,
        ),
        (
            "examples",
            "Smooth cubic transitions preserve clamped endpoints",
            8,
            None,
        ),
    ];
    let rewrites = [
        (
            "eased = Math.smoothstep(lower, upper, value)",
            "eased = unit * unit * (3 - 2 * unit)",
        ),
        (
            "$Curve.eased = Math.smoothstep(lower, upper, $Curve.sample)",
            "$Curve.eased = $Curve.unit * $Curve.unit * (3 - 2 * $Curve.unit)",
        ),
        (
            "eased = Math.smoothstep(0, 1, value)",
            "eased = unit * unit * (3 - 2 * unit)",
        ),
        ("empty = Math.smoothstep()", "empty = Math.lerp()"),
        ("one = Math.smoothstep(0)", "one = Math.lerp(0)"),
        ("two = Math.smoothstep(0, 1)", "two = Math.lerp(0, 1)"),
        (
            "withExtra = Math.smoothstep(0, 1, 0.5, 99)",
            "withExtra = Math.saturate(half * half * (3 - 2 * half), 99)",
        ),
        (
            "withIgnored = Math.smoothstep(0, 1, 0.5, ignored)",
            "withIgnored = Math.saturate(half * half * (3 - 2 * half), ignored)",
        ),
    ];
    let mut identities = HashSet::new();
    let mut previous = HashMap::new();
    let mut assertions = 0;
    let mut returns = 0;
    let mut rewritten = 0;
    for (document, title, count, expected_return) in expected {
        assert!(identities.insert((document, title)));
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == document)
            .unwrap()
            .1;
        let matching = cases
            .iter()
            .enumerate()
            .filter(|(_, case)| case.title == title)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "{document}: {title}");
        let (position, case) = matching[0];
        if let Some(before) = previous.insert(document, position) {
            assert!(before < position, "{document}: relative order changed");
        }
        assert_eq!(case.expected.as_deref(), expected_return, "{title}");
        assert!(case.source.contains("unit = Math.saturate("), "{title}");
        let mut accepted = case.source.clone();
        let mut changes = 0;
        for (candidate, control) in rewrites {
            changes += accepted.matches(candidate).count();
            accepted = accepted.replace(candidate, control);
        }
        assert_ne!(case.source, accepted, "{title}");
        assert_eq!(
            changes,
            case.source.matches("Math.smoothstep(").count(),
            "{title}"
        );
        let actual_ast = parse(&case.source).unwrap();
        let accepted_ast = parse(&accepted).unwrap();
        assert_ne!(
            actual_ast, accepted_ast,
            "the runtime feature changes the result AST"
        );
        let actual_assertions = common::assertions::protected_assertions(&actual_ast);
        assert_eq!(actual_assertions.len(), count, "{title}");
        assert_eq!(
            actual_assertions,
            common::assertions::protected_assertions(&accepted_ast),
            "{title}"
        );
        let accepted_case = common::Case {
            title: case.title.clone(),
            source: accepted,
            expected: case.expected.clone(),
        };
        let mut actual = Runtime::new();
        let mut control = Runtime::new();
        let actual_value = common::check_with_count(&mut actual, case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        let control_value = common::check_with_count(&mut control, &accepted_case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assert_eq!(actual.assertions_run(), count, "{title}");
        assert_eq!(control.assertions_run(), count, "{title}");
        assert_eq!(
            actual.serialize_json(&actual_value).unwrap(),
            control.serialize_json(&control_value).unwrap(),
            "{title}"
        );
        if !title.contains("class rules") {
            let dependencies = if document == "readme" {
                ["value"].as_slice()
            } else {
                ["lower", "upper", "value"].as_slice()
            };
            assert_dependencies(&actual, NodeKey::variable("eased"), dependencies);
            assert_dependencies(&control, NodeKey::variable("eased"), &["unit"]);
        }
        assertions += count;
        returns += usize::from(expected_return.is_some());
        rewritten += changes;
    }
    assert_eq!(identities.len(), 8);
    assert_eq!(assertions, 101);
    assert_eq!(returns, 2);
    assert_eq!(rewritten, 13);
}

#[test]
fn native_cli_runs_exact_file_bytes_with_json_results_errors_and_literal_eof() {
    for (index, (source, success, stdout, stderr)) in [
        (
            "assert(Math.smoothstep(0, 8, 2), 0.15625)\nMath.smoothstep(0, 8, 2)",
            true,
            "0.15625\n",
            "",
        ),
        ("Math.smoothstep(8, 0, 2)", true, "0.84375\n", ""),
        ("Math.smoothstep(null, 1, 0.5)", true, "0.5\n", ""),
        ("Math.smoothstep(0, 1, null)", true, "0.0\n", ""),
        (
            "Math.smoothstep(0, 1, Number.POSITIVE_INFINITY)",
            true,
            "1.0\n",
            "",
        ),
        ("Math.smoothstep(2, 2, 2)", true, "null\n", ""),
        ("String(Math.smoothstep())", true, "\"NaN\"\n", ""),
        ("String(Math.smoothstep(0))", true, "\"NaN\"\n", ""),
        ("String(Math.smoothstep(0, 1))", true, "\"NaN\"\n", ""),
        (
            "Math.smoothstep(0, 1, 0.5, Number.NaN, \"invalid\", {})",
            true,
            "0.5\n",
            "",
        ),
        (
            "def Number(value) => 999\nMath.smoothstep(\"0\", \"8\", \"2\")",
            true,
            "0.15625\n",
            "",
        ),
        (
            "Math.smoothstep(0, 8, 2, missing)",
            false,
            "",
            "ReferenceError: missing is not defined (line 1, column 1)\n",
        ),
        (
            "Math.smoothstep(0, 8, 2, Math.random())",
            false,
            "",
            "TypeError: Math.random is not supported (line 1, column 1)\n",
        ),
        (
            "class Sensor\nsensor = Sensor()\nMath.smoothstep(sensor.absent, 8, 2, missing)",
            false,
            "",
            "ReferenceError: missing is not defined (line 3, column 1)\n",
        ),
        (
            "Math.smoothstep(",
            false,
            "",
            "SyntaxError: Unexpected end of input (line 1, column 17)\n",
        ),
        (
            "Math.smoothstep(\n",
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
                "nuc33-math-smoothstep-{}-{index}.nuc",
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

fn repl(source: &str, graph: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_nucleoid"));
    command.arg("--json");
    if graph {
        command.arg("--graph");
    }
    let mut child = command
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
fn normal_stdin_eof_and_repl_recovery_need_no_new_call_buffering() {
    let banner = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";
    for (source, values, stderr) in [
        (
            "value = 0.25\neased = Math.smoothstep(0, 1, value)\nvalue = 0.75\neased",
            "0.25\n0.15625\n0.75\n0.84375\n",
            "",
        ),
        (
            "value = 0.25\neased = Math.smoothstep(0, 1, value)\nMath.smoothstep(0, 1, 0.5, missing)\nvalue = 0.75\neased\nString(Math.smoothstep())\nMath.smoothstep(0, 1, 0.5, 99)\n.exit",
            "0.25\n0.15625\n0.75\n0.84375\n\"NaN\"\n0.5\n",
            "ReferenceError: missing is not defined (line 1, column 1)\n",
        ),
    ] {
        assert!(!source.ends_with('\n'));
        let output = repl(source, false);
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, format!("{banner}{values}").as_bytes());
        assert_eq!(output.stderr, stderr.as_bytes());
    }
    let output = repl(
        "lower = 0\nupper = 8\nvalue = 2\nextra = 99\neased = Math.smoothstep(lower, upper, value, extra)\nvalue = 4\neased",
        true,
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("eased [variable]"), "{stdout}");
    assert!(
        stdout.contains("reads    lower, upper, value, extra"),
        "{stdout}"
    );
    assert!(stdout.lines().any(|line| line == "0.5"), "{stdout}");
}
