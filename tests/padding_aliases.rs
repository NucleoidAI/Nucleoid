mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Spanned, Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime, Value};

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid pads strings with Python's and Ruby's rjust and ljust",
    "Nucleoid pads strings with C#'s PadLeft and PadRight",
    "Nucleoid keeps padding aliases live in class level rules",
    "Nucleoid rolls back failed updates through padding aliases",
];
const REFERENCE_TITLE: &str = "Nucleoid pads strings with rjust, ljust, PadLeft and PadRight";
const README_TITLE: &str = "Padding aliases keep a help desk's ticket board aligned";
const EXAMPLES_TITLE: &str = "Padding aliases keep every order's receipt line aligned";

const PAIRS: [(&str, &str); 4] = [
    ("rjust", "padStart"),
    ("PadLeft", "padStart"),
    ("ljust", "padEnd"),
    ("PadRight", "padEnd"),
];

const ALIASES: [&str; 4] = ["rjust", "PadLeft", "ljust", "PadRight"];

const SETUP: &str = "log = []";

const TEXTS: [(&str, &str); 30] = [
    ("", "\"7\""),
    ("", "\"ab\""),
    ("", "\"\""),
    ("", "\"x\""),
    ("", "\"long text\""),
    ("", "\"Zoë\""),
    ("", "\"añejo\""),
    ("", "\"😀\""),
    ("", "\"e\u{301}\""),
    ("", "\"日本\""),
    ("", "\"aΩ日😀\""),
    ("s = \"ab\"", "s"),
    ("o = {code: \"9\"}", "o.code"),
    (
        "class Tag(code):\n    this.code = code\nt = Tag(\"42\")",
        "t.code",
    ),
    ("def code() => \"77\"", "code()"),
    ("", "\"a,bc\".split(\",\")[1]"),
    ("xs = [\"a\", \"b\"]", "xs.map(x -> x + \"!\")[1]"),
    ("", "\"hello\".slice(0, 2)"),
    ("", "\"hello\".substring(3)"),
    ("xs = [\"a\", \"b\"]", "xs.join(\"-\")"),
    ("", "\"ab\".repeat(2)"),
    ("stem = \"ab\"", "`${stem}C`"),
    ("xs = [\"ab\", \"cd\"]", "xs[1]"),
    ("s = \"Ab\"\ns = \"Cd\"", "s@first"),
    ("s = \"Ab\"\ns = \"Cd\"", "s@last"),
    ("s = \"Ab\"", "s.value"),
    ("flag = true", "(flag ? \"Yes\" : \"No\")"),
    ("", "String(42)"),
    ("", "(typeof 5)"),
    ("", "\"x\".padStart(2, \"-\")"),
];

const ARGUMENTS: [&str; 10] = [
    "",
    "5",
    "5, \"0\"",
    "6, \"ab\"",
    "4, \"\"",
    "0, \"x\"",
    "-2, \"x\"",
    "\"4\", 0",
    "7, \"😀é\"",
    "5, \"-\", log.push(\"extra\")",
];

const LENGTHS: [&str; 22] = [
    "0",
    "1",
    "3",
    "7",
    "2.9",
    "-3",
    "-0.5",
    "\"4\"",
    "\" 3 \"",
    "\"x\"",
    "0 / 0",
    "Number.NaN",
    "null",
    "undefined",
    "true",
    "false",
    "[3]",
    "[]",
    "[1, 2]",
    "{}",
    "o.missing",
    "Number.NEGATIVE_INFINITY",
];

const FILLS: [&str; 18] = [
    "\"0\"",
    "\"ab\"",
    "\"\"",
    "\"123456\"",
    "\"é\"",
    "\"😀é\"",
    "\" \"",
    "0",
    "1.5",
    "null",
    "undefined",
    "true",
    "false",
    "[1, 2]",
    "{}",
    "x -> x",
    "/a/",
    "o.missing",
];

const HUGE_LENGTHS: [&str; 5] = [
    "Number.POSITIVE_INFINITY",
    "1 / 0",
    "\"Infinity\"",
    "1e300",
    "1e15",
];

const OTHERS: [(&str, &str); 33] = [
    ("", "[\"ab\", \"Cd\"]"),
    ("xs = [\"ab\"]", "xs"),
    ("", "[]"),
    ("xs = [\"a\"]\nxs.push(\"b\")", "xs@first"),
    ("class Item(v):\n    this.v = v\nItem(\"a\")", "Item"),
    (
        "class Item(v):\n    this.v = v\nItem(\"a\")\nkind = Item",
        "kind",
    ),
    ("class Item(v):\n    this.v = v\nItem(\"a\")", "$Item"),
    ("class Empty", "Empty"),
    ("", "5"),
    ("", "0"),
    ("", "true"),
    ("f = x -> x", "f"),
    ("", "Date(0)"),
    ("", "/a/"),
    ("o = {a: 1}", "o"),
    ("class P(v):\n    this.v = v\np = P(\"a\")", "p"),
    ("u = null", "u"),
    ("o = {}", "o.missing"),
    ("", "Math"),
    ("", "Number"),
    ("", "String"),
    ("", "Date"),
    ("", "Object"),
    ("", "List"),
    ("", "Class"),
    ("", "JSON"),
    ("", "RegExp"),
    ("", "Function"),
    ("", "Boolean"),
    ("class String(v):\n    this.v = v\nString(1)", "String"),
    ("class Math(v):\n    this.v = v\nMath(1)", "Math"),
    ("def String() => 1", "String"),
    ("", "Ghost"),
];

const OTHER_ARGUMENTS: [&str; 5] = ["", "3", "3, \"0\"", "log.push(1)", "missing"];

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

fn names_alias(token: &Token) -> bool {
    matches!(token, Token::Identifier(name) if ALIASES.contains(&name.as_str()))
}

fn calls_member(tokens: &[Token]) -> bool {
    tokens.windows(3).any(|window| {
        window[0] == Token::Dot && names_alias(&window[1]) && window[2] == Token::LeftParen
    })
}

fn declares_member(tokens: &[Token]) -> bool {
    let defined = tokens.windows(2).any(|window| {
        (matches!(window[0], Token::Keyword(Keyword::Def | Keyword::Function))
            && names_alias(&window[1]))
            || (names_alias(&window[0]) && window[1] == Token::Colon)
    });
    let assigned = tokens.windows(3).any(|window| {
        window[0] == Token::Dot && names_alias(&window[1]) && window[2] == Token::Assign
    });
    defined || assigned
}

fn calls_alias(source: &str) -> bool {
    let tokens = tokens(source);
    calls_member(&tokens) && !declares_member(&tokens)
}

fn spelled_out(source: &str) -> String {
    let mut spelled = source.to_string();
    for (alias, spelling) in PAIRS {
        spelled = spelled.replace(&format!(".{alias}("), &format!(".{spelling}("));
    }
    spelled
}

fn touches(first: &Spanned, second: &Spanned) -> bool {
    second.position.line == first.position.line
        && second.position.column == first.position.column + 1
}

fn writes_range(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(3).any(|window| {
            window[0].token == Token::Dot
                && window[1].token == Token::Dot
                && matches!(window[2].token, Token::Less | Token::Assign)
                && touches(&window[0], &window[1])
                && touches(&window[1], &window[2])
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
            title: "padding alias regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
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
    let mut alias = Runtime::new();
    let mut spelled = Runtime::new();
    for (step, spelling) in steps {
        assert_eq!(
            outcome(&mut alias, step),
            outcome(&mut spelled, spelling),
            "{step}"
        );
        for name in names {
            let key = NodeKey::variable(*name);
            let declared = alias.graph.contains(&key);
            assert_eq!(declared, spelled.graph.contains(&key), "{step}: {name}");
            if declared {
                assert_eq!(
                    dependencies(&alias, name),
                    dependencies(&spelled, name),
                    "{step}: {name}"
                );
            }
        }
    }
    assert_eq!(alias.assertions_run(), spelled.assertions_run());
    assert!(alias.take_assertions().is_empty(), "{steps:?}");
    assert!(spelled.take_assertions().is_empty(), "{steps:?}");
}

#[test]
fn only_the_appended_cases_call_the_aliases_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(calls_alias(&case.source), new, "{}", case.title);
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
    let appended: Vec<Token> = documents
        .iter()
        .flat_map(|(_, cases)| cases)
        .filter(|case| is_appended(&case.title))
        .flat_map(|case| tokens(&case.source))
        .collect();
    for alias in ALIASES {
        let named = Token::Identifier(alias.to_string());
        assert!(
            appended
                .windows(2)
                .any(|pair| pair[0] == Token::Dot && pair[1] == named),
            "the appended cases call {alias}"
        );
    }
    assert_eq!(assertions, 27);
    assert_eq!(cases, 1348);
}

#[test]
fn every_appended_case_gives_what_its_accepted_spelling_gives() {
    let documents = documents();
    let mut compared = 0;
    for case in documents
        .iter()
        .flat_map(|(_, cases)| cases)
        .filter(|case| is_appended(&case.title))
    {
        let spelled = spelled_out(&case.source);
        assert!(!calls_alias(&spelled), "{}", case.title);
        assert_ne!(spelled, case.source, "{}", case.title);
        let assertions = common::assertions::protected_assertions(&parse(&case.source).unwrap());
        let spelled_assertions =
            common::assertions::protected_assertions(&parse(&spelled).unwrap());
        assert_eq!(assertions.len(), spelled_assertions.len(), "{}", case.title);
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
}

#[test]
fn the_audit_finds_every_alias_call_and_no_declared_member() {
    for source in [
        "s.rjust(3)",
        "s.ljust(3, \"0\")",
        "\"7\".PadLeft(3, \"0\")",
        "f(x).PadRight(4)",
        "$Room.label = String($Room.number).rjust(3, \"0\")",
        "names.map(n -> n.ljust(4, \".\"))",
        "`${s.PadLeft(3)} shown`",
        "`${count(1)} and ${f({a: t.PadRight(2)})}`",
        "x = cond ? s.rjust(2) : null",
    ] {
        assert!(calls_alias(source), "{source}");
    }
    for source in [
        "s.padStart(3)",
        "s.padEnd(3, \"0\")",
        "s.rjust",
        "rjust(s, 3)",
        "ljust = s.padEnd(3)",
        "\"rjust\"",
        "`${rjust}`",
        "# s.ljust(3)\nx = 1",
        "s.Rjust(3)",
        "s.padLeft(3)",
        "s.PadStart(3)",
        "s.center(5)",
        "s.zfill(3)",
        "class Cell:\n    def rjust(n) => n\nCell().rjust(2)",
        "o = {ljust: n -> n}\no.ljust(1)",
        "o = {}\no.PadLeft = x -> x\no.PadLeft(2)",
        "function PadRight(x) { return x }\ns.PadRight(2)",
    ] {
        assert!(!calls_alias(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_together_after_nuc_148_and_before_nuc_139() {
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
        let ranges = cases
            .iter()
            .rposition(|case| writes_range(&case.source))
            .expect("the cases of NUC 148 stand in every document");
        let xor = cases
            .iter()
            .position(|case| writes_xor_stage(&case.source))
            .expect("the cases of NUC 139 stand in every document");
        let bang_in = cases
            .iter()
            .position(|case| writes_bang_in(&case.source))
            .expect("the cases of NUC 135 stand in every document");
        assert!(ranges < first, "{}", own[0]);
        assert!(first + own.len() <= xor, "{}", own[0]);
        assert!(first + own.len() <= bang_in, "{}", own[0]);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Padding method aliases\n\n```\n")
        .expect("the language reference documents the padding aliases");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(case.expected.as_deref(), Some("11"));
    for line in [
        "[NUC 157](nuc-0157.md)",
        "`padStart` and `padEnd` ([NUC 111](nuc-0111.md)), their padding aliases `rjust` and `PadLeft` of `padStart` and `ljust` and `PadRight` of `padEnd` ([NUC 157](nuc-0157.md))",
        "| Padded string output of `padStart` or `padEnd` is too large to represent or allocate | `Padded string is too large` |",
        "| Padded string output of the aliases `rjust`, `ljust`, `PadLeft` or `PadRight` is too large to represent or allocate | `Padded string is too large` |",
        "text.rjust(n, fill)  text.ljust(n)     Python and Ruby spellings: exactly text.padStart(n, fill), text.padEnd(n) (NUC 157)",
        "text.PadLeft(n, fill)  text.PadRight(n) C# spellings: exactly text.padStart(n, fill), text.padEnd(n) (NUC 157)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    let specification = include_str!("../nucleoid.spec.md").replace("\r\n", "\n");
    let (_, contract) = specification
        .split_once("\n## Padding method aliases\n")
        .expect("the specification states the padding aliases");
    let (contract, _) = contract.split_once("\n## ").unwrap();
    let [first, .., last] = SPEC_TITLES;
    for title in [first, last] {
        assert!(contract.contains(&format!("`{title}`")), "{title}");
    }
    assert!(contract.contains("`tests/padding_aliases.rs`"));
}

#[test]
fn each_alias_gives_its_accepted_spellings_result_on_every_string() {
    let mut changed = 0;
    let mut compared = 0;
    for (setup, receiver) in TEXTS {
        for (alias, spelled) in PAIRS {
            for arguments in ARGUMENTS {
                let written = format!("{SETUP}\n{setup}\n[({receiver}).{alias}({arguments}), log]");
                let found = fresh(&written);
                assert!(found.starts_with("[\""), "{written}: {found}");
                let spelling =
                    format!("{SETUP}\n{setup}\n[({receiver}).{spelled}({arguments}), log]");
                assert_eq!(found, fresh(&spelling), "{written}");
                let shown = format!("{SETUP}\n{setup}\ntypeof ({receiver}).{alias}({arguments})");
                assert_eq!(fresh(&shown), "\"string\"", "{shown}");
                compared += 1;
            }
            let plain = fresh(&format!("{setup}\n{receiver}"));
            if fresh(&format!("{setup}\n({receiver}).{alias}(12, \"*\")")) != plain {
                changed += 1;
            }
        }
    }
    assert_eq!(compared, TEXTS.len() * PAIRS.len() * ARGUMENTS.len());
    assert_eq!(changed, TEXTS.len() * PAIRS.len());
    for (source, expected) in [
        ("\"7\".rjust(3, \"0\")", "\"007\""),
        ("\"7\".PadLeft(3, \"0\")", "\"007\""),
        ("\"ab\".ljust(4, \".\")", "\"ab..\""),
        ("\"ab\".PadRight(4, \".\")", "\"ab..\""),
        (
            "[\"ab\".rjust(5), \"ab\".ljust(5) + \"|\"]",
            "[\"   ab\",\"ab   |\"]",
        ),
        (
            "[\"ab\".rjust(7, \"xy\"), \"ab\".ljust(7, \"xy\"), \"5\".PadLeft(4, \"ab\"), \"5\".PadRight(4, \"ab\")]",
            "[\"xyxyxab\",\"abxyxyx\",\"aba5\",\"5aba\"]",
        ),
        (
            "[\"5\".rjust(4, \"123456\"), \"abc\".ljust(6, \"123456\")]",
            "[\"1235\",\"abc123\"]",
        ),
        (
            "[\"long\".rjust(2, \"0\"), \"long\".PadRight(4, \"0\"), \"\".ljust(3, \"ab\")]",
            "[\"long\",\"long\",\"aba\"]",
        ),
        (
            "[\"😀\".PadLeft(3), \"Zoë\".PadRight(5, \"·\"), \"日\".rjust(3, \"本\")]",
            "[\"  😀\",\"Zoë··\",\"本本日\"]",
        ),
        (
            "e = String.fromCharCode(101, 769)\n[e.length, e.rjust(3, \"-\").length, e.ljust(3, \"-\").slice(-1)]",
            "[2.0,3.0,\"-\"]",
        ),
        (
            "[\"x\".PadLeft(4, \"😀é\"), \"x\".ljust(4, \"😀é\")]",
            "[\"😀é😀x\",\"x😀é😀\"]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn lengths_and_fills_convert_as_for_the_spelling() {
    let mut padded = 0;
    for (alias, spelled) in PAIRS {
        for length in LENGTHS {
            let fills = FILLS.iter().map(|fill| format!(", {fill}"));
            for fill in fills.chain([String::new()]) {
                let written = format!("o = {{}}\n\"ab\".{alias}({length}{fill})");
                let found = fresh(&written);
                assert_eq!(
                    found,
                    fresh(&format!("o = {{}}\n\"ab\".{spelled}({length}{fill})")),
                    "{written}"
                );
                if found != "\"ab\"" {
                    padded += 1;
                }
            }
        }
    }
    assert_eq!(padded, 360);
    for (source, expected) in [
        (
            "[\"x\".ljust(\"4\", \"-\"), \"x\".ljust(2.9, \"-\"), \"x\".ljust(-3, \"-\"), \"x\".ljust(null, \"-\"), \"x\".ljust(0 / 0, \"-\")]",
            "[\"x---\",\"x-\",\"x\",\"x\",\"x\"]",
        ),
        (
            "[\"5\".rjust(3, 0), \"5\".rjust(3, null), \"5\".rjust(3, true), \"5\".rjust(3, \"\"), \"5\".rjust(3), \"5\".rjust(3, undefined)]",
            "[\"005\",\"nu5\",\"tr5\",\"5\",\"  5\",\"nu5\"]",
        ),
        (
            "o = {}\n[\"5\".PadLeft(4, o.missing), \"5\".PadRight(4, o.missing)]",
            "[\"   5\",\"5   \"]",
        ),
        (
            "[\"ab\".PadLeft(true, \"-\"), \"ab\".PadRight([5], \"-\"), \"ab\".ljust([1, 2], \"-\")]",
            "[\"ab\",\"ab---\",\"ab\"]",
        ),
        (
            "[\"5\".PadRight(4, [1, 2]), \"5\".rjust(4, {}), \"5\".ljust(4, x -> x)]",
            "[\"51,2\",\"[ob5\",\"5[fu\"]",
        ),
        ("[\"ab\".rjust(), \"ab\".PadRight()]", "[\"ab\",\"ab\"]"),
        ("d = Date(5)\n\"7\".PadLeft(d, \"0\")", "\"00007\""),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn oversized_results_raise_padded_string_is_too_large() {
    for (alias, spelled) in PAIRS {
        for length in HUGE_LENGTHS {
            let written = format!("\"ab\".{alias}({length}, \"-\")");
            assert_eq!(
                failure(&written),
                "TypeError: Padded string is too large (line 1, column 1)",
                "{written}"
            );
            assert_eq!(
                failure(&written),
                failure(&format!("\"ab\".{spelled}({length}, \"-\")")),
                "{written}"
            );
            assert_eq!(
                result(&format!("\"ab\".{alias}({length}, \"\")")),
                "\"ab\"",
                "{written}"
            );
        }
        let error = Runtime::new()
            .run(&format!("\"x\".{alias}(1 / 0)"))
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), "Padded string is too large");
        assert_eq!(
            result(&format!("\"x\".{alias}(1000000).length")),
            "1000000.0"
        );
    }
}

#[test]
fn arguments_are_evaluated_and_their_errors_propagate_as_for_the_spelling() {
    let mut raised = 0;
    for (setup, receiver) in TEXTS {
        for (alias, spelled) in PAIRS {
            for arguments in ["missing", "3, missing", "log.push(1), o.missing.deeper"] {
                let written =
                    format!("{SETUP}\no = {{}}\n{setup}\n({receiver}).{alias}({arguments})");
                let spelling =
                    format!("{SETUP}\no = {{}}\n{setup}\n({receiver}).{spelled}({arguments})");
                let found = fresh(&written);
                assert_eq!(found, fresh(&spelling), "{written}");
                raised += usize::from(found.starts_with("ReferenceError: missing is not defined"));
            }
        }
    }
    assert_eq!(raised, TEXTS.len() * PAIRS.len() * 2);
}

#[test]
fn other_receivers_raise_or_read_exactly_as_an_unknown_method_does() {
    let mut kept = 0;
    for (setup, receiver) in OTHERS {
        for alias in ALIASES {
            for arguments in OTHER_ARGUMENTS {
                let written = format!("{SETUP}\n{setup}\n[({receiver}).{alias}({arguments}), log]");
                let unknown = format!("{SETUP}\n{setup}\n[({receiver}).nope({arguments}), log]");
                assert_eq!(
                    fresh(&written),
                    fresh(&unknown).replace("nope", alias),
                    "{written}"
                );
                kept += 1;
            }
        }
    }
    assert_eq!(kept, OTHERS.len() * ALIASES.len() * OTHER_ARGUMENTS.len());
}

#[test]
fn the_kept_readings_keep_their_exact_accepted_results() {
    for (source, expected) in [
        ("s = \"ab\"\nv = s.ljust\nv", "null"),
        (
            "s = \"ab\"\n[s.rjust, s.ljust, s.PadLeft, s.PadRight]",
            "[null,null,null,null]",
        ),
        (
            "o = {ljust: 1, PadLeft: 2, rjust: 3, PadRight: 4}\n[o.ljust, o.PadLeft, o.rjust, o.PadRight]",
            "[1.0,2.0,3.0,4.0]",
        ),
        ("o = {rjust: n -> n * 2}\no.rjust(4)", "8.0"),
        (
            "o = {PadLeft: (n, f) -> \"own\", PadRight: (n, f) -> \"own end\"}\n[o.PadLeft(3, \"0\"), o.PadRight(3)]",
            "[\"own\",\"own end\"]",
        ),
        (
            "o = {a: 1}\n[o.rjust(3), o.ljust(3, \"x\"), o.PadLeft(), o.PadRight(2)]",
            "[null,null,null,null]",
        ),
        (
            "class Cell(v):\n    this.v = v\n    def ljust(n) => n + 1\nc = Cell(1)\nc.ljust(2)",
            "3.0",
        ),
        (
            "class Cell(v):\n    this.v = v\nc = Cell(1)\nc.rjust = 5\nc.rjust",
            "5.0",
        ),
        (
            "class P(v):\n    this.v = v\np = P(\"a\")\n[p.rjust(3), p.PadRight(3)]",
            "[null,null]",
        ),
        ("def ljust(s, n) => s + n\nljust(\"a\", 1)", "\"a1\""),
        ("ljust = 3\nljust + 1", "4.0"),
        (
            "def rjust(s, n) => \"fn\"\ndef PadLeft(s) => 2\n[rjust(\"a\", 1), PadLeft(\"b\"), \"7\".rjust(3, \"0\")]",
            "[\"fn\",2.0,\"007\"]",
        ),
        ("n = null\ny = n.PadLeft(3)\ny", "null"),
        (
            "u = null\n[u.rjust(missing), u.ljust(missing), u.PadLeft(), u.PadRight(1, 2)]",
            "[null,null,null,null]",
        ),
        ("o = {}\no.label.rjust(missing)", "null"),
        (
            "calls = []\nn = null\nn.rjust(calls.push(1))\nn.PadRight(calls.push(2))\ncalls",
            "[]",
        ),
        (
            "class Tag(name):\n    this.name = name\n$Tag.r = $Tag.rjust(3)\nt = Tag(\"x\")\nt.r",
            "null",
        ),
        (
            "class Tag(name):\n    this.name = name\n    def ljust(n) => \"own\"\n$Tag.r = $Tag.ljust(3)\nt = Tag(\"x\")\nt.r",
            "\"own\"",
        ),
        (
            "s = \"5\"\n[s.padStart(3, \"0\"), s.padEnd(3, \".\"), s.padStart(3, \"0\", 1)]",
            "[\"005\",\"5..\",\"005\"]",
        ),
        (
            "class Item(v):\n    this.v = v\nItem(\"a\")\n[Item.rjust, Item.PadRight]",
            "[null,null]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "x = 7\ny = x.rjust(3)\ny",
            "TypeError: number.rjust is not a function (line 2, column 1)",
        ),
        (
            "xs = [\"a\"]\ny = xs.ljust(3)\ny",
            "TypeError: xs.ljust is not a function (line 2, column 1)",
        ),
        (
            "y = true.ljust(3)\ny",
            "TypeError: boolean.ljust is not a function (line 1, column 1)",
        ),
        (
            "class Box(v):\n    this.v = v\nBox(1)\ny = Box.rjust(3)\ny",
            "TypeError: Box.rjust is not a function (line 4, column 1)",
        ),
        (
            "(5).PadLeft(3, \"0\")",
            "TypeError: number.PadLeft is not a function (line 1, column 1)",
        ),
        (
            "[\"a\", \"b\"].PadRight(3)",
            "TypeError: [\"a\",\"b\"].PadRight is not a function (line 1, column 1)",
        ),
        (
            "f = x -> x\nf.ljust(3)",
            "TypeError: function.ljust is not a function (line 2, column 1)",
        ),
        (
            "d = Date(5)\nd.rjust(3)",
            "TypeError: Date.rjust is not a function (line 2, column 1)",
        ),
        (
            "/a/.PadLeft(3)",
            "TypeError: PadLeft is not a function (line 1, column 1)",
        ),
        (
            "o = {rjust: 3}\no.rjust()",
            "TypeError: obj:0.rjust is not a function (line 2, column 1)",
        ),
        (
            "y = String.rjust(\"7\", 3)\ny",
            "TypeError: String.rjust is not a function (line 1, column 1)",
        ),
        (
            "String.PadRight(\"7\", 3)",
            "TypeError: String.PadRight is not a function (line 1, column 1)",
        ),
        (
            "class String(v):\n    this.v = v\nString(1)\nString.ljust(3)",
            "TypeError: String.ljust is not a function (line 4, column 1)",
        ),
        (
            "Math.PadLeft(3)",
            "TypeError: Math.PadLeft is not a function (line 1, column 1)",
        ),
        (
            "String.rjust(missing)",
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
        (
            "s = \"5\"\ns.ljust = 1",
            "TypeError: Cannot assign 'ljust' on string (line 2, column 1)",
        ),
        (
            "s = \"5\"\ns.PadLeft = 1",
            "TypeError: Cannot assign 'PadLeft' on string (line 2, column 1)",
        ),
        (
            "rjust(\"5\")",
            "ReferenceError: rjust is not defined (line 1, column 1)",
        ),
        (
            "PadRight(\"5\", 3)",
            "ReferenceError: PadRight is not defined (line 1, column 1)",
        ),
        (
            "\"5\".rjust(missing, \"0\")",
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
        (
            "y = \"7\".padLeft(3)\ny",
            "TypeError: 7.padLeft is not a function (line 1, column 1)",
        ),
        (
            "y = \"7\".PadStart(3)\ny",
            "TypeError: 7.PadStart is not a function (line 1, column 1)",
        ),
        (
            "y = \"7\".padRight(3)\ny",
            "TypeError: 7.padRight is not a function (line 1, column 1)",
        ),
        (
            "y = \"7\".pad(3)\ny",
            "TypeError: 7.pad is not a function (line 1, column 1)",
        ),
        (
            "y = \"7\".center(5)\ny",
            "TypeError: 7.center is not a function (line 1, column 1)",
        ),
        (
            "y = \"7\".zfill(3)\ny",
            "TypeError: 7.zfill is not a function (line 1, column 1)",
        ),
        (
            "y = \"ab\".LJust(3)\ny",
            "TypeError: ab.LJust is not a function (line 1, column 1)",
        ),
        (
            "\"ab\".Rjust(3)",
            "TypeError: ab.Rjust is not a function (line 1, column 1)",
        ),
        (
            "\"ab\".padleft(3)",
            "TypeError: ab.padleft is not a function (line 1, column 1)",
        ),
        (
            "\"ab\".PADLEFT(3)",
            "TypeError: ab.PADLEFT is not a function (line 1, column 1)",
        ),
        (
            "\"ab\".pad_left(3)",
            "TypeError: ab.pad_left is not a function (line 1, column 1)",
        ),
        (
            "\"ab\".padding(3)",
            "TypeError: ab.padding is not a function (line 1, column 1)",
        ),
    ] {
        assert_eq!(failure(source), expected, "{source}");
    }
}

#[test]
fn the_public_string_method_takes_the_four_aliases() {
    let method = nucleoid::builtins::string_method;
    for (alias, spelled) in PAIRS {
        for arguments in [
            Vec::new(),
            vec![Value::number(5)],
            vec![Value::number(5), Value::string("0")],
            vec![Value::number(6), Value::string("ab")],
            vec![Value::string("4"), Value::number(0), Value::Bool(true)],
            vec![Value::number(4), Value::Null],
            vec![Value::number(4), Value::Undefined],
            vec![Value::number(4), Value::string("")],
            vec![Value::number(-1), Value::string("x")],
            vec![Value::number(5), Value::string("😀é")],
        ] {
            for receiver in ["7", "", "Zoë", "😀", "long text"] {
                assert_eq!(
                    method(receiver, alias, &arguments).unwrap(),
                    method(receiver, spelled, &arguments).unwrap(),
                    "{receiver}.{alias}({arguments:?})"
                );
            }
        }
        let error = method("x", alias, &[Value::number(f64::INFINITY)]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), "Padded string is too large");
    }
    for (receiver, name, arguments, expected) in [
        (
            "7",
            "rjust",
            vec![Value::number(3), Value::string("0")],
            "007",
        ),
        (
            "7",
            "PadLeft",
            vec![Value::number(3), Value::string("0")],
            "007",
        ),
        (
            "ab",
            "ljust",
            vec![Value::number(4), Value::string(".")],
            "ab..",
        ),
        (
            "ab",
            "PadRight",
            vec![Value::number(4), Value::string(".")],
            "ab..",
        ),
        (
            "5",
            "ljust",
            vec![Value::number(4), Value::string("ab")],
            "5aba",
        ),
        ("😀", "PadLeft", vec![Value::number(3)], "  😀"),
        ("5", "rjust", vec![Value::number(3), Value::Null], "nu5"),
        ("5", "PadRight", Vec::new(), "5"),
    ] {
        assert_eq!(
            method(receiver, name, &arguments).unwrap(),
            Value::string(expected),
            "{receiver}.{name}"
        );
    }
    for name in [
        "padLeft", "padRight", "pad", "PadStart", "center", "zfill", "LJust", "Rjust", "padleft",
        "PADLEFT",
    ] {
        let error = method("7", name, &[Value::number(3)]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), format!("7.{name} is not a function"));
    }
}

#[test]
fn declarations_follow_their_receiver_length_and_fill_as_their_spellings_do() {
    for (alias, spelled) in PAIRS {
        let steps: Vec<(String, String)> = [
            "label = \"7\"",
            "width = 3",
            "fill = \"0\"",
            "shown = label.NAME(width, fill)",
            "plain = label.NAME(width)",
            "[shown, plain]",
            "label = \"42\"",
            "[shown, plain]",
            "width = 5",
            "[shown, plain]",
            "fill = \"\"",
            "[shown, plain]",
            "fill = \"ab\"",
            "label = \"日本\"",
            "[shown, plain]",
            "o = {code: \"9\"}",
            "boxed = o.code.NAME(3, \"0\")",
            "o.code = \"12\"",
            "boxed",
            "ids = [\"7\"]",
            "codes = ids.map(id -> id.NAME(3, \"0\"))",
            "ids.push(\"42\")",
            "codes",
            "ids.pop()",
            "ids = [\"1\", \"22\", \"333\"]",
            "codes",
            "stem = \"ab\"",
            "tail = `${stem}C`.NAME(5, \".\")",
            "stem = \"x\"",
            "tail",
        ]
        .iter()
        .map(|step| (step.replace("NAME", alias), step.replace("NAME", spelled)))
        .collect();
        let steps: Vec<(&str, &str)> = steps
            .iter()
            .map(|(written, spelling)| (written.as_str(), spelling.as_str()))
            .collect();
        behaves_as_spelled(&steps, &["shown", "plain", "boxed", "codes", "tail"]);
    }
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "label = \"7\"\nwidth = 3\nfill = \"0\"\nshown = label.rjust(width, fill)\nassert(shown, \"007\")\nlabel = \"42\"\nwidth = 4\nfill = \"-\"\nassert(shown, \"--42\")",
    );
    assert_eq!(dependencies(&runtime, "shown"), ["label", "width", "fill"]);
}

#[test]
fn class_level_rules_pad_each_instance_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            (
                "class Room(floor, number):\n    this.floor = floor\n    this.number = number",
                "class Room(floor, number):\n    this.floor = floor\n    this.number = number",
            ),
            (
                "$Room.label = String($Room.floor).rjust(2, \"0\") + \"-\" + String($Room.number).PadLeft(3, \"0\")",
                "$Room.label = String($Room.floor).padStart(2, \"0\") + \"-\" + String($Room.number).padStart(3, \"0\")",
            ),
            (
                "$Room.sign = $Room.label.ljust(8, \".\") + $Room.label.PadRight(7, \"*\")",
                "$Room.sign = $Room.label.padEnd(8, \".\") + $Room.label.padEnd(7, \"*\")",
            ),
            ("suite = Room(3, 7)", "suite = Room(3, 7)"),
            ("hall = Room(12, 45)", "hall = Room(12, 45)"),
            (
                "[suite.label, hall.label, suite.sign, hall.sign]",
                "[suite.label, hall.label, suite.sign, hall.sign]",
            ),
            ("suite.number = 120", "suite.number = 120"),
            ("hall.floor = 4", "hall.floor = 4"),
            (
                "[suite.label, hall.label, suite.sign, hall.sign]",
                "[suite.label, hall.label, suite.sign, hall.sign]",
            ),
            (
                "signs = Room.map(room -> room.label.ljust(8, \".\"))",
                "signs = Room.map(room -> room.label.padEnd(8, \".\"))",
            ),
            ("Room(1, 1)", "Room(1, 1)"),
            ("signs", "signs"),
        ],
        &["signs"],
    );
    behaves_as_spelled(
        &[
            (
                "class User(code):\n    this.code = code",
                "class User(code):\n    this.code = code",
            ),
            (
                "if $User.code.rjust(3, \"0\") == \"007\":\n    $User.lucky = true\nelse:\n    $User.lucky = false",
                "if $User.code.padStart(3, \"0\") == \"007\":\n    $User.lucky = true\nelse:\n    $User.lucky = false",
            ),
            ("a = User(\"7\")", "a = User(\"7\")"),
            ("b = User(\"8\")", "b = User(\"8\")"),
            ("[a.lucky, b.lucky]", "[a.lucky, b.lucky]"),
            ("a.code = \"9\"", "a.code = \"9\""),
            ("b.code = \"07\"", "b.code = \"07\""),
            ("[a.lucky, b.lucky]", "[a.lucky, b.lucky]"),
        ],
        &[],
    );
    assert_eq!(
        result(
            "class Ticket(id):\n    this.id = id\n$Ticket.code = \"T-\" + String($Ticket.id).PadLeft(4, \"0\")\nt = Ticket(7)\nu = Ticket(42)\nt.id = 12345\n[t.code, u.code]"
        ),
        "[\"T-12345\",\"T-0042\"]"
    );
}

#[test]
fn a_held_receiver_replaced_by_another_kind_reads_the_alias_name_on_it() {
    for (alias, spelled) in PAIRS {
        let padded = if spelled == "padStart" {
            "\"..ab\""
        } else {
            "\"ab..\""
        };
        let repadded = if spelled == "padStart" {
            "\"..cd\""
        } else {
            "\"cd..\""
        };
        let raised = |kind: &str, name: &str| {
            Some(format!(
                "TypeError: {kind}.{name} is not a function (line 1, column 1)"
            ))
        };
        let member = format!("s = {{{alias}: (n, f) -> \"own\"}}");
        let boxed = format!("class Box:\n    def {alias}(n, f) => \"boxed\"\ns = Box()");
        let read = |name: &str| vec!["s".to_string(), format!("obj:0.{name}")];
        let kept = vec!["s".to_string()];
        for (replacement, written_error, spelled_error, values, written_from, spelled_from) in [
            (
                "s = [\"x\"]".to_string(),
                raised("s", alias),
                raised("s", spelled),
                [padded, padded],
                kept.clone(),
                kept.clone(),
            ),
            (
                "s = 7".to_string(),
                raised("number", alias),
                raised("number", spelled),
                [padded, padded],
                kept.clone(),
                kept.clone(),
            ),
            (
                "s = false".to_string(),
                raised("boolean", alias),
                raised("boolean", spelled),
                [padded, padded],
                kept.clone(),
                kept.clone(),
            ),
            (
                "s = Date(0)".to_string(),
                raised("Date", alias),
                raised("Date", spelled),
                [padded, padded],
                kept.clone(),
                kept.clone(),
            ),
            (
                member,
                None,
                None,
                ["\"own\"", "null"],
                read(alias),
                read(spelled),
            ),
            (
                boxed,
                None,
                None,
                ["\"boxed\"", "null"],
                kept.clone(),
                vec!["s".to_string(), format!("s.{spelled}")],
            ),
            (
                "s = {other: 1}".to_string(),
                None,
                None,
                ["null", "null"],
                read(alias),
                read(spelled),
            ),
            (
                "s = null".to_string(),
                None,
                None,
                ["null", "null"],
                kept.clone(),
                kept.clone(),
            ),
            (
                "s = \"cd\"".to_string(),
                None,
                None,
                [repadded, repadded],
                kept.clone(),
                kept.clone(),
            ),
        ] {
            let mut written = Runtime::new();
            let mut spelling = Runtime::new();
            written
                .run(&format!("s = \"ab\"\nu = s.{alias}(4, \".\")"))
                .unwrap();
            spelling
                .run(&format!("s = \"ab\"\nu = s.{spelled}(4, \".\")"))
                .unwrap();
            for (runtime, error, value, from) in [
                (&mut written, written_error, values[0], written_from),
                (&mut spelling, spelled_error, values[1], spelled_from),
            ] {
                match (runtime.run(&replacement), error) {
                    (Err(found), Some(expected)) => {
                        assert_eq!(renumbered(&found.to_string()), expected, "{replacement}");
                        assert_eq!(json(runtime, "s"), "\"ab\"", "{replacement}");
                    }
                    (Ok(_), None) => {}
                    (found, expected) => {
                        panic!("{replacement}: {found:?} where {expected:?} was expected")
                    }
                }
                assert_eq!(json(runtime, "u"), value, "{replacement}");
                let names: Vec<String> = from.iter().map(|name| format!("\"{name}\"")).collect();
                assert_eq!(
                    json(runtime, "(why u)[0].from"),
                    format!("[{}]", names.join(",")),
                    "{replacement}"
                );
            }
        }
    }
}

#[test]
fn explanations_name_the_dependencies_of_the_accepted_spelling() {
    let setup = "class Tag(code):\n    this.code = code\nt = Tag(\"9\")\nlabel = \"7\"\nwidth = 3\nfill = \"0\"\nw = 3\n";
    let declarations = "shown = label.rjust(width, fill)\nplain = label.PadLeft(w, \"0\")\ntail = label.ljust(4)\nbox = t.code.PadRight(width, fill)\nmixed = `${label}!`.rjust(width)";
    let mut alias = Runtime::new();
    let mut spelled = Runtime::new();
    alias.run(&format!("{setup}{declarations}")).unwrap();
    spelled
        .run(&format!("{setup}{}", spelled_out(declarations)))
        .unwrap();
    for (name, rule, spelling) in [
        (
            "shown",
            "\"shown = label.rjust(width,fill)\"",
            "\"shown = label.padStart(width,fill)\"",
        ),
        (
            "plain",
            "\"plain = label.PadLeft(w,\\\"0\\\")\"",
            "\"plain = label.padStart(w,\\\"0\\\")\"",
        ),
        (
            "tail",
            "\"tail = label.ljust(4)\"",
            "\"tail = label.padEnd(4)\"",
        ),
        (
            "box",
            "\"box = t.code.PadRight(width,fill)\"",
            "\"box = t.code.padEnd(width,fill)\"",
        ),
    ] {
        let query = format!("(why {name})[0].rule");
        assert_eq!(json(&mut alias, &query), rule);
        assert_eq!(json(&mut spelled, &query), spelling);
        assert_eq!(
            dependencies(&alias, name),
            dependencies(&spelled, name),
            "{name}"
        );
        for query in [
            format!("(why {name}).map(step -> step.from)"),
            format!("(why {name}).map(step -> step.node)"),
            format!("(why {name}).map(step -> step.holds)"),
            format!("(why {name} |> stated).map(step -> step.node)"),
        ] {
            assert_eq!(
                json(&mut alias, &query),
                json(&mut spelled, &query),
                "{query}"
            );
        }
    }
    assert_eq!(
        dependencies(&alias, "mixed"),
        dependencies(&spelled, "mixed")
    );
    assert_eq!(dependencies(&alias, "shown"), ["label", "width", "fill"]);
    assert_eq!(
        json(&mut alias, "(why shown)[0].from"),
        "[\"label\",\"width\",\"fill\"]"
    );
    for query in [
        "(affects label).map(step -> step.node)",
        "(affects width).map(step -> step.node)",
        "(affects fill).map(step -> step.node)",
        "(affects t).map(step -> step.node)",
    ] {
        assert_eq!(
            json(&mut alias, query),
            json(&mut spelled, query),
            "{query}"
        );
    }
    alias
        .run("label = \"42\"\nwidth = 5\nt.code = \"1\"")
        .unwrap();
    spelled
        .run("label = \"42\"\nwidth = 5\nt.code = \"1\"")
        .unwrap();
    assert_eq!(
        json(&mut alias, "[shown, plain, tail, box, mixed]"),
        "[\"00042\",\"042\",\"42  \",\"10000\",\"  42!\"]"
    );
    assert_eq!(
        json(&mut alias, "[shown, plain, tail, box, mixed]"),
        json(&mut spelled, "[shown, plain, tail, box, mixed]")
    );
    for (source, rendered) in [
        ("t = s.rjust(3, \"0\")", "t=s.rjust(3,\"0\")"),
        ("t = s.PadLeft(n)", "t=s.PadLeft(n)"),
        ("t = s.ljust(n, \"ab\", 2)", "t=s.ljust(n,\"ab\",2)"),
        (
            "$Room.k = $Room.name.PadRight(4, \".\")",
            "$Room.k=$Room.name.PadRight(4,\".\")",
        ),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), rendered, "{source}");
    }
}

#[test]
fn the_receiver_and_then_every_argument_evaluate_once_in_written_order() {
    let functions = "log = []\ndef text():\n    log.push(\"text\")\n    return \"7\"\ndef size():\n    log.push(\"size\")\n    return 3\ndef fill():\n    log.push(\"fill\")\n    return \"0\"\n";
    for (alias, spelled) in PAIRS {
        let padded = if spelled == "padStart" { "007" } else { "700" };
        for (call, expected) in [
            (
                format!("text().{alias}(size(), fill())"),
                format!("[\"{padded}\",[\"text\",\"size\",\"fill\"]]"),
            ),
            (
                format!("text().{alias}(size(), fill(), log.push(\"extra\"))"),
                format!("[\"{padded}\",[\"text\",\"size\",\"fill\",\"extra\"]]"),
            ),
            (
                format!("\"abc\".{alias}(size(), fill())"),
                "[\"abc\",[\"size\",\"fill\"]]".to_string(),
            ),
            (
                format!("text().{alias}(size(), \"\")"),
                "[\"7\",[\"text\",\"size\"]]".to_string(),
            ),
        ] {
            let source = format!("{functions}[{call}, log]");
            assert_eq!(result(&source), expected, "{source}");
            assert_eq!(
                result(&format!("{functions}[{}, log]", spelled_out(&call))),
                expected,
                "{source}"
            );
        }
        let mut runtime = Runtime::new();
        runtime.run(functions).unwrap();
        assert_eq!(
            runtime
                .run(&format!("text().{alias}(size(), missing, fill())"))
                .unwrap_err()
                .to_string(),
            "ReferenceError: missing is not defined (line 1, column 1)"
        );
        assert_eq!(json(&mut runtime, "log"), "[]");
    }
}

#[test]
fn null_bindings_defer_declarations_as_they_do_for_the_spelling() {
    for (alias, spelled) in PAIRS {
        for (setup, declaration, write) in [
            ("fill = null", "shown = \"5\".NAME(3, fill)", "fill = \"0\""),
            (
                "width = null",
                "shown = \"5\".NAME(width, \"0\")",
                "width = 3",
            ),
            ("s = null", "shown = s.NAME(3, \"0\")", "s = \"7\""),
        ] {
            let mut written = Runtime::new();
            let mut spelling = Runtime::new();
            written
                .run(&format!("{setup}\n{}", declaration.replace("NAME", alias)))
                .unwrap();
            spelling
                .run(&format!(
                    "{setup}\n{}",
                    declaration.replace("NAME", spelled)
                ))
                .unwrap();
            assert_eq!(json(&mut written, "shown"), "null", "{declaration}");
            assert_eq!(json(&mut spelling, "shown"), "null", "{declaration}");
            written.run(write).unwrap();
            spelling.run(write).unwrap();
            assert_eq!(
                json(&mut written, "shown"),
                json(&mut spelling, "shown"),
                "{declaration}"
            );
            assert_ne!(json(&mut written, "shown"), "null", "{declaration}");
        }
    }
}

#[test]
fn aliases_read_in_every_statement_position() {
    for (source, expected) in [
        (
            "r = 0\ntry:\n    r = (5).rjust(3)\ncatch error:\n    r = error\nr",
            "\"TypeError: number.rjust is not a function\"",
        ),
        (
            "r = 0\ntry:\n    r = [\"a\"].PadRight(3)\ncatch error:\n    r = error\nr",
            "\"TypeError: [\\\"a\\\"].PadRight is not a function\"",
        ),
        (
            "code = \"7\"\nshown = code.rjust(3, \"0\")\nseen = suppose code = \"42\": shown\n[seen, shown, code]",
            "[\"042\",\"007\",\"7\"]",
        ),
        ("n = 7\n`#${String(n).PadLeft(3, \"0\")}`", "\"#007\""),
        (
            "code = \"7\"\nif code.ljust(3, \"0\") == \"700\":\n    kind = \"padded\"\nelse:\n    kind = \"plain\"\nkind",
            "\"padded\"",
        ),
        (
            "code = \"1234\"\ncode.PadLeft(3, \"0\") == code ? \"kept\" : \"cut\"",
            "\"kept\"",
        ),
        (
            "def label(n) => \"#\" + String(n).rjust(3, \"0\")\n[label(7), label(1234)]",
            "[\"#007\",\"#1234\"]",
        ),
        (
            "def row(s):\n    return s.PadRight(5, \".\") + \"|\"\nrow(\"ab\")",
            "\"ab...|\"",
        ),
        (
            "names = [\"a\", \"bbb\", \"cc\"]\nnames.filter(n -> n.ljust(3).length == 3).map(n -> n.rjust(4, \"-\"))",
            "[\"---a\",\"-bbb\",\"--cc\"]",
        ),
        (
            "codes = [\"7\", \"42\"]\ncodes.some(c -> c.PadLeft(2, \"0\") == \"07\") && codes.every(c -> c.PadRight(2, \"0\").length == 2)",
            "true",
        ),
        (
            "i = 0\nwhile String(i).rjust(3, \"0\") != \"003\":\n    i = i + 1\ni",
            "3.0",
        ),
        (
            "class Tag(code):\n    this.code = code\n    def shown() => this.code.rjust(3, \"0\")\n    def tail() => self.code.ljust(3, \".\")\nt = Tag(\"7\")\n[t.shown(), t.tail()]",
            "[\"007\",\"7..\"]",
        ),
        (
            "codes = [\"7\", \"42\"]\ncodes.map(code -> code.PadLeft(3, \"0\")).join(\"|\")",
            "\"007|042\"",
        ),
        (
            "\"7\".rjust(2, \"0\").ljust(4, \".\").PadLeft(5)",
            "\" 07..\"",
        ),
        (
            "class P(v):\n    this.v = v\np = P(\"ab\")\n[p.v.rjust(3), p.v.PadRight(3, \"!\")]",
            "[\" ab\",\"ab!\"]",
        ),
        (
            "class T(id):\n    this.id = id\nif $T.id.PadLeft(2, \"0\") == \"07\":\n    $T.lucky = true\nt = T(\"7\")\nu = T(\"8\")\n[t.lucky, u.lucky]",
            "[true,null]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
        assert_eq!(
            result(&spelled_out(source)),
            spelled_names(expected),
            "{source}"
        );
    }
}

fn spelled_names(text: &str) -> String {
    let mut spelled = text.to_string();
    for (alias, spelling) in PAIRS {
        spelled = spelled.replace(&format!(".{alias} "), &format!(".{spelling} "));
    }
    spelled
}

#[test]
fn failed_statements_and_transactions_restore_alias_declarations() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "width = 3\nshown = \"7\".rjust(width, \"0\")\nfailure = \"\"\ntry:\n    width = 1 / 0\ncatch error:\n    failure = error\nassert(failure, TypeError(\"Padded string is too large\"))\nassert([width, shown], [3, \"007\"])",
    );
    checked(
        &mut runtime,
        "label = \"ab\"\nout = label.ljust(4, \".\")\ntry:\n    label = 7\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.ljust is not a function\"))\nassert([label, out], [\"ab\", \"ab..\"])",
    );
    checked(
        &mut runtime,
        "class Ticket(id):\n    this.id = id\n$Ticket.code = $Ticket.id.PadLeft(4, \"0\")\nt = Ticket(\"7\")\ntry:\n    t.id = 7\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.PadLeft is not a function\"))\nassert([t.id, t.code], [\"7\", \"0007\"])",
    );
    checked(
        &mut runtime,
        "words = [\"ab\"]\nrows = words.map(word -> word.PadRight(3, \"-\"))\ntry:\n    words.push(true)\ncatch error:\n    failure = error\nassert([failure, words, rows], [TypeError(\"boolean.PadRight is not a function\"), [\"ab\"], [\"ab-\"]])",
    );
    checked(
        &mut runtime,
        "try:\n    width = 5\n    t.id = \"12\"\n    words.push(\"c\")\n    assert([shown, t.code, rows], [\"00007\", \"0012\", [\"ab-\", \"c--\"]])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([shown, t.code, width, t.id, rows], [\"007\", \"0007\", 3, \"7\", [\"ab-\"]])",
    );
    assert_eq!(dependencies(&runtime, "shown"), ["width"]);
    assert_eq!(
        failure("width = 3\nshown = \"7\".PadLeft(width, \"0\")\nwidth = 1 / 0"),
        "TypeError: Padded string is too large (line 3, column 1)"
    );
    assert_eq!(
        failure("label = \"a\"\nout = label.rjust(3)\nlabel = [1]"),
        "TypeError: label.rjust is not a function (line 3, column 1)"
    );
    let mut recovered = Runtime::new();
    recovered
        .run("ids = []\ncodes = ids.map(id -> id.rjust(size, \"0\"))")
        .unwrap();
    assert_eq!(
        recovered.run("ids.push(\"7\")").unwrap_err().to_string(),
        "ReferenceError: size is not defined (line 1, column 1)"
    );
    assert_eq!(json(&mut recovered, "[ids, codes]"), "[[],[]]");
    recovered.run("size = 2\nids.push(\"7\")").unwrap();
    assert_eq!(json(&mut recovered, "codes"), "[\"07\"]");
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    for (alias, spelled) in PAIRS {
        let padded = if spelled == "padStart" { "005" } else { "500" };
        assert_eq!(
            result(&format!(
                "try:\n    r = \"5\".{alias}(3, \"0\")\ncatch e:\n    r = \"caught\"\nr"
            )),
            format!("\"{padded}\"")
        );
        assert_eq!(
            result(&format!(
                "r = 0\ntry:\n    r = \"x\".{alias}(1 / 0)\ncatch error:\n    r = error\nr"
            )),
            "\"TypeError: Padded string is too large\""
        );
        assert_eq!(
            result(&format!(
                "xs = [\"5\"]\ntry:\n    r = xs.{alias}(3)\ncatch e:\n    r = \"caught\"\nr"
            )),
            "\"caught\""
        );
    }
}

#[test]
fn large_pads_and_long_fills_build_the_spellings_exact_results() {
    for (alias, spelled) in PAIRS {
        let mut runtime = Runtime::new();
        runtime
            .run(&format!(
                "long = \"x\".{alias}(1000000, \"ab\")\nspelled = \"x\".{spelled}(1000000, \"ab\")\nfill = \"0123456789\".repeat(100)\nwide = \"x\".{alias}(2500, fill)"
            ))
            .unwrap();
        assert_eq!(
            json(
                &mut runtime,
                &format!(
                    "[long.length, long == spelled, wide.length, wide == \"x\".{spelled}(2500, fill)]"
                )
            ),
            "[1000000.0,true,2500.0,true]",
            "{alias}"
        );
    }
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-padding-aliases-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_the_aliases() {
    let program = "ids = [\"7\", \"42\"]\ncodes = ids.map(id -> id.rjust(3, \"0\"))\nitem = \"tea\"\nline = item.ljust(6, \".\")\nleft = item.PadLeft(5, \"*\")\nright = item.PadRight(5, \"*\")\nids.push(\"5\")\n[codes, line, left, right]";
    let spelled = spelled_out(program);
    assert!(!calls_alias(&spelled), "{spelled}");
    let file = SourceFile::new("file");
    for options in [
        &[][..],
        &["--json"][..],
        &["--graph"][..],
        &["--json", "--graph"][..],
    ] {
        let alias = file.run(program, options);
        let written = file.run(&spelled, options);
        assert!(alias.status.success(), "{options:?} {alias:?}");
        assert!(alias.stderr.is_empty(), "{options:?} {alias:?}");
        assert_eq!(alias.stdout, written.stdout, "{options:?}");
    }
    assert_eq!(
        String::from_utf8(file.run(program, &["--json"]).stdout).unwrap(),
        "[[\"007\",\"042\",\"005\"],\"tea...\",\"**tea\",\"tea**\"]\n"
    );
    let graph = String::from_utf8(
        file.run(
            "label = \"7\"\nwidth = 3\nfill = \"0\"\nshown = label.rjust(width, fill)\nshown",
            &["--graph"],
        )
        .stdout,
    )
    .unwrap();
    assert!(
        graph.contains("shown [variable]\n    reads    label, width, fill"),
        "{graph}"
    );
    for (source, message) in [
        (
            "xs = [1]\n(5).ljust(3)\n",
            "TypeError: number.ljust is not a function (line 2, column 1)\n",
        ),
        (
            "\"x\".PadLeft(1 / 0)\n",
            "TypeError: Padded string is too large (line 1, column 1)\n",
        ),
    ] {
        let failed = file.run(source, &["--json"]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(String::from_utf8(failed.stderr).unwrap(), message);
    }
    let session = "ids = [\"7\"]\ncodes = ids.map(id -> id.rjust(3, \"0\"))\ncodes\nids.push(\"42\")\ncodes\n\"ab\".ljust(4, \".\")\n\"5\".PadLeft(3, \"0\")\n\"Zoë\".PadRight(5, \"·\")\n.exit\n";
    let spelled_session = spelled_out(session);
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let alias = repl(&session.replace('\n', newline), options);
            let written = repl(&spelled_session.replace('\n', newline), options);
            assert!(alias.status.success(), "{options:?} {alias:?}");
            assert_eq!(alias.stdout, written.stdout, "{options:?}");
            assert_eq!(alias.stderr, written.stderr, "{options:?}");
            let stdout = String::from_utf8(alias.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "[\"007\"]\n2.0\n[\"007\",\"042\"]\n\"ab..\"\n\"005\"\n\"Zoë··\"\n"
            } else {
                "007\n2\n007,042\nab..\n005\nZoë··\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
        }
    }
    let failing = repl(
        "\"x\".rjust(1 / 0)\n\"x\".PadRight(3, \"-\")\n.exit\n",
        &["--plain"],
    );
    assert!(failing.status.success(), "{failing:?}");
    assert_eq!(
        String::from_utf8(failing.stderr).unwrap(),
        "TypeError: Padded string is too large (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(failing.stdout).unwrap();
    assert!(stdout.contains("x--\n"), "{stdout}");
}
