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
fn named_expression_body_errors_use_the_existing_expression_grammar() {
    for (source, message) in [
        ("def missing() =>", "Unexpected end of input"),
        ("def missing() =>\n", "Unexpected end of line"),
        (
            "def invalid() => { return 1 }",
            "Expected a key but found keyword 'return'",
        ),
        (
            "def invalid() => { value = 1 }",
            "Expected ':' but found '='",
        ),
        (
            "def invalid(value:) => value",
            "Expected a type but found ')'",
        ),
        ("def invalid() => (1 +)", "Unexpected ')'"),
    ] {
        let error = failure(source);
        assert_eq!(error.kind(), nucleoid::ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert!(error.position().is_some(), "{source}");
    }
}

#[test]
fn compound_assignment_diagnostics_name_exact_tokens_and_targets() {
    for operator in ["+=", "-=", "*=", "/=", "%=", "&&=", "||="] {
        for (source, message) in [
            (
                format!("amount {operator}"),
                "Unexpected end of input".to_string(),
            ),
            (
                format!("amount {operator}\n"),
                "Unexpected end of line".to_string(),
            ),
            (
                format!("amount {operator} {operator} 1"),
                format!("Unexpected '{operator}'"),
            ),
            (
                format!("amount {operator} (1 +)"),
                "Unexpected ')'".to_string(),
            ),
        ] {
            let error = failure(&source);
            assert_eq!(error.kind(), nucleoid::ErrorKind::Syntax, "{source}");
            assert_eq!(error.message(), message, "{source}");
            assert!(error.position().is_some(), "{source}");
        }

        let source = format!("saved = 2\n1 {operator} 3");
        let error = failure(&source);
        assert_eq!(error.kind(), nucleoid::ErrorKind::Syntax);
        assert_eq!(error.message(), "Invalid compound assignment target");
        assert_eq!(
            error
                .position()
                .map(|position| (position.line, position.column)),
            Some((2, 3))
        );
    }
}

#[test]
fn logical_compound_runtime_errors_follow_the_expanded_assignment() {
    for (operator, logical, evaluated_left) in [("&&=", "&&", "true"), ("||=", "||", "false")] {
        for (setup, target, rhs, kind, message) in [
            (
                String::new(),
                "missing",
                "true",
                nucleoid::ErrorKind::Reference,
                "missing is not defined",
            ),
            (
                format!("enabled = {evaluated_left}\n"),
                "enabled",
                "missing",
                nucleoid::ErrorKind::Reference,
                "missing is not defined",
            ),
            (
                String::new(),
                "this.enabled",
                "true",
                nucleoid::ErrorKind::Reference,
                "this is not defined",
            ),
            (
                "number = 1\n".to_string(),
                "number.reading",
                "true",
                nucleoid::ErrorKind::Type,
                "Cannot assign 'reading' on number",
            ),
            (
                "class Item\nitem = Item()\n".to_string(),
                "item.value",
                "true",
                nucleoid::ErrorKind::Type,
                "Cannot use 'value' as a property",
            ),
            (
                format!("class Gate\ngate = Gate()\n$Gate.enabled = {evaluated_left}\n"),
                "$Gate.enabled",
                "true",
                nucleoid::ErrorKind::Type,
                "Circular Dependency",
            ),
        ] {
            let compact = format!("{setup}{target} {operator} {rhs}");
            let expanded = format!("{setup}{target} = {target} {logical} ({rhs})");
            let compact_error = failure(&compact);
            assert_eq!(compact_error, failure(&expanded), "{compact}");
            assert_eq!(compact_error.kind(), kind, "{compact}");
            assert_eq!(compact_error.message(), message, "{compact}");
        }
    }
}

#[test]
fn separated_logical_assignments_and_word_compounds_are_not_added() {
    for source in [
        "enabled && = true",
        "enabled || = true",
        "enabled and= true",
        "enabled or= true",
    ] {
        let error = failure(source);
        assert_eq!(error.kind(), nucleoid::ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), "Unexpected '='", "{source}");
        assert!(error.position().is_some(), "{source}");
    }
}

#[test]
fn compound_runtime_errors_follow_the_expanded_assignment() {
    for (setup, compact, expanded, kind, message) in [
        (
            "",
            "missing += 1",
            "missing = missing + 1",
            nucleoid::ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "amount = 10\n",
            "amount += missing",
            "amount = amount + missing",
            nucleoid::ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "",
            "this.reading += 1",
            "this.reading = this.reading + 1",
            nucleoid::ErrorKind::Reference,
            "this is not defined",
        ),
        (
            "number = 1\n",
            "number.reading += 1",
            "number.reading = number.reading + 1",
            nucleoid::ErrorKind::Type,
            "Cannot assign 'reading' on number",
        ),
        (
            "class Item:\n    pass\nitem = Item()\n",
            "item.value += 1",
            "item.value = item.value + 1",
            nucleoid::ErrorKind::Type,
            "Cannot use 'value' as a property",
        ),
        (
            "class Scale:\n    pass\nscale = Scale()\n$Scale.factor = 2\n",
            "$Scale.factor *= 3",
            "$Scale.factor = $Scale.factor * 3",
            nucleoid::ErrorKind::Type,
            "Circular Dependency",
        ),
    ] {
        let compact_error = failure(&format!("{setup}{compact}"));
        let expanded_error = failure(&format!("{setup}{expanded}"));
        assert_eq!(compact_error, expanded_error, "{compact}");
        assert_eq!(compact_error.kind(), kind, "{compact}");
        assert_eq!(compact_error.message(), message, "{compact}");
    }
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
