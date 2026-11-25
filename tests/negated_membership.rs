mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Spanned, Token, tokenize};
use nucleoid::lang::ast::{BinaryOp, Expr, Stmt, UnaryOp, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime};
use serde::Deserialize;

const SPEC_TITLES: [&str; 3] = [
    "Nucleoid tests negated membership with Kotlin's !in",
    "Nucleoid keeps !in rules live over class populations",
    "Nucleoid rolls back failed updates through !in",
];
const REFERENCE_TITLE: &str = "Nucleoid negates membership tests with !in";
const README_TITLE: &str = "Kotlin's !in keeps an access rule live";
const EXAMPLES_TITLE: &str = "Negated membership keeps a support queue's open tickets live";

const NUC_97_AND_98_SPEC_TITLES: [&str; 10] = [
    "Nucleoid tests membership with in and not in",
    "Nucleoid keeps membership tests live",
    "Nucleoid binds membership tests as comparisons",
    "Nucleoid waits on null collections and rejects other values in membership tests",
    "Nucleoid keeps in readings where membership tests do not apply",
    "Nucleoid states class predicates with is",
    "Nucleoid spells class references with every",
    "Nucleoid quantifies with every as with all",
    "Nucleoid keeps every an ordinary name where no name follows it",
    "Nucleoid keeps other is statements on a class reference",
];
const NUC_97_AND_98_REFERENCE_TITLES: [&str; 2] = [
    "Nucleoid tests membership with in",
    "Nucleoid states universal claims with every and is",
];
const NUC_97_AND_98_README_TITLES: [&str; 2] = [
    "Membership tests read in the order they are said",
    "Every states a class-level claim as it is said",
];
const NUC_97_AND_98_EXAMPLES_TITLES: [&str; 2] = [
    "Membership tests keep an order book's rules live",
    "Every keeps a fleet's rules stated as claims about every truck",
];

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

fn documents() -> [Vec<common::Case>; 4] {
    [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ]
}

fn appended(title: &str) -> bool {
    SPEC_TITLES.contains(&title) || [REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE].contains(&title)
}

fn adjacent(bang: &Spanned, keyword: &Spanned) -> bool {
    bang.token == Token::Bang
        && keyword.token == Token::Keyword(Keyword::In)
        && keyword.position.line == bang.position.line
        && keyword.position.column == bang.position.column + 1
}

fn bang_ins(source: &str) -> Vec<Position> {
    tokenize(source)
        .unwrap()
        .windows(2)
        .filter(|pair| adjacent(&pair[0], &pair[1]))
        .map(|pair| pair[0].position)
        .collect()
}

fn interpolations(template: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("${") {
        let body = &rest[start + 2..];
        let mut depth = 0usize;
        let mut end = body.len();
        for (index, character) in body.char_indices() {
            match character {
                '{' => depth += 1,
                '}' if depth == 0 => {
                    end = index;
                    break;
                }
                '}' => depth -= 1,
                _ => {}
            }
        }
        found.push(&body[..end]);
        rest = &body[end..];
    }
    found
}

fn writes_bang_in(source: &str) -> bool {
    let Ok(tokens) = tokenize(source) else {
        return false;
    };
    tokens.windows(2).any(|pair| adjacent(&pair[0], &pair[1]))
        || tokens.iter().any(|spanned| match &spanned.token {
            Token::Template(text) => interpolations(text).into_iter().any(writes_bang_in),
            _ => false,
        })
}

fn spelled(source: &str) -> (String, usize) {
    let positions = bang_ins(source);
    let mut lines: Vec<String> = source.split('\n').map(str::to_string).collect();
    for position in positions.iter().rev() {
        let line = &mut lines[position.line - 1];
        let (offset, bang) = line.char_indices().nth(position.column - 1).unwrap();
        assert_eq!(bang, '!');
        let spaced = line[..offset]
            .chars()
            .next_back()
            .is_none_or(char::is_whitespace);
        line.replace_range(offset..=offset, if spaced { "not " } else { " not " });
    }
    (lines.join("\n"), positions.len())
}

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn tree(source: &str) -> String {
    format!("{:?}", statements(source))
}

fn rendered(source: &str) -> String {
    generate_all(&statements(source))
}

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn negated(item: Expr, collection: Expr) -> Expr {
    Expr::Binary {
        operator: BinaryOp::NotIn,
        left: Box::new(item),
        right: Box::new(collection),
    }
}

fn result(source: &str) -> String {
    let mut runtime = Runtime::new();
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    assert!(runtime.take_assertions().is_empty(), "{source}");
    runtime.serialize_json(&value).unwrap()
}

fn checked(source: &str, expected: &str) -> String {
    let count = common::assertions::protected_assertions(&statements(source)).len();
    let case = common::Case {
        title: "negated membership regression".to_string(),
        source: source.to_string(),
        expected: Some(expected.to_string()),
    };
    let mut runtime = Runtime::new();
    let value = common::check_with_count(&mut runtime, &case, count)
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run(), count, "{source}");
    runtime.serialize_json(&value).unwrap()
}

fn failure(source: &str) -> (ErrorKind, String, Option<Position>) {
    let error = Runtime::new().run(source).unwrap_err();
    (error.kind(), error.message(), error.position())
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn dependencies(runtime: &Runtime, name: &str) -> Vec<String> {
    let mut found: Vec<String> = runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap_or_else(|| panic!("{name} is filed"))
        .dependencies
        .iter()
        .map(|key| key.as_str().to_string())
        .collect();
    found.sort();
    found
}

#[test]
fn the_appended_cases_run_every_assertion_and_end_each_document() {
    let [spec, reference, readme, examples] = documents();
    let added = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (&reference, REFERENCE_TITLE),
        (&readme, README_TITLE),
        (&examples, EXAMPLES_TITLE),
    ]);
    let mut assertions = 0;
    for (cases, title) in added {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(writes_bang_in(&case.source), "{title} writes !in");
        assert!(case.expected.is_some(), "{title} declares a return");
        let count = common::assertions::protected_assertions(&statements(&case.source)).len();
        assert!(count > 0, "{title} asserts its behavior");
        let mut runtime = Runtime::new();
        common::check_with_count(&mut runtime, case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assert_eq!(runtime.assertions_run(), count, "{title}");
        assertions += count;
    }
    assert_eq!(assertions, 30);
    for (cases, own, earlier) in [
        (&spec, &SPEC_TITLES[..], &NUC_97_AND_98_SPEC_TITLES[..]),
        (
            &reference,
            &[REFERENCE_TITLE][..],
            &NUC_97_AND_98_REFERENCE_TITLES[..],
        ),
        (
            &readme,
            &[README_TITLE][..],
            &NUC_97_AND_98_README_TITLES[..],
        ),
        (
            &examples,
            &[EXAMPLES_TITLE][..],
            &NUC_97_AND_98_EXAMPLES_TITLES[..],
        ),
    ] {
        let first = position(cases, own[0]);
        for (offset, title) in own.iter().enumerate() {
            assert_eq!(
                position(cases, title),
                first + offset,
                "{title} stands with the other appended cases"
            );
        }
        assert_eq!(
            first + own.len(),
            cases.len() - 7 * usize::from(own[0] == SPEC_TITLES[0] || own[0] == REFERENCE_TITLE),
            "the appended cases end the document of {}",
            own[0]
        );
        for title in earlier {
            assert!(
                position(cases, title) < first,
                "{title} stands before {}",
                own[0]
            );
        }
    }
}

fn position(cases: &[common::Case], title: &str) -> usize {
    let found: Vec<usize> = cases
        .iter()
        .enumerate()
        .filter(|(_, case)| case.title == title)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(found.len(), 1, "{title} stands once in its document");
    found[0]
}

#[test]
fn only_the_appended_cases_write_bang_in() {
    let documents = documents();
    let mut cases = 0;
    let mut written = 0;
    for case in documents.iter().flatten() {
        cases += 1;
        let new = appended(&case.title);
        assert_eq!(writes_bang_in(&case.source), new, "{}", case.title);
        if new {
            written += bang_ins(&case.source).len();
        }
    }
    assert_eq!(written, 25);
    assert_eq!(cases, 1320);
}

#[test]
fn the_audit_finds_every_adjacent_bang_in_and_nothing_else() {
    for source in [
        "x !in xs",
        "ok = [x !in xs]",
        "ok = 1!in [2]",
        "if x !in xs:\n    r = 1",
        "$Order.ok = $Order.v !in xs",
        "ok = f(y, x !in xs)",
        "ok = `${(x !in xs)}`",
        "ok = `a ${[\"b\", {k: [x !in xs]}]}`",
        "ok = `${x !in xs}`",
        "ok = x !in xs: v",
        "ok = !in xs",
    ] {
        assert!(writes_bang_in(source), "{source}");
    }
    for source in [
        "x not in xs",
        "x in xs",
        "x ! in xs",
        "x !\tin xs",
        "ok = [x !\n     in xs]",
        "x !/* c */in xs",
        "x !# c\nin xs",
        "x !inx",
        "\"x !in xs\"",
        "'!in'",
        "# x !in xs",
        "x != y",
        "x !== y",
        "!(x in xs)",
        "`x !in xs`",
        "`${x} !in ${xs}`",
        "o.in",
    ] {
        assert!(!writes_bang_in(source), "{source}");
    }
    assert_eq!(
        spelled("ok = [x !in xs, 1!in ys, y!in zs]\n`${(z !in zs)}`"),
        (
            "ok = [x not in xs, 1 not in ys, y not in zs]\n`${(z !in zs)}`".to_string(),
            3
        )
    );
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Negated membership with !in\n\n```\n")
        .expect("the language reference documents negated membership");
    let (example, rest) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(case.expected.as_deref(), Some("false"));
    let (section, _) = rest.split_once("\n### ").unwrap();
    assert!(section.contains("Full detail: [NUC 135](nuc-0135.md)."));
    assert!(reference.contains(
        "item !in collection                    Kotlin spelling: exactly item not in collection, ! and in adjacent (NUC 135)\n"
    ));
    assert!(reference.contains("[negated membership with `!in`](#negated-membership-with-in)"));
    let spec = include_str!("../nucleoid.spec.md").replace("\r\n", "\n");
    assert!(
        spec.contains("\n## Negated membership with !in\n\n- [NUC 135](docs/nuc-0135.md) adds")
    );
    assert!(spec.contains("[Negated membership with !in](#negated-membership-with-in)"));
}

#[test]
fn each_case_gives_what_its_not_in_spelling_gives() {
    let mut rewrites = 0;
    for case in documents()
        .iter()
        .flatten()
        .filter(|case| appended(&case.title))
    {
        let (source, count) = spelled(&case.source);
        assert!(count > 0, "{}", case.title);
        assert!(!writes_bang_in(&source), "{}", case.title);
        rewrites += count;
        let assertions = common::assertions::protected_assertions(&statements(&case.source)).len();
        assert_eq!(
            assertions,
            common::assertions::protected_assertions(&statements(&source)).len(),
            "{}",
            case.title
        );
        let spelling = common::Case {
            title: case.title.clone(),
            source,
            expected: case.expected.clone(),
        };
        let mut alias = Runtime::new();
        let mut written = Runtime::new();
        let alias_value = common::check_with_count(&mut alias, case, assertions)
            .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
        let written_value = common::check_with_count(&mut written, &spelling, assertions)
            .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
        assert_eq!(
            alias.serialize_json(&alias_value).unwrap(),
            written.serialize_json(&written_value).unwrap(),
            "{}",
            case.title
        );
        assert_eq!(
            alias.assertions_run(),
            written.assertions_run(),
            "{}",
            case.title
        );
        assert_eq!(
            statements(&case.source),
            statements(&spelling.source),
            "{}",
            case.title
        );
    }
    assert_eq!(rewrites, 25);
}

const CONTEXTS: [&str; 66] = [
    "{m}",
    "ok = {m}",
    "ok = f({m})",
    "ok = f(1, {m})",
    "ok = [{m}, 1]",
    "ok = {k: {m}}",
    "ok = {k: {m} j: 1}",
    "ok = {...o, k: {m}}",
    "ok = c and {m}",
    "ok = {m} or c",
    "ok = c && {m}",
    "ok = not ({m})",
    "ok = !({m})",
    "ok = ({m}) ? 1 : 2",
    "ok = {m} ? 1 : 2",
    "ok = c ? {m} : 2",
    "ok = c ? 1 : {m}",
    "ok = 1 if {m} else 2",
    "ok = {m} if c else 2",
    "ok = {m} == true",
    "ok = {m} ?? c",
    "ok &&= {m}",
    "ok := {m}",
    "ok is {m}",
    "ok = (a is {m})",
    "a, b = [{m}, 1]",
    "$Order.ok = {m}",
    "if $Order.v:\n    $Order.ok = {m}",
    "assert: {m}, true",
    "assert({m}, true)",
    "ok = xs.filter(w => {m})",
    "ok = xs.filter(w -> {m})",
    "ok = xs.filter(lambda w: {m})",
    "ok = xs.filter(function(w) { return {m} })",
    "ok = xs.filter(== {m})",
    "ok = all t in ts: {m}",
    "ok = any t in ts: {m}",
    "ok = every t in ts: {m}",
    "ok = all t in [{m}]: t",
    "ok = all t in {m}: t",
    "ok = [{m} for t in ts]",
    "ok = [t for t in {m}]",
    "ok = [t for t in ts if {m}]",
    "ok = `${({m})}`",
    "ok = `${[{m}]}`",
    "ok = `${f({m})}`",
    "ok = `${c ? {m} : 0}`",
    "if {m}:\n    r = 1",
    "if {m}\n    r = 1",
    "if c:\n    r = 1\nelif {m}:\n    r = 2",
    "if c:\n    r = 1\nelse {m}\n    r = 2",
    "while {m}:\n    r = 1",
    "repeat:\n    r = 1\nuntil {m}",
    "for t of [{m}]:\n    r = t",
    "def g() => {m}",
    "def g():\n    return {m}",
    "class C:\n    def m(self) => {m}",
    "{ ok = {m} }",
    "ok = lambda: {m}",
    "ok = ys[{m}]",
    "ok = ys[{m}:]",
    "ok = ys[1:{m}]",
    "suppose ok = {m}:\n    r = 1",
    "throw {m}",
    "ok = typeof ({m})",
    "ok = ({m}).toString()",
];

#[test]
fn bang_in_builds_the_tree_of_not_in_wherever_not_in_stands() {
    let mut compared = 0;
    for context in CONTEXTS {
        let alias = tree(&context.replace("{m}", "x !in xs"));
        assert_eq!(
            alias,
            tree(&context.replace("{m}", "x not in xs")),
            "{context}"
        );
        assert_eq!(
            alias,
            tree(&context.replace("{m}", "x < xs")).replace("operator: Less,", "operator: NotIn,"),
            "{context}"
        );
        compared += 1;
    }
    assert_eq!(compared, 66);
    assert_eq!(
        statements("ok = x !in xs"),
        vec![Stmt::Assign {
            target: name("ok"),
            value: negated(name("x"), name("xs")),
        }]
    );
    assert_eq!(
        statements("o.p = .a !in .b"),
        statements("o.p = .a not in .b")
    );
    assert_eq!(
        statements("[x !in ys for x in xs]"),
        statements("xs.map(x => x not in ys)")
    );
    for (alias, spelling) in [
        ("ok = [4\n!in xs]", "ok = [4\nnot in xs]"),
        (
            "ok = (item\n      !in blocked)",
            "ok = (item\n      not in blocked)",
        ),
        ("ok = f(1,\n  x\n  !in xs)", "ok = f(1,\n  x\n  not in xs)"),
        ("ok = 1!in[2]", "ok = 1 not in [2]"),
        ("ok = (x)!in(xs)", "ok = (x) not in (xs)"),
        ("ok = \"b\"!in\"abc\"", "ok = \"b\" not in \"abc\""),
    ] {
        assert_eq!(statements(alias), statements(spelling), "{alias:?}");
    }
}

#[test]
fn bang_in_binds_as_tightly_as_not_in() {
    for (source, grouped) in [
        ("ok = x + 1 !in xs", "ok = (x + 1) !in xs"),
        ("ok = x !in xs + ys", "ok = x !in (xs + ys)"),
        ("ok = x * 2 !in xs", "ok = (x * 2) !in xs"),
        ("ok = x ** 2 !in xs", "ok = (x ** 2) !in xs"),
        ("ok = x !in xs == true", "ok = (x !in xs) == true"),
        ("ok = x !in xs != y !in ys", "ok = (x !in xs) != (y !in ys)"),
        ("ok = a < b !in c", "ok = (a < b) !in c"),
        ("ok = a !in b < c", "ok = (a !in b) < c"),
        ("ok = a !in b !in c", "ok = (a !in b) !in c"),
        ("ok = a !in b in c", "ok = (a !in b) in c"),
        ("ok = a in b !in c", "ok = (a in b) !in c"),
        ("ok = a not in b !in c", "ok = (a not in b) !in c"),
        (
            "ok = x !in xs and y !in ys",
            "ok = (x !in xs) and (y !in ys)",
        ),
        ("ok = x !in xs || y", "ok = (x !in xs) || y"),
        ("ok = x !in xs ?? y", "ok = (x !in xs) ?? y"),
        ("ok = not x !in xs", "ok = (not x) !in xs"),
        ("ok = !x !in xs", "ok = (!x) !in xs"),
        ("ok = -x !in xs", "ok = (-x) !in xs"),
        ("ok = typeof x !in xs", "ok = (typeof x) !in xs"),
        ("ok = x !in why c", "ok = x !in (why c)"),
        ("ok = x !in sum of xs", "ok = x !in (sum of xs)"),
        ("ok = x !in xs ? 1 : 2", "ok = (x !in xs) ? 1 : 2"),
        ("ok = (a is b !in c)", "ok = (a is (b !in c))"),
        ("ok = (a !in b is c)", "ok = ((a !in b) is c)"),
    ] {
        assert_eq!(statements(source), statements(grouped), "{source}");
        assert_eq!(
            statements(source),
            statements(&source.replace("!in", "not in")),
            "{source}"
        );
    }
    assert_ne!(
        statements("ok = not x !in xs"),
        statements("ok = not (x !in xs)")
    );
    assert_ne!(statements("ok = !x in xs"), statements("ok = x !in xs"));
    assert_eq!(statements("ok = !x in xs"), statements("ok = (!x) in xs"));
}

const SETUP: &str = "class Seat(row):\n    this.row = row\nseat = Seat(1)\nother = Seat(2)\nmixed = [1, \"a\", null, [1, 2], {x: 1}, true, seat]\nempty = []\ntext = \"a1 true null [object]\"\nblank = \"\"";

#[test]
fn bang_in_gives_the_results_of_not_in_over_lists_classes_and_strings() {
    let items = [
        "1",
        "2",
        "\"a\"",
        "\"1\"",
        "\"\"",
        "null",
        "true",
        "false",
        "[1, 2]",
        "[2, 1]",
        "{x: 1}",
        "{x: 2}",
        "Number.NaN",
        "seat",
        "other",
        "{id: \"seat\", row: 1}",
        "{row: 1}",
    ];
    let collections = ["mixed", "empty", "Seat", "text", "blank", "[seat]"];
    let mut compared = 0;
    for item in items {
        for collection in collections {
            let source = format!(
                "{SETUP}\n[{item} !in {collection}, {item} not in {collection}, !{collection}.includes({item})]"
            );
            let value: serde_json::Value = serde_json::from_str(&result(&source)).unwrap();
            assert_eq!(value[0], value[1], "{item} !in {collection}");
            assert_eq!(value[0], value[2], "{item} !in {collection}");
            assert!(value[0].is_boolean(), "{item} !in {collection}");
            compared += 1;
        }
    }
    assert_eq!(compared, 102);
    assert_eq!(
        result(&format!(
            "{SETUP}\n[seat !in Seat, {{row: 1}} !in Seat, 1 !in \"a1\", \"\" !in blank, Number.NaN !in [Number.NaN], \"z\" !in \"hello\"]"
        )),
        "[false,true,false,false,true,true]"
    );
}

const LIVE: &str = "tags = [\"red\"]\nmissing = \"green\" !in tags\nassert(missing, true)\ntags.push(\"green\")\nassert(missing, false)\ntags = [\"blue\"]\nassert(missing, true)\nclass Guest\nuninvited = {id: \"ben\"} !in Guest\nassert(uninvited, true)\nben = Guest()\nassert(uninvited, false)\ndelete ben\nassert(uninvited, true)\nclass Order(status):\n    this.status = status\n$Order.pending = $Order.status !in [\"paid\", \"shipped\"]\nif $Order.status !in [\"new\", \"paid\"]:\n    $Order.flagged = true\nfirst = Order(\"new\")\nsecond = Order(\"lost\")\nassert([first.pending, second.pending, first.flagged, second.flagged], [true, true, null, true])\nfirst.status = \"paid\"\nassert(first.pending, false)\nsecond.status = \"shipped\"\nassert(second.pending, false)\nr = 0\nif \"green\" !in tags:\n    r = 1\nassert(r, 1)\nr";

#[test]
fn bang_in_rules_stay_live_and_read_what_not_in_reads() {
    let (written, rewrites) = spelled(LIVE);
    assert_eq!(rewrites, 5);
    assert_eq!(checked(LIVE, "1"), checked(&written, "1"));
    for (setup, alias, spelling) in [
        ("tags = [1]\nitem = 1", "item !in tags", "item not in tags"),
        ("word = \"abc\"", "\"b\" !in word", "\"b\" not in word"),
        (
            "class Seat\nseat = Seat()",
            "seat !in Seat",
            "seat not in Seat",
        ),
        (
            "class Box\nbox = Box()\nbox.tags = [1]",
            "1 !in box.tags",
            "1 not in box.tags",
        ),
        ("tags = [1]", "1 !in tags.value", "1 not in tags.value"),
    ] {
        let mut left = Runtime::new();
        left.run(&format!("{setup}\nknown = {alias}")).unwrap();
        let mut right = Runtime::new();
        right.run(&format!("{setup}\nknown = {spelling}")).unwrap();
        assert_eq!(
            dependencies(&left, "known"),
            dependencies(&right, "known"),
            "{alias}"
        );
    }
    assert_eq!(
        result(
            "tags = [\"b\"]\nknown = \"a\" !in tags.value\ntags.push(\"a\")\n[known, \"a\" !in tags]"
        ),
        "[true,false]"
    );
}

#[test]
fn null_and_undefined_collections_wait_as_for_not_in() {
    assert_eq!(
        result("n = null\ndef g():\n    pass\no = {a: 1}\n[1 !in n, 1 !in o.missing, 1 !in g()]"),
        "[null,null,null]"
    );
    assert_eq!(
        result(
            "tags = null\nunseen = \"a\" !in tags\nassert(unseen, null)\ntags = [\"a\"]\nunseen"
        ),
        "false"
    );
    assert_eq!(
        result(
            "class Item\n$Item.cold = \"red\" !in $Item.tags\ncart = Item()\nassert(cart.cold, null)\ncart.tags = [\"blue\"]\nassert(cart.cold, true)\ncart.tags.push(\"red\")\ncart.cold"
        ),
        "false"
    );
    assert_eq!(result("n = null\nr = 0\nif 1 !in n:\n    r = 1\nr"), "0.0");
}

#[test]
fn other_collections_raise_the_type_error_of_not_in_and_roll_back() {
    for (setup, expression, collection) in [
        ("count = 5", "1 !in count", "count"),
        ("flag = true", "1 !in flag", "flag"),
        ("record = {a: 1}", "\"a\" !in record", "record"),
        ("class Seat\nseat = Seat()", "1 !in seat", "seat"),
        ("f = v => v", "1 !in f", "f"),
        ("d = Date(0)", "1 !in d", "d"),
        ("r = /a/", "1 !in r", "r"),
        ("xs = [1]", "1 !in xs.length", "xs.length"),
        ("x = Math", "1 !in x", "x"),
        ("pass", "\"a\" !in {a: 1}", "{\"a\":1}"),
        ("pass", "1 !in Math", "Math"),
        ("pass", "1 !in Class", "Class"),
        ("pass", "1 !in 5", "5"),
        ("pass", "1 !in true", "true"),
        ("xs = [1]", "1 in (2 !in xs)", "(2 not in xs)"),
    ] {
        let line = setup.lines().count() + 1;
        let source = format!("{setup}\n{expression}");
        assert_eq!(
            failure(&source),
            (
                ErrorKind::Type,
                format!("{collection} is not a collection"),
                Some(Position::new(line, 1))
            ),
            "{expression}"
        );
        assert_eq!(
            failure(&source),
            failure(&spelled(&source).0),
            "{expression}"
        );
    }
    assert_eq!(
        failure("class Order\n$Order.ok = 1 !in $Order\norder = Order()"),
        (
            ErrorKind::Type,
            "Order is not a collection".to_string(),
            Some(Position::new(3, 1))
        )
    );
    let mut runtime = Runtime::new();
    runtime.run("items = [1]\nabsent = 2 !in items").unwrap();
    let error = runtime.run("items = 7").unwrap_err();
    assert_eq!(error.message(), "items is not a collection");
    let kept = runtime.run("[items, absent]").unwrap();
    assert_eq!(runtime.serialize_json(&kept).unwrap(), "[[1.0],true]");
    let mut rules = Runtime::new();
    rules
        .run("class Order(status):\n    this.status = status\napproved = [\"paid\"]\n$Order.pending = $Order.status !in approved\nfirst = Order(\"new\")")
        .unwrap();
    let error = rules.run("approved = 5").unwrap_err();
    assert_eq!(
        (error.kind(), error.message()),
        (ErrorKind::Type, "approved is not a collection".to_string())
    );
    let kept = rules.run("[approved, first.pending]").unwrap();
    assert_eq!(rules.serialize_json(&kept).unwrap(), "[[\"paid\"],true]");
    assert_eq!(
        result("class Math\nm = Math()\n[m !in Math, m not in Math]"),
        "[false,false]"
    );
}

#[test]
fn bang_and_in_apart_keep_their_accepted_errors() {
    for (source, message, line, column) in [
        ("ok = 4 ! in xs", "Unexpected keyword 'in'", 1, 13),
        ("ok = 4 !\tin xs", "Unexpected keyword 'in'", 1, 13),
        ("ok = 4 !  in xs", "Unexpected keyword 'in'", 1, 14),
        ("ok = 4 !\u{c}in xs", "Unexpected keyword 'in'", 1, 13),
        ("ok = 4 !\rin xs", "Unexpected keyword 'in'", 1, 13),
        ("ok = 4 !\nin xs", "Unexpected end of line", 2, 1),
        ("ok = 4 !\r\nin xs", "Unexpected end of line", 2, 1),
        ("ok = 4 !\n    in xs", "Unexpected end of line", 2, 5),
        ("ok = 4 !/* c */in xs", "Unexpected keyword 'in'", 1, 19),
        ("ok = 4 !/**/in xs", "Unexpected keyword 'in'", 1, 16),
        ("ok = 4 !# c\nin xs", "Unexpected end of line", 2, 1),
        ("ok = 4 !// c\nin xs", "Unexpected end of line", 2, 1),
        ("ok = [4 !\nin xs]", "Expected ']' but found '!'", 1, 9),
        (
            "ok = [4 !\n         in xs]",
            "Expected ']' but found '!'",
            1,
            9,
        ),
        ("ok = [4 ! in xs]", "Expected ']' but found '!'", 1, 9),
        (
            "ok = (4 !\n         in xs)",
            "Expected ')' but found '!'",
            1,
            9,
        ),
        ("ok = 4 !!in xs", "Unexpected keyword 'in'", 1, 13),
        ("ok = 4 !not in xs", "Unexpected keyword 'in'", 1, 16),
        ("ok = 4 not !in xs", "Unexpected keyword 'in'", 1, 16),
        ("ok = 4 !=in xs", "Unexpected keyword 'in'", 1, 13),
        ("ok is !in xs", "Unexpected keyword 'in'", 1, 11),
        (
            "y = 1 if 4 ! in xs else 2",
            "Expected ':' but found '!'",
            1,
            12,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source:?}"
        );
    }
    let aligned = "ok = [4 !\n         in xs]";
    let tokens = tokenize(aligned).unwrap();
    let bang = tokens
        .iter()
        .position(|spanned| spanned.token == Token::Bang)
        .unwrap();
    assert_eq!(tokens[bang + 1].token, Token::Keyword(Keyword::In));
    assert_eq!(
        tokens[bang + 1].position.column,
        tokens[bang].position.column + 1
    );
    assert_ne!(tokens[bang + 1].position.line, tokens[bang].position.line);
}

#[test]
fn bang_keeps_its_prefix_reading_and_in_keeps_its_other_readings() {
    assert_eq!(result("inx = false\nx = 1\nx !inx"), "true");
    assert_eq!(result("inx = false\n[!inx]"), "[true]");
    assert_eq!(
        statements("x !inx"),
        vec![
            Stmt::Expression(name("x")),
            Stmt::Expression(Expr::Unary {
                operator: UnaryOp::Not,
                operand: Box::new(name("inx")),
            }),
        ]
    );
    assert_eq!(
        result("xs = [1, 2, 3]\n[!(4 in xs), !4 in xs, 1 != 2, 1 !== 1, \"a !in b\"]"),
        "[true,false,true,false,\"a !in b\"]"
    );
    assert_eq!(
        result("xs = [1]\nx = 1\n[(x)in xs, (2)in xs, [1]in [[1]], (x)not in xs]"),
        "[true,false,true,false]"
    );
    for (source, message, line, column) in [
        ("ok = !in xs", "Unexpected keyword 'in'", 1, 10),
        (
            "for x !in xs:\n    pass",
            "Expected 'of' in for statement",
            1,
            7,
        ),
        ("o = {in: 1}", "Expected a key but found keyword 'in'", 1, 8),
        ("a, b = 4 !in xs, 2", "Unexpected ','", 1, 4),
        ("{a: 4 !in xs}", "Unexpected ':'", 1, 5),
        ("[1,in xs]", "Unexpected keyword 'in'", 1, 7),
        ("ok =in xs", "Unexpected keyword 'in'", 1, 8),
        ("c ?in xs : 1", "Unexpected keyword 'in'", 1, 7),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source:?}"
        );
    }
    assert_eq!(
        result(
            "record = {a: 1}\nrecord.in = [2]\nxs = [1, 2]\n[1 !in record.in, all v in xs: v !in [3], [v for v in xs if v !in record.in]]"
        ),
        "[true,true,[1.0]]"
    );
    assert!(matches!(
        statements("for x in xs:\n    pass").as_slice(),
        [Stmt::For { variable, .. }] if variable == "x"
    ));
    assert!(matches!(
        statements("for k, v in record:\n    pass").as_slice(),
        [Stmt::ForEntries { key, value, .. }] if key == "k" && value == "v"
    ));
}

#[test]
fn ignored_tails_keep_their_accepted_readings() {
    assert_eq!(
        result(
            "x = 5\nxs = [5]\ndef f(v) => v\nc = true\n[`a ${x !in xs} b`, `${x !in xs: 1}`, `${(x !in xs)}`, `${[x !in xs]}`, `${f(x !in xs)}`, `${c ? x !in xs : 0}`, `${(v => v !in xs)(4)}`]"
        ),
        "[\"a 5 b\",\"5\",\"false\",\"false\",\"false\",\"false\",\"true\"]"
    );
    for source in ["x !in xs", "x !in xs: v", "x !in", "x ! in xs"] {
        assert_eq!(parse_expression(source).unwrap(), name("x"), "{source}");
    }
    assert_eq!(
        parse_expression("(x !in xs)").unwrap(),
        negated(name("x"), name("xs"))
    );
    assert_eq!(
        parse_expression("[x !in xs]").unwrap(),
        Expr::List(vec![negated(name("x"), name("xs"))])
    );
    assert_eq!(
        parse_expression("a if x !in xs else b").unwrap(),
        parse_expression("(x not in xs) ? a : b").unwrap()
    );
    let error = parse_expression("A if c !in q else (B").unwrap_err();
    assert_eq!(
        (error.message(), error.position()),
        (
            "Expected ')' but found end of input".to_string(),
            Some(Position::new(1, 21))
        )
    );
    assert_eq!(
        parse_expression("a if x ! in xs else b").unwrap(),
        name("a")
    );
}

#[test]
fn malformed_bang_in_tests_report_where_the_parser_stops() {
    for (source, message, line, column) in [
        ("ok = x !in", "Unexpected end of input", 1, 11),
        ("ok = x !in\n", "Unexpected end of line", 2, 1),
        ("ok = (x !in )", "Unexpected ')'", 1, 14),
        ("ok = [x !in ]", "Unexpected ']'", 1, 14),
        ("ok = x !in in xs", "Unexpected keyword 'in'", 1, 15),
        ("if x !in:", "Unexpected ':'", 1, 10),
        ("if x !in:\n    r = 1", "Unexpected ':'", 1, 9),
        ("ok = x !in xs: v", "Unexpected keyword 'in'", 1, 12),
        ("ok = all 1 !in xs: v", "Unexpected keyword 'in'", 1, 16),
        ("ok = (1 !in xs: v)", "Expected ')' but found '!'", 1, 9),
        ("ok = {a: 1 !in xs: 2}", "Expected '}' but found '!'", 1, 12),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source:?}"
        );
    }
    for source in [
        "if x !in xs: r = 1",
        "if x !in xs:\n    r = 1",
        "while x !in xs: xs.push(x)",
        "ok = c ? x !in xs : 1",
        "ok = ys[x !in xs:]",
        "ok = all v in a !in b: v",
    ] {
        assert!(writes_bang_in(source), "{source}");
        assert_eq!(
            statements(source),
            statements(&source.replace("!in", "not in")),
            "{source}"
        );
    }
}

fn chained(operator: &str, count: usize) -> Result<(), String> {
    let tail = format!(" {operator} b").repeat(count);
    Runtime::check(&format!("ok = a{tail}")).map_err(|error| error.message())
}

fn deepest(opener: &str, base: &str, closer: &str) -> usize {
    let started = Instant::now();
    for depth in 1..200 {
        let source = format!(
            "ok = {}{base}{}",
            opener.repeat(depth),
            closer.repeat(depth)
        );
        let checked = Runtime::check(&source);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{opener} at depth {depth}"
        );
        if let Err(error) = checked {
            assert_eq!(
                error.message(),
                "Expressions are nested too deeply",
                "{opener} at depth {depth}"
            );
            return depth;
        }
    }
    panic!("{opener} never reaches the nesting limit")
}

#[test]
fn bang_in_keeps_the_chain_and_nesting_limits_of_not_in() {
    for count in [127, 128, 129] {
        assert_eq!(chained("!in", count), chained("not in", count), "{count}");
        assert_eq!(chained("!in", count), chained("<", count), "{count}");
    }
    assert_eq!(chained("!in", 128), Ok(()));
    assert_eq!(
        chained("!in", 129),
        Err("Expression has too many operands".to_string())
    );
    let mixed = format!("ok = a{}{}", " not in b".repeat(64), " !in b".repeat(65));
    assert_eq!(
        Runtime::check(&mixed).unwrap_err().message(),
        "Expression has too many operands"
    );
    for [alias, spelling] in [
        [("x !in (", "x", ")"), ("x not in (", "x", ")")],
        [("[x !in ", "x", "]"), ("[x not in ", "x", "]")],
        [("f(x !in ", "x", ")"), ("f(x not in ", "x", ")")],
        [
            ("(x !in xs) !in (", "x", ")"),
            ("(x not in xs) not in (", "x", ")"),
        ],
    ] {
        let limit = deepest(alias.0, alias.1, alias.2);
        assert!(limit > 15, "{alias:?} stops at {limit}");
        assert_eq!(
            limit,
            deepest(spelling.0, spelling.1, spelling.2),
            "{alias:?}"
        );
    }
}

#[test]
fn explanations_graph_keys_and_rules_render_not_in() {
    for (source, expected) in [
        ("ok = x !in xs", "ok=(x not in xs)"),
        ("ok = !(x !in xs)", "ok=!(x not in xs)"),
        ("ok = (a !in b) !in c", "ok=((a not in b) not in c)"),
        ("ok = a < b !in c", "ok=(a<b not in c)"),
        ("ok = (a !in b) ** 2", "ok=(a not in b)**2"),
        ("ok = `${(a !in b)}`", "ok=`${(a not in b)}`"),
    ] {
        assert_eq!(rendered(source), expected, "{source}");
        assert_eq!(
            rendered(source),
            rendered(&source.replace("!in", "not in")),
            "{source}"
        );
        assert_eq!(
            statements(&rendered(source)),
            statements(source),
            "{source}"
        );
    }
    assert_eq!(
        result("tags = [\"a\"]\nitem = \"b\"\nknown = item !in tags\n(why known).map(s => s.rule)"),
        "[\"known = (item not in tags)\",\"item = \\\"b\\\"\",\"tags = [\\\"a\\\"]\"]"
    );
    assert_eq!(
        result("tags = [\"a\"]\nknown = \"a\" !in tags\n(affects tags).map(s => s.node)"),
        "[\"known\"]"
    );
    let mut runtime = Runtime::new();
    runtime
        .run("xs = [1]\nif 2 !in xs:\n    a = 1\nif 2 not in xs:\n    b = 1")
        .unwrap();
    let alias = NodeKey::conditional(&parse_expression("(2 !in xs)").unwrap(), None);
    assert_eq!(
        alias,
        NodeKey::conditional(&parse_expression("(2 not in xs)").unwrap(), None)
    );
    assert!(runtime.graph.retrieve(&alias).is_some(), "{alias:?}");
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-negated-membership-{label}-{}.nuc",
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

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap()
}

#[test]
fn files_graphs_and_repl_sessions_run_bang_in() {
    let program = "blocked = [\"eve\"]\nclass User(name):\n    this.name = name\n$User.allowed = $User.name !in blocked\nbob = User(\"bob\")\neve = User(\"eve\")\nopen = 4 !in [1, 2]\nif \"bob\" !in blocked:\n    greeting = \"hi\"\nblocked.push(\"bob\")\n[bob.allowed, eve.allowed, open, greeting]";
    let (written, rewrites) = spelled(program);
    assert_eq!(rewrites, 3);
    let file = SourceFile::new("file");
    for options in [
        &[][..],
        &["--json"][..],
        &["--graph"][..],
        &["--json", "--graph"][..],
    ] {
        let alias = file.run(program, options);
        let spelling = file.run(&written, options);
        assert!(alias.status.success(), "{options:?} {alias:?}");
        assert!(alias.stderr.is_empty(), "{options:?} {alias:?}");
        assert_eq!(alias.stdout, spelling.stdout, "{options:?}");
    }
    assert_eq!(
        text(&file.run(program, &["--json"]).stdout),
        "[false,false,true,\"hi\"]\n"
    );
    let graph = file.run(program, &["--graph"]);
    let graph = text(&graph.stdout);
    assert!(graph.contains("  open [variable]\n"), "{graph}");
    assert!(
        graph.contains("if((\"bob\" not in blocked)) [if]\n    reads    blocked"),
        "{graph}"
    );
    assert!(!graph.contains("!in"), "{graph}");
    for (source, expected) in [
        (
            "record = {a: 1}\n\"a\" !in record",
            "TypeError: record is not a collection (line 2, column 1)\n",
        ),
        (
            "ok = x !in",
            "SyntaxError: Unexpected end of input (line 1, column 11)\n",
        ),
        (
            "ok = 4 ! in xs",
            "SyntaxError: Unexpected keyword 'in' (line 1, column 13)\n",
        ),
    ] {
        let output = file.run(source, &["--json"]);
        assert!(!output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert_eq!(text(&output.stderr), expected, "{source}");
    }
    let session = "xs = [1, 2]\nif 3 !in xs\n    r = 1\n\nr\ntags = [\"a\"]\nknown = \"b\" !in tags\nknown\ntags.push(\"b\")\nknown\nok = 4 ! in xs\n.exit\n";
    let (written_session, session_rewrites) = spelled(session);
    assert_eq!(session_rewrites, 2);
    for options in [&[][..], &["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let alias = repl(&session.replace('\n', newline), options);
            let spelling = repl(&written_session.replace('\n', newline), options);
            assert!(alias.status.success(), "{options:?} {alias:?}");
            assert_eq!(alias.stdout, spelling.stdout, "{options:?}");
            assert_eq!(alias.stderr, spelling.stderr, "{options:?}");
            assert_eq!(
                text(&alias.stderr),
                "SyntaxError: Unexpected keyword 'in' (line 1, column 13)\n",
                "{options:?}"
            );
            let expected = if options.contains(&"--json") {
                "[1.0,2.0]\n1.0\n1.0\n[\"a\"]\ntrue\ntrue\n2.0\nfalse\n"
            } else {
                "1,2\n1\n1\na\ntrue\ntrue\n2\nfalse\n"
            };
            assert_eq!(
                text(&alias.stdout),
                format!("{BANNER}{expected}"),
                "{options:?}"
            );
        }
    }
}

#[derive(Deserialize)]
struct Record {
    id: String,
    description: String,
    code: String,
    #[serde(rename = "return")]
    returns: Option<String>,
}

#[test]
fn the_synthesized_records_run_their_assertions_and_returns() {
    let records: Vec<Record> = include_str!("../dataset/nucleoid.synth.96.jsonl")
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(records.len(), 12);
    for (index, record) in records.iter().enumerate() {
        assert_eq!(record.id, format!("synth-96-{:04}", index + 1));
        assert!(!record.description.trim().is_empty(), "{}", record.id);
        assert!(writes_bang_in(&record.code), "{}", record.id);
        let expected = record.returns.as_deref().unwrap();
        let value = checked(&record.code, expected);
        let (spelling, _) = spelled(&record.code);
        assert_eq!(value, checked(&spelling, expected), "{}", record.id);
    }
}
