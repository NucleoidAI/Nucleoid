mod common;

use std::cmp::Ordering;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime, Value};
use serde::Deserialize;

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid finds the quantiles of lists with quantile",
    "Nucleoid finds the quartiles of class instance populations with quantile",
    "Nucleoid follows the rank and the selector of quantiles",
    "Nucleoid rolls back failed quantile updates",
];
const REFERENCE_TITLE: &str = "Nucleoid finds the quantiles of lists and populations with quantile";
const README_TITLE: &str = "The ninetieth percentile keeps the slow tail of response times live";
const EXAMPLES_TITLE: &str = "Ninetieth percentiles flag the routes whose trips run late";

const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid finds the quantiles of lists with quantile",
    "source": "# Nucleoid finds the quantiles of lists with quantile\n\nwaits = [12, 3, 45, 7, 30, 18]\ns = waits.sortBy(* 1)\nqt = h -> Math.lerp(s[h], s[h + 1], h % 1)\nslow = qt(s.length * 0.9 - 0.9)\nquick = qt((s.length - 1) * 0.2)\nassert([slow, quick, waits], [37.5, 7, [12, 3, 45, 7, 30, 18]])\nwaits.push(50)\nassert([slow, quick], [47, 8])\nwaits.shift()\nassert([slow, quick], [47.5, 7])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid finds the quartiles of class instance populations with quantile",
    "source": "# Nucleoid finds the quartiles of class instance populations with quantile\n\nclass Parcel(weight):\n    this.weight = weight\nParcel(4)\nParcel(9)\nheavy = Parcel(15)\nParcel(6)\nParcel(2)\ns = Parcel.sortBy(.weight)\nqt = h -> Math.lerp(s[h].weight, s[h + 1].weight, h % 1)\nlower = qt(s.length * 0.25 - 0.25)\nupper = qt(s.length * 0.75 - 0.75)\nassert([lower, upper], [4, 9])\nParcel(11)\nassert([lower, upper], [4.5, 10.5])\nheavy.weight = 1\nassert([lower, upper], [2.5, 8.25])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid follows the rank and the selector of quantiles",
    "source": "# Nucleoid follows the rank and the selector of quantiles\n\nscores = [70, 85, 90, 60, 75, 80]\nrank = 0.9\ns = scores.sort()\nqt = h -> Math.lerp(s[h], s[h + 1], h % 1)\ntop = qt(s.length * rank - rank)\nbonus = 5\ncurved = qt(s.length * 0.75 - 0.75) + bonus\nassert([top, curved], [87.5, 88.75])\nrank = 0.2\nassert([top, curved], [70, 88.75])\nbonus = 10\nassert([top, curved], [70, 93.75])\nscores.push(95)\nassert([top, curved], [71, 97.5])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back failed quantile updates",
    "source": "# Nucleoid rolls back failed quantile updates\n\ndef qh s h => Math.lerp(s[h], s[h + 1], h % 1)\ndef checked(level):\n    if level < 0:\n        throw \"NEGATIVE_LEVEL\"\n    return level\nlevels = [3, 5, 10, 8]\nhigh = qh(levels.sortBy(checked), levels.length * 0.75 - 0.75)\nfailure = \"\"\ntry:\n    levels.push(-1)\ncatch error:\n    failure = error\nassert([failure, levels, high], [\"NEGATIVE_LEVEL\", [3, 5, 10, 8], 8.5])\ntry:\n    levels.push(6)\n    assert(high, 8)\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([levels, high], [[3, 5, 10, 8], 8.5])\nclass Tank(level): this.level = level\nTank(4)\nTank(8)\nTank(6)\nlow = qh(Tank.map(.level).sortBy(checked), Tank.length * 0.25 - 0.25)\nfailure = \"\"\ntry:\n    Tank(-2)\ncatch error:\n    failure = error\nassert([failure, Tank.length, low], [\"NEGATIVE_LEVEL\", 3, 5])\nTank(12)\nlow",
    "expected": "5.5"
  },
  {
    "document": "reference",
    "title": "Nucleoid finds the quantiles of lists and populations with quantile",
    "source": "# Nucleoid finds the quantiles of lists and populations with quantile\n\ndef qh s h => Math.lerp(s[h], s[h + 1], h % 1)\nrain = [18, 21, 24, 15]\nwet = qh(rain.sort(), rain.length * 0.75 - 0.75)\nclass Lap(seconds):\n    this.seconds = seconds\nLap(7)\nLap(9)\nLap(4)\nquick = qh(Lap.map(.seconds).sort(), Lap.length * 0.25 - 0.25)\nassert([wet, quick], [21.75, 5.5])\nrain.push(30)\nLap(5)\nassert([wet, quick], [24, 4.75])\nquick",
    "expected": "4.75"
  },
  {
    "document": "readme",
    "title": "The ninetieth percentile keeps the slow tail of response times live",
    "source": "class Request(millis):\n    this.millis = millis\nRequest(120)\nRequest(80)\nspike = Request(300)\nRequest(95)\nRequest(110)\nRequest(140)\ns = Request.sortBy(.millis)\nh = Request.length * 0.9 - 0.9\np90 = Math.lerp(s[h].millis, s[h + 1].millis, h % 1)\nassert(p90, 220)\nspike.millis = 100\nRequest(150)\nassert(p90, 144)\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Ninetieth percentiles flag the routes whose trips run late",
    "source": "def qh s h => (1 - h % 1) * s[h] + h % 1 * s[h + 1]\nclass Route(name, times):\n    this.name = name\n    this.times = times\n$Route.slow = qh($Route.times.sort(), ($Route.times.length - 1) * 0.9)\n$Route.late = $Route.slow > 40\nnorth = Route(\"north\", [22, 25, 31, 28, 35, 24])\nsouth = Route(\"south\", [30, 41, 38, 33, 36, 52])\nassert([north.late, south.slow, south.late], [false, 46.5, true])\nnorth.times.push(50)\nsouth.times.pop()\n[north.slow, north.late, south.slow, south.late]\n",
    "expected": "[41,true,39.8,false]"
  }
]"###;

const WRITTEN_OUT: [(&str, &str); 15] = [
    (
        "slow = waits.quantile(0.9)",
        "s = waits.sortBy(* 1)\nqt = h -> Math.lerp(s[h], s[h + 1], h % 1)\nslow = qt(s.length * 0.9 - 0.9)",
    ),
    (
        "quick = waits.quantile(0.2)",
        "quick = qt((s.length - 1) * 0.2)",
    ),
    (
        "lower = Parcel.quantile(0.25, parcel -> parcel.weight)",
        "s = Parcel.sortBy(.weight)\nqt = h -> Math.lerp(s[h].weight, s[h + 1].weight, h % 1)\nlower = qt(s.length * 0.25 - 0.25)",
    ),
    (
        "upper = Parcel.quantile(0.75, parcel -> parcel.weight)",
        "upper = qt(s.length * 0.75 - 0.75)",
    ),
    (
        "top = scores.quantile(rank)",
        "s = scores.sort()\nqt = h -> Math.lerp(s[h], s[h + 1], h % 1)\ntop = qt(s.length * rank - rank)",
    ),
    (
        "scores.quantile(0.75, score -> score + bonus)",
        "qt(s.length * 0.75 - 0.75) + bonus",
    ),
    (
        "\n\ndef checked(level):",
        "\n\ndef qh s h => Math.lerp(s[h], s[h + 1], h % 1)\ndef checked(level):",
    ),
    (
        "levels.quantile(0.75, checked)",
        "qh(levels.sortBy(checked), levels.length * 0.75 - 0.75)",
    ),
    (
        "Tank.quantile(0.25, tank -> checked(tank.level))",
        "qh(Tank.map(.level).sortBy(checked), Tank.length * 0.25 - 0.25)",
    ),
    (
        "\n\nrain = [18, 21, 24, 15]",
        "\n\ndef qh s h => Math.lerp(s[h], s[h + 1], h % 1)\nrain = [18, 21, 24, 15]",
    ),
    (
        "rain.quantile(0.75)",
        "qh(rain.sort(), rain.length * 0.75 - 0.75)",
    ),
    (
        "Lap.quantile(0.25, lap -> lap.seconds)",
        "qh(Lap.map(.seconds).sort(), Lap.length * 0.25 - 0.25)",
    ),
    (
        "p90 = Request.quantile(0.9, request -> request.millis)",
        "s = Request.sortBy(.millis)\nh = Request.length * 0.9 - 0.9\np90 = Math.lerp(s[h].millis, s[h + 1].millis, h % 1)",
    ),
    (
        "class Route(name, times):",
        "def qh s h => (1 - h % 1) * s[h] + h % 1 * s[h + 1]\nclass Route(name, times):",
    ),
    (
        "$Route.times.quantile(0.9)",
        "qh($Route.times.sort(), ($Route.times.length - 1) * 0.9)",
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

const RANKS: [&str; 13] = [
    "0",
    "1",
    "-0",
    "0.5",
    "0.25",
    "0.75",
    "0.1",
    "0.9",
    "0.01",
    "0.99",
    "0.3",
    "0.7",
    "0.3333333333333333",
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

fn query_calls(source: &str) -> usize {
    let tokens: Vec<Token> = tokenize(source)
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    tokens
        .windows(3)
        .filter(|window| {
            matches!(window, [Token::Dot, Token::Identifier(name), Token::LeftParen] if name == "quantile")
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
            title: "quantile regression".to_string(),
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

fn reference_quantile(values: &[f64], rank: f64) -> f64 {
    if values.iter().any(|value| value.is_nan()) {
        return f64::NAN;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    if sorted.is_empty() {
        return f64::NAN;
    }
    let position = (sorted.len() - 1) as f64 * rank;
    let lower = position.floor() as usize;
    let fraction = position - position.floor();
    if fraction == 0.0 {
        return sorted[lower];
    }
    let (low, high) = (sorted[lower], sorted[lower + 1]);
    if low == high {
        return low;
    }
    (1.0 - fraction) * low + fraction * high
}

fn applies_to_spelling(values: &[f64], rank: f64) -> bool {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return false;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    let position = (sorted.len() - 1) as f64 * rank;
    let lower = position.floor() as usize;
    if rank.to_bits() == (-0.0_f64).to_bits() && sorted[0].to_bits() == (-0.0_f64).to_bits() {
        return false;
    }
    position == position.floor() || sorted[lower] != sorted[lower + 1]
}

fn spelled_quantile(list: &str, rank: &str) -> String {
    format!(
        "((s, h) -> (1 - h % 1) * s[h] + h % 1 * s[Math.ceil(h)])({list}.sortBy(v -> v), ({list}.length - 1) * {rank})"
    )
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
fn only_the_appended_cases_take_quantiles_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    let mut calls = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        let found = query_calls(&case.source);
        assert_eq!(found > 0, new, "{}", case.title);
        calls += found;
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
    assert_eq!(calls, 12);
    assert_eq!(assertions, 20);
    assert_eq!(cases, 1318);
}

#[test]
fn the_appended_cases_stand_after_nuc_123_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(titles[807], "Nucleoid rolls back failed variance updates");
    assert_eq!(titles[808..812], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid measures the spread of lists and populations with variance and stdev",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Standard deviations keep the scatter of delivery times live as orders change",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Standard deviations flag the machines whose readings drift",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 15 - 8 * usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 7].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### List and population quantiles\n\n```\n")
        .expect("the language reference documents quantiles");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("`quantile` ([NUC 124](nuc-0124.md))"));
    assert!(reference.contains("`quantile expects a number from 0 to 1`"));
    assert!(reference.contains("`quantile expects a function`"));
    assert_eq!(case.expected.as_deref(), Some("4.75"));
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
        assert_eq!(query_calls(&written), 0, "{}", sealed.title);
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
fn quantiles_equal_the_type_7_reference_bit_for_bit() {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let special: Vec<Vec<(String, f64)>> = [
        vec![],
        vec![("-0".to_string(), -0.0)],
        vec![("-0".to_string(), -0.0), ("0".to_string(), 0.0)],
        vec![("0".to_string(), 0.0), ("-0".to_string(), -0.0)],
        vec![
            ("-0".to_string(), -0.0),
            ("-0".to_string(), -0.0),
            ("0".to_string(), 0.0),
        ],
        vec![("5".to_string(), 5.0)],
        vec![("3".to_string(), 3.0), ("3".to_string(), 3.0)],
        vec![
            ("2".to_string(), 2.0),
            ("2".to_string(), 2.0),
            ("2".to_string(), 2.0),
            ("9".to_string(), 9.0),
        ],
        vec![("0.1".to_string(), 0.1), ("0.7".to_string(), 0.7)],
        vec![
            ("0.1".to_string(), 0.1),
            ("0.1".to_string(), 0.1),
            ("0.1".to_string(), 0.1),
        ],
        vec![
            ("1e308".to_string(), 1e308),
            ("1.5e308".to_string(), 1.5e308),
        ],
        vec![("1e308".to_string(), 1e308), ("1e308".to_string(), 1e308)],
        vec![
            ("-1.7976931348623157e308".to_string(), -f64::MAX),
            ("1.7976931348623157e308".to_string(), f64::MAX),
        ],
        vec![
            ("5e-324".to_string(), 5e-324),
            ("1e-323".to_string(), 1e-323),
        ],
        vec![
            ("5e-324".to_string(), 5e-324),
            ("5e-324".to_string(), 5e-324),
        ],
        vec![
            ("1000000001".to_string(), 1_000_000_001.0),
            ("1000000002".to_string(), 1_000_000_002.0),
            ("1000000003".to_string(), 1_000_000_003.0),
        ],
        vec![
            ("9007199254740993".to_string(), 9_007_199_254_740_992.0),
            ("3".to_string(), 3.0),
        ],
        vec![
            ("Number.POSITIVE_INFINITY".to_string(), f64::INFINITY),
            ("1".to_string(), 1.0),
        ],
        vec![
            ("Number.POSITIVE_INFINITY".to_string(), f64::INFINITY),
            ("Number.NEGATIVE_INFINITY".to_string(), f64::NEG_INFINITY),
        ],
        vec![
            ("Number.POSITIVE_INFINITY".to_string(), f64::INFINITY),
            ("Number.POSITIVE_INFINITY".to_string(), f64::INFINITY),
        ],
        vec![
            ("1e999".to_string(), f64::INFINITY),
            ("-3".to_string(), -3.0),
            ("1e999".to_string(), f64::INFINITY),
        ],
        vec![("0 / 0".to_string(), f64::NAN), ("1".to_string(), 1.0)],
    ]
    .into();
    let mut lists = special;
    for round in 0..300 {
        let length = (next() % 14) as usize;
        let items: Vec<(String, f64)> = (0..length)
            .map(|_| {
                let raw = (next() % 2_000_001) as f64 - 1_000_000.0;
                let scale = [1.0, 3.0, 10.0, 1000.0, 1_000_000.0][(next() % 5) as usize];
                let value = if round % 5 == 0 {
                    ((next() % 4) as f64) - 1.0
                } else {
                    raw / scale
                };
                (format!("{value:?}"), value)
            })
            .collect();
        lists.push(items);
    }
    let mut runtime = Runtime::new();
    let mut compared = 0;
    let mut spelled = 0;
    let mut equal_neighbours = 0;
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
        runtime.run(&format!("xs = {text}")).unwrap();
        let random = format!("{:?}", (next() % 1_000_001) as f64 / 1_000_000.0);
        for rank_text in RANKS.iter().copied().chain([random.as_str()]) {
            let rank: f64 = match rank_text {
                "-0" => -0.0,
                other => other.parse().unwrap(),
            };
            let expected = reference_quantile(&values, rank);
            let measured = number(&mut runtime, &format!("xs.quantile({rank_text})"));
            assert!(
                same_number(measured, expected),
                "{text} at {rank_text}: {measured:?} and {expected:?}"
            );
            compared += 1;
            let tripled: Vec<f64> = values.iter().map(|value| value * 3.0).collect();
            let selected = number(
                &mut runtime,
                &format!("xs.quantile({rank_text}, item -> item * 3)"),
            );
            assert!(
                same_number(selected, reference_quantile(&tripled, rank)),
                "{text} at {rank_text} with a selector"
            );
            if applies_to_spelling(&values, rank) {
                let written = number(&mut runtime, &spelled_quantile("xs", rank_text));
                assert!(
                    same_number(measured, written),
                    "{text} at {rank_text}: the spelling gives {written:?}"
                );
                spelled += 1;
            } else if !values.is_empty() && values.iter().all(|value| value.is_finite()) {
                equal_neighbours += 1;
            }
        }
    }
    assert!(lists.len() > 300);
    assert!(compared > 4000, "{compared}");
    assert!(spelled > 3000, "{spelled}");
    assert!(equal_neighbours > 100, "{equal_neighbours}");
}

#[test]
fn zeros_keep_their_receiver_order_in_long_receivers() {
    let mut runtime = Runtime::new();
    let mut signs = [0_usize; 2];
    for length in [21_usize, 22, 33, 50, 64, 101, 257, 500, 1001] {
        for pattern in 0..4_usize {
            let values: Vec<f64> = (0..length)
                .map(
                    |position| match (position * 7 + pattern * 3 + position / 9) % 5 {
                        0 | 3 => -0.0,
                        1 => 0.0,
                        2 => 1.0,
                        _ => -1.0,
                    },
                )
                .collect();
            let text = format!(
                "[{}]",
                values
                    .iter()
                    .map(|value| {
                        if value.to_bits() == (-0.0_f64).to_bits() {
                            "-0".to_string()
                        } else {
                            format!("{value:?}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            runtime.run(&format!("xs = {text}")).unwrap();
            let negated: Vec<f64> = values.iter().map(|value| -value).collect();
            for rank in [
                "0.3", "0.35", "0.4", "0.45", "0.5", "0.55", "0.6", "0.65", "0.7",
            ] {
                let rank_value: f64 = rank.parse().unwrap();
                let expected = reference_quantile(&values, rank_value);
                let measured = number(&mut runtime, &format!("xs.quantile({rank})"));
                assert!(
                    same_number(measured, expected),
                    "{length} {pattern} {rank}: {measured:?} and {expected:?}"
                );
                let selected = number(&mut runtime, &format!("xs.quantile({rank}, x -> -x)"));
                assert!(
                    same_number(selected, reference_quantile(&negated, rank_value)),
                    "{length} {pattern} {rank} negated"
                );
                if expected == 0.0 {
                    signs[usize::from(expected.is_sign_negative())] += 1;
                }
            }
        }
    }
    assert!(signs[0] > 20 && signs[1] > 20, "{signs:?}");
    let mut population = Runtime::new();
    let mut source = String::from("class Item(v):\n    this.v = v\n");
    let mut levels: Vec<f64> = Vec::new();
    for position in 0..101_usize {
        let value: f64 = match (position * 3 + position / 7) % 4 {
            0 | 2 => -0.0,
            1 => 0.0,
            _ => 2.0,
        };
        levels.push(value);
        source.push_str(if value.to_bits() == (-0.0_f64).to_bits() {
            "Item(-0)\n"
        } else if value == 0.0 {
            "Item(0)\n"
        } else {
            "Item(2)\n"
        });
    }
    population.run(&source).unwrap();
    for rank in ["0.1", "0.2", "0.3", "0.4", "0.5", "0.6"] {
        let rank_value: f64 = rank.parse().unwrap();
        let measured = number(
            &mut population,
            &format!("Item.quantile({rank}, item -> item.v)"),
        );
        assert!(
            same_number(measured, reference_quantile(&levels, rank_value)),
            "population at {rank}"
        );
    }
}

#[test]
fn equal_neighbours_give_their_value() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("[3, 3].quantile(0.01)", 3.0),
        ("[3, 3, 9].quantile(0.01)", 3.0),
        ("[1, 2, 2, 2, 9].quantile(0.4)", 2.0),
        ("[0.1, 0.1, 0.1].quantile(0.3)", 0.1),
        ("[1e308, 1e308].quantile(0.5)", 1e308),
        ("[5e-324, 5e-324].quantile(0.5)", 5e-324),
        ("[1, 2].quantile(0.5)", 1.5),
        ("[3, 9, 3].quantile(0.4)", 3.0),
        ("1 / [-0, 0].quantile(0.5)", f64::NEG_INFINITY),
        ("1 / [0, -0].quantile(0.5)", f64::INFINITY),
        ("1 / [-0, 0].quantile(0.3)", f64::NEG_INFINITY),
        ("1 / [0, -0].quantile(0.25)", f64::INFINITY),
        ("1 / [-0, -0].quantile(0.7)", f64::NEG_INFINITY),
        ("1 / [-0, 0].quantile(1)", f64::INFINITY),
        ("1 / [-1, -0].quantile(1)", f64::NEG_INFINITY),
        (
            "[Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY].quantile(0.5)",
            f64::INFINITY,
        ),
    ] {
        assert!(
            same_number(number(&mut runtime, source), expected),
            "{source}"
        );
    }
    assert!(same_number(
        number(&mut runtime, &spelled_quantile("[3, 3]", "0.01")),
        2.9999999999999996
    ));
    assert!(same_number(
        number(&mut runtime, &spelled_quantile("[-0, 0]", "0.5")),
        0.0
    ));
    assert!(same_number(
        number(&mut runtime, "Math.lerp(3, 3, 0.01)"),
        3.0
    ));
}

#[test]
fn nan_values_give_nan_before_long_receivers_are_sorted() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut runtime = Runtime::new();
    for _ in 0..40 {
        let length = 21 + (next() % 200) as usize;
        let spacing = 2 + next() % 9;
        let mut items: Vec<String> = (0..length)
            .map(|_| {
                if next() % spacing == 0 {
                    "0 / 0".to_string()
                } else {
                    (next() % 1000).to_string()
                }
            })
            .collect();
        items.push("0 / 0".to_string());
        let text = format!("[{}]", items.join(", "));
        runtime.run(&format!("xs = {text}")).unwrap();
        for source in [
            "xs.quantile(0.9)",
            "xs.quantile(0.5, x -> x)",
            "xs.median()",
            "xs.median(x -> x)",
        ] {
            assert!(number(&mut runtime, source).is_nan(), "{source} of {text}");
        }
    }
    runtime
        .run("class Reading(level):\n    this.level = level")
        .unwrap();
    for position in 0..120 {
        let level = if position % 7 == 3 {
            "0 / 0".to_string()
        } else {
            ((position * 37) % 101).to_string()
        };
        runtime.run(&format!("Reading({level})")).unwrap();
    }
    for source in [
        "Reading.quantile(0.25, r -> r.level)",
        "Reading.median(r -> r.level)",
    ] {
        assert!(number(&mut runtime, source).is_nan(), "{source}");
    }
}

#[test]
fn quantiles_convert_values_with_the_conversion_of_sum() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Order\norder = Order()\nholder = {}\nwhen = Date(5)\nf = x -> x")
        .unwrap();
    for value in VALUES {
        for (query, control) in [
            (
                format!("[{value}].quantile(0.5)"),
                format!("[{value}].map(x -> x * 1).quantile(0.5)"),
            ),
            (
                format!("[2, {value}].quantile(0.25)"),
                format!("[2, {value}].map(x -> x * 1).quantile(0.25)"),
            ),
            (
                format!("[{value}, 2].quantile(0.9)"),
                format!("[{value}, 2].map(x -> x * 1).quantile(0.9)"),
            ),
            (
                format!("[{value}, 2, 7].quantile(0.5)"),
                format!("[{value}, 2, 7].median()"),
            ),
            (
                format!("[7, {value}, 2].quantile(0.5, x -> x)"),
                format!("[7, {value}, 2].median(x -> x)"),
            ),
            (
                format!("[1, {value}, 4].quantile(1)"),
                format!("[1, {value}, 4].map(x -> x * 1).quantile(1)"),
            ),
        ] {
            assert_eq!(
                reading(&mut runtime, &query),
                reading(&mut runtime, &control),
                "{query}"
            );
        }
    }
    for (source, expected) in [
        ("[2, \"4\", true, null].quantile(0.5)", "1.5"),
        ("[2, \" 4 \", true].quantile(0.5)", "2.0"),
        ("[[2], [4]].quantile(0.5)", "3.0"),
        ("[2, [\"4\"]].quantile(0.5)", "3.0"),
        ("[3, \"\", 5].quantile(0.5)", "3.0"),
        ("[2, false].quantile(0.5)", "1.0"),
        ("d = Date(4)\n[d, 2].quantile(0.5)", "3.0"),
        ("String([2, \"x\"].quantile(0.5))", "\"NaN\""),
        ("String([2, {}].quantile(0.5))", "\"NaN\""),
        ("String([2, [1, 2]].quantile(0.5))", "\"NaN\""),
        ("String([2, x -> x].quantile(0.5))", "\"NaN\""),
        ("[1.5, 2.25, 3.125].quantile(0.5)", "2.25"),
        ("[\"10\", \"9\"].quantile(0)", "9.0"),
        ("[\"10\", \"9\"].sortBy(v -> v)[0]", "\"10\""),
        ("[\"10\", \"9\", \"100\"].quantile(1)", "100.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn an_undefined_value_makes_the_result_undefined_after_visiting_every_item() {
    for (source, expected) in [
        ("o = {}\n[2, o.missing, 3].quantile(0.5)", "null"),
        (
            "o = {}\nString([2, o.missing, 3].quantile(0.5))",
            "\"undefined\"",
        ),
        (
            "o = {}\nString([2, o.missing].quantile(0))",
            "\"undefined\"",
        ),
        (
            "o = {}\nString([0 / 0, o.missing].quantile(0.5))",
            "\"undefined\"",
        ),
        (
            "o = {}\nString([o.missing, 0 / 0].quantile(1))",
            "\"undefined\"",
        ),
        (
            "o = {}\nString([Number.POSITIVE_INFINITY, o.missing].quantile(0.5))",
            "\"undefined\"",
        ),
        (
            "String([{w: 2}, {}].quantile(0.5, x -> x.w))",
            "\"undefined\"",
        ),
        (
            "o = {}\nseen = []\ndef pick(x):\n    seen.push(x)\n    return x == 3 ? o.missing : x\n[String([2, 3, 4].quantile(0.5, pick)), seen]",
            "[\"undefined\",[2.0,3.0,4.0]]",
        ),
        ("o = {}\ntypeof [o.missing].quantile(0.5)", "\"undefined\""),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "xs = [{w: 2}, {}]\nq = xs.quantile(0.5, x -> x.w)\nq",
            "null",
        ),
        ("xs[1].w = 6\nq", "4.0"),
        ("delete xs[1].w\nq", "null"),
        ("xs.pop()\nq", "2.0"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn null_items_count_zero_while_reads_inside_a_selector_settle() {
    for (source, expected) in [
        ("p = [2, null].quantile(0.5)\np", "1.0"),
        ("p = [2, null, 6].quantile(0)\np", "0.0"),
        ("xs = [2, null]\np = xs.sortBy(v -> v)\np", "null"),
        ("[2, null].quantile(0.5)", "1.0"),
        (
            "xs = [{w: 2}, {w: null}]\np = xs.quantile(0.5, x -> x.w)\np",
            "null",
        ),
        (
            "xs = [{w: 2}, {w: null}]\nxs.quantile(0.5, x -> x.w)",
            "1.0",
        ),
        (
            "limit = null\nxs = [2, 4]\np = xs.quantile(0.5, x -> x + limit)\nlimit = 1\np",
            "4.0",
        ),
        ("xs = [2, null, 3]\np = xs.quantile(0.5, x -> 1)\np", "1.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn single_values_give_themselves_and_empty_receivers_give_nan() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("[5].quantile(0)", 5.0),
        ("[5].quantile(0.3)", 5.0),
        ("[5].quantile(1)", 5.0),
        ("[-7.25].quantile(0.9)", -7.25),
        ("[1e300].quantile(0.5)", 1e300),
        ("1 / [-0].quantile(0.5)", f64::NEG_INFINITY),
        (
            "[Number.NEGATIVE_INFINITY].quantile(0.4)",
            f64::NEG_INFINITY,
        ),
    ] {
        assert!(
            same_number(number(&mut runtime, source), expected),
            "{source}"
        );
    }
    for source in [
        "[].quantile(0.5)",
        "[].quantile(0)",
        "[].quantile(1)",
        "[1, 2].filter(x -> x > 5).quantile(0.5)",
        "[].quantile(0.5, x -> x * 2)",
    ] {
        assert!(number(&mut runtime, source).is_nan(), "{source}");
    }
    for (source, expected) in [
        ("typeof [].quantile(0.5)", "\"number\""),
        ("String([].quantile(0.9))", "\"NaN\""),
        ("class Empty\nString(Empty.quantile(0.5))", "\"NaN\""),
        (
            "class Empty\nString(Empty.quantile(0.5, empty -> empty.size))",
            "\"NaN\"",
        ),
        (
            "class Order(total):\n    this.total = total\nOrder(4)\nString(Order.filter(o -> o.total > 9).quantile(0.5, o -> o.total))",
            "\"NaN\"",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class Lap(seconds):\n    this.seconds = seconds\nq = Lap.quantile(0.5, lap -> lap.seconds)\nString(q)",
            "\"NaN\"",
        ),
        ("Lap(61)\nq", "61.0"),
        ("Lap(63)\nq", "62.0"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn infinities_take_their_places_and_whole_ranks_return_them() {
    for (source, expected) in [
        (
            "zs = [1, Number.POSITIVE_INFINITY]\n[zs.quantile(0), String(zs.quantile(0.5)), String(zs.quantile(1))]",
            "[1.0,\"Infinity\",\"Infinity\"]",
        ),
        (
            "zs = [1, Number.POSITIVE_INFINITY]\n[String(((s, h) -> (1 - h % 1) * s[h] + h % 1 * s[Math.ceil(h)])(zs.sortBy(v -> v), (zs.length - 1) * 1)), String(((s, h) -> (1 - h % 1) * s[h] + h % 1 * s[Math.ceil(h)])(zs.sortBy(v -> v), (zs.length - 1) * 0))]",
            "[\"NaN\",\"1\"]",
        ),
        (
            "String([Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY].quantile(0.5))",
            "\"NaN\"",
        ),
        (
            "String([Number.NEGATIVE_INFINITY, 1].quantile(0.5))",
            "\"-Infinity\"",
        ),
        (
            "String([Number.NEGATIVE_INFINITY, 1].quantile(0))",
            "\"-Infinity\"",
        ),
        (
            "String([Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY].quantile(0.3))",
            "\"Infinity\"",
        ),
        (
            "x = 1e999\n[typeof x, x > 1e308, [1, x].quantile(0), String([1, x].quantile(1))]",
            "[\"number\",true,1.0,\"Infinity\"]",
        ),
        ("String([1, 0 / 0].quantile(0))", "\"NaN\""),
        ("String([1, \"a\"].quantile(1))", "\"NaN\""),
        ("String([1e308, 1e308].median())", "\"Infinity\""),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("[1e308, 1.5e308].quantile(0.5)", 1.25e308),
        (
            "[1.7976931348623157e308, 1.7976931348623157e308].quantile(0.5)",
            f64::MAX,
        ),
        (
            "[-1.7976931348623157e308, 1.7976931348623157e308].quantile(0.5)",
            0.0,
        ),
    ] {
        assert!(
            same_number(number(&mut runtime, source), expected),
            "{source}"
        );
    }
}

#[test]
fn the_rank_must_be_a_number_from_zero_to_one_even_on_empty_receivers() {
    let message = (
        ErrorKind::Type,
        "quantile expects a number from 0 to 1".to_string(),
    );
    for rank in [
        "",
        "0 / 0",
        "-0.1",
        "1.5",
        "1.0000000000000002",
        "-1",
        "2",
        "90",
        "Number.POSITIVE_INFINITY",
        "Number.NEGATIVE_INFINITY",
        "1e999",
        "\"0.5\"",
        "\"\"",
        "true",
        "false",
        "null",
        "o.missing",
        "[0.5]",
        "{}",
        "x -> x",
        "Date(0)",
        "/a/",
    ] {
        for receiver in [
            "xs = [1, 2, 3]\no = {}\nxs",
            "o = {}\n[]",
            "o = {}\nclass Empty\nEmpty",
            "o = {}\nclass Order(total):\n    this.total = total\nOrder(1)\nOrder",
            "o = {}\nclass Order(total):\n    this.total = total\nOrder(1)\nkind = Order\nkind",
            "o = {}\nclass Order(total):\n    this.total = total\nOrder(1)\n$Order",
        ] {
            let source = format!("{receiver}.quantile({rank})");
            assert_eq!(failure(&source), message, "{source}");
        }
        if !rank.is_empty() {
            let source = format!("o = {{}}\nxs = [1, 2]\nxs.quantile({rank}, 5)");
            assert_eq!(failure(&source), message, "{source}");
        }
    }
    assert_eq!(
        failure(
            "class Calculator(base):\n    this.base = base\n    def quantile(a, b) => this.base + a * b\nc = Calculator(1)\nCalculator.quantile(2, 3)"
        ),
        message
    );
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("[3, 1, 2].quantile(0)", 1.0),
        ("[3, 1, 2].quantile(1)", 3.0),
        ("[3, 1, 2].quantile(-0)", 1.0),
        ("[3, 1, 2].quantile(1e-300)", 1.0),
        ("[3, 1, 2].quantile(0.9999999999999999)", 2.9999999999999996),
        ("r = 0.5\n[3, 1, 2].quantile(r)", 2.0),
    ] {
        assert!(
            same_number(number(&mut runtime, source), expected),
            "{source}"
        );
    }
    assert_eq!(
        failure("xs = [1, 2]\nxs.quantile(missing)"),
        (ErrorKind::Reference, "missing is not defined".to_string())
    );
    assert_eq!(
        failure("xs = [1, 2]\nxs.quantile(2, missing)"),
        (ErrorKind::Reference, "missing is not defined".to_string())
    );
    let mut runtime = Runtime::new();
    runtime
        .run("xs = [4, 8, 15, 16]\nr = 0.5\nq = xs.quantile(r)")
        .unwrap();
    assert_eq!(json(&mut runtime, "q"), "11.5");
    assert_eq!(
        outcome(&mut runtime, "r = 2"),
        "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(json(&mut runtime, "[r, q]"), "[0.5,11.5]");
    assert_eq!(json(&mut runtime, "r = 1\nq"), "16.0");
}

#[test]
fn the_selector_runs_once_per_item_in_receiver_order_with_only_the_item() {
    for (source, expected) in [
        (
            "seen = []\ndef weigh(item, position):\n    seen.push(position ?? item)\n    return item\n[[3, 1, 2].quantile(0.5, weigh), seen]",
            "[2.0,[3.0,1.0,2.0]]",
        ),
        ("[2, 3].quantile(0.5, (x, i) -> i ?? x)", "2.5"),
        ("big = x -> x * 2\n[1, 2].quantile(0.5, big)", "3.0"),
        ("def big(x) => x * 2\n[1, 2].quantile(0.5, big)", "3.0"),
        ("[1, 2].quantile(0.5, lambda x: x * 2)", "3.0"),
        ("[1, 2].quantile(0.5, x => x * 2)", "3.0"),
        ("[1, 2].quantile(0.5, (x) -> x * 2)", "3.0"),
        (
            "factor = 3\ndef scaled(values) => values.quantile(0.5, v -> v * factor)\nscaled([1, 3])",
            "6.0",
        ),
        (
            "seen = []\ndef mark(label, value):\n    seen.push(label)\n    return value\nxs = [3, 1]\n[mark(\"receiver\", xs).quantile(mark(\"rank\", 0.5), mark(\"selector\", x -> x), mark(\"extra\", 0)), seen]",
            "[2.0,[\"receiver\",\"rank\",\"selector\",\"extra\"]]",
        ),
        (
            "seen = []\ndef mark() => seen.push(1)\nxs = null\nr = xs.quantile(mark(), mark())\n[r, seen]",
            "[null,[]]",
        ),
        (
            "seen = []\nclass Unit(n):\n    this.n = n\nUnit(2)\nUnit(4)\n[Unit.quantile(0.5, unit -> seen.push(unit.n) > 0 ? unit.n : 0), seen]",
            "[3.0,[2.0,4.0]]",
        ),
        ("xs = [2, 4]\nxs.quantile(0.5, x -> x, 5)", "3.0"),
        (
            "xs = [2, 4]\nxs.quantile(0.5, x -> x, null, \"extra\")",
            "3.0",
        ),
        ("xs = [2, 4]\nxs.quantile(1, x -> -x)", "-2.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn a_supplied_selector_that_is_not_a_function_raises_even_on_empty_receivers() {
    let message = (ErrorKind::Type, "quantile expects a function".to_string());
    for source in [
        "xs = [1, 2]\nxs.quantile(0.5, null)",
        "xs = [1, 2]\nxs.quantile(0.5, 5)",
        "xs = [1, 2]\nxs.quantile(0, 1)",
        "xs = [1, 2]\nxs.quantile(1, \"k\")",
        "xs = [1, 2]\nxs.quantile(0.5, [x -> x])",
        "xs = [1, 2]\nxs.quantile(0.5, {})",
        "xs = [1, 2]\nxs.quantile(0.5, true)",
        "o = {}\nxs = [1, 2]\nxs.quantile(0.5, o.missing)",
        "[].quantile(0.5, null)",
        "[].quantile(0.5, 5)",
        "class Empty\nEmpty.quantile(0.5, 1)",
        "class Order(total):\n    this.total = total\nOrder(1)\nOrder.quantile(0.5, \"total\")",
        "class Order(total):\n    this.total = total\nOrder(1)\nkind = Order\nkind.quantile(0.5, 5)",
        "class Order(total):\n    this.total = total\nOrder(1)\n$Order.quantile(0.5, null)",
        "class Calculator(base):\n    this.base = base\n    def quantile(a, b) => this.base + a * b\nc = Calculator(1)\nCalculator.quantile(0.5, 3)",
    ] {
        assert_eq!(failure(source), message, "{source}");
    }
    let mut runtime = Runtime::new();
    runtime
        .run("xs = [2, 4]\nys = xs.quantile(0.5, x -> x)")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "ys = xs.quantile(0.5, 1)"),
        "TypeError: quantile expects a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(json(&mut runtime, "xs.push(6)\nys"), "4.0");
    assert_eq!(
        failure("[1, 2].median(0.5)"),
        (ErrorKind::Type, "median expects a function".to_string())
    );
}

#[test]
fn lists_however_they_are_reached_are_receivers() {
    for (source, expected) in [
        ("[1, 2, 3, 4].quantile(0.5)", "2.5"),
        ("[1, 2, 3, 4].quantile(0.25)", "1.75"),
        ("xs = [1, 3]\nxs.quantile(0.5)", "2.0"),
        (
            "o = {inner: {items: [{n: 2}, {n: 6}]}}\no.inner.items.quantile(0.5, i -> i.n)",
            "4.0",
        ),
        ("xs = [1, 3, 5]\nxs[1:].quantile(0.5)", "4.0"),
        ("xs = [1, 3]\nxs.map(x -> x * 2).quantile(0.5)", "4.0"),
        ("xs = [1, 3, 9]\nxs.filter(x -> x < 5).quantile(0.5)", "2.0"),
        ("xs = [1, 3, 5]\nxs.slice(0, 2).quantile(0.5)", "2.0"),
        ("xs = [1]\nxs.concat([3], 5).quantile(0.75)", "4.0"),
        ("def make() => [1, 3, 5]\nmake().quantile(0.75)", "4.0"),
        ("xs = [1, 3]\nxs.value.quantile(0.5)", "2.0"),
        ("xs = [1, 3]\nxs.push(5)\nxs@first.quantile(0.5)", "2.0"),
        ("grid = [[1, 3], [4]]\ngrid[0].quantile(0.5)", "2.0"),
        ("xs = [9, 1, 5]\nxs.sortBy(x -> -x).quantile(0.5)", "5.0"),
        (
            "class Team(laps):\n    this.laps = laps\n    def slow():\n        return this.laps.quantile(0.9)\nt = Team([61, 58, 64])\nt.slow()",
            "63.4",
        ),
        (
            "rows = [[1, 3], [2, 6]]\nrows.map(row -> row.quantile(0.5))",
            "[2.0,4.0]",
        ),
        (
            "rows = [[1, 3], [2, 6]]\nrows.map(row -> row.quantile(0.5)).quantile(0.5)",
            "3.0",
        ),
        ("quantile = [1, 3]\nquantile.quantile(0.5)", "2.0"),
        ("Order = [2, 6]\nOrder.quantile(0.5)", "4.0"),
        (
            "xs = [3, 1, 2]\nq = xs.quantile(0.5)\n[q, xs]",
            "[2.0,[3.0,1.0,2.0]]",
        ),
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
        ("Order.quantile(0.5, o -> o.total)", "7.0"),
        ("Order.quantile(0.9, o -> o.total)", "11.000000000000002"),
        ("Order.quantile(0.9, .total)", "11.000000000000002"),
        ("Order[1:].quantile(0.5, o -> o.total)", "9.5"),
        ("Order.list().quantile(0.25, o -> o.total)", "6.0"),
        ("kind.quantile(0.5, o -> o.total)", "7.0"),
        ("$Order.quantile(0.5, o -> o.total)", "7.0"),
        (
            "Order.filter(o -> o.total > 6).quantile(0.5, o -> o.total)",
            "9.5",
        ),
        ("Order.map(o -> o.total).quantile(0.75)", "9.5"),
        ("String(Order.quantile(0.5))", "\"NaN\""),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "class Team(name):\n    this.name = name\n    def quantile(q): return q * 10\nred = Team(\"red\")\n[String(Team.quantile(0.5)), Team.quantile(0.5, t -> t.quantile(0.5)), red.quantile(0.5)]",
            "[\"NaN\",5.0,5.0]",
        ),
        (
            "class Calculator(base):\n    this.base = base\n    def quantile(a, b) => this.base + a * b\nc = Calculator(1)\nd = Calculator(3)\n[c.quantile(2, 3), Calculator.quantile(1, x -> x.base)]",
            "[7.0,3.0]",
        ),
        (
            "class Empty\n[String(Empty.quantile(0.5)), String(Empty.quantile(1, e -> e.n))]",
            "[\"NaN\",\"NaN\"]",
        ),
        (
            "class A:\n    def init(n):\n        this.n = n\nclass B extends A:\n    def init(n):\n        super(n)\nA(2)\nB(7)\nA(4)\n[A.quantile(0.5, a -> a.n), B.quantile(0.5, b -> b.n)]",
            "[3.0,7.0]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn half_quantiles_are_medians_apart_from_overflow_subnormal_halves_and_zero_order() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut runtime = Runtime::new();
    for round in 0..200 {
        let length = 1 + (next() % 9) as usize;
        let values: Vec<String> = (0..length)
            .map(|_| {
                let raw = (next() % 20_001) as f64 - 10_000.0;
                format!("{:?}", raw / [1.0, 7.0, 100.0][(round % 3) as usize])
            })
            .collect();
        let text = format!("[{}]", values.join(", "));
        let quantile = number(&mut runtime, &format!("{text}.quantile(0.5)"));
        let median = number(&mut runtime, &format!("{text}.median()"));
        assert!(same_number(quantile, median), "{text}");
    }
    for (source, expected) in [
        ("[3, 1, 2].quantile(0.5)", 2.0),
        ("[3, 1, 2].median()", 2.0),
        ("[1e308, 1.5e308].quantile(0.5)", 1.25e308),
        ("[1e308, 1.5e308].median()", f64::INFINITY),
        ("[1e308, 1e308].quantile(0.5)", 1e308),
        ("[1e308, 1e308].median()", f64::INFINITY),
        ("[5e-324, 1e-323].quantile(0.5)", 5e-324),
        ("[5e-324, 1e-323].median()", 1e-323),
        ("[5e-324, 5e-324].quantile(0.5)", 5e-324),
        ("[5e-324, 5e-324].median()", 5e-324),
        ("1 / [-0, 0].quantile(0.5)", f64::NEG_INFINITY),
        ("1 / [-0, 0].median()", f64::INFINITY),
        ("1 / [0, -0].quantile(0.5)", f64::INFINITY),
        ("1 / [0, -0].median()", f64::INFINITY),
        ("1 / [-0, -0].quantile(0.5)", f64::NEG_INFINITY),
        ("1 / [-0, -0].median()", f64::NEG_INFINITY),
        ("[0.1, 0.7].quantile(0.5)", 0.39999999999999997),
        ("[0.1, 0.7].median()", 0.39999999999999997),
        ("[4, 1, 3, 2].quantile(0.5)", 2.5),
        ("[4, 1, 3, 2].median()", 2.5),
    ] {
        assert!(
            same_number(number(&mut runtime, source), expected),
            "{source}"
        );
    }
}

#[test]
fn declarations_follow_list_mutations_and_property_changes() {
    behaves_as_spelled(
        &[
            (
                "xs = [{n: \"a\", k: 3}, {n: \"b\", k: 1}, {n: \"c\", k: 7}]",
                "xs = [{n: \"a\", k: 3}, {n: \"b\", k: 1}, {n: \"c\", k: 7}]",
            ),
            ("factor = 2", "factor = 2"),
            (
                "q = xs.quantile(0.3, x -> x.k * factor)",
                "q = ((s, h) -> (1 - h % 1) * s[h] + h % 1 * s[Math.ceil(h)])(xs.map(x -> x.k * factor).sortBy(v -> v), (xs.length - 1) * 0.3)",
            ),
            ("q", "q"),
            ("xs.push({n: \"d\", k: 8})", "xs.push({n: \"d\", k: 8})"),
            ("q", "q"),
            ("factor = 3", "factor = 3"),
            ("q", "q"),
            ("xs.pop()", "xs.pop()"),
            ("xs.shift()", "xs.shift()"),
            ("q", "q"),
            (
                "xs = [{n: \"e\", k: 0.5}, {n: \"f\", k: 6}]",
                "xs = [{n: \"e\", k: 0.5}, {n: \"f\", k: 6}]",
            ),
            ("xs[0].k = 9", "xs[0].k = 9"),
            ("q", "q"),
        ],
        &["q"],
    );
    assert_eq!(
        result("xs = [2, 9]\nq = xs.quantile(0.3)\nxs = []\nString(q)"),
        "\"NaN\""
    );
    assert_eq!(
        result(&format!(
            "xs = [2, 9]\nq = {}\nxs = []\nString(q)",
            spelled_quantile("xs", "0.3")
        )),
        "\"null\""
    );
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "scores = [2, 4, 4, 4, 5, 5, 7, 9]\np90 = scores.quantile(0.9)\nlow = scores.quantile(0.25)\n[p90, low]",
            "[7.6,4.0]",
        ),
        ("scores.push(20)\n[p90, low]", "[11.200000000000003,4.0]"),
        ("scores.pop()\n[p90, low]", "[7.6,4.0]"),
        ("scores.shift()\n[p90, low]", "[7.800000000000001,4.0]"),
        ("scores.push(2)\n[p90, low]", "[7.6,4.0]"),
        ("scores = [6]\n[p90, low]", "[6.0,6.0]"),
        ("scores = [1, 3]\n[p90, low]", "[2.8000000000000003,1.5]"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn declarations_follow_a_changed_rank() {
    behaves_as_spelled(
        &[
            (
                "levels = [4, 8, 15, 16, 23, 42]",
                "levels = [4, 8, 15, 16, 23, 42]",
            ),
            ("rank = 0.3", "rank = 0.3"),
            (
                "q = levels.quantile(rank)",
                "q = ((s, h) -> (1 - h % 1) * s[h] + h % 1 * s[Math.ceil(h)])(levels.sortBy(v -> v), (levels.length - 1) * rank)",
            ),
            ("q", "q"),
            ("rank = 0.8", "rank = 0.8"),
            ("q", "q"),
            ("levels.push(50)", "levels.push(50)"),
            ("q", "q"),
            ("rank = 0", "rank = 0"),
            ("q", "q"),
        ],
        &["q"],
    );
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "levels = [4, 8, 15, 16, 23, 42]\nrank = 0.25\nq = levels.quantile(rank)\nq",
            "9.75",
        ),
        ("rank = 0.75\nq", "21.25"),
        ("rank = 1\nq", "42.0"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    assert_eq!(
        outcome(&mut runtime, "rank = -1"),
        "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(json(&mut runtime, "[rank, q]"), "[1.0,42.0]");
}

#[test]
fn declarations_follow_new_and_deleted_instances_and_class_level_rules() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class R(n, t):\n    this.n = n\n    this.t = t\nupper = R.quantile(0.75, r -> r.t)\nString(upper)",
            "\"NaN\"",
        ),
        ("a = R(\"a\", 3)\nb = R(\"b\", 5)\nupper", "4.5"),
        ("c = R(\"c\", 10)\nupper", "7.5"),
        ("b.t = 4\nupper", "7.0"),
        ("delete c.t\nupper", "null"),
        ("delete c.n\ndelete c\nupper", "3.75"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class Room(name):\n    this.name = name\nclass Sensor(room, temp):\n    this.room = room\n    this.temp = temp\n$Room.high = Sensor.filter(sensor -> sensor.room == $Room).quantile(0.75, sensor -> sensor.temp)\nlab = Room(\"lab\")\nhall = Room(\"hall\")\nSensor(lab, 18)\nprobe = Sensor(lab, 22)\nSensor(hall, 20)\nSensor(hall, 26)\n[lab.high, hall.high]",
            "[21.0,24.5]",
        ),
        ("Sensor(lab, 20)\nlab.high", "21.0"),
        ("probe.temp = 30\nlab.high", "25.0"),
        ("probe.room = hall\n[lab.high, hall.high]", "[19.5,28.0]"),
        ("attic = Room(\"attic\")\nString(attic.high)", "\"NaN\""),
        ("Sensor(attic, 15)\nattic.high", "15.0"),
        (
            "levels = Room.quantile(0.5, room -> room.high)\nlevels",
            "19.5",
        ),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "class Machine(readings):\n    this.readings = readings\n$Machine.high = $Machine.readings.quantile(0.9)\npress = Machine([50, 54, 46])\nlathe = Machine([30])\npress.readings.push(50)\nlathe.readings = [1, 3]\n[press.high, lathe.high]",
            "[52.800000000000004,2.8000000000000003]",
        ),
        (
            "class P(x):\n    this.x = x\n$P.reach = P.quantile(1, other -> other.x - $P.x)\np = P(1)\nq = P(2)\nr = P(6)\n[p.reach, q.reach, r.reach]",
            "[5.0,4.0,0.0]",
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
            "times = [1, 3]\np90 = times.quantile(0.9)\n(why p90).map(step -> step.rule)"
        ),
        "[\"p90 = times.quantile(0.9)\",\"times = [1,3]\"]"
    );
    assert_eq!(json(&mut runtime, "(why p90)[0].from"), "[\"times\"]");
    assert_eq!(
        json(&mut runtime, "(affects times).map(step -> step.node)"),
        "[\"p90\"]"
    );
    assert_eq!(dependencies(&runtime, "p90"), ["times"]);
    assert_eq!(
        json(
            &mut runtime,
            "rank = 0.5\nmid = times.quantile(rank)\n[(why mid)[0].from, (affects rank).map(step -> step.node)]"
        ),
        "[[\"times\",\"rank\"],[\"mid\"]]"
    );
    assert_eq!(
        json(
            &mut runtime,
            "rates = [0.5]\nlow = rates.quantile(0.1, rate -> rate * 2)\n(why low).map(step -> step.rule)"
        ),
        "[\"low = rates.quantile(0.1,function())\",\"rates = [0.5]\"]"
    );
    let mut runtime = Runtime::new();
    runtime
        .run("class Bid(amount):\n    this.amount = amount\nbook = Bid.quantile(0.5, bid -> bid.amount)\nBid(5)")
        .unwrap();
    assert_eq!(dependencies(&runtime, "book"), ["$Bid", "obj:0.amount"]);
    let mut spelled = Runtime::new();
    assert_eq!(
        json(
            &mut spelled,
            "times = [1, 3]\np90 = ((s, h) -> (1 - h % 1) * s[h] + h % 1 * s[Math.ceil(h)])(times.sortBy(v -> v), (times.length - 1) * 0.9)\n(why p90).map(step -> step.rule)"
        ),
        "[\"p90 = function()(times.sortBy(function()),times.length-1*0.9)\",\"times = [1,3]\"]"
    );
    assert_eq!(dependencies(&spelled, "p90"), ["times"]);
    assert_eq!(
        result(
            "def key(x) => x.k\nr = [].quantile(0.5, key)\n(affects key).map(step -> step.node)"
        ),
        "[\"r\"]"
    );
    for (source, message) in [
        (
            "xs = [1, 2]\nxs.quantile(0.5)(1)",
            "xs.quantile(0.5) is not a function",
        ),
        (
            "xs = [1, 2]\nall x in xs.quantile(0.5): x",
            "xs.quantile(0.5) is not a collection",
        ),
        (
            "xs = [1, 2]\nwhy xs.quantile(0.5)",
            "Cannot explain xs.quantile(0.5)",
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
fn snapshots_hypotheticals_templates_and_conditions_read_quantiles() {
    for (source, expected) in [
        (
            "xs = [1, 3]\nq = xs.quantile(0.5)\nfrozen = q.value\nxs.push(5)\n[q, frozen]",
            "[3.0,2.0]",
        ),
        (
            "xs = [1, 3]\nfrozen = xs.value.quantile(0.5)\nxs.push(5)\nfrozen",
            "2.0",
        ),
        (
            "xs = [1, 3]\nq = xs.quantile(0.5)\nseen = suppose xs.push(5): q\n[seen, q]",
            "[3.0,2.0]",
        ),
        (
            "xs = [1, 3]\nlabel = `${xs.quantile(1)} high`\nxs.push(5)\nlabel",
            "\"5 high\"",
        ),
        (
            "xs = [1, 3]\nif xs.quantile(0.5) > 1.5:\n    flag = true\nflag",
            "true",
        ),
        (
            "xs = [1, 3]\nsize = xs.quantile(0.9) > 2.5 ? \"wide\" : \"narrow\"\nxs.push(9)\nsize",
            "\"wide\"",
        ),
        (
            "xs = [1, 3]\nonce := xs.quantile(0.5)\nxs.push(5)\n[once, xs.quantile(0.5)]",
            "[2.0,3.0]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn failed_statements_and_transactions_restore_quantiles() {
    behaves_as_spelled(
        &[
            ("rows = [{k: 2}, {k: 4}]", "rows = [{k: 2}, {k: 4}]"),
            (
                "q = rows.quantile(0.25, row -> row.k > 9 ? row.k.toUpperCase() : row.k)",
                "q = ((s, h) -> (1 - h % 1) * s[h] + h % 1 * s[Math.ceil(h)])(rows.map(row -> row.k > 9 ? row.k.toUpperCase() : row.k).sortBy(v -> v), (rows.length - 1) * 0.25)",
            ),
            (
                "try:\n    rows.push({k: 6})\n    assert(q, 3)\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
                "try:\n    rows.push({k: 6})\n    assert(q, 3)\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
            ),
            ("[rows.length, q]", "[rows.length, q]"),
            ("rows.push({k: 10})", "rows.push({k: 10})"),
            ("[rows.length, q]", "[rows.length, q]"),
            ("rows.push({k: 6})", "rows.push({k: 6})"),
            ("q", "q"),
        ],
        &["q"],
    );
    let mut runtime = Runtime::new();
    runtime
        .run("rows = [{k: 2}, {k: 4}]\nq = rows.quantile(0.5, row -> row.k > 9 ? row.k.toUpperCase() : row.k)")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "rows.push({k: 10})"),
        "TypeError: number.toUpperCase is not a function at Some(Position { line: 1, column: 1 })"
    );
    assert_eq!(json(&mut runtime, "[rows.length, q]"), "[2.0,3.0]");
    assert_eq!(
        result(
            "class Probe(t):\n    this.t = t\nProbe(2)\nProbe(4)\nheat = Probe.quantile(0.5, p -> p.t > 9 ? p.t.toUpperCase() : p.t)\nfailure = \"\"\ntry:\n    Probe(50)\ncatch error:\n    failure = error\n[failure, Probe.length, heat]"
        ),
        "[\"TypeError: number.toUpperCase is not a function\",2.0,3.0]"
    );
    assert_eq!(
        result(
            "xs = [1, 3]\nq = xs.quantile(0.5)\ntry:\n    xs.push(5)\n    throw \"UNDO\"\ncatch error:\n    pass\nq"
        ),
        "2.0"
    );
}

#[test]
fn large_receivers_take_quantiles_through_the_public_api() {
    let mut runtime = Runtime::new();
    runtime
        .run("holder = {items: 0}\ncounter = {n: 0}\ndef tick(x):\n    counter.n = counter.n + 1\n    return x")
        .unwrap();
    let count = 100_000_u32;
    hold(
        &mut runtime,
        Value::List(
            (0..count)
                .map(|position| Value::Number(if position % 2 == 0 { 3.0 } else { 1.0 }))
                .collect(),
        ),
    );
    assert_eq!(json(&mut runtime, "holder.items.quantile(0.5)"), "2.0");
    assert_eq!(json(&mut runtime, "holder.items.quantile(0.25)"), "1.0");
    assert_eq!(json(&mut runtime, "holder.items.quantile(0.9)"), "3.0");
    assert_eq!(
        json(
            &mut runtime,
            "[holder.items.quantile(0.5, tick), counter.n]"
        ),
        "[2.0,100000.0]"
    );
    assert_eq!(json(&mut runtime, "holder.items.length"), "100000.0");
    assert_eq!(json(&mut runtime, "holder.items[0]"), "3.0");
}

#[test]
fn sums_averages_medians_and_variances_keep_their_results_beside_quantiles() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("[].sum()", 0.0),
        ("[].product()", 1.0),
        ("[0.1, 0.2, 0.3].sum()", 0.6000000000000001),
        ("[0.1, 0.2, 0.3].average()", 0.20000000000000004),
        ("[0.1, 0.2, 0.3].variance()", 0.006666666666666665),
        ("[0.1, 0.2, 0.3].stdev()", 0.0816496580927726),
        ("[2, \"3\", true].sum()", 6.0),
        ("[1, 2].average()", 1.5),
        ("[4, 1, 3, 2].median()", 2.5),
        ("[2, 3, 4].product()", 24.0),
        ("[5, 1, 5].mode()", 5.0),
        ("[5, 1, 9].max()", 9.0),
        ("[5, 1, 9].min()", 1.0),
    ] {
        assert!(
            same_number(number(&mut runtime, source), expected),
            "{source}"
        );
    }
    assert!(number(&mut runtime, "[].average()").is_nan());
    assert!(number(&mut runtime, "[].median()").is_nan());
    assert!(number(&mut runtime, "[].variance()").is_nan());
    for (source, expected) in [
        ("o = {}\nString([1, o.missing].median())", "\"undefined\""),
        ("t = [1, null].sum()\nt", "1.0"),
        ("xs = [1, 2]\ntotal = xs.median()\nxs.push(3)\ntotal", "2.0"),
        ("[1, 3].distinct()", "[1.0,3.0]"),
        ("[3, 1, 2].sortBy(v -> v)", "[1.0,2.0,3.0]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn the_compatibility_rows_give_their_documented_results() {
    let orders = "class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)\nOrder(12)\n";
    let team = "class Team(name):\n    this.name = name\n    def quantile(q): return q * 10\nred = Team(\"red\")\n";
    let calculator = "class Calculator(base):\n    this.base = base\n    def quantile(a, b) => this.base + a * b\nc = Calculator(1)\n";
    let xs = "xs = [2, 4, 4, 4, 5, 5, 7, 9]\n";
    for (source, expected) in [
        ("[1, 2, 3, 4].quantile(0.5)".to_string(), "2.5"),
        ("[1, 2, 3, 4].quantile(0.25)".to_string(), "1.75"),
        ("String([].quantile(0.5))".to_string(), "\"NaN\""),
        ("[5].quantile(0.3)".to_string(), "5.0"),
        (format!("{xs}xs.quantile(0.9)"), "7.6"),
        (
            format!("{xs}[xs.quantile(0), xs.quantile(0.25), xs.quantile(1)]"),
            "[2.0,4.0,9.0]",
        ),
        (format!("{xs}xs.quantile(0.25, x -> x * 2)"), "8.0"),
        ("o = {items: [1, 3]}\no.items.quantile(0.5)".to_string(), "2.0"),
        ("xs = [1, 3]\nxs.map(x -> x * 2).quantile(0.5)".to_string(), "4.0"),
        (
            "xs = [1, 3, 9]\nxs.filter(x -> x < 5).quantile(0.5)".to_string(),
            "2.0",
        ),
        ("xs = [1, 3]\nxs.value.quantile(0.5)".to_string(), "2.0"),
        ("xs = [1, 3]\nxs.push(5)\nxs@first.quantile(0.5)".to_string(), "2.0"),
        ("xs = [1, 3, 5]\nxs[1:].quantile(0.5)".to_string(), "4.0"),
        ("xs = [1, 3, 5]\nxs.slice(0, 2).quantile(0.5)".to_string(), "2.0"),
        ("xs = [1]\nxs.concat([3]).quantile(0.5)".to_string(), "2.0"),
        ("grid = [[1, 3], [4]]\ngrid[0].quantile(0.5)".to_string(), "2.0"),
        ("def make() => [1, 3, 5]\nmake().quantile(0.75)".to_string(), "4.0"),
        (format!("{orders}Order.quantile(0.5, o -> o.total)"), "7.0"),
        (format!("{orders}Order.quantile(0.9, .total)"), "11.000000000000002"),
        (format!("{orders}Order[1:].quantile(0.5, o -> o.total)"), "9.5"),
        (
            format!("{orders}Order.list().quantile(0.25, o -> o.total)"),
            "6.0",
        ),
        (
            format!("{orders}kind = Order\nkind.quantile(0.5, o -> o.total)"),
            "7.0",
        ),
        (format!("{orders}$Order.quantile(0.5, o -> o.total)"), "7.0"),
        (
            format!("{orders}Order.filter(o -> o.total > 6).quantile(0.5, o -> o.total)"),
            "9.5",
        ),
        (
            "class Empty\nString(Empty.quantile(0.5))".to_string(),
            "\"NaN\"",
        ),
        (
            "class Box(sides):\n    this.sides = sides\n$Box.mid = $Box.sides.quantile(0.5)\nb = Box([1, 3])\nb.mid".to_string(),
            "2.0",
        ),
        (format!("{team}String(Team.quantile(0.5))"), "\"NaN\""),
        (format!("{team}Team.quantile(0.5, t -> t.quantile(0.5))"), "5.0"),
        (
            format!("{calculator}Calculator.quantile(2, 3)"),
            "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 5, column: 1 })",
        ),
        ("[2, \"4\", true, null].quantile(0.5)".to_string(), "1.5"),
        ("[[2], [4]].quantile(0.5)".to_string(), "3.0"),
        ("String([1, \"a\"].quantile(0.5))".to_string(), "\"NaN\""),
        ("big = [1e308, 1.5e308]\nbig.quantile(0.5)".to_string(), "1.25e+308"),
        (
            "zs = [1, Number.POSITIVE_INFINITY]\n[zs.quantile(0), String(zs.quantile(0.5)), String(zs.quantile(1))]".to_string(),
            "[1.0,\"Infinity\",\"Infinity\"]",
        ),
        (
            "zs = [Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY]\nString(zs.quantile(0.5))".to_string(),
            "\"NaN\"",
        ),
        (
            "o = {}\nString([1, o.missing].quantile(0.5))".to_string(),
            "\"undefined\"",
        ),
        ("xs = [3, 1, 2]\n1 / xs.quantile(-0)".to_string(), "1.0"),
        ("xs = [1, 3]\nxs.quantile(0.5, x -> x, 5)".to_string(), "2.0"),
        ("[3, 3].quantile(0.01)".to_string(), "3.0"),
        (
            "a = [-0, 0]\n[String(1 / a.quantile(0.5)), String(1 / a.median())]".to_string(),
            "[\"-Infinity\",\"Infinity\"]",
        ),
        (
            "a = [0, -0]\n[String(1 / a.quantile(0.5)), String(1 / a.median())]".to_string(),
            "[\"Infinity\",\"Infinity\"]",
        ),
        (
            "a = [1e308, 1e308]\n[a.quantile(0.5), String(a.median())]".to_string(),
            "[1e+308,\"Infinity\"]",
        ),
        (
            "a = [5e-324, 1e-323]\n[a.quantile(0.5), a.median()]".to_string(),
            "[5e-324,1e-323]",
        ),
        (
            "xs = [1, 3]\nxs.quantile()".to_string(),
            "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [1, 3]\nxs.quantile(1.5)".to_string(),
            "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [1, 3]\nxs.quantile(-0.1)".to_string(),
            "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [1, 3]\nxs.quantile(\"0.5\")".to_string(),
            "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [1, 3]\nxs.quantile(null)".to_string(),
            "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [1, 3]\nxs.quantile(true)".to_string(),
            "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [1, 3]\nxs.quantile(0 / 0)".to_string(),
            "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [1, 3]\nxs.quantile([0.5])".to_string(),
            "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 2, column: 1 })",
        ),
        (
            "[].quantile(2)".to_string(),
            "TypeError: quantile expects a number from 0 to 1 at Some(Position { line: 1, column: 1 })",
        ),
        (
            "xs = [1, 3]\nxs.quantile(0.5, 1)".to_string(),
            "TypeError: quantile expects a function at Some(Position { line: 2, column: 1 })",
        ),
        (
            "[].quantile(0.5, null)".to_string(),
            "TypeError: quantile expects a function at Some(Position { line: 1, column: 1 })",
        ),
        (
            format!("{orders}Order.quantile(0.5, \"total\")"),
            "TypeError: quantile expects a function at Some(Position { line: 6, column: 1 })",
        ),
        (
            "xs = [1, 3]\nxs.quantile(0.5)(1)".to_string(),
            "TypeError: xs.quantile(0.5) is not a function at Some(Position { line: 2, column: 1 })",
        ),
        (
            "xs = [1, 3]\nall t in xs.quantile(0.5): t".to_string(),
            "TypeError: xs.quantile(0.5) is not a collection at Some(Position { line: 2, column: 1 })",
        ),
        (
            "def scaled(xs, k) => xs.quantile(0.5, x -> x * k)\nscaled([1, 3], 3)".to_string(),
            "ReferenceError: k is not defined at Some(Position { line: 2, column: 1 })",
        ),
        (
            "def scaled(xs, k) => xs.median(x -> x * k)\nscaled([1, 3], 3)".to_string(),
            "ReferenceError: k is not defined at Some(Position { line: 2, column: 1 })",
        ),
        ("quantile = [1, 3]\nquantile.quantile(0.5)".to_string(), "2.0"),
    ] {
        assert_eq!(outcome(&mut Runtime::new(), &source), expected, "{source}");
    }
}

#[test]
fn unchanged_readings_keep_their_accepted_results() {
    for (source, expected) in [
        ("xs = [1]\nxs.quantile", "null"),
        ("class Order\nOrder.quantile", "null"),
        ("xs = [1]\ntypeof xs.quantile", "\"undefined\""),
        ("o = {quantile: 3}\no.quantile", "3.0"),
        ("x = {quantile: 4}.quantile\nx", "4.0"),
        ("o = {quantile: n -> n * 2}\no.quantile(4)", "8.0"),
        ("o = {}\no.quantile(0.5)", "null"),
        ("class P\np = P()\np.quantile(0.5)", "null"),
        (
            "class Calculator:\n    def quantile(a, b): return a + b\ncalc = Calculator()\n[calc.quantile(2, 3), calc.quantiles]",
            "[5.0,null]",
        ),
        (
            "class Team(name, scores):\n    this.name = name\n    this.scores = scores\n    def quantile(q): return q * 10\nred = Team(\"red\", [1, 2, 3])\nred.quantile(0.5)",
            "5.0",
        ),
        (
            "def quantile(values, q) => values.length * q\nquantile([1, 2], 0.5)",
            "1.0",
        ),
        (
            "quantile = [1, 2]\nq = quantile.length\n[quantile, q]",
            "[[1.0,2.0],2.0]",
        ),
        ("def g(quantile) => quantile * 2\ng(4)", "8.0"),
        ("f = lambda quantile: quantile + 1\nf(4)", "5.0"),
        (
            "class S(quantile):\n    this.quantile = quantile\ns = S([1, 2])\ns.quantile",
            "[1.0,2.0]",
        ),
        ("n = null\nn.quantile(0.5)", "null"),
        ("o = {}\no.items.quantile(0.5)", "null"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, kind, message) in [
        (
            "\"abc\".quantile(0.5)",
            ErrorKind::Type,
            "abc.quantile is not a function",
        ),
        (
            "(5).quantile(0.5)",
            ErrorKind::Type,
            "number.quantile is not a function",
        ),
        (
            "true.quantile(0.5)",
            ErrorKind::Type,
            "boolean.quantile is not a function",
        ),
        (
            "d = Date(0)\nd.quantile(0.5)",
            ErrorKind::Type,
            "Date.quantile is not a function",
        ),
        (
            "def f() => 1\nf.quantile(0.5)",
            ErrorKind::Type,
            "function.quantile is not a function",
        ),
        (
            "/a/.quantile(0.5)",
            ErrorKind::Type,
            "quantile is not a function",
        ),
        (
            "Math.quantile([1, 2], 0.5)",
            ErrorKind::Type,
            "Math.quantile is not a function",
        ),
        (
            "quantile([1, 2], 0.5)",
            ErrorKind::Reference,
            "quantile is not defined",
        ),
        (
            "xs = [2, 4, 4, 4, 5, 5, 7, 9]\nxs.percentile(50)",
            ErrorKind::Type,
            "xs.percentile is not a function",
        ),
        (
            "xs = [2, 4, 4, 4, 5, 5, 7, 9]\nxs.quantiles(4)",
            ErrorKind::Type,
            "xs.quantiles is not a function",
        ),
        (
            "[1, 2].median(0.5)",
            ErrorKind::Type,
            "median expects a function",
        ),
        (
            "xs = [2, 4, 4, 4, 5, 5, 7, 9]\nquantile of xs",
            ErrorKind::Syntax,
            "Unexpected keyword 'of'",
        ),
        (
            "xs = [1, 3]\nwhy xs.quantile(0.5)",
            ErrorKind::Type,
            "Cannot explain xs.quantile(0.5)",
        ),
        (
            "xs = [1, 2]\nxs.quantile(missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
    ] {
        assert_eq!(failure(source), (kind, message.to_string()), "{source}");
    }
    let error = Runtime::new()
        .run("xs = [2, 4, 4, 4, 5, 5, 7, 9]\nquantile of xs")
        .unwrap_err();
    assert_eq!(
        format!("{:?}", error.position()),
        "Some(Position { line: 2, column: 13 })"
    );
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    assert_eq!(
        result(
            "xs = [1, 3]\nr = 0\ntry:\n    r = xs.quantile(0.5)\ncatch error:\n    r = error\nr"
        ),
        "2.0"
    );
    assert_eq!(
        result("xs = [1, 3]\nr = 0\nif xs.quantile(0.5) > 1.5:\n    r = 1\nr"),
        "1.0"
    );
    assert_eq!(
        result(
            "xs = [1, 3]\nr = 0\ntry:\n    r = xs.quantiles(0.5)\ncatch error:\n    r = error\nr"
        ),
        "\"TypeError: xs.quantiles is not a function\""
    );
    assert_eq!(
        result("xs = [1, 3]\nr = 0\ntry:\n    r = xs.quantile(5)\ncatch error:\n    r = error\nr"),
        "\"TypeError: quantile expects a number from 0 to 1\""
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-list-quantiles-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_quantile() {
    let program = "scores = [2, 4, 4, 4, 5, 5, 7, 9]\np90 = scores.quantile(0.9)\nscores.push(20)\n[p90, scores.quantile(0.25)]";
    let file = SourceFile::new("file");
    let measured = file.run(program, &["--json"]);
    assert!(measured.status.success(), "{measured:?}");
    assert!(measured.stderr.is_empty(), "{measured:?}");
    assert_eq!(
        String::from_utf8(measured.stdout).unwrap(),
        "[11.200000000000003,4.0]\n"
    );
    let plain = file.run(program, &[]);
    assert!(plain.status.success(), "{plain:?}");
    assert_eq!(
        String::from_utf8(plain.stdout).unwrap(),
        "11.200000000000003,4\n"
    );
    let graph = String::from_utf8(
        file.run(
            "scores = [1, 3]\nmid = scores.quantile(0.5)\nmid",
            &["--json", "--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        graph.contains(
            "  scores [variable]\n    updates  mid\n  mid [variable]\n    reads    scores"
        ),
        "{graph}"
    );
    let block = String::from_utf8(
        file.run(
            "xs = [1, 3]\nif xs.quantile(0.5) > 1.5:\n    flag = true\nflag",
            &["--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        block.contains("  xs [variable]\n    updates  if(xs.quantile(0.5)>1.5)\n  if(xs.quantile(0.5)>1.5) [if]\n    reads    xs"),
        "{block}"
    );
    for (source, message) in [
        (
            "xs = [1]\nxs.quantile(5)",
            "TypeError: quantile expects a number from 0 to 1 (line 2, column 1)\n",
        ),
        (
            "xs = [1]\nxs.quantile(0.5, 5)",
            "TypeError: quantile expects a function (line 2, column 1)\n",
        ),
    ] {
        let failed = file.run(source, &["--json"]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(String::from_utf8(failed.stderr).unwrap(), message);
    }
    let session = "xs = [1, 3]\nq = xs.quantile(0.5)\nq\nxs.push(5)\nq\nxs\nxs.quantile(2)\n.clear\nxs = [2, 6]\nxs.quantile(0.25, x -> x + 1)\n.exit\n";
    for (options, expected) in [
        (
            &["--plain"][..],
            "1,3\n2\n2\n3\n3\n1,3,5\nState cleared.\n2,6\n4\n",
        ),
        (
            &["--plain", "--json"][..],
            "[1.0,3.0]\n2.0\n2.0\n3.0\n3.0\n[1.0,3.0,5.0]\nState cleared.\n[2.0,6.0]\n4.0\n",
        ),
        (
            &["--json"][..],
            "[1.0,3.0]\n2.0\n2.0\n3.0\n3.0\n[1.0,3.0,5.0]\nState cleared.\n[2.0,6.0]\n4.0\n",
        ),
    ] {
        for newline in ["\n", "\r\n"] {
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.ends_with(expected), "{options:?} {stdout}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                "TypeError: quantile expects a number from 0 to 1 (line 1, column 1)\n"
            );
        }
    }
}
