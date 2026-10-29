//! Bare multi-parameter headers are an optional spelling of named return blocks.

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{
    Expr, FunctionBody, Parameter, Stmt, parse, parse_expression, parse_program,
};
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn equivalent(bare: &str, explicit: &str) -> Vec<Stmt> {
    let bare_ast = parse(bare).unwrap();
    let explicit_ast = parse(explicit).unwrap();
    assert_eq!(bare_ast, explicit_ast, "{bare}");
    assert_eq!(
        generate_all(&bare_ast),
        generate_all(&explicit_ast),
        "{bare}"
    );
    bare_ast
}

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
fn complete_lists_preserve_parameter_order_metadata_and_named_return_lowering() {
    for names in [
        vec!["left", "right"],
        vec!["left", "right", "factor"],
        vec!["type", "value", "_private", "A1", "why", "affects", "model"],
    ] {
        let parameters = names.join(", ");
        let bare = format!("def calculate {parameters} => 1");
        let explicit = format!("def calculate({parameters}) => 1");
        let statements = equivalent(&bare, &explicit);
        equivalent(
            &bare,
            &format!("def calculate({parameters}):\n    return 1"),
        );
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected one named function");
        };
        assert_eq!(function.name.as_deref(), Some("calculate"));
        assert_eq!(
            function.parameters,
            names
                .into_iter()
                .map(|name| Parameter {
                    name: name.to_string(),
                    type_name: None,
                    default: None,
                })
                .collect::<Vec<_>>()
        );
        assert_eq!(
            function.body,
            FunctionBody::Block(vec![Stmt::Return(Some(Expr::Number(1.0)))])
        );
        assert_eq!(generate_all(&statements), "def calculate");
    }

    for expression in [
        "left + right * factor",
        "left = right",
        "left += right",
        "left ||= right",
        "{ left, right }",
        "[left, right]",
        "value => value * 2",
        "(left, right) => ({ left, right })",
        "function(value) { return value }",
        "why left",
    ] {
        let bare = format!("def calculate left, right => {expression}");
        let statements = equivalent(
            &bare,
            &format!("def calculate(left, right) => {expression}"),
        );
        equivalent(
            &bare,
            &format!("def calculate(left, right):\n    return {expression}"),
        );
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected one named function");
        };
        assert_eq!(
            function.body,
            FunctionBody::Block(vec![Stmt::Return(Some(
                parse_expression(expression).unwrap()
            ))])
        );
    }
}

#[test]
fn lookahead_uses_existing_tokens_and_preserves_body_and_statement_boundaries() {
    assert_eq!(
        tokenize("def add left, right => left + right")
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        [
            Token::Keyword(Keyword::Def),
            Token::Identifier("add".to_string()),
            Token::Identifier("left".to_string()),
            Token::Comma,
            Token::Identifier("right".to_string()),
            Token::Arrow,
            Token::Identifier("left".to_string()),
            Token::Plus,
            Token::Identifier("right".to_string()),
            Token::Eof,
        ]
    );
    for source in [
        "def add left,right=>left + right",
        "def add\tleft,\tright\t=> left + right",
        "def add left, right => (left + right)",
        "def add left, right => (\n    left +\n    right\n)",
    ] {
        equivalent(source, "def add(left, right) => left + right");
    }
    for suffix in ["\nnext = 3", "; next = 3", " # body\nnext = 3"] {
        equivalent(
            &format!("def add left, right => left + right{suffix}"),
            &format!("def add(left, right) => left + right{suffix}"),
        );
    }
    equivalent(
        "{ def add left, right => left + right; next = 3 }",
        "{ def add(left, right) => left + right; next = 3 }",
    );
    equivalent(
        "def outer(initial):\n    def combine left, right, scale => (left + right) * scale\n    return combine(initial, 2, 3)",
        "def outer(initial):\n    def combine(left, right, scale) => (left + right) * scale\n    return combine(initial, 2, 3)",
    );
    assert_eq!(
        parse_program("def add left, right => left + right\nnext = 3")
            .unwrap()
            .positions,
        [Position::new(1, 1), Position::new(2, 1)]
    );
}

#[test]
fn incomplete_or_unsupported_lists_preserve_parenthesized_parser_errors_and_positions() {
    // NUC28 adds only the real-indented block counterpart, not inline suites.
    equivalent(
        "def f left, right:\n    return left",
        "def f(left, right):\n    return left",
    );
    for header in [
        "left, right",
        "left, right, => left",
        "left, => left",
        "left,, right => left",
        "left, right other => left",
        "left: int, right, => left",
        "left, right: int, => left",
        "left: int, right: int, => left",
        "left = 1, right => left",
        "left, right = 1 => left",
        "left, [right] => left",
        "left, {right} => left",
        "left, ...right => left",
        "left, this => left",
        "left, self => left",
        "left, $Right => left",
        "left, 1 => left",
        "left, \"right\" => left",
        "left, right:\nreturn left",
        "left, right:;\n    return left",
        "left, right,:\n    return left",
        "left, right: return left",
        "left, right: { return left }",
        "left, right - > int => left",
        "left, right\n=> left",
        "left,\nright => left",
        "left\n, right => left",
        "left, right; => left",
        "left, right # boundary\n=> left",
        "left, # boundary\nright => left",
    ] {
        let source = format!("def f {header}");
        let error = Runtime::check(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(
            error.message(),
            "Expected '(' but found identifier 'left'",
            "{source}"
        );
        assert_eq!(error.position(), Some(Position::new(1, 7)), "{source}");
    }
    for (source, message, position) in [
        (
            "def f this, right => right",
            "Expected '(' but found keyword 'this'",
            Position::new(1, 7),
        ),
        (
            "def f\nleft, right => left",
            "Expected '(' but found end of line",
            Position::new(1, 5),
        ),
        (
            "def f; left, right => left",
            "Expected '(' but found ';'",
            Position::new(1, 6),
        ),
        (
            "def f(left, right:) => left",
            "Expected a type but found ')'",
            Position::new(1, 21),
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(position), "{source}");
    }
}

#[test]
fn qualifying_headers_keep_exact_expression_and_lexical_diagnostics() {
    for (body, message, bare_position, explicit_position) in [
        (
            "",
            "Unexpected end of input",
            Position::new(1, 21),
            Position::new(1, 22),
        ),
        (
            "\n",
            "Unexpected end of line",
            Position::new(2, 1),
            Position::new(2, 1),
        ),
        (
            " (left +)",
            "Unexpected ')'",
            Position::new(1, 30),
            Position::new(1, 31),
        ),
        (
            " { return left }",
            "Expected a key but found keyword 'return'",
            Position::new(1, 31),
            Position::new(1, 32),
        ),
        (
            " { left = 1 }",
            "Expected ':' but found '='",
            Position::new(1, 29),
            Position::new(1, 30),
        ),
        (
            " {\n    left,\n}",
            "Expected a key but found an indented block",
            Position::new(2, 5),
            Position::new(2, 5),
        ),
        (
            " @",
            "Unexpected character '@'",
            Position::new(1, 22),
            Position::new(1, 23),
        ),
    ] {
        for (source, position) in [
            (format!("def f left, right =>{body}"), bare_position),
            (format!("def f(left, right) =>{body}"), explicit_position),
        ] {
            let error = Runtime::check(&source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            assert_eq!(error.message(), message, "{source}");
            assert_eq!(error.position(), Some(position), "{source}");
        }
    }
}

#[test]
fn arity_local_assignments_nested_definitions_and_shadowing_keep_existing_bindings() {
    let explicit = r#"
left = 10
right = 20
def pair(left, right) => [left, right]
def assign(left, right) => left += right
def operation number => 99
def apply(operation, number) => operation(number)
def outer(initial):
    def combine(left, right, scale) => (left + right) * scale
    return combine(initial, 2, 3)
assert(pair(3, 4), [3, 4])
assert(pair(3), [3, null])
assert(pair(), [null, null])
assert(pair(3, 4, 5), [3, 4])
assert(assign(3, 4), 7)
assert(left, 10)
assert(right, 20)
assert(outer(4), 18)
assert(apply(null, 3), null)
assert(apply(number => number * 2, 3), 6)
try:
    apply(7, 3)
catch error:
    assert(error, TypeError("operation is not a function"))
assert(operation(3), 99)
"#;
    let bare = explicit
        .replace("def pair(left, right)", "def pair left, right")
        .replace("def assign(left, right)", "def assign left, right")
        .replace(
            "def apply(operation, number)",
            "def apply operation, number",
        )
        .replace(
            "def combine(left, right, scale)",
            "def combine left, right, scale",
        );
    check_forms(&bare, explicit, 12);
}

#[test]
fn methods_initializers_this_self_and_super_keep_the_same_receiver_path() {
    let explicit = r#"
class Parent:
    def init(left, right) => this.amount = left + right
    def reading(offset, scale) => self.amount * scale + offset
class Child: Parent
    def init(left, right) => super(left, right)
    def reading(offset, scale) => self.amount * scale + offset
    def increase(delta, extra) => this.amount += delta + extra
child = Child(2, 3)
observed = child.reading(1, 2)
assert(child.id, "child")
assert(child.amount, 5)
assert(observed, 11)
assert(child.increase(2, 1), 8)
assert(child.amount, 8)
assert(observed, 17)
assert(Parent.length, 0)
assert(Child.length, 1)
"#;
    let bare = explicit
        .replace("def init(left, right)", "def init left, right")
        .replace("def reading(offset, scale)", "def reading offset, scale")
        .replace("def increase(delta, extra)", "def increase delta, extra");
    check_forms(&bare, explicit, 8);
}

#[test]
fn objects_lists_and_returned_anonymous_functions_keep_existing_noncapturing_scope() {
    let explicit = r#"
def coordinates(x, y) => {x, y}
def pair(left, right) => [left, right]
def factory(ignored, other) => (left, right) => left + right
point = coordinates(2, 3)
assert(point.x, 2)
assert(point.y, 3)
assert(pair("left", "right"), ["left", "right"])
callback = factory(10, 20)
assert(callback(2, 3), 5)
"#;
    let bare = explicit
        .replace("def coordinates(x, y)", "def coordinates x, y")
        .replace("def pair(left, right)", "def pair left, right")
        .replace("def factory(ignored, other)", "def factory ignored, other");
    check_forms(&bare, explicit, 4);

    for (explicit, message, line) in [
        (
            "def wrap(left, right) => { missing }\nwrap(1, 2)",
            "missing is not defined",
            2,
        ),
        (
            "def factory(left, right) => value => ({ left, right })\ncallback = factory(1, 2)\ncallback(3)",
            "left is not defined",
            3,
        ),
        (
            "def receiver(left, right) => this.amount\nreceiver(1, 2)",
            "this is not defined",
            2,
        ),
    ] {
        let bare = explicit.replace("(left, right) =>", " left, right =>");
        equivalent(&bare, explicit);
        let bare_error = Runtime::new().run(&bare).unwrap_err();
        assert_eq!(bare_error, Runtime::new().run(explicit).unwrap_err());
        assert_eq!(bare_error.kind(), ErrorKind::Reference);
        assert_eq!(bare_error.message(), message);
        assert_eq!(bare_error.position(), Some(Position::new(line, 1)));
    }
}

#[test]
fn live_references_snapshots_redefinition_and_dependency_graphs_remain_identical() {
    let explicit = r#"
bias = 2
def scaled(left, right) => left * right + bias
live = scaled
snapshot = scaled.value
result = live(3, 4)
frozen = snapshot(3, 4)
assert(result, 14)
assert(frozen, 14)
def scaled(left, right) => left + right + bias
assert(result, 9)
assert(frozen, 14)
bias = 4
assert(result, 11)
assert(frozen, 16)
"#;
    let bare = explicit.replace("def scaled(left, right)", "def scaled left, right");
    let [mut bare, mut explicit] = check_forms(&bare, explicit, 6);
    assert!(bare.graph.keys().eq(explicit.graph.keys()));
    for node in bare.graph.nodes() {
        let other = explicit.graph.retrieve(&node.key).unwrap();
        assert_eq!(node.kind, other.kind);
        assert_eq!(node.dependencies, other.dependencies);
        assert_eq!(node.dependents, other.dependents);
    }
    for source in ["why result", "model |> why"] {
        let actual = bare.run(source).unwrap();
        let expected = explicit.run(source).unwrap();
        assert_eq!(
            bare.serialize_json(&actual).unwrap(),
            explicit.serialize_json(&expected).unwrap()
        );
    }
    assert_eq!(
        NodeKey::block(&parse("def add left, right => left + right").unwrap(), None).as_str(),
        "block(def add)"
    );
}

#[test]
fn failed_redefinitions_and_invalid_headers_preserve_transactions_and_recovery() {
    equivalent(
        "def reading left, right:\n    return left",
        "def reading(left, right):\n    return left",
    );
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, definition) in runtimes.iter_mut().zip([
        "def reading left, right => source + left + right",
        "def reading(left, right) => source + left + right",
    ]) {
        runtime
            .run(&format!("source = 3\n{definition}\ntotal = reading(2, 4)"))
            .unwrap();
    }
    for (runtime, definition) in runtimes.iter_mut().zip([
        "def reading left, right => missing + left + right",
        "def reading(left, right) => missing + left + right",
    ]) {
        let error = runtime.run(definition).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.run("total").unwrap(), Value::Number(9.0));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
        for header in [
            "def reading left, right, => left",
            "def reading left, right: int, => left",
            "def reading left, right:\nreturn left",
            "def reading left,\nright => left",
        ] {
            let error = runtime.run(&format!("source = 99\n{header}")).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.position(), Some(Position::new(2, 13)));
            assert_eq!(runtime.run("source").unwrap(), Value::Number(3.0));
            assert_eq!(runtime.run("total").unwrap(), Value::Number(9.0));
        }
        runtime.run("source = 4").unwrap();
        assert_eq!(runtime.run("total").unwrap(), Value::Number(10.0));
        assert_eq!(runtime.run("reading(3, 5)").unwrap(), Value::Number(12.0));
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
fn parenthesized_single_typed_block_class_anonymous_and_call_grammars_remain_available() {
    for signature in [
        "()",
        "(left)",
        "(left: int)",
        "(left, right)",
        "(left: int, right)",
        "(left: int, right: str,)",
        "(\n    left,\n    right,\n)",
    ] {
        let arrow = format!("def f{signature} => 1");
        equivalent(&arrow, &format!("def f{signature}:\n    return 1"));
        assert!(parse(&format!("def f{signature}: pass")).is_ok());
        assert!(parse(&format!("def f{signature}: {{ return 1 }}")).is_ok());
        assert!(parse(&format!("class Item{signature}")).is_ok());
    }
    for (bare, explicit) in [
        ("def f left => left", "def f(left) => left"),
        ("def f left: int => left", "def f(left: int) => left"),
        (
            "def f left:\n    return left",
            "def f(left):\n    return left",
        ),
        (
            "def f left: int:\n    return left",
            "def f(left: int):\n    return left",
        ),
    ] {
        equivalent(bare, explicit);
    }
    for (source, block) in [
        ("left => left", false),
        ("(left, right) => left + right", false),
        ("(left: int, right) => { return left + right }", true),
        ("function(left, right) { return left + right }", true),
    ] {
        let Expr::Function(function) = parse_expression(source).unwrap() else {
            panic!("expected an anonymous function");
        };
        assert!(function.name.is_none());
        assert_eq!(matches!(function.body, FunctionBody::Block(_)), block);
    }
    assert!(parse("left, right => left + right").is_err());
    assert_eq!(
        parse("add left right").unwrap(),
        ["add", "left", "right"].map(|name| Stmt::Expression(Expr::Identifier(name.to_string())))
    );
    assert_eq!(
        parse_expression("add").unwrap(),
        Expr::Identifier("add".to_string())
    );
    assert_eq!(
        parse_expression("add(2, 3)").unwrap(),
        Expr::Call {
            callee: Box::new(Expr::Identifier("add".to_string())),
            arguments: vec![Expr::Number(2.0), Expr::Number(3.0)],
        }
    );
}

#[test]
fn large_flat_parameter_lists_keep_the_existing_uncapped_parameter_behavior() {
    for count in [2, 3, 129, 1024] {
        let names = (0..count)
            .map(|index| format!("p{index}"))
            .collect::<Vec<_>>();
        let parameters = names.join(", ");
        let bare = format!("def many {parameters} => p{}", count - 1);
        let explicit = format!("def many({parameters}) => p{}", count - 1);
        let statements = equivalent(&bare, &explicit);
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected one named function");
        };
        assert_eq!(function.parameters.len(), count);
        for (parameter, name) in function.parameters.iter().zip(names) {
            assert_eq!(parameter.name, name);
            assert_eq!(parameter.type_name, None);
        }
        let arguments = vec!["7"; count].join(", ");
        for source in [bare, explicit] {
            let mut runtime = Runtime::new();
            runtime.run(&source).unwrap();
            assert_eq!(
                runtime.run(&format!("many({arguments})")).unwrap(),
                Value::Number(7.0)
            );
        }
    }
}

#[test]
fn expression_nesting_and_chain_limits_still_fail_atomically_without_limiting_headers() {
    let mut runtime = Runtime::new();
    runtime.run("saved = 7").unwrap();
    for (body, message) in [
        (
            format!("{}left", "-".repeat(200)),
            "Expressions are nested too deeply",
        ),
        (
            format!("left{}", " + 1".repeat(200)),
            "Expression has too many operands",
        ),
        (
            format!("root{}", ".next".repeat(200)),
            "Expression has too many parts",
        ),
        (
            format!("{}left", "saved = ".repeat(200)),
            "Expressions are nested too deeply",
        ),
    ] {
        for header in ["def limited left, right =>", "def limited(left, right) =>"] {
            let error = runtime
                .run(&format!("saved = 3\n{header} {body}"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert!(error.position().is_some());
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        }
    }
    runtime.run("def add left, right => left + right").unwrap();
    assert_eq!(runtime.run("add(2, 3)").unwrap(), Value::Number(5.0));
}
