use nucleoid::lang::ast::lexer::Keyword;
use nucleoid::{Runtime, SyntaxOptions, Value};
use rustyline::{Cmd, EventHandler, KeyCode, KeyEvent, Modifiers};

use super::{
    Pending, indented_alternative_header, indented_body_header, indented_condition_header,
    indented_named_arrow_header, indented_try_header, next_indentation,
};
use crate::INDENT;
use crate::interactive::{CONTINUATION_PROMPT, repl_editor, setting_value};

#[test]
fn continuation_prompt_has_no_extra_separator_before_body_indentation() {
    let indentation = next_indentation("class Student(name, age):", true);
    assert_eq!(format!("{CONTINUATION_PROMPT}{indentation}"), "..    ");
}

#[test]
fn tab_inserts_the_same_four_literal_spaces_as_body_indentation() {
    let mut editor = repl_editor().unwrap();
    let Some(EventHandler::Simple(Cmd::Insert(1, spaces))) =
        editor.unbind_sequence(KeyEvent(KeyCode::Tab, Modifiers::NONE))
    else {
        panic!("Tab must insert literal spaces");
    };
    assert_eq!(spaces, INDENT);
    assert_eq!(spaces, "    ");
}

#[test]
fn class_body_keystrokes_use_the_editors_visible_indentation() {
    let mut source = String::new();
    let mut indentation = String::new();
    for input in [
        "class Student(name, age):",
        "this.name = name",
        "this.age = age",
    ] {
        let line = format!("{indentation}{input}");
        source.push_str(&line);
        source.push('\n');
        indentation = next_indentation(&line, indented_body_header(&line));
    }
    assert_eq!(
        source,
        "class Student(name, age):\n    this.name = name\n    this.age = age\n"
    );
    assert_eq!(indentation, "    ");
    let mut runtime = Runtime::new();
    runtime.run(&source).unwrap();
    runtime
        .run("student = Student(age=20, name=\"Can\")")
        .unwrap();
    assert_eq!(
        runtime.run("student.name").unwrap(),
        Value::String("Can".into())
    );
    assert_eq!(runtime.run("student.age").unwrap(), Value::Number(20.0));
}

#[test]
fn indentation_tracks_nested_bodies_tabs_and_explicit_dedents() {
    for (line, expected) in [
        ("class Student(name, age):", "    "),
        ("    this.name = name", "    "),
        ("    if age > 18:", "        "),
        ("    if age > 18 # colon: in comment", "        "),
        ("\telif if ready # comment", "\t    "),
        ("        this.adult = true", "        "),
        ("    this.age = age", "    "),
        ("\tif ready:", "\t    "),
        ("\t\tvalue = 1", "\t\t"),
        ("    def identity value => # body", "        "),
        ("    try # body", "        "),
        ("    # a comment ending in a colon:", "    "),
        ("value = \"class Student:\"", ""),
    ] {
        assert_eq!(
            next_indentation(line, indented_body_header(line)),
            expected,
            "{line:?}"
        );
    }
}

#[test]
fn lexical_block_headers_include_comments_and_parent_classes() {
    for line in [
        "class Student(name, age): # properties",
        "class Student: Human",
        "class Student(name, age): Human # parent",
        "def compute(value): # body",
        "if ready: # body",
        "else: # body",
        "try: # body",
        "catch issue: # body",
    ] {
        assert!(indented_body_header(line), "{line:?}");
    }
    for line in [
        "# class Student:",
        "\"class Student:\"",
        "message = \"if ready:\"",
        "class Student(name, age)",
        "class Student\n    this.name = name\n    this.age = age",
        "catch",
        "else",
    ] {
        assert!(!indented_body_header(line), "{line:?}");
    }
}

fn submitted(lines: &[&str], options: SyntaxOptions) -> Vec<Option<String>> {
    let mut pending = Pending::default();
    lines
        .iter()
        .map(|line| pending.accept(line, options))
        .collect()
}

#[test]
fn named_function_headers_buffer_until_their_braces_close() {
    let options = SyntaxOptions::default();
    let source = "function scale(amount) {\n\n    factor = 2 # }\n    return amount * factor\n}\n";
    let lines: Vec<&str> = source.lines().collect();
    let mut expected = vec![None; lines.len() - 1];
    expected.push(Some(source.to_string()));
    assert_eq!(submitted(&lines, options), expected);

    let mut pending = Pending::default();
    assert_eq!(
        pending.accept("    function scale(amount) {", options),
        None
    );
    assert_eq!(pending.indentation, "    ");
    assert_eq!(pending.accept("        return amount", options), None);
    assert_eq!(pending.indentation, "        ");

    let mut runtime = Runtime::new();
    runtime.run(source).unwrap();
    assert_eq!(runtime.run("scale(4)").unwrap(), Value::Number(8.0));

    for line in [
        "function scale(amount) { return amount * 2 }",
        "function scale(amount) {} {}",
        "scale = function (amount) {",
        "{",
        "record.function scale {",
        "label = \"function scale(amount) {\"",
        "# function scale(amount) {",
        "function (amount) {",
    ] {
        assert_eq!(
            submitted(&[line], options),
            [Some(format!("{line}\n"))],
            "{line:?}"
        );
    }
}

#[test]
fn named_function_bodies_buffer_through_nested_function_literals_and_declarations() {
    let lines = [
        "function outer(amount) {",
        "    helper = function (x) { return x }",
        "    function inner(y) { return y }",
        "    return helper(amount) + inner(amount)",
        "}",
    ];
    let mut expected = vec![None; lines.len() - 1];
    expected.push(Some(format!("{}\n", lines.join("\n"))));
    assert_eq!(submitted(&lines, SyntaxOptions::default()), expected);
}

#[test]
fn named_function_bodies_inside_colon_blocks_wait_for_both_endings() {
    let options = SyntaxOptions::default();
    let lines = [
        "if ready:",
        "    function scale(amount) {",
        "",
        "        return amount",
        "    }",
        "",
    ];
    let mut expected = vec![None; lines.len() - 1];
    expected.push(Some(
        "if ready:\n    function scale(amount) {\n\n        return amount\n    }\n".to_string(),
    ));
    assert_eq!(submitted(&lines, options), expected);

    let lines = [
        "function grade(score) {",
        "    if score > 50:",
        "        return \"pass\"",
        "    return \"fail\"",
        "}",
        "",
    ];
    let mut expected = vec![None; lines.len() - 1];
    expected.push(Some(format!("{}\n", lines[..5].join("\n"))));
    assert_eq!(submitted(&lines, options), expected);
}

#[test]
fn named_function_buffering_ends_at_lexical_errors_and_follows_comment_options() {
    let options = SyntaxOptions::default();
    assert_eq!(
        submitted(&["function scale(amount) {", "    amount ~ 2"], options),
        [
            None,
            Some("function scale(amount) {\n    amount ~ 2\n".to_string())
        ]
    );
    let enabled = SyntaxOptions {
        c_style_comments: true,
        is_assignment: false,
        conditional_words: false,
    };
    assert_eq!(
        submitted(
            &[
                "function scale(amount) { /* {",
                "",
                "*/ return amount // }",
                "}"
            ],
            enabled
        ),
        [
            None,
            None,
            None,
            Some("function scale(amount) { /* {\n\n*/ return amount // }\n}\n".to_string())
        ]
    );
}

#[test]
fn extends_class_headers_buffer_only_through_their_colon() {
    for line in [
        "class Dog extends Animal:",
        "class Dog(name, breed) extends Animal:",
        "class Dog extends Animal: # body",
    ] {
        assert!(indented_body_header(line), "{line:?}");
        assert_eq!(
            next_indentation(line, indented_body_header(line)),
            INDENT,
            "{line:?}"
        );
    }
    for line in [
        "class Dog extends Animal",
        "class Dog(name) extends Animal # bodyless",
        "class Dog extends Animal: this.sound = \"woof\"",
        "extends = 1",
    ] {
        assert!(!indented_body_header(line), "{line:?}");
        assert_eq!(
            next_indentation(line, indented_body_header(line)),
            "",
            "{line:?}"
        );
    }
}

#[test]
fn bare_alternatives_indent_only_their_existing_continuation_body() {
    for line in [
        "else",
        "else # body",
        "else ready",
        "elif ready # body",
        "else if ready",
        "elif if ready",
        "catch",
        "catch issue",
        "catch(issue)",
        "catch (issue) # body",
    ] {
        assert!(indented_alternative_header(line), "{line:?}");
        assert_eq!(next_indentation(line, false), "    ", "{line:?}");
    }
    for line in [
        "# catch",
        "\"else\"",
        "else ready;",
        "catch issue extra",
        "catch()",
        "catch(issue, other)",
        "catch(issue) extra",
    ] {
        assert!(!indented_alternative_header(line), "{line:?}");
    }
}

#[test]
fn colon_free_condition_continuation_uses_balanced_lexical_headers_not_suffixes() {
    for condition in [
        "ready",
        "ready@first",
        "items@0[1:3] == [2, 3]",
        "not ready and available",
        "items[1:3] == [2, 3]",
        "check({ready: items[:]}, (x: int) => x)",
        "(x: int) => { return x }",
        "({ready: true}).ready",
        "(# unmatched-looking :[}\n ready)",
        "\"else if :\" == \"else if :\"",
        "/[:{}]/.test(\":\")",
        "`else if ${value}:`",
        "ready # : [} => are only a comment",
    ] {
        for prefix in ["if", "else if", "elif if", "else", "elif"] {
            let line = format!("\t{prefix} {condition}");
            let keyword = if prefix == "if" {
                Keyword::If
            } else {
                Keyword::Else
            };
            assert!(indented_condition_header(&line, keyword), "{line:?}");
            let last_indent = line
                .lines()
                .next_back()
                .unwrap()
                .chars()
                .take_while(|character| matches!(character, ' ' | '\t'))
                .collect::<String>();
            assert_eq!(
                next_indentation(&line, indented_body_header(&line)),
                format!("{last_indent}{INDENT}")
            );
            assert_eq!(indented_body_header(&line), prefix == "if", "{line:?}");
        }
    }
    for line in [
        "",
        "# if ready",
        "\"if ready\"",
        "/if ready/",
        "`if ready`",
        "iffy = 1",
        "value = \"if ready\"",
        "if",
        "if # condition missing",
        "if ready:",
        "if ready: pass",
        "if ready;\n    pass",
        "if ready\n    pass",
        "if (ready",
        "if ready)",
        "if [ready)",
        "if {ready: true]",
        "if ready @",
        "if ready@unknown",
        "if ready@-1",
        "if \"unterminated",
        "if ready\u{a0}",
    ] {
        assert!(!indented_condition_header(line, Keyword::If), "{line:?}");
    }
    // Qualification never pretends that these are valid expressions.
    // Real parsing must still report them on blank-line commit or EOF.
    assert!(indented_condition_header("if ready +", Keyword::If));
    assert!(!indented_body_header("else ready"));
    assert!(!indented_body_header("elif if ready"));
}

#[test]
fn conditional_expression_colons_pair_with_their_question_marks_in_headers() {
    for condition in [
        "ready ? on : off",
        "ready ? (x: int) => x : off",
        "a ? b : c ? d : e",
        "a ? b ? c : d : e",
        "(ready ? on : off) == on",
        "on if ready else off",
        "ready ? `${a ? b : c}` : off",
    ] {
        for prefix in ["if", "else if", "elif if", "else", "elif"] {
            let line = format!("{prefix} {condition}");
            let keyword = if prefix == "if" {
                Keyword::If
            } else {
                Keyword::Else
            };
            assert!(indented_condition_header(&line, keyword), "{line:?}");
            assert_eq!(indented_body_header(&line), prefix == "if", "{line:?}");
        }
    }
    for line in [
        "if ready ?",
        "if ready ? on",
        "if ready : off",
        "if ready ? on : off:",
        "if ready ? on : off: pass",
        "if ready ? on : off; other",
    ] {
        assert!(!indented_condition_header(line, Keyword::If), "{line:?}");
    }
    // A colon-bearing header still buffers through its final colon.
    assert!(indented_body_header("if ready ? on : off:"));
    assert!(!indented_body_header("label = ready ? on : off"));
    assert!(!indented_body_header("label = on if ready else off"));
}

#[test]
fn quantifier_colons_pair_with_their_prefix_in_headers() {
    for condition in [
        "all v in xs: v > 0",
        "any v in xs: v ? on : off",
        "(all v in xs: v) and ready",
        "all t in Task: any w in Worker: w.skill == t.skill",
        "ready ? all v in xs: v : false",
        "all v in ready ? xs : ys: v",
        "all v in xs: [v][0:1]",
        "all v in xs: v ?? false",
    ] {
        for prefix in ["if", "else if", "elif if", "else", "elif"] {
            let line = format!("{prefix} {condition}");
            let keyword = if prefix == "if" {
                Keyword::If
            } else {
                Keyword::Else
            };
            assert!(indented_condition_header(&line, keyword), "{line:?}");
            assert_eq!(indented_body_header(&line), prefix == "if", "{line:?}");
        }
    }
    for line in [
        "if all v in xs",
        "if all v in xs: v > 0:",
        "if all v in xs: v > 0: pass",
        "if all v in xs: v; other",
        "if all v of xs: v",
        "if all: v",
        "if all in xs: v",
    ] {
        assert!(!indented_condition_header(line, Keyword::If), "{line:?}");
    }
    assert!(indented_body_header("if all v in xs: v > 0:"));
    assert!(!indented_body_header("ok = all v in xs: v > 0"));
    assert!(indented_alternative_header("else any v in xs: v > 1"));
    assert_eq!(next_indentation("elif all v in xs: v", false), INDENT);
}

#[test]
fn while_headers_buffer_through_their_suite_colon() {
    for line in [
        "while count < 3:",
        "while not done: # loop",
        "while ready ? on : off:",
        "while all v in xs: v > 0:",
    ] {
        assert!(indented_body_header(line), "{line:?}");
        assert_eq!(next_indentation(line, true), INDENT, "{line:?}");
    }
    for line in [
        "while count < 3: count = count + 1",
        "while ready: tick",
        "while count < 3",
        "while = 3",
    ] {
        assert!(!indented_body_header(line), "{line:?}");
    }
    assert_eq!(
        next_indentation("    while count < 3:", true),
        format!("    {INDENT}")
    );
}

#[test]
fn nullish_coalescing_pairs_with_no_colon_in_headers() {
    for condition in [
        "ready ?? false",
        "a ?? b ?? c",
        "(ready ?? false) == true",
        "ready ? on ?? off : off",
        "ready ?? false ? on : off",
        "on if ready ?? false else off",
        "`${ready ?? \"off\"}` == \"on\"",
        "items[index ?? 0]",
    ] {
        for prefix in ["if", "else if", "elif if", "else", "elif"] {
            let line = format!("{prefix} {condition}");
            let keyword = if prefix == "if" {
                Keyword::If
            } else {
                Keyword::Else
            };
            assert!(indented_condition_header(&line, keyword), "{line:?}");
            assert_eq!(indented_body_header(&line), prefix == "if", "{line:?}");
        }
    }
    for line in [
        "if ready ? ? off",
        "if ready ?? on : off",
        "if ready ?? false:",
        "if ready ?? false: pass",
        "if ready ?? false; other",
    ] {
        assert!(!indented_condition_header(line, Keyword::If), "{line:?}");
    }
    assert!(indented_body_header("if ready ?? false:"));
    assert!(!indented_body_header("label = ready ?? \"off\""));
}

#[test]
fn exponentiation_headers_buffer_exactly_like_other_operators() {
    for line in [
        "if side ** 2 > limit",
        "if (side ** 2) > limit # body",
        "if side * side > limit",
    ] {
        assert!(indented_body_header(line), "{line:?}");
        assert!(indented_condition_header(line, Keyword::If), "{line:?}");
    }
    assert!(indented_body_header("if side ** 2 > limit:"));
    assert!(!indented_condition_header(
        "if side ** 2 > limit:",
        Keyword::If
    ));
    for line in [
        "area = side ** 2",
        "area **= 2",
        "area = side ** 2 if ready else 0",
        "pattern = /x**y/",
        "label = \"2**8\"",
    ] {
        assert!(!indented_body_header(line), "{line:?}");
    }
    let mut runtime = Runtime::new();
    runtime.run("side = 3\nlimit = 8").unwrap();
    runtime
        .run("if side ** 2 > limit\n    big = side ** 2\n")
        .unwrap();
    assert_eq!(runtime.run("big").unwrap(), Value::Number(9.0));
}

#[test]
fn value_assignments_run_at_once_and_end_colon_free_header_scans() {
    for line in [
        "stock := stock - qty",
        "stock :=",
        "if ready := 1",
        "if x := y ? 1 : 2",
        "else total := 0",
        "elif total := 0",
        "if ready: stock := stock - qty",
    ] {
        assert!(!indented_body_header(line), "{line:?}");
        assert!(!indented_condition_header(line, Keyword::If), "{line:?}");
        assert!(!indented_alternative_header(line), "{line:?}");
        assert_eq!(next_indentation(line, false), "", "{line:?}");
    }
    for line in ["if ready:", "if ready", "if f(x := 1)"] {
        assert!(indented_body_header(line), "{line:?}");
    }
    let mut runtime = Runtime::new();
    runtime.run("stock = 10\nqty = 3").unwrap();
    assert_eq!(
        runtime.run("stock := stock - qty").unwrap(),
        Value::Number(7.0)
    );
    runtime.run("qty = 5").unwrap();
    assert_eq!(runtime.run("stock").unwrap(), Value::Number(7.0));
}

#[test]
fn suppose_headers_buffer_through_their_final_colon() {
    for line in [
        "suppose reading = 95:",
        "peak = suppose reading = 95:",
        "suppose:",
        "peak = suppose:",
        "suppose ok = all v in xs: v > 0:",
        "suppose x = ready ? 1 : 2: # body",
    ] {
        assert!(indented_body_header(line), "{line:?}");
        assert_eq!(next_indentation(line, true), INDENT, "{line:?}");
    }
    for line in [
        "suppose reading = 95: alarm",
        "peak = suppose reading = 95: alarm",
        "suppose reading = 95",
        "suppose = 1",
        "x = suppose + 1",
    ] {
        assert!(!indented_body_header(line), "{line:?}");
    }
    let mut runtime = Runtime::new();
    runtime.run("reading = 40\nalarm = reading > 90").unwrap();
    assert_eq!(
        runtime
            .run("peak = suppose reading = 95:\n    alarm\n")
            .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        runtime.run("suppose reading = 95: alarm").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(runtime.run("alarm").unwrap(), Value::Bool(false));
}

#[test]
fn multiline_input_keeps_the_final_physical_lines_indentation() {
    let source = "class Student(name, age):\n    this.name = name\n    this.age = age\n";
    assert!(!indented_body_header(source));
    assert_eq!(next_indentation(source, false), "    ");
    assert_eq!(
        next_indentation("    if ready:\n        value = 1", false),
        "        "
    );
}

#[test]
fn only_a_complete_lexical_bare_try_header_starts_exception_continuation() {
    for line in [
        "try",
        "try ",
        "\ttry\t",
        "    try # catch: and => are only a comment",
        "try\t# try",
        "try \u{c}\r# accepted layout",
    ] {
        assert!(indented_try_header(line), "{line:?}");
    }
}

#[test]
fn exception_continuation_rejects_text_prefixes_extra_tokens_and_lexical_errors() {
    for line in [
        "",
        " \t",
        "# try",
        "\"try\"",
        "'try'",
        "`try`",
        "/try/",
        "tryout",
        "try_",
        "trying",
        "value = \"try\"",
        "value = try",
        "try = 1",
        "try:",
        "try;",
        "try pass",
        "try {}",
        "try()",
        "try error",
        "try =>",
        "catch",
        "catch error",
        "catch(error)",
        "def f() =>",
        "try @",
        "try \"unterminated",
        "try\u{a0}",
        "try\u{2003}",
        "try\u{b}",
    ] {
        assert!(!indented_try_header(line), "{line:?}");
    }
}

#[test]
fn only_named_headers_with_a_bodyless_arrow_start_continuation() {
    for line in [
        "def f() =>",
        "def f(value: int,) => # body",
        "    def f value =>",
        "\tdef f left: int, right, factor: number => # =>",
    ] {
        assert!(indented_named_arrow_header(line), "{line}");
    }
    for line in [
        "value =>",
        "(value: int) =>",
        "callback = value =>",
        "def f value => value",
        "def f value => next =>",
        "def f value => \"=>\"",
        "def f value => /=>/",
        "def f value => `=>`",
        "def f value => value # =>",
        "\"def f value =>\"",
        "# def f value =>",
        "1 # =>",
        "def f value => @",
    ] {
        assert!(!indented_named_arrow_header(line), "{line}");
    }
}

#[test]
fn thin_named_arrow_headers_buffer_exactly_like_fat_ones() {
    for line in [
        "def f() ->",
        "def f(value: int,) -> # body",
        "    def f value ->",
        "\tdef f left: int, right, factor: number -> # ->",
        "def f left right ->",
    ] {
        assert!(indented_named_arrow_header(line), "{line}");
        assert!(indented_body_header(line), "{line}");
        assert!(
            indented_named_arrow_header(&line.replace("->", "=>")),
            "{line}"
        );
    }
    for line in [
        "value ->",
        "(value: int) ->",
        "callback = value ->",
        "def f value -> value",
        "def f value -> next ->",
        "def f value => next ->",
        "def f value -> next =>",
        "def f value -> \"->\"",
        "def f value -> /->/",
        "def f value -> `->`",
        "def f value -> value # ->",
        "\"def f value ->\"",
        "# def f value ->",
        "def f value - >",
        "def f value -> @",
        "f = lambda:",
    ] {
        assert!(!indented_named_arrow_header(line), "{line}");
    }
}

#[test]
fn repl_settings_toggle_or_accept_explicit_values() {
    assert_eq!(setting_value("", false), Some(true));
    assert_eq!(setting_value("", true), Some(false));
    assert_eq!(setting_value("on", false), Some(true));
    assert_eq!(setting_value("off", true), Some(false));
    assert_eq!(setting_value("invalid", false), None);
}
