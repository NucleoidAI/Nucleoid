//! One ordinary typed parameter before a real indented suite is optional sugar.

mod common;

use std::fs;
use std::io::{self, Write as _};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{
    Expr, FunctionBody, Parameter, Stmt, parse, parse_expression, parse_program,
};
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn equivalent(bare: &str, explicit: &str) -> Vec<Stmt> {
    let bare_ast = parse(bare).unwrap_or_else(|error| panic!("{bare}\n{error}"));
    let explicit_ast = parse(explicit).unwrap_or_else(|error| panic!("{explicit}\n{error}"));
    assert_eq!(bare_ast, explicit_ast, "{bare}");
    assert_eq!(
        generate_all(&bare_ast),
        generate_all(&explicit_ast),
        "{bare}"
    );
    bare_ast
}

fn check_forms(
    bare: &str,
    explicit: &str,
    assertions: usize,
    expected: Option<&str>,
) -> [Runtime; 2] {
    let statements = equivalent(bare, explicit);
    assert_eq!(
        common::assertions::protected_assertions(&statements).len(),
        assertions
    );
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, source) in runtimes.iter_mut().zip([bare, explicit]) {
        common::check_with_count(
            runtime,
            &common::Case {
                title: "typed indented block pair".to_string(),
                source: source.to_string(),
                expected: expected.map(str::to_string),
            },
            assertions,
        )
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
        assert_eq!(runtime.assertions_run(), assertions);
    }
    runtimes
}

fn named_graphs_match(before: &Runtime, after: &Runtime, names: &[&str]) {
    assert_eq!(before.graph.len(), after.graph.len());
    for name in names {
        let key = NodeKey::variable(*name);
        let before = before.graph.retrieve(&key).unwrap();
        let after = after.graph.retrieve(&key).unwrap();
        assert_eq!(before.kind, after.kind, "{key}");
        assert_eq!(before.dependencies, after.dependencies, "{key}");
        assert_eq!(before.dependents, after.dependents, "{key}");
    }
}

#[test]
fn typed_parameter_complete_block_ast_and_generator_match_the_explicit_form() {
    for (name, annotation) in [
        ("amount", "int"),
        ("_private", "ArbitraryType"),
        ("A1", "str"),
        ("type", "value"),
        ("why", "model"),
    ] {
        let bare = format!("def double {name}: {annotation}:\n    return {name} * 2");
        let explicit = format!("def double({name}: {annotation}):\n    return {name} * 2");
        let statements = equivalent(&bare, &explicit);
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected exactly one named function");
        };
        assert_eq!(function.name.as_deref(), Some("double"));
        assert_eq!(
            function.parameters,
            [Parameter {
                name: name.to_string(),
                type_name: Some(annotation.to_string()),
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
    let bare = "def double amount: int:\n    offset = 1\n    return amount * 2 + offset\nnext = 3";
    let explicit =
        "def double(amount: int):\n    offset = 1\n    return amount * 2 + offset\nnext = 3";
    equivalent(bare, explicit);
    let bare_program = parse_program(bare).unwrap();
    let explicit_program = parse_program(explicit).unwrap();
    assert_eq!(bare_program.positions, explicit_program.positions);
    assert_eq!(
        bare_program.positions,
        [Position::new(1, 1), Position::new(4, 1)]
    );
}

#[test]
fn exact_existing_tokens_and_qualifying_layout_need_a_real_indent() {
    assert_eq!(
        tokenize("def double amount: int:\n    return amount * 2")
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        [
            Token::Keyword(Keyword::Def),
            Token::Identifier("double".to_string()),
            Token::Identifier("amount".to_string()),
            Token::Colon,
            Token::Identifier("int".to_string()),
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
        "def double amount:int:\n    return amount * 2",
        "def double\tamount:\tint:\n\treturn amount * 2",
        "def double amount: int:\r\n    return amount * 2",
        "def double amount: int: # suite\n\n# blank\n    # body\n    return amount * 2",
        "def double amount: int:\n;\n    return amount * 2",
        "def double amount: int:\n    ;\n    return amount * 2",
        "def double amount: int:\n    return (\n        amount * 2\n    )",
    ] {
        equivalent(source, "def double(amount: int):\n    return amount * 2");
    }
    equivalent(
        "def double amount: int:\n    { return amount * 2 }",
        "def double(amount: int):\n    { return amount * 2 }",
    );
}

#[test]
fn formerly_negative_single_typed_headers_have_positive_full_ast_runtime_pairs() {
    // Exact former fixtures in the NUC18/19/20 tests, before adding assertions.
    for (bare, explicit, assertion) in [
        (
            "def f amount: int:\n    return amount",
            "def f(amount: int):\n    return amount",
            "assert(f(3), 3)",
        ),
        (
            "def identity amount: int:\n    return amount",
            "def identity(amount: int):\n    return amount",
            "assert(identity(4), 4)",
        ),
        (
            "class Gauge:\n    def reading amount: int:\n        return amount",
            "class Gauge:\n    def reading(amount: int):\n        return amount",
            "gauge = Gauge()\nassert(gauge.reading(5), 5)",
        ),
        (
            "saved = 3\ndef reading amount: int:\n    return amount",
            "saved = 3\ndef reading(amount: int):\n    return amount",
            "assert(saved, 3)\nassert(reading(6), 6)",
        ),
        (
            "def reading amount: int:\n    return amount",
            "def reading(amount: int):\n    return amount",
            "assert(reading(7), 7)",
        ),
        (
            "def scaled amount: int:\n    return amount",
            "def scaled(amount: int):\n    return amount",
            "assert(scaled(8), 8)",
        ),
    ] {
        equivalent(bare, explicit);
        check_forms(
            &format!("{bare}\n{assertion}"),
            &format!("{explicit}\n{assertion}"),
            if bare.starts_with("saved") { 2 } else { 1 },
            None,
        );
    }
}

#[test]
fn unsupported_typed_headers_do_not_consume_prefixes_or_body_layout() {
    for source in [
        "def f amount: int:",
        "def f amount: int:\n",
        "def f amount: int: # empty\n# comments only",
        "def f amount: int:\n    # indented comment only",
        "def f amount: int:\nreturn amount",
        "def f amount: int:\n;\nreturn amount",
        "def f amount: int: return amount",
        "def f amount: int: { return amount }",
        "def f amount: int { return amount }",
        "def f amount: int:;\n    return amount",
        "def f amount: int, other:\n    return amount",
        "def f amount: int, other: str:\n    return amount",
        "def f amount: int = 1:\n    return amount",
        "def f amount = 1: int:\n    return amount",
        "def f amount: int -> int:\n    return amount",
        "def f amount: Int.Name:\n    return amount",
        "def f amount: List[int]:\n    return amount",
        "def f amount: List<int>:\n    return amount",
        "def f amount: this:\n    return amount",
        "def f amount: self:\n    return amount",
        "def f amount: null:\n    return amount",
        "def f amount: \"int\":\n    return amount",
        "def f amount: 1:\n    return amount",
        "def f amount::\n    return amount",
        "def f amount:\nint:\n    return amount",
        "def f amount:; int:\n    return amount",
        "def f amount: # colon\nint:\n    return amount",
        "def f amount: int\n:\n    return amount",
        "def f amount: int;:\n    return amount",
        "def f amount: int # type\n:\n    return amount",
        "def f amount\n: int:\n    return amount",
        "def f amount;: int:\n    return amount",
        "def f amount # parameter\n: int:\n    return amount",
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
    for (source, message, position) in [
        (
            "def f\namount: int:\n    return amount",
            "Expected '(' but found end of line",
            Position::new(1, 5),
        ),
        (
            "def f # name\namount: int:\n    return amount",
            "Expected '(' but found end of line",
            Position::new(1, 5),
        ),
        (
            "def f; amount: int:\n    return amount",
            "Expected '(' but found ';'",
            Position::new(1, 6),
        ),
        (
            "def f:\n    return 1",
            "Expected '(' but found ':'",
            Position::new(1, 6),
        ),
        (
            "def f this: int:\n    return this",
            "Expected '(' but found keyword 'this'",
            Position::new(1, 7),
        ),
        (
            "def f [amount]: int:\n    return amount",
            "Expected '(' but found '['",
            Position::new(1, 7),
        ),
        (
            "def f {amount}: int:\n    return amount",
            "Expected '(' but found '{'",
            Position::new(1, 7),
        ),
        (
            "def f ...amount: int:\n    return amount",
            "Expected '(' but found '.'",
            Position::new(1, 7),
        ),
        (
            "def f amount: int | str:\n    return amount",
            "Unexpected character '|'",
            Position::new(1, 19),
        ),
        (
            "class Gauge:\n    def reading amount: int, other:\n        return amount",
            "Expected '(' but found identifier 'amount'",
            Position::new(2, 17),
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(position), "{source}");
    }
    for source in [
        "amount: int:\n    return amount",
        "callback = amount: int:\n    return amount",
        "function amount: int:\n    return amount",
    ] {
        assert_eq!(
            Runtime::check(source).unwrap_err().kind(),
            ErrorKind::Syntax
        );
    }
}

#[test]
fn qualifying_headers_keep_exact_downstream_suite_diagnostics_and_positions() {
    for (body, message, line, column) in [
        ("return amount +)", "Unexpected ')'", 2, 21),
        ("return @", "Unexpected character '@'", 2, 12),
        ("return (amount +)", "Unexpected ')'", 2, 22),
    ] {
        let bare = Runtime::check(&format!("def f amount: int:\n    {body}")).unwrap_err();
        let explicit = Runtime::check(&format!("def f(amount: int):\n    {body}")).unwrap_err();
        assert_eq!(bare, explicit);
        assert_eq!(bare.kind(), ErrorKind::Syntax);
        assert_eq!(bare.message(), message);
        assert_eq!(bare.position(), Some(Position::new(line, column)));
    }
}

#[test]
fn untyped_suite_colons_and_typed_arrow_eof_errors_are_not_reinterpreted() {
    let statements = equivalent(
        "def f amount:\n    int => amount",
        "def f(amount):\n    int => amount",
    );
    let [Stmt::Function(function)] = statements.as_slice() else {
        panic!("expected one untyped named function");
    };
    assert_eq!(function.parameters[0].type_name, None);
    let FunctionBody::Block(body) = &function.body else {
        panic!("expected the unchanged suite body");
    };
    assert!(matches!(
        body.as_slice(),
        [Stmt::Expression(Expr::Function(_))]
    ));
    check_forms(
        "def f amount:\n    int => amount\nassert(f(3), null)",
        "def f(amount):\n    int => amount\nassert(f(3), null)",
        1,
        None,
    );
    equivalent("def f amount => amount", "def f(amount) => amount");
    equivalent(
        "def f amount: int => amount",
        "def f(amount: int) => amount",
    );
    for (source, message, position) in [
        (
            "def f amount: int =>",
            "Unexpected end of input",
            Position::new(1, 21),
        ),
        (
            "def f amount: int =>\n",
            "Unexpected end of line",
            Position::new(2, 1),
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(position));
    }
}

#[test]
fn parenthesized_signatures_class_lists_and_anonymous_forms_keep_their_metadata() {
    for signature in [
        "()",
        "(amount)",
        "(amount: int)",
        "(amount: ArbitraryType,)",
        "(amount, other)",
        "(amount: int, other: str)",
        "(\n    amount: int,\n    other: str,\n)",
    ] {
        let statements = equivalent(
            &format!("def f{signature}:\n    return 1"),
            &format!("def f{signature}: return 1"),
        );
        equivalent(
            &format!("def f{signature} => 1"),
            &format!("def f{signature}:\n    return 1"),
        );
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected the existing named function");
        };
        let class = parse(&format!("class Gauge{signature}:\n    pass")).unwrap();
        let [Stmt::Class(class)] = class.as_slice() else {
            panic!("expected the existing class parameter list");
        };
        assert_eq!(class.parameters, function.parameters);
        let braced = parse(&format!("def f{signature}: {{ return 1 }}")).unwrap();
        let [Stmt::Function(braced)] = braced.as_slice() else {
            panic!("expected the existing braced suite");
        };
        assert_eq!(braced.parameters, function.parameters);
        assert_eq!(
            braced.body,
            FunctionBody::Block(vec![Stmt::Block(vec![Stmt::Return(Some(Expr::Number(
                1.0
            )))])])
        );
    }
    for (source, annotation, block) in [
        ("amount => amount", None, false),
        ("(amount: int) => amount", Some("int"), false),
        ("(amount: int) => { return amount }", Some("int"), true),
        ("function(amount: str) { return amount }", Some("str"), true),
    ] {
        let Expr::Function(function) = parse_expression(source).unwrap() else {
            panic!("expected the existing anonymous function");
        };
        assert!(function.name.is_none());
        assert_eq!(function.parameters[0].type_name.as_deref(), annotation);
        assert_eq!(matches!(function.body, FunctionBody::Block(_)), block);
    }
    for (source, message, column) in [
        (
            "def f(amount:):\n    return amount",
            "Expected a type but found ')'",
            15,
        ),
        (
            "def f(amount = 1): return amount",
            "Expected ')' but found '='",
            14,
        ),
        ("def f() -> int: return 1", "Expected ':' but found '-'", 9),
        ("class Gauge(amount:)", "Expected a type but found ')'", 21),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(1, column)));
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
fn annotations_binding_literal_locals_and_null_returns_keep_existing_scope() {
    let explicit = r#"amount = 10
def identity(amount: ArbitraryType):
    return amount
def double(amount: int):
    offset = 1
    return amount * 2 + offset
def ignored(amount: int):
    amount + 100
def empty(amount: int):
    return
def operation(number: int): return 99
def apply(operation: ArbitraryType):
    return operation(3)
def bare value:
    return value + 1
def typed(amount: int):
    return amount + 2
def arrow value: int => value + 3
def pair(left: int, right: int):
    return left + right
def unit():
    return 42
assert(identity(3), 3)
assert(identity(), null)
assert(identity(null), null)
assert(identity(3, 4), 3)
assert(identity("text"), "text")
assert(double(3), 7)
assert(ignored(5), null)
assert(empty(5), null)
assert(apply(null), null)
assert(apply(value => value * 2), 6)
try:
    apply(7)
catch error:
    assert(error, TypeError("operation is not a function"))
assert(operation(3), 99)
assert(bare(4), 5)
assert(typed(4), 6)
assert(arrow(4), 7)
assert(pair(2, 3), 5)
assert(unit(), 42)
assert(amount, 10)
identity("text")"#;
    let bare = explicit
        .replace(
            "def identity(amount: ArbitraryType):",
            "def identity amount: ArbitraryType:",
        )
        .replace("def double(amount: int):", "def double amount: int:")
        .replace("def ignored(amount: int):", "def ignored amount: int:")
        .replace("def empty(amount: int):", "def empty amount: int:")
        .replace(
            "def apply(operation: ArbitraryType):",
            "def apply operation: ArbitraryType:",
        );
    for runtime in check_forms(&bare, explicit, 18, Some("\"text\"")) {
        assert!(!runtime.graph.contains(&NodeKey::variable("ArbitraryType")));
        assert!(!runtime.graph.contains(&NodeKey::variable("int")));
    }
}

#[test]
fn both_return_paths_and_nested_noncapturing_definitions_execute_without_new_capture() {
    let explicit = "fee = 2\ndef total(amount: int):\n    if amount > 0:\n        return amount + fee\n    return fee\ninvoice = total(5)\nassert(invoice, 7)\nassert(total(0), 2)\nfee = 3\nassert(invoice, 8)\nassert(total(0), 3)\ninvoice";
    let bare = explicit.replace("def total(amount: int):", "def total amount: int:");
    let [before, after] = check_forms(&bare, explicit, 4, Some("8"));
    named_graphs_match(&before, &after, &["fee", "total", "invoice"]);
    check_forms(
        "def outer amount: int:\n    def inner value: int:\n        return value * 2\n    offset = 1\n    return inner(amount) + offset\nassert(outer(3), 7)",
        "def outer(amount: int):\n    def inner(value: int):\n        return value * 2\n    offset = 1\n    return inner(amount) + offset\nassert(outer(3), 7)",
        1,
        None,
    );
}

#[test]
fn live_references_definition_snapshots_replacement_and_named_graphs_stay_equivalent() {
    let explicit = "base = 2\ndef scaled(amount: ArbitraryType):\n    return amount + base\nlive = scaled\nsnapshot = scaled.value\nresult = live(3)\nfrozen = snapshot(3)\nassert(result, 5)\nassert(frozen, 5)\ndef scaled(amount: int):\n    return amount * base\nassert(result, 6)\nassert(frozen, 5)\nbase = 4\nassert(result, 12)\nassert(frozen, 7)\nresult";
    let bare = explicit
        .replace(
            "def scaled(amount: ArbitraryType):",
            "def scaled amount: ArbitraryType:",
        )
        .replace("def scaled(amount: int):", "def scaled amount: int:");
    let [mut before, mut after] = check_forms(&bare, explicit, 6, Some("12"));
    named_graphs_match(
        &before,
        &after,
        &["base", "scaled", "live", "snapshot", "result", "frozen"],
    );
    for source in ["why result", "model |> why"] {
        let before_value = before.run(source).unwrap();
        let after_value = after.run(source).unwrap();
        assert_eq!(
            before.serialize_json(&before_value).unwrap(),
            after.serialize_json(&after_value).unwrap()
        );
    }
    for (runtime, definition) in [&mut before, &mut after].into_iter().zip([
        "def scaled amount: int:\n    return amount + base",
        "def scaled(amount: int):\n    return amount + base",
    ]) {
        runtime.run("scaled = amount => amount + 10").unwrap();
        assert_eq!(runtime.run("result").unwrap(), Value::Number(13.0));
        runtime.run(definition).unwrap();
        assert_eq!(runtime.run("result").unwrap(), Value::Number(7.0));
    }
    named_graphs_match(&before, &after, &["scaled", "result"]);
}

#[test]
fn typed_methods_initializers_receivers_and_explicit_or_bare_super_keep_adoption() {
    let explicit = "class Parent:\n    def init(amount: int):\n        this.amount = amount\n    def reading(offset: int):\n        return self.amount + offset\nclass Child: Parent\n    def init(amount: int):\n        super(amount)\n        this.ready = true\n    def reading(offset: int):\n        return self.amount + offset\n    def increase(delta: int):\n        this.amount += delta\n        return self.amount\nchild = Child(4)\nobserved = child.reading(3)\nassert(child.id, \"child\")\nassert(child.ready, true)\nassert(observed, 7)\nassert(child.increase(2), 6)\nassert(observed, 9)\nassert(Parent.length, 0)\nassert(Child.length, 1)\nclass Flag:\n    def init():\n        this.ready = true\nclass Tagged: Flag\n    def init(amount: int):\n        super\n        self.amount = amount\ntagged = Tagged(8)\nassert(tagged.ready, true)\nassert(tagged.amount, 8)";
    let bare = explicit
        .replace("def init(amount: int):", "def init amount: int:")
        .replace("def reading(offset: int):", "def reading offset: int:")
        .replace("def increase(delta: int):", "def increase delta: int:");
    check_forms(&bare, explicit, 9, None);
    check_forms(
        "class Parent:\n    def init amount: int:\n        this.amount = amount\nclass Child: Parent\n    def init amount: int:\n        super\nchild = Child(4)\nassert(child.amount, null)\nassert(Child.length, 1)",
        "class Parent:\n    def init(amount: int):\n        this.amount = amount\nclass Child: Parent\n    def init(amount: int):\n        super\nchild = Child(4)\nassert(child.amount, null)\nassert(Child.length, 1)",
        2,
        None,
    );
}

#[test]
fn failed_redefinitions_caught_values_bad_headers_and_recovery_preserve_transactions() {
    let mut runtimes = check_forms(
        "saved = 7\nsource = 3\ndef reading amount: int:\n    return source + amount\ntotal = reading(2)\nassert(total, 5)",
        "saved = 7\nsource = 3\ndef reading(amount: int):\n    return source + amount\ntotal = reading(2)\nassert(total, 5)",
        1,
        None,
    );
    for (runtime, definition) in runtimes.iter_mut().zip([
        "def reading amount: int:\n    return missing + amount",
        "def reading(amount: int):\n    return missing + amount",
    ]) {
        let error = runtime.run(definition).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.run("total").unwrap(), Value::Number(5.0));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
        for header in [
            "def reading amount: int, other:\n    return amount",
            "def reading amount: int:\nreturn amount",
            "def reading amount: int: return amount",
        ] {
            let error = runtime.run(&format!("saved = 3\n{header}")).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(
                error.message(),
                "Expected '(' but found identifier 'amount'"
            );
            assert_eq!(error.position(), Some(Position::new(2, 13)));
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
            assert_eq!(runtime.run("reading(3)").unwrap(), Value::Number(6.0));
        }
        runtime.run("source = 4").unwrap();
        assert_eq!(runtime.run("total").unwrap(), Value::Number(6.0));
        assert_eq!(runtime.run("reading(3)").unwrap(), Value::Number(7.0));
    }
    named_graphs_match(
        &runtimes[0],
        &runtimes[1],
        &["saved", "source", "reading", "total"],
    );
    check_forms(
        "source = 3\ndef reading amount: int:\n    return source + amount\ntotal = reading(2)\ntry:\n    def reading amount: int:\n        return missing + amount\ncatch error:\n    assert(error, ReferenceError(\"missing is not defined\"))\nassert(total, 5)\nsource = 4\nassert(total, 6)",
        "source = 3\ndef reading(amount: int):\n    return source + amount\ntotal = reading(2)\ntry:\n    def reading(amount: int):\n        return missing + amount\ncatch error:\n    assert(error, ReferenceError(\"missing is not defined\"))\nassert(total, 5)\nsource = 4\nassert(total, 6)",
        3,
        None,
    );
}

#[test]
fn inherited_parameter_dependent_local_and_noncapturing_failures_remain_paired() {
    let mut runtimes = check_forms(
        "fee = 2\ndef total amount: int:\n    adjusted = amount + fee\n    return adjusted\ninvoice = total(5)\nassert(invoice, 7)",
        "fee = 2\ndef total(amount: int):\n    adjusted = amount + fee\n    return adjusted\ninvoice = total(5)\nassert(invoice, 7)",
        1,
        None,
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
    named_graphs_match(&runtimes[0], &runtimes[1], &["fee", "total", "invoice"]);
    for (bare, explicit, message, line) in [
        (
            "def f amount: int:\n    return this.amount\nf(3)",
            "def f(amount: int):\n    return this.amount\nf(3)",
            "this is not defined",
            3,
        ),
        (
            "def factory amount: int:\n    return value => ({ amount })\ncallback = factory(5)\ncallback(7)",
            "def factory(amount: int):\n    return value => ({ amount })\ncallback = factory(5)\ncallback(7)",
            "amount is not defined",
            4,
        ),
        (
            "def outer amount: int:\n    def inner value: int:\n        return value * 2\n    offset = 1\n    if amount > 0:\n        return inner(amount) + offset\n    return offset\nassert(outer(3), 7)\nassert(outer(0), 1)",
            "def outer(amount: int):\n    def inner(value: int):\n        return value * 2\n    offset = 1\n    if amount > 0:\n        return inner(amount) + offset\n    return offset\nassert(outer(3), 7)\nassert(outer(0), 1)",
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
fn bounded_depth_operand_postfix_and_assignment_errors_preserve_recovery() {
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
        let mut errors = Vec::new();
        for header in ["def limited amount: int:", "def limited(amount: int):"] {
            let error = runtime
                .run(&format!("saved = 3\n{header}\n    return {body}"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert!(error.position().is_some());
            errors.push(error);
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        }
        assert_eq!(errors[0], errors[1]);
    }
    let body = format!("{}pass{}", "{ ".repeat(65), " }".repeat(65));
    for header in ["def limited amount: int:", "def limited(amount: int):"] {
        let error = Runtime::check(&format!("{header}\n    {body}")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), "Statements are nested too deeply");
    }
    runtime
        .run("def double amount: int:\n    return amount * 2")
        .unwrap();
    assert_eq!(runtime.run("double(3)").unwrap(), Value::Number(6.0));
}

#[derive(Debug)]
struct NativeFile(PathBuf);

impl NativeFile {
    fn new(label: &str, source: &str) -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "nuc24-typed-block-{}-{label}.nuc",
            std::process::id()
        ));
        let writer = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        let file = Self(path);
        // Close the writer before guard cleanup, including on write errors.
        {
            let mut writer = writer;
            writer.write_all(source.as_bytes())?;
        }
        assert_eq!(fs::read(&file.0)?, source.as_bytes());
        Ok(file)
    }
}

impl Drop for NativeFile {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

#[test]
fn native_fixture_creation_preserves_existing_files_and_removes_only_owned_paths() {
    let source = "assert(1, 1)";
    let file = NativeFile::new("ownership-control", source).unwrap();
    let path = file.0.clone();
    let error = NativeFile::new("ownership-control", "replacement").unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
    drop(file);
    assert_eq!(
        fs::metadata(&path).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn native_ordinary_files_preserve_exact_eof_bytes_and_canonical_graph_output() {
    let explicit = "base = 2\ndef double(amount: int):\n    return amount * 2\nresult = double(3) + base\nassert(result, 8)\nbase = 1\nassert(result, 7)\nresult";
    let bare = explicit.replace("def double(amount: int):", "def double amount: int:");
    let mut outputs = Vec::new();
    for (label, source) in [("bare", bare.as_str()), ("explicit", explicit)] {
        assert!(!source.ends_with('\n'));
        let file = NativeFile::new(label, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .args(["--json", "--graph"])
            .arg(&file.0)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.lines().next(), Some("7.0"));
        assert!(stdout.contains("double [function]"), "{stdout}");
        assert!(!stdout.contains("def double amount: int"), "{stdout}");
        outputs.push(stdout);
    }
    assert_eq!(outputs[0], outputs[1]);
    for (index, source) in [
        "def double amount: int:\n    return amount * 2",
        "def double(amount: int):\n    return amount * 2",
    ]
    .into_iter()
    .enumerate()
    {
        let file = NativeFile::new(&format!("body-eof-{index}"), source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg("--json")
            .arg(&file.0)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, b"null\n");
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn native_ordinary_file_errors_and_assertion_failure_control_keep_exact_diagnostics() {
    for (index, source, message) in [
        (
            0,
            "def f amount: int:",
            "SyntaxError: Expected '(' but found identifier 'amount' (line 1, column 7)\n",
        ),
        (
            1,
            "def f amount: int:\n    return amount +)",
            "SyntaxError: Unexpected ')' (line 2, column 21)\n",
        ),
        (
            2,
            "def f amount: int:\n    return missing + amount\nf(3)",
            "ReferenceError: missing is not defined (line 3, column 1)\n",
        ),
    ] {
        let file = NativeFile::new(&format!("error-{index}"), source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg(&file.0)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, message.as_bytes());
    }
    let file = NativeFile::new("assertion-control", "assert(1, 2)").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
        .arg(&file.0)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"null\n");
    assert_eq!(output.stderr, b"assertion failed: expected 2, got 1\n");
}

fn repl(source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn repl_stdin_eof_runs_a_final_indented_body_in_both_forms() {
    let mut outputs = Vec::new();
    for header in ["def double amount: int:", "def double(amount: int):"] {
        let output = repl(&format!(
            ".json on\n.graph on\n{header}\n    return amount * 2"
        ));
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("double [function]"), "{stdout}");
        outputs.push(stdout);
    }
    assert_eq!(outputs[0], outputs[1]);
}

#[test]
fn repl_recovers_from_nonqualifying_headers_and_bounded_errors_before_stdin_eof() {
    let source = format!(
        ".json on\nbase = 2\ndef scaled amount: int:\n    return amount + base\n\nresult = scaled(3)\nbase = 4\nresult\n\
         def scaled amount: int, other:\n    return amount\n\n\
         def limited amount: int:\n    return amount{}\n\n\
         def scaled amount: int:\n    return amount * base\n\nassert(result, 12)\nresult",
        " + 1".repeat(200)
    );
    let output = repl(&source);
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
    assert!(!stderr.contains("assertion failed"), "{stderr}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.lines().any(|line| line == "7.0"), "{stdout}");
    assert!(stdout.lines().any(|line| line == "12.0"), "{stdout}");
}
