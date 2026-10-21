//! Exercise the actual build exporter, not a restated statement classifier.

#[allow(dead_code)]
#[path = "../build.rs"]
mod build_exporter;

mod common;

use build_exporter::build_statements::{assertion, has_grouped_assignment, statements};
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse_program;
use nucleoid::{Runtime, Value};

fn check_export(source: &str, expected_parts: usize, assertions: usize) -> Vec<String> {
    let parts = statements(source);
    assert_eq!(parts.len(), expected_parts, "{source}\n{parts:#?}");
    let whole_ast = parse_program(source).unwrap().statements;
    let split_ast = parts
        .iter()
        .flat_map(|part| parse_program(part).unwrap().statements)
        .collect::<Vec<_>>();
    assert_eq!(whole_ast, split_ast, "{source}");
    assert_eq!(generate_all(&whole_ast), generate_all(&split_ast));

    let mut whole = Runtime::new();
    let whole_value = whole.run(source).unwrap();
    assert_eq!(whole.assertions_run(), assertions, "{source}");
    assert!(whole.take_assertions().is_empty(), "{source}");
    let mut split = Runtime::new();
    let mut split_value = Value::Null;
    for part in &parts {
        split_value = split.run(part).unwrap_or_else(|error| {
            panic!("exported statement:\n{part}\nwhole source:\n{source}\n{error}")
        });
    }
    assert_eq!(split.assertions_run(), assertions, "{source}");
    assert!(split.take_assertions().is_empty(), "{source}");
    assert_eq!(
        whole.serialize_json(&whole_value).unwrap(),
        split.serialize_json(&split_value).unwrap(),
        "{source}"
    );
    parts
}

#[test]
fn both_aliases_keep_spaced_tabbed_and_compact_condition_starts() {
    for keyword in ["else", "elif"] {
        for tail in [
            " true",
            "\ttrue",
            "\u{000c}true",
            "(true)",
            "[1] == [1]",
            "{ready: true}.ready",
            "\"ready\" == \"ready\"",
            "'ready' == 'ready'",
            "`ready` == \"ready\"",
            "/ok/.test(\"ok\")",
            "!false",
        ] {
            for newline in ["\n", "\r\n"] {
                let source = format!(
                    "if false:\n    selected = 0\n{keyword}{tail}:\n    selected = 2\n\
                     else:\n    selected = 3\nassert(selected, 2)\nselected"
                )
                .replace('\n', newline);
                let parts = check_export(&source, 3, 1);
                assert!(parts[0].contains(&format!("\n{keyword}{tail}:")));
            }
        }
    }
}

#[test]
fn balanced_multiline_headers_comments_and_real_indents_stay_attached() {
    for keyword in ["else", "elif"] {
        for condition in [
            "(\n    true # colon : and unmatched-looking [}\n)",
            "[\n    1, 2\n][0:1] == [1]",
            "({\n    ready: {ok: true}\n}).ready.ok",
            "((value: int) => value)(1)",
            "(/[:{}]/).test(\":\")",
        ] {
            for newline in ["\n", "\r\n"] {
                let source = format!(
                    "# leading\nif false:\n\tselected = 0\n\n# between clauses\n\
                     {keyword}{condition}: # suite :\n\n# before body\n\tselected = 2\n\
                     elif false:\n\tselected = 3\nelse # final\n\tselected = 4\n\
                     # next statement\nassert(selected, 2)\nselected"
                )
                .replace('\n', newline);
                check_export(&source, 3, 1);
            }
        }
    }
}

#[test]
fn legacy_explicit_and_final_alias_forms_keep_their_bodies() {
    for explicit in ["else if", "elif if"] {
        for body in [
            " selected = 2",
            " { selected = 2 }",
            " selected = 1; selected = 2",
        ] {
            for final_header in ["else:", "elif:", "else", "elif"] {
                for comment in ["", " # fallback"] {
                    let source = format!(
                        "if false: selected = 0\n{explicit}\t(true):{body}\n\
                         {final_header}{comment}\n    selected = 3\n\
                         assert(selected, 2)\nselected"
                    );
                    check_export(&source, 3, 1);
                }
            }
        }
    }
    for final_header in ["else:", "elif:", "else", "elif", "else#final", "elif#final"] {
        let source = format!(
            "if false:\n    selected = 0\n{final_header}\n    selected = 3\n\
             assert(selected, 3)\nselected"
        );
        check_export(&source, 3, 1);
    }
}

#[test]
fn all_ascii_identifier_continuations_match_the_actual_lexer_boundary() {
    for keyword in ["else", "elif", "catch"] {
        for suffix in b'A'..=b'Z' {
            let name = format!("{keyword}{}", char::from(suffix));
            check_independent_identifier(&name);
        }
        for suffix in b'a'..=b'z' {
            let name = format!("{keyword}{}", char::from(suffix));
            check_independent_identifier(&name);
        }
        for suffix in b'0'..=b'9' {
            let name = format!("{keyword}{}", char::from(suffix));
            check_independent_identifier(&name);
        }
        check_independent_identifier(&format!("{keyword}_"));
    }
    for name in ["elsewhere", "else1", "elif_name", "else_name", "elifwhere"] {
        check_independent_identifier(name);
        let source = format!(
            "def {name}(value): return value\nif false:\n    pass\n{name}(7)\n\
             assert({name}(3), 3)\n8"
        );
        check_export(&source, 5, 1);
    }
}

fn check_independent_identifier(name: &str) {
    assert_eq!(
        tokenize(name).unwrap()[0].token,
        Token::Identifier(name.to_string())
    );
    let source = format!("if false:\n    pass\n{name} = 7\nassert({name}, 7)\n{name}");
    let parts = check_export(&source, 4, 1);
    assert_eq!(parts[0], "if false:\n    pass");
    assert_eq!(parts[1], format!("{name} = 7"));
}

#[test]
fn generated_top_level_class_case_uses_direct_else_and_executes_all_eight_assertions() {
    let record = include_str!("../dataset/nucleoid.spec.jsonl")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|record| {
            record["description"] == "Nucleoid keeps direct conditional class alternatives reactive"
        })
        .unwrap();
    let source = record["code"].as_str().unwrap();
    assert!(source.contains("\nelse $Ticket.priority == \"urgent\":"));
    assert!(source.contains("\nelif $Ticket.state == \"review\":"));
    // Observe the same final value after both execution modes; standalone
    // assertion runs intentionally do not retain a preceding run's return.
    let parts = check_export(&format!("{source}\nsecond.queue"), 22, 8);
    let chain = &parts[7];
    assert!(chain.starts_with("if $Ticket.state"));
    assert!(chain.contains("\nelse $Ticket.priority"));
    assert!(chain.contains("\nelif $Ticket.state"));
    assert!(chain.ends_with("$Ticket.queue = \"normal\""));
}

#[test]
fn compact_class_reference_conditions_retain_independent_reactive_rules() {
    for keyword in ["else", "elif"] {
        let source = format!(
            "class Item\nfirst = Item()\nfirst.active = true\n\
             if $Item.active == false:\n    $Item.choice = 0\n\
             {keyword}$Item.active:\n    $Item.choice = 2\nelse:\n    $Item.choice = 3\n\
             assert(first.choice, 2)\nfirst.active = false\n\
             assert(first.choice, 0)\nfirst.choice"
        );
        check_export(&source, 8, 2);
    }
}

#[test]
fn grouped_invalid_headers_keep_the_whole_programs_parser_error() {
    for keyword in ["else", "elif"] {
        for tail in [
            " ready: pass",
            " ready: { pass }",
            " ready:;\n    pass",
            " ready\n    pass",
            "(true]:\n    pass",
            " true +:\n    pass",
            " true:\npass",
            "{\n    ready: true\n}.ready:\n    pass",
        ] {
            let source = format!("if false: pass\n{keyword}{tail}");
            let whole = parse_program(&source).err().expect("invalid whole header");
            let parts = statements(&source);
            // An unindented body remains a separate statement, as before.
            let split = parse_program(&parts[0])
                .err()
                .expect("invalid split header");
            assert_eq!(whole, split, "{source}");
        }
    }
}

#[test]
fn standalone_alternatives_are_not_made_valid_by_export_grouping() {
    for keyword in ["else", "elif"] {
        let source = format!("{keyword}(true):\n    pass");
        let parts = statements(&source);
        assert_eq!(parts.as_slice(), std::slice::from_ref(&source));
        let error = parse_program(&parts[0])
            .err()
            .expect("standalone alternative");
        assert_eq!(error.message(), "Unexpected keyword 'else'");
    }
}

#[test]
fn nonconditional_continuations_and_comment_ownership_are_unchanged() {
    assert_eq!(
        statements(
            "# leading\ntry:\n    pass\n# catch\ncatch error:\n    pass\n\
             finally:\n    pass\n# following\nafter = 1"
        ),
        [
            "# leading\ntry:\n    pass\n# catch\ncatch error:\n    pass\nfinally:\n    pass",
            "# following\nafter = 1",
        ]
    );
    check_export(
        "items = [\n    1,\n    2\n]\nobject = ({\n    ready: true\n})\n\
         assert(items[0] + items[1], 3)\nassert(object.ready, true)\nitems",
        5,
        2,
    );
}

#[test]
fn actual_splitter_keeps_every_complete_catch_header_with_its_try() {
    for (header, binding) in [
        ("catch", "error"),
        ("catch issue", "issue"),
        ("catch(issue)", "issue"),
        ("catch (issue)", "issue"),
        ("catch\tissue", "issue"),
    ] {
        for try_colon in ["", ":"] {
            for catch_colon in ["", ":"] {
                for newline in ["\n", "\r\n"] {
                    let source = format!(
                        "# leading\ntry{try_colon} # try: in comment\n    throw 7\n\
                         # catch: in comment\n{header}{catch_colon} # catch header\n\
                         \x20   assert({binding}, 7)\n# following\nresult = 12\nresult"
                    )
                    .replace('\n', newline);
                    let parts = check_export(&source, 3, 1);
                    assert!(parts[0].contains(&format!("\n{header}{catch_colon}")));
                    assert!(parts[1].starts_with("# following\n"));
                }
            }
        }
    }
    check_export(
        "try\n    throw 7\ncatch# default\n    assert(error, 7)\n12",
        2,
        1,
    );
}

#[test]
fn actual_renderer_keeps_original_exception_program_checks_and_returns_in_one_run() {
    for (header, binding) in [
        ("catch", "error"),
        ("catch problem", "problem"),
        ("catch(problem)", "problem"),
        ("catch (problem)", "problem"),
    ] {
        for try_colon in ["", ":"] {
            for catch_colon in ["", ":"] {
                let source = format!(
                    "saved = 2\ntry{try_colon}\n    saved = 9\n    throw 7\n\
                     {header}{catch_colon}\n    assert({binding}, 7)\n\
                     assert(saved, 2)\n12"
                );
                let rendered = render_assertion_fixture(&source, Some("12"));
                assert!(
                    rendered.contains(&format!("{source}\n# return: 12")),
                    "{rendered}"
                );
                assert_eq!(rendered.matches("    run(").count(), 1, "{rendered}");
                assert!(!rendered.contains("run_error("), "{rendered}");
                assert!(!rendered.contains("assert_eq!("), "{rendered}");
                common::runner()(&format!("{source}\n# return: 12"));
            }
        }
    }
}

#[test]
fn exception_export_selection_is_not_a_second_header_grammar_or_success_fallback() {
    for (source, expected, reason) in [
        (
            "try;\n    throw 7\ncatch\n    assert(error, 7)",
            None,
            "Expected ':'",
        ),
        (
            "try\n    throw 7\ncatch(problem + 1)\n    assert(problem, 7)",
            None,
            "Expected ')'",
        ),
        (
            "try\n    throw 7\ncatch problem extra\n    assert(problem, 7)",
            None,
            "Expected ':'",
        ),
        (
            "try\n    throw 7\ncatch problem\n    assert(1, 7)",
            None,
            "expected 7, got 1",
        ),
        (
            "try\n    throw 7\ncatch\n    assert(error, 8)",
            None,
            "expected 8, got 7",
        ),
        (
            "try\n    pass\ncatch\n    assert(error, 7)",
            None,
            "only 0 of 1 assertions ran",
        ),
        (
            "try\n    throw 7\ncatch problem\n    assert(problem, 7)\n12",
            Some("13"),
            "returned 12.0, expected 13",
        ),
    ] {
        let rendered = render_assertion_fixture(source, expected);
        assert!(rendered.contains(source), "{rendered}");
        assert!(!rendered.contains("run_error("), "{rendered}");
        assert_eq!(rendered.matches("    run(").count(), 1, "{rendered}");
        let error = common::check(&common::Case {
            title: "invalid original exception record".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        })
        .unwrap_err();
        assert!(error.contains(reason), "{source}\n{error}");
    }
    let source = "# try catch: are comments\ntext = \"try catch:\"\n\
                  pattern = /catch:/\ntemplate = `try:`\ncatcher = 7\nassert(catcher, 7)";
    // Preserve the inherited return distinction: an assertion-only separate
    // run yields null, whereas the whole program retains the assignment's 7.
    let mut whole = Runtime::new();
    assert_eq!(whole.run(source).unwrap(), Value::Number(7.0));
    assert_eq!(whole.assertions_run(), 1);
    assert!(whole.take_assertions().is_empty());
    let mut split = Runtime::new();
    let mut last = Value::Null;
    for part in statements(source) {
        last = split.run(&part).unwrap();
    }
    assert_eq!(last, Value::Null);
    assert_eq!(split.assertions_run(), 1);
    assert!(split.take_assertions().is_empty());
    // Observe the value explicitly when comparing the two execution modes.
    let parts = check_export(&format!("{source}\ncatcher"), 6, 1);
    assert_eq!(parts[3], "catcher = 7");
    let rendered = render_assertion_fixture(source, None);
    assert!(
        rendered.contains("assert_eq!(run(\"catcher\"), 7);"),
        "{rendered}"
    );
}

#[test]
fn shared_actual_exporter_scanner_recognizes_both_assertion_spellings() {
    for (actual, expected) in [
        ("reading", "3"),
        ("[1, 2, {label: \"a,b\"}]", "[1, 2, {label: \"a,b\"}]"),
        ("call(1, [2, 3])", "({ready: true, label: \"x,y\"})"),
        ("`a,b${call(1, 2)}`", "\"a,b1\""),
        ("\"quoted\\\",comma\"", "'quoted,comma'"),
    ] {
        for prefix in ["assert:", "assert :", "assert\t:"] {
            let source = format!(" {prefix} {actual}, {expected} ");
            assert_eq!(assertion(&source), Some((actual, expected)), "{source}");
        }
        let source = format!("assert({actual}, {expected})");
        assert_eq!(assertion(&source), Some((actual, expected)), "{source}");
    }
    for source in [
        "assert: int",
        "assert: Custom",
        "assertive: 1, 1",
        "Assert: 1, 1",
        "record.assert: 1, 1",
        "\"assert: 1, 1\"",
        "# assert: 1, 1",
        "def check(assert: int):",
        "record = {assert: 1}",
    ] {
        assert_eq!(assertion(source), None, "{source}");
    }
}

fn render_assertion_fixture(code: &str, returns: Option<&str>) -> String {
    let records: Vec<build_exporter::Record> = serde_json::from_value(serde_json::json!([{
        "description": "Actual exporter fixture",
        "code": code,
        "return": returns,
    }]))
    .unwrap();
    build_exporter::render_records(&records)
}

#[test]
fn actual_renderer_shares_direct_caught_scoped_and_return_classification() {
    for (candidate, control, returns, fragment) in [
        (
            "reading = 3\nassert: reading, 3",
            "reading = 3\nassert(reading, 3)",
            None,
            "assert_eq!(run(\"reading\"), 3);",
        ),
        (
            "try:\n    missing\ncatch error:\n    assert: error, ReferenceError(\"missing is not defined\")",
            "try:\n    missing\ncatch error:\n    assert(error, ReferenceError(\"missing is not defined\"))",
            None,
            "let mut run = runner();",
        ),
        (
            "if true:\n    reading = 3\n    assert: reading, 3",
            "if true:\n    reading = 3\n    assert(reading, 3)",
            None,
            "assert_eq!(run(",
        ),
        (
            "reading = 7\nreading\nassert: reading, 7",
            "reading = 7\nreading\nassert(reading, 7)",
            Some("7"),
            "# return: 7",
        ),
    ] {
        equivalent_export_ast(candidate, control);
        let new = render_assertion_fixture(candidate, returns);
        let old = render_assertion_fixture(control, returns);
        let source = match returns {
            Some(expected) => format!("{candidate}\n# return: {expected}"),
            None => candidate.to_string(),
        };
        let validation = format!("    runner()({source:?});\n");
        assert!(new.contains(&validation), "{new}");
        // Keep the complete old/new extraction equality check as well as
        // checking the new, unmodified-source validation statement.
        if candidate.starts_with("try:") {
            // Exception records retain both original programs, rather than
            // extracting just a thrown value and a literal expectation.
            assert!(new.contains(candidate), "{new}");
            assert!(old.contains(control), "{old}");
            assert!(!new.contains("run_error("), "{new}");
            assert!(!old.contains("run_error("), "{old}");
            common::runner()(candidate);
            common::runner()(control);
        } else {
            assert_eq!(new.replacen(&validation, "", 1), old, "{candidate}");
        }
        assert!(new.contains(fragment), "{new}");
    }
}

fn equivalent_export_ast(candidate: &str, control: &str) {
    assert_eq!(
        parse_program(candidate).unwrap().statements,
        parse_program(control).unwrap().statements,
    );
}

#[test]
fn deferred_assertions_stay_with_their_invocation_in_the_actual_renderer() {
    let source = "def check(reading):\n    assert: reading, 3\n    return reading\ncheck(3)";
    let rendered = render_assertion_fixture(source, Some("3"));
    assert!(rendered.contains(source), "{rendered}");
    assert!(rendered.contains("# return: 3"), "{rendered}");
    assert_eq!(rendered.matches("    run(").count(), 1, "{rendered}");
    check_export(source, 2, 1);
}

#[test]
fn expression_assignment_export_selection_preserves_operators_and_quoted_text() {
    for source in [
        "pair = Pair(b=2, a=1)",
        "pair = Pair(b=(counter=counter+1), a=counter)",
        "items = [value=1]",
        "record = {amount: (value += 1)}",
        "assert((amount ||= 1), 1)",
        "assert(`value=${amount=1}`, \"value=1\")",
    ] {
        assert!(has_grouped_assignment(source), "{source}");
    }
    for source in [
        "amount = 1",
        "class Pair(a, b):\n    this.a = a",
        "{\n    local = 1\n    global = local\n}",
        "assert(a == b, true)",
        "assert(a === b, true)",
        "assert(a != b, true)",
        "assert(a <= b, true)",
        "assert(a >= b, true)",
        "list.map((item) => item + 1)",
        "assert(\"a=(b=1)\", 'a=(b=1)')",
    ] {
        assert!(!has_grouped_assignment(source), "{source}");
    }
}

#[test]
fn actual_renderer_keeps_named_and_legacy_assignment_calls_in_one_scope() {
    for source in [
        "class Pair(*, a, b):\n    this.a = a\n    this.b = b\ncounter = 0\n\
         pair = Pair(b=(counter=counter+1), a=(counter=counter+1))\n\
         assert(pair.a, 2)\nassert(pair.b, 1)\nassert(counter, 2)",
        "class Legacy(a, b):\n    this.a = a\n    this.b = b\na = 0\nb = 0\n\
         legacy = Legacy(b=2, a=1)\nassert(legacy.a, 2)\nassert(legacy.b, 1)\n\
         assert(a, 1)\nassert(b, 2)",
    ] {
        let rendered = render_assertion_fixture(source, None);
        assert!(rendered.contains(source), "{rendered}");
        assert_eq!(rendered.matches("    run(").count(), 1, "{rendered}");
        assert!(!rendered.contains("assert_eq!(run("), "{rendered}");
        common::runner()(source);
    }
}

#[test]
fn actual_generated_colon_cases_keep_local_and_nested_assertions_executable() {
    let dataset = include_str!("../dataset/nucleoid.spec.jsonl");
    let mut records = 0;
    let mut assertions = 0;
    for line in dataset.lines() {
        let record: serde_json::Value = serde_json::from_str(line).unwrap();
        let title = record["description"].as_str().unwrap();
        if !title.contains("colon-led assertion") {
            continue;
        }
        let source = record["code"].as_str().unwrap();
        let mut runtime = Runtime::new();
        runtime.run(source).unwrap();
        assert!(runtime.take_assertions().is_empty(), "{source}");
        assertions += runtime.assertions_run();
        records += 1;
        let rendered = render_assertion_fixture(source, record["return"].as_str());
        assert!(
            rendered.contains("fn actual_exporter_fixture()"),
            "{rendered}"
        );
        if title.contains("scope and return") {
            assert!(rendered.contains("assert: saved, \"draft\""), "{rendered}");
            // Separate run() calls do not retain the preceding call's
            // result. Observe packet explicitly in both execution modes.
            check_export(&format!("{source}\npacket"), 6, 4);
        }
    }
    assert_eq!(records, 5);
    assert_eq!(assertions, 21);
    let committed = include_str!("nucleoid.spec.rs");
    assert!(committed.contains("fn checks_values_with_colon_led_assertion_statements()"));
    assert!(committed.contains("fn preserves_scope_and_return_flow_around_colon_led_assertions()"));
}

fn assert_original_is_validated(source: &str, returns: Option<&str>) -> String {
    let rendered = render_assertion_fixture(source, returns);
    let checked_source = match returns {
        Some(expected) => format!("{source}\n# return: {expected}"),
        None => source.to_string(),
    };
    assert!(
        rendered.contains(&format!("    runner()({checked_source:?});")),
        "{source:?}\n{rendered}"
    );
    rendered
}

#[test]
fn actual_renderer_validates_the_four_reported_invalid_originals_before_extraction() {
    for (source, message) in [
        ("assert\n    : 1, 1", "Unexpected ':'"),
        ("assert:\n    1, 1", "Unexpected end of line"),
        ("assert: 1,\n    1", "Unexpected end of line"),
        ("assert\u{00a0}: 1, 1", "Unexpected character '\u{00a0}'"),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), nucleoid::ErrorKind::Syntax, "{source:?}");
        assert_eq!(error.message(), message, "{source:?}");
        let rendered = assert_original_is_validated(source, None);
        let validation = rendered.find("    runner()(").unwrap();
        let extracted = rendered.find("    assert_eq!(run(\"1\"), 1);").unwrap();
        assert!(validation < extracted, "{rendered}");
        assert!(
            std::panic::catch_unwind(|| common::runner()(source)).is_err(),
            "{source:?}"
        );
    }
}

#[test]
fn actual_renderer_preserves_supported_horizontal_whitespace_without_normalizing_the_original() {
    for whitespace in ["", " ", "\t", "\u{000c}", "\r", " \t\u{000c}\r"] {
        let source = format!("assert{whitespace}:{whitespace}1{whitespace},{whitespace}1");
        let control = format!("assert{whitespace}({whitespace}1{whitespace},{whitespace}1)");
        equivalent_export_ast(&source, &control);
        Runtime::check(&source).unwrap();
        Runtime::check(&control).unwrap();
        common::runner()(&source);
        common::runner()(&control);
        let rendered = assert_original_is_validated(&source, None);
        assert!(
            rendered.contains("assert_eq!(run(\"1\"), 1);"),
            "{rendered}"
        );
        let legacy = render_assertion_fixture(&control, None);
        assert!(legacy.contains("assert_eq!(run(\"1\"), 1);"), "{legacy}");
    }
    let source = "    assert: 1, 1";
    let control = "    assert(1, 1)";
    equivalent_export_ast(source, control);
    common::runner()(source);
    common::runner()(control);
    assert_original_is_validated(source, None);
}

#[test]
fn actual_renderer_preserves_grouped_multiline_expression_controls_in_both_spellings() {
    for (actual, expected) in [
        ("(\n    1 + 2\n)", "3"),
        ("3", "(\n    1 + 2\n)"),
        ("[\n    1,\n    2\n]", "[1, 2]"),
        ("({\n    ready: true\n})", "{ready: true}"),
        ("(\n    (\n        3\n    )\n)", "(\n    3\n)"),
    ] {
        let source = format!("assert: {actual}, {expected}");
        let control = format!("assert({actual}, {expected})");
        equivalent_export_ast(&source, &control);
        common::runner()(&source);
        common::runner()(&control);
        let rendered = assert_original_is_validated(&source, None);
        assert!(rendered.contains("assert_eq!("), "{rendered}");
        assert!(
            render_assertion_fixture(&control, None).contains("assert_eq!("),
            "{control}"
        );
    }
    let control = "assert(\n    1,\n    1\n)";
    common::runner()(control);
    assert!(render_assertion_fixture(control, None).contains("assert_eq!(run(\"1\"), 1);"));
}

#[test]
fn actual_renderer_validates_malformed_delimiters_layout_unicode_and_scoped_extractions() {
    let mut sources = vec![
        "assert: 1, 1,".to_string(),
        "assert: 1, 1, 2".to_string(),
        "assert: , 1".to_string(),
        "assert: 1,, 1".to_string(),
        "assert: (1], 1".to_string(),
        "assert: 1;, 1".to_string(),
        "if true:\n    assert\u{00a0}: 1, 1".to_string(),
        "try:\n    missing\ncatch error:\n    assert\u{00a0}: 1, 1".to_string(),
    ];
    for newline in ["\n", "\r\n"] {
        for (before, after) in [
            ("assert", ": 1, 1"),
            ("assert:", "1, 1"),
            ("assert: 1", ", 1"),
            ("assert: 1,", "1"),
        ] {
            sources.push(format!("{before}{newline}    {after}"));
        }
    }
    for whitespace in [
        '\u{000b}', '\u{0085}', '\u{00a0}', '\u{2003}', '\u{2028}', '\u{2029}', '\u{3000}',
        '\u{feff}',
    ] {
        for (before, after) in [
            ("", "assert: 1, 1"),
            ("assert", ": 1, 1"),
            ("assert:", "1, 1"),
            ("assert: 1", ", 1"),
            ("assert: 1,", "1"),
            ("assert: 1, 1", ""),
        ] {
            sources.push(format!("{before}{whitespace}{after}"));
        }
    }
    for source in sources {
        assert_eq!(
            Runtime::check(&source)
                .expect_err(&format!("malformed source {source:?}"))
                .kind(),
            nucleoid::ErrorKind::Syntax,
            "{source:?}"
        );
        let rendered = render_assertion_fixture(&source, None);
        // A recognized extraction must validate the complete raw record.
        // Otherwise the ordinary fallback must still execute invalid source.
        if rendered.contains("assert_eq!(") {
            assert_original_is_validated(&source, None);
        }
        assert!(
            std::panic::catch_unwind(|| common::runner()(&source)).is_err(),
            "{source:?}\n{rendered}"
        );
    }
}

#[test]
fn actual_renderer_keeps_legacy_declarations_and_assert_name_contexts() {
    use nucleoid::lang::ast::Stmt;

    for source in ["assert: int", "assert: int\n", "assert: int;"] {
        assert_eq!(
            parse_program(source).unwrap().statements,
            [Stmt::Declaration {
                name: "assert".to_string(),
                type_name: "int".to_string(),
            }]
        );
        let error = Runtime::new().run(source).unwrap_err();
        assert_eq!(error.kind(), nucleoid::ErrorKind::Reference);
        assert_eq!(error.message(), "Missing definition");
        let rendered = render_assertion_fixture(source, None);
        assert!(!rendered.contains("assert_eq!("), "{rendered}");
        assert!(!rendered.contains("    runner()("), "{rendered}");
        assert!(rendered.contains("run(\"assert: int"), "{rendered}");
    }
    let source = "assert = 12\ndef identity(assert: int) => assert\n\
                  record = {assert: identity(4)}\nassert: record.assert, 4";
    common::runner()(source);
    assert_original_is_validated(source, None);
    assert!(Runtime::check("{ assert: int }").is_err());
}

#[test]
fn actual_renderer_source_validation_keeps_real_failures_coverage_and_expected_returns() {
    for (source, failure) in [
        ("assert: 1, 2", "1 assertion(s): expected 2, got 1"),
        ("if false:\n    assert: 1, 1", "only 0 of 1 assertions ran"),
    ] {
        assert_original_is_validated(source, None);
        let error = common::check(&common::Case {
            title: "actual renderer source validation".to_string(),
            source: source.to_string(),
            expected: None,
        })
        .unwrap_err();
        assert!(error.contains(failure), "{error}");
    }
    let source = "7\nassert: 1, 1";
    assert_original_is_validated(source, Some("7"));
    let wrong_return = format!("{source}\n# return: 8");
    assert!(std::panic::catch_unwind(|| common::runner()(&wrong_return)).is_err());
}
