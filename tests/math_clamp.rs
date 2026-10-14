//! Ordered numeric clamping through the unchanged static Math call path.

mod common;

use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let statements = parse(source).unwrap();
    let protected = common::assertions::protected_assertions(&statements);
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "Math.clamp regression".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        },
        protected.len(),
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, protected.len());
    value
}

fn run(source: &str) -> Runtime {
    let mut runtime = Runtime::new();
    checked(&mut runtime, source, None);
    runtime
}

fn pair(template: &str, candidate_call: &str, accepted_call: &str) -> [Runtime; 2] {
    let sources = [
        template.replace("__BOUND__", candidate_call),
        template.replace("__BOUND__", accepted_call),
    ];
    let protected = sources
        .each_ref()
        .map(|source| common::assertions::protected_assertions(&parse(source).unwrap()));
    assert_eq!(protected[0], protected[1]);
    sources.each_ref().map(|source| run(source))
}

fn number_bits(runtime: &Runtime, name: &str) -> u64 {
    let Some(Value::Number(number)) = runtime.state.variable(name) else {
        panic!("{name} is not a number");
    };
    number.to_bits()
}

#[test]
fn finite_fractional_values_and_all_three_live_dependencies_match_composition() {
    let runtimes = pair(
        r#"
input = 1.25
lower = -0.5
upper = 0.75
bounded = __BOUND__
assert(bounded, 0.75)
input = 0.25
assert(bounded, 0.25)
lower = 0.5
assert(bounded, 0.5)
upper = 0.4
assert(bounded, 0.4)
lower = -1
input = -2.5
assert(bounded, -1)
upper = 3
input = 2.75
assert(bounded, 2.75)
trace = why bounded
assert(trace[0].from, ["input", "lower", "upper"])
assert(trace[0].holds, 2.75)
"#,
        "Math.clamp(input, lower, upper)",
        "Math.min(Math.max(input, lower), upper)",
    );
    for runtime in runtimes {
        let node = runtime
            .graph
            .retrieve(&NodeKey::variable("bounded"))
            .unwrap();
        assert_eq!(
            node.dependencies
                .iter()
                .map(NodeKey::as_str)
                .collect::<Vec<_>>(),
            ["input", "lower", "upper"]
        );
    }
}

#[test]
fn numeric_coercion_in_each_position_and_non_numeric_values_keep_existing_handling() {
    run(r#"
assert(Math.clamp(" 7 ", 0, 5), 5)
assert(Math.clamp(3, " 4 ", "5"), 4)
assert(Math.clamp(true, false, 2), 1)
assert(Math.clamp(null, -1, 1), 0)
assert(Math.clamp([], 1, 3), 1)
assert(Math.clamp(["4"], 0, 3), 3)
assert(Math.clamp(Date(12), 0, Date(10)), 10)
assert(String(Math.clamp("invalid", 0, 5)), "NaN")
assert(String(Math.clamp(2, "invalid", 5)), "NaN")
assert(String(Math.clamp(2, 0, "invalid")), "NaN")
assert(String(Math.clamp([1, 2], 0, 5)), "NaN")
assert(String(Math.clamp({}, 0, 5)), "NaN")
assert(String(Math.clamp(/probe/, 0, 5)), "NaN")
assert(String(Math.clamp(value => value, 0, 5)), "NaN")
class Probe
assert(String(Math.clamp(Probe, 0, 5)), "NaN")
input = null
bounded = Math.clamp(input, 0, 5)
assert(bounded, null)
input = 3
assert(bounded, 3)
"#);
}

#[test]
fn nan_nonfinite_equal_reversed_omitted_and_numeric_extra_inputs_do_not_throw() {
    run(r#"
assert(Math.clamp(Number.POSITIVE_INFINITY, -10, 10), 10)
assert(Math.clamp(Number.NEGATIVE_INFINITY, -10, 10), -10)
assert(Math.clamp(Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY)
assert(Math.clamp(3, 4, 4), 4)
assert(Math.clamp(5, 10, 0), 0)
assert(Math.clamp(-100, 10, 0), 0)
assert(String(Math.clamp(Number.NaN, 0, 10)), "NaN")
assert(String(Math.clamp(2, Number.NaN, 10)), "NaN")
assert(String(Math.clamp(2, 0, Number.NaN)), "NaN")
assert(String(Math.clamp()), "NaN")
assert(String(Math.clamp(2)), "NaN")
assert(String(Math.clamp(2, 0)), "NaN")
assert(Math.clamp(3, 0, 5, 99), 3)
assert(Math.clamp(3, 0, 5, Number.NaN, "invalid", {}), 3)
"#);
}

#[test]
fn all_arguments_are_evaluated_left_to_right_even_when_a_value_is_undefined() {
    run(r#"
class Log
log = Log()
log.order = 0
bounded = Math.clamp(
    log.order = 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3,
    log.order = log.order.value * 10 + 4,
    log.order = log.order.value * 10 + 5
)
assert(log.order, 12345)
assert(bounded, 12)
deferred = Math.clamp(log.absent, 0, 5, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.absent = 3
assert(deferred, 3)
assert(log.order, 7)
"#);
}

#[test]
fn extras_are_live_dependencies_and_the_any_undefined_gate_includes_them() {
    let runtime = run(r#"
class Sensor
sensor = Sensor()
input = 3
lower = 0
upper = 5
extra = 99
bounded = Math.clamp(input, lower, upper, extra)
assert(bounded, 3)
extra = Number.NaN
assert(bounded, 3)
deferred = Math.clamp(input, lower, upper, 99, sensor.optional)
assert(deferred, null)
sensor.optional = 9
assert(deferred, 3)
delete sensor.optional
assert(deferred, null)
sensor.optional = 2
assert(deferred, 3)
"#);
    let node = runtime
        .graph
        .retrieve(&NodeKey::variable("bounded"))
        .unwrap();
    assert_eq!(
        node.dependencies
            .iter()
            .map(NodeKey::as_str)
            .collect::<Vec<_>>(),
        ["input", "lower", "upper", "extra"]
    );
    assert!(
        runtime
            .graph
            .retrieve(&NodeKey::variable("deferred"))
            .unwrap()
            .dependencies
            .contains(&NodeKey::property(&ObjectId::named("sensor"), "optional"))
    );
}

#[test]
fn strict_unknown_arguments_stop_before_dispatch_and_rollback_prior_argument_effects() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 7
bounded = Math.clamp(3, 0, 5)
try:
    bounded = Math.clamp(log.order = 9, 0, 5, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 7)
assert(bounded, 3)
try:
    Math.clamp(log.absent, 0, 5, missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
try:
    Math.clamp(3, 0, 5, Math.random())
catch error:
    assert(error, TypeError("Math.random is not supported"))
"#);
    for source in [
        "Math.clamp(missing, 0, 5)",
        "Math.clamp(3, missing, 5)",
        "Math.clamp(3, 0, missing)",
        "Math.clamp(3, 0, 5, missing)",
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
    checked(
        &mut runtime,
        "assert(log.order, 7)\nassert(bounded, 3)",
        None,
    );
}

#[test]
fn deferred_properties_and_class_rules_recover_after_deletion_and_failed_rewrites() {
    let runtimes = pair(
        r#"
class Sensor
sensor = Sensor()
observed = __BOUND__
assert(observed, null)
sensor.amount = 12
assert(observed, 10)
delete sensor.amount
assert(observed, null)
sensor.amount = 3
assert(observed, 3)
try:
    observed = Math.clamp(missing, 0, 10)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(observed, 3)
sensor.amount = 6
assert(observed, 6)
class Reading(amount):
    this.amount = amount
$Reading.bounded = Math.clamp($Reading.amount, 0, 10)
first = Reading(12)
second = Reading(-3)
assert(first.bounded, 10)
assert(second.bounded, 0)
first.amount = 4
assert(first.bounded, 4)
second.amount = 8
assert(second.bounded, 8)
"#,
        "Math.clamp(sensor.amount, 0, 10)",
        "Math.min(Math.max(sensor.amount, 0), 10)",
    );
    for runtime in runtimes {
        assert!(
            runtime
                .graph
                .retrieve(&NodeKey::variable("observed"))
                .unwrap()
                .dependencies
                .contains(&NodeKey::property(&ObjectId::named("sensor"), "amount"))
        );
    }
}

#[test]
fn signed_zero_changes_and_repeated_nan_propagate_like_the_ordered_composition() {
    let mut runtimes = pair(
        r#"
input = -0.0
bounded = __BOUND__
reciprocal = 1 / bounded
assert(reciprocal, Number.NEGATIVE_INFINITY)
"#,
        "Math.clamp(input, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY)",
        "Math.min(Math.max(input, Number.NEGATIVE_INFINITY), Number.POSITIVE_INFINITY)",
    );
    for (source, expected_bits) in [
        (
            "input = 0.0\nassert(reciprocal, Number.POSITIVE_INFINITY)",
            0.0_f64.to_bits(),
        ),
        (
            "input = -0.0\nassert(reciprocal, Number.NEGATIVE_INFINITY)",
            (-0.0_f64).to_bits(),
        ),
        (
            "input = Number.NaN\nassert(String(bounded), \"NaN\")",
            f64::NAN.to_bits(),
        ),
        (
            "input = Number.NaN\nassert(String(bounded), \"NaN\")",
            f64::NAN.to_bits(),
        ),
        ("input = 2\nassert(bounded, 2)", 2.0_f64.to_bits()),
    ] {
        for runtime in &mut runtimes {
            checked(runtime, source, None);
            assert_eq!(number_bits(runtime, "bounded"), expected_bits);
        }
        assert_eq!(
            number_bits(&runtimes[0], "bounded"),
            number_bits(&runtimes[1], "bounded")
        );
    }
}

#[test]
fn namespace_shadowing_math_named_members_and_unsupported_method_reads_are_unchanged() {
    run(r#"
assert(Math.clamp(12, 0, 10), 10)
assert(Math.PI > 3, true)
reference = Math.clamp
old_reference = Math.max
indirect = reference(12, 0, 10)
assert(reference, null)
assert(old_reference, null)
assert(indirect, null)
class Namespace:
    def clamp(value, lower, upper):
        return value + lower + upper
class Holder:
    def Math(number):
        return number + 4
namespace = Namespace()
holder = Holder()
holder.namespace = namespace
assert(holder.Math(2), 6)
assert(holder.namespace.clamp(2, 3, 4), 9)
holder.Math = namespace
assert(holder.Math.clamp(2, 3, 4), 9)
def scoped(Math):
    return Math.clamp(2, 3, 4)
assert(scoped(namespace), 9)
assert(scoped(null), null)
Math = namespace
assert(Math.clamp(2, 3, 4), 9)
"#);
    run(r#"
Math = 7
try:
    Math.clamp(2, 0, 5)
catch error:
    assert(error, TypeError("number.clamp is not a function"))
"#);
}

#[test]
fn named_wrappers_and_failed_redefinitions_retain_dependency_edges_and_recovery() {
    let runtimes = pair(
        r#"
input = 12
lower = 0
upper = 10
def bounded amount: ArbitraryType => __BOUND__
result = bounded(input)
assert(result, 10)
input = 4
assert(result, 4)
lower = 6
assert(result, 6)
upper = 5
assert(result, 5)
try:
    def bounded amount: ArbitraryType => Math.clamp(missing, lower, upper)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(result, 5)
lower = 0
upper = 8
input = 7
assert(result, 7)
assert(bounded(3), 3)
"#,
        "Math.clamp(amount, lower, upper)",
        "Math.min(Math.max(amount, lower), upper)",
    );
    for runtime in runtimes {
        let node = runtime
            .graph
            .retrieve(&NodeKey::variable("result"))
            .unwrap();
        for key in ["bounded", "input", "lower", "upper"] {
            assert!(node.dependencies.contains(&NodeKey::variable(key)), "{key}");
        }
        assert!(!runtime.graph.contains(&NodeKey::variable("ArbitraryType")));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
}

#[test]
fn application_rule_failures_rollback_the_clamp_and_keep_the_queue_usable() {
    let runtimes = pair(
        r#"
input = 4
bounded = __BOUND__
if bounded > 8:
    throw "TOO_HIGH"
try:
    input = 12
catch error:
    assert(error, "TOO_HIGH")
assert(input, 4)
assert(bounded, 4)
input = 6
assert(bounded, 6)
"#,
        "Math.clamp(input, 0, 10)",
        "Math.min(Math.max(input, 0), 10)",
    );
    for runtime in runtimes {
        let node = runtime
            .graph
            .retrieve(&NodeKey::variable("bounded"))
            .unwrap();
        assert_eq!(
            node.dependencies
                .iter()
                .map(NodeKey::as_str)
                .collect::<Vec<_>>(),
            ["input"]
        );
    }
}

#[test]
fn inherited_math_errors_and_undefined_pre_dispatch_gate_remain_exact() {
    run(r#"
try:
    Math.unknown()
catch error:
    assert(error, TypeError("Math.unknown is not a function"))
try:
    Math.random()
catch error:
    assert(error, TypeError("Math.random is not supported"))
class Sensor
sensor = Sensor()
unknown = Math.unknown(sensor.absent)
random = Math.random(sensor.absent)
assert(unknown, null)
assert(random, null)
assert(Math.max(), Number.NEGATIVE_INFINITY)
assert(Math.min(), Number.POSITIVE_INFINITY)
assert(String(Math.max(1, Number.NaN, 2)), "NaN")
assert(String(Math.min(1, Number.NaN, 2)), "NaN")
assert(Math.round(0.49999999999999994), 0)
assert(1 / Math.round(-0.5), Number.NEGATIVE_INFINITY)
"#);
}

#[test]
fn native_cli_uses_ordinary_files_with_exact_eof_and_existing_diagnostics() {
    for (index, (source, success, stdout, stderr)) in [
        (
            "assert(Math.clamp(12, 0, 10), 10)\nMath.clamp(12, 0, 10)",
            true,
            "10.0\n",
            "",
        ),
        (
            "Math.clamp(3, 0, 5, missing)",
            false,
            "",
            "ReferenceError: missing is not defined (line 1, column 1)\n",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!(
                "nuc21-math-clamp-{}-{index}.nuc",
                std::process::id()
            ));
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap()
            .write_all(source.as_bytes())
            .unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg("--json")
            .arg(&path)
            .stdin(Stdio::null())
            .output();
        let cleanup = fs::remove_file(&path);
        let output = output.unwrap();
        cleanup.unwrap();
        assert_eq!(output.status.success(), success, "{output:?}");
        assert_eq!(
            String::from_utf8(output.stdout)
                .unwrap()
                .replace("\r\n", "\n"),
            stdout
        );
        assert_eq!(
            String::from_utf8(output.stderr)
                .unwrap()
                .replace("\r\n", "\n"),
            stderr
        );
    }
}

#[test]
fn repl_reports_strict_extra_argument_error_and_recovers_with_the_prior_state() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            b"input = 3\nbounded = Math.clamp(input, 0, 5)\nMath.clamp(3, 0, 5, missing)\ninput = 4\nbounded\nMath.clamp(5, 10, 0)\n.exit\n",
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .replace("\r\n", "\n");
    assert_eq!(
        stdout
            .lines()
            .filter(|line| line.parse::<f64>().is_ok())
            .collect::<Vec<_>>(),
        ["3.0", "3.0", "4.0", "4.0", "0.0"]
    );
    assert_eq!(
        String::from_utf8(output.stderr)
            .unwrap()
            .replace("\r\n", "\n"),
        "ReferenceError: missing is not defined (line 1, column 1)\n"
    );
}
