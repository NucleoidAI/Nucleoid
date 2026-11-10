mod common;

use std::time::{Duration, Instant};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, Quantifier, Stmt, parse, parse_expression, parse_program};
use nucleoid::{ErrorKind, Position, Runtime};

const WORDS: [&str; 2] = ["all", "any"];

const EXAMPLE: &str = "class Object(number):\n    this.number = number\n\nevery Object.label is square\n\nif any Object.number > 4:\n  all Object.color is red\n\na = Object(3)\nb = Object(7)\nassert([a.label, b.label], [\"square\", \"square\"])\nassert([a.color, b.color], [null, \"red\"])\n[a.color, b.color]";

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid spells class references with any and all",
    "Nucleoid writes class level rules and conditionals with all and any",
    "Nucleoid states class predicates with all and any",
    "Nucleoid keeps all and any as names and quantifiers beside class references",
];
const REFERENCE_TITLE: &str = "Nucleoid writes class conditions with any and their rules with all";
const README_TITLE: &str = "Any and all state a class condition and the rule it guards";
const EXAMPLES_TITLE: &str = "Any and all keep every shelf's reorder rule live";

const CLASS_CONTEXTS: [&str; 46] = [
    "{c}.mortal = true",
    "{c}.big = {c}.total > 10",
    "{c}.legs is 2",
    "{c}.tag is {c}.name",
    "{c}.ok = all v in xs: v > {c}.total",
    "if {c}.total > 10:\n    {c}.big = true",
    "if {c}.total > 10:\n    {c}.big = true\nelse:\n    {c}.big = false",
    "if {c}.total > 10\n    {c}.big = true",
    "if {c}:\n    r = 1",
    "while {c}.open:\n    r = 1",
    "r = {c}",
    "r = [{c}, 1]",
    "r = {k: {c}}",
    "r = {k: {c}, j: 1}",
    "r = {k: {c} j: 1}",
    "r = f({c})",
    "r = ({c})",
    "r = not {c}",
    "r = 1 + {c}.total",
    "r = c ? {c} : 1",
    "r = c ? 1 : {c}",
    "r = {c} if c else 1",
    "r = 1 if {c} else 2",
    "r = {c} ?? 1",
    "r = why {c}.total",
    "r = sum of {c}",
    "r = all v in {c}: v",
    "r = all v in xs: {c}",
    "r = xs.map(v => {c})",
    "r = lambda: {c}",
    "r = [{c} for v in xs]",
    "r = [v for v in xs if {c}]",
    "r = xs[{c}]",
    "r = xs[{c}:]",
    "r = `${({c})}`",
    "r is {c}",
    "r is [{c}]",
    "assert: {c}.mortal, true",
    "{ {c}.mortal = true }",
    "def g() => {c}.total",
    "x0 = 0 {c}.mortal = true",
    "r = x |> why {c}.mortal = true",
    "r = x ?? y {c}.mortal = true",
    "r = new {c}",
    "{c}@first",
    "{c}.total@first",
];

const PROTECTED: [&str; 92] = [
    "xs = [1, 2]\nr = xs.{w}(v => v > 0)",
    "r = x.{w}",
    "r = x.{w} y",
    "{w} = 3",
    "{w} = 3\nr = {w}.x",
    "r = {w}(1)",
    "r = [{w}, x]",
    "r = [{w}]",
    "for {w} in xs:\n    r = {w}",
    "for {w} of xs:\n    pass",
    "def {w}(x) => x",
    "def {w} x => x",
    "def f {w} x => {w} + x",
    "def f({w}, x) => {w}",
    "def f {w}, x => {w}",
    "def f {w}:\n    return {w}",
    "r = ({w}, x) => {w}",
    "r = {w} => {w}",
    "r = lambda {w}: {w}",
    "r = lambda {w}, x: {w}",
    "{w}: int",
    "r = {{w}: 1}",
    "r = {{w}}",
    "r = {k: 1, {w}: 2}",
    "r = {k: {w}}",
    "r = {k: {w} x: 1}",
    "r = {k: {w}, x: 1}",
    "r = {k: c ? 1 : {w} x: 2}",
    "r = f({w}=1)",
    "class Item({w}):\n    this.{w} = {w}\nr = Item({w}=1)",
    "{{w}} = o",
    "{k: {w}} = o",
    "{w}, x = [1, 2]",
    "class {w}:\n    pass",
    "class K({w}):\n    this.{w} = {w}",
    "try:\n    x = 1\ncatch {w}:\n    pass",
    "r = every {w} in xs: {w} > 0",
    "r = [{w} for {w} in xs]",
    "enum E: {w}, other",
    "{w} is 3",
    "{w} is not 3",
    "{w} is",
    "r = ({w} is 3)",
    "if {w} is 3:\n    r = 1",
    "{w} x: int",
    "{w} assert: 1, 1",
    "{w} suppose:\n    x = 1",
    "{w} lambda: 1",
    "r = `${{w} x}`",
    "r = `${{w} v in xs}`",
    "r = `${1 + {w} x}`",
    "{w}\nx = 1",
    "{w} $Human",
    "{w} 1",
    "{w} \"s\"",
    "{w} (x)",
    "{w} [x]",
    "r = {w}.{w}",
    "r = {w} ?? 1",
    "r = {w} |> why",
    "r = x |> {w}",
    "{w}@first",
    "r = {w} if c else x",
    "r = Member.{w}()",
    "r = Member.{w}().length",
    "def h {w} other => {w} * other",
    "r = ({w}, other) => {w} + other",
    "r = {a: {w} b: b}",
    "r = all {w} in [1, 2]: {w} > 0",
    "r = any {w} in [0, 2]: {w}",
    "r = all n in {w}: n > other",
    "r = any n in {w}: n > other",
    "{w} = true\nr = {w} and not other",
    "r = {w}[0]",
    "assert: {w}, 7",
    "r = {{w}, other}",
    "r = `${({w})}`",
    "r = not {w}",
    "r = -{w}",
    "r = {w} + x",
    "r = [{w}, {w}]",
    "r = f({w}, x)",
    "if {w}:\n    r = 1",
    "if {w}\n    r = 1",
    "while {w}:\n    r = 1",
    "r = c ? {w} : 2",
    "{w} += 1",
    "{w} := 1",
    "{w}.x = 1",
    "{w}[0] = 1",
    "delete {w}",
    "throw {w}",
];

const STATEMENT_POSITIONS: [&str; 29] = [
    "{s}",
    "z0 = 0; {s}",
    "z0 = 0 {s}",
    "{s}; z1 = 1",
    "{s}\nz1 = 1",
    "{s}; {s}",
    "{s}\n{s}",
    "if c:\n{i}",
    "if c: {s}",
    "if c\n{i}",
    "if c:\n    pass\nelse:\n{i}",
    "if c:\n    pass\nelif d:\n{i}",
    "if c: {s} else: {s}",
    "for v in xs:\n{i}",
    "while c:\n{i}\n    c = false",
    "try:\n{i}\ncatch e:\n    pass",
    "try:\n    pass\ncatch e:\n{i}",
    "suppose:\n{i}",
    "def g():\n{i}\n    return 1",
    "function g() { {s} }",
    "class K:\n{i}",
    "class K:\n    def m():\n    {i}",
    "class K: {s}",
    "{ {s} }",
    "r = () => { {s} }",
    "r = f(() => { {s} })",
    "r = f(() => {\n{s};\nreturn 1\n})",
    "r = f(() => {\nz2 = 2\n{s}\n})",
    "r = `${(() => { {s} })()}`",
];

const JUXTAPOSED_SENTENCES: [(&str, &str); 22] = [
    ("{s} is mortal z1 = 1", "$Human\nis\nmortal\nz1 = 1"),
    (
        "{s} is mortal {s} is wise",
        "$Human\nis\nmortal\n$Human.wise = true",
    ),
    ("{s} is mortal x", "$Human\nis\nmortal\nx"),
    (
        "{s} is mortal assert: 1, 1",
        "$Human\nis\nmortal\nassert: 1, 1",
    ),
    (
        "{s} is mortal if c:\n    z1 = 1",
        "$Human\nis\nmortal\nif c:\n    z1 = 1",
    ),
    ("{s} is 5", "$Human\nis\n5"),
    ("{s} is true", "$Human\nis\ntrue"),
    ("{s} is \"mortal\"", "$Human\nis\n\"mortal\""),
    ("{s} is (mortal)", "$Human\nis(mortal)"),
    ("{s} is not mortal", "$Human\nis\nnot mortal"),
    ("{s} is mortal and wise", "$Human\nis\nmortal and wise"),
    ("{s} is mortal.x", "$Human\nis\nmortal.x"),
    ("{s} is mortal = 1", "$Human\nis\nmortal = 1"),
    ("{s} is mortal := 1", "$Human\nis\nmortal := 1"),
    ("{s} is mortal: int", "$Human\nis\nmortal: int"),
    ("{s} is mortal is wise", "$Human\nis\nmortal is wise"),
    ("{s} is is wise", "$Human\nis is wise"),
    ("{s} is all v in xs: v", "$Human\nis\nall v in xs: v"),
    ("{s} is any Order", "$Human\nis\nany Order"),
    ("{s} is", "$Human\nis"),
    ("{s} is\nmortal", "$Human\nis\nmortal"),
    ("x = {s} is mortal", "x = $Human\nis\nmortal"),
];

const NAME_JUXTAPOSITIONS: [(&str, &str, &str); 10] = [
    ("{w} Human = 4", "{w}\nHuman = 4", "$Human = 4"),
    ("r = {w} x", "r = {w}\nx", "r = $x"),
    ("r = v => {w} x", "r = v => {w}\nx", "r = v => $x"),
    ("why {w} x", "why {w}\nx", "why $x"),
    ("not {w} x.y", "not {w}\nx.y", "not $x.y"),
    ("{w} Order(5)", "{w}\nOrder(5)", "$Order(5)"),
    ("x0 = 0 {w} x", "x0 = 0 {w}\nx", "x0 = 0 $x"),
    ("r = {w} /* c */ x", "r = {w}\nx", "r = $x"),
    (
        "r = x |> why {w} y",
        "r = x |> why {w}\ny",
        "r = x |> why $y",
    ),
    ("r = x ?? {w} y", "r = x ?? {w}\ny", "r = x ?? $y"),
];

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn rendered(source: &str) -> String {
    generate_all(&statements(source))
}

fn place(position: &str, statement: &str) -> String {
    let indented = statement
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    position.replace("{i}", &indented).replace("{s}", statement)
}

fn renamed(template: &str, word: &str) -> String {
    format!("{:?}", parse(&template.replace("{w}", "qq"))).replace("qq", word)
}

fn fresh(source: &str, expected: &str) {
    expect(&mut Runtime::new(), source, expected);
}

fn expect(runtime: &mut Runtime, source: &str, expected: &str) {
    let parsed = statements(source);
    let assertions = common::assertions::protected_assertions(&parsed).len();
    let case = common::Case {
        title: "any and all class reference regression".to_string(),
        source: source.to_string(),
        expected: Some(expected.to_string()),
    };
    common::check_with_count(runtime, &case, assertions)
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn failure(source: &str) -> (ErrorKind, String, Option<Position>) {
    let error = Runtime::new().run(source).unwrap_err();
    (error.kind(), error.message(), error.position())
}

fn read(runtime: &mut Runtime, source: &str) -> String {
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    runtime.serialize_json(&value).unwrap()
}

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn class(name: &str) -> Expr {
    Expr::ClassRef(name.to_string())
}

fn predicate(subject: &str, property: &str) -> Stmt {
    Stmt::Assign {
        target: Expr::Member {
            object: Box::new(class(subject)),
            property: property.to_string(),
        },
        value: Expr::Bool(true),
    }
}

fn spells_class(source: &str) -> bool {
    let tokens: Vec<Token> = tokenize(source)
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    tokens.windows(3).any(|window| {
        matches!(
            window,
            [Token::Identifier(word), Token::Identifier(name), next]
                if WORDS.contains(&word.as_str())
                    && name != "is"
                    && Quantifier::starting(&window[0], &window[1], next).is_none()
        )
    })
}

fn graph_of(runtime: &Runtime) -> Vec<(String, String, Vec<String>)> {
    runtime
        .graph
        .nodes()
        .map(|node| {
            (
                node.key.to_string(),
                node.kind.as_str().to_string(),
                node.dependencies.iter().map(ToString::to_string).collect(),
            )
        })
        .collect()
}

#[test]
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let readme = common::snippets(include_str!("../README.md"), "README.md");
    let examples = common::snippets(include_str!("../docs/examples.md"), "docs/examples.md");
    let appended = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (&reference, REFERENCE_TITLE),
        (&readme, README_TITLE),
        (&examples, EXAMPLES_TITLE),
    ]);
    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is added exactly once");
        let case = matching[0];
        assert!(spells_class(&case.source), "{title} spells a class");
        let count = common::assertions::protected_assertions(&statements(&case.source)).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 24);
}

#[test]
fn the_users_example_tests_any_in_the_condition_and_sets_all_in_its_rule() {
    fresh(EXAMPLE, "[null,\"red\"]");
    assert_eq!(
        statements("if any Object.number > 4:\n  all Object.color is red"),
        statements("if $Object.number > 4:\n  $Object.color is red")
    );
    assert_eq!(
        statements("if any Object.number > 4\n  all Object.color is red"),
        statements("if $Object.number > 4:\n  $Object.color is red")
    );
    assert_eq!(
        statements("all Object.color is red"),
        statements("$Object.color is red")
    );
    for sentence in [
        "all Object is red",
        "any Object is red",
        "(all Object) is red",
        "$Object is red",
        "$Object.red = true",
    ] {
        assert_eq!(
            statements(sentence),
            [predicate("Object", "red")],
            "{sentence}"
        );
    }
    assert_eq!(
        statements("all human is mortal"),
        [predicate("human", "mortal")]
    );
    fresh(
        "class human\nany human is mortal\nh = human()\nassert(h.mortal, true)\nh.mortal",
        "true",
    );
    assert_eq!(
        failure("class Human\nall human is mortal"),
        (
            ErrorKind::Reference,
            "human is not defined".to_string(),
            Some(Position::new(2, 1))
        )
    );
}

#[test]
fn any_and_all_names_build_exactly_the_tree_of_their_class_references() {
    let mut compared = 0;
    for context in CLASS_CONTEXTS {
        for word in WORDS {
            for (spelled, reference) in [("Order", "$Order"), ("order", "$order")] {
                let source = context.replace("{c}", &format!("{word} {spelled}"));
                let expected = context.replace("{c}", reference);
                assert_eq!(statements(&source), statements(&expected), "{source}");
                assert_eq!(rendered(&source), rendered(&expected), "{source}");
                compared += 1;
            }
        }
    }
    assert_eq!(compared, 184);
    for word in WORDS {
        assert_eq!(
            parse_expression(&format!("{word} Human")).unwrap(),
            class("Human")
        );
        assert_eq!(
            parse_expression(&format!("{word} Human.mortal")).unwrap(),
            parse_expression("$Human.mortal").unwrap()
        );
        assert_eq!(
            parse_expression(&format!("{word} Human is mortal")).unwrap(),
            class("Human")
        );
        assert_eq!(
            parse_expression(&format!("{word} /* note */ Human")).unwrap(),
            class("Human")
        );
        assert_eq!(
            statements(&format!("r = ({word}\nHuman)")),
            statements("r = ($Human)")
        );
        assert_eq!(
            statements(&format!("{word} Human.mortal = true")),
            statements("every Human.mortal = true")
        );
    }
    assert_eq!(
        statements("all Order.big = any Order.total > every Order.limit"),
        statements("$Order.big = $Order.total > $Order.limit")
    );
}

#[test]
fn quantifiers_keep_their_priority_over_class_references() {
    for (source, quantifier) in [
        ("all x in xs: x", Quantifier::All),
        ("any x in xs: x", Quantifier::Any),
        ("all Order in xs: Order", Quantifier::All),
        ("any all in xs: all", Quantifier::Any),
    ] {
        let parsed = parse_expression(source).unwrap();
        assert!(
            matches!(&parsed, Expr::Quantified { quantifier: found, .. } if *found == quantifier),
            "{source}: {parsed:?}"
        );
    }
    assert_eq!(
        statements("r = all v in all Order: v"),
        statements("r = all v in $Order: v")
    );
    assert_eq!(
        statements("r = any v in xs: all Order.total > v"),
        statements("r = any v in xs: $Order.total > v")
    );
    fresh(
        "all = [1, 2, 3]\nany = 2\nr = all n in all: n > any\nassert(r, false)\nassert(any n in all: n > any, true)\nr",
        "false",
    );
    fresh(
        "class Task(done):\n    this.done = done\nTask(true)\nTask(false)\nassert([all t in Task: t.done, any t in Task: t.done], [false, true])\nall Task.open = not all Task.done\nall t in Task: t.open == not t.done",
        "true",
    );
}

#[test]
fn template_interpolations_keep_any_and_all_at_their_top_level() {
    for word in WORDS {
        fresh(
            &format!("{word} = 5\nx = 1\nt = `${{{word} x}}`\nassert(t, \"5\")\nt"),
            "\"5\"",
        );
        fresh(
            &format!("{word} = 5\nxs = [1]\nt = `${{{word} v in xs}}`\nassert(t, \"5\")\nt"),
            "\"5\"",
        );
        fresh(
            &format!("{word} = 5\nx = 1\nt = `${{1 + {word} x}}`\nassert(t, \"6\")\nt"),
            "\"6\"",
        );
        fresh(
            &format!("class Human\nt = `${{({word} Human)}}`\nassert(t, `${{($Human)}}`)\nt"),
            "\"class Human\"",
        );
    }
}

#[test]
fn any_and_all_keep_their_accepted_reading_unless_a_name_follows_them_where_an_operand_begins() {
    let mut compared = 0;
    for template in PROTECTED {
        assert!(!template.contains("qq"), "{template}");
        for word in WORDS {
            let source = template.replace("{w}", word);
            assert_eq!(
                format!("{:?}", parse(&source)),
                renamed(template, word),
                "{source}"
            );
            compared += 1;
        }
    }
    assert_eq!(compared, 184);
    for word in WORDS {
        assert_eq!(
            statements(&format!("{word} x: int")),
            [
                Stmt::Expression(name(word)),
                Stmt::Declaration {
                    name: "x".to_string(),
                    type_name: "int".to_string(),
                }
            ]
        );
        assert_eq!(
            statements(&format!("r = {{k: {word} x: 1}}")),
            statements(&format!("r = {{k: {word}, x: 1}}"))
        );
        assert_eq!(
            parse_expression(&format!("{word} is 3")).unwrap(),
            name(word)
        );
        assert_eq!(
            parse_expression(&format!("{word} x: 1")).unwrap(),
            name(word)
        );
        assert_eq!(parse_expression(word).unwrap(), name(word));
        fresh(
            &format!(
                "{word} = 2\no = {{k: {word} x: 1}}\nassert(o, {{k: 2, x: 1}})\n{word} is 7\nassert(o, {{k: 7, x: 1}})\nassert([{word}, ({word} is 7)], [7, true])\n{word} assert: 1, 1\n{word}"
            ),
            "7",
        );
    }
    fresh(
        "all = [1, 2]\nany = 3\nassert([all.length, all.every(v => v > 0), [1, 2].any(v => v > 1)], [2, true, true])\nassert(all and any, 3)\nr = {all, any}\nassert(r, {all: [1, 2], any: 3})\nall ?? 9",
        "[1,2]",
    );
    fresh(
        "def all(xs) => xs.length\ndef any(xs) => xs.length > 0\ndef h all any => all * any\nf = (all, any) => all + any\n[all([1, 2]), any([]), h(2, 3), f(1, 2)]",
        "[2,false,6,3]",
    );
    fresh(
        "class Seen(size):\n    this.size = size\na = Seen(1)\nb = Seen(2)\nassert(Seen.all().length, 2)\nfor all in Seen:\n    all.doubled = all.size * 2\n[a.doubled, b.doubled]",
        "[2,4]",
    );
}

#[test]
fn any_and_all_followed_by_a_name_were_juxtapositions_and_now_read_the_class_reference() {
    for word in WORDS {
        for (source, juxtaposed, reference) in NAME_JUXTAPOSITIONS {
            let source = source.replace("{w}", word);
            let juxtaposed = juxtaposed.replace("{w}", word);
            assert_ne!(statements(&source), statements(&juxtaposed), "{source}");
            assert_eq!(statements(&source), statements(reference), "{source}");
        }
        assert_eq!(
            failure(&format!("{word} = 3\n{word} Human = 4")),
            (
                ErrorKind::Syntax,
                "Cannot assign to $Human".to_string(),
                Some(Position::new(2, 1))
            )
        );
        assert_eq!(
            failure(&format!("c = 1\n{word} = 2\nx = {word} c\nx")),
            (
                ErrorKind::Reference,
                "c is not defined".to_string(),
                Some(Position::new(3, 1))
            )
        );
        assert_eq!(parse_expression(&format!("{word} x")).unwrap(), class("x"));
        for (source, message, column) in [
            ("{w} x => x", "Unexpected '=>'", 10),
            ("{w} all v in xs: v", "Unexpected keyword 'in'", 14),
            ("{w} x := 1", "Invalid value assignment target", 7),
            ("{w} x += 1", "Invalid compound assignment target", 7),
        ] {
            let source = source.replace("{w}", word);
            assert_eq!(
                syntax_error(&source),
                (message.to_string(), Position::new(1, column)),
                "{source}"
            );
        }
    }
    for (source, message, column) in [
        ("ok = all v of xs: v", "Unexpected keyword 'of'", 15),
        ("ok = any v of xs: v", "Unexpected keyword 'of'", 15),
        (
            "ok = all v in xs v > 0",
            "Expected ':' but found identifier 'v'",
            18,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(1, column)),
            "{source}"
        );
    }
}

#[test]
fn other_words_followed_by_a_name_keep_their_juxtaposed_reading() {
    for word in [
        "All", "ALL", "Any", "ANY", "Every", "EVERY", "some", "each", "anyone", "allow", "everyone",
    ] {
        for (source, juxtaposed, _) in NAME_JUXTAPOSITIONS {
            let source = source.replace("{w}", word);
            let juxtaposed = juxtaposed.replace("{w}", word);
            assert_eq!(statements(&source), statements(&juxtaposed), "{source}");
        }
        for sentence in ["Object.color is red", "Human is mortal"] {
            assert_eq!(
                statements(&format!("{word} {sentence}")),
                statements(&format!("{word}\n{sentence}")),
                "{word} {sentence}"
            );
        }
        assert_eq!(
            parse_expression(&format!("{word} Human")).unwrap(),
            name(word)
        );
        fresh(
            &format!(
                "{word} = 3\n{word} Human = 4\nassert([{word}, Human], [3, 4])\n[{word}, Human]"
            ),
            "[3,4]",
        );
        assert_eq!(
            syntax_error(&format!("if {word} Object.number > 4:\n    r = 1")),
            (
                "Expected ':' but found identifier 'Object'".to_string(),
                Position::new(1, format!("if {word} ").len() + 1)
            ),
            "{word}"
        );
    }
}

#[test]
fn predicate_sentences_take_all_and_any_subjects_in_every_statement_position() {
    let mut compared = 0;
    for position in STATEMENT_POSITIONS {
        let expected = place(position, "$Human.mortal = true");
        for sentence in ["all Human is mortal", "any Human is mortal"] {
            let source = place(position, sentence);
            assert_eq!(statements(&source), statements(&expected), "{source}");
            compared += 1;
        }
    }
    assert_eq!(compared, 58);
    assert_eq!(
        statements("all Human is mortal; any Order is big"),
        [predicate("Human", "mortal"), predicate("Order", "big")]
    );
    assert_eq!(
        statements("if c: all Human is mortal else: any Human is wise"),
        statements("if c: $Human.mortal = true else: $Human.wise = true")
    );
    assert_eq!(
        statements("try: any Human is mortal catch e: all Human is wise"),
        statements("try: $Human.mortal = true catch e: $Human.wise = true")
    );
    for word in [
        "is", "every", "all", "any", "lambda", "suppose", "why", "Human",
    ] {
        assert_eq!(
            statements(&format!("all Human is {word}")),
            [predicate("Human", word)],
            "{word}"
        );
    }
    let program = parse_program("x = 1\nall Human is mortal\nany Order is big").unwrap();
    assert_eq!(program.statements.len(), 3);
    assert_eq!(
        program.positions,
        [
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(3, 1)
        ]
    );
}

#[test]
fn other_is_statements_on_spelled_class_references_keep_their_juxtaposed_reading() {
    let mut compared = 0;
    for subject in ["all Human", "any Human"] {
        for (source, reading) in JUXTAPOSED_SENTENCES {
            let source = source.replace("{s}", subject);
            assert_eq!(statements(&source), statements(reading), "{source}");
            compared += 1;
        }
    }
    assert_eq!(compared, 44);
    for source in [
        "if all Human is mortal:\n    r = 1",
        "r = (any Human is mortal)",
        "r = [all Human is mortal]",
    ] {
        let tree = format!("{:?}", statements(source));
        assert!(
            tree.contains(
                "Binary { operator: Is, left: ClassRef(\"Human\"), right: Identifier(\"mortal\") }"
            ),
            "{source}: {tree}"
        );
    }
}

#[test]
fn class_references_spelled_with_any_and_all_are_class_level_rules() {
    fresh(
        "class Order(total):\n    this.total = total\nsmall = Order(5)\nall Order.big = any Order.total > 10\nlarge = Order(20)\nassert([small.big, large.big], [false, true])\nsmall.total = 50\nsmall.big",
        "true",
    );
    fresh(
        "class Order(total):\n    this.total = total\nlarge = Order(200)\nif any Order.total > 100:\n    all Order.vip = true\nsmall = Order(5)\nassert([small.vip, large.vip], [null, true])\nsmall.total = 150\nsmall.vip",
        "true",
    );
    fresh(
        "class Order(total):\n    this.total = total\no = Order(5)\nall Order.size = 1\nassert(o.size, 1)\n$Order.size = 2\nassert(o.size, 2)\nany Order.size = 3\no.size",
        "3",
    );
    fresh(
        "class Order(total):\n    this.total = total\na = Order(5)\nb = Order(6)\nall Order.tag = \"x\"\na.tag = \"own\"\nassert([a.tag, b.tag], [\"own\", \"x\"])\nc = Order(1)\n[a.tag, b.tag, c.tag]",
        "[\"own\",\"x\",\"x\"]",
    );
    fresh(
        "class Order(total):\n    this.total = total\nall Order.big = all Order.total > 10\nclass Rush(total) extends Order:\n    this.total = total\nrush = Rush(50)\nassert(rush.big, true)\nany Rush.big = false\nrush.big",
        "false",
    );
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "class Human(name):\n    this.name = name\ns = Human(\"S\")\nsuppose:\n    all Human is divine\n    assert(s.divine, true)\n[s.divine]",
        "[null]",
    );
    let before = read(&mut runtime, "s.wise");
    let error = runtime
        .run("any Human is wise\nthrow \"UNDO\"")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Thrown);
    assert_eq!(read(&mut runtime, "s.wise"), before);
    let error = runtime
        .run("all Human.legs = 2\nall Human.legs = all Human.legs + 1")
        .unwrap_err();
    assert_eq!(
        (error.kind(), error.message()),
        (ErrorKind::Type, "Circular Dependency".to_string())
    );
    assert_eq!(read(&mut runtime, "s.legs"), "null");
    for (spelled, dollar) in [
        ("all Ghost.mortal = true", "$Ghost.mortal = true"),
        ("any Ghost is real", "$Ghost.real = true"),
        (
            "if any Ghost.x > 1:\n    all Ghost.y = 1",
            "if $Ghost.x > 1:\n    $Ghost.y = 1",
        ),
        ("all Ghost", "$Ghost"),
    ] {
        assert_eq!(failure(spelled), failure(dollar), "{spelled}");
    }
}

#[test]
fn spelled_class_references_render_and_explain_as_their_dollar_spelling() {
    for word in WORDS {
        assert_eq!(
            rendered(&format!("{word} Human.mortal = true")),
            "$Human.mortal=true"
        );
        assert_eq!(
            rendered(&format!("{word} Human is mortal")),
            "$Human.mortal=true"
        );
        assert_eq!(
            rendered(&format!("{{ {word} Human is mortal }}")),
            rendered("{ $Human.mortal = true }")
        );
        assert_eq!(
            rendered(&format!(
                "if {word} Order.total > 10:\n    {word} Order.big = true"
            )),
            rendered("if $Order.total > 10:\n    $Order.big = true")
        );
    }
    let dollar = EXAMPLE
        .replace("every Object", "$Object")
        .replace("any Object", "$Object")
        .replace("all Object", "$Object");
    let mut spelled = Runtime::new();
    let mut accepted = Runtime::new();
    expect(&mut spelled, EXAMPLE, "[null,\"red\"]");
    expect(&mut accepted, &dollar, "[null,\"red\"]");
    assert_eq!(graph_of(&spelled), graph_of(&accepted));
    for query in [
        "(why a.label).map(step => step.rule)",
        "(why b.color).map(step => step.rule)",
        "(affects b.number).map(step => step.rule)",
        "(affects a.number).map(step => step.node)",
    ] {
        assert_eq!(
            read(&mut spelled, query),
            read(&mut accepted, query),
            "{query}"
        );
    }
    assert_eq!(
        read(&mut spelled, "(affects b.number).map(step => step.rule)"),
        "[\"if($Object.number>4)@b\"]"
    );
    let ifs = |runtime: &Runtime| {
        graph_of(runtime)
            .into_iter()
            .filter(|(key, _, _)| key.starts_with("if("))
            .count()
    };
    assert_eq!(ifs(&spelled), 2);
    expect(
        &mut spelled,
        "if $Object.number > 4:\n    $Object.color is blue\nb.color",
        "\"blue\"",
    );
    assert_eq!(ifs(&spelled), 2);
    expect(
        &mut spelled,
        "if any Object.number > 4:\n    all Object.color is green\nb.color",
        "\"green\"",
    );
    assert_eq!(ifs(&spelled), 2);
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "class Human(name):\n    this.name = name\nany Human is mortal\ns = Human(\"S\")\nassert((why s.mortal)[0].rule, \"$Human.mortal = true\")\nassert((why s.mortal)[0].state, \"derived\")\ns.mortal",
        "true",
    );
}

const NESTED_SHAPES: [[(&str, &str, &str, &str); 2]; 6] = [
    [
        ("class H\n", "all H.w = function() { ", "any H.w = 1", " }"),
        ("class H\n", "$H.w = function() { ", "$H.w = 1", " }"),
    ],
    [
        ("r = ", "(any H.x + ", "all H.x", ")"),
        ("r = ", "($H.x + ", "$H.x", ")"),
    ],
    [
        ("r = ", "[all H, ", "any H", "]"),
        ("r = ", "[$H, ", "$H", "]"),
    ],
    [
        (
            "class H\n",
            "g = function() {\nall H is w\n",
            "any H is w",
            "\n}",
        ),
        (
            "class H\n",
            "g = function() {\n$H.w = true\n",
            "$H.w = true",
            "\n}",
        ),
    ],
    [
        (
            "class H\n",
            "any H is w = function() { ",
            "all H is w = 1",
            " }",
        ),
        ("class H\n", "$H is w = function() { ", "$H is w = 1", " }"),
    ],
    [
        ("class H\n", "if any H.x > 0: ", "all H.y = 1", ""),
        ("class H\n", "if $H.x > 0: ", "$H.y = 1", ""),
    ],
];

fn deepest((head, opener, base, closer): (&str, &str, &str, &str)) -> usize {
    let started = Instant::now();
    for depth in 1..200 {
        let source = format!(
            "{head}{}{base}{}",
            opener.repeat(depth),
            closer.repeat(depth)
        );
        let checked = Runtime::check(&source);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{opener}{base} at depth {depth}"
        );
        if let Err(error) = checked {
            assert!(
                matches!(
                    error.message().as_str(),
                    "Expressions are nested too deeply" | "Statements are nested too deeply"
                ),
                "{opener}{base} at depth {depth}: {error}"
            );
            return depth;
        }
    }
    panic!("{opener}{base} never reaches the nesting limit")
}

#[test]
fn nested_spelled_class_references_parse_each_level_once_up_to_the_nesting_limit() {
    for [spelled, accepted] in NESTED_SHAPES {
        let limit = deepest(spelled);
        assert!(limit > 15, "{spelled:?} stops at {limit}");
        assert_eq!(limit, deepest(accepted), "{spelled:?}");
    }
}
