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
    "Nucleoid tests prefixes and suffixes with startswith and endswith",
    "Nucleoid joins lists and class populations with the separator as the receiver",
    "Nucleoid rolls back failed updates through Python string methods",
];
const REFERENCE_TITLE: &str = "Nucleoid spells startswith, endswith and join as Python does";
const README_TITLE: &str = "Python string methods classify files and list them";
const EXAMPLES_TITLE: &str = "Python string methods keep an order desk's rules live";

const TEXTS: [(&str, &str); 12] = [
    ("", "\"report.csv\""),
    ("s = \"report.csv\"", "s"),
    ("o = {n: \"report.csv\"}", "o.n"),
    ("def name() => \"report.csv\"", "name()"),
    ("stem = \"report\"", "`${stem}.csv`"),
    ("s = \"report.csv\"", "s.value"),
    ("s = \"report.csv\"\ns = \"notes.txt\"", "s@first"),
    ("xs = [\"notes.txt\", \"report.csv\"]", "xs[1]"),
    ("", "\"REPORT.CSV\".toLowerCase()"),
    ("flag = true", "(flag ? \"report.csv\" : \"notes.txt\")"),
    ("", "\"\""),
    ("", "\"café.csv\""),
];

const AFFIXES: [&str; 15] = [
    "\"report\"",
    "\".csv\"",
    "\"\"",
    "\"report.csv\"",
    "\"report.csv.bak\"",
    "\"REPORT\"",
    "\"rep\", 3",
    "\"csv\", 0, 99",
    "5",
    "null",
    "true",
    "[\".csv\"]",
    "",
    "undefined",
    "\"café\"",
];

const POPULATION: &str = "class Tag(n):\n    this.n = n\nred = Tag(1)\nblue = Tag(2)\nTag(3)\nclass Empty\nsep = \" / \"";

const LISTS: [(&str, &str); 22] = [
    ("", "[\"a\", \"b\"]"),
    ("xs = [\"a\", \"b\", \"c\"]", "xs"),
    ("o = {items: [\"a\", \"b\"]}", "o.items"),
    ("def make() => [\"a\", \"b\"]", "make()"),
    ("xs = [\"a\", \"b\", \"c\"]", "xs[1:]"),
    ("xs = [\"a\", \"b\"]", "xs.value"),
    ("xs = [\"a\"]\nxs.push(\"b\")", "xs@first"),
    ("grid = [[\"a\", \"b\"], [\"c\"]]", "grid[0]"),
    ("xs = [\"a\", \"b\"]", "xs.map(x -> x + \"!\")"),
    ("xs = [\"a\"]", "xs.concat([\"b\", \"c\"])"),
    ("", "[]"),
    ("", "[\"a\", 1, 2.5, -0, 0 / 0, null, true, [1, [2]], \"\"]"),
    ("o = {k: 1}", "[o, {k: 2}, red]"),
    ("", "Tag"),
    ("kind = Tag", "kind"),
    ("", "$Tag"),
    ("", "Tag[1:]"),
    ("", "Tag.list()"),
    ("", "Tag.filter(tag -> tag.n > 1)"),
    ("", "Tag.map(tag -> tag.n)"),
    ("", "Empty"),
    ("class List(v):\n    this.v = v\nList(1)\nList(2)", "List"),
];

const SEPARATORS: [&str; 7] = ["\", \"", "\"\"", "\"-\"", "\" | \"", "sep", "`+`", "\"→\""];

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
    let affix = tokens.windows(3).any(|window| {
        window[0] == Token::Dot
            && matches!(&window[1], Token::Identifier(name) if name == "startswith" || name == "endswith")
            && window[2] == Token::LeftParen
    });
    let join = tokens.windows(4).any(|window| {
        matches!(window[0], Token::Str(_) | Token::Template(_))
            && window[1] == Token::Dot
            && window[2] == Token::Identifier("join".to_string())
            && window[3] == Token::LeftParen
    });
    affix || join
}

fn accepted(name: &str) -> &'static str {
    match name {
        "startswith" => "startsWith",
        "endswith" => "endsWith",
        other => panic!("{other} is not an alias"),
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
    let error = Runtime::new()
        .run(source)
        .expect_err("the program raises an error");
    error.to_string()
}

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "python string alias regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"))
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

fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names
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
            if alias.graph.retrieve(&NodeKey::variable(*name)).is_some() {
                assert_eq!(
                    sorted(dependencies(&alias, name)),
                    sorted(dependencies(&spelled, name)),
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
    assert_eq!(assertions, 26);
    assert_eq!(cases, 1326);
}

#[test]
fn the_audit_finds_every_alias_spelling_and_nothing_else() {
    for source in [
        "s.startswith(\"a\")",
        "s.endswith(\"a\")",
        "\", \".join(xs)",
        "`-`.join(xs)",
        "f(x).startswith(\"a\")",
    ] {
        assert!(calls_alias(source), "{source}");
    }
    for source in [
        "s.startsWith(\"a\")",
        "xs.join(\", \")",
        "sep.join(xs)",
        "s.startswith",
        "startswith(\"a\")",
        "o = {startswith: 1}",
        "\"startswith\"",
    ] {
        assert!(!calls_alias(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_109_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[761],
        "Nucleoid rolls back failed updates through sections"
    );
    assert_eq!(titles[762..765], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid writes one-parameter functions as operator sections",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Operator sections pass a comparison without naming a parameter",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Operator sections keep every sensor's alarm count live",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 28 - 12 * usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 20].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Python string method aliases\n\n```\n")
        .expect("the language reference documents the Python string aliases");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 110](nuc-0110.md)"));
    assert!(reference.contains(
        "text.startswith(p)  text.endswith(s)  Python spellings: exactly text.startsWith(p), text.endsWith(s) (NUC 110)"
    ));
    assert!(reference.contains(
        "\", \".join(items)  \", \".join(Name)      Python spelling: exactly items.join(\", \"), Name.join(\", \") (NUC 110)"
    ));
    assert_eq!(case.expected.as_deref(), Some("true"));
}

#[test]
fn prefix_and_suffix_aliases_give_their_accepted_spellings_results() {
    let mut answers = [0_usize; 2];
    for name in ["startswith", "endswith"] {
        for (setup, text) in TEXTS {
            for affix in AFFIXES {
                let alias = format!("{setup}\n({text}).{name}({affix})");
                let spelled = format!("{setup}\n({text}).{}({affix})", accepted(name));
                let found = fresh(&alias);
                assert_eq!(found, fresh(&spelled), "{alias}");
                match found.as_str() {
                    "true" => answers[0] += 1,
                    "false" => answers[1] += 1,
                    other => panic!("{alias}: {other}"),
                }
            }
        }
    }
    assert_eq!(
        answers.iter().sum::<usize>(),
        2 * TEXTS.len() * AFFIXES.len()
    );
    assert!(answers.iter().all(|count| *count > 50), "{answers:?}");
    for (source, expected) in [
        ("\"report.csv\".startswith(\"report\")", "true"),
        ("\"report.csv\".endswith(\".csv\")", "true"),
        ("\"report.csv\".startswith(\"Report\")", "false"),
        ("\"report.csv\".endswith(\"report\")", "false"),
        ("\"abc\".startswith(\"b\", 1)", "false"),
        ("\"abc\".endswith(\"b\", 0, 2)", "false"),
        ("\"null value\".startswith(null)", "true"),
        ("\"undefined value\".startswith()", "true"),
        ("\"a.csv\".endswith([\".csv\", \".tsv\"])", "false"),
        ("\"x.csv,.tsv\".endswith([\".csv\", \".tsv\"])", "true"),
        ("\"12abc\".startswith(12)", "true"),
        ("\"\".endswith(\"\")", "true"),
        ("\"café\".endswith(\"é\")", "true"),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
}

#[test]
fn join_on_a_string_returns_the_list_join_of_its_argument() {
    let mut joined = 0;
    for separator in SEPARATORS {
        for (setup, items) in LISTS {
            let alias = format!("{POPULATION}\n{setup}\n{separator}.join({items})");
            let spelled = format!("{POPULATION}\n{setup}\n({items}).join({separator})");
            let found = fresh(&alias);
            assert_eq!(found, fresh(&spelled), "{alias}");
            assert!(found.starts_with('"'), "{alias}: {found}");
            joined += 1;
        }
    }
    assert_eq!(joined, SEPARATORS.len() * LISTS.len());
    for (source, expected) in [
        ("\", \".join([\"ada\", \"bo\"])", "\"ada, bo\""),
        (
            "\"-\".join([\"a\", 1, null, true, [1, 2]])",
            "\"a-1-null-true-1,2\"",
        ),
        ("\"-\".join([])", "\"\""),
        ("\"\".join([\"a\", \"b\"])", "\"ab\""),
        ("\"-\".join([\"a\", \"b\"], 1)", "\"a-b\""),
        ("class Empty\n\"-\".join(Empty)", "\"\""),
        (
            "class Tag(n):\n    this.n = n\nred = Tag(1)\nblue = Tag(2)\n\"/\".join(Tag)",
            "\"[object red]/[object blue]\"",
        ),
        (
            "class Tag(n):\n    this.n = n\nTag(1)\nTag(2)\nf = x -> \"-\".join(x)\ng = x -> x.join(\"-\")\n[f(Tag) == g(Tag), f([1, 2])]",
            "[true,\"1-2\"]",
        ),
        (
            "class Object(n):\n    this.n = n\nObject(1)\n\"-\".join(Object) == Object.join(\"-\")",
            "true",
        ),
        (
            "class P(n):\n    this.n = n\nP(1)\nkind = P\ndelete P\n[\"-\".join(kind) == kind.join(\"-\"), kind.length]",
            "[true,1.0]",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for name in [
        "Boolean", "Object", "Class", "List", "Function", "JSON", "RegExp",
    ] {
        let declared = format!("class {name}(n):\n    this.n = n\n{name}(1)\n{name}(2)");
        let found = fresh(&format!("{declared}\n\"-\".join({name})"));
        assert_eq!(
            found,
            fresh(&format!("{declared}\n{name}.join(\"-\")")),
            "{name}"
        );
        assert_eq!(found, "\"[object obj:0]-[object obj:1]\"", "{name}");
    }
}

#[test]
fn join_keeps_the_accepted_type_error_for_every_other_argument() {
    let classes = |name: &str| format!("class {name}(n):\n    this.n = n\n{name}(1)");
    let mut checked = 0;
    for (setup, argument) in [
        (String::new(), "\"abc\""),
        (String::new(), "\"\""),
        (String::new(), "5"),
        (String::new(), "0 / 0"),
        (String::new(), "true"),
        (String::new(), "{k: 1}"),
        (
            "class Tag(n):\n    this.n = n\nred = Tag(1)".to_string(),
            "red",
        ),
        (String::new(), "null"),
        (String::new(), "undefined"),
        ("o = {}".to_string(), "o.missing"),
        ("xs = null".to_string(), "xs"),
        (String::new(), "x -> x"),
        (String::new(), "/a/"),
        (String::new(), "Date(0)"),
        (String::new(), ""),
        (String::new(), "Math"),
        (String::new(), "Number"),
        (String::new(), "String"),
        (String::new(), "Date"),
        (String::new(), "Object"),
        (String::new(), "List"),
        (String::new(), "Boolean"),
        (String::new(), "Class"),
        (String::new(), "JSON"),
        (String::new(), "RegExp"),
        (String::new(), "Function"),
        (classes("Math"), "Math"),
        (classes("Number"), "Number"),
        (classes("String"), "String"),
        (classes("Date"), "Date"),
        (format!("{}\nkind = Math", classes("Math")), "kind"),
    ] {
        let source = format!("{setup}\n\"-\".join({argument})");
        let line = source.lines().count();
        assert_eq!(
            failure(&source),
            format!("TypeError: -.join is not a function (line {line}, column 1)"),
            "{source}"
        );
        checked += 1;
    }
    assert_eq!(checked, 31);
    for name in ["Math", "Number", "String", "Date"] {
        let declared = classes(name);
        assert_eq!(
            failure(&format!("{declared}\n{name}.join(\"-\")")),
            format!("TypeError: {name}.join is not a function (line 4, column 1)"),
            "{name}"
        );
    }
    assert_eq!(
        failure("class Tag(n):\n    this.n = n\n$Tag.label = \"-\".join($Tag)\nTag(1)"),
        "TypeError: -.join is not a function (line 4, column 1)"
    );
    assert_eq!(
        failure("sep = \"+\"\nsep.join(\"ab\")"),
        "TypeError: +.join is not a function (line 2, column 1)"
    );
    assert_eq!(
        failure("x = 5\nx.join([\"a\"])"),
        "TypeError: number.join is not a function (line 2, column 1)"
    );
    assert_eq!(
        failure("\"-\".join(missing)"),
        "ReferenceError: missing is not defined (line 1, column 1)"
    );
}

#[test]
fn unchanged_routes_keep_their_accepted_results() {
    for (source, expected) in [
        (
            "s = \"abc\"\n[s.startswith, s.endswith, s.join]",
            "[null,null,null]",
        ),
        (
            "n = null\n[n.startswith(\"a\"), n.endswith(\"a\"), n.join([\"x\"])]",
            "[null,null,null]",
        ),
        (
            "calls = []\nn = null\nn.join(calls.push(1))\nn.startswith(calls.push(2))\ncalls",
            "[]",
        ),
        (
            "o = {}\n[o.n.startswith(\"a\"), o.n.join([\"x\"])]",
            "[null,null]",
        ),
        (
            "o = {startswith: x -> 1, endswith: x -> 2, join: x -> 3}\n[o.startswith(\"a\"), o.endswith(\"a\"), o.join([\"x\"])]",
            "[1.0,2.0,3.0]",
        ),
        (
            "o = {}\n[o.startswith(\"a\"), o.endswith(\"a\"), o.join([\"x\"])]",
            "[null,null,null]",
        ),
        (
            "def startswith(x) => 1\ndef endswith(x) => 2\ndef join(x) => 3\n[startswith(\"a\"), endswith(\"a\"), join([\"x\"])]",
            "[1.0,2.0,3.0]",
        ),
        (
            "class G:\n    def join(x) => x\n    def startswith(x) => x + 1\ng = G()\n[g.join(1), g.startswith(1), G().join(5)]",
            "[1.0,2.0,5.0]",
        ),
        (
            "class Bag:\n    def join(x) => 9\nbag = Bag()\nbag.join([\"a\"])",
            "9.0",
        ),
        (
            "class Doc(name):\n    this.name = name\nd = Doc(\"a.csv\")\n[d.startswith(\"a\"), d.join(\"-\")]",
            "[null,null]",
        ),
        (
            "xs = [\"a\", \"b\"]\nclass Tag(n):\n    this.n = n\nTag(1)\n[xs.join(\", \"), Tag.join(\"-\"), xs.join(), [].join(\"-\")]",
            "[\"a, b\",\"[object obj:0]\",\"ab\",\"\"]",
        ),
        (
            "s = \"report.csv\"\ns = \"notes.txt\"\ns@first.startsWith(\"rep\")",
            "true",
        ),
    ] {
        assert_eq!(result(source), expected, "{source}");
    }
    for (source, expected) in [
        (
            "s = \"abc\"\ns.endswith = 1",
            "TypeError: Cannot assign 'endswith' on string (line 2, column 1)",
        ),
        (
            "s = \"abc\"\ns.startswith = 1",
            "TypeError: Cannot assign 'startswith' on string (line 2, column 1)",
        ),
        (
            "s = \"abc\"\ns.join = 1",
            "TypeError: Cannot assign 'join' on string (line 2, column 1)",
        ),
        (
            "String.startswith(\"a\")",
            "TypeError: String.startswith is not a function (line 1, column 1)",
        ),
        (
            "String.endswith(\"a\")",
            "TypeError: String.endswith is not a function (line 1, column 1)",
        ),
        (
            "String.join(\", \", [\"a\"])",
            "TypeError: String.join is not a function (line 1, column 1)",
        ),
        (
            "xs = [\"a\"]\nxs.startswith(\"a\")",
            "TypeError: xs.startswith is not a function (line 2, column 1)",
        ),
        (
            "xs = [\"a\"]\nxs.endswith(\"a\")",
            "TypeError: xs.endswith is not a function (line 2, column 1)",
        ),
        (
            "x = 5\nx.startswith(\"5\")",
            "TypeError: number.startswith is not a function (line 2, column 1)",
        ),
        (
            "true.endswith(\"e\")",
            "TypeError: boolean.endswith is not a function (line 1, column 1)",
        ),
        (
            "Date(0).startswith(\"1\")",
            "TypeError: Date.startswith is not a function (line 1, column 1)",
        ),
        (
            "r = /a/\nr.startswith(\"a\")",
            "TypeError: startswith is not a function (line 2, column 1)",
        ),
        (
            "f = x -> x\nf.startswith(\"a\")",
            "TypeError: function.startswith is not a function (line 2, column 1)",
        ),
        (
            "class Tag(n):\n    this.n = n\nTag(1)\nTag.startswith(\"a\")",
            "TypeError: Tag.startswith is not a function (line 4, column 1)",
        ),
        (
            "class Tag(n):\n    this.n = n\nTag.endswith(\"a\")",
            "TypeError: Tag.endswith is not a function (line 3, column 1)",
        ),
        (
            "Math.startswith(\"a\")",
            "TypeError: Math.startswith is not a function (line 1, column 1)",
        ),
        (
            "startswith(\"a\")",
            "ReferenceError: startswith is not defined (line 1, column 1)",
        ),
        (
            "s = \"abc\"\nsum of s",
            "TypeError: abc.sum is not a function (line 2, column 1)",
        ),
    ] {
        assert_eq!(failure(source), expected, "{source}");
    }
}

#[test]
fn the_public_string_method_takes_the_two_affix_aliases() {
    let suffix = [Value::String(".csv".to_string())];
    assert_eq!(
        nucleoid::builtins::string_method("report.csv", "endswith", &suffix).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        nucleoid::builtins::string_method("report.csv", "startswith", &suffix).unwrap(),
        Value::Bool(false)
    );
    let error = nucleoid::builtins::string_method("-", "join", &[Value::List(Vec::new())])
        .expect_err("join with a list argument is dispatched by the runtime");
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "-.join is not a function");
}

#[test]
fn declarations_follow_their_receivers_and_arguments_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            ("names = [\"ada\"]", "names = [\"ada\"]"),
            ("sep = \", \"", "sep = \", \""),
            ("label = sep.join(names)", "label = names.join(sep)"),
            ("plain = \"-\".join(names)", "plain = names.join(\"-\")"),
            ("names.push(\"bo\")", "names.push(\"bo\")"),
            ("[label, plain]", "[label, plain]"),
            ("sep = \" / \"", "sep = \" / \""),
            ("names.push(\"cy\", \"di\")", "names.push(\"cy\", \"di\")"),
            ("[label, plain]", "[label, plain]"),
            ("names.pop()", "names.pop()"),
            ("names.shift()", "names.shift()"),
            ("[label, plain]", "[label, plain]"),
            ("names = [\"x\", \"y\"]", "names = [\"x\", \"y\"]"),
            ("[label, plain]", "[label, plain]"),
            ("names = []", "names = []"),
            ("[label, plain]", "[label, plain]"),
        ],
        &["label", "plain"],
    );
    behaves_as_spelled(
        &[
            (
                "class Tag(n):\n    this.n = n",
                "class Tag(n):\n    this.n = n",
            ),
            ("label = \"-\".join(Tag)", "label = Tag.join(\"-\")"),
            (
                "names = \",\".join(Tag.map(tag -> tag.n))",
                "names = Tag.map(tag -> tag.n).join(\",\")",
            ),
            ("[label, names]", "[label, names]"),
            ("t = Tag(1)", "t = Tag(1)"),
            ("u = Tag(2)", "u = Tag(2)"),
            ("[label, names]", "[label, names]"),
            ("u.n = 5", "u.n = 5"),
            ("[label, names]", "[label, names]"),
            ("delete u.n", "delete u.n"),
            ("delete u", "delete u"),
            ("[label, names]", "[label, names]"),
        ],
        &["label", "names"],
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "names = [\"ada\"]\nlabel = \", \".join(names)\nassert(label, \"ada\")\nnames.push(\"bo\")\nassert(label, \"ada, bo\")\nnames.shift()\nassert(label, \"bo\")",
    );
    assert_eq!(dependencies(&runtime, "label"), ["names"]);
}

#[test]
fn class_level_rules_follow_each_instance_as_their_spellings_do() {
    behaves_as_spelled(
        &[
            (
                "class File(name):\n    this.name = name",
                "class File(name):\n    this.name = name",
            ),
            (
                "$File.table = $File.name.endswith(\".csv\")",
                "$File.table = $File.name.endsWith(\".csv\")",
            ),
            (
                "$File.draft = $File.name.startswith(\"draft_\")",
                "$File.draft = $File.name.startsWith(\"draft_\")",
            ),
            ("f = File(\"draft_a.csv\")", "f = File(\"draft_a.csv\")"),
            ("g = File(\"b.txt\")", "g = File(\"b.txt\")"),
            (
                "[f.table, f.draft, g.table, g.draft]",
                "[f.table, f.draft, g.table, g.draft]",
            ),
            ("f.name = \"b.txt\"", "f.name = \"b.txt\""),
            ("g.name = \"draft_c.csv\"", "g.name = \"draft_c.csv\""),
            (
                "[f.table, f.draft, g.table, g.draft]",
                "[f.table, f.draft, g.table, g.draft]",
            ),
            (
                "tables = \", \".join(File.filter(file -> file.table).map(file -> file.name))",
                "tables = File.filter(file -> file.table).map(file -> file.name).join(\", \")",
            ),
            ("File(\"x.csv\")", "File(\"x.csv\")"),
            ("f.name = \"f.csv\"", "f.name = \"f.csv\""),
            ("tables", "tables"),
        ],
        &["tables"],
    );
    behaves_as_spelled(
        &[
            (
                "class Order(items):\n    this.items = items",
                "class Order(items):\n    this.items = items",
            ),
            (
                "$Order.summary = \", \".join($Order.items)",
                "$Order.summary = $Order.items.join(\", \")",
            ),
            ("o = Order([\"tea\"])", "o = Order([\"tea\"])"),
            ("p = Order([])", "p = Order([])"),
            ("o.items.push(\"jam\")", "o.items.push(\"jam\")"),
            ("[o.summary, p.summary]", "[o.summary, p.summary]"),
            ("o.items = [\"rye\"]", "o.items = [\"rye\"]"),
            ("p.items.push(\"oat\")", "p.items.push(\"oat\")"),
            ("[o.summary, p.summary]", "[o.summary, p.summary]"),
        ],
        &[],
    );
    assert_eq!(
        result(
            "class Order(code):\n    this.code = code\n$Order.rush = $Order.code.startswith(\"RUSH-\")\na = Order(\"RUSH-1\")\nb = Order(\"STD-2\")\nb.code = \"RUSH-2\"\n[a.rush, b.rush]"
        ),
        "[true,true]"
    );
}

#[test]
fn explanations_name_the_dependencies_of_the_spelling_in_reading_order() {
    let setup = "names = [\"ada\"]\nsep = \", \"\n";
    let mut alias = Runtime::new();
    let mut spelled = Runtime::new();
    alias
        .run(&format!("{setup}label = sep.join(names)\nplain = \"-\".join(names)\nfile = \"a.csv\"\nkind = file.endswith(\".csv\")"))
        .unwrap();
    spelled
        .run(&format!("{setup}label = names.join(sep)\nplain = names.join(\"-\")\nfile = \"a.csv\"\nkind = file.endsWith(\".csv\")"))
        .unwrap();
    assert_eq!(
        json(&mut alias, "(why label)[0].rule"),
        "\"label = sep.join(names)\""
    );
    assert_eq!(
        json(&mut alias, "(why plain)[0].rule"),
        "\"plain = \\\"-\\\".join(names)\""
    );
    assert_eq!(
        json(&mut alias, "(why kind)[0].rule"),
        "\"kind = file.endswith(\\\".csv\\\")\""
    );
    assert_eq!(
        json(&mut alias, "(why label)[0].from"),
        "[\"sep\",\"names\"]"
    );
    assert_eq!(
        json(&mut spelled, "(why label)[0].from"),
        "[\"names\",\"sep\"]"
    );
    assert_eq!(
        json(&mut alias, "(why label).map(step -> step.node)"),
        "[\"label\",\"sep\",\"names\"]"
    );
    assert_eq!(
        json(&mut spelled, "(why label).map(step -> step.node)"),
        "[\"label\",\"names\",\"sep\"]"
    );
    assert_eq!(dependencies(&alias, "label"), ["sep", "names"]);
    assert_eq!(dependencies(&spelled, "label"), ["names", "sep"]);
    for query in [
        "(why plain).map(step -> step.from)",
        "(why plain).map(step -> step.node)",
        "(why kind).map(step -> step.from)",
        "(affects names).map(step -> step.node)",
        "(affects sep).map(step -> step.node)",
        "(affects file).map(step -> step.node)",
        "why plain |> stated",
        "kind |> why |> derived",
    ] {
        let found = json(&mut alias, query);
        let expected = json(&mut spelled, query)
            .replace("names.join(\\\"-\\\")", "\\\"-\\\".join(names)")
            .replace("file.endsWith(", "file.endswith(");
        assert_eq!(found, expected, "{query}");
    }
    for (alias_text, spelled_text) in [
        ("t = \"a\".startswith(\"b\")", "t = \"a\".startsWith(\"b\")"),
        ("t = s.endswith(\".csv\", 1)", "t = s.endsWith(\".csv\", 1)"),
    ] {
        assert_eq!(
            generate_all(&parse(alias_text).unwrap())
                .replace("startswith", "startsWith")
                .replace("endswith", "endsWith"),
            generate_all(&parse(spelled_text).unwrap())
        );
    }
    assert_eq!(
        generate_all(&parse("t = \"a\".startswith(\"b\")").unwrap()),
        "t=\"a\".startswith(\"b\")"
    );
    assert_eq!(
        generate_all(&parse("t = \", \".join(xs)").unwrap()),
        "t=\", \".join(xs)"
    );
}

#[test]
fn the_receiver_and_then_every_argument_evaluate_once_in_written_order() {
    let functions = "log = []\ndef items():\n    log.push(\"items\")\n    return [\"x\", \"y\"]\ndef sep():\n    log.push(\"sep\")\n    return \"-\"\ndef text():\n    log.push(\"text\")\n    return \"ab\"\ndef prefix():\n    log.push(\"prefix\")\n    return \"a\"\n";
    for (source, expected) in [
        (
            "[sep().join(items()), log]",
            "[\"x-y\",[\"sep\",\"items\"]]",
        ),
        (
            "[items().join(sep()), log]",
            "[\"x-y\",[\"items\",\"sep\"]]",
        ),
        (
            "[text().startswith(prefix()), log]",
            "[true,[\"text\",\"prefix\"]]",
        ),
        (
            "[text().endswith(prefix(), log.push(\"extra\")), log]",
            "[false,[\"text\",\"prefix\",\"extra\"]]",
        ),
        (
            "[sep().join(items(), log.push(\"extra\")), log]",
            "[\"x-y\",[\"sep\",\"items\",\"extra\"]]",
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
            .run("sep().join(text(), log.push(\"extra\"))")
            .unwrap_err()
            .to_string(),
        "TypeError: -.join is not a function (line 1, column 1)"
    );
    assert_eq!(json(&mut runtime, "log"), "[]");
    assert_eq!(
        failure("\"-\".join([\"a\"], missing)"),
        "ReferenceError: missing is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure("\"abc\".startswith(\"a\", missing)"),
        "ReferenceError: missing is not defined (line 1, column 1)"
    );
}

#[test]
fn aliases_read_in_every_statement_position() {
    for (source, expected) in [
        (
            "result = \"\"\ntry:\n    result = \"-\".join(\"abc\")\ncatch error:\n    result = error\nresult",
            "\"TypeError: -.join is not a function\"",
        ),
        (
            "names = [\"a\"]\nlabel = \", \".join(names)\nseen = suppose names.push(\"b\"): label\n[seen, label, names]",
            "[\"a, b\",\"a\",[\"a\"]]",
        ),
        (
            "names = [\"a\", \"b\"]\n`${\", \".join(names)}!`",
            "\"a, b!\"",
        ),
        ("file = \"a.csv\"\n`${file.endswith(\".csv\")}`", "\"true\""),
        (
            "file = \"a.csv\"\nif file.endswith(\".csv\"):\n    kind = \"table\"\nelse:\n    kind = \"text\"\nkind",
            "\"table\"",
        ),
        (
            "file = \"a.txt\"\nfile.endswith(\".csv\") ? \"table\" : \"text\"",
            "\"text\"",
        ),
        (
            "def kind(f) => f.endswith(\".csv\") ? \"table\" : \"text\"\n[kind(\"a.csv\"), kind(\"b.txt\")]",
            "[\"table\",\"text\"]",
        ),
        (
            "def listing(xs):\n    return \", \".join(xs)\nlisting([\"a\", \"b\"])",
            "\"a, b\"",
        ),
        (
            "names = [\"ann\", \"bob\", \"amy\"]\nnames.filter(n -> n.startswith(\"a\")).map(n -> \"-\".join([n, n]))",
            "[\"ann-ann\",\"amy-amy\"]",
        ),
        (
            "files = [\"a.csv\", \"b.txt\"]\nfiles.some(f -> f.endswith(\".txt\")) && files.every(f -> f.startswith(\"\"))",
            "true",
        ),
        (
            "i = 0\nwhile \"-\".join([i, \"\"]).startswith(\"3\") == false:\n    i = i + 1\ni",
            "3.0",
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
        "class File(name):\n    this.name = name\n$File.table = $File.name.endswith(\".csv\")\nreport = File(\"report.csv\")\nfailure = \"\"\ntry:\n    report.name = 7\ncatch error:\n    failure = error\nassert(failure, TypeError(\"number.endswith is not a function\"))\nassert([report.name, report.table], [\"report.csv\", true])",
    );
    checked(
        &mut runtime,
        "names = [\"ada\", \"bo\"]\nlabel = \", \".join(names)\ntry:\n    names = \"ada\"\ncatch error:\n    failure = error\nassert(failure, TypeError(\", .join is not a function\"))\nassert([names, label], [[\"ada\", \"bo\"], \"ada, bo\"])",
    );
    checked(
        &mut runtime,
        "failure = \"\"\ntry:\n    names = null\ncatch error:\n    failure = error\nassert(failure, TypeError(\", .join is not a function\"))\nassert(label, \"ada, bo\")",
    );
    checked(
        &mut runtime,
        "try:\n    names.push(\"cy\")\n    report.name = \"report.txt\"\n    assert([label, report.table], [\"ada, bo, cy\", false])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert([label, report.table, names], [\"ada, bo\", true, [\"ada\", \"bo\"]])",
    );
    assert_eq!(dependencies(&runtime, "label"), ["names"]);
    assert_eq!(
        failure("names = [\"a\"]\nlabel = \", \".join(names)\nnames = 5"),
        "TypeError: , .join is not a function (line 3, column 1)"
    );
    let mut recovered = Runtime::new();
    recovered
        .run("names = [\"a\"]\nlabel = \", \".join(names)")
        .unwrap();
    assert!(recovered.run("names = 5").is_err());
    assert_eq!(json(&mut recovered, "[names, label]"), "[[\"a\"],\"a\"]");
    recovered.run("names.push(\"b\")").unwrap();
    assert_eq!(json(&mut recovered, "label"), "\"a, b\"");
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_a_value() {
    assert_eq!(
        result("try:\n    x = \"a\".startswith(\"a\")\ncatch e:\n    x = \"caught\"\nx"),
        "true"
    );
    assert_eq!(
        result(
            "names = [\"a\", \"b\"]\nr = 0\ntry:\n    r = \", \".join(names)\ncatch error:\n    r = error\nr"
        ),
        "\"a, b\""
    );
    assert_eq!(
        result("r = 0\ntry:\n    r = \", \".join(\"ab\")\ncatch error:\n    r = error\nr"),
        "\"TypeError: , .join is not a function\""
    );
}

#[test]
fn large_receivers_join_through_the_public_api() {
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
                .map(|position| Value::String(format!("i{position}")))
                .collect(),
        ),
    );
    let alias = json(&mut runtime, "\",\".join(holder.items)");
    let spelled = json(&mut runtime, "holder.items.join(\",\")");
    assert_eq!(alias, spelled);
    assert!(alias.starts_with("\"i0,i1,i2,"));
    assert!(alias.ends_with(",i99999\""));
    assert_eq!(
        json(
            &mut runtime,
            "holder.items.filter(item -> item.endswith(\"999\")).length"
        ),
        "100.0"
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-python-string-aliases-{label}-{}.nuc",
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
    source
        .replace(".startswith(", ".startsWith(")
        .replace(".endswith(", ".endsWith(")
        .replace("\", \".join(names)", "names.join(\", \")")
        .replace("\"-\".join(Tag)", "Tag.join(\"-\")")
}

#[test]
fn files_graphs_and_repl_sessions_run_the_aliases() {
    let program = "class Tag(n):\n    this.n = n\nnames = [\"ada\"]\nlabel = \", \".join(names)\nfile = \"a.csv\"\nkind = file.endswith(\".csv\")\ndraft = file.startswith(\"draft_\")\nTag(1)\ntags = \"-\".join(Tag)\nnames.push(\"bo\")\n[label, kind, draft, tags]";
    let spelled = spelled_out(program);
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
        "[\"ada, bo\",true,false,\"[object obj:0]\"]\n"
    );
    let graph = String::from_utf8(file.run(program, &["--graph"]).stdout).unwrap();
    assert!(
        graph.contains("label [variable]\n    reads    names"),
        "{graph}"
    );
    let failed = file.run("xs = [1]\n\"-\".join(\"abc\")\n", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "TypeError: -.join is not a function (line 2, column 1)\n"
    );
    let session = "names = [\"ada\"]\nlabel = \", \".join(names)\nlabel\nnames.push(\"bo\")\nlabel\n\"a.csv\".endswith(\".csv\")\n\"a.csv\".startswith(\"b\")\n.exit\n";
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
                "\"ada\"\n2.0\n\"ada, bo\"\ntrue\nfalse\n"
            } else {
                "ada\n2\nada, bo\ntrue\nfalse\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
        }
    }
    let failing = repl(
        "\"-\".join(\"abc\")\n\"-\".join([\"a\", \"b\"])\n.exit\n",
        &["--plain"],
    );
    assert!(failing.status.success(), "{failing:?}");
    assert_eq!(
        String::from_utf8(failing.stderr).unwrap(),
        "TypeError: -.join is not a function (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(failing.stdout).unwrap();
    assert!(stdout.contains("a-b\n"), "{stdout}");
}
