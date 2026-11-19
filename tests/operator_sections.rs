mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::thread;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{parse, parse_expression};
use nucleoid::{ErrorKind, Runtime};
use serde::Deserialize;

const SPEC_TITLES: [&str; 6] = [
    "Nucleoid passes comparison sections as one-parameter functions",
    "Nucleoid reads the rest of a section argument with the usual precedence",
    "Nucleoid fills and combines values with nullish and logical sections",
    "Nucleoid keeps section operands live like lambda bodies",
    "Nucleoid passes sections to functions and constructors beside other arguments",
    "Nucleoid rolls back failed updates through sections",
];
const REFERENCE_TITLE: &str = "Nucleoid writes one-parameter functions as operator sections";
const README_TITLE: &str = "Operator sections pass a comparison without naming a parameter";
const EXAMPLES_TITLE: &str = "Operator sections keep every sensor's alarm count live";

const OPERATORS: [&str; 14] = [
    "==", "!=", "===", "!==", "<", "<=", ">", ">=", "*", "%", "**", "&&", "||", "??",
];

const RESTS: [&str; 27] = [
    "1",
    "a",
    "a + 1",
    "a - 1",
    "a * 2",
    "a / 2",
    "a % 2",
    "a ** 2",
    "-a",
    "!a",
    "not a",
    "typeof a",
    "a == 1",
    "a < 1",
    "a in b",
    "a && b",
    "a || b",
    "a ?? b",
    "a ? b : c",
    "a if b else c",
    "f(a)",
    "[a, b]",
    "{k: a}",
    "a.b[0]",
    "x -> x + a",
    "a |> why",
    "a = 1",
];

const QUERIES: [&str; 21] = [
    "filter",
    "map",
    "count",
    "some",
    "every",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "partition",
    "sortBy",
    "maxBy",
    "minBy",
    "sum",
    "max",
    "min",
    "average",
    "median",
    "distinct",
    "mode",
    "flatMap",
];

const OPERANDS: [&str; 10] = [
    "1",
    "limit",
    "limit + 1",
    "ys.length",
    "ys[0] * 2",
    "limit > 2 ? 3 : 4",
    "-1",
    "null",
    "\"a\"",
    "ys.count(y -> y > 1)",
];

const LARGE_STACK: usize = 16 * 1024 * 1024;

const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid passes comparison sections as one-parameter functions",
    "source": "# Nucleoid passes comparison sections as one-parameter functions\n\nreadings = [12, 31, 7, 45, 18]\nhigh = readings.filter(x -> x > 20)\nlow = readings.count(x -> x <= 12)\npeak = readings.find(x -> x >= 40)\nspot = readings.findIndex(x -> x < 10)\nalarm = readings.max() > 50\nsane = !readings.includes(0)\nassert([high, low, peak, spot, alarm, sane], [[31, 45], 2, 45, 2, false, true])\nreadings.push(60, 0)\nassert([high, low, peak, spot, alarm, sane], [[31, 45, 60], 3, 45, 2, true, false])\nreadings.shift()\nassert([low, spot], [2, 1])\nhigh.length",
    "expected": "3"
  },
  {
    "document": "spec",
    "title": "Nucleoid reads the rest of a section argument with the usual precedence",
    "source": "# Nucleoid reads the rest of a section argument with the usual precedence\n\nvalues = [3, 8, 1, 9]\nlow = 2\ndoubled = [x * 2 + 1 for x in values]\nsquares = [x ** 2 - 1 for x in values]\nevens = values.count(x -> x % 2 == 0)\nrests = [x % 3 for x in values]\nsizes = [x > 4 ? \"big\" : \"small\" for x in values]\nabove = values.filter(x -> x > low + 1)\nassert([doubled, squares], [[7, 17, 3, 19], [8, 63, 0, 80]])\nassert([evens, rests], [1, [0, 2, 1, 0]])\nassert([sizes, above], [[\"small\", \"big\", \"small\", \"big\"], [8, 9]])\nlow = 0\nvalues.push(4)\nassert([above, evens, rests], [[3, 8, 9, 4], 2, [0, 2, 1, 0, 1]])\nsizes",
    "expected": "[\"small\",\"big\",\"small\",\"big\",\"small\"]"
  },
  {
    "document": "spec",
    "title": "Nucleoid fills and combines values with nullish and logical sections",
    "source": "# Nucleoid fills and combines values with nullish and logical sections\n\nscores = [7, null, 3, null]\nfilled = [x ?? 0 for x in scores]\nnames = [\"ada\", \"\", \"bo\"]\nshown = [x || \"anonymous\" for x in names]\nflags = [true, false, true]\nlabels = [x && \"on\" for x in flags]\nexact = flags.count(x -> x === true)\nassert(filled, [7, 0, 3, 0])\nassert([shown, labels, exact], [[\"ada\", \"anonymous\", \"bo\"], [\"on\", false, \"on\"], 2])\nscores.push(null, 5)\nnames.push(\"\")\nflags.push(1)\nassert(filled, [7, 0, 3, 0, 0, 5])\nassert(shown, [\"ada\", \"anonymous\", \"bo\", \"anonymous\"])\n[labels, exact]",
    "expected": "[[\"on\",false,\"on\",\"on\"],2]"
  },
  {
    "document": "spec",
    "title": "Nucleoid keeps section operands live like lambda bodies",
    "source": "# Nucleoid keeps section operands live like lambda bodies\n\nlimit = 20\nreadings = [12, 31, 7]\nover = readings.filter(x -> x > limit)\nalerts = over.count(x -> x >= limit * 2)\nassert([over, alerts], [[31], 0])\nlimit = 10\nassert([over, alerts], [[12, 31], 1])\nreadings.push(45)\nassert([over, alerts], [[12, 31, 45], 2])\nreadings = [5, 25]\nassert([over, alerts], [[25], 1])\nalerts",
    "expected": "1"
  },
  {
    "document": "spec",
    "title": "Nucleoid passes sections to functions and constructors beside other arguments",
    "source": "# Nucleoid passes sections to functions and constructors beside other arguments\n\ndef keep(values, test, change) => values.filter(test).map(change)\nclass Gate(name, test):\n    this.name = name\n    this.test = test\nloud = Gate(\"loud\", x -> x > 80)\nquiet = new Gate(\"quiet\", x -> x < 30)\nlevels = [20, 90, 50, 85]\nheard = levels.filter(loud.test)\nscaled = keep(levels, x -> x >= 50, x -> x / 10)\nfirst = levels.find(x -> x > 100) ?? levels.find(quiet.test)\nassert([heard, scaled, first], [[90, 85], [9, 5, 8.5], 20])\nlevels.push(120)\nassert([heard, scaled, first], [[90, 85, 120], [9, 5, 8.5, 12], 120])\nquiet.name",
    "expected": "\"quiet\""
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back failed updates through sections",
    "source": "# Nucleoid rolls back failed updates through sections\n\ncounts = [3, 5]\nfilled = [x ?? spare() for x in counts]\nfailure = \"\"\ntry:\n    counts.push(null)\ncatch error:\n    failure = error\nassert(failure, ReferenceError(\"spare is not defined\"))\nassert([counts.length, filled], [2, [3, 5]])\ndef spare() => 0\ncounts.push(null)\nassert(filled, [3, 5, 0])\nbig = filled.filter(x -> x > 4)\ntry:\n    counts.push(9)\n    assert([filled, big], [[3, 5, 0, 9], [5, 9]])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\n[filled, big]",
    "expected": "[[3,5,0],[5]]"
  },
  {
    "document": "reference",
    "title": "Nucleoid writes one-parameter functions as operator sections",
    "source": "# Nucleoid writes one-parameter functions as operator sections\n\nprices = [12, 30, 8, 45]\nlimit = 10\npicked = prices.filter(x -> x > limit)\ndoubled = [x * 2 for x in picked]\ncheap = prices.count(x -> x <= limit)\nassert(picked, [12, 30, 45])\nassert([doubled, cheap], [[24, 60, 90], 1])\nlimit = 20\nassert([doubled, cheap], [[60, 90], 2])\nstock = [4, null, 7]\n[x ?? 0 for x in stock]",
    "expected": "[4,0,7]"
  },
  {
    "document": "readme",
    "title": "Operator sections pass a comparison without naming a parameter",
    "source": "cells = [0, 3, 0, 7, 7]\nfilled = cells.filter(x -> x != 0)\nsevens = cells.count(x -> x == 7)\nassert([filled, sevens], [[3, 7, 7], 2])\ncells.push(7)\nassert([filled, sevens], [[3, 7, 7, 7], 3])\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Operator sections keep every sensor's alarm count live",
    "source": "class Sensor(name, readings, limit):\n    this.name = name\n    this.readings = readings\n    this.limit = limit\n$Sensor.alarms = .readings.count(x -> x > .limit)\n$Sensor.first = .readings.findIndex(x -> x > .limit)\nnorth = Sensor(\"north\", [12, 31, 7], 20)\nsouth = Sensor(\"south\", [5, 9], 10)\nassert([north.alarms, north.first, south.first], [1, 1, -1])\nnorth.limit = 10\nsouth.readings.push(14)\n[north.alarms, north.first, south.alarms, south.first]\n",
    "expected": "[2,0,1,2]"
  }
]"###;

const WRITTEN_OUT: [(&str, &str); 32] = [
    ("readings.filter(> 20)", "readings.filter(x -> x > 20)"),
    ("readings.count(<= 12)", "readings.count(x -> x <= 12)"),
    ("readings.find(>= 40)", "readings.find(x -> x >= 40)"),
    (
        "readings.findIndex(< 10)",
        "readings.findIndex(x -> x < 10)",
    ),
    ("readings.some(> 50)", "readings.max() > 50"),
    ("readings.every(!= 0)", "!readings.includes(0)"),
    ("values.map(* 2 + 1)", "[x * 2 + 1 for x in values]"),
    ("values.map(** 2 - 1)", "[x ** 2 - 1 for x in values]"),
    ("values.count(% 2 == 0)", "values.count(x -> x % 2 == 0)"),
    ("values.map(% 3)", "[x % 3 for x in values]"),
    (
        r#"values.map(> 4 ? "big" : "small")"#,
        r#"[x > 4 ? "big" : "small" for x in values]"#,
    ),
    (
        "values.filter(> low + 1)",
        "values.filter(x -> x > low + 1)",
    ),
    ("scores.map(?? 0)", "[x ?? 0 for x in scores]"),
    (
        r#"names.map(|| "anonymous")"#,
        r#"[x || "anonymous" for x in names]"#,
    ),
    (r#"flags.map(&& "on")"#, r#"[x && "on" for x in flags]"#),
    ("flags.count(=== true)", "flags.count(x -> x === true)"),
    (
        "readings.filter(> limit)",
        "readings.filter(x -> x > limit)",
    ),
    (
        "over.count(>= limit * 2)",
        "over.count(x -> x >= limit * 2)",
    ),
    (r#"Gate("loud", > 80)"#, r#"Gate("loud", x -> x > 80)"#),
    (
        r#"new Gate("quiet", < 30)"#,
        r#"new Gate("quiet", x -> x < 30)"#,
    ),
    (
        "keep(levels, >= 50, x -> x / 10)",
        "keep(levels, x -> x >= 50, x -> x / 10)",
    ),
    ("levels.find(> 100)", "levels.find(x -> x > 100)"),
    ("counts.map(?? spare())", "[x ?? spare() for x in counts]"),
    ("filled.filter(> 4)", "filled.filter(x -> x > 4)"),
    ("prices.filter(> limit)", "prices.filter(x -> x > limit)"),
    ("picked.map(* 2)", "[x * 2 for x in picked]"),
    ("prices.count(<= limit)", "prices.count(x -> x <= limit)"),
    ("stock.map(?? 0)", "[x ?? 0 for x in stock]"),
    ("cells.filter(!= 0)", "cells.filter(x -> x != 0)"),
    ("cells.count(== 7)", "cells.count(x -> x == 7)"),
    (
        "$Sensor.readings.count(> $Sensor.limit)",
        ".readings.count(x -> x > .limit)",
    ),
    (
        "$Sensor.readings.findIndex(> $Sensor.limit)",
        ".readings.findIndex(x -> x > .limit)",
    ),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedCase {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

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

fn section_operator(token: &Token) -> bool {
    matches!(
        token,
        Token::Equal
            | Token::NotEqual
            | Token::StrictEqual
            | Token::StrictNotEqual
            | Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual
            | Token::Star
            | Token::Percent
            | Token::StarStar
            | Token::AndAnd
            | Token::OrOr
            | Token::QuestionQuestion
    )
}

fn holds_section(source: &str) -> bool {
    tokenize(source).unwrap().windows(2).any(|pair| {
        matches!(pair[0].token, Token::LeftParen | Token::Comma) && section_operator(&pair[1].token)
    })
}

fn written_out(source: &str) -> (String, usize) {
    let mut written = source.to_string();
    let mut rewrites = 0;
    for (section, spelling) in WRITTEN_OUT {
        rewrites += written.matches(section).count();
        written = written.replace(section, spelling);
    }
    (written, rewrites)
}

fn json(runtime: &mut Runtime, source: &str) -> String {
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    runtime.serialize_json(&value).unwrap()
}

fn result(source: &str) -> String {
    json(&mut Runtime::new(), source)
}

fn outcome(runtime: &mut Runtime, source: &str) -> String {
    match runtime.run(source) {
        Ok(value) => runtime.serialize_json(&value).unwrap(),
        Err(error) => format!(
            "{}: {} at {:?}",
            error.kind(),
            error.message(),
            error.position()
        ),
    }
}

fn fresh(source: &str) -> String {
    outcome(&mut Runtime::new(), source)
}

fn failure(source: &str) -> (ErrorKind, String, (usize, usize)) {
    let error = Runtime::new().run(source).unwrap_err();
    let position = error
        .position()
        .unwrap_or_else(|| panic!("{source}: {error} has no position"));
    (
        error.kind(),
        error.message(),
        (position.line, position.column),
    )
}

fn tree(source: &str) -> Result<String, String> {
    parse(source)
        .map(|statements| format!("{statements:?}"))
        .map_err(|error| error.to_string())
}

fn renamed(tree: &str) -> String {
    tree.replace("\"p0\"", "\"\"")
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

fn behaves_as_spelled(steps: &[(&str, &str)], names: &[&str]) {
    let mut sectioned = Runtime::new();
    let mut spelled = Runtime::new();
    for (step, spelling) in steps {
        assert_eq!(
            renumbered(&outcome(&mut sectioned, step)),
            renumbered(&outcome(&mut spelled, spelling)),
            "{step}"
        );
        for name in names {
            if sectioned
                .graph
                .retrieve(&NodeKey::variable(*name))
                .is_some()
            {
                assert_eq!(
                    dependencies(&sectioned, name),
                    dependencies(&spelled, name),
                    "{step}: {name}"
                );
            }
        }
    }
    assert_eq!(sectioned.assertions_run(), spelled.assertions_run());
    assert!(sectioned.take_assertions().is_empty(), "{steps:?}");
    assert!(spelled.take_assertions().is_empty(), "{steps:?}");
}

fn same_as_spelled(pairs: &[(&str, &str, &str)]) {
    for (section, spelled, expected) in pairs {
        let sectioned = renumbered(&fresh(section));
        assert_eq!(sectioned, renumbered(&fresh(spelled)), "{section}");
        assert_eq!(sectioned, *expected, "{section}");
    }
}

fn deepest(build: impl Fn(usize) -> String) -> (usize, String) {
    for depth in 1..200 {
        if let Err(error) = Runtime::check(&build(depth)) {
            assert_eq!(error.kind(), ErrorKind::Syntax, "depth {depth}: {error}");
            return (depth, error.message());
        }
    }
    panic!("no depth limit was reached");
}

fn on_large_stack(work: impl FnOnce() + Send + 'static) {
    thread::Builder::new()
        .stack_size(LARGE_STACK)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn only_the_appended_cases_hold_sections_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(holds_section(&case.source), new, "{}", case.title);
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
    assert_eq!(assertions, 28);
    assert_eq!(cases, 1209);
}

#[test]
fn the_appended_cases_stand_after_nuc_108_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(titles[755], "Nucleoid rolls back failed partition updates");
    assert_eq!(titles[756..762], SPEC_TITLES);
    assert_eq!(titles[804], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid partitions lists and populations with partition",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Partition keeps shipped and pending orders apart in one live rule",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Partition keeps every store's shelves to restock apart from the stocked ones",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(index, cases.len() - 16, "{title}");
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 13].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Operator section arguments\n\n```\n")
        .expect("the language reference documents operator sections");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 109](nuc-0109.md)"));
    assert!(reference.contains(
        "f(> 4)  f(* 2 + 1)                     operator section argument, exactly f(v => v > 4) (NUC 109)"
    ));
    assert_eq!(case.expected.as_deref(), Some("[4,0,7]"));
}

#[test]
fn every_case_is_its_accepted_counterpart_with_only_the_sections_written_out() {
    let sealed: Vec<SealedCase> = serde_json::from_str(ACCEPTED).unwrap();
    let documents = documents();
    assert_eq!(sealed.len(), 9);
    let mut rewrites = 0;
    for sealed in &sealed {
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == sealed.document)
            .unwrap()
            .1;
        let matching: Vec<&common::Case> = cases
            .iter()
            .filter(|case| case.title == sealed.title)
            .collect();
        assert_eq!(matching.len(), 1, "{}", sealed.title);
        let case = matching[0];
        assert_eq!(case.expected, sealed.expected, "{}", sealed.title);
        let (written, count) = written_out(&case.source);
        assert_eq!(written, sealed.source, "{}", sealed.title);
        assert!(!holds_section(&written), "{}", sealed.title);
        rewrites += count;
        let accepted = common::assertions::protected_assertions(&parse(&sealed.source).unwrap());
        let candidate = common::assertions::protected_assertions(&parse(&case.source).unwrap());
        assert_eq!(accepted, candidate, "{}", sealed.title);
        let control = common::Case {
            title: sealed.title.clone(),
            source: sealed.source.clone(),
            expected: sealed.expected.clone(),
        };
        for form in [case, &control] {
            let mut runtime = Runtime::new();
            common::check_with_count(&mut runtime, form, accepted.len())
                .unwrap_or_else(|reason| panic!("{}: {reason}", sealed.title));
            assert_eq!(runtime.assertions_run(), accepted.len(), "{}", sealed.title);
        }
    }
    assert_eq!(rewrites, 32);
}

#[test]
fn every_operator_reads_the_rest_of_its_argument_as_its_lambda_body() {
    let mut compared = 0;
    for operator in OPERATORS {
        for rest in RESTS {
            let section = tree(&format!("f({operator} {rest})"));
            let spelled =
                tree(&format!("f(p0 -> p0 {operator} {rest})")).map(|tree| renamed(&tree));
            match (&section, &spelled) {
                (Ok(section), Ok(spelled)) => assert_eq!(section, spelled, "{operator} {rest}"),
                (Err(_), Err(_)) => {}
                _ => panic!("{operator} {rest}: {section:?} against {spelled:?}"),
            }
            let later = tree(&format!("g(0, {operator} {rest}, 2)"));
            let later_spelled =
                tree(&format!("g(0, p0 -> p0 {operator} {rest}, 2)")).map(|tree| renamed(&tree));
            assert_eq!(later, later_spelled, "g(0, {operator} {rest}, 2)");
            compared += 1;
        }
    }
    assert_eq!(compared, OPERATORS.len() * RESTS.len());
    for (source, expected) in [
        ("[3].map(* 2 + 1)", "[7.0]"),
        ("[2].map(** 3 ** 2)", "[512.0]"),
        ("[2].map(** 3 - 1)", "[7.0]"),
        ("[5].map(% 3 * 2)", "[4.0]"),
        ("[5].map(> 4 == true)", "[true]"),
        ("[5].map(== 5 ? \"y\" : \"n\")", "[\"y\"]"),
        ("[null].map(?? 1 + 1)", "[2.0]"),
        ("[0].map(|| 2 && 3)", "[3.0]"),
        ("[1].map(&& 2 || 3)", "[2.0]"),
        ("[4].map(< 5 && 1)", "[1.0]"),
        ("[4].map(* 2 > 7)", "[true]"),
        ("[4].map(>= 2 + 2)", "[true]"),
        ("[4].map(* -1)", "[-4.0]"),
        ("[4].map(> 2 if true else false)", "[true]"),
        ("[4].map(== 4 is true)", "[true]"),
        ("[4].map(< [5, 6].length * 3)", "[true]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn sections_give_their_lambda_spellings_results_across_queries_operands_and_items() {
    let setup = "xs = [3, 8, 1, 9, 0, -2, 4.5, null, \"a\", true, [2]]\nlimit = 4\nys = [1, 2, 3]";
    let mut sectioned = Runtime::new();
    let mut spelled = Runtime::new();
    sectioned.run(setup).unwrap();
    spelled.run(setup).unwrap();
    let mut compared = 0;
    for operator in OPERATORS {
        for operand in OPERANDS {
            for query in QUERIES {
                let section = format!("xs.{query}({operator} {operand})");
                let lambda = format!("xs.{query}(x -> x {operator} {operand})");
                assert_eq!(
                    outcome(&mut sectioned, &section),
                    outcome(&mut spelled, &lambda),
                    "{section}"
                );
                compared += 1;
            }
            let section = format!("xs.reduce({operator} {operand}, 1)");
            let lambda = format!("xs.reduce(x -> x {operator} {operand}, 1)");
            assert_eq!(
                outcome(&mut sectioned, &section),
                outcome(&mut spelled, &lambda),
                "{section}"
            );
            compared += 1;
        }
    }
    let population = "class Item(n):\n    this.n = n\nItem(3)\nItem(8)\nItem(1)\nlimit = 4";
    let mut sectioned = Runtime::new();
    let mut spelled = Runtime::new();
    sectioned.run(population).unwrap();
    spelled.run(population).unwrap();
    for operator in OPERATORS {
        for query in QUERIES {
            let section = format!("Item.map(i -> i.n).{query}({operator} limit)");
            let lambda = format!("Item.map(i -> i.n).{query}(x -> x {operator} limit)");
            assert_eq!(
                outcome(&mut sectioned, &section),
                outcome(&mut spelled, &lambda),
                "{section}"
            );
            compared += 1;
        }
    }
    assert_eq!(compared, 14 * 10 * 22 + 14 * 21);
}

#[test]
fn sections_stand_wherever_a_call_takes_a_positional_argument() {
    same_as_spelled(&[
        (
            "xs = [3, 8]\nxs.filter(> 4)",
            "xs = [3, 8]\nxs.filter(x -> x > 4)",
            "[8.0]",
        ),
        (
            "xs = [3, 8]\nxs.filter(> 4, 1)",
            "xs = [3, 8]\nxs.filter(x -> x > 4, 1)",
            "[8.0]",
        ),
        (
            "xs = [3, 8]\nxs.filter(> 4,)",
            "xs = [3, 8]\nxs.filter(x -> x > 4,)",
            "[8.0]",
        ),
        (
            "xs = [3, 8]\nxs.filter(\n    > 4\n)",
            "xs = [3, 8]\nxs.filter(\n    x -> x > 4\n)",
            "[8.0]",
        ),
        (
            "xs = [3, 8]\nxs.filter(>4)",
            "xs = [3, 8]\nxs.filter(x -> x>4)",
            "[8.0]",
        ),
        (
            "xs = [3, 8]\nxs.filter(> /* limit */ 4)",
            "xs = [3, 8]\nxs.filter(x -> x > /* limit */ 4)",
            "[8.0]",
        ),
        (
            "def two(f, g, x) => [f(x), g(x)]\ntwo(> 1, < 1, 2)",
            "def two(f, g, x) => [f(x), g(x)]\ntwo(x -> x > 1, x -> x < 1, 2)",
            "[true,false]",
        ),
        (
            "def at(x, f) => f(x)\nat(5, * 2)",
            "def at(x, f) => f(x)\nat(5, x -> x * 2)",
            "10.0",
        ),
        (
            "class Box(test):\n    this.test = test\nBox(> 4).test(5)",
            "class Box(test):\n    this.test = test\nBox(x -> x > 4).test(5)",
            "true",
        ),
        (
            "class Box(test):\n    this.test = test\nnew Box(> 4).test(3)",
            "class Box(test):\n    this.test = test\nnew Box(x -> x > 4).test(3)",
            "false",
        ),
        (
            "class Base(test):\n    this.test = test\nclass Sub extends Base:\n    def init():\n        super(> 4)\nSub().test(9)",
            "class Base(test):\n    this.test = test\nclass Sub extends Base:\n    def init():\n        super(x -> x > 4)\nSub().test(9)",
            "true",
        ),
        (
            "xs = [3, 8]\n(xs.filter)(> 4)",
            "xs = [3, 8]\n(xs.filter)(x -> x > 4)",
            "[8.0]",
        ),
        (
            "xs = [3, 8]\nxs (> 4)",
            "xs = [3, 8]\nxs (x -> x > 4)",
            "TypeError: xs is not a function at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [3, 8]\nxs.filter(> 4).map(* 10)",
            "xs = [3, 8]\nxs.filter(x -> x > 4).map(x -> x * 10)",
            "[80.0]",
        ),
        (
            "xs = [3, 8]\nys = [5, 6, 7]\nxs.filter(> ys.count(>= 6))",
            "xs = [3, 8]\nys = [5, 6, 7]\nxs.filter(x -> x > ys.count(y -> y >= 6))",
            "[3.0,8.0]",
        ),
        (
            "xs = [3, 8]\n[xs.some(> 4), xs.every(> 4), xs.find(> 4), xs.findIndex(> 4)]",
            "xs = [3, 8]\n[xs.some(x -> x > 4), xs.every(x -> x > 4), xs.find(x -> x > 4), xs.findIndex(x -> x > 4)]",
            "[true,false,8.0,1.0]",
        ),
        (
            "f = [1, 2].map(> 1)\nf",
            "f = [1, 2].map(x -> x > 1)\nf",
            "[false,true]",
        ),
        (
            "test = [3, 8].filter\ntest(> 4)",
            "test = [3, 8].filter\ntest(x -> x > 4)",
            "null",
        ),
        (
            "o = {check: f -> f(3)}\no.check(< 4)",
            "o = {check: f -> f(3)}\no.check(x -> x < 4)",
            "true",
        ),
        (
            "def apply(f) => f(2)\n[apply(** 3), apply(% 2), apply(?? 9), apply(&& 5), apply(|| 5)]",
            "def apply(f) => f(2)\n[apply(x -> x ** 3), apply(x -> x % 2), apply(x -> x ?? 9), apply(x -> x && 5), apply(x -> x || 5)]",
            "[8.0,0.0,2.0,5.0,2.0]",
        ),
        (
            "def apply(f) => f()\napply(?? 9)",
            "def apply(f) => f()\napply(x -> x ?? 9)",
            "9.0",
        ),
        (
            "f = [1].map(== 1)[0]\ntypeof [1].map\n[typeof [].filter(> 1), `${[1].map(> 0)}`]",
            "f = [1].map(x -> x == 1)[0]\ntypeof [1].map\n[typeof [].filter(x -> x > 1), `${[1].map(x -> x > 0)}`]",
            "[\"List\",\"true\"]",
        ),
    ]);
}

#[test]
fn standing_sections_follow_writes_as_their_lambdas_do() {
    behaves_as_spelled(
        &[
            ("limit = 20", "limit = 20"),
            ("readings = [12, 31, 7]", "readings = [12, 31, 7]"),
            (
                "over = readings.filter(> limit)",
                "over = readings.filter(x -> x > limit)",
            ),
            (
                "alerts = over.count(>= limit * 2)",
                "alerts = over.count(x -> x >= limit * 2)",
            ),
            ("[over, alerts]", "[over, alerts]"),
            ("limit = 10", "limit = 10"),
            ("[over, alerts]", "[over, alerts]"),
            ("readings.push(45)", "readings.push(45)"),
            ("[over, alerts]", "[over, alerts]"),
            ("readings.shift()", "readings.shift()"),
            ("readings = [5, 25]", "readings = [5, 25]"),
            ("[over, alerts]", "[over, alerts]"),
            (
                "(why over).map(step -> step.rule)",
                "(why over).map(step -> step.rule)",
            ),
            (
                "(affects limit).map(step -> step.node)",
                "(affects limit).map(step -> step.node)",
            ),
            ("frozen = over.value", "frozen = over.value"),
            ("readings.push(99)", "readings.push(99)"),
            ("[over, frozen]", "[over, frozen]"),
            ("names = [\"a\", \"\"]", "names = [\"a\", \"\"]"),
            (
                "shown = names.map(|| \"none\")",
                "shown = names.map(x -> x || \"none\")",
            ),
            ("names.push(null)", "names.push(null)"),
            ("shown", "shown"),
            (
                "filled = [1, null].map(?? 0)",
                "filled = [1, null].map(x -> x ?? 0)",
            ),
            ("filled", "filled"),
        ],
        &["over", "alerts", "frozen", "shown", "filled"],
    );
    behaves_as_spelled(
        &[
            (
                "class Sensor(readings, limit):\n    this.readings = readings\n    this.limit = limit",
                "class Sensor(readings, limit):\n    this.readings = readings\n    this.limit = limit",
            ),
            (
                "$Sensor.alarms = $Sensor.readings.count(> $Sensor.limit)",
                "$Sensor.alarms = $Sensor.readings.count(x -> x > $Sensor.limit)",
            ),
            (
                "loud = Sensor.map(s -> s.alarms).filter(> 0)",
                "loud = Sensor.map(s -> s.alarms).filter(x -> x > 0)",
            ),
            (
                "north = Sensor([12, 31, 7], 20)",
                "north = Sensor([12, 31, 7], 20)",
            ),
            ("south = Sensor([5], 10)", "south = Sensor([5], 10)"),
            (
                "[north.alarms, south.alarms, loud]",
                "[north.alarms, south.alarms, loud]",
            ),
            ("north.limit = 10", "north.limit = 10"),
            ("south.readings.push(14)", "south.readings.push(14)"),
            (
                "[north.alarms, south.alarms, loud]",
                "[north.alarms, south.alarms, loud]",
            ),
            ("delete south.readings", "delete south.readings"),
            ("[south.alarms, loud]", "[south.alarms, loud]"),
        ],
        &["loud"],
    );
}

#[test]
fn the_parameter_has_no_name_an_operand_could_read_or_shadow() {
    for (source, expected) in [
        ("x = 5\n[3, 8].filter(> x)", "[8.0]"),
        ("v = 5\n[3, 8].filter(> v)", "[8.0]"),
        ("it = 5\n[3, 8].filter(> it)", "[8.0]"),
        ("p0 = 5\n[3, 8].filter(> p0)", "[8.0]"),
        ("[3, 8].filter(> [x for x in [1, 5]].length)", "[3.0,8.0]"),
        ("[3, 8].filter(> ((any x in [1]: x > 0) ? 4 : 0))", "[8.0]"),
        ("[3, 8].filter(> [6].map(x -> x)[0])", "[8.0]"),
        ("[3, 8].filter(> [6].map(* 1)[0])", "[8.0]"),
        ("[3, 8].map(* [2].map(* 5)[0])", "[30.0,80.0]"),
        ("[3, 8].map(?? (x -> x)(7))", "[3.0,8.0]"),
        ("[null].map(?? (x -> x)(7))", "[7.0]"),
        ("[3, 8].map(== [3].find(> 2))", "[true,false]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    assert_eq!(
        failure("[3].filter(> missing)"),
        (
            ErrorKind::Reference,
            "missing is not defined".to_string(),
            (1, 1)
        )
    );
    assert_eq!(result("[].filter(> missing)"), "[]");
    let mut runtime = Runtime::new();
    json(&mut runtime, "xs = [3, 8]\nbig = xs.filter(> 4)\nbig");
    assert!(runtime.graph.retrieve(&NodeKey::variable("")).is_none());
    assert!(runtime.state.variable("").is_none());
    assert_eq!(dependencies(&runtime, "big"), ["xs"]);
    for source in [
        "xs = [3, 8]\nbig = xs.filter(> 4)\nwhy big",
        "xs = [3, 8]\nbig = xs.filter(> 4)\naffects xs",
        "xs = [3, 8]\nbig = xs.filter(> 4)\n[big, xs.filter(> 1), [1].map(?? 2)]",
        "f = [1].map(> 0)\nkeep = x -> x\n[f, typeof keep, `${[2].map(* 3)}`]",
    ] {
        let rendered = result(source);
        assert!(!rendered.contains("(>"), "{rendered}");
        assert!(!rendered.contains("\"\""), "{rendered}");
    }
}

#[test]
fn explanations_graph_keys_and_renderings_are_those_of_the_lambda() {
    let section = "xs = [3, 8]\nlimit = 4\nbig = xs.filter(> limit)";
    let spelled = "xs = [3, 8]\nlimit = 4\nbig = xs.filter(x -> x > limit)";
    let mut sectioned = Runtime::new();
    let mut written = Runtime::new();
    sectioned.run(section).unwrap();
    written.run(spelled).unwrap();
    for query in [
        "(why big).map(step -> step.rule)",
        "(why big).map(step -> step.from)",
        "(affects limit).map(step -> step.rule)",
        "(affects xs).map(step -> step.node)",
        "why big |> stated",
        "big |> why |> derived",
    ] {
        assert_eq!(
            json(&mut sectioned, query),
            json(&mut written, query),
            "{query}"
        );
    }
    assert_eq!(
        json(&mut sectioned, "(why big)[0].rule"),
        "\"big = xs.filter(function())\""
    );
    assert_eq!(dependencies(&sectioned, "big"), ["xs", "limit"]);
    assert_eq!(
        dependencies(&sectioned, "big"),
        dependencies(&written, "big")
    );
    assert_eq!(
        generate_all(&parse("big = xs.filter(> limit)").unwrap()),
        generate_all(&parse("big = xs.filter(x -> x > limit)").unwrap())
    );
    assert_eq!(
        generate_all(&parse("xs.map(* 2 + 1).filter(>= 3)").unwrap()),
        "xs.map(function()).filter(function())"
    );
    behaves_as_spelled(
        &[
            ("xs = [3, 8]", "xs = [3, 8]"),
            (
                "if xs.some(> 4):\n    high = true\nelse:\n    high = false",
                "if xs.some(x -> x > 4):\n    high = true\nelse:\n    high = false",
            ),
            (
                "if xs.some(< 2):\n    low = true\nelse:\n    low = false",
                "if xs.some(x -> x < 2):\n    low = true\nelse:\n    low = false",
            ),
            ("[high, low]", "[high, low]"),
            ("xs.push(1)", "xs.push(1)"),
            ("[high, low]", "[high, low]"),
            ("xs = [2]", "xs = [2]"),
            ("[high, low]", "[high, low]"),
        ],
        &[],
    );
    assert_eq!(
        failure("[1].map(> 4 = 1)"),
        (ErrorKind::Syntax, "Cannot assign to >4".to_string(), (1, 1))
    );
    assert_eq!(
        failure("[1].map(x -> x > 4 = 1)"),
        (
            ErrorKind::Syntax,
            "Cannot assign to x>4".to_string(),
            (1, 1)
        )
    );
    assert_eq!(
        failure("xs = [3, 8]\nxs.filter(> 4).nope()"),
        (
            ErrorKind::Type,
            "xs.filter(function()).nope is not a function".to_string(),
            (2, 1)
        )
    );
}

#[test]
fn operators_that_do_not_begin_a_section_keep_their_accepted_readings() {
    let xs = "xs = [3, 8, 1, 9]\n";
    for (line, kind, message, position) in [
        (
            "xs.map(- 1)",
            ErrorKind::Type,
            "map expects a function",
            (2, 1),
        ),
        (
            "xs.map(+ 1)",
            ErrorKind::Type,
            "map expects a function",
            (2, 1),
        ),
        (
            "xs.filter(! 4)",
            ErrorKind::Type,
            "filter expects a function",
            (2, 1),
        ),
        ("xs.map(/ 2)", ErrorKind::Syntax, "Unexpected '/'", (2, 10)),
        (
            "xs.map(/ 2).map(/ 3)",
            ErrorKind::Syntax,
            "Expected ')' but found number 3",
            (2, 19),
        ),
        (
            "xs.filter(in [1, 9])",
            ErrorKind::Syntax,
            "Unexpected keyword 'in'",
            (2, 14),
        ),
        (
            "xs.filter(not in [1, 9])",
            ErrorKind::Syntax,
            "Unexpected keyword 'in'",
            (2, 18),
        ),
        (
            "xs.filter(is 8)",
            ErrorKind::Syntax,
            "Expected ')' but found number 8",
            (2, 14),
        ),
        (
            "xs.filter(is not 8)",
            ErrorKind::Syntax,
            "Expected ')' but found keyword 'not'",
            (2, 14),
        ),
        (
            "xs.filter(and 1)",
            ErrorKind::Syntax,
            "Unexpected keyword 'and'",
            (2, 15),
        ),
        (
            "xs.filter(or 1)",
            ErrorKind::Syntax,
            "Unexpected keyword 'or'",
            (2, 14),
        ),
        (
            "xs.filter(= 4)",
            ErrorKind::Syntax,
            "Unexpected '='",
            (2, 13),
        ),
        (
            "xs.filter(? 1 : 2)",
            ErrorKind::Syntax,
            "Unexpected '?'",
            (2, 13),
        ),
        (
            "xs.filter(: 4)",
            ErrorKind::Syntax,
            "Unexpected ':'",
            (2, 13),
        ),
        (
            "xs.filter(+= 1)",
            ErrorKind::Syntax,
            "Unexpected '+='",
            (2, 14),
        ),
        (
            "xs.filter(|> why)",
            ErrorKind::Syntax,
            "Unexpected '|>'",
            (2, 14),
        ),
        (
            "xs.filter(-> 4)",
            ErrorKind::Syntax,
            "Unexpected '->'",
            (2, 14),
        ),
        (
            "xs.filter(=> 4)",
            ErrorKind::Syntax,
            "Unexpected '=>'",
            (2, 14),
        ),
        (
            "xs.filter(:= 4)",
            ErrorKind::Syntax,
            "Unexpected ':='",
            (2, 14),
        ),
        (
            "xs.filter(... xs)",
            ErrorKind::Syntax,
            "Unexpected '.'",
            (2, 12),
        ),
    ] {
        assert_eq!(
            failure(&format!("{xs}{line}\n")),
            (kind, message.to_string(), position),
            "{line}"
        );
    }
    assert_eq!(result("xs = [3, 8, 1, 9]\nxs.filter(.length)"), "[]");
}

#[test]
fn operators_that_do_not_begin_a_section_keep_their_readings_in_the_section_run() {
    let head = "xs = [3, 8, 1, 9]\ndef two(a, b) => b\n";
    for form in [
        "xs.map(+ 1)",
        "xs.map(- 1)",
        "xs.filter(! 4)",
        "xs.filter(not 4)",
        "xs.filter(typeof 4)",
        "two(1, + 1)",
        "two(1, - 1)",
        "two(1, ! 0)",
        "xs.map(/ 2)",
        "xs.filter(in [1, 9])",
        "xs.filter(not in [1, 9])",
        "xs.filter(is 8)",
        "xs.filter(is not 8)",
        "xs.filter(and 1)",
        "xs.filter(or 1)",
        "xs.filter(= 4)",
        "xs.filter(? 1 : 2)",
        "xs.filter(: 4)",
        "xs.filter(+= 1)",
        "xs.filter(|> why)",
        "xs.filter(-> 4)",
        "xs.filter(=> 4)",
        "xs.filter(:= 4)",
        "xs.filter(... xs)",
        "xs.filter(.length)",
        "two(1, -> 4)",
        "two(1, => 4)",
    ] {
        let sectioned = format!("{head}first = xs.filter(> 4)\nsecond = {form}\n[first, second]");
        let spelled =
            format!("{head}first = xs.filter(x -> x > 4)\nsecond = {form}\n[first, second]");
        assert_eq!(fresh(&sectioned), fresh(&spelled), "{form}");
    }
    assert_eq!(
        fresh(&format!("{head}[xs.filter(> 4), xs.map(- 1)]")),
        "TypeError: map expects a function at Some(Position { line: 3, column: 1 })"
    );
    assert_eq!(
        fresh(&format!(
            "{head}[xs.filter(> 4), two(1, - 1), two(1, + 1), two(1, ! 0)]"
        )),
        "[[8.0,9.0],-1.0,1.0,true]"
    );
    assert_eq!(
        failure(&format!("{head}xs.filter(> 4)\nxs.filter(-> 4)")),
        (ErrorKind::Syntax, "Unexpected '->'".to_string(), (4, 14))
    );
    assert_eq!(
        failure(&format!("{head}xs.filter(> 4)\ntwo(1, => 4)")),
        (ErrorKind::Syntax, "Unexpected '=>'".to_string(), (4, 11))
    );
}

#[test]
fn section_rests_read_as_lambda_bodies_in_every_enclosing_context() {
    let head = "xs = [3, 8, 1, 9]\nys = [1, 9]\nflag = true\n";
    for (section, spelled) in [
        (
            "xs.map(> 4 if flag else 0)",
            "xs.map(x -> x > 4 if flag else 0)",
        ),
        ("xs.map(== 1 in ys)", "xs.map(x -> x == 1 in ys)"),
        ("xs.map(?? lambda: 0)", "xs.map(x -> x ?? lambda: 0)"),
        ("xs.map(?? lambda v: 7)", "xs.map(x -> x ?? lambda v: 7)"),
        ("xs.map(* sum of ys)", "xs.map(x -> x * sum of ys)"),
        ("xs.map(> 4 ? 1 : 0)", "xs.map(x -> x > 4 ? 1 : 0)"),
    ] {
        let top = fresh(&format!("{head}{section}"));
        assert!(!top.contains("Error"), "{section}: {top}");
        for wrapper in [
            "r = @@\nr",
            "r = `${@@}`\nr",
            "r = 0\nif @@.length > 0:\n    r = @@\nr",
            "r = 1 if @@.length > 0 else 2\nr",
            "o = {k: @@}\no.k",
            "o = {k: 1 j: @@}\no.j",
            "r = [@@][0]\nr",
            "r = (@@)\nr",
            "r = suppose: @@\nr",
            "def g() => @@\ng()",
            "r = ys.map(z -> @@)[0]\nr",
            "r = [v for v in @@]\nr",
            "r = [v for v in ys if @@.length > 0]\nr",
        ] {
            assert_eq!(
                fresh(&format!("{head}{}", wrapper.replace("@@", section))),
                fresh(&format!("{head}{}", wrapper.replace("@@", spelled))),
                "{wrapper}: {section}"
            );
        }
    }
}

#[test]
fn item_words_stop_at_a_section_as_at_a_lambda_body() {
    let head = "colors = [\"RED\", \"BLUE\"]\n";
    same_as_spelled(&[
        (
            &format!("{head}picked is colors.filter(== [RED][0])\npicked"),
            &format!("{head}picked is colors.filter(x -> x == [RED][0])\npicked"),
            "ReferenceError: RED is not defined at Some(Position { line: 2, column: 1 })",
        ),
        (
            &format!("{head}picked is [colors.filter(== \"RED\"), BLUE]\npicked"),
            &format!("{head}picked is [colors.filter(x -> x == \"RED\"), BLUE]\npicked"),
            "[[\"RED\"],\"BLUE\"]",
        ),
        (
            &format!(
                "{head}picked is {{k: colors.filter(!= \"RED\"), j: BLUE}}\n[picked.k, picked.j]"
            ),
            &format!(
                "{head}picked is {{k: colors.filter(x -> x != \"RED\"), j: BLUE}}\n[picked.k, picked.j]"
            ),
            "[[\"BLUE\"],\"BLUE\"]",
        ),
        (
            &format!("{head}picked is [colors.map(?? [GREEN][0]), AMBER]\npicked"),
            &format!("{head}picked is [colors.map(x -> x ?? [GREEN][0]), AMBER]\npicked"),
            "[[\"RED\",\"BLUE\"],\"AMBER\"]",
        ),
    ]);
}

#[test]
fn positions_other_than_a_whole_call_argument_keep_their_accepted_errors() {
    for (source, message, position) in [
        ("f = (> 4)\nf(5)\n", "Unexpected '>'", (1, 8)),
        (
            "xs = [3, 8, 1, 9]\nxs.filter((> 4))\n",
            "Unexpected '>'",
            (2, 14),
        ),
        ("xs = [3, 8, 1, 9]\n[> 4]\n", "Unexpected '>'", (2, 4)),
        ("xs = [3, 8, 1, 9]\n{a: > 4}\n", "Unexpected ':'", (2, 5)),
        ("xs = [3, 8, 1, 9]\nxs[> 4]\n", "Unexpected '>'", (2, 6)),
        (
            "def f(test) => test(5)\nf(test= > 4)\n",
            "Unexpected '>'",
            (2, 11),
        ),
        (
            "xs = [3, 8, 1, 9]\nxs.filter(x -> > 4)\n",
            "Unexpected '>'",
            (2, 18),
        ),
        ("x = > 4\n", "Unexpected '>'", (1, 7)),
        (
            "xs = [3, 8, 1, 9]\nassert: > 4, true\n",
            "Unexpected '>'",
            (2, 11),
        ),
        (
            "a = 1\nb = a + 1\nb |> depth(> 1)\n",
            "Unexpected '>'",
            (3, 14),
        ),
        ("a = (> 4) |> why\n", "Unexpected '>'", (1, 8)),
        ("(> 4)\n", "Unexpected '>'", (1, 4)),
        (
            "def g(> 4) => 1\n",
            "Expected a parameter but found '>'",
            (1, 9),
        ),
        (
            "class C(> 4)\n",
            "Expected a parameter but found '>'",
            (1, 11),
        ),
        (
            "f = function(> 4) { return 1 }\n",
            "Expected a parameter but found '>'",
            (1, 16),
        ),
        (
            "xs = [3, 8, 1, 9]\nxs.map(1 +)\n",
            "Unexpected ')'",
            (2, 11),
        ),
        ("x = 1\nx **= (1 +)\n", "Unexpected ')'", (2, 11)),
        ("p = [1, 2]\na, *rest = p\n", "Unexpected ','", (2, 4)),
        (
            "class Pair(*, a)\n",
            "Expected a parameter but found '*'",
            (1, 13),
        ),
        (
            "callback = (*, a) => a\n",
            "Expected a parameter but found '*'",
            (1, 14),
        ),
        ("x = **2\n", "Unexpected '**'", (1, 7)),
        ("b = 1\nx = ?? b\n", "Unexpected '??'", (2, 8)),
        ("x :=> 2\n", "Unexpected '>'", (1, 7)),
        (
            "price = 1\ntotal = 2\ntotal ||> path(price)\n",
            "Unexpected '>'",
            (3, 11),
        ),
        ("callback = value - > value\n", "Unexpected '>'", (1, 22)),
        ("def f(x) => x\nf(.a == 1)\n", "Unexpected '.'", (2, 4)),
    ] {
        assert_eq!(
            failure(source),
            (ErrorKind::Syntax, message.to_string(), position),
            "{source}"
        );
    }
}

#[test]
fn malformed_sections_report_the_error_further_into_the_source() {
    let xs = "xs = [3, 8, 1, 9]\n";
    for (line, message, position) in [
        ("xs.filter(>)", "Unexpected '>'", (2, 12)),
        ("xs.filter(> )", "Unexpected '>'", (2, 13)),
        ("xs.map(** )", "Unexpected '**'", (2, 11)),
        ("xs.map(?? )", "Unexpected '??'", (2, 11)),
        ("xs.filter(> 4 &&)", "Unexpected ')'", (2, 17)),
        ("xs.filter(> 4 && < 9)", "Unexpected '<'", (2, 20)),
        ("xs.filter(> > 4)", "Unexpected '>'", (2, 15)),
        ("xs.filter(>, 1)", "Unexpected ','", (2, 14)),
        (
            "xs.filter(> 4 4)",
            "Expected ')' but found number 4",
            (2, 15),
        ),
        ("xs.filter(> 4 &&, 1)", "Unexpected ','", (2, 19)),
        ("xs.filter(> 4, 1 +)", "Unexpected ')'", (2, 19)),
        ("xs.filter(> 4 ? 1)", "Expected ':' but found ')'", (2, 18)),
        (
            "xs.filter(> 4 += 1)",
            "Invalid compound assignment target",
            (2, 15),
        ),
        (
            "xs.filter(> 4 := 1)",
            "Expected ')' but found ':='",
            (2, 15),
        ),
        ("xs.filter(> 4) +", "Unexpected end of line", (3, 1)),
        ("xs.filter(> 4)\nx = )", "Unexpected ')'", (3, 5)),
        ("x = )\nxs.filter(> 4)", "Unexpected ')'", (2, 5)),
        ("`${xs.filter(> 4) +}`", "Unexpected end of input", (2, 20)),
        (
            "`${xs.filter(> 4) && }`",
            "Unexpected end of input",
            (2, 22),
        ),
        (
            "t = `${xs.filter(> 1)}`\nu = `${1 +}`",
            "Unexpected end of input",
            (3, 11),
        ),
    ] {
        assert_eq!(
            failure(&format!("{xs}{line}\n")),
            (ErrorKind::Syntax, message.to_string(), position),
            "{line}"
        );
    }
}

#[test]
fn failing_programs_without_a_section_window_keep_their_accepted_errors() {
    for (source, message, position) in [
        ("x = `${1 +}`\n", "Unexpected end of input", (1, 11)),
        (
            "b = {}\nx = `a ${b.}`\n",
            "Expected a property but found end of input",
            (2, 12),
        ),
        (
            "t = `${function() { return ) }}`\n",
            "Unexpected ')'",
            (1, 30),
        ),
        (
            "y = 2\nt = `a ${function() { class Q:\n    def m(self): 1 }}`\ny\n",
            "Expected a parameter but found keyword 'this'",
            (3, 15),
        ),
        (
            "xs = [1]\n`${xs.filter(x -> x >)}`\n",
            "Unexpected ')'",
            (2, 23),
        ),
        (
            "`${[1, 2}`\n",
            "Expected ']' but found end of input",
            (1, 9),
        ),
        ("x = 1 +\n`${x}`\n", "Unexpected end of line", (2, 1)),
        ("x = 1\n`${x}` +\n", "Unexpected end of line", (3, 1)),
        ("`${`${1 +}`}`\n", "Unexpected character '$'", (1, 5)),
        ("a = `x` b = )\n", "Unexpected ')'", (1, 13)),
        ("if `${1}`:\n    x = \n", "Unexpected end of line", (3, 1)),
        (
            "xs = [1]\n`${xs.map(x -> x * )}`\nys = `${xs}`\n",
            "Unexpected ')'",
            (2, 21),
        ),
        ("`${1}` `${2 +}`\n", "Unexpected end of input", (1, 14)),
        ("s = `${x}`\nf(, x)\n", "Unexpected ','", (2, 5)),
        ("s = `t`\n[1, > 2]\n", "Unexpected '>'", (2, 7)),
        ("s = `t`\nf(test= > 2)\n", "Unexpected '>'", (2, 11)),
        ("s = `t`\nx = (> 2)\n", "Unexpected '>'", (2, 8)),
        (
            "s = `t`\nclass C(> 4)\n",
            "Expected a parameter but found '>'",
            (2, 11),
        ),
        ("s = `t`\nxs[> 1]\n", "Unexpected '>'", (2, 6)),
        (
            "s = `${1}`\ndef f(a, *) => a\n",
            "Expected a parameter but found '*'",
            (2, 11),
        ),
        ("s = `t`\na, *rest = [1, 2]\n", "Unexpected ','", (2, 4)),
        ("s = `${1}`\nf = x -> > 4\n", "Unexpected '>'", (2, 12)),
        ("s = `${1}`\nx = **2\n", "Unexpected '**'", (2, 7)),
        (
            "class P:\n    def m(self) => 1\nt = `${1}`\nx = )\n",
            "Expected a parameter but found keyword 'this'",
            (2, 15),
        ),
        ("while `${1}` is a: b\nx = )\n", "Unexpected ')'", (2, 5)),
    ] {
        assert_eq!(
            failure(source),
            (ErrorKind::Syntax, message.to_string(), position),
            "{source}"
        );
    }
    assert_eq!(
        failure("s = `${x}`\nf(x, )\n"),
        (ErrorKind::Reference, "x is not defined".to_string(), (1, 1))
    );
    assert_eq!(result("a = 1\n`${a is}`"), "\"1\"");
}

#[test]
fn accepted_inputs_holding_sections_in_ignored_tails_keep_their_results() {
    assert_eq!(
        result("xs = [3, 8, 1, 9]\n`v ${xs.length, xs.filter(> 4)}`"),
        "\"v 4\""
    );
    assert_eq!(
        result("xs = [3, 8, 1, 9]\n`${xs.length, xs.filter(> 4)}`"),
        "\"4\""
    );
    for items in ["[1]", "[9]"] {
        assert_eq!(
            result(&format!("xs = {items}\n`${{1 if xs.some(> 4) else 2}}`")),
            "\"1\"",
            "{items}"
        );
    }
    let expression = |source: &str| format!("{:?}", parse_expression(source).unwrap());
    assert_eq!(expression("1 if xs.some(> 4) else 2"), "Number(1.0)");
    assert_eq!(
        expression("xs.length, xs.filter(> 4)"),
        expression("xs.length")
    );
    assert_eq!(expression("a if f(== 1, 2) else b"), "Identifier(\"a\")");
    assert_eq!(
        expression("xs.length xs.filter(> 4)"),
        expression("xs.length")
    );
    assert_eq!(expression("1 if [xs.some(> 4)] else 2"), "Number(1.0)");
    assert_eq!(
        expression("(1 if xs.some(> 4) else 2)"),
        renamed(&expression("(1 if xs.some(p0 -> p0 > 4) else 2)"))
    );
    assert_eq!(
        expression("xs.filter(> 4)"),
        renamed(&expression("xs.filter(p0 -> p0 > 4)"))
    );
    assert_eq!(
        parse_expression("xs.filter(> 4 && < 9)")
            .unwrap_err()
            .to_string(),
        "SyntaxError: Unexpected '<' (line 1, column 20)"
    );
}

#[test]
fn tentative_statements_and_retries_read_sections_as_their_lambdas() {
    same_as_spelled(&[
        (
            "xs = [3, 8, 1, 9]\nfirst, second = xs.filter(> 4)\n[first, second]",
            "xs = [3, 8, 1, 9]\nfirst, second = xs.filter(x -> x > 4)\n[first, second]",
            "[8.0,9.0]",
        ),
        (
            "xs = [3, 8, 1, 9]\n{length} = xs.filter(> 4)\nlength",
            "xs = [3, 8, 1, 9]\n{length} = xs.filter(x -> x > 4)\nlength",
            "2.0",
        ),
        (
            "xs = [3, 8, 1, 9]\n{length: n} = xs.filter(> 4)\nn",
            "xs = [3, 8, 1, 9]\n{length: n} = xs.filter(x -> x > 4)\nn",
            "2.0",
        ),
        (
            "xs = [3, 8, 1, 9]\nn = 0\nwhile xs.some(> n):\n    n += 4\nn",
            "xs = [3, 8, 1, 9]\nn = 0\nwhile xs.some(x -> x > n):\n    n += 4\nn",
            "12.0",
        ),
        (
            "xs = [3, 8, 1, 9]\nr = suppose xs.push(20): xs.filter(> 8)\n[r, xs]",
            "xs = [3, 8, 1, 9]\nr = suppose xs.push(20): xs.filter(x -> x > 8)\n[r, xs]",
            "[[9.0,20.0],[3.0,8.0,1.0,9.0]]",
        ),
        (
            "xs = [3, 8, 1, 9]\nxs.filter(> 4 is true)",
            "xs = [3, 8, 1, 9]\nxs.filter(x -> x > 4 is true)",
            "[8.0,9.0]",
        ),
        (
            "xs = [3, 8, 1, 9]\nxs.filter(> [y for y in xs].length)",
            "xs = [3, 8, 1, 9]\nxs.filter(x -> x > [y for y in xs].length)",
            "[8.0,9.0]",
        ),
        (
            "o = {q: 5, p: 0}\nxs = [3, 8]\no.p = xs.filter(> .q)\no.p",
            "o = {q: 5, p: 0}\nxs = [3, 8]\no.p = xs.filter(x -> x > .q)\no.p",
            "[8.0]",
        ),
        (
            "class A:\n    def m(self, xs) => xs.filter(> 4)\nA().m([3, 8])",
            "class A:\n    def m(self, xs) => xs.filter(x -> x > 4)\nA().m([3, 8])",
            "[8.0]",
        ),
        (
            "xs = [1]\ny = 1 if xs.some(> 4) else 2\ny",
            "xs = [1]\ny = 1 if xs.some(x -> x > 4) else 2\ny",
            "2.0",
        ),
        (
            "xs = [9]\n[1 if xs.some(> 4) else 2]",
            "xs = [9]\n[1 if xs.some(x -> x > 4) else 2]",
            "[1.0]",
        ),
        (
            "xs = [3, 8]\nflag = (xs.some(> 4) is true)\nflag",
            "xs = [3, 8]\nflag = (xs.some(x -> x > 4) is true)\nflag",
            "true",
        ),
        (
            "flag = (1 is 1)\nxs = [3]\nwhile xs.some(> 4) is a: b",
            "flag = (1 is 1)\nxs = [3]\nwhile xs.some(x -> x > 4) is a: b",
            "ReferenceError: while is not defined at Some(Position { line: 3, column: 1 })",
        ),
        (
            "a = 1\nxs = [3]\ns = suppose xs.some(> 1) is a: a\ns",
            "a = 1\nxs = [3]\ns = suppose xs.some(x -> x > 1) is a: a\ns",
            "ReferenceError: suppose is not defined at Some(Position { line: 3, column: 1 })",
        ),
        (
            "x is [3, 8].filter(> 4)\nx",
            "x is [3, 8].filter(y -> y > 4)\nx",
            "[8.0]",
        ),
    ]);
}

#[test]
fn interpolations_read_sections_and_report_program_positions() {
    same_as_spelled(&[
        (
            "xs = [3, 8]\n`${xs.filter(> 4)}`",
            "xs = [3, 8]\n`${xs.filter(x -> x > 4)}`",
            "\"8\"",
        ),
        (
            "xs = [3, 8]\n`${xs.filter(> 4).length} big`",
            "xs = [3, 8]\n`${xs.filter(x -> x > 4).length} big`",
            "\"1 big\"",
        ),
        (
            "xs = [3, 8]\n`${xs.map(* 2)} and ${xs.map(?? 0).length}`",
            "xs = [3, 8]\n`${xs.map(x -> x * 2)} and ${xs.map(x -> x ?? 0).length}`",
            "\"6,16 and 2\"",
        ),
        (
            "xs = [3, 8]\nwhy (> 4)",
            "xs = [3, 8]\nwhy (x -> x > 4)",
            "ReferenceError: why is not defined at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [1]\n`${xs.some(> 4) ? 1 : 2}`",
            "xs = [1]\n`${xs.some(x -> x > 4) ? 1 : 2}`",
            "\"2\"",
        ),
        (
            "xs = [3, 8]\nlabel = `${xs.count(> 2)} over`\nxs.push(9)\nlabel",
            "xs = [3, 8]\nlabel = `${xs.count(x -> x > 2)} over`\nxs.push(9)\nlabel",
            "\"3 over\"",
        ),
        (
            "xs = [1]\n`${xs (> 4)}`",
            "xs = [1]\n`${xs (x -> x > 4)}`",
            "TypeError: xs is not a function at Some(Position { line: 2, column: 1 })",
        ),
    ]);
    assert_eq!(
        failure("xs = [1]\n`${xs.filter(> 4)}` + )\n"),
        (ErrorKind::Syntax, "Unexpected ')'".to_string(), (2, 23))
    );
}

#[test]
fn list_queries_take_sections_as_their_lambdas() {
    for (section, expected) in [
        ("[3, 8, 1, 9].partition(> 4)", "[[8.0,9.0],[3.0,1.0]]"),
        ("[13, 42, 7, 29].sortBy(* -1)", "[42.0,29.0,13.0,7.0]"),
        ("[13, 42, 7, 29].sortBy(% 10)", "[42.0,13.0,7.0,29.0]"),
        ("[13, 42, 7, 29].maxBy(% 10)", "29.0"),
        ("[13, 42, 7, 29].minBy(% 10)", "42.0"),
        ("[13, 42, 7, 29].maxBy(* -1)", "7.0"),
        ("[1, 2, 3].sum(* 2)", "12.0"),
        ("[1, 2, 3].max(* -1)", "-1.0"),
        ("[1, 2, 3].min(** 2)", "1.0"),
        ("[1, 2, 3].average(* 2)", "4.0"),
        ("[1, 2, 3, 4].median(* 2)", "5.0"),
        ("[1, 2, 3, 4].distinct(% 2)", "[1.0,0.0]"),
        ("[1, 2, 3, 4].mode(> 1)", "true"),
        ("[1, 2, 3].findLast(< 3)", "2.0"),
        ("[1, 2, 3].findLastIndex(< 3)", "1.0"),
        ("[1, 2, 3].flatMap(* 2)", "[2.0,4.0,6.0]"),
        ("[1, 2, 3].reduce(* 2, 1)", "8.0"),
        ("[1, 2, 3].count(!== 2)", "2.0"),
        ("[1, 2, 3].every(=== 1)", "false"),
        ("[\"a\", \"b\"].filter(== \"a\")", "[\"a\"]"),
        ("[\"a\", \"b\"].map(< \"b\")", "[true,false]"),
    ] {
        let spelled = section
            .replacen("(> ", "(x -> x > ", 1)
            .replacen("(* ", "(x -> x * ", 1)
            .replacen("(% ", "(x -> x % ", 1)
            .replacen("(** ", "(x -> x ** ", 1)
            .replacen("(< ", "(x -> x < ", 1)
            .replacen("(!== ", "(x -> x !== ", 1)
            .replacen("(=== ", "(x -> x === ", 1)
            .replacen("(== ", "(x -> x == ", 1);
        assert!(!holds_section(&spelled), "{spelled}");
        assert_eq!(result(section), expected, "{section}");
        assert_eq!(result(&spelled), expected, "{spelled}");
    }
}

#[test]
fn failed_updates_through_sections_roll_back_as_their_lambdas() {
    behaves_as_spelled(
        &[
            ("counts = [3, 5]", "counts = [3, 5]"),
            (
                "filled = counts.map(?? spare())",
                "filled = counts.map(x -> x ?? spare())",
            ),
            (
                "failure = \"\"\ntry:\n    counts.push(null)\ncatch error:\n    failure = error\n[failure, counts.length, filled]",
                "failure = \"\"\ntry:\n    counts.push(null)\ncatch error:\n    failure = error\n[failure, counts.length, filled]",
            ),
            ("counts.push(null)", "counts.push(null)"),
            ("[counts, filled]", "[counts, filled]"),
            ("def spare() => 0", "def spare() => 0"),
            ("counts.push(null)\nfilled", "counts.push(null)\nfilled"),
            (
                "big = filled.filter(> 4)",
                "big = filled.filter(x -> x > 4)",
            ),
            (
                "try:\n    counts.push(9)\n    assert([filled, big], [[3, 5, 0, 9], [5, 9]])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\n[filled, big]",
                "try:\n    counts.push(9)\n    assert([filled, big], [[3, 5, 0, 9], [5, 9]])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\n[filled, big]",
            ),
            (
                "seen = suppose counts.push(7): big\n[seen, big]",
                "seen = suppose counts.push(7): big\n[seen, big]",
            ),
        ],
        &["filled", "big"],
    );
    assert_eq!(
        failure("counts = [3, null]\nfilled = counts.map(?? spare())"),
        (
            ErrorKind::Reference,
            "spare is not defined".to_string(),
            (2, 1)
        )
    );
}

#[test]
fn a_section_cut_off_at_the_nesting_limit_leaves_no_operand_behind() {
    on_large_stack(cut_off_sections);
}

fn cut_off_sections() {
    let nested =
        |depth: usize, opening: &str| format!("{}1{}", opening.repeat(depth), ")".repeat(depth));
    let mut failures = 0;
    for depth in 1..=40 {
        let section = nested(depth, "f(> ");
        let lambda = nested(depth, "f(x -> x > ");
        for shape in [
            "f = x -> x\nr = 1 if @ else 2 if @ else 3\nr",
            "f = x -> x\ny = 1 if @ else 2\ny",
            "f = x -> x\nr = suppose @: 1\nr",
            "f = x -> x\nwhile @ is a: b",
        ] {
            let sectioned = Runtime::new().run(&shape.replace('@', &section));
            let spelled = Runtime::new().run(&shape.replace('@', &lambda));
            match (sectioned, spelled) {
                (Ok(left), Ok(right)) => {
                    assert_eq!(left.to_string(), right.to_string(), "{shape} at {depth}");
                }
                (Err(left), Err(right)) => {
                    failures += 1;
                    assert_eq!(
                        (
                            left.kind(),
                            left.message(),
                            left.position().map(|at| at.line)
                        ),
                        (
                            right.kind(),
                            right.message(),
                            right.position().map(|at| at.line)
                        ),
                        "{shape} at {depth}"
                    );
                }
                (left, right) => panic!("{shape} at {depth}: {left:?} against {right:?}"),
            }
        }
    }
    assert!(failures >= 30, "{failures}");
}

#[test]
fn deep_conditions_stay_in_the_section_where_lambdas_read_them_around_the_lambda() {
    on_large_stack(deep_conditions);
}

fn deep_conditions() {
    let chain = |depth: usize, opening: &str| {
        format!(
            "f = x -> x\ny = {}c{}",
            opening.repeat(depth),
            " else 0)".repeat(depth)
        )
    };
    for depth in 1..=20 {
        let spelled = tree(&chain(depth, "f(p0 -> p0 > 1 if ")).map(|tree| renamed(&tree));
        assert_eq!(tree(&chain(depth, "f(> 1 if ")), spelled, "{depth}");
        assert!(
            !spelled.unwrap().contains("consequent: Function"),
            "{depth}"
        );
    }
    assert_eq!(
        failure(&chain(21, "f(> 1 if ")),
        (
            ErrorKind::Syntax,
            "Expected ')' but found keyword 'if'".to_string(),
            (2, 11)
        )
    );
    let around = tree(&chain(21, "f(p0 -> p0 > 1 if ")).unwrap();
    assert!(around.contains("consequent: Function"), "{around}");
}

#[test]
fn nested_sections_reach_the_limits_of_their_lambda_spellings() {
    on_large_stack(nested_sections);
}

fn nested_sections() {
    let section =
        deepest(|depth| format!("f = x -> x\n{}1{}", "f(> ".repeat(depth), ")".repeat(depth)));
    let spelled = deepest(|depth| {
        format!(
            "f = x -> x\n{}1{}",
            "f(x -> x > ".repeat(depth),
            ")".repeat(depth)
        )
    });
    assert_eq!(section, spelled);
    assert_eq!(section.1, "Expressions are nested too deeply");
    assert!(section.0 > 20, "{section:?}");
    let templated = deepest(|depth| {
        format!(
            "f = x -> x\ns = `${{{}1{}}}`",
            "f(> ".repeat(depth),
            ")".repeat(depth)
        )
    });
    let templated_spelled = deepest(|depth| {
        format!(
            "f = x -> x\ns = `${{{}1{}}}`",
            "f(x -> x > ".repeat(depth),
            ")".repeat(depth)
        )
    });
    assert_eq!(templated, templated_spelled);
    let operands = vec!["1"; 129].join(" + ");
    assert_eq!(
        fresh(&format!("[1].map(> {operands})")),
        fresh(&format!("[1].map(x -> x > {operands})"))
    );
    assert_eq!(result(&format!("[1].map(< {operands})")), "[true]");
    let operands = vec!["1"; 130].join(" + ");
    let (kind, message, (line, column)) = failure(&format!("[1].map(> {operands})"));
    let spelled = failure(&format!("[1].map(x -> x > {operands})"));
    assert_eq!(message, "Expression has too many operands");
    assert_eq!((kind, message, (line, column + "x -> x ".len())), spelled);
    let wide: Vec<String> = (0..2000).map(|index| format!("> {index}")).collect();
    let call = format!("def many(...fs) => fs.length\nmany({})", wide.join(", "));
    assert_eq!(result(&call), "2000.0");
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-operator-sections-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_sections_as_their_lambdas() {
    let program = "rows = [2, 3, 1]\nbig = rows.filter(> 1)\nrows.push(5)\nbig";
    let spelled = program.replace("(> 1)", "(x -> x > 1)");
    let file = SourceFile::new("file");
    for options in [
        &[][..],
        &["--json"][..],
        &["--graph"][..],
        &["--json", "--graph"][..],
    ] {
        let sectioned = file.run(program, options);
        let written = file.run(&spelled, options);
        assert!(sectioned.status.success(), "{options:?} {sectioned:?}");
        assert!(sectioned.stderr.is_empty(), "{options:?} {sectioned:?}");
        assert_eq!(sectioned.stdout, written.stdout, "{options:?}");
    }
    assert_eq!(
        String::from_utf8(file.run(program, &["--json"]).stdout).unwrap(),
        "[2.0,3.0,5.0]\n"
    );
    assert_eq!(
        String::from_utf8(file.run(program, &[]).stdout).unwrap(),
        "2,3,5\n"
    );
    let graph = String::from_utf8(file.run(program, &["--json", "--graph"]).stdout).unwrap();
    assert!(graph.contains("big [variable]"), "{graph}");
    assert!(!graph.contains("(>"), "{graph}");
    let failed = file.run("xs = [1]\nxs.filter(> 4 && < 9)\n", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "SyntaxError: Unexpected '<' (line 2, column 20)\n"
    );
    let session = "xs = [3, 8, 1]\nbig = xs.filter(> 2)\nbig\nxs.push(9)\nbig\nxs.map(?? 0)\nxs.filter(> 4 && < 9)\n.exit\n";
    let spelled_session = session
        .replace("(> 2)", "(x -> x > 2)")
        .replace("(?? 0)", "(x -> x ?? 0)");
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let sectioned = repl(&session.replace('\n', newline), options);
            let written = repl(&spelled_session.replace('\n', newline), options);
            assert!(sectioned.status.success(), "{options:?} {sectioned:?}");
            assert_eq!(sectioned.stdout, written.stdout, "{options:?}");
            let stdout = String::from_utf8(sectioned.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "[3.0,8.0]\n4.0\n[3.0,8.0,9.0]\n[3.0,8.0,1.0,9.0]\n"
            } else {
                "3,8\n4\n3,8,9\n3,8,1,9\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
            assert_eq!(
                String::from_utf8(sectioned.stderr).unwrap(),
                "SyntaxError: Unexpected '<' (line 1, column 20)\n",
                "{options:?}"
            );
        }
    }
}
