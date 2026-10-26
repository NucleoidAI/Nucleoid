//! A bare named-arrow parameter uses the existing parameter and return-block AST.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{
    Expr, FunctionBody, Parameter, Stmt, parse, parse_expression, parse_program,
};
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn equivalent(bare: &str, explicit: &str) {
    let bare_ast = parse(bare).unwrap();
    let explicit_ast = parse(explicit).unwrap();
    assert_eq!(bare_ast, explicit_ast, "{bare}");
    assert_eq!(
        generate_all(&bare_ast),
        generate_all(&explicit_ast),
        "{bare}"
    );
}

/// Compare complete programs, including the execution of every assertion.
fn check_forms(bare: &str, explicit: &str, assertions: usize) -> [Runtime; 2] {
    equivalent(bare, explicit);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let mut returns = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([bare, explicit]) {
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
fn bare_parameters_lower_to_the_same_named_return_block_and_canonical_source() {
    for name in [
        "amount", "_private", "A1", "type", "value", "why", "affects", "model",
    ] {
        let bare = format!("def calculate {name} => 1");
        let explicit = format!("def calculate({name}) => 1");
        equivalent(&bare, &explicit);
        equivalent(&bare, &format!("def calculate({name}):\n    return 1"));

        let statements = parse(&bare).unwrap();
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected one named function");
        };
        assert_eq!(function.name.as_deref(), Some("calculate"));
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
            FunctionBody::Block(vec![Stmt::Return(Some(Expr::Number(1.0)))])
        );
        assert_eq!(generate_all(&statements), "def calculate");
    }

    for expression in [
        "amount + 1 * factor",
        "amount = factor",
        "amount += factor",
        "{ amount, scaled: amount * factor }",
        "value => value * 2",
        "(value: int) => { return value * 2 }",
        "why amount",
    ] {
        let bare = format!("def calculate amount => {expression}");
        equivalent(&bare, &format!("def calculate(amount) => {expression}"));
        equivalent(
            &bare,
            &format!("def calculate(amount):\n    return {expression}"),
        );
        let statements = parse(&bare).unwrap();
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected one named function");
        };
        let FunctionBody::Block(body) = &function.body else {
            panic!("named arrows must keep block execution");
        };
        assert_eq!(
            body,
            &[Stmt::Return(Some(parse_expression(expression).unwrap()))]
        );
        assert_eq!(
            generate_all(body),
            format!("return {}", parse_expression(expression).unwrap())
        );
    }
}

#[test]
fn immediate_identifier_arrow_lookahead_reuses_existing_lexer_tokens() {
    assert_eq!(
        tokenize("def double amount => amount * 2")
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        [
            Token::Keyword(Keyword::Def),
            Token::Identifier("double".to_string()),
            Token::Identifier("amount".to_string()),
            Token::Arrow,
            Token::Identifier("amount".to_string()),
            Token::Star,
            Token::Number(2.0),
            Token::Eof,
        ]
    );
    for source in [
        "def double amount=>amount * 2",
        "def double\tamount\t=> amount * 2",
        "def double amount => (amount * 2)",
        "def double amount => (\n    amount *\n    2\n)",
    ] {
        equivalent(source, "def double(amount) => amount * 2");
    }
}

#[test]
fn methods_initializers_and_statement_boundaries_keep_the_existing_ast() {
    for (bare, explicit) in [
        (
            "class Gauge:\n    def init amount => this.amount = amount\n    def reading offset => self.amount + offset",
            "class Gauge:\n    def init(amount) => this.amount = amount\n    def reading(offset) => self.amount + offset",
        ),
        (
            "def double amount => amount * 2\nnext = 3",
            "def double(amount) => amount * 2\nnext = 3",
        ),
        (
            "def double amount => amount * 2; next = 3",
            "def double(amount) => amount * 2; next = 3",
        ),
        (
            "def double amount => amount * 2 # body\nnext = 3",
            "def double(amount) => amount * 2 # body\nnext = 3",
        ),
        (
            "{ def double amount => amount * 2; next = 3 }",
            "{ def double(amount) => amount * 2; next = 3 }",
        ),
        (
            "def outer(amount):\n    def double value => value * 2\n    return double(amount)",
            "def outer(amount):\n    def double(value) => value * 2\n    return double(amount)",
        ),
        (
            "def f amount => value => {\n    return value\n}",
            "def f(amount) => value => {\n    return value\n}",
        ),
    ] {
        equivalent(bare, explicit);
    }

    let program = parse_program("def double amount => amount * 2\nnext = 3").unwrap();
    assert_eq!(
        program.positions,
        [Position::new(1, 1), Position::new(2, 1)]
    );
}

#[test]
fn unguarded_headers_keep_exact_accepted_parameter_errors_and_positions() {
    // NUC27 adds this formerly unsupported prefix, with the exact explicit AST.
    equivalent(
        "def f amount, other => amount",
        "def f(amount, other) => amount",
    );
    // NUC29 adds the typed/mixed counterpart without changing these errors.
    equivalent(
        "def f amount: int, other => amount",
        "def f(amount: int, other) => amount",
    );
    equivalent(
        "class Gauge:\n    def reading amount: int, other => amount",
        "class Gauge:\n    def reading(amount: int, other) => amount",
    );
    // NUC37 adds a complete comma-free list, not a mixed-separator prefix.
    equivalent(
        "def f amount other => amount",
        "def f(amount, other) => amount",
    );
    // Captured using the unchanged accepted runtime before the parser edit.
    for (source, message, line, column) in [
        (
            "def f amount:",
            "Expected '(' but found identifier 'amount'",
            1,
            7,
        ),
        (
            "def f amount: int, other, => amount",
            "Expected '(' but found identifier 'amount'",
            1,
            7,
        ),
        (
            "def f amount\n=> amount",
            "Expected '(' but found identifier 'amount'",
            1,
            7,
        ),
        (
            "def f amount # boundary\n=> amount",
            "Expected '(' but found identifier 'amount'",
            1,
            7,
        ),
        (
            "def f amount; => amount",
            "Expected '(' but found identifier 'amount'",
            1,
            7,
        ),
        (
            "def f amount other, third => amount",
            "Expected '(' but found identifier 'amount'",
            1,
            7,
        ),
        (
            "def f amount = 1 => amount",
            "Expected '(' but found identifier 'amount'",
            1,
            7,
        ),
        (
            "def f amount -> int => amount",
            "Expected '(' but found identifier 'amount'",
            1,
            7,
        ),
        (
            "def f amount",
            "Expected '(' but found identifier 'amount'",
            1,
            7,
        ),
        ("def f => 1", "Expected '(' but found '=>'", 1, 7),
        (
            "def f\namount => amount",
            "Expected '(' but found end of line",
            1,
            5,
        ),
        (
            "def f; amount => amount",
            "Expected '(' but found ';'",
            1,
            6,
        ),
        ("def f", "Expected '(' but found end of input", 1, 6),
        ("def f 1 => 1", "Expected '(' but found number 1", 1, 7),
        (
            "def f this => this",
            "Expected '(' but found keyword 'this'",
            1,
            7,
        ),
        (
            "def f [amount] => amount",
            "Expected '(' but found '['",
            1,
            7,
        ),
        (
            "def f {amount} => amount",
            "Expected '(' but found '{'",
            1,
            7,
        ),
        (
            "def f $Amount => 1",
            "Expected '(' but found '$Amount'",
            1,
            7,
        ),
        (
            "def f \"amount\" => 1",
            "Expected '(' but found a string",
            1,
            7,
        ),
        (
            "def f ...amount => amount",
            "Expected '(' but found '.'",
            1,
            7,
        ),
        (
            "class Gauge:\n    def reading amount: int, other, => amount",
            "Expected '(' but found identifier 'amount'",
            2,
            17,
        ),
        (
            "def f(amount:) => amount",
            "Expected a type but found ')'",
            1,
            16,
        ),
        (
            "def f(amount = []) => amount",
            "Expected a scalar literal parameter default",
            1,
            16,
        ),
        (
            "def f(amount, other:) => amount",
            "Expected a type but found ')'",
            1,
            23,
        ),
        (
            "def f(amount => amount",
            "Expected ')' but found '=>'",
            1,
            14,
        ),
        (
            "def f(1) => 1",
            "Expected a parameter but found number 1",
            1,
            8,
        ),
        ("def f() -> int => 1", "Expected ':' but found '-'", 1, 9),
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
fn arrow_body_errors_and_brace_layout_still_use_the_accepted_expression_parser() {
    for (bare, explicit, message) in [
        (
            "def f amount =>",
            "def f(amount) =>",
            "Unexpected end of input",
        ),
        (
            "def f amount =>\n",
            "def f(amount) =>\n",
            "Unexpected end of line",
        ),
        (
            "def f amount =>\namount",
            "def f(amount) =>\namount",
            "Unexpected end of line",
        ),
        (
            "def f amount => { return amount }",
            "def f(amount) => { return amount }",
            "Expected a key but found keyword 'return'",
        ),
        (
            "def f amount => { amount = 1 }",
            "def f(amount) => { amount = 1 }",
            "Expected ':' but found '='",
        ),
        (
            "def f amount => (amount +)",
            "def f(amount) => (amount +)",
            "Unexpected ')'",
        ),
    ] {
        for source in [bare, explicit] {
            let error = Runtime::check(source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            assert_eq!(error.message(), message, "{source}");
            assert!(error.position().is_some(), "{source}");
        }
    }
    let bare_error = Runtime::check("def f amount => {\n    amount,\n}").unwrap_err();
    let explicit_error = Runtime::check("def f(amount) => {\n    amount,\n}").unwrap_err();
    assert_eq!(bare_error.kind(), explicit_error.kind());
    assert_eq!(bare_error.message(), explicit_error.message());
    assert_eq!(bare_error.kind(), ErrorKind::Syntax);
    assert_eq!(
        bare_error.message(),
        "Expected a key but found an indented block"
    );
    assert_eq!(bare_error.position(), Some(Position::new(2, 5)));
    assert_eq!(bare_error.position(), explicit_error.position());
}

#[test]
fn parenthesized_signatures_and_anonymous_forms_are_not_changed() {
    for signature in [
        "()",
        "(amount)",
        "(amount: int)",
        "(amount, other)",
        "(amount: int, other: str)",
        "(\n    amount: int,\n    other,\n)",
    ] {
        let statements = parse(&format!("def f{signature} => 1")).unwrap();
        assert_eq!(
            statements,
            parse(&format!("def f{signature}:\n    return 1")).unwrap()
        );
        assert!(parse(&format!("def f{signature}:\n    pass")).is_ok());
    }
    let typed = parse("def f(amount: int) => amount").unwrap();
    let [Stmt::Function(typed)] = typed.as_slice() else {
        panic!("expected one typed function");
    };
    assert_eq!(typed.parameters[0].type_name.as_deref(), Some("int"));

    for (source, block) in [
        ("amount => amount * 2", false),
        ("(amount) => amount * 2", false),
        ("amount => { return amount * 2 }", true),
        ("(amount: int) => { return amount * 2 }", true),
        ("function(amount) { return amount * 2 }", true),
    ] {
        let Expr::Function(function) = parse_expression(source).unwrap() else {
            panic!("expected an anonymous function");
        };
        assert!(function.name.is_none());
        assert_eq!(
            matches!(function.body, FunctionBody::Block(_)),
            block,
            "{source}"
        );
    }
    assert_eq!(
        parse_expression("double").unwrap(),
        Expr::Identifier("double".to_string())
    );
    // Adjacent identifiers remain separate expressions, never an implicit call.
    assert_eq!(
        parse("double amount").unwrap(),
        [
            Stmt::Expression(Expr::Identifier("double".to_string())),
            Stmt::Expression(Expr::Identifier("amount".to_string())),
        ]
    );
}

#[test]
fn arity_annotations_and_local_shadowing_keep_the_existing_bindings() {
    check_forms(
        r#"
amount = 10
def identity amount => amount
def assign amount => amount = 2
def typed(amount: int) => amount + 1
def block(amount):
    return amount + 2
def outer(amount):
    return identity(amount)
assert(identity(3), 3)
assert(identity(), null)
assert(identity(3, 4), 3)
assert(assign(99), 2)
assert(amount, 10)
assert(typed(4), 5)
assert(block(4), 6)
assert(outer(8), 8)
assert(amount, 10)
"#,
        r#"
amount = 10
def identity(amount) => amount
def assign(amount) => amount = 2
def typed(amount: int) => amount + 1
def block(amount):
    return amount + 2
def outer(amount):
    return identity(amount)
assert(identity(3), 3)
assert(identity(), null)
assert(identity(3, 4), 3)
assert(assign(99), 2)
assert(amount, 10)
assert(typed(4), 5)
assert(block(4), 6)
assert(outer(8), 8)
assert(amount, 10)
"#,
        9,
    );
    check_forms(
        r#"
def operation number => 99
def apply operation => operation(3)
assert(apply(null), null)
assert(apply(number => number * 2), 6)
try:
    apply(7)
catch error:
    assert(error, TypeError("operation is not a function"))
assert(operation(3), 99)
"#,
        r#"
def operation(number) => 99
def apply(operation) => operation(3)
assert(apply(null), null)
assert(apply(number => number * 2), 6)
try:
    apply(7)
catch error:
    assert(error, TypeError("operation is not a function"))
assert(operation(3), 99)
"#,
        4,
    );
}

#[test]
fn reactive_redefinition_live_references_and_snapshots_keep_identical_graphs() {
    let [mut bare, mut explicit] = check_forms(
        r#"
base = 2
def scaled amount => amount + base
live = scaled
snapshot = scaled.value
result = live(3)
frozen = snapshot(3)
assert(result, 5)
assert(frozen, 5)
def scaled amount => amount * base
assert(result, 6)
assert(frozen, 5)
base = 4
assert(result, 12)
assert(frozen, 7)
"#,
        r#"
base = 2
def scaled(amount) => amount + base
live = scaled
snapshot = scaled.value
result = live(3)
frozen = snapshot(3)
assert(result, 5)
assert(frozen, 5)
def scaled(amount) => amount * base
assert(result, 6)
assert(frozen, 5)
base = 4
assert(result, 12)
assert(frozen, 7)
"#,
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
        NodeKey::block(&parse("def double amount => amount * 2").unwrap(), None).as_str(),
        "block(def double)"
    );
}

#[test]
fn methods_and_expression_initializers_preserve_receivers_and_explicit_super() {
    check_forms(
        r#"
class Parent:
    def init amount => this.amount = amount
    def reading offset => self.amount + offset
class Child: Parent
    def init amount => super(amount)
    def reading offset => self.amount + offset
    def increase delta => this.amount += delta
child = Child(4)
observed = child.reading(3)
assert(child.id, "child")
assert(child.amount, 4)
assert(observed, 7)
assert(child.increase(2), 6)
assert(child.amount, 6)
assert(observed, 9)
assert(Parent.length, 0)
assert(Child.length, 1)
"#,
        r#"
class Parent:
    def init(amount) => this.amount = amount
    def reading(offset) => self.amount + offset
class Child: Parent
    def init(amount) => super(amount)
    def reading(offset) => self.amount + offset
    def increase(delta) => this.amount += delta
child = Child(4)
observed = child.reading(3)
assert(child.id, "child")
assert(child.amount, 4)
assert(observed, 7)
assert(child.increase(2), 6)
assert(child.amount, 6)
assert(observed, 9)
assert(Parent.length, 0)
assert(Child.length, 1)
"#,
        8,
    );
}

#[test]
fn object_bodies_and_returned_lambdas_keep_strict_reads_and_no_new_capture() {
    check_forms(
        "def wrap amount => { amount }\ndef mapper unused => value => value * 2\nassert(wrap(5).amount, 5)\ncallback = mapper(99)\nassert(callback(3), 6)",
        "def wrap(amount) => { amount }\ndef mapper(unused) => value => value * 2\nassert(wrap(5).amount, 5)\ncallback = mapper(99)\nassert(callback(3), 6)",
        2,
    );
    for (bare, explicit, message, line) in [
        (
            "def wrap amount => { missing }\nwrap(5)",
            "def wrap(amount) => { missing }\nwrap(5)",
            "missing is not defined",
            2,
        ),
        (
            "def factory amount => value => ({ amount })\ncallback = factory(5)\ncallback(7)",
            "def factory(amount) => value => ({ amount })\ncallback = factory(5)\ncallback(7)",
            "amount is not defined",
            3,
        ),
        (
            "def receiver amount => this.amount\nreceiver(5)",
            "def receiver(amount) => this.amount\nreceiver(5)",
            "this is not defined",
            2,
        ),
    ] {
        equivalent(bare, explicit);
        let bare_error = Runtime::new().run(bare).unwrap_err();
        let explicit_error = Runtime::new().run(explicit).unwrap_err();
        assert_eq!(bare_error, explicit_error, "{bare}");
        assert_eq!(bare_error.kind(), ErrorKind::Reference);
        assert_eq!(bare_error.message(), message);
        assert_eq!(bare_error.position(), Some(Position::new(line, 1)));
    }
}

#[test]
fn failed_redefinitions_restore_functions_results_and_dependency_edges() {
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, definition) in runtimes.iter_mut().zip([
        "def reading amount => source + amount",
        "def reading(amount) => source + amount",
    ]) {
        runtime
            .run(&format!("source = 3\n{definition}\ntotal = reading(2)"))
            .unwrap();
    }
    for (runtime, definition) in runtimes.iter_mut().zip([
        "def reading amount => missing + amount",
        "def reading(amount) => missing + amount",
    ]) {
        let error = runtime.run(definition).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.run("total").unwrap(), Value::Number(5.0));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
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
fn invalid_headers_do_not_execute_earlier_assignments_or_replace_functions() {
    let mut runtime = Runtime::new();
    runtime
        .run("saved = 7\ndef identity amount => amount")
        .unwrap();
    for header in [
        "def identity amount: int, other,:\n    return amount",
        "def identity amount: int, other, => amount",
        "def identity amount, other, => amount",
        "def identity amount\n=> amount",
        "def identity amount; => amount",
        "def identity => 1",
    ] {
        let error = runtime.run(&format!("saved = 3\n{header}")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        assert_eq!(runtime.run("identity(3)").unwrap(), Value::Number(3.0));
    }
    equivalent(
        "def identity amount, other => amount",
        "def identity(amount, other) => amount",
    );
    runtime.run("def identity amount, other => amount").unwrap();
    assert_eq!(runtime.run("identity(3, 4)").unwrap(), Value::Number(3.0));
    assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
}

#[test]
fn expression_resource_limits_still_report_errors_and_allow_runtime_recovery() {
    let mut runtime = Runtime::new();
    runtime.run("saved = 7").unwrap();
    for (body, message) in [
        (
            format!("{}amount", "-".repeat(4096)),
            "Expressions are nested too deeply",
        ),
        (
            format!("amount{}", " + 1".repeat(4096)),
            "Expression has too many operands",
        ),
        (
            format!("root{}", ".next".repeat(4096)),
            "Expression has too many parts",
        ),
        (
            format!("{}amount", "saved = ".repeat(4096)),
            "Expressions are nested too deeply",
        ),
    ] {
        let bare = format!("def limited amount => {body}");
        let explicit = format!("def limited(amount) => {body}");
        for source in [&bare, &explicit] {
            let error = runtime.run(&format!("saved = 3\n{source}")).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            assert_eq!(error.message(), message, "{source}");
            assert!(error.position().is_some());
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        }
    }
    runtime.run("def double amount => amount * 2").unwrap();
    assert_eq!(runtime.run("double(3)").unwrap(), Value::Number(6.0));
}

#[test]
fn cli_executes_bare_named_arrows_and_keeps_canonical_graph_source() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("bare-named-arrows.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().next(), Some("7.0"));
    assert!(stdout.contains("scaled [function]"), "{stdout}");
    assert!(
        stdout.contains("block(def double;doubled=double(3);assert(doubled,6))"),
        "{stdout}"
    );
    assert!(!stdout.contains("def double amount"), "{stdout}");
}

#[test]
fn cli_retains_parameter_boundary_and_runtime_reference_diagnostics() {
    for (fixture, error) in [
        (
            "bare-named-arrow-boundary.nuc",
            "SyntaxError: Expected '(' but found identifier 'amount' (line 1, column 7)",
        ),
        (
            "bare-named-arrow-missing.nuc",
            "ReferenceError: missing is not defined (line 2, column 1)",
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
fn repl_recovers_from_guard_and_operand_errors_without_changing_arrow_entry() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let source = format!(
        ".json on\nbase = 2\ndef scaled amount => amount + base\nresult = scaled(3)\nbase = 4\nresult\n\
         def scaled amount: int, other, => amount\n\
         def limited amount => amount{}\n\
         def scaled amount => amount * base\nassert(result, 12)\nresult\n.exit\n",
        " + 1".repeat(4096)
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
