//! In-place list sorting (NUC 58): writable receivers, receivers without a
//! list place, standing sorts, unchanged ordering, history and rollback.

mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "in-place sort regression".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count);
    value
}

fn run(source: &str) -> Runtime {
    let mut runtime = Runtime::new();
    checked(&mut runtime, source, None);
    runtime
}

fn dependencies(runtime: &Runtime, name: &str) -> Vec<String> {
    runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap_or_else(|| panic!("{name} is filed"))
        .dependencies
        .iter()
        .map(|key| key.as_str().to_string())
        .collect()
}

const SPEC_TITLES: [&str; 7] = [
    "Nucleoid sorts a list in place like JavaScript",
    "Nucleoid sorts properties and instance members in place",
    "Nucleoid returns sorted copies from receivers without a list place",
    "Nucleoid sorts only the local binding of a copied list",
    "Nucleoid keeps a list sorted with a standing sort",
    "Nucleoid records in place sorts as list versions",
    "Nucleoid rolls back failed in place sorts",
];

#[test]
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let appended = SPEC_TITLES
        .iter()
        .map(|title| (&spec, *title))
        .chain([(&reference, "Nucleoid sorts lists in place")]);

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
    assert_eq!(assertions, 74);

    // The accepted reference assertion on a literal receiver is untouched.
    let builtins = reference
        .iter()
        .find(|case| case.title == "Nucleoid provides the built-in members the reference lists")
        .unwrap();
    assert!(builtins.source.contains("assert([3, 1, 2].sort()[0], 1)"));
    common::check(builtins).unwrap();
}

#[test]
fn writable_receivers_sort_in_place_and_return_the_stored_list() {
    let runtime = run(r#"
scores = [30, 10, 20]
first = scores[0]
assert(scores.sort((a, b) => a - b), [10, 20, 30])
assert(scores, [10, 20, 30])
assert(first, 10)
board = { entries: [3, 1, 2] }
assert(board.entries.sort((a, b) => b - a), [3, 2, 1])
assert(board.entries, [3, 2, 1])
(scores).sort((a, b) => b - a)
assert(scores, [30, 20, 10])
class Queue:
    def init(jobs) => self.jobs = jobs
    def order() => this.jobs.sort()
queue = Queue([3, 1, 2])
assert(queue.order(), [1, 2, 3])
assert(queue.jobs, [1, 2, 3])
older = { items: [3, 1, 2] }
newer = { items: [9, 8] }
picked = older
picked = newer
assert(picked@first.items.sort(), [1, 2, 3])
assert(picked@first["items"].sort(), [1, 2, 3])
assert(older.items, [1, 2, 3])
assert(newer.items, [9, 8])
assert(scores@0, [30, 10, 20])
assert(scores@1, [10, 20, 30])
assert(scores@2, [30, 20, 10])
assert(scores@3, null)
"#);
    assert_eq!(dependencies(&runtime, "first"), ["scores"]);
}

#[test]
fn a_member_receiver_is_read_once_and_snapshots_stay_detached() {
    let runtime = run(r#"
calls = []
board = { entries: [3, 1, 2] }
def pick():
    calls.push(1)
    return board
assert(pick().entries.sort(), [1, 2, 3])
assert(calls.length, 1)
assert(board.entries, [1, 2, 3])
class Tag(names):
    this.names = names
assert(Tag(["b", "a"]).names.sort(), ["a", "b"])
assert(Tag.length, 1)
assert(Tag[0].names, ["a", "b"])
scores = [30, 10, 20]
snapshot = scores.value.sort()
assert(scores.value.sort(), [10, 20, 30])
assert(scores, [30, 10, 20])
scores = [5, 4]
assert(snapshot, [10, 20, 30])
"#);
    assert!(dependencies(&runtime, "snapshot").is_empty());
}

#[test]
fn receivers_without_a_list_place_keep_their_accepted_results() {
    run(r#"
scores = [30, 10, 20]
ranked = scores.slice().sort((a, b) => a - b)
scores.push(5)
assert(scores, [30, 10, 20, 5])
assert(ranked, [5, 10, 20, 30])
assert([3, 1, 2].sort(), [1, 2, 3])
assert([3, 1, 2].sort()[0], 1)
grid = [[3, 1], [2]]
assert(grid[0].sort(), [1, 3])
assert(grid, [[3, 1], [2]])
record = { items: [2, 1] }
assert(record["items"].sort(), [1, 2])
assert(record.items, [2, 1])
history = [3, 1, 2]
history = [9, 8]
assert(history@first.sort(), [1, 2, 3])
assert(history@last.sort(), [8, 9])
assert(history@0.value.sort(), [1, 2, 3])
assert(history, [9, 8])
assert(history@first, [3, 1, 2])
assert(history@2, null)
nested = [[3, 1], 2]
nested.push(0)
assert(nested@0[0].sort(), [1, 3])
assert(nested@first, [[3, 1], 2])
holder = {}
assert(holder.items.sort(), null)
try:
    "cba".sort()
catch error:
    assert(error, TypeError("cba.sort is not a function"))
"#);
}

#[test]
fn push_pop_and_shift_keep_their_accepted_receivers_and_errors() {
    let mut runtime = run(r#"
arr = [1]
n = arr.push(2)
n = arr.push(3)
assert(arr, [1, 2, 3, 2])
assert(n, 3)
grid = [[3, 1], [2]]
assert(grid[0].push(5), 3)
assert(grid, [[3, 1], [2]])
record = { items: [2, 1] }
assert(record["items"].pop(), 1)
assert(record.items, [2, 1])
calls = []
board = { entries: [3, 1, 2] }
def pick():
    calls.push(1)
    return board
pick().entries.shift()
assert(calls.length, 2)
assert(board.entries, [1, 2])
values = [30, 10, 20]
count = values.value.push(9)
values = [5, 4]
assert(count, 3)
"#);
    for source in [
        "values@first.push(1)",
        "values@last.pop()",
        "values@0.shift()",
        "values@first.value.append(1)",
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{source}");
        assert_eq!(error.message(), "State versions are read-only", "{source}");
    }
    checked(&mut runtime, "assert(values@first, [30, 10, 20])", None);
}

#[test]
fn ordering_and_comparator_calls_are_unchanged() {
    run(r#"
calls = []
def compare(a, b):
    calls.push([a, b])
    return a - b
values = [3, 1, 2]
values.sort(compare)
assert(values, [1, 2, 3])
assert(calls, [[1, 3], [2, 1], [2, 3]])
calls = []
class Size(amount):
    this.amount = amount
Size(3); Size(1); Size(2)
def compareSize(a, b):
    calls.push([a.amount, b.amount])
    return a.amount - b.amount
assert(Size.sort(compareSize).map(size => size.amount), [1, 2, 3])
assert(calls, [[1, 3], [2, 1], [2, 3]])
assert(Size.map(size => size.amount), [3, 1, 2])
mixed = [10, 9, 1, "b", "a", "C", true, null, [2]]
mixed.sort()
assert(mixed, [1, 10, [2], 9, "C", "a", "b", null, true])
tied = ["1", 1, true, [1], "true", 1]
tied.sort()
assert(tied, ["1", 1, [1], 1, true, "true"])
people =[{ name: "a", age: 30 }, { name: "b", age: 20 }, { name: "c", age: 30 }, { name: "d", age: 20 }]
people.sort((x, y) => x.age - y.age)
assert(people.map(person => person.name), ["b", "d", "a", "c"])
ignored = [3, 1, 2]
ignored.sort(5)
assert(ignored, [1, 2, 3])
reversed = [1, 2, 3]
reversed.sort((a, b) => -1)
assert(reversed, [3, 2, 1])
kept = [3, 1, 2]
kept.sort((a, b) => a > b)
assert(kept, [3, 1, 2])
undefinedOrder = [3, 1, 2]
undefinedOrder.sort((a, b) => Number.NaN)
assert(undefinedOrder, [3, 1, 2])
empty = []
assert(empty.sort((a, b) => missing), [])
assert(empty@1, null)
"#);
}

#[test]
fn standing_sorts_rerun_once_per_change_and_follow_comparator_reads() {
    let runtime = run(r#"
direction = 1
scores = [3, 1, 2]
ranked = scores.sort((a, b) => (a - b) * direction)
assert(scores, [1, 2, 3])
scores.push(0)
assert(scores@0, [3, 1, 2])
assert(scores@1, [1, 2, 3])
assert(scores@2, [1, 2, 3, 0])
assert(scores@3, [0, 1, 2, 3])
assert(scores@4, null)
direction = -1
assert(scores, [3, 2, 1, 0])
assert(ranked, [3, 2, 1, 0])
scores = [5, 9]
assert(scores, [9, 5])
assert(ranked, [9, 5])
def compare(a, b) => a - b
named = [3, 1, 2]
ordered = named.sort(compare)
def compare(a, b) => b - a
assert(named, [3, 2, 1])
assert(ordered, [3, 2, 1])
flipped = [1, 2, 3]
flipping = flipped.sort((a, b) => -1)
assert(flipped, [3, 2, 1])
flipped.push(4)
assert(flipped, [4, 1, 2, 3])
values = [3, 1, 2]
values = values.sort()
values.push(0)
assert(values, [1, 2, 3, 0])
"#);
    assert_eq!(dependencies(&runtime, "ranked"), ["scores", "direction"]);
    assert_eq!(dependencies(&runtime, "ordered"), ["named", "compare"]);
    assert!(dependencies(&runtime, "values").is_empty());
}

#[test]
fn equal_sorts_append_no_version_and_wake_nothing_inside_a_cascade() {
    run(r#"
scores = [1, 2, 3]
log = []
watched = scores.length + log.push("run") * 0
assert(log.length, 1)
scores.sort()
assert(log.length, 2)
assert(scores@1, null)
source = [1, 2]
derived = source.slice()
audit = []
seen = derived.length + audit.push("run") * 0
ranked = derived.sort()
assert(audit.length, 2)
source.push(3)
assert(derived, [1, 2, 3])
assert(audit.length, 3)
"#);
}

#[test]
fn class_populations_keep_their_order_and_class_rules_sort_each_instance() {
    let mut runtime = run(r#"
class Item(size):
    this.size = size
Item(2); Item(1); Item(3)
smallest = Item.sort((x, y) => x.size - y.size)[0].size
assert(smallest, 1)
assert(Item[0].size, 2)
Item(0)
assert(smallest, 0)
assert(Item.map(item => item.size), [2, 1, 3, 0])
class Squad(scores):
    this.scores = scores
$Squad.best = $Squad.scores.sort((a, b) => b - a)[0]
red = Squad([3, 1, 2])
blue = Squad([5, 9])
assert(red.scores, [3, 2, 1])
assert(blue.scores, [9, 5])
red.scores.push(7)
assert(red.scores, [7, 3, 2, 1])
assert(red.best, 7)
assert(blue.scores, [9, 5])
"#);
    let error = runtime
        .run("$Squad.scores = $Squad.scores.sort()")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Circular Dependency");
    let error = runtime.run("Item.push(1)").unwrap_err();
    assert_eq!(error.message(), "Item.push is not a function");
}

#[test]
fn locals_parameters_and_loop_bindings_sort_only_their_binding() {
    run(r#"
def ordered(values):
    values.sort()
    return values
original = [3, 1, 2]
assert(ordered(original), [1, 2, 3])
assert(original, [3, 1, 2])
rows = [[3, 1], [2, 0]]
for row of rows:
    row.sort()
assert(rows, [[3, 1], [2, 0]])
"#);
}

#[test]
fn comparator_errors_and_failed_transactions_restore_state_and_graph() {
    let initial = r#"
def compare(a, b):
    if a > 100:
        throw "TOO_LARGE"
    return a - b
scores = [3, 1, 2]
ranked = scores.sort(compare)
assert(scores, [1, 2, 3])
"#;
    let mut actual = run(initial);
    let control = run(initial);
    for (source, thrown) in [
        ("scores.push(500)", "TOO_LARGE"),
        ("scores.sort((a, b) => { throw \"BAD\" })", "BAD"),
        ("scores = [9, 8]\nthrow \"ROLLBACK\"", "ROLLBACK"),
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
        "scores.push(0)\nassert(scores, [0, 1, 2, 3])\nassert(ranked, [0, 1, 2, 3])\nassert(scores@3, [0, 1, 2, 3])",
        None,
    );
}

#[test]
fn conflicting_standing_sorts_do_not_settle_and_roll_back() {
    let initial = r#"
scores = [2, 1]
up = scores.sort((a, b) => a - b)
down = scores.sort((a, b) => b - a)
"#;
    let mut actual = Runtime::new();
    actual.run(initial).unwrap();
    let mut control = Runtime::new();
    control.run(initial).unwrap();

    let error = actual.run("scores = [1, 3]").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Propagation did not settle");
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);

    checked(
        &mut actual,
        "down = 0\nscores = [3, 1, 2]\nassert(scores, [1, 2, 3])\nassert(up, [1, 2, 3])",
        None,
    );
}

#[test]
fn replacing_a_standing_sort_keeps_the_accepted_replacement_order() {
    // While a replacement's right-hand side runs, the replaced rule is still
    // filed, so the replacement's write re-runs it once, as for `push`.
    run(r#"
scores = [3, 1, 2]
ranked = scores.sort((a, b) => a - b)
ranked = scores.sort((a, b) => b - a)
assert(scores, [1, 2, 3])
assert(ranked, [3, 2, 1])
scores.push(0)
assert(scores, [3, 2, 1, 0])
assert(ranked, [3, 2, 1, 0])
arr = [1]
n = arr.push(2)
n = arr.push(3)
assert(arr, [1, 2, 3, 2])
"#);
}
