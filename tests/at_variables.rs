mod common;

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{
    BinaryOp, Expr, FunctionBody, Stmt, UnaryOp, parse, parse_expression, parse_program,
};
use nucleoid::{ErrorKind, Position, Runtime};
use regex::Regex;

const PRELUDE: &str = "a = 1\nc = true\nx = 1\nxs = [1, 2]\nfirst = 5\nlast = 6\nprevious = 7\nlambda = 8\no = {p: 2, m: (v) => v}\ndef f(v) => v\ndef pick(color) => color\nclass Light(color):\n    this.color = color\n";

const OPERANDS: [(&str, &str); 9] = [
    ("@a", "(a)"),
    ("@z", "(z)"),
    ("@a.p", "(a).p"),
    ("@xs[0]", "(xs)[0]"),
    ("@f(1)", "(f)(1)"),
    ("@a@first", "(a)@first"),
    ("@last", "(last)"),
    ("@lambda", "(lambda)"),
    ("@Light(color=RED)", "(Light)(color=RED)"),
];

const VALUE_CONTEXTS: [&str; 44] = [
    "b is {v}",
    "b is {v} + 2",
    "b is 1 + {v}",
    "b is [{v}, 1]",
    "b is [A, {v}, B]",
    "b is {{k: {v}}}",
    "b is {{k: {v}, j: B}}",
    "b is Light(color={v})",
    "b is Light({v})",
    "b is pick(color={v})",
    "b is new Light(color={v})",
    "b is {v} if c else B",
    "b is A if {v} else B",
    "b is A if c else {v}",
    "b is c ? {v} : B",
    "b is {v} ? A : B",
    "b is c ? A : {v}",
    "b is xs[{v}]",
    "b is xs[{v}:2]",
    "b is ({v})",
    "b is not {v}",
    "b is -{v}",
    "b is typeof {v}",
    "b is {v} ?? 3",
    "b is null ?? {v}",
    "b is all v in xs: v > {v}",
    "b is any v in [{v}]: v",
    "b is [v for v in [{v}]]",
    "b is sum of [{v}, 1]",
    "b is {{...o, k: {v}}}",
    "b is (x is {v})",
    "b is [x is {v}]",
    "b is {v} == 1",
    "b is {v} and c",
    "b is c or {v}",
    "b is [[{v}]]",
    "b is Light(color=[{v}])",
    "b is A if {v} is 1 else B",
    "b is {{k: c ? {v} : B}}",
    "b is f({v}) + f(2)",
    "b is [{v}, {v}]",
    "b is {v} * {v}",
    "b is xs |> depth({v})",
    "b is [{v}].length",
];

const COMPARISON_CONTEXTS: [&str; 14] = [
    "r = (x is {v})",
    "r = (x is not {v})",
    "r = (x is {v} + 1)",
    "r = (x is ({v}))",
    "r = (x is [{v}])",
    "r = f(x is {v})",
    "if x is {v}:\n    r = 1",
    "if x is {v}\n    r = 1",
    "r = [w for w in xs if w is {v}]",
    "r = xs.filter(w => w is {v})",
    "r = (x is {v}) and c",
    "assert: x is {v}, true",
    "while x is {v}:\n    x = x + 1",
    "r = (xs is [{v}, 2])",
];

const POSITIONS: [&str; 7] = [
    "{s}",
    "z0 = 0; {s}",
    "if c:\n{i}",
    "def g():\n{i}\n    return 1\ng()",
    "suppose c:\n{i}",
    "class K:\n{i}\nK()",
    "{{ {s} }}",
];

fn place(position: &str, statement: &str) -> Option<String> {
    if statement.contains('\n') && (position.contains("{{") || position.contains("; ")) {
        return None;
    }
    let indented = statement
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    Some(
        position
            .replace("{i}", &indented)
            .replace("{{ ", "{ ")
            .replace(" }}", " }")
            .replace("{s}", statement),
    )
}

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn tree(source: &str) -> String {
    format!("{:?}", statements(source))
}

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn word(name: &str) -> Expr {
    Expr::Unary {
        operator: UnaryOp::Word,
        operand: Box::new(Expr::Identifier(name.to_string())),
    }
}

fn assign(target: &str, value: Expr) -> Stmt {
    Stmt::Assign {
        target: name(target),
        value,
    }
}

fn outcome(source: &str) -> String {
    let mut runtime = Runtime::new();
    let result = match runtime.run(source) {
        Ok(value) => format!(
            "value {}",
            runtime
                .serialize_json(&value)
                .unwrap_or_else(|error| error.to_string())
        ),
        Err(error) => format!("{:?} {}", error.kind(), error.message()),
    };
    let objects = Regex::new(r"obj:\d+").unwrap();
    format!(
        "{}; assertions {}; failures {}",
        objects.replace_all(&result, "obj"),
        runtime.assertions_run(),
        runtime.take_assertions().len()
    )
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn assert_syntax_error(source: &str, message: &str, line: usize, column: usize) {
    assert_eq!(
        syntax_error(source),
        (message.to_string(), Position::new(line, column)),
        "{source}"
    );
}

fn reference_error(source: &str) -> (String, Position) {
    let error = Runtime::new().run(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference, "{source}");
    (error.message(), error.position().unwrap())
}

fn fresh(source: &str, expected: &str) {
    let parsed = statements(source);
    let assertions = common::assertions::protected_assertions(&parsed).len();
    let case = common::Case {
        title: "at variable regression".to_string(),
        source: source.to_string(),
        expected: Some(expected.to_string()),
    };
    common::check_with_count(&mut Runtime::new(), &case, assertions)
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

const UNEXPECTED_AT: &str = "Unexpected character '@'";
const SELECTOR: &str = "Expected a non-negative integer, 'first', 'last', or 'previous' after '@'";

#[test]
fn the_users_examples_read_a_variable_and_raise_a_reference_error() {
    fresh("a is 1\nb is @a + 2\nassert(b, 3)\nb", "3");
    let mut runtime = Runtime::new();
    runtime.run("a is 1\nb is @a + 2").unwrap();
    assert_eq!(runtime.run("b").unwrap().to_string(), "3");
    assert_eq!(
        reference_error("b is @c + 3"),
        ("c is not defined".to_string(), Position::new(1, 1))
    );
    assert_eq!(reference_error("b is @c + 3"), reference_error("b = c + 3"));
}

#[test]
fn at_variables_build_the_tree_of_their_parenthesized_names_in_both_regions() {
    let mut compared = 0;
    for context in VALUE_CONTEXTS.iter().chain(COMPARISON_CONTEXTS.iter()) {
        for (at, parenthesized) in OPERANDS {
            if at == "@last" && context.starts_with("b is {v}") {
                continue;
            }
            for position in POSITIONS {
                let (Some(source), Some(expected)) = (
                    place(position, &context.replace("{v}", at)),
                    place(position, &context.replace("{v}", parenthesized)),
                ) else {
                    continue;
                };
                let source = source.replace("{{", "{").replace("}}", "}");
                let expected = expected.replace("{{", "{").replace("}}", "}");
                assert_eq!(tree(&source), tree(&expected), "{source}");
                assert_eq!(
                    outcome(&format!("{PRELUDE}{source}")),
                    outcome(&format!("{PRELUDE}{expected}")),
                    "{source}"
                );
                compared += 1;
            }
        }
    }
    assert_eq!(compared, 3544);
}

#[test]
fn at_variables_are_never_words() {
    assert_eq!(statements("b is @A"), vec![assign("b", name("A"))]);
    assert_eq!(statements("b is A"), vec![assign("b", word("A"))]);
    assert_eq!(
        statements("b is @A if c else B"),
        vec![assign(
            "b",
            Expr::Conditional {
                condition: Box::new(name("c")),
                consequent: Box::new(name("A")),
                alternate: Box::new(word("B")),
            }
        )]
    );
    assert_eq!(
        statements("b is [@A, B]"),
        vec![assign("b", Expr::List(vec![name("A"), word("B")]))]
    );
    assert_eq!(
        statements("b is {k: @A, j: B}"),
        vec![assign(
            "b",
            Expr::ObjectLiteral(vec![
                ("k".to_string(), name("A")),
                ("j".to_string(), word("B"))
            ])
        )]
    );
    let compare = |source: &str| match statements(source).into_iter().next() {
        Some(Stmt::If { condition, .. }) => condition,
        other => panic!("{source}: {other:?}"),
    };
    assert_eq!(
        compare("if x is @C:\n    r = 1"),
        Expr::Binary {
            operator: BinaryOp::Equal,
            left: Box::new(name("x")),
            right: Box::new(name("C")),
        }
    );
    assert_eq!(
        compare("if x is not @C:\n    r = 1"),
        Expr::Binary {
            operator: BinaryOp::NotEqual,
            left: Box::new(name("x")),
            right: Box::new(name("C")),
        }
    );
    assert_eq!(
        compare("if x is C:\n    r = 1"),
        Expr::Binary {
            operator: BinaryOp::Is,
            left: Box::new(name("x")),
            right: Box::new(name("C")),
        }
    );
}

#[test]
fn an_at_variable_raises_where_a_word_reads_its_text() {
    for (marked, bare, text) in [
        ("x is @A", "x is A", "\"A\""),
        ("x is @A if c else B", "x is A if c else B", "\"A\""),
        ("x is [@A]", "x is [A]", "[\"A\"]"),
        ("x is {k: @A}", "x is {k: A}", "{\"k\":\"A\"}"),
        ("x is pick(color=@A)", "x is pick(color=A)", "\"A\""),
        ("x is (y is @A)", "x is (y is A)", "false"),
    ] {
        let prelude = "c = true\ny = 1\ndef pick(color) => color\n";
        assert_eq!(
            reference_error(&format!("{prelude}{marked}")),
            ("A is not defined".to_string(), Position::new(4, 1)),
            "{marked}"
        );
        assert_eq!(
            outcome(&format!("{prelude}{bare}\nx")),
            format!("value {text}; assertions 0; failures 0"),
            "{bare}"
        );
        assert_eq!(
            outcome(&format!("{prelude}A = 4\n{marked}\nx")),
            outcome(&format!("{prelude}A = 4\n{bare}\nx")),
            "{marked}"
        );
    }
    assert_eq!(
        reference_error("x = 2\nif x is @C:\n    r = 1"),
        ("C is not defined".to_string(), Position::new(2, 1))
    );
    fresh(
        "x = 2\nC = 2\nr = 0\nif x is @C:\n    r = 1\nassert(r, 1)\nr",
        "1",
    );
}

#[test]
fn standing_values_follow_their_at_variables_and_class_rules_require_them() {
    fresh(
        "a = 1\nb is @a + 2\nassert(b, 3)\na = 5\nassert(b, 7)\nb",
        "7",
    );
    let mut runtime = Runtime::new();
    runtime.run("a = 1\nb is [@a, A]").unwrap();
    let marked = runtime.run("b |> why").unwrap();
    let marked = runtime.serialize_json(&marked).unwrap();
    let mut parenthesized = Runtime::new();
    parenthesized.run("a = 1\nb is [(a), A]").unwrap();
    let expected = parenthesized.run("b |> why").unwrap();
    assert_eq!(marked, parenthesized.serialize_json(&expected).unwrap());
    let light = "class Light(color):\n    this.color = color\n";
    assert_eq!(
        reference_error(&format!("{light}$Light.glow is @GLOW")),
        ("GLOW is not defined".to_string(), Position::new(3, 1))
    );
    fresh(
        &format!(
            "{light}$Light.glow is GLOW\nfirst = Light(\"RED\")\nassert(first.glow, \"GLOW\")\nfirst.glow"
        ),
        "\"GLOW\"",
    );
    fresh(
        &format!(
            "{light}GLOW = \"on\"\n$Light.glow is @GLOW\nfirst = Light(\"RED\")\nassert(first.glow, \"on\")\nGLOW = \"off\"\nassert(first.glow, \"off\")\nfirst.glow"
        ),
        "\"off\"",
    );
    fresh(
        &format!(
            "{light}RED = \"red\"\n$Light.lit = false\nif $Light.color is @RED:\n    $Light.lit = true\nfirst = Light(\"red\")\nsecond = Light(\"RED\")\nassert([first.lit, second.lit], [true, false])\nfirst.lit"
        ),
        "true",
    );
}

#[test]
fn postfix_chains_and_version_selectors_follow_at_variables() {
    fresh(
        "o = {p: {q: 3}, m: (v) => v * 2}\nxs = [4, 5]\nn = 1\nn = 2\nb is [@o.p.q, @xs[1], @o.m(4), @n@first, @n@last]\nassert(b, [3, 5, 8, 1, 2])\nb",
        "[3,5,8,1,2]",
    );
    fresh(
        "class Light(color):\n    this.color is color\nRED = \"red\"\nb is @Light(color=RED)\nd is @Light(color=BLUE)\nassert([b.color, d.color], [\"red\", \"BLUE\"])\nb.color",
        "\"red\"",
    );
    assert_eq!(
        tree("b is @a@first"),
        "[Assign { target: Identifier(\"b\"), value: Version { reference: Identifier(\"a\"), selector: First } }]"
    );
}

#[test]
fn prefix_first_last_and_previous_are_variables_except_right_after_a_statement_is() {
    fresh(
        "first = 1\nlast = 2\nprevious = 3\nb is [@first, @last, @previous]\nc is (@first) + @last\nassert(b, [1, 2, 3])\nassert(c, 3)\nc",
        "3",
    );
    for selector in ["first", "last", "previous"] {
        let source = format!("b is @{selector}");
        let statements = statements(&source);
        assert_eq!(statements.len(), 2, "{source}");
        assert_eq!(statements[0], Stmt::Expression(name("b")), "{source}");
        assert!(
            matches!(&statements[1], Stmt::Expression(Expr::Version { reference, .. }) if **reference == name("is")),
            "{source}: {statements:?}"
        );
    }
    assert_eq!(tree("b is @0"), tree("b\nis@0"));
    assert_eq!(tree("b is @ first"), tree("b\nis@first"));
    assert_eq!(tree("b is @first + 1"), tree("b\nis@first + 1"));
    assert_eq!(
        statements("b is (@first)"),
        vec![assign("b", name("first"))]
    );
}

#[test]
fn a_leading_at_outside_the_regions_keeps_its_accepted_diagnostic() {
    for (source, column) in [
        ("b = @c", 5),
        ("@a", 1),
        ("value = @", 9),
        ("r = f(@a)", 7),
        ("r = (x == @a)", 11),
        ("r = (@a is x)", 6),
        ("r = (x is 1) == @a", 17),
        ("if @a:\n    r = 1", 4),
        ("throw @a", 7),
        ("delete @a", 8),
        ("x is () => @a", 12),
        ("x is (v) => @a", 13),
        ("x is lambda: @a", 14),
        ("x is function() { return @a }", 26),
        ("x is y = @a", 10),
        ("x is (y = @a)", 11),
        ("x is o.m(k=@a)", 12),
        ("x is [v for v in xs if @a]", 24),
        ("r = (x is (v) => @a)", 18),
        ("r = (x is c ? @a : 1)", 15),
        ("r = f(x is Light(color=@a))", 24),
        ("def g() => @a", 12),
    ] {
        assert_syntax_error(source, UNEXPECTED_AT, 1, column);
    }
    assert_syntax_error("def g():\n    return @a\ng()", UNEXPECTED_AT, 2, 12);
    assert_syntax_error("while @a:\n    r = 1", SELECTOR, 1, 8);
    let (message, position) = syntax_error("x is `${@a}`");
    assert_eq!(message, UNEXPECTED_AT);
    assert_eq!(position.line, 1);
}

#[test]
fn targets_and_comprehension_elements_reject_at_variables_with_the_accepted_errors() {
    for (source, message, column) in [
        ("x is @y = 1", SELECTOR, 7),
        ("x is @y += 1", SELECTOR, 7),
        ("x is @y.p = 1", SELECTOR, 7),
        ("x is @a + @b = 1", SELECTOR, 7),
        ("x is @a if c else 2 = 3", SELECTOR, 7),
        ("x is (@y) = 1", UNEXPECTED_AT, 7),
        ("x is [@y] = 1", UNEXPECTED_AT, 7),
        ("x is [@y] += [1]", UNEXPECTED_AT, 7),
        ("x is xs[@i] = 1", UNEXPECTED_AT, 9),
        ("x is c ? 1 : @y = 2", UNEXPECTED_AT, 14),
        ("x is [@a for v in xs]", UNEXPECTED_AT, 7),
        ("x is [[@a] for v in xs]", UNEXPECTED_AT, 8),
        ("x is [@a.p for v in xs]", UNEXPECTED_AT, 7),
        ("r = f(x is @a = 1)", UNEXPECTED_AT, 12),
        ("x is (@a = 1)", UNEXPECTED_AT, 7),
        ("x is Light(@color=1)", UNEXPECTED_AT, 12),
    ] {
        assert_syntax_error(source, message, 1, column);
    }
}

#[test]
fn spaces_comments_and_other_tokens_after_at_keep_their_readings() {
    for (source, message, column) in [
        ("x is [@ a]", UNEXPECTED_AT, 7),
        ("x is [@\ta]", UNEXPECTED_AT, 7),
        ("x is [@/* c */a]", UNEXPECTED_AT, 7),
        ("x is [@\na]", UNEXPECTED_AT, 7),
        ("x is [@1]", UNEXPECTED_AT, 7),
        ("x is [@\"s\"]", UNEXPECTED_AT, 7),
        ("x is [@(a)]", UNEXPECTED_AT, 7),
        ("x is [@$Light]", UNEXPECTED_AT, 7),
        ("x is [@true]", UNEXPECTED_AT, 7),
        ("x is [@this]", UNEXPECTED_AT, 7),
        ("x is [@not a]", UNEXPECTED_AT, 7),
        ("x is [@@a]", UNEXPECTED_AT, 7),
        ("x is [@]", UNEXPECTED_AT, 7),
        ("x is @ a", SELECTOR, 8),
        ("r = (x is @ a)", UNEXPECTED_AT, 11),
        ("x is [@1a]", SELECTOR, 8),
    ] {
        assert_syntax_error(source, message, 1, column);
    }
}

#[test]
fn comparisons_accept_at_variables_only_in_their_right_operand() {
    fresh(
        "x = 2\nC = 2\nD = 3\nxs = [1, 2, 3]\nassert(x is @C, true)\nassert(x is not @D, true)\nassert(x is @C + 0, true)\nassert([w for w in xs if w is @D], [3])\nassert(xs.filter(w => w is not @C), [1, 3])\nassert(x is [@C][0], true)\nr = (x is (@C))\nr",
        "true",
    );
    fresh(
        "x = 1\nC = 3\nwhile x is not @C:\n    x = x + 1\nassert(x, 3)\nx",
        "3",
    );
    fresh(
        "x = 2\nC = 2\nr = 0\nif x is @C\n    r = 1\nassert(r, 1)\nr",
        "1",
    );
    assert_syntax_error("r = (@a is x)", UNEXPECTED_AT, 1, 6);
}

#[test]
fn header_colons_after_at_lambda_belong_to_while_and_suppose() {
    fresh(
        "lambda = 3\nx = 1\nwhile x is not @lambda:\n    x = x + 1\nassert(x, 3)\nx",
        "3",
    );
    fresh(
        "lambda = 2\nx = 2\nr = 0\nsuppose x is @lambda:\n    r = 1\nassert(r, 0)\nr",
        "0",
    );
    assert_eq!(
        tree("while x is @lambda:\n    x = x + 1"),
        tree("while x is (lambda):\n    x = x + 1")
    );
    assert_eq!(
        syntax_error("x = 1\nwhile x is lambda:\n    x = x + 1"),
        ("Unexpected end of line".to_string(), Position::new(3, 5))
    );
}

#[test]
fn at_variables_render_as_their_parenthesized_names_and_read_back() {
    for (source, rendering) in [
        ("b is @a", "b=a"),
        ("b is @a + 2", "b=a+2"),
        ("b is c ? @a : B", "b is c?(a):B"),
        ("b is @a if c else B", "b is c?(a):B"),
        ("b is [@A, B]", "b is [(A),B]"),
        ("b is {k: @A, j: B}", "b is {\"k\":(A),\"j\":B}"),
        (
            "b is Light(color=@A, size=B)",
            "b is Light(color=(A),size=B)",
        ),
        ("b is (@a)", "b=a"),
    ] {
        let statements = statements(source);
        assert_eq!(generate_all(&statements), rendering, "{source}");
        assert_eq!(parse(rendering).unwrap(), statements, "{source}");
    }
    let mut runtime = Runtime::new();
    runtime.run("a = 1\n{ total is [@a, B] }").unwrap();
    let graph = format!(
        "{:?}",
        runtime
            .graph
            .nodes()
            .map(|node| node.key.to_string())
            .collect::<Vec<_>>()
    );
    assert!(graph.contains("block(total is [(a),B])"), "{graph}");
}

#[test]
fn every_parse_entry_point_and_statement_position_reads_at_variables() {
    let program = parse_program("a = 1\nb is @a").unwrap();
    assert_eq!(
        program.positions,
        vec![Position::new(1, 1), Position::new(2, 1)]
    );
    assert_eq!(program.statements[1], assign("b", name("a")));
    assert!(Runtime::check("b is [@a, A]").is_ok());
    let Expr::Function(function) = parse_expression("() => { b is @a }").unwrap() else {
        panic!("not a function");
    };
    let FunctionBody::Block(body) = &function.body else {
        panic!("not a block body");
    };
    assert_eq!(body, &vec![assign("b", name("a"))]);
    assert_syntax_error("@a + 1", UNEXPECTED_AT, 1, 1);
    assert!(parse_expression("@a + 1").is_err());
    let mut runtime = Runtime::new();
    runtime.run("a = 1\nb is @a + 1").unwrap();
    runtime.clear();
    runtime.run("a = 4\nb is @a + 1").unwrap();
    assert_eq!(runtime.run("b").unwrap().to_string(), "5");
    fresh(
        "a = 2\nif true:\n    b is @a\n{ c is @a }\ndef g():\n    d is @a\n    return d\nassert([b, g()], [2, 2])\nb",
        "2",
    );
}

#[test]
fn at_variables_combine_with_comments_and_the_word_features() {
    fresh(
        "a = 2 // two\nb is @a /* marked */ + 1 # three\nRED = \"red\"\nlevel is [@RED, GREEN] if true else LOW\nassert([b, level], [3, [\"red\", \"GREEN\"]])\nb",
        "3",
    );
}

#[test]
fn the_documented_examples_run_as_written() {
    fresh(
        "a is 1\nb is @a + 2\nassert(b, 3)\na = 5\nassert(b, 7)\nb",
        "7",
    );
    fresh(
        "RED = \"red\"\ncolors is [@RED, GREEN]\nassert(colors, [\"red\", \"GREEN\"])\nlight is \"GREEN\"\nGO = \"GREEN\"\nstate is GO if light is @GO else STOP\nassert(state, \"GREEN\")\nstate",
        "\"GREEN\"",
    );
    let reference = "a is 1\nb is @a + 2\nRED = \"red\"\ncolors is [@RED, GREEN]\n";
    fresh(
        &format!("{reference}assert([b, colors], [3, [\"red\", \"GREEN\"]])\nb"),
        "3",
    );
    fresh(
        "a is 1\nb is @a + 2\nRED = \"red\"\ncolors is [@RED, GREEN]\nassert([b, colors], [3, [\"red\", \"GREEN\"]])\ncolors[0]",
        "\"red\"",
    );
    assert_eq!(
        reference_error("colors is [@RED, GREEN]"),
        ("RED is not defined".to_string(), Position::new(1, 1))
    );
    assert_eq!(
        reference_error("b is @c"),
        ("c is not defined".to_string(), Position::new(1, 1))
    );
    fresh("b is c\nassert(b, \"c\")\nb", "\"c\"");
}
