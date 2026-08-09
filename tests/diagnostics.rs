//! Errors have to say where they are, or the reader cannot act on them.

use nucleoid::{Error, Runtime};

fn failure(source: &str) -> Error {
    let mut runtime = Runtime::new();

    match runtime.run(source) {
        Ok(value) => panic!("expected an error, got {value}"),
        Err(error) => error,
    }
}

fn at(source: &str) -> (usize, usize) {
    let error = failure(source);
    let position = error
        .position()
        .unwrap_or_else(|| panic!("{error} carries no position"));

    (position.line, position.column)
}

#[test]
fn a_syntax_error_points_at_the_line_it_is_on() {
    // The `:` is missing at the end of line 4.
    assert_eq!(at("a = 1\nb = 2\n\nif a > 1\n    c = 3\n").0, 4);

    // The list is never closed, so the report is at the end of the input.
    assert_eq!(at("x = 1\ny = [1, 2\n").0, 3);
}

#[test]
fn a_syntax_error_names_what_it_found() {
    let error = failure("a = 1\nif a > 1\n    b = 2\n");

    assert_eq!(error.kind(), nucleoid::ErrorKind::Syntax);
    assert_eq!(
        error.message(),
        "Expected ':' but found end of line",
        "the message should name the token, not its debug form"
    );
}

#[test]
fn a_runtime_error_points_at_its_statement() {
    assert_eq!(at("total = 10\nrate = total * missing\n"), (2, 1));

    assert_eq!(
        at("class Item:\n    pass\n\nitem1 = Item()\nitem1.value = 3\n"),
        (5, 1)
    );

    assert_eq!(
        at("first = 1\nsecond = first + 1\nfirst = second + 1\n"),
        (3, 1)
    );
}

#[test]
fn a_position_is_rendered_with_the_message() {
    let error = failure("total = 10\nrate = total * missing\n");

    assert_eq!(
        error.to_string(),
        "ReferenceError: missing is not defined (line 2, column 1)"
    );
}

#[test]
fn the_caught_value_carries_no_position() {
    let mut runtime = Runtime::new();

    // What a program sees in `catch` is part of the language, so it stays
    // exactly as specified.
    runtime
        .run("try:\n    t = e + 1\ncatch error:\n    assert(error, ReferenceError(\"e is not defined\"))\n")
        .unwrap();

    assert!(
        runtime.take_assertions().is_empty(),
        "the caught value should not gain a position"
    );
}

#[test]
fn columns_count_characters_not_bytes() {
    // The `?` sits at character 22 of a line holding multi-byte text; counting
    // bytes would report a larger column.
    assert_eq!(at("greeting = \"héllo wörld\" ?\n"), (1, 26));
}

/// The examples in NUC 9 and `docs/README.md` say exactly this.
#[test]
fn the_documented_examples_report_what_the_reference_says() {
    let error = failure("a = 1\n\nt = e + 1\n");

    assert_eq!(
        error.to_string(),
        "ReferenceError: e is not defined (line 3, column 1)"
    );
}

#[test]
fn a_failure_inside_a_block_is_placed_at_the_statement_that_set_it_off() {
    // The reference is explicit that this is the top-level statement, not the
    // line within it.
    assert_eq!(at("a = 1\n\n{\n    b = 2\n    c = missing\n}\n"), (3, 1));
}

#[test]
fn the_documented_limits_are_reported_not_crashed() {
    let deep = format!("a = {}1{}", "(".repeat(400), ")".repeat(400));
    assert_eq!(
        failure(&deep).message(),
        "Expressions are nested too deeply"
    );

    let chain: String = (0..400).map(|index| format!(" + {index}")).collect();
    assert_eq!(
        failure(&format!("b = 0{chain}")).message(),
        "Expression has too many operands"
    );
}

#[test]
fn an_unexpected_character_is_placed() {
    let error = failure("a = 1\nb = ?\n");

    assert_eq!(error.message(), "Unexpected character '?'");
    assert_eq!(error.position().map(|p| (p.line, p.column)), Some((2, 5)));
}
