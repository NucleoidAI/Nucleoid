mod common;

use std::io::Write as _;
use std::process::{Command, Stdio};

use nucleoid::lang::ast::generator::{generate, generate_all};
use nucleoid::lang::ast::lexer::tokenize;
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_program};
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn checked(source: &str) -> Runtime {
    let mut runtime = Runtime::new();
    common::check_with_count(
        &mut runtime,
        &common::Case {
            title: "deferred suite regression".to_string(),
            source: source.to_string(),
            expected: Some("1".to_string()),
        },
        common::assertions::protected_assertions(&parse(source).unwrap()).len(),
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    runtime
}

fn result(source: &str) -> String {
    let mut runtime = Runtime::new();
    match runtime.run(source) {
        Ok(value) => runtime.serialize_json(&value).unwrap(),
        Err(error) => error.to_string(),
    }
}

#[test]
fn indented_defer_builds_one_additive_statement_with_original_top_level_positions() {
    let wanted = vec![
        Stmt::Defer(vec![Stmt::Pass]),
        Stmt::Return(Some(Expr::Number(1.0))),
    ];
    for source in [
        "defer:\n    pass\nreturn 1",
        "defer :\n    pass\nreturn 1",
        "defer: # later\n    pass\nreturn 1",
        "defer:\n\n# note\n    pass\nreturn 1",
        "defer:\r\n\tpass\r\nreturn 1",
    ] {
        assert_eq!(parse(source).unwrap(), wanted, "{source}");
    }
    let program = parse_program("defer:\n    pass\nreturn 1").unwrap();
    assert_eq!(
        program.positions,
        [Position::new(1, 1), Position::new(3, 1)]
    );
    assert_eq!(generate(&wanted[0]), "defer{pass}");
    assert_eq!(generate(&Stmt::Block(vec![Stmt::Pass])), "{pass}");
    assert!(matches!(
        parse("defer: Type").unwrap().as_slice(),
        [Stmt::Declaration { name, type_name }] if name == "defer" && type_name == "Type"
    ));
}

#[test]
fn legacy_names_keep_parse_debug_render_layout_tokens_results_and_diagnostics() {
    for source in [
        "defer = 3\ndefer",
        "defer: Type",
        "defer: Type; 1",
        "defer: Type\n    2",
        "defer: Type\n    pass",
        "defer: 1",
        "defer: pass",
        "defer: { 1 }",
        "defer:\n1",
        "defer = 4\ndefer\n    2",
        "defer = 4\n1 defer\n    2",
        "1 defer:\n    2",
        "defer = 2\nx = true ? defer : 1\nx",
        "x = {defer: 2}\nx.defer",
        "x = {a: 1 defer: 2}\nx.defer",
        "defer = 1\nx = [1, 2, 3]\nx[defer:]",
        "def defer(v):\n    return v * 2\ndefer(4)",
        "def apply(defer):\n    return defer + 2\napply(4)",
        "class defer(v)\nx = new defer(3)\nx.v",
        "enum defer:\n    one two\ndefer.two",
        "defer = 5\nf = (+ defer)\nf(2)",
        "def identity(x) => x\nidentity((defer: Type))",
        "def identity(x) => x\nidentity(() => {\n    defer:\n        1\n})",
        "items = [() => {\n    defer:\n        1\n}]",
        "defer = 5\ndef f():\n    return 1\n    defer: Type\nf()",
        "defer = 7\ndef f():\n    defer\n        return 2\nf()",
        "finally = 4\nrepeat = 2\nwhile = 1\ndefer = finally + repeat + while\ndefer",
    ] {
        let ordinary = source.replace("defer", "later");
        let normalize = |text: String| text.replace("later", "defer");
        assert_eq!(
            format!("{:?}", tokenize(source)),
            normalize(format!("{:?}", tokenize(&ordinary))),
            "{source}"
        );
        assert_eq!(
            format!("{:?}", parse_program(source)),
            normalize(format!("{:?}", parse_program(&ordinary))),
            "{source}"
        );
        if let (Ok(actual), Ok(expected)) = (parse(source), parse(&ordinary)) {
            assert_eq!(
                generate_all(&actual),
                normalize(generate_all(&expected)),
                "{source}"
            );
        }
        assert_eq!(result(source), normalize(result(&ordinary)), "{source}");
    }
}

#[test]
fn nonqualifying_suite_forms_keep_their_legacy_errors_and_try_still_requires_catch() {
    for source in [
        "defer: 1",
        "defer: pass",
        "defer: { pass }",
        "defer:\npass",
        "1 defer:\n    pass",
        "f(() => {\n    defer:\n        pass\n})",
        "xs = [() => {\n    defer:\n        pass\n}]",
    ] {
        let error = Runtime::check(source).unwrap_err();
        let legacy = Runtime::check(&source.replace("defer", "later")).unwrap_err();
        assert_eq!(error.kind(), legacy.kind(), "{source}");
        assert_eq!(error.message(), legacy.message(), "{source}");
        assert_eq!(error.position(), legacy.position(), "{source}");
    }
    for source in [
        "try:\n    pass\nfinally:\n    pass",
        "try:\n    defer:\n        pass\nfinally:\n    pass",
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), "Expected 'catch' after 'try'");
    }
}

#[test]
fn registrations_run_only_when_reached_and_keep_normal_values() {
    checked(
        "history = []\ndef complete(value):\n    defer:\n        history.push(value)\n    if value > 0:\n        return value * 2\n    if value < 0:\n        return -value\n    return 0\nassert(complete(3), 6)\nassert(complete(-4), 4)\nassert(complete(0), 0)\nassert(history, [3, -4, 0])\n1",
    );
    checked(
        "history = []\ndef early():\n    defer:\n        history.push(1)\n    return 4\n    defer:\n        history.push(2)\nassert(early(), 4)\nassert(history, [1])\nif false:\n    defer:\n        history.push(3)\nassert(history, [1])\n1",
    );
    assert_eq!(result("defer:\n    9\n3"), "3.0");
    assert_eq!(result("3\ndefer:\n    9"), "3.0");
    assert_eq!(result("defer:\n    9"), "null");
}

#[test]
fn deferred_statements_read_exit_values_without_changing_computed_return_values() {
    checked(
        "history = []\ndef snapshot(value):\n    defer:\n        history.push(value)\n        value = 9\n    value = 4\n    return [value]\nassert(snapshot(2), [4])\nassert(history, [4])\n1",
    );
    checked(
        "history = []\ndef snapshot(value):\n    defer:\n        history.push(value)\n    defer:\n        value = 8\n    return value\nassert(snapshot(2), 2)\nassert(history, [8])\n1",
    );
}

#[test]
fn lifo_returns_and_errors_replace_pending_outcomes_without_skipping_older_suites() {
    checked(
        "history = []\ndef unwind(value):\n    defer:\n        history.push(value * 10 + 1)\n    defer:\n        history.push(value * 10 + 2)\n    return value\nassert(unwind(2), 2)\nassert(history, [22, 21])\n1",
    );
    checked(
        "history = []\ndef replace():\n    defer:\n        history.push(1)\n        return 3\n    defer:\n        history.push(2)\n        return 2\n    return 1\nassert(replace(), 3)\nassert(history, [2, 1])\n1",
    );
    checked(
        "history = []\ndef replace():\n    defer:\n        history.push(1)\n        return 3\n    defer:\n        history.push(2)\n        throw 'cleanup'\n    throw 'body'\nassert(replace(), 3)\nassert(history, [1])\n1",
    );
    let error = Runtime::new()
        .run("defer:\n    throw 'older'\ndefer:\n    return 2\nreturn 1")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Thrown);
    assert_eq!(error.message(), "older");
}

#[test]
fn failed_work_is_rolled_back_before_cleanup_and_failed_cleanup_before_older_cleanup() {
    checked(
        "state = 0\ndef recover(value):\n    defer:\n        return [state, value]\n    defer:\n        state = 99\n        throw 'cleanup'\n    state = 5\n    value = 7\n    throw 'body'\nassert(recover(1), [0, 7])\nassert(state, 0)\n1",
    );
    checked(
        "state = 0\ndef recover():\n    defer:\n        return state\n    state = 5\n    throw 'body'\nassert(recover(), 0)\nassert(state, 0)\n1",
    );
}

#[test]
fn escaping_error_undoes_successful_cleanup_with_the_enclosing_transaction() {
    let mut runtime = Runtime::new();
    runtime.run("state = 0").unwrap();
    let error = runtime
        .run("defer:\n    state = 7\nthrow 'body'")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Thrown);
    assert_eq!(error.message(), "body");
    assert_eq!(error.position(), Some(Position::new(3, 1)));
    assert_eq!(runtime.run("state").unwrap(), Value::Number(0.0));
    assert_eq!(runtime.run("3").unwrap(), Value::Number(3.0));
    assert_eq!(runtime.run("state").unwrap(), Value::Number(0.0));
    checked(
        "state = 0\ntry:\n    defer:\n        state = 7\n    throw 'body'\ncatch reason:\n    assert(reason, 'body')\nassert(state, 0)\n1",
    );
}

#[test]
fn unreplaced_errors_keep_kind_message_and_position_and_replacements_get_their_own_position() {
    for (source, original) in [
        ("defer:\n    pass\nmissing", "pass\npass\nmissing"),
        ("defer:\n    pass\nthrow 7", "pass\npass\nthrow 7"),
        (
            "defer:\n    pass\nMath.unknown()",
            "pass\npass\nMath.unknown()",
        ),
    ] {
        let error = Runtime::new().run(source).unwrap_err();
        let original = Runtime::new().run(original).unwrap_err();
        assert_eq!(error.kind(), original.kind());
        assert_eq!(error.message(), original.message());
        assert_eq!(error.position(), original.position());
    }
    let error = Runtime::new()
        .run("defer:\n    missing_cleanup\nthrow 7")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing_cleanup is not defined");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn nested_blocks_unwind_before_their_parents_and_before_existing_finally_clauses() {
    checked(
        "history = []\ndef nested():\n    defer:\n        history.push(1)\n    try:\n        defer:\n            history.push(2)\n        while true:\n            defer:\n                history.push(3)\n            history.push(4)\n            return 8\n    catch reason:\n        throw reason\n    finally:\n        history.push(5)\nassert(nested(), 8)\nassert(history, [4, 3, 2, 5, 1])\n1",
    );
    checked(
        "history = []\n{\n    defer:\n        history.push(1)\n    {\n        defer:\n            history.push(2)\n        history.push(3)\n    }\n    history.push(4)\n}\nassert(history, [3, 2, 4, 1])\n1",
    );
}

#[test]
fn parameter_loop_catch_and_receiver_scopes_remain_active_during_cleanup() {
    checked(
        "history = []\ndef collect(value):\n    defer:\n        history.push(value)\n    for letter of 'ab':\n        defer:\n            history.push(letter)\n        history.push('body')\n    try:\n        throw 'caught'\n    catch reason:\n        defer:\n            history.push(reason)\n        history.push('catch')\n    return value\nassert(collect(4), 4)\nassert(history, ['body', 'a', 'body', 'b', 'catch', 'caught', 4])\n1",
    );
    checked(
        "history = []\nclass Box(amount):\n    defer:\n        this.amount = amount\n    this.ready = true\n    def read(extra):\n        defer:\n            history.push(this.amount + extra)\n        return this.amount\nbox = new Box(4)\nassert(box.amount, 4)\nassert(box.read(3), 4)\nassert(history, [7])\n1",
    );
}

#[test]
fn loops_keep_their_existing_return_behavior_with_a_fresh_defer_frame_per_iteration() {
    checked(
        "history = []\ndef leave():\n    for letter of 'abc':\n        defer:\n            history.push(letter)\n        return letter\nassert(leave(), 'a')\nassert(history, ['a'])\nrepeat 3:\n    defer:\n        history.push(1)\nassert(history, ['a', 1, 1, 1])\n1",
    );
    checked(
        "history = []\nindex = 0\nwhile index < 3:\n    defer:\n        history.push(index)\n    index = index + 1\nassert(history, [1, 2, 3])\ndef leave():\n    repeat 2:\n        defer:\n            history.push(4)\n        return 6\nassert(leave(), 6)\nassert(history, [1, 2, 3, 4])\n1",
    );
}

#[test]
fn deferred_suites_may_register_bounded_child_suites_but_not_extend_the_parent_queue() {
    checked(
        "history = []\ndefer:\n    history.push(1)\ndefer:\n    defer:\n        history.push(2)\n    history.push(3)\nreturn 1",
    );
    let mut runtime = Runtime::new();
    runtime
        .run("history = []\ndefer:\n    history.push(1)\ndefer:\n    defer:\n        history.push(2)\n    history.push(3)")
        .unwrap();
    assert_eq!(
        runtime.run("history").unwrap(),
        Value::List(vec![
            Value::Number(3.0),
            Value::Number(2.0),
            Value::Number(1.0)
        ])
    );
    let mut source = "history = []\n".to_string();
    for number in 0..256 {
        source.push_str(&format!("defer:\n    history.push({number})\n"));
    }
    runtime.run(&source).unwrap();
    assert_eq!(runtime.run("history.length").unwrap(), Value::Number(256.0));
    assert_eq!(runtime.run("history[0]").unwrap(), Value::Number(255.0));
    assert_eq!(runtime.run("history[255]").unwrap(), Value::Number(0.0));
}

#[test]
fn parsing_and_cleanup_recursion_keep_the_existing_depth_limits_and_runtime_recovers() {
    let mut nested = String::new();
    for depth in 0..100 {
        nested.push_str(&format!("{}defer:\n", "    ".repeat(depth)));
    }
    nested.push_str(&format!("{}pass", "    ".repeat(100)));
    let error = Runtime::check(&nested).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(error.message(), "Statements are nested too deeply");
    let mut runtime = Runtime::new();
    let error = runtime
        .run("def descend():\n    defer:\n        return descend()\n    return 1\ndescend()")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Maximum statement depth exceeded");
    assert_eq!(runtime.run("7").unwrap(), Value::Number(7.0));
}

#[test]
fn default_bearing_declarations_render_defer_as_a_real_indented_suite() {
    let source = "def complete(value=2):\n    defer:\n        value = 9\n    return value";
    let tree = parse(source).unwrap();
    let generated = generate(&tree[0]);
    assert!(generated.contains("\n    defer:\n        value=9"));
    assert_eq!(parse(&generated).unwrap(), tree);
    assert_eq!(result(&format!("{generated}\ncomplete()")), "2.0");
}

#[test]
fn direct_node_conversion_rejects_registration_without_an_executable_block() {
    let statement = Stmt::Defer(vec![Stmt::Pass]);
    let error = Nuc::convert(&mut Runtime::new(), &mut Scope::new(), &statement).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(
        error.message(),
        "Deferred suite requires an executable statement block"
    );
    assert!(matches!(
        Nuc::convert(&mut Runtime::new(), &mut Scope::new(), &Stmt::Pass).unwrap(),
        Nuc::Pass
    ));
}

#[test]
fn assertion_integrity_includes_deferred_bodies_in_source_order_and_executes_them() {
    let source = "defer:\n    assert(1, 1)\nassert(2, 2)\n1";
    assert_eq!(
        common::assertions::protected_assertions(&parse(source).unwrap()),
        ["assert(1,1)", "assert(2,2)"]
    );
    assert_eq!(checked(source).assertions_run(), 2);
}

#[test]
fn exactly_two_paired_document_cases_preserve_original_assertions_and_expected_returns() {
    for (document, title, original) in [
        (
            include_str!("../nucleoid.spec.md"),
            "Nucleoid runs deferred suites on block exit",
            "audit = []\ndef complete(value):\n    try:\n        if value > 0:\n            return value * 2\n        if value < 0:\n            return -value\n        return 0\n    catch reason:\n        throw reason\n    finally:\n        audit.push(value)\nassert(complete(3), 6)\nassert(audit, [3])\nassert(complete(-4), 4)\nassert(audit, [3, -4])\nassert(complete(0), 0)\nassert(audit, [3, -4, 0])\n1",
        ),
        (
            include_str!("reference.md"),
            "Nucleoid unwinds deferred suites in reverse order",
            "history = []\ndef unwind(value):\n    try:\n        try:\n            return value\n        catch reason:\n            throw reason\n        finally:\n            history.push(value * 10 + 2)\n    catch reason:\n        throw reason\n    finally:\n        history.push(value * 10 + 1)\nassert(unwind(0), 0)\nassert(history, [2, 1])\nassert(unwind(2), 2)\nassert(history, [2, 1, 22, 21])\nassert(unwind(-2), -2)\nassert(history, [2, 1, 22, 21, -18, -19])\n1",
        ),
    ] {
        let cases = common::cases(document);
        assert_eq!(cases.iter().filter(|case| case.title == title).count(), 1);
        let case = cases.get(cases.len() - 3).unwrap();
        assert_eq!(case.title, title);
        assert_eq!(case.expected.as_deref(), Some("1"));
        let protected = common::assertions::protected_assertions(&parse(original).unwrap());
        assert_eq!(protected.len(), 6);
        assert_eq!(
            common::assertions::protected_assertions(&parse(&case.source).unwrap()),
            protected
        );
        for source in [case.source.as_str(), original] {
            assert_eq!(checked(source).assertions_run(), 6);
        }
    }
}

#[test]
fn human_reference_reproduces_the_complete_executable_unwind_case() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Deferred suites\n\n```\n")
        .unwrap();
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == "Nucleoid unwinds deferred suites in reverse order")
        .unwrap();
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(source.trim(), example.trim());
    assert!(reference.contains("[NUC 147](nuc-0147.md)"));
    assert_eq!(case.expected.as_deref(), Some("1"));
}

#[test]
fn new_synthesized_records_are_self_contained_and_execute_their_assertions_and_returns() {
    let records = include_str!("../dataset/nucleoid.synth.147.jsonl");
    assert_eq!(records.lines().count(), 3);
    for (index, line) in records.lines().enumerate() {
        let record: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(record["id"], format!("synth-147-{:04}", index + 1));
        let source = record["code"].as_str().unwrap();
        let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
        let mut runtime = Runtime::new();
        common::check_with_count(
            &mut runtime,
            &common::Case {
                title: record["description"].as_str().unwrap().to_string(),
                source: source.to_string(),
                expected: Some(record["return"].as_str().unwrap().to_string()),
            },
            count,
        )
        .unwrap();
        assert_eq!(runtime.assertions_run(), count);
    }
}

#[test]
fn standing_rules_track_deferred_reads_and_hypotheses_still_undo_cleanup() {
    checked(
        "input = 2\nobserved = 0\nif true:\n    defer:\n        observed = input * 2\n    pass\nassert(observed, 4)\ninput = 3\nassert(observed, 6)\n1",
    );
    checked(
        "state = 1\nsuppose:\n    defer:\n        state = 3\n    state = 2\nassert(state, 1)\n1",
    );
}

#[test]
fn deferred_class_rules_keep_the_existing_non_class_block_guard() {
    for body in [
        "    $Sample.doubled = sample.amount * 2",
        "    defer:\n        $Sample.doubled = sample.amount * 2",
        "    defer:\n        defer:\n            $Sample.doubled = sample.amount * 2",
        "    if true:\n        defer:\n            $Sample.doubled = sample.amount * 2",
    ] {
        let mut runtime = Runtime::new();
        runtime
            .run("class Sample\nsample = Sample()\nsample.amount = 3")
            .unwrap();
        let error = runtime.run(&format!("{{\n{body}\n}}")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{body}");
        assert_eq!(
            error.message(),
            "Cannot define class declaration in non-class block",
            "{body}"
        );
        assert_eq!(runtime.run("sample.amount").unwrap(), Value::Number(3.0));
    }
    checked(
        "class Sample\n{\n    defer:\n        $Sample.doubled = $Sample.amount * 2\n}\nsample = Sample()\nsample.amount = 3\nassert(sample.doubled, 6)\nsample.amount = 4\nassert(sample.doubled, 8)\n1",
    );
}

#[test]
fn native_repl_buffers_defer_and_checks_real_execution_rather_than_process_status_alone() {
    for newline in ["\n", "\r\n"] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .args(["--plain", "--json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let source = "history = []\ndefer:\n    history.push(1)\n\nhistory\n.exit\n";
        child
            .stdin
            .take()
            .unwrap()
            .write_all(source.replace('\n', newline).as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.ends_with("[]\nnull\n[1.0]\n"), "{stdout:?}");
    }
}
