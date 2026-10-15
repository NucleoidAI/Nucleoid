//! Typed bare named arrows reuse the accepted parameter metadata and return block.

mod common;

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
                title: "typed named-arrow pair".to_string(),
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

#[test]
fn typed_metadata_and_named_return_block_are_exactly_the_parenthesized_form() {
    for (name, annotation) in [
        ("amount", "int"),
        ("_private", "ArbitraryType"),
        ("A1", "str"),
        ("type", "value"),
        ("why", "model"),
    ] {
        let bare = format!("def calculate {name}: {annotation} => 1");
        let statements = equivalent(&bare, &format!("def calculate({name}: {annotation}) => 1"));
        equivalent(
            &bare,
            &format!("def calculate({name}: {annotation}):\n    return 1"),
        );
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected a named function");
        };
        assert_eq!(function.name.as_deref(), Some("calculate"));
        assert_eq!(
            function.parameters,
            [Parameter {
                name: name.to_string(),
                type_name: Some(annotation.to_string()),
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
        "`total=${amount}`",
        "value => value * 2",
        "(value: int) => { return value * 2 }",
        "why amount",
    ] {
        let statements = equivalent(
            &format!("def calculate amount: int => {expression}"),
            &format!("def calculate(amount: int) => {expression}"),
        );
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected a named function");
        };
        let FunctionBody::Block(body) = &function.body else {
            panic!("named arrows must retain their return block");
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
fn the_immediate_typed_guard_uses_existing_tokens_and_statement_positions() {
    assert_eq!(
        tokenize("def double amount: int => amount * 2")
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
            Token::Arrow,
            Token::Identifier("amount".to_string()),
            Token::Star,
            Token::Number(2.0),
            Token::Eof,
        ]
    );
    for source in [
        "def double amount:int=>amount * 2",
        "def double\tamount:\tint\t=> amount * 2",
        "def double amount: int => (\n    amount *\n    2\n)",
        "def double amount: int => (\r\n    amount * 2\r\n)",
    ] {
        equivalent(source, "def double(amount: int) => amount * 2");
    }
    for suffix in ["\nnext = 3", "; next = 3", " # body\nnext = 3"] {
        equivalent(
            &format!("def double amount: int => amount * 2{suffix}"),
            &format!("def double(amount: int) => amount * 2{suffix}"),
        );
    }
    let program = parse_program("def double amount: int => amount * 2\nnext = 3").unwrap();
    assert_eq!(
        program.positions,
        [Position::new(1, 1), Position::new(2, 1)]
    );
}

#[test]
fn typed_lookahead_does_not_reinterpret_existing_untyped_arrow_or_colon_bodies() {
    let statements = equivalent(
        "def f amount:\n    int => amount",
        "def f(amount):\n    int => amount",
    );
    let [Stmt::Function(function)] = statements.as_slice() else {
        panic!("expected the accepted untyped block");
    };
    assert_eq!(function.parameters[0].type_name, None);
    let FunctionBody::Block(body) = &function.body else {
        panic!("expected an untyped colon suite");
    };
    assert!(matches!(
        body.as_slice(),
        [Stmt::Expression(Expr::Function(_))]
    ));
    check_forms(
        "def f amount:\n    int => amount\nassert(f(3), null)\ndef identity amount => amount\nassert(identity(4), 4)",
        "def f(amount):\n    int => amount\nassert(f(3), null)\ndef identity amount => amount\nassert(identity(4), 4)",
        2,
        None,
    );
}

#[test]
fn formerly_rejected_typed_arrow_inputs_now_have_positive_equivalent_coverage() {
    for (bare, explicit) in [
        (
            "def f amount: int => amount",
            "def f(amount: int) => amount",
        ),
        (
            "def identity amount: int => amount",
            "def identity(amount: int) => amount",
        ),
        (
            "class Gauge:\n    def reading amount: int => amount",
            "class Gauge:\n    def reading(amount: int) => amount",
        ),
        (
            "def wrap amount: int => { amount: amount }",
            "def wrap(amount: int) => { amount: amount }",
        ),
    ] {
        equivalent(bare, explicit);
    }
    check_forms(
        "def f amount: int => amount\ndef identity amount: int => amount\nclass Gauge:\n    def reading amount: int => amount\ndef wrap amount: int => { amount: amount }\ngauge = Gauge()\nassert(f(3), 3)\nassert(identity(4), 4)\nassert(gauge.reading(5), 5)\nassert(wrap(6).amount, 6)",
        "def f(amount: int) => amount\ndef identity(amount: int) => amount\nclass Gauge:\n    def reading(amount: int) => amount\ndef wrap(amount: int) => { amount: amount }\ngauge = Gauge()\nassert(f(3), 3)\nassert(identity(4), 4)\nassert(gauge.reading(5), 5)\nassert(wrap(6).amount, 6)",
        4,
        None,
    );
}

#[test]
fn every_unsupported_typed_prefix_keeps_the_parameter_boundary_position() {
    for source in [
        "def f amount: int, other:\n    return amount",
        "def f amount: int: return amount",
        "def f amount: int: { return amount }",
        "def f amount: int { return amount }",
        "def f amount: int, other, => amount",
        "def f amount: int, other: str, => amount",
        "def f amount: int = 1 => amount",
        "def f amount: int -> int => amount",
        "def f amount: Int.Name => amount",
        "def f amount: List[int] => amount",
        "def f amount: List<int> => amount",
        "def f amount: this => amount",
        "def f amount: null => amount",
        "def f amount: \"int\" => amount",
        "def f amount: 1 => amount",
        "def f amount: => amount",
        "def f amount: int",
        "def f amount: int\n=> amount",
        "def f amount: int; => amount",
        "def f amount: int # annotation\n=> amount",
        "def f amount:\nint => amount",
        "def f amount:; int => amount",
        "def f amount: # colon\nint => amount",
        "def f amount\n: int => amount",
        "def f amount;: int => amount",
        "def f amount # parameter\n: int => amount",
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
            "def f amount: int | str => amount",
            "Unexpected character '|'",
            Position::new(1, 19),
        ),
        (
            "def f\namount: int => amount",
            "Expected '(' but found end of line",
            Position::new(1, 5),
        ),
        (
            "def f; amount: int => amount",
            "Expected '(' but found ';'",
            Position::new(1, 6),
        ),
        (
            "class Gauge:\n    def reading amount: int, other, => amount",
            "Expected '(' but found identifier 'amount'",
            Position::new(2, 17),
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(position), "{source}");
    }
    assert!(Runtime::check("callback = amount: int => amount").is_err());
}

#[test]
fn the_shared_parameter_parser_preserves_legacy_forms_and_exact_diagnostics() {
    for signature in [
        "()",
        "(amount)",
        "(amount: int)",
        "(amount: ArbitraryType,)",
        "(amount: int, other: str)",
        "(\n    amount: int,\n    other: str,\n)",
    ] {
        equivalent(
            &format!("def f{signature} => 1"),
            &format!("def f{signature}:\n    return 1"),
        );
        assert!(Runtime::check(&format!("def f{signature}: return 1")).is_ok());
        assert!(Runtime::check(&format!("def f{signature}: {{ return 1 }}")).is_ok());
        let class = parse(&format!("class Gauge{signature}:\n    pass")).unwrap();
        let [Stmt::Class(class)] = class.as_slice() else {
            panic!("expected a class with its unchanged parameter list");
        };
        let named = parse(&format!("def f{signature} => 1")).unwrap();
        let [Stmt::Function(function)] = named.as_slice() else {
            panic!("expected a function");
        };
        assert_eq!(class.parameters, function.parameters);
    }
    for (source, annotation, block) in [
        ("amount => amount", None, false),
        ("(amount: int) => amount", Some("int"), false),
        (
            "(amount: ArbitraryType) => { return amount }",
            Some("ArbitraryType"),
            true,
        ),
        ("function(amount: str) { return amount }", Some("str"), true),
    ] {
        let Expr::Function(function) = parse_expression(source).unwrap() else {
            panic!("expected an unchanged anonymous function");
        };
        assert!(function.name.is_none());
        assert_eq!(function.parameters[0].type_name.as_deref(), annotation);
        assert_eq!(matches!(function.body, FunctionBody::Block(_)), block);
    }
    for (source, message, column) in [
        (
            "def f(amount:) => amount",
            "Expected a type but found ')'",
            16,
        ),
        (
            "def f(amount = 1) => amount",
            "Expected ')' but found '='",
            14,
        ),
        (
            "def f(amount, other:) => amount",
            "Expected a type but found ')'",
            23,
        ),
        ("def f(amount => amount", "Expected ')' but found '=>'", 14),
        (
            "def f(1) => 1",
            "Expected a parameter but found number 1",
            8,
        ),
        ("def f() -> int => 1", "Expected ':' but found '-'", 9),
        ("class Gauge(amount:)", "Expected a type but found ')'", 21),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(Position::new(1, column)), "{source}");
    }
}

#[test]
fn valid_typed_guards_keep_exact_downstream_expression_diagnostics() {
    for (suffix, message, line, column) in [
        ("", "Unexpected end of input", 1, 21),
        ("\n", "Unexpected end of line", 2, 1),
        (" (amount +)", "Unexpected ')'", 1, 32),
        (
            " { return amount }",
            "Expected a key but found keyword 'return'",
            1,
            31,
        ),
        (" { amount = 1 }", "Expected ':' but found '='", 1, 31),
        (
            " {\n    amount,\n}",
            "Expected a key but found an indented block",
            2,
            5,
        ),
        (" @", "Unexpected character '@'", 1, 22),
    ] {
        let bare = Runtime::check(&format!("def f amount: int =>{suffix}")).unwrap_err();
        let explicit = Runtime::check(&format!("def f(amount: int) =>{suffix}")).unwrap_err();
        assert_eq!(bare.kind(), ErrorKind::Syntax);
        assert_eq!(bare.kind(), explicit.kind());
        assert_eq!(bare.message(), message);
        assert_eq!(bare.message(), explicit.message());
        assert_eq!(bare.position(), Some(Position::new(line, column)));
        assert_eq!(
            explicit.position(),
            Some(Position::new(line, column + usize::from(line == 1)))
        );
    }
}

#[test]
fn annotations_stay_metadata_without_resolution_constraints_or_new_arity_rules() {
    let explicit = r#"
amount = 10
def identity(amount: ArbitraryType) => amount
def assign(amount: int) => amount = 2
def operation(number: int) => 99
def apply(operation: ArbitraryType) => operation(3)
def bare value => value + 1
def block(amount: int):
    return amount + 2
assert(identity(3), 3)
assert(identity(), null)
assert(identity(null), null)
assert(identity(3, 4), 3)
assert(identity("text"), "text")
assert(assign("anything"), 2)
assert(amount, 10)
assert(apply(null), null)
assert(apply(value => value * 2), 6)
try:
    apply(7)
catch error:
    assert(error, TypeError("operation is not a function"))
assert(operation(3), 99)
assert(bare(4), 5)
assert(block(4), 6)
identity("text")
"#;
    let bare = explicit
        .replace(
            "def identity(amount: ArbitraryType)",
            "def identity amount: ArbitraryType",
        )
        .replace("def assign(amount: int)", "def assign amount: int")
        .replace("def operation(number: int)", "def operation number: int")
        .replace(
            "def apply(operation: ArbitraryType)",
            "def apply operation: ArbitraryType",
        );
    for runtime in check_forms(&bare, explicit, 13, Some("\"text\"")) {
        assert!(!runtime.graph.contains(&NodeKey::variable("ArbitraryType")));
        assert!(!runtime.graph.contains(&NodeKey::variable("int")));
    }
}

#[test]
fn typed_redefinitions_live_references_snapshots_and_graphs_remain_equivalent() {
    let explicit = "base = 2\ndef scaled(amount: int) => amount + base\nlive = scaled\nsnapshot = scaled.value\nresult = live(3)\nfrozen = snapshot(3)\nassert(result, 5)\nassert(frozen, 5)\ndef scaled(amount: int) => amount * base\nassert(result, 6)\nassert(frozen, 5)\nbase = 4\nassert(result, 12)\nassert(frozen, 7)\nresult";
    let bare = explicit.replace("def scaled(amount: int)", "def scaled amount: int");
    let [mut bare, mut explicit] = check_forms(&bare, explicit, 6, Some("12"));
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
            &parse("def double amount: int => amount * 2").unwrap(),
            None
        )
        .as_str(),
        "block(def double)"
    );
}

#[test]
fn typed_methods_initializers_receivers_and_super_keep_the_accepted_path() {
    let explicit = "class Parent:\n    def init(amount: int) => this.amount = amount\n    def reading(offset: int) => self.amount + offset\nclass Child: Parent\n    def init(amount: int) => super(amount)\n    def reading(offset: int) => self.amount + offset\n    def increase(delta: int) => this.amount += delta\nchild = Child(4)\nobserved = child.reading(3)\nassert(child.id, \"child\")\nassert(child.amount, 4)\nassert(observed, 7)\nassert(child.increase(2), 6)\nassert(child.amount, 6)\nassert(observed, 9)\nassert(Parent.length, 0)\nassert(Child.length, 1)";
    let bare = explicit
        .replace("def init(amount: int)", "def init amount: int")
        .replace("def reading(offset: int)", "def reading offset: int")
        .replace("def increase(delta: int)", "def increase delta: int");
    check_forms(&bare, explicit, 8, None);
    check_forms(
        "class Parent:\n    def init amount: int => this.amount = amount\nclass Child: Parent\n    def init amount: int => super\nchild = Child(4)\nassert(child.amount, null)\nassert(Child.length, 1)",
        "class Parent:\n    def init(amount: int) => this.amount = amount\nclass Child: Parent\n    def init(amount: int) => super\nchild = Child(4)\nassert(child.amount, null)\nassert(Child.length, 1)",
        2,
        None,
    );
}

#[test]
fn typed_object_template_and_returned_anonymous_bodies_keep_existing_scope_rules() {
    check_forms(
        "def wrap amount: int => { amount }\ndef text amount: int => `n=${amount}`\ndef mapper unused: ArbitraryType => (value: int) => { return value * 2 }\nassert(wrap(5).amount, 5)\nassert(text(3), \"n=3\")\ncallback = mapper(99)\nassert(callback(3), 6)",
        "def wrap(amount: int) => { amount }\ndef text(amount: int) => `n=${amount}`\ndef mapper(unused: ArbitraryType) => (value: int) => { return value * 2 }\nassert(wrap(5).amount, 5)\nassert(text(3), \"n=3\")\ncallback = mapper(99)\nassert(callback(3), 6)",
        3,
        None,
    );
    for (bare, explicit, message, line) in [
        (
            "def wrap amount: int => { missing }\nwrap(5)",
            "def wrap(amount: int) => { missing }\nwrap(5)",
            "missing is not defined",
            2,
        ),
        (
            "def factory amount: int => value => ({ amount })\ncallback = factory(5)\ncallback(7)",
            "def factory(amount: int) => value => ({ amount })\ncallback = factory(5)\ncallback(7)",
            "amount is not defined",
            3,
        ),
        (
            "def receiver amount: int => this.amount\nreceiver(5)",
            "def receiver(amount: int) => this.amount\nreceiver(5)",
            "this is not defined",
            2,
        ),
        (
            "def outer amount:\n    def inner value: int => value * 2\n    offset = 1\n    if amount > 0:\n        return inner(amount) + offset\n    return offset\nassert(outer(3), 7)\nassert(outer(0), 1)",
            "def outer amount:\n    def inner(value: int) => value * 2\n    offset = 1\n    if amount > 0:\n        return inner(amount) + offset\n    return offset\nassert(outer(3), 7)\nassert(outer(0), 1)",
            "amount is not defined",
            8,
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
fn typed_redefinition_failures_and_unsupported_headers_preserve_rollback() {
    let mut runtimes = check_forms(
        "saved = 7\nsource = 3\ndef reading amount: int => source + amount\ntotal = reading(2)\nassert(total, 5)",
        "saved = 7\nsource = 3\ndef reading(amount: int) => source + amount\ntotal = reading(2)\nassert(total, 5)",
        1,
        None,
    );
    for (runtime, definition) in runtimes.iter_mut().zip([
        "def reading amount: int => missing + amount",
        "def reading(amount: int) => missing + amount",
    ]) {
        let error = runtime.run(definition).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.run("total").unwrap(), Value::Number(5.0));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
        for header in [
            "def reading amount: int, other, => amount",
            "def reading amount: int, other:\n    return amount",
            "def reading amount: int\n=> amount",
        ] {
            let error = runtime.run(&format!("saved = 3\n{header}")).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
            assert_eq!(runtime.run("reading(3)").unwrap(), Value::Number(6.0));
        }
        runtime.run("source = 4").unwrap();
        assert_eq!(runtime.run("total").unwrap(), Value::Number(6.0));
        assert_eq!(runtime.run("reading(3)").unwrap(), Value::Number(7.0));
    }
    assert!(runtimes[0].graph.keys().eq(runtimes[1].graph.keys()));
    for node in runtimes[0].graph.nodes() {
        let other = runtimes[1].graph.retrieve(&node.key).unwrap();
        assert_eq!(node.dependencies, other.dependencies);
        assert_eq!(node.dependents, other.dependents);
    }
}

#[test]
fn typed_bodies_keep_resource_limits_and_runtime_recovery() {
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
        for header in ["def limited amount: int =>", "def limited(amount: int) =>"] {
            let error = runtime
                .run(&format!("saved = 3\n{header} {body}"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert!(error.position().is_some());
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        }
    }
    runtime.run("def double amount: int => amount * 2").unwrap();
    assert_eq!(runtime.run("double(3)").unwrap(), Value::Number(6.0));
}

#[test]
fn native_cli_runs_typed_and_parenthesized_fixtures_with_identical_canonical_output() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let mut outputs = Vec::new();
    for fixture in [
        "bare-typed-named-arrows.nuc",
        "typed-named-arrows-explicit.nuc",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .args(["--json", "--graph"])
            .arg(root.join(fixture))
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.lines().next(), Some("7.0"));
        assert!(stdout.contains("scaled [function]"), "{stdout}");
        assert!(!stdout.contains("def scaled amount: int"), "{stdout}");
        outputs.push(stdout);
    }
    assert_eq!(outputs[0], outputs[1]);
    for (fixture, expected) in [
        (
            "bare-named-arrow-boundary.nuc",
            "SyntaxError: Expected '(' but found identifier 'amount' (line 1, column 7)",
        ),
        (
            "bare-typed-named-arrow-missing.nuc",
            "ReferenceError: missing is not defined (line 2, column 1)",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg(root.join(fixture))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert_eq!(String::from_utf8(output.stderr).unwrap().trim(), expected);
    }
}

#[test]
fn typed_repl_recovers_from_unsupported_headers_and_bounded_operand_errors() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let source = format!(
        ".json on\nbase = 2\ndef scaled amount: int => amount + base\nresult = scaled(3)\nbase = 4\nresult\n\
         def scaled amount: int, other, => amount\n\
         def limited amount: int => amount{}\n\
         def scaled amount: int => amount * base\nassert(result, 12)\nresult\n.exit\n",
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
