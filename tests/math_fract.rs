//! Floor-based fractional parts beside independent accepted arithmetic.
//! Ordinary regressions use embedded controls, not Git or session artifacts.

mod common;

use std::collections::HashMap;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

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
            title: "Math.fract regression".to_string(),
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

fn assert_numeric_same(actual: &Value, expected: &Value) {
    let (Value::Number(actual), Value::Number(expected)) = (actual, expected) else {
        panic!("both controls must be numeric: {actual:?}, {expected:?}");
    };
    if expected.is_nan() {
        assert!(actual.is_nan(), "{actual} is not NaN");
    } else {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
}

fn assert_numeric_parity(left: &Runtime, right: &Runtime, name: &str) {
    assert_numeric_same(
        left.state.variable(name).unwrap(),
        right.state.variable(name).unwrap(),
    );
}

struct SealedCase {
    document: &'static str,
    title: &'static str,
    accepted: &'static str,
    assertions: usize,
    expected: Option<&'static str>,
    rewrites: usize,
}

// These accepted forms independently pin the entire new programs, including
// comments, ordered assertions and both catches, without fixing future tails.
const SEALED: &[SealedCase] = &[
    SealedCase {
        document: "spec",
        title: "Nucleoid keeps floor fractional parts reactive",
        accepted: r#"# Nucleoid keeps floor fractional parts reactive

value = 2.75
whole = Math.floor(value)
fraction = value - Math.floor(value)
assert(whole, 2)
assert(fraction, 0.75)
value = -1.25
assert(whole, -2)
assert(fraction, 0.75)
value = 4.125
assert(whole, 4)
assert(fraction, 0.125)
value = -2
assert(whole, -2)
assert(fraction, 0)
value = 0
assert(whole, 0)
assert(fraction, 0)
value = -0
assert(whole, 0)
assert(fraction, 0)
value = 1.5
assert(whole, 1)
assert(fraction, 0.5)
fraction"#,
        assertions: 14,
        expected: Some("0.5"),
        rewrites: 1,
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves numeric coercion while computing floor fractional parts",
        accepted: r#"# Nucleoid preserves numeric coercion while computing floor fractional parts

value = " 2.75 "
whole = Math.floor(value)
fraction = value - Math.floor(value)
assert(whole, 2)
assert(fraction, 0.75)
value = true
assert(whole, 1)
assert(fraction, 0)
value = false
assert(whole, 0)
assert(fraction, 0)
value = []
assert(whole, 0)
assert(fraction, 0)
value = ["3.125"]
assert(whole, 3)
assert(fraction, 0.125)
value = Date(1234)
assert(whole, 1234)
assert(fraction, 0)
value = [1, 2]
assert(Number.isNaN(whole), true)
assert(Number.isNaN(fraction), true)
value = "not-a-number"
assert(Number.isNaN(whole), true)
assert(Number.isNaN(fraction), true)"#,
        assertions: 16,
        expected: None,
        rewrites: 1,
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves nonfinite fractional parts omissions and rounded endpoints",
        accepted: r#"# Nucleoid preserves nonfinite fractional parts omissions and rounded endpoints

value = -0.00000000000000000001
whole = Math.floor(value)
fraction = value - Math.floor(value)
assert(whole, -1)
assert(fraction, 1)
value = Number.POSITIVE_INFINITY
assert(whole, Number.POSITIVE_INFINITY)
assert(Number.isNaN(fraction), true)
value = Number.NEGATIVE_INFINITY
assert(whole, Number.NEGATIVE_INFINITY)
assert(Number.isNaN(fraction), true)
value = Number.NaN
assert(Number.isNaN(whole), true)
assert(Number.isNaN(fraction), true)
value = Number.MAX_VALUE
assert(whole, Number.MAX_VALUE)
assert(fraction, 0)
value = -Number.MAX_VALUE
assert(whole, -Number.MAX_VALUE)
assert(fraction, 0)
empty = Math.floor()
assert(Number.isNaN(empty), true)
half = 0.5
withExtra = Math.saturate(half - Math.floor(half), 99)
ignored = Number.NaN
withIgnored = Math.saturate(half - Math.floor(half), ignored)
assert(half, 0.5)
assert(withExtra, 0.5)
assert(withIgnored, 0.5)
ignored = "not-a-number"
assert(withIgnored, 0.5)"#,
        assertions: 17,
        expected: None,
        rewrites: 4,
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid applies floor fractional parts to live class rules",
        accepted: r#"# Nucleoid applies floor fractional parts to live class rules

class Reading
$Reading.whole = Math.floor($Reading.sample)
$Reading.fraction = $Reading.sample - Math.floor($Reading.sample)
first = Reading()
second = Reading()
first.sample = 2.25
second.sample = -1.75
assert(first.whole, 2)
assert(first.fraction, 0.25)
assert(second.whole, -2)
assert(second.fraction, 0.25)
first.sample = -1.25
assert(first.whole, -2)
assert(first.fraction, 0.75)
assert(second.fraction, 0.25)
third = Reading()
assert(third.whole, null)
assert(third.fraction, null)
third.sample = 8.125
assert(third.whole, 8)
assert(third.fraction, 0.125)
delete first.sample
assert(first.whole, null)
assert(first.fraction, null)
first.sample = -3.5
assert(first.whole, -4)
assert(first.fraction, 0.5)
assert(Reading.length, 3)"#,
        assertions: 16,
        expected: None,
        rewrites: 1,
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves deferred fractional parts and rollback",
        accepted: r#"# Nucleoid preserves deferred fractional parts and rollback

class Sensor
sensor = Sensor()
value = sensor.reading
whole = Math.floor(value)
fraction = value - Math.floor(value)
withDeferredExtra = Math.saturate(value - Math.floor(value), sensor.optional)
assert(value, null)
assert(whole, null)
assert(fraction, null)
assert(withDeferredExtra, null)
sensor.reading = 2.25
assert(whole, 2)
assert(fraction, 0.25)
assert(withDeferredExtra, null)
delete sensor.reading
assert(value, null)
assert(fraction, null)
sensor.reading = 3.75
assert(whole, 3)
assert(fraction, 0.75)
sensor.optional = 99
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = 5.5
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 3.75)
assert(whole, 3)
assert(fraction, 0.75)
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(sensor.reading, 3.75)
delete sensor.optional
assert(withDeferredExtra, null)
assert(fraction, 0.75)
sensor.optional = [1, 2]
assert(withDeferredExtra, 0.75)
sensor.reading = 2.25
assert(whole, 2)
assert(fraction, 0.25)
assert(withDeferredExtra, 0.25)
fraction"#,
        assertions: 25,
        expected: Some("0.25"),
        rewrites: 2,
    },
    SealedCase {
        document: "reference",
        title: "Nucleoid preserves floor fractional part relationships",
        accepted: r#"# Nucleoid preserves floor fractional part relationships

class Signal
signal = Signal()
signal.sample = 1.75
whole = Math.floor(signal.sample)
fraction = signal.sample - Math.floor(signal.sample)
assert(whole, 1)
assert(fraction, 0.75)
signal.sample = -1.5
assert(whole, -2)
assert(fraction, 0.5)
signal.sample = 4.125
assert(fraction, 0.125)"#,
        assertions: 5,
        expected: None,
        rewrites: 1,
    },
    SealedCase {
        document: "readme",
        title: "Floor fractional parts keep negative inputs live",
        accepted: r#"value = -1.25
whole = Math.floor(value)
fraction = value - Math.floor(value)
assert(whole, -2)
assert(fraction, 0.75)
value = 3.5
assert(fraction, 0.5)"#,
        assertions: 3,
        expected: None,
        rewrites: 1,
    },
    SealedCase {
        document: "examples",
        title: "Floor fractional parts preserve negative and rounded endpoints",
        accepted: r#"value = -2.75
whole = Math.floor(value)
fraction = value - Math.floor(value)
assert(whole, -3)
assert(fraction, 0.25)
value = 0.5
assert(whole, 0)
assert(fraction, 0.5)
value = 2
assert(whole, 2)
assert(fraction, 0)
value = -0.00000000000000000001
assert(whole, -1)
assert(fraction, 1)"#,
        assertions: 8,
        expected: None,
        rewrites: 1,
    },
];

#[test]
fn eight_sealed_cases_keep_exact_sources_assertions_returns_and_twelve_rewrites() {
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
    // Whole RHS replacements precede the ordinary fraction substitutions.
    let rewrites = [
        (
            "withExtra = Math.saturate(half - Math.floor(half), 99)",
            "withExtra = Math.fract(half, 99)",
        ),
        (
            "withIgnored = Math.saturate(half - Math.floor(half), ignored)",
            "withIgnored = Math.fract(half, ignored)",
        ),
        (
            "withDeferredExtra = Math.saturate(value - Math.floor(value), sensor.optional)",
            "withDeferredExtra = Math.fract(value, sensor.optional)",
        ),
        ("empty = Math.floor()", "empty = Math.fract()"),
        (
            "fraction = value - Math.floor(value)",
            "fraction = Math.fract(value)",
        ),
        (
            "$Reading.fraction = $Reading.sample - Math.floor($Reading.sample)",
            "$Reading.fraction = Math.fract($Reading.sample)",
        ),
        (
            "fraction = signal.sample - Math.floor(signal.sample)",
            "fraction = Math.fract(signal.sample)",
        ),
    ];
    let mut previous = HashMap::new();
    let mut totals = (0, 0, 0);
    for sealed in SEALED {
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == sealed.document)
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
        let mut candidate = sealed.accepted.to_string();
        let mut changes = 0;
        for (accepted, intrinsic) in rewrites {
            changes += candidate.matches(accepted).count();
            candidate = candidate.replace(accepted, intrinsic);
        }
        assert_eq!(changes, sealed.rewrites, "{}", sealed.title);
        assert_eq!(candidate, case.source.trim(), "{}", sealed.title);
        assert_eq!(changes, candidate.matches("Math.fract(").count());
        let actual_ast = parse(&case.source).unwrap();
        let accepted_ast = parse(sealed.accepted).unwrap();
        assert_ne!(actual_ast, accepted_ast, "distinct RHS controls required");
        let protected = common::assertions::protected_assertions(&accepted_ast);
        assert_eq!(protected.len(), sealed.assertions, "{}", sealed.title);
        assert_eq!(
            common::assertions::protected_assertions(&actual_ast),
            protected,
            "{}",
            sealed.title
        );
        let control = common::Case {
            title: sealed.title.to_string(),
            source: sealed.accepted.to_string(),
            expected: sealed.expected.map(str::to_string),
        };
        let mut runtimes = [Runtime::new(), Runtime::new()];
        for (runtime, source) in runtimes.iter_mut().zip([case, &control]) {
            common::check_with_count(runtime, source, sealed.assertions)
                .unwrap_or_else(|reason| panic!("{}: {reason}", sealed.title));
            assert_eq!(runtime.assertions_run(), sealed.assertions);
        }
        if sealed.title.contains("class rules") {
            for name in ["first", "second", "third"] {
                for property in ["whole", "fraction"] {
                    let id = ObjectId::named(name);
                    assert_numeric_same(
                        runtimes[0].state.property(&id, property).unwrap(),
                        runtimes[1].state.property(&id, property).unwrap(),
                    );
                    for runtime in &runtimes {
                        assert!(runtime.graph.reaches(
                            &NodeKey::property(&id, "sample"),
                            &NodeKey::property(&id, property),
                        ));
                    }
                }
            }
        } else {
            for name in ["whole", "fraction"] {
                assert_numeric_parity(&runtimes[0], &runtimes[1], name);
            }
            let input = if sealed.document == "reference" {
                NodeKey::property(&ObjectId::named("signal"), "sample")
            } else {
                NodeKey::variable("value")
            };
            for runtime in &runtimes {
                assert!(
                    runtime
                        .graph
                        .reaches(&input, &NodeKey::variable("fraction"))
                );
            }
        }
        totals.0 += sealed.assertions;
        totals.1 += usize::from(sealed.expected.is_some());
        totals.2 += changes;
    }
    assert_eq!(totals, (104, 2, 12));
}

#[test]
fn reactive_coercion_rounding_nonfinite_and_zero_bits_match_floor_subtraction() {
    let template =
        "value = -0\nwhole = Math.floor(value)\nfraction = __FRACTION__\nreciprocal = 1 / fraction";
    let mut runtimes = ["Math.fract(value)", "value - Math.floor(value)"]
        .map(|expression| run(&template.replace("__FRACTION__", expression)));
    for source in [
        "assert(fraction, 0)",
        "value = 0",
        "value = -0",
        "value = 5e-324",
        "value = -5e-324\nassert(fraction, 1)",
        "value = Number.MIN_VALUE / 2",
        "value = -Number.MIN_VALUE / 2\nassert(fraction, 1)",
        "value = -0.00000000000000000001\nassert(fraction, 1)",
        "value = -1.25\nassert(fraction, 0.75)",
        "value = 1 - Number.EPSILON / 2",
        "value = 1\nassert(fraction, 0)",
        "value = 1 + Number.EPSILON",
        "value = -4503599627370495.5\nassert(fraction, 0.5)",
        "value = Number.MAX_VALUE\nassert(fraction, 0)",
        "value = -Number.MAX_VALUE\nassert(fraction, 0)",
        "value = Number.POSITIVE_INFINITY\nassert(Number.isNaN(fraction), true)",
        "value = Number.NEGATIVE_INFINITY\nassert(Number.isNaN(fraction), true)",
        "value = Number.NaN\nassert(Number.isNaN(fraction), true)",
        "value = Number.NaN\nassert(Number.isNaN(fraction), true)",
        "value = \" -2.75 \"\nassert(fraction, 0.25)",
        "value = true\nassert(fraction, 0)",
        "value = false\nassert(fraction, 0)",
        "value = []\nassert(fraction, 0)",
        "value = [\"3.125\"]\nassert(fraction, 0.125)",
        "value = Date(-1234)\nassert(fraction, 0)",
        "value = [1, 2]\nassert(Number.isNaN(fraction), true)",
        "value = \"invalid\"\nassert(Number.isNaN(fraction), true)",
        "value = 2.25\nassert(fraction, 0.25)",
    ] {
        for runtime in &mut runtimes {
            checked(runtime, source, None);
        }
        for name in ["whole", "fraction", "reciprocal"] {
            assert_numeric_parity(&runtimes[0], &runtimes[1], name);
        }
    }
}

#[test]
fn argument_effects_run_once_in_order_before_gating_or_strict_errors() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 0
fraction = Math.fract(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3,
    log.order = log.order.value * 10 + 4,
    log.order = log.order.value * 10 + 5
)
assert(log.order, 12345)
assert(fraction, 0)
invalid = Math.fract(Number.NaN, log.order = 8)
assert(Number.isNaN(invalid), true)
assert(log.order, 8)
deferred = Math.fract(log.input, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.input = -1.25
assert(deferred, 0.75)
assert(log.order, 7)
log.order = 0
mapped = [1.25, 2.5, 3.75].map(value => Math.fract(value, log.order = log.order.value * 10 + 1))
assert(mapped, [0.25, 0.5, 0.75])
assert(log.order, 111)
try:
    Math.fract(log.order = 9, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 111)
try:
    Math.fract(log.absent, Math.random())
catch error:
    assert(error, TypeError("Math.random is not supported"))
"#);
    for (source, kind, message) in [
        (
            "Math.fract(missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.fract(0.5, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.fract(log.absent, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.fract(0.5, 99, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.fract(log.absent, Math.random())",
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
        .run("log.order = 9\nMath.fract(0.5, missing)")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    checked(
        &mut runtime,
        "assert(log.order, 111)\nassert(deferred, 0.75)",
        None,
    );
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
}

#[test]
fn undefined_first_or_extra_values_keep_reads_deletion_and_recovery() {
    for (arguments, defined) in [
        ("probe.input", "-1.25"),
        ("-1.25, probe.input", "{}"),
        ("-1.25, 99, probe.input", "Number.NaN"),
        ("-1.25, 99, \"invalid\", probe.input", "[1, 2]"),
    ] {
        let runtime = run(&format!(
            "class Probe\nprobe = Probe()\nfraction = Math.fract({arguments})\n\
             assert(fraction, null)\nprobe.input = {defined}\nassert(fraction, 0.75)\n\
             delete probe.input\nassert(fraction, null)\nprobe.input = {defined}\nassert(fraction, 0.75)"
        ));
        assert!(
            runtime
                .graph
                .retrieve(&NodeKey::variable("fraction"))
                .unwrap()
                .dependencies
                .contains(&NodeKey::property(&ObjectId::named("probe"), "input"))
        );
    }
    let runtime = run(r#"
class Probe
probe = Probe()
input = -1.25
extra = 99
fraction = Math.fract(input, extra)
assert(fraction, 0.75)
extra = Number.NaN
assert(fraction, 0.75)
assert(Math.unknown(probe.absent), null)
assert(Math.random(probe.absent), null)
trace = why fraction
assert(trace[0].from, ["input", "extra"])
assert(trace[0].holds, 0.75)
"#);
    assert_eq!(dependencies(&runtime, "fraction"), ["input", "extra"]);
}

#[test]
fn intrinsic_literal_coercion_does_not_override_live_or_parameter_null_deferral() {
    for expression in [
        "null",
        "\" -2.75 \"",
        "\"\"",
        "true",
        "false",
        "[]",
        "[\"3.125\"]",
        "Date(1234)",
        "[1, 2]",
        "{}",
        "/probe/",
        "value => value",
    ] {
        let mut runtime = Runtime::new();
        let actual = checked(&mut runtime, &format!("Math.fract({expression})"), None);
        let expected = checked(
            &mut runtime,
            &format!("({expression}) - Math.floor({expression})"),
            None,
        );
        assert_numeric_same(&actual, &expected);
    }
    let mut runtime = run(r#"
assert(Math.fract(null), 0)
assert(Number.isNaN(Math.fract()), true)
assert(Math.fract(-1.25, Number.NaN, "invalid", {}, null), 0.75)
assert(Math.fract(2.25, [1, 2], Date(99), /probe/, value => value), 0.25)
class Probe
assert(Number.isNaN(Math.fract(Probe)), true)
assert(Number.isNaN(Math.fract(Probe())), true)
value = -1.25
extra = 99
fraction = Math.fract(value, extra)
def fractional input: Numeric => Math.fract(input)
omitted = fractional()
parameter = fractional(null)
assert(omitted, null)
assert(parameter, null)
assert(fractional(-1.25, 99), 0.75)
"#);
    for (name, restored) in [("value", "-1.25"), ("extra", "99")] {
        checked(
            &mut runtime,
            &format!(
                "{name} = null\nassert(fraction, null)\n{name} = {restored}\nassert(fraction, 0.75)"
            ),
            None,
        );
    }
}

#[test]
fn intrinsic_names_shadowing_members_and_static_method_reads_keep_lookup_rules() {
    let runtime = run(r#"
def Number(value) => 999
def floor(value) => 999
assert(Number("-1.25"), 999)
assert(floor(-1.25), 999)
text = "-1.25"
fraction = Math.fract(text)
assert(fraction, 0.75)
text = "3.125"
assert(fraction, 0.125)
def numeric(Number):
    return Math.fract("-1.25")
assert(numeric(null), 0.75)
assert(Math.floor(-1.25), -2)
"#);
    assert_eq!(dependencies(&runtime, "fraction"), ["text"]);
    let runtime = run(r#"
Number = 7
fraction = Math.fract("-1.25")
assert(fraction, 0.75)
reference = Math.fract
indirect = reference(-1.25)
assert(reference, null)
assert(indirect, null)
class Namespace:
    def fract value => value + 8
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.fract(2), 10)
def scoped(Math):
    return Math.fract(3)
assert(scoped(namespace), 11)
assert(scoped(null), null)
Math = namespace
assert(Math.fract(4), 12)
"#);
    assert!(dependencies(&runtime, "fraction").is_empty());
    for name in ["Number", "floor", "Math.floor"] {
        assert!(!dependencies(&runtime, "fraction").contains(&name));
    }
    let mut runtime = run("Math = 7");
    let error = runtime.run("Math.fract(-1.25)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.fract is not a function");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn typed_wrappers_receivers_and_ordinary_call_apis_preserve_frames() {
    let runtime = run(r#"
input = -1.25
extra = 99
def fractional amount: ArbitraryType => Math.fract(amount, extra)
fraction = fractional(input)
assert(fraction, 0.75)
input = 3.125
assert(fraction, 0.125)
try:
    def fractional amount: ArbitraryType => Math.fract(missing, extra)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(fraction, 0.125)
input = -2.5
assert(fraction, 0.5)
class Reading:
    def init sample => this.sample = sample
    def fraction() => Math.fract(self.sample)
reading = Reading(-1.25)
assert(reading.fraction(), 0.75)
reading.sample = 2.125
assert(reading.fraction(), 0.125)
"#);
    for name in ["fractional", "input", "extra"] {
        assert!(dependencies(&runtime, "fraction").contains(&name), "{name}");
    }
    for name in ["ArbitraryType", "missing"] {
        assert!(!runtime.graph.contains(&NodeKey::variable(name)));
    }

    let statements = parse("Math.fract(value)").unwrap();
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
    scope.declare("value", Value::Number(-1.25));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    assert_numeric_same(
        &call.resolve(&mut runtime, &mut scope).unwrap(),
        &Value::Number(0.75),
    );
    let failed = parse_expression("Math.fract(value, missing)").unwrap();
    let error = Ast::convert(&failed)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.retrieve("value"), Some(&Value::Number(-1.25)));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
}

#[test]
fn rejected_relationship_rewrites_restore_graph_subscriptions_and_class_rules() {
    let runtime = run(r#"
class Reading
$Reading.fraction = Math.fract($Reading.sample)
reading = Reading()
reading.sample = -1.25
value = reading.sample
alternative = 4.125
fraction = Math.fract(value)
trace = why fraction
assert(fraction, 0.75)
assert(reading.fraction, 0.75)
try:
    fraction = Math.fract(alternative)
    reading.sample = 2.5
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(fraction, 0.75)
assert(reading.fraction, 0.75)
assert(trace[0].from, ["value"])
alternative = -2.25
assert(fraction, 0.75)
reading.sample = 3.125
assert(fraction, 0.125)
assert(reading.fraction, 0.125)
delete reading.sample
assert(value, null)
assert(fraction, null)
assert(reading.fraction, null)
reading.sample = -3.5
assert(fraction, 0.5)
assert(reading.fraction, 0.5)
"#);
    assert_eq!(dependencies(&runtime, "fraction"), ["value"]);
    assert!(runtime.graph.reaches(
        &NodeKey::property(&ObjectId::named("reading"), "sample"),
        &NodeKey::variable("fraction"),
    ));
    assert!(!runtime.graph.reaches(
        &NodeKey::variable("alternative"),
        &NodeKey::variable("fraction"),
    ));
}

#[test]
fn native_cli_preserves_exact_file_bytes_json_errors_and_literal_eof() {
    for (index, (source, success, stdout, stderr)) in [
        (
            "assert(Math.fract(-1.25), 0.75)\nMath.fract(-1.25)",
            true,
            "0.75\n",
            "",
        ),
        ("Math.fract(2.125)", true, "0.125\n", ""),
        ("Math.fract(-0.00000000000000000001)", true, "1.0\n", ""),
        ("Math.fract(Number.POSITIVE_INFINITY)", true, "null\n", ""),
        ("String(Math.fract())", true, "\"NaN\"\n", ""),
        (
            "Math.fract(-1.25, Number.NaN, \"invalid\", {})",
            true,
            "0.75\n",
            "",
        ),
        (
            "Math.fract(0.5, missing)",
            false,
            "",
            "ReferenceError: missing is not defined (line 1, column 1)\n",
        ),
        (
            "class Probe\nprobe = Probe()\nMath.fract(probe.absent, Math.random())",
            false,
            "",
            "TypeError: Math.random is not supported (line 3, column 1)\n",
        ),
        (
            "Math.fract(",
            false,
            "",
            "SyntaxError: Unexpected end of input (line 1, column 12)\n",
        ),
        (
            "Math.fract(\n",
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
                "nuc36-math-fract-{}-{index}.nuc",
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

#[test]
fn stdin_literal_eof_and_repl_error_recovery_keep_live_fractional_parts() {
    let source = "value = 0.25\nfraction = Math.fract(value)\nMath.fract(0.5, missing)\nvalue = -1.25\nfraction";
    assert!(!source.ends_with('\n'));
    for graph in [false, true] {
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
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            output.stderr,
            b"ReferenceError: missing is not defined (line 1, column 1)\n"
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        if graph {
            assert!(stdout.contains("fraction [variable]"), "{stdout}");
            assert!(stdout.contains("value [variable]"), "{stdout}");
            assert!(stdout.contains("0.75"), "{stdout}");
            assert!(!stdout.contains("missing [variable]"), "{stdout}");
        } else {
            let banner = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";
            assert_eq!(stdout, format!("{banner}0.25\n0.25\n-1.25\n0.75\n"));
        }
    }
}
