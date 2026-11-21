use std::io::Write;
use std::process::{Command, Output, Stdio};

use serde_json::{Value as Json, json};

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

#[test]
fn default_repl_resolves_objects_and_nested_lists_without_json_flags() {
    for arguments in [&[][..], &["--plain"][..]] {
        let output = repl(
            arguments,
            "item = {count: 1, nested: {label: \"Ada\"}}\n.json off\nitem\n[[{count: 2}]]\nclass Person(name): this.name = name\nperson = Person(\"Ada\")\nperson\n.exit\n",
        );
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let text = std::str::from_utf8(&output.stdout).unwrap();
        let results: Vec<Json> = text
            .lines()
            .filter(|line| line.starts_with('{') || line.starts_with('['))
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(
            results,
            [
                json!({"count": 1.0, "nested": {"label": "Ada"}}),
                json!({"count": 1.0, "nested": {"label": "Ada"}}),
                json!([[{"count": 2.0}]]),
                json!({"id": "person", "name": "Ada"}),
                json!({"id": "person", "name": "Ada"}),
            ],
            "{text}"
        );
    }
}

#[test]
fn automatic_object_output_preserves_plain_values_and_explicit_json_settings() {
    let output = repl(
        &["--plain"],
        "\"hello\"\n[1, 2]\n\"[object obj:1]\"\n.json on\n\"hello\"\n.json off\n\"hello\"\n.exit\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let text = std::str::from_utf8(&output.stdout).unwrap();
    let lines: Vec<_> = text.lines().collect();
    assert!(
        lines.windows(7).any(|lines| lines
            == [
                "hello",
                "1,2",
                "[object obj:1]",
                "json on.",
                "\"hello\"",
                "json off.",
                "hello",
            ]),
        "{text}"
    );
}

#[test]
fn default_object_output_reports_cycles_and_keeps_the_repl_usable() {
    let output = repl(
        &["--plain"],
        "item = {}\nitem.self = item\nitem\n7\n.exit\n",
    );
    assert!(output.status.success(), "{output:?}");
    let text = std::str::from_utf8(&output.stdout).unwrap();
    let errors = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        errors.contains("TypeError: Converting circular structure to JSON"),
        "{errors}"
    );
    assert!(!text.contains("[object"), "{text}");
    assert!(text.lines().any(|line| line == "7"), "{text}");
}
