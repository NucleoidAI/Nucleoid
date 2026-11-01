mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::sync::Arc;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::parse;
use nucleoid::{ObjectId, Runtime, Value};
use serde_json::json;

const ACCUMULATOR: &str = "def kept list => list.reduce(lambda found, item: found.includes(item) ? found : found.concat([item]), [])";

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "list distinct regression".to_string(),
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

fn pair(runtime: &mut Runtime, source: &str) -> (Vec<Value>, Vec<Value>) {
    match value(runtime, source) {
        Value::List(mut both) if both.len() == 2 => {
            let second = both.pop().unwrap();
            let first = both.pop().unwrap();
            match (first, second) {
                (Value::List(first), Value::List(second)) => (first, second),
                other => panic!("{source} gave {other:?}"),
            }
        }
        other => panic!("{source} gave {other:?}"),
    }
}

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid removes duplicate list items with distinct",
    "Nucleoid lists the distinct values of class instance populations",
    "Nucleoid compares distinct values with deep equality",
    "Nucleoid calls the distinct selector once per item with only the item",
    "Nucleoid keeps distinct values live in class level rules",
    "Nucleoid uses distinct values in functions methods and conditional expressions",
    "Nucleoid keeps distinct as an ordinary name beside the distinct query",
    "Nucleoid rolls back failed distinct updates",
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
            "Nucleoid lists the distinct values of lists and populations",
        ),
        (
            &readme,
            "Distinct values keep the regions of orders live as orders change",
        ),
        (
            &examples,
            "Distinct values keep the categories stocked in each warehouse live",
        ),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(case.source.contains(".distinct("), "{title} uses distinct");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 88);
}

#[test]
fn distinct_accepts_every_receiver_the_list_queries_accept() {
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
        ("[1, 3, 1, 2, 3].distinct()", json!([1, 3, 2])),
        ("visits.distinct()", json!(["lab", "den", "hall"])),
        ("cart.items.distinct()", json!([2, 1])),
        (
            "visits.filter(visit -> visit != \"den\").distinct()",
            json!(["lab", "hall"]),
        ),
        (
            "visits.map(visit -> visit.length).distinct()",
            json!([3, 4]),
        ),
        ("visits.slice(1).distinct()", json!(["den", "lab", "hall"])),
        (
            "visits.concat([\"attic\", \"lab\"]).distinct()",
            json!(["lab", "den", "hall", "attic"]),
        ),
        ("visits[2:].distinct()", json!(["lab", "hall", "den"])),
        ("visits.value.distinct()", json!(["lab", "den", "hall"])),
        (
            "[[1, 2], [3]].flatMap(row -> row.concat([1])).distinct()",
            json!([1, 2, 3]),
        ),
        ("history@first.distinct()", json!([1])),
        ("history.distinct()", json!([1, 2])),
        ("[].distinct()", json!([])),
        ("visits.distinct(visit -> visit.length)", json!([3, 4])),
        (
            "Order.distinct(order -> order.region)",
            json!(["north", "south", "east"]),
        ),
        (
            "Order[1:].distinct(order -> order.region)",
            json!(["south", "north", "east"]),
        ),
        (
            "Order[:3].distinct(order -> order.region)",
            json!(["north", "south"]),
        ),
        (
            "Order.list().distinct(order -> order.region)",
            json!(["north", "south", "east"]),
        ),
        (
            "Order.filter(order -> order.region != \"south\").distinct(order -> order.region)",
            json!(["north", "east"]),
        ),
        (
            "kind.distinct(order -> order.region)",
            json!(["north", "south", "east"]),
        ),
        ("Order.distinct().length", json!(4)),
        ("Empty.distinct()", json!([])),
        ("Empty.distinct(empty -> empty.size)", json!([])),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
}

#[test]
fn distinct_returns_what_the_includes_accumulator_returns() {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
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
        "1.5",
        "Number.NaN",
        "Number.POSITIVE_INFINITY",
        "null",
        "holder.missing",
        "\"1\"",
        "\"a\"",
        "\"\"",
        "true",
        "false",
        "Date(0)",
        "Date(1)",
        "Robot",
        "Item",
        "ada",
        "twin",
        "shared",
        "helper",
        "pattern",
        "[]",
        "[1]",
        "[1, 2]",
        "[2, 1]",
        "[[1]]",
        "[Number.NaN]",
        "[null]",
        "[holder.missing]",
        "[-0]",
        "[0]",
        "[ada]",
        "[{a: 1}]",
        "{}",
        "{a: 1}",
        "{a: 1, b: 2}",
        "{b: 2, a: 1}",
        "{a: null}",
        "{a: holder.missing}",
        "{a: [1]}",
        "{a: {c: 1}}",
        "{a: {c: 2}}",
        "{a: ada}",
        "{a: twin}",
        "{k: null, m: 1}",
        "{k: holder.missing, m: 1, z: 5}",
        "{m: 1}",
        "{m: 1, q: holder.missing}",
        "{id: \"ada\", name: \"ada\"}",
    ];
    let mut runtime = Runtime::new();
    runtime
        .run(&format!(
            "{ACCUMULATOR}\nholder = {{}}\nclass Robot(name): this.name = name\nclass Item\nada = Robot(\"ada\")\ntwin = Robot(\"ada\")\nshared = {{a: 1}}\ndef helper(value) => value\npattern = /a/"
        ))
        .unwrap();
    let mut lists = vec![
        String::new(),
        "{k: null, m: 1}, {k: holder.missing, m: 1, z: 5}".to_string(),
        "{k: holder.missing, m: 1, z: 5}, {k: null, m: 1}".to_string(),
        "shared, shared, {a: 1}".to_string(),
        "-0, 0, -0".to_string(),
        "0, -0".to_string(),
        "holder.missing, null, holder.missing".to_string(),
        "null, holder.missing".to_string(),
        "Number.NaN, Number.NaN".to_string(),
        "Date(1), 1, Date(1)".to_string(),
        "1, Date(1)".to_string(),
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
        for (distinct, reduced) in [
            ("values.distinct()", "kept(values)"),
            (
                "values.distinct(item -> [item])",
                "kept(values.map(item -> [item]))",
            ),
        ] {
            let source = format!("[{distinct}, {reduced}]");
            let (left, right) = pair(&mut runtime, &source);
            assert!(
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(&right)
                        .all(|(left, right)| identical(left, right)),
                "[{items}]: {distinct} gave {left:?}, the accumulator {right:?}"
            );
        }
    }
}

#[test]
fn distinct_keeps_first_occurrences_in_receiver_order() {
    let mut runtime = run(r#"
visits = ["lab", "den", "lab", "hall", "den"]
rooms = visits.distinct()
assert(rooms, ["lab", "den", "hall"])
visits.shift()
assert(rooms, ["den", "lab", "hall"])
visits.pop()
assert(rooms, ["den", "lab", "hall"])
visits.pop()
assert(rooms, ["den", "lab"])
"#);
    let (mut repl, _) = common::runners();
    repl("holder = {}");
    assert_eq!(repl("String(1 / [-0, 0].distinct()[0])"), "-Infinity");
    assert_eq!(repl("String(1 / [0, -0].distinct()[0])"), "Infinity");
    assert_eq!(repl("[-0, 0, -0].distinct().length"), 1);
    assert_eq!(
        repl("typeof [holder.missing, null].distinct()[0]"),
        "undefined"
    );
    assert_eq!(repl("typeof [null, holder.missing].distinct()[0]"), "null");
    assert_eq!(repl("[holder.missing, null].distinct().length"), 1);
    checked(
        &mut runtime,
        "visits.push(\"attic\")\nassert(rooms, [\"den\", \"lab\", \"attic\"])",
    );
}

#[test]
fn values_are_compared_with_deep_equality_never_converted() {
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
"#);
    for (source, expected) in [
        (
            "[1, \"1\", true, 1, \"1\", true].distinct()",
            json!([1, "1", true]),
        ),
        ("[0, false, \"\", null, []].distinct().length", json!(5)),
        ("[\"10\", 10, \" 10 \"].distinct().length", json!(3)),
        ("[Date(5), 5, \"5\", Date(5)].distinct().length", json!(2)),
        ("[5, Date(5)].distinct()", json!([5])),
        ("[Robot, Robot, ada].distinct().length", json!(2)),
        (
            "[{x: 1, y: 2}, {y: 2, x: 1}, {x: 2, y: 1}].distinct()",
            json!([{"x": 1, "y": 2}, {"x": 2, "y": 1}]),
        ),
        ("[{x: 1}, {x: 1, y: 2}, {x: 1}].distinct().length", json!(2)),
        ("[{x: null}, {x: null}, {}].distinct().length", json!(2)),
        (
            "[[1, 2], [1, 2], [2, 1], [1, [2]], [1, [2]]].distinct()",
            json!([[1, 2], [2, 1], [1, [2]]]),
        ),
        ("[[ada], [twin], [ada]].distinct().length", json!(2)),
        ("[ada, twin, ada].distinct().length", json!(2)),
        ("[shared, shared, {k: 1}].distinct().length", json!(1)),
        (
            "[ada, {id: \"ada\", name: \"ada\"}].distinct().length",
            json!(1),
        ),
        ("[helper, helper].distinct().length", json!(2)),
        ("[pattern, pattern].distinct().length", json!(2)),
        ("[Number.NaN, Number.NaN, 1].distinct().length", json!(3)),
        ("[[Number.NaN], [Number.NaN]].distinct().length", json!(2)),
        ("[a, b].distinct().length", json!(1)),
        ("[b, a].distinct().length", json!(2)),
        ("[a].includes(b)", json!(true)),
        ("[b].includes(a)", json!(false)),
        (
            "[{m: 1}, {m: 1, q: holder.missing}].distinct().length",
            json!(1),
        ),
        (
            "[{m: 1, q: holder.missing}, {m: 1}].distinct().length",
            json!(1),
        ),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    assert_eq!(run("String([Number.NaN, Number.NaN].distinct()[1])"), "NaN");
    let mut runtime = Runtime::new();
    runtime
        .run("class Robot(name): this.name = name\nada = Robot(\"ada\")\ntwin = Robot(\"ada\")\nshared = {k: 1}")
        .unwrap();
    for (source, kept) in [
        ("[ada, twin, ada].distinct()", vec!["ada", "twin"]),
        ("[shared, {k: 1}, shared].distinct()", vec!["shared"]),
        ("[twin, ada].distinct(robot -> robot)", vec!["twin", "ada"]),
    ] {
        let expected: Vec<Value> = kept.iter().map(|name| value(&mut runtime, name)).collect();
        match value(&mut runtime, source) {
            Value::List(items) => assert!(
                items.len() == expected.len()
                    && items
                        .iter()
                        .zip(&expected)
                        .all(|(item, expected)| identical(item, expected)),
                "{source} gave {items:?}"
            ),
            other => panic!("{source} gave {other:?}"),
        }
    }
}

#[test]
fn null_items_are_kept_while_reads_inside_a_selector_settle() {
    let mut runtime = run(r#"
values = [1, null, 1]
plain = values.distinct()
selected = values.distinct(value -> value)
reduced = values.reduce((found, value) -> found.includes(value) ? found : found.concat([value]), [])
class Item(price):
    this.price = price
pen = Item(3)
gift = Item(null)
prices = Item.distinct(item -> item.price)
mapped = Item.map(item -> item.price)
assert(plain, [1, null])
assert(selected, null)
assert(reduced, null)
assert(prices, null)
assert(mapped, null)
"#);
    checked(
        &mut runtime,
        "gift.price = 5\nassert(prices, [3, 5])\nvalues.push(null)\nvalues.push(2)\nassert(plain, [1, null, 2])",
    );
}

#[test]
fn undefined_values_are_kept_without_an_undefined_total() {
    let mut runtime = run(r#"
values = [1]
values.push(values[9])
values.push(1)
values.push(null)
kept = values.distinct()
class Item
pen = Item()
pen.price = 3
book = Item()
prices = Item.distinct(item -> item.price)
assert(kept, [1, null])
assert(prices, [3, null])
"#);
    let (mut repl, _) = common::runners();
    repl("values = [1]\nvalues.push(values[9])\nholder = {}");
    assert_eq!(repl("typeof values.distinct()[1]"), "undefined");
    assert_eq!(repl("values.distinct().length"), 2);
    assert_eq!(repl("[1, 2].distinct(item -> holder.missing).length"), 1);
    assert_eq!(
        repl("typeof [1, 2].distinct(item -> holder.missing)[0]"),
        "undefined"
    );
    assert_eq!(repl("[].distinct(item -> holder.missing)"), json!([]));
    checked(
        &mut runtime,
        "book.price = 12\nassert(prices, [3, 12])\nbook.price = 3\nassert(prices, [3])",
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
values = [3, 1, 3]
class Item(size):
    this.size = size
Item(5)
Item(7)
Item(5)
"#);
    for (query, result, visits) in [
        (
            "values.distinct(note)",
            json!([3, 1]),
            json!([[3, null, null], [1, null, null], [3, null, null]]),
        ),
        (
            "Item.distinct(item -> note(item.size))",
            json!([5, 7]),
            json!([[5, null, null], [7, null, null], [5, null, null]]),
        ),
        ("[].distinct(note)", json!([]), json!([])),
        (
            "values.distinct(lambda first, second: second)",
            json!([null]),
            json!([]),
        ),
    ] {
        run("log = []");
        assert_eq!(run(query), result, "{query}");
        assert_eq!(run("log"), visits, "{query}");
    }
    run("log = []\nItem(1)");
    assert_eq!(
        run("Item.distinct(item -> note(item.size))"),
        json!([5, 7, 1])
    );
    assert_eq!(run("log.length"), 4);
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
            let source = format!("{receiver}.distinct({argument})");
            assert_eq!(
                fail(&source),
                "TypeError: distinct expects a function",
                "{source}"
            );
        }
    }
    for source in [
        "[1].distinct(5, missing)",
        "[1].distinct(missing, 5)",
        "Item.distinct(missing)",
    ] {
        assert_eq!(
            fail(source),
            "ReferenceError: missing is not defined",
            "{source}"
        );
    }
    assert_eq!(run("nothing.distinct(log.push(1))"), json!(null));
    assert_eq!(run("typeof nothing.distinct()"), "undefined");
    assert_eq!(run("typeof holder.missing.distinct()"), "undefined");
    assert_eq!(run("log"), json!([]));
    assert_eq!(
        run("[1, 2, 1].distinct(value -> value, log.push(\"a\"), log.push(\"b\"))"),
        json!([1, 2])
    );
    assert_eq!(run("log"), json!(["a", "b"]));
    assert_eq!(run("[1, 2].distinct(value -> value, 1)"), json!([1, 2]));
    assert_eq!(run("[].distinct(value -> value, 5)"), json!([]));
}

#[test]
fn the_receiver_selector_and_extras_are_evaluated_once_in_order() {
    let (mut run, _) = common::runners();
    run(r#"
calls = []
def source():
    calls.push("receiver")
    return [1, 2, 1]
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
    assert_eq!(run("source().distinct(pick(), extra())"), json!([10, 20]));
    assert_eq!(run("calls"), json!(["receiver", "selector", "extra"]));
    assert_eq!(run("Item.distinct(grow(9))"), json!([1]));
    assert_eq!(run("Item.length"), 2);
    assert_eq!(run("Item.distinct(grow(1))"), json!([1, 9]));
    assert_eq!(run("Item.length"), 3);
}

#[test]
fn distinct_depends_on_the_receiver_the_arguments_and_the_selector_reads() {
    let mut runtime = run(r#"
values = [1, 2, 1]
kept = values.distinct()
frozen = values.value.distinct()
class Order(region):
    this.region = region
a = Order("north")
b = Order("south")
regions = Order.distinct(order -> order.region)
suffix = "!"
def label order => order.region + suffix
labels = Order.distinct(label)
assert(kept, [1, 2])
assert(regions, ["north", "south"])
assert(labels, ["north!", "south!"])
"#);
    assert_eq!(variable_dependencies(&runtime, "kept"), ["values"]);
    assert!(variable_dependencies(&runtime, "frozen").is_empty());
    assert_eq!(
        variable_dependencies(&runtime, "regions"),
        ["$Order", "a.region", "b.region"]
    );
    assert_eq!(
        variable_dependencies(&runtime, "labels"),
        ["$Order", "label", "a.region", "suffix", "b.region"]
    );
    checked(
        &mut runtime,
        "values.push(3)\nassert(kept, [1, 2, 3])\nassert(frozen, [1, 2])\nc = Order(\"north\")\n\
         assert(regions, [\"north\", \"south\"])\nsuffix = \"?\"\nassert(labels, [\"north?\", \"south?\"])\n\
         b.region = \"north\"\nassert(regions, [\"north\"])\nassert(labels, [\"north?\"])",
    );
    assert_eq!(
        variable_dependencies(&runtime, "regions"),
        ["$Order", "a.region", "b.region", "c.region"]
    );
}

#[test]
fn comparisons_do_not_subscribe_to_the_compared_objects() {
    let mut runtime = run(r#"
first = {k: 1}
second = {k: 2}
records = [first, second]
kept = records.distinct()
assert(kept.length, 2)
first.k = 2
assert(kept.length, 2)
"#);
    assert_eq!(variable_dependencies(&runtime, "kept"), ["records"]);
    assert_eq!(
        value(&mut runtime, "records.distinct().length"),
        Value::Number(1.0)
    );
    checked(&mut runtime, "records.push(first)\nassert(kept.length, 1)");
}

#[test]
fn populations_gaining_and_losing_instances_re_evaluate_distinct_values() {
    run(r#"
class Visit(room):
    this.room = room
rooms = Visit.distinct(visit -> visit.room)
assert(rooms, [])
first = Visit("lab")
second = Visit("den")
third = Visit("lab")
assert(rooms, ["lab", "den"])
delete first.room
assert(rooms, [null, "den", "lab"])
delete first
assert(rooms, ["den", "lab"])
delete second.room
delete second
assert(rooms, ["lab"])
assert(Visit.length, 1)
delete third.room
delete third
assert(rooms, [])
Visit("hall")
assert(rooms, ["hall"])
class Empty
none = Empty.distinct(empty -> empty.size)
assert(none, [])
Empty()
assert(none, [null])
"#);
}

#[test]
fn class_level_rules_list_distinct_values_for_each_instance() {
    let mut runtime = run(r#"
class Player(team, role):
    this.team = team
    this.role = role
class Team(name, tags):
    this.name = name
    this.tags = tags
$Team.roles = Player.filter(player -> player.team == $Team).distinct(player -> player.role)
$Team.labels = $Team.tags.distinct()
red = Team("red", ["fast", "fast"])
blue = Team("blue", [])
Player(red, "guard")
ana = Player(blue, "wing")
Player(red, "guard")
assert(red.roles, ["guard"])
assert(blue.roles, ["wing"])
assert(red.labels, ["fast"])
assert(blue.labels, [])
ana.role = "center"
blue.tags.push("tall")
blue.tags.push("tall")
assert(blue.roles, ["center"])
assert(blue.labels, ["tall"])
Player(red, "wing")
assert(red.roles, ["guard", "wing"])
"#);
    let roles = dependencies(
        &runtime,
        &NodeKey::property(&ObjectId::named("blue"), "roles"),
    );
    for key in ["$Player", "ana.role"] {
        assert!(roles.contains(&key), "{key} in {roles:?}");
    }
    let labels = dependencies(
        &runtime,
        &NodeKey::property(&ObjectId::named("blue"), "labels"),
    );
    assert!(labels.contains(&"blue.tags"), "blue.tags in {labels:?}");
    checked(
        &mut runtime,
        "green = Team(\"green\", [\"new\", \"new\", \"old\"])\nassert(green.roles, [])\nassert(green.labels, [\"new\", \"old\"])",
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
report = {distinct: 5}
holder = {distinct: value -> value * 2}
empty = {}
distinct = 3
class Review
$Review.rate = $Review.distinct / 10
review = Review()
review.distinct = 42
def twice distinct => distinct * 2
nothing = null
count = 5
flag = true
when = Date(5)
pattern = /a/
def helper(value) => value
"#);
    for (source, expected) in [
        ("values.distinct", json!(null)),
        ("typeof values.distinct", json!("undefined")),
        ("Order.distinct", json!(null)),
        ("typeof Order.distinct", json!("undefined")),
        ("report.distinct", json!(5)),
        ("holder.distinct(4)", json!(8)),
        ("empty.distinct()", json!(null)),
        ("typeof empty.distinct()", json!("undefined")),
        ("distinct", json!(3)),
        ("review.distinct", json!(42)),
        ("review.rate", json!(4.2)),
        ("twice(4)", json!(8)),
        ("nothing.distinct()", json!(null)),
        ("values.distinct().length + distinct", json!(5)),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    run("x = {distinct: 4}.distinct");
    assert_eq!(run("x"), 4);
    for (source, message) in [
        (
            "\"abc\".distinct()",
            "TypeError: abc.distinct is not a function",
        ),
        (
            "count.distinct()",
            "TypeError: number.distinct is not a function",
        ),
        (
            "flag.distinct()",
            "TypeError: boolean.distinct is not a function",
        ),
        (
            "when.distinct()",
            "TypeError: Date.distinct is not a function",
        ),
        (
            "pattern.distinct()",
            "TypeError: distinct is not a function",
        ),
        (
            "helper.distinct()",
            "TypeError: function.distinct is not a function",
        ),
        (
            "Math.distinct([1])",
            "TypeError: Math.distinct is not a function",
        ),
        ("Class.distinct()", "ReferenceError: Class is not defined"),
        ("List.distinct()", "ReferenceError: List is not defined"),
        (
            "values.distinct(missing)",
            "ReferenceError: missing is not defined",
        ),
    ] {
        assert_eq!(fail(source), message, "{source}");
    }
    let (mut defined, mut missing) = common::runners();
    assert_eq!(
        missing("distinct([1, 2])"),
        "ReferenceError: distinct is not defined"
    );
    defined("def distinct(items) => items.length");
    assert_eq!(defined("distinct([1, 1])"), 2);
    assert_eq!(defined("[1, 1].distinct()"), json!([1]));
}

#[test]
fn compatibility_rows_give_the_documented_results() {
    let orders =
        "class Order(region):\n    this.region = region\nOrder(\"n\")\nOrder(\"s\")\nOrder(\"n\")";
    let shelf = "class Shelf:\n    def distinct(): return 9\nshelf = Shelf()";
    let kind = format!("{orders}\nkind = Order");
    for (setup, source, expected) in [
        ("", "[1, 2, 1].distinct()", json!([1, 2])),
        ("", "[].distinct()", json!([])),
        ("xs = [3, 1, 3]", "xs.distinct()", json!([3, 1])),
        ("xs = [1, 2, 3]", "xs.distinct(x -> x % 2)", json!([1, 0])),
        ("o = {items: [1, 1]}", "o.items.distinct()", json!([1])),
        (
            "xs = [3, 4]",
            "xs.map(x -> x % 2).distinct()",
            json!([1, 0]),
        ),
        ("xs = [1, 1, 2]", "xs.value.distinct()", json!([1, 2])),
        ("xs = [1]\nxs.push(1)", "xs@first.distinct()", json!([1])),
        (orders, "Order.distinct(o -> o.region)", json!(["n", "s"])),
        (
            orders,
            "Order[1:].distinct(o -> o.region)",
            json!(["s", "n"]),
        ),
        (
            orders,
            "Order.list().distinct(o -> o.region)",
            json!(["n", "s"]),
        ),
        (
            kind.as_str(),
            "kind.distinct(o -> o.region)",
            json!(["n", "s"]),
        ),
        ("class Empty", "Empty.distinct()", json!([])),
        (shelf, "Shelf.distinct()", json!([{"id": "shelf"}])),
        (shelf, "Shelf.distinct(s -> s.distinct())", json!([9])),
        (
            "",
            "try:\n    v = [1, 1].distinct()\ncatch e:\n    v = e\nv",
            json!([1]),
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
            "xs.distinct(5)",
            "TypeError: distinct expects a function",
        ),
        (
            "",
            "[].distinct(null)",
            "TypeError: distinct expects a function",
        ),
        (
            orders,
            "Order.distinct(\"region\")",
            "TypeError: distinct expects a function",
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
        "class Shelf(items):\n    this.items = items\n    def distinct(): return this.items.distinct().length\nfirst = Shelf([1, 1, 2])\nsecond = Shelf([3])",
    );
    assert_eq!(run("first.distinct()"), 2);
    assert_eq!(run("second.distinct()"), 1);
    assert_eq!(
        run("Shelf.distinct(shelf -> shelf.distinct())"),
        json!([2, 1])
    );
    assert_eq!(run("Shelf.distinct().length"), 2);
    assert_eq!(run("Shelf.length"), 2);
}

#[test]
fn rendering_and_reasoning_keep_the_ordinary_call() {
    let (mut run, _) = common::runners();
    run(
        "values = [1, 2, 1]\nkept = values.distinct()\nclass Order(region):\n    this.region = region\nOrder(\"north\")\nregions = Order.distinct(order -> order.region)",
    );
    assert_eq!(run("(why kept)[0].rule"), "kept = values.distinct()");
    assert_eq!(run("(why kept)[0].holds"), json!([1, 2]));
    assert_eq!(
        run("(why regions)[0].rule"),
        "regions = Order.distinct(function())"
    );
    assert_eq!(run("(why regions)[0].state"), "derived");
    assert_eq!(run("(why regions)[0].holds"), json!(["north"]));
}

#[test]
fn failed_distinct_updates_restore_state_order_and_graph() {
    let initial = r#"
def check(value):
    if value == "void":
        throw "BAD_ZONE"
    return value
zones = ["a", "b", "a"]
seen = zones.distinct(check)
class Parcel(zone):
    this.zone = zone
Parcel("b")
Parcel("a")
routes = Parcel.distinct(parcel -> check(parcel.zone))
assert(seen, ["a", "b"])
assert(routes, ["b", "a"])
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
        "assert(zones, [\"a\", \"b\", \"a\"])\nzones.shift()\nassert(seen, [\"b\", \"a\"])\nParcel(\"c\")\nassert(routes, [\"b\", \"a\", \"c\"])",
    );
}

#[test]
fn suppose_blocks_discard_distinct_changes_and_keep_order() {
    let mut runtime = run(r#"
zones = ["b", "a", "b", "c"]
seen = zones.distinct()
class Visit(room):
    this.room = room
lab = Visit("lab")
den = Visit("den")
Visit("lab")
rooms = Visit.distinct(visit -> visit.room)
suppose zones.shift(): assert(seen, ["a", "b", "c"])
suppose zones.sort(): assert(seen, ["a", "b", "c"])
suppose Visit("hall"): assert(rooms, ["lab", "den", "hall"])
suppose:
    delete den.room
    delete den
    assert(rooms, ["lab"])
moved = suppose lab.room = "hall": rooms
assert(moved, ["hall", "den", "lab"])
assert(zones, ["b", "a", "b", "c"])
assert(seen, ["b", "a", "c"])
assert(rooms, ["lab", "den"])
"#);
    checked(
        &mut runtime,
        "zones.shift()\nassert(seen, [\"a\", \"b\", \"c\"])\nVisit(\"hall\")\nassert(rooms, [\"lab\", \"den\", \"hall\"])",
    );
}

#[test]
fn large_receivers_keep_every_distinct_value_and_select_each_item_once() {
    let items = 20_000_u32;
    let list = (0..items)
        .map(|item| ((item * 7_919) % items).to_string())
        .collect::<Vec<_>>()
        .join(",\n");
    let mut runtime = Runtime::new();
    runtime
        .run(&format!(
            "values = [{list}]\ncounter = {{n: 0}}\ndef note(value):\n    counter.n = counter.n + 1\n    return value\ndef row(value):\n    return {{k: value, v: null, tags: [value % 3]}}\nrows = values.map(row)\nholder = {{}}\ngaps = values.map(value -> ({{k: value, gap: holder.missing}}))"
        ))
        .unwrap();
    for source in [
        "values.distinct().length",
        "values.map(value -> String(value)).distinct().length",
        "values.map(value -> [value, value % 7]).distinct().length",
        "rows.distinct().length",
        "gaps.distinct().length",
        "rows.concat(gaps).distinct().length - values.length",
        "values.distinct(note).length",
    ] {
        assert_eq!(
            value(&mut runtime, source),
            Value::Number(f64::from(items)),
            "{source}"
        );
    }
    assert_eq!(
        value(&mut runtime, "counter.n"),
        Value::Number(f64::from(items))
    );
    assert_eq!(
        value(&mut runtime, "values.distinct(value -> value % 10).length"),
        Value::Number(10.0)
    );
    runtime
        .run("kept = values.distinct(value -> value)")
        .unwrap();
    assert_eq!(
        runtime.run("kept.length").unwrap(),
        Value::Number(f64::from(items))
    );
    assert_eq!(
        runtime.run("(why kept)[0].rule").unwrap(),
        Value::String("kept = values.distinct(function())".to_string())
    );
}
