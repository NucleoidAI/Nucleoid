//! Optional exception-suite colons reuse the accepted AST and transactional path.
//! These ordinary regressions need neither Git history nor session artifacts.

mod common;

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Stmt, parse, parse_program};
use nucleoid::lang::evaluation::Flow;
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::value::ObjectId;
use nucleoid::{ErrorKind, Position, Runtime, Value};

/// Fixture-only lexical edit: remove only qualified exception-header colons.
/// Literal/comment colons, annotations and inline suites are never rewritten.
fn omit(control: &str, omit_try: bool, omit_catch: bool) -> String {
    let tokens = tokenize(control).unwrap();
    let mut edits = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let mut next = index + 1;
        match token.token {
            Token::Keyword(Keyword::Try) if omit_try => {}
            Token::Keyword(Keyword::Catch) if omit_catch => match tokens[next].token {
                Token::Identifier(_) => next += 1,
                Token::LeftParen => next += 3,
                _ => {}
            },
            _ => continue,
        }
        if tokens.get(next).map(|token| &token.token) != Some(&Token::Colon) {
            continue;
        }
        let layout = &tokens[next + 1..];
        if !matches!(layout[0].token, Token::Newline | Token::Indent)
            || !layout
                .iter()
                .find(|token| !matches!(token.token, Token::Newline | Token::Semicolon))
                .is_some_and(|token| token.token == Token::Indent)
        {
            continue;
        }
        let position = tokens[next].position;
        let mut offset = 0;
        for (line, text) in control.split_inclusive('\n').enumerate() {
            if line + 1 == position.line {
                offset += text.char_indices().nth(position.column - 1).unwrap().0;
                break;
            }
            offset += text.len();
        }
        edits.push(offset);
    }
    let mut result = control.to_string();
    for offset in edits.into_iter().rev() {
        assert_eq!(result.remove(offset), ':');
    }
    result
}

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
    let mut values = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([actual, control]) {
        let value = common::check_with_count(
            runtime,
            &common::Case {
                title: "paired exception suites".to_string(),
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
    assert!(runtimes[0].graph.keys().eq(runtimes[1].graph.keys()));
    for node in runtimes[0].graph.nodes() {
        let other = runtimes[1].graph.retrieve(&node.key).unwrap();
        assert_eq!(node.kind, other.kind);
        assert_eq!(node.dependencies, other.dependencies);
        assert_eq!(node.dependents, other.dependents);
        assert!(
            runtimes[0]
                .graph
                .shape_dependencies(&node.key)
                .eq(runtimes[1].graph.shape_dependencies(&node.key))
        );
    }
    runtimes
}

#[test]
fn complete_existing_catch_headers_and_each_independent_omission_share_the_full_try_ast() {
    for (header, parameter) in [
        ("catch", "error"),
        ("catch problem", "problem"),
        ("catch(problem)", "problem"),
        ("catch (problem)", "problem"),
        ("catch\tproblem", "problem"),
        ("catch\t(problem)", "problem"),
    ] {
        let control = format!("try:\n    pass\n{header}:\n    pass");
        for (omit_try, omit_catch) in [(false, false), (true, false), (false, true), (true, true)] {
            let actual = omit(&control, omit_try, omit_catch);
            let ast = equivalent(&actual, &control);
            assert_eq!(
                ast,
                [Stmt::Try {
                    body: vec![Stmt::Pass],
                    parameter: parameter.to_string(),
                    catch: vec![Stmt::Pass],
                }]
            );
            assert_eq!(
                generate_all(&ast),
                "try",
                "the existing generator is unchanged"
            );
            pair(&actual, &control, 0, Some("null"));
        }
        let control = format!("try:\n    throw 7\n{header}:\n    assert({parameter}, 7)");
        for (omit_try, omit_catch) in [(true, false), (false, true), (true, true)] {
            pair(&omit(&control, omit_try, omit_catch), &control, 1, None);
        }
    }
}

#[test]
fn real_indentation_comments_blank_lines_crlf_and_tabs_keep_the_accepted_layout() {
    for newline in ["\n", "\r\n"] {
        for body in [
            "\n    pass",
            "\n\tpass",
            " # header comment\n\n# before body\n    # body comment\n    pass",
            "\n;\n    pass",
            "\n    ;\n    pass;",
        ] {
            for suffix in ["", "\n", "\n# EOF", "\nafter = 3", "\n; after = 3"] {
                let control =
                    format!("try:{body}\ncatch (issue):{body}{suffix}").replace('\n', newline);
                let actual = omit(&control, true, true);
                assert_ne!(actual, control);
                pair(&actual, &control, 0, None);
            }
        }
    }
    let source = "try\n    pass\ncatch\n    pass\nafter = 3";
    assert_eq!(
        parse_program(source).unwrap().positions,
        [Position::new(1, 1), Position::new(5, 1)]
    );
}

#[test]
fn legacy_colons_keep_inline_semicolon_and_braced_precedence_with_mixed_suites() {
    for control in [
        "try: throw 7\ncatch issue: assert(issue, 7)",
        "try: { throw 7 }\ncatch(issue): { assert(issue, 7) }",
        "try: pass; throw 7\ncatch: assert(error, 7); pass",
        "try: throw 7\ncatch issue:\n    assert(issue, 7)",
        "try:\n    throw 7\ncatch(issue): assert(issue, 7)",
        "try:\n    { throw 7 }\ncatch:\n    { assert(error, 7) }",
    ] {
        pair(&omit(control, true, true), control, 1, None);
    }
}

#[test]
fn only_header_colons_change_not_literal_comment_annotation_or_object_colons() {
    let control = "text = \"try:\ncatch:\n    literal\"\n\
                   pattern = /try:catch:/\ntemplate = `catch:`\n\
                   record = {label: \"catch:\"}\n\
                   try: # try: catch:\n    throw record.label\n\
                   catch(problem): # catch:\n    assert(problem, \"catch:\")";
    let actual = omit(control, true, true);
    assert_eq!(
        actual,
        "text = \"try:\ncatch:\n    literal\"\n\
         pattern = /try:catch:/\ntemplate = `catch:`\n\
         record = {label: \"catch:\"}\n\
         try # try: catch:\n    throw record.label\n\
         catch(problem) # catch:\n    assert(problem, \"catch:\")"
    );
    pair(&actual, control, 1, None);
}

#[test]
fn nested_catches_keep_complete_association_and_actual_thrown_values() {
    for control in [
        "state = \"ready\"\ntry:\n    try:\n        throw \"INNER\"\n\
         \x20   catch inner:\n        throw inner + \"-OUTER\"\n\
         catch outer:\n    assert(outer, \"INNER-OUTER\")\nassert(state, \"ready\")",
        "state = 1\ntry:\n    try:\n        missing\n\
         \x20   catch:\n        throw error\ncatch (issue):\n\
         \x20   assert(issue, ReferenceError(\"missing is not defined\"))\nassert(state, 1)",
    ] {
        for (omit_try, omit_catch) in [(true, false), (false, true), (true, true)] {
            pair(&omit(control, omit_try, omit_catch), control, 2, None);
        }
    }
}

#[test]
fn savepoints_restore_live_dependencies_snapshots_and_reasoning_reads() {
    let control = "source = 2\nderived = source * 3\nstartup = derived.value\nmessage = \"\"\n\
                   try:\n    source = 9\n    throw \"ABORT\"\n\
                   catch failure:\n    message = failure\n    assert(failure, \"ABORT\")\n\
                   assert(source, 2)\nassert(derived, 6)\nassert(message, \"ABORT\")\n\
                   source = 4\nassert(derived, 12)\nassert(startup, 6)\nderived";
    let mut runtimes = pair(&omit(control, true, true), control, 6, Some("12"));
    let mut explanations = Vec::new();
    for runtime in &mut runtimes {
        let explanation = runtime.run("why derived").unwrap();
        explanations.push(runtime.serialize_json(&explanation).unwrap());
        assert_eq!(
            runtime.run("(why derived)[1].node").unwrap(),
            Value::String("source".into())
        );
        assert_eq!(
            runtime.run("(why derived)[1].holds").unwrap(),
            Value::Number(4.0)
        );
    }
    assert_eq!(explanations[0], explanations[1]);
}

#[test]
fn class_rules_rollback_and_other_instances_remain_independent() {
    let control = "class Parcel(state: str):\n    this.state = state\n\
                   first = Parcel(\"queued\")\nsecond = Parcel(\"waiting\")\n\
                   $Parcel.label = $Parcel.state + \"!\"\n\
                   try:\n    first.state = \"broken\"\n    throw \"ABORT\"\n\
                   catch reason:\n    assert(reason, \"ABORT\")\n\
                   assert(first.state, \"queued\")\nassert(first.label, \"queued!\")\n\
                   assert(second.label, \"waiting!\")\nfirst.state = \"sent\"\n\
                   assert(first.label, \"sent!\")\nassert(second.label, \"waiting!\")\nfirst.label";
    pair(&omit(control, true, true), control, 6, Some("\"sent!\""));
}

#[test]
fn function_returns_bare_returns_and_non_capture_keep_existing_meanings() {
    let control = "value = 3\ndef recover(flag):\n    try:\n        if flag:\n\
                   \x20           throw \"FALLBACK\"\n        return value\n\
                   \x20   catch issue:\n        return issue\n\
                   def invoke(value):\n    return recover(false)\n\
                   first = invoke(99)\nsecond = recover(true)\n\
                   assert(first, 3)\nassert(second, \"FALLBACK\")";
    pair(&omit(control, true, true), control, 2, None);
    for returned in ["return", "pass"] {
        let control = format!(
            "def recover():\n    try:\n        throw \"STOP\"\n\
             \x20   catch:\n        {returned}\nassert(recover(), null)"
        );
        pair(&omit(&control, true, true), &control, 1, None);
    }
}

#[test]
fn direct_try_nodes_preserve_return_flow_and_restore_caller_scope_and_receivers() {
    for control in [
        "try:\n    return value + 1\ncatch problem:\n    return 0",
        "try:\n    throw \"STOP\"\ncatch problem:\n    return value + 1",
    ] {
        let actual = omit(control, true, true);
        equivalent(&actual, control);
        for source in [actual.as_str(), control] {
            let mut runtime = Runtime::new();
            let mut scope = Scope::new();
            scope.push();
            scope.declare("value", Value::Number(99.0));
            scope.declare("error", Value::String("outer".to_string()));
            let receiver = ObjectId::named("caller");
            scope.set_this(Some(receiver.clone()));
            scope.set_instance(Some(receiver.clone()));
            let mut node =
                Nuc::convert(&mut runtime, &mut scope, &parse(source).unwrap()[0]).unwrap();
            assert!(matches!(
                node.run(&mut runtime, &mut scope).unwrap().flow,
                Flow::Return(Value::Number(100.0))
            ));
            assert_eq!(scope.depth(), 2);
            assert_eq!(scope.retrieve("value"), Some(&Value::Number(99.0)));
            assert_eq!(scope.retrieve("problem"), None);
            assert_eq!(
                scope.retrieve("error"),
                Some(&Value::String("outer".to_string()))
            );
            assert_eq!(scope.this(), Some(&receiver));
            assert_eq!(scope.instance(), Some(&receiver));
        }
    }
}

fn syntax(source: &str, message: &str, position: Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source:?}");
    assert_eq!(error.message(), message, "{source:?}");
    assert_eq!(error.position(), Some(position), "{source:?}");
}

#[test]
fn nonqualifying_headers_keep_exact_existing_errors_and_positions() {
    for (source, message, position) in [
        (
            "try",
            "Expected ':' but found end of input",
            Position::new(1, 4),
        ),
        (
            "try\n",
            "Expected ':' but found end of line",
            Position::new(1, 1),
        ),
        (
            "try\n# comment\n    # no body",
            "Expected ':' but found end of line",
            Position::new(1, 1),
        ),
        (
            "try\npass\ncatch: pass",
            "Expected ':' but found end of line",
            Position::new(1, 1),
        ),
        (
            "try;\n    pass\ncatch: pass",
            "Expected ':' but found ';'",
            Position::new(1, 4),
        ),
        (
            "try pass\ncatch: pass",
            "Expected ':' but found keyword 'pass'",
            Position::new(1, 5),
        ),
        (
            "try { pass }\ncatch: pass",
            "Expected ':' but found '{'",
            Position::new(1, 5),
        ),
        (
            "try: pass\ncatch",
            "Expected ':' but found end of input",
            Position::new(2, 6),
        ),
        (
            "try: pass\ncatch\npass",
            "Expected ':' but found end of line",
            Position::new(2, 1),
        ),
        (
            "try: pass\ncatch issue;\n    pass",
            "Expected ':' but found ';'",
            Position::new(2, 12),
        ),
        (
            "try: pass\ncatch issue pass",
            "Expected ':' but found keyword 'pass'",
            Position::new(2, 13),
        ),
        (
            "try: pass\ncatch(issue) { pass }",
            "Expected ':' but found '{'",
            Position::new(2, 14),
        ),
        (
            "try: pass\ncatch (issue, other): pass",
            "Expected ')' but found ','",
            Position::new(2, 13),
        ),
        (
            "try: pass\ncatch issue extra:\n    pass",
            "Expected ':' but found identifier 'extra'",
            Position::new(2, 13),
        ),
        (
            "try:\npass\ncatch: pass",
            "Expected an indented block but found keyword 'pass'",
            Position::new(2, 1),
        ),
        (
            "try: pass\ncatch:\npass",
            "Expected an indented block but found keyword 'pass'",
            Position::new(3, 1),
        ),
    ] {
        syntax(source, message, position);
    }
    for tail in [
        "catcher:\n    pass",
        "catch_name:\n    pass",
        "finally:\n    pass",
        "",
    ] {
        let source = format!("try:\n    pass\n{tail}");
        let actual = omit(&source, true, true);
        let old_error = Runtime::check(&source).unwrap_err();
        let new_error = Runtime::check(&actual).unwrap_err();
        assert_eq!(old_error, new_error, "{source:?}");
        assert_eq!(old_error.message(), "Expected 'catch' after 'try'");
    }
}

#[test]
fn lexical_and_body_failures_keep_physical_positions_after_either_qualified_header() {
    for (body, message, try_column, catch_column) in [
        ("value = (1 +)", "Unexpected ')'", 17, 18),
        ("value = @", "Unexpected character '@'", 13, 13),
    ] {
        for (actual, control, column) in [
            (
                format!("try\n    {body}\ncatch: pass"),
                format!("try:\n    {body}\ncatch: pass"),
                try_column,
            ),
            (
                format!("try: pass\ncatch (issue)\n    {body}"),
                format!("try: pass\ncatch (issue):\n    {body}"),
                catch_column,
            ),
        ] {
            let old_error = Runtime::check(&control).unwrap_err();
            assert_eq!(old_error, Runtime::check(&actual).unwrap_err());
            assert_eq!(old_error.message(), message);
            assert_eq!(old_error.position().unwrap().column, column);
        }
    }
    for whitespace in ['\u{000b}', '\u{00a0}', '\u{2003}', '\u{2028}'] {
        for source in [
            format!("try{whitespace}\n    pass\ncatch:\n    pass"),
            format!("try: pass\ncatch{whitespace}\n    pass"),
        ] {
            let error = Runtime::check(&source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(
                error.message(),
                format!("Unexpected character '{whitespace}'")
            );
        }
    }
}

#[test]
fn shared_statement_operator_and_postfix_budgets_and_parser_progress_are_unchanged() {
    let mut control = "pass".to_string();
    let mut success = 0;
    let mut limited = 0;
    for _ in 0..68 {
        let indented = control
            .lines()
            .map(|line| format!("    {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        control = format!("try:\n{indented}\ncatch:\n    pass");
        let actual = omit(&control, true, true);
        match (parse(&actual), parse(&control)) {
            (Ok(actual), Ok(control)) => {
                assert_eq!(actual, control);
                success += 1;
            }
            (Err(actual), Err(control)) => {
                assert_eq!(actual, control);
                assert_eq!(actual.message(), "Statements are nested too deeply");
                limited += 1;
            }
            _ => panic!("different statement budget"),
        }
    }
    assert!(success > 0 && limited > 0);
    for count in 120..=132 {
        for operand in [
            format!("1{}", " + 1".repeat(count)),
            format!("record{}", ".field".repeat(count)),
            format!("{}1", "-".repeat(count)),
        ] {
            for control in [
                format!("try:\n    {operand}\ncatch:\n    pass"),
                format!("try: pass\ncatch:\n    {operand}"),
            ] {
                let actual = omit(&control, true, true);
                match (parse(&actual), parse(&control)) {
                    (Ok(actual), Ok(control)) => assert_eq!(actual, control),
                    (Err(actual), Err(control)) => assert_eq!(actual, control),
                    _ => panic!("different expression budget for {operand}"),
                }
            }
        }
    }
    syntax(
        "try\n    pass",
        "Expected 'catch' after 'try'",
        Position::new(2, 9),
    );
    let control = "try:\n    pass\ncatch:\n    pass\nafter = 3\nassert(after, 3)";
    pair(&omit(control, true, true), control, 1, None);
}

#[test]
fn wrong_assertions_untaken_catches_and_wrong_returns_never_become_successes() {
    for (control, expected, reason) in [
        (
            "try:\n    throw 7\ncatch issue:\n    assert(1, 7)",
            None,
            "expected 7, got 1",
        ),
        (
            "try:\n    throw 7\ncatch:\n    assert(error, 8)",
            None,
            "expected 8, got 7",
        ),
        (
            "try:\n    pass\ncatch issue:\n    assert(issue, 7)",
            None,
            "only 0 of 1 assertions ran",
        ),
        (
            "try:\n    throw 7\ncatch:\n    assert(error, 7)\n12",
            Some("13"),
            "returned 12.0, expected 13",
        ),
    ] {
        let actual = omit(control, true, true);
        equivalent(&actual, control);
        for source in [actual.as_str(), control] {
            let error = common::check(&common::Case {
                title: "negative exception coverage".to_string(),
                source: source.to_string(),
                expected: expected.map(str::to_string),
            })
            .unwrap_err();
            assert!(error.contains(reason), "{source}\n{error}");
        }
    }
}

#[test]
fn new_training_shard_executes_every_assertion_and_explicit_return() {
    let records = include_str!("../dataset/nucleoid.synth.27.jsonl")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 4);
    for ((index, record), count) in records.iter().enumerate().zip([4, 4, 5, 5]) {
        assert_eq!(record["id"], format!("synth-27-{:04}", index + 1));
        let source = record["code"].as_str().unwrap();
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
        assert_eq!(
            common::assertions::protected_assertions(&parse(source).unwrap()).len(),
            count
        );
        assert_eq!(runtime.assertions_run(), count);
    }
}
