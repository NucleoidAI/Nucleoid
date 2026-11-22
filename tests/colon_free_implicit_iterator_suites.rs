mod common;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_program};
use nucleoid::{ErrorKind, Position, Runtime, Value};

#[test]
fn colon_free_layout_preserves_for_nodes_positions_and_canonical_keys() {
    for layout in ["\n    ", "\r\n\t", " # header\n\n    ", "\n;\n    "] {
        let omitted = parse_program(&format!("for values{layout}pass\nnext = 3")).unwrap();
        let legacy = parse_program(&format!("for values:{layout}pass\nnext = 3")).unwrap();
        assert_eq!(omitted.statements, legacy.statements);
        assert_eq!(omitted.positions, legacy.positions);
        assert_eq!(
            generate_all(&omitted.statements),
            generate_all(&legacy.statements)
        );
        let [
            Stmt::For {
                variable,
                iterable,
                body,
            },
            _,
        ] = omitted.statements.as_slice()
        else {
            panic!("expected the existing For followed by the dedented assignment");
        };
        assert_eq!(variable, "it");
        assert_eq!(iterable, &Expr::Identifier("values".to_string()));
        assert_eq!(body, &[Stmt::Pass]);
        assert_eq!(
            NodeKey::block(&omitted.statements, None),
            NodeKey::block(&legacy.statements, None)
        );
    }
}

#[test]
fn qualifying_body_and_source_errors_keep_positions_rollback_and_recovery() {
    for (source, legacy, kind, message) in [
        (
            "saved = 99\nfor missing\n    pass",
            "saved = 99\nfor missing:\n    pass",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "broken = null\nsaved = 99\nfor broken\n    pass",
            "broken = null\nsaved = 99\nfor broken:\n    pass",
            ErrorKind::Type,
            "broken is not iterable",
        ),
        (
            "saved = 99\nfor text\n    saved = )",
            "saved = 99\nfor text:\n    saved = )",
            ErrorKind::Syntax,
            "Unexpected ')'",
        ),
    ] {
        let mut runtime = Runtime::new();
        runtime.run("saved = 7\ntext = \"ab\"").unwrap();
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(error.message(), message);
        assert!(error.position().is_some());
        assert_eq!(error, runtime.run(legacy).unwrap_err());
        assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
        runtime
            .run("for text\n    saved += 1\nassert(saved, 9)")
            .unwrap();
        assert!(runtime.take_assertions().is_empty());
        assert_eq!(runtime.assertions_run(), 1);
    }

    let mut omitted = String::new();
    let mut legacy = String::new();
    for depth in 0..65 {
        let indentation = "    ".repeat(depth);
        omitted.push_str(&format!("{indentation}for text\n"));
        legacy.push_str(&format!("{indentation}for text:\n"));
    }
    let indentation = "    ".repeat(65);
    omitted.push_str(&format!("{indentation}pass"));
    legacy.push_str(&format!("{indentation}pass"));
    let error = Runtime::check(&omitted).unwrap_err();
    assert_eq!(error, Runtime::check(&legacy).unwrap_err());
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(error.message(), "Expressions are nested too deeply");
}

#[test]
fn colon_omission_does_not_widen_sources_or_explicit_loop_suites() {
    Runtime::check("for values\n    pass").unwrap();
    for (source, message, position) in [
        (
            "for values;\n    pass",
            "Expected 'of' in for statement",
            Position::new(1, 11),
        ),
        (
            "for values.items\n    pass",
            "Expected 'of' in for statement",
            Position::new(1, 11),
        ),
        (
            "for values: pass",
            "Expected 'of' in for statement",
            Position::new(1, 11),
        ),
        (
            "for item of Items\n    pass",
            "Expected ':' but found end of line",
            Position::new(1, 13),
        ),
        (
            "rec = {a: 1}\nfor k, v in rec\n    pass\n",
            "Expected ':' but found end of line",
            Position::new(2, 13),
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(position), "{source}");
    }
    for source in [
        "for values",
        "for values\npass",
        "for values pass",
        "for values { pass }",
        "for values:;\n    pass",
        "for values[0]\n    pass",
        "for values()\n    pass",
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(
            error.message(),
            "Expected 'of' in for statement",
            "{source}"
        );
    }
}

#[test]
fn all_six_paired_cases_execute_their_fourteen_assertions_and_expected_returns() {
    let documents = [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ];
    let paired = [
        (
            0,
            "Nucleoid omits colons from implicit instance iterator suites",
            3,
            None,
        ),
        (
            0,
            "Nucleoid nests colon free implicit iterator scopes",
            2,
            None,
        ),
        (
            0,
            "Nucleoid propagates returns and rollback through colon free implicit iterators",
            5,
            Some("9"),
        ),
        (
            1,
            "Nucleoid iterates named strings without a suite colon",
            2,
            None,
        ),
        (
            2,
            "Named iterator suites use indentation without a colon",
            1,
            None,
        ),
        (
            3,
            "Colon-free implicit iterators keep population walks one shot",
            1,
            Some("5"),
        ),
    ];
    let mut executed = 0;
    let mut legacy_executed = 0;
    for (document, title, count, expected) in paired {
        let matching: Vec<_> = documents[document]
            .iter()
            .filter(|case| case.title == title)
            .collect();
        let [case] = matching.as_slice() else {
            panic!("expected exactly one paired case titled {title}");
        };
        assert_eq!(case.expected.as_deref(), expected);
        assert_eq!(
            common::assertions::protected_assertions(&parse(&case.source).unwrap()).len(),
            count
        );
        let mut runtime = Runtime::new();
        common::check_with_count(&mut runtime, case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assert_eq!(runtime.assertions_run(), count, "{title}");
        executed += runtime.assertions_run();

        let legacy = common::Case {
            title: case.title.clone(),
            source: case
                .source
                .lines()
                .map(|line| {
                    if line.trim_start().starts_with("for ") {
                        format!("{line}:")
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            expected: case.expected.clone(),
        };
        assert_eq!(parse(&case.source).unwrap(), parse(&legacy.source).unwrap());
        let mut runtime = Runtime::new();
        common::check_with_count(&mut runtime, &legacy, count)
            .unwrap_or_else(|reason| panic!("legacy {title}: {reason}"));
        assert_eq!(runtime.assertions_run(), count, "legacy {title}");
        legacy_executed += runtime.assertions_run();
    }
    assert_eq!(executed, 14);
    assert_eq!(legacy_executed, 14);
}

fn repl(source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
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

#[test]
fn named_iterator_repl_keeps_blank_line_eof_and_error_recovery() {
    for ending in ["", "\n\nseen\n.exit\n"] {
        let omitted =
            format!("text = \"ab\"\nseen = []\nfor text # source\n    seen.push(it)\nseen{ending}");
        let legacy = format!(
            "text = \"ab\"\nseen = []\nfor text: # source\n    seen.push(it)\nseen{ending}"
        );
        let output = repl(&omitted);
        let control = repl(&legacy);
        assert!(output.status.success(), "{output:?}");
        assert!(control.status.success(), "{control:?}");
        assert_eq!(output.stdout, control.stdout);
        assert_eq!(output.stderr, b"");
        assert_eq!(control.stderr, b"");
        assert!(output.stdout.ends_with(b"[\"a\",\"b\"]\n"), "{output:?}");
    }
    let recovered = repl("for text\n\n7\n.exit\n");
    assert!(recovered.status.success(), "{recovered:?}");
    let errors = std::str::from_utf8(&recovered.stderr).unwrap();
    assert!(errors.contains("Expected 'of' in for statement"));
    assert_eq!(errors.lines().count(), 1);
    assert!(recovered.stdout.ends_with(b"7.0\n"), "{recovered:?}");
}

#[test]
fn contextual_identifier_sources_keep_both_iterator_spellings() {
    for name in ["while", "enum", "model", "repeat", "until"] {
        let tokens = tokenize(name).unwrap();
        assert!(matches!(
            tokens.first().map(|spanned| &spanned.token),
            Some(Token::Identifier(identifier)) if identifier == name
        ));
        let omitted = format!(
            "{name} = \"ab\"\nit = \"outside\"\nseen = []\nfor {name}\n    seen.push(it)\n\
             assert(seen, [\"a\", \"b\"])\nassert(it, \"outside\")\nseen"
        );
        let legacy = omitted.replace(&format!("for {name}\n"), &format!("for {name}:\n"));
        let program = parse_program(&omitted).unwrap();
        let control = parse_program(&legacy).unwrap();
        assert_eq!(program.statements, control.statements, "{name}");
        assert_eq!(program.positions, control.positions, "{name}");
        assert_eq!(
            generate_all(&program.statements),
            generate_all(&control.statements),
            "{name}"
        );
        for source in [&omitted, &legacy] {
            let case = common::Case {
                title: name.to_string(),
                source: source.clone(),
                expected: Some("[\"a\",\"b\"]".to_string()),
            };
            let mut runtime = Runtime::new();
            common::check_with_count(&mut runtime, &case, 2).unwrap();
            assert_eq!(runtime.assertions_run(), 2, "{name}");
        }
        let output = repl(&format!("{omitted}\n\n.exit\n"));
        let control = repl(&format!("{legacy}\n\n.exit\n"));
        assert!(output.status.success(), "{name}: {output:?}");
        assert!(control.status.success(), "{name}: {control:?}");
        assert_eq!(output.stdout, control.stdout, "{name}");
        assert_eq!(output.stderr, b"", "{name}");
        assert_eq!(control.stderr, b"", "{name}");
        assert!(
            output.stdout.ends_with(b"[\"a\",\"b\"]\n"),
            "{name}: {output:?}"
        );
    }

    let omitted = r#"text = "ab"
seen = []
for text
    repeat:
        seen.push(it)
    until true
repeat:
    for text
        seen.push(it)
until seen.length >= 4
assert(seen, ["a", "b", "a", "b"])
seen"#;
    let legacy = omitted.replace("for text\n", "for text:\n");
    assert_eq!(parse(omitted).unwrap(), parse(&legacy).unwrap());
    for source in [omitted, legacy.as_str()] {
        let case = common::Case {
            title: "implicit iterator and repeat-until nesting".to_string(),
            source: source.to_string(),
            expected: Some("[\"a\",\"b\",\"a\",\"b\"]".to_string()),
        };
        let mut runtime = Runtime::new();
        common::check_with_count(&mut runtime, &case, 1).unwrap();
        assert_eq!(runtime.assertions_run(), 1);
    }
}
