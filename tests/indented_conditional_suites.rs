//! Conditional colon omission is additive, source-only and indentation-bound.

mod common;
#[allow(dead_code)]
#[path = "support/conditional_headers.rs"]
mod conditional_headers;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use conditional_headers::omit;
use nucleoid::graph::{NodeKey, NodeKind};
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_program};
use nucleoid::lang::evaluation::Flow;
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn equivalent(actual: &str, control: &str) -> Vec<Stmt> {
    let actual_program = parse_program(actual).unwrap_or_else(|error| panic!("{actual}\n{error}"));
    let control_program =
        parse_program(control).unwrap_or_else(|error| panic!("{control}\n{error}"));
    assert_eq!(
        actual_program.statements, control_program.statements,
        "{actual}"
    );
    assert_eq!(
        actual_program.positions, control_program.positions,
        "{actual}"
    );
    assert_eq!(
        generate_all(&actual_program.statements),
        generate_all(&control_program.statements)
    );
    assert_eq!(
        common::assertions::protected_assertions(&actual_program.statements),
        common::assertions::protected_assertions(&control_program.statements)
    );
    actual_program.statements
}

fn pair(actual: &str, control: &str, count: usize, expected: Option<&str>) -> [Runtime; 2] {
    let ast = equivalent(actual, control);
    assert_eq!(common::assertions::protected_assertions(&ast).len(), count);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let values = runtimes
        .iter_mut()
        .zip([actual, control])
        .map(|(runtime, source)| {
            let value = common::check_with_count(
                runtime,
                &common::Case {
                    title: "conditional suite pair".to_string(),
                    source: source.to_string(),
                    expected: expected.map(str::to_string),
                },
                count,
            )
            .unwrap_or_else(|error| panic!("{source}\n{error}"));
            assert_eq!(runtime.assertions_run(), count, "{source}");
            value
        })
        .collect::<Vec<_>>();
    // Includes node keys, complete node metadata, sequence, instance, ordered
    // dependencies/dependents, shape reads and state modulo fresh object IDs.
    runtime_equivalence::assert_equivalent(
        &runtimes[0],
        Some(&values[0]),
        &runtimes[1],
        Some(&values[1]),
    );
    runtimes
}

#[test]
fn each_conditional_path_reuses_complete_asts_canonical_source_and_positions() {
    for condition in [
        "ready",
        "not ready and available",
        "first = second",
        "items[1:3] == [2, 3]",
        "items[:].length > 0",
        "({ready: {nested: true}}).ready.nested",
        "check({ready: value}, items[:], (item: int) => item)",
        "(item: int) => { return item }",
        "function(item) { return item }",
        "(\n    ready # : [} are only a comment\n)",
        "({\n    ready: true\n}).ready",
        "\"if : else [}\" == \"if : else [}\"",
        "/[:{}]/.test(text)",
        "`else if ${value}:`",
    ] {
        for layout in [
            "\n    pass",
            "\r\n\tpass",
            " # comment : [}\n\n# blank\n    # body\n    pass",
            "\n;\n    pass",
            "\n    ;\n    pass;",
        ] {
            for prefix in ["if", "else if", "elif if", "else", "elif"] {
                let before = if prefix == "if" {
                    ""
                } else {
                    "if false: pass\n"
                };
                let control = format!("{before}{prefix} {condition}:{layout}\nafter = 3");
                let (actual, sites) = omit(&control);
                assert_eq!(sites, 1, "{control}");
                let ast = equivalent(&actual, &control);
                assert!(matches!(ast[0], Stmt::If { .. }));
            }
        }
    }
    for suffix in ["", "\n", "\n# EOF", "\nafter = 3", "\n; after = 3"] {
        let control =
            format!("if false:\n    pass\nelse if true:\n    pass\nelse:\n    pass{suffix}");
        let (actual, sites) = omit(&control);
        assert_eq!(sites, 2);
        assert!(actual.contains("\nelse:\n"));
        pair(&actual, &control, 0, None);
    }
    assert_eq!(
        parse_program("if true\n    pass\nelse if false\n    pass\nelse\n    pass\nafter = 3")
            .unwrap()
            .positions,
        [Position::new(1, 1), Position::new(7, 1)]
    );
}

#[test]
fn lexical_edits_leave_literal_comment_annotation_object_and_slice_colons_exact() {
    let control = "text = \"if true:\nelse:\n    café😀\"\npattern = /if:else:/\n\
                   template = `if:${text}:else:`\n\
                   if ((item: int) => { return item })([1, 2][0:1][0]): # if: else:\n\
                   \x20   result = {label: \"café😀:\"}\nelse true: # elif:\n    pass\n\
                   else:\n    pass";
    let (actual, sites) = omit(control);
    assert_eq!(sites, 2);
    assert!(actual.contains("if ((item: int) => { return item })([1, 2][0:1][0]) # if: else:"));
    assert!(actual.contains("result = {label: \"café😀:\"}"));
    assert!(actual.ends_with("else:\n    pass"));
    equivalent(&actual, control);
    let mut old = tokenize(control)
        .unwrap()
        .into_iter()
        .map(|s| s.token)
        .collect::<Vec<_>>();
    let new = tokenize(&actual)
        .unwrap()
        .into_iter()
        .map(|s| s.token)
        .collect::<Vec<_>>();
    let mut index = 0;
    old.retain(|token| {
        if new.get(index) == Some(token) {
            index += 1;
            true
        } else {
            assert_eq!(*token, Token::Colon);
            false
        }
    });
    assert_eq!(old, new);
}

#[test]
fn fixture_edits_skip_both_final_aliases_while_both_optional_colon_forms_still_execute() {
    for header in ["else:", "elif:", "else", "elif"] {
        for newline in ["\n", "\r\n"] {
            for indent in ["    ", "\t"] {
                let source = format!(
                    "if false:\n{indent}result = 1\n{header} # final : [}}\n\
                     {indent}result = 2\nassert(result, 2)\nresult"
                )
                .replace('\n', newline);
                let (actual, sites) = omit(&source);
                assert_eq!(sites, 1);
                assert_eq!(actual, source.replacen("if false:", "if false", 1));
                let control = format!(
                    "if false:\n{indent}result = 1\nelse:\n\
                     {indent}result = 2\nassert(result, 2)\nresult"
                )
                .replace('\n', newline);
                pair(&actual, &control, 1, Some("2"));
            }
        }
    }
}

#[test]
fn mixed_legacy_suites_nested_chains_and_dangling_else_keep_the_same_association() {
    for control in [
        "if true:\n    if false: result = 1\n    else if true:\n        result = 2\n    else: result = 3\nelse if true:\n    result = 4\nassert(result, 2)",
        "if false: { result = 1 }\nelse true:\n    if false:\n        result = 2\n    elif true:\n        result = 3\n    else\n        result = 4\nelse: { result = 5 }\nassert(result, 3)",
        "if false: result = 1\nelse if false: { result = 2 }\nelse if true:\n    result = 3\n    if false: result = 4 else: result = 5\nelif: result = 6\nassert(result, 5)",
        "if true: if false: result = 1 else if true:\n    result = 2\nelse: result = 3\nassert(result, 2)",
        "if false:\n    result = 0\nelif if false: result = 1; result = 2\nelif if true:\n    result = 3\nelif:\n    result = 4\nassert(result, 3)",
    ] {
        pair(&omit(control).0, control, 1, None);
    }
    for condition in ["0", "null", "\"\"", "false", "[]"] {
        let control = format!("if {condition}:\n    result = 1\nelse:\n    result = 2\nresult");
        pair(&omit(&control).0, &control, 0, None);
    }
}

fn dependencies(runtime: &Runtime) -> Vec<&str> {
    runtime
        .graph
        .retrieve(&NodeKey::conditional(
            &Expr::Identifier("first".into()),
            None,
        ))
        .unwrap()
        .dependencies
        .iter()
        .map(NodeKey::as_str)
        .collect()
}

#[test]
fn actual_short_circuit_reads_rule_replacement_and_failed_updates_preserve_full_graph_metadata() {
    let control = "if first:\n    selected = active\nelse second and missing:\n    selected = \"bad\"\nelif true:\n    selected = fallback";
    let actual = omit(control).0;
    equivalent(&actual, control);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, source) in runtimes.iter_mut().zip([actual.as_str(), control]) {
        runtime
            .run("first = true\nsecond = false\nactive = \"a\"\nfallback = \"b\"")
            .unwrap();
        runtime.run(source).unwrap();
        assert_eq!(runtime.run("selected").unwrap(), Value::string("a"));
        assert_eq!(dependencies(runtime), ["first", "active"]);
    }
    runtime_equivalence::assert_equivalent(&runtimes[0], None, &runtimes[1], None);
    for runtime in &mut runtimes {
        runtime.run("first = false").unwrap();
        assert_eq!(runtime.run("selected").unwrap(), Value::string("b"));
        assert_eq!(dependencies(runtime), ["first", "second", "fallback"]);
        let error = runtime.run("second = true").unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(runtime.run("second").unwrap(), Value::Bool(false));
        assert_eq!(runtime.run("selected").unwrap(), Value::string("b"));
        assert_eq!(dependencies(runtime), ["first", "second", "fallback"]);
        runtime.run("fallback = \"c\"").unwrap();
        assert_eq!(runtime.run("selected").unwrap(), Value::string("c"));
        assert_eq!(
            runtime
                .graph
                .nodes()
                .filter(|node| node.kind == NodeKind::If)
                .count(),
            1
        );
    }
    runtime_equivalence::assert_equivalent(&runtimes[0], None, &runtimes[1], None);
    let replacement = "if first:\n    selected = fallback\nelse if true:\n    selected = active";
    for (runtime, source) in runtimes
        .iter_mut()
        .zip([omit(replacement).0.as_str(), replacement])
    {
        runtime.run(source).unwrap();
        assert_eq!(runtime.run("selected").unwrap(), Value::string("a"));
        assert_eq!(dependencies(runtime), ["first", "active"]);
    }
    runtime_equivalence::assert_equivalent(&runtimes[0], None, &runtimes[1], None);
}

#[test]
fn per_instance_class_rules_and_other_standing_rules_remain_independent() {
    let control = "class Parcel\nfirst = Parcel()\nfirst.state = \"queued\"\n\
                   second = Parcel()\nsecond.state = \"ready\"\n\
                   if $Parcel.state == \"blocked\":\n    $Parcel.lane = \"hold\"\n\
                   else $Parcel.state == \"ready\":\n    $Parcel.lane = \"send\"\n\
                   elif:\n    $Parcel.lane = \"wait\"\n\
                   $Parcel.label = $Parcel.lane + \"!\"\n\
                   assert(first.lane, \"wait\")\nassert(second.label, \"send!\")\n\
                   first.state = \"blocked\"\nassert(first.label, \"hold!\")\n\
                   assert(second.lane, \"send\")\nsecond.state = \"queued\"\n\
                   assert(first.lane, \"hold\")\nassert(second.label, \"wait!\")\nsecond.label";
    pair(&omit(control).0, control, 6, Some("\"wait!\""));
}

#[test]
fn function_caller_receiver_scopes_explicit_returns_and_noncapture_stay_unchanged() {
    let control = "class Router:\n    def init(prefix):\n        this.prefix = prefix\n\
                   \x20   def choose(status):\n        if status:\n            return this.prefix + status\n\
                   \x20       else if false:\n            return \"bad\"\n        else:\n            return\n\
                   router = Router(\"a:\")\nstatus = \"ready\"\nlive = router.choose(status)\n\
                   snapshot = live.value\n\
                   assert(live, \"a:ready\")\nassert(snapshot, \"a:ready\")\n\
                   assert(router.choose(false), null)\nassert(status, \"ready\")";
    pair(&omit(control).0, control, 4, None);
    let control = "value = 3\ndef read():\n    if false:\n        return 0\n\
                   \x20   else true:\n        return value\n\
                   def invoke(value):\n    return read()\nassert(invoke(99), 3)";
    pair(&omit(control).0, control, 1, None);
    for expression in ["value + 1", "missing"] {
        let control = format!("if false:\n    pass\nelse true:\n    return {expression}");
        let actual = omit(&control).0;
        let ast = equivalent(&actual, &control);
        for source in [&actual, &control] {
            let mut runtime = Runtime::new();
            let mut scope = Scope::new();
            scope.push();
            scope.declare("value", Value::Number(99.0));
            let receiver = ObjectId::named("caller");
            scope.set_this(Some(receiver.clone()));
            scope.set_instance(Some(receiver.clone()));
            let statements = parse(source).unwrap();
            assert_eq!(statements, ast);
            let mut node = Nuc::convert(&mut runtime, &mut scope, &statements[0]).unwrap();
            let result = node.run(&mut runtime, &mut scope);
            if expression == "missing" {
                let error = result.err().expect("missing must fail");
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
        }
    }
    let control = "def factory(hidden):\n    if true:\n        return value => hidden + value\nfactory(\"x\")(\"y\")";
    let actual = omit(control).0;
    equivalent(&actual, control);
    for source in [actual.as_str(), control] {
        let error = Runtime::new().run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "hidden is not defined");
        assert_eq!(error.position(), Some(Position::new(4, 1)));
    }
}

#[test]
fn caught_and_uncaught_failures_restore_transactions_and_allow_later_recovery() {
    let control = "source = 2\nderived = source * 3\nsnapshot = derived.value\n\
                   try:\n    if source > 0:\n        source = 9\n        throw \"ABORT\"\n\
                   \x20   else true:\n        throw \"WRONG\"\ncatch reason:\n    assert(reason, \"ABORT\")\n\
                   assert(source, 2)\nassert(derived, 6)\nsource = 4\n\
                   assert(derived, 12)\nassert(snapshot, 6)\nderived";
    pair(&omit(control).0, control, 5, Some("12"));
    for control in [
        "if true:\n    marker = 9\n    throw \"STOP\"",
        "if false:\n    pass\nelse if true:\n    marker = 9\n    throw \"STOP\"",
        "if false:\n    pass\nelif true:\n    marker = 9\n    throw \"STOP\"",
    ] {
        let actual = omit(control).0;
        equivalent(&actual, control);
        let mut runtimes = [Runtime::new(), Runtime::new()];
        let mut errors = Vec::new();
        for (runtime, source) in runtimes.iter_mut().zip([actual.as_str(), control]) {
            runtime
                .run("marker = 7\ninput = 2\noutput = input * 3")
                .unwrap();
            errors.push(runtime.run(source).unwrap_err());
            assert_eq!(runtime.run("marker").unwrap(), Value::Number(7.0));
            runtime.run("input = 4").unwrap();
            assert_eq!(runtime.run("output").unwrap(), Value::Number(12.0));
        }
        assert_eq!(errors[0], errors[1]);
        assert_eq!(errors[0].kind(), ErrorKind::Thrown);
        assert_eq!(errors[0].message(), "STOP");
        assert_eq!(errors[0].position(), Some(Position::new(1, 1)));
        runtime_equivalence::assert_equivalent(&runtimes[0], None, &runtimes[1], None);
    }
}

fn syntax(source: &str, message: &str, line: usize, column: usize) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    assert_eq!(error.message(), message, "{source}");
    assert_eq!(
        error.position(),
        Some(Position::new(line, column)),
        "{source}"
    );
}

#[test]
fn nonqualifying_layout_keeps_inherited_errors_and_exact_source_anchors() {
    for (tail, message, column) in [
        ("", "Expected ':' but found end of input", 8),
        ("\n", "Expected ':' but found end of line", 4),
        ("\n    # no body", "Expected ':' but found end of line", 4),
        ("\npass", "Expected ':' but found end of line", 4),
        (";\n    pass", "Expected ':' but found ';'", 8),
        (" pass", "Expected ':' but found keyword 'pass'", 9),
        (" { pass }", "Expected ':' but found '{'", 9),
    ] {
        syntax(&format!("if true{tail}"), message, 1, column);
    }
    for prefix in ["else if", "elif if"] {
        syntax(
            &format!("if false: pass\n{prefix} true\npass"),
            "Expected ':' but found end of line",
            2,
            9,
        );
        syntax(
            &format!("if false: pass\n{prefix} true;\n    pass"),
            "Expected ':' but found ';'",
            2,
            13,
        );
    }
    for prefix in ["else", "elif"] {
        for tail in [
            "ready",
            "ready\n",
            "ready\n    # no body",
            "ready\npass",
            "ready pass",
            "ready { pass }",
            "ready;\n    pass",
            "ready: pass",
            "ready: { pass }",
            "ready:;\n    pass",
            "(ready]\n    pass",
            "ready[)\n    pass",
            "(ready\n    pass",
        ] {
            let found = if tail.starts_with('(') {
                "'('"
            } else {
                "identifier 'ready'"
            };
            syntax(
                &format!("if false: pass\n{prefix} {tail}"),
                &format!("Expected ':' but found {found}"),
                2,
                6,
            );
        }
        syntax(
            &format!("{prefix} true\n    pass"),
            "Unexpected keyword 'else'",
            1,
            6,
        );
    }
    for (source, message, line, column) in [
        (
            "for item of Items\n    pass",
            "Expected ':' but found end of line",
            1,
            13,
        ),
        (
            "def f()\n    pass",
            "Expected ':' but found end of line",
            1,
            7,
        ),
        (
            "class Item\n    pass",
            "Expected ':' but found end of line",
            1,
            7,
        ),
        (
            "try pass\ncatch: pass",
            "Expected ':' but found keyword 'pass'",
            1,
            5,
        ),
    ] {
        syntax(source, message, line, column);
    }
}

#[test]
fn qualifying_malformed_expressions_and_bodies_are_reported_by_the_real_parser() {
    for prefix in ["if", "else if", "elif if", "else", "elif"] {
        let before = if prefix == "if" {
            ""
        } else {
            "if false: pass\n"
        };
        for (body, message, column) in [
            ("value = (1 +)", "Unexpected ')'", 18),
            ("value = @", "Unexpected character '@'", 13),
        ] {
            for colon in ["", ":"] {
                syntax(
                    &format!("{before}{prefix} true{colon}\n    {body}"),
                    message,
                    if prefix == "if" { 2 } else { 3 },
                    column,
                );
            }
        }
        let source = format!("{before}{prefix} true +\n    pass");
        let error = Runtime::check(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), "Unexpected end of line");
        assert_eq!(
            error.position(),
            Some(Position::new(if prefix == "if" { 2 } else { 3 }, 5))
        );
    }
}

#[test]
fn inherited_expression_chain_and_statement_bounds_are_neither_bypassed_nor_relaxed() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            for prefix in ["if", "else if", "elif if", "else", "elif"] {
                let before = if prefix == "if" {
                    ""
                } else {
                    "if false: pass\n"
                };
                for count in [127, 128, 129] {
                    for (condition, allowed, message) in [
                        (
                            format!("value{}", " + 1".repeat(count)),
                            count <= 128,
                            "Expression has too many operands",
                        ),
                        (
                            format!("root{}", ".next".repeat(count)),
                            count < 128,
                            "Expression has too many parts",
                        ),
                    ] {
                        let control = format!("{before}{prefix} {condition}:\n    pass");
                        let actual = omit(&control).0;
                        if allowed {
                            equivalent(&actual, &control);
                        } else {
                            for source in [&actual, &control] {
                                let error = Runtime::check(source).unwrap_err();
                                assert_eq!(error.kind(), ErrorKind::Syntax);
                                assert_eq!(error.message(), message);
                                let mut runtime = Runtime::new();
                                runtime.run("kept = 7").unwrap();
                                assert!(runtime.run(&format!("kept = 3\n{source}")).is_err());
                                assert_eq!(
                                    runtime.state.variable("kept"),
                                    Some(&Value::Number(7.0))
                                );
                            }
                        }
                    }
                }
                for condition in [
                    format!("{}value", "-".repeat(80)),
                    format!("{}true{}", "(".repeat(80), ")".repeat(80)),
                ] {
                    let control = format!("{before}{prefix} {condition}:\n    pass");
                    for source in [omit(&control).0.as_str(), &control] {
                        assert_eq!(
                            Runtime::check(source).unwrap_err().message(),
                            "Expressions are nested too deeply"
                        );
                    }
                }
            }
            let mut control = String::new();
            for depth in 0..80 {
                control.push_str(&format!("{}if true:\n", "    ".repeat(depth)));
            }
            control.push_str(&format!("{}pass", "    ".repeat(80)));
            let actual = omit(&control).0;
            let errors = [
                Runtime::check(&actual).unwrap_err(),
                Runtime::check(&control).unwrap_err(),
            ];
            assert_eq!(errors[0], errors[1]);
            assert_eq!(errors[0].message(), "Expressions are nested too deeply");
        })
        .unwrap()
        .join()
        .unwrap();
}
