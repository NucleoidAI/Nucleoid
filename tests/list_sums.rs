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
            title: "list sum regression".to_string(),
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

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid totals lists with sum",
    "Nucleoid totals class instance populations with sum",
    "Nucleoid converts summed values to numbers",
    "Nucleoid calls the sum selector once per item with only the item",
    "Nucleoid keeps sums live in class level rules",
    "Nucleoid uses sums in functions methods and conditional expressions",
    "Nucleoid keeps sum as an ordinary name beside the sum query",
    "Nucleoid rolls back failed sum updates",
];

#[test]
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let readme = common::snippets(include_str!("../README.md"), "README.md");
    let examples = common::snippets(include_str!("../docs/examples.md"), "docs/examples.md");
    let appended = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (&reference, "Nucleoid totals lists and populations with sum"),
        (&readme, "Sums keep totals live as instances change"),
        (&examples, "Sums keep room and building power loads live"),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(case.source.contains(".sum("), "{title} uses sum");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 85);
}

#[test]
fn sums_total_every_receiver_the_list_queries_accept() {
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
        ("[1, 2, 3].sum()", 6.0),
        ("prices.sum()", 20.0),
        ("cart.items.sum()", 3.0),
        ("prices.filter(price -> price > 5).sum()", 16.0),
        ("prices.map(price -> price * 2).sum()", 40.0),
        ("prices.slice(1).sum()", 16.0),
        ("prices.concat([5]).sum()", 25.0),
        ("prices[1:].sum()", 16.0),
        ("prices.value.sum()", 20.0),
        ("history@first.sum()", 1.0),
        ("history.sum()", 3.0),
        ("[].sum()", 0.0),
        ("Order.sum(order -> order.total)", 23.0),
        ("Order[1:].sum(order -> order.total)", 18.0),
        ("Order.list().sum(order -> order.total)", 23.0),
        (
            "Order.filter(order -> order.total > 6).sum(order -> order.total)",
            18.0,
        ),
        ("kind.sum(order -> order.total)", 23.0),
        ("Empty.sum()", 0.0),
        ("Empty.sum(empty -> empty.size)", 0.0),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
}

#[test]
fn numeric_totals_equal_the_accumulator_reduce_bit_for_bit() {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut lists: Vec<String> = vec![
        "0.1, 0.2, 0.3".to_string(),
        "-0".to_string(),
        "-0, -0".to_string(),
        "1, -1".to_string(),
        "9007199254740992, 1, 1".to_string(),
        "0.1, -0.1, 0.3".to_string(),
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
        let total = number(&mut runtime, &format!("[{items}].sum()"));
        let reduced = number(
            &mut runtime,
            &format!("[{items}].reduce((total, item) => total + item, 0)"),
        );
        assert!(
            same_number(total, reduced),
            "[{items}]: {total:?} and {reduced:?}"
        );
        let selected = number(&mut runtime, &format!("[{items}].sum(item -> item)"));
        assert!(same_number(total, selected), "[{items}] with a selector");
    }
    assert!(same_number(
        number(&mut runtime, "[0.1, 0.2, 0.3].sum()"),
        0.6000000000000001
    ));
    assert_eq!(number(&mut runtime, "1 / [-0].sum()"), f64::INFINITY);
    assert_eq!(number(&mut runtime, "1 / [].sum()"), f64::INFINITY);
}

#[test]
fn summed_values_use_the_numeric_conversion_of_arithmetic() {
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
            format!("[{value}].sum()"),
            format!("[{value}].sum(item -> item)"),
            format!("[{value}, 0].sum()"),
        ] {
            let total = number(&mut runtime, &source);
            assert!(same_number(total, expected), "{source}: {total:?}");
        }
        let arithmetic = number(&mut runtime, &format!("0 + ({value} - 0)"));
        assert!(
            same_number(
                number(&mut runtime, &format!("[{value}].sum()")),
                arithmetic
            ),
            "{value} converts as subtraction does"
        );
    }
    for (source, expected) in [
        ("[Number.POSITIVE_INFINITY, 1].sum()", f64::INFINITY),
        (
            "[Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY].sum()",
            f64::NAN,
        ),
        ("[Number.NaN, 1].sum()", f64::NAN),
        ("[Number.MAX_VALUE, Number.MAX_VALUE].sum()", f64::INFINITY),
        ("[\"2\", \"3\"].sum()", 5.0),
        ("[true, true, false].sum()", 2.0),
    ] {
        let total = number(&mut runtime, source);
        assert!(same_number(total, expected), "{source}: {total:?}");
    }
    assert_eq!(
        runtime
            .run("[\"2\", \"3\"].reduce((total, item) => total + item, 0)")
            .unwrap(),
        Value::String("023".to_string())
    );
}

#[test]
fn an_undefined_value_makes_the_total_undefined() {
    let mut runtime = run(r#"
values = [1]
values.push(values[9])
class Item
pen = Item()
pen.price = 3
book = Item()
total = values.sum()
spent = Item.sum(item -> item.price)
assert(total, null)
assert(spent, null)
"#);
    let (mut repl, _) = common::runners();
    repl("values = [1]\nvalues.push(values[9])\nholder = {}");
    assert_eq!(repl("typeof values.sum()"), "undefined");
    assert_eq!(
        repl("typeof [Number.NaN].concat(values).sum()"),
        "undefined"
    );
    assert_eq!(
        repl("typeof [1, 2].sum(item -> holder.missing)"),
        "undefined"
    );
    assert_eq!(repl("typeof [].sum(item -> holder.missing)"), "number");
    checked(
        &mut runtime,
        "book.price = 12\nassert(spent, 15)\nvalues.pop()\nassert(total, 1)",
    );
}

#[test]
fn null_items_count_zero_while_reads_inside_a_selector_settle() {
    let mut runtime = run(r#"
values = [1, null, 2]
plain = values.sum()
selected = values.sum(value -> value)
reduced = values.reduce((total, value) -> total + value, 0)
class Item(price):
    this.price = price
pen = Item(3)
gift = Item(null)
spent = Item.sum(item -> item.price)
mapped = Item.map(item -> item.price)
assert(plain, 3)
assert(selected, null)
assert(reduced, null)
assert(spent, null)
assert(mapped, null)
"#);
    for source in [
        "values.sum()",
        "values.sum(value -> value)",
        "values.reduce((total, value) -> total + value, 0)",
    ] {
        assert_eq!(number(&mut runtime, source), 3.0, "{source}");
    }
    checked(
        &mut runtime,
        "gift.price = 5\nassert(spent, 8)\nvalues.push(null)\nassert(plain, 3)",
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
    for (query, total, visits) in [
        (
            "values.sum(note)",
            json!(6),
            json!([[3, null, null], [1, null, null], [2, null, null]]),
        ),
        (
            "Item.sum(item -> note(item.size))",
            json!(12),
            json!([[5, null, null], [7, null, null]]),
        ),
        (
            "String([\"x\", 1, 2].sum(note))",
            json!("NaN"),
            json!([["x", null, null], [1, null, null], [2, null, null]]),
        ),
        ("[].sum(note)", json!(0), json!([])),
    ] {
        run("log = []");
        assert_eq!(run(query), total, "{query}");
        assert_eq!(run("log"), visits, "{query}");
    }
    run("log = []\nvalues.push(values[9])");
    assert_eq!(run("typeof values.sum(note)"), "undefined");
    assert_eq!(run("log.length"), 4);
    run("log = []\nItem(1)");
    assert_eq!(run("Item.sum(item -> note(item.size))"), 13);
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
            let source = format!("{receiver}.sum({argument})");
            assert_eq!(
                fail(&source),
                "TypeError: sum expects a function",
                "{source}"
            );
        }
    }
    assert_eq!(
        fail("[1].sum(5, missing)"),
        "ReferenceError: missing is not defined"
    );
    assert_eq!(
        fail("[1].sum(missing, 5)"),
        "ReferenceError: missing is not defined"
    );
    assert_eq!(
        fail("Item.sum(missing)"),
        "ReferenceError: missing is not defined"
    );
    assert_eq!(run("nothing.sum(log.push(1))"), json!(null));
    assert_eq!(run("typeof nothing.sum()"), "undefined");
    assert_eq!(run("log"), json!([]));
    assert_eq!(
        run("[1, 2].sum(value -> value, log.push(\"a\"), log.push(\"b\"))"),
        3
    );
    assert_eq!(run("log"), json!(["a", "b"]));
    assert_eq!(run("[1, 2].sum(value -> value, 99)"), 3);
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
    assert_eq!(run("source().sum(pick(), extra())"), 60);
    assert_eq!(run("calls"), json!(["receiver", "selector", "extra"]));
    assert_eq!(run("Item.sum(grow())"), 1);
    assert_eq!(run("Item.length"), 2);
}

#[test]
fn sums_depend_on_the_receiver_the_arguments_and_the_selector_reads() {
    let mut runtime = run(r#"
values = [1, 2]
total = values.sum()
frozen = values.value.sum()
class Order(amount):
    this.amount = amount
a = Order(1)
b = Order(2)
revenue = Order.sum(order -> order.amount)
rate = 2
def scaled order => order.amount * rate
weighted = Order.sum(scaled)
assert(total, 3)
assert(revenue, 3)
assert(weighted, 6)
"#);
    assert_eq!(variable_dependencies(&runtime, "total"), ["values"]);
    assert!(variable_dependencies(&runtime, "frozen").is_empty());
    assert_eq!(
        variable_dependencies(&runtime, "revenue"),
        ["$Order", "a.amount", "b.amount"]
    );
    assert_eq!(
        variable_dependencies(&runtime, "weighted"),
        ["$Order", "scaled", "a.amount", "rate", "b.amount"]
    );
    checked(
        &mut runtime,
        "values.push(3)\nassert(total, 6)\nassert(frozen, 3)\nc = Order(3)\nassert(revenue, 6)\n\
         rate = 3\nassert(weighted, 18)\nb.amount = 5\nassert(revenue, 9)\nassert(weighted, 27)",
    );
    assert_eq!(
        variable_dependencies(&runtime, "revenue"),
        ["$Order", "a.amount", "b.amount", "c.amount"]
    );
}

#[test]
fn populations_gaining_and_losing_instances_re_evaluate_sums() {
    run(r#"
class Visit(minutes):
    this.minutes = minutes
first = Visit(10)
second = Visit(20)
total = Visit.sum(visit -> visit.minutes)
assert(total, 30)
third = Visit(5)
assert(total, 35)
delete second.minutes
assert(total, null)
delete second
assert(total, 15)
assert(Visit.length, 2)
class Empty
none = Empty.sum(empty -> empty.size)
assert(none, 0)
Empty()
assert(none, null)
"#);
}

#[test]
fn class_level_rules_sum_for_each_instance() {
    let mut runtime = run(r#"
class Player(team, score):
    this.team = team
    this.score = score
class Team(name, bonuses):
    this.name = name
    this.bonuses = bonuses
$Team.points = Player.filter(player -> player.team == $Team).sum(player -> player.score) + $Team.bonuses.sum()
red = Team("red", [1])
blue = Team("blue", [])
Player(red, 3)
ana = Player(blue, 4)
assert(red.points, 4)
assert(blue.points, 4)
ana.score = 6
red.bonuses.push(2)
assert(blue.points, 6)
assert(red.points, 6)
Player(red, 1)
assert(red.points, 7)
"#);
    let points = dependencies(
        &runtime,
        &NodeKey::property(&ObjectId::named("blue"), "points"),
    );
    for key in ["$Player", "ana.score", "blue.bonuses"] {
        assert!(points.contains(&key), "{key} in {points:?}");
    }
    checked(
        &mut runtime,
        "green = Team(\"green\", [5])\nassert(green.points, 5)",
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
report = {sum: 5}
holder = {sum: value -> value * 2}
empty = {}
sum = 3
class Review
$Review.rate = $Review.sum / 10
review = Review()
review.sum = 42
def twice sum => sum * 2
nothing = null
count = 5
flag = true
when = Date(5)
pattern = /a/
def helper(value) => value
"#);
    for (source, expected) in [
        ("values.sum", json!(null)),
        ("typeof values.sum", json!("undefined")),
        ("Order.sum", json!(null)),
        ("typeof Order.sum", json!("undefined")),
        ("Math.sum", json!(null)),
        ("typeof Math.sum", json!("undefined")),
        ("report.sum", json!(5)),
        ("holder.sum(4)", json!(8)),
        ("empty.sum()", json!(null)),
        ("typeof empty.sum()", json!("undefined")),
        ("sum", json!(3)),
        ("review.sum", json!(42)),
        ("review.rate", json!(4.2)),
        ("twice(4)", json!(8)),
        ("nothing.sum()", json!(null)),
        ("values.sum() + sum", json!(6)),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    for (source, message) in [
        ("\"abc\".sum()", "TypeError: abc.sum is not a function"),
        ("count.sum()", "TypeError: number.sum is not a function"),
        ("flag.sum()", "TypeError: boolean.sum is not a function"),
        ("when.sum()", "TypeError: Date.sum is not a function"),
        ("pattern.sum()", "TypeError: sum is not a function"),
        ("helper.sum()", "TypeError: function.sum is not a function"),
        ("Math.sum([1])", "TypeError: Math.sum is not a function"),
        ("Class.sum()", "ReferenceError: Class is not defined"),
        ("List.sum()", "ReferenceError: List is not defined"),
        (
            "values.sum(missing)",
            "ReferenceError: missing is not defined",
        ),
    ] {
        assert_eq!(fail(source), message, "{source}");
    }
    let (mut defined, mut missing) = common::runners();
    assert_eq!(missing("sum([1, 2])"), "ReferenceError: sum is not defined");
    defined("def sum(left, right) => left + right");
    assert_eq!(defined("sum(1, 2)"), 3);
    assert_eq!(defined("[1, 2].sum()"), 3);
}

#[test]
fn class_names_run_the_population_query_beside_instance_methods() {
    let (mut run, _) = common::runners();
    run("class Bag:\n    def sum(): return 9\nfirst = Bag()\nsecond = Bag()");
    assert_eq!(run("first.sum()"), 9);
    assert_eq!(run("Bag.sum(bag -> bag.sum())"), 18);
    assert_eq!(run("String(Bag.sum())"), "NaN");
    assert_eq!(run("Bag.length"), 2);
}

#[test]
fn rendering_and_reasoning_keep_the_ordinary_call() {
    let (mut run, _) = common::runners();
    run(
        "values = [1, 2]\ntotal = values.sum()\nclass Order(amount):\n    this.amount = amount\nOrder(1)\nrevenue = Order.sum(order -> order.amount)",
    );
    assert_eq!(run("(why total)[0].rule"), "total = values.sum()");
    assert_eq!(
        run("(why revenue)[0].rule"),
        "revenue = Order.sum(function())"
    );
    assert_eq!(run("(why revenue)[0].state"), "derived");
    assert_eq!(run("(why revenue)[0].holds"), 1);
}

#[test]
fn failed_sum_updates_restore_state_and_graph() {
    let initial = r#"
def check(value):
    if value > 100:
        throw "TOO_LARGE"
    return value
values = [1, 3]
total = values.sum(check)
class Item(size):
    this.size = size
small = Item(1)
sizes = Item.sum(item -> check(item.size))
assert(total, 4)
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
        "mid = Item(3)\nassert(sizes, 4)\nvalues.push(5)\nassert(total, 9)",
    );
}

#[test]
fn sums_stay_linear_in_the_receiver() {
    let items = 20_000;
    let list = (0..items)
        .map(|item| item.to_string())
        .collect::<Vec<_>>()
        .join(",\n");
    let mut runtime = Runtime::new();
    runtime
        .run(&format!(
            "values = [{list}]\ncounter = {{n: 0}}\ndef note(value):\n    counter.n = counter.n + 1\n    return value"
        ))
        .unwrap();
    assert_eq!(number(&mut runtime, "values.sum()"), 199_990_000.0);
    assert_eq!(number(&mut runtime, "values.sum(note)"), 199_990_000.0);
    assert_eq!(number(&mut runtime, "counter.n"), f64::from(items));
    runtime.run("total = values.sum(value -> value)").unwrap();
    assert_eq!(number(&mut runtime, "total"), 199_990_000.0);
    assert_eq!(
        runtime.run("(why total)[0].rule").unwrap(),
        Value::String("total = values.sum(function())".to_string())
    );
}
