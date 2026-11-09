mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use nucleoid::builtins;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Ast, Stmt, parse, parse_expression};
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "Math.cot regression".to_string(),
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

fn assert_numeric_same(actual: &Value, expected: &Value) {
    let (Value::Number(actual), Value::Number(expected)) = (actual, expected) else {
        panic!("numeric controls required: {actual:?}, {expected:?}");
    };
    if expected.is_nan() {
        assert!(actual.is_nan(), "{actual} is not NaN");
    } else {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
}

#[test]
fn numeric_bits_match_accepted_tangent_then_reciprocal_without_rearrangement() {
    let half_pi = std::f64::consts::FRAC_PI_2;
    for input in [
        0.0,
        -0.0,
        f64::from_bits(1),
        -f64::from_bits(1),
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        f64::MAX,
        -f64::MAX,
        0.1,
        -0.1,
        0.5,
        -0.5,
        1.0,
        -1.0,
        1.0e-200,
        -1.0e-200,
        1.0e200,
        -1.0e200,
        std::f64::consts::PI,
        -std::f64::consts::PI,
        half_pi,
        -half_pi,
        f64::from_bits(half_pi.to_bits() - 1),
        f64::from_bits(half_pi.to_bits() + 1),
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ] {
        let arguments = [Value::Number(input)];
        let Value::Number(tangent) = builtins::math("tan", &arguments).unwrap() else {
            panic!("the accepted tangent must be numeric");
        };
        assert_numeric_same(
            &builtins::math("cot", &arguments).unwrap(),
            &Value::Number(1.0 / tangent),
        );
    }
    assert_numeric_same(
        &builtins::math("cot", &[Value::Number(0.0)]).unwrap(),
        &Value::Number(f64::INFINITY),
    );
    assert_numeric_same(
        &builtins::math("cot", &[Value::Number(-0.0)]).unwrap(),
        &Value::Number(f64::NEG_INFINITY),
    );
    assert_numeric_same(
        &builtins::math("cot", &[]).unwrap(),
        &Value::Number(f64::NAN),
    );
}

#[test]
fn intrinsic_coercion_and_undefined_gate_preserve_accepted_numeric_controls() {
    for expression in [
        "null",
        "true",
        "false",
        "\"\"",
        "\" 1 \"",
        "\"invalid\"",
        "[]",
        "[\"0.5\"]",
        "[1, 2]",
        "Date(-1)",
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
        let actual = checked(&mut runtime, &format!("Math.cot({expression})"), None);
        let expected = checked(&mut runtime, &format!("1 / Math.tan({expression})"), None);
        assert_numeric_same(&actual, &expected);
    }
    for arguments in [
        vec![Value::Undefined],
        vec![Value::Number(1.0), Value::Undefined],
        vec![Value::String("invalid".to_string()), Value::Undefined],
        vec![Value::Number(f64::NAN), Value::Null, Value::Undefined],
    ] {
        assert_eq!(builtins::math("cot", &arguments).unwrap(), Value::Undefined);
    }
    run(r#"
assert(Math.cot(null), Number.POSITIVE_INFINITY)
assert(Number.isNaN(Math.cot()), true)
assert(Math.cot(1, Number.NaN, "invalid", {}, null), 1 / Math.tan(1))
assert(Math.cot(-1, [1, 2], Date(99), /probe/, value => value), 1 / Math.tan(-1))
"#);
}

#[test]
fn first_and_ignored_extra_reads_keep_deletion_deferral_and_recovery() {
    for prefix in ["Math.cot", "1 / Math.tan"] {
        for (arguments, defined) in [
            ("probe.input", "1"),
            ("1, probe.input", "{}"),
            ("1, 99, probe.input", "Number.NaN"),
            ("1, 99, \"invalid\", probe.input", "[1, 2]"),
        ] {
            let runtime = run(&format!(
                "class Probe\nprobe = Probe()\nratio = {prefix}({arguments})\n\
                 assert(ratio, null)\nprobe.input = {defined}\nassert(ratio, 1 / Math.tan(1))\n\
                 delete probe.input\nassert(ratio, null)\n\
                 probe.input = {defined}\nassert(ratio, 1 / Math.tan(1))"
            ));
            assert!(runtime.graph.reaches(
                &NodeKey::property(&ObjectId::named("probe"), "input"),
                &NodeKey::variable("ratio"),
            ));
            assert_eq!(
                runtime
                    .graph
                    .retrieve(&NodeKey::variable("ratio"))
                    .unwrap()
                    .dependencies
                    .iter()
                    .map(NodeKey::as_str)
                    .collect::<Vec<_>>(),
                ["probe", "probe.input"],
            );
        }
    }
    let runtime = run(r#"
angle = 1
extra = 9
ratio = Math.cot(angle, extra)
assert(ratio, 1 / Math.tan(1))
angle = null
assert(ratio, null)
angle = -1
assert(ratio, 1 / Math.tan(-1))
extra = null
assert(ratio, null)
extra = Number.NaN
assert(ratio, 1 / Math.tan(-1))
def cotangent input: Numeric => Math.cot(input)
omitted = cotangent()
parameter = cotangent(null)
assert(omitted, null)
assert(parameter, null)
assert(cotangent(1, 99), 1 / Math.tan(1))
class Probe
probe = Probe()
invalid = Math.cot("invalid", probe.optional)
assert(invalid, null)
probe.optional = 1
assert(Number.isNaN(invalid), true)
"#);
    for name in ["angle", "extra"] {
        assert!(
            runtime
                .graph
                .reaches(&NodeKey::variable(name), &NodeKey::variable("ratio"),)
        );
    }
}

#[test]
fn all_expressions_run_once_left_to_right_before_gating_and_strict_errors() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 0
ratio = Math.cot(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3
)
assert(log.order, 123)
assert(ratio, 1 / Math.tan(1))
deferred = Math.cot(log.input, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.input = 1
assert(deferred, 1 / Math.tan(1))
assert(log.order, 7)
log.order = 0
mapped = [0, 1, -1].map(value => Math.cot(value, log.order = log.order.value * 10 + 1))
assert(mapped, [Number.POSITIVE_INFINITY, 1 / Math.tan(1), 1 / Math.tan(-1)])
assert(log.order, 111)
try:
    Math.cot(log.order = 9, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 111)
try:
    Math.cot(log.absent, Math.random(), missing)
catch error:
    assert(error, TypeError("Math.random is not supported"))
try:
    Math.cot(log.absent, missing, Math.random())
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(Math.unknown(log.absent), null)
assert(Math.random(log.absent), null)
"#);
    for (source, kind, message) in [
        (
            "Math.cot(missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.cot(1, 99, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.cot(log.absent, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.cot(log.absent, Math.random())",
            ErrorKind::Type,
            "Math.random is not supported",
        ),
        (
            "Math.unknown()",
            ErrorKind::Type,
            "Math.unknown is not a function",
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
        .run("log.order = 9\nMath.cot(log.absent, missing)")
        .unwrap_err();
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    checked(&mut runtime, "assert(log.order, 111)", None);
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
}

#[test]
fn namespace_shadowing_member_lookup_and_unsupported_value_reads_stay_ordinary() {
    let runtime = run(r#"
def Number(value) => 999
assert(Number("1"), 999)
text = "1"
ratio = Math.cot(text)
assert(ratio, 1 / Math.tan(1))
text = "-1"
assert(ratio, 1 / Math.tan(-1))
def intrinsic(Number):
    return Math.cot("1")
assert(intrinsic(null), 1 / Math.tan(1))
"#);
    assert_eq!(
        runtime
            .graph
            .retrieve(&NodeKey::variable("ratio"))
            .unwrap()
            .dependencies
            .iter()
            .map(NodeKey::as_str)
            .collect::<Vec<_>>(),
        ["text"],
    );
    run(r#"
Number = 7
assert(Math.cot("1"), 1 / Math.tan(1))
reference = Math.cot
indirect = reference(1)
assert(reference, null)
assert(indirect, null)
record = {cot: 7}
assert(record.cot, 7)
class Namespace:
    def cot value => value + 8
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(namespace.cot(2), 10)
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.cot(2), 10)
def scoped(Math):
    return Math.cot(3)
assert(scoped(namespace), 11)
assert(scoped(null), null)
Math = namespace
assert(Math.cot(4), 12)
"#);
    let mut runtime = run("Math = 7");
    let error = runtime.run("Math.cot(1)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.cot is not a function");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn wrappers_receivers_and_direct_calls_preserve_dependencies_and_scope_frames() {
    let runtime = run(r#"
input = 1
extra = 99
def cotangent amount: Numeric => Math.cot(amount, extra)
ratio = cotangent(input)
assert(ratio, 1 / Math.tan(1))
input = -1
assert(ratio, 1 / Math.tan(-1))
class Reading(angle):
    this.angle = angle
    def ratio() => Math.cot(self.angle)
reading = Reading(1)
assert(reading.ratio(), 1 / Math.tan(1))
reading.angle = -1
assert(reading.ratio(), 1 / Math.tan(-1))
"#);
    let node = runtime.graph.retrieve(&NodeKey::variable("ratio")).unwrap();
    for name in ["cotangent", "input", "extra"] {
        assert!(node.dependencies.contains(&NodeKey::variable(name)));
    }
    let statements = parse("Math.cot(input)").unwrap();
    let [Stmt::Expression(expression)] = statements.as_slice() else {
        panic!("the intrinsic must remain an ordinary call");
    };
    let Ast::Call(call) = Ast::convert(expression) else {
        panic!("no new AST representation");
    };
    assert_eq!(call.arguments().len(), 1);
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.push();
    scope.declare("input", Value::number(1));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    let expected = checked(&mut runtime, "1 / Math.tan(1)", None);
    assert_numeric_same(&call.resolve(&mut runtime, &mut scope).unwrap(), &expected);
    let failed = parse_expression("Math.cot(input, missing)").unwrap();
    let error = Ast::convert(&failed)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.retrieve("input"), Some(&Value::number(1)));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
}

#[test]
fn failed_replacements_restore_state_graph_and_independent_class_rules() {
    let initial = r#"
class Direction(angle):
    this.angle = angle
$Direction.ratio = Math.cot($Direction.angle)
first = Direction(1)
second = Direction(-1)
angle = first.angle
alternative = 0.5
ratio = Math.cot(angle)
assert(ratio, 1 / Math.tan(1))
assert(first.ratio, 1 / Math.tan(1))
assert(second.ratio, 1 / Math.tan(-1))
"#;
    let mut actual = run(initial);
    let mut control = run(initial);
    let failures = [
        "ratio = Math.cot(alternative)\nfirst.angle = 0\nthrow \"ROLLBACK\"",
        "ratio = Math.cot(alternative)\nratio = Math.cot(missing)",
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
alternative = -1
assert(ratio, 1 / Math.tan(1))
delete first.angle
assert(ratio, null)
assert(first.ratio, null)
assert(second.ratio, 1 / Math.tan(-1))
first.angle = 0.5
assert(ratio, 1 / Math.tan(0.5))
assert(first.ratio, 1 / Math.tan(0.5))
assert(second.ratio, 1 / Math.tan(-1))
ratio == 1 / Math.tan(0.5)
"#,
            Some("true"),
        );
        assert!(!runtime.graph.reaches(
            &NodeKey::variable("alternative"),
            &NodeKey::variable("ratio"),
        ));
    }
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
}
