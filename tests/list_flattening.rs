mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::thread;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime, Value};
use regex::Regex;
use serde::Deserialize;

const SPEC_TITLES: [&str; 6] = [
    "Nucleoid flattens nested lists by one level",
    "Nucleoid flattens lists to a given depth",
    "Nucleoid keeps flattened lists live",
    "Nucleoid flattens lists in class level rules",
    "Nucleoid queries the cells of flattened grids",
    "Nucleoid rolls back failed flattening updates",
];
const REFERENCE_TITLE: &str = "Nucleoid flattens nested lists with flat";
const README_TITLE: &str = "Flat turns nested lists into one live list";
const EXAMPLES_TITLE: &str = "Flattening keeps the dirty tiles of every room counted";

const DEEP: usize = 25_000;
const LARGE_STACK: usize = 256 * 1024 * 1024;

const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid flattens nested lists by one level",
    "source": "# Nucleoid flattens nested lists by one level\n\nnested = [[1, 2], 2, [[3, [4]]], [], \"ab\"]\nonce = nested.flatMap(v => v)\ntwice = once.flatMap(v => v)\nassert(once, [1, 2, 2, [3, [4]], \"ab\"])\nassert(twice, [1, 2, 2, 3, [4], \"ab\"])\nassert(nested, [[1, 2], 2, [[3, [4]]], [], \"ab\"])\nnested.push([[5]], 6)\nassert(once, [1, 2, 2, [3, [4]], \"ab\", [5], 6])\nassert(twice, [1, 2, 2, 3, [4], \"ab\", 5, 6])\ntwice.length",
    "expected": "8"
  },
  {
    "document": "spec",
    "title": "Nucleoid flattens lists to a given depth",
    "source": "# Nucleoid flattens lists to a given depth\n\ndeep = [1, [2, [3, [4, [5]]]], [[6]]]\ntwo = deep.flatMap(v => v).flatMap(v => v)\nthree = deep.flatMap(v => v).flatMap(v => v).flatMap(v => v)\nsame = deep.map(v => v)\nassert(two, [1, 2, 3, [4, [5]], 6])\nassert(three, [1, 2, 3, 4, [5], 6])\nassert(same, [1, [2, [3, [4, [5]]]], [[6]]])\ndeep.push([[7, [8]]])\nassert(two, [1, 2, 3, [4, [5]], 6, 7, [8]])\nassert(three, [1, 2, 3, 4, [5], 6, 7, 8])\nassert(same, deep)\nthree.length",
    "expected": "8"
  },
  {
    "document": "spec",
    "title": "Nucleoid keeps flattened lists live",
    "source": "# Nucleoid keeps flattened lists live\n\nshifts = [[\"ana\", \"bo\"], [\"cy\"]]\nstaff = shifts.flatMap(v => v)\nsize = staff.length\nassert([staff, size], [[\"ana\", \"bo\", \"cy\"], 3])\nshifts.push([\"di\"])\nassert([staff, size], [[\"ana\", \"bo\", \"cy\", \"di\"], 4])\nshifts = [[\"eve\"], [], [\"fay\", \"gus\"]]\nassert([staff, size], [[\"eve\", \"fay\", \"gus\"], 3])\nshifts.shift()\nassert([staff, size], [[\"fay\", \"gus\"], 2])\nsize",
    "expected": "2"
  },
  {
    "document": "spec",
    "title": "Nucleoid flattens lists in class level rules",
    "source": "# Nucleoid flattens lists in class level rules\n\nclass Team(squads):\n    this.squads = squads\n$Team.members = $Team.squads.flatMap(v => v)\nred = Team([[\"ana\", \"bo\"], [\"cy\"]])\nblue = Team([[\"di\"]])\nassert([red.members, blue.members], [[\"ana\", \"bo\", \"cy\"], [\"di\"]])\nred.squads.push([\"ed\"])\nblue.squads = [[\"fay\"], [\"gus\", \"hal\"]]\nassert([red.members, blue.members], [[\"ana\", \"bo\", \"cy\", \"ed\"], [\"fay\", \"gus\", \"hal\"]])\nred.members.length",
    "expected": "4"
  },
  {
    "document": "spec",
    "title": "Nucleoid queries the cells of flattened grids",
    "source": "# Nucleoid queries the cells of flattened grids\n\ngrid = [[1, 0, 2], [0, 3, 2]]\ncells = grid.flatMap(v => v)\ntotal = cells.sum()\ncolors = cells.distinct()\nfilled = cells.count(cell => cell > 0)\nassert([total, colors, filled], [8, [1, 0, 2, 3], 4])\ngrid.push([5, 0, 0])\nassert([total, colors, filled], [13, [1, 0, 2, 3, 5], 5])\ngrid = [[4, 4], [0]]\n[total, colors, filled]",
    "expected": "[8,[4,0],2]"
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back failed flattening updates",
    "source": "# Nucleoid rolls back failed flattening updates\n\nrows = [[1, [2]], [3]]\ncells = rows.flatMap(v => v)\ntry:\n    rows.push([4])\n    assert(cells, [1, [2], 3, 4])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([rows, cells], [[[1, [2]], [3]], [1, [2], 3]])\ntry:\n    cells = rows.flatMap(v => v).flatMap(v => v)\n    assert(cells, [1, 2, 3])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nrows.push([[5]])\ncells",
    "expected": "[1,[2],3,[5]]"
  },
  {
    "document": "reference",
    "title": "Nucleoid flattens nested lists with flat",
    "source": "# Nucleoid flattens nested lists with flat\n\npallets = [[[\"bolt\", \"nut\"], [\"gear\"]], [[\"cog\"]]]\ncrates = pallets.flatMap(v => v)\nparts = pallets.flatMap(v => v).flatMap(v => v)\nassert(crates, [[\"bolt\", \"nut\"], [\"gear\"], [\"cog\"]])\nassert(parts, [\"bolt\", \"nut\", \"gear\", \"cog\"])\npallets.push([[\"pin\"], [\"rod\", \"nib\"]])\nassert(parts, [\"bolt\", \"nut\", \"gear\", \"cog\", \"pin\", \"rod\", \"nib\"])\ncrates.length",
    "expected": "5"
  },
  {
    "document": "readme",
    "title": "Flat turns nested lists into one live list",
    "source": "weeks = [[\"mon\", \"tue\"], [\"wed\"]]\ndays = weeks.flatMap(v => v)\nassert(days, [\"mon\", \"tue\", \"wed\"])\nweeks.push([\"thu\", \"fri\"])\nassert(days, [\"mon\", \"tue\", \"wed\", \"thu\", \"fri\"])\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Flattening keeps the dirty tiles of every room counted",
    "source": "class Room(tiles):\n    this.tiles = tiles\n$Room.floor = $Room.tiles.flatMap(v => v)\n$Room.dirty = $Room.floor.count(tile => tile == \"dirt\")\nkitchen = Room([[\"dirt\", \"clean\"], [\"clean\", \"dirt\"]])\nhall = Room([[\"clean\"]])\nassert([kitchen.dirty, hall.dirty], [2, 0])\nkitchen.tiles.push([\"dirt\", \"dirt\"])\nhall.tiles = [[\"dirt\"], [\"clean\"]]\n[kitchen.dirty, hall.dirty]\n",
    "expected": "[4,1]"
  }
]"###;

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

fn calls_flat(source: &str) -> bool {
    tokenize(source).unwrap().windows(3).any(|window| {
        window[0].token == Token::Dot
            && window[1].token == Token::Identifier("flat".to_string())
            && window[2].token == Token::LeftParen
    })
}

fn flat_calls() -> Regex {
    Regex::new(r"\.flat\((\d*)\)").unwrap()
}

fn spelled(source: &str) -> String {
    flat_calls()
        .replace_all(source, |captures: &regex::Captures<'_>| {
            match captures[1].parse::<usize>().unwrap_or(1) {
                0 => ".map(v => v)".to_string(),
                levels => ".flatMap(v => v)".repeat(levels),
            }
        })
        .into_owned()
}

fn rendered(text: &str) -> String {
    flat_calls()
        .replace_all(text, |captures: &regex::Captures<'_>| {
            match captures[1].parse::<usize>().unwrap_or(1) {
                0 => ".map(function())".to_string(),
                levels => ".flatMap(function())".repeat(levels),
            }
        })
        .into_owned()
}

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "flat regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
    value
}

fn json(runtime: &mut Runtime, source: &str) -> String {
    let value = checked(runtime, source);
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

fn failure(source: &str) -> (ErrorKind, String) {
    let error = Runtime::new().run(source).unwrap_err();
    (error.kind(), error.message())
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

fn graph_shape(runtime: &Runtime) -> String {
    let nodes: Vec<String> = runtime
        .graph
        .nodes()
        .map(|node| {
            let reads: Vec<String> = node.dependencies.iter().map(ToString::to_string).collect();
            let updates: Vec<String> = node.dependents.iter().map(ToString::to_string).collect();
            format!("{} [{}] {reads:?} {updates:?}", node.key, node.kind)
        })
        .collect();
    renumbered(&nodes.join("\n"))
}

fn behaves_alike(pairs: &[(&str, &str)]) {
    let mut flat = Runtime::new();
    let mut identity = Runtime::new();
    for (step, written) in pairs {
        assert_eq!(
            renumbered(&rendered(&outcome(&mut flat, step))),
            renumbered(&outcome(&mut identity, written)),
            "{step}"
        );
        assert_eq!(
            rendered(&graph_shape(&flat)),
            graph_shape(&identity),
            "{step}"
        );
    }
    assert_eq!(flat.assertions_run(), identity.assertions_run());
    assert!(flat.take_assertions().is_empty(), "{pairs:?}");
    assert!(identity.take_assertions().is_empty(), "{pairs:?}");
}

fn behaves_as_spelled(steps: &[&str]) {
    let written: Vec<String> = steps.iter().map(|step| spelled(step)).collect();
    assert!(
        steps
            .iter()
            .zip(&written)
            .any(|(step, written)| step != written),
        "{steps:?} call flat"
    );
    let pairs: Vec<(&str, &str)> = steps
        .iter()
        .copied()
        .zip(written.iter().map(String::as_str))
        .collect();
    behaves_alike(&pairs);
}

fn dependencies(runtime: &Runtime, name: &str) -> Vec<String> {
    runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap()
        .dependencies
        .iter()
        .map(ToString::to_string)
        .collect()
}

fn deep_chain(levels: usize) -> Value {
    let mut value = Value::Number(1.0);
    for _ in 0..levels {
        value = Value::List(vec![value]);
    }
    value
}

fn hold(runtime: &mut Runtime, items: Value) {
    let Some(Value::Object(id)) = runtime.state.variable("holder").cloned() else {
        panic!("holder is an object");
    };
    runtime
        .state
        .object_mut(&id)
        .unwrap()
        .properties
        .insert("items".to_string(), items);
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
fn only_the_appended_cases_flatten_lists_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(calls_flat(&case.source), new, "{}", case.title);
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
    assert_eq!(assertions, 30);
    assert_eq!(cases, 1105);
}

#[test]
fn the_appended_cases_stand_before_the_cases_of_nuc_98() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(titles[745..751], SPEC_TITLES);
    assert_eq!(titles[756], "Nucleoid states class predicates with is");
    assert_eq!(titles.len(), 761);
    for (cases, title) in [
        (&reference, REFERENCE_TITLE),
        (&readme, README_TITLE),
        (&examples, EXAMPLES_TITLE),
    ] {
        assert_eq!(cases[cases.len() - 3].title, title);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Reactive list flattening\n\n```\n")
        .expect("the language reference documents flattening");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 103](nuc-0103.md)"));
    assert_eq!(case.expected.as_deref(), Some("5"));
}

#[test]
fn every_case_is_its_accepted_counterpart_with_only_the_flattening_written_out() {
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
        assert_eq!(spelled(&case.source), sealed.source, "{}", sealed.title);
        rewrites += flat_calls().find_iter(&case.source).count();
        let accepted = common::assertions::protected_assertions(&parse(&sealed.source).unwrap());
        let candidate = common::assertions::protected_assertions(&parse(&case.source).unwrap());
        assert_eq!(accepted, candidate, "{}", sealed.title);
        let control = common::Case {
            title: sealed.title.clone(),
            source: sealed.source.clone(),
            expected: sealed.expected.clone(),
        };
        let mut runtimes = [Runtime::new(), Runtime::new()];
        for (runtime, form) in runtimes.iter_mut().zip([case, &control]) {
            common::check_with_count(runtime, form, accepted.len())
                .unwrap_or_else(|reason| panic!("{}: {reason}", sealed.title));
            assert_eq!(runtime.assertions_run(), accepted.len(), "{}", sealed.title);
        }
        assert_eq!(
            rendered(&graph_shape(&runtimes[0])),
            graph_shape(&runtimes[1]),
            "{}",
            sealed.title
        );
    }
    assert_eq!(rewrites, 14);
}

#[test]
fn an_omitted_depth_is_one_and_a_supplied_depth_converts_as_the_index_of_at() {
    let prelude = "class P\nprobe = P()\no = {}\ndeep = [1, [2, [3, [4, [5, [6]]]]]]";
    let levels = |depth: &str| {
        let mut runtime = Runtime::new();
        runtime.run(prelude).unwrap();
        let flattened = json(&mut runtime, &format!("deep.flat({depth})"));
        (0..7)
            .find(|levels| {
                let written = match levels {
                    0 => "deep.map(v => v)".to_string(),
                    levels => format!("deep{}", ".flatMap(v => v)".repeat(*levels)),
                };
                json(&mut runtime, &written) == flattened
            })
            .unwrap_or_else(|| panic!("{depth} gives {flattened}"))
    };
    assert_eq!(levels(""), 1);
    for (depth, expected) in [
        ("1", 1),
        ("1.9", 1),
        ("true", 1),
        ("\"1\"", 1),
        ("[1]", 1),
        ("2", 2),
        ("2.9", 2),
        ("\"2\"", 2),
        ("\" 2 \"", 2),
        ("[2]", 2),
        ("3", 3),
        ("new Date(3)", 3),
        ("0", 0),
        ("-0", 0),
        ("0.5", 0),
        ("null", 0),
        ("undefined", 0),
        ("false", 0),
        ("\"\"", 0),
        ("\" \"", 0),
        ("[]", 0),
        ("-1", 0),
        ("-0.5", 0),
        ("Number.NEGATIVE_INFINITY", 0),
        ("Number.NaN", 0),
        ("\"a\"", 0),
        ("\"0x10\"", 0),
        ("\"NaN\"", 0),
        ("[1, 2]", 0),
        ("{}", 0),
        ("probe", 0),
        ("P", 0),
        ("x => x", 0),
        ("/a/", 0),
        ("o.missing", 0),
        ("Number.POSITIVE_INFINITY", 5),
        ("1 / 0", 5),
        ("\"Infinity\"", 5),
        ("\"inf\"", 5),
        ("1e300", 5),
        ("Number.MAX_VALUE", 5),
    ] {
        assert_eq!(levels(depth), expected, "{depth}");
    }
    assert_eq!(
        result("deep = [1, [2, [3]]]\n[deep.flat(1e300), deep.flat(Number.POSITIVE_INFINITY)]"),
        "[[1.0,2.0,3.0],[1.0,2.0,3.0]]"
    );
    assert_eq!(
        result(
            "xs = [10, 20, 30]\n[xs.at(1.9), xs.at(true), xs.at(\"2\"), xs.at(null), xs.at(Number.NaN), xs.at([1])]"
        ),
        "[20.0,20.0,30.0,10.0,10.0,20.0]"
    );
}

#[test]
fn arguments_after_the_depth_are_evaluated_left_to_right_and_ignored() {
    let log = "log = []\ndef note(value):\n    log.push(value)\n    return value\n";
    assert_eq!(
        result(&format!(
            "{log}xs = [[1], [[2]]]\nflat = xs.flat(note(2), note(\"extra\"))\n[flat, log]"
        )),
        "[[1.0,2.0],[2.0,\"extra\"]]"
    );
    assert_eq!(
        failure("xs = [[1]]\nxs.flat(1, missing)"),
        (ErrorKind::Reference, "missing is not defined".to_string())
    );
    assert_eq!(
        failure("xs = [[1]]\nxs.flat(missing, 1)"),
        (ErrorKind::Reference, "missing is not defined".to_string())
    );
    behaves_alike(&[
        ("xs = [[1], [[2]]]", "xs = [[1], [[2]]]"),
        ("ys = xs.flat(1, 2, 3)", "ys = xs.flatMap(v => v, 2, 3)"),
        ("xs.push([3])", "xs.push([3])"),
        ("ys", "ys"),
    ]);
}

#[test]
fn flattening_keeps_order_duplicates_identities_and_non_list_items() {
    assert_eq!(
        result("[[1, 2], 2, [[3, [4]]], [], \"ab\", true, null, [[]], [null]].flat()"),
        "[1.0,2.0,2.0,[3.0,[4.0]],\"ab\",true,null,[],null]"
    );
    assert_eq!(
        result(
            "a = {k: 1}\nys = [[a], [a, [a]]].flat(2)\nys[0].k = 5\n[a.k, ys.length, ys[2] == a, ys[0] == ys[1]]"
        ),
        "[5.0,3.0,true,true]"
    );
    let kinds = "f = x => x * 2\nd = new Date(5)\nr = /a+/\nys = [[f], [d], [r], [\"\"], [0]].flat()\ng = ys[0]\n[g(7), ys[1] == d, ys[2].test(\"caab\"), typeof ys[0], ys.length]";
    assert_eq!(result(kinds), "[14.0,true,true,\"Function\",5.0]");
    behaves_as_spelled(&[kinds]);
    assert_eq!(
        result("xs = [[1, 2], [3]]\nys = xs.flat(5)\n[xs, ys]"),
        "[[[1.0,2.0],[3.0]],[1.0,2.0,3.0]]"
    );
    assert_eq!(result("[].flat()"), "[]");
    assert_eq!(
        result("[[], [[]], [[[]]]].flat(Number.POSITIVE_INFINITY)"),
        "[]"
    );
}

#[test]
fn null_items_settle_declarations_exactly_as_the_identity_callback_does() {
    for (source, expected) in [
        ("cells = [[1], null].flat()\ncells", "null"),
        ("cells = [[1, null]].flat()\ncells", "[1.0,null]"),
        ("cells = [[null]].flat()\ncells", "[null]"),
        ("cells = [[null]].flat(2)\ncells", "null"),
        ("cells = [1, null].flat(0)\ncells", "null"),
        (
            "cells = [[null]].flat(Number.POSITIVE_INFINITY)\ncells",
            "null",
        ),
        ("[[1], null].flat()", "[1.0,null]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
        assert_eq!(result(&spelled(source)), expected, "{source}");
    }
    behaves_as_spelled(&[
        "rows = [[1], null]",
        "cells = rows.flat()",
        "count = rows.flat().length",
        "box = {}",
        "box.cells = rows.flat()",
        "class Holder(rows):\n    this.rows = rows",
        "$Holder.cells = $Holder.rows.flat()",
        "holder = Holder([[2], null])",
        "def spread(items) => items.flat()",
        "returned = spread([[3], null])",
        "listed = [rows.flat(), 1]",
        "if rows.flat().length > 1:\n    big = true",
        "[cells, count, box.cells, holder.cells, returned, listed, big]",
        "rows.flat()",
        "assert(rows.flat(), [1, null])",
        "rows = [[1], [null]]",
        "[cells, count, box.cells, listed]",
        "rows = [[4], null]",
        "deeper = rows.flat(2)",
        "copy = rows.flat(0)",
        "[cells, deeper, copy]",
    ]);
}

#[test]
fn declarations_follow_their_receivers_depths_and_populations() {
    behaves_as_spelled(&[
        "xs = [[1, 2], [3]]",
        "ys = xs.flat()",
        "n = xs.flat().length",
        "xs.push([4, [5]])",
        "[ys, n]",
        "xs.pop()",
        "xs.shift()",
        "[ys, n]",
        "xs = [[6], [], [7, 8]]",
        "[ys, n]",
        "o = {rows: [[1], [2, [3]]]}",
        "zs = o.rows.flat(2)",
        "o.rows.push([[4]])",
        "o.rows = [[[5]]]",
        "zs",
        "a = {items: [1, 2]}",
        "b = {items: [3]}",
        "rows = [a.items, b.items]",
        "cells = rows.flat()",
        "a.items.push(9)",
        "cells",
    ]);
    assert_eq!(
        result("d = 1\nxs = [[1], [[2]]]\nys = xs.flat(d)\nfirst = ys.value\nd = 2\n[first, ys]"),
        "[[1.0,[2.0]],[1.0,2.0]]"
    );
    assert_eq!(
        result("xs = [[1], [2]]\nys = xs.flat()\nxs[0].push(9)\n[xs, ys]"),
        "[[[1.0],[2.0]],[1.0,2.0]]"
    );
    behaves_as_spelled(&[
        "xs = [[1], [2]]",
        "ys = xs.flat()",
        "xs[0].push(9)",
        "[xs, ys]",
    ]);
}

#[test]
fn populations_flatten_to_their_instance_lists_at_every_depth() {
    behaves_as_spelled(&[
        "class Item(tags):\n    this.tags = tags",
        "first = Item([\"a\"])",
        "items = Item.flat()",
        "deeper = Item.flat(3)",
        "copy = Item.flat(0)",
        "second = Item([\"b\", \"c\"])",
        "[items.length, deeper.length, copy.length, items[1] == second]",
        "Item.flat().map(item => item.tags)",
    ]);
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Item\nfirst = Item()\nitems = Item.flat()\nevery = Item.flat(Number.POSITIVE_INFINITY)\nsecond = Item()\nassert(items, Item.list())\nassert(every, [first, second])",
    );
    assert_eq!(dependencies(&runtime, "items"), ["$Item"]);
    assert_eq!(result("class Batch\nempty = Batch.flat()\nempty"), "[]");
}

#[test]
fn members_slices_calls_and_state_versions_are_receivers() {
    behaves_as_spelled(&[
        "xs = [[1], [2]]",
        "xs.push([3, [4]])",
        "versions = xs@first.flat()",
        "sliced = xs[1:].flat(2)",
        "def layers() => [[5], [[6]]]",
        "called = layers().flat()",
        "o = {inner: {rows: [[7], [8]]}}",
        "member = o.inner.rows.flat()",
        "[versions, sliced, called, member]",
        "class Grid(rows):\n    this.rows = rows\n    def cells():\n        return this.rows.flat()",
        "g = Grid([[1, 2], [3]])",
        "g.cells()",
    ]);
    assert_eq!(
        result("xs = [[1], [2]]\nxs.push([3])\nxs@first.flat()"),
        "[1.0,2.0]"
    );
}

#[test]
fn rendering_and_dependencies_name_the_call_as_written() {
    let mut runtime = Runtime::new();
    assert_eq!(
        json(
            &mut runtime,
            "rows = [[1, 2], [3]]\nd = 2\ncells = rows.flat()\ndeep = rows.flat(d)\n(why cells).map(step -> step.rule)"
        ),
        "[\"cells = rows.flat()\",\"rows = [[1,2],[3]]\"]"
    );
    assert_eq!(
        json(&mut runtime, "(why deep).map(step -> step.rule)"),
        "[\"deep = rows.flat(d)\",\"rows = [[1,2],[3]]\",\"d = 2\"]"
    );
    assert_eq!(
        json(&mut runtime, "(affects rows).map(step -> step.node)"),
        "[\"cells\",\"deep\"]"
    );
    assert_eq!(dependencies(&runtime, "cells"), ["rows"]);
    assert_eq!(dependencies(&runtime, "deep"), ["rows", "d"]);
    let mut spelled_runtime = Runtime::new();
    spelled_runtime
        .run("rows = [[1]]\ncells = rows.flatMap(v => v)")
        .unwrap();
    assert_eq!(
        json(&mut spelled_runtime, "(why cells).map(step -> step.rule)"),
        "[\"cells = rows.flatMap(function())\",\"rows = [[1]]\"]"
    );
}

#[test]
fn failed_statements_and_transactions_restore_flattened_declarations() {
    behaves_as_spelled(&[
        "rows = [[1, [2]], [3]]",
        "cells = rows.flat()",
        "try:\n    rows.push([4])\n    assert(cells, [1, [2], 3, 4])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
        "[rows, cells]",
        "try:\n    cells = rows.flat(2)\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
        "rows.push([[5]])",
        "cells",
    ]);
    let mut runtime = Runtime::new();
    runtime
        .run("rows = [[1], [2]]\ncells = rows.flat()")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "cells = rows.flat(missing)"),
        "ReferenceError: missing is not defined at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(json(&mut runtime, "rows.push([3])\ncells"), "[1.0,2.0,3.0]");
    assert_eq!(dependencies(&runtime, "cells"), ["rows"]);
}

#[test]
fn unchanged_readings_keep_their_accepted_results() {
    for (source, expected) in [
        ("xs = [[1]]\nxs.flat", "null"),
        ("class P\nP.flat", "null"),
        ("o = {flat: 3}\no.flat", "3.0"),
        ("o = {flat: x => [x, x]}\no.flat(2)", "[2.0,2.0]"),
        ("class P\np = P()\np.flat()", "null"),
        (
            "class P(n):\n    this.n = n\n    def flat():\n        return [this.n]\np = P(3)\np.flat()",
            "[3.0]",
        ),
        ("x = null\nx.flat()", "null"),
        ("x = null\ny = x.flat()\ny", "null"),
        ("o = {}\no.missing.flat()", "null"),
        ("def flat(x) => x\nflat([[1]])", "[[1.0]]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, kind, message) in [
        (
            "xs = [[1]]\nxs.flat(missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "xs = [[1]]\nxs.flat(Infinity)",
            ErrorKind::Reference,
            "Infinity is not defined",
        ),
        (
            "\"ab\".flat()",
            ErrorKind::Type,
            "ab.flat is not a function",
        ),
        (
            "(5).flat()",
            ErrorKind::Type,
            "number.flat is not a function",
        ),
        (
            "true.flat()",
            ErrorKind::Type,
            "boolean.flat is not a function",
        ),
        (
            "d = new Date(5)\nd.flat()",
            ErrorKind::Type,
            "Date.flat is not a function",
        ),
        ("/a/.flat()", ErrorKind::Type, "flat is not a function"),
        (
            "f = x => x\nf.flat()",
            ErrorKind::Type,
            "function.flat is not a function",
        ),
        (
            "Math.flat()",
            ErrorKind::Type,
            "Math.flat is not a function",
        ),
        (
            "Number.flat()",
            ErrorKind::Type,
            "Number.flat is not a function",
        ),
        (
            "xs = [[1]]\nxs.flatten()",
            ErrorKind::Type,
            "xs.flatten is not a function",
        ),
    ] {
        assert_eq!(failure(source), (kind, message.to_string()), "{source}");
    }
}

#[test]
fn deep_lists_flatten_end_to_end_through_the_public_api() {
    on_large_stack(|| {
        let mut runtime = Runtime::new();
        runtime.run("holder = {items: 0}").unwrap();
        hold(&mut runtime, Value::List(vec![deep_chain(DEEP)]));
        assert_eq!(
            json(
                &mut runtime,
                "cells = holder.items.flat(Number.POSITIVE_INFINITY)\ncells"
            ),
            "[1.0]"
        );
        assert_eq!(json(&mut runtime, "holder.items.flat(24999).length"), "1.0");
        assert_eq!(json(&mut runtime, "holder.items.flat(0).length"), "1.0");
        hold(
            &mut runtime,
            Value::List((0..DEEP).map(|_| deep_chain(3)).collect()),
        );
        assert_eq!(
            json(
                &mut runtime,
                "holder.items.flat(Number.POSITIVE_INFINITY).length"
            ),
            "25000.0"
        );
    });
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-list-flattening-{label}-{}.nuc",
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

#[test]
fn files_graphs_and_repl_sessions_run_flat_as_its_flat_map_spelling() {
    let program = "rows = [[1, 2], [3, [4]]]\ncells = rows.flat()\ndeep = rows.flat(2)\nrows.push([5])\n[cells, deep]";
    let file = SourceFile::new("file");
    for options in [&["--json"][..], &[][..]] {
        let flat = file.run(program, options);
        assert!(flat.status.success(), "{flat:?}");
        assert!(flat.stderr.is_empty(), "{flat:?}");
        let written = file.run(&spelled(program), options);
        assert_eq!(flat.stdout, written.stdout, "{options:?}");
    }
    assert_eq!(
        String::from_utf8(file.run(program, &["--json"]).stdout).unwrap(),
        "[[1.0,2.0,3.0,[4.0],5.0],[1.0,2.0,3.0,4.0,5.0]]\n"
    );
    let graph = String::from_utf8(file.run(program, &["--json", "--graph"]).stdout).unwrap();
    let written_graph =
        String::from_utf8(file.run(&spelled(program), &["--json", "--graph"]).stdout).unwrap();
    assert_eq!(rendered(&graph), written_graph);
    assert!(
        graph.contains(
            "  rows [variable]\n    updates  cells, deep\n  cells [variable]\n    reads    rows\n"
        ),
        "{graph}"
    );
    let failed = file.run("xs = [[1]]\nxs.flat(missing)", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "ReferenceError: missing is not defined (line 2, column 1)\n"
    );
    let session = "rows = [[1], [2, [3]]]\ncells = rows.flat()\ncells\nrows.push([4])\ncells\nrows.flat(2)\nrows.flat(missing)\n.clear\nrows = [[5]]\nrows.flat()\n.exit\n";
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let options: &[&str] = if plain { &["--plain"] } else { &[] };
            let flat = repl(&session.replace('\n', newline), options);
            let written = repl(&spelled(session).replace('\n', newline), options);
            assert!(flat.status.success(), "{flat:?}");
            assert_eq!(flat.stdout, written.stdout, "{plain} {newline:?}");
            assert_eq!(flat.stderr, written.stderr, "{plain} {newline:?}");
            let stdout = String::from_utf8(flat.stdout).unwrap();
            assert!(
                stdout.contains("[1.0,2.0,[3.0]]\n3.0\n[1.0,2.0,[3.0],4.0]\n[1.0,2.0,3.0,4.0]\n"),
                "{stdout}"
            );
            assert!(stdout.contains("[5.0]\n"), "{stdout}");
            assert_eq!(
                String::from_utf8(flat.stderr).unwrap(),
                "ReferenceError: missing is not defined (line 1, column 1)\n"
            );
        }
    }
}
