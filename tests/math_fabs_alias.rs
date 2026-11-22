mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use nucleoid::builtins;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{Ast, parse, parse_expression};
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "Math.fabs alias regression".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count);
    value
}

fn number_bits(value: &Value) -> u64 {
    let Value::Number(number) = value else {
        panic!("a numeric result is required: {value:?}");
    };
    number.to_bits()
}

#[test]
fn fabs_and_abs_preserve_absolute_value_bits_including_nan_payloads() {
    const MAGNITUDE_BITS: u64 = 0x7fff_ffff_ffff_ffff;
    for bits in [
        0,
        0x8000_0000_0000_0000,
        1,
        0x8000_0000_0000_0001,
        0x000f_ffff_ffff_ffff,
        0x800f_ffff_ffff_ffff,
        0x0010_0000_0000_0000,
        0x8010_0000_0000_0000,
        3.5_f64.to_bits(),
        (-3.5_f64).to_bits(),
        f64::MAX.to_bits(),
        (-f64::MAX).to_bits(),
        f64::INFINITY.to_bits(),
        f64::NEG_INFINITY.to_bits(),
        0x7ff8_0000_0000_0123,
        0xfff8_0000_0000_0123,
        0x7ff0_0000_0000_0456,
        0xfff0_0000_0000_0456,
    ] {
        let input = [Value::Number(f64::from_bits(bits))];
        let canonical = builtins::math("abs", &input).unwrap();
        let alias = builtins::math("fabs", &input).unwrap();
        assert_eq!(number_bits(&canonical), bits & MAGNITUDE_BITS);
        assert_eq!(number_bits(&alias), number_bits(&canonical));
    }
}

#[test]
fn intrinsic_coercions_omission_and_all_supplied_undefined_values_keep_the_abs_gate() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, "class Probe\nprobe = Probe()", None);
    for source in [
        "null",
        "true",
        "false",
        "\"\"",
        "\" -3.5 \"",
        "\"invalid\"",
        "[]",
        "[\"-2\"]",
        "[1, 2]",
        "Date(-9)",
        "{}",
        "Probe",
        "/probe/",
        "value -> value",
        "Number.NaN",
        "Number.NEGATIVE_INFINITY",
    ] {
        let value = checked(&mut runtime, source, None);
        let input = std::slice::from_ref(&value);
        assert_eq!(
            number_bits(&builtins::math("fabs", input).unwrap()),
            number_bits(&builtins::math("abs", input).unwrap()),
            "{source}"
        );
        assert_eq!(
            builtins::math("fabs", &[Value::number(-3), value]).unwrap(),
            Value::number(3),
            "an ignored extra must not be converted: {source}"
        );
    }
    assert_eq!(
        number_bits(&builtins::math("fabs", &[]).unwrap()),
        number_bits(&builtins::math("abs", &[]).unwrap())
    );
    assert!(builtins::math("fabs", &[]).unwrap().to_number().is_nan());
    for arguments in [
        vec![Value::Undefined],
        vec![Value::number(-3), Value::Undefined],
        vec![Value::string("invalid"), Value::Undefined],
        vec![Value::Number(f64::NAN), Value::Null, Value::Undefined],
    ] {
        for name in ["fabs", "abs", "unknown"] {
            assert_eq!(builtins::math(name, &arguments).unwrap(), Value::Undefined);
        }
    }
    let nested = [Value::List(vec![Value::Undefined])];
    assert_eq!(
        number_bits(&builtins::math("fabs", &nested).unwrap()),
        number_bits(&builtins::math("abs", &nested).unwrap())
    );
}

#[test]
fn eager_argument_snapshots_errors_and_effect_rollback_match_abs() {
    for method in ["fabs", "abs"] {
        let mut runtime = Runtime::new();
        checked(
            &mut runtime,
            &format!(
                r#"
class Log
log = Log()
log.order = 0
result = Math.{method}(log.order = 1, log.order = log.order.value * 10 + 2, log.order = log.order.value * 10 + 3)
assert([result, log.order], [1, 123])
deferred = Math.{method}(log.input, log.order = 7)
assert([deferred, log.order], [null, 7])
log.input = -3
assert([deferred, log.order], [3, 7])
log.order = 0
mapped = [-3, -2, 1].map(value -> Math.{method}(value, log.order = log.order.value * 10 + 1))
assert([mapped, log.order], [[3, 2, 1], 111])
try:
    Math.{method}(log.order = 9, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 111)
try:
    Math.{method}(log.absent, Math.random(), missing)
catch error:
    assert(error, TypeError("Math.random is not supported"))
try:
    Math.{method}(log.absent, missing, Math.random())
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(Math.unknown(log.absent), null)
"#
            ),
            None,
        );
        for (arguments, kind, message) in [
            ("missing", ErrorKind::Reference, "missing is not defined"),
            (
                "-3, missing",
                ErrorKind::Reference,
                "missing is not defined",
            ),
            (
                "log.absent, Math.random()",
                ErrorKind::Type,
                "Math.random is not supported",
            ),
        ] {
            let keys = runtime.graph.keys().cloned().collect::<Vec<_>>();
            let error = runtime
                .run(&format!("Math.{method}({arguments})"))
                .unwrap_err();
            assert_eq!(error.kind(), kind);
            assert_eq!(error.message(), message);
            assert_eq!(error.position(), Some(Position::new(1, 1)));
            assert_eq!(runtime.graph.keys().cloned().collect::<Vec<_>>(), keys);
        }
        let error = runtime
            .run(&format!(
                "log.order = 9\nMath.{method}(log.absent, missing)"
            ))
            .unwrap_err();
        assert_eq!(error.position(), Some(Position::new(2, 1)));
        checked(&mut runtime, "assert(log.order, 111)", None);
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
}

#[test]
fn named_and_class_rules_keep_dependencies_deferral_recovery_and_failed_replacements() {
    for method in ["fabs", "abs"] {
        let initial = format!(
            r#"
class Reading(input):
    this.input = input
$Reading.magnitude = Math.{method}($Reading.input)
first = Reading(-3)
second = Reading(2)
extra = 9
alternative = -7
def magnitude input: Numeric => Math.{method}(input, extra)
result = magnitude(first.input)
assert([result, first.magnitude, second.magnitude], [3, 3, 2])
"#
        );
        let mut actual = Runtime::new();
        let mut control = Runtime::new();
        for runtime in [&mut actual, &mut control] {
            checked(runtime, &initial, None);
        }
        let failures = [
            format!("result = Math.{method}(alternative)\nfirst.input = -10\nthrow \"ROLLBACK\""),
            format!("result = Math.{method}(alternative)\nresult = Math.{method}(missing)"),
        ];
        for source in &failures {
            actual.run(source).unwrap_err();
            runtime_equivalence::assert_equivalent(&actual, None, &control, None);
        }
        for source in &failures {
            control.run(source).unwrap_err();
        }
        for runtime in [&mut actual, &mut control] {
            checked(
                runtime,
                r#"
extra = null
assert(result, null)
extra = 1
assert(result, 3)
delete first.input
assert([result, first.magnitude, second.magnitude], [null, null, 2])
first.input = -4
assert([result, first.magnitude, second.magnitude], [4, 4, 2])
def magnitude input: Numeric => 2 * Math.abs(input, extra)
assert(result, 8)
result
"#,
                Some("8"),
            );
            for input in [
                NodeKey::function("magnitude"),
                NodeKey::property(&ObjectId::named("first"), "input"),
                NodeKey::variable("extra"),
            ] {
                assert!(runtime.graph.reaches(&input, &NodeKey::variable("result")));
            }
            assert!(!runtime.graph.reaches(
                &NodeKey::variable("alternative"),
                &NodeKey::variable("result")
            ));
        }
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    }
}

#[test]
fn native_value_reads_shadowing_ordinary_members_and_other_receivers_keep_routing() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
def fabs(value) => value + 10
def Number(value) => 999
assert([fabs(3), Number("3")], [13, 999])
assert(Math.fabs(" -3 "), 3)
native = Math
assert(native.fabs(-3), native.abs(-3))
def absolute(namespace, value) => namespace.fabs(value)
assert(absolute(Math, -3), 3)
log = Object()
log.count = 0
reference = Math.fabs
assert([Math.abs, Math.fabs, reference], [null, null, null])
reference(log.count = 1)
none = null
none.fabs(log.count = 2)
log.absent.fabs(log.count = 3)
assert(log.count, 0)
record = {fabs: 7}
callable = {fabs: value -> value + 5}
assert([record.fabs, callable.fabs(3)], [7, 8])
class Namespace:
    def fabs(value) => value + 8
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.fabs(2), 10)
def scoped(Math) => Math.fabs(3)
assert([scoped(namespace), scoped(null)], [11, null])
Math = namespace
assert(Math.fabs(4), 12)
"#,
        None,
    );
    let mut runtime = Runtime::new();
    checked(&mut runtime, "class Probe", None);
    for (receiver, label) in [
        ("[-3]", "[-3]"),
        ("Probe", "Probe"),
        ("(-3)", "number"),
        ("\"abc\"", "abc"),
    ] {
        for method in ["fabs", "abs"] {
            let error = runtime.run(&format!("{receiver}.{method}()")).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Type);
            assert_eq!(
                error.message(),
                format!("{label}.{method} is not a function")
            );
            assert_eq!(error.position(), Some(Position::new(1, 1)));
        }
    }
    checked(&mut runtime, "Math = 7", None);
    let error = runtime.run("Math.fabs(1)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.fabs is not a function");
}

#[test]
fn written_calls_graph_nodes_and_scope_frames_preserve_the_alias_without_forwarding() {
    let source = "input = -3\nextra = 9\nresult = Math.fabs(input, extra)";
    assert_eq!(
        generate_all(&parse(source).unwrap()),
        "input=-3;extra=9;result=Math.fabs(input,extra)"
    );
    let mut runtime = Runtime::new();
    checked(&mut runtime, source, Some("3"));
    let node = runtime
        .graph
        .retrieve(&NodeKey::variable("result"))
        .unwrap();
    assert_eq!(
        node.dependencies
            .iter()
            .map(NodeKey::as_str)
            .collect::<Vec<_>>(),
        ["input", "extra"]
    );
    let Some(Nuc::Variable(variable)) = &node.node else {
        panic!("the call remains a variable expression");
    };
    assert_eq!(variable.value.to_string(), "Math.fabs(input,extra)");
    checked(
        &mut runtime,
        r#"
Number = 7
input = "-4"
assert(result, 4)
def intrinsic(Number) => Math.fabs(" -2 ")
assert(intrinsic(null), 2)
"#,
        None,
    );
    let mut scope = Scope::new();
    scope.push();
    scope.declare("input", Value::number(-5));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    let expression = parse_expression("Math.fabs(input)").unwrap();
    assert_eq!(
        Ast::convert(&expression)
            .resolve(&mut runtime, &mut scope)
            .unwrap(),
        Value::number(5)
    );
    let expression = parse_expression("Math.fabs(input, missing)").unwrap();
    let error = Ast::convert(&expression)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.retrieve("input"), Some(&Value::number(-5)));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
}
