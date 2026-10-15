//! Unclamped ordered normalization beside independent accepted arithmetic.
//! Ordinary regressions require neither Git nor external session artifacts.

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
            title: "Math.inverseLerp regression".to_string(),
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

// Exact accepted v2 counterparts, independently pinning every non-RHS byte,
// canonical assertion, return and both real catches without fixing future tails.
const SEALED: &[SealedCase] = &[
    SealedCase {
        document: "spec",
        title: "Nucleoid keeps inverse interpolation reactive",
        accepted: r#"# Nucleoid keeps inverse interpolation reactive

start = 2
end = 10
sample = 4
weight = (sample - start) / (end - start)
reconstructed = Math.lerp(start, end, weight)
assert(weight, 0.25)
assert(reconstructed, 4)
sample = 6
assert(weight, 0.5)
assert(reconstructed, 6)
sample = 14
assert(weight, 1.5)
assert(reconstructed, 14)
sample = -2
assert(weight, -0.5)
assert(reconstructed, -2)
start = 10
end = 2
sample = 8
assert(weight, 0.25)
assert(reconstructed, 8)
sample = 12
assert(weight, -0.25)
assert(reconstructed, 12)
sample = 6
assert(weight, 0.5)
assert(reconstructed, 6)
weight"#,
        assertions: 14,
        expected: Some("0.5"),
        rewrites: 1,
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves intrinsic conversion during inverse interpolation",
        accepted: r#"# Nucleoid preserves intrinsic conversion during inverse interpolation

start = " 2 "
end = "10"
sample = "4"
weight = (sample - start) / (end - start)
assert(weight, 0.25)
assert(start, " 2 ")
start = false
end = true
sample = false
assert(weight, 0)
sample = true
assert(weight, 1)
sample = [0.5]
assert(weight, 0.5)
sample = []
assert(weight, 0)
sample = "not-a-number"
assert(Number.isNaN(weight), true)
sample = [1, 2]
assert(Number.isNaN(weight), true)
start = Date(0)
end = Date(8)
sample = Date(2)
assert(weight, 0.25)
sample = Date(6)
assert(weight, 0.75)
start = []
end = ["8"]
sample = ["2"]
assert(weight, 0.25)
sample = false
assert(weight, 0)"#,
        assertions: 12,
        expected: None,
        rewrites: 1,
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves inverse interpolation endpoints nonfinite inputs and omissions",
        accepted: r#"# Nucleoid preserves inverse interpolation endpoints nonfinite inputs and omissions

start = 0
end = 8
sample = 2
weight = (sample - start) / (end - start)
assert(weight, 0.25)
sample = start
assert(weight, 0)
sample = end
assert(weight, 1)
sample = 16
assert(weight, 2)
sample = -8
assert(weight, -1)
start = 2
end = 2
sample = 2
assert(Number.isNaN(weight), true)
sample = 3
assert(weight, Number.POSITIVE_INFINITY)
sample = 1
assert(weight, Number.NEGATIVE_INFINITY)
start = 0
end = Number.POSITIVE_INFINITY
sample = 8
assert(weight, 0)
sample = Number.POSITIVE_INFINITY
assert(Number.isNaN(weight), true)
end = 8
sample = Number.NEGATIVE_INFINITY
assert(weight, Number.NEGATIVE_INFINITY)
start = Number.NEGATIVE_INFINITY
end = Number.POSITIVE_INFINITY
sample = 0
assert(Number.isNaN(weight), true)
start = -Number.MAX_VALUE
end = Number.MAX_VALUE
sample = 0
assert(weight, 0)
start = 0
end = 8
sample = Number.NaN
assert(Number.isNaN(weight), true)
empty = Math.floor()
assert(Number.isNaN(empty), true)
one = Math.lerp(0)
assert(Number.isNaN(one), true)
two = Math.lerp(0, 8)
assert(Number.isNaN(two), true)
literal = (null - false) / (true - false)
assert(literal, 0)
withExtra = Math.saturate((2 - 0) / (8 - 0), 99)
assert(withExtra, 0.25)
ignored = Number.NaN
withIgnored = Math.saturate((2 - 0) / (8 - 0), ignored)
assert(withIgnored, 0.25)
ignored = "not-a-number"
assert(withIgnored, 0.25)
ignored = null
assert(withIgnored, null)
ignored = []
assert(withIgnored, 0.25)"#,
        assertions: 23,
        expected: None,
        rewrites: 7,
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid applies inverse interpolation to live class rules",
        accepted: r#"# Nucleoid applies inverse interpolation to live class rules

class Gauge
$Gauge.weight = ($Gauge.sample - $Gauge.start) / ($Gauge.end - $Gauge.start)
first = Gauge()
second = Gauge()
first.start = 0
first.end = 8
first.sample = 2
second.start = 10
second.end = 2
second.sample = 8
assert(first.weight, 0.25)
assert(second.weight, 0.25)
first.sample = 12
assert(first.weight, 1.5)
assert(second.weight, 0.25)
first.end = 16
assert(first.weight, 0.75)
third = Gauge()
assert(third.weight, null)
third.start = 0
third.end = 4
third.sample = 1
assert(third.weight, 0.25)
delete first.sample
assert(first.weight, null)
first.sample = -4
assert(first.weight, -0.25)
assert(Gauge.length, 3)"#,
        assertions: 10,
        expected: None,
        rewrites: 1,
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves deferred inverse interpolation and rollback",
        accepted: r#"# Nucleoid preserves deferred inverse interpolation and rollback

class Sensor
sensor = Sensor()
start = sensor.lower
end = sensor.upper
sample = sensor.reading
weight = (sample - start) / (end - start)
withDeferredExtra = Math.saturate((sample - start) / (end - start), sensor.optional)
assert(weight, null)
assert(withDeferredExtra, null)
sensor.lower = 0
sensor.upper = 8
sensor.reading = 2
assert(weight, 0.25)
assert(withDeferredExtra, null)
sensor.optional = 99
assert(withDeferredExtra, 0.25)
delete sensor.reading
assert(weight, null)
assert(withDeferredExtra, null)
sensor.reading = 6
assert(weight, 0.75)
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = 4
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 6)
assert(weight, 0.75)
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(sensor.reading, 6)
delete sensor.optional
assert(withDeferredExtra, null)
assert(weight, 0.75)
sensor.optional = [1, 2]
assert(withDeferredExtra, 0.75)
sensor.reading = 2
assert(weight, 0.25)
assert(withDeferredExtra, 0.25)
weight"#,
        assertions: 20,
        expected: Some("0.25"),
        rewrites: 2,
    },
    SealedCase {
        document: "reference",
        title: "Nucleoid preserves inverse interpolation relationships",
        accepted: r#"# Nucleoid preserves inverse interpolation relationships

start = 2
end = 10
sample = 4
weight = (sample - start) / (end - start)
assert(weight, 0.25)
sample = 6
assert(weight, 0.5)
start = 0
assert(weight, 0.6)
end = 12
assert(weight, 0.5)
sample = 18
assert(weight, 1.5)"#,
        assertions: 5,
        expected: None,
        rewrites: 1,
    },
    SealedCase {
        document: "readme",
        title: "Inverse interpolation keeps live inputs unclamped",
        accepted: r#"start = 0
end = 8
sample = 2
weight = (sample - start) / (end - start)
assert(weight, 0.25)
sample = 12
assert(weight, 1.5)
end = 16
assert(weight, 0.75)"#,
        assertions: 3,
        expected: None,
        rewrites: 1,
    },
    SealedCase {
        document: "examples",
        title: "Inverse interpolation handles descending live ranges",
        accepted: r#"start = 10
end = 2
sample = 8
weight = (sample - start) / (end - start)
assert(weight, 0.25)
sample = 4
assert(weight, 0.75)
sample = 14
assert(weight, -0.5)
start = 18
assert(weight, 0.25)"#,
        assertions: 4,
        expected: None,
        rewrites: 1,
    },
];

const REWRITES: &[(&str, &str)] = &[
    (
        "withExtra = Math.saturate((2 - 0) / (8 - 0), 99)",
        "withExtra = Math.inverseLerp(0, 8, 2, 99)",
    ),
    (
        "withIgnored = Math.saturate((2 - 0) / (8 - 0), ignored)",
        "withIgnored = Math.inverseLerp(0, 8, 2, ignored)",
    ),
    (
        "withDeferredExtra = Math.saturate((sample - start) / (end - start), sensor.optional)",
        "withDeferredExtra = Math.inverseLerp(start, end, sample, sensor.optional)",
    ),
    ("empty = Math.floor()", "empty = Math.inverseLerp()"),
    ("one = Math.lerp(0)", "one = Math.inverseLerp(0)"),
    ("two = Math.lerp(0, 8)", "two = Math.inverseLerp(0, 8)"),
    (
        "literal = (null - false) / (true - false)",
        "literal = Math.inverseLerp(false, true, null)",
    ),
    (
        "weight = (sample - start) / (end - start)",
        "weight = Math.inverseLerp(start, end, sample)",
    ),
    (
        "$Gauge.weight = ($Gauge.sample - $Gauge.start) / ($Gauge.end - $Gauge.start)",
        "$Gauge.weight = Math.inverseLerp($Gauge.start, $Gauge.end, $Gauge.sample)",
    ),
];

#[test]
fn eight_sealed_v2_cases_keep_non_rhs_bytes_assertions_returns_and_fifteen_substitutions() {
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
        for &(accepted, intrinsic) in REWRITES {
            changes += candidate.matches(accepted).count();
            candidate = candidate.replace(accepted, intrinsic);
        }
        assert_eq!(changes, sealed.rewrites, "{}", sealed.title);
        assert_eq!(candidate, case.source.trim(), "{}", sealed.title);
        assert_eq!(changes, candidate.matches("Math.inverseLerp(").count());
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
                let id = ObjectId::named(name);
                assert_numeric_same(
                    runtimes[0].state.property(&id, "weight").unwrap(),
                    runtimes[1].state.property(&id, "weight").unwrap(),
                );
                for runtime in &runtimes {
                    for input in ["start", "end", "sample"] {
                        assert!(runtime.graph.reaches(
                            &NodeKey::property(&id, input),
                            &NodeKey::property(&id, "weight"),
                        ));
                    }
                }
            }
        } else {
            assert_numeric_parity(&runtimes[0], &runtimes[1], "weight");
            for runtime in &runtimes {
                for input in ["start", "end", "sample"] {
                    assert!(
                        runtime
                            .graph
                            .reaches(&NodeKey::variable(input), &NodeKey::variable("weight"),)
                    );
                }
            }
            if sealed.expected == Some("0.5") {
                assert_numeric_parity(&runtimes[0], &runtimes[1], "reconstructed");
            }
            if sealed.expected == Some("0.25") {
                assert_numeric_parity(&runtimes[0], &runtimes[1], "withDeferredExtra");
                for runtime in &runtimes {
                    assert!(runtime.graph.reaches(
                        &NodeKey::property(&ObjectId::named("sensor"), "optional"),
                        &NodeKey::variable("withDeferredExtra"),
                    ));
                }
            }
        }
        totals.0 += sealed.assertions;
        totals.1 += usize::from(sealed.expected.is_some());
        totals.2 += changes;
    }
    assert_eq!(totals, (91, 2, 15));
}

#[test]
fn live_numeric_edges_coercion_and_repeated_nan_match_ordered_controls_by_bits() {
    let template = "start = 0\nend = 8\nsample = 2\nweight = __WEIGHT__\n\
                    reconstructed = Math.lerp(start, end, weight)\nreciprocal = 1 / weight";
    let mut runtimes = [
        "Math.inverseLerp(start, end, sample)",
        "(sample - start) / (end - start)",
    ]
    .map(|expression| run(&template.replace("__WEIGHT__", expression)));
    for source in [
        "assert(weight, 0.25)",
        "sample = start\nassert(weight, 0)",
        "sample = end\nassert(weight, 1)",
        "sample = 16\nassert(weight, 2)",
        "sample = -8\nassert(weight, -1)",
        "start = 10\nend = 2\nsample = 8\nassert(weight, 0.25)",
        "sample = 12\nassert(weight, -0.25)",
        "start = 2\nend = 2\nsample = 2\nassert(Number.isNaN(weight), true)",
        "sample = 3\nassert(weight, Number.POSITIVE_INFINITY)",
        "sample = 1\nassert(weight, Number.NEGATIVE_INFINITY)",
        "start = -Number.MAX_VALUE\nend = Number.MAX_VALUE\nsample = 0\nassert(weight, 0)",
        "sample = Number.MAX_VALUE\nassert(Number.isNaN(weight), true)",
        "start = 0\nend = Number.POSITIVE_INFINITY\nsample = 8\nassert(weight, 0)",
        "sample = Number.POSITIVE_INFINITY\nassert(Number.isNaN(weight), true)",
        "start = Number.NEGATIVE_INFINITY\nend = Number.POSITIVE_INFINITY\nsample = 0",
        "start = 0\nend = 8\nsample = Number.NEGATIVE_INFINITY",
        "start = -0\nend = 1\nsample = -0",
        "start = 0\nsample = -0",
        "end = -1",
        "start = -0\nsample = 0",
        "start = 0\nend = 5e-324\nsample = 5e-324\nassert(weight, 1)",
        "end = Number.MIN_VALUE\nsample = Number.MIN_VALUE / 2\nassert(weight, 0.5)",
        "end = Number.MAX_VALUE\nsample = 5e-324\nassert(weight, 0)",
        "start = 10000000000000000\nend = start + 2\nsample = start\nassert(weight, 0)",
        "start = \" 2 \"\nend = \"10\"\nsample = \"4\"\nassert(weight, 0.25)",
        "start = false\nend = true\nsample = [0.5]\nassert(weight, 0.5)",
        "start = Date(0)\nend = Date(8)\nsample = Date(2)\nassert(weight, 0.25)",
        "start = []\nend = [\"8\"]\nsample = [\"2\"]\nassert(weight, 0.25)",
        "sample = [1, 2]\nassert(Number.isNaN(weight), true)",
        "sample = \"invalid\"\nassert(Number.isNaN(weight), true)",
        "sample = Number.NaN\nassert(Number.isNaN(weight), true)",
        "sample = Number.NaN\nassert(Number.isNaN(weight), true)",
        "start = 0\nend = 8\nsample = 2\nassert(weight, 0.25)",
    ] {
        for runtime in &mut runtimes {
            checked(runtime, source, None);
        }
        for name in ["weight", "reconstructed", "reciprocal"] {
            assert_numeric_parity(&runtimes[0], &runtimes[1], name);
        }
    }
}

#[test]
fn literal_value_kinds_in_all_three_positions_and_ignored_extras_keep_intrinsic_conversion() {
    for expression in [
        "null",
        "\" 2 \"",
        "\"\"",
        "true",
        "false",
        "[]",
        "[\"2\"]",
        "[[\"2\"]]",
        "Date(2)",
        "[1, 2]",
        "\"invalid\"",
        "{}",
        "/probe/",
        "value => value",
        "Probe",
        "Probe()",
    ] {
        for position in 0..3 {
            let mut arguments = ["0", "8", "2"];
            arguments[position] = expression;
            let [start, end, value] = arguments;
            let mut runtime = run("class Probe");
            let actual = checked(
                &mut runtime,
                &format!("Math.inverseLerp({start}, {end}, {value})"),
                None,
            );
            let expected = checked(
                &mut runtime,
                &format!("(({value}) - ({start})) / (({end}) - ({start}))"),
                None,
            );
            assert_numeric_same(&actual, &expected);
        }
    }
    run(r#"
assert(Number.isNaN(Math.inverseLerp()), true)
assert(Number.isNaN(Math.inverseLerp(0)), true)
assert(Number.isNaN(Math.inverseLerp(0, 8)), true)
assert(Math.inverseLerp(null, 8, 2), 0.25)
assert(Math.inverseLerp(2, null, 1), 0.5)
assert(Math.inverseLerp(0, 8, null), 0)
assert(Math.inverseLerp(0, 8, 2, Number.NaN, "invalid", {}, null), 0.25)
assert(Math.inverseLerp(0, 8, 2, [1, 2], Date(99), /probe/, value => value), 0.25)
class Probe
assert(Math.inverseLerp(0, 8, 2, Probe, Probe()), 0.25)
"#);
}

#[test]
fn every_supplied_expression_runs_once_left_to_right_even_after_undefined_or_nan() {
    run(r#"
class Log
log = Log()
log.order = 0
weight = Math.inverseLerp(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3,
    log.order = log.order.value * 10 + 4,
    log.order = log.order.value * 10 + 5
)
assert(log.order, 12345)
assert(weight, (123 - 1) / (12 - 1))
invalid = Math.inverseLerp(Number.NaN, 8, 2, log.order = 8)
assert(Number.isNaN(invalid), true)
assert(log.order, 8)
deferred = Math.inverseLerp(log.input, 8, 2, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.input = 0
assert(deferred, 0.25)
assert(log.order, 7)
log.order = 0
mapped = [2, 4, 6].map(value => Math.inverseLerp(0, 8, value, log.order = log.order.value * 10 + 1))
assert(mapped, [0.25, 0.5, 0.75])
assert(log.order, 111)
"#);
}

#[test]
fn undefined_at_each_position_and_extras_keeps_live_dependencies_why_deletion_and_recovery() {
    for (arguments, defined) in [
        ("probe.input, 8, 2", "0"),
        ("0, probe.input, 2", "8"),
        ("0, 8, probe.input", "2"),
        ("0, 8, 2, probe.input", "{}"),
        ("0, 8, 2, 99, probe.input", "Number.NaN"),
        ("0, 8, 2, 99, \"invalid\", probe.input", "[1, 2]"),
    ] {
        let runtime = run(&format!(
            "class Probe\nprobe = Probe()\nweight = Math.inverseLerp({arguments})\n\
             assert(weight, null)\nprobe.input = {defined}\nassert(weight, 0.25)\n\
             delete probe.input\nassert(weight, null)\nprobe.input = {defined}\nassert(weight, 0.25)"
        ));
        assert!(
            runtime
                .graph
                .retrieve(&NodeKey::variable("weight"))
                .unwrap()
                .dependencies
                .contains(&NodeKey::property(&ObjectId::named("probe"), "input"))
        );
    }
    let runtime = run(r#"
class Probe
probe = Probe()
start = 0
end = 8
sample = 2
extra = 99
weight = Math.inverseLerp(start, end, sample, extra)
trace = why weight
assert(trace[0].from, ["start", "end", "sample", "extra"])
assert(trace[0].holds, 0.25)
sample = 12
assert(trace[0].holds, 1.5)
end = 16
assert(trace[0].holds, 0.75)
extra = Number.NaN
assert(weight, 0.75)
assert(Math.unknown(probe.absent), null)
assert(Math.random(probe.absent), null)
"#);
    assert_eq!(
        dependencies(&runtime, "weight"),
        ["start", "end", "sample", "extra"]
    );
    for input in ["start", "end", "sample", "extra"] {
        assert!(
            runtime
                .graph
                .reaches(&NodeKey::variable(input), &NodeKey::variable("weight"),)
        );
    }
}

#[test]
fn strict_failures_in_all_arguments_and_after_undefined_preserve_positions_and_rollback() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 111
weight = Math.inverseLerp(0, 8, 2)
try:
    weight = Math.inverseLerp(log.order = 9, 8, 2, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 111)
assert(weight, 0.25)
try:
    Math.inverseLerp(log.absent, 8, 2, missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
try:
    Math.inverseLerp(log.absent, 8, 2, Math.random())
catch error:
    assert(error, TypeError("Math.random is not supported"))
"#);
    for (source, kind, message) in [
        (
            "Math.inverseLerp(missing, 8, 2)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.inverseLerp(0, missing, 2)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.inverseLerp(0, 8, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.inverseLerp(0, 8, 2, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.inverseLerp(log.absent, 8, 2, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.inverseLerp(0, 8, 2, 99, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.inverseLerp(log.absent, 8, 2, Math.random())",
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
        .run("log.order = 9\nMath.inverseLerp(0, 8, 2, missing)")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    checked(
        &mut runtime,
        "assert(log.order, 111)\nassert(weight, 0.25)",
        None,
    );
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
}

#[test]
fn intrinsic_literal_conversion_does_not_override_live_or_parameter_null_deferral() {
    let mut runtime = run(r#"
start = 0
end = 8
sample = 2
extra = 99
weight = Math.inverseLerp(start, end, sample, extra)
def normalized(start: Numeric, end: Numeric, sample: Numeric) => Math.inverseLerp(start, end, sample)
omitted = normalized()
fewer = normalized(0, 8)
first = normalized(null, 8, 2)
second = normalized(0, null, 2)
third = normalized(0, 8, null)
assert(omitted, null)
assert(fewer, null)
assert(first, null)
assert(second, null)
assert(third, null)
assert(normalized(0, 8, 2, 99), 0.25)
assert(Number.isNaN(Math.inverseLerp(0, 8)), true)
"#);
    for (name, restored) in [
        ("start", "0"),
        ("end", "8"),
        ("sample", "2"),
        ("extra", "99"),
    ] {
        checked(
            &mut runtime,
            &format!(
                "{name} = null\nassert(weight, null)\n{name} = {restored}\nassert(weight, 0.25)"
            ),
            None,
        );
    }
    assert!(!runtime.graph.contains(&NodeKey::variable("Numeric")));
}

#[test]
fn intrinsic_names_namespace_shadowing_members_and_noncallable_method_reads_keep_lookup() {
    let runtime = run(r#"
def Number(value) => 999
def inverseLerp(start, end, value) => 999
assert(Number("2"), 999)
assert(inverseLerp(0, 8, 2), 999)
sample = "2"
weight = Math.inverseLerp("0", "8", sample)
assert(weight, 0.25)
sample = "6"
assert(weight, 0.75)
def numeric(Number):
    return Math.inverseLerp("0", "8", "2")
assert(numeric(null), 0.25)
assert(Math.lerp(0, 8, 0.25), 2)
assert(Math.fract(-1.25), 0.75)
"#);
    assert_eq!(dependencies(&runtime, "weight"), ["sample"]);
    let runtime = run(r#"
Number = 7
weight = Math.inverseLerp("0", "8", "2")
assert(weight, 0.25)
reference = Math.inverseLerp
indirect = reference(0, 8, 2)
assert(reference, null)
assert(indirect, null)
class Namespace:
    def inverseLerp(start, end, value) => start + end + value
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.inverseLerp(2, 3, 4), 9)
def scoped(Math):
    return Math.inverseLerp(2, 3, 4)
assert(scoped(namespace), 9)
assert(scoped(null), null)
Math = namespace
assert(Math.inverseLerp(2, 3, 4), 9)
"#);
    assert!(dependencies(&runtime, "weight").is_empty());
    let mut runtime = run("Math = 7");
    let error = runtime.run("Math.inverseLerp(0, 8, 2)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.inverseLerp is not a function");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn typed_wrappers_aliases_snapshots_receivers_and_call_apis_preserve_frames_and_local_null() {
    let runtime = run(r#"
input = 2
end = 8
def normalize amount: ArbitraryType => Math.inverseLerp(0, end, amount)
alias = normalize
frozen = normalize.value
weight = alias(input)
snapshot = weight.value
assert(weight, 0.25)
assert(snapshot, 0.25)
input = 6
assert(weight, 0.75)
assert(snapshot, 0.25)
try:
    def normalize amount: ArbitraryType => Math.inverseLerp(0, end, missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(weight, 0.75)
input = 4
assert(weight, 0.5)
def normalize amount: ArbitraryType => Math.inverseLerp(0, end, amount) + 1
assert(weight, 1.5)
assert(frozen(2), 0.25)
class Reading:
    def init(start, end, sample):
        this.start = start
        this.end = end
        this.sample = sample
    def weight() => Math.inverseLerp(self.start, self.end, self.sample)
reading = Reading(10, 2, 8)
assert(reading.weight(), 0.25)
reading.sample = 4
assert(reading.weight(), 0.75)
"#);
    for name in ["alias", "input", "end"] {
        assert!(dependencies(&runtime, "weight").contains(&name), "{name}");
    }
    for name in ["ArbitraryType", "missing"] {
        assert!(!runtime.graph.contains(&NodeKey::variable(name)));
    }

    let statements = parse("Math.inverseLerp(start, end, sample)").unwrap();
    let [Stmt::Expression(expression)] = statements.as_slice() else {
        panic!("the intrinsic must remain an ordinary call");
    };
    let Ast::Call(call) = Ast::convert(expression) else {
        panic!("no new AST representation");
    };
    assert_eq!(call.arguments().len(), 3);
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.push();
    scope.declare("start", Value::number(0));
    scope.declare("end", Value::number(8));
    scope.declare("sample", Value::number(2));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    assert_numeric_same(
        &call.resolve(&mut runtime, &mut scope).unwrap(),
        &Value::number(0.25),
    );
    scope.declare("sample", Value::Null);
    let accepted = parse_expression("(sample - start) / (end - start)").unwrap();
    assert_numeric_same(
        &call.resolve(&mut runtime, &mut scope).unwrap(),
        &Ast::convert(&accepted)
            .resolve(&mut runtime, &mut scope)
            .unwrap(),
    );
    assert_numeric_same(
        &call.resolve(&mut runtime, &mut scope).unwrap(),
        &Value::number(0),
    );
    let failed = parse_expression("Math.inverseLerp(start, end, sample, missing)").unwrap();
    let error = Ast::convert(&failed)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.retrieve("sample"), Some(&Value::Null));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
}

#[test]
fn failed_relationship_and_class_rule_rewrites_restore_subscriptions_and_recovery() {
    let runtime = run(r#"
class Gauge
$Gauge.weight = Math.inverseLerp($Gauge.start, $Gauge.end, $Gauge.sample)
reading = Gauge()
reading.start = 0
reading.end = 8
reading.sample = 2
sample = reading.sample
alternative = 4
weight = Math.inverseLerp(0, 8, sample)
trace = why weight
assert(weight, 0.25)
assert(reading.weight, 0.25)
try:
    weight = Math.inverseLerp(0, 8, alternative)
    reading.sample = 6
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(weight, 0.25)
assert(reading.weight, 0.25)
assert(trace[0].from, ["sample"])
try:
    $Gauge.weight = Math.inverseLerp(missing, $Gauge.end, $Gauge.sample)
catch error:
    assert(error, ReferenceError("missing is not defined"))
alternative = 12
assert(weight, 0.25)
reading.sample = 6
assert(weight, 0.75)
assert(reading.weight, 0.75)
delete reading.sample
assert(sample, null)
assert(weight, null)
assert(reading.weight, null)
reading.sample = -4
assert(weight, -0.5)
assert(reading.weight, -0.5)
"#);
    assert_eq!(dependencies(&runtime, "weight"), ["sample"]);
    assert!(runtime.graph.reaches(
        &NodeKey::property(&ObjectId::named("reading"), "sample"),
        &NodeKey::variable("weight"),
    ));
    assert!(!runtime.graph.reaches(
        &NodeKey::variable("alternative"),
        &NodeKey::variable("weight"),
    ));
}

#[test]
fn uncaught_and_caught_application_failures_restore_inputs_and_leave_propagation_usable() {
    let mut runtimes = [
        "Math.inverseLerp(start, end, sample)",
        "(sample - start) / (end - start)",
    ]
    .map(|expression| {
        run(&format!(
            "start = 0\nend = 8\nsample = 2\nweight = {expression}\n\
             if weight > 1:\n    throw \"TOO_HIGH\""
        ))
    });
    for runtime in &mut runtimes {
        let keys = runtime.graph.keys().cloned().collect::<Vec<_>>();
        let error = runtime.run("sample = 12").unwrap_err();
        assert_eq!(error.thrown_value(), Some(&Value::string("TOO_HIGH")));
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.graph.keys().cloned().collect::<Vec<_>>(), keys);
        checked(
            runtime,
            r#"
assert(sample, 2)
assert(weight, 0.25)
try:
    sample = 16
catch error:
    assert(error, "TOO_HIGH")
assert(sample, 2)
assert(weight, 0.25)
sample = 6
assert(weight, 0.75)
"#,
            None,
        );
    }
    assert_numeric_parity(&runtimes[0], &runtimes[1], "weight");
}

#[test]
fn native_cli_preserves_exact_file_bytes_numeric_rounding_errors_and_literal_eof() {
    for (index, (source, success, stdout, stderr)) in [
        (
            "assert(Math.inverseLerp(0, 8, 2), 0.25)\nMath.inverseLerp(0, 8, 2)",
            true,
            "0.25\n",
            "",
        ),
        ("Math.inverseLerp(0, 8, 12)", true, "1.5\n", ""),
        ("Math.inverseLerp(10, 2, 14)", true, "-0.5\n", ""),
        (
            "Math.inverseLerp(0, 3, 1)",
            true,
            "0.3333333333333333\n",
            "",
        ),
        (
            "Math.inverseLerp(-Number.MAX_VALUE, Number.MAX_VALUE, 0)",
            true,
            "0.0\n",
            "",
        ),
        ("Math.inverseLerp(2, 2, 2)", true, "null\n", ""),
        ("Math.inverseLerp(2, 2, 3)", true, "null\n", ""),
        ("String(Math.inverseLerp())", true, "\"NaN\"\n", ""),
        ("String(Math.inverseLerp(0))", true, "\"NaN\"\n", ""),
        ("String(Math.inverseLerp(0, 8))", true, "\"NaN\"\n", ""),
        ("Math.inverseLerp(0, 8, null)", true, "0.0\n", ""),
        (
            "Math.inverseLerp(0, 8, 2, Number.NaN, \"invalid\", {})",
            true,
            "0.25\n",
            "",
        ),
        (
            "Math.inverseLerp(0, 8, 2, missing)",
            false,
            "",
            "ReferenceError: missing is not defined (line 1, column 1)\n",
        ),
        (
            "class Probe\nprobe = Probe()\nMath.inverseLerp(probe.absent, 8, 2, Math.random())",
            false,
            "",
            "TypeError: Math.random is not supported (line 3, column 1)\n",
        ),
        (
            "Math.inverseLerp(",
            false,
            "",
            "SyntaxError: Unexpected end of input (line 1, column 18)\n",
        ),
        (
            "Math.inverseLerp(\n",
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
                "nuc38-math-inverse-lerp-{}-{index}.nuc",
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
fn stdin_literal_eof_and_repl_error_recovery_keep_unclamped_live_inputs_and_framing() {
    let source = "start = 0\nend = 8\nsample = 2\nweight = Math.inverseLerp(start, end, sample)\n\
                  Math.inverseLerp(0, 8, 2, missing)\nsample = 12\nweight";
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
            for name in ["start", "end", "sample", "weight"] {
                assert!(stdout.contains(&format!("{name} [variable]")), "{stdout}");
            }
            assert!(stdout.contains("1.5"), "{stdout}");
            assert!(!stdout.contains("missing [variable]"), "{stdout}");
        } else {
            let banner = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";
            assert_eq!(stdout, format!("{banner}0.0\n8.0\n2.0\n0.25\n12.0\n1.5\n"));
        }
    }
}
