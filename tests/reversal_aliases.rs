mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime, Value};

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid reverses lists with reversed and toReversed",
    "Nucleoid reverses class populations in reverse creation order",
    "Nucleoid reverses strings by character with reversed and reverse",
    "Nucleoid rolls back failed updates through reversal aliases",
];
const REFERENCE_TITLE: &str =
    "Nucleoid reverses lists, populations and strings as Kotlin, Swift and JavaScript do";
const README_TITLE: &str = "Reversal methods keep an undo history newest first";
const EXAMPLES_TITLE: &str = "Reversal methods trace every delivery route back to its depot";

const LIST_ALIASES: [&str; 2] = ["reversed", "toReversed"];
const TEXT_ALIASES: [&str; 2] = ["reversed", "reverse"];
const NAMES: [&str; 3] = ["reverse", "reversed", "toReversed"];

const SPELLED: [(&str, &str); 25] = [
    ("history.reversed()", "history.reverse()"),
    ("history.toReversed()", "history.reverse()"),
    ("[].reversed()", "[].reverse()"),
    ("[[1, 2], 3].toReversed()", "[[1, 2], 3].reverse()"),
    ("Stop.reversed()", "Stop.reverse()"),
    ("Stop.toReversed()", "Stop.reverse()"),
    ("$Train.cars.reversed()", "$Train.cars.reverse()"),
    ("word.reversed()", "word[::-1]"),
    ("word.reverse()", "word[::-1]"),
    ("\"a😀b\".reversed()", "\"a😀b\"[::-1]"),
    ("\"\".reverse()", "\"\"[::-1]"),
    ("\"stressed\".reversed()", "\"stressed\"[::-1]"),
    (
        "$Word.text.toLowerCase().reversed()",
        "$Word.text.toLowerCase()[::-1]",
    ),
    ("entries.reversed()", "entries.reverse()"),
    ("$Tag.label.reversed()", "$Tag.label[::-1]"),
    ("readings.reversed()", "readings.reverse()"),
    ("Visit.toReversed()", "Visit.reverse()"),
    ("\"café\".reverse()", "\"café\"[::-1]"),
    ("edits.reversed()", "edits.reverse()"),
    ("edits.toReversed()", "edits.reverse()"),
    ("$Route.stops.reversed()", "$Route.stops.reverse()"),
    ("$Route.stops.toReversed()", "$Route.stops.reverse()"),
    ("\"café\".reversed()", "\"café\"[::-1]"),
    ("\"a😀b\".reverse()", "\"a😀b\"[::-1]"),
    ("\"\".reversed()", "\"\"[::-1]"),
];

const SETUP: &str = "log = []\nclass Item(v):\n    this.v = v\nItem(1)\nItem(2)\nItem(3)\nclass Empty\nclass Box(v):\n    this.v = v\n    def reversed() => \"own\"\n    def toReversed() => \"own too\"\n    def reverse() => \"mine\"\nBox(1)\nBox(2)\nclass Kind(v):\n    this.v = v\nKind(1)";

const LISTS: [(&str, &str); 33] = [
    ("", "[1, 2, 3]"),
    ("", "[]"),
    ("", "[7]"),
    ("", "[[1, 2], [3, [4]], 5]"),
    ("", "[null, \"a\", true, 2.5, {k: 1}]"),
    ("xs = [1, 2, 3]", "xs"),
    ("o = {items: [1, 2]}", "o.items"),
    ("o = {p: {q: [5, 6, 7]}}", "o.p.q"),
    (
        "class Holder(items):\n    this.items = items\nh = Holder([3, 4])",
        "h.items",
    ),
    ("def make() => [1, 2, 3, 4]", "make()"),
    ("xs = [1, 2, 3]", "xs[1:]"),
    ("xs = [1, 2, 3, 4]", "xs[-2:]"),
    ("xs = [1, 2]", "xs.value"),
    ("xs = [1]\nxs.push(2)", "xs@first"),
    ("xs = [1]\nxs.push(2)", "xs@last"),
    ("grid = [[1, 2], [3]]", "grid[0]"),
    ("xs = [1, 2, 3]", "xs.filter(x -> x > 1)"),
    ("xs = [1, 2]", "xs.map(x -> x * 2)"),
    ("xs = [1]", "xs.concat([2, 3])"),
    ("xs = [3, 1, 2]", "xs.reverse()"),
    ("", "\"a,b,c\".split(\",\")"),
    ("", "\"abc\".split(\"\")"),
    ("flag = true", "(flag ? [1, 2] : [])"),
    ("objs = [{a: 1}, {a: 2}]", "objs"),
    ("", "Item"),
    ("kind = Item", "kind"),
    ("", "$Item"),
    ("", "Empty"),
    ("", "Box"),
    ("", "Item.list()"),
    ("", "Item[1:]"),
    ("", "Item.filter(item -> item.v > 1)"),
    ("class List(v):\n    this.v = v\nList(1)\nList(2)", "List"),
];

const TEXTS: [(&str, &str); 32] = [
    ("", "\"abc\""),
    ("", "\"\""),
    ("", "\"a\""),
    ("", "\"racecar\""),
    ("", "\"café crème\""),
    ("", "\"e\u{301}x\""),
    ("", "\"a😀b\""),
    ("", "\"\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F467} ok\""),
    ("", "\"\u{1F1EB}\u{1F1F7}\""),
    ("", "\"日本語\""),
    ("", "\"שלום עולם\""),
    ("", "\"مرحبا\""),
    ("", "\"नमस्ते\""),
    ("", "\"Ab 12 !? Zoë Ω\""),
    ("s = \"hello\"", "s"),
    ("o = {name: \"Ada\"}", "o.name"),
    ("class P(v):\n    this.v = v\np = P(\"Lin\")", "p.v"),
    ("def word() => \"word\"", "word()"),
    ("stem = \"ab\"", "`${stem}c`"),
    ("s = \"ab\"", "s.value"),
    ("s = \"ab\"\ns = \"abcd\"", "s@first"),
    ("s = \"ab\"\ns = \"abcd\"", "s@last"),
    ("xs = [\"x\", \"yz\"]", "xs[1]"),
    ("", "\"ABC\".toLowerCase()"),
    ("xs = [\"a\", \"b\"]", "xs.join(\"-\")"),
    ("", "\"ab\".repeat(3)"),
    ("", "\"Hello World\".slice(0, 5)"),
    ("", "\"a,bc\".split(\",\")[1]"),
    ("flag = true", "(flag ? \"yes\" : \"no\")"),
    ("", "String(42)"),
    ("", "(typeof 5)"),
    ("", "\"abc\".reversed()"),
];

const ARGUMENTS: [&str; 7] = [
    "",
    "1",
    "1, \"two\", null, [3]",
    "log.push(\"a\"), log.push(\"b\")",
    "x -> x",
    "Kind",
    "[], Kind",
];

const OTHERS: [(&str, &str); 27] = [
    ("", "5"),
    ("", "0"),
    ("", "true"),
    ("f = x -> x", "f"),
    ("", "Date(0)"),
    ("", "/a/"),
    ("o = {a: 1}", "o"),
    ("class P(v):\n    this.v = v\np = P(1)", "p"),
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
    ("class Math(v):\n    this.v = v\nMath(1)", "Math"),
    ("class Number(v):\n    this.v = v\nNumber(1)", "Number"),
    ("class String(v):\n    this.v = v\nString(1)", "String"),
    ("class Date(v):\n    this.v = v\nDate(1)", "Date"),
    ("def String() => 1", "String"),
    ("", "Ghost"),
];

const OTHER_ARGUMENTS: [&str; 4] = ["", "\"a\"", "log.push(1)", "missing"];

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
    matches!(token, Token::Identifier(name) if LIST_ALIASES.contains(&name.as_str()))
}

fn alias_calls(tokens: &[Token]) -> usize {
    tokens
        .windows(3)
        .filter(|window| {
            window[0] == Token::Dot && names_alias(&window[1]) && window[2] == Token::LeftParen
        })
        .count()
}

fn text_reversals(tokens: &[Token]) -> usize {
    tokens
        .windows(4)
        .filter(|window| {
            matches!(window[0], Token::Str(_) | Token::Template(_))
                && window[1] == Token::Dot
                && window[2] == Token::Identifier("reverse".to_string())
                && window[3] == Token::LeftParen
        })
        .count()
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
    (alias_calls(&tokens) > 0 && !declares_member(&tokens)) || text_reversals(&tokens) > 0
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

fn checked(runtime: &mut Runtime, source: &str) -> String {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "reversal alias regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
    renumbered(&runtime.serialize_json(&value).unwrap())
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

fn spelled_out(source: &str) -> (String, usize) {
    let mut spelled = source.to_string();
    let mut rewrites = 0;
    for (alias, spelling) in SPELLED {
        rewrites += spelled.matches(alias).count();
        spelled = spelled.replace(alias, spelling);
    }
    (spelled, rewrites)
}

#[test]
fn only_the_appended_cases_call_the_aliases_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    let mut calls = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(calls_alias(&case.source), new, "{}", case.title);
        if new {
            let tokens = tokens(&case.source);
            calls += alias_calls(&tokens) + text_reversals(&tokens);
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
    assert_eq!(calls, 23);
    assert_eq!(assertions, 20);
    assert_eq!(cases, 1293);
}

#[test]
fn the_audit_finds_every_alias_call_and_no_declared_member() {
    for source in [
        "xs.reversed()",
        "xs.toReversed(1)",
        "f(x).reversed()",
        "[1, 2].toReversed()",
        "Truck.reversed().map(t -> t.plate)",
        "$Route.stops.toReversed()",
        "\"abc\".reverse()",
        "`${name}!`.reverse()",
        "`${xs.reversed()} shown`",
        "`${count(1)} and ${f({a: t.toReversed()})}`",
        "x = cond ? xs.reversed() : null",
    ] {
        assert!(calls_alias(source), "{source}");
    }
    for source in [
        "xs.reverse()",
        "Truck.reverse()",
        "xs.reversed",
        "s.toReversed",
        "reversed(xs)",
        "reversed = xs.reverse()",
        "\"reversed\"",
        "`${reversed}`",
        "# xs.reversed()\nx = 1",
        "xs.Reversed()",
        "xs.asReversed()",
        "class Shelf:\n    def reversed() => 1\nShelf().reversed()",
        "o = {toReversed: () => 1}\no.toReversed()",
        "o = {}\no.reversed = x -> x\no.reversed(2)",
        "function reversed(x) { return x }\nxs.reversed()",
    ] {
        assert!(!calls_alias(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_129_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(titles[827], "Nucleoid rolls back failed zip updates");
    assert_eq!(titles[828..832], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid zips lists and populations with zip",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Zip weighs every grade by its credits for a live grade point average",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Zip measures every climb between the checkpoints of a trail",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 8 - 2 * usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 2].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Reversal method aliases\n\n```\n")
        .expect("the language reference documents the reversal aliases");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 131](nuc-0131.md)"));
    for line in [
        "items.reversed()  items.toReversed()   exactly items.reverse(), a new list; Name.reversed() lists the latest instance first (NUC 131)",
        "text.reversed()  text.reverse()        exactly text[::-1], the characters in reverse order (NUC 131)",
        "`slice`, `reverse` (a new list in reverse order), `sort`",
        "the reversal aliases `reversed()` and `toReversed()` of `reverse()` ([NUC 131](nuc-0131.md))",
        "the reversals `reverse()` and `reversed()`, exactly `text[::-1]` ([NUC 131](nuc-0131.md))",
        "[root section](../nucleoid.spec.md#reversal-method-aliases)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    assert_eq!(
        case.expected.as_deref(),
        Some("[\"desserts\",\"éfac\",\"b😀a\"]")
    );
    let specification = include_str!("../nucleoid.spec.md").replace("\r\n", "\n");
    assert!(specification.contains("## Reversal method aliases\n\n- [NUC 131](docs/nuc-0131.md)"));
}

#[test]
fn every_case_gives_what_its_accepted_spelling_gives() {
    let documents = documents();
    let mut rewrites = 0;
    let mut compared = 0;
    for case in documents
        .iter()
        .flat_map(|(_, cases)| cases)
        .filter(|case| is_appended(&case.title))
    {
        let (spelled, count) = spelled_out(&case.source);
        assert!(!calls_alias(&spelled), "{spelled}");
        rewrites += count;
        let written = common::Case {
            title: case.title.clone(),
            source: case.source.clone(),
            expected: case.expected.clone(),
        };
        let spelling = common::Case {
            title: case.title.clone(),
            source: spelled.clone(),
            expected: case.expected.clone(),
        };
        let count = common::assertions::protected_assertions(&parse(&spelled).unwrap()).len();
        let mut alias = Runtime::new();
        let mut accepted = Runtime::new();
        let alias_value = common::check_with_count(&mut alias, &written, count)
            .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
        let accepted_value = common::check_with_count(&mut accepted, &spelling, count)
            .unwrap_or_else(|reason| panic!("{} spelled: {reason}", case.title));
        assert_eq!(
            renumbered(&alias.serialize_json(&alias_value).unwrap()),
            renumbered(&accepted.serialize_json(&accepted_value).unwrap()),
            "{}",
            case.title
        );
        assert_eq!(alias.assertions_run(), accepted.assertions_run());
        compared += 1;
    }
    assert_eq!(compared, 7);
    assert_eq!(rewrites, 24);
}

#[test]
fn list_and_population_aliases_are_reverse_on_every_receiver() {
    let mut compared = 0;
    let mut changed = 0;
    for (setup, receiver) in LISTS {
        for alias in LIST_ALIASES {
            for arguments in ARGUMENTS {
                let written = format!("{SETUP}\n{setup}\n[({receiver}).{alias}({arguments}), log]");
                let spelling =
                    format!("{SETUP}\n{setup}\n[({receiver}).reverse({arguments}), log]");
                let found = fresh(&written);
                assert!(found.starts_with("[["), "{written}: {found}");
                assert_eq!(found, fresh(&spelling), "{written}");
                compared += 1;
            }
            let plain = fresh(&format!("{SETUP}\n{setup}\n{receiver}"));
            let reversed = fresh(&format!("{SETUP}\n{setup}\n({receiver}).{alias}()"));
            if reversed != plain {
                changed += 1;
            }
        }
    }
    assert_eq!(compared, LISTS.len() * LIST_ALIASES.len() * ARGUMENTS.len());
    assert!(changed > LISTS.len(), "{changed}");
    for (source, expected) in [
        ("[1, 2, 3].reversed()", "[3.0,2.0,1.0]"),
        ("[1, 2, 3].toReversed()", "[3.0,2.0,1.0]"),
        ("[[1, 2], 3].reversed()", "[3.0,[1.0,2.0]]"),
        ("[[1, [2, 3]], 4].toReversed()", "[4.0,[1.0,[2.0,3.0]]]"),
        ("[[], [].toReversed()]", "[[],[]]"),
        ("[\"a\"].reversed()", "[\"a\"]"),
        (
            "class T(v):\n    this.v = v\nT(1)\nT(2)\nT.reversed().map(t -> t.v)",
            "[2.0,1.0]",
        ),
        (
            "class T(v):\n    this.v = v\nT(1)\nT(2)\n$T.toReversed().map(t -> t.v)",
            "[2.0,1.0]",
        ),
        ("class E\n[E.reversed(), E.toReversed()]", "[[],[]]"),
        (
            "class T(v):\n    this.v = v\nT(1)\nT(2)\nT(3)\n[T[1:].reversed().map(t -> t.v), T.list().toReversed().map(t -> t.v)]",
            "[[3.0,2.0],[3.0,2.0,1.0]]",
        ),
        (
            "class T(v):\n    this.v = v\nT(1)\nT(2)\ndef back(c) => c.reversed().map(t -> t.v)\nback(T)",
            "[2.0,1.0]",
        ),
        ("[1, 2].reversed(0, \"ignored\", null)", "[2.0,1.0]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn string_aliases_are_the_stepped_reversal_on_every_string() {
    let mut compared = 0;
    let mut changed = 0;
    for (setup, receiver) in TEXTS {
        for alias in TEXT_ALIASES {
            for arguments in ARGUMENTS {
                let written = format!("{SETUP}\n{setup}\n[({receiver}).{alias}({arguments}), log]");
                let spelling =
                    format!("{SETUP}\n{setup}\n[[({receiver}), [{arguments}]][0][::-1], log]");
                let found = fresh(&written);
                assert!(found.starts_with("[\""), "{written}: {found}");
                assert_eq!(found, fresh(&spelling), "{written}");
                let shown = format!("{SETUP}\n{setup}\ntypeof ({receiver}).{alias}({arguments})");
                assert_eq!(fresh(&shown), "\"string\"", "{shown}");
                compared += 1;
            }
            let plain = fresh(&format!("{setup}\n{receiver}"));
            if fresh(&format!("{setup}\n({receiver}).{alias}()")) != plain {
                changed += 1;
            }
        }
    }
    assert_eq!(compared, TEXTS.len() * TEXT_ALIASES.len() * ARGUMENTS.len());
    assert!(changed > TEXTS.len(), "{changed}");
    for (source, expected) in [
        ("\"abc\".reversed()", "\"cba\""),
        ("\"abc\".reverse()", "\"cba\""),
        ("\"café\".reversed()", "\"éfac\""),
        ("\"añejo\".reverse()", "\"ojeña\""),
        ("\"a😀b\".reversed()", "\"b😀a\""),
        ("[\"\".reversed(), \"\".reverse()]", "[\"\",\"\"]"),
        ("\"stressed\".reversed()", "\"desserts\""),
        ("\"racecar\".reverse() == \"racecar\"", "true"),
        (
            "[\"e\u{301}x\".reversed() == \"x\u{301}e\", \"e\u{301}x\".reversed().length]",
            "[true,3.0]",
        ),
        (
            "\"\u{1F1EB}\u{1F1F7}\".reverse() == \"\u{1F1F7}\u{1F1EB}\"",
            "true",
        ),
        ("\"日本語\".reversed()", "\"語本日\""),
        ("\"abc\".reversed().reverse()", "\"abc\""),
        ("w = \"ab\"\n`${w}c`.reversed()", "\"cba\""),
        ("\"ab\".reverse(1, \"x\", null)", "\"ba\""),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn arguments_are_evaluated_and_their_errors_propagate_as_for_the_spelling() {
    let mut raised = 0;
    for (setup, receiver) in LISTS {
        for alias in LIST_ALIASES {
            for arguments in [
                "missing",
                "\"a\", missing",
                "log.push(1), o9.missing.deeper",
            ] {
                let written =
                    format!("{SETUP}\no9 = {{}}\n{setup}\n({receiver}).{alias}({arguments})");
                let spelling =
                    format!("{SETUP}\no9 = {{}}\n{setup}\n({receiver}).reverse({arguments})");
                let found = fresh(&written);
                assert_eq!(found, fresh(&spelling), "{written}");
                raised += usize::from(found.starts_with("ReferenceError: missing is not defined"));
            }
        }
    }
    assert_eq!(raised, LISTS.len() * LIST_ALIASES.len() * 2);
    let mut texts = 0;
    for (setup, receiver) in TEXTS {
        for alias in TEXT_ALIASES {
            for arguments in ["missing", "\"a\", missing"] {
                let written = format!("{SETUP}\n{setup}\n({receiver}).{alias}({arguments})");
                assert_eq!(
                    fresh(&written),
                    format!(
                        "ReferenceError: missing is not defined (line {}, column 1)",
                        written.lines().count()
                    ),
                    "{written}"
                );
                texts += 1;
            }
        }
    }
    assert_eq!(texts, TEXTS.len() * TEXT_ALIASES.len() * 2);
}

#[test]
fn other_receivers_raise_or_read_exactly_as_an_unknown_method_does() {
    let mut kept = 0;
    for (setup, receiver) in OTHERS {
        for name in NAMES {
            for arguments in OTHER_ARGUMENTS {
                let written = format!("log = []\n{setup}\n[({receiver}).{name}({arguments}), log]");
                let unknown = format!("log = []\n{setup}\n[({receiver}).nope({arguments}), log]");
                assert_eq!(
                    fresh(&written),
                    fresh(&unknown).replace("nope", name),
                    "{written}"
                );
                kept += 1;
            }
        }
    }
    for (setup, receiver) in TEXTS {
        for arguments in OTHER_ARGUMENTS {
            let written = format!("log = []\n{setup}\n[({receiver}).toReversed({arguments}), log]");
            let unknown = format!("log = []\n{setup}\n[({receiver}).nope({arguments}), log]");
            assert_eq!(
                fresh(&written),
                fresh(&unknown).replace("nope", "toReversed"),
                "{written}"
            );
            kept += 1;
        }
    }
    assert_eq!(
        kept,
        OTHERS.len() * NAMES.len() * OTHER_ARGUMENTS.len() + TEXTS.len() * OTHER_ARGUMENTS.len()
    );
}

#[test]
fn the_kept_readings_keep_their_exact_accepted_results() {
    for (source, expected) in [
        (
            "o = {reversed: () -> 1, reverse: () -> 2, toReversed: x -> x + 1}\n[o.reversed(), o.reverse(), o.toReversed(2)]",
            "[1.0,2.0,3.0]",
        ),
        (
            "xs = [1, 2]\ns = \"ab\"\n[xs.reversed, xs.toReversed, xs.reverse, s.reverse, s.reversed, s.toReversed]",
            "[null,null,null,null,null,null]",
        ),
        ("def reversed(x) => x.length\nreversed([1, 2])", "2.0"),
        (
            "reversed = [1, 2, 3]\nreversed.sort((a, b) => -1)\nreversed",
            "[3.0,2.0,1.0]",
        ),
        (
            "class Team(name):\n    this.name = name\n    def reversed(): return this.name + \"!\"\nred = Team(\"red\")\n[red.reversed(), Team.reverse().length]",
            "[\"red!\",1.0]",
        ),
        (
            "class Team(name):\n    this.name = name\n    def reverse(): return this.name + \"?\"\nred = Team(\"red\")\n[red.reverse(), Team.reverse().length]",
            "[\"red?\",1.0]",
        ),
        (
            "class Q:\n    def reverse() => \"mine\"\nq = Q()\n[q.reverse(), q.reversed(), q.toReversed()]",
            "[\"mine\",null,null]",
        ),
        (
            "class Team(name):\n    this.name = name\n    def toReversed() => 7\nred = Team(\"red\")\n$Team.x = $Team.toReversed()\nred.x",
            "7.0",
        ),
        (
            "class T(v):\n    this.v = v\n$T.r = $T.reversed()\nt = T(1)\nt.r",
            "null",
        ),
        (
            "u = null\n[u.reversed(missing), u.toReversed(missing), u.reverse(missing)]",
            "[null,null,null]",
        ),
        ("o = {}\no.items.reversed(missing)", "null"),
        (
            "calls = []\nn = null\nn.reversed(calls.push(1))\nn.reverse(calls.push(2))\ncalls",
            "[]",
        ),
        (
            "xs = [3, 1, 2]\nys = xs.reverse()\n[xs, ys]",
            "[[3.0,1.0,2.0],[2.0,1.0,3.0]]",
        ),
        (
            "xs = [1, 2]\nxs.reversed()\nxs.toReversed()\nxs.reverse()\nxs",
            "[1.0,2.0]",
        ),
        (
            "kind = \"other\"\n[kind is reversed, kind is toReversed]",
            "[false,false]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "xs = [1, 2]\nxs.reversed = 1",
            "TypeError: Cannot assign 'reversed' on list (line 2, column 1)",
        ),
        (
            "xs = [1, 2]\nxs.toReversed = 1",
            "TypeError: Cannot assign 'toReversed' on list (line 2, column 1)",
        ),
        (
            "s = \"ab\"\ns.reverse = 1",
            "TypeError: Cannot assign 'reverse' on string (line 2, column 1)",
        ),
        (
            "s = \"ab\"\ns.reversed = 1",
            "TypeError: Cannot assign 'reversed' on string (line 2, column 1)",
        ),
        (
            "o = {reverse: 3}\no.reverse()",
            "TypeError: obj:0.reverse is not a function (line 2, column 1)",
        ),
        (
            "o = {toReversed: \"x\"}\no.toReversed()",
            "TypeError: obj:0.toReversed is not a function (line 2, column 1)",
        ),
        (
            "\"abc\".toReversed()",
            "TypeError: abc.toReversed is not a function (line 1, column 1)",
        ),
        (
            "(5).reversed()",
            "TypeError: number.reversed is not a function (line 1, column 1)",
        ),
        (
            "b = true\nb.reverse()",
            "TypeError: boolean.reverse is not a function (line 2, column 1)",
        ),
        (
            "f = x -> x\nf.toReversed()",
            "TypeError: function.toReversed is not a function (line 2, column 1)",
        ),
        (
            "d = Date(0)\nd.reversed()",
            "TypeError: Date.reversed is not a function (line 2, column 1)",
        ),
        (
            "r = /a/\nr.reverse()",
            "TypeError: reverse is not a function (line 2, column 1)",
        ),
        (
            "Math.reversed([1])",
            "TypeError: Math.reversed is not a function (line 1, column 1)",
        ),
        (
            "Number.toReversed()",
            "TypeError: Number.toReversed is not a function (line 1, column 1)",
        ),
        (
            "String.reverse(\"ab\")",
            "TypeError: String.reverse is not a function (line 1, column 1)",
        ),
        (
            "class String(v):\n    this.v = v\nString(1)\nString.reversed()",
            "TypeError: String.reversed is not a function (line 4, column 1)",
        ),
        (
            "Boolean.reversed()",
            "ReferenceError: Boolean is not defined (line 1, column 1)",
        ),
        (
            "List.toReversed()",
            "ReferenceError: List is not defined (line 1, column 1)",
        ),
        (
            "reversed([1, 2])",
            "ReferenceError: reversed is not defined (line 1, column 1)",
        ),
        (
            "reverse([1, 2])",
            "ReferenceError: reverse is not defined (line 1, column 1)",
        ),
        (
            "xs = [1, 2]\nreversed of xs",
            "SyntaxError: Unexpected keyword 'of' (line 2, column 13)",
        ),
        (
            "[3, 1].sorted()",
            "TypeError: [3,1].sorted is not a function (line 1, column 1)",
        ),
        (
            "[2, 1].toSorted()",
            "TypeError: [2,1].toSorted is not a function (line 1, column 1)",
        ),
        (
            "[1, 2].asReversed()",
            "TypeError: [1,2].asReversed is not a function (line 1, column 1)",
        ),
        (
            "[1, 2].Reversed()",
            "TypeError: [1,2].Reversed is not a function (line 1, column 1)",
        ),
        (
            "[1, 2].to_reversed()",
            "TypeError: [1,2].to_reversed is not a function (line 1, column 1)",
        ),
        (
            "\"ab\".to_reversed()",
            "TypeError: ab.to_reversed is not a function (line 1, column 1)",
        ),
        (
            "\"ab\".Reverse()",
            "TypeError: ab.Reverse is not a function (line 1, column 1)",
        ),
        (
            "\"ab\".reversedString()",
            "TypeError: ab.reversedString is not a function (line 1, column 1)",
        ),
        (
            "[1, 2].reversed(missing)",
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
        (
            "\"abc\".reverse(missing)",
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
    ] {
        assert_eq!(failure(source), expected, "{source}");
    }
}

#[test]
fn the_compatibility_rows_give_their_documented_results() {
    let trucks = "class Truck(plate):\n    this.plate = plate\nTruck(\"A\")\nTruck(\"B\")\n";
    let t_class = "class T(v):\n    this.v = v\nT(1)\nT(2)\n";
    let rows = [
        ("[1, 2, 3].reversed()".to_string(), "[3.0,2.0,1.0]"),
        ("[1, 2, 3].toReversed()".to_string(), "[3.0,2.0,1.0]"),
        (
            "xs = [1, 2, 3]\n[xs.reversed(), xs]".to_string(),
            "[[3.0,2.0,1.0],[1.0,2.0,3.0]]",
        ),
        (
            "[[].reversed(), [[1, [2, 3]], 4].toReversed()]".to_string(),
            "[[],[4.0,[1.0,[2.0,3.0]]]]",
        ),
        (
            format!("{trucks}Truck.reversed().map(t -> t.plate)"),
            "[\"B\",\"A\"]",
        ),
        (
            format!("{trucks}back = Truck.toReversed()\nTruck(\"C\")\nback.map(t -> t.plate)"),
            "[\"C\",\"B\",\"A\"]",
        ),
        (
            format!(
                "{trucks}Truck(\"C\")\n[Truck[1:].reversed().map(t -> t.plate), Truck.list().toReversed().length]"
            ),
            "[[\"C\",\"B\"],3.0]",
        ),
        (
            format!("{t_class}$T.reversed().map(t -> t.v)"),
            "[2.0,1.0]",
        ),
        (
            format!(
                "{t_class}kind = T\ndef back(c) => c.toReversed().map(t -> t.v)\n[kind.reversed().map(t -> t.v), back(T)]"
            ),
            "[[2.0,1.0],[2.0,1.0]]",
        ),
        (
            "class Empty\n[Empty.reversed(), Empty.toReversed()]".to_string(),
            "[[],[]]",
        ),
        ("\"abc\".reversed()".to_string(), "\"cba\""),
        ("\"abc\".reverse()".to_string(), "\"cba\""),
        (
            "[\"café\".reversed(), \"añejo\".reverse(), \"a😀b\".reversed()]".to_string(),
            "[\"éfac\",\"ojeña\",\"b😀a\"]",
        ),
        ("[\"\".reversed(), \"\".reverse()]".to_string(), "[\"\",\"\"]"),
        (
            "\"e\u{301}x\".reversed() == \"x\u{301}e\"".to_string(),
            "true",
        ),
        ("w = \"ab\"\n`${w}c`.reversed()".to_string(), "\"cba\""),
        (
            "xs = [1, 2]\nxs.push(3)\nxs@first.toReversed()".to_string(),
            "[2.0,1.0]",
        ),
        (
            "s = \"ab\"\ns = \"abcd\"\n[s@first.reversed(), s.value.reverse()]".to_string(),
            "[\"ba\",\"dcba\"]",
        ),
        (
            "xs = [1, 2]\nys = xs.value.reversed()\nxs.push(3)\nys".to_string(),
            "[2.0,1.0]",
        ),
        (
            "class Box(items):\n    this.items = items\n$Box.back = $Box.items.reversed()\nb = Box([1, 2])\nb.items.push(3)\nb.back".to_string(),
            "[3.0,2.0,1.0]",
        ),
        (
            "class Word(text):\n    this.text = text\n$Word.back = $Word.text.reversed()\nw = Word(\"abc\")\nw.text = \"xyz\"\nw.back".to_string(),
            "\"zyx\"",
        ),
        (
            "class Team(name):\n    this.name = name\n    def reversed(): return this.name + \"!\"\nred = Team(\"red\")\n[red.reversed(), Team.reversed().length]".to_string(),
            "[\"red!\",1.0]",
        ),
        (
            "class List(v):\n    this.v = v\nList(1)\nList(2)\nList.reversed().map(l -> l.v)".to_string(),
            "[2.0,1.0]",
        ),
        (
            "reversed = [3, 1, 2]\nreversed.reversed()".to_string(),
            "[2.0,1.0,3.0]",
        ),
        (
            "log = []\n[[1, 2].toReversed(log.push(1), log.push(2)), log]".to_string(),
            "[[2.0,1.0],[1.0,2.0]]",
        ),
        (
            "xs = [1, 2]\nys = xs.reverse()\nxs = \"abc\"\nys".to_string(),
            "\"cba\"",
        ),
        (
            "xs = [1, 2]\nys = xs.reversed()\nxs = \"abc\"\nys".to_string(),
            "\"cba\"",
        ),
        (
            "xs = [1, 2]\nys = xs.toReversed()\nxs = \"abc\"".to_string(),
            "TypeError: abc.toReversed is not a function (line 3, column 1)",
        ),
        (
            "xs = [1, 2]\nys = xs.reversed()\nxs = 5".to_string(),
            "TypeError: number.reversed is not a function (line 3, column 1)",
        ),
        (
            "s = \"ab\"\nt = s.reversed()\ns = 5".to_string(),
            "TypeError: number.reversed is not a function (line 3, column 1)",
        ),
        (
            "s = \"ab\"\nt = s.reversed()\ns = {reversed: () => \"own\"}\nt".to_string(),
            "\"own\"",
        ),
        (
            "xs = [1, 2]\nys = xs.reversed()\n(why ys)[0].rule".to_string(),
            "\"ys = xs.reversed()\"",
        ),
        (
            "try:\n    r = [1, 2].reversed()\ncatch e:\n    r = e\nr".to_string(),
            "[2.0,1.0]",
        ),
        (
            "[1, 2].reversed(missing)".to_string(),
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
    ];
    for (source, expected) in &rows {
        assert_eq!(fresh(source), *expected, "{source}");
    }
    assert_eq!(rows.len(), 34);
}

#[test]
fn the_public_string_method_takes_the_two_string_aliases() {
    for text in ["", "abc", "café crème", "a😀b", "e\u{301}x", "日本語"] {
        let expected = Value::String(text.chars().rev().collect());
        for alias in TEXT_ALIASES {
            assert_eq!(
                nucleoid::builtins::string_method(text, alias, &[]).unwrap(),
                expected,
                "{alias} {text}"
            );
            assert_eq!(
                nucleoid::builtins::string_method(
                    text,
                    alias,
                    &[Value::Number(1.0), Value::Null, Value::Undefined]
                )
                .unwrap(),
                expected,
                "{alias} {text} with arguments"
            );
        }
    }
    for name in ["toReversed", "Reverse", "to_reversed", "reversedString"] {
        let error = nucleoid::builtins::string_method("abc", name, &[])
            .expect_err("the name is not a string method");
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), format!("abc.{name} is not a function"));
    }
}

#[test]
fn declarations_follow_their_receivers_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            ("history = [1, 2, 3]", "history = [1, 2, 3]"),
            ("a = history.reversed()", "a = history.reverse()"),
            ("b = history.toReversed(9)", "b = history.reverse(9)"),
            ("top = history.reversed()[0]", "top = history.reverse()[0]"),
            ("[a, b, top]", "[a, b, top]"),
            ("history.push(4)", "history.push(4)"),
            ("[a, b, top]", "[a, b, top]"),
            ("history.pop()", "history.pop()"),
            ("history.shift()", "history.shift()"),
            ("[a, b, top]", "[a, b, top]"),
            (
                "history.sort((x, y) => y - x)",
                "history.sort((x, y) => y - x)",
            ),
            ("[a, b, top]", "[a, b, top]"),
            ("history = [\"z\", \"y\"]", "history = [\"z\", \"y\"]"),
            ("[a, b, top]", "[a, b, top]"),
            ("history = []", "history = []"),
            ("[a, b, top]", "[a, b, top]"),
        ],
        &["a", "b", "top"],
    );
    behaves_as_spelled(
        &[
            (
                "class Stop(name):\n    this.name = name",
                "class Stop(name):\n    this.name = name",
            ),
            ("Stop(\"a\")", "Stop(\"a\")"),
            ("s2 = Stop(\"b\")", "s2 = Stop(\"b\")"),
            ("home = Stop.reversed()", "home = Stop.reverse()"),
            (
                "names = Stop.toReversed().map(stop -> stop.name)",
                "names = Stop.reverse().map(stop -> stop.name)",
            ),
            ("[home.length, names]", "[home.length, names]"),
            ("Stop(\"c\")", "Stop(\"c\")"),
            ("[home.length, names]", "[home.length, names]"),
            ("s2.name = \"B\"", "s2.name = \"B\""),
            ("[home.length, names]", "[home.length, names]"),
            ("delete s2.name", "delete s2.name"),
            ("delete s2", "delete s2"),
            ("[home.length, names]", "[home.length, names]"),
        ],
        &["home", "names"],
    );
    behaves_as_spelled(
        &[
            ("word = \"crème\"", "word = \"crème\""),
            ("m = word.reversed()", "m = word[::-1]"),
            ("n = word.reverse(1)", "n = word[::-1]"),
            ("[m, n]", "[m, n]"),
            ("word = \"a😀b\"", "word = \"a😀b\""),
            ("[m, n]", "[m, n]"),
            ("word = \"\"", "word = \"\""),
            ("[m, n]", "[m, n]"),
            ("first = \"Ana\"", "first = \"Ana\""),
            (
                "full = `${first} Lima`.reversed()",
                "full = `${first} Lima`[::-1]",
            ),
            ("first = \"Bo\"", "first = \"Bo\""),
            ("full", "full"),
        ],
        &["m", "n", "full"],
    );
    behaves_as_spelled(
        &[
            (
                "o = {items: [1, 2], name: \"Ada\"}",
                "o = {items: [1, 2], name: \"Ada\"}",
            ),
            ("r = o.items.toReversed()", "r = o.items.reverse()"),
            ("q = o.name.reversed()", "q = o.name[::-1]"),
            ("[r, q]", "[r, q]"),
            ("o.items.push(3)", "o.items.push(3)"),
            ("o.name = \"Bob\"", "o.name = \"Bob\""),
            ("[r, q]", "[r, q]"),
            (
                "o = {items: [7], name: \"Cy\"}",
                "o = {items: [7], name: \"Cy\"}",
            ),
            ("[r, q]", "[r, q]"),
        ],
        &["r", "q"],
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "history = [\"open\", \"save\"]\nundo = history.reversed()\nassert(undo, [\"save\", \"open\"])\nhistory.push(\"close\")\nassert(undo, [\"close\", \"save\", \"open\"])",
    );
    assert_eq!(dependencies(&runtime, "undo"), ["history"]);
}

#[test]
fn class_level_rules_follow_each_instance_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            (
                "class Train(cars, code):\n    this.cars = cars\n    this.code = code",
                "class Train(cars, code):\n    this.cars = cars\n    this.code = code",
            ),
            (
                "$Train.rear = $Train.cars.reversed()",
                "$Train.rear = $Train.cars.reverse()",
            ),
            (
                "$Train.back = $Train.cars.toReversed()",
                "$Train.back = $Train.cars.reverse()",
            ),
            (
                "$Train.mirror = $Train.code.reversed()",
                "$Train.mirror = $Train.code[::-1]",
            ),
            (
                "$Train.flipped = $Train.code.reverse()",
                "$Train.flipped = $Train.code[::-1]",
            ),
            (
                "a = Train([\"e\", \"c\"], \"AB1\")",
                "a = Train([\"e\", \"c\"], \"AB1\")",
            ),
            ("b = Train([], \"\")", "b = Train([], \"\")"),
            (
                "[a.rear, a.back, a.mirror, a.flipped, b.rear, b.mirror]",
                "[a.rear, a.back, a.mirror, a.flipped, b.rear, b.mirror]",
            ),
            ("a.cars.push(\"x\")", "a.cars.push(\"x\")"),
            ("b.code = \"ZY\"", "b.code = \"ZY\""),
            (
                "[a.rear, a.back, a.mirror, a.flipped, b.rear, b.mirror]",
                "[a.rear, a.back, a.mirror, a.flipped, b.rear, b.mirror]",
            ),
            ("a.cars = [\"q\"]", "a.cars = [\"q\"]"),
            (
                "[a.rear, a.back, a.mirror, a.flipped, b.rear, b.mirror]",
                "[a.rear, a.back, a.mirror, a.flipped, b.rear, b.mirror]",
            ),
        ],
        &[],
    );
    behaves_as_spelled(
        &[
            (
                "class Word(text):\n    this.text = text",
                "class Word(text):\n    this.text = text",
            ),
            (
                "if $Word.text.reversed() == $Word.text:\n    $Word.kind = \"palindrome\"\nelse:\n    $Word.kind = \"plain\"",
                "if $Word.text[::-1] == $Word.text:\n    $Word.kind = \"palindrome\"\nelse:\n    $Word.kind = \"plain\"",
            ),
            ("a = Word(\"noon\")", "a = Word(\"noon\")"),
            ("b = Word(\"moon\")", "b = Word(\"moon\")"),
            ("[a.kind, b.kind]", "[a.kind, b.kind]"),
            ("b.text = \"otto\"", "b.text = \"otto\""),
            ("a.text = \"nun\"", "a.text = \"nun\""),
            ("[a.kind, b.kind]", "[a.kind, b.kind]"),
            ("a.text = \"nuns\"", "a.text = \"nuns\""),
            ("[a.kind, b.kind]", "[a.kind, b.kind]"),
        ],
        &[],
    );
    assert_eq!(
        result(
            "class Shelf(books):\n    this.books = books\n$Shelf.spines = $Shelf.books.toReversed()\nleft = Shelf([\"A\", \"B\"])\nright = Shelf([\"C\"])\nspines = Shelf.map(shelf -> shelf.spines)\nleft.books.push(\"D\")\nspines"
        ),
        "[[\"D\",\"B\",\"A\"],[\"C\"]]"
    );
}

fn held_rows(
    alias: &str,
) -> [(
    &'static str,
    Option<String>,
    &'static str,
    Vec<&'static str>,
); 8] {
    let raised = |kind: &str| Some(format!("TypeError: {kind}.{alias} is not a function"));
    [
        ("xs = [3, 4, 5]", None, "[5.0,4.0,3.0]", vec!["xs"]),
        ("xs = 5", raised("number"), "[2.0,1.0]", vec!["xs"]),
        ("xs = false", raised("boolean"), "[2.0,1.0]", vec!["xs"]),
        ("xs = Date(0)", raised("Date"), "[2.0,1.0]", vec!["xs"]),
        ("xs = x -> x", raised("function"), "[2.0,1.0]", vec!["xs"]),
        ("xs = {other: 1}", None, "null", vec!["xs", "member"]),
        ("xs = null", None, "null", vec!["xs"]),
        ("xs = [\"a\"]", None, "[\"a\"]", vec!["xs"]),
    ]
}

#[test]
fn a_held_receiver_replaced_by_another_kind_reads_the_alias_name_on_it() {
    for alias in NAMES {
        for (replacement, error, value, from) in held_rows(alias) {
            let mut runtime = Runtime::new();
            runtime
                .run(&format!("xs = [1, 2]\nys = xs.{alias}()"))
                .unwrap();
            match (runtime.run(replacement), error) {
                (Err(found), Some(expected)) => {
                    assert_eq!(
                        found.to_string(),
                        format!("{expected} (line 1, column 1)"),
                        "{alias}: {replacement}"
                    );
                    assert_eq!(json(&mut runtime, "xs"), "[1.0,2.0]", "{replacement}");
                }
                (Ok(_), None) => {}
                (found, expected) => {
                    panic!("{alias}: {replacement}: {found:?} where {expected:?} was expected")
                }
            }
            assert_eq!(json(&mut runtime, "ys"), value, "{alias}: {replacement}");
            let names: Vec<String> = from
                .iter()
                .map(|name| {
                    if *name == "member" {
                        format!("\"obj:0.{alias}\"")
                    } else {
                        format!("\"{name}\"")
                    }
                })
                .collect();
            assert_eq!(
                json(&mut runtime, "(why ys)[0].from"),
                format!("[{}]", names.join(",")),
                "{alias}: {replacement}"
            );
        }
    }
    for (alias, value) in [
        ("reversed", "\"cba\""),
        ("reverse", "\"cba\""),
        ("toReversed", "[2.0,1.0]"),
    ] {
        let mut runtime = Runtime::new();
        runtime
            .run(&format!("xs = [1, 2]\nys = xs.{alias}()"))
            .unwrap();
        let written = runtime.run("xs = \"abc\"");
        assert_eq!(written.is_ok(), alias != "toReversed", "{alias}");
        if let Err(error) = written {
            assert_eq!(
                error.to_string(),
                "TypeError: abc.toReversed is not a function (line 1, column 1)"
            );
        }
        assert_eq!(json(&mut runtime, "ys"), value, "{alias}");
    }
    for (alias, own) in [
        ("reversed", "\"own\""),
        ("toReversed", "\"own too\""),
        ("reverse", "\"mine\""),
    ] {
        assert_eq!(
            result(&format!(
                "xs = [1, 2]\nys = xs.{alias}()\nxs = {{reversed: () => \"own\", toReversed: () => \"own too\", reverse: () => \"mine\"}}\nys"
            )),
            own,
            "{alias}"
        );
    }
    for (replacement, reversed, reverse, stepped) in [
        ("s = \"xyz\"", "\"zyx\"", "\"zyx\"", "\"zyx\""),
        ("s = [1, 2]", "[2.0,1.0]", "[2.0,1.0]", "[2.0,1.0]"),
        ("s = 5", "error", "error", "null"),
        ("s = true", "error", "error", "null"),
        ("s = Date(0)", "error", "error", "null"),
        ("s = x -> x", "error", "error", "null"),
        ("s = {reversed: () => \"own\"}", "\"own\"", "null", "null"),
        ("s = {reverse: () => \"mine\"}", "null", "\"mine\"", "null"),
        ("s = {other: 1}", "null", "null", "null"),
        ("s = null", "null", "null", "null"),
    ] {
        for (call, expected) in [
            (".reversed()", reversed),
            (".reverse()", reverse),
            ("[::-1]", stepped),
        ] {
            let mut runtime = Runtime::new();
            runtime.run(&format!("s = \"ab\"\nt = s{call}")).unwrap();
            match runtime.run(replacement) {
                Ok(_) => assert_eq!(json(&mut runtime, "t"), expected, "{call}: {replacement}"),
                Err(error) => {
                    assert_eq!(expected, "error", "{call}: {replacement}: {error}");
                    assert!(
                        error.to_string().starts_with("TypeError: ")
                            && error.to_string().contains(&format!(
                                ".{} is not a function",
                                call.trim_start_matches('.').trim_end_matches("()")
                            )),
                        "{call}: {replacement}: {error}"
                    );
                    assert_eq!(json(&mut runtime, "[s, t]"), "[\"ab\",\"ba\"]");
                }
            }
        }
    }
}

#[test]
fn explanations_name_the_dependencies_of_the_accepted_spelling() {
    let setup = "class Stop(name):\n    this.name = name\nStop(\"a\")\nStop(\"b\")\nhistory = [1, 2]\nword = \"ab\"\n";
    let mut alias = Runtime::new();
    let mut spelled = Runtime::new();
    alias
        .run(&format!(
            "{setup}u = history.reversed()\nv = history.toReversed(1)\nh = Stop.reversed()\nm = word.reversed()\nn = word.reverse()"
        ))
        .unwrap();
    spelled
        .run(&format!(
            "{setup}u = history.reverse()\nv = history.reverse(1)\nh = Stop.reverse()\nm = word[::-1]\nn = word[::-1]"
        ))
        .unwrap();
    for (name, rule, spelling) in [
        (
            "u",
            "\"u = history.reversed()\"",
            "\"u = history.reverse()\"",
        ),
        (
            "v",
            "\"v = history.toReversed(1)\"",
            "\"v = history.reverse(1)\"",
        ),
        ("h", "\"h = Stop.reversed()\"", "\"h = Stop.reverse()\""),
        ("m", "\"m = word.reversed()\"", "\"m = word[::-1]\""),
        ("n", "\"n = word.reverse()\"", "\"n = word[::-1]\""),
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
            format!("(why {name}).map(step -> step.state)"),
        ] {
            assert_eq!(
                json(&mut alias, &query),
                json(&mut spelled, &query),
                "{query}"
            );
        }
    }
    assert_eq!(dependencies(&alias, "u"), ["history"]);
    assert_eq!(dependencies(&alias, "h"), ["$Stop"]);
    assert_eq!(dependencies(&alias, "m"), ["word"]);
    for query in [
        "(affects history).map(step -> step.node)",
        "(affects word).map(step -> step.node)",
    ] {
        assert_eq!(
            json(&mut alias, query),
            json(&mut spelled, query),
            "{query}"
        );
    }
    alias
        .run("history.push(3)\nStop(\"c\")\nword = \"xyz\"")
        .unwrap();
    spelled
        .run("history.push(3)\nStop(\"c\")\nword = \"xyz\"")
        .unwrap();
    let values = "[u, v, h.map(stop -> stop.name), m, n]";
    assert_eq!(
        json(&mut alias, values),
        "[[3.0,2.0,1.0],[3.0,2.0,1.0],[\"c\",\"b\",\"a\"],\"zyx\",\"zyx\"]"
    );
    assert_eq!(json(&mut alias, values), json(&mut spelled, values));
    for (source, rendered) in [
        ("ys = xs.reversed()", "ys=xs.reversed()"),
        ("ys = xs.toReversed(1, 2)", "ys=xs.toReversed(1,2)"),
        ("t = \"ab\".reverse()", "t=\"ab\".reverse()"),
        (
            "$Train.rear = $Train.cars.toReversed()",
            "$Train.rear=$Train.cars.toReversed()",
        ),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), rendered, "{source}");
    }
}

#[test]
fn the_receiver_and_then_every_argument_evaluate_once_in_written_order() {
    let functions = "log = []\ndef items(label):\n    log.push(label)\n    return [1, 2]\ndef text(label):\n    log.push(label)\n    return \"ab\"\ndef arg(label):\n    log.push(label)\n    return 0\n";
    for (receiver, name, value) in [
        ("items", "reversed", "[2.0,1.0]"),
        ("items", "toReversed", "[2.0,1.0]"),
        ("items", "reverse", "[2.0,1.0]"),
        ("text", "reversed", "\"ba\""),
        ("text", "reverse", "\"ba\""),
    ] {
        for (call, expected_log) in [
            (format!("{receiver}(\"r\").{name}()"), "[\"r\"]"),
            (
                format!("{receiver}(\"r\").{name}(arg(\"a\"), arg(\"b\"))"),
                "[\"r\",\"a\",\"b\"]",
            ),
            (
                format!("{receiver}(\"r\").{name}(arg(\"a\"), items(\"x\"), arg(\"c\"))"),
                "[\"r\",\"a\",\"x\",\"c\"]",
            ),
        ] {
            let source = format!("{functions}[{call}, log]");
            assert_eq!(
                result(&source),
                format!("[{value},{expected_log}]"),
                "{source}"
            );
        }
        let mut runtime = Runtime::new();
        runtime.run(functions).unwrap();
        assert_eq!(
            runtime
                .run(&format!(
                    "{receiver}(\"r\").{name}(arg(\"a\"), missing, arg(\"b\"))"
                ))
                .unwrap_err()
                .to_string(),
            "ReferenceError: missing is not defined (line 1, column 1)"
        );
        assert_eq!(json(&mut runtime, "log"), "[]");
    }
    assert_eq!(
        result(
            "class T(v):\n    this.v = v\nT(1)\nlog = []\ndef arg(label):\n    log.push(label)\n    return 0\n[T.reversed(arg(\"a\"), arg(\"b\")).length, log]"
        ),
        "[1.0,[\"a\",\"b\"]]"
    );
}

#[test]
fn aliases_read_in_every_statement_position() {
    for (source, expected) in [
        (
            "r = 0\ntry:\n    r = (5).reversed()\ncatch error:\n    r = error\nr",
            "\"TypeError: number.reversed is not a function\"",
        ),
        (
            "r = 0\ntry:\n    r = \"ab\".toReversed()\ncatch error:\n    r = error\nr",
            "\"TypeError: ab.toReversed is not a function\"",
        ),
        (
            "word = \"ab\"\nmirror = word.reversed()\nseen = suppose word = \"xyz\": mirror\n[seen, mirror, word]",
            "[\"zyx\",\"ba\",\"ab\"]",
        ),
        (
            "name = \"ada\"\n`${name.reversed()}|${[1, 2].toReversed()}`",
            "\"ada|2,1\"",
        ),
        (
            "code = \"level\"\nif code.reverse() == code:\n    state = \"same\"\nelse:\n    state = \"differs\"\nstate",
            "\"same\"",
        ),
        (
            "xs = [1, 2]\nxs.reversed()[0] == 2 ? \"last\" : \"first\"",
            "\"last\"",
        ),
        (
            "def mirror(text) => text.reversed() + \"|\" + text\n[mirror(\"ab\"), mirror(\"\")]",
            "[\"ba|ab\",\"|\"]",
        ),
        (
            "def back(xs):\n    return xs.toReversed()\nback([1, 2, 3])",
            "[3.0,2.0,1.0]",
        ),
        (
            "words = [\"ab\", \"cd\"]\nwords.map(w -> w.reverse()).filter(w -> w.reversed() != \"cd\")",
            "[\"ba\"]",
        ),
        (
            "seen = []\nfor c in \"abc\".reversed():\n    seen.push(c)\nseen",
            "[\"c\",\"b\",\"a\"]",
        ),
        (
            "stack = [1, 2, 3]\ni = 0\nwhile stack.reversed()[i] != 1:\n    i = i + 1\ni",
            "2.0",
        ),
        (
            "class Tag(name):\n    this.name = name\n    def back() => this.name.reversed()\n    def flip() => self.name.reverse()\nt = Tag(\"ab\")\n[t.back(), t.flip()]",
            "[\"ba\",\"ba\"]",
        ),
        (
            "[[1, 2, 3].reversed().reversed(), \"abc\".reverse().reversed(), [1, 2].toReversed().reverse()]",
            "[[1.0,2.0,3.0],\"abc\",[1.0,2.0]]",
        ),
        (
            "class P(v):\n    this.v = v\np = P([1, 2])\nq = P(\"ab\")\n[p.v.reversed(), q.v.reverse()]",
            "[[2.0,1.0],\"ba\"]",
        ),
        (
            "class Team(name):\n    this.name = name\n    def reversed() => \"own\"\n$Team.r = $Team.reversed()\n$Team.n = $Team.name.reversed()\nt = Team(\"ab\")\n[t.r, t.n]",
            "[\"own\",\"ba\"]",
        ),
        (
            "o = {a: 1}\nxs = [o, 2]\nys = xs.reversed()\nys[1].a = 5\n[xs[0].a, ys[1] == o, xs.length]",
            "[5.0,true,2.0]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn failed_statements_and_transactions_restore_alias_declarations() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "def latest(log):\n    if log.length > 3:\n        throw \"FULL\"\n    return log\nentries = [\"boot\", \"sync\"]\nrecent = latest(entries.reversed())\nfailure = \"\"\ntry:\n    entries.push(\"scan\", \"send\")\ncatch error:\n    failure = error\nassert([failure, entries, recent], [\"FULL\", [\"boot\", \"sync\"], [\"sync\", \"boot\"]])",
    );
    checked(
        &mut runtime,
        "class Tag(label):\n    this.label = label\n$Tag.mirror = $Tag.label.reversed()\ntag = Tag(\"AB\")\ntry:\n    tag.label = 7\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.reversed is not a function\"))\nassert([tag.label, tag.mirror], [\"AB\", \"BA\"])",
    );
    checked(
        &mut runtime,
        "class Job(name):\n    this.name = name\nJob(\"a\")\nqueue = latest(Job.toReversed())\ntry:\n    Job(\"b\")\n    Job(\"c\")\n    Job(\"d\")\ncatch error:\n    failure = error\nassert([failure, Job.length, queue.length], [\"FULL\", 1, 1])",
    );
    checked(
        &mut runtime,
        "word = \"ab\"\nmirror = word.reverse()\ntry:\n    word = \"xyz\"\n    entries.push(\"scan\")\n    tag.label = \"CD\"\n    assert([mirror, recent, tag.mirror], [\"zyx\", [\"scan\", \"sync\", \"boot\"], \"DC\"])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([word, mirror, entries, recent, tag.mirror], [\"ab\", \"ba\", [\"boot\", \"sync\"], [\"sync\", \"boot\"], \"BA\"])",
    );
    assert_eq!(dependencies(&runtime, "mirror"), ["word"]);
    assert_eq!(
        failure("xs = [1, 2]\nys = xs.toReversed()\nxs = \"ab\""),
        "TypeError: ab.toReversed is not a function (line 3, column 1)"
    );
    let mut recovered = Runtime::new();
    recovered.run("s = \"ab\"\nt = s.reversed()").unwrap();
    assert!(recovered.run("s = 5").is_err());
    assert_eq!(json(&mut recovered, "[s, t]"), "[\"ab\",\"ba\"]");
    recovered.run("s = \"été\"").unwrap();
    assert_eq!(json(&mut recovered, "t"), "\"été\"");
    recovered.run("s = \"nuit\"").unwrap();
    assert_eq!(json(&mut recovered, "t"), "\"tiun\"");
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    for (receiver, alias, value) in [
        ("[1, 2]", "reversed", "[2.0,1.0]"),
        ("[1, 2]", "toReversed", "[2.0,1.0]"),
        ("Item", "reversed", "[3.0,2.0,1.0]"),
        ("Item", "toReversed", "[3.0,2.0,1.0]"),
        ("\"ab\"", "reversed", "\"ba\""),
        ("\"ab\"", "reverse", "\"ba\""),
    ] {
        let mapped = if receiver == "Item" {
            ".map(item -> item.v)"
        } else {
            ""
        };
        assert_eq!(
            result(&format!(
                "{SETUP}\ntry:\n    r = {receiver}.{alias}(){mapped}\ncatch e:\n    r = \"caught\"\nr"
            )),
            value,
            "{receiver}.{alias}"
        );
    }
    for source in [
        "try:\n    r = \"ab\".toReversed()\ncatch e:\n    r = \"caught\"\nr",
        "try:\n    r = (5).reversed()\ncatch e:\n    r = \"caught\"\nr",
        "try:\n    r = [1].sorted()\ncatch e:\n    r = \"caught\"\nr",
    ] {
        assert_eq!(result(source), "\"caught\"", "{source}");
    }
}

#[test]
fn long_lists_and_strings_reverse_through_the_runtime_and_the_public_api() {
    let mut runtime = Runtime::new();
    runtime
        .run("long = \"abc😀\".repeat(30000)\nitems = long.split(\"\")\nback = items.reversed()\nflipped = long.reversed()")
        .unwrap();
    assert_eq!(
        json(
            &mut runtime,
            "[items.length, back.length, back == items.reverse(), items.toReversed() == back, flipped.length, flipped == long[::-1], flipped.reverse() == long]"
        ),
        "[120000.0,120000.0,true,true,120000.0,true,true]"
    );
    let text = "café 😀 e\u{301} ".repeat(10_000);
    let Value::String(reversed) =
        nucleoid::builtins::string_method(&text, "reversed", &[]).unwrap()
    else {
        panic!("the alias returns a string");
    };
    assert_eq!(reversed, text.chars().rev().collect::<String>());
    assert_eq!(reversed.chars().count(), text.chars().count());
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-reversal-aliases-{label}-{}.nuc",
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
    let program = "class Stop(name):\n    this.name = name\nStop(\"a\")\nhistory = [1, 2]\nundo = history.reversed()\nlast = history.toReversed()[0]\nhome = Stop.reversed()\nword = \"crème\"\nmirror = word.reversed()\nflip = word.reverse()\nhistory.push(3)\nStop(\"b\")\nword = \"a😀b\"\n[undo, last, home.map(stop -> stop.name), mirror, flip]";
    let (spelled, rewrites) = spelled_out(program);
    assert_eq!(rewrites, 5, "{spelled}");
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
        "[[3.0,2.0,1.0],3.0,[\"b\",\"a\"],\"b😀a\",\"b😀a\"]\n"
    );
    let graph = String::from_utf8(file.run(program, &["--graph"]).stdout).unwrap();
    assert!(
        graph.contains("undo [variable]\n    reads    history"),
        "{graph}"
    );
    assert!(
        graph.contains("mirror [variable]\n    reads    word"),
        "{graph}"
    );
    let failed = file.run("x = 5\nx.reversed()\n", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "TypeError: number.reversed is not a function (line 2, column 1)\n"
    );
    let session = "history = [1, 2]\nundo = history.reversed()\nundo\nhistory.push(3)\nundo\nhistory.toReversed()\n\"café\".reversed()\n\"a😀b\".reverse()\n\"\".reversed()\nclass Stop(name):\n    this.name = name\n\nStop(\"a\")\nStop(\"b\")\nStop.reversed().map(stop -> stop.name)\n.exit\n";
    let (spelled_session, session_rewrites) = spelled_out(session);
    assert_eq!(session_rewrites, 6, "{spelled_session}");
    assert!(!calls_alias(&spelled_session), "{spelled_session}");
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let alias = repl(&session.replace('\n', newline), options);
            let written = repl(&spelled_session.replace('\n', newline), options);
            assert!(alias.status.success(), "{options:?} {alias:?}");
            assert_eq!(alias.stdout, written.stdout, "{options:?}");
            assert_eq!(alias.stderr, written.stderr, "{options:?}");
            let stdout = String::from_utf8(alias.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "[2.0,1.0]\n3.0\n[3.0,2.0,1.0]\n[3.0,2.0,1.0]\n\"éfac\"\n\"b😀a\"\n\"\"\n"
            } else {
                "2,1\n3\n3,2,1\n3,2,1\néfac\nb😀a\n\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
            let names = if options.contains(&"--json") {
                "[\"b\",\"a\"]\n"
            } else {
                "b,a\n"
            };
            assert!(stdout.contains(names), "{options:?} {stdout}");
        }
    }
    let failing = repl(
        "(5).reversed()\n\"Ab\".toReversed()\n\"Ab\".reversed()\n.exit\n",
        &["--plain"],
    );
    assert!(failing.status.success(), "{failing:?}");
    assert_eq!(
        String::from_utf8(failing.stderr).unwrap(),
        "TypeError: number.reversed is not a function (line 1, column 1)\nTypeError: Ab.toReversed is not a function (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(failing.stdout).unwrap();
    assert!(stdout.contains("bA\n"), "{stdout}");
}
