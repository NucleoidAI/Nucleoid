use std::io::Write as _;
use std::process::{Command, Output, Stdio};

fn repl(source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
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

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).unwrap()
}

fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).unwrap()
}

fn recovery(try_header: &str, catch_header: &str, binding: &str) -> String {
    format!(
        "result = 0\n{try_header}\n    throw \"RECOVERED\"\n{catch_header}\n    assert({binding}, \"RECOVERED\")\n    result = {binding}\nresult"
    )
}

#[test]
fn root_try_buffers_all_bindings_and_independent_colons_until_blank_or_eof() {
    for try_header in ["try", "try:"] {
        for (catch_header, binding) in [
            ("catch", "error"),
            ("catch issue", "issue"),
            ("catch(issue)", "issue"),
            ("catch (issue)", "issue"),
        ] {
            for colon in ["", ":"] {
                let source = recovery(try_header, &format!("{catch_header}{colon}"), binding);
                for ending in ["", "\n", "\n\n.exit\n"] {
                    let output = repl(&format!("{source}{ending}"));
                    assert!(output.status.success(), "{source}\n{output:?}");
                    assert!(output.stderr.is_empty(), "{source}\n{output:?}");
                    assert_eq!(
                        stdout(&output)
                            .lines()
                            .filter(|line| *line == "\"RECOVERED\"")
                            .count(),
                        1,
                        "{source}\n{output:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn root_try_comments_tabs_and_accepted_spaces_keep_literal_source_layout() {
    for indent in ["    ", "\t", " \t"] {
        for newline in ["\n", "\r\n"] {
            let source = recovery(
                "try \t\u{c}# header: not punctuation",
                "catch (issue) # catch:",
                "issue",
            )
            .replace("    ", indent)
            .replace('\n', newline);
            for ending in ["", "\n\n.exit\n"] {
                let output = repl(&format!("{source}{ending}"));
                assert!(output.status.success(), "{output:?}");
                assert!(output.stderr.is_empty(), "{output:?}");
                assert!(
                    stdout(&output).lines().any(|line| line == "\"RECOVERED\""),
                    "{output:?}"
                );
            }
        }
    }
}

#[test]
fn committed_body_binding_parse_and_assertion_errors_remain_visible_and_recover() {
    for (source, diagnostic) in [
        ("try", "Expected ':' but found end of line"),
        ("try\n    pass", "Expected 'catch' after 'try'"),
        ("try\n    pass\ncatch", "Expected ':' but found end of line"),
        (
            "try\n    pass\ncatch issue extra\n    pass",
            "Expected ':' but found identifier 'extra'",
        ),
        (
            "try\n    value = @\ncatch\n    pass",
            "Unexpected character '@'",
        ),
        (
            "try\n    throw \"body\"\ncatch\n    missing",
            "ReferenceError: missing is not defined",
        ),
        (
            "try\n    throw \"body\"\ncatch\n    assert(1, 2)",
            "assertion failed: expected 2, got 1",
        ),
    ] {
        let output = repl(&format!(
            "{source}\n\n.help\n{}\n\n.exit\n",
            recovery("try", "catch", "error")
        ));
        // The existing REPL exit is not a program-success signal.
        assert!(output.status.success(), "{output:?}");
        assert!(stderr(&output).contains(diagnostic), "{source}\n{output:?}");
        assert_eq!(stderr(&output).lines().count(), 1, "{output:?}");
        assert!(stdout(&output).contains(".clear"), "{output:?}");
        assert!(
            stdout(&output).lines().any(|line| line == "\"RECOVERED\""),
            "{output:?}"
        );
    }
}

#[test]
fn missing_catch_and_missing_body_fail_at_eof_instead_of_silent_success() {
    for (source, diagnostic) in [
        ("try", "Expected ':'"),
        ("try\n    pass", "Expected 'catch' after 'try'"),
        ("try\n    pass\ncatch", "Expected ':'"),
        ("try\ncatch\n    pass", "Expected ':'"),
    ] {
        let output = repl(source);
        assert!(output.status.success(), "{output:?}");
        assert!(stderr(&output).contains(diagnostic), "{output:?}");
        assert_eq!(stderr(&output).lines().count(), 1, "{output:?}");
    }
}

#[test]
fn literal_comment_and_identifier_text_do_not_hold_the_next_command_in_a_buffer() {
    for line in [
        "\"try\"",
        "'try'",
        "`try`",
        "/try/",
        "# try",
        "tryout = 7",
        "try_ = 8",
        "trying = 9",
        "value = \"try\"",
    ] {
        let output = repl(&format!("{line}\n.help\n.exit\n"));
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{line}\n{output:?}");
        assert!(stdout(&output).contains(".clear"), "{line}\n{output:?}");
    }
}

#[test]
fn invalid_try_prefixes_and_standalone_catch_fail_immediately_not_as_headers() {
    for line in [
        "try extra",
        "try;",
        "try()",
        "try {}",
        "try = 1",
        "try @",
        "try\u{a0}",
        "catch",
        "catch error",
        "catch(error)",
    ] {
        let output = repl(&format!("{line}\n.help\n.exit\n"));
        assert!(output.status.success(), "{output:?}");
        assert!(
            stderr(&output).contains("SyntaxError:"),
            "{line}\n{output:?}"
        );
        assert_eq!(stderr(&output).lines().count(), 1, "{line}\n{output:?}");
        assert!(stdout(&output).contains(".clear"), "{line}\n{output:?}");
    }
}

#[test]
fn accepted_colon_arrow_and_command_interaction_is_preserved() {
    let output = repl(
        ".help\n.json off\n.json on\n.graph on\n.graph off\nsaved = 7\ntry:\n    throw saved\ncatch issue:\n    assert(issue, 7)\n\n\
         def identity value => # existing header\n    value\n\nidentity(8)\n.clear\n.quit\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    for text in [
        ".clear",
        "json off.",
        "json on.",
        "graph on.",
        "graph off.",
        "State cleared.",
    ] {
        assert!(stdout(&output).contains(text), "{output:?}");
    }
    assert!(
        stdout(&output).lines().any(|line| line == "8.0"),
        "{output:?}"
    );
}

#[test]
fn commands_inside_a_pending_try_remain_source_until_commit() {
    let output = repl("try\n.exit\n\n.help\n.exit\n");
    assert!(output.status.success(), "{output:?}");
    assert!(stderr(&output).contains("SyntaxError:"), "{output:?}");
    assert_eq!(stderr(&output).lines().count(), 1, "{output:?}");
    assert!(stdout(&output).contains(".clear"), "{output:?}");
}
