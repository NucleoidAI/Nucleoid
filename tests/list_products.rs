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
    "Nucleoid multiplies lists with product",
    "Nucleoid multiplies class instance populations with product",
    "Nucleoid keeps products live in class level rules",
    "Nucleoid rolls back failed product updates",
];
const REFERENCE_TITLE: &str = "Nucleoid multiplies lists and populations with product";
const README_TITLE: &str = "Products keep the chance that a trip connects live as flights change";
const EXAMPLES_TITLE: &str = "Products keep every account's compounded balance live";
const METHOD_TITLES: [&str; 2] = [
    "Nucleoid preserves bare multiple parameter methods",
    "Nucleoid preserves comma-free methods and initializers",
];

const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid multiplies lists with product",
    "source": "# Nucleoid multiplies lists with product\n\ndef m a b => a * b\nsides = [2, 3, 4]\nvolume = sides.reduce(m, 1)\nfloor = sides.slice(0, 2).reduce(m, 1)\nassert([volume, floor], [24, 6])\nsides.pop()\nsides.push(10)\nassert([volume, floor], [60, 6])\nsides = []\nassert([volume, floor], [1, 1])\naccount = {rates: [0.1, 0.1]}\ngrowth = account.rates.map(rate -> 1 + rate).reduce(m)\nassert(growth, 1.2100000000000002)\naccount.rates.push(0.1)\nassert(growth, 1.3310000000000004)\naccount.rates.push(-0.5)\nassert(growth, 0.6655000000000002)",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid multiplies class instance populations with product",
    "source": "# Nucleoid multiplies class instance populations with product\n\ndef g0 t part => t * part.works\nclass Part(name, works):\n    this.name = name\n    this.works = works\npump = Part(\"pump\", 0.5)\nPart(\"valve\", 0.75)\nchance = Part.reduce(g0, 1)\nnewest = Part[-1:].reduce(g0, 1)\nassert([chance, newest], [0.375, 0.75])\nPart(\"seal\", 0.5)\nassert([chance, newest], [0.1875, 0.5])\npump.works = 1\nassert(chance, 0.375)\nclass Day(dry): this.dry = dry\nstaysDry = Day.reduce(lambda t, day: t * day.dry, 1)\nassert(staysDry, 1)\nDay(0.5)\nDay(0.25)\nassert(staysDry, 0.125)",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid keeps products live in class level rules",
    "source": "# Nucleoid keeps products live in class level rules\n\ndef m a b => a * b\nclass Box(sides): this.sides = sides\n$Box.volume = .sides.reduce(m, 1)\ncrate = Box([2, 3, 4])\ncube = Box([3, 3, 3])\nassert([crate.volume, cube.volume], [24, 27])\ncrate.sides.push(0.5)\ncube.sides = []\nassert([crate.volume, cube.volume], [12, 1])\nclass Route(name): this.name = name\nclass Leg(route, onTime):\n    this.route = route\n    this.onTime = onTime\n$Route.chance = Leg.filter(leg -> leg.route == $Route).map(.onTime).reduce(m, 1)\nnorth = Route(\"north\")\nsouth = Route(\"south\")\nLeg(north, 0.5)\nslow = Leg(north, 0.5)\nLeg(south, 0.75)\nnetwork = Route.map(.chance).reduce(m, 1)\nassert([north.chance, south.chance, network], [0.25, 0.75, 0.1875])\nslow.onTime = 1\nLeg(south, 0.5)\nassert([north.chance, south.chance, network], [0.5, 0.375, 0.1875])\nwest = Route(\"west\")\nassert([west.chance, network], [1, 0.1875])\nLeg(west, 0.5)\nassert(network, 0.09375)",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back failed product updates",
    "source": "# Nucleoid rolls back failed product updates\n\ndef checked(chance):\n    if chance > 1:\n        throw \"NOT_A_CHANCE\"\n    return chance\nchances = [0.5, 0.5]\nboth = chances.reduce(lambda t, x: t * checked(x), 1)\nfailure = \"\"\ntry:\n    chances.push(2)\ncatch error:\n    failure = error\nassert([failure, chances, both], [\"NOT_A_CHANCE\", [0.5, 0.5], 0.25])\ntry:\n    chances.push(0.5)\n    assert(both, 0.125)\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([chances, both], [[0.5, 0.5], 0.25])\nclass Stage(chance): this.chance = chance\nStage(0.5)\npasses = Stage.reduce(lambda t, stage: t * checked(stage.chance), 1)\nfailure = \"\"\ntry:\n    Stage(4)\ncatch error:\n    failure = error\nassert([failure, Stage.length, passes], [\"NOT_A_CHANCE\", 1, 0.5])\nStage(0.25)\npasses",
    "expected": "0.125"
  },
  {
    "document": "reference",
    "title": "Nucleoid multiplies lists and populations with product",
    "source": "# Nucleoid multiplies lists and populations with product\n\ndef m a b => a * b\nsides = [2, 3, 4]\nvolume = sides.reduce(m, 1)\nclass Check(name, chance):\n    this.name = name\n    this.chance = chance\nCheck(\"lint\", 0.5)\nbuild = Check(\"build\", 0.75)\ngreen = Check.map(.chance).reduce(m, 1)\nassert([volume, green], [24, 0.375])\nsides.push(0.5)\nbuild.chance = 1\nCheck(\"deploy\", 0.5)\nassert([volume, green], [12, 0.25])\nsides = []\nvolume",
    "expected": "1"
  },
  {
    "document": "readme",
    "title": "Products keep the chance that a trip connects live as flights change",
    "source": "class Flight(code, onTime):\n    this.code = code\n    this.onTime = onTime\nfirst = Flight(\"AB1\", 0.5)\nFlight(\"CD2\", 0.75)\ntrip = Flight.reduce(lambda t, flight: t * flight.onTime, 1)\nassert(trip, 0.375)\nfirst.onTime = 0.75\nFlight(\"EF3\", 0.5)\nassert(trip, 0.28125)\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Products keep every account's compounded balance live",
    "source": "class Account(start, rates):\n    this.start = start\n    this.rates = rates\n$Account.balance = $Account.start * .rates.reduce(lambda t, rate: t * (1 + rate), 1)\nsaver = Account(100, [0.2, 0.1])\nopened = Account(200, [])\nassert([saver.balance, opened.balance], [132, 200])\nsaver.rates.push(0.25)\nopened.rates.push(1)\n[saver.balance, opened.balance]\n",
    "expected": "[165,400]"
  }
]"###;

const WRITTEN_OUT: [(&str, &str); 14] = [
    ("$Box.sides.product()", ".sides.reduce(m, 1)"),
    (
        "sides.slice(0, 2).product()",
        "sides.slice(0, 2).reduce(m, 1)",
    ),
    ("sides.product()", "sides.reduce(m, 1)"),
    (
        "account.rates.product(rate -> 1 + rate)",
        "account.rates.map(rate -> 1 + rate).reduce(m)",
    ),
    ("Part.product(part -> part.works)", "Part.reduce(g0, 1)"),
    (
        "Part[-1:].product(part -> part.works)",
        "Part[-1:].reduce(g0, 1)",
    ),
    (
        "Day.product(day -> day.dry)",
        "Day.reduce(lambda t, day: t * day.dry, 1)",
    ),
    (
        "Leg.filter(leg -> leg.route == $Route).product(leg -> leg.onTime)",
        "Leg.filter(leg -> leg.route == $Route).map(.onTime).reduce(m, 1)",
    ),
    (
        "Route.product(route -> route.chance)",
        "Route.map(.chance).reduce(m, 1)",
    ),
    (
        "chances.product(checked)",
        "chances.reduce(lambda t, x: t * checked(x), 1)",
    ),
    (
        "Stage.product(stage -> checked(stage.chance))",
        "Stage.reduce(lambda t, stage: t * checked(stage.chance), 1)",
    ),
    (
        "Check.product(check -> check.chance)",
        "Check.map(.chance).reduce(m, 1)",
    ),
    (
        "Flight.product(flight -> flight.onTime)",
        "Flight.reduce(lambda t, flight: t * flight.onTime, 1)",
    ),
    (
        "$Account.rates.product(rate -> 1 + rate)",
        ".rates.reduce(lambda t, rate: t * (1 + rate), 1)",
    ),
];

const HELPERS: [(&str, &str); 4] = [
    (
        "Nucleoid multiplies lists with product",
        "def m a b => a * b",
    ),
    (
        "Nucleoid multiplies class instance populations with product",
        "def g0 t part => t * part.works",
    ),
    (
        "Nucleoid keeps products live in class level rules",
        "def m a b => a * b",
    ),
    (
        "Nucleoid multiplies lists and populations with product",
        "def m a b => a * b",
    ),
];

const VALUES: [&str; 29] = [
    "3",
    "-2.5",
    "-0",
    "0",
    "true",
    "false",
    "null",
    "\"\"",
    "\"  \"",
    "\" 4 \"",
    "\"7e1\"",
    "\"x\"",
    "\"1,2\"",
    "[]",
    "[5]",
    "[\"6\"]",
    "[2, 3]",
    "{}",
    "{k: 1}",
    "when",
    "f",
    "/a/",
    "order",
    "Order",
    "0 / 0",
    "Number.POSITIVE_INFINITY",
    "Number.NEGATIVE_INFINITY",
    "1e308",
    "holder.missing",
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

fn product_receivers(source: &str) -> Vec<Token> {
    let tokens: Vec<Token> = tokenize(source)
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    tokens
        .windows(4)
        .filter_map(|window| match window {
            [
                receiver,
                Token::Dot,
                Token::Identifier(name),
                Token::LeftParen,
            ] if name == "product" => Some(receiver.clone()),
            _ => None,
        })
        .collect()
}

fn calls_product(source: &str) -> bool {
    !product_receivers(source).is_empty()
}

fn written_out(case: &common::Case) -> (String, usize) {
    let mut written = case.source.clone();
    let mut rewrites = 0;
    for (call, spelling) in WRITTEN_OUT {
        rewrites += written.matches(call).count();
        written = written.replace(call, spelling);
    }
    if let Some((_, helper)) = HELPERS.iter().find(|(title, _)| *title == case.title) {
        let (head, rest) = written.split_once("\n\n").unwrap();
        written = format!("{head}\n\n{helper}\n{rest}");
    }
    (written, rewrites)
}

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "product regression".to_string(),
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

fn reading(runtime: &mut Runtime, source: &str) -> String {
    match runtime.run(source) {
        Ok(Value::Number(number)) if number.is_nan() => "NaN".to_string(),
        Ok(Value::Number(number)) => format!("{:#018x}", number.to_bits()),
        Ok(other) => format!("{other:?}"),
        Err(error) => format!("{}: {}", error.kind(), error.message()),
    }
}

fn number(runtime: &mut Runtime, source: &str) -> f64 {
    match runtime.run(source).unwrap() {
        Value::Number(number) => number,
        other => panic!("{source}: {other:?}"),
    }
}

fn same_number(left: f64, right: f64) -> bool {
    left.to_bits() == right.to_bits() || (left.is_nan() && right.is_nan())
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
    let mut multiplied = Runtime::new();
    let mut reduced = Runtime::new();
    for (step, spelling) in steps {
        let before = multiplied.assertions_run();
        assert_eq!(
            renumbered(&outcome(&mut multiplied, step)),
            renumbered(&outcome(&mut reduced, spelling)),
            "{step}"
        );
        assert_eq!(
            multiplied.assertions_run() - before,
            common::assertions::protected_assertions(&parse(step).unwrap()).len(),
            "{step}"
        );
        for name in names {
            if multiplied
                .graph
                .retrieve(&NodeKey::variable(*name))
                .is_some()
            {
                assert_eq!(
                    dependencies(&multiplied, name),
                    dependencies(&reduced, name),
                    "{step}: {name}"
                );
            }
        }
    }
    assert_eq!(multiplied.assertions_run(), reduced.assertions_run());
    assert!(multiplied.take_assertions().is_empty(), "{steps:?}");
    assert!(reduced.take_assertions().is_empty(), "{steps:?}");
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
fn only_the_appended_cases_multiply_with_product_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        let method = METHOD_TITLES.contains(&case.title.as_str());
        assert_eq!(calls_product(&case.source), new || method, "{}", case.title);
        if method {
            assert_eq!(
                product_receivers(&case.source),
                [Token::Identifier("calculator".to_string())],
                "{}",
                case.title
            );
        }
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
        .chain(METHOD_TITLES)
    {
        let matching = documents
            .iter()
            .flat_map(|(_, cases)| cases)
            .filter(|case| case.title == title)
            .count();
        assert_eq!(matching, 1, "{title}");
    }
    assert_eq!(assertions, 27);
    assert_eq!(cases, 1177);
}

#[test]
fn the_appended_cases_stand_after_nuc_112_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[772],
        "Nucleoid rolls back failed updates through node names"
    );
    assert_eq!(titles[773..777], SPEC_TITLES);
    assert_eq!(titles[787], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid names the nodes of selections with nodes",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Nodes names the facts a heater rests on and what insulation moves",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Nodes names the facts and rules behind a loan decision",
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
        .split_once("### List and population products\n\n```\n")
        .expect("the language reference documents products");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 114](nuc-0114.md)"));
    assert_eq!(case.expected.as_deref(), Some("1"));
}

#[test]
fn every_case_is_its_accepted_counterpart_with_only_the_products_written_out() {
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
        assert!(!calls_product(&written), "{}", sealed.title);
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
    assert_eq!(rewrites, 15);
}

#[test]
fn products_equal_the_accumulator_reduce_bit_for_bit() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut lists: Vec<String> = [
        "",
        "-0",
        "-0, -0",
        "0, -0",
        "-1, 0",
        "1.1, 2.2, 3.3",
        "0.1, 0.1, 0.1",
        "1e308, 10",
        "1e308, 10, 0",
        "5e-324, 0.5",
        "1e-308, 1e-308",
        "0 / 0, 0",
        "Number.POSITIVE_INFINITY, 0",
        "Number.NEGATIVE_INFINITY, -1",
        "9007199254740993, 3",
        "-2, -3, -4",
        "0.5, 0.25, 0.125",
    ]
    .map(str::to_string)
    .into();
    for _ in 0..400 {
        let length = (next() % 9) as usize;
        let items: Vec<String> = (0..length)
            .map(|_| {
                let raw = (next() % 2_000_001) as f64 - 1_000_000.0;
                let scale = [1.0, 3.0, 10.0, 1000.0, 1_000_000.0][(next() % 5) as usize];
                format!("{}", raw / scale)
            })
            .collect();
        lists.push(items.join(", "));
    }
    let mut runtime = Runtime::new();
    for items in &lists {
        let product = number(&mut runtime, &format!("[{items}].product()"));
        let reduced = number(
            &mut runtime,
            &format!("[{items}].reduce((total, item) => total * item, 1)"),
        );
        assert!(
            same_number(product, reduced),
            "[{items}]: {product:?} and {reduced:?}"
        );
        let identity = number(&mut runtime, &format!("[{items}].product(item -> item)"));
        assert!(same_number(product, identity), "[{items}] with a selector");
        let selected = number(
            &mut runtime,
            &format!("[{items}].product(item -> item * 2 - 1)"),
        );
        let mapped = number(
            &mut runtime,
            &format!("[{items}].reduce((total, item) => total * (item * 2 - 1), 1)"),
        );
        assert!(
            same_number(selected, mapped),
            "[{items}]: {selected:?} and {mapped:?}"
        );
    }
    assert!(same_number(
        number(&mut runtime, "[1.1, 2.2, 3.3].product()"),
        7.986000000000001
    ));
    assert!(same_number(
        number(&mut runtime, "[0.1, 0.1, 0.1].product(x -> 1 + x)"),
        1.3310000000000004
    ));
    assert_eq!(
        number(&mut runtime, "1 / [-1, 0].product()"),
        f64::NEG_INFINITY
    );
    assert_eq!(
        number(&mut runtime, "1 / [-0].product()"),
        f64::NEG_INFINITY
    );
    assert_eq!(number(&mut runtime, "1 / [-0].sum()"), f64::INFINITY);
    assert_eq!(number(&mut runtime, "[].product()"), 1.0);
    assert_eq!(number(&mut runtime, "[1e308, 10].product()"), f64::INFINITY);
    assert!(number(&mut runtime, "[Number.POSITIVE_INFINITY, 0].product()").is_nan());
}

#[test]
fn products_convert_values_with_the_conversion_of_multiplication() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Order\norder = Order()\nholder = {}\nwhen = Date(5)\nf = x -> x")
        .unwrap();
    for value in VALUES {
        for (product, reduced) in [
            (
                format!("[{value}].product()"),
                format!("[{value}].reduce((t, x) => t * x, 1)"),
            ),
            (
                format!("[2, {value}].product()"),
                format!("[2, {value}].reduce((t, x) => t * x, 1)"),
            ),
            (
                format!("[{value}, 2].product()"),
                format!("[{value}, 2].reduce((t, x) => t * x, 1)"),
            ),
            (
                format!("[{value}].product(x -> x)"),
                format!("[{value}].reduce((t, x) => t * x, 1)"),
            ),
            (format!("[{value}].product()"), format!("1 * ({value})")),
        ] {
            assert_eq!(
                reading(&mut runtime, &product),
                reading(&mut runtime, &reduced),
                "{product}"
            );
        }
    }
    for (source, expected) in [
        ("[2, \"3\", true, null].product()", "0.0"),
        ("[2, \" 3 \", true].product()", "6.0"),
        ("[[2], [3]].product()", "6.0"),
        ("[2, [\"4\"]].product()", "8.0"),
        ("[3, \"\", 5].product()", "0.0"),
        ("[2, false].product()", "0.0"),
        ("d = Date(4)\n[d, 2].product()", "8.0"),
        ("String([2, \"x\"].product())", "\"NaN\""),
        ("String([2, {}].product())", "\"NaN\""),
        ("String([2, [1, 2]].product())", "\"NaN\""),
        ("String([2, x -> x].product())", "\"NaN\""),
        ("String([1e308, 10].product())", "\"Infinity\""),
        ("String([-1e308, 10].product())", "\"-Infinity\""),
        ("String([Number.POSITIVE_INFINITY, 0].product())", "\"NaN\""),
        ("String(1 / [-1, 0].product())", "\"-Infinity\""),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn an_undefined_value_makes_the_product_undefined_after_visiting_every_item() {
    for (source, expected) in [
        ("o = {}\n[2, o.missing, 3].product()", "null"),
        (
            "o = {}\nString([2, o.missing, 3].product())",
            "\"undefined\"",
        ),
        (
            "o = {}\nString([0 / 0, o.missing].product())",
            "\"undefined\"",
        ),
        (
            "o = {}\nString([o.missing, 0 / 0].product())",
            "\"undefined\"",
        ),
        ("o = {}\nString([0, o.missing].product())", "\"undefined\""),
        (
            "o = {}\nString([{w: 2}, {}].product(x -> x.w))",
            "\"undefined\"",
        ),
        (
            "o = {}\nseen = []\ndef pick(x):\n    seen.push(x)\n    return x == 3 ? o.missing : x\n[String([2, 3, 4].product(pick)), seen]",
            "[\"undefined\",[2.0,3.0,4.0]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("xs = [{w: 2}, {}]\np = xs.product(x -> x.w)\np", "null"),
        ("xs[1].w = 5\np", "10.0"),
        ("delete xs[1].w\np", "null"),
        ("xs.pop()\np", "2.0"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn null_items_count_zero_while_reads_inside_a_selector_settle() {
    for (source, expected) in [
        ("p = [2, null].product()\np", "0.0"),
        ("p = [2, null].reduce((t, x) => t * x, 1)\np", "null"),
        ("[2, null].product()", "0.0"),
        ("[2, null].reduce((t, x) => t * x, 1)", "0.0"),
        ("p = [1, null].sum()\np", "1.0"),
        (
            "xs = [{w: 2}, {w: null}]\np = xs.product(x -> x.w)\np",
            "null",
        ),
        ("xs = [{w: 2}, {w: null}]\nxs.product(x -> x.w)", "0.0"),
        (
            "limit = null\nxs = [2, 3]\np = xs.product(x -> x + limit)\nlimit = 1\np",
            "12.0",
        ),
        ("xs = [2, null, 3]\np = xs.product(x -> 1)\np", "1.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn the_selector_runs_once_per_item_in_receiver_order_with_only_the_item() {
    for (source, expected) in [
        (
            "seen = []\ndef weigh(item, position):\n    seen.push(position ?? item)\n    return item\n[[3, 1, 2].product(weigh), seen]",
            "[6.0,[3.0,1.0,2.0]]",
        ),
        ("[2, 3].product((x, i) -> i ?? 5)", "25.0"),
        ("big = x -> x + 1\n[1, 2].product(big)", "6.0"),
        ("def big(x) => x + 1\n[1, 2].product(big)", "6.0"),
        ("[1, 2].product(lambda x: x + 1)", "6.0"),
        ("[1, 2].product(x => x + 1)", "6.0"),
        ("[1, 2].product((x) -> x + 1)", "6.0"),
        (
            "factor = 3\ndef scaled(values) => values.product(v -> v * factor)\nscaled([1, 2])",
            "18.0",
        ),
        (
            "seen = []\ndef mark(label, value):\n    seen.push(label)\n    return value\nxs = [3, 2]\n[mark(\"receiver\", xs).product(mark(\"selector\", x -> x), mark(\"extra\", 0)), seen]",
            "[6.0,[\"receiver\",\"selector\",\"extra\"]]",
        ),
        (
            "seen = []\ndef mark() => seen.push(1)\nxs = null\nr = xs.product(mark())\n[r, seen]",
            "[null,[]]",
        ),
        (
            "seen = []\nclass Unit(n):\n    this.n = n\nUnit(2)\nUnit(3)\n[Unit.product(unit -> seen.push(unit.n) > 0 ? unit.n : 0), seen]",
            "[6.0,[2.0,3.0]]",
        ),
        (
            "seen = []\ndef zero(x):\n    seen.push(x)\n    return 0\n[[1, 2, 3].product(zero), seen]",
            "[0.0,[1.0,2.0,3.0]]",
        ),
        ("xs = [2, 3]\nxs.product(x -> x, 5)", "6.0"),
        ("xs = [2, 3]\nxs.product(x -> x, null, \"extra\")", "6.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for source in [
        "xs = [1, 2]\nxs.product(missing)",
        "xs = [1, 2]\nxs.product(5, missing)",
        "xs = [1, 2]\nxs.product(x -> x, missing)",
    ] {
        assert_eq!(
            failure(source),
            (ErrorKind::Reference, "missing is not defined".to_string()),
            "{source}"
        );
    }
}

#[test]
fn a_supplied_selector_that_is_not_a_function_raises_even_on_empty_receivers() {
    for source in [
        "xs = [1, 2]\nxs.product(null)",
        "xs = [1, 2]\nxs.product(5)",
        "xs = [1, 2]\nxs.product(\"k\")",
        "xs = [1, 2]\nxs.product([x -> x])",
        "xs = [1, 2]\nxs.product({})",
        "xs = [1, 2]\nxs.product(true)",
        "[].product(null)",
        "[].product(5)",
        "class Empty\nEmpty.product(1)",
        "class Order(total):\n    this.total = total\nOrder(1)\nOrder.product(\"total\")",
        "class Calculator(base):\n    this.base = base\n    def product(a, b) => this.base + a * b\nc = Calculator(1)\nCalculator.product(2, 3)",
    ] {
        assert_eq!(
            failure(source),
            (ErrorKind::Type, "product expects a function".to_string()),
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    runtime.run("xs = [2, 3]\nys = xs.product(x -> x)").unwrap();
    assert_eq!(
        outcome(&mut runtime, "ys = xs.product(1)"),
        "TypeError: product expects a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(json(&mut runtime, "xs.push(4)\nys"), "24.0");
    assert_eq!(result("[].product()"), "1.0");
    assert_eq!(
        failure("xs = [1]\nxs.sum(5)"),
        (ErrorKind::Type, "sum expects a function".to_string())
    );
}

#[test]
fn lists_however_they_are_reached_are_receivers() {
    for (source, expected) in [
        ("[3, 1, 2].product()", "6.0"),
        ("xs = [3, 1, 2]\nxs.product()", "6.0"),
        (
            "o = {inner: {items: [{n: 2}, {n: 5}]}}\no.inner.items.product(i -> i.n)",
            "10.0",
        ),
        ("xs = [3, 1, 2]\nxs[1:].product()", "2.0"),
        ("xs = [3, 4]\nxs.map(x -> x * 2).product()", "48.0"),
        ("xs = [3, 4, 5]\nxs.filter(x -> x > 3).product()", "20.0"),
        ("xs = [3, 4, 5]\nxs.slice(0, 2).product()", "12.0"),
        ("xs = [3]\nxs.concat([4], 5).product()", "60.0"),
        ("def make() => [2, 3, 4]\nmake().product()", "24.0"),
        ("xs = [1, 2, 3]\nxs.value.product()", "6.0"),
        ("xs = [2]\nxs.push(3)\nxs@first.product()", "2.0"),
        ("grid = [[2, 3], [4]]\ngrid[0].product()", "6.0"),
        (
            "class Box(sides):\n    this.sides = sides\n    def volume():\n        return this.sides.product()\nb = Box([2, 3, 4])\nb.volume()",
            "24.0",
        ),
        (
            "rows = [[2, 3], [4, 5]]\nrows.map(row -> row.product())",
            "[6.0,20.0]",
        ),
        (
            "rows = [[2, 3], [4, 5]]\nrows.map(row -> row.product()).product()",
            "120.0",
        ),
        ("[2, 3].concat([4]).product(x -> x - 1)", "6.0"),
        ("Order = [2, 5]\nOrder.product()", "10.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn populations_by_name_through_a_variable_and_by_dollar_name_are_receivers() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)\nOrder(2)\nkind = Order")
        .unwrap();
    for (source, expected) in [
        ("Order.product(o -> o.total)", "70.0"),
        ("Order[1:].product(o -> o.total)", "14.0"),
        ("Order.list().product(o -> o.total)", "70.0"),
        ("kind.product(o -> o.total)", "70.0"),
        ("$Order.product(o -> o.total)", "70.0"),
        (
            "Order.filter(o -> o.total > 4).product(o -> o.total)",
            "35.0",
        ),
        ("Order.map(o -> o.total).product()", "70.0"),
        ("String(Order.product())", "\"NaN\""),
        ("Order.reduce((t, o) => t * o.total, 1)", "70.0"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "class Bag(n):\n    this.n = n\n    def product(): return 9\na = Bag(2)\nb = Bag(3)\n[Bag.product(bag -> bag.n), a.product(), Bag.product(bag -> bag.product())]",
            "[6.0,9.0,81.0]",
        ),
        (
            "class Calculator(base):\n    this.base = base\n    def product(a, b) => this.base + a * b\nc = Calculator(1)\nd = Calculator(2)\n[c.product(2, 3), Calculator.product(x -> x.base)]",
            "[7.0,2.0]",
        ),
        (
            "class Empty\n[Empty.product(), Empty.product(e -> e.n)]",
            "[1.0,1.0]",
        ),
        (
            "class A:\n    def init(n):\n        this.n = n\nclass B extends A:\n    def init(n):\n        super(n)\nA(2)\nB(7)\nA(3)\n[A.product(a -> a.n), B.product(b -> b.n), A.reduce((t, a) => t * a.n, 1), B.reduce((t, b) => t * b.n, 1)]",
            "[6.0,7.0,6.0,7.0]",
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
            ("factor = 2", "factor = 2"),
            (
                "p = xs.product(x -> x.k * factor)",
                "p = xs.reduce((total, x) => total * (x.k * factor), 1)",
            ),
            ("p", "p"),
            ("xs.push({n: \"c\", k: 4})", "xs.push({n: \"c\", k: 4})"),
            ("p", "p"),
            ("factor = 1", "factor = 1"),
            ("p", "p"),
            ("xs.pop()", "xs.pop()"),
            ("xs.shift()", "xs.shift()"),
            ("p", "p"),
            (
                "xs = [{n: \"d\", k: 0.5}, {n: \"e\", k: 6}]",
                "xs = [{n: \"d\", k: 0.5}, {n: \"e\", k: 6}]",
            ),
            ("xs[0].k = 9", "xs[0].k = 9"),
            ("p", "p"),
            ("xs = []", "xs = []"),
            ("p", "p"),
        ],
        &["p"],
    );
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("sides = [2, 3]\nvolume = sides.product()\nvolume", "6.0"),
        ("sides.push(4)\nvolume", "24.0"),
        ("sides.pop()\nvolume", "6.0"),
        ("sides.shift()\nvolume", "3.0"),
        ("sides = []\nvolume", "1.0"),
        ("sides = [0.5, 0.5]\nvolume", "0.25"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn declarations_follow_new_and_deleted_instances_and_class_level_rules() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class R(n, t):\n    this.n = n\n    this.t = t\nchance = R.product(r -> r.t)\nchance",
            "1.0",
        ),
        ("a = R(\"a\", 0.5)\nb = R(\"b\", 0.25)\nchance", "0.125"),
        ("c = R(\"c\", 0.5)\nchance", "0.0625"),
        ("b.t = 1\nchance", "0.25"),
        ("delete c.t\nchance", "null"),
        ("delete c.n\ndelete c\nchance", "0.5"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class Team(name):\n    this.name = name\nclass Player(team, odds):\n    this.team = team\n    this.odds = odds\n$Team.chance = Player.filter(p -> p.team == $Team.name).product(p -> p.odds)\nreds = Team(\"reds\")\nblues = Team(\"blues\")\nPlayer(\"reds\", 0.5)\nPlayer(\"blues\", 0.25)\ncy = Player(\"reds\", 0.5)\n[reds.chance, blues.chance]",
            "[0.25,0.25]",
        ),
        ("Player(\"blues\", 0.5)\nblues.chance", "0.125"),
        ("cy.odds = 1\nreds.chance", "0.5"),
        (
            "cy.team = \"blues\"\n[reds.chance, blues.chance]",
            "[0.5,0.125]",
        ),
        ("greens = Team(\"greens\")\ngreens.chance", "1.0"),
        (
            "league = Team.product(team -> team.chance)\nleague",
            "0.0625",
        ),
        ("Player(\"greens\", 0.5)\nleague", "0.03125"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "class P(x):\n    this.x = x\n$P.spread = P.product(other -> Math.abs(other.x - $P.x) + 1)\np = P(1)\nq = P(2)\nr = P(4)\n[p.spread, q.spread, r.spread]",
            "[8.0,6.0,12.0]",
        ),
        (
            "class Room(tiles):\n    this.tiles = tiles\n$Room.area = $Room.tiles.product()\nhall = Room([3, 4])\nden = Room([2])\nhall.tiles.push(2)\nden.tiles = [5, 5]\n[hall.area, den.area]",
            "[24.0,25.0]",
        ),
        (
            "class Room(tiles):\n    this.tiles = tiles\n$Room.area = $Room.tiles.product()\nempty = Room([])\nempty.area",
            "1.0",
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
            "sides = [2, 3]\nvolume = sides.product()\n(why volume).map(step -> step.rule)"
        ),
        "[\"volume = sides.product()\",\"sides = [2,3]\"]"
    );
    assert_eq!(json(&mut runtime, "(why volume)[0].from"), "[\"sides\"]");
    assert_eq!(
        json(&mut runtime, "(affects sides).map(step -> step.node)"),
        "[\"volume\"]"
    );
    assert_eq!(dependencies(&runtime, "volume"), ["sides"]);
    assert_eq!(
        json(
            &mut runtime,
            "rates = [0.5]\ngrowth = rates.product(rate -> 1 + rate)\n(why growth).map(step -> step.rule)"
        ),
        "[\"growth = rates.product(function())\",\"rates = [0.5]\"]"
    );
    let mut runtime = Runtime::new();
    runtime
        .run("class Bid(amount):\n    this.amount = amount\nbook = Bid.product(bid -> bid.amount)\nBid(5)")
        .unwrap();
    assert_eq!(dependencies(&runtime, "book"), ["$Bid", "obj:0.amount"]);
    let mut spelled = Runtime::new();
    assert_eq!(
        json(
            &mut spelled,
            "sides = [2, 3]\nvolume = sides.reduce((total, side) => total * side, 1)\n(why volume).map(step -> step.rule)"
        ),
        "[\"volume = sides.reduce(function(),1)\",\"sides = [2,3]\"]"
    );
    assert_eq!(dependencies(&spelled, "volume"), ["sides"]);
    assert_eq!(
        result("def key(x) => x.k\nr = [].product(key)\n(affects key).map(step -> step.node)"),
        "[\"r\"]"
    );
    for (source, message) in [
        (
            "xs = [1, 2]\nxs.product()(1)",
            "xs.product() is not a function",
        ),
        (
            "xs = [1, 2]\nall x in xs.product(): x",
            "xs.product() is not a collection",
        ),
        (
            "xs = [1, 2]\nwhy xs.product()",
            "Cannot explain xs.product()",
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
fn snapshots_hypotheticals_templates_and_conditions_read_products() {
    for (source, expected) in [
        (
            "xs = [2, 3]\np = xs.product()\nfrozen = p.value\nxs.push(4)\n[p, frozen]",
            "[24.0,6.0]",
        ),
        (
            "xs = [2, 3]\nfrozen = xs.value.product()\nxs.push(4)\nfrozen",
            "6.0",
        ),
        (
            "xs = [2, 3]\np = xs.product()\nseen = suppose xs.push(4): p\n[seen, p]",
            "[24.0,6.0]",
        ),
        (
            "xs = [2, 3]\nlabel = `${xs.product()} ways`\nxs.push(4)\nlabel",
            "\"24 ways\"",
        ),
        (
            "xs = [2, 3]\nif xs.product() > 5:\n    flag = true\nflag",
            "true",
        ),
        (
            "xs = [2, 3]\nsize = xs.product() > 10 ? \"big\" : \"small\"\nxs.push(4)\nsize",
            "\"big\"",
        ),
        (
            "xs = [2, 3]\nonce := xs.product()\nxs.push(4)\n[once, xs.product()]",
            "[6.0,24.0]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn failed_statements_and_transactions_restore_products() {
    behaves_as_spelled(
        &[
            ("rows = [{k: 2}, {k: 3}]", "rows = [{k: 2}, {k: 3}]"),
            (
                "p = rows.product(row -> row.k > 9 ? row.k.toUpperCase() : row.k)",
                "p = rows.reduce((total, row) => total * (row.k > 9 ? row.k.toUpperCase() : row.k), 1)",
            ),
            (
                "try:\n    rows.push({k: 5})\n    assert(p, 30)\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
                "try:\n    rows.push({k: 5})\n    assert(p, 30)\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
            ),
            ("[rows.length, p]", "[rows.length, p]"),
            ("rows.push({k: 10})", "rows.push({k: 10})"),
            ("[rows.length, p]", "[rows.length, p]"),
            ("rows.push({k: 4})", "rows.push({k: 4})"),
            ("p", "p"),
        ],
        &["p"],
    );
    let mut runtime = Runtime::new();
    runtime
        .run("rows = [{k: 2}, {k: 3}]\np = rows.product(row -> row.k > 9 ? row.k.toUpperCase() : row.k)")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "rows.push({k: 10})"),
        "TypeError: number.toUpperCase is not a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(json(&mut runtime, "[rows.length, p]"), "[2.0,6.0]");
    assert_eq!(
        result(
            "class Probe(t):\n    this.t = t\nProbe(2)\nProbe(3)\nheat = Probe.product(p -> p.t > 9 ? p.t.toUpperCase() : p.t)\nfailure = \"\"\ntry:\n    Probe(50)\ncatch error:\n    failure = error\n[failure, Probe.length, heat]"
        ),
        "[\"TypeError: number.toUpperCase is not a function\",2.0,6.0]"
    );
    assert_eq!(
        result(
            "xs = [2, 3]\np = xs.product()\ntry:\n    xs.push(4)\n    throw \"UNDO\"\ncatch error:\n    pass\np"
        ),
        "6.0"
    );
}

#[test]
fn large_receivers_multiply_through_the_public_api() {
    let mut runtime = Runtime::new();
    runtime
        .run("holder = {items: 0}\ncounter = {n: 0}\ndef tick(x):\n    counter.n = counter.n + 1\n    return x")
        .unwrap();
    let count = 100_000_u32;
    hold(
        &mut runtime,
        Value::List(
            (0..count)
                .map(|position| Value::Number(if position % 10_000 == 0 { 2.0 } else { 1.0 }))
                .collect(),
        ),
    );
    assert_eq!(json(&mut runtime, "holder.items.product()"), "1024.0");
    assert_eq!(
        json(&mut runtime, "holder.items.reduce((t, x) => t * x, 1)"),
        "1024.0"
    );
    assert_eq!(
        json(&mut runtime, "[holder.items.product(tick), counter.n]"),
        "[1024.0,100000.0]"
    );
    assert_eq!(json(&mut runtime, "holder.items.length"), "100000.0");
}

#[test]
fn sums_keep_their_results_beside_products() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("[].sum()", 0.0),
        ("[].product()", 1.0),
        ("[0.1, 0.2, 0.3].sum()", 0.6000000000000001),
        ("[0.1, 0.2, 0.3].product()", 0.006000000000000001),
        ("[2, \"3\", true].sum()", 6.0),
        ("[2, \"3\", true].product()", 6.0),
        ("[1, 2].average()", 1.5),
        ("[4, 1, 3, 2].median()", 2.5),
    ] {
        assert!(
            same_number(number(&mut runtime, source), expected),
            "{source}"
        );
    }
    assert_eq!(number(&mut runtime, "1 / [-0].sum()"), f64::INFINITY);
    assert_eq!(
        number(&mut runtime, "1 / [-0].product()"),
        f64::NEG_INFINITY
    );
    for (source, expected) in [
        ("o = {}\nString([1, o.missing].sum())", "\"undefined\""),
        ("t = [1, null].sum()\nt", "1.0"),
        ("t = [1, null].sum(x -> x)\nt", "null"),
        ("xs = [1, 2]\ntotal = xs.sum()\nxs.push(3)\ntotal", "6.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn the_compatibility_rows_give_their_documented_results() {
    let orders = "class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)\n";
    let calculator = "class Calculator(base):\n    this.base = base\n    def product(a, b) => this.base + a * b\nc = Calculator(1)\n";
    let bag = "class Bag:\n    def product(): return 9\nbag = Bag()\n";
    for (source, expected) in [
        ("[2, 3, 4].product()".to_string(), "24.0"),
        ("[].product()".to_string(), "1.0"),
        ("xs = [2, 3]\nxs.product(x -> x * 2)".to_string(), "24.0"),
        ("o = {items: [2, 3]}\no.items.product()".to_string(), "6.0"),
        ("xs = [3, 4]\nxs.map(x -> x * 2).product()".to_string(), "48.0"),
        ("xs = [2, 3]\nxs.value.product()".to_string(), "6.0"),
        ("xs = [2]\nxs.push(3)\nxs@first.product()".to_string(), "2.0"),
        ("xs = [2, 3, 4]\nxs[1:].product()".to_string(), "12.0"),
        ("xs = [2, 3, 4]\nxs.slice(0, 2).product()".to_string(), "6.0"),
        ("xs = [2]\nxs.concat([3]).product()".to_string(), "6.0"),
        ("grid = [[2, 3], [4]]\ngrid[0].product()".to_string(), "6.0"),
        ("def make() => [2, 3, 4]\nmake().product()".to_string(), "24.0"),
        (format!("{orders}Order.product(o -> o.total)"), "35.0"),
        (format!("{orders}Order[1:].product(o -> o.total)"), "7.0"),
        (format!("{orders}Order.list().product(o -> o.total)"), "35.0"),
        (format!("{orders}kind = Order\nkind.product(o -> o.total)"), "35.0"),
        (format!("{orders}$Order.product(o -> o.total)"), "35.0"),
        ("class Empty\nEmpty.product()".to_string(), "1.0"),
        (
            "class Box(sides):\n    this.sides = sides\n$Box.volume = $Box.sides.product()\nb = Box([2, 3, 4])\nb.volume".to_string(),
            "24.0",
        ),
        (
            format!("{calculator}Calculator.product(2, 3)"),
            "TypeError: product expects a function at Some(Position { line: 5, column: 1 })",
        ),
        (format!("{bag}String(Bag.product())"), "\"NaN\""),
        (format!("{bag}Bag.product(b -> b.product())"), "9.0"),
        ("[1.1, 2.2, 3.3].product()".to_string(), "7.986000000000001"),
        ("[2, \"3\", true, null].product()".to_string(), "0.0"),
        ("[[2], [3]].product()".to_string(), "6.0"),
        ("[-1, 0].product()".to_string(), "-0.0"),
        ("xs = [1e308, 10]\nString(xs.product())".to_string(), "\"Infinity\""),
        ("o = {}\nString([2, o.missing].product())".to_string(), "\"undefined\""),
        ("xs = [2, 3]\nxs.product(x -> x, 5)".to_string(), "6.0"),
        (
            "xs = [2, 3]\nxs.product(5)".to_string(),
            "TypeError: product expects a function at Some(Position { line: 2, column: 1 })",
        ),
        (
            "[].product(null)".to_string(),
            "TypeError: product expects a function at Some(Position { line: 1, column: 1 })",
        ),
        (
            format!("{orders}Order.product(\"total\")"),
            "TypeError: product expects a function at Some(Position { line: 5, column: 1 })",
        ),
        (
            "xs = [1, 2]\nxs.product()(1)".to_string(),
            "TypeError: xs.product() is not a function at Some(Position { line: 2, column: 1 })",
        ),
        (
            "def scaled(xs, k) => xs.product(x -> x * k)\nscaled([1, 2], 3)".to_string(),
            "ReferenceError: k is not defined at Some(Position { line: 2, column: 1 })",
        ),
        (
            "def scaled(xs, k) => xs.sum(x -> x * k)\nscaled([1, 2], 3)".to_string(),
            "ReferenceError: k is not defined at Some(Position { line: 2, column: 1 })",
        ),
    ] {
        assert_eq!(outcome(&mut Runtime::new(), &source), expected, "{source}");
    }
}

#[test]
fn unchanged_readings_keep_their_accepted_results() {
    for (source, expected) in [
        ("xs = [1]\nxs.product", "null"),
        ("class Order\nOrder.product", "null"),
        ("xs = [1]\ntypeof xs.product", "\"undefined\""),
        ("o = {product: 3}\no.product", "3.0"),
        ("x = {product: 4}.product\nx", "4.0"),
        ("o = {product: n -> n * 10}\no.product(4)", "40.0"),
        ("o = {}\no.product()", "null"),
        ("class P\np = P()\np.product()", "null"),
        (
            "class Calculator(base):\n    this.base = base\n    def product(a, b) => this.base + a * b\nc = Calculator(1)\nc.product(2, 3)",
            "7.0",
        ),
        (
            "product = 3\ndef scale product => product * 2\n[product, scale(product)]",
            "[3.0,6.0]",
        ),
        ("def product(xs) => xs.length\nproduct([1, 2])", "2.0"),
        ("f = lambda product: product + 1\nf(4)", "5.0"),
        (
            "class T(n):\n    this.n = n\nT(1)\nT(2)\nseen = []\nfor product of T:\n    seen.push(product.n)\nseen",
            "[1.0,2.0]",
        ),
        (
            "class S(product):\n    this.product = product\ns = S([2, 3])\ns.product",
            "[2.0,3.0]",
        ),
        ("xs = null\nxs.product(i -> i)", "null"),
        ("o = {}\no.items.product()", "null"),
        ("[2, 3].sum()", "5.0"),
        ("[2, 3].reduce((t, x) => t * x, 1)", "6.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, kind, message) in [
        (
            "\"abc\".product()",
            ErrorKind::Type,
            "abc.product is not a function",
        ),
        (
            "(5).product()",
            ErrorKind::Type,
            "number.product is not a function",
        ),
        (
            "true.product()",
            ErrorKind::Type,
            "boolean.product is not a function",
        ),
        (
            "d = new Date(5)\nd.product()",
            ErrorKind::Type,
            "Date.product is not a function",
        ),
        (
            "f = x => x\nf.product()",
            ErrorKind::Type,
            "function.product is not a function",
        ),
        (
            "/a/.product()",
            ErrorKind::Type,
            "product is not a function",
        ),
        (
            "Item = 5\nItem.product()",
            ErrorKind::Type,
            "number.product is not a function",
        ),
        (
            "Math.product([1])",
            ErrorKind::Type,
            "Math.product is not a function",
        ),
        (
            "Number.product(1)",
            ErrorKind::Type,
            "Number.product is not a function",
        ),
        (
            "String.product()",
            ErrorKind::Type,
            "String.product is not a function",
        ),
        (
            "Date.product()",
            ErrorKind::Type,
            "Date.product is not a function",
        ),
        (
            "List.product()",
            ErrorKind::Reference,
            "List is not defined",
        ),
        (
            "Object.product()",
            ErrorKind::Reference,
            "Object is not defined",
        ),
        (
            "product([1, 2])",
            ErrorKind::Reference,
            "product is not defined",
        ),
        (
            "xs = [2, 3]\nproduct of xs",
            ErrorKind::Syntax,
            "Unexpected keyword 'of'",
        ),
        (
            "xs = [1]\nxs.products()",
            ErrorKind::Type,
            "xs.products is not a function",
        ),
    ] {
        assert_eq!(failure(source), (kind, message.to_string()), "{source}");
    }
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    assert_eq!(
        result("xs = [2, 3]\nr = 0\ntry:\n    r = xs.product()\ncatch error:\n    r = error\nr"),
        "6.0"
    );
    assert_eq!(
        result("xs = [2, 3]\nr = 0\ntry:\n    r = xs.products()\ncatch error:\n    r = error\nr"),
        "\"TypeError: xs.products is not a function\""
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-list-products-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_product() {
    let program = "sides = [2, 3, 4]\nvolume = sides.product()\nsides.push(0.5)\nvolume";
    let file = SourceFile::new("file");
    let multiplied = file.run(program, &["--json"]);
    assert!(multiplied.status.success(), "{multiplied:?}");
    assert!(multiplied.stderr.is_empty(), "{multiplied:?}");
    assert_eq!(String::from_utf8(multiplied.stdout).unwrap(), "12.0\n");
    let plain = file.run(program, &[]);
    assert!(plain.status.success(), "{plain:?}");
    assert_eq!(String::from_utf8(plain.stdout).unwrap(), "12\n");
    let graph = String::from_utf8(file.run(program, &["--json", "--graph"]).stdout).unwrap();
    assert!(
        graph.contains(
            "  sides [variable]\n    updates  volume\n  volume [variable]\n    reads    sides"
        ),
        "{graph}"
    );
    let block = String::from_utf8(
        file.run(
            "xs = [2, 3]\nif xs.product() > 5:\n    flag = true\nflag",
            &["--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        block.contains("  xs [variable]\n    updates  if(xs.product()>5)\n  if(xs.product()>5) [if]\n    reads    xs"),
        "{block}"
    );
    let failed = file.run("xs = [1]\nxs.product(5)", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "TypeError: product expects a function (line 2, column 1)\n"
    );
    let session = "xs = [2, 3]\np = xs.product()\np\nxs.push(4)\np\nxs\nxs.product(5)\n.clear\nxs = [5]\nxs.product(x -> x + 1)\n.exit\n";
    for (options, expected) in [
        (
            &["--plain"][..],
            "2,3\n6\n6\n3\n24\n2,3,4\nState cleared.\n5\n6\n",
        ),
        (
            &["--plain", "--json"][..],
            "[2.0,3.0]\n6.0\n6.0\n3.0\n24.0\n[2.0,3.0,4.0]\nState cleared.\n[5.0]\n6.0\n",
        ),
        (
            &["--json"][..],
            "[2.0,3.0]\n6.0\n6.0\n3.0\n24.0\n[2.0,3.0,4.0]\nState cleared.\n[5.0]\n6.0\n",
        ),
    ] {
        for newline in ["\n", "\r\n"] {
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.ends_with(expected), "{options:?} {stdout}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                "TypeError: product expects a function (line 1, column 1)\n"
            );
        }
    }
}
