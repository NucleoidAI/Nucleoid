mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use nucleoid::builtins;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Ast, Stmt, parse, parse_expression};
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "Math.cube regression".to_string(),
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

fn assert_same(actual: &Value, expected: &Value) {
    if let (Value::Number(actual), Value::Number(expected)) = (actual, expected) {
        if expected.is_nan() {
            assert!(actual.is_nan(), "{actual} is not NaN");
        } else {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
    } else {
        assert_eq!(actual, expected);
    }
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

#[test]
fn measured_programs_change_only_the_pinned_rhs_and_execute_six_assertions_each() {
    for (cases, title, accepted, before, after) in [
        (
            common::cases(include_str!("../nucleoid.spec.md")),
            "Nucleoid keeps cubes reactive",
            "sample = {amplitude: 3}\noffset = 2\ncubed = Math.square(sample.amplitude + offset) * (sample.amplitude + offset)\nassert(cubed, 125)\nsample.amplitude = -4\nassert(cubed, -8)\noffset = 1\nassert(cubed, -27)\nsample.amplitude = 0\nassert(cubed, 1)\noffset = -1\nassert(cubed, -1)\nsample.amplitude = 3\nassert(cubed, 8)\n1",
            "cubed = Math.square(sample.amplitude + offset) * (sample.amplitude + offset)",
            "cubed = Math.cube(sample.amplitude + offset)",
        ),
        (
            common::cases(include_str!("reference.md")),
            "Nucleoid preserves cubing argument dependencies",
            "probe = {input: 2, extra: 7}\noffset = 1\nobserved = Math.square(probe.input + offset, probe.extra) * (probe.input + offset)\nassert(observed, 27)\nprobe.input = 3\nassert(observed, 64)\ndelete probe.extra\nassert(observed, null)\nprobe.extra = 0\nassert(observed, 64)\noffset = -2\nassert(observed, 1)\nprobe.input = -1\nassert(observed, -27)\n1",
            "observed = Math.square(probe.input + offset, probe.extra) * (probe.input + offset)",
            "observed = Math.cube(probe.input + offset, probe.extra)",
        ),
    ] {
        let matching = cases
            .iter()
            .filter(|case| case.title == title)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1);
        let case = matching[0];
        assert_eq!(case.expected.as_deref(), Some("1"));
        assert_eq!(accepted.matches(before).count(), 1);
        assert_eq!(
            case.source.trim(),
            format!("# {title}\n\n{}", accepted.replace(before, after))
        );
        let assertions = common::assertions::protected_assertions(&parse(accepted).unwrap());
        assert_eq!(assertions.len(), 6);
        assert_eq!(
            common::assertions::protected_assertions(&parse(&case.source).unwrap()),
            assertions
        );
        for source in [case.source.as_str(), accepted] {
            checked(&mut Runtime::new(), source, Some("1"));
        }
    }
}

#[test]
fn intrinsic_first_value_coercions_omission_and_nested_undefined_stay_numeric() {
    for (input, expected) in [
        (Value::Null, 0.0),
        (Value::Bool(true), 1.0),
        (Value::Bool(false), 0.0),
        (Value::String(String::new()), 0.0),
        (Value::String(" -3.25 ".to_string()), -34.328125),
        (Value::String("invalid".to_string()), f64::NAN),
        (Value::List(vec![]), 0.0),
        (Value::List(vec![Value::String("-2".to_string())]), -8.0),
        (
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            f64::NAN,
        ),
        (Value::List(vec![Value::Undefined]), f64::NAN),
        (Value::Date(-3), -27.0),
        (Value::Object(ObjectId::named("unconverted")), f64::NAN),
        (Value::Class("Probe".to_string()), f64::NAN),
    ] {
        assert_same(
            &builtins::math("cube", &[input]).unwrap(),
            &Value::Number(expected),
        );
    }
    assert_same(
        &builtins::math("cube", &[]).unwrap(),
        &Value::Number(f64::NAN),
    );
    let mut runtime = run("class Probe");
    for input in [
        "/probe/",
        "value => value",
        "Probe",
        "Probe.value",
        "Probe()",
        "Date(-3)",
        "[\"-2\"]",
    ] {
        let actual = checked(&mut runtime, &format!("Math.cube({input})"), None);
        let expected = checked(
            &mut runtime,
            &format!("Math.square({input}) * ({input})"),
            None,
        );
        assert_same(&actual, &expected);
    }
    run(r#"
assert(Math.cube(null), 0)
assert(Math.cube(true), 1)
assert(Math.cube(" -3.25 "), -34.328125)
assert(Number.isNaN(Math.cube()), true)
assert(Math.cube(-3, Number.NaN, "invalid", {}, null, [1, 2]), -27)
assert(Math.cube(2, Date(99), /probe/, value => value), 8)
"#);
}

#[test]
fn ordered_multiplications_preserve_pinned_finite_subnormal_and_signed_zero_bits() {
    for (input, bits) in [
        (5.0, 0x405f_4000_0000_0000),
        (-3.25, 0xc041_2a00_0000_0000),
        (0.0, 0x0000_0000_0000_0000),
        (-0.0, 0x8000_0000_0000_0000),
        (f64::from_bits(1), 0x0000_0000_0000_0000),
        (-f64::from_bits(1), 0x8000_0000_0000_0000),
        (1e-107, 0x0000_0000_0000_00ca),
        (-1e-107, 0x8000_0000_0000_00ca),
        (0.999_999_999_999_999_9, 0x3fef_ffff_ffff_fffd),
        (1.000_000_000_000_000_2, 0x3ff0_0000_0000_0003),
        (1.234_567_890_123_456_7, 0x3ffe_1b58_af0e_a355),
        (1e102, 0x7f76_c8e5_ca23_9029),
        (-1e103, 0xfff0_0000_0000_0000),
        (f64::MAX, 0x7ff0_0000_0000_0000),
        (-f64::MAX, 0xfff0_0000_0000_0000),
        (f64::INFINITY, 0x7ff0_0000_0000_0000),
        (f64::NEG_INFINITY, 0xfff0_0000_0000_0000),
    ] {
        assert_same(
            &builtins::math("cube", &[Value::Number(input)]).unwrap(),
            &Value::Number(f64::from_bits(bits)),
        );
    }
    for input in [f64::NAN, f64::from_bits(0xfff8_0000_0000_1234)] {
        assert_same(
            &builtins::math("cube", &[Value::Number(input)]).unwrap(),
            &Value::Number(f64::NAN),
        );
    }
}

#[test]
fn any_supplied_top_level_undefined_gates_before_coercion_and_extras_are_ignored() {
    for index in 0..5 {
        let mut arguments = vec![Value::Number(-3.0); 5];
        arguments[index] = Value::Undefined;
        assert_eq!(
            builtins::math("cube", &arguments).unwrap(),
            Value::Undefined
        );
    }
    assert_same(
        &builtins::math(
            "cube",
            &[
                Value::Number(-3.0),
                Value::List(vec![Value::Undefined]),
                Value::Class("not_numeric".to_string()),
                Value::Number(f64::NAN),
                Value::Object(ObjectId::named("ignored")),
            ],
        )
        .unwrap(),
        &Value::Number(-27.0),
    );
    for name in ["cube", "unknown", "random", "toDegrees"] {
        assert_eq!(
            builtins::math(name, &[Value::Number(f64::NAN), Value::Undefined]).unwrap(),
            Value::Undefined,
        );
    }
}

#[test]
fn all_arguments_execute_once_left_to_right_including_after_undefined_and_before_errors() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 0
cubed = Math.cube(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3
)
assert([cubed, log.order], [1, 123])
deferred = Math.cube(log.absent, log.order = 4, log.order = 5)
assert([deferred, log.order], [null, 5])
log.absent = -3
assert([deferred, log.order], [-27, 5])
extraMissing = Math.cube(3, log.missing, log.order = 6)
assert([extraMissing, log.order], [null, 6])
invalid = Math.cube(Number.NaN, log.order = 7)
assert(Number.isNaN(invalid), true)
assert(log.order, 7)
log.order = 0
mapped = [3, -4].map(value => Math.cube(value, log.order = log.order.value * 10 + 1))
assert([mapped, log.order], [[27, -64], 11])
"#);
    for (source, kind, message) in [
        (
            "Math.cube(missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.cube(3, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.cube(log.missing, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.cube(3, log.missing, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.cube(log.missing, Math.random(), missing)",
            ErrorKind::Type,
            "Math.random is not supported",
        ),
        (
            "Math.cube(log.missing, missing, Math.random())",
            ErrorKind::Reference,
            "missing is not defined",
        ),
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
    let error = runtime
        .run("Math.cube(log.order = 9, missing, log.order = 99)")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    checked(&mut runtime, "assert(log.order, 11)", None);
}

#[test]
fn argument_subscriptions_and_observations_survive_live_null_deletion_and_recovery() {
    for (candidate, control) in [
        (
            "Math.cube(probe.input)",
            "Math.square(probe.input) * probe.input",
        ),
        (
            "Math.cube(3, probe.input)",
            "Math.square(3, probe.input) * 3",
        ),
        (
            "Math.cube(3, 99, probe.input)",
            "Math.square(3, 99, probe.input) * 3",
        ),
    ] {
        for expression in [candidate, control] {
            let runtime = run(&format!(
                "class Probe\nprobe = Probe()\ncubed = {expression}\n\
                 assert(cubed, null)\nprobe.input = 3\nassert(cubed, 27)\n\
                 delete probe.input\nassert(cubed, null)\n\
                 probe.input = 3\nassert(cubed, 27)\n\
                 probe.input = null\nassert(cubed, null)\n\
                 probe.input = 3\nassert(cubed, 27)"
            ));
            assert_eq!(dependencies(&runtime, "cubed"), ["probe", "probe.input"]);
            assert!(runtime.graph.reaches(
                &NodeKey::property(&ObjectId::named("probe"), "input"),
                &NodeKey::variable("cubed"),
            ));
        }
    }
    for expression in ["Math.cube(input)", "Math.square(input) * input"] {
        run(&format!(
            "input = -3\ncubed = {expression}\ntrace = why cubed\n\
             assert(trace[0].holds, -27)\nassert(trace[0].from, [\"input\"])\n\
             input = 4\nassert(trace[0].holds, 64)\n\
             delete input\nassert(cubed, null)\n\
             input = -2\nassert(trace[0].holds, -8)"
        ));
    }
}

#[test]
fn functions_captures_intrinsic_conversion_and_parameter_deferral_keep_existing_behavior() {
    let intrinsic = run(r#"
def Number(value) => 999
input = " -3 "
output = Math.cube(input)
assert(Number(input), 999)
assert(output, -27)
"#);
    assert_eq!(dependencies(&intrinsic, "output"), ["input"]);
    let runtime = run(r#"
extra = 7
def cubic(value: Numeric) => Math.cube(value, extra)
def legacy(value: Numeric) => Math.square(value, extra) * value
input = "-3"
live = cubic(input)
frozen = Math.cube(input.value)
assert([live, frozen], [-27, -27])
input = "4"
assert([live, frozen], [64, -27])
omitted = cubic()
parameter = cubic(null)
assert([parameter, omitted], [null, null])
assert([cubic(null), cubic()], [legacy(null), legacy()])
extra = null
assert(live, null)
extra = Number.NaN
assert(live, 64)
def intrinsic(Number):
    return Math.cube(" -5 ")
assert(intrinsic(null), -125)
class Probe
capturedClass = Probe.value
assert(Number.isNaN(Math.cube(capturedClass)), true)
"#);
    let reads = dependencies(&runtime, "live");
    for name in ["cubic", "input", "extra"] {
        assert!(reads.contains(&name), "{name}");
    }
    assert!(!reads.contains(&"Number"));
    assert!(!runtime.graph.contains(&NodeKey::variable("Numeric")));
    assert!(dependencies(&runtime, "frozen").is_empty());
}

#[test]
fn native_values_shadowed_members_and_closed_receiver_dispatch_stay_ordinary() {
    run(r#"
Number = 7
assert(Math.cube("-3"), -27)
reference = Math.cube
indirect = reference(3)
assert([reference, indirect], [null, null])
record = {cube: 7}
assert(record.cube, 7)
class Namespace:
    def cube(value) => value + 8
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.cube(3), 11)
def scoped(Math):
    return Math.cube(3)
assert([scoped(namespace), scoped(null)], [11, null])
Math = namespace
assert(Math.cube(4), 12)
"#);
    assert!(builtins::math_constant("cube").is_none());
    assert_eq!(Runtime::new().run("Math.cube").unwrap(), Value::Undefined);
    for (setup, source, kind, message) in [
        (
            "Math = 7",
            "Math.cube(3)",
            ErrorKind::Type,
            "number.cube is not a function",
        ),
        (
            "items = [1]",
            "items.cube()",
            ErrorKind::Type,
            "items.cube is not a function",
        ),
        (
            "class Population\nPopulation()",
            "Population.cube()",
            ErrorKind::Type,
            "Population.cube is not a function",
        ),
        ("", "cube(3)", ErrorKind::Reference, "cube is not defined"),
        (
            "",
            "math.cube(3)",
            ErrorKind::Reference,
            "math is not defined",
        ),
        (
            "",
            "Math.Cube(3)",
            ErrorKind::Type,
            "Math.Cube is not a function",
        ),
        (
            "",
            "Math.toDegrees(3)",
            ErrorKind::Type,
            "Math.toDegrees is not a function",
        ),
        (
            "",
            "Math.unknown(3)",
            ErrorKind::Type,
            "Math.unknown is not a function",
        ),
        (
            "",
            "Math.random()",
            ErrorKind::Type,
            "Math.random is not supported",
        ),
    ] {
        let error = run(setup).run(source).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(1, 1)));
    }
}

#[test]
fn ordinary_call_ast_graph_source_and_caller_frames_are_preserved() {
    let runtime = run("input = -3\nextra = 7\ncubed = Math.cube(input, extra)");
    let node = runtime.graph.retrieve(&NodeKey::variable("cubed")).unwrap();
    let Some(Nuc::Variable(variable)) = &node.node else {
        panic!("the source remains a variable rule");
    };
    assert_eq!(
        variable.value,
        parse_expression("Math.cube(input, extra)").unwrap()
    );
    assert_eq!(dependencies(&runtime, "cubed"), ["input", "extra"]);
    let statements = parse("Math.cube(input, extra)").unwrap();
    let [Stmt::Expression(expression)] = statements.as_slice() else {
        panic!("the helper remains an ordinary expression");
    };
    let Ast::Call(call) = Ast::convert(expression) else {
        panic!("the helper remains an ordinary call");
    };
    assert_eq!(
        call.function(),
        Some(&parse_expression("Math.cube").unwrap())
    );
    assert_eq!(
        call.arguments(),
        [
            parse_expression("input").unwrap(),
            parse_expression("extra").unwrap(),
        ]
    );
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.push();
    scope.declare("input", Value::Number(-0.0));
    scope.declare("extra", Value::Number(f64::NAN));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    assert_same(
        &call.resolve(&mut runtime, &mut scope).unwrap(),
        &Value::Number(-0.0),
    );
    let failed = parse_expression("Math.cube(input, extra, missing)").unwrap();
    let error = Ast::convert(&failed)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_same(scope.retrieve("input").unwrap(), &Value::Number(-0.0));
    assert!(matches!(scope.retrieve("extra"), Some(Value::Number(value)) if value.is_nan()));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
}

#[test]
fn independent_class_rules_methods_and_instances_keep_deletion_and_recovery() {
    run(r#"
class Signal(amplitude):
    this.amplitude = amplitude
    def cubed() => Math.cube(self.amplitude)
$Signal.cubic = Math.cube($Signal.amplitude)
first = Signal(3)
second = Signal(-4)
assert([first.cubic, second.cubic, first.cubed()], [27, -64, 27])
first.amplitude = -5
assert([first.cubic, second.cubic], [-125, -64])
third = Signal()
assert(third.cubic, null)
third.amplitude = 2
assert([first.cubic, second.cubic, third.cubic], [-125, -64, 8])
delete first.amplitude
assert([first.cubic, second.cubic], [null, -64])
first.amplitude = -6
assert([first.cubic, second.cubic], [-216, -64])
assert(Signal.length, 3)
"#);
}

#[test]
fn failed_single_dependency_transactions_restore_snapshots_and_recover_like_arithmetic() {
    for expression in ["Math.cube(input)", "Math.square(input) * input"] {
        let initial = format!("input = -3\ncubed = {expression}");
        let mut actual = run(&initial);
        let control = run(&initial);
        let error = actual.run("input = -5\nthrow \"ROLLBACK\"").unwrap_err();
        assert_eq!(
            error.thrown_value(),
            Some(&Value::String("ROLLBACK".to_string()))
        );
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
        let error = actual.run("input = -7\nMath.cube(missing)").unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(2, 1)));
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
        checked(
            &mut actual,
            "assert([input, cubed], [-3, -27])\ninput = -4\nassert(cubed, -64)\n\
             delete input\nassert(cubed, null)\ninput = 5\nassert(cubed, 125)\ncubed",
            Some("125"),
        );
        assert_eq!(dependencies(&actual, "cubed"), ["input"]);
    }
}

#[test]
fn synthesized_cubing_programs_execute_all_assertions_and_expected_returns() {
    #[derive(serde::Deserialize)]
    struct Example {
        code: String,
        #[serde(rename = "return")]
        expected: Option<String>,
    }
    let records = include_str!("../dataset/nucleoid.synth.146.jsonl")
        .lines()
        .map(|line| serde_json::from_str::<Example>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 4);
    for record in records {
        checked(
            &mut Runtime::new(),
            &record.code,
            record.expected.as_deref(),
        );
    }
}
