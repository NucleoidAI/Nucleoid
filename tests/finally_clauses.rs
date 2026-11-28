#[expect(dead_code)]
#[path = "../build.rs"]
mod build_exporter;

mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use build_exporter::build_statements::statements as exported_statements;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::{generate, generate_all};
use nucleoid::lang::ast::{Expr, FunctionBody, Stmt, parse, parse_program};
use nucleoid::nuc::Nuc;
use nucleoid::nuc::r#try::Try;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, Position, Runtime, Value};

const SPEC_TITLES: [&str; 3] = [
    "Nucleoid runs a finally clause however a try statement ends",
    "Nucleoid rolls back a failed try body before its finally clause runs",
    "Nucleoid runs a finally clause before an error or a return leaves",
];
const REFERENCE_TITLE: &str = "Nucleoid runs finally clauses as Python, JavaScript and Java do";
const README_TITLE: &str = "Finally clauses record every attempt to unlock a door";
const EXAMPLES_TITLE: &str = "Finally clauses log every dispatch of a driver";

fn documents() -> [(&'static str, Vec<common::Case>); 4] {
    [
        ("spec", common::cases(include_str!("../nucleoid.spec.md"))),
        ("reference", common::cases(include_str!("reference.md"))),
        (
            "readme",
            common::snippets(include_str!("../README.md"), "README.md"),
        ),
        (
            "examples",
            common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
        ),
    ]
}

fn is_appended(title: &str) -> bool {
    SPEC_TITLES.contains(&title) || [REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE].contains(&title)
}

fn block_holds_finally(body: &FunctionBody) -> bool {
    matches!(body, FunctionBody::Block(statements) if contains_finally(statements))
}

fn contains_finally(statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| match statement {
        Stmt::TryFinally { .. } => true,
        Stmt::If {
            consequent,
            alternate,
            ..
        } => {
            contains_finally(consequent)
                || alternate
                    .as_ref()
                    .is_some_and(|statement| contains_finally(std::slice::from_ref(statement)))
        }
        Stmt::Block(body)
        | Stmt::For { body, .. }
        | Stmt::ForEntries { body, .. }
        | Stmt::While { body, .. }
        | Stmt::Repeat { body, .. }
        | Stmt::Suppose { body, .. } => contains_finally(body),
        Stmt::Try { body, catch, .. } => contains_finally(body) || contains_finally(catch),
        Stmt::Function(function) => block_holds_finally(&function.body),
        Stmt::Class(class) => {
            contains_finally(&class.constructor)
                || class
                    .methods
                    .iter()
                    .any(|method| block_holds_finally(&method.body))
        }
        _ => false,
    })
}

fn finally_anywhere(statements: &[Stmt]) -> bool {
    format!("{statements:?}").contains("TryFinally {")
}

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "finally clause regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
}

fn outcome(runtime: &mut Runtime, source: &str) -> String {
    match runtime.run(source) {
        Ok(value) => runtime.serialize_json(&value).unwrap(),
        Err(error) => error.to_string(),
    }
}

fn fresh(source: &str) -> String {
    outcome(&mut Runtime::new(), source)
}

fn failure(source: &str) -> (ErrorKind, String, Option<Position>) {
    let error = Runtime::new().run(source).unwrap_err();
    (error.kind(), error.message(), error.position())
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn equivalent(source: &str, expected: &str) -> Vec<Stmt> {
    let actual = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    let wanted = parse(expected).unwrap_or_else(|error| panic!("{expected}: {error}"));
    assert_eq!(actual, wanted, "{source}");
    assert_eq!(generate_all(&actual), generate_all(&wanted), "{source}");
    actual
}

fn push(value: f64) -> Stmt {
    Stmt::Expression(Expr::Call {
        callee: Box::new(Expr::Member {
            object: Box::new(Expr::Identifier("log".to_string())),
            property: "push".to_string(),
        }),
        arguments: vec![Expr::Number(value)],
    })
}

#[test]
fn only_the_appended_cases_hold_finally_clauses_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let statements =
            parse(&case.source).unwrap_or_else(|error| panic!("{}: {error}", case.title));
        let new = is_appended(&case.title);
        assert_eq!(contains_finally(&statements), new, "{}", case.title);
        assert_eq!(finally_anywhere(&statements), new, "{}", case.title);
        if new {
            let count = common::assertions::protected_assertions(&statements).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            let mut runtime = Runtime::new();
            common::check_with_count(&mut runtime, case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assert_eq!(runtime.assertions_run(), count, "{}", case.title);
            assertions += count;
            let tree = format!("{statements:?}");
            for nested in [
                "Repeat {",
                "While {",
                "Suppose {",
                "ValueAssign {",
                "ForEntries {",
            ] {
                assert!(!tree.contains(nested), "{} holds {nested}", case.title);
            }
        }
    }
    for title in SPEC_TITLES
        .into_iter()
        .chain([REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE])
    {
        let matching = documents
            .iter()
            .flat_map(|(_, cases)| cases)
            .filter(|case| case.title == title)
            .count();
        assert_eq!(matching, 1, "{title}");
    }
    assert_eq!(assertions, 20);
    assert_eq!(cases, 1348);
}

#[test]
fn the_audit_finds_every_finally_clause_and_nothing_else() {
    for source in [
        "try:\n    pass\ncatch:\n    pass\nfinally:\n    pass",
        "try: pass\ncatch: pass\nfinally: pass",
        "if true:\n    try:\n        pass\n    catch:\n        pass\n    finally:\n        pass",
        "for v of [1]:\n    try: pass\n    catch: pass\n    finally: pass",
        "{\n    try: pass\n    catch: pass\n    finally: pass\n}",
        "try:\n    try: pass\n    catch: pass\n    finally: pass\ncatch: pass",
        "try: pass\ncatch:\n    try: pass\n    catch: pass\n    finally: pass",
        "def f():\n    try:\n        return 1\n    catch:\n        return 2\n    finally:\n        pass",
        "class C:\n    def m():\n        try: pass\n        catch: pass\n        finally: pass",
        "class C(v):\n    try: this.v = v\n    catch: pass\n    finally: pass",
        "while false:\n    try: pass\n    catch: pass\n    finally: pass",
        "repeat 1:\n    try: pass\n    catch: pass\n    finally: pass",
    ] {
        let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert!(contains_finally(&statements), "{source}");
        assert!(finally_anywhere(&statements), "{source}");
    }
    let callback = "f = (k) => {\n    try:\n        x = 1\n    catch:\n        x = 2\n    finally:\n        y = 3\n    return y\n}";
    let statements = parse(callback).unwrap();
    assert!(!contains_finally(&statements), "{callback}");
    assert!(finally_anywhere(&statements), "{callback}");
    for source in [
        "try:\n    pass\ncatch:\n    pass",
        "finally = 1",
        "finally(1)",
        "o = {finally: 1}\no.finally",
        "try:\n    x = 1\ncatch:\n    x = 2\nfinally: x",
        "try:\n    x = 1\ncatch:\n    x = 2\nfinally\n    x = 3",
        "# finally:\nx = 1",
        "s = \"finally: x = 1\"",
        "def finally(v) => v",
    ] {
        let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert!(!contains_finally(&statements), "{source}");
        assert!(!finally_anywhere(&statements), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_118_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[789],
        "Nucleoid rolls back failed updates through case conversion aliases"
    );
    assert_eq!(titles[790..793], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid converts case as Kotlin, Rust and Swift do",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Case conversion methods keep a sensor's labels normalized",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Case conversions match badge scans to every employee",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 23 - 12 * usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 12].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Finally clauses\n\n```\n")
        .expect("the language reference documents finally clauses");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 119](nuc-0119.md)"));
    for line in [
        "try:                                   run the body; a failure rolls it back",
        "catch name:                            then runs this suite with the error bound",
        "finally:                               runs however the statement ends (NUC 119)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    assert_eq!(case.expected.as_deref(), Some("2"));
}

#[test]
fn every_document_case_exports_its_clauses_with_their_try_statements() {
    for case in documents()
        .iter()
        .flat_map(|(_, cases)| cases)
        .filter(|case| is_appended(&case.title))
    {
        let parts = exported_statements(&case.source);
        let whole = parse_program(&case.source).unwrap().statements;
        let split: Vec<Stmt> = parts
            .iter()
            .flat_map(|part| {
                parse_program(part)
                    .unwrap_or_else(|error| panic!("{}: {part}\n{error}", case.title))
                    .statements
            })
            .collect();
        assert_eq!(whole, split, "{}", case.title);
        for part in &parts {
            let trimmed = part.trim_start();
            assert!(!trimmed.starts_with("finally"), "{}: {part}", case.title);
        }
        assert!(contains_finally(&whole), "{}", case.title);
    }
    for source in [
        "try:\n    x = 1\ncatch:\n    x = 2\nfinally:\n    x = 3\ny = x",
        "try: x = 1\ncatch: x = 2\nfinally: x = 3\ny = x",
        "try: x = 1\ncatch: x = 2\nfinally: x = 3; z = 4\ny = x",
        "try: x = 1\ncatch: x = 2\nfinally: { x = 3 }\ny = x",
        "try:\n    x = 1\ncatch:\n    x = 2\n\n# then\nfinally:\n    x = 3\ny = x",
        "try\n    x = 1\ncatch problem\n    x = 2\nfinally:\n    x = 3\ny = x",
    ] {
        let parts = exported_statements(source);
        assert_eq!(parts.len(), 2, "{source}\n{parts:#?}");
        assert_eq!(parts[1], "y = x", "{source}");
        let whole = parse_program(source).unwrap().statements;
        let split: Vec<Stmt> = parts
            .iter()
            .flat_map(|part| parse_program(part).unwrap().statements)
            .collect();
        assert_eq!(whole, split, "{source}");
        assert!(matches!(whole[0], Stmt::TryFinally { .. }), "{source}");
    }
}

#[test]
fn every_suite_form_builds_one_try_finally_statement() {
    let canonical =
        "try:\n    log.push(1)\ncatch error:\n    log.push(2)\nfinally:\n    log.push(3)";
    let expected = [Stmt::TryFinally {
        body: vec![push(1.0)],
        parameter: "error".to_string(),
        catch: vec![push(2.0)],
        finally: vec![push(3.0)],
    }];
    assert_eq!(parse(canonical).unwrap(), expected);
    for source in [
        "try: log.push(1)\ncatch: log.push(2)\nfinally: log.push(3)",
        "try:\n    log.push(1)\ncatch:\n    log.push(2)\nfinally :\n    log.push(3)",
        "try:\n    log.push(1)\ncatch:\n    log.push(2)\n\n\nfinally:\n    log.push(3)",
        "try:\n    log.push(1)\ncatch:\n    log.push(2)\n# after the catch\nfinally:\n    log.push(3)",
        "try:\n    log.push(1)\ncatch:\n    log.push(2)\n    # inside the catch\n  # shallower\nfinally:  # the clause\n    log.push(3)",
        "try:\r\n    log.push(1)\r\ncatch:\r\n    log.push(2)\r\nfinally:\r\n    log.push(3)",
        "try:\n\tlog.push(1)\ncatch:\n\tlog.push(2)\nfinally:\n\tlog.push(3)",
        "try\n    log.push(1)\ncatch\n    log.push(2)\nfinally:\n    log.push(3)",
        "try:\n    log.push(1)\ncatch: log.push(2)\nfinally:\n    log.push(3)",
        "try: log.push(1)\ncatch:\n    log.push(2)\nfinally: log.push(3)",
        "try:\n    log.push(1)\ncatch:\n    log.push(2)\n  finally:\n    log.push(3)",
        "try:\n    log.push(1)\n  catch:\n    log.push(2)\nfinally:\n    log.push(3)",
    ] {
        equivalent(source, canonical);
    }
    for (header, parameter) in [
        ("catch", "error"),
        ("catch problem", "problem"),
        ("catch(problem)", "problem"),
        ("catch (problem)", "problem"),
        ("catch\tproblem", "problem"),
        ("catch ( problem )", "problem"),
    ] {
        for (colon, layout) in [(":", "\n    "), (":", " "), ("", "\n    ")] {
            let source = format!(
                "try:\n    log.push(1)\n{header}{colon}{layout}log.push(2)\nfinally: log.push(3)"
            );
            let statements = parse(&source).unwrap_or_else(|error| panic!("{source}: {error}"));
            assert_eq!(
                statements,
                [Stmt::TryFinally {
                    body: vec![push(1.0)],
                    parameter: parameter.to_string(),
                    catch: vec![push(2.0)],
                    finally: vec![push(3.0)],
                }],
                "{source}"
            );
        }
    }
    assert_eq!(
        parse("try: log.push(1)\ncatch: log.push(2)\nfinally: log.push(3); log.push(4)").unwrap(),
        [Stmt::TryFinally {
            body: vec![push(1.0)],
            parameter: "error".to_string(),
            catch: vec![push(2.0)],
            finally: vec![push(3.0), push(4.0)],
        }]
    );
    assert_eq!(
        parse("try: log.push(1)\ncatch: log.push(2)\nfinally: { log.push(3); log.push(4) }")
            .unwrap(),
        [Stmt::TryFinally {
            body: vec![push(1.0)],
            parameter: "error".to_string(),
            catch: vec![push(2.0)],
            finally: vec![Stmt::Block(vec![push(3.0), push(4.0)])],
        }]
    );
    assert_eq!(
        parse("try: log.push(1)\ncatch: log.push(2)").unwrap(),
        [Stmt::Try {
            body: vec![push(1.0)],
            parameter: "error".to_string(),
            catch: vec![push(2.0)],
        }]
    );
    let program = parse_program("a = 1\ntry: pass\ncatch: pass\nfinally: pass\nb = 2").unwrap();
    assert_eq!(program.statements.len(), 3);
    assert_eq!(
        program.positions,
        [
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(5, 1)
        ]
    );
}

#[test]
fn the_innermost_try_statement_takes_the_clause_that_follows_its_catch_suite() {
    let inner = "try:\n    try:\n        log.push(1)\n    catch:\n        log.push(2)\n    finally:\n        log.push(3)\ncatch:\n    log.push(4)";
    assert_eq!(
        parse(inner).unwrap(),
        [Stmt::Try {
            body: vec![Stmt::TryFinally {
                body: vec![push(1.0)],
                parameter: "error".to_string(),
                catch: vec![push(2.0)],
                finally: vec![push(3.0)],
            }],
            parameter: "error".to_string(),
            catch: vec![push(4.0)],
        }]
    );
    let outer = "try:\n    log.push(1)\ncatch:\n    try:\n        log.push(2)\n    catch:\n        log.push(3)\nfinally:\n    log.push(4)";
    assert_eq!(
        parse(outer).unwrap(),
        [Stmt::TryFinally {
            body: vec![push(1.0)],
            parameter: "error".to_string(),
            catch: vec![Stmt::Try {
                body: vec![push(2.0)],
                parameter: "error".to_string(),
                catch: vec![push(3.0)],
            }],
            finally: vec![push(4.0)],
        }]
    );
    let one_line =
        "try: log.push(1)\ncatch: try: log.push(2)\ncatch: log.push(3)\nfinally: log.push(4)";
    assert_eq!(
        parse(one_line).unwrap(),
        [Stmt::Try {
            body: vec![push(1.0)],
            parameter: "error".to_string(),
            catch: vec![Stmt::TryFinally {
                body: vec![push(2.0)],
                parameter: "error".to_string(),
                catch: vec![push(3.0)],
                finally: vec![push(4.0)],
            }],
        }]
    );
}

#[test]
fn the_kept_readings_keep_their_exact_accepted_results() {
    for (source, expected) in [
        ("finally = 3\nfinally", "3.0"),
        ("o = {finally: 2}\no.finally", "2.0"),
        (
            "try:\n    x = 1\ncatch:\n    x = 2\nfinally = 4\n[x, finally]",
            "[1.0,4.0]",
        ),
        ("o = {a: 2 finally: 3}\no.finally", "3.0"),
        ("finally = 5\nc = true\nx = c ? finally : 1\nx", "5.0"),
        ("xs = [1, 2, 3]\nfinally = 1\nxs[finally:]", "[2.0,3.0]"),
        ("def finally(v) => v * 2\nfinally(4)", "8.0"),
        (
            "try:\n    x = 1\ncatch:\n    x = 2\nfinally(3)",
            "ReferenceError: finally is not defined (line 5, column 1)",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally\n    x = 3\nx",
            "ReferenceError: finally is not defined (line 6, column 1)",
        ),
        (
            "x = 0\ntry:\n    x = 5\nfinally:\n    x = x + 10\nx",
            "SyntaxError: Expected 'catch' after 'try' (line 4, column 1)",
        ),
        (
            "x = 0\ntry: x = 1 catch: x = 2 finally: x = 3\nx",
            "SyntaxError: Unexpected ':' (line 2, column 34)",
        ),
        (
            "x = 0\ntry: x = 1 catch: x = 2; finally: x = 3\nx",
            "SyntaxError: Unexpected ':' (line 2, column 35)",
        ),
        (
            "finally:\n    x = 1",
            "SyntaxError: Unexpected ':' (line 1, column 8)",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\n    finally:\n        x = 3\nx",
            "SyntaxError: Unexpected ':' (line 6, column 12)",
        ),
        (
            "x = 2 finally\nx",
            "ReferenceError: finally is not defined (line 1, column 7)",
        ),
        (
            "x = 0\ntry: x = 1 catch: x = 2 finally\nx",
            "ReferenceError: finally is not defined (line 2, column 25)",
        ),
        (
            "def f():\n    try:\n        return 1\n    catch:\n        return 2\nfinally:\n    x = 3",
            "SyntaxError: Unexpected ':' (line 6, column 8)",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally: x",
            "ReferenceError: Missing definition (line 6, column 1)",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally: x; y = 2",
            "ReferenceError: Missing definition (line 6, column 1)",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally: int\nx",
            "ReferenceError: Missing definition (line 6, column 1)",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally: x\n    y = 1\n",
            "ReferenceError: Missing definition (line 6, column 1)",
        ),
        (
            "def g():\n    try:\n        return 1\n    catch:\n        return 2\n    finally: x\ng()",
            "1.0",
        ),
        (
            "def g():\n    try:\n        return 1\n    catch:\n        return 2\n    finally: x\n        y = 2\ng()",
            "SyntaxError: Unexpected an indented block (line 7, column 9)",
        ),
        (
            "r = [1].map(k => {\n    try:\n        throw \"x\"\n    catch e:\n        v = e\n    finally:\n        w = 1\n    return v\n})\nr",
            "SyntaxError: Unexpected ':' (line 7, column 9)",
        ),
        (
            "r = [1].map(k => {\n    try: throw \"x\"\n    catch e: v = e\n    finally: w = 1\n    return v\n})\nr",
            "SyntaxError: Unexpected ':' (line 4, column 14)",
        ),
        (
            "x = 0\ntry: x = 1\ncatch: x = 2\n    finally: x = 3\nx",
            "SyntaxError: Unexpected ':' (line 4, column 14)",
        ),
        (
            "x = 0\ntry: x = 1\ncatch: x = 2\n;finally: x = 3\nx",
            "SyntaxError: Unexpected ':' (line 4, column 11)",
        ),
        (
            "f = function(k) {\n    try:\n        x = 1\n    catch:\n        x = 2\n}\nfinally: y = 3",
            "SyntaxError: Unexpected ':' (line 7, column 10)",
        ),
        ("enum finally: A, B\nfinally.A", "\"A\""),
        ("def twice(finally) => finally * 2\ntwice(3)", "6.0"),
        (
            "total = 0\nfor finally of [1, 2]: total = total + finally\ntotal",
            "0.0",
        ),
        ("finally := 5\nfinally", "5.0"),
        (
            "finally: int",
            "ReferenceError: Missing definition (line 1, column 1)",
        ),
        ("o = {finally: 1}\no.finally = 2\no.finally", "2.0"),
        (
            "def finally(v) => v + 1\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally(3)",
            "4.0",
        ),
        (
            "finally = 1\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally += 1\nfinally",
            "2.0",
        ),
        (
            "finally = {x: 0}\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally.x = 5\nfinally.x",
            "5.0",
        ),
        (
            "finally = true\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally ? 1 : 2",
            "1.0",
        ),
        (
            "finally = [7]\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally[0]",
            "7.0",
        ),
        (
            "try:\n    pass\n\nfinally:\n    pass",
            "SyntaxError: Expected 'catch' after 'try' (line 4, column 1)",
        ),
        (
            "try: pass\nfinally: pass\ncatch: pass",
            "SyntaxError: Expected 'catch' after 'try' (line 2, column 1)",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\nelse:\n    x = 3",
            "SyntaxError: Unexpected keyword 'else' (line 6, column 5)",
        ),
        (
            "x = 0\nif x == 0:\n    x = 1\nfinally:\n    x = 2",
            "SyntaxError: Unexpected ':' (line 4, column 8)",
        ),
        (
            "x = 0\nwhile x < 1:\n    x = x + 1\nfinally:\n    x = 5",
            "SyntaxError: Unexpected ':' (line 4, column 8)",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\n;\nfinally:\n    x = 3\nx",
            "SyntaxError: Unexpected ':' (line 7, column 8)",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch: x = 2\n;\nfinally: x = 3\nx",
            "SyntaxError: Unexpected ':' (line 6, column 10)",
        ),
        (
            "def f():\n    try:\n        return 1\n    catch:\n        return 2\n  finally:\n    return 3\nf()",
            "SyntaxError: Unexpected ':' (line 6, column 10)",
        ),
    ] {
        assert_eq!(fresh(source), expected, "{source}");
    }
}

#[test]
fn every_outcome_runs_the_finally_suite_once_and_completes_as_its_suite_did() {
    let suite = |label: &str, ending: &str| match ending {
        "completes" => format!("log.push(\"{label}\")"),
        "returns" => format!("log.push(\"{label}\")\n        return \"{label}!\""),
        _ => format!("log.push(\"{label}\")\n        throw \"{label}?\""),
    };
    for (body, catch, finally, expected) in [
        (
            "completes",
            "completes",
            "completes",
            "[\"N\",[\"b\",\"f\",\"after\"]]",
        ),
        (
            "completes",
            "completes",
            "returns",
            "[\"f!\",[\"b\",\"f\"]]",
        ),
        ("completes", "completes", "fails", "[\"error f?\",[]]"),
        (
            "returns",
            "completes",
            "completes",
            "[\"b!\",[\"b\",\"f\"]]",
        ),
        ("returns", "completes", "returns", "[\"f!\",[\"b\",\"f\"]]"),
        ("returns", "completes", "fails", "[\"error f?\",[]]"),
        (
            "fails",
            "completes",
            "completes",
            "[\"N\",[\"c\",\"f\",\"after\"]]",
        ),
        ("fails", "completes", "returns", "[\"f!\",[\"c\",\"f\"]]"),
        ("fails", "completes", "fails", "[\"error f?\",[]]"),
        ("fails", "returns", "completes", "[\"c!\",[\"c\",\"f\"]]"),
        ("fails", "returns", "returns", "[\"f!\",[\"c\",\"f\"]]"),
        ("fails", "returns", "fails", "[\"error f?\",[]]"),
        ("fails", "fails", "completes", "[\"error c?\",[]]"),
        ("fails", "fails", "returns", "[\"f!\",[\"f\"]]"),
        ("fails", "fails", "fails", "[\"error f?\",[]]"),
    ] {
        let source = format!(
            "log = []\ndef run():\n    try:\n        {}\n    catch problem:\n        {}\n    finally:\n        {}\n    log.push(\"after\")\n    return \"N\"\noutcome = \"\"\ntry:\n    outcome = run()\ncatch failure:\n    outcome = \"error \" + failure\n[outcome, log]",
            suite("b", body),
            suite("c", catch),
            suite("f", finally)
        );
        assert_eq!(fresh(&source), expected, "{source}");
    }
}

#[test]
fn top_level_statements_complete_as_their_suites_did() {
    for (source, expected) in [
        (
            "log = []\ntry:\n    log.push(1)\ncatch:\n    log.push(2)\nfinally:\n    log.push(3)",
            "1.0",
        ),
        (
            "log = []\ntry:\n    throw 1\ncatch:\n    log.push(2)\nfinally:\n    log.push(3)",
            "1.0",
        ),
        (
            "log = []\ntry:\n    log.push(1)\ncatch:\n    pass\nfinally:\n    log.push(2)\nlog",
            "[1.0,2.0]",
        ),
        (
            "log = []\ntry:\n    return 1\ncatch:\n    pass\nfinally:\n    log.push(2)\nlog",
            "1.0",
        ),
        (
            "log = []\ntry:\n    throw 1\ncatch:\n    return log\nfinally:\n    log.push(2)\nlog.push(3)",
            "[]",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally:\n    return 7",
            "7.0",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally:\n    return\nx = 9",
            "null",
        ),
        (
            "x = 0\ntry:\n    return x\ncatch:\n    pass\nfinally:\n    x = 5",
            "0.0",
        ),
        (
            "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\nfinally:\n    y = missing",
            "ReferenceError: missing is not defined (line 2, column 1)",
        ),
        (
            "x = 0\ntry:\n    throw \"a\"\ncatch:\n    throw \"b\"\nfinally:\n    x = 3",
            "b (line 2, column 1)",
        ),
    ] {
        assert_eq!(fresh(source), expected, "{source}");
    }
}

#[test]
fn failed_suites_roll_back_before_the_finally_suite_runs() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Tank(level):\n    this.level = level\ntank = Tank(80)\nreading = tank.level * 2\naudit = []\ntry:\n    tank.level = 120\n    audit.push(\"body\")\n    assert(reading, 240)\n    throw \"OVERFLOW\"\ncatch alarm:\n    assert([tank.level, reading, audit], [80, 160, []])\n    audit.push(alarm)\nfinally:\n    assert(audit, [\"OVERFLOW\"])\n    audit.push(\"closed\")\nassert(audit, [\"OVERFLOW\", \"closed\"])",
    );
    checked(
        &mut runtime,
        "def risky(t):\n    try:\n        t.level = 1\n        throw \"BODY\"\n    catch first:\n        t.level = 2\n        audit.push(first)\n        throw \"CATCH\"\n    finally:\n        assert([t.level, audit], [80, [\"OVERFLOW\", \"closed\"]])\n        return \"handled\"\nassert(risky(tank), \"handled\")\nassert([tank.level, reading], [80, 160])",
    );
    checked(
        &mut runtime,
        "def kept(t):\n    try:\n        t.level = 50\n        return \"kept\"\n    catch:\n        return \"lost\"\n    finally:\n        assert([t.level, reading], [50, 100])\nassert(kept(tank), \"kept\")\nassert([tank.level, reading], [50, 100])",
    );
}

#[test]
fn an_error_that_leaves_the_statement_rolls_back_its_finally_suite() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "log = []\ntry:\n    try:\n        throw \"a\"\n    catch e:\n        throw e + \"!\"\n    finally:\n        log.push(\"inner\")\n        assert(log, [\"inner\"])\ncatch outer:\n    assert(log, [])\n    log.push(outer)\nassert(log, [\"a!\"])",
    );
    checked(
        &mut runtime,
        "try:\n    try:\n        x = 1\n    catch:\n        x = 2\n    finally:\n        log.push(\"late\")\n        throw \"late\"\ncatch e:\n    r = e\nassert([r, log], [\"late\", [\"a!\"]])",
    );
    let initial = "class Tag(name):\n    this.name = name\ncount = 0\nlabels = Tag.map(tag -> tag.name)\nassert(labels, [])";
    let mut actual = Runtime::new();
    checked(&mut actual, initial);
    let mut control = Runtime::new();
    checked(&mut control, initial);
    for (source, thrown) in [
        (
            "try:\n    Tag(\"a\")\n    count = 1\ncatch:\n    pass\nfinally:\n    Tag(\"b\")\n    count = 2\n    throw \"STOP\"",
            Some("STOP"),
        ),
        (
            "try:\n    throw \"a\"\ncatch:\n    Tag(\"c\")\n    throw \"b\"\nfinally:\n    Tag(\"d\")\n    count = 3",
            Some("b"),
        ),
        (
            "try:\n    Tag(\"e\")\ncatch:\n    pass\nfinally:\n    count = 4\n    missing.value = 1",
            None,
        ),
        (
            "Tag(\"f\")\ntry:\n    count = 5\n    return 1\ncatch:\n    pass\nfinally:\n    throw \"OVERRIDE\"",
            Some("OVERRIDE"),
        ),
    ] {
        let error = actual.run(source).unwrap_err();
        assert_eq!(
            error.thrown_value(),
            thrown
                .map(|value| Value::String(value.to_string()))
                .as_ref(),
            "{source}"
        );
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    }
    checked(
        &mut actual,
        "try:\n    Tag(\"g\")\ncatch:\n    pass\nfinally:\n    Tag(\"h\")\n    count = Tag.length\nassert([labels, count], [[\"g\", \"h\"], 2])",
    );
}

#[test]
fn a_pending_return_keeps_the_value_it_computed() {
    for (source, expected) in [
        (
            "n = 1\ndef f():\n    try:\n        return n\n    catch:\n        return -1\n    finally:\n        n = 5\n[f(), n]",
            "[1.0,5.0]",
        ),
        (
            "def f(order):\n    try:\n        order.state = \"paid\"\n        return order.state\n    catch:\n        return \"failed\"\n    finally:\n        order.state = \"closed\"\ncart = {state: \"open\"}\n[f(cart), cart.state]",
            "[\"paid\",\"closed\"]",
        ),
        (
            "items = [1]\ndef f():\n    try:\n        return items.length\n    catch:\n        return 0\n    finally:\n        items.push(2)\n[f(), items]",
            "[1.0,[1.0,2.0]]",
        ),
        (
            "def f(k):\n    try:\n        throw k\n    catch e:\n        return e * 10\n    finally:\n        k = 0\n[f(2), f(3)]",
            "[20.0,30.0]",
        ),
    ] {
        assert_eq!(fresh(source), expected, "{source}");
    }
}

#[test]
fn a_return_in_the_finally_suite_replaces_the_pending_outcome() {
    for (source, expected) in [
        (
            "def f():\n    try:\n        return 1\n    catch:\n        return 2\n    finally:\n        return 3\nf()",
            "3.0",
        ),
        (
            "def f():\n    try:\n        throw \"a\"\n    catch e:\n        return e\n    finally:\n        return \"finally\"\nf()",
            "\"finally\"",
        ),
        (
            "def f():\n    try:\n        throw \"a\"\n    catch e:\n        throw \"b\"\n    finally:\n        return \"kept\"\nf()",
            "\"kept\"",
        ),
        (
            "count = 0\ndef f():\n    try:\n        throw \"a\"\n    catch e:\n        count = count + 1\n        throw \"b\"\n    finally:\n        return count\n[f(), count]",
            "[0.0,0.0]",
        ),
        (
            "def f():\n    try:\n        x = 1\n    catch:\n        x = 2\n    finally:\n        return \"early\"\n    return \"late\"\nf()",
            "\"early\"",
        ),
        (
            "def f():\n    try:\n        return 1\n    catch:\n        return 2\n    finally:\n        throw \"over\"\nr = \"\"\ntry:\n    r = f()\ncatch e:\n    r = e\nr",
            "\"over\"",
        ),
    ] {
        assert_eq!(fresh(source), expected, "{source}");
    }
}

#[test]
fn the_finally_suite_runs_in_the_scope_of_the_try_statement() {
    for (source, expected) in [
        (
            "try:\n    throw \"x\"\ncatch problem:\n    pass\nfinally:\n    seen = problem",
            "ReferenceError: problem is not defined (line 1, column 1)",
        ),
        (
            "def f():\n    try:\n        throw \"x\"\n    catch problem:\n        pass\n    finally:\n        return problem\nf()",
            "ReferenceError: problem is not defined (line 8, column 1)",
        ),
        (
            "problem = \"outer\"\ntry:\n    throw \"x\"\ncatch problem:\n    inner = problem\nfinally:\n    seen = problem\n[inner, seen]",
            "[\"x\",\"outer\"]",
        ),
        (
            "def f(k):\n    try:\n        throw k + 1\n    catch e:\n        pass\n    finally:\n        return k * 10\nf(4)",
            "40.0",
        ),
        (
            "def f(k):\n    try:\n        k = k + 1\n    catch e:\n        pass\n    finally:\n        k = k * 2\n    return k\nf(4)",
            "10.0",
        ),
        (
            "class Meter(v):\n    this.v = v\n    def read(scale):\n        try:\n            throw \"x\"\n        catch e:\n            return 0\n        finally:\n            return this.v * scale\nMeter(3).read(2)",
            "6.0",
        ),
    ] {
        assert_eq!(fresh(source), expected, "{source}");
    }
}

#[test]
fn nested_try_statements_run_each_finally_suite() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "log = []\ntry:\n    throw \"a\"\ncatch e:\n    log.push(e)\nfinally:\n    try:\n        throw \"b\"\n    catch f:\n        log.push(f)\n    finally:\n        log.push(\"inner\")\nassert(log, [\"a\", \"b\", \"inner\"])",
    );
    checked(
        &mut runtime,
        "steps = []\ntry:\n    try:\n        steps.push(1)\n    catch:\n        steps.push(2)\n    finally:\n        steps.push(3)\n    throw \"out\"\ncatch:\n    try:\n        steps.push(4)\n    catch:\n        steps.push(5)\n    finally:\n        steps.push(6)\nfinally:\n    steps.push(7)\nassert(steps, [4, 6, 7])",
    );
    checked(
        &mut runtime,
        "def depth(n):\n    try:\n        if n == 0:\n            throw \"bottom\"\n        return depth(n - 1)\n    catch e:\n        return e\n    finally:\n        trail.push(n)\ntrail = []\nassert(depth(3), \"bottom\")\nassert(trail, [0, 1, 2, 3])",
    );
}

#[test]
fn statements_run_their_finally_suites_in_every_context() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Job(v):\n    this.v = v\nJob(1)\nJob(2)\nJob(3)\nlog = []\nfor job of Job:\n    try:\n        if job.v == 2:\n            throw job.v\n        log.push(job.v)\n    catch e:\n        log.push(-e)\n    finally:\n        log.push(0)\nassert(log, [1, 0, -2, 0, 3, 0])",
    );
    checked(
        &mut runtime,
        "n = 0\nhits = []\nwhile n < 2:\n    try:\n        n = n + 1\n    catch:\n        pass\n    finally:\n        hits.push(n)\nrepeat 2:\n    try: hits.push(\"r\")\n    catch: pass\n    finally: hits.push(\"f\")\nassert(hits, [1, 2, \"r\", \"f\", \"r\", \"f\"])",
    );
    checked(
        &mut runtime,
        "class Door(name):\n    this.name = name\n    this.opened = []\n    try:\n        throw \"new\"\n    catch e:\n        this.opened.push(e)\n    finally:\n        this.opened.push(\"ready\")\n    def open(code):\n        try:\n            if code != 7:\n                throw \"wrong\"\n            return \"open\"\n        catch reason:\n            return reason\n        finally:\n            this.opened.push(code)\nfront = Door(\"front\")\nassert([front.open(1), front.open(7)], [\"wrong\", \"open\"])\nassert(front.opened, [\"new\", \"ready\", 1, 7])",
    );
    checked(
        &mut runtime,
        "seen = []\nmark = (v) => {\n    try:\n        if v < 0:\n            throw \"negative\"\n        return v\n    catch e:\n        return e\n    finally:\n        seen.push(v)\n}\nassert([mark(2), mark(-1)], [2, \"negative\"])\nassert(seen, [2, -1])",
    );
    checked(
        &mut runtime,
        "score = 1\ndef chance():\n    try:\n        score = 5\n        return score\n    catch:\n        return 0\n    finally:\n        score = score + 1\npicked = suppose:\n    chance()\nassert([picked, score], [5, 1])",
    );
}

#[test]
fn standing_rules_run_the_finally_suite_whenever_they_run_it() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "trigger = 0\nif trigger > 0:\n    try:\n        x = trigger\n    catch:\n        x = 0\n    finally:\n        seen = trigger * 10\ntrigger = 1\ntrigger = 2\nassert([x, seen], [2, 20])",
    );
    checked(
        &mut runtime,
        "log = []\ndef compute(v):\n    try:\n        return v * 2\n    catch:\n        return 0\n    finally:\n        log.push(v)\nbase = 1\ntotal = compute(base)\nbase = 4\nassert([total, log], [8, [1, 4]])",
    );
    checked(
        &mut runtime,
        "a = 1\ntry:\n    throw \"x\"\ncatch:\n    y = a * 2\nfinally:\n    z = a * 3\na = 5\nassert([y, z], [10, 15])",
    );
}

#[test]
fn finally_declarations_are_explained_as_catch_declarations() {
    let caught = "a = 1\ntry:\n    throw \"x\"\ncatch:\n    z = a * 3";
    let finished = "a = 1\ntry:\n    throw \"x\"\ncatch:\n    pass\nfinally:\n    z = a * 3";
    let mut left = Runtime::new();
    let mut right = Runtime::new();
    left.run(caught).unwrap();
    right.run(finished).unwrap();
    for query in [
        "z",
        "why z",
        "affects a",
        "(why z)[0].from",
        "(affects a)[0]",
    ] {
        assert_eq!(
            outcome(&mut left, query),
            outcome(&mut right, query),
            "{query}"
        );
    }
    assert_eq!(outcome(&mut right, "(why z)[0].from"), "[\"a\"]");
    let key = NodeKey::variable("z");
    assert_eq!(
        left.graph.retrieve(&key).unwrap().dependencies,
        right.graph.retrieve(&key).unwrap().dependencies
    );
    left.run("a = 2").unwrap();
    right.run("a = 2").unwrap();
    assert_eq!(outcome(&mut right, "z"), "6.0");
    assert_eq!(outcome(&mut left, "z"), outcome(&mut right, "z"));
}

#[test]
fn graph_keys_and_renderings_show_the_clause() {
    let with = parse("try: x = 1\ncatch: x = 2\nfinally: x = 3").unwrap();
    let without = parse("try: x = 1\ncatch: x = 2").unwrap();
    assert_eq!(generate_all(&with), "try");
    assert_eq!(generate(&with[0]), "try");
    assert_eq!(generate_all(&without), "try");
    let block =
        parse("{\n    try: x = 1\n    catch: x = 2\n    finally: x = 3\n    y = 4\n}").unwrap();
    assert_eq!(generate_all(&block), "{try;y=4}");
    let rendered = parse(
        "def f(k = 1):\n    try:\n        x = k\n    catch problem:\n        x = 2\n    finally:\n        x = 3",
    )
    .unwrap();
    let source = generate_all(&rendered);
    assert_eq!(
        source,
        "def f(k=1):\n    try:\n        x=k\n    catch problem:\n        x=2\n    finally:\n        x=3"
    );
    assert_eq!(parse(&source).unwrap(), rendered);
    let plain =
        parse("def f(k = 1):\n    try:\n        x = k\n    catch problem:\n        x = 2").unwrap();
    assert_eq!(
        generate_all(&plain),
        "def f(k=1):\n    try:\n        x=k\n    catch problem:\n        x=2"
    );
    let nested = parse(
        "def f(k = 1):\n    try:\n        try:\n            x = k\n        catch:\n            pass\n        finally:\n            x = 0\n    catch:\n        pass\n    finally: pass",
    )
    .unwrap();
    let source = generate_all(&nested);
    assert_eq!(parse(&source).unwrap(), nested, "{source}");
    let mut runtime = Runtime::new();
    runtime
        .run("def f(k = 1):\n    try:\n        return k\n    catch:\n        return 0\n    finally:\n        k = 2")
        .unwrap();
    assert_eq!(outcome(&mut runtime, "f()"), "1.0");
}

#[test]
fn the_public_try_node_keeps_its_constructor_and_takes_the_suite() {
    let node = Try::new(vec![Stmt::Pass], "error".to_string(), vec![Stmt::Pass]);
    assert_eq!(node.finally, None);
    let with = Try {
        finally: Some(vec![Stmt::Pass]),
        ..Try::new(vec![Stmt::Pass], "error".to_string(), vec![Stmt::Pass])
    };
    assert_eq!(with.finally.as_deref(), Some(&[Stmt::Pass][..]));
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    for (source, finally) in [
        ("try: pass\ncatch: pass\nfinally: x = 1", Some("x = 1")),
        ("try: pass\ncatch: pass", None),
    ] {
        let statements = parse(source).unwrap();
        let converted = Nuc::convert(&mut runtime, &mut scope, &statements[0]).unwrap();
        let Nuc::Try(node) = converted else {
            panic!("{source} converts to a try node");
        };
        assert_eq!(node.body, [Stmt::Pass]);
        assert_eq!(node.parameter, "error");
        assert_eq!(node.catch, [Stmt::Pass]);
        assert_eq!(
            node.finally,
            finally.map(|suite| parse(suite).unwrap()),
            "{source}"
        );
    }
}

#[test]
fn malformed_clauses_report_exact_errors() {
    let head = "try:\n    x = 1\ncatch:\n    x = 2\n";
    for (tail, message, position) in [
        ("finally:", "Unexpected end of input", Position::new(5, 9)),
        (
            "finally:\n",
            "Expected an indented block but found end of input",
            Position::new(6, 1),
        ),
        (
            "finally:\nx = 3\n",
            "Expected an indented block but found identifier 'x'",
            Position::new(6, 1),
        ),
        (
            "finally:\n    # nothing\n",
            "Expected an indented block but found end of input",
            Position::new(7, 1),
        ),
        (
            "finally:\n    # nothing",
            "Expected an indented block but found end of input",
            Position::new(6, 14),
        ),
        ("finally:;", "Unexpected ';'", Position::new(5, 10)),
        ("finally:: pass", "Unexpected ':'", Position::new(5, 11)),
        ("finally :: pass", "Unexpected ':'", Position::new(5, 12)),
        (
            "finally: {\n    x = 3\n",
            "Unterminated block",
            Position::new(7, 1),
        ),
        (
            "finally: pass\nfinally: pass\n",
            "Unexpected ':'",
            Position::new(6, 10),
        ),
        (
            "finally: x = (1 +)\n",
            "Unexpected ')'",
            Position::new(5, 18),
        ),
        (
            "finally:\n    x = (1 +)\n",
            "Unexpected ')'",
            Position::new(6, 13),
        ),
    ] {
        let source = format!("{head}{tail}");
        assert_eq!(
            syntax_error(&source),
            (message.to_string(), position),
            "{source:?}"
        );
    }
}

#[test]
fn runtime_errors_keep_the_position_of_the_running_statement() {
    let head = "x = 0\ntry:\n    x = 1\ncatch:\n    x = 2\n";
    for (tail, kind, message) in [
        (
            "finally: y = missing",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        ("finally: throw \"STOP\"", ErrorKind::Thrown, "STOP"),
        ("finally: x()", ErrorKind::Type, "x is not a function"),
    ] {
        let source = format!("{head}{tail}");
        assert_eq!(
            failure(&source),
            (kind, message.to_string(), Some(Position::new(2, 1))),
            "{source}"
        );
    }
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-finally-clauses-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_finally_clauses() {
    let program = "a = 1\nlog = []\ntry:\n    throw \"x\"\ncatch problem:\n    log.push(problem)\nfinally:\n    total = a * 3\n    log.push(\"done\")\na = 2\n[total, log]";
    let file = SourceFile::new("file");
    for (options, expected) in [
        (&[][..], "6,x,done\n"),
        (&["--json"][..], "[6.0,[\"x\",\"done\"]]\n"),
    ] {
        let output = file.run(program, options);
        assert!(output.status.success(), "{options:?} {output:?}");
        assert!(output.stderr.is_empty(), "{options:?} {output:?}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
    let graph = String::from_utf8(file.run(program, &["--graph"]).stdout).unwrap();
    assert!(
        graph.contains("total [variable]\n    reads    a"),
        "{graph}"
    );
    let failed = file.run(
        "try:\n    x = 1\ncatch:\n    x = 2\nfinally:\n    y = missing\n",
        &[],
    );
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "ReferenceError: missing is not defined (line 1, column 1)\n"
    );
    let kept = file.run(
        "try:\n    x = 1\ncatch:\n    x = 2\nfinally: x\n",
        &["--json"],
    );
    assert_eq!(
        String::from_utf8(kept.stderr).unwrap(),
        "ReferenceError: Missing definition (line 5, column 1)\n"
    );
    let session = "log = []\ntry:\n    log.push(1)\ncatch:\n    log.push(2)\nfinally:\n    log.push(3)\n\nlog\ntry:\n    log.push(4)\ncatch:\n    log.push(5)\n\nfinally:\n    log.push(6)\n\nlog\n.exit\n";
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{options:?} {output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "[]\n1.0\n[1.0,3.0]\n3.0\n[1.0,3.0,4.0]\n"
            } else {
                "\n1\n1,3\n3\n1,3,4\n"
            };
            assert!(stdout.ends_with(expected), "{options:?} {stdout:?}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                "SyntaxError: Unexpected ':' (line 1, column 8)\n",
                "{options:?}"
            );
        }
    }
}
