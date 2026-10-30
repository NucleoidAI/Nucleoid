use std::io::Write;
use std::process::{Command, Output, Stdio};

fn repl(arguments: &[&str], source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(arguments)
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

#[test]
fn redirected_stdio_automatically_keeps_the_line_repl_without_screen_controls() {
    let output = repl(
        &["--json"],
        "class Person(name):\n    this.name = name\n\nperson = Person(\"Ada\")\nperson\n.exit\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let text = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        text.lines()
            .any(|line| line == "{\"id\":\"person\",\"name\":\"Ada\"}"),
        "{text}"
    );
    assert!(!text.contains('\u{1b}'), "{text}");
}

#[test]
fn plain_mode_preserves_json_graph_commands_and_error_recovery() {
    let output = repl(
        &["--plain", "--json", "--graph"],
        "value = {name: \"Ada\", score: 9}\nvalue = (\nvalue\n.clear\n.quit\n",
    );
    assert!(output.status.success(), "{output:?}");
    let text = std::str::from_utf8(&output.stdout).unwrap();
    let errors = std::str::from_utf8(&output.stderr).unwrap();
    assert!(errors.contains("SyntaxError"), "{errors}");
    assert!(text.contains("{\"name\":\"Ada\",\"score\":9.0}"), "{text}");
    assert!(text.contains("logic graph"), "{text}");
    assert!(text.contains("State cleared."), "{text}");
    assert!(!text.contains('\u{1b}'), "{text}");
}
