mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::builtins;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Spanned, Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, ObjectId, Runtime, Value};

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid keeps the distance between two instances live with Math.hypot",
    "Nucleoid keeps Math.hypot distances live in class level rules",
    "Nucleoid measures lengths in three dimensions with Math.hypot",
    "Nucleoid rolls back failed updates through Math.hypot",
];
const REFERENCE_TITLE: &str = "Nucleoid keeps Euclidean lengths live with Math.hypot";
const README_TITLE: &str = "Math.hypot keeps a drone's distance to its landing pad live";
const EXAMPLES_TITLE: &str = "Math.hypot keeps the antennas that serve a moving phone live";

const SPELLINGS: [(&str, &str); 10] = [
    (
        "Math.hypot(charger.x - robot.x, charger.y - robot.y)",
        "((charger.x - robot.x) ** 2 + (charger.y - robot.y) ** 2) ** 0.5",
    ),
    (
        "Math.hypot($Drone.x - home.x, $Drone.y - home.y)",
        "((.x - home.x) ** 2 + (.y - home.y) ** 2) ** 0.5",
    ),
    (
        "Math.hypot(crate.w, crate.d, crate.h)",
        "(crate.w ** 2 + crate.d ** 2 + crate.h ** 2) ** 0.5",
    ),
    (
        "Math.hypot(move.x, move.y, move.z)",
        "(move.x ** 2 + move.y ** 2 + move.z ** 2) ** 0.5",
    ),
    ("Math.hypot(dx, dy)", "(dx ** 2 + dy ** 2) ** 0.5"),
    (
        "Math.hypot(leg.dx, leg.dy, rise)",
        "(leg.dx ** 2 + leg.dy ** 2 + rise ** 2) ** 0.5",
    ),
    (
        "Math.hypot(velocity.x, velocity.y, velocity.z)",
        "(velocity.x ** 2 + velocity.y ** 2 + velocity.z ** 2) ** 0.5",
    ),
    (
        "Math.hypot(drone.x - pad.x, drone.y - pad.y)",
        "((drone.x - pad.x) ** 2 + (drone.y - pad.y) ** 2) ** 0.5",
    ),
    (
        "Math.hypot($Antenna.x - phone.x, $Antenna.y - phone.y)",
        "((.x - phone.x) ** 2 + (.y - phone.y) ** 2) ** 0.5",
    ),
    (
        "Math.hypot(dx, dy, dz)",
        "(dx ** 2 + dy ** 2 + dz ** 2) ** 0.5",
    ),
];

const KINDS: [(&str, &str); 26] = [
    ("", "3"),
    ("", "-3"),
    ("", "2.5"),
    ("", "-0"),
    ("", "0"),
    ("", "1e-3"),
    ("", "\"3\""),
    ("", "\" 4 \""),
    ("", "\"\""),
    ("", "\"x\""),
    ("", "\"1e3\""),
    ("", "true"),
    ("", "false"),
    ("", "null"),
    ("", "undefined"),
    ("", "[3]"),
    ("", "[]"),
    ("", "[1, 2]"),
    ("", "[\"-5\"]"),
    ("", "{}"),
    ("", "Date(12)"),
    ("", "x -> x"),
    ("", "/a/"),
    ("class Box", "Box"),
    ("class Box(v):\n    this.v = v\nb = Box(1)", "b"),
    ("o = {a: 2}", "o.a"),
];

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

fn interpolations(raw: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = raw;
    while let Some((_, after)) = rest.split_once("${") {
        let mut depth = 0_usize;
        let mut end = after.len();
        for (index, character) in after.char_indices() {
            match character {
                '{' => depth += 1,
                '}' if depth == 0 => {
                    end = index;
                    break;
                }
                '}' => depth -= 1,
                _ => {}
            }
        }
        found.push(&after[..end]);
        rest = after.get(end + 1..).unwrap_or_default();
    }
    found
}

fn tokens(source: &str) -> Vec<Token> {
    let mut found = Vec::new();
    for spanned in tokenize(source).unwrap() {
        if let Token::Template(raw) = &spanned.token {
            for interpolation in interpolations(raw) {
                found.extend(tokens(interpolation));
            }
        }
        found.push(spanned.token);
    }
    found
}

fn calls_hypot(source: &str) -> bool {
    tokens(source).windows(4).any(|window| {
        window[0] == Token::Identifier("Math".to_string())
            && window[1] == Token::Dot
            && window[2] == Token::Identifier("hypot".to_string())
            && window[3] == Token::LeftParen
    })
}

fn spelled_out(source: &str) -> String {
    let mut spelled = source.to_string();
    for (written, spelling) in SPELLINGS {
        spelled = spelled.replace(written, spelling);
    }
    spelled
}

fn touches(first: &Spanned, second: &Spanned) -> bool {
    second.position.line == first.position.line
        && second.position.column == first.position.column + 1
}

fn calls_padding_alias(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(3).any(|window| {
            window[0].token == Token::Dot
                && matches!(&window[1].token, Token::Identifier(name) if ["rjust", "ljust", "PadLeft", "PadRight"].contains(&name.as_str()))
                && window[2].token == Token::LeftParen
        })
    })
}

fn writes_xor_stage(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(3).any(|window| {
            window[0].token == Token::Pipe
                && window[1].token == Token::Identifier("xor".to_string())
                && window[2].token == Token::LeftParen
        })
    })
}

fn writes_bang_in(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(2).any(|pair| {
            pair[0].token == Token::Bang
                && pair[1].token == Token::Keyword(Keyword::In)
                && touches(&pair[0], &pair[1])
        })
    })
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

fn json(runtime: &mut Runtime, source: &str) -> String {
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    renumbered(&runtime.serialize_json(&value).unwrap())
}

fn result(source: &str) -> String {
    json(&mut Runtime::new(), source)
}

fn outcome(runtime: &mut Runtime, source: &str) -> String {
    match runtime.run(source) {
        Ok(value) => renumbered(&runtime.serialize_json(&value).unwrap()),
        Err(error) => renumbered(&error.to_string()),
    }
}

fn fresh(source: &str) -> String {
    outcome(&mut Runtime::new(), source)
}

fn failure(source: &str) -> String {
    renumbered(
        &Runtime::new()
            .run(source)
            .expect_err("the program raises an error")
            .to_string(),
    )
}

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "Math.hypot regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
}

fn dependencies(runtime: &Runtime, name: &str) -> Vec<String> {
    let keys: Vec<String> = runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap()
        .dependencies
        .iter()
        .map(ToString::to_string)
        .collect();
    renumbered(&keys.join("\n"))
        .split('\n')
        .map(str::to_string)
        .collect()
}

fn number(value: Value) -> f64 {
    match value {
        Value::Number(number) => number,
        other => panic!("{other:?} is not a number"),
    }
}

fn hypot(values: &[f64]) -> f64 {
    let arguments: Vec<Value> = values.iter().copied().map(Value::Number).collect();
    number(builtins::math("hypot", &arguments).unwrap())
}

fn direct(values: &[f64]) -> f64 {
    values
        .iter()
        .fold(0.0, |total, value| total + value * value)
        .sqrt()
}

fn power_of_two(exponent: i32) -> f64 {
    if exponent < -1022 {
        f64::from_bits(1_u64 << (exponent + 1074))
    } else {
        f64::from_bits(((exponent + 1023) as u64) << 52)
    }
}

fn same_bits(actual: f64, expected: f64, context: &str) {
    assert_eq!(
        actual.to_bits(),
        expected.to_bits(),
        "{context}: {actual:e} against {expected:e}"
    );
}

struct Numbers(u64);

impl Numbers {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn magnitude(&mut self, lowest: i32, highest: i32) -> f64 {
        let span = u64::try_from(highest - lowest + 1).unwrap();
        let exponent = lowest + i32::try_from(self.next() % span).unwrap();
        let mantissa =
            1.0 + f64::from(u32::try_from(self.next() >> 40).unwrap()) / f64::from(1_u32 << 24);
        let sign = if self.next() % 2 == 0 { 1.0 } else { -1.0 };
        sign * mantissa * power_of_two(exponent)
    }
}

fn pythagorean_tuples() -> Vec<(Vec<f64>, f64)> {
    let mut tuples = Vec::new();
    for m in 2_u64..40 {
        for n in 1..m {
            if (m - n) % 2 == 1 && gcd(m, n) == 1 {
                let (a, b, c) = (m * m - n * n, 2 * m * n, m * m + n * n);
                for k in 1..=3 {
                    tuples.push((vec![(k * a) as f64, (k * b) as f64], (k * c) as f64));
                }
            }
        }
    }
    for a in 1_u64..25 {
        for b in a..25 {
            for c in b..25 {
                let sum = a * a + b * b + c * c;
                let root = sum.isqrt();
                if root * root == sum {
                    tuples.push((vec![a as f64, b as f64, c as f64], root as f64));
                }
            }
        }
    }
    for a in 1_u64..10 {
        for b in a..10 {
            for c in b..10 {
                for d in c..10 {
                    let sum = a * a + b * b + c * c + d * d;
                    let root = sum.isqrt();
                    if root * root == sum {
                        tuples.push((vec![a as f64, b as f64, c as f64, d as f64], root as f64));
                    }
                }
            }
        }
    }
    tuples
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[test]
fn only_the_appended_cases_call_math_hypot_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(calls_hypot(&case.source), new, "{}", case.title);
        if new {
            let count =
                common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            assert!(case.expected.is_some(), "{} declares a return", case.title);
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
    assert_eq!(assertions, 29);
    assert_eq!(cases, 1347);
}

#[test]
fn every_appended_case_gives_what_its_accepted_spelling_gives() {
    let documents = documents();
    let mut compared = 0;
    let mut calls = 0;
    for case in documents
        .iter()
        .flat_map(|(_, cases)| cases)
        .filter(|case| is_appended(&case.title))
    {
        for (written, _) in SPELLINGS {
            calls += case.source.matches(written).count();
        }
        let spelled = spelled_out(&case.source);
        assert!(!calls_hypot(&spelled), "{}", case.title);
        let assertions = common::assertions::protected_assertions(&parse(&case.source).unwrap());
        let spelled_assertions =
            common::assertions::protected_assertions(&parse(&spelled).unwrap());
        assert_eq!(assertions, spelled_assertions, "{}", case.title);
        for source in [&case.source, &spelled] {
            common::check_with_assertions(
                &common::Case {
                    title: case.title.clone(),
                    source: source.clone(),
                    expected: case.expected.clone(),
                },
                assertions.len(),
            )
            .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
        }
        compared += 1;
    }
    assert_eq!(compared, 7);
    assert_eq!(calls, 10);
}

#[test]
fn the_audit_finds_every_hypot_call_and_nothing_else() {
    for source in [
        "Math.hypot(3, 4)",
        "Math.hypot()",
        "d = Math.hypot(a)",
        "$P.r = Math.hypot($P.x, $P.y)",
        "xs.map(p -> Math.hypot(p.x, p.y))",
        "`${Math.hypot(3, 4)} m`",
        "`${f({a: Math.hypot(1, 2)})}`",
        "x = ok ? Math.hypot(1, 1) : 0",
    ] {
        assert!(calls_hypot(source), "{source}");
    }
    for source in [
        "Math.hypot",
        "f = Math.hypot",
        "hypot(3, 4)",
        "o.hypot(3, 4)",
        "Math.Hypot(3, 4)",
        "Math.hypotenuse(3, 4)",
        "String.hypot(3, 4)",
        "\"Math.hypot(3, 4)\"",
        "# Math.hypot(3, 4)\nx = 1",
        "Math[\"hypot\"]",
        "def hypot(a, b) => a + b",
        "def hypotenuse(a, b) => (a ** 2 + b ** 2) ** 0.5",
    ] {
        assert!(!calls_hypot(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_together_after_nuc_157_and_before_nuc_139() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    for (cases, own) in [
        (&spec, &SPEC_TITLES[..]),
        (&reference, &[REFERENCE_TITLE][..]),
        (&readme, &[README_TITLE][..]),
        (&examples, &[EXAMPLES_TITLE][..]),
    ] {
        let first = cases
            .iter()
            .position(|case| case.title == own[0])
            .unwrap_or_else(|| panic!("{} is appended", own[0]));
        for (offset, title) in own.iter().enumerate() {
            assert_eq!(
                cases.get(first + offset).map(|case| case.title.as_str()),
                Some(*title),
                "{title} stands with the other appended cases"
            );
        }
        let padding = cases
            .iter()
            .rposition(|case| calls_padding_alias(&case.source))
            .expect("the cases of NUC 157 stand in every document");
        let xor = cases
            .iter()
            .position(|case| writes_xor_stage(&case.source))
            .expect("the cases of NUC 139 stand in every document");
        let bang_in = cases
            .iter()
            .position(|case| writes_bang_in(&case.source))
            .expect("the cases of NUC 135 stand in every document");
        assert_eq!(padding + 1, first, "{}", own[0]);
        assert_eq!(first + own.len(), xor, "{}", own[0]);
        assert!(first + own.len() <= bang_in, "{}", own[0]);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Reactive hypotenuse\n\n```\n")
        .expect("the language reference documents Math.hypot");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(case.expected.as_deref(), Some("29"));
    for line in [
        "[NUC 159](nuc-0159.md)",
        "`copySign(value, sign)` ([NUC 142](nuc-0142.md)), `hypot(value, ...)` ([NUC 159](nuc-0159.md)) |",
        "Math.hypot(value, ...)                   static square root of the sum of the squares, without overflow (NUC 159)",
        "[hypotenuse contract](../nucleoid.spec.md#reactive-hypotenuse)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    let specification = include_str!("../nucleoid.spec.md").replace("\r\n", "\n");
    let (_, contract) = specification
        .split_once("\n## Reactive hypotenuse\n")
        .expect("the specification states Math.hypot");
    let (contract, _) = contract.split_once("\n## ").unwrap();
    let [first, .., last] = SPEC_TITLES;
    for title in [first, last] {
        assert!(contract.contains(&format!("`{title}`")), "{title}");
    }
    assert!(contract.contains("`tests/math_hypot.rs`"));
    assert!(contract.contains("[NUC 159](docs/nuc-0159.md)"));
}

#[test]
fn every_argument_count_gives_the_euclidean_norm() {
    for (source, expected) in [
        ("Math.hypot()", "0.0"),
        ("Math.hypot(-7)", "7.0"),
        ("Math.hypot(3, 4)", "5.0"),
        ("Math.hypot(1, 2, 2)", "3.0"),
        ("Math.hypot(1, 1, 1, 1)", "2.0"),
        ("Math.hypot(1, 1, 1, 1, 12)", "12.165525060596439"),
        ("Math.hypot(2, 4, 5, 6, 0)", "9.0"),
        ("Math.hypot(3, 4, 12, 84)", "85.0"),
        ("Math.hypot(1.5, 2)", "2.5"),
        ("Math.hypot(0.1, 0.2)", "0.223606797749979"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    let mut numbers = Numbers(0x9E37_79B9_7F4A_7C15);
    for count in 0..=5 {
        for _ in 0..200 {
            let values: Vec<f64> = (0..count).map(|_| numbers.magnitude(-60, 60)).collect();
            same_bits(hypot(&values), direct(&values), &format!("{values:?}"));
            let listed: Vec<String> = values.iter().map(|value| format!("{value:e}")).collect();
            let written = format!("Math.hypot({})", listed.join(", "));
            let squares: Vec<String> = listed
                .iter()
                .map(|value| format!("({value}) * ({value})"))
                .collect();
            let spelled = if squares.is_empty() {
                "Math.sqrt(0)".to_string()
            } else {
                format!("Math.sqrt({})", squares.join(" + "))
            };
            assert_eq!(fresh(&written), fresh(&spelled), "{written}");
        }
    }
}

#[test]
fn the_direct_range_is_exactly_math_sqrt_of_the_sum_of_squares_in_order() {
    let mut numbers = Numbers(0x2545_F491_4F6C_DD1D);
    for count in 1..=5 {
        for _ in 0..2000 {
            let values: Vec<f64> = (0..count).map(|_| numbers.magnitude(-449, 449)).collect();
            same_bits(hypot(&values), direct(&values), &format!("{values:?}"));
        }
    }
    let bound = power_of_two(450);
    for values in [
        vec![bound, bound],
        vec![bound, 1.0, -bound],
        vec![power_of_two(-450), power_of_two(-450)],
        vec![power_of_two(-450), power_of_two(-1074)],
        vec![bound, power_of_two(-1074), 3.0],
    ] {
        same_bits(hypot(&values), direct(&values), &format!("{values:?}"));
    }
}

#[test]
fn scaling_every_argument_by_a_power_of_two_scales_the_result_exactly() {
    let mut numbers = Numbers(0xD1B5_4A32_D192_ED03);
    let mut scaled = 0;
    for _ in 0..3000 {
        let count = 1 + usize::try_from(numbers.next() % 5).unwrap();
        let values: Vec<f64> = (0..count).map(|_| numbers.magnitude(-100, 100)).collect();
        let base = hypot(&values);
        for shift in [-920, -600, -460, -451, 451, 460, 600, 900] {
            let factor = power_of_two(shift);
            let moved: Vec<f64> = values.iter().map(|value| value * factor).collect();
            same_bits(
                hypot(&moved),
                base * factor,
                &format!("{values:?} by 2^{shift}"),
            );
            scaled += 1;
        }
    }
    assert_eq!(scaled, 24000);
}

#[test]
fn pythagorean_tuples_and_their_power_of_two_multiples_are_exact() {
    let tuples = pythagorean_tuples();
    assert!(tuples.len() > 200, "{}", tuples.len());
    for (values, root) in &tuples {
        same_bits(hypot(values), *root, &format!("{values:?}"));
        let negated: Vec<f64> = values.iter().map(|value| -value).collect();
        same_bits(hypot(&negated), *root, &format!("{negated:?}"));
    }
    let mut checked = 0;
    for (values, root) in tuples.iter().take(40) {
        for exponent in -1074..=1023 {
            let factor = power_of_two(exponent);
            let expected = root * factor;
            if !expected.is_finite() {
                continue;
            }
            let moved: Vec<f64> = values.iter().map(|value| value * factor).collect();
            same_bits(
                hypot(&moved),
                expected,
                &format!("{values:?} by 2^{exponent}"),
            );
            checked += 1;
        }
    }
    assert!(checked > 70000, "{checked}");
    for (values, root) in [
        (vec![8.0, 15.0], 17.0),
        (vec![20.0, 15.0], 25.0),
        (vec![20.0, 21.0], 29.0),
        (vec![-6.0, 8.0], 10.0),
        (vec![15.0, 8.0], 17.0),
        (vec![-15.0, 8.0], 17.0),
        (vec![6.0, 8.0], 10.0),
        (vec![3.0, 4.0], 5.0),
        (vec![2.0, 3.0, 6.0], 7.0),
        (vec![6.0, 3.0, 6.0], 9.0),
        (vec![6.0, 7.0, 6.0], 11.0),
        (vec![1.0, 2.0, 2.0], 3.0),
        (vec![-2.0, 6.0, -9.0], 11.0),
        (vec![2.0, 9.0, 6.0], 11.0),
        (vec![-3.0, 4.0, 12.0], 13.0),
        (vec![5.0, -12.0], 13.0),
        (vec![6.0, 4.0, 12.0], 14.0),
        (vec![-9.0, -12.0], 15.0),
        (vec![15.0, 20.0], 25.0),
        (vec![12.0, 5.0], 13.0),
        (vec![-12.0, -9.0], 15.0),
        (vec![12.0, 9.0], 15.0),
        (vec![-12.0, -5.0], 13.0),
    ] {
        same_bits(hypot(&values), root, &format!("{values:?}"));
    }
}

#[test]
fn infinity_wins_over_nan_and_neither_sign_nor_zero_changes_the_result() {
    let infinite = [f64::INFINITY, f64::NEG_INFINITY];
    for count in 1..=4 {
        for position in 0..count {
            for infinity in infinite {
                for filler in [f64::NAN, 3.0, -0.0, f64::MAX] {
                    let mut values = vec![filler; count];
                    values[position] = infinity;
                    same_bits(hypot(&values), f64::INFINITY, &format!("{values:?}"));
                }
            }
            let mut values = vec![2.0; count];
            values[position] = f64::NAN;
            assert!(hypot(&values).is_nan(), "{values:?}");
        }
    }
    for values in [
        vec![],
        vec![0.0],
        vec![-0.0],
        vec![0.0, -0.0],
        vec![-0.0, -0.0, -0.0],
    ] {
        same_bits(hypot(&values), 0.0, &format!("{values:?}"));
    }
    for signs in 0..8_u32 {
        let values: Vec<f64> = [2.0, 3.0, 6.0]
            .iter()
            .enumerate()
            .map(|(index, value)| {
                if signs & (1 << index) == 0 {
                    *value
                } else {
                    -value
                }
            })
            .collect();
        same_bits(hypot(&values), 7.0, &format!("{values:?}"));
    }
    for (source, expected) in [
        ("1 / Math.hypot()", "null"),
        (
            "[1 / Math.hypot() > 0, 1 / Math.hypot(-0) > 0, 1 / Math.hypot(0, -0) > 0]",
            "[true,true,true]",
        ),
        ("Number.isNaN(Math.hypot(0 / 0, 3))", "true"),
        (
            "Math.hypot(0 / 0, -1 / 0) == Number.POSITIVE_INFINITY",
            "true",
        ),
        (
            "Math.hypot(1 / 0, 0 / 0) == Number.POSITIVE_INFINITY",
            "true",
        ),
        (
            "Math.hypot(\"x\", -1 / 0) == Number.POSITIVE_INFINITY",
            "true",
        ),
        (
            "Math.hypot(\"-inf\", 2) == Number.POSITIVE_INFINITY",
            "true",
        ),
        ("Number.isNaN(Math.hypot(\"x\", 4))", "true"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn large_and_small_magnitudes_neither_overflow_nor_vanish() {
    for (values, expected) in [
        (vec![1e200, 1e200], 1e200 * std::f64::consts::SQRT_2),
        (vec![3e-200, 4e-200], 5e-200),
        (vec![1e-300, 1e300], 1e300),
        (vec![f64::MAX, 0.0], f64::MAX),
        (vec![f64::MAX, f64::MAX], f64::INFINITY),
        (vec![f64::MIN_POSITIVE, 0.0], f64::MIN_POSITIVE),
        (vec![5e-324, 5e-324], 5e-324),
        (vec![3e-320, 4e-320], 5e-320),
        (vec![power_of_two(511); 4], power_of_two(512)),
        (vec![power_of_two(-600); 4], power_of_two(-599)),
    ] {
        same_bits(hypot(&values), expected, &format!("{values:?}"));
    }
    for (source, expected) in [
        ("Math.hypot(1e200, 1e200)", "1.414213562373095e+200"),
        ("(1e200 ** 2 + 1e200 ** 2) ** 0.5", "null"),
        ("Math.hypot(3e-200, 4e-200)", "5e-200"),
        ("(3e-200 ** 2 + 4e-200 ** 2) ** 0.5", "0.0"),
        ("Math.hypot(1e-300, 1e300)", "1e+300"),
        ("Math.hypot(1.7e308, 1.7e308)", "null"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn arguments_convert_as_the_arguments_of_math_max_do() {
    let mut defined = 0;
    for (setup, kind) in KINDS {
        let written = format!("{setup}\n[Math.hypot({kind}, 4), Math.hypot(4, {kind})]");
        let spelled = format!(
            "{setup}\n[Math.sqrt(Math.square({kind}) + 16), Math.sqrt(16 + Math.square({kind}))]"
        );
        let found = fresh(&written);
        assert_eq!(found, fresh(&spelled), "{written}");
        let max = format!(
            "{setup}\nNumber.isNaN(Math.max({kind}, 4)) == Number.isNaN(Math.hypot({kind}, 4))"
        );
        assert_eq!(fresh(&max), "true", "{max}");
        defined += usize::from(found != "[null,null]");
    }
    assert_eq!(defined, 19);
    for (source, expected) in [
        ("Math.hypot(\"3\", 4, true, null)", "5.0990195135927845"),
        (
            "Math.sqrt(Math.square(\"3\") + Math.square(4) + Math.square(true) + Math.square(null))",
            "5.0990195135927845",
        ),
        ("Math.hypot([3], 4)", "5.0"),
        ("Math.hypot([[3]], [4])", "5.0"),
        ("Math.hypot(\" 3 \", \"4\")", "5.0"),
        ("Math.hypot(undefined, 5)", "5.0"),
        ("Math.hypot(Date(3), Date(4))", "5.0"),
        ("Math.hypot(\"\", [], false, 0)", "0.0"),
        ("Math.hypot(\"1e3\", 0)", "1000.0"),
        ("Math.hypot([1, 2])", "null"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn the_undefined_gate_and_argument_errors_keep_their_order() {
    for source in [
        "o = {}\nMath.hypot(o.missing, 4)",
        "o = {}\nMath.hypot(3, o.missing)",
        "o = {}\nMath.hypot(o.missing)",
        "o = {}\nMath.hypot(3, 4, 12, o.missing, 1)",
        "o = {}\nMath.hypot(0 / 0, o.missing)",
        "o = {}\nMath.hypot(1 / 0, o.missing)",
    ] {
        assert_eq!(result(source), "null", "{source}");
        assert_eq!(
            Runtime::new().run(source).unwrap(),
            Value::Undefined,
            "{source}"
        );
    }
    for source in [
        "Math.hypot(3, missing)",
        "Math.hypot(missing, 4)",
        "o = {}\nMath.hypot(o.missing, missing)",
        "o = {}\nMath.hypot(missing, o.missing)",
        "Math.hypot(1 / 0, missing)",
    ] {
        assert_eq!(
            failure(source),
            format!(
                "ReferenceError: missing is not defined (line {}, column 1)",
                source.lines().count()
            ),
            "{source}"
        );
    }
    let functions = "log = []\ndef first():\n    log.push(\"first\")\n    return 3\ndef second():\n    log.push(\"second\")\n    return 4\n";
    assert_eq!(
        result(&format!("{functions}[Math.hypot(first(), second()), log]")),
        "[5.0,[\"first\",\"second\"]]"
    );
    assert_eq!(
        result(&format!(
            "{functions}o = {{}}\n[Math.hypot(o.missing, first(), second()), log]"
        )),
        "[null,[\"first\",\"second\"]]"
    );
    let mut runtime = Runtime::new();
    runtime.run(functions).unwrap();
    assert_eq!(
        runtime
            .run("Math.hypot(first(), missing, second())")
            .unwrap_err()
            .to_string(),
        "ReferenceError: missing is not defined (line 1, column 1)"
    );
    assert_eq!(json(&mut runtime, "log"), "[]");
}

#[test]
fn the_public_math_takes_hypot() {
    let math = builtins::math;
    for (arguments, expected) in [
        (Vec::new(), 0.0),
        (vec![Value::Number(3.0), Value::Number(4.0)], 5.0),
        (vec![Value::Number(-7.0)], 7.0),
        (
            vec![
                Value::string("3"),
                Value::Number(4.0),
                Value::Bool(true),
                Value::Null,
            ],
            26.0_f64.sqrt(),
        ),
        (
            vec![Value::List(vec![Value::Number(3.0)]), Value::Date(4)],
            5.0,
        ),
        (
            vec![
                Value::List(Vec::new()),
                Value::string(""),
                Value::Bool(false),
            ],
            0.0,
        ),
        (
            vec![Value::Number(f64::NEG_INFINITY), Value::Number(f64::NAN)],
            f64::INFINITY,
        ),
    ] {
        same_bits(
            number(math("hypot", &arguments).unwrap()),
            expected,
            &format!("{arguments:?}"),
        );
    }
    for arguments in [
        vec![Value::string("x"), Value::Number(4.0)],
        vec![
            Value::Object(ObjectId::named("unconverted")),
            Value::Number(4.0),
        ],
        vec![Value::Class("Probe".to_string())],
        vec![Value::List(vec![Value::Number(1.0), Value::Number(2.0)])],
        vec![Value::Number(f64::NAN), Value::Number(3.0)],
    ] {
        assert!(
            number(math("hypot", &arguments).unwrap()).is_nan(),
            "{arguments:?}"
        );
    }
    for arguments in [
        vec![Value::Undefined],
        vec![Value::Number(3.0), Value::Undefined],
        vec![Value::Number(f64::INFINITY), Value::Undefined],
        vec![Value::Undefined, Value::Number(f64::NAN)],
    ] {
        assert_eq!(
            math("hypot", &arguments).unwrap(),
            Value::Undefined,
            "{arguments:?}"
        );
    }
    for name in ["Hypot", "hypotenuse", "HYPOT", "dist", "norm"] {
        let error = math(name, &[Value::Number(3.0), Value::Number(4.0)]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), format!("Math.{name} is not a function"));
        assert_eq!(math(name, &[Value::Undefined]).unwrap(), Value::Undefined);
    }
    assert_eq!(builtins::math_constant("hypot"), None);
}

#[test]
fn declarations_class_level_rules_and_mappings_stay_live() {
    let first = "a = {x: 1, y: 2}\nb = {x: 4, y: 6}\nd = Math.hypot(b.x - a.x, b.y - a.y)\nassert(d, 5)\nb.x = 6\nb.y = 14\nassert(d, 13)\na = {x: -3, y: 2}\nassert(d, 15)";
    let mut runtime = Runtime::new();
    let mut spelled = Runtime::new();
    checked(&mut runtime, first);
    checked(
        &mut spelled,
        &first.replace(
            "Math.hypot(b.x - a.x, b.y - a.y)",
            "((b.x - a.x) ** 2 + (b.y - a.y) ** 2) ** 0.5",
        ),
    );
    assert_eq!(dependencies(&runtime, "d"), dependencies(&spelled, "d"));
    assert_eq!(
        dependencies(&runtime, "d"),
        ["b", "obj:0.x", "a", "obj:1.x", "obj:0.y", "obj:1.y"]
    );
    checked(
        &mut runtime,
        "class Point(x, y):\n    this.x = x\n    this.y = y\n$Point.r = Math.hypot($Point.x, $Point.y)\np = Point(3, 4)\nq = Point(5, 12)\nassert([p.r, q.r], [5, 13])\nq.x = 9\nq.y = 40\nassert([p.r, q.r], [5, 41])\nradii = Point.map(point -> point.r)\nfar = Point.filter(point -> Math.hypot(point.x, point.y) > 10).length\nPoint(20, 21)\nassert([radii, far], [[5, 41, 29], 2])",
    );
    checked(
        &mut runtime,
        "steps = [{x: 3, y: 4}]\nlengths = steps.map(step -> Math.hypot(step.x, step.y))\nassert(lengths, [5])\nsteps.push({x: 6, y: 8}, {x: 20, y: 21})\nassert(lengths, [5, 10, 29])\nsteps[0].x = -3\nassert(lengths, [5, 10, 29])",
    );
    checked(
        &mut runtime,
        "dx = 3\ndy = 4\nscale = 2\nscaled = Math.hypot(dx * scale, dy * scale)\ndef norm2(a, b) => Math.hypot(a, b)\nviaFunction = norm2(dx, dy)\nassert([scaled, viaFunction], [10, 5])\ndx = 6\ndy = 8\nassert([scaled, viaFunction], [20, 10])",
    );
}

#[test]
fn null_bindings_defer_declarations_as_for_every_math_call() {
    for (setup, declaration, write, expected) in [
        ("x = null", "d = Math.hypot(x, 4)", "x = 3", "5.0"),
        ("x = null", "d = Math.hypot(4, x)", "x = 3", "5.0"),
        ("o = {x: null}", "d = Math.hypot(o.x, 4)", "o.x = 3", "5.0"),
    ] {
        let mut hypot = Runtime::new();
        let mut max = Runtime::new();
        hypot.run(&format!("{setup}\n{declaration}")).unwrap();
        max.run(&format!("{setup}\n{}", declaration.replace("hypot", "max")))
            .unwrap();
        assert_eq!(json(&mut hypot, "d"), "null", "{declaration}");
        assert_eq!(json(&mut max, "d"), "null", "{declaration}");
        hypot.run(write).unwrap();
        assert_eq!(json(&mut hypot, "d"), expected, "{declaration}");
    }
    assert_eq!(result("Math.hypot(null, 4)"), "4.0");
}

#[test]
fn explanations_and_renderings_show_the_call_as_written() {
    let mut runtime = Runtime::new();
    runtime
        .run("a = 3\nb = 4\nc = 12\nd = Math.hypot(a, b)\ne = Math.hypot(a, b, c)")
        .unwrap();
    assert_eq!(
        json(&mut runtime, "(why d)[0].rule"),
        "\"d = Math.hypot(a,b)\""
    );
    assert_eq!(json(&mut runtime, "(why d)[0].from"), "[\"a\",\"b\"]");
    assert_eq!(json(&mut runtime, "(why e)[0].holds"), "13.0");
    assert_eq!(
        json(&mut runtime, "(why e).map(step -> step.node)"),
        "[\"e\",\"a\",\"b\",\"c\"]"
    );
    assert_eq!(
        json(&mut runtime, "(affects a).map(step -> step.node)"),
        "[\"d\",\"e\"]"
    );
    assert_eq!(
        json(&mut runtime, "(affects c).map(step -> step.node)"),
        "[\"e\"]"
    );
    assert_eq!(dependencies(&runtime, "e"), ["a", "b", "c"]);
    for (source, rendered) in [
        ("d = Math.hypot(a, b)", "d=Math.hypot(a,b)"),
        ("d = Math.hypot()", "d=Math.hypot()"),
        (
            "$P.r = Math.hypot($P.x - o.x, $P.y)",
            "$P.r=Math.hypot($P.x-o.x,$P.y)",
        ),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), rendered, "{source}");
    }
}

#[test]
fn failed_updates_roll_back_through_math_hypot() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "p = [3]\nd = Math.hypot(p.at(0), 4)\nfailure = \"\"\ntry:\n    p = 7\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.at is not a function\"))\nassert([p, d], [[3], 5])",
    );
    checked(
        &mut runtime,
        "legs = []\nlengths = legs.map(leg -> Math.hypot(leg.dx, leg.dy, rise))\ntry:\n    legs.push({dx: 2, dy: 3})\ncatch error:\n    failure = error\nassert(failure, ReferenceError(\"rise is not defined\"))\nassert([legs, lengths], [[], []])\nrise = 6\nlegs.push({dx: 2, dy: 3})\nassert(lengths, [7])",
    );
    checked(
        &mut runtime,
        "dx = 6\ndy = 8\nstep = Math.hypot(dx, dy)\ntry:\n    dx = 15\n    legs.push({dx: 6, dy: 7})\n    assert([step, lengths], [17, [7, 11]])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([step, dx, lengths], [10, 6, [7]])",
    );
    assert_eq!(dependencies(&runtime, "step"), ["dx", "dy"]);
    assert_eq!(
        failure("p = [3]\nd = Math.hypot(p.at(0), 4)\np = 7"),
        "TypeError: number.at is not a function (line 3, column 1)"
    );
    let mut recovered = Runtime::new();
    recovered
        .run("ps = []\nds = ps.map(p -> Math.hypot(p.x, size))")
        .unwrap();
    assert_eq!(
        recovered.run("ps.push({x: 3})").unwrap_err().to_string(),
        "ReferenceError: size is not defined (line 1, column 1)"
    );
    assert_eq!(json(&mut recovered, "[ps, ds]"), "[[],[]]");
    recovered.run("size = 4\nps.push({x: 3})").unwrap();
    assert_eq!(json(&mut recovered, "ds"), "[5.0]");
}

#[test]
fn math_hypot_reads_in_every_statement_position() {
    for (source, expected) in [
        (
            "r = 0\ntry:\n    r = Math.hypot(3, missing)\ncatch error:\n    r = error\nr",
            "\"ReferenceError: missing is not defined\"",
        ),
        (
            "x = 3\nd = Math.hypot(x, 4)\nseen = suppose x = 0: d\n[seen, d, x]",
            "[4.0,5.0,3.0]",
        ),
        ("x = 3\n`${Math.hypot(x, 4)}!`", "\"5!\""),
        (
            "x = 3\nif Math.hypot(x, 4) == 5:\n    kind = \"exact\"\nelse:\n    kind = \"rounded\"\nkind",
            "\"exact\"",
        ),
        (
            "x = 3\nMath.hypot(x, 4) > 4 ? \"far\" : \"near\"",
            "\"far\"",
        ),
        (
            "def dist(a, b) => Math.hypot(a, b)\n[dist(3, 4), dist(8, 15)]",
            "[5.0,17.0]",
        ),
        (
            "def norm(v):\n    return Math.hypot(v.x, v.y, v.z)\nnorm({x: 2, y: 3, z: 6})",
            "7.0",
        ),
        (
            "class V(x, y):\n    this.x = x\n    this.y = y\n    def norm() => Math.hypot(this.x, this.y)\n    def magnitude() => Math.hypot(self.x, self.y)\nv = V(5, 12)\n[v.norm(), v.magnitude()]",
            "[13.0,13.0]",
        ),
        (
            "ps = [[3, 4], [1, 1], [6, 8]]\nps.filter(p -> Math.hypot(p[0], p[1]) > 2).map(p -> Math.hypot(p[0], p[1]))",
            "[5.0,10.0]",
        ),
        (
            "ps = [[3, 4], [6, 8]]\n[ps.some(p -> Math.hypot(p[0], p[1]) == 10), ps.every(p -> Math.hypot(p[0], p[1]) >= 5)]",
            "[true,true]",
        ),
        (
            "i = 0\nwhile Math.hypot(i, 4) < 5:\n    i = i + 1\ni",
            "3.0",
        ),
        (
            "class P(x, y):\n    this.x = x\n    this.y = y\nif Math.hypot($P.x, $P.y) > 10:\n    $P.far = true\np = P(3, 4)\nq = P(9, 40)\n[p.far, q.far]",
            "[null,true]",
        ),
        ("Math.hypot(Math.hypot(3, 4), 12)", "13.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    assert_eq!(
        result("r = \"\"\ntry:\n    r = Math.hypot(3, 4)\ncatch e:\n    r = \"caught\"\nr"),
        "5.0"
    );
    assert_eq!(
        result("r = \"\"\ntry:\n    r = Math.Hypot(3, 4)\ncatch e:\n    r = \"caught\"\nr"),
        "\"caught\""
    );
}

#[test]
fn the_kept_readings_keep_their_exact_accepted_results() {
    assert_eq!(Runtime::new().run("Math.hypot").unwrap(), Value::Undefined);
    for (source, expected) in [
        ("Math.hypot", "null"),
        ("f = Math.hypot\nf", "null"),
        ("typeof Math.hypot", "\"undefined\""),
        ("Math[\"hypot\"]", "null"),
        ("o = {hypot: (a, b) -> a + b}\no.hypot(3, 4)", "7.0"),
        ("o = {hypot: 5}\no.hypot", "5.0"),
        ("def hypot(a, b) => a + b\nhypot(3, 4)", "7.0"),
        ("hypot = 5\nhypot + 1", "6.0"),
        (
            "class Geo:\n    def hypot(a, b) => a * b\ng = Geo()\ng.hypot(3, 4)",
            "12.0",
        ),
        (
            "def hypotenuse(a, b) => (a ** 2 + b ** 2) ** 0.5\nhypotenuse(3, 4)",
            "5.0",
        ),
        ("x = \"dist\"\n[x is hypot, x is dist]", "[false,true]"),
        (
            "namespace = {hypot: (a, b) -> a * b}\ndef scoped(Math):\n    return Math.hypot(3, 4)\nscoped(namespace)",
            "12.0",
        ),
        ("u = null\nu.hypot(3)", "null"),
        ("Math.sqrt(25)", "5.0"),
        ("Math.max(3, 4, 12)", "12.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "Math.Hypot(3, 4)",
            "TypeError: Math.Hypot is not a function (line 1, column 1)",
        ),
        (
            "Math.hypotenuse(3, 4)",
            "TypeError: Math.hypotenuse is not a function (line 1, column 1)",
        ),
        (
            "String.hypot(3, 4)",
            "TypeError: String.hypot is not a function (line 1, column 1)",
        ),
        (
            "x = 3\ny = x.hypot(4)",
            "TypeError: number.hypot is not a function (line 2, column 1)",
        ),
        (
            "xs = [3, 4]\ny = xs.hypot()",
            "TypeError: xs.hypot is not a function (line 2, column 1)",
        ),
        (
            "true.hypot(1)",
            "TypeError: boolean.hypot is not a function (line 1, column 1)",
        ),
        (
            "\"ab\".hypot(1)",
            "TypeError: ab.hypot is not a function (line 1, column 1)",
        ),
        (
            "class Box\nBox.hypot(1)",
            "TypeError: Box.hypot is not a function (line 2, column 1)",
        ),
        (
            "hypot(3, 4)",
            "ReferenceError: hypot is not defined (line 1, column 1)",
        ),
        (
            "Math.hypot = 1",
            "ReferenceError: Math is not defined (line 1, column 1)",
        ),
        (
            "Math = 7\nMath.hypot(3, 4)",
            "TypeError: number.hypot is not a function (line 2, column 1)",
        ),
        (
            "Math.random()",
            "TypeError: Math.random is not supported (line 1, column 1)",
        ),
        (
            "Math.toDegrees(3)",
            "TypeError: Math.toDegrees is not a function (line 1, column 1)",
        ),
    ] {
        assert_eq!(failure(source), expected, "{source}");
    }
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-math-hypot-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_math_hypot() {
    let program = "class Pt(x, y):\n    this.x = x\n    this.y = y\na = Pt(1, 1)\nb = Pt(4, 5)\nd = Math.hypot(b.x - a.x, b.y - a.y)\nsteps = [{x: 3, y: 4}]\nlengths = steps.map(s -> Math.hypot(s.x, s.y))\nb.x = 7\nb.y = 9\nsteps.push({x: 2, y: 3})\n[d, lengths, Math.hypot(2, 3, 6), Math.hypot()]";
    let spelled = program
        .replace(
            "Math.hypot(b.x - a.x, b.y - a.y)",
            "((b.x - a.x) ** 2 + (b.y - a.y) ** 2) ** 0.5",
        )
        .replace("Math.hypot(s.x, s.y)", "Math.sqrt(s.x * s.x + s.y * s.y)")
        .replace("Math.hypot(2, 3, 6)", "Math.sqrt(4 + 9 + 36)")
        .replace("Math.hypot()", "Math.sqrt(0)");
    assert!(!calls_hypot(&spelled), "{spelled}");
    let file = SourceFile::new("file");
    for options in [
        &[][..],
        &["--json"][..],
        &["--graph"][..],
        &["--json", "--graph"][..],
    ] {
        let written = file.run(program, options);
        let accepted = file.run(&spelled, options);
        assert!(written.status.success(), "{options:?} {written:?}");
        assert!(written.stderr.is_empty(), "{options:?} {written:?}");
        assert_eq!(written.stdout, accepted.stdout, "{options:?}");
    }
    assert_eq!(
        String::from_utf8(file.run(program, &["--json"]).stdout).unwrap(),
        "[10.0,[5.0,3.605551275463989],7.0,0.0]\n"
    );
    assert_eq!(
        String::from_utf8(file.run(program, &[]).stdout).unwrap(),
        "10,5,3.605551275463989,7,0\n"
    );
    let graph = String::from_utf8(
        file.run("a = 3\nb = 4\nd = Math.hypot(a, b)\nd", &["--graph"])
            .stdout,
    )
    .unwrap();
    assert!(graph.contains("d [variable]\n    reads    a, b"), "{graph}");
    for (source, message) in [
        (
            "Math.hypot(3, missing)\n",
            "ReferenceError: missing is not defined (line 1, column 1)\n",
        ),
        (
            "x = 1\nMath.Hypot(3)\n",
            "TypeError: Math.Hypot is not a function (line 2, column 1)\n",
        ),
    ] {
        let failed = file.run(source, &["--json"]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(String::from_utf8(failed.stderr).unwrap(), message);
    }
    let session = "steps = [{x: 3, y: 4}]\nlengths = steps.map(s -> Math.hypot(s.x, s.y))\nlengths\nsteps.push({x: 6, y: 8})\nlengths\nMath.hypot(1, 2, 2)\nMath.hypot(-0)\nMath.hypot(1e200, 1e200) > 1e200\n.exit\n";
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{options:?} {output:?}");
            assert!(output.stderr.is_empty(), "{options:?} {output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "[5.0]\n2.0\n[5.0,10.0]\n3.0\n0.0\ntrue\n"
            } else {
                "5\n2\n5,10\n3\n0\ntrue\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
        }
    }
    let failing = repl(
        "Math.hypot(missing)\nMath.hypot(8, 15)\n.exit\n",
        &["--plain"],
    );
    assert!(failing.status.success(), "{failing:?}");
    assert_eq!(
        String::from_utf8(failing.stderr).unwrap(),
        "ReferenceError: missing is not defined (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(failing.stdout).unwrap();
    assert!(stdout.contains("17\n"), "{stdout}");
}
