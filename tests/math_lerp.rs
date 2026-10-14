//! Ordered numeric interpolation through the unchanged static Math call path.

mod common;

use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let protected = common::assertions::protected_assertions(&parse(source).unwrap());
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "Math.lerp regression".to_string(),
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

fn pair(template: &str, candidate_expression: &str, accepted_expression: &str) -> [Runtime; 2] {
    let sources = [
        template.replace("__LERP__", candidate_expression),
        template.replace("__LERP__", accepted_expression),
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
fn fractional_results_extrapolation_and_all_three_live_inputs_match_ordered_arithmetic() {
    let runtimes = pair(
        r#"
start = 0
end = 10
weight = 0.25
position = __LERP__
assert(position, 2.5)
start = 2
assert(position, 4)
end = 18
assert(position, 6)
weight = 0.5
assert(position, 10)
weight = -0.5
assert(position, -6)
weight = 1.5
assert(position, 26)
trace = why position
assert(trace[0].from, ["start", "end", "weight"])
assert(trace[0].holds, 26)
"#,
        "Math.lerp(start, end, weight)",
        "Number(start) + (Number(end) - Number(start)) * Number(weight)",
    );
    for runtime in runtimes {
        let node = runtime
            .graph
            .retrieve(&NodeKey::variable("position"))
            .unwrap();
        assert_eq!(
            node.dependencies
                .iter()
                .map(NodeKey::as_str)
                .collect::<Vec<_>>(),
            ["start", "end", "weight"]
        );
    }
}

#[test]
fn intrinsic_numeric_coercion_handles_each_position_and_non_numeric_value_kind() {
    run(r#"
assert(Math.lerp(" 2 ", "10", "0.25"), 4)
assert(Math.lerp(false, true, true), 1)
assert(Math.lerp(null, 4, 0.5), 2)
assert(Math.lerp([], 4, 0.5), 2)
assert(Math.lerp(["2"], ["10"], ["0.25"]), 4)
assert(Math.lerp(Date(0), Date(10), 0.5), 5)
assert(String(Math.lerp("invalid", 10, 0.5)), "NaN")
assert(String(Math.lerp(2, "invalid", 0.5)), "NaN")
assert(String(Math.lerp(2, 10, "invalid")), "NaN")
assert(String(Math.lerp([1, 2], 10, 0.5)), "NaN")
assert(String(Math.lerp({}, 10, 0.5)), "NaN")
assert(String(Math.lerp(2, /probe/, 0.5)), "NaN")
assert(String(Math.lerp(2, 10, amount => amount)), "NaN")
class Probe
assert(String(Math.lerp(Probe, 10, 0.5)), "NaN")
"#);
}

#[test]
fn literal_null_conversion_does_not_override_live_null_assignment_context() {
    run(r#"
assert(Math.lerp(null, 4, 0.5), 2)
assert(Math.lerp(2, 10, 0.5, null), 6)
start = null
end = 10
weight = 0.5
position = Math.lerp(start, end, weight)
assert(position, null)
start = 2
assert(position, 6)
end = null
assert(position, null)
end = 10
assert(position, 6)
weight = null
assert(position, null)
weight = 0.25
assert(position, 4)
extra = null
observed = Math.lerp(2, 10, 0.5, extra)
assert(observed, null)
extra = 99
assert(observed, 6)
"#);
}

#[test]
fn equal_reversed_nonfinite_overflow_cancellation_and_arity_have_no_shortcuts() {
    run(r#"
assert(Math.lerp(2, 10, 0), 2)
assert(Math.lerp(2, 10, 1), 10)
assert(Math.lerp(4, 4, 2), 4)
assert(Math.lerp(10, 2, 0.25), 8)
assert(Math.lerp(2, 10, 2), 18)
assert(Math.lerp(0, Number.POSITIVE_INFINITY, 0.5), Number.POSITIVE_INFINITY)
assert(Math.lerp(0, Number.NEGATIVE_INFINITY, 0.5), Number.NEGATIVE_INFINITY)
assert(String(Math.lerp(0, Number.POSITIVE_INFINITY, 0)), "NaN")
assert(String(Math.lerp(Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY, 0.5)), "NaN")
assert(String(Math.lerp(Number.POSITIVE_INFINITY, 1, 0)), "NaN")
assert(String(Math.lerp(Number.NaN, 10, 0.5)), "NaN")
assert(String(Math.lerp(2, Number.NaN, 0.5)), "NaN")
assert(String(Math.lerp(2, 10, Number.NaN)), "NaN")
assert(String(Math.lerp()), "NaN")
assert(String(Math.lerp(2)), "NaN")
assert(String(Math.lerp(2, 10)), "NaN")
assert(Math.lerp(2, 10, 0.5, 99), 6)
assert(Math.lerp(2, 10, 0.5, Number.NaN, "invalid", {}), 6)
assert(String(Math.lerp(-Number.MAX_VALUE, Number.MAX_VALUE, 0)), "NaN")
assert(Math.lerp(-Number.MAX_VALUE, Number.MAX_VALUE, 0.5), Number.POSITIVE_INFINITY)
assert(Math.lerp(Number.MAX_VALUE, -Number.MAX_VALUE, 1), Number.NEGATIVE_INFINITY)
assert(Math.lerp(10000000000000000, 1, 1), 0)
"#);
}

#[test]
fn every_argument_expression_runs_once_left_to_right_including_ignored_extras() {
    run(r#"
class Log
log = Log()
log.order = 0
position = Math.lerp(
    log.order = 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3,
    log.order = log.order.value * 10 + 4,
    log.order = log.order.value * 10 + 5
)
assert(log.order, 12345)
assert(position, 1354)
zero_infinite = Math.lerp(0, Number.POSITIVE_INFINITY, 0, log.order = 6)
assert(String(zero_infinite), "NaN")
assert(log.order, 6)
deferred = Math.lerp(log.absent, 10, 0.5, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.absent = 2
assert(deferred, 6)
assert(log.order, 7)
"#);
}

#[test]
fn all_supplied_undefined_positions_defer_and_ignored_extras_remain_live_dependencies() {
    let runtime = run(r#"
class Sensor
sensor = Sensor()
start = 2
end = 10
weight = 0.5
extra = 99
position = Math.lerp(start, end, weight, extra)
assert(position, 6)
extra = Number.NaN
assert(position, 6)
deferred = Math.lerp(start, end, weight, 99, sensor.optional)
assert(deferred, null)
sensor.optional = 9
assert(deferred, 6)
delete sensor.optional
assert(deferred, null)
sensor.optional = 2
assert(deferred, 6)
first = Math.lerp(sensor.start, 10, 0.5)
second = Math.lerp(2, sensor.end, 0.5)
third = Math.lerp(2, 10, sensor.weight)
assert(first, null)
assert(second, null)
assert(third, null)
sensor.start = 2
sensor.end = 10
sensor.weight = 0.5
assert(first, 6)
assert(second, 6)
assert(third, 6)
delete sensor.start
delete sensor.end
delete sensor.weight
assert(first, null)
assert(second, null)
assert(third, null)
"#);
    let node = runtime
        .graph
        .retrieve(&NodeKey::variable("position"))
        .unwrap();
    assert_eq!(
        node.dependencies
            .iter()
            .map(NodeKey::as_str)
            .collect::<Vec<_>>(),
        ["start", "end", "weight", "extra"]
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
fn strict_unknown_arguments_and_extra_errors_rollback_prior_argument_effects() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 7
position = Math.lerp(2, 10, 0.5)
try:
    position = Math.lerp(log.order = 9, 10, 0.5, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 7)
assert(position, 6)
try:
    Math.lerp(log.absent, 10, 0.5, missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
try:
    Math.lerp(2, 10, 0.5, Math.random())
catch error:
    assert(error, TypeError("Math.random is not supported"))
"#);
    for source in [
        "Math.lerp(missing, 10, 0.5)",
        "Math.lerp(2, missing, 0.5)",
        "Math.lerp(2, 10, missing)",
        "Math.lerp(2, 10, 0.5, missing)",
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
    checked(
        &mut runtime,
        "assert(log.order, 7)\nassert(position, 6)\nposition",
        Some("6"),
    );
}

#[test]
fn numeric_only_deferred_properties_recover_like_raw_arithmetic_after_failed_updates() {
    let runtimes = pair(
        r#"
class Sensor
sensor = Sensor()
end = 10
weight = 0.5
position = __LERP__
assert(position, null)
sensor.start = 2
assert(position, 6)
delete sensor.start
assert(position, null)
sensor.start = 4
assert(position, 7)
try:
    position = Math.lerp(missing, end, weight)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(position, 7)
end = 20
assert(position, 12)
weight = -0.5
assert(position, -4)
"#,
        "Math.lerp(sensor.start, end, weight)",
        "sensor.start + (end - sensor.start) * weight",
    );
    for runtime in runtimes {
        assert!(
            runtime
                .graph
                .retrieve(&NodeKey::variable("position"))
                .unwrap()
                .dependencies
                .contains(&NodeKey::property(&ObjectId::named("sensor"), "start"))
        );
    }
}

#[test]
fn numeric_only_class_rules_keep_independent_instances_deletions_and_rollback() {
    let runtimes = pair(
        r#"
class Blend(start, end, weight):
    this.start = start
    this.end = end
    this.weight = weight
$Blend.position = __LERP__
first = Blend(0, 10, 0.25)
second = Blend(10, 0, 0.5)
assert(first.position, 2.5)
assert(second.position, 5)
first.weight = 0.75
assert(first.position, 7.5)
second.end = 20
assert(second.position, 15)
first.start = 2
assert(first.position, 8)
delete first.start
assert(first.position, null)
first.start = 4
assert(first.position, 8.5)
try:
    $Blend.position = Math.lerp(missing, $Blend.end, $Blend.weight)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(first.position, 8.5)
assert(second.position, 15)
first.end = 20
assert(first.position, 16)
"#,
        "Math.lerp($Blend.start, $Blend.end, $Blend.weight)",
        "$Blend.start + ($Blend.end - $Blend.start) * $Blend.weight",
    );
    for runtime in runtimes {
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
        assert!(
            runtime
                .graph
                .retrieve(&NodeKey::property(&ObjectId::named("first"), "position"))
                .unwrap()
                .dependencies
                .contains(&NodeKey::property(&ObjectId::named("first"), "start"))
        );
    }
}

#[test]
fn signed_zero_changes_and_repeated_nan_match_the_ordered_binary_arithmetic_bits() {
    let mut runtimes = pair(
        r#"
start = -0.0
end = 0.0
weight = -1
position = __LERP__
reciprocal = 1 / position
assert(reciprocal, Number.NEGATIVE_INFINITY)
"#,
        "Math.lerp(start, end, weight)",
        "Number(start) + (Number(end) - Number(start)) * Number(weight)",
    );
    for (source, expected_bits) in [
        (
            "weight = 1\nassert(reciprocal, Number.POSITIVE_INFINITY)",
            0.0_f64.to_bits(),
        ),
        (
            "weight = -1\nassert(reciprocal, Number.NEGATIVE_INFINITY)",
            (-0.0_f64).to_bits(),
        ),
        (
            "start = 0.0\nassert(reciprocal, Number.POSITIVE_INFINITY)",
            0.0_f64.to_bits(),
        ),
        (
            "start = -0.0\nassert(reciprocal, Number.NEGATIVE_INFINITY)",
            (-0.0_f64).to_bits(),
        ),
    ] {
        for runtime in &mut runtimes {
            checked(runtime, source, None);
            assert_eq!(number_bits(runtime, "position"), expected_bits);
        }
    }
    for source in [
        "start = Number.NaN\nassert(String(position), \"NaN\")",
        "start = Number.NaN\nassert(String(position), \"NaN\")",
        "start = 2\nend = 10\nweight = 0.5\nassert(position, 6)",
    ] {
        for runtime in &mut runtimes {
            checked(runtime, source, None);
        }
        assert_eq!(
            number_bits(&runtimes[0], "position"),
            number_bits(&runtimes[1], "position")
        );
    }
}

#[test]
fn intrinsic_coercion_does_not_call_or_track_a_shadowed_number_namespace() {
    let runtime = run(r#"
def Number(amount) => 999
assert(Number("2"), 999)
def numeric(Number):
    return Math.lerp("2", "10", "0.5")
assert(numeric(null), 6)
weight = 0.5
position = Math.lerp("2", "10", weight)
assert(position, 6)
weight = 0.25
assert(position, 4)
"#);
    let node = runtime
        .graph
        .retrieve(&NodeKey::variable("position"))
        .unwrap();
    assert_eq!(
        node.dependencies
            .iter()
            .map(NodeKey::as_str)
            .collect::<Vec<_>>(),
        ["weight"]
    );
    let runtime =
        run("Number = 7\nposition = Math.lerp(\"2\", \"10\", \"0.5\")\nassert(position, 6)");
    assert!(
        !runtime
            .graph
            .retrieve(&NodeKey::variable("position"))
            .unwrap()
            .dependencies
            .contains(&NodeKey::variable("Number"))
    );
}

#[test]
fn math_shadowing_ordinary_math_members_and_unsupported_method_reads_stay_unchanged() {
    run(r#"
assert(Math.lerp(2, 10, 0.5), 6)
assert(Math.PI > 3, true)
reference = Math.lerp
old_reference = Math.max
indirect = reference(2, 10, 0.5)
assert(reference, null)
assert(old_reference, null)
assert(indirect, null)
class Namespace:
    def lerp(start, end, weight):
        return start + end + weight
class Holder:
    def Math(number):
        return number + 4
namespace = Namespace()
holder = Holder()
holder.namespace = namespace
assert(holder.Math(2), 6)
assert(holder.namespace.lerp(2, 3, 4), 9)
holder.Math = namespace
assert(holder.Math.lerp(2, 3, 4), 9)
def scoped(Math):
    return Math.lerp(2, 3, 4)
assert(scoped(namespace), 9)
assert(scoped(null), null)
Math = namespace
assert(Math.lerp(2, 3, 4), 9)
"#);
    run(r#"
Math = 7
try:
    Math.lerp(2, 10, 0.5)
catch error:
    assert(error, TypeError("number.lerp is not a function"))
"#);
}

#[test]
fn named_wrappers_and_failed_redefinitions_preserve_dependencies_and_recovery() {
    let runtimes = pair(
        r#"
input = 2
end = 10
weight = 0.5
def blend amount: ArbitraryType => __LERP__
result = blend(input)
assert(result, 6)
input = 4
assert(result, 7)
end = 20
assert(result, 12)
weight = 0.25
assert(result, 8)
try:
    def blend amount: ArbitraryType => Math.lerp(missing, end, weight)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(result, 8)
input = 8
assert(result, 11)
assert(blend(0), 5)
"#,
        "Math.lerp(amount, end, weight)",
        "Number(amount) + (Number(end) - Number(amount)) * Number(weight)",
    );
    for runtime in runtimes {
        let node = runtime
            .graph
            .retrieve(&NodeKey::variable("result"))
            .unwrap();
        for key in ["blend", "input", "end", "weight"] {
            assert!(node.dependencies.contains(&NodeKey::variable(key)), "{key}");
        }
        assert!(!runtime.graph.contains(&NodeKey::variable("ArbitraryType")));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
}

#[test]
fn application_rule_failures_rollback_weight_updates_and_keep_propagation_usable() {
    let runtimes = pair(
        r#"
start = 0
end = 10
weight = 0.4
position = __LERP__
if position > 8:
    throw "TOO_HIGH"
try:
    weight = 1.2
catch error:
    assert(error, "TOO_HIGH")
assert(weight, 0.4)
assert(position, 4)
weight = 0.6
assert(position, 6)
"#,
        "Math.lerp(start, end, weight)",
        "Number(start) + (Number(end) - Number(start)) * Number(weight)",
    );
    for runtime in runtimes {
        assert_eq!(
            runtime
                .graph
                .retrieve(&NodeKey::variable("position"))
                .unwrap()
                .dependencies
                .iter()
                .map(NodeKey::as_str)
                .collect::<Vec<_>>(),
            ["start", "end", "weight"]
        );
    }
}

#[test]
fn existing_math_clamp_extrema_rounding_and_exact_errors_are_unchanged() {
    run(r#"
assert(Math.clamp(12, 0, 10), 10)
assert(Math.clamp(5, 10, 0), 0)
assert(Math.max(), Number.NEGATIVE_INFINITY)
assert(Math.min(), Number.POSITIVE_INFINITY)
assert(String(Math.max(1, Number.NaN, 2)), "NaN")
assert(String(Math.min(1, Number.NaN, 2)), "NaN")
assert(Math.round(0.49999999999999994), 0)
assert(1 / Math.round(-0.5), Number.NEGATIVE_INFINITY)
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
"#);
}

#[test]
fn native_cli_uses_owned_ordinary_files_with_exact_bytes_and_eof_diagnostics() {
    for (index, (source, success, stdout, stderr)) in [
        (
            "assert(Math.lerp(2, 10, 0.5), 6)\nMath.lerp(2, 10, 0.5)",
            true,
            "6.0\n",
            "",
        ),
        ("Math.lerp(2, 10, 2)", true, "18.0\n", ""),
        (
            "String(Math.lerp(0, Number.POSITIVE_INFINITY, 0))",
            true,
            "\"NaN\"\n",
            "",
        ),
        (
            "Math.lerp(2, 10, 0.5, missing)",
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
                "nuc22-math-lerp-{}-{index}.nuc",
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
fn repl_exact_stdin_recovers_after_a_strict_extra_error_without_an_appended_newline() {
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
            b"start = 2\nend = 10\nweight = 0.5\nposition = Math.lerp(start, end, weight)\nMath.lerp(2, 10, 0.5, missing)\nend = 18\nposition\nMath.lerp(2, 10, 2)\n.exit",
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\r\n", "\n")
            .lines()
            .filter(|line| line.parse::<f64>().is_ok())
            .collect::<Vec<_>>(),
        ["2.0", "10.0", "0.5", "6.0", "18.0", "10.0", "18.0"]
    );
    assert_eq!(
        String::from_utf8(output.stderr)
            .unwrap()
            .replace("\r\n", "\n"),
        "ReferenceError: missing is not defined (line 1, column 1)\n"
    );
}
