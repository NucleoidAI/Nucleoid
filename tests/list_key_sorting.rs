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
    "Nucleoid sorts lists by numeric and text keys with sortBy",
    "Nucleoid keeps key sorted lists live while the receiver keeps its order",
    "Nucleoid sorts class populations by key with sortBy",
    "Nucleoid ranks instances by key in class level rules",
    "Nucleoid waits for every sortBy key to be comparable",
    "Nucleoid rolls back failed key sorting updates",
];
const REFERENCE_TITLE: &str = "Nucleoid sorts lists and populations by key with sortBy";
const README_TITLE: &str = "Sorting by key keeps a live agenda while the list keeps its order";
const EXAMPLES_TITLE: &str = "Sorting by key keeps every courier's route in order of distance";

const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid sorts lists by numeric and text keys with sortBy",
    "source": "# Nucleoid sorts lists by numeric and text keys with sortBy\n\nparcels = [{code: \"B12\", kg: 4}, {code: \"A7\", kg: 9}, {code: \"B3\", kg: 4}, {code: \"C1\", kg: 2}]\nlight = parcels[:].sort(lambda a, b: a.kg - b.kg)\ncoded = parcels[:].sort(lambda a, b: -(a.code < b.code))\nassert(light.map(parcel -> parcel.code), [\"C1\", \"B12\", \"B3\", \"A7\"])\nassert(coded.map(parcel -> parcel.code), [\"A7\", \"B12\", \"B3\", \"C1\"])\nassert(light[1] == parcels[0], true)\nparcels.push({code: \"A10\", kg: 4})\nassert(light.map(parcel -> parcel.code), [\"C1\", \"B12\", \"B3\", \"A10\", \"A7\"])\nassert(coded.map(parcel -> parcel.code), [\"A10\", \"A7\", \"B12\", \"B3\", \"C1\"])\nparcels.map(parcel -> parcel.code)",
    "expected": "[\"B12\",\"A7\",\"B3\",\"C1\",\"A10\"]"
  },
  {
    "document": "spec",
    "title": "Nucleoid keeps key sorted lists live while the receiver keeps its order",
    "source": "# Nucleoid keeps key sorted lists live while the receiver keeps its order\n\nstops = [{id: 1, eta: 30}, {id: 2, eta: 10}, {id: 3, eta: 20}]\nroute = stops[:].sort(lambda a, b: a.eta - b.eta).map(stop -> stop.id)\nassert(route, [2, 3, 1])\nstops.push({id: 4, eta: 15})\nassert(route, [2, 4, 3, 1])\nstops.shift()\nassert(route, [2, 4, 3])\nstops[0].eta = 25\nassert(route, [4, 3, 2])\nstops.pop()\nassert(route, [3, 2])\nstops.map(stop -> stop.id)",
    "expected": "[2,3]"
  },
  {
    "document": "spec",
    "title": "Nucleoid sorts class populations by key with sortBy",
    "source": "# Nucleoid sorts class populations by key with sortBy\n\nclass Runner(name, time):\n    this.name = name\n    this.time = time\nboard = Runner.sort(lambda a, b: a.time - b.time)\nassert(board, [])\nana = Runner(\"ana\", 31)\nbo = Runner(\"bo\", 28)\ncy = Runner(\"cy\", 35)\nassert(board, [bo, ana, cy])\ncy.time = 27\nRunner(\"di\", 28)\nassert(board.map(runner -> runner.name), [\"cy\", \"bo\", \"di\", \"ana\"])\ndelete bo.name\ndelete bo.time\ndelete bo\nassert(board.map(runner -> runner.name), [\"cy\", \"di\", \"ana\"])\nboard.length",
    "expected": "3"
  },
  {
    "document": "spec",
    "title": "Nucleoid ranks instances by key in class level rules",
    "source": "# Nucleoid ranks instances by key in class level rules\n\nclass Runner(name, time):\n    this.name = name\n    this.time = time\n$Runner.place = Runner.sort(lambda a, b: a.time - b.time).indexOf($Runner) + 1\nana = Runner(\"ana\", 31)\nbo = Runner(\"bo\", 28)\ncy = Runner(\"cy\", 31)\nassert([ana.place, bo.place, cy.place], [2, 1, 3])\nbo.time = 33\nassert([ana.place, bo.place, cy.place], [1, 3, 2])\nRunner(\"di\", 25)\n[ana.place, bo.place, cy.place]",
    "expected": "[2,4,3]"
  },
  {
    "document": "spec",
    "title": "Nucleoid waits for every sortBy key to be comparable",
    "source": "# Nucleoid waits for every sortBy key to be comparable\n\nclass Job(name):\n    this.name = name\nqueue = Job.maxBy(.due) && Job.sort(lambda a, b: a.due - b.due)\npress = Job(\"press\")\npress.due = new Date(\"2026-12-01\")\ncoat = Job(\"coat\")\nassert(queue, null)\ncoat.due = new Date(\"2026-11-20\")\nassert(queue, [coat, press])\nweld = Job(\"weld\")\nweld.due = new Date(\"2026-11-28\")\nassert(queue, [coat, weld, press])\nweld.due = 0 / 0\nassert(queue, null)\nweld.due = coat.due\nqueue.map(job -> job.name)",
    "expected": "[\"coat\",\"weld\",\"press\"]"
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back failed key sorting updates",
    "source": "# Nucleoid rolls back failed key sorting updates\n\nparcels = [{id: 1, code: \"B2\"}, {id: 2, code: \"a9\"}]\nlabels = parcels[:].sort(lambda a, b: -(a.code.toUpperCase() < b.code.toUpperCase())).map(parcel -> parcel.id)\nfailure = \"\"\ntry:\n    parcels.push({id: 3, code: 7})\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.toUpperCase is not a function\"))\nassert([parcels.length, labels], [2, [2, 1]])\nparcels.push({id: 4, code: \"a1\"})\nassert(labels, [4, 2, 1])\ntry:\n    parcels[0].code = \"a0\"\n    assert(labels, [1, 4, 2])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nlabels",
    "expected": "[4,2,1]"
  },
  {
    "document": "reference",
    "title": "Nucleoid sorts lists and populations by key with sortBy",
    "source": "# Nucleoid sorts lists and populations by key with sortBy\n\nscores = [{name: \"ben\", points: 3}, {name: \"ana\", points: 9}, {name: \"dan\", points: 5}, {name: \"cy\", points: 9}]\nranked = scores[:].sort(lambda a, b: b.points - a.points)\nclass Bid(who, amount):\n    this.who = who\n    this.amount = amount\nBid(\"ada\", 40)\nBid(\"bo\", 65)\nbook = Bid.sort(lambda a, b: a.amount - b.amount)\nassert(ranked.map(score -> score.name), [\"ana\", \"cy\", \"dan\", \"ben\"])\nassert(book.map(bid -> bid.who), [\"ada\", \"bo\"])\nscores.push({name: \"eve\", points: 7})\nBid(\"col\", 50)\nassert(ranked.map(score -> score.name), [\"ana\", \"cy\", \"eve\", \"dan\", \"ben\"])\nassert(book.map(bid -> bid.who), [\"ada\", \"col\", \"bo\"])\nassert(scores[0].name, \"ben\")\nbook[1].who",
    "expected": "\"col\""
  },
  {
    "document": "readme",
    "title": "Sorting by key keeps a live agenda while the list keeps its order",
    "source": "tasks = [{name: \"ship\", due: 3}, {name: \"plan\", due: 1}, {name: \"build\", due: 2}]\nagenda = tasks[:].sort(lambda a, b: a.due - b.due)\nassert(agenda.map(task -> task.name), [\"plan\", \"build\", \"ship\"])\ntasks.push({name: \"test\", due: 2})\nassert(agenda.map(task -> task.name), [\"plan\", \"build\", \"test\", \"ship\"])\ntasks[2].due = 4\nassert(agenda.map(task -> task.name), [\"plan\", \"test\", \"ship\", \"build\"])\nassert(tasks.map(task -> task.name), [\"ship\", \"plan\", \"build\", \"test\"])\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Sorting by key keeps every courier's route in order of distance",
    "source": "class Courier(name, stops):\n    this.name = name\n    this.stops = stops\n$Courier.route = $Courier.stops[:].sort(lambda a, b: a.km - b.km).map(stop -> stop.name)\nana = Courier(\"ana\", [{name: \"mill\", km: 12}, {name: \"dock\", km: 3}, {name: \"farm\", km: 7}])\nbo = Courier(\"bo\", [{name: \"pier\", km: 5}])\nassert([ana.route, bo.route], [[\"dock\", \"farm\", \"mill\"], [\"pier\"]])\nana.stops.push({name: \"yard\", km: 1})\nana.stops[0].km = 2\nbo.stops = [{name: \"gate\", km: 9}, {name: \"lane\", km: 2}]\nassert(ana.stops[0].name, \"mill\")\n[ana.route, bo.route]\n",
    "expected": "[[\"yard\",\"mill\",\"dock\",\"farm\"],[\"lane\",\"gate\"]]"
  }
]"###;

const WRITTEN_OUT: [(&str, &str); 10] = [
    (
        "parcels.sortBy(parcel -> parcel.kg)",
        "parcels[:].sort(lambda a, b: a.kg - b.kg)",
    ),
    (
        "parcels.sortBy(parcel -> parcel.code)",
        "parcels[:].sort(lambda a, b: -(a.code < b.code))",
    ),
    (
        "stops.sortBy(stop -> stop.eta)",
        "stops[:].sort(lambda a, b: a.eta - b.eta)",
    ),
    (
        "Runner.sortBy(runner -> runner.time)",
        "Runner.sort(lambda a, b: a.time - b.time)",
    ),
    (
        "Job.sortBy(job -> job.due)",
        "Job.maxBy(.due) && Job.sort(lambda a, b: a.due - b.due)",
    ),
    (
        "parcels.sortBy(parcel -> parcel.code.toUpperCase())",
        "parcels[:].sort(lambda a, b: -(a.code.toUpperCase() < b.code.toUpperCase()))",
    ),
    (
        "scores.sortBy(score -> -score.points)",
        "scores[:].sort(lambda a, b: b.points - a.points)",
    ),
    (
        "Bid.sortBy(bid -> bid.amount)",
        "Bid.sort(lambda a, b: a.amount - b.amount)",
    ),
    (
        "tasks.sortBy(task -> task.due)",
        "tasks[:].sort(lambda a, b: a.due - b.due)",
    ),
    (
        "$Courier.stops.sortBy(stop -> stop.km)",
        "$Courier.stops[:].sort(lambda a, b: a.km - b.km)",
    ),
];

const DECORATE: &str = ".sort((a, b) -> a[0] < b[0] ? -1 : 0).map(p -> p[1])";

const ORDERED_NUMBERS: [&str; 15] = [
    "0",
    "-0",
    "1",
    "-1.5",
    "2",
    "10",
    "9",
    "3",
    "true",
    "false",
    "null",
    "[]",
    "[2]",
    "new Date(5)",
    "new Date(9)",
];
const ORDERED_TEXTS: [&str; 9] = [
    "\"10\"", "\"9\"", "\"2\"", "\"a\"", "\"B\"", "\"b\"", "\"\"", "\"a b\"", "\"A\"",
];
const UNORDERED: [&str; 4] = ["0 / 0", "{}", "[1, 2]", "x -> x"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedCase {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

struct Keys(u64);

impl Keys {
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

fn calls_sort_by(source: &str) -> bool {
    tokenize(source).unwrap().windows(3).any(|window| {
        window[0].token == Token::Dot
            && window[1].token == Token::Identifier("sortBy".to_string())
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
            title: "sortBy regression".to_string(),
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

fn rendered(text: &str) -> String {
    text.replace(
        ".sortBy(function())",
        ".map(function()).sort(function()).map(function())",
    )
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

fn behaves_as_decorated(steps: &[(&str, &str)]) {
    let mut sorted = Runtime::new();
    let mut decorated = Runtime::new();
    for (step, spelled) in steps {
        assert_eq!(
            renumbered(&rendered(&outcome(&mut sorted, step))),
            renumbered(&outcome(&mut decorated, spelled)),
            "{step}"
        );
        assert_eq!(
            rendered(&graph_shape(&sorted)),
            graph_shape(&decorated),
            "{step}"
        );
    }
    assert_eq!(sorted.assertions_run(), decorated.assertions_run());
    assert!(sorted.take_assertions().is_empty(), "{steps:?}");
    assert!(decorated.take_assertions().is_empty(), "{steps:?}");
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

fn records(keys: &[Option<&str>]) -> String {
    let items: Vec<String> = keys
        .iter()
        .enumerate()
        .map(|(position, key)| match key {
            Some(key) => format!("{{i: {position}, k: {key}}}"),
            None => format!("{{i: {position}}}"),
        })
        .collect();
    format!("xs = [{}]", items.join(", "))
}

#[test]
fn only_the_appended_cases_sort_by_key_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(calls_sort_by(&case.source), new, "{}", case.title);
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
    assert_eq!(assertions, 36);
    assert_eq!(cases, 1105);
}

#[test]
fn the_appended_cases_stand_after_nuc_104_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[732],
        "Nucleoid keeps all and any as names and quantifiers beside class references"
    );
    assert_eq!(titles[733..739], SPEC_TITLES);
    assert_eq!(titles[745], "Nucleoid flattens nested lists by one level");
    for (cases, title, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        assert_eq!(cases[cases.len() - 5].title, title);
        assert_eq!(cases[cases.len() - 3].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### List and population sorting by key\n\n```\n")
        .expect("the language reference documents sorting by key");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 105](nuc-0105.md)"));
    assert_eq!(case.expected.as_deref(), Some("\"col\""));
}

#[test]
fn every_case_is_its_accepted_counterpart_with_only_the_sort_by_written_out() {
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
        assert!(!calls_sort_by(&written), "{}", sealed.title);
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
    assert_eq!(rewrites, 11);
}

#[test]
fn ordered_keys_give_the_decorate_spelling_and_unordered_keys_give_undefined_as_max_by_does() {
    let mut keys = Keys(105);
    let mixed: Vec<&str> = ORDERED_NUMBERS
        .iter()
        .chain(&ORDERED_TEXTS)
        .copied()
        .collect();
    let pools: [&[&str]; 3] = [&ORDERED_NUMBERS, &ORDERED_TEXTS, &mixed];
    let mut compared = 0;
    let mut undefined = 0;
    for round in 0..360 {
        let pool = pools[round % pools.len()];
        let length = [0, 1, 2, 3, 5, 8, 13, 21, 24, 34][keys.pick(10)];
        let unordered = round % 4 == 3 && length > 0;
        let mut list: Vec<Option<&str>> = (0..length)
            .map(|_| Some(pool[keys.pick(pool.len())]))
            .collect();
        if unordered {
            let position = keys.pick(length);
            let which = keys.pick(UNORDERED.len() + 1);
            list[position] = UNORDERED.get(which).copied();
        }
        let program = records(&list);
        let mut runtime = Runtime::new();
        runtime.run(&program).unwrap();
        let sorted = json(&mut runtime, "xs.sortBy(x -> x.k).map(x -> x.i)");
        let decorated = json(
            &mut runtime,
            &format!("xs.map(x -> [x.k, x]){DECORATE}.map(x -> x.i)"),
        );
        let extreme = json(&mut runtime, "xs.maxBy(x -> x.k)");
        if unordered {
            assert_eq!(sorted, "null", "{program}");
            assert_eq!(extreme, "null", "{program}");
            undefined += 1;
        } else {
            assert_eq!(sorted, decorated, "{program}");
            assert_eq!(extreme == "null", length == 0, "{program}");
            compared += 1;
        }
    }
    assert_eq!(compared + undefined, 360);
    assert!(undefined > 60, "{undefined}");
}

#[test]
fn long_receivers_keep_equal_keys_in_receiver_order() {
    let keys: Vec<String> = (0..200)
        .map(|position| (position % 3).to_string())
        .collect();
    let list: Vec<Option<&str>> = keys.iter().map(|key| Some(key.as_str())).collect();
    let expected: Vec<String> = (0..3)
        .flat_map(|bucket| (0..200).filter(move |position| position % 3 == bucket))
        .map(|position| format!("{position}.0"))
        .collect();
    let expected = format!("[{}]", expected.join(","));
    let mut runtime = Runtime::new();
    runtime.run(&records(&list)).unwrap();
    assert_eq!(
        json(&mut runtime, "xs.sortBy(x -> x.k).map(x -> x.i)"),
        expected
    );
    let texts: Vec<String> = (0..200)
        .map(|position| format!("\"{}\"", position % 3))
        .collect();
    let list: Vec<Option<&str>> = texts.iter().map(|key| Some(key.as_str())).collect();
    runtime.run(&records(&list)).unwrap();
    assert_eq!(
        json(&mut runtime, "xs.sortBy(x -> x.k).map(x -> x.i)"),
        expected
    );
    let mut runtime = Runtime::new();
    runtime
        .run("class Bin(i, k):\n    this.i = i\n    this.k = k")
        .unwrap();
    for position in 0..60 {
        runtime
            .run(&format!(
                "Bin({position}, {})",
                if position % 2 == 0 { "-0" } else { "0" }
            ))
            .unwrap();
    }
    let ordered: Vec<String> = (0..60).map(|position| format!("{position}.0")).collect();
    assert_eq!(
        json(&mut runtime, "Bin.sortBy(bin -> bin.k).map(bin -> bin.i)"),
        format!("[{}]", ordered.join(","))
    );
    assert_eq!(
        json(
            &mut runtime,
            "Bin.sortBy(bin -> bin.k).map(bin -> 1 / bin.k > 0)"
        ),
        json(&mut runtime, "Bin.map(bin -> 1 / bin.k > 0)")
    );
}

#[test]
fn keys_compare_as_less_than_compares_them() {
    for (keys, expected) in [
        ("[\"10\", \"9\", \"2\"]", "[\"10\",\"2\",\"9\"]"),
        ("[10, 9, 2]", "[2.0,9.0,10.0]"),
        (
            "[true, null, false, [2], [], \"1\"]",
            "[null,false,[],true,\"1\",[2.0]]",
        ),
        ("[\"b\", 5, \"a\", 3]", "[\"a\",\"b\",3.0,5.0]"),
        ("[\"10\", 9, \"9\"]", "[9.0,\"10\",\"9\"]"),
        ("[9.5, \"10\", \"9\"]", "[\"9\",9.5,\"10\"]"),
        ("[\"B\", \"a\", \"A\", \"b\"]", "[\"A\",\"B\",\"a\",\"b\"]"),
    ] {
        assert_eq!(
            result(&format!("{keys}.sortBy(key -> key)")),
            expected,
            "{keys}"
        );
        assert_eq!(
            result(&format!("{keys}.map(x -> [x, x]){DECORATE}")),
            expected,
            "{keys}"
        );
    }
    assert_eq!(
        result(
            "xs = [new Date(9), new Date(-1), new Date(5)]\nxs.sortBy(d -> d) == [xs[1], xs[2], xs[0]]"
        ),
        "true"
    );
    assert_eq!(
        result(
            "[{n: \"a\", k: 0}, {n: \"b\", k: -0}, {n: \"c\", k: 0}].sortBy(x -> x.k).map(x -> x.n)"
        ),
        "[\"a\",\"b\",\"c\"]"
    );
    assert_eq!(
        result(
            "[{n: \"b\", k: 2}, {n: \"a\", k: 1}, {n: \"c\", k: 2}].sortBy(x -> x.k).map(x -> x.n)"
        ),
        "[\"a\",\"b\",\"c\"]"
    );
}

#[test]
fn declarations_differ_from_the_accepted_spellings_only_where_they_read_differently() {
    assert_eq!(
        result("xs = [1, null]\nr = xs.sortBy(x -> 1)\nr"),
        "[1.0,null]"
    );
    assert_eq!(result("xs = [1, null]\nr = xs.maxBy(x -> 1)\nr"), "1.0");
    assert_eq!(
        result(&format!(
            "xs = [1, null]\nr = xs.map(x -> [1, x]){DECORATE}\nr"
        )),
        "null"
    );
    assert_eq!(result("xs = [1, null]\nm = xs.map(x -> x)\nm"), "null");
    for statement in [
        "xs.sortBy(x -> 1)".to_string(),
        format!("xs.map(x -> [1, x]){DECORATE}"),
    ] {
        assert_eq!(
            result(&format!("xs = [1, null]\n{statement}")),
            "[1.0,null]"
        );
    }
    assert_eq!(
        result("xs = [{k: null}]\nr = xs[:].sort((a, b) -> a.k < b.k ? -1 : 0)\nr"),
        "[{\"k\":null}]"
    );
    assert_eq!(
        result("xs = [{k: null}]\nr = xs.sortBy(x -> x.k)\nr"),
        "null"
    );
    assert_eq!(
        result(&format!(
            "xs = [{{k: null}}]\nr = xs.map(x -> [x.k, x]){DECORATE}\nr"
        )),
        "null"
    );
    let affected = |declaration: &str| {
        result(&format!(
            "def key(x) => x.k\nr = {declaration}\n(affects key).map(step -> step.node)"
        ))
    };
    assert_eq!(affected("[].sortBy(key)"), "[\"r\"]");
    assert_eq!(affected("[].maxBy(key)"), "[\"r\"]");
    assert_eq!(
        affected(&format!("[].map(x -> [key(x), x]){DECORATE}")),
        "[]"
    );
    for (source, expected) in [
        ("xs = [1, null]\nr = xs.sortBy(x -> x)\nr", "null"),
        (
            "xs = [{k: null}, {k: -1}]\nr = xs.sortBy(x -> x.k)\nr",
            "null",
        ),
        (
            "[{k: null}, {k: -1}].sortBy(x -> x.k).map(x -> x.k)",
            "[-1.0,null]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn lists_however_they_are_reached_are_receivers() {
    for (source, expected) in [
        ("[3, 1, 2].sortBy(x -> x)", "[1.0,2.0,3.0]"),
        ("xs = [3, 1, 2]\nxs.sortBy(x -> x)", "[1.0,2.0,3.0]"),
        (
            "o = {inner: {items: [{n: \"b\"}, {n: \"a\"}]}}\no.inner.items.sortBy(i -> i.n).map(i -> i.n)",
            "[\"a\",\"b\"]",
        ),
        ("xs = [3, 1, 2]\nxs[1:].sortBy(x -> x)", "[1.0,2.0]"),
        (
            "xs = [3, 4]\nxs.map(x -> x * 2).sortBy(x -> -x)",
            "[8.0,6.0]",
        ),
        (
            "def make() => [2, 3, 1]\nmake().sortBy(x -> x)",
            "[1.0,2.0,3.0]",
        ),
        ("xs = [1, 2, 3]\nxs.value.sortBy(x -> -x)", "[3.0,2.0,1.0]"),
        ("xs = [2]\nxs.push(1)\nxs@first.sortBy(x -> x)", "[2.0]"),
        ("grid = [[3, 1], [2]]\ngrid[0].sortBy(x -> x)", "[1.0,3.0]"),
        (
            "class Shelf(items):\n    this.items = items\n    def sorted():\n        return this.items.sortBy(item -> item.price)\ns = Shelf([{price: 3}, {price: 1}])\ns.sorted().map(item -> item.price)",
            "[1.0,3.0]",
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
            "Order.sortBy(o -> o.total).map(o -> o.total)",
            "[5.0,6.0,7.0]",
        ),
        (
            "Order[1:].sortBy(o -> o.total).map(o -> o.total)",
            "[6.0,7.0]",
        ),
        (
            "Order.list().sortBy(o -> -o.total).map(o -> o.total)",
            "[7.0,6.0,5.0]",
        ),
        (
            "kind.sortBy(o -> o.total).map(o -> o.total)",
            "[5.0,6.0,7.0]",
        ),
        (
            "$Order.sortBy(o -> o.total).map(o -> o.total)",
            "[5.0,6.0,7.0]",
        ),
        (
            "Order.filter(o -> o.total > 5).sortBy(o -> -o.total).map(o -> o.total)",
            "[7.0,6.0]",
        ),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    assert_eq!(
        result(
            "class Bag(n):\n    this.n = n\n    def sortBy(f): return 9\na = Bag(2)\nb = Bag(1)\n[Bag.sortBy(bag -> bag.n) == [b, a], a.sortBy(1)]"
        ),
        "[true,9.0]"
    );
    assert_eq!(result("class Empty\nEmpty.sortBy(e -> e)"), "[]");
}

#[test]
fn items_keep_their_identity_and_the_receiver_is_never_written() {
    for (source, expected) in [
        (
            "a = {k: 2}\nb = {k: 1}\nxs = [a, b]\nys = xs.sortBy(x -> x.k)\n[ys[0] == b, ys[1] == a, xs[0] == a, xs.length]",
            "[true,true,true,2.0]",
        ),
        (
            "o = {items: [3, 1, 2]}\nsorted = o.items.sortBy(x -> x)\n[o.items, sorted]",
            "[[3.0,1.0,2.0],[1.0,2.0,3.0]]",
        ),
        (
            "class T(n):\n    this.n = n\nT(2)\nT(1)\ns = T.sortBy(t -> t.n)\n[T.map(t -> t.n), s.map(t -> t.n)]",
            "[[2.0,1.0],[1.0,2.0]]",
        ),
        (
            "scores = [30, 10, 20]\nup = scores.sortBy(s -> s)\ndown = scores.sortBy(s -> -s)\nscores.push(40)\n[scores, up, down]",
            "[[30.0,10.0,20.0,40.0],[10.0,20.0,30.0,40.0],[40.0,30.0,20.0,10.0]]",
        ),
        ("[].sortBy(x -> x)", "[]"),
        ("xs = []\nys = xs.sortBy(x -> x)\nys", "[]"),
        (
            "class Job\nqueue = Job.sortBy(job -> job.due)\nfirst := queue\nJob()\n[first, queue]",
            "[[],null]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn missing_null_and_non_function_selectors_raise_even_on_empty_receivers() {
    for source in [
        "xs = [1, 2]\nxs.sortBy()",
        "xs = [1, 2]\nxs.sortBy(null)",
        "xs = [1, 2]\nxs.sortBy(5)",
        "xs = [1, 2]\nxs.sortBy(\"k\")",
        "[].sortBy()",
        "[].sortBy(null)",
        "class Empty\nEmpty.sortBy(1)",
        "class Empty\nEmpty.sortBy()",
        "class Order(total):\n    this.total = total\nOrder(1)\nOrder.sortBy(\"total\")",
    ] {
        assert_eq!(
            failure(source),
            (ErrorKind::Type, "sortBy expects a function".to_string()),
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    runtime.run("xs = [2, 1]\nys = xs.sortBy(x -> x)").unwrap();
    assert_eq!(
        outcome(&mut runtime, "ys = xs.sortBy(1)"),
        "TypeError: sortBy expects a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(json(&mut runtime, "xs.push(0)\nys"), "[0.0,1.0,2.0]");
}

#[test]
fn arguments_are_evaluated_once_left_to_right_after_the_receiver() {
    assert_eq!(
        result(
            "seen = []\ndef mark(label, value):\n    seen.push(label)\n    return value\nxs = [3, 1]\n[mark(\"receiver\", xs).sortBy(mark(\"selector\", x -> x), mark(\"extra\", 0)), seen]"
        ),
        "[[1.0,3.0],[\"receiver\",\"selector\",\"extra\"]]"
    );
    assert_eq!(
        result(
            "seen = []\ndef weigh(item, second):\n    seen.push([item.n, second])\n    return item.k\nxs = [{n: \"a\", k: 2}, {n: \"b\", k: 1}, {n: \"c\", k: 2}]\n[xs.sortBy(weigh).map(x -> x.n), seen]"
        ),
        "[[\"b\",\"a\",\"c\"],[[\"a\",null],[\"b\",null],[\"c\",null]]]"
    );
    assert_eq!(
        result(
            "seen = []\ndef key(item):\n    seen.push(item.n)\n    return item.k\nxs = [{n: \"a\", k: 0 / 0}, {n: \"b\", k: 1}]\n[xs.sortBy(key), seen]"
        ),
        "[null,[\"a\",\"b\"]]"
    );
    assert_eq!(
        result(
            "seen = []\ndef mark() => seen.push(1)\nxs = null\nr = xs.sortBy(mark())\n[r, seen]"
        ),
        "[null,[]]"
    );
    assert_eq!(
        failure("xs = [1, 2]\nxs.sortBy(missing)"),
        (ErrorKind::Reference, "missing is not defined".to_string())
    );
    assert_eq!(
        failure("xs = [1, 2]\nxs.sortBy(5, missing)"),
        (ErrorKind::Reference, "missing is not defined".to_string())
    );
}

#[test]
fn declarations_follow_list_mutations_and_property_changes() {
    behaves_as_decorated(&[
        (
            "xs = [{n: \"a\", k: 3}, {n: \"b\", k: 1}]",
            "xs = [{n: \"a\", k: 3}, {n: \"b\", k: 1}]",
        ),
        (
            "ys = xs.sortBy(x -> x.k)",
            &format!("ys = xs.map(x -> [x.k, x]){DECORATE}"),
        ),
        ("ys.map(y -> y.n)", "ys.map(y -> y.n)"),
        ("xs.push({n: \"c\", k: 2})", "xs.push({n: \"c\", k: 2})"),
        ("ys.map(y -> y.n)", "ys.map(y -> y.n)"),
        ("xs.pop()", "xs.pop()"),
        ("xs.shift()", "xs.shift()"),
        ("ys.map(y -> y.n)", "ys.map(y -> y.n)"),
        (
            "xs = [{n: \"d\", k: 0}, {n: \"e\", k: -1}]",
            "xs = [{n: \"d\", k: 0}, {n: \"e\", k: -1}]",
        ),
        ("xs[0].k = -2", "xs[0].k = -2"),
        ("ys.map(y -> y.n)", "ys.map(y -> y.n)"),
    ]);
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "xs = [{n: \"a\", k: 3}, {n: \"b\", k: 1}]\nys = xs.sortBy(x -> x.k).map(x -> x.n)\nys",
            "[\"b\",\"a\"]",
        ),
        ("xs.push({n: \"c\", k: 2})\nys", "[\"b\",\"c\",\"a\"]"),
        ("xs.pop()\nys", "[\"b\",\"a\"]"),
        ("xs.shift()\nys", "[\"b\"]"),
        (
            "xs = [{n: \"d\", k: 0}, {n: \"e\", k: -1}]\nys",
            "[\"e\",\"d\"]",
        ),
        ("xs[0].k = -2\nys", "[\"d\",\"e\"]"),
        ("xs[1].k = 0 / 0\nys", "null"),
        ("xs[1].k = 9\nys", "[\"d\",\"e\"]"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn declarations_follow_new_and_deleted_instances_and_class_level_rules() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class R(n, t):\n    this.n = n\n    this.t = t\nboard = R.sortBy(r -> r.t)\nboard",
            "[]",
        ),
        ("a = R(\"a\", 3)\nb = R(\"b\", 1)\nboard == [b, a]", "true"),
        ("c = R(\"c\", 2)\nboard == [b, c, a]", "true"),
        ("b.t = 5\nboard == [c, a, b]", "true"),
        ("delete c.t\nboard", "null"),
        ("delete c.n\ndelete c\nboard == [a, b]", "true"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class Team(name):\n    this.name = name\nclass Player(team, name, goals):\n    this.team = team\n    this.name = name\n    this.goals = goals\n$Team.roster = Player.filter(p -> p.team == $Team.name).sortBy(p -> -p.goals).map(p -> p.name)\nreds = Team(\"reds\")\nblues = Team(\"blues\")\nPlayer(\"reds\", \"ana\", 3)\nPlayer(\"blues\", \"bo\", 5)\ncy = Player(\"reds\", \"cy\", 7)\n[reds.roster, blues.roster]",
            "[[\"cy\",\"ana\"],[\"bo\"]]",
        ),
        (
            "Player(\"blues\", \"di\", 9)\nblues.roster",
            "[\"di\",\"bo\"]",
        ),
        ("cy.goals = 1\nreds.roster", "[\"ana\",\"cy\"]"),
        (
            "cy.team = \"blues\"\n[reds.roster, blues.roster]",
            "[[\"ana\"],[\"di\",\"bo\",\"cy\"]]",
        ),
        ("greens = Team(\"greens\")\ngreens.roster", "[]"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    assert_eq!(
        result(
            "class P(x):\n    this.x = x\n$P.rank = P.sortBy(other -> Math.abs(other.x - $P.x)).indexOf($P)\np = P(1)\nq = P(5)\n[p.rank, q.rank]"
        ),
        "[0.0,0.0]"
    );
}

#[test]
fn rendering_and_dependencies_name_the_call_as_written() {
    let mut runtime = Runtime::new();
    assert_eq!(
        json(
            &mut runtime,
            "rows = [2, 1]\nranked = rows.sortBy(row -> row)\n(why ranked).map(step -> step.rule)"
        ),
        "[\"ranked = rows.sortBy(function())\",\"rows = [2,1]\"]"
    );
    assert_eq!(
        json(&mut runtime, "(affects rows).map(step -> step.node)"),
        "[\"ranked\"]"
    );
    assert_eq!(dependencies(&runtime, "ranked"), ["rows"]);
    let mut runtime = Runtime::new();
    runtime
        .run("class Bid(amount):\n    this.amount = amount\nbook = Bid.sortBy(bid -> bid.amount)\nBid(5)")
        .unwrap();
    assert_eq!(dependencies(&runtime, "book"), ["$Bid", "obj:0.amount"]);
    let mut spelled = Runtime::new();
    spelled
        .run(&format!(
            "rows = [2, 1]\nranked = rows.map(row -> [row, row]){DECORATE}"
        ))
        .unwrap();
    assert_eq!(
        json(&mut spelled, "(why ranked).map(step -> step.rule)"),
        "[\"ranked = rows.map(function()).sort(function()).map(function())\",\"rows = [2,1]\"]"
    );
}

#[test]
fn failed_statements_and_transactions_restore_sorted_declarations() {
    behaves_as_decorated(&[
        ("rows = [{k: 2}, {k: 1}]", "rows = [{k: 2}, {k: 1}]"),
        (
            "sorted = rows.sortBy(row -> row.k > 9 ? row.k.toUpperCase() : row.k)",
            &format!(
                "sorted = rows.map(row -> [row.k > 9 ? row.k.toUpperCase() : row.k, row]){DECORATE}"
            ),
        ),
        (
            "try:\n    rows.push({k: 0})\n    assert(sorted.map(row -> row.k), [0, 1, 2])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
            "try:\n    rows.push({k: 0})\n    assert(sorted.map(row -> row.k), [0, 1, 2])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
        ),
        (
            "[rows.length, sorted.map(row -> row.k)]",
            "[rows.length, sorted.map(row -> row.k)]",
        ),
        ("rows.push({k: 10})", "rows.push({k: 10})"),
        (
            "[rows.length, sorted.map(row -> row.k)]",
            "[rows.length, sorted.map(row -> row.k)]",
        ),
        ("rows.push({k: 3})", "rows.push({k: 3})"),
        ("sorted.map(row -> row.k)", "sorted.map(row -> row.k)"),
    ]);
    let mut runtime = Runtime::new();
    runtime
        .run("rows = [{k: 2}, {k: 1}]\nsorted = rows.sortBy(row -> row.k > 9 ? row.k.toUpperCase() : row.k)")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "rows.push({k: 10})"),
        "TypeError: number.toUpperCase is not a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(
        json(&mut runtime, "[rows.length, sorted.map(row -> row.k)]"),
        "[2.0,[1.0,2.0]]"
    );
    assert_eq!(
        result(
            "class Probe(t):\n    this.t = t\nProbe(1)\nProbe(2)\ncoldest = Probe.sortBy(p -> p.t > 9 ? p.t.toUpperCase() : p.t)\nfailure = \"\"\ntry:\n    Probe(50)\ncatch error:\n    failure = error\n[failure, Probe.length, coldest.map(p -> p.t)]"
        ),
        "[\"TypeError: number.toUpperCase is not a function\",2.0,[1.0,2.0]]"
    );
}

#[test]
fn large_receivers_sort_through_the_public_api() {
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
            "sorted = holder.items.sortBy(x -> -x)\n[sorted.length, sorted[0], sorted[99999], sorted.every(x -> x >= 0)]"
        ),
        "[100000.0,99999.0,0.0,true]"
    );
    assert_eq!(
        json(&mut runtime, "holder.items[:3]"),
        "[0.0,7919.0,15838.0]"
    );
}

#[test]
fn unchanged_readings_keep_their_accepted_results() {
    for (source, expected) in [
        ("xs = [1]\nxs.sortBy", "null"),
        ("class Order\nOrder.sortBy", "null"),
        ("o = {sortBy: 3}\no.sortBy", "3.0"),
        ("x = {sortBy: 4}.sortBy\nx", "4.0"),
        ("o = {sortBy: n -> n * 10}\no.sortBy(4)", "40.0"),
        ("r = {sortBy: f -> 5}.sortBy(x -> x)\nr", "5.0"),
        ("class P\np = P()\np.sortBy(x -> x)", "null"),
        ("r = {a: 1}\nr.sortBy(x -> x)", "null"),
        (
            "class Bag:\n    def sortBy(f): return 9\nbag = Bag()\nbag.sortBy(1)",
            "9.0",
        ),
        (
            "sortBy = 3\ndef scale sortBy => sortBy * 2\n[sortBy, scale(sortBy)]",
            "[3.0,6.0]",
        ),
        ("xs = null\nxs.sortBy(i -> i)", "null"),
        (
            "xs = [3, 1, 2]\nranked = xs.sort((a, b) -> a - b)\n[xs, ranked]",
            "[[1.0,2.0,3.0],[1.0,2.0,3.0]]",
        ),
        (
            "xs = [3, 1, 2]\nranked = xs.slice().sort((a, b) -> a < b ? -1 : 0)\nxs.push(0)\n[xs, ranked]",
            "[[3.0,1.0,2.0,0.0],[0.0,1.0,2.0,3.0]]",
        ),
        ("[{k: 2}, {k: 5}].maxBy(x -> x.k).k", "5.0"),
        ("[{k: 2}, {k: 5}].minBy(x -> x.k).k", "2.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, kind, message) in [
        (
            "\"abc\".sortBy(x -> x)",
            ErrorKind::Type,
            "abc.sortBy is not a function",
        ),
        (
            "(5).sortBy(x -> x)",
            ErrorKind::Type,
            "number.sortBy is not a function",
        ),
        (
            "true.sortBy(x -> x)",
            ErrorKind::Type,
            "boolean.sortBy is not a function",
        ),
        (
            "d = new Date(5)\nd.sortBy(x -> x)",
            ErrorKind::Type,
            "Date.sortBy is not a function",
        ),
        (
            "f = x => x\nf.sortBy(x -> x)",
            ErrorKind::Type,
            "function.sortBy is not a function",
        ),
        (
            "/a/.sortBy(x -> x)",
            ErrorKind::Type,
            "sortBy is not a function",
        ),
        (
            "Item = 5\nItem.sortBy(i -> i)",
            ErrorKind::Type,
            "number.sortBy is not a function",
        ),
        (
            "Math.sortBy(x -> x)",
            ErrorKind::Type,
            "Math.sortBy is not a function",
        ),
        (
            "Number.sortBy(x -> x)",
            ErrorKind::Type,
            "Number.sortBy is not a function",
        ),
        (
            "String.sortBy(x -> x)",
            ErrorKind::Type,
            "String.sortBy is not a function",
        ),
        (
            "Date.sortBy(x -> x)",
            ErrorKind::Type,
            "Date.sortBy is not a function",
        ),
        (
            "List.sortBy(x -> x)",
            ErrorKind::Reference,
            "List is not defined",
        ),
        (
            "Object.sortBy(x -> x)",
            ErrorKind::Reference,
            "Object is not defined",
        ),
        (
            "Boolean.sortBy(x -> x)",
            ErrorKind::Reference,
            "Boolean is not defined",
        ),
        (
            "class Order\nClass.sortBy(x -> x)",
            ErrorKind::Reference,
            "Class is not defined",
        ),
        (
            "sortBy([1, 2])",
            ErrorKind::Reference,
            "sortBy is not defined",
        ),
        (
            "sortBy of [1, 2]",
            ErrorKind::Syntax,
            "Unexpected keyword 'of'",
        ),
        (
            "xs = [1]\nxs.sortedBy(x -> x)",
            ErrorKind::Type,
            "xs.sortedBy is not a function",
        ),
    ] {
        assert_eq!(failure(source), (kind, message.to_string()), "{source}");
    }
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-list-key-sorting-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_sort_by() {
    let program = "rows = [2, 3, 1]\nranked = rows.sortBy(row -> -row)\nrows.push(0)\nranked";
    let file = SourceFile::new("file");
    let sorted = file.run(program, &["--json"]);
    assert!(sorted.status.success(), "{sorted:?}");
    assert!(sorted.stderr.is_empty(), "{sorted:?}");
    assert_eq!(
        String::from_utf8(sorted.stdout).unwrap(),
        "[3.0,2.0,1.0,0.0]\n"
    );
    let plain = file.run(program, &[]);
    assert!(plain.status.success(), "{plain:?}");
    let graph = String::from_utf8(file.run(program, &["--json", "--graph"]).stdout).unwrap();
    assert!(
        graph.contains(
            "  rows [variable]\n    updates  ranked\n  ranked [variable]\n    reads    rows"
        ),
        "{graph}"
    );
    let failed = file.run("xs = [1]\nxs.sortBy()", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "TypeError: sortBy expects a function (line 2, column 1)\n"
    );
    let session = "xs = [3, 1, 2]\nys = xs.sortBy(x -> x)\nys\nxs.push(0)\nys\nxs\nxs.sortBy()\n.clear\nxs = [\"b\", \"a\"]\nxs.sortBy(x -> x)\n.exit\n";
    for plain in [false, true] {
        for newline in ["\n", "\r\n"] {
            let options: &[&str] = if plain { &["--plain"] } else { &[] };
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(
                stdout.contains("[1.0,2.0,3.0]\n4.0\n[0.0,1.0,2.0,3.0]\n[3.0,1.0,2.0,0.0]\n"),
                "{stdout}"
            );
            assert!(stdout.contains("[\"a\",\"b\"]\n"), "{stdout}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                "TypeError: sortBy expects a function (line 1, column 1)\n"
            );
        }
    }
}
