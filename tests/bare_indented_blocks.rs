//! One bare parameter before an indented colon suite keeps the named block AST.

mod common;

use std::env;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use indexmap::IndexMap;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{
    Expr, FunctionBody, Parameter, Stmt, parse, parse_expression, parse_program,
};
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde::Deserialize;

fn equivalent(bare: &str, explicit: &str) {
    let bare_ast = parse(bare).unwrap_or_else(|error| panic!("{bare}\n{error}"));
    let explicit_ast = parse(explicit).unwrap_or_else(|error| panic!("{explicit}\n{error}"));
    assert_eq!(bare_ast, explicit_ast, "{bare}");
    assert_eq!(
        generate_all(&bare_ast),
        generate_all(&explicit_ast),
        "{bare}"
    );
}

/// Protect assertions and coverage, not unconstrained process-global object IDs.
fn check_forms(bare: &str, explicit: &str, assertions: usize) -> [Runtime; 2] {
    equivalent(bare, explicit);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, source) in runtimes.iter_mut().zip([bare, explicit]) {
        runtime
            .run(source)
            .unwrap_or_else(|error| panic!("{source}\n{error}"));
        assert_eq!(runtime.assertions_run(), assertions, "{source}");
        let failures = runtime.take_assertions();
        assert!(failures.is_empty(), "{source}\n{failures:?}");
    }
    runtimes
}

#[test]
fn bare_parameter_keeps_function_metadata_body_and_canonical_source() {
    for name in [
        "amount", "_private", "A1", "type", "value", "why", "affects", "model",
    ] {
        let bare = format!("def double {name}:\n    return {name} * 2");
        let explicit = format!("def double({name}):\n    return {name} * 2");
        equivalent(&bare, &explicit);
        let statements = parse(&bare).unwrap();
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected one named function");
        };
        assert_eq!(function.name.as_deref(), Some("double"));
        assert_eq!(
            function.parameters,
            [Parameter {
                name: name.to_string(),
                type_name: None,
                default: None,
            }]
        );
        assert_eq!(
            function.body,
            FunctionBody::Block(vec![Stmt::Return(Some(
                parse_expression(&format!("{name} * 2")).unwrap()
            ))])
        );
        assert_eq!(generate_all(&statements), "def double");
    }

    // This exact NUC18 negative input is intentionally a positive NUC19 case.
    let [mut bare, mut explicit] = check_forms(
        "def identity amount:\n    return amount",
        "def identity(amount):\n    return amount",
        0,
    );
    for runtime in [&mut bare, &mut explicit] {
        assert_eq!(runtime.run("identity(3)").unwrap(), Value::Number(3.0));
    }
}

#[test]
fn identifier_colon_and_indentation_use_only_existing_tokens() {
    assert_eq!(
        tokenize("def double amount:\n    return amount * 2")
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        [
            Token::Keyword(Keyword::Def),
            Token::Identifier("double".to_string()),
            Token::Identifier("amount".to_string()),
            Token::Colon,
            Token::Newline,
            Token::Indent,
            Token::Keyword(Keyword::Return),
            Token::Identifier("amount".to_string()),
            Token::Star,
            Token::Number(2.0),
            Token::Dedent,
            Token::Eof,
        ]
    );
    for source in [
        "def double amount:\n    return amount * 2",
        "def double\tamount:\n\treturn amount * 2",
        "def double amount:\r\n    return amount * 2",
        "def double amount: # suite\n\n# blank\n    # body\n    return amount * 2",
        "def double amount:\n;\n    return amount * 2",
        "def double amount:\n    ;\n    return amount * 2",
    ] {
        equivalent(source, "def double(amount):\n    return amount * 2");
    }
}

#[test]
fn unsupported_headers_keep_parameter_errors_and_exact_positions() {
    // NUC28 adds this exact formerly unsupported indented header.
    equivalent(
        "def f amount, other:\n    return amount",
        "def f(amount, other):\n    return amount",
    );
    // NUC29 extends arrows; NUC30 separately adds typed/mixed indented blocks.
    equivalent(
        "def f amount: int, other => amount",
        "def f(amount: int, other) => amount",
    );
    // NUC40 turns only the complete real-indented comma-free form positive.
    check_forms(
        "def f amount other:\n    return amount\nassert(f(3, 4), 3)",
        "def f amount, other:\n    return amount\nassert(f(3, 4), 3)",
        1,
    );
    for source in [
        "def f amount:",
        "def f amount:\n",
        "def f amount:\n# no body",
        "def f amount:\nreturn amount",
        "def f amount:\n;\nreturn amount",
        "def f amount: int, other,:\n    return amount",
        "def f amount: int, other, => amount",
        "def f amount: return amount",
        "def f amount: { return amount }",
        "def f amount:;\n    return amount",
        "def f amount { return amount }",
        "def f amount, other:\nreturn amount",
        "def f amount, other:;\n    return amount",
        "def f amount, other,:\n    return amount",
        "def f amount other:\nreturn amount",
        "def f amount = 1:\n    return amount",
        "def f amount -> int:\n    return amount",
        "def f amount\n:\n    return amount",
        "def f amount # header\n:\n    return amount",
        "def f amount;:\n    return amount",
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(
            error.message(),
            "Expected '(' but found identifier 'amount'",
            "{source}"
        );
        assert_eq!(error.position(), Some(Position::new(1, 7)), "{source}");
    }
    for (source, message, line, column) in [
        ("def f:\n    return 1", "Expected '(' but found ':'", 1, 6),
        (
            "def f\namount:\n    return amount",
            "Expected '(' but found end of line",
            1,
            5,
        ),
        (
            "def f; amount:\n    return amount",
            "Expected '(' but found ';'",
            1,
            6,
        ),
        (
            "def f this:\n    return this",
            "Expected '(' but found keyword 'this'",
            1,
            7,
        ),
        (
            "def f [amount]:\n    return amount",
            "Expected '(' but found '['",
            1,
            7,
        ),
        (
            "def f {amount}:\n    return amount",
            "Expected '(' but found '{'",
            1,
            7,
        ),
        (
            "def f $Amount:\n    return 1",
            "Expected '(' but found '$Amount'",
            1,
            7,
        ),
        (
            "def f \"amount\":\n    return 1",
            "Expected '(' but found a string",
            1,
            7,
        ),
        (
            "def f 1:\n    return 1",
            "Expected '(' but found number 1",
            1,
            7,
        ),
        (
            "def f ...amount:\n    return amount",
            "Expected '(' but found '.'",
            1,
            7,
        ),
        (
            "def f(amount:):\n    return amount",
            "Expected a type but found ')'",
            1,
            15,
        ),
        (
            "def f(amount = []):\n    return amount",
            "Expected a scalar literal parameter default",
            1,
            16,
        ),
        (
            "def f() -> int:\n    return 1",
            "Expected ':' but found '-'",
            1,
            9,
        ),
        (
            "class Gauge:\n    def reading amount: int, other,:\n        return amount",
            "Expected '(' but found identifier 'amount'",
            2,
            17,
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
    for source in [
        "amount:\n    return amount",
        "function amount:\n    return amount",
    ] {
        assert_eq!(
            Runtime::check(source).unwrap_err().kind(),
            ErrorKind::Syntax
        );
    }
}

#[test]
fn body_errors_after_a_valid_guard_keep_suite_diagnostics_and_positions() {
    for (bare, explicit, message, line, column) in [
        (
            "def f amount:\n    return amount +)",
            "def f(amount):\n    return amount +)",
            "Unexpected ')'",
            2,
            21,
        ),
        (
            "def f amount:\n    return @",
            "def f(amount):\n    return @",
            "Unexpected character '@'",
            2,
            12,
        ),
    ] {
        let bare_error = Runtime::check(bare).unwrap_err();
        let explicit_error = Runtime::check(explicit).unwrap_err();
        assert_eq!(bare_error, explicit_error);
        assert_eq!(bare_error.kind(), ErrorKind::Syntax);
        assert_eq!(bare_error.message(), message);
        assert_eq!(bare_error.position(), Some(Position::new(line, column)));
    }
}

#[test]
fn legacy_parenthesized_suites_signatures_and_named_arrows_remain_available() {
    for signature in [
        "()",
        "(amount)",
        "(amount: int)",
        "(amount,)",
        "(amount, other)",
        "(amount: int, other: str)",
        "(\n    amount,\n)",
        "(\n    amount: int,\n    other,\n)",
    ] {
        let block = parse(&format!("def f{signature}:\n    return 1")).unwrap();
        assert_eq!(
            block,
            parse(&format!("def f{signature}: return 1")).unwrap()
        );
        let braced = parse(&format!("def f{signature}: {{ return 1 }}")).unwrap();
        let [Stmt::Function(function)] = braced.as_slice() else {
            panic!("expected one named function");
        };
        assert_eq!(
            function.body,
            FunctionBody::Block(vec![Stmt::Block(vec![Stmt::Return(Some(Expr::Number(
                1.0
            )))])])
        );
        assert_eq!(block, parse(&format!("def f{signature} => 1")).unwrap());
    }
    equivalent("def f amount => amount * 2", "def f(amount) => amount * 2");
    equivalent(
        "def f amount => (\n    amount * 2\n)",
        "def f(amount):\n    return amount * 2",
    );
    for source in [
        "amount => amount * 2",
        "(amount) => { return amount * 2 }",
        "function(amount) { return amount * 2 }",
        "class Gauge(amount):\n    this.amount = amount",
    ] {
        assert!(parse(source).is_ok(), "{source}");
    }
    assert_eq!(
        parse_expression("double").unwrap(),
        Expr::Identifier("double".to_string())
    );
    assert_eq!(
        parse("double amount").unwrap(),
        [
            Stmt::Expression(Expr::Identifier("double".to_string())),
            Stmt::Expression(Expr::Identifier("amount".to_string())),
        ]
    );
}

#[test]
fn nested_definitions_multiple_statements_and_return_paths_keep_scope() {
    check_forms(
        "def outer amount:\n    def inner value:\n        return value * 2\n    offset = 1\n    return inner(amount) + offset\nassert(outer(3), 7)",
        "def outer(amount):\n    def inner(value):\n        return value * 2\n    offset = 1\n    return inner(amount) + offset\nassert(outer(3), 7)",
        1,
    );
    check_forms(
        "fee = 2\ndef total amount:\n    if amount > 0:\n        return amount + fee\n    return fee\nassert(total(5), 7)\nassert(total(0), 2)",
        "fee = 2\ndef total(amount):\n    if amount > 0:\n        return amount + fee\n    return fee\nassert(total(5), 7)\nassert(total(0), 2)",
        2,
    );
    let program = parse_program("def double amount:\n    return amount * 2\nnext = 3").unwrap();
    assert_eq!(
        program.positions,
        [Position::new(1, 1), Position::new(3, 1)]
    );
}

#[test]
fn binding_and_block_null_returns_keep_explicit_alternatives() {
    check_forms(
        "amount = 10\ndef identity amount:\n    return amount\ndef ignored amount:\n    amount + 100\ndef empty amount:\n    return\ndef typed(amount: int): return amount + 1\ndef pair(left, right): return left + right\ndef unit(): return 42\ndef arrow amount => amount + 3\nassert(identity(3), 3)\nassert(identity(), null)\nassert(identity(3, 4), 3)\nassert(amount, 10)\nassert(ignored(5), null)\nassert(empty(5), null)\nassert(typed(4), 5)\nassert(pair(2, 3), 5)\nassert(unit(), 42)\nassert(arrow(4), 7)",
        "amount = 10\ndef identity(amount):\n    return amount\ndef ignored(amount):\n    amount + 100\ndef empty(amount):\n    return\ndef typed(amount: int): return amount + 1\ndef pair(left, right): return left + right\ndef unit(): return 42\ndef arrow amount => amount + 3\nassert(identity(3), 3)\nassert(identity(), null)\nassert(identity(3, 4), 3)\nassert(amount, 10)\nassert(ignored(5), null)\nassert(empty(5), null)\nassert(typed(4), 5)\nassert(pair(2, 3), 5)\nassert(unit(), 42)\nassert(arrow(4), 7)",
        10,
    );
    check_forms(
        "def operation(number): return 99\ndef apply operation:\n    return operation(3)\nassert(apply(null), null)\nassert(apply(number => number * 2), 6)\ntry:\n    apply(7)\ncatch error:\n    assert(error, TypeError(\"operation is not a function\"))\nassert(operation(3), 99)",
        "def operation(number): return 99\ndef apply(operation):\n    return operation(3)\nassert(apply(null), null)\nassert(apply(number => number * 2), 6)\ntry:\n    apply(7)\ncatch error:\n    assert(error, TypeError(\"operation is not a function\"))\nassert(operation(3), 99)",
        4,
    );
}

#[test]
fn reactive_references_snapshots_and_canonical_reasoning_keep_the_same_graph() {
    let [mut bare, mut explicit] = check_forms(
        "base = 2\ndef scaled amount:\n    return amount + base\nlive = scaled\nsnapshot = scaled.value\nresult = live(3)\nfrozen = snapshot(3)\nassert(result, 5)\nassert(frozen, 5)\ndef scaled amount:\n    return amount * base\nassert(result, 6)\nassert(frozen, 5)\nbase = 4\nassert(result, 12)\nassert(frozen, 7)",
        "base = 2\ndef scaled(amount):\n    return amount + base\nlive = scaled\nsnapshot = scaled.value\nresult = live(3)\nfrozen = snapshot(3)\nassert(result, 5)\nassert(frozen, 5)\ndef scaled(amount):\n    return amount * base\nassert(result, 6)\nassert(frozen, 5)\nbase = 4\nassert(result, 12)\nassert(frozen, 7)",
        6,
    );
    assert!(bare.graph.keys().eq(explicit.graph.keys()));
    for node in bare.graph.nodes() {
        let other = explicit.graph.retrieve(&node.key).unwrap();
        assert_eq!(node.kind, other.kind);
        assert_eq!(node.dependencies, other.dependencies);
        assert_eq!(node.dependents, other.dependents);
    }
    for source in ["why result", "model |> why"] {
        let bare_value = bare.run(source).unwrap();
        let explicit_value = explicit.run(source).unwrap();
        assert_eq!(
            bare.serialize_json(&bare_value).unwrap(),
            explicit.serialize_json(&explicit_value).unwrap()
        );
    }
    assert_eq!(
        NodeKey::block(
            &parse("def double amount:\n    return amount * 2").unwrap(),
            None
        )
        .as_str(),
        "block(def double)"
    );
    for runtime in [&mut bare, &mut explicit] {
        runtime.run("scaled = amount => amount + 10").unwrap();
        assert_eq!(runtime.run("result").unwrap(), Value::Number(13.0));
    }
}

#[test]
fn methods_init_and_explicit_or_bare_super_preserve_receiver_and_adoption() {
    check_forms(
        "class Parent:\n    def init amount:\n        this.amount = amount\n    def reading offset:\n        return self.amount + offset\nclass Child: Parent\n    def init amount:\n        super(amount)\n        this.ready = true\n    def reading offset:\n        return self.amount + offset\n    def increase delta:\n        this.amount += delta\n        return self.amount\nchild = Child(4)\nobserved = child.reading(3)\nassert(child.id, \"child\")\nassert(child.ready, true)\nassert(observed, 7)\nassert(child.increase(2), 6)\nassert(observed, 9)\nassert(Parent.length, 0)\nassert(Child.length, 1)\nclass Flag:\n    def init():\n        this.ready = true\nclass Tagged: Flag\n    def init amount:\n        super\n        self.amount = amount\ntagged = Tagged(8)\nassert(tagged.ready, true)\nassert(tagged.amount, 8)",
        "class Parent:\n    def init(amount):\n        this.amount = amount\n    def reading(offset):\n        return self.amount + offset\nclass Child: Parent\n    def init(amount):\n        super(amount)\n        this.ready = true\n    def reading(offset):\n        return self.amount + offset\n    def increase(delta):\n        this.amount += delta\n        return self.amount\nchild = Child(4)\nobserved = child.reading(3)\nassert(child.id, \"child\")\nassert(child.ready, true)\nassert(observed, 7)\nassert(child.increase(2), 6)\nassert(observed, 9)\nassert(Parent.length, 0)\nassert(Child.length, 1)\nclass Flag:\n    def init():\n        this.ready = true\nclass Tagged: Flag\n    def init(amount):\n        super\n        self.amount = amount\ntagged = Tagged(8)\nassert(tagged.ready, true)\nassert(tagged.amount, 8)",
        9,
    );
}

#[test]
fn failed_redefinitions_and_invalid_headers_roll_back_and_recover() {
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, definition) in runtimes.iter_mut().zip([
        "def reading amount:\n    return source + amount",
        "def reading(amount):\n    return source + amount",
    ]) {
        runtime
            .run(&format!(
                "source = 3\n{definition}\ntotal = reading(2)\nsaved = 7"
            ))
            .unwrap();
    }
    for (runtime, definition) in runtimes.iter_mut().zip([
        "def reading amount:\n    return missing + amount",
        "def reading(amount):\n    return missing + amount",
    ]) {
        let error = runtime.run(definition).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.run("total").unwrap(), Value::Number(5.0));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
        let error = runtime
            .run("saved = 3\ndef reading amount: int, other,:\n    return amount")
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(
            error.message(),
            "Expected '(' but found identifier 'amount'"
        );
        assert_eq!(error.position(), Some(Position::new(2, 13)));
        assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        runtime.run("source = 4").unwrap();
        assert_eq!(runtime.run("total").unwrap(), Value::Number(6.0));
        assert_eq!(runtime.run("reading(3)").unwrap(), Value::Number(7.0));
    }
    assert!(runtimes[0].graph.keys().eq(runtimes[1].graph.keys()));
    for node in runtimes[0].graph.nodes() {
        assert_eq!(
            node.dependencies,
            runtimes[1].graph.retrieve(&node.key).unwrap().dependencies
        );
    }
}

#[test]
fn receiver_and_nested_definition_errors_are_not_new_runtime_semantics() {
    for (bare, explicit, message, line) in [
        (
            "def f amount:\n    return this.amount\nf(3)",
            "def f(amount):\n    return this.amount\nf(3)",
            "this is not defined",
            3,
        ),
        (
            "def outer amount:\n    def inner value:\n        return value * 2\n    offset = 1\n    if amount > 0:\n        return inner(amount) + offset\n    return offset\nassert(outer(3), 7)\nassert(outer(0), 1)",
            "def outer(amount):\n    def inner(value):\n        return value * 2\n    offset = 1\n    if amount > 0:\n        return inner(amount) + offset\n    return offset\nassert(outer(3), 7)\nassert(outer(0), 1)",
            "amount is not defined",
            9,
        ),
    ] {
        equivalent(bare, explicit);
        let bare_error = Runtime::new().run(bare).unwrap_err();
        let explicit_error = Runtime::new().run(explicit).unwrap_err();
        assert_eq!(bare_error, explicit_error);
        assert_eq!(bare_error.kind(), ErrorKind::Reference);
        assert_eq!(bare_error.message(), message);
        assert_eq!(bare_error.position(), Some(Position::new(line, 1)));
    }
}

#[test]
fn parameter_dependent_local_reactivity_keeps_its_inherited_failure_and_rollback() {
    let mut runtimes = check_forms(
        "fee = 2\ndef total amount:\n    adjusted = amount + fee\n    return adjusted\ninvoice = total(5)\nassert(invoice, 7)",
        "fee = 2\ndef total(amount):\n    adjusted = amount + fee\n    return adjusted\ninvoice = total(5)\nassert(invoice, 7)",
        1,
    );
    let mut errors = Vec::new();
    for runtime in &mut runtimes {
        let error = runtime.run("fee = 3").unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "amount is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        errors.push(error);
        assert_eq!(runtime.run("fee").unwrap(), Value::Number(2.0));
        assert_eq!(runtime.run("invoice").unwrap(), Value::Number(7.0));
    }
    assert_eq!(errors[0], errors[1]);
}

#[test]
fn resource_limits_in_indented_bodies_still_reject_and_allow_recovery() {
    let mut runtime = Runtime::new();
    runtime.run("saved = 7").unwrap();
    for (body, message) in [
        (
            format!("{}amount", "-".repeat(200)),
            "Expressions are nested too deeply",
        ),
        (
            format!("amount{}", " + 1".repeat(200)),
            "Expression has too many operands",
        ),
        (
            format!("root{}", ".next".repeat(200)),
            "Expression has too many parts",
        ),
        (
            format!("{}amount", "saved = ".repeat(200)),
            "Expressions are nested too deeply",
        ),
    ] {
        for header in ["def limited amount:", "def limited(amount):"] {
            let error = runtime
                .run(&format!("saved = 3\n{header}\n    return {body}"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert!(error.position().is_some());
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        }
    }
    let body = format!("{}pass{}", "{ ".repeat(65), " }".repeat(65));
    for header in ["def limited amount:", "def limited(amount):"] {
        let source = format!("{header}\n    {body}");
        let error = Runtime::check(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), "Statements are nested too deeply");
    }
    runtime
        .run("def double amount:\n    return amount * 2")
        .unwrap();
    assert_eq!(runtime.run("double(3)").unwrap(), Value::Number(6.0));
}

#[test]
fn native_cli_runs_both_spellings_and_keeps_canonical_graph_output() {
    let mut outputs = Vec::new();
    for fixture in [
        "bare-indented-blocks.nuc",
        "parenthesized-indented-blocks.nuc",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(fixture);
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .args(["--json", "--graph"])
            .arg(path)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.lines().next(), Some("7.0"));
        assert!(
            stdout.contains("block(def double;doubled=double(3);assert(doubled,6))"),
            "{stdout}"
        );
        assert!(!stdout.contains("def double amount"), "{stdout}");
        outputs.push(stdout);
    }
    assert_eq!(outputs[0], outputs[1]);
}

#[test]
fn native_cli_keeps_unsupported_header_and_body_error_positions() {
    for (fixture, expected) in [
        (
            "bare-indented-block-boundary.nuc",
            "SyntaxError: Expected '(' but found identifier 'amount' (line 1, column 7)",
        ),
        (
            "bare-indented-block-missing.nuc",
            "ReferenceError: missing is not defined (line 3, column 1)",
        ),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(fixture);
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg(path)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(String::from_utf8(output.stderr).unwrap().trim(), expected);
    }
}

#[test]
fn repl_collects_indented_bodies_and_recovers_without_changing_arrow_forms() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let source = format!(
        ".json on\nbase = 2\ndef scaled amount:\n    return amount + base\n\nresult = scaled(3)\nbase = 4\nresult\n\
         def scaled amount: int, other,:\n    return amount\n\n\
         def limited amount:\n    return amount{}\n\n\
         def scaled amount => amount * base\nassert(result, 12)\nresult\n\
         def scaled(amount):\n    return amount + base\n\nassert(result, 7)\nresult\n.exit\n",
        " + 1".repeat(200)
    );
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 2, "{stderr}");
    assert!(
        stderr.contains(
            "SyntaxError: Expected '(' but found identifier 'amount' (line 1, column 12)"
        ),
        "{stderr}"
    );
    assert!(
        stderr.contains("SyntaxError: Expression has too many operands"),
        "{stderr}"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.lines().any(|line| line == "7.0"), "{stdout}");
    assert!(stdout.lines().any(|line| line == "12.0"), "{stdout}");
    assert!(!stderr.contains("assertion failed"), "{stderr}");
}

// This development-only check consumes sealed metadata but never reads or
// computes token counts, writes a report, or decides proposal acceptance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Document {
    Spec,
    Reference,
    Readme,
    Examples,
}

impl Document {
    const ALL: [Self; 4] = [Self::Spec, Self::Reference, Self::Readme, Self::Examples];

    fn path(self) -> &'static str {
        match self {
            Self::Spec => "nucleoid.spec.md",
            Self::Reference => "tests/reference.md",
            Self::Readme => "README.md",
            Self::Examples => "docs/examples.md",
        }
    }

    fn cases(self, source: &str) -> Vec<common::Case> {
        match self {
            Self::Spec | Self::Reference => common::cases(source),
            Self::Readme | Self::Examples => common::snippets(source, self.path()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct CaseId {
    document: Document,
    title: String,
}

#[derive(Deserialize)]
struct ProtectedCase {
    id: CaseId,
    assertions: Vec<String>,
    expected: Option<String>,
}

#[derive(Deserialize)]
struct CompatibilityBaseline {
    metric: String,
    revision: String,
    cases: Vec<ProtectedCase>,
}

#[derive(Deserialize)]
struct ExtraBaseline {
    document: Document,
    title: String,
    source: String,
    expected: Option<String>,
}

fn git(root: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("--no-pager")
        .args(arguments)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

/// Validate source edits from lexer positions, never regex text matches.
/// Unchanged eligible legacy headers are deliberately allowed.
fn delimiter_only_changes(legacy: &str, candidate: &str) -> usize {
    let legacy_lines: Vec<_> = legacy.trim().lines().collect();
    let candidate_lines: Vec<_> = candidate.trim().lines().collect();
    assert_eq!(
        legacy_lines.len(),
        candidate_lines.len(),
        "body/layout lines changed"
    );
    let tokens = tokenize(legacy.trim()).unwrap();
    let mut eligible = IndexMap::new();
    for (index, window) in tokens.windows(7).enumerate() {
        let [definition, name, left, parameter, right, colon, body] = window else {
            unreachable!("seven-token window")
        };
        if definition.token != Token::Keyword(Keyword::Def)
            || !matches!(name.token, Token::Identifier(_))
            || left.token != Token::LeftParen
            || !matches!(parameter.token, Token::Identifier(_))
            || right.token != Token::RightParen
            || colon.token != Token::Colon
            || !matches!(body.token, Token::Newline | Token::Indent)
            || window[..6]
                .iter()
                .any(|token| token.position.line != definition.position.line)
            || !tokens
                .iter()
                .skip(index + 6)
                .find(|token| !matches!(token.token, Token::Newline | Token::Semicolon))
                .is_some_and(|token| token.token == Token::Indent)
        {
            continue;
        }
        let line = definition.position.line - 1;
        let mut spelling: Vec<char> = legacy_lines[line].chars().collect();
        assert_eq!(spelling[left.position.column - 1], '(');
        assert_eq!(spelling[right.position.column - 1], ')');
        spelling[left.position.column - 1] = ' ';
        spelling.remove(right.position.column - 1);
        eligible.insert(line, spelling.into_iter().collect::<String>());
    }
    let mut changed = 0;
    for (line, (before, after)) in legacy_lines.iter().zip(&candidate_lines).enumerate() {
        if before != after {
            assert_eq!(
                eligible.get(&line).map(String::as_str),
                Some(*after),
                "only eligible named header delimiters may change at line {}",
                line + 1
            );
            changed += 1;
        }
    }
    changed
}

#[test]
fn source_compaction_does_not_touch_strings_templates_comments_or_other_forms() {
    let legacy = r#"
text = "def f(amount):"
template = `def f(amount):
    return amount`
# def f(amount):
def f(amount):
    return amount
def inline(amount): return amount
def typed(amount: int):
    return amount
def multiline(
    amount
):
    return amount
class Gauge(amount):
    this.amount = amount
def arrow amount => amount
"#;
    let candidate = legacy.replace("\ndef f(amount):\n", "\ndef f amount:\n");
    assert_eq!(delimiter_only_changes(legacy, &candidate), 1);
    equivalent(&candidate, legacy);
}

fn run_protected(case: &common::Case, protected: usize) -> usize {
    let mut runtime = Runtime::new();
    // Check the return with the normative JSON/UUID comparison in the same
    // execution whose coverage is measured, not in a second fresh runtime.
    common::check_with_count(&mut runtime, case, protected)
        .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
    runtime.assertions_run()
}

#[test]
fn protected_returns_and_coverage_come_from_the_checked_execution() {
    let case = common::Case {
        title: "checked block result".to_string(),
        source: "def double amount:\n    return amount * 2\nassert(double(3), 6)\ndouble(4)"
            .to_string(),
        expected: Some("8".to_string()),
    };
    let mut runtime = Runtime::new();
    runtime.run("assert(1, 1)").unwrap();
    let value = common::check_with_count(&mut runtime, &case, 1).unwrap();
    assert_eq!(value, Value::Number(8.0));
    assert_eq!(runtime.assertions_run(), 2);
    assert_eq!(run_protected(&case, 1), 1);
}

#[test]
#[should_panic(expected = "returned 2.0, expected 3")]
fn protected_cases_reject_wrong_expected_returns() {
    run_protected(
        &common::Case {
            title: "wrong expected return".to_string(),
            source: "assert(1, 1)\n2".to_string(),
            expected: Some("3".to_string()),
        },
        1,
    );
}

#[test]
#[should_panic(expected = "only 0 of 1 assertions ran")]
fn protected_cases_reject_unexecuted_assertions() {
    run_protected(
        &common::Case {
            title: "unexecuted assertion".to_string(),
            source: "if false:\n    assert(1, 2)\n3".to_string(),
            expected: Some("3".to_string()),
        },
        1,
    );
}

#[test]
#[should_panic(expected = "1 assertion(s): expected 2, got 1")]
fn protected_cases_reject_failed_assertions() {
    run_protected(
        &common::Case {
            title: "failed assertion".to_string(),
            source: "assert(1, 2)\n3".to_string(),
            expected: Some("3".to_string()),
        },
        1,
    );
}

#[test]
#[ignore = "requires caller-sealed NUC19_COMPAT_BASELINE and NUC19_COMPAT_EXTRAS"]
fn pinned_corpus_preserves_ast_assertions_expected_returns_and_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let baseline: CompatibilityBaseline = serde_json::from_str(
        &fs::read_to_string(env::var_os("NUC19_COMPAT_BASELINE").expect("sealed baseline path"))
            .unwrap(),
    )
    .unwrap();
    let extras: Vec<ExtraBaseline> = serde_json::from_str(
        &fs::read_to_string(env::var_os("NUC19_COMPAT_EXTRAS").expect("sealed extras path"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(baseline.metric, "nucleoid-lexer-with-layout-v1");
    assert_eq!(git(root, &["rev-parse", "HEAD"]).trim(), baseline.revision);
    let mut legacy = IndexMap::new();
    let mut candidate = IndexMap::new();
    for document in Document::ALL {
        for (destination, source) in [
            (
                &mut legacy,
                git(
                    root,
                    &[
                        "show",
                        &format!("{}:{}", baseline.revision, document.path()),
                    ],
                ),
            ),
            (
                &mut candidate,
                fs::read_to_string(root.join(document.path())).unwrap(),
            ),
        ] {
            for case in document.cases(&source) {
                let id = CaseId {
                    document,
                    title: case.title.clone(),
                };
                assert!(destination.insert(id, case).is_none(), "duplicate identity");
            }
        }
    }
    for extra in extras {
        let id = CaseId {
            document: extra.document,
            title: extra.title.clone(),
        };
        assert!(
            legacy
                .insert(
                    id,
                    common::Case {
                        title: extra.title,
                        source: extra.source,
                        expected: extra.expected
                    }
                )
                .is_none(),
            "duplicate extra identity"
        );
    }
    assert_eq!(legacy.len(), baseline.cases.len());
    assert_eq!(candidate.len(), baseline.cases.len());
    for document in Document::ALL {
        let expected: Vec<_> = baseline
            .cases
            .iter()
            .filter(|case| case.id.document == document)
            .map(|case| &case.id)
            .collect();
        assert_eq!(
            legacy
                .keys()
                .filter(|id| id.document == document)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            candidate
                .keys()
                .filter(|id| id.document == document)
                .collect::<Vec<_>>(),
            expected
        );
    }
    let mut protected = 0;
    let mut executed = [0, 0];
    let mut expected_returns = 0;
    let mut shortened_headers = 0;
    for case in &baseline.cases {
        let legacy = legacy.get(&case.id).unwrap();
        let candidate = candidate.get(&case.id).unwrap();
        for source in [legacy, candidate] {
            assert_eq!(source.expected, case.expected, "{:?}", case.id);
            assert_eq!(
                common::assertions::protected_assertions(&parse(&source.source).unwrap()),
                case.assertions,
                "{:?}",
                case.id
            );
        }
        shortened_headers += delimiter_only_changes(&legacy.source, &candidate.source);
        equivalent(&candidate.source, &legacy.source);
        let pair = [
            run_protected(legacy, case.assertions.len()),
            run_protected(candidate, case.assertions.len()),
        ];
        assert_eq!(pair[0], pair[1], "{:?}: executed coverage changed", case.id);
        for (total, count) in executed.iter_mut().zip(pair) {
            *total += count;
        }
        protected += case.assertions.len();
        expected_returns += usize::from(case.expected.is_some());
    }
    println!(
        "{} paired programs preserve document/title/order, complete ASTs, canonical generators, {protected} ordered protected assertions, {expected_returns} explicit expected returns, and executed coverage {executed:?}. Only {shortened_headers} eligible named-header delimiter pairs changed; all body/layout lines and other spellings stayed intact. No token measurement or acceptance evaluation ran.",
        baseline.cases.len()
    );
}
