//! Instance list ranges and list queries (NUC 61): ranges, `list` and `all`
//! on class instance lists; `at`, `lastIndexOf`, `findIndex`, `findLast` and
//! `findLastIndex` on lists and instance lists; and the unchanged `slice`,
//! `includes`, `indexOf` and `concat`.

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
            title: "list query regression".to_string(),
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

fn dependencies(runtime: &Runtime, key: &NodeKey) -> Vec<String> {
    runtime
        .graph
        .retrieve(key)
        .unwrap_or_else(|| panic!("{key:?} is filed"))
        .dependencies
        .iter()
        .map(|key| key.as_str().to_string())
        .collect()
}

fn variable_dependencies(runtime: &Runtime, name: &str) -> Vec<String> {
    dependencies(runtime, &NodeKey::variable(name))
}

const SPEC_TITLES: [&str; 10] = [
    "Nucleoid slices class instance lists with ranges",
    "Nucleoid keeps instance list ranges live",
    "Nucleoid lists every instance with list and all",
    "Nucleoid compares list members with deep equality",
    "Nucleoid concatenates lists and instance lists into new lists",
    "Nucleoid reads positions from either end with at and lastIndexOf",
    "Nucleoid searches from either end with findIndex findLast and findLastIndex",
    "Nucleoid keeps list queries live",
    "Nucleoid applies instance list ranges and queries in class level rules",
    "Nucleoid rolls back failed list query updates",
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
            "Nucleoid reads instance lists with ranges and list queries",
        ),
        (
            &readme,
            "Instance lists slice like lists and answer live queries",
        ),
        (
            &examples,
            "Live leaderboards read instance lists by range and position",
        ),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 180);
}

#[test]
fn instance_list_ranges_slice_like_lists_for_every_bound() {
    let (mut run, _) = common::runners();
    run("class Item\na = Item()\nb = Item()\nc = Item()\nd = Item()\nitems = [a, b, c, d]");
    let bounds = [
        None,
        Some("0"),
        Some("1"),
        Some("3"),
        Some("4"),
        Some("9"),
        Some("-1"),
        Some("-3"),
        Some("-4"),
        Some("-9"),
        Some("1.5"),
        Some("-1.5"),
        Some("null"),
        Some("true"),
        Some("\"2\""),
        Some("\"x\""),
        Some("[1]"),
        Some("Number.NaN"),
    ];
    for start in bounds {
        for end in bounds {
            let range = format!("[{}:{}]", start.unwrap_or(""), end.unwrap_or(""));
            let arguments = match (start, end) {
                (None, None) => String::new(),
                (start, None) => start.unwrap().to_string(),
                (start, Some(end)) => format!("{}, {end}", start.unwrap_or("0")),
            };
            let expected = run(&format!("items{range}"));
            assert_eq!(run(&format!("Item{range}")), expected, "Item{range}");
            assert_eq!(
                run(&format!("Item.slice({arguments})")),
                expected,
                "Item.slice({arguments})"
            );
        }
    }
}

#[test]
fn ranges_and_listings_read_the_population_once_before_their_arguments() {
    let (mut run, _) = common::runners();
    run("class Item\na = Item()\ndef grow():\n    Item()\n    return 9");
    assert_eq!(run("Item[0:grow()].length"), 1);
    assert_eq!(run("Item.length"), 2);
    assert_eq!(run("Item.slice(0, grow()).length"), 2);
    assert_eq!(run("Item.length"), 3);
    assert_eq!(run("Item.list(grow()).length"), 3);
    assert_eq!(run("Item.all(grow()).length"), 4);
    assert_eq!(run("Item.length"), 5);
}

#[test]
fn ranges_and_listings_depend_on_the_population_and_their_bounds() {
    let mut runtime = run(r#"
class Task
a = Task()
count = 1
recent = Task[-2:]
opening = Task[:count]
pinned = Task[:count.value]
everyone = Task.list()
roster = Task.all()
class Guest
party = Guest.concat(Task.list())
class Team(size):
    this.size = size
$Team.roster = Task[0:$Team.size]
red = Team(1)
assert(red.roster, [a])
"#);
    assert_eq!(variable_dependencies(&runtime, "recent"), ["$Task"]);
    assert_eq!(
        variable_dependencies(&runtime, "opening"),
        ["$Task", "count"]
    );
    assert_eq!(variable_dependencies(&runtime, "pinned"), ["$Task"]);
    assert_eq!(variable_dependencies(&runtime, "everyone"), ["$Task"]);
    assert_eq!(variable_dependencies(&runtime, "roster"), ["$Task"]);
    assert_eq!(
        variable_dependencies(&runtime, "party"),
        ["$Guest", "$Task"]
    );
    assert_eq!(
        dependencies(
            &runtime,
            &NodeKey::property(&ObjectId::named("red"), "roster")
        ),
        ["$Task", "red.size"]
    );
    checked(
        &mut runtime,
        "b = Task()\nassert(recent, [a, b])\nassert(opening, [a])\nassert(red.roster, [a])\n\
         red.size = 2\nassert(red.roster, [a, b])\nassert(everyone, [a, b])\nassert(party, [a, b])",
    );
}

#[test]
fn list_and_all_belong_to_instance_lists_only() {
    let (mut run, mut fail) = common::runners();
    run("class Task\na = Task()\nb = Task()\nplain = [1, 2]\nlog = []");
    for source in ["Task.list()", "Task.all()", "Task[:]", "Task.slice()"] {
        assert_eq!(run(source), json!([{"id": "a"}, {"id": "b"}]), "{source}");
    }
    assert_eq!(run("Task.list(log.push(1), log.push(2)).length"), 2);
    assert_eq!(run("log"), json!([1, 2]));
    assert_eq!(
        fail("Task.all(missing)"),
        "ReferenceError: missing is not defined"
    );
    for (source, message) in [
        ("plain.list()", "TypeError: plain.list is not a function"),
        ("plain.all()", "TypeError: plain.all is not a function"),
        ("[1, 2].all()", "TypeError: [1,2].all is not a function"),
        ("\"ab\".list()", "TypeError: ab.list is not a function"),
        ("Math.list()", "TypeError: Math.list is not a function"),
        ("Class.list()", "ReferenceError: Class is not defined"),
    ] {
        assert_eq!(fail(source), message, "{source}");
    }
    run(r#"
class Shelf:
    def list() => "own list"
shelf = Shelf()
shelf.all = () => "own all"
assert(shelf.list(), "own list")
assert(shelf.all(), "own all")
assert(Shelf.list(), [shelf])
"#);
}

#[test]
fn unchanged_forms_keep_their_accepted_results() {
    let (mut run, _) = common::runners();
    run("class User\na = User()\nb = User()\nvalues = [1, 2, 3]");
    for (source, expected) in [
        ("Class[:]", json!(null)),
        ("Math[:]", json!(null)),
        ("typeof Class[:]", json!("undefined")),
        ("User[0]", json!({"id": "a"})),
        ("User[\"b\"]", json!({"id": "b"})),
        ("User[9]", json!(null)),
        ("typeof User[9]", json!("null")),
        ("User[-1]", json!({"id": "a"})),
        ("User[1.5]", json!({"id": "b"})),
        ("values[-1]", json!(3)),
        ("values[-5]", json!(1)),
        ("typeof values[9]", json!("undefined")),
        ("values.slice(-2)", json!([2, 3])),
        ("values.includes(2)", json!(true)),
        ("values.indexOf(3)", json!(2)),
        ("values.concat([4])", json!([1, 2, 3, 4])),
        ("values.contains(2)", json!(true)),
        ("values.any(value => value > 2)", json!(true)),
        ("values.reverse()", json!([3, 2, 1])),
        ("User.reverse()", json!([{"id": "b"}, {"id": "a"}])),
        ("[1].concat(User)", json!([1, "User"])),
        ("values", json!([1, 2, 3])),
        ("\"abc\"[1:]", json!("bc")),
    ] {
        assert_eq!(run(source), expected, "{source}");
    }
}

#[test]
fn at_resolves_indexes_as_javascript_does() {
    let (mut run, _) = common::runners();
    run(
        "values = [10, 20, 30]\nclass Lap\nfirst = Lap()\nsecond = Lap()\nthird = Lap()\nclass Empty",
    );
    let items = [10, 20, 30];
    let laps = ["first", "second", "third"];
    for (index, position) in [
        ("0", Some(0)),
        ("1", Some(1)),
        ("2", Some(2)),
        ("3", None),
        ("9", None),
        ("-1", Some(2)),
        ("-2", Some(1)),
        ("-3", Some(0)),
        ("-4", None),
        ("-9", None),
        ("0.5", Some(0)),
        ("1.9", Some(1)),
        ("2.999", Some(2)),
        ("3.5", None),
        ("-0.5", Some(0)),
        ("-1.5", Some(2)),
        ("-3.9", Some(0)),
        ("-4.0", None),
        ("", Some(0)),
        ("null", Some(0)),
        ("true", Some(1)),
        ("false", Some(0)),
        ("\"2\"", Some(2)),
        ("\" 1 \"", Some(1)),
        ("\"\"", Some(0)),
        ("\"x\"", Some(0)),
        ("[2]", Some(2)),
        ("[]", Some(0)),
        ("[1, 2]", Some(0)),
        ("{}", Some(0)),
        ("Number.NaN", Some(0)),
        ("Number.POSITIVE_INFINITY", None),
        ("Number.NEGATIVE_INFINITY", None),
        ("Number.MAX_VALUE", None),
        ("-Number.MAX_VALUE", None),
        ("0, 99", Some(0)),
    ] {
        let list = format!("values.at({index})");
        let population = format!("Lap.at({index})");
        match position {
            Some(position) => {
                assert_eq!(run(&list), json!(items[position]), "{list}");
                assert_eq!(
                    run(&population),
                    json!({"id": laps[position]}),
                    "{population}"
                );
            }
            None => {
                assert_eq!(run(&format!("typeof {list}")), "undefined", "{list}");
                assert_eq!(run(&format!("typeof {population}")), "null", "{population}");
            }
        }
    }
    assert_eq!(run("typeof [].at(0)"), "undefined");
    assert_eq!(run("typeof Empty.at(0)"), "null");
    assert_eq!(run("typeof Empty.at()"), "null");
}

#[test]
fn searches_visit_items_in_order_and_stop_at_the_first_match() {
    let (mut run, _) = common::runners();
    run(r#"
log = []
def accept(value):
    log.push(value)
    return value % 2 == 0
values = [1, 2, 3, 4, 5]
class Item(size):
    this.size = size
Item(1)
Item(2)
Item(3)
Item(4)
Item(5)
"#);
    for (query, result, visits) in [
        ("values.find(accept)", json!(2), json!([1, 2])),
        ("values.findIndex(accept)", json!(1), json!([1, 2])),
        ("values.findLast(accept)", json!(4), json!([5, 4])),
        ("values.findLastIndex(accept)", json!(3), json!([5, 4])),
        (
            "values.findIndex(value => accept(value) && value > 9)",
            json!(-1),
            json!([1, 2, 3, 4, 5]),
        ),
        (
            "values.findLast(value => accept(value) && value > 9)",
            json!(null),
            json!([5, 4, 3, 2, 1]),
        ),
        (
            "values.findLastIndex(value => accept(value) && value > 9)",
            json!(-1),
            json!([5, 4, 3, 2, 1]),
        ),
        (
            "Item.find(item => accept(item.size)).size",
            json!(2),
            json!([1, 2]),
        ),
        (
            "Item.findIndex(item => accept(item.size))",
            json!(1),
            json!([1, 2]),
        ),
        (
            "Item.findLast(item => accept(item.size)).size",
            json!(4),
            json!([5, 4]),
        ),
        (
            "Item.findLastIndex(item => accept(item.size))",
            json!(3),
            json!([5, 4]),
        ),
        ("[].findLast(accept)", json!(null), json!([])),
        ("[].findLastIndex(accept)", json!(-1), json!([])),
    ] {
        run("log = []");
        assert_eq!(run(query), result, "{query}");
        assert_eq!(run("log"), visits, "{query}");
    }
    for (query, result) in [
        (
            "[7, 8].findIndex((value, position, list) => position == null && list == null)",
            json!(0),
        ),
        (
            "[7, 8].findLast((value, position) => position == null)",
            json!(8),
        ),
        (
            "[7, 8].findLastIndex((value, position) => position == null)",
            json!(1),
        ),
    ] {
        assert_eq!(run(query), result, "{query}");
    }
}

#[test]
fn searches_depend_only_on_the_items_they_visit() {
    let mut runtime = run(r#"
class Task(hot):
    this.hot = hot
a = Task(true)
b = Task(false)
c = Task(true)
latest = Task.findLast(task => task.hot)
early = Task.findIndex(task => task.hot)
spot = Task.findLastIndex(task => task.hot)
assert(latest, c)
assert(early, 0)
assert(spot, 2)
"#);
    assert_eq!(
        variable_dependencies(&runtime, "latest"),
        ["$Task", "c.hot"]
    );
    assert_eq!(variable_dependencies(&runtime, "early"), ["$Task", "a.hot"]);
    assert_eq!(variable_dependencies(&runtime, "spot"), ["$Task", "c.hot"]);
    checked(
        &mut runtime,
        "c.hot = false\nassert(latest, a)\nassert(spot, 0)\nassert(early, 0)",
    );
    assert_eq!(
        variable_dependencies(&runtime, "latest"),
        ["$Task", "c.hot", "b.hot", "a.hot"]
    );
}

#[test]
fn callback_validation_follows_find() {
    let (mut run, mut fail) = common::runners();
    run("class Empty\nclass Item\nItem()\nlog = []\nnothing = null");
    for name in ["find", "findIndex", "findLast", "findLastIndex"] {
        let message = format!("TypeError: {name} expects a function");
        for receiver in ["[]", "[1, 2]", "Empty", "Item"] {
            for argument in ["", "5", "null", "\"f\"", "[1]", "{}"] {
                let source = format!("{receiver}.{name}({argument})");
                assert_eq!(fail(&source), message.as_str(), "{source}");
            }
        }
        // Every supplied argument, including an unused extra, is evaluated
        // before the callback is validated; a failed run rolls back their
        // effects, so the error order shows it.
        assert_eq!(
            fail(&format!("[1].{name}(missing, 5)")),
            "ReferenceError: missing is not defined"
        );
        assert_eq!(
            fail(&format!("[1].{name}(5, missing)")),
            "ReferenceError: missing is not defined"
        );
        run("log = []");
        assert_eq!(run(&format!("nothing.{name}(log.push(1))")), json!(null));
        assert_eq!(run("log"), json!([]));
    }
    for query in ["at", "lastIndexOf", "list", "all"] {
        assert_eq!(run(&format!("nothing.{query}(log.push(1))")), json!(null));
        assert_eq!(run("log"), json!([]));
    }
}

#[test]
fn comparisons_use_deep_equality_without_subscribing_to_compared_objects() {
    let mut runtime = run(r#"
items = [{ n: 1 }, [2], "3"]
probe = { n: 1 }
found = items.includes(probe)
first = items.indexOf(probe)
last = items.lastIndexOf(probe)
assert(found, true)
assert(first, 0)
assert(last, 0)
"#);
    for name in ["found", "first", "last"] {
        assert_eq!(variable_dependencies(&runtime, name), ["items", "probe"]);
    }
    checked(
        &mut runtime,
        "probe.n = 2\nassert(found, true)\nassert(first, 0)\n\
         items.push({ n: 2 })\nassert(found, true)\nassert(first, 3)\nassert(last, 3)\n\
         probe = [2]\nassert(first, 1)\nassert(last, 1)",
    );
}

#[test]
fn failed_query_updates_restore_state_and_graph() {
    let initial = r#"
def check(value):
    if value > 100:
        throw "TOO_LARGE"
    return value > 2
values = [1, 3]
spot = values.findLastIndex(check)
class Item(size):
    this.size = size
small = Item(1)
pick = Item.findLast(item => check(item.size))
recent = Item[-1:]
assert(spot, 1)
assert(pick, null)
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
        "mid = Item(3)\nassert(pick, mid)\nassert(recent, [mid])\nvalues.push(5)\nassert(spot, 2)",
    );
}
