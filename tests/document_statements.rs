//! Exercise the actual build exporter, not a restated statement classifier.

#[path = "../build_statements.rs"]
mod build_statements;

use build_statements::statements;
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
    for keyword in ["else", "elif"] {
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
