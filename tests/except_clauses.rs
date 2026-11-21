#[expect(dead_code)]
#[path = "../build.rs"]
mod build_exporter;

mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Stmt, parse, parse_program};
use nucleoid::{ErrorKind, Position, Runtime};

const SPEC_TITLES: [&str; 3] = [
    "Nucleoid catches every error with except clauses",
    "Nucleoid rolls back a failed update before its except clause runs",
    "Nucleoid reads except clauses in methods, functions and nested try statements",
];
const REFERENCE_TITLE: &str = "Nucleoid reads Python except clauses as catch clauses";
const README_TITLE: &str = "Except clauses fall back to another sensor when a reading fails";
const EXAMPLES_TITLE: &str =
    "Except clauses refuse an overdrawn transfer and keep both wallets whole";

const HEADERS: [(&str, &str); 6] = [
    ("except:", "catch:"),
    ("except Exception:", "catch:"),
    ("except Exception as problem:", "catch problem:"),
    ("except Exception as error:", "catch error:"),
    ("except  Exception  as  problem :", "catch problem:"),
    ("except\tException\tas\tproblem\t:", "catch problem:"),
];

const UNCAUGHT: &str = "SyntaxError: Expected 'catch' after 'try'";

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

fn offset(source: &str, position: Position) -> usize {
    let start: usize = source
        .split_inclusive('\n')
        .take(position.line - 1)
        .map(str::len)
        .sum();
    let line = &source[start..];
    start
        + line
            .char_indices()
            .nth(position.column - 1)
            .map_or(line.len(), |(index, _)| index)
}

fn word(token: Option<&Token>, text: &str) -> bool {
    matches!(token, Some(Token::Identifier(name)) if name == text)
}

fn respelled_headers(source: &str) -> Vec<(usize, usize, String)> {
    let Ok(spanned) = tokenize(source) else {
        return Vec::new();
    };
    let tokens: Vec<&Token> = spanned.iter().map(|spanned| &spanned.token).collect();
    let mut headers = Vec::new();
    for index in 0..tokens.len() {
        if !word(tokens.get(index).copied(), "except") {
            continue;
        }
        let at = |step: usize| tokens.get(index + step).copied();
        let (spelling, colon) = if !word(at(1), "Exception") {
            ("catch".to_string(), 1)
        } else if !word(at(2), "as") {
            ("catch".to_string(), 2)
        } else if let Some(Token::Identifier(name)) = at(3) {
            (format!("catch {name}"), 4)
        } else {
            continue;
        };
        if at(colon) == Some(&Token::Colon) {
            headers.push((
                offset(source, spanned[index].position),
                offset(source, spanned[index + colon].position),
                spelling,
            ));
        }
    }
    headers
}

fn except_clauses(source: &str) -> Vec<(usize, usize, String)> {
    let Ok(tree) = parse(source) else {
        return Vec::new();
    };
    respelled_headers(source)
        .into_iter()
        .filter(|(start, end, spelling)| {
            let respelled = format!("{}{spelling}{}", &source[..*start], &source[*end..]);
            parse(&respelled).is_ok_and(|other| other == tree)
        })
        .collect()
}

fn catch_spelling(source: &str) -> String {
    let mut spelled = source.to_string();
    for (start, end, spelling) in except_clauses(source).into_iter().rev() {
        spelled.replace_range(start..end, &spelling);
    }
    spelled
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

fn same_tree(source: &str, spelled: &str) -> Vec<Stmt> {
    let tree = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    let wanted = parse(spelled).unwrap_or_else(|error| panic!("{spelled}: {error}"));
    assert_eq!(tree, wanted, "{source}");
    assert_eq!(generate_all(&tree), generate_all(&wanted), "{source}");
    tree
}

fn behaves_as_spelled(steps: &[&str]) {
    let mut written = Runtime::new();
    let mut spelled = Runtime::new();
    let mut assertions = 0;
    for step in steps {
        let respelled = step
            .replace("except Exception as ", "catch ")
            .replace("except Exception:", "catch:")
            .replace("except:", "catch:");
        let tree = same_tree(step, &respelled);
        assertions += common::assertions::protected_assertions(&tree).len();
        assert_eq!(
            outcome(&mut written, step),
            outcome(&mut spelled, &respelled),
            "{step}"
        );
    }
    assert_eq!(written.assertions_run(), assertions, "{steps:?}");
    assert_eq!(written.assertions_run(), spelled.assertions_run());
    assert!(written.take_assertions().is_empty(), "{steps:?}");
    assert!(spelled.take_assertions().is_empty(), "{steps:?}");
}

#[test]
fn only_the_appended_cases_hold_except_clauses_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    let mut clauses = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let found = except_clauses(&case.source).len();
        let new = is_appended(&case.title);
        assert_eq!(found > 0, new, "{}", case.title);
        if new {
            clauses += found;
            let statements = parse(&case.source).unwrap();
            let count = common::assertions::protected_assertions(&statements).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            let mut runtime = Runtime::new();
            common::check_with_count(&mut runtime, case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assert_eq!(runtime.assertions_run(), count, "{}", case.title);
            assertions += count;
            let tree = format!("{statements:?}");
            for nested in [
                "TryFinally {",
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
    assert_eq!(clauses, 15);
    assert_eq!(assertions, 16);
    assert_eq!(cases, 1230);
}

#[test]
fn the_audit_finds_every_except_clause_and_nothing_else() {
    for (source, found) in [
        ("try:\n    pass\nexcept:\n    pass", 1),
        ("try: pass\nexcept Exception: pass", 1),
        ("try: pass except Exception as e: pass", 1),
        (
            "def f():\n    try:\n        return 1\n    except Exception as e:\n        return e",
            1,
        ),
        (
            "class C:\n    def m():\n        try: return 1\n        except: return 2",
            1,
        ),
        ("f = () => {\n    try: return 1\n    except: return 2\n}", 1),
        ("r = [1].map(k => { try: return k except: return 0 })", 1),
        (
            "try:\n    try: pass\n    except: pass\nexcept Exception as e:\n    pass",
            2,
        ),
        (
            "try:\n    pass\nexcept:\n    pass\nfinally:\n    pass\no = {except: 1}",
            1,
        ),
    ] {
        assert_eq!(except_clauses(source).len(), found, "{source}");
    }
    for source in [
        "try:\n    pass\ncatch:\n    pass",
        "except = 1",
        "except(1)",
        "o = {except: 1}\no.except",
        "o = {a: 1 except: 2}",
        "except: int",
        "f = lambda except: except",
        "x = c ? except : 1",
        "xs[except:]",
        "def except(v) => v",
        "a = 1\nb = a + 1\nc = (b |> why |> except(why a))",
        "# except:\nx = 1",
        "s = \"except: x\"",
        "try:\n    pass\ncatch except:\n    pass",
    ] {
        assert!(except_clauses(source).is_empty(), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_121_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[800],
        "Nucleoid rolls back failed updates through held values"
    );
    assert_eq!(titles[801..804], SPEC_TITLES);
    assert_eq!(titles[816], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid gives the values of selections with holds",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Holds lists the readings behind a frost warning and what the wind moves",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Holds shows the figures behind an invoice and the totals on the way",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(index, cases.len() - 7, "{title}");
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 4].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Python-style except clauses\n\n```\n")
        .expect("the language reference documents except clauses");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(case.expected.as_deref(), Some("0"));
    for line in [
        "[NUC 122](nuc-0122.md)",
        "except Exception as name:              Python's spelling of catch name: (NUC 122)",
        "except:                                Python's spelling of catch:, binding error (NUC 122)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    assert!(include_str!("../README.md").contains("[NUC 122](docs/nuc-0122.md)"));
    assert!(include_str!("../docs/examples.md").contains("[NUC 122](nuc-0122.md)"));
    assert!(include_str!("../nucleoid.spec.md").contains("\n## Python-style except clauses\n"));
}

#[test]
fn every_case_is_its_catch_spelling_with_only_the_headers_written_in_python() {
    let mut spellings = 0;
    for case in documents()
        .iter()
        .flat_map(|(_, cases)| cases)
        .filter(|case| is_appended(&case.title))
    {
        let spelled = catch_spelling(&case.source);
        assert_ne!(spelled, case.source, "{}", case.title);
        assert!(except_clauses(&spelled).is_empty(), "{}", case.title);
        same_tree(&case.source, &spelled);
        let mut written = Runtime::new();
        let mut accepted = Runtime::new();
        assert_eq!(
            outcome(&mut written, &case.source),
            outcome(&mut accepted, &spelled),
            "{}",
            case.title
        );
        assert_eq!(written.assertions_run(), accepted.assertions_run());
        assert!(written.take_assertions().is_empty(), "{}", case.title);
        assert!(accepted.take_assertions().is_empty(), "{}", case.title);
        spellings += 1;
    }
    assert_eq!(spellings, 6);
}

#[test]
fn every_document_case_exports_as_one_program_with_its_clauses() {
    let records: Vec<build_exporter::Record> = include_str!("../dataset/nucleoid.spec.jsonl")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .filter(|record| SPEC_TITLES.contains(&record["description"].as_str().unwrap()))
        .map(|record| serde_json::from_value(record).unwrap())
        .collect();
    assert_eq!(records.len(), 3);
    for record in &records {
        let rendered = build_exporter::render_records(std::slice::from_ref(record));
        assert_eq!(rendered.matches("    run(").count(), 1, "{rendered}");
        assert!(!rendered.contains("run_error("), "{rendered}");
        assert!(rendered.contains("except"), "{rendered}");
    }
    let committed = include_str!("nucleoid.spec.rs");
    for name in [
        "fn catches_every_error_with_except_clauses()",
        "fn rolls_back_a_failed_update_before_its_except_clause_runs()",
        "fn reads_except_clauses_in_methods_functions_and_nested_try_statements()",
    ] {
        assert!(committed.contains(name), "{name}");
    }
}

#[test]
fn every_header_builds_the_tree_of_its_catch_spelling() {
    let contexts = [
        "try:\n    x = missing\nHEADER\n    x = BINDER\nx",
        "try: x = missing\nHEADER x = BINDER; y = 2\n[x, y]",
        "try: x = missing HEADER x = BINDER\nx",
        "try: { x = missing }\nHEADER { x = BINDER }\nx",
        "try:\n    x = missing\n\n\n# between\nHEADER # after\n    x = BINDER\nx",
        "try\n    x = missing\nHEADER\n    x = BINDER\nx",
        "try:\n\tx = missing\nHEADER\n\tx = BINDER\nx",
        "def f(v):\n    try:\n        return v * missing\n    HEADER\n        return BINDER\nf(2)",
        "class C:\n    def m(v):\n        try:\n            return v * missing\n        HEADER\n            return BINDER\nC().m(2)",
        "class P(v):\n    try:\n        this.v = v * missing\n    HEADER\n        this.v = BINDER\nP(3).v",
        "f = () => {\n    try:\n        return missing\n    HEADER\n        return BINDER\n}\nf()",
        "r = [1].map(k => {\n    try:\n        return k * missing\n    HEADER\n        return BINDER\n})\nr",
        "if true:\n    try:\n        x = missing\n    HEADER\n        x = BINDER\nx",
        "try:\n    try:\n        x = missing\n    HEADER\n        throw BINDER\nHEADER\n    x = BINDER\nx",
        "try:\n    x = missing\nHEADER\n    x = BINDER\nfinally:\n    y = 1\n[x, y]",
        "x = 1\ntry:\n    x = 2\nHEADER\n    x = BINDER\nx",
    ];
    for (header, catch) in HEADERS {
        let binder = header
            .split_whitespace()
            .nth(3)
            .map_or("error", |name| name.trim_end_matches(':'));
        for context in contexts {
            let source = context.replace("HEADER", header).replace("BINDER", binder);
            let spelled = context.replace("HEADER", catch).replace("BINDER", binder);
            for (source, spelled) in [
                (source.clone(), spelled.clone()),
                (source.replace('\n', "\r\n"), spelled.replace('\n', "\r\n")),
            ] {
                same_tree(&source, &spelled);
                assert_eq!(catch_spelling(&source), spelled, "{source}");
                assert_eq!(fresh(&source), fresh(&spelled), "{source}");
            }
        }
    }
    let source = "try:\n    x = 1\nexcept:\n    x = 2\nfinally:\n    x = 3";
    assert!(matches!(
        parse(source).unwrap().as_slice(),
        [Stmt::TryFinally { parameter, .. }] if parameter == "error"
    ));
    let source = "try:\n    x = 1\nexcept Exception as problem:\n    x = 2";
    assert!(matches!(
        parse(source).unwrap().as_slice(),
        [Stmt::Try { parameter, .. }] if parameter == "problem"
    ));
    let positions = parse_program("x = 1\ntry: x = missing\nexcept: x = 2\nx")
        .unwrap()
        .positions;
    assert_eq!(
        positions,
        [
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(4, 1)
        ]
    );
}

#[test]
fn binders_name_any_word_and_exception_stays_a_fixed_word() {
    for binder in [
        "e",
        "e1",
        "_e",
        "error",
        "Exception",
        "as",
        "except",
        "finally",
        "catcher",
    ] {
        let source =
            format!("try:\n    throw 7\nexcept Exception as {binder}:\n    x = {binder}\nx");
        let spelled = format!("try:\n    throw 7\ncatch {binder}:\n    x = {binder}\nx");
        same_tree(&source, &spelled);
        assert_eq!(fresh(&source), "7.0", "{source}");
    }
    for source in [
        "Exception = 5\nas = 6\ntry:\n    x = missing\nexcept Exception:\n    x = [Exception, as]\nx",
        "class Exception(m):\n    this.m = m\ntry:\n    throw 3\nexcept Exception as e:\n    x = e\nx",
    ] {
        let spelled = catch_spelling(source);
        assert_ne!(spelled, source);
        same_tree(source, &spelled);
    }
    assert_eq!(
        fresh(
            "Exception = 5\nas = 6\ntry:\n    x = missing\nexcept Exception:\n    x = [Exception, as]\nx"
        ),
        "[5.0,6.0]"
    );
}

#[test]
fn other_headers_keep_the_accepted_error_and_position() {
    for header in [
        "except e:",
        "except ValueError:",
        "except BaseException:",
        "except exception:",
        "except Exceptions:",
        "except EXCEPTION:",
        "except (TypeError, ReferenceError):",
        "except (Exception):",
        "except Exception, e:",
        "except Exception as (e):",
        "except Exception as 5:",
        "except Exception as this:",
        "except Exception as self:",
        "except Exception as None:",
        "except Exception as \"e\":",
        "except Exception as e.f:",
        "except Exception as [e]:",
        "except Exception as e as f:",
        "except Exception as e, f:",
        "except Exception as:",
        "except as e:",
        "except as:",
        "except Exception e:",
        "except*:",
        "except* Exception:",
        "except Exception as e",
        "except Exception",
        "except",
        "except e",
        "except(1)",
        "except.x = 1",
        "except[0]",
        "except + 1",
        "Except:",
        "EXCEPT:",
        "except:= 1",
        "except Exception as e := 1",
        "except if x:",
        "except Exception as error.message:",
    ] {
        let source = format!("try:\n    x = missing\n{header}\n    x = 2\nx");
        assert_eq!(
            fresh(&source),
            format!("{UNCAUGHT} (line 3, column 1)"),
            "{source}"
        );
        assert!(except_clauses(&source).is_empty(), "{source}");
        let error = Runtime::check(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.position(), Some(Position::new(3, 1)), "{source}");
    }
    for (source, position) in [
        ("try:\n    x = missing\nexcept = 1", (3, 1)),
        (
            "def f():\n    try:\n        return missing\n    except e:\n        return 2\nf()",
            (4, 5),
        ),
        (
            "class C:\n    def m():\n        try:\n            return missing\n        except ValueError:\n            return 2\nC().m()",
            (5, 9),
        ),
        (
            "try:\n    try:\n        x = missing\n    except Exception as e as f:\n        x = 1\ncatch:\n    x = 2\nx",
            (4, 5),
        ),
        ("try\n    x = missing\nexcept e:\n    x = 2\nx", (3, 1)),
        ("try: x = missing except e: x = 2\nx", (1, 18)),
        ("try:\n    x = missing\n\nexcept (e):\n    x = 2\nx", (4, 1)),
        (
            "try:\n    x = missing\nexcept BaseException:\n    x = 1\nfinally:\n    x = 2\nx",
            (3, 1),
        ),
        ("try:\n    y = 1\nexcept e:\n    y = 2", (3, 1)),
        ("try:\n    x = 1\nexcept(1)", (3, 1)),
        ("xs = [1, 2]\ntry: a, b = xs except: a = 0\na", (2, 7)),
        ("try: x = 1; except: y = 2\nx", (1, 19)),
        (
            "class P\np = P()\ntry: $P is open except: x = 1\np.open",
            (3, 9),
        ),
    ] {
        assert_eq!(
            fresh(source),
            format!("{UNCAUGHT} (line {}, column {})", position.0, position.1),
            "{source}"
        );
    }
    assert_eq!(
        fresh("try: enum Color: RED, GREEN except: x = 1\nColor.RED"),
        "SyntaxError: Expected ',' but found identifier 'except' (line 1, column 29)"
    );
}

#[test]
fn a_try_statement_takes_one_clause() {
    for (header, second, expected) in [
        (
            "except:",
            "except:",
            "SyntaxError: Unexpected ':' (line 5, column 7)",
        ),
        (
            "except:",
            "catch:",
            "SyntaxError: Unexpected keyword 'catch' (line 5, column 6)",
        ),
        (
            "except:",
            "else:",
            "SyntaxError: Unexpected keyword 'else' (line 5, column 5)",
        ),
        (
            "except:",
            "except e:",
            "SyntaxError: Unexpected ':' (line 5, column 9)",
        ),
        (
            "except Exception as e:",
            "except Exception as f:",
            "SyntaxError: Unexpected ':' (line 5, column 22)",
        ),
        (
            "except:",
            "finally",
            "ReferenceError: finally is not defined (line 5, column 1)",
        ),
        (
            "catch:",
            "except Exception as e:",
            "SyntaxError: Unexpected ':' (line 5, column 22)",
        ),
        (
            "catch:",
            "except:",
            "SyntaxError: Unexpected ':' (line 5, column 7)",
        ),
    ] {
        let source = format!("try:\n    x = missing\n{header}\n    x = 1\n{second}\n    x = 2\nx");
        assert_eq!(fresh(&source), expected, "{source}");
        let catch = header
            .replace("except Exception as", "catch")
            .replace("except", "catch");
        let spelled = source.replacen(header, &catch, 1);
        assert_eq!(fresh(&spelled), expected, "{spelled}");
        let parsed = parse(&format!("try:\n    x = missing\n{header}\n    x = 1"));
        let accepted = parse(&format!("try:\n    x = missing\n{catch}\n    x = 1"));
        assert_eq!(parsed.ok(), accepted.ok(), "{header}");
    }
}

#[test]
fn except_stays_an_ordinary_name() {
    for (source, expected) in [
        ("except = 3\nexcept", "3.0"),
        ("o = {except: 1}\no.except", "1.0"),
        ("s = {except: () => 7}\ns.except()", "7.0"),
        (
            "a = 1\nb = a + 1\nc = b + 1\n(c |> why |> except(why b)).length",
            "1.0",
        ),
        ("class Sets:\n    def except() => 7\nSets().except()", "7.0"),
        ("def except(v) => v * 2\nexcept(4)", "8.0"),
        ("def twice(except) => except * 2\ntwice(3)", "6.0"),
        ("f = lambda except: except + 1\nf(1)", "2.0"),
        (
            "try:\n    x = 1\ncatch:\n    x = 2\nexcept = 5\n[x, except]",
            "[1.0,5.0]",
        ),
        (
            "try:\n    x = missing\ncatch except:\n    x = except\nx",
            "\"ReferenceError: missing is not defined\"",
        ),
        (
            "except: int",
            "ReferenceError: Missing definition (line 1, column 1)",
        ),
        (
            "except:\n    x = 1\nx",
            "SyntaxError: Unexpected ':' (line 1, column 7)",
        ),
        (
            "x = 1\nexcept:\n    x = 2",
            "SyntaxError: Unexpected ':' (line 2, column 7)",
        ),
        ("except = 5\nc = true\nx = c ? except : 1\nx", "5.0"),
        ("xs = [1, 2, 3]\nexcept = 1\nxs[except:]", "[2.0,3.0]"),
        ("o = {a: 2 except: 3}\no.except", "3.0"),
        ("Exception = 5\nas = 6\n[Exception, as]", "[5.0,6.0]"),
        (
            "class Exception(msg):\n    this.msg = msg\ne = Exception(\"x\")\ne.msg",
            "\"x\"",
        ),
        ("except := 5\nexcept", "5.0"),
        ("enum except: A, B\nexcept.A", "\"A\""),
        (
            "class I(n):\n    this.n = n\nI(4)\ntotal = []\nfor except of I:\n    total.push(except.n)\ntotal",
            "[4.0]",
        ),
        ("except = 1\nexcept += 2\nexcept", "3.0"),
        (
            "def except(v) => v + 1\ntry:\n    x = 1\ncatch:\n    x = 2\nexcept(3)",
            "4.0",
        ),
    ] {
        assert_eq!(fresh(source), expected, "{source}");
        assert!(except_clauses(source).is_empty(), "{source}");
    }
}

#[test]
fn clauses_roll_back_bind_and_scope_as_their_catch_spellings() {
    behaves_as_spelled(&[
        "balance = 10\nlimit = balance * 2",
        "try:\n    balance = 20\n    assert(limit, 40)\n    throw \"fail\"\nexcept:\n    assert([balance, limit, error], [10, 20, \"fail\"])\n    note = \"rolled back\"",
        "[balance, limit, note]",
        "e = 1",
        "try:\n    x = missing\nexcept Exception as e:\n    x = e\n[x, e]",
        "try:\n    seen = error\nexcept Exception:\n    seen = error\nseen",
        "try:\n    throw [1, {k: 2}]\nexcept Exception as thrown:\n    kept = thrown\nkept",
        "try:\n    throw 5\nexcept Exception as n:\n    kind = typeof n\nkind",
        "try:\n    throw TypeError(\"bad\")\nexcept Exception as t:\n    message = t\nmessage",
        "try:\n    v = (5).calibrate(2)\nexcept:\n    v = error\nv",
        "try:\n    x = missing\nexcept:\n    x = alsoMissing\nx",
        "items = [1, 2]",
        "try:\n    items.push(3)\n    throw \"stop\"\nexcept:\n    items.push(9)\nitems",
        "a = 1",
        "try:\n    b = a * 2\n    throw \"x\"\nexcept:\n    c = a * 3",
        "a = 5",
        "[c, typeof b]",
        "try:\n    d = a * 2\nexcept:\n    d = 0",
        "a = 7",
        "d",
    ]);
}

#[test]
fn returns_loops_nesting_and_finally_run_as_their_catch_spellings() {
    behaves_as_spelled(&[
        "def safe(v):\n    try:\n        return v.total * missing\n    except Exception as e:\n        return e\nsafe({total: 2})",
        "def guard(v):\n    try:\n        if v > 1:\n            throw \"BIG\"\n        return v\n    except:\n        return -1\n[guard(1), guard(5)]",
        "class Account:\n    def withdraw(amount):\n        try:\n            return amount / missing\n        except:\n            return 0\nAccount().withdraw(5)",
        "class I(n):\n    this.n = n\nI(1)\nI(2)\nlog = []",
        "for i of I:\n    try:\n        throw i.n\n    except Exception as k:\n        log.push(k)\nlog",
        "def outer(v):\n    try:\n        try:\n            throw v\n        except Exception as inner:\n            throw inner * 10\n    except Exception as outside:\n        return outside + 1\nouter(3)",
        "x = 0\ntry:\n    x = missing\nexcept:\n    x = 2\nfinally:\n    x = x + 10\nx",
        "def f():\n    try:\n        return missing\n    except:\n        return 2\n    finally:\n        log.push(\"f\")\n[f(), log]",
        "def g():\n    try:\n        throw 1\n    except Exception as e:\n        throw e + 1\n    finally:\n        return 9\ng()",
        "flag = true\nif flag:\n    try:\n        y = missing\n    except:\n        y = 5\ny",
        "a = 1\nr = suppose a = 2:\n    try:\n        z = a * missing\n    except:\n        z = a\n    z\n[r, a]",
        "f = () => {\n    try:\n        return missing\n    except:\n        return 3\n}\nf()",
        "class P(v):\n    try:\n        this.v = v * missing\n    except Exception as e:\n        this.v = v\np = P(4)\np.v",
    ]);
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-except-clauses-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_except_clauses_as_catch_clauses() {
    let program = "a = 1\nlog = []\ntry:\n    throw \"x\"\nexcept Exception as problem:\n    log.push(problem)\n    total = a * 3\na = 2\n[total, log]";
    let spelled = catch_spelling(program);
    assert_ne!(spelled, program);
    let file = SourceFile::new("file");
    for options in [&[][..], &["--json"][..], &["--graph"][..]] {
        for newline in ["\n", "\r\n"] {
            let written = file.run(&program.replace('\n', newline), options);
            let accepted = file.run(&spelled.replace('\n', newline), options);
            assert!(written.status.success(), "{options:?} {written:?}");
            assert_eq!(written.stdout, accepted.stdout, "{options:?}");
            assert_eq!(written.stderr, accepted.stderr, "{options:?}");
        }
    }
    let output = file.run(program, &["--json"]);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "[6.0,[\"x\"]]\n");
    let failed = file.run("try:\n    x = missing\nexcept:\n    y = alsoMissing\n", &[]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "ReferenceError: alsoMissing is not defined (line 1, column 1)\n"
    );
    let kept = file.run("try:\n    y = 1\nexcept e:\n    y = 2\n", &[]);
    assert_eq!(
        String::from_utf8(kept.stderr).unwrap(),
        format!("{UNCAUGHT} (line 3, column 1)\n")
    );
    let session = "log = []\ntry:\n    log.push(1)\n    throw 2\nexcept Exception as e:\n    log.push(e)\n\nlog\ntry:\n    log.push(3)\n\nexcept:\n    log.push(4)\n\nlog\ntry:\n    y = 1\nexcept e:\n    y = 2\n\ny = 5\ny\n.exit\n";
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let written = repl(&session.replace('\n', newline), options);
            let accepted = repl(
                &session
                    .replace("except Exception as e:", "catch e:")
                    .replace('\n', newline),
                options,
            );
            assert!(written.status.success(), "{options:?} {written:?}");
            assert_eq!(written.stderr, accepted.stderr, "{options:?}");
            let stdout = String::from_utf8(written.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "[]\n1.0\n[2.0]\n[2.0]\n5.0\n5.0\n"
            } else {
                "\n1\n2\n2\n5\n5\n"
            };
            assert!(stdout.ends_with(expected), "{options:?} {stdout:?}");
            assert_eq!(
                String::from_utf8(written.stderr).unwrap(),
                format!(
                    "{UNCAUGHT} (line 3, column 1)\nSyntaxError: Unexpected ':' (line 1, column 7)\n{UNCAUGHT} (line 3, column 1)\n"
                ),
                "{options:?}"
            );
            assert_eq!(String::from_utf8(accepted.stdout).unwrap(), stdout);
        }
    }
}
