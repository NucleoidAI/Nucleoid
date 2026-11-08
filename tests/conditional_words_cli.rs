use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

const WORDS: [&str; 2] = ["--is-assignment", "--conditional-words"];

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-conditional-words-{label}-{}.nuc",
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
fn the_flag_reads_word_branches_in_files_only_beside_is_assignments() {
    let file = SourceFile::new("file");
    let source = "ready = true\nstatus is OPEN if ready else WAITING\nlevel is ready ? HIGH : LOW\nOPEN = 1\n[status, level]";
    for options in [&WORDS[..], &["--conditional-words", "--is-assignment"][..]] {
        let output = file.run(source, options);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert_eq!(text(&output.stdout), "[1.0,\"HIGH\"]\n");
    }
    let output = file.run(source, &["--is-assignment"]);
    assert!(!output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        text(&output.stderr),
        "ReferenceError: OPEN is not defined (line 2, column 1)\n"
    );
    for options in [
        &[][..],
        &["--conditional-words"][..],
        &["--c-style-comments", "--conditional-words"][..],
    ] {
        let output = file.run(source, options);
        assert!(!output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert_eq!(
            text(&output.stderr),
            "ReferenceError: status is not defined (line 2, column 1)\n"
        );
    }
    let help = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success(), "{help:?}");
    let help = text(&help.stdout);
    assert!(help.contains("--conditional-words"), "{help}");
    assert!(help.contains("--is-assignment"), "{help}");
    assert!(help.lines().all(|line| !line.ends_with('.')), "{help}");
}

#[test]
fn the_flag_combines_with_c_style_comments_and_reports_the_errors_of_is_assignments() {
    let file = SourceFile::new("comments");
    let source = "ready = false // flag\nstatus is OPEN /* yes */ if ready else WAITING // no\nlevel is 6 / 2 if ready else 0\n[status, level]";
    for options in [
        &[
            "--is-assignment",
            "--conditional-words",
            "--c-style-comments",
        ][..],
        &[
            "--c-style-comments",
            "--conditional-words",
            "--is-assignment",
        ][..],
    ] {
        let output = file.run(source, options);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert_eq!(text(&output.stdout), "[\"WAITING\",0.0]\n");
    }
    for (source, expected) in [
        (
            "a is A if c else",
            "SyntaxError: Unexpected end of input (line 1, column 17)\n",
        ),
        (
            "a is c ? A",
            "SyntaxError: Expected ':' but found end of input (line 1, column 11)\n",
        ),
        (
            "c = true\na is (A) if c else B",
            "ReferenceError: A is not defined (line 2, column 1)\n",
        ),
        (
            "c = true\na is X if c else Y = 5",
            "SyntaxError: Cannot assign to (c?X:Y) (line 2, column 1)\n",
        ),
    ] {
        for options in [&WORDS[..], &["--is-assignment"][..]] {
            let output = file.run(source, options);
            assert!(!output.status.success(), "{output:?}");
            assert_eq!(text(&output.stderr), expected, "{source}");
        }
    }
}

#[test]
fn graph_output_shows_word_branches_and_their_pending_words() {
    let file = SourceFile::new("graph");
    let mut options = WORDS.to_vec();
    options.push("--graph");
    let output = file.run(
        "c = true\na is A if c else B\n{ total is A if c else (B) }\na",
        &options,
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        text(&output.stdout),
        "\"A\"\n\nlogic graph (4 nodes)\n  c [variable]\n    updates  a, block(total is c?A:(B))\n  a [variable]\n    reads    c, A\n  A [pending]\n    updates  a, block(total is c?A:(B))\n  block(total is c?A:(B)) [block]\n    reads    c, A\n"
    );
}

#[test]
fn plain_and_redirected_repls_honor_the_flag_and_clear_keeps_it() {
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let mut options = WORDS.to_vec();
            if plain {
                options.push("--plain");
            }
            let source = "ready = false\nstatus is OPEN if ready else WAITING\nready = true\nstatus\n.clear\nb is YES if true else NO\nb\nif true:\n    c is ON if true else OFF\n\nc\n.exit\n";
            let output = repl(&source.replace('\n', newline), &options);
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            assert_eq!(
                text(&output.stdout),
                format!(
                    "{BANNER}false\n\"WAITING\"\ntrue\n\"OPEN\"\nState cleared.\n\"YES\"\n\"YES\"\n\"ON\"\n\"ON\"\n"
                )
            );
        }
    }
    let output = repl(
        "b is YES if true else NO\nb\n",
        &["--plain", "--is-assignment"],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(text(&output.stdout), BANNER);
    assert_eq!(
        text(&output.stderr),
        "ReferenceError: YES is not defined (line 1, column 1)\nReferenceError: b is not defined (line 1, column 1)\n"
    );
}
