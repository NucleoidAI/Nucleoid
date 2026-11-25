mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Ast, Expr, Stmt, UnaryOp, parse, parse_expression};
use nucleoid::{ErrorKind, Runtime, Value};
use serde::Deserialize;

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid takes every other item of a list with stepped slices",
    "Nucleoid reverses strings by character with stepped slices",
    "Nucleoid strides class instance populations with stepped slices",
    "Nucleoid rolls back failed updates through stepped slices",
];
const REFERENCE_TITLE: &str = "Nucleoid walks lists from a position with stepped slices";
const README_TITLE: &str = "Stepped slices deal the chores round-robin as roommates move in";
const EXAMPLES_TITLE: &str = "Stepped slices mirror the rows of every sprite";

const ACCEPTED: &str = r######"[
  {
    "document": "spec",
    "title": "Nucleoid takes every other item of a list with stepped slices",
    "source": "# Nucleoid takes every other item of a list with stepped slices\n\nsamples = [12, 15, 17, 11, 14, 16]\nleft = samples.filter(s -> samples.indexOf(s) % 2 == 0)\nright = samples.filter(s -> s not in left)\nassert([left, right], [[12, 17, 14], [15, 11, 16]])\nsamples.push(13)\nassert([left, right], [[12, 17, 14, 13], [15, 11, 16]])\nsamples.shift()\nassert([left, right], [[15, 11, 16], [17, 14, 13]])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid reverses strings by character with stepped slices",
    "source": "# Nucleoid reverses strings by character with stepped slices\n\nword = \"añejo\"\nmirror = word.split(\"\").reverse().join()\npalindrome = mirror == word\nassert([mirror, palindrome], [\"ojeña\", false])\nword = \"reconocer\"\nassert([mirror, palindrome], [\"reconocer\", true])\nword = \"Øresund\"\nassert([mirror, palindrome], [\"dnuserØ\", false])\nword = \"über\"\nassert([mirror, palindrome], [\"rebü\", false])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid strides class instance populations with stepped slices",
    "source": "# Nucleoid strides class instance populations with stepped slices\n\nclass Lap(seconds):\n    this.seconds = seconds\nLap(62)\nLap(58)\nLap(57)\nLap(61)\nstride = 2\nsampled = Lap.filter(lap -> Lap.indexOf(lap) % stride == 0).map(lap -> lap.seconds)\nassert(sampled, [62, 57])\nLap(60)\nassert(sampled, [62, 57, 60])\nstride = 3\nassert(sampled, [62, 61])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back failed updates through stepped slices",
    "source": "# Nucleoid rolls back failed updates through stepped slices\n\ndef checked(amount):\n    if amount < 0:\n        throw \"NEGATIVE\"\n    return amount\nprices = [4, 9, 7, 12]\npicked = prices.filter(p -> prices.indexOf(p) % 2 == 0).map(checked)\nfailure = \"\"\ntry:\n    prices.push(-3)\ncatch error:\n    failure = error\nassert([failure, prices, picked], [\"NEGATIVE\", [4, 9, 7, 12], [4, 7]])\ntry:\n    prices.push(5)\n    assert(picked, [4, 7, 5])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([prices, picked], [[4, 9, 7, 12], [4, 7]])\nclass Crate(weight): this.weight = weight\nCrate(6)\nCrate(2)\nCrate(9)\nlatest = Crate.reverse().filter(c -> Crate.reverse().indexOf(c) % 2 == 0).map(crate -> checked(crate.weight))\nfailure = \"\"\ntry:\n    Crate(-1)\ncatch error:\n    failure = error\nassert([failure, Crate.length, latest], [\"NEGATIVE\", 3, [9, 6]])\nCrate(5)\nlatest",
    "expected": "[5,2]"
  },
  {
    "document": "reference",
    "title": "Nucleoid walks lists from a position with stepped slices",
    "source": "# Nucleoid walks lists from a position with stepped slices\n\nroute = [\"Harbor\", \"Mill\", \"Oak\", \"Central\", \"Park\", \"Airport\"]\nhere = 2\nahead = route[here:].filter(s -> route.indexOf(s) % 2 == here % 2)\nback = route[:here + 1].reverse()\nassert([ahead, back], [[\"Oak\", \"Park\"], [\"Oak\", \"Mill\", \"Harbor\"]])\nhere = 3\nassert([ahead, back], [[\"Central\", \"Airport\"], [\"Central\", \"Oak\", \"Mill\", \"Harbor\"]])\nroute.shift()\n[ahead, back]",
    "expected": "[[\"Park\"],[\"Park\",\"Central\",\"Oak\",\"Mill\"]]"
  },
  {
    "document": "readme",
    "title": "Stepped slices deal the chores round-robin as roommates move in",
    "source": "chores = [\"dishes\", \"laundry\", \"trash\", \"floors\", \"plants\", \"mail\"]\nclass Roommate(turn):\n    this.turn = turn\n$Roommate.duties = chores.filter(c -> chores.indexOf(c) % Roommate.length == $Roommate.turn)\nana = Roommate(0)\nben = Roommate(1)\nassert([ana.duties, ben.duties], [[\"dishes\", \"trash\", \"plants\"], [\"laundry\", \"floors\", \"mail\"]])\ncy = Roommate(2)\nchores.push(\"cooking\")\nassert([ana.duties, cy.duties], [[\"dishes\", \"floors\", \"cooking\"], [\"trash\", \"mail\"]])\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Stepped slices mirror the rows of every sprite",
    "source": "class Sprite(rows):\n    this.rows = rows\n$Sprite.mirrored = $Sprite.rows.map(row -> row.split(\"\").reverse().join())\n$Sprite.symmetric = $Sprite.mirrored == $Sprite.rows\narrow = Sprite([\"#..\", \"##.\", \"#..\"])\nheart = Sprite([\".#.#.\", \"#####\", \".###.\", \"..#..\"])\nassert([arrow.mirrored, arrow.symmetric, heart.symmetric], [[\"..#\", \".##\", \"..#\"], false, true])\nheart.rows.push(\"#....\")\n[heart.mirrored, heart.symmetric]\n",
    "expected": "[[\".#.#.\",\"#####\",\".###.\",\"..#..\",\"....#\"],false]"
  }
]"######;

const WRITTEN_OUT: [(&str, &str); 10] = [
    (
        "samples[::2]",
        "samples.filter(s -> samples.indexOf(s) % 2 == 0)",
    ),
    ("samples[1::2]", "samples.filter(s -> s not in left)"),
    ("word[::-1]", "word.split(\"\").reverse().join()"),
    (
        "Lap[::stride]",
        "Lap.filter(lap -> Lap.indexOf(lap) % stride == 0)",
    ),
    (
        "prices[::2]",
        "prices.filter(p -> prices.indexOf(p) % 2 == 0)",
    ),
    (
        "Crate[::-2]",
        "Crate.reverse().filter(c -> Crate.reverse().indexOf(c) % 2 == 0)",
    ),
    (
        "route[here::2]",
        "route[here:].filter(s -> route.indexOf(s) % 2 == here % 2)",
    ),
    ("route[here::-1]", "route[:here + 1].reverse()"),
    (
        "chores[$Roommate.turn::Roommate.length]",
        "chores.filter(c -> chores.indexOf(c) % Roommate.length == $Roommate.turn)",
    ),
    ("row[::-1]", "row.split(\"\").reverse().join()"),
];

const STEP_ERROR: &str = "Slice step must be a nonzero number";

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

fn stepped_slices(source: &str) -> usize {
    let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    format!("{statements:?}").matches("SteppedSlice {").count()
}

fn written_out(case: &common::Case) -> (String, usize) {
    let mut written = case.source.clone();
    let mut rewrites = 0;
    for (slice, spelling) in WRITTEN_OUT {
        rewrites += written.matches(slice).count();
        written = written.replace(slice, spelling);
    }
    (written, rewrites)
}

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "stepped slice regression".to_string(),
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
    let mut renamed = String::new();
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
        renamed.push_str(before);
        renamed.push_str("obj:");
        renamed.push_str(&ordinal.to_string());
        rest = tail;
    }
    renamed.push_str(rest);
    renamed
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

fn numbers(text: &str) -> Vec<Vec<f64>> {
    serde_json::from_str(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn python_positions(
    length: i64,
    start: Option<i64>,
    stop: Option<i64>,
    step: Option<i64>,
) -> Vec<i64> {
    let step = step.unwrap_or(1);
    let (lower, upper) = if step < 0 {
        (-1, length - 1)
    } else {
        (0, length)
    };
    let adjust = |bound: i64| {
        if bound < 0 {
            (bound + length).max(lower)
        } else {
            bound.min(upper)
        }
    };
    let mut position = start.map_or(if step < 0 { upper } else { lower }, adjust);
    let stop = stop.map_or(if step < 0 { lower } else { upper }, adjust);
    let mut taken = Vec::new();
    while (step > 0 && position < stop) || (step < 0 && position > stop) {
        taken.push(position);
        position += step;
    }
    taken
}

fn truncated_positions(
    length: usize,
    start: Option<f64>,
    stop: Option<f64>,
    step: f64,
) -> Vec<f64> {
    let size = length as f64;
    let stride = step.trunc();
    let mut taken = Vec::new();
    if stride > 0.0 {
        let bound = |bound: f64| {
            if bound < 0.0 {
                (size + bound).max(0.0).trunc()
            } else if bound.is_nan() {
                0.0
            } else {
                bound.trunc().min(size)
            }
        };
        let mut position = start.map_or(0.0, bound);
        let stop = stop.map_or(size, bound);
        while position < stop {
            taken.push(position);
            position += stride;
        }
    } else {
        let bound = |bound: f64| {
            let counted = if bound < 0.0 { size + bound } else { bound };
            let truncated = if counted.is_nan() {
                0.0
            } else {
                counted.trunc()
            };
            truncated.max(-1.0).min(size - 1.0)
        };
        let mut position = start.map_or(size - 1.0, bound);
        let stop = stop.map_or(-1.0, bound);
        while position > stop {
            taken.push(position);
            position += stride;
        }
    }
    taken
}

fn part(value: Option<i64>) -> String {
    value.map_or_else(String::new, |value| value.to_string())
}

fn float_part(value: Option<f64>) -> String {
    match value {
        None => String::new(),
        Some(value) if value.is_nan() => "(0 / 0)".to_string(),
        Some(value) if value == f64::INFINITY => "Number.POSITIVE_INFINITY".to_string(),
        Some(value) if value == f64::NEG_INFINITY => "Number.NEGATIVE_INFINITY".to_string(),
        Some(value) if value.to_bits() == (-0.0_f64).to_bits() => "(-0)".to_string(),
        Some(value) => format!("({value:?})"),
    }
}

#[test]
fn only_the_appended_cases_write_stepped_slices_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    let mut slices = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        let found = stepped_slices(&case.source);
        assert_eq!(found > 0, new, "{}", case.title);
        slices += found;
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
    assert_eq!(slices, 10);
    assert_eq!(assertions, 20);
    assert_eq!(cases, 1320);
}

#[test]
fn the_appended_cases_stand_after_nuc_124_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(titles[811], "Nucleoid rolls back failed quantile updates");
    assert_eq!(titles[812..816], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid finds the quantiles of lists and populations with quantile",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "The ninetieth percentile keeps the slow tail of response times live",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Ninetieth percentiles flag the routes whose trips run late",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 14 - 9 * usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 6].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Stepped slices\n\n```\n")
        .expect("the language reference documents stepped slices");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 125](nuc-0125.md)"));
    assert!(reference.contains("`TypeError(\"Slice step must be a nonzero number\")`"));
    assert!(reference.contains("[stepped slices](../nucleoid.spec.md#stepped-slices)"));
    assert_eq!(
        case.expected.as_deref(),
        Some("[[\"Park\"],[\"Park\",\"Central\",\"Oak\",\"Mill\"]]")
    );
}

#[test]
fn every_case_is_its_accepted_counterpart_with_only_the_stepped_slices_written_out() {
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
        assert_eq!(stepped_slices(&written), 0, "{}", sealed.title);
        assert_eq!(stepped_slices(&case.source), count, "{}", sealed.title);
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
fn stepped_slices_select_what_pythons_rule_selects() {
    let parts: Vec<Option<i64>> = std::iter::once(None).chain((-9..=9).map(Some)).collect();
    let steps: Vec<Option<i64>> = std::iter::once(None)
        .chain((-9..=9).filter(|step| *step != 0).map(Some))
        .collect();
    let mut compared = 0;
    for length in 0..=7_i64 {
        let items: Vec<String> = (0..length).map(|item| (100 + item).to_string()).collect();
        for step in &steps {
            let mut slices = Vec::new();
            let mut expected = Vec::new();
            for start in &parts {
                for stop in &parts {
                    slices.push(format!(
                        "xs[{}:{}:{}]",
                        part(*start),
                        part(*stop),
                        part(*step)
                    ));
                    expected.push(
                        python_positions(length, *start, *stop, *step)
                            .into_iter()
                            .map(|position| (100 + position) as f64)
                            .collect::<Vec<f64>>(),
                    );
                }
            }
            let source = format!("xs = [{}]\n[{}]", items.join(", "), slices.join(", "));
            assert_eq!(numbers(&result(&source)), expected, "{length} {step:?}");
            compared += expected.len();
        }
    }
    assert_eq!(compared, 8 * 19 * 20 * 20);
}

#[test]
fn strings_and_populations_walk_the_positions_of_pythons_rule() {
    let parts: Vec<Option<i64>> = std::iter::once(None).chain((-6..=6).map(Some)).collect();
    let steps: Vec<Option<i64>> = [
        None,
        Some(1),
        Some(2),
        Some(3),
        Some(-1),
        Some(-2),
        Some(-4),
    ]
    .to_vec();
    let letters = ["a", "ñ", "日", "🙂", "e\u{301}", "z"];
    for length in 0..=letters.len() {
        let text: String = letters[..length].concat();
        let characters: Vec<char> = text.chars().collect();
        let mut population = String::from("class Unit(n):\n    this.n = n\n");
        for item in 0..length {
            population.push_str(&format!("Unit({item})\n"));
        }
        for step in &steps {
            let mut slices = Vec::new();
            let mut classes = Vec::new();
            let mut expected_text = Vec::new();
            let mut expected_units = Vec::new();
            for start in &parts {
                for stop in &parts {
                    let written = format!("[{}:{}:{}]", part(*start), part(*stop), part(*step));
                    slices.push(format!("text{written}"));
                    classes.push(format!("Unit{written}.map(unit -> unit.n)"));
                    let positions = python_positions(characters.len() as i64, *start, *stop, *step);
                    expected_text.push(serde_json::Value::String(
                        positions
                            .iter()
                            .map(|position| characters[*position as usize])
                            .collect(),
                    ));
                    let units = python_positions(length as i64, *start, *stop, *step);
                    expected_units.push(serde_json::json!(
                        units.iter().map(|unit| *unit as f64).collect::<Vec<f64>>()
                    ));
                }
            }
            let source = format!(
                "text = {}\n[{}]",
                serde_json::to_string(&text).unwrap(),
                slices.join(", ")
            );
            let measured: serde_json::Value = serde_json::from_str(&result(&source)).unwrap();
            assert_eq!(
                measured,
                serde_json::Value::Array(expected_text),
                "{text} {step:?}"
            );
            let source = format!("{population}[{}]", classes.join(", "));
            let measured: serde_json::Value = serde_json::from_str(&result(&source)).unwrap();
            assert_eq!(
                measured,
                serde_json::Value::Array(expected_units),
                "{length} {step:?}"
            );
        }
    }
}

#[test]
fn fractional_nan_and_infinite_bounds_follow_the_truncation_rule() {
    let bounds = [
        None,
        Some(-7.5),
        Some(-6.5),
        Some(-2.5),
        Some(-0.5),
        Some(-0.0),
        Some(0.5),
        Some(2.5),
        Some(5.5),
        Some(6.5),
        Some(7.5),
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
        Some(1e300),
        Some(-1e300),
    ];
    let steps = [
        1.0,
        2.0,
        3.0,
        -1.0,
        -2.0,
        -3.0,
        1.5,
        -1.5,
        2.9,
        -2.9,
        6.0,
        7.0,
        -6.0,
        -7.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        1e300,
        -1e300,
    ];
    for length in 0..=6_usize {
        let items: Vec<String> = (0..length).map(|item| item.to_string()).collect();
        for step in steps {
            let mut slices = Vec::new();
            let mut expected = Vec::new();
            for start in bounds {
                for stop in bounds {
                    slices.push(format!(
                        "xs[{}:{}:{}]",
                        float_part(start),
                        float_part(stop),
                        float_part(Some(step))
                    ));
                    expected.push(truncated_positions(length, start, stop, step));
                }
            }
            let source = format!("xs = [{}]\n[{}]", items.join(", "), slices.join(", "));
            assert_eq!(numbers(&result(&source)), expected, "{length} {step}");
        }
    }
    for (source, expected) in [
        (
            "xs = [10, 20, 30, 40, 50, 60]\nxs[-2.5::-1]",
            "[40.0,30.0,20.0,10.0]",
        ),
        ("xs = [10, 20, 30, 40, 50, 60]\nxs[:-2.5:-1]", "[60.0,50.0]"),
        ("xs = [10, 20, 30, 40, 50, 60]\nxs[0 / 0::-1]", "[10.0]"),
        (
            "xs = [10, 20, 30, 40, 50, 60]\nxs[-2.5:]",
            "[40.0,50.0,60.0]",
        ),
        (
            "xs = [10, 20, 30, 40, 50, 60]\nxs[-2.5::1]",
            "[40.0,50.0,60.0]",
        ),
        ("xs = [10, 20, 30, 40, 50, 60]\nxs[2.7::2]", "[30.0,50.0]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn a_step_of_one_or_an_omitted_step_is_the_one_colon_slice() {
    let bounds = [
        "",
        "0",
        "1",
        "3",
        "-1",
        "-3",
        "9",
        "-9",
        "2.5",
        "-2.5",
        "0 / 0",
        "null",
        "true",
        "\"2\"",
        "[1]",
        "Number.POSITIVE_INFINITY",
        "Number.NEGATIVE_INFINITY",
    ];
    for receiver in [
        "[10, 20, 30, 40, 50, 60]",
        "\"añejo!\"",
        "Unit",
        "[]",
        "\"\"",
    ] {
        let mut pairs = Vec::new();
        for start in bounds {
            for stop in bounds {
                pairs.push(format!(
                    "[xs[{start}:{stop}] == xs[{start}:{stop}:1], xs[{start}:{stop}] == xs[{start}:{stop}:], xs[{start}:{stop}] == xs[{start}:{stop}:true]]"
                ));
            }
        }
        let source = format!(
            "class Unit(n):\n    this.n = n\nUnit(1)\nUnit(2)\nUnit(3)\nxs = {receiver}\n[{}].every(row -> row.every(same -> same))",
            pairs.join(", ")
        );
        assert_eq!(result(&source), "true", "{receiver}");
    }
}

#[test]
fn steps_convert_as_bounds_do() {
    let valid = [
        ("2", "[10.0,30.0,50.0]"),
        ("\"2\"", "[10.0,30.0,50.0]"),
        ("\" 3 \"", "[10.0,40.0]"),
        ("\"-2\"", "[60.0,40.0,20.0]"),
        ("\"2.7\"", "[10.0,30.0,50.0]"),
        ("\"+2\"", "[10.0,30.0,50.0]"),
        ("\"1e1\"", "[10.0]"),
        ("\"inf\"", "[10.0]"),
        ("\"-Infinity\"", "[60.0]"),
        ("true", "[10.0,20.0,30.0,40.0,50.0,60.0]"),
        ("[2]", "[10.0,30.0,50.0]"),
        ("[-2]", "[60.0,40.0,20.0]"),
        ("[\"3\"]", "[10.0,40.0]"),
        ("[[2]]", "[10.0,30.0,50.0]"),
        ("[true]", "[10.0,20.0,30.0,40.0,50.0,60.0]"),
        ("Date(\"2020-1-1\")", "[10.0]"),
        ("Date(2)", "[10.0,30.0,50.0]"),
        ("Date(-2)", "[60.0,40.0,20.0]"),
        ("1.5", "[10.0,20.0,30.0,40.0,50.0,60.0]"),
        ("-1.5", "[60.0,50.0,40.0,30.0,20.0,10.0]"),
        ("2.7", "[10.0,30.0,50.0]"),
        ("-2.7", "[60.0,40.0,20.0]"),
        ("1e300", "[10.0]"),
        ("-1e300", "[60.0]"),
        ("Number.POSITIVE_INFINITY", "[10.0]"),
        ("Number.NEGATIVE_INFINITY", "[60.0]"),
    ];
    for (step, expected) in valid {
        let source = format!("xs = [10, 20, 30, 40, 50, 60]\nxs[::{step}]");
        assert_eq!(result(&source), expected, "{source}");
    }
    let invalid = [
        "0",
        "-0",
        "0.5",
        "-0.5",
        "-0.9",
        "0 / 0",
        "null",
        "false",
        "\"\"",
        "\" \"",
        "\"0\"",
        "\"0.5\"",
        "\"a\"",
        "\"2a\"",
        "\"0x10\"",
        "\"NaN\"",
        "\"1_0\"",
        "[]",
        "[0]",
        "[null]",
        "[false]",
        "[[]]",
        "[1, 2]",
        "[\"a\"]",
        "{a: 1}",
        "{}",
        "o.missing",
        "x -> x",
        "g",
        "Unit",
        "Unit(2)",
        "/a/",
        "/2/",
        "Date(0)",
    ];
    for step in invalid {
        for receiver in [
            "[10, 20, 30]",
            "\"abc\"",
            "Unit",
            "null",
            "o.missing",
            "5",
            "true",
            "{a: 1}",
            "x -> x",
            "Math",
        ] {
            let source = format!(
                "class Unit(n):\n    this.n = n\nUnit(1)\ndef g(v):\n    return v\no = {{}}\nxs = {receiver}\nxs[::{step}]"
            );
            assert_eq!(
                failure(&source),
                (ErrorKind::Type, STEP_ERROR.to_string()),
                "{source}"
            );
        }
    }
}

#[test]
fn every_part_is_evaluated_in_order_before_the_step_is_checked() {
    assert_eq!(
        result(
            "seen = []\ndef mark(label, value):\n    seen.push(label)\n    return value\n[mark(\"object\", [1, 2, 3])[mark(\"start\", 0):mark(\"stop\", 3):mark(\"step\", 1)], seen]"
        ),
        "[[1.0,2.0,3.0],[\"object\",\"start\",\"stop\",\"step\"]]"
    );
    assert_eq!(
        result(
            "seen = []\ndef mark(label, value):\n    seen.push(label)\n    return value\n[mark(\"object\", \"abc\")[:mark(\"stop\", 2):], seen]"
        ),
        "[\"ab\",[\"object\",\"stop\"]]"
    );
    assert_eq!(
        result(
            "seen = []\ndef mark(label, value):\n    seen.push(label)\n    return value\n[mark(\"list\", [1, 2, 3])[mark(\"start\", 0):mark(\"stop\", 2)], mark(\"text\", \"abc\")[mark(\"from\", 1):], mark(\"tail\", \"abc\")[:mark(\"upto\", 2)], seen]"
        ),
        "[[1.0,2.0],\"bc\",\"ab\",[\"list\",\"start\",\"stop\",\"text\",\"from\",\"tail\",\"upto\"]]"
    );
    for (source, expected) in [
        (
            "xs = [1, 2]\nxs[missing::0]",
            (ErrorKind::Reference, "missing is not defined"),
        ),
        (
            "xs = [1, 2]\nxs[1:missing:0]",
            (ErrorKind::Reference, "missing is not defined"),
        ),
        (
            "xs = [1, 2]\nxs[::missing]",
            (ErrorKind::Reference, "missing is not defined"),
        ),
        (
            "missing[::0]",
            (ErrorKind::Reference, "missing is not defined"),
        ),
        (
            "xs = [1, 2]\nxs[first:second:0]",
            (ErrorKind::Reference, "first is not defined"),
        ),
        ("n = null\nn[::0]", (ErrorKind::Type, STEP_ERROR)),
        (
            "(5)[:missing:]",
            (ErrorKind::Reference, "missing is not defined"),
        ),
        (
            "xs = [1, 2, 3]\ny is [xs[:[k][0]] for x in xs]",
            (ErrorKind::Reference, "k is not defined"),
        ),
        (
            "xs = [1, 2, 3]\ny is [xs[::[k][0]] for x in xs]",
            (ErrorKind::Reference, "k is not defined"),
        ),
        (
            "xs = [1, 2, 3]\ny is [[k][0][0:] for x in xs]",
            (ErrorKind::Reference, "k is not defined"),
        ),
        (
            "xs = [1, 2, 3]\ny is [[k][0][::1] for x in xs]",
            (ErrorKind::Reference, "k is not defined"),
        ),
        (
            "xs = [1, 2, 3]\ny is xs[::[k][0]]",
            (ErrorKind::Type, STEP_ERROR),
        ),
    ] {
        assert_eq!(
            failure(source),
            (expected.0, expected.1.to_string()),
            "{source}"
        );
    }
    assert_eq!(
        result(
            "class C(n):\n    this.n = n\nC(1)\nC(2)\ndef grow():\n    C(3)\n    return 1\n[C[::grow()].map(c -> c.n), C.length]"
        ),
        "[[1.0,2.0],3.0]"
    );
    assert_eq!(
        result(
            "seen = []\ndef mark(label, value):\n    seen.push(label)\n    return value\nclass C(n):\n    this.n = n\nC(1)\nC(2)\ndef grow():\n    C(3)\n    return 5\n[C[mark(\"start\", 0):grow()].map(c -> c.n), C.length, seen]"
        ),
        "[[1.0,2.0],3.0,[\"start\"]]"
    );
}

#[test]
fn every_shape_parses_to_one_stepped_slice_with_its_parts() {
    let name = |name: &str| Box::new(Expr::Identifier(name.to_string()));
    let number = |value: f64| Box::new(Expr::Number(value));
    let negative = |value: f64| {
        Box::new(Expr::Unary {
            operator: UnaryOp::Negate,
            operand: Box::new(Expr::Number(value)),
        })
    };
    let stepped = |start, end, step| Expr::SteppedSlice {
        object: name("xs"),
        start,
        end,
        step,
    };
    for (source, expected) in [
        ("xs[::]", stepped(None, None, None)),
        ("xs[::2]", stepped(None, None, Some(number(2.0)))),
        ("xs[1::]", stepped(Some(number(1.0)), None, None)),
        (
            "xs[1::2]",
            stepped(Some(number(1.0)), None, Some(number(2.0))),
        ),
        ("xs[:4:]", stepped(None, Some(number(4.0)), None)),
        (
            "xs[:4:2]",
            stepped(None, Some(number(4.0)), Some(number(2.0))),
        ),
        (
            "xs[1:4:]",
            stepped(Some(number(1.0)), Some(number(4.0)), None),
        ),
        (
            "xs[1:4:2]",
            stepped(Some(number(1.0)), Some(number(4.0)), Some(number(2.0))),
        ),
        ("xs[::-1]", stepped(None, None, Some(negative(1.0)))),
        ("xs[ : : 2 ]", stepped(None, None, Some(number(2.0)))),
        ("xs[::\n2]", stepped(None, None, Some(number(2.0)))),
        (
            "xs[a:b:c]",
            stepped(Some(name("a")), Some(name("b")), Some(name("c"))),
        ),
    ] {
        assert_eq!(
            parse(source).unwrap(),
            [Stmt::Expression(expected)],
            "{source}"
        );
    }
    let tree = |source: &str| format!("{:?}", parse(source).unwrap());
    for (source, slices, stepped) in [
        ("xs[1:4]", 1, 0),
        ("xs[:]", 1, 0),
        ("xs[c ? 1 : 2]", 0, 0),
        ("xs[c ? 1 : 2 : 4]", 1, 0),
        ("xs[1:c ? 3 : 4]", 1, 0),
        ("xs[1:c ? 3 : 4 : 2]", 0, 1),
        ("xs[c ? 1 : 2 :: 2]", 0, 1),
        ("items[all v in xs: v:]", 1, 0),
        ("items[:any v in xs: v]", 1, 0),
        ("items[:any v in xs: v:2]", 0, 1),
        ("items[0 if all v in xs: v else 1:]", 1, 0),
        ("values[1:lambda: 2]", 1, 0),
        ("values[::lambda: 2]", 0, 1),
        ("values[lambda: f(lambda: 1)]", 1, 0),
        ("xs[::2][1:]", 1, 1),
        ("xs[1:][::2]", 1, 1),
    ] {
        let parsed = tree(source);
        assert_eq!(
            parsed.matches("Slice {").count() - parsed.matches("SteppedSlice {").count(),
            slices,
            "{source}"
        );
        assert_eq!(
            parsed.matches("SteppedSlice {").count(),
            stepped,
            "{source}"
        );
    }
    let expression = |source: &str| parse_expression(source).unwrap();
    let member = Expr::Member {
        object: name("o"),
        property: "items".to_string(),
    };
    assert_eq!(expression("o.items[::2]").first(), Some(&*name("o")));
    assert_eq!(expression("o.items[::2]").object(), Some(&member));
    assert!(expression("(why ys)[::-1]").contains_reasoning());
    assert!(!expression("xs[::(why ys).length]").contains_reasoning());
    for (stepped, sliced) in [
        ("o.items[::2]", "o.items[:2]"),
        ("(why ys)[::-1]", "(why ys)[:1]"),
        ("xs[::(why ys).length]", "xs[:(why ys).length]"),
        ("xs@first[1::2]", "xs@first[1:]"),
        ("f(x)[::-1]", "f(x)[1:]"),
    ] {
        let (stepped, sliced) = (expression(stepped), expression(sliced));
        assert_eq!(stepped.first(), sliced.first());
        assert_eq!(stepped.object(), sliced.object());
        assert_eq!(stepped.contains_reasoning(), sliced.contains_reasoning());
        assert!(matches!(Ast::convert(&stepped), Ast::Array(_)));
        assert!(matches!(Ast::convert(&sliced), Ast::Array(_)));
    }
    assert_eq!(
        result(
            "xs = [10, 20, 30, 40, 50, 60]\nc = 2\n[xs[:c is 2], xs[c is 2:], xs[::c is 2], xs[:c is 2:], xs[c is 2::2]]"
        ),
        "[[10.0],[20.0,30.0,40.0,50.0,60.0],[10.0,20.0,30.0,40.0,50.0,60.0],[10.0],[20.0,40.0,60.0]]"
    );
    for (source, message, line, column) in [
        ("xs[1:2:3:4]", "Expected ']' but found ':'", 1, 9),
        ("xs[:::]", "Expected ']' but found ':'", 1, 6),
        ("xs[::2:]", "Expected ']' but found ':'", 1, 7),
        ("xs[1:2:3", "Expected ']' but found end of input", 1, 9),
        ("xs[1:2 3]", "Expected ']' but found number 3", 1, 8),
        ("x = [suppose: 1]", "Expected ']' but found ':'", 1, 13),
    ] {
        let error = parse(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        let position = error.position().unwrap();
        assert_eq!((position.line, position.column), (line, column), "{source}");
    }
}

#[test]
fn rendering_why_affects_and_graph_keys_show_the_slice_as_written() {
    for (source, rendered) in [
        ("xs[::]", "xs[::]"),
        ("xs[::2]", "xs[::2]"),
        ("xs[1::2]", "xs[1::2]"),
        ("xs[:4:2]", "xs[:4:2]"),
        ("xs[1:5:]", "xs[1:5:]"),
        ("xs[a:b:c]", "xs[a:b:c]"),
        ("s[::-1]", "s[::-1]"),
        ("xs[c ? 1 : 2 :: 2]", "xs[(c?1:2)::2]"),
        ("xs[::2][::-1]", "xs[::2][::-1]"),
        ("o.items[::2].length", "o.items[::2].length"),
        ("xs[1:3]", "xs[1:3]"),
    ] {
        let Some(Stmt::Expression(expression)) = parse(source).unwrap().into_iter().next() else {
            panic!("{source}");
        };
        assert_eq!(expression.to_string(), rendered, "{source}");
        assert_eq!(
            parse(&expression.to_string()).unwrap(),
            parse(source).unwrap(),
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    assert_eq!(
        json(
            &mut runtime,
            "xs = [1, 2, 3, 4]\na = 0\nb = 4\nc = 2\nys = xs[a:b:c]\n(why ys).map(step -> step.rule)"
        ),
        "[\"ys = xs[a:b:c]\",\"xs = [1,2,3,4]\",\"a = 0\",\"b = 4\",\"c = 2\"]"
    );
    assert_eq!(
        json(&mut runtime, "(why ys)[0].from"),
        "[\"xs\",\"a\",\"b\",\"c\"]"
    );
    assert_eq!(
        json(&mut runtime, "(affects c).map(step -> step.node)"),
        "[\"ys\"]"
    );
    assert_eq!(dependencies(&runtime, "ys"), ["a", "b", "c", "xs"]);
    assert_eq!(
        json(
            &mut runtime,
            "zs = xs[1:5:]\nws = xs[::]\n[(why zs)[0].rule, (why ws)[0].rule]"
        ),
        "[\"zs = xs[1:5:]\",\"ws = xs[::]\"]"
    );
    let mut runtime = Runtime::new();
    runtime
        .run("class Bid(amount):\n    this.amount = amount\nstep = 2\nbook = Bid[::step]\nBid(5)")
        .unwrap();
    assert_eq!(dependencies(&runtime, "book"), ["$Bid", "step"]);
    for (source, kind, message) in [
        (
            "xs = [1, 2]\nxs[::2](1)",
            ErrorKind::Type,
            "xs[::2] is not a function",
        ),
        (
            "xs = [1, 2]\nall x in xs[::2](1): x",
            ErrorKind::Type,
            "xs[::2] is not a function",
        ),
        (
            "xs = [1, 2]\nwhy xs[::2]",
            ErrorKind::Type,
            "Cannot explain xs[::2]",
        ),
        (
            "xs = [1, 2]\ndelete xs[::2]",
            ErrorKind::Syntax,
            "Cannot delete xs[::2]",
        ),
        (
            "xs = [1, 2]\nxs[::2] = [1]",
            ErrorKind::Syntax,
            "Cannot assign to xs[::2]",
        ),
        (
            "xs = [1, 2]\nxs[1:3] = [1]",
            ErrorKind::Syntax,
            "Cannot assign to xs[1:3]",
        ),
    ] {
        assert_eq!(failure(source), (kind, message.to_string()), "{source}");
    }
}

#[test]
fn every_receiver_kind_takes_its_positions() {
    for (source, expected) in [
        ("[10, 20, 30, 40][1::2]", "[20.0,40.0]"),
        ("xs = [10, 20, 30, 40]\nxs[::-2]", "[40.0,20.0]"),
        (
            "o = {items: [1, 2, 3, 4, 5]}\n[o.items[::2], o[\"items\"][1::2]]",
            "[[1.0,3.0,5.0],[2.0,4.0]]",
        ),
        (
            "grid = [[1, 2], [3, 4], [5, 6]]\n[grid[::2], grid[1][::-1]]",
            "[[[1.0,2.0],[5.0,6.0]],[4.0,3.0]]",
        ),
        ("def make() => [1, 2, 3, 4]\nmake()[::3]", "[1.0,4.0]"),
        ("xs = [1, 2, 3, 4]\nxs.filter(x -> x > 1)[::2]", "[2.0,4.0]"),
        ("xs = [1, 2, 3, 4]\nxs.value[::-1]", "[4.0,3.0,2.0,1.0]"),
        ("xs = [1, 2]\nxs.push(3)\nxs@first[::-1]", "[2.0,1.0]"),
        (
            "class Box(items):\n    this.items = items\n    def odd():\n        return this.items[1::2]\nBox([1, 2, 3, 4]).odd()",
            "[2.0,4.0]",
        ),
        (
            "xs = [1, 2, 3, 4]\nys = xs.map(x -> [x, x * 10][::-1])\nys[0]",
            "[10.0,1.0]",
        ),
        ("def back(s) => s[::-1]\nback(\"abc\")", "\"cba\""),
        ("f = lambda v: v[::-1]\nf([3, 1, 2])", "[2.0,1.0,3.0]"),
        ("[][::-1]", "[]"),
        ("\"\"[::2]", "\"\""),
        ("\"añejo\"[::-1]", "\"ojeña\""),
        ("\"a🙂b\"[::-1]", "\"b🙂a\""),
        ("\"e\u{301}x\"[::-1]", "\"x\u{301}e\""),
        ("\"héllo wörld\"[1::2]", "\"él öl\""),
        (
            "class Unit(n):\n    this.n = n\nUnit(1)\nUnit(2)\nUnit(3)\nkind = Unit\n[Unit[::2].map(u -> u.n), kind[::-1].map(u -> u.n)]",
            "[[1.0,3.0],[3.0,2.0,1.0]]",
        ),
        ("class Empty\nEmpty[::2]", "[]"),
        (
            "class Base(n):\n    this.n = n\nclass Sub(n) extends Base:\n    this.n = n\nSub(1)\nSub(2)\nSub(3)\nSub[::2].map(s -> s.n)",
            "[1.0,3.0]",
        ),
        (
            "class Team(first, every):\n    this.first = first\n    this.every = every\nclass Player(name):\n    this.name = name\n$Team.members = Player[$Team.first::$Team.every].map(p -> p.name)\nPlayer(\"a\")\nPlayer(\"b\")\nPlayer(\"c\")\nPlayer(\"d\")\nt = Team(1, 2)\nPlayer(\"e\")\nPlayer(\"f\")\nt.members",
            "[\"b\",\"d\",\"f\"]",
        ),
        ("all x in [1, 2, 3, 4][::2]: x % 2 == 1", "true"),
        (
            "xs = [5, 6, 7]\n[x * 2 for x in xs[::-1]]",
            "[14.0,12.0,10.0]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for receiver in [
        "null",
        "o.missing",
        "5",
        "true",
        "{a: 1}",
        "x -> x",
        "Math",
        "Date(\"2020-1-1\")",
        "/a/",
    ] {
        let source = format!("o = {{}}\nxs = {receiver}\n[xs[::2], xs[1:3:-1], xs[::]]");
        let mut runtime = Runtime::new();
        let value = runtime.run(&source).unwrap();
        assert_eq!(
            format!("{value:?}"),
            "List([Undefined, Undefined, Undefined])",
            "{source}"
        );
    }
    for slice in ["xs@first[1:]", "xs@first[::-1]", "xs@first[1::2]"] {
        let source = format!("xs = [1, 2]\nxs.push(3)\n{slice}.push(4)");
        assert_eq!(
            failure(&source),
            (ErrorKind::Type, "State versions are read-only".to_string()),
            "{source}"
        );
    }
}

#[test]
fn declarations_follow_mutations_bounds_steps_and_instances() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "xs = [1, 2, 3, 4, 5]\nstart = 0\nstep = 2\nys = xs[start::step]\nys",
            "[1.0,3.0,5.0]",
        ),
        ("xs.push(6, 7)\nys", "[1.0,3.0,5.0,7.0]"),
        ("xs.pop()\nys", "[1.0,3.0,5.0]"),
        ("xs.shift()\nys", "[2.0,4.0,6.0]"),
        ("start = 1\nys", "[3.0,5.0]"),
        ("step = -1\nys", "[3.0,2.0]"),
        ("start = -1\nys", "[6.0,5.0,4.0,3.0,2.0]"),
        ("step = 3.5\nys", "[6.0]"),
        ("xs = [9, 8, 7, 6]\nys", "[6.0]"),
        ("start = 0\nys", "[9.0,6.0]"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class P(n):\n    this.n = n\na = P(1)\nb = P(2)\nc = P(3)\nys = P[::2].map(p -> p.n)\nys",
            "[1.0,3.0]",
        ),
        ("P(4)\nP(5)\nys", "[1.0,3.0,5.0]"),
        ("delete b.n\ndelete b\nys", "[1.0,4.0]"),
        ("c.n = 30\nys", "[1.0,4.0]"),
        ("a.n = 10\nys", "[10.0,4.0]"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class Row(cells):\n    this.cells = cells\nstep = 2\n$Row.dark = $Row.cells[::step]\nr = Row([5, 8, 2, 9])\nr.dark",
            "[5.0,2.0]",
        ),
        ("step = 3\nr.dark", "[5.0,9.0]"),
        ("r.cells.push(4)\nr.dark", "[5.0,9.0]"),
        ("r.cells.push(1, 7)\nr.dark", "[5.0,9.0,7.0]"),
        ("q = Row(\"abcdef\")\nq.dark", "\"ad\""),
        ("step = -2\n[r.dark, q.dark]", "[[7.0,4.0,2.0,5.0],\"fdb\"]"),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn snapshots_hypotheticals_templates_and_conditions_read_stepped_slices() {
    for (source, expected) in [
        (
            "xs = [10, 20, 30, 40]\nys = xs[::2]\nfrozen = ys.value\nxs.push(50)\n[ys, frozen]",
            "[[10.0,30.0,50.0],[10.0,30.0]]",
        ),
        (
            "xs = [10, 20, 30, 40]\nk = 2\nys = xs[::k.value]\nk = 3\nys",
            "[10.0,30.0]",
        ),
        (
            "xs = [10, 20, 30, 40]\nk = 2\nys = xs[k.value - 1::k]\nk = 3\nys",
            "[20.0]",
        ),
        (
            "xs = [10, 20, 30, 40, 50, 60]\nb = 4\nys = xs[:b.value:2]\nb = 6\nys",
            "[10.0,30.0]",
        ),
        (
            "xs = [10, 20, 30, 40, 50, 60]\nk = 2\nys = xs[::k.value]\nk = 3\nxs.push(70)\nys",
            "[10.0,30.0,50.0,70.0]",
        ),
        (
            "xs = [10, 20, 30, 40, 50, 60]\nb = 4\nys = xs[:b.value:2]\nb = 6\nxs.push(70)\nys",
            "[10.0,30.0]",
        ),
        (
            "xs = [10, 20, 30, 40, 50, 60]\na = 1\nys = xs[a.value::2]\na = 2\nxs.push(70)\nys",
            "[20.0,40.0,60.0]",
        ),
        (
            "xs = [10, 20, 30, 40, 50, 60]\nb = 4\nys = xs[:b.value]\nb = 6\nxs.push(70)\nys",
            "[10.0,20.0,30.0,40.0]",
        ),
        (
            "xs = [10, 20, 30, 40]\nfrozen = xs.value[::2]\nxs.push(50)\nfrozen",
            "[10.0,30.0]",
        ),
        (
            "xs = [10, 20, 30, 40]\nk = 2\nys = xs[::k]\nseen = suppose k = 3: ys\n[seen, ys]",
            "[[10.0,40.0],[10.0,30.0]]",
        ),
        (
            "xs = [10, 20, 30, 40]\nys = xs[::2]\nseen = suppose xs.push(50): ys\n[seen, ys, xs]",
            "[[10.0,30.0,50.0],[10.0,30.0],[10.0,20.0,30.0,40.0]]",
        ),
        (
            "xs = [10, 20, 30, 40]\nlabel = `${xs[::-1]} back`\nxs.push(50)\nlabel",
            "\"50,40,30,20,10 back\"",
        ),
        (
            "xs = [10, 20, 30, 40]\nif xs[::2].length == 2:\n    flag = true\nflag",
            "true",
        ),
        (
            "xs = [10, 20, 30]\nsize = xs[::3].length > 1 ? \"wide\" : \"narrow\"\nfirst := size\nxs.push(40)\n[first, size]",
            "[\"narrow\",\"wide\"]",
        ),
        (
            "xs = [1, 2, 3, 4]\nonce := xs[::2]\nxs.push(5)\n[once, xs[::2]]",
            "[[1.0,3.0],[1.0,3.0,5.0]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn failed_statements_and_transactions_restore_stepped_slices() {
    let mut runtime = Runtime::new();
    runtime
        .run("xs = [1, 2, 3, 4]\nk = 2\nys = xs[::k]")
        .unwrap();
    assert_eq!(
        outcome(&mut runtime, "k = 0"),
        format!("TypeError: {STEP_ERROR} at Some(Position {{ line: 1, column: 1 }})")
    );
    assert_eq!(json(&mut runtime, "[k, ys]"), "[2.0,[1.0,3.0]]");
    assert_eq!(
        outcome(&mut runtime, "k = \"a\""),
        format!("TypeError: {STEP_ERROR} at Some(Position {{ line: 1, column: 1 }})")
    );
    assert_eq!(json(&mut runtime, "k = 3\n[k, ys]"), "[3.0,[1.0,4.0]]");
    for (source, expected) in [
        (
            "xs = [1, 2, 3, 4]\nk = 2\nys = xs[::k]\ntry:\n    k = 0\ncatch error:\n    failure = error\n[failure, k, ys]",
            "[\"TypeError: Slice step must be a nonzero number\",2.0,[1.0,3.0]]",
        ),
        (
            "def checked(n):\n    if n < 0:\n        throw \"NEGATIVE\"\n    return n\nxs = [4, 9, 7, 12]\npicked = xs[::2].map(checked)\nfailure = \"\"\ntry:\n    xs.push(-3)\ncatch error:\n    failure = error\n[failure, xs, picked]",
            "[\"NEGATIVE\",[4.0,9.0,7.0,12.0],[4.0,7.0]]",
        ),
        (
            "xs = [1, 2, 3]\nys = xs[::-1]\ntry:\n    xs.push(4)\n    assert(ys, [4, 3, 2, 1])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\n[xs, ys]",
            "[[1.0,2.0,3.0],[3.0,2.0,1.0]]",
        ),
        (
            "def checked(n):\n    if n < 0:\n        throw \"NEGATIVE\"\n    return n\nclass Crate(weight): this.weight = weight\nCrate(6)\nCrate(2)\nCrate(9)\nlatest = Crate[::-2].map(crate -> checked(crate.weight))\nfailure = \"\"\ntry:\n    Crate(-1)\ncatch error:\n    failure = error\n[failure, Crate.length, latest]",
            "[\"NEGATIVE\",3.0,[9.0,6.0]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn class_level_rules_check_the_references_of_every_part_when_declared() {
    for part in [
        "[missing::2]",
        "[:missing:2]",
        "[::missing]",
        "[1:2:missing]",
    ] {
        let source = format!(
            "class Item\nfailure = \"\"\ntry:\n    $Item.part = \"abcd\"{part}\ncatch error:\n    failure = error\nfailure"
        );
        assert_eq!(
            result(&source),
            "\"ReferenceError: missing is not defined\"",
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "class Item\nstep = 2\n$Item.part = \"abcdef\"[::step]\nitem = Item()\nitem.part",
            "\"ace\"",
        ),
        ("step = -3\nitem.part", "\"fc\""),
    ] {
        assert_eq!(json(&mut runtime, source), expected, "{source}");
    }
    for (rule, change) in [
        ("copy = $Item.list[::2]", "$Item.source = $Item.copy[0]"),
        (
            "part = \"abcd\"[$Item.source::2]",
            "$Item.source = $Item.part.length",
        ),
        (
            "head = \"abcd\"[:$Item.source:]",
            "$Item.source = $Item.head.length",
        ),
        (
            "every = \"abcd\"[::$Item.source]",
            "$Item.source = $Item.every.length",
        ),
    ] {
        let source = format!(
            "class Item\n$Item.list = [$Item.source]\n$Item.{rule}\nfailure = \"\"\ntry:\n    {change}\ncatch error:\n    failure = error\nfailure"
        );
        assert_eq!(
            result(&source),
            "\"TypeError: Circular Dependency\"",
            "{source}"
        );
    }
}

#[test]
fn large_receivers_take_their_positions_through_the_public_api() {
    let mut runtime = Runtime::new();
    runtime.run("holder = {items: []}").unwrap();
    let Some(Value::Object(id)) = runtime.state.variable("holder").cloned() else {
        panic!("holder is an object");
    };
    runtime.state.object_mut(&id).unwrap().properties.insert(
        "items".to_string(),
        Value::List(
            (0..100_000)
                .map(|item| Value::Number(f64::from(item)))
                .collect(),
        ),
    );
    assert_eq!(
        json(
            &mut runtime,
            "xs = holder.items\n[xs[::1000].length, xs[::-1][0], xs[::-1][99999], xs[1::2].length, xs[::-7].length, xs[::Number.POSITIVE_INFINITY], xs[-1::-50000]]"
        ),
        "[100.0,99999.0,0.0,50000.0,14286.0,[0.0],[99999.0,49999.0]]"
    );
}

const LIST: &str = "xs = [10, 20, 30, 40, 50, 60]\n";
const READINGS: &str = "class Reading(level):\n    this.level = level\nReading(3)\nReading(5)\nReading(10)\nReading(7)\n";

#[test]
fn the_compatibility_rows_give_their_documented_results() {
    let step = format!("TypeError: {STEP_ERROR} at Some(Position {{ line: 2, column: 1 }})");
    for (program, expected) in [
        (format!("{LIST}xs[::2]"), "[10.0,30.0,50.0]".to_string()),
        (format!("{LIST}xs[1::2]"), "[20.0,40.0,60.0]".to_string()),
        (format!("{LIST}xs[:4:2]"), "[10.0,30.0]".to_string()),
        (format!("{LIST}xs[1:5:2]"), "[20.0,40.0]".to_string()),
        (
            format!("{LIST}xs[::-1]"),
            "[60.0,50.0,40.0,30.0,20.0,10.0]".to_string(),
        ),
        (
            format!("{LIST}xs[5:1:-1]"),
            "[60.0,50.0,40.0,30.0]".to_string(),
        ),
        (
            format!("{LIST}xs[3::-1]"),
            "[40.0,30.0,20.0,10.0]".to_string(),
        ),
        (
            format!("{LIST}xs[1:5:]"),
            "[20.0,30.0,40.0,50.0]".to_string(),
        ),
        (
            format!("{LIST}xs[::]"),
            "[10.0,20.0,30.0,40.0,50.0,60.0]".to_string(),
        ),
        (format!("{LIST}xs[::0]"), step.clone()),
        (format!("{LIST}xs[::0.5]"), step.clone()),
        (format!("{LIST}xs[::null]"), step.clone()),
        (format!("{LIST}xs[::\"a\"]"), step.clone()),
        (format!("{LIST}xs[::\"2\"]"), "[10.0,30.0,50.0]".to_string()),
        (format!("{LIST}xs[::2.7]"), "[10.0,30.0,50.0]".to_string()),
        (
            format!("{LIST}xs[::Number.POSITIVE_INFINITY]"),
            "[10.0]".to_string(),
        ),
        (format!("{LIST}xs[::2][::2]"), "[10.0,50.0]".to_string()),
        (
            format!("{LIST}xs[1:2:3:4]"),
            "SyntaxError: Expected ']' but found ':' at Some(Position { line: 2, column: 9 })"
                .to_string(),
        ),
        (
            format!("{LIST}c = true\nxs[1:c ? 3 : 4 : 2]"),
            "[20.0]".to_string(),
        ),
        (
            format!("{LIST}c = false\nxs[c ? 1 : 2 :: 2]"),
            "[30.0,50.0]".to_string(),
        ),
        (
            format!("{LIST}evens = xs[::2]\nxs.push(70)\nevens"),
            "[10.0,30.0,50.0,70.0]".to_string(),
        ),
        (
            format!("{LIST}k = 2\nys = xs[::k]\nk = 3\nys"),
            "[10.0,40.0]".to_string(),
        ),
        (
            format!("{LIST}k = 2\nys = xs[::k.value]\nk = 3\nys"),
            "[10.0,30.0,50.0]".to_string(),
        ),
        (
            format!("{LIST}xs[::missing]"),
            "ReferenceError: missing is not defined at Some(Position { line: 2, column: 1 })"
                .to_string(),
        ),
        (
            format!("{LIST}xs[::2] = [1]"),
            "SyntaxError: Cannot assign to xs[::2] at Some(Position { line: 2, column: 1 })"
                .to_string(),
        ),
        (
            "s = \"stressed\"\ns[::-1]".to_string(),
            "\"desserts\"".to_string(),
        ),
        (
            "s = \"héllo wörld\"\ns[::-1]".to_string(),
            "\"dlröw olléh\"".to_string(),
        ),
        ("s = \"abcdef\"\ns[1::2]".to_string(), "\"bdf\"".to_string()),
        (
            format!("{READINGS}Reading[::2].map(r -> r.level)"),
            "[3.0,10.0]".to_string(),
        ),
        (
            format!("{READINGS}Reading[1::2].map(r -> r.level)"),
            "[5.0,7.0]".to_string(),
        ),
        ("class Empty\nEmpty[::2]".to_string(), "[]".to_string()),
        ("n = null\nn[::2]".to_string(), "null".to_string()),
        ("n = null\nn[::0]".to_string(), step.clone()),
        ("o = {a: 1}\no[::2]".to_string(), "null".to_string()),
        ("(5)[::2]".to_string(), "null".to_string()),
    ] {
        assert_eq!(
            outcome(&mut Runtime::new(), &program),
            expected,
            "{program}"
        );
    }
}

#[test]
fn unchanged_forms_keep_their_accepted_results() {
    for (program, expected) in [
        (
            format!("{LIST}[xs[1:3], xs[:2], xs[-2:], xs[:]]"),
            "[[20.0,30.0],[10.0,20.0],[50.0,60.0],[10.0,20.0,30.0,40.0,50.0,60.0]]",
        ),
        (
            "s = \"abcdef\"\n[s[1:3], s[-2:], s[:]]".to_string(),
            "[\"bc\",\"ef\",\"abcdef\"]",
        ),
        (
            format!("{READINGS}Reading[1:].map(.level)"),
            "[5.0,10.0,7.0]",
        ),
        (format!("{LIST}xs.slice(0, 4, 2)"), "[10.0,20.0,30.0,40.0]"),
        (format!("{LIST}c = true\nxs[c ? 1 : 2]"), "20.0"),
        (
            format!("{LIST}c = true\nxs[c ? 1 : 2 : 4]"),
            "[20.0,30.0,40.0]",
        ),
        (format!("{LIST}c = true\nxs[1:c ? 3 : 4]"), "[20.0,30.0]"),
        ("o = {a: 1, b: 2}\n[o[\"a\"], o.b]".to_string(), "[1.0,2.0]"),
        ("c = false\nx = c ? 1 : 2\nx".to_string(), "2.0"),
        (
            format!("{LIST}ys = xs[1:4]\nxs.push(70)\nys"),
            "[20.0,30.0,40.0]",
        ),
        ("x = [1, 2, 3]\nx[x[0]:x[2]]".to_string(), "[2.0,3.0]"),
        (
            "items = [1, 2, 3, 4]\nxs = [true]\n[items[all v in xs: v:], items[:any v in xs: v]]"
                .to_string(),
            "[[2.0,3.0,4.0],[1.0]]",
        ),
        ("values = [1, 2, 3]\nvalues[1:lambda: 2]".to_string(), "[]"),
        (
            "xs = [1, 2, 3]\n[xs.slice(1), \"abc\".slice(1, 2), xs.at(-1), xs[-1]]".to_string(),
            "[[2.0,3.0],\"b\",3.0,3.0]",
        ),
    ] {
        assert_eq!(result(&program), expected, "{program}");
    }
    for (program, expected) in [
        (
            format!("{LIST}xs[1:3] = [1]"),
            "SyntaxError: Cannot assign to xs[1:3] at Some(Position { line: 2, column: 1 })",
        ),
        (
            "x = [suppose: 1]".to_string(),
            "SyntaxError: Expected ']' but found ':' at Some(Position { line: 1, column: 13 })",
        ),
        (
            format!("{LIST}xs[1:2 3]"),
            "SyntaxError: Expected ']' but found number 3 at Some(Position { line: 2, column: 8 })",
        ),
    ] {
        assert_eq!(
            outcome(&mut Runtime::new(), &program),
            expected,
            "{program}"
        );
    }
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-stepped-slices-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_stepped_slices() {
    let file = SourceFile::new("file");
    let program = "xs = [10, 20, 30, 40, 50, 60]\nevens = xs[::2]\nxs.push(70)\n[evens, xs[::-3], \"añejo\"[::-1], xs[::\n-6]]";
    for newline in ["\n", "\r\n"] {
        let source = program.replace('\n', newline);
        let measured = file.run(&source, &["--json"]);
        assert!(measured.status.success(), "{measured:?}");
        assert!(measured.stderr.is_empty(), "{measured:?}");
        assert_eq!(
            String::from_utf8(measured.stdout).unwrap(),
            "[[10.0,30.0,50.0,70.0],[70.0,40.0,10.0],\"ojeña\",[70.0,10.0]]\n"
        );
        let plain = file.run(&source, &[]);
        assert!(plain.status.success(), "{plain:?}");
        assert_eq!(
            String::from_utf8(plain.stdout).unwrap(),
            "10,30,50,70,70,40,10,ojeña,70,10\n"
        );
    }
    let graph = String::from_utf8(
        file.run(
            "xs = [1, 2, 3]\nk = 2\nys = xs[::k]\nys",
            &["--json", "--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        graph.contains("  xs [variable]\n    updates  ys\n  k [variable]\n    updates  ys\n  ys [variable]\n    reads    xs, k"),
        "{graph}"
    );
    let block = String::from_utf8(
        file.run(
            "xs = [1, 2, 3]\nif xs[::2].length > 1:\n    flag = true\nflag",
            &["--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        block.contains("  xs [variable]\n    updates  if(xs[::2].length>1)\n  if(xs[::2].length>1) [if]\n    reads    xs"),
        "{block}"
    );
    for (source, message) in [
        (
            "xs = [1]\nxs[::0]",
            "TypeError: Slice step must be a nonzero number (line 2, column 1)\n",
        ),
        (
            "xs = [1]\nxs[1:2:3:4]",
            "SyntaxError: Expected ']' but found ':' (line 2, column 9)\n",
        ),
    ] {
        let failed = file.run(source, &["--json"]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(String::from_utf8(failed.stderr).unwrap(), message);
    }
    let session = "xs = [10, 20, 30, 40]\nys = xs[::2]\nys\nxs.push(50)\nys\nxs[::\n-1]\n\ns = \"añejo\"\ns[::-1]\nxs[::0]\n.clear\nxs = [1, 2, 3]\nxs[1::]\n.exit\n";
    for (options, expected) in [
        (
            &["--plain"][..],
            "10,20,30,40\n10,30\n10,30\n5\n10,30,50\n50,40,30,20,10\nañejo\nojeña\nState cleared.\n1,2,3\n2,3\n",
        ),
        (
            &["--plain", "--json"][..],
            "[10.0,20.0,30.0,40.0]\n[10.0,30.0]\n[10.0,30.0]\n5.0\n[10.0,30.0,50.0]\n[50.0,40.0,30.0,20.0,10.0]\n\"añejo\"\n\"ojeña\"\nState cleared.\n[1.0,2.0,3.0]\n[2.0,3.0]\n",
        ),
        (
            &["--json"][..],
            "[10.0,20.0,30.0,40.0]\n[10.0,30.0]\n[10.0,30.0]\n5.0\n[10.0,30.0,50.0]\n[50.0,40.0,30.0,20.0,10.0]\n\"añejo\"\n\"ojeña\"\nState cleared.\n[1.0,2.0,3.0]\n[2.0,3.0]\n",
        ),
    ] {
        for newline in ["\n", "\r\n"] {
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.ends_with(expected), "{options:?} {stdout}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                "TypeError: Slice step must be a nonzero number (line 1, column 1)\n"
            );
        }
    }
}
