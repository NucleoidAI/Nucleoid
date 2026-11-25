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
    "Nucleoid measures the spread of lists with variance and stdev",
    "Nucleoid measures the spread of class instance populations with variance and stdev",
    "Nucleoid computes the variances of large values from their deviations",
    "Nucleoid rolls back failed variance updates",
];
const REFERENCE_TITLE: &str =
    "Nucleoid measures the spread of lists and populations with variance and stdev";
const README_TITLE: &str =
    "Standard deviations keep the scatter of delivery times live as orders change";
const EXAMPLES_TITLE: &str = "Standard deviations flag the machines whose readings drift";

const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid measures the spread of lists with variance and stdev",
    "source": "# Nucleoid measures the spread of lists with variance and stdev\n\nscores = [2, 4, 4, 4, 5, 5, 7, 9]\nspread = scores.average(x -> (x - scores.average()) ** 2)\ndeviation = spread ** 0.5\nassert([spread, deviation], [4, 2])\nscores.push(11)\nassert([spread, deviation], [7.111111111111111, 2.6666666666666665])\nscores = [6]\nassert([spread, deviation], [0, 0])\nteam = {laps: [61, 58, 64]}\nwobble = (team.laps.average(** 2) - team.laps.average() ** 2) ** 0.5\nassert(wobble, 2.449489742783178)\nteam.laps.push(57)\nassert(wobble, 2.7386127875258306)",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid measures the spread of class instance populations with variance and stdev",
    "source": "# Nucleoid measures the spread of class instance populations with variance and stdev\n\nclass Reading(level):\n    this.level = level\nReading(3)\nReading(5)\nspike = Reading(10)\ngain = 2\nspread = Reading.average(x -> (x.level - Reading.average(.level)) ** 2)\nscaled = [x.level * gain for x in Reading[-3:]]\nrecent = scaled.average(x -> (x - scaled.average()) ** 2) ** 0.5\nassert([spread, recent], [8.666666666666666, 5.887840577551898])\nReading(6)\nassert([spread, recent], [6.5, 4.320493798938574])\nspike.level = 4\ngain = 3\nassert([spread, recent], [1.25, 2.449489742783178])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid computes the variances of large values from their deviations",
    "source": "# Nucleoid computes the variances of large values from their deviations\n\nstamps = [1000000001, 1000000002, 1000000003]\njitter = stamps.average(x -> (x - stamps.average()) ** 2)\nwobble = jitter ** 0.5\nassert([jitter, wobble], [0.6666666666666666, 0.816496580927726])\nstamps.push(1000000004)\nassert([jitter, wobble], [1.25, 1.118033988749895])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back failed variance updates",
    "source": "# Nucleoid rolls back failed variance updates\n\ndef checked(level):\n    if level < 0:\n        throw \"NEGATIVE_LEVEL\"\n    return level\nlevels = [3, 5, 10]\nspread = levels.average(x -> (checked(x) - levels.average(checked)) ** 2)\nfailure = \"\"\ntry:\n    levels.push(-1)\ncatch error:\n    failure = error\nassert([failure, levels, spread], [\"NEGATIVE_LEVEL\", [3, 5, 10], 8.666666666666666])\ntry:\n    levels.push(6)\n    assert(spread, 6.5)\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([levels, spread], [[3, 5, 10], 8.666666666666666])\nclass Gauge(level): this.level = level\nGauge(4)\nGauge(8)\nwobble = Gauge.average(x -> (checked(x.level) - Gauge.map(.level).average(checked)) ** 2) ** 0.5\nfailure = \"\"\ntry:\n    Gauge(-2)\ncatch error:\n    failure = error\nassert([failure, Gauge.length, wobble], [\"NEGATIVE_LEVEL\", 2, 2])\nGauge(12)\nwobble",
    "expected": "3.265986323710904"
  },
  {
    "document": "reference",
    "title": "Nucleoid measures the spread of lists and populations with variance and stdev",
    "source": "# Nucleoid measures the spread of lists and populations with variance and stdev\n\ntemps = [18, 21, 24]\nswing = (temps.average(** 2) - temps.average() ** 2) ** 0.5\nclass Shot(score):\n    this.score = score\nShot(7)\nShot(9)\nspread = Shot.average(x -> (x.score - Shot.average(.score)) ** 2)\nassert([swing, spread], [2.449489742783178, 1])\ntemps.push(17)\nShot(5)\nassert([swing, spread], [2.7386127875258306, 2.6666666666666665])\nspread",
    "expected": "2.6666666666666665"
  },
  {
    "document": "readme",
    "title": "Standard deviations keep the scatter of delivery times live as orders change",
    "source": "class Delivery(minutes):\n    this.minutes = minutes\nDelivery(30)\nDelivery(25)\nlate = Delivery(35)\nscatter = Delivery.average(x -> (x.minutes - Delivery.average(.minutes)) ** 2) ** 0.5\nassert(scatter, 4.08248290463863)\nlate.minutes = 90\nDelivery(40)\nassert(scatter, 25.829972899714782)\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Standard deviations flag the machines whose readings drift",
    "source": "class Machine(name, readings):\n    this.name = name\n    this.readings = readings\n$Machine.drift = .readings.average(x -> (x - .readings.average()) ** 2) ** 0.5\n$Machine.unstable = $Machine.drift > 2\npress = Machine(\"press\", [50, 54, 46])\nlathe = Machine(\"lathe\", [30, 31, 29])\nassert([press.drift, lathe.unstable], [3.265986323710904, false])\nlathe.readings.push(35)\npress.readings.push(50)\n[lathe.unstable, press.drift]\n",
    "expected": "[true,2.8284271247461903]"
  }
]"###;

const WRITTEN_OUT: [(&str, &str); 13] = [
    (
        "scores.variance()",
        "scores.average(x -> (x - scores.average()) ** 2)",
    ),
    ("deviation = scores.stdev()", "deviation = spread ** 0.5"),
    (
        "team.laps.stdev()",
        "(team.laps.average(** 2) - team.laps.average() ** 2) ** 0.5",
    ),
    (
        "Reading.variance(reading -> reading.level)",
        "Reading.average(x -> (x.level - Reading.average(.level)) ** 2)",
    ),
    (
        "recent = Reading[-3:].stdev(reading -> reading.level * gain)",
        "scaled = [x.level * gain for x in Reading[-3:]]\nrecent = scaled.average(x -> (x - scaled.average()) ** 2) ** 0.5",
    ),
    (
        "stamps.variance()",
        "stamps.average(x -> (x - stamps.average()) ** 2)",
    ),
    ("stamps.stdev()", "jitter ** 0.5"),
    (
        "levels.variance(checked)",
        "levels.average(x -> (checked(x) - levels.average(checked)) ** 2)",
    ),
    (
        "Gauge.stdev(gauge -> checked(gauge.level))",
        "Gauge.average(x -> (checked(x.level) - Gauge.map(.level).average(checked)) ** 2) ** 0.5",
    ),
    (
        "temps.stdev()",
        "(temps.average(** 2) - temps.average() ** 2) ** 0.5",
    ),
    (
        "Shot.variance(shot -> shot.score)",
        "Shot.average(x -> (x.score - Shot.average(.score)) ** 2)",
    ),
    (
        "Delivery.stdev(delivery -> delivery.minutes)",
        "Delivery.average(x -> (x.minutes - Delivery.average(.minutes)) ** 2) ** 0.5",
    ),
    (
        "$Machine.readings.stdev()",
        ".readings.average(x -> (x - .readings.average()) ** 2) ** 0.5",
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

fn query_calls(source: &str) -> Vec<String> {
    let tokens: Vec<Token> = tokenize(source)
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    tokens
        .windows(3)
        .filter_map(|window| match window {
            [Token::Dot, Token::Identifier(name), Token::LeftParen]
                if name == "variance" || name == "stdev" =>
            {
                Some(name.clone())
            }
            _ => None,
        })
        .collect()
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
            title: "variance regression".to_string(),
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

fn reference_variance(values: &[f64]) -> f64 {
    let count = values.len() as f64;
    let mut total = 0.0;
    for value in values {
        total += value;
    }
    let mean = total / count;
    let mut squares = 0.0;
    for value in values {
        let deviation = value - mean;
        squares += deviation * deviation;
    }
    squares / count
}

fn spelled_variance(list: &str) -> String {
    format!("{list}.average(x -> (x - {list}.average()) * (x - {list}.average()))")
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
fn only_the_appended_cases_measure_spread_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    let mut calls = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        let found = query_calls(&case.source);
        assert_eq!(!found.is_empty(), new, "{}", case.title);
        calls += found.len();
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
    assert_eq!(calls, 13);
    assert_eq!(assertions, 20);
    assert_eq!(cases, 1316);
}

#[test]
fn the_appended_cases_stand_after_nuc_122_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[803],
        "Nucleoid reads except clauses in methods, functions and nested try statements"
    );
    assert_eq!(titles[804..808], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid reads Python except clauses as catch clauses",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Except clauses fall back to another sensor when a reading fails",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Except clauses refuse an overdrawn transfer and keep both wallets whole",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 16 - 7 * usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 8].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### List and population variances and standard deviations\n\n```\n")
        .expect("the language reference documents variances");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 123](nuc-0123.md)"));
    assert!(reference.contains("`variance expects a function` or `stdev expects a function`"));
    assert_eq!(case.expected.as_deref(), Some("2.6666666666666665"));
}

#[test]
fn every_case_is_its_accepted_counterpart_with_only_the_queries_written_out() {
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
        assert!(query_calls(&written).is_empty(), "{}", sealed.title);
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
    assert_eq!(rewrites, 13);
}

#[test]
fn variances_equal_the_two_pass_reference_bit_for_bit() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let special: Vec<Vec<(String, f64)>> = [
        vec![],
        vec![("-0".to_string(), -0.0)],
        vec![("-0".to_string(), -0.0), ("-0".to_string(), -0.0)],
        vec![("0".to_string(), 0.0), ("-0".to_string(), -0.0)],
        vec![("5".to_string(), 5.0)],
        vec![("1e308".to_string(), 1e308), ("1e308".to_string(), 1e308)],
        vec![("1e200".to_string(), 1e200), ("-1e200".to_string(), -1e200)],
        vec![("1e154".to_string(), 1e154), ("-1e154".to_string(), -1e154)],
        vec![
            ("1e-200".to_string(), 1e-200),
            ("3e-200".to_string(), 3e-200),
        ],
        vec![("5e-324".to_string(), 5e-324), ("0".to_string(), 0.0)],
        vec![
            ("1000000001".to_string(), 1_000_000_001.0),
            ("1000000002".to_string(), 1_000_000_002.0),
            ("1000000003".to_string(), 1_000_000_003.0),
        ],
        vec![
            ("0.1".to_string(), 0.1),
            ("0.2".to_string(), 0.2),
            ("0.3".to_string(), 0.3),
        ],
        vec![
            ("1.5".to_string(), 1.5),
            ("2.25".to_string(), 2.25),
            ("3.125".to_string(), 3.125),
        ],
        vec![
            ("9007199254740993".to_string(), 9_007_199_254_740_992.0),
            ("3".to_string(), 3.0),
        ],
        vec![
            ("-981.1196981493752".to_string(), -981.1196981493752),
            ("981.1196981493752".to_string(), 981.1196981493752),
        ],
        vec![
            ("0".to_string(), 0.0),
            ("0".to_string(), 0.0),
            ("887.754".to_string(), 887.754),
        ],
        vec![("0 / 0".to_string(), f64::NAN), ("1".to_string(), 1.0)],
        vec![
            ("Number.POSITIVE_INFINITY".to_string(), f64::INFINITY),
            ("1".to_string(), 1.0),
        ],
        vec![
            ("Number.POSITIVE_INFINITY".to_string(), f64::INFINITY),
            ("Number.NEGATIVE_INFINITY".to_string(), f64::NEG_INFINITY),
        ],
    ]
    .into();
    let mut lists = special;
    for round in 0..400 {
        let length = (next() % 12) as usize;
        let offset = if round % 4 == 0 { 1e9 } else { 0.0 };
        let items: Vec<(String, f64)> = (0..length)
            .map(|_| {
                let raw = (next() % 2_000_001) as f64 - 1_000_000.0;
                let scale = [1.0, 3.0, 10.0, 1000.0, 1_000_000.0][(next() % 5) as usize];
                let value = offset + raw / scale;
                (format!("{value:?}"), value)
            })
            .collect();
        lists.push(items);
    }
    let mut runtime = Runtime::new();
    for items in &lists {
        let text = format!(
            "[{}]",
            items
                .iter()
                .map(|(text, _)| text.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        let values: Vec<f64> = items.iter().map(|(_, value)| *value).collect();
        let expected = reference_variance(&values);
        runtime.run(&format!("xs = {text}")).unwrap();
        let variance = number(&mut runtime, "xs.variance()");
        assert!(
            same_number(variance, expected),
            "{text}: {variance:?} and {expected:?}"
        );
        let deviation = number(&mut runtime, "xs.stdev()");
        assert!(
            same_number(deviation, expected.sqrt()),
            "{text}: {deviation:?}"
        );
        let spelled = number(&mut runtime, &spelled_variance("xs"));
        assert!(same_number(variance, spelled), "{text}: {spelled:?}");
        let rooted = number(
            &mut runtime,
            &format!("Math.sqrt({})", spelled_variance("xs")),
        );
        assert!(same_number(deviation, rooted), "{text}: {rooted:?}");
        let tripled: Vec<f64> = values.iter().map(|value| value * 3.0).collect();
        let selected = number(&mut runtime, "xs.variance(item -> item * 3)");
        assert!(
            same_number(selected, reference_variance(&tripled)),
            "{text} with a selector"
        );
        let selected_deviation = number(&mut runtime, "xs.stdev(item -> item * 3)");
        assert!(
            same_number(selected_deviation, reference_variance(&tripled).sqrt()),
            "{text} with a selector"
        );
    }
    assert!(lists.len() > 400);
    assert!(same_number(
        number(&mut runtime, "[2, 4, 4, 4, 5, 5, 7, 9].variance()"),
        4.0
    ));
    assert!(same_number(
        number(&mut runtime, "[2, 4, 4, 4, 5, 5, 7, 9].stdev()"),
        2.0
    ));
    assert!(same_number(
        number(
            &mut runtime,
            "[1000000001, 1000000002, 1000000003].variance()"
        ),
        0.6666666666666666
    ));
    assert!(same_number(
        number(
            &mut runtime,
            "[1000000001, 1000000002, 1000000003].average(x -> x * x) - [1000000001, 1000000002, 1000000003].average() ** 2"
        ),
        0.0
    ));
    assert!(same_number(
        number(&mut runtime, "[3, 5, 10].variance()"),
        8.666666666666666
    ));
    assert!(same_number(
        number(
            &mut runtime,
            "[3, 5, 10].average(x -> x * x) - [3, 5, 10].average() ** 2"
        ),
        8.666666666666664
    ));
}

#[test]
fn variances_convert_values_with_the_conversion_of_sum() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Order\norder = Order()\nholder = {}\nwhen = Date(5)\nf = x -> x")
        .unwrap();
    for value in VALUES {
        for (query, spelled) in [
            (
                format!("[{value}].variance()"),
                spelled_variance(&format!("[{value}]")),
            ),
            (
                format!("[2, {value}].variance()"),
                spelled_variance(&format!("[2, {value}]")),
            ),
            (
                format!("[{value}, 2].variance()"),
                spelled_variance(&format!("[{value}, 2]")),
            ),
            (
                format!("[{value}, 2, 7].stdev()"),
                format!(
                    "Math.sqrt({})",
                    spelled_variance(&format!("[{value}, 2, 7]"))
                ),
            ),
            (
                format!("[{value}, 1].variance(x -> x)"),
                spelled_variance(&format!("[{value}, 1]")),
            ),
            (
                format!("[1, {value}].variance()"),
                format!("[1, {value}].map(x -> x * 1).variance()"),
            ),
        ] {
            assert_eq!(
                reading(&mut runtime, &query),
                reading(&mut runtime, &spelled),
                "{query}"
            );
        }
    }
    for (source, expected) in [
        ("[2, \"4\", true, null].variance()", "2.1875"),
        ("[2, \" 4 \", true].variance()", "1.5555555555555554"),
        ("[[2], [4]].stdev()", "1.0"),
        ("[2, [\"4\"]].variance()", "1.0"),
        ("[3, \"\", 5].variance()", "4.222222222222222"),
        ("[2, false].variance()", "1.0"),
        ("d = Date(4)\n[d, 2].variance()", "1.0"),
        ("String([2, \"x\"].variance())", "\"NaN\""),
        ("String([2, {}].stdev())", "\"NaN\""),
        ("String([2, [1, 2]].variance())", "\"NaN\""),
        ("String([2, x -> x].variance())", "\"NaN\""),
        ("[1.5, 2.25, 3.125].variance()", "0.44097222222222215"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn an_undefined_value_makes_both_results_undefined_after_visiting_every_item() {
    for (source, expected) in [
        ("o = {}\n[2, o.missing, 3].variance()", "null"),
        (
            "o = {}\nString([2, o.missing, 3].variance())",
            "\"undefined\"",
        ),
        ("o = {}\nString([2, o.missing].stdev())", "\"undefined\""),
        (
            "o = {}\nString([0 / 0, o.missing].variance())",
            "\"undefined\"",
        ),
        (
            "o = {}\nString([o.missing, 0 / 0].stdev())",
            "\"undefined\"",
        ),
        (
            "o = {}\nString([Number.POSITIVE_INFINITY, o.missing].variance())",
            "\"undefined\"",
        ),
        ("String([{w: 2}, {}].variance(x -> x.w))", "\"undefined\""),
        (
            "o = {}\nseen = []\ndef pick(x):\n    seen.push(x)\n    return x == 3 ? o.missing : x\n[String([2, 3, 4].stdev(pick)), seen]",
            "[\"undefined\",[2.0,3.0,4.0]]",
        ),
        ("o = {}\ntypeof [o.missing].variance()", "\"undefined\""),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("xs = [{w: 2}, {}]\nv = xs.variance(x -> x.w)\nv", "null"),
        ("xs[1].w = 6\nv", "4.0"),
        ("delete xs[1].w\nv", "null"),
        ("xs.pop()\nv", "0.0"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn null_items_count_zero_while_reads_inside_a_selector_settle() {
    for (source, expected) in [
        ("p = [2, null].variance()\np", "1.0"),
        ("p = [2, null].stdev()\np", "1.0"),
        (
            "xs = [2, null]\np = xs.average(x -> (x - xs.average()) * (x - xs.average()))\np",
            "null",
        ),
        ("[2, null].variance()", "1.0"),
        (
            "xs = [2, null]\nxs.average(x -> (x - xs.average()) * (x - xs.average()))",
            "1.0",
        ),
        (
            "xs = [{w: 2}, {w: null}]\np = xs.variance(x -> x.w)\np",
            "null",
        ),
        ("xs = [{w: 2}, {w: null}]\nxs.variance(x -> x.w)", "1.0"),
        (
            "limit = null\nxs = [2, 4]\np = xs.variance(x -> x + limit)\nlimit = 1\np",
            "1.0",
        ),
        ("xs = [2, null, 3]\np = xs.variance(x -> 1)\np", "0.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn single_values_give_zero_and_empty_receivers_give_nan() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("[5].variance()", 0.0),
        ("[5].stdev()", 0.0),
        ("[-7.25].variance()", 0.0),
        ("[1e300].stdev()", 0.0),
        ("1 / [-0].variance()", f64::INFINITY),
        ("1 / [-0, -0].stdev()", f64::INFINITY),
        ("1 / [3, 3].variance()", f64::INFINITY),
    ] {
        assert!(
            same_number(number(&mut runtime, source), expected),
            "{source}"
        );
    }
    for source in [
        "[].variance()",
        "[].stdev()",
        "[1, 2].filter(x -> x > 5).variance()",
        "[].variance(x -> x * 2)",
    ] {
        assert!(number(&mut runtime, source).is_nan(), "{source}");
    }
    for (source, expected) in [
        ("typeof [].variance()", "\"number\""),
        ("String([].stdev())", "\"NaN\""),
        ("class Empty\nString(Empty.variance())", "\"NaN\""),
        (
            "class Empty\nString(Empty.stdev(empty -> empty.size))",
            "\"NaN\"",
        ),
        (
            "class Order(total):\n    this.total = total\nOrder(4)\nString(Order.filter(o -> o.total > 9).variance(o -> o.total))",
            "\"NaN\"",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class Lap(seconds):\n    this.seconds = seconds\nspread = Lap.variance(lap -> lap.seconds)\nString(spread)",
            "\"NaN\"",
        ),
        ("Lap(61)\nspread", "0.0"),
        ("Lap(63)\nspread", "1.0"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn infinities_and_overflow_follow_the_two_pass_computation() {
    for (source, expected) in [
        (
            "String([Number.POSITIVE_INFINITY, 1].variance())",
            "\"NaN\"",
        ),
        ("String([Number.POSITIVE_INFINITY].variance())", "\"NaN\""),
        ("String([1, Number.NEGATIVE_INFINITY].stdev())", "\"NaN\""),
        (
            "String([Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY].variance())",
            "\"NaN\"",
        ),
        ("String([0 / 0, 1].stdev())", "\"NaN\""),
        ("String([1, \"a\"].stdev())", "\"NaN\""),
        ("xs = [1e200, -1e200]\nString(xs.stdev())", "\"Infinity\""),
        ("String([1e308, 1e308].variance())", "\"Infinity\""),
        ("String([1e154, -1e154].variance())", "\"Infinity\""),
        ("[1e-200, 3e-200].variance()", "0.0"),
        ("[1e-200, 3e-200].stdev()", "0.0"),
        (
            "String([1.7976931348623157e308, 1.7976931348623157e308].stdev())",
            "\"Infinity\"",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn the_selector_runs_once_per_item_in_receiver_order_with_only_the_item() {
    for (source, expected) in [
        (
            "seen = []\ndef weigh(item, position):\n    seen.push(position ?? item)\n    return item\n[[3, 1, 2].variance(weigh), seen]",
            "[0.6666666666666666,[3.0,1.0,2.0]]",
        ),
        (
            "seen = []\ndef weigh(item, position):\n    seen.push(position ?? item)\n    return item\n[[3, 1, 2].stdev(weigh), seen]",
            "[0.816496580927726,[3.0,1.0,2.0]]",
        ),
        ("[2, 3].variance((x, i) -> i ?? x)", "0.25"),
        ("big = x -> x * 2\n[1, 2].variance(big)", "1.0"),
        ("def big(x) => x * 2\n[1, 2].stdev(big)", "1.0"),
        ("[1, 2].variance(lambda x: x * 2)", "1.0"),
        ("[1, 2].variance(x => x * 2)", "1.0"),
        ("[1, 2].variance((x) -> x * 2)", "1.0"),
        (
            "factor = 3\ndef scaled(values) => values.variance(v -> v * factor)\nscaled([1, 3])",
            "9.0",
        ),
        (
            "seen = []\ndef mark(label, value):\n    seen.push(label)\n    return value\nxs = [3, 1]\n[mark(\"receiver\", xs).variance(mark(\"selector\", x -> x), mark(\"extra\", 0)), seen]",
            "[1.0,[\"receiver\",\"selector\",\"extra\"]]",
        ),
        (
            "seen = []\ndef mark() => seen.push(1)\nxs = null\nr = xs.stdev(mark())\n[r, seen]",
            "[null,[]]",
        ),
        (
            "seen = []\nclass Unit(n):\n    this.n = n\nUnit(2)\nUnit(4)\n[Unit.variance(unit -> seen.push(unit.n) > 0 ? unit.n : 0), seen]",
            "[1.0,[2.0,4.0]]",
        ),
        ("xs = [2, 4]\nxs.variance(x -> x, 5)", "1.0"),
        ("xs = [2, 4]\nxs.stdev(x -> x, null, \"extra\")", "1.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for source in [
        "xs = [1, 2]\nxs.variance(missing)",
        "xs = [1, 2]\nxs.stdev(5, missing)",
        "xs = [1, 2]\nxs.variance(x -> x, missing)",
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
    for (query, message) in [
        ("variance", "variance expects a function"),
        ("stdev", "stdev expects a function"),
    ] {
        for source in [
            format!("xs = [1, 2]\nxs.{query}(null)"),
            format!("xs = [1, 2]\nxs.{query}(5)"),
            format!("xs = [1, 2]\nxs.{query}(1)"),
            format!("xs = [1, 2]\nxs.{query}(\"k\")"),
            format!("xs = [1, 2]\nxs.{query}([x -> x])"),
            format!("xs = [1, 2]\nxs.{query}({{}})"),
            format!("xs = [1, 2]\nxs.{query}(true)"),
            format!("[].{query}(null)"),
            format!("[].{query}(5)"),
            format!("class Empty\nEmpty.{query}(1)"),
            format!(
                "class Order(total):\n    this.total = total\nOrder(1)\nOrder.{query}(\"total\")"
            ),
            format!(
                "class Calculator(base):\n    this.base = base\n    def {query}(a, b) => this.base + a * b\nc = Calculator(1)\nCalculator.{query}(2, 3)"
            ),
        ] {
            assert_eq!(
                failure(&source),
                (ErrorKind::Type, message.to_string()),
                "{source}"
            );
        }
    }
    let mut runtime = Runtime::new();
    runtime
        .run("xs = [2, 4]\nys = xs.variance(x -> x)")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "ys = xs.variance(1)"),
        "TypeError: variance expects a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(json(&mut runtime, "xs.push(6)\nys"), "2.6666666666666665");
    assert_eq!(
        failure("[1, 2].median(1)"),
        (ErrorKind::Type, "median expects a function".to_string())
    );
}

#[test]
fn lists_however_they_are_reached_are_receivers() {
    for (source, expected) in [
        ("[1, 2, 3, 4].variance()", "1.25"),
        ("[1, 2, 3, 4].stdev()", "1.118033988749895"),
        ("xs = [1, 3]\nxs.variance()", "1.0"),
        (
            "o = {inner: {items: [{n: 2}, {n: 6}]}}\no.inner.items.variance(i -> i.n)",
            "4.0",
        ),
        ("xs = [1, 3, 5]\nxs[1:].variance()", "1.0"),
        ("xs = [1, 3]\nxs.map(x -> x * 2).stdev()", "2.0"),
        ("xs = [1, 3, 9]\nxs.filter(x -> x < 5).variance()", "1.0"),
        ("xs = [1, 3, 5]\nxs.slice(0, 2).stdev()", "1.0"),
        (
            "xs = [1]\nxs.concat([3], 5).variance()",
            "2.6666666666666665",
        ),
        (
            "def make() => [1, 3, 5]\nmake().variance()",
            "2.6666666666666665",
        ),
        ("xs = [1, 3]\nxs.value.variance()", "1.0"),
        ("xs = [1, 3]\nxs.push(5)\nxs@first.variance()", "1.0"),
        ("grid = [[1, 3], [4]]\ngrid[0].variance()", "1.0"),
        (
            "class Team(laps):\n    this.laps = laps\n    def wobble():\n        return this.laps.stdev()\nt = Team([61, 58, 64])\nt.wobble()",
            "2.449489742783178",
        ),
        (
            "rows = [[1, 3], [2, 6]]\nrows.map(row -> row.variance())",
            "[1.0,4.0]",
        ),
        (
            "rows = [[1, 3], [2, 6]]\nrows.map(row -> row.variance()).variance()",
            "2.25",
        ),
        ("variance = [1, 3]\nvariance.variance()", "1.0"),
        ("stdev = [1, 3]\nstdev.stdev()", "1.0"),
        ("Order = [2, 6]\nOrder.variance()", "4.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn populations_by_name_through_a_variable_and_by_dollar_name_are_receivers() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)\nOrder(12)\nkind = Order")
        .unwrap();
    for (source, expected) in [
        ("Order.variance(o -> o.total)", "8.666666666666666"),
        ("Order.stdev(o -> o.total)", "2.943920288775949"),
        ("Order.variance(.total)", "8.666666666666666"),
        ("Order[1:].variance(o -> o.total)", "6.25"),
        ("Order.list().variance(o -> o.total)", "8.666666666666666"),
        ("kind.variance(o -> o.total)", "8.666666666666666"),
        ("$Order.variance(o -> o.total)", "8.666666666666666"),
        ("Order.filter(o -> o.total > 6).stdev(o -> o.total)", "2.5"),
        ("Order.map(o -> o.total).variance()", "8.666666666666666"),
        ("String(Order.variance())", "\"NaN\""),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "class Team(name):\n    this.name = name\n    def variance(): return 7\n    def stdev(): return 8\nred = Team(\"red\")\n[String(Team.variance()), Team.stdev(t -> t.variance()), red.variance(), red.stdev()]",
            "[\"NaN\",0.0,7.0,8.0]",
        ),
        (
            "class Calculator(base):\n    this.base = base\n    def variance(a, b) => this.base + a * b\nc = Calculator(1)\nd = Calculator(3)\n[c.variance(2, 3), Calculator.variance(x -> x.base)]",
            "[7.0,1.0]",
        ),
        (
            "class Empty\n[String(Empty.variance()), String(Empty.stdev(e -> e.n))]",
            "[\"NaN\",\"NaN\"]",
        ),
        (
            "class A:\n    def init(n):\n        this.n = n\nclass B extends A:\n    def init(n):\n        super(n)\nA(2)\nB(7)\nA(4)\n[A.variance(a -> a.n), B.variance(b -> b.n)]",
            "[1.0,0.0]",
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
                "v = xs.variance(x -> x.k * factor)",
                "v = xs.average(x -> (x.k * factor - xs.average(x -> x.k * factor)) * (x.k * factor - xs.average(x -> x.k * factor)))",
            ),
            (
                "d = xs.stdev(x -> x.k * factor)",
                "d = Math.sqrt(xs.average(x -> (x.k * factor - xs.average(x -> x.k * factor)) * (x.k * factor - xs.average(x -> x.k * factor))))",
            ),
            ("[v, d]", "[v, d]"),
            ("xs.push({n: \"c\", k: 8})", "xs.push({n: \"c\", k: 8})"),
            ("[v, d]", "[v, d]"),
            ("factor = 1", "factor = 1"),
            ("[v, d]", "[v, d]"),
            ("xs.pop()", "xs.pop()"),
            ("xs.shift()", "xs.shift()"),
            ("[v, d]", "[v, d]"),
            (
                "xs = [{n: \"d\", k: 0.5}, {n: \"e\", k: 6}]",
                "xs = [{n: \"d\", k: 0.5}, {n: \"e\", k: 6}]",
            ),
            ("xs[0].k = 9", "xs[0].k = 9"),
            ("[v, d]", "[v, d]"),
            ("xs = []", "xs = []"),
            ("[String(v), String(d)]", "[String(v), String(d)]"),
        ],
        &["v", "d"],
    );
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "scores = [2, 4, 4, 4, 5, 5, 7, 9]\nspread = scores.variance()\ndeviation = scores.stdev()\n[spread, deviation]",
            "[4.0,2.0]",
        ),
        (
            "scores.push(11)\n[spread, deviation]",
            "[7.111111111111111,2.6666666666666665]",
        ),
        ("scores.pop()\n[spread, deviation]", "[4.0,2.0]"),
        (
            "scores.shift()\n[spread, deviation]",
            "[3.102040816326531,1.761261143705422]",
        ),
        ("scores.push(2)\n[spread, deviation]", "[4.0,2.0]"),
        ("scores = [6]\n[spread, deviation]", "[0.0,0.0]"),
        ("scores = [1, 3]\n[spread, deviation]", "[1.0,1.0]"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn declarations_follow_new_and_deleted_instances_and_class_level_rules() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class R(n, t):\n    this.n = n\n    this.t = t\nspread = R.variance(r -> r.t)\nString(spread)",
            "\"NaN\"",
        ),
        ("a = R(\"a\", 3)\nb = R(\"b\", 5)\nspread", "1.0"),
        ("c = R(\"c\", 10)\nspread", "8.666666666666666"),
        ("b.t = 4\nspread", "9.555555555555555"),
        ("delete c.t\nspread", "null"),
        ("delete c.n\ndelete c\nspread", "0.25"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class Room(name):\n    this.name = name\nclass Sensor(room, temp):\n    this.room = room\n    this.temp = temp\n$Room.spread = Sensor.filter(sensor -> sensor.room == $Room).stdev(sensor -> sensor.temp)\nlab = Room(\"lab\")\nhall = Room(\"hall\")\nSensor(lab, 18)\nprobe = Sensor(lab, 22)\nSensor(hall, 20)\nSensor(hall, 26)\n[lab.spread, hall.spread]",
            "[2.0,3.0]",
        ),
        ("Sensor(lab, 20)\nlab.spread", "1.632993161855452"),
        ("probe.temp = 20\nlab.spread", "0.9428090415820634"),
        (
            "probe.room = hall\n[lab.spread, hall.spread]",
            "[1.0,2.8284271247461903]",
        ),
        ("attic = Room(\"attic\")\nString(attic.spread)", "\"NaN\""),
        ("Sensor(attic, 15)\nattic.spread", "0.0"),
        (
            "levels = Room.variance(room -> room.spread)\nlevels",
            "1.3714606389452912",
        ),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "class Machine(readings):\n    this.readings = readings\n$Machine.drift = $Machine.readings.stdev()\npress = Machine([50, 54, 46])\nlathe = Machine([30])\npress.readings.push(50)\nlathe.readings = [1, 3]\n[press.drift, lathe.drift]",
            "[2.8284271247461903,1.0]",
        ),
        (
            "class P(x):\n    this.x = x\n$P.spread = P.variance(other -> other.x - $P.x)\np = P(1)\nq = P(2)\nr = P(6)\n[p.spread, q.spread, r.spread]",
            "[4.666666666666667,4.666666666666667,4.666666666666667]",
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
            "scores = [1, 3]\nspread = scores.variance()\n(why spread).map(step -> step.rule)"
        ),
        "[\"spread = scores.variance()\",\"scores = [1,3]\"]"
    );
    assert_eq!(json(&mut runtime, "(why spread)[0].from"), "[\"scores\"]");
    assert_eq!(
        json(&mut runtime, "(affects scores).map(step -> step.node)"),
        "[\"spread\"]"
    );
    assert_eq!(dependencies(&runtime, "spread"), ["scores"]);
    assert_eq!(
        json(
            &mut runtime,
            "rates = [0.5]\nwobble = rates.stdev(rate -> rate * 2)\n(why wobble).map(step -> step.rule)"
        ),
        "[\"wobble = rates.stdev(function())\",\"rates = [0.5]\"]"
    );
    let mut runtime = Runtime::new();
    runtime
        .run("class Bid(amount):\n    this.amount = amount\nbook = Bid.variance(bid -> bid.amount)\nBid(5)")
        .unwrap();
    assert_eq!(dependencies(&runtime, "book"), ["$Bid", "obj:0.amount"]);
    let mut spelled = Runtime::new();
    assert_eq!(
        json(
            &mut spelled,
            "scores = [1, 3]\nspread = scores.average(x -> (x - scores.average()) * (x - scores.average()))\n(why spread).map(step -> step.rule)"
        ),
        "[\"spread = scores.average(function())\",\"scores = [1,3]\"]"
    );
    assert_eq!(dependencies(&spelled, "spread"), ["scores"]);
    assert_eq!(
        result("def key(x) => x.k\nr = [].variance(key)\n(affects key).map(step -> step.node)"),
        "[\"r\"]"
    );
    for (source, message) in [
        (
            "xs = [1, 2]\nxs.variance()(1)",
            "xs.variance() is not a function",
        ),
        (
            "xs = [1, 2]\nall x in xs.variance(): x",
            "xs.variance() is not a collection",
        ),
        (
            "xs = [1, 2]\nwhy xs.variance()",
            "Cannot explain xs.variance()",
        ),
        ("xs = [1, 2]\nxs.stdev()(1)", "xs.stdev() is not a function"),
    ] {
        assert_eq!(
            failure(source),
            (ErrorKind::Type, message.to_string()),
            "{source}"
        );
    }
}

#[test]
fn snapshots_hypotheticals_templates_and_conditions_read_variances() {
    for (source, expected) in [
        (
            "xs = [1, 3]\nv = xs.variance()\nfrozen = v.value\nxs.push(5)\n[v, frozen]",
            "[2.6666666666666665,1.0]",
        ),
        (
            "xs = [1, 3]\nfrozen = xs.value.stdev()\nxs.push(5)\nfrozen",
            "1.0",
        ),
        (
            "xs = [1, 3]\nv = xs.variance()\nseen = suppose xs.push(5): v\n[seen, v]",
            "[2.6666666666666665,1.0]",
        ),
        (
            "xs = [1, 3]\nlabel = `${xs.stdev()} apart`\nxs.push(2)\nlabel",
            "\"0.816496580927726 apart\"",
        ),
        (
            "xs = [1, 3]\nif xs.variance() > 0.5:\n    flag = true\nflag",
            "true",
        ),
        (
            "xs = [1, 3]\nsize = xs.stdev() > 1 ? \"wide\" : \"narrow\"\nxs.push(9)\nsize",
            "\"wide\"",
        ),
        (
            "xs = [1, 3]\nonce := xs.variance()\nxs.push(5)\n[once, xs.variance()]",
            "[1.0,2.6666666666666665]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn failed_statements_and_transactions_restore_variances() {
    behaves_as_spelled(
        &[
            ("rows = [{k: 2}, {k: 4}]", "rows = [{k: 2}, {k: 4}]"),
            (
                "v = rows.variance(row -> row.k > 9 ? row.k.toUpperCase() : row.k)",
                "v = rows.average(row -> ((row.k > 9 ? row.k.toUpperCase() : row.k) - rows.average(row -> row.k > 9 ? row.k.toUpperCase() : row.k)) * ((row.k > 9 ? row.k.toUpperCase() : row.k) - rows.average(row -> row.k > 9 ? row.k.toUpperCase() : row.k)))",
            ),
            (
                "try:\n    rows.push({k: 6})\n    assert(v, 2.6666666666666665)\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
                "try:\n    rows.push({k: 6})\n    assert(v, 2.6666666666666665)\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
            ),
            ("[rows.length, v]", "[rows.length, v]"),
            ("rows.push({k: 10})", "rows.push({k: 10})"),
            ("[rows.length, v]", "[rows.length, v]"),
            ("rows.push({k: 6})", "rows.push({k: 6})"),
            ("v", "v"),
        ],
        &["v"],
    );
    let mut runtime = Runtime::new();
    runtime
        .run("rows = [{k: 2}, {k: 4}]\nv = rows.stdev(row -> row.k > 9 ? row.k.toUpperCase() : row.k)")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "rows.push({k: 10})"),
        "TypeError: number.toUpperCase is not a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(json(&mut runtime, "[rows.length, v]"), "[2.0,1.0]");
    assert_eq!(
        result(
            "class Probe(t):\n    this.t = t\nProbe(2)\nProbe(4)\nheat = Probe.variance(p -> p.t > 9 ? p.t.toUpperCase() : p.t)\nfailure = \"\"\ntry:\n    Probe(50)\ncatch error:\n    failure = error\n[failure, Probe.length, heat]"
        ),
        "[\"TypeError: number.toUpperCase is not a function\",2.0,1.0]"
    );
    assert_eq!(
        result(
            "xs = [1, 3]\nv = xs.variance()\ntry:\n    xs.push(5)\n    throw \"UNDO\"\ncatch error:\n    pass\nv"
        ),
        "1.0"
    );
}

#[test]
fn large_receivers_measure_spread_through_the_public_api() {
    let mut runtime = Runtime::new();
    runtime
        .run("holder = {items: 0}\ncounter = {n: 0}\ndef tick(x):\n    counter.n = counter.n + 1\n    return x")
        .unwrap();
    let count = 100_000_u32;
    hold(
        &mut runtime,
        Value::List(
            (0..count)
                .map(|position| Value::Number(if position % 2 == 0 { 1.0 } else { 3.0 }))
                .collect(),
        ),
    );
    assert_eq!(json(&mut runtime, "holder.items.variance()"), "1.0");
    assert_eq!(json(&mut runtime, "holder.items.stdev()"), "1.0");
    assert_eq!(
        json(&mut runtime, "[holder.items.variance(tick), counter.n]"),
        "[1.0,100000.0]"
    );
    assert_eq!(json(&mut runtime, "holder.items.length"), "100000.0");
}

#[test]
fn sums_averages_medians_and_products_keep_their_results_beside_variances() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("[].sum()", 0.0),
        ("[].product()", 1.0),
        ("[0.1, 0.2, 0.3].sum()", 0.6000000000000001),
        ("[0.1, 0.2, 0.3].average()", 0.20000000000000004),
        ("[0.1, 0.2, 0.3].variance()", 0.006666666666666665),
        ("[2, \"3\", true].sum()", 6.0),
        ("[1, 2].average()", 1.5),
        ("[4, 1, 3, 2].median()", 2.5),
        ("[2, 3, 4].product()", 24.0),
        ("[5, 1, 5].mode()", 5.0),
        ("[5, 1, 9].max()", 9.0),
    ] {
        assert!(
            same_number(number(&mut runtime, source), expected),
            "{source}"
        );
    }
    assert!(number(&mut runtime, "[].average()").is_nan());
    assert!(number(&mut runtime, "[].median()").is_nan());
    for (source, expected) in [
        ("o = {}\nString([1, o.missing].average())", "\"undefined\""),
        ("t = [1, null].sum()\nt", "1.0"),
        (
            "xs = [1, 2]\ntotal = xs.average()\nxs.push(3)\ntotal",
            "2.0",
        ),
        ("[1, 3].distinct()", "[1.0,3.0]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn the_compatibility_rows_give_their_documented_results() {
    let orders = "class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)\nOrder(12)\n";
    let team = "class Team(name):\n    this.name = name\n    def variance(): return 7\n    def stdev(): return 8\nred = Team(\"red\")\n";
    let calculator = "class Calculator(base):\n    this.base = base\n    def variance(a, b) => this.base + a * b\nc = Calculator(1)\n";
    for (source, expected) in [
        ("[1, 2, 3, 4].variance()".to_string(), "1.25"),
        ("[1, 2, 3, 4].stdev()".to_string(), "1.118033988749895"),
        ("String([].variance())".to_string(), "\"NaN\""),
        ("[5].variance()".to_string(), "0.0"),
        (
            "xs = [2, 4, 4, 4, 5, 5, 7, 9]\nxs.variance(x -> x * 2)".to_string(),
            "16.0",
        ),
        (
            "xs = [2, 4, 4, 4, 5, 5, 7, 9]\nxs.stdev(x -> x * 2)".to_string(),
            "4.0",
        ),
        ("o = {items: [1, 3]}\no.items.variance()".to_string(), "1.0"),
        ("xs = [1, 3]\nxs.map(x -> x * 2).stdev()".to_string(), "2.0"),
        (
            "xs = [1, 3, 9]\nxs.filter(x -> x < 5).variance()".to_string(),
            "1.0",
        ),
        ("xs = [1, 3]\nxs.value.variance()".to_string(), "1.0"),
        ("xs = [1, 3]\nxs.push(5)\nxs@first.variance()".to_string(), "1.0"),
        ("xs = [1, 3, 5]\nxs[1:].variance()".to_string(), "1.0"),
        ("xs = [1, 3, 5]\nxs.slice(0, 2).stdev()".to_string(), "1.0"),
        ("xs = [1]\nxs.concat([3]).variance()".to_string(), "1.0"),
        ("grid = [[1, 3], [4]]\ngrid[0].variance()".to_string(), "1.0"),
        (
            "def make() => [1, 3, 5]\nmake().variance()".to_string(),
            "2.6666666666666665",
        ),
        (
            format!("{orders}Order.variance(o -> o.total)"),
            "8.666666666666666",
        ),
        (
            format!("{orders}Order.stdev(o -> o.total)"),
            "2.943920288775949",
        ),
        (format!("{orders}Order.variance(.total)"), "8.666666666666666"),
        (format!("{orders}Order[1:].variance(o -> o.total)"), "6.25"),
        (
            format!("{orders}Order.list().variance(o -> o.total)"),
            "8.666666666666666",
        ),
        (
            format!("{orders}kind = Order\nkind.variance(o -> o.total)"),
            "8.666666666666666",
        ),
        (
            format!("{orders}$Order.variance(o -> o.total)"),
            "8.666666666666666",
        ),
        (
            format!("{orders}Order.filter(o -> o.total > 6).stdev(o -> o.total)"),
            "2.5",
        ),
        (
            "class Empty\nString(Empty.variance())".to_string(),
            "\"NaN\"",
        ),
        (
            "class Box(sides):\n    this.sides = sides\n$Box.spread = $Box.sides.variance()\nb = Box([1, 3])\nb.spread".to_string(),
            "1.0",
        ),
        (format!("{team}String(Team.variance())"), "\"NaN\""),
        (format!("{team}Team.stdev(t -> t.variance())"), "0.0"),
        (
            format!("{calculator}Calculator.variance(2, 3)"),
            "TypeError: variance expects a function at Some(Position { line: 5, column: 1 })",
        ),
        ("[2, \"4\", true, null].variance()".to_string(), "2.1875"),
        ("[[2], [4]].stdev()".to_string(), "1.0"),
        (
            "[1.5, 2.25, 3.125].variance()".to_string(),
            "0.44097222222222215",
        ),
        (
            "[1000000001, 1000000002, 1000000003].variance()".to_string(),
            "0.6666666666666666",
        ),
        ("String([1, \"a\"].stdev())".to_string(), "\"NaN\""),
        (
            "String([Number.POSITIVE_INFINITY, 1].variance())".to_string(),
            "\"NaN\"",
        ),
        (
            "xs = [1e200, -1e200]\nString(xs.stdev())".to_string(),
            "\"Infinity\"",
        ),
        (
            "o = {}\nString([1, o.missing].variance())".to_string(),
            "\"undefined\"",
        ),
        ("xs = [1, 3]\nxs.variance(x -> x, 5)".to_string(), "1.0"),
        (
            "xs = [1, 3]\nxs.variance(5)".to_string(),
            "TypeError: variance expects a function at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [1, 3]\nxs.stdev(\"total\")".to_string(),
            "TypeError: stdev expects a function at Some(Position { line: 2, column: 1 })",
        ),
        (
            "[].variance(null)".to_string(),
            "TypeError: variance expects a function at Some(Position { line: 1, column: 1 })",
        ),
        (
            format!("{orders}Order.stdev(\"total\")"),
            "TypeError: stdev expects a function at Some(Position { line: 6, column: 1 })",
        ),
        (
            "xs = [1, 3]\nxs.variance()(1)".to_string(),
            "TypeError: xs.variance() is not a function at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [1, 3]\nall t in xs.variance(): t".to_string(),
            "TypeError: xs.variance() is not a collection at Some(Position { line: 2, column: 1 })",
        ),
        (
            "def scaled(xs, k) => xs.variance(x -> x * k)\nscaled([1, 3], 3)".to_string(),
            "ReferenceError: k is not defined at Some(Position { line: 2, column: 1 })",
        ),
        (
            "def scaled(xs, k) => xs.sum(x -> x * k)\nscaled([1, 3], 3)".to_string(),
            "ReferenceError: k is not defined at Some(Position { line: 2, column: 1 })",
        ),
        ("variance = [1, 3]\nvariance.variance()".to_string(), "1.0"),
    ] {
        assert_eq!(outcome(&mut Runtime::new(), &source), expected, "{source}");
    }
}

#[test]
fn unchanged_readings_keep_their_accepted_results() {
    for (source, expected) in [
        ("xs = [1]\nxs.variance", "null"),
        ("xs = [1]\nxs.stdev", "null"),
        ("class Order\nOrder.variance", "null"),
        ("xs = [1]\ntypeof xs.variance", "\"undefined\""),
        (
            "o = {variance: 3, stdev: 4}\n[o.variance, o.stdev]",
            "[3.0,4.0]",
        ),
        ("x = {stdev: 4}.stdev\nx", "4.0"),
        ("o = {variance: n -> n * 2}\no.variance(4)", "8.0"),
        ("o = {}\no.variance()", "null"),
        ("class P\np = P()\np.stdev()", "null"),
        (
            "class Calculator(base):\n    this.base = base\n    def variance(a, b) => this.base + a * b\nc = Calculator(1)\n[c.variance(2, 3), c.stdev]",
            "[7.0,null]",
        ),
        (
            "class Team(name):\n    this.name = name\n    def variance(): return 7\nred = Team(\"red\")\nred.variance()",
            "7.0",
        ),
        (
            "def variance(values) => values.length\nvariance([1, 2])",
            "2.0",
        ),
        ("variance = [1, 2]\nvariance.length", "2.0"),
        ("def g(stdev) => stdev * 2\ng(4)", "8.0"),
        ("f = lambda variance: variance + 1\nf(4)", "5.0"),
        (
            "class S(variance):\n    this.variance = variance\ns = S([1, 2])\ns.variance",
            "[1.0,2.0]",
        ),
        ("xs = null\nxs.variance(i -> i)", "null"),
        ("o = {}\no.items.stdev()", "null"),
        (
            "class Team(scores):\n    this.scores = scores\n    def mean() => this.scores.average()\ndef variance(team) => team.scores.average(score -> score * score) - team.mean() ** 2\nred = Team([2, 4])\nvariance(red)",
            "1.0",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, kind, message) in [
        (
            "\"abc\".variance()",
            ErrorKind::Type,
            "abc.variance is not a function",
        ),
        (
            "(5).stdev()",
            ErrorKind::Type,
            "number.stdev is not a function",
        ),
        (
            "true.variance()",
            ErrorKind::Type,
            "boolean.variance is not a function",
        ),
        (
            "d = Date(0)\nd.stdev()",
            ErrorKind::Type,
            "Date.stdev is not a function",
        ),
        (
            "def f() => 1\nf.variance()",
            ErrorKind::Type,
            "function.variance is not a function",
        ),
        (
            "/a/.variance()",
            ErrorKind::Type,
            "variance is not a function",
        ),
        (
            "Math.variance([1, 2])",
            ErrorKind::Type,
            "Math.variance is not a function",
        ),
        (
            "Math.stdev(1, 2)",
            ErrorKind::Type,
            "Math.stdev is not a function",
        ),
        (
            "variance([1, 2])",
            ErrorKind::Reference,
            "variance is not defined",
        ),
        (
            "xs = [2, 4, 4, 4, 5, 5, 7, 9]\nxs.std()",
            ErrorKind::Type,
            "xs.std is not a function",
        ),
        (
            "xs = [2, 4, 4, 4, 5, 5, 7, 9]\nvariance of xs",
            ErrorKind::Syntax,
            "Unexpected keyword 'of'",
        ),
        (
            "xs = [1, 3]\nstdev of xs",
            ErrorKind::Syntax,
            "Unexpected keyword 'of'",
        ),
        (
            "xs = [1, 3]\nwhy xs.variance()",
            ErrorKind::Type,
            "Cannot explain xs.variance()",
        ),
        (
            "xs = [1]\nxs.variances()",
            ErrorKind::Type,
            "xs.variances is not a function",
        ),
    ] {
        assert_eq!(failure(source), (kind, message.to_string()), "{source}");
    }
    let error = Runtime::new()
        .run("xs = [2, 4, 4, 4, 5, 5, 7, 9]\nvariance of xs")
        .unwrap_err();
    assert_eq!(
        format!("{:?}", error.position()),
        "Some(Position { line: 2, column: 13 })"
    );
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    assert_eq!(
        result("xs = [1, 3]\nr = 0\ntry:\n    r = xs.stdev()\ncatch error:\n    r = error\nr"),
        "1.0"
    );
    assert_eq!(
        result("xs = [1, 3]\nr = 0\nif xs.variance() > 0.5:\n    r = 1\nr"),
        "1.0"
    );
    assert_eq!(
        result("xs = [1, 3]\nr = 0\ntry:\n    r = xs.stdevs()\ncatch error:\n    r = error\nr"),
        "\"TypeError: xs.stdevs is not a function\""
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-list-variances-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_variance_and_stdev() {
    let program = "scores = [2, 4, 4, 4, 5, 5, 7, 9]\nspread = scores.variance()\nscores.push(11)\n[spread, scores.stdev()]";
    let file = SourceFile::new("file");
    let measured = file.run(program, &["--json"]);
    assert!(measured.status.success(), "{measured:?}");
    assert!(measured.stderr.is_empty(), "{measured:?}");
    assert_eq!(
        String::from_utf8(measured.stdout).unwrap(),
        "[7.111111111111111,2.6666666666666665]\n"
    );
    let plain = file.run(program, &[]);
    assert!(plain.status.success(), "{plain:?}");
    assert_eq!(
        String::from_utf8(plain.stdout).unwrap(),
        "7.111111111111111,2.6666666666666665\n"
    );
    let graph = String::from_utf8(
        file.run(
            "scores = [1, 3]\nspread = scores.variance()\nspread",
            &["--json", "--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        graph.contains(
            "  scores [variable]\n    updates  spread\n  spread [variable]\n    reads    scores"
        ),
        "{graph}"
    );
    let block = String::from_utf8(
        file.run(
            "xs = [1, 3]\nif xs.variance() > 0.5:\n    flag = true\nflag",
            &["--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        block.contains("  xs [variable]\n    updates  if(xs.variance()>0.5)\n  if(xs.variance()>0.5) [if]\n    reads    xs"),
        "{block}"
    );
    let failed = file.run("xs = [1]\nxs.stdev(5)", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "TypeError: stdev expects a function (line 2, column 1)\n"
    );
    let session = "xs = [1, 3]\nv = xs.variance()\nv\nxs.push(5)\nv\nxs\nxs.variance(5)\n.clear\nxs = [2, 6]\nxs.stdev(x -> x + 1)\n.exit\n";
    for (options, expected) in [
        (
            &["--plain"][..],
            "1,3\n1\n1\n3\n2.6666666666666665\n1,3,5\nState cleared.\n2,6\n2\n",
        ),
        (
            &["--plain", "--json"][..],
            "[1.0,3.0]\n1.0\n1.0\n3.0\n2.6666666666666665\n[1.0,3.0,5.0]\nState cleared.\n[2.0,6.0]\n2.0\n",
        ),
        (
            &["--json"][..],
            "[1.0,3.0]\n1.0\n1.0\n3.0\n2.6666666666666665\n[1.0,3.0,5.0]\nState cleared.\n[2.0,6.0]\n2.0\n",
        ),
    ] {
        for newline in ["\n", "\r\n"] {
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.ends_with(expected), "{options:?} {stdout}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                "TypeError: variance expects a function (line 1, column 1)\n"
            );
        }
    }
}
