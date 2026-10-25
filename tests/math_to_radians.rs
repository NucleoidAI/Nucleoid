//! Intrinsic degree conversion beside independent accepted arithmetic.
//! Ordinary controls are sealed here; no Git or session artifact is required.

mod common;
#[path = "support/radian_cases.rs"]
mod radian_cases;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::collections::HashMap;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Ast, Stmt, parse, parse_expression};
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

use radian_cases::{SEALED, assert_numeric_same};

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "Math.toRadians regression".to_string(),
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
fn nine_sealed_cases_preserve_sources_assertions_return_and_twenty_one_rhs_rewrites() {
    let documents = radian_cases::documents();
    let mut previous = HashMap::new();
    let mut totals = (0, 0, 0);
    assert_eq!(SEALED.len(), 9);
    for sealed in SEALED {
        let cases = &documents
            .iter()
            .find(|(document, _)| *document == sealed.document)
            .unwrap()
            .1;
        let matching = cases
            .iter()
            .enumerate()
            .filter(|(_, case)| case.title == sealed.title)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "{}", sealed.title);
        let (position, case) = matching[0];
        if let Some(before) = previous.insert(sealed.document, position) {
            assert!(before < position, "{}: relative order", sealed.title);
        }
        assert_eq!(
            case.expected.as_deref(),
            sealed.expected,
            "{}",
            sealed.title
        );
        let accepted = radian_cases::documented_source(sealed);
        let (candidate, edits) = radian_cases::rewrite(&accepted);
        assert_eq!(edits, sealed.rewrites, "{}", sealed.title);
        assert_eq!(candidate, case.source.trim(), "{}", sealed.title);
        assert_eq!(candidate.matches("Math.toRadians(").count(), edits);
        let old = parse(&accepted).unwrap();
        let new = parse(&case.source).unwrap();
        assert_ne!(new, old, "new RHS call ASTs intentionally differ");
        let assertions = common::assertions::protected_assertions(&old);
        assert_eq!(assertions.len(), sealed.assertions);
        assert_eq!(common::assertions::protected_assertions(&new), assertions);
        let original = common::Case {
            title: sealed.title.to_string(),
            source: sealed.accepted.to_string(),
            expected: sealed.expected.map(str::to_string),
        };
        let mut runtimes = [Runtime::new(), Runtime::new()];
        let mut results = Vec::new();
        for (runtime, source) in runtimes.iter_mut().zip([case, &original]) {
            results.push(
                common::check_with_count(runtime, source, sealed.assertions)
                    .unwrap_or_else(|reason| panic!("{}: {reason}", sealed.title)),
            );
            assert_eq!(runtime.assertions_run(), sealed.assertions);
        }
        if sealed.expected.is_some() {
            assert_numeric_same(&results[0], &results[1]);
        }
        radian_cases::assert_observations(&runtimes[0], &runtimes[1], sealed);
        totals.0 += sealed.assertions;
        totals.1 += usize::from(sealed.expected.is_some());
        totals.2 += edits;
    }
    assert_eq!(totals, (55, 1, 21));
}

#[test]
fn reactive_coercion_extremes_subnormal_and_zero_bits_match_division_first_arithmetic() {
    let template = "value = -0\nconverted = __RADIANS__\nreciprocal = 1 / converted";
    let mut runtimes = ["Math.toRadians(value)", "value * (Math.PI / 180)"]
        .map(|expression| run(&template.replace("__RADIANS__", expression)));
    for source in [
        "assert(1 / converted, Number.NEGATIVE_INFINITY)",
        "value = 0\nassert(1 / converted, Number.POSITIVE_INFINITY)",
        "value = -0\nassert(1 / converted, Number.NEGATIVE_INFINITY)",
        "value = 5e-324\nassert(converted, 0)",
        "value = -5e-324\nassert(1 / converted, Number.NEGATIVE_INFINITY)",
        "value = Number.MIN_VALUE\nassert(converted > 0, true)",
        "value = -Number.MIN_VALUE\nassert(converted < 0, true)",
        "value = Number.MAX_VALUE\nassert(Number.isFinite(converted), true)",
        "value = -Number.MAX_VALUE\nassert(Number.isFinite(converted), true)",
        "value = Number.POSITIVE_INFINITY\nassert(converted, Number.POSITIVE_INFINITY)",
        "value = Number.NEGATIVE_INFINITY\nassert(converted, Number.NEGATIVE_INFINITY)",
        "value = Number.NaN\nassert(Number.isNaN(converted), true)",
        "value = \" 180 \"\nassert(converted, Math.PI)",
        "value = true\nassert(converted, Math.PI / 180)",
        "value = false\nassert(converted, 0)",
        "value = []\nassert(converted, 0)",
        "value = [\"90\"]\nassert(converted, Math.PI / 2)",
        "value = Date(180)\nassert(converted, Math.PI)",
        "value = [1, 2]\nassert(Number.isNaN(converted), true)",
        "value = \"invalid\"\nassert(Number.isNaN(converted), true)",
        "value = -180\nassert(converted, -Math.PI)",
        "value = 90\nassert(converted, Math.PI / 2)",
    ] {
        for runtime in &mut runtimes {
            checked(runtime, source, None);
        }
        for name in ["converted", "reciprocal"] {
            assert_numeric_same(
                runtimes[0].state.variable(name).unwrap(),
                runtimes[1].state.variable(name).unwrap(),
            );
        }
    }
    for runtime in &mut runtimes {
        checked(runtime, "converted", Some("1.5707963267948966"));
    }
}

#[test]
fn all_argument_effects_run_once_left_to_right_before_gating_and_errors() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 0
converted = Math.toRadians(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3,
    log.order = log.order.value * 10 + 4,
    log.order = log.order.value * 10 + 5
)
assert(log.order, 12345)
assert(converted, Math.PI / 180)
invalid = Math.toRadians(Number.NaN, log.order = 8)
assert(Number.isNaN(invalid), true)
assert(log.order, 8)
deferred = Math.toRadians(log.input, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.input = 180
assert(deferred, Math.PI)
assert(log.order, 7)
log.order = 0
mapped = [180, 90, -180].map(value => Math.toRadians(value, log.order = log.order.value * 10 + 1))
assert(mapped, [Math.PI, Math.PI / 2, -Math.PI])
assert(log.order, 111)
try:
    Math.toRadians(log.order = 9, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 111)
try:
    Math.toRadians(log.absent, Math.random(), missing)
catch error:
    assert(error, TypeError("Math.random is not supported"))
try:
    Math.toRadians(log.absent, missing, Math.random())
catch error:
    assert(error, ReferenceError("missing is not defined"))
"#);
    for (source, kind, message) in [
        (
            "Math.toRadians(missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.toRadians(180, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.toRadians(log.absent, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.toRadians(180, 99, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.toRadians(log.absent, Math.random())",
            ErrorKind::Type,
            "Math.random is not supported",
        ),
        (
            "Math.unknown()",
            ErrorKind::Type,
            "Math.unknown is not a function",
        ),
        (
            "Math.toDegrees(180)",
            ErrorKind::Type,
            "Math.toDegrees is not a function",
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
        .run("log.order = 9\nMath.toRadians(180, missing)")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    checked(
        &mut runtime,
        "assert(log.order, 111)\nassert(deferred, Math.PI)",
        None,
    );
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
}

#[test]
fn undefined_first_and_every_extra_slot_keep_actual_reads_deletion_and_recovery() {
    for (arguments, defined) in [
        ("probe.input", "180"),
        ("180, probe.input", "{}"),
        ("180, 99, probe.input", "Number.NaN"),
        ("180, 99, \"invalid\", probe.input", "[1, 2]"),
    ] {
        let runtime = run(&format!(
            "class Probe\nprobe = Probe()\nconverted = Math.toRadians({arguments})\n\
             assert(converted, null)\nprobe.input = {defined}\nassert(converted, Math.PI)\n\
             delete probe.input\nassert(converted, null)\nprobe.input = {defined}\nassert(converted, Math.PI)"
        ));
        assert!(runtime.graph.reaches(
            &NodeKey::property(&ObjectId::named("probe"), "input"),
            &NodeKey::variable("converted"),
        ));
        assert_eq!(
            dependencies(&runtime, "converted"),
            ["probe", "probe.input"]
        );
    }
    let runtime = run(r#"
class Probe
probe = Probe()
input = 180
extra = 99
converted = Math.toRadians(input, extra)
assert(converted, Math.PI)
extra = Number.NaN
assert(converted, Math.PI)
assert(Math.unknown(probe.absent), null)
assert(Math.random(probe.absent), null)
trace = why converted
assert(trace[0].from, ["input", "extra"])
assert(trace[0].holds, Math.PI)
"#);
    assert_eq!(dependencies(&runtime, "converted"), ["input", "extra"]);
}

#[test]
fn literal_coercion_omission_and_unconverted_extras_preserve_live_null_deferral() {
    for expression in [
        "null",
        "\" 180 \"",
        "\"\"",
        "true",
        "false",
        "[]",
        "[\"90\"]",
        "Date(180)",
        "[1, 2]",
        "{}",
        "/probe/",
        "value => value",
    ] {
        let mut runtime = Runtime::new();
        let actual = checked(&mut runtime, &format!("Math.toRadians({expression})"), None);
        let expected = checked(
            &mut runtime,
            &format!("({expression}) * (Math.PI / 180)"),
            None,
        );
        assert_numeric_same(&actual, &expected);
    }
    let mut runtime = run(r#"
assert(Math.toRadians(null), 0)
assert(Number.isNaN(Math.toRadians()), true)
assert(Math.toRadians(180, Number.NaN, "invalid", {}, null), Math.PI)
assert(Math.toRadians(90, [1, 2], Date(99), /probe/, value => value), Math.PI / 2)
class Probe
assert(Number.isNaN(Math.toRadians(Probe)), true)
assert(Number.isNaN(Math.toRadians(Probe())), true)
value = 180
extra = 99
converted = Math.toRadians(value, extra)
def radians input: Numeric => Math.toRadians(input)
omitted = radians()
parameter = radians(null)
assert(omitted, null)
assert(parameter, null)
assert(radians(180, 99), Math.PI)
"#);
    for (name, restored) in [("value", "180"), ("extra", "99")] {
        checked(
            &mut runtime,
            &format!(
                "{name} = null\nassert(converted, null)\n{name} = {restored}\nassert(converted, Math.PI)"
            ),
            None,
        );
    }
}

#[test]
fn intrinsic_factor_conversion_shadowing_and_method_reads_keep_lookup_rules() {
    let runtime = run(r#"
def Number(value) => 999
assert(Number("180"), 999)
text = "180"
converted = Math.toRadians(text)
assert(converted, Math.PI)
text = "90"
assert(converted, Math.PI / 2)
def numeric(Number):
    return Math.toRadians("180")
assert(numeric(null), Math.PI)
"#);
    assert_eq!(dependencies(&runtime, "converted"), ["text"]);
    let runtime = run(r#"
Number = 7
converted = Math.toRadians("180")
reference = Math.toRadians
indirect = reference(180)
assert(reference, null)
assert(indirect, null)
class Namespace:
    def toRadians value => value + 8
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
namespace.PI = 99
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.toRadians(2), 10)
def scoped(Math):
    return Math.toRadians(3)
assert(scoped(namespace), 11)
assert(scoped(null), null)
Math = namespace
assert(Math.toRadians(4), 12)
"#);
    assert_numeric_same(
        runtime.state.variable("converted").unwrap(),
        &Value::Number(std::f64::consts::PI),
    );
    assert!(dependencies(&runtime, "converted").is_empty());
    for name in ["Number", "Math", "Math.PI"] {
        assert!(!dependencies(&runtime, "converted").contains(&name));
    }
    let mut runtime = run("Math = 7");
    let error = runtime.run("Math.toRadians(180)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.toRadians is not a function");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn typed_wrappers_receivers_and_ordinary_call_apis_preserve_frames() {
    let runtime = run(r#"
input = 180
extra = 99
def radians amount: ArbitraryType => Math.toRadians(amount, extra)
converted = radians(input)
assert(converted, Math.PI)
input = 90
assert(converted, Math.PI / 2)
try:
    def radians amount: ArbitraryType => Math.toRadians(missing, extra)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(converted, Math.PI / 2)
input = -180
assert(converted, -Math.PI)
class Reading:
    def init degrees => this.degrees = degrees
    def radians() => Math.toRadians(self.degrees)
reading = Reading(180)
assert(reading.radians(), Math.PI)
reading.degrees = 90
assert(reading.radians(), Math.PI / 2)
"#);
    for name in ["radians", "input", "extra"] {
        assert!(
            dependencies(&runtime, "converted").contains(&name),
            "{name}"
        );
    }
    for name in ["ArbitraryType", "missing"] {
        assert!(!runtime.graph.contains(&NodeKey::variable(name)));
    }
    let statements = parse("Math.toRadians(value)").unwrap();
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
    scope.declare("value", Value::Number(180.0));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    assert_numeric_same(
        &call.resolve(&mut runtime, &mut scope).unwrap(),
        &Value::Number(std::f64::consts::PI),
    );
    let failed = parse_expression("Math.toRadians(value, missing)").unwrap();
    let error = Ast::convert(&failed)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.retrieve("value"), Some(&Value::Number(180.0)));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
}

#[test]
fn failed_rewrites_restore_complete_state_graph_and_class_subscriptions() {
    let initial = r#"
class Reading
$Reading.radians = Math.toRadians($Reading.degrees)
reading = Reading()
reading.degrees = 180
value = reading.degrees
alternative = 45
converted = Math.toRadians(value)
assert(converted, Math.PI)
assert(reading.radians, Math.PI)
"#;
    let mut actual = run(initial);
    let mut control = run(initial);
    let error = actual
        .run("converted = Math.toRadians(alternative)\nreading.degrees = 90\nthrow \"ROLLBACK\"")
        .unwrap_err();
    assert_eq!(
        error.thrown_value(),
        Some(&Value::String("ROLLBACK".to_string()))
    );
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    let error = actual
        .run("converted = Math.toRadians(alternative)\nconverted = Math.toRadians(missing)")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    // Failed transactions restore existing graph nodes but can consume future
    // sequence numbers. Mirror that history before comparing recovery snapshots;
    // the two immediate comparisons above independently protect rollback itself.
    for source in [
        "converted = Math.toRadians(alternative)\nreading.degrees = 90\nthrow \"ROLLBACK\"",
        "converted = Math.toRadians(alternative)\nconverted = Math.toRadians(missing)",
    ] {
        control.run(source).unwrap_err();
    }
    for runtime in [&mut actual, &mut control] {
        checked(
            runtime,
            r#"
alternative = -180
assert(converted, Math.PI)
reading.degrees = 90
assert(converted, Math.PI / 2)
assert(reading.radians, Math.PI / 2)
delete reading.degrees
assert(value, null)
assert(converted, null)
assert(reading.radians, null)
reading.degrees = -180
assert(converted, -Math.PI)
assert(reading.radians, -Math.PI)
converted
"#,
            Some("-3.141592653589793"),
        );
        assert_eq!(dependencies(runtime, "converted"), ["value"]);
        assert!(runtime.graph.reaches(
            &NodeKey::property(&ObjectId::named("reading"), "degrees"),
            &NodeKey::variable("converted"),
        ));
        assert!(!runtime.graph.reaches(
            &NodeKey::variable("alternative"),
            &NodeKey::variable("converted"),
        ));
    }
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
}
