//! NUC 57: `->` as an optional spelling of `=>`, and contextual Python-style
//! `lambda` expressions lowering to the existing anonymous arrow function.

mod common;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, FunctionBody, Stmt, parse, parse_expression};
use nucleoid::{Error, ErrorKind, Position, Runtime, Value};

/// Both sources parse to the same complete AST and canonical source.
fn equivalent(alias: &str, canonical: &str) -> Vec<Stmt> {
    let expected = parse(canonical).unwrap_or_else(|error| panic!("{canonical}: {error}"));
    let actual = parse(alias).unwrap_or_else(|error| panic!("{alias}: {error}"));
    assert_eq!(actual, expected, "{alias}");
    assert_eq!(generate_all(&actual), generate_all(&expected), "{alias}");
    actual
}

/// Both sources parse identically, or fail with the same complete error.
fn same_outcome(alias: &str, canonical: &str) -> Result<Vec<Stmt>, Error> {
    let actual = parse(alias);
    match (&actual, parse(canonical)) {
        (Ok(actual), Ok(expected)) => assert_eq!(actual, &expected, "{alias}"),
        (Err(actual), Err(expected)) => {
            assert_eq!(actual.kind(), expected.kind(), "{alias}");
            assert_eq!(actual.message(), expected.message(), "{alias}");
            assert_eq!(actual.position(), expected.position(), "{alias}");
        }
        (actual, expected) => panic!("{alias}: {actual:?} differs from {expected:?}"),
    }
    actual
}

/// The source parses, or fails at the same position with the same message,
/// exactly as when `lambda` is spelled as another same-length ordinary name.
fn reads_lambda_as_a_name(source: &str) {
    let renamed = source.replace("lambda", "lambdo");
    match (parse(source), parse(&renamed)) {
        (Ok(actual), Ok(expected)) => assert_eq!(
            format!("{actual:?}"),
            format!("{expected:?}").replace("\"lambdo\"", "\"lambda\""),
            "{source}"
        ),
        (Err(actual), Err(expected)) => {
            assert_eq!(actual.kind(), expected.kind(), "{source}");
            assert_eq!(actual.message(), expected.message(), "{source}");
            assert_eq!(actual.position(), expected.position(), "{source}");
        }
        (actual, expected) => panic!("{source}: {actual:?} differs from {expected:?}"),
    }
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

/// Every accepted arrow position, written with the canonical `=>`.
const FAT_ARROW_PROGRAMS: &[&str] = &[
    "callback = value => value * 2",
    "callback = (value) => value * 2",
    "callback = () => 1",
    "callback = (left, right) => left + right",
    "callback = (left: int, right) => left + right",
    "callback = (left, right = 2) => left + right",
    "callback = (value,) => value",
    "callback = value => { return value * 2 }",
    "callback = (value) => {\n    total = value\n    return total\n}",
    "found = items.find(element => element == 1)",
    "found = items.find(element => { return element == 2 })",
    "list = Result.filter(r => r.score > low).filter(r => r.score < high)",
    "curried = left => right => left + right",
    "record = { double: value => value * 2, label: 1 }",
    "record = { double: value => value * 2 label: 1 }",
    "callbacks = [value => value, (left, right) => left]",
    "matches = value => /a-b/.test(value)",
    "def shift(value) => value + 1",
    "def shift(value: int,) => value + 1",
    "def shift() => 1",
    "def scale(value = 2) => value * 2",
    "def shift value => value + 1",
    "def shift value: int => value + 1",
    "def area width, height => width * height",
    "def area width: int, height => width * height",
    "def area width: int, height: int => width * height",
    "def area width height => width * height",
    "def area width height factor => width * height * factor",
    "def shift value =>\n    value + 1",
    "def shift(value) =>\n    value + 1",
    "def area width height =>\n    width * height",
    "def shift value => # header\n    value + 1",
    "def shift value =>\n    {total: value}",
    "def area(\n    width,\n    height\n) => width * height",
    "class Gauge:\n    def init amount => this.amount = amount\n    def reading() => self.amount\n    def scaled factor => self.amount * factor",
    "class Meter:\n    def adjust(delta) => self.reading += delta",
    "def outer value => value => value",
    "def f() => int => 1",
    "def f amount => int => amount",
    "if items.some(item => item > 1):\n    found = true",
    "if items.some(item => item > 1)\n    found = true",
    "for item of items.filter(item => item > 1):\n    total = item",
    "text = `${items.map(item => item * 2)}`",
    "first = items[items.indexOf(items.find(item => item > 1))]",
    "assert(items.map(item => item * 2), [2, 4])",
    "assert: items.map(item => item * 2), [2, 4]",
    "caller = Student(rule=value => value)",
    "explanation = why items.find(item => item > 1)",
];

#[test]
fn thin_arrows_parse_exactly_like_fat_arrows_in_every_accepted_position() {
    for canonical in FAT_ARROW_PROGRAMS {
        let alias = canonical.replace("=>", "->");
        assert_ne!(&alias, canonical);
        equivalent(&alias, canonical);
        // Each arrow is independent, so the spellings may be mixed.
        equivalent(&canonical.replacen("=>", "->", 1), canonical);
    }
}

#[test]
fn thin_arrow_programs_run_like_fat_arrow_programs() {
    for (canonical, expected) in [
        (
            "offset = 1\nitems = [1, 2, 3]\nlarge = items.filter(item => item > offset)\nassert(large, [2, 3])\noffset = 2\nassert(large, [3])\nlarge",
            Some("[3]"),
        ),
        (
            "def area width height =>\n    width * height\nscale = 2\ndef scaled width height => area(width, height) * scale\nresult = scaled(3, 4)\nassert(result, 24)\nscale = 3\nassert(result, 36)\nresult",
            Some("36"),
        ),
        (
            "class Gauge:\n    def init amount => this.amount = amount\n    def scaled factor => self.amount * factor\ngauge = Gauge(4)\nassert(gauge.scaled(3), 12)\ngauge.amount = 5\nassert(gauge.scaled(3), 15)",
            None,
        ),
        (
            "element = 38.5\nsamples = [38.8, 39.1]\nfound = samples.find(element => element == 38.8)\nassert(found, 38.8)\nassert(element, 38.5)",
            None,
        ),
    ] {
        for source in [canonical.to_string(), canonical.replace("=>", "->")] {
            let case = common::Case {
                title: "thin arrow equivalence".to_string(),
                source,
                expected: expected.map(str::to_string),
            };
            common::check(&case).unwrap_or_else(|reason| panic!("{}\n{reason}", case.source));
        }
    }
}

#[test]
fn separated_or_quoted_arrow_text_is_not_an_arrow() {
    let tokens = |source: &str| -> Vec<Token> {
        tokenize(source)
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect()
    };
    assert_eq!(
        tokens("value - > other"),
        [
            Token::Identifier("value".into()),
            Token::Minus,
            Token::Greater,
            Token::Identifier("other".into()),
            Token::Eof,
        ]
    );
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("[\"a\", \"b\"].join(\"->\")", "a->b"),
        ("'x->y'", "x->y"),
        ("`${1}->${2}`", "1->2"),
        ("/a->b/.test(\"a->b\")", "true"),
        ("value = 3 # value -> 4\nvalue", "3"),
        ("flag = 5 > -1\nflag", "true"),
    ] {
        assert_eq!(
            runtime.run(source).unwrap().to_string(),
            expected,
            "{source}"
        );
    }
    // A separated arrow keeps the accepted diagnostic of a lone `>` operand.
    assert_eq!(
        syntax_error("callback = value - > value"),
        ("Unexpected '>'".to_string(), Position::new(1, 22))
    );
}

/// The return-annotation probes of earlier bare-header regressions, with
/// their original adjacent `->`. Those regressions now spell them `- >`.
const RETURN_ANNOTATION_LOOKALIKES: &[&str] = &[
    "def f amount -> int:\n    return amount",
    "def f() -> int:\n    return 1",
    "def f left, right -> int:\n    return left",
    "def f left, right -> int => left",
    "def f amount -> int => amount",
    "def f() -> int => 1",
    "def f amount: int -> int:\n    return amount",
    "def f() -> int: return 1",
    "def f left: int, right -> int:\n    return left",
    "def f left: int, right: int -> int:\n    return left",
    "def f left: int, right -> int => left",
    "def f left: int, right: int -> int => left",
    "def f amount: int -> int => amount",
    "def f left right -> int:\n    pass",
    "def f left right -> int => left",
    "def f(x) -> int { return x }",
];

#[test]
fn return_annotation_lookalikes_are_ordinary_arrows() {
    let mut valid = 0;
    for source in RETURN_ANNOTATION_LOOKALIKES {
        if same_outcome(source, &source.replace("->", "=>")).is_ok() {
            valid += 1;
        }
        let separated = source.replace("->", "- >");
        assert_eq!(
            Runtime::check(&separated).unwrap_err().kind(),
            ErrorKind::Syntax,
            "{separated}"
        );
    }
    // Only the forms whose `=>` spelling was already valid now parse.
    assert_eq!(valid, 8);
    for (source, message, line, column) in [
        ("def f() -> int:\n    return 1", "Unexpected ':'", 1, 15),
        (
            "def f amount -> int:\n    return amount",
            "Unexpected ':'",
            1,
            20,
        ),
        ("def f() -> int: return 1", "Unexpected ':'", 1, 17),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    runtime.run("def f() -> int => 1").unwrap();
    assert_eq!(runtime.run("f()(5)").unwrap(), Value::Number(1.0));
}

#[test]
fn diagnostics_name_the_arrow_as_written() {
    for (source, message, line, column) in [
        ("def f -> 1", "Expected '(' but found '->'", 1, 7),
        ("def f => 1", "Expected '(' but found '=>'", 1, 7),
        ("x = ->", "Unexpected '->'", 1, 7),
        ("x = 1->2", "Unexpected '->'", 1, 8),
        ("x-->y", "Unexpected '->'", 1, 5),
        ("def f(x ->", "Expected ')' but found '->'", 1, 9),
        ("def f x ->", "Unexpected end of input", 1, 11),
        ("def f x ->\n", "Unexpected end of line", 2, 1),
        ("def f x ->\nx", "Unexpected end of line", 2, 1),
        ("def f x ->;\n    x", "Unexpected ';'", 1, 11),
        ("f = (a, b) ->", "Unexpected end of input", 1, 14),
        ("f = x -> { return x", "Unterminated block", 1, 20),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn lambdas_lower_to_the_parenthesized_anonymous_arrow() {
    for (alias, canonical) in [
        ("f = lambda: 1", "f = () => (1)"),
        ("f = lambda value: value * 2", "f = (value) => (value * 2)"),
        (
            "f = lambda left, right: left + right",
            "f = (left, right) => (left + right)",
        ),
        (
            "f = lambda a, b, c: [a, b, c]",
            "f = (a, b, c) => ([a, b, c])",
        ),
        ("f = lambda a,b:a", "f = (a, b) => (a)"),
        ("f = lambda: {x: 1, y: 2}", "f = () => ({x: 1, y: 2})"),
        ("f = lambda x: {x}", "f = (x) => ({x})"),
        ("f = lambda: lambda: 1", "f = () => (() => (1))"),
        (
            "f = lambda a: lambda b: a + b",
            "f = (a) => ((b) => (a + b))",
        ),
        ("f = lambda x: x = 2", "f = (x) => (x = 2)"),
        ("f = lambda x: x.name", "f = (x) => (x.name)"),
        ("f = lambda x: not x", "f = (x) => (not x)"),
        ("f = lambda: model |> why", "f = () => (model |> why)"),
        ("f = lambda x: x -> x", "f = (x) => (x => x)"),
        ("f = lambda: int", "f = () => (int)"),
        ("f = lambda a, b: int", "f = (a, b) => (int)"),
        // Only a single `assert` and its colon start a statement.
        (
            "f = lambda assert, other: other",
            "f = (assert, other) => (other)",
        ),
        (
            "f = lambda first, assert: first",
            "f = (first, assert) => (first)",
        ),
        (
            "f = lambda Assert: Assert * 2",
            "f = (Assert) => (Assert * 2)",
        ),
        ("lambda: x + 1", "() => (x + 1)"),
        ("x = 1 + lambda: 2", "x = 1 + () => (2)"),
        (
            "mapped = items.map(lambda x: x * 2)",
            "mapped = items.map((x) => (x * 2))",
        ),
        (
            "mapped = items.map(lambda x: x)",
            "mapped = items.map((x) => (x))",
        ),
        (
            "sum = items.reduce(lambda total, x: total + x, 0)",
            "sum = items.reduce((total, x) => (total + x), 0)",
        ),
        (
            "list = Result.filter(lambda r: r.score > low)",
            "list = Result.filter((r) => (r.score > low))",
        ),
        (
            "pairs = [lambda: 1, lambda x: x, lambda a, b: a]",
            "pairs = [() => (1), (x) => (x), (a, b) => (a)]",
        ),
        ("value = (lambda x: x)(3)", "value = ((x) => (x))(3)"),
        ("value = (lambda: 1)()", "value = (() => (1))()"),
        (
            "record = { make: lambda: 1 }",
            "record = { make: () => (1) }",
        ),
        (
            "record = { add: lambda a, b: a + b, scale: 2 }",
            "record = { add: (a, b) => (a + b), scale: 2 }",
        ),
        (
            "record = { make: lambda: 1 next: 2 }",
            "record = { make: () => (1) next: 2 }",
        ),
        (
            "record = { make: (lambda x: x) }",
            "record = { make: ((x) => (x)) }",
        ),
        ("{ f = lambda x: y }", "{ f = (x) => (y) }"),
        (
            "def make() => lambda x: x * 2",
            "def make() => (x) => (x * 2)",
        ),
        (
            "def make():\n    return lambda x: x * 2",
            "def make():\n    return (x) => (x * 2)",
        ),
        (
            "if (lambda: ready)():\n    pass",
            "if (() => (ready))():\n    pass",
        ),
        (
            "for item of items.filter(lambda x: x > 1):\n    pass",
            "for item of items.filter((x) => (x > 1)):\n    pass",
        ),
        ("first = items[f(lambda: 0)]", "first = items[f(() => (0))]"),
        (
            "part = values[lambda: f(lambda: 1)]",
            "part = values[lambda: f(() => (1))]",
        ),
        ("part = values[1:lambda: 2]", "part = values[1:() => (2)]"),
        (
            "text = `${items.map(lambda x: x * 2)}`",
            "text = `${items.map((x) => (x * 2))}`",
        ),
        (
            "assert(items.map(lambda x: x * 2), [2])",
            "assert(items.map((x) => (x * 2)), [2])",
        ),
        (
            "assert: items.map(lambda x: x * 2), [2]",
            "assert: items.map((x) => (x * 2)), [2]",
        ),
        ("assert: lambda: 1, 2", "assert: () => (1), 2"),
        (
            "caller = Student(rule=lambda value: value)",
            "caller = Student(rule=(value) => (value))",
        ),
    ] {
        equivalent(alias, canonical);
    }
    let Expr::Function(function) = parse_expression("lambda left, right: left").unwrap() else {
        panic!("a lambda is an anonymous function");
    };
    assert_eq!(function.name, None);
    assert!(matches!(function.body, FunctionBody::Expression(_)));
    let names: Vec<_> = function
        .parameters
        .iter()
        .map(|parameter| {
            (
                parameter.name.as_str(),
                &parameter.type_name,
                &parameter.default,
            )
        })
        .collect();
    assert_eq!(names, [("left", &None, &None), ("right", &None, &None)]);
    let many = (0..1024)
        .map(|index| format!("p{index}"))
        .collect::<Vec<_>>();
    equivalent(
        &format!("f = lambda {}: p0", many.join(", ")),
        &format!("f = ({}) => (p0)", many.join(", ")),
    );
}

/// Each program parses exactly as if `lambda` were any other name. Apart from
/// the thin arrows in a few parameter-name rows, these are accepted syntax.
const ORDINARY_LAMBDA_NAMES: &[&str] = &[
    "lambda = 1",
    "lambda.x = 1",
    "x = lambda.x",
    "x = lambda(1)",
    "x = lambda[0]",
    "x = lambda@0",
    "lambda += 1",
    "lambda ||= 1",
    "keyed = { lambda: 1 }",
    "short = { lambda }",
    "both = { first: 1, lambda }",
    "member = record.lambda",
    "made = Student(lambda=1)",
    "def lambda(x) => x",
    "def f(lambda) => lambda",
    "def f(lambda: int) => lambda",
    "def f lambda => lambda",
    "def f lambda -> lambda",
    "def f lambda: int => lambda",
    "def f lambda:\n    return lambda",
    "callback = (lambda: int) => lambda",
    "callback = lambda => lambda",
    "callback = lambda -> lambda",
    "callback = (lambda, other) -> lambda",
    "x: lambda",
    "lambda: int",
    "lambda: int; other = 1",
    "class lambda",
    "class Holder:\n    lambda: int",
    "class Holder:\n    this.copy = lambda x: int",
    "for lambda of items:\n    pass",
    "for lambda:\n    pass",
    "for item of lambda:\n    pass",
    "if lambda:\n    pass",
    "if lambda: x = 1",
    "if lambda\n    pass",
    "if not lambda and ready:\n    pass",
    "if ready:\n    pass\nelse lambda:\n    pass",
    "if ready:\n    pass\nelif lambda:\n    pass",
    "if ready:\n    pass\nelse if lambda:\n    pass",
    "if ready:\n    pass\nelse lambda\n    pass",
    "slice = items[lambda:]",
    "slice = items[lambda:2]",
    "slice = items[lambda: 1]",
    "slice = items[lambda + 1:]",
    "slice = items[:lambda]",
    "item = items[lambda]",
    "record = { first: lambda next: 2 }",
    "record = { first: lambda next: 2, other: 3 }",
    "record = { first: 1 + lambda next: 2 }",
    "record = { first: value => lambda next: 2 }",
    "record = { first: lambda lambda: 2 }",
    "text = `${lambda: 1}`",
    "text = `${lambda x: x}`",
    "text = `${lambda a, b: a}`",
    "f = lambda x: int",
    "f = lambda x: x",
    "f = lambda x: int; g = 1",
    "f = lambda lambda: lambda",
    "def pick():\n    return lambda x: int",
    "if ready:\n    f = lambda x: int",
    "try:\n    pass\ncatch lambda:\n    pass",
    "assert(lambda, 1)",
    "assert: lambda, 1",
    "explanation = why lambda",
    "chain = lambda |> why",
    "delete lambda",
    "kind = typeof lambda",
    "flag = not lambda",
    "f = lambda x y: x",
    // Statement loops accept NUC 45's `assert:` statement directly after the
    // expression that `lambda` ends, in every statement context.
    "lambda assert: x, 1",
    "x = lambda assert: x, 1",
    "x = lambda assert : x, 1",
    "x = lambda assert:x,1",
    "x = lambda assert: x, 1; y = 2",
    "x = lambda assert: x, 1\ny = 2",
    "x = lambda assert: x, 1 assert: x, 1",
    "x = lambda assert: lambda, 1",
    "x = lambda assert: x, lambda",
    "x += lambda assert: x, 1",
    "x = y = lambda assert: x, 1",
    "total = total + lambda assert: total, 2",
    "x = not lambda assert: x, true",
    "x = why lambda assert: x, 1",
    "x = new lambda assert: x, 1",
    "x = items[1:] + lambda assert: x, 1",
    "assert: 1, lambda assert: 1, 1",
    "throw lambda assert: 1, 1",
    "delete lambda assert: 1, 1",
    "def pick():\n    return lambda assert: 1, 1",
    "def pick(): return lambda assert: 1, 1",
    "def pick(): { return lambda assert: 1, 1 }",
    "def pick() => lambda assert: 1, 1",
    "def pick value => lambda assert: value, 1",
    "def pick left right => lambda assert: left, 1",
    "callback = value => lambda assert: value, 1",
    "callback = (value) => { value = lambda assert: value, 1 }",
    "callback = function(value) { return lambda assert: value, 1 }",
    "class Holder:\n    this.copy = lambda assert: this.copy, 1",
    "class Holder:\n    lambda assert: 1, 1",
    "class Holder: this.copy = lambda assert: this.copy, 1",
    "if ready: x = lambda assert: x, 1",
    "if ready:\n    x = lambda assert: x, 1",
    "if ready\n    x = lambda assert: x, 1",
    "if ready:\n    pass\nelse: x = lambda assert: x, 1",
    "for item of items: x = lambda assert: x, 1",
    "for items:\n    x = lambda assert: x, 1",
    "try:\n    x = lambda assert: x, 1\ncatch error:\n    pass",
    "{ x = lambda assert: x, 1 }",
    "record = { make: value => { return lambda assert: value, 1 } }",
    "f = lambda assert: int",
    "f = lambda assert: assert",
    "f = lambda assert: int; g = lambda assert: 1, 1",
];

#[test]
fn lambda_keeps_every_accepted_name_reading() {
    for source in ORDINARY_LAMBDA_NAMES {
        parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        reads_lambda_as_a_name(source);
    }
}

#[test]
fn accepted_lambda_names_keep_their_runtime_meaning() {
    for (source, expected) in [
        (
            "lambda = 3\nclass Gauge:\n    this.f = lambda x: int\ng = Gauge()\ng.f",
            "3",
        ),
        (
            "lambda = 4\ndef pick():\n    return lambda x: int\npick()",
            "4",
        ),
        ("lambda = 5\nlabel = `${lambda: 9}`\nlabel", "5"),
        ("lambda = [1, 2, 3]\nlambda[lambda[0]:]", "2,3"),
        (
            "lambda = 7\nb = 2\no = { a: lambda b: 3 }\n[o.a, o.b]",
            "7,3",
        ),
        ("lambda = true\nif lambda: r = 1\nr", "1"),
        ("lambda = 2\ndouble = lambda -> lambda * 2\ndouble(5)", "10"),
    ] {
        assert_eq!(
            Runtime::new().run(source).unwrap().to_string(),
            expected,
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    runtime.run("lambda = 1").unwrap();
    let error = runtime.run("f = lambda x: int").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "Missing definition");
}

#[test]
fn juxtaposed_assertion_statements_keep_their_accepted_results() {
    // Each program returns the same value at the accepted revision, and its
    // colon assertion after `lambda` runs and passes.
    for (source, expected) in [
        ("lambda = 1\nx = lambda assert: x, 1\nx", "1"),
        ("lambda = 5\ny = lambda assert: lambda, 5\ny", "5"),
        (
            "lambda = 2\ntotal = 0\ntotal = total + lambda assert: total, 2\ntotal",
            "2",
        ),
        (
            "lambda = 3\nclass Box:\n    this.copy = lambda assert: this.copy, 3\nbox = Box()\nbox.copy",
            "3",
        ),
        (
            "lambda = 4\nx = lambda assert: lambda, 4\nassert(x, 4)\nx",
            "4",
        ),
        ("lambda = 1\nx = lambda\nassert: x, 1\nx", "1"),
        ("lambda = 6\nif lambda: x = lambda assert: x, 6\nx", "6"),
        (
            "lambda = 2\ndef twice value => lambda * value\nx = twice(3) + lambda assert: x, 8\nlambda = 3\nassert(x, 12)\nx",
            "12",
        ),
        (
            "lambda = 1\nx = lambda assert: x, 1; y = lambda assert: y, 1\n[x, y]",
            "[1,1]",
        ),
    ] {
        let case = common::Case {
            title: "juxtaposed assertion statement".to_string(),
            source: source.to_string(),
            expected: Some(expected.to_string()),
        };
        reads_lambda_as_a_name(source);
        common::check(&case).unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    }
    // The statement after `return` is unreachable, exactly as accepted.
    let mut runtime = Runtime::new();
    let value = runtime
        .run("lambda = 7\ndef f():\n    return lambda assert: 1, 1\nf()")
        .unwrap();
    assert_eq!(value, Value::Number(7.0));
    assert_eq!(runtime.assertions_run(), 0);
    // A failing juxtaposed assertion is still reported as an assertion.
    let mut runtime = Runtime::new();
    runtime.run("lambda = 1\nx = lambda assert: x, 2").unwrap();
    assert_eq!(runtime.assertions_run(), 1);
    assert_eq!(runtime.take_assertions().len(), 1);
}

#[test]
fn lambdas_run_reactively_with_arrow_scoping() {
    for source in [
        "scale = 3\nitems = [1, 2, 3]\ntripled = items.map(lambda item: item * scale)\nassert(tripled, [3, 6, 9])\nscale = 2\nassert(tripled, [2, 4, 6])\nitems.push(4)\nassert(tripled, [2, 4, 6, 8])",
        "class Reading(level: int):\n    this.level = level\nReading(4); Reading(9); Reading(15)\nthreshold = 5\nhigh = Reading.filter(lambda reading: reading.level > threshold)\nassert(high.length, 2)\nthreshold = 10\nassert(high.length, 1)",
        "element = 38.5\nsamples = [38.8, 39.1]\nfound = samples.find(lambda element: element == 38.8)\nassert(found, 38.8)\nassert(element, 38.5)",
        "def apply(callback, value) => callback(value)\nresult = apply(lambda value: value + 1, 2)\nassert(result, 3)",
        "pair = lambda left, right: {left, right}\nassert(pair(1, 2).right, 2)\nassert((lambda: 7)(), 7)",
    ] {
        let case = common::Case {
            title: "lambda behavior".to_string(),
            source: source.to_string(),
            expected: None,
        };
        common::check(&case).unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    }
    // Lambdas keep the anonymous arrow's existing noncapturing scope.
    for (lambda, arrow) in [
        (
            "make = lambda a: lambda b: a + b\nmake(1)(2)",
            "make = (a) => ((b) => (a + b))\nmake(1)(2)",
        ),
        (
            "make = lambda: lambda: missing\nmake()()",
            "make = () => (() => (missing))\nmake()()",
        ),
    ] {
        let actual = Runtime::new().run(lambda).unwrap_err();
        let expected = Runtime::new().run(arrow).unwrap_err();
        assert_eq!(actual.kind(), expected.kind(), "{lambda}");
        assert_eq!(actual.message(), expected.message(), "{lambda}");
    }
}

#[test]
fn malformed_lambdas_keep_their_accepted_readings_and_errors() {
    for source in [
        "f = lambda x=1: x",
        "f = lambda (x): x",
        "f = lambda x,: x",
        "f = lambda *x: x",
        "f = lambda self: self",
        "f = lambda x y: x",
        "f = lambda this: 1",
        "f = lambda\n: 1",
        "f = lambda x\n: 1",
        "f = [lambda x y: x]",
        "f = items[lambda x: x]",
        "f = items[lambda a, b: a]",
        "f = lambda assert:",
        "f = lambda assert: 1, 2, 3",
        "f = lambda assert: x, 1 }",
        "f = [lambda assert: 1, 1]",
        "items.map(lambda assert: assert)",
    ] {
        reads_lambda_as_a_name(source);
    }
    for (source, message, line, column) in [
        ("f = lambda x:", "Unexpected end of input", 1, 14),
        ("f = lambda:", "Unexpected end of input", 1, 12),
        ("f = lambda a, b:", "Unexpected end of input", 1, 17),
        ("f = lambda:\n    1", "Unexpected end of line", 2, 5),
        ("f = lambda x: )", "Unexpected ')'", 1, 16),
        ("f = lambda x: int, y: int: x", "Unexpected ','", 1, 20),
        (
            "f = lambda x: { return x }",
            "Expected a key but found keyword 'return'",
            1,
            24,
        ),
        ("f = lambda x=1: x", "Unexpected ':'", 1, 17),
        ("f = lambda x,: x", "Unexpected ','", 1, 14),
        ("f = lambda assert:", "Unexpected end of input", 1, 19),
        (
            "f = lambda assert: 1, 2, 3",
            "Assertion statements expect exactly two arguments",
            1,
            24,
        ),
        (
            "items.map(lambda assert: assert)",
            "Expected ')' but found identifier 'assert'",
            1,
            18,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn nested_lambdas_keep_the_expression_nesting_bound() {
    let deep = format!("f = {}1", "lambda: ".repeat(80));
    assert_eq!(syntax_error(&deep).0, "Expressions are nested too deeply");
    let arrows = format!("f = {}1", "() -> ".repeat(80));
    assert_eq!(syntax_error(&arrows).0, "Expressions are nested too deeply");
    let shallow = format!("f = {}1", "lambda: ".repeat(20));
    assert!(Runtime::check(&shallow).is_ok());
}

fn repl(source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
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
fn native_stdin_buffers_thin_named_headers_and_runs_lambdas() {
    let output = repl(
        ".json on\ndef shift value -> # header\n    value + 1\n\nshift(2)\nadd = lambda left, right: left + right\nadd(3, 4)\n[1, 2].map(item -> item * 10)\n\"->\"",
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in ["3.0", "7.0", "[10.0,20.0]", "\"->\""] {
        assert!(stdout.lines().any(|actual| actual == line), "{stdout}");
    }
    let output = repl("def shift value ->\nvalue\n\nshift(2)");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("SyntaxError: Unexpected end of line (line 2, column 1)"),
        "{output:?}"
    );
}
