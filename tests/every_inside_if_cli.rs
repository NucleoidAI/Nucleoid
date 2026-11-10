use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

const ORDERS: &str = "class Order(total):\n    this.total = total\nlarge = Order(200)\nif any Order.total > 100:\n    all Order.vip = true\nsmall = Order(5)\n[small.vip, large.vip]";

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-every-inside-if-{label}-{}.nuc",
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
fn files_report_every_inside_if_before_they_run() {
    let file = SourceFile::new("file");
    let output = file.run(ORDERS, &["--json"]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(text(&output.stdout), "[null,true]\n");
    for (source, expected) in [
        (
            ORDERS.replace("any Order", "every Order"),
            "SyntaxError: Use all instead of every inside if (line 4, column 4)\n",
        ),
        (
            ORDERS.replace("all Order", "every Order"),
            "SyntaxError: Use all instead of every inside if (line 5, column 5)\n",
        ),
        (
            format!("x = 1\n{}", ORDERS.replace("any Order", "every Order")),
            "SyntaxError: Use all instead of every inside if (line 5, column 4)\n",
        ),
        (
            "c = true\nif c: every Order is gold".to_string(),
            "SyntaxError: Use all instead of every inside if (line 2, column 7)\n",
        ),
    ] {
        for options in [&[][..], &["--json"][..], &["--graph"][..]] {
            let output = file.run(&source, options);
            assert_eq!(output.status.code(), Some(1), "{output:?}");
            assert!(output.stdout.is_empty(), "{output:?}");
            assert_eq!(text(&output.stderr), expected, "{source}");
        }
    }
    let output = file.run(
        "class Order(total):\n    this.total = total\nevery Order.big = every Order.total > 10\nlarge = Order(200)\nlarge.big",
        &["--graph"],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(text(&output.stdout).starts_with("true\n\nlogic graph"));
}

#[test]
fn plain_and_redirected_repls_report_every_inside_buffered_if_statements() {
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let options: &[&str] = if plain { &["--plain"] } else { &[] };
            let source = "class Order(total):\n    this.total = total\n\nif every Order.total > 100:\n    every Order.vip = true\n\nif any Order.total > 100:\n    all Order.vip = true\n\nlarge = Order(200)\nlarge.vip\nif every Order.total > 100\n    every Order.vip = false\n\nlarge.vip\nif large.vip: every Order.tier is gold\nevery Order.tier is gold\nlarge.tier\n.exit\n";
            let output = repl(&source.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            assert_eq!(
                text(&output.stderr),
                "SyntaxError: Use all instead of every inside if (line 1, column 4)\nSyntaxError: Use all instead of every inside if (line 1, column 4)\nSyntaxError: Use all instead of every inside if (line 1, column 15)\n"
            );
            assert_eq!(
                text(&output.stdout),
                format!(
                    "{BANNER}null\nnull\n{{\"id\":\"large\",\"total\":200.0,\"vip\":true}}\ntrue\ntrue\nnull\n\"gold\"\n"
                )
            );
        }
    }
}
