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
fn the_flag_reads_is_assignments_in_files_and_the_default_keeps_juxtaposed_statements() {
    let file = SourceFile::new("file");
    let source = "a is good\nb is \"good\"\nc is 'good'\ngood = 3\n[a, b, c]";
    let output = file.run(source, &["--is-assignment"]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(text(&output.stdout), "[3.0,\"good\",\"good\"]\n");
    for options in [&[][..], &["--c-style-comments"][..]] {
        let output = file.run(source, options);
        assert!(!output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert_eq!(
            text(&output.stderr),
            "ReferenceError: a is not defined (line 1, column 1)\n"
        );
    }
    let output = file.run("is = 4\na = 1\na is 5", &[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(text(&output.stdout), "5.0\n");
    let output = file.run("is = 4\na = 1\na is 5\n[a, is]", &["--is-assignment"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(text(&output.stdout), "[5.0,4.0]\n");
    let help = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success(), "{help:?}");
    assert!(text(&help.stdout).contains("--is-assignment"), "{help:?}");
    assert!(
        text(&help.stdout).contains("--c-style-comments"),
        "{help:?}"
    );
    assert!(
        text(&help.stdout).lines().all(|line| !line.ends_with('.')),
        "{help:?}"
    );
}

#[test]
fn the_flag_combines_with_c_style_comments_and_reports_errors_of_equals() {
    let file = SourceFile::new("comments");
    let source = "a is good // a word\nb /* name */ is /* text */ 'x'\nc is 6 / 2\n[a, b, c]";
    for options in [
        &["--is-assignment", "--c-style-comments"][..],
        &["--c-style-comments", "--is-assignment"][..],
    ] {
        let output = file.run(source, options);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert_eq!(text(&output.stdout), "[\"good\",\"x\",3.0]\n");
    }
    let output = file.run(source, &["--c-style-comments"]);
    assert!(!output.status.success(), "{output:?}");
    assert_eq!(
        text(&output.stderr),
        "ReferenceError: a is not defined (line 1, column 1)\n"
    );
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
        let output = file.run(source, &["--is-assignment"]);
        assert!(!output.status.success(), "{output:?}");
        assert_eq!(text(&output.stderr), expected, "{source}");
    }
}

#[test]
fn graph_output_shows_word_assignments_and_their_pending_words() {
    let file = SourceFile::new("graph");
    let output = file.run(
        "a is good\nb is 5\n{ total is good }\na",
        &["--is-assignment", "--graph"],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        text(&output.stdout),
        "\"good\"\n\nlogic graph (4 nodes)\n  a [variable]\n    reads    good\n  good [pending]\n    updates  a, block(total is good)\n  b [variable]\n  block(total is good) [block]\n    reads    good\n"
    );
}

#[test]
fn plain_and_redirected_repls_honor_the_flag_and_clear_keeps_it() {
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let mut options = vec!["--is-assignment"];
            if plain {
                options.push("--plain");
            }
            let source = "a is good\na\ngood = 2\na\n.clear\nb is ok\nb\nif true:\n    c is yes\n\nc\n.exit\n";
            let output = repl(&source.replace('\n', newline), &options);
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
    let output = repl("a is good\na\n", &["--plain"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(text(&output.stdout), BANNER);
    assert_eq!(
        text(&output.stderr),
        "ReferenceError: a is not defined (line 1, column 1)\nReferenceError: a is not defined (line 1, column 1)\n"
    );
}
