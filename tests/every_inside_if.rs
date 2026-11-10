mod common;

use std::time::{Duration, Instant};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_expression};
use nucleoid::{Error, ErrorKind, Position, Runtime};

const EVERY_INSIDE_IF: &str = "Use all instead of every inside if";

const EXAMPLE: &str = "class Object(number):\n    this.number = number\n\nevery Object.label is square\n\nif any Object.number > 4:\n    all Object.color is red\n\na = Object(3)\nb = Object(7)\nassert([a.label, b.label], [\"square\", \"square\"])\nassert([a.color, b.color], [null, \"red\"])\n[a.color, b.color]";

const IF_POSITIONS: [&str; 58] = [
    "if {c}.total > 100:\n    r = 1",
    "if {c}.total > 100\n    r = 1",
    "if {c}.total > 100: r = 1",
    "if {c}:\n    r = 1",
    "if ({c}.total > 100):\n    r = 1",
    "if c:\n    r = 1\nelif {c}.total > 100:\n    r = 2",
    "if c:\n    r = 1\nelse if {c}.total > 100:\n    r = 2",
    "if c:\n    r = 1\nelif if {c}.total > 100:\n    r = 2",
    "if c:\n    r = 1\nelse {c}.total > 100\n    r = 2",
    "if c:\n    r = 1\nelif {c}.total > 100\n    r = 2",
    "if c:\n    r = 1\nelse if {c}.total > 100\n    r = 2",
    "if c:\n    {c}.vip = true",
    "if c\n    {c}.vip = true",
    "if c: {c}.vip = true",
    "if c: r = 1; {c}.vip = true",
    "if c:\n    r = 1\nelse:\n    {c}.vip = true",
    "if c: r = 1 else: {c}.vip = true",
    "if c:\n    r = 1\nelif d:\n    {c}.vip = true",
    "if c:\n    r = 1\nelse d\n    {c}.vip = true",
    "if c:\n    pass\nelif d:\n    pass\nelse:\n    {c}.vip = true",
    "if c:\n    if d:\n        {c}.vip = true",
    "if c:\n    {\n        {c}.vip = true\n    }",
    "if c:\n    while d:\n        {c}.vip = true",
    "if c:\n    while {c}.open:\n        r = 1",
    "if c:\n    for v in xs:\n        {c}.vip = true",
    "if c:\n    for v in {c}.items:\n        r = 1",
    "if c:\n    try:\n        {c}.vip = true\n    catch e:\n        pass",
    "if c:\n    suppose:\n        {c}.vip = true",
    "if c:\n    suppose {c}.total = 1:\n        r = 1",
    "if c:\n    f = function() { {c}.vip = true }",
    "if c:\n    f = () => {c}.total",
    "if c:\n    f = () => { {c}.vip = true }",
    "if c:\n    f = lambda: {c}.total",
    "if c:\n    def g():\n        {c}.vip = true",
    "if c:\n    def g() => {c}.total",
    "if c:\n    function g() { {c}.vip = true }",
    "if c:\n    class K:\n        {c}.vip = true",
    "if c:\n    t = `${({c})}`",
    "if `${({c}.total)}` == \"5\":\n    r = 1",
    "if c:\n    r = 1 if {c}.vip else 2",
    "if c:\n    r = (1 if {c}.vip else 2)",
    "if c:\n    r = d ? {c}.vip : 2",
    "if c:\n    r = [v for v in xs if {c}.vip]",
    "if c:\n    r = [{c} for v in xs]",
    "if c:\n    r = {k: {c}.vip}",
    "if c:\n    r = {k: {c} j: 1}",
    "if c:\n    r = f({c}.vip)",
    "if c:\n    r = [{c}, 1]",
    "if c:\n    r = {c}@first",
    "if c:\n    r = xs[{c}]",
    "if c:\n    r = why {c}.total",
    "if c:\n    r = sum of {c}",
    "if c:\n    {c}.kind is mortal",
    "if c:\n    r is {c}",
    "if c:\n    {c} is mortal",
    "if c:\n    ({c}) is mortal",
    "if c:\n    {a} = {c}",
    "if c:\n    o.total = .size + {c}.total",
];

const OUTSIDE_POSITIONS: [&str; 28] = [
    "{c}.vip = true",
    "{c}.big = {c}.total > 10",
    "{c} is mortal",
    "r = 1 if {c} is mortal else 2",
    "{c}.fill is 5 if {c}.parts.length is 1 else 0",
    "while {c}.open:\n    {c}.vip = true\n    r = 1",
    "for v in {c}.items:\n    {c}.vip = true",
    "for v in xs:\n    {c}.vip = true",
    "try:\n    {c}.vip = true\ncatch e:\n    {c}.vip = false",
    "suppose {c}.total = 1:\n    {c}.vip = true",
    "suppose:\n    {c}.vip = true",
    "def g():\n    {c}.vip = true",
    "def g() => {c}.total",
    "function g() { {c}.vip = true }",
    "f = () => { {c}.vip = true }",
    "f = lambda: {c}.total",
    "class K:\n    {c}.vip = true",
    "class K: {c}.vip = true",
    "r = 1 if {c}.vip else 2",
    "r = {c}.vip if c else 2",
    "r = c ? {c}.vip : 2",
    "r = [v for v in xs if {c}.vip]",
    "r = `${({c})}`",
    "{ {c}.vip = true }",
    "{a} = {c}",
    "o.total = .size + {c}.total",
    "r = 1\nif c:\n    r = 2\n{c}.vip = true",
    "def g():\n    if c:\n        r = 1\n    {c}.vip = true",
];

const KEPT_READINGS: [&str; 37] = [
    "every = 3",
    "every is 3",
    "every is not 3",
    "r = (every is 3)",
    "every x: int",
    "every assert: 1, 1",
    "every lambda: 1",
    "r = {k: every x: 1}",
    "r = `${every x}`",
    "r = `${1 + every x}`",
    "r = `${every v in xs}`",
    "r = xs.every(v => v > 0)",
    "r = x.every",
    "r = every.x",
    "r = every(1)",
    "r = [every, x]",
    "r = every + 1",
    "r = every ?? 1",
    "r = every |> why",
    "every\nx = 1",
    "def every(x) => x",
    "def f(every) => every",
    "def f every x => every + x",
    "r = lambda every: every",
    "r = every => every",
    "r = {every: 1}",
    "r = {every}",
    "r = f(every=1)",
    "{every} = o",
    "every, x = [1, 2]",
    "for every in xs:\n    pass",
    "r = all every in xs: every > 0",
    "r = [every for every in xs]",
    "try:\n    x = 1\ncatch every:\n    pass",
    "while every lambda: 1",
    "suppose every lambda: 1",
    "every@first",
];

const NESTED_SHAPES: [(&str, &str, &str, &str, &str); 6] = [
    ("class Order\n", "if c: ", "{c}.y = 1", "", ""),
    (
        "class Order\n",
        "if c: f = function() { ",
        "{c}.y = 1",
        " }",
        "",
    ),
    ("class Order\nif ", "(", "{c}.x", ")", ":\n    r = 1"),
    ("class Order\nif c: r = ", "[", "{c}", "]", ""),
    ("class Order\nif c: r = ", "() => ", "{c}.x", "", ""),
    ("class Order\nif c: r = ", "(", "{c}.x + 1", ")", ""),
];

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn rendered(source: &str) -> String {
    generate_all(&statements(source))
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn misplaced(source: &str) -> (String, Position) {
    let index = source.find("every").unwrap();
    let before = source.get(..index).unwrap();
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap().chars().count() + 1;
    (EVERY_INSIDE_IF.to_string(), Position::new(line, column))
}

fn spelled(template: &str, word: &str) -> String {
    template.replace("{c}", &format!("{word} Order"))
}

fn dollar(template: &str) -> String {
    template.replace("{c}", "$Order")
}

fn renamed(source: &str) -> String {
    format!("{:?}", parse(&source.replace("every", "qq"))).replace("qq", "every")
}

fn fresh(source: &str, expected: &str) {
    let parsed = statements(source);
    let assertions = common::assertions::protected_assertions(&parsed).len();
    let case = common::Case {
        title: "every inside if regression".to_string(),
        source: source.to_string(),
        expected: Some(expected.to_string()),
    };
    common::check_with_count(&mut Runtime::new(), &case, assertions)
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn class(name: &str) -> Expr {
    Expr::ClassRef(name.to_string())
}

fn nested(shape: (&str, &str, &str, &str, &str), word: &str, depth: usize) -> String {
    let (head, opener, base, closer, tail) = shape;
    let base = if word == "$" {
        dollar(base)
    } else {
        spelled(base, word)
    };
    format!(
        "{head}{}{base}{}{tail}",
        opener.repeat(depth),
        closer.repeat(depth)
    )
}

fn too_deep(message: &str) -> bool {
    matches!(
        message,
        "Expressions are nested too deeply" | "Statements are nested too deeply"
    )
}

#[test]
fn the_users_example_writes_any_in_the_condition_and_all_in_its_clause() {
    fresh(EXAMPLE, "[null,\"red\"]");
    assert_eq!(
        syntax_error(&EXAMPLE.replace("any Object", "every Object")),
        (EVERY_INSIDE_IF.to_string(), Position::new(6, 4))
    );
    assert_eq!(
        syntax_error(&EXAMPLE.replace("all Object", "every Object")),
        (EVERY_INSIDE_IF.to_string(), Position::new(7, 5))
    );
    let mut runtime = Runtime::new();
    let error = runtime
        .run("x = 1\nif every Object.number > 4:\n    y = 2")
        .unwrap_err();
    assert_eq!(
        (error.kind(), error.message(), error.position()),
        (
            ErrorKind::Syntax,
            EVERY_INSIDE_IF.to_string(),
            Some(Position::new(2, 4))
        )
    );
    assert_eq!(runtime.run("x").unwrap_err().message(), "x is not defined");
}

#[test]
fn every_class_references_raise_the_error_wherever_an_if_statement_holds_them() {
    for template in IF_POSITIONS {
        let source = spelled(template, "every");
        assert_eq!(syntax_error(&source), misplaced(&source), "{source}");
        let reference = dollar(template);
        assert_eq!(
            rendered(&spelled(template, "all")),
            rendered(&reference),
            "{template}"
        );
    }
    for (source, line, column) in [
        ("if c:\n    r = every /* note */ Order", 2, 9),
        ("if c:\n    r = (every\nOrder)", 2, 10),
        ("if c:\n    r = every Order(5)", 2, 9),
        (
            "if c:\n    r = 1\nelse:\n    r = every Order\n    s = every Order",
            4,
            9,
        ),
        ("if every Order.a > 1: every Order.b = 2", 1, 4),
        ("if c: if every Order.a > 1: r = 1", 1, 10),
        ("if c:\n    every = 3\n    every Order = 4", 3, 5),
    ] {
        assert_eq!(
            syntax_error(source),
            (EVERY_INSIDE_IF.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn every_class_references_and_sentences_outside_if_statements_keep_their_reading() {
    for template in OUTSIDE_POSITIONS {
        let source = spelled(template, "every");
        let reference = dollar(template);
        assert_eq!(statements(&source), statements(&reference), "{source}");
        assert_eq!(rendered(&source), rendered(&reference), "{source}");
    }
    assert_eq!(parse_expression("every Human").unwrap(), class("Human"));
    assert_eq!(
        parse_expression("every Human is mortal").unwrap(),
        class("Human")
    );
    let error = parse_expression("function() { if c: r = every Human }").unwrap_err();
    assert_eq!(
        (error.message(), error.position()),
        (EVERY_INSIDE_IF.to_string(), Some(Position::new(1, 24)))
    );
    fresh(
        "class Order(total):\n    this.total = total\nevery Order.big = every Order.total > 10\nif any Order.total > 100:\n    all Order.vip = true\nsmall = Order(5)\nlarge = Order(200)\nassert([small.big, large.big], [false, true])\nassert([small.vip, large.vip], [null, true])\nevery Order.tier is \"gold\"\nassert(large.tier, \"gold\")\nsmall.big",
        "false",
    );
}

#[test]
fn every_keeps_its_other_readings_inside_if_statements() {
    let mut compared = 0;
    for statement in KEPT_READINGS {
        let indented = statement
            .lines()
            .map(|line| format!("    {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        for source in [
            format!("if c:\n{indented}"),
            format!("if c:\n    pass\nelse:\n{indented}"),
        ] {
            assert_eq!(
                format!("{:?}", parse(&source)),
                renamed(&source),
                "{source}"
            );
            compared += 1;
        }
    }
    assert_eq!(compared, 74);
    for (every, all) in [
        (
            "if every v in xs: v > 0:\n    r = 1",
            "if all v in xs: v > 0:\n    r = 1",
        ),
        (
            "if every v in xs: v > 0\n    r = 1",
            "if all v in xs: v > 0\n    r = 1",
        ),
        (
            "if c:\n    r = 1\nelse every v in xs: v > 0\n    r = 2",
            "if c:\n    r = 1\nelse all v in xs: v > 0\n    r = 2",
        ),
        (
            "if c:\n    r = every v in Order: v.total > 10",
            "if c:\n    r = all v in Order: v.total > 10",
        ),
    ] {
        assert_eq!(statements(every), statements(all), "{every}");
    }
    for source in [
        "if every is 3:\n    r = 1",
        "if every:\n    r = 1",
        "if every.ready:\n    r = 1",
        "if every(1):\n    r = 1",
        "if `${every x}` == \"3\":\n    r = 1",
    ] {
        assert_eq!(format!("{:?}", parse(source)), renamed(source), "{source}");
    }
    fresh(
        "every = 2\nxs = [1, 2]\nc = true\nif c:\n    r = every v in xs: v > 0\n    t = `${every x}`\n    u = every + 1\nassert([r, t, u], [true, \"2\", 3])\nu",
        "3",
    );
}

#[test]
fn any_all_and_dollar_class_references_keep_their_reading_inside_if_statements() {
    for template in IF_POSITIONS {
        let reference = statements(&dollar(template));
        for word in ["all", "any"] {
            assert_eq!(
                statements(&spelled(template, word)),
                reference,
                "{template}"
            );
        }
    }
    fresh(
        "class Order(total):\n    this.total = total\nlarge = Order(200)\nif any Order.total > 100:\n    all Order.vip = true\nelif $Order.total > 50:\n    any Order.vip = false\nsmall = Order(60)\nassert([small.vip, large.vip], [false, true])\nsmall.total = 150\nsmall.vip",
        "true",
    );
}

#[test]
fn probes_and_sub_parsers_report_the_error_at_its_every() {
    for source in [
        "if c:\n    x = (1 if every Order.ok else 2)",
        "if c:\n    while every Order.total > 1:\n        r = 1",
        "if c:\n    while `${(every Order)}` == \"x\":\n        r = 1",
        "if c:\n    while x is every Order.total:\n        r = 1",
        "if c:\n    suppose every Order.total = 1:\n        r = 1",
        "if c:\n    suppose x is every Order.total:\n        r = 1",
        "if c:\n    {a} = every Order",
        "if c:\n    t = `${(every Order)}`",
        "if c:\n    t = `a\n${(every Order)}`",
        "if x is every Order:\n    r = 1",
        "if (x is every Order):\n    r = 1",
        "if c:\n    r = [v for v in xs if every Order.ok]",
        "if c:\n    o.total = .size + every Order.total",
        "if c:\n    o.total = .size\n    r = every Order",
    ] {
        assert_eq!(syntax_error(source), misplaced(source), "{source}");
    }
    for (source, message, line, column) in [
        (
            "if c:\n    x = (2 if every Order.ok)",
            "Expected ')' but found keyword 'if'",
            2,
            12,
        ),
        ("if c:\n    a, b = every Order, 2", "Unexpected ','", 2, 8),
        ("if c:\n    {a} = every Order, 2", "Unexpected '='", 2, 11),
        (
            "if c:\n    x = (2 if every Order.ok else)",
            "Unexpected ')'",
            2,
            35,
        ),
        (
            "if c:\n    x = 1 if every Order.ok\n    y = 2",
            "Expected ':' but found end of line",
            2,
            26,
        ),
        (
            "if c:\n    while every Order.x y:\n        r = 1",
            "Unexpected ':'",
            2,
            26,
        ),
        (
            "if every Order.total > 1:\n    y = (\n",
            "Unexpected the end of a block",
            3,
            1,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    for source in [
        "if c:\n    while every lambda: 1",
        "if c:\n    suppose every lambda: 1",
        "if c:\n    r = (every is 3)",
        "if c:\n    r = (1 if every else 2)",
    ] {
        assert_eq!(format!("{:?}", parse(source)), renamed(source), "{source}");
    }
}

#[test]
fn nested_every_inside_if_stops_where_its_all_spelling_stops() {
    let started = Instant::now();
    for shape in NESTED_SHAPES {
        let mut limit = None;
        for depth in 1..200 {
            let source = nested(shape, "every", depth);
            let every = Runtime::check(&source).unwrap_err();
            let accepted = Runtime::check(&nested(shape, "all", depth));
            let dollar = Runtime::check(&nested(shape, "$", depth));
            assert_eq!(
                accepted.as_ref().map_err(Error::message),
                dollar.as_ref().map_err(Error::message),
                "{shape:?} at depth {depth}"
            );
            let Err(error) = accepted else {
                assert_eq!(
                    syntax_error(&source),
                    misplaced(&source),
                    "{shape:?} at depth {depth}"
                );
                continue;
            };
            assert!(too_deep(&error.message()), "{shape:?}: {error}");
            assert_eq!(
                every.message(),
                error.message(),
                "{shape:?} at depth {depth}"
            );
            limit = Some(depth);
            break;
        }
        assert!(limit.is_some_and(|depth| depth > 15), "{shape:?}");
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{shape:?} took {:?}",
            started.elapsed()
        );
    }
}
