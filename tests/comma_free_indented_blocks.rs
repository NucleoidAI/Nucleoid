//! Optional all-comma-free bare blocks, beside unchanged accepted controls.
//! Ordinary tests and all native children need neither Git nor session files.

mod common;
#[path = "support/named_arrow_headers.rs"]
mod headers;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Expr, FunctionBody, Parameter, Stmt, parse, parse_program};
use nucleoid::lang::evaluation::Flow;
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value, statement};
use serde::Deserialize;

fn equivalent(candidate: &str, accepted: &str) -> Vec<Stmt> {
    let actual = parse(candidate).unwrap_or_else(|error| panic!("{candidate}\n{error}"));
    let expected = parse(accepted).unwrap_or_else(|error| panic!("{accepted}\n{error}"));
    assert_eq!(actual, expected, "{candidate}");
    assert_eq!(generate_all(&actual), generate_all(&expected));
    assert_eq!(
        common::assertions::protected_assertions(&actual),
        common::assertions::protected_assertions(&expected)
    );
    actual
}

fn check_pair(
    candidate: &str,
    accepted: &str,
    assertions: usize,
    expected: Option<&str>,
) -> [Runtime; 2] {
    assert_ne!(candidate, accepted, "exercise actual comma omission");
    let ast = equivalent(candidate, accepted);
    assert_eq!(
        common::assertions::protected_assertions(&ast).len(),
        assertions
    );
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let values = runtimes
        .iter_mut()
        .zip([candidate, accepted])
        .map(|(runtime, source)| {
            let value = common::check_with_count(
                runtime,
                &common::Case {
                    title: "comma-free indented block pair".to_string(),
                    source: source.to_string(),
                    expected: expected.map(str::to_string),
                },
                assertions,
            )
            .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
            assert_eq!(runtime.assertions_run(), assertions, "{source}");
            value
        })
        .collect::<Vec<_>>();
    runtime_equivalence::assert_equivalent(
        &runtimes[0],
        Some(&values[0]),
        &runtimes[1],
        Some(&values[1]),
    );
    runtimes
}

fn pair(accepted: &str, assertions: usize, expected: Option<&str>) -> [Runtime; 2] {
    check_pair(
        &headers::comma_free_block_headers(accepted),
        accepted,
        assertions,
        expected,
    )
}

#[test]
fn complete_two_three_duplicate_and_1024_name_lists_preserve_metadata_and_binding() {
    for names in [
        vec!["left", "right"],
        vec!["left", "right", "factor"],
        vec!["type", "value", "_private", "A1", "why", "affects", "model"],
        vec!["same", "same"],
    ] {
        let candidate = format!("def f {}:\n    return 1", names.join(" "));
        let accepted = format!("def f {}:\n    return 1", names.join(", "));
        let ast = equivalent(&candidate, &accepted);
        equivalent(
            &candidate,
            &format!("def f({}):\n    return 1", names.join(", ")),
        );
        let [Stmt::Function(function)] = ast.as_slice() else {
            panic!("one complete named function");
        };
        assert_eq!(function.name.as_deref(), Some("f"));
        assert_eq!(
            function.parameters,
            names
                .iter()
                .map(|name| Parameter {
                    name: (*name).to_string(),
                    type_name: None
                })
                .collect::<Vec<_>>()
        );
        assert_eq!(
            function.body,
            FunctionBody::Block(vec![Stmt::Return(Some(Expr::Number(1.0)))])
        );
        assert_eq!(generate_all(&ast), "def f");
    }
    pair(
        "def duplicate same, same:\n    return same\nassert(duplicate(2, 3), 3)",
        1,
        None,
    );
    for count in [2, 3, 129, 1024] {
        let names = (0..count).map(|i| format!("p{i}")).collect::<Vec<_>>();
        let accepted = format!(
            "def many {}:\n    return p{}\nmany({})",
            names.join(", "),
            count - 1,
            vec!["7"; count].join(", ")
        );
        let ast = equivalent(&headers::comma_free_block_headers(&accepted), &accepted);
        let Stmt::Function(function) = &ast[0] else {
            panic!("named block")
        };
        assert_eq!(function.parameters.len(), count);
        for (name, parameter) in names.iter().zip(&function.parameters) {
            assert_eq!(&parameter.name, name);
            assert_eq!(parameter.type_name, None);
        }
        pair(&accepted, 0, Some("7"));
    }
}

#[test]
fn exact_tokens_layout_tabs_crlf_comments_and_final_suite_colon_are_preserved() {
    assert_eq!(
        tokenize("def add left right:\n    return left + right")
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        [
            Token::Keyword(Keyword::Def),
            Token::Identifier("add".to_string()),
            Token::Identifier("left".to_string()),
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
    for candidate in [
        "def add left right:\n    return left + right",
        "def add\tleft\tright:\n\treturn left + right",
        "def add left right:\r\n    return left + right",
        "def add left right: # suite\n\n# blank\n    # body\n    return left + right",
        "def add left right:\n;\n    return left + right",
        "def add left right:\n    ;\n    return left + right",
    ] {
        let accepted = headers::legacy_block_headers(candidate);
        pair(&accepted, 0, None);
        equivalent(candidate, "def add(left, right):\n    return left + right");
        assert_eq!(
            parse_program(candidate).unwrap().positions,
            parse_program(&accepted).unwrap().positions
        );
    }
    for suffix in [
        "",
        "\n",
        " # EOF",
        "\nnext = 3",
        "\n# boundary\nnext = 3",
        "\n; next = 3",
    ] {
        let candidate = format!("def f left right:\n    int => left\n    return right{suffix}");
        equivalent(&candidate, &headers::legacy_block_headers(&candidate));
    }
    assert_eq!(
        parse_program("def add left right:\n    return left + right\nnext = 3")
            .unwrap()
            .positions,
        [Position::new(1, 1), Position::new(3, 1)]
    );
}

#[test]
fn all_failed_prefixes_keep_the_original_fallback_messages_positions_and_bytes() {
    for header in [
        "left right",
        "left right:",
        "left right:\n",
        "left right:\n# no body",
        "left right:\n    # no body",
        "left right:\nreturn left",
        "left right:\n;\nreturn left",
        "left right:;\n    return left",
        "left right: return left",
        "left right: { return left }",
        "left right:\n{ return left }",
        "left right { return left }",
        "left right, third:\n    return left",
        "left, right third:\n    return left",
        "left right,:\n    return left",
        "left right third, fourth:\n    return left",
        "left: int right:\n    return left",
        "left right: int:\n    return left",
        "left: int right: int:\n    return left",
        "left right: Type.Member:\n    pass",
        "left right: List[int]:\n    pass",
        "left right -> int:\n    pass",
        "left = 1 right:\n    pass",
        "left right = 1:\n    pass",
        "left [right]:\n    pass",
        "left {right}:\n    pass",
        "left ...right:\n    pass",
        "left this:\n    pass",
        "left self:\n    pass",
        "left super:\n    pass",
        "left null:\n    pass",
        "left $Right:\n    pass",
        "left 1:\n    pass",
        "left \"right\":\n    pass",
        "left right\n:\n    pass",
        "left\nright:\n    pass",
        "left right;:\n    pass",
        "left; right:\n    pass",
        "left right # boundary\n:\n    pass",
        "left # boundary\nright:\n    pass",
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
        assert_eq!(headers::legacy_block_headers(&source), source);
        assert_eq!(headers::comma_free_block_headers(&source), source);
    }
    for (source, message, position) in [
        (
            "def f this right:\n    pass",
            "Expected '(' but found keyword 'this'",
            Position::new(1, 7),
        ),
        (
            "def f self right:\n    pass",
            "Expected '(' but found keyword 'this'",
            Position::new(1, 7),
        ),
        (
            "def f $Left right:\n    pass",
            "Expected '(' but found '$Left'",
            Position::new(1, 7),
        ),
        (
            "def f 1 right:\n    pass",
            "Expected '(' but found number 1",
            Position::new(1, 7),
        ),
        (
            "def f\nleft right:\n    pass",
            "Expected '(' but found end of line",
            Position::new(1, 5),
        ),
        (
            "def f; left right:\n    pass",
            "Expected '(' but found ';'",
            Position::new(1, 6),
        ),
        (
            "def f:\n    pass",
            "Expected '(' but found ':'",
            Position::new(1, 6),
        ),
        (
            "def f(left right):\n    pass",
            "Expected ')' but found identifier 'right'",
            Position::new(1, 12),
        ),
        (
            "def f(left, right:):\n    pass",
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
fn legacy_typed_parenthesized_arrow_anonymous_class_and_call_grammars_remain_exact() {
    for signature in [
        "()",
        "(left)",
        "(left: int)",
        "(left, right)",
        "(left: int, right)",
        "(left: int, right: str,)",
        "(\n    left,\n    right,\n)",
    ] {
        equivalent(
            &format!("def f{signature}:\n    return 1"),
            &format!("def f{signature} => 1"),
        );
        assert!(parse(&format!("def f{signature}: pass")).is_ok());
        assert!(parse(&format!("def f{signature}: {{ return 1 }}")).is_ok());
        assert!(parse(&format!("class Item{signature}")).is_ok());
    }
    for signature in [
        "left",
        "left: int",
        "left, right",
        "left: int, right",
        "left, right: int",
        "left: int, right: int",
    ] {
        equivalent(
            &format!("def f {signature}:\n    return 1"),
            &format!("def f({signature}):\n    return 1"),
        );
        equivalent(
            &format!("def f {signature} => 1"),
            &format!("def f({signature}) => 1"),
        );
    }
    equivalent(
        "def f left right => left + right",
        "def f left, right => left + right",
    );
    for source in [
        "(left, right) => left + right",
        "(left: int, right) => { return left + right }",
        "function(left, right) { return left + right }",
    ] {
        let ast = parse(source).unwrap();
        let [Stmt::Expression(Expr::Function(function))] = ast.as_slice() else {
            panic!("anonymous control")
        };
        assert!(function.name.is_none());
    }
    for source in [
        "class Item left right",
        "class Item(left right)",
        "(left right) => left",
        "left right:\n    return left",
    ] {
        assert!(parse(source).is_err(), "{source}");
    }
    assert_eq!(
        parse("add left right").unwrap(),
        ["add", "left", "right"].map(|name| Stmt::Expression(Expr::Identifier(name.to_string())))
    );
    equivalent("left right => left", "left\nright => left");
}

#[test]
fn whole_statement_suites_keep_inner_punctuation_branches_and_all_return_paths() {
    for body in [
        "if left: return right\n    return left",
        "if left: { return right }\n    return left",
        "{ return left + right }",
        "return (\n        left +\n        right\n    )",
    ] {
        let accepted = format!(
            "def f left, right:\n    {body}\nassert(f(2, 3), {})",
            if body.starts_with("if") { 3 } else { 5 }
        );
        pair(&accepted, 1, None);
    }
    let accepted = r#"
def coordinates x, y:
    return {x, y}
def pair left, right:
    return [left, right]
def label left, right:
    return `${left},${right}`
def factory ignored, other:
    return (left: int, right) => left + right
def outer initial, extra:
    def combine first, second, scale:
        return (first + second) * scale
    return combine(initial, extra, 3)
def noReturn left, right:
    left + right
def bareReturn left, right:
    return
point = coordinates(2, 3)
assert(point.x, 2)
assert(point.y, 3)
assert(pair("left", "right"), ["left", "right"])
assert(label("a", "b"), "a,b")
callback = factory(10, 20)
assert(callback(2, 3), 5)
assert(outer(4, 2), 18)
assert(noReturn(2, 3), null)
assert(bareReturn(2, 3), null)
"#;
    let candidate = headers::comma_free_block_headers(accepted);
    for body in [
        "return {x, y}",
        "return [left, right]",
        "(left: int, right) => left + right",
        "`${left},${right}`",
    ] {
        assert!(candidate.contains(body), "{body}");
    }
    pair(accepted, 8, None);
}

#[test]
fn missing_null_extra_arguments_assignments_and_variable_callable_shadowing_keep_scope() {
    let accepted = r#"
left = 10
right = 20
def pair left, right:
    return [left, right]
def add left, right:
    left += right
    return left
def operation number => 99
def apply operation, number:
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
    pair(accepted, 11, None);
}

#[test]
fn explicit_and_bare_super_methods_init_this_self_and_named_adoption_are_unchanged() {
    let accepted = r#"
class Parent:
    def init left, right:
        this.amount = left + right
    def reading offset, scale:
        return self.amount * scale + offset
class Child: Parent
    def init left, right:
        super(left, right)
    def reading offset, scale:
        return self.amount * scale + offset
    def increase delta, extra:
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
    def init left, right:
        super
        self.amount = left + right
tagged = Tagged(3, 5)
assert(tagged.ready, true)
assert(tagged.amount, 8)
"#;
    pair(accepted, 10, None);
}

#[test]
fn noncapture_receiver_failures_and_actual_body_positions_keep_exact_errors() {
    for (accepted, message, line) in [
        (
            "def wrap left, right:\n    return { missing }\nwrap(1, 2)",
            "missing is not defined",
            3,
        ),
        (
            "def factory left, right:\n    return value => ({ left, right })\ncallback = factory(1, 2)\ncallback(3)",
            "left is not defined",
            4,
        ),
        (
            "def receiver left, right:\n    return this.amount\nreceiver(1, 2)",
            "this is not defined",
            3,
        ),
        (
            "def receiver left, right:\n    return self.amount\nreceiver(1, 2)",
            "this is not defined",
            3,
        ),
    ] {
        let candidate = headers::comma_free_block_headers(accepted);
        equivalent(&candidate, accepted);
        let error = Runtime::new().run(&candidate).unwrap_err();
        assert_eq!(error, Runtime::new().run(accepted).unwrap_err());
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(line, 1)));
    }
    for (body, message, position) in [
        ("return left +)", "Unexpected ')'", Position::new(2, 19)),
        ("return @", "Unexpected character '@'", Position::new(2, 12)),
        (
            "return { left = 1 }",
            "Expected ':' but found '='",
            Position::new(2, 19),
        ),
    ] {
        for header in [
            "def f left right:",
            "def f left, right:",
            "def f(left, right):",
        ] {
            let error = Runtime::check(&format!("{header}\n    {body}")).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert_eq!(error.position(), Some(position));
        }
    }
}

#[test]
fn live_aliases_snapshots_graph_keys_dependencies_order_and_reasoning_are_identical() {
    let accepted = r#"
bias = 2
def scaled left, right:
    return left * right + bias
live = scaled
snapshot = scaled.value
left = 3
right = 4
result = live(left, right)
frozen = snapshot(left, right)
assert(result, 14)
assert(frozen, 14)
def scaled left, right:
    return left + right + bias
assert(result, 9)
assert(frozen, 14)
bias = 4
assert(result, 11)
assert(frozen, 16)
result
"#;
    let [mut actual, mut expected] = pair(accepted, 6, Some("11"));
    assert!(actual.graph.keys().eq(expected.graph.keys()));
    for node in actual.graph.nodes() {
        let other = expected.graph.retrieve(&node.key).unwrap();
        assert_eq!(node.dependencies, other.dependencies);
        assert_eq!(node.dependents, other.dependents);
        assert_eq!(node.sequence, other.sequence);
        assert_eq!(format!("{:?}", node.node), format!("{:?}", other.node));
        assert!(
            actual
                .graph
                .shape_dependencies(&node.key)
                .eq(expected.graph.shape_dependencies(&node.key))
        );
    }
    for source in ["why result", "model |> why"] {
        let a = actual.run(source).unwrap();
        let b = expected.run(source).unwrap();
        assert_eq!(
            actual.serialize_json(&a).unwrap(),
            expected.serialize_json(&b).unwrap()
        );
        runtime_equivalence::assert_equivalent(&actual, Some(&a), &expected, Some(&b));
    }
    assert_eq!(
        NodeKey::block(
            &parse("def add left right:\n    return left + right").unwrap(),
            None
        )
        .as_str(),
        "block(def add)"
    );
}

#[test]
fn real_caught_and_uncaught_rollback_preserve_state_edges_and_recovery() {
    pair(
        "def fail left, right:\n    throw \"EXPECTED\"\ntry:\n    fail(2, 3)\ncatch error:\n    assert(error, \"EXPECTED\")",
        1,
        None,
    );
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, header) in runtimes
        .iter_mut()
        .zip(["def reading left right:", "def reading left, right:"])
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
            "def reading left right, extra:\n    return left",
            "def reading left right: int:\n    return left",
            "def reading left right:\nreturn left",
            "def reading left right:;\n    return left",
            "def reading left\nright:\n    return left",
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
    runtime_equivalence::assert_equivalent(&runtimes[0], None, &runtimes[1], None);
}

#[test]
fn parameter_dependent_local_keeps_its_inherited_failure_rollback_and_recovery() {
    let accepted = "fee = 2\ndef total amount, other:\n    adjusted = amount + fee\n    return adjusted\ninvoice = total(5, 0)\nassert(invoice, 7)";
    let mut runtimes = pair(accepted, 1, None);
    let mut errors = Vec::new();
    for runtime in &mut runtimes {
        let error = runtime.run("fee = 3").unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "amount is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        errors.push(error);
        assert_eq!(runtime.run("fee").unwrap(), Value::Number(2.0));
        assert_eq!(runtime.run("invoice").unwrap(), Value::Number(7.0));
        runtime
            .run("input = 2\noutput = input * 3\ninput = 4")
            .unwrap();
        assert_eq!(runtime.run("output").unwrap(), Value::Number(12.0));
    }
    assert_eq!(errors[0], errors[1]);
    runtime_equivalence::assert_equivalent(&runtimes[0], None, &runtimes[1], None);
}

#[test]
fn inherited_body_bounds_fail_atomically_without_imposing_flat_parameter_bounds() {
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
        let mut errors = Vec::new();
        for header in ["def limited left right:", "def limited left, right:"] {
            let mut runtime = Runtime::new();
            runtime.run("saved = 7").unwrap();
            let error = runtime
                .run(&format!("saved = 3\n{header}\n    return {body}"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert_eq!(error.position().unwrap().line, 3);
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
            runtime
                .run("def add left right:\n    return left + right")
                .unwrap();
            assert_eq!(runtime.run("add(2, 3)").unwrap(), Value::Number(5.0));
            errors.push(error);
        }
        assert_eq!(errors[0], errors[1]);
    }
    let body = (1..100)
        .map(|depth| format!("{}if true:\n", "    ".repeat(depth)))
        .collect::<String>();
    let errors = ["def limited left right:", "def limited left, right:"].map(|header| {
        Runtime::check(&format!("{header}\n{body}{}pass", "    ".repeat(100))).unwrap_err()
    });
    assert_eq!(errors[0], errors[1]);
    // Conditions enter expression parsing at the inherited shared depth first.
    assert_eq!(errors[0].message(), "Expressions are nested too deeply");
    let body = format!("{}pass{}", "{ ".repeat(65), " }".repeat(65));
    let errors = ["def limited left right:", "def limited left, right:"]
        .map(|header| Runtime::check(&format!("{header}\n    {body}")).unwrap_err());
    assert_eq!(errors[0], errors[1]);
    assert_eq!(errors[0].message(), "Statements are nested too deeply");
}

#[test]
fn direct_compile_and_nodes_preserve_caller_scopes_frames_receivers_return_and_failure() {
    for expression in ["left + right", "left += right", "missing"] {
        let candidate = format!("def f left right:\n    return {expression}");
        let accepted = headers::legacy_block_headers(&candidate);
        assert_eq!(
            statement::compile(&candidate).unwrap().statements,
            parse(&accepted).unwrap()
        );
        assert_eq!(
            statement::compile(&candidate).unwrap().positions,
            [Position::new(1, 1)]
        );
        for source in [&candidate, &accepted] {
            let mut runtime = Runtime::new();
            let mut scope = Scope::new();
            scope.push();
            scope.declare("left", Value::Number(99.0));
            scope.declare("right", Value::Number(88.0));
            let receiver = ObjectId::named("caller");
            scope.set_this(Some(receiver.clone()));
            scope.set_instance(Some(receiver.clone()));
            let ast = parse(source).unwrap();
            let mut definition = Nuc::convert(&mut runtime, &mut scope, &ast[0]).unwrap();
            assert!(matches!(definition, Nuc::Function(_)));
            definition.before(&mut runtime, &mut scope).unwrap();
            let outcome = definition.run(&mut runtime, &mut scope).unwrap();
            definition
                .graph(&mut runtime, outcome.dependencies)
                .unwrap();
            definition.after(&mut runtime).unwrap();
            let call = parse("f(2, 3)").unwrap();
            let mut node = Nuc::convert(&mut runtime, &mut scope, &call[0]).unwrap();
            if expression == "missing" {
                let error = node.run(&mut runtime, &mut scope).err().unwrap();
                assert_eq!(error.kind(), ErrorKind::Reference);
                assert_eq!(error.message(), "missing is not defined");
            } else {
                assert!(matches!(
                    node.run(&mut runtime, &mut scope).unwrap().flow,
                    Flow::Normal(Value::Number(5.0))
                ));
            }
            assert_eq!(scope.depth(), 2);
            assert_eq!(scope.retrieve("left"), Some(&Value::Number(99.0)));
            assert_eq!(scope.retrieve("right"), Some(&Value::Number(88.0)));
            assert_eq!(scope.this(), Some(&receiver));
            assert_eq!(scope.instance(), Some(&receiver));
            runtime
                .run("input = 2\noutput = input * 3\ninput = 4")
                .unwrap();
            assert_eq!(runtime.run("output").unwrap(), Value::Number(12.0));
        }
    }
}

#[test]
fn shared_normalization_is_explicit_token_qualified_and_retains_every_other_byte() {
    let accepted = "# def fake left,right:\ntext = \"def fake left,right:\"\ndef f left,right:\n    return {left,right}\ndef arrow left, right => left + right\ndef typed left: int, right:\n    return [left,right]\ncallback = (left,right) => left + right";
    let candidate = "# def fake left,right:\ntext = \"def fake left,right:\"\ndef f left right:\n    return {left,right}\ndef arrow left, right => left + right\ndef typed left: int, right:\n    return [left,right]\ncallback = (left,right) => left + right";
    assert_eq!(headers::comma_free_block_headers(accepted), candidate);
    equivalent(candidate, accepted);
    assert_eq!(
        headers::legacy_block_headers(candidate),
        accepted.replace("def f left,right:", "def f left, right:")
    );
    assert!(headers::comma_free_headers(accepted).contains("def f left,right:"));
    assert!(headers::legacy_headers(candidate).contains("def f left right:"));
    for (accepted, candidate) in [
        (
            "def f left,right:\n    return left",
            "def f left right:\n    return left",
        ),
        (
            "def f left ,right:\n    return left",
            "def f left right:\n    return left",
        ),
        (
            "def f left,\tright:\n    return left",
            "def f left\tright:\n    return left",
        ),
        (
            "def f left , right:\n    return left",
            "def f left  right:\n    return left",
        ),
    ] {
        assert_eq!(headers::comma_free_block_headers(accepted), candidate);
        equivalent(candidate, accepted);
    }
    let code = "def f left right:\n    return [left,right]";
    let encoded = serde_json::to_string(code).unwrap();
    let raw = format!(
        " {{ \"description\" : \"code: def fake left right:\", \"nested\": {{\"code\":\"not a program\"}}, \"code\" : {encoded}, \"return\":null, \"annotation\":[1,2] }}\r\n"
    );
    let expected = raw.replace(
        &encoded,
        &serde_json::to_string(&headers::legacy_block_headers(code)).unwrap(),
    );
    assert_eq!(
        headers::legacy_block_training_bytes(raw.as_bytes()),
        expected.as_bytes()
    );
    assert_eq!(
        headers::legacy_block_training_bytes(expected.as_bytes()),
        expected.as_bytes()
    );
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedCase {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

fn sealed() -> Vec<SealedCase> {
    serde_json::from_str(ACCEPTED_V2).unwrap()
}

fn documents() -> [(&'static str, Vec<common::Case>); 4] {
    [
        ("spec", common::cases(include_str!("../nucleoid.spec.md"))),
        ("reference", common::cases(include_str!("reference.md"))),
        (
            "readme",
            common::snippets(include_str!("../README.md"), "README.md"),
        ),
        (
            "examples",
            common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
        ),
    ]
}

#[test]
fn eight_exact_v2_counterparts_keep_identities_order_comments_51_assertions_and_return21() {
    let sealed = sealed();
    assert_eq!(sealed.len(), 8);
    let mut checked = 0;
    let mut assertions = 0;
    let mut returns = Vec::new();
    for ((document, mut cases), prefix) in documents().into_iter().zip([378, 76, 27, 82]) {
        assert!(
            cases.len() >= prefix,
            "{document}: protected prefix truncated"
        );
        let own = sealed
            .iter()
            .filter(|case| case.document == document)
            .collect::<Vec<_>>();
        let selected = |cases: &[common::Case]| {
            cases
                .iter()
                .filter(|case| own.iter().any(|sealed| sealed.title == case.title))
                .map(|case| case.title.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            selected(&cases),
            own.iter()
                .map(|case| case.title.clone())
                .collect::<Vec<_>>()
        );
        for (offset, accepted) in own.iter().enumerate() {
            let case = &cases[prefix - own.len() + offset];
            assert_eq!(case.title, accepted.title);
            assert_eq!(case.expected, accepted.expected);
            assert_eq!(
                case.source.trim(),
                headers::comma_free_block_headers(&accepted.source)
            );
            let count = [8, 8, 10, 10, 7, 3, 2, 3][checked];
            check_pair(
                &case.source,
                &accepted.source,
                count,
                accepted.expected.as_deref(),
            );
            checked += 1;
            assertions += count;
            if let Some(value) = &accepted.expected {
                returns.push(value.as_str());
            }
        }
        cases.push(common::Case {
            title: "future unrelated tail".to_string(),
            source: "assert(1, 1)".to_string(),
            expected: None,
        });
        assert_eq!(selected(&cases).len(), own.len());
    }
    assert_eq!((checked, assertions), (8, 51));
    assert_eq!(returns, ["21"]);
}

#[test]
fn all_current_rewritten_programs_keep_full_ast_canonical_state_graph_and_actual_coverage() {
    let mut checked = 0;
    for (_, cases) in documents() {
        for case in cases {
            let accepted = headers::legacy_block_headers(&case.source);
            if accepted == case.source {
                continue;
            }
            let assertions =
                common::assertions::protected_assertions(&parse(&accepted).unwrap()).len();
            check_pair(
                &case.source,
                &accepted,
                assertions,
                case.expected.as_deref(),
            );
            checked += 1;
        }
    }
    assert!(
        checked >= 17,
        "nine historical and eight new programs remain covered"
    );
}

struct NativeFile {
    directory: PathBuf,
    path: PathBuf,
}

impl NativeFile {
    fn new(label: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("nucleoid-nuc40-{label}-{}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("program.nuc");
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        Self { directory, path }
    }

    fn run(&self, bytes: &[u8]) -> Output {
        fs::write(&self.path, bytes).unwrap();
        assert_eq!(fs::read(&self.path).unwrap(), bytes);
        Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .args(["--json", "--graph"])
            .arg(&self.path)
            .output()
            .unwrap()
    }
}

impl Drop for NativeFile {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.directory).unwrap();
    }
}

fn repl(bytes: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    child.wait_with_output().unwrap()
}

fn same_outputs(outputs: &[Output; 2]) {
    assert_eq!(outputs[0].status.code(), outputs[1].status.code());
    assert_eq!(outputs[0].stdout, outputs[1].stdout);
    assert_eq!(outputs[0].stderr, outputs[1].stderr);
}

#[test]
fn native_exact_lf_crlf_tab_files_and_all_sealed_pairs_keep_json_and_canonical_graphs() {
    let file = NativeFile::new("pairs");
    for candidate in [
        "def add left right:\n    total = left + right\n    return total\nadd(2, 3)",
        "def add\tleft\tright:\r\n\ttotal = left + right\r\n\treturn total\r\nadd(2, 3)",
        "def add left right: # suite\n\n# blank\n    return left + right\nadd(2, 3)\n",
    ] {
        let accepted = headers::legacy_block_headers(candidate);
        let outputs = [
            file.run(candidate.as_bytes()),
            file.run(accepted.as_bytes()),
        ];
        for output in &outputs {
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            assert_eq!(
                String::from_utf8_lossy(&output.stdout).lines().next(),
                Some("5.0")
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("add [function]"));
        }
        same_outputs(&outputs);
    }
    for accepted in sealed() {
        let candidate = headers::comma_free_block_headers(&accepted.source);
        let outputs = [
            file.run(candidate.as_bytes()),
            file.run(accepted.source.as_bytes()),
        ];
        for output in &outputs {
            assert!(output.status.success(), "{}: {output:?}", accepted.title);
            assert!(output.stderr.is_empty(), "{output:?}");
            if let Some(expected) = &accepted.expected {
                assert_eq!(
                    String::from_utf8_lossy(&output.stdout).lines().next(),
                    Some(format!("{expected}.0").as_str())
                );
            }
        }
        same_outputs(&outputs);
    }
}

#[test]
fn native_file_incomplete_headers_body_errors_positions_and_assertion_failures_are_exact() {
    let file = NativeFile::new("errors");
    for (source, message) in [
        (
            "def f left right",
            "SyntaxError: Expected '(' but found identifier 'left' (line 1, column 7)",
        ),
        (
            "def f left right:",
            "SyntaxError: Expected '(' but found identifier 'left' (line 1, column 7)",
        ),
        (
            "def f left right:\r\n\t# no body",
            "SyntaxError: Expected '(' but found identifier 'left' (line 1, column 7)",
        ),
        (
            "def f left right:;\n    return left",
            "SyntaxError: Expected '(' but found identifier 'left' (line 1, column 7)",
        ),
        (
            "def f left right:\n    return left +)",
            "SyntaxError: Unexpected ')' (line 2, column 19)",
        ),
        (
            "def f left right:\r\n\treturn left +)",
            "SyntaxError: Unexpected ')' (line 2, column 16)",
        ),
        (
            "def f left right:\n    return @",
            "SyntaxError: Unexpected character '@' (line 2, column 12)",
        ),
        (
            "def f left right:\n    return missing\nf(2, 3)",
            "ReferenceError: missing is not defined (line 3, column 1)",
        ),
    ] {
        let output = file.run(source.as_bytes());
        assert!(!output.status.success(), "{output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap().trim_end(),
            message
        );
    }
    let candidate = "def f left right:\n    return left + right\nassert(f(2, 3), 6)";
    let accepted = headers::legacy_block_headers(candidate);
    let outputs = [
        file.run(candidate.as_bytes()),
        file.run(accepted.as_bytes()),
    ];
    assert!(!outputs[0].status.success());
    assert!(
        String::from_utf8_lossy(&outputs[0].stderr).contains("assertion failed: expected 6, got 5")
    );
    same_outputs(&outputs);
}

#[test]
fn native_stdin_pending_blank_commit_literal_eof_crlf_tabs_and_methods_stay_paired() {
    for candidate in [
        ".json on\n.graph on\ndef add left right:\n    return left + right\n\nadd(2, 3)",
        ".json on\r\n.graph on\r\ndef add\tleft\tright:\r\n\treturn left + right\r\n\r\nadd(2, 3)",
        ".json on\nclass Item:\n    def init left right:\n        this.amount = left + right\n    def reading offset scale:\n        return self.amount * scale + offset\n\nitem = Item(2, 3)\nitem.reading(1, 2)",
    ] {
        let accepted = headers::legacy_block_headers(candidate);
        let outputs = [repl(candidate.as_bytes()), repl(accepted.as_bytes())];
        for output in &outputs {
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            assert!(
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .any(|line| line == "5.0" || line == "11.0")
            );
        }
        same_outputs(&outputs);
    }
    for ending in ["", "\n", "\r\n"] {
        let candidate =
            format!(".json on\n.graph on\ndef add left right:\n    return left + right{ending}");
        let accepted = headers::legacy_block_headers(&candidate);
        let outputs = [repl(candidate.as_bytes()), repl(accepted.as_bytes())];
        for output in &outputs {
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            assert!(String::from_utf8_lossy(&output.stdout).contains("add [function]"));
        }
        same_outputs(&outputs);
    }
    for source in [
        "def f left right:",
        "def f left right:\n",
        "def f left right:\r\n\t# pending",
    ] {
        let output = repl(source.as_bytes());
        assert_eq!(
            String::from_utf8_lossy(&output.stderr).trim_end(),
            "SyntaxError: Expected '(' but found identifier 'left' (line 1, column 7)"
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("f [function]"));
    }
}

#[test]
fn native_repl_commented_colon_header_buffers_the_body_and_keeps_outputs_paired() {
    let candidate = ".json on\r\n.graph on\r\ndef add\tleft\tright: # header\r\n\treturn left + right\r\n\r\nadd(2, 3)";
    let accepted = headers::legacy_block_headers(candidate);
    let outputs = [repl(candidate.as_bytes()), repl(accepted.as_bytes())];
    same_outputs(&outputs);
    for output in &outputs {
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.lines().any(|line| line == "5.0"), "{output:?}");
        assert!(stdout.contains("add [function]"), "{output:?}");
    }
}

#[test]
fn native_repl_recovers_previous_definition_after_structured_and_body_limit_errors() {
    let candidate = format!(
        ".json on\ndef add left right:\n    return left + right\n\nadd(2, 3)\n\
         def add left right, extra:\n    return left\n\n\
         def add left right:\n    return left +)\n\n\
         def limited left right:\n    return left{}\n\nadd(3, 4)\nadd(4, 5)",
        " + 1".repeat(200)
    );
    let accepted = headers::legacy_block_headers(&candidate);
    let outputs = [repl(candidate.as_bytes()), repl(accepted.as_bytes())];
    for output in &outputs {
        assert!(output.status.success(), "{output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(stderr.lines().count(), 3, "{stderr}");
        assert!(
            stderr.contains(
                "SyntaxError: Expected '(' but found identifier 'left' (line 1, column 9)"
            ),
            "{stderr}"
        );
        assert!(
            stderr.contains("SyntaxError: Unexpected ')' (line 2, column 18)"),
            "{stderr}"
        );
        assert!(
            stderr.contains("SyntaxError: Expression has too many operands"),
            "{stderr}"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        for expected in ["5.0", "7.0", "9.0"] {
            assert!(stdout.lines().any(|line| line == expected), "{stdout}");
        }
    }
    same_outputs(&outputs);
}

// Exact complete v2 inputs; explicit Child.total is intentional. No fixture
// discovery, Git, inherited-method repair, or session artifact is required.
const ACCEPTED_V2: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid defines comma-free bare indented blocks",
    "source": "# Nucleoid defines comma-free bare indented blocks\n\ndef add left, right:\n    total = left + right\n    return total\ndef scaled left, right, factor:\n    return (left + right) * factor\ndef largest left, right:\n    if left > right:\n        return left\n    return right\ndef pair left, right:\n    return [left, right]\ndef coordinates x, y:\n    return {x, y}\ndef noReturn left, right:\n    left + right\nassert(add(2, 3), 5)\nassert(scaled(2, 3, 4), 20)\nassert(largest(7, 3), 7)\nassert(largest(2, 9), 9)\nassert(pair(\"left\", \"right\"), [\"left\", \"right\"])\npoint = coordinates(2, 3)\nassert(point.x, 2)\nassert(point.y, 3)\nassert(noReturn(2, 3), null)",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid keeps comma-free bare indented blocks reactive",
    "source": "# Nucleoid keeps comma-free bare indented blocks reactive\n\noffset = 1\ndef adjusted left, right:\n    return left + right + offset\nalias = adjusted\nfrozen = adjusted.value\nleft = 2\nright = 3\ntotal = alias(left, right)\nsnapshot = total.value\nassert(total, 6)\nassert(snapshot, 6)\noffset = 4\nassert(total, 9)\nleft = 5\nassert(total, 12)\ndef adjusted left, right:\n    return left * right + offset\nassert(total, 19)\nassert(frozen(2, 3), 9)\nassert(snapshot, 6)\nright = 2\nassert(total, 14)",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid preserves comma-free block methods and initializers",
    "source": "# Nucleoid preserves comma-free block methods and initializers\n\nclass Parent:\n    def init left, right:\n        this.left = left\n        this.right = right\n    def total left, right:\n        return this.left + this.right + left + right\nclass Child: Parent\n    def init left, right:\n        super(left, right)\n        self.extra = 2\n    def total left, right:\n        return this.left + this.right + left + right\nchild = Child(2, 3)\nassert(child.left, 2)\nassert(child.right, 3)\nassert(child.extra, 2)\nobserved = child.total(4, 5)\nassert(observed, 14)\nchild.left = 7\nassert(observed, 19)\nresult = child.total(1, 2)\nassert(result, 13)\nchild.right = 4\nassert(observed, 20)\nassert(result, 14)\nassert(Parent.length, 0)\nassert(Child.length, 1)",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid preserves comma-free block bindings and explicit returns",
    "source": "# Nucleoid preserves comma-free block bindings and explicit returns\n\nleft = 10\nright = 20\ndef pair first, second:\n    return [first, second]\ndef add first, second:\n    first += second\n    return first\ndef outer initial, extra:\n    def combine first, second, scale:\n        return (first + second) * scale\n    return combine(initial, extra, 3)\ndef factory ignored, other:\n    return (first, second) => first + second\ndef bareReturn first, second:\n    return\nassert(pair(3, 4), [3, 4])\nassert(pair(3), [3, null])\nassert(pair(), [null, null])\nassert(pair(3, 4, 99), [3, 4])\nassert(add(3, 4), 7)\nassert(left, 10)\nassert(right, 20)\nassert(outer(4, 2), 18)\ncallback = factory(10, 20)\nassert(callback(2, 3), 5)\nassert(bareReturn(3, 4), null)",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back comma-free block redefinitions",
    "source": "# Nucleoid rolls back comma-free block redefinitions\n\nsource = 3\ndef reading left, right:\n    return source + left + right\nresult = reading(2, 4)\nassert(result, 9)\ntry:\n    def reading left, right:\n        return missing + left + right\ncatch error:\n    assert(error, ReferenceError(\"missing is not defined\"))\nassert(result, 9)\nsource = 5\nassert(result, 11)\ndef combined first, second, scale:\n    return (first + second) * scale\ndef prefer current, fallback:\n    current ||= fallback\n    return current\nassert(prefer(0, 9), 9)\nassert(prefer(3, 9), 3)\nassert(combined(2, 5, 3), 21)\ncombined(2, 5, 3)",
    "expected": "21"
  },
  {
    "document": "reference",
    "title": "Nucleoid preserves comma-free block relationships",
    "source": "# Nucleoid preserves comma-free block relationships\n\nscale = 1\ndef area width, height:\n    return width * height * scale\nwidth = 3\nheight = 4\nresult = area(width, height)\nassert(result, 12)\nscale = 2\nassert(result, 24)\nwidth = 5\nassert(result, 40)",
    "expected": null
  },
  {
    "document": "readme",
    "title": "Indented blocks can omit every parameter comma",
    "source": "def add left, right:\n    return left + right\nleft = 2\nright = 3\ntotal = add(left, right)\nassert(total, 5)\nleft = 5\nassert(total, 8)",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Comma-free blocks keep live input relationships",
    "source": "bias = 1\ndef combined left, right:\n    total = left + right\n    return total + bias\nleft = 2\nright = 3\nreading = combined(left, right)\nassert(reading, 6)\nbias = 4\nassert(reading, 9)\nleft = 5\nassert(reading, 12)",
    "expected": null
  }
]"###;
