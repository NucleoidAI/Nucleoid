mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

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
            title: "list extreme regression".to_string(),
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

fn assert_math_parity(runtime: &mut Runtime, items: &str) {
    for query in ["max", "min"] {
        let math = number(runtime, &format!("Math.{query}({items})"));
        for source in [
            format!("[{items}].{query}()"),
            format!("[{items}].{query}(item -> item)"),
        ] {
            let extreme = number(runtime, &source);
            assert!(
                same_number(extreme, math),
                "{source}: {extreme:?} but Math.{query} gives {math:?}"
            );
        }
    }
}

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid finds list extremes with max and min",
    "Nucleoid finds extremes of class instance populations with max and min",
    "Nucleoid compares extreme values as numbers",
    "Nucleoid calls the extreme selector once per item with only the item",
    "Nucleoid keeps extremes live in class level rules",
    "Nucleoid uses extremes in functions methods and conditional expressions",
    "Nucleoid keeps max and min as ordinary names beside the extreme queries",
    "Nucleoid rolls back failed extreme updates",
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
            "Nucleoid finds extremes of lists and populations with max and min",
        ),
        (
            &readme,
            "Extremes keep the highest and lowest bids live as bids change",
        ),
        (&examples, "Extremes keep room temperature ranges live"),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(
            case.source.contains(".max(") || case.source.contains(".min("),
            "{title} uses an extreme"
        );
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 101);
}

#[test]
fn extremes_accept_every_receiver_the_list_queries_accept() {
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
Order(11)
kind = Order
class Empty
"#);
    for (source, expected) in [
        ("[1, 3, 2].max()", 3.0),
        ("[1, 3, 2].min()", 1.0),
        ("prices.max()", 10.0),
        ("prices.min()", 4.0),
        ("cart.items.max()", 2.0),
        ("prices.filter(price -> price > 5).min()", 6.0),
        ("prices.map(price -> price * -2).max()", -8.0),
        ("prices.slice(1).min()", 6.0),
        ("prices.concat([15]).max()", 15.0),
        ("prices[:2].max()", 6.0),
        ("prices.value.min()", 4.0),
        ("history@first.max()", 1.0),
        ("history.max()", 2.0),
        ("[].max()", f64::NEG_INFINITY),
        ("[].min()", f64::INFINITY),
        ("Order.max(order -> order.total)", 11.0),
        ("Order.min(order -> order.total)", 5.0),
        ("Order[1:].min(order -> order.total)", 7.0),
        ("Order[:2].max(order -> order.total)", 7.0),
        ("Order.list().max(order -> order.total)", 11.0),
        (
            "Order.filter(order -> order.total > 6).min(order -> order.total)",
            7.0,
        ),
        ("kind.max(order -> order.total)", 11.0),
        ("Empty.max()", f64::NEG_INFINITY),
        ("Empty.min()", f64::INFINITY),
        ("Empty.max(empty -> empty.size)", f64::NEG_INFINITY),
        ("Empty.min(empty -> empty.size)", f64::INFINITY),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
}

#[test]
fn extremes_equal_math_max_and_min_bit_for_bit() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut lists: Vec<String> = vec![
        String::new(),
        "0".to_string(),
        "-0".to_string(),
        "0, -0".to_string(),
        "-0, 0".to_string(),
        "-0, -0".to_string(),
        "0, 0".to_string(),
        "0, -0, 0".to_string(),
        "-0, 0, -0".to_string(),
        "1, -1".to_string(),
        "Number.NaN".to_string(),
        "1, Number.NaN, 2".to_string(),
        "Number.NaN, Number.POSITIVE_INFINITY".to_string(),
        "Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY".to_string(),
        "Number.MAX_VALUE, Number.MIN_VALUE, -Number.MAX_VALUE".to_string(),
        "9007199254740993, 9007199254740992".to_string(),
    ];
    for _ in 0..300 {
        let length = (next() % 12) as usize;
        let items: Vec<String> = (0..length)
            .map(|_| match next() % 10 {
                0 => "0".to_string(),
                1 => "-0".to_string(),
                _ => {
                    let raw = (next() % 2_000_001) as f64 - 1_000_000.0;
                    let scale = [1.0, 10.0, 1000.0, 1_000_000.0][(next() % 4) as usize];
                    format!("{}", raw / scale)
                }
            })
            .collect();
        lists.push(items.join(", "));
    }
    let mut runtime = Runtime::new();
    for items in &lists {
        assert_math_parity(&mut runtime, items);
    }
}

#[test]
fn signed_zero_ties_follow_the_math_helpers_in_order() {
    let mut runtime = Runtime::new();
    for items in [
        "0, -0",
        "-0, 0",
        "0, 0",
        "-0, -0",
        "0, -0, 0",
        "-0, 0, -0",
        "-0, 0, 0",
        "0, 0, -0",
        "-0",
        "0",
    ] {
        assert_math_parity(&mut runtime, items);
        for query in ["max", "min"] {
            let reciprocal = number(&mut runtime, &format!("1 / [{items}].{query}()"));
            let math = number(&mut runtime, &format!("1 / Math.{query}({items})"));
            assert_eq!(reciprocal, math, "1 / [{items}].{query}()");
            assert!(reciprocal.is_infinite(), "1 / [{items}].{query}()");
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[test]
fn signed_zero_ties_keep_the_later_zero_unlike_javascript_on_the_reference_build() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("1 / [0, -0].max()", f64::NEG_INFINITY),
        ("1 / Math.max(0, -0)", f64::NEG_INFINITY),
        ("1 / [-0, 0].max()", f64::INFINITY),
        ("1 / Math.max(-0, 0)", f64::INFINITY),
        ("1 / [-0, 0].min()", f64::INFINITY),
        ("1 / Math.min(-0, 0)", f64::INFINITY),
        ("1 / [0, -0].min()", f64::NEG_INFINITY),
        ("1 / Math.min(0, -0)", f64::NEG_INFINITY),
        ("1 / [-0].max()", f64::NEG_INFINITY),
        ("1 / [-0].min()", f64::NEG_INFINITY),
    ] {
        assert_eq!(number(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn compared_values_use_the_numeric_conversion_of_the_math_helpers() {
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
        for query in ["max", "min"] {
            for source in [
                format!("[{value}].{query}()"),
                format!("[{value}].{query}(item -> item)"),
                format!("Math.{query}({value})"),
            ] {
                let extreme = number(&mut runtime, &source);
                assert!(same_number(extreme, expected), "{source}: {extreme:?}");
            }
            let paired = number(&mut runtime, &format!("[{value}, 0].{query}()"));
            let math = number(&mut runtime, &format!("Math.{query}({value}, 0)"));
            assert!(same_number(paired, math), "[{value}, 0].{query}()");
        }
    }
    for (source, expected) in [
        ("[\"10\", \"9\"].max()", 10.0),
        ("[\"10\", \"9\"].min()", 9.0),
        ("[true, false].max()", 1.0),
        ("[Date(0), Date(5000)].max()", 5000.0),
        ("[\"apple\", \"pear\"].max()", f64::NAN),
        ("[Number.POSITIVE_INFINITY, 1].max()", f64::INFINITY),
        ("[Number.NEGATIVE_INFINITY, 1].min()", f64::NEG_INFINITY),
        ("[Number.NaN, 1].min()", f64::NAN),
        ("[1, Number.NaN].max()", f64::NAN),
    ] {
        let extreme = number(&mut runtime, source);
        assert!(same_number(extreme, expected), "{source}: {extreme:?}");
    }
}

#[test]
fn extremes_are_numbers_never_the_winning_item() {
    let (mut run, _) = common::runners();
    run("def hi a b => a > b ? a : b\nwhen = Date(5000)");
    for (source, expected) in [
        ("[\"10\"].max()", json!(10)),
        ("typeof [\"10\"].max()", json!("number")),
        ("[\"10\"].reduce(hi)", json!("10")),
        ("[\"10\", \"9\"].reduce(hi)", json!("9")),
        ("[true].max()", json!(1)),
        ("typeof [when].max()", json!("number")),
        ("[when].max()", json!(5000)),
        ("[null].min()", json!(0)),
        ("typeof [null].min()", json!("number")),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
}

#[test]
fn an_undefined_value_makes_the_extreme_undefined() {
    let mut runtime = run(r#"
values = [1]
values.push(values[9])
class Item
pen = Item()
pen.price = 3
book = Item()
highest = values.max()
lowest = values.min()
priciest = Item.max(item -> item.price)
cheapest = Item.min(item -> item.price)
assert(highest, null)
assert(lowest, null)
assert(priciest, null)
assert(cheapest, null)
"#);
    let (mut repl, _) = common::runners();
    repl("values = [1]\nvalues.push(values[9])\nholder = {}");
    for query in ["max", "min"] {
        assert_eq!(repl(&format!("typeof values.{query}()")), "undefined");
        assert_eq!(
            repl(&format!("typeof [Number.NaN].concat(values).{query}()")),
            "undefined"
        );
        assert_eq!(
            repl(&format!("typeof values.concat([Number.NaN]).{query}()")),
            "undefined"
        );
        assert_eq!(
            repl(&format!("typeof [1, 2].{query}(item -> holder.missing)")),
            "undefined"
        );
        assert_eq!(
            repl(&format!("typeof [].{query}(item -> holder.missing)")),
            "number"
        );
    }
    checked(
        &mut runtime,
        "book.price = 12\nassert(priciest, 12)\nassert(cheapest, 3)\nvalues.pop()\nassert(highest, 1)\nassert(lowest, 1)",
    );
}

#[test]
fn null_items_count_zero_while_reads_inside_a_selector_settle() {
    let mut runtime = run(r#"
values = [5, null, 2]
plain = values.min()
top = values.max()
selected = values.min(value -> value)
reduced = values.reduce((low, value) -> Math.min(low, value))
class Item(price):
    this.price = price
pen = Item(3)
gift = Item(null)
cheapest = Item.min(item -> item.price)
mapped = Item.map(item -> item.price)
assert(plain, 0)
assert(top, 5)
assert(selected, null)
assert(reduced, null)
assert(cheapest, null)
assert(mapped, null)
"#);
    for source in [
        "values.min()",
        "values.min(value -> value)",
        "values.reduce((low, value) -> Math.min(low, value))",
        "Math.min(5, null, 2)",
    ] {
        assert_eq!(number(&mut runtime, source), 0.0, "{source}");
    }
    checked(
        &mut runtime,
        "gift.price = 5\nassert(cheapest, 3)\nvalues.push(null)\nassert(plain, 0)\nvalues.shift()\nvalues.pop()\nvalues.shift()\nassert(plain, 2)",
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
    for (query, extreme, visits) in [
        (
            "values.max(note)",
            json!(3),
            json!([[3, null, null], [1, null, null], [2, null, null]]),
        ),
        (
            "values.min(note)",
            json!(1),
            json!([[3, null, null], [1, null, null], [2, null, null]]),
        ),
        (
            "Item.max(item -> note(item.size))",
            json!(7),
            json!([[5, null, null], [7, null, null]]),
        ),
        (
            "String([\"x\", 1, 2].min(note))",
            json!("NaN"),
            json!([["x", null, null], [1, null, null], [2, null, null]]),
        ),
        ("String([].max(note))", json!("-Infinity"), json!([])),
        ("String([].min(note))", json!("Infinity"), json!([])),
    ] {
        run("log = []");
        assert_eq!(run(query), extreme, "{query}");
        assert_eq!(run("log"), visits, "{query}");
    }
    run("log = []\nvalues.push(values[9])");
    assert_eq!(run("typeof values.max(note)"), "undefined");
    assert_eq!(run("log.length"), 4);
    run("log = []\nItem(1)");
    assert_eq!(run("Item.min(item -> note(item.size))"), 1);
    assert_eq!(
        run("log"),
        json!([[5, null, null], [7, null, null], [1, null, null]])
    );
}

#[test]
fn a_supplied_selector_must_be_a_function() {
    let (mut run, mut fail) = common::runners();
    run("class Empty\nclass Item\nItem()\nlog = []\nnothing = null\nholder = {}");
    for query in ["max", "min"] {
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
                let source = format!("{receiver}.{query}({argument})");
                let message = format!("TypeError: {query} expects a function");
                assert_eq!(fail(&source), message.as_str(), "{source}");
            }
        }
        for source in [
            format!("[1].{query}(5, missing)"),
            format!("[1].{query}(missing, 5)"),
            format!("Item.{query}(missing)"),
        ] {
            assert_eq!(
                fail(&source),
                "ReferenceError: missing is not defined",
                "{source}"
            );
        }
        assert_eq!(run(&format!("nothing.{query}(log.push(1))")), json!(null));
        assert_eq!(run(&format!("typeof nothing.{query}()")), "undefined");
        assert_eq!(run("log"), json!([]));
    }
    assert_eq!(
        run("[1, 2].max(value -> value, log.push(\"a\"), log.push(\"b\"))"),
        2
    );
    assert_eq!(run("log"), json!(["a", "b"]));
    assert_eq!(run("[1, 2].max(value -> value, 99)"), 2);
    assert_eq!(run("[1, 2].min(value -> value, -99)"), 1);
    assert_eq!(run("[1, 2].max(value -> -value, 5)"), -1);
    assert_eq!(run("[].max(value -> value, 5)"), f64::NEG_INFINITY);
    assert_eq!(run("[].min(value -> value, -99)"), f64::INFINITY);
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
class Item(size):
    this.size = size
Item(1)
def grow(size):
    Item(size)
    return item -> item.size
"#);
    assert_eq!(run("source().max(pick(), extra())"), 30);
    assert_eq!(run("calls"), json!(["receiver", "selector", "extra"]));
    run("calls = []");
    assert_eq!(run("source().min(pick(), extra())"), 10);
    assert_eq!(run("calls"), json!(["receiver", "selector", "extra"]));
    assert_eq!(run("Item.max(grow(9))"), 1);
    assert_eq!(run("Item.length"), 2);
    assert_eq!(run("Item.min(grow(-9))"), 1);
    assert_eq!(run("Item.length"), 3);
}

#[test]
fn extremes_depend_on_the_receiver_the_arguments_and_the_selector_reads() {
    let mut runtime = run(r#"
values = [1, 2]
top = values.max()
frozen = values.value.min()
class Order(amount):
    this.amount = amount
a = Order(1)
b = Order(2)
highest = Order.max(order -> order.amount)
rate = 2
def scaled order => order.amount * rate
lowest = Order.min(scaled)
assert(top, 2)
assert(highest, 2)
assert(lowest, 2)
"#);
    assert_eq!(variable_dependencies(&runtime, "top"), ["values"]);
    assert!(variable_dependencies(&runtime, "frozen").is_empty());
    assert_eq!(
        variable_dependencies(&runtime, "highest"),
        ["$Order", "a.amount", "b.amount"]
    );
    assert_eq!(
        variable_dependencies(&runtime, "lowest"),
        ["$Order", "scaled", "a.amount", "rate", "b.amount"]
    );
    checked(
        &mut runtime,
        "values.push(3)\nassert(top, 3)\nassert(frozen, 1)\nc = Order(3)\nassert(highest, 3)\n\
         rate = 3\nassert(lowest, 3)\na.amount = 5\nassert(highest, 5)\nassert(lowest, 6)",
    );
    assert_eq!(
        variable_dependencies(&runtime, "highest"),
        ["$Order", "a.amount", "b.amount", "c.amount"]
    );
}

#[test]
fn populations_gaining_and_losing_instances_re_evaluate_extremes() {
    run(r#"
class Visit(minutes):
    this.minutes = minutes
longest = Visit.max(visit -> visit.minutes)
shortest = Visit.min(visit -> visit.minutes)
assert(longest, Number.NEGATIVE_INFINITY)
assert(shortest, Number.POSITIVE_INFINITY)
first = Visit(10)
second = Visit(20)
assert(longest, 20)
assert(shortest, 10)
third = Visit(5)
assert(shortest, 5)
delete second.minutes
assert(longest, null)
delete second
assert(longest, 10)
assert(shortest, 5)
assert(Visit.length, 2)
delete third.minutes
delete third
delete first.minutes
delete first
assert(longest, Number.NEGATIVE_INFINITY)
assert(shortest, Number.POSITIVE_INFINITY)
class Empty
none = Empty.max(empty -> empty.size)
assert(none, Number.NEGATIVE_INFINITY)
Empty()
assert(none, null)
"#);
}

#[test]
fn class_level_rules_take_extremes_for_each_instance() {
    let mut runtime = run(r#"
class Player(team, score):
    this.team = team
    this.score = score
class Team(name, floors):
    this.name = name
    this.floors = floors
$Team.best = Player.filter(player -> player.team == $Team).max(player -> player.score)
$Team.floor = $Team.floors.min()
red = Team("red", [4, 2])
blue = Team("blue", [])
Player(red, 3)
ana = Player(blue, 4)
assert(red.best, 3)
assert(blue.best, 4)
assert(red.floor, 2)
assert(blue.floor, Number.POSITIVE_INFINITY)
ana.score = 6
blue.floors.push(9)
assert(blue.best, 6)
assert(blue.floor, 9)
Player(red, 8)
assert(red.best, 8)
"#);
    let best = dependencies(
        &runtime,
        &NodeKey::property(&ObjectId::named("blue"), "best"),
    );
    for key in ["$Player", "ana.score"] {
        assert!(best.contains(&key), "{key} in {best:?}");
    }
    let floor = dependencies(
        &runtime,
        &NodeKey::property(&ObjectId::named("blue"), "floor"),
    );
    assert!(floor.contains(&"blue.floors"), "blue.floors in {floor:?}");
    checked(
        &mut runtime,
        "green = Team(\"green\", [5])\nassert(green.best, Number.NEGATIVE_INFINITY)\nassert(green.floor, 5)",
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
report = {max: 5, min: 1}
holder = {max: value -> value * 2}
empty = {}
max = 3
min = 1
class Review
$Review.rate = $Review.max / 10
review = Review()
review.max = 42
def twice max => max * 2
def half min => min / 2
nothing = null
count = 5
flag = true
when = Date(5)
pattern = /a/
def helper(value) => value
"#);
    for (source, expected) in [
        ("values.max", json!(null)),
        ("values.min", json!(null)),
        ("typeof values.max", json!("undefined")),
        ("typeof values.min", json!("undefined")),
        ("Order.max", json!(null)),
        ("typeof Order.min", json!("undefined")),
        ("Math.max", json!(null)),
        ("typeof Math.max", json!("undefined")),
        ("typeof Math.min", json!("undefined")),
        ("report.max", json!(5)),
        ("report.min", json!(1)),
        ("holder.max(4)", json!(8)),
        ("empty.max()", json!(null)),
        ("typeof empty.min()", json!("undefined")),
        ("max", json!(3)),
        ("min", json!(1)),
        ("review.max", json!(42)),
        ("review.rate", json!(4.2)),
        ("twice(4)", json!(8)),
        ("half(4)", json!(2)),
        ("nothing.max()", json!(null)),
        ("nothing.min()", json!(null)),
        ("values.max() + max", json!(5)),
        ("Math.max(2, 7)", json!(7)),
        ("Math.min(2, 7)", json!(2)),
        ("Math.max([7])", json!(7)),
        ("String(Math.max())", json!("-Infinity")),
        ("String(Math.min())", json!("Infinity")),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    for (source, message) in [
        ("\"abc\".max()", "TypeError: abc.max is not a function"),
        ("count.min()", "TypeError: number.min is not a function"),
        ("flag.max()", "TypeError: boolean.max is not a function"),
        ("when.min()", "TypeError: Date.min is not a function"),
        ("pattern.max()", "TypeError: max is not a function"),
        ("helper.min()", "TypeError: function.min is not a function"),
        ("Class.max()", "ReferenceError: Class is not defined"),
        ("List.min()", "ReferenceError: List is not defined"),
        (
            "values.max(missing)",
            "ReferenceError: missing is not defined",
        ),
    ] {
        assert_eq!(fail(source), message, "{source}");
    }
    let (mut defined, mut missing) = common::runners();
    assert_eq!(missing("max([1, 2])"), "ReferenceError: max is not defined");
    assert_eq!(missing("min([1, 2])"), "ReferenceError: min is not defined");
    defined("def max(left, right) => left > right ? left : right");
    assert_eq!(defined("max(1, 2)"), 2);
    assert_eq!(defined("[1, 2].max()"), 2);
    assert_eq!(defined("[1, 2].min()"), 1);
}

#[test]
fn compatibility_rows_give_the_documented_results() {
    let orders = "class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)";
    let bag = "class Bag:\n    def max(): return 9\n    def min(): return 1\nbag = Bag()";
    let kind = format!("{orders}\nkind = Order");
    for (setup, source, expected) in [
        ("", "[1, 2, 3].max()", json!(3)),
        ("", "String([].min())", json!("Infinity")),
        ("xs = [1, 2, 3]", "xs.max(x -> x * 2)", json!(6)),
        ("xs = [1, 2, 3]", "xs.min()", json!(1)),
        ("o = {items: [1, 2]}", "o.items.max()", json!(2)),
        ("xs = [3, 4]", "xs.map(x -> x * 2).min()", json!(6)),
        ("xs = [1, 2, 3]", "xs.value.max()", json!(3)),
        ("xs = [1]\nxs.push(2)", "xs@first.max()", json!(1)),
        (orders, "Order.max(o -> o.total)", json!(7)),
        (orders, "Order.min(o -> o.total)", json!(5)),
        (orders, "Order[1:].max(o -> o.total)", json!(7)),
        (orders, "Order.list().min(o -> o.total)", json!(5)),
        (kind.as_str(), "kind.max(o -> o.total)", json!(7)),
        ("class Empty", "String(Empty.max())", json!("-Infinity")),
        ("class Empty", "String(Empty.min())", json!("Infinity")),
        (bag, "String(Bag.max())", json!("NaN")),
        (bag, "Bag.max(b -> b.max())", json!(9)),
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
            "xs.max(5)",
            "TypeError: max expects a function",
        ),
        ("", "[].min(null)", "TypeError: min expects a function"),
        (
            orders,
            "Order.max(\"total\")",
            "TypeError: max expects a function",
        ),
    ] {
        let (mut run, mut fail) = common::runners();
        if !setup.is_empty() {
            run(setup);
        }
        assert_eq!(fail(source), message, "{setup}\n{source}");
    }
    let (mut run, _) = common::runners();
    run("x = {max: 4}.max\ny = {min: 1}.min");
    assert_eq!(run("[x, y]"), json!([4, 1]));
}

#[test]
fn class_names_run_the_population_queries_beside_instance_methods() {
    let (mut run, _) = common::runners();
    run(
        "class Bag:\n    def max(): return 9\n    def min(): return 1\nfirst = Bag()\nsecond = Bag()",
    );
    assert_eq!(run("first.max()"), 9);
    assert_eq!(run("second.min()"), 1);
    assert_eq!(run("Bag.max(bag -> bag.max())"), 9);
    assert_eq!(run("Bag.min(bag -> bag.min())"), 1);
    assert_eq!(run("String(Bag.max())"), "NaN");
    assert_eq!(run("String(Bag.min())"), "NaN");
    assert_eq!(run("Bag.length"), 2);
}

#[test]
fn rendering_and_reasoning_keep_the_ordinary_call() {
    let (mut run, _) = common::runners();
    run(
        "values = [1, 2]\ntop = values.max()\nclass Order(amount):\n    this.amount = amount\nOrder(1)\nlowest = Order.min(order -> order.amount)",
    );
    assert_eq!(run("(why top)[0].rule"), "top = values.max()");
    assert_eq!(
        run("(why lowest)[0].rule"),
        "lowest = Order.min(function())"
    );
    assert_eq!(run("(why lowest)[0].state"), "derived");
    assert_eq!(run("(why lowest)[0].holds"), 1);
}

#[test]
fn failed_extreme_updates_restore_state_and_graph() {
    let initial = r#"
def check(value):
    if value > 100:
        throw "TOO_LARGE"
    return value
values = [1, 3]
top = values.max(check)
class Item(size):
    this.size = size
small = Item(1)
smallest = Item.min(item -> check(item.size))
assert(top, 3)
assert(smallest, 1)
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
        "tiny = Item(0)\nassert(smallest, 0)\nvalues.push(5)\nassert(top, 5)",
    );
}

#[test]
fn extremes_stay_linear_in_the_receiver() {
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
    assert_eq!(number(&mut runtime, "values.max()"), 19_999.0);
    assert_eq!(number(&mut runtime, "values.min()"), 0.0);
    assert_eq!(number(&mut runtime, "values.max(note)"), 19_999.0);
    assert_eq!(number(&mut runtime, "counter.n"), f64::from(items));
    assert_eq!(number(&mut runtime, "values.min(note)"), 0.0);
    assert_eq!(number(&mut runtime, "counter.n"), f64::from(2 * items));
    runtime.run("top = values.max(value -> value)").unwrap();
    assert_eq!(number(&mut runtime, "top"), 19_999.0);
    assert_eq!(
        runtime.run("(why top)[0].rule").unwrap(),
        Value::String("top = values.max(function())".to_string())
    );
}
