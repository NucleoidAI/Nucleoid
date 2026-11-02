mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::sync::Arc;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::parse;
use nucleoid::{ObjectId, Runtime, Value};
use serde_json::json;

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "list mode regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count);
}

fn run(source: &str) -> Runtime {
    let mut runtime = Runtime::new();
    checked(&mut runtime, source);
    runtime
}

fn dependencies<'a>(runtime: &'a Runtime, key: &NodeKey) -> Vec<&'a str> {
    runtime
        .graph
        .retrieve(key)
        .unwrap_or_else(|| panic!("{key} is filed"))
        .dependencies
        .iter()
        .map(NodeKey::as_str)
        .collect()
}

fn variable_dependencies<'a>(runtime: &'a Runtime, name: &str) -> Vec<&'a str> {
    dependencies(runtime, &NodeKey::variable(name))
}

fn value(runtime: &mut Runtime, source: &str) -> Value {
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
}

/// The same value, not merely an equal one: objects by identity, functions
/// and regular expressions by pointer, and numbers by their bits, so `0` and
/// `-0` differ while every NaN is the same.
fn identical(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Undefined, Value::Undefined) | (Value::Null, Value::Null) => true,
        (Value::Number(left), Value::Number(right)) => {
            left.to_bits() == right.to_bits() || (left.is_nan() && right.is_nan())
        }
        (Value::Bool(left), Value::Bool(right)) => left == right,
        (Value::String(left), Value::String(right)) => left == right,
        (Value::Date(left), Value::Date(right)) => left == right,
        (Value::Class(left), Value::Class(right)) => left == right,
        (Value::Object(left), Value::Object(right)) => left == right,
        (Value::Function(left), Value::Function(right)) => Arc::ptr_eq(left, right),
        (Value::Regex(left), Value::Regex(right)) => Arc::ptr_eq(left, right),
        (Value::List(left), Value::List(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| identical(left, right))
        }
        _ => false,
    }
}

/// The position `mode` must return among `length` items, computed the slow
/// way, where `matches(kept, item)` is `[values[kept]].includes(values[item])`:
/// an item is kept unless a kept item matches it, every other item counts
/// for the earliest kept item that matches it, and the first of the highest
/// counts wins.
fn expected_position(length: usize, matches: impl Fn(usize, usize) -> bool) -> Option<usize> {
    let mut kept: Vec<(usize, usize)> = Vec::new();
    for item in 0..length {
        match kept
            .iter_mut()
            .find(|(position, _)| matches(*position, item))
        {
            Some((_, count)) => *count += 1,
            None => kept.push((item, 1)),
        }
    }
    let highest = kept.iter().map(|&(_, count)| count).max()?;
    kept.iter()
        .find(|&&(_, count)| count == highest)
        .map(|&(position, _)| position)
}

fn booleans(value: Value) -> Vec<bool> {
    match value {
        Value::List(items) => items
            .into_iter()
            .map(|item| match item {
                Value::Bool(flag) => flag,
                other => panic!("a match is a boolean, not {other:?}"),
            })
            .collect(),
        other => panic!("matches are a list, not {other:?}"),
    }
}

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid finds the most frequent list item with mode",
    "Nucleoid finds the most frequent value of class instance populations",
    "Nucleoid compares mode items with deep equality",
    "Nucleoid calls the mode selector once per item with only the item",
    "Nucleoid keeps modes live in class level rules",
    "Nucleoid uses modes in functions methods and conditional expressions",
    "Nucleoid keeps mode as an ordinary name beside the mode query",
    "Nucleoid rolls back failed mode updates",
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
            "Nucleoid finds the most frequent items of lists and populations",
        ),
        (
            &readme,
            "Modes keep the most requested size live as orders change",
        ),
        (
            &examples,
            "Modes keep the most common defect of each production line live",
        ),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(case.source.contains(".mode("), "{title} uses mode");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 87);
}

#[test]
fn no_earlier_document_case_calls_mode() {
    let documents = [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ];
    let appended = [
        "Nucleoid finds the most frequent items of lists and populations",
        "Modes keep the most requested size live as orders change",
        "Modes keep the most common defect of each production line live",
    ];
    let mut with_mode = 0;
    for case in documents.iter().flatten() {
        let uses = case.source.contains(".mode(");
        let new =
            SPEC_TITLES.contains(&case.title.as_str()) || appended.contains(&case.title.as_str());
        assert_eq!(uses, new, "{}", case.title);
        with_mode += usize::from(uses);
    }
    assert_eq!(with_mode, 11);
}

#[test]
fn mode_accepts_every_receiver_the_list_queries_accept() {
    let (mut run, _) = common::runners();
    run(r#"
visits = ["lab", "den", "lab", "hall", "den"]
cart = {items: [2, 1, 2]}
history = [1]
history.push(1)
history.push(2)
class Order(region):
    this.region = region
Order("north")
Order("south")
Order("north")
Order("east")
kind = Order
class Empty
"#);
    for (source, expected) in [
        ("[1, 3, 1, 2, 3].mode()", json!(1)),
        ("visits.mode()", json!("lab")),
        ("cart.items.mode()", json!(2)),
        (
            "visits.filter(visit -> visit != \"lab\").mode()",
            json!("den"),
        ),
        ("visits.map(visit -> visit.length).mode()", json!(3)),
        ("visits.slice(1).mode()", json!("den")),
        ("visits.concat([\"hall\", \"hall\"]).mode()", json!("hall")),
        ("visits[2:].mode()", json!("lab")),
        ("visits.value.mode()", json!("lab")),
        (
            "[[1, 2], [3]].flatMap(row -> row.concat([3])).mode()",
            json!(3),
        ),
        ("history@first.mode()", json!(1)),
        ("history.mode()", json!(1)),
        ("typeof [].mode()", json!("undefined")),
        ("visits.mode(visit -> visit.length)", json!(3)),
        ("Order.mode(order -> order.region)", json!("north")),
        ("Order[1:].mode(order -> order.region)", json!("south")),
        ("Order[:3].mode(order -> order.region)", json!("north")),
        ("Order.list().mode(order -> order.region)", json!("north")),
        (
            "Order.filter(order -> order.region != \"north\").mode(order -> order.region)",
            json!("south"),
        ),
        ("kind.mode(order -> order.region)", json!("north")),
        ("Order.mode() == Order[0]", json!(true)),
        ("typeof Empty.mode()", json!("undefined")),
        ("typeof Empty.mode(empty -> empty.size)", json!("undefined")),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
}

#[test]
fn mode_counts_each_item_for_the_earliest_kept_value_that_includes_it() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let atoms = [
        "0",
        "-0",
        "1",
        "2",
        "Number.NaN",
        "null",
        "holder.missing",
        "\"1\"",
        "\"a\"",
        "true",
        "false",
        "Date(1)",
        "Robot",
        "ada",
        "twin",
        "shared",
        "helper",
        "pattern",
        "[]",
        "[1]",
        "[1, 2]",
        "[Number.NaN]",
        "[null]",
        "[holder.missing]",
        "[-0]",
        "[ada]",
        "{}",
        "{a: 1}",
        "{a: 1, b: 2}",
        "{b: 2, a: 1}",
        "{a: null}",
        "{a: holder.missing}",
        "{a: {c: 1}}",
        "{a: {c: 2}}",
        "{k: null, m: 1}",
        "{k: holder.missing, m: 1, z: 5}",
        "{m: 1, z: 5}",
        "{l: null, m: 1}",
        "{k: holder.missing, l: holder.missing, m: 1, z: 5}",
        "{id: \"ada\", name: \"ada\"}",
    ];
    let mut runtime = Runtime::new();
    runtime
        .run("holder = {}\nclass Robot(name): this.name = name\nada = Robot(\"ada\")\ntwin = Robot(\"ada\")\nshared = {a: 1}\ndef helper(value) => value\npattern = /a/")
        .unwrap();
    let mut lists = vec![
        String::new(),
        "3, 4, 4, 3".to_string(),
        "{k: null, m: 1}, {k: holder.missing, m: 1, z: 5}".to_string(),
        "{k: holder.missing, m: 1, z: 5}, {k: null, m: 1}".to_string(),
        "{k: null, m: 1}, {m: 1, z: 5}, {k: holder.missing, m: 1, z: 5}".to_string(),
        "{m: 1, z: 5}, {k: null, m: 1}, {k: holder.missing, m: 1, z: 5}".to_string(),
        "{k: null, m: 1}, {l: null, m: 1}, {l: null, m: 1}, {k: holder.missing, l: holder.missing, m: 1, z: 5}"
            .to_string(),
        "shared, shared, {a: 1}".to_string(),
        "-0, 0, 0".to_string(),
        "0, -0, -0".to_string(),
        "holder.missing, null, 1".to_string(),
        "null, holder.missing, 1".to_string(),
        "Number.NaN, Number.NaN, 1".to_string(),
        "1, Number.NaN, 1".to_string(),
        "Date(1), 1, 1".to_string(),
        "helper, helper, pattern, pattern".to_string(),
    ];
    for _ in 0..500 {
        let length = (next() % 11) as usize;
        let items: Vec<&str> = (0..length)
            .map(|_| atoms[(next() % atoms.len() as u64) as usize])
            .collect();
        lists.push(items.join(", "));
    }
    for items in &lists {
        runtime.run(&format!("values = [{items}]")).unwrap();
        let length = match value(&mut runtime, "values") {
            Value::List(values) => values.len(),
            other => panic!("[{items}] gave {other:?}"),
        };
        for (wrapped, query) in [
            (false, "values.mode()"),
            (true, "values.mode(item -> [item])"),
        ] {
            let operand = |position: usize| {
                if wrapped {
                    format!("[values[{position}]]")
                } else {
                    format!("values[{position}]")
                }
            };
            let mut matrix = Vec::with_capacity(length);
            for kept in 0..length {
                let row: Vec<String> = (0..length)
                    .map(|item| format!("[{}].includes({})", operand(kept), operand(item)))
                    .collect();
                matrix.push(booleans(value(
                    &mut runtime,
                    &format!("[{}]", row.join(", ")),
                )));
            }
            let expected = match expected_position(length, |kept, item| matrix[kept][item]) {
                Some(position) => value(&mut runtime, &operand(position)),
                None => Value::Undefined,
            };
            let actual = value(&mut runtime, query);
            assert!(
                identical(&actual, &expected),
                "[{items}]: {query} gave {actual:?}, expected {expected:?}"
            );
        }
    }
}

#[test]
fn ties_go_to_the_value_that_occurs_first() {
    let (mut repl, _) = common::runners();
    repl("holder = {}\nfirst = {x: 1}\nsecond = {x: 1}\nthird = {x: 2}");
    for (source, expected) in [
        ("[3, 4, 4, 3].mode()", json!(3)),
        ("[4, 3, 3, 4].mode()", json!(4)),
        ("[3, 4, 4].mode()", json!(4)),
        ("[1, 2, 3].mode()", json!(1)),
        ("[\"b\", \"a\", \"a\", \"b\", \"c\"].mode()", json!("b")),
        ("[5].mode()", json!(5)),
        ("String(1 / [-0, 0, 0].mode())", json!("-Infinity")),
        ("String(1 / [0, -0, -0].mode())", json!("Infinity")),
        ("[-0, 0, 1, 1, 1].mode()", json!(1)),
        (
            "typeof [holder.missing, null, 1].mode()",
            json!("undefined"),
        ),
        ("[null, holder.missing, 1].mode()", json!(null)),
        ("typeof [null, holder.missing, 1].mode()", json!("null")),
        ("[1, null, null].mode()", json!(null)),
        ("[third, first, second].mode() == first", json!(true)),
        ("[third, first, second].mode() == second", json!(false)),
        ("[first, third, second, third].mode() == first", json!(true)),
        ("[third, third, first, second].mode() == third", json!(true)),
    ] {
        assert_eq!(repl(source), expected, "{source}");
    }
    let mut runtime = run(r#"
sizes = ["M", "S", "M", "L"]
top = sizes.mode()
assert(top, "M")
sizes.push("L")
assert(top, "M")
sizes.push("L")
assert(top, "L")
sizes.shift()
assert(top, "L")
"#);
    checked(
        &mut runtime,
        "sizes = [\"S\", \"L\", \"S\", \"L\"]\nassert(top, \"S\")\nsizes.shift()\nassert(top, \"L\")",
    );
}

#[test]
fn items_are_compared_with_deep_equality_never_converted() {
    let (mut run, _) = common::runners();
    run(r#"
holder = {}
class Robot(name):
    this.name = name
ada = Robot("ada")
twin = Robot("ada")
shared = {k: 1}
def helper(value) => value
pattern = /a/
a = {k: null, m: 1}
b = {k: holder.missing, m: 1, z: 5}
plain = {m: 1, z: 5}
other = {l: null, m: 1}
both = {k: holder.missing, l: holder.missing, m: 1, z: 5}
"#);
    for (source, expected) in [
        ("[1, \"1\", \"1\", true, true].mode()", json!("1")),
        ("[2, \"2\", 2].mode()", json!(2)),
        ("[true, 1, true].mode()", json!(true)),
        ("[0, false, \"\", false].mode()", json!(false)),
        ("[\"10\", 10, \" 10 \", 10].mode()", json!(10)),
        ("typeof [Date(5), 5, \"5\", 5].mode()", json!("Date")),
        ("[5, Date(5), Date(5)].mode()", json!(5)),
        ("[Robot, ada, Robot].mode() == Robot", json!(true)),
        (
            "[{x: 1, y: 2}, {y: 2, x: 1}, {x: 2, y: 1}].mode()",
            json!({"x": 1, "y": 2}),
        ),
        (
            "[{x: 1}, {x: 1, y: 2}, {x: 1, y: 2}].mode()",
            json!({"x": 1, "y": 2}),
        ),
        ("[{x: null}, {}, {x: null}].mode()", json!({"x": null})),
        ("[[1, 2], [2, 1], [2, 1]].mode()", json!([2, 1])),
        ("[[1, [2]], [1, [2]], [1]].mode()", json!([1, [2]])),
        ("[[], [], [1]].mode()", json!([])),
        ("[[ada], [twin], [twin]].mode()[0] == twin", json!(true)),
        ("[ada, twin, twin].mode() == twin", json!(true)),
        ("[ada, twin].mode() == ada", json!(true)),
        (
            "[shared, {k: 1}, {k: 2}, {k: 2}].mode() == shared",
            json!(true),
        ),
        (
            "[ada, {id: \"ada\", name: \"ada\"}, twin].mode() == ada",
            json!(true),
        ),
        ("typeof [helper, helper, 1].mode()", json!("Function")),
        ("[helper, 1, 1].mode()", json!(1)),
        ("typeof [pattern, pattern].mode()", json!("RegExp")),
        ("[pattern, 2, 2].mode()", json!(2)),
        ("String([Number.NaN, Number.NaN, 1].mode())", json!("NaN")),
        ("[1, Number.NaN, 1].mode()", json!(1)),
        ("String([Number.NaN, 1].mode())", json!("NaN")),
        ("[1, Number.NaN].mode()", json!(1)),
        ("[Number.NaN, Number.NaN, 2, 2].mode()", json!(2)),
        (
            "[[Number.NaN], [Number.NaN], [1]].mode()[0] == 1",
            json!(false),
        ),
        ("[a, b].mode() == a", json!(true)),
        ("[b, a].mode() == b", json!(true)),
        ("[b, a, a].mode() == a", json!(true)),
        ("[b, a, b].mode() == b", json!(true)),
        ("[a, b, b].mode() == a", json!(true)),
        ("[a].includes(b)", json!(true)),
        ("[b].includes(a)", json!(false)),
        ("[a, plain, b].mode() == a", json!(true)),
        ("[plain, a, b].mode() == plain", json!(true)),
        ("[a, plain, b, plain].mode() == a", json!(true)),
        ("[a, other, other, both].mode() == a", json!(true)),
        (
            "[{m: 1}, {m: 1, q: holder.missing}, {m: 2}].mode()",
            json!({"m": 1}),
        ),
        (
            "[{m: 1, q: holder.missing}, {m: 1}, {m: 2}].mode().m",
            json!(1),
        ),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    runtime
        .run("class Robot(name): this.name = name\nada = Robot(\"ada\")\ntwin = Robot(\"ada\")\nshared = {k: 1}\nnumbers = [3, 3]")
        .unwrap();
    for (source, kept) in [
        ("[twin, ada, ada].mode()", "ada"),
        ("[shared, {k: 1}, shared].mode()", "shared"),
        ("[{k: 1}, shared, shared].mode() == shared", "false"),
        ("[twin, ada].mode(robot -> robot)", "twin"),
        ("[numbers, [3, 3]].mode()", "numbers"),
    ] {
        let expected = value(&mut runtime, kept);
        let actual = value(&mut runtime, source);
        assert!(
            identical(&actual, &expected),
            "{source} gave {actual:?}, expected {expected:?}"
        );
    }
}

#[test]
fn null_items_are_counted_while_reads_inside_a_selector_settle() {
    let mut runtime = run(r#"
values = [1, null, 1]
plain = values.mode()
selected = values.mode(value -> value)
class Item(price):
    this.price = price
pen = Item(3)
gift = Item(null)
price = Item.mode(item -> item.price)
assert(plain, 1)
assert(selected, null)
assert(price, null)
"#);
    checked(
        &mut runtime,
        "gift.price = 3\nassert(price, 3)\nvalues.push(null)\nvalues.push(null)\nassert(plain, null)",
    );
    assert_eq!(
        value(&mut runtime, "[1, null, 1].mode(item -> [item])"),
        Value::List(vec![Value::Number(1.0)])
    );
}

#[test]
fn undefined_items_and_results_are_counted_and_an_empty_receiver_is_undefined() {
    let mut runtime = run(r#"
values = [1]
values.push(values[9])
values.push(null)
common = values.mode()
class Item
pen = Item()
pen.price = 3
book = Item()
ink = Item()
price = Item.mode(item -> item.price)
empty = []
none = empty.mode()
assert(common, null)
assert(price, null)
assert(none, null)
"#);
    let (mut repl, _) = common::runners();
    repl("values = [1]\nvalues.push(values[9])\nvalues.push(values[9])\nholder = {}");
    assert_eq!(repl("typeof values.mode()"), "undefined");
    assert_eq!(
        repl("typeof [1, 2].mode(item -> holder.missing)"),
        "undefined"
    );
    assert_eq!(repl("typeof [].mode()"), "undefined");
    assert_eq!(repl("typeof [].mode(item -> holder.missing)"), "undefined");
    assert_eq!(repl("[].mode() ?? \"none\""), "none");
    checked(
        &mut runtime,
        "book.price = 3\nassert(price, 3)\nempty.push(\"x\")\nassert(none, \"x\")\nvalues.push(1)\nassert(common, 1)",
    );
}

#[test]
fn selectors_receive_only_the_item_once_per_item_in_order() {
    let (mut run, _) = common::runners();
    run(r#"
log = []
def note(item, position, list):
    log.push([item, position, list])
    return item % 2
values = [3, 1, 2, 5]
class Item(size):
    this.size = size
Item(5)
Item(7)
Item(5)
"#);
    for (query, result, visits) in [
        (
            "values.mode(note)",
            json!(1),
            json!([
                [3, null, null],
                [1, null, null],
                [2, null, null],
                [5, null, null]
            ]),
        ),
        (
            "Item.mode(item -> note(item.size))",
            json!(1),
            json!([[5, null, null], [7, null, null], [5, null, null]]),
        ),
        ("typeof [].mode(note)", json!("undefined"), json!([])),
        (
            "values.mode(lambda first, second: second)",
            json!(null),
            json!([]),
        ),
    ] {
        run("log = []");
        assert_eq!(run(query), result, "{query}");
        assert_eq!(run("log"), visits, "{query}");
    }
    run("log = []\nItem(2)\nItem(4)\nItem(6)");
    assert_eq!(run("Item.mode(item -> note(item.size))"), 1);
    assert_eq!(run("log.length"), 6);
    run("log = []\nItem(8)");
    assert_eq!(run("Item.mode(item -> note(item.size))"), 0);
    assert_eq!(run("log.length"), 7);
    assert_eq!(
        run("[\"ox\", \"cat\", \"dog\", \"ab\", \"bee\"].mode(word -> word.length)"),
        3
    );
    assert_eq!(
        run("[\"ox\", \"cat\", \"dog\", \"ab\", \"bee\"].map(word -> word.length).mode()"),
        3
    );
}

#[test]
fn a_supplied_selector_must_be_a_function() {
    let (mut run, mut fail) = common::runners();
    run("class Empty\nclass Item\nItem()\nlog = []\nnothing = null\nholder = {}");
    for receiver in ["[]", "[1, 2]", "Empty", "Item"] {
        for argument in [
            "5",
            "null",
            "\"region\"",
            "[1]",
            "{}",
            "true",
            "holder.missing",
        ] {
            let source = format!("{receiver}.mode({argument})");
            assert_eq!(
                fail(&source),
                "TypeError: mode expects a function",
                "{source}"
            );
        }
    }
    for source in [
        "[1].mode(5, missing)",
        "[1].mode(missing, 5)",
        "Item.mode(missing)",
    ] {
        assert_eq!(
            fail(source),
            "ReferenceError: missing is not defined",
            "{source}"
        );
    }
    assert_eq!(run("nothing.mode(log.push(1))"), json!(null));
    assert_eq!(run("typeof nothing.mode()"), "undefined");
    assert_eq!(run("typeof holder.missing.mode()"), "undefined");
    assert_eq!(run("log"), json!([]));
    assert_eq!(
        run("[1, 2, 2].mode(value -> value, log.push(\"a\"), log.push(\"b\"))"),
        2
    );
    assert_eq!(run("log"), json!(["a", "b"]));
    assert_eq!(run("[1, 2].mode(value -> value, 1)"), 1);
    assert_eq!(run("typeof [].mode(value -> value, 5)"), "undefined");
}

#[test]
fn the_receiver_selector_and_extras_are_evaluated_once_in_order() {
    let (mut run, _) = common::runners();
    run(r#"
calls = []
def source():
    calls.push("receiver")
    return [1, 2, 2]
def pick():
    calls.push("selector")
    return value -> value * 10
def extra():
    calls.push("extra")
    return 0
class Item(size):
    this.size = size
Item(1)
def grow(size):
    Item(size)
    return item -> item.size
"#);
    assert_eq!(
        run("source().mode(pick(), extra(), calls.push(\"last\"))"),
        20
    );
    assert_eq!(
        run("calls"),
        json!(["receiver", "selector", "extra", "last"])
    );
    assert_eq!(run("Item.mode(grow(9))"), 1);
    assert_eq!(run("Item.length"), 2);
    assert_eq!(run("Item.mode(grow(9))"), 1);
    assert_eq!(run("Item.length"), 3);
    assert_eq!(run("Item.mode(item -> item.size)"), 9);
}

#[test]
fn mode_depends_on_the_receiver_the_arguments_and_the_selector_reads() {
    let mut runtime = run(r#"
values = [1, 2, 2]
top = values.mode()
frozen = values.value.mode()
class Order(region):
    this.region = region
a = Order("north")
b = Order("south")
favorite = Order.mode(order -> order.region)
suffix = "!"
def label order => order.region + suffix
labels = Order.mode(label)
assert(top, 2)
assert(favorite, "north")
assert(labels, "north!")
"#);
    assert_eq!(variable_dependencies(&runtime, "top"), ["values"]);
    assert!(variable_dependencies(&runtime, "frozen").is_empty());
    assert_eq!(
        variable_dependencies(&runtime, "favorite"),
        ["$Order", "a.region", "b.region"]
    );
    assert_eq!(
        variable_dependencies(&runtime, "labels"),
        ["$Order", "label", "a.region", "suffix", "b.region"]
    );
    checked(
        &mut runtime,
        "values.push(1)\nvalues.push(1)\nassert(top, 1)\nassert(frozen, 2)\nc = Order(\"south\")\n\
         assert(favorite, \"south\")\nsuffix = \"?\"\nassert(labels, \"south?\")\n\
         b.region = \"north\"\nassert(favorite, \"north\")\nassert(labels, \"north?\")",
    );
    assert_eq!(
        variable_dependencies(&runtime, "favorite"),
        ["$Order", "a.region", "b.region", "c.region"]
    );
}

#[test]
fn comparisons_do_not_subscribe_to_the_compared_objects() {
    let mut runtime = run(r#"
first = {k: 1}
second = {k: 2}
records = [first, second, second]
common = records.mode()
assert(common == second, true)
first.k = 2
assert(common == second, true)
"#);
    assert_eq!(variable_dependencies(&runtime, "common"), ["records"]);
    assert_eq!(
        value(&mut runtime, "records.mode() == first"),
        Value::Bool(true)
    );
    checked(
        &mut runtime,
        "records.push(first)\nassert(common == first, true)",
    );
}

#[test]
fn populations_gaining_and_losing_instances_re_evaluate_modes() {
    run(r#"
class Visit(room):
    this.room = room
busiest = Visit.mode(visit -> visit.room)
assert(busiest, null)
first = Visit("lab")
second = Visit("den")
third = Visit("den")
assert(busiest, "den")
delete second.room
delete second
assert(busiest, "lab")
Visit("lab")
assert(busiest, "lab")
delete first.room
delete first
assert(busiest, "den")
delete third.room
delete third
assert(busiest, "lab")
assert(Visit.length, 1)
class Empty
none = Empty.mode(empty -> empty.size)
assert(none, null)
Empty()
assert(none, null)
"#);
}

#[test]
fn class_level_rules_find_modes_for_each_instance() {
    let mut runtime = run(r#"
class Player(team, role):
    this.team = team
    this.role = role
class Team(name, tags):
    this.name = name
    this.tags = tags
$Team.role = Player.filter(player -> player.team == $Team).mode(player -> player.role)
$Team.label = $Team.tags.mode()
red = Team("red", ["fast", "tall", "fast"])
blue = Team("blue", [])
Player(red, "guard")
ana = Player(blue, "wing")
Player(red, "guard")
Player(red, "wing")
assert(red.role, "guard")
assert(blue.role, "wing")
assert(red.label, "fast")
assert(blue.label, null)
ana.role = "center"
blue.tags.push("tall")
assert(blue.role, "center")
assert(blue.label, "tall")
Player(red, "wing")
Player(red, "wing")
assert(red.role, "wing")
"#);
    let roles = dependencies(
        &runtime,
        &NodeKey::property(&ObjectId::named("blue"), "role"),
    );
    for key in ["$Player", "ana.role"] {
        assert!(roles.contains(&key), "{key} in {roles:?}");
    }
    let labels = dependencies(
        &runtime,
        &NodeKey::property(&ObjectId::named("blue"), "label"),
    );
    assert!(labels.contains(&"blue.tags"), "blue.tags in {labels:?}");
    checked(
        &mut runtime,
        "green = Team(\"green\", [\"new\", \"old\", \"old\"])\nassert(green.role, null)\nassert(green.label, \"old\")",
    );
}

#[test]
fn property_reads_bindings_and_other_receivers_keep_their_accepted_results() {
    let (mut run, mut fail) = common::runners();
    run(r#"
values = [1, 2]
class Order(region):
    this.region = region
Order("north")
report = {mode: 5}
holder = {mode: value -> value * 2}
empty = {}
mode = 3
class Review
$Review.rate = $Review.mode / 10
review = Review()
review.mode = 42
def twice mode => mode * 2
nothing = null
count = 5
flag = true
when = Date(5)
pattern = /a/
def helper(value) => value
class Box:
    def mode(): return 7
box = Box()
"#);
    for (source, expected) in [
        ("values.mode", json!(null)),
        ("typeof values.mode", json!("undefined")),
        ("Order.mode", json!(null)),
        ("typeof Order.mode", json!("undefined")),
        ("report.mode", json!(5)),
        ("holder.mode(4)", json!(8)),
        ("empty.mode()", json!(null)),
        ("typeof empty.mode()", json!("undefined")),
        ("mode", json!(3)),
        ("review.mode", json!(42)),
        ("review.rate", json!(4.2)),
        ("twice(4)", json!(8)),
        ("box.mode()", json!(7)),
        ("nothing.mode()", json!(null)),
        ("values.mode() + mode", json!(4)),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    run("x = {mode: 4}.mode");
    assert_eq!(run("x"), 4);
    for (source, message) in [
        ("\"abc\".mode()", "TypeError: abc.mode is not a function"),
        ("count.mode()", "TypeError: number.mode is not a function"),
        ("flag.mode()", "TypeError: boolean.mode is not a function"),
        ("when.mode()", "TypeError: Date.mode is not a function"),
        ("pattern.mode()", "TypeError: mode is not a function"),
        (
            "helper.mode()",
            "TypeError: function.mode is not a function",
        ),
        ("Math.mode([1])", "TypeError: Math.mode is not a function"),
        ("Class.mode()", "ReferenceError: Class is not defined"),
        ("List.mode()", "ReferenceError: List is not defined"),
        (
            "values.mode(missing)",
            "ReferenceError: missing is not defined",
        ),
    ] {
        assert_eq!(fail(source), message, "{source}");
    }
    let (mut defined, mut missing) = common::runners();
    assert_eq!(
        missing("mode([1, 2])"),
        "ReferenceError: mode is not defined"
    );
    defined("def mode(items) => items.length");
    assert_eq!(defined("mode([1, 1])"), 2);
    assert_eq!(defined("[1, 1].mode()"), 1);
}

#[test]
fn motivating_forms_keep_their_accepted_results() {
    let (mut run, mut fail) = common::runners();
    run(r#"
xs = [1, "1", "1", "1", 1]
sorted = xs.slice().sort()
counts = xs.map(x => sorted.lastIndexOf(x) - sorted.indexOf(x) + 1)
picked = xs[counts.indexOf(counts.max())]
"#);
    assert_eq!(run("[picked, counts]"), json!([1, [5, 3, 3, 3, 5]]));
    assert_eq!(run("xs.mode()"), "1");
    run("xs = [3, 4, 4, 3, 5, 4]");
    assert_eq!(run("[picked, xs.mode()]"), json!([4, 4]));
    assert_eq!(
        fail("xs.map(x => xs.count(y => y == x))"),
        "ReferenceError: x is not defined"
    );
    assert_eq!(fail("xs.count(4)"), "TypeError: count expects a function");
    assert_eq!(run("xs.max(x -> x * 2)"), 10);
    assert_eq!(run("xs.count(x -> x == 4)"), 3);
}

#[test]
fn compatibility_rows_give_the_documented_results() {
    let orders =
        "class Order(region):\n    this.region = region\nOrder(\"n\")\nOrder(\"s\")\nOrder(\"n\")";
    let shelf = "class Shelf:\n    def mode(): return 9\nshelf = Shelf()";
    let kind = format!("{orders}\nkind = Order");
    for (setup, source, expected) in [
        ("", "[1, 2, 2].mode()", json!(2)),
        ("", "typeof [].mode()", json!("undefined")),
        ("xs = [3, 4, 4]", "xs.mode()", json!(4)),
        ("xs = [1, 2, 3]", "xs.mode(x -> x % 2)", json!(1)),
        ("o = {items: [1, 1]}", "o.items.mode()", json!(1)),
        ("xs = [3, 4]", "xs.map(x -> x % 2).mode()", json!(1)),
        ("xs = [1, 1, 2]", "xs.value.mode()", json!(1)),
        ("xs = [1]\nxs.push(1)", "xs@first.mode()", json!(1)),
        (orders, "Order.mode(o -> o.region)", json!("n")),
        (orders, "Order[1:].mode(o -> o.region)", json!("s")),
        (orders, "Order.list().mode(o -> o.region)", json!("n")),
        (kind.as_str(), "kind.mode(o -> o.region)", json!("n")),
        ("class Empty", "typeof Empty.mode()", json!("undefined")),
        (shelf, "Shelf.mode() == shelf", json!(true)),
        (shelf, "Shelf.mode(s -> s.mode())", json!(9)),
        (
            "",
            "try:\n    v = [1, 1].mode()\ncatch e:\n    v = e\nv",
            json!(1),
        ),
    ] {
        let (mut run, _) = common::runners();
        if !setup.is_empty() {
            run(setup);
        }
        assert_eq!(run(source), expected, "{setup}\n{source}");
    }
    for (setup, source, message) in [
        (
            "xs = [1, 2]",
            "xs.mode(5)",
            "TypeError: mode expects a function",
        ),
        ("", "[].mode(null)", "TypeError: mode expects a function"),
        (
            orders,
            "Order.mode(\"region\")",
            "TypeError: mode expects a function",
        ),
    ] {
        let (mut run, mut fail) = common::runners();
        if !setup.is_empty() {
            run(setup);
        }
        assert_eq!(fail(source), message, "{setup}\n{source}");
    }
}

#[test]
fn class_names_run_the_population_query_beside_instance_methods() {
    let (mut run, _) = common::runners();
    run(
        "class Shelf(items):\n    this.items = items\n    def mode(): return this.items.mode()\nfirst = Shelf([1, 2, 2])\nsecond = Shelf([3, 3])\nthird = Shelf([2])",
    );
    assert_eq!(run("first.mode()"), 2);
    assert_eq!(run("second.mode()"), 3);
    assert_eq!(run("Shelf.mode(shelf -> shelf.mode())"), 2);
    assert_eq!(run("Shelf.mode() == first"), true);
    assert_eq!(run("Shelf.length"), 3);
}

#[test]
fn rendering_and_reasoning_keep_the_ordinary_call() {
    let (mut run, _) = common::runners();
    run(
        "values = [1, 2, 2]\ntop = values.mode()\nclass Order(region):\n    this.region = region\nOrder(\"north\")\nfavorite = Order.mode(order -> order.region)",
    );
    assert_eq!(run("(why top)[0].rule"), "top = values.mode()");
    assert_eq!(run("(why top)[0].holds"), 2);
    assert_eq!(
        run("(why favorite)[0].rule"),
        "favorite = Order.mode(function())"
    );
    assert_eq!(run("(why favorite)[0].state"), "derived");
    assert_eq!(run("(why favorite)[0].holds"), "north");
}

#[test]
fn failed_mode_updates_restore_state_order_and_graph() {
    let initial = r#"
def check(value):
    if value == "void":
        throw "BAD_ZONE"
    return value
zones = ["a", "b", "b"]
busiest = zones.mode(check)
class Parcel(zone):
    this.zone = zone
Parcel("b")
Parcel("a")
Parcel("a")
routes = Parcel.mode(parcel -> check(parcel.zone))
assert(busiest, "b")
assert(routes, "a")
"#;
    let mut actual = run(initial);
    let control = run(initial);
    for (source, thrown) in [
        ("zones.push(\"void\")", "BAD_ZONE"),
        ("zones.shift()\nzones.push(\"void\")", "BAD_ZONE"),
        ("zones.sort()\nzones.push(\"void\")", "BAD_ZONE"),
        ("bad = Parcel(\"void\")", "BAD_ZONE"),
        ("first = Parcel(\"c\")\nthrow \"ROLLBACK\"", "ROLLBACK"),
    ] {
        let error = actual.run(source).unwrap_err();
        assert_eq!(
            error.thrown_value(),
            Some(&Value::String(thrown.to_string())),
            "{source}"
        );
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    }
    checked(
        &mut actual,
        "assert(zones, [\"a\", \"b\", \"b\"])\nzones.push(\"a\")\nzones.push(\"a\")\nassert(busiest, \"a\")\nParcel(\"b\")\nParcel(\"b\")\nassert(routes, \"b\")",
    );
}

#[test]
fn suppose_blocks_discard_mode_changes() {
    let mut runtime = run(r#"
zones = ["b", "a", "b", "a"]
busiest = zones.mode()
class Visit(room):
    this.room = room
lab = Visit("lab")
den = Visit("den")
Visit("den")
room = Visit.mode(visit -> visit.room)
suppose zones.shift(): assert(busiest, "a")
suppose Visit("lab"): assert(room, "lab")
suppose:
    delete den.room
    delete den
    assert(room, "lab")
moved = suppose lab.room = "den": room
assert(moved, "den")
assert(zones, ["b", "a", "b", "a"])
assert(busiest, "b")
assert(room, "den")
"#);
    checked(
        &mut runtime,
        "zones.shift()\nassert(busiest, \"a\")\nVisit(\"lab\")\nVisit(\"lab\")\nassert(room, \"lab\")",
    );
}

#[test]
fn large_receivers_find_the_mode_and_select_each_item_once() {
    let items = 20_000_u32;
    let list = (0..items)
        .map(|item| ((item * 7_919) % items).to_string())
        .collect::<Vec<_>>()
        .join(",\n");
    let mut runtime = Runtime::new();
    runtime
        .run(&format!(
            "values = [{list}]\ncounter = {{n: 0}}\ndef note(value):\n    counter.n = counter.n + 1\n    return value\ndef row(value):\n    return {{k: value % 50, v: null, tags: [value % 3]}}\nrows = values.map(row)\nholder = {{}}\ngaps = values.map(value -> ({{k: value % 50, gap: holder.missing}}))"
        ))
        .unwrap();
    for (source, expected) in [
        ("values.mode() == values[0]", Value::Bool(true)),
        ("values.concat([19999]).mode()", Value::Number(19_999.0)),
        (
            "values.map(value -> String(value % 7)).mode()",
            Value::String("0".to_string()),
        ),
        (
            "values.map(value -> [value % 5, value % 2]).mode()[0]",
            Value::Number(0.0),
        ),
        ("rows.mode() == rows[0]", Value::Bool(true)),
        ("rows.mode().k", Value::Number(0.0)),
        ("gaps.mode().k", Value::Number(0.0)),
        ("rows.concat(gaps).mode() == gaps[0]", Value::Bool(true)),
        ("values.mode(note) == values[0]", Value::Bool(true)),
        ("counter.n", Value::Number(f64::from(items))),
        ("values.mode(value -> value % 10)", Value::Number(0.0)),
        (
            "values.mode(value -> value % 10 == 3 ? 3 : value)",
            Value::Number(3.0),
        ),
    ] {
        let actual = value(&mut runtime, source);
        assert!(
            identical(&actual, &expected),
            "{source} gave {actual:?}, expected {expected:?}"
        );
    }
    runtime
        .run("common = values.mode(value -> value % 4)")
        .unwrap();
    assert_eq!(runtime.run("common").unwrap(), Value::Number(0.0));
    assert_eq!(
        runtime.run("(why common)[0].rule").unwrap(),
        Value::String("common = values.mode(function())".to_string())
    );
}
