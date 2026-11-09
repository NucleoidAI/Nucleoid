use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-universal-claims-{label}-{}.nuc",
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
fn files_read_universal_claims_and_report_the_errors_of_their_spellings() {
    let file = SourceFile::new("file");
    let output = file.run(
        "class Human(name: str):\n    this.name = name\nevery Human is mortal\nsocrates = Human(\"Socrates\")\n[socrates.mortal, every h in Human: h.mortal]",
        &[],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(text(&output.stdout), "[true,true]\n");
    for (source, expected) in [
        (
            "every = 3\nevery Human = 4",
            "SyntaxError: Cannot assign to $Human (line 2, column 1)\n",
        ),
        (
            "ok = every v in xs",
            "SyntaxError: Expected ':' but found end of input (line 1, column 19)\n",
        ),
        (
            "every Ghost is real",
            "ReferenceError: Ghost is not defined (line 1, column 1)\n",
        ),
        (
            "class Order\n$Order is 5",
            "ReferenceError: is is not defined (line 2, column 8)\n",
        ),
    ] {
        let output = file.run(source, &[]);
        assert!(!output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert_eq!(text(&output.stderr), expected, "{source}");
    }
}

#[test]
fn plain_and_redirected_repls_buffer_colon_free_every_quantifier_headers() {
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let options: &[&str] = if plain { &["--plain"] } else { &[] };
            let source = "xs = [1, 2]\nif every v in xs: v > 0\n    r = 1\n\nr\nclass Human\nevery Human is mortal\nh = Human()\nh.mortal\nok = every v in xs\n.clear\nevery Human is mortal\n.exit\n";
            let output = repl(&source.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            assert_eq!(
                text(&output.stderr),
                "SyntaxError: Expected ':' but found end of line (line 1, column 17)\nReferenceError: Human is not defined (line 1, column 1)\n"
            );
            assert_eq!(
                text(&output.stdout),
                format!(
                    "{BANNER}[1.0,2.0]\n1.0\n1.0\nnull\nnull\n{{\"id\":\"h\",\"mortal\":true}}\ntrue\nState cleared.\n"
                )
            );
        }
    }
}

#[test]
fn graph_output_shows_predicate_sentences_as_class_level_rules() {
    let file = SourceFile::new("graph");
    for sentence in ["$Human is mortal", "every Human is mortal"] {
        let output = file.run(
            &format!("class Human\n{sentence}\nh = Human()\nh.mortal"),
            &["--graph"],
        );
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert_eq!(
            text(&output.stdout),
            "true\n\nlogic graph (3 nodes)\n  $Human [class]\n  h [object]\n  h.mortal [property]\n"
        );
    }
}
