mod common;

use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, FunctionBody, Stmt, parse, parse_expression};
use nucleoid::{Error, ErrorKind, Runtime, Value};
use serde_json::json;

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid reads list item properties with selector arguments",
    "Nucleoid reads nested property paths with selector arguments",
    "Nucleoid reads class instance properties with selector arguments",
    "Nucleoid keeps selector arguments live in class level rules",
    "Nucleoid passes selector arguments as ordinary functions",
    "Nucleoid uses selector arguments in headers methods lambdas and templates",
    "Nucleoid rolls back failed selector argument updates",
    "Nucleoid tracks only the properties that selector arguments read",
];
const REFERENCE_TITLE: &str = "Nucleoid reads properties with selector arguments";
const README_TITLE: &str = "Selector arguments read properties without naming a parameter";
const EXAMPLES_TITLE: &str = "Selector arguments keep the stock levels of every bin live";

const CALL_NEAR_MISSES: [(&str, &str, usize, usize); 40] = [
    ("f(.a + 1)", "Unexpected '.'", 2, 4),
    ("f(.a())", "Unexpected '.'", 2, 4),
    ("f(.)", "Unexpected '.'", 2, 4),
    ("f(..a)", "Unexpected '.'", 2, 4),
    ("f(...xs)", "Unexpected '.'", 2, 4),
    ("f(.a.5)", "Unexpected '.'", 2, 4),
    ("f(.a b)", "Unexpected '.'", 2, 4),
    ("f(.$A)", "Unexpected '.'", 2, 4),
    ("f(.\"s\")", "Unexpected '.'", 2, 4),
    ("f(.a[0])", "Unexpected '.'", 2, 4),
    ("f(.a[1:])", "Unexpected '.'", 2, 4),
    ("f(.a@first)", "Unexpected '.'", 2, 4),
    ("f(.a ?? 1)", "Unexpected '.'", 2, 4),
    ("f(.a == 1)", "Unexpected '.'", 2, 4),
    ("f(.a = 1)", "Unexpected '.'", 2, 4),
    ("f(.a => 1)", "Unexpected '.'", 2, 4),
    ("f(.a -> 1)", "Unexpected '.'", 2, 4),
    ("f(.a ? 1 : 2)", "Unexpected '.'", 2, 4),
    ("f(.a if b else c)", "Unexpected '.'", 2, 4),
    ("f(.a and b)", "Unexpected '.'", 2, 4),
    ("f(.a |> why)", "Unexpected '.'", 2, 4),
    ("f(.a.b())", "Unexpected '.'", 2, 4),
    ("f(!.a)", "Unexpected '.'", 2, 5),
    ("f(-.a)", "Unexpected '.'", 2, 5),
    ("f(not .a)", "Unexpected '.'", 2, 8),
    ("f(typeof .a)", "Unexpected '.'", 2, 11),
    ("f((.a))", "Unexpected '.'", 2, 5),
    ("f([.a])", "Unexpected '.'", 2, 5),
    ("f({k: .a})", "Unexpected '.'", 2, 8),
    ("f(x -> .a)", "Unexpected '.'", 2, 9),
    ("f(lambda: .a)", "Unexpected '.'", 2, 12),
    ("f(true ? .a : .b)", "Unexpected '.'", 2, 11),
    ("f(1, .a + 1)", "Unexpected '.'", 2, 7),
    ("f(,.a)", "Unexpected ','", 2, 4),
    ("f(.a .5)", "Unexpected '.'", 2, 4),
    ("f(. 5)", "Unexpected '.'", 2, 5),
    ("f(.\n)", "Unexpected '.'", 3, 1),
    ("f(.a\n+ 1)", "Unexpected '.'", 2, 4),
    ("P(n = .a)", "Unexpected '.'", 2, 8),
    ("f(x)\n.a(1)", "Unexpected '.'", 3, 2),
];

const OTHER_NEAR_MISSES: [(&str, &str, usize, usize); 13] = [
    ("x = .a", "Unexpected '.'", 1, 6),
    ("x = [.a]", "Unexpected '.'", 1, 7),
    ("x = {k: .a}", "Unexpected '.'", 1, 10),
    ("x = (.a)", "Unexpected '.'", 1, 7),
    ("xs[.a]", "Unexpected '.'", 1, 5),
    ("assert: .a, 1", "Unexpected '.'", 1, 10),
    ("assert: 1, .a", "Unexpected '.'", 1, 13),
    ("x = a?.b", "Unexpected '.'", 1, 8),
    ("x = a?? .b", "Unexpected '.'", 1, 10),
    (".a", "Unexpected '.'", 1, 2),
    (".a(1)", "Unexpected '.'", 1, 2),
    ("x -> .a", "Unexpected '.'", 1, 7),
    ("xs = [1]\nxs\n.length", "Unexpected '.'", 3, 2),
];

type Failure = (ErrorKind, String, usize, usize);
type Outcome = Result<serde_json::Value, Failure>;

fn normal(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Number(number) => json!(number.as_f64().unwrap()),
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(normal).collect())
        }
        serde_json::Value::Object(fields) => serde_json::Value::Object(
            fields
                .into_iter()
                .map(|(key, value)| (key, normal(value)))
                .collect(),
        ),
        other => other,
    }
}

fn json_of(runtime: &Runtime, value: &Value) -> serde_json::Value {
    normal(serde_json::from_str(&runtime.serialize_json(value).unwrap()).unwrap())
}

fn failure(error: &Error) -> Failure {
    let position = error.position().expect("errors are positioned");
    (
        error.kind(),
        error.message(),
        position.line,
        position.column,
    )
}

fn outcome_in(runtime: &mut Runtime, source: &str) -> Outcome {
    match runtime.run(source) {
        Ok(value) => Ok(json_of(runtime, &value)),
        Err(error) => Err(failure(&error)),
    }
}

fn outcome(source: &str) -> Outcome {
    outcome_in(&mut Runtime::new(), source)
}

fn evaluate(source: &str) -> serde_json::Value {
    outcome(source).unwrap_or_else(|error| panic!("{source}: {error:?}"))
}

fn error(kind: ErrorKind, message: &str, line: usize, column: usize) -> Outcome {
    Err((kind, message.to_string(), line, column))
}

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "property selector regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
}

fn uses_selector(source: &str) -> bool {
    tokenize(source).unwrap().windows(3).any(|window| {
        matches!(window[0].token, Token::LeftParen | Token::Comma)
            && window[1].token == Token::Dot
            && matches!(window[2].token, Token::Identifier(_) | Token::Keyword(_))
    })
}

fn deep_records() -> String {
    let mut lines = vec!["o126 = {a: 7}".to_string()];
    lines.extend(
        (0..126)
            .rev()
            .map(|index| format!("o{index} = {{a: o{}}}", index + 1)),
    );
    lines.join("\n")
}

fn dependencies(runtime: &Runtime, name: &str) -> BTreeSet<String> {
    runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap_or_else(|| panic!("{name} is filed"))
        .dependencies
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn selectors_parse_to_anonymous_functions_of_one_unnamed_parameter() {
    let statements = parse("f(1, .a.b)").unwrap();
    let [Stmt::Expression(Expr::Call { arguments, .. })] = statements.as_slice() else {
        panic!("{statements:?}");
    };
    let [Expr::Number(_), Expr::Function(function)] = arguments.as_slice() else {
        panic!("{arguments:?}");
    };
    assert!(function.name.is_none());
    let [parameter] = function.parameters.as_slice() else {
        panic!("{function:?}");
    };
    assert!(parameter.type_name.is_none() && parameter.default.is_none() && !parameter.rest);
    assert!(
        tokenize(&parameter.name)
            .unwrap()
            .iter()
            .all(|spanned| spanned.token == Token::Eof)
    );
    let FunctionBody::Expression(body) = &function.body else {
        panic!("{function:?}");
    };
    assert_eq!(body.to_string(), ".a.b");
    assert_eq!(generate(&statements[0]), "f(1,.a.b)");
    for spelling in [
        "f(1, . a .b)",
        "f(1,\n    .a\n    .b\n)",
        "f(1, .a # step\n    .b)",
        "f(1, .a.b,)",
    ] {
        assert_eq!(parse(spelling).unwrap(), statements, "{spelling:?}");
    }
    assert_eq!(parse("f(1, .a /* step */ .b)").unwrap(), statements);
    assert_eq!(
        parse_expression("xs.map(.a)").unwrap().to_string(),
        "xs.map(.a)"
    );
}

#[test]
fn selectors_are_arguments_of_every_parenthesized_argument_list() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"def call(value, key) => key(value)
def id(key) => key
class Box(key): this.key = key
class Base(key): this.key = key
class Child(seed): Base
    def init(seed):
        super(.size)
class Shelf(items):
    this.items = items
    def weight(): return this.items.sum(.kg)
reader = list -> list.map(.a)
tools = {apply: (key, value) -> key(value)}
class Picker:
    def pick(key, value): return key(value)
assert: tools.apply(.a, {a: 11}), 11
assert: Picker().pick(.a, {a: 12}), 12
assert: call({a: 1}, .a), 1
assert: id(.a)({a: 2}), 2
assert: Box(.a).key({a: 3}), 3
assert: new Box(.a).key({a: 4}), 4
assert: Child(1).key({size: 5}), 5
assert: [{a: 6}].map(.a), [6]
assert: call(call({a: {b: 7}}, .a), .b), 7
assert: `${[{a: 8}].map(.a)}`, "8"
assert: reader([{a: 9}]), [9]
assert: Shelf([{kg: 4}, {kg: 6}]).weight(), 10
"#,
    );
    checked(
        &mut runtime,
        r#"flags = []
if [{p: true}].some(.p):
    flags.push("if")
queue = [{open: true}, {open: false}]
while queue.count(.open) > 0:
    queue.shift()
total = 0
for box of Box.filter(.key):
    total += 1
note = "many" if queue.count(.open) > 1 else "few"
pick = [{a: 1}].some(.a) ? "yes" : "no"
assert: [flags, queue.length, total, note, pick], [["if"], 1, 2, "few", "yes"]
"#,
    );
}

#[test]
fn selector_names_are_the_names_of_member_access() {
    for name in [
        "if",
        "else",
        "elif",
        "for",
        "of",
        "in",
        "class",
        "def",
        "function",
        "return",
        "throw",
        "delete",
        "try",
        "catch",
        "pass",
        "not",
        "and",
        "or",
        "true",
        "false",
        "null",
        "None",
        "undefined",
        "super",
        "this",
        "self",
        "typeof",
        "new",
        "lambda",
        "assert",
        "suppose",
        "while",
        "extends",
        "all",
        "any",
        "sum",
        "max",
        "length",
        "e5",
        "_x",
        "x1",
    ] {
        let member = parse_expression(&format!("x.{name}")).unwrap();
        let Expr::Member { property, .. } = &member else {
            panic!("{member:?}");
        };
        let selector = parse_expression(&format!("f(.{name})")).unwrap();
        assert_eq!(selector.to_string(), format!("f(.{property})"), "{name}");
        let source =
            format!("r = {{\"{property}\": 3}}\n[[r].map(.{name}), [r].map(v -> v.{name})]");
        assert_eq!(evaluate(&source), normal(json!([[3], [3]])), "{name}");
    }
}

#[test]
fn incomplete_selectors_keep_their_accepted_diagnostics() {
    for (call, message, line, column) in CALL_NEAR_MISSES {
        let source = format!("def f(x) => x\n{call}");
        assert_eq!(
            outcome(&source),
            error(ErrorKind::Syntax, message, line, column),
            "{call:?}"
        );
    }
    for (source, message, line, column) in OTHER_NEAR_MISSES {
        assert_eq!(
            outcome(source),
            error(ErrorKind::Syntax, message, line, column),
            "{source:?}"
        );
    }
}

#[test]
fn a_complete_selector_moves_a_former_error_to_the_argument_after_it() {
    assert_eq!(
        outcome("def f(x) => x\nf(.a, .b + 1)"),
        error(ErrorKind::Syntax, "Unexpected '.'", 2, 8)
    );
    assert_eq!(
        outcome("def f(x) => x\nf(.a,,)"),
        error(ErrorKind::Syntax, "Unexpected ','", 2, 7)
    );
}

#[test]
fn tentative_statements_accept_selectors_in_their_values() {
    let records = "xs = [{a: 1, length: 4}, {a: 2}]\n";
    for (statement, read, expected) in [
        (
            "first, second = xs.map(.a)",
            "[first, second]",
            json!([1, 2]),
        ),
        ("first, = xs.map(.a)", "first", json!(1)),
        ("{length} = xs.map(.a)", "length", json!(2)),
        ("{length: count} = xs.map(.a)", "count", json!(2)),
        ("{a} = xs.find(.length)", "a", json!(1)),
    ] {
        assert_eq!(
            evaluate(&format!("{records}{statement}\n{read}")),
            normal(expected),
            "{statement}"
        );
    }
    assert_eq!(
        outcome(&format!("{records}first, second = xs.map(.a + 1)")),
        error(ErrorKind::Syntax, "Unexpected ','", 2, 8)
    );
}

#[test]
fn numbers_and_member_access_keep_their_accepted_readings() {
    assert_eq!(evaluate("def f(x) => x\nf(.5)"), normal(json!(0.5)));
    assert_eq!(evaluate("def f(x) => x\nf(.5e1)"), normal(json!(5)));
    for spelling in ["f(r .b)", "f(r. b)", "f(r\n.b)", "f(r\n    .b\n)"] {
        assert_eq!(
            evaluate(&format!("r = {{b: 4}}\ndef f(x) => x\n{spelling}")),
            normal(json!(4)),
            "{spelling:?}"
        );
    }
}

#[test]
fn selector_paths_hold_at_most_127_properties() {
    let path = ".a".repeat(127);
    assert_eq!(
        evaluate(&format!(
            "{}\n[[o0].map({path}), [o0].map(v -> v{path})]",
            deep_records()
        )),
        normal(json!([[7], [7]]))
    );
    let too_many = error(ErrorKind::Syntax, "Expression has too many parts", 2, 257);
    for count in [128, 200] {
        let path = ".a".repeat(count);
        assert_eq!(outcome(&format!("def f(x) => x\nf({path})")), too_many);
        assert_eq!(outcome(&format!("def f(x) => x\nf({path} + 1)")), too_many);
    }
    assert_eq!(
        outcome(&format!("def f(x) => x\nf({path} + 1)")),
        error(ErrorKind::Syntax, "Unexpected '.'", 2, 4)
    );
    assert_eq!(
        outcome(&format!("f = v -> v{}", ".a".repeat(128))),
        error(ErrorKind::Syntax, "Expression has too many parts", 1, 267)
    );
}

#[test]
fn nested_selector_arguments_meet_the_nesting_limit_of_any_argument() {
    let mut nested = 0;
    for depth in [50, 60, 61, 62, 63, 64, 65, 70] {
        let open = "f(".repeat(depth);
        let close = ")".repeat(depth);
        let prefix = "def f(x) => x\nv = {a: 1}\ntypeof ";
        let selector = outcome(&format!("{prefix}{open}.a{close}"));
        let name = outcome(&format!("{prefix}{open}v{close}"));
        match (&selector, &name) {
            (Ok(selector), Ok(name)) => {
                assert_eq!(selector, &json!("Function"), "{depth}");
                assert_eq!(name, &json!("Object"), "{depth}");
            }
            (Err(selector), Err(name)) => {
                assert_eq!(selector, name, "{depth}");
                assert_eq!(selector.1, "Expressions are nested too deeply");
                nested += 1;
            }
            _ => panic!("{depth}: {selector:?} and {name:?}"),
        }
    }
    assert!((1..8).contains(&nested));
}

#[test]
fn selector_evaluation_meets_the_depth_limit_where_the_lambda_does() {
    let records = deep_records();
    let path = ".a".repeat(127);
    let mut values = 0;
    let mut limits = 0;
    for extra in [0, 40, 55, 56, 57, 58, 59, 60, 61, 62, 64, 80, 100] {
        let tail = " + 0".repeat(extra);
        let program = |key: &str| format!("{records}\ndef ap(g, x) => g(x)\nap({key}, o0){tail}");
        let selector = outcome(&program(&path));
        assert_eq!(
            selector,
            outcome(&program(&format!("v -> v{path}"))),
            "{extra}"
        );
        match selector {
            Ok(value) => {
                assert_eq!(value, normal(json!(7)));
                values += 1;
            }
            Err((kind, message, _, _)) => {
                assert_eq!(
                    (kind, message.as_str()),
                    (ErrorKind::Type, "Maximum expression depth exceeded")
                );
                limits += 1;
            }
        }
    }
    assert!(values > 0 && limits > 0);
}

#[test]
fn selectors_read_what_the_lambda_reads_on_every_receiver() {
    let setup = "class Item(a): this.a = a\nItem({b: 5})\nItem(null)\nItem([1, 2])";
    for receiver in [
        "[{a: 1}, {a: null}, {}, {a: {b: 2}}, {a: {b: {c: 3}}}]",
        "[1, true, \"text\", [1, 2, 3], null, {length: 4}]",
        "[x -> x, /a/, new Date(0)]",
        "Item",
        "Item[1:]",
    ] {
        for path in [
            "a", "a.b", "a.b.c", "length", "a.length", "b", "this", "class",
        ] {
            let selector = format!("{receiver}.map(.{path})");
            let lambda = format!("{receiver}.map(v -> v.{path})");
            assert_eq!(
                outcome(&format!("{setup}\n{selector}")),
                outcome(&format!("{setup}\n{lambda}")),
                "{selector}"
            );
            assert_eq!(
                outcome(&format!("{setup}\nys = {selector}\nys")),
                outcome(&format!("{setup}\nys = {lambda}\nys")),
                "ys = {selector}"
            );
        }
    }
}

#[test]
fn selectors_take_only_their_first_argument() {
    assert_eq!(evaluate("def call(g) => g()\ncall(.a)"), json!(null));
    assert_eq!(
        evaluate("def call(g) => g({a: 1}, {a: 2})\ncall(.a)"),
        normal(json!(1))
    );
    assert_eq!(
        evaluate(
            "chain = {at: 1, next: {at: 2, next: {at: 3}}}\n[[1].reduce(.next, chain).at, [1, 2].reduce(.next, chain).at]"
        ),
        normal(json!([2, 3]))
    );
    let items = "xs = [{k: 2}, {k: 1}, {k: 3}]";
    assert_eq!(
        evaluate(&format!("{items}\nxs.sort(.k)")),
        evaluate(&format!("{items}\nxs.sort(a -> a.k)"))
    );
    assert_eq!(
        outcome(&format!("{items}\nxs.forEach(.k)")),
        outcome(&format!("{items}\nxs.forEach(a -> a.k)"))
    );
    assert_eq!(
        outcome(&format!("{items}\nxs.forEach(.k)")),
        error(ErrorKind::Type, "xs.forEach is not a function", 2, 1)
    );
}

#[test]
fn selector_values_are_functions_that_show_no_parameter() {
    assert_eq!(
        evaluate("def id(g) => g\n[typeof id(.a), String(id(.a)), `${id(.a)}`, [id(.a)]]"),
        json!(["Function", "[function]", "[function]", ["[function]"]])
    );
    assert_eq!(
        evaluate("def id(g) => g\ng := id(.a.b)\n(why g)[0].rule"),
        json!("g = .a.b")
    );
}

#[test]
fn selectors_stay_live_where_the_lambda_stays_live() {
    let steps: [(&str, &str); 16] = [
        (
            "class Line(sku, qty, item):\n    this.sku = sku\n    this.qty = qty\n    this.item = item",
            "",
        ),
        ("class Order(lines): this.lines = lines", ""),
        (
            "$Order.units = $Order.lines.sum(.qty)",
            "$Order.units = $Order.lines.sum(v -> v.qty)",
        ),
        (
            "$Order.names = $Order.lines.map(.item.name)",
            "$Order.names = $Order.lines.map(v -> v.item.name)",
        ),
        (
            "busiest = Order.maxBy(.units)",
            "busiest = Order.maxBy(v -> v.units)",
        ),
        (
            "bulk = Line.filter(.qty).count(.item.bulk)",
            "bulk = Line.filter(v -> v.qty).count(v -> v.item.bulk)",
        ),
        ("pen = {name: \"pen\", bulk: false}", ""),
        ("ink = {name: \"ink\", bulk: true}", ""),
        ("a = Line(\"a\", 2, pen)", ""),
        ("b = Line(\"b\", 0, ink)", ""),
        ("first = Order([a, b])", ""),
        ("second = Order([])", ""),
        ("b.qty = 9", ""),
        ("ink.name = \"ink jet\"", ""),
        ("c = Line(\"c\", 20, ink)\nsecond.lines.push(c)", ""),
        ("first.lines = [a]\ndelete b.item", ""),
    ];
    let probe = "[first.units, first.names, second.units, second.names, busiest, bulk]";
    let mut selector = Runtime::new();
    let mut lambda = Runtime::new();
    let mut settled = 0;
    for (step, spelled) in steps {
        let other = if spelled.is_empty() { step } else { spelled };
        assert_eq!(
            outcome_in(&mut selector, step),
            outcome_in(&mut lambda, other),
            "{step}"
        );
        let observed = outcome_in(&mut selector, probe);
        assert_eq!(observed, outcome_in(&mut lambda, probe), "after {step}");
        settled += usize::from(observed.is_ok());
    }
    let second = outcome_in(&mut selector, "second").unwrap();
    assert_eq!(
        outcome_in(&mut selector, probe),
        Ok(normal(json!([2, ["pen"], 20, ["ink jet"], second, 1])))
    );
    assert_eq!(settled, 5);
}

#[test]
fn selectors_record_the_dependencies_of_the_lambda() {
    let program = |hottest: &str, rooms: &str| {
        format!(
            "class Sensor(room, temp):\n    this.room = room\n    this.temp = temp\nhottest = Sensor.maxBy({hottest})\nrooms = Sensor.map({rooms})\nlab = Sensor(\"lab\", 21)\nhall = Sensor(\"hall\", 25)"
        )
    };
    let mut selector = Runtime::new();
    checked(&mut selector, &program(".temp", ".room"));
    let mut lambda = Runtime::new();
    checked(&mut lambda, &program("s -> s.temp", "s -> s.room"));
    for name in ["hottest", "rooms"] {
        assert_eq!(
            dependencies(&selector, name),
            dependencies(&lambda, name),
            "{name}"
        );
    }
    let hottest = dependencies(&selector, "hottest");
    assert!(hottest.contains("lab.temp") && hottest.contains("hall.temp"));
    assert!(!hottest.contains("lab.room") && !hottest.contains("hall.room"));
    let rooms = dependencies(&selector, "rooms");
    assert!(rooms.contains("lab.room") && !rooms.contains("lab.temp"));
}

#[test]
fn failed_updates_through_selectors_roll_back() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "def checked(total):\n    if total > 100:\n        throw \"TOO_MANY\"\n    return total\ncart = [{qty: 2}, {qty: 5}]\nunits = cart.sum(.qty)\nlargest = cart.maxBy(.qty)\ncap = checked(units)",
    );
    let failed = runtime.run("cart.push({qty: 500})").unwrap_err();
    assert_eq!(
        failed.thrown_value(),
        Some(&Value::String("TOO_MANY".to_string()))
    );
    assert_eq!(
        outcome_in(&mut runtime, "[units, cap, largest.qty, cart.length]"),
        Ok(normal(json!([7, 7, 5, 2])))
    );
    checked(
        &mut runtime,
        "failure = \"\"\ntry:\n    cart.push({qty: 400})\ncatch error:\n    failure = error\nassert: [failure, units, cart.length], [\"TOO_MANY\", 7, 2]\nsuppose:\n    cart.push({qty: 30})\n    assert: [units, cap, largest.qty], [37, 37, 30]\nassert: [units, cap, largest.qty, cart.length], [7, 7, 5, 2]",
    );
}

#[test]
fn value_inside_a_selector_keeps_the_snapshot_meaning() {
    assert_eq!(
        evaluate("[{value: 3}].map(.value)"),
        normal(json!([{"value": 3}]))
    );
    let program = |frozen: &str, live: &str| {
        format!(
            "p = {{a: 1}}\nxs = [p]\nfrozen = xs.map({frozen})\nlive = xs.map({live})\np.a = 2\n[frozen, live]"
        )
    };
    assert_eq!(
        evaluate(&program(".a.value", ".a")),
        normal(json!([[1], [2]]))
    );
    assert_eq!(
        evaluate(&program(".a.value", ".a")),
        evaluate(&program("v -> v.a.value", "v -> v.a"))
    );
    let missing = "r = {}\n";
    for selector in [".value", ".value.a"] {
        assert_eq!(
            outcome(&format!("{missing}[r.missing].map({selector})")),
            error(ErrorKind::Reference, ".value is not defined", 2, 1)
        );
    }
    assert_eq!(
        outcome("[{}].map(.a.value)"),
        error(ErrorKind::Reference, ".a is not defined", 1, 1)
    );
    assert_eq!(
        outcome("[{a: {}}].map(.a.b.value)"),
        error(ErrorKind::Reference, ".a.b is not defined", 1, 1)
    );
    assert_eq!(
        outcome(&format!("{missing}[r.missing].map(v -> v.value)")),
        error(ErrorKind::Reference, "v is not defined", 2, 1)
    );
    assert_eq!(
        outcome("[{}].map(v -> v.a.value)"),
        error(ErrorKind::Reference, "v.a is not defined", 1, 1)
    );
}

#[test]
fn null_steps_settle_declarations_as_they_do_through_the_lambda() {
    for spelling in [".to.city", "v -> v.to.city"] {
        let mut runtime = Runtime::new();
        checked(
            &mut runtime,
            &format!(
                "oslo = {{to: {{city: \"Oslo\"}}}}\nrows = [oslo, {{}}]\ncities = rows.map({spelling})"
            ),
        );
        assert_eq!(
            outcome_in(&mut runtime, "cities"),
            Ok(normal(json!(["Oslo", null])))
        );
        checked(&mut runtime, "oslo.to = null");
        assert_eq!(outcome_in(&mut runtime, "cities"), Ok(json!(null)));
        assert_eq!(
            outcome_in(&mut runtime, &format!("rows.map({spelling})")),
            Ok(json!([null, null]))
        );
    }
}

#[test]
fn selectors_render_as_written_and_never_show_their_parameter() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"class Sensor(room, temp, info):
    this.room = room
    this.temp = temp
    this.info = info
hottest = Sensor.maxBy(.temp)
labels = Sensor.map(.info.self)
note = `${Sensor.map(.room).join(",")}`
def id(g) => g
picked := id(.room)
if Sensor.some(.temp):
    warm = true
if Sensor.some(.room):
    warm = true
lab = Sensor("lab", 21, {"this": "L"})
hall = Sensor("hall", 25, {"this": "H"})
"#,
    );
    assert_eq!(
        outcome_in(
            &mut runtime,
            "[(why hottest)[0].rule, (why labels)[0].rule, (why note)[0].rule, (why picked)[0].rule]"
        ),
        Ok(json!([
            "hottest = Sensor.maxBy(.temp)",
            "labels = Sensor.map(.info.this)",
            "note = `${Sensor.map(.room).join(\",\")}`",
            "picked = .room"
        ]))
    );
    assert_eq!(
        outcome_in(&mut runtime, "(affects lab.temp).map(s -> s.rule)"),
        Ok(json!([
            "hottest = Sensor.maxBy(.temp)",
            "if(Sensor.some(.temp))"
        ]))
    );
    assert_eq!(
        outcome_in(&mut runtime, "(why hottest)[0].from"),
        Ok(json!(["$Sensor", "lab.temp", "hall.temp"]))
    );
    let keys: Vec<String> = runtime
        .graph
        .nodes()
        .map(|node| node.key.to_string())
        .collect();
    for key in ["if(Sensor.some(.temp))", "if(Sensor.some(.room))"] {
        assert!(keys.iter().any(|found| found == key), "{key}: {keys:?}");
    }
    for key in &keys {
        assert!(!key.is_empty() && !key.starts_with('.'), "{key}");
        assert!(!key.contains("function()") && !key.contains("(.)"), "{key}");
    }
    assert!(!runtime.state.has_variable(""));
    assert_eq!(
        outcome("[{a: 1}].map(.a).nope()"),
        error(
            ErrorKind::Type,
            "[{\"a\":1}].map(.a).nope is not a function",
            1,
            1
        )
    );
    assert_eq!(
        outcome("def f(x) => x\nf(1, .a.b)(2)"),
        error(ErrorKind::Type, "f(1,.a.b) is not a function", 2, 1)
    );
    let mut lambdas = Runtime::new();
    checked(
        &mut lambdas,
        "xs = [{p: false, q: true}]\nif xs.some(x -> x.q):\n    flag = 1\nif xs.some(x -> x.p):\n    flag = 1",
    );
    let mut selectors = Runtime::new();
    checked(
        &mut selectors,
        "xs = [{p: false, q: true}]\nif xs.some(.q):\n    flag = 1\nif xs.some(.p):\n    flag = 1",
    );
    let conditions = |runtime: &Runtime| {
        runtime
            .graph
            .nodes()
            .filter(|node| node.key.as_str().starts_with("if("))
            .count()
    };
    assert_eq!((conditions(&lambdas), conditions(&selectors)), (1, 2));
}

#[test]
fn cli_runs_selector_files_with_json_and_graph_output() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("property-selectors.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().next(), Some("270.0"));
    assert!(stdout.contains("if(Bid.some(.open))"), "{stdout}");
    assert!(!stdout.contains("function()"), "{stdout}");
}

#[test]
fn repl_buffers_selector_lines_and_keeps_dot_commands() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            b"class Bid(amount):\n    this.amount = amount\n\n.json on\nBid(5)\nBid(9)\ndef top(list):\n    return list.maxBy(\n        .amount\n    )\n\ntop(Bid).amount\n[Bid.map(.amount), Bid.sum(.amount)]\nranked = Bid.map(.amount)\n(why ranked)[0].rule\n.exit\n",
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("json on."), "{stdout}");
    assert!(stdout.lines().any(|line| line == "9.0"), "{stdout}");
    assert!(
        stdout.lines().any(|line| line == "[[5.0,9.0],14.0]"),
        "{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line == "\"ranked = Bid.map(.amount)\""),
        "{stdout}"
    );
}

#[test]
fn the_appended_cases_use_selectors_and_run_every_assertion() {
    let documents = [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ];
    let appended: Vec<&str> = SPEC_TITLES
        .into_iter()
        .chain([REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE])
        .collect();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flatten() {
        cases += 1;
        let new = appended.contains(&case.title.as_str());
        assert_eq!(uses_selector(&case.source), new, "{}", case.title);
        if new {
            let count =
                common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
            common::check_with_assertions(case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assertions += count;
        }
    }
    for title in &appended {
        let matching = documents
            .iter()
            .flatten()
            .filter(|case| case.title == *title)
            .count();
        assert_eq!(matching, 1, "{title}");
    }
    assert_eq!(assertions, 37);
    assert_eq!(cases, 1044);
}
