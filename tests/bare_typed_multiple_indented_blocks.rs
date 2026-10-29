//! Typed/mixed bare multiple-parameter headers preserve the entire named block.
//! Ordinary tests need neither Git nor session artifacts. Only the two explicitly
//! ignored historical audits require the pinned accepted Git objects.

mod common;
#[path = "support/named_arrow_headers.rs"]
mod named_arrow_headers;

use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Expr, FunctionBody, Parameter, Stmt, parse, parse_program};
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

/// Change only eligible typed/mixed multiple-parameter indented block headers.
fn bare_blocks(explicit: &str) -> String {
    let header = Regex::new(
        r"(?m)^([ \t]*def [A-Za-z_]\w*)\(([A-Za-z_]\w*(?:: [A-Za-z_]\w*)?(?:, [A-Za-z_]\w*(?:: [A-Za-z_]\w*)?)+)\):$",
    )
    .unwrap();
    header
        .replace_all(explicit, |captures: &regex::Captures<'_>| {
            if captures[2].contains(':') {
                format!("{} {}:", &captures[1], &captures[2])
            } else {
                captures[0].to_string()
            }
        })
        .into_owned()
}

fn check_forms(
    bare: &str,
    explicit: &str,
    assertions: usize,
    expected: Option<&str>,
) -> [Runtime; 2] {
    assert_ne!(
        bare, explicit,
        "a runtime pair must exercise the new header"
    );
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
                title: "typed multiple indented-block pair".to_string(),
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

fn graphs_match(bare: &Runtime, explicit: &Runtime) {
    assert!(bare.graph.keys().eq(explicit.graph.keys()));
    for node in bare.graph.nodes() {
        let other = explicit.graph.retrieve(&node.key).unwrap();
        assert_eq!(node.kind, other.kind, "{}", node.key);
        assert_eq!(node.dependencies, other.dependencies, "{}", node.key);
        assert_eq!(node.dependents, other.dependents, "{}", node.key);
        assert!(
            bare.graph
                .shape_dependencies(&node.key)
                .eq(explicit.graph.shape_dependencies(&node.key)),
            "{}",
            node.key
        );
    }
}

#[test]
fn every_annotation_position_preserves_complete_metadata_order_body_and_rendering() {
    for names in [vec!["left", "right"], vec!["left", "right", "factor"]] {
        for mask in 1..(1 << names.len()) {
            let parameters = names
                .iter()
                .enumerate()
                .map(|(index, name)| Parameter {
                    name: (*name).to_string(),
                    type_name: (mask & (1 << index) != 0).then(|| format!("Type{index}")),
                    default: None,
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
            let statements = equivalent(
                &format!("def calculate {header}:\n    return 1"),
                &format!("def calculate({header}):\n    return 1"),
            );
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
        "def f type: value, _private, A1: ArbitraryType, why: model, affects:\n    return type",
        "def f(type: value, _private, A1: ArbitraryType, why: model, affects):\n    return type",
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
fn final_untyped_suite_colon_is_not_an_annotation_and_final_typed_colons_stay_distinct() {
    for header in [
        "left: int, right",
        "left, right: int",
        "left: int, right: int",
        "left: int, right, factor",
        "left, right: int, factor: number",
    ] {
        // The first identifier in the body must never become a header annotation.
        check_forms(
            &format!("def f {header}:\n    int => left\nassert(f(2, 3), null)"),
            &format!("def f({header}):\n    int => left\nassert(f(2, 3), null)"),
            1,
            None,
        );
    }
    for (bare, explicit) in [
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
}

#[test]
fn immediate_existing_tokens_and_real_indent_leave_layout_and_positions_for_suite() {
    assert_eq!(
        tokenize("def add left: int, right: int:\n    return left + right")
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
            Token::Colon,
            Token::Identifier("int".to_string()),
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
    for header in [
        "left: int, right",
        "left, right: int",
        "left: int, right: int",
    ] {
        for layout in [
            "\n    ",
            "\r\n    ",
            " # suite\n\n# blank\n    # body\n    ",
            "\n;\n    ",
            "\n    ;\n    ",
        ] {
            equivalent(
                &format!("def add {header}:{layout}return left + right"),
                &format!("def add({header}):\n    return left + right"),
            );
        }
    }
    for source in [
        "def add left:int,right:\n    return left + right",
        "def add\tleft:\tint,\tright:\n\treturn left + right",
    ] {
        equivalent(
            source,
            "def add(left: int, right):\n    return left + right",
        );
    }
    for suffix in ["\nnext = 3", "\n# boundary\nnext = 3", "\n; next = 3"] {
        equivalent(
            &format!("def add left: int, right:\n    return left + right{suffix}"),
            &format!("def add(left: int, right):\n    return left + right{suffix}"),
        );
    }
    let bare = "def add left: int, right:\n    return left + right\nnext = 3";
    let explicit = "def add(left: int, right):\n    return left + right\nnext = 3";
    assert_eq!(
        parse_program(bare).unwrap().positions,
        parse_program(explicit).unwrap().positions
    );
    assert_eq!(
        parse_program(bare).unwrap().positions,
        [Position::new(1, 1), Position::new(3, 1)]
    );
}

#[test]
fn formerly_negative_typed_multiple_headers_now_have_distinct_positive_runtime_pairs() {
    for (bare, explicit, assertions, count) in [
        (
            "def f amount: int, other:\n    return amount",
            "def f(amount: int, other):\n    return amount",
            "assert(f(3, 4), 3)",
            1,
        ),
        (
            "def f amount: int, other: str:\n    return amount",
            "def f(amount: int, other: str):\n    return amount",
            "assert(f(3, \"four\"), 3)",
            1,
        ),
        (
            "class Gauge:\n    def reading amount: int, other:\n        return amount",
            "class Gauge:\n    def reading(amount: int, other):\n        return amount",
            "gauge = Gauge()\nassert(gauge.reading(5, 6), 5)",
            1,
        ),
        (
            "saved = 3\ndef reading amount: int, other:\n    return amount",
            "saved = 3\ndef reading(amount: int, other):\n    return amount",
            "assert(saved, 3)\nassert(reading(6, 7), 6)",
            2,
        ),
        (
            "def identity amount: int, other:\n    return amount",
            "def identity(amount: int, other):\n    return amount",
            "assert(identity(7, 8), 7)",
            1,
        ),
        (
            "def scaled amount: int, other:\n    return amount",
            "def scaled(amount: int, other):\n    return amount",
            "assert(scaled(8, 9), 8)",
            1,
        ),
        (
            "def reading left, right: int:\n    return left",
            "def reading(left, right: int):\n    return left",
            "assert(reading(9, 10), 9)",
            1,
        ),
    ] {
        check_forms(
            &format!("{bare}\n{assertions}"),
            &format!("{explicit}\n{assertions}"),
            count,
            None,
        );
    }
    for header in [
        "left: int, right",
        "left, right: int",
        "left: int, right: int",
        "left: int, right, factor: number",
        "left, right: int, factor",
    ] {
        for layout in ["\n    ", "\n    # comment\n    "] {
            check_forms(
                &format!("def f {header}:{layout}return left\nassert(f(2, 3), 2)"),
                &format!("def f({header}):{layout}return left\nassert(f(2, 3), 2)"),
                1,
                None,
            );
        }
    }
}

#[test]
fn failed_complete_scans_preserve_parameter_boundary_messages_and_positions() {
    for header in [
        "left: int, right",
        "left: int, right:",
        "left: int, right:\n",
        "left: int, right:\n# no body",
        "left: int, right:\n    # no body",
        "left: int, right:\nreturn left",
        "left: int, right:\n;\nreturn left",
        "left: int, right:;\n    return left",
        "left: int, right: return left",
        "left: int, right: { return left }",
        "left: int, right:\n{ return left }",
        "left: int, right { return left }",
        "left: int, right,:\n    return left",
        "left: int,:\n    return left",
        "left: int,, right:\n    return left",
        "left: int, right other:\n    return left",
        "left:, right:\n    return left",
        "left: int, right::\n    return left",
        "left: int, right: int, third::\n    return left",
        "left = 1, right: int:\n    return left",
        "left: int = 1, right:\n    return left",
        "left: int, right = 1:\n    return left",
        "left: int, [right]:\n    return left",
        "left: int, {right}:\n    return left",
        "left: int, ...right:\n    return left",
        "left: int, this:\n    return left",
        "left: int, self:\n    return left",
        "left: int, $Right:\n    return left",
        "left: int, 1:\n    return left",
        "left: int, \"right\":\n    return left",
        "left: int, right - > int:\n    return left",
        "left: int, right: int - > int:\n    return left",
        "left: int, right: int: int:\n    return left",
        "left: Int.Name, right:\n    return left",
        "left, right: List[int]:\n    return left",
        "left: List<int>, right:\n    return left",
        "left: $Type, right:\n    return left",
        "left, right: this:\n    return left",
        "left: null, right:\n    return left",
        "left: \"int\", right:\n    return left",
        "left: 1, right:\n    return left",
        "left: int, right\n:\n    return left",
        "left: int,\nright:\n    return left",
        "left\n: int, right:\n    return left",
        "left:\nint, right:\n    return left",
        "left: int\n, right:\n    return left",
        "left: int, right:\nint:\n    return left",
        "left: int, right;:\n    return left",
        "left: int, right # boundary\n:\n    return left",
        "left: int, # boundary\nright:\n    return left",
        "left: # annotation\nint, right:\n    return left",
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
            "def f this: int, right:\n    return right",
            "Expected '(' but found keyword 'this'",
            Position::new(1, 7),
        ),
        (
            "def f\nleft: int, right:\n    return left",
            "Expected '(' but found end of line",
            Position::new(1, 5),
        ),
        (
            "def f; left: int, right:\n    return left",
            "Expected '(' but found ';'",
            Position::new(1, 6),
        ),
        (
            "class Gauge:\n    def reading left: int, right,:\n        return left",
            "Expected '(' but found identifier 'left'",
            Position::new(2, 17),
        ),
        (
            "def f(left: int, right:):\n    return left",
            "Expected a type but found ')'",
            Position::new(1, 25),
        ),
        (
            "def f(a:):\n    return a",
            "Expected a type but found ')'",
            Position::new(1, 10),
        ),
        (
            "def f left: int | str, right:\n    return left",
            "Unexpected character '|'",
            Position::new(1, 17),
        ),
        (
            "def f left: int^, right:\n    return left",
            "Unexpected character '^'",
            Position::new(1, 16),
        ),
        // NUC 59 makes `?` a token, so this nullable-annotation lookalike
        // now fails at the same boundary as other unsupported annotations.
        (
            "def f left: int?, right:\n    return left",
            "Expected '(' but found identifier 'left'",
            Position::new(1, 7),
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(position), "{source}");
    }
}

#[test]
fn every_typed_or_untyped_final_parameter_requires_real_indent_not_inline_or_braced() {
    for header in [
        "left: int, right",
        "left, right: int",
        "left: int, right: int",
        "left: int, right, factor: number",
        "left, right: int, factor",
    ] {
        for suffix in [
            ":",
            ":\n    # comment only",
            ":\nreturn left",
            ":;\n    return left",
            ": return left",
            ": { return left }",
            " { return left }",
            ",:\n    return left",
            "\n:\n    return left",
            " # break\n:\n    return left",
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
}

#[test]
fn qualifying_headers_preserve_exact_body_and_lexical_diagnostics() {
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
            "def f left: int, right:",
            "def f(left: int, right):",
            "def f left, right: int:",
            "def f(left, right: int):",
            "def f left: int, right: int:",
            "def f(left: int, right: int):",
        ] {
            let source = format!("{header}\n    {body}");
            let error = Runtime::check(&source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            assert_eq!(error.message(), message, "{source}");
            assert_eq!(error.position(), Some(position), "{source}");
        }
    }
}

#[test]
fn multistatement_branching_nested_and_inner_inline_or_braced_bodies_keep_explicit_returns() {
    let explicit = r#"
def add(left: int, right: int):
    total = left + right
    return total
def scaled(left: number, right, factor: number):
    return (left + right) * factor
def largest(left, right: number):
    if left > right:
        return left
    return right
def outer(initial: int):
    def combine(left: int, right, scale: int):
        return (left + right) * scale
    return combine(initial, 2, 3)
def noReturn(left: int, right: int):
    left + right
def bareReturn(left, right: int):
    return
assert(add(2, 3), 5)
assert(scaled(2, 3, 4), 20)
assert(largest(7, 3), 7)
assert(largest(2, 9), 9)
assert(outer(4), 18)
assert(noReturn(2, 3), null)
assert(bareReturn(2, 3), null)
"#;
    check_forms(&bare_blocks(explicit), explicit, 7, None);
    for body in [
        "if left: return right\n    return left",
        "if left: { return right }\n    return left",
        "{ return left + right }",
        "return (\n        left +\n        right\n    )",
    ] {
        equivalent(
            &format!("def choose left: int, right:\n    {body}"),
            &format!("def choose(left: int, right):\n    {body}"),
        );
    }
}

#[test]
fn unchecked_annotations_arity_local_shadowing_and_null_callable_deferral_are_unchanged() {
    let explicit = r#"
left = 10
right = 20
def pair(left: int, right: int):
    return [left, right]
def add(left: int, right):
    left += right
    return left
def invoke(operation: Callback, amount: int):
    return operation(amount)
def increment(amount: int) => amount + 1
def operation(amount) => amount + 100
def outer(initial: int):
    def combine(left: int, right, scale: int):
        return (left + right) * scale
    return combine(initial, 2, 3)
def noReturn(left: int, right: int):
    left + right
def bareReturn(left, right: int):
    return
assert(pair(3, 4), [3, 4])
assert(pair(3), [3, null])
assert(pair(), [null, null])
assert(pair(3, 4, 5), [3, 4])
assert(pair("three", true), ["three", true])
assert(add(3, 4), 7)
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
assert(noReturn(2, 3), null)
assert(bareReturn(2, 3), null)
pair(8, 9)
"#;
    let [bare, explicit] = check_forms(&bare_blocks(explicit), explicit, 15, Some("[8,9]"));
    graphs_match(&bare, &explicit);
    for runtime in [bare, explicit] {
        for annotation in ["int", "Callback"] {
            assert!(!runtime.graph.contains(&NodeKey::variable(annotation)));
        }
    }
}

#[test]
fn methods_initializers_receivers_and_explicit_or_bare_super_preserve_adoption_and_identity() {
    let explicit = r#"
class Parent:
    def init(left: int, right: int):
        this.amount = left + right
    def reading(offset: int, scale):
        return self.amount * scale + offset
class Child: Parent
    def init(left: int, right):
        super(left, right)
    def reading(offset, scale: int):
        return self.amount * scale + offset
    def increase(delta: int, extra):
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
    def init(left: int, right):
        super
        self.amount = left + right
tagged = Tagged(3, 5)
assert(tagged.ready, true)
assert(tagged.amount, 8)
"#;
    check_forms(&bare_blocks(explicit), explicit, 10, None);

    // This pair really changes both typed/mixed headers, including bare super.
    let explicit = "class Parent:\n    def init(left: int, right):\n        this.amount = left\nclass Child: Parent\n    def init(left: int, right: int):\n        super\nchild = Child(2, 3)\nassert(child.amount, null)\nassert(child.id, \"child\")\nassert(Parent.length, 0)\nassert(Child.length, 1)";
    check_forms(&bare_blocks(explicit), explicit, 4, None);
}

#[test]
fn objects_templates_and_returned_functions_do_not_gain_capture_or_receivers() {
    let explicit = r#"
def point(x: int, y):
    return {x, y}
def text(left, right: str):
    return `${left},${right}`
def factory(ignored: Unknown, other):
    return (left: int, right) => left + right
p = point(2, 3)
assert(p.x, 2)
assert(p.y, 3)
assert(text(2, 3), "2,3")
callback = factory(10, 20)
assert(callback(2, 3), 5)
"#;
    check_forms(&bare_blocks(explicit), explicit, 4, None);
    for (explicit, message, line) in [
        (
            "def wrap(left: int, right):\n    return { missing }\nwrap(1, 2)",
            "missing is not defined",
            3,
        ),
        (
            "def factory(left: int, right):\n    return value => ({ left, right })\ncallback = factory(1, 2)\ncallback(3)",
            "left is not defined",
            4,
        ),
        (
            "def receiver(left: int, right):\n    return this.amount\nreceiver(1, 2)",
            "this is not defined",
            3,
        ),
        (
            "def receiver(left: int, right):\n    return self.amount\nreceiver(1, 2)",
            "this is not defined",
            3,
        ),
    ] {
        let bare = bare_blocks(explicit);
        assert_ne!(bare, explicit);
        equivalent(&bare, explicit);
        let error = Runtime::new().run(&bare).unwrap_err();
        assert_eq!(error, Runtime::new().run(explicit).unwrap_err());
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(line, 1)));
    }
}

#[test]
fn live_aliases_definition_snapshots_redefinition_graph_keys_and_reasoning_stay_identical() {
    let explicit = r#"
bias = 2
def scaled(left: int, right):
    return left * right + bias
live = scaled
snapshot = scaled.value
result = live(3, 4)
frozen = snapshot(3, 4)
assert(result, 14)
assert(frozen, 14)
def scaled(left: int, right: int):
    return left + right + bias
assert(result, 9)
assert(frozen, 14)
bias = 4
assert(result, 11)
assert(frozen, 16)
result
"#;
    let [mut bare, mut explicit] = check_forms(&bare_blocks(explicit), explicit, 6, Some("11"));
    graphs_match(&bare, &explicit);
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
            &parse("def add left: int, right: int:\n    return left + right").unwrap(),
            None
        )
        .as_str(),
        "block(def add)"
    );
}

#[test]
fn reactive_argument_and_body_inputs_preserve_edges_without_annotation_dependencies() {
    let explicit = r#"
left = 2
right = 3
bias = 1
def total(left: Undeclared, right):
    return left + right + bias
result = total(left, right)
assert(result, 6)
left = 4
assert(result, 8)
right = 5
assert(result, 10)
bias = 2
assert(result, 11)
result
"#;
    let [bare, explicit] = check_forms(&bare_blocks(explicit), explicit, 4, Some("11"));
    graphs_match(&bare, &explicit);
    for runtime in [bare, explicit] {
        assert!(!runtime.graph.contains(&NodeKey::variable("Undeclared")));
    }
}

#[test]
fn caught_failed_redefinition_preserves_rollback_coverage_and_explicit_final_return() {
    let explicit = r#"
source = 3
def reading(left: int, right: int):
    return source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading(left: int, right):
        return missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)
"#;
    let [bare, explicit] = check_forms(&bare_blocks(explicit), explicit, 5, Some("21"));
    graphs_match(&bare, &explicit);
    for runtime in [bare, explicit] {
        assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
    }
}

#[test]
fn malformed_headers_and_uncaught_redefinitions_are_atomic_and_recoverable() {
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, header) in runtimes.iter_mut().zip([
        "def reading left: int, right:",
        "def reading(left: int, right):",
    ]) {
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
            "def reading left: int, right,:\n    return left",
            "def reading left, right: int,:\n    return left",
            "def reading left: int, right:\nreturn left",
            "def reading left: int, right:;\n    return left",
            "def reading left: int,\nright:\n    return left",
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
    graphs_match(&runtimes[0], &runtimes[1]);
}

#[test]
fn inherited_parameter_dependent_local_and_nested_propagation_failures_stay_paired() {
    let explicit = "fee = 2\ndef total(amount: int, other):\n    adjusted = amount + fee\n    return adjusted\ninvoice = total(5, 1)\nassert(invoice, 7)";
    let mut runtimes = check_forms(&bare_blocks(explicit), explicit, 1, None);
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
    graphs_match(&runtimes[0], &runtimes[1]);

    let explicit = "def outer(amount: int, other):\n    def inner(value: int, extra):\n        return value * 2\n    offset = 1\n    if amount > 0:\n        return inner(amount, other) + offset\n    return offset\nassert(outer(3, 1), 7)\nassert(outer(0, 1), 1)";
    let bare = bare_blocks(explicit);
    assert_ne!(bare, explicit);
    equivalent(&bare, explicit);
    let error = Runtime::new().run(&bare).unwrap_err();
    assert_eq!(error, Runtime::new().run(explicit).unwrap_err());
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "amount is not defined");
    assert_eq!(error.position(), Some(Position::new(9, 1)));
}

#[test]
fn legacy_single_multiple_arrow_parenthesized_anonymous_class_and_call_forms_stay_exact() {
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
            &format!("def f{signature}:\n    return 1"),
            &format!("def f{signature} => 1"),
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
            "def f left: int, right => left + right",
            "def f(left: int, right) => left + right",
        ),
        (
            "def f left, right:\n    return left + right",
            "def f(left, right):\n    return left + right",
        ),
    ] {
        equivalent(bare, explicit);
    }
    for source in [
        "left => left",
        "(left: int, right) => left + right",
        "(left, right: str) => { return left + right }",
        "function(left: int, right) { return left + right }",
    ] {
        let statements = parse(source).unwrap();
        let [Stmt::Expression(Expr::Function(function))] = statements.as_slice() else {
            panic!("expected an unchanged anonymous function");
        };
        assert!(function.name.is_none());
    }
    for source in [
        "left: int, right:\n    return left",
        "callback = left: int, right:\n    return left",
        "class Item left: int, right",
    ] {
        assert!(Runtime::check(source).is_err(), "{source}");
    }
    assert_eq!(
        parse("add left right").unwrap(),
        ["add", "left", "right"].map(|name| Stmt::Expression(Expr::Identifier(name.to_string())))
    );
}

#[test]
fn flat_typed_and_mixed_lists_are_uncapped_through_1024_parameters() {
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
            let bare = format!("def many {parameters}:\n    return p{}", count - 1);
            let explicit = format!("def many({parameters}):\n    return p{}", count - 1);
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
            let error = Runtime::check(&format!(
                "def many {parameters},:\n    return p{}",
                count - 1
            ))
            .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), "Expected '(' but found identifier 'p0'");
            assert_eq!(error.position(), Some(Position::new(1, 10)));
        }
    }
}

#[test]
fn inherited_expression_nesting_and_chain_limits_fail_atomically_then_recover() {
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
        let mut errors = Vec::new();
        for header in [
            "def limited left: int, right:",
            "def limited(left: int, right):",
        ] {
            let error = runtime
                .run(&format!("saved = 3\n{header}\n    return {body}"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert!(error.position().is_some());
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
            errors.push(error);
        }
        assert_eq!(errors[0], errors[1]);
    }
    runtime
        .run("def add left: int, right: int:\n    return left + right")
        .unwrap();
    assert_eq!(runtime.run("add(2, 3)").unwrap(), Value::Number(5.0));
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
fn native_repl_keeps_canonical_graphs_real_indented_bodies_and_stdin_eof() {
    let explicit = ".json on\n.graph on\nbias = 2\ndef scaled(left: int, right: int):\n    return left * right + bias\n\nresult = scaled(3, 4)\nbias = 4\nassert(result, 16)\nresult\n.exit\n";
    let bare = bare_blocks(explicit);
    assert_ne!(bare, explicit);
    let outputs = [repl(&bare), repl(explicit)];
    for output in &outputs {
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.lines().any(|line| line == "16.0"), "{stdout}");
        assert!(stdout.contains("scaled [function]"), "{stdout}");
        assert!(!stdout.contains("def scaled left"), "{stdout}");
    }
    assert_eq!(outputs[0].stdout, outputs[1].stdout);

    let explicit = ".json on\n.graph on\ndef add(left: int, right):\n    return left + right";
    let bare = bare_blocks(explicit);
    assert_ne!(bare, explicit);
    let outputs = [repl(&bare), repl(explicit)];
    for output in &outputs {
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("add [function]"));
    }
    assert_eq!(outputs[0].stdout, outputs[1].stdout);
}

#[test]
fn native_repl_recovers_after_invalid_same_prefix_and_body_limit_errors() {
    let source = format!(
        ".json on\nbias = 2\ndef scaled left: int, right:\n    return left * right + bias\n\nresult = scaled(3, 4)\n\
         def scaled left: int, right,:\n    return left\n\n\
         def limited left: int, right:\n    return left{}\n\n\
         bias = 4\nassert(result, 16)\nresult",
        " + 1".repeat(200)
    );
    let output = repl(&source);
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 2, "{stderr}");
    assert!(
        stderr
            .contains("SyntaxError: Expected '(' but found identifier 'left' (line 1, column 12)"),
        "{stderr}"
    );
    assert!(
        stderr.contains("SyntaxError: Expression has too many operands"),
        "{stderr}"
    );
    assert!(!stderr.contains("assertion failed"), "{stderr}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.lines().any(|line| line == "16.0"), "{stdout}");
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

const ACCEPTED_REVISION: &str = "c007624a00d1f7d949de3bb1e2c109c53b6f36a4";

fn accepted_source(path: &str) -> String {
    let output = Command::new("git")
        .args(["--no-pager", "show", &format!("{ACCEPTED_REVISION}:{path}")])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("requires Git and pinned accepted objects; see the ignored audit run command");
    assert!(
        output.status.success(),
        "{path}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("accepted sources are UTF-8")
}

#[test]
#[ignore = "requires Git history at c007624; run cargo test --test bare_typed_multiple_indented_blocks -- --ignored"]
fn historical_corpus_and_eight_appended_counterparts_keep_complete_asts_and_protected_assertions() {
    // Preserve NUC32's equivalent sole-return layout while independently
    // checking this proposal's parameter-parenthesis omission.
    let header = Regex::new(
        r"(?m)^([ \t]*def [A-Za-z_]\w*) ([A-Za-z_]\w*(?:: [A-Za-z_]\w*)?(?:, [A-Za-z_]\w*(?:: [A-Za-z_]\w*)?)+)(:| =>)$",
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
            323,
            5,
        ),
        (
            "tests/reference.md",
            include_str!("reference.md"),
            true,
            65,
            1,
        ),
        ("README.md", include_str!("../README.md"), false, 16, 1),
        (
            "docs/examples.md",
            include_str!("../docs/examples.md"),
            false,
            71,
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
            // Historical cases can legitimately be identical; do not require
            // a new-syntax difference in this generic compatibility audit.
            let statements = equivalent(&after.source, &before.source);
            assertions += common::assertions::protected_assertions(&statements).len();
            historical += 1;
        }
        for case in own_additions {
            let normalized = named_arrow_headers::legacy_block_headers(
                &named_arrow_headers::legacy_headers(&case.source),
            );
            let explicit = header.replace_all(&normalized, "$1($2)$3");
            assert_ne!(normalized, explicit, "{}", case.title);
            let statements = equivalent(&case.source, &explicit);
            let count = common::assertions::protected_assertions(&statements).len();
            check_forms(&case.source, &explicit, count, case.expected.as_deref());
            assertions += count;
            additions += 1;
        }
    }
    assert_eq!(historical, 475);
    assert_eq!(additions, 8);
    assert_eq!(assertions, 1835);
}

#[test]
#[ignore = "requires Git history at c007624; run cargo test --test bare_typed_multiple_indented_blocks -- --ignored"]
fn all_synthesized_asts_fields_and_order_remain_exact() {
    let indented_arrow =
        Regex::new(r"(?m)^([ \t]*def [^\n]+) =>([ \t]*(?:#[^\n]*)?)\n([ \t]+)([^\n]+)").unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut records = 0;
    let mut sets = 0;
    let mut rewritten = 0;
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
            let before_code = before_code.as_str().unwrap();
            let after_code = after_code.as_str().unwrap();
            equivalent(after_code, before_code);
            // Count this proposal's header rewrites, not later optional
            // sole-return or final-else layouts. The actual full AST and all metadata
            // above still compare directly with the pinned historical source.
            let comma_headers = named_arrow_headers::legacy_block_headers(
                &named_arrow_headers::legacy_headers(after_code),
            );
            let mut comparable = indented_arrow
                .replace_all(&comma_headers, "$1:$2\n${3}return $4")
                .into_owned();
            let tokens = tokenize(&comparable).unwrap();
            for (index, token) in tokens.iter().enumerate().rev() {
                if token.token != Token::Keyword(Keyword::Else)
                    || !matches!(tokens[index + 1].token, Token::Newline | Token::Indent)
                    || !tokens[index + 1..]
                        .iter()
                        .find(|next| !matches!(next.token, Token::Newline | Token::Semicolon))
                        .is_some_and(|next| next.token == Token::Indent)
                {
                    continue;
                }
                let line_start = comparable
                    .split_inclusive('\n')
                    .take(token.position.line - 1)
                    .map(str::len)
                    .sum::<usize>();
                let column = comparable[line_start..]
                    .char_indices()
                    .nth(token.position.column + 3)
                    .unwrap()
                    .0;
                comparable.insert(line_start + column, ':');
            }
            rewritten += usize::from(before_code != comparable);
            records += 1;
        }
        sets += 1;
    }
    assert_eq!(sets, 21);
    assert_eq!(records, 1081);
    // No existing synthesized record has an eligible typed multiple block.
    assert_eq!(rewritten, 0);
}
