mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use nucleoid::builtins;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Ast, parse, parse_expression};
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn assert_same(actual: &Value, control: &Value) {
    match (actual, control) {
        (Value::Number(actual), Value::Number(control)) => {
            if control.is_nan() {
                assert!(actual.is_nan(), "{actual:?} != {control:?}");
            } else {
                assert_eq!(actual.to_bits(), control.to_bits());
            }
        }
        _ => assert_eq!(actual, control),
    }
}

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "decimal power boundary".to_string(),
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

fn paired_sources(source: &str) -> [String; 2] {
    ["Math.exp10(", "Math.pow(10, "].map(|call| source.replace("DECIMAL(", call))
}

#[test]
fn numerical_bits_match_the_unchanged_pow_dispatch() {
    let exponents = [
        f64::NEG_INFINITY,
        -f64::MAX,
        -400.0,
        -324.0,
        -323.5,
        -323.0,
        -308.0,
        -3.5,
        -1.0,
        -f64::MIN_POSITIVE,
        -f64::from_bits(1),
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::EPSILON,
        0.5,
        1.0,
        1.5,
        3.0,
        308.0,
        f64::from_bits(309.0_f64.to_bits() - 1),
        309.0,
        f64::MAX,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(0xfff8_0000_0000_0042),
        f64::from_bits(0x7ff0_0000_0000_0001),
    ];
    let mut bits = 0x1610_57bd_83a2_690f_u64;
    let samples = (0..4096).map(|_| {
        bits = bits
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        f64::from_bits(bits)
    });
    let quarters = (-1600..=1300).map(|quarter| f64::from(quarter) / 4.0);
    for exponent in exponents.into_iter().chain(samples).chain(quarters) {
        let input = Value::Number(exponent);
        assert_same(
            &builtins::math("exp10", std::slice::from_ref(&input)).unwrap(),
            &builtins::math("pow", &[Value::number(10), input]).unwrap(),
        );
    }
    for (input, expected) in [
        (-400.0, 0.0),
        (-2.0, 0.01),
        (-1.0, 0.1),
        (-0.0, 1.0),
        (0.0, 1.0),
        (2.0, 100.0),
        (3.0, 1000.0),
        (309.0, f64::INFINITY),
        (f64::NEG_INFINITY, 0.0),
        (f64::INFINITY, f64::INFINITY),
    ] {
        for method in ["exp10", "pow"] {
            let arguments = if method == "pow" {
                vec![Value::number(10), Value::Number(input)]
            } else {
                vec![Value::Number(input)]
            };
            assert_same(
                &builtins::math(method, &arguments).unwrap(),
                &Value::Number(expected),
            );
        }
    }
}

#[test]
fn every_value_kind_and_supplied_undefined_position_matches_pow() {
    let values = [
        Value::Undefined,
        Value::Null,
        Value::Bool(false),
        Value::Bool(true),
        Value::number(-2),
        Value::Number(f64::NAN),
        Value::string(" 2 "),
        Value::string(""),
        Value::string("invalid"),
        Value::List(vec![]),
        Value::List(vec![Value::string("-1")]),
        Value::List(vec![Value::Undefined]),
        Value::List(vec![Value::Null]),
        Value::List(vec![Value::number(1), Value::number(2)]),
        Value::Object(ObjectId::named("probe")),
        Value::Class("Probe".to_string()),
        Value::Date(-1),
        Runtime::new().run("/pow/").unwrap(),
        Runtime::new().run("value => value").unwrap(),
    ];
    assert_same(
        &builtins::math("exp10", &[]).unwrap(),
        &builtins::math("pow", &[Value::number(10)]).unwrap(),
    );
    for input in &values {
        assert_same(
            &builtins::math("exp10", std::slice::from_ref(input)).unwrap(),
            &builtins::math("pow", &[Value::number(10), input.clone()]).unwrap(),
        );
        for extra in &values {
            let arguments = [input.clone(), extra.clone(), Value::Null];
            let control = [Value::number(10), input.clone(), extra.clone(), Value::Null];
            assert_same(
                &builtins::math("exp10", &arguments).unwrap(),
                &builtins::math("pow", &control).unwrap(),
            );
        }
    }
    for position in 0..6 {
        let mut arguments = vec![Value::number(2); 6];
        arguments[position] = Value::Undefined;
        let control: Vec<_> = std::iter::once(Value::number(10))
            .chain(arguments.iter().cloned())
            .collect();
        assert_same(
            &builtins::math("exp10", &arguments).unwrap(),
            &builtins::math("pow", &control).unwrap(),
        );
        assert_same(
            &builtins::math("exp10", &arguments).unwrap(),
            &Value::Undefined,
        );
    }
    assert!(builtins::math_constant("exp10").is_none());
}

#[test]
fn the_appended_spec_case_keeps_twelve_assertions_and_its_pow_control() {
    let cases = common::cases(include_str!("../nucleoid.spec.md"));
    let case = cases.last().unwrap();
    assert_eq!(
        case.title,
        "Nucleoid keeps base ten exponentiation reactive"
    );
    assert_eq!(case.expected.as_deref(), Some("1000"));
    assert_eq!(case.source.matches("scale = Math.exp10(power)").count(), 1);
    assert_eq!(
        case.source
            .matches("assert(scale, Math.pow(10, power))")
            .count(),
        1
    );
    let control = common::Case {
        title: case.title.clone(),
        source: case.source.replacen(
            "scale = Math.exp10(power)",
            "scale = Math.pow(10, power)",
            1,
        ),
        expected: case.expected.clone(),
    };
    let assertions = common::assertions::protected_assertions(&parse(&case.source).unwrap());
    assert_eq!(assertions.len(), 12);
    assert_eq!(
        common::assertions::protected_assertions(&parse(&control.source).unwrap()),
        assertions
    );
    common::check_with_assertions(case, 12).unwrap();
    common::check_with_assertions(&control, 12).unwrap();
}

#[test]
fn reactive_exponents_and_ignored_extra_reads_keep_recovery_and_dependencies() {
    for source in paired_sources(
        r#"
class Probe
probe = Probe()
probe.power = 2
extra = 9
scale = DECIMAL(probe.power, extra, probe.optional)
assert(scale, null)
probe.optional = 1
assert(scale, 100)
extra = "ignored"
assert(scale, 100)
probe.power = -1
assert(scale, 0.1)
extra = null
assert(scale, null)
extra = Number.NaN
assert(scale, 0.1)
delete probe.power
assert(scale, null)
probe.power = 3
assert(scale, 1000)
delete probe.optional
assert(scale, null)
probe.optional = [1, 2]
assert(scale, 1000)
"#,
    ) {
        let runtime = run(&source);
        let destination = NodeKey::variable("scale");
        for dependency in [
            NodeKey::variable("extra"),
            NodeKey::property(&ObjectId::named("probe"), "power"),
            NodeKey::property(&ObjectId::named("probe"), "optional"),
        ] {
            assert!(
                runtime.graph.reaches(&dependency, &destination),
                "{dependency:?}"
            );
        }
    }
}

#[test]
fn arguments_run_once_in_order_before_the_gate_and_keep_strict_error_rollback() {
    for source in paired_sources(
        r#"
class Log
log = Log()
log.order = 0
scale = DECIMAL(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3
)
assert(log.order, 123)
assert(scale, 10)
deferred = DECIMAL(log.input, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.input = 3
assert(deferred, 1000)
assert(log.order, 7)
log.order = 0
mapped = [0, 1, -1].map(value => DECIMAL(value, log.order = log.order.value * 10 + 1))
assert(mapped, [1, 10, 0.1])
assert(log.order, 111)
try:
    DECIMAL(log.order = 9, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 111)
try:
    DECIMAL(log.absent, Math.random(), missing)
catch error:
    assert(error, TypeError("Math.random is not supported"))
try:
    DECIMAL(log.absent, missing, Math.random())
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(Math.unknown(log.absent), null)
assert(Math.random(log.absent), null)
"#,
    ) {
        let mut runtime = run(&source);
        for call in paired_sources("log.order = 9\nDECIMAL(log.absent, missing)") {
            let error = runtime.run(&call).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Reference);
            assert_eq!(error.message(), "missing is not defined");
            assert_eq!(error.position(), Some(Position::new(2, 1)));
            checked(&mut runtime, "assert(log.order, 111)", None);
            assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
        }
    }
}

#[test]
fn intrinsic_conversion_and_null_deferral_keep_shadowing_and_native_member_reads() {
    for source in paired_sources(
        r#"
def Number(value) => 999
assert(Number("2"), 999)
text = "2"
scale = DECIMAL(text)
assert(scale, 100)
text = "-1"
assert(scale, 0.1)
def numeric(Number):
    return DECIMAL("3")
assert(numeric(null), 1000)
"#,
    ) {
        let runtime = run(&source);
        let node = runtime.graph.retrieve(&NodeKey::variable("scale")).unwrap();
        assert_eq!(
            node.dependencies
                .iter()
                .map(NodeKey::as_str)
                .collect::<Vec<_>>(),
            ["text"]
        );
    }
    run(r#"
assert(Math.exp10(null), 1)
assert(Number.isNaN(Math.exp10()), true)
assert(Math.exp10(2, Number.NaN, "invalid", {}, null), 100)
def power input: Numeric => Math.exp10(input)
omitted = power()
parameter = power(null)
assert(omitted, null)
assert(parameter, null)
assert(power(3, 99), 1000)
reference = Math.exp10
indirect = reference(3)
assert(reference, null)
assert(indirect, null)
class Namespace:
    def exp10 value => value + 8
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.exp10(2), 10)
def scoped(Math):
    return Math.exp10(3)
assert(scoped(namespace), 11)
assert(scoped(null), null)
Math = namespace
assert(Math.exp10(4), 12)
"#);
    let mut runtime = run("Number = 7\nassert(Math.exp10(\"2\"), 100)\nMath = 7");
    let error = runtime.run("Math.exp10(3)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.exp10 is not a function");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn wrappers_independent_class_rules_and_direct_resolution_keep_scope_frames() {
    for source in paired_sources(
        r#"
input = 3
extra = 99
def power amount: Numeric => DECIMAL(amount, extra)
scale = power(input)
assert(scale, 1000)
input = -2
assert(scale, 0.01)
class Reading(power):
    this.power = power
$Reading.factor = DECIMAL($Reading.power)
first = Reading(2)
second = Reading(-1)
assert(first.factor, 100)
assert(second.factor, 0.1)
first.power = 3
assert(first.factor, 1000)
assert(second.factor, 0.1)
delete first.power
assert(first.factor, null)
first.power = -2
assert(first.factor, 0.01)
"#,
    ) {
        let runtime = run(&source);
        let node = runtime.graph.retrieve(&NodeKey::variable("scale")).unwrap();
        for name in ["power", "input", "extra"] {
            assert!(node.dependencies.contains(&NodeKey::variable(name)));
        }
    }
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.push();
    scope.declare("input", Value::number(3));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    for source in paired_sources("DECIMAL(input)") {
        let expression = parse_expression(&source).unwrap();
        assert_same(
            &Ast::convert(&expression)
                .resolve(&mut runtime, &mut scope)
                .unwrap(),
            &Value::number(1000),
        );
    }
    let failed = parse_expression("Math.exp10(input, missing)").unwrap();
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
fn failed_replacements_restore_complete_state_graph_and_future_propagation() {
    let initial = r#"
class Reading(power):
    this.power = power
$Reading.factor = Math.exp10($Reading.power)
reading = Reading(2)
second = Reading(-1)
input = reading.power
alternative = 5
scale = Math.exp10(input)
assert(scale, 100)
assert(reading.factor, 100)
assert(second.factor, 0.1)
"#;
    let mut actual = run(initial);
    let mut control = run(initial);
    let failures = [
        "scale = Math.exp10(alternative)\nreading.power = 3\nthrow \"ROLLBACK\"",
        "scale = Math.exp10(alternative)\nscale = Math.exp10(missing)",
    ];
    let error = actual.run(failures[0]).unwrap_err();
    assert_eq!(error.thrown_value(), Some(&Value::string("ROLLBACK")));
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    let error = actual.run(failures[1]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    for source in failures {
        control.run(source).unwrap_err();
    }
    for runtime in [&mut actual, &mut control] {
        checked(
            runtime,
            r#"
alternative = 4
assert(scale, 100)
delete reading.power
assert(scale, null)
assert(reading.factor, null)
assert(second.factor, 0.1)
reading.power = 3
assert(scale, 1000)
assert(reading.factor, 1000)
assert(second.factor, 0.1)
scale
"#,
            Some("1000"),
        );
        assert!(!runtime.graph.reaches(
            &NodeKey::variable("alternative"),
            &NodeKey::variable("scale")
        ));
    }
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
}

#[test]
fn the_self_contained_synth_record_executes_all_assertions_and_its_return() {
    let records = include_str!("../dataset/nucleoid.synth.161.jsonl");
    assert_eq!(records.lines().count(), 1);
    let record: serde_json::Value = serde_json::from_str(records.trim()).unwrap();
    assert_eq!(record["id"], "synth-161-0001");
    let code = record["code"].as_str().unwrap();
    assert_eq!(
        common::assertions::protected_assertions(&parse(code).unwrap()).len(),
        12
    );
    checked(&mut Runtime::new(), code, record["return"].as_str());
}
