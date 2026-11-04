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
            title: "list extreme item regression".to_string(),
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

const COMPARABLE: &[&str] = &[
    "1",
    "-2",
    "0",
    "-0",
    "2.5",
    "1 / 0",
    "-1 / 0",
    "true",
    "false",
    "null",
    "new Date(5)",
    "[7]",
    "[]",
    "[true]",
];
const TEXT: &[&str] = &[
    "\"a\"", "\"B\"", "\"10\"", "\"9\"", "\" 7 \"", "\"\"", "\"abc\"", "\"ab\"",
];
const UNORDERED: &[&str] = &[
    "0 / 0", "", "{a: 1}", "[1, 2]", "[\"x\"]", "/a/", "(v -> v)",
];

fn lists(
    seed: u64,
    keys: &[&str],
    count: usize,
    lengths: std::ops::RangeInclusive<u64>,
) -> Vec<String> {
    let mut state = seed;
    let mut next = move |bound: u64| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) % bound
    };
    let span = lengths.end() - lengths.start() + 1;
    (0..count)
        .map(|_| {
            let length = lengths.start() + next(span);
            let items: Vec<String> = (0..length)
                .map(|index| {
                    let pick = usize::try_from(next(keys.len() as u64)).unwrap();
                    match keys[pick] {
                        "" => format!("{{i: {index}}}"),
                        key => format!("{{i: {index}, k: {key}}}"),
                    }
                })
                .collect();
            format!("[{}]", items.join(", "))
        })
        .collect()
}

fn index_of(item: &str) -> String {
    format!("{item} == null ? -1 : {item}.i")
}

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid finds the extreme items of lists with maxBy and minBy",
    "Nucleoid finds the extreme instances of class populations with maxBy and minBy",
    "Nucleoid compares maxBy and minBy keys as the comparison operators do",
    "Nucleoid calls the extreme item selector once per item with only the item",
    "Nucleoid keeps extreme items live in class level rules",
    "Nucleoid uses extreme items in functions methods and conditional expressions",
    "Nucleoid keeps maxBy and minBy as ordinary names beside the extreme item queries",
    "Nucleoid rolls back failed extreme item updates",
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
            "Nucleoid finds the extreme items of lists and populations with maxBy and minBy",
        ),
        (
            &readme,
            "Extreme items keep the leading and lowest bids live as bids change",
        ),
        (
            &examples,
            "Extreme items keep the nearest free charger of every robot live",
        ),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(
            case.source.contains(".maxBy(") || case.source.contains(".minBy("),
            "{title} uses an extreme item"
        );
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 107);
}

#[test]
fn extreme_items_accept_every_receiver_the_list_queries_accept() {
    let (mut run, _) = common::runners();
    run(r#"
prices = [4, 6, 10]
cart = {items: [{sku: "pen", price: 2}, {sku: "ink", price: 9}]}
history = [1]
history.push(2)
class Order(total):
    this.total = total
Order(5)
Order(11)
Order(7)
kind = Order
class Empty
"#);
    for (source, expected) in [
        ("[1, 3, 2].maxBy(x -> x)", json!(3)),
        ("[1, 3, 2].minBy(x -> x)", json!(1)),
        ("prices.maxBy(price -> -price)", json!(4)),
        ("prices.minBy(price -> price % 4)", json!(4)),
        ("cart.items.maxBy(item -> item.price).sku", json!("ink")),
        (
            "prices.filter(price -> price > 5).minBy(price -> price)",
            json!(6),
        ),
        (
            "prices.map(price -> price * -2).maxBy(price -> price)",
            json!(-8),
        ),
        ("prices.slice(1).minBy(price -> price)", json!(6)),
        ("prices.concat([15]).maxBy(price -> price)", json!(15)),
        ("prices[:2].maxBy(price -> price)", json!(6)),
        ("prices.value.minBy(price -> price)", json!(4)),
        ("history@first.maxBy(value -> value)", json!(1)),
        ("history.maxBy(value -> value)", json!(2)),
        ("typeof [].maxBy(x -> x)", json!("undefined")),
        ("typeof [].minBy(x -> x)", json!("undefined")),
        ("Order.maxBy(order -> order.total).total", json!(11)),
        ("Order.minBy(order -> order.total).total", json!(5)),
        ("Order[1:].minBy(order -> order.total).total", json!(7)),
        ("Order[:2].maxBy(order -> order.total).total", json!(11)),
        ("Order.list().maxBy(order -> order.total).total", json!(11)),
        (
            "Order.filter(order -> order.total < 10).maxBy(order -> order.total).total",
            json!(7),
        ),
        ("kind.minBy(order -> order.total).total", json!(5)),
        (
            "typeof Empty.maxBy(empty -> empty.size)",
            json!("undefined"),
        ),
        (
            "typeof Empty.minBy(empty -> empty.size)",
            json!("undefined"),
        ),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
}

#[test]
fn the_result_is_the_item_itself_never_its_key() {
    let (mut run, _) = common::runners();
    run(r#"
first = {at: 9, c: 18}
second = {at: 12, c: 24}
readings = [first, second, {at: 15, c: 24}]
class Bid(amount): this.amount = amount
low = Bid(3)
high = Bid(9)
"#);
    assert_eq!(run("readings.maxBy(reading -> reading.c) == second"), true);
    assert_eq!(run("readings.minBy(reading -> reading.c) == first"), true);
    assert_eq!(
        run("readings.maxBy(reading -> reading.c)"),
        json!({"at": 12, "c": 24})
    );
    assert_eq!(run("Bid.maxBy(bid -> bid.amount) == high"), true);
    assert_eq!(run("Bid.minBy(bid -> bid.amount) == low"), true);
    assert_eq!(run("[\"10\", \" 7 \"].minBy(text -> Number(text))"), " 7 ");
    assert_eq!(run("[[1, 2], [3]].maxBy(row -> row.length)"), json!([1, 2]));
}

#[test]
fn keys_compare_as_the_comparison_operators_compare() {
    let (mut run, _) = common::runners();
    run("a = new Date(1000)\nb = new Date(5000)\nc = new Date(3000)");
    for (source, expected) in [
        (
            "[\"mia\", \"Ada\", \"zoe\", \"bo\"].minBy(name -> name)",
            json!("Ada"),
        ),
        (
            "[\"mia\", \"Ada\", \"zoe\", \"bo\"].maxBy(name -> name)",
            json!("zoe"),
        ),
        ("[\"10\", \"9\"].maxBy(code -> code)", json!("9")),
        ("[\"10\", \"9\"].minBy(code -> code)", json!("10")),
        ("[\"10\", \"9\"].maxBy(code -> Number(code))", json!("10")),
        ("[40, \"75\", 40].maxBy(kg -> kg)", json!("75")),
        ("[40, \"9\", 40].maxBy(kg -> kg)", json!(40)),
        ("[\"b\", 5].maxBy(key -> key)", json!("b")),
        ("[5, \"b\"].maxBy(key -> key)", json!(5)),
        ("[5, \"b\", 7].maxBy(key -> key)", json!(7)),
        ("[\"b\", 5, 7, \"c\"].maxBy(key -> key)", json!("c")),
        ("[a, b, c].maxBy(date -> date) == b", json!(true)),
        ("[a, b, c].minBy(date -> date) == a", json!(true)),
        ("[true, false, true].maxBy(flag -> flag)", json!(true)),
        ("[true, false, true].minBy(flag -> flag)", json!(false)),
        (
            "[{k: null, i: 0}, {k: -1, i: 1}].maxBy(x -> x.k).i",
            json!(0),
        ),
        (
            "[{k: null, i: 0}, {k: 1, i: 1}].maxBy(x -> x.k).i",
            json!(1),
        ),
        ("[{k: [5], i: 0}, {k: 3, i: 1}].maxBy(x -> x.k).i", json!(0)),
        ("[{k: [], i: 0}, {k: -3, i: 1}].minBy(x -> x.k).i", json!(1)),
        ("[1 / 0, 7, -1 / 0].maxBy(x -> x)", json!(null)),
        (
            "String([1 / 0, 7, -1 / 0].maxBy(x -> x))",
            json!("Infinity"),
        ),
        (
            "String([1 / 0, 7, -1 / 0].minBy(x -> x))",
            json!("-Infinity"),
        ),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
}

#[test]
fn the_first_item_wins_a_tie() {
    let (mut run, _) = common::runners();
    for (source, expected) in [
        (
            "[{i: 0, k: 2}, {i: 1, k: 2}, {i: 2, k: 1}].maxBy(x -> x.k).i",
            0,
        ),
        (
            "[{i: 0, k: 1}, {i: 1, k: 2}, {i: 2, k: 2}].maxBy(x -> x.k).i",
            1,
        ),
        (
            "[{i: 0, k: 2}, {i: 1, k: 1}, {i: 2, k: 1}].minBy(x -> x.k).i",
            1,
        ),
        ("[{i: 0, k: 0}, {i: 1, k: -0}].maxBy(x -> x.k).i", 0),
        ("[{i: 0, k: -0}, {i: 1, k: 0}].maxBy(x -> x.k).i", 0),
        ("[{i: 0, k: 0}, {i: 1, k: -0}].minBy(x -> x.k).i", 0),
        ("[{i: 0, k: -0}, {i: 1, k: 0}].minBy(x -> x.k).i", 0),
        ("[{i: 0, k: \"a\"}, {i: 1, k: \"a\"}].minBy(x -> x.k).i", 0),
        ("[{i: 0, k: 1}, {i: 1, k: true}].maxBy(x -> x.k).i", 0),
        ("[{i: 0, k: true}, {i: 1, k: \"1\"}].maxBy(x -> x.k).i", 0),
        (
            "[{i: 0, k: null}, {i: 1, k: false}, {i: 2, k: 0}].minBy(x -> x.k).i",
            0,
        ),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    assert_eq!(
        run("String(1 / [-0, 0].maxBy(x -> x)) + String(1 / [0, -0].minBy(x -> x))"),
        "-InfinityInfinity"
    );
}

#[test]
fn a_key_that_cannot_be_compared_with_itself_makes_the_result_undefined() {
    let (mut run, _) = common::runners();
    run(r#"
log = []
def note(item):
    log.push(item.i)
    return item.k
class Kind
def helper(value) => value
"#);
    for key in [
        "0 / 0", "{}.none", "{a: 1}", "Kind", "helper", "/a/", "[1, 2]", "[\"x\"]", "[0 / 0]",
    ] {
        for query in ["maxBy", "minBy"] {
            for position in 0..3 {
                let items: Vec<String> = (0..3)
                    .map(|index| {
                        let value = if index == position {
                            key.to_string()
                        } else {
                            (index * 10 + 1).to_string()
                        };
                        format!("{{i: {index}, k: {value}}}")
                    })
                    .collect();
                let source = format!("typeof [{}].{query}(note)", items.join(", "));
                run("log = []");
                assert_eq!(run(&source), "undefined", "{source}");
                assert_eq!(run("log"), json!([0, 1, 2]), "{source}");
            }
        }
    }
    assert_eq!(
        run("typeof [{i: 0}, {i: 1, k: 1}].maxBy(item -> item.k)"),
        "undefined"
    );
    assert_eq!(
        run("[{i: 0, k: \"abc\"}, {i: 1, k: \"\"}].maxBy(note).i"),
        0
    );
    assert_eq!(run("[{i: 0, k: [5]}, {i: 1, k: [\"7\"]}].maxBy(note).i"), 1);
}

#[test]
fn an_unknown_key_settles_a_declaration_to_null_until_it_is_known() {
    run(r#"
class Bid(bidder, amount):
    this.bidder = bidder
    this.amount = amount
ada = Bid("ada", 120)
bo = Bid("bo", 90)
leader = Bid.maxBy(bid -> bid.amount)
lowest = Bid.minBy(bid -> bid.amount)
assert(leader, ada)
assert(lowest, bo)
delete bo.amount
assert(leader, null)
assert(lowest, null)
bo.amount = 200
assert(leader, bo)
assert(lowest, ada)
ada.amount = 0 / 0
assert(leader, null)
ada.amount = 50
assert(lowest, ada)
readings = [{c: 3}, {c: 5}]
warmest = readings.maxBy(reading -> reading.c)
readings.push({t: 9})
assert(warmest, null)
readings.pop()
assert(warmest.c, 5)
"#);
}

#[test]
fn the_scan_equals_the_reduce_spelling_when_every_key_is_comparable() {
    let keys = [COMPARABLE, TEXT].concat();
    let mut program = Vec::new();
    for list in lists(77, &keys, 400, 0..=6) {
        program.push(format!(
            "assert: {}, {}",
            index_of(&format!("{list}.maxBy(x -> x.k)")),
            index_of(&format!(
                "{list}.reduce((best, item) -> item.k > best.k ? item : best)"
            ))
        ));
        program.push(format!(
            "assert: {}, {}",
            index_of(&format!("{list}.minBy(x -> x.k)")),
            index_of(&format!(
                "{list}.reduce((best, item) -> item.k < best.k ? item : best)"
            ))
        ));
    }
    run(&program.join("\n"));
}

#[test]
fn the_scan_equals_the_quantifier_search_when_keys_do_not_mix_strings() {
    let mut program = Vec::new();
    for (seed, keys) in [
        (7_u64, TEXT.to_vec()),
        (8, [COMPARABLE, UNORDERED].concat()),
    ] {
        for list in lists(seed, &keys, 300, 0..=5) {
            program.push(format!(
                "assert: {}, {}",
                index_of(&format!("{list}.maxBy(x -> x.k)")),
                index_of(&format!("{list}.find(x -> all y in {list}: x.k >= y.k)"))
            ));
            program.push(format!(
                "assert: {}, {}",
                index_of(&format!("{list}.minBy(x -> x.k)")),
                index_of(&format!("{list}.find(x -> all y in {list}: x.k <= y.k)"))
            ));
        }
    }
    run(&program.join("\n"));
}

#[test]
fn keys_that_are_not_strings_select_the_item_whose_key_is_the_max_or_min() {
    let keys = [COMPARABLE, UNORDERED].concat();
    let mut program = Vec::new();
    for list in lists(9, &keys, 400, 1..=5) {
        for (by, plain) in [("maxBy", "max"), ("minBy", "min")] {
            program.push(format!(
                "assert: {}, Number.isNaN({list}.{plain}(x -> x.k)) ? -1 : {list}.findIndex(x -> Number(x.k) == {list}.{plain}(y -> y.k))",
                index_of(&format!("{list}.{by}(x -> x.k)"))
            ));
        }
    }
    run(&program.join("\n"));
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
            "values.maxBy(note)",
            json!(3),
            json!([[3, null, null], [1, null, null], [2, null, null]]),
        ),
        (
            "values.minBy(note)",
            json!(1),
            json!([[3, null, null], [1, null, null], [2, null, null]]),
        ),
        (
            "Item.maxBy(item -> note(item.size)).size",
            json!(7),
            json!([[5, null, null], [7, null, null]]),
        ),
        (
            "typeof [\"x\", {}, 2].minBy(note)",
            json!("undefined"),
            json!([["x", null, null], [{}, null, null], [2, null, null]]),
        ),
        ("typeof [].maxBy(note)", json!("undefined"), json!([])),
        ("typeof [].minBy(note)", json!("undefined"), json!([])),
    ] {
        run("log = []");
        assert_eq!(run(query), extreme, "{query}");
        assert_eq!(run("log"), visits, "{query}");
    }
    run("log = []\nItem(1)");
    assert_eq!(run("Item.minBy(item -> note(item.size)).size"), 1);
    assert_eq!(
        run("log"),
        json!([[5, null, null], [7, null, null], [1, null, null]])
    );
    assert_eq!(run("[1, 2].maxBy((item, rest) -> rest ?? item)"), 2);
}

#[test]
fn a_selector_is_required() {
    let (mut run, mut fail) = common::runners();
    run("class Empty\nclass Item\nItem()\nlog = []\nnothing = null\nholder = {}");
    for query in ["maxBy", "minBy"] {
        for receiver in ["[]", "[1, 2]", "Empty", "Item"] {
            for argument in [
                "",
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
log = []
"#);
    assert_eq!(run("source().maxBy(pick(), extra())"), 3);
    assert_eq!(run("calls"), json!(["receiver", "selector", "extra"]));
    run("calls = []");
    assert_eq!(run("source().minBy(pick(), extra())"), 1);
    assert_eq!(run("calls"), json!(["receiver", "selector", "extra"]));
    assert_eq!(run("Item.maxBy(grow(9)).size"), 1);
    assert_eq!(run("Item.length"), 2);
    assert_eq!(run("Item.minBy(grow(-9)).size"), 1);
    assert_eq!(run("Item.length"), 3);
    assert_eq!(
        run("[1, 2].maxBy(value -> value, log.push(\"a\"), log.push(\"b\"))"),
        2
    );
    assert_eq!(run("log"), json!(["a", "b"]));
    assert_eq!(run("[1, 2].maxBy(value -> value, 99)"), 2);
    assert_eq!(run("[1, 2].minBy(value -> value, -99)"), 1);
    assert_eq!(run("[1, 2].maxBy(value -> -value, 5)"), 1);
    assert_eq!(run("typeof [].maxBy(value -> value, 5)"), "undefined");
    assert_eq!(run("typeof [].minBy(value -> value, [1])"), "undefined");
}

#[test]
fn extreme_items_depend_on_the_receiver_the_arguments_and_the_selector_reads() {
    let mut runtime = run(r#"
values = [1, 2]
top = values.maxBy(value -> value)
frozen = values.value.minBy(value -> value)
class Order(amount):
    this.amount = amount
a = Order(1)
b = Order(2)
highest = Order.maxBy(order -> order.amount)
rate = 2
def scaled order => order.amount * rate
lowest = Order.minBy(scaled)
assert(top, 2)
assert(highest, b)
assert(lowest, a)
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
        "values.push(3)\nassert(top, 3)\nassert(frozen, 1)\nc = Order(3)\nassert(highest, c)\n\
         rate = -1\nassert(lowest, c)\na.amount = 5\nassert(highest, a)\nassert(lowest, a)",
    );
    assert_eq!(
        variable_dependencies(&runtime, "highest"),
        ["$Order", "a.amount", "b.amount", "c.amount"]
    );
}

#[test]
fn populations_gaining_and_losing_instances_re_evaluate_extreme_items() {
    run(r#"
class Visit(room, minutes):
    this.room = room
    this.minutes = minutes
longest = Visit.maxBy(visit -> visit.minutes)
shortest = Visit.minBy(visit -> visit.minutes)
assert(longest, null)
assert(shortest, null)
first = Visit("lab", 10)
second = Visit("den", 20)
assert(longest, second)
assert(shortest, first)
third = Visit("hall", 5)
assert(shortest, third)
delete second.minutes
assert(longest, null)
delete second.room
delete second
assert(longest, first)
assert(shortest, third)
assert(Visit.length, 2)
delete third.minutes
delete third.room
delete third
delete first.minutes
delete first.room
delete first
assert(longest, null)
assert(shortest, null)
class Empty
none = Empty.maxBy(empty -> empty.size)
assert(none, null)
lone = Empty()
assert(none, null)
"#);
}

#[test]
fn class_level_rules_take_extreme_items_for_each_instance() {
    let mut runtime = run(r#"
class Player(team, name, score):
    this.team = team
    this.name = name
    this.score = score
class Team(name, target):
    this.name = name
    this.target = target
$Team.best = Player.filter(player -> player.team == $Team).maxBy(player -> player.score)
$Team.fit = Player.minBy(player -> Math.abs(player.score - $Team.target))
red = Team("red", 2)
blue = Team("blue", 9)
Player(red, "ann", 3)
ana = Player(blue, "ana", 4)
assert(red.best.name, "ann")
assert(blue.best, ana)
assert(red.fit.name, "ann")
assert(blue.fit, ana)
ana.score = 8
assert(blue.best, ana)
assert(blue.fit, ana)
Player(red, "cy", 9)
assert(red.best.name, "cy")
assert(blue.fit.name, "cy")
"#);
    let best = dependencies(
        &runtime,
        &NodeKey::property(&ObjectId::named("blue"), "best"),
    );
    for key in ["$Player", "ana.score"] {
        assert!(best.contains(&key), "{key} in {best:?}");
    }
    let fit = dependencies(
        &runtime,
        &NodeKey::property(&ObjectId::named("blue"), "fit"),
    );
    for key in ["$Player", "ana.score", "blue.target"] {
        assert!(fit.contains(&key), "{key} in {fit:?}");
    }
    checked(
        &mut runtime,
        "green = Team(\"green\", 7)\nassert(green.best, null)\nassert(green.fit, ana)",
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
report = {maxBy: 5, minBy: 1}
holder = {maxBy: value -> value * 2}
empty = {}
maxBy = 3
minBy = 1
class Review
$Review.rate = $Review.maxBy / 10
review = Review()
review.maxBy = 42
def twice maxBy => maxBy * 2
def half minBy => minBy / 2
nothing = null
count = 5
flag = true
when = Date(5)
pattern = /a/
def helper(value) => value
"#);
    for (source, expected) in [
        ("values.maxBy", json!(null)),
        ("values.minBy", json!(null)),
        ("typeof values.maxBy", json!("undefined")),
        ("typeof values.minBy", json!("undefined")),
        ("Order.maxBy", json!(null)),
        ("typeof Order.minBy", json!("undefined")),
        ("report.maxBy", json!(5)),
        ("report.minBy", json!(1)),
        ("holder.maxBy(4)", json!(8)),
        ("empty.maxBy()", json!(null)),
        ("typeof empty.minBy()", json!("undefined")),
        ("maxBy", json!(3)),
        ("minBy", json!(1)),
        ("review.maxBy", json!(42)),
        ("review.rate", json!(4.2)),
        ("twice(4)", json!(8)),
        ("half(4)", json!(2)),
        ("nothing.maxBy(x -> x)", json!(null)),
        ("nothing.minBy(x -> x)", json!(null)),
        ("values.maxBy(x -> x) + maxBy", json!(5)),
        ("values.reduce((a, b) -> b > a ? b : a)", json!(2)),
        ("values.max()", json!(2)),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    for (source, message) in [
        (
            "\"abc\".maxBy(x -> x)",
            "TypeError: abc.maxBy is not a function",
        ),
        (
            "count.minBy(x -> x)",
            "TypeError: number.minBy is not a function",
        ),
        (
            "flag.maxBy(x -> x)",
            "TypeError: boolean.maxBy is not a function",
        ),
        (
            "when.minBy(x -> x)",
            "TypeError: Date.minBy is not a function",
        ),
        (
            "pattern.maxBy(x -> x)",
            "TypeError: maxBy is not a function",
        ),
        (
            "helper.minBy(x -> x)",
            "TypeError: function.minBy is not a function",
        ),
        ("Math.maxBy([1])", "TypeError: Math.maxBy is not a function"),
        (
            "Class.maxBy(x -> x)",
            "ReferenceError: Class is not defined",
        ),
        ("List.minBy(x -> x)", "ReferenceError: List is not defined"),
        (
            "values.maxBy(missing)",
            "ReferenceError: missing is not defined",
        ),
        ("maxBy of values", "SyntaxError: Unexpected keyword 'of'"),
    ] {
        assert_eq!(fail(source), message, "{source}");
    }
    let (mut defined, mut missing) = common::runners();
    assert_eq!(
        missing("maxBy([1, 2])"),
        "ReferenceError: maxBy is not defined"
    );
    assert_eq!(
        missing("minBy([1, 2])"),
        "ReferenceError: minBy is not defined"
    );
    defined("def maxBy(left, right) => left > right ? left : right");
    assert_eq!(defined("maxBy(1, 3)"), 3);
    assert_eq!(defined("[1, 3, 2].maxBy(x -> -x)"), 1);
    assert_eq!(defined("[1, 3, 2].minBy(x -> -x)"), 3);
}

#[test]
fn compatibility_rows_give_the_documented_results() {
    let orders = "class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)";
    let bag = "class Bag:\n    def maxBy(f): return 9\n    def minBy(f): return 1\nbag = Bag()";
    let kind = format!("{orders}\nkind = Order");
    for (setup, source, expected) in [
        ("", "[1, 3, 2].maxBy(x -> x)", json!(3)),
        ("", "typeof [].minBy(x -> x)", json!("undefined")),
        (
            "xs = [{s: 1}, {s: 3}, {s: 3}]",
            "xs.maxBy(x -> x.s) == xs[1]",
            json!(true),
        ),
        ("xs = [3, 1, 2]", "xs.minBy(x -> x)", json!(1)),
        (
            "o = {items: [{n: \"b\"}, {n: \"a\"}]}",
            "o.items.minBy(i -> i.n).n",
            json!("a"),
        ),
        ("xs = [3, 4]", "xs.map(x -> x * 2).maxBy(x -> -x)", json!(6)),
        ("xs = [1, 2, 3]", "xs.value.maxBy(x -> x)", json!(3)),
        ("xs = [1]\nxs.push(2)", "xs@first.maxBy(x -> x)", json!(1)),
        (orders, "Order.maxBy(o -> o.total).total", json!(7)),
        (orders, "Order.minBy(o -> o.total).total", json!(5)),
        (orders, "Order[1:].minBy(o -> o.total).total", json!(7)),
        (orders, "Order.list().maxBy(o -> o.total).total", json!(7)),
        (kind.as_str(), "kind.minBy(o -> o.total).total", json!(5)),
        (
            "class Empty",
            "typeof Empty.maxBy(e -> e)",
            json!("undefined"),
        ),
        (bag, "Bag.maxBy(b -> b.maxBy(1)) == bag", json!(true)),
        ("", "typeof [1, 0 / 0, 3].maxBy(x -> x)", json!("undefined")),
        (
            "xs = [1, 2]\nr = 0\ntry:\n    r = xs.maxBy(x -> x)\ncatch e:\n    r = e",
            "r",
            json!(2),
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
            "xs.maxBy(5)",
            "TypeError: maxBy expects a function",
        ),
        (
            "xs = [1, 2]",
            "xs.maxBy()",
            "TypeError: maxBy expects a function",
        ),
        ("", "[].minBy(null)", "TypeError: minBy expects a function"),
        (
            orders,
            "Order.maxBy(\"total\")",
            "TypeError: maxBy expects a function",
        ),
    ] {
        let (mut run, mut fail) = common::runners();
        if !setup.is_empty() {
            run(setup);
        }
        assert_eq!(fail(source), message, "{setup}\n{source}");
    }
    let (mut run, _) = common::runners();
    run("x = {maxBy: 4}.maxBy\ny = {minBy: 1}.minBy");
    assert_eq!(run("[x, y]"), json!([4, 1]));
}

#[test]
fn class_names_run_the_population_queries_beside_instance_methods() {
    let (mut run, _) = common::runners();
    run(
        "class Bag(size):\n    this.size = size\n    def maxBy(f): return 9\n    def minBy(f): return 1\nfirst = Bag(2)\nsecond = Bag(5)",
    );
    assert_eq!(run("first.maxBy(1)"), 9);
    assert_eq!(run("second.minBy(1)"), 1);
    assert_eq!(run("Bag.maxBy(bag -> bag.size) == second"), true);
    assert_eq!(run("Bag.minBy(bag -> bag.size) == first"), true);
    assert_eq!(run("Bag.maxBy(bag -> bag.maxBy(0)) == first"), true);
    assert_eq!(run("Bag.length"), 2);
}

#[test]
fn rendering_and_reasoning_keep_the_ordinary_call() {
    let (mut run, _) = common::runners();
    run(
        "values = [1, 2]\ntop = values.maxBy(value -> value)\nclass Order(amount):\n    this.amount = amount\nOrder(1)\nlowest = Order.minBy(order -> order.amount)",
    );
    assert_eq!(run("(why top)[0].rule"), "top = values.maxBy(function())");
    assert_eq!(run("(why top)[0].holds"), 2);
    assert_eq!(
        run("(why lowest)[0].rule"),
        "lowest = Order.minBy(function())"
    );
    assert_eq!(run("(why lowest)[0].state"), "derived");
    assert_eq!(run("(affects values)[0].node"), "top");
}

#[test]
fn failed_extreme_item_updates_restore_state_and_graph() {
    let initial = r#"
def check(value):
    if value > 100:
        throw "TOO_LARGE"
    return value
values = [1, 3]
top = values.maxBy(check)
class Item(size):
    this.size = size
small = Item(1)
smallest = Item.minBy(item -> check(item.size))
assert(top, 3)
assert(smallest, small)
"#;
    let mut actual = run(initial);
    let control = run(initial);
    for (source, thrown) in [
        ("values.push(500)", "TOO_LARGE"),
        ("big = Item(500)", "TOO_LARGE"),
        ("small.size = 700", "TOO_LARGE"),
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
        "kept = suppose values.push(50): top\nassert(kept, 50)\nassert(top, 3)\n\
         tiny = Item(0)\nassert(smallest, tiny)\nvalues.push(5)\nassert(top, 5)",
    );
}

#[test]
fn extreme_items_stay_linear_in_the_receiver() {
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
    assert_eq!(number(&mut runtime, "values.maxBy(note)"), 19_999.0);
    assert_eq!(number(&mut runtime, "counter.n"), f64::from(items));
    assert_eq!(number(&mut runtime, "values.minBy(note)"), 0.0);
    assert_eq!(number(&mut runtime, "counter.n"), f64::from(2 * items));
    runtime
        .run("class Cell(size): this.size = size\nvalues.map(value -> Cell(value)).length")
        .unwrap();
    assert_eq!(number(&mut runtime, "Cell.length"), f64::from(items));
    assert_eq!(
        number(&mut runtime, "Cell.maxBy(cell -> note(cell.size)).size"),
        19_999.0
    );
    assert_eq!(number(&mut runtime, "counter.n"), f64::from(3 * items));
    runtime.run("top = values.maxBy(value -> value)").unwrap();
    assert_eq!(number(&mut runtime, "top"), 19_999.0);
    assert_eq!(
        runtime.run("(why top)[0].rule").unwrap(),
        Value::String("top = values.maxBy(function())".to_string())
    );
}
