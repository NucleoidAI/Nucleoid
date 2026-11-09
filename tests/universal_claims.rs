mod common;

use std::time::{Duration, Instant};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{
    Expr, Quantifier, Stmt, UnaryOp, VersionSelector, parse, parse_expression, parse_program,
};
use nucleoid::{ErrorKind, Position, Runtime};

const SOCRATES: &str = "class Human(name: str):\n    this.name = name\n\nevery Human is mortal\n\nsocrates = Human(\"Socrates\")\nassert(socrates.mortal, true)\nassert(every h in Human: h.mortal, true)\nsocrates.mortal";

const QUANTIFIERS: [(&str, &str); 8] = [
    ("every v in xs: v > 0", "all v in xs: v > 0"),
    (
        "every v in Order: v.total > 10",
        "all v in Order: v.total > 10",
    ),
    (
        "every v in xs: any w in xs: v == w",
        "all v in xs: any w in xs: v == w",
    ),
    (
        "any v in xs: every w in xs: v == w",
        "any v in xs: all w in xs: v == w",
    ),
    ("every every in xs: every", "all every in xs: every"),
    ("every is in xs: is", "all is in xs: is"),
    ("every v in why x: v", "all v in why x: v"),
    ("every v in c ? xs : []: v", "all v in c ? xs : []: v"),
];

const QUANTIFIER_CONTEXTS: [&str; 24] = [
    "{q}",
    "ok = {q}",
    "ok = f({q})",
    "ok = [{q}, 1]",
    "ok = {k: {q}}",
    "ok = {k: ({q}), j: 1}",
    "ok = c and {q}",
    "ok = not {q}",
    "ok = ({q}) ? 1 : 2",
    "ok = 1 if ({q}) else 2",
    "ok is {q}",
    "$Order.ok = {q}",
    "assert: {q}, true",
    "ok = xs.filter(w => {q})",
    "ok = all t in xs: {q}",
    "ok = `${({q})}`",
    "if {q}:\n    r = 1",
    "if {q}\n    r = 1",
    "if c:\n    r = 1\nelif {q}:\n    r = 2",
    "if c:\n    r = 1\nelse {q}\n    r = 2",
    "while {q}:\n    r = 1",
    "def g() => {q}",
    "{ ok = {q} }",
    "ok = lambda: {q}",
];

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

const PROTECTED: [&str; 63] = [
    "xs = [1, 2]\nr = xs.every(v => v > 0)",
    "r = x.every",
    "r = x.every y",
    "every = 3",
    "every = 3\nr = every.x",
    "r = every(1)",
    "r = [every, x]",
    "r = [every]",
    "for every in xs:\n    r = every",
    "for every of xs:\n    pass",
    "def every(x) => x",
    "def every x => x",
    "def f every x => every + x",
    "def f(every, x) => every",
    "def f every, x => every",
    "def f every:\n    return every",
    "r = (every, x) => every",
    "r = every => every",
    "r = lambda every: every",
    "r = lambda every, x: every",
    "every: int",
    "r = {every: 1}",
    "r = {every}",
    "r = {k: 1, every: 2}",
    "r = {k: every}",
    "r = {k: every x: 1}",
    "r = {k: every, x: 1}",
    "r = {k: c ? 1 : every x: 2}",
    "r = f(every=1)",
    "class Item(every):\n    this.every = every\nr = Item(every=1)",
    "{every} = o",
    "{k: every} = o",
    "every, x = [1, 2]",
    "class every:\n    pass",
    "class K(every):\n    this.every = every",
    "try:\n    x = 1\ncatch every:\n    pass",
    "r = all every in xs: every > 0",
    "r = [every for every in xs]",
    "enum E: every, other",
    "every is 3",
    "every is not 3",
    "every is",
    "r = (every is 3)",
    "if every is 3:\n    r = 1",
    "every x: int",
    "every assert: 1, 1",
    "every suppose:\n    x = 1",
    "every lambda: 1",
    "r = `${every x}`",
    "r = `${every v in xs}`",
    "r = `${1 + every x}`",
    "every\nx = 1",
    "every $Human",
    "every 1",
    "every \"s\"",
    "every (x)",
    "every [x]",
    "r = every.every",
    "r = every ?? 1",
    "r = every |> why",
    "r = x |> every",
    "every@first",
    "r = every if c else x",
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

fn renamed(source: &str) -> String {
    format!("{:?}", parse(&source.replace("every", "qq"))).replace("qq", "every")
}

fn fresh(source: &str, expected: &str) {
    expect(&mut Runtime::new(), source, expected);
}

fn expect(runtime: &mut Runtime, source: &str, expected: &str) {
    let parsed = statements(source);
    let assertions = common::assertions::protected_assertions(&parsed).len();
    let case = common::Case {
        title: "universal claim regression".to_string(),
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

fn quantified(binder: &str, collection: Expr, condition: Expr) -> Expr {
    Expr::Quantified {
        quantifier: Quantifier::All,
        binder: binder.to_string(),
        collection: Box::new(collection),
        condition: Box::new(condition),
    }
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

const SPEC_TITLES: [&str; 5] = [
    "Nucleoid states class predicates with is",
    "Nucleoid spells class references with every",
    "Nucleoid quantifies with every as with all",
    "Nucleoid keeps every an ordinary name where no name follows it",
    "Nucleoid keeps other is statements on a class reference",
];

#[test]
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let readme = common::snippets(include_str!("../README.md"), "README.md");
    let examples = common::snippets(include_str!("../docs/examples.md"), "docs/examples.md");
    let appended = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (
            &reference,
            "Nucleoid states universal claims with every and is",
        ),
        (&readme, "Every states a class-level claim as it is said"),
        (
            &examples,
            "Every keeps a fleet's rules stated as claims about every truck",
        ),
    ]);
    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        let count = common::assertions::protected_assertions(&statements(&case.source)).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 32);
    let last: Vec<&str> = spec
        .iter()
        .skip(spec.len() - SPEC_TITLES.len())
        .map(|case| case.title.as_str())
        .collect();
    assert_eq!(last, SPEC_TITLES);
    for (cases, title) in [
        (
            &reference,
            "Nucleoid states universal claims with every and is",
        ),
        (&readme, "Every states a class-level claim as it is said"),
        (
            &examples,
            "Every keeps a fleet's rules stated as claims about every truck",
        ),
    ] {
        assert_eq!(cases.last().map(|case| case.title.as_str()), Some(title));
    }
}

#[test]
fn the_users_sentences_state_that_every_human_is_mortal() {
    fresh(SOCRATES, "true");
    let rule = statements("$Human.mortal = true");
    assert_eq!(rule, [predicate("Human", "mortal")]);
    for sentence in [
        "$Human is mortal",
        "every Human is mortal",
        "every Human.mortal = true",
        "every Human.mortal is true",
        "($Human) is mortal",
        "(every Human) is mortal",
    ] {
        assert_eq!(statements(sentence), rule, "{sentence}");
    }
    assert_eq!(
        statements("every human is mortal"),
        [predicate("human", "mortal")]
    );
    fresh(
        "class human\nevery human is mortal\nh = human()\nassert(h.mortal, true)\nh.mortal",
        "true",
    );
    assert_eq!(
        failure("class Human\nevery human is mortal"),
        (
            ErrorKind::Reference,
            "human is not defined".to_string(),
            Some(Position::new(2, 1))
        )
    );
}

#[test]
fn every_quantifiers_build_exactly_the_tree_of_all() {
    let mut compared = 0;
    for context in QUANTIFIER_CONTEXTS {
        for (every, all) in QUANTIFIERS {
            let spelled = context.replace("{q}", every);
            let expected = context.replace("{q}", all);
            assert_eq!(statements(&spelled), statements(&expected), "{spelled}");
            assert_eq!(rendered(&spelled), rendered(&expected), "{spelled}");
            compared += 1;
        }
    }
    assert_eq!(compared, 192);
    assert_eq!(
        parse_expression("every v in xs: v").unwrap(),
        quantified("v", name("xs"), name("v"))
    );
    assert_eq!(
        parse_expression("every v in xs: v trailing").unwrap(),
        quantified("v", name("xs"), name("v"))
    );
    assert_eq!(
        rendered("ok = every v in xs: v > 0"),
        "ok=(all v in xs:v>0)"
    );
    assert_eq!(
        parse("ok = every v in xs: v").unwrap(),
        parse("ok = all v in xs: v").unwrap()
    );
}

#[test]
fn every_quantifiers_evaluate_and_stay_live_as_all_does() {
    fresh(
        "class Check\nready = every c in Check: c.passed\nassert(ready, true)\nfirst = Check()\nfirst.passed = true\nassert(ready, true)\nsecond = Check()\nassert(ready, false)\nsecond.passed = true\nassert(ready, all c in Check: c.passed)\nready",
        "true",
    );
    fresh(
        "xs = [3, 4]\nlimit = 2\nabove = every v in xs: v > limit\nassert(above, xs.every(v => v > limit))\nlimit = 3\nassert(above, false)\nxs = [5]\nabove",
        "true",
    );
    fresh(
        "class Task(skill):\n    this.skill = skill\nclass Worker(skill):\n    this.skill = skill\ncovered = every t in Task: any w in Worker: w.skill == t.skill\nTask(\"a\")\nassert(covered, false)\nWorker(\"a\")\ncovered",
        "true",
    );
    fresh(
        "every = [1, 2]\nr = every n in every: n > 0\nassert(every, [1, 2])\nr",
        "true",
    );
    fresh(
        "if every v in [1, 2]: v > 0\n    r = 1\nelse:\n    r = 2\nr",
        "1",
    );
    fresh(
        "xs = [0]\nif false:\n    r = 1\nelse every v in xs: v > 0\n    r = 2\nelse:\n    r = 3\nr",
        "3",
    );
}

#[test]
fn template_interpolations_keep_every_at_their_top_level() {
    fresh(
        "every = 5\nxs = [1]\nt = `${every v in xs}`\nassert(t, \"5\")\nt",
        "\"5\"",
    );
    fresh(
        "every = 5\nx = 1\nt = `${every x}`\nassert(t, \"5\")\nt",
        "\"5\"",
    );
    fresh(
        "every = 5\nx = 1\nt = `${1 + every x}`\nassert(t, \"6\")\nt",
        "\"6\"",
    );
    fresh(
        "xs = [1]\nt = `${(every v in xs: v > 0)}`\nassert(t, \"true\")\nt",
        "\"true\"",
    );
    fresh(
        "class Human\nt = `${(every Human)}`\nassert(t, `${($Human)}`)\nt",
        "\"class Human\"",
    );
}

#[test]
fn every_names_build_exactly_the_tree_of_their_class_references() {
    let mut compared = 0;
    for context in CLASS_CONTEXTS {
        for (spelled, reference) in [("every Order", "$Order"), ("every order", "$order")] {
            let source = context.replace("{c}", spelled);
            let expected = context.replace("{c}", reference);
            assert_eq!(statements(&source), statements(&expected), "{source}");
            assert_eq!(rendered(&source), rendered(&expected), "{source}");
            compared += 1;
        }
    }
    assert_eq!(compared, 92);
    assert_eq!(parse_expression("every Human").unwrap(), class("Human"));
    assert_eq!(
        parse_expression("every Human.mortal").unwrap(),
        parse_expression("$Human.mortal").unwrap()
    );
    assert_eq!(
        parse_expression("every Human is mortal").unwrap(),
        class("Human")
    );
    assert_eq!(rendered("every Human.mortal = true"), "$Human.mortal=true");
    assert_eq!(
        rendered("if every Order.total > 10:\n    every Order.big = true"),
        rendered("if $Order.total > 10:\n    $Order.big = true")
    );
}

#[test]
fn class_references_spelled_with_every_are_class_level_rules() {
    fresh(
        "class Order(total):\n    this.total = total\nevery Order.big = every Order.total > 10\nsmall = Order(5)\nlarge = Order(20)\nassert([small.big, large.big], [false, true])\nsmall.total = 50\nsmall.big",
        "true",
    );
    fresh(
        "class Order(total):\n    this.total = total\nif every Order.total > 10:\n    every Order.big = true\nlarge = Order(20)\nsmall = Order(5)\nassert(small.big, null)\nlarge.big",
        "true",
    );
    fresh(
        "class Human(name):\n    this.name = name\nevery Human.legs is 2\nevery Human.tag is every Human.name\ns = Human(\"S\")\nassert(s.tag, \"S\")\ns.legs",
        "2",
    );
    assert_eq!(
        failure("every = 3\nevery Human = 4"),
        (
            ErrorKind::Syntax,
            "Cannot assign to $Human".to_string(),
            Some(Position::new(2, 1))
        )
    );
    assert_eq!(
        failure("every Ghost.mortal = true"),
        failure("$Ghost.mortal = true")
    );
}

#[test]
fn every_keeps_its_accepted_reading_unless_a_name_follows_it_where_an_operand_begins() {
    for source in PROTECTED {
        assert_eq!(format!("{:?}", parse(source)), renamed(source), "{source}");
    }
    assert_eq!(
        statements("every x: int"),
        [
            Stmt::Expression(name("every")),
            Stmt::Declaration {
                name: "x".to_string(),
                type_name: "int".to_string(),
            }
        ]
    );
    assert_eq!(
        statements("r = {k: every x: 1}"),
        statements("r = {k: every, x: 1}")
    );
    assert_eq!(parse_expression("every is 3").unwrap(), name("every"));
    assert_eq!(parse_expression("every x: 1").unwrap(), name("every"));
    assert_eq!(parse_expression("every").unwrap(), name("every"));
    fresh(
        "every = 2\no = {k: every x: 1}\nassert(o, {k: 2, x: 1})\nevery is 7\nassert(o, {k: 7, x: 1})\nassert([every, (every is 7)], [7, true])\nevery assert: 1, 1\nevery",
        "7",
    );
}

#[test]
fn every_followed_by_a_name_was_a_juxtaposition_and_now_reads_the_class_reference() {
    for (source, juxtaposed, reference) in [
        ("every Human = 4", "every\nHuman = 4", "$Human = 4"),
        ("r = every x", "r = every\nx", "r = $x"),
        ("r = v => every x", "r = v => every\nx", "r = v => $x"),
        ("why every x", "why every\nx", "why $x"),
        ("not every x.y", "not every\nx.y", "not $x.y"),
        ("every Order(5)", "every\nOrder(5)", "$Order(5)"),
        ("x0 = 0 every x", "x0 = 0 every\nx", "x0 = 0 $x"),
    ] {
        assert_ne!(statements(source), statements(juxtaposed), "{source}");
        assert_eq!(statements(source), statements(reference), "{source}");
    }
    for (source, message, column) in [
        ("every x => x", "Unexpected '=>'", 12),
        ("every all v in xs: v", "Unexpected keyword 'in'", 16),
        ("every x := 1", "Invalid value assignment target", 9),
        ("every x += 1", "Invalid compound assignment target", 9),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(1, column)),
            "{source}"
        );
    }
}

#[test]
fn predicate_sentences_build_the_tree_of_their_true_property_in_every_statement_position() {
    let mut compared = 0;
    for position in STATEMENT_POSITIONS {
        let expected = place(position, "$Human.mortal = true");
        for sentence in ["$Human is mortal", "every Human is mortal"] {
            let source = place(position, sentence);
            assert_eq!(statements(&source), statements(&expected), "{source}");
            compared += 1;
        }
    }
    assert_eq!(compared, 58);
    assert_eq!(
        statements("$Human is mortal; $Order is big"),
        [predicate("Human", "mortal"), predicate("Order", "big")]
    );
    assert_eq!(
        statements("$Human is mortal\n$Human is wise"),
        statements("$Human.mortal = true\n$Human.wise = true")
    );
    assert_eq!(
        statements("if c: $Human is mortal else: $Human is wise"),
        statements("if c: $Human.mortal = true else: $Human.wise = true")
    );
    assert_eq!(
        statements("try:\n    $Human is mortal\ncatch e: $Human is wise"),
        statements("try:\n    $Human.mortal = true\ncatch e: $Human.wise = true")
    );
    assert_eq!(
        statements("try: $Human is mortal catch e: $Human is wise"),
        statements("try: $Human.mortal = true catch e: $Human.wise = true")
    );
    for word in [
        "is", "every", "all", "any", "lambda", "suppose", "while", "enum", "assert", "why", "sum",
        "model", "first", "Human",
    ] {
        assert_eq!(
            statements(&format!("$Human is {word}")),
            [predicate("Human", word)],
            "{word}"
        );
    }
    let program = parse_program("x = 1\n$Human is mortal\nevery Order is big").unwrap();
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
fn other_is_statements_on_a_class_reference_keep_their_juxtaposed_reading() {
    for (source, reading) in [
        ("$Human is mortal z1 = 1", "$Human\nis\nmortal\nz1 = 1"),
        (
            "$Human is mortal $Human is wise",
            "$Human\nis\nmortal\n$Human.wise = true",
        ),
        ("$Human is mortal x", "$Human\nis\nmortal\nx"),
        (
            "$Human is mortal assert: 1, 1",
            "$Human\nis\nmortal\nassert: 1, 1",
        ),
        (
            "$Human is mortal if c:\n    z1 = 1",
            "$Human\nis\nmortal\nif c:\n    z1 = 1",
        ),
        (
            "r = f(() => {\n$Human is mortal\nreturn 1\n})",
            "r = f(() => {\n$Human; is; mortal; return 1\n})",
        ),
        ("$Order is 5", "$Order\nis\n5"),
        ("$Human is true", "$Human\nis\ntrue"),
        ("$Human is \"mortal\"", "$Human\nis\n\"mortal\""),
        ("$Human is 'mortal'", "$Human\nis\n'mortal'"),
        ("$Human is (mortal)", "$Human\nis(mortal)"),
        ("$Human is [mortal]", "$Human\nis[mortal]"),
        ("$Human is mortal and wise", "$Human\nis\nmortal and wise"),
        ("$Human is mortal or wise", "$Human\nis\nmortal or wise"),
        ("$Human is not mortal", "$Human\nis\nnot mortal"),
        ("$Human is mortal.x", "$Human\nis\nmortal.x"),
        ("$Human is mortal()", "$Human\nis\nmortal()"),
        ("$Human is mortal[0]", "$Human\nis\nmortal[0]"),
        ("$Human is mortal + 1", "$Human\nis\nmortal + 1"),
        ("$Human is mortal = 1", "$Human\nis\nmortal = 1"),
        ("$Human is mortal += 1", "$Human\nis\nmortal += 1"),
        ("$Human is mortal := 1", "$Human\nis\nmortal := 1"),
        ("$Human is mortal: int", "$Human\nis\nmortal: int"),
        ("$Human is mortal is wise", "$Human\nis\nmortal is wise"),
        ("$Human is is wise", "$Human\nis is wise"),
        ("$Human is mortal => 1", "$Human\nis\nmortal => 1"),
        (
            "$Human is mortal if c else d",
            "$Human\nis\nmortal if c else d",
        ),
        ("$Human is mortal ? a : b", "$Human\nis\nmortal ? a : b"),
        ("$Human is mortal ?? 1", "$Human\nis\nmortal ?? 1"),
        ("$Human is mortal |> why", "$Human\nis\nmortal |> why"),
        ("$Human is every Order", "$Human\nis\nevery Order"),
        ("$Human is all v in xs: v", "$Human\nis\nall v in xs: v"),
        ("$Human is why x", "$Human\nis\nwhy x"),
        ("$Human is sum of xs", "$Human\nis\nsum of xs"),
        ("$Human is lambda: 1", "$Human\nis\nlambda: 1"),
        (
            "$Human is suppose:\n    x = 1",
            "$Human\nis\nsuppose:\n    x = 1",
        ),
        ("$Human is enum E: A, B", "$Human\nis\nenum E: A, B"),
        ("$Human is while c: x = 1", "$Human\nis\nwhile c: x = 1"),
        ("$Human is assert: 1, 1", "$Human\nis\nassert: 1, 1"),
        ("$Human is $Mortal", "$Human\nis\n$Mortal"),
        ("$Human is this", "$Human\nis\nthis"),
        ("$Human is 1 + 2", "$Human\nis\n1 + 2"),
        ("$Human is", "$Human\nis"),
        ("$Human is\nmortal", "$Human\nis\nmortal"),
        ("this is mortal", "this\nis\nmortal"),
        ("-$Human is mortal", "-$Human\nis\nmortal"),
        ("return $Human is mortal", "return $Human\nis\nmortal"),
        ("x = $Human is mortal", "x = $Human\nis\nmortal"),
    ] {
        assert_eq!(statements(source), statements(reading), "{source}");
    }
    assert_eq!(
        syntax_error("$Human is @mortal"),
        (
            "Expected a non-negative integer, 'first', 'last', or 'previous' after '@'".to_string(),
            Position::new(1, 12)
        )
    );
    let word = Stmt::Assign {
        target: Expr::Member {
            object: Box::new(class("Human")),
            property: "kind".to_string(),
        },
        value: Expr::Unary {
            operator: UnaryOp::Word,
            operand: Box::new(name("mortal")),
        },
    };
    assert_eq!(statements("$Human.kind is mortal"), [word]);
    assert_eq!(
        statements("$Human@first is mortal"),
        [Stmt::Assign {
            target: Expr::Version {
                reference: Box::new(class("Human")),
                selector: VersionSelector::First,
            },
            value: Expr::Unary {
                operator: UnaryOp::Word,
                operand: Box::new(name("mortal")),
            },
        }]
    );
    assert_eq!(
        statements("socrates is mortal"),
        [Stmt::Assign {
            target: name("socrates"),
            value: Expr::Unary {
                operator: UnaryOp::Word,
                operand: Box::new(name("mortal")),
            },
        }]
    );
    for source in [
        "if $Human is mortal:\n    r = 1",
        "r = ($Human is mortal)",
        "r = [$Human is mortal]",
    ] {
        let tree = format!("{:?}", statements(source));
        assert!(
            tree.contains(
                "Binary { operator: Is, left: ClassRef(\"Human\"), right: Identifier(\"mortal\") }"
            ),
            "{source}: {tree}"
        );
    }
    assert_eq!(
        parse_expression("$Human is mortal").unwrap(),
        class("Human")
    );
}

#[test]
fn predicate_sentences_are_the_class_level_rules_they_state() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "class Human(name):\n    this.name = name\nplato = Human(\"Plato\")\nmortal = 5\n$Human is mortal\nsocrates = Human(\"Socrates\")\nassert([plato.mortal, socrates.mortal], [true, true])\nassert(mortal, 5)\nassert((why socrates.mortal).map(step -> step.rule), [\"$Human.mortal = true\"])\nsocrates.mortal",
        "true",
    );
    expect(
        &mut runtime,
        "$Human.mortal = false\nassert([plato.mortal, socrates.mortal], [false, false])\nevery Human is mortal\n[plato.mortal, socrates.mortal]",
        "[true,true]",
    );
    expect(
        &mut runtime,
        "plato.mortal = false\nassert(socrates.mortal, true)\nplato.mortal",
        "false",
    );
    expect(
        &mut runtime,
        "suppose:\n    every Human is divine\n    assert(socrates.divine, true)\n[socrates.divine]",
        "[null]",
    );
    expect(
        &mut runtime,
        "class Greek(name) extends Human:\n    this.name = name\nzeno = Greek(\"Zeno\")\nzeno.mortal",
        "true",
    );
    let before = runtime.run("socrates.wise").unwrap().to_string();
    let error = runtime.run("$Human is wise\nthrow \"UNDO\"").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Thrown);
    assert_eq!(runtime.run("socrates.wise").unwrap().to_string(), before);
    assert_eq!(
        failure("every Ghost is mortal"),
        failure("$Ghost.mortal = true")
    );
}

#[test]
fn predicate_sentences_render_and_explain_as_their_equals_spelling() {
    for sentence in ["$Human is mortal", "every Human is mortal"] {
        assert_eq!(rendered(sentence), "$Human.mortal=true");
        assert_eq!(
            rendered(&format!("{{ {sentence} }}")),
            rendered("{ $Human.mortal = true }")
        );
    }
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "class Human(name):\n    this.name = name\nevery Human is mortal\ns = Human(\"S\")\nassert((why s.mortal)[0].rule, \"$Human.mortal = true\")\nassert((why s.mortal)[0].state, \"derived\")\ns.mortal",
        "true",
    );
}

const NESTED_SHAPES: [[(&str, &str, &str, &str); 2]; 6] = [
    [
        ("class H\n", "$H is w = function() { ", "$H is w = 1", " }"),
        (
            "class H\n",
            "$H\nis\nw = function() { ",
            "$H\nis\nw = 1",
            " }",
        ),
    ],
    [
        ("class H\n", "g = function() {\n$H is w\n", "$H is w", "\n}"),
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
            "every H.w = function() { ",
            "every H.w = 1",
            " }",
        ),
        ("class H\n", "$H.w = function() { ", "$H.w = 1", " }"),
    ],
    [
        ("r = ", "(every H.x + ", "every H.x", ")"),
        ("r = ", "($H.x + ", "$H.x", ")"),
    ],
    [
        ("r = ", "every v in [1]: ", "true", ""),
        ("r = ", "all v in [1]: ", "true", ""),
    ],
    [
        ("r = ", "every v in (", "[1]", "): v"),
        ("r = ", "all v in (", "[1]", "): v"),
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
fn nested_universal_claims_parse_each_level_once_up_to_the_nesting_limit() {
    for [spelled, accepted] in NESTED_SHAPES {
        let limit = deepest(spelled);
        assert!(limit > 15, "{spelled:?} stops at {limit}");
        assert_eq!(limit, deepest(accepted), "{spelled:?}");
    }
}

#[test]
fn malformed_every_quantifiers_report_the_diagnostics_of_all() {
    for (source, message, line, column) in [
        (
            "ok = every v in xs v > 0",
            "Expected ':' but found identifier 'v'",
            1,
            20,
        ),
        (
            "ok = every v in xs",
            "Expected ':' but found end of input",
            1,
            19,
        ),
        (
            "ok = every v in xs\n",
            "Expected ':' but found end of line",
            1,
            17,
        ),
        ("ok = every v in xs:", "Unexpected end of input", 1, 20),
        ("ok = every v in : v", "Unexpected ':'", 1, 19),
        (
            "ok = every v in xs:\n    v > 1",
            "Unexpected end of line",
            2,
            5,
        ),
        ("ok = every 1 in xs: v", "Unexpected keyword 'in'", 1, 17),
        ("ok = every (v) in xs: v", "Unexpected keyword 'in'", 1, 19),
        ("ok = every this in xs: 1", "Unexpected keyword 'in'", 1, 20),
        ("ok = every v of xs: v", "Unexpected keyword 'of'", 1, 17),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    assert_eq!(
        failure("x = 5\nok = every v in x: v"),
        (
            ErrorKind::Type,
            "x is not a collection".to_string(),
            Some(Position::new(2, 1))
        )
    );
}
