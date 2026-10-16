//! Optional final-else colons preserve the complete accepted suite.
//! Ordinary regressions require neither Git history nor session artifacts.

mod common;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Stmt, parse, parse_program};
use nucleoid::lang::evaluation::Flow;
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::statement;
use nucleoid::value::ObjectId;
use nucleoid::{ErrorKind, Position, Runtime, Value};

/// Edit only lexical Else/Colon pairs with genuine suite layout, never literal
/// or comment contents. The inverse restores only qualifying bare Else tokens.
fn else_colons(source: &str, omit: bool) -> String {
    let tokens = tokenize(source).unwrap();
    let mut edits = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.token != Token::Keyword(Keyword::Else) {
            continue;
        }
        let next = index + 1;
        let has_colon = tokens[next].token == Token::Colon;
        if has_colon != omit {
            continue;
        }
        let body = &tokens[next + usize::from(has_colon)..];
        if !matches!(body[0].token, Token::Newline | Token::Indent)
            || !body
                .iter()
                .find(|token| !matches!(token.token, Token::Newline | Token::Semicolon))
                .is_some_and(|token| token.token == Token::Indent)
        {
            continue;
        }
        let position = if omit {
            tokens[next].position
        } else {
            Position::new(token.position.line, token.position.column + 4)
        };
        let mut offset = 0;
        for (line, text) in source.split_inclusive('\n').enumerate() {
            if line + 1 == position.line {
                offset += text.char_indices().nth(position.column - 1).unwrap().0;
                break;
            }
            offset += text.len();
        }
        edits.push(offset);
    }
    let mut result = source.to_string();
    for offset in edits.into_iter().rev() {
        if omit {
            assert_eq!(result.remove(offset), ':');
        } else {
            result.insert(offset, ':');
        }
    }
    result
}

fn equivalent(actual: &str, explicit: &str) -> Vec<Stmt> {
    let actual_ast = parse(actual).unwrap_or_else(|error| panic!("{actual}\n{error}"));
    let explicit_ast = parse(explicit).unwrap_or_else(|error| panic!("{explicit}\n{error}"));
    assert_eq!(actual_ast, explicit_ast, "{actual}");
    assert_eq!(generate_all(&actual_ast), generate_all(&explicit_ast));
    actual_ast
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

fn check_pair(actual: &str, explicit: &str, count: usize, expected: Option<&str>) {
    let statements = equivalent(actual, explicit);
    assert_eq!(
        common::assertions::protected_assertions(&statements),
        common::assertions::protected_assertions(&parse(explicit).unwrap())
    );
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let mut values = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([actual, explicit]) {
        let value = common::check_with_count(
            runtime,
            &common::Case {
                title: "indented else pair".to_string(),
                source: source.to_string(),
                expected: expected.map(str::to_string),
            },
            count,
        )
        .unwrap_or_else(|error| panic!("{source}\n{error}"));
        assert_eq!(runtime.assertions_run(), count, "{source}");
        values.push(runtime.serialize_json(&value).unwrap());
    }
    assert_eq!(values[0], values[1]);
    graphs_match(&runtimes[0], &runtimes[1]);
}

#[test]
fn genuine_layout_reuses_the_complete_alternate_block_and_top_level_positions() {
    for suffix in ["", "\n", "\n# EOF", "\nnext = 3", "\n; next = 3"] {
        for body in [
            "\n    result = 2",
            "\r\n\tresult = 2",
            " # else: is only a comment\n\n# blank\n    # body\n    result = 2",
            "\n;\n    result = 2",
            "\n    ;\n    result = 2;",
        ] {
            let actual = format!("if false: pass\nelse{body}{suffix}");
            let explicit = else_colons(&actual, false);
            assert_ne!(actual, explicit);
            let statements = equivalent(&actual, &explicit);
            let Stmt::If { alternate, .. } = &statements[0] else {
                panic!("expected conditional");
            };
            assert!(matches!(alternate.as_deref(), Some(Stmt::Block(body)) if !body.is_empty()));
            assert_eq!(
                parse_program(&actual).unwrap().positions,
                parse_program(&explicit).unwrap().positions
            );
        }
    }
    let source = "if false:\n    pass\nelse\n    pass\nnext = 3";
    assert_eq!(
        statement::compile(source).unwrap().positions,
        [Position::new(1, 1), Position::new(5, 1)]
    );
}

#[test]
fn lexer_and_rewriter_leave_literals_comments_and_every_other_colon_alone() {
    let explicit = "text = \"first\nelse:\n    last\"\npattern = /else:/\ntemplate = `else:`\nif false: pass\nelse: # else:\n    pass";
    let actual = else_colons(explicit, true);
    assert_eq!(
        actual,
        "text = \"first\nelse:\n    last\"\npattern = /else:/\ntemplate = `else:`\nif false: pass\nelse # else:\n    pass"
    );
    equivalent(&actual, explicit);
    let mut old_tokens = tokenize(explicit).unwrap();
    let colon = old_tokens
        .windows(2)
        .position(|pair| {
            pair[0].token == Token::Keyword(Keyword::Else) && pair[1].token == Token::Colon
        })
        .unwrap()
        + 1;
    old_tokens.remove(colon);
    assert_eq!(
        old_tokens
            .into_iter()
            .map(|token| token.token)
            .collect::<Vec<_>>(),
        tokenize(&actual)
            .unwrap()
            .into_iter()
            .map(|token| token.token)
            .collect::<Vec<_>>()
    );
}

#[test]
fn legacy_inline_semicolon_braced_and_mixed_nested_association_stay_exact() {
    for explicit in [
        "result = 0\nif false: result = 1 else: result = 2; result = 3\nassert(result, 3)",
        "result = 0\nif false: { result = 1 }\nelse: { result = 2 }\nassert(result, 2)",
        "result = 0\nif true:\n    if false: result = 1\n    else:\n        result = 2\nelse: result = 3\nassert(result, 2)",
        "result = 0\nif false:\n    result = 1\nelse:\n    if false: { result = 2 }\n    else: result = 3\nassert(result, 3)",
        "result = 0\nif false: result = 1\nelse if false:\n    result = 2\nelse:\n    if true:\n        result = 3\n    else:\n        result = 4\nassert(result, 3)",
        "result = 0\nif true: if false: result = 1 else: result = 2\nassert(result, 2)",
    ] {
        check_pair(&else_colons(explicit, true), explicit, 1, None);
    }
}

fn syntax(source: &str, message: &str, position: Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    assert_eq!(error.message(), message, "{source}");
    assert_eq!(error.position(), Some(position), "{source}");
}

#[test]
fn unqualified_new_prefixes_keep_the_actual_colon_required_diagnostics() {
    for tail in [
        "\n",
        "\n# comment",
        "\n    # comment",
        "\npass",
        "\n;\npass",
    ] {
        syntax(
            &format!("if false: pass\nelse{tail}"),
            "Expected ':' but found end of line",
            Position::new(2, 1),
        );
    }
    for (tail, message, column) in [
        ("", "Expected ':' but found end of input", 5),
        (";\n    pass", "Expected ':' but found ';'", 5),
        (" pass", "Expected ':' but found keyword 'pass'", 6),
        (" { pass }", "Expected ':' but found '{'", 6),
        ("; pass", "Expected ':' but found ';'", 5),
    ] {
        syntax(
            &format!("if false: pass\nelse{tail}"),
            message,
            Position::new(2, column),
        );
    }
    syntax(
        "if false: pass\nelse:\npass",
        "Expected an indented block but found keyword 'pass'",
        Position::new(3, 1),
    );
}

#[test]
fn qualifying_bodies_keep_inherited_lexical_and_expression_error_positions() {
    for (body, message, column) in [
        ("value = (1 +)", "Unexpected ')'", 18),
        ("value = @", "Unexpected character '@'", 13),
    ] {
        for header in ["else", "else:"] {
            syntax(
                &format!("if false: pass\n{header}\n    {body}"),
                message,
                Position::new(3, column),
            );
        }
    }
}

#[test]
fn all_other_headers_keep_their_accepted_colon_or_body_requirements() {
    for (source, message, position) in [
        (
            "if true\n    pass",
            "Expected ':' but found end of line",
            Position::new(1, 4),
        ),
        (
            "if false: pass\nelse if true\n    pass",
            "Expected ':' but found end of line",
            Position::new(2, 9),
        ),
        (
            "for item of Items\n    pass",
            "Expected ':' but found end of line",
            Position::new(1, 13),
        ),
        (
            "def f()\n    pass",
            "Expected ':' but found end of line",
            Position::new(1, 7),
        ),
        (
            "class Item\n    pass",
            "Expected ':' but found end of line",
            Position::new(1, 7),
        ),
        (
            "callback = value =>\n    value",
            "Unexpected end of line",
            Position::new(2, 5),
        ),
    ] {
        syntax(source, message, position);
    }
    for source in [
        "class Item",
        "def f value =>\n    value",
        "callback = value => value",
        "callback = function(value) { return value }",
        "try: pass\ncatch error: pass",
    ] {
        Runtime::check(source).unwrap();
    }
}

#[test]
fn newly_qualified_exception_rows_keep_their_complete_accepted_controls() {
    // Exactly the two former negative rows authorized by NUC46's preflight.
    for (actual, explicit) in [
        (
            "try\n    pass\ncatch error:\n    pass",
            "try:\n    pass\ncatch error:\n    pass",
        ),
        (
            "try: pass\ncatch error\n    pass",
            "try: pass\ncatch error:\n    pass",
        ),
    ] {
        assert_eq!(
            equivalent(actual, explicit),
            [Stmt::Try {
                body: vec![Stmt::Pass],
                parameter: "error".to_string(),
                catch: vec![Stmt::Pass],
            }]
        );
        assert_eq!(
            parse_program(actual).unwrap().positions,
            parse_program(explicit).unwrap().positions
        );
        check_pair(actual, explicit, 0, Some("null"));
    }
}

#[test]
fn direct_nodes_preserve_return_flow_caller_scope_receivers_and_error_recovery() {
    for expression in ["value + 1", "missing"] {
        let explicit = format!("if false: pass\nelse:\n    return {expression}");
        let actual = else_colons(&explicit, true);
        assert_eq!(
            statement::compile(&actual).unwrap().statements,
            parse(&explicit).unwrap()
        );
        for source in [&actual, &explicit] {
            let mut runtime = Runtime::new();
            let mut scope = Scope::new();
            scope.push();
            scope.declare("value", Value::Number(99.0));
            let receiver = ObjectId::named("caller");
            scope.set_this(Some(receiver.clone()));
            scope.set_instance(Some(receiver.clone()));
            let statements = parse(source).unwrap();
            let mut node = Nuc::convert(&mut runtime, &mut scope, &statements[0]).unwrap();
            let result = node.run(&mut runtime, &mut scope);
            if expression == "missing" {
                let error = result.err().expect("missing read must fail");
                assert_eq!(error.kind(), ErrorKind::Reference);
                assert_eq!(error.message(), "missing is not defined");
            } else {
                assert!(matches!(
                    result.unwrap().flow,
                    Flow::Return(Value::Number(100.0))
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
fn short_circuit_graph_relationships_and_uncaught_rollback_remain_unchanged() {
    let explicit = "flag = false\ninput = 3\nif flag && missing:\n    output = 1\nelse:\n    output = input\nassert(output, 3)\ninput = 5\nassert(output, 5)";
    check_pair(&else_colons(explicit, true), explicit, 2, None);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for runtime in &mut runtimes {
        runtime
            .run("marker = 7\ninput = 2\noutput = input * 3")
            .unwrap();
    }
    let explicit = "if false: pass\nelse:\n    marker = 9\n    throw \"ABORT\"";
    let actual = else_colons(explicit, true);
    let mut errors = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([&actual, explicit]) {
        let error = runtime.run(source).unwrap_err();
        errors.push((error.kind(), error.message().to_string(), error.position()));
        assert_eq!(runtime.run("marker").unwrap(), Value::Number(7.0));
        runtime.run("input = 4").unwrap();
        assert_eq!(runtime.run("output").unwrap(), Value::Number(12.0));
    }
    assert_eq!(errors[0], errors[1]);
    graphs_match(&runtimes[0], &runtimes[1]);
}

#[test]
fn unchanged_resource_limits_fail_atomically_without_claiming_a_host_stack_repair() {
    for (body, message) in [
        (
            format!("value{}", " + 1".repeat(200)),
            "Expression has too many operands",
        ),
        (
            format!("root{}", ".next".repeat(200)),
            "Expression has too many parts",
        ),
        (
            format!("{}value", "-".repeat(200)),
            "Expressions are nested too deeply",
        ),
    ] {
        for header in ["else", "else:"] {
            let mut runtime = Runtime::new();
            runtime.run("saved = 7").unwrap();
            let error = runtime
                .run(&format!("saved = 3\nif false: pass\n{header}\n    {body}"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        }
    }
    // Exercise the shared nesting bound on an explicitly roomy test stack.
    // This does not change or claim to fix the Windows default-stack boundary.
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let mut source = String::new();
            for depth in 0..80 {
                let indent = "    ".repeat(depth);
                source.push_str(&format!("{indent}if false: pass\n{indent}else\n"));
            }
            source.push_str(&format!("{}pass", "    ".repeat(80)));
            for source in [&source, &else_colons(&source, false)] {
                let error = Runtime::check(source).unwrap_err();
                assert_eq!(error.kind(), ErrorKind::Syntax);
                assert_eq!(error.message(), "Expressions are nested too deeply");
            }
        })
        .unwrap()
        .join()
        .unwrap();
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
fn native_repl_buffers_from_if_through_bare_else_until_blank_commit_or_literal_eof() {
    for ending in ["", "\n", "\n\n"] {
        for newline in ["\n", "\r\n"] {
            let actual = format!(".json on\nresult = 0\nif false:\n\tresult = 1\nelse\n\tresult = 2\n\tresult{ending}").replace('\n', newline);
            let explicit = else_colons(&actual, false);
            let outputs = [repl(&actual), repl(&explicit)];
            for output in &outputs {
                assert!(output.status.success(), "{output:?}");
                assert!(output.stderr.is_empty(), "{output:?}");
                assert!(
                    String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .any(|line| line == "2.0"),
                    "{output:?}"
                );
            }
            assert_eq!(outputs[0].stdout, outputs[1].stdout);
        }
    }
    let output = repl(
        ".json on\nresult = 0\nif false:\n    result = 1\nelse # else: in comment\n    result = 2\n\nresult\n7",
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    for value in ["2.0", "7.0"] {
        assert!(stdout.lines().any(|line| line == value), "{stdout}");
    }
}

#[test]
fn native_repl_preserves_immediate_inline_and_literal_interaction_and_recovers() {
    let output = repl(
        ".json on\nresult = 0\nif false: result = 1 else: result = 2\nresult\n\"else:\"\n/else:/\n`else:`\n1 # else: not a header\n# else: not a header\n7",
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    for value in ["2.0", "1.0", "7.0", "\"else:\""] {
        assert!(stdout.lines().any(|line| line == value), "{stdout}");
    }
    let output = repl(
        ".json on\nsaved = 7\nif false:\n    pass\nelse;\n    saved = 1\n\nsaved\nif false:\n    pass\nelse\n    saved = 9\n\nsaved",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        "SyntaxError: Expected ':' but found ';' (line 3, column 5)"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.lines().filter(|line| *line == "7.0").count() >= 2,
        "{stdout}"
    );
    assert!(stdout.lines().any(|line| line == "9.0"), "{stdout}");
}

#[test]
fn native_files_keep_results_canonical_graphs_and_physical_diagnostics() {
    let directory = std::env::temp_dir().join(format!("nucleoid-nuc34-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("program.nuc");
    for newline in ["\n", "\r\n"] {
        let actual = "flag = false\ninput = 3\nif flag:\n\toutput = 1\nelse\n\toutput = input\nassert(output, 3)\ninput = 5\nassert(output, 5)\noutput".replace('\n', newline);
        let explicit = else_colons(&actual, false);
        let mut previous = None;
        for source in [&actual, &explicit] {
            fs::write(&path, source).unwrap();
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
    for (source, message) in [
        (
            "if false: pass\nelse",
            "SyntaxError: Expected ':' but found end of input (line 2, column 5)",
        ),
        (
            "if false: pass\nelse\n    value = (1 +)",
            "SyntaxError: Unexpected ')' (line 3, column 18)",
        ),
        (
            "if false: pass\nelse\n    missing",
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
    ] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg(&path)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{output:?}");
        assert_eq!(String::from_utf8(output.stderr).unwrap().trim(), message);
    }
    fs::remove_dir_all(directory).unwrap();
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
fn eight_sealed_counterparts_preserve_order_comments_assertions_returns_and_coverage() {
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
                .collect::<Vec<_>>()
        );
        for own in &fixtures {
            let case = cases.iter().find(|case| case.title == own.title).unwrap();
            assert_eq!(case.expected.as_deref(), own.expected);
            let body = case
                .source
                .trim()
                .strip_prefix(&format!("# {}\n\n", case.title))
                .unwrap_or(case.source.trim());
            assert_eq!(body, else_colons(own.explicit, true));
            assert_ne!(body, own.explicit);
            let count =
                common::assertions::protected_assertions(&parse(own.explicit).unwrap()).len();
            assert_eq!(count, own.assertions);
            check_pair(&case.source, own.explicit, count, own.expected);
            checked += 1;
            assertions += count;
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
    assert_eq!(assertions, 45);
    assert_eq!(returns, ["17", "6"]);
}

#[test]
fn rewritten_normative_sources_preserve_complete_asts_canonical_rendering_and_execution() {
    let mut checked = 0;
    for (_, cases) in documents() {
        for case in cases {
            let explicit = else_colons(&case.source, false);
            if explicit == case.source {
                continue;
            }
            let count = common::assertions::protected_assertions(&parse(&explicit).unwrap()).len();
            check_pair(&case.source, &explicit, count, case.expected.as_deref());
            checked += 1;
        }
    }
    assert!(checked >= OWN_CASES.len());
    println!("Complete AST/canonical/graph/runtime pairs: {checked} normative programs");
}

struct OwnCase {
    document: &'static str,
    title: &'static str,
    assertions: usize,
    expected: Option<&'static str>,
    explicit: &'static str,
}

// Accepted-syntax sources from the caller's eight pre-executed v2 inputs.
const OWN_CASES: &[OwnCase] = &[
    OwnCase {
        document: "spec",
        title: "Nucleoid associates nested indented else suites with their conditions",
        assertions: 8,
        expected: Some("17"),
        explicit: r#"class Gauge
gauge = Gauge()
gauge.outer = false
gauge.inner = false
first = 3
second = 5
third = 7
if gauge.outer:
    gauge.result = first
else:
    if gauge.inner:
        gauge.result = second
    else:
        gauge.result = third
assert(gauge.result, 7)
gauge.inner = true
assert(gauge.result, 5)
gauge.outer = true
assert(gauge.result, 3)
first = 11
assert(gauge.result, 11)
gauge.outer = false
assert(gauge.result, 5)
second = 13
assert(gauge.result, 13)
gauge.inner = false
assert(gauge.result, 7)
third = 17
assert(gauge.result, 17)
gauge.result"#,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid keeps indented else class rules independent per instance",
        assertions: 10,
        expected: None,
        explicit: r#"class Tier
first = Tier()
first.score = 1
second = Tier()
second.score = 9
threshold = 5
low = "LOW"
high = "HIGH"
if $Tier.score >= threshold:
    $Tier.label = high
else:
    $Tier.label = low
assert(first.label, "LOW")
assert(second.label, "HIGH")
low = "L"
assert(first.label, "L")
assert(second.label, "HIGH")
first.score = 8
assert(first.label, "HIGH")
assert(second.label, "HIGH")
threshold = 10
assert(first.label, "L")
assert(second.label, "L")
second.score = 12
assert(first.label, "L")
assert(second.label, "HIGH")"#,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid preserves returns from indented else function branches",
        assertions: 4,
        expected: Some("6"),
        explicit: r#"def choose(flag, value):
    if flag:
        return value
    else:
        return value + 1
enabled = false
amount = 2
chosen = choose(enabled, amount)
assert(chosen, 3)
enabled = true
assert(chosen, 2)
amount = 5
assert(chosen, 5)
enabled = false
assert(chosen, 6)
chosen"#,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid rolls back throws from indented else suites",
        assertions: 4,
        expected: None,
        explicit: r#"marker = 7
try:
    if false:
        marker = 8
    else:
        marker = 9
        throw "ABORT"
catch error:
    assert(error, "ABORT")
assert(marker, 7)
gate = false
if gate:
    result = 1
else:
    result = 2
assert(result, 2)
gate = true
assert(result, 1)"#,
    },
    OwnCase {
        document: "spec",
        title: "Nucleoid preserves legacy suites beside indented else bodies",
        assertions: 7,
        expected: None,
        explicit: r#"result = 0
if true: result = 1
else: result = 2
assert(result, 1)
if false:
    result = 3
else:
    result = 4
assert(result, 4)
if true: { result = 5 }
else: { result = 6 }
assert(result, 5)
text = "else:"
assert(text, "else:")
flag = false
value = 2
if flag:
    answer = value
else: # else: in a comment is not a delimiter
    answer = value + 10
assert(answer, 12)
flag = true
assert(answer, 2)
value = 6
assert(answer, 6)"#,
    },
    OwnCase {
        document: "reference",
        title: "Nucleoid preserves live property relationships in indented else suites",
        assertions: 4,
        expected: None,
        explicit: r#"class Signal
signal = Signal()
signal.enabled = false
base = 4
if signal.enabled:
    signal.output = base
else:
    signal.output = -base
assert(signal.output, -4)
base = 6
assert(signal.output, -6)
signal.enabled = true
assert(signal.output, 6)
base = 8
assert(signal.output, 8)"#,
    },
    OwnCase {
        document: "readme",
        title: "Indented else suites keep live inputs concise",
        assertions: 3,
        expected: None,
        explicit: r#"enabled = false
raw = 2
if enabled:
    display = raw
else:
    display = raw + 1
assert(display, 3)
enabled = true
assert(display, 2)
raw = 5
assert(display, 5)"#,
    },
    OwnCase {
        document: "examples",
        title: "Indented else suites preserve conditional chains",
        assertions: 5,
        expected: None,
        explicit: r#"class Stock
item = Stock()
item.quantity = 0
limit = 5
if item.quantity > limit:
    item.status = "HIGH"
else if item.quantity > 0:
    item.status = "LOW"
else:
    item.status = "EMPTY"
assert(item.status, "EMPTY")
item.quantity = 2
assert(item.status, "LOW")
item.quantity = 8
assert(item.status, "HIGH")
limit = 10
assert(item.status, "LOW")
item.quantity = 0
assert(item.status, "EMPTY")"#,
    },
];
