mod common;

use std::time::{Duration, Instant};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::operator::binary;
use nucleoid::lang::ast::{BinaryOp, Expr, Stmt, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime, Value};

const SPEC_TITLES: [&str; 5] = [
    "Nucleoid tests membership with in and not in",
    "Nucleoid keeps membership tests live",
    "Nucleoid binds membership tests as comparisons",
    "Nucleoid waits on null collections and rejects other values in membership tests",
    "Nucleoid keeps in readings where membership tests do not apply",
];
const REFERENCE_TITLE: &str = "Nucleoid tests membership with in";
const README_TITLE: &str = "Membership tests read in the order they are said";
const EXAMPLES_TITLE: &str = "Membership tests keep an order book's rules live";

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn rendered(source: &str) -> String {
    generate_all(&statements(source))
}

fn tree(source: &str) -> String {
    format!("{:?}", statements(source))
}

fn holds_membership(source: &str) -> bool {
    let tree = tree(source);
    tree.contains("operator: In,") || tree.contains("operator: NotIn,")
}

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn membership(operator: BinaryOp, item: Expr, collection: Expr) -> Expr {
    Expr::Binary {
        operator,
        left: Box::new(item),
        right: Box::new(collection),
    }
}

fn result(source: &str) -> String {
    let mut runtime = Runtime::new();
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    assert!(runtime.take_assertions().is_empty(), "{source}");
    runtime.serialize_json(&value).unwrap()
}

fn failure(source: &str) -> (ErrorKind, String, Option<Position>) {
    let error = Runtime::new().run(source).unwrap_err();
    (error.kind(), error.message(), error.position())
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn dependencies(runtime: &Runtime, name: &str) -> Vec<String> {
    let mut found: Vec<String> = runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap_or_else(|| panic!("{name} is filed"))
        .dependencies
        .iter()
        .map(|key| key.as_str().to_string())
        .collect();
    found.sort();
    found
}

fn documents() -> [Vec<common::Case>; 4] {
    [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ]
}

#[test]
fn added_document_cases_run_every_assertion_before_the_every_entries() {
    let [spec, reference, readme, examples] = documents();
    let added = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (&reference, REFERENCE_TITLE),
        (&readme, README_TITLE),
        (&examples, EXAMPLES_TITLE),
    ]);
    let mut assertions = 0;
    for (cases, title) in added {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is added exactly once");
        let case = matching[0];
        assert!(holds_membership(&case.source), "{title} tests membership");
        let count = common::assertions::protected_assertions(&statements(&case.source)).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 65);
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    let first = titles
        .iter()
        .position(|title| *title == SPEC_TITLES[0])
        .unwrap();
    assert_eq!(titles[first..first + 5], SPEC_TITLES);
    assert_eq!(
        titles[first + 5],
        "Nucleoid states class predicates with is"
    );
    for (cases, title, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid states universal claims with every and is",
        ),
        (
            &readme,
            README_TITLE,
            "Every states a class-level claim as it is said",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Every keeps a fleet's rules stated as claims about every truck",
        ),
    ] {
        let titles: Vec<&str> = cases.iter().map(|case| case.title.as_str()).collect();
        let index = titles.iter().position(|found| *found == title).unwrap();
        assert_eq!(titles[index + 1], following, "{title}");
    }
}

#[test]
fn no_earlier_document_case_holds_a_membership_test() {
    let mut earlier = 0;
    for cases in documents() {
        let added = cases
            .iter()
            .position(|case| {
                SPEC_TITLES.contains(&case.title.as_str())
                    || [REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE]
                        .contains(&case.title.as_str())
            })
            .unwrap();
        for case in &cases[..added] {
            assert!(!holds_membership(&case.source), "{}", case.title);
            earlier += 1;
        }
    }
    assert_eq!(earlier, 1071);
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Membership tests\n")
        .expect("the language reference documents membership tests");
    let (_, block) = section.split_once("```\n").unwrap();
    let (example, _) = block.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is added");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 97](nuc-0097.md)"));
}

const COMPARISON_CONTEXTS: [&str; 38] = [
    "{m}",
    "ok = {m}",
    "ok = f({m})",
    "ok = [{m}, 1]",
    "ok = {k: {m}}",
    "ok = {k: {m} j: 1}",
    "ok = c and {m}",
    "ok = {m} or c",
    "ok = not ({m})",
    "ok = ({m}) ? 1 : 2",
    "ok = {m} ? 1 : 2",
    "ok = c ? {m} : 2",
    "ok = c ? 1 : {m}",
    "ok = 1 if {m} else 2",
    "ok = {m} if c else 2",
    "ok = {m} == true",
    "ok = {m} ?? c",
    "ok is {m}",
    "$Order.ok = {m}",
    "assert: {m}, true",
    "assert({m}, true)",
    "ok = xs.filter(w => {m})",
    "ok = all t in ts: {m}",
    "ok = all t in [{m}]: t",
    "ok = `${({m})}`",
    "if {m}:\n    r = 1",
    "if {m}\n    r = 1",
    "if c:\n    r = 1\nelif {m}:\n    r = 2",
    "if c:\n    r = 1\nelse {m}\n    r = 2",
    "while {m}:\n    r = 1",
    "def g() => {m}",
    "def g():\n    return {m}",
    "{ ok = {m} }",
    "ok = lambda: {m}",
    "ok = ys[{m}]",
    "ok = ys[{m}:]",
    "ok = ys[1:{m}]",
    "ok = [x for x in xs if {m}]",
];

#[test]
fn membership_tests_build_the_tree_of_a_comparison_wherever_one_stands() {
    let mut compared = 0;
    for context in COMPARISON_CONTEXTS {
        let less = tree(&context.replace("{m}", "x < xs"));
        for (written, operator) in [("x in xs", "In"), ("x not in xs", "NotIn")] {
            let source = context.replace("{m}", written);
            assert_eq!(
                tree(&source),
                less.replace("operator: Less,", &format!("operator: {operator},")),
                "{source}"
            );
            compared += 1;
        }
    }
    assert_eq!(compared, 76);
    assert_eq!(
        statements("ok = x in xs"),
        vec![Stmt::Assign {
            target: name("ok"),
            value: membership(BinaryOp::In, name("x"), name("xs")),
        }]
    );
    assert_eq!(
        statements("[x in ys for x in xs]"),
        statements("xs.map(x => x in ys)")
    );
}

#[test]
fn membership_tests_bind_as_tightly_as_comparisons() {
    for (source, grouped) in [
        ("ok = x + 1 in xs", "ok = (x + 1) in xs"),
        ("ok = x in xs + ys", "ok = x in (xs + ys)"),
        ("ok = x * 2 not in xs", "ok = (x * 2) not in xs"),
        ("ok = x ** 2 in xs", "ok = (x ** 2) in xs"),
        ("ok = x in xs == true", "ok = (x in xs) == true"),
        ("ok = x in xs != y in ys", "ok = (x in xs) != (y in ys)"),
        ("ok = a < b in c", "ok = (a < b) in c"),
        ("ok = a in b < c", "ok = (a in b) < c"),
        ("ok = a in b in c", "ok = (a in b) in c"),
        ("ok = a not in b in c", "ok = (a not in b) in c"),
        ("ok = x in xs and y in ys", "ok = (x in xs) and (y in ys)"),
        (
            "ok = x in xs or y not in ys",
            "ok = (x in xs) or (y not in ys)",
        ),
        ("ok = x in xs && y", "ok = (x in xs) && y"),
        ("ok = x in xs ?? y", "ok = (x in xs) ?? y"),
        ("ok = not x in xs", "ok = (not x) in xs"),
        ("ok = !x in xs", "ok = (!x) in xs"),
        ("ok = -x in xs", "ok = (-x) in xs"),
        ("ok = typeof x in xs", "ok = (typeof x) in xs"),
        ("ok = x in why c", "ok = x in (why c)"),
        ("ok = x in sum of xs", "ok = x in (sum of xs)"),
        ("ok = x in xs ? 1 : 2", "ok = (x in xs) ? 1 : 2"),
        ("ok = (a is b in c)", "ok = (a is (b in c))"),
        ("ok = (a in b is c)", "ok = ((a in b) is c)"),
    ] {
        assert_eq!(statements(source), statements(grouped), "{source}");
    }
    assert_ne!(
        statements("ok = not x in xs"),
        statements("ok = not (x in xs)")
    );
    assert_ne!(
        statements("ok = not (x in xs)"),
        statements("ok = x not in xs")
    );
}

#[test]
fn membership_tests_render_grouped_and_apart_from_includes() {
    for (source, expected) in [
        ("ok = x in xs", "ok=(x in xs)"),
        ("ok = x not in xs", "ok=(x not in xs)"),
        ("ok = xs.includes(x)", "ok=xs.includes(x)"),
        ("ok = !xs.includes(x)", "ok=!xs.includes(x)"),
        ("ok = not (x in xs)", "ok=!(x in xs)"),
        ("ok = (a in b) in c", "ok=((a in b) in c)"),
        ("ok = a in (b in c)", "ok=(a in (b in c))"),
        ("ok = a < b in c", "ok=(a<b in c)"),
        ("ok = a in b == c", "ok=(a in b)==c"),
        ("ok = (a in b).x", "ok=(a in b).x"),
        ("ok = (a in b) ** 2", "ok=(a in b)**2"),
        ("ok = 2 ** (a in b)", "ok=2**(a in b)"),
        ("ok = `${(a in b)}`", "ok=`${(a in b)}`"),
    ] {
        assert_eq!(rendered(source), expected, "{source}");
        assert_eq!(
            statements(&rendered(source)),
            statements(source),
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    runtime
        .run("xs = [1]\nif 1 in xs:\n    a = 1\nif xs.includes(1):\n    b = 1")
        .unwrap();
    let keys: Vec<NodeKey> = ["(1 in xs)", "xs.includes(1)"]
        .into_iter()
        .map(|condition| NodeKey::conditional(&parse_expression(condition).unwrap(), None))
        .collect();
    assert_ne!(keys[0], keys[1]);
    for key in &keys {
        assert!(runtime.graph.retrieve(key).is_some(), "{key:?}");
    }
}

const SETUP: &str = "class Seat(row):\n    this.row = row\nseat = Seat(1)\nother = Seat(2)\nmixed = [1, \"a\", null, [1, 2], {x: 1}, true, seat]\nempty = []\ntext = \"a1 true null [object]\"\nblank = \"\"";

#[test]
fn membership_tests_give_the_results_of_includes_over_lists_classes_and_strings() {
    let items = [
        "1",
        "2",
        "\"a\"",
        "\"1\"",
        "\"\"",
        "null",
        "true",
        "false",
        "[1, 2]",
        "[2, 1]",
        "{x: 1}",
        "{x: 2}",
        "Number.NaN",
        "seat",
        "other",
        "{id: \"seat\", row: 1}",
        "{row: 1}",
    ];
    let collections = ["mixed", "empty", "Seat", "text", "blank", "[seat]"];
    let mut compared = 0;
    for item in items {
        for collection in collections {
            let source = format!(
                "{SETUP}\n[{item} in {collection}, {collection}.includes({item}), {item} not in {collection}, !{collection}.includes({item})]"
            );
            let value: serde_json::Value = serde_json::from_str(&result(&source)).unwrap();
            assert_eq!(value[0], value[1], "{item} in {collection}");
            assert_eq!(value[2], value[3], "{item} not in {collection}");
            assert!(value[0].is_boolean(), "{item} in {collection}");
            compared += 1;
        }
    }
    assert_eq!(compared, 102);
    assert_eq!(
        result(&format!(
            "{SETUP}\n[seat in Seat, {{row: 1}} in Seat, {{id: \"seat\", row: 1}} in Seat, 1 in \"a1\", \"\" in blank, Number.NaN in [Number.NaN]]"
        )),
        "[true,false,true,true,true,false]"
    );
}

#[test]
fn declarations_with_membership_tests_read_what_their_includes_spelling_reads() {
    for (setup, membership, includes) in [
        (
            "tags = [1]\nitem = 1",
            "item in tags",
            "tags.includes(item)",
        ),
        (
            "tags = [1]\nitem = 1",
            "item not in tags",
            "!tags.includes(item)",
        ),
        ("word = \"abc\"", "\"b\" in word", "word.includes(\"b\")"),
        (
            "class Seat\nseat = Seat()",
            "seat in Seat",
            "Seat.includes(seat)",
        ),
        (
            "class Box\nbox = Box()\nbox.tags = [1]",
            "1 in box.tags",
            "box.tags.includes(1)",
        ),
        ("tags = [1]", "1 in tags.value", "tags.value.includes(1)"),
    ] {
        let mut left = Runtime::new();
        left.run(&format!("{setup}\nknown = {membership}")).unwrap();
        let mut right = Runtime::new();
        right.run(&format!("{setup}\nknown = {includes}")).unwrap();
        assert_eq!(
            dependencies(&left, "known"),
            dependencies(&right, "known"),
            "{membership}"
        );
    }
    assert_eq!(
        result(
            "tags = [\"b\"]\nknown = \"a\" in tags.value\ntags.push(\"a\")\n[known, \"a\" in tags]"
        ),
        "[false,true]"
    );
    assert_eq!(
        result("tags = [\"a\"]\nitem = \"a\"\nknown = item in tags\n(why known).map(s => s.rule)"),
        "[\"known = (item in tags)\",\"item = \\\"a\\\"\",\"tags = [\\\"a\\\"]\"]"
    );
    assert_eq!(
        result("tags = [\"a\"]\nknown = \"a\" in tags\n(affects tags).map(s => s.node)"),
        "[\"known\"]"
    );
}

#[test]
fn membership_tests_stay_live_in_declarations_rules_and_conditions() {
    let source = "tags = [\"red\"]\nknown = \"green\" in tags\nmissing = \"green\" not in tags\nassert([known, missing], [false, true])\ntags.push(\"green\")\nassert([known, missing], [true, false])\ntags = [\"blue\"]\nassert([known, missing], [false, true])\nclass Guest\ninvited = {id: \"ben\"} in Guest\nassert(invited, false)\nben = Guest()\nassert(invited, true)\ndelete ben\nassert(invited, false)\nclass Order(status):\n    this.status = status\n$Order.open = $Order.status in [\"new\", \"paid\"]\nif $Order.status not in [\"new\", \"paid\"]:\n    $Order.done = true\nfirst = Order(\"new\")\nsecond = Order(\"shipped\")\nassert([first.open, second.open, first.done, second.done], [true, false, null, true])\nfirst.status = \"lost\"\nassert([first.open, first.done], [false, true])\nr = 0\nif \"green\" in tags:\n    r = 1\nassert(r, 0)\ntags.push(\"green\")\nassert(r, 1)\nr";
    let count = common::assertions::protected_assertions(&statements(source)).len();
    assert_eq!(count, 10);
    let case = common::Case {
        title: "membership liveness".to_string(),
        source: source.to_string(),
        expected: Some("1".to_string()),
    };
    common::check_with_assertions(&case, count).unwrap();
}

#[test]
fn null_and_undefined_collections_test_nothing_and_wait() {
    assert_eq!(
        result(
            "n = null\ndef g():\n    pass\no = {a: 1}\n[1 in n, 1 not in n, 1 in o.missing, 1 not in o.missing, 1 in g()]"
        ),
        "[null,null,null,null,null]"
    );
    assert_eq!(
        result(
            "tags = null\nseen = \"a\" in tags\nunseen = \"a\" not in tags\nassert([seen, unseen], [null, null])\ntags = [\"a\"]\n[seen, unseen]"
        ),
        "[true,false]"
    );
    assert_eq!(
        result(
            "class Item\n$Item.hot = \"red\" in $Item.tags\n$Item.cold = \"red\" not in $Item.tags\ncart = Item()\nassert([cart.hot, cart.cold], [null, null])\ncart.tags = [\"blue\"]\nassert([cart.hot, cart.cold], [false, true])\ncart.tags.push(\"red\")\n[cart.hot, cart.cold]"
        ),
        "[true,false]"
    );
    assert_eq!(
        result("n = null\nr = 0\nif 1 not in n:\n    r = 1\nr"),
        "0.0"
    );
}

#[test]
fn other_collections_raise_type_errors_and_roll_back() {
    for (setup, expression, collection) in [
        ("count = 5", "1 in count", "count"),
        ("flag = true", "1 not in flag", "flag"),
        ("record = {a: 1}", "\"a\" in record", "record"),
        ("record = {a: 1}", "\"a\" not in record", "record"),
        ("class Seat\nseat = Seat()", "1 in seat", "seat"),
        ("f = v => v", "1 in f", "f"),
        ("d = Date(0)", "1 in d", "d"),
        ("r = /a/", "1 in r", "r"),
        ("xs = [1]", "1 in xs.length", "xs.length"),
        ("x = Math", "1 in x", "x"),
        ("pass", "\"a\" in {a: 1}", "{\"a\":1}"),
        ("pass", "1 in Math", "Math"),
        ("pass", "1 in Class", "Class"),
        ("pass", "1 in 5", "5"),
        ("pass", "1 in true", "true"),
    ] {
        let line = setup.lines().count() + 1;
        assert_eq!(
            failure(&format!("{setup}\n{expression}")),
            (
                ErrorKind::Type,
                format!("{collection} is not a collection"),
                Some(Position::new(line, 1))
            ),
            "{expression}"
        );
    }
    assert_eq!(
        failure("class Order\n$Order.ok = 1 in $Order\norder = Order()"),
        (
            ErrorKind::Type,
            "Order is not a collection".to_string(),
            Some(Position::new(3, 1))
        )
    );
    let mut runtime = Runtime::new();
    runtime.run("items = [1]\npresent = 1 in items").unwrap();
    let error = runtime.run("items = 7").unwrap_err();
    assert_eq!(error.message(), "items is not a collection");
    let kept = runtime.run("[items, present]").unwrap();
    assert_eq!(runtime.serialize_json(&kept).unwrap(), "[[1.0],true]");
    assert_eq!(
        result("class Math\nm = Math()\n[m in Math, m not in Math]"),
        "[true,false]"
    );
    let item = Value::number(1);
    for collection in [
        Value::List(vec![item.clone()]),
        Value::String("1".to_string()),
        Value::Undefined,
    ] {
        for (operator, spelling) in [(BinaryOp::In, "in"), (BinaryOp::NotIn, "not in")] {
            let error = binary(operator, &item, &collection).unwrap_err();
            assert_eq!(
                (error.kind(), error.message(), error.position()),
                (
                    ErrorKind::Type,
                    format!("'{spelling}' needs the runtime's state"),
                    None
                ),
                "{spelling} {collection}"
            );
        }
    }
}

#[test]
fn in_keeps_its_loop_comprehension_quantifier_and_property_readings() {
    for source in [
        "for x in xs:\n    pass",
        "for k, v in record:\n    pass",
        "for xs:\n    pass",
        "ys = [x for x in xs]",
        "ys = [x for x of xs if x]",
        "ok = all v in xs: v",
        "ok = any v in xs: v",
        "ok = every v in xs: v",
        "ok = record.in",
        "record.in = 1",
        "ok = f(.in)",
        "ok = `${x in xs}`",
        "ok = `${x not in xs}`",
    ] {
        assert!(!holds_membership(source), "{source}");
    }
    assert!(matches!(
        statements("for x in xs:\n    pass").as_slice(),
        [Stmt::For { variable, .. }] if variable == "x"
    ));
    assert!(matches!(
        statements("for k, v in record:\n    pass").as_slice(),
        [Stmt::ForEntries { key, value, .. }] if key == "k" && value == "v"
    ));
    assert_eq!(
        statements("ys = [x for x in xs]"),
        statements("ys = xs.map(x => x)")
    );
    assert_eq!(
        statements("ok = record.in"),
        vec![Stmt::Assign {
            target: name("ok"),
            value: Expr::Member {
                object: Box::new(name("record")),
                property: "in".to_string(),
            },
        }]
    );
    assert_eq!(
        result(
            "record = {a: 1}\nrecord.in = 2\nxs = [1, 2]\n[record.in in xs, all v in xs: v in [1, 2]]"
        ),
        "[true,true]"
    );
}

#[test]
fn ignored_tails_keep_their_accepted_readings() {
    assert_eq!(
        result(
            "x = 5\nxs = [5]\ndef f(v) => v\nc = true\n[`${x in xs}`, `${x not in xs}`, `${x in xs: 1}`, `${(x in xs)}`, `${(x not in xs)}`, `${[x in xs]}`, `${f(x in xs)}`, `${c ? x in xs : 0}`]"
        ),
        "[\"5\",\"5\",\"5\",\"true\",\"false\",\"true\",\"true\",\"true\"]"
    );
    for source in ["x in xs", "x not in xs", "x in xs: v", "x in", "x not"] {
        assert_eq!(parse_expression(source).unwrap(), name("x"), "{source}");
    }
    assert_eq!(parse_expression("all in xs").unwrap(), name("all"));
    assert_eq!(
        parse_expression("(x in xs)").unwrap(),
        membership(BinaryOp::In, name("x"), name("xs"))
    );
    assert_eq!(
        parse_expression("[x not in xs]").unwrap(),
        Expr::List(vec![membership(BinaryOp::NotIn, name("x"), name("xs"))])
    );
    assert_eq!(
        parse_expression("a if x in xs else b").unwrap(),
        parse_expression("(x in xs) ? a : b").unwrap()
    );
    let error = parse_expression("A if c in q else (B").unwrap_err();
    assert_eq!(
        (error.message(), error.position()),
        (
            "Expected ')' but found end of input".to_string(),
            Some(Position::new(1, 20))
        )
    );
}

#[test]
fn malformed_quantifiers_keep_their_accepted_errors_outside_clauses() {
    for (source, message, line, column) in [
        ("ok = all 1 in xs: v", "Unexpected keyword 'in'", 1, 15),
        ("ok = all (v) in xs: v", "Unexpected keyword 'in'", 1, 17),
        ("ok = all this in xs: 1", "Unexpected keyword 'in'", 1, 18),
        ("ok = any 1 in xs: v", "Unexpected keyword 'in'", 1, 15),
        ("ok = every 1 in xs: v", "Unexpected keyword 'in'", 1, 17),
        ("ok = every (v) in xs: v", "Unexpected keyword 'in'", 1, 19),
        ("ok = every this in xs: 1", "Unexpected keyword 'in'", 1, 20),
        ("every all v in xs: v", "Unexpected keyword 'in'", 1, 16),
        ("ok = all 1 not in xs: v", "Unexpected keyword 'in'", 1, 19),
        ("ok = x in xs: v", "Unexpected keyword 'in'", 1, 11),
        ("ok = x not in xs: v", "Unexpected keyword 'in'", 1, 15),
        (
            "ok = (1 in xs: v)",
            "Expected ')' but found keyword 'in'",
            1,
            9,
        ),
        (
            "ok = {a: 1 in xs: 2}",
            "Expected '}' but found keyword 'in'",
            1,
            12,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    for source in [
        "if x in xs: r = 1",
        "if x not in xs:\n    r = 1",
        "while x in xs: xs.pop()",
        "ok = c ? x in xs : 1",
        "ok = ys[x in xs:]",
        "ok = all v in a in b: v",
    ] {
        assert!(holds_membership(source), "{source}");
    }
}

#[test]
fn malformed_membership_tests_report_where_the_parser_stops() {
    for (source, message, line, column) in [
        ("ok = x in", "Unexpected end of input", 1, 10),
        ("ok = x in\n", "Unexpected end of line", 2, 1),
        ("ok = x not in", "Unexpected end of input", 1, 14),
        ("ok = x not", "Unexpected end of input", 1, 11),
        ("ok = (x in )", "Unexpected ')'", 1, 13),
        ("ok = [x in ]", "Unexpected ']'", 1, 13),
        ("ok = x in in xs", "Unexpected keyword 'in'", 1, 14),
        ("ok = x not not in xs", "Unexpected keyword 'in'", 1, 19),
        ("if x in:", "Unexpected ':'", 1, 9),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
}

fn chained(operator: &str, count: usize) -> Result<(), String> {
    let tail = format!(" {operator} b").repeat(count);
    Runtime::check(&format!("ok = a{tail}")).map_err(|error| error.message())
}

fn deepest(opener: &str, base: &str, closer: &str) -> usize {
    let started = Instant::now();
    for depth in 1..200 {
        let source = format!(
            "ok = {}{base}{}",
            opener.repeat(depth),
            closer.repeat(depth)
        );
        let checked = Runtime::check(&source);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{opener} at depth {depth}"
        );
        if let Err(error) = checked {
            assert_eq!(
                error.message(),
                "Expressions are nested too deeply",
                "{opener} at depth {depth}"
            );
            return depth;
        }
    }
    panic!("{opener} never reaches the nesting limit")
}

#[test]
fn membership_tests_keep_the_chain_and_nesting_limits_of_comparisons() {
    for count in [127, 128, 129] {
        assert_eq!(chained("in", count), chained("<", count), "{count}");
        assert_eq!(chained("not in", count), chained("<", count), "{count}");
    }
    assert_eq!(chained("in", 128), Ok(()));
    assert_eq!(
        chained("in", 129),
        Err("Expression has too many operands".to_string())
    );
    let mixed = format!("ok = a{}{}", " < b".repeat(64), " in b".repeat(65));
    assert_eq!(
        Runtime::check(&mixed).unwrap_err().message(),
        "Expression has too many operands"
    );
    for [membership, comparison] in [
        [("x in (", "x", ")"), ("x < (", "x", ")")],
        [("[x in ", "x", "]"), ("[x < ", "x", "]")],
        [("f(x not in ", "x", ")"), ("f(x < ", "x", ")")],
        [("(x in xs) in (", "x", ")"), ("(x < xs) < (", "x", ")")],
    ] {
        let limit = deepest(membership.0, membership.1, membership.2);
        assert!(limit > 15, "{membership:?} stops at {limit}");
        assert_eq!(
            limit,
            deepest(comparison.0, comparison.1, comparison.2),
            "{membership:?}"
        );
    }
}
