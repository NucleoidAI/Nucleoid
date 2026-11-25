mod common;

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Spanned, Token, tokenize};
use nucleoid::lang::ast::{parse, parse_expression, parse_program};
use nucleoid::{ErrorKind, Position, Runtime};

const READINGS: &str = "seed = 2\nreadings = [\n    seed,\n    seed + 1,\n    seed * 2\n]\nassert(readings, [2, 3, 4])\nassert(readings.length, 3)\nassert(readings[0], 2)\nseed = 5\nassert(readings, [5, 6, 10])\nassert(readings.length, 3)\nassert(readings[2], 10)\n1";
const ROWS: &str = "seed = 2\nrows = [\n    [seed, seed + 1],\n    [seed * 2, seed * 3]\n]\nassert(rows[0], [2, 3])\nassert(rows[1], [4, 6])\nassert(rows.length, 2)\nseed = 4\nassert(rows[0], [4, 5])\nassert(rows[1], [8, 12])\nassert(rows[1].length, 2)\n1";

fn equivalent(actual: &str, commas: &str) {
    let actual_tree = parse(actual).unwrap_or_else(|error| panic!("{actual}\n{error}"));
    let comma_tree = parse(commas).unwrap_or_else(|error| panic!("{commas}\n{error}"));
    assert_eq!(actual_tree, comma_tree, "{actual}");
    assert_eq!(generate_all(&actual_tree), generate_all(&comma_tree));
    assert_eq!(
        common::assertions::protected_assertions(&actual_tree),
        common::assertions::protected_assertions(&comma_tree)
    );
}

fn checked(source: &str, count: usize, expected: &str) -> Runtime {
    assert_eq!(
        common::assertions::protected_assertions(&parse(source).unwrap()).len(),
        count,
        "{source}"
    );
    let mut runtime = Runtime::new();
    common::check_with_count(
        &mut runtime,
        &common::Case {
            title: "newline list regression".to_string(),
            source: source.to_string(),
            expected: Some(expected.to_string()),
        },
        count,
    )
    .unwrap_or_else(|error| panic!("{source}\n{error}"));
    assert_eq!(runtime.assertions_run(), count, "{source}");
    runtime
}

fn pair(actual: &str, commas: &str, count: usize, expected: &str) {
    equivalent(actual, commas);
    checked(actual, count, expected);
    checked(commas, count, expected);
}

#[test]
fn supplied_counterparts_keep_identities_six_assertions_and_expected_returns() {
    for (document, title, commas) in [
        (
            include_str!("../nucleoid.spec.md"),
            "Nucleoid separates list elements on new lines",
            READINGS,
        ),
        (
            include_str!("reference.md"),
            "Nucleoid nests newline separated lists",
            ROWS,
        ),
    ] {
        let cases = common::cases(document);
        let matches: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matches.len(), 1);
        let case = matches[0];
        assert_eq!(case.expected.as_deref(), Some("1"));
        pair(case.source.trim(), commas, 6, "1");
    }
}

#[test]
fn empty_mixed_trailing_and_commented_boundaries_reuse_the_list_ast() {
    for (actual, commas, expected) in [
        ("[]", "[]", "[]"),
        ("[\n]", "[]", "[]"),
        ("[1\n2\n3]", "[1,2,3]", "[1,2,3]"),
        ("[1,\n2\n3,]", "[1,2,3,]", "[1,2,3]"),
        ("[1\n\n# gap\n2]", "[1,2]", "[1,2]"),
        ("[1// gap\n2]", "[1,2]", "[1,2]"),
        ("[1/*\n*/2]", "[1,2]", "[1,2]"),
        ("[\r\n\t1\r\n\t2\r\n]", "[1,2]", "[1,2]"),
        (
            "[{a: 1}\n{b: 2}]",
            "[{a: 1},{b: 2}]",
            r#"[{"a":1},{"b":2}]"#,
        ),
    ] {
        pair(actual, commas, 0, expected);
        assert_eq!(
            parse_expression(actual).unwrap(),
            parse_expression(commas).unwrap()
        );
    }
}

#[test]
fn complete_accepted_expressions_win_across_physical_lines() {
    for (actual, ordinary, expected) in [
        ("f = x => x + 1\n[f\n(2)]", "f = x => x + 1\n[f(2)]", "[3]"),
        ("[[4,5]\n[1]]", "[[4,5][1]]", "[5]"),
        ("[1\n[0]]", "[1[0]]", "[null]"),
        ("[1\n+2\n*3]", "[1+2*3]", "[7]"),
        ("[1\n-2]", "[1-2]", "[-1]"),
        ("[true\n? 1\n: 2]", "[true?1:2]", "[1]"),
        ("[1\nif true\nelse 2]", "[1 if true else 2]", "[1]"),
        ("[1\nis\n2]", "[1 is 2]", "[false]"),
        ("[1\nis\nnot\n2]", "[1 is not 2]", "[true]"),
        ("[x * 2\nfor x in [1,2]]", "[x * 2 for x in [1,2]]", "[2,4]"),
        ("[{a: 3}\n.a]", "[{a: 3}.a]", "[3]"),
    ] {
        pair(actual, ordinary, 0, expected);
        pair(
            &format!("prefix = [9\n10]\n{actual}"),
            &format!("prefix = [9,10]\n{ordinary}"),
            0,
            expected,
        );
    }
}

#[test]
fn accepted_runtime_failure_is_not_reinterpreted_as_two_items() {
    let source = "result = [1\n(2)]\nresult";
    equivalent(source, "result = [1(2)]\nresult");
    let error = Runtime::new().run(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "1 is not a function");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn same_line_omissions_and_failed_whole_items_keep_pinned_diagnostics() {
    for (source, message, line, column) in [
        ("items = [1 2]", "Expected ']' but found number 2", 1, 12),
        ("items = [1\r2]", "Expected ']' but found number 2", 1, 12),
        (
            "items = [1/*gap*/2]",
            "Expected ']' but found number 2",
            1,
            18,
        ),
        (
            "items = [\"é\n猫\" 2]",
            "Expected ']' but found number 2",
            2,
            4,
        ),
        (
            "items = ['x\ny' 2]",
            "Expected ']' but found number 2",
            2,
            4,
        ),
        (
            "items = [`x\ny` 2]",
            "Expected ']' but found number 2",
            2,
            4,
        ),
        (
            "`values:${[\"é\n猫\" 2]}`",
            "Expected ']' but found number 2",
            2,
            4,
        ),
        (
            "items = [/a\\\nb/ 2]",
            "Expected ']' but found number 2",
            2,
            4,
        ),
        ("items = [1 +\n]", "Unexpected ']'", 2, 2),
        (
            "items = [f\n(1 2)]",
            "Expected ')' but found number 2",
            2,
            4,
        ),
        (
            "`values:${[f\n(1 2)]}`",
            "Expected ')' but found number 2",
            2,
            4,
        ),
        ("items = [[1]\n[2, 3]]", "Expected ']' but found ','", 2, 3),
        (
            "items = [Math.abs(\n1) 2]",
            "Expected ']' but found number 2",
            2,
            4,
        ),
        (
            "items = [true ? 1\n2]",
            "Expected ':' but found number 2",
            2,
            1,
        ),
        ("take(1\n2)", "Expected ')' but found number 2", 2, 1),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(
            error.position(),
            Some(Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn public_token_stream_and_unicode_start_positions_are_unchanged() {
    let tokens = tokenize("[\"é\n猫\" 2]").unwrap();
    let expected = [
        (Token::LeftBracket, 1, 1),
        (Token::Str("é\n猫".to_string()), 1, 2),
        (Token::Number(2.0), 2, 4),
        (Token::RightBracket, 2, 5),
        (Token::Eof, 2, 6),
    ]
    .into_iter()
    .map(|(token, line, column)| Spanned {
        token,
        position: Position::new(line, column),
    })
    .collect::<Vec<_>>();
    assert_eq!(tokens, expected);
    assert_eq!(
        tokenize("[1\n2]").unwrap(),
        vec![
            Spanned {
                token: Token::LeftBracket,
                position: Position::new(1, 1)
            },
            Spanned {
                token: Token::Number(1.0),
                position: Position::new(1, 2)
            },
            Spanned {
                token: Token::Number(2.0),
                position: Position::new(2, 1)
            },
            Spanned {
                token: Token::RightBracket,
                position: Position::new(2, 2)
            },
            Spanned {
                token: Token::Eof,
                position: Position::new(2, 3)
            },
        ]
    );
}

#[test]
fn multiline_literals_need_a_physical_break_after_their_ending_token() {
    pair("[\"é\n猫\"\n2]", "[\"é\n猫\",2]", 0, "[\"é\\n猫\",2]");
    pair("['x\ny'\n2]", "['x\ny',2]", 0, "[\"x\\ny\",2]");
    pair("[`x\ny`\n2]", "[`x\ny`,2]", 0, "[\"x\\ny\",2]");
    pair("[/a\\\nb/\n2].length", "[/a\\\nb/,2].length", 0, "2");
}

#[test]
fn item_word_marking_and_explicit_variable_readings_are_unchanged() {
    pair(
        "colors is [red\nblue]\ncolors",
        "colors is [red,blue]\ncolors",
        0,
        "[\"red\",\"blue\"]",
    );
    pair(
        "red = 2\nblue = 3\ncolors = [red\nblue]\ncolors",
        "red = 2\nblue = 3\ncolors = [red,blue]\ncolors",
        0,
        "[2,3]",
    );
    let error = Runtime::new().run("colors = [red\nblue]").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "red is not defined");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn interpolation_and_operator_section_retries_keep_source_boundaries() {
    pair(
        "text = `values:${[1\n2]}`\ntext",
        "text = `values:${[1,2]}`\ntext",
        0,
        "\"values:1,2\"",
    );
    for (actual, commas, expected) in [
        ("`values:${[1\n2]}`", "`values:${[1,2]}`", "\"values:1,2\""),
        (
            "(`values:${[1/*\n*/2]}`)",
            "(`values:${[1,2]}`)",
            "\"values:1,2\"",
        ),
        (
            "`values:${[1\n2].map(*2)}`",
            "`values:${[1,2].map(*2)}`",
            "\"values:2,4\"",
        ),
        (
            "`values:${[1\n2]} same:${[1\nis\n2]}`",
            "`values:${[1,2]} same:${[1 is 2]}`",
            "\"values:1,2 same:false\"",
        ),
    ] {
        pair(actual, commas, 0, expected);
        assert_eq!(
            parse_expression(actual).unwrap(),
            parse_expression(commas).unwrap()
        );
    }
    pair("[2\n3].map(*2)", "[2,3].map(*2)", 0, "[4,6]");
}

#[test]
fn receiver_parameter_retries_keep_newline_lists_inside_methods() {
    let source = "class Sample:\n    def init(self, input):\n        self.input = input\n    def readings(self) => [self.input\nself.input + 1]\nitem = Sample(2)\nitem.readings()";
    let commas = source.replace(
        "[self.input\nself.input + 1]",
        "[self.input,self.input + 1]",
    );
    pair(source, &commas, 0, "[2,3]");
}

#[test]
fn class_rules_and_transaction_rollback_keep_standing_list_reads() {
    let source = "class Sensor\nfirst = Sensor()\nsecond = Sensor()\nfirst.reading = 2\nsecond.reading = 5\n$Sensor.samples = [$Sensor.reading\n$Sensor.reading + 1]\nassert(first.samples, [2,3])\nassert(second.samples, [5,6])\nfirst.reading = 7\nassert(first.samples, [7,8])\nassert(second.samples, [5,6])\ntry:\n    first.reading = 9\n    throw \"rollback\"\ncatch error:\n    assert(error, \"rollback\")\nassert(first.samples, [7,8])\nfirst.reading = 4\nassert(first.samples, [4,5])\nassert(second.samples, [5,6])\n1";
    let commas = source.replace(
        "[$Sensor.reading\n$Sensor.reading + 1]",
        "[$Sensor.reading,$Sensor.reading + 1]",
    );
    pair(source, &commas, 8, "1");
}

#[test]
fn ordinary_control_flow_uses_the_same_lists_and_assertion_coverage() {
    let source = "class Entry(this.number)\na = Entry(1)\nb = Entry(2)\nc = Entry(3)\nseen = []\nfor n in [a\nb\nc]:\n    seen.push(n.number)\nassert(seen, [1,2,3])\ndef choose(values):\n    if values.length == 3:\n        return [values[0]\nvalues[2]]\n    return []\nresult = choose(seen)\nassert(result, [1,3])\nresult.length";
    let commas = source
        .replace("[a\nb\nc]", "[a,b,c]")
        .replace("[values[0]\nvalues[2]]", "[values[0],values[2]]");
    pair(source, &commas, 2, "2");
}

#[test]
fn newlines_do_not_become_a_list_size_limit_or_bypass_expression_guards() {
    let source = format!(
        "items = [{}]\nassert(items.length, 512)\nitems[511]",
        vec!["1"; 512].join("\n")
    );
    checked(&source, 1, "1");
    let nested = format!("items = {}1{}", "[".repeat(65), "]".repeat(65));
    let error = Runtime::check(&nested).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(error.message(), "Expressions are nested too deeply");
    let chain = format!("items = [{}]", vec!["1"; 130].join("\n+"));
    let error = Runtime::check(&chain).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(error.message(), "Expression has too many operands");
}

#[test]
fn legacy_top_level_statement_positions_are_unchanged() {
    let source = "items = [1,\n2]\nassert(items.length, 2)\nitems[0]";
    assert_eq!(
        parse_program(source).unwrap().positions,
        vec![
            Position::new(1, 1),
            Position::new(3, 1),
            Position::new(4, 1)
        ]
    );
    checked(source, 1, "1");
}
