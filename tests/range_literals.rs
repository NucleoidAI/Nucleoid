mod common;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::lang::ast::generator::generate;
use nucleoid::lang::ast::lexer::{Keyword, Spanned, Token, tokenize};
use nucleoid::lang::ast::operator::binary;
use nucleoid::lang::ast::{BinaryOp, Expr, Stmt, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde_json::{Value as Json, json};

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid builds half-open and closed integer ranges",
    "Nucleoid keeps class-level integer range rules live",
    "Nucleoid maps integer ranges to positions and coordinates",
    "Nucleoid rolls back failed integer range updates",
];

const REFERENCE_TITLE: &str = "Nucleoid writes consecutive integers as range literals";
const README_TITLE: &str = "Ranges keep a shift's hours in step with its bounds";
const EXAMPLES_TITLE: &str = "Ranges keep every elevator's stops live";

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

const MEMBER: &str = "Expected a property but found '.'";

const BOUNDS: &str = "Range bounds must be integers";

const TOO_LARGE: &str = "Range is too large";

struct Counterpart {
    title: &'static str,
    rewrites: &'static [(&'static str, &'static str)],
    before: usize,
    after: usize,
}

const SPELLED: [Counterpart; 7] = [
    Counterpart {
        title: SPEC_TITLES[0],
        rewrites: &[
            (
                "floors = lowest..=highest\n",
                "floors = [\n    -2, -1\n    0\n    1\n    2\n    3\n    4\n    5\n][lowest + 2:highest + 3]\n",
            ),
            ("basement = lowest..<0\n", "basement = floors[:-lowest]\n"),
            ("upper = 1..=highest\n", "upper = floors[1 - lowest:]\n"),
        ],
        before: 193,
        after: 173,
    },
    Counterpart {
        title: SPEC_TITLES[1],
        rewrites: &[(
            "$Shift.hours = $Shift.start..<$Shift.end\n",
            "$Shift.hours = [\n    6\n    7\n    8\n    9\n    10\n    11\n    12\n    13\n    14\n    15\n    16\n    17\n    18\n    19\n    20\n    21\n    22\n][$Shift.start - 6:$Shift.end - 6]\n",
        )],
        before: 244,
        after: 221,
    },
    Counterpart {
        title: SPEC_TITLES[2],
        rewrites: &[
            (
                "columns = 3\n",
                "xs = [\n    0\n    1\n    2\n    3\n    4\n    5\n    6\n    7\n    8\n    9\n    10\n    11\n    12\n]\ncolumns = 3\n",
            ),
            (
                "cells = (0..<columns * rows).map(",
                "cells = xs[:columns * rows].map(",
            ),
            (
                "labels = (1..=seats).map(n -> row + n)\n",
                "labels = xs[1:seats + 1].map(n -> row + n)\n",
            ),
        ],
        before: 227,
        after: 210,
    },
    Counterpart {
        title: SPEC_TITLES[3],
        rewrites: &[
            (
                "first = 1\n",
                "def count(n):\n    if n % 1: throw TypeError(\"Range bounds must be integers\")\n    if n > 100000: throw TypeError(\"Range is too large\")\n    return n\nfirst = 1\n",
            ),
            (
                "days = first..=last\n",
                "days = [\n    1\n    2\n    3\n    4\n    5\n    6\n    7\n][:count(last - first + 1)]\n",
            ),
        ],
        before: 242,
        after: 193,
    },
    Counterpart {
        title: REFERENCE_TITLE,
        rewrites: &[
            (
                "exclusive = first..<last\n",
                "exclusive = [\n    -1\n    0\n    1\n    2\n    3\n    4\n][first + 1:last + 1]\n",
            ),
            (
                "inclusive = first..=last\n",
                "inclusive = first > last ? [] : exclusive.concat(last)\n",
            ),
        ],
        before: 147,
        after: 126,
    },
    Counterpart {
        title: README_TITLE,
        rewrites: &[(
            "hours = open..<close\n",
            "hours = [\n    9\n    10\n    11\n    12\n    13\n    14\n][:close - 9]\n",
        )],
        before: 66,
        after: 57,
    },
    Counterpart {
        title: EXAMPLES_TITLE,
        rewrites: &[(
            "$Elevator.stops = $Elevator.lowest..=$Elevator.highest\n",
            "$Elevator.stops = [\n    -1\n    0\n    1\n    2\n    3\n    4\n    5\n    6\n    7\n    8\n    9\n    10\n    11\n    12\n][$Elevator.lowest + 1:$Elevator.highest + 2]\n",
        )],
        before: 169,
        after: 148,
    },
];

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "integer range regression".to_string(),
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

fn failure(source: &str) -> (ErrorKind, String, Option<Position>) {
    failure_in(&mut Runtime::new(), source)
}

fn failure_in(runtime: &mut Runtime, source: &str) -> (ErrorKind, String, Option<Position>) {
    let error = runtime.run(source).expect_err("the program must fail");
    (error.kind(), error.message(), error.position())
}

fn error(
    kind: ErrorKind,
    message: &str,
    line: usize,
    column: usize,
) -> (ErrorKind, String, Option<Position>) {
    (kind, message.to_string(), Some(Position::new(line, column)))
}

fn syntax_error(source: &str) -> (String, Option<Position>) {
    let error = parse(source).expect_err("the source must not parse");
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position())
}

fn at(message: &str, line: usize, column: usize) -> (String, Option<Position>) {
    (message.to_string(), Some(Position::new(line, column)))
}

fn tokens(source: &str) -> usize {
    tokenize(source.trim())
        .unwrap()
        .iter()
        .filter(|spanned| spanned.token != Token::Eof)
        .count()
}

fn touches(first: &Spanned, second: &Spanned) -> bool {
    second.position.line == first.position.line
        && second.position.column == first.position.column + 1
}

fn writes_range(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(3).any(|window| {
            window[0].token == Token::Dot
                && window[1].token == Token::Dot
                && matches!(window[2].token, Token::Less | Token::Assign)
                && touches(&window[0], &window[1])
                && touches(&window[1], &window[2])
        }) || tokens.iter().any(|spanned| {
            matches!(&spanned.token, Token::Template(raw) if raw.contains("..<") || raw.contains("..="))
        })
    })
}

fn writes_bang_in(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(2).any(|pair| {
            pair[0].token == Token::Bang
                && pair[1].token == Token::Keyword(Keyword::In)
                && touches(&pair[0], &pair[1])
        })
    })
}

fn writes_xor_stage(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(3).any(|window| {
            window[0].token == Token::Pipe
                && window[1].token == Token::Identifier("xor".to_string())
                && window[2].token == Token::LeftParen
        })
    })
}

fn ranges(source: &str) -> usize {
    let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    format!("{statements:?}").matches("operator: Range").count()
}

fn expression(source: &str) -> Expr {
    parse_expression(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn value_of(source: &str) -> Expr {
    match parse(source).unwrap().pop() {
        Some(Stmt::Assign { value, .. }) => value,
        other => panic!("{source} ends with {other:?}"),
    }
}

fn range(operator: BinaryOp, start: Expr, end: Expr) -> Expr {
    Expr::Binary {
        operator,
        left: Box::new(start),
        right: Box::new(end),
    }
}

fn name(text: &str) -> Expr {
    Expr::Identifier(text.to_string())
}

fn numbers(items: impl IntoIterator<Item = i32>) -> Json {
    Json::Array(
        items
            .into_iter()
            .map(|item| json!(f64::from(item)))
            .collect(),
    )
}

fn cli(label: &str, arguments: &[&str], source: &str) -> Output {
    let directory =
        std::env::temp_dir().join(format!("nucleoid-nuc148-{label}-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let file = directory.join("program.nuc");
    fs::write(&file, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(arguments)
        .arg(&file)
        .output()
        .unwrap();
    fs::remove_dir_all(&directory).unwrap();
    output
}

fn repl(arguments: &[&str], source: &str) -> Output {
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
        .write_all(source.as_bytes())
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
fn appended_document_cases_run_every_assertion() {
    let [spec, reference, readme, examples] = documents();
    let appended = SPEC_TITLES
        .iter()
        .zip([Some("6"), Some("12"), Some("12"), Some("28")])
        .map(|(title, expected)| (&spec, *title, expected))
        .chain([
            (&reference, REFERENCE_TITLE, Some("[-1,0,1,2,3]")),
            (&readme, README_TITLE, Some("[9,10,11,12,13,14]")),
            (&examples, EXAMPLES_TITLE, Some("[\"high\"]")),
        ]);

    let mut assertions = 0;
    for (cases, title, expected) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(ranges(&case.source) > 0, "{title} writes a range");
        assert_eq!(case.expected.as_deref(), expected, "{title}");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 28);
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Integer range literals\n\n```\n")
        .expect("the language reference documents ranges");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(case.expected.as_deref(), Some("[-1,0,1,2,3]"));
    for line in [
        "Full detail: [NUC 148](nuc-0148.md).",
        "start..<end                            the list of integers from start up to, not including, end (NUC 148)",
        "start..=end                            the list of integers from start through end; between comparisons and + (NUC 148)",
        "| A range of more than 100,000 integers, raised before any item is built | `Range is too large` |",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    let negated = reference.find("### Negated membership with !in").unwrap();
    let ranges = reference.find("### Integer range literals").unwrap();
    let indexing = reference.find("### Indexing, slicing, and length").unwrap();
    assert!(negated < ranges && ranges < indexing);
    assert!(include_str!("../README.md").contains("[NUC 148](docs/nuc-0148.md)"));
    assert!(include_str!("../docs/examples.md").contains("[NUC 148](nuc-0148.md)"));
    let spec = include_str!("../nucleoid.spec.md").replace("\r\n", "\n");
    let contract = spec.find("\n## Integer range literals\n").unwrap();
    assert!(spec.find("\n## Stepped slices\n").unwrap() < contract);
    assert!(
        contract
            < spec
                .find("\n## Object entry loops with ordered rollback\n")
                .unwrap()
    );
}

#[test]
fn no_other_document_case_writes_a_range() {
    let mut cases = 0;
    let mut with_range = 0;
    for case in documents().iter().flatten() {
        cases += 1;
        let writes = writes_range(&case.source);
        assert_eq!(writes, is_appended(&case.title), "{}", case.title);
        with_range += usize::from(writes);
    }
    assert_eq!(with_range, 7);
    assert_eq!(cases, 1347);
}

#[test]
fn the_audit_finds_adjacent_range_operators_and_nothing_else() {
    for source in [
        "x = 0..<3",
        "x = a ..= b",
        "x = [f(0..<2)]",
        "`${0..<3}`",
        "`a..=b`",
        "x = (0..<3).length",
    ] {
        assert!(writes_range(source), "{source}");
    }
    for source in [
        "x = 0. .<3",
        "x = 0.. <3",
        "x = 0..<=3",
        "x = 0..==3",
        "x = 0..=>3",
        "x = 0...3",
        "x = 1..2",
        "x = \"0..<3\"",
        "x = 1 # 0..<3",
        "x = a.b",
        "`${x}.<3`",
    ] {
        assert!(!writes_range(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_together_before_the_later_cases() {
    let [spec, reference, readme, examples] = documents();
    for (cases, own) in [
        (&spec, &SPEC_TITLES[..]),
        (&reference, &[REFERENCE_TITLE][..]),
        (&readme, &[README_TITLE][..]),
        (&examples, &[EXAMPLES_TITLE][..]),
    ] {
        let first = cases
            .iter()
            .position(|case| case.title == own[0])
            .unwrap_or_else(|| panic!("{} is appended", own[0]));
        for (offset, title) in own.iter().enumerate() {
            assert_eq!(
                cases.get(first + offset).map(|case| case.title.as_str()),
                Some(*title),
                "{title} stands with the other appended cases"
            );
        }
        let xor = cases
            .iter()
            .position(|case| writes_xor_stage(&case.source))
            .expect("the cases of NUC 139 stand in every document");
        let bang_in = cases
            .iter()
            .position(|case| writes_bang_in(&case.source))
            .expect("the cases of NUC 135 stand in every document");
        assert!(first + own.len() <= xor, "{}", own[0]);
        assert!(first + own.len() <= bang_in, "{}", own[0]);
    }
}

#[test]
fn every_case_is_its_frozen_counterpart_with_only_the_ranges_written_out() {
    let documents = documents();
    let mut before = 0;
    let mut after = 0;
    for Counterpart {
        title,
        rewrites,
        before: spelled_tokens,
        after: range_tokens,
    } in SPELLED
    {
        let case = documents
            .iter()
            .flatten()
            .find(|case| case.title == title)
            .unwrap();
        let mut counterpart = case.source.clone();
        for (written, spelling) in rewrites {
            assert_eq!(
                counterpart.matches(written).count(),
                1,
                "{title}: {written}"
            );
            counterpart = counterpart.replacen(written, spelling, 1);
        }
        assert!(writes_range(&case.source), "{title}");
        assert!(!writes_range(&counterpart), "{title}");
        assert_eq!(tokens(&case.source), range_tokens, "{title}");
        assert_eq!(tokens(&counterpart), spelled_tokens, "{title}");
        assert!(range_tokens < spelled_tokens, "{title}");
        let assertions = common::assertions::protected_assertions(&parse(&case.source).unwrap());
        assert_eq!(
            assertions,
            common::assertions::protected_assertions(&parse(&counterpart).unwrap()),
            "{title}"
        );
        for source in [&case.source, &counterpart] {
            common::check_with_assertions(
                &common::Case {
                    title: title.to_string(),
                    source: source.clone(),
                    expected: case.expected.clone(),
                },
                assertions.len(),
            )
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        }
        before += spelled_tokens;
        after += range_tokens;
    }
    assert_eq!((before, after), (1_288, 1_128));
}

#[test]
fn ranges_parse_to_binary_expressions_between_comparisons_and_sums() {
    assert_eq!(
        expression("0..<n"),
        range(BinaryOp::Range, Expr::Number(0.0), name("n"))
    );
    assert_eq!(
        expression("a ..= b"),
        range(BinaryOp::RangeInclusive, name("a"), name("b"))
    );
    assert_eq!(BinaryOp::Range.as_str(), "..<");
    assert_eq!(BinaryOp::RangeInclusive.as_str(), "..=");
    for spaced in ["a ..< b", "a..< b", "a ..<b", "(a..<\nb)"] {
        assert_eq!(expression(spaced), expression("a..<b"), "{spaced}");
    }
    for (source, grouped) in [
        ("0..<n + 1", "0..<(n + 1)"),
        ("a - 1..=b * 2", "(a - 1)..=(b * 2)"),
        ("-1..<2", "(-1)..<2"),
        ("a ** 2..=b", "(a ** 2)..=b"),
        ("a < 0..<3", "a < (0..<3)"),
        ("0..<3 == xs", "(0..<3) == xs"),
        ("c ? 0..<a : 1..=b", "c ? (0..<a) : (1..=b)"),
        ("x ?? 0..<3", "x ?? (0..<3)"),
        ("x && 0..<3 || y", "(x && (0..<3)) || y"),
        ("a.b..<c.d", "(a.b)..<(c.d)"),
        ("f(a)..<g(b)", "(f(a))..<(g(b))"),
        ("xs[0]..<xs[1]", "(xs[0])..<(xs[1])"),
        ("$C.a..<this.b", "($C.a)..<(this.b)"),
        ("not a..<b", "(not a)..<b"),
        ("sum of xs..<3", "(sum of xs)..<3"),
        ("why x..<3", "(why x)..<3"),
        ("x..<3 |> why", "(x..<3) |> why"),
    ] {
        assert_eq!(expression(source), expression(grouped), "{source}");
    }
    for (source, grouped) in [
        ("y = x in 1..=5", "y = x in (1..=5)"),
        ("y = x !in 0..<n", "y = x !in (0..<n)"),
        ("y = x not in 0..<n", "y = x not in (0..<n)"),
        ("y = 0..<3 if c else 1..=2", "y = (0..<3) if c else (1..=2)"),
        (
            "y = [i for i in 0..<n if i > 1]",
            "y = [i for i in (0..<n) if i > 1]",
        ),
        ("y = all i in 0..<n: i < n", "y = all i in (0..<n): i < n"),
    ] {
        assert_eq!(value_of(source), value_of(grouped), "{source}");
    }
}

#[test]
fn a_range_does_not_chain_and_needs_both_operands() {
    assert_eq!(syntax_error("x = a..<b..<c"), at("Unexpected '.'", 1, 10));
    assert_eq!(syntax_error("x = a..=b ..= c"), at("Unexpected '.'", 1, 11));
    assert_eq!(syntax_error("x = (a..<b..<c)"), at("Unexpected '.'", 1, 11));
    assert_eq!(syntax_error("f(a..<b..<c)"), at("Unexpected '.'", 1, 8));
    assert_eq!(
        syntax_error("x = 1..="),
        at("Unexpected end of input", 1, 9)
    );
    assert_eq!(
        syntax_error("x = 1..<\ny = 2"),
        at("Unexpected end of line", 2, 1)
    );
    assert_eq!(syntax_error("f(0..<)"), at("Unexpected ')'", 1, 8));
    assert_eq!(syntax_error("x = ..<3"), at("Unexpected '.'", 1, 6));
    assert_eq!(
        failure("x = (1..<2)..<3"),
        error(ErrorKind::Type, BOUNDS, 1, 1)
    );
    assert_eq!(
        failure("x = 1..<(2..<3)"),
        error(ErrorKind::Type, BOUNDS, 1, 1)
    );
}

#[test]
fn the_further_error_wins_when_the_range_run_fails_too() {
    assert_eq!(
        failure("class S(ok)\nif every S.ok:\n    y = 0..<3"),
        error(ErrorKind::Syntax, MEMBER, 3, 12)
    );
    assert_eq!(
        failure("class S(ok)\ny = 0..<3\nif every S.ok:\n    z = 1"),
        error(
            ErrorKind::Syntax,
            "Use all instead of every inside if",
            3,
            4
        )
    );
    assert_eq!(
        failure("x = 0..<3\ny = )"),
        error(ErrorKind::Syntax, "Unexpected ')'", 2, 6)
    );
}

#[test]
fn ranges_share_the_nesting_limit_of_every_expression() {
    let shallow = format!("x = {}0..<3{}\nx", "(".repeat(30), ")".repeat(30));
    assert_eq!(result(&shallow), numbers(0..3));
    let deep = format!("x = {}0..<3{}", "(".repeat(70), ")".repeat(70));
    assert_eq!(
        syntax_error(&deep).0,
        "Expressions are nested too deeply".to_string()
    );
    let bound = format!("x = 0..<{}3{}", "(".repeat(70), ")".repeat(70));
    assert_eq!(
        syntax_error(&bound).0,
        "Expressions are nested too deeply".to_string()
    );
}

#[test]
fn near_misses_keep_their_accepted_errors() {
    for (source, message, line, column) in [
        (
            "x = 1..2\nx",
            "Expected a property but found number 0.2",
            1,
            7,
        ),
        ("x = 1..2", "Expected a property but found number 0.2", 1, 9),
        (
            "`${1..2}`",
            "Expected a property but found number 0.2",
            1,
            8,
        ),
        ("a = 1\nb = 3\nx = a..b", MEMBER, 3, 8),
        ("x = 1...3", MEMBER, 1, 8),
        ("b = {q: 1}\nx = {a: 1 ...b}", MEMBER, 2, 13),
        ("a = 1\nb = 3\nx = a. .<b", MEMBER, 3, 9),
        ("a = 1\nb = 3\nx = a.. <b", MEMBER, 3, 9),
        ("a = 1\nx = (a.\n       .<3)", MEMBER, 3, 9),
        ("a = 1\nx = (a..\n        <3)", MEMBER, 3, 9),
        ("x = 0..<=3", MEMBER, 1, 8),
        ("x = 0..==3", MEMBER, 1, 8),
        ("x = 0..=>3", MEMBER, 1, 8),
        ("x = 0..\nx", MEMBER, 1, 7),
        ("x = 0.<3", "Expected a property but found '<'", 1, 8),
        ("def f(xs) => xs\nf(..<3)", "Unexpected '.'", 2, 4),
        (
            "class P(this..<x)",
            "Expected a parameter but found keyword 'this'",
            1,
            13,
        ),
    ] {
        assert_eq!(
            failure(source),
            error(ErrorKind::Syntax, message, line, column),
            "{source}"
        );
    }
}

#[test]
fn bounds_list_consecutive_integers_in_ascending_order() {
    for (source, expected) in [
        ("0..<5", numbers(0..5)),
        ("1..=5", numbers(1..6)),
        ("-2..<1", numbers(-2..1)),
        ("-3..=-1", numbers(-3..0)),
        ("5..<5", json!([])),
        ("5..<2", json!([])),
        ("5..=4", json!([])),
        ("5..=5", numbers(5..6)),
        ("-0..<2", numbers(0..2)),
        ("2.0..<4", numbers(2..4)),
        ("1e2..<102", numbers(100..102)),
        ("0..<0", json!([])),
        ("0..=0", numbers(0..1)),
    ] {
        assert_eq!(result(&format!("x = {source}\nx")), expected, "{source}");
    }
    assert_eq!(result("x = (1 / (-0..<2)[0]) > 0\nx"), json!(true));
    assert_eq!(
        result(
            "x = 9007199254740990..=9007199254740991\n[x.length, x[0] == 9007199254740990, x[1] == 9007199254740991, x[1] - x[0]]"
        ),
        json!([2.0, true, true, 1.0])
    );
    assert_eq!(
        result(
            "x = -9007199254740991..<-9007199254740989\n[x.length, x[0] == -9007199254740991, x[1] == -9007199254740990]"
        ),
        json!([2.0, true, true])
    );
}

#[test]
fn ranges_hold_at_most_one_hundred_thousand_items() {
    assert_eq!(result("x = 1..=100000\nx.length"), json!(100_000.0));
    assert_eq!(result("x = 0..<100000\nx.length"), json!(100_000.0));
    assert_eq!(
        result("x = -50000..<50000\n[x.length, x[0], x.at(-1)]"),
        json!([100_000.0, -50_000.0, 49_999.0])
    );
    for source in [
        "x = 0..=100000",
        "x = 0..<100001",
        "x = 1..<100002",
        "x = -9007199254740991..<9007199254740991",
        "x = 0..<1e15",
    ] {
        assert_eq!(
            failure(source),
            error(ErrorKind::Type, TOO_LARGE, 1, 1),
            "{source}"
        );
    }
}

#[test]
fn every_bound_that_is_not_a_safe_integer_raises() {
    for source in [
        "x = 0..<2.5",
        "x = 0.5..<2",
        "x = null..<3",
        "x = 0..<null",
        "x = \"0\"..<3",
        "x = 0..<\"3\"",
        "x = true..<3",
        "x = 0..<false",
        "x = [0]..<3",
        "x = 0..<[3]",
        "o = {}\nx = o..<3",
        "def f() => 1\nx = f..<3",
        "x = /a/..<3",
        "x = Date(0)..<3",
        "class C\nx = C..<3",
        "class C(v)\nc = C(1)\nx = c..<3",
        "x = 0..<0 / 0",
        "x = 0..<1 / 0",
        "x = -1 / 0..<0",
        "x = 0..<9007199254740992",
        "x = -9007199254740992..<0",
        "x = 9007199254740991..=9007199254740992",
        "x = (0..<2)..<3",
    ] {
        assert_eq!(
            failure(source),
            error(ErrorKind::Type, BOUNDS, source.lines().count(), 1),
            "{source}"
        );
    }
}

#[test]
fn an_undefined_bound_makes_the_range_undefined_after_both_bounds_run() {
    assert_eq!(result("o = {}\nx = 0..<o.n\nx"), Json::Null);
    assert_eq!(result("o = {}\nx = o.n..=3\nx"), Json::Null);
    assert_eq!(result("o = {}\nx = o.n..<2.5\nx"), Json::Null);
    assert_eq!(result("o = {}\nx = \"a\"..<o.n\nx"), Json::Null);
    assert_eq!(result("o = {}\nx = 1..=o.n\no.n = 3\nx"), numbers(1..4));
    assert_eq!(
        failure("o = {}\nx = o.n..<missing"),
        error(ErrorKind::Reference, "missing is not defined", 2, 1)
    );
    assert_eq!(
        failure("x = first..<second"),
        error(ErrorKind::Reference, "first is not defined", 1, 1)
    );
    assert_eq!(
        failure("first = 1\nx = first..<second"),
        error(ErrorKind::Reference, "second is not defined", 2, 1)
    );
}

#[test]
fn the_binary_operator_function_builds_ranges() {
    let three = |end| binary(BinaryOp::Range, &Value::Number(0.0), &Value::Number(end));
    assert_eq!(
        three(3.0).unwrap(),
        Value::List(vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0)
        ])
    );
    assert_eq!(
        binary(
            BinaryOp::RangeInclusive,
            &Value::Number(2.0),
            &Value::Number(3.0)
        )
        .unwrap(),
        Value::List(vec![Value::Number(2.0), Value::Number(3.0)])
    );
    assert_eq!(three(-1.0).unwrap(), Value::List(Vec::new()));
    assert_eq!(
        binary(BinaryOp::Range, &Value::Undefined, &Value::Number(1.0)).unwrap(),
        Value::Undefined
    );
    assert_eq!(
        binary(
            BinaryOp::RangeInclusive,
            &Value::Number(1.0),
            &Value::Undefined
        )
        .unwrap(),
        Value::Undefined
    );
    for bad in [
        three(0.5),
        three(f64::NAN),
        three(f64::INFINITY),
        binary(BinaryOp::Range, &Value::Null, &Value::Number(1.0)),
        binary(
            BinaryOp::Range,
            &Value::String("0".to_string()),
            &Value::Number(1.0),
        ),
    ] {
        let error = bad.unwrap_err();
        assert_eq!(
            (error.kind(), error.message()),
            (ErrorKind::Type, BOUNDS.to_string())
        );
    }
    let error = three(100_001.0).unwrap_err();
    assert_eq!(
        (error.kind(), error.message()),
        (ErrorKind::Type, TOO_LARGE.to_string())
    );
    match binary(
        BinaryOp::RangeInclusive,
        &Value::Number(1.0),
        &Value::Number(100_000.0),
    )
    .unwrap()
    {
        Value::List(items) => assert_eq!(items.len(), 100_000),
        other => panic!("{other:?}"),
    }
}

#[test]
fn standing_ranges_follow_their_bounds_and_roll_back() {
    let mut runtime = Runtime::new();
    assert_eq!(
        evaluate(&mut runtime, "low = 1\nhigh = 3\nspan = low..=high\nspan"),
        numbers(1..4)
    );
    assert_eq!(evaluate(&mut runtime, "high = 5\nspan"), numbers(1..6));
    assert_eq!(evaluate(&mut runtime, "low = 6\nspan"), json!([]));
    assert_eq!(evaluate(&mut runtime, "low = -1\nspan.length"), json!(7.0));
    assert_eq!(
        evaluate(
            &mut runtime,
            "frozen = 0..<high.value\nhalf = 0..<high\nhigh = 2\n[frozen.length, half.length, span.length]"
        ),
        json!([5.0, 2.0, 4.0])
    );
    assert_eq!(
        failure_in(&mut runtime, "high = 2.5"),
        error(ErrorKind::Type, BOUNDS, 1, 1)
    );
    assert_eq!(
        failure_in(&mut runtime, "low = -200000"),
        error(ErrorKind::Type, TOO_LARGE, 1, 1)
    );
    assert_eq!(
        evaluate(&mut runtime, "[low, high, span, half]"),
        json!([-1.0, 2.0, [-1.0, 0.0, 1.0, 2.0], [0.0, 1.0]])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "n = 3\nx = 0..<n\ny = suppose n = 5: x.length\n[y, x.length]"
        ),
        json!([5.0, 3.0])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "m = 2\nz = 0..<m\ntry:\n    m = 4\n    assert(z, [0, 1, 2, 3])\n    throw \"UNDO\"\ncatch error:\n    reason = error\n[reason, z]"
        ),
        json!(["UNDO", [0.0, 1.0]])
    );
}

#[test]
fn class_level_ranges_hold_for_instances_created_before_and_after() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Row(first, last):\n    this.first = first\n    this.last = last\na = Row(1, 3)\n$Row.seats = $Row.first..=$Row.last\n$Row.middle = 2 in $Row.seats\nb = Row(5, 6)\n",
    );
    assert_eq!(
        evaluate(&mut runtime, "[a.seats, b.seats, a.middle, b.middle]"),
        json!([[1.0, 2.0, 3.0], [5.0, 6.0], true, false])
    );
    checked(&mut runtime, "a.first = 3\nb.first = 2\nb.last = 2\n");
    assert_eq!(
        evaluate(&mut runtime, "[a.seats, b.seats, a.middle, b.middle]"),
        json!([[3.0], [2.0], false, true])
    );
    assert_eq!(
        evaluate(&mut runtime, "Row.map(row -> row.seats.length).sum()"),
        json!(2.0)
    );
    assert_eq!(
        failure_in(&mut runtime, "$Row.bad = $Row.first..<limit"),
        error(ErrorKind::Reference, "limit is not defined", 1, 1)
    );
    assert_eq!(
        failure_in(&mut runtime, "a.last = 1.5"),
        error(ErrorKind::Type, BOUNDS, 1, 1)
    );
    assert_eq!(evaluate(&mut runtime, "a.seats"), json!([3.0]));
    assert_eq!(
        evaluate(
            &mut runtime,
            "class Box(size)\n$Box.cells = 0..<$Box.size\nbox = Box(2)\nbox.size = 4\nbox.cells"
        ),
        numbers(0..4)
    );
}

#[test]
fn ranges_are_lists_in_every_expression_position() {
    for (source, expected) in [
        ("2 in 0..<3", json!(true)),
        ("3 in 0..<3", json!(false)),
        ("3 in 0..=3", json!(true)),
        ("2.5 in 0..=3", json!(false)),
        ("x = 4\nx !in 0..<3", json!(true)),
        ("(0..<3) == [0, 1, 2]", json!(true)),
        ("typeof (0..<3) == typeof [0]", json!(true)),
        ("(1..=4).sum()", json!(10.0)),
        ("(0..<5)[1:3]", numbers(1..3)),
        ("(0..<10)[::3]", json!([0.0, 3.0, 6.0, 9.0])),
        ("(0..<3)[1]", json!(1.0)),
        ("[i * i for i in 0..<4]", json!([0.0, 1.0, 4.0, 9.0])),
        ("[i for i in 0..<6 if i % 2]", json!([1.0, 3.0, 5.0])),
        ("all i in 0..<3: i < 3", json!(true)),
        ("any i in 1..=3: i == 2", json!(true)),
        ("def f(n) => 0..<n\nf(3)", numbers(0..3)),
        ("[1, 2].map(n -> 0..<n)", json!([[0.0], [0.0, 1.0]])),
        ("[1, 2].map(lambda n: 1..=n)", json!([[1.0], [1.0, 2.0]])),
        ("x = 1 if 0..<0 else 2\nx", json!(1.0)),
        ("o = {k: 0..<2}\no.k", numbers(0..2)),
        ("[0..<2, 1..=1]", json!([[0.0, 1.0], [1.0]])),
        ("`${0..<3}`", json!("0,1,2")),
        ("x = 2\ny = x ?? 0..<3\ny", json!(2.0)),
        ("x := 0..<2\nx", numbers(0..2)),
        ("a, b = 0..<2\n[b, a]", json!([1.0, 0.0])),
        ("def f():\n    return 1..=3\nf()", numbers(1..4)),
        (
            "class Box:\n    def __init__(self, size):\n        self.size = size\n    def cells(self):\n        return 0..<self.size\nBox(3).cells()",
            numbers(0..3),
        ),
        (
            "xs = (0..<3).concat(5..=6)\nxs",
            json!([0.0, 1.0, 2.0, 5.0, 6.0]),
        ),
        ("x = 1..=3\nx.indexOf(2)", json!(1.0)),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn a_range_is_a_list_and_every_list_is_truthy() {
    let limit = failure("while [0]:\n    y = 1");
    assert_eq!(
        limit,
        error(ErrorKind::Type, "Loop did not terminate", 1, 1)
    );
    for source in [
        "while 0..<3:\n    y = 1",
        "while 0..<0:\n    y = 1",
        "x = 5\nwhile x..<3:\n    y = 1",
        "while []:\n    y = 1",
    ] {
        let (kind, message, _) = failure(source);
        assert_eq!((kind, message), (limit.0, limit.1.clone()), "{source}");
    }
    assert_eq!(result("y = 0\nif 0..<0:\n    y = 1\ny"), json!(1.0));
    assert_eq!(result("x = 3\ny = x in 0..<3 ? 1 : 2\ny"), json!(2.0));
    assert_eq!(
        result("x = 2\ny = 0\nif x in 0..<3:\n    y = 1\ny"),
        json!(1.0)
    );
}

#[test]
fn reserved_positions_keep_their_accepted_errors() {
    for (source, line, column) in [
        ("xs = [5, 6]\nxs[0..<1]", 2, 7),
        ("xs = [5, 6]\nxs[0..<1:]", 2, 7),
        ("xs = [5, 6]\nxs[:0..<1]", 2, 8),
        ("xs = [5, 6]\nxs[::0..<1]", 2, 9),
        ("xs = [5, 6]\nxs[0:0..=1]", 2, 9),
        ("xs = [5, 6]\nxs[1:2:0..<1]", 2, 11),
        ("xs = [5, 6]\nxs[x in 0..<3]", 2, 12),
        ("xs = [5, 6]\nys = [xs[0..<1]]", 2, 13),
        ("xs = [5, 6]\nwhile xs[0..<1]:\n    y = 1", 2, 13),
        ("class Seat(n):\n    this.n = n\nSeat(1)\nSeat[1..=3]", 4, 9),
        ("for x in 0..<3:\n    y = x", 1, 13),
        ("for x of 0..<3:\n    y = x", 1, 13),
        ("for k, v in 0..<3:\n    y = v", 1, 16),
        ("y = 0\nfor x in 0..<3: y = x", 2, 13),
        ("repeat 0..<3:\n    y = 1", 1, 11),
        ("repeat 0..<3: y = 1", 1, 11),
        ("repeat n + 0..<3:\n    y = 1", 1, 15),
        ("repeat x in 0..<3:\n    y = 1", 1, 16),
        ("n = 1\nrepeat n..=3:\n    y = 1", 2, 11),
    ] {
        assert_eq!(
            failure(source),
            error(ErrorKind::Syntax, MEMBER, line, column),
            "{source}"
        );
    }
}

#[test]
fn parenthesized_and_nested_ranges_are_ordinary_lists_in_reserved_positions() {
    for (source, expected) in [
        (
            "xs = [5, 6, 7]\n[xs[(0..<2)], xs[[0, 1]]]",
            json!([5.0, 5.0]),
        ),
        ("xs = [5, 6, 7]\n[xs[(2..=2)], xs[[2]]]", json!([7.0, 7.0])),
        ("xs = [5, 6, 7]\nxs[(1..<2):]", json!([6.0, 7.0])),
        ("xs = [5, 6, 7]\nxs[(0..<3).length - 1]", json!(7.0)),
        ("xs = [5, 6, 7]\nxs[[0..<2][0][1]]", json!(6.0)),
        ("xs = [5, 6, 7]\nxs[{k: 1..=1}.k[0]]", json!(6.0)),
        (
            "def h(v) => v.length\nxs = [5, 6, 7]\nxs[h(0..<2)]",
            json!(7.0),
        ),
        (
            "def h(v) => v.length\nxs = [5, 6, 7]\nxs[1:h(0..<3)]",
            json!([6.0, 7.0]),
        ),
        ("xs = [5, 6, 7]\nxs[`${0..<1}`]", json!(5.0)),
        ("y = 0\nfor x in (0..<3):\n    y = x\ny", json!(0.0)),
        (
            "ys = []\nrepeat (3..<4):\n    ys.push(1)\nys.length",
            json!(3.0),
        ),
        (
            "ys = []\nrepeat (0..<3):\n    ys.push(1)\nys.length",
            json!(0.0),
        ),
        (
            "ys = []\nrepeat (0..<3).length:\n    ys.push(1)\nys.length",
            json!(3.0),
        ),
        (
            "def f(xs) => xs.length\nys = []\nrepeat f(0..<3):\n    ys.push(1)\nys.length",
            json!(3.0),
        ),
        (
            "ys = []\nrepeat 2:\n    ys.push(0..<2)\nys",
            json!([[0.0, 1.0], [0.0, 1.0]]),
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for source in [
        "y = xs[x |> why |> depth(0..<1)]",
        "y = xs[function() { return 0..<2 }]",
        "y = xs[v => { return 0..<2 }]",
        "for v in f(0..<2):\n    y = v",
        "for v in [0..<2]:\n    y = v",
        "for v in {k: 0..<2}.k:\n    y = v",
        "repeat [0..<2].length:\n    y = 1",
    ] {
        assert!(parse(source).is_ok(), "{source}");
    }
}

#[test]
fn interpolations_keep_the_tails_they_ignored() {
    for (source, plain) in [
        ("`${x |> why ..<3}`", "`${x |> why}`"),
        ("`${x |> why ..=3}`", "`${x |> why}`"),
        ("`${x |> why ..< 3}`", "`${x |> why}`"),
        (
            "`${x |> why |> depth(1) ..<3}`",
            "`${x |> why |> depth(1)}`",
        ),
        (
            "`${x |> why |> union(why x) ..<3}`",
            "`${x |> why |> union(why x)}`",
        ),
        ("`${x |> why |> stated ..<3}`", "`${x |> why |> stated}`"),
        ("`${model |> why ..<3}`", "`${model |> why}`"),
        ("`${x |> why |> nodes ..<3}`", "`${x |> why |> nodes}`"),
        ("`${x |> why |> holds ..<3}`", "`${x |> why |> holds}`"),
        ("`${x if true else 2 ..<3}`", "`${x}`"),
        ("`${x is 1 ..<3}`", "`${x}`"),
        ("`${x in [1] ..<3}`", "`${x}`"),
    ] {
        assert_eq!(parse(source).unwrap(), parse(plain).unwrap(), "{source}");
        assert!(result(&format!("x = 1\n{source}")).is_string(), "{source}");
    }
    for (source, expected) in [
        ("`${x |> why |> nodes ..<3}`", "x"),
        ("`${x |> why |> holds ..<3}`", "1"),
        ("`${x if true else 2 ..<3}`", "1"),
        ("`${x is 1 ..<3}`", "1"),
        ("`${x in [1] ..<3}`", "1"),
        ("`${x}..<3`", "1..<3"),
    ] {
        assert_eq!(
            result(&format!("x = 1\n{source}")),
            json!(expected),
            "{source}"
        );
    }
    for (source, message) in [
        ("`${all v in [1]: v > 0 ..<3}`", "all is not defined"),
        ("`${lambda v: v ..<3}`", "lambda is not defined"),
        ("`${suppose: x ..<3}`", "suppose is not defined"),
    ] {
        assert_eq!(
            failure(&format!("x = 1\n{source}")),
            error(ErrorKind::Reference, message, 2, 1),
            "{source}"
        );
    }
    assert_eq!(
        failure("x = 1\n`${x |> why |> diff ..<3}`"),
        error(ErrorKind::Type, "Cannot use 'diff' outside suppose", 2, 1)
    );
}

#[test]
fn interpolations_that_failed_at_the_operator_read_the_range() {
    for (source, expected) in [
        ("`${x ..<3}`", json!("1,2")),
        ("`${(x) ..<3}`", json!("1,2")),
        ("`${x ?? y ..<3}`", json!("1")),
        ("`${true ? x : 2 ..<3}`", json!("1")),
        ("`${v -> v ..<3}`", json!("[function]")),
        ("`${[1, 5].filter(> 4).length ..<3}`", json!("1,2")),
        ("`${[{a: 1}].map(.a)[0] ..<3}`", json!("1,2")),
        ("`${0..<x + 2}`", json!("0,1,2")),
        ("`${0..=x}`", json!("0,1")),
    ] {
        assert_eq!(
            result(&format!("x = 1\ny = 2\n{source}")),
            expected,
            "{source}"
        );
    }
    for source in ["`${why x ..<3}`", "`${[x] ..<3}`", "`${(x is 1) ..<3}`"] {
        assert_eq!(
            failure(&format!("x = 1\n{source}")),
            error(ErrorKind::Type, BOUNDS, 2, 1),
            "{source}"
        );
    }
}

#[test]
fn parse_expression_keeps_every_accepted_reading() {
    for (source, accepted) in [
        ("x |> why ..<3", "x |> why"),
        ("x |> why ..= 3", "x |> why"),
        ("x |> why |> nodes ..<3", "x |> why |> nodes"),
        ("x |> why |> depth(1) ..<3", "x |> why |> depth(1)"),
        ("model |> why ..<3", "model |> why"),
        ("x is 1 ..<3", "x"),
        ("x in [1] ..<3", "x"),
    ] {
        assert_eq!(expression(source), expression(accepted), "{source}");
    }
    assert_eq!(
        expression("x ..<3"),
        range(BinaryOp::Range, name("x"), Expr::Number(3.0))
    );
    assert_eq!(
        expression("lambda v: v ..<3"),
        expression("lambda v: (v..<3)")
    );
    assert_eq!(
        expression("x if c else 2 ..<3"),
        expression("x if c else (2..<3)")
    );
    let error = parse_expression("xs[0..<1]").unwrap_err();
    assert_eq!((error.message(), error.position()), at(MEMBER, 1, 7));
}

#[test]
fn ranges_render_as_written_and_parse_back() {
    for source in [
        "0..<n",
        "a..=b",
        "(0..<n).length",
        "(1..=k).map(f)",
        "0..<n + 1",
        "-1..<2",
        "c ? 0..<a : 1..=b",
        "f(0..<n)",
        "[0..<n, a..=b]",
        "(0..<n)[1]",
        "(0..<n)[1:]",
        "2 ** (0..<3)",
        "(0..<3) ** 2",
        "`${0..<3}`",
        "{k: 0..<3}",
        "x ?? 0..<3",
        "0..<3 == xs",
        "a < 0..<3",
        "not a..<b",
        "sum of xs..<3",
        "(0..<n)(1)",
    ] {
        let parsed = expression(source);
        let rendered = parsed.to_string();
        assert_eq!(expression(&rendered), parsed, "{source} => {rendered}");
    }
    for (source, rendered) in [
        ("0..<n + 1", "0..<n+1"),
        ("(0..<n).length", "(0..<n).length"),
        ("(1..=k).map(f)", "(1..=k).map(f)"),
        ("(0..<n)[1]", "(0..<n)[1]"),
        ("`${0..<3}`", "`${0..<3}`"),
        ("(0..<n) + 1", "0..<n+1"),
        ("2 ** (0..<3)", "2**(0..<3)"),
    ] {
        assert_eq!(expression(source).to_string(), rendered, "{source}");
    }
    assert_ne!(expression("(0..<n) + 1"), expression("0..<n+1"));
    assert_eq!(expression("(a + b).c").to_string(), "a+b.c");
    assert_eq!(expression("(a < b)[0]").to_string(), "a<b[0]");
    assert_eq!(
        generate(&parse("hours = first..<last").unwrap()[0]),
        "hours=first..<last"
    );
    assert_eq!(
        generate(&parse("def f(n = 1) => 0..<n").unwrap()[0]),
        "def f(n=1):\n    return 0..<n"
    );
    assert_eq!(
        generate(&parse("y = 2 in 0..=n").unwrap()[0]),
        "y=(2 in 0..=n)"
    );
}

#[test]
fn explanations_and_graph_keys_quote_the_rendered_range() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, "first = 1\nlast = 4\nhours = first..<last\n");
    assert_eq!(
        evaluate(&mut runtime, "(why hours).map(step -> step.rule)"),
        json!(["hours = first..<last", "first = 1", "last = 4"])
    );
    assert_eq!(
        evaluate(&mut runtime, "(affects last).map(step -> step.node)"),
        json!(["hours"])
    );
    assert_eq!(
        failure_in(&mut runtime, "first..<last |> why"),
        error(ErrorKind::Type, "Cannot explain first..<last", 1, 1)
    );
    let output = cli(
        "graph",
        &["--json", "--graph"],
        "first = 1\nlast = 4\nhours = first..<last\nif (first..<last).length > 2:\n    flag = 1\nclass Shift(start, end):\n    this.start = start\n    this.end = end\n$Shift.hours = $Shift.start..=$Shift.end\ns = Shift(1, 2)\nhours.length\n",
    );
    assert!(output.status.success(), "{output:?}");
    let listing = String::from_utf8(output.stdout).unwrap();
    assert!(listing.starts_with("3.0\n"), "{listing}");
    for line in [
        "  first [variable]\n    updates  hours, if((first..<last).length>2)\n",
        "  hours [variable]\n    reads    first, last\n",
        "  if((first..<last).length>2) [if]\n    reads    first, last\n",
        "  s.hours [property]\n    reads    s.start, s.end\n",
    ] {
        assert!(listing.contains(line), "{listing}");
    }
}

#[test]
fn the_binary_runs_ranges_from_files_and_the_line_repl() {
    let output = cli("file", &["--json"], "n = 3\nx = 0..<n\nn = 5\nx\n");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "[0.0,1.0,2.0,3.0,4.0]\n"
    );
    let failed = cli("error", &[], "xs = [1]\nxs[0..<1]\n");
    assert!(!failed.status.success());
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "SyntaxError: Expected a property but found '.' (line 2, column 7)\n"
    );
    for ending in ["\n", "\r\n"] {
        let lines = [
            "n = 3",
            "x = 0..<n",
            "n = 5",
            "x.length",
            "y = 1..=",
            "xs = [7, 8]",
            "xs[0..<1]",
            "xs[(0..<1)]",
            "(1..=3).sum()",
        ];
        let output = repl(&["--plain", "--json"], &(lines.join(ending) + ending));
        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{BANNER}3.0\n[0.0,1.0,2.0]\n5.0\n5.0\n[7.0,8.0]\n7.0\n6.0\n")
        );
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "SyntaxError: Unexpected end of line (line 2, column 1)\nSyntaxError: Expected a property but found '.' (line 1, column 7)\n"
        );
        let plain = repl(&["--plain"], &(lines.join(ending) + ending));
        assert!(plain.status.success(), "{plain:?}");
        assert_eq!(
            String::from_utf8(plain.stdout).unwrap(),
            format!("{BANNER}3\n0,1,2\n5\n5\n7,8\n7\n6\n")
        );
    }
}

#[test]
fn ranges_stand_in_newline_separated_list_elements() {
    assert_eq!(
        result(
            "a = 2\nxs = [\n    0..<a\n    a..=4\n]\nassert(xs, [[0, 1], [2, 3, 4]])\na = 3\nassert(xs, [[0, 1, 2], [3, 4]])\nxs.length"
        ),
        json!(2.0)
    );
    assert_eq!(
        value_of("xs = [\n    0..<a\n    a..=4\n]"),
        value_of("xs = [0..<a, a..=4]")
    );
    for (source, expected) in [
        (
            "xs = [0..<2, 5\n    1..=2]\nxs",
            json!([[0.0, 1.0], 5.0, [1.0, 2.0]]),
        ),
        (
            "rows = [\n    [0..<2\n    5],\n    [1..=2]\n]\nrows",
            json!([[[0.0, 1.0], 5.0], [[1.0, 2.0]]]),
        ),
        (
            "ys = [\n    [i * 2 for i in 0..<3]\n    7\n]\nys",
            json!([[0.0, 2.0, 4.0], 7.0]),
        ),
        ("n = 3\ns = `${[0..<n\n9]}`\nn = 1\ns", json!("0,9")),
        ("n = 3\ns = `${[0..<n\n1..=1].length}`\ns", json!("2")),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    assert_eq!(
        syntax_error("ys = [\n    [i * 2 for i in 0..<3]\n    [i for i in 1..=2]\n]"),
        at("Expected ']' but found keyword 'for'", 3, 8)
    );
}

#[test]
fn the_line_break_run_keeps_its_reading_before_the_range_run() {
    assert_eq!(
        result("x = 1\nxs = [1\n2]\ns = `${x if false else 2 ..<3}`\n[xs, s]"),
        json!([[1.0, 2.0], "1"])
    );
    assert_eq!(
        parse("x = 1\nxs = [1\n2]\ns = `${x |> why ..<3}`").unwrap(),
        parse("x = 1\nxs = [1, 2]\ns = `${x |> why}`").unwrap()
    );
    assert_eq!(result("x = 1\ns = `${[x\n2].length ..<3}`\ns"), json!("2"));
}

#[test]
fn the_probes_of_the_range_run_read_ranges() {
    for body in ["0..<3", "[0, 1, 2]"] {
        let words = "xs = [1]\nf = xs.filter(x => x is 1)\n";
        assert_eq!(
            parse(&format!("{words}while v is suppose:\n    a = {body}")).unwrap(),
            parse(&format!("{words}while\nv = suppose:\n    a = {body}")).unwrap(),
            "{body}"
        );
        assert_eq!(
            parse(&format!("while v is suppose:\n    a = {body}")).unwrap(),
            parse(&format!("while\nv = suppose:\n    a = {body}")).unwrap(),
            "{body}"
        );
    }
}

#[test]
fn the_range_run_keeps_the_line_breaks_of_the_source() {
    assert_eq!(
        result(
            "xs = [1\n2]\nclass O\no = O()\no.is = 1\nx = 1\nwhile x < 3: x = x + 1\nr = 0..<3\n[xs, r, o.is, x]"
        ),
        json!([[1.0, 2.0], [0.0, 1.0, 2.0], 1.0, 3.0])
    );
    assert_eq!(
        result(
            "xs = [1\n2]\nclass O\no = O()\no.is = 1\nx = 1\nwhile x < 3: x = x + 1\nr = 3\n[xs, r, o.is, x]"
        ),
        json!([[1.0, 2.0], 3.0, 1.0, 3.0])
    );
}

#[test]
fn the_range_run_repeats_the_line_break_run_and_reports_the_further_error() {
    assert_eq!(
        syntax_error("xs = [0..<2\n3 4]"),
        at("Expected ']' but found number 4", 2, 3)
    );
    assert_eq!(
        syntax_error("xs = [\n    0..<2\n    3\nx = )"),
        at("Unexpected ')'", 4, 6)
    );
    assert_eq!(
        syntax_error("class S(ok)\nxs = [1\n2]\nif every S.ok:\n    y = 0..<3"),
        at(MEMBER, 5, 12)
    );
}

#[test]
fn deferred_suites_read_ranges() {
    assert_eq!(
        result(
            "log = []\ndef run(n):\n    defer:\n        log.push(0..<n)\n    return n\nassert(run(2), 2)\nassert(log, [[0, 1]])\nassert(run(0), 0)\nassert(log, [[0, 1], []])\nlog.length"
        ),
        json!(2.0)
    );
    assert!(matches!(
        parse("defer:\n    x = 0..<n").unwrap().as_slice(),
        [Stmt::Defer(body)] if body.len() == 1
    ));
    assert_eq!(
        failure("def run(n):\n    defer:\n        x = 0..<n\n    return n\nrun(0.5)"),
        error(ErrorKind::Type, BOUNDS, 5, 1)
    );
    assert_eq!(
        failure("n = 0.5\ndefer:\n    x = 0..<n\ny = 1"),
        error(ErrorKind::Type, BOUNDS, 2, 1)
    );
}

#[test]
fn truthy_value_assignments_take_and_skip_ranges() {
    assert_eq!(
        result("n = 2\nxs = [9]\nxs &&:= 0..<n\nassert(xs, [0, 1])\nn = 4\nassert(xs, [0, 1])\nxs"),
        numbers(0..2)
    );
    assert_eq!(
        result("zero = 0\nzero &&:= 0..<\"x\"\nassert(zero, 0)\nzero"),
        json!(0.0)
    );
    assert_eq!(
        result("class B\nb = B()\nb.v = [1]\nb.v &&:= 1..=3\nb.v"),
        numbers(1..4)
    );
    assert_eq!(
        failure("xs = [9]\nxs &&:= 0..<1.5"),
        error(ErrorKind::Type, BOUNDS, 2, 1)
    );
    assert!(matches!(
        parse("xs &&:= 0..<n").unwrap().as_slice(),
        [Stmt::TruthyValueAssign {
            value: Expr::Binary {
                operator: BinaryOp::Range,
                ..
            },
            ..
        }]
    ));
}
