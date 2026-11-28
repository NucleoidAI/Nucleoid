use super::*;
use crate::lang::ast::lexer::tokenize;

fn mentions(source: &str) -> bool {
    Parser::new(tokenize(source).unwrap()).mentions_ranges()
}

fn reading(source: &str) -> Parser {
    Parser {
        ranges: Ranges::Read,
        ..Parser::new(tokenize(source).unwrap())
    }
}

#[test]
fn only_adjacent_operators_and_template_text_start_the_range_run() {
    for source in ["x = 0..<3", "x = a ..= b", "`${0..<3}`", "`a..=b`"] {
        assert!(mentions(source), "{source}");
    }
    for source in [
        "x = 0. .<3",
        "x = 0.. <3",
        "x = 0..<=3",
        "x = 0..==3",
        "x = 0...3",
        "x = 1..2",
        "x = \"0..<3\"",
        "`${x}.<3`",
    ] {
        assert!(!mentions(source), "{source}");
    }
}

#[test]
fn the_operator_is_read_only_in_the_range_run_outside_reserved_positions() {
    assert_eq!(reading("..<").range_operator(), Some(BinaryOp::Range));
    assert_eq!(
        reading("..=").range_operator(),
        Some(BinaryOp::RangeInclusive)
    );
    assert_eq!(Parser::new(tokenize("..<").unwrap()).range_operator(), None);
    let mut reserved = reading("..<");
    assert_eq!(reserved.reserved(|parser| parser.range_operator()), None);
    assert_eq!(
        reserved.reserved(|parser| parser.unreserved(|open| open.range_operator())),
        Some(BinaryOp::Range)
    );
    assert_eq!(reading(". .<").range_operator(), None);
}

#[test]
fn a_repeat_count_reserves_only_its_top_level_ranges() {
    let at = |source: &str| {
        let mut parser = reading(source);
        parser.position = parser.tokens.len();
        parser
            .reserve_count(0)
            .err()
            .and_then(|error| error.position())
            .map(|position| position.column)
    };
    assert_eq!(at("repeat 0..<3"), Some(11));
    assert_eq!(at("repeat (0..<3)"), None);
    assert_eq!(at("repeat f(0..<3) + [1..=2][0]"), None);
    assert_eq!(at("repeat (a) + b..=3"), Some(17));
}
