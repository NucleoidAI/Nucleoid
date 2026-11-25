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
            title: "Math.square regression".to_string(),
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
fn measured_programs_change_only_the_pinned_expression_and_execute_all_six_assertions() {
    for (cases, title, accepted, before, after) in [
        (
            common::cases(include_str!("../nucleoid.spec.md")),
            "Nucleoid keeps squares reactive",
            "sample = {amplitude: 3}\noffset = 2\nsquared = (sample.amplitude + offset) * (sample.amplitude + offset)\nassert(squared, 25)\nsample.amplitude = -4\nassert(squared, 4)\noffset = 1\nassert(squared, 9)\nsample.amplitude = 0\nassert(squared, 1)\noffset = -1\nassert(squared, 1)\nsample.amplitude = 3\nassert(squared, 4)\n1",
            "(sample.amplitude + offset) * (sample.amplitude + offset)",
            "Math.square(sample.amplitude + offset)",
        ),
        (
            common::cases(include_str!("reference.md")),
            "Nucleoid preserves squaring argument dependencies",
            "probe = {amplitude: 3, extra: 7}\noffset = 2\nsquared = Math.abs(probe.amplitude + offset, probe.extra) * Math.abs(probe.amplitude + offset, probe.extra)\nassert(squared, 25)\nprobe.amplitude = -4\nassert(squared, 4)\ndelete probe.extra\nassert(squared, undefined)\nprobe.extra = 9\nassert(squared, 4)\noffset = 1\nassert(squared, 9)\nprobe.extra = Number.NaN\nassert(squared, 9)\n1",
            "Math.abs(probe.amplitude + offset, probe.extra) * Math.abs(probe.amplitude + offset, probe.extra)",
            "Math.square(probe.amplitude + offset, probe.extra)",
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
fn direct_intrinsic_coercions_omission_and_signed_results_match_accepted_products() {
    let mut runtime = run("class Probe");
    for expression in [
        "null",
        "true",
        "false",
        "\"\"",
        "\" -3.25 \"",
        "\"invalid\"",
        "[]",
        "[\"-2\"]",
        "[1, 2]",
        "Date(-3)",
        "{}",
        "/probe/",
        "value => value",
        "Probe",
        "Probe.value",
        "Probe()",
        "Number.NaN",
        "Number.POSITIVE_INFINITY",
        "Number.NEGATIVE_INFINITY",
        "-0",
        "Number.MIN_VALUE",
    ] {
        let actual = checked(&mut runtime, &format!("Math.square({expression})"), None);
        let expected = checked(
            &mut runtime,
            &format!("Math.abs({expression}) * Math.abs({expression})"),
            None,
        );
        assert_same(&actual, &expected);
    }
    run(r#"
assert(Number.isNaN(Math.square()), true)
assert(1 / Math.square(-0), Number.POSITIVE_INFINITY)
assert(Math.square(Number.MIN_VALUE), 0)
assert(Math.square(Number.MAX_VALUE), Number.POSITIVE_INFINITY)
assert(Math.square(3, Number.NaN, "invalid", {}, null, [1, 2]), 9)
assert(Math.square(-2, Date(99), /probe/, value => value), 4)
"#);
}

#[test]
fn every_argument_read_stays_live_through_deletion_null_and_recovery() {
    for (candidate, control) in [
        (
            "Math.square(probe.input)",
            "Math.abs(probe.input) * Math.abs(probe.input)",
        ),
        (
            "Math.square(3, probe.input)",
            "Math.abs(3, probe.input) * Math.abs(3, probe.input)",
        ),
        (
            "Math.square(3, 99, probe.input)",
            "Math.abs(3, 99, probe.input) * Math.abs(3, 99, probe.input)",
        ),
    ] {
        for expression in [candidate, control] {
            let runtime = run(&format!(
                "class Probe\nprobe = Probe()\nsquared = {expression}\n\
                 assert(squared, null)\nprobe.input = -3\nassert(squared, 9)\n\
                 delete probe.input\nassert(squared, null)\n\
                 probe.input = -3\nassert(squared, 9)\n\
                 probe.input = null\nassert(squared, null)\n\
                 probe.input = -3\nassert(squared, 9)"
            ));
            assert_eq!(dependencies(&runtime, "squared"), ["probe", "probe.input"]);
            assert!(runtime.graph.reaches(
                &NodeKey::property(&ObjectId::named("probe"), "input"),
                &NodeKey::variable("squared"),
            ));
        }
    }
    for name in ["square", "unknown", "random"] {
        let mut runtime = run("class Probe\nprobe = Probe()");
        assert_eq!(
            checked(&mut runtime, &format!("Math.{name}(3, probe.absent)"), None),
            Value::Undefined
        );
    }
}

#[test]
fn wrappers_captured_values_and_intrinsic_conversion_keep_existing_deferral() {
    let shadowed = run(r#"
def Number(value) => 999
assert(Number("-3"), 999)
input = "-3"
output = Math.square(input)
assert(output, 9)
"#);
    assert_eq!(dependencies(&shadowed, "output"), ["input"]);
    let runtime = run(r#"
extra = 7
def squared(value: Numeric) => Math.square(value, extra)
def legacy(value: Numeric) => Math.abs(value, extra) * Math.abs(value, extra)
input = "-3"
live = squared(input)
frozen = Math.square(input.value)
assert([live, frozen], [9, 9])
input = "4"
assert([live, frozen], [16, 9])
omitted = squared()
parameter = squared(null)
assert([parameter, omitted], [null, null])
assert([squared(null), squared()], [legacy(null), legacy()])
extra = null
assert(live, null)
extra = Number.NaN
assert(live, 16)
def intrinsic(Number):
    return Math.square(" -5 ")
assert(intrinsic(null), 25)
class Probe
capturedClass = Probe.value
assert(Number.isNaN(Math.square(capturedClass)), true)
"#);
    let reads = dependencies(&runtime, "live");
    for name in ["squared", "input", "extra"] {
        assert!(reads.contains(&name), "{name}");
    }
    assert!(!reads.contains(&"Number"));
    assert!(!runtime.graph.contains(&NodeKey::variable("Numeric")));
    assert!(dependencies(&runtime, "frozen").is_empty());
}

#[test]
fn supplied_expressions_run_once_left_to_right_even_after_undefined() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 0
squared = Math.square(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3
)
assert([squared, log.order], [1, 123])
deferred = Math.square(log.absent, log.order = 4, log.order = 5)
assert([deferred, log.order], [null, 5])
log.absent = -3
assert([deferred, log.order], [9, 5])
extraMissing = Math.square(3, log.missing, log.order = 6)
assert([extraMissing, log.order], [null, 6])
invalid = Math.square(Number.NaN, log.order = 7)
assert(Number.isNaN(invalid), true)
assert(log.order, 7)
log.order = 0
mapped = [3, -4].map(value => Math.square(value, log.order = log.order.value * 10 + 1))
assert([mapped, log.order], [[9, 16], 11])
"#);
    for (source, kind, message) in [
        (
            "Math.square(missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.square(3, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.square(log.missing, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.square(3, log.missing, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.square(log.missing, Math.random(), missing)",
            ErrorKind::Type,
            "Math.random is not supported",
        ),
        (
            "Math.square(log.missing, missing, Math.random())",
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
        .run("Math.square(log.order = 9, missing, log.order = 99)")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    checked(&mut runtime, "assert(log.order, 11)", None);
}

#[test]
fn shadowed_members_and_native_method_values_and_closed_receiver_dispatch_stay_ordinary() {
    run(r#"
Number = 7
assert(Math.square("-3"), 9)
reference = Math.square
indirect = reference(3)
assert([reference, indirect], [null, null])
record = {square: 7}
assert(record.square, 7)
class Namespace:
    def square(value) => value + 8
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.square(3), 11)
def scoped(Math):
    return Math.square(3)
assert([scoped(namespace), scoped(null)], [11, null])
offset = 1
operation = value => Math.square(value + offset)
control = value => (value + offset) * (value + offset)
assert(operation(3), control(3))
offset = -1
assert(operation(3), control(3))
Math = namespace
assert(Math.square(4), 12)
"#);
    assert!(builtins::math_constant("square").is_none());
    assert_eq!(Runtime::new().run("Math.square").unwrap(), Value::Undefined);
    for (setup, source, kind, message) in [
        (
            "Math = 7",
            "Math.square(3)",
            ErrorKind::Type,
            "number.square is not a function",
        ),
        (
            "items = [1]",
            "items.square()",
            ErrorKind::Type,
            "items.square is not a function",
        ),
        (
            "class Population\nPopulation()",
            "Population.square()",
            ErrorKind::Type,
            "Population.square is not a function",
        ),
        (
            "",
            "square(3)",
            ErrorKind::Reference,
            "square is not defined",
        ),
        (
            "",
            "Math.sqr(3)",
            ErrorKind::Type,
            "Math.sqr is not a function",
        ),
        (
            "",
            "Math.Square(3)",
            ErrorKind::Type,
            "Math.Square is not a function",
        ),
        (
            "",
            "Math.toDegrees(3)",
            ErrorKind::Type,
            "Math.toDegrees is not a function",
        ),
    ] {
        let error = run(setup).run(source).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(1, 1)));
    }
}

#[test]
fn ordinary_call_ast_graph_source_and_caller_frames_are_not_lowered() {
    let runtime = run("input = -3\nextra = 7\nsquared = Math.square(input, extra)");
    let node = runtime
        .graph
        .retrieve(&NodeKey::variable("squared"))
        .unwrap();
    let Some(Nuc::Variable(variable)) = &node.node else {
        panic!("the source remains a variable rule");
    };
    assert_eq!(
        variable.value,
        parse_expression("Math.square(input, extra)").unwrap()
    );
    assert_eq!(dependencies(&runtime, "squared"), ["input", "extra"]);
    let statements = parse("Math.square(input, extra)").unwrap();
    let [Stmt::Expression(expression)] = statements.as_slice() else {
        panic!("the helper remains an ordinary expression");
    };
    let Ast::Call(call) = Ast::convert(expression) else {
        panic!("the helper remains an ordinary call");
    };
    assert_eq!(
        call.function(),
        Some(&parse_expression("Math.square").unwrap())
    );
    assert_eq!(
        call.arguments(),
        [
            parse_expression("input").unwrap(),
            parse_expression("extra").unwrap()
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
        &Value::Number(0.0),
    );
    let failed = parse_expression("Math.square(input, extra, missing)").unwrap();
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
fn independent_class_rules_and_instances_keep_deletion_and_recovery() {
    run(r#"
class Signal(amplitude):
    this.amplitude = amplitude
    def squared() => Math.square(self.amplitude)
$Signal.energy = Math.square($Signal.amplitude)
first = Signal(3)
second = Signal(-4)
assert([first.energy, second.energy, first.squared()], [9, 16, 9])
first.amplitude = -5
assert([first.energy, second.energy], [25, 16])
third = Signal()
assert(third.energy, null)
third.amplitude = 2
assert([first.energy, second.energy, third.energy], [25, 16, 4])
delete first.amplitude
assert([first.energy, second.energy, first.squared()], [null, 16, null])
first.amplitude = -6
assert([first.energy, second.energy, first.squared()], [36, 16, 36])
assert(Signal.length, 3)
"#);
}

#[test]
fn failed_single_dependency_transactions_restore_snapshots_and_recover_like_arithmetic() {
    for expression in ["Math.square(input)", "Math.abs(input) * Math.abs(input)"] {
        let initial = format!("input = -3\nsquared = {expression}");
        let mut actual = run(&initial);
        let control = run(&initial);
        let error = actual.run("input = -5\nthrow \"ROLLBACK\"").unwrap_err();
        assert_eq!(
            error.thrown_value(),
            Some(&Value::String("ROLLBACK".to_string()))
        );
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
        let error = actual.run("input = -7\nMath.square(missing)").unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(2, 1)));
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
        checked(
            &mut actual,
            "assert([input, squared], [-3, 9])\ninput = -4\nassert(squared, 16)\n\
             delete input\nassert(squared, null)\ninput = 5\nassert(squared, 25)\nsquared",
            Some("25"),
        );
        assert_eq!(dependencies(&actual, "squared"), ["input"]);
    }
}

#[test]
fn focused_synthesized_programs_execute_their_assertions_and_expected_returns() {
    #[derive(serde::Deserialize)]
    struct Example {
        code: String,
        #[serde(rename = "return")]
        expected: Option<String>,
    }
    let records = include_str!("../dataset/nucleoid.synth.143.jsonl")
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
