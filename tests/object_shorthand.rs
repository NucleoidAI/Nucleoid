//! Object-property shorthand lowers to existing fields, not a new runtime path.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Ast, Expr, FunctionBody, Stmt, parse, parse_expression, parse_program};
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn equivalent(compact: &str, explicit: &str) {
    let compact_ast = parse(compact).unwrap();
    let explicit_ast = parse(explicit).unwrap();
    assert_eq!(compact_ast, explicit_ast, "{compact}");
    assert_eq!(
        generate_all(&compact_ast),
        generate_all(&explicit_ast),
        "{compact}"
    );
}

/// Run both complete programs, including every assertion in their checks.
fn check_forms(
    setup: &str,
    compact: &str,
    explicit: &str,
    checks: &str,
    assertions: usize,
) -> [Runtime; 2] {
    let compact = format!("{setup}\n{compact}\n{checks}");
    let explicit = format!("{setup}\n{explicit}\n{checks}");
    equivalent(&compact, &explicit);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let mut returns = Vec::new();
    for (runtime, source) in runtimes
        .iter_mut()
        .zip([compact.as_str(), explicit.as_str()])
    {
        let value = runtime.run(source).unwrap();
        returns.push(runtime.serialize_json(&value).unwrap());
        assert_eq!(runtime.assertions_run(), assertions, "{source}");
        let failures = runtime.take_assertions();
        assert!(failures.is_empty(), "{source}\n{failures:?}");
    }
    assert_eq!(returns[0], returns[1]);
    runtimes
}

#[test]
fn identifier_fields_lower_to_existing_object_ast_and_expanded_canonical_source() {
    let expression = parse_expression("{ amount, label }").unwrap();
    assert_eq!(
        expression,
        Expr::ObjectLiteral(vec![
            ("amount".to_string(), Expr::Identifier("amount".to_string())),
            ("label".to_string(), Expr::Identifier("label".to_string())),
        ])
    );
    assert!(matches!(Ast::convert(&expression), Ast::Object(_)));
    assert_eq!(expression.to_string(), r#"{"amount":amount,"label":label}"#);
    assert_eq!(
        expression,
        parse_expression(r#"{ "amount": amount, label: label }"#).unwrap()
    );

    for name in [
        "amount", "_private", "A1", "why", "affects", "model", "type", "value",
    ] {
        let compact = format!("record = {{ {name} }}");
        equivalent(&compact, &format!("record = {{ \"{name}\": {name} }}"));
    }
}

#[test]
fn mixed_nested_and_trailing_fields_keep_their_full_ast_and_field_order() {
    for (compact, explicit) in [
        (
            r#"record = { amount, label: "fixed", "amount": other, 1: amount, nested: { label }, items: [{ amount }], }"#,
            r#"record = { amount: amount, label: "fixed", "amount": other, 1: amount, nested: { label: label }, items: [{ amount: amount }], }"#,
        ),
        ("return { amount }", "return { amount: amount }"),
        ("accept({ amount })", "accept({ amount: amount })"),
        ("items = [{ amount }]", "items = [{ amount: amount }]"),
        ("({ amount }).amount", "({ amount: amount }).amount"),
        (
            "def wrap(amount) => { amount }",
            "def wrap(amount) => { amount: amount }",
        ),
        (
            "builder = amount => ({ amount })",
            "builder = amount => ({ amount: amount })",
        ),
        (
            "def wrap(amount):\n    return { amount }",
            "def wrap(amount):\n    return { amount: amount }",
        ),
        (
            "builder = function(amount) { return { amount } }",
            "builder = function(amount) { return { amount: amount } }",
        ),
        (
            "record = { amount: (amount = 3), other }",
            "record = { amount: (amount = 3), other: other }",
        ),
    ] {
        equivalent(compact, explicit);
    }

    let expression = parse_expression("{ amount, label, amount: other }").unwrap();
    let Expr::ObjectLiteral(entries) = expression else {
        panic!("expected an object");
    };
    assert_eq!(
        entries,
        [
            ("amount".to_string(), Expr::Identifier("amount".to_string())),
            ("label".to_string(), Expr::Identifier("label".to_string())),
            ("amount".to_string(), Expr::Identifier("other".to_string())),
        ]
    );
}

#[test]
fn existing_newline_semicolon_and_comma_rules_apply_without_consuming_lookahead() {
    for (compact, explicit) in [
        ("record = { amount; }", "record = { amount: amount; }"),
        (
            "record = { amount\n, label;\n, }",
            "record = { amount: amount\n, label: label;\n, }",
        ),
        (
            "record = {;\namount;;,\n;label;;\n}",
            "record = {;\namount: amount;;,\n;label: label;;\n}",
        ),
        (
            "record = { amount # field boundary\n, label # closing boundary\n}",
            "record = { amount: amount # field boundary\n, label: label # closing boundary\n}",
        ),
        (
            "record = ({\n    amount,\n    label: label,\n})",
            "record = ({\n    amount: amount,\n    label: label,\n})",
        ),
    ] {
        equivalent(compact, explicit);
    }

    let program = parse_program("record = { amount }\nnext = 1").unwrap();
    assert_eq!(
        program.positions,
        [Position::new(1, 1), Position::new(2, 1)]
    );
}

#[test]
fn statement_blocks_and_named_and_anonymous_arrows_keep_existing_disambiguation() {
    assert_eq!(
        parse("{ amount }").unwrap(),
        [Stmt::Block(vec![Stmt::Expression(Expr::Identifier(
            "amount".to_string()
        ))])]
    );
    let Expr::Function(anonymous) = parse_expression("amount => { amount }").unwrap() else {
        panic!("expected an anonymous function");
    };
    assert_eq!(
        anonymous.body,
        FunctionBody::Block(vec![Stmt::Expression(Expr::Identifier(
            "amount".to_string()
        ))])
    );
    let Expr::Function(returning) = parse_expression("amount => ({ amount })").unwrap() else {
        panic!("expected an anonymous function");
    };
    assert_eq!(
        returning.body,
        FunctionBody::Expression(Box::new(parse_expression("{ amount }").unwrap()))
    );
    let named = parse("def wrap(amount) => { amount }").unwrap();
    let [Stmt::Function(named)] = named.as_slice() else {
        panic!("expected a named function");
    };
    assert_eq!(
        named.body,
        FunctionBody::Block(vec![Stmt::Return(Some(
            parse_expression("{ amount }").unwrap()
        ))])
    );

    check_forms(
        "amount = 4\n{ amount }\nblock = amount => { amount }\ndef ordinary(amount):\n    amount",
        "def wrap(amount) => { amount }\nreturning = amount => ({ amount })",
        "def wrap(amount) => { amount: amount }\nreturning = amount => ({ amount: amount })",
        "assert(block(2), null)\nassert(ordinary(3), null)\nassert(wrap(5).amount, 5)\nassert(returning(7).amount, 7)",
        4,
    );
}

#[test]
fn invalid_key_tokens_and_missing_colons_keep_exact_accepted_diagnostics() {
    // Captured with the unchanged accepted runtime before editing the parser.
    for (source, message, line, column) in [
        (
            "record = { amount other }",
            "Expected ':' but found identifier 'other'",
            1,
            19,
        ),
        (
            "record = { amount = 1 }",
            "Expected ':' but found '='",
            1,
            19,
        ),
        (
            "record = { amount.label }",
            "Expected ':' but found '.'",
            1,
            18,
        ),
        (
            "record = { amount true }",
            "Expected ':' but found keyword 'true'",
            1,
            19,
        ),
        (
            "record = { amount",
            "Expected ':' but found end of input",
            1,
            18,
        ),
        (
            "record = { amount\nother }",
            "Expected ':' but found end of line",
            1,
            12,
        ),
        (
            "record = { amount;\nother }",
            "Expected ':' but found ';'",
            1,
            18,
        ),
        (
            "record = { \"amount\" }",
            "Expected ':' but found '}'",
            1,
            21,
        ),
        ("record = { 1 }", "Expected ':' but found '}'", 1, 14),
        (
            "record = { true }",
            "Expected a key but found keyword 'true'",
            1,
            17,
        ),
        (
            "record = { this }",
            "Expected a key but found keyword 'this'",
            1,
            17,
        ),
        (
            "record = { self }",
            "Expected a key but found keyword 'this'",
            1,
            17,
        ),
        (
            "record = { $Amount }",
            "Expected a key but found '$Amount'",
            1,
            20,
        ),
        (
            "record = { [amount] }",
            "Expected a key but found '['",
            1,
            13,
        ),
        (
            "record = { `amount` }",
            "Expected a key but found a template string",
            1,
            21,
        ),
        (
            "record = { amount\n: amount }",
            "Expected ':' but found end of line",
            1,
            12,
        ),
        (
            "record = {\n    amount: amount,\n}",
            "Expected a key but found an indented block",
            2,
            5,
        ),
        (
            "def wrap amount => { amount: amount }",
            "Expected '(' but found identifier 'amount'",
            1,
            10,
        ),
        (
            "def wrap(amount) { amount: amount }",
            "Expected ':' but found '{'",
            1,
            18,
        ),
        ("{ amount, label }", "Unexpected ','", 1, 11),
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
fn malformed_explicit_fields_keep_their_expression_errors_and_positions() {
    for (source, message, line, column) in [
        ("record = { amount: }", "Unexpected '}'", 1, 21),
        (
            "record = { amount: 1 label: 2 }",
            "Expected '}' but found identifier 'label'",
            1,
            22,
        ),
        (
            "record = { amount: 1",
            "Expected '}' but found end of input",
            1,
            21,
        ),
        ("record = { amount:", "Unexpected end of input", 1, 19),
        (
            "record = { amount:\namount }",
            "Unexpected end of line",
            2,
            1,
        ),
        (
            "record = { amount: 1,, label: 2 }",
            "Expected a key but found ','",
            1,
            24,
        ),
        (
            "record = { amount: 1; label: 2 }",
            "Expected '}' but found identifier 'label'",
            1,
            23,
        ),
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
fn missing_commas_unfinished_objects_and_layout_cannot_mutate_earlier_state() {
    let mut runtime = Runtime::new();
    runtime.run("saved = 7").unwrap();
    // New shorthand prefixes can parse, but never hide an invalid later field.
    for (invalid, message, line, column) in [
        (
            "record = { amount label }",
            "Expected ':' but found identifier 'label'",
            1,
            19,
        ),
        (
            "record = { amount\nlabel }",
            "Expected ':' but found end of line",
            1,
            12,
        ),
        (
            "record = { amount; label }",
            "Expected ':' but found ';'",
            1,
            18,
        ),
        (
            "record = { amount",
            "Expected ':' but found end of input",
            1,
            18,
        ),
        ("record = { amount;\n", "Expected ':' but found ';'", 1, 18),
        (
            "record = { amount, label",
            "Expected ':' but found end of input",
            1,
            25,
        ),
        (
            "record = { amount,",
            "Expected a key but found end of input",
            1,
            19,
        ),
        ("record = ({ amount)", "Expected ':' but found ')'", 1, 19),
        (
            "record = { amount,, label }",
            "Expected a key but found ','",
            1,
            21,
        ),
        (
            "record = { amount\n: amount }",
            "Expected ':' but found end of line",
            1,
            12,
        ),
        (
            "record = {\n    amount,\n}",
            "Expected a key but found an indented block",
            2,
            5,
        ),
        (
            "record = { amount\n, label",
            "Expected ':' but found end of input",
            2,
            8,
        ),
        (
            "record = { amount,\nlabel other }",
            "Expected ':' but found identifier 'other'",
            2,
            7,
        ),
    ] {
        let error = Runtime::check(invalid).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{invalid}");
        assert_eq!(error.message(), message, "{invalid}");
        assert_eq!(
            error.position(),
            Some(Position::new(line, column)),
            "{invalid}"
        );
        let error = runtime.run(&format!("saved = 3\n{invalid}")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{invalid}");
        assert_eq!(error.message(), message, "{invalid}");
        assert_eq!(
            error.position(),
            Some(Position::new(line + 1, column)),
            "{invalid}"
        );
        assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        assert!(!runtime.graph.contains(&NodeKey::variable("record")));
    }
}

#[test]
fn every_field_evaluates_in_order_before_duplicate_keys_replace_values() {
    let runtimes = check_forms(
        r#"
amount = 3
events = { trace: "" }
def mark(label, value):
    events.trace = events.trace + label
    return value
"#,
        r#"record = { amount, first: mark("A", 1), amount: mark("B", 7), last: mark("C", 9), "first": mark("D", 4), nested: { amount }, items: [{ amount }], }"#,
        r#"record = { amount: amount, first: mark("A", 1), amount: mark("B", 7), last: mark("C", 9), "first": mark("D", 4), nested: { amount: amount }, items: [{ amount: amount }], }"#,
        r#"
assert(events.trace, "ABCD")
assert(record.amount, 7)
assert(record.first, 4)
assert(record.nested.amount, 3)
assert(record.items[0].amount, 3)
"#,
        5,
    );
    for runtime in runtimes {
        let Value::Object(id) = runtime.state.variable("record").unwrap() else {
            panic!("expected the existing object value");
        };
        let keys: Vec<&str> = runtime
            .state
            .object(id)
            .unwrap()
            .properties
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["amount", "first", "last", "nested", "items"]);
    }
}

#[test]
fn local_shadowing_receivers_and_object_aliases_use_existing_bindings() {
    check_forms(
        r#"
amount = 100
label = "outer"
payload = { count: 1 }
alias = payload
"#,
        r#"
def pack(amount, label, payload) => { amount, label, payload }
class Box:
    def init(amount):
        this.amount = amount
    def pack(amount):
        return { amount, receiver: self, payload: alias }
"#,
        r#"
def pack(amount, label, payload) => { amount: amount, label: label, payload: payload }
class Box:
    def init(amount):
        this.amount = amount
    def pack(amount):
        return { amount: amount, receiver: self, payload: alias }
"#,
        r#"
local = pack(7, "inner", payload)
box = Box(8)
received = box.pack(9)
assert(local.amount, 7)
assert(local.label, "inner")
assert(received.amount, 9)
assert(received.receiver.id, "box")
assert(received.receiver.amount, 8)
received.payload.count = 3
assert(alias.count, 3)
assert(payload.count, 3)
assert(local.payload.count, 3)
amount = 200
label = "changed"
assert(local.amount, 7)
assert(local.label, "inner")
assert(received.amount, 9)
"#,
        11,
    );
}

#[test]
fn reactive_reads_snapshots_strict_errors_and_rollback_preserve_dependency_edges() {
    let mut compact = Runtime::new();
    let mut explicit = Runtime::new();
    for (compact_source, explicit_source) in [
        (
            "amount = 2\nalternate = 7\nlive = { amount }\nfrozen = { amount: amount.value }\nderived = live.amount * 2",
            "amount = 2\nalternate = 7\nlive = { amount: amount }\nfrozen = { amount: amount.value }\nderived = live.amount * 2",
        ),
        ("amount = 5", "amount = 5"),
        (
            "try:\n    live = { alternate }\n    throw \"ABORT\"\ncatch error:\n    assert(error, \"ABORT\")",
            "try:\n    live = { alternate: alternate }\n    throw \"ABORT\"\ncatch error:\n    assert(error, \"ABORT\")",
        ),
        ("alternate = 9", "alternate = 9"),
        ("amount = 6", "amount = 6"),
    ] {
        equivalent(compact_source, explicit_source);
        let compact_value = compact.run(compact_source).unwrap();
        let explicit_value = explicit.run(explicit_source).unwrap();
        assert_eq!(
            compact.serialize_json(&compact_value).unwrap(),
            explicit.serialize_json(&explicit_value).unwrap()
        );
        // Anonymous property-node identities are runtime-local; compare the
        // named declarations' complete edges rather than their anonymous ids.
        assert_eq!(compact.graph.len(), explicit.graph.len());
        for name in ["live", "frozen"] {
            let key = NodeKey::variable(name);
            assert_eq!(
                compact.graph.retrieve(&key).unwrap().dependencies,
                explicit.graph.retrieve(&key).unwrap().dependencies,
                "{compact_source}: {name}"
            );
        }
    }
    for runtime in [&mut compact, &mut explicit] {
        assert_eq!(runtime.run("live.amount").unwrap(), Value::Number(6.0));
        assert_eq!(runtime.run("frozen.amount").unwrap(), Value::Number(2.0));
        assert_eq!(runtime.run("derived").unwrap(), Value::Number(12.0));
        let live = runtime.graph.retrieve(&NodeKey::variable("live")).unwrap();
        assert!(live.dependencies.contains(&NodeKey::variable("amount")));
        assert!(!live.dependencies.contains(&NodeKey::variable("alternate")));
        let frozen = runtime
            .graph
            .retrieve(&NodeKey::variable("frozen"))
            .unwrap();
        assert!(!frozen.dependencies.contains(&NodeKey::variable("amount")));
        assert_eq!(runtime.assertions_run(), 1);
        assert!(runtime.take_assertions().is_empty());
    }
    for (compact_source, explicit_source) in [
        ("record = { missing }", "record = { missing: missing }"),
        (
            "record = { missing, missing: 7 }",
            "record = { missing: missing, missing: 7 }",
        ),
    ] {
        let compact_error = compact.run(compact_source).unwrap_err();
        let explicit_error = explicit.run(explicit_source).unwrap_err();
        assert_eq!(compact_error, explicit_error);
        assert_eq!(compact_error.kind(), ErrorKind::Reference);
        assert_eq!(compact_error.message(), "missing is not defined");
        assert_eq!(compact_error.position(), Some(Position::new(1, 1)));
        assert!(!compact.graph.contains(&NodeKey::variable("record")));
        assert!(!explicit.graph.contains(&NodeKey::variable("record")));
    }

    // The shorthand must not add closure capture to returned anonymous functions.
    for fields in ["{ amount }", "{ amount: amount }"] {
        let source = format!(
            "def factory(amount):\n    return value => ({fields})\ncallback = factory(5)\ncallback(7)"
        );
        let error = Runtime::new().run(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "amount is not defined");
        assert_eq!(error.position(), Some(Position::new(4, 1)));
    }
}

#[test]
fn graph_keys_and_reasoning_sources_stay_quoted_and_expanded() {
    let mut compact = Runtime::new();
    let mut explicit = Runtime::new();
    for (compact_source, explicit_source) in [
        ("amount = 3\nenabled = false", "amount = 3\nenabled = false"),
        ("record = { amount }", "record = { amount: amount }"),
        (
            "{ nested = { record, items: [{ amount }] } }",
            "{ nested = { record: record, items: [{ amount: amount }] } }",
        ),
        (
            "if enabled and { amount }.amount > 0:\n    doubled = amount * 2",
            "if enabled and { amount: amount }.amount > 0:\n    doubled = amount * 2",
        ),
        ("amount = 4", "amount = 4"),
        ("why record", "why record"),
        ("model |> why", "model |> why"),
    ] {
        let compact_value = compact.run(compact_source).unwrap();
        let explicit_value = explicit.run(explicit_source).unwrap();
        assert_eq!(
            compact.serialize_json(&compact_value).unwrap(),
            explicit.serialize_json(&explicit_value).unwrap(),
            "{compact_source}"
        );
        assert!(
            compact.graph.keys().eq(explicit.graph.keys()),
            "{compact_source}"
        );
    }
    assert!(
        compact
            .graph
            .keys()
            .any(|key| key.as_str() == r#"if(enabled&&{"amount":amount}.amount>0)"#)
    );
    assert_eq!(
        NodeKey::block(&parse("record = { amount }").unwrap(), None).as_str(),
        r#"block(record={"amount":amount})"#
    );
    let trace = compact.run("why record").unwrap();
    let trace = compact.serialize_json(&trace).unwrap();
    assert!(trace.contains(r#"record = {\"amount\":amount}"#), "{trace}");
}

#[test]
fn shorthand_reuses_lexer_tokens_and_does_not_bypass_parser_resource_bounds() {
    assert_eq!(
        tokenize("{ amount, label }")
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        [
            Token::LeftBrace,
            Token::Identifier("amount".to_string()),
            Token::Comma,
            Token::Identifier("label".to_string()),
            Token::RightBrace,
            Token::Eof,
        ]
    );
    let tokens = tokenize("record = {\n    amount,\n}").unwrap();
    assert!(tokens.iter().any(|spanned| spanned.token == Token::Indent));
    assert!(tokens.iter().any(|spanned| spanned.token == Token::Dedent));

    let separators = ";\n".repeat(2048);
    equivalent(
        &format!("record = {{ amount{separators}}}"),
        &format!("record = {{ amount: amount{separators}}}"),
    );
    let error = Runtime::check(&format!("record = {{ amount{separators}")).unwrap_err();
    assert_eq!(error.message(), "Expected ':' but found ';'");
    assert_eq!(error.position(), Some(Position::new(1, 18)));

    for (source, message) in [
        (
            format!(
                "record = {}{{ amount }}{}",
                "{ next: ".repeat(200),
                " }".repeat(200)
            ),
            "Expressions are nested too deeply",
        ),
        (
            format!("record = {{ amount }}{}", ".next".repeat(4096)),
            "Expression has too many parts",
        ),
        (
            format!("{}amount{}", "{".repeat(200), "}".repeat(200)),
            "Statements are nested too deeply",
        ),
    ] {
        let error = Runtime::check(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), message);
    }
}

#[test]
fn cli_executes_shorthand_and_prints_expanded_canonical_graph_source() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("object-shorthand.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().next(), Some("6.0"));
    assert!(
        stdout.contains(r#""reading":reading,"items":[{"amount":amount}]"#),
        "{stdout}"
    );
    assert!(stdout.contains("reads    amount, label"), "{stdout}");
}

#[test]
fn cli_preserves_strict_reference_and_boundary_diagnostics() {
    for (fixture, error) in [
        (
            "object-shorthand-missing.nuc",
            "ReferenceError: missing is not defined (line 2, column 1)",
        ),
        (
            "object-shorthand-boundary.nuc",
            "SyntaxError: Expected ':' but found identifier 'other' (line 1, column 19)",
        ),
    ] {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(fixture);
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg(fixture)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert_eq!(String::from_utf8(output.stderr).unwrap().trim(), error);
    }
}

#[test]
fn repl_executes_single_line_object_expressions_without_changing_block_entry() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(
        b".json on\namount = 4\nlabel = \"meter\"\nreading = { amount, label }\namount = 6\nassert(reading.amount, 6)\nreading.amount\n.exit\n",
    ).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.lines().any(|line| line == "6.0"), "{stdout}");
}
