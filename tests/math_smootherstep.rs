//! Ordered quintic easing beside independent accepted arithmetic controls.
//! These ordinary tests need neither Git nor external proposal artifacts.

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
            title: "Math.smootherstep regression".to_string(),
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

fn assert_numeric_parity(left: &Runtime, left_name: &str, right: &Runtime, right_name: &str) {
    let (Some(Value::Number(left)), Some(Value::Number(right))) = (
        left.state.variable(left_name),
        right.state.variable(right_name),
    ) else {
        panic!("{left_name} and {right_name} must both be numeric");
    };
    if right.is_nan() {
        assert!(left.is_nan(), "{left_name}: {left} is not NaN");
    } else {
        assert_eq!(left.to_bits(), right.to_bits(), "{left_name}: {right_name}");
    }
}

#[test]
fn eight_sealed_identities_keep_controls_order_assertions_returns_and_thirteen_rewrites() {
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
    let expected = [
        (
            "spec",
            "Nucleoid keeps quintic smootherstep reactive to all three inputs",
            16,
            Some("0.5"),
        ),
        (
            "spec",
            "Nucleoid preserves numeric coercion while applying quintic smootherstep",
            10,
            None,
        ),
        (
            "spec",
            "Nucleoid preserves special values and omitted smootherstep inputs",
            24,
            None,
        ),
        (
            "spec",
            "Nucleoid applies quintic smootherstep to live class rules",
            19,
            None,
        ),
        (
            "spec",
            "Nucleoid preserves deferred reads and rollback for quintic smootherstep",
            17,
            Some("0.103515625"),
        ),
        (
            "reference",
            "Nucleoid preserves quintic smootherstep relationships",
            4,
            None,
        ),
        (
            "readme",
            "Quintic smootherstep keeps input changes live",
            3,
            None,
        ),
        (
            "examples",
            "Smooth quintic transitions preserve clamped endpoints",
            8,
            None,
        ),
    ];
    let rewrites = [
        (
            "eased = Math.smootherstep(lower, upper, value)",
            "eased = unit * unit * unit * (unit * (6 * unit - 15) + 10)",
        ),
        (
            "eased = Math.smootherstep(0, 1, value)",
            "eased = unit * unit * unit * (unit * (6 * unit - 15) + 10)",
        ),
        (
            "$Curve.eased = Math.smootherstep(lower, upper, $Curve.sample)",
            "$Curve.eased = $Curve.unit * $Curve.unit * $Curve.unit * ($Curve.unit * (6 * $Curve.unit - 15) + 10)",
        ),
        ("empty = Math.smootherstep()", "empty = Math.lerp()"),
        ("one = Math.smootherstep(0)", "one = Math.lerp(0)"),
        ("two = Math.smootherstep(0, 1)", "two = Math.lerp(0, 1)"),
        (
            "withExtra = Math.smootherstep(0, 1, 0.5, 99)",
            "withExtra = Math.saturate(half * half * half * (half * (6 * half - 15) + 10), 99)",
        ),
        (
            "withIgnored = Math.smootherstep(0, 1, 0.5, ignored)",
            "withIgnored = Math.saturate(half * half * half * (half * (6 * half - 15) + 10), ignored)",
        ),
    ];
    let mut previous = HashMap::new();
    let mut totals = (0, 0, 0);
    for (document, title, count, expected_return) in expected {
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == document)
            .unwrap()
            .1;
        let matching = cases
            .iter()
            .enumerate()
            .filter(|(_, case)| case.title == title)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "{document}: {title}");
        let (position, case) = matching[0];
        if let Some(before) = previous.insert(document, position) {
            assert!(before < position, "{document}: relative order changed");
        }
        assert_eq!(case.expected.as_deref(), expected_return, "{title}");
        assert!(case.source.contains("unit = Math.saturate("), "{title}");
        let mut accepted = case.source.clone();
        let mut changes = 0;
        for (candidate, control) in rewrites {
            changes += accepted.matches(candidate).count();
            accepted = accepted.replace(candidate, control);
        }
        assert_eq!(changes, case.source.matches("Math.smootherstep(").count());
        let actual_ast = parse(&case.source).unwrap();
        let accepted_ast = parse(&accepted).unwrap();
        assert_ne!(
            actual_ast, accepted_ast,
            "{title}: distinct controls required"
        );
        let protected = common::assertions::protected_assertions(&actual_ast);
        assert_eq!(protected.len(), count, "{title}");
        assert_eq!(
            protected,
            common::assertions::protected_assertions(&accepted_ast)
        );
        let control = common::Case {
            title: case.title.clone(),
            source: accepted,
            expected: case.expected.clone(),
        };
        let mut runtimes = [Runtime::new(), Runtime::new()];
        for (runtime, source) in runtimes.iter_mut().zip([case, &control]) {
            common::check_with_count(runtime, source, count)
                .unwrap_or_else(|reason| panic!("{title}: {reason}"));
            assert_eq!(runtime.assertions_run(), count, "{title}");
        }
        if !title.contains("class rules") {
            let inputs = if document == "readme" {
                ["value"].as_slice()
            } else {
                ["lower", "upper", "value"].as_slice()
            };
            assert_eq!(dependencies(&runtimes[0], "eased"), inputs);
            assert_eq!(dependencies(&runtimes[1], "eased"), ["unit"]);
            for runtime in &runtimes {
                for input in inputs {
                    assert!(
                        runtime
                            .graph
                            .reaches(&NodeKey::variable(*input), &NodeKey::variable("eased"),)
                    );
                }
            }
            assert_numeric_parity(&runtimes[0], "eased", &runtimes[1], "eased");
        }
        totals.0 += count;
        totals.1 += usize::from(expected_return.is_some());
        totals.2 += changes;
    }
    assert_eq!(totals, (101, 2, 13));
}

#[test]
fn reactive_boundary_bits_match_quintic_controls_without_changing_cubic_results() {
    let template = r#"
lower = 0
upper = 1
value = -0
unit = Math.saturate((value - lower) / (upper - lower))
eased = __EASED__
reciprocal = 1 / eased
cubic = Math.smoothstep(lower, upper, value)
cubic_control = unit * unit * (3 - 2 * unit)
"#;
    let mut runtimes = [
        "Math.smootherstep(lower, upper, value)",
        "unit * unit * unit * (unit * (6 * unit - 15) + 10)",
    ]
    .map(|expression| run(&template.replace("__EASED__", expression)));
    for source in [
        "value = 0",
        "value = -0\nlower = -0",
        "value = Number.MIN_VALUE / 2",
        "value = -Number.MIN_VALUE / 2",
        "value = 0.1",
        "value = 0.5 - Number.EPSILON / 4",
        "value = 0.5 + Number.EPSILON / 2",
        "value = 1 - Number.EPSILON / 2\nassert(eased > 1, true)",
        "value = 1",
        "value = 1 + Number.EPSILON",
        "value = Number.POSITIVE_INFINITY",
        "value = Number.NEGATIVE_INFINITY",
        "value = Number.NaN",
        "value = Number.NaN",
        "lower = 8\nupper = 0\nvalue = 2",
        "lower = 2\nupper = 2\nvalue = 2",
        "value = 1",
        "value = 3",
        "lower = -Number.MAX_VALUE\nupper = Number.MAX_VALUE\nvalue = 0",
        "value = Number.MAX_VALUE",
        "lower = Number.NEGATIVE_INFINITY\nupper = Number.POSITIVE_INFINITY\nvalue = 0",
        "lower = 0\nupper = 8\nvalue = 2\nassert(eased, 0.103515625)\nassert(cubic, 0.15625)",
    ] {
        for runtime in &mut runtimes {
            checked(runtime, source, None);
            assert_numeric_parity(runtime, "cubic", runtime, "cubic_control");
        }
        for name in ["unit", "eased", "reciprocal"] {
            assert_numeric_parity(&runtimes[0], name, &runtimes[1], name);
        }
    }
}

#[test]
fn argument_effects_strict_errors_and_rollback_still_precede_numeric_dispatch() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 0
eased = Math.smootherstep(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3,
    log.order = log.order.value * 10 + 4,
    log.order = log.order.value * 10 + 5
)
assert(log.order, 12345)
assert(eased, 1)
invalid = Math.smootherstep(Number.NaN, 1, 0.5, log.order = 8)
assert(Number.isNaN(invalid), true)
assert(log.order, 8)
deferred = Math.smootherstep(log.lower, 1, 0.5, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.lower = 0
assert(deferred, 0.5)
assert(log.order, 7)
log.order = 0
mapped = [1, 2, 3].map(value => Math.smootherstep(0, 4, value, log.order = log.order.value * 10 + value))
assert(mapped, [0.103515625, 0.5, 0.896484375])
assert(log.order, 123)
try:
    Math.smootherstep(log.order = 9, 8, 2, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 123)
try:
    Math.smootherstep(log.absent, 8, 2, Math.random())
catch error:
    assert(error, TypeError("Math.random is not supported"))
"#);
    for source in [
        "Math.smootherstep(missing, 8, 2)",
        "Math.smootherstep(0, missing, 2)",
        "Math.smootherstep(0, 8, missing)",
        "Math.smootherstep(0, 8, 2, missing)",
        "Math.smootherstep(log.absent, 8, 2, missing)",
    ] {
        let keys = runtime.graph.keys().cloned().collect::<Vec<_>>();
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.graph.keys().cloned().collect::<Vec<_>>(), keys);
    }
    let error = runtime
        .run("log.order = 9\nMath.smootherstep(0, 8, 2, missing)")
        .unwrap_err();
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    checked(
        &mut runtime,
        "assert(log.order, 123)\nassert(deferred, 0.5)",
        None,
    );
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
}

#[test]
fn undefined_in_each_numeric_or_extra_position_retains_reads_and_recovers() {
    for (arguments, defined) in [
        ("probe.input, 8, 2", "0"),
        ("0, probe.input, 2", "8"),
        ("0, 8, probe.input", "2"),
        ("0, 8, 2, probe.input", "{}"),
        ("0, 8, 2, 99, probe.input", "Number.NaN"),
    ] {
        let runtime = run(&format!(
            "class Probe\nprobe = Probe()\neased = Math.smootherstep({arguments})\n\
             assert(eased, null)\nprobe.input = {defined}\nassert(eased, 0.103515625)\n\
             delete probe.input\nassert(eased, null)\nprobe.input = {defined}\nassert(eased, 0.103515625)"
        ));
        assert!(
            runtime
                .graph
                .retrieve(&NodeKey::variable("eased"))
                .unwrap()
                .dependencies
                .contains(&NodeKey::property(&ObjectId::named("probe"), "input"))
        );
    }
}

#[test]
fn intrinsic_coercion_does_not_replace_live_null_or_parameter_deferral() {
    let mut runtime = run(r#"
assert(Math.smootherstep(" 0 ", "8", "2"), 0.103515625)
assert(Math.smootherstep(false, true, true), 1)
assert(Math.smootherstep(null, 1, 0.5), 0.5)
assert(Math.smootherstep(0, null, 1), 1)
assert(Math.smootherstep(0, 1, null), 0)
assert(Math.smootherstep([], ["8"], ["6"]), 0.896484375)
assert(Math.smootherstep(Date(0), Date(8), Date(4)), 0.5)
assert(Number.isNaN(Math.smootherstep([0, 1], 1, 0.5)), true)
assert(Number.isNaN(Math.smootherstep(0, {}, 0.5)), true)
assert(Number.isNaN(Math.smootherstep(0, 1, /probe/)), true)
assert(Number.isNaN(Math.smootherstep(value => value, 1, 0.5)), true)
assert(Number.isNaN(Math.smootherstep()), true)
assert(Number.isNaN(Math.smootherstep(0)), true)
assert(Number.isNaN(Math.smootherstep(0, 1)), true)
assert(Math.smootherstep(0, 8, 2, Number.NaN, "invalid", {}, null), 0.103515625)
lower = 0
upper = 8
value = 2
extra = 99
eased = Math.smootherstep(lower, upper, value, extra)
def ease lower: Bound, upper: Bound, value: Input => Math.smootherstep(lower, upper, value)
missingResult = ease()
fewer = ease(0, 1)
parameter = ease(null, 1, 0.5)
assert(missingResult, null)
assert(fewer, null)
assert(parameter, null)
assert(ease(0, 8, 2, 99), 0.103515625)
"#);
    for (name, restored) in [
        ("lower", "0"),
        ("upper", "8"),
        ("value", "2"),
        ("extra", "99"),
    ] {
        checked(
            &mut runtime,
            &format!(
                "{name} = null\nassert(eased, null)\n{name} = {restored}\nassert(eased, 0.103515625)"
            ),
            None,
        );
    }
}

#[test]
fn intrinsic_names_shadowing_ordinary_members_and_method_reads_keep_existing_lookup() {
    let runtime = run(r#"
def Number(value) => 999
def saturate(value) => 999
assert(Number("2"), 999)
assert(saturate(0.25), 999)
text = "2"
eased = Math.smootherstep("0", "8", text)
assert(eased, 0.103515625)
text = "6"
assert(eased, 0.896484375)
def numeric(Number):
    return Math.smootherstep("0", "8", "4")
assert(numeric(null), 0.5)
assert(Math.saturate(0.25), 0.25)
"#);
    assert_eq!(dependencies(&runtime, "eased"), ["text"]);
    let runtime = run(r#"
Number = 7
eased = Math.smootherstep("0", "8", "2")
assert(eased, 0.103515625)
reference = Math.smootherstep
indirect = reference(0, 8, 2)
assert(reference, null)
assert(indirect, null)
class Namespace:
    def smootherstep lower, upper, value => lower + upper + value
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.smootherstep(2, 3, 4), 9)
def scoped(Math):
    return Math.smootherstep(2, 3, 4)
assert(scoped(namespace), 9)
assert(scoped(null), null)
Math = namespace
assert(Math.smootherstep(2, 3, 4), 9)
"#);
    assert!(dependencies(&runtime, "eased").is_empty());
    assert!(!runtime.graph.contains(&NodeKey::variable("Math.saturate")));
    run(r#"
Math = 7
try:
    Math.smootherstep(0, 8, 2)
catch error:
    assert(error, TypeError("number.smootherstep is not a function"))
"#);
}

#[test]
fn typed_wrappers_receivers_and_ordinary_call_apis_preserve_live_frames() {
    let runtime = run(r#"
lower = 0
upper = 8
input = 2
extra = 99
def ease amount: ArbitraryType => Math.smootherstep(lower, upper, amount, extra)
eased = ease(input)
assert(eased, 0.103515625)
input = 6
assert(eased, 0.896484375)
upper = 16
assert(eased, 0.27520751953125)
try:
    def ease amount: ArbitraryType => Math.smootherstep(missing, upper, amount)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(eased, 0.27520751953125)
input = 8
assert(eased, 0.5)
class Curve:
    def init sample => this.sample = sample
    def reading lower, upper => Math.smootherstep(lower, upper, self.sample)
curve = Curve(2)
assert(curve.reading(0, 8), 0.103515625)
curve.sample = 6
assert(curve.reading(0, 8), 0.896484375)
"#);
    for name in ["ease", "input", "lower", "upper", "extra"] {
        assert!(dependencies(&runtime, "eased").contains(&name), "{name}");
    }
    assert!(!runtime.graph.contains(&NodeKey::variable("ArbitraryType")));
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));

    let statements = parse("Math.smootherstep(lower, upper, value)").unwrap();
    let [Stmt::Expression(expression)] = statements.as_slice() else {
        panic!("the intrinsic must remain an ordinary call");
    };
    let Ast::Call(call) = Ast::convert(expression) else {
        panic!("no new AST representation");
    };
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.push();
    scope.declare("lower", Value::number(0));
    scope.declare("upper", Value::number(8));
    scope.declare("value", Value::number(2));
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    assert_eq!(
        call.resolve(&mut runtime, &mut scope).unwrap(),
        Value::number(0.103515625)
    );
    let failed = parse_expression("Math.smootherstep(lower, upper, value, missing)").unwrap();
    let error = Ast::convert(&failed)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.retrieve("value"), Some(&Value::number(2)));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
}

#[test]
fn native_cli_preserves_exact_file_bytes_json_errors_and_literal_eof() {
    for (index, (source, success, stdout, stderr)) in [
        (
            "assert(Math.smootherstep(0, 8, 2), 0.103515625)\nMath.smootherstep(0, 8, 2)",
            true,
            "0.103515625\n",
            "",
        ),
        ("Math.smootherstep(8, 0, 2)", true, "0.896484375\n", ""),
        ("Math.smootherstep(2, 2, 2)", true, "null\n", ""),
        ("String(Math.smootherstep())", true, "\"NaN\"\n", ""),
        (
            "Math.smootherstep(0, 8, 2, Number.NaN, \"invalid\", {})",
            true,
            "0.103515625\n",
            "",
        ),
        (
            "Math.smootherstep(0, 8, 2, missing)",
            false,
            "",
            "ReferenceError: missing is not defined (line 1, column 1)\n",
        ),
        (
            "class Probe\nprobe = Probe()\nMath.smootherstep(probe.absent, 8, 2, Math.random())",
            false,
            "",
            "TypeError: Math.random is not supported (line 3, column 1)\n",
        ),
        (
            "Math.smootherstep(",
            false,
            "",
            "SyntaxError: Unexpected end of input (line 1, column 19)\n",
        ),
        (
            "Math.smootherstep(\n",
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
                "nuc35-math-smootherstep-{}-{index}.nuc",
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
fn stdin_literal_eof_and_repl_error_recovery_keep_live_graphs() {
    let source = "value = 0.25\neased = Math.smootherstep(0, 1, value)\nMath.smootherstep(0, 1, 0.5, missing)\nvalue = 0.75\neased";
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
            assert!(stdout.contains("eased [variable]"), "{stdout}");
            assert!(stdout.contains("value [variable]"), "{stdout}");
            assert!(stdout.contains("0.896484375"), "{stdout}");
        } else {
            let banner = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";
            assert_eq!(
                stdout,
                format!("{banner}0.25\n0.103515625\n0.75\n0.896484375\n")
            );
        }
    }
}
