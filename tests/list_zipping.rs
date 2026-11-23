mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime, Value};
use serde::Deserialize;

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid pairs the items of two lists by position with zip",
    "Nucleoid pairs the instances of two classes in population order with zip",
    "Nucleoid combines the items of two lists with a function in zip",
    "Nucleoid rolls back failed zip updates",
];
const REFERENCE_TITLE: &str = "Nucleoid zips lists and populations with zip";
const README_TITLE: &str = "Zip weighs every grade by its credits for a live grade point average";
const EXAMPLES_TITLE: &str = "Zip measures every climb between the checkpoints of a trail";

const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid pairs the items of two lists by position with zip",
    "source": "# Nucleoid pairs the items of two lists by position with zip\n\nstops = [\"Harbor\", \"Mill\", \"Oak\"]\nminutes = [0, 4, 9]\ntimetable = [[stop, minutes[stops.indexOf(stop)]] for stop in stops[:minutes.length]]\nassert(timetable, [[\"Harbor\", 0], [\"Mill\", 4], [\"Oak\", 9]])\nstops.push(\"Park\")\nassert(timetable.length, 3)\nminutes.push(15)\nminutes.push(22)\nassert(timetable, [[\"Harbor\", 0], [\"Mill\", 4], [\"Oak\", 9], [\"Park\", 15]])\nstops.shift()\ntimetable[0]",
    "expected": "[\"Mill\",0]"
  },
  {
    "document": "spec",
    "title": "Nucleoid pairs the instances of two classes in population order with zip",
    "source": "# Nucleoid pairs the instances of two classes in population order with zip\n\nclass Driver(name):\n    this.name = name\nclass Truck(plate):\n    this.plate = plate\nada = Driver(\"Ada\")\nDriver(\"Bo\")\nTruck(\"T-1\")\ncrews = [[driver, Truck[Driver.indexOf(driver)]] for driver in Driver[:Truck.length]]\nassert([crews.length, crews[0][0], crews[0][1].plate], [1, ada, \"T-1\"])\nTruck(\"T-2\")\nTruck(\"T-3\")\nassert(crews.map(crew -> crew[0].name + \":\" + crew[1].plate), [\"Ada:T-1\", \"Bo:T-2\"])\nDriver(\"Cy\")\ncrews.map(crew -> crew[1].plate)",
    "expected": "[\"T-1\",\"T-2\",\"T-3\"]"
  },
  {
    "document": "spec",
    "title": "Nucleoid combines the items of two lists with a function in zip",
    "source": "# Nucleoid combines the items of two lists with a function in zip\n\nprices = [4, 4, 2]\nquantities = [3, 1, 3]\ndiscount = 0\ntotal = prices[:quantities.length].reduce(lambda lines, price: lines.concat((price - discount) * quantities[lines.length]), []).sum()\nassert(total, 22)\nprices.push(5)\nassert(total, 22)\nquantities.push(2)\nquantities.push(9)\nassert(total, 32)\ndiscount = 1\ntotal",
    "expected": "23"
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back failed zip updates",
    "source": "# Nucleoid rolls back failed zip updates\n\ndef gap(load, limit):\n    if load > limit:\n        throw \"OVERLOAD\"\n    return limit - load\nlimits = [4, 8, 6]\nloads = [3, 5]\nspare = [gap(load, limits[loads.indexOf(load)]) for load in loads]\nfailure = \"\"\ntry:\n    loads.push(9)\ncatch error:\n    failure = error\nassert([failure, loads, spare], [\"OVERLOAD\", [3, 5], [1, 3]])\ntry:\n    loads.push(2)\n    assert(spare, [1, 3, 4])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([loads, spare], [[3, 5], [1, 3]])\nclass Bay(load): this.load = load\nBay(1)\nBay(7)\nroom = [gap(bay.load, limits[Bay.indexOf(bay)]) for bay in Bay[:limits.length]]\nfailure = \"\"\ntry:\n    Bay(8)\ncatch error:\n    failure = error\nassert([failure, Bay.length, room], [\"OVERLOAD\", 2, [3, 1]])\nBay(6)\nBay(5)\nroom",
    "expected": "[3,1,0]"
  },
  {
    "document": "reference",
    "title": "Nucleoid zips lists and populations with zip",
    "source": "# Nucleoid zips lists and populations with zip\n\nrunners = [\"ana\", \"bo\", \"cy\"]\nbibs = [7, 9]\nentries = [[runners[bibs.indexOf(bib)], bib] for bib in bibs]\nclass Lap(seconds):\n    this.seconds = seconds\nLap(62)\nLap(58)\ntargets = [60, 60]\ngaps = [lap.seconds - targets[Lap.indexOf(lap)] for lap in Lap[:targets.length]]\nassert([entries, gaps], [[[\"ana\", 7], [\"bo\", 9]], [2, -2]])\nbibs.push(8)\nLap(61)\nassert([entries.length, gaps], [3, [2, -2]])\ntargets.push(60)\ngaps",
    "expected": "[2,-2,1]"
  },
  {
    "document": "readme",
    "title": "Zip weighs every grade by its credits for a live grade point average",
    "source": "grades = [4, 4, 3]\ncredits = [3, 2, 3]\ngpa = grades.reduce(lambda points, grade: points.concat(grade * credits[points.length]), []).sum() / credits.sum()\nassert(gpa, 3.625)\ngrades.push(2)\ncredits.push(4)\nassert(gpa, 3.0833333333333335)\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Zip measures every climb between the checkpoints of a trail",
    "source": "class Trail(heights):\n    this.heights = heights\n$Trail.climbs = .heights[1:].reduce(lambda climbs, high: climbs.concat(high - .heights[climbs.length]), [])\n$Trail.steep = $Trail.climbs.max() > 100\nridge = Trail([120, 180, 150, 240])\ndale = Trail([40, 60, 60, 80])\nassert([ridge.climbs, ridge.steep, dale.climbs], [[60, -30, 90], false, [20, 0, 20]])\nridge.heights.push(390)\ndale.heights.push(200)\n[ridge.climbs, ridge.steep, dale.steep]\n",
    "expected": "[[60,-30,90,150],true,true]"
  }
]"###;

const WRITTEN_OUT: [(&str, &str); 9] = [
    (
        "stops.zip(minutes)",
        "[[stop, minutes[stops.indexOf(stop)]] for stop in stops[:minutes.length]]",
    ),
    (
        "Driver.zip(Truck)",
        "[[driver, Truck[Driver.indexOf(driver)]] for driver in Driver[:Truck.length]]",
    ),
    (
        "prices.zip(quantities, (price, quantity) -> (price - discount) * quantity)",
        "prices[:quantities.length].reduce(lambda lines, price: lines.concat((price - discount) * quantities[lines.length]), [])",
    ),
    (
        "loads.zip(limits, gap)",
        "[gap(load, limits[loads.indexOf(load)]) for load in loads]",
    ),
    (
        "Bay.zip(limits, (bay, limit) -> gap(bay.load, limit))",
        "[gap(bay.load, limits[Bay.indexOf(bay)]) for bay in Bay[:limits.length]]",
    ),
    (
        "runners.zip(bibs)",
        "[[runners[bibs.indexOf(bib)], bib] for bib in bibs]",
    ),
    (
        "Lap.zip(targets, (lap, target) -> lap.seconds - target)",
        "[lap.seconds - targets[Lap.indexOf(lap)] for lap in Lap[:targets.length]]",
    ),
    (
        "grades.zip(credits, (grade, credit) -> grade * credit)",
        "grades.reduce(lambda points, grade: points.concat(grade * credits[points.length]), [])",
    ),
    (
        "$Trail.heights.zip($Trail.heights[1:], (low, high) -> high - low)",
        ".heights[1:].reduce(lambda climbs, high: climbs.concat(high - .heights[climbs.length]), [])",
    ),
];

const DRIVERS: &str =
    "class Driver(name):\n    this.name = name\nDriver(\"ada\")\nDriver(\"bo\")\nDriver(\"cy\")\n";
const TRUCKS: &str = "class Truck(plate):\n    this.plate = plate\nTruck(\"T1\")\nTruck(\"T2\")\n";
const TEAM: &str = "class Team(name):\n    this.name = name\n    def zip(other): return this.name + other\nred = Team(\"red\")\n";
const CALCULATOR: &str = "class Calc(base):\n    this.base = base\n    def zip(a, b) => this.base + a * b\nc = Calc(1)\n";

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

fn zip_calls(source: &str) -> usize {
    let tokens: Vec<Token> = tokenize(source)
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    tokens
        .windows(3)
        .filter(|window| {
            matches!(window, [Token::Dot, Token::Identifier(name), Token::LeftParen] if name == "zip")
        })
        .count()
}

fn written_out(case: &common::Case) -> (String, usize) {
    let mut written = case.source.clone();
    let mut rewrites = 0;
    for (call, spelling) in WRITTEN_OUT {
        rewrites += written.matches(call).count();
        written = written.replace(call, spelling);
    }
    (written, rewrites)
}

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "zip regression".to_string(),
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

fn dependencies(runtime: &Runtime, name: &str) -> Vec<String> {
    let mut found: Vec<String> = runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap()
        .dependencies
        .iter()
        .map(|key| renumbered(&key.to_string()))
        .collect();
    found.sort();
    found
}

fn behaves_as_spelled(steps: &[(&str, &str)], names: &[&str]) {
    let mut measured = Runtime::new();
    let mut spelled = Runtime::new();
    for (step, spelling) in steps {
        let before = measured.assertions_run();
        assert_eq!(
            renumbered(&outcome(&mut measured, step)),
            renumbered(&outcome(&mut spelled, spelling)),
            "{step}"
        );
        assert_eq!(
            measured.assertions_run() - before,
            common::assertions::protected_assertions(&parse(step).unwrap()).len(),
            "{step}"
        );
        for name in names {
            if measured.graph.retrieve(&NodeKey::variable(*name)).is_some() {
                assert_eq!(
                    dependencies(&measured, name),
                    dependencies(&spelled, name),
                    "{step}: {name}"
                );
            }
        }
    }
    assert_eq!(measured.assertions_run(), spelled.assertions_run());
    assert!(measured.take_assertions().is_empty(), "{steps:?}");
    assert!(spelled.take_assertions().is_empty(), "{steps:?}");
}

fn spelled_pairs(list: &str, other: &str) -> String {
    format!(
        "{list}[:{other}.length].reduce((acc, x) -> acc.concat([[x, {other}[acc.length]]]), [])"
    )
}

fn spelled_combined(list: &str, other: &str, combine: &str) -> String {
    format!(
        "{list}[:{other}.length].reduce((acc, x) -> acc.concat([({combine})(x, {other}[acc.length])]), [])"
    )
}

fn hold(runtime: &mut Runtime, name: &str, items: Value) {
    let Some(Value::Object(id)) = runtime.state.variable("holder").cloned() else {
        panic!("holder is an object");
    };
    runtime
        .state
        .object_mut(&id)
        .unwrap()
        .properties
        .insert(name.to_string(), items);
}

#[test]
fn only_the_appended_cases_zip_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    let mut calls = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        let found = zip_calls(&case.source);
        assert_eq!(found > 0, new, "{}", case.title);
        calls += found;
        if new {
            let count =
                common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            let mut runtime = Runtime::new();
            common::check_with_count(&mut runtime, case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assert_eq!(runtime.assertions_run(), count, "{}", case.title);
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
    assert_eq!(calls, 9);
    assert_eq!(assertions, 18);
    assert_eq!(cases, 1299);
}

#[test]
fn the_audit_counts_zip_calls_and_no_reads_or_writes_of_a_zip_member() {
    for (source, expected) in [
        ("xs.zip(ys)", 1),
        ("xs.zip(ys).zip(zs)", 2),
        ("Driver.zip(Truck, (d, t) -> d.zip)", 1),
        ("zips = shipments.map(.to.zip)", 0),
        ("bergen.to.zip = \"5003\"", 0),
        ("ada.zip = \"69001\"", 0),
        ("lamp.label.zip", 0),
        ("zip(xs, ys)", 0),
        ("zip = [1]\nzip.length", 0),
        ("\"xs.zip(ys)\"", 0),
        ("# xs.zip(ys)\n1", 0),
    ] {
        assert_eq!(zip_calls(source), expected, "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_126_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[819],
        "Nucleoid rolls back a repeat-until loop that fails during an iteration"
    );
    assert_eq!(titles[824..828], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid repeats statements until a condition holds",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Repeat-until loops move a lift to the next call",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Repeat-until loops give every ferry enough lifeboats",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 10 - 2 * usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 2].title, previous);
        assert_eq!(cases[index + 3].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### List and population zipping\n\n```\n")
        .expect("the language reference documents zip");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("`zip(other)` and `zip(other, combine)` ([NUC 129](nuc-0129.md))"));
    assert!(reference.contains("`zip expects a list`"));
    assert!(reference.contains("`zip expects a function`"));
    assert!(reference.contains(
        "[list and population zipping](../nucleoid.spec.md#list-and-population-zipping)"
    ));
    assert_eq!(case.expected.as_deref(), Some("[2,-2,1]"));
}

#[test]
fn every_case_is_its_accepted_counterpart_with_only_the_zip_calls_written_out() {
    let sealed: Vec<SealedCase> = serde_json::from_str(ACCEPTED).unwrap();
    let documents = documents();
    assert_eq!(sealed.len(), 7);
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
        let (written, count) = written_out(case);
        assert_eq!(written, sealed.source, "{}", sealed.title);
        assert_eq!(zip_calls(&written), 0, "{}", sealed.title);
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
    assert_eq!(rewrites, 9);
}

#[test]
fn pairs_hold_the_items_at_each_position_up_to_the_shorter_sequence() {
    for (source, expected) in [
        (
            "[1, 2, 3].zip([4, 5, 6])",
            "[[1.0,4.0],[2.0,5.0],[3.0,6.0]]",
        ),
        ("[1, 2, 3].zip([\"a\", \"b\"])", "[[1.0,\"a\"],[2.0,\"b\"]]"),
        ("[1].zip([4, 5, 6])", "[[1.0,4.0]]"),
        ("[[].zip([1]), [1].zip([]), [].zip([])]", "[[],[],[]]"),
        ("[7].zip([8])", "[[7.0,8.0]]"),
        (
            "[[1, 2], [3]].zip([[4], [5, 6]])",
            "[[[1.0,2.0],[4.0]],[[3.0],[5.0,6.0]]]",
        ),
        ("[null, 1].zip([2, undefined])", "[[null,2.0],[1.0,null]]"),
        (
            "o = {}\n[o.missing, 1].zip([3, 4])",
            "[[null,3.0],[1.0,4.0]]",
        ),
        (
            "[true, \"x\"].zip([false, \"\"])",
            "[[true,false],[\"x\",\"\"]]",
        ),
        (
            "a = {k: 1}\nb = {k: 2}\npairs = [a].zip([b])\n[pairs[0][0] == a, pairs[0][1] == b, pairs[0][0].k]",
            "[true,true,1.0]",
        ),
        (
            "xs = [1, 2]\npairs = xs.zip(xs)\n[pairs, xs]",
            "[[[1.0,1.0],[2.0,2.0]],[1.0,2.0]]",
        ),
        (
            "xs = [3, 1]\nys = [4, 2]\npairs = xs.zip(ys)\n[pairs, xs, ys]",
            "[[[3.0,4.0],[1.0,2.0]],[3.0,1.0],[4.0,2.0]]",
        ),
        (
            "[1, 2].zip([3, 4]).zip([5, 6])",
            "[[[1.0,3.0],5.0],[[2.0,4.0],6.0]]",
        ),
        ("[1, 2].zip([3, 4]).map(p -> p[0] * p[1])", "[3.0,8.0]"),
        ("[1, 2].zip([3, 4]).length", "2.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn lists_however_they_are_reached_are_receivers_and_partners() {
    for (source, expected) in [
        (
            "xs = [1, 2]\nys = [3, 4]\nxs.zip(ys)",
            "[[1.0,3.0],[2.0,4.0]]",
        ),
        (
            "o = {xs: [1, 2], ys: [3, 4]}\no.xs.zip(o.ys)",
            "[[1.0,3.0],[2.0,4.0]]",
        ),
        (
            "o = {inner: {items: [1, 2]}}\no.inner.items.zip([5])",
            "[[1.0,5.0]]",
        ),
        (
            "xs = [1, 2, 3, 4, 5, 6]\nxs[::2].zip(xs[1::2])",
            "[[1.0,2.0],[3.0,4.0],[5.0,6.0]]",
        ),
        (
            "xs = [1, 4, 9, 16]\nxs.zip(xs[1:], (a, b) -> b - a)",
            "[3.0,5.0,7.0]",
        ),
        ("xs = [1, 2]\nxs.value.zip([3])", "[[1.0,3.0]]"),
        ("xs = [1, 2]\nxs.push(5)\nxs@first.zip([3])", "[[1.0,3.0]]"),
        ("grid = [[1, 2], [3]]\ngrid[0].zip(grid[1])", "[[1.0,3.0]]"),
        (
            "def make() => [1, 2]\nmake().zip([3, 4])",
            "[[1.0,3.0],[2.0,4.0]]",
        ),
        (
            "xs = [1, 2, 3]\nxs.sortBy(x -> -x).zip(xs)",
            "[[3.0,1.0],[2.0,2.0],[1.0,3.0]]",
        ),
        (
            "xs = [1, 2, 3]\nxs.slice(0, 2).zip(xs)",
            "[[1.0,1.0],[2.0,2.0]]",
        ),
        (
            "xs = [1, 2, 3]\nxs.concat([4]).zip(xs)",
            "[[1.0,1.0],[2.0,2.0],[3.0,3.0]]",
        ),
        (
            "xs = [1, 5, 2]\nxs.filter(x -> x > 1).zip(xs)",
            "[[5.0,1.0],[2.0,5.0]]",
        ),
        (
            "xs = [1, 2]\nxs.map(x -> x * 2).zip(xs)",
            "[[2.0,1.0],[4.0,2.0]]",
        ),
        ("xs = [1, 2]\nkind = xs\nkind.zip([3])", "[[1.0,3.0]]"),
        ("zip = [1, 2]\nzip.zip([3, 4])", "[[1.0,3.0],[2.0,4.0]]"),
        ("Order = [2, 6]\nOrder.zip([1])", "[[2.0,1.0]]"),
        (
            "def pairs(xs, ys):\n    return xs.zip(ys)\npairs([1, 2], [3, 4])",
            "[[1.0,3.0],[2.0,4.0]]",
        ),
        (
            "def pairs(xs, ys) => xs.zip(ys)\npairs([1], [2])",
            "[[1.0,2.0]]",
        ),
        (
            "class Team(names, shirts):\n    this.names = names\n    this.shirts = shirts\n    def roster():\n        return this.names.zip(this.shirts)\nt = Team([\"ana\", \"bo\"], [7, 9, 11])\nt.roster()",
            "[[\"ana\",7.0],[\"bo\",9.0]]",
        ),
        (
            "rows = [[1, 2], [3, 4]]\nrows.map(row -> row.zip([10, 20], (a, b) -> a * b))",
            "[[10.0,40.0],[30.0,80.0]]",
        ),
        (
            "xs = [1, 2]\nys = [3, 4]\n[xs.zip(ys)[1], xs.zip(ys)[1][0]]",
            "[[2.0,4.0],2.0]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn populations_are_receivers_and_partners_in_population_order() {
    let mut runtime = Runtime::new();
    runtime
        .run(&format!("{DRIVERS}{TRUCKS}kind = Driver"))
        .unwrap();
    for (source, expected) in [
        (
            "Driver.zip(Truck).map(p -> p[0].name + p[1].plate)",
            "[\"adaT1\",\"boT2\"]",
        ),
        (
            "[1, 2, 3].zip(Truck).map(p -> p[1].plate)",
            "[\"T1\",\"T2\"]",
        ),
        (
            "Driver.zip([10, 20, 30, 40]).map(p -> p[1])",
            "[10.0,20.0,30.0]",
        ),
        (
            "Driver.zip(Driver).map(p -> p[0] == p[1])",
            "[true,true,true]",
        ),
        ("kind.zip([1]).length", "1.0"),
        ("$Driver.zip([1]).length", "1.0"),
        ("Driver[1:].zip([1])[0][0].name", "\"bo\""),
        ("Driver.list().zip([1]).length", "1.0"),
        ("Driver.filter(d -> true).zip([1]).length", "1.0"),
        (
            "Driver.map(d -> d.name).zip(Truck.map(t -> t.plate))",
            "[[\"ada\",\"T1\"],[\"bo\",\"T2\"]]",
        ),
        (
            "Truck.zip(Driver, (t, d) -> d.name + t.plate)",
            "[\"adaT1\",\"boT2\"]",
        ),
        ("Driver.zip(kind).length", "3.0"),
        ("Truck.zip(Driver)[1][1] == Driver[1]", "true"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    for (source, expected) in [
        ("class Empty\n[Empty.zip([1]), [1].zip(Empty)]", "[[],[]]"),
        (
            &format!("{TEAM}[Team.zip([1]).length, Team.zip([1])[0][0] == red, red.zip(\"!\")]"),
            "[1.0,true,\"red!\"]",
        ),
        (
            &format!("{CALCULATOR}[c.zip(2, 3), Calc.zip([4], (calc, n) -> calc.zip(n, 2))]"),
            "[7.0,[9.0]]",
        ),
        (
            "class A:\n    def init(n):\n        this.n = n\nclass B extends A:\n    def init(n):\n        super(n)\nA(2)\nB(7)\nA(4)\n[A.zip(B).map(p -> [p[0].n, p[1].n]), B.zip(A).length]",
            "[[[2.0,7.0]],1.0]",
        ),
        (
            "class Math(n):\n    this.n = n\nMath(7)\n[1, 2].zip(Math).map(p -> p[1].n)",
            "[7.0]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn combiners_run_once_per_position_in_order_with_both_items() {
    for (source, expected) in [
        (
            "seen = []\ndef log(a, b):\n    seen.push([a, b])\n    return a + b\n[[1, 2, 3].zip([10, 20], log), seen]",
            "[[11.0,22.0],[[1.0,10.0],[2.0,20.0]]]",
        ),
        (
            "seen = []\ndef log(a, b):\n    seen.push([a, b])\n    return a\n[[1].zip([10, 20, 30], log), seen]",
            "[[1.0],[[1.0,10.0]]]",
        ),
        (
            "seen = []\ndef log(a, b):\n    seen.push(1)\n    return a\n[[].zip([1, 2], log), [1, 2].zip([], log), seen]",
            "[[],[],[]]",
        ),
        (
            "[1, 2, 3].zip([4, 5, 6], (a, b) -> a * b)",
            "[4.0,10.0,18.0]",
        ),
        ("[3, 5].zip([4, 5], (a, b) -> a < b)", "[true,false]"),
        ("[1, 2].zip([3, 4], (a) -> a)", "[1.0,2.0]"),
        ("[1, 2].zip([3, 4], (a, b, c) -> c)", "[null,null]"),
        ("[1, 2].zip([3, 4], (a, b, c) -> c ?? b)", "[3.0,4.0]"),
        ("[1, 2].zip([3, 4], () -> 0)", "[0.0,0.0]"),
        ("[1, 2].zip([3, 4], lambda a, b: a - b)", "[-2.0,-2.0]"),
        ("[1, 2].zip([3, 4], (a, b) => a + b)", "[4.0,6.0]"),
        (
            "def plus(a, b) => a + b\n[1, 2].zip([3, 4], plus)",
            "[4.0,6.0]",
        ),
        (
            "plus = (a, b) -> a + b\n[1, 2].zip([3, 4], plus)",
            "[4.0,6.0]",
        ),
        (
            "[1, 2].zip([3, 4], (a, b) -> [a, b, a + b])",
            "[[1.0,3.0,4.0],[2.0,4.0,6.0]]",
        ),
        ("[1, 2].zip([3, 4], (a, b) -> [])", "[[],[]]"),
        (
            "def pack(a, b):\n    return {first: a, second: b}\n[1, 2].zip([3, 4], pack).map(o -> o.second)",
            "[3.0,4.0]",
        ),
        (
            "[\"a\", \"b\"].zip([\"x\", \"y\"], (a, b) -> a + b)",
            "[\"ax\",\"by\"]",
        ),
        (
            "[1, 2].zip([3, 4], (a, b) -> a > 1 ? \"big\" : \"small\")",
            "[\"small\",\"big\"]",
        ),
        (
            "factor = 3\n[1, 2].zip([3, 4], (a, b) -> (a + b) * factor)",
            "[12.0,18.0]",
        ),
        (
            "class Unit(n):\n    this.n = n\nUnit(2)\nUnit(4)\nUnit.zip([10, 20], (unit, weight) -> unit.n * weight)",
            "[20.0,80.0]",
        ),
        (
            "seen = []\nclass Unit(n):\n    this.n = n\nUnit(2)\nUnit(4)\n[[1, 2].zip(Unit, (k, unit) -> seen.push(unit.n) > 0 ? k * unit.n : 0), seen]",
            "[[2.0,8.0],[2.0,4.0]]",
        ),
        (
            "[1, 2].zip([3, 4], (a, b) -> [1, 2].zip([a, b]))",
            "[[[1.0,1.0],[2.0,3.0]],[[1.0,2.0],[2.0,4.0]]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn the_other_sequence_must_be_a_list_or_a_declared_class_even_on_empty_receivers() {
    let message = (ErrorKind::Type, "zip expects a list".to_string());
    for source in [
        "[1, 2].zip()",
        "[1, 2].zip(5)",
        "[1, 2].zip(\"ab\")",
        "[1, 2].zip(true)",
        "[1, 2].zip(null)",
        "[1, 2].zip(undefined)",
        "o = {}\n[1, 2].zip(o.missing)",
        "[1, 2].zip({a: 1})",
        "class P\np = P()\n[1, 2].zip(p)",
        "[1, 2].zip(x -> x)",
        "def f() => 1\n[1, 2].zip(f)",
        "[1, 2].zip(Date(0))",
        "[1, 2].zip(/a/)",
        "[1, 2].zip(Math)",
        "[1, 2].zip(Object)",
        "[1, 2].zip(5, 6)",
        "[1, 2].zip(5, (a, b) -> a)",
        "[].zip()",
        "[].zip(5)",
        "[].zip(null, null)",
        "class Empty\nEmpty.zip(5)",
        "class Empty\nEmpty.zip()",
        "class Order(total):\n    this.total = total\nOrder(1)\nOrder.zip(\"total\")",
        "class Order(total):\n    this.total = total\nOrder(1)\nkind = Order\nkind.zip(1)",
        "class Order(total):\n    this.total = total\nOrder(1)\n$Order.zip(null)",
        "class Calc(base):\n    this.base = base\n    def zip(a, b) => this.base + a * b\nc = Calc(1)\nCalc.zip(2, 3)",
        "xs = [1, 2]\nxs.zip(xs[0])",
    ] {
        assert_eq!(failure(source), message, "{source}");
    }
    let mut runtime = Runtime::new();
    runtime
        .run("xs = [1, 2]\nys = [3, 4]\npairs = xs.zip(ys)")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "ys = 5"),
        "TypeError: zip expects a list at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(
        json(&mut runtime, "[ys, pairs]"),
        "[[3.0,4.0],[[1.0,3.0],[2.0,4.0]]]"
    );
    assert_eq!(
        outcome(&mut runtime, "pairs = xs.zip(\"34\")"),
        "TypeError: zip expects a list at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(
        json(&mut runtime, "ys.push(6)\npairs"),
        "[[1.0,3.0],[2.0,4.0]]"
    );
}

#[test]
fn a_supplied_combiner_must_be_a_function_even_on_empty_sequences() {
    let message = (ErrorKind::Type, "zip expects a function".to_string());
    for source in [
        "[1, 2].zip([3, 4], null)",
        "[1, 2].zip([3, 4], undefined)",
        "o = {}\n[1, 2].zip([3, 4], o.missing)",
        "[1, 2].zip([3, 4], 5)",
        "[1, 2].zip([3, 4], \"f\")",
        "[1, 2].zip([3, 4], [(a, b) -> a])",
        "[1, 2].zip([3, 4], {})",
        "[1, 2].zip([3, 4], true)",
        "[1, 2].zip([3, 4], [5, 6])",
        "class P\n[1, 2].zip([3, 4], P)",
        "[].zip([], 5)",
        "[].zip([3, 4], null)",
        "[1, 2].zip([], \"f\")",
        "class Empty\nEmpty.zip([1], null)",
        "class Order(total):\n    this.total = total\nOrder(1)\nOrder.zip([1], 5)",
    ] {
        assert_eq!(failure(source), message, "{source}");
    }
    let mut runtime = Runtime::new();
    runtime
        .run("xs = [2, 4]\nys = [1, 3]\nsums = xs.zip(ys, (a, b) -> a + b)")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "sums = xs.zip(ys, 1)"),
        "TypeError: zip expects a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(
        json(&mut runtime, "xs.push(6)\nys.push(5)\nsums"),
        "[3.0,7.0,11.0]"
    );
}

#[test]
fn arguments_are_evaluated_once_left_to_right_and_extras_are_ignored() {
    for (source, expected) in [
        (
            "seen = []\ndef mark(label, value):\n    seen.push(label)\n    return value\nxs = [3, 1]\n[mark(\"receiver\", xs).zip(mark(\"other\", [5, 6]), mark(\"combine\", (a, b) -> a), mark(\"extra\", 0)), seen]",
            "[[3.0,1.0],[\"receiver\",\"other\",\"combine\",\"extra\"]]",
        ),
        (
            "seen = []\ndef mark() => seen.push(1)\nxs = null\nr = xs.zip(mark(), mark())\n[r, seen]",
            "[null,[]]",
        ),
        ("o = {}\nr = o.items.zip([1])\nr", "null"),
        ("[1, 2].zip([3, 4], (a, b) -> a + b, 9)", "[4.0,6.0]"),
        (
            "[1, 2].zip([3, 4], (a, b) -> a + b, null, \"extra\")",
            "[4.0,6.0]",
        ),
        ("[1, 2].zip([3, 4], (a, b) -> a, [5, 6])", "[1.0,2.0]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for source in [
        "xs = [1, 2]\nxs.zip(missing)",
        "xs = [1, 2]\nxs.zip([2], missing)",
        "xs = [1, 2]\nxs.zip([2], (a, b) -> a, missing)",
        "xs = [1, 2]\nxs.zip(5, missing)",
    ] {
        assert_eq!(
            failure(source),
            (ErrorKind::Reference, "missing is not defined".to_string()),
            "{source}"
        );
    }
}

#[test]
fn declarations_follow_both_sequences_and_what_the_combiner_reads() {
    behaves_as_spelled(
        &[
            ("xs = [1, 2]", "xs = [1, 2]"),
            ("ys = [3, 4, 5]", "ys = [3, 4, 5]"),
            (
                "pairs = xs.zip(ys)",
                &format!("pairs = {}", spelled_pairs("xs", "ys")),
            ),
            ("pairs", "pairs"),
            ("xs.push(6)", "xs.push(6)"),
            ("pairs", "pairs"),
            ("xs.push(7)", "xs.push(7)"),
            ("pairs", "pairs"),
            ("ys.push(8)", "ys.push(8)"),
            ("pairs", "pairs"),
            ("xs.shift()", "xs.shift()"),
            ("pairs", "pairs"),
            ("ys.pop()", "ys.pop()"),
            ("pairs", "pairs"),
            ("ys = [9]", "ys = [9]"),
            ("pairs", "pairs"),
            ("xs = []", "xs = []"),
            ("pairs", "pairs"),
        ],
        &["pairs"],
    );
    behaves_as_spelled(
        &[
            ("rows = [{k: 2}, {k: 5}]", "rows = [{k: 2}, {k: 5}]"),
            ("weights = [10, 100]", "weights = [10, 100]"),
            ("factor = 2", "factor = 2"),
            (
                "total = rows.zip(weights, (row, weight) -> row.k * weight * factor)",
                &format!(
                    "total = {}",
                    spelled_combined(
                        "rows",
                        "weights",
                        "(row, weight) -> row.k * weight * factor"
                    )
                ),
            ),
            ("total", "total"),
            ("factor = 3", "factor = 3"),
            ("total", "total"),
            ("rows[0].k = 4", "rows[0].k = 4"),
            ("total", "total"),
            ("rows.push({k: 1})", "rows.push({k: 1})"),
            ("weights.push(1000)", "weights.push(1000)"),
            ("total", "total"),
        ],
        &["total"],
    );
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "xs = [1, 2]\nys = [3, 4]\npairs = xs.zip(ys)\npairs",
            "[[1.0,3.0],[2.0,4.0]]",
        ),
        ("xs.push(5)\npairs", "[[1.0,3.0],[2.0,4.0]]"),
        ("ys.push(6)\npairs", "[[1.0,3.0],[2.0,4.0],[5.0,6.0]]"),
        ("xs.pop()\npairs", "[[1.0,3.0],[2.0,4.0]]"),
        ("xs = [\"a\"]\npairs", "[[\"a\",3.0]]"),
        ("ys = []\npairs", "[]"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn declarations_follow_new_and_deleted_instances_and_class_level_rules() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class Driver(name):\n    this.name = name\nclass Truck(plate):\n    this.plate = plate\ncrews = Driver.zip(Truck, (d, t) -> d.name + \":\" + t.plate)\ncrews",
            "[]",
        ),
        ("a = Driver(\"a\")\ncrews", "[]"),
        ("t1 = Truck(\"T1\")\ncrews", "[\"a:T1\"]"),
        (
            "b = Driver(\"b\")\nTruck(\"T2\")\ncrews",
            "[\"a:T1\",\"b:T2\"]",
        ),
        ("t1.plate = \"T9\"\ncrews", "[\"a:T9\",\"b:T2\"]"),
        ("delete a.name\ndelete a\ncrews", "[\"b:T9\"]"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class Order(items, prices):\n    this.items = items\n    this.prices = prices\n$Order.lines = $Order.items.zip($Order.prices)\n$Order.total = $Order.prices.zip($Order.items, (price, item) -> price).sum()\no = Order([\"a\", \"b\"], [3, 4])\n[o.lines, o.total]",
            "[[[\"a\",3.0],[\"b\",4.0]],7.0]",
        ),
        (
            "o.items.push(\"c\")\n[o.lines, o.total]",
            "[[[\"a\",3.0],[\"b\",4.0]],7.0]",
        ),
        (
            "o.prices.push(5)\n[o.lines, o.total]",
            "[[[\"a\",3.0],[\"b\",4.0],[\"c\",5.0]],12.0]",
        ),
        (
            "p = Order([\"x\"], [9, 1])\n[p.lines, p.total, o.total]",
            "[[[\"x\",9.0]],9.0,12.0]",
        ),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "class Trail(heights):\n    this.heights = heights\n$Trail.climbs = $Trail.heights.zip($Trail.heights[1:], (low, high) -> high - low)\nt = Trail([1, 4, 9])\nt.heights.push(16)\nt.heights = [5, 3]\nt.climbs",
            "[-2.0]",
        ),
        (
            "class P(x):\n    this.x = x\n$P.gaps = P.zip(P[1:], (a, b) -> b.x - a.x)\np = P(1)\nq = P(4)\nr = P(9)\n[p.gaps, r.gaps]",
            "[[3.0,5.0],[3.0,5.0]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn zip_equals_its_accepted_reduce_spelling_over_generated_lists() {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let pool = [
        "1", "2", "-3", "4.5", "0", "\"a\"", "\"b\"", "true", "[1]", "[]", "{k: 1}",
    ];
    let combiners = [
        "(a, b) -> [a, b]",
        "(a, b) -> b",
        "(a, b) -> String(a) + String(b)",
        "(a, b) -> [b, [a]]",
        "(a) -> a",
    ];
    for round in 0..120 {
        let mut list = |limit: u64| -> String {
            let length = (next() % limit) as usize;
            let items: Vec<&str> = (0..length)
                .map(|_| pool[(next() % pool.len() as u64) as usize])
                .collect();
            format!("[{}]", items.join(", "))
        };
        let xs = list(7);
        let ys = list(7);
        let combine = combiners[round % combiners.len()];
        let setup = format!("xs = {xs}\nys = {ys}");
        let measured = result(&format!("{setup}\n[xs.zip(ys), xs.zip(ys, {combine})]"));
        let spelled = result(&format!(
            "{setup}\n[{}, {}]",
            spelled_pairs("xs", "ys"),
            spelled_combined("xs", "ys", combine)
        ));
        assert_eq!(renumbered(&measured), renumbered(&spelled), "{setup}");
        let declared = result(&format!(
            "{setup}\np = xs.zip(ys, {combine})\nxs.push(7)\nys.push(8)\np"
        ));
        let declared_spelling = result(&format!(
            "{setup}\np = {}\nxs.push(7)\nys.push(8)\np",
            spelled_combined("xs", "ys", combine)
        ));
        assert_eq!(
            renumbered(&declared),
            renumbered(&declared_spelling),
            "{setup}"
        );
    }
}

#[test]
fn the_index_of_spelling_matches_only_while_every_item_is_found_at_its_position() {
    for (xs, ys) in [
        ("[1, 2, 3]", "[4, 5, 6]"),
        ("[\"a\", \"b\"]", "[1]"),
        ("[3, 1, 2, 9]", "[0, 0, 0]"),
    ] {
        assert_eq!(
            result(&format!("xs = {xs}\nys = {ys}\nxs.zip(ys)")),
            result(&format!(
                "xs = {xs}\nys = {ys}\nxs[:ys.length].map(x -> [x, ys[xs.indexOf(x)]])"
            )),
            "{xs} {ys}"
        );
    }
    assert_eq!(
        result(
            "xs = [1, 1, 2]\nys = [4, 5, 6]\n[xs.zip(ys), xs[:ys.length].map(x -> [x, ys[xs.indexOf(x)]])]"
        ),
        "[[[1.0,4.0],[1.0,5.0],[2.0,6.0]],[[1.0,4.0],[1.0,4.0],[2.0,6.0]]]"
    );
}

#[test]
fn null_items_stay_in_their_pairs_while_reads_inside_a_combiner_settle() {
    for (source, expected) in [
        ("r = [1, null].zip([3, 4])\nr", "[[1.0,3.0],[null,4.0]]"),
        ("r = [1, 2].zip([null, 4])\nr", "[[1.0,null],[2.0,4.0]]"),
        ("r = [1, null].zip([3, 4], (a, b) -> b)\nr", "[3.0,4.0]"),
        ("r = [1, null].zip([3, 4], (a, b) -> a)\nr", "null"),
        ("r = [1, 2].zip([null, 4], (a, b) -> b)\nr", "null"),
        (
            &format!(
                "xs = [1, null]\nys = [3, 4]\nr = {}\nr",
                spelled_pairs("xs", "ys")
            ),
            "null",
        ),
        (
            &format!(
                "xs = [1, 2]\nys = [null, 4]\nr = {}\nr",
                spelled_pairs("xs", "ys")
            ),
            "[[1.0,null],[2.0,4.0]]",
        ),
        ("xs = [null]\nr = xs.zip(xs)\nr", "[[null,null]]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn rendering_and_dependencies_name_the_call_as_written() {
    let mut runtime = Runtime::new();
    assert_eq!(
        json(
            &mut runtime,
            "xs = [1, 2]\nys = [3, 4]\npairs = xs.zip(ys)\n(why pairs).map(step -> step.rule)"
        ),
        "[\"pairs = xs.zip(ys)\",\"xs = [1,2]\",\"ys = [3,4]\"]"
    );
    assert_eq!(json(&mut runtime, "(why pairs)[0].from"), "[\"xs\",\"ys\"]");
    assert_eq!(
        json(&mut runtime, "(affects ys).map(step -> step.node)"),
        "[\"pairs\"]"
    );
    assert_eq!(dependencies(&runtime, "pairs"), ["xs", "ys"]);
    assert_eq!(
        json(
            &mut runtime,
            "k = 2\nsums = xs.zip(ys, (a, b) -> a + b * k)\n[(why sums).map(step -> step.rule), (affects k).map(step -> step.node)]"
        ),
        "[[\"sums = xs.zip(ys,function())\",\"xs = [1,2]\",\"ys = [3,4]\",\"k = 2\"],[\"sums\"]]"
    );
    assert_eq!(dependencies(&runtime, "sums"), ["k", "xs", "ys"]);
    let mut runtime = Runtime::new();
    runtime
        .run("class Driver(name):\n    this.name = name\nclass Truck(plate):\n    this.plate = plate\ncrews = Driver.zip(Truck, (d, t) -> d.name)\nDriver(\"a\")\nTruck(\"b\")")
        .unwrap();
    assert_eq!(
        dependencies(&runtime, "crews"),
        ["$Driver", "$Truck", "obj:0.name"]
    );
    let mut spelled = Runtime::new();
    spelled
        .run(&format!(
            "xs = [1, 2]\nys = [3, 4]\npairs = {}",
            spelled_pairs("xs", "ys")
        ))
        .unwrap();
    assert_eq!(dependencies(&spelled, "pairs"), ["xs", "ys"]);
    assert_eq!(
        json(&mut spelled, "(why pairs)[0].rule"),
        "\"pairs = xs[:ys.length].reduce(function(),[])\""
    );
    for (source, message) in [
        (
            "xs = [1, 2]\nys = [3, 4]\nxs.zip(ys)(1)",
            "xs.zip(ys) is not a function",
        ),
        (
            "xs = [1, 2]\nys = [3, 4]\nwhy xs.zip(ys)",
            "Cannot explain xs.zip(ys)",
        ),
    ] {
        assert_eq!(
            failure(source),
            (ErrorKind::Type, message.to_string()),
            "{source}"
        );
    }
}

#[test]
fn snapshots_hypotheticals_templates_and_conditions_read_zips() {
    for (source, expected) in [
        (
            "xs = [1, 3]\nys = [2, 4]\np = xs.zip(ys)\nfrozen = p.value\nxs.push(5)\nys.push(6)\n[p, frozen]",
            "[[[1.0,2.0],[3.0,4.0],[5.0,6.0]],[[1.0,2.0],[3.0,4.0]]]",
        ),
        (
            "xs = [1, 3]\nfrozen = xs.value.zip([2, 4, 6])\nxs.push(5)\nfrozen",
            "[[1.0,2.0],[3.0,4.0]]",
        ),
        (
            "xs = [1, 3]\nys = [2, 4, 6]\np = xs.zip(ys)\nseen = suppose xs.push(5): p\n[seen, p]",
            "[[[1.0,2.0],[3.0,4.0],[5.0,6.0]],[[1.0,2.0],[3.0,4.0]]]",
        ),
        (
            "xs = [1, 3]\nys = [2, 4]\nlabel = `${xs.zip(ys).length} pairs`\nxs.push(5)\nys.push(6)\nlabel",
            "\"3 pairs\"",
        ),
        (
            "xs = [1, 3]\nys = [2, 4]\nif xs.zip(ys).length > 1:\n    flag = true\nflag",
            "true",
        ),
        (
            "xs = [1, 3]\nys = [2, 4]\nonce := xs.zip(ys)\nxs.push(5)\nys.push(6)\n[once, xs.zip(ys)]",
            "[[[1.0,2.0],[3.0,4.0]],[[1.0,2.0],[3.0,4.0],[5.0,6.0]]]",
        ),
        (
            "xs = [1, 2]\nall t in xs.zip([3, 4]): t.length == 2",
            "true",
        ),
        ("xs = [1, 2]\nany t in xs.zip([3, 4]): t[1] == 4", "true"),
        ("xs = [1, 2]\n[2, 4] in xs.zip([3, 4])", "true"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn failed_statements_and_transactions_restore_zips() {
    behaves_as_spelled(
        &[
            ("rows = [{k: 2}, {k: 4}]", "rows = [{k: 2}, {k: 4}]"),
            ("limits = [5, 5, 5]", "limits = [5, 5, 5]"),
            (
                "spare = rows.zip(limits, (row, limit) -> row.k > limit ? row.k.toUpperCase() : limit - row.k)",
                &format!(
                    "spare = {}",
                    spelled_combined(
                        "rows",
                        "limits",
                        "(row, limit) -> row.k > limit ? row.k.toUpperCase() : limit - row.k"
                    )
                ),
            ),
            (
                "try:\n    rows.push({k: 1})\n    assert(spare, [3, 1, 4])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
                "try:\n    rows.push({k: 1})\n    assert(spare, [3, 1, 4])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
            ),
            ("[rows.length, spare]", "[rows.length, spare]"),
            ("rows.push({k: 9})", "rows.push({k: 9})"),
            ("[rows.length, spare]", "[rows.length, spare]"),
            ("rows.push({k: 5})", "rows.push({k: 5})"),
            ("spare", "spare"),
        ],
        &["spare"],
    );
    let mut runtime = Runtime::new();
    runtime
        .run("rows = [{k: 2}, {k: 4}]\nlimits = [5, 5, 5]\nspare = rows.zip(limits, (row, limit) -> row.k > limit ? row.k.toUpperCase() : limit - row.k)")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "rows.push({k: 10})"),
        "TypeError: number.toUpperCase is not a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(
        json(&mut runtime, "[rows.length, spare]"),
        "[2.0,[3.0,1.0]]"
    );
    assert_eq!(
        result(
            "def gap(load, limit):\n    if load > limit:\n        throw \"OVERLOAD\"\n    return limit - load\nclass Bay(load): this.load = load\nBay(1)\nlimits = [4, 8]\nroom = Bay.zip(limits, (bay, limit) -> gap(bay.load, limit))\nfailure = \"\"\ntry:\n    Bay(9)\ncatch error:\n    failure = error\n[failure, Bay.length, room]"
        ),
        "[\"OVERLOAD\",1.0,[3.0]]"
    );
    assert_eq!(
        result(
            "xs = [1, 3]\nys = [2, 4, 6]\np = xs.zip(ys)\ntry:\n    xs.push(5)\n    throw \"UNDO\"\ncatch error:\n    pass\np"
        ),
        "[[1.0,2.0],[3.0,4.0]]"
    );
    assert_eq!(
        result(
            "xs = [1]\nys = [2]\np = xs.zip(ys, (a, b) -> b > 5 ? missing : a + b)\nfailure = \"\"\ntry:\n    xs.push(3)\n    ys.push(9)\ncatch error:\n    failure = error\n[failure, xs, ys, p]"
        ),
        "[\"ReferenceError: missing is not defined\",[1.0],[2.0],[3.0]]"
    );
}

#[test]
fn large_sequences_zip_through_the_public_api() {
    let mut runtime = Runtime::new();
    runtime
        .run("holder = {items: 0, others: 0}\ncounter = {n: 0}\ndef tick(a, b):\n    counter.n = counter.n + 1\n    return a + b")
        .unwrap();
    let count = 100_000_u32;
    hold(
        &mut runtime,
        "items",
        Value::List(
            (0..count)
                .map(|position| Value::Number(f64::from(position)))
                .collect(),
        ),
    );
    hold(
        &mut runtime,
        "others",
        Value::List(
            (0..count + 7)
                .map(|position| Value::Number(f64::from(position) * 2.0))
                .collect(),
        ),
    );
    assert_eq!(
        json(&mut runtime, "holder.items.zip(holder.others).length"),
        "100000.0"
    );
    assert_eq!(
        json(&mut runtime, "holder.items.zip(holder.others)[99999]"),
        "[99999.0,199998.0]"
    );
    assert_eq!(
        json(&mut runtime, "holder.others.zip(holder.items).length"),
        "100000.0"
    );
    assert_eq!(
        json(
            &mut runtime,
            "[holder.items.zip(holder.others, tick)[99999], counter.n]"
        ),
        "[299997.0,100000.0]"
    );
    assert_eq!(json(&mut runtime, "holder.items.length"), "100000.0");
    assert_eq!(json(&mut runtime, "holder.others.length"), "100007.0");
}

#[test]
fn other_list_queries_keep_their_results_beside_zip() {
    for (source, expected) in [
        ("[1, 2].concat([3], 4)", "[1.0,2.0,3.0,4.0]"),
        ("[1, 2].concat([[3]])", "[1.0,2.0,[3.0]]"),
        ("[1, 2].map(x -> [x, x])", "[[1.0,1.0],[2.0,2.0]]"),
        ("[1, 2].flatMap(x -> [x, x])", "[1.0,1.0,2.0,2.0]"),
        ("[[1, 2], [3, 4]].flat()", "[1.0,2.0,3.0,4.0]"),
        (
            "[1, 2].reduce((acc, x) -> acc.concat([[x]]), [])",
            "[[1.0],[2.0]]",
        ),
        ("[3, 1, 2].sortBy(v -> v)", "[1.0,2.0,3.0]"),
        ("[1, 2, 3].partition(x -> x > 1)", "[[2.0,3.0],[1.0]]"),
        ("[1, 2, 3].sum()", "6.0"),
        ("[1, 2].join(\"-\")", "\"1-2\""),
        ("[1, 2].slice(1)", "[2.0]"),
        ("[1, 2].at(-1)", "2.0"),
        ("[1, 2].size()", "2.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, message) in [
        (
            "xs = [1, 2]\nxs.zipWith([3], (a, b) -> a)",
            "xs.zipWith is not a function",
        ),
        ("xs = [1, 2]\nxs.unzip()", "xs.unzip is not a function"),
        (
            "xs = [1, 2]\nxs.zip_longest([3])",
            "xs.zip_longest is not a function",
        ),
        ("xs = [1, 2]\nxs.zap([3])", "xs.zap is not a function"),
        ("xs = [1, 2]\nxs.zips([3])", "xs.zips is not a function"),
        (
            "class Driver\nDriver.zipWith([1])",
            "Driver.zipWith is not a function",
        ),
        ("xs = [1]\nxs.list()", "xs.list is not a function"),
    ] {
        assert_eq!(
            failure(source),
            (ErrorKind::Type, message.to_string()),
            "{source}"
        );
    }
}

#[test]
fn the_compatibility_rows_give_their_documented_results() {
    let drivers_and_trucks = format!("{DRIVERS}{TRUCKS}");
    for (source, expected) in [
        ("[1, 2, 3].zip([4, 5, 6])".to_string(), "[[1.0,4.0],[2.0,5.0],[3.0,6.0]]"),
        ("[1, 2, 3].zip([\"a\", \"b\"])".to_string(), "[[1.0,\"a\"],[2.0,\"b\"]]"),
        ("[1].zip([4, 5, 6])".to_string(), "[[1.0,4.0]]"),
        ("[[].zip([1]), [1].zip([])]".to_string(), "[[],[]]"),
        ("[1, 2, 3].zip([4, 5, 6], (a, b) -> a * b)".to_string(), "[4.0,10.0,18.0]"),
        (
            "prices = [3, 5, 2]\nquantities = [2, 1, 4]\nprices.zip(quantities, (p, q) -> p * q).sum()".to_string(),
            "19.0",
        ),
        ("[[1, 2], [3]].zip([[4], [5, 6]])".to_string(), "[[[1.0,2.0],[4.0]],[[3.0],[5.0,6.0]]]"),
        ("[null, 1].zip([2, undefined])".to_string(), "[[null,2.0],[1.0,null]]"),
        (
            "xs = [1, 2]\nys = [3, 4]\npairs = xs.zip(ys)\nxs.push(5)\nys.push(6)\npairs".to_string(),
            "[[1.0,3.0],[2.0,4.0],[5.0,6.0]]",
        ),
        (
            "xs = [1, 2]\nys = [3, 4]\nk = 2\nsums = xs.zip(ys, (a, b) -> a + b * k)\nk = 3\nsums".to_string(),
            "[10.0,14.0]",
        ),
        ("xs = [1, 2]\nys = [3, 4]\nxs.zip(ys).map(p -> p[0] * p[1])".to_string(), "[3.0,8.0]"),
        ("[1, 2].zip([3, 4]).zip([5, 6])".to_string(), "[[[1.0,3.0],5.0],[[2.0,4.0],6.0]]"),
        ("o = {xs: [1, 2], ys: [3, 4]}\no.xs.zip(o.ys)".to_string(), "[[1.0,3.0],[2.0,4.0]]"),
        ("xs = [1, 2, 3, 4, 5, 6]\nxs[::2].zip(xs[1::2])".to_string(), "[[1.0,2.0],[3.0,4.0],[5.0,6.0]]"),
        ("xs = [1, 4, 9, 16]\nxs.zip(xs[1:], (a, b) -> b - a)".to_string(), "[3.0,5.0,7.0]"),
        ("xs = [1, 2]\nxs.value.zip([3])".to_string(), "[[1.0,3.0]]"),
        ("xs = [1, 2]\nxs.push(5)\nxs@first.zip([3])".to_string(), "[[1.0,3.0]]"),
        ("grid = [[1, 2], [3]]\ngrid[0].zip(grid[1])".to_string(), "[[1.0,3.0]]"),
        ("def make() => [1, 2]\nmake().zip([3, 4])".to_string(), "[[1.0,3.0],[2.0,4.0]]"),
        ("xs = [1, 2, 3]\nxs.sortBy(x -> -x).zip(xs)".to_string(), "[[3.0,1.0],[2.0,2.0],[1.0,3.0]]"),
        ("xs = [1, 2, 3]\nxs.slice(0, 2).zip(xs)".to_string(), "[[1.0,1.0],[2.0,2.0]]"),
        ("xs = [1, 2, 3]\nxs.concat([4]).zip(xs)".to_string(), "[[1.0,1.0],[2.0,2.0],[3.0,3.0]]"),
        ("xs = [1, 2]\nkind = xs\nkind.zip([3])".to_string(), "[[1.0,3.0]]"),
        ("zip = [1, 2]\nzip.zip([3, 4])".to_string(), "[[1.0,3.0],[2.0,4.0]]"),
        ("def pairs(xs, ys):\n    return xs.zip(ys)\npairs([1, 2], [3, 4])".to_string(), "[[1.0,3.0],[2.0,4.0]]"),
        (
            format!("{drivers_and_trucks}Driver.zip(Truck).map(p -> p[0].name + p[1].plate)"),
            "[\"adaT1\",\"boT2\"]",
        ),
        (format!("{drivers_and_trucks}[1, 2, 3].zip(Truck).map(p -> p[1].plate)"), "[\"T1\",\"T2\"]"),
        (format!("{DRIVERS}Driver.zip([10, 20, 30, 40]).map(p -> p[1])"), "[10.0,20.0,30.0]"),
        (format!("{DRIVERS}Driver.zip(Driver).map(p -> p[0] == p[1])"), "[true,true,true]"),
        (format!("{drivers_and_trucks}pairs = Driver.zip(Truck)\nTruck(\"T3\")\npairs.length"), "3.0"),
        (format!("{DRIVERS}kind = Driver\nkind.zip([1]).length"), "1.0"),
        (format!("{DRIVERS}$Driver.zip([1]).length"), "1.0"),
        (format!("{DRIVERS}Driver[1:].zip([1])[0][0].name"), "\"bo\""),
        (format!("{DRIVERS}Driver.list().zip([1]).length"), "1.0"),
        (format!("{DRIVERS}Driver.filter(d -> true).zip([1]).length"), "1.0"),
        ("class Empty\nEmpty.zip([1])".to_string(), "[]"),
        (
            "class Box(sides):\n    this.sides = sides\n$Box.pairs = $Box.sides.zip($Box.sides[1:])\nb = Box([1, 2, 3])\nb.pairs".to_string(),
            "[[1.0,2.0],[2.0,3.0]]",
        ),
        (format!("{TEAM}Team.zip([1]).length"), "1.0"),
        ("[1, 2].zip([3, 4], (a, b) -> a + b, 9)".to_string(), "[4.0,6.0]"),
        ("[1, 2].zip([3, 4], (a) -> a)".to_string(), "[1.0,2.0]"),
        ("[1, 2].zip([3, 4], (a, b, c) -> c)".to_string(), "[null,null]"),
        (
            "def check(b):\n    if b > 3:\n        throw \"BIG\"\n    return b\nr = \"\"\ntry:\n    [1, 2].zip([3, 4], (a, b) -> check(b))\ncatch e:\n    r = e\nr".to_string(),
            "\"BIG\"",
        ),
        (
            "xs = [1, 2]\nys = [3, 4]\nr = 0\ntry:\n    r = xs.zip(ys)\ncatch error:\n    r = error\nr".to_string(),
            "[[1.0,3.0],[2.0,4.0]]",
        ),
    ] {
        assert_eq!(result(&source), expected, "{source}");
    }
    for (source, expected) in [
        (
            format!("{CALCULATOR}Calc.zip(2, 3)"),
            "TypeError: zip expects a list at Some(Position { line: 5, column: 1 })",
        ),
        (
            "[1, 2].zip(5)".to_string(),
            "TypeError: zip expects a list at Some(Position { line: 1, column: 1 })",
        ),
        (
            "[1, 2].zip()".to_string(),
            "TypeError: zip expects a list at Some(Position { line: 1, column: 1 })",
        ),
        (
            "[].zip(5)".to_string(),
            "TypeError: zip expects a list at Some(Position { line: 1, column: 1 })",
        ),
        (
            "[1, 2].zip([3, 4], 5)".to_string(),
            "TypeError: zip expects a function at Some(Position { line: 1, column: 1 })",
        ),
        (
            "xs = [1, 2]\nys = [3, 4]\nxs.zip(ys)(1)".to_string(),
            "TypeError: xs.zip(ys) is not a function at Some(Position { line: 3, column: 1 })",
        ),
        (
            "def scaled(xs, ys, k) => xs.zip(ys, (a, b) -> a * k)\nscaled([1, 2], [3, 4], 2)"
                .to_string(),
            "ReferenceError: k is not defined at Some(Position { line: 2, column: 1 })",
        ),
        (
            "def scaled(xs, k) => xs.map(x -> x * k)\nscaled([1, 2], 2)".to_string(),
            "ReferenceError: k is not defined at Some(Position { line: 2, column: 1 })",
        ),
    ] {
        assert_eq!(outcome(&mut Runtime::new(), &source), expected, "{source}");
    }
}

#[test]
fn unchanged_readings_keep_their_accepted_results() {
    for (source, expected) in [
        ("o = {zip: (a) -> a + 1}\no.zip(1)", "2.0"),
        ("o = {a: 1}\no.zip([1])", "null"),
        ("zip = (a, b) -> [a, b]\nzip(1, 2)", "[1.0,2.0]"),
        (
            "def zip(a, b) => a.length + b.length\nzip([1], [2, 3])",
            "3.0",
        ),
        ("xs = [1, 2]\nxs.zip", "null"),
        ("xs = [1, 2]\nr = xs.zip\nr", "null"),
        ("xs = [1]\ntypeof xs.zip", "\"undefined\""),
        (
            "class Address(zip):\n    this.zip = zip\na = Address(\"0150\")\na.zip",
            "\"0150\"",
        ),
        (
            "class Address(zip):\n    this.zip = zip\nAddress(\"0150\")\nAddress.zip",
            "null",
        ),
        (
            "class Team(name):\n    this.name = name\n    def zip(other):\n        return this.name + other\nred = Team(\"red\")\nred.zip(\"!\")",
            "\"red!\"",
        ),
        (
            "class Calc(base):\n    this.base = base\n    def zip(a, b) => this.base + a * b\nc = Calc(1)\nc.zip(2, 3)",
            "7.0",
        ),
        (
            "shipments = [{to: {zip: \"0150\"}}, {to: {zip: null}}]\nshipments.map(.to.zip)",
            "[\"0150\",null]",
        ),
        ("o = {zip: 3}\no.zip", "3.0"),
        ("x = {zip: 4}.zip\nx", "4.0"),
        ("def g(zip) => zip * 2\ng(4)", "8.0"),
        ("f = lambda zip: zip + 1\nf(4)", "5.0"),
        (
            "class S(zip):\n    this.zip = zip\ns = S([1, 2])\ns.zip",
            "[1.0,2.0]",
        ),
        ("n = null\nn.zip(missing)", "null"),
        ("o = {}\no.items.zip([1])", "null"),
        (
            "bergen = {to: {zip: \"5000\"}}\nbergen.to.zip = \"5003\"\nbergen.to.zip",
            "\"5003\"",
        ),
        ("x = 5\nzip = 3\nzip + x", "8.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, kind, message) in [
        (
            "\"ab\".zip(\"cd\")",
            ErrorKind::Type,
            "ab.zip is not a function",
        ),
        (
            "x = 5\nx.zip([1])",
            ErrorKind::Type,
            "number.zip is not a function",
        ),
        (
            "(5).zip([1])",
            ErrorKind::Type,
            "number.zip is not a function",
        ),
        (
            "true.zip([1])",
            ErrorKind::Type,
            "boolean.zip is not a function",
        ),
        (
            "Date(0).zip([1])",
            ErrorKind::Type,
            "Date.zip is not a function",
        ),
        ("/a/.zip([1])", ErrorKind::Type, "zip is not a function"),
        (
            "f = x -> x\nf.zip([1])",
            ErrorKind::Type,
            "function.zip is not a function",
        ),
        (
            "Math.zip([1], [2])",
            ErrorKind::Type,
            "Math.zip is not a function",
        ),
        (
            "Object.zip([1])",
            ErrorKind::Reference,
            "Object is not defined",
        ),
        ("List.zip([1])", ErrorKind::Reference, "List is not defined"),
        ("JSON.zip([1])", ErrorKind::Reference, "JSON is not defined"),
        ("zip([1], [2])", ErrorKind::Reference, "zip is not defined"),
        (
            "xs = [1, 2]\nzip of xs",
            ErrorKind::Syntax,
            "Unexpected keyword 'of'",
        ),
    ] {
        assert_eq!(failure(source), (kind, message.to_string()), "{source}");
    }
    let error = Runtime::new().run("xs = [1, 2]\nzip of xs").unwrap_err();
    assert_eq!(
        format!("{:?}", error.position()),
        "Some(Position { line: 2, column: 8 })"
    );
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    assert_eq!(
        result("xs = [1, 3]\nr = 0\ntry:\n    r = xs.zip([2, 4])\ncatch error:\n    r = error\nr"),
        "[[1.0,2.0],[3.0,4.0]]"
    );
    assert_eq!(
        result("xs = [1, 3]\nr = 0\nif xs.zip([2]).length == 1:\n    r = 1\nr"),
        "1.0"
    );
    assert_eq!(
        result("xs = [1, 3]\nr = 0\ntry:\n    r = xs.zipWith([2])\ncatch error:\n    r = error\nr"),
        "\"TypeError: xs.zipWith is not a function\""
    );
    assert_eq!(
        result("xs = [1, 3]\nr = 0\ntry:\n    r = xs.zip(5)\ncatch error:\n    r = error\nr"),
        "\"TypeError: zip expects a list\""
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-list-zipping-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_zip() {
    let program = "xs = [1, 2]\nys = [3, 4, 5]\npairs = xs.zip(ys)\nxs.push(6)\n[pairs, xs.zip(ys, (a, b) -> a * b)]";
    let file = SourceFile::new("file");
    let measured = file.run(program, &["--json"]);
    assert!(measured.status.success(), "{measured:?}");
    assert!(measured.stderr.is_empty(), "{measured:?}");
    assert_eq!(
        String::from_utf8(measured.stdout).unwrap(),
        "[[[1.0,3.0],[2.0,4.0],[6.0,5.0]],[3.0,8.0,30.0]]\n"
    );
    let plain = file.run(program, &[]);
    assert!(plain.status.success(), "{plain:?}");
    assert_eq!(
        String::from_utf8(plain.stdout).unwrap(),
        "1,3,2,4,6,5,3,8,30\n"
    );
    let graph = String::from_utf8(
        file.run(
            "xs = [1, 3]\nys = [2, 4]\npairs = xs.zip(ys)\npairs",
            &["--json", "--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        graph.contains(
            "  xs [variable]\n    updates  pairs\n  ys [variable]\n    updates  pairs\n  pairs [variable]\n    reads    xs, ys"
        ),
        "{graph}"
    );
    let populations = String::from_utf8(
        file.run(
            "class Driver(name):\n    this.name = name\nclass Truck(plate):\n    this.plate = plate\ncrews = Driver.zip(Truck)\nDriver(\"a\")\nTruck(\"b\")\ncrews.length",
            &["--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        populations.contains(
            "  $Driver [class]\n    updates  crews\n  $Truck [class]\n    updates  crews\n  crews [variable]\n    reads    $Driver, $Truck"
        ),
        "{populations}"
    );
    let block = String::from_utf8(
        file.run(
            "xs = [1, 3]\nys = [2, 4]\nif xs.zip(ys).length > 1:\n    flag = true\nflag",
            &["--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        block.contains("  xs [variable]\n    updates  if(xs.zip(ys).length>1)\n  ys [variable]\n    updates  if(xs.zip(ys).length>1)\n  if(xs.zip(ys).length>1) [if]\n    reads    xs, ys"),
        "{block}"
    );
    for (source, message) in [
        (
            "xs = [1]\nxs.zip(5)",
            "TypeError: zip expects a list (line 2, column 1)\n",
        ),
        (
            "xs = [1]\nxs.zip([2], 5)",
            "TypeError: zip expects a function (line 2, column 1)\n",
        ),
    ] {
        let failed = file.run(source, &["--json"]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(String::from_utf8(failed.stderr).unwrap(), message);
    }
    let session = "xs = [1, 3]\nys = [2, 4, 6]\np = xs.zip(ys)\np\nxs.push(5)\np\nxs.zip(5)\n.clear\nxs = [2, 6]\nxs.zip([1, 2], (a, b) -> a * b)\n.exit\n";
    for (options, expected) in [
        (
            &["--plain"][..],
            "1,3\n2,4,6\n1,2,3,4\n1,2,3,4\n3\n1,2,3,4,5,6\nState cleared.\n2,6\n2,12\n",
        ),
        (
            &["--plain", "--json"][..],
            "[1.0,3.0]\n[2.0,4.0,6.0]\n[[1.0,2.0],[3.0,4.0]]\n[[1.0,2.0],[3.0,4.0]]\n3.0\n[[1.0,2.0],[3.0,4.0],[5.0,6.0]]\nState cleared.\n[2.0,6.0]\n[2.0,12.0]\n",
        ),
        (
            &["--json"][..],
            "[1.0,3.0]\n[2.0,4.0,6.0]\n[[1.0,2.0],[3.0,4.0]]\n[[1.0,2.0],[3.0,4.0]]\n3.0\n[[1.0,2.0],[3.0,4.0],[5.0,6.0]]\nState cleared.\n[2.0,6.0]\n[2.0,12.0]\n",
        ),
    ] {
        for newline in ["\n", "\r\n"] {
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.ends_with(expected), "{options:?} {stdout}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                "TypeError: zip expects a list (line 1, column 1)\n"
            );
        }
    }
}
