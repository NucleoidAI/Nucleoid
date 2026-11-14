mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime};

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid reads receiver properties in class level rules",
    "Nucleoid reads receiver properties through nested targets",
    "Nucleoid reads receiver properties through this and self",
    "Nucleoid reads receiver properties in compound assignments",
    "Nucleoid reads receiver properties inside lambdas of class level rules",
    "Nucleoid reads receiver properties in conditionals objects and templates",
    "Nucleoid continues receiver reads with members indexes and calls",
    "Nucleoid rolls back failed receiver assignments",
];
const REFERENCE_TITLE: &str = "Nucleoid reads properties of the assigned object with a leading dot";
const README_TITLE: &str = "A leading dot reads a property of the assigned object";
const EXAMPLES_TITLE: &str = "Leading dots keep an order line's rules about its own fields";

const OPERATORS: [&str; 10] = ["=", ":=", "+=", "-=", "*=", "**=", "/=", "%=", "&&=", "||="];

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

fn begins_operand(token: &Token) -> bool {
    matches!(
        token,
        Token::Assign
            | Token::ColonAssign
            | Token::PlusAssign
            | Token::MinusAssign
            | Token::StarAssign
            | Token::StarStarAssign
            | Token::SlashAssign
            | Token::PercentAssign
            | Token::AndAndAssign
            | Token::OrOrAssign
            | Token::Plus
            | Token::Minus
            | Token::Star
            | Token::StarStar
            | Token::Slash
            | Token::Percent
            | Token::Bang
            | Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual
            | Token::Equal
            | Token::NotEqual
            | Token::StrictEqual
            | Token::StrictNotEqual
            | Token::AndAnd
            | Token::OrOr
            | Token::Question
            | Token::QuestionQuestion
            | Token::Colon
            | Token::LeftBracket
            | Token::LeftBrace
            | Token::Arrow
            | Token::ThinArrow
            | Token::Keyword(
                Keyword::Return
                    | Keyword::Throw
                    | Keyword::Not
                    | Keyword::TypeOf
                    | Keyword::And
                    | Keyword::Or
                    | Keyword::In
                    | Keyword::Of
                    | Keyword::If
                    | Keyword::Else
            )
    )
}

fn holds_receiver_read(source: &str) -> bool {
    let tokens: Vec<Token> = tokenize(source)
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    tokens.windows(4).any(|window| {
        window[0] != Token::Dot
            && begins_operand(&window[1])
            && window[2] == Token::Dot
            && matches!(window[3], Token::Identifier(_) | Token::Keyword(_))
    })
}

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn same_tree(receiver: &str, written: &str) {
    let read = statements(receiver);
    let spelled = statements(written);
    assert_eq!(read, spelled, "{receiver}");
    assert_eq!(generate_all(&read), generate_all(&spelled), "{receiver}");
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}: {error}");
    (error.message(), error.position().unwrap())
}

fn failure(source: &str) -> (ErrorKind, String, Option<Position>) {
    let error = Runtime::new().run(source).unwrap_err();
    (error.kind(), error.message(), error.position())
}

fn outcome(runtime: &mut Runtime, source: &str) -> String {
    match runtime.run(source) {
        Ok(value) => runtime.serialize_json(&value).unwrap(),
        Err(error) => format!(
            "{}: {} at {:?}",
            error.kind(),
            error.message(),
            error.position()
        ),
    }
}

fn renumbered(text: &str) -> String {
    let mut seen: Vec<&str> = Vec::new();
    let mut result = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("obj:") {
        let digits = after
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(after.len());
        let (identifier, tail) = after.split_at(digits);
        let ordinal = match seen.iter().position(|known| *known == identifier) {
            Some(ordinal) => ordinal,
            None => {
                seen.push(identifier);
                seen.len() - 1
            }
        };
        result.push_str(before);
        result.push_str("obj:");
        result.push_str(&ordinal.to_string());
        rest = tail;
    }
    result.push_str(rest);
    result
}

fn graph_shape(runtime: &Runtime) -> String {
    let nodes: Vec<String> = runtime
        .graph
        .nodes()
        .map(|node| {
            let reads: Vec<String> = node.dependencies.iter().map(ToString::to_string).collect();
            let updates: Vec<String> = node.dependents.iter().map(ToString::to_string).collect();
            format!("{} [{}] {reads:?} {updates:?}", node.key, node.kind)
        })
        .collect();
    renumbered(&nodes.join("\n"))
}

fn behaves_alike(steps: &[(&str, &str)]) {
    let mut read = Runtime::new();
    let mut spelled = Runtime::new();
    for (receiver, written) in steps {
        assert_eq!(
            renumbered(&outcome(&mut read, receiver)),
            renumbered(&outcome(&mut spelled, written)),
            "{receiver}"
        );
        assert_eq!(graph_shape(&read), graph_shape(&spelled), "{receiver}");
    }
}

#[test]
fn only_the_appended_cases_read_receivers_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flatten() {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(holds_receiver_read(&case.source), new, "{}", case.title);
        if new {
            let count = common::assertions::protected_assertions(&statements(&case.source)).len();
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
    assert_eq!(assertions, 24);
    assert_eq!(cases, 1105);
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Implicit receiver properties\n\n```\n")
        .expect("the language reference documents receiver reads");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 102](nuc-0102.md)"));
    assert_eq!(case.expected.as_deref(), Some("\"6 for 30\""));
}

#[test]
fn every_assignment_operator_reads_the_receiver_of_its_property_target() {
    for operator in OPERATORS {
        same_tree(
            &format!("o.p {operator} .a"),
            &format!("o.p {operator} o.a"),
        );
        same_tree(
            &format!("o.p {operator} .a * (.b + 1)"),
            &format!("o.p {operator} o.a * (o.b + 1)"),
        );
        let mut read = Runtime::new();
        let mut spelled = Runtime::new();
        let program = "o = {a: 2, b: 3, p: 4}\no.p OP VALUE\no.a = 5\no.p".replace("OP", operator);
        assert_eq!(
            outcome(&mut read, &program.replace("VALUE", ".a + .b")),
            outcome(&mut spelled, &program.replace("VALUE", "o.a + o.b")),
            "{operator}"
        );
    }
}

#[test]
fn receivers_are_the_target_without_its_last_property() {
    for (receiver, written) in [
        ("o.p = .a", "o.p = o.a"),
        ("o.c.p = .a", "o.c.p = o.c.a"),
        ("(o.c).p = .a", "(o.c).p = o.c.a"),
        ("o.c.d.p = .a.b", "o.c.d.p = o.c.d.a.b"),
        ("$C.p = .a", "$C.p = $C.a"),
        ("every C.p = .a * 2", "$C.p = $C.a * 2"),
        ("every C.p := .a", "$C.p := $C.a"),
        ("o.a@first.p = .b", "o.a@first.p = o.a@first.b"),
        (
            "class P(a):\n    this.a = a\n    this.p = .a + 1",
            "class P(a):\n    this.a = a\n    this.p = this.a + 1",
        ),
        (
            "class P(a):\n    self.a = a\n    self.p = .a + 1",
            "class P(a):\n    self.a = a\n    self.p = self.a + 1",
        ),
        (
            "class P:\n    def grow(k):\n        self.p = .p + k\n        return self.p",
            "class P:\n    def grow(k):\n        self.p = self.p + k\n        return self.p",
        ),
    ] {
        same_tree(receiver, written);
    }
}

#[test]
fn nested_values_lambdas_and_function_bodies_use_the_innermost_receiver() {
    for (receiver, written) in [
        ("o.p = (q.r = .a) + .b", "o.p = (q.r = q.a) + o.b"),
        (
            "o.p = [.a, (q.r = .b), .c]",
            "o.p = [o.a, (q.r = q.b), o.c]",
        ),
        (
            "o.p = {k: (q.r = .a), j: .b}",
            "o.p = {k: (q.r = q.a), j: o.b}",
        ),
        ("o.p = (x = 1) + .a", "o.p = (x = 1) + o.a"),
        ("o.p = xs.map(v -> v * .k)", "o.p = xs.map(v -> v * o.k)"),
        ("o.p = xs.map(v => v * .k)", "o.p = xs.map(v => v * o.k)"),
        ("o.p = lambda v: v + .a", "o.p = lambda v: v + o.a"),
        (
            "o.p = xs.map(function(v) { return v * .k })",
            "o.p = xs.map(function(v) { return v * o.k })",
        ),
        (
            "o.f = function() { q.r = .a\nreturn .b }",
            "o.f = function() { q.r = q.a\nreturn o.b }",
        ),
        (
            "o.f = function() { this.y = .a\nreturn .b }",
            "o.f = function() { this.y = this.a\nreturn o.b }",
        ),
        (
            "o.f = function() { def g(v) => v + .a\nreturn g(1) }",
            "o.f = function() { def g(v) => v + o.a\nreturn g(1) }",
        ),
        (
            "o.f = function() { for v in .xs: o.n = v + .k\nreturn o.n }",
            "o.f = function() { for v in o.xs: o.n = v + o.k\nreturn o.n }",
        ),
        (
            "$C.t = C.filter(c -> c.h > .h).length",
            "$C.t = C.filter(c -> c.h > $C.h).length",
        ),
        (
            "o.p = all v in .xs: v > .min",
            "o.p = all v in o.xs: v > o.min",
        ),
        ("o.p = .xs |> why", "o.p = o.xs |> why"),
        ("o.p = x |> depth(.n)", "o.p = x |> depth(o.n)"),
        ("o.p = .a ?? .b", "o.p = o.a ?? o.b"),
        ("o.p = not .a or -.b", "o.p = not o.a or -o.b"),
        ("o.p = typeof .a", "o.p = typeof o.a"),
        ("o.p = xs[.i] + xs[.i:]", "o.p = xs[o.i] + xs[o.i:]"),
        ("o.p = f(1 + .a, -.b)", "o.p = f(1 + o.a, -o.b)"),
        (
            "o.p = f((.a), [.b], {k: .c})",
            "o.p = f((o.a), [o.b], {k: o.c})",
        ),
        ("o.p = f(x -> x + .a)", "o.p = f(x -> x + o.a)"),
    ] {
        same_tree(receiver, written);
    }
    behaves_alike(&[
        (
            "class Box(w):\n    this.w = w",
            "class Box(w):\n    this.w = w",
        ),
        (
            "class Tray(w):\n    this.w = w\n    this.xs = [1, 2].map(i -> i * .w)",
            "class Tray(w):\n    this.w = w\n    this.xs = [1, 2].map(i -> i * this.w)",
        ),
        ("t = Tray(3)", "t = Tray(3)"),
        (
            "o = {a: 2, k: 3}\no.f = function() { return .a * 10 }\no.p = [1, 2].map(v -> v * .k)",
            "o = {a: 2, k: 3}\no.f = function() { return o.a * 10 }\no.p = [1, 2].map(v -> v * o.k)",
        ),
        (
            "o.a = 3\no.k = 4\n[o.f(), o.p]",
            "o.a = 3\no.k = 4\n[o.f(), o.p]",
        ),
    ]);
}

#[test]
fn reads_continue_with_every_postfix_suffix_and_name_the_property_as_member_access_does() {
    for (receiver, written) in [
        ("o.p = .a.b", "o.p = o.a.b"),
        ("o.p = .a[0]", "o.p = o.a[0]"),
        ("o.p = .a[1:]", "o.p = o.a[1:]"),
        ("o.p = .a[:2]", "o.p = o.a[:2]"),
        ("o.p = .a()", "o.p = o.a()"),
        ("o.p = .a.b.c(1, .d)", "o.p = o.a.b.c(1, .d)"),
        ("o.p = .a.c(1 + .d)", "o.p = o.a.c(1 + o.d)"),
        ("o.p = .a[0].b", "o.p = o.a[0].b"),
        ("o.p = .a@first", "o.p = o.a@first"),
        ("o.p = .a@last + .a@0", "o.p = o.a@last + o.a@0"),
        ("o.p = .a.length", "o.p = o.a.length"),
        ("o.p = .a.value", "o.p = o.a.value"),
        ("o.p = .self", "o.p = o.self"),
        ("o.p = .this", "o.p = o.this"),
        ("o.p = .elif", "o.p = o.elif"),
        ("o.p = .None", "o.p = o.None"),
        ("o.p = .undefined", "o.p = o.undefined"),
        ("o.p = .class", "o.p = o.class"),
        ("o.p = .if", "o.p = o.if"),
        ("o.p = .first", "o.p = o.first"),
    ] {
        same_tree(receiver, written);
    }
}

#[test]
fn template_interpolations_read_the_receiver_of_their_assignment() {
    for (receiver, written) in [
        ("o.p = `${.a}-${.b}`", "o.p = `${o.a}-${o.b}`"),
        ("o.p = `${(q.r = .a)}`", "o.p = `${(q.r = q.a)}`"),
        ("o.p = `${.a .b}`", "o.p = `${o.a .b}`"),
        (
            "o.p = `${.a if .b else .c}`",
            "o.p = `${o.a if o.b else o.c}`",
        ),
        ("o.p = `${[.a, .b]}`", "o.p = `${[o.a, o.b]}`"),
        ("o.p = `x ${.a} y`.length", "o.p = `x ${o.a} y`.length"),
        (
            "o.f = function() { return `${.a}` }",
            "o.f = function() { return `${o.a}` }",
        ),
    ] {
        same_tree(receiver, written);
    }
    assert_eq!(
        syntax_error("x = `${.a}`"),
        ("Unexpected '.'".to_string(), Position::new(1, 9))
    );
}

#[test]
fn tentative_parts_read_receivers_in_the_final_parse() {
    for (receiver, written) in [
        ("o.p = .a if .b else .c", "o.p = o.a if o.b else o.c"),
        (
            "o.p = .a if .b else .c if .d else .e",
            "o.p = o.a if o.b else o.c if o.d else o.e",
        ),
        (
            "o.f = function() { while 0 < .n: o.n = .n - 1\nreturn o.n }",
            "o.f = function() { while 0 < o.n: o.n = o.n - 1\nreturn o.n }",
        ),
        (
            "o.f = function() { s = suppose o.n = .n + 1: o.n\nreturn s }",
            "o.f = function() { s = suppose o.n = o.n + 1: o.n\nreturn s }",
        ),
        ("{a} = (q.r = .x)", "{a} = (q.r = q.x)"),
        (
            "o.f = function() { [a, b] = [(q.r = .x), 2]\nreturn a }",
            "o.f = function() { [a, b] = [(q.r = q.x), 2]\nreturn a }",
        ),
    ] {
        same_tree(receiver, written);
    }
    assert_eq!(
        syntax_error("o.f = function() { [a, b] = [(q.r = .x), .y]\nreturn a }"),
        ("Unexpected '.'".to_string(), Position::new(1, 43))
    );
    behaves_alike(&[
        ("q = {x: {a: 7}, r: 0}", "q = {x: {a: 7}, r: 0}"),
        (
            "{a} = (q.r = .x)\n[a, q.r.a]",
            "{a} = (q.r = q.x)\n[a, q.r.a]",
        ),
    ]);
    assert_eq!(
        parse_expression("o.p = .a if .b else .c").unwrap(),
        parse_expression("o.p = o.a if o.b else o.c").unwrap()
    );
    behaves_alike(&[
        (
            "o = {a: 1, b: false, c: 3, n: 3}",
            "o = {a: 1, b: false, c: 3, n: 3}",
        ),
        ("o.p = .a if .b else .c", "o.p = o.a if o.b else o.c"),
        ("o.b = true\no.p", "o.b = true\no.p"),
        (
            "o.f = function() { while 0 < .n: o.n = .n - 1\nreturn o.n }\no.f()",
            "o.f = function() { while 0 < o.n: o.n = o.n - 1\nreturn o.n }\no.f()",
        ),
    ]);
}

#[test]
fn kept_readings_keep_their_accepted_diagnostics() {
    for (source, message, line, column) in [
        ("x = .a", "Unexpected '.'", 1, 6),
        (".a = 1", "Unexpected '.'", 1, 2),
        ("o.p = (.a = 1)", "Unexpected '.'", 1, 9),
        ("o.p = (x = .a)", "Unexpected '.'", 1, 13),
        ("o.p = f(.a + 1)", "Unexpected '.'", 1, 10),
        ("o.p = f(.a.b + 1)", "Unexpected '.'", 1, 10),
        ("o.p = f(x, .a + 1)", "Unexpected '.'", 1, 13),
        ("o.p = xs.map(.a + 1)", "Unexpected '.'", 1, 15),
        ("o.p = assert(.a + 1, 1)", "Unexpected '.'", 1, 15),
        ("o.p = new X(.a + 1)", "Unexpected '.'", 1, 14),
        ("o.p = super(.a + 1)", "Unexpected '.'", 1, 14),
        ("o.p = P(n = .a)", "Unexpected '.'", 1, 14),
        ("o.p = (x is .a)", "Unexpected '.'", 1, 14),
        ("o.p = (1 is .a)", "Unexpected '.'", 1, 14),
        ("o.p = 1 is not .a", "Unexpected '.'", 1, 17),
        ("$Obj.d is 2 * .h", "Unexpected '.'", 1, 16),
        ("o.p is (q.r = .a)", "Unexpected '.'", 1, 16),
        ("assert: .a, 1", "Unexpected '.'", 1, 10),
        ("while .a: x", "Unexpected ':'", 1, 11),
        ("{a} = .b", "Unexpected '='", 1, 7),
        ("a, b = .c", "Unexpected ','", 1, 4),
        ("xs[0].p = .a", "Unexpected '.'", 1, 12),
        ("f().p = .a", "Unexpected '.'", 1, 10),
        ("o.p@first = .a", "Unexpected '.'", 1, 14),
        ("$C = .a", "Unexpected '.'", 1, 7),
        ("if .a: x = 1", "Unexpected '.'", 1, 5),
        ("def f() => .a", "Unexpected '.'", 1, 13),
        ("throw .a", "Unexpected '.'", 1, 8),
        ("o.f = function() { .a }", "Unexpected '.'", 1, 21),
        ("o.f = function() { x = .a }", "Unexpected '.'", 1, 25),
        ("o.f = function() { {a} = .pair }", "Unexpected '='", 1, 26),
        ("x = a?? .b", "Unexpected '.'", 1, 10),
        ("o.p = .", "Unexpected '.'", 1, 8),
        ("o.p = ..a", "Unexpected '.'", 1, 8),
        ("o.p = [...xs]", "Unexpected '.'", 1, 9),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    assert!(parse_expression(".a").is_err());
    assert!(parse_expression("x = .a").is_err());
}

#[test]
fn is_assignments_keep_their_juxtaposed_readings() {
    for (source, juxtaposed) in [
        ("$Obj.h is .h * 2", "$Obj.h\nis.h * 2"),
        ("$Obj.h is .h@first * 2", "$Obj.h\nis.h@first * 2"),
        ("o.p is .a", "o.p\nis.a"),
        ("o.p = x is .a", "o.p = x\nis.a"),
        ("o.p = 3 is .a", "o.p = 3\nis.a"),
        (
            "class P(a):\n    this.a = a\n    this.b is .a",
            "class P(a):\n    this.a = a\n    this.b\n    is.a",
        ),
    ] {
        assert_eq!(statements(source), statements(juxtaposed), "{source}");
    }
    assert_eq!(
        failure("class Obj(h):\n    this.h = h\nb = Obj(2)\n$Obj.h is .h * 2"),
        (
            ErrorKind::Reference,
            "is is not defined".to_string(),
            Some(Position::new(4, 8))
        )
    );
}

#[test]
fn selectors_member_access_and_numbers_keep_their_readings() {
    for (source, expected) in [
        ("def f(g) => g({a: 7})\no = {a: 1}\no.p = f(.a)\no.p", "7.0"),
        (
            "def f(g) => g({a: 7})\no = {a: 1}\no.p = f(.a) + .a\no.p",
            "8.0",
        ),
        (
            "def f(v) => v * 10\no = {a: 2}\no.p = f(1 + .a)\no.p",
            "30.0",
        ),
        ("x = {a: 4}\no = {}\no.p = x .a\no.p", "4.0"),
        ("o = {}\no.p = 1 .a\no.p", "null"),
        ("o = {}\no.p = .5\no.p", "0.5"),
        ("o = {a: 1}\no.p = [o.a\n.a]\no.p", "[null]"),
        ("o = {}\no.p = {...{a: 2}}\no.p", "{\"a\":2.0}"),
    ] {
        assert_eq!(outcome(&mut Runtime::new(), source), expected, "{source}");
    }
}

#[test]
fn errors_whose_dots_now_read_report_the_furthest_position() {
    for (source, message, line, column) in [
        ("o.p = .a +", "Unexpected end of input", 1, 11),
        ("o.p = a?.b", "Expected ':' but found end of input", 1, 11),
        ("o.p = .a\n.b", "Unexpected '.'", 2, 2),
        ("o.p = `${.a +}`", "Unexpected end of input", 1, 14),
        (
            "o.p += .a.",
            "Expected a property but found end of input",
            1,
            11,
        ),
        (
            "o.p = .a .",
            "Expected a property but found end of input",
            1,
            11,
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
fn receiver_paths_and_chains_respect_the_accepted_limits() {
    let long = ".a".repeat(127);
    same_tree(
        &format!("(o{long}).p = .x.y"),
        &format!("(o{long}).p = (o{long}).x.y"),
    );
    let unreadable = format!("o.q = .a\n(o{long}).b.p = .x");
    assert_eq!(
        syntax_error(&unreadable),
        (
            "Expression has too many parts".to_string(),
            Position::new(2, unreadable.lines().nth(1).unwrap().len() - 1)
        )
    );
    assert_eq!(
        syntax_error(&format!("(o{long}).b.p = .x")),
        (
            "Unexpected '.'".to_string(),
            Position::new(1, long.len() + 12)
        )
    );
    let suffixes = ".b".repeat(126);
    same_tree(
        &format!("o.p = .a{suffixes}"),
        &format!("o.p = (o).a{suffixes}"),
    );
    let (message, _) = syntax_error(&format!("o.p = .a{suffixes}.b"));
    assert_eq!(message, "Expression has too many parts");
    let (written, _) = syntax_error(&format!("o.p = (o).a{suffixes}.b"));
    assert_eq!(message, written);
}

#[test]
fn reads_after_new_count_toward_the_chain_bound_as_their_written_out_paths() {
    let suffixes = ".b".repeat(126);
    for news in ["new ", "new new "] {
        same_tree(
            &format!("o.p = {news}.a{suffixes}"),
            &format!("o.p = {news}(o).a{suffixes}"),
        );
        for source in [
            format!("o.p = {news}.a{suffixes}.b"),
            format!("o.p = {news}(o).a{suffixes}.b"),
            format!("o.p = {news}o.a{suffixes}.b"),
        ] {
            assert_eq!(
                syntax_error(&source),
                (
                    "Expression has too many parts".to_string(),
                    Position::new(1, source.len() + 1)
                ),
                "{source}"
            );
        }
    }
}

#[test]
fn receiver_reads_render_explain_depend_and_update_as_their_written_out_paths() {
    same_tree(
        "{ o.p = .a + 1 }\nif o.b:\n    o.q = .b",
        "{ o.p = o.a + 1 }\nif o.b:\n    o.q = o.b",
    );
    assert_eq!(
        generate_all(&statements("$Order.total = .price * .qty")),
        "$Order.total=$Order.price*$Order.qty"
    );
    behaves_alike(&[
        (
            "class Order(price, qty):\n    this.price = price\n    this.qty = qty",
            "class Order(price, qty):\n    this.price = price\n    this.qty = qty",
        ),
        (
            "$Order.total = .price * .qty",
            "$Order.total = $Order.price * $Order.qty",
        ),
        ("first = Order(4, 5)", "first = Order(4, 5)"),
        ("first.qty = 6\nfirst.total", "first.qty = 6\nfirst.total"),
        ("(why first.total)[0].rule", "(why first.total)[0].rule"),
        (
            "(affects first.qty).map(step -> step.node)",
            "(affects first.qty).map(step -> step.node)",
        ),
        (
            "$Order.change = .total - .total@first",
            "$Order.change = $Order.total - $Order.total@first",
        ),
        (
            "first.price = 5\nfirst.change",
            "first.price = 5\nfirst.change",
        ),
    ]);
    behaves_alike(&[
        ("o = {c: {a: 1}, a: 2}", "o = {c: {a: 1}, a: 2}"),
        ("o.c.p = .a", "o.c.p = o.c.a"),
        ("o.q = .c.p + .a", "o.q = o.c.p + o.a"),
        ("o.s = .a.value + 1", "o.s = o.a.value + 1"),
        (
            "o.c.a = 5\no.a = 3\n[o.c.p, o.q, o.s]",
            "o.c.a = 5\no.a = 3\n[o.c.p, o.q, o.s]",
        ),
        ("{ o.t = .a * 2 }\no.t", "{ o.t = o.a * 2 }\no.t"),
        (
            "o.l = `${.a}/${.c.a}`\no.a = 9\no.l",
            "o.l = `${o.a}/${o.c.a}`\no.a = 9\no.l",
        ),
    ]);
}

#[test]
fn failed_transactions_roll_back_receiver_rules_as_their_written_out_spellings() {
    behaves_alike(&[
        (
            "class Cart(qty, price):\n    this.qty = qty\n    this.price = price",
            "class Cart(qty, price):\n    this.qty = qty\n    this.price = price",
        ),
        (
            "$Cart.total = .qty * .price",
            "$Cart.total = $Cart.qty * $Cart.price",
        ),
        ("cart = Cart(2, 5)", "cart = Cart(2, 5)"),
        (
            "try:\n    cart.price = 7\n    $Cart.total = .qty + .price\n    throw \"UNDO\"\ncatch error:\n    error",
            "try:\n    cart.price = 7\n    $Cart.total = $Cart.qty + $Cart.price\n    throw \"UNDO\"\ncatch error:\n    error",
        ),
        ("[cart.price, cart.total]", "[cart.price, cart.total]"),
        ("$Cart.total = .total + 1", "$Cart.total = $Cart.total + 1"),
        ("cart.qty = 3\ncart.total", "cart.qty = 3\ncart.total"),
        (
            "o = {a: 1}\no.p = .a + ghost",
            "o = {a: 1}\no.p = o.a + ghost",
        ),
        ("o.p", "o.p"),
    ]);
}

#[test]
fn the_final_parse_runs_only_after_every_accepted_parse_fails() {
    assert_eq!(
        parse_expression("o.p = [v for v in xs] if .c else 2").unwrap(),
        parse_expression("o.p = [v for v in xs]").unwrap()
    );
    assert_eq!(
        parse_expression("o.p = x .a if .c else 2").unwrap(),
        parse_expression("o.p = x .a").unwrap()
    );
    for (receiver, written) in [
        ("o.p = (x is 3) ? .a : .b", "o.p = (x is 3) ? o.a : o.b"),
        ("o.p = (x is not y) and .a", "o.p = (x is not y) and o.a"),
        ("o.p = (1 is not .a)", "o.p = (1 is not o.a)"),
        ("o.p = [1 is not .a]", "o.p = [1 is not o.a]"),
        (
            "o.p = [v * .k for v in .xs]",
            "o.p = [v * o.k for v in o.xs]",
        ),
        (
            "o.p = [v for v of .xs if v > .min]",
            "o.p = [v for v of o.xs if v > o.min]",
        ),
        ("o.p = `${.a}`\nr = x is 3", "o.p = `${o.a}`\nr = x is 3"),
        ("o.p = {...base, k: .a}", "o.p = {...base, k: o.a}"),
    ] {
        same_tree(receiver, written);
    }
    let Stmt::Assign { value, .. } = &statements("o.p = {...base}")[0] else {
        panic!("a property assignment");
    };
    assert!(matches!(value, Expr::SpreadObject(_)), "{value:?}");
}

#[derive(Clone, Copy, Debug)]
struct Shape {
    head: &'static str,
    opener: &'static str,
    base: &'static str,
    closer: &'static str,
    tail: &'static str,
}

const fn shape(
    head: &'static str,
    opener: &'static str,
    base: &'static str,
    closer: &'static str,
    tail: &'static str,
) -> Shape {
    Shape {
        head,
        opener,
        base,
        closer,
        tail,
    }
}

const NESTED_SHAPES: [[Shape; 2]; 8] = [
    [
        shape("o.p = ", "f(x -> ", ".a", ")", ""),
        shape("o.p = ", "f(x -> ", "o.a", ")", ""),
    ],
    [
        shape("o.p = ", "(o.q = ", ".a", ")", ""),
        shape("o.p = ", "(o.q = ", "o.a", ")", ""),
    ],
    [
        shape("o.p = ", "{k: ", ".a", "}", ""),
        shape("o.p = ", "{k: ", "o.a", "}", ""),
    ],
    [
        shape("o.p = ", "[", ".a", "]", ""),
        shape("o.p = ", "[", "o.a", "]", ""),
    ],
    [
        shape("o.p = ", "function() { return ", ".a", " }", ""),
        shape("o.p = ", "function() { return ", "o.a", " }", ""),
    ],
    [
        shape("o.p = `${", "(", ".a", ")", "}`"),
        shape("o.p = `${", "(", "o.a", ")", "}`"),
    ],
    [
        shape("o.p = ", "1 if .b else (", ".a", ")", ""),
        shape("o.p = ", "1 if o.b else (", "o.a", ")", ""),
    ],
    [
        shape("o.p = ", ".b ? (", ".a", ") : 0", ""),
        shape("o.p = ", "o.b ? (", "o.a", ") : 0", ""),
    ],
];

fn deepest(shape: Shape) -> usize {
    let Shape {
        head,
        opener,
        base,
        closer,
        tail,
    } = shape;
    let started = Instant::now();
    for depth in 1..200 {
        let source = format!(
            "{head}{}{base}{}{tail}",
            opener.repeat(depth),
            closer.repeat(depth)
        );
        let checked = Runtime::check(&source);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{opener}{base} at depth {depth}"
        );
        if let Err(error) = checked {
            assert!(
                matches!(
                    error.message().as_str(),
                    "Expressions are nested too deeply" | "Statements are nested too deeply"
                ),
                "{opener}{base} at depth {depth}: {error}"
            );
            return depth;
        }
    }
    panic!("{opener}{base} never reaches the nesting limit")
}

#[test]
fn nested_receiver_reads_parse_each_level_once_up_to_the_nesting_limit() {
    for [read, written] in NESTED_SHAPES {
        let limit = deepest(read);
        assert!(limit > 15, "{read:?} stops at {limit}");
        assert_eq!(limit, deepest(written), "{read:?}");
    }
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-implicit-receivers-{label}-{}.nuc",
            std::process::id()
        ));
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        Self(path)
    }

    fn run(&self, source: &str, options: &[&str]) -> Output {
        std::fs::write(&self.0, source).unwrap();
        Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .args(options)
            .arg("--json")
            .arg(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for SourceFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).unwrap();
    }
}

fn repl(source: &str, options: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(options)
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
fn files_and_repl_sessions_run_receiver_reads_as_their_written_out_spellings() {
    let program = "class Order(price, qty):\n    this.price = price\n    this.qty = qty\n$Order.total = .price * .qty\norder = Order(4, 5)\norder.label = `${.qty} for ${.total}`\norder.qty = 6\n[order.total, order.label]";
    let written = "class Order(price, qty):\n    this.price = price\n    this.qty = qty\n$Order.total = $Order.price * $Order.qty\norder = Order(4, 5)\norder.label = `${order.qty} for ${order.total}`\norder.qty = 6\n[order.total, order.label]";
    let file = SourceFile::new("file");
    for options in [&[][..], &["--graph"][..]] {
        let read = file.run(program, options);
        assert!(read.status.success(), "{read:?}");
        assert!(read.stderr.is_empty(), "{read:?}");
        let spelled = file.run(written, options);
        assert_eq!(read.stdout, spelled.stdout, "{options:?}");
    }
    assert_eq!(
        String::from_utf8(file.run(program, &[]).stdout).unwrap(),
        "[24.0,\"6 for 24\"]\n"
    );
    let failed = file.run("o = {a: 1}\nx = .a", &[]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "SyntaxError: Unexpected '.' (line 2, column 6)\n"
    );
    let session = "o = {a: 1, b: 2}\no.p = .a + .b\no.p\no.a = 5\no.p\no.l = `${.a}-${.b}`\no.l\nx = .a\n.clear\no = {a: 1}\no.p = .a\no.p\n.exit\n";
    let spelled = "o = {a: 1, b: 2}\no.p = o.a + o.b\no.p\no.a = 5\no.p\no.l = `${o.a}-${o.b}`\no.l\nx = .a\n.clear\no = {a: 1}\no.p = o.a\no.p\n.exit\n";
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let options: &[&str] = if plain { &["--plain"] } else { &[] };
            let read = repl(&session.replace('\n', newline), options);
            let written = repl(&spelled.replace('\n', newline), options);
            assert!(read.status.success(), "{read:?}");
            assert_eq!(read.stdout, written.stdout, "{plain} {newline:?}");
            assert_eq!(read.stderr, written.stderr, "{plain} {newline:?}");
            let stdout = String::from_utf8(read.stdout).unwrap();
            assert!(stdout.contains("\n3.0\n3.0\n5.0\n7.0\n"), "{stdout}");
            assert!(stdout.contains("\"5-2\""), "{stdout}");
            assert_eq!(
                String::from_utf8(read.stderr).unwrap(),
                "SyntaxError: Unexpected '.' (line 1, column 6)\n"
            );
        }
    }
}
