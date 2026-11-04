mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, ObjectId, Runtime, Value};
use serde_json::json;

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid averages lists with average",
    "Nucleoid averages class instance populations with average",
    "Nucleoid converts averaged values to numbers",
    "Nucleoid calls the average selector once per item with only the item",
    "Nucleoid keeps averages live in class level rules",
    "Nucleoid uses averages in functions methods and conditional expressions",
    "Nucleoid keeps average as an ordinary name beside the average query",
    "Nucleoid rolls back failed average updates",
];
const REFERENCE_TITLE: &str = "Nucleoid averages lists and populations with average";
const README_TITLE: &str = "Averages keep mean temperatures live as readings change";
const EXAMPLES_TITLE: &str = "Averages keep the rating of every product live as reviews arrive";

type Failure = (ErrorKind, String, usize, usize);

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "list average regression".to_string(),
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

fn number(runtime: &mut Runtime, source: &str) -> f64 {
    match runtime.run(source) {
        Ok(Value::Number(number)) => number,
        other => panic!("{source} gave {other:?}"),
    }
}

fn same_number(left: f64, right: f64) -> bool {
    left.to_bits() == right.to_bits() || (left.is_nan() && right.is_nan())
}

fn same_value(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => same_number(*left, *right),
        _ => left == right,
    }
}

fn failure(source: &str) -> Failure {
    let error = Runtime::new().run(source).unwrap_err();
    let position = error.position().expect("errors are positioned");
    (
        error.kind(),
        error.message(),
        position.line,
        position.column,
    )
}

fn expected(kind: ErrorKind, message: &str, line: usize, column: usize) -> Failure {
    (kind, message.to_string(), line, column)
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
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(case.source.contains(".average("), "{title} uses average");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 48);
}

#[test]
fn averages_accept_every_receiver_the_list_queries_accept() {
    let (mut run, _) = common::runners();
    run(r#"
prices = [4, 6, 10]
cart = {items: [1, 2]}
history = [1]
history.push(2)
class Order(total):
    this.total = total
Order(5)
Order(7)
Order(12)
kind = Order
class Empty
"#);
    for (source, expected) in [
        ("[1, 2, 3].average()", 2.0),
        ("prices.average()", 20.0 / 3.0),
        ("cart.items.average()", 1.5),
        ("prices.filter(price -> price > 5).average()", 8.0),
        ("prices.map(price -> price * 2).average()", 40.0 / 3.0),
        ("prices.slice(1).average()", 8.0),
        ("prices.concat([4]).average()", 6.0),
        ("prices[1:].average()", 8.0),
        ("prices.value.average()", 20.0 / 3.0),
        ("history@first.average()", 1.0),
        ("history.average()", 1.5),
        ("Order.average(order -> order.total)", 8.0),
        ("Order.average(.total)", 8.0),
        ("Order[1:].average(order -> order.total)", 9.5),
        ("Order[-1:].average(.total)", 12.0),
        ("Order.list().average(order -> order.total)", 8.0),
        (
            "Order.filter(order -> order.total > 6).average(order -> order.total)",
            9.5,
        ),
        ("kind.average(order -> order.total)", 8.0),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    for source in [
        "String([].average())",
        "String(Empty.average())",
        "String(Empty.average(empty -> empty.size))",
        "String(prices.filter(price -> price > 50).average())",
    ] {
        assert_eq!(run(source), "NaN", "{source}");
    }
}

#[test]
fn numeric_averages_equal_the_divided_sum_bit_for_bit() {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut lists: Vec<String> = vec![
        String::new(),
        "0.1, 0.2, 0.3".to_string(),
        "-0".to_string(),
        "-0, -0".to_string(),
        "1, -1".to_string(),
        "9007199254740992, 1, 1".to_string(),
        "0.1, -0.1, 0.3".to_string(),
        "1e308, 1e308".to_string(),
    ];
    for _ in 0..300 {
        let length = (next() % 12) as usize;
        let items: Vec<String> = (0..length)
            .map(|_| {
                let raw = (next() % 2_000_001) as f64 - 1_000_000.0;
                let scale = [1.0, 10.0, 1000.0, 1_000_000.0][(next() % 4) as usize];
                format!("{}", raw / scale)
            })
            .collect();
        lists.push(items.join(", "));
    }
    let mut runtime = Runtime::new();
    for items in &lists {
        let mean = number(&mut runtime, &format!("[{items}].average()"));
        let divided = number(&mut runtime, &format!("[{items}].sum() / [{items}].length"));
        let noun = number(
            &mut runtime,
            &format!("sum of [{items}] / [{items}].length"),
        );
        assert!(
            same_number(mean, divided),
            "[{items}]: {mean:?} and {divided:?}"
        );
        assert!(same_number(mean, noun), "[{items}] with sum of");
        let selected = number(
            &mut runtime,
            &format!("[{items}].average(item -> item * 3)"),
        );
        let summed = number(
            &mut runtime,
            &format!("[{items}].sum(item -> item * 3) / [{items}].length"),
        );
        assert!(same_number(selected, summed), "[{items}] with a selector");
    }
    assert!(same_number(
        number(&mut runtime, "[0.1, 0.2, 0.3].average()"),
        0.6000000000000001 / 3.0
    ));
    assert_eq!(
        number(&mut runtime, "[0.1, 0.2, 0.3].average()"),
        0.20000000000000004
    );
    assert_eq!(number(&mut runtime, "1 / [-0].average()"), f64::INFINITY);
    assert_eq!(
        number(&mut runtime, "[1e308, 1e308].average()"),
        f64::INFINITY
    );
    assert!(number(&mut runtime, "[].average()").is_nan());
}

#[test]
fn population_averages_equal_the_divided_population_sum() {
    let mut runtime = run(r#"
class Sample(amount):
    this.amount = amount
Sample(0.1)
Sample(0.2)
Sample(0.3)
Sample(-7.25)
Sample(1e15)
"#);
    for selector in [
        "sample -> sample.amount",
        "sample -> sample.amount * sample.amount",
        ".amount",
    ] {
        let mean = number(&mut runtime, &format!("Sample.average({selector})"));
        let divided = number(
            &mut runtime,
            &format!("Sample.sum({selector}) / Sample.length"),
        );
        assert!(
            same_number(mean, divided),
            "{selector}: {mean:?} and {divided:?}"
        );
    }
}

#[test]
fn averaged_values_use_the_numeric_conversion_of_sum() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Order\nholder = {}\nwhen = Date(5)")
        .unwrap();
    for (value, expected) in [
        ("3", 3.0),
        ("-2.5", -2.5),
        ("true", 1.0),
        ("false", 0.0),
        ("null", 0.0),
        ("\"\"", 0.0),
        ("\"  \"", 0.0),
        ("\"3\"", 3.0),
        ("\" 4 \"", 4.0),
        ("\"2.5\"", 2.5),
        ("\"x\"", f64::NAN),
        ("\"3px\"", f64::NAN),
        ("[]", 0.0),
        ("[5]", 5.0),
        ("[[6]]", 6.0),
        ("[\"7\"]", 7.0),
        ("[1, 2]", f64::NAN),
        ("{}", f64::NAN),
        ("holder", f64::NAN),
        ("when", 5.0),
        ("/a/", f64::NAN),
        ("Order", f64::NAN),
        ("(value -> value)", f64::NAN),
        ("Number.NaN", f64::NAN),
        ("Number.POSITIVE_INFINITY", f64::INFINITY),
        ("Number.NEGATIVE_INFINITY", f64::NEG_INFINITY),
    ] {
        for source in [
            format!("[{value}].average()"),
            format!("[{value}].average(item -> item)"),
        ] {
            let mean = number(&mut runtime, &source);
            assert!(same_number(mean, expected), "{source}: {mean:?}");
        }
        let pair = number(&mut runtime, &format!("[{value}, 0].average()"));
        assert!(
            same_number(pair, (expected + 0.0) / 2.0),
            "[{value}, 0]: {pair:?}"
        );
        let summed = number(&mut runtime, &format!("[{value}].sum() / 1"));
        assert!(
            same_number(
                number(&mut runtime, &format!("[{value}].average()")),
                summed
            ),
            "{value} converts as sum does"
        );
    }
    for (source, expected) in [
        ("[Number.POSITIVE_INFINITY, 1].average()", f64::INFINITY),
        (
            "[Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY].average()",
            f64::NAN,
        ),
        ("[Number.NaN, 1].average()", f64::NAN),
        ("[\"2\", \"3\"].average()", 2.5),
        ("[true, true, false, false].average()", 0.5),
    ] {
        let mean = number(&mut runtime, source);
        assert!(same_number(mean, expected), "{source}: {mean:?}");
    }
}

#[test]
fn every_item_counts_toward_the_divisor() {
    let (mut run, _) = common::runners();
    run("class Item(price):\n    this.price = price\nItem(4)\nItem(null)");
    for (source, expected) in [
        ("[2, null].average()", 1.0),
        ("[2].average()", 2.0),
        ("[\"\", 4].average()", 2.0),
        ("[false, 1].average()", 0.5),
        ("[[], 3].average()", 1.5),
        ("[true, false].average()", 0.5),
        ("Item.average(item -> item.price)", 2.0),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    assert_eq!(run("String([\"x\", 1].average())"), "NaN");
}

#[test]
fn an_undefined_value_makes_the_average_undefined() {
    let mut runtime = run(r#"
values = [1]
values.push(values[9])
class Item
pen = Item()
pen.price = 3
book = Item()
mean = values.average()
spent = Item.average(item -> item.price)
assert(mean, null)
assert(spent, null)
"#);
    let (mut repl, _) = common::runners();
    repl("values = [1]\nvalues.push(values[9])\nholder = {}");
    assert_eq!(repl("typeof values.average()"), "undefined");
    assert_eq!(
        repl("typeof [Number.NaN].concat(values).average()"),
        "undefined"
    );
    assert_eq!(
        repl("typeof [1, 2].average(item -> holder.missing)"),
        "undefined"
    );
    assert_eq!(repl("typeof [].average(item -> holder.missing)"), "number");
    checked(
        &mut runtime,
        "book.price = 12\nassert(spent, 7.5)\nvalues.pop()\nassert(mean, 1)",
    );
}

#[test]
fn null_items_count_zero_while_reads_inside_a_selector_settle() {
    let mut runtime = run(r#"
values = [1, null, 2]
plain = values.average()
selected = values.average(value -> value)
class Item(price):
    this.price = price
pen = Item(3)
gift = Item(null)
spent = Item.average(item -> item.price)
divided = Item.sum(item -> item.price) / Item.length
assert(plain, 1)
assert(selected, null)
assert(spent, null)
assert(divided, null)
"#);
    for source in ["values.average()", "values.average(value -> value)"] {
        assert_eq!(number(&mut runtime, source), 1.0, "{source}");
    }
    checked(
        &mut runtime,
        "gift.price = 5\nassert(spent, 4)\nassert(divided, 4)\nvalues.push(null)\nassert(plain, 0.75)",
    );
}

#[test]
fn empty_receivers_average_to_nan_rather_than_null() {
    let mut runtime = run(r#"
class Empty
empty = []
mean = empty.average()
none = Empty.average(item -> item.size)
filtered = [1, 2].filter(value -> value > 5).average()
assert(Number.isNaN(mean), true)
assert(mean == null, false)
assert(Number.isNaN(none), true)
assert(Number.isNaN(filtered), true)
assert(typeof mean, "number")
"#);
    let (mut repl, _) = common::runners();
    repl("log = []\ndef note(value):\n    log.push(value)\n    return value");
    assert_eq!(repl("String([].average(note))"), "NaN");
    assert_eq!(repl("log"), json!([]));
    assert_eq!(repl("typeof [].average()"), "number");
    checked(
        &mut runtime,
        "empty.push(4)\nassert(mean, 4)\nEmpty()\nassert(none, null)",
    );
}

#[test]
fn selectors_receive_only_the_item_once_per_item_in_order() {
    let (mut run, _) = common::runners();
    run(r#"
log = []
def note(item, position, list):
    log.push([item, position, list])
    return item
values = [3, 1, 2]
class Item(size):
    this.size = size
Item(5)
Item(7)
"#);
    for (query, mean, visits) in [
        (
            "values.average(note)",
            json!(2),
            json!([[3, null, null], [1, null, null], [2, null, null]]),
        ),
        (
            "Item.average(item -> note(item.size))",
            json!(6),
            json!([[5, null, null], [7, null, null]]),
        ),
        (
            "String([\"x\", 1, 2].average(note))",
            json!("NaN"),
            json!([["x", null, null], [1, null, null], [2, null, null]]),
        ),
        ("String([].average(note))", json!("NaN"), json!([])),
    ] {
        run("log = []");
        assert_eq!(run(query), mean, "{query}");
        assert_eq!(run("log"), visits, "{query}");
    }
    run("log = []\nvalues.push(values[9])");
    assert_eq!(run("typeof values.average(note)"), "undefined");
    assert_eq!(run("log.length"), 4);
    run("log = []\nItem(1)");
    assert_eq!(run("Item.average(item -> note(item.size))"), 13.0 / 3.0);
    assert_eq!(
        run("log"),
        json!([[5, null, null], [7, null, null], [1, null, null]])
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
            "\"total\"",
            "[1]",
            "{}",
            "true",
            "holder.missing",
        ] {
            let source = format!("{receiver}.average({argument})");
            assert_eq!(
                fail(&source),
                "TypeError: average expects a function",
                "{source}"
            );
        }
    }
    assert_eq!(
        fail("[1].average(5, missing)"),
        "ReferenceError: missing is not defined"
    );
    assert_eq!(
        fail("[1].average(missing, 5)"),
        "ReferenceError: missing is not defined"
    );
    assert_eq!(
        fail("Item.average(missing)"),
        "ReferenceError: missing is not defined"
    );
    assert_eq!(run("nothing.average(log.push(1))"), json!(null));
    assert_eq!(run("typeof nothing.average()"), "undefined");
    assert_eq!(run("log"), json!([]));
    assert_eq!(
        run("[1, 2].average(value -> value, log.push(\"a\"), log.push(\"b\"))"),
        1.5
    );
    assert_eq!(run("log"), json!(["a", "b"]));
    assert_eq!(run("[1, 2].average(value -> value, 99)"), 1.5);
    assert_eq!(run("[1, 2].average(value -> value, 99, 7)"), 1.5);
}

#[test]
fn the_receiver_selector_and_extras_are_evaluated_once_in_order() {
    let (mut run, _) = common::runners();
    run(r#"
calls = []
def source():
    calls.push("receiver")
    return [1, 2, 3]
def pick():
    calls.push("selector")
    return value -> value * 10
def extra():
    calls.push("extra")
    return 0
class Item
Item()
def grow():
    Item()
    return item -> 1
"#);
    assert_eq!(run("source().average(pick(), extra())"), 20);
    assert_eq!(run("calls"), json!(["receiver", "selector", "extra"]));
    assert_eq!(run("Item.average(grow())"), 1);
    assert_eq!(run("Item.length"), 2);
}

#[test]
fn averages_depend_on_what_the_divided_sum_depends_on() {
    let mut runtime = run(r#"
values = [1, 2]
mean = values.average()
divided = values.sum() / values.length
frozen = values.value.average()
class Order(amount):
    this.amount = amount
a = Order(1)
b = Order(2)
typical = Order.average(order -> order.amount)
total = Order.sum(order -> order.amount) / Order.length
rate = 2
def scaled order => order.amount * rate
weighted = Order.average(scaled)
assert(mean, 1.5)
assert(typical, 1.5)
assert(weighted, 3)
"#);
    assert_eq!(variable_dependencies(&runtime, "mean"), ["values"]);
    assert_eq!(
        variable_dependencies(&runtime, "mean"),
        variable_dependencies(&runtime, "divided")
    );
    assert_eq!(
        variable_dependencies(&runtime, "frozen"),
        Vec::<&str>::new()
    );
    assert_eq!(
        variable_dependencies(&runtime, "typical"),
        ["$Order", "a.amount", "b.amount"]
    );
    assert_eq!(
        variable_dependencies(&runtime, "typical"),
        variable_dependencies(&runtime, "total")
    );
    assert_eq!(
        variable_dependencies(&runtime, "weighted"),
        ["$Order", "scaled", "a.amount", "rate", "b.amount"]
    );
    checked(
        &mut runtime,
        "values.push(3)\nassert(mean, 2)\nassert(frozen, 1.5)\nc = Order(3)\nassert(typical, 2)\n\
         rate = 3\nassert(weighted, 6)\nb.amount = 5\nassert(typical, 3)\nassert(weighted, 9)",
    );
    assert_eq!(
        variable_dependencies(&runtime, "typical"),
        ["$Order", "a.amount", "b.amount", "c.amount"]
    );
}

#[test]
fn live_averages_stay_equal_to_the_divided_sum_as_models_change() {
    let mut runtime = run(r#"
class Item(price, qty):
    this.price = price
    this.qty = qty
values = [3, null, "4"]
pen = Item(2, 5)
cap = Item(null, 1)
"#);
    let pairs = [
        ("values.average()", "values.sum() / values.length"),
        (
            "values.average(value -> value)",
            "values.sum(value -> value) / values.length",
        ),
        (
            "Item.average(item -> item.price * item.qty)",
            "Item.sum(item -> item.price * item.qty) / Item.length",
        ),
        (
            "Item.filter(item -> item.qty > 1).average(item -> item.price)",
            "Item.filter(item -> item.qty > 1).sum(item -> item.price) / Item.filter(item -> item.qty > 1).length",
        ),
        (
            "Item[-2:].average(item -> item.qty)",
            "Item[-2:].sum(item -> item.qty) / Item[-2:].length",
        ),
    ];
    for (index, (mean, divided)) in pairs.iter().enumerate() {
        runtime
            .run(&format!("mean{index} = {mean}\ndivided{index} = {divided}"))
            .unwrap();
    }
    let compare = |runtime: &mut Runtime, step: &str| {
        for (index, (average, quotient)) in pairs.iter().enumerate() {
            let mean = runtime.run(&format!("mean{index}")).unwrap();
            let divided = runtime.run(&format!("divided{index}")).unwrap();
            assert!(
                same_value(&mean, &divided),
                "{step}: {average} gave {mean:?}, {quotient} gave {divided:?}"
            );
            assert_eq!(
                variable_dependencies(runtime, &format!("mean{index}")),
                variable_dependencies(runtime, &format!("divided{index}")),
                "{step}: {average}"
            );
        }
    };
    compare(&mut runtime, "declared");
    for step in [
        "values.push(5)",
        "cap.price = 4",
        "Item(7, 2)",
        "values = []",
        "values.push(\"x\")",
        "delete pen.qty",
        "delete pen.price",
        "delete pen",
        "values = [0.1, 0.2, 0.3]",
    ] {
        runtime.run(step).unwrap();
        compare(&mut runtime, step);
    }
}

#[test]
fn populations_gaining_and_losing_instances_re_evaluate_averages() {
    run(r#"
class Visit(minutes):
    this.minutes = minutes
first = Visit(10)
second = Visit(20)
typical = Visit.average(visit -> visit.minutes)
assert(typical, 15)
third = Visit(6)
assert(typical, 12)
delete second.minutes
assert(typical, null)
delete second
assert(typical, 8)
assert(Visit.length, 2)
class Empty
none = Empty.average(empty -> empty.size)
assert(Number.isNaN(none), true)
Empty()
assert(none, null)
"#);
}

#[test]
fn class_level_rules_average_for_each_instance() {
    let mut runtime = run(r#"
class Player(team, score):
    this.team = team
    this.score = score
class Team(name, bonuses):
    this.name = name
    this.bonuses = bonuses
$Team.form = Player.filter(player -> player.team == $Team).average(player -> player.score) + $Team.bonuses.average()
red = Team("red", [1])
blue = Team("blue", [2, 4])
Player(red, 3)
ana = Player(blue, 4)
assert(red.form, 4)
assert(blue.form, 7)
ana.score = 6
red.bonuses.push(3)
assert(blue.form, 9)
assert(red.form, 5)
Player(red, 1)
assert(red.form, 4)
"#);
    let form = dependencies(
        &runtime,
        &NodeKey::property(&ObjectId::named("blue"), "form"),
    );
    for key in ["$Player", "ana.score", "blue.bonuses"] {
        assert!(form.contains(&key), "{key} in {form:?}");
    }
    checked(
        &mut runtime,
        "green = Team(\"green\", [5])\nassert(Number.isNaN(green.form), true)\nPlayer(green, 7)\nassert(green.form, 12)",
    );
}

#[test]
fn property_reads_bindings_and_other_receivers_keep_their_accepted_results() {
    let (mut run, mut fail) = common::runners();
    run(r#"
values = [1, 2]
class Order(total):
    this.total = total
Order(5)
report = {average: 3}
holder = {average: value -> value * 2}
empty = {}
average = 7
class Review
$Review.rate = $Review.average / 10
review = Review()
review.average = 42
def twice average => average * 2
nothing = null
count = 5
flag = true
when = Date(5)
pattern = /a/
def helper(value) => value
x = {average: 5}.average
class Stats:
    def average(): return 42
stats = Stats()
"#);
    for (source, expected) in [
        ("values.average", json!(null)),
        ("typeof values.average", json!("undefined")),
        ("Order.average", json!(null)),
        ("typeof Order.average", json!("undefined")),
        ("Math.average", json!(null)),
        ("typeof Math.average", json!("undefined")),
        ("report.average", json!(3)),
        ("holder.average(4)", json!(8)),
        ("empty.average()", json!(null)),
        ("typeof empty.average()", json!("undefined")),
        ("average", json!(7)),
        ("review.average", json!(42)),
        ("review.rate", json!(4.2)),
        ("twice(4)", json!(8)),
        ("x", json!(5)),
        ("nothing.average()", json!(null)),
        ("stats.average()", json!(42)),
        ("String(Stats.average())", json!("NaN")),
        ("Stats.average(each -> each.average())", json!(42)),
        ("values.average() + average", json!(8.5)),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    for (source, message) in [
        (
            "\"abc\".average()",
            "TypeError: abc.average is not a function",
        ),
        (
            "count.average()",
            "TypeError: number.average is not a function",
        ),
        (
            "flag.average()",
            "TypeError: boolean.average is not a function",
        ),
        (
            "when.average()",
            "TypeError: Date.average is not a function",
        ),
        ("pattern.average()", "TypeError: average is not a function"),
        (
            "helper.average()",
            "TypeError: function.average is not a function",
        ),
        (
            "Math.average(1, 2)",
            "TypeError: Math.average is not a function",
        ),
        (
            "Math.average([1])",
            "TypeError: Math.average is not a function",
        ),
        ("Class.average()", "ReferenceError: Class is not defined"),
        ("List.average()", "ReferenceError: List is not defined"),
        (
            "values.average(missing)",
            "ReferenceError: missing is not defined",
        ),
    ] {
        assert_eq!(fail(source), message, "{source}");
    }
    let (mut defined, mut missing) = common::runners();
    assert_eq!(
        missing("average([1, 2])"),
        "ReferenceError: average is not defined"
    );
    defined("def average(xs) => xs.sum() / xs.length");
    assert_eq!(defined("average([2, 4])"), 3);
    assert_eq!(defined("[2, 4].average()"), 3);
    let (mut assigned, _) = common::runners();
    assigned("class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)\nOrder.average = 7");
    assert_eq!(assigned("Order.average"), json!(null));
    assert_eq!(assigned("Order.average(order -> order.total)"), 6);
    assigned("Order.sum = 9");
    assert_eq!(assigned("Order.sum"), json!(null));
}

#[test]
fn average_of_keeps_its_accepted_syntax_errors() {
    for (source, expected_failure) in [
        (
            "xs = [1]\naverage of xs",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 2, 12),
        ),
        (
            "xs = [1]\nx = average of xs",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 2, 16),
        ),
        (
            "average of [1, 2]",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 1, 12),
        ),
        (
            "xs = [1]\n[average of xs]",
            expected(
                ErrorKind::Syntax,
                "Expected ']' but found keyword 'of'",
                2,
                10,
            ),
        ),
        (
            "xs = [1]\nAverage of xs",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 2, 12),
        ),
        (
            "xs = [1]\naverages of xs",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 2, 13),
        ),
        (
            "xs = [1]\nxs.average of xs",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 2, 15),
        ),
        (
            "xs = [1]\n`${average of xs}`",
            expected(ErrorKind::Reference, "average is not defined", 2, 1),
        ),
    ] {
        assert_eq!(failure(source), expected_failure, "{source}");
    }
    let (mut run, _) = common::runners();
    run("average = 3\nxs = [1]");
    assert_eq!(run("`${average of xs}`"), "3");
}

#[test]
fn compatibility_rows_give_the_documented_results() {
    let orders = "class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)";
    let bag = "class Bag:\n    def average(): return 9\nbag = Bag()";
    let kind = format!("{orders}\nkind = Order");
    for (setup, source, expected) in [
        ("", "[1, 2, 3].average()", json!(2)),
        ("", "String([].average())", json!("NaN")),
        ("xs = [1, 2, 3]", "xs.average(x -> x * 2)", json!(4)),
        ("o = {items: [1, 2]}", "o.items.average()", json!(1.5)),
        ("xs = [3, 4]", "xs.map(x -> x * 2).average()", json!(7)),
        ("xs = [1, 2, 3]", "xs.value.average()", json!(2)),
        ("xs = [1]\nxs.push(2)", "xs@first.average()", json!(1)),
        (orders, "Order.average(o -> o.total)", json!(6)),
        (orders, "Order.average(.total)", json!(6)),
        (orders, "Order[1:].average(o -> o.total)", json!(7)),
        (orders, "Order.list().average(o -> o.total)", json!(6)),
        (kind.as_str(), "kind.average(o -> o.total)", json!(6)),
        ("class Empty", "String(Empty.average())", json!("NaN")),
        (bag, "String(Bag.average())", json!("NaN")),
        (bag, "Bag.average(b -> b.average())", json!(9)),
        ("", "[0.1, 0.2, 0.3].average()", json!(0.20000000000000004)),
        ("", "String([1, \"a\"].average())", json!("NaN")),
        ("xs = [1, 2]", "xs.average(x -> x, 5)", json!(1.5)),
        (
            "xs = [1, 2]\nr = 0\ntry:\n    r = xs.average()\ncatch error:\n    r = error",
            "r",
            json!(1.5),
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
            "xs.average(5)",
            "TypeError: average expects a function",
        ),
        (
            "",
            "[].average(null)",
            "TypeError: average expects a function",
        ),
        (
            orders,
            "Order.average(\"total\")",
            "TypeError: average expects a function",
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
fn selector_errors_are_raised_at_the_start_of_their_statement() {
    for (source, line) in [
        ("[].average(null)", 1),
        ("xs = [1, 2]\nxs.average(5)", 2),
        (
            "class Order(total):\n    this.total = total\nOrder(5)\nOrder.average(\"total\")",
            4,
        ),
    ] {
        assert_eq!(
            failure(source),
            expected(ErrorKind::Type, "average expects a function", line, 1),
            "{source}"
        );
    }
}

#[test]
fn class_names_run_the_population_query_beside_instance_methods() {
    let (mut run, _) = common::runners();
    run("class Bag:\n    def average(): return 9\nfirst = Bag()\nsecond = Bag()");
    assert_eq!(run("first.average()"), 9);
    assert_eq!(run("Bag.average(bag -> bag.average())"), 9);
    assert_eq!(run("String(Bag.average())"), "NaN");
    assert_eq!(run("Bag.length"), 2);
}

#[test]
fn rendering_reasoning_and_keys_keep_the_ordinary_call() {
    let mut runtime = run(r#"
temps = [18, 24]
mean = temps.average()
hot = mean > 20
class Order(amount):
    this.amount = amount
a = Order(4)
b = Order(8)
typical = Order.average(order -> order.amount)
picked = Order.average(.amount)
filtered = Order.filter(order -> order.amount > 5).average(order -> order.amount * 2)
if temps.average() > 20:
    warm = true
if sum of temps / temps.length > 20:
    warm = true
assert(warm, true)
"#);
    let (mut repl, mut fail) = common::runners();
    repl(
        "temps = [18, 24]\nmean = temps.average()\nhot = mean > 20\nclass Order(amount):\n    this.amount = amount\na = Order(4)\nb = Order(8)\ntypical = Order.average(order -> order.amount)\npicked = Order.average(.amount)\nfiltered = Order.filter(order -> order.amount > 5).average(order -> order.amount * 2)",
    );
    for (source, expected) in [
        ("(why mean)[0].rule", json!("mean = temps.average()")),
        (
            "(why typical)[0].rule",
            json!("typical = Order.average(function())"),
        ),
        (
            "(why picked)[0].rule",
            json!("picked = Order.average(.amount)"),
        ),
        (
            "(why filtered)[0].rule",
            json!("filtered = Order.filter(function()).average(function())"),
        ),
        ("(why mean)[0].from", json!(["temps"])),
        (
            "(why typical)[0].from",
            json!(["$Order", "a.amount", "b.amount"]),
        ),
        ("(why typical)[0].state", json!("derived")),
        ("(why typical)[0].holds", json!(6)),
        (
            "(affects temps).map(step -> step.rule)",
            json!(["mean = temps.average()", "hot = mean>20"]),
        ),
    ] {
        assert_eq!(repl(source), expected, "{source}");
    }
    for (source, message) in [
        (
            "why temps.average()",
            "TypeError: Cannot explain temps.average()",
        ),
        (
            "all t in temps.average(): t",
            "TypeError: temps.average() is not a collection",
        ),
        (
            "temps.average()(1)",
            "TypeError: temps.average() is not a function",
        ),
        (
            "Order.average(.amount)(1)",
            "TypeError: Order.average(.amount) is not a function",
        ),
        (
            "delete temps.average()",
            "SyntaxError: Cannot delete temps.average()",
        ),
        (
            "temps.average() = 1",
            "SyntaxError: Cannot assign to temps.average()",
        ),
    ] {
        assert_eq!(fail(source), message, "{source}");
    }
    let keys: Vec<String> = runtime
        .graph
        .nodes()
        .map(|node| node.key.to_string())
        .collect();
    for key in [
        "mean",
        "typical",
        "if(temps.average()>20)",
        "if(sum of temps/temps.length>20)",
    ] {
        assert!(keys.iter().any(|found| found == key), "{key}: {keys:?}");
    }
    checked(
        &mut runtime,
        "temps.push(30)\nassert(mean, 24)\nassert(hot, true)\nb.amount = 10\nassert(typical, 7)\nassert(filtered, 20)",
    );
}

#[test]
fn cli_runs_average_files_with_json_and_graph_output() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("list-averages.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(&fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().next(), Some("[21.0,23.0,null]"));
    assert!(
        stdout.contains("if(Reading.average(function())>20)"),
        "{stdout}"
    );
    let plain = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg(&fixture)
        .output()
        .unwrap();
    assert!(plain.status.success(), "{plain:?}");
    assert_eq!(
        String::from_utf8(plain.stdout).unwrap().lines().next(),
        Some("21,23,NaN")
    );
}

#[test]
fn repl_reads_average_lines_and_reports_their_errors() {
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
            b"class Reading(celsius):\n    this.celsius = celsius\n\n.json on\nReading(20)\nReading(23)\nmean = Reading.average(reading -> reading.celsius)\n[mean, [1, 2].average(), [].average()]\n(why mean)[0].rule\nxs = [1]\naverage of xs\nxs.average(5)\n.exit\n",
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stdout.contains("json on."), "{stdout}");
    assert!(stdout.lines().any(|line| line == "21.5"), "{stdout}");
    assert!(
        stdout.lines().any(|line| line == "[21.5,1.5,null]"),
        "{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line == "\"mean = Reading.average(function())\""),
        "{stdout}"
    );
    assert_eq!(
        stderr.lines().collect::<Vec<_>>(),
        [
            "SyntaxError: Unexpected keyword 'of' (line 1, column 12)",
            "TypeError: average expects a function (line 1, column 1)",
        ]
    );
}

#[test]
fn failed_average_updates_restore_state_and_graph() {
    let initial = r#"
def check(value):
    if value > 100:
        throw "TOO_LARGE"
    return value
values = [1, 3]
mean = values.average(check)
class Item(size):
    this.size = size
small = Item(1)
sizes = Item.average(item -> check(item.size))
assert(mean, 2)
assert(sizes, 1)
"#;
    let mut actual = run(initial);
    let control = run(initial);
    for (source, thrown) in [
        ("values.push(500)", "TOO_LARGE"),
        ("big = Item(500)", "TOO_LARGE"),
        ("mid = Item(3)\nthrow \"ROLLBACK\"", "ROLLBACK"),
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
        "suppose:\n    Item(3)\n    assert(sizes, 2)\nassert(sizes, 1)\nassert(Item.length, 1)",
    );
    checked(
        &mut actual,
        "mid = Item(3)\nassert(sizes, 2)\nvalues.push(5)\nassert(mean, 3)",
    );
}

#[test]
fn averages_stay_linear_in_the_receiver() {
    let items = 20_000;
    let list = (0..items)
        .map(|item| ((item * 7_919) % items).to_string())
        .collect::<Vec<_>>()
        .join(",\n");
    let mut runtime = Runtime::new();
    runtime
        .run(&format!(
            "values = [{list}]\ncounter = {{n: 0}}\ndef note(value):\n    counter.n = counter.n + 1\n    return value"
        ))
        .unwrap();
    assert_eq!(number(&mut runtime, "values.average()"), 9_999.5);
    assert_eq!(number(&mut runtime, "values.average(note)"), 9_999.5);
    assert_eq!(number(&mut runtime, "counter.n"), f64::from(items));
    runtime
        .run("mean = values.average(value -> value)")
        .unwrap();
    assert_eq!(number(&mut runtime, "mean"), 9_999.5);
    assert_eq!(
        runtime.run("(why mean)[0].rule").unwrap(),
        Value::String("mean = values.average(function())".to_string())
    );
}
