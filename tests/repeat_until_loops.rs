#[expect(dead_code)]
#[path = "../build.rs"]
mod build_exporter;

mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::collections::BTreeSet;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use build_exporter::build_statements::statements as exported_statements;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::{generate, generate_all};
use nucleoid::lang::ast::{BinaryOp, Expr, Parameter, Stmt, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde::Deserialize;

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid repeats a block until its condition holds",
    "Nucleoid returns from inside repeat-until loops in methods",
    "Nucleoid nests repeat-until loops inside other loops",
    "Nucleoid rolls back a repeat-until loop that fails during an iteration",
];
const REFERENCE_TITLE: &str = "Nucleoid repeats statements until a condition holds";
const README_TITLE: &str = "Repeat-until loops move a lift to the next call";
const EXAMPLES_TITLE: &str = "Repeat-until loops give every ferry enough lifeboats";

const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid repeats a block until its condition holds",
    "source": "# Nucleoid repeats a block until its condition holds\n\nminute = 0\nchimes = minute % 15 == 0\nwaited = 0\nwhile chimes == !waited: minute += 5; waited += 5\nassert([minute, waited, chimes], [15, 15, true])\nwhile chimes != waited > 15: minute += 5; waited += 5\nassert([minute, waited], [30, 30])\nminute = 32\nassert([chimes, waited], [false, 30])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid returns from inside repeat-until loops in methods",
    "source": "# Nucleoid returns from inside repeat-until loops in methods\n\nclass Drone(battery):\n    this.battery = battery\n    this.distance = 0\n    def fly(target):\n        while true:\n            this.battery -= 10\n            this.distance += 3\n            if this.empty: return \"stranded\"\n            if this.distance >= target: return\n$Drone.empty = $Drone.battery < 10\nscout = Drone(40)\nassert([scout.fly(6), scout.distance, scout.empty], [null, 6, false])\nassert([scout.fly(0), scout.distance, scout.battery], [null, 9, 10])\nassert([scout.fly(20), scout.distance, scout.empty], [\"stranded\", 12, true])\nspare = Drone(25)\nassert([spare.fly(6), spare.distance], [\"stranded\", 6])\nscout.battery = 50\nassert([scout.empty, spare.empty], [false, true])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid nests repeat-until loops inside other loops",
    "source": "# Nucleoid nests repeat-until loops inside other loops\n\nclass Truck(minimum):\n    this.minimum = minimum\n    this.load = 0\n    this.crates = 0\n$Truck.full = $Truck.load >= $Truck.minimum\nTruck(6)\nTruck(0)\nTruck(7)\nassert(Truck.map(truck -> truck.full), [false, true, false])\ndock = [4, 3, 5, 2, 6, 1]\nshipped = []\nfor truck of Truck:\n    while [truck.load += dock.shift(), truck.crates += 1] && !truck.full: pass\n    shipped.push(truck.load)\nassert([shipped, dock], [[7, 5, 8], [1]])\nTruck(3)\nassert(Truck.map(truck -> truck.crates), [2, 1, 2, 0])\nTruck.map(truck -> truck.full)",
    "expected": "[true,true,true,false]"
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back a repeat-until loop that fails during an iteration",
    "source": "# Nucleoid rolls back a repeat-until loop that fails during an iteration\n\nclass Oven(temp):\n    this.temp = temp\n    this.steps = 0\n    def heat():\n        while true: this.temp += 10; this.steps += 1; if this.ready: return\nmarks = [150, 180, 220]\n$Oven.ready = marks.includes($Oven.temp)\nif $Oven.temp > 200:\n    throw \"OVERHEAT\"\noven = Oven(150)\noven.heat()\nassert([oven.temp, oven.steps, oven.ready], [180, 3, true])\ntry:\n    oven.heat()\ncatch error:\n    failure = error\nassert([failure, oven.temp, oven.steps], [\"OVERHEAT\", 180, 3])\nmarks.push(190)\noven.heat()\n[oven.temp, oven.steps]",
    "expected": "[190,4]"
  },
  {
    "document": "reference",
    "title": "Nucleoid repeats statements until a condition holds",
    "source": "# Nucleoid repeats statements until a condition holds\n\ndays = [1, 2, 4, 5]\ntoday = 2\nopen = days.includes(today)\nwhile open != today > 2: today += 1\nassert([today, open], [4, true])\ndays.push(3)\ntoday",
    "expected": "4"
  },
  {
    "document": "readme",
    "title": "Repeat-until loops move a lift to the next call",
    "source": "class Lift(floor):\n    this.floor = floor\nlift = Lift(1)\ncalls = [1, 4, 6]\nstopped = calls.includes(lift.floor)\nwhile [lift.floor += 1] && !stopped: pass\nassert(lift.floor, 4)\ncalls.push(2)\nassert([lift.floor, stopped], [4, true])\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Repeat-until loops give every ferry enough lifeboats",
    "source": "class Lifeboat(ferry):\n    this.ferry = ferry\n    this.seats = 20\nclass Ferry(passengers):\n    this.passengers = passengers\n    this.crew = 0\n    def equip():\n        while true: Lifeboat(this); this.crew += 2; if this.safe: return\n$Ferry.seats = Lifeboat.filter(boat -> boat.ferry == $Ferry).sum(boat -> boat.seats)\n$Ferry.safe = $Ferry.seats >= $Ferry.passengers + $Ferry.crew\nbig = Ferry(50)\ndinghy = Ferry(0)\nbig.equip()\ndinghy.equip()\nassert([big.crew, dinghy.crew, Lifeboat.length], [6, 2, 4])\nbig.passengers = 60\n[big.seats, big.safe, dinghy.safe]\n",
    "expected": "[60,false,true]"
  }
]"###;

const WRITTEN_OUT: [(&str, &str); 9] = [
    (
        "repeat: minute += 5; waited += 5\nuntil chimes\nassert([minute, waited, chimes]",
        "while chimes == !waited: minute += 5; waited += 5\nassert([minute, waited, chimes]",
    ),
    (
        "repeat: minute += 5; waited += 5\nuntil chimes\nassert([minute, waited]",
        "while chimes != waited > 15: minute += 5; waited += 5\nassert([minute, waited]",
    ),
    (
        "        repeat:\n            this.battery -= 10\n",
        "        while true:\n            this.battery -= 10\n",
    ),
    (
        "\n        until this.distance >= target\n",
        "\n            if this.distance >= target: return\n",
    ),
    (
        "    repeat: truck.load += dock.shift(); truck.crates += 1\n    until truck.full\n",
        "    while [truck.load += dock.shift(), truck.crates += 1] && !truck.full: pass\n",
    ),
    (
        "        repeat: this.temp += 10; this.steps += 1\n        until this.ready\n",
        "        while true: this.temp += 10; this.steps += 1; if this.ready: return\n",
    ),
    (
        "repeat: today += 1\nuntil open\n",
        "while open != today > 2: today += 1\n",
    ),
    (
        "repeat: lift.floor += 1\nuntil stopped\n",
        "while [lift.floor += 1] && !stopped: pass\n",
    ),
    (
        "        repeat: Lifeboat(this); this.crew += 2\n        until this.safe\n",
        "        while true: Lifeboat(this); this.crew += 2; if this.safe: return\n",
    ),
];

const MISSING_UNTIL: &str = "Expected 'until' after 'repeat'";
const UNBOUNDED: &str = "Loop did not terminate";

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

fn loops(statements: &[Stmt]) -> usize {
    format!("{statements:?}").matches("RepeatUntil {").count()
}

fn written_out(case: &common::Case) -> (String, usize) {
    let mut written = case.source.clone();
    let mut rewrites = 0;
    for (looped, accepted) in WRITTEN_OUT {
        rewrites += written.matches(looped).count();
        written = written.replace(looped, accepted);
    }
    (written, rewrites)
}

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    assert!(count > 0, "{source}");
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "repeat-until loop regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
}

fn run(runtime: &mut Runtime, source: &str) -> Value {
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn json(runtime: &mut Runtime, source: &str) -> String {
    let value = run(runtime, source);
    runtime.serialize_json(&value).unwrap()
}

fn evaluate(source: &str) -> String {
    json(&mut Runtime::new(), source)
}

fn outcome(source: &str) -> String {
    let mut runtime = Runtime::new();
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

fn same_as(source: &str, accepted: &str) -> String {
    let looped = outcome(source);
    assert_eq!(looped, outcome(accepted), "{source}");
    looped
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

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn number(value: f64) -> Expr {
    Expr::Number(value)
}

fn binary(operator: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::Binary {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn increment(target: &str) -> Stmt {
    Stmt::Assign {
        target: name(target),
        value: binary(BinaryOp::Add, name(target), number(1.0)),
    }
}

fn keys(runtime: &Runtime) -> BTreeSet<String> {
    runtime
        .graph
        .keys()
        .map(|key| key.as_str().to_string())
        .collect()
}

#[test]
fn only_the_appended_cases_repeat_until_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    let mut found = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let statements =
            parse(&case.source).unwrap_or_else(|error| panic!("{}: {error}", case.title));
        let new = is_appended(&case.title);
        assert_eq!(loops(&statements) > 0, new, "{}", case.title);
        found += loops(&statements);
        if new {
            assert!(
                !format!("{statements:?}").contains("Repeat { count: "),
                "{}",
                case.title
            );
            let count = common::assertions::protected_assertions(&statements).len();
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
    assert_eq!(found, 8);
    assert_eq!(assertions, 17);
    assert_eq!(cases, 1264);
}

#[test]
fn the_appended_cases_stand_after_nuc_125_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[815],
        "Nucleoid rolls back failed updates through stepped slices"
    );
    assert_eq!(titles[816..820], SPEC_TITLES);
    assert_eq!(titles[832], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid walks lists from a position with stepped slices",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Stepped slices deal the chores round-robin as roommates move in",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Stepped slices mirror the rows of every sprite",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(index, cases.len() - 8, "{title}");
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 4].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### repeat until\n\n```\n")
        .expect("the language reference documents repeat-until loops");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 126](nuc-0126.md)"));
    assert!(reference.contains("`TypeError(\"Loop did not terminate\")`"));
    assert!(reference.contains("`SyntaxError(\"Expected 'until' after 'repeat'\")`"));
    assert!(reference.contains(
        "[exit-controlled repeat-until loops](../nucleoid.spec.md#exit-controlled-repeat-until-loops)"
    ));
    assert_eq!(case.expected.as_deref(), Some("4"));
}

#[test]
fn every_case_is_its_accepted_counterpart_with_only_the_loops_written_out() {
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
        assert_eq!(loops(&parse(&written).unwrap()), 0, "{}", sealed.title);
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
fn assertion_coverage_visits_the_body_and_then_the_condition() {
    let source =
        "n = 0\nrepeat:\n    n = n + 1\n    assert(n > 0, true)\nuntil assert(n, n) ?? n >= 2\nn";
    let statements = parse(source).unwrap();
    assert_eq!(
        common::assertions::protected_assertions(&statements),
        ["assert(n>0,true)", "assert(n,n)"]
    );
    let mut runtime = Runtime::new();
    let value = common::check_with_count(
        &mut runtime,
        &common::Case {
            title: "assertions in a repeat-until loop".to_string(),
            source: source.to_string(),
            expected: Some("2".to_string()),
        },
        2,
    )
    .unwrap();
    assert_eq!(value, Value::Number(2.0));
    assert_eq!(runtime.assertions_run(), 4);
}

#[test]
fn the_exporter_keeps_every_until_line_with_its_loop() {
    let mut exported = 0;
    for case in documents()
        .iter()
        .flat_map(|(_, cases)| cases)
        .filter(|case| is_appended(&case.title))
    {
        let parts = exported_statements(&case.source);
        let whole = parse(&case.source).unwrap();
        let split: Vec<Stmt> = parts
            .iter()
            .flat_map(|part| {
                parse(part).unwrap_or_else(|error| panic!("{}: {part}\n{error}", case.title))
            })
            .collect();
        assert_eq!(whole, split, "{}", case.title);
        for part in &parts {
            assert!(
                !part.trim_start().starts_with("until"),
                "{}: {part}",
                case.title
            );
        }
        exported += parts.len();
    }
    assert_eq!(exported, 70);
    for source in [
        "x = 1\nrepeat: x = x * 2\nuntil x > 100\ny = x",
        "x = 1\nrepeat:\n    x = x * 2\nuntil x > 100\ny = x",
        "x = 1\nrepeat: x = x * 2\n\n# then\nuntil x > 100\ny = x",
    ] {
        let parts = exported_statements(source);
        assert_eq!(parts.len(), 3, "{source}\n{parts:#?}");
        assert_eq!(parts[2], "y = x", "{source}");
        let whole = parse(source).unwrap();
        let split: Vec<Stmt> = parts.iter().flat_map(|part| parse(part).unwrap()).collect();
        assert_eq!(whole, split, "{source}");
        assert!(matches!(whole[1], Stmt::RepeatUntil { .. }), "{source}");
    }
    assert_eq!(
        exported_statements("until = 1\nuntil_ready = 2\nuntilx = 3"),
        ["until = 1", "until_ready = 2", "untilx = 3"]
    );
    let source = "x = 1\nuntil = 5\nx + until";
    let parts = exported_statements(source);
    assert_eq!(parts, ["x = 1\nuntil = 5", "x + until"]);
    let split: Vec<Stmt> = parts.iter().flat_map(|part| parse(part).unwrap()).collect();
    assert_eq!(split, parse(source).unwrap());
}

#[test]
fn repeat_until_loops_build_one_statement_with_canonical_renderings() {
    let statements = parse("repeat:\n    n = n + 1\nuntil n > 3").unwrap();
    assert_eq!(
        statements,
        [Stmt::RepeatUntil {
            body: vec![increment("n")],
            condition: binary(BinaryOp::Greater, name("n"), number(3.0)),
        }]
    );
    assert_eq!(generate_all(&statements), "repeat{n=n+1}until(n>3)");
    assert!(!format!("{statements:?}").contains("Repeat { count: "));

    equivalent(
        "repeat: n = n + 1\nuntil n > 3",
        "repeat:\n    n = n + 1\nuntil n > 3",
    );
    equivalent(
        "repeat: n = n + 1; seen = n\nuntil n > 3",
        "repeat:\n    n = n + 1\n    seen = n\nuntil n > 3",
    );
    equivalent(
        "repeat:\n    n = n + 1\n\n# done?\n\nuntil n > 3",
        "repeat:\n\n    # comment\n    n = n + 1\nuntil n > 3",
    );
    equivalent(
        "repeat: n = n + 1\n  # a comment line\nuntil n > 3;",
        "repeat: n = n + 1\nuntil n > 3",
    );
    equivalent(
        "repeat:\r\n    n = n + 1\r\n\r\nuntil n > 3\r\n",
        "repeat:\n    n = n + 1\nuntil n > 3\n",
    );
    equivalent(
        "repeat: n = n + 1\nuntil (n > 3)",
        "repeat: n = n + 1\nuntil n > 3",
    );
    assert_eq!(
        generate_all(
            &parse("repeat:\n    repeat: b()\n    until c\n    d = 1\nuntil e\nf()").unwrap()
        ),
        "repeat{repeat{b()}until(c);d=1}until(e);f()"
    );
    assert_eq!(
        generate_all(&parse("repeat: tick()\nuntil ready ? a : b").unwrap()),
        "repeat{tick()}until((ready?a:b))"
    );
    assert_eq!(
        NodeKey::block(&parse("repeat: n = n + 1\nuntil n > 3").unwrap(), None).as_str(),
        "block(repeat{n=n+1}until(n>3))"
    );

    let source = "def step(n = 3):\n    repeat:\n        n = n - 1\n    until n < 1\n    return n";
    let function = parse(source).unwrap();
    let rendered = generate(&function[0]);
    assert_eq!(
        rendered,
        "def step(n=3):\n    repeat:\n        n=n-1\n    until n<1\n    return n"
    );
    assert_eq!(parse(&rendered).unwrap(), function);
    let nested = parse(
        "def fill(n = 2):\n    repeat:\n        repeat: n = n + 1\n        until n > 4\n    until n > 9\n    return n",
    )
    .unwrap();
    let rendered = generate(&nested[0]);
    assert_eq!(
        rendered,
        "def fill(n=2):\n    repeat:\n        repeat:\n            n=n+1\n        until n>4\n    until n>9\n    return n"
    );
    assert_eq!(parse(&rendered).unwrap(), nested);
    assert_eq!(evaluate(&format!("{rendered}\nfill()")), "10.0");
}

#[test]
fn loops_stand_wherever_a_statement_starts() {
    for (source, expected) in [
        (
            "n = 0\nif true:\n    repeat: n += 1\n    until n > 2\nn",
            "3.0",
        ),
        ("n = 0\nif true: repeat: n += 1\nuntil n > 2\nn", "3.0"),
        (
            "n = 0\nif false: pass\nelse:\n    repeat:\n        n += 1\n    until n > 2\nn",
            "3.0",
        ),
        (
            "n = 0\n{\n    repeat: n = n + 1\n    until n > 3\n}\nn",
            "4.0",
        ),
        ("n = 0\n{\nrepeat: n = n + 1\nuntil n > 3\n}\nn", "4.0"),
        (
            "def f(n):\n    repeat: n = n * 2\n    until n > 50\n    return n\nf(3)",
            "96.0",
        ),
        (
            "f = (n) => {\n    repeat: n = n * 2\n    until n > 50\n    return n\n}\nf(3)",
            "96.0",
        ),
        (
            "f = function(n) {\n    repeat:\n        n = n * 3\n    until n > 50\n    return n\n}\nf(2)",
            "54.0",
        ),
        (
            "class Gauge(level):\n    repeat: level = level + 5\n    until level >= 20\n    this.level = level\nGauge(3).level",
            "23.0",
        ),
        (
            "n = 0\nwhile n < 10:\n    repeat: n += 1\n    until n % 4 == 0\nn",
            "12.0",
        ),
        (
            "n = 0\nrepeat 2:\n    repeat: n += 1\n    until n % 3 == 0\nn",
            "6.0",
        ),
        (
            "class K(v):\n    this.v = v\nK(1)\nK(2)\nK(3)\nn = 0\nfor k of K:\n    repeat: n += k.v\n    until n % 2 == 0\nn",
            "10.0",
        ),
        (
            "n = 0\nrepeat:\n    repeat:\n        n += 1\n    until n % 3 == 0\n    n += 10\nuntil n > 20\nn",
            "25.0",
        ),
        (
            "n = 0\ntry:\n    repeat: n += 1\n    until n > 2\ncatch error:\n    n = -1\nfinally:\n    n = n * 10\nn",
            "30.0",
        ),
        (
            "n = 0\nm = suppose:\n    repeat: n += 1\n    until n >= 3\n    n\n[m, n]",
            "[3.0,0.0]",
        ),
    ] {
        let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert!(loops(&statements) > 0, "{source}");
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn an_until_line_between_the_body_and_repeat_closes_the_loop() {
    for (source, aligned, expected) in [
        (
            "n = 0\nrepeat:\n    n = n + 1\n  until n > 3\nn",
            "n = 0\nrepeat:\n    n = n + 1\nuntil n > 3\nn",
            "4.0",
        ),
        (
            "n = 0\nif true:\n    repeat:\n        n = n + 1\n      until n > 3\n    n = n * 10\nn",
            "n = 0\nif true:\n    repeat:\n        n = n + 1\n    until n > 3\n    n = n * 10\nn",
            "40.0",
        ),
        (
            "n = 0\nrepeat:\n    n = n + 1\n\n # a comment\n  until n > 3\nn",
            "n = 0\nrepeat:\n    n = n + 1\nuntil n > 3\nn",
            "4.0",
        ),
    ] {
        let statements = equivalent(source, aligned);
        assert_eq!(loops(&statements), 1, "{source}");
        assert_eq!(evaluate(source), expected, "{source}");
    }
    assert_eq!(
        evaluate("a = 0\nif false:\n    a = 1\n  else: a = 2\na"),
        "2.0"
    );
    assert_eq!(
        syntax_error("n = 0\nif true:\n    repeat:\n        n = n + 1\n  until n > 3\nn"),
        (MISSING_UNTIL.to_string(), Position::new(5, 3))
    );
}

#[test]
fn until_closes_the_innermost_same_line_repeat_at_its_line() {
    let statements = parse("for t of T: repeat: t.l += 1\nuntil t.full; t.done = true").unwrap();
    let [Stmt::For { body, .. }] = statements.as_slice() else {
        panic!("{statements:?}");
    };
    assert!(
        matches!(
            body.as_slice(),
            [Stmt::RepeatUntil { .. }, Stmt::Assign { .. }]
        ),
        "{body:?}"
    );
    assert_eq!(
        evaluate(
            "class T(m):\n    this.m = m\n    this.l = 0\n$T.full = $T.l >= $T.m\nT(2)\nT(0)\nfor t of T: repeat: t.l += 1\nuntil t.full; t.done = true\nT.map(t -> [t.l, t.done])"
        ),
        "[[2.0,true],[1.0,true]]"
    );
    assert_eq!(
        evaluate("a = 0\nif true: if false: a = 1\nelse: a = 2\na"),
        "2.0"
    );
    assert_eq!(
        evaluate("n = 0\nif true: repeat: n += 1\nuntil n > 1\nn"),
        "2.0"
    );
    assert_eq!(
        evaluate("x = 0\nrepeat: x += 1\nuntil x > 3; y = x * 10\n[x, y]"),
        "[4.0,40.0]"
    );
}

#[test]
fn the_condition_is_parsed_like_an_if_header_and_ends_the_line() {
    for (source, expected) in [
        (
            "c = true\nx = 0\nrepeat: x = x + 1\nuntil c ? x >= 2 : x >= 5\nx",
            "2.0",
        ),
        (
            "xs = [3, 1, 2]\nrepeat: xs = xs.map(v -> v + 1)\nuntil all v in xs: v > 4\nxs",
            "[7.0,5.0,6.0]",
        ),
        (
            "xs = [3, 1, 2]\nrepeat: xs.push(xs.length)\nuntil any v in xs: v > 4\nxs",
            "[3.0,1.0,2.0,3.0,4.0,5.0]",
        ),
        (
            "o = {}\nn = 0\nrepeat: n = n + 1\nuntil (o.limit ?? 3) <= n\nn",
            "3.0",
        ),
        (
            "price = 2\ntotal = price * 3\nrepeat: price = price + 1\nuntil total > 10 && (total |> why).length > 1\nprice",
            "4.0",
        ),
        (
            "xs = []\nrepeat: xs.push(xs.length)\nuntil xs.some(x -> x > 2)\nxs",
            "[0.0,1.0,2.0,3.0]",
        ),
        (
            "n = 0\nrepeat: n = n + 1\nuntil (lambda: n > 2)()\nn",
            "3.0",
        ),
        (
            "n = 0\nrepeat: n = n + 1\nuntil n not in [0, 1, 2]\nn",
            "3.0",
        ),
        (
            "n = 0\nrepeat: n = n + 1\nuntil n >= 2; n = n * 10\nn",
            "20.0",
        ),
        ("n = 0\nrepeat:\n    n = n + 1\nuntil n >= 2\n\n\nn", "2.0"),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
    for (source, own_line, expected) in [
        (
            "x = 0\n{\nrepeat: x += 1\nuntil x > 3 }\nx",
            "x = 0\n{\nrepeat: x += 1\nuntil x > 3\n}\nx",
            "4.0",
        ),
        (
            "x = 0\nf = () -> { repeat: x += 1\nuntil x > 3 }\nf()\nx",
            "x = 0\nf = () -> { repeat: x += 1\nuntil x > 3\n}\nf()\nx",
            "4.0",
        ),
        (
            "x = 0\n{ if true: repeat: x += 1\nuntil x > 3 }\nx",
            "x = 0\n{ if true: repeat: x += 1\nuntil x > 3\n}\nx",
            "4.0",
        ),
    ] {
        assert_eq!(parse(source).unwrap(), parse(own_line).unwrap(), "{source}");
        assert_eq!(evaluate(source), expected, "{source}");
        assert_eq!(evaluate(own_line), expected, "{own_line}");
    }
    for (source, message, line, column) in [
        (
            "x = 1\nrepeat:\n    x = x * 2\nuntil x > 100:\n    x",
            "Unexpected ':'",
            4,
            14,
        ),
        (
            "x = 0\nrepeat: x += 1\nuntil x > 3 y",
            "Unexpected identifier 'y'",
            3,
            13,
        ),
        (
            "x = 0\nrepeat: x += 1\nuntil x > 3 else: x = 1",
            "Unexpected keyword 'else'",
            3,
            13,
        ),
        (
            "x = 0\nrepeat: x += 1\nuntil x > 3 }",
            "Unexpected '}'",
            3,
            14,
        ),
        (
            "x = 0\nrepeat: x += 1\nuntil x > 3 }\nx",
            "Unexpected '}'",
            3,
            13,
        ),
        (
            "x = 0\nrepeat: x += 1\nuntil x > 3 4",
            "Unexpected number 4",
            3,
            13,
        ),
        (
            "x = 1\nrepeat:\n    x = 2\nuntil",
            "Unexpected end of input",
            4,
            6,
        ),
        (
            "x = 1\nrepeat: x = 2\nuntil\nx",
            "Unexpected end of line",
            4,
            1,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn repeat_until_keeps_every_accepted_reading_of_both_names() {
    let declaration = |type_name: &str| Stmt::Declaration {
        name: "repeat".to_string(),
        type_name: type_name.to_string(),
    };
    assert_eq!(
        parse("repeat: tick\nuntil done").unwrap(),
        [
            declaration("tick"),
            Stmt::Expression(name("until")),
            Stmt::Expression(name("done")),
        ]
    );
    assert_eq!(
        parse("repeat: tick\n    n = n + 1\nuntil done").unwrap(),
        [
            declaration("tick"),
            increment("n"),
            Stmt::Expression(name("until")),
            Stmt::Expression(name("done")),
        ]
    );
    assert_eq!(
        parse("repeat: x; until y").unwrap(),
        [
            declaration("x"),
            Stmt::Expression(name("until")),
            Stmt::Expression(name("y")),
        ]
    );
    for word in [
        "int", "assert", "lambda", "suppose", "every", "is", "until", "repeat",
    ] {
        assert_eq!(
            parse(&format!("repeat: {word}")).unwrap(),
            [declaration(word)],
            "{word}"
        );
    }
    assert_eq!(
        parse("{\n    repeat: int\n}").unwrap(),
        [Stmt::Block(vec![declaration("int")])]
    );
    let Stmt::Class(gate) = &parse("class Gate:\n    repeat: int\n    until: int").unwrap()[0]
    else {
        panic!("a class");
    };
    let fields: Vec<&Parameter> = gate.fields.iter().collect();
    assert_eq!(
        fields
            .iter()
            .map(|field| (field.name.as_str(), field.type_name.as_deref()))
            .collect::<Vec<_>>(),
        [("repeat", Some("int")), ("until", Some("int"))]
    );
    assert!(gate.constructor.is_empty());
    assert_eq!(
        parse("repeat := 1").unwrap(),
        [Stmt::ValueAssign {
            target: name("repeat"),
            value: number(1.0),
        }]
    );
    assert_eq!(
        parse("until = 5").unwrap(),
        [Stmt::Assign {
            target: name("until"),
            value: number(5.0),
        }]
    );
    assert_eq!(
        parse("until x > 3").unwrap(),
        [
            Stmt::Expression(name("until")),
            Stmt::Expression(binary(BinaryOp::Greater, name("x"), number(3.0))),
        ]
    );
    assert!(matches!(
        parse("repeat 3: tick()").unwrap().as_slice(),
        [Stmt::Repeat { .. }]
    ));
    assert_eq!(
        parse_expression("repeat:\n    x = 1\nuntil x").unwrap(),
        name("repeat")
    );
    assert_eq!(parse_expression("until x > 3").unwrap(), name("until"));
    for source in [
        "repeat(4)",
        "\"ab\".repeat(3)",
        "point = {repeat: 1, until: 2}",
        "for until of [1, 2]:\n    x = until",
        "def until(x) => x + 1",
        "def f(repeat, until) => repeat + until",
        "o.until = 1\no.repeat = 2",
        "repeat ready: tick",
        "repeat = until",
    ] {
        assert_eq!(loops(&parse(source).unwrap()), 0, "{source}");
    }

    for (source, expected) in [
        ("repeat := 3\nrepeat", "3.0"),
        ("repeat = 2\nrepeat", "2.0"),
        ("until = 5\nuntil", "5.0"),
        ("\"ab\".repeat(3)", "\"ababab\""),
        (
            "o = {repeat: 1, until: 2}\n[o.repeat, o.until]",
            "[1.0,2.0]",
        ),
        ("x = 0\nfor until in [1, 2]:\n    x = until\nx", "0.0"),
        ("def until(x) => x + 1\nuntil(4)", "5.0"),
        ("def f(repeat, until) => repeat * until\nf(3, 4)", "12.0"),
        ("n = 0\nrepeat 3:\n    n = n + 1\nn", "3.0"),
        ("x = 1\nwhile x < 100:\n    x = x * 2\nx", "128.0"),
        ("until = 1\nx = 3\nuntil x > 2", "true"),
        ("repeat = 3\nuntil = 4\nrepeat + until", "7.0"),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
    for (source, kind, message, line, column) in [
        (
            "repeat: tick\nuntil done",
            ErrorKind::Reference,
            "Missing definition",
            1,
            1,
        ),
        (
            "x = 5\nrepeat: x\nuntil x > 3",
            ErrorKind::Reference,
            "Missing definition",
            2,
            1,
        ),
        (
            "repeat: int\nrepeat",
            ErrorKind::Reference,
            "Missing definition",
            1,
            1,
        ),
        (
            "until x > 3",
            ErrorKind::Reference,
            "until is not defined",
            1,
            1,
        ),
    ] {
        assert_eq!(
            failure(source),
            (kind, message.to_string(), Some(Position::new(line, column))),
            "{source}"
        );
    }
}

#[test]
fn colon_statements_after_repeat_become_loop_bodies() {
    for (source, expected) in [
        ("repeat: assert: x, 1\nuntil true", "assert(x,1)"),
        ("repeat: lambda: 1\nuntil true", "function()"),
        ("repeat: suppose: x\nuntil true", "suppose:x"),
        ("repeat: suppose: x = 1\nuntil true", "suppose{x=1}"),
        ("repeat: suppose:\n    x = 1\nuntil true", "suppose{x=1}"),
        ("repeat: x := 1\nuntil true", "x:=1"),
        ("repeat: x: int; y = 1\nuntil true", "x:int;y=1"),
    ] {
        let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        let [Stmt::RepeatUntil { body, condition }] = statements.as_slice() else {
            panic!("{source}: {statements:?}");
        };
        assert_eq!(condition, &Expr::Bool(true), "{source}");
        assert_eq!(generate_all(body), expected, "{source}");
    }
    for (source, expected) in [
        ("x = 1\nrepeat: assert: x, 1\nuntil true\nx", "1.0"),
        ("n = 0\nrepeat: n := n + 1\nuntil n > 1\nn", "2.0"),
        ("x = 1\nrepeat: suppose: x = 5\nuntil true\nx", "1.0"),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn malformed_loops_report_exact_errors() {
    for (source, message, line, column) in [
        ("x = 1\nrepeat:\n    x = 2\ny = 3", MISSING_UNTIL, 4, 1),
        ("x = 1\nrepeat: x = 2\ny = 3", MISSING_UNTIL, 3, 1),
        ("x = 1\nrepeat: x = 2", MISSING_UNTIL, 2, 14),
        ("x = 1\nrepeat:\n    x = 2", MISSING_UNTIL, 3, 10),
        ("x = 0\nrepeat: x += 1; until x > 3", MISSING_UNTIL, 2, 23),
        ("x = 0\nrepeat: x += 1 until x > 3", MISSING_UNTIL, 2, 16),
        ("x = 0\nrepeat: x += 1\n;\nuntil x > 3", MISSING_UNTIL, 3, 1),
        ("x = 0\nrepeat: x += 1\n; until x > 3", MISSING_UNTIL, 3, 1),
        (
            "x = 0\nrepeat:\n    x += 1\n;\nuntil x > 3",
            MISSING_UNTIL,
            4,
            1,
        ),
        (
            "x = 0\nrepeat: x += 1\n    until x > 3",
            MISSING_UNTIL,
            3,
            5,
        ),
        (
            "x = 0\nrepeat:\n    x += 1\n    until x > 3",
            MISSING_UNTIL,
            4,
            16,
        ),
        (
            "x = 0\nif true:\n    repeat: x += 1\nuntil x > 3\nx",
            MISSING_UNTIL,
            4,
            1,
        ),
        (
            "x = 0\nif true:\n    repeat:\n        x += 1\nuntil x > 3",
            MISSING_UNTIL,
            5,
            1,
        ),
        (
            "f = [1].map(n -> { repeat: n += 1\nuntil n > 3 })",
            MISSING_UNTIL,
            2,
            1,
        ),
        (
            "x = [repeat: 1\nuntil x]",
            "Expected ']' but found ':'",
            1,
            12,
        ),
        ("repeat:", "Unexpected end of input", 1, 8),
        (
            "x = 0\nrepeat:\n",
            "Expected an indented block but found end of input",
            3,
            1,
        ),
        (
            "x = 0\nrepeat:\nx = 1\nuntil x",
            "Expected an indented block but found identifier 'x'",
            3,
            1,
        ),
        ("x = 0\nrepeat:;", "Unexpected ';'", 2, 9),
        ("x = 0\nrepeat:: pass\nuntil x", "Unexpected ':'", 2, 10),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn the_body_runs_first_and_the_condition_after_every_iteration() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
log = []
n = 0
def check():
    log.push("test")
    return n >= 2
repeat:
    log.push("body")
    n += 1
until check()
assert(log, ["body", "test", "body", "test"])
done = true
k = 0
repeat: k += 1
until done
assert(k, 1)
assert(n, 2)
"#,
    );
    assert_eq!(
        same_as(
            "x = 500\nrepeat:\n    x = x + 1\nuntil x > 100\nx",
            "x = 500\nx = x + 1\nwhile !(x > 100):\n    x = x + 1\nx"
        ),
        "501.0"
    );
    assert_eq!(evaluate("x = 1\nrepeat: x = x * 2\nuntil x > 100"), "null");
    for value in [
        "0",
        "1",
        "-1",
        "\"\"",
        "\"0\"",
        "\"a\"",
        "null",
        "undefined",
        "false",
        "true",
        "[]",
        "[0]",
        "{}",
        "0 / 0",
        "Number.POSITIVE_INFINITY",
        "missing.value",
    ] {
        let looped = evaluate(&format!(
            "missing = {{}}\nn = 0\nc = {value}\nrepeat:\n    n += 1\n    if n == 3: c = true\nuntil c\nn"
        ));
        let tested = evaluate(&format!(
            "missing = {{}}\nc = {value}\nif c:\n    n = 1\nelse:\n    n = 3\nn"
        ));
        assert_eq!(looped, tested, "{value}");
    }
}

#[test]
fn each_iteration_sees_the_propagation_its_assignments_cause() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
level = 0
doubled = level * 2
seen = []
repeat:
    level = level + 1
    seen.push(doubled)
until doubled >= 6
assert(level, 3)
assert(seen, [2, 4, 6])
"#,
    );
    checked(
        &mut runtime,
        r#"
class Cell(size):
    this.size = size
$Cell.big = $Cell.size > 1
big = Cell.filter(cell => cell.big).length
repeat: Cell(Cell.length)
until big >= 2
assert(Cell.length, 4)
assert(big, 2)
"#,
    );
    checked(
        &mut runtime,
        r#"
n = 0
repeat: n = n + 1
until n.value >= 3
assert(n, 3)
x = 1
y = x
repeat: x = x + 1
until y.value > 4
assert([x, y], [5, 5])
"#,
    );
}

#[test]
fn the_bound_allows_100000_iterations_and_rolls_back_past_it() {
    assert_eq!(
        evaluate("n = 0\nrepeat: n = n + 1\nuntil n >= 100000\nn"),
        "100000.0"
    );
    let mut runtime = Runtime::new();
    run(&mut runtime, "n = 5\ndoubled = n * 2\ntrace = []");
    let mut control = Runtime::new();
    run(&mut control, "n = 5\ndoubled = n * 2\ntrace = []");
    for (source, line) in [
        ("n = 0\nrepeat: n = n + 1\nuntil n > 100000\nn", 2),
        ("trace.push(1)\nrepeat: n = n + 1\nuntil n < 0", 2),
        ("repeat: n = n + 1\nuntil false", 1),
        ("repeat:\n    n = 0\nuntil n", 1),
        ("def spin():\n    repeat: pass\n    until false\nspin()", 4),
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{source}");
        assert_eq!(error.message(), UNBOUNDED, "{source}");
        assert_eq!(error.position(), Some(Position::new(line, 1)), "{source}");
        assert_eq!(error.thrown_value(), None, "{source}");
        runtime_equivalence::assert_equivalent(&runtime, None, &control, None);
    }
    assert_eq!(
        evaluate(
            "total = 0\nrepeat:\n    k = 0\n    repeat: k += 1; total += 1\n    until k >= 60000\nuntil total >= 120000\ntotal"
        ),
        "120000.0"
    );
    assert_eq!(
        evaluate(
            "level = 12\ntry:\n    repeat: level = level + 3\n    until false\ncatch error:\n    failure = error\n[failure, level]"
        ),
        "[\"TypeError: Loop did not terminate\",12.0]"
    );
    assert_eq!(
        failure("x = 1\nwhile true: x = x + 1"),
        failure("x = 1\nrepeat: x = x + 1\nuntil false")
    );
}

#[test]
fn returns_leave_the_enclosing_function_block_loop_or_program() {
    assert_eq!(
        same_as(
            "def first(xs):\n    i = 0\n    repeat:\n        if xs[i] > 4: return xs[i]\n        i = i + 1\n    until i >= xs.length\n    return null\nfirst([1, 2, 5, 7])",
            "def first(xs):\n    i = 0\n    while true:\n        if xs[i] > 4: return xs[i]\n        i = i + 1\n        if i >= xs.length: return null\nfirst([1, 2, 5, 7])"
        ),
        "5.0"
    );
    assert_eq!(
        evaluate(
            "log = []\ndef check():\n    log.push(1)\n    return false\ndef f():\n    repeat:\n        return 5\n    until check()\n[f(), log]"
        ),
        "[5.0,[]]"
    );
    let mut runtime = Runtime::new();
    assert_eq!(
        json(
            &mut runtime,
            "n = 0\nrepeat:\n    n = n + 1\n    if n == 3: return n\nuntil false\nn = 99"
        ),
        "3.0"
    );
    assert_eq!(json(&mut runtime, "n"), "3.0");
    assert_eq!(
        evaluate("{\n    repeat: return 7\n    until false\n}"),
        "7.0"
    );
    assert_eq!(
        evaluate(
            "class Task\nTask()\nfor task of Task:\n    repeat: return \"out\"\n    until false"
        ),
        "\"out\""
    );
    assert_eq!(
        evaluate(
            "n = 0\nrepeat:\n    repeat:\n        n = n + 1\n        if n == 2: return n\n    until n > 5\nuntil false\nn"
        ),
        "2.0"
    );
    assert_eq!(
        evaluate("f = (n) => {\n    repeat: return n * 2\n    until false\n}\nf(4)"),
        "8.0"
    );
    assert_eq!(
        evaluate("def f():\n    repeat: x = 1\n    until true\nf()"),
        "null"
    );
}

#[test]
fn bodies_run_in_the_enclosing_scope_like_while_bodies() {
    for (looped, accepted, expected) in [
        (
            "def total(n):\n    sum = 0\n    repeat:\n        sum = sum + n\n        n = n - 1\n    until n == 0\n    return sum\ntotal(4)",
            "def total(n):\n    sum = 0\n    while n != 0:\n        sum = sum + n\n        n = n - 1\n    return sum\ntotal(4)",
            "10.0",
        ),
        (
            "class Item\nItem()\nfor item of Item:\n    total = 0\n    repeat: total = total + 1\n    until total == 3\ntotal",
            "class Item\nItem()\nfor item of Item:\n    total = 0\n    while total != 3: total = total + 1\ntotal",
            "3.0",
        ),
        (
            "class K(v):\n    this.v = v\nK(1)\nK(2)\nK(3)\nn = 0\nfor k of K:\n    repeat: n += k.v\n    until n % 2 == 0\nn",
            "class K(v):\n    this.v = v\nK(1)\nK(2)\nK(3)\nn = 0\nfor k of K:\n    n += k.v\n    while n % 2 != 0: n += k.v\nn",
            "10.0",
        ),
        (
            "def f(ks):\n    n = 0\n    for k of ks:\n        repeat: n += k.v\n        until n % 2 == 0\n    return n\nclass K(v):\n    this.v = v\nf([K(1), K(2), K(3), 4])",
            "def f(ks):\n    n = 0\n    for k of ks:\n        n += k.v\n        while n % 2 != 0: n += k.v\n    return n\nclass K(v):\n    this.v = v\nf([K(1), K(2), K(3), 4])",
            "0.0",
        ),
        (
            "n = 0\nrepeat:\n    n = n + 1\n    {\n        n = 5\n    }\nuntil true\nn",
            "n = 0\nwhile n == 0:\n    n = n + 1\n    {\n        n = 5\n    }\nn",
            "5.0",
        ),
        (
            "class Counter:\n    def tick(self, times):\n        repeat:\n            self.count = (self.count ?? 0) + 1\n            times -= 1\n        until times <= 0\n        return self.count\ncounter = Counter()\n[counter.tick(3), counter.tick(2)]",
            "class Counter:\n    def tick(self, times):\n        while times > 0:\n            self.count = (self.count ?? 0) + 1\n            times -= 1\n        return self.count\ncounter = Counter()\n[counter.tick(3), counter.tick(2)]",
            "[3.0,5.0]",
        ),
        (
            "class Ladder(height):\n    this.rungs = []\n    repeat: this.rungs.push(this.rungs.length)\n    until this.rungs.length >= height\nshort = Ladder(2)\ntall = Ladder(4)\n[short.rungs, tall.rungs]",
            "class Ladder(height):\n    this.rungs = []\n    while this.rungs.length < height: this.rungs.push(this.rungs.length)\nshort = Ladder(2)\ntall = Ladder(4)\n[short.rungs, tall.rungs]",
            "[[0.0,1.0],[0.0,1.0,2.0,3.0]]",
        ),
        (
            "class Box:\n    def grow(self, v):\n        repeat:\n            v = v * 2\n        until v > 10\n        return v\nBox().grow(1)",
            "class Box:\n    def grow(self, v):\n        v = v * 2\n        while v <= 10:\n            v = v * 2\n        return v\nBox().grow(1)",
            "16.0",
        ),
    ] {
        assert_eq!(same_as(looped, accepted), expected, "{looped}");
    }
    assert_eq!(
        failure("repeat: pass\nuntil this.ready"),
        (
            ErrorKind::Reference,
            "this is not defined".to_string(),
            Some(Position::new(1, 1))
        )
    );
    assert_eq!(
        failure("n = 0\nrepeat: n = missing + 1\nuntil n"),
        (
            ErrorKind::Reference,
            "missing is not defined".to_string(),
            Some(Position::new(2, 1))
        )
    );
}

#[test]
fn loops_leave_no_standing_rule_and_never_rerun() {
    let mut runtime = Runtime::new();
    run(&mut runtime, "limit = 3\ncount = 0\ntrace = []\nseen = 0");
    let before = keys(&runtime);
    run(
        &mut runtime,
        "repeat:\n    count = count + 1\n    if count > 1:\n        trace.push(count)\n    {\n        seen = count\n    }\nuntil count >= limit",
    );
    assert_eq!(keys(&runtime), before);
    checked(
        &mut runtime,
        "assert(count, 3)\nassert(trace, [2, 3])\nassert(seen, 3)\nlimit = 10\ncount = 0\nassert(count, 0)\nassert(trace, [2, 3])\nassert(seen, 3)",
    );
}

#[test]
fn loops_inside_standing_rules_run_whenever_those_rules_run() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Gauge
gauge = Gauge()
gauge.target = 2
gauge.level = 0
if gauge.target > 0:
    repeat: gauge.level = gauge.level + 1
    until gauge.level >= gauge.target
assert(gauge.level, 2)
gauge.target = 5
assert(gauge.level, 5)
def fill(n):
    level = 0
    repeat: level = level + 2
    until level >= n
    return level
target = 3
filled = fill(target)
assert(filled, 4)
target = 7
assert(filled, 8)
"#,
    );
}

#[test]
fn class_references_follow_while_loops() {
    for (looped, accepted, expected) in [
        (
            "class Item\nItem()\nItem()\nn = 0\nrepeat: n = n + 1\nuntil n >= $Item.length\nn",
            "class Item\nItem()\nItem()\nn = 1\nwhile n < $Item.length: n = n + 1\nn",
            "2.0",
        ),
        (
            "class Item\nfirst = Item()\nrepeat: $Item.flag = true\nuntil true\nsecond = Item()\n[first.flag, second.flag]",
            "class Item\nfirst = Item()\nwhile !first.flag: $Item.flag = true\nsecond = Item()\n[first.flag, second.flag]",
            "[true,true]",
        ),
        (
            "class Tank(level):\n    this.level = level\nif $Tank.level < 3:\n    repeat: $Tank.level = $Tank.level + 1\n    until $Tank.level >= 3\nlow = Tank(0)\nhigh = Tank(5)\n[low.level, high.level]",
            "class Tank(level):\n    this.level = level\nif $Tank.level < 3:\n    while $Tank.level < 3: $Tank.level = $Tank.level + 1\nlow = Tank(0)\nhigh = Tank(5)\n[low.level, high.level]",
            "[3.0,5.0]",
        ),
    ] {
        assert_eq!(same_as(looped, accepted), expected, "{looped}");
    }
    assert_eq!(
        failure("repeat: pass\nuntil $Missing.n"),
        (
            ErrorKind::Reference,
            "Missing is not defined".to_string(),
            Some(Position::new(1, 1))
        )
    );
    assert_eq!(
        failure("repeat: $Missing.n = 1\nuntil true"),
        failure("while true: $Missing.n = 1")
    );
}

#[test]
fn errors_and_thrown_values_propagate_unchanged() {
    for (source, kind, message, line) in [
        (
            "n = 0\nrepeat: n = 1\nuntil missing",
            ErrorKind::Reference,
            "missing is not defined",
            2,
        ),
        (
            "def spin():\n    repeat: x = 1\n    until missing\nspin()",
            ErrorKind::Reference,
            "missing is not defined",
            4,
        ),
        (
            "x = 1\nrepeat: pass\nuntil x()",
            ErrorKind::Type,
            "x is not a function",
            2,
        ),
    ] {
        assert_eq!(
            failure(source),
            (kind, message.to_string(), Some(Position::new(line, 1))),
            "{source}"
        );
    }
    for (source, thrown, line) in [
        ("repeat: throw \"STOP\"\nuntil true", "STOP", 1),
        (
            "def boom():\n    throw \"BOOM\"\nn = 0\nrepeat: n = n + 1\nuntil boom()",
            "BOOM",
            4,
        ),
    ] {
        let error = Runtime::new().run(source).unwrap_err();
        assert_eq!(
            error.thrown_value(),
            Some(&Value::String(thrown.to_string())),
            "{source}"
        );
        assert_eq!(error.position(), Some(Position::new(line, 1)), "{source}");
    }
}

#[test]
fn failed_iterations_restore_state_and_graph() {
    let initial = r#"
class Parcel
stock = 16
shipped = 0
left = stock - shipped
assert(left, 16)
"#;
    let mut actual = Runtime::new();
    checked(&mut actual, initial);
    let mut control = Runtime::new();
    checked(&mut control, initial);
    for (source, thrown) in [
        (
            "repeat:\n    if left < 3:\n        throw \"SHORT\"\n    Parcel()\n    shipped = shipped + 3\nuntil false",
            Some("SHORT"),
        ),
        ("repeat: shipped = 1\nuntil missing", None),
        ("repeat: shipped = shipped + 1\nuntil false", None),
        (
            "repeat:\n    shipped = shipped + 1\n    missing.value = 1\nuntil true",
            None,
        ),
        (
            "def boom():\n    throw \"BOOM\"\nrepeat:\n    Parcel()\n    shipped = shipped + 1\nuntil boom()",
            Some("BOOM"),
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
        "repeat:\n    Parcel()\n    shipped = shipped + 3\nuntil Parcel.length >= 2\nassert(Parcel.length, 2)\nassert(left, 10)",
    );
    assert_eq!(
        evaluate(
            "class Oven(temp):\n    this.temp = temp\nif $Oven.temp > 200:\n    throw \"OVERHEAT\"\noven = Oven(180)\ntry:\n    repeat: oven.temp += 10\n    until oven.temp > 250\ncatch error:\n    failure = error\n[failure, oven.temp]"
        ),
        "[\"OVERHEAT\",180.0]"
    );
}

#[test]
fn the_compatibility_rows_give_their_documented_results() {
    for (source, expected) in [
        ("x = 1\nrepeat:\n    x = x * 2\nuntil x > 100\nx", "128.0"),
        ("x = 500\nrepeat:\n    x = x + 1\nuntil x > 100\nx", "501.0"),
        (
            "x = 1\nn = 0\nrepeat: x = x * 2; n = n + 1\nuntil x > 100\n[x, n]",
            "[128.0,7.0]",
        ),
        (
            "def first(xs):\n    i = 0\n    repeat:\n        if xs[i] > 4: return xs[i]\n        i = i + 1\n    until i >= xs.length\n    return null\nfirst([1, 2, 5, 7])",
            "5.0",
        ),
        (
            "class Box:\n    def grow(self, v):\n        repeat:\n            v = v * 2\n        until v > 10\n        return v\nBox().grow(1)",
            "16.0",
        ),
        (
            "repeat = 3\nx = 0\nrepeat:\n    x = x + repeat\nuntil x > 10\nx",
            "12.0",
        ),
        ("x = 1\nrepeat:\n    x = x * 2\n\nuntil x > 100\nx", "128.0"),
        (
            "x = 1\nrepeat:\n    x = x * 2\n# note\nuntil x > 100\nx",
            "128.0",
        ),
        (
            "c = true\nx = 0\nrepeat:\n    x = x + 1\nuntil c ? x >= 2 : x >= 5\nx",
            "2.0",
        ),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn loops_run_inside_and_around_newer_statements() {
    for (source, expected) in [
        (
            "n = 0\ntry:\n    suppose:\n        repeat: n = n + 1\n        until false\ncatch error:\n    failure = error\n[failure, n]",
            "[\"TypeError: Loop did not terminate\",0.0]",
        ),
        (
            "n = 0\nseen = []\nrepeat:\n    n = n + 1\n    v = suppose:\n        n = n * 10\n        n\n    seen.push(v)\nuntil n >= 2\n[seen, n]",
            "[[10.0,20.0],2.0]",
        ),
        ("n = 0\nrepeat: n := n + 1\nuntil n >= 3\nn", "3.0"),
        (
            "o = {a: 1, b: 2}\ntotal = 0\nrepeat:\n    for k, v of o: total = total + v\nuntil total > 5\ntotal",
            "6.0",
        ),
        ("n = 2\nrepeat: n **= 2\nuntil n > 100\nn", "256.0"),
        ("x = 3\nn = 0\nrepeat: n = n + 1\nuntil x is 3\nn", "1.0"),
        (
            "xs = [1, 5]\nys = xs.filter(> 4)\nrepeat: xs.push(9)\nuntil ys.length >= 3\nys",
            "[5.0,9.0,9.0]",
        ),
        (
            "n = 2\nt = `${n}`\nrepeat: n = n + 1\nuntil t == \"4\"\n[n, t]",
            "[4.0,\"4\"]",
        ),
        (
            "xs = [x for x in [1, 2]]\nrepeat: xs.push(0)\nuntil xs.length > 3\nxs",
            "[1.0,2.0,0.0,0.0]",
        ),
        (
            "{a, b} = {a: 2, b: 1}\nrepeat: b = b + a\nuntil b > 6\nb",
            "7.0",
        ),
        (
            "n = 0\nrepeat:\n    i = 0\n    while i < 3:\n        n = n + 1\n        i = i + 1\nuntil n >= 6\nn",
            "6.0",
        ),
        (
            "n = 0\nrepeat:\n    repeat 2: n = n + 1\nuntil n >= 5\nn",
            "6.0",
        ),
        (
            "price = 2\ntotal = price * 3\nsteps = total |> why\nrepeat: price = price + 1\nuntil total >= 12\ntotal",
            "12.0",
        ),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn why_and_affects_report_what_the_loop_left_as_its_while_spelling_does() {
    let looped = evaluate(
        "base = 2\ntotal = 0\nrepeat:\n    total = total + base\nuntil total >= 6\nwhy total",
    );
    let accepted =
        evaluate("base = 2\ntotal = 0\nwhile total < 6:\n    total = total + base\nwhy total");
    assert_eq!(looped, accepted);
    assert_eq!(
        looped,
        "[{\"from\":[],\"holds\":6.0,\"node\":\"total\",\"rule\":\"total = 0\",\"state\":\"stated\"}]"
    );
    let looped = evaluate(
        "base = 2\nsum = base * 3\nn = 0\nrepeat: n = n + 1\nuntil n > sum\n(affects base).map(step -> step.node)",
    );
    assert_eq!(looped, "[\"sum\"]");
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "n = 0\n{\n    repeat: n = n + 1\n    until n > 3\n}",
    );
    assert!(
        keys(&runtime).contains("block(repeat{n=n+1}until(n>3))"),
        "{:?}",
        keys(&runtime)
    );
    assert_eq!(json(&mut runtime, "n"), "4.0");
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-repeat-until-loops-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_repeat_until_loops() {
    let file = SourceFile::new("file");
    let program = "x = 1\nn = 0\nrepeat:\n    x = x * 2\n\n    # once more\n    n = n + 1\nuntil x > 100\nrepeat: n = n + 1\nuntil true\n[x, n]";
    for newline in ["\n", "\r\n"] {
        let source = program.replace('\n', newline);
        let measured = file.run(&source, &["--json"]);
        assert!(measured.status.success(), "{measured:?}");
        assert!(measured.stderr.is_empty(), "{measured:?}");
        assert_eq!(String::from_utf8(measured.stdout).unwrap(), "[128.0,8.0]\n");
        let plain = file.run(&source, &[]);
        assert!(plain.status.success(), "{plain:?}");
        assert_eq!(String::from_utf8(plain.stdout).unwrap(), "128,8\n");
    }
    let graph = String::from_utf8(file.run(program, &["--json", "--graph"]).stdout).unwrap();
    assert!(graph.contains("  x [variable]"), "{graph}");
    assert!(!graph.contains("repeat"), "{graph}");
    let block = String::from_utf8(
        file.run(
            "n = 0\n{\n    repeat: n = n + 1\n    until n > 3\n}\nn",
            &["--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        block.contains(
            "  n [variable]\n    updates  block(repeat{n=n+1}until(n>3))\n  block(repeat{n=n+1}until(n>3)) [block]\n    reads    n"
        ),
        "{block}"
    );
    for (source, message) in [
        (
            "n = 0\nrepeat: n = n + 1\nuntil false",
            "TypeError: Loop did not terminate (line 2, column 1)\n",
        ),
        (
            "n = 0\nrepeat:\n    n = n + 1\nn",
            "SyntaxError: Expected 'until' after 'repeat' (line 4, column 1)\n",
        ),
        (
            "n = 0\nrepeat: n = n + 1\nuntil n > 3:",
            "SyntaxError: Unexpected ':' (line 3, column 12)\n",
        ),
    ] {
        let failed = file.run(source, &["--json"]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(String::from_utf8(failed.stderr).unwrap(), message);
    }
    let session = "x = 1\nrepeat:\n    x = x * 2\nuntil x > 100\n\nx\nrepeat: x = x + 1\nuntil x > 200\nx\nrepeat:\n    x = x + 1\n\nuntil x > 300\nx\n.exit\n";
    for (options, expected) in [
        (&["--plain"][..], "1\nnull\n128\n128\n128\n"),
        (
            &["--plain", "--json"][..],
            "1.0\nnull\n128.0\n128.0\n128.0\n",
        ),
        (&["--json"][..], "1.0\nnull\n128.0\n128.0\n128.0\n"),
    ] {
        for newline in ["\n", "\r\n"] {
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.ends_with(expected), "{options:?} {stdout}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                "SyntaxError: Expected 'until' after 'repeat' (line 2, column 1)\nReferenceError: until is not defined (line 1, column 1)\nSyntaxError: Expected 'until' after 'repeat' (line 3, column 1)\nReferenceError: until is not defined (line 1, column 1)\n",
                "{options:?}"
            );
        }
    }
}
