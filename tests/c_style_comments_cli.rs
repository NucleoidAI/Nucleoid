use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

struct SourceFile(PathBuf);

impl SourceFile {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-c-style-comments-{}.nuc",
            std::process::id()
        ));
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        Self(path)
    }

    fn run(&self, source: &str, options: &[&str]) -> Output {
        std::fs::write(&self.0, source).unwrap();
        Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .args(options)
            .arg("--json")
            .arg(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for SourceFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).unwrap();
    }
}

fn repl(source: &str, options: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(options)
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

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap()
}

#[test]
fn file_flag_is_explicit_and_preserves_default_regex_and_failure_contracts() {
    let file = SourceFile::new();
    let output = file.run(
        "# hash\nvalue = 12 /* units */ / 2 // result\nassert(value, 6)\nvalue",
        &["--c-style-comments"],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(text(&output.stdout).trim(), "6.0");
    for (source, expected) in [("//", "\"\""), ("1 // 2 /", "null")] {
        let output = file.run(source, &[]);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert_eq!(text(&output.stdout).trim(), expected);
    }
    for (source, expected) in [
        (
            "// comment",
            "ReferenceError: comment is not defined (line 1, column 4)",
        ),
        ("/* comment */", "SyntaxError: Invalid regular expression:"),
    ] {
        let output = file.run(source, &[]);
        assert!(!output.status.success(), "{output:?}");
        assert!(text(&output.stderr).starts_with(expected), "{output:?}");
        let output = file.run(source, &["--c-style-comments"]);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert_eq!(text(&output.stdout).trim(), "null");
    }
    let output = file.run("value = 1\n\t/* open", &["--c-style-comments"]);
    assert!(!output.status.success(), "{output:?}");
    assert_eq!(
        text(&output.stderr).trim(),
        "SyntaxError: Unterminated block comment (line 2, column 2)"
    );
    let help = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success(), "{help:?}");
    assert!(text(&help.stdout).contains("--c-style-comments"));
}

#[test]
fn plain_and_redirected_repls_honor_comment_headers_multiline_blocks_and_clear() {
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let mut options = vec!["--c-style-comments"];
            if plain {
                options.push("--plain");
            }
            let source = "# hash\nvalue = 12 // comment ending with :\n\
                          /* ignored if true:\n\nignored .exit\n*/\n\
                          if true // header\n\tvalue /= 2\n\tassert(value, 6)\n\n\
                          value\n.clear\nvalue = 8 /* still enabled */\nvalue\n.help\n.exit\n";
            let output = repl(&source.replace('\n', newline), &options);
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            let stdout = text(&output.stdout);
            assert!(stdout.lines().any(|line| line == "6.0"), "{output:?}");
            assert!(stdout.lines().any(|line| line == "8.0"), "{output:?}");
            assert!(stdout.contains("State cleared."), "{output:?}");
            assert!(stdout.contains(".clear"), "{output:?}");
            assert!(!stdout.contains('\u{1b}'), "{output:?}");
        }
    }
}

#[test]
fn comment_aware_continuation_handles_arrows_try_and_closing_a_header_comment() {
    for source in [
        "def half value => // header:\n    value / 2\nhalf(10)",
        "try /* header */\n    throw 5\ncatch issue // alternative\n    issue",
        "if true /* header\nignored } :\n*/\n    5",
        "if true // outer\n    if true /* header\nignored :\n*/\n        5",
    ] {
        for ending in ["", "\n\n.exit\n"] {
            let output = repl(&format!("{source}{ending}"), &["--c-style-comments"]);
            assert!(output.status.success(), "{source}\n{output:?}");
            assert!(output.stderr.is_empty(), "{source}\n{output:?}");
            assert!(
                text(&output.stdout).lines().any(|line| line == "5.0"),
                "{source}\n{output:?}"
            );
        }
    }
}

#[test]
fn unterminated_repl_blocks_wait_through_blank_lines_and_report_once_at_eof() {
    let output = repl(
        "value = 3\n/* open\n\n.exit\n",
        &["--plain", "--c-style-comments"],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        text(&output.stderr).trim(),
        "SyntaxError: Unterminated block comment (line 1, column 1)"
    );
    assert!(text(&output.stdout).lines().any(|line| line == "3.0"));
    let output = repl("// comment\n.help\n.exit\n", &["--plain"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        text(&output.stderr).trim(),
        "ReferenceError: comment is not defined (line 1, column 4)"
    );
    assert!(text(&output.stdout).contains(".clear"), "{output:?}");
}
