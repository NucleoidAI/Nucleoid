mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{BinaryOp, Expr, parse, parse_expression};
use nucleoid::{ErrorKind, Runtime, Value};
use serde_json::json;

fn run(runtime: &mut Runtime, source: &str) -> Value {
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "exponentiation regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
}

fn read(runtime: &mut Runtime, source: &str) -> serde_json::Value {
    let value = run(runtime, source);
    serde_json::from_str(&runtime.serialize_json(&value).unwrap()).unwrap()
}

fn failure(source: &str) -> (ErrorKind, String, Option<(usize, usize)>) {
    let error = Runtime::new().run(source).unwrap_err();
    (
        error.kind(),
        error.message(),
        error
            .position()
            .map(|position| (position.line, position.column)),
    )
}

fn dependencies(runtime: &Runtime, key: &NodeKey) -> Vec<String> {
    runtime
        .graph
        .retrieve(key)
        .unwrap_or_else(|| panic!("{key:?} is filed"))
        .dependencies
        .iter()
        .map(|key| key.as_str().to_string())
        .collect()
}

fn mentions_power(source: &str) -> bool {
    tokenize(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
        .into_iter()
        .any(|spanned| matches!(spanned.token, Token::StarStar | Token::StarStarAssign))
}

fn same_number(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(actual), Value::Number(expected)) => {
            actual.to_bits() == expected.to_bits() || (actual.is_nan() && expected.is_nan())
        }
        (actual, expected) => actual == expected,
    }
}

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid raises numbers to powers with the exponentiation operator",
    "Nucleoid binds prefix operators around the exponentiation operator",
    "Nucleoid gives the exponentiation operator the results of Math.pow",
    "Nucleoid keeps exponentiation relationships live in class level rules",
    "Nucleoid keeps compound exponentiation assignments equivalent",
    "Nucleoid uses the exponentiation operator in functions callbacks and literals",
    "Nucleoid keeps double stars in strings and regular expressions beside the exponentiation operator",
    "Nucleoid rolls back failed exponentiation updates",
];

const SEALED_POW_TITLES: [&str; 3] = [
    "Nucleoid runs a nested block statement of variable",
    "Nucleoid runs a nested block statement of property",
    "Nucleoid creates multiple else if statement of class after initialization",
];

#[test]
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let readme = common::snippets(include_str!("../README.md"), "README.md");
    let examples = common::snippets(include_str!("../docs/examples.md"), "docs/examples.md");
    let appended = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (
            &reference,
            "Nucleoid raises values to live powers with exponentiation",
        ),
        (&readme, "Exponentiation keeps compound growth live"),
        (&examples, "Powers keep a kinetic energy model live"),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(mentions_power(&case.source), "{title} raises a power");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 87);

    for title in SEALED_POW_TITLES {
        let case = spec
            .iter()
            .find(|case| case.title == title)
            .unwrap_or_else(|| panic!("{title} is kept"));
        assert!(!mentions_power(&case.source), "{title}");
        assert!(case.source.contains("Math.pow"), "{title}");
    }
    let built_ins = reference
        .iter()
        .find(|case| case.title == "Nucleoid provides the built-in members the reference lists")
        .unwrap();
    assert!(built_ins.source.contains("assert(Math.pow(2, 10), 1024)"));
}

#[test]
fn exponentiation_returns_the_bits_of_math_pow_through_the_runtime() {
    let operands = [
        "null",
        "true",
        "false",
        "\"3\"",
        "\" 3 \"",
        "\"\"",
        "\"a\"",
        "[2]",
        "[]",
        "[1, 2]",
        "0",
        "-0",
        "1",
        "-1",
        "2",
        "-2",
        "0.5",
        "-0.5",
        "3",
        "1 / 3",
        "-8",
        "Number.NaN",
        "Number.POSITIVE_INFINITY",
        "Number.NEGATIVE_INFINITY",
        "1e308",
        "5e-324",
        "Date(2)",
    ];
    let mut runtime = Runtime::new();
    for base in operands {
        for exponent in operands {
            let source = format!("[({base}) ** ({exponent}), Math.pow({base}, {exponent})]");
            let Value::List(results) = run(&mut runtime, &source) else {
                panic!("{source} lists two results");
            };
            assert!(same_number(&results[0], &results[1]), "{source}");
        }
    }
    assert_eq!(
        read(
            &mut runtime,
            "[2 ** 10, 2 ** 0.5, (-8) ** (1 / 3), 0 ** -1, (-0) ** -1, 1 / (-0) ** 3]"
        ),
        json!([1024.0, std::f64::consts::SQRT_2, null, null, null, null])
    );
    assert_eq!(
        run(&mut runtime, "1 / (-0) ** 3"),
        Value::Number(f64::NEG_INFINITY)
    );
}

#[test]
fn stored_powers_settle_freeze_and_depend_like_math_pow() {
    let setup = "x = 2\nn = 3\nclass Box\nbox = Box()\nside = 4\nstep = 1.5\n";
    let mut power = Runtime::new();
    run(
        &mut power,
        &format!(
            "{setup}y = x ** n\nmissing = box.side ** 2\nfrozen = side ** step.value\nliteral = null ** n"
        ),
    );
    let mut call = Runtime::new();
    run(
        &mut call,
        &format!(
            "{setup}y = Math.pow(x, n)\nmissing = Math.pow(box.side, 2)\nfrozen = Math.pow(side, step.value)\nliteral = Math.pow(null, n)"
        ),
    );
    let probe = "[y, missing, frozen, literal]";
    let keys = ["y", "missing", "frozen", "literal"].map(NodeKey::variable);
    for step in [
        None,
        Some("x = 3"),
        Some("n = 2"),
        Some("box.side = 5"),
        Some("step = 3"),
        Some("side = 9"),
        Some("x = null"),
        Some("n = -1"),
        Some("x = 4"),
    ] {
        if let Some(step) = step {
            run(&mut power, step);
            run(&mut call, step);
        }
        assert_eq!(read(&mut power, probe), read(&mut call, probe), "{step:?}");
        for key in &keys {
            assert_eq!(
                dependencies(&power, key),
                dependencies(&call, key),
                "{key:?} after {step:?}"
            );
        }
    }
    assert_eq!(read(&mut power, probe), json!([0.25, 25.0, 27.0, null]));
    assert_eq!(dependencies(&power, &NodeKey::variable("frozen")), ["side"]);
    assert_eq!(
        read(&mut power, "n = null\nzero = n.value ** 2\nzero"),
        json!(0.0)
    );
}

#[test]
fn reasoning_reports_power_declarations_like_math_pow_with_their_own_rule() {
    let mut power = Runtime::new();
    run(&mut power, "x = 2\nn = 3\ny = x ** n");
    let mut call = Runtime::new();
    run(&mut call, "x = 2\nn = 3\ny = Math.pow(x, n)");
    for probe in [
        "(why y).map(s => [s.node, s.holds, s.state, s.from])",
        "(affects x).map(s => s.node)",
        "(affects n).map(s => s.node)",
        "(model |> derived).map(s => s.node)",
    ] {
        assert_eq!(read(&mut power, probe), read(&mut call, probe), "{probe}");
    }
    assert_eq!(read(&mut power, "(why y)[0].rule"), json!("y = x**n"));
    assert_eq!(
        read(&mut call, "(why y)[0].rule"),
        json!("y = Math.pow(x,n)")
    );
    assert_eq!(
        read(
            &mut power,
            "z = (-x) ** n\nw = -x ** n\n[(why z)[0].rule, (why w)[0].rule, z, w]"
        ),
        json!(["z = (-x)**n", "w = -x**n", -8.0, -8.0])
    );
}

#[test]
fn class_level_power_rules_validate_names_and_cycles_like_other_operators() {
    for operator in ["**", "*"] {
        assert_eq!(
            failure(&format!("class C\n$C.cube = $C.side {operator} missing")),
            (
                ErrorKind::Reference,
                "missing is not defined".to_string(),
                Some((2, 1))
            ),
            "{operator}"
        );
        assert_eq!(
            failure(&format!(
                "class C\n$C.cube = $C.side {operator} 3\n$C.side = $C.cube {operator} 2"
            )),
            (
                ErrorKind::Type,
                "Circular Dependency".to_string(),
                Some((3, 1))
            ),
            "{operator}"
        );
    }
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Cube(side):\n    this.side = side\n$Cube.volume = $Cube.side ** 3\nsmall = Cube(2)\nassert(small.volume, 8)\nsmall.side = 3\nassert(small.volume, 27)\nlarge = Cube(4)\nassert(large.volume, 64)",
    );
}

#[test]
fn compound_power_assignments_match_the_expanded_assignment() {
    for (compact, expanded) in [
        (
            "value = 3\nstep = 2\nvalue **= step\nstep = 3\nvalue",
            "value = 3\nstep = 2\nvalue = value ** (step)\nstep = 3\nvalue",
        ),
        (
            "class M\nm = M()\nm.r = 2\nm.p = 3\nm.r **= m.p\nm.p = 2\nm.r",
            "class M\nm = M()\nm.r = 2\nm.p = 3\nm.r = m.r ** (m.p)\nm.p = 2\nm.r",
        ),
        (
            "def raise(current, power) => current **= power\nraise(2, 10)",
            "def raise(current, power) => current = current ** power\nraise(2, 10)",
        ),
        (
            "a = 2\nb = 3\na **= b **= 2\n[a, b]",
            "a = 2\nb = 3\na = a ** (b = b ** 2)\n[a, b]",
        ),
        (
            "rate = 0.5\nvalue = 16\nvalue **= rate.value\nrate = 2\nvalue",
            "rate = 0.5\nvalue = 16\nvalue = value ** (rate.value)\nrate = 2\nvalue",
        ),
        (
            "class Meter(reading):\n    this.reading = reading\n    def scale(power):\n        power **= 2\n        this.reading **= power\n        return this.reading\nmeter = Meter(2)\nmeter.scale(-1)",
            "class Meter(reading):\n    this.reading = reading\n    def scale(power):\n        power = power ** 2\n        this.reading = this.reading ** power\n        return this.reading\nmeter = Meter(2)\nmeter.scale(-1)",
        ),
    ] {
        let mut left = Runtime::new();
        let mut right = Runtime::new();
        assert_eq!(
            read(&mut left, compact),
            read(&mut right, expanded),
            "{compact}"
        );
        assert_eq!(
            generate_all(&parse(compact).unwrap()),
            generate_all(&parse(expanded).unwrap()),
            "{compact}"
        );
    }
    let mut runtime = Runtime::new();
    assert_eq!(
        read(
            &mut runtime,
            "value = 3\nstep = 2\nvalue **= step\nstep = 3\nvalue"
        ),
        json!(729.0)
    );
}

#[test]
fn failed_power_updates_restore_state_and_graph() {
    let initial = r#"
def checked(value):
    if value > 1000:
        throw "TOO_LARGE"
    return value
base = 2
power = 3
result = checked(base ** power)
balance = 3
if balance > 100:
    throw "LIMIT"
assert(result, 8)
"#;
    let mut actual = Runtime::new();
    checked(&mut actual, initial);
    let mut control = Runtime::new();
    checked(&mut control, initial);
    for (source, thrown) in [
        ("power = 11", "TOO_LARGE"),
        ("base = 32", "TOO_LARGE"),
        ("balance **= power + 2", "LIMIT"),
        ("balance **= 2\nthrow \"ROLLBACK\"", "ROLLBACK"),
    ] {
        let error = actual.run(source).unwrap_err();
        assert_eq!(
            error.thrown_value(),
            Some(&Value::String(thrown.to_string())),
            "{source}"
        );
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    }
    checked(
        &mut actual,
        "power = 5\nassert(result, 32)\nbalance **= 2\nassert(balance, 9)\npower = 2\nassert(balance, 9)",
    );
}

#[test]
fn malformed_and_failing_powers_report_exact_errors() {
    for (source, kind, message, position) in [
        ("x = **2", ErrorKind::Syntax, "Unexpected '**'", (1, 7)),
        (
            "x = 2 **",
            ErrorKind::Syntax,
            "Unexpected end of input",
            (1, 9),
        ),
        (
            "x = 2 **\n",
            ErrorKind::Syntax,
            "Unexpected end of line",
            (2, 1),
        ),
        (
            "x **=",
            ErrorKind::Syntax,
            "Unexpected end of input",
            (1, 6),
        ),
        (
            "x = 1\nx ** = 2",
            ErrorKind::Syntax,
            "Unexpected '='",
            (2, 8),
        ),
        (
            "x = 1\nx **= **= 2",
            ErrorKind::Syntax,
            "Unexpected '**='",
            (2, 11),
        ),
        (
            "x = 1\nx **= (1 +)",
            ErrorKind::Syntax,
            "Unexpected ')'",
            (2, 12),
        ),
        ("x = 2 *** 3", ErrorKind::Syntax, "Unexpected '*'", (1, 11)),
        ("x = 2 ** * 3", ErrorKind::Syntax, "Unexpected '*'", (1, 12)),
        (
            "2 **= 3",
            ErrorKind::Syntax,
            "Invalid compound assignment target",
            (1, 3),
        ),
        (
            "x = 1\n(x + 1) **= 2",
            ErrorKind::Syntax,
            "Invalid compound assignment target",
            (2, 9),
        ),
        ("x **= 3", ErrorKind::Reference, "x is not defined", (1, 1)),
        (
            "class P\np = P()\np.v = 2\ndelete p.v ** 2",
            ErrorKind::Syntax,
            "Cannot delete p.v**2",
            (4, 1),
        ),
        (
            "class P\np = P()\np.v = 2\ny = delete p.v ** 2",
            ErrorKind::Syntax,
            "Cannot delete p.v**2",
            (4, 1),
        ),
        (
            "x = 2\ny = why x ** 2",
            ErrorKind::Type,
            "Cannot explain x**2",
            (2, 1),
        ),
        (
            "x = 2\ny = (x ** 2) = 1",
            ErrorKind::Syntax,
            "Cannot assign to x**2",
            (2, 1),
        ),
        (
            "class Scale\nscale = Scale()\n$Scale.factor = 2\n$Scale.factor **= 3",
            ErrorKind::Type,
            "Circular Dependency",
            (4, 1),
        ),
    ] {
        assert_eq!(
            failure(source),
            (kind, message.to_string(), Some(position)),
            "{source}"
        );
    }
    let deep = format!("x = 1{}", " ** 1".repeat(62));
    assert_eq!(
        failure(&deep),
        (
            ErrorKind::Syntax,
            "Expressions are nested too deeply".to_string(),
            Some((1, 315))
        )
    );
}

#[test]
fn prefix_operators_apply_to_the_whole_power_and_new_stays_primary() {
    let mut runtime = Runtime::new();
    assert_eq!(
        read(
            &mut runtime,
            "x = 3\n[-x ** 2, +x ** 2, !x ** 0, not x ** 0, typeof x ** 2, (-x) ** 2, x ** -1, x ** +2, 2 ** -3 ** 2]"
        ),
        json!([
            -9.0,
            9.0,
            false,
            false,
            "number",
            9.0,
            0.3333333333333333,
            9.0,
            0.001953125
        ])
    );
    assert_eq!(
        parse_expression("new Foo(3) ** 2").unwrap(),
        parse_expression("Foo(3) ** 2").unwrap()
    );
    assert_eq!(
        read(
            &mut runtime,
            "class Foo(n):\n    this.n = n\nNumber.isNaN(new Foo(3) ** 2)"
        ),
        json!(true)
    );
}

#[test]
fn power_ignores_a_variable_named_math() {
    let mut runtime = Runtime::new();
    assert_eq!(
        run(&mut runtime, "Math = 1\ny = 2 ** 3\ny"),
        Value::Number(8.0)
    );
    let error = runtime.run("z = Math.pow(2, 3)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.pow is not a function");
}

#[test]
fn operands_evaluate_left_to_right_like_math_pow_arguments() {
    let setup = "def fail(label):\n    throw label\n";
    for expression in [
        "fail(\"left\") ** fail(\"right\")",
        "Math.pow(fail(\"left\"), fail(\"right\"))",
    ] {
        let error = Runtime::new()
            .run(&format!("{setup}{expression}"))
            .unwrap_err();
        assert_eq!(
            error.thrown_value(),
            Some(&Value::String("left".to_string())),
            "{expression}"
        );
    }
}

#[test]
fn template_interpolations_keep_ignoring_trailing_tokens() {
    let mut runtime = Runtime::new();
    assert_eq!(
        read(&mut runtime, "x = 3\nt = `${x y ** 2}`\nt"),
        json!("3")
    );
    assert_eq!(read(&mut runtime, "u = `${x ** 2}`\nu"), json!("9"));
    assert_eq!(
        parse_expression("x y ** 2").unwrap(),
        Expr::Identifier("x".to_string())
    );
    assert_eq!(
        parse_expression("2 ** 3").unwrap(),
        Expr::Binary {
            operator: BinaryOp::Power,
            left: Box::new(Expr::Number(2.0)),
            right: Box::new(Expr::Number(3.0)),
        }
    );
}

#[test]
fn single_stars_and_double_star_text_keep_their_accepted_meaning() {
    let mut runtime = Runtime::new();
    assert_eq!(
        read(
            &mut runtime,
            "x = 2\nx *= 3\npattern = /x**y/\n[x, 2 * 3, pattern, pattern.test(\"xxy\"), \"2**3\".split(\"**\")]"
        ),
        json!([6.0, 6.0, "x**y", true, ["2", "3"]])
    );
    assert_eq!(
        failure("a * * b"),
        (
            ErrorKind::Syntax,
            "Unexpected '*'".to_string(),
            Some((1, 7))
        )
    );
}

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

fn repl(source: &str) -> Output {
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
    child.wait_with_output().unwrap()
}

#[test]
fn the_native_repl_runs_power_headers_and_compound_powers() {
    let output =
        repl("side = 3\nif side ** 2 > 8\n    big = side ** 2\n\nbig\nside **= 2\nside\nbig\n");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stderr, b"");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{BANNER}3.0\n9.0\n9.0\n9.0\n9.0\n81.0\n")
    );
}
