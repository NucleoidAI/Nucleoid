//! `//` and `/* ... */` comments take priority over regular expressions.

mod common;

use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, parse, parse_expression, parse_program};
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn checked(source: &str, expected: &str, assertions: usize) {
    let statements = parse(source).unwrap();
    assert_eq!(
        common::assertions::protected_assertions(&statements).len(),
        assertions,
        "{source}"
    );
    let mut runtime = Runtime::new();
    let value = runtime.run(source).unwrap();
    assert_eq!(
        runtime.serialize_json(&value).unwrap(),
        expected,
        "{source}"
    );
    assert!(runtime.take_assertions().is_empty(), "{source}");
    assert_eq!(runtime.assertions_run(), assertions, "{source}");
}

#[test]
fn slash_pairs_start_comments_before_regexes_and_division() {
    for mut runtime in [Runtime::new(), Runtime::default()] {
        for source in ["//", "// comment", "/* comment */"] {
            assert_eq!(parse(source).unwrap(), Vec::new(), "{source}");
            assert!(Runtime::check(source).is_ok(), "{source}");
            assert_eq!(runtime.run(source).unwrap(), Value::Null, "{source}");
        }
        assert_eq!(parse_expression("1 // 2 /").unwrap(), Expr::Number(1.0));
        assert_eq!(runtime.run("1 // 2 /").unwrap(), Value::Number(1.0));
        assert_eq!(runtime.assertions_run(), 0);
    }
}

#[test]
fn all_comment_forms_preserve_reactivity_and_non_nesting_blocks() {
    checked(
        "# hash comments stay available\nbase = 8 // first value\n\
         total = base /* units */ / 2\nassert(total, 4)\n\
         base = 12\nassert(total, 6)\ntotal // result",
        "6.0",
        2,
    );
    checked(
        "/* outer /* is just text */ value = 7\nassert(value, 7)\nvalue",
        "7.0",
        1,
    );
    for source in [
        "// ignored",
        "/* ignored */",
        "# ignored",
        "/*\n\tignored\n*/",
    ] {
        checked(source, "null", 0);
    }
}

#[test]
fn layout_newlines_tabs_crlf_and_original_character_positions_are_retained() {
    let source = "if true:\r\n\t/* 文\r\n\t many : */\r\n\tvalue = (6 /*\r\n*/ / 2) // end\r\n# blank\r\nvalue";
    let tokens = tokenize(source).unwrap();
    let control = tokenize("if true:\n\tvalue = (6 / 2)\nvalue").unwrap();
    assert_eq!(
        tokens
            .iter()
            .map(|spanned| &spanned.token)
            .collect::<Vec<_>>(),
        control
            .iter()
            .map(|spanned| &spanned.token)
            .collect::<Vec<_>>()
    );
    let positions: Vec<_> = tokens
        .iter()
        .filter(|spanned| spanned.token == Token::Identifier("value".into()))
        .map(|spanned| spanned.position)
        .collect();
    assert_eq!(positions, [Position::new(4, 2), Position::new(7, 1)]);
    checked(source, "3.0", 0);

    let tokens = tokenize("/* bulky */value = 1").unwrap();
    assert_eq!(tokens[0].token, Token::Identifier("value".into()));
    assert_eq!(tokens[0].position, Position::new(1, 12));
    assert!(!tokens.iter().any(|spanned| spanned.token == Token::Indent));
    let error = Runtime::check("if true:\n/*                     */pass").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(
        error.message(),
        "Expected an indented block but found keyword 'pass'"
    );

    for (source, expected) in [
        (
            "left/**/right",
            vec![
                Token::Identifier("left".into()),
                Token::Identifier("right".into()),
                Token::Eof,
            ],
        ),
        (
            "1 +/**/= 2",
            vec![
                Token::Number(1.0),
                Token::Plus,
                Token::Assign,
                Token::Number(2.0),
                Token::Eof,
            ],
        ),
        (
            "first = 1 /*\n*/ second = 2",
            vec![
                Token::Identifier("first".into()),
                Token::Assign,
                Token::Number(1.0),
                Token::Newline,
                Token::Indent,
                Token::Identifier("second".into()),
                Token::Assign,
                Token::Number(2.0),
                Token::Dedent,
                Token::Eof,
            ],
        ),
    ] {
        assert_eq!(
            tokenize(source)
                .unwrap()
                .into_iter()
                .map(|spanned| spanned.token)
                .collect::<Vec<_>>(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn unterminated_blocks_are_syntax_errors_at_the_original_opening_slash() {
    for source in [
        "/*",
        "value = 1\r\n\t/* open\nstill open",
        "value = \"é\"; /* open",
        "value = `é ${1 /* }`",
        "value = `line\né ${(1 + /* }`",
    ] {
        let offset = source.find("/*").unwrap();
        let prefix = &source[..offset];
        let expected = Position::new(
            prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
            prefix.rsplit('\n').next().unwrap().chars().count() + 1,
        );
        let mut runtime = Runtime::new();
        runtime.run("saved = 9").unwrap();
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), "Unterminated block comment", "{source}");
        assert_eq!(error.position(), Some(expected), "{source}");
        assert_eq!(
            error.to_string(),
            format!("SyntaxError: Unterminated block comment ({expected})"),
            "{source}"
        );
        assert_eq!(runtime.state.variable("saved"), Some(&Value::Number(9.0)));
        assert!(!runtime.state.has_variable("value"));
        assert_eq!(runtime.assertions_run(), 0);
    }
}

#[test]
fn strings_regexes_template_text_and_interpolations_keep_their_boundaries() {
    let literals = r#""// /* */ #" '# // /* */' `// /* */ #`
12 / 3; amount /= 2; /[/*#]+/; /a\/\/b/; /=/;"#;
    assert_eq!(
        tokenize(literals)
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        [
            Token::Str("// /* */ #".into()),
            Token::Str("# // /* */".into()),
            Token::Template("// /* */ #".into()),
            Token::Newline,
            Token::Number(12.0),
            Token::Slash,
            Token::Number(3.0),
            Token::Semicolon,
            Token::Identifier("amount".into()),
            Token::SlashAssign,
            Token::Number(2.0),
            Token::Semicolon,
            Token::Regex("[/*#]+".into()),
            Token::Semicolon,
            Token::Regex(r"a\/\/b".into()),
            Token::Semicolon,
            Token::Regex("=".into()),
            Token::Semicolon,
            Token::Eof,
        ]
    );
    checked(
        r##"text = "// /* */ #"
single = '/* // # */'
pattern = /[/*#]+/
escaped = /a\/\/b/
amount = 12
amount /= 2
assert(amount, 6)
assert(text, "// /* */ #")
assert(single, "/* // # */")
assert(pattern.test("/*#"), true)
assert(escaped.test("a//b"), true)
assert(`${/[{}]/.test("}")}`, "true")
result = `// /* # ${6 /* } { */ / 2} ${"}" /* } */} ${(1 // }
+ 2)} ${(1 # }
+ 3)}`
assert(result, "// /* # 3 } 3 4")
result"##,
        r#""// /* # 3 } 3 4""#,
        7,
    );
}

#[test]
fn comments_apply_to_all_source_entry_points_and_survive_clear() {
    let source = "value = 4 /* units */ // done\nvalue";
    let program = parse_program(source).unwrap();
    assert_eq!(program.statements, parse("value = 4\nvalue").unwrap());
    assert_eq!(
        program.positions,
        [Position::new(1, 1), Position::new(2, 1)]
    );
    assert!(Runtime::check(source).is_ok());
    assert_eq!(
        parse_expression("(8 /* units */ / 2)").unwrap(),
        parse_expression("8 / 2").unwrap()
    );
    assert_eq!(
        nucleoid::statement::compile(source).unwrap().statements,
        program.statements
    );
    let mut runtime = Runtime::new();
    assert_eq!(runtime.run(source).unwrap(), Value::Number(4.0));
    runtime.clear();
    assert_eq!(runtime.run(source).unwrap(), Value::Number(4.0));
    assert_eq!(runtime.run("1 // 2 /").unwrap(), Value::Number(1.0));
    assert_eq!(runtime.assertions_run(), 0);
}

#[test]
fn long_suppressed_trivia_runs_preserve_operand_context() {
    let trivia = "\n\n// ignored\n/* ignored\n*/\n".repeat(1024);
    for (head, tail) in [("12", "/ 3 / 2"), ("amount", "/= 3 / 2"), ("value@", "2")] {
        let actual = tokenize(&format!("({head}{trivia}{tail})")).unwrap();
        let control = tokenize(&format!("({head} {tail})")).unwrap();
        assert_eq!(
            actual
                .iter()
                .map(|spanned| &spanned.token)
                .collect::<Vec<_>>(),
            control
                .iter()
                .map(|spanned| &spanned.token)
                .collect::<Vec<_>>(),
            "{head} ... {tail}"
        );
    }
    checked(&format!("(12{trivia}/ 3 / 2)"), "2.0", 0);
}
