mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime, Value};

const SPEC_TITLES: [&str; 3] = [
    "Nucleoid converts case with Kotlin, Rust and Swift spellings",
    "Nucleoid keeps class-level and mapped case conversions live",
    "Nucleoid rolls back failed updates through case conversion aliases",
];
const REFERENCE_TITLE: &str = "Nucleoid converts case as Kotlin, Rust and Swift do";
const README_TITLE: &str = "Case conversion methods keep a sensor's labels normalized";
const EXAMPLES_TITLE: &str = "Case conversions match badge scans to every employee";

const PAIRS: [(&str, &str, &str); 6] = [
    ("uppercase", "toUpperCase", "upper"),
    ("to_uppercase", "toUpperCase", "upper"),
    ("uppercased", "toUpperCase", "upper"),
    ("lowercase", "toLowerCase", "lower"),
    ("to_lowercase", "toLowerCase", "lower"),
    ("lowercased", "toLowerCase", "lower"),
];

const ALIASES: [&str; 6] = [
    "uppercase",
    "to_uppercase",
    "uppercased",
    "lowercase",
    "to_lowercase",
    "lowercased",
];

const SETUP: &str = "log = []";

const TEXTS: [(&str, &str); 30] = [
    ("", "\"abc\""),
    ("", "\"ABC def\""),
    ("", "\"\""),
    ("", "\"Straße\""),
    ("", "\"ﬁx ﬂow ﬀ\""),
    ("", "\"ǅemal ǈ ǋ\""),
    ("", "\"ΟΔΟΣ Σ ΑΣ.\""),
    ("", "\"İstanbul ıi\""),
    ("", "\"e\u{301}cole Ü\""),
    ("", "\"😀 1,2 #9 ½\""),
    ("s = \"Front Door\"", "s"),
    ("o = {label: \"Gate\"}", "o.label"),
    (
        "class Tag(name):\n    this.name = name\nt = Tag(\"Red Tag\")",
        "t.name",
    ),
    ("def name() => \"Ada Lovelace\"", "name()"),
    ("", "\"a,B,c\".split(\",\")[1]"),
    ("xs = [\"Ab\", \"cD\"]", "xs.map(x -> x + \"!\")[1]"),
    ("", "\"Hello World\".slice(0, 5)"),
    ("", "\"Hello World\".substring(6)"),
    ("xs = [\"a\", \"B\"]", "xs.join(\"-\")"),
    ("", "\"aB\".repeat(3)"),
    ("stem = \"ab\"", "`${stem}C`"),
    ("xs = [\"Ab\", \"cD\"]", "xs[0]"),
    ("s = \"Ab\"\ns = \"Cd\"", "s@first"),
    ("s = \"Ab\"\ns = \"Cd\"", "s@last"),
    ("s = \"Ab\"", "s.value"),
    ("flag = true", "(flag ? \"Yes\" : \"No\")"),
    ("", "String(42)"),
    ("", "(typeof 5)"),
    ("", "\"MiXeD\".toLowerCase()"),
    ("", "\"x\".padStart(3, \"Ab\")"),
];

const ARGUMENTS: [&str; 8] = [
    "",
    "\"tr\"",
    "1, \"two\", null, [3]",
    "log.push(\"a\"), log.push(\"b\")",
    "x -> x",
    "[\"x\", 1], {k: 1}",
    "Kind",
    "[], Kind",
];

const KINDS: &str = "class Kind(v):\n    this.v = v\nKind(1)\nKind(2)";

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
            title: "case conversion alias regression".to_string(),
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
    for title in [SPEC_TITLES[0], REFERENCE_TITLE] {
        let case = documents
            .iter()
            .flat_map(|(_, cases)| cases)
            .find(|case| case.title == title)
            .unwrap();
        let tokens = tokens(&case.source);
        for alias in ALIASES {
            let named = Token::Identifier(alias.to_string());
            assert!(
                tokens
                    .windows(2)
                    .any(|pair| pair[0] == Token::Dot && pair[1] == named),
                "{title} calls {alias}"
            );
        }
    }
    assert_eq!(assertions, 23);
    assert_eq!(cases, 1203);
}

#[test]
fn the_audit_finds_every_alias_call_and_no_declared_member() {
    for source in [
        "s.uppercase()",
        "s.to_lowercase(1)",
        "f(x).uppercased()",
        "\"ab\".lowercase()",
        "$Tag.name.to_uppercase()",
        "names.map(n -> n.lowercased())",
        "`${s.to_uppercase()} shown`",
        "`${count(1)} and ${f({a: t.lowercased()})}`",
        "x = cond ? s.lowercase() : null",
    ] {
        assert!(calls_alias(source), "{source}");
    }
    for source in [
        "s.toUpperCase()",
        "s.upper()",
        "s.lower(1)",
        "s.uppercase",
        "uppercase(s)",
        "uppercase = s.toUpperCase()",
        "\"uppercase\"",
        "`${uppercase}`",
        "# s.uppercase()\nx = 1",
        "s.Uppercase()",
        "s.to_ascii_uppercase()",
        "class Shout:\n    def uppercase() => 1\nShout().uppercase()",
        "o = {lowercase: () => 1}\no.lowercase()",
        "o = {}\no.to_uppercase = x -> x\no.to_uppercase(2)",
        "function uppercased(x) { return x }\ns.uppercased()",
    ] {
        assert!(!calls_alias(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_117_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[786],
        "Nucleoid rolls back failed updates through first and last aliases"
    );
    assert_eq!(titles[787..790], SPEC_TITLES);
    assert_eq!(titles[801], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid reads first and last elements as Kotlin and Rust do",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "First and last methods keep a sensor's newest reading live",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "First and last methods keep the next urgent patient live",
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
        .split_once("### Case conversion method aliases\n\n```\n")
        .expect("the language reference documents the case conversion aliases");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 118](nuc-0118.md)"));
    for line in [
        "text.uppercase()  text.lowercase()     Kotlin spellings: exactly text.toUpperCase(), text.toLowerCase() (NUC 118)",
        "text.to_uppercase()  text.to_lowercase() Rust spellings: exactly text.toUpperCase(), text.toLowerCase() (NUC 118)",
        "text.uppercased()  text.lowercased()   Swift spellings: exactly text.toUpperCase(), text.toLowerCase() (NUC 118)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    assert_eq!(
        case.expected.as_deref(),
        Some("[\"STRASSE\",\"école\",\"OK\"]")
    );
}

#[test]
fn each_alias_gives_its_accepted_spellings_result_on_every_string() {
    let mut changed = 0;
    let mut compared = 0;
    for (setup, receiver) in TEXTS {
        for (alias, spelled, legacy) in PAIRS {
            for arguments in ARGUMENTS {
                let written =
                    format!("{SETUP}\n{KINDS}\n{setup}\n[({receiver}).{alias}({arguments}), log]");
                let found = fresh(&written);
                assert!(found.starts_with("[\""), "{written}: {found}");
                for other in [spelled, legacy] {
                    let spelling = format!(
                        "{SETUP}\n{KINDS}\n{setup}\n[({receiver}).{other}({arguments}), log]"
                    );
                    assert_eq!(found, fresh(&spelling), "{written}");
                }
                let shown =
                    format!("{SETUP}\n{KINDS}\n{setup}\ntypeof ({receiver}).{alias}({arguments})");
                assert_eq!(fresh(&shown), "\"string\"", "{shown}");
                compared += 1;
            }
            let plain = fresh(&format!("{setup}\n{receiver}"));
            if fresh(&format!("{setup}\n({receiver}).{alias}()")) != plain {
                changed += 1;
            }
        }
    }
    assert_eq!(compared, TEXTS.len() * PAIRS.len() * ARGUMENTS.len());
    assert!(changed > TEXTS.len() * 4, "{changed}");
    for (source, expected) in [
        ("\"abc\".uppercase()", "\"ABC\""),
        ("\"abc\".to_uppercase()", "\"ABC\""),
        ("\"abc\".uppercased()", "\"ABC\""),
        ("\"ABC\".lowercase()", "\"abc\""),
        ("\"ABC\".to_lowercase()", "\"abc\""),
        ("\"ABC\".lowercased()", "\"abc\""),
        ("\"MiXeD 42!\".uppercase()", "\"MIXED 42!\""),
        ("\"MiXeD 42!\".lowercased()", "\"mixed 42!\""),
        ("[\"\".uppercase(), \"\".to_lowercase()]", "[\"\",\"\"]"),
        (
            "s = \"Straße\"\n[s.uppercase(), s.uppercase().length, s.length, s.to_lowercase()]",
            "[\"STRASSE\",7.0,6.0,\"straße\"]",
        ),
        ("\"ﬁx\".to_uppercase()", "\"FIX\""),
        (
            "[\"ǅ\".uppercased(), \"ǅ\".lowercased(), \"Ǆ\".to_lowercase()]",
            "[\"Ǆ\",\"ǆ\",\"ǆ\"]",
        ),
        (
            "[\"ΟΔΟΣ\".lowercase(), \"ΣΑΣ Σ\".to_lowercase()]",
            "[\"οδος\",\"σας σ\"]",
        ),
        (
            "[\"İ\".lowercase().length, \"ı\".uppercase(), \"I\".lowercased()]",
            "[2.0,\"I\",\"i\"]",
        ),
        (
            "[\"e\u{301}\".uppercase().length, \"😀 9\".uppercased()]",
            "[2.0,\"😀 9\"]",
        ),
        ("\"abc\".uppercase(\"tr\", {locale: \"tr\"})", "\"ABC\""),
        ("\"Iİ\".to_lowercase(\"tr-TR\").length", "3.0"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn arguments_are_evaluated_and_their_errors_propagate_as_for_the_spelling() {
    let mut raised = 0;
    for (setup, receiver) in TEXTS {
        for (alias, spelled, _) in PAIRS {
            for arguments in ["missing", "\"a\", missing", "log.push(1), o.missing.deeper"] {
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
        (
            "s = \"ab\"\n[s.uppercase, s.to_lowercase, s.uppercased, s.lowercase, s.to_uppercase, s.lowercased]",
            "[null,null,null,null,null,null]",
        ),
        (
            "u = null\n[u.uppercase(missing), u.to_lowercase(missing), u.lowercased()]",
            "[null,null,null]",
        ),
        ("o = {}\no.label.uppercase(missing)", "null"),
        (
            "calls = []\nn = null\nn.uppercase(calls.push(1))\nn.to_lowercase(calls.push(2))\ncalls",
            "[]",
        ),
        (
            "o = {uppercase: () => \"own\", lowercased: x -> x + 1}\n[o.uppercase(), o.lowercased(1)]",
            "[\"own\",2.0]",
        ),
        (
            "o = {a: 1}\n[o.uppercase(), o.to_lowercase(\"x\")]",
            "[null,null]",
        ),
        (
            "class Shout:\n    def uppercase() => \"own upper\"\n    def to_lowercase(x) => x\ns = Shout()\n[s.uppercase(), s.to_lowercase(5)]",
            "[\"own upper\",5.0]",
        ),
        (
            "class P(v):\n    this.v = v\np = P(\"Ab\")\n[p.uppercase(), p.lowercased()]",
            "[null,null]",
        ),
        (
            "def uppercase(x) => \"fn \" + x\ndef to_lowercase(x) => 3\n[uppercase(\"a\"), to_lowercase(\"B\")]",
            "[\"fn a\",3.0]",
        ),
        (
            "class Tag(name):\n    this.name = name\n$Tag.u = $Tag.uppercase()\nt = Tag(\"x\")\nt.u",
            "null",
        ),
        (
            "class Tag(name):\n    this.name = name\n    def uppercase() => \"own\"\n$Tag.u = $Tag.uppercase()\nt = Tag(\"x\")\nt.u",
            "\"own\"",
        ),
        (
            "s = \"Ab\"\n[s.toUpperCase(), s.toLowerCase(), s.upper(), s.lower(), s.toUpperCase(1, 2)]",
            "[\"AB\",\"ab\",\"AB\",\"ab\",\"AB\"]",
        ),
        (
            "class Item(v):\n    this.v = v\nItem(\"a\")\n[Item.uppercase, Item.lowercased]",
            "[null,null]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "s = \"ab\"\ns.uppercase = 1",
            "TypeError: Cannot assign 'uppercase' on string (line 2, column 1)",
        ),
        (
            "s = \"ab\"\ns.to_lowercase = 1",
            "TypeError: Cannot assign 'to_lowercase' on string (line 2, column 1)",
        ),
        (
            "o = {uppercase: 3}\no.uppercase()",
            "TypeError: obj:0.uppercase is not a function (line 2, column 1)",
        ),
        (
            "xs = [\"a\"]\nxs.uppercase()",
            "TypeError: xs.uppercase is not a function (line 2, column 1)",
        ),
        (
            "class Item(v):\n    this.v = v\nItem(\"a\")\nItem.lowercased()",
            "TypeError: Item.lowercased is not a function (line 4, column 1)",
        ),
        (
            "(5).uppercase()",
            "TypeError: number.uppercase is not a function (line 1, column 1)",
        ),
        (
            "b = true\nb.to_lowercase()",
            "TypeError: boolean.to_lowercase is not a function (line 2, column 1)",
        ),
        (
            "f = x -> x\nf.uppercased()",
            "TypeError: function.uppercased is not a function (line 2, column 1)",
        ),
        (
            "d = Date(0)\nd.lowercase()",
            "TypeError: Date.lowercase is not a function (line 2, column 1)",
        ),
        (
            "r = /a/\nr.to_uppercase()",
            "TypeError: to_uppercase is not a function (line 2, column 1)",
        ),
        (
            "String.uppercase()",
            "TypeError: String.uppercase is not a function (line 1, column 1)",
        ),
        (
            "String.to_lowercase(\"AB\")",
            "TypeError: String.to_lowercase is not a function (line 1, column 1)",
        ),
        (
            "class String(v):\n    this.v = v\nString(1)\nString.uppercase()",
            "TypeError: String.uppercase is not a function (line 4, column 1)",
        ),
        (
            "Math.lowercased()",
            "TypeError: Math.lowercased is not a function (line 1, column 1)",
        ),
        (
            "uppercase(\"a\")",
            "ReferenceError: uppercase is not defined (line 1, column 1)",
        ),
        (
            "String.uppercase(missing)",
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
        (
            "\"abc\".capitalize()",
            "TypeError: abc.capitalize is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".capitalized()",
            "TypeError: abc.capitalized is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".to_ascii_uppercase()",
            "TypeError: abc.to_ascii_uppercase is not a function (line 1, column 1)",
        ),
        (
            "\"ABC\".to_ascii_lowercase()",
            "TypeError: ABC.to_ascii_lowercase is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".ToUpper()",
            "TypeError: abc.ToUpper is not a function (line 1, column 1)",
        ),
        (
            "\"ABC\".ToLowerInvariant()",
            "TypeError: ABC.ToLowerInvariant is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".upcase()",
            "TypeError: abc.upcase is not a function (line 1, column 1)",
        ),
        (
            "\"ABC\".downcase()",
            "TypeError: ABC.downcase is not a function (line 1, column 1)",
        ),
        (
            "\"ABC\".casefold()",
            "TypeError: ABC.casefold is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".swapcase()",
            "TypeError: abc.swapcase is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".title()",
            "TypeError: abc.title is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".replaceFirstChar(x -> x)",
            "TypeError: abc.replaceFirstChar is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".Uppercase()",
            "TypeError: abc.Uppercase is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".UPPERCASE()",
            "TypeError: abc.UPPERCASE is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".toUppercase()",
            "TypeError: abc.toUppercase is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".to_upper()",
            "TypeError: abc.to_upper is not a function (line 1, column 1)",
        ),
        (
            "\"abc\".uppercase_()",
            "TypeError: abc.uppercase_ is not a function (line 1, column 1)",
        ),
        (
            "\"ABC\".lowerCase()",
            "TypeError: ABC.lowerCase is not a function (line 1, column 1)",
        ),
    ] {
        assert_eq!(failure(source), expected, "{source}");
    }
}

#[test]
fn the_public_string_method_takes_the_six_aliases() {
    for (alias, spelled, legacy) in PAIRS {
        let expected = nucleoid::builtins::string_method("Straße ǅ", spelled, &[]).unwrap();
        assert_eq!(
            nucleoid::builtins::string_method("Straße ǅ", alias, &[]).unwrap(),
            expected,
            "{alias}"
        );
        assert_eq!(
            nucleoid::builtins::string_method("Straße ǅ", legacy, &[]).unwrap(),
            expected,
            "{legacy}"
        );
        assert_eq!(
            nucleoid::builtins::string_method(
                "Straße ǅ",
                alias,
                &[Value::Number(1.0), Value::Null, Value::Undefined]
            )
            .unwrap(),
            expected,
            "{alias} with arguments"
        );
    }
    assert_eq!(
        nucleoid::builtins::string_method("Straße ǅ", "uppercase", &[]).unwrap(),
        Value::String("STRASSE Ǆ".to_string())
    );
    assert_eq!(
        nucleoid::builtins::string_method("Straße ǅ", "lowercased", &[]).unwrap(),
        Value::String("straße ǆ".to_string())
    );
    assert_eq!(
        nucleoid::builtins::string_method("", "to_uppercase", &[]).unwrap(),
        Value::String(String::new())
    );
    for name in ["capitalize", "Uppercase", "to_ascii_lowercase", "upcase"] {
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
            ("label = \"Front Door\"", "label = \"Front Door\""),
            ("a = label.lowercase()", "a = label.toLowerCase()"),
            ("b = label.to_lowercase()", "b = label.toLowerCase()"),
            ("c = label.lowercased()", "c = label.toLowerCase()"),
            ("d = label.uppercase()", "d = label.toUpperCase()"),
            ("e = label.to_uppercase()", "e = label.toUpperCase()"),
            (
                "f = label.uppercased(\"tr\")",
                "f = label.toUpperCase(\"tr\")",
            ),
            ("[a, b, c, d, e, f]", "[a, b, c, d, e, f]"),
            ("label = \"Straße\"", "label = \"Straße\""),
            ("[a, b, c, d, e, f]", "[a, b, c, d, e, f]"),
            ("label = \"\"", "label = \"\""),
            ("[a, b, c, d, e, f]", "[a, b, c, d, e, f]"),
        ],
        &["a", "b", "c", "d", "e", "f"],
    );
    behaves_as_spelled(
        &[
            ("first = \"Ana\"", "first = \"Ana\""),
            ("last = \"Lima\"", "last = \"Lima\""),
            (
                "full = (first + \" \" + last).uppercase()",
                "full = (first + \" \" + last).toUpperCase()",
            ),
            (
                "slug = `${first}-${last}`.to_lowercase()",
                "slug = `${first}-${last}`.toLowerCase()",
            ),
            ("[full, slug]", "[full, slug]"),
            ("last = \"Ölçer\"", "last = \"Ölçer\""),
            ("[full, slug]", "[full, slug]"),
            ("first = \"BO\"", "first = \"BO\""),
            ("[full, slug]", "[full, slug]"),
        ],
        &["full", "slug"],
    );
    behaves_as_spelled(
        &[
            ("o = {name: \"Ana\"}", "o = {name: \"Ana\"}"),
            ("n = o.name.uppercased()", "n = o.name.toUpperCase()"),
            (
                "class Tag(name):\n    this.name = name",
                "class Tag(name):\n    this.name = name",
            ),
            ("t = Tag(\"Red\")", "t = Tag(\"Red\")"),
            ("m = t.name.to_lowercase()", "m = t.name.toLowerCase()"),
            ("words = [\"Ab\", \"cD\"]", "words = [\"Ab\", \"cD\"]"),
            (
                "w = words.map(x -> x.lowercase())",
                "w = words.map(x -> x.toLowerCase())",
            ),
            ("[n, m, w]", "[n, m, w]"),
            ("o.name = \"Bo\"", "o.name = \"Bo\""),
            ("t.name = \"BLUE\"", "t.name = \"BLUE\""),
            ("words.push(\"Éé\")", "words.push(\"Éé\")"),
            ("[n, m, w]", "[n, m, w]"),
            ("o = {name: \"Cy\"}", "o = {name: \"Cy\"}"),
            ("words = [\"Z\"]", "words = [\"Z\"]"),
            ("[n, m, w]", "[n, m, w]"),
        ],
        &["n", "m", "w"],
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "label = \"Gate\"\nkey = label.lowercase()\nassert(key, \"gate\")\nlabel = \"Straße\"\nassert(key, \"straße\")\nlabel = \"DOOR\"\nassert(key, \"door\")",
    );
    assert_eq!(dependencies(&runtime, "key"), ["label"]);
}

#[test]
fn class_level_rules_follow_each_instance_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            (
                "class Badge(name, team):\n    this.name = name\n    this.team = team",
                "class Badge(name, team):\n    this.name = name\n    this.team = team",
            ),
            (
                "$Badge.login = $Badge.name.to_lowercase()",
                "$Badge.login = $Badge.name.toLowerCase()",
            ),
            (
                "$Badge.crest = $Badge.team.uppercased()",
                "$Badge.crest = $Badge.team.toUpperCase()",
            ),
            (
                "$Badge.handle = $Badge.name.lowercase() + \"@\" + $Badge.team.uppercase()",
                "$Badge.handle = $Badge.name.toLowerCase() + \"@\" + $Badge.team.toUpperCase()",
            ),
            (
                "a = Badge(\"Ana\", \"blue\")",
                "a = Badge(\"Ana\", \"blue\")",
            ),
            ("b = Badge(\"BO\", \"Red\")", "b = Badge(\"BO\", \"Red\")"),
            (
                "[a.login, a.crest, a.handle, b.login, b.crest, b.handle]",
                "[a.login, a.crest, a.handle, b.login, b.crest, b.handle]",
            ),
            ("a.name = \"Straße\"", "a.name = \"Straße\""),
            ("b.team = \"ǅ\"", "b.team = \"ǅ\""),
            (
                "[a.login, a.crest, a.handle, b.login, b.crest, b.handle]",
                "[a.login, a.crest, a.handle, b.login, b.crest, b.handle]",
            ),
        ],
        &[],
    );
    behaves_as_spelled(
        &[
            (
                "class User(name):\n    this.name = name",
                "class User(name):\n    this.name = name",
            ),
            (
                "if $User.name.to_uppercase() == \"ADMIN\":\n    $User.role = \"admin\"\nelse:\n    $User.role = \"user\"",
                "if $User.name.toUpperCase() == \"ADMIN\":\n    $User.role = \"admin\"\nelse:\n    $User.role = \"user\"",
            ),
            ("a = User(\"Admin\")", "a = User(\"Admin\")"),
            ("b = User(\"bo\")", "b = User(\"bo\")"),
            ("[a.role, b.role]", "[a.role, b.role]"),
            ("a.name = \"ana\"", "a.name = \"ana\""),
            ("b.name = \"aDmIn\"", "b.name = \"aDmIn\""),
            ("[a.role, b.role]", "[a.role, b.role]"),
        ],
        &[],
    );
    assert_eq!(
        result(
            "class Tag(name):\n    this.name = name\n$Tag.key = $Tag.name.lowercased()\nred = Tag(\"RED\")\nblue = Tag(\"Blue\")\nkeys = Tag.map(tag -> tag.key)\nred.name = \"Crimson\"\nkeys"
        ),
        "[\"crimson\",\"blue\"]"
    );
}

#[test]
fn a_held_receiver_replaced_by_another_kind_reads_the_alias_name_on_it() {
    for (alias, spelled, _) in PAIRS {
        let upper = spelled == "toUpperCase";
        let converted = if upper { "\"AB\"" } else { "\"ab\"" };
        let rewritten = if upper { "\"CD\"" } else { "\"cd\"" };
        let raised = |kind: &str, name: &str| {
            Some(format!(
                "TypeError: {kind}.{name} is not a function (line 1, column 1)"
            ))
        };
        let member = format!("s = {{{alias}: () => \"own\"}}");
        let boxed = format!("class Box:\n    def {alias}() => \"boxed\"\ns = Box()");
        let read = |name: &str| vec!["s".to_string(), format!("obj:0.{name}")];
        let kept = vec!["s".to_string()];
        for (replacement, written_error, spelled_error, values, written_from, spelled_from) in [
            (
                "s = [\"x\"]".to_string(),
                raised("s", alias),
                raised("s", spelled),
                [converted, converted],
                kept.clone(),
                kept.clone(),
            ),
            (
                "s = 7".to_string(),
                raised("number", alias),
                raised("number", spelled),
                [converted, converted],
                kept.clone(),
                kept.clone(),
            ),
            (
                "s = false".to_string(),
                raised("boolean", alias),
                raised("boolean", spelled),
                [converted, converted],
                kept.clone(),
                kept.clone(),
            ),
            (
                "s = Date(0)".to_string(),
                raised("Date", alias),
                raised("Date", spelled),
                [converted, converted],
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
                "s = \"Cd\"".to_string(),
                None,
                None,
                [rewritten, rewritten],
                kept.clone(),
                kept.clone(),
            ),
        ] {
            let mut written = Runtime::new();
            let mut spelling = Runtime::new();
            written
                .run(&format!("s = \"aB\"\nu = s.{alias}()"))
                .unwrap();
            spelling
                .run(&format!("s = \"aB\"\nu = s.{spelled}()"))
                .unwrap();
            for (runtime, error, value, from) in [
                (&mut written, written_error, values[0], written_from),
                (&mut spelling, spelled_error, values[1], spelled_from),
            ] {
                match (runtime.run(&replacement), error) {
                    (Err(found), Some(expected)) => {
                        assert_eq!(renumbered(&found.to_string()), expected, "{replacement}");
                        assert_eq!(json(runtime, "s"), "\"aB\"", "{replacement}");
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
    assert_eq!(
        result(
            "s = \"aB\"\nu = s.uppercase()\ns = {uppercase: () => \"own\"}\ns.uppercase = () => \"changed\"\nu"
        ),
        "\"changed\""
    );
    assert_eq!(
        result(
            "s = \"aB\"\nu = s.toUpperCase()\ns = {uppercase: () => \"own\"}\ns.uppercase = () => \"changed\"\nu"
        ),
        "null"
    );
}

#[test]
fn explanations_name_the_dependencies_of_the_accepted_spelling() {
    let setup = "class Tag(name):\n    this.name = name\nt = Tag(\"Red\")\nlabel = \"Door\"\n";
    let mut alias = Runtime::new();
    let mut spelled = Runtime::new();
    alias
        .run(&format!(
            "{setup}k = label.lowercase()\nc = label.to_uppercase(1)\nb = label.uppercased()\nn = t.name.to_lowercase()\nw = `${{label}}!`.lowercased()"
        ))
        .unwrap();
    spelled
        .run(&format!(
            "{setup}k = label.toLowerCase()\nc = label.toUpperCase(1)\nb = label.toUpperCase()\nn = t.name.toLowerCase()\nw = `${{label}}!`.toLowerCase()"
        ))
        .unwrap();
    for (name, rule, spelling) in [
        (
            "k",
            "\"k = label.lowercase()\"",
            "\"k = label.toLowerCase()\"",
        ),
        (
            "c",
            "\"c = label.to_uppercase(1)\"",
            "\"c = label.toUpperCase(1)\"",
        ),
        (
            "b",
            "\"b = label.uppercased()\"",
            "\"b = label.toUpperCase()\"",
        ),
        (
            "n",
            "\"n = t.name.to_lowercase()\"",
            "\"n = t.name.toLowerCase()\"",
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
    assert_eq!(dependencies(&alias, "w"), dependencies(&spelled, "w"), "w");
    assert_eq!(dependencies(&alias, "k"), ["label"]);
    for query in [
        "(affects label).map(step -> step.node)",
        "(affects t).map(step -> step.node)",
    ] {
        assert_eq!(
            json(&mut alias, query),
            json(&mut spelled, query),
            "{query}"
        );
    }
    alias.run("label = \"Straße\"\nt.name = \"BLUE\"").unwrap();
    spelled
        .run("label = \"Straße\"\nt.name = \"BLUE\"")
        .unwrap();
    assert_eq!(
        json(&mut alias, "[k, c, b, n, w]"),
        "[\"straße\",\"STRASSE\",\"STRASSE\",\"blue\",\"straße!\"]"
    );
    assert_eq!(
        json(&mut alias, "[k, c, b, n, w]"),
        json(&mut spelled, "[k, c, b, n, w]")
    );
    for (source, rendered) in [
        ("k = label.lowercase()", "k=label.lowercase()"),
        (
            "c = label.to_uppercase(\"tr\", 2)",
            "c=label.to_uppercase(\"tr\",2)",
        ),
        (
            "$Tag.k = $Tag.name.lowercased()",
            "$Tag.k=$Tag.name.lowercased()",
        ),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), rendered, "{source}");
    }
}

#[test]
fn the_receiver_and_then_every_argument_evaluate_once_in_written_order() {
    let functions = "log = []\ndef text(label):\n    log.push(label)\n    return \"Ab\"\ndef arg(label):\n    log.push(label)\n    return 0\n";
    for (alias, spelled, _) in PAIRS {
        let converted = if spelled == "toUpperCase" { "AB" } else { "ab" };
        for (call, expected_log) in [
            (format!("text(\"r\").{alias}()"), "[\"r\"]"),
            (
                format!("text(\"r\").{alias}(arg(\"a\"), arg(\"b\"))"),
                "[\"r\",\"a\",\"b\"]",
            ),
            (
                format!("text(\"r\").{alias}(arg(\"a\"), text(\"x\"), arg(\"c\"))"),
                "[\"r\",\"a\",\"x\",\"c\"]",
            ),
        ] {
            let source = format!("{functions}[{call}, log]");
            let expected = format!("[\"{converted}\",{expected_log}]");
            assert_eq!(result(&source), expected, "{source}");
            assert_eq!(
                result(&source.replace(alias, spelled)),
                expected,
                "{source}"
            );
        }
        let mut runtime = Runtime::new();
        runtime.run(functions).unwrap();
        assert_eq!(
            runtime
                .run(&format!(
                    "text(\"r\").{alias}(arg(\"a\"), missing, arg(\"b\"))"
                ))
                .unwrap_err()
                .to_string(),
            "ReferenceError: missing is not defined (line 1, column 1)"
        );
        assert_eq!(json(&mut runtime, "log"), "[]");
    }
}

#[test]
fn aliases_read_in_every_statement_position() {
    for (source, expected) in [
        (
            "r = 0\ntry:\n    r = (5).uppercase()\ncatch error:\n    r = error\nr",
            "\"TypeError: number.uppercase is not a function\"",
        ),
        (
            "r = 0\ntry:\n    r = [\"a\"].to_lowercase()\ncatch error:\n    r = error\nr",
            "\"TypeError: [\\\"a\\\"].to_lowercase is not a function\"",
        ),
        (
            "label = \"Door\"\nkey = label.lowercase()\nseen = suppose label = \"GATE\": key\n[seen, key, label]",
            "[\"gate\",\"door\",\"Door\"]",
        ),
        (
            "name = \"ada\"\n`Hello, ${name.uppercased()}!`",
            "\"Hello, ADA!\"",
        ),
        (
            "code = \"ok\"\nif code.to_uppercase() == \"OK\":\n    state = \"fine\"\nelse:\n    state = \"bad\"\nstate",
            "\"fine\"",
        ),
        (
            "answer = \"YES\"\nanswer.lowercased() == \"yes\" ? \"agreed\" : \"refused\"",
            "\"agreed\"",
        ),
        (
            "def shout(text) => text.uppercase() + \"!\"\n[shout(\"hey\"), shout(\"Straße\")]",
            "[\"HEY!\",\"STRASSE!\"]",
        ),
        (
            "def quiet(text):\n    return text.to_lowercase()\nquiet(\"LOUD\")",
            "\"loud\"",
        ),
        (
            "names = [\"Ana\", \"BO\"]\nnames.map(name -> name.lowercase()).filter(name -> name.uppercased() != \"BO\")",
            "[\"ana\"]",
        ),
        (
            "words = [\"A\", \"b\", \"STOP\", \"c\"]\ni = 0\nwhile words[i].lowercase() != \"stop\":\n    i = i + 1\ni",
            "2.0",
        ),
        (
            "class Tag(name):\n    this.name = name\n    def loud() => this.name.uppercased()\n    def soft() => self.name.to_lowercase()\nt = Tag(\"Mixed\")\n[t.loud(), t.soft()]",
            "[\"MIXED\",\"mixed\"]",
        ),
        (
            "codes = [\"ab\", \"Cd\"]\ncodes.map(code -> code.to_uppercase()).join(\"|\")",
            "\"AB|CD\"",
        ),
        ("\"A-b\".lowercase().uppercased().to_lowercase()", "\"a-b\""),
        (
            "class P(v):\n    this.v = v\np = P(\"Ab\")\n[p.v.uppercase(), p.v.to_lowercase()]",
            "[\"AB\",\"ab\"]",
        ),
        (
            "def uppercase(x) => \"fn \" + x\ndef to_lowercase(x) => 3\n[uppercase(\"a\"), to_lowercase(\"B\"), \"a\".uppercase(), \"B\".to_lowercase()]",
            "[\"fn a\",3.0,\"A\",\"b\"]",
        ),
        (
            "class Tag(name):\n    this.name = name\n    def uppercase() => \"own\"\n$Tag.u = $Tag.uppercase()\n$Tag.n = $Tag.name.uppercase()\nt = Tag(\"x\")\n[t.u, t.n]",
            "[\"own\",\"X\"]",
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
        "label = \"door\"\ncode = label.uppercase()\nfailure = \"\"\ntry:\n    label = 7\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.uppercase is not a function\"))\nassert([label, code], [\"door\", \"DOOR\"])",
    );
    checked(
        &mut runtime,
        "class Tag(name):\n    this.name = name\n$Tag.key = $Tag.name.to_lowercase()\nred = Tag(\"RED\")\ntry:\n    red.name = true\ncatch error:\n    failure = error\nassert(failure, TypeError(\"boolean.to_lowercase is not a function\"))\nassert([red.name, red.key], [\"RED\", \"red\"])",
    );
    checked(
        &mut runtime,
        "words = [\"Ab\"]\nsoft = words.map(word -> word.lowercased())\ntry:\n    words.push(3)\ncatch error:\n    failure = error\nassert([failure, words, soft], [TypeError(\"number.lowercased is not a function\"), [\"Ab\"], [\"ab\"]])",
    );
    checked(
        &mut runtime,
        "try:\n    label = \"gate\"\n    red.name = \"Blue\"\n    words.push(\"CD\")\n    assert([code, red.key, soft], [\"GATE\", \"blue\", [\"ab\", \"cd\"]])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([label, code, red.key, soft], [\"door\", \"DOOR\", \"red\", [\"ab\"]])",
    );
    assert_eq!(dependencies(&runtime, "code"), ["label"]);
    assert_eq!(
        failure("label = \"a\"\ncode = label.to_uppercase()\nlabel = [1]"),
        "TypeError: label.to_uppercase is not a function (line 3, column 1)"
    );
    let mut recovered = Runtime::new();
    recovered
        .run("label = \"Ab\"\nkey = label.lowercase()")
        .unwrap();
    assert!(recovered.run("label = 5").is_err());
    assert_eq!(json(&mut recovered, "[label, key]"), "[\"Ab\",\"ab\"]");
    recovered.run("label = \"ÉTÉ\"").unwrap();
    assert_eq!(json(&mut recovered, "key"), "\"été\"");
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    for alias in ALIASES {
        let converted = if alias.contains("upper") { "AB" } else { "ab" };
        assert_eq!(
            result(&format!(
                "s = \"aB\"\ntry:\n    r = s.{alias}()\ncatch e:\n    r = \"caught\"\nr"
            )),
            format!("\"{converted}\"")
        );
        assert_eq!(
            result(&format!(
                "xs = [\"aB\"]\ntry:\n    r = xs.{alias}()\ncatch e:\n    r = \"caught\"\nr"
            )),
            "\"caught\""
        );
    }
}

#[test]
fn long_strings_convert_through_the_runtime_and_the_public_api() {
    let mut runtime = Runtime::new();
    runtime
        .run("long = \"aßc\".repeat(40000)\nupper = long.uppercase()\nlower = upper.to_lowercase()")
        .unwrap();
    assert_eq!(
        json(
            &mut runtime,
            "[long.length, upper.length, lower.length, upper == long.toUpperCase(), lower == upper.toLowerCase()]"
        ),
        "[120000.0,160000.0,160000.0,true,true]"
    );
    let text = "Straße ǅ ".repeat(10_000);
    let Value::String(upper) = nucleoid::builtins::string_method(&text, "uppercased", &[]).unwrap()
    else {
        panic!("the alias returns a string");
    };
    assert_eq!(upper, text.to_uppercase());
    assert_eq!(upper.chars().count(), 100_000);
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-case-conversion-aliases-{label}-{}.nuc",
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

fn spelled_out(source: &str) -> String {
    let mut spelled = source.to_string();
    for (alias, spelling, _) in PAIRS {
        spelled = spelled.replace(&format!(".{alias}("), &format!(".{spelling}("));
    }
    spelled
}

#[test]
fn files_graphs_and_repl_sessions_run_the_aliases() {
    let program = "label = \"Front Door\"\nkey = label.lowercase()\ncode = label.to_uppercase()\nshown = label.uppercased()\nsoft = label.to_lowercase()\nquiet = label.lowercased()\nloud = label.uppercase()\nlabel = \"Straße\"\n[key, code, shown, soft, quiet, loud]";
    let spelled = spelled_out(program);
    assert!(!spelled.contains("case("), "{spelled}");
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
        "[\"straße\",\"STRASSE\",\"STRASSE\",\"straße\",\"straße\",\"STRASSE\"]\n"
    );
    let graph = String::from_utf8(file.run(program, &["--graph"]).stdout).unwrap();
    assert!(
        graph.contains("key [variable]\n    reads    label"),
        "{graph}"
    );
    let failed = file.run("x = 5\nx.uppercase()\n", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "TypeError: number.uppercase is not a function (line 2, column 1)\n"
    );
    let session = "name = \"Ada\"\nhandle = name.lowercase()\nhandle\nname = \"ÉMILE\"\nhandle\nname.to_uppercase()\n\"ǅ\".uppercased()\n\"OK\".to_lowercase(1)\n\"\".lowercased()\n.exit\n";
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
                "\"Ada\"\n\"ada\"\n\"ada\"\n\"ÉMILE\"\n\"émile\"\n\"ÉMILE\"\n\"Ǆ\"\n\"ok\"\n\"\"\n"
            } else {
                "Ada\nada\nada\nÉMILE\némile\nÉMILE\nǄ\nok\n\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
        }
    }
    let failing = repl(
        "(5).uppercase()\n\"Ab\".lowercased()\n.exit\n",
        &["--plain"],
    );
    assert!(failing.status.success(), "{failing:?}");
    assert_eq!(
        String::from_utf8(failing.stderr).unwrap(),
        "TypeError: number.uppercase is not a function (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(failing.stdout).unwrap();
    assert!(stdout.contains("ab\n"), "{stdout}");
}
