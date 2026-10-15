//! Bare multi-parameter indented headers preserve the complete named block.

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Expr, FunctionBody, Parameter, Stmt, parse, parse_program};
use nucleoid::{ErrorKind, Position, Runtime, Value};
use regex::Regex;
use serde::Deserialize;

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

fn check_forms(bare: &str, explicit: &str, assertions: usize) -> [Runtime; 2] {
    equivalent(bare, explicit);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let mut returns = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([bare, explicit]) {
        let value = runtime
            .run(source)
            .unwrap_or_else(|error| panic!("{source}\n{error}"));
        returns.push(runtime.serialize_json(&value).unwrap());
        assert_eq!(runtime.assertions_run(), assertions, "{source}");
        let failures = runtime.take_assertions();
        assert!(failures.is_empty(), "{source}\n{failures:?}");
    }
    assert_eq!(returns[0], returns[1]);
    runtimes
}

#[test]
fn complete_lists_preserve_order_untyped_metadata_body_and_canonical_rendering() {
    for names in [
        vec!["left", "right"],
        vec!["left", "right", "factor"],
        vec!["type", "value", "_private", "A1", "why", "affects", "model"],
    ] {
        let parameters = names.join(", ");
        let statements = equivalent(
            &format!("def calculate {parameters}:\n    return 1"),
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
                })
                .collect::<Vec<_>>()
        );
        assert_eq!(
            function.body,
            FunctionBody::Block(vec![Stmt::Return(Some(Expr::Number(1.0)))])
        );
        assert_eq!(generate_all(&statements), "def calculate");
    }
}

#[test]
fn non_consuming_lookahead_leaves_normal_layout_for_the_existing_suite() {
    assert_eq!(
        tokenize("def add left, right:\n    return left + right")
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
            Token::Colon,
            Token::Newline,
            Token::Indent,
            Token::Keyword(Keyword::Return),
            Token::Identifier("left".to_string()),
            Token::Plus,
            Token::Identifier("right".to_string()),
            Token::Dedent,
            Token::Eof,
        ]
    );
    for source in [
        "def add left,right:\n    return left + right",
        "def add\tleft,\tright:\n\treturn left + right",
        "def add left, right:\r\n    return left + right",
        "def add left, right: # suite\n\n# blank\n    # body\n    return left + right",
        "def add left, right:\n;\n    return left + right",
        "def add left, right:\n    ;\n    return left + right",
    ] {
        equivalent(source, "def add(left, right):\n    return left + right");
    }
    for suffix in ["\nnext = 3", "\n# boundary\nnext = 3", "\n; next = 3"] {
        equivalent(
            &format!("def add left, right:\n    return left + right{suffix}"),
            &format!("def add(left, right):\n    return left + right{suffix}"),
        );
    }
    assert_eq!(
        parse_program("def add left, right:\n    return left + right\nnext = 3")
            .unwrap()
            .positions,
        [Position::new(1, 1), Position::new(3, 1)]
    );
}

#[test]
fn incomplete_typed_inline_braced_and_separated_headers_keep_exact_errors() {
    for header in [
        "left, right",
        "left, right:",
        "left, right:\n",
        "left, right:\n# no body",
        "left, right:\n    # no body",
        "left, right:\nreturn left",
        "left, right:\n;\nreturn left",
        "left, right:;\n    return left",
        "left, right: return left",
        "left, right: { return left }",
        "left, right:\n{ return left }",
        "left, right { return left }",
        "left, right,:\n    return left",
        "left,:\n    return left",
        "left,, right:\n    return left",
        "left, right other:\n    return left",
        "left: int, right,:\n    return left",
        "left, right: int,:\n    return left",
        "left: int, right: int,:\n    return left",
        "left = 1, right:\n    return left",
        "left, right = 1:\n    return left",
        "left, [right]:\n    return left",
        "left, {right}:\n    return left",
        "left, ...right:\n    return left",
        "left, this:\n    return left",
        "left, self:\n    return left",
        "left, $Right:\n    return left",
        "left, 1:\n    return left",
        "left, \"right\":\n    return left",
        "left, right -> int:\n    return left",
        "left, right\n:\n    return left",
        "left,\nright:\n    return left",
        "left\n, right:\n    return left",
        "left, right;:\n    return left",
        "left, right # boundary\n:\n    return left",
        "left, # boundary\nright:\n    return left",
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
            "def f this, right:\n    return right",
            "Expected '(' but found keyword 'this'",
            Position::new(1, 7),
        ),
        (
            "def f\nleft, right:\n    return left",
            "Expected '(' but found end of line",
            Position::new(1, 5),
        ),
        (
            "def f; left, right:\n    return left",
            "Expected '(' but found ';'",
            Position::new(1, 6),
        ),
        (
            "def f(left, right:):\n    return left",
            "Expected a type but found ')'",
            Position::new(1, 20),
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(position), "{source}");
    }
}

#[test]
fn qualifying_blocks_keep_existing_body_and_lexical_diagnostics_and_positions() {
    for (body, message, position) in [
        ("return left +)", "Unexpected ')'", Position::new(2, 19)),
        ("return @", "Unexpected character '@'", Position::new(2, 12)),
        (
            "return { left = 1 }",
            "Expected ':' but found '='",
            Position::new(2, 19),
        ),
    ] {
        for header in ["def f left, right:", "def f(left, right):"] {
            let source = format!("{header}\n    {body}");
            let error = Runtime::check(&source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            assert_eq!(error.message(), message, "{source}");
            assert_eq!(error.position(), Some(position), "{source}");
        }
    }
}

#[test]
fn nested_multistatement_and_branching_bodies_keep_explicit_return_behavior() {
    let explicit = r#"
def add(left, right):
    total = left + right
    return total
def scaled(left, right, factor):
    return (left + right) * factor
def largest(left, right):
    if left > right:
        return left
    return right
def outer(initial):
    def combine(left, right, scale):
        return (left + right) * scale
    return combine(initial, 2, 3)
def noReturn(left, right):
    left + right
def bareReturn(left, right):
    return
assert(add(2, 3), 5)
assert(scaled(2, 3, 4), 20)
assert(largest(7, 3), 7)
assert(largest(2, 9), 9)
assert(outer(4), 18)
assert(noReturn(2, 3), null)
assert(bareReturn(2, 3), null)
"#;
    let bare = explicit
        .replace("def add(left, right)", "def add left, right")
        .replace(
            "def scaled(left, right, factor)",
            "def scaled left, right, factor",
        )
        .replace("def largest(left, right)", "def largest left, right")
        .replace(
            "def combine(left, right, scale)",
            "def combine left, right, scale",
        )
        .replace("def noReturn(left, right)", "def noReturn left, right")
        .replace("def bareReturn(left, right)", "def bareReturn left, right");
    check_forms(&bare, explicit, 7);

    // An indented suite may contain existing inline and braced inner blocks.
    for body in [
        "if left: return right\n    return left",
        "if left: { return right }\n    return left",
        "{ return left + right }",
        "return (\n        left +\n        right\n    )",
    ] {
        equivalent(
            &format!("def choose left, right:\n    {body}"),
            &format!("def choose(left, right):\n    {body}"),
        );
    }
}

#[test]
fn arity_parameter_assignments_and_non_callable_shadowing_keep_existing_scope() {
    let explicit = r#"
left = 10
right = 20
def pair(left, right):
    return [left, right]
def add(left, right):
    left += right
    return left
def operation number => 99
def apply(operation, number):
    return operation(number)
assert(pair(3, 4), [3, 4])
assert(pair(3), [3, null])
assert(pair(), [null, null])
assert(pair(3, 4, 5), [3, 4])
assert(add(3, 4), 7)
assert(left, 10)
assert(right, 20)
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
        .replace("def add(left, right)", "def add left, right")
        .replace(
            "def apply(operation, number)",
            "def apply operation, number",
        );
    check_forms(&bare, explicit, 11);
}

#[test]
fn methods_initializers_and_explicit_or_bare_super_keep_receiver_and_adoption() {
    let explicit = r#"
class Parent:
    def init(left, right):
        this.amount = left + right
    def reading(offset, scale):
        return self.amount * scale + offset
class Child: Parent
    def init(left, right):
        super(left, right)
    def reading(offset, scale):
        return self.amount * scale + offset
    def increase(delta, extra):
        this.amount += delta + extra
        return this.amount
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
class Flag:
    def init():
        this.ready = true
class Tagged: Flag
    def init(left, right):
        super
        self.amount = left + right
tagged = Tagged(3, 5)
assert(tagged.ready, true)
assert(tagged.amount, 8)
"#;
    let bare = explicit
        .replace("def init(left, right)", "def init left, right")
        .replace("def reading(offset, scale)", "def reading offset, scale")
        .replace("def increase(delta, extra)", "def increase delta, extra");
    check_forms(&bare, explicit, 10);
}

#[test]
fn objects_lists_templates_and_returned_anonymous_functions_do_not_gain_capture() {
    let explicit = r#"
def coordinates(x, y):
    return {x, y}
def pair(left, right):
    return [left, right]
def label(left, right):
    return `${left}:${right}`
def factory(ignored, other):
    return (left, right) => left + right
point = coordinates(2, 3)
assert(point.x, 2)
assert(point.y, 3)
assert(pair("left", "right"), ["left", "right"])
assert(label("a", "b"), "a:b")
callback = factory(10, 20)
assert(callback(2, 3), 5)
"#;
    let bare = explicit
        .replace("def coordinates(x, y)", "def coordinates x, y")
        .replace("def pair(left, right)", "def pair left, right")
        .replace("def label(left, right)", "def label left, right")
        .replace("def factory(ignored, other)", "def factory ignored, other");
    check_forms(&bare, explicit, 5);

    for (explicit, message, line) in [
        (
            "def wrap(left, right):\n    return { missing }\nwrap(1, 2)",
            "missing is not defined",
            3,
        ),
        (
            "def factory(left, right):\n    return value => ({ left, right })\ncallback = factory(1, 2)\ncallback(3)",
            "left is not defined",
            4,
        ),
        (
            "def receiver(left, right):\n    return this.amount\nreceiver(1, 2)",
            "this is not defined",
            3,
        ),
    ] {
        let bare = explicit.replace("(left, right):", " left, right:");
        equivalent(&bare, explicit);
        let error = Runtime::new().run(&bare).unwrap_err();
        assert_eq!(error, Runtime::new().run(explicit).unwrap_err());
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(line, 1)));
    }
}

#[test]
fn live_references_definition_snapshots_redefinition_and_graphs_remain_identical() {
    let explicit = r#"
bias = 2
def scaled(left, right):
    return left * right + bias
live = scaled
snapshot = scaled.value
result = live(3, 4)
frozen = snapshot(3, 4)
assert(result, 14)
assert(frozen, 14)
def scaled(left, right):
    return left + right + bias
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
        NodeKey::block(
            &parse("def add left, right:\n    return left + right").unwrap(),
            None
        )
        .as_str(),
        "block(def add)"
    );
}

#[test]
fn caught_redefinition_failures_execute_the_assertion_and_preserve_final_return() {
    let explicit = r#"
source = 3
def reading(left, right):
    return source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading(left, right):
        return missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)
"#;
    let bare = explicit.replace("def reading(left, right)", "def reading left, right");
    let runtimes = check_forms(&bare, explicit, 5);
    for mut runtime in runtimes {
        assert_eq!(runtime.run("reading(8, 9)").unwrap(), Value::Number(21.0));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
}

#[test]
fn invalid_headers_and_failed_redefinitions_leave_state_and_edges_recoverable() {
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, header) in runtimes
        .iter_mut()
        .zip(["def reading left, right:", "def reading(left, right):"])
    {
        runtime
            .run(&format!(
                "source = 3\n{header}\n    return source + left + right\ntotal = reading(2, 4)"
            ))
            .unwrap();
        let error = runtime
            .run(&format!("{header}\n    return missing + left + right"))
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.run("total").unwrap(), Value::Number(9.0));
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
        for invalid in [
            "def reading left, right,:\n    return left",
            "def reading left, right: int,:\n    return left",
            "def reading left, right:\nreturn left",
            "def reading left, right:;\n    return left",
            "def reading left,\nright:\n    return left",
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
fn legacy_parenthesized_single_typed_arrow_class_anonymous_and_call_forms_stay_valid() {
    for signature in [
        "()",
        "(left)",
        "(left: int)",
        "(left, right)",
        "(left: int, right)",
        "(left: int, right: str,)",
        "(\n    left,\n    right,\n)",
    ] {
        let block = format!("def f{signature}:\n    return 1");
        equivalent(&block, &format!("def f{signature} => 1"));
        assert!(parse(&format!("def f{signature}: pass")).is_ok());
        assert!(parse(&format!("def f{signature}: {{ return 1 }}")).is_ok());
        assert!(parse(&format!("class Item{signature}")).is_ok());
    }
    for (bare, explicit) in [
        ("def f left => left", "def f(left) => left"),
        ("def f left: int => left", "def f(left: int) => left"),
        (
            "def f left, right => left + right",
            "def f(left, right) => left + right",
        ),
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
    assert!(parse("left, right:\n    return left").is_err());
    assert_eq!(
        parse("add left right").unwrap(),
        ["add", "left", "right"].map(|name| Stmt::Expression(Expr::Identifier(name.to_string())))
    );
    for source in [
        "left => left",
        "(left, right) => left + right",
        "(left: int, right) => { return left + right }",
        "function(left, right) { return left + right }",
    ] {
        let statements = parse(source).unwrap();
        let [Stmt::Expression(Expr::Function(function))] = statements.as_slice() else {
            panic!("expected an anonymous function");
        };
        assert!(function.name.is_none());
    }
}

#[test]
fn flat_lists_remain_uncapped_past_expression_chain_limits() {
    for count in [2, 3, 129, 1024] {
        let names = (0..count)
            .map(|index| format!("p{index}"))
            .collect::<Vec<_>>();
        let parameters = names.join(", ");
        let bare = format!("def many {parameters}:\n    return p{}", count - 1);
        let explicit = format!("def many({parameters}):\n    return p{}", count - 1);
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
fn existing_body_nesting_and_chain_limits_fail_atomically_then_recover() {
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
        for header in ["def limited left, right:", "def limited(left, right):"] {
            let error = runtime
                .run(&format!("saved = 3\n{header}\n    return {body}"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert!(error.position().is_some());
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        }
    }
    runtime
        .run("def add left, right:\n    return left + right")
        .unwrap();
    assert_eq!(runtime.run("add(2, 3)").unwrap(), Value::Number(5.0));
}

fn accepted_source(path: &str) -> String {
    let output = Command::new("git")
        .args([
            "--no-pager",
            "show",
            &format!("49bc92a9412dbbdc2d1989c34b21d470f1e49e4c:{path}"),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git can read the pinned accepted sources");
    assert!(
        output.status.success(),
        "{path}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("accepted sources are UTF-8")
}

#[test]
#[ignore = "requires Git history at 49bc92a; run cargo test --test bare_multiple_indented_blocks -- --ignored"]
fn all_historical_measured_asts_and_the_eight_appended_counterparts_remain_exact() {
    let header =
        Regex::new(r"(?m)^([ \t]*def [A-Za-z_]\w*) ([A-Za-z_]\w*(?:, [A-Za-z_]\w*)+):$").unwrap();
    let mut historical = 0;
    let mut additions = 0;
    for (path, current, case_file, added) in [
        (
            "nucleoid.spec.md",
            include_str!("../nucleoid.spec.md"),
            true,
            5,
        ),
        ("tests/reference.md", include_str!("reference.md"), true, 1),
        ("README.md", include_str!("../README.md"), false, 1),
        (
            "docs/examples.md",
            include_str!("../docs/examples.md"),
            false,
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
        assert!(after.len() >= before.len() + added, "{path}");
        for (before, after) in before.iter().zip(&after) {
            assert_eq!(before.title, after.title, "{path}");
            assert_eq!(before.expected, after.expected, "{}", before.title);
            equivalent(&after.source, &before.source);
            historical += 1;
        }
        for case in &after[before.len()..before.len() + added] {
            let explicit = header.replace_all(&case.source, "$1($2):");
            assert_ne!(case.source, explicit, "{}", case.title);
            let statements = equivalent(&case.source, &explicit);
            let assertions = common::assertions::protected_assertions(&statements);
            common::check_with_assertions(case, assertions.len()).unwrap();
            additions += 1;
        }
    }
    assert_eq!(historical, 459);
    assert_eq!(additions, 8);
}

#[derive(Deserialize)]
struct TrainingRecord {
    id: String,
    description: String,
    code: String,
    #[serde(rename = "return")]
    returns: Option<String>,
}

#[test]
#[ignore = "requires Git history at 49bc92a; run cargo test --test bare_multiple_indented_blocks -- --ignored"]
fn synchronized_synthesized_records_keep_complete_asts_and_protected_fields() {
    for split in [
        "01", "02", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15", "16", "17", "18",
        "19", "20", "21",
    ] {
        let path = format!("dataset/nucleoid.synth.{split}.jsonl");
        let accepted = accepted_source(&path);
        let current =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(&path)).unwrap();
        let records = |text: &str| {
            text.lines()
                .map(|line| serde_json::from_str::<TrainingRecord>(line).unwrap())
                .collect::<Vec<_>>()
        };
        let before = records(&accepted);
        let after = records(&current);
        assert!(after.len() >= before.len(), "{path}");
        for (before, after) in before.iter().zip(&after) {
            assert_eq!(before.id, after.id, "{path}");
            assert_eq!(before.description, after.description, "{}", before.id);
            assert_eq!(before.returns, after.returns, "{}", before.id);
            equivalent(&after.code, &before.code);
        }
    }
}
