//! NUC37 is optional header punctuation, not a new function or call mechanism.
//! Ordinary checks embed accepted counterparts and never need Git or sessions.

mod common;
#[path = "support/named_arrow_headers.rs"]
mod named_arrow_headers;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use named_arrow_headers::{comma_free_headers, legacy_headers};
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{
    Expr, FunctionBody, Parameter, Stmt, parse, parse_expression, parse_program,
};
use nucleoid::lang::evaluation::Flow;
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::statement;
use nucleoid::value::ObjectId;
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn equivalent(candidate: &str, accepted: &str) -> Vec<Stmt> {
    let actual = parse(candidate).unwrap_or_else(|error| panic!("{candidate}\n{error}"));
    let expected = parse(accepted).unwrap_or_else(|error| panic!("{accepted}\n{error}"));
    assert_eq!(actual, expected, "{candidate}");
    assert_eq!(
        generate_all(&actual),
        generate_all(&expected),
        "{candidate}"
    );
    actual
}

fn graphs_match(actual: &Runtime, expected: &Runtime) {
    runtime_equivalence::assert_equivalent(actual, None, expected, None);
    assert!(actual.graph.keys().eq(expected.graph.keys()));
    for node in actual.graph.nodes() {
        let other = expected.graph.retrieve(&node.key).unwrap();
        assert_eq!(node.kind, other.kind, "{}", node.key);
        assert_eq!(node.dependencies, other.dependencies, "{}", node.key);
        assert_eq!(node.dependents, other.dependents, "{}", node.key);
        assert_eq!(node.sequence, other.sequence, "{}", node.key);
        assert_eq!(
            format!("{:?}", node.node),
            format!("{:?}", other.node),
            "{}",
            node.key
        );
        assert!(
            actual
                .graph
                .shape_dependencies(&node.key)
                .eq(expected.graph.shape_dependencies(&node.key))
        );
    }
}

fn check_pair(
    candidate: &str,
    accepted: &str,
    assertions: usize,
    expected: Option<&str>,
) -> [Runtime; 2] {
    assert_ne!(candidate, accepted, "a pair must exercise comma omission");
    let statements = equivalent(candidate, accepted);
    let protected = common::assertions::protected_assertions(&statements);
    assert_eq!(protected.len(), assertions);
    assert_eq!(
        protected,
        common::assertions::protected_assertions(&parse(accepted).unwrap())
    );
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let mut values = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([candidate, accepted]) {
        let value = common::check_with_count(
            runtime,
            &common::Case {
                title: "comma-free named-arrow pair".to_string(),
                source: source.to_string(),
                expected: expected.map(str::to_string),
            },
            assertions,
        )
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
        assert_eq!(runtime.assertions_run(), assertions, "{source}");
        values.push(value);
    }
    runtime_equivalence::assert_equivalent(
        &runtimes[0],
        Some(&values[0]),
        &runtimes[1],
        Some(&values[1]),
    );
    runtimes
}

#[test]
fn complete_identifier_runs_keep_order_none_annotations_name_and_return_lowering() {
    for names in [
        vec!["left", "right"],
        vec!["left", "right", "factor"],
        vec!["type", "value", "_private", "A1", "why", "affects", "model"],
        vec!["same", "same"],
    ] {
        let candidate = format!("def calculate {} => 1", names.join(" "));
        let accepted = format!("def calculate {} => 1", names.join(", "));
        let statements = equivalent(&candidate, &accepted);
        equivalent(
            &candidate,
            &format!("def calculate({}) => 1", names.join(", ")),
        );
        equivalent(
            &candidate,
            &format!("def calculate({}):\n    return 1", names.join(", ")),
        );
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected exactly one named function");
        };
        assert_eq!(function.name.as_deref(), Some("calculate"));
        assert_eq!(
            function.parameters,
            names
                .iter()
                .map(|name| Parameter {
                    name: (*name).to_string(),
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
    check_pair(
        "def duplicate same same => same\nassert(duplicate(2, 3), 3)",
        "def duplicate same, same => same\nassert(duplicate(2, 3), 3)",
        1,
        None,
    );
}

#[test]
fn flat_two_three_and_1024_parameter_headers_keep_existing_uncapped_behavior() {
    for count in [2, 3, 129, 1024] {
        let names = (0..count).map(|i| format!("p{i}")).collect::<Vec<_>>();
        let candidate = format!("def many {} => p{}", names.join(" "), count - 1);
        let accepted = format!("def many {} => p{}", names.join(", "), count - 1);
        let statements = equivalent(&candidate, &accepted);
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected exactly one named function");
        };
        assert_eq!(function.parameters.len(), count);
        for (parameter, name) in function.parameters.iter().zip(names) {
            assert_eq!(parameter.name, name);
            assert_eq!(parameter.type_name, None);
        }
        let call = format!("many({})", vec!["7"; count].join(", "));
        for source in [&candidate, &accepted] {
            let mut runtime = Runtime::new();
            runtime.run(source).unwrap();
            assert_eq!(runtime.run(&call).unwrap(), Value::Number(7.0));
        }
    }
}

#[test]
fn every_expression_uses_the_existing_named_arrow_body_not_an_anonymous_block() {
    for body in [
        "left + right * factor",
        "left = right",
        "left += right",
        "left ||= right",
        "{ left, right }",
        "[left, right]",
        "`total=${left + right}`",
        "value => value * 2",
        "(left, right) => ({ left, right })",
        "(left: int, right) => { return left + right }",
        "function(value) { return value }",
        "why left",
    ] {
        for candidate in [
            format!("def calculate left right => {body}"),
            format!("def calculate left right =>\n    {body}"),
        ] {
            let statements =
                equivalent(&candidate, &format!("def calculate left, right => {body}"));
            let [Stmt::Function(function)] = statements.as_slice() else {
                panic!("expected a named function");
            };
            assert_eq!(
                function.body,
                FunctionBody::Block(vec![Stmt::Return(Some(parse_expression(body).unwrap()))])
            );
        }
    }
}

#[test]
fn existing_lexer_tokens_and_physical_statement_positions_are_unchanged() {
    assert_eq!(
        tokenize("def add left right => left + right")
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        [
            Token::Keyword(Keyword::Def),
            Token::Identifier("add".to_string()),
            Token::Identifier("left".to_string()),
            Token::Identifier("right".to_string()),
            Token::Arrow,
            Token::Identifier("left".to_string()),
            Token::Plus,
            Token::Identifier("right".to_string()),
            Token::Eof,
        ]
    );
    for source in [
        "def add left right=>left + right",
        "def add\tleft\tright\t=> left + right",
        "def add left right => (left + right)",
        "def add left right => (\n    left +\n    right\n)",
        "def add left right =>\r\n\t(left + right)",
    ] {
        equivalent(source, "def add left, right => left + right");
    }
    for suffix in ["\nnext = 3", "; next = 3", " # body\nnext = 3"] {
        equivalent(
            &format!("def add left right => left + right{suffix}"),
            &format!("def add left, right => left + right{suffix}"),
        );
    }
    equivalent(
        "{ def add left right => left + right; next = 3 }",
        "{ def add left, right => left + right; next = 3 }",
    );
    let source = "def add left right => # header\r\n\tleft + right\r\nnext = 3";
    assert_eq!(
        parse_program(source).unwrap().positions,
        [Position::new(1, 1), Position::new(3, 1)]
    );
    assert_eq!(
        parse_program(source).unwrap().positions,
        parse_program(&legacy_headers(source)).unwrap().positions
    );
}

#[test]
fn whitespace_comments_parentheses_and_eof_keep_nuc32_body_layout() {
    for layout in [
        "\n    ",
        " # header\n    ",
        "\n\n    # body\n    ",
        "\r\n\t",
        "\n;\n    ",
        "\n    ;\n    ",
    ] {
        for ending in ["", "\n", ";\n", "\n    ;\n", " # end\n"] {
            let source = format!("def f left right =>{layout}left + right{ending}\nnext = 3");
            equivalent(
                &source,
                "def f left, right:\n    return left + right\nnext = 3",
            );
        }
    }
    for body in [
        "(\n        left +\n        right\n    )",
        "{ left, right }",
        "(\r\n\t\tleft + right\r\n\t)",
    ] {
        let source = format!("def f left right =>\n    {body}\nnext = 3");
        equivalent(&source, &legacy_headers(&source));
    }
}

#[test]
fn incomplete_mixed_typed_or_block_prefixes_fall_back_at_the_original_identifier() {
    for header in [
        "left right",
        "left right, third => left",
        "left, right third => left",
        "left right, => left",
        "left: int right => left",
        "left right: int => left",
        "left: int right: int => left",
        "left right, third: int => left",
        "left right: => left",
        "left right: Type.Member => left",
        "left right: List[int] => left",
        "left right - > int => left",
        "left = 1 right => left",
        "left right = 1 => left",
        "left [right] => left",
        "left {right} => left",
        "left ...right => left",
        "left this => left",
        "left self => left",
        "left super => left",
        "left null => left",
        "left $Right => left",
        "left 1 => left",
        "left \"right\" => left",
        "left right:\nreturn left",
        "left right: return left",
        "left right: { return left }",
        "left right:;\n    return left",
        "left right\n=> left",
        "left\nright => left",
        "left\rright\n=> left",
        "left right; => left",
        "left; right => left",
        "left right # boundary\n=> left",
        "left # boundary\nright => left",
        "left right\r\n\t=> left",
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
        assert_eq!(legacy_headers(&source), source);
    }
}

#[test]
fn later_comma_free_blocks_replace_the_old_negative_with_full_equivalence() {
    let candidate = "def f left right:\n    return left\nassert(f(2, 3), 2)";
    let accepted = "def f left, right:\n    return left\nassert(f(2, 3), 2)";
    check_pair(candidate, accepted, 1, None);
    // These exports must remain arrow-only even after the block extension.
    assert_eq!(legacy_headers(candidate), candidate);
    assert_eq!(comma_free_headers(accepted), accepted);
}

#[test]
fn nonidentifier_nullary_and_parenthesized_errors_keep_the_old_messages() {
    for (source, message, line, column) in [
        (
            "def f this right => right",
            "Expected '(' but found keyword 'this'",
            1,
            7,
        ),
        (
            "def f self right => right",
            "Expected '(' but found keyword 'this'",
            1,
            7,
        ),
        (
            "def f $Left right => right",
            "Expected '(' but found '$Left'",
            1,
            7,
        ),
        (
            "def f 1 right => right",
            "Expected '(' but found number 1",
            1,
            7,
        ),
        (
            "def f\nleft right => left",
            "Expected '(' but found end of line",
            1,
            5,
        ),
        (
            "def f; left right => left",
            "Expected '(' but found ';'",
            1,
            6,
        ),
        ("def f => 1", "Expected '(' but found '=>'", 1, 7),
        ("def f:\n    1", "Expected '(' but found ':'", 1, 6),
        (
            "def f(left right) => left",
            "Expected ')' but found identifier 'right'",
            1,
            12,
        ),
        (
            "def f(left, right:) => left",
            "Expected a type but found ')'",
            1,
            21,
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
fn inline_expression_and_lexical_errors_report_the_actual_shorter_header_positions() {
    for (body, message, new_position, old_position) in [
        (
            "",
            "Unexpected end of input",
            Position::new(1, 20),
            Position::new(1, 21),
        ),
        (
            "\n",
            "Unexpected end of line",
            Position::new(2, 1),
            Position::new(2, 1),
        ),
        (
            ";\n    left",
            "Unexpected ';'",
            Position::new(1, 20),
            Position::new(1, 21),
        ),
        (
            " (left +)",
            "Unexpected ')'",
            Position::new(1, 29),
            Position::new(1, 30),
        ),
        (
            " { return left }",
            "Expected a key but found keyword 'return'",
            Position::new(1, 30),
            Position::new(1, 31),
        ),
        (
            " { left = 1 }",
            "Expected ':' but found '='",
            Position::new(1, 28),
            Position::new(1, 29),
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
            Position::new(1, 21),
            Position::new(1, 22),
        ),
    ] {
        for (source, position) in [
            (format!("def f left right =>{body}"), new_position),
            (format!("def f left, right =>{body}"), old_position),
        ] {
            let error = Runtime::check(&source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            assert_eq!(error.message(), message, "{source}");
            assert_eq!(error.position(), Some(position), "{source}");
        }
    }
}

#[test]
fn genuinely_indented_bodies_keep_exact_existing_errors_and_one_expression() {
    for (body, message, line, column) in [
        ("\nleft", "Unexpected end of line", 2, 1),
        (
            "\n    left\n    left + 1",
            "Expected the end of a block but found identifier 'left'",
            3,
            5,
        ),
        (
            "\n    left; right",
            "Expected the end of a block but found identifier 'right'",
            2,
            11,
        ),
        (
            "\n    left 1",
            "Expected the end of a block but found number 1",
            2,
            10,
        ),
        (
            "\n    left\n        1",
            "Expected the end of a block but found an indented block",
            3,
            9,
        ),
        ("\n    return left", "Unexpected keyword 'return'", 2, 12),
        ("\n    (left +)", "Unexpected ')'", 2, 13),
        (
            "\n    { return left }",
            "Expected a key but found keyword 'return'",
            2,
            14,
        ),
        ("\n    @", "Unexpected character '@'", 2, 5),
        (
            "\r\n\tleft\r\n\tright",
            "Expected the end of a block but found identifier 'right'",
            3,
            2,
        ),
    ] {
        for header in ["def f left right =>", "def f left, right =>"] {
            let source = format!("{header}{body}");
            let error = Runtime::check(&source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            assert_eq!(error.message(), message, "{source}");
            assert_eq!(
                error.position(),
                Some(Position::new(line, column)),
                "{source}"
            );
        }
    }
}

#[test]
fn test_only_normalization_is_token_qualified_and_preserves_identifier_boundaries() {
    let accepted = "# def fake left,right => left\ntext = \"def fake left,right => left\"\ndef add left,right=>{left,right}\ndef typed left: int, right => left\ndef block left, right:\n    return left\ncallback = (left,right) => left + right";
    let candidate = "# def fake left,right => left\ntext = \"def fake left,right => left\"\ndef add left right=>{left,right}\ndef typed left: int, right => left\ndef block left, right:\n    return left\ncallback = (left,right) => left + right";
    assert_eq!(comma_free_headers(accepted), candidate);
    assert_eq!(
        legacy_headers(candidate),
        accepted.replace("def add left,right=>", "def add left, right=>")
    );
    equivalent(candidate, accepted);
    for (accepted, candidate) in [
        ("def f left,right=>left", "def f left right=>left"),
        ("def f left ,right=>left", "def f left right=>left"),
        ("def f left,\tright=>left", "def f left\tright=>left"),
        ("def f left , right=>left", "def f left  right=>left"),
    ] {
        assert_eq!(comma_free_headers(accepted), candidate);
        equivalent(candidate, accepted);
    }
    for source in [
        "def f left right, third => left",
        "def f left, right third => left",
        "def f left right: int => left",
        "def f left right:\n    return left",
        "def f(left, right) => left",
        "def f left # boundary\nright => left",
    ] {
        assert_eq!(legacy_headers(source), source);
        assert_eq!(comma_free_headers(source), source);
    }
}

#[test]
fn legacy_parameters_blocks_anonymous_classes_and_ordinary_expressions_stay_available() {
    for signature in [
        "()",
        "(left)",
        "(left: int)",
        "(left, right)",
        "(left: int, right)",
        "(left: int, right: str,)",
        "(\n    left,\n    right,\n)",
        " left",
        " left: int",
        " left, right",
        " left: int, right",
        " left, right: int",
        " left: int, right: int",
    ] {
        equivalent(
            &format!("def f{signature} => 1"),
            &format!("def f{signature}:\n    return 1"),
        );
        let source = format!("def f{signature} =>\n    1\nassert(f(), 1)");
        let mut runtime = Runtime::new();
        runtime.run(&source).unwrap();
        assert_eq!(runtime.assertions_run(), 1);
        assert!(runtime.take_assertions().is_empty());
    }
    for signature in ["()", "(left, right)", "(left: int, right: int)"] {
        assert!(parse(&format!("def f{signature}: pass")).is_ok());
        assert!(parse(&format!("def f{signature}: {{ return 1 }}")).is_ok());
        assert!(parse(&format!("class Item{signature}")).is_ok());
    }
    for source in [
        "left => left",
        "(left, right) => left + right",
        "(left: int, right) => { return left + right }",
        "function(left, right) { return left + right }",
    ] {
        let Expr::Function(function) = parse_expression(source).unwrap() else {
            panic!("expected an anonymous function");
        };
        assert!(function.name.is_none());
    }
    // The pre-existing parser already reads this as an identifier expression
    // followed by a single-parameter anonymous arrow, not a new two-name arrow.
    equivalent("left right => left", "left\nright => left");
    let ordinary = parse("left right => left").unwrap();
    let [
        Stmt::Expression(Expr::Identifier(left)),
        Stmt::Expression(Expr::Function(function)),
    ] = ordinary.as_slice()
    else {
        panic!("expected two ordinary legacy expressions");
    };
    assert_eq!(left, "left");
    assert!(function.name.is_none());
    assert_eq!(function.parameters.len(), 1);
    assert_eq!(function.parameters[0].name, "right");
    for source in ["left, right => left", "class Item left right"] {
        assert!(parse(source).is_err(), "{source}");
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
fn binding_arity_null_extras_shadowing_and_local_parameter_assignment_stay_paired() {
    let accepted = r#"
left = 10
right = 20
def pair left, right => [left, right]
def assign left, right => left += right
def operation number => 99
def apply operation, number => operation(number)
def outer(initial):
    def combine left, right, scale => (left + right) * scale
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
    check_pair(&comma_free_headers(accepted), accepted, 12, None);
}

#[test]
fn methods_initializers_receivers_super_and_named_adoption_share_the_old_runtime_path() {
    let accepted = r#"
class Parent:
    def init left, right => this.amount = left + right
    def reading offset, scale => self.amount * scale + offset
class Child: Parent
    def init left, right => super(left, right)
    def reading offset, scale => self.amount * scale + offset
    def increase delta, extra => this.amount += delta + extra
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
    check_pair(&comma_free_headers(accepted), accepted, 8, None);
    let accepted = r#"
class Parent:
    def init():
        this.amount = 7
class Child: Parent
    def init left, right => super
    def reading left, right => self.amount + left + right
child = Child(2, 3)
assert(child.id, "child")
assert(child.amount, 7)
assert(child.reading(2, 3), 12)
"#;
    check_pair(&comma_free_headers(accepted), accepted, 3, None);
}

#[test]
fn returned_anonymous_functions_keep_their_commas_and_inherited_noncapture_errors() {
    let accepted = r#"
def coordinates x, y => {x, y}
def factory ignored, other => (left, right) => left + right
point = coordinates(2, 3)
assert(point.x, 2)
assert(point.y, 3)
callback = factory(10, 20)
assert(callback(2, 3), 5)
"#;
    let candidate = comma_free_headers(accepted);
    assert!(candidate.contains("(left, right) => left + right"));
    check_pair(&candidate, accepted, 3, None);
    for (accepted, message, line) in [
        (
            "def wrap left, right => { missing }\nwrap(1, 2)",
            "missing is not defined",
            2,
        ),
        (
            "def factory left, right => value => ({ left, right })\ncallback = factory(1, 2)\ncallback(3)",
            "left is not defined",
            3,
        ),
        (
            "def receiver left, right => this.amount\nreceiver(1, 2)",
            "this is not defined",
            2,
        ),
        (
            "def receiver left, right => self.amount\nreceiver(1, 2)",
            "this is not defined",
            2,
        ),
    ] {
        let candidate = comma_free_headers(accepted);
        equivalent(&candidate, accepted);
        let error = Runtime::new().run(&candidate).unwrap_err();
        assert_eq!(error, Runtime::new().run(accepted).unwrap_err());
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(line, 1)));
    }
}

#[test]
fn live_aliases_value_snapshots_redefinition_and_graph_reasoning_stay_identical() {
    let accepted = r#"
bias = 2
def scaled left, right => left * right + bias
live = scaled
snapshot = scaled.value
result = live(3, 4)
frozen = snapshot(3, 4)
assert(result, 14)
assert(frozen, 14)
def scaled left, right => left + right + bias
assert(result, 9)
assert(frozen, 14)
bias = 4
assert(result, 11)
assert(frozen, 16)
"#;
    let [mut actual, mut expected] = check_pair(&comma_free_headers(accepted), accepted, 6, None);
    for source in ["why result", "model |> why"] {
        let a = actual.run(source).unwrap();
        let b = expected.run(source).unwrap();
        assert_eq!(
            actual.serialize_json(&a).unwrap(),
            expected.serialize_json(&b).unwrap()
        );
    }
    assert_eq!(
        NodeKey::block(&parse("def add left right => left + right").unwrap(), None).as_str(),
        "block(def add)"
    );
}

#[test]
fn uncaught_redefinitions_and_invalid_header_transactions_roll_back_and_recover() {
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, header) in runtimes
        .iter_mut()
        .zip(["def reading left right =>", "def reading left, right =>"])
    {
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
            "def reading left right, extra => left",
            "def reading left right: int => left",
            "def reading left right:\nreturn left",
            "def reading left\nright => left",
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
fn body_resource_limits_still_fail_atomically_without_becoming_header_limits() {
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
        for header in ["def limited left right =>", "def limited left, right =>"] {
            let mut runtime = Runtime::new();
            runtime.run("saved = 7").unwrap();
            let error = runtime
                .run(&format!("saved = 3\n{header} {body}"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert_eq!(error.position().unwrap().line, 2);
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
            runtime.run("def add left right => left + right").unwrap();
            assert_eq!(runtime.run("add(2, 3)").unwrap(), Value::Number(5.0));
        }
    }
}

#[test]
fn runtime_pairing_preserves_generated_identity_aliases_all_state_and_observable_values() {
    let accepted = "def objects left, right => [{ left }, { right }]\nobjects(2, 3)";
    check_pair(&comma_free_headers(accepted), accepted, 0, None);
    let mut actual = Runtime::new();
    let mut expected = Runtime::new();
    let a = actual
        .run("first = { amount: 1 }\nsecond = first\n[first, second]")
        .unwrap();
    let b = expected
        .run("first = { amount: 1 }\nsecond = first\n[first, second]")
        .unwrap();
    runtime_equivalence::assert_equivalent(&actual, Some(&a), &expected, Some(&b));
    expected.run("first.amount = 2").unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runtime_equivalence::assert_equivalent(&actual, Some(&a), &expected, Some(&b));
        }))
        .is_err(),
        "state/property differences must never be normalized away"
    );
    let mut different_alias = Runtime::new();
    let c = different_alias
        .run("first = { amount: 1 }\nsecond = { amount: 1 }\n[first, second]")
        .unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runtime_equivalence::assert_equivalent(&actual, Some(&a), &different_alias, Some(&c));
        }))
        .is_err(),
        "distinct objects must not collapse into an alias"
    );
}

#[test]
fn compile_node_and_frame_apis_keep_caller_parameters_and_receiver_context() {
    for expression in ["left + right", "left += right", "missing"] {
        let candidate = format!("def f left right =>\n    {expression}");
        let accepted = legacy_headers(&candidate);
        let program = statement::compile(&candidate).unwrap();
        assert_eq!(program.statements, parse(&accepted).unwrap());
        assert_eq!(program.positions, [Position::new(1, 1)]);
        for source in [&candidate, &accepted] {
            let mut runtime = Runtime::new();
            let mut scope = Scope::new();
            scope.push();
            scope.declare("left", Value::Number(99.0));
            scope.declare("right", Value::Number(88.0));
            let receiver = ObjectId::named("caller");
            scope.set_this(Some(receiver.clone()));
            scope.set_instance(Some(receiver.clone()));
            let statements = parse(source).unwrap();
            let mut definition = Nuc::convert(&mut runtime, &mut scope, &statements[0]).unwrap();
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

fn repl(source: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(source).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn native_stdin_repl_keeps_byte_exact_blank_commit_crlf_tabs_methods_and_eof() {
    for candidate in [
        ".json on\ndef add left right => left + right\nadd(2, 3)",
        ".json on\ndef add left right => # header\n    left + right\n\nadd(2, 3)",
        ".json on\r\ndef add left right => # header\r\n\t(left + right)\r\n\r\nadd(2, 3)",
        ".json on\nclass Item:\n    def init left right =>\n        this.amount = left + right\n    def reading offset scale =>\n        self.amount * scale + offset\n\nitem = Item(2, 3)\nitem.reading(1, 2)",
    ] {
        let accepted = legacy_headers(candidate);
        let outputs = [repl(candidate.as_bytes()), repl(accepted.as_bytes())];
        for output in &outputs {
            assert!(output.status.success(), "{candidate}\n{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.lines().any(|line| line == "5.0" || line == "11.0"),
                "{stdout}"
            );
        }
        assert_eq!(outputs[0].stdout, outputs[1].stdout);
    }
    for ending in ["", "\n", "\r\n"] {
        let candidate = format!(
            ".json on\n.graph on\ndef add left right => # header\n    left + right{ending}"
        );
        let accepted = legacy_headers(&candidate);
        let outputs = [repl(candidate.as_bytes()), repl(accepted.as_bytes())];
        for output in &outputs {
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            assert!(String::from_utf8_lossy(&output.stdout).contains("add [function]"));
        }
        assert_eq!(outputs[0].stdout, outputs[1].stdout);
    }
}

#[test]
fn native_repl_header_line_parenthesis_keeps_the_inherited_noncontinuation_errors() {
    let candidate = ".json on\ndef add left right => (\n    left + right\n)\nadd(2, 3)";
    let accepted = legacy_headers(candidate);
    let outputs = [repl(candidate.as_bytes()), repl(accepted.as_bytes())];
    assert_eq!(outputs[0].status.code(), outputs[1].status.code());
    assert_eq!(outputs[0].stdout, outputs[1].stdout);
    assert_eq!(outputs[0].stderr, outputs[1].stderr);
    assert_eq!(
        String::from_utf8_lossy(&outputs[0].stderr)
            .lines()
            .collect::<Vec<_>>(),
        [
            "SyntaxError: Unexpected end of input (line 2, column 1)",
            "ReferenceError: left is not defined (line 1, column 5)",
            "SyntaxError: Unexpected ')' (line 1, column 1)",
            "ReferenceError: add is not defined (line 1, column 1)",
        ]
    );
}

#[test]
fn native_stdin_structured_body_errors_recover_the_previous_definition() {
    let source = b".json on\ndef add left right =>\n    left + right\n\nadd(2, 3)\ndef add left right =>\n    left\n    right\n\nadd(3, 4)\ndef add left right =>\nleft\n\nadd(4, 5)";
    let output = repl(source);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        [
            "SyntaxError: Expected the end of a block but found identifier 'right' (line 3, column 5)",
            "SyntaxError: Unexpected end of line (line 2, column 1)",
        ]
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    for expected in ["5.0", "7.0", "9.0"] {
        assert!(stdout.lines().any(|line| line == expected), "{stdout}");
    }
    let output = repl(b"def f left right =>\r\n\tleft\r\n\tright");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim_end(),
        "SyntaxError: Expected the end of a block but found identifier 'right' (line 3, column 2)"
    );
}

#[test]
fn native_file_executes_each_sealed_pair_and_preserves_physical_body_diagnostics() {
    let directory = std::env::temp_dir().join(format!("nucleoid-nuc37-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("program.nuc");
    for source in [
        b"def add left right => (\n    left +\n    right\n)\nadd(2, 3)".as_slice(),
        b"def add left right =>\r\n\t(left + right)\r\nadd(2, 3)".as_slice(),
    ] {
        let accepted = legacy_headers(std::str::from_utf8(source).unwrap());
        let mut previous = None;
        for bytes in [source, accepted.as_bytes()] {
            fs::write(&path, bytes).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
                .args(["--json", "--graph"])
                .arg(&path)
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            assert_eq!(
                String::from_utf8_lossy(&output.stdout).lines().next(),
                Some("5.0")
            );
            if let Some(previous) = &previous {
                assert_eq!(&output.stdout, previous);
            } else {
                previous = Some(output.stdout);
            }
        }
    }
    for own in OWN_CASES {
        let candidate = comma_free_headers(own.accepted);
        let mut previous = None;
        for source in [&candidate, own.accepted] {
            fs::write(&path, source.as_bytes()).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
                .args(["--json", "--graph"])
                .arg(&path)
                .output()
                .unwrap();
            assert!(output.status.success(), "{}\n{output:?}", own.title);
            assert!(output.stderr.is_empty(), "{output:?}");
            if let Some(expected) = own.expected {
                assert_eq!(
                    serde_json::from_str::<f64>(
                        String::from_utf8_lossy(&output.stdout)
                            .lines()
                            .next()
                            .unwrap()
                    )
                    .unwrap(),
                    serde_json::from_str::<f64>(expected).unwrap()
                );
            }
            if let Some(previous) = &previous {
                assert_eq!(&output.stdout, previous, "{}", own.title);
            } else {
                previous = Some(output.stdout);
            }
        }
    }
    for (source, message) in [
        (
            b"def f left right =>\n    left\n    right".as_slice(),
            "SyntaxError: Expected the end of a block but found identifier 'right' (line 3, column 5)",
        ),
        (
            b"def f left right =>\r\n\tleft\r\n\tright".as_slice(),
            "SyntaxError: Expected the end of a block but found identifier 'right' (line 3, column 2)",
        ),
        (
            b"def f left right =>\nleft".as_slice(),
            "SyntaxError: Unexpected end of line (line 2, column 1)",
        ),
        (
            b"def f left right =>\n    missing\nf(2, 3)".as_slice(),
            "ReferenceError: missing is not defined (line 3, column 1)",
        ),
        (
            b"def f left right =>".as_slice(),
            "SyntaxError: Unexpected end of input (line 1, column 20)",
        ),
    ] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg(&path)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap().trim_end(),
            message
        );
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn positive_gitless_probe_when_the_full_new_suite_is_run_with_restricted_path() {
    if std::env::var_os("NUC37_REQUIRE_GITLESS").is_none() {
        return;
    }
    let error = Command::new("git").arg("--version").output().unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    // Rust's Windows PATH resolver can synthesize NotFound without an OS
    // code. The native launcher also probes the same restricted environment
    // and requires CreateProcess's Win32 error 2 before running these tests.
    #[cfg(windows)]
    assert!(matches!(error.raw_os_error(), None | Some(2)), "{error}");
}

struct OwnCase {
    document: &'static str,
    title: &'static str,
    assertions: usize,
    expected: Option<&'static str>,
    accepted: &'static str,
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
fn eight_sealed_identities_comments_order_30_assertions_return21_and_coverage_are_protected() {
    let mut checked = 0;
    let mut assertions = 0;
    let mut returns = Vec::new();
    for (document, mut cases) in documents() {
        let fixtures = OWN_CASES
            .iter()
            .filter(|own| own.document == document)
            .collect::<Vec<_>>();
        let selected = |cases: &[common::Case]| {
            cases
                .iter()
                .filter(|case| fixtures.iter().any(|own| own.title == case.title))
                .map(|case| case.title.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            selected(&cases),
            fixtures
                .iter()
                .map(|own| own.title.to_string())
                .collect::<Vec<_>>(),
            "{document}: missing, duplicate or reordered own identity"
        );
        for own in &fixtures {
            let case = cases.iter().find(|case| case.title == own.title).unwrap();
            assert_eq!(case.expected.as_deref(), own.expected);
            assert_eq!(case.source.trim(), comma_free_headers(own.accepted).trim());
            check_pair(&case.source, own.accepted, own.assertions, own.expected);
            checked += 1;
            assertions += own.assertions;
            if let Some(expected) = own.expected {
                returns.push(expected);
            }
        }
        cases.push(common::Case {
            title: "future unrelated tail".to_string(),
            source: "assert(1, 1)".to_string(),
            expected: None,
        });
        assert_eq!(selected(&cases).len(), fixtures.len());
    }
    assert_eq!(checked, 8);
    assert_eq!(assertions, 30);
    assert_eq!(returns, ["21"]);
}

#[test]
fn all_rewritten_normative_forms_keep_complete_asts_canonical_source_graph_and_runtime() {
    let mut checked = 0;
    for (_, cases) in documents() {
        for case in cases {
            let accepted = legacy_headers(&case.source);
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
    assert!(checked >= OWN_CASES.len());
}

// Unchanged accepted-syntax programs from the eight pre-executed NUC37 inputs.
const OWN_CASES: &[OwnCase] = &[
    OwnCase {
        document: "spec",
        title: "Nucleoid defines comma-free bare named arrows",
        assertions: 6,
        expected: None,
        accepted: r###"# Nucleoid defines comma-free bare named arrows

def add left, right => left + right
def scaled left, right, factor => (left + right) * factor
def pair left, right => [left, right]
def coordinates x, y => {x, y}
def indented left, right =>
    left + right
assert(add(2, 3), 5)
assert(scaled(2, 3, 4), 20)
assert(pair("left", "right"), ["left", "right"])
point = coordinates(2, 3)
assert(point.x, 2)
assert(point.y, 3)
assert(indented(2, 3), 5)"###,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid keeps comma-free bare named arrows reactive",
        assertions: 5,
        expected: None,
        accepted: r###"# Nucleoid keeps comma-free bare named arrows reactive

bias = 1
def total left, right => left + right + bias
tracked = total(2, 3)
assert(tracked, 6)
bias = 4
assert(tracked, 9)
def projected first, second, third => first * second + third
left = 2
right = 3
offset = 4
result = projected(left, right, offset)
assert(result, 10)
left = 5
assert(result, 19)
right = 2
assert(result, 14)"###,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid preserves comma-free methods and initializers",
        assertions: 3,
        expected: None,
        accepted: r###"# Nucleoid preserves comma-free methods and initializers

class Calculator:
    def init base, scale => this.base = base * scale
    def combine left, right => this.base + left + right
    def product left, right, scale => self.base + left * right * scale
calculator = Calculator(1, 2)
result = calculator.combine(3, 4)
assert(result, 9)
calculator.base = 5
assert(result, 12)
assert(calculator.product(2, 3, 4), 29)"###,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid preserves nested comma-free bare named arrows",
        assertions: 2,
        expected: None,
        accepted: r###"# Nucleoid preserves nested comma-free bare named arrows

def outer(initial):
    def combine left, right, scale => (left + right) * scale
    return combine(initial, 2, 3)
assert(outer(4), 18)
def factory(ignored) => (left, right) => left + right
custom = factory(10)
assert(custom(2, 3), 5)"###,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid rolls back comma-free named arrow redefinitions",
        assertions: 7,
        expected: Some("21"),
        accepted: r###"# Nucleoid rolls back comma-free named arrow redefinitions

source = 3
def reading left, right => source + left + right
result = reading(2, 4)
assert(result, 9)
try:
    def reading left, right => missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(result, 9)
source = 5
assert(result, 11)
def prefer current, fallback => current ||= fallback
def combined first, second, scale => (first + second) * scale
assert(prefer(0, 9), 9)
assert(prefer(3, 9), 3)
assert(combined(2, 5, 3), 21)
combined(2, 5, 3)"###,
    },
    OwnCase {
        document: "reference",
        title: "Nucleoid preserves comma-free named arrow relationships",
        assertions: 3,
        expected: None,
        accepted: r###"# Nucleoid preserves comma-free named arrow relationships

scale = 1
def area width, height => width * height * scale
width = 3
height = 4
result = area(width, height)
assert(result, 12)
scale = 2
assert(result, 24)
width = 5
assert(result, 40)"###,
    },
    OwnCase {
        document: "readme",
        title: "Named arrow parameters can omit every comma",
        assertions: 2,
        expected: None,
        accepted: r###"def add left, right => left + right
assert(add(2, 3), 5)
def scaled left, right, factor => (left + right) * factor
assert(scaled(2, 3, 4), 20)"###,
    },
    OwnCase {
        document: "examples",
        title: "Comma-free named arrows keep live inputs",
        assertions: 2,
        expected: None,
        accepted: r###"bias = 1
def total left, right => left + right + bias
result = total(2, 3)
assert(result, 6)
bias = 4
assert(result, 9)"###,
    },
];
