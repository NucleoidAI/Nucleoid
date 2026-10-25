//! Native CLI framing and continuation through the actual executable.

#[allow(dead_code)]
#[path = "support/conditional_headers.rs"]
mod conditional_headers;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use conditional_headers::omit;

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

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

#[test]
fn initial_colon_free_if_buffers_each_alternative_through_blank_commit_and_eof_with_exact_json_bytes()
 {
    for alternative in ["else if", "elif if", "else", "elif"] {
        for newline in ["\n", "\r\n"] {
            for ending in ["", "\n", "\n\n"] {
                for literal in [r#""café😀\0""#, "\"café😀\0\""] {
                    let control = format!("if false:\n\tresult = \"hold\"\n{alternative} true:\n\tresult = {literal}\nelse:\n\tresult = \"wait\"\nresult{ending}").replace('\n', newline);
                    let (actual, sites) = omit(&control);
                    assert_eq!(sites, 2);
                    assert!(actual.contains(&format!("{newline}else:{newline}")));
                    for source in [&actual, &control] {
                        let output = repl(source);
                        assert!(output.status.success(), "{source}\n{output:?}");
                        assert_eq!(output.stderr, b"", "{source}");
                        assert_eq!(
                            output.stdout,
                            format!("{BANNER}\"café😀\\u0000\"\n").as_bytes(),
                            "{source}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn lexical_comments_and_balanced_condition_delimiters_do_not_confuse_buffering() {
    for condition in [
        "true # : [} => are only comment text",
        "({ready: [1, 2][0:1][0]}).ready == 1",
        "((value: int) => { return value })(true)",
        "/[:{}]/.test(\":\")",
        "\"if: else: [}\" == \"if: else: [}\"",
        "`else:${1}:` == \"else:1:\"",
    ] {
        for indent in ["    ", "\t", " \t"] {
            // Colon-bearing controls put the colon before the lexical comment.
            let header = condition.replace(" #", ": #");
            let header = if header == condition {
                format!("{header}:")
            } else {
                header
            };
            let control = format!(
                "if {header}\n{indent}result = 12\nelse: # final:\n{indent}result = 0\nresult"
            );
            let actual = omit(&control).0;
            let output = repl(&format!("{actual}\n\n.help\n.exit\n"));
            assert!(output.status.success(), "{output:?}");
            assert_eq!(output.stderr, b"", "{actual}");
            assert!(
                output
                    .stdout
                    .starts_with(format!("{BANNER}12.0\n").as_bytes()),
                "{output:?}"
            );
            assert!(
                String::from_utf8_lossy(&output.stdout).contains(".clear"),
                "{output:?}"
            );
        }
    }
}

#[test]
fn missing_bodies_malformed_conditions_and_runtime_errors_remain_visible_at_commit_and_eof() {
    for (source, message) in [
        ("if true", "Expected ':' but found end of line"),
        (
            "if true\n    # no body",
            "Expected ':' but found end of line",
        ),
        ("if true\npass", "Expected ':' but found end of line"),
        ("if true +\n    pass", "Unexpected end of line"),
        ("if true\n    value = (1 +)", "Unexpected ')'"),
        (
            "if false\n    pass\nelse (true]\n    pass",
            "Expected ':' but found '('",
        ),
        (
            "if false\n    pass\nelif true;\n    pass",
            "Expected ':' but found keyword 'true'",
        ),
        (
            "if true\n    missing",
            "ReferenceError: missing is not defined",
        ),
        (
            "if true\n    assert(1, 2)",
            "assertion failed: expected 2, got 1",
        ),
    ] {
        for ending in ["", "\n\n.help\n7\n.exit\n"] {
            let output = repl(&format!("{source}{ending}"));
            assert!(output.status.success(), "{output:?}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains(message), "{source}\n{output:?}");
            assert_eq!(stderr.lines().count(), 1, "{source}\n{output:?}");
            if !ending.is_empty() {
                assert!(
                    String::from_utf8_lossy(&output.stdout).contains(".clear"),
                    "{output:?}"
                );
                assert!(output.stdout.ends_with(b"7.0\n"), "{output:?}");
            }
        }
    }
}

#[test]
fn failed_submissions_roll_back_and_do_not_capture_commands_after_blank_line_recovery() {
    let output = repl(
        "saved = 7\nif false # :\n    pass\nelse true;\n    saved = 1\n\nsaved\nif false\n    pass\nelif if true\n    saved = 9\n\nsaved\n.exit\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stderr,
        b"SyntaxError: Expected ':' but found keyword 'true' (line 3, column 6)\n"
    );
    assert_eq!(
        output.stdout,
        format!("{BANNER}7.0\n7.0\n9.0\n9.0\n").as_bytes()
    );
    let output = repl("if true\n.exit\n\n.help\n.exit\n");
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("SyntaxError:"),
        "{output:?}"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr).lines().count(), 1);
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(".clear"),
        "{output:?}"
    );
}

#[test]
fn alternatives_identifiers_literals_and_unrelated_invalid_prefixes_do_not_open_initial_buffers() {
    for line in [
        "\"if ready\"",
        "'if ready'",
        "`if ready`",
        "/if ready/",
        "# if ready",
        "iffy = 7",
        "if_ = 8",
        "value = \"if ready\"",
    ] {
        let output = repl(&format!("{line}\n.help\n.exit\n"));
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stderr, b"", "{line}");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(".clear"),
            "{line}\n{output:?}"
        );
    }
    for line in [
        "else true",
        "elif if true",
        "if true;",
        "if (true]",
        "if true @",
    ] {
        let output = repl(&format!("{line}\n.help\n.exit\n"));
        assert!(output.status.success(), "{output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("SyntaxError:"),
            "{output:?}"
        );
        assert_eq!(String::from_utf8_lossy(&output.stderr).lines().count(), 1);
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(".clear"),
            "{line}\n{output:?}"
        );
    }
}

#[test]
fn files_keep_full_graph_bytes_no_banner_and_physical_eof_diagnostics() {
    let directory = std::env::temp_dir().join(format!(
        "nucleoid-indented-conditionals-{}",
        std::process::id()
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("program.nuc");
    for newline in ["\n", "\r\n"] {
        let control = "flag = false\ninput = 3\nif flag:\n\toutput = 1\nelse if input > 0:\n\toutput = input\nelse:\n\toutput = 0\nassert(output, 3)\ninput = 5\nassert(output, 5)\noutput".replace('\n', newline);
        let actual = omit(&control).0;
        let mut previous = None;
        for source in [&actual, &control] {
            fs::write(&path, source).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
                .args(["--json", "--graph"])
                .arg(&path)
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            assert_eq!(output.stderr, b"");
            assert!(
                output.stdout.starts_with(b"5.0\n\nlogic graph"),
                "{output:?}"
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
            "if true",
            "SyntaxError: Expected ':' but found end of input (line 1, column 8)",
        ),
        (
            "if true\n    # no body",
            "SyntaxError: Expected ':' but found end of line (line 1, column 4)",
        ),
        (
            "if true;\n    pass",
            "SyntaxError: Expected ':' but found ';' (line 1, column 8)",
        ),
        (
            "if true\n    value = (1 +)",
            "SyntaxError: Unexpected ')' (line 2, column 18)",
        ),
        (
            "if false\n    pass\nelse ready { pass }",
            "SyntaxError: Expected ':' but found identifier 'ready' (line 3, column 6)",
        ),
        (
            "if true\n    missing",
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
    ] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg("--json")
            .arg(&path)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{output:?}");
        assert_eq!(output.stdout, b"");
        assert_eq!(output.stderr, format!("{message}\n").as_bytes());
    }
    fs::remove_file(path).unwrap();
    fs::remove_dir(directory).unwrap();
}
