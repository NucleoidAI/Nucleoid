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
    "Nucleoid finds the middle values of lists with median",
    "Nucleoid finds the medians of class instance populations",
    "Nucleoid orders values as numbers to find medians",
    "Nucleoid calls the median selector once per item with only the item",
    "Nucleoid keeps medians live in class level rules",
    "Nucleoid uses medians in functions methods and conditional expressions",
    "Nucleoid keeps median as an ordinary name beside the median query",
    "Nucleoid rolls back failed median updates",
];
const REFERENCE_TITLE: &str = "Nucleoid finds the medians of lists and populations with median";
const README_TITLE: &str = "Medians keep typical delivery times live as orders arrive";
const EXAMPLES_TITLE: &str =
    "Medians keep the typical rent of every district live as listings change";
const COUNTERPART: &str =
    "def med v => v.sort(lambda a, b: a - b)[v.length / 2 - 0.5 : v.length / 2 + 1].average()";

type Failure = (ErrorKind, String, usize, usize);

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "list median regression".to_string(),
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

fn reference_median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap());
    let half = sorted.len() / 2;
    if sorted.is_empty() {
        f64::NAN
    } else if half * 2 == sorted.len() {
        (sorted[half - 1] + sorted[half]) / 2.0
    } else {
        sorted[half]
    }
}

fn depends_on_zero_order(values: &[f64]) -> bool {
    let mut negative_zeros_first = values.to_vec();
    negative_zeros_first.sort_by(f64::total_cmp);
    !same_number(
        reference_median(&negative_zeros_first),
        reference_median(values),
    )
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
        assert!(case.source.contains(".median("), "{title} uses median");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 38);
}

#[test]
fn the_language_reference_shows_the_reference_program() {
    let reference = include_str!("../docs/README.md");
    let (_, section) = reference
        .split_once("### List and population medians\n\n```\n")
        .expect("the language reference documents medians");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 84](nuc-0084.md)"));
    assert!(reference.contains("`median expects a function`"));
}

#[test]
fn medians_accept_every_receiver_the_list_queries_accept() {
    let (mut run, _) = common::runners();
    run(r#"
prices = [4, 10, 6]
shop = {stock: {prices: [9, 1, 5, 7]}}
groups = [[5, 1], [8, 2, 6]]
history = [1]
history.push(2)
history.push(9)
class Order(total):
    this.total = total
Order(5)
Order(12)
Order(7)
Order(30)
kind = Order
class Empty
"#);
    for (source, expected) in [
        ("[3, 1, 2].median()", 2.0),
        ("prices.median()", 6.0),
        ("shop.stock.prices.median()", 6.0),
        ("groups[1].median()", 6.0),
        ("groups[0].median()", 3.0),
        ("prices.filter(price -> price > 5).median()", 8.0),
        ("prices.map(price -> price * 2).median()", 12.0),
        ("prices.slice(1).median()", 8.0),
        ("prices.concat([1, 2]).median()", 4.0),
        ("prices[1:].median()", 8.0),
        ("prices.value.median()", 6.0),
        ("history@first.median()", 1.0),
        ("history@1.median()", 1.5),
        ("history.median()", 2.0),
        ("Order.median(order -> order.total)", 9.5),
        ("Order.median(.total)", 9.5),
        ("Order[1:].median(order -> order.total)", 12.0),
        ("Order[-3:].median(.total)", 12.0),
        ("Order[-2:].median(.total)", 18.5),
        ("Order.list().median(order -> order.total)", 9.5),
        (
            "Order.filter(order -> order.total > 6).median(order -> order.total)",
            12.0,
        ),
        ("kind.median(order -> order.total)", 9.5),
        ("kind.filter(order -> order.total < 10).median(.total)", 6.0),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    for source in [
        "String([].median())",
        "String(Empty.median())",
        "String(Empty.median(empty -> empty.size))",
        "String(prices.filter(price -> price > 50).median())",
        "String(Order.filter(order -> order.total > 99).median(.total))",
        "String(kind.filter(order -> order.total > 99).median(order -> order.total))",
    ] {
        assert_eq!(run(source), "NaN", "{source}");
    }
}

#[test]
fn numeric_medians_equal_a_reference_median_bit_for_bit() {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut lists: Vec<String> = vec![
        String::new(),
        "0.1, 0.2".to_string(),
        "0.1, 0.2, 0.3".to_string(),
        "-0".to_string(),
        "-0, -0".to_string(),
        "1, -1".to_string(),
        "9007199254740992, 1, 1".to_string(),
        "0.1, -0.1, 0.3".to_string(),
        "1e308, 1e308".to_string(),
        "1e308, 1.5e308, 1".to_string(),
        "5e-324, 5e-324".to_string(),
        "10, 9, 1".to_string(),
        "21, 9, -3, 7.5".to_string(),
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
    runtime.run(COUNTERPART).unwrap();
    for items in &lists {
        let values: Vec<f64> = if items.is_empty() {
            Vec::new()
        } else {
            items
                .split(", ")
                .map(|item| item.parse::<f64>().unwrap())
                .collect()
        };
        let median = number(&mut runtime, &format!("[{items}].median()"));
        let reference = reference_median(&values);
        assert!(
            same_number(median, reference),
            "[{items}]: {median:?} and {reference:?}"
        );
        let selected = number(&mut runtime, &format!("[{items}].median(item -> item * 3)"));
        let tripled: Vec<f64> = values.iter().map(|value| value * 3.0).collect();
        assert!(
            same_number(selected, reference_median(&tripled)),
            "[{items}] with a selector"
        );
        if !items.contains("-0") {
            let spelled = number(&mut runtime, &format!("med([{items}])"));
            assert!(
                same_number(median, spelled),
                "[{items}] against the accepted spelling: {spelled:?}"
            );
        }
    }
    assert_eq!(
        number(&mut runtime, "[0.1, 0.2].median()"),
        0.15000000000000002
    );
    assert_eq!(number(&mut runtime, "[10, 9, 1].median()"), 9.0);
    assert_eq!(number(&mut runtime, "[4, 1, 3, 2].median()"), 2.5);
    assert!(number(&mut runtime, "[].median()").is_nan());
}

#[test]
fn population_medians_equal_the_median_of_the_mapped_values() {
    let mut runtime = run(r#"
class Sample(amount):
    this.amount = amount
Sample(0.1)
Sample(0.2)
Sample(0.3)
Sample(-7.25)
Sample(1e15)
Sample(42)
"#);
    for selector in [
        "sample -> sample.amount",
        "sample -> sample.amount * sample.amount",
        ".amount",
    ] {
        let median = number(&mut runtime, &format!("Sample.median({selector})"));
        let mapped = number(&mut runtime, &format!("Sample.map({selector}).median()"));
        assert!(
            same_number(median, mapped),
            "{selector}: {median:?} and {mapped:?}"
        );
    }
    assert_eq!(number(&mut runtime, "Sample.median(.amount)"), 0.25);
}

#[test]
fn median_values_use_the_numeric_conversion_of_sum() {
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
            format!("[{value}].median()"),
            format!("[{value}].median(item -> item)"),
            format!("[{value}, {value}, {value}].median()"),
        ] {
            let median = number(&mut runtime, &source);
            assert!(same_number(median, expected), "{source}: {median:?}");
        }
        let pair = number(&mut runtime, &format!("[{value}, 0].median()"));
        assert!(
            same_number(pair, (expected + 0.0) / 2.0),
            "[{value}, 0]: {pair:?}"
        );
        let summed = number(&mut runtime, &format!("[{value}].sum()"));
        assert!(
            same_number(number(&mut runtime, &format!("[{value}].median()")), summed),
            "{value} converts as sum does"
        );
    }
    for (source, expected) in [
        ("[\"2\", \"10\", \"3\"].median()", 3.0),
        ("[true, true, false].median()", 1.0),
        ("[null, 1, 2].median()", 1.0),
        ("[\"\", [4], 9].median()", 4.0),
        ("[when, 2, 9].median()", 5.0),
    ] {
        let median = number(&mut runtime, source);
        assert!(same_number(median, expected), "{source}: {median:?}");
    }
}

#[test]
fn a_value_that_converts_to_nan_makes_the_median_nan() {
    let mut runtime = Runtime::new();
    for source in [
        "[1, \"x\", 3].median()",
        "[Number.NaN, 1, 2].median()",
        "[1, 2, 3, {}].median()",
        "[[1, 2], 5].median()",
        "[0 / 0].median()",
        "[1, 2].median(value -> value == 2 ? \"x\" : value)",
        "[Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY].median()",
    ] {
        assert!(number(&mut runtime, source).is_nan(), "{source}");
    }
    run(r#"
readings = [3, 9, 4]
middle = readings.median()
assert(middle, 4)
readings.push("broken")
assert(Number.isNaN(middle), true)
readings.pop()
assert(middle, 4)
"#);
}

#[test]
fn an_undefined_value_makes_the_median_undefined() {
    let mut runtime = run(r#"
values = [1]
values.push(values[9])
class Item
pen = Item()
pen.price = 3
book = Item()
middle = values.median()
spent = Item.median(item -> item.price)
assert(middle, null)
assert(spent, null)
"#);
    let (mut repl, _) = common::runners();
    repl("values = [1]\nvalues.push(values[9])\nholder = {}");
    assert_eq!(repl("typeof values.median()"), "undefined");
    assert_eq!(
        repl("typeof [1, 2].median(item -> holder.missing)"),
        "undefined"
    );
    assert_eq!(repl("typeof [].median(item -> holder.missing)"), "number");
    checked(
        &mut runtime,
        "book.price = 12\nassert(spent, 7.5)\nvalues.pop()\nassert(middle, 1)",
    );
}

#[test]
fn undefined_wins_over_nan_as_it_does_for_average() {
    let (mut repl, _) = common::runners();
    repl("values = [1]\nvalues.push(values[9])\nholder = {}");
    for receiver in [
        "[Number.NaN].concat(values)",
        "values.concat([Number.NaN])",
        "[\"x\"].concat(values)",
        "values.concat([{}, \"x\"])",
    ] {
        for query in ["median", "average"] {
            let source = format!("typeof {receiver}.{query}()");
            assert_eq!(repl(&source), "undefined", "{source}");
        }
    }
    for query in ["median", "average"] {
        let source =
            format!("typeof [\"x\", 2].{query}(value -> value == 2 ? holder.missing : value)");
        assert_eq!(repl(&source), "undefined", "{source}");
        let source =
            format!("typeof [2, \"x\"].{query}(value -> value == 2 ? holder.missing : value)");
        assert_eq!(repl(&source), "undefined", "{source}");
    }
    run(r#"
holder = {}
readings = [3, "broken", 4]
middle = readings.median()
assert(Number.isNaN(middle), true)
readings.push(holder.missing)
assert(middle, null)
readings.pop()
assert(Number.isNaN(middle), true)
"#);
}

#[test]
fn null_items_count_zero_while_reads_inside_a_selector_settle() {
    let mut runtime = run(r#"
values = [1, null, 2]
plain = values.median()
selected = values.median(value -> value)
class Item(price):
    this.price = price
pen = Item(3)
gift = Item(null)
spent = Item.median(item -> item.price)
assert(plain, 1)
assert(selected, null)
assert(spent, null)
"#);
    for source in ["values.median()", "values.median(value -> value)"] {
        assert_eq!(number(&mut runtime, source), 1.0, "{source}");
    }
    checked(
        &mut runtime,
        "gift.price = 5\nassert(spent, 4)\nvalues.push(null)\nassert(plain, 0.5)",
    );
}

#[test]
fn empty_receivers_give_nan_rather_than_null() {
    let mut runtime = run(r#"
class Empty
class Order(total):
    this.total = total
Order(5)
empty = []
middle = empty.median()
none = Empty.median(item -> item.size)
filtered = [1, 2].filter(value -> value > 5).median()
unmatched = Order.filter(order -> order.total > 99).median(order -> order.total)
assert(Number.isNaN(middle), true)
assert(middle == null, false)
assert(Number.isNaN(none), true)
assert(Number.isNaN(filtered), true)
assert(Number.isNaN(unmatched), true)
assert(typeof middle, "number")
"#);
    let (mut repl, _) = common::runners();
    repl("log = []\ndef note(value):\n    log.push(value)\n    return value");
    assert_eq!(repl("String([].median(note))"), "NaN");
    assert_eq!(repl("log"), json!([]));
    assert_eq!(repl("typeof [].median()"), "number");
    checked(
        &mut runtime,
        "empty.push(4)\nassert(middle, 4)\nOrder(120)\nassert(unmatched, 120)\nEmpty()\nassert(none, null)",
    );
}

#[test]
fn negative_zero_keeps_its_sign_in_a_stable_order() {
    let mut runtime =
        run("class Zero(amount):\n    this.amount = amount\nZero(-0)\nZero(1)\nZero(-1)");
    for (source, negative) in [
        ("[-0].median()", true),
        ("[0].median()", false),
        ("[0, -0, -0].median()", true),
        ("[0, -0, 1].median()", true),
        ("[-0, 0, 1].median()", false),
        ("[1, 0, -0].median()", true),
        ("[1, -0, 0].median()", false),
        ("[-0, -0].median()", true),
        ("[-0, 0].median()", false),
        ("[0, -0].median()", false),
        ("[0, 1, -0].median(value -> -value)", true),
        ("Zero.median(zero -> zero.amount)", true),
    ] {
        let median = number(&mut runtime, source);
        assert_eq!(median, 0.0, "{source}");
        assert_eq!(median.is_sign_negative(), negative, "{source}");
    }
    assert_eq!(
        number(&mut runtime, "1 / [-0, -0].median()"),
        f64::NEG_INFINITY
    );
    assert_eq!(
        number(&mut runtime, "1 / [-0, -0].average()"),
        f64::INFINITY
    );
    assert_eq!(number(&mut runtime, "1 / [-0].average()"), f64::INFINITY);
    assert_eq!(number(&mut runtime, "1 / [-0].median()"), f64::NEG_INFINITY);
}

#[test]
fn zeros_keep_their_receiver_order_in_long_receivers() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut generate = move |length: usize| -> Vec<&'static str> {
        (0..length)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                ["-0", "0", "-0", "0", "1", "-1", "2", "-2"][(state % 8) as usize]
            })
            .collect()
    };
    let to_numbers =
        |items: &[&str]| -> Vec<f64> { items.iter().map(|item| item.parse().unwrap()).collect() };
    let negate = |values: &[f64]| -> Vec<f64> { values.iter().map(|value| -value).collect() };
    let mut runtime = Runtime::new();
    let mut zero_signs = [0, 0];
    let mut order_sensitive = 0;
    for length in (21..=64).chain([101, 256, 1001]) {
        let items = generate(length);
        let values = to_numbers(&items);
        let negated = negate(&values);
        let list = items.join(", ");
        let median = number(&mut runtime, &format!("[{list}].median()"));
        let reference = reference_median(&values);
        assert!(
            same_number(median, reference),
            "[{list}]: {median:?} and {reference:?}"
        );
        let selected = number(&mut runtime, &format!("[{list}].median(item -> -item)"));
        assert!(
            same_number(selected, reference_median(&negated)),
            "[{list}] negated: {selected:?}"
        );
        if median == 0.0 {
            zero_signs[usize::from(median.is_sign_negative())] += 1;
        }
        if depends_on_zero_order(&values) {
            order_sensitive += 1;
        }
    }
    assert_eq!(zero_signs, [25, 22]);
    assert_eq!(order_sensitive, 19);
    let levels = generate(101);
    let values = to_numbers(&levels);
    let negated = negate(&values);
    assert!(depends_on_zero_order(&values));
    assert!(depends_on_zero_order(&negated));
    let creations: Vec<String> = levels
        .iter()
        .map(|level| format!("Reading({level})"))
        .collect();
    let mut population = run(&format!(
        "class Reading(level):\n    this.level = level\n{}",
        creations.join("\n")
    ));
    for (selector, reference) in [
        ("reading -> reading.level", reference_median(&values)),
        (".level", reference_median(&values)),
        ("reading -> -reading.level", reference_median(&negated)),
    ] {
        let median = number(&mut population, &format!("Reading.median({selector})"));
        assert!(
            same_number(median, reference),
            "{selector}: {median:?} and {reference:?}"
        );
    }
}

#[test]
fn even_counts_take_the_mean_of_the_two_middle_values_as_written() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("[1, 2].median()", 1.5),
        ("[2, 1].median()", 1.5),
        ("[0.1, 0.2].median()", 0.15000000000000002),
        ("[3, 3].median()", 3.0),
        ("[5e-324, 5e-324].median()", 5e-324),
        ("[1e308, 1e308].median()", f64::INFINITY),
        ("[-1e308, -1e308].median()", f64::NEG_INFINITY),
        ("[1e308, 1.5e308, 1].median()", 1e308),
        (
            "[Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY].median()",
            f64::INFINITY,
        ),
        (
            "[Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY].median()",
            f64::NEG_INFINITY,
        ),
        ("[Number.POSITIVE_INFINITY, 1].median()", f64::INFINITY),
        ("[Number.NEGATIVE_INFINITY, 5].median()", f64::NEG_INFINITY),
        ("[Number.NEGATIVE_INFINITY, 1, 2].median()", 1.0),
        (
            "[Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY].median()",
            f64::NAN,
        ),
        (
            "[Number.NEGATIVE_INFINITY, 0, 1, Number.POSITIVE_INFINITY].median()",
            0.5,
        ),
    ] {
        let median = number(&mut runtime, source);
        assert!(same_number(median, expected), "{source}: {median:?}");
    }
    for items in ["1, 2", "0.1, 0.2", "1e308, 1e308", "-3, 7.5", "5e-324, 1"] {
        let median = number(&mut runtime, &format!("[{items}].median()"));
        let average = number(&mut runtime, &format!("[{items}].average()"));
        assert!(same_number(median, average), "[{items}]");
    }
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
    for (query, median, visits) in [
        (
            "values.median(note)",
            json!(2),
            json!([[3, null, null], [1, null, null], [2, null, null]]),
        ),
        (
            "Item.median(item -> note(item.size))",
            json!(6),
            json!([[5, null, null], [7, null, null]]),
        ),
        (
            "String([\"x\", 1, 2].median(note))",
            json!("NaN"),
            json!([["x", null, null], [1, null, null], [2, null, null]]),
        ),
        ("String([].median(note))", json!("NaN"), json!([])),
    ] {
        run("log = []");
        assert_eq!(run(query), median, "{query}");
        assert_eq!(run("log"), visits, "{query}");
    }
    run("log = []\nvalues.push(values[9])");
    assert_eq!(run("typeof values.median(note)"), "undefined");
    assert_eq!(run("log.length"), 4);
    run("log = []\nItem(1)");
    assert_eq!(run("Item.median(item -> note(item.size))"), 5);
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
            let source = format!("{receiver}.median({argument})");
            assert_eq!(
                fail(&source),
                "TypeError: median expects a function",
                "{source}"
            );
        }
    }
    assert_eq!(
        fail("[1].median(5, missing)"),
        "ReferenceError: missing is not defined"
    );
    assert_eq!(
        fail("[1].median(missing, 5)"),
        "ReferenceError: missing is not defined"
    );
    assert_eq!(
        fail("Item.median(missing)"),
        "ReferenceError: missing is not defined"
    );
    assert_eq!(run("nothing.median(log.push(1))"), json!(null));
    assert_eq!(run("typeof nothing.median()"), "undefined");
    assert_eq!(run("holder.missing.median(log.push(2))"), json!(null));
    assert_eq!(run("log"), json!([]));
    assert_eq!(
        run("[1, 2].median(value -> value, log.push(\"a\"), log.push(\"b\"))"),
        1.5
    );
    assert_eq!(run("log"), json!(["a", "b"]));
    assert_eq!(run("[1, 2].median(value -> value, 99)"), 1.5);
    assert_eq!(run("[1, 2].median(value -> value, 99, 7)"), 1.5);
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
    assert_eq!(run("source().median(pick(), extra())"), 20);
    assert_eq!(run("calls"), json!(["receiver", "selector", "extra"]));
    assert_eq!(run("Item.median(grow())"), 1);
    assert_eq!(run("Item.length"), 2);
}

#[test]
fn medians_depend_on_what_the_accepted_spelling_depends_on() {
    let mut runtime = run(r#"
values = [1, 2, 9]
middle = values.median()
spelled = values[:].sort(lambda a, b: a - b)[values.length / 2 - 0.5 : values.length / 2 + 1].average()
frozen = values.value.median()
class Order(amount):
    this.amount = amount
a = Order(1)
b = Order(2)
c = Order(9)
typical = Order.median(order -> order.amount)
total = Order.map(order -> order.amount).sort(lambda a, b: a - b)[Order.length / 2 - 0.5 : Order.length / 2 + 1].average()
rate = 2
def scaled order => order.amount * rate
weighted = Order.median(scaled)
assert(middle, 2)
assert(typical, 2)
assert(weighted, 4)
"#);
    assert_eq!(variable_dependencies(&runtime, "middle"), ["values"]);
    assert_eq!(
        variable_dependencies(&runtime, "middle"),
        variable_dependencies(&runtime, "spelled")
    );
    assert_eq!(
        variable_dependencies(&runtime, "frozen"),
        Vec::<&str>::new()
    );
    assert_eq!(
        variable_dependencies(&runtime, "typical"),
        ["$Order", "a.amount", "b.amount", "c.amount"]
    );
    assert_eq!(
        variable_dependencies(&runtime, "typical"),
        variable_dependencies(&runtime, "total")
    );
    assert_eq!(
        variable_dependencies(&runtime, "weighted"),
        [
            "$Order", "scaled", "a.amount", "rate", "b.amount", "c.amount"
        ]
    );
    checked(
        &mut runtime,
        "values.push(3)\nassert(middle, 2.5)\nassert(spelled, 2.5)\nassert(frozen, 2)\nd = Order(3)\n\
         assert(typical, 2.5)\nrate = 3\nassert(weighted, 7.5)\nb.amount = 5\nassert(typical, 4)\n\
         assert(total, 4)\nassert(weighted, 12)",
    );
    assert_eq!(
        variable_dependencies(&runtime, "typical"),
        ["$Order", "a.amount", "b.amount", "c.amount", "d.amount"]
    );
}

#[test]
fn live_medians_follow_list_mutations_instances_and_properties() {
    let mut runtime = run(r#"
class Item(price, qty):
    this.price = price
    this.qty = qty
values = [3, 8, "4"]
pen = Item(2, 5)
cap = Item(6, 1)
mug = Item(9, 3)
m0 = values.median()
m1 = values.median(value -> value * 2)
m2 = Item.median(item -> item.price * item.qty)
m3 = Item.filter(item -> item.qty > 1).median(item -> item.price)
m4 = Item[-2:].median(item -> item.qty)
"#);
    let read = |runtime: &mut Runtime| {
        let value = runtime
            .run("[m0, m1, m2, m3, m4].map(value -> typeof value == \"number\" ? String(value) : value)")
            .unwrap();
        runtime.serialize_json(&value).unwrap()
    };
    assert_eq!(read(&mut runtime), r#"["4","8","10","5.5","2"]"#);
    for (step, expected) in [
        ("values.push(5)", r#"["4.5","9","10","5.5","2"]"#),
        ("cap.qty = 4", r#"["4.5","9","24","6","3.5"]"#),
        ("Item(7, 2)", r#"["4.5","9","19","6.5","2.5"]"#),
        ("values.shift()", r#"["5","10","19","6.5","2.5"]"#),
        ("values.pop()", r#"["6","12","19","6.5","2.5"]"#),
        ("values = []", r#"["NaN","NaN","19","6.5","2.5"]"#),
        ("values.push(\"x\")", r#"["NaN","NaN","19","6.5","2.5"]"#),
        ("delete pen.qty", r#"["NaN","NaN",null,"7","2.5"]"#),
        ("delete pen.price", r#"["NaN","NaN",null,"7","2.5"]"#),
        ("delete pen", r#"["NaN","NaN","24","7","2.5"]"#),
        (
            "values = [0.1, 0.2, 0.3, 0.4]",
            r#"["0.25","0.5","24","7","2.5"]"#,
        ),
    ] {
        runtime.run(step).unwrap();
        assert_eq!(read(&mut runtime), expected, "{step}");
    }
}

#[test]
fn populations_gaining_and_losing_instances_re_evaluate_medians() {
    run(r#"
class Visit(minutes):
    this.minutes = minutes
first = Visit(10)
second = Visit(20)
third = Visit(40)
typical = Visit.median(visit -> visit.minutes)
assert(typical, 20)
fourth = Visit(6)
assert(typical, 15)
delete second.minutes
assert(typical, null)
delete second
assert(typical, 10)
assert(Visit.length, 3)
class Empty
none = Empty.median(empty -> empty.size)
assert(Number.isNaN(none), true)
Empty()
assert(none, null)
"#);
}

#[test]
fn class_level_rules_take_a_median_for_each_instance() {
    let mut runtime = run(r#"
class Player(team, score):
    this.team = team
    this.score = score
class Team(name, bonuses):
    this.name = name
    this.bonuses = bonuses
$Team.form = Player.filter(player -> player.team == $Team).median(player -> player.score) + $Team.bonuses.median()
red = Team("red", [1, 10, 2])
blue = Team("blue", [2, 4])
Player(red, 3)
Player(red, 20)
Player(red, 5)
ana = Player(blue, 4)
assert(red.form, 7)
assert(blue.form, 7)
ana.score = 6
red.bonuses.push(30)
assert(blue.form, 9)
assert(red.form, 11)
Player(red, 1)
assert(red.form, 10)
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
fn the_receiver_is_never_written_and_keeps_its_order_and_versions() {
    let mut runtime = run(r#"
waits = [12, 3, 45]
shop = {prices: [9, 1, 5]}
middle = waits.median()
priced = shop.prices.median()
doubled = waits.median(wait -> wait * 2)
waits.median()
shop.prices.median()
assert(waits, [12, 3, 45])
assert(shop.prices, [9, 1, 5])
assert(waits@last, [12, 3, 45])
assert(typeof waits@1, "undefined")
assert(typeof shop.prices@1, "undefined")
class Item(size):
    this.size = size
Item(3)
Item(1)
Item(2)
spread = Item.median(item -> item.size)
assert(Item.map(item -> item.size), [3, 1, 2])
assert([middle, priced, doubled, spread], [12, 5, 24, 2])
"#);
    checked(
        &mut runtime,
        "waits.push(0)\nassert(waits@1, [12, 3, 45, 0])\nassert(typeof waits@2, \"undefined\")\n\
         assert(middle, 7.5)\nassert(waits, [12, 3, 45, 0])\nsorted = [3, 1, 2]\n\
         sorted.sort((a, b) -> a - b)\nassert(sorted@1, [1, 2, 3])",
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
report = {median: 3}
holder = {median: value -> value * 2}
empty = {}
median = 7
class Review
$Review.rate = $Review.median / 10
review = Review()
review.median = 42
def twice median => median * 2
nothing = null
count = 5
flag = true
when = Date(5)
pattern = /a/
def helper(value) => value
x = {median: 4}.median
keyed = {"median": 2}
class Stats:
    def median(): return 42
stats = Stats()
class Shelf(median):
    this.median = median
shelf = Shelf([1, 2])
stats2 = {median: 4, mean: 5}
{median: m, mean} = stats2
pair = [3, 4]
first, second = pair
seen = []
for key, value of {median: 3}:
    seen.push(key)
"#);
    for (source, expected) in [
        ("values.median", json!(null)),
        ("typeof values.median", json!("undefined")),
        ("Order.median", json!(null)),
        ("typeof Order.median", json!("undefined")),
        ("Math.median", json!(null)),
        ("typeof Math.median", json!("undefined")),
        ("`m=${values.median}`", json!("m=undefined")),
        ("report.median", json!(3)),
        ("holder.median(4)", json!(8)),
        ("empty.median()", json!(null)),
        ("typeof empty.median()", json!("undefined")),
        ("median", json!(7)),
        ("review.median", json!(42)),
        ("review.rate", json!(4.2)),
        ("twice(4)", json!(8)),
        ("x", json!(4)),
        ("keyed[\"median\"]", json!(2)),
        ("nothing.median()", json!(null)),
        ("typeof nothing.median().length", json!("undefined")),
        ("stats.median()", json!(42)),
        ("String(Stats.median())", json!("NaN")),
        ("Stats.median(each -> each.median())", json!(42)),
        ("shelf.median", json!([1, 2])),
        ("[m, mean]", json!([4, 5])),
        ("[first, second]", json!([3, 4])),
        ("seen", json!(["median"])),
        ("values.median() + median", json!(8.5)),
        ("(lambda median: median + 1)(2)", json!(3)),
        ("(median -> median * 3)(2)", json!(6)),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
    for (source, message) in [
        (
            "\"abc\".median()",
            "TypeError: abc.median is not a function",
        ),
        (
            "count.median()",
            "TypeError: number.median is not a function",
        ),
        (
            "flag.median()",
            "TypeError: boolean.median is not a function",
        ),
        ("when.median()", "TypeError: Date.median is not a function"),
        ("pattern.median()", "TypeError: median is not a function"),
        (
            "helper.median()",
            "TypeError: function.median is not a function",
        ),
        (
            "Math.median(1, 2)",
            "TypeError: Math.median is not a function",
        ),
        (
            "Math.median([1])",
            "TypeError: Math.median is not a function",
        ),
        ("Class.median()", "ReferenceError: Class is not defined"),
        ("List.median()", "ReferenceError: List is not defined"),
        (
            "values.median(missing)",
            "ReferenceError: missing is not defined",
        ),
    ] {
        assert_eq!(fail(source), message, "{source}");
    }
    let (mut defined, mut missing) = common::runners();
    assert_eq!(
        missing("median([1, 2])"),
        "ReferenceError: median is not defined"
    );
    defined("def median(v) => 99\nxs = [1, 2]");
    assert_eq!(defined("median(xs)"), 99);
    assert_eq!(defined("[median(xs), xs.median()]"), json!([99, 1.5]));
    let (mut assigned, _) = common::runners();
    assigned("class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)\nOrder.median = 7");
    assert_eq!(assigned("Order.median"), json!(null));
    assert_eq!(assigned("Order.median(order -> order.total)"), 6);
    let (mut live, _) = common::runners();
    live("stats = {median: 4}\n{median} = stats\nstats.median = 6");
    assert_eq!(live("median"), 6);
    let (mut looped, _) = common::runners();
    looped(
        "class P(v):\n    this.v = v\nP(1)\nP(2)\nfor median of P:\n    median.w = median.v * 10\nmedian = 5\nflag = 0\nif median > 1:\n    flag = 1",
    );
    assert_eq!(
        looped("[P.map(p -> p.w), flag, median]"),
        json!([[10, 20], 1, 5])
    );
}

#[test]
fn median_of_keeps_its_accepted_syntax_errors() {
    for (source, expected_failure) in [
        (
            "xs = [1]\nmedian of xs",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 2, 11),
        ),
        (
            "xs = [1]\nx = median of xs",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 2, 15),
        ),
        (
            "median of [1, 2]",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 1, 11),
        ),
        (
            "xs = [1]\n[median of xs]",
            expected(
                ErrorKind::Syntax,
                "Expected ']' but found keyword 'of'",
                2,
                9,
            ),
        ),
        (
            "xs = [1]\nMedian of xs",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 2, 11),
        ),
        (
            "xs = [1]\nmedians of xs",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 2, 12),
        ),
        (
            "xs = [1]\nxs.median of xs",
            expected(ErrorKind::Syntax, "Unexpected keyword 'of'", 2, 14),
        ),
        (
            "xs = [1]\n`${median of xs}`",
            expected(ErrorKind::Reference, "median is not defined", 2, 1),
        ),
    ] {
        assert_eq!(failure(source), expected_failure, "{source}");
    }
    let (mut run, mut fail) = common::runners();
    run("median = 3\nxs = [1]");
    assert_eq!(run("`${median of xs}`"), "3");
    assert_eq!(
        fail("for x of median of xs: x"),
        "SyntaxError: Expected ':' but found keyword 'of'"
    );
}

#[test]
fn compatibility_rows_give_the_documented_results() {
    let orders = "class Order(total):\n    this.total = total\nOrder(5)\nOrder(7)";
    let bag = "class Bag:\n    def median(): return 9\nbag = Bag()";
    let kind = format!("{orders}\nkind = Order");
    for (setup, source, expected) in [
        ("", "[1, 2, 3].median()", json!(2)),
        ("", "String([].median())", json!("NaN")),
        ("xs = [3, 1, 2]", "xs.median(x -> x * 2)", json!(4)),
        ("o = {items: [1, 2]}", "o.items.median()", json!(1.5)),
        ("xs = [3, 4]", "xs.map(x -> x * 2).median()", json!(7)),
        ("xs = [1, 2, 3]", "xs.value.median()", json!(2)),
        ("xs = [1]\nxs.push(2)", "xs@first.median()", json!(1)),
        (orders, "Order.median(o -> o.total)", json!(6)),
        (orders, "Order.median(.total)", json!(6)),
        (orders, "Order[1:].median(o -> o.total)", json!(7)),
        (orders, "Order.list().median(o -> o.total)", json!(6)),
        (kind.as_str(), "kind.median(o -> o.total)", json!(6)),
        ("class Empty", "String(Empty.median())", json!("NaN")),
        (bag, "String(Bag.median())", json!("NaN")),
        (bag, "Bag.median(b -> b.median())", json!(9)),
        ("", "[10, 9, 1].median()", json!(9)),
        ("", "[4, 1, 3, 2].median()", json!(2.5)),
        ("", "[0.1, 0.2].median()", json!(0.15000000000000002)),
        ("", "String([1, \"a\"].median())", json!("NaN")),
        ("xs = [1, 2]", "xs.median(x -> x, 5)", json!(1.5)),
        (
            "xs = [1, 2]\nr = 0\ntry:\n    r = xs.median()\ncatch error:\n    r = error",
            "r",
            json!(1.5),
        ),
        (
            "xs = [1, 2]\nflag = 0\nif xs.median() > 1:\n    flag = 1",
            "flag",
            json!(1),
        ),
        ("median = [3, 1, 2]", "median.median()", json!(2)),
        ("def f(median) => median.median()", "f([1, 2, 3])", json!(2)),
        (
            "class S(median):\n    this.median = median\ns = S([1, 2])",
            "s.median.median()",
            json!(1.5),
        ),
        (
            "class S:\n    def median(): return 42\ns = S()",
            "s.median()",
            json!(42),
        ),
        ("o = {median: 3}", "o.median", json!(3)),
        ("o = {median: x -> x * 2}", "o.median(4)", json!(8)),
        ("o = {}", "o.median()", json!(null)),
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
            "xs.median(5)",
            "TypeError: median expects a function",
        ),
        (
            "",
            "[].median(null)",
            "TypeError: median expects a function",
        ),
        (
            orders,
            "Order.median(\"total\")",
            "TypeError: median expects a function",
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
        ("[].median(null)", 1),
        ("xs = [1, 2]\nxs.median(5)", 2),
        (
            "class Order(total):\n    this.total = total\nOrder(5)\nOrder.median(\"total\")",
            4,
        ),
    ] {
        assert_eq!(
            failure(source),
            expected(ErrorKind::Type, "median expects a function", line, 1),
            "{source}"
        );
    }
}

#[test]
fn class_names_run_the_population_query_beside_instance_methods() {
    let (mut run, _) = common::runners();
    run("class Bag:\n    def median(): return 9\nfirst = Bag()\nsecond = Bag()");
    assert_eq!(run("first.median()"), 9);
    assert_eq!(run("Bag.median(bag -> bag.median())"), 9);
    assert_eq!(run("String(Bag.median())"), "NaN");
    assert_eq!(run("Bag.length"), 2);
}

#[test]
fn rendering_reasoning_and_keys_keep_the_ordinary_call() {
    let mut runtime = run(r#"
temps = [18, 24, 9]
middle = temps.median()
hot = middle > 20
class Order(amount):
    this.amount = amount
a = Order(4)
b = Order(8)
typical = Order.median(order -> order.amount)
picked = Order.median(.amount)
filtered = Order.filter(order -> order.amount > 5).median(order -> order.amount * 2)
if temps.median() > 17:
    warm = true
assert(warm, true)
"#);
    let (mut repl, mut fail) = common::runners();
    repl(
        "temps = [18, 24, 9]\nmiddle = temps.median()\nhot = middle > 20\nclass Order(amount):\n    this.amount = amount\na = Order(4)\nb = Order(8)\ntypical = Order.median(order -> order.amount)\npicked = Order.median(.amount)\nfiltered = Order.filter(order -> order.amount > 5).median(order -> order.amount * 2)",
    );
    for (source, expected) in [
        ("(why middle)[0].rule", json!("middle = temps.median()")),
        (
            "(why typical)[0].rule",
            json!("typical = Order.median(function())"),
        ),
        (
            "(why picked)[0].rule",
            json!("picked = Order.median(.amount)"),
        ),
        (
            "(why filtered)[0].rule",
            json!("filtered = Order.filter(function()).median(function())"),
        ),
        ("(why middle)[0].from", json!(["temps"])),
        (
            "(why typical)[0].from",
            json!(["$Order", "a.amount", "b.amount"]),
        ),
        ("(why typical)[0].state", json!("derived")),
        ("(why typical)[0].holds", json!(6)),
        (
            "(affects temps).map(step -> step.rule)",
            json!(["middle = temps.median()", "hot = middle>20"]),
        ),
    ] {
        assert_eq!(repl(source), expected, "{source}");
    }
    for (source, message) in [
        (
            "why temps.median()",
            "TypeError: Cannot explain temps.median()",
        ),
        (
            "all t in temps.median(): t",
            "TypeError: temps.median() is not a collection",
        ),
        (
            "temps.median()(1)",
            "TypeError: temps.median() is not a function",
        ),
        (
            "Order.median(.amount)(1)",
            "TypeError: Order.median(.amount) is not a function",
        ),
        (
            "delete temps.median()",
            "SyntaxError: Cannot delete temps.median()",
        ),
        (
            "temps.median() = 1",
            "SyntaxError: Cannot assign to temps.median()",
        ),
    ] {
        assert_eq!(fail(source), message, "{source}");
    }
    let keys: Vec<String> = runtime
        .graph
        .nodes()
        .map(|node| node.key.to_string())
        .collect();
    for key in ["middle", "typical", "if(temps.median()>17)"] {
        assert!(keys.iter().any(|found| found == key), "{key}: {keys:?}");
    }
    checked(
        &mut runtime,
        "temps.push(30)\nassert(middle, 21)\nassert(hot, true)\nb.amount = 10\nassert(typical, 7)\nassert(filtered, 20)",
    );
}

#[test]
fn cli_runs_median_files_with_json_and_graph_output() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("list-medians.nuc");
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
        stdout.contains("if(Reading.median(function())>20)"),
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
fn repl_reads_median_lines_and_reports_their_errors() {
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
            b"class Reading(celsius):\n    this.celsius = celsius\n\n.json on\nReading(20)\nReading(23)\nmiddle = Reading.median(reading -> reading.celsius)\n[middle, [1, 2].median(), [].median()]\n(why middle)[0].rule\nxs = [1]\nmedian of xs\nxs.median(5)\n.exit\n",
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
            .any(|line| line == "\"middle = Reading.median(function())\""),
        "{stdout}"
    );
    assert_eq!(
        stderr.lines().collect::<Vec<_>>(),
        [
            "SyntaxError: Unexpected keyword 'of' (line 1, column 11)",
            "TypeError: median expects a function (line 1, column 1)",
        ]
    );
}

#[test]
fn failed_median_updates_restore_state_and_graph() {
    let initial = r#"
def check(value):
    if value > 100:
        throw "TOO_LARGE"
    return value
values = [1, 3, 8]
middle = values.median(check)
class Item(size):
    this.size = size
small = Item(1)
sizes = Item.median(item -> check(item.size))
assert(middle, 3)
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
        "failure = \"\"\ntry:\n    values.push(500)\ncatch error:\n    failure = error\nassert(failure, \"TOO_LARGE\")\n\
         assert(values, [1, 3, 8])\nassert(middle, 3)",
    );
    checked(
        &mut actual,
        "suppose:\n    Item(3)\n    assert(sizes, 2)\nassert(sizes, 1)\nassert(Item.length, 1)",
    );
    checked(
        &mut actual,
        "mid = Item(3)\nassert(sizes, 2)\nvalues.push(5)\nassert(middle, 4)",
    );
}

#[test]
fn medians_sort_each_receiver_once_and_scale_with_it() {
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
    assert_eq!(number(&mut runtime, "values.median()"), 9_999.5);
    assert_eq!(number(&mut runtime, "values.median(note)"), 9_999.5);
    assert_eq!(number(&mut runtime, "counter.n"), f64::from(items));
    runtime
        .run("middle = values.median(value -> value * value)")
        .unwrap();
    assert_eq!(number(&mut runtime, "middle"), 99_990_000.5);
    assert_eq!(
        runtime.run("(why middle)[0].rule").unwrap(),
        Value::String("middle = values.median(function())".to_string())
    );
    assert_eq!(number(&mut runtime, "values[0]"), 0.0);
    assert_eq!(number(&mut runtime, "values[1]"), 7_919.0);
}
