use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-is-assignments-{label}-{}.nuc",
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
fn files_read_is_assignments_without_a_flag() {
    let file = SourceFile::new("file");
    let source = "a is good\nb is \"good\"\nc is 'good'\ngood = 3\n[a, b, c]";
    let output = file.run(source, &[]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(text(&output.stdout), "[3.0,\"good\",\"good\"]\n");
    let output = file.run("is = 4\na = 1\na is 5\n[a, is]", &[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(text(&output.stdout), "[5.0,4.0]\n");
    let output = file.run(source, &["--is-assignment"]);
    assert!(!output.status.success(), "{output:?}");
    assert!(
        text(&output.stderr).contains("unexpected argument '--is-assignment'"),
        "{output:?}"
    );
    let help = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success(), "{help:?}");
    assert!(!text(&help.stdout).contains("--is-assignment"), "{help:?}");
    assert!(
        !text(&help.stdout).contains("--c-style-comments"),
        "{help:?}"
    );
    assert!(
        text(&help.stdout).lines().all(|line| !line.ends_with('.')),
        "{help:?}"
    );
}

#[test]
fn is_assignments_combine_with_c_style_comments_and_report_errors_of_equals() {
    let file = SourceFile::new("comments");
    let source = "a is good // a word\nb /* name */ is /* text */ 'x'\nc is 6 / 2\n[a, b, c]";
    let output = file.run(source, &[]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(text(&output.stdout), "[\"good\",\"x\",3.0]\n");
    for (source, expected) in [
        (
            "a is (1 + )",
            "SyntaxError: Unexpected ')' (line 1, column 12)\n",
        ),
        (
            "a is b := 1",
            "SyntaxError: Invalid value assignment target (line 1, column 8)\n",
        ),
        (
            "a is (good)",
            "ReferenceError: good is not defined (line 1, column 1)\n",
        ),
    ] {
        let output = file.run(source, &[]);
        assert!(!output.status.success(), "{output:?}");
        assert_eq!(text(&output.stderr), expected, "{source}");
    }
}

#[test]
fn graph_output_shows_word_assignments_and_their_pending_words() {
    let file = SourceFile::new("graph");
    let output = file.run("a is good\nb is 5\n{ total is good }\na", &["--graph"]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        text(&output.stdout),
        "\"good\"\n\nlogic graph (4 nodes)\n  a [variable]\n    reads    good\n  good [pending]\n    updates  a, block(total is good)\n  b [variable]\n  block(total is good) [block]\n    reads    good\n"
    );
}

#[test]
fn plain_and_redirected_repls_read_is_assignments_and_clear_keeps_them() {
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let options: &[&str] = if plain { &["--plain"] } else { &[] };
            let source = "a is good\na\ngood = 2\na\n.clear\nb is ok\nb\nif true:\n    c is yes\n\nc\n.exit\n";
            let output = repl(&source.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            assert_eq!(
                text(&output.stdout),
                format!(
                    "{BANNER}\"good\"\n\"good\"\n2.0\n2.0\nState cleared.\n\"ok\"\n\"ok\"\n\"yes\"\n\"yes\"\n"
                )
            );
        }
    }
}
