//! Unit-interval saturation through the unchanged static Math call path.

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
            title: "Math.saturate regression".to_string(),
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

fn pair(template: &str, candidate: &str, accepted: &str) -> [Runtime; 2] {
    let sources = [
        template.replace("__SATURATE__", candidate),
        template.replace("__SATURATE__", accepted),
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

fn assert_dependencies(runtime: &Runtime, key: NodeKey, expected: &[&str]) {
    let node = runtime.graph.retrieve(&key).unwrap();
    assert_eq!(
        node.dependencies
            .iter()
            .map(NodeKey::as_str)
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn finite_live_inputs_and_ignored_extra_reads_keep_exact_named_dependencies() {
    let runtimes = pair(
        r#"
reading = -2
extra = 99
bounded = __SATURATE__
assert(bounded, 0)
reading = 0.25
assert(bounded, 0.25)
reading = 3
assert(bounded, 1)
reading = 0.5
assert(bounded, 0.5)
reading = 1
assert(bounded, 1)
reading = -1
assert(bounded, 0)
extra = Number.NaN
assert(bounded, 0)
trace = why bounded
assert(trace[0].from, ["reading", "extra"])
assert(trace[0].holds, 0)
"#,
        "Math.saturate(reading, extra)",
        "Math.clamp(reading, 0, 1, extra)",
    );
    for runtime in runtimes {
        assert_dependencies(
            &runtime,
            NodeKey::variable("bounded"),
            &["reading", "extra"],
        );
    }
}

#[test]
fn intrinsic_conversion_preserves_all_language_value_kinds() {
    run(r#"
assert(Math.saturate(" 0.375 "), 0.375)
assert(Math.saturate("-3"), 0)
assert(Math.saturate("2"), 1)
assert(Math.saturate(""), 0)
assert(Math.saturate(false), 0)
assert(Math.saturate(true), 1)
assert(Math.saturate(null), 0)
assert(Math.saturate([]), 0)
assert(Math.saturate(["0.625"]), 0.625)
assert(Math.saturate(Date(1)), 1)
assert(Math.saturate(Date(-1)), 0)
assert(Number.isNaN(Math.saturate("invalid")), true)
assert(Number.isNaN(Math.saturate([0, 1])), true)
assert(Number.isNaN(Math.saturate({})), true)
assert(Number.isNaN(Math.saturate(/probe/)), true)
assert(Number.isNaN(Math.saturate(amount => amount)), true)
class Probe
assert(Number.isNaN(Math.saturate(Probe)), true)
assert(Number.isNaN(Math.saturate(Probe())), true)
"#);
}

#[test]
fn omission_and_numerically_ignored_extras_do_not_introduce_validation_or_coercion() {
    run(r#"
assert(Number.isNaN(Math.saturate()), true)
assert(Math.saturate(0.5, 99), 0.5)
assert(Math.saturate(0.5, Number.NaN, "invalid", {}, null), 0.5)
assert(Math.saturate(0.5, [0, 1], Date(99), /probe/, amount => amount), 0.5)
assert(Math.saturate(1 / 0), 1)
assert(Math.saturate(-1 / 0), 0)
assert(Number.isNaN(Math.saturate(0 / 0)), true)
"#);
}

#[test]
fn boundary_subnormal_zero_and_repeated_nan_updates_match_accepted_composition_bits() {
    let mut runtimes = pair(
        r#"
input = -0
bounded = __SATURATE__
reciprocal = 1 / bounded
assert(reciprocal, 1 / Math.min(Math.max(input, 0), 1))
"#,
        "Math.saturate(input)",
        "Math.clamp(input, 0, 1)",
    );
    assert_eq!(
        number_bits(&runtimes[0], "bounded"),
        number_bits(&runtimes[1], "bounded")
    );
    for source in [
        "input = 0\nassert(reciprocal, 1 / Math.min(Math.max(input, 0), 1))",
        "input = -0\nassert(reciprocal, 1 / Math.min(Math.max(input, 0), 1))",
        "input = Number.MIN_VALUE / 2\nassert(bounded, Number.MIN_VALUE / 2)",
        "input = -Number.MIN_VALUE / 2\nassert(bounded, 0)",
        "input = 1 - Number.EPSILON / 2\nassert(bounded, 1 - Number.EPSILON / 2)",
        "input = 1\nassert(bounded, 1)",
        "input = 1 + Number.EPSILON\nassert(bounded, 1)",
        "input = Number.POSITIVE_INFINITY\nassert(bounded, 1)",
        "input = Number.NEGATIVE_INFINITY\nassert(bounded, 0)",
        "input = Number.NaN\nassert(Number.isNaN(bounded), true)",
        "input = Number.NaN\nassert(Number.isNaN(bounded), true)",
        "input = 0.375\nassert(bounded, 0.375)",
    ] {
        for runtime in &mut runtimes {
            checked(runtime, source, None);
        }
        for name in ["bounded", "reciprocal"] {
            assert_eq!(
                number_bits(&runtimes[0], name),
                number_bits(&runtimes[1], name),
                "{source}: {name}"
            );
        }
    }
}

#[test]
fn every_supplied_expression_runs_once_left_to_right_even_for_ignored_values() {
    run(r#"
class Log
log = Log()
log.order = 0
bounded = Math.saturate(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3,
    log.order = log.order.value * 10 + 4,
    log.order = log.order.value * 10 + 5
)
assert(log.order, 12345)
assert(bounded, 1)
invalid = Math.saturate(Number.NaN, log.order = 8)
assert(Number.isNaN(invalid), true)
assert(log.order, 8)
deferred = Math.saturate(log.absent, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.absent = 0.75
assert(deferred, 0.75)
assert(log.order, 7)
"#);
}

#[test]
fn undefined_first_and_extra_properties_recover_after_creation_and_deletion() {
    let runtime = run(r#"
class Sensor
sensor = Sensor()
input = 0.375
extra = 99
bounded = Math.saturate(input, extra)
assert(bounded, 0.375)
extra = Number.NaN
assert(bounded, 0.375)
first = Math.saturate(sensor.reading)
optional = Math.saturate(input, 99, sensor.optional)
assert(first, null)
assert(optional, null)
sensor.reading = 2
sensor.optional = 9
assert(first, 1)
assert(optional, 0.375)
delete sensor.reading
delete sensor.optional
assert(first, null)
assert(optional, null)
sensor.reading = 0.625
sensor.optional = 2
assert(first, 0.625)
assert(optional, 0.375)
"#);
    assert_dependencies(&runtime, NodeKey::variable("bounded"), &["input", "extra"]);
    for (name, property) in [("first", "reading"), ("optional", "optional")] {
        assert!(
            runtime
                .graph
                .retrieve(&NodeKey::variable(name))
                .unwrap()
                .dependencies
                .contains(&NodeKey::property(&ObjectId::named("sensor"), property))
        );
    }
}

#[test]
fn literal_null_numeric_conversion_does_not_replace_live_null_assignment_handling() {
    run(r#"
assert(Math.saturate(null), 0)
assert(Math.saturate(0.625, null), 0.625)
input = null
bounded = Math.saturate(input)
assert(bounded, null)
input = 0.375
assert(bounded, 0.375)
input = null
assert(bounded, null)
input = 2
assert(bounded, 1)
extra = null
observed = Math.saturate(0.625, extra)
assert(observed, null)
extra = 99
assert(observed, 0.625)
extra = null
assert(observed, null)
"#);
}

#[test]
fn strict_argument_errors_preserve_positions_caught_values_and_prior_effect_rollback() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 7
bounded = Math.saturate(0.375)
try:
    bounded = Math.saturate(log.order = 9, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 7)
assert(bounded, 0.375)
try:
    Math.saturate(log.absent, missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
try:
    Math.saturate(0.5, Math.random())
catch error:
    assert(error, TypeError("Math.random is not supported"))
"#);
    for source in [
        "Math.saturate(missing)",
        "Math.saturate(0.5, missing)",
        "Math.saturate(0.5, 99, missing)",
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
    checked(
        &mut runtime,
        "assert(log.order, 7)\nassert(bounded, 0.375)\nbounded",
        Some("0.375"),
    );
}

#[test]
fn deferred_property_rules_recover_after_deletion_and_failed_rewrites() {
    let runtimes = pair(
        r#"
class Sensor
sensor = Sensor()
bounded = __SATURATE__
assert(bounded, null)
sensor.reading = -2
assert(bounded, 0)
delete sensor.reading
assert(bounded, null)
sensor.reading = 0.75
assert(bounded, 0.75)
try:
    bounded = Math.saturate(missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(bounded, 0.75)
sensor.reading = 2
assert(bounded, 1)
"#,
        "Math.saturate(sensor.reading)",
        "Math.clamp(sensor.reading, 0, 1)",
    );
    for runtime in runtimes {
        assert!(
            runtime
                .graph
                .retrieve(&NodeKey::variable("bounded"))
                .unwrap()
                .dependencies
                .contains(&NodeKey::property(&ObjectId::named("sensor"), "reading"))
        );
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
}

#[test]
fn live_class_rules_preserve_independent_instances_deletion_and_redefinition_rollback() {
    let runtimes = pair(
        r#"
class Level(reading):
    this.reading = reading
$Level.bounded = __SATURATE__
first = Level(-2)
second = Level(0.4)
assert(first.bounded, 0)
assert(second.bounded, 0.4)
first.reading = 0.8
assert(first.bounded, 0.8)
second.reading = 2
assert(second.bounded, 1)
delete first.reading
assert(first.bounded, null)
assert(second.bounded, 1)
first.reading = 0.625
assert(first.bounded, 0.625)
try:
    $Level.bounded = Math.saturate(missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(first.bounded, 0.625)
assert(second.bounded, 1)
first.reading = 0.25
assert(first.bounded, 0.25)
assert(second.bounded, 1)
"#,
        "Math.saturate($Level.reading)",
        "Math.clamp($Level.reading, 0, 1)",
    );
    for runtime in runtimes {
        for (object, other) in [("first", "second"), ("second", "first")] {
            let dependencies = &runtime
                .graph
                .retrieve(&NodeKey::property(&ObjectId::named(object), "bounded"))
                .unwrap()
                .dependencies;
            assert!(dependencies.contains(&NodeKey::property(&ObjectId::named(object), "reading")));
            assert!(!dependencies.contains(&NodeKey::property(&ObjectId::named(other), "reading")));
        }
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
}

#[test]
fn intrinsic_conversion_neither_calls_nor_tracks_shadowed_number_bindings() {
    let runtime = run(r#"
def Number(amount) => 999
assert(Number("0.375"), 999)
def numeric(Number):
    return Math.saturate(" 0.625 ")
assert(numeric(null), 0.625)
text = "0.375"
bounded = Math.saturate(text)
assert(bounded, 0.375)
text = "2"
assert(bounded, 1)
"#);
    assert_dependencies(&runtime, NodeKey::variable("bounded"), &["text"]);
    let runtime = run(r#"
Number = 7
bounded = Math.saturate("0.625")
assert(bounded, 0.625)
"#);
    assert_dependencies(&runtime, NodeKey::variable("bounded"), &[]);
}

#[test]
fn math_shadowing_ordinary_members_and_unsupported_method_value_reads_are_unchanged() {
    run(r#"
assert(Math.saturate(0.375), 0.375)
assert(Math.PI > 3, true)
reference = Math.saturate
old_reference = Math.clamp
indirect = reference(0.375)
assert(reference, null)
assert(old_reference, null)
assert(indirect, null)
class Namespace:
    def saturate(amount):
        return amount + 2
class Holder:
    def Math(amount):
        return amount + 4
namespace = Namespace()
holder = Holder()
holder.namespace = namespace
assert(holder.Math(2), 6)
assert(holder.namespace.saturate(2), 4)
holder.Math = namespace
assert(holder.Math.saturate(2), 4)
def scoped(Math):
    return Math.saturate(2)
assert(scoped(namespace), 4)
assert(scoped(null), null)
Math = namespace
assert(Math.saturate(2), 4)
"#);
    run(r#"
Math = 7
try:
    Math.saturate(0.375)
catch error:
    assert(error, TypeError("number.saturate is not a function"))
"#);
}

#[test]
fn named_typed_wrappers_and_failed_redefinitions_keep_live_reads_and_recovery() {
    let runtimes = pair(
        r#"
input = -2
extra = 99
def unit amount: ArbitraryType => __SATURATE__
bounded = unit(input)
assert(bounded, 0)
input = 0.375
assert(bounded, 0.375)
extra = 12
assert(bounded, 0.375)
try:
    def unit amount: ArbitraryType => Math.saturate(missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(bounded, 0.375)
input = 2
assert(bounded, 1)
assert(unit(0.625), 0.625)
"#,
        "Math.saturate(amount, extra)",
        "Math.clamp(amount, 0, 1, extra)",
    );
    for runtime in runtimes {
        let node = runtime
            .graph
            .retrieve(&NodeKey::variable("bounded"))
            .unwrap();
        for key in ["unit", "input", "extra"] {
            assert!(node.dependencies.contains(&NodeKey::variable(key)), "{key}");
        }
        assert!(!runtime.graph.contains(&NodeKey::variable("ArbitraryType")));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
}

#[test]
fn application_rule_failures_rollback_input_and_leave_propagation_usable() {
    let runtimes = pair(
        r#"
input = 0.4
bounded = __SATURATE__
if bounded > 0.8:
    throw "TOO_HIGH"
try:
    input = 2
catch error:
    assert(error, "TOO_HIGH")
assert(input, 0.4)
assert(bounded, 0.4)
input = 0.6
assert(bounded, 0.6)
"#,
        "Math.saturate(input)",
        "Math.clamp(input, 0, 1)",
    );
    for runtime in runtimes {
        assert_dependencies(&runtime, NodeKey::variable("bounded"), &["input"]);
    }
}

#[test]
fn clamp_lerp_extrema_rounding_constants_and_existing_errors_keep_their_behavior() {
    run(r#"
assert(Math.clamp(12, 0, 10), 10)
assert(Math.clamp(5, 10, 0), 0)
assert(Math.lerp(2, 10, 0.5), 6)
assert(Math.max(), Number.NEGATIVE_INFINITY)
assert(Math.min(), Number.POSITIVE_INFINITY)
assert(Number.isNaN(Math.max(1, Number.NaN, 2)), true)
assert(Number.isNaN(Math.min(1, Number.NaN, 2)), true)
assert(Math.round(0.49999999999999994), 0)
assert(1 / Math.round(-0.5), Number.NEGATIVE_INFINITY)
assert(Math.PI > 3, true)
assert(Math.E > 2, true)
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
fn native_cli_runs_exact_byte_files_and_keeps_literal_eof_diagnostics() {
    for (index, (source, success, stdout, stderr)) in [
        (
            "assert(Math.saturate(0.375), 0.375)\nMath.saturate(0.375)",
            true,
            "0.375\n",
            "",
        ),
        ("Math.clamp(0.375, 0, 1)", true, "0.375\n", ""),
        ("Math.saturate(null)", true, "0.0\n", ""),
        ("Math.clamp(null, 0, 1)", true, "0.0\n", ""),
        ("Math.saturate(1 / 0)", true, "1.0\n", ""),
        ("Math.saturate(-1 / 0)", true, "0.0\n", ""),
        ("String(Math.saturate())", true, "\"NaN\"\n", ""),
        ("String(Math.clamp())", true, "\"NaN\"\n", ""),
        (
            "Math.saturate(0.5, Number.NaN, \"invalid\", {})",
            true,
            "0.5\n",
            "",
        ),
        (
            "def Number(amount) => 999\nMath.saturate(\"0.625\")",
            true,
            "0.625\n",
            "",
        ),
        (
            "Math.saturate(0.5, missing)",
            false,
            "",
            "ReferenceError: missing is not defined (line 1, column 1)\n",
        ),
        (
            "Math.clamp(0.5, 0, 1, missing)",
            false,
            "",
            "ReferenceError: missing is not defined (line 1, column 1)\n",
        ),
        (
            "Math.saturate(0.5, Math.random())",
            false,
            "",
            "TypeError: Math.random is not supported (line 1, column 1)\n",
        ),
        (
            "Math.saturate(",
            false,
            "",
            "SyntaxError: Unexpected end of input (line 1, column 15)\n",
        ),
        (
            "Math.saturate(\n",
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
                "nuc23-math-saturate-{}-{index}.nuc",
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
fn normal_stdin_eof_and_repl_recovery_preserve_exact_output_without_appended_newlines() {
    let banner = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";
    for (source, values, stderr) in [
        (
            "ratio = 0.25\nbounded = Math.saturate(ratio)\nratio = 2\nbounded",
            "0.25\n0.25\n2.0\n1.0\n",
            "",
        ),
        (
            "ratio = 0.25\nbounded = Math.saturate(ratio)\nMath.saturate(0.5, missing)\nratio = 2\nbounded\nString(Math.saturate())\nMath.saturate(0.5, 99)\n.exit",
            "0.25\n0.25\n2.0\n1.0\n\"NaN\"\n0.5\n",
            "ReferenceError: missing is not defined (line 1, column 1)\n",
        ),
    ] {
        assert!(!source.ends_with('\n'));
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
            .write_all(source.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, format!("{banner}{values}").as_bytes());
        assert_eq!(output.stderr, stderr.as_bytes());
    }
}
