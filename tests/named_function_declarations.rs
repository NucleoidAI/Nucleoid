mod common;

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Expr, FunctionBody, Stmt, parse, parse_program};
use nucleoid::{ErrorKind, Position, Runtime};

const SPEC_TITLES: [&str; 6] = [
    "Nucleoid declares named functions with the function keyword",
    "Nucleoid keeps function declarations equivalent to def declarations",
    "Nucleoid declares methods and init with the function keyword",
    "Nucleoid declares functions wherever a statement may start",
    "Nucleoid keeps function declarations live without hoisting",
    "Nucleoid rolls back failed function declarations",
];
const REFERENCE_TITLE: &str = "Nucleoid accepts function as the named declaration spelling";
const README_TITLE: &str = "Function declarations keep JavaScript-style functions live";
const EXAMPLES_TITLE: &str = "Function declarations keep a class's grading rule live";

fn declares_named_function(source: &str) -> bool {
    let tokens = tokenize(source).unwrap();
    tokens.windows(3).any(|window| {
        matches!(
            window,
            [previous, function, name]
                if previous.token != Token::Dot
                    && function.token == Token::Keyword(Keyword::Function)
                    && matches!(name.token, Token::Identifier(_))
        )
    }) || matches!(
        tokens.as_slice(),
        [function, name, ..]
            if function.token == Token::Keyword(Keyword::Function)
                && matches!(name.token, Token::Identifier(_))
    )
}

fn equivalent(function: &str, def: &str) {
    let function_ast = parse(function).unwrap_or_else(|error| panic!("{function}: {error}"));
    let def_ast = parse(def).unwrap_or_else(|error| panic!("{def}: {error}"));
    assert_eq!(function_ast, def_ast, "{function}");
    assert_eq!(
        generate_all(&function_ast),
        generate_all(&def_ast),
        "{function}"
    );
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

fn graph_shape(runtime: &Runtime) -> Vec<String> {
    runtime
        .graph
        .nodes()
        .map(|node| {
            let reads: Vec<String> = node.dependencies.iter().map(ToString::to_string).collect();
            let updates: Vec<String> = node.dependents.iter().map(ToString::to_string).collect();
            format!("{} [{}] {reads:?} {updates:?}", node.key, node.kind)
        })
        .collect()
}

#[test]
fn function_declarations_build_the_def_declarations() {
    for (function, def) in [
        (
            "function double(amount) { return amount * 2 }",
            "def double(amount):\n    return amount * 2",
        ),
        (
            "function double(amount) {\n    return amount * 2\n}",
            "def double(amount): return amount * 2",
        ),
        ("function now() { return 1 }", "def now() => 1"),
        (
            "function total(a, b) { sum = a + b; return sum }",
            "def total(a, b):\n    sum = a + b\n    return sum",
        ),
        (
            "function total(a, b) {\n    sum = a + b\n\n    # result\n    return sum\n}",
            "def total(a, b):\n    sum = a + b\n    return sum",
        ),
        (
            "function total(a, b) { sum = a + b return sum }",
            "def total(a, b):\n    sum = a + b\n    return sum",
        ),
        (
            "function label(name: str, count: int = 1, ...tags) { return name }",
            "def label(name: str, count: int = 1, ...tags):\n    return name",
        ),
        (
            "function pair(\n    left,\n    right,\n) { return [left, right] }",
            "def pair(\n    left,\n    right,\n):\n    return [left, right]",
        ),
        (
            "function sign(n) {\n    if n > 0:\n        return 1\n    else:\n        return -1\n}",
            "def sign(n):\n    if n > 0:\n        return 1\n    else:\n        return -1",
        ),
        (
            "function count(items) {\n    total = 0\n    for item of items:\n        total = total + 1\n    return total\n}",
            "def count(items):\n    total = 0\n    for item of items:\n        total = total + 1\n    return total",
        ),
        (
            "function guard(a) { { inner = a } return inner }",
            "def guard(a):\n    { inner = a }\n    return inner",
        ),
        (
            "function show(a) { return { size: a } }",
            "def show(a):\n    return { size: a }",
        ),
        (
            "function f(a) { return a } f(2)",
            "def f(a):\n    return a\nf(2)",
        ),
        (
            "function f(a) { return a }; g = f(1)",
            "def f(a):\n    return a\ng = f(1)",
        ),
        (
            "x = 1 function f(a) { return a }",
            "x = 1\ndef f(a):\n    return a",
        ),
        (
            "function f(a) { return a } function g(b) { return f(b) }",
            "def f(a):\n    return a\ndef g(b):\n    return f(b)",
        ),
        (
            "function outer() { function inner(a) { return a } return inner(1) }",
            "def outer():\n    def inner(a):\n        return a\n    return inner(1)",
        ),
        (
            "def outer():\n    function inner(a) { return a }\n    return inner",
            "def outer():\n    def inner(a):\n        return a\n    return inner",
        ),
        (
            "callback = function (x) { function inner(a) { return a } return inner(x) }",
            "callback = function (x) {\n    def inner(a):\n        return a\n    return inner(x)\n}",
        ),
        (
            "step = (x) => { function inner(a) { return a } return inner(x) }",
            "step = (x) => {\n    def inner(a):\n        return a\n    return inner(x)\n}",
        ),
    ] {
        equivalent(function, def);
    }
}

#[test]
fn function_declarations_start_wherever_a_statement_may_start() {
    for (function, def) in [
        (
            "if ready:\n    function f(a) { return a }\nnext = 1",
            "if ready:\n    def f(a):\n        return a\nnext = 1",
        ),
        (
            "if ready: function f(a) { return a }",
            "if ready:\n    def f(a):\n        return a",
        ),
        (
            "if ready:\n    pass\nelse:\n    function f(a) { return a }",
            "if ready:\n    pass\nelse:\n    def f(a):\n        return a",
        ),
        (
            "for item of items:\n    function f(a) { return a }",
            "for item of items:\n    def f(a):\n        return a",
        ),
        (
            "while n < 3:\n    function f(a) { return a }\n    n = n + 1",
            "while n < 3:\n    def f(a):\n        return a\n    n = n + 1",
        ),
        (
            "try:\n    function f(a) { return a }\ncatch error:\n    function f(a) { return 0 }",
            "try:\n    def f(a):\n        return a\ncatch error:\n    def f(a):\n        return 0",
        ),
        (
            "{ function f(a) { return a } }",
            "{\n    def f(a):\n        return a\n}",
        ),
        (
            "{ x = 1; function f(a) { return a }; y = 2 }",
            "{\n    x = 1\n    def f(a):\n        return a\n    y = 2\n}",
        ),
        (
            "class A: function f() { return 1 }",
            "class A: def f(): return 1",
        ),
    ] {
        equivalent(function, def);
    }
}

#[test]
fn class_bodies_declare_methods_and_adopt_init_as_def_does() {
    for (function, def) in [
        (
            "class Shape:\n    function area() { return 0 }",
            "class Shape:\n    def area():\n        return 0",
        ),
        (
            "class Shape:\n    function init(sides) { this.sides = sides }\n    function describe() { return this.sides }",
            "class Shape:\n    def init(sides):\n        this.sides = sides\n    def describe():\n        return this.sides",
        ),
        (
            "class Point:\n    function init(this.x, self.y = 0) { this.sum = x + y }",
            "class Point:\n    def init(this.x, self.y = 0):\n        this.sum = x + y",
        ),
        (
            "class Point(x, y):\n    function init(x, y) {\n        this.x = x\n        this.y = y\n    }",
            "class Point(x, y):\n    def init(x, y):\n        this.x = x\n        this.y = y",
        ),
        (
            "class Mixed:\n    x: int\n    def first() => 1\n    function second() { return 2 }\n    this.ready = true",
            "class Mixed:\n    x: int\n    def first() => 1\n    def second():\n        return 2\n    this.ready = true",
        ),
        (
            "class Dog: Animal\n    function speak() { return \"woof\" }",
            "class Dog: Animal\n    def speak():\n        return \"woof\"",
        ),
        (
            "class Dog extends Animal:\n    function speak() { return \"woof\" }",
            "class Dog: Animal\n    def speak():\n        return \"woof\"",
        ),
    ] {
        equivalent(function, def);
    }

    let statements = parse("class Point:\n    function init(this.x) {}").unwrap();
    let [Stmt::Class(class)] = statements.as_slice() else {
        panic!("expected one class");
    };
    let [init] = class.methods.as_slice() else {
        panic!("expected one method");
    };
    assert_eq!(init.name.as_deref(), Some("init"));
    assert_eq!(init.body, FunctionBody::Block(parse("this.x = x").unwrap()));
}

#[test]
fn empty_bodies_declare_functions_that_return_null() {
    let statements = parse("function noop() {}").unwrap();
    let [Stmt::Function(function)] = statements.as_slice() else {
        panic!("expected one function");
    };
    assert_eq!(function.name.as_deref(), Some("noop"));
    assert!(function.parameters.is_empty());
    assert_eq!(function.body, FunctionBody::Block(Vec::new()));

    let mut runtime = Runtime::new();
    let value = runtime
        .run("function noop(a) {}\nfunction later(a) {\n\n}\n[noop(1), later(2)]")
        .unwrap();
    assert_eq!(runtime.serialize_json(&value).unwrap(), "[null,null]");
}

#[test]
fn malformed_declarations_report_exact_syntax_errors() {
    for (source, message, line, column) in [
        ("function f(a) => a", "Expected '{' but found '=>'", 1, 15),
        ("function f(a) -> a", "Expected '{' but found '->'", 1, 15),
        (
            "function f(a):\n    return a",
            "Expected '{' but found ':'",
            1,
            14,
        ),
        (
            "function f(a): number { return a }",
            "Expected '{' but found ':'",
            1,
            14,
        ),
        (
            "function f(a)\n{ return a }",
            "Expected '{' but found end of line",
            1,
            13,
        ),
        (
            "function f(a)\n    { return a }",
            "Expected '{' but found end of line",
            1,
            13,
        ),
        (
            "function f(a); { return a }",
            "Expected '{' but found ';'",
            1,
            14,
        ),
        (
            "function f(a)",
            "Expected '{' but found end of input",
            1,
            14,
        ),
        (
            "function f a { return a }",
            "Expected '(' but found identifier 'a'",
            1,
            12,
        ),
        (
            "function f { return 1 }",
            "Expected '(' but found '{'",
            1,
            12,
        ),
        (
            "function f<T>(a) { return a }",
            "Expected '(' but found '<'",
            1,
            11,
        ),
        ("function f() { return 1", "Unterminated block", 1, 24),
        (
            "function f(",
            "Expected a parameter but found end of input",
            1,
            12,
        ),
        (
            "function f($a) { return $a }",
            "Expected a parameter but found '$a'",
            1,
            14,
        ),
        (
            "function f(a: ) { return a }",
            "Expected a type but found ')'",
            1,
            17,
        ),
        (
            "function f(a = []) { return a }",
            "Expected a scalar literal parameter default",
            1,
            16,
        ),
        (
            "function f(...rest, a) { return a }",
            "Rest parameter must be last",
            1,
            21,
        ),
        (
            "function f(this.x) { return x }",
            "Expected a parameter but found keyword 'this'",
            1,
            16,
        ),
        (
            "function init(this.x) { return 1 }",
            "Expected a parameter but found keyword 'this'",
            1,
            19,
        ),
        (
            "class A:\n    function speak(this.x) { return 1 }",
            "Expected a parameter but found keyword 'this'",
            2,
            24,
        ),
        ("function f(a) { return a +) }", "Unexpected ')'", 1, 29),
        (
            "function f(a) { return a } else: pass",
            "Unexpected keyword 'else'",
            1,
            32,
        ),
        ("function f() {} = 1", "Unexpected '='", 1, 19),
        ("while function f() {}:\n    pass", "Unexpected ':'", 1, 22),
        ("suppose function f() {}: n", "Unexpected ':'", 1, 26),
        (
            "function amount:\n    return amount",
            "Expected '(' but found ':'",
            1,
            16,
        ),
        (
            "function amount: int:\n    return amount",
            "Expected '(' but found ':'",
            1,
            16,
        ),
    ] {
        syntax_error(source, message, line, column);
    }
}

#[test]
fn named_function_expressions_keep_their_accepted_errors() {
    for (source, column) in [
        ("g = function f(a) { return a }", 14),
        ("return function f() { return 1 }", 17),
        ("values = [function f() {}]", 20),
        ("record = {key: function f() {}}", 25),
        ("call(function f() {})", 15),
        ("x = (function f() {})", 15),
        ("if function f() {}:\n    pass", 13),
        ("total = 1 + function f() {}", 22),
        ("made = new function f() {}", 21),
        ("kind = typeof function f() {}", 24),
        ("flag = !function f() {}", 18),
        ("throw function f() {}", 16),
        ("delete function f() {}", 17),
        ("pick = ready ? function f() {} : 0", 25),
        ("label = `${function f() {}}`", 21),
        ("for x of function f() {}:\n    pass", 19),
        ("x = function f", 14),
    ] {
        syntax_error(source, "Expected '(' but found identifier 'f'", 1, column);
    }
    syntax_error("function", "Expected '(' but found end of input", 1, 9);
    syntax_error(
        "function function() {}",
        "Expected '(' but found keyword 'function'",
        1,
        10,
    );
    syntax_error(
        "function 5(a) { return a }",
        "Expected '(' but found number 5",
        1,
        10,
    );
}

#[test]
fn tentative_parses_take_declarations_inside_function_bodies() {
    for (source, expected) in [
        (
            "a, b = (() => { function f(x) { return x }; return [1, 2] })()\n[a, b]",
            "[1.0,2.0]",
        ),
        (
            "{a, b} = (() => { function f(x) { return x }; return {a: 1, b: 2} })()\n[a, b]",
            "[1.0,2.0]",
        ),
        (
            "n = 0\nwhile (() => { function f() {}; return n < 2 })():\n    n = n + 1\nn",
            "2.0",
        ),
        (
            "v = 1 if (() => { function f() {}; return true })() else 2\nv",
            "1.0",
        ),
        (
            "t = `${(() => { function f(x) { return x }; return f(7) })()}`\nt",
            "\"7\"",
        ),
        (
            "value = 1\nw = (value is (() => { function f() {}; return 1 })())\nw",
            "true",
        ),
        (
            "n = 1\nk = suppose n = (() => { function f() {}; return 5 })(): n\nk",
            "5.0",
        ),
        (
            "ready = true\nif (() => { function f() {}; return ready })():\n    hit = 1\nhit",
            "1.0",
        ),
    ] {
        let mut runtime = Runtime::new();
        let value = runtime
            .run(source)
            .unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(
            runtime.serialize_json(&value).unwrap(),
            expected,
            "{source}"
        );
    }
    syntax_error(
        "a, b = (() => { function f(x) { return x }; return [1, 2] }",
        "Unexpected ','",
        1,
        4,
    );
}

#[test]
fn anonymous_literals_and_function_names_keep_their_meanings() {
    let statements = parse("function (a) { return a }").unwrap();
    let [Stmt::Expression(Expr::Function(function))] = statements.as_slice() else {
        panic!("expected an anonymous function expression");
    };
    assert!(function.name.is_none());
    for (source, expected) in [
        ("double = function (a) { return a * 2 }\ndouble(4)", "8.0"),
        ("x = 1 function (a) { return a }\nx", "1.0"),
        ("record = {\"function\": 3}\nrecord.function", "3.0"),
        (
            "record = {\"function\": 4}\n[record].map(item => item.function)",
            "[4.0]",
        ),
    ] {
        let mut runtime = Runtime::new();
        let value = runtime
            .run(source)
            .unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(
            runtime.serialize_json(&value).unwrap(),
            expected,
            "{source}"
        );
    }
    let error = Runtime::new().run("x.function f").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "x is not defined");
}

#[test]
fn function_and_def_runtimes_keep_identical_results_graphs_and_explanations() {
    let mut def = Runtime::new();
    let mut function = Runtime::new();
    for (def_source, function_source) in [
        (
            "def fee(amount):\n    return amount * 0.1",
            "function fee(amount) { return amount * 0.1 }",
        ),
        ("base = 200", "base = 200"),
        ("charge = fee(base)", "charge = fee(base)"),
        ("charge", "charge"),
        (
            "def fee(amount):\n    return amount * 0.2",
            "function fee(amount) {\n    return amount * 0.2\n}",
        ),
        ("charge", "charge"),
        ("frozen = fee.value", "frozen = fee.value"),
        (
            "def fee(amount):\n    return amount * 0.3",
            "function fee(amount) { return amount * 0.3 }",
        ),
        ("[charge, frozen(100)]", "[charge, frozen(100)]"),
        ("base = 300", "base = 300"),
        ("why charge", "why charge"),
        ("affects base", "affects base"),
        (
            "def label(name: str, count = 1, ...tags):\n    return [name, count, tags]",
            "function label(name: str, count = 1, ...tags) { return [name, count, tags] }",
        ),
        (
            "[label(\"a\"), label(\"b\", 2, \"x\", \"y\")]",
            "[label(\"a\"), label(\"b\", 2, \"x\", \"y\")]",
        ),
        (
            "class Item:\n    def init(this.name, this.qty):\n        this.ready = true\n    def doubled():\n        return this.qty * 2",
            "class Item:\n    function init(this.name, this.qty) { this.ready = true }\n    function doubled() { return this.qty * 2 }",
        ),
        ("pen = Item(\"pen\", 3)", "pen = Item(\"pen\", 3)"),
        (
            "$Item.cost = fee($Item.qty * 100)",
            "$Item.cost = fee($Item.qty * 100)",
        ),
        (
            "[pen.name, pen.ready, pen.doubled(), pen.cost]",
            "[pen.name, pen.ready, pen.doubled(), pen.cost]",
        ),
        ("pen.qty = 5", "pen.qty = 5"),
        ("[pen.doubled(), pen.cost]", "[pen.doubled(), pen.cost]"),
        ("model |> why", "model |> why"),
        (
            "try:\n    def fee(amount):\n        return 0\n    throw \"UNDO\"\ncatch error:\n    reason = error",
            "try:\n    function fee(amount) { return 0 }\n    throw \"UNDO\"\ncatch error:\n    reason = error",
        ),
        ("[reason, charge, fee(10)]", "[reason, charge, fee(10)]"),
        ("fee = 4", "fee = 4"),
        ("[fee, charge]", "[fee, charge]"),
        (
            "def fee(amount):\n    return amount",
            "function fee(amount) { return amount }",
        ),
        ("[charge, typeof fee]", "[charge, typeof fee]"),
    ] {
        match (def.run(def_source), function.run(function_source)) {
            (Ok(def_value), Ok(function_value)) => assert_eq!(
                def.serialize_json(&def_value).unwrap(),
                function.serialize_json(&function_value).unwrap(),
                "{function_source}"
            ),
            (Err(def_error), Err(function_error)) => {
                assert_eq!(def_error, function_error, "{function_source}");
                assert_eq!(def_error.kind(), ErrorKind::Type, "{function_source}");
            }
            (def_result, function_result) => panic!(
                "different results for {function_source}: {def_result:?}, {function_result:?}"
            ),
        }
        assert_eq!(
            graph_shape(&def),
            graph_shape(&function),
            "{function_source}"
        );
    }
    function
        .run("assert(charge, 300)\nassert(frozen(10), 2)\nassert(pen.cost, 500)\nassert(reason, \"UNDO\")")
        .unwrap();
    assert_eq!(function.assertions_run(), 4);
    let failures = function.take_assertions();
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
fn declarations_are_not_hoisted_and_failed_programs_install_nothing() {
    for source in [
        "early = total(1, 2)\nfunction total(a, b) { return a + b }",
        "early = total(1, 2)\ndef total(a, b):\n    return a + b",
    ] {
        let mut runtime = Runtime::new();
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference, "{source}");
        assert_eq!(error.message(), "total is not defined", "{source}");
        assert_eq!(error.position(), Some(Position::new(1, 1)), "{source}");
        let error = runtime.run("total").unwrap_err();
        assert_eq!(error.message(), "total is not defined", "{source}");
    }

    let mut runtime = Runtime::new();
    runtime.run("saved = 7").unwrap();
    for invalid in [
        "function f(a) => a",
        "function f(a) { return a",
        "function f(a): number { return a }",
    ] {
        let error = runtime
            .run(&format!(
                "saved = 3\nfunction keep(a) {{ return a }}\n{invalid}"
            ))
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{invalid}");
        assert_eq!(runtime.run("saved").unwrap().to_string(), "7");
        assert_eq!(
            runtime.run("keep").unwrap_err().message(),
            "keep is not defined"
        );
    }
    let error = runtime
        .run("function keep(a) { return a }\nsaved = 3\nthrow \"UNDO\"")
        .unwrap_err();
    assert_eq!(
        error.thrown_value().map(ToString::to_string).as_deref(),
        Some("UNDO")
    );
    assert_eq!(runtime.run("saved").unwrap().to_string(), "7");
    assert_eq!(
        runtime.run("keep").unwrap_err().message(),
        "keep is not defined"
    );
}

#[test]
fn juxtaposed_declarations_keep_their_statement_positions() {
    let program = parse_program("function f(a) { return a } f(2)\nnext = missing").unwrap();
    assert_eq!(
        program.positions,
        [
            Position::new(1, 1),
            Position::new(1, 28),
            Position::new(2, 1)
        ]
    );
    let error = Runtime::new()
        .run("function f(a) { return a } f(2)\nnext = missing")
        .unwrap_err();
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(2, 1)));

    let error = Runtime::new()
        .run("export function f(a) { return a }")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "export is not defined");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn canonical_rendering_writes_the_def_spelling() {
    assert_eq!(
        generate_all(&parse("function f(a) { return a }").unwrap()),
        "def f"
    );
    let rendered = generate_all(&parse("function f(a = 1, ...rest) { return [a, rest] }").unwrap());
    assert_eq!(
        rendered,
        generate_all(&parse("def f(a = 1, ...rest):\n    return [a, rest]").unwrap())
    );
    assert!(rendered.starts_with("def f(a=1,...rest):\n"), "{rendered}");
    assert!(!rendered.contains("function"), "{rendered}");
    let rendered = generate_all(&parse("class Box:\n    function init(size = 1) {}").unwrap());
    assert!(rendered.contains("def init(size=1):\n"), "{rendered}");
    assert!(!rendered.contains("function"), "{rendered}");
}

#[test]
fn function_declarations_obey_the_nesting_limits_of_def_declarations() {
    let braced = |depth: usize, body: &str| {
        format!(
            "{}{body}{}",
            "function f() { ".repeat(depth),
            " }".repeat(depth)
        )
    };
    let indented = |depth: usize, body: &str| {
        let mut source = String::new();
        for level in 0..depth {
            source.push_str(&"    ".repeat(level));
            source.push_str("def f():\n");
        }
        source.push_str(&"    ".repeat(depth));
        source.push_str(body);
        source
    };
    for depth in [62, 63, 64, 65] {
        let function = Runtime::check(&braced(depth, "return 1"))
            .map_err(|error| (error.kind(), error.message().to_string()));
        let def = Runtime::check(&indented(depth, "return 1"))
            .map_err(|error| (error.kind(), error.message().to_string()));
        assert_eq!(function, def, "{depth}");
    }
    assert!(Runtime::check(&braced(62, "return 1")).is_ok());
    assert_eq!(
        Runtime::check(&braced(63, "return 1"))
            .unwrap_err()
            .message(),
        "Expressions are nested too deeply"
    );
    assert!(Runtime::check(&braced(64, "")).is_ok());
    for depth in [65, 4096] {
        let error = Runtime::check(&braced(depth, "")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), "Statements are nested too deeply");
    }
}

#[test]
fn cli_runs_named_function_declarations_in_files() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("named-function-declarations.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().next(), Some("[12.0,\"large\"]"));
    assert!(stdout.contains("area [function]"), "{stdout}");
    assert!(stdout.contains("size [function]"), "{stdout}");
}

#[test]
fn only_the_appended_cases_declare_named_functions_and_run_every_assertion() {
    let documents = [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ];
    let appended: Vec<&str> = SPEC_TITLES
        .into_iter()
        .chain([REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE])
        .collect();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flatten() {
        cases += 1;
        let new = appended.contains(&case.title.as_str());
        assert_eq!(declares_named_function(&case.source), new, "{}", case.title);
        if new {
            let count =
                common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            common::check_with_assertions(case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assertions += count;
        }
    }
    for title in &appended {
        let matching = documents
            .iter()
            .flatten()
            .filter(|case| case.title == *title)
            .count();
        assert_eq!(matching, 1, "{title}");
    }
    assert_eq!(assertions, 35);
    assert_eq!(cases, 1041);
    assert!(declares_named_function("x = 1 function f() {}"));
    assert!(!declares_named_function("record.function f"));
    assert!(!declares_named_function("f = function (a) { return a }"));
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Named function declarations\n\n```\n")
        .expect("the language reference documents named declarations");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 89](nuc-0089.md)"));
    assert_eq!(case.expected.as_deref(), Some("9"));
}

#[test]
fn repl_buffers_named_declarations_until_their_braces_close() {
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
        .write_all(
            b".json on\nfunction scale(amount) {\n\n    factor = 3\n    return amount * factor\n}\nscale(4)\nfunction twice(amount) { return amount * 2 }\ntwice(5)\nif true:\n    function half(amount) {\n        return amount / 2\n    }\n\nhalf(8)\n.exit\n",
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let results: Vec<&str> = stdout.lines().skip(4).collect();
    assert_eq!(
        results,
        ["null", "12.0", "null", "10.0", "null", "4.0"],
        "{stdout}"
    );
}
