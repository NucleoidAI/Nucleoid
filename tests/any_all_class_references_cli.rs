use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

const EXAMPLE: &str = "class Object(number):\n    this.number = number\n\nevery Object.label is square\n\nif any Object.number > 4:\n  all Object.color is red\n\na = Object(3)\nb = Object(7)\n[a.label, b.label, a.color, b.color]";

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-any-all-class-references-{label}-{}.nuc",
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
fn files_run_the_users_example_and_report_the_errors_of_the_dollar_spelling() {
    let file = SourceFile::new("file");
    let output = file.run(EXAMPLE, &["--json"]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        text(&output.stdout),
        "[\"square\",\"square\",null,\"red\"]\n"
    );
    let dollar = file.run(
        &EXAMPLE
            .replace("any Object", "$Object")
            .replace("all Object", "$Object"),
        &["--json"],
    );
    assert_eq!(dollar.stdout, output.stdout);
    for (source, expected) in [
        (
            "all = 3\nall Human = 4",
            "SyntaxError: Cannot assign to $Human (line 2, column 1)\n",
        ),
        (
            "any = 3\nany Human = 4",
            "SyntaxError: Cannot assign to $Human (line 2, column 1)\n",
        ),
        (
            "c = 1\nall = 2\nx = all c\nx",
            "ReferenceError: c is not defined (line 3, column 1)\n",
        ),
        (
            "any Ghost is real",
            "ReferenceError: Ghost is not defined (line 1, column 1)\n",
        ),
        (
            "all x => x",
            "SyntaxError: Unexpected '=>' (line 1, column 10)\n",
        ),
    ] {
        let output = file.run(source, &["--json"]);
        assert!(!output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert_eq!(text(&output.stderr), expected, "{source}");
    }
}

#[test]
fn graph_output_shows_spelled_conditionals_as_their_dollar_spelling() {
    let file = SourceFile::new("graph");
    let spelled = file.run(EXAMPLE, &["--graph"]);
    assert!(spelled.status.success(), "{spelled:?}");
    assert!(spelled.stderr.is_empty(), "{spelled:?}");
    let dollar = file.run(
        &EXAMPLE
            .replace("every Object", "$Object")
            .replace("any Object", "$Object")
            .replace("all Object", "$Object"),
        &["--graph"],
    );
    assert_eq!(text(&spelled.stdout), text(&dollar.stdout));
    assert!(
        text(&spelled.stdout)
            .contains("  if($Object.number>4)@b [if]\n    reads    b.number, red\n"),
        "{}",
        text(&spelled.stdout)
    );
    for sentence in ["all Human is mortal", "any Human is mortal"] {
        let output = file.run(
            &format!("class Human\n{sentence}\nh = Human()\nh.mortal"),
            &["--graph"],
        );
        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            text(&output.stdout),
            "true\n\nlogic graph (3 nodes)\n  $Human [class]\n  h [object]\n  h.mortal [property]\n"
        );
    }
}

#[test]
fn plain_and_redirected_repls_buffer_colon_free_any_class_headers() {
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let options: &[&str] = if plain { &["--plain"] } else { &[] };
            let source = "class Object(number):\n    this.number = number\n\nevery Object.label is square\nif any Object.number > 4\n    all Object.color is red\n\nb = Object(7)\nb.color\nif any Object.number > 9:\n    all Object.size is big\n\nb.size\nall = 3\nall Human = 4\n.clear\nall Object.color is red\n.exit\n";
            let output = repl(&source.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            assert_eq!(
                text(&output.stderr),
                "SyntaxError: Cannot assign to $Human (line 1, column 1)\nReferenceError: Object is not defined (line 1, column 1)\n"
            );
            assert_eq!(
                text(&output.stdout),
                format!(
                    "{BANNER}null\nnull\nnull\n{{\"color\":\"red\",\"id\":\"b\",\"label\":\"square\",\"number\":7.0}}\n\"red\"\nnull\nnull\n3.0\nState cleared.\n"
                )
            );
        }
    }
}
