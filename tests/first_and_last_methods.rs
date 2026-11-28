mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{Runtime, Value};

const SPEC_TITLES: [&str; 3] = [
    "Nucleoid reads the first and last items of lists and populations",
    "Nucleoid finds the first and last matches with first and last predicates",
    "Nucleoid rolls back failed updates through first and last aliases",
];
const REFERENCE_TITLE: &str = "Nucleoid reads first and last elements as Kotlin and Rust do";
const README_TITLE: &str = "First and last methods keep a sensor's newest reading live";
const EXAMPLES_TITLE: &str = "First and last methods keep the next urgent patient live";

const ALIASES: [&str; 2] = ["first", "last"];

const SETUP: &str = "class Item(v):\n    this.v = v\nItem(1)\nItem(2)\nItem(3)\nclass Empty\nclass Reader(v):\n    this.v = v\n    def first() => \"own first\"\n    def last() => \"own last\"\nReader(1)\nReader(2)\nhollow = {}";

const RECEIVERS: [(&str, &str); 29] = [
    ("", "[1, 2, 3]"),
    ("", "[]"),
    ("xs = [1, 2, 3]", "xs"),
    ("xs = []", "xs"),
    ("o = {items: [1, 2]}", "o.items"),
    ("def make() => [3, 2, 1]", "make()"),
    ("xs = [1, 2, 3]", "xs[1:]"),
    ("xs = [1, 2, 3]", "xs.value"),
    ("xs = [1]\nxs.push(2)", "xs@first"),
    ("xs = [1]\nxs.push(2)", "xs@last"),
    ("grid = [[1, 2], [3]]", "grid[0]"),
    ("xs = [1, 2, 3]", "xs.filter(x -> x > 1)"),
    ("xs = [1, 2]", "xs.map(x -> x * 2)"),
    ("xs = [1]", "xs.concat([2, 3])"),
    ("", "\"a,b,c\".split(\",\")"),
    ("", "[null, [], \"\"]"),
    ("", "[{v: 1}, {v: 2}]"),
    ("xs = [3]\nxs.pop()", "xs"),
    ("", "Item.list()"),
    ("", "Item[1:]"),
    ("", "Item.filter(item -> item.v > 1)"),
    ("", "Item"),
    ("kind = Item", "kind"),
    ("", "$Item"),
    ("", "Empty"),
    ("", "Reader"),
    ("class List(v):\n    this.v = v\nList(1)\nList(2)", "List"),
    (
        "class Object(v):\n    this.v = v\nObject(1)\nObject(2)",
        "Object",
    ),
    ("class Class(v):\n    this.v = v\nClass(1)", "Class"),
];

const PREDICATES: [&str; 4] = [
    "x -> true",
    "x -> false",
    "x -> x == 2 || x.v == 2",
    "x -> true, 9, \"extra\"",
];

const NON_FUNCTIONS: [&str; 7] = [
    "3",
    "\"a\"",
    "null",
    "[1]",
    "{v: 1}",
    "Item",
    "hollow.missing",
];

const OTHERS: [(&str, &str); 31] = [
    ("", "5"),
    ("", "0"),
    ("", "true"),
    ("f = x -> x", "f"),
    ("", "Date(0)"),
    ("", "/a/"),
    ("", "\"abc\""),
    ("", "\"\""),
    ("s = \"hello\"", "s"),
    ("stem = \"ab\"", "`${stem}c`"),
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
    (
        "class Math(v):\n    this.v = v\nMath(1)\nkind = Math",
        "kind",
    ),
    ("", "Ghost"),
];

const ARGUMENTS: [&str; 4] = ["", "x -> true", "3", "missing"];

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
            title: "first and last alias regression".to_string(),
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
    let mut declared = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(calls_alias(&case.source), new, "{}", case.title);
        let tokens = tokens(&case.source);
        if !new && calls_member(&tokens) {
            assert!(declares_member(&tokens), "{}", case.title);
            declared += 1;
        }
        if new {
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
    assert!(declared > 0, "an accepted case calls a member it declares");
    assert_eq!(assertions, 28);
    assert_eq!(cases, 1333);
}

#[test]
fn the_audit_finds_every_alias_call_and_no_declared_member() {
    for source in [
        "xs.first()",
        "xs.last()",
        "xs.first(x -> x > 1)",
        "Item.last(item -> item.open, 9)",
        "f(x).first()",
        "[1, 2].last()",
        "$Sensor.readings.last()",
        "kind.first()",
        "`${xs.last()} items`",
        "`${count(1)} and ${f({a: queue.first()})}`",
        "x = cond ? xs.first() : null",
    ] {
        assert!(calls_alias(source), "{source}");
    }
    for source in [
        "xs.at(0)",
        "xs.find(x -> x)",
        "xs.first",
        "xs@first",
        "xs@last.length",
        "first(xs)",
        "last = xs.at(-1)",
        "\"first\"",
        "`${first}`",
        "class Reader:\n    def first(records) => records.length\nreader = Reader()\nreader.first([1])",
        "class Deck:\n    def last() => 1\nDeck().last()",
        "o = {first: () => 1}\no.first()",
        "o = {}\no.last = x -> x\no.last(2)",
        "function first(xs) { return 1 }\nxs.first()",
    ] {
        assert!(!calls_alias(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_116_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[783],
        "Nucleoid rolls back failed updates through size and emptiness aliases"
    );
    assert_eq!(titles[784..787], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid spells size, len and isEmpty as Java, Kotlin and Rust do",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Size and emptiness methods keep an order queue's rules live",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Emptiness methods keep every shelf's restock count live",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 23 - 12 * usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 14].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### First and last element methods\n\n```\n")
        .expect("the language reference documents the first and last aliases");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 117](nuc-0117.md)"));
    assert!(reference.contains(
        "items.first()  items.last()            exactly items.at(0) and items.at(-1); Name.last() is the latest instance (NUC 117)"
    ));
    assert!(reference.contains(
        "items.first(test)  items.last(test)    exactly items.find(test) and items.findLast(test), on lists and populations (NUC 117)"
    ));
    assert_eq!(case.expected.as_deref(), Some("[null,null]"));
}

#[test]
fn each_alias_gives_its_accepted_spellings_result_on_every_receiver() {
    let mut answers = [0_usize; 4];
    for (setup, receiver) in RECEIVERS {
        for (alias, at, find) in [("first", "0", "find"), ("last", "-1", "findLast")] {
            let mut pairs = vec![(
                format!("({receiver}).{alias}()"),
                format!("({receiver}).at({at})"),
            )];
            for predicate in PREDICATES {
                pairs.push((
                    format!("({receiver}).{alias}({predicate})"),
                    format!("({receiver}).{find}({predicate})"),
                ));
            }
            for (written, spelled) in pairs {
                let found = fresh(&format!("{SETUP}\n{setup}\n{written}"));
                assert_eq!(
                    found,
                    fresh(&format!("{SETUP}\n{setup}\n{spelled}")),
                    "{setup}\n{written}"
                );
                assert_eq!(
                    fresh(&format!("{SETUP}\n{setup}\ntypeof {written}")),
                    fresh(&format!("{SETUP}\n{setup}\ntypeof {spelled}")),
                    "{setup}\ntypeof {written}"
                );
                match found.as_str() {
                    "null" => answers[0] += 1,
                    number if number.parse::<f64>().is_ok() => answers[1] += 1,
                    object if object.starts_with('{') => answers[2] += 1,
                    other => {
                        assert!(!other.contains("Error"), "{written}: {other}");
                        answers[3] += 1;
                    }
                }
            }
        }
    }
    assert_eq!(
        answers.iter().sum::<usize>(),
        RECEIVERS.len() * ALIASES.len() * (1 + PREDICATES.len())
    );
    assert!(
        answers[..3].iter().all(|count| *count > 50) && answers[3] > 0,
        "{answers:?}"
    );
    for (source, expected) in [
        ("[4, 5, 6].first()", "4.0"),
        ("[4, 5, 6].last()", "6.0"),
        ("[].first()", "null"),
        ("[4, 5, 6].first(x -> x > 4)", "5.0"),
        ("[4, 5, 6].last(x -> x < 6)", "5.0"),
        ("[4, 5, 6].last(x -> x > 9)", "null"),
        (
            "class Item(v):\n    this.v = v\nItem(1)\nItem(2)\nItem.last().v",
            "2.0",
        ),
        ("class Empty\n[Empty.first(), Empty.last()]", "[null,null]"),
        ("xs = [[1, 2], [3]]\nxs.first().last()", "2.0"),
        ("xs = [1, 2]\n[xs.first(), xs.first]", "[1.0,null]"),
        (
            "[typeof [].first(), typeof [].last(), typeof [1].first(x -> false), typeof [1].last(x -> false)]",
            "[\"undefined\",\"undefined\",\"null\",\"null\"]",
        ),
        (
            "class Empty\n[typeof Empty.first(), typeof Empty.last(), typeof Empty.first(x -> true)]",
            "[\"null\",\"null\",\"null\"]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn other_first_arguments_raise_finds_error_with_the_name_as_written() {
    let mut raised = 0;
    for (setup, receiver) in RECEIVERS {
        for (alias, find) in [("first", "find"), ("last", "findLast")] {
            for argument in NON_FUNCTIONS {
                let written = format!("{SETUP}\n{setup}\n({receiver}).{alias}({argument})");
                let spelled = format!("{SETUP}\n{setup}\n({receiver}).{find}({argument})");
                let found = fresh(&written);
                assert_eq!(
                    found,
                    fresh(&spelled).replace(
                        &format!("{find} expects a function"),
                        &format!("{alias} expects a function")
                    ),
                    "{written}"
                );
                assert!(
                    found.starts_with(&format!("TypeError: {alias} expects a function")),
                    "{written}: {found}"
                );
                raised += 1;
            }
        }
    }
    assert_eq!(
        raised,
        RECEIVERS.len() * ALIASES.len() * NON_FUNCTIONS.len()
    );
}

#[test]
fn arguments_are_evaluated_and_their_errors_propagate_as_for_find() {
    let mut checked_programs = 0;
    for (setup, receiver) in RECEIVERS {
        for (alias, find) in [("first", "find"), ("last", "findLast")] {
            for arguments in ["missing", "x -> true, missing"] {
                let written = format!("{SETUP}\n{setup}\n({receiver}).{alias}({arguments})");
                let spelled = format!("{SETUP}\n{setup}\n({receiver}).{find}({arguments})");
                let found = fresh(&written);
                assert_eq!(found, fresh(&spelled), "{written}");
                assert!(
                    found.starts_with("ReferenceError: missing is not defined"),
                    "{written}: {found}"
                );
                checked_programs += 1;
            }
        }
    }
    assert_eq!(checked_programs, RECEIVERS.len() * ALIASES.len() * 2);
}

#[test]
fn other_receivers_raise_or_read_exactly_as_an_unknown_method_does() {
    let mut kept = 0;
    for (setup, receiver) in OTHERS {
        for alias in ALIASES {
            for arguments in ARGUMENTS {
                let written = format!("{setup}\n({receiver}).{alias}({arguments})");
                let unknown = format!("{setup}\n({receiver}).nope({arguments})");
                assert_eq!(
                    fresh(&written),
                    fresh(&unknown).replace("nope", alias),
                    "{written}"
                );
                kept += 1;
            }
        }
    }
    assert_eq!(kept, OTHERS.len() * ALIASES.len() * ARGUMENTS.len());
}

#[test]
fn the_kept_readings_keep_their_exact_accepted_results() {
    for (source, expected) in [
        ("xs = [1, 2]\n[xs.first, xs.last]", "[null,null]"),
        (
            "class Item(v): this.v = v\nItem(1)\n[Item.first, Item.last]",
            "[null,null]",
        ),
        (
            "u = null\n[u.first(missing), u.last(missing)]",
            "[null,null]",
        ),
        ("o = {}\no.items.last(missing)", "null"),
        (
            "calls = []\nn = null\nn.first(calls.push(1))\nn.last(calls.push(2))\ncalls",
            "[]",
        ),
        ("o = {a: 1}\n[o.first(), o.last()]", "[null,null]"),
        (
            "o = {first: () => 9, last: x -> x}\n[o.first(), o.last(4)]",
            "[9.0,4.0]",
        ),
        (
            "class Deck:\n    def first() => \"own first\"\n    def last(n) => n\nd = Deck()\n[d.first(), d.last(5), Deck.first() == d, Deck.last(x -> true) == d]",
            "[\"own first\",5.0,true,true]",
        ),
        (
            "class P(v):\n    this.v = v\np = P(1)\n[p.first(), p.last()]",
            "[null,null]",
        ),
        (
            "def first(xs) => 7\ndef last(xs) => \"mine\"\nxs = [1, 2]\n[first(xs), last(xs), xs.first(), xs.last()]",
            "[7.0,\"mine\",1.0,2.0]",
        ),
        (
            "class Item(v): this.v = v\n$Item.f = $Item.first()\n$Item.l = $Item.last(x -> true)\nItem(1)\nItem.map(i -> [i.f, i.l])",
            "[[null,null]]",
        ),
        (
            "class Item(v): this.v = v\nItem(1)\nItem(2)\nf = c -> c.last().v\n[f(Item), f([{v: 5}])]",
            "[2.0,5.0]",
        ),
        (
            "xs = [1]\nxs.push(2)\n[xs@first, xs@last]",
            "[[1.0],[1.0,2.0]]",
        ),
        (
            "xs = [1]\nxs.push(2)\n[xs@first.last(), xs@last.first()]",
            "[1.0,1.0]",
        ),
        (
            "[\"abc\"[0], \"abc\"[-1], \"abc\".find(\"c\")]",
            "[\"a\",\"c\",2.0]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "xs = [1]\nxs.first = 2",
            "TypeError: Cannot assign 'first' on list (line 2, column 1)",
        ),
        (
            "xs = [1]\nxs.last = 2",
            "TypeError: Cannot assign 'last' on list (line 2, column 1)",
        ),
        (
            "s = \"ab\"\ns.first = 2",
            "TypeError: Cannot assign 'first' on string (line 2, column 1)",
        ),
        (
            "o = {first: 3}\no.first()",
            "TypeError: obj:0.first is not a function (line 2, column 1)",
        ),
        (
            "\"abc\".first()",
            "TypeError: abc.first is not a function (line 1, column 1)",
        ),
        (
            "s = \"abc\"\ns.last(x -> true)",
            "TypeError: abc.last is not a function (line 2, column 1)",
        ),
        (
            "x = 5\nx.first()",
            "TypeError: number.first is not a function (line 2, column 1)",
        ),
        (
            "(5).last()",
            "TypeError: number.last is not a function (line 1, column 1)",
        ),
        (
            "b = true\nb.first()",
            "TypeError: boolean.first is not a function (line 2, column 1)",
        ),
        (
            "f = x -> x\nf.last()",
            "TypeError: function.last is not a function (line 2, column 1)",
        ),
        (
            "d = Date(0)\nd.first()",
            "TypeError: Date.first is not a function (line 2, column 1)",
        ),
        (
            "r = /a/\nr.last()",
            "TypeError: last is not a function (line 2, column 1)",
        ),
        (
            "Math.first()",
            "TypeError: Math.first is not a function (line 1, column 1)",
        ),
        (
            "String.last()",
            "TypeError: String.last is not a function (line 1, column 1)",
        ),
        (
            "Number.first()",
            "TypeError: Number.first is not a function (line 1, column 1)",
        ),
        (
            "Date.last()",
            "TypeError: Date.last is not a function (line 1, column 1)",
        ),
        (
            "class Math(v): this.v = v\nMath(1)\nMath.first()",
            "TypeError: Math.first is not a function (line 3, column 1)",
        ),
        (
            "def Math() => 1\nMath.first()",
            "TypeError: Math.first is not a function (line 2, column 1)",
        ),
        (
            "def Date() => 1\nDate.last(x -> true)",
            "TypeError: Date.last is not a function (line 2, column 1)",
        ),
        (
            "Class.first()",
            "ReferenceError: Class is not defined (line 1, column 1)",
        ),
        (
            "Object.last()",
            "ReferenceError: Object is not defined (line 1, column 1)",
        ),
        (
            "List.first()",
            "ReferenceError: List is not defined (line 1, column 1)",
        ),
        (
            "Ghost.last()",
            "ReferenceError: Ghost is not defined (line 1, column 1)",
        ),
        (
            "first()",
            "ReferenceError: first is not defined (line 1, column 1)",
        ),
        (
            "xs = [1]\nxs.first(3)",
            "TypeError: first expects a function (line 2, column 1)",
        ),
        (
            "xs = []\nxs.last(null)",
            "TypeError: last expects a function (line 2, column 1)",
        ),
        (
            "class Empty\nEmpty.first(\"a\")",
            "TypeError: first expects a function (line 2, column 1)",
        ),
        (
            "xs = [1]\nxs.find()",
            "TypeError: find expects a function (line 2, column 1)",
        ),
        (
            "xs = []\nxs.findLast(2)",
            "TypeError: findLast expects a function (line 2, column 1)",
        ),
        (
            "xs = [1]\nxs.front()",
            "TypeError: xs.front is not a function (line 2, column 1)",
        ),
        (
            "xs = [1]\nxs.back()",
            "TypeError: xs.back is not a function (line 2, column 1)",
        ),
        (
            "xs = [1]\nxs.head()",
            "TypeError: xs.head is not a function (line 2, column 1)",
        ),
        (
            "xs = [1]\nxs.firstOrNull()",
            "TypeError: xs.firstOrNull is not a function (line 2, column 1)",
        ),
        (
            "xs = [1]\nxs.getLast()",
            "TypeError: xs.getLast is not a function (line 2, column 1)",
        ),
    ] {
        assert_eq!(failure(source), expected, "{source}");
    }
}

#[test]
fn declarations_follow_their_receivers_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            ("readings = [21, 23]", "readings = [21, 23]"),
            ("n = readings.first()", "n = readings.at(0)"),
            ("m = readings.last()", "m = readings.at(-1)"),
            (
                "f = readings.first(r -> r > 22)",
                "f = readings.find(r -> r > 22)",
            ),
            (
                "g = readings.last(r -> r < 30)",
                "g = readings.findLast(r -> r < 30)",
            ),
            ("[n, m, f, g]", "[n, m, f, g]"),
            ("readings.push(25, 31)", "readings.push(25, 31)"),
            ("[n, m, f, g]", "[n, m, f, g]"),
            ("readings.pop()", "readings.pop()"),
            ("readings.shift()", "readings.shift()"),
            ("[n, m, f, g]", "[n, m, f, g]"),
            (
                "readings.sort((a, b) -> b - a)",
                "readings.sort((a, b) -> b - a)",
            ),
            ("[n, m, f, g]", "[n, m, f, g]"),
            ("readings = [1, 50]", "readings = [1, 50]"),
            ("[n, m, f, g]", "[n, m, f, g]"),
            ("readings.append(24)", "readings.append(24)"),
            ("[n, m, f, g]", "[n, m, f, g]"),
            ("readings = []", "readings = []"),
            ("[n, m, f, g]", "[n, m, f, g]"),
        ],
        &["n", "m", "f", "g"],
    );
    behaves_as_spelled(
        &[
            ("o = {items: []}", "o = {items: []}"),
            ("n = o.items.last()", "n = o.items.at(-1)"),
            (
                "f = o.items.first(x -> x > 1)",
                "f = o.items.find(x -> x > 1)",
            ),
            ("[n, f]", "[n, f]"),
            ("o.items.push(1)", "o.items.push(1)"),
            ("[n, f]", "[n, f]"),
            ("o.items = [1, 2, 3]", "o.items = [1, 2, 3]"),
            ("[n, f]", "[n, f]"),
        ],
        &["n", "f"],
    );
    behaves_as_spelled(
        &[
            (
                "class Item(v):\n    this.v = v",
                "class Item(v):\n    this.v = v",
            ),
            ("n = Item.first()", "n = Item.at(0)"),
            ("m = Item.last()", "m = Item.at(-1)"),
            ("kind = Item", "kind = Item"),
            (
                "k = kind.last(i -> i.v > 1)",
                "k = kind.findLast(i -> i.v > 1)",
            ),
            ("[n, m, k]", "[n, m, k]"),
            ("a = Item(1)", "a = Item(1)"),
            ("b = Item(2)", "b = Item(2)"),
            ("[n, m, k]", "[n, m, k]"),
            ("c = Item(3)", "c = Item(3)"),
            ("[n, m, k]", "[n, m, k]"),
            ("delete c", "delete c"),
            ("[n, m, k]", "[n, m, k]"),
            ("b.v = 0", "b.v = 0"),
            ("[n, m, k]", "[n, m, k]"),
        ],
        &["n", "m", "k"],
    );
    behaves_as_spelled(
        &[
            (
                "class Order(open):\n    this.open = open",
                "class Order(open):\n    this.open = open",
            ),
            ("tea = Order(false)", "tea = Order(false)"),
            ("cake = Order(true)", "cake = Order(true)"),
            (
                "next = Order.first(order -> order.open)",
                "next = Order.find(order -> order.open)",
            ),
            (
                "newest = Order.last(order -> order.open)",
                "newest = Order.findLast(order -> order.open)",
            ),
            (
                "[next == cake, newest == cake]",
                "[next == cake, newest == cake]",
            ),
            ("tea.open = true", "tea.open = true"),
            (
                "[next == tea, newest == cake]",
                "[next == tea, newest == cake]",
            ),
            ("cake.open = false", "cake.open = false"),
            (
                "[next == tea, newest == tea]",
                "[next == tea, newest == tea]",
            ),
            ("tea.open = false", "tea.open = false"),
            ("[next, newest]", "[next, newest]"),
        ],
        &["next", "newest"],
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "queue = []\nhead = queue.first()\nassert(head, null)\nqueue.push(1, 2)\nassert(head, 1)\nqueue.shift()\nassert(head, 2)",
    );
    assert_eq!(dependencies(&runtime, "head"), ["queue"]);
}

#[test]
fn class_level_rules_follow_each_instance_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            (
                "class Sensor(name):\n    this.name = name\n    this.readings = []",
                "class Sensor(name):\n    this.name = name\n    this.readings = []",
            ),
            (
                "$Sensor.newest = $Sensor.readings.last()",
                "$Sensor.newest = $Sensor.readings.at(-1)",
            ),
            (
                "$Sensor.oldest = $Sensor.readings.first()",
                "$Sensor.oldest = $Sensor.readings.at(0)",
            ),
            (
                "$Sensor.alarm = $Sensor.readings.first(r -> r > 40)",
                "$Sensor.alarm = $Sensor.readings.find(r -> r > 40)",
            ),
            (
                "$Sensor.calm = $Sensor.readings.last(r -> r < 20)",
                "$Sensor.calm = $Sensor.readings.findLast(r -> r < 20)",
            ),
            ("a = Sensor(\"a\")", "a = Sensor(\"a\")"),
            ("b = Sensor(\"b\")", "b = Sensor(\"b\")"),
            (
                "[a.newest, a.oldest, a.alarm, a.calm, b.newest, b.alarm]",
                "[a.newest, a.oldest, a.alarm, a.calm, b.newest, b.alarm]",
            ),
            ("a.readings.push(35, 52, 10)", "a.readings.push(35, 52, 10)"),
            ("b.readings.push(45)", "b.readings.push(45)"),
            (
                "[a.newest, a.oldest, a.alarm, a.calm, b.newest, b.alarm]",
                "[a.newest, a.oldest, a.alarm, a.calm, b.newest, b.alarm]",
            ),
            ("a.readings.shift()", "a.readings.shift()"),
            ("a.readings.shift()", "a.readings.shift()"),
            ("b.readings = [12, 41]", "b.readings = [12, 41]"),
            (
                "[a.newest, a.oldest, a.alarm, a.calm, b.newest, b.alarm, b.calm]",
                "[a.newest, a.oldest, a.alarm, a.calm, b.newest, b.alarm, b.calm]",
            ),
        ],
        &[],
    );
    behaves_as_spelled(
        &[
            (
                "class Bin(items):\n    this.items = items",
                "class Bin(items):\n    this.items = items",
            ),
            (
                "if $Bin.items.first() == \"pin\":\n    $Bin.state = \"pinned\"\nelse:\n    $Bin.state = \"other\"",
                "if $Bin.items.at(0) == \"pin\":\n    $Bin.state = \"pinned\"\nelse:\n    $Bin.state = \"other\"",
            ),
            ("a = Bin([\"pin\"])", "a = Bin([\"pin\"])"),
            ("b = Bin([])", "b = Bin([])"),
            ("[a.state, b.state]", "[a.state, b.state]"),
            ("a.items.shift()", "a.items.shift()"),
            ("b.items.push(\"pin\")", "b.items.push(\"pin\")"),
            ("[a.state, b.state]", "[a.state, b.state]"),
        ],
        &[],
    );
    behaves_as_spelled(
        &[
            (
                "class Visit(name):\n    this.name = name",
                "class Visit(name):\n    this.name = name",
            ),
            (
                "$Visit.latest = Visit.last() == $Visit",
                "$Visit.latest = Visit.at(-1) == $Visit",
            ),
            ("a = Visit(\"a\")", "a = Visit(\"a\")"),
            ("[a.latest]", "[a.latest]"),
            ("b = Visit(\"b\")", "b = Visit(\"b\")"),
            ("[a.latest, b.latest]", "[a.latest, b.latest]"),
        ],
        &[],
    );
    assert_eq!(
        result(
            "class Sensor(name):\n    this.name = name\n    this.readings = []\n$Sensor.newest = $Sensor.readings.last()\noven = Sensor(\"oven\")\noven.readings.push(3, 7)\nhot = Sensor.filter(s -> s.newest > 5).first()\n[oven.newest, hot == oven]"
        ),
        "[7.0,true]"
    );
}

#[test]
fn explanations_name_the_dependencies_of_the_accepted_spelling() {
    let setup = "class Item(v):\n    this.v = v\nItem(1)\nqueue = [1, 2]\n";
    let mut alias = Runtime::new();
    let mut spelled = Runtime::new();
    alias
        .run(&format!(
            "{setup}n = queue.first()\nl = queue.last()\nf = queue.first(x -> x > 1)\nc = Item.last()\nd = Item.last(i -> i.v > 0)"
        ))
        .unwrap();
    spelled
        .run(&format!(
            "{setup}n = queue.at(0)\nl = queue.at(-1)\nf = queue.find(x -> x > 1)\nc = Item.at(-1)\nd = Item.findLast(i -> i.v > 0)"
        ))
        .unwrap();
    for (name, rule, spelling) in [
        ("n", "\"n = queue.first()\"", "\"n = queue.at(0)\""),
        ("l", "\"l = queue.last()\"", "\"l = queue.at(-1)\""),
        (
            "f",
            "\"f = queue.first(function())\"",
            "\"f = queue.find(function())\"",
        ),
        ("c", "\"c = Item.last()\"", "\"c = Item.at(-1)\""),
        (
            "d",
            "\"d = Item.last(function())\"",
            "\"d = Item.findLast(function())\"",
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
    assert_eq!(dependencies(&alias, "n"), ["queue"]);
    assert_eq!(dependencies(&alias, "c"), ["$Item"]);
    let query = "(affects queue).map(step -> step.node)";
    assert_eq!(
        json(&mut alias, query),
        json(&mut spelled, query),
        "{query}"
    );
    alias.run("queue.push(3)\nItem(5)").unwrap();
    spelled.run("queue.push(3)\nItem(5)").unwrap();
    assert_eq!(
        json(&mut alias, "[n, l, f, c.v, d.v]"),
        "[1.0,3.0,2.0,5.0,5.0]"
    );
    assert_eq!(
        json(&mut alias, "[n, l, f, c.v, d.v]"),
        json(&mut spelled, "[n, l, f, c.v, d.v]")
    );
    for (source, rendered) in [
        ("n = xs.first()", "n=xs.first()"),
        ("l = xs.last(test, 2)", "l=xs.last(test,2)"),
        ("$Bin.e = $Bin.items.last()", "$Bin.e=$Bin.items.last()"),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), rendered, "{source}");
    }
}

#[test]
fn the_receiver_and_then_every_argument_evaluate_once_in_written_order() {
    let functions = "log = []\ndef items():\n    log.push(\"items\")\n    return [1, 2]\ndef test(label):\n    log.push(label)\n    return x -> x > 1\ndef arg(label):\n    log.push(label)\n    return 0\n";
    for (source, expected) in [
        ("[items().first(), log]", "[1.0,[\"items\"]]"),
        ("[items().last(), log]", "[2.0,[\"items\"]]"),
        (
            "[items().first(test(\"a\"), arg(\"b\")), log]",
            "[2.0,[\"items\",\"a\",\"b\"]]",
        ),
        (
            "[items().last(test(\"a\"), arg(\"b\"), arg(\"c\")), log]",
            "[2.0,[\"items\",\"a\",\"b\",\"c\"]]",
        ),
        (
            "[items().find(test(\"a\"), arg(\"b\")), log]",
            "[2.0,[\"items\",\"a\",\"b\"]]",
        ),
    ] {
        assert_eq!(
            result(&format!("{functions}{source}")),
            expected,
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    runtime.run(functions).unwrap();
    assert_eq!(
        runtime
            .run("items().first(test(\"a\"), missing, arg(\"b\"))")
            .unwrap_err()
            .to_string(),
        "ReferenceError: missing is not defined (line 1, column 1)"
    );
    assert_eq!(json(&mut runtime, "log"), "[]");
    assert_eq!(
        runtime
            .run("items().last(arg(\"a\"))")
            .unwrap_err()
            .to_string(),
        "TypeError: last expects a function (line 1, column 1)"
    );
    assert_eq!(json(&mut runtime, "log"), "[]");
    for (source, expected) in [
        (
            "class Item(v):\n    this.v = v\nItem(1)\nItem(2)\n[Item.last(x -> true, Item(3)).v, Item.last().v]",
            "[2.0,3.0]",
        ),
        (
            "class Item(v):\n    this.v = v\nItem(1)\n[Item.find(x -> x.v > 1, Item(3)), Item.first(x -> x.v > 1, Item(4)).v]",
            "[null,3.0]",
        ),
        (
            "xs = [1]\n[xs.last(x -> true, xs.push(2)), xs]",
            "[1.0,[1.0,2.0]]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for source in [
        "xs = [1]\nxs.first(missing)",
        "xs = [1]\nxs.last(x -> true, missing)",
        "class Item\nItem.first(missing)",
        "xs = [1]\nxs.find(missing)",
    ] {
        assert!(
            failure(source).starts_with("ReferenceError: missing is not defined"),
            "{source}"
        );
    }
}

#[test]
fn aliases_read_in_every_statement_position() {
    for (source, expected) in [
        (
            "r = 0\ntry:\n    r = (5).first()\ncatch error:\n    r = error\nr",
            "\"TypeError: number.first is not a function\"",
        ),
        (
            "r = 0\ntry:\n    r = [1].first(2)\ncatch error:\n    r = error\nr",
            "\"TypeError: first expects a function\"",
        ),
        (
            "jobs = [1, 2]\nnewest = jobs.last()\nseen = suppose jobs.pop(): newest\n[seen, newest, jobs]",
            "[1.0,2.0,[1.0,2.0]]",
        ),
        ("jobs = [\"a\", \"b\"]\n`${jobs.last()} last`", "\"b last\""),
        (
            "jobs = [3, 9]\nif jobs.first(j -> j > 5) == 9:\n    state = \"late\"\nelse:\n    state = \"early\"\nstate",
            "\"late\"",
        ),
        (
            "jobs = []\njobs.first() == null ? \"idle\" : \"busy\"",
            "\"idle\"",
        ),
        (
            "def ends(xs) => [xs.first(), xs.last()]\n[ends([1, 2, 3]), ends([])]",
            "[[1.0,3.0],[null,null]]",
        ),
        (
            "def newest(xs):\n    return xs.last(x -> x > 0)\n[newest([1, -2]), newest([-1])]",
            "[1.0,null]",
        ),
        (
            "lists = [[1, 2], [], [3]]\nlists.filter(xs -> xs.first() != null).map(xs -> xs.last())",
            "[2.0,3.0]",
        ),
        (
            "queue = [1, 2, 3]\ntotal = 0\nwhile queue.first() != null:\n    total = total + queue.shift()\ntotal",
            "6.0",
        ),
        (
            "class Task(title):\n    this.title = title\nTask(\"a\")\nTask(\"b\")\nTask.map(task -> task.title).last()",
            "\"b\"",
        ),
        (
            "class Shelf(items):\n    this.items = items\n    def front() => this.items.first()\n    def back() => this.items.last(x -> x > 1)\ns = Shelf([1, 2, 3])\n[s.front(), s.back()]",
            "[1.0,3.0]",
        ),
        (
            "class Item(v):\n    this.v = v\nItem(4)\nItem(6)\nnewest = Item.last()\n[newest.v, Item.first(i -> i.v > 5).v]",
            "[6.0,6.0]",
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
        "stock = [\"bolt\", \"nut\"]\nnewest = stock.last()\nfailure = \"\"\ntry:\n    stock = 7\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.last is not a function\"))\nassert([stock, newest], [[\"bolt\", \"nut\"], \"nut\"])",
    );
    checked(
        &mut runtime,
        "class Sensor(name):\n    this.name = name\n    this.readings = [18]\n$Sensor.latest = $Sensor.readings.last()\ndoor = Sensor(\"door\")\ntry:\n    door.readings = 0\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.last is not a function\"))\nassert([door.readings, door.latest], [[18], 18])",
    );
    checked(
        &mut runtime,
        "first = [5]\nopen = first.first(x -> x > 1)\ntry:\n    first = \"text\"\ncatch error:\n    failure = error\nassert([failure, first, open], [TypeError(\"text.first is not a function\"), [5], 5])",
    );
    checked(
        &mut runtime,
        "try:\n    stock.push(\"gear\")\n    door.readings.push(30)\n    assert([newest, door.latest], [\"gear\", 30])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([stock, newest, door.readings, door.latest], [[\"bolt\", \"nut\"], \"nut\", [18], 18])",
    );
    assert_eq!(dependencies(&runtime, "newest"), ["stock"]);
    assert_eq!(
        failure("stock = [1]\nnewest = stock.last()\nstock = 7"),
        "TypeError: number.last is not a function (line 3, column 1)"
    );
    let mut recovered = Runtime::new();
    recovered.run("stock = [1]\nhead = stock.first()").unwrap();
    assert!(recovered.run("stock = 5").is_err());
    assert_eq!(json(&mut recovered, "[stock, head]"), "[[1.0],1.0]");
    recovered.run("stock.shift()").unwrap();
    assert_eq!(json(&mut recovered, "head"), "null");
    recovered.run("stock.push(8)").unwrap();
    assert_eq!(json(&mut recovered, "head"), "8.0");
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    assert_eq!(
        result("xs = [1]\ntry:\n    r = xs.first()\ncatch e:\n    r = \"caught\"\nr"),
        "1.0"
    );
    assert_eq!(
        result(
            "class Item(v):\n    this.v = v\nItem(2)\ntry:\n    r = Item.last(i -> true).v\ncatch e:\n    r = \"caught\"\nr"
        ),
        "2.0"
    );
    assert_eq!(
        result("try:\n    r = \"ab\".first()\ncatch e:\n    r = \"caught\"\nr"),
        "\"caught\""
    );
}

#[test]
fn large_receivers_read_their_ends_through_the_public_api() {
    let mut runtime = Runtime::new();
    runtime.run("holder = {items: 0}").unwrap();
    let count = 100_000_u32;
    let Some(Value::Object(id)) = runtime.state.variable("holder").cloned() else {
        panic!("holder is an object");
    };
    runtime.state.object_mut(&id).unwrap().properties.insert(
        "items".to_string(),
        Value::List(
            (0..count)
                .map(|index| Value::Number(f64::from(index)))
                .collect(),
        ),
    );
    assert_eq!(
        json(
            &mut runtime,
            "[holder.items.first(), holder.items.last(), holder.items.at(0), holder.items.at(-1)]"
        ),
        "[0.0,99999.0,0.0,99999.0]"
    );
    assert_eq!(
        json(
            &mut runtime,
            "[holder.items.last(x -> x < 99990), holder.items.first(x -> x > 9)]"
        ),
        "[99989.0,10.0]"
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-first-and-last-methods-{label}-{}.nuc",
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
    let program = "class Item(v):\n    this.v = v\nreadings = [21, 23]\noldest = readings.first()\nnewest = readings.last()\nhot = readings.first(r -> r > 22)\nItem(1)\nItem(2)\nlatest = Item.last()\nreadings.push(25)\n[oldest, newest, hot, latest.v]";
    let spelled = "class Item(v):\n    this.v = v\nreadings = [21, 23]\noldest = readings.at(0)\nnewest = readings.at(-1)\nhot = readings.find(r -> r > 22)\nItem(1)\nItem(2)\nlatest = Item.at(-1)\nreadings.push(25)\n[oldest, newest, hot, latest.v]";
    let file = SourceFile::new("file");
    for options in [
        &[][..],
        &["--json"][..],
        &["--graph"][..],
        &["--json", "--graph"][..],
    ] {
        let alias = file.run(program, options);
        let written = file.run(spelled, options);
        assert!(alias.status.success(), "{options:?} {alias:?}");
        assert!(alias.stderr.is_empty(), "{options:?} {alias:?}");
        assert_eq!(alias.stdout, written.stdout, "{options:?}");
    }
    assert_eq!(
        String::from_utf8(file.run(program, &["--json"]).stdout).unwrap(),
        "[21.0,25.0,23.0,2.0]\n"
    );
    let graph = String::from_utf8(file.run(program, &["--graph"]).stdout).unwrap();
    assert!(
        graph.contains("newest [variable]\n    reads    readings"),
        "{graph}"
    );
    let failed = file.run("x = 5\nx.last()\n", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "TypeError: number.last is not a function (line 2, column 1)\n"
    );
    let session = "queue = [\"a\"]\nhead = queue.first()\nhead\nqueue.push(\"b\")\nqueue.last()\nqueue.first(x -> x == \"b\")\n[].first()\n.exit\n";
    let spelled_session = "queue = [\"a\"]\nhead = queue.at(0)\nhead\nqueue.push(\"b\")\nqueue.at(-1)\nqueue.find(x -> x == \"b\")\n[].at(0)\n.exit\n";
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let alias = repl(&session.replace('\n', newline), options);
            let written = repl(&spelled_session.replace('\n', newline), options);
            assert!(alias.status.success(), "{options:?} {alias:?}");
            assert_eq!(alias.stdout, written.stdout, "{options:?}");
            assert_eq!(alias.stderr, written.stderr, "{options:?}");
            let stdout = String::from_utf8(alias.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "\"a\"\n2.0\n\"b\"\n\"b\"\nnull\n"
            } else {
                "a\n2\nb\nb\nundefined\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
        }
    }
    let failing = repl("(5).first()\n[1, 2].last()\n.exit\n", &["--plain"]);
    assert!(failing.status.success(), "{failing:?}");
    assert_eq!(
        String::from_utf8(failing.stderr).unwrap(),
        "TypeError: number.first is not a function (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(failing.stdout).unwrap();
    assert!(stdout.contains("2\n"), "{stdout}");
}
