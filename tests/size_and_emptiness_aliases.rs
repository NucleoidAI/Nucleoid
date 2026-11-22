mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime, Value};

const SPEC_TITLES: [&str; 3] = [
    "Nucleoid sizes lists, populations and strings with size and len",
    "Nucleoid tests emptiness with isEmpty, is_empty and isNotEmpty",
    "Nucleoid rolls back failed updates through size and emptiness aliases",
];
const REFERENCE_TITLE: &str = "Nucleoid spells size, len and isEmpty as Java, Kotlin and Rust do";
const README_TITLE: &str = "Size and emptiness methods keep an order queue's rules live";
const EXAMPLES_TITLE: &str = "Emptiness methods keep every shelf's restock count live";

const ALIASES: [&str; 5] = ["size", "len", "isEmpty", "is_empty", "isNotEmpty"];
const NAMES: [&str; 6] = ["size", "len", "length", "isEmpty", "is_empty", "isNotEmpty"];

const SETUP: &str = "class Item(v):\n    this.v = v\nItem(1)\nItem(2)\nItem(3)\nclass Empty\nclass Box:\n    def size() => 99\n    def isEmpty() => \"own\"\nBox()";

const RECEIVERS: [(&str, &str); 37] = [
    ("", "[1, 2, 3]"),
    ("", "[]"),
    ("xs = [1, 2]", "xs"),
    ("o = {items: [1]}", "o.items"),
    ("def make() => [1, 2, 3, 4]", "make()"),
    ("xs = [1, 2, 3]", "xs[1:]"),
    ("xs = [1, 2]", "xs.value"),
    ("xs = [1]\nxs.push(2)", "xs@first"),
    ("grid = [[1, 2], [3]]", "grid[0]"),
    ("xs = [1, 2, 3]", "xs.filter(x -> x > 1)"),
    ("xs = [1, 2]", "xs.map(x -> x * 2)"),
    ("xs = [1]", "xs.concat([2, 3])"),
    ("", "\"a,b,c\".split(\",\")"),
    ("", "[null, [], \"\"]"),
    ("", "Item.list()"),
    ("", "Item[1:]"),
    ("", "Item.filter(item -> item.v > 1)"),
    ("", "Item"),
    ("kind = Item", "kind"),
    ("", "$Item"),
    ("", "Empty"),
    ("", "Box"),
    ("class List(v):\n    this.v = v\nList(1)", "List"),
    (
        "class Object(v):\n    this.v = v\nObject(1)\nObject(2)",
        "Object",
    ),
    ("", "\"abc\""),
    ("", "\"\""),
    ("s = \"hello\"", "s"),
    ("o = {name: \"Ada\"}", "o.name"),
    ("def word() => \"word\"", "word()"),
    ("stem = \"ab\"", "`${stem}c`"),
    ("s = \"ab\"", "s.value"),
    ("s = \"ab\"\ns = \"abcd\"", "s@first"),
    ("xs = [\"x\", \"yz\"]", "xs[1]"),
    ("", "\"ABC\".toLowerCase()"),
    ("", "\"Zoë\""),
    ("", "\"日本語\""),
    ("flag = true", "(flag ? \"yes\" : \"no\")"),
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
    (
        "class Math(v):\n    this.v = v\nMath(1)\nkind = Math",
        "kind",
    ),
    ("", "Ghost"),
];

const ARGUMENTS: [&str; 4] = ["", "1", "1, \"two\", null", "missing"];

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

fn calls_alias(source: &str) -> bool {
    let tokens: Vec<Token> = tokenize(source)
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    let alias = tokens.windows(3).any(|window| {
        window[0] == Token::Dot
            && matches!(&window[1], Token::Identifier(name) if ALIASES.contains(&name.as_str()))
            && window[2] == Token::LeftParen
    });
    let text_length = tokens.windows(4).any(|window| {
        matches!(window[0], Token::Str(_) | Token::Template(_))
            && window[1] == Token::Dot
            && window[2] == Token::Identifier("length".to_string())
            && window[3] == Token::LeftParen
    });
    alias || text_length
}

fn spelling(name: &str, receiver: &str) -> String {
    match name {
        "size" | "len" | "length" => format!("({receiver}).length"),
        "isEmpty" | "is_empty" => format!("({receiver}).length == 0"),
        "isNotEmpty" => format!("({receiver}).length > 0"),
        other => panic!("{other} is not a size name"),
    }
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
        Err(error) => error.to_string(),
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
            title: "size and emptiness alias regression".to_string(),
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
    assert_eq!(assertions, 27);
    assert_eq!(cases, 1276);
}

#[test]
fn the_audit_finds_every_alias_spelling_and_nothing_else() {
    for source in [
        "xs.size()",
        "xs.len(1)",
        "xs.isEmpty()",
        "xs.is_empty()",
        "xs.isNotEmpty()",
        "Item.size()",
        "f(x).len()",
        "\"abc\".length()",
        "`${s}`.length()",
        "$Bin.items.isEmpty()",
    ] {
        assert!(calls_alias(source), "{source}");
    }
    for source in [
        "xs.length",
        "xs.length()",
        "s.length()",
        "xs.size",
        "s.isEmpty",
        "size(xs)",
        "isEmpty(xs)",
        "o = {size: 1, len: 2}",
        "\"size\"",
        "size = xs.length",
        "\"abc\".length",
    ] {
        assert!(!calls_alias(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_115_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[780],
        "Nucleoid nests repeat loops and rolls back a failed iteration"
    );
    assert_eq!(titles[781..784], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid repeats statements a counted number of times",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Repeat loops advance a world model a counted number of steps",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Repeat loops build the cars of every train",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(index, cases.len() - 20, "{title}");
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 15].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Size and emptiness method aliases\n\n```\n")
        .expect("the language reference documents the size and emptiness aliases");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 116](nuc-0116.md)"));
    assert!(reference.contains(
        "items.isEmpty()  items.is_empty()      exactly items.length == 0, on lists, populations and strings (NUC 116)"
    ));
    assert_eq!(case.expected.as_deref(), Some("true"));
}

#[test]
fn each_alias_gives_its_accepted_spellings_result_on_every_receiver() {
    let mut answers = [0_usize; 3];
    for (setup, receiver) in RECEIVERS {
        for name in NAMES {
            for arguments in &ARGUMENTS[..3] {
                let alias = format!("{SETUP}\n{setup}\n({receiver}).{name}({arguments})");
                let spelled = format!("{SETUP}\n{setup}\n{}", spelling(name, receiver));
                let found = fresh(&alias);
                assert_eq!(found, fresh(&spelled), "{alias}");
                match found.as_str() {
                    "true" => answers[0] += 1,
                    "false" => answers[1] += 1,
                    number => {
                        assert!(number.parse::<f64>().is_ok(), "{alias}: {number}");
                        answers[2] += 1;
                    }
                }
            }
        }
    }
    assert_eq!(
        answers.iter().sum::<usize>(),
        RECEIVERS.len() * NAMES.len() * 3
    );
    assert!(answers.iter().all(|count| *count > 50), "{answers:?}");
    for (source, expected) in [
        ("[1, 2, 3].size()", "3.0"),
        ("[].len()", "0.0"),
        ("[[]].isEmpty()", "false"),
        ("[].is_empty()", "true"),
        ("[0].isNotEmpty()", "true"),
        ("\"abc\".size()", "3.0"),
        ("\"Zoë\".len()", "3.0"),
        ("\"日本語\".length()", "3.0"),
        ("\"\".isEmpty()", "true"),
        ("\" \".is_empty()", "false"),
        ("\"\".isNotEmpty()", "false"),
        ("class Item\nItem.size()", "0.0"),
        (
            "class Item\n[Item.isEmpty(), Item.isNotEmpty()]",
            "[true,false]",
        ),
        ("xs = [1, 2]\nxs.size(9, 8)", "2.0"),
        ("xs = [1, 2]\n[xs.size(), xs.size] ", "[2.0,null]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn arguments_are_evaluated_and_their_errors_propagate_as_for_length() {
    let mut checked_programs = 0;
    for (setup, receiver) in RECEIVERS {
        for name in NAMES {
            let alias = format!("{SETUP}\n{setup}\n({receiver}).{name}(missing)");
            let spelled = format!("{SETUP}\n{setup}\n({receiver}).length(missing)");
            let found = fresh(&alias);
            assert_eq!(found, fresh(&spelled), "{alias}");
            assert!(
                found.starts_with("ReferenceError: missing is not defined"),
                "{alias}: {found}"
            );
            checked_programs += 1;
        }
    }
    assert_eq!(checked_programs, RECEIVERS.len() * NAMES.len());
}

#[test]
fn other_receivers_raise_or_read_exactly_as_their_length_call_does() {
    let mut kept = 0;
    for (setup, receiver) in OTHERS {
        for name in ALIASES {
            for arguments in ARGUMENTS {
                let alias = format!("{setup}\n({receiver}).{name}({arguments})");
                let spelled = format!("{setup}\n({receiver}).length({arguments})");
                let expected = fresh(&spelled).replace(
                    "length is not a function",
                    &format!("{name} is not a function"),
                );
                assert_eq!(fresh(&alias), expected, "{alias}");
                kept += 1;
            }
        }
    }
    assert_eq!(kept, OTHERS.len() * ALIASES.len() * ARGUMENTS.len());
}

#[test]
fn the_kept_readings_keep_their_exact_accepted_results() {
    for (source, expected) in [
        ("xs = [1, 2]\nxs.size", "null"),
        ("s = \"ab\"\n[s.isEmpty, s.len, s.size]", "[null,null,null]"),
        (
            "class Item(v): this.v = v\nItem(1)\n[Item.size, Item.isEmpty]",
            "[null,null]",
        ),
        ("u = null\nu.size(missing)", "null"),
        ("o = {}\no.items.isEmpty(missing)", "null"),
        (
            "calls = []\nn = null\nn.size(calls.push(1))\nn.isEmpty(calls.push(2))\ncalls",
            "[]",
        ),
        (
            "o = {a: 1}\n[o.size(), o.isEmpty(), o.len()]",
            "[null,null,null]",
        ),
        (
            "o = {size: () => 9, isEmpty: () => \"own\", len: x -> x}\n[o.size(), o.isEmpty(), o.len(4)]",
            "[9.0,\"own\",4.0]",
        ),
        (
            "class Stack:\n    def isEmpty() => \"own\"\n    def size() => 7\ns = Stack()\n[s.isEmpty(), s.size(), Stack.size(), Stack.isEmpty()]",
            "[\"own\",7.0,1.0,false]",
        ),
        (
            "class Box:\n    def length() => 7\nb = Box()\n[b.length(), Box.length(), b.size(), Box.size()]",
            "[7.0,1.0,null,1.0]",
        ),
        (
            "class P(v):\n    this.v = v\np = P(1)\n[p.size(), p.isEmpty(), p.len()]",
            "[null,null,null]",
        ),
        (
            "def size(xs) => 7\ndef isEmpty(xs) => \"mine\"\nxs = [1]\n[size(xs), isEmpty(xs), xs.size(), xs.isEmpty()]",
            "[7.0,\"mine\",1.0,false]",
        ),
        (
            "class Item(v): this.v = v\n$Item.n = $Item.size()\n$Item.e = $Item.isEmpty()\nItem(1)\nItem.map(i -> [i.n, i.e])",
            "[[null,null]]",
        ),
        (
            "class Item(v): this.v = v\nItem(1)\nf = c -> c.size()\n[f(Item), f([1, 2]), f(\"abc\")]",
            "[1.0,2.0,3.0]",
        ),
        ("class Math(v): this.v = v\nMath(1)\n[Math.length]", "[1.0]"),
        ("[\"日本\".length, \"Zoë\".length]", "[2.0,3.0]"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "xs = [1]\nxs.size = 2",
            "TypeError: Cannot assign 'size' on list (line 2, column 1)",
        ),
        (
            "xs = [1]\nxs.isEmpty = true",
            "TypeError: Cannot assign 'isEmpty' on list (line 2, column 1)",
        ),
        (
            "s = \"ab\"\ns.size = 2",
            "TypeError: Cannot assign 'size' on string (line 2, column 1)",
        ),
        (
            "o = {size: 3}\no.size()",
            "TypeError: obj:0.size is not a function (line 2, column 1)",
        ),
        (
            "x = 5\nx.len()",
            "TypeError: number.len is not a function (line 2, column 1)",
        ),
        (
            "(5).size()",
            "TypeError: number.size is not a function (line 1, column 1)",
        ),
        (
            "b = true\nb.isEmpty()",
            "TypeError: boolean.isEmpty is not a function (line 2, column 1)",
        ),
        (
            "f = x -> x\nf.size()",
            "TypeError: function.size is not a function (line 2, column 1)",
        ),
        (
            "d = Date(0)\nd.isNotEmpty()",
            "TypeError: Date.isNotEmpty is not a function (line 2, column 1)",
        ),
        (
            "r = /a/\nr.size()",
            "TypeError: size is not a function (line 2, column 1)",
        ),
        (
            "Math.size()",
            "TypeError: Math.size is not a function (line 1, column 1)",
        ),
        (
            "String.size()",
            "TypeError: String.size is not a function (line 1, column 1)",
        ),
        (
            "Number.isEmpty()",
            "TypeError: Number.isEmpty is not a function (line 1, column 1)",
        ),
        (
            "Date.len()",
            "TypeError: Date.len is not a function (line 1, column 1)",
        ),
        (
            "class Math(v): this.v = v\nMath(1)\nMath.size()",
            "TypeError: Math.size is not a function (line 3, column 1)",
        ),
        (
            "Class.size()",
            "ReferenceError: Class is not defined (line 1, column 1)",
        ),
        (
            "Object.isEmpty()",
            "ReferenceError: Object is not defined (line 1, column 1)",
        ),
        (
            "List.len()",
            "ReferenceError: List is not defined (line 1, column 1)",
        ),
        (
            "Ghost.size()",
            "ReferenceError: Ghost is not defined (line 1, column 1)",
        ),
        (
            "size()",
            "ReferenceError: size is not defined (line 1, column 1)",
        ),
        (
            "s = \"abc\"\nsum of s",
            "TypeError: abc.sum is not a function (line 2, column 1)",
        ),
        (
            "xs = [1]\nxs.count()",
            "TypeError: count expects a function (line 2, column 1)",
        ),
        (
            "xs = [1]\nxs.empty()",
            "TypeError: xs.empty is not a function (line 2, column 1)",
        ),
        (
            "\"ab\".count()",
            "TypeError: ab.count is not a function (line 1, column 1)",
        ),
    ] {
        assert_eq!(failure(source), expected, "{source}");
    }
}

#[test]
fn the_public_string_method_takes_the_size_and_emptiness_names() {
    let size = |receiver: &str, name: &str, arguments: &[Value]| {
        nucleoid::builtins::string_method(receiver, name, arguments).unwrap()
    };
    for name in ["size", "len", "length"] {
        assert_eq!(size("Zoë", name, &[]), Value::Number(3.0), "{name}");
        assert_eq!(size("", name, &[]), Value::Number(0.0), "{name}");
        assert_eq!(
            size("ab", name, &[Value::Number(9.0), Value::Null]),
            Value::Number(2.0),
            "{name}"
        );
    }
    for name in ["isEmpty", "is_empty"] {
        assert_eq!(size("", name, &[]), Value::Bool(true), "{name}");
        assert_eq!(size(" ", name, &[]), Value::Bool(false), "{name}");
    }
    assert_eq!(size("", "isNotEmpty", &[]), Value::Bool(false));
    assert_eq!(size("日", "isNotEmpty", &[]), Value::Bool(true));
    for name in ["count", "empty", "Size", "is_not_empty", "isBlank"] {
        let error = nucleoid::builtins::string_method("ab", name, &[])
            .expect_err("other names keep the accepted error");
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), format!("ab.{name} is not a function"));
    }
}

#[test]
fn declarations_follow_their_receivers_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            ("jobs = [\"a\"]", "jobs = [\"a\"]"),
            ("n = jobs.size()", "n = jobs.length"),
            ("m = jobs.len()", "m = jobs.length"),
            ("e = jobs.isEmpty()", "e = jobs.length == 0"),
            ("f = jobs.is_empty()", "f = jobs.length == 0"),
            ("g = jobs.isNotEmpty()", "g = jobs.length > 0"),
            ("[n, m, e, f, g]", "[n, m, e, f, g]"),
            ("jobs.push(\"b\", \"c\")", "jobs.push(\"b\", \"c\")"),
            ("[n, m, e, f, g]", "[n, m, e, f, g]"),
            ("jobs.pop()", "jobs.pop()"),
            ("jobs.shift()", "jobs.shift()"),
            ("jobs.shift()", "jobs.shift()"),
            ("[n, m, e, f, g]", "[n, m, e, f, g]"),
            ("jobs = [1, 2, 3, 4]", "jobs = [1, 2, 3, 4]"),
            ("[n, m, e, f, g]", "[n, m, e, f, g]"),
            ("jobs.append(5)", "jobs.append(5)"),
            ("jobs.sort()", "jobs.sort()"),
            ("[n, m, e, f, g]", "[n, m, e, f, g]"),
        ],
        &["n", "m", "e", "f", "g"],
    );
    behaves_as_spelled(
        &[
            ("s = \"ab\"", "s = \"ab\""),
            ("n = s.size()", "n = s.length"),
            ("t = s.length()", "t = s.length"),
            ("e = s.isEmpty()", "e = s.length == 0"),
            ("g = s.isNotEmpty()", "g = s.length > 0"),
            ("[n, t, e, g]", "[n, t, e, g]"),
            ("s = \"\"", "s = \"\""),
            ("[n, t, e, g]", "[n, t, e, g]"),
            ("s = \"Zoë\"", "s = \"Zoë\""),
            ("[n, t, e, g]", "[n, t, e, g]"),
        ],
        &["n", "t", "e", "g"],
    );
    behaves_as_spelled(
        &[
            ("o = {items: []}", "o = {items: []}"),
            ("n = o.items.size()", "n = o.items.length"),
            ("e = o.items.is_empty()", "e = o.items.length == 0"),
            ("[n, e]", "[n, e]"),
            ("o.items.push(1)", "o.items.push(1)"),
            ("[n, e]", "[n, e]"),
            ("o.items = [1, 2, 3]", "o.items = [1, 2, 3]"),
            ("[n, e]", "[n, e]"),
        ],
        &["n", "e"],
    );
    behaves_as_spelled(
        &[
            (
                "class Item(v):\n    this.v = v",
                "class Item(v):\n    this.v = v",
            ),
            ("n = Item.size()", "n = Item.length"),
            ("e = Item.isEmpty()", "e = Item.length == 0"),
            ("kind = Item", "kind = Item"),
            ("k = kind.len()", "k = kind.length"),
            ("[n, e, k]", "[n, e, k]"),
            ("a = Item(1)", "a = Item(1)"),
            ("b = Item(2)", "b = Item(2)"),
            ("[n, e, k]", "[n, e, k]"),
            ("delete a", "delete a"),
            ("[n, e, k]", "[n, e, k]"),
            ("delete b", "delete b"),
            ("[n, e, k]", "[n, e, k]"),
        ],
        &["n", "e", "k"],
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "queue = []\nwaiting = queue.isEmpty()\nassert(waiting, true)\nqueue.push(1)\nassert(waiting, false)\nqueue.pop()\nassert(waiting, true)",
    );
    assert_eq!(dependencies(&runtime, "waiting"), ["queue"]);
}

#[test]
fn class_level_rules_follow_each_instance_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            (
                "class Bin(items):\n    this.items = items",
                "class Bin(items):\n    this.items = items",
            ),
            (
                "$Bin.empty = $Bin.items.isEmpty()",
                "$Bin.empty = $Bin.items.length == 0",
            ),
            (
                "$Bin.count = $Bin.items.size()",
                "$Bin.count = $Bin.items.length",
            ),
            (
                "$Bin.full = $Bin.items.isNotEmpty()",
                "$Bin.full = $Bin.items.length > 0",
            ),
            ("a = Bin([])", "a = Bin([])"),
            ("b = Bin([1])", "b = Bin([1])"),
            (
                "[a.empty, a.count, a.full, b.empty, b.count, b.full]",
                "[a.empty, a.count, a.full, b.empty, b.count, b.full]",
            ),
            ("a.items.push(1, 2)", "a.items.push(1, 2)"),
            ("b.items.pop()", "b.items.pop()"),
            (
                "[a.empty, a.count, a.full, b.empty, b.count, b.full]",
                "[a.empty, a.count, a.full, b.empty, b.count, b.full]",
            ),
            ("a.items = []", "a.items = []"),
            (
                "restock = Bin.filter(bin -> bin.empty).size()",
                "restock = Bin.filter(bin -> bin.empty).length",
            ),
            ("Bin([])", "Bin([])"),
            ("restock", "restock"),
            ("b.items.push(3)", "b.items.push(3)"),
            ("restock", "restock"),
        ],
        &["restock"],
    );
    behaves_as_spelled(
        &[
            (
                "class Tag(name):\n    this.name = name",
                "class Tag(name):\n    this.name = name",
            ),
            (
                "$Tag.blank = $Tag.name.is_empty()",
                "$Tag.blank = $Tag.name.length == 0",
            ),
            (
                "$Tag.width = $Tag.name.len()",
                "$Tag.width = $Tag.name.length",
            ),
            ("t = Tag(\"\")", "t = Tag(\"\")"),
            ("[t.blank, t.width]", "[t.blank, t.width]"),
            ("t.name = \"Zoë\"", "t.name = \"Zoë\""),
            ("[t.blank, t.width]", "[t.blank, t.width]"),
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
                "if $Bin.items.isEmpty():\n    $Bin.state = \"empty\"\nelse:\n    $Bin.state = \"stocked\"",
                "if $Bin.items.length == 0:\n    $Bin.state = \"empty\"\nelse:\n    $Bin.state = \"stocked\"",
            ),
            ("a = Bin([])", "a = Bin([])"),
            ("b = Bin([1])", "b = Bin([1])"),
            ("[a.state, b.state]", "[a.state, b.state]"),
            ("a.items.push(1)", "a.items.push(1)"),
            ("b.items.pop()", "b.items.pop()"),
            ("[a.state, b.state]", "[a.state, b.state]"),
        ],
        &[],
    );
    assert_eq!(
        result(
            "class Shelf(items):\n    this.items = items\n$Shelf.share = $Shelf.items.size() / Shelf.size()\na = Shelf([1, 2])\nb = Shelf([3])\nShelf([])\n[a.share, b.share]"
        ),
        "[0.6666666666666666,0.3333333333333333]"
    );
}

#[test]
fn explanations_name_the_dependencies_of_the_accepted_spelling() {
    let setup = "class Item(v):\n    this.v = v\nItem(1)\nqueue = [1]\ntext = \"ab\"\n";
    let mut alias = Runtime::new();
    let mut spelled = Runtime::new();
    alias
        .run(&format!(
            "{setup}n = queue.size()\ne = queue.isEmpty()\nc = Item.len()\nw = text.length()"
        ))
        .unwrap();
    spelled
        .run(&format!(
            "{setup}n = queue.length\ne = queue.length == 0\nc = Item.length\nw = text.length"
        ))
        .unwrap();
    assert_eq!(json(&mut alias, "(why n)[0].rule"), "\"n = queue.size()\"");
    assert_eq!(
        json(&mut alias, "(why e)[0].rule"),
        "\"e = queue.isEmpty()\""
    );
    assert_eq!(json(&mut alias, "(why c)[0].rule"), "\"c = Item.len()\"");
    assert_eq!(json(&mut alias, "(why w)[0].rule"), "\"w = text.length()\"");
    for name in ["n", "e", "c", "w"] {
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
    assert_eq!(dependencies(&alias, "w"), ["text"]);
    for root in ["queue", "text"] {
        let query = format!("(affects {root}).map(step -> step.node)");
        assert_eq!(
            json(&mut alias, &query),
            json(&mut spelled, &query),
            "{query}"
        );
    }
    alias.run("queue.push(2)").unwrap();
    spelled.run("queue.push(2)").unwrap();
    assert_eq!(json(&mut alias, "[n, e, c, w]"), "[2.0,false,1.0,2.0]");
    assert_eq!(
        json(&mut alias, "[n, e, c, w]"),
        json(&mut spelled, "[n, e, c, w]")
    );
    for (source, rendered) in [
        ("n = xs.size()", "n=xs.size()"),
        ("e = xs.is_empty()", "e=xs.is_empty()"),
        ("t = \"ab\".length()", "t=\"ab\".length()"),
        (
            "$Bin.e = $Bin.items.isNotEmpty()",
            "$Bin.e=$Bin.items.isNotEmpty()",
        ),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), rendered, "{source}");
    }
}

#[test]
fn the_receiver_and_then_every_argument_evaluate_once_in_written_order() {
    let functions = "log = []\ndef items():\n    log.push(\"items\")\n    return [1, 2]\ndef text():\n    log.push(\"text\")\n    return \"abc\"\ndef arg(label):\n    log.push(label)\n    return 0\n";
    for (source, expected) in [
        (
            "[items().size(arg(\"a\"), arg(\"b\")), log]",
            "[2.0,[\"items\",\"a\",\"b\"]]",
        ),
        (
            "[items().isEmpty(arg(\"a\")), log]",
            "[false,[\"items\",\"a\"]]",
        ),
        (
            "[text().len(arg(\"a\"), arg(\"b\")), log]",
            "[3.0,[\"text\",\"a\",\"b\"]]",
        ),
        ("[text().length(arg(\"x\")), log]", "[3.0,[\"text\",\"x\"]]"),
        (
            "[items().length(arg(\"x\")), log]",
            "[2.0,[\"items\",\"x\"]]",
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
            .run("items().size(arg(\"a\"), missing, arg(\"b\"))")
            .unwrap_err()
            .to_string(),
        "ReferenceError: missing is not defined (line 1, column 1)"
    );
    assert_eq!(json(&mut runtime, "log"), "[]");
    for source in [
        "xs = [1]\nxs.size(missing)",
        "\"ab\".isEmpty(missing)",
        "class Item\nItem.len(missing)",
        "xs = [1]\nxs.length(missing)",
        "\"ab\".length(missing)",
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
            "r = 0\ntry:\n    r = (5).size()\ncatch error:\n    r = error\nr",
            "\"TypeError: number.size is not a function\"",
        ),
        (
            "jobs = [1]\nidle = jobs.isEmpty()\nseen = suppose jobs.pop(): idle\n[seen, idle, jobs]",
            "[true,false,[1.0]]",
        ),
        ("jobs = [1, 2]\n`${jobs.size()} jobs`", "\"2 jobs\""),
        ("name = \"Zoë\"\n`${name.length()}`", "\"3\""),
        (
            "jobs = []\nif jobs.isEmpty():\n    state = \"idle\"\nelse:\n    state = \"busy\"\nstate",
            "\"idle\"",
        ),
        (
            "jobs = [1]\njobs.isNotEmpty() ? \"busy\" : \"idle\"",
            "\"busy\"",
        ),
        (
            "def total(xs) => xs.size() + xs.len()\n[total([1, 2]), total(\"abc\")]",
            "[4.0,6.0]",
        ),
        (
            "def blank(s):\n    return s.is_empty()\n[blank(\"\"), blank(\"a\")]",
            "[true,false]",
        ),
        (
            "lists = [[1], [], [2, 3]]\nlists.filter(xs -> xs.isNotEmpty()).map(xs -> xs.len())",
            "[1.0,2.0]",
        ),
        (
            "words = [\"a\", \"\", \"bc\"]\nwords.some(w -> w.isEmpty()) && words.every(w -> w.size() < 3)",
            "true",
        ),
        (
            "queue = [1, 2, 3]\nwhile queue.isNotEmpty():\n    queue.pop()\nqueue.size()",
            "0.0",
        ),
        (
            "class Task(title):\n    this.title = title\nTask(\"a\")\nTask(\"b\")\nTask.map(task -> task.title.size()).reduce((sum, n) => sum + n, 0)",
            "2.0",
        ),
        (
            "class Shelf(items):\n    this.items = items\n    def count() => this.items.size()\n    def empty() => this.items.is_empty()\ns = Shelf([1, 2])\n[s.count(), s.empty()]",
            "[2.0,false]",
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
        "stock = [\"bolt\", \"nut\"]\nlow = stock.size() < 3\nfailure = \"\"\ntry:\n    stock = 7\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.size is not a function\"))\nassert([stock, low], [[\"bolt\", \"nut\"], true])",
    );
    checked(
        &mut runtime,
        "class Bin(items):\n    this.items = items\n$Bin.empty = $Bin.items.isEmpty()\nbin = Bin([\"pin\"])\ntry:\n    bin.items = 0\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.isEmpty is not a function\"))\nassert([bin.items, bin.empty], [[\"pin\"], false])",
    );
    checked(
        &mut runtime,
        "label = \"ok\"\nshown = label.length()\ntry:\n    label = true\ncatch error:\n    failure = error\nassert([failure, label, shown], [TypeError(\"boolean.length is not a function\"), \"ok\", 2])",
    );
    checked(
        &mut runtime,
        "try:\n    stock.push(\"gear\")\n    bin.items.pop()\n    assert([low, bin.empty], [false, true])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([stock, low, bin.items, bin.empty], [[\"bolt\", \"nut\"], true, [\"pin\"], false])",
    );
    assert_eq!(dependencies(&runtime, "low"), ["stock"]);
    assert_eq!(
        failure("stock = [1]\nlow = stock.size() < 3\nstock = 7"),
        "TypeError: number.size is not a function (line 3, column 1)"
    );
    let mut recovered = Runtime::new();
    recovered.run("stock = [1]\nlow = stock.isEmpty()").unwrap();
    assert!(recovered.run("stock = 5").is_err());
    assert_eq!(json(&mut recovered, "[stock, low]"), "[[1.0],false]");
    recovered.run("stock.pop()").unwrap();
    assert_eq!(json(&mut recovered, "low"), "true");
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    assert_eq!(
        result("xs = [1]\ntry:\n    r = xs.size()\ncatch e:\n    r = \"caught\"\nr"),
        "1.0"
    );
    assert_eq!(
        result("try:\n    r = \"ab\".length()\ncatch e:\n    r = \"caught\"\nr"),
        "2.0"
    );
    assert_eq!(
        result("try:\n    r = (5).isEmpty()\ncatch e:\n    r = \"caught\"\nr"),
        "\"caught\""
    );
}

#[test]
fn large_receivers_count_through_the_public_api() {
    let mut runtime = Runtime::new();
    runtime.run("holder = {items: 0}").unwrap();
    let count = 100_000_u32;
    let Some(Value::Object(id)) = runtime.state.variable("holder").cloned() else {
        panic!("holder is an object");
    };
    runtime.state.object_mut(&id).unwrap().properties.insert(
        "items".to_string(),
        Value::List((0..count).map(|_| Value::Number(1.0)).collect()),
    );
    assert_eq!(
        json(
            &mut runtime,
            "[holder.items.size(), holder.items.len(), holder.items.length]"
        ),
        "[100000.0,100000.0,100000.0]"
    );
    assert_eq!(
        json(
            &mut runtime,
            "[holder.items.isEmpty(), holder.items.isNotEmpty()]"
        ),
        "[false,true]"
    );
    assert_eq!(
        json(
            &mut runtime,
            "long = \"é\".repeat(100000)\n[long.size(), long.length(), long.length, long.is_empty()]"
        ),
        "[100000.0,100000.0,100000.0,false]"
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-size-and-emptiness-aliases-{label}-{}.nuc",
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
    let program = "class Item(v):\n    this.v = v\nqueue = [\"a\"]\nwaiting = queue.size()\nidle = queue.isEmpty()\nbusy = queue.isNotEmpty()\nname = \"Zoë\"\nletters = name.length()\nItem(1)\ncount = Item.len()\nqueue.push(\"b\")\n[waiting, idle, busy, letters, count]";
    let spelled = "class Item(v):\n    this.v = v\nqueue = [\"a\"]\nwaiting = queue.length\nidle = queue.length == 0\nbusy = queue.length > 0\nname = \"Zoë\"\nletters = name.length\nItem(1)\ncount = Item.length\nqueue.push(\"b\")\n[waiting, idle, busy, letters, count]";
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
        "[2.0,false,true,3.0,1.0]\n"
    );
    let graph = String::from_utf8(file.run(program, &["--graph"]).stdout).unwrap();
    assert!(
        graph.contains("waiting [variable]\n    reads    queue"),
        "{graph}"
    );
    let failed = file.run("x = 5\nx.size()\n", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "TypeError: number.size is not a function (line 2, column 1)\n"
    );
    let session = "queue = [\"a\"]\nwaiting = queue.size()\nwaiting\nqueue.push(\"b\")\nwaiting\nqueue.isEmpty()\n\"Zoë\".len()\n\"\".is_empty()\n.exit\n";
    let spelled_session = "queue = [\"a\"]\nwaiting = queue.length\nwaiting\nqueue.push(\"b\")\nwaiting\nqueue.length == 0\n\"Zoë\".length\n\"\".length == 0\n.exit\n";
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let alias = repl(&session.replace('\n', newline), options);
            let written = repl(&spelled_session.replace('\n', newline), options);
            assert!(alias.status.success(), "{options:?} {alias:?}");
            assert_eq!(alias.stdout, written.stdout, "{options:?}");
            assert_eq!(alias.stderr, written.stderr, "{options:?}");
            let stdout = String::from_utf8(alias.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "1.0\n2.0\n2.0\nfalse\n3.0\ntrue\n"
            } else {
                "1\n2\n2\nfalse\n3\ntrue\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
        }
    }
    let failing = repl("(5).size()\n[1, 2].size()\n.exit\n", &["--plain"]);
    assert!(failing.status.success(), "{failing:?}");
    assert_eq!(
        String::from_utf8(failing.stderr).unwrap(),
        "TypeError: number.size is not a function (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(failing.stdout).unwrap();
    assert!(stdout.contains("2\n"), "{stdout}");
}
