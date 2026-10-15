//! NUC32 pairs genuine indented expression bodies with unchanged return suites.
//! Every ordinary test is self-contained: no Git history or session artifacts.

mod common;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{Expr, FunctionBody, Stmt, parse, parse_expression, parse_program};
use nucleoid::lang::evaluation::Flow;
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::statement;
use nucleoid::value::ObjectId;
use nucleoid::{ErrorKind, Position, Runtime, Value};
use regex::Regex;

fn equivalent(indented: &str, explicit: &str) -> Vec<Stmt> {
    assert_ne!(indented, explicit, "a pair must exercise the new layout");
    let actual = parse(indented).unwrap_or_else(|error| panic!("{indented}\n{error}"));
    let expected = parse(explicit).unwrap_or_else(|error| panic!("{explicit}\n{error}"));
    assert_eq!(actual, expected, "{indented}");
    assert_eq!(generate_all(&actual), generate_all(&expected), "{indented}");
    actual
}

/// These fixtures have an immediately following, sole return expression.
/// Leave bare returns, absent returns and multi-statement outer bodies alone.
fn indent_returns(explicit: &str) -> String {
    Regex::new(r"(?m)^([ \t]*def [^\n]+):([ \t]*(?:#[^\n]*)?)\n([ \t]+)return[ \t]+([^\n]+)")
        .unwrap()
        .replace_all(explicit, "$1 =>$2\n$3$4")
        .into_owned()
}

fn explicit_returns(indented: &str) -> String {
    Regex::new(r"(?m)^([ \t]*def [^\n]+) =>([ \t]*(?:#[^\n]*)?)\n([ \t]+)([^\n]+)")
        .unwrap()
        .replace_all(indented, "$1:$2\n${3}return $4")
        .into_owned()
}

fn check_pair(
    indented: &str,
    explicit: &str,
    assertions: usize,
    expected: Option<&str>,
) -> [Runtime; 2] {
    let statements = equivalent(indented, explicit);
    assert_eq!(
        common::assertions::protected_assertions(&statements).len(),
        assertions
    );
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let mut values = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([indented, explicit]) {
        let value = common::check_with_count(
            runtime,
            &common::Case {
                title: "indented named-arrow pair".to_string(),
                source: source.to_string(),
                expected: expected.map(str::to_string),
            },
            assertions,
        )
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
        assert_eq!(runtime.assertions_run(), assertions, "{source}");
        values.push(runtime.serialize_json(&value).unwrap());
    }
    assert_eq!(values[0], values[1]);
    runtimes
}

fn graphs_match(actual: &Runtime, expected: &Runtime) {
    assert!(actual.graph.keys().eq(expected.graph.keys()));
    for node in actual.graph.nodes() {
        let other = expected.graph.retrieve(&node.key).unwrap();
        assert_eq!(node.kind, other.kind, "{}", node.key);
        assert_eq!(node.dependencies, other.dependencies, "{}", node.key);
        assert_eq!(node.dependents, other.dependents, "{}", node.key);
        assert!(
            actual
                .graph
                .shape_dependencies(&node.key)
                .eq(expected.graph.shape_dependencies(&node.key))
        );
    }
}

const SIGNATURES: &[&str] = &[
    "()",
    "(value)",
    "(value: ArbitraryType,)",
    "(left, right)",
    "(left: int, right)",
    "(left, right: number,)",
    "(\n    left: int,\n    right: number,\n)",
    " value",
    " value: ArbitraryType",
    " left, right",
    " left: int, right",
    " left, right: number",
    " left: int, right: number",
    " left: int, right, factor: number",
];

#[test]
fn all_existing_parameter_spellings_reuse_exact_metadata_and_named_return_blocks() {
    for signature in SIGNATURES {
        let indented = format!("def f{signature} =>\n    7\nassert(f(), 7)\nf()");
        let explicit = format!("def f{signature}:\n    return 7\nassert(f(), 7)\nf()");
        let statements = equivalent(&indented, &explicit);
        let [Stmt::Function(function), ..] = statements.as_slice() else {
            panic!("expected a named function");
        };
        assert_eq!(function.name.as_deref(), Some("f"));
        assert_eq!(
            function.body,
            FunctionBody::Block(vec![Stmt::Return(Some(Expr::Number(7.0)))])
        );
        check_pair(&indented, &explicit, 1, Some("7"));

        let indented = format!(
            "class Item:\n    def f{} =>\n        7",
            signature.replace('\n', "\n    ")
        );
        let explicit = format!(
            "class Item:\n    def f{}:\n        return 7",
            signature.replace('\n', "\n    ")
        );
        equivalent(&indented, &explicit);
    }
}

#[test]
fn complete_expression_ast_and_canonical_generation_include_assignments_and_function_values() {
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
        "super",
        "super(left)",
    ] {
        let statements = equivalent(
            &format!("def f left: int, right, factor: number =>\n    {expression}"),
            &format!("def f left: int, right, factor: number:\n    return {expression}"),
        );
        let [Stmt::Function(function)] = statements.as_slice() else {
            panic!("expected a named function");
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
fn real_indentation_accepts_existing_comments_crlf_tabs_and_parenthesized_layout() {
    for layout in [
        "\n    ",
        "\r\n    ",
        "\n\t",
        " # header\n\n# blank\n    # body\n    ",
        "\n;\n    ",
        "\n    ;\n    ",
    ] {
        for ending in ["", "\n", ";\n", "\n    ;\n", " # end\n"] {
            equivalent(
                &format!("def f value =>{layout}value + 1{ending}\nnext = 3"),
                "def f value:\n    return value + 1\nnext = 3",
            );
        }
    }
    for expression in [
        "(\n        value +\n        1\n    )",
        "(\r\n\t\tvalue + 1\r\n\t)",
        "(\n    # operand\n    value + 1\n)",
    ] {
        equivalent(
            &format!("def f value =>\n    {expression}\nnext = 3"),
            "def f value:\n    return value + 1\nnext = 3",
        );
    }
    let source = "def f value =>\n    value + 1\nnext = 3";
    assert_eq!(
        parse_program(source).unwrap().positions,
        [Position::new(1, 1), Position::new(3, 1)]
    );
    assert_eq!(
        parse_program(source).unwrap().positions,
        parse_program(&explicit_returns(source)).unwrap().positions
    );
}

#[test]
fn exactly_one_expression_requires_the_closing_body_dedent() {
    for (source, message, line, column) in [
        (
            "def f x =>\n    x\n    x + 1",
            "Expected the end of a block but found identifier 'x'",
            3,
            5,
        ),
        (
            "def f x =>\n    x; x + 1",
            "Expected the end of a block but found identifier 'x'",
            2,
            8,
        ),
        (
            "def f x =>\n    x 1",
            "Expected the end of a block but found number 1",
            2,
            7,
        ),
        (
            "def f x =>\n    x\n        1",
            "Expected the end of a block but found an indented block",
            3,
            9,
        ),
        (
            "def f x =>\n    return x",
            "Unexpected keyword 'return'",
            2,
            12,
        ),
        ("def f x =>\n    (x +)", "Unexpected ')'", 2, 10),
        (
            "def f x =>\n    { return x }",
            "Expected a key but found keyword 'return'",
            2,
            14,
        ),
        ("def f x =>\n    @", "Unexpected character '@'", 2, 5),
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
    // A named object expression is not the anonymous arrow's statement block.
    let named = parse("def f x =>\n    { x }").unwrap();
    let anonymous = parse("x => { x }").unwrap();
    let [Stmt::Function(named)] = named.as_slice() else {
        panic!("expected a named function");
    };
    let [Stmt::Expression(Expr::Function(anonymous))] = anonymous.as_slice() else {
        panic!("expected an anonymous function");
    };
    assert!(
        matches!(&named.body, FunctionBody::Block(body) if matches!(body.as_slice(), [Stmt::Return(Some(Expr::ObjectLiteral(_)))]))
    );
    assert!(
        matches!(&anonymous.body, FunctionBody::Block(body) if matches!(body.as_slice(), [Stmt::Expression(Expr::Identifier(_))]))
    );
    for (source, message, line, column) in [
        ("x =>\n    x", "Unexpected end of line", 2, 5),
        ("(x: int) =>\n    x", "Unexpected end of line", 2, 5),
        ("callback = x =>\n    x", "Unexpected end of line", 2, 5),
        (
            "def f x =>\n    if x: return x",
            "Unexpected keyword 'if'",
            2,
            8,
        ),
        ("def f x =>\n    pass", "Unexpected keyword 'pass'", 2, 9),
        ("def f x =>\n    ;", "Unexpected the end of a block", 2, 6),
        ("def f x =>\n    x\n)", "Unexpected ')'", 3, 2),
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
fn nonqualifying_layout_and_unsupported_headers_keep_legacy_errors_and_positions() {
    for (source, message, line, column) in [
        ("def f x =>", "Unexpected end of input", 1, 11),
        ("def f x =>\n", "Unexpected end of line", 2, 1),
        ("def f x =>\nx", "Unexpected end of line", 2, 1),
        ("def f x =>\n;\nx", "Unexpected end of line", 2, 1),
        (
            "def f left: int, right =>\nleft",
            "Unexpected end of line",
            2,
            1,
        ),
        ("def f x =>;\n    x", "Unexpected ';'", 1, 11),
        ("def f =>\n    1", "Expected '(' but found '=>'", 1, 7),
        ("def f:\n    1", "Expected '(' but found ':'", 1, 6),
        (
            "def f left: int, right, =>\n    left",
            "Expected '(' but found identifier 'left'",
            1,
            7,
        ),
        (
            "def f x\n=>\n    x",
            "Expected '(' but found identifier 'x'",
            1,
            7,
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
fn valid_inline_arrows_following_statements_colon_suites_and_anonymous_forms_are_not_reinterpreted()
{
    let inline = parse("def f x => x\n    7\nnext = 3").unwrap();
    assert_eq!(inline, parse("def f x => x\n7\nnext = 3").unwrap());
    assert_eq!(inline.len(), 3);
    for source in [
        "def f x => x; next = 3",
        "def f(x) => (\n    x + 1\n)\nnext = 3",
        "def f(x): return x; next = 3",
        "def f(x): { return x }",
        "def f x:\n    x\n    return",
        "def f x:\n    int => x",
        "(x: int) => x",
        "(x) => { return x }",
        "function(x) { return x }",
        "add left right",
    ] {
        let statements = parse(source).unwrap();
        assert!(!statements.is_empty());
    }
    let statements = parse("x => x").unwrap();
    let [Stmt::Expression(Expr::Function(function))] = statements.as_slice() else {
        panic!("expected an anonymous expression function");
    };
    assert!(matches!(&function.body, FunctionBody::Expression(_)));
}

#[test]
fn null_callable_binding_noncallable_errors_and_noncapture_stay_paired() {
    let explicit = r#"
def operation value:
    return value + 100
def increment value:
    return value + 1
def invoke operation: Callback, value:
    return operation(value)
assert(invoke(increment, 3), 4)
assert(invoke(null, 3), null)
try:
    invoke(7, 3)
catch error:
    assert(error, TypeError("operation is not a function"))
"#;
    check_pair(&indent_returns(explicit), explicit, 3, None);
    for (explicit, message) in [
        (
            "def wrap value:\n    return { missing }\nwrap(3)",
            "missing is not defined",
        ),
        (
            "def factory value:\n    return x => ({ value })\ncallback = factory(3)\ncallback(4)",
            "value is not defined",
        ),
        (
            "def receiver value:\n    return this.amount\nreceiver(3)",
            "this is not defined",
        ),
        (
            "def receiver value:\n    return self.amount\nreceiver(3)",
            "this is not defined",
        ),
    ] {
        let indented = indent_returns(explicit);
        equivalent(&indented, explicit);
        let error = Runtime::new().run(&indented).unwrap_err();
        assert_eq!(error, Runtime::new().run(explicit).unwrap_err());
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), message);
        assert!(error.position().is_some());
    }
}

#[test]
fn graph_reasoning_source_and_frame_apis_keep_named_block_execution() {
    let explicit = "bias = 2\ndef total left: int, right:\n    return left + right + bias\nresult = total(3, 4)\nresult";
    let [mut actual, mut expected] = check_pair(&indent_returns(explicit), explicit, 0, Some("9"));
    graphs_match(&actual, &expected);
    for source in ["why result", "model |> why"] {
        let a = actual.run(source).unwrap();
        let b = expected.run(source).unwrap();
        assert_eq!(
            actual.serialize_json(&a).unwrap(),
            expected.serialize_json(&b).unwrap()
        );
    }
    let statements = parse("def total left, right =>\n    left + right").unwrap();
    assert_eq!(
        NodeKey::block(&statements, None).as_str(),
        "block(def total)"
    );
    assert_eq!(generate_all(&statements), "def total");
}

#[test]
fn direct_compile_and_node_apis_preserve_caller_frames_on_return_and_error() {
    for expression in ["value + 1", "value += 1", "missing"] {
        let indented = format!("def f value =>\n    {expression}");
        let explicit = format!("def f value:\n    return {expression}");
        Runtime::check(&indented).unwrap();
        let program = statement::compile(&indented).unwrap();
        assert_eq!(program.statements, parse(&explicit).unwrap());
        assert_eq!(program.positions, [Position::new(1, 1)]);
        for source in [&indented, &explicit] {
            let mut runtime = Runtime::new();
            let mut scope = Scope::new();
            scope.push();
            scope.declare("value", Value::Number(99.0));
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
            let call = parse("f(3)").unwrap();
            let mut node = Nuc::convert(&mut runtime, &mut scope, &call[0]).unwrap();
            let result = node.run(&mut runtime, &mut scope);
            if expression == "missing" {
                let error = result.err().expect("the missing read must fail");
                assert_eq!(error.kind(), ErrorKind::Reference);
                assert_eq!(error.message(), "missing is not defined");
            } else {
                assert!(matches!(
                    result.unwrap().flow,
                    Flow::Normal(Value::Number(4.0))
                ));
            }
            assert_eq!(scope.depth(), 2);
            assert_eq!(scope.retrieve("value"), Some(&Value::Number(99.0)));
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
fn nullary_initializer_bare_super_adoption_and_uncaught_rollback_keep_existing_semantics() {
    let explicit = "class Parent:\n    def init():\n        return this.amount = 3\nclass Child: Parent\n    def init():\n        return super\nchild = Child()\nassert(child.id, \"child\")\nassert(child.amount, 3)\nassert(Parent.length, 0)\nassert(Child.length, 1)";
    check_pair(&indent_returns(explicit), explicit, 4, None);
    let explicit = "source = 3\ndef reading left, right:\n    return source + left + right\ntotal = reading(2, 4)";
    let [mut actual, mut expected] = check_pair(&indent_returns(explicit), explicit, 0, Some("9"));
    for runtime in [&mut actual, &mut expected] {
        let error = runtime
            .run("def reading left, right =>\n    missing + left + right")
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.run("total").unwrap(), Value::Number(9.0));
        let error = runtime
            .run("source = 99\ndef reading left, right =>\n    left\n    right")
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.position(), Some(Position::new(4, 5)));
        assert_eq!(runtime.run("source").unwrap(), Value::Number(3.0));
        runtime.run("source = 4").unwrap();
        assert_eq!(runtime.run("total").unwrap(), Value::Number(10.0));
    }
    graphs_match(&actual, &expected);
}

#[test]
fn flat_lists_and_inherited_expression_statement_chain_and_postfix_bounds_remain() {
    for count in [2, 129, 1024] {
        for parenthesized in [false, true] {
            for all_typed in [false, true] {
                let parameters = (0..count)
                    .map(|i| {
                        if all_typed || i % 3 == 0 {
                            format!("p{i}: Unknown{i}")
                        } else {
                            format!("p{i}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let signature = if parenthesized {
                    format!("({parameters})")
                } else {
                    format!(" {parameters}")
                };
                let indented = format!("def many{signature} =>\n    p{}", count - 1);
                let explicit = format!("def many{signature}:\n    return p{}", count - 1);
                let statements = equivalent(&indented, &explicit);
                let [Stmt::Function(function)] = statements.as_slice() else {
                    panic!("expected named function")
                };
                assert_eq!(function.parameters.len(), count);
                let arguments = vec!["7"; count].join(", ");
                for source in [&indented, &explicit] {
                    let mut runtime = Runtime::new();
                    runtime.run(source).unwrap();
                    assert_eq!(
                        runtime.run(&format!("many({arguments})")).unwrap(),
                        Value::Number(7.0)
                    );
                }
            }
        }
    }
    let mut runtime = Runtime::new();
    runtime.run("saved = 7").unwrap();
    for (expression, message) in [
        (
            format!("{}value", "-".repeat(200)),
            "Expressions are nested too deeply",
        ),
        (
            format!("value{}", " + 1".repeat(200)),
            "Expression has too many operands",
        ),
        (
            format!("root{}", ".next".repeat(200)),
            "Expression has too many parts",
        ),
        (
            format!("{}value", "saved = ".repeat(200)),
            "Expressions are nested too deeply",
        ),
    ] {
        let error = runtime
            .run(&format!(
                "saved = 3\ndef limited value =>\n    {expression}"
            ))
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), message);
        assert_eq!(error.position().unwrap().line, 3);
        assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
    }
    let mut nested = String::new();
    for depth in 0..100 {
        nested.push_str(&format!("{}def f{depth}():\n", "    ".repeat(depth)));
    }
    nested.push_str(&format!(
        "{}def last value =>\n{}value",
        "    ".repeat(100),
        "    ".repeat(101)
    ));
    let error = Runtime::check(&nested).unwrap_err();
    assert_eq!(error.message(), "Statements are nested too deeply");
    runtime.run("def limited value =>\n    value + 1").unwrap();
    assert_eq!(runtime.run("limited(3)").unwrap(), Value::Number(4.0));
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
fn native_stdin_supports_all_parameter_forms_methods_blank_commit_and_eof() {
    for signature in SIGNATURES {
        let explicit = format!(
            ".json on\ndef f{signature}:\n    return 7\n\nf()\nclass Item:\n    def init value:\n        return this.amount = value\n    def reading{}:\n        return self.amount\n\nitem = Item(9)\nitem.reading()",
            signature.replace('\n', "\n    ")
        );
        // Multiline parenthesized headers were not interactive continuations;
        // cover those through the native file path instead of changing entry.
        if signature.contains('\n') {
            continue;
        }
        let indented = indent_returns(&explicit);
        let outputs = [repl(&indented), repl(&explicit)];
        for output in &outputs {
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.lines().any(|line| line == "7.0"), "{stdout}");
            assert!(stdout.lines().any(|line| line == "9.0"), "{stdout}");
        }
        assert_eq!(outputs[0].stdout, outputs[1].stdout);
    }
    for ending in ["", "\n"] {
        let output = repl(&format!(
            ".json on\n.graph on\ndef identity value => # => header\n    value{ending}"
        ));
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("identity [function]"));
    }
    let output = repl(".json on\r\ndef f value => # header\r\n\t(value + 1)\r\n\r\nf(2)");
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "3.0")
    );
}

#[test]
fn native_stdin_keeps_inline_anonymous_literal_and_comment_interaction() {
    let source = ".json on\ndef identity value => value\nidentity(4)\n4 # =>\n\"=>\"\n/=>/\n`=>`\ncallback = value => value + 1\ncallback(4)\n# def false value =>\n6";
    let output = repl(source);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.lines().filter(|line| *line == "4.0").count() >= 2,
        "{stdout}"
    );
    assert!(stdout.lines().any(|line| line == "5.0"), "{stdout}");
    assert!(stdout.lines().any(|line| line == "6.0"), "{stdout}");
    assert!(stdout.contains("\"=>\""), "{stdout}");
}

#[test]
fn native_stdin_reports_structured_body_errors_then_recovers_existing_state() {
    let source = ".json on\ndef identity value =>\n    value\n\nidentity(4)\ndef identity value =>\n    value\n    value + 1\n\nidentity(5)\ndef identity value =>\nvalue\n\nidentity(6)";
    let output = repl(source);
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 2, "{stderr}");
    assert!(stderr.contains("SyntaxError: Expected the end of a block but found identifier 'value' (line 3, column 5)"), "{stderr}");
    assert!(
        stderr.contains("SyntaxError: Unexpected end of line (line 2, column 1)"),
        "{stderr}"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    for value in ["4.0", "5.0", "6.0"] {
        assert!(stdout.lines().any(|line| line == value), "{stdout}");
    }
    let output = repl("def f value =>\n    value\n    value + 1");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        "SyntaxError: Expected the end of a block but found identifier 'value' (line 3, column 5)"
    );
}

#[test]
fn native_file_path_executes_complete_programs_and_keeps_physical_diagnostics() {
    let directory = std::env::temp_dir().join(format!("nucleoid-nuc32-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("program.nuc");
    for signature in SIGNATURES {
        let explicit = format!("def f{signature}:\n    return 7\nassert(f(), 7)\nf()");
        let indented = indent_returns(&explicit);
        // A multiline signature's following return still belongs to the same header.
        let indented = if signature.contains('\n') {
            format!("def f{signature} =>\n    7\nassert(f(), 7)\nf()")
        } else {
            indented
        };
        for source in [&indented, &explicit] {
            fs::write(&path, source.as_bytes()).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
                .args(["--json", "--graph"])
                .arg(&path)
                .output()
                .unwrap();
            assert!(output.status.success(), "{source}\n{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(
                stdout.starts_with("7.0\n") || stdout.starts_with("7.0\r\n"),
                "{stdout}"
            );
            assert!(stdout.contains("f [function]"), "{stdout}");
        }
    }
    // Reuse the sealed pairs for real file execution of methods/init/super,
    // nested definitions, aliases/snapshots and caught rollback as well.
    for own in OWN_CASES {
        let indented = indent_returns(own.explicit);
        let mut previous = None;
        for source in [&indented, own.explicit] {
            fs::write(&path, source.as_bytes()).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
                .args(["--json", "--graph"])
                .arg(&path)
                .output()
                .unwrap();
            assert!(output.status.success(), "{}\n{output:?}", own.title);
            assert!(output.stderr.is_empty(), "{output:?}");
            if let Some(expected) = own.expected {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert_eq!(
                    serde_json::from_str::<f64>(stdout.lines().next().unwrap()).unwrap(),
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
            "def f x =>\n    x\n    1",
            "SyntaxError: Expected the end of a block but found number 1 (line 3, column 5)",
        ),
        (
            "def f x =>\nx",
            "SyntaxError: Unexpected end of line (line 2, column 1)",
        ),
        (
            "def f x =>\n    missing\nf(3)",
            "ReferenceError: missing is not defined (line 3, column 1)",
        ),
    ] {
        fs::write(&path, source.as_bytes()).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg(&path)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{output:?}");
        assert_eq!(String::from_utf8(output.stderr).unwrap().trim(), message);
    }
    fs::remove_dir_all(directory).unwrap();
}

struct OwnCase {
    document: &'static str,
    title: &'static str,
    assertions: usize,
    expected: Option<&'static str>,
    explicit: &'static str,
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

fn own_cases<'a>(document: &str, cases: &'a [common::Case]) -> Vec<&'a common::Case> {
    cases
        .iter()
        .filter(|case| {
            OWN_CASES
                .iter()
                .any(|own| own.document == document && own.title == case.title)
        })
        .collect()
}

#[test]
fn eight_paired_document_identities_order_assertions_and_returns_are_protected() {
    let mut checked = 0;
    let mut assertions = 0;
    let mut returns = Vec::new();
    for (document, mut cases) in documents() {
        let fixtures = OWN_CASES
            .iter()
            .filter(|own| own.document == document)
            .collect::<Vec<_>>();
        let selected = own_cases(document, &cases);
        assert_eq!(
            selected.len(),
            fixtures.len(),
            "{document}: missing or duplicate identity"
        );
        for (case, own) in selected.into_iter().zip(&fixtures) {
            assert_eq!(case.title, own.title, "{document}: relative order");
            assert_eq!(case.expected.as_deref(), own.expected);
            assert_eq!(case.source.trim(), indent_returns(own.explicit).trim());
            let [actual, expected] =
                check_pair(&case.source, own.explicit, own.assertions, own.expected);
            graphs_match(&actual, &expected);
            checked += 1;
            assertions += own.assertions;
            if let Some(expected) = own.expected {
                returns.push(expected);
            }
        }
        // Future cases are allowed after these identities; never assume a tail
        // length or the entire current corpus size in an ordinary regression.
        cases.push(common::Case {
            title: "future unrelated case".to_string(),
            source: "assert(1, 1)".to_string(),
            expected: None,
        });
        assert_eq!(own_cases(document, &cases).len(), fixtures.len());
    }
    assert_eq!(checked, 8);
    assert_eq!(assertions, 41);
    assert_eq!(returns, ["20", "21"]);
}

#[test]
fn rewritten_normative_documents_keep_full_asts_canonical_assertions_and_execution() {
    let mut document_programs = 0;
    for (_, cases) in documents() {
        for case in cases {
            let explicit = explicit_returns(&case.source);
            if explicit == case.source {
                continue;
            }
            let count = common::assertions::protected_assertions(&parse(&explicit).unwrap()).len();
            check_pair(&case.source, &explicit, count, case.expected.as_deref());
            document_programs += 1;
        }
    }
    assert!(document_programs >= OWN_CASES.len());
    println!("AST/canonical/runtime pairs: {document_programs} normative document programs");
}

// Unchanged accepted-syntax counterparts from the eight sealed NUC32 inputs.
const OWN_CASES: &[OwnCase] = &[
    OwnCase {
        document: "spec",
        title: "Nucleoid defines indented named arrow expressions",
        assertions: 5,
        expected: Some("20"),
        explicit: r###"# Nucleoid defines indented named arrow expressions

def literal():
    return 4
def twice amount:
    return amount * 2
def typed amount: int:
    return amount + 1
def add left, right:
    return left + right
def scaled left: int, right, factor: number:
    return (left + right) * factor
assert(literal(), 4)
assert(twice(3), 6)
assert(typed(4), 5)
assert(add(2, 3), 5)
assert(scaled(2, 3, 4), 20)
scaled(2, 3, 4)"###,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid keeps indented named arrows reactive",
        assertions: 6,
        expected: None,
        explicit: r###"# Nucleoid keeps indented named arrows reactive

offset = 2
def adjusted number:
    return number + offset
live = adjusted
snapshot = adjusted.value
result = live(3)
frozen = snapshot(3)
assert(result, 5)
assert(frozen, 5)
def adjusted number:
    return number * offset
assert(result, 6)
assert(frozen, 5)
offset = 4
assert(result, 12)
assert(frozen, 7)"###,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid preserves indented named arrow methods and initializers",
        assertions: 8,
        expected: None,
        explicit: r###"# Nucleoid preserves indented named arrow methods and initializers

class Parent:
    def init value:
        return this.amount = value
    def reading offset: int:
        return self.amount + offset
class Child: Parent
    def init value:
        return super(value)
    def reading offset: int:
        return this.amount * 2 + offset
    def change delta, extra:
        return this.amount += delta + extra
child = Child(3)
observed = child.reading(1)
assert(child.id, "child")
assert(child.amount, 3)
assert(observed, 7)
assert(child.change(2, 1), 6)
assert(child.amount, 6)
assert(observed, 13)
assert(Parent.length, 0)
assert(Child.length, 1)"###,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid preserves indented named arrow bindings and returned values",
        assertions: 12,
        expected: None,
        explicit: r###"# Nucleoid preserves indented named arrow bindings and returned values

left = 10
right = 20
def pair left: int, right:
    return [left, right]
def point x, y:
    return {x, y}
def text left, right:
    return `${left},${right}`
def factory ignored:
    return value => value + 1
def assign left, right:
    return left += right
def outer initial:
    def local amount:
        return amount * 2
    return local(initial)
assert(pair(3, 4), [3, 4])
assert(pair(3), [3, null])
assert(pair(), [null, null])
assert(pair("three", true), ["three", true])
p = point(2, 3)
assert(p.x, 2)
assert(p.y, 3)
assert(text(2, 3), "2,3")
callback = factory(0)
assert(callback(5), 6)
assert(assign(3, 4), 7)
assert(left, 10)
assert(right, 20)
assert(outer(4), 8)"###,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid rolls back failed indented named arrow redefinitions",
        assertions: 5,
        expected: Some("21"),
        explicit: r###"# Nucleoid rolls back failed indented named arrow redefinitions

source = 3
def reading left: int, right: int:
    return source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading left: int, right:
        return missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)"###,
    },
    OwnCase {
        document: "reference",
        title: "Nucleoid preserves indented named arrow relationships",
        assertions: 2,
        expected: None,
        explicit: r###"# Nucleoid preserves indented named arrow relationships

scale = 2
def area width: number, height: number:
    return width * height * scale
result = area(3, 4)
assert(result, 24)
scale = 3
assert(result, 36)"###,
    },
    OwnCase {
        document: "readme",
        title: "Named arrows can indent a single expression",
        assertions: 1,
        expected: None,
        explicit: r###"def square value:
    return value * value
assert(square(4), 16)"###,
    },
    OwnCase {
        document: "examples",
        title: "Indented named arrows keep live inputs",
        assertions: 2,
        expected: None,
        explicit: r###"bias = 1
def total left: int, right:
    return left + right + bias
result = total(2, 3)
assert(result, 6)
bias = 4
assert(result, 9)"###,
    },
];
