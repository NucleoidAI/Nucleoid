mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::parse;
use nucleoid::{Runtime, Value};
use serde_json::json;

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "predicate count regression".to_string(),
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

#[test]
fn predicates_are_required_even_for_empty_receivers() {
    let (mut run, mut fail) = common::runners();
    run("class Empty\nclass Item\nItem()\nholder = {}\nnothing = null\nlog = []");
    for receiver in ["[]", "[1]", "Empty", "Item"] {
        for callback in ["", "null"] {
            let source = format!("{receiver}.count({callback})");
            assert_eq!(
                fail(&source),
                "TypeError: count expects a function",
                "{source}"
            );
        }
    }
    for source in ["[1].count(0)", "[].count(holder.missing)"] {
        assert_eq!(
            fail(source),
            "TypeError: count expects a function",
            "{source}"
        );
    }
    assert_eq!(run("[].count(item -> true)"), 0);
    assert_eq!(run("Empty.count(item -> true)"), 0);
    assert_eq!(
        fail("[1].count(0, missing)"),
        "ReferenceError: missing is not defined"
    );
    assert_eq!(run("typeof nothing.count(log.push(1))"), "undefined");
    assert_eq!(run("log"), json!([]));
}

#[test]
fn counts_visit_every_snapshot_item_after_evaluating_all_arguments() {
    let (mut run, _) = common::runners();
    run(r#"
log = []
values = [3, 0, 2]
def source():
    log.push("receiver")
    return values
def visit(item, position, list):
    log.push([item, position, list])
    return item
def pick():
    log.push("predicate")
    return visit
def extra():
    log.push("extra")
    values.push(9)
    return 0
class Item(flag): this.flag = flag
Item(3)
Item(0)
def grow():
    Item(9)
    return item -> visit(item.flag)
"#);
    assert_eq!(
        run("source().count(pick(), extra(), log.push(\"last extra\"))"),
        2
    );
    assert_eq!(
        run("log"),
        json!([
            "receiver",
            "predicate",
            "extra",
            "last extra",
            [3, null, null],
            [0, null, null],
            [2, null, null]
        ])
    );
    assert_eq!(run("values"), json!([3, 0, 2, 9]));
    assert_eq!(
        run("[false, 0, \"\", null, Number.NaN, 2, \"x\", [], {}].count(item -> item)"),
        4
    );
    run("log = []");
    assert_eq!(run("Item.count(grow())"), 1);
    assert_eq!(run("log"), json!([[3, null, null], [0, null, null]]));
    assert_eq!(run("Item.length"), 3);
}

#[test]
fn counts_keep_filter_dependencies_missing_reads_and_detached_receivers() {
    let mut runtime = run(r#"
values = [0, 2]
limit = 1
live = values.count(value -> value > limit)
frozen = values.value.count(value -> value > limit.value)
holder = {}
deferred = values.count(value -> holder.ready)
legacy = values.filter(value -> holder.ready).length
assert: [live, frozen, deferred, legacy], [1, 1, 0, 0]
"#);
    let dependencies = &runtime
        .graph
        .retrieve(&NodeKey::variable("live"))
        .unwrap()
        .dependencies;
    assert_eq!(
        dependencies.iter().map(NodeKey::as_str).collect::<Vec<_>>(),
        ["values", "limit"]
    );
    assert!(
        runtime
            .graph
            .retrieve(&NodeKey::variable("frozen"))
            .unwrap()
            .dependencies
            .is_empty()
    );
    checked(
        &mut runtime,
        r#"
assert: typeof values.count(value -> holder.ready), "number"
assert: typeof values.filter(value -> holder.ready).length, "number"
values.push(3)
assert: [live, frozen], [2, 1]
holder.ready = null
assert: [deferred, legacy], [null, null]
holder.ready = true
assert: [deferred, legacy], [3, 3]
limit = 2
assert: [live, frozen], [1, 1]
assert: values@first.count(value -> value > 1), 1
assert: values@first.filter(value -> value > 1).length, 1
"#,
    );
}

#[test]
fn class_rules_and_failed_updates_preserve_ordinary_count_names() {
    let initial = r#"
count = 3
report = {count: 9}
callable = {count: value -> value * 2}
class Box:
    def count(): return 7
box = Box()
kind = Box
class Item(box, ready):
    this.box = box
    this.ready = ready
def accepted(value):
    if value == "bad":
        throw "BAD"
    return value
$Box.open = Item.count(item -> item.box == $Box and accepted(item.ready))
first = Item(box, true)
def positive(value):
    if value > 9:
        throw "LARGE"
    return value > 0
values = [1, 3]
total = values.count(positive)
assert: [count, report.count, callable.count(4), box.count()], [3, 9, 8, 7]
assert: Box.count(each -> each.count() == 7), 1
assert: kind.count(each -> each.count() == 7), 1
assert: [box.open, total], [1, 2]
assert: typeof values.count, "undefined"
"#;
    let mut actual = run(initial);
    let control = run(initial);
    for (source, thrown) in [
        ("values.push(10)", "LARGE"),
        ("first.ready = \"bad\"", "BAD"),
        ("later = Item(box, \"bad\")", "BAD"),
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
        r#"
first.ready = false
assert: box.open, 0
second = Item(box, true)
assert: box.open, 1
values.push(2)
assert: total, 3
"#,
    );
}
