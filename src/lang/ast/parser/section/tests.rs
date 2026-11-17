use super::{Sections, section_operator};
use crate::lang::ast::lexer::{Token, tokenize};
use crate::lang::ast::parser::Parser;

const OPERATORS: [&str; 14] = [
    "==", "!=", "===", "!==", "<", "<=", ">", ">=", "*", "%", "**", "&&", "||", "??",
];

fn mentions(source: &str) -> bool {
    Parser::new(tokenize(source).unwrap()).mentions_sections()
}

fn reading(source: &str) -> Parser {
    Parser {
        sections: Sections::Read,
        ..Parser::new(tokenize(source).unwrap())
    }
}

#[test]
fn the_section_operators_are_the_fourteen_binary_operators_no_operand_begins_with() {
    for operator in OPERATORS {
        let tokens = tokenize(operator).unwrap();
        assert_eq!(tokens.len(), 2, "{operator}");
        assert!(section_operator(&tokens[0].token), "{operator}");
    }
    for source in [
        "+", "-", "!", "=", ":=", "+=", "-=", "*=", "**=", "/=", "%=", "&&=", "||=", "?", ":",
        "|>", "=>", "->", ".", ",", "(", ")", "[", "]", "{", "}", ";", "is", "in", "not", "and",
        "or", "typeof", "x", "1", "\"s\"", "`t`",
    ] {
        let token = &tokenize(source).unwrap()[0].token;
        assert!(!section_operator(token), "{source}");
    }
    let division = tokenize("1 / 2").unwrap();
    assert_eq!(division[1].token, Token::Slash);
    assert!(!section_operator(&division[1].token));
    for token in [Token::Newline, Token::Indent, Token::Dedent, Token::Eof] {
        assert!(!section_operator(&token), "{token:?}");
    }
}

#[test]
fn the_run_starts_for_a_section_window_or_a_template() {
    for operator in OPERATORS {
        assert!(mentions(&format!("f({operator} 1)")), "{operator}");
        assert!(mentions(&format!("f(1, {operator} 1)")), "{operator}");
        assert!(mentions(&format!("f(\n    {operator} 1\n)")), "{operator}");
    }
    for source in [
        "xs.filter(>4)",
        "f(a)(> 1)",
        "(> 4)",
        "a, *rest = p",
        "class Pair(*, a)",
        "x = `plain`",
        "x = `${1}`",
        "`${f(> 1)}`",
    ] {
        assert!(mentions(source), "{source}");
    }
    for source in [
        "x = > 4",
        "[> 4]",
        "{a: > 4}",
        "f(test= > 4)",
        "x -> > 4",
        "f(- 1)",
        "f(+ 1)",
        "f(! 1)",
        "f(1 > 2)",
        "f(a, b > 2)",
        "f(is 1)",
        "f(in xs)",
        "f(= 1)",
        "f(? 1 : 2)",
        "f(|> why)",
        "f(.a == 1)",
        "f(x -> x > 1)",
        "xs[> 1]",
        "x = )",
        "",
    ] {
        assert!(!mentions(source), "{source}");
    }
}

#[test]
fn a_section_starts_only_while_sections_are_read_and_only_at_an_operator() {
    for operator in OPERATORS {
        let source = format!("{operator} 1");
        assert!(
            !Parser::new(tokenize(&source).unwrap()).starts_section(),
            "{operator}"
        );
        assert!(reading(&source).starts_section(), "{operator}");
    }
    for source in [
        "1", "x > 1", "- 1", "+ 1", "! 1", "/ 1", "is 1", "in xs", ".a", ")", "",
    ] {
        assert!(!reading(source).starts_section(), "{source}");
    }
}
