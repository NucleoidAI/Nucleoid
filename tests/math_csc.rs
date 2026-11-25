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
            title: "Math.csc regression".to_string(),
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
fn numeric_bits_match_sine_then_reciprocal_without_pole_shortcuts() {
    let pi = std::f64::consts::PI;
    for input in [
        0.0,
        -0.0,
        f64::from_bits(1),
        -f64::from_bits(1),
        f64::from_bits(0x000f_ffff_ffff_ffff),
        -f64::from_bits(0x000f_ffff_ffff_ffff),
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        f64::MAX,
        f64::MIN,
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
        std::f64::consts::FRAC_PI_2,
        -std::f64::consts::FRAC_PI_2,
        pi,
        -pi,
        2.0 * pi,
        -2.0 * pi,
        f64::from_bits(pi.to_bits() - 1),
        f64::from_bits(pi.to_bits() + 1),
        -f64::from_bits(pi.to_bits() - 1),
        -f64::from_bits(pi.to_bits() + 1),
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        f64::from_bits(0xfff8_0000_0000_0141),
    ] {
        let arguments = [Value::Number(input)];
        let Value::Number(sine) = builtins::math("sin", &arguments).unwrap() else {
            panic!("the accepted sine must be numeric");
        };
        assert_numeric_same(
            &builtins::math("csc", &arguments).unwrap(),
            &Value::Number(1.0 / sine),
        );
    }
    for (input, expected) in [(0.0, f64::INFINITY), (-0.0, f64::NEG_INFINITY)] {
        assert_numeric_same(
            &builtins::math("csc", &[Value::Number(input)]).unwrap(),
            &Value::Number(expected),
        );
    }
    assert_numeric_same(
        &builtins::math("csc", &[]).unwrap(),
        &Value::Number(f64::NAN),
    );
}

#[test]
fn intrinsic_coercions_and_top_level_undefined_keep_the_unary_contract() {
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
        let actual = checked(&mut runtime, &format!("Math.csc({expression})"), None);
        let expected = checked(&mut runtime, &format!("1 / Math.sin({expression})"), None);
        assert_numeric_same(&actual, &expected);
    }
    for arguments in [
        vec![Value::Undefined],
        vec![Value::Number(1.0), Value::Undefined],
        vec![Value::String("invalid".to_string()), Value::Undefined],
        vec![Value::Number(f64::NAN), Value::Null, Value::Undefined],
    ] {
        for name in ["csc", "unknown", "random"] {
            assert_eq!(builtins::math(name, &arguments).unwrap(), Value::Undefined);
        }
    }
    let nested = Value::List(vec![Value::Undefined]);
    assert_numeric_same(
        &builtins::math("csc", std::slice::from_ref(&nested)).unwrap(),
        &Value::Number(f64::NAN),
    );
    assert_numeric_same(
        &builtins::math("csc", &[Value::Number(1.0), nested]).unwrap(),
        &Value::Number(1.0 / 1.0_f64.sin()),
    );
    run(r#"
assert(Math.csc(null), Number.POSITIVE_INFINITY)
assert(Math.csc(-0), Number.NEGATIVE_INFINITY)
assert(Number.isNaN(Math.csc()), true)
assert(Math.csc(1, Number.NaN, "invalid", {}, null), 1 / Math.sin(1))
assert(Math.csc(-1, [1, 2], Date(99), /probe/, value => value), 1 / Math.sin(-1))
"#);
    #[derive(serde::Deserialize)]
    struct Example {
        code: String,
        #[serde(rename = "return")]
        expected: Option<String>,
    }
    for line in include_str!("../dataset/nucleoid.synth.141.jsonl").lines() {
        let example: Example = serde_json::from_str(line).unwrap();
        checked(
            &mut Runtime::new(),
            &example.code,
            example.expected.as_deref(),
        );
    }
}

#[test]
fn first_and_ignored_reads_and_parameter_null_deferral_recover_dependencies() {
    for prefix in ["Math.csc", "1 / Math.sin"] {
        for (arguments, defined) in [
            ("probe.input", "Math.PI / 2"),
            ("Math.PI / 2, probe.input", "{}"),
            ("Math.PI / 2, 99, probe.input", "Number.NaN"),
            ("Math.PI / 2, 99, \"invalid\", probe.input", "[1, 2]"),
        ] {
            let runtime = run(&format!(
                "class Probe\nprobe = Probe()\nratio = {prefix}({arguments})\n\
                 assert(ratio, null)\nprobe.input = {defined}\nassert(ratio, 1)\n\
                 delete probe.input\nassert(ratio, null)\n\
                 probe.input = {defined}\nassert(ratio, 1)"
            ));
            assert_eq!(dependencies(&runtime, "ratio"), ["probe", "probe.input"]);
            assert!(runtime.graph.reaches(
                &NodeKey::property(&ObjectId::named("probe"), "input"),
                &NodeKey::variable("ratio"),
            ));
        }
    }
    let runtime = run(r#"
angle = Math.PI / 2
extra = 9
ratio = Math.csc(angle, extra)
assert(ratio, 1)
angle = null
assert(ratio, null)
angle = -Math.PI / 2
assert(ratio, -1)
extra = null
assert(ratio, null)
extra = Number.NaN
assert(ratio, -1)
def cosecant amount: Numeric => Math.csc(amount, extra)
wrapped = cosecant(angle)
omitted = cosecant()
parameter = cosecant(null)
assert(omitted, null)
assert(parameter, null)
assert(wrapped, -1)
angle = Math.PI / 2
assert(wrapped, 1)
class Probe
probe = Probe()
invalid = Math.csc("invalid", probe.optional)
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
    for name in ["cosecant", "angle", "extra"] {
        assert!(dependencies(&runtime, "wrapped").contains(&name), "{name}");
    }
    assert!(!runtime.graph.contains(&NodeKey::variable("Numeric")));
}

#[test]
fn eager_arguments_run_once_left_to_right_after_undefined_and_keep_errors() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 0
ratio = Math.csc(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3
)
assert(log.order, 123)
assert(ratio, 1 / Math.sin(1))
deferred = Math.csc(log.input, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.input = Math.PI / 2
assert(deferred, 1)
assert(log.order, 7)
log.order = 0
mapped = [1, -1].map(value => Math.csc(value, log.order = log.order.value * 10 + 1))
assert(mapped, [1 / Math.sin(1), 1 / Math.sin(-1)])
assert(log.order, 11)
try:
    Math.csc(log.order = 9, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 11)
try:
    Math.csc(log.absent, Math.random(), missing)
catch error:
    assert(error, TypeError("Math.random is not supported"))
try:
    Math.csc(log.absent, missing, Math.random())
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(Math.csc(log.absent), null)
assert(Math.unknown(log.absent), null)
assert(Math.random(log.absent), null)
"#);
    for (source, kind, message) in [
        (
            "Math.csc(missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.csc(1, 99, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.csc(log.absent, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.csc(log.absent, Math.random())",
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
        .run("log.order = 9\nMath.csc(log.absent, missing)")
        .unwrap_err();
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    checked(&mut runtime, "assert(log.order, 11)", None);
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
}

#[test]
fn shadowing_captures_value_reads_ast_frames_and_dispatch_remain_ordinary() {
    let runtime = run(r#"
def Number(value) => 999
assert(Number("1"), 999)
text = "0"
ratio = Math.csc(text)
assert(ratio, 1 / Math.sin(0))
text = "1"
assert(ratio, 1 / Math.sin(1))
frozen = Math.csc(text.value)
text = "0"
assert(ratio, 1 / Math.sin(0))
assert(frozen, 1 / Math.sin(1))
def intrinsic(Number):
    return Math.csc("1")
assert(intrinsic(null), 1 / Math.sin(1))
"#);
    assert_eq!(dependencies(&runtime, "ratio"), ["text"]);
    assert!(dependencies(&runtime, "frozen").is_empty());
    let node = runtime.graph.retrieve(&NodeKey::variable("ratio")).unwrap();
    let Some(Nuc::Variable(variable)) = &node.node else {
        panic!("the written source must remain the graph rule");
    };
    assert_eq!(variable.value, parse_expression("Math.csc(text)").unwrap());
    run(r#"
Number = 7
assert(Math.csc("1"), 1 / Math.sin(1))
reference = Math.csc
indirect = reference(1)
assert(reference, null)
assert(indirect, null)
record = {csc: 7}
assert(record.csc, 7)
class Namespace:
    def csc value => value + 8
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.csc(2), 10)
def scoped(Math):
    return Math.csc(3)
assert(scoped(namespace), 11)
assert(scoped(null), null)
captured = 0
operation = value => Math.csc(value + captured)
control = value => 1 / Math.sin(value + captured)
assert(operation(1), control(1))
captured = 1
assert(operation(1), control(1))
Math = namespace
assert(Math.csc(4), 12)
"#);
    assert!(builtins::math_constant("csc").is_none());
    let mut native = Runtime::new();
    assert_eq!(native.run("Math.csc").unwrap(), Value::Undefined);
    let mut runtime = run("Math = 7");
    let error = runtime.run("Math.csc(1)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.csc is not a function");
    let mut runtime = run("items = [1]\nclass Population\nPopulation()");
    for (source, kind, message) in [
        (
            "items.csc()",
            ErrorKind::Type,
            "items.csc is not a function",
        ),
        (
            "Population.csc()",
            ErrorKind::Type,
            "Population.csc is not a function",
        ),
        ("csc(1)", ErrorKind::Reference, "csc is not defined"),
        (
            "Math.csch(1)",
            ErrorKind::Type,
            "Math.csch is not a function",
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
    let statements = parse("Math.csc(input)").unwrap();
    let [Stmt::Expression(expression)] = statements.as_slice() else {
        panic!("the helper must remain an ordinary call");
    };
    let Ast::Call(call) = Ast::convert(expression) else {
        panic!("the helper introduces no lowering");
    };
    assert_eq!(
        call.function(),
        Some(&parse_expression("Math.csc").unwrap())
    );
    assert_eq!(call.arguments(), [parse_expression("input").unwrap()]);
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.push();
    scope.declare("input", Value::Number(std::f64::consts::FRAC_PI_2));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    assert_numeric_same(
        &call.resolve(&mut runtime, &mut scope).unwrap(),
        &Value::Number(1.0),
    );
    let failed = parse_expression("Math.csc(input, missing)").unwrap();
    let error = Ast::convert(&failed)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(
        scope.retrieve("input"),
        Some(&Value::Number(std::f64::consts::FRAC_PI_2))
    );
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
}

#[test]
fn rollback_restores_complete_state_and_graph_with_independent_class_rules() {
    let initial = r#"
class Direction(angle):
    this.angle = angle
    def scale() => Math.csc(self.angle)
$Direction.ratio = Math.csc($Direction.angle)
first = Direction(Math.PI / 2)
second = Direction(-Math.PI / 2)
angle = first.angle
alternative = 0.5
ratio = Math.csc(angle)
assert(ratio, 1)
assert(first.ratio, 1)
assert(second.ratio, -1)
assert(first.scale(), 1)
"#;
    let mut actual = run(initial);
    let mut control = run(initial);
    let failures = [
        "ratio = Math.csc(alternative)\nfirst.angle = 1\nthrow \"ROLLBACK\"",
        "ratio = Math.csc(alternative)\nratio = Math.csc(missing)",
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
assert(ratio, 1)
delete first.angle
assert(ratio, null)
assert(first.ratio, null)
assert(second.ratio, -1)
first.angle = -Math.PI / 2
assert(ratio, -1)
assert(first.ratio, -1)
assert(first.scale(), -1)
assert(second.ratio, -1)
third = Direction(Math.PI / 2)
assert(third.ratio, 1)
ratio
"#,
            Some("-1"),
        );
        assert_eq!(dependencies(runtime, "ratio"), ["angle"]);
        assert!(!runtime.graph.reaches(
            &NodeKey::variable("alternative"),
            &NodeKey::variable("ratio"),
        ));
        assert!(runtime.graph.reaches(
            &NodeKey::property(&ObjectId::named("first"), "angle"),
            &NodeKey::variable("ratio"),
        ));
    }
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
}
