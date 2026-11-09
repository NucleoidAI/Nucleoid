use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-at-variables-{label}-{}.nuc",
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
fn files_read_at_variables_without_a_flag_and_report_reference_errors() {
    let file = SourceFile::new("file");
    let output = file.run("a is 1\nb is @a + 2\nb", &[]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(text(&output.stdout), "3.0\n");
    for (source, expected) in [
        (
            "b is @c + 3",
            "ReferenceError: c is not defined (line 1, column 1)\n",
        ),
        (
            "colors is [@RED, GREEN]",
            "ReferenceError: RED is not defined (line 1, column 1)\n",
        ),
        (
            "b = @a",
            "SyntaxError: Unexpected character '@' (line 1, column 5)\n",
        ),
        (
            "x is @y = 1",
            "SyntaxError: Expected a non-negative integer, 'first', 'last', or 'previous' after '@' (line 1, column 7)\n",
        ),
    ] {
        let output = file.run(source, &[]);
        assert!(!output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert_eq!(text(&output.stderr), expected, "{source}");
    }
    let output = file.run("b is 1", &["--at-variables"]);
    assert!(!output.status.success(), "{output:?}");
    assert!(
        text(&output.stderr).contains("unexpected argument '--at-variables'"),
        "{output:?}"
    );
}

#[test]
fn plain_and_redirected_repls_buffer_colon_free_headers_with_at_variables() {
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let options: &[&str] = if plain { &["--plain"] } else { &[] };
            let source = "x = 2\nC = 2\nif x is @C\n    y = 1\n\ny\nif x is not @C\n    z = 1\nelse\n    z = 3\n\nz\nb is @x + 1\nb\n.clear\nc is [@D, E]\n.exit\n";
            let output = repl(&source.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            assert_eq!(
                text(&output.stderr),
                "ReferenceError: D is not defined (line 1, column 1)\n"
            );
            assert_eq!(
                text(&output.stdout),
                format!("{BANNER}2.0\n2.0\n1.0\n1.0\n3.0\n3.0\n3.0\n3.0\nState cleared.\n")
            );
        }
    }
}

#[test]
fn graph_output_shows_at_variables_as_references() {
    let file = SourceFile::new("graph");
    let output = file.run(
        "a = 1\nxs is [@a, A]\n{ total is {k: @a, j: B} }\nxs",
        &["--graph"],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        text(&output.stdout),
        "[1.0,\"A\"]\n\nlogic graph (5 nodes)\n  a [variable]\n    updates  xs, block(total is {\"k\":(a),\"j\":B})\n  xs [variable]\n    reads    a, A\n  A [pending]\n    updates  xs\n  block(total is {\"k\":(a),\"j\":B}) [block]\n    reads    a, B\n  B [pending]\n    updates  block(total is {\"k\":(a),\"j\":B})\n"
    );
}
