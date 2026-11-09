use super::*;
use crate::lang::ast::generator::{generate, generate_all};
use crate::lang::ast::lexer::tokenize;
use crate::lang::ast::{FunctionBody, Noun};

#[test]
fn direct_condition_lookahead_requires_balanced_header_and_real_suite_without_consuming() {
    for (header, qualifies) in [
        ("ready:\n    pass", true),
        ("not ready and available:\n    pass", true),
        ("items[1:3] == [2, 3]:\n    pass", true),
        ("choose({ready: items[:]}, (x: int) => x):\n    pass", true),
        ("({ready: {nested: true}}).ready:\n    pass", true),
        ("(\n ready\n):\n    pass", true),
        ("ready: # header\n\n# blank\n    # body\n    pass", true),
        ("ready:\r\n\tpass", true),
        ("ready:\n;\n    pass", true),
        ("ready:\n    ;\n    pass", true),
        ("ready\n    pass", true),
        ("items[1:3] == [2, 3]\n    pass", true),
        ("choose({ready: items[:]}, (x: int) => x)\r\n\tpass", true),
        ("ready # header\n\n# blank\n    # body\n    pass", true),
        ("ready\n;\n    pass", true),
        ("ready\n    ;\n    pass", true),
        // A top-level conditional expression's colon pairs with its `?`.
        ("ready ? on : off:\n    pass", true),
        ("ready ? on : off\n    pass", true),
        ("a ? b ? c : d : e:\n    pass", true),
        ("a ? b : c ? d : e\n    pass", true),
        ("(ready ? on : off):\n    pass", true),
        ("on if ready else off:\n    pass", true),
        ("ready ? (x: int) => x : off:\n    pass", true),
        // Qualification is not expression validation. Real parsing must
        // report these malformed expressions rather than fall back.
        ("ready +:\n    pass", true),
        ("():\n    pass", true),
        ("pass:\n    pass", true),
        ("ready +\n    pass", true),
        ("ready ? on:\n    pass", true),
        ("ready ?\n    pass", true),
        ("(ready ? on):\n    pass", true),
        (":\n    pass", false),
        ("\n    pass", false),
        ("ready: pass", false),
        ("ready: { pass }", false),
        ("ready { pass }", false),
        ("ready:;\n    pass", false),
        ("ready:\npass", false),
        ("ready:\n# no body", false),
        ("ready:\n    # no body", false),
        ("ready:", false),
        ("ready", false),
        ("ready;\n    pass", false),
        ("ready\npass", false),
        ("ready\n    # no body", false),
        ("(ready]\n    pass", false),
        ("{ready: true\n    pass", false),
        ("ready\n:\n    pass", false),
        ("ready; other:\n    pass", false),
        ("ready # boundary\nother:\n    pass", false),
        ("items[:]\nother:\n    pass", false),
        ("(ready]:\n    pass", false),
        ("[ready):\n    pass", false),
        ("{ready: true]:\n    pass", false),
        ("ready):\n    pass", false),
        ("(ready:\n    pass", false),
        ("{ready: true:\n    pass", false),
        ("ready ? on : off: pass", false),
        ("ready ? on : off", false),
        ("ready ? on : off:", false),
        ("ready ? on", false),
        ("ready ? on : off; other:\n    pass", false),
    ] {
        let mut parser = Parser::new(tokenize(&format!("else {header}")).unwrap());
        parser.expect(&Token::Keyword(Keyword::Else)).unwrap();
        let position = parser.position;
        let tokens = parser.tokens.clone();
        assert_eq!(parser.has_direct_condition_suite(), qualifies, "{header}");
        assert_eq!(parser.position, position, "{header}");
        assert_eq!(parser.depth, 0, "{header}");
        assert_eq!(parser.tokens, tokens, "{header}");
    }
}

#[test]
fn direct_condition_lookahead_pairs_quantifier_colons_without_consuming() {
    for (header, qualifies) in [
        ("all v in xs: v > 0:\n    pass", true),
        ("all v in xs: v > 0\n    pass", true),
        ("any v in xs: v ? on : off:\n    pass", true),
        ("ready ? all v in xs: v : off:\n    pass", true),
        ("all t in ts: any w in ws: w == t:\n    pass", true),
        ("all v in ready ? xs : ys: v\n    pass", true),
        ("all v in xs: v ?? false:\n    pass", true),
        ("(all v in xs: v):\n    pass", true),
        ("all v in xs:\n    pass", true),
        ("all v in xs: v: pass", false),
        ("all v in xs: v", false),
        ("all v in xs: v;\n    pass", false),
        ("all v of xs: v:\n    pass", false),
        ("all: v\n    pass", false),
    ] {
        let mut parser = Parser::new(tokenize(&format!("else {header}")).unwrap());
        parser.expect(&Token::Keyword(Keyword::Else)).unwrap();
        let position = parser.position;
        let tokens = parser.tokens.clone();
        assert_eq!(parser.has_direct_condition_suite(), qualifies, "{header}");
        assert_eq!(parser.position, position, "{header}");
        assert_eq!(parser.depth, 0, "{header}");
        assert_eq!(parser.tokens, tokens, "{header}");
    }
}

#[test]
fn direct_condition_lookahead_stops_at_a_top_level_value_assignment() {
    for (header, qualifies) in [
        ("ready := 1:\n    pass", false),
        ("ready := 1\n    pass", false),
        ("x := ready ? 1 : 2:\n    pass", false),
        ("f(x := 1):\n    pass", true),
        ("ready:\n    x := 1", true),
        ("ready\n    x := 1", true),
    ] {
        let mut parser = Parser::new(tokenize(&format!("else {header}")).unwrap());
        parser.expect(&Token::Keyword(Keyword::Else)).unwrap();
        let position = parser.position;
        let tokens = parser.tokens.clone();
        assert_eq!(parser.has_direct_condition_suite(), qualifies, "{header}");
        assert_eq!(parser.position, position, "{header}");
        assert_eq!(parser.depth, 0, "{header}");
        assert_eq!(parser.tokens, tokens, "{header}");
    }
}

#[test]
fn direct_condition_lookahead_reuses_immediate_indent_guard_and_stops_at_layout() {
    let mut parser = Parser::new(tokenize("ready:\n    pass").unwrap());
    assert!(parser.has_direct_condition_suite());
    parser.tokens.remove(2);
    assert_eq!(parser.peek_at(2), &Token::Indent);
    assert!(parser.has_direct_condition_suite());
    parser.tokens.insert(
        2,
        Spanned {
            token: Token::Semicolon,
            position: Position::new(1, 7),
        },
    );
    assert!(!parser.has_direct_condition_suite());
    assert_eq!(parser.position, 0);
    for layout in [
        vec![Token::Newline, Token::Indent],
        vec![Token::Indent],
        vec![Token::Newline, Token::Semicolon, Token::Indent],
    ] {
        parser.tokens = std::iter::once(Token::Identifier("ready".into()))
            .chain(layout)
            .chain([Token::Keyword(Keyword::Pass), Token::Eof])
            .map(|token| Spanned {
                token,
                position: Position::new(1, 1),
            })
            .collect();
        let tokens = parser.tokens.clone();
        assert!(parser.has_direct_condition_suite());
        assert_eq!(parser.position, 0);
        assert_eq!(parser.depth, 0);
        assert_eq!(parser.tokens, tokens);
    }
    for boundary in [
        Token::Newline,
        Token::Semicolon,
        Token::Indent,
        Token::Dedent,
        Token::Eof,
    ] {
        let qualifies = boundary == Token::Indent;
        parser.tokens = [
            Token::Identifier("ready".into()),
            boundary,
            Token::Colon,
            Token::Newline,
            Token::Indent,
            Token::Keyword(Keyword::Pass),
            Token::Eof,
        ]
        .into_iter()
        .map(|token| Spanned {
            token,
            position: Position::new(1, 1),
        })
        .collect();
        let tokens = parser.tokens.clone();
        assert_eq!(parser.has_direct_condition_suite(), qualifies);
        assert_eq!(parser.position, 0);
        assert_eq!(parser.tokens, tokens);
    }
}

#[test]
fn comma_free_arrow_lookahead_qualifies_the_whole_prefix_without_consuming() {
    for (header, end) in [
        ("left right => left", Some(2)),
        ("left right factor => left", Some(3)),
        ("left\tright=>left", Some(2)),
        ("left => left", None),
        ("left, right => left", None),
        ("left: int, right => left", None),
        ("left: int right => left", None),
        ("left right: int => left", None),
        ("left right, third => left", None),
        ("left, right third => left", None),
        ("left right:\n    return left", None),
        ("left right\n=> left", None),
        ("left # boundary\nright => left", None),
        ("left right; => left", None),
        ("left this => left", None),
        ("left right", None),
    ] {
        let mut parser = Parser::new(tokenize(&format!("def f {header}")).unwrap());
        parser.expect(&Token::Keyword(Keyword::Def)).unwrap();
        assert_eq!(parser.advance(), Token::Identifier("f".to_string()));
        let original = parser.position;
        assert_eq!(parser.comma_free_arrow_end(), end, "{header}");
        assert_eq!(parser.position, original, "{header}");
        assert_eq!(parser.peek(), &Token::Identifier("left".to_string()));
    }
}

#[test]
fn comma_free_block_lookahead_requires_the_whole_header_and_real_suite_without_consuming() {
    for (header, end) in [
        ("left right:\n    return left", Some(2)),
        ("left right factor:\n    return left", Some(3)),
        ("left\tright:\r\n\treturn left", Some(2)),
        ("same same:\n    return same", Some(2)),
        (
            "type value _private A1 why affects model:\n    pass",
            Some(7),
        ),
        (
            "left right: # header\n\n# blank\n    # body\n    pass",
            Some(2),
        ),
        ("left right:\n;\n    pass", Some(2)),
        ("left right:\n    ;\n    pass", Some(2)),
        ("left right:\n    int => left", Some(2)),
        ("left:\n    pass", None),
        ("left, right:\n    pass", None),
        ("left: int, right:\n    pass", None),
        ("left: int right:\n    pass", None),
        ("left right: int:\n    pass", None),
        ("left: int right: int:\n    pass", None),
        ("left right, third:\n    pass", None),
        ("left, right third:\n    pass", None),
        ("left right,:\n    pass", None),
        ("left right third, fourth:\n    pass", None),
        ("left right => left", None),
        ("left right", None),
        ("left right:", None),
        ("left right:\n", None),
        ("left right:\n    # no body", None),
        ("left right:\nreturn left", None),
        ("left right:\n;\nreturn left", None),
        ("left right:;\n    pass", None),
        ("left right: return left", None),
        ("left right: { return left }", None),
        ("left right:\n{ return left }", None),
        ("left right { return left }", None),
        ("left right\n:\n    pass", None),
        ("left\nright:\n    pass", None),
        ("left right;:\n    pass", None),
        ("left; right:\n    pass", None),
        ("left right # boundary\n:\n    pass", None),
        ("left # boundary\nright:\n    pass", None),
        ("left this:\n    pass", None),
        ("left self:\n    pass", None),
        ("left null:\n    pass", None),
        ("left $Right:\n    pass", None),
        ("left 1:\n    pass", None),
        ("left \"right\":\n    pass", None),
        ("left = 1 right:\n    pass", None),
        ("left right = 1:\n    pass", None),
        ("left ...right:\n    pass", None),
        ("left [right]:\n    pass", None),
        ("left {right}:\n    pass", None),
        ("left right -> int:\n    pass", None),
        ("this right:\n    pass", None),
        ("$Left right:\n    pass", None),
        ("1 right:\n    pass", None),
        ("(left right):\n    pass", None),
        (":\n    pass", None),
    ] {
        let mut parser = Parser::new(tokenize(&format!("def f {header}")).unwrap());
        parser.expect(&Token::Keyword(Keyword::Def)).unwrap();
        assert_eq!(parser.advance(), Token::Identifier("f".to_string()));
        let position = parser.position;
        let tokens = parser.tokens.clone();
        assert_eq!(parser.comma_free_block_end(), end, "{header}");
        assert_eq!(parser.position, position, "{header}");
        assert_eq!(parser.depth, 0, "{header}");
        assert_eq!(parser.tokens, tokens, "{header}");
    }
}

#[test]
fn lambda_lookahead_qualifies_whole_headers_by_context_without_consuming() {
    use ExpressionContext::{Clause, Interpolation, ObjectValue, Ordinary};
    for (source, context, count) in [
        ("lambda: 1", Ordinary, Some(0)),
        ("lambda: value", Ordinary, Some(0)),
        ("lambda x: x + 1", Ordinary, Some(1)),
        ("lambda x: x.name", Ordinary, Some(1)),
        ("lambda x: x)", Ordinary, Some(1)),
        ("lambda x: x }", Ordinary, Some(1)),
        ("lambda x: int: x", Ordinary, Some(1)),
        ("lambda x: 1", Ordinary, Some(1)),
        ("lambda a, b: a", Ordinary, Some(2)),
        ("lambda a, b, c: a", Ordinary, Some(3)),
        ("lambda a,\tb : a", Ordinary, Some(2)),
        // A single name, a colon and one name before a statement boundary
        // is an expression followed by a typed declaration.
        ("lambda x: x", Ordinary, None),
        ("lambda x: int\n", Ordinary, None),
        ("lambda x: int; other", Ordinary, None),
        ("lambda x: int # comment", Ordinary, None),
        ("lambda a, b: int", Ordinary, Some(2)),
        ("lambda: int", Ordinary, Some(0)),
        // NUC45: a single `assert` and its colon start an assertion
        // statement after the expression, whatever follows the colon.
        ("lambda assert: x, 1", Ordinary, None),
        ("lambda assert: 1, 1", Ordinary, None),
        ("lambda assert: x", Ordinary, None),
        ("lambda assert: int", Ordinary, None),
        ("lambda assert:", Ordinary, None),
        ("lambda assert : x, 1", Ordinary, None),
        ("lambda assert: x, 1", ObjectValue, None),
        ("lambda assert: x, 1", Clause, None),
        ("lambda assert, b: 1", Ordinary, Some(2)),
        ("lambda a, assert: 1", Ordinary, Some(2)),
        ("lambda Assert: x, 1", Ordinary, Some(1)),
        ("lambda asserts: x, 1", Ordinary, Some(1)),
        // NUC44: one name and a colon in an object value is a property.
        ("lambda x: 1", ObjectValue, None),
        ("lambda x: x", ObjectValue, None),
        ("lambda: 1", ObjectValue, Some(0)),
        ("lambda a, b: 1", ObjectValue, Some(2)),
        // Clause headers, subscript starts, conditional consequents and
        // interpolations own it.
        ("lambda: 1", Clause, None),
        ("lambda x: 1", Clause, None),
        ("lambda a, b: 1", Clause, None),
        ("lambda: 1", Interpolation, None),
        ("lambda x: 1", Interpolation, None),
        ("lambda a, b: 1", Interpolation, None),
        ("lambda", Ordinary, None),
        ("lambda x", Ordinary, None),
        ("lambda x, : x", Ordinary, None),
        ("lambda x,: x", Ordinary, None),
        ("lambda , x: x", Ordinary, None),
        ("lambda x y: x", Ordinary, None),
        ("lambda x = 1: x", Ordinary, None),
        ("lambda (x): x", Ordinary, None),
        ("lambda [x]: x", Ordinary, None),
        ("lambda *x: x", Ordinary, None),
        ("lambda this: 1", Ordinary, None),
        ("lambda self: 1", Ordinary, None),
        ("lambda null: 1", Ordinary, None),
        ("lambda $Type: 1", Ordinary, None),
        ("lambda 1: 1", Ordinary, None),
        ("lambda \"x\": 1", Ordinary, None),
        ("lambda => 1", Ordinary, None),
        ("lambda -> 1", Ordinary, None),
        ("lambda(x)", Ordinary, None),
        ("lambda.x", Ordinary, None),
        ("lambda[0]", Ordinary, None),
        ("lambda = 1", Ordinary, None),
        ("lambda\n: 1", Ordinary, None),
        ("lambda x\n: 1", Ordinary, None),
        ("lambda; x: 1", Ordinary, None),
        ("Lambda: 1", Ordinary, None),
        ("lambdas: 1", Ordinary, None),
        ("x: 1", Ordinary, None),
    ] {
        let mut parser = Parser::new(tokenize(source).unwrap());
        parser.context = context;
        let tokens = parser.tokens.clone();
        assert_eq!(
            parser.lambda_parameter_count(),
            count,
            "{source} in {context:?}"
        );
        assert_eq!(parser.position, 0, "{source}");
        assert_eq!(parser.depth, 0, "{source}");
        assert_eq!(parser.tokens, tokens, "{source}");
    }
}

#[test]
fn noun_query_lookahead_qualifies_a_noun_and_of_by_context_without_consuming() {
    use ExpressionContext::{Clause, Interpolation, ObjectValue, Ordinary};
    for (source, context, noun) in [
        ("sum of xs", Ordinary, Some(Noun::Sum)),
        ("max of xs", Ordinary, Some(Noun::Max)),
        ("min of xs", Ordinary, Some(Noun::Min)),
        ("mode of xs", Ordinary, Some(Noun::Mode)),
        ("mode of", Ordinary, Some(Noun::Mode)),
        ("sum of of", Ordinary, Some(Noun::Sum)),
        ("sum of xs", Clause, Some(Noun::Sum)),
        ("sum of xs", ObjectValue, Some(Noun::Sum)),
        // An interpolation ignores trailing tokens, so a leading noun
        // stays the name it was at the accepted revision.
        ("sum of xs", Interpolation, None),
        ("sum", Ordinary, None),
        ("sum = 1", Ordinary, None),
        ("sum.of", Ordinary, None),
        ("sum(of)", Ordinary, None),
        ("sum\nof xs", Ordinary, None),
        ("sum in xs", Ordinary, None),
        ("count of xs", Ordinary, None),
        ("distinct of xs", Ordinary, None),
        ("sort of xs", Ordinary, None),
        ("Sum of xs", Ordinary, None),
        ("sums of xs", Ordinary, None),
        ("$sum of xs", Ordinary, None),
        ("\"sum\" of xs", Ordinary, None),
        ("of xs", Ordinary, None),
    ] {
        let mut parser = Parser::new(tokenize(source).unwrap());
        parser.context = context;
        let tokens = parser.tokens.clone();
        assert_eq!(parser.noun_query(), noun, "{source} in {context:?}");
        assert_eq!(parser.position, 0, "{source}");
        assert_eq!(parser.depth, 0, "{source}");
        assert_eq!(parser.tokens, tokens, "{source}");
    }
}

#[test]
fn noun_queries_take_a_unary_operand_while_loop_headers_keep_their_names() {
    let mut parser = Parser::new(tokenize("sum of xs.map(f) + 1").unwrap());
    let expression = parser.expression().unwrap();
    assert_eq!(parser.depth, 0);
    assert_eq!(parser.context, ExpressionContext::Ordinary);
    assert_eq!(expression.to_string(), "sum of xs.map(f)+1");
    let Expr::Binary { left, .. } = &expression else {
        panic!("the sum is the left operand: {expression:?}");
    };
    assert!(matches!(
        left.as_ref(),
        Expr::Of {
            noun: Noun::Sum,
            operand,
        } if matches!(operand.as_ref(), Expr::Call { .. })
    ));
    assert_eq!(
        generate_all(&parse("for sum of xs:\n    pass").unwrap()),
        "for sum of xs"
    );
    assert_eq!(
        generate_all(&parse("for k, max of rec:\n    pass").unwrap()),
        "for k,max of rec"
    );
    assert_eq!(
        generate_all(&parse("for x of mode of rows:\n    pass").unwrap()),
        "for x of mode of rows"
    );
}

#[test]
fn colon_statement_lookahead_uses_the_statement_parser_guards() {
    for (source, offset, starts) in [
        ("value: int", 0, true),
        ("value: int\nnext", 0, true),
        ("value: int; next", 0, true),
        ("value: int # comment", 0, true),
        ("assert: value, 1", 0, true),
        ("assert : value, 1", 0, true),
        ("assert: int", 0, true),
        ("assert: value", 0, true),
        ("assert:", 0, true),
        ("lambda x: int", 1, true),
        ("lambda assert: x, 1", 1, true),
        ("value: 1", 0, false),
        ("value: int other", 0, false),
        ("value: int }", 0, false),
        ("value: $Type", 0, false),
        ("value", 0, false),
        ("value, other: int", 0, false),
        ("Assert: value, 1", 0, false),
        ("asserts: value, 1", 0, false),
        ("assert(value, 1)", 0, false),
        ("assert = 1", 0, false),
        (": int", 0, false),
        ("this: int", 0, false),
        ("lambda x: 1", 1, false),
        ("lambda x: x + 1", 1, false),
    ] {
        let parser = Parser::new(tokenize(source).unwrap());
        assert_eq!(
            parser.starts_colon_statement_at(offset),
            starts,
            "{source} at {offset}"
        );
        assert_eq!(parser.position, 0, "{source}");
    }
    // The `assert:` branch of the statement parser uses the same guard.
    let statements = parse("assert: value, 1\nassert : value, 1").unwrap();
    assert_eq!(statements.len(), 2);
    assert_eq!(statements[0], statements[1]);
    assert_eq!(statements[0], parse("assert(value, 1)").unwrap()[0]);
    // The declaration guard checks the leading name and its colon itself,
    // so neither its branches nor the lambda scan recheck them.
    for (source, declares) in [
        ("value: int", true),
        ("value: int\nnext", true),
        ("value other int", false),
        ("value = int", false),
        ("1: int", false),
        ("this: int", false),
    ] {
        let parser = Parser::new(tokenize(source).unwrap());
        assert_eq!(parser.is_type_declaration_at(0), declares, "{source}");
    }
    assert_eq!(
        parse("value: int").unwrap(),
        [Stmt::Declaration {
            name: "value".into(),
            type_name: "int".into(),
        }]
    );
    let statements = parse("class Holder:\n    value: int").unwrap();
    let [Stmt::Class(class)] = statements.as_slice() else {
        panic!("one class with a typed field");
    };
    assert_eq!(class.constructor, []);
    let [field] = class.fields.as_slice() else {
        panic!("one typed field");
    };
    assert_eq!(field.name, "value");
    assert_eq!(field.type_name.as_deref(), Some("int"));
}

#[test]
fn python_conditions_consume_nothing_unless_else_follows_and_reuse_outcomes() {
    for (source, qualifies) in [
        ("a if b else c", true),
        ("a if b elif c", true),
        ("a if not b or c else d", true),
        ("a if lambda else c", true),
        ("a if x => x else c", true),
        ("a if (b if c else d) else e", true),
        ("a if b: c", false),
        ("a if b\n    c", false),
        ("a if b; c", false),
        ("a if b c else d", false),
        ("a if b", false),
        ("a if (b else c", false),
        ("a if lambda: 1 else c", false),
        ("a if b ? c : d else e", false),
        ("a if b if c else d else e", false),
        ("a if b = 1 else c", false),
        ("a if", false),
    ] {
        let mut parser = Parser::new(tokenize(source).unwrap());
        parser.advance();
        let start = parser.position;
        let tokens = parser.tokens.clone();
        let first = parser.python_condition();
        assert_eq!(first.is_some(), qualifies, "{source}");
        if !qualifies {
            assert_eq!(parser.position, start, "{source}");
        }
        assert_eq!(parser.depth, 0, "{source}");
        assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
        let recorded = parser.python_conditions.len();
        assert!(
            parser.python_conditions.contains_key(&(start, 0)),
            "{source}"
        );
        let end = parser.position;
        parser.position = start;
        assert_eq!(parser.python_condition(), first, "{source}");
        assert_eq!(parser.position, end, "{source}");
        assert_eq!(parser.python_conditions.len(), recorded, "{source}");
        assert_eq!(parser.tokens, tokens, "{source}");
    }

    let mut parser = Parser::new(tokenize("a if b else c").unwrap());
    parser.advance();
    parser.context = ExpressionContext::Interpolation;
    assert_eq!(parser.python_condition(), None);
    assert_eq!(parser.position, 1);
    assert!(parser.python_conditions.is_empty());
}

#[test]
fn suppose_premises_consume_nothing_unless_a_colon_follows_and_reuse_outcomes() {
    for (source, premise) in [
        ("suppose reading = 95: alarm", Some("reading=95")),
        ("suppose not ready: y", Some("!ready")),
        ("suppose x = c ? 1 : 2: x", Some("x=(c?1:2)")),
        (
            "suppose ok = all v in xs: v > 0: ok",
            Some("ok=(all v in xs:v>0)"),
        ),
        ("suppose Item(5): Item.length", Some("Item(5)")),
        ("suppose $Order.fee = 0: fees", Some("$Order.fee=0")),
        ("suppose delete x: y", Some("delete x")),
        ("suppose reading = 95\n    reading = 50", None),
        ("suppose reading = 95 { reading = 50 }", None),
        ("suppose f = lambda: 1", None),
        ("suppose x.lambda: 1", None),
        ("suppose a b: c", None),
        ("suppose (reading: 95", None),
    ] {
        let mut parser = Parser::new(tokenize(source).unwrap());
        parser.advance();
        let start = parser.position;
        let tokens = parser.tokens.clone();
        let first = parser.suppose_premise();
        assert_eq!(first.as_ref().map(generate).as_deref(), premise, "{source}");
        if first.is_some() {
            assert_eq!(parser.peek(), &Token::Colon, "{source}");
        } else {
            assert_eq!(parser.position, start, "{source}");
        }
        assert_eq!(parser.depth, 0, "{source}");
        assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
        let recorded = parser.suppose_premises.len();
        assert!(
            parser.suppose_premises.contains_key(&(start, 0)),
            "{source}"
        );
        let end = parser.position;
        parser.position = start;
        assert_eq!(parser.suppose_premise(), first, "{source}");
        assert_eq!(parser.position, end, "{source}");
        assert_eq!(parser.suppose_premises.len(), recorded, "{source}");
        assert_eq!(parser.tokens, tokens, "{source}");
    }

    for source in [
        "suppose x: int",
        "suppose assert: x, 1",
        "suppose",
        "suppose\nx",
        "suppose; x",
    ] {
        let mut parser = Parser::new(tokenize(source).unwrap());
        parser.advance();
        assert_eq!(parser.suppose_premise(), None, "{source}");
        assert_eq!(parser.position, 1, "{source}");
        assert!(parser.suppose_premises.is_empty(), "{source}");
    }
}

#[test]
fn lambdas_and_thin_arrows_lower_to_the_existing_anonymous_function_ast() {
    for (alias, canonical) in [
        ("lambda: 1", "() => (1)"),
        ("lambda value: value * 2", "(value) => (value * 2)"),
        (
            "lambda left, right: left + right",
            "(left, right) => (left + right)",
        ),
        ("lambda: {x: 1}", "() => ({x: 1})"),
        ("lambda: lambda: 1", "() => (() => (1))"),
        ("lambda x: x = 2", "(x) => (x = 2)"),
        ("value -> value * 2", "value => value * 2"),
        ("(value: int) -> value", "(value: int) => value"),
        ("(value = 1) -> value", "(value = 1) => value"),
        ("value -> { return value }", "value => { return value }"),
        ("() -> 1", "() => 1"),
    ] {
        let alias_ast = parse_expression(alias).unwrap();
        assert_eq!(alias_ast, parse_expression(canonical).unwrap(), "{alias}");
        assert!(matches!(alias_ast, Expr::Function(_)), "{alias}");
    }
    let mut parser = Parser::new(tokenize("{ a: lambda: 1, b: 2 }").unwrap());
    parser.skip_separators();
    parser.expression().unwrap();
    assert_eq!(parser.context, ExpressionContext::Ordinary);
    assert_eq!(parser.depth, 0);
}

#[test]
fn comma_free_block_layout_uses_exactly_the_existing_suite_guard() {
    let mut parser = Parser::new(tokenize("left right:\n    pass").unwrap());
    assert_eq!(parser.comma_free_block_end(), Some(2));
    // The existing suite parser also accepts an immediate genuine Indent.
    parser.tokens.remove(3);
    assert_eq!(parser.peek_at(3), &Token::Indent);
    assert_eq!(parser.comma_free_block_end(), Some(2));
    parser.tokens.insert(
        3,
        Spanned {
            token: Token::Semicolon,
            position: Position::new(1, 12),
        },
    );
    assert_eq!(parser.comma_free_block_end(), None);
    assert_eq!(parser.position, 0);
}

#[test]
fn comma_free_blocks_leave_the_final_colon_and_complete_named_block_unchanged() {
    for count in [2, 3, 129, 1024] {
        let names = (0..count).map(|i| format!("p{i}")).collect::<Vec<_>>();
        let candidate = format!("def f {}:\n    int => p0\n    return p0", names.join(" "));
        let accepted = format!("def f {}:\n    int => p0\n    return p0", names.join(", "));
        let actual = parse(&candidate).unwrap();
        let expected = parse(&accepted).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(generate_all(&actual), generate_all(&expected));
        let [Stmt::Function(function)] = actual.as_slice() else {
            panic!("one complete named block required");
        };
        assert_eq!(function.name.as_deref(), Some("f"));
        assert_eq!(function.parameters.len(), count);
        for (parameter, name) in function.parameters.iter().zip(&names) {
            assert_eq!(&parameter.name, name);
            assert_eq!(parameter.type_name, None);
        }
        assert!(matches!(&function.body, FunctionBody::Block(body) if body.len() == 2));
    }
}

#[test]
fn compound_assignments_lower_to_identical_assignment_and_binary_asts() {
    for operator in ["+", "-", "*", "**", "/", "%"] {
        for target in [
            "amount",
            "meter.reading",
            "parent.meter.reading",
            "$Scale.factor",
            "this.reading",
            "self.reading",
            "(amount)",
            "(this).reading",
            "($Scale).factor",
        ] {
            for value in [
                "2",
                "-3",
                "+3",
                "1 + 2 * 3",
                "1 - 2 - 3",
                "step.value",
                "first = second",
                "first or second",
            ] {
                let compact = format!("{target} {operator}= {value}");
                let expanded = format!("{target} = {target} {operator} ({value})");
                assert_eq!(
                    parse_expression(&compact).unwrap(),
                    parse_expression(&expanded).unwrap(),
                    "{compact}"
                );
                let compact_ast = parse(&compact).unwrap();
                let expanded_ast = parse(&expanded).unwrap();
                assert_eq!(compact_ast, expanded_ast, "{compact}");
                assert_eq!(
                    generate_all(&compact_ast),
                    generate_all(&expanded_ast),
                    "{compact}"
                );
            }
        }
    }
}

#[test]
fn compound_assignments_keep_assignment_precedence_and_right_associativity() {
    for (compact, expanded) in [
        (
            "first += second *= third -= 1",
            "first = first + (second = second * (third = third - 1))",
        ),
        ("outer = inner += 1", "outer = (inner = inner + 1)"),
        ("first += second > 2", "first = first + (second > 2)"),
        (
            "def combined(a, b) => a += b *= 2",
            "def combined(a, b) => a = a + (b = b * 2)",
        ),
        (
            "class Meter:\n    def adjust(delta) => self.reading += delta",
            "class Meter:\n    def adjust(delta) => self.reading = self.reading + delta",
        ),
        (
            "callback = number => number %= 3",
            "callback = number => number = number % 3",
        ),
        (
            "if active:\n    amount += step\nelse:\n    amount -= step",
            "if active:\n    amount = amount + step\nelse:\n    amount = amount - step",
        ),
        (
            "for item of items:\n    item.count += 1",
            "for item of items:\n    item.count = item.count + 1",
        ),
        (
            "assert((amount += 2), 4)",
            "assert((amount = amount + 2), 4)",
        ),
        (
            "text = `value: ${amount += 2}`",
            "text = `value: ${amount = amount + 2}`",
        ),
        (
            "first **= second **= 2",
            "first = first ** (second = second ** 2)",
        ),
        (
            "def raise(current, power) => current **= power",
            "def raise(current, power) => current = current ** power",
        ),
        (
            "for item of items:\n    item.count **= 2",
            "for item of items:\n    item.count = item.count ** 2",
        ),
        (
            "text = `value: ${amount **= 2}`",
            "text = `value: ${amount = amount ** 2}`",
        ),
    ] {
        let compact_ast = parse(compact).unwrap();
        let expanded_ast = parse(expanded).unwrap();
        assert_eq!(compact_ast, expanded_ast, "{compact}");
        assert_eq!(
            generate_all(&compact_ast),
            generate_all(&expanded_ast),
            "{compact}"
        );
    }
    assert_eq!(
        generate_all(&parse("amount += step").unwrap()),
        "amount=amount+step"
    );
    assert_eq!(
        generate_all(&parse("amount **= step").unwrap()),
        "amount=amount**step"
    );
    assert_eq!(
        generate_all(&parse("amount **= step + 1").unwrap()),
        "amount=amount**(step+1)"
    );
}

#[test]
fn logical_compounds_lower_to_existing_logical_and_assignment_nodes() {
    for (symbol, operator) in [("&&", LogicalOp::And), ("||", LogicalOp::Or)] {
        assert_eq!(
            parse_expression(&format!("target {symbol}= rhs")).unwrap(),
            Expr::Assign {
                target: Box::new(Expr::Identifier("target".into())),
                value: Box::new(Expr::Logical {
                    operator,
                    left: Box::new(Expr::Identifier("target".into())),
                    right: Box::new(Expr::Identifier("rhs".into())),
                }),
            }
        );
        for target in [
            "enabled",
            "gate.enabled",
            "parent.gate.enabled",
            "$Gate.enabled",
            "this.enabled",
            "self.enabled",
            "(enabled)",
            "(this).enabled",
            "($Gate).enabled",
            "(parent.gate).enabled",
        ] {
            for rhs in [
                "false",
                "7",
                "-3",
                "+3",
                "!other",
                "first || second && third",
                "first or second and third",
                "first + second * 3 > 0",
                "source.value",
                "first = second",
                "first += 2",
                "model |> why",
                "/=/",
            ] {
                let compact = format!("{target} {symbol}= {rhs}");
                let expanded = format!("{target} = {target} {symbol} ({rhs})");
                assert_eq!(
                    parse_expression(&compact).unwrap(),
                    parse_expression(&expanded).unwrap(),
                    "{compact}"
                );
                let compact_ast = parse(&compact).unwrap();
                let expanded_ast = parse(&expanded).unwrap();
                assert_eq!(compact_ast, expanded_ast, "{compact}");
                assert_eq!(
                    generate_all(&compact_ast),
                    generate_all(&expanded_ast),
                    "{compact}"
                );
            }
        }
    }
}

#[test]
fn logical_compounds_preserve_grouping_right_associativity_and_contexts() {
    for (compact, expanded) in [
        ("a &&= b || c", "a = a && (b || c)"),
        ("a ||= b && c", "a = a || (b && c)"),
        ("a &&= b ||= c += 2", "a = a && (b = b || (c = c + 2))"),
        (
            "a ||= b *= c &&= false",
            "a = a || (b = b * (c = c && false))",
        ),
        (
            "outer = inner ||= fallback",
            "outer = (inner = inner || fallback)",
        ),
        (
            "def combined(a, b) => a ||= b &&= 9",
            "def combined(a, b) => a = a || (b = b && 9)",
        ),
        (
            "class Gate:\n    def allow(fallback) => self.enabled ||= fallback",
            "class Gate:\n    def allow(fallback) => self.enabled = self.enabled || fallback",
        ),
        (
            "callback = value => value &&= allowed",
            "callback = value => value = value && allowed",
        ),
        (
            "if active:\n    ready ||= fallback\nelse:\n    ready &&= allowed",
            "if active:\n    ready = ready || fallback\nelse:\n    ready = ready && allowed",
        ),
        (
            "for gate of gates:\n    gate.enabled ||= fallback",
            "for gate of gates:\n    gate.enabled = gate.enabled || fallback",
        ),
        (
            "assert((enabled ||= fallback), true)",
            "assert((enabled = enabled || fallback), true)",
        ),
        (
            "text = `ready: ${enabled &&= allowed}`",
            "text = `ready: ${enabled = enabled && allowed}`",
        ),
        (
            "enabled &&= first or second",
            "enabled = enabled and (first or second)",
        ),
        (
            "enabled ||= first and second",
            "enabled = enabled or (first and second)",
        ),
    ] {
        let compact_ast = parse(compact).unwrap();
        let expanded_ast = parse(expanded).unwrap();
        assert_eq!(compact_ast, expanded_ast, "{compact}");
        assert_eq!(
            generate_all(&compact_ast),
            generate_all(&expanded_ast),
            "{compact}"
        );
    }
    assert_eq!(
        generate_all(&parse("enabled ||= fallback").unwrap()),
        "enabled=enabled||fallback"
    );
    assert_eq!(
        generate_all(&parse("ready &&= allowed").unwrap()),
        "ready=ready&&allowed"
    );
    for legacy in [
        "get().enabled = true",
        "(first = second).enabled = true",
        "items[0].enabled = false",
    ] {
        assert!(parse(legacy).is_ok(), "{legacy}");
    }
}

#[test]
fn compound_assignments_reject_non_path_targets_before_cloning() {
    for target in [
        "1",
        "true",
        "null",
        "$Scale",
        "this",
        "self",
        "first + second",
        "first and second",
        "items[0]",
        "items[0].count",
        "items[:]",
        "get().count",
        "(first = second)",
        "(first += 1)",
        "(first += 1).count",
        "(first ||= 1)",
        "(first &&= 1).count",
        "(first => first)",
        "model |> why",
        "first ** second",
        "(first **= 1)",
    ] {
        for operator in ["+=", "-=", "*=", "**=", "/=", "%=", "&&=", "||="] {
            let source = format!("{target} {operator} 2");
            let error = parse(&source).unwrap_err();
            assert_eq!(error.kind(), crate::ErrorKind::Syntax, "{source}");
            assert_eq!(
                error.message(),
                "Invalid compound assignment target",
                "{source}"
            );
            assert!(error.position().is_some(), "{source}");
        }
    }
}

#[test]
fn compound_target_limits_include_parenthesized_member_chains() {
    for operator in ["+=", "-=", "*=", "**=", "/=", "%=", "&&=", "||="] {
        let allowed = format!(
            "(root{}){} {operator} 1",
            ".next".repeat(64),
            ".next".repeat(64)
        );
        assert!(parse(&allowed).is_ok(), "{operator}");

        let excessive = format!(
            "(root{}){} {operator} 1",
            ".next".repeat(64),
            ".next".repeat(65)
        );
        let error = parse(&excessive).unwrap_err();
        assert_eq!(
            error.message(),
            "Expression has too many parts",
            "{operator}"
        );
    }
}

#[test]
fn exponentiation_binds_tighter_than_products_and_groups_to_the_right() {
    for (compact, grouped) in [
        ("a ** b ** c", "a ** (b ** c)"),
        ("a * b ** c", "a * (b ** c)"),
        ("a ** b * c", "(a ** b) * c"),
        ("a / b ** c % d", "a / (b ** c) % d"),
        ("a + b ** c - d", "a + (b ** c) - d"),
        ("a ** b > c", "(a ** b) > c"),
        ("a ** b ?? c", "(a ** b) ?? c"),
        ("a ** b && c", "(a ** b) && c"),
        ("a ? b ** c : d ** e", "a ? (b ** c) : (d ** e)"),
        ("-a ** b", "-(a ** b)"),
        ("+a ** b", "+(a ** b)"),
        ("!a ** b", "!(a ** b)"),
        ("not a ** b", "not (a ** b)"),
        ("typeof a ** b", "typeof (a ** b)"),
        ("delete a ** b", "delete (a ** b)"),
        ("why a ** b", "(a ** b) |> why"),
        ("new a ** b", "a ** b"),
        ("a ** -b", "a ** (-b)"),
        ("a ** -b ** c", "a ** (-(b ** c))"),
        ("a ** !b", "a ** (!b)"),
        ("a ** typeof b", "a ** (typeof b)"),
        ("a.b ** c[0]", "(a.b) ** (c[0])"),
        ("f(a) ** g(b)", "(f(a)) ** (g(b))"),
        ("a@first ** 2", "(a@first) ** 2"),
        ("x => x ** 2", "x => (x ** 2)"),
        ("lambda x: x ** 2", "(x) => (x ** 2)"),
    ] {
        assert_eq!(
            parse_expression(compact).unwrap(),
            parse_expression(grouped).unwrap(),
            "{compact}"
        );
    }
    assert_eq!(
        parse_expression("a ** b").unwrap(),
        Expr::Binary {
            operator: BinaryOp::Power,
            left: Box::new(Expr::Identifier("a".into())),
            right: Box::new(Expr::Identifier("b".into())),
        }
    );
}

#[test]
fn power_rendering_reads_back_as_the_same_expression() {
    for (source, rendered) in [
        ("x ** 2", "x**2"),
        ("x ** 2 ** 3", "x**2**3"),
        ("(x ** 2) ** 3", "(x**2)**3"),
        ("-x ** 2", "-x**2"),
        ("(-x) ** 3", "(-x)**3"),
        ("(!x) ** 2", "(!x)**2"),
        ("(typeof x) ** 2", "(typeof x)**2"),
        ("x ** -y", "x**-y"),
        ("x ** (1 + 1)", "x**(1+1)"),
        ("x ** (y > 1)", "x**(y>1)"),
        ("(2 * x) ** 2", "(2*x)**2"),
        ("2 * x ** 2", "2*x**2"),
        ("x ** 2 * 2", "x**2*2"),
        ("(a || b) ** 2", "(a||b)**2"),
        ("x ** (a && b)", "x**(a&&b)"),
        ("(a ?? b) ** 2", "(a??b)**2"),
        ("x ** (a ?? b)", "x**(a??b)"),
        ("(c ? a : b) ** 2", "(c?a:b)**2"),
        ("x ** (c ? a : b)", "x**(c?a:b)"),
        ("(why x) ** 2", "(x|>why)**2"),
        ("2 ** (x |> why)", "2**(x|>why)"),
        ("(x = 2) ** 2", "(x=2)**2"),
        ("x ** (y = 2)", "x**(y=2)"),
        ("(delete x.y) ** 2", "(delete x.y)**2"),
        ("x ** delete y", "x**delete y"),
        ("((a = 1) => a) ** 2", "((a=1)=>a)**2"),
        ("2 ** ((a = 1) => a)", "2**((a=1)=>a)"),
        ("x.y ** z[0]", "x.y**z[0]"),
        ("f(x) ** g(y)", "f(x)**g(y)"),
        ("(all v in xs: v) ** 2", "(all v in xs:v)**2"),
    ] {
        let expression = parse_expression(source).unwrap();
        assert_eq!(expression.to_string(), rendered, "{source}");
        assert_eq!(parse_expression(rendered).unwrap(), expression, "{source}");
    }
    let literal = Expr::Binary {
        operator: BinaryOp::Power,
        left: Box::new(Expr::Number(-3.0)),
        right: Box::new(Expr::Number(-2.0)),
    };
    assert_eq!(literal.to_string(), "(-3)**-2");
    assert_eq!(
        generate_all(&parse("area = side ** 2 * depth").unwrap()),
        "area=side**2*depth"
    );
}

#[test]
fn power_chains_share_the_expression_nesting_budget() {
    let assigned = |count: usize| format!("x = 1{}", " ** 1".repeat(count));
    assert!(parse(&assigned(61)).is_ok());
    let error = parse(&assigned(62)).unwrap_err();
    assert_eq!(error.message(), "Expressions are nested too deeply");
    assert_eq!(error.position(), Some(Position::new(1, 315)));
    let bare = |count: usize| format!("1{}", " ** 1".repeat(count));
    assert!(parse(&bare(62)).is_ok());
    let error = parse(&bare(63)).unwrap_err();
    assert_eq!(error.message(), "Expressions are nested too deeply");
    assert_eq!(error.position(), Some(Position::new(1, 316)));
}

#[test]
fn named_expression_bodies_lower_to_single_return_blocks() {
    for expression in [
        "number + 1 * factor",
        "number = factor",
        "{value: number, scaled: number * factor}",
        "value => value * 2",
        "(value: int) => { return value * 2 }",
        "why number",
    ] {
        let compact = parse(&format!(
            "def calculate(number: int, factor) => {expression}"
        ))
        .unwrap();
        let expanded = parse(&format!(
            "def calculate(number: int, factor):\n    return {expression}"
        ))
        .unwrap();

        assert_eq!(compact, expanded, "{expression}");
        assert_eq!(generate_all(&compact), generate_all(&expanded));

        let [Stmt::Function(function)] = compact.as_slice() else {
            panic!("expected one named function");
        };
        let FunctionBody::Block(body) = &function.body else {
            panic!("named expression bodies must use block execution");
        };

        assert!(matches!(body.as_slice(), [Stmt::Return(Some(_))]));
        assert_eq!(
            generate_all(body),
            format!("return {}", parse_expression(expression).unwrap())
        );
    }
}

#[test]
fn named_expression_methods_share_block_lowering() {
    let compact = parse(
        r"
class Gauge:
    def init(amount: int) => this.amount = amount
    def reading() => self.amount
",
    )
    .unwrap();
    let expanded = parse(
        r"
class Gauge:
    def init(amount: int):
        return this.amount = amount
    def reading():
        return self.amount
",
    )
    .unwrap();

    assert_eq!(compact, expanded);
    assert_eq!(generate_all(&compact), generate_all(&expanded));

    let [Stmt::Class(class)] = compact.as_slice() else {
        panic!("expected one class");
    };
    assert_eq!(class.methods.len(), 2);
    assert!(class.methods.iter().all(|method| matches!(
        &method.body,
        FunctionBody::Block(body) if matches!(body.as_slice(), [Stmt::Return(Some(_))])
    )));
}

#[test]
fn named_expression_bodies_allow_parenthesized_continuations() {
    let compact = parse("def double(number: int) => (\n    number *\n    2\n)\nother = 3").unwrap();
    let expanded = parse("def double(number: int):\n    return number * 2\nother = 3").unwrap();

    assert_eq!(compact, expanded);
    assert_eq!(generate_all(&compact), generate_all(&expanded));
}

#[test]
fn while_conditions_qualify_only_colon_headers_and_reuse_outcomes() {
    for (source, condition) in [
        ("while n < 3:\n    n = n + 1", Some("n<3")),
        ("while n < 3: n = n + 1", Some("n<3")),
        ("while (ready): pass", Some("ready")),
        ("while -n: pass", Some("-n")),
        ("while [n]: pass", Some("[n]")),
        ("while {}: pass", Some("{}")),
        ("while x if y else z: pass", Some("(y?x:z)")),
        ("while all v in xs: v > 0: pass", Some("(all v in xs:v>0)")),
        ("while ready ? on : off: pass", Some("(ready?on:off)")),
        ("while x => lambda + 1: pass", Some("function()")),
        ("while [lambda: 1]: pass", Some("[function()]")),
        ("while while: pass", Some("while")),
        ("while ready: tick()", Some("ready")),
        ("while ready: 1", Some("ready")),
        ("while n < 3 {\n    n = n + 1\n}", None),
        ("while n < 3\n    n = n + 1", None),
        ("while lambda: 1", None),
        ("while ready + lambda: 1", None),
        ("while not lambda: 1", None),
        ("while ready ? on : lambda: 1", None),
        ("while x => lambda: 1", None),
        ("while gauge.lambda: tick()", None),
        ("while lambda x: x", None),
        ("while x if y: z", None),
        ("while all v in xs: v", None),
        ("while: int", None),
        ("while(4)", None),
        ("while = 1", None),
        ("while", None),
        ("while\n    ready: int", None),
        ("while ready\n: tick()", None),
        ("while @first: tick()", None),
    ] {
        let mut parser = Parser::new(tokenize(source).unwrap());
        let start = parser.position;
        let tokens = parser.tokens.clone();
        let first = parser.while_condition();
        assert_eq!(
            first.as_ref().map(ToString::to_string).as_deref(),
            condition,
            "{source}"
        );
        if first.is_some() {
            assert_eq!(parser.peek(), &Token::Colon, "{source}");
        } else {
            assert_eq!(parser.position, start, "{source}");
        }
        assert_eq!(parser.depth, 0, "{source}");
        assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
        let recorded = parser.while_conditions.len();
        assert!(
            parser.while_conditions.contains_key(&(start, 0)),
            "{source}"
        );
        let end = parser.position;
        parser.position = start;
        assert_eq!(parser.while_condition(), first, "{source}");
        assert_eq!(parser.position, end, "{source}");
        assert_eq!(parser.while_conditions.len(), recorded, "{source}");
        assert_eq!(parser.tokens, tokens, "{source}");
    }

    for source in [
        "while ready: tick",
        "while ready: tick;",
        "while assert: ready, true",
        "value = 1",
    ] {
        let mut parser = Parser::new(tokenize(source).unwrap());
        assert_eq!(parser.while_condition(), None, "{source}");
        assert_eq!(parser.position, 0, "{source}");
        assert_eq!(parser.depth, 0, "{source}");
        assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
        assert!(parser.while_conditions.is_empty(), "{source}");
    }
}

#[test]
fn nested_unqualified_while_headers_are_attempted_once_per_position() {
    let levels = 16;
    let mut source = String::new();
    for level in 0..levels {
        source.push_str(&"    ".repeat(level));
        source.push_str("while (function() {\n");
    }
    source.push_str(&"    ".repeat(levels));
    source.push_str("ready\n");
    for level in (0..levels).rev() {
        source.push_str(&"    ".repeat(level));
        source.push_str("})\n");
    }

    let mut parser = Parser::new(tokenize(&source).unwrap());
    let program = parser.program().unwrap();

    assert_eq!(program.statements.len(), 1);
    assert!(matches!(
        &program.statements[0],
        Stmt::Expression(Expr::Call { callee, .. })
            if matches!(callee.as_ref(), Expr::Identifier(name) if name == "while")
    ));
    assert!(parser.while_conditions.len() <= levels * 2);
}

#[test]
fn while_conditions_leave_suppose_heads_and_value_assignments_to_the_accepted_parser() {
    for (source, condition) in [
        ("while suppose: x", None),
        ("while suppose:\n    x = 1", None),
        ("while x = suppose: x", None),
        ("while x = (suppose): x", None),
        ("while point.x = suppose: x", None),
        ("while (a) = suppose: a", None),
        ("while [a] = suppose: a", None),
        ("while (suppose): pass", None),
        ("while (x = suppose): pass", None),
        ("while suppose x > 1: x", None),
        ("while x = suppose y: y", None),
        ("while := 1", None),
        ("while x := 1", None),
        ("while x:= 1", None),
        ("while suppose > 1: pass", Some("suppose>1")),
        ("while x == suppose: pass", Some("x==suppose")),
        ("while x = suppose + 1: pass", Some("x=suppose+1")),
        ("while x = y = suppose: pass", Some("x=y=suppose")),
        ("while not suppose: pass", Some("!suppose")),
        ("while [suppose]: pass", Some("[suppose]")),
        ("while x: y := 1", Some("x")),
        ("while x > 0: x := x - 1", Some("x>0")),
    ] {
        let mut parser = Parser::new(tokenize(source).unwrap());
        let first = parser.while_condition();
        assert_eq!(
            first.as_ref().map(ToString::to_string).as_deref(),
            condition,
            "{source}"
        );
        if first.is_some() {
            assert_eq!(parser.peek(), &Token::Colon, "{source}");
        } else {
            assert_eq!(parser.position, 0, "{source}");
        }
        assert_eq!(parser.depth, 0, "{source}");
        assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
    }
}

#[test]
fn word_headers_consult_the_accepted_grammar_without_changing_the_parser() {
    for (source, condition) in [
        ("while v is x: T", Some("(v is x)")),
        ("while f() is x: T", None),
        ("while v is assert: a, b", Some("(v is assert)")),
        ("while f() is assert: a, b", None),
        ("while x is Y: pass", Some("(x is Y)")),
        ("while x is not Y: pass", Some("(x is not Y)")),
        ("while ready: f(x is Y)", Some("ready")),
    ] {
        let mut parser = Parser::new(tokenize(source).unwrap());
        parser.words = true;
        let tokens = parser.tokens.clone();
        let first = parser.while_condition();
        assert_eq!(
            first.as_ref().map(ToString::to_string).as_deref(),
            condition,
            "{source}"
        );
        if first.is_some() {
            assert_eq!(parser.peek(), &Token::Colon, "{source}");
        } else {
            assert_eq!(parser.position, 0, "{source}");
        }
        assert_eq!(parser.depth, 0, "{source}");
        assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
        assert!(parser.words && !parser.enclosed, "{source}");
        assert_eq!(parser.tokens, tokens, "{source}");
        assert_eq!(parser.while_conditions.len(), 1, "{source}");
        assert!(parser.suppose_premises.is_empty(), "{source}");
        assert!(parser.python_conditions.is_empty(), "{source}");
    }

    for (source, premise) in [
        ("suppose v is x: T", Some("(v is x)")),
        ("suppose f() is x: T", None),
        ("suppose v is Y: pass", Some("(v is Y)")),
    ] {
        let mut parser = Parser::new(tokenize(source).unwrap());
        parser.words = true;
        parser.advance();
        let tokens = parser.tokens.clone();
        let first = parser.suppose_premise();
        assert_eq!(first.as_ref().map(generate).as_deref(), premise, "{source}");
        if first.is_some() {
            assert_eq!(parser.peek(), &Token::Colon, "{source}");
        } else {
            assert_eq!(parser.position, 1, "{source}");
        }
        assert_eq!(parser.depth, 0, "{source}");
        assert_eq!(parser.context, ExpressionContext::Ordinary, "{source}");
        assert!(parser.words && !parser.enclosed, "{source}");
        assert_eq!(parser.tokens, tokens, "{source}");
        assert_eq!(parser.suppose_premises.len(), 1, "{source}");
        assert!(parser.while_conditions.is_empty(), "{source}");
    }
}
