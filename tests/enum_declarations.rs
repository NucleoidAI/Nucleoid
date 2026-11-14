mod common;

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{EnumMember, Expr, ParameterDefault, Stmt, parse};
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde_json::{Value as Json, json};

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid declares enums whose members name themselves",
    "Nucleoid gives enum members explicit scalar values",
    "Nucleoid declares enums in indented blocks",
    "Nucleoid keeps class level rules over enum members live",
    "Nucleoid updates readers when an enum is declared again",
    "Nucleoid declares enums wherever a statement may start",
    "Nucleoid rolls back enum declarations with their transactions",
    "Nucleoid keeps enum an ordinary name outside enum declarations",
];
const REFERENCE_TITLE: &str = "Nucleoid declares enums of named members";
const README_TITLE: &str = "Enums name a model's categories once";
const EXAMPLES_TITLE: &str = "Enums keep a ticket's priority rules readable";

const READ_ONLY: &str = "Enums are read-only";
const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

fn member(name: &str, value: ParameterDefault) -> EnumMember {
    EnumMember {
        name: name.to_string(),
        value,
    }
}

fn named(name: &str) -> EnumMember {
    member(name, ParameterDefault::String(name.to_string()))
}

fn declaration(name: &str, members: Vec<EnumMember>) -> Stmt {
    Stmt::Assign {
        target: Expr::Identifier(name.to_string()),
        value: Expr::Enum(members),
    }
}

fn declares_enum(source: &str) -> bool {
    let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    format!("{statements:?}").contains("value: Enum([EnumMember")
}

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "enum declaration regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
    value
}

fn json_of(runtime: &Runtime, value: &Value) -> Json {
    serde_json::from_str(&runtime.serialize_json(value).unwrap()).unwrap()
}

fn evaluate(runtime: &mut Runtime, source: &str) -> Json {
    let value = checked(runtime, source);
    json_of(runtime, &value)
}

fn result(source: &str) -> Json {
    evaluate(&mut Runtime::new(), source)
}

fn failure_in(runtime: &mut Runtime, source: &str) -> (ErrorKind, String, Position) {
    let error = runtime.run(source).expect_err("the program must fail");
    (
        error.kind(),
        error.message(),
        error.position().expect("errors are positioned"),
    )
}

fn failure(source: &str) -> (ErrorKind, String, Position) {
    failure_in(&mut Runtime::new(), source)
}

fn error(
    kind: ErrorKind,
    message: &str,
    line: usize,
    column: usize,
) -> (ErrorKind, String, Position) {
    (kind, message.to_string(), Position::new(line, column))
}

fn syntax_error(source: &str, message: &str, line: usize, column: usize) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    assert_eq!(error.message(), message, "{source}");
    assert_eq!(
        error.position(),
        Some(Position::new(line, column)),
        "{source}"
    );
}

fn normalized(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find("obj:") {
        output.push_str(&rest[..index + 4]);
        rest = rest[index + 4..].trim_start_matches(|character: char| character.is_ascii_digit());
        output.push('N');
    }
    output.push_str(rest);
    output
}

fn graph_listing(runtime: &Runtime) -> Vec<String> {
    let mut listing: Vec<String> = runtime
        .graph
        .nodes()
        .map(|node| {
            let mut reads: Vec<String> =
                node.dependencies.iter().map(ToString::to_string).collect();
            let mut updates: Vec<String> =
                node.dependents.iter().map(ToString::to_string).collect();
            reads.sort();
            updates.sort();
            normalized(&format!(
                "{} [{}] reads {reads:?} updates {updates:?}",
                node.key, node.kind
            ))
        })
        .collect();
    listing.sort();
    listing
}

fn binary(arguments: &[&str], input: Option<&str>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.unwrap_or_default().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn documents() -> [Vec<common::Case>; 4] {
    [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ]
}

fn is_appended(title: &str) -> bool {
    SPEC_TITLES.contains(&title) || [REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE].contains(&title)
}

#[test]
fn enum_declarations_parse_to_assignments_of_enum_values() {
    let status = vec![declaration("Status", vec![named("PENDING"), named("PAID")])];
    for source in [
        "enum Status: PENDING, PAID",
        "enum Status:PENDING,PAID",
        "enum Status: PENDING = \"PENDING\", PAID",
        "enum Status:\n    PENDING\n    PAID",
        "enum Status:\n\n    PENDING\n\n    PAID\n",
        "enum Status:\n    PENDING\n    # a comment\n    PAID",
    ] {
        assert_eq!(parse(source).unwrap(), status, "{source}");
    }
    assert_eq!(
        parse(
            "enum Level: LOW = 1, ZERO = -0, UP = +2, DOWN = -2.5, NAME = \"n\", ON = true, OFF = false, NONE = null, NIL = None, VOID = undefined"
        )
        .unwrap(),
        vec![declaration(
            "Level",
            vec![
                member("LOW", ParameterDefault::Number(1.0)),
                member("ZERO", ParameterDefault::Number(-0.0)),
                member("UP", ParameterDefault::Number(2.0)),
                member("DOWN", ParameterDefault::Number(-2.5)),
                member("NAME", ParameterDefault::String("n".to_string())),
                member("ON", ParameterDefault::Bool(true)),
                member("OFF", ParameterDefault::Bool(false)),
                member("NONE", ParameterDefault::Null),
                member("NIL", ParameterDefault::Null),
                member("VOID", ParameterDefault::Null),
            ],
        )]
    );
    assert_ne!(
        parse("enum Z: A = -0, B").unwrap(),
        parse("enum Z: A = 0, B").unwrap()
    );
    assert_eq!(
        parse("enum Unit:\n    ONLY = 7").unwrap(),
        vec![declaration(
            "Unit",
            vec![member("ONLY", ParameterDefault::Number(7.0))]
        )]
    );
    for name in ["enum", "while", "is", "model", "any", "first", "True"] {
        assert_eq!(
            parse(&format!("enum {name}: A, B")).unwrap(),
            vec![declaration(name, vec![named("A"), named("B")])],
            "{name}"
        );
    }
    assert_eq!(
        parse("enum Mixed: assert, lambda, suppose, enum").unwrap(),
        vec![declaration(
            "Mixed",
            vec![
                named("assert"),
                named("lambda"),
                named("suppose"),
                named("enum")
            ]
        )]
    );
}

#[test]
fn members_hold_their_names_or_literal_values_in_declaration_order() {
    assert_eq!(
        result("enum Status: PENDING, PAID\nStatus"),
        json!({"PENDING": "PENDING", "PAID": "PAID"})
    );
    assert_eq!(
        result(
            "enum Level: LOW = 1, NAME = \"n\", ON = true, NONE = null, DOWN = -2.5\n[Level.LOW, Level.NAME, Level.ON, Level.NONE, Level.DOWN, Level[\"NAME\"], Level.MISSING]"
        ),
        json!([1.0, "n", true, null, -2.5, "n", null])
    );
    assert_eq!(
        result(
            "enum Status: PENDING, PAID\nkeys = []\nfor key, value of Status:\n    keys.push(key + \"=\" + value)\nkeys"
        ),
        json!(["PENDING=PENDING", "PAID=PAID"])
    );
    assert_eq!(
        result(
            "enum Order: Z, A = 1, M\nkeys = []\nfor key, value of Order:\n    keys.push(key)\nkeys"
        ),
        json!(["Z", "A", "M"])
    );
    assert_eq!(
        result(
            "enum Status: PENDING, PAID\n[typeof Status, Status == Status, Status == {PENDING: \"PENDING\", PAID: \"PAID\"}]"
        ),
        json!(["Object", true, false])
    );
    assert_eq!(
        result(
            "enum Status: PENDING, PAID\nassert(Status, {PENDING: \"PENDING\", PAID: \"PAID\"})\nStatus.PAID"
        ),
        json!("PAID")
    );
    let mut runtime = Runtime::new();
    let value = runtime.run("enum Status: PENDING, PAID\nStatus").unwrap();
    assert!(value.to_string().starts_with("[object obj:"), "{value}");
}

#[test]
fn enum_declarations_start_wherever_a_statement_may_start() {
    for (source, expected) in [
        ("x = 1; enum S: A, B\n[x, S.B]", json!([1.0, "B"])),
        ("x = 1 enum S: A, B\n[x, S.B]", json!([1.0, "B"])),
        ("if true: enum S: A, B\nS.A", json!("A")),
        ("if false: x = 1\nelse: enum S: A, B\nS.B", json!("B")),
        ("if true:\n    enum S: A, B\nS.B", json!("B")),
        (
            "ready = true\nif ready:\n    enum S:\n        A\n        B\nS.B",
            json!("B"),
        ),
        (
            "go = true\nwhile go: enum S: A, B; go = false\nS.A",
            json!("A"),
        ),
        (
            "class P\np = P()\nfor item of P: enum S: A, B\nS.B",
            json!("B"),
        ),
        ("try: enum S: A, B\ncatch error: pass\nS.A", json!("A")),
        (
            "try:\n    throw 1\ncatch error:\n    enum S: A, B\nS.B",
            json!("B"),
        ),
        ("seen = suppose: enum S: A, B; S.B\nseen", json!("B")),
        ("{ enum S: A, B }\nS.A", json!("A")),
        ("{\n    enum S: A, B\n}\nS.B", json!("B")),
        (
            "def f():\n    enum S: A, B\n    return S.B\nf()",
            json!("B"),
        ),
        ("def f(): enum S: A, B\nf()\nS.A", json!("A")),
        (
            "f = function () { enum S: A, B; return S.A }\nf()",
            json!("A"),
        ),
        ("f = x => { enum S: A, B; return S.B }\nf(1)", json!("B")),
        ("function g() { enum S: A, B; return S.A }\ng()", json!("A")),
        (
            "x = `${(() => { enum S: P, Q; return S.Q })()}`\nx",
            json!("Q"),
        ),
        (
            "a, b = (() => { enum S: P, Q; return [S.P, S.Q] })()\n[a, b]",
            json!(["P", "Q"]),
        ),
        (
            "v = 1 if (() => { enum S: P, Q; return S.P == \"P\" })() else 2\nv",
            json!(1.0),
        ),
        (
            "while (() => { enum S: P, Q; return false })(): pass\nS.Q",
            json!("Q"),
        ),
        (
            "r = suppose x = (() => { enum S: P, Q; return 1 })(): x\nr",
            json!(1.0),
        ),
        (
            "x = 1\nok = (x is 1)\nenum S: A, B\n[ok, S.A]",
            json!([true, "A"]),
        ),
        ("class A:\n    pass\nenum S: X, Y\nS.Y", json!("Y")),
        (
            "class A:\n    class B:\n        pass\nenum S: X, Y\nS.X",
            json!("X"),
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    assert_eq!(
        failure("suppose enum S: A, B"),
        error(ErrorKind::Reference, "suppose is not defined", 1, 1)
    );
}

#[test]
fn class_bodies_keep_every_enum_spelling() {
    for (source, message, line, column) in [
        (
            "class A:\n    enum Status: PENDING, PAID",
            "Unexpected ':'",
            2,
            18,
        ),
        (
            "class A:\n    def m():\n        enum S: X, Y",
            "Unexpected ':'",
            3,
            17,
        ),
        (
            "class A:\n    if true:\n        enum S: X, Y",
            "Unexpected ':'",
            3,
            17,
        ),
        (
            "class A:\n    enum S:\n        X\n        Y",
            "Unexpected ':'",
            2,
            11,
        ),
        ("class A(b):\nenum S: X, Y", "Unexpected ':'", 2, 9),
        ("class A: enum S: X, Y", "Unexpected ':'", 1, 18),
        (
            "class A extends B:\n    enum S: X, Y",
            "Unexpected ':'",
            2,
            13,
        ),
        (
            "class A:\n    x = `${(() => { enum S: P, Q; return S.P })()}`",
            "Unexpected ':'",
            2,
            29,
        ),
    ] {
        syntax_error(source, message, line, column);
    }
    assert_eq!(
        parse("class A:\n    enum Status: PENDING").unwrap(),
        parse("class A:\n    enum\n    Status: PENDING").unwrap()
    );
    assert_eq!(
        failure("class A:\n    enum Status: PENDING\nA()"),
        error(ErrorKind::Reference, "enum is not defined", 3, 1)
    );
    assert_eq!(
        result("enum = 2\nclass A:\n    enum Status: PENDING\nA.length"),
        json!(0.0)
    );
}

#[test]
fn single_names_and_colon_statements_keep_their_accepted_readings() {
    let typed = |name: &str, type_name: &str| Stmt::Declaration {
        name: name.to_string(),
        type_name: type_name.to_string(),
    };
    let word = || Stmt::Expression(Expr::Identifier("enum".to_string()));
    assert_eq!(
        parse("enum Status: PENDING").unwrap(),
        vec![word(), typed("Status", "PENDING")]
    );
    assert_eq!(
        parse("enum Status: str").unwrap(),
        vec![word(), typed("Status", "str")]
    );
    assert_eq!(
        parse("enum Status: PENDING; PAID").unwrap(),
        vec![
            word(),
            typed("Status", "PENDING"),
            Stmt::Expression(Expr::Identifier("PAID".to_string()))
        ]
    );
    assert_eq!(
        parse("if true: enum S: A").unwrap(),
        parse("if true: enum\nS: A").unwrap()
    );
    for source in [
        "enum Status: PENDING",
        "enum Status: str",
        "enum Status: PENDING; PAID",
        "enum assert: 1, 1",
        "enum assert: x",
        "enum lambda: A = 1",
        "enum lambda: x + 1",
        "enum lambda: A",
        "enum suppose: a, b = 1",
        "enum suppose:\n    A\n    B",
        "enum suppose: x",
        "if true: enum S: A",
        "{ enum S: A; B }",
    ] {
        assert!(!declares_enum(source), "{source}");
        assert_eq!(
            failure(source),
            error(ErrorKind::Reference, "enum is not defined", 1, 1),
            "{source}"
        );
    }
    assert_eq!(
        failure("x = 1 enum Status: PENDING"),
        error(ErrorKind::Reference, "enum is not defined", 1, 7)
    );
    assert_eq!(
        failure("enum = 3\nenum Status: PENDING"),
        error(ErrorKind::Reference, "Missing definition", 2, 6)
    );
}

#[test]
fn enum_stays_an_ordinary_name_everywhere_else() {
    assert_eq!(result("enum = 3\nenum"), json!(3.0));
    assert_eq!(result("record = {enum: 1}\nrecord.enum"), json!(1.0));
    assert_eq!(result("def f(enum) => enum * 2\nf(4)"), json!(8.0));
    assert_eq!(
        result("enum = 5\nenum Size: SMALL, LARGE\n[enum, Size.LARGE]"),
        json!([5.0, "LARGE"])
    );
    assert_eq!(
        failure("enum.x"),
        error(ErrorKind::Reference, "enum is not defined", 1, 1)
    );
    for (source, message, line, column) in [
        ("enum Status { PENDING, PAID }", "Unexpected ','", 1, 24),
        (
            "enum class Status { PENDING, PAID }",
            "Expected ':' but found '{'",
            1,
            19,
        ),
        ("x = enum Status: A, B", "Unexpected ':'", 1, 18),
        (
            "f = x => x\nf(enum Status: A, B)",
            "Expected ')' but found identifier 'Status'",
            2,
            8,
        ),
    ] {
        syntax_error(source, message, line, column);
    }
}

#[test]
fn malformed_declarations_report_exact_syntax_errors() {
    for (source, message, line, column) in [
        (
            "enum Level: LOW = 1",
            "Enum 'Level' needs at least two members on one line",
            1,
            20,
        ),
        (
            "enum Level: LOW = 1\nx = 2",
            "Enum 'Level' needs at least two members on one line",
            1,
            19,
        ),
        (
            "{ enum Status: PENDING }",
            "Enum 'Status' needs at least two members on one line",
            1,
            24,
        ),
        (
            "enum Status: PENDING, PENDING",
            "Enum 'Status' already has member 'PENDING'",
            1,
            23,
        ),
        (
            "enum Status:\n    A\n    A",
            "Enum 'Status' already has member 'A'",
            3,
            5,
        ),
        (
            "enum Status: A = 1, A = x",
            "Enum 'Status' already has member 'A'",
            1,
            21,
        ),
        (
            "enum Status: PENDING,",
            "Expected a name but found end of input",
            1,
            22,
        ),
        (
            "enum Status: A, B,",
            "Expected a name but found end of input",
            1,
            19,
        ),
        (
            "enum Status: 1, 2",
            "Expected a name but found number 1",
            1,
            14,
        ),
        (
            "enum Status:",
            "Expected a name but found end of input",
            1,
            13,
        ),
        ("enum Status: , A", "Expected a name but found ','", 1, 14),
        (
            "enum Status: if, A",
            "Expected a name but found keyword 'if'",
            1,
            14,
        ),
        (
            "enum Status: PENDING PAID",
            "Expected ',' but found identifier 'PAID'",
            1,
            22,
        ),
        (
            "enum Status: A, B x = 1",
            "Expected ',' but found identifier 'x'",
            1,
            19,
        ),
        (
            "enum Status: A = 1 + 2, B",
            "Expected ',' but found '+'",
            1,
            20,
        ),
        (
            "enum Level: LOW = x, HIGH",
            "Expected a scalar literal enum member value",
            1,
            19,
        ),
        (
            "enum Level: LOW = -\"x\", HIGH",
            "Expected a scalar literal enum member value",
            1,
            20,
        ),
        (
            "enum Level: LOW = -true, HIGH",
            "Expected a scalar literal enum member value",
            1,
            20,
        ),
        (
            "enum Level: LOW = `t`, HIGH",
            "Expected a scalar literal enum member value",
            1,
            19,
        ),
        (
            "enum Level: LOW =, HIGH",
            "Expected a scalar literal enum member value",
            1,
            18,
        ),
        (
            "enum Level: LOW = 1e999, HIGH",
            "Enum member value must be a finite number",
            1,
            19,
        ),
        (
            "enum Level: LOW = -1e999, HIGH",
            "Enum member value must be a finite number",
            1,
            20,
        ),
        (
            "enum Status:\nx = 1",
            "Expected an indented block but found identifier 'x'",
            2,
            1,
        ),
        (
            "enum Status:\n    A B",
            "Expected end of line but found identifier 'B'",
            2,
            7,
        ),
        (
            "enum Status:\n    A; B",
            "Expected end of line but found ';'",
            2,
            6,
        ),
        (
            "enum Status:\n    A,\n    B",
            "Expected end of line but found ','",
            2,
            6,
        ),
        (
            "enum Status:\n    A\n        B",
            "Expected a name but found an indented block",
            3,
            9,
        ),
        (
            "enum Status:\n    A = x\n    B",
            "Expected a scalar literal enum member value",
            2,
            9,
        ),
        (
            "enum S: P, Q else: x = 1",
            "Unexpected keyword 'else'",
            1,
            18,
        ),
    ] {
        syntax_error(source, message, line, column);
    }
}

#[test]
fn former_errors_now_declare_enums() {
    for (source, expected) in [
        (
            "enum Status: PENDING, PAID\nStatus",
            json!({"PENDING": "PENDING", "PAID": "PAID"}),
        ),
        (
            "enum Status:\n    PENDING\n    PAID\nStatus",
            json!({"PENDING": "PENDING", "PAID": "PAID"}),
        ),
        (
            "enum Level: LOW = 1, HIGH = 3\nLevel",
            json!({"LOW": 1.0, "HIGH": 3.0}),
        ),
        (
            "enum Color:\n    RED\n\n    GREEN\nx = 1\n[Color, x]",
            json!([{"RED": "RED", "GREEN": "GREEN"}, 1.0]),
        ),
        (
            "x = 1 enum Status: A, B\n[x, Status]",
            json!([1.0, {"A": "A", "B": "B"}]),
        ),
        (
            "if true: enum Status: A, B\nStatus",
            json!({"A": "A", "B": "B"}),
        ),
        ("{ enum Status: A, B }\nStatus", json!({"A": "A", "B": "B"})),
        (
            "def f():\n    enum Status: A, B\n    return Status.B\n[f(), Status]",
            json!(["B", {"A": "A", "B": "B"}]),
        ),
        ("enum Status: A, B; x = Status.B\nx", json!("B")),
        (
            "a, b = (() => { enum S: P, Q; return [S.P, S.Q] })()\n[a, b]",
            json!(["P", "Q"]),
        ),
        (
            "x = `${(() => { enum S: P, Q; return S.P })()}`\nx",
            json!("P"),
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

fn rejects(setup: &str, write: &str, line: usize, column: usize) {
    let mut runtime = Runtime::new();
    runtime
        .run(setup)
        .unwrap_or_else(|error| panic!("{setup}: {error}"));
    let before = (
        evaluate(&mut runtime, "[S, reader]"),
        graph_listing(&runtime),
    );
    assert_eq!(
        failure_in(&mut runtime, write),
        error(ErrorKind::Type, READ_ONLY, line, column),
        "{write}"
    );
    assert_eq!(
        (
            evaluate(&mut runtime, "[S, reader]"),
            graph_listing(&runtime)
        ),
        before,
        "{write}"
    );
}

#[test]
fn every_write_to_an_enum_raises_and_leaves_the_state_unchanged() {
    let setup = "enum S: A, B = 2\nreader = S.A";
    for write in [
        "S.A = \"X\"",
        "S.NEW = 1",
        "S.A += \"X\"",
        "S.B *= 2",
        "S.A ||= \"X\"",
        "S.A &&= \"X\"",
        "S.A := \"X\"",
        "S.NEW := 1",
        "(S.A = \"X\")",
        "x = (S.A = \"X\")",
        "delete S.A",
        "delete S.MISSING",
        "x = delete S.A",
        "S@first.A = 1",
        "S.value = 1",
        "if true: S.A = 1",
        "{ S.A = 1 }",
        "suppose: S.A = 1",
        "seen = suppose: S.A = 1; S",
    ] {
        rejects(setup, write, 1, 1);
    }
    for (extra, write) in [
        ("s = S", "s.B = 3"),
        ("frozen = S.value", "frozen.A = 1"),
        ("holder = {kind: S}", "holder.kind.A = 1"),
        ("f = v -> (S.A = v)", "f(1)"),
        ("def g(e):\n    e.A = 1\n    return e", "g(S)"),
        ("def h(e):\n    delete e.A", "h(S)"),
        (
            "class Box(kind):\n    this.kind = kind\nbox = Box(S)",
            "box.kind.A = 1",
        ),
        (
            "class Holder(kind):\n    this.kind = kind\n    def set():\n        this.kind.A = 1\nholder = Holder(S)",
            "holder.set()",
        ),
        ("class Writer:\n    S.A = 1", "w = Writer()"),
        ("flag = false\nif flag:\n    S.A = 1", "flag = true"),
        ("copy = suppose: S", "copy.A = 1"),
        ("pair = suppose: [S, 1]", "pair[0].A = 1"),
    ] {
        rejects(&format!("{setup}\n{extra}"), write, 1, 1);
    }
}

#[test]
fn list_write_backs_that_reach_an_enum_raise_and_leave_the_state_unchanged() {
    let setup = "enum S: A, B = 2\nreader = S.A\nplain = {items: []}\nflag = false\ni = 0";
    assert_eq!(
        result(&format!(
            "{setup}\nother = {{items: [9]}}\n(flag ? other : plain).items.push(flag = true)\n[plain, other]"
        )),
        json!([{"items": []}, {"items": [true]}])
    );
    for write in [
        "(flag ? S : plain).items.push(flag = true)",
        "(S if flag else plain).items.append(flag = true)",
        "x = (flag ? S : plain).items.push(flag = true)",
        "[plain, S][i].items.push(i = 1)",
    ] {
        rejects(setup, write, 1, 1);
    }
}

#[test]
fn writes_are_caught_as_type_errors_and_roll_back_with_their_transaction() {
    assert_eq!(
        result(
            "enum S: A, B\nreader = S.A\ntry:\n    S.A = \"X\"\ncatch error:\n    caught = error\n[caught, reader, S]"
        ),
        json!(["TypeError: Enums are read-only", "A", {"A": "A", "B": "B"}])
    );
    assert_eq!(
        result(
            "enum S: A, B\nflag = false\nif flag:\n    S.A = 1\ntry:\n    flag = true\ncatch error:\n    caught = error\n[caught, flag, S.A]"
        ),
        json!(["TypeError: Enums are read-only", false, "A"])
    );
    let mut runtime = Runtime::new();
    runtime.run("enum S: A, B\ncount = 0").unwrap();
    assert_eq!(
        failure_in(&mut runtime, "count = 1\nS.A = 2"),
        error(ErrorKind::Type, READ_ONLY, 2, 1)
    );
    assert_eq!(evaluate(&mut runtime, "[count, S.A]"), json!([0.0, "A"]));
    assert_eq!(
        failure_in(&mut runtime, "count = 1\nmissing + 1"),
        error(ErrorKind::Reference, "missing is not defined", 2, 1)
    );
    assert_eq!(evaluate(&mut runtime, "count"), json!(0.0));
    assert_eq!(
        failure_in(&mut runtime, "S.A.push(1)"),
        error(ErrorKind::Type, "A.push is not a function", 1, 1)
    );
    assert_eq!(evaluate(&mut runtime, "S"), json!({"A": "A", "B": "B"}));
}

#[test]
fn rebinding_the_name_is_not_a_write_to_the_enum() {
    assert_eq!(
        result("enum S: A, B\nreader = S.A\nS = {A: \"X\"}\nS.A = \"Y\"\n[reader, S.A]"),
        json!(["Y", "Y"])
    );
    assert_eq!(
        result("enum S: A, B\nreader = S.B\nenum S: A, B = \"Z\"\nreader"),
        json!("Z")
    );
    assert_eq!(result("enum S: A, B\nS = 5\nS"), json!(5.0));
    let (kind, message, position) = failure("enum S: A, B\ndelete S");
    assert_eq!((kind, position), (ErrorKind::Type, Position::new(2, 1)));
    assert!(
        message.starts_with("Cannot delete object 'obj:"),
        "{message}"
    );
    assert_eq!(
        result("enum S:\n    id\nreader = S.id\nremoved = delete S\n[removed, reader]"),
        result("s = {id: \"id\"}\nreader = s.id\nremoved = delete s\n[removed, reader]")
    );
    assert_eq!(
        result("enum S:\n    id\nreader = S.id\nremoved = delete S\n[removed, reader]"),
        json!([true, null])
    );
}

#[test]
fn spreads_copy_enums_into_ordinary_objects() {
    assert_eq!(
        result("enum S: A, B\ncopy = {...S, C: \"C\"}\ncopy.A = \"X\"\ndelete copy.B\n[copy, S]"),
        json!([{"A": "X", "C": "C"}, {"A": "A", "B": "B"}])
    );
    assert_eq!(
        result("enum S: A, B\nlive = {...S}\nenum S: A, B, C\nlive"),
        json!({"A": "A", "B": "B", "C": "C"})
    );
}

#[test]
fn suppose_blocks_keep_copies_read_only_and_roll_back_their_enums() {
    assert_eq!(
        result("enum S: A, B\ncopy = suppose: S\n[copy, copy == S]"),
        json!([{"A": "A", "B": "B"}, false])
    );
    assert_eq!(
        result(
            "made = suppose: enum T: X, Y; T\ntry:\n    made.X = 1\ncatch error:\n    caught = error\n[made, caught]"
        ),
        json!([{"X": "X", "Y": "Y"}, "TypeError: Enums are read-only"])
    );
    assert_eq!(
        result("plain = suppose: ({X: 1})\nplain.X = 2\nplain.X"),
        json!(2.0)
    );
    assert_eq!(
        failure("suppose: enum T: X, Y\nT"),
        error(ErrorKind::Reference, "T is not defined", 2, 1)
    );
    assert_eq!(
        result("enum S: A, B\nview = S.B\nseen = suppose: enum S: A, C; view\n[seen, view, S]"),
        json!([null, "B", {"A": "A", "B": "B"}])
    );
}

#[test]
fn failed_declarations_roll_back_and_leave_earlier_enums_read_only() {
    let mut runtime = Runtime::new();
    runtime.run("enum S: A, B\nview = S.B").unwrap();
    assert_eq!(
        failure_in(&mut runtime, "{ enum S: A, C; missing + 1 }"),
        error(ErrorKind::Reference, "missing is not defined", 1, 1)
    );
    assert_eq!(
        evaluate(&mut runtime, "[S, view]"),
        json!([{"A": "A", "B": "B"}, "B"])
    );
    assert_eq!(
        failure_in(&mut runtime, "S.A = 1"),
        error(ErrorKind::Type, READ_ONLY, 1, 1)
    );
    runtime
        .run("try:\n    enum S: Q, R\n    throw \"UNDO\"\ncatch error:\n    pass")
        .unwrap();
    assert_eq!(
        evaluate(&mut runtime, "[S, view]"),
        json!([{"A": "A", "B": "B"}, "B"])
    );
    runtime.run("plain = {A: 1}\nplain.A = 2").unwrap();
    assert_eq!(evaluate(&mut runtime, "plain.A"), json!(2.0));
}

#[test]
fn enum_declarations_explain_and_file_as_object_literal_declarations_do() {
    let pairs = [
        (
            "enum S: A, B\nt = S.B\nu = t + \"!\"",
            "S = {A: \"A\", B: \"B\"}\nt = S.B\nu = t + \"!\"",
        ),
        (
            "enum S: A, B\nclass C(k):\n    this.k = k\n$C.ok = $C.k == S.A\nc = C(S.A)\nenum S: A = \"Z\", B",
            "S = {A: \"A\", B: \"B\"}\nclass C(k):\n    this.k = k\n$C.ok = $C.k == S.A\nc = C(S.A)\nS = {A: \"Z\", B: \"B\"}",
        ),
        (
            "flag = true\nif flag:\n    enum S: A, B\nt = S.A",
            "flag = true\nif flag:\n    S = {A: \"A\", B: \"B\"}\nt = S.A",
        ),
    ];
    for (enumeration, literal) in pairs {
        let mut left = Runtime::new();
        let mut right = Runtime::new();
        left.run(enumeration).unwrap();
        right.run(literal).unwrap();
        assert_eq!(graph_listing(&left), graph_listing(&right), "{enumeration}");
        for query in [
            "(why t).map(s -> s.node)",
            "(why t).map(s -> s.state)",
            "(why t).map(s -> s.holds)",
            "(affects S).map(s -> s.node)",
            "(model |> why).map(s -> s.node)",
        ] {
            let left_value = left.run(query).ok().map(|value| json_of(&left, &value));
            let right_value = right.run(query).ok().map(|value| json_of(&right, &value));
            assert_eq!(left_value, right_value, "{enumeration}\n{query}");
        }
    }
    let mut runtime = Runtime::new();
    runtime.run("enum S: A, B = 2\nt = S.B").unwrap();
    assert_eq!(
        evaluate(&mut runtime, "(why t).map(s -> s.rule)"),
        json!(["t = S.B", "enum S:A,B=2"])
    );
    assert_eq!(evaluate(&mut runtime, "(why S)[0].state"), json!("stated"));
    runtime.run("enum One:\n    ONLY").unwrap();
    assert_eq!(
        evaluate(&mut runtime, "(why One)[0].rule"),
        json!("enum One:\n    ONLY")
    );
}

#[test]
fn canonical_rendering_writes_the_declaration_and_parses_back_to_it() {
    for (source, rendered) in [
        ("enum S: A, B", "enum S:A,B"),
        ("enum S: A = \"A\", B = \"b\"", "enum S:A,B=\"b\""),
        (
            "enum L: LOW = 1, HIGH = -2.5, Z = -0, P = +3, E = 1e3",
            "enum L:LOW=1,HIGH=-2.5,Z=-0,P=3,E=1000",
        ),
        (
            "enum F: ON = true, OFF = false, N = None, U = undefined",
            "enum F:ON=true,OFF=false,N=null,U=null",
        ),
        (
            "enum Q: TEXT = \"a\\\"b\\\\c\\nd\", B",
            "enum Q:TEXT=\"a\\\"b\\\\c\\nd\",B",
        ),
        (
            "enum C:\n    RED\n    GREEN = \"g\"",
            "enum C:RED,GREEN=\"g\"",
        ),
        ("enum U:\n    ONLY", "enum U:\n    ONLY"),
        ("enum U: ONLY = 1, OTHER", "enum U:ONLY=1,OTHER"),
        ("enum U:\n    ONLY = 1", "enum U:\n    ONLY=1"),
    ] {
        let statements = parse(source).unwrap();
        assert_eq!(generate_all(&statements), rendered, "{source}");
        assert_eq!(parse(rendered).unwrap(), statements, "{source}");
    }
    assert_eq!(
        generate_all(&parse("{ enum S: A, B }").unwrap()),
        "{enum S:A,B}"
    );
    assert_eq!(
        generate_all(&parse("if ready: enum S: A, B").unwrap()),
        "if(ready){enum S:A,B}"
    );
    assert_eq!(
        Expr::Enum(vec![named("A"), member("B", ParameterDefault::Number(1.0))]).to_string(),
        "enum{A,B=1}"
    );
    assert_eq!(named("A").to_string(), "A");
    assert_eq!(
        member("A", ParameterDefault::String("a".to_string())).to_string(),
        "A=\"a\""
    );
    assert_eq!(
        generate_all(&[Stmt::Assign {
            target: Expr::Member {
                object: Box::new(Expr::Identifier("a".to_string())),
                property: "b".to_string(),
            },
            value: Expr::Enum(vec![named("X")]),
        }]),
        "a.b=enum{X}"
    );
}

#[test]
fn large_enums_parse_evaluate_and_render_linearly() {
    let names: Vec<String> = (0..20_000).map(|index| format!("M{index}")).collect();
    let source = format!("enum Big:\n    {}", names.join("\n    "));
    let statements = parse(&source).unwrap();
    assert_eq!(
        generate_all(&statements),
        format!("enum Big:{}", names.join(","))
    );
    let mut runtime = Runtime::new();
    runtime.run(&source).unwrap();
    assert_eq!(
        evaluate(&mut runtime, "[Big.M0, Big.M19999, Big.M20000]"),
        json!(["M0", "M19999", null])
    );
    syntax_error(
        &format!("{source}\n    M7"),
        "Enum 'Big' already has member 'M7'",
        20_002,
        5,
    );
    let line: Vec<String> = (0..2_000).map(|index| format!("N{index}")).collect();
    let source = format!("enum Wide: {}", line.join(", "));
    let statements = parse(&source).unwrap();
    assert_eq!(parse(&generate_all(&statements)).unwrap(), statements);
    assert_eq!(
        result(&format!("{source}\n[Wide.N0, Wide.N1999]")),
        json!(["N0", "N1999"])
    );
    syntax_error(
        &format!("{source}, N7"),
        "Enum 'Wide' already has member 'N7'",
        1,
        source.len() + 3,
    );
}

#[test]
fn enum_declarations_pass_the_statement_depth_guard() {
    let nested =
        |depth: usize, body: &str| format!("{}{body}{}", "{ ".repeat(depth), " }".repeat(depth));
    let indented =
        |depth: usize, body: &str| format!("{}{body}\n{}", "{ ".repeat(depth), "}".repeat(depth));
    for depth in [62, 63, 64, 65, 4096] {
        let statement =
            Runtime::check(&nested(depth, "pass")).map_err(|error| (error.kind(), error.message()));
        let line = Runtime::check(&nested(depth, "enum S: A, B"))
            .map_err(|error| (error.kind(), error.message()));
        let block = Runtime::check(&indented(depth, "enum S:\n    A"))
            .map_err(|error| (error.kind(), error.message()));
        assert_eq!(line, statement, "{depth}");
        assert_eq!(block, statement, "{depth}");
    }
    assert!(Runtime::check(&nested(63, "enum S: A, B")).is_ok());
    assert_eq!(
        Runtime::check(&nested(64, "enum S: A, B"))
            .unwrap_err()
            .message(),
        "Statements are nested too deeply"
    );
}

#[test]
fn cli_prints_enums_as_objects_and_lists_them_as_variables() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("enum-declarations.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.lines().next(),
        Some("[{\"PAID\":\"PAID\",\"PENDING\":\"PENDING\"},true]")
    );
    assert!(
        stdout.contains("  Status [variable]\n    updates  order.paid\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("reads    order.status, Status, obj:"),
        "{stdout}"
    );
    let plain = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("fixtures")
                .join("enum-declarations.nuc"),
        )
        .output()
        .unwrap();
    let stdout = String::from_utf8(plain.stdout).unwrap();
    assert!(stdout.starts_with("[object obj:"), "{stdout}");
    assert!(stdout.trim_end().ends_with(",true"), "{stdout}");
}

#[test]
fn the_line_repl_buffers_indented_enums_until_a_blank_line() {
    let output = binary(
        &["--json"],
        Some(
            "enum Status:\n    PENDING\n    PAID\n\nStatus.PAID\nenum Level: LOW = 1, HIGH = 3\nLevel.HIGH\nStatus.PAID = \"X\"\nStatus\n",
        ),
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "{BANNER}{{\"PAID\":\"PAID\",\"PENDING\":\"PENDING\"}}\n\"PAID\"\n{{\"HIGH\":3.0,\"LOW\":1.0}}\n3.0\n{{\"PAID\":\"PAID\",\"PENDING\":\"PENDING\"}}\n"
        )
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "TypeError: Enums are read-only (line 1, column 1)\n"
    );
}

#[test]
fn only_the_appended_cases_declare_enums_and_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flatten() {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(declares_enum(&case.source), new, "{}", case.title);
        if new {
            let count =
                common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            common::check_with_assertions(case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assertions += count;
        }
    }
    for title in SPEC_TITLES
        .into_iter()
        .chain([REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE])
    {
        let matching = documents
            .iter()
            .flatten()
            .filter(|case| case.title == title)
            .count();
        assert_eq!(matching, 1, "{title}");
    }
    assert_eq!(assertions, 37);
    assert_eq!(cases, 1105);
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Enum declarations\n\n```\n")
        .expect("the language reference documents enum declarations");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 90](nuc-0090.md)"));
    assert_eq!(case.expected.as_deref(), Some("1"));
}
