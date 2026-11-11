use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-membership-tests-{label}-{}.nuc",
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
fn files_run_membership_tests_and_report_their_errors() {
    let file = SourceFile::new("file");
    let output = file.run(
        "tags = [\"red\"]\nknown = \"green\" in tags\ntags.push(\"green\")\n[known, \"red\" not in tags]",
        &[],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(text(&output.stdout), "[true,false]\n");
    for (source, expected) in [
        (
            "record = {a: 1}\n\"a\" in record",
            "TypeError: record is not a collection (line 2, column 1)\n",
        ),
        (
            "ok = all (v) in xs: v",
            "SyntaxError: Unexpected keyword 'in' (line 1, column 17)\n",
        ),
        (
            "ok = x in",
            "SyntaxError: Unexpected end of input (line 1, column 10)\n",
        ),
    ] {
        let output = file.run(source, &[]);
        assert!(!output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert_eq!(text(&output.stderr), expected, "{source}");
    }
}

#[test]
fn graph_output_shows_membership_tests_in_their_written_form() {
    let file = SourceFile::new("graph");
    let output = file.run(
        "tags = [\"a\"]\nknown = \"a\" in tags\nif \"b\" not in tags:\n    missing = true\nknown",
        &["--graph"],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        text(&output.stdout),
        "true\n\nlogic graph (3 nodes)\n  tags [variable]\n    updates  known, if((\"b\" not in tags))\n  known [variable]\n    reads    tags\n  if((\"b\" not in tags)) [if]\n    reads    tags\n"
    );
}

#[test]
fn plain_and_redirected_repls_buffer_colon_free_membership_headers() {
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let options: &[&str] = if plain { &["--plain"] } else { &[] };
            let source = "xs = [1, 2]\nif 2 in xs\n    r = 1\n\nr\ntags = [\"a\"]\nknown = \"b\" not in tags\nknown\nok = all 1 in xs: v\n.exit\n";
            let output = repl(&source.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            assert_eq!(
                text(&output.stderr),
                "SyntaxError: Unexpected keyword 'in' (line 1, column 15)\n"
            );
            assert_eq!(
                text(&output.stdout),
                format!("{BANNER}[1.0,2.0]\n1.0\n1.0\n[\"a\"]\ntrue\ntrue\n")
            );
        }
    }
}
