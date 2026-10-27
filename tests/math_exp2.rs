//! Base-two powers use the accepted Math call, conversion and graph machinery.

mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Ast, parse, parse_expression};
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "Math.exp2 regression".to_string(),
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

#[test]
fn reactive_inputs_and_ignored_extra_reads_keep_deletion_and_recovery() {
    for call in [
        "Math.exp2(probe.power, extra, probe.optional)",
        "Math.pow(2, probe.power, extra, probe.optional)",
    ] {
        let runtime = run(&format!(
            r#"
class Probe
probe = Probe()
probe.power = 2
extra = 9
converted = {call}
assert(converted, null)
probe.optional = 1
assert(converted, 4)
extra = "ignored"
assert(converted, 4)
probe.power = -1
assert(converted, 0.5)
extra = null
assert(converted, null)
extra = Number.NaN
assert(converted, 0.5)
delete probe.power
assert(converted, null)
probe.power = 3
assert(converted, 8)
delete probe.optional
assert(converted, null)
probe.optional = [1, 2]
assert(converted, 8)
"#
        ));
        let destination = NodeKey::variable("converted");
        for source in [
            NodeKey::variable("extra"),
            NodeKey::property(&ObjectId::named("probe"), "power"),
            NodeKey::property(&ObjectId::named("probe"), "optional"),
        ] {
            assert!(runtime.graph.reaches(&source, &destination), "{source:?}");
        }
    }
}

#[test]
fn all_expressions_run_once_in_order_before_gating_and_strict_errors() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 0
converted = Math.exp2(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3
)
assert(log.order, 123)
assert(converted, 2)
deferred = Math.exp2(log.input, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.input = 3
assert(deferred, 8)
assert(log.order, 7)
log.order = 0
mapped = [0, 1, -1].map(value => Math.exp2(value, log.order = log.order.value * 10 + 1))
assert(mapped, [1, 2, 0.5])
assert(log.order, 111)
try:
    Math.exp2(log.order = 9, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 111)
try:
    Math.exp2(log.absent, Math.random(), missing)
catch error:
    assert(error, TypeError("Math.random is not supported"))
try:
    Math.exp2(log.absent, missing, Math.random())
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(Math.unknown(log.absent), null)
assert(Math.random(log.absent), null)
"#);
    for (source, kind, message) in [
        (
            "Math.exp2(missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.exp2(2, 9, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.exp2(log.absent, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.exp2(log.absent, Math.random())",
            ErrorKind::Type,
            "Math.random is not supported",
        ),
        (
            "Math.unknown()",
            ErrorKind::Type,
            "Math.unknown is not a function",
        ),
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(1, 1)));
    }
    let error = runtime
        .run("log.order = 9\nMath.exp2(log.absent, missing)")
        .unwrap_err();
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    checked(&mut runtime, "assert(log.order, 111)", None);
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
}

#[test]
fn intrinsic_coercion_null_deferral_and_shadowing_keep_existing_lookup() {
    let runtime = run(r#"
def Number(value) => 999
assert(Number("3"), 999)
text = "3"
converted = Math.exp2(text)
assert(converted, 8)
text = "-1"
assert(converted, 0.5)
def numeric(Number):
    return Math.exp2("3")
assert(numeric(null), 8)
"#);
    let node = runtime
        .graph
        .retrieve(&NodeKey::variable("converted"))
        .unwrap();
    assert_eq!(
        node.dependencies
            .iter()
            .map(NodeKey::as_str)
            .collect::<Vec<_>>(),
        ["text"],
    );
    run(r#"
assert(Math.exp2(null), 1)
assert(Number.isNaN(Math.exp2()), true)
assert(Math.exp2(3, Number.NaN, "invalid", {}, null), 8)
assert(Math.exp2(-1, [1, 2], Date(99), /probe/, value => value), 0.5)
def power input: Numeric => Math.exp2(input)
omitted = power()
parameter = power(null)
assert(omitted, null)
assert(parameter, null)
assert(power(3, 99), 8)
reference = Math.exp2
indirect = reference(3)
assert(reference, null)
assert(indirect, null)
class Namespace:
    def exp2 value => value + 8
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.exp2(2), 10)
def scoped(Math):
    return Math.exp2(3)
assert(scoped(namespace), 11)
assert(scoped(null), null)
Math = namespace
assert(Math.exp2(4), 12)
"#);
    let mut runtime = run("Number = 7\nassert(Math.exp2(\"3\"), 8)\nMath = 7");
    let error = runtime.run("Math.exp2(3)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.exp2 is not a function");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn wrappers_receivers_and_direct_calls_preserve_dependencies_and_scope_frames() {
    let runtime = run(r#"
input = 3
extra = 99
def power amount: Numeric => Math.exp2(amount, extra)
converted = power(input)
assert(converted, 8)
input = -2
assert(converted, 0.25)
class Reading:
    def init power => this.power = power
    def factor() => Math.exp2(self.power)
class Child: Reading
    def init(power):
        super(power)
    def factor() => Math.exp2(self.power)
reading = Child(2)
assert(reading.factor(), 4)
reading.power = -1
assert(reading.factor(), 0.5)
"#);
    let node = runtime
        .graph
        .retrieve(&NodeKey::variable("converted"))
        .unwrap();
    for name in ["power", "input", "extra"] {
        assert!(node.dependencies.contains(&NodeKey::variable(name)));
    }
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.push();
    scope.declare("input", Value::number(3));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    let expression = parse_expression("Math.exp2(input)").unwrap();
    assert_eq!(
        Ast::convert(&expression)
            .resolve(&mut runtime, &mut scope)
            .unwrap(),
        Value::number(8),
    );
    let failed = parse_expression("Math.exp2(input, missing)").unwrap();
    let error = Ast::convert(&failed)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.retrieve("input"), Some(&Value::number(3)));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
}

#[test]
fn failed_replacements_restore_state_graph_and_independent_class_rules() {
    let initial = r#"
class Reading(power):
    this.power = power
$Reading.factor = Math.exp2($Reading.power)
reading = Reading(2)
second = Reading(-1)
input = reading.power
alternative = 5
converted = Math.exp2(input)
assert(converted, 4)
assert(reading.factor, 4)
assert(second.factor, 0.5)
"#;
    let mut actual = run(initial);
    let mut control = run(initial);
    let failures = [
        "converted = Math.exp2(alternative)\nreading.power = 3\nthrow \"ROLLBACK\"",
        "converted = Math.exp2(alternative)\nconverted = Math.exp2(missing)",
    ];
    let error = actual.run(failures[0]).unwrap_err();
    assert_eq!(
        error.thrown_value(),
        Some(&Value::String("ROLLBACK".to_string()))
    );
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    let error = actual.run(failures[1]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    // Rollback restores existing nodes, not future sequence allocation.
    for source in failures {
        control.run(source).unwrap_err();
    }
    for runtime in [&mut actual, &mut control] {
        checked(
            runtime,
            r#"
alternative = 4
assert(converted, 4)
delete reading.power
assert(converted, null)
assert(reading.factor, null)
assert(second.factor, 0.5)
reading.power = 3
assert(converted, 8)
assert(reading.factor, 8)
assert(second.factor, 0.5)
converted
"#,
            Some("8"),
        );
        assert!(!runtime.graph.reaches(
            &NodeKey::variable("alternative"),
            &NodeKey::variable("converted"),
        ));
    }
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
}
