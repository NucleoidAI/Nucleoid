use super::*;

fn tokens(source: &str) -> Vec<Token> {
    tokenize(source)
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect()
}

#[test]
fn compound_operators_are_single_tokens_with_exact_descriptions() {
    for (symbol, operator) in [
        ("+=", Token::PlusAssign),
        ("-=", Token::MinusAssign),
        ("*=", Token::StarAssign),
        ("**=", Token::StarStarAssign),
        ("/=", Token::SlashAssign),
        ("%=", Token::PercentAssign),
        ("&&=", Token::AndAndAssign),
        ("||=", Token::OrOrAssign),
    ] {
        let source = format!("value{symbol}-2");
        let spanned = tokenize(&source).unwrap();
        assert_eq!(operator.describe(), format!("'{symbol}'"));
        assert_eq!(spanned[1].position, Position::new(1, 6));
        assert_eq!(
            tokens(&source),
            [
                Token::Identifier("value".into()),
                operator,
                Token::Minus,
                Token::Number(2.0),
                Token::Eof,
            ],
            "{source}"
        );
        assert_eq!(
            spanned
                .iter()
                .filter(|spanned| spanned.token != Token::Eof)
                .count(),
            4
        );
    }
}

#[test]
fn compound_longest_match_keeps_adjacent_signs_and_old_operators() {
    assert_eq!(
        tokens("a+=+1; b-=-2; c*=-3"),
        [
            Token::Identifier("a".into()),
            Token::PlusAssign,
            Token::Plus,
            Token::Number(1.0),
            Token::Semicolon,
            Token::Identifier("b".into()),
            Token::MinusAssign,
            Token::Minus,
            Token::Number(2.0),
            Token::Semicolon,
            Token::Identifier("c".into()),
            Token::StarAssign,
            Token::Minus,
            Token::Number(3.0),
            Token::Eof,
        ]
    );
    assert_eq!(
        tokens("a + = b"),
        [
            Token::Identifier("a".into()),
            Token::Plus,
            Token::Assign,
            Token::Identifier("b".into()),
            Token::Eof,
        ]
    );
    assert_eq!(
        tokens("a==b; a===b; a!=b; a!==b; x=>x"),
        [
            Token::Identifier("a".into()),
            Token::Equal,
            Token::Identifier("b".into()),
            Token::Semicolon,
            Token::Identifier("a".into()),
            Token::StrictEqual,
            Token::Identifier("b".into()),
            Token::Semicolon,
            Token::Identifier("a".into()),
            Token::NotEqual,
            Token::Identifier("b".into()),
            Token::Semicolon,
            Token::Identifier("a".into()),
            Token::StrictNotEqual,
            Token::Identifier("b".into()),
            Token::Semicolon,
            Token::Identifier("x".into()),
            Token::Arrow,
            Token::Identifier("x".into()),
            Token::Eof,
        ]
    );
}

#[test]
fn logical_compounds_keep_symbol_word_and_separated_operators_distinct() {
    assert_eq!(
        tokens("a&&=!b; c||=+2; d&&=-3"),
        [
            Token::Identifier("a".into()),
            Token::AndAndAssign,
            Token::Bang,
            Token::Identifier("b".into()),
            Token::Semicolon,
            Token::Identifier("c".into()),
            Token::OrOrAssign,
            Token::Plus,
            Token::Number(2.0),
            Token::Semicolon,
            Token::Identifier("d".into()),
            Token::AndAndAssign,
            Token::Minus,
            Token::Number(3.0),
            Token::Eof,
        ]
    );
    for (symbol, token) in [
        ("&&", Token::AndAnd),
        ("||", Token::OrOr),
        ("and", Token::Keyword(Keyword::And)),
        ("or", Token::Keyword(Keyword::Or)),
    ] {
        assert_eq!(
            tokens(&format!("a {symbol} b")),
            [
                Token::Identifier("a".into()),
                token.clone(),
                Token::Identifier("b".into()),
                Token::Eof,
            ]
        );
        assert_eq!(
            tokens(&format!("a {symbol} = b")),
            [
                Token::Identifier("a".into()),
                token,
                Token::Assign,
                Token::Identifier("b".into()),
                Token::Eof,
            ]
        );
    }
    let spanned = tokenize("first &&= 1\nsecond ||= 2").unwrap();
    assert_eq!(spanned[1].token, Token::AndAndAssign);
    assert_eq!(spanned[1].position, Position::new(1, 7));
    assert_eq!(spanned[5].token, Token::OrOrAssign);
    assert_eq!(spanned[5].position, Position::new(2, 8));
}

#[test]
fn slash_assign_preserves_division_and_regex_contexts() {
    assert_eq!(
        tokens("amount/=2/3"),
        [
            Token::Identifier("amount".into()),
            Token::SlashAssign,
            Token::Number(2.0),
            Token::Slash,
            Token::Number(3.0),
            Token::Eof,
        ]
    );
    assert_eq!(
        tokens("(amount)/=2"),
        [
            Token::LeftParen,
            Token::Identifier("amount".into()),
            Token::RightParen,
            Token::SlashAssign,
            Token::Number(2.0),
            Token::Eof,
        ]
    );
    for (literal, pattern) in [
        ("/=/", "="),
        ("/==/i", "=="),
        (r"/=\/x/", r"=\/x"),
        ("/[=/]+/", "[=/]+"),
        ("/abc/", "abc"),
        ("/&&=/", "&&="),
        ("/[|&=]+/", "[|&=]+"),
    ] {
        for prefix in ["", "\n", "# preceding comment\n"] {
            let source = format!("{prefix}{literal}");
            assert_eq!(
                tokens(&source),
                [Token::Regex(pattern.into()), Token::Eof],
                "{source}"
            );
        }
        let source = format!("{literal}/2");
        assert_eq!(
            tokens(&source),
            [
                Token::Regex(pattern.into()),
                Token::Slash,
                Token::Number(2.0),
                Token::Eof,
            ],
            "{source}"
        );
        for assignment in ["=", "+=", "-=", "*=", "**=", "/=", "%=", "&&=", "||="] {
            let source = format!("pattern {assignment} {literal}");
            let tokenized = tokens(&source);
            assert_eq!(tokenized[2], Token::Regex(pattern.into()), "{source}");
            assert_eq!(tokenized.len(), 4, "{source}");
        }
    }

    let spanned = tokenize("pattern = /=/i\namount /= 2").unwrap();
    assert_eq!(spanned[2].token, Token::Regex("=".into()));
    assert_eq!(spanned[2].position, Position::new(1, 11));
    assert_eq!(spanned[5].token, Token::SlashAssign);
    assert_eq!(spanned[5].position, Position::new(2, 8));
}

#[test]
fn thin_arrows_are_single_adjacent_tokens_named_as_written() {
    let identifier = |name: &str| Token::Identifier(name.into());
    assert_eq!(Token::ThinArrow.describe(), "'->'");
    assert_eq!(Token::Arrow.describe(), "'=>'");
    assert!(Token::ThinArrow.is_arrow() && Token::Arrow.is_arrow());
    assert!(!Token::Minus.is_arrow() && !Token::Greater.is_arrow());
    for (source, mut expected) in [
        (
            "x->x",
            vec![identifier("x"), Token::ThinArrow, identifier("x")],
        ),
        (
            "x - > y",
            vec![
                identifier("x"),
                Token::Minus,
                Token::Greater,
                identifier("y"),
            ],
        ),
        (
            "x-->y",
            vec![
                identifier("x"),
                Token::Minus,
                Token::ThinArrow,
                identifier("y"),
            ],
        ),
        (
            "a-=>b",
            vec![
                identifier("a"),
                Token::MinusAssign,
                Token::Greater,
                identifier("b"),
            ],
        ),
        (
            "a->=b",
            vec![
                identifier("a"),
                Token::ThinArrow,
                Token::Assign,
                identifier("b"),
            ],
        ),
        (
            "a>-1",
            vec![
                identifier("a"),
                Token::Greater,
                Token::Minus,
                Token::Number(1.0),
            ],
        ),
        (
            "x => y",
            vec![identifier("x"), Token::Arrow, identifier("y")],
        ),
        (
            "x -> /a->b/",
            vec![
                identifier("x"),
                Token::ThinArrow,
                Token::Regex("a->b".into()),
            ],
        ),
        ("\"a->b\"", vec![Token::Str("a->b".into())]),
        ("'a->b'", vec![Token::Str("a->b".into())]),
        ("`a->${b}`", vec![Token::Template("a->${b}".into())]),
        ("x # a->b", vec![identifier("x")]),
    ] {
        expected.push(Token::Eof);
        assert_eq!(tokens(source), expected, "{source}");
    }
    let spanned = tokenize("f = value ->\n    value").unwrap();
    assert_eq!(spanned[3].token, Token::ThinArrow);
    assert_eq!(spanned[3].position, Position::new(1, 11));
}

#[test]
fn question_marks_are_single_tokens_and_keep_literal_text() {
    let identifier = |name: &str| Token::Identifier(name.into());
    assert_eq!(Token::Question.describe(), "'?'");
    for (source, mut expected) in [
        (
            "a?b:c",
            vec![
                identifier("a"),
                Token::Question,
                identifier("b"),
                Token::Colon,
                identifier("c"),
            ],
        ),
        (
            "a ?? b",
            vec![identifier("a"), Token::QuestionQuestion, identifier("b")],
        ),
        (
            "a ? ? b",
            vec![
                identifier("a"),
                Token::Question,
                Token::Question,
                identifier("b"),
            ],
        ),
        (
            "a?.b",
            vec![
                identifier("a"),
                Token::Question,
                Token::Dot,
                identifier("b"),
            ],
        ),
        (
            "a?.5:1",
            vec![
                identifier("a"),
                Token::Question,
                Token::Number(0.5),
                Token::Colon,
                Token::Number(1.0),
            ],
        ),
        (
            "a ?: b",
            vec![
                identifier("a"),
                Token::Question,
                Token::Colon,
                identifier("b"),
            ],
        ),
        (
            "c ? /a?/ : /b/",
            vec![
                identifier("c"),
                Token::Question,
                Token::Regex("a?".into()),
                Token::Colon,
                Token::Regex("b".into()),
            ],
        ),
        ("\"why?\"", vec![Token::Str("why?".into())]),
        ("'why?'", vec![Token::Str("why?".into())]),
        ("`why? ${a}`", vec![Token::Template("why? ${a}".into())]),
        ("/colou?r/", vec![Token::Regex("colou?r".into())]),
        ("x # why?", vec![identifier("x")]),
    ] {
        expected.push(Token::Eof);
        assert_eq!(tokens(source), expected, "{source}");
    }
    let spanned = tokenize("x = c ? 1 : 2").unwrap();
    assert_eq!(spanned[3].token, Token::Question);
    assert_eq!(spanned[3].position, Position::new(1, 7));
    assert_eq!(
        tokens("a ? b / 2 : c"),
        [
            identifier("a"),
            Token::Question,
            identifier("b"),
            Token::Slash,
            Token::Number(2.0),
            Token::Colon,
            identifier("c"),
            Token::Eof,
        ]
    );
}

#[test]
fn double_question_marks_are_one_token_outside_literal_text() {
    let identifier = |name: &str| Token::Identifier(name.into());
    assert_eq!(Token::QuestionQuestion.describe(), "'??'");
    for (source, mut expected) in [
        (
            "a??b",
            vec![identifier("a"), Token::QuestionQuestion, identifier("b")],
        ),
        (
            "a ??? b",
            vec![
                identifier("a"),
                Token::QuestionQuestion,
                Token::Question,
                identifier("b"),
            ],
        ),
        (
            "a ?? ? b",
            vec![
                identifier("a"),
                Token::QuestionQuestion,
                Token::Question,
                identifier("b"),
            ],
        ),
        (
            "a????b",
            vec![
                identifier("a"),
                Token::QuestionQuestion,
                Token::QuestionQuestion,
                identifier("b"),
            ],
        ),
        (
            "a??.5",
            vec![identifier("a"), Token::QuestionQuestion, Token::Number(0.5)],
        ),
        (
            "a ?? /b?/",
            vec![
                identifier("a"),
                Token::QuestionQuestion,
                Token::Regex("b?".into()),
            ],
        ),
        (
            "a ??= b",
            vec![
                identifier("a"),
                Token::QuestionQuestion,
                Token::Assign,
                identifier("b"),
            ],
        ),
        (
            "c ? a ?? b : d",
            vec![
                identifier("c"),
                Token::Question,
                identifier("a"),
                Token::QuestionQuestion,
                identifier("b"),
                Token::Colon,
                identifier("d"),
            ],
        ),
        ("\"why??\"", vec![Token::Str("why??".into())]),
        ("'why??'", vec![Token::Str("why??".into())]),
        (
            "`why?? ${a ?? b}`",
            vec![Token::Template("why?? ${a ?? b}".into())],
        ),
        ("/colou??r/", vec![Token::Regex("colou??r".into())]),
        ("x # a ?? b", vec![identifier("x")]),
    ] {
        expected.push(Token::Eof);
        assert_eq!(tokens(source), expected, "{source}");
    }
    let spanned = tokenize("x = a ?? b").unwrap();
    assert_eq!(spanned[3].token, Token::QuestionQuestion);
    assert_eq!(spanned[3].position, Position::new(1, 7));
    assert_eq!(spanned[4].position, Position::new(1, 10));
}

#[test]
fn double_stars_are_single_tokens_outside_literal_text() {
    let identifier = |name: &str| Token::Identifier(name.into());
    assert_eq!(Token::StarStar.describe(), "'**'");
    assert_eq!(Token::StarStarAssign.describe(), "'**='");
    for (source, mut expected) in [
        (
            "a**b",
            vec![identifier("a"), Token::StarStar, identifier("b")],
        ),
        (
            "a * * b",
            vec![identifier("a"), Token::Star, Token::Star, identifier("b")],
        ),
        (
            "a***b",
            vec![
                identifier("a"),
                Token::StarStar,
                Token::Star,
                identifier("b"),
            ],
        ),
        (
            "a****b",
            vec![
                identifier("a"),
                Token::StarStar,
                Token::StarStar,
                identifier("b"),
            ],
        ),
        (
            "a**=b",
            vec![identifier("a"), Token::StarStarAssign, identifier("b")],
        ),
        (
            "a* *=b",
            vec![
                identifier("a"),
                Token::Star,
                Token::StarAssign,
                identifier("b"),
            ],
        ),
        (
            "a ** = b",
            vec![
                identifier("a"),
                Token::StarStar,
                Token::Assign,
                identifier("b"),
            ],
        ),
        (
            "a**==b",
            vec![
                identifier("a"),
                Token::StarStarAssign,
                Token::Assign,
                identifier("b"),
            ],
        ),
        (
            "a***=b",
            vec![
                identifier("a"),
                Token::StarStar,
                Token::StarAssign,
                identifier("b"),
            ],
        ),
        (
            "a*=*b",
            vec![
                identifier("a"),
                Token::StarAssign,
                Token::Star,
                identifier("b"),
            ],
        ),
        (
            "2**-1",
            vec![
                Token::Number(2.0),
                Token::StarStar,
                Token::Minus,
                Token::Number(1.0),
            ],
        ),
        (
            "a**.5",
            vec![identifier("a"), Token::StarStar, Token::Number(0.5)],
        ),
        (
            "a ** /x**y/",
            vec![
                identifier("a"),
                Token::StarStar,
                Token::Regex("x**y".into()),
            ],
        ),
        (
            "a **= /=/",
            vec![
                identifier("a"),
                Token::StarStarAssign,
                Token::Regex("=".into()),
            ],
        ),
        (
            "a ** b / 2",
            vec![
                identifier("a"),
                Token::StarStar,
                identifier("b"),
                Token::Slash,
                Token::Number(2.0),
            ],
        ),
        ("/x**y/", vec![Token::Regex("x**y".into())]),
        ("/a**/i", vec![Token::Regex("a**".into())]),
        ("\"2**8\"", vec![Token::Str("2**8".into())]),
        ("'**='", vec![Token::Str("**=".into())]),
        (
            "`2**${a ** b}`",
            vec![Token::Template("2**${a ** b}".into())],
        ),
        ("x # a ** b", vec![identifier("x")]),
    ] {
        expected.push(Token::Eof);
        assert_eq!(tokens(source), expected, "{source}");
    }
    let spanned = tokenize("x = a ** b\nx **= 2").unwrap();
    assert_eq!(spanned[3].token, Token::StarStar);
    assert_eq!(spanned[3].position, Position::new(1, 7));
    assert_eq!(spanned[7].token, Token::StarStarAssign);
    assert_eq!(spanned[7].position, Position::new(2, 3));
}

#[test]
fn colon_assign_is_one_adjacent_token_outside_literal_text() {
    let identifier = |name: &str| Token::Identifier(name.into());
    assert_eq!(Token::ColonAssign.describe(), "':='");
    for (source, mut expected) in [
        (
            "x:=1",
            vec![identifier("x"), Token::ColonAssign, Token::Number(1.0)],
        ),
        (
            "x := 1",
            vec![identifier("x"), Token::ColonAssign, Token::Number(1.0)],
        ),
        (
            "x : = 1",
            vec![
                identifier("x"),
                Token::Colon,
                Token::Assign,
                Token::Number(1.0),
            ],
        ),
        (
            "x: =1",
            vec![
                identifier("x"),
                Token::Colon,
                Token::Assign,
                Token::Number(1.0),
            ],
        ),
        (
            "x :== 1",
            vec![
                identifier("x"),
                Token::ColonAssign,
                Token::Assign,
                Token::Number(1.0),
            ],
        ),
        (
            "x :=> 1",
            vec![
                identifier("x"),
                Token::ColonAssign,
                Token::Greater,
                Token::Number(1.0),
            ],
        ),
        (
            "x ::= 1",
            vec![
                identifier("x"),
                Token::Colon,
                Token::ColonAssign,
                Token::Number(1.0),
            ],
        ),
        (
            "x ?:= 1",
            vec![
                identifier("x"),
                Token::Question,
                Token::ColonAssign,
                Token::Number(1.0),
            ],
        ),
        (
            "x := -1",
            vec![
                identifier("x"),
                Token::ColonAssign,
                Token::Minus,
                Token::Number(1.0),
            ],
        ),
        (
            "x := /=/",
            vec![
                identifier("x"),
                Token::ColonAssign,
                Token::Regex("=".into()),
            ],
        ),
        (
            "a ? b : c",
            vec![
                identifier("a"),
                Token::Question,
                identifier("b"),
                Token::Colon,
                identifier("c"),
            ],
        ),
        (
            "xs[1:2]",
            vec![
                identifier("xs"),
                Token::LeftBracket,
                Token::Number(1.0),
                Token::Colon,
                Token::Number(2.0),
                Token::RightBracket,
            ],
        ),
        ("\"a:=b\"", vec![Token::Str("a:=b".into())]),
        ("'a:=b'", vec![Token::Str("a:=b".into())]),
        ("`${x:=2}`", vec![Token::Template("${x:=2}".into())]),
        ("/a:=b/", vec![Token::Regex("a:=b".into())]),
        ("x # a := b", vec![identifier("x")]),
    ] {
        expected.push(Token::Eof);
        assert_eq!(tokens(source), expected, "{source}");
    }
    let spanned = tokenize("stock := 5\nstock = 6").unwrap();
    assert_eq!(spanned[1].token, Token::ColonAssign);
    assert_eq!(spanned[1].position, Position::new(1, 7));
    assert_eq!(spanned[5].token, Token::Assign);
    assert_eq!(spanned[5].position, Position::new(2, 7));
    assert_eq!(
        spanned
            .iter()
            .filter(|spanned| spanned.token != Token::Eof)
            .count(),
        7
    );
}
