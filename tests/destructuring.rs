mod common;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_program};
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde_json::json;

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid destructures object fields into standing declarations",
    "Nucleoid destructures renamed fields of instances",
    "Nucleoid unpacks lists and strings into comma separated names",
    "Nucleoid destructures inside functions blocks and loops",
    "Nucleoid destructures in conditional branches and hypotheticals",
    "Nucleoid explains destructured names as their declarations",
    "Nucleoid keeps block object and list readings beside destructuring",
    "Nucleoid rolls back failed destructuring",
];

const REFERENCE_TITLE: &str = "Nucleoid destructures records and pairs into live names";
const README_TITLE: &str = "Destructuring declares one live name per field or position";
const EXAMPLES_TITLE: &str = "Destructuring keeps a rover heading model live";

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "destructuring regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
    value
}

fn integral(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Number(number) => match number.as_f64() {
            Some(float) if float.fract() == 0.0 && float.abs() < 1e15 => json!(float as i64),
            _ => serde_json::Value::Number(number),
        },
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(integral).collect())
        }
        serde_json::Value::Object(fields) => serde_json::Value::Object(
            fields
                .into_iter()
                .map(|(key, value)| (key, integral(value)))
                .collect(),
        ),
        other => other,
    }
}

fn read(runtime: &mut Runtime, source: &str) -> serde_json::Value {
    let value = checked(runtime, source);
    integral(serde_json::from_str(&runtime.serialize_json(&value).unwrap()).unwrap())
}

fn evaluate(source: &str) -> serde_json::Value {
    read(&mut Runtime::new(), source)
}

fn failure_in(runtime: &mut Runtime, source: &str) -> (ErrorKind, String, Option<Position>) {
    let error = runtime.run(source).unwrap_err();
    (error.kind(), error.message(), error.position())
}

fn failure(source: &str) -> (ErrorKind, String, Option<Position>) {
    failure_in(&mut Runtime::new(), source)
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn pattern_shaped(tokens: &[Token]) -> bool {
    let assign = |rest: &[Token]| {
        rest.iter()
            .position(|token| !matches!(token, Token::Identifier(_) | Token::Colon | Token::Comma))
            .is_some_and(|end| {
                rest.get(end) == Some(&Token::RightBrace)
                    && rest.get(end + 1) == Some(&Token::Assign)
            })
    };
    match tokens {
        [Token::LeftBrace, Token::Identifier(_), rest @ ..] => assign(rest),
        [Token::Identifier(_), Token::Comma, rest @ ..] => rest
            .iter()
            .position(|token| !matches!(token, Token::Identifier(_) | Token::Comma))
            .is_some_and(|end| rest[end] == Token::Assign),
        _ => false,
    }
}

fn uses_destructuring(source: &str) -> bool {
    let tokens: Vec<Token> = tokenize(source.trim())
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    (0..tokens.len()).any(|start| {
        let begins = start == 0
            || matches!(
                tokens[start - 1],
                Token::Newline
                    | Token::Semicolon
                    | Token::Indent
                    | Token::Dedent
                    | Token::LeftBrace
                    | Token::Colon
            );
        begins && pattern_shaped(&tokens[start..])
    })
}

fn nested(depth: usize, innermost: &str, size: usize, block: fn(&str) -> String) -> String {
    let numbers = (0..size)
        .map(|number| number.to_string())
        .collect::<Vec<_>>()
        .join(",\n");
    (0..depth)
        .rev()
        .fold(innermost.to_string(), |inner, level| {
            format!("{{v{level}}} = [{numbers}, {}]", block(&inner))
        })
}

#[test]
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let readme = common::snippets(include_str!("../README.md"), "README.md");
    let examples = common::snippets(include_str!("../docs/examples.md"), "docs/examples.md");
    let appended = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (&reference, REFERENCE_TITLE),
        (&readme, README_TITLE),
        (&examples, EXAMPLES_TITLE),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(uses_destructuring(&case.source), "{title} destructures");
        let statements = parse(&case.source).unwrap();
        let count = common::assertions::protected_assertions(&statements).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 45);
}

#[test]
fn accepted_documents_destructure_only_in_the_appended_cases() {
    let documents = [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ];
    let mut destructuring = 0;
    let mut cases = 0;
    for case in documents.iter().flatten() {
        cases += 1;
        let uses = uses_destructuring(&case.source);
        let new = SPEC_TITLES.contains(&case.title.as_str())
            || [REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE].contains(&case.title.as_str());
        if uses {
            destructuring += 1;
        }
        assert_eq!(uses, new, "{}", case.title);
    }
    assert_eq!(destructuring, 11);
    assert_eq!(cases, 904);
}

#[test]
fn destructuring_parses_to_the_written_out_declarations() {
    for (destructuring, declarations) in [
        ("{x, y} = point", "x = point.x\ny = point.y"),
        ("{x,y}=point", "x = point.x\ny = point.y"),
        ("{ x, y, } = point", "x = point.x\ny = point.y"),
        ("{x} = point", "x = point.x"),
        ("{x,} = point", "x = point.x"),
        ("{x: a, y: b} = p", "a = p.x\nb = p.y"),
        ("{x: a y: b} = p", "a = p.x\nb = p.y"),
        ("{x: x} = p", "x = p.x"),
        (
            "{name: other area: larger, depth} = den",
            "other = den.name\nlarger = den.area\ndepth = den.depth",
        ),
        ("a, b = pair", "a = pair[0]\nb = pair[1]"),
        ("a, b, = pair", "a = pair[0]\nb = pair[1]"),
        ("first, = word", "first = word[0]"),
        ("a,b,c=xs", "a = xs[0]\nb = xs[1]\nc = xs[2]"),
        (
            "{x, y} = rover.position",
            "x = rover.position.x\ny = rover.position.y",
        ),
        (
            "dx, dy = rover.heading",
            "dx = rover.heading[0]\ndy = rover.heading[1]",
        ),
        (
            "{qty, price} = checked(order)",
            "qty = checked(order).qty\nprice = checked(order).price",
        ),
        ("{x} = a ?? b", "x = (a ?? b).x"),
        ("{x} = c ? p : q", "x = (c ? p : q).x"),
        ("{x} = p if c else q", "x = (p if c else q).x"),
        ("a, b = c ? p : q", "a = (c ? p : q)[0]\nb = (c ? p : q)[1]"),
        ("a, b = [1, 2]", "a = [1, 2][0]\nb = [1, 2][1]"),
        ("a, b = \"ok\"", "a = \"ok\"[0]\nb = \"ok\"[1]"),
        ("{f} = lambda: 1", "f = (lambda: 1).f"),
        ("{length} = items", "length = items.length"),
        ("{value} = box", "value = box.value"),
        ("{x} = suppose", "x = suppose.x"),
        (
            "{assert: a, lambda: b, suppose: c} = p",
            "a = p.assert\nb = p.lambda\nc = p.suppose",
        ),
        ("{x: int} = point", "int = point.x"),
        ("lambda, all = pair", "lambda = pair[0]\nall = pair[1]"),
        ("{x, y: x} = p", "x = p.x\nx = p.y"),
        ("a, a = pair", "a = pair[0]\na = pair[1]"),
        ("{x} = point; y = 1", "x = point.x; y = 1"),
        ("{x} = point # comment", "x = point.x"),
        (
            "if ready: low, high = pair",
            "if ready: low = pair[0]; high = pair[1]",
        ),
        (
            "if c: a, b = pair else: a = 0",
            "if c: a = pair[0]; b = pair[1] else: a = 0",
        ),
        (
            "try: {x} = point catch e: x = 2",
            "try: x = point.x catch e: x = 2",
        ),
        ("{ {x, y} = point }", "{ x = point.x\ny = point.y }"),
        ("{ a, b = pair }", "{ a = pair[0]; b = pair[1] }"),
        (
            "f = (p) => { {x, y} = p; return x + y }",
            "f = (p) => { x = p.x; y = p.y; return x + y }",
        ),
        (
            "f = function() { a, b = pair }",
            "f = function() { a = pair[0]; b = pair[1] }",
        ),
        (
            "def length(leg):\n    start, end = leg.span\n    return end - start",
            "def length(leg):\n    start = leg.span[0]\n    end = leg.span[1]\n    return end - start",
        ),
        (
            "class Box:\n    {size} = config",
            "class Box:\n    size = config.size",
        ),
        (
            "if ready:\n    a, b = pair",
            "if ready:\n    a = pair[0]\n    b = pair[1]",
        ),
        (
            "if c:\n    pass\nelse:\n    a, b = pair",
            "if c:\n    pass\nelse:\n    a = pair[0]\n    b = pair[1]",
        ),
        (
            "for leg of Leg:\n    start, end = leg.span",
            "for leg of Leg:\n    start = leg.span[0]\n    end = leg.span[1]",
        ),
        (
            "for side, pair of pairs:\n    low, high = pair",
            "for side, pair of pairs:\n    low = pair[0]\n    high = pair[1]",
        ),
        (
            "for side, pair of pairs: {low, high} = pair",
            "for side, pair of pairs: low = pair.low; high = pair.high",
        ),
        (
            "total = suppose:\n    {x, y} = point\n    x + y",
            "total = suppose:\n    x = point.x\n    y = point.y\n    x + y",
        ),
        (
            "if true:\n    a, b = [1, 2]",
            "if true:\n    a = [1, 2][0]\n    b = [1, 2][1]",
        ),
    ] {
        assert_eq!(
            parse(destructuring).unwrap(),
            parse(declarations).unwrap(),
            "{destructuring}"
        );
    }
}

#[test]
fn each_target_lowers_to_a_member_or_index_read_of_the_value() {
    let value = Expr::Call {
        callee: Box::new(Expr::Identifier("source".to_string())),
        arguments: vec![],
    };
    assert_eq!(
        parse("{x, y: z} = source()\na, b = source()").unwrap(),
        vec![
            Stmt::Assign {
                target: Expr::Identifier("x".to_string()),
                value: Expr::Member {
                    object: Box::new(value.clone()),
                    property: "x".to_string(),
                },
            },
            Stmt::Assign {
                target: Expr::Identifier("z".to_string()),
                value: Expr::Member {
                    object: Box::new(value.clone()),
                    property: "y".to_string(),
                },
            },
            Stmt::Assign {
                target: Expr::Identifier("a".to_string()),
                value: Expr::Index {
                    object: Box::new(value.clone()),
                    index: Box::new(Expr::Number(0.0)),
                },
            },
            Stmt::Assign {
                target: Expr::Identifier("b".to_string()),
                value: Expr::Index {
                    object: Box::new(value),
                    index: Box::new(Expr::Number(1.0)),
                },
            },
        ]
    );
}

#[test]
fn every_declaration_keeps_the_position_of_its_destructuring() {
    let program = parse_program("q = 1\n{x, y} = point\n  \na, b, c = xs\nz = 2").unwrap();
    assert_eq!(
        program.positions,
        vec![
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(2, 1),
            Position::new(4, 1),
            Position::new(4, 1),
            Position::new(4, 1),
            Position::new(5, 1),
        ]
    );
    assert_eq!(program.statements.len(), 7);
}

#[test]
fn destructured_names_explain_and_render_as_their_declarations() {
    let mut destructured = Runtime::new();
    let mut written = Runtime::new();
    let setup = "point = {x: 3, y: 4}\npair = [5, 6]\n";
    read(
        &mut destructured,
        &format!("{setup}{{x, y}} = point\nfirst, second = pair\n{{ {{left, right}} = point }}"),
    );
    read(
        &mut written,
        &format!(
            "{setup}x = point.x\ny = point.y\nfirst = pair[0]\nsecond = pair[1]\n{{ left = point.left\nright = point.right }}"
        ),
    );
    let explanation = "[(affects point).map(step -> step.rule), (affects pair).map(step -> step.rule), (why y)[0].rule, (why second)[0].rule, (why second)[0].from]";
    assert_eq!(
        read(&mut destructured, explanation),
        read(&mut written, explanation)
    );
    assert_eq!(
        read(&mut destructured, explanation),
        json!([
            [
                "x = point.x",
                "y = point.y",
                "block(left=point.left;right=point.right)"
            ],
            ["first = pair[0]", "second = pair[1]"],
            "y = point.y",
            "second = pair[1]",
            ["pair"]
        ])
    );
}

#[test]
fn near_misses_keep_their_accepted_diagnostics() {
    for (source, message, line, column) in [
        ("{x} = point if ready", "Unexpected '='", 1, 7),
        ("{x, y} = point)", "Unexpected ','", 1, 5),
        ("{x, y} = point +", "Unexpected ','", 1, 5),
        ("a, b = )", "Unexpected ','", 1, 4),
        ("{x, y} = point = other", "Unexpected ','", 1, 5),
        ("a, b = c = pair", "Unexpected ','", 1, 4),
        ("{x: $A} = point", "Unexpected ':'", 1, 5),
        ("{x, y} := point", "Unexpected ','", 1, 5),
        ("a, b = c += 1", "Unexpected ','", 1, 4),
        ("const [a, b] = pair", "Expected ']' but found ','", 1, 9),
        ("const {x} = point", "Unexpected '='", 1, 13),
        (
            "def f({x, y}) => x",
            "Expected a parameter but found '{'",
            1,
            8,
        ),
        ("{x = 1, y} = p", "Unexpected ','", 1, 9),
        ("{x,, y} = p", "Unexpected ','", 1, 4),
        (
            "if true:\n    pass\nelse a, b = pair",
            "Expected ':' but found identifier 'a'",
            3,
            6,
        ),
        ("{} = point", "Unexpected '='", 1, 6),
        ("{x, y} == point", "Unexpected ','", 1, 5),
        ("{ q = 1 } {x} = point", "Unexpected '='", 1, 17),
        ("b = 2 {x, y} = point", "Unexpected ','", 1, 11),
        ("c = 1 a, b = pair", "Unexpected ','", 1, 10),
        ("{x: a y} = point", "Unexpected ':'", 1, 5),
        ("{if: a} = point", "Unexpected ':'", 1, 6),
        ("{x: this} = point", "Unexpected ':'", 1, 5),
        ("{, x} = p", "Unexpected ','", 1, 4),
        ("let {x, y} = point", "Unexpected ','", 1, 9),
        ("{x, y} =", "Unexpected ','", 1, 5),
        ("a, b =\nc = 1", "Unexpected ','", 1, 4),
        ("{\n    x,\n    y\n} = point", "Unexpected ','", 2, 6),
        ("{x,\ny} = point", "Unexpected ','", 1, 3),
        ("{x: {y}} = p", "Unexpected ':'", 1, 5),
        ("{x}\n= point", "Unexpected '='", 2, 3),
        ("{0: a} = p", "Unexpected ':'", 1, 5),
        ("{x, y} = a, b", "Unexpected ','", 1, 5),
        ("(a, b) = pair", "Expected ')' but found ','", 1, 3),
        ("{x: a.b} = p", "Unexpected ':'", 1, 5),
        ("{ amount: 1 next: 2 }", "Unexpected ':'", 1, 11),
        ("{ amount, label }", "Unexpected ','", 1, 11),
        ("{x} += point", "Unexpected '+='", 1, 8),
        ("a, b += pair", "Unexpected ','", 1, 4),
        ("a.b, c = pair", "Unexpected ','", 1, 6),
        ("a,, b = pair", "Unexpected ','", 1, 3),
        ("a[0], c = pair", "Unexpected ','", 1, 7),
        ("a,\nb = pair", "Unexpected ','", 1, 2),
        ("a, b = suppose pair = [1, 2]: pair", "Unexpected ','", 1, 4),
        ("{x, ...rest} = p", "Unexpected ','", 1, 5),
        ("{x; y} = point", "Unexpected '='", 1, 10),
        ("{x y} = point", "Unexpected '='", 1, 9),
        ("{x y: z} = point", "Unexpected ':'", 1, 7),
        ("a, *rest = p", "Unexpected ','", 1, 4),
        ("{\"x\"} = p", "Unexpected '='", 1, 9),
        ("{\"x\": a} = p", "Unexpected ':'", 1, 7),
        ("{x, y} = suppose: point", "Unexpected ','", 1, 5),
        ("{x} = suppose y = 1: y", "Unexpected '='", 1, 7),
        ("suppose a, b = pair: a", "Unexpected ','", 1, 12),
        ("x = 1\ny = 2\nx, y = y, x", "Unexpected ','", 3, 4),
        ("a, b = 1, 2", "Unexpected ','", 1, 4),
        ("{\n    x: int\n} = point", "Unexpected '='", 3, 5),
        ("{x, y} = point: 1", "Unexpected ','", 1, 5),
        ("{x} = y := 1", "Unexpected '='", 1, 7),
        ("var a, b = pair", "Unexpected ','", 1, 8),
        ("point = {x: 1}\n{x} = point c = 1", "Unexpected '='", 2, 7),
        ("pair = [1, 2]\na, b = pair c = 1", "Unexpected ','", 2, 4),
        ("x = 1 catch e: 1", "Unexpected keyword 'catch'", 1, 13),
        ("x = 1 else: y = 2", "Unexpected keyword 'else'", 1, 11),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn accepted_readings_beside_destructuring_are_unchanged() {
    assert_eq!(evaluate("x = 5\n{ x }"), json!(5));
    assert_eq!(
        evaluate("x = 1\ny = 2\nbox = {x, y,}\nbox"),
        json!({"x": 1, "y": 2})
    );
    assert_eq!(
        evaluate("x = 1\ny = 2\nrecord = {left: x right: y}\nrecord"),
        json!({"left": 1, "right": 2})
    );
    assert_eq!(
        failure("pair = [5, 6]\n[a, b] = pair"),
        (
            ErrorKind::Syntax,
            "Cannot assign to [a,b]".to_string(),
            Some(Position::new(2, 1))
        )
    );
    assert_eq!(evaluate("if false:\n    [a, b] = pair\n1"), json!(1));
    assert_eq!(
        evaluate(
            "point = {x: 1, y: 2}\nadd = lambda a, b: a + b\nok = all v in [1, 2]: v > 0\n[add(1, 2), ok]"
        ),
        json!([3, true])
    );
    for header in ["for a, b of pairs:\n    pass", "for a, b in pairs: pass"] {
        assert_eq!(
            parse(header).unwrap(),
            vec![Stmt::ForEntries {
                key: "a".to_string(),
                value: "b".to_string(),
                iterable: Expr::Identifier("pairs".to_string()),
                body: vec![Stmt::Pass],
            }],
            "{header}"
        );
    }
}

#[test]
fn keyword_and_type_named_keys_and_targets_now_destructure() {
    assert_eq!(
        evaluate(
            "p = {assert: 1, lambda: 2, suppose: 3}\n{assert: a, lambda: b, suppose: c} = p\n[a, b, c]"
        ),
        json!([1, 2, 3])
    );
    assert_eq!(evaluate("point = {x: 7}\n{x: int} = point\nint"), json!(7));
    assert_eq!(evaluate("p = {assert: 4}\n{assert: x} = p\nx"), json!(4));
    assert_eq!(evaluate("p = {lambda: 5}\n{lambda: x} = p\nx"), json!(5));
    assert_eq!(evaluate("p = {suppose: 6}\n{suppose: x} = p\nx"), json!(6));
}

#[test]
fn a_stray_else_or_catch_after_a_destructuring_reports_the_keyword() {
    assert_eq!(
        syntax_error("pair = [1, 2]\na, b = pair else: x = 1"),
        (
            "Unexpected keyword 'else'".to_string(),
            Position::new(2, 17)
        )
    );
    assert_eq!(
        syntax_error("point = {x: 1}\n{x} = point catch e: 1"),
        (
            "Unexpected keyword 'catch'".to_string(),
            Position::new(2, 19)
        )
    );
}

#[test]
fn statements_after_a_destructuring_follow_its_end() {
    assert_eq!(
        evaluate("point = {x: 1}\n{x} = point; c = 2\n[x, c]"),
        json!([1, 2])
    );
    assert_eq!(
        evaluate("pair = [1, 2]\n{ a, b = pair }\n[a, b]"),
        json!([1, 2])
    );
    assert_eq!(
        evaluate("c = false\npair = [1, 2]\nif c: a, b = pair else: a = 0\na"),
        json!(0)
    );
    assert_eq!(
        evaluate("point = {x: 1}\ntry: {x} = point catch e: x = 2\nx"),
        json!(1)
    );
    assert_eq!(evaluate("try: {x} = missing catch e: x = 2\nx"), json!(2));
    assert_eq!(evaluate("if true:\n    a, b = [1, 2]"), json!(2));
}

#[test]
fn a_destructuring_names_at_most_128_targets() {
    let names = |count: usize| {
        (0..count)
            .map(|index| format!("n{index}"))
            .collect::<Vec<_>>()
    };
    let bounded = names(128).join(", ");
    assert_eq!(parse(&format!("{bounded} = xs")).unwrap().len(), 128);
    assert_eq!(
        evaluate(&format!("xs = [1, 2, 3]\n{bounded} = xs\n[n0, n2, n127]")),
        json!([1, 3, null])
    );
    assert_eq!(parse(&format!("{{{bounded}}} = o")).unwrap().len(), 128);

    let over = names(129);
    let column = over[..128].iter().map(|name| name.len() + 2).sum::<usize>() + 1;
    assert_eq!(
        syntax_error(&format!("{} = xs", over.join(", "))),
        (
            "Destructuring has too many targets".to_string(),
            Position::new(1, column)
        )
    );
    assert_eq!(
        syntax_error(&format!("{{{}}} = o", over.join(", "))),
        (
            "Destructuring has too many targets".to_string(),
            Position::new(1, column + 1)
        )
    );
    assert_eq!(
        syntax_error(&format!("{} = 1, 2", over.join(", "))),
        ("Unexpected ','".to_string(), Position::new(1, 5))
    );
}

#[test]
fn a_failed_recognition_stops_before_the_value() {
    for (pattern, message, column) in [
        ("{x}", "Unexpected '='", 7),
        ("{x,}", "Unexpected ','", 4),
        ("{x, y}", "Unexpected ','", 5),
        ("{x: a}", "Unexpected ':'", 5),
        ("{x: a y: b}", "Unexpected ':'", 5),
        ("{x: a, y}", "Unexpected ':'", 5),
        ("{assert: a}", "Expected ',' but found '}'", 11),
        ("{lambda: a}", "Unexpected '='", 15),
        ("{suppose: a}", "Unexpected '='", 16),
        ("{x: int}", "Unexpected ':'", 5),
        ("a,", "Unexpected ','", 4),
        ("a, b", "Unexpected ','", 4),
        ("a, b,", "Unexpected ','", 4),
        ("lambda, b", "Unexpected ','", 9),
        ("all, b", "Unexpected ','", 6),
        ("suppose, b", "Unexpected ','", 10),
        ("assert, b", "Unexpected ','", 9),
    ] {
        for value in ["]", ")"] {
            let source = format!("{pattern} = {value}");
            assert_eq!(
                syntax_error(&source),
                (message.to_string(), Position::new(1, column)),
                "{source}"
            );
        }
    }
}

#[test]
fn nested_destructurings_in_function_values_parse_once() {
    let arrow: fn(&str) -> String = |inner| format!("() => {{ {inner} }}");
    let function: fn(&str) -> String = |inner| format!("function() {{ {inner} }}");
    for block in [arrow, function] {
        let parsed = parse(&nested(15, "w = 1", 2_000, block)).unwrap();
        assert!(
            matches!(
                parsed.as_slice(),
                [Stmt::Assign {
                    target: Expr::Identifier(name),
                    value: Expr::Member { property, .. },
                }] if name == "v0" && property == "v0"
            ),
            "{parsed:?}"
        );
        assert_eq!(
            syntax_error(&nested(15, "{z} = )", 2_000, block)),
            ("Unexpected '='".to_string(), Position::new(1, 8))
        );
        assert_eq!(
            syntax_error(&nested(15, "w = 1", 2_000, block).replacen("w = 1", "w = 1 )", 1)),
            ("Unexpected '='".to_string(), Position::new(1, 8))
        );
    }
}

#[test]
fn destructured_names_stay_live() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "point = {x: 1, y: 2}\n{x, y} = point\npoint.x = 5\nassert: [x, y], [5, 2]\npoint = {x: 7}\nassert: [x, y], [7, null]\npoint.y = 8\nassert: y, 8",
    );
    checked(
        &mut runtime,
        "pair = [1]\nfirst, second = pair\nassert: [first, second], [1, null]\npair = [3, 4, 5]\nassert: [first, second], [3, 4]",
    );
    checked(
        &mut runtime,
        "o = {at: {x: 1, y: 2}}\n{x: ax, y: ay} = o.at\no.at = {x: 5, y: 6}\nassert: [ax, ay], [5, 6]",
    );
}

#[test]
fn the_value_runs_once_per_target() {
    assert_eq!(
        evaluate(
            "class Point(x, y):\n    this.x = x\n    this.y = y\n{x, y} = Point(1, 2)\n[x, y, Point.length]"
        ),
        json!([1, 2, 2])
    );
    assert_eq!(
        evaluate(
            "class Point(x, y):\n    this.x = x\n    this.y = y\np = Point(1, 2)\n{x, y} = p\n[x, y, Point.length]"
        ),
        json!([1, 2, 1])
    );
}

#[test]
fn propagation_runs_between_targets_as_between_declarations() {
    let rule = "limits = [4, 5]\nlow, high = limits\nif low > high: throw \"INVERTED\"\n";
    assert_eq!(
        failure(&format!("{rule}other = [10, 20]\nlow, high = other")),
        failure(&format!(
            "{rule}other = [10, 20]\nlow = other[0]\nhigh = other[1]"
        ))
    );
    assert_eq!(
        failure(&format!("{rule}other = [10, 20]\nlow, high = other")),
        (
            ErrorKind::Thrown,
            "INVERTED".to_string(),
            Some(Position::new(5, 1))
        )
    );
    assert_eq!(
        evaluate(&format!("{rule}limits = [10, 20]\n[low, high]")),
        json!([10, 20])
    );
}

#[test]
fn duplicate_targets_keep_the_last_declaration() {
    assert_eq!(
        evaluate("p = {x: 1, y: 2}\n{x, y: x} = p\n[x, (why x)[0].rule]"),
        json!([2, "x = p.y"])
    );
    assert_eq!(
        evaluate("pair = [1, 2]\na, a = pair\n[a, (why a)[0].rule]"),
        json!([2, "a = pair[1]"])
    );
}

#[test]
fn the_statement_value_is_the_last_target() {
    assert_eq!(evaluate("point = {x: 3, y: 4}\n{x, y} = point"), json!(4));
    assert_eq!(evaluate("a, b = [5, 6]"), json!(6));
    assert_eq!(evaluate("suppose = {x: 3}\n{x} = suppose\nx"), json!(3));
}

#[test]
fn contexts_follow_the_written_out_declarations() {
    assert_eq!(
        evaluate("def f(a):\n    a, b = [10, 20]\n    return a + b\nr = f(1)\n[r, b]"),
        json!([30, 20])
    );
    assert_eq!(
        failure("def f(a):\n    a, b = [10, 20]\n    return a + b\nr = f(1)\na"),
        (
            ErrorKind::Reference,
            "a is not defined".to_string(),
            Some(Position::new(5, 1))
        )
    );
    assert_eq!(
        evaluate(
            "class Leg(span):\n    this.span = span\nLeg([0, 4])\nLeg([4, 10])\ntotal = 0\nfor leg of Leg:\n    start, end = leg.span\n    total = total + end - start\n[total, start, end]"
        ),
        json!([10, 4, 10])
    );
    assert_eq!(
        evaluate(
            "pairs = {left: [1, 2], right: [3, 4]}\ntotal = 0\nfor side, pair of pairs:\n    low, high = pair\n    total = total + low * high\n[total, low, high]"
        ),
        json!([14, 3, 4])
    );
    assert_eq!(
        evaluate(
            "point = {x: 1, y: 2}\nshift = {x: 10, y: 20}\n{\n    {x, y} = point\n    {\n        {x, y} = shift\n        moved = x + y\n    }\n}\n[x, y, moved]"
        ),
        json!([1, 2, 30])
    );
    assert_eq!(
        evaluate("config = {size: 4}\nclass Box:\n    {size} = config\nBox()\nsize"),
        json!(4)
    );
    assert_eq!(
        evaluate("c = false\npair = [3, 4]\nif c:\n    pass\nelse:\n    a, b = pair\n[a, b]"),
        json!([3, 4])
    );
    assert_eq!(
        evaluate("f = (p) => { {x, y} = p; return x + y }\nf({x: 1, y: 2})"),
        json!(3)
    );
    assert_eq!(
        evaluate("point = {x: 1, y: 2}\nsum = suppose:\n    {x, y} = point\n    x + y\nsum"),
        json!(3)
    );
    assert_eq!(
        failure("point = {x: 1, y: 2}\nsum = suppose:\n    {x, y} = point\n    x + y\nx"),
        (
            ErrorKind::Reference,
            "x is not defined".to_string(),
            Some(Position::new(5, 1))
        )
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "pair = [3, 4]\nready = false\nif ready: low, high = pair\nready = true\nassert: [low, high], [3, 4]\npair = [5, 6]\nassert: [low, high], [5, 6]\nready = false\npair = [7, 8]\nassert: [low, high], [5, 6]",
    );
}

#[test]
fn failed_destructuring_rolls_back() {
    assert_eq!(
        failure("{x, y} = missing"),
        (
            ErrorKind::Reference,
            "missing is not defined".to_string(),
            Some(Position::new(1, 1))
        )
    );
    assert_eq!(
        failure("q = 1\n  \nr = 2\na, b = missing"),
        (
            ErrorKind::Reference,
            "missing is not defined".to_string(),
            Some(Position::new(4, 1))
        )
    );
    let guard = "limits = [4, 50]\nlow, high = limits\nif high - low > 100: throw \"WIDE\"\nwide = [9, 300]\n";
    assert_eq!(
        failure(&format!("{guard}low, high = wide")),
        (
            ErrorKind::Thrown,
            "WIDE".to_string(),
            Some(Position::new(5, 1))
        )
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{guard}moved = \"\"\ntry: low, high = wide\ncatch error: moved = error\nassert: [moved, low, high], [\"WIDE\", 4, 50]"
        ),
    );
    assert_eq!(
        failure_in(
            &mut runtime,
            "wide = [1, 2]\nlow, high = wide\nif low > 0: throw \"LATE\""
        ),
        (
            ErrorKind::Thrown,
            "LATE".to_string(),
            Some(Position::new(3, 1))
        )
    );
    assert_eq!(read(&mut runtime, "[low, high]"), json!([4, 50]));
}

#[test]
fn while_loops_and_destructuring_compose_as_the_written_out_declarations() {
    for (destructuring, declarations) in [
        (
            "while i < 3:\n    a, b = pair\n    i = i + 1",
            "while i < 3:\n    a = pair[0]\n    b = pair[1]\n    i = i + 1",
        ),
        (
            "while i < 3: {x, y} = point; i = i + 1",
            "while i < 3: x = point.x; y = point.y; i = i + 1",
        ),
        ("while, b = pair", "while = pair[0]\nb = pair[1]"),
        ("{while, b} = p", "while = p.while\nb = p.b"),
        ("{loop: while} = p", "while = p.loop"),
        ("a, b = while", "a = while[0]\nb = while[1]"),
        (
            "total = suppose:\n    {x, y} = point\n    while x < y: x = x + 1\n    x",
            "total = suppose:\n    x = point.x\n    y = point.y\n    while x < y: x = x + 1\n    x",
        ),
    ] {
        assert_eq!(
            parse(destructuring).unwrap(),
            parse(declarations).unwrap(),
            "{destructuring}"
        );
    }

    for (destructuring, declarations, expected) in [
        (
            "pairs = [[1, 2], [3, 4], [5, 6]]\ni = 0\ntotal = 0\nwhile i < pairs.length:\n    low, high = pairs[i]\n    total = total + low * high\n    i = i + 1\n[total, low, high, i]",
            "pairs = [[1, 2], [3, 4], [5, 6]]\ni = 0\ntotal = 0\nwhile i < pairs.length:\n    low = pairs[i][0]\n    high = pairs[i][1]\n    total = total + low * high\n    i = i + 1\n[total, low, high, i]",
            json!([44, 5, 6, 3]),
        ),
        (
            "point = {x: 0, y: 10}\nsteps = 0\nwhile steps < 3: {x, y} = point; point = {x: x + 1, y: y - 1}; steps = steps + 1\n[x, y, point, steps]",
            "point = {x: 0, y: 10}\nsteps = 0\nwhile steps < 3: x = point.x; y = point.y; point = {x: x + 1, y: y - 1}; steps = steps + 1\n[x, y, point, steps]",
            json!([2, 8, {"x": 3, "y": 7}, 3]),
        ),
        (
            "bounds = [2, 5]\nlow, high = bounds\nn = low\nwhile n < high: n = n + 1\nbounds = [0, 1]\n[low, high, n]",
            "bounds = [2, 5]\nlow = bounds[0]\nhigh = bounds[1]\nn = low\nwhile n < high: n = n + 1\nbounds = [0, 1]\n[low, high, n]",
            json!([0, 1, 0]),
        ),
        (
            "while, extra = [0, 5]\nwhile while < extra: while = while + 1\n[while, extra]",
            "while = [0, 5][0]\nextra = [0, 5][1]\nwhile while < extra: while = while + 1\n[while, extra]",
            json!([5, 5]),
        ),
        (
            "p = {while: 7, b: 2}\n{while, b} = p\n{while: loop} = p\n{b: while} = p\n[while, b, loop]",
            "p = {while: 7, b: 2}\nwhile = p.while\nb = p.b\nloop = p.while\nwhile = p.b\n[while, b, loop]",
            json!([2, 2, 7]),
        ),
        (
            "while = {x: 4, y: [8, 9]}\n{x, y} = while\nfirst, second = while.y\n[x, first, second]",
            "while = {x: 4, y: [8, 9]}\nx = while.x\ny = while.y\nfirst = while.y[0]\nsecond = while.y[1]\n[x, first, second]",
            json!([4, 8, 9]),
        ),
        (
            "point = {x: 1, y: 4}\nreached = suppose:\n    {x, y} = point\n    while x < y: x = x + 1\n    x\n[reached, point.x]",
            "point = {x: 1, y: 4}\nreached = suppose:\n    x = point.x\n    y = point.y\n    while x < y: x = x + 1\n    x\n[reached, point.x]",
            json!([4, 1]),
        ),
        (
            "def count(limit):\n    n = 0\n    while n < limit:\n        n = n + 1\n    return n\nlow, high = [count(2), count(5)]\n[low, high]",
            "def count(limit):\n    n = 0\n    while n < limit:\n        n = n + 1\n    return n\nlow = [count(2), count(5)][0]\nhigh = [count(2), count(5)][1]\n[low, high]",
            json!([2, 5]),
        ),
        (
            "n = 0\ntry:\n    while n < 3:\n        n = n + 1\n        a, b = missing\ncatch e: failure = e\n[n, failure]",
            "n = 0\ntry:\n    while n < 3:\n        n = n + 1\n        a = missing[0]\n        b = missing[1]\ncatch e: failure = e\n[n, failure]",
            json!([0, "ReferenceError: missing is not defined"]),
        ),
    ] {
        assert_eq!(evaluate(destructuring), expected, "{destructuring}");
        assert_eq!(evaluate(declarations), expected, "{declarations}");
    }

    for (destructuring, declarations, expected) in [
        (
            "while true: a, b = [1, 2]",
            "while true: a = [1, 2][0]; b = [1, 2][1]",
            (
                ErrorKind::Type,
                "Loop did not terminate".to_string(),
                Some(Position::new(1, 1)),
            ),
        ),
        (
            "point = {x: 1, y: 4}\nreached = suppose:\n    {x, y} = point\n    while x < y: x = x + 1\n    x\nx",
            "point = {x: 1, y: 4}\nreached = suppose:\n    x = point.x; y = point.y\n    while x < y: x = x + 1\n    x\nx",
            (
                ErrorKind::Reference,
                "x is not defined".to_string(),
                Some(Position::new(6, 1)),
            ),
        ),
    ] {
        assert_eq!(failure(destructuring), expected, "{destructuring}");
        assert_eq!(failure(declarations), expected, "{declarations}");
    }

    for (source, message, line, column) in [
        ("while a, b = pair: pass", "Unexpected ','", 1, 10),
        ("while, b = pair: pass", "Unexpected ','", 1, 8),
        ("{x} = while x: y", "Unexpected '='", 1, 7),
        ("a, b = while ready: tick()", "Unexpected ','", 1, 4),
        ("while (a, b) = pair: pass", "Unexpected ':'", 1, 22),
        ("while lambda: a, b = pair", "Unexpected ','", 1, 18),
        (
            "{x} = [() => {\n    while n < 3: n = n + 1\n}] )",
            "Unexpected '='",
            1,
            7,
        ),
        (
            "a, b = [function() {\n    while n < 3:\n        n = n + 1\n}] +",
            "Unexpected ','",
            1,
            4,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
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
fn the_native_repl_runs_destructuring_at_once() {
    let output = repl(
        "point = {x: 3, y: 4}\n{x, y} = point\npoint.x = 8\nx\na, b = [5, 6]\nb\nlet {q} = point\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{BANNER}{{\"x\":3.0,\"y\":4.0}}\n4.0\n8.0\n8.0\n6.0\n6.0\n")
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "SyntaxError: Unexpected '=' (line 1, column 11)\n"
    );
}
