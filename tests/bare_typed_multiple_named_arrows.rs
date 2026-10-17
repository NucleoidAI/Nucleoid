//! Typed/mixed bare multiple-parameter named arrows preserve the explicit AST.
//! Only the two explicitly ignored corpus audits need historical Git objects.

mod common;
#[path = "support/named_arrow_headers.rs"]
mod named_arrow_headers;

use std::fs;
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
use regex::Regex;

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
    let mut returns = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([bare, explicit]) {
        let value = common::check_with_count(
            runtime,
            &common::Case {
                title: "typed multiple named-arrow pair".to_string(),
                source: source.to_string(),
                expected: expected.map(str::to_string),
            },
            assertions,
        )
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
        assert_eq!(runtime.assertions_run(), assertions, "{source}");
        returns.push(runtime.serialize_json(&value).unwrap());
    }
    assert_eq!(returns[0], returns[1]);
    runtimes
}

#[test]
fn every_annotation_position_preserves_complete_metadata_order_and_named_lowering() {
    for names in [vec!["left", "right"], vec!["left", "right", "factor"]] {
        for mask in 1..(1 << names.len()) {
            let parameters = names
                .iter()
                .enumerate()
                .map(|(index, name)| Parameter {
                    name: (*name).to_string(),
                    type_name: (mask & (1 << index) != 0).then(|| format!("Type{index}")),
                })
                .collect::<Vec<_>>();
            let header = parameters
                .iter()
                .map(|parameter| match &parameter.type_name {
                    Some(annotation) => format!("{}: {annotation}", parameter.name),
                    None => parameter.name.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            let bare = format!("def calculate {header} => 1");
            let statements = equivalent(&bare, &format!("def calculate({header}) => 1"));
            equivalent(&bare, &format!("def calculate({header}):\n    return 1"));
            let [Stmt::Function(function)] = statements.as_slice() else {
                panic!("expected one named function");
            };
            assert_eq!(function.name.as_deref(), Some("calculate"));
            assert_eq!(function.parameters, parameters);
            assert_eq!(
                function.body,
                FunctionBody::Block(vec![Stmt::Return(Some(Expr::Number(1.0)))])
            );
            assert_eq!(generate_all(&statements), "def calculate");
        }
    }
    let statements = equivalent(
        "def f type: value, _private, A1: ArbitraryType, why: model, affects => type",
        "def f(type: value, _private, A1: ArbitraryType, why: model, affects) => type",
    );
    let [Stmt::Function(function)] = statements.as_slice() else {
        panic!("expected one named function");
    };
    assert_eq!(
        function
            .parameters
            .iter()
            .map(|parameter| (parameter.name.as_str(), parameter.type_name.as_deref()))
            .collect::<Vec<_>>(),
        [
            ("type", Some("value")),
            ("_private", None),
            ("A1", Some("ArbitraryType")),
            ("why", Some("model")),
            ("affects", None),
        ]
    );
}

#[test]
fn expression_bodies_keep_precedence_objects_templates_and_anonymous_body_grammar() {
    for expression in [
        "left + right * factor",
        "left = right",
        "left += right",
        "left ||= right",
        "{ left, right }",
        "[left, right]",
        "`total=${left + right}`",
        "value => value * 2",
        "(left: int, right) => ({ left, right })",
        "(left, right: int) => { return left + right }",
        "function(value: int) { return value }",
        "why left",
    ] {
        let bare = format!("def calculate left: int, right, factor: number => {expression}");
        let statements = equivalent(
            &bare,
            &format!("def calculate(left: int, right, factor: number) => {expression}"),
        );
        equivalent(
            &bare,
            &format!("def calculate(left: int, right, factor: number):\n    return {expression}"),
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
fn immediate_lookahead_uses_existing_tokens_and_leaves_body_and_statement_boundaries() {
    assert_eq!(
        tokenize("def add left: int, right => left + right")
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        [
            Token::Keyword(Keyword::Def),
            Token::Identifier("add".to_string()),
            Token::Identifier("left".to_string()),
            Token::Colon,
            Token::Identifier("int".to_string()),
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
        "def add left:int,right=>left + right",
        "def add\tleft:\tint,\tright\t=> left + right",
        "def add left: int, right => (left + right)",
        "def add left: int, right => (\n    left +\n    right\n)",
        "def add left: int, right => (\r\n    left + right\r\n)",
    ] {
        equivalent(source, "def add(left: int, right) => left + right");
    }
    for suffix in ["\nnext = 3", "; next = 3", " # body\nnext = 3"] {
        equivalent(
            &format!("def add left: int, right => left + right{suffix}"),
            &format!("def add(left: int, right) => left + right{suffix}"),
        );
    }
    equivalent(
        "{ def add left: int, right => left + right; next = 3 }",
        "{ def add(left: int, right) => left + right; next = 3 }",
    );
    equivalent(
        "def outer(initial: int):\n    def combine left: int, right, scale: int => (left + right) * scale\n    return combine(initial, 2, 3)",
        "def outer(initial: int):\n    def combine(left: int, right, scale: int) => (left + right) * scale\n    return combine(initial, 2, 3)",
    );
    assert_eq!(
        parse_program("def add left: int, right => left + right\nnext = 3")
            .unwrap()
            .positions,
        [Position::new(1, 1), Position::new(2, 1)]
    );
}

#[test]
fn unsupported_prefixes_preserve_exact_parameter_errors_without_partial_consumption() {
    for header in [
        "left: int, right",
        "left: int, right, => left",
        "left, right: int, => left",
        "left: int, => left",
        "left: int,, right => left",
        "left: int, right other => left",
        "left:, right => left",
        "left: int, right: => left",
        "left: int, right: int, third: => left",
        "left: int, right = 1 => left",
        "left = 1, right: int => left",
        "left: int = 1, right => left",
        "left: int, [right] => left",
        "left: int, {right} => left",
        "left: int, ...right => left",
        "left: int, this => left",
        "left: int, self => left",
        "left: int, $Right => left",
        "left: int, 1 => left",
        "left: int, \"right\" => left",
        "left: int, right -> int => left",
        "left: int, right: int -> int => left",
        "left: int, right: int: int => left",
        "left: Int.Name, right => left",
        "left, right: List[int] => left",
        "left: List<int>, right => left",
        "left: $Type, right => left",
        "left, right: this => left",
        "left: null, right => left",
        "left: \"int\", right => left",
        "left: 1, right => left",
        "left: int, right\n=> left",
        "left: int,\nright => left",
        "left\n: int, right => left",
        "left:\nint, right => left",
        "left: int\n, right => left",
        "left: int, right:\nint => left",
        "left: int, right; => left",
        "left: int, right # boundary\n=> left",
        "left: int, # boundary\nright => left",
        "left: # annotation\nint, right => left",
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
            "def f this: int, right => right",
            "Expected '(' but found keyword 'this'",
            Position::new(1, 7),
        ),
        (
            "def f\nleft: int, right => left",
            "Expected '(' but found end of line",
            Position::new(1, 5),
        ),
        (
            "def f; left: int, right => left",
            "Expected '(' but found ';'",
            Position::new(1, 6),
        ),
        (
            "class Gauge:\n    def reading left: int, right, => left",
            "Expected '(' but found identifier 'left'",
            Position::new(2, 17),
        ),
        (
            "def f(left: int, right:) => left",
            "Expected a type but found ')'",
            Position::new(1, 26),
        ),
        (
            "def f left: int | str, right => left",
            "Unexpected character '|'",
            Position::new(1, 17),
        ),
        (
            "def f left: int?, right => left",
            "Unexpected character '?'",
            Position::new(1, 16),
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(position), "{source}");
    }
}

#[test]
fn typed_multiple_blocks_require_real_indent_and_all_earlier_guards_stay_exact() {
    for header in [
        "left: int, right",
        "left, right: int",
        "left: int, right: int",
        "left: int, right, factor: number",
        "left, right: int, factor",
    ] {
        // NUC30 adds only these real-indented block counterparts.
        for suffix in [":\n    return left", ":\n    # comment\n    return left"] {
            equivalent(
                &format!("def f {header}{suffix}"),
                &format!("def f({header}){suffix}"),
            );
        }
        for suffix in [
            ": return left",
            ": { return left }",
            " { return left }",
            ":;\n    return left",
        ] {
            let source = format!("def f {header}{suffix}");
            let error = Runtime::check(&source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            assert_eq!(
                error.message(),
                "Expected '(' but found identifier 'left'",
                "{source}"
            );
            assert_eq!(error.position(), Some(Position::new(1, 7)), "{source}");
        }
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
        (
            "def f left, right => left + right",
            "def f(left, right) => left + right",
        ),
        (
            "def f left, right:\n    return left + right",
            "def f(left, right):\n    return left + right",
        ),
        (
            "def f left:\n    int => left",
            "def f(left):\n    int => left",
        ),
        (
            "def f left, right:\n    int => left",
            "def f(left, right):\n    int => left",
        ),
    ] {
        equivalent(bare, explicit);
    }
    check_forms(
        "def f left, right:\n    int => left\nassert(f(2, 3), null)",
        "def f(left, right):\n    int => left\nassert(f(2, 3), null)",
        1,
        None,
    );
}

#[test]
fn qualifying_arrows_keep_exact_body_errors_and_lexical_positions() {
    for (body, message, line, column) in [
        ("", "Unexpected end of input", 1, 26),
        ("\n", "Unexpected end of line", 2, 1),
        (" (left +)", "Unexpected ')'", 1, 35),
        (
            " { return left }",
            "Expected a key but found keyword 'return'",
            1,
            36,
        ),
        (" { left = 1 }", "Expected ':' but found '='", 1, 34),
        (
            " {\n    left,\n}",
            "Expected a key but found an indented block",
            2,
            5,
        ),
        (" @", "Unexpected character '@'", 1, 27),
    ] {
        for (header, shift) in [
            ("def f left: int, right =>", 0),
            ("def f(left: int, right) =>", usize::from(line == 1)),
        ] {
            let source = format!("{header}{body}");
            let error = Runtime::check(&source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            assert_eq!(error.message(), message, "{source}");
            assert_eq!(
                error.position(),
                Some(Position::new(line, column + shift)),
                "{source}"
            );
        }
    }
}

#[test]
fn annotations_remain_metadata_with_accepted_arity_shadowing_null_and_callable_binding() {
    let explicit = r#"
left = 10
right = 20
def pair(left: int, right: int) => [left, right]
def assign(left: int, right) => left += right
def invoke(operation: Callback, amount: int) => operation(amount)
def increment(amount: int) => amount + 1
def operation(amount) => amount + 100
def outer(initial: int):
    def combine(left: int, right, scale: int) => (left + right) * scale
    return combine(initial, 2, 3)
assert(pair(3, 4), [3, 4])
assert(pair(3), [3, null])
assert(pair(), [null, null])
assert(pair(3, 4, 5), [3, 4])
assert(pair("three", true), ["three", true])
assert(assign(3, 4), 7)
assert(left, 10)
assert(right, 20)
assert(invoke(increment, 4), 5)
assert(invoke(null, 3), null)
try:
    invoke(7, 3)
catch error:
    assert(error, TypeError("operation is not a function"))
assert(outer(4), 18)
assert(operation(3), 103)
pair(8, 9)
"#;
    let bare = explicit
        .replace(
            "def pair(left: int, right: int)",
            "def pair left: int, right: int",
        )
        .replace(
            "def assign(left: int, right)",
            "def assign left: int, right",
        )
        .replace(
            "def invoke(operation: Callback, amount: int)",
            "def invoke operation: Callback, amount: int",
        )
        .replace(
            "def combine(left: int, right, scale: int)",
            "def combine left: int, right, scale: int",
        );
    for runtime in check_forms(&bare, explicit, 13, Some("[8,9]")) {
        for annotation in ["int", "Callback"] {
            assert!(!runtime.graph.contains(&NodeKey::variable(annotation)));
        }
    }
}

#[test]
fn typed_and_mixed_methods_initializers_and_receivers_keep_explicit_and_bare_super() {
    let explicit = r#"
class Parent:
    def init(left: int, right: int) => this.amount = left + right
    def reading(offset: int, scale: int) => self.amount * scale + offset
class Child: Parent
    def init(left: int, right) => super(left, right)
    def reading(offset, scale: int) => self.amount * scale + offset
    def increase(delta: int, extra) => this.amount += delta + extra
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
        .replace(
            "def init(left: int, right: int)",
            "def init left: int, right: int",
        )
        .replace("def init(left: int, right)", "def init left: int, right")
        .replace(
            "def reading(offset: int, scale: int)",
            "def reading offset: int, scale: int",
        )
        .replace(
            "def reading(offset, scale: int)",
            "def reading offset, scale: int",
        )
        .replace(
            "def increase(delta: int, extra)",
            "def increase delta: int, extra",
        );
    check_forms(&bare, explicit, 8, None);
    check_forms(
        "class Parent:\n    def init left: int, right => this.amount = left\nclass Child: Parent\n    def init left: int, right: int => super\nchild = Child(2, 3)\nassert(child.amount, null)\nassert(Child.length, 1)",
        "class Parent:\n    def init(left: int, right) => this.amount = left\nclass Child: Parent\n    def init(left: int, right: int) => super\nchild = Child(2, 3)\nassert(child.amount, null)\nassert(Child.length, 1)",
        2,
        None,
    );
}

#[test]
fn objects_templates_and_returned_functions_preserve_existing_noncapturing_errors() {
    check_forms(
        "def point x: int, y => {x, y}\ndef text left, right: str => `${left},${right}`\ndef factory ignored: Unknown, other => (left: int, right) => left + right\np = point(2, 3)\nassert(p.x, 2)\nassert(p.y, 3)\nassert(text(2, 3), \"2,3\")\ncallback = factory(10, 20)\nassert(callback(2, 3), 5)",
        "def point(x: int, y) => {x, y}\ndef text(left, right: str) => `${left},${right}`\ndef factory(ignored: Unknown, other) => (left: int, right) => left + right\np = point(2, 3)\nassert(p.x, 2)\nassert(p.y, 3)\nassert(text(2, 3), \"2,3\")\ncallback = factory(10, 20)\nassert(callback(2, 3), 5)",
        4,
        None,
    );
    for (explicit, message, line) in [
        (
            "def wrap(left: int, right) => { missing }\nwrap(1, 2)",
            "missing is not defined",
            2,
        ),
        (
            "def factory(left: int, right) => value => ({ left, right })\ncallback = factory(1, 2)\ncallback(3)",
            "left is not defined",
            3,
        ),
        (
            "def receiver(left: int, right) => this.amount\nreceiver(1, 2)",
            "this is not defined",
            2,
        ),
        (
            "def receiver(left: int, right) => self.amount\nreceiver(1, 2)",
            "this is not defined",
            2,
        ),
    ] {
        let bare = explicit.replace("(left: int, right) =>", " left: int, right =>");
        equivalent(&bare, explicit);
        let error = Runtime::new().run(&bare).unwrap_err();
        assert_eq!(error, Runtime::new().run(explicit).unwrap_err());
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(line, 1)));
    }
}

#[test]
fn live_aliases_definition_snapshots_redefinition_and_graphs_remain_identical() {
    let explicit = r#"
bias = 2
def scaled(left: int, right: int) => left * right + bias
live = scaled
snapshot = scaled.value
result = live(3, 4)
frozen = snapshot(3, 4)
assert(result, 14)
assert(frozen, 14)
def scaled(left: int, right: int) => left + right + bias
assert(result, 9)
assert(frozen, 14)
bias = 4
assert(result, 11)
assert(frozen, 16)
result
"#;
    let bare = explicit.replace(
        "def scaled(left: int, right: int)",
        "def scaled left: int, right: int",
    );
    let [mut bare, mut explicit] = check_forms(&bare, explicit, 6, Some("11"));
    assert!(bare.graph.keys().eq(explicit.graph.keys()));
    for node in bare.graph.nodes() {
        let other = explicit.graph.retrieve(&node.key).unwrap();
        assert_eq!(node.kind, other.kind);
        assert_eq!(node.dependencies, other.dependencies);
        assert_eq!(node.dependents, other.dependents);
        assert!(
            bare.graph
                .shape_dependencies(&node.key)
                .eq(explicit.graph.shape_dependencies(&node.key))
        );
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
        NodeKey::block(
            &parse("def add left: int, right: int => left + right").unwrap(),
            None
        )
        .as_str(),
        "block(def add)"
    );
}

#[test]
fn caught_failed_redefinition_preserves_relationship_rollback_and_explicit_return() {
    let explicit = r#"
source = 3
def reading(left: int, right: int) => source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading(left: int, right: int) => missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)
"#;
    let bare = explicit.replace(
        "def reading(left: int, right: int)",
        "def reading left: int, right: int",
    );
    for runtime in check_forms(&bare, explicit, 5, Some("21")) {
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
}

#[test]
fn invalid_prefixes_and_failed_redefinitions_are_atomic_and_allow_recovery() {
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, header) in runtimes.iter_mut().zip([
        "def reading left: int, right =>",
        "def reading(left: int, right) =>",
    ]) {
        runtime
            .run(&format!(
                "source = 3\n{header} source + left + right\ntotal = reading(2, 4)"
            ))
            .unwrap();
        let error = runtime
            .run(&format!("{header} missing + left + right"))
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.run("total").unwrap(), Value::Number(9.0));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
        for invalid in [
            "def reading left: int, right, => left",
            "def reading left, right: int,:\n    return left",
            "def reading left: int, right,:\n    return left",
            "def reading left: int, right: return left",
            "def reading left: int, right: { return left }",
            "def reading left: int,\nright => left",
        ] {
            let error = runtime.run(&format!("source = 99\n{invalid}")).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), "Expected '(' but found identifier 'left'");
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
        let other = runtimes[1].graph.retrieve(&node.key).unwrap();
        assert_eq!(node.dependencies, other.dependencies);
        assert_eq!(node.dependents, other.dependents);
    }
}

#[test]
fn parenthesized_class_anonymous_and_call_grammars_remain_available() {
    for signature in [
        "()",
        "(left)",
        "(left: int)",
        "(left, right)",
        "(left: int, right)",
        "(left, right: str,)",
        "(left: int, right: str,)",
        "(\n    left: int,\n    right,\n)",
    ] {
        let statements = equivalent(
            &format!("def f{signature} => 1"),
            &format!("def f{signature}:\n    return 1"),
        );
        assert!(parse(&format!("def f{signature}: pass")).is_ok());
        assert!(parse(&format!("def f{signature}: {{ return 1 }}")).is_ok());
        let class = parse(&format!("class Item{signature}")).unwrap();
        let [Stmt::Class(class)] = class.as_slice() else {
            panic!("expected an unchanged class");
        };
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected a named function");
        };
        assert_eq!(class.parameters, function.parameters);
    }
    for source in [
        "left => left",
        "(left: int, right) => left + right",
        "(left, right: str) => { return left + right }",
        "function(left: int, right) { return left + right }",
    ] {
        let Expr::Function(function) = parse_expression(source).unwrap() else {
            panic!("expected an unchanged anonymous function");
        };
        assert!(function.name.is_none());
    }
    for source in [
        "left: int, right => left + right",
        "callback = left: int, right => left + right",
        "class Item left: int, right",
    ] {
        assert!(Runtime::check(source).is_err(), "{source}");
    }
    assert_eq!(
        parse("add left right").unwrap(),
        ["add", "left", "right"].map(|name| Stmt::Expression(Expr::Identifier(name.to_string())))
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
fn flat_all_typed_and_mixed_lists_remain_uncapped_through_1024_parameters() {
    for count in [2, 3, 129, 1024] {
        for all_typed in [false, true] {
            let parameters = (0..count)
                .map(|index| {
                    if all_typed || index % 3 == 0 {
                        format!("p{index}: Unknown{index}")
                    } else {
                        format!("p{index}")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let bare = format!("def many {parameters} => p{}", count - 1);
            let explicit = format!("def many({parameters}) => p{}", count - 1);
            let statements = equivalent(&bare, &explicit);
            let [Stmt::Function(function)] = statements.as_slice() else {
                panic!("expected one named function");
            };
            assert_eq!(function.parameters.len(), count);
            for (index, parameter) in function.parameters.iter().enumerate() {
                assert_eq!(parameter.name, format!("p{index}"));
                assert_eq!(
                    parameter.type_name,
                    (all_typed || index % 3 == 0).then(|| format!("Unknown{index}"))
                );
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
}

#[test]
fn inherited_expression_nesting_and_chain_bounds_fail_atomically_then_recover() {
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
        for header in [
            "def limited left: int, right =>",
            "def limited(left: int, right) =>",
        ] {
            let error = runtime
                .run(&format!("saved = 3\n{header} {body}"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert!(error.position().is_some());
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        }
    }
    runtime
        .run("def add left: int, right: int => left + right")
        .unwrap();
    assert_eq!(runtime.run("add(2, 3)").unwrap(), Value::Number(5.0));
}

#[test]
fn native_repl_keeps_canonical_graphs_and_recovers_from_invalid_typed_prefixes() {
    let explicit = ".json on\n.graph on\nbias = 2\ndef scaled(left: int, right: int) => left * right + bias\nresult = scaled(3, 4)\nbias = 4\nassert(result, 16)\nresult\n.exit\n";
    let bare = explicit.replace(
        "def scaled(left: int, right: int)",
        "def scaled left: int, right: int",
    );
    let run = |source: &str| {
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
    };
    let outputs = [run(&bare), run(explicit)];
    for output in &outputs {
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.lines().any(|line| line == "16.0"), "{stdout}");
        assert!(stdout.contains("scaled [function]"), "{stdout}");
        assert!(!stdout.contains("def scaled left"), "{stdout}");
    }
    assert_eq!(outputs[0].stdout, outputs[1].stdout);
    let source = bare.replace("bias = 4", "def scaled left: int, right, => left\nbias = 4");
    let output = run(&source);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        "SyntaxError: Expected '(' but found identifier 'left' (line 1, column 12)"
    );
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .any(|line| line == "16.0")
    );
}

/// Select only this proposal's protected prefix, never later appended cases.
fn protected_corpus_slices(
    cases: &[common::Case],
    historical_count: usize,
    added_count: usize,
) -> Option<(&[common::Case], &[common::Case])> {
    let prefix = cases.get(..historical_count + added_count)?;
    Some(prefix.split_at(historical_count))
}

#[test]
fn historical_corpus_selection_allows_later_tails_but_requires_all_protected_cases() {
    let cases = [
        "historical one",
        "historical two",
        "own addition one",
        "own addition two",
        "later one",
        "later two",
    ]
    .map(|title| common::Case {
        title: title.to_string(),
        source: String::new(),
        expected: None,
    });
    for length in 4..=cases.len() {
        let (historical, additions) = protected_corpus_slices(&cases[..length], 2, 2).unwrap();
        assert!(
            historical
                .iter()
                .map(|case| case.title.as_str())
                .eq(["historical one", "historical two"])
        );
        assert!(
            additions
                .iter()
                .map(|case| case.title.as_str())
                .eq(["own addition one", "own addition two"])
        );
    }
    for length in 0..4 {
        assert!(
            protected_corpus_slices(&cases[..length], 2, 2).is_none(),
            "must reject a protected prefix truncated to {length} cases"
        );
    }
}

const ACCEPTED_REVISION: &str = "c40b64bc6338b84ce3d76019c6e0ee1e13900da1";

fn accepted_source(path: &str) -> String {
    let output = Command::new("git")
        .args(["--no-pager", "show", &format!("{ACCEPTED_REVISION}:{path}")])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("requires Git and the pinned accepted objects; see the ignore run command");
    assert!(
        output.status.success(),
        "{path}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("accepted sources are UTF-8")
}

#[test]
#[ignore = "requires Git history at c40b64b; run cargo test --test bare_typed_multiple_named_arrows -- --ignored"]
fn historical_measured_asts_and_the_eight_appended_counterparts_remain_exact() {
    let header = Regex::new(
        r"(?m)^([ \t]*def [A-Za-z_]\w*) ([A-Za-z_]\w*(?:: [A-Za-z_]\w*)?(?:, [A-Za-z_]\w*(?:: [A-Za-z_]\w*)?)+) =>",
    )
    .unwrap();
    let mut historical = 0;
    let mut additions = 0;
    let mut assertions = 0;
    for (path, current, case_file, old_count, added) in [
        (
            "nucleoid.spec.md",
            include_str!("../nucleoid.spec.md"),
            true,
            318,
            5,
        ),
        (
            "tests/reference.md",
            include_str!("reference.md"),
            true,
            64,
            1,
        ),
        ("README.md", include_str!("../README.md"), false, 15, 1),
        (
            "docs/examples.md",
            include_str!("../docs/examples.md"),
            false,
            70,
            1,
        ),
    ] {
        let accepted = accepted_source(path);
        let (before, after) = if case_file {
            (common::cases(&accepted), common::cases(current))
        } else {
            (
                common::snippets(&accepted, path),
                common::snippets(current, path),
            )
        };
        assert_eq!(before.len(), old_count, "{path}");
        let (current_historical, own_additions) = protected_corpus_slices(&after, old_count, added)
            .unwrap_or_else(|| {
                panic!(
                    "{path}: expected at least {} protected cases, found {}",
                    old_count + added,
                    after.len()
                )
            });
        for (before, after) in before.iter().zip(current_historical) {
            assert_eq!(before.title, after.title, "{path}");
            assert_eq!(before.expected, after.expected, "{}", before.title);
            let statements = equivalent(&after.source, &before.source);
            assertions += common::assertions::protected_assertions(&statements).len();
            historical += 1;
        }
        for case in own_additions {
            let normalized = named_arrow_headers::legacy_block_headers(
                &named_arrow_headers::legacy_headers(&case.source),
            );
            let explicit = header.replace_all(&normalized, "$1($2) =>");
            assert_ne!(normalized, explicit, "{}", case.title);
            let statements = equivalent(&case.source, &explicit);
            let count = common::assertions::protected_assertions(&statements).len();
            check_forms(&case.source, &explicit, count, case.expected.as_deref());
            assertions += count;
            additions += 1;
        }
    }
    assert_eq!(historical, 467);
    assert_eq!(additions, 8);
    assert_eq!(assertions, 1796);
}

#[test]
#[ignore = "requires Git history at c40b64b; run cargo test --test bare_typed_multiple_named_arrows -- --ignored"]
fn all_synthesized_records_preserve_complete_asts_fields_and_order() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut records = 0;
    let mut sets = 0;
    // Audit the 21 shards in this pinned revision, not later independent
    // shards. Explicit enumeration also fails if a historical shard is deleted.
    for split in 1..=21 {
        let filename = format!("nucleoid.synth.{split:02}.jsonl");
        let accepted = accepted_source(&format!("dataset/{filename}"));
        let current = fs::read_to_string(root.join("dataset").join(&filename)).unwrap();
        let before = accepted.lines().collect::<Vec<_>>();
        let after = current.lines().collect::<Vec<_>>();
        assert_eq!(before.len(), after.len(), "{filename}");
        for (before, after) in before.into_iter().zip(after) {
            let mut before: serde_json::Value = serde_json::from_str(before).unwrap();
            let mut after: serde_json::Value = serde_json::from_str(after).unwrap();
            let before_code = before.as_object_mut().unwrap().remove("code").unwrap();
            let after_code = after.as_object_mut().unwrap().remove("code").unwrap();
            assert_eq!(before, after, "{filename}");
            equivalent(after_code.as_str().unwrap(), before_code.as_str().unwrap());
            records += 1;
        }
        sets += 1;
    }
    assert_eq!(sets, 21);
    assert_eq!(records, 1081);
}
