mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime, Value};

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid pads strings at the start with padStart",
    "Nucleoid pads strings at the end with padEnd",
    "Nucleoid keeps padded strings live in declarations and class level rules",
    "Nucleoid rolls back failed updates through padded strings",
];
const REFERENCE_TITLE: &str = "Nucleoid pads strings with padStart and padEnd";
const README_TITLE: &str = "Padding keeps a price list aligned as it changes";
const EXAMPLES_TITLE: &str = "Padded labels follow every bin's aisle and slot";

const TOO_LARGE: &str = "TypeError: Padded string is too large (line 1, column 1)";

type Rewrite = (&'static str, &'static str);

struct Counterpart {
    title: &'static str,
    rewrites: &'static [Rewrite],
    before: usize,
    after: usize,
}

const SPELLED: [Counterpart; 7] = [
    Counterpart {
        title: "Nucleoid pads strings at the start with padStart",
        rewrites: &[
            (
                "ids = [\"7\", \"42\", \"123\", \"1234\"]",
                "def pad(n, w = 3) => \"0\".repeat(w - String(n).length) + n\nids = [\"7\", \"42\", \"123\", \"1234\"]",
            ),
            (
                "codes = ids.map(id -> id.padStart(3, \"0\"))",
                "codes = ids.map(pad)",
            ),
            (
                "clock = String(h).padStart(2, \"0\") + \":\" + String(m).padStart(2, \"0\")",
                "clock = pad(h, 2) + \":\" + pad(m, 2)",
            ),
        ],
        before: 124,
        after: 117,
    },
    Counterpart {
        title: "Nucleoid pads strings at the end with padEnd",
        rewrites: &[
            (
                "item.padEnd(width)",
                "item + \" \".repeat(width - item.length)",
            ),
            (
                "title.padEnd(12, \".:\")",
                "title + \".:\".repeat(12 - title.length).slice(0, 12 - title.length)",
            ),
        ],
        before: 110,
        after: 89,
    },
    Counterpart {
        title: "Nucleoid keeps padded strings live in declarations and class level rules",
        rewrites: &[
            (
                "label.padStart(width, fill)",
                "fill.repeat(width - label.length) + label",
            ),
            (
                "String($Ticket.id).padStart(4, \"0\")",
                "\"0\".repeat(4 - String($Ticket.id).length) + $Ticket.id",
            ),
        ],
        before: 167,
        after: 157,
    },
    Counterpart {
        title: "Nucleoid rolls back failed updates through padded strings",
        rewrites: &[(
            "id.padStart(size, \"0\")",
            "\"0\".repeat(size - id.length) + id",
        )],
        before: 154,
        after: 150,
    },
    Counterpart {
        title: "Nucleoid pads strings with padStart and padEnd",
        rewrites: &[
            ("id.padStart(3, \"0\")", "\"0\".repeat(3 - id.length) + id"),
            ("id.padEnd(4, \".\")", "id + \".\".repeat(4 - id.length)"),
        ],
        before: 100,
        after: 92,
    },
    Counterpart {
        title: "Padding keeps a price list aligned as it changes",
        rewrites: &[
            (
                "item.padEnd(8, \".\")",
                "item + \".\".repeat(8 - item.length)",
            ),
            (
                "price.padStart(3)",
                "\" \".repeat(3 - price.length) + price",
            ),
        ],
        before: 57,
        after: 47,
    },
    Counterpart {
        title: "Padded labels follow every bin's aisle and slot",
        rewrites: &[
            (
                "$Bin.aisle.padEnd(3, \"-\")",
                "$Bin.aisle + \"-\".repeat(3 - $Bin.aisle.length)",
            ),
            (
                "String($Bin.slot).padStart(3, \"0\")",
                "\"0\".repeat(3 - String($Bin.slot).length) + $Bin.slot",
            ),
        ],
        before: 122,
        after: 110,
    },
];

const LENGTHS: [(&str, &str); 33] = [
    ("", "3"),
    ("", "3.9"),
    ("", "\"3\""),
    ("", "\" 3 \""),
    ("", "[3]"),
    ("", "true"),
    ("", "1"),
    ("", "\"1\""),
    ("", "0"),
    ("", "-0"),
    ("", "0.5"),
    ("", "-1"),
    ("", "-2.5"),
    ("", "null"),
    ("", "undefined"),
    ("", "false"),
    ("", "\"\""),
    ("", "[]"),
    ("", "Number.NEGATIVE_INFINITY"),
    ("", "Number.NaN"),
    ("", "0 / 0"),
    ("", "\"a\""),
    ("", "\"0x10\""),
    ("", "[1, 2]"),
    ("", "{}"),
    ("class P\np = P()", "p"),
    ("", "x -> x"),
    ("", "/a/"),
    ("o = {}", "o.missing"),
    ("d = Date(5)", "d"),
    ("", "4"),
    ("", "10"),
    ("", "\"10\""),
];

const HUGE_LENGTHS: [&str; 6] = [
    "Number.POSITIVE_INFINITY",
    "1 / 0",
    "\"Infinity\"",
    "\"inf\"",
    "1e300",
    "1e15",
];

const FILLS: [&str; 16] = [
    "\"0\"",
    "\"ab\"",
    "\"123456\"",
    "0",
    "1.5",
    "-0",
    "1e21",
    "null",
    "undefined",
    "true",
    "false",
    "[1, 2]",
    "[\"a\", [\"b\"]]",
    "x -> x",
    "/a/",
    "\"日本\"",
];

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

fn calls_padding(source: &str) -> bool {
    tokenize(source).unwrap().windows(3).any(|window| {
        window[0].token == Token::Dot
            && matches!(&window[1].token, Token::Identifier(name) if name == "padStart" || name == "padEnd")
            && window[2].token == Token::LeftParen
    })
}

fn tokens(source: &str) -> usize {
    tokenize(source.trim())
        .unwrap()
        .iter()
        .filter(|spanned| spanned.token != Token::Eof)
        .count()
}

fn renumbered(text: &str) -> String {
    let mut seen: Vec<&str> = Vec::new();
    let mut result = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("obj:") {
        let digits = after
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(after.len());
        let (identifier, tail) = after.split_at(digits);
        let ordinal = match seen.iter().position(|known| *known == identifier) {
            Some(ordinal) => ordinal,
            None => {
                seen.push(identifier);
                seen.len() - 1
            }
        };
        result.push_str(before);
        result.push_str("obj:");
        result.push_str(&ordinal.to_string());
        rest = tail;
    }
    result.push_str(rest);
    result
}

fn json(runtime: &mut Runtime, source: &str) -> String {
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    renumbered(&runtime.serialize_json(&value).unwrap())
}

fn result(source: &str) -> String {
    json(&mut Runtime::new(), source)
}

fn outcome(runtime: &mut Runtime, source: &str) -> String {
    match runtime.run(source) {
        Ok(value) => renumbered(&runtime.serialize_json(&value).unwrap()),
        Err(error) => error.to_string(),
    }
}

fn fresh(source: &str) -> String {
    outcome(&mut Runtime::new(), source)
}

fn failure(source: &str) -> String {
    let error = Runtime::new()
        .run(source)
        .expect_err("the program raises an error");
    error.to_string()
}

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "string padding regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"))
}

fn dependencies(runtime: &Runtime, name: &str) -> Vec<String> {
    runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap()
        .dependencies
        .iter()
        .map(|key| renumbered(&key.to_string()))
        .collect()
}

fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names
}

fn behaves_as_spelled(steps: &[(&str, &str)], names: &[&str]) {
    let mut padded = Runtime::new();
    let mut spelled = Runtime::new();
    for (step, spelling) in steps {
        assert_eq!(
            outcome(&mut padded, step),
            outcome(&mut spelled, spelling),
            "{step}"
        );
        for name in names {
            if padded.graph.retrieve(&NodeKey::variable(*name)).is_some() {
                assert_eq!(
                    sorted(dependencies(&padded, name)),
                    sorted(dependencies(&spelled, name)),
                    "{step}: {name}"
                );
            }
        }
    }
    assert_eq!(padded.assertions_run(), spelled.assertions_run());
    assert!(padded.take_assertions().is_empty(), "{steps:?}");
    assert!(spelled.take_assertions().is_empty(), "{steps:?}");
}

#[test]
fn only_the_appended_cases_call_padding_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(calls_padding(&case.source), new, "{}", case.title);
        if new {
            let count =
                common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            common::check_with_assertions(case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assertions += count;
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
    assert_eq!(assertions, 23);
    assert_eq!(cases, 1276);
}

#[test]
fn the_audit_finds_every_padding_call_and_nothing_else() {
    for source in [
        "s.padStart(3)",
        "s.padEnd(3, \"0\")",
        "\"7\".padStart(3, \"0\")",
        "`${n}`.padEnd(2)",
        "f(x).padStart(1)",
        "xs.map(s -> s.padEnd(4))",
        "$T.code = $T.id.padStart(4, \"0\")",
    ] {
        assert!(calls_padding(source), "{source}");
    }
    for source in [
        "s.padStart",
        "padStart(s, 3)",
        "o = {padStart: 1, padEnd: 2}",
        "\"padStart\"",
        "s.padLeft(3)",
        "s.repeat(3)",
        "def padEnd(s) => s",
        "s.padStart = 1",
    ] {
        assert!(!calls_padding(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_110_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[764],
        "Nucleoid rolls back failed updates through Python string methods"
    );
    assert_eq!(titles[765..769], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid spells startswith, endswith and join as Python does",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Python string methods classify files and list them",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Python string methods keep an order desk's rules live",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(index, cases.len() - 24, "{title}");
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 19].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Reactive string padding\n\n```\n")
        .expect("the language reference documents string padding");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(case.expected.as_deref(), Some("\"123.|\""));
    for line in [
        "[NUC 111](nuc-0111.md)",
        "`padStart` and `padEnd` ([NUC 111](nuc-0111.md))",
        "| Padded string output of `padStart` or `padEnd` is too large to represent or allocate | `Padded string is too large` |",
        "text.padStart(n, fill)  text.padEnd(n)  live text padded before or after to n characters; fill defaults to \" \" (NUC 111)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
}

#[test]
fn every_case_is_its_frozen_counterpart_with_only_the_padding_written_out() {
    let documents = documents();
    for Counterpart {
        title,
        rewrites,
        before,
        after,
    } in SPELLED
    {
        let case = documents
            .iter()
            .flat_map(|(_, cases)| cases)
            .find(|case| case.title == title)
            .unwrap();
        let mut counterpart = case.source.clone();
        for (padded, spelled) in rewrites {
            assert_eq!(counterpart.matches(padded).count(), 1, "{title}: {padded}");
            counterpart = counterpart.replacen(padded, spelled, 1);
        }
        assert!(calls_padding(&case.source), "{title}");
        assert!(!calls_padding(&counterpart), "{title}");
        assert_eq!(tokens(&case.source), after, "{title}");
        assert_eq!(tokens(&counterpart), before, "{title}");
        let assertions = common::assertions::protected_assertions(&parse(&case.source).unwrap());
        assert_eq!(
            assertions,
            common::assertions::protected_assertions(&parse(&counterpart).unwrap()),
            "{title}"
        );
        for source in [&case.source, &counterpart] {
            common::check_with_assertions(
                &common::Case {
                    title: title.to_string(),
                    source: source.clone(),
                    expected: case.expected.clone(),
                },
                assertions.len(),
            )
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        }
    }
}

#[test]
fn lengths_convert_as_the_count_of_repeat() {
    let mut padded = 0;
    for (setup, length) in LENGTHS {
        let setup = if setup.is_empty() {
            String::new()
        } else {
            format!("{setup}\n")
        };
        let found = fresh(&format!(
            "{setup}[\"ab\".padStart({length}, \"-\"), \"ab\".padEnd({length}, \"-\")]"
        ));
        let spelled = fresh(&format!(
            "{setup}n = \"-\".repeat({length}).length\n[\"-\".repeat(n - 2) + \"ab\", \"ab\" + \"-\".repeat(n - 2)]"
        ));
        assert_eq!(found, spelled, "{length}");
        if found != "[\"ab\",\"ab\"]" {
            padded += 1;
        }
    }
    assert_eq!(padded, 9);
    for (source, expected) in [
        ("\"ab\".padStart()", "\"ab\""),
        ("\"ab\".padEnd()", "\"ab\""),
        ("\"ab\".padStart(4.9, \"-\")", "\"--ab\""),
        ("\"ab\".padStart(\"4\", \"-\")", "\"--ab\""),
        ("\"ab\".padEnd(true, \"-\")", "\"ab\""),
        ("\"ab\".padEnd(5, \"-\")", "\"ab---\""),
        ("\"ab\".padStart(null, \"-\")", "\"ab\""),
        ("\"ab\".padStart(-3, \"-\")", "\"ab\""),
        ("\"\".padStart(0, \"x\")", "\"\""),
        ("\"\".padStart(1, \"x\")", "\"x\""),
        ("\"abc\".padStart(3, \"0\")", "\"abc\""),
        ("\"abc\".padStart(4, \"0\")", "\"0abc\""),
        ("\"abc\".padEnd(2, \"0\")", "\"abc\""),
        ("d = Date(5)\n\"7\".padStart(d, \"0\")", "\"00007\""),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for length in HUGE_LENGTHS {
        for name in ["padStart", "padEnd"] {
            assert_eq!(
                failure(&format!("\"ab\".{name}({length}, \"-\")")),
                TOO_LARGE,
                "{name} {length}"
            );
            assert_eq!(
                result(&format!("\"ab\".{name}({length}, \"\")")),
                "\"ab\"",
                "{name} {length}"
            );
        }
    }
}

#[test]
fn fills_convert_to_their_string_forms() {
    for fill in FILLS {
        let found = fresh(&format!(
            "[\"5\".padStart(4, {fill}), \"5\".padEnd(4, {fill})]"
        ));
        let spelled = fresh(&format!(
            "f = String({fill}).repeat(3).slice(0, 3)\n[f + \"5\", \"5\" + f]"
        ));
        assert_eq!(found, spelled, "{fill}");
    }
    for (source, expected) in [
        (
            "[\"5\".padStart(4), \"5\".padEnd(4)]",
            "[\"   5\",\"5   \"]",
        ),
        (
            "o = {}\n[\"5\".padStart(4, o.missing), \"5\".padEnd(4, o.missing)]",
            "[\"   5\",\"5   \"]",
        ),
        (
            "o = {}\nshown = \"5\".padStart(4, o.missing)\nshown",
            "\"   5\"",
        ),
        (
            "[\"5\".padStart(4, \" \"), \"5\".padEnd(4, \" \")]",
            "[\"   5\",\"5   \"]",
        ),
        (
            "[\"5\".padStart(4, \"0\"), \"5\".padEnd(4, \"0\")]",
            "[\"0005\",\"5000\"]",
        ),
        (
            "[\"5\".padStart(4, \"ab\"), \"5\".padEnd(4, \"ab\")]",
            "[\"aba5\",\"5aba\"]",
        ),
        (
            "[\"5\".padStart(4, \"123456\"), \"5\".padEnd(4, \"123456\")]",
            "[\"1235\",\"5123\"]",
        ),
        (
            "[\"5\".padStart(4, \"\"), \"5\".padEnd(4, \"\")]",
            "[\"5\",\"5\"]",
        ),
        (
            "[\"5\".padStart(4, 0), \"5\".padEnd(4, 1.5)]",
            "[\"0005\",\"51.5\"]",
        ),
        (
            "[\"5\".padStart(4, null), \"5\".padEnd(4, null)]",
            "[\"nul5\",\"5nul\"]",
        ),
        (
            "[\"5\".padStart(4, undefined), \"5\".padEnd(4, undefined)]",
            "[\"nul5\",\"5nul\"]",
        ),
        ("x = undefined\n\"5\".padStart(4, x)", "\"nul5\""),
        (
            "[\"5\".padStart(4, true), \"5\".padEnd(4, false)]",
            "[\"tru5\",\"5fal\"]",
        ),
        (
            "[\"5\".padStart(4, [1, 2]), \"5\".padEnd(4, [])]",
            "[\"1,25\",\"5\"]",
        ),
        (
            "[\"5\".padStart(4, {}), \"5\".padEnd(4, x -> x)]",
            "[\"[ob5\",\"5[fu\"]",
        ),
        (
            "[\"5\".padStart(4, /a/), \"5\".padEnd(6, \"日本\")]",
            "[\"/a/5\",\"5日本日本日\"]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn lengths_and_cuts_count_unicode_scalar_values() {
    for (source, expected) in [
        (
            "[\"日本\".padStart(4, \"-\"), \"日本\".padEnd(4, \"-\"), \"日本\".length]",
            "[\"--日本\",\"日本--\",2.0]",
        ),
        ("\"😀\".padStart(3)", "\"  😀\""),
        ("\"😀\".padEnd(3, \"ab\")", "\"😀ab\""),
        ("\"x\".padEnd(4, \"日本\")", "\"x日本日\""),
        ("\"é\".padStart(3, \"é\")", "\"ééé\""),
        ("\"日\".padStart(3, \"本\")", "\"本本日\""),
        ("\"é\".padEnd(4, \"·\") == \"é\" + \"·\".repeat(3)", "true"),
        (
            "e = String.fromCharCode(101, 769)\n[e.length, e.padStart(3, \"-\").length, e.padStart(3, \"-\").slice(0, 1)]",
            "[2.0,3.0,\"-\"]",
        ),
        (
            "[\"x\".padStart(4, \"😀é\"), \"x\".padEnd(4, \"😀é\")]",
            "[\"😀é😀x\",\"x😀é😀\"]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn oversized_results_raise_padded_string_is_too_large() {
    for source in [
        "\"x\".padStart(1 / 0)",
        "\"x\".padEnd(Number.POSITIVE_INFINITY, \"ab\")",
        "\"日\".padStart(1 / 0)",
        "\"x\".padStart(1 / 0, \"日\")",
        "\"x\".padEnd(1e15)",
        "\"x\".padStart(1e15, \"ab\")",
    ] {
        assert_eq!(failure(source), TOO_LARGE, "{source}");
    }
    let error = Runtime::new().run("\"x\".padEnd(1 / 0)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Padded string is too large");
    assert_eq!(result("\"x\".padStart(1 / 0, \"\")"), "\"x\"");
    assert_eq!(result("\"x\".padStart(1000000).length"), "1000000.0");
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "width = 3\nshown = \"7\".padStart(width, \"0\")\nfailure = \"\"\ntry:\n    width = 1 / 0\ncatch error:\n    failure = error\nassert(failure, TypeError(\"Padded string is too large\"))\nassert([width, shown], [3, \"007\"])",
    );
    runtime.run("width = 4").unwrap();
    assert_eq!(json(&mut runtime, "shown"), "\"0007\"");
}

#[test]
fn unchanged_routes_keep_their_accepted_results() {
    for (source, expected) in [
        (
            "o = {padStart: (n, f) -> \"own\", padEnd: (n, f) -> \"own end\"}\n[o.padStart(3, \"0\"), o.padEnd(3)]",
            "[\"own\",\"own end\"]",
        ),
        ("o = {}\n[o.padStart(3, \"0\"), o.padEnd(3)]", "[null,null]"),
        (
            "n = null\n[n.padStart(3, \"0\"), n.padEnd(3)]",
            "[null,null]",
        ),
        (
            "calls = []\nn = null\nn.padStart(calls.push(1))\nn.padEnd(calls.push(2))\ncalls",
            "[]",
        ),
        ("n = null\nn.padStart(missing, boom())", "null"),
        (
            "o = {}\n[o.missing.padStart(3, \"0\"), o.missing.padEnd(3)]",
            "[null,null]",
        ),
        ("s = \"5\"\n[s.padStart, s.padEnd]", "[null,null]"),
        ("\"5\".padStart", "null"),
        (
            "def padStart(s, n) => s\ndef padEnd(s, n) => n\n[padStart(\"5\", 3), padEnd(\"5\", 3)]",
            "[\"5\",3.0]",
        ),
        (
            "class P:\n    def padEnd(n) => \"method\"\n    def padStart(n, f) => \"q\" + n\np = P()\n[p.padEnd(3), p.padStart(3, \"0\"), P().padEnd(3)]",
            "[\"method\",\"q3\",\"method\"]",
        ),
        (
            "class P\np = P()\n[p.padStart(3), p.padEnd(3)]",
            "[null,null]",
        ),
        (
            "[\"ab\".repeat(2), \"abc\".slice(1), \"a b\".split(\" \"), \"ab\".replaceAll(\"a\", \"c\")]",
            "[\"abab\",\"bc\",[\"a\",\"b\"],\"cb\"]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "x = 5\nx.padStart(3, \"0\")",
            "TypeError: number.padStart is not a function (line 2, column 1)",
        ),
        (
            "(5).padEnd(3)",
            "TypeError: number.padEnd is not a function (line 1, column 1)",
        ),
        (
            "[\"a\", \"b\"].padStart(3)",
            "TypeError: [\"a\",\"b\"].padStart is not a function (line 1, column 1)",
        ),
        (
            "xs = [\"a\"]\nxs.padEnd(3)",
            "TypeError: xs.padEnd is not a function (line 2, column 1)",
        ),
        (
            "true.padStart(3)",
            "TypeError: boolean.padStart is not a function (line 1, column 1)",
        ),
        (
            "d = Date(5)\nd.padStart(3)",
            "TypeError: Date.padStart is not a function (line 2, column 1)",
        ),
        (
            "/a/.padStart(3)",
            "TypeError: padStart is not a function (line 1, column 1)",
        ),
        (
            "f = x -> x\nf.padEnd(3)",
            "TypeError: function.padEnd is not a function (line 2, column 1)",
        ),
        (
            "class Tag\nTag.padStart(3)",
            "TypeError: Tag.padStart is not a function (line 2, column 1)",
        ),
        (
            "class Tag(n):\n    this.n = n\nTag(1)\nTag.padEnd(3)",
            "TypeError: Tag.padEnd is not a function (line 4, column 1)",
        ),
        (
            "Math.padStart(3)",
            "TypeError: Math.padStart is not a function (line 1, column 1)",
        ),
        (
            "Number.padEnd(3)",
            "TypeError: Number.padEnd is not a function (line 1, column 1)",
        ),
        (
            "String.padStart(\"5\", 3)",
            "TypeError: String.padStart is not a function (line 1, column 1)",
        ),
        (
            "String.padEnd(\"5\", 3)",
            "TypeError: String.padEnd is not a function (line 1, column 1)",
        ),
        (
            "String.padStart(missing)",
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
        (
            "s = \"5\"\ns.padStart = 1",
            "TypeError: Cannot assign 'padStart' on string (line 2, column 1)",
        ),
        (
            "s = \"5\"\ns.padEnd = 1",
            "TypeError: Cannot assign 'padEnd' on string (line 2, column 1)",
        ),
        (
            "padStart(\"5\")",
            "ReferenceError: padStart is not defined (line 1, column 1)",
        ),
        (
            "padEnd(\"5\")",
            "ReferenceError: padEnd is not defined (line 1, column 1)",
        ),
        (
            "\"5\".padStart(missing, \"0\")",
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
        (
            "\"5\".padEnd(3, missing)",
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
    ] {
        assert_eq!(failure(source), expected, "{source}");
    }
}

#[test]
fn the_public_string_method_pads_and_keeps_every_other_name() {
    let pad = |receiver: &str, name: &str, arguments: &[Value]| {
        nucleoid::builtins::string_method(receiver, name, arguments)
    };
    for (receiver, name, arguments, expected) in [
        (
            "7",
            "padStart",
            vec![Value::number(3), Value::string("0")],
            "007",
        ),
        (
            "ab",
            "padEnd",
            vec![Value::number(4), Value::string(".")],
            "ab..",
        ),
        ("5", "padStart", vec![Value::number(3)], "  5"),
        (
            "5",
            "padStart",
            vec![Value::number(3), Value::Undefined],
            "  5",
        ),
        ("5", "padEnd", vec![Value::number(3), Value::Null], "5nu"),
        ("5", "padStart", Vec::new(), "5"),
        ("5", "padEnd", vec![Value::Undefined], "5"),
        (
            "5",
            "padStart",
            vec![Value::string("3"), Value::number(0), Value::Bool(true)],
            "005",
        ),
    ] {
        assert_eq!(
            pad(receiver, name, &arguments).unwrap(),
            Value::string(expected),
            "{receiver}.{name}"
        );
    }
    let error = pad("x", "padStart", &[Value::number(f64::INFINITY)]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Padded string is too large");
    for name in ["padLeft", "padRight", "pad", "PadStart"] {
        let error = pad("7", name, &[Value::number(3)]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), format!("7.{name} is not a function"));
    }
}

#[test]
fn declarations_follow_their_receiver_length_and_fill_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            ("label = \"7\"", "label = \"7\""),
            ("width = 3", "width = 3"),
            ("fill = \"0\"", "fill = \"0\""),
            (
                "shown = label.padStart(width, fill)",
                "shown = fill.repeat(width - label.length) + label",
            ),
            (
                "tail = label.padEnd(width, fill)",
                "tail = label + fill.repeat(width - label.length)",
            ),
            ("[shown, tail]", "[shown, tail]"),
            ("label = \"42\"", "label = \"42\""),
            ("[shown, tail]", "[shown, tail]"),
            ("width = 5", "width = 5"),
            ("[shown, tail]", "[shown, tail]"),
            ("fill = \"\"", "fill = \"\""),
            ("[shown, tail]", "[shown, tail]"),
            ("fill = \"*\"", "fill = \"*\""),
            ("label = \"123456\"", "label = \"123456\""),
            ("[shown, tail]", "[shown, tail]"),
            ("o = {code: \"9\"}", "o = {code: \"9\"}"),
            (
                "boxed = o.code.padStart(3, \"0\")",
                "boxed = \"0\".repeat(3 - o.code.length) + o.code",
            ),
            ("o.code = \"12\"", "o.code = \"12\""),
            ("boxed", "boxed"),
            ("ids = [\"7\"]", "ids = [\"7\"]"),
            (
                "codes = ids.map(id -> id.padStart(3, \"0\"))",
                "codes = ids.map(id -> \"0\".repeat(3 - id.length) + id)",
            ),
            ("ids.push(\"42\")", "ids.push(\"42\")"),
            ("codes", "codes"),
            ("ids.pop()", "ids.pop()"),
            (
                "ids = [\"1\", \"22\", \"333\"]",
                "ids = [\"1\", \"22\", \"333\"]",
            ),
            ("codes", "codes"),
        ],
        &["shown", "tail", "boxed", "codes"],
    );
    behaves_as_spelled(
        &[
            ("label = \"5\"", "label = \"5\""),
            ("fill = \"ab\"", "fill = \"ab\""),
            (
                "shown = label.padStart(6, fill)",
                "shown = fill.repeat(6 - label.length).slice(0, 6 - label.length) + label",
            ),
            (
                "tail = label.padEnd(6, fill)",
                "tail = label + fill.repeat(6 - label.length).slice(0, 6 - label.length)",
            ),
            ("[shown, tail]", "[shown, tail]"),
            ("fill = \"123\"", "fill = \"123\""),
            ("[shown, tail]", "[shown, tail]"),
            ("label = \"日本\"", "label = \"日本\""),
            ("[shown, tail]", "[shown, tail]"),
        ],
        &["shown", "tail"],
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "label = \"7\"\nwidth = 3\nfill = \"0\"\nshown = label.padStart(width, fill)\nassert(shown, \"007\")\nlabel = \"42\"\nwidth = 4\nfill = \"-\"\nassert(shown, \"--42\")",
    );
    assert_eq!(dependencies(&runtime, "shown"), ["label", "width", "fill"]);
}

#[test]
fn class_level_rules_pad_each_instance_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            (
                "class Bin(aisle, slot):\n    this.aisle = aisle\n    this.slot = slot",
                "class Bin(aisle, slot):\n    this.aisle = aisle\n    this.slot = slot",
            ),
            (
                "$Bin.label = $Bin.aisle.padEnd(3, \"-\") + String($Bin.slot).padStart(3, \"0\")",
                "$Bin.label = $Bin.aisle + \"-\".repeat(3 - $Bin.aisle.length) + \"0\".repeat(3 - String($Bin.slot).length) + String($Bin.slot)",
            ),
            ("north = Bin(\"N\", 7)", "north = Bin(\"N\", 7)"),
            ("east = Bin(\"E2\", 45)", "east = Bin(\"E2\", 45)"),
            ("[north.label, east.label]", "[north.label, east.label]"),
            ("north.slot = 12", "north.slot = 12"),
            ("east.aisle = \"EAST\"", "east.aisle = \"EAST\""),
            ("[north.label, east.label]", "[north.label, east.label]"),
            (
                "labels = Bin.map(bin -> bin.label.padEnd(8, \".\"))",
                "labels = Bin.map(bin -> bin.label + \".\".repeat(8 - bin.label.length))",
            ),
            ("Bin(\"W\", 3)", "Bin(\"W\", 3)"),
            ("labels", "labels"),
        ],
        &["labels"],
    );
    assert_eq!(
        result(
            "class Ticket(id):\n    this.id = id\n$Ticket.code = \"T-\" + String($Ticket.id).padStart(4, \"0\")\nt = Ticket(7)\nu = Ticket(42)\nt.id = 12345\n[t.code, u.code]"
        ),
        "[\"T-12345\",\"T-0042\"]"
    );
}

#[test]
fn explanations_name_the_receiver_length_and_fill_in_reading_order() {
    let setup = "label = \"7\"\nwidth = 3\nfill = \"0\"\nw = 3\n";
    let mut padded = Runtime::new();
    let mut spelled = Runtime::new();
    padded
        .run(&format!("{setup}shown = label.padStart(width, fill)\nplain = label.padStart(w, \"0\")\ntail = label.padEnd(4)"))
        .unwrap();
    spelled
        .run(&format!("{setup}shown = fill.repeat(width - label.length) + label\nplain = \"0\".repeat(w - label.length) + label\ntail = label + \" \".repeat(4 - label.length)"))
        .unwrap();
    assert_eq!(
        json(&mut padded, "(why shown)[0].rule"),
        "\"shown = label.padStart(width,fill)\""
    );
    assert_eq!(
        json(&mut padded, "(why plain)[0].rule"),
        "\"plain = label.padStart(w,\\\"0\\\")\""
    );
    assert_eq!(
        json(&mut padded, "(why tail)[0].rule"),
        "\"tail = label.padEnd(4)\""
    );
    assert_eq!(
        json(&mut padded, "(why shown)[0].from"),
        "[\"label\",\"width\",\"fill\"]"
    );
    assert_eq!(
        json(&mut spelled, "(why shown)[0].from"),
        "[\"fill\",\"width\",\"label\"]"
    );
    assert_eq!(
        json(&mut padded, "(why shown).map(step -> step.node)"),
        "[\"shown\",\"label\",\"width\",\"fill\"]"
    );
    assert_eq!(
        json(&mut spelled, "(why shown).map(step -> step.node)"),
        "[\"shown\",\"fill\",\"width\",\"label\"]"
    );
    assert_eq!(
        json(&mut padded, "(why plain)[0].from"),
        "[\"label\",\"w\"]"
    );
    assert_eq!(
        json(&mut spelled, "(why plain)[0].from"),
        "[\"w\",\"label\"]"
    );
    assert_eq!(dependencies(&padded, "shown"), ["label", "width", "fill"]);
    assert_eq!(dependencies(&spelled, "shown"), ["fill", "width", "label"]);
    for query in [
        "(why tail).map(step -> step.from)",
        "(affects label).map(step -> step.node).length",
        "(affects fill).map(step -> step.node)",
        "(affects width).map(step -> step.node)",
        "tail |> why |> derived |> depth(1)",
    ] {
        let found = json(&mut padded, query);
        let expected = json(&mut spelled, query).replace(
            "tail = label+\\\" \\\".repeat(4-label.length)",
            "tail = label.padEnd(4)",
        );
        assert_eq!(found, expected, "{query}");
    }
    assert_eq!(
        generate_all(&parse("t = s.padStart(3, \"0\")").unwrap()),
        "t=s.padStart(3,\"0\")"
    );
    assert_eq!(
        generate_all(&parse("t = s.padEnd(n)").unwrap()),
        "t=s.padEnd(n)"
    );
}

#[test]
fn the_receiver_and_then_every_argument_evaluate_once_in_written_order() {
    let functions = "log = []\ndef text():\n    log.push(\"text\")\n    return \"7\"\ndef size():\n    log.push(\"size\")\n    return 3\ndef fill():\n    log.push(\"fill\")\n    return \"0\"\n";
    for (source, expected) in [
        (
            "[text().padStart(size(), fill()), log]",
            "[\"007\",[\"text\",\"size\",\"fill\"]]",
        ),
        (
            "[text().padEnd(size(), fill(), log.push(\"extra\")), log]",
            "[\"700\",[\"text\",\"size\",\"fill\",\"extra\"]]",
        ),
        (
            "[\"abc\".padStart(size(), fill()), log]",
            "[\"abc\",[\"size\",\"fill\"]]",
        ),
        (
            "[text().padStart(size(), \"\"), log]",
            "[\"7\",[\"text\",\"size\"]]",
        ),
    ] {
        assert_eq!(
            result(&format!("{functions}{source}")),
            expected,
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    runtime.run(functions).unwrap();
    assert_eq!(
        runtime
            .run("text().padStart(size(), missing)")
            .unwrap_err()
            .to_string(),
        "ReferenceError: missing is not defined (line 1, column 1)"
    );
    assert_eq!(json(&mut runtime, "log"), "[]");
}

#[test]
fn null_bindings_defer_declarations_as_they_do_for_repeat() {
    for (setup, declaration, write, settled) in [
        (
            "fill = null",
            "shown = \"5\".padStart(3, fill)",
            "fill = \"0\"",
            "\"005\"",
        ),
        (
            "width = null",
            "shown = \"5\".padStart(width, \"0\")",
            "width = 3",
            "\"005\"",
        ),
        (
            "s = null",
            "shown = s.padStart(3, \"0\")",
            "s = \"7\"",
            "\"007\"",
        ),
        (
            "s = null",
            "shown = s.padEnd(3, \".\")",
            "s = \"ab\"",
            "\"ab.\"",
        ),
        ("n = null", "shown = \"a\".repeat(n)", "n = 2", "\"aa\""),
        (
            "fill = null",
            "shown = fill.repeat(3 - \"5\".length) + \"5\"",
            "fill = \"0\"",
            "\"005\"",
        ),
    ] {
        let mut runtime = Runtime::new();
        runtime.run(&format!("{setup}\n{declaration}")).unwrap();
        assert_eq!(json(&mut runtime, "shown"), "null", "{declaration}");
        runtime.run(write).unwrap();
        assert_eq!(json(&mut runtime, "shown"), settled, "{declaration}");
    }
    for (source, expected) in [
        ("shown = \"5\".padStart(3, null)\nshown", "\"nu5\""),
        ("shown = \"5\".padStart(3, undefined)\nshown", "\"nu5\""),
        ("fill = null\n\"5\".padStart(3, fill)", "\"nu5\""),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn padding_reads_in_every_statement_position() {
    for (source, expected) in [
        (
            "result = \"\"\ntry:\n    result = (5).padStart(3)\ncatch error:\n    result = error\nresult",
            "\"TypeError: number.padStart is not a function\"",
        ),
        (
            "code = \"7\"\nshown = code.padStart(3, \"0\")\nseen = suppose code = \"42\": shown\n[seen, shown, code]",
            "[\"042\",\"007\",\"7\"]",
        ),
        ("n = 7\n`#${String(n).padStart(3, \"0\")}`", "\"#007\""),
        (
            "code = \"7\"\nif code.padStart(3, \"0\") == \"007\":\n    kind = \"padded\"\nelse:\n    kind = \"plain\"\nkind",
            "\"padded\"",
        ),
        (
            "code = \"1234\"\ncode.padStart(3, \"0\") == code ? \"kept\" : \"cut\"",
            "\"kept\"",
        ),
        (
            "def label(n) => \"#\" + String(n).padStart(3, \"0\")\n[label(7), label(1234)]",
            "[\"#007\",\"#1234\"]",
        ),
        (
            "def row(s):\n    return s.padEnd(5, \".\") + \"|\"\nrow(\"ab\")",
            "\"ab...|\"",
        ),
        (
            "names = [\"a\", \"bbb\", \"cc\"]\nnames.filter(n -> n.padEnd(3).length == 3).map(n -> n.padStart(4, \"-\"))",
            "[\"---a\",\"-bbb\",\"--cc\"]",
        ),
        (
            "codes = [\"7\", \"42\"]\ncodes.some(c -> c.padStart(2, \"0\") == \"07\") && codes.every(c -> c.padEnd(2, \"0\").length == 2)",
            "true",
        ),
        (
            "i = 0\nwhile String(i).padStart(3, \"0\") != \"003\":\n    i = i + 1\ni",
            "3.0",
        ),
        (
            "class T(id):\n    this.id = id\nif $T.id.padStart(2, \"0\") == \"07\":\n    $T.lucky = true\nt = T(\"7\")\nu = T(\"8\")\n[t.lucky, u.lucky]",
            "[true,null]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn failed_statements_and_transactions_restore_padded_declarations() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Ticket(id):\n    this.id = id\n$Ticket.code = $Ticket.id.padStart(4, \"0\")\nt = Ticket(\"7\")\nfailure = \"\"\ntry:\n    t.id = 7\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.padStart is not a function\"))\nassert([t.id, t.code], [\"7\", \"0007\"])",
    );
    checked(
        &mut runtime,
        "width = 3\nshown = \"7\".padStart(width, \"0\")\ntry:\n    width = 1 / 0\ncatch error:\n    failure = error\nassert(failure, TypeError(\"Padded string is too large\"))\nassert([width, shown], [3, \"007\"])",
    );
    checked(
        &mut runtime,
        "try:\n    width = 5\n    t.id = \"12\"\n    assert([shown, t.code], [\"00007\", \"0012\"])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([shown, t.code, width, t.id], [\"007\", \"0007\", 3, \"7\"])",
    );
    assert_eq!(dependencies(&runtime, "shown"), ["width"]);
    assert_eq!(
        failure("width = 3\nshown = \"7\".padStart(width, \"0\")\nwidth = 1 / 0"),
        "TypeError: Padded string is too large (line 3, column 1)"
    );
    let mut recovered = Runtime::new();
    recovered
        .run("ids = []\ncodes = ids.map(id -> id.padStart(size, \"0\"))")
        .unwrap();
    assert!(recovered.run("ids.push(\"7\")").is_err());
    assert_eq!(json(&mut recovered, "[ids, codes]"), "[[],[]]");
    recovered.run("size = 2\nids.push(\"7\")").unwrap();
    assert_eq!(json(&mut recovered, "codes"), "[\"07\"]");
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    assert_eq!(
        result("try:\n    r = \"5\".padStart(3, \"0\")\ncatch e:\n    r = \"caught\"\nr"),
        "\"005\""
    );
    assert_eq!(
        result("r = 0\ntry:\n    r = \"x\".padStart(1 / 0)\ncatch error:\n    r = error\nr"),
        "\"TypeError: Padded string is too large\""
    );
}

#[test]
fn large_pads_and_long_fills_build_their_exact_results() {
    let mut runtime = Runtime::new();
    runtime
        .run("long = \"x\".padStart(1000000, \"ab\")\nfill = \"0123456789\".repeat(100)\nwide = \"x\".padEnd(2500, fill)")
        .unwrap();
    assert_eq!(
        json(
            &mut runtime,
            "[long.length, long.slice(0, 4), long.slice(-3), long == \"ab\".repeat(500000).slice(0, 999999) + \"x\"]"
        ),
        "[1000000.0,\"abab\",\"bax\",true]"
    );
    assert_eq!(
        json(
            &mut runtime,
            "[wide.length, wide.slice(1, 4), wide.slice(-1), wide == \"x\" + fill.repeat(3).slice(0, 2499)]"
        ),
        "[2500.0,\"012\",\"8\",true]"
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-string-padding-{label}-{}.nuc",
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

fn spelled_out(source: &str) -> String {
    source
        .replace("id.padStart(3, \"0\")", "\"0\".repeat(3 - id.length) + id")
        .replace(
            "item.padEnd(6, \".\")",
            "item + \".\".repeat(6 - item.length)",
        )
        .replace("\"ab\".padEnd(4, \".\")", "\"ab\" + \".\".repeat(2)")
}

#[test]
fn files_graphs_and_repl_sessions_pad() {
    let program = "ids = [\"7\", \"42\"]\ncodes = ids.map(id -> id.padStart(3, \"0\"))\nitem = \"tea\"\nline = item.padEnd(6, \".\")\nids.push(\"5\")\n[codes, line]";
    let spelled = spelled_out(program);
    assert!(!calls_padding(&spelled));
    let file = SourceFile::new("file");
    for options in [
        &[][..],
        &["--json"][..],
        &["--graph"][..],
        &["--json", "--graph"][..],
    ] {
        let padded = file.run(program, options);
        let written = file.run(&spelled, options);
        assert!(padded.status.success(), "{options:?} {padded:?}");
        assert!(padded.stderr.is_empty(), "{options:?} {padded:?}");
        assert_eq!(padded.stdout, written.stdout, "{options:?}");
    }
    assert_eq!(
        String::from_utf8(file.run(program, &["--json"]).stdout).unwrap(),
        "[[\"007\",\"042\",\"005\"],\"tea...\"]\n"
    );
    let graph = String::from_utf8(
        file.run(
            "label = \"7\"\nwidth = 3\nfill = \"0\"\nshown = label.padStart(width, fill)\nshown",
            &["--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        graph.contains("shown [variable]\n    reads    label, width, fill"),
        "{graph}"
    );
    for (source, message) in [
        (
            "xs = [1]\n(5).padStart(3)\n",
            "TypeError: number.padStart is not a function (line 2, column 1)\n",
        ),
        (
            "\"x\".padStart(1 / 0)\n",
            "TypeError: Padded string is too large (line 1, column 1)\n",
        ),
    ] {
        let failed = file.run(source, &["--json"]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(String::from_utf8(failed.stderr).unwrap(), message);
    }
    let session = "ids = [\"7\"]\ncodes = ids.map(id -> id.padStart(3, \"0\"))\ncodes\nids.push(\"42\")\ncodes\n\"ab\".padEnd(4, \".\")\n.exit\n";
    let spelled_session = spelled_out(session);
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let padded = repl(&session.replace('\n', newline), options);
            let written = repl(&spelled_session.replace('\n', newline), options);
            assert!(padded.status.success(), "{options:?} {padded:?}");
            assert_eq!(padded.stdout, written.stdout, "{options:?}");
            assert_eq!(padded.stderr, written.stderr, "{options:?}");
            let stdout = String::from_utf8(padded.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "[\"007\"]\n2.0\n[\"007\",\"042\"]\n\"ab..\"\n"
            } else {
                "007\n2\n007,042\nab..\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
        }
    }
    let failing = repl(
        "\"x\".padStart(1 / 0)\n\"x\".padStart(3, \"-\")\n.exit\n",
        &["--plain"],
    );
    assert!(failing.status.success(), "{failing:?}");
    assert_eq!(
        String::from_utf8(failing.stderr).unwrap(),
        "TypeError: Padded string is too large (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(failing.stdout).unwrap();
    assert!(stdout.contains("--x\n"), "{stdout}");
}
