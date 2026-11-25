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
            title: "Math.copySign regression".to_string(),
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

fn number(value: &Value) -> f64 {
    let Value::Number(number) = value else {
        panic!("numeric result required: {value:?}");
    };
    *number
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
fn non_nan_magnitude_bits_and_signs_survive_extremes_while_nans_stay_classified() {
    for magnitude in [
        0.0,
        -0.0,
        f64::from_bits(1),
        -f64::from_bits(1),
        f64::from_bits(0x000f_ffff_ffff_ffff),
        -f64::from_bits(0x000f_ffff_ffff_ffff),
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        0.1,
        -0.1,
        1.0,
        -1.0,
        f64::MAX,
        f64::MIN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let accepted_magnitude = builtins::math("abs", &[Value::Number(magnitude)]).unwrap();
        for sign in [
            0.0,
            -0.0,
            f64::from_bits(1),
            -f64::from_bits(1),
            f64::MIN_POSITIVE,
            -f64::MIN_POSITIVE,
            1.0,
            -1.0,
            f64::MAX,
            f64::MIN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            let actual =
                builtins::math("copySign", &[Value::Number(magnitude), Value::Number(sign)])
                    .unwrap();
            let result = number(&actual);
            assert_eq!(
                result.abs().to_bits(),
                number(&accepted_magnitude).to_bits()
            );
            assert_eq!(result.is_sign_negative(), sign.is_sign_negative());
            assert_eq!(result.is_finite(), magnitude.is_finite());
        }
    }
    for nan in [f64::NAN, f64::from_bits(0xfff8_0000_0000_0142)] {
        for other in [0.0, -0.0, 3.0, -3.0, f64::MAX, f64::INFINITY, nan] {
            for arguments in [
                [Value::Number(nan), Value::Number(other)],
                [Value::Number(other), Value::Number(nan)],
            ] {
                assert!(number(&builtins::math("copySign", &arguments).unwrap()).is_nan());
            }
        }
    }
}

#[test]
fn intrinsic_two_input_coercion_omission_extras_and_top_level_undefined_are_distinct() {
    for expression in [
        "null",
        "true",
        "false",
        "\"\"",
        "\" -7 \"",
        "\"invalid\"",
        "[]",
        "[\"-2\"]",
        "[1, 2]",
        "Date(-2)",
        "{}",
        "/probe/",
        "value => value",
        "Probe",
        "Probe()",
        "Number.NaN",
        "Number.POSITIVE_INFINITY",
        "Number.NEGATIVE_INFINITY",
    ] {
        let mut runtime = run("class Probe");
        for (candidate, control) in [
            (
                format!("Math.copySign({expression}, -1)"),
                format!("-Math.abs({expression})"),
            ),
            (
                format!("Math.copySign(3, {expression})"),
                format!("Math.abs(3) * Math.sign(Math.sign({expression}) + 1 / ({expression}))"),
            ),
        ] {
            let actual = checked(&mut runtime, &candidate, None);
            let expected = checked(&mut runtime, &control, None);
            assert_same(&actual, &expected);
        }
    }
    for arguments in [
        vec![],
        vec![Value::Number(3.0)],
        vec![Value::Number(f64::NAN), Value::Number(-1.0)],
        vec![Value::Number(3.0), Value::String("invalid".to_string())],
    ] {
        assert!(number(&builtins::math("copySign", &arguments).unwrap()).is_nan());
    }
    for position in 0..7 {
        let mut arguments = vec![Value::Number(3.0), Value::Number(-1.0)];
        arguments.resize(7, Value::Number(f64::NAN));
        arguments[position] = Value::Undefined;
        for name in ["copySign", "unknown", "random"] {
            assert_eq!(builtins::math(name, &arguments).unwrap(), Value::Undefined);
        }
    }
    let nested = Value::List(vec![Value::Undefined]);
    for arguments in [
        [nested.clone(), Value::Number(-1.0)],
        [Value::Number(3.0), nested.clone()],
    ] {
        assert!(number(&builtins::math("copySign", &arguments).unwrap()).is_nan());
    }
    assert_same(
        &builtins::math(
            "copySign",
            &[Value::Number(3.0), Value::Number(-1.0), nested],
        )
        .unwrap(),
        &Value::Number(-3.0),
    );
    run(r#"
assert(Math.copySign(3, -1, Number.NaN, "invalid", {}, null), -3)
assert(Math.copySign(-3, 0, [1, 2], Date(99), /probe/, value => value), 3)
assert(1 / Math.copySign(null, -0), Number.NEGATIVE_INFINITY)
assert(Number.isNaN(Math.copySign()), true)
assert(Number.isNaN(Math.copySign(3)), true)
"#);
}

#[test]
fn sealed_document_programs_keep_all_assertions_and_returns_and_synth_records_execute() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    for (cases, title, accepted, before, after) in [
        (
            &spec,
            "Nucleoid keeps sign copying reactive",
            "magnitude = 3\ndirection = -1\ncopied = Math.abs(magnitude) * Math.sign(Math.sign(direction) + 1 / direction)\nassert(copied, -3)\ndirection = 1\nassert(copied, 3)\nmagnitude = -7\nassert(copied, 7)\ndirection = -0\nassert(copied, -7)\nmagnitude = 0\nassert(1 / copied, Number.NEGATIVE_INFINITY)\ndirection = 0\nassert(1 / copied, Number.POSITIVE_INFINITY)\n1",
            "Math.abs(magnitude) * Math.sign(Math.sign(direction) + 1 / direction)",
            "Math.copySign(magnitude, direction)",
        ),
        (
            &reference,
            "Nucleoid preserves sign copying argument dependencies",
            "magnitude = 3\ndirection = -1\ngate = 1\ncopied = Math.abs(magnitude, gate) * Math.sign(Math.sign(direction) + 1 / direction)\nassert(copied, -3)\ndirection = 1\nassert(copied, 3)\ngate = 2\nassert(copied, 3)\ndelete gate\nassert(copied, undefined)\ngate = 3\nassert(copied, 3)\ndirection = -1\nassert(copied, -3)\n1",
            "Math.abs(magnitude, gate) * Math.sign(Math.sign(direction) + 1 / direction)",
            "Math.copySign(magnitude, direction, gate)",
        ),
    ] {
        let matching = cases
            .iter()
            .filter(|case| case.title == title)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1);
        let case = matching[0];
        assert_eq!(cases.iter().rev().nth(4).unwrap().title, title);
        assert_eq!(case.expected.as_deref(), Some("1"));
        assert_eq!(accepted.matches(before).count(), 1);
        let candidate = accepted.replace(before, after);
        assert_eq!(case.source.trim(), format!("# {title}\n\n{candidate}"));
        let assertions = common::assertions::protected_assertions(&parse(accepted).unwrap());
        assert_eq!(assertions.len(), 6);
        assert_eq!(
            common::assertions::protected_assertions(&parse(&case.source).unwrap()),
            assertions
        );
        for source in [&case.source, &accepted.to_string()] {
            let mut runtime = Runtime::new();
            checked(&mut runtime, source, Some("1"));
            assert_eq!(runtime.assertions_run(), 6);
        }
    }
    #[derive(serde::Deserialize)]
    struct Example {
        code: String,
        #[serde(rename = "return")]
        expected: Option<String>,
    }
    let examples = include_str!("../dataset/nucleoid.synth.142.jsonl")
        .lines()
        .map(|line| serde_json::from_str::<Example>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(examples.len(), 4);
    for example in examples {
        checked(
            &mut Runtime::new(),
            &example.code,
            example.expected.as_deref(),
        );
    }
}

#[test]
fn every_argument_slot_keeps_subscriptions_deletion_and_live_null_recovery() {
    for (candidate, control, defined) in [
        (
            "Math.copySign(probe.input, -1)",
            "Math.abs(probe.input) * Math.sign(Math.sign(-1) + 1 / -1)",
            "3",
        ),
        (
            "Math.copySign(3, probe.input)",
            "Math.abs(3) * Math.sign(Math.sign(probe.input) + 1 / probe.input)",
            "-1",
        ),
        (
            "Math.copySign(3, -1, probe.input)",
            "Math.abs(3, probe.input) * Math.sign(Math.sign(-1) + 1 / -1)",
            "Number.NaN",
        ),
        (
            "Math.copySign(3, -1, 99, probe.input)",
            "Math.abs(3, 99, probe.input) * Math.sign(Math.sign(-1) + 1 / -1)",
            "{}",
        ),
    ] {
        for expression in [candidate, control] {
            let runtime = run(&format!(
                "class Probe\nprobe = Probe()\ncopied = {expression}\n\
                 assert(copied, null)\nprobe.input = {defined}\nassert(copied, -3)\n\
                 delete probe.input\nassert(copied, null)\n\
                 probe.input = {defined}\nassert(copied, -3)"
            ));
            assert_eq!(dependencies(&runtime, "copied"), ["probe", "probe.input"]);
            assert!(runtime.graph.reaches(
                &NodeKey::property(&ObjectId::named("probe"), "input"),
                &NodeKey::variable("copied"),
            ));
        }
    }
    for expression in [
        "Math.copySign(magnitude, direction, extra)",
        "Math.abs(magnitude, extra) * Math.sign(Math.sign(direction) + 1 / direction)",
    ] {
        let mut runtime = run(&format!(
            "magnitude = 3\ndirection = -1\nextra = 9\ncopied = {expression}\nassert(copied, -3)"
        ));
        for (name, restored) in [("magnitude", "3"), ("direction", "-1"), ("extra", "9")] {
            checked(
                &mut runtime,
                &format!(
                    "{name} = null\nassert(copied, null)\n{name} = {restored}\nassert(copied, -3)"
                ),
                None,
            );
        }
        assert!(
            runtime
                .graph
                .reaches(&NodeKey::variable("extra"), &NodeKey::variable("copied"),)
        );
    }
}

#[test]
fn wrappers_preserve_parameter_deferral_and_direct_null_calls_match_accepted_controls() {
    let mut runtime = run(r#"
extra = 9
def copied(amount: Numeric, sign: Numeric) => Math.copySign(amount, sign, extra)
def legacy(amount: Numeric, sign: Numeric) => Math.abs(amount, extra) * Math.sign(Math.sign(sign) + 1 / sign)
magnitude = 3
direction = -1
wrapped = copied(magnitude, direction)
assert(wrapped, -3)
omitted = copied()
parameter = copied(null, -1)
assert(omitted, null)
assert(parameter, null)
direction = 1
assert(wrapped, 3)
magnitude = -7
assert(wrapped, 7)
extra = null
assert(wrapped, null)
extra = Number.NaN
assert(wrapped, 7)
"#);
    for arguments in ["null, -1", "3, null", "", "3", "\"-3\", -0"] {
        let actual = checked(&mut runtime, &format!("copied({arguments})"), None);
        let expected = checked(&mut runtime, &format!("legacy({arguments})"), None);
        assert_same(&actual, &expected);
    }
    for name in ["copied", "magnitude", "direction", "extra"] {
        assert!(dependencies(&runtime, "wrapped").contains(&name), "{name}");
    }
    assert!(!runtime.graph.contains(&NodeKey::variable("Numeric")));
}

#[test]
fn eager_effects_and_strict_errors_continue_after_undefined_but_stop_on_failure() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 0
copied = Math.copySign(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3,
    log.order = log.order.value * 10 + 4
)
assert(log.order, 1234)
assert(copied, 1)
deferred = Math.copySign(log.input, log.order = -7, log.order = 8)
assert(deferred, null)
assert(log.order, 8)
log.input = 3
assert(deferred, -3)
assert(log.order, 8)
invalid = Math.copySign(3, Number.NaN, log.order = 6)
assert(Number.isNaN(invalid), true)
assert(log.order, 6)
extraMissing = Math.copySign(3, -1, log.absent, log.order = 5)
assert(extraMissing, null)
assert(log.order, 5)
log.order = 0
mapped = [3, -7].map(value => Math.copySign(value, -1, log.order = log.order.value * 10 + 1))
assert(mapped, [-3, -7])
assert(log.order, 11)
try:
    Math.copySign(log.order = 9, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 11)
try:
    Math.copySign(log.absent, Math.random(), missing)
catch error:
    assert(error, TypeError("Math.random is not supported"))
try:
    Math.copySign(log.absent, missing, Math.random())
catch error:
    assert(error, ReferenceError("missing is not defined"))
"#);
    for (source, kind, message) in [
        (
            "Math.copySign(missing, -1)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.copySign(3, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.copySign(3, -1, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.copySign(log.absent, -1, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.copySign(3, -1, log.absent, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.copySign(log.absent, Math.random())",
            ErrorKind::Type,
            "Math.random is not supported",
        ),
    ] {
        let keys = runtime.graph.keys().cloned().collect::<Vec<_>>();
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.graph.keys().cloned().collect::<Vec<_>>(), keys);
    }
    let error = runtime
        .run("log.order = 9\nMath.copySign(log.absent, missing)")
        .unwrap_err();
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    checked(&mut runtime, "assert(log.order, 11)", None);
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
}

#[test]
fn shadowing_captures_method_values_and_closed_receiver_dispatch_stay_ordinary() {
    let runtime = run(r#"
def Number(value) => 999
assert(Number("3"), 999)
magnitude = "-3"
direction = "-1"
copied = Math.copySign(magnitude, direction)
frozen = Math.copySign(magnitude.value, direction.value)
assert(copied, -3)
direction = "1"
assert(copied, 3)
assert(frozen, -3)
def intrinsic(Number):
    return Math.copySign("3", "-1")
assert(intrinsic(null), -3)
class Probe
capturedClass = Probe.value
assert(Number.isNaN(Math.copySign(capturedClass, -1)), true)
"#);
    assert_eq!(dependencies(&runtime, "copied"), ["magnitude", "direction"]);
    assert!(dependencies(&runtime, "frozen").is_empty());
    run(r#"
Number = 7
assert(Math.copySign("3", "-1"), -3)
reference = Math.copySign
indirect = reference(3, -1)
assert(reference, null)
assert(indirect, null)
record = {copySign: 7}
assert(record.copySign, 7)
class Namespace:
    def copySign(value, sign) => value + sign + 8
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.copySign(3, -1), 10)
def scoped(Math):
    return Math.copySign(3, -1)
assert(scoped(namespace), 10)
assert(scoped(null), null)
captured = -1
operation = value => Math.copySign(value, captured)
control = value => Math.abs(value) * Math.sign(Math.sign(captured) + 1 / captured)
assert(operation(3), control(3))
captured = 1
assert(operation(3), control(3))
Math = namespace
assert(Math.copySign(4, -1), 11)
"#);
    assert!(builtins::math_constant("copySign").is_none());
    assert_eq!(
        Runtime::new().run("Math.copySign").unwrap(),
        Value::Undefined
    );
    let mut runtime = run("Math = 7");
    let error = runtime.run("Math.copySign(3, -1)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.copySign is not a function");
    let mut runtime = run("items = [1]\nclass Population\nPopulation()");
    for (source, kind, message) in [
        (
            "items.copySign(-1)",
            ErrorKind::Type,
            "items.copySign is not a function",
        ),
        (
            "Population.copySign(-1)",
            ErrorKind::Type,
            "Population.copySign is not a function",
        ),
        (
            "copySign(3, -1)",
            ErrorKind::Reference,
            "copySign is not defined",
        ),
        (
            "Math.copysign(3, -1)",
            ErrorKind::Type,
            "Math.copysign is not a function",
        ),
        (
            "Math.toDegrees(3)",
            ErrorKind::Type,
            "Math.toDegrees is not a function",
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
}

#[test]
fn ordinary_call_ast_graph_source_and_caller_frames_remain_intact() {
    let runtime =
        run("magnitude = 3\ndirection = -1\ncopied = Math.copySign(magnitude, direction)");
    let node = runtime
        .graph
        .retrieve(&NodeKey::variable("copied"))
        .unwrap();
    let Some(Nuc::Variable(variable)) = &node.node else {
        panic!("the source must remain a variable rule");
    };
    assert_eq!(
        variable.value,
        parse_expression("Math.copySign(magnitude, direction)").unwrap()
    );
    let statements = parse("Math.copySign(magnitude, direction)").unwrap();
    let [Stmt::Expression(expression)] = statements.as_slice() else {
        panic!("the helper must remain an ordinary call");
    };
    let Ast::Call(call) = Ast::convert(expression) else {
        panic!("the helper introduces no lowering");
    };
    assert_eq!(
        call.function(),
        Some(&parse_expression("Math.copySign").unwrap())
    );
    assert_eq!(
        call.arguments(),
        [
            parse_expression("magnitude").unwrap(),
            parse_expression("direction").unwrap(),
        ]
    );
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.push();
    scope.declare("magnitude", Value::Number(0.0));
    scope.declare("direction", Value::Number(-1.0));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    assert_same(
        &call.resolve(&mut runtime, &mut scope).unwrap(),
        &Value::Number(-0.0),
    );
    let failed = parse_expression("Math.copySign(magnitude, direction, missing)").unwrap();
    let error = Ast::convert(&failed)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.retrieve("magnitude"), Some(&Value::Number(0.0)));
    assert_eq!(scope.retrieve("direction"), Some(&Value::Number(-1.0)));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
}

#[test]
fn rollback_restores_complete_snapshots_and_independent_class_rules_then_recovers() {
    let initial = r#"
class Signal(magnitude, direction):
    this.magnitude = magnitude
    this.direction = direction
    def signed() => Math.copySign(self.magnitude, self.direction)
$Signal.copied = Math.copySign($Signal.magnitude, $Signal.direction)
first = Signal(3, -1)
second = Signal(-7, 1)
magnitude = first.magnitude
direction = first.direction
extra = 9
alternative = 99
copied = Math.copySign(magnitude, direction, extra)
assert(copied, -3)
assert(first.copied, -3)
assert(second.copied, 7)
assert(first.signed(), -3)
"#;
    let mut actual = run(initial);
    let mut control = run(initial);
    let failures = [
        "copied = Math.copySign(alternative, direction, extra)\nfirst.direction = 1\nthird = Signal(9, -1)\n$Signal.copied = Math.copySign($Signal.magnitude, 1)\nthrow \"ROLLBACK\"",
        "copied = Math.copySign(alternative, direction, extra)\ncopied = Math.copySign(missing, direction)",
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
    for source in failures {
        control.run(source).unwrap_err();
    }
    for runtime in [&mut actual, &mut control] {
        checked(
            runtime,
            r#"
alternative = 5
assert(copied, -3)
delete extra
assert(copied, null)
extra = Number.NaN
assert(copied, -3)
first.direction = 1
assert(copied, 3)
assert(first.copied, 3)
assert(second.copied, 7)
delete first.magnitude
assert(copied, null)
assert(first.copied, null)
assert(second.copied, 7)
first.magnitude = -5
assert(copied, 5)
assert(first.copied, 5)
assert(first.signed(), 5)
third = Signal(9, -1)
assert(third.copied, -9)
copied
"#,
            Some("5"),
        );
        assert_eq!(
            dependencies(runtime, "copied"),
            ["magnitude", "direction", "extra"]
        );
        assert!(!runtime.graph.reaches(
            &NodeKey::variable("alternative"),
            &NodeKey::variable("copied"),
        ));
        assert!(runtime.graph.reaches(
            &NodeKey::property(&ObjectId::named("first"), "magnitude"),
            &NodeKey::variable("copied"),
        ));
    }
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
}
