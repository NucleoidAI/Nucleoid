use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-item-words-{label}-{}.nuc",
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
fn files_read_item_words_without_a_flag() {
    let file = SourceFile::new("file");
    let source = "colors is [RED, GREEN]\npair is {left: RED, right: BLUE}\nfirst is pick(color=RED)\nRED = 1\n[colors, pair, first]";
    let output = file.run(&format!("def pick(color) => color\n{source}"), &[]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        text(&output.stdout),
        "[[1.0,\"GREEN\"],{\"left\":1.0,\"right\":\"BLUE\"},1.0]\n"
    );
    let output = file.run(&source.replace(" is ", " = "), &[]);
    assert!(!output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        text(&output.stderr),
        "ReferenceError: RED is not defined (line 1, column 1)\n"
    );
    let output = file.run(source, &["--item-words"]);
    assert!(!output.status.success(), "{output:?}");
    assert!(
        text(&output.stderr).contains("unexpected argument '--item-words'"),
        "{output:?}"
    );
    let help = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success(), "{help:?}");
    let help = text(&help.stdout);
    assert!(!help.contains("--item-words"), "{help}");
    assert!(help.lines().all(|line| !line.ends_with('.')), "{help}");
}

#[test]
fn item_words_combine_with_comments_and_conditional_words_and_report_the_errors_of_is_assignments()
{
    let file = SourceFile::new("combined");
    let source = "ready = false // flag\nlevel is [HIGH if ready else LOW, /* second */ MID]\nx is [6 / 2]\n[level, x]";
    let output = file.run(source, &[]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(text(&output.stdout), "[[\"LOW\",\"MID\"],[3.0]]\n");
    for (source, expected) in [
        (
            "a is [A, ",
            "SyntaxError: Unexpected end of input (line 1, column 10)\n",
        ),
        (
            "a is {k: A",
            "SyntaxError: Expected '}' but found end of input (line 1, column 11)\n",
        ),
        (
            "a is Light(color=A",
            "SyntaxError: Expected ')' but found end of input (line 1, column 19)\n",
        ),
        (
            "c = true\na is [(A)]",
            "ReferenceError: A is not defined (line 2, column 1)\n",
        ),
        (
            "o = {m: (v) => v}\nr is o.m(k=A)",
            "ReferenceError: A is not defined (line 2, column 1)\n",
        ),
    ] {
        let output = file.run(source, &[]);
        assert!(!output.status.success(), "{output:?}");
        assert_eq!(text(&output.stderr), expected, "{source}");
    }
}

#[test]
fn graph_output_shows_item_words_and_their_pending_words() {
    let file = SourceFile::new("graph");
    let output = file.run(
        "c = true\nxs is [A, (c)]\n{ total is {k: A, j: (c)} }\nxs",
        &["--graph"],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        text(&output.stdout),
        "[\"A\",true]\n\nlogic graph (4 nodes)\n  c [variable]\n    updates  xs, block(total is {\"k\":A,\"j\":(c)})\n  xs [variable]\n    reads    A, c\n  A [pending]\n    updates  xs, block(total is {\"k\":A,\"j\":(c)})\n  block(total is {\"k\":A,\"j\":(c)}) [block]\n    reads    A, c\n"
    );
}

#[test]
fn plain_and_redirected_repls_read_item_words_and_clear_keeps_them() {
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let options: &[&str] = if plain { &["--plain"] } else { &[] };
            let source = "xs is [A, B]\nxs\nA = 1\nxs\n.clear\nm is {k: YES}\nm\nif true:\n    n is [ON]\n\nn\n.exit\n";
            let output = repl(&source.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            assert_eq!(
                text(&output.stdout),
                format!(
                    "{BANNER}[\"A\",\"B\"]\n[\"A\",\"B\"]\n1.0\n[1.0,\"B\"]\nState cleared.\n{{\"k\":\"YES\"}}\n{{\"k\":\"YES\"}}\n[\"ON\"]\n[\"ON\"]\n"
                )
            );
        }
    }
}
