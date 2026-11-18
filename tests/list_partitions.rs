mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime, Value};
use serde::Deserialize;

const SPEC_TITLES: [&str; 6] = [
    "Nucleoid partitions lists into the items a predicate keeps and drops",
    "Nucleoid partitions class populations with partition",
    "Nucleoid fills an empty side of a partition as its items change",
    "Nucleoid maps and counts each side of a partition",
    "Nucleoid partitions items by the truthiness of the predicate result",
    "Nucleoid rolls back failed partition updates",
];
const REFERENCE_TITLE: &str = "Nucleoid partitions lists and populations with partition";
const README_TITLE: &str = "Partition keeps shipped and pending orders apart in one live rule";
const EXAMPLES_TITLE: &str =
    "Partition keeps every store's shelves to restock apart from the stocked ones";

const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid partitions lists into the items a predicate keeps and drops",
    "source": "# Nucleoid partitions lists into the items a predicate keeps and drops\n\nreadings = [12, 31, 7, 45, 18]\nlimit = 20\nsplit = [readings.filter(reading -> reading > limit), readings.filter(reading -> reading <= limit)]\nassert(split, [[31, 45], [12, 7, 18]])\nreadings.push(26)\nassert(split, [[31, 45, 26], [12, 7, 18]])\nlimit = 15\nassert(split, [[31, 45, 18, 26], [12, 7]])\nassert(readings, [12, 31, 7, 45, 18, 26])\nreadings = []\nassert(split, [[], []])\nsplit.length",
    "expected": "2"
  },
  {
    "document": "spec",
    "title": "Nucleoid partitions class populations with partition",
    "source": "# Nucleoid partitions class populations with partition\n\nclass Order(item, shipped):\n    this.item = item\n    this.shipped = shipped\nbook = [Order.filter(.shipped), Order.filter(order -> !order.shipped)]\nassert(book, [[], []])\npen = Order(\"pen\", true)\nink = Order(\"ink\", false)\ncap = Order(\"cap\", true)\nassert(book, [[pen, cap], [ink]])\nink.shipped = true\npen.shipped = false\nassert(book, [[ink, cap], [pen]])\ndelete pen.item\ndelete pen.shipped\ndelete pen\nassert(book, [[ink, cap], []])\nbook[0][0] == ink",
    "expected": "true"
  },
  {
    "document": "spec",
    "title": "Nucleoid fills an empty side of a partition as its items change",
    "source": "# Nucleoid fills an empty side of a partition as its items change\n\ntasks = [{name: \"plan\", done: true}, {name: \"draw\", done: true}]\nboard = [tasks.filter(.done), tasks.filter(task -> !task.done)]\nassert(board, [tasks, []])\ntasks.push({name: \"test\", done: false})\nassert(board[1][0] == tasks[2], true)\ntasks[0].done = false\nassert(board.map(side -> side.map(task -> task.name)), [[\"draw\"], [\"plan\", \"test\"]])\ntasks.shift()\nboard[1].map(task -> task.name)",
    "expected": "[\"test\"]"
  },
  {
    "document": "spec",
    "title": "Nucleoid maps and counts each side of a partition",
    "source": "# Nucleoid maps and counts each side of a partition\n\nclass Shape(name, area):\n    this.name = name\n    this.area = area\nparts = [Shape.filter(shape -> shape.area >= 10), Shape.filter(shape -> shape.area < 10)]\nnames = parts.map(side -> side.map(shape -> shape.name))\nsizes = parts.map(side -> side.length)\nShape(\"square\", 16)\nShape(\"dot\", 1)\nbar = Shape(\"bar\", 12)\nassert([names, sizes], [[[\"square\", \"bar\"], [\"dot\"]], [2, 1]])\nbar.area = 4\nassert(names, [[\"square\"], [\"dot\", \"bar\"]])\nShape(\"ring\", 30)\nsizes",
    "expected": "[2,2]"
  },
  {
    "document": "spec",
    "title": "Nucleoid partitions items by the truthiness of the predicate result",
    "source": "# Nucleoid partitions items by the truthiness of the predicate result\n\nanswers = [\"yes\", \"\", 0, 7, false, \"no\"]\ngiven = [answers.filter(answer -> answer), answers.filter(answer -> !answer)]\nassert(given, [[\"yes\", 7, \"no\"], [\"\", 0, false]])\nanswers.push([], \"\", true)\nanswers.shift()\nassert(given, [[7, \"no\", [], true], [\"\", 0, false, \"\"]])\ngiven[1].length",
    "expected": "4"
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back failed partition updates",
    "source": "# Nucleoid rolls back failed partition updates\n\nparcels = [{code: \"b2\", kg: 4}, {code: \"A7\", kg: 9}]\nweights = [parcels.filter(parcel -> parcel.code.toUpperCase() < \"B\"), parcels.filter(parcel -> parcel.code.toUpperCase() >= \"B\")].map(side -> side.map(parcel -> parcel.kg))\nfailure = \"\"\ntry:\n    parcels.push({code: 7, kg: 1})\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.toUpperCase is not a function\"))\nassert([parcels.length, weights], [2, [[9], [4]]])\nparcels.push({code: \"a1\", kg: 2})\nassert(weights, [[9, 2], [4]])\ntry:\n    parcels[0].code = \"a0\"\n    assert(weights, [[4, 9, 2], []])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nweights",
    "expected": "[[9,2],[4]]"
  },
  {
    "document": "reference",
    "title": "Nucleoid partitions lists and populations with partition",
    "source": "# Nucleoid partitions lists and populations with partition\n\nscores = [72, 45, 90, 58]\npassing = [scores.filter(score -> score >= 60), scores.filter(score -> score < 60)]\nclass Bid(who, amount):\n    this.who = who\n    this.amount = amount\nada = Bid(\"ada\", 40)\nBid(\"bo\", 65)\nbook = [Bid.filter(bid -> bid.amount > 50), Bid.filter(bid -> bid.amount <= 50)]\nassert(passing, [[72, 90], [45, 58]])\nassert(book.map(side -> side.map(bid -> bid.who)), [[\"bo\"], [\"ada\"]])\nscores.push(61, 12)\nBid(\"col\", 30)\nada.amount = 70\nassert(passing, [[72, 90, 61], [45, 58, 12]])\nassert(book.map(side -> side.map(bid -> bid.who)), [[\"ada\", \"bo\"], [\"col\"]])\nassert(scores[0], 72)\nbook[1][0].who",
    "expected": "\"col\""
  },
  {
    "document": "readme",
    "title": "Partition keeps shipped and pending orders apart in one live rule",
    "source": "class Order(total, shipped):\n    this.total = total\n    this.shipped = shipped\ntotals = [Order.filter(.shipped), Order.filter(order -> !order.shipped)].map(side -> side.map(order -> order.total))\nOrder(40, true)\nlate = Order(15, false)\nOrder(90, true)\nassert(totals, [[40, 90], [15]])\nlate.shipped = true\nOrder(25, false)\nassert(totals, [[40, 15, 90], [25]])\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Partition keeps every store's shelves to restock apart from the stocked ones",
    "source": "class Store(name, shelves):\n    this.name = name\n    this.shelves = shelves\n$Store.restock = [.shelves.filter(shelf -> shelf.have < shelf.need), .shelves.filter(shelf -> shelf.have >= shelf.need)].map(side -> side.map(shelf -> shelf.item))\nnorth = Store(\"north\", [{item: \"tea\", have: 2, need: 5}, {item: \"jam\", have: 9, need: 4}, {item: \"oat\", have: 1, need: 3}])\nsouth = Store(\"south\", [{item: \"rice\", have: 8, need: 2}])\nassert([north.restock, south.restock], [[[\"tea\", \"oat\"], [\"jam\"]], [[], [\"rice\"]]])\nnorth.shelves[0].have = 7\nsouth.shelves.push({item: \"salt\", have: 0, need: 1})\n[north.restock, south.restock]\n",
    "expected": "[[[\"oat\"],[\"tea\",\"jam\"]],[[\"salt\"],[\"rice\"]]]"
  }
]"###;

const WRITTEN_OUT: [(&str, &str); 9] = [
    (
        "readings.partition(reading -> reading > limit)",
        "[readings.filter(reading -> reading > limit), readings.filter(reading -> reading <= limit)]",
    ),
    (
        "Order.partition(order -> order.shipped)",
        "[Order.filter(.shipped), Order.filter(order -> !order.shipped)]",
    ),
    (
        "tasks.partition(task -> task.done)",
        "[tasks.filter(.done), tasks.filter(task -> !task.done)]",
    ),
    (
        "Shape.partition(shape -> shape.area >= 10)",
        "[Shape.filter(shape -> shape.area >= 10), Shape.filter(shape -> shape.area < 10)]",
    ),
    (
        "answers.partition(answer -> answer)",
        "[answers.filter(answer -> answer), answers.filter(answer -> !answer)]",
    ),
    (
        r#"parcels.partition(parcel -> parcel.code.toUpperCase() < "B")"#,
        r#"[parcels.filter(parcel -> parcel.code.toUpperCase() < "B"), parcels.filter(parcel -> parcel.code.toUpperCase() >= "B")]"#,
    ),
    (
        "scores.partition(score -> score >= 60)",
        "[scores.filter(score -> score >= 60), scores.filter(score -> score < 60)]",
    ),
    (
        "Bid.partition(bid -> bid.amount > 50)",
        "[Bid.filter(bid -> bid.amount > 50), Bid.filter(bid -> bid.amount <= 50)]",
    ),
    (
        "$Store.shelves.partition(shelf -> shelf.have < shelf.need)",
        "[.shelves.filter(shelf -> shelf.have < shelf.need), .shelves.filter(shelf -> shelf.have >= shelf.need)]",
    ),
];

const VALUES: [&str; 18] = [
    "0", "-0", "1", "2", "3", "-1.5", "0 / 0", "\"\"", "\"0\"", "\"a\"", "true", "false", "null",
    "[]", "[0]", "{}", "{k: 0}", "{k: 4}",
];
const BODIES: [&str; 10] = [
    "x",
    "!x",
    "x > 1",
    "x <= 0",
    "x == 0",
    "x != x",
    "typeof x == \"number\"",
    "x.k",
    "x.k > 2",
    "[1, 2].includes(x)",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedCase {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

struct Picks(u64);

impl Picks {
    fn pick(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from((self.0 >> 33) % u64::try_from(bound).unwrap()).unwrap()
    }
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

fn calls_partition(source: &str) -> bool {
    tokenize(source).unwrap().windows(3).any(|window| {
        window[0].token == Token::Dot
            && window[1].token == Token::Identifier("partition".to_string())
            && window[2].token == Token::LeftParen
    })
}

fn written_out(source: &str) -> (String, usize) {
    let mut written = source.to_string();
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
            title: "partition regression".to_string(),
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
    runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap()
        .dependencies
        .iter()
        .map(|key| renumbered(&key.to_string()))
        .collect()
}

fn spelled(body: &str) -> String {
    format!("[xs.filter(x -> {body}), xs.filter(x -> not ({body}))]")
}

fn behaves_as_spelled(steps: &[(&str, &str)], names: &[&str]) {
    let mut partitioned = Runtime::new();
    let mut filtered = Runtime::new();
    for (step, spelling) in steps {
        assert_eq!(
            renumbered(&outcome(&mut partitioned, step)),
            renumbered(&outcome(&mut filtered, spelling)),
            "{step}"
        );
        for name in names {
            if partitioned
                .graph
                .retrieve(&NodeKey::variable(*name))
                .is_some()
            {
                assert_eq!(
                    dependencies(&partitioned, name),
                    dependencies(&filtered, name),
                    "{step}: {name}"
                );
            }
        }
    }
    assert_eq!(partitioned.assertions_run(), filtered.assertions_run());
    assert!(partitioned.take_assertions().is_empty(), "{steps:?}");
    assert!(filtered.take_assertions().is_empty(), "{steps:?}");
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

#[test]
fn only_the_appended_cases_partition_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(calls_partition(&case.source), new, "{}", case.title);
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
    assert_eq!(assertions, 29);
    assert_eq!(cases, 1171);
}

#[test]
fn the_appended_cases_stand_after_nuc_107_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[749],
        "Nucleoid rolls back failed self parameter method calls and constructions"
    );
    assert_eq!(titles[750..756], SPEC_TITLES);
    assert_eq!(titles[784], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid accepts self parameters in methods and constructors",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Methods may declare self as Python methods do",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Python-style methods keep a tank farm's rules live",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(index, cases.len() - 11, "{title}");
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 8].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### List and population partitions\n\n```\n")
        .expect("the language reference documents partitions");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 108](nuc-0108.md)"));
    assert_eq!(case.expected.as_deref(), Some("\"col\""));
}

#[test]
fn every_case_is_its_accepted_counterpart_with_only_the_partition_written_out() {
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
        assert!(!calls_partition(&written), "{}", sealed.title);
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
    assert_eq!(rewrites, 10);
}

#[test]
fn partitions_equal_the_accepted_spelling_for_predicates_without_side_effects() {
    let mut picks = Picks(108);
    let mut compared = 0;
    for round in 0..400 {
        let length = [0, 1, 2, 3, 5, 8, 13, 21][picks.pick(8)];
        let items: Vec<&str> = (0..length)
            .map(|_| VALUES[picks.pick(VALUES.len())])
            .collect();
        let body = BODIES[round % BODIES.len()];
        let mut runtime = Runtime::new();
        runtime
            .run(&format!("xs = [{}]", items.join(", ")))
            .unwrap();
        assert_eq!(
            outcome(&mut runtime, &format!("xs.partition(x -> {body})")),
            outcome(&mut runtime, &spelled(body)),
            "{items:?} {body}"
        );
        compared += 1;
    }
    for round in 0..120 {
        let length = [0, 1, 2, 4, 7, 12][picks.pick(6)];
        let body = BODIES[round % BODIES.len()];
        let mut runtime = Runtime::new();
        runtime
            .run("class Item(n, k):\n    this.n = n\n    this.k = k")
            .unwrap();
        for position in 0..length {
            runtime
                .run(&format!(
                    "Item({position}, {})",
                    VALUES[picks.pick(VALUES.len())]
                ))
                .unwrap();
        }
        let predicate = body.replace('x', "i.k");
        assert_eq!(
            outcome(
                &mut runtime,
                &format!("Item.partition(i -> {predicate}).map(side -> side.map(i -> i.n))")
            ),
            outcome(
                &mut runtime,
                &format!(
                    "[Item.filter(i -> {predicate}), Item.filter(i -> not ({predicate}))].map(side -> side.map(i -> i.n))"
                )
            ),
            "{predicate}"
        );
        compared += 1;
    }
    assert_eq!(compared, 520);
}

#[test]
fn truthiness_sends_every_falsy_result_to_the_second_list() {
    for (values, expected) in [
        (
            "[false, 0, -0, 0 / 0, \"\", null]",
            "[[],[false,0.0,-0.0,null,\"\",null]]",
        ),
        (
            "[true, 2, -1, \"0\", \" \", [], {}, [0], new Date(0)]",
            "[9.0,0.0]",
        ),
    ] {
        let source = if expected.starts_with("[[") {
            format!("{values}.partition(x -> x)")
        } else {
            format!("{values}.partition(x -> x).map(side -> side.length)")
        };
        assert_eq!(result(&source), expected, "{values}");
    }
    assert_eq!(
        result(
            "f = x -> x\nclass C\nc = C()\n[f, /a/, c, C].partition(x -> x).map(side -> side.length)"
        ),
        "[4.0,0.0]"
    );
    assert_eq!(
        result("[{}.k, 5].partition(x -> x).map(side -> side.length)"),
        "[1.0,1.0]"
    );
    for (source, expected) in [
        ("[1, 0 / 0, 9].partition(x -> x > 4)", "[[9.0],[1.0,null]]"),
        ("[1, 0 / 0, 9].filter(x -> x <= 4)", "[1.0]"),
        (
            "[1, 0 / 0, 9].partition(x -> x > 4)[1].length - [1, 0 / 0, 9].filter(x -> x <= 4).length",
            "1.0",
        ),
        (
            "ys = [{k: 1}, {}, {k: 9}]\n[ys.partition(y -> y.k > 4).map(side -> side.length), ys.filter(y -> y.k <= 4).length]",
            "[[1.0,2.0],1.0]",
        ),
        (
            "zs = [undefined, 5]\n[zs.partition(z -> z > 4), zs.filter(z -> z <= 4)]",
            "[[[5.0],[null]],[null]]",
        ),
        (
            "ws = [\"abc\", {}, [1, 2], 5]\n[ws.partition(w -> w > 4)[1].length, ws.filter(w -> w <= 4).length]",
            "[3.0,0.0]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn the_predicate_runs_once_per_item_in_receiver_order_with_only_the_item() {
    for (source, expected) in [
        (
            "calls = []\nxs = [1, 2, 3]\nr = xs.partition(x -> calls.push(x) > 0 && x > 1)\n[r, calls]",
            "[[[2.0,3.0],[1.0]],[1.0,2.0,3.0]]",
        ),
        (
            "calls = []\nxs = [1, 2, 3]\nr = [xs.filter(x -> calls.push(x) > 0 && x > 1), xs.filter(x -> !(calls.push(x) > 0 && x > 1))]\n[r, calls]",
            "[[[2.0,3.0],[1.0]],[1.0,2.0,3.0,1.0,2.0,3.0]]",
        ),
        (
            "flip = {on: false}\ndef toggle(x):\n    flip.on = !flip.on\n    return flip.on\n[1, 2, 3].partition(x -> toggle(x))",
            "[[1.0,3.0],[2.0]]",
        ),
        (
            "flip = {on: false}\ndef toggle(x):\n    flip.on = !flip.on\n    return flip.on\nxs = [1, 2, 3]\n[xs.filter(x -> toggle(x)), xs.filter(x -> !toggle(x))]",
            "[[1.0,3.0],[1.0,3.0]]",
        ),
        ("[5, 6].partition((x, i) -> i)", "[[],[5.0,6.0]]"),
        (
            "seen = []\ndef weigh(item, second):\n    seen.push([item, second])\n    return item > 1\n[[1, 2].partition(weigh), seen]",
            "[[[2.0],[1.0]],[[1.0,null],[2.0,null]]]",
        ),
        (
            "seen = []\ndef mark(label, value):\n    seen.push(label)\n    return value\nxs = [3, 1]\n[mark(\"receiver\", xs).partition(mark(\"predicate\", x -> x > 2), mark(\"extra\", 0)), seen]",
            "[[[3.0],[1.0]],[\"receiver\",\"predicate\",\"extra\"]]",
        ),
        (
            "seen = []\ndef mark() => seen.push(1)\nxs = null\nr = xs.partition(mark())\n[r, seen]",
            "[null,[]]",
        ),
        (
            "seen = []\nclass Unit(n):\n    this.n = n\nUnit(1)\nUnit(2)\nUnit(3)\n[Unit.partition(unit -> seen.push(unit.n) > 0 && unit.n != 2).map(side -> side.map(unit -> unit.n)), seen]",
            "[[[1.0,3.0],[2.0]],[1.0,2.0,3.0]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    assert_eq!(
        failure("xs = [1, 2]\nxs.partition(missing)"),
        (ErrorKind::Reference, "missing is not defined".to_string())
    );
    assert_eq!(
        failure("xs = [1, 2]\nxs.partition(5, missing)"),
        (ErrorKind::Reference, "missing is not defined".to_string())
    );
}

#[test]
fn missing_null_and_non_function_predicates_raise_even_on_empty_receivers() {
    for source in [
        "xs = [1, 2]\nxs.partition()",
        "xs = [1, 2]\nxs.partition(null)",
        "xs = [1, 2]\nxs.partition(5)",
        "xs = [1, 2]\nxs.partition(\"k\")",
        "xs = [1, 2]\nxs.partition([x -> x])",
        "[].partition()",
        "[].partition(null)",
        "class Empty\nEmpty.partition(1)",
        "class Empty\nEmpty.partition()",
        "class Order(total):\n    this.total = total\nOrder(1)\nOrder.partition(\"total\")",
    ] {
        assert_eq!(
            failure(source),
            (ErrorKind::Type, "partition expects a function".to_string()),
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    runtime
        .run("xs = [2, 1]\nys = xs.partition(x -> x > 1)")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "ys = xs.partition(1)"),
        "TypeError: partition expects a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(json(&mut runtime, "xs.push(3)\nys"), "[[2.0,3.0],[1.0]]");
    assert_eq!(
        failure("xs = [1]\nxs.filter(5)"),
        (ErrorKind::Type, "filter expects a function".to_string())
    );
}

#[test]
fn lists_however_they_are_reached_are_receivers() {
    for (source, expected) in [
        ("[3, 1, 2].partition(x -> x > 1)", "[[3.0,2.0],[1.0]]"),
        (
            "xs = [3, 1, 2]\nxs.partition(x -> x > 1)",
            "[[3.0,2.0],[1.0]]",
        ),
        (
            "o = {inner: {items: [{n: \"b\"}, {n: \"a\"}]}}\no.inner.items.partition(i -> i.n == \"a\").map(side -> side.map(i -> i.n))",
            "[[\"a\"],[\"b\"]]",
        ),
        (
            "xs = [3, 1, 2]\nxs[1:].partition(x -> x > 1)",
            "[[2.0],[1.0]]",
        ),
        (
            "xs = [3, 4]\nxs.map(x -> x * 2).partition(x -> x > 7)",
            "[[8.0],[6.0]]",
        ),
        (
            "def make() => [2, 3, 1]\nmake().partition(x -> x > 1)",
            "[[2.0,3.0],[1.0]]",
        ),
        (
            "xs = [1, 2, 3]\nxs.value.partition(x -> x > 1)",
            "[[2.0,3.0],[1.0]]",
        ),
        (
            "xs = [1]\nxs.push(2)\nxs@first.partition(x -> x > 1)",
            "[[],[1.0]]",
        ),
        (
            "grid = [[3, 1], [2]]\ngrid[0].partition(x -> x > 1)",
            "[[3.0],[1.0]]",
        ),
        (
            "class Shelf(items):\n    this.items = items\n    def split():\n        return this.items.partition(item -> item.qty > 0)\ns = Shelf([{qty: 0}, {qty: 2}])\ns.split().map(side -> side.length)",
            "[1.0,1.0]",
        ),
        (
            "rows = [[1, 0], [2, 3]]\nrows.map(row -> row.partition(x -> x > 0))",
            "[[[1.0],[0.0]],[[2.0,3.0],[]]]",
        ),
        (
            "xs = [3, 8, 1, 9]\nxs.partition(x -> x > 4)[0].partition(x -> x > 8)",
            "[[9.0],[8.0]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn populations_by_name_through_a_variable_and_by_dollar_name_are_receivers() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)\nOrder(6)\nkind = Order")
        .unwrap();
    for (source, expected) in [
        (
            "Order.partition(o -> o.total > 5).map(side -> side.map(o -> o.total))",
            "[[7.0,6.0],[5.0]]",
        ),
        (
            "Order[1:].partition(o -> o.total > 6).map(side -> side.map(o -> o.total))",
            "[[7.0],[6.0]]",
        ),
        (
            "Order.list().partition(o -> o.total < 6).map(side -> side.map(o -> o.total))",
            "[[5.0],[7.0,6.0]]",
        ),
        (
            "kind.partition(o -> o.total > 5).map(side -> side.map(o -> o.total))",
            "[[7.0,6.0],[5.0]]",
        ),
        (
            "$Order.partition(o -> o.total > 5).map(side -> side.map(o -> o.total))",
            "[[7.0,6.0],[5.0]]",
        ),
        (
            "Order.filter(o -> o.total > 5).partition(o -> o.total > 6).map(side -> side.map(o -> o.total))",
            "[[7.0],[6.0]]",
        ),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    assert_eq!(
        result(
            "class Bag(n):\n    this.n = n\n    def partition(f): return 9\na = Bag(2)\nb = Bag(1)\n[Bag.partition(bag -> bag.n > 1) == [[a], [b]], a.partition(1)]"
        ),
        "[true,9.0]"
    );
    assert_eq!(result("class Empty\nEmpty.partition(e -> e)"), "[[],[]]");
    let subclasses = "class A:\n    def init(n):\n        this.n = n\nclass B extends A:\n    def init(n):\n        super(n)\nA(1)\nB(7)\nA(9)\n";
    assert_eq!(
        result(&format!(
            "{subclasses}[A.partition(a -> a.n > 2).map(side -> side.map(a -> a.n)), B.partition(b -> b.n > 2).map(side -> side.map(b -> b.n))]"
        )),
        "[[[9.0],[1.0]],[[7.0],[]]]"
    );
    assert_eq!(
        result(&format!(
            "{subclasses}[[A.filter(a -> a.n > 2), A.filter(a -> a.n <= 2)].map(side -> side.map(a -> a.n)), [B.filter(b -> b.n > 2), B.filter(b -> b.n <= 2)].map(side -> side.map(b -> b.n))]"
        )),
        "[[[9.0],[1.0]],[[7.0],[]]]"
    );
}

#[test]
fn selectors_named_functions_and_lambdas_are_predicates() {
    for (source, expected) in [
        (
            "xs = [{a: true, n: 1}, {a: false, n: 2}, {a: 1, n: 3}]\nxs.partition(.a).map(side -> side.map(.n))",
            "[[1.0,3.0],[2.0]]",
        ),
        (
            "xs = [{p: {q: 0}}, {p: {q: 2}}]\nxs.partition(.p.q).map(side -> side.length)",
            "[1.0,1.0]",
        ),
        (
            "big = x -> x > 2\n[1, 3, 5].partition(big)",
            "[[3.0,5.0],[1.0]]",
        ),
        (
            "def big(x) => x > 2\n[1, 3, 5].partition(big)",
            "[[3.0,5.0],[1.0]]",
        ),
        ("[1, 3, 5].partition(lambda x: x > 2)", "[[3.0,5.0],[1.0]]"),
        ("[1, 3, 5].partition(x => x > 2)", "[[3.0,5.0],[1.0]]"),
        ("[1, 3, 5].partition((x) -> x > 2)", "[[3.0,5.0],[1.0]]"),
        (
            "limit = 4\ndef split(values) => values.partition(v -> v > limit)\nsplit([1, 5, 9])",
            "[[5.0,9.0],[1.0]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn items_keep_their_identity_and_the_receiver_is_never_written() {
    for (source, expected) in [
        (
            "a = {k: 2}\nb = {k: 1}\nxs = [a, b]\nys = xs.partition(x -> x.k > 1)\n[ys[0][0] == a, ys[1][0] == b, xs[0] == a, xs.length]",
            "[true,true,true,2.0]",
        ),
        (
            "o = {items: [3, 1, 2]}\nsplit = o.items.partition(x -> x > 1)\n[o.items, split]",
            "[[3.0,1.0,2.0],[[3.0,2.0],[1.0]]]",
        ),
        (
            "class T(n):\n    this.n = n\nt = T(2)\nT(1)\ns = T.partition(item -> item.n > 1)\n[T.map(item -> item.n), s[0][0] == t, T.length]",
            "[[2.0,1.0],true,2.0]",
        ),
        (
            "scores = [30, 10, 20]\nhigh = scores.partition(s -> s > 15)\nlow = scores.partition(s -> s < 15)\nscores.push(40)\n[scores, high, low]",
            "[[30.0,10.0,20.0,40.0],[[30.0,20.0,40.0],[10.0]],[[10.0],[30.0,20.0,40.0]]]",
        ),
        ("[].partition(x -> x)", "[[],[]]"),
        ("xs = []\nys = xs.partition(x -> x)\nys", "[[],[]]"),
        ("xs = []\nxs.partition(x -> x).length", "2.0"),
        (
            "class Job\nqueue = Job.partition(job -> job.due)\nfirst := queue\nJob()\n[first, queue.map(side -> side.length)]",
            "[[[],[]],[0.0,1.0]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn declarations_follow_list_mutations_and_property_changes() {
    behaves_as_spelled(
        &[
            (
                "xs = [{n: \"a\", k: 3}, {n: \"b\", k: 1}]",
                "xs = [{n: \"a\", k: 3}, {n: \"b\", k: 1}]",
            ),
            ("limit = 2", "limit = 2"),
            (
                "ys = xs.partition(x -> x.k > limit)",
                "ys = [xs.filter(x -> x.k > limit), xs.filter(x -> not (x.k > limit))]",
            ),
            (
                "ys.map(side -> side.map(y -> y.n))",
                "ys.map(side -> side.map(y -> y.n))",
            ),
            ("xs.push({n: \"c\", k: 2})", "xs.push({n: \"c\", k: 2})"),
            (
                "ys.map(side -> side.map(y -> y.n))",
                "ys.map(side -> side.map(y -> y.n))",
            ),
            ("limit = 1", "limit = 1"),
            (
                "ys.map(side -> side.map(y -> y.n))",
                "ys.map(side -> side.map(y -> y.n))",
            ),
            ("xs.pop()", "xs.pop()"),
            ("xs.shift()", "xs.shift()"),
            (
                "ys.map(side -> side.map(y -> y.n))",
                "ys.map(side -> side.map(y -> y.n))",
            ),
            (
                "xs = [{n: \"d\", k: 0}, {n: \"e\", k: 5}]",
                "xs = [{n: \"d\", k: 0}, {n: \"e\", k: 5}]",
            ),
            ("xs[0].k = 9", "xs[0].k = 9"),
            (
                "ys.map(side -> side.map(y -> y.n))",
                "ys.map(side -> side.map(y -> y.n))",
            ),
        ],
        &["ys"],
    );
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "xs = [{n: \"a\", k: 3}, {n: \"b\", k: 1}]\nlimit = 2\nys = xs.partition(x -> x.k > limit).map(side -> side.map(x -> x.n))\nys",
            "[[\"a\"],[\"b\"]]",
        ),
        ("xs.push({n: \"c\", k: 2})\nys", "[[\"a\"],[\"b\",\"c\"]]"),
        ("limit = 1\nys", "[[\"a\",\"c\"],[\"b\"]]"),
        ("xs.pop()\nys", "[[\"a\"],[\"b\"]]"),
        ("xs.shift()\nys", "[[],[\"b\"]]"),
        (
            "xs = [{n: \"d\", k: 0}, {n: \"e\", k: 5}]\nys",
            "[[\"e\"],[\"d\"]]",
        ),
        ("xs[0].k = 9\nys", "[[\"d\",\"e\"],[]]"),
        ("xs[1].k = null\nys", "null"),
        ("xs[1].k = 0\nys", "[[\"d\"],[\"e\"]]"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn declarations_follow_new_and_deleted_instances_and_class_level_rules() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class R(n, t):\n    this.n = n\n    this.t = t\nboard = R.partition(r -> r.t > 2)\nboard",
            "[[],[]]",
        ),
        (
            "a = R(\"a\", 3)\nb = R(\"b\", 1)\nboard == [[a], [b]]",
            "true",
        ),
        ("c = R(\"c\", 5)\nboard == [[a, c], [b]]", "true"),
        ("b.t = 4\nboard == [[a, b, c], []]", "true"),
        ("delete c.t\nboard == [[a, b], [c]]", "true"),
        ("delete c.n\ndelete c\nboard == [[a, b], []]", "true"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class Team(name):\n    this.name = name\nclass Player(team, name, fit):\n    this.team = team\n    this.name = name\n    this.fit = fit\n$Team.lineup = Player.filter(p -> p.team == $Team.name).partition(p -> p.fit).map(side -> side.map(p -> p.name))\nreds = Team(\"reds\")\nblues = Team(\"blues\")\nPlayer(\"reds\", \"ana\", true)\nPlayer(\"blues\", \"bo\", false)\ncy = Player(\"reds\", \"cy\", false)\n[reds.lineup, blues.lineup]",
            "[[[\"ana\"],[\"cy\"]],[[],[\"bo\"]]]",
        ),
        (
            "Player(\"blues\", \"di\", true)\nblues.lineup",
            "[[\"di\"],[\"bo\"]]",
        ),
        ("cy.fit = true\nreds.lineup", "[[\"ana\",\"cy\"],[]]"),
        (
            "cy.team = \"blues\"\n[reds.lineup, blues.lineup]",
            "[[[\"ana\"],[]],[[\"cy\",\"di\"],[\"bo\"]]]",
        ),
        ("greens = Team(\"greens\")\ngreens.lineup", "[[],[]]"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "class P(x):\n    this.x = x\n$P.near = P.partition(other -> Math.abs(other.x - $P.x) < 2)[0].length\np = P(1)\nq = P(2)\nr = P(9)\n[p.near, q.near, r.near]",
            "[2.0,2.0,1.0]",
        ),
        (
            "class Room(name, tiles):\n    this.name = name\n    this.tiles = tiles\n$Room.split = $Room.tiles.partition(tile -> tile > 0)\nhall = Room(\"hall\", [3, 0, 5])\nden = Room(\"den\", [0])\nhall.tiles.push(2)\nden.tiles = [4, 0]\n[hall.split, den.split]",
            "[[[3.0,5.0,2.0],[0.0]],[[4.0],[0.0]]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn reads_inside_predicates_follow_the_rules_of_filter() {
    for (source, expected) in [
        (
            "xs = [1, null]\nr = xs.partition(x -> true)\nr",
            "[[1.0,null],[]]",
        ),
        ("xs = [1, null]\nr = xs.filter(x -> true)\nr", "[1.0,null]"),
        ("xs = [1, null]\nr = xs.partition(x -> x)\nr", "null"),
        ("xs = [1, null]\nr = xs.filter(x -> x)\nr", "null"),
        (
            "xs = [{k: null}, {k: 1}]\nr = xs.partition(x -> x.k)\nr",
            "null",
        ),
        (
            "limit = null\nxs = [1, 5]\nr = xs.partition(x -> x > limit)\nlimit = 2\nr",
            "[[5.0],[1.0]]",
        ),
        ("xs = [1, null]\nxs.partition(x -> x)", "[[1.0],[null]]"),
        (
            "xs = [{k: 1}, {}]\nr = xs.partition(x -> x.k > 0)\nr.map(side -> side.length)",
            "[1.0,1.0]",
        ),
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
            "rows = [2, 1]\nlimit = 1\nsplit = rows.partition(row -> row > limit)\n(why split).map(step -> step.rule)"
        ),
        "[\"split = rows.partition(function())\",\"rows = [2,1]\",\"limit = 1\"]"
    );
    assert_eq!(
        json(&mut runtime, "(affects limit).map(step -> step.node)"),
        "[\"split\"]"
    );
    assert_eq!(dependencies(&runtime, "split"), ["rows", "limit"]);
    let mut runtime = Runtime::new();
    runtime
        .run("class Bid(amount):\n    this.amount = amount\nbook = Bid.partition(bid -> bid.amount > 3)\nBid(5)")
        .unwrap();
    assert_eq!(dependencies(&runtime, "book"), ["$Bid", "obj:0.amount"]);
    let mut spelled = Runtime::new();
    spelled
        .run("rows = [2, 1]\nlimit = 1\nsplit = [rows.filter(row -> row > limit), rows.filter(row -> row <= limit)]")
        .unwrap();
    assert_eq!(
        json(&mut spelled, "(why split).map(step -> step.rule)"),
        "[\"split = [rows.filter(function()),rows.filter(function())]\",\"rows = [2,1]\",\"limit = 1\"]"
    );
    assert_eq!(dependencies(&spelled, "split"), ["rows", "limit"]);
    assert_eq!(
        result("def key(x) => x.k\nr = [].partition(key)\n(affects key).map(step -> step.node)"),
        "[\"r\"]"
    );
}

#[test]
fn snapshots_hypotheticals_destructuring_templates_and_conditions_read_partitions() {
    for (source, expected) in [
        (
            "xs = [3, 8]\nparts = xs.partition(x -> x > 4)\nfrozen = parts.value\nxs.push(9, 1)\n[parts, frozen]",
            "[[[8.0,9.0],[3.0,1.0]],[[8.0],[3.0]]]",
        ),
        (
            "xs = [3, 8]\nparts = xs.partition(x -> x > 4)\nseen = suppose xs.push(9, 1): parts.map(side -> side.length)\n[seen, parts]",
            "[[2.0,2.0],[[8.0],[3.0]]]",
        ),
        (
            "xs = [3, 8, 1, 9]\nkept, dropped = xs.partition(x -> x > 4)\nxs.push(5, 2)\n[kept, dropped]",
            "[[8.0,9.0,5.0],[3.0,1.0,2.0]]",
        ),
        (
            "xs = [3, 8, 1]\nlabel = `${xs.partition(x -> x > 4)[1].length} small`\nxs.push(2)\nlabel",
            "\"3 small\"",
        ),
        (
            "xs = [3, 8, 1]\nif xs.partition(x -> x > 4)[0].length > 0:\n    flag = true\nflag",
            "true",
        ),
        (
            "xs = [3, 8, 1]\nsizes = xs.partition(x -> x > 4).map(side -> side.length)\nxs.push(6)\nsizes",
            "[2.0,2.0]",
        ),
        (
            "xs = [3, 8, 1]\nmore = xs.partition(x -> x > 4)[0].length > 1 ? \"many\" : \"few\"\nxs.push(6)\nmore",
            "\"many\"",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn failed_statements_and_transactions_restore_partitioned_declarations() {
    behaves_as_spelled(
        &[
            ("rows = [{k: 2}, {k: 1}]", "rows = [{k: 2}, {k: 1}]"),
            (
                "split = rows.partition(row -> row.k > 9 ? row.k.toUpperCase() : row.k > 1)",
                "split = [rows.filter(row -> row.k > 9 ? row.k.toUpperCase() : row.k > 1), rows.filter(row -> not (row.k > 9 ? row.k.toUpperCase() : row.k > 1))]",
            ),
            (
                "try:\n    rows.push({k: 5})\n    assert(split.map(side -> side.length), [2, 1])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
                "try:\n    rows.push({k: 5})\n    assert(split.map(side -> side.length), [2, 1])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
            ),
            (
                "[rows.length, split.map(side -> side.map(row -> row.k))]",
                "[rows.length, split.map(side -> side.map(row -> row.k))]",
            ),
            ("rows.push({k: 10})", "rows.push({k: 10})"),
            (
                "[rows.length, split.map(side -> side.map(row -> row.k))]",
                "[rows.length, split.map(side -> side.map(row -> row.k))]",
            ),
            ("rows.push({k: 3})", "rows.push({k: 3})"),
            (
                "split.map(side -> side.map(row -> row.k))",
                "split.map(side -> side.map(row -> row.k))",
            ),
        ],
        &["split"],
    );
    let mut runtime = Runtime::new();
    runtime
        .run("rows = [{k: 2}, {k: 1}]\nsplit = rows.partition(row -> row.k > 9 ? row.k.toUpperCase() : row.k > 1)")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "rows.push({k: 10})"),
        "TypeError: number.toUpperCase is not a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(
        json(
            &mut runtime,
            "[rows.length, split.map(side -> side.map(row -> row.k))]"
        ),
        "[2.0,[[2.0],[1.0]]]"
    );
    assert_eq!(
        result(
            "class Probe(t):\n    this.t = t\nProbe(1)\nProbe(2)\nwarm = Probe.partition(p -> p.t > 9 ? p.t.toUpperCase() : p.t > 1)\nfailure = \"\"\ntry:\n    Probe(50)\ncatch error:\n    failure = error\n[failure, Probe.length, warm.map(side -> side.map(p -> p.t))]"
        ),
        "[\"TypeError: number.toUpperCase is not a function\",2.0,[[2.0],[1.0]]]"
    );
    assert_eq!(
        result(
            "xs = [1, 2]\nsplit = xs.partition(x -> x > 1)\ntry:\n    xs.push(3)\n    throw \"UNDO\"\ncatch error:\n    pass\nsplit"
        ),
        "[[2.0],[1.0]]"
    );
}

#[test]
fn large_receivers_partition_through_the_public_api() {
    let mut runtime = Runtime::new();
    runtime.run("holder = {items: 0}").unwrap();
    let count = 100_000_u32;
    hold(
        &mut runtime,
        Value::List(
            (0..count)
                .map(|position| Value::Number(f64::from((position * 7_919) % count)))
                .collect(),
        ),
    );
    assert_eq!(
        json(
            &mut runtime,
            "parts = holder.items.partition(x -> x % 3 == 0)\n[parts[0].length, parts[1].length, parts[0][1], parts[1][0], parts[0].every(x -> x % 3 == 0)]"
        ),
        "[33334.0,66666.0,23757.0,7919.0,true]"
    );
    assert_eq!(
        json(&mut runtime, "holder.items[:3]"),
        "[0.0,7919.0,15838.0]"
    );
}

#[test]
fn unchanged_readings_keep_their_accepted_results() {
    for (source, expected) in [
        ("xs = [1]\nxs.partition", "null"),
        ("class Order\nOrder.partition", "null"),
        ("o = {partition: 3}\no.partition", "3.0"),
        ("x = {partition: 4}.partition\nx", "4.0"),
        ("o = {partition: n -> n * 10}\no.partition(4)", "40.0"),
        ("r = {partition: f -> 5}.partition(x -> x)\nr", "5.0"),
        ("class P\np = P()\np.partition(x -> x)", "null"),
        ("r = {a: 1}\nr.partition(x -> x)", "null"),
        (
            "class Bag:\n    def partition(f): return 9\nbag = Bag()\nbag.partition(1)",
            "9.0",
        ),
        (
            "partition = 3\ndef scale partition => partition * 2\n[partition, scale(partition)]",
            "[3.0,6.0]",
        ),
        ("def partition(xs) => xs.length\npartition([1, 2])", "2.0"),
        ("xs = null\nxs.partition(i -> i)", "null"),
        ("o = {}\no.items.partition(x -> x)", "null"),
        ("[1, 5].filter(x -> x > 2)", "[5.0]"),
        ("[1, 5].count(x -> x > 2)", "1.0"),
        (
            "[[1, 5].filter(x -> x > 2), [1, 5].filter(x -> x <= 2)]",
            "[[5.0],[1.0]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, kind, message) in [
        (
            "\"abc\".partition(x -> x)",
            ErrorKind::Type,
            "abc.partition is not a function",
        ),
        (
            "(5).partition(x -> x)",
            ErrorKind::Type,
            "number.partition is not a function",
        ),
        (
            "true.partition(x -> x)",
            ErrorKind::Type,
            "boolean.partition is not a function",
        ),
        (
            "d = new Date(5)\nd.partition(x -> x)",
            ErrorKind::Type,
            "Date.partition is not a function",
        ),
        (
            "f = x => x\nf.partition(x -> x)",
            ErrorKind::Type,
            "function.partition is not a function",
        ),
        (
            "/a/.partition(x -> x)",
            ErrorKind::Type,
            "partition is not a function",
        ),
        (
            "Item = 5\nItem.partition(i -> i)",
            ErrorKind::Type,
            "number.partition is not a function",
        ),
        (
            "Math.partition(x -> x)",
            ErrorKind::Type,
            "Math.partition is not a function",
        ),
        (
            "Number.partition(x -> x)",
            ErrorKind::Type,
            "Number.partition is not a function",
        ),
        (
            "String.partition(x -> x)",
            ErrorKind::Type,
            "String.partition is not a function",
        ),
        (
            "Date.partition(x -> x)",
            ErrorKind::Type,
            "Date.partition is not a function",
        ),
        (
            "List.partition(x -> x)",
            ErrorKind::Reference,
            "List is not defined",
        ),
        (
            "Object.partition(x -> x)",
            ErrorKind::Reference,
            "Object is not defined",
        ),
        (
            "Boolean.partition(x -> x)",
            ErrorKind::Reference,
            "Boolean is not defined",
        ),
        (
            "class Order\nClass.partition(x -> x)",
            ErrorKind::Reference,
            "Class is not defined",
        ),
        (
            "partition([1, 2])",
            ErrorKind::Reference,
            "partition is not defined",
        ),
        (
            "partition of [1, 2]",
            ErrorKind::Syntax,
            "Unexpected keyword 'of'",
        ),
        (
            "xs = [1]\nxs.partitions(x -> x)",
            ErrorKind::Type,
            "xs.partitions is not a function",
        ),
    ] {
        assert_eq!(failure(source), (kind, message.to_string()), "{source}");
    }
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    assert_eq!(
        result(
            "xs = [2, 1]\nr = 0\ntry:\n    r = xs.partition(x -> x > 1)\ncatch error:\n    r = error\nr"
        ),
        "[[2.0],[1.0]]"
    );
    assert_eq!(
        result(
            "xs = [2, 1]\nr = 0\ntry:\n    r = xs.partitions(x -> x > 1)\ncatch error:\n    r = error\nr"
        ),
        "\"TypeError: xs.partitions is not a function\""
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-list-partitions-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_partition() {
    let program = "rows = [2, 3, 1]\nsplit = rows.partition(row -> row > 1)\nrows.push(0)\nsplit";
    let file = SourceFile::new("file");
    let partitioned = file.run(program, &["--json"]);
    assert!(partitioned.status.success(), "{partitioned:?}");
    assert!(partitioned.stderr.is_empty(), "{partitioned:?}");
    assert_eq!(
        String::from_utf8(partitioned.stdout).unwrap(),
        "[[2.0,3.0],[1.0,0.0]]\n"
    );
    let plain = file.run(program, &[]);
    assert!(plain.status.success(), "{plain:?}");
    assert_eq!(String::from_utf8(plain.stdout).unwrap(), "2,3,1,0\n");
    let graph = String::from_utf8(file.run(program, &["--json", "--graph"]).stdout).unwrap();
    assert!(
        graph.contains(
            "  rows [variable]\n    updates  split\n  split [variable]\n    reads    rows"
        ),
        "{graph}"
    );
    let failed = file.run("xs = [1]\nxs.partition()", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "TypeError: partition expects a function (line 2, column 1)\n"
    );
    let session = "xs = [3, 1, 2]\nys = xs.partition(x -> x > 1)\nys\nxs.push(0)\nys\nxs\nxs.partition()\n.clear\nxs = [\"b\", \"a\"]\nxs.partition(x -> x == \"a\")\n.exit\n";
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let options: &[&str] = if plain { &["--plain"] } else { &[] };
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(
                stdout
                    .contains("[[3.0,2.0],[1.0]]\n4.0\n[[3.0,2.0],[1.0,0.0]]\n[3.0,1.0,2.0,0.0]\n"),
                "{stdout}"
            );
            assert!(stdout.contains("[[\"a\"],[\"b\"]]\n"), "{stdout}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                "TypeError: partition expects a function (line 1, column 1)\n"
            );
        }
    }
}
