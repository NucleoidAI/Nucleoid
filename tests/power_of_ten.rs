use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .replace("\r\n", "\n")
}

fn production_sources() -> Vec<PathBuf> {
    let mut pending = vec![root().join("src")];
    let mut found = Vec::new();

    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                found.push(path);
            }
        }
    }

    found.sort();
    found
}

fn lints(source: &str, opening: &str) -> Vec<String> {
    let start = source
        .find(opening)
        .unwrap_or_else(|| panic!("missing {opening}"));
    let body = &source[start + opening.len()..];
    let end = body.find(')').unwrap();
    body[..end]
        .split(',')
        .map(str::trim)
        .filter(|lint| !lint.is_empty())
        .map(str::to_string)
        .collect()
}

fn code_lines(source: &str) -> impl Iterator<Item = (usize, &str)> {
    source
        .lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line.trim()))
        .filter(|(_, line)| !line.starts_with("//"))
}

#[test]
fn library_and_binary_enforce_the_same_lint_policy() {
    let library = read(&root().join("src/lib.rs"));
    let binary = read(&root().join("src/main.rs"));

    for opening in ["#![warn(", "#![cfg_attr(\n    test,\n    allow("] {
        let enforced = lints(&library, opening);
        assert!(!enforced.is_empty(), "{opening}");
        assert_eq!(enforced, lints(&binary, opening), "{opening}");
    }

    let enforced = lints(&library, "#![warn(");
    for lint in [
        "clippy::pedantic",
        "clippy::panic",
        "clippy::unwrap_used",
        "clippy::expect_used",
        "clippy::indexing_slicing",
        "clippy::infinite_loop",
        "clippy::let_underscore_must_use",
    ] {
        assert!(enforced.iter().any(|enforced| enforced == lint), "{lint}");
    }
}

#[test]
fn functions_are_held_to_sixty_lines() {
    let configuration = read(&root().join("clippy.toml"));
    assert!(
        configuration
            .lines()
            .any(|line| line.replace(' ', "") == "too-many-lines-threshold=60"),
        "{configuration}"
    );
}

#[test]
fn production_code_avoids_macros_dynamic_dispatch_and_function_pointers() {
    let forbidden = [
        ("macro_rules!", Regex::new(r"\bmacro_rules!").unwrap()),
        ("trait object", Regex::new(r"\bdyn\s").unwrap()),
        ("function pointer", Regex::new(r"\bfn\s*\(").unwrap()),
        (
            "shared mutability",
            Regex::new(r"\b(Rc|RefCell|Cell)<").unwrap(),
        ),
    ];

    for path in production_sources() {
        let source = read(&path);
        for (number, line) in code_lines(&source) {
            for (construct, pattern) in &forbidden {
                assert!(
                    !pattern.is_match(line),
                    "{construct} at {}:{number}: {line}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn conditional_compilation_is_limited_to_tests_and_platform_keys() {
    let attribute = Regex::new(r"#!?\[cfg(?:_attr)?\(\s*([^,)\s]+)").unwrap();
    let macro_call = Regex::new(r"\bcfg!\(([^)]*)\)").unwrap();

    for path in production_sources() {
        let source = read(&path);
        for captures in attribute.captures_iter(&source) {
            assert_eq!(&captures[1], "test", "{}", path.display());
        }
        for captures in macro_call.captures_iter(&source) {
            assert_eq!(&captures[1], "windows", "{}", path.display());
        }
    }
}
