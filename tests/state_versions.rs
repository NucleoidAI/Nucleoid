//! Automatic storage history, additive @ syntax, and sealed accepted controls.
//! Ordinary tests do not open session artifacts, invoke Git, or evaluate NUCs.

mod common;
#[path = "support/state_version_cases.rs"]
mod version_cases;

use nucleoid::expression::Expression;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Ast, Expr, VersionSelector, parse, parse_expression};
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "state-version regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"))
}

fn error(runtime: &mut Runtime, source: &str, kind: ErrorKind, message: &str) {
    let error = runtime.run(source).unwrap_err();
    assert_eq!(error.kind(), kind, "{source}: {error}");
    assert_eq!(error.message(), message, "{source}: {error}");
    assert!(error.position().is_some(), "{source}: source diagnostic");
}

#[test]
fn nine_sealed_pairs_preserve_all_69_assertions_one_return_and_exposed_values() {
    let documents = version_cases::documents();
    let mut totals = (0, 0, 0);
    for (sealed, position) in version_cases::SEALED
        .iter()
        .zip([439, 440, 441, 442, 443, 444, 86, 35, 91])
    {
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == sealed.document)
            .unwrap()
            .1;
        assert_eq!(
            cases
                .iter()
                .filter(|case| case.title == sealed.title)
                .count(),
            1
        );
        let current = &cases[position];
        assert_eq!(current.title, sealed.title);
        assert_eq!(current.expected.as_deref(), sealed.expected);
        let documented = if matches!(sealed.document, "spec" | "reference") {
            format!("# {}\n\n{}", sealed.title, sealed.candidate)
        } else {
            sealed.candidate.to_string()
        };
        assert_eq!(current.source.trim(), documented);
        let assertions = common::assertions::protected_assertions(&parse(sealed.accepted).unwrap());
        assert_eq!(assertions.len(), sealed.assertions);
        assert_eq!(
            common::assertions::protected_assertions(&parse(&current.source).unwrap()),
            assertions
        );
        let original = common::Case {
            title: sealed.title.to_string(),
            source: sealed.accepted.to_string(),
            expected: sealed.expected.map(str::to_string),
        };
        let mut runtimes = [Runtime::new(), Runtime::new()];
        let mut returned = Vec::new();
        for (runtime, case) in runtimes.iter_mut().zip([&original, current]) {
            returned.push(
                common::check_with_count(runtime, case, sealed.assertions)
                    .unwrap_or_else(|reason| panic!("{}: {reason}", sealed.title)),
            );
            assert_eq!(
                runtime.assertions_run(),
                sealed.assertions,
                "{}",
                sealed.title
            );
        }
        version_cases::assert_observations(&runtimes[1], &runtimes[0], sealed);
        if sealed.expected.is_some() {
            assert_eq!(returned[0], returned[1]);
        }
        totals.0 += 1;
        totals.1 += sealed.assertions;
        totals.2 += usize::from(sealed.expected.is_some());
    }
    assert_eq!(totals, (9, 69, 1));
}

#[test]
fn exact_indices_aliases_contextual_words_and_ordinary_dot_fields_compose() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
first = 1
last = 2
previous = 3
def sum(first, last, previous) => first + last + previous
assert(sum(first, last, previous), 6)
record = {first: "given", last: "family", previous: "ordinary", list: [1, 2]}
record.first = "changed"
assert(record.first, "changed")
assert(record.last, "family")
assert(record.previous, "ordinary")
assert(record.first@0, "given")
assert(record.first@1, "changed")
assert(record.first@first, "given")
assert(record.first@last, "changed")
assert(record.first@previous, "given")
assert(record.first@2, null)
assert(record["first"]@0, "given")
assert(record.list@first[-1], 2)
assert(record.list@first[-2:-1], [1])
assert(record.list@first.slice(-2, -1), [1])
assert(record.list@first.map(item => item * 2), [2, 4])
assert(record.first@first.toUpperCase(), "GIVEN")
"#,
    );
    assert_eq!(tokenize("@").unwrap()[0].token, Token::At);
    let source = format!("value@{}", usize::MAX);
    assert_eq!(
        parse_expression(&source).unwrap(),
        Expr::Version {
            reference: Box::new(Expr::Identifier("value".to_string())),
            selector: VersionSelector::Index(usize::MAX),
        }
    );
    checked(&mut runtime, &format!("assert(first@{}, null)", usize::MAX));
    // Selector numbers are exact even above f64's integral precision.
    if usize::BITS >= 64 {
        assert!(matches!(
            parse_expression("first@9007199254740993").unwrap(),
            Expr::Version {
                selector: VersionSelector::Index(9_007_199_254_740_993),
                ..
            }
        ));
    }
}

#[test]
fn malformed_selectors_fail_explicitly_at_the_selector_without_changing_negative_subscripts() {
    let message = "Expected a non-negative integer, 'first', 'last', or 'previous' after '@'";
    for suffix in [
        "",
        "-1",
        "-2",
        "+1",
        "1.5",
        "1.0",
        ".5",
        "1e0",
        "unknown",
        "First",
        "last0",
        "0name",
        "true",
        "(0)",
        "[0]",
        "@0",
        "184467440737095516160",
    ] {
        let source = format!("value@{suffix}");
        let failure = Runtime::check(&source).unwrap_err();
        assert_eq!(failure.kind(), ErrorKind::Syntax, "{source}: {failure}");
        assert_eq!(failure.message(), message, "{source}: {failure}");
        assert_eq!(failure.position(), Some(Position::new(1, 7)), "{source}");
    }
    for source in ["value.@first", "value.@0"] {
        assert_eq!(
            Runtime::check(source).unwrap_err().kind(),
            ErrorKind::Syntax
        );
    }
    // The accepted lexer treats .0/.1 as numeric literals, never as history.
    for source in ["value.0", "value.1"] {
        assert!(matches!(
            tokenize(source).unwrap()[1].token,
            Token::Number(_)
        ));
    }
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
items = [1, 2, 3]
assert(items[-1], 3)
assert(items[-2], 2)
assert(items[-2:-1], [2])
assert(items.slice(-2, -1), [2])
literal = "name@-1"
assert(literal, "name@-1")
"#,
    );
}

#[test]
fn histories_persist_across_commits_and_readers_never_append_equal_source_values() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "input = 1\namount = input * 2\nfirst = amount@first\nlatest = amount@last\nprevious = amount@previous",
    );
    checked(
        &mut runtime,
        "input = 2\ninput = 2\nassert(amount, 4)\nassert(first, 2)\nassert(latest, 4)\nassert(previous, 2)",
    );
    for _ in 0..8 {
        checked(&mut runtime, "amount@first\namount@last\namount@previous");
    }
    checked(
        &mut runtime,
        "assert(amount@0, 2)\nassert(amount@1, 4)\nassert(amount@2, null)\nassert(first@1, null)\nassert(latest@1, 4)",
    );
    checked(
        &mut runtime,
        "input = 3\nassert(amount@2, 6)\nassert(previous, 4)\nassert(first@1, null)",
    );
    assert!(
        runtime
            .graph
            .retrieve(&NodeKey::variable("first"))
            .unwrap()
            .dependencies
            .contains(&NodeKey::variable("amount"))
    );
}

#[test]
fn missing_slots_indices_and_previous_recover_without_fabricating_current_only_histories() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "old = absent@first\nlatest = absent@last\nprevious = absent@previous\nassert(old, null)\nassert(latest, null)\nassert(previous, null)",
    );
    checked(
        &mut runtime,
        "absent = 10\nassert(old, 10)\nassert(latest, 10)\nassert(previous, null)",
    );
    checked(
        &mut runtime,
        "absent = 20\nassert(old, 10)\nassert(latest, 20)\nassert(previous, 10)",
    );
    checked(
        &mut runtime,
        "holder = {}\nfuture = holder.item@1\noriginal = holder.item@first\nassert(future, null)\nassert(original, null)",
    );
    checked(
        &mut runtime,
        "holder.item = 7\nassert(original, 7)\nassert(future, null)\nholder.item = 8\nassert(future, 8)",
    );
    checked(
        &mut runtime,
        "delete holder.item\nassert(original, null)\nassert(future, null)\nholder.item = 99\nassert(original, 99)\nassert(future, null)\nassert(holder.item@previous, null)",
    );
    assert!(
        runtime
            .graph
            .retrieve(&NodeKey::variable("future"))
            .unwrap()
            .dependencies
            .iter()
            .any(|key| key.as_str().ends_with(".item"))
    );
}

#[test]
fn valid_old_values_survive_current_null_undefined_and_snapshot_freezing() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
value = "first"
original = value@first
latest = value@last
saved = value@last.value
value = null
assert(original, "first")
assert(latest, null)
assert(saved, "first")
assert(value@first.toUpperCase(), "FIRST")
value = "third"
assert(value@previous, null)
assert(original, "first")
assert(latest, "third")
assert(saved, "first")
holder = {item: 7}
missing = {}
[holder.item = missing.item]
"#,
    );
    let Value::Object(id) = runtime.state.variable("holder").unwrap() else {
        panic!()
    };
    assert_eq!(runtime.state.property(id, "item"), Some(&Value::Undefined));
    checked(
        &mut runtime,
        "oldProperty = holder.item@0\nnewProperty = holder.item@last\nassert(oldProperty, 7)\nassert(newProperty, null)",
    );
    assert_eq!(
        runtime.state.variable("oldProperty"),
        Some(&Value::Number(7.0))
    );
    assert!(
        runtime
            .graph
            .retrieve(&NodeKey::variable("saved"))
            .unwrap()
            .dependencies
            .is_empty()
    );
    checked(
        &mut runtime,
        "holder.item = 8\nassert(oldProperty, 7)\nassert(newProperty, 8)\nassert(holder.item@1, null)\nassert(holder.item@2, 8)",
    );
}

#[test]
fn underlying_property_owners_switch_with_aliases_and_dependencies_drop_the_old_owner() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
left = {reading: 1}
right = {reading: 10}
left.reading = 2
selected = left
original = selected.reading@first
latest = selected["reading"]@last
assert(original, 1)
assert(latest, 2)
selected = right
assert(original, 10)
assert(latest, 10)
right.reading = 11
assert(original, 10)
assert(latest, 11)
left.reading = 3
assert(latest, 11)
assert(left.reading@previous, 2)
assert(right.reading@previous, 10)
"#,
    );
    let Value::Object(left) = runtime.state.variable("left").unwrap() else {
        panic!()
    };
    let Value::Object(right) = runtime.state.variable("right").unwrap() else {
        panic!()
    };
    let dependencies = &runtime
        .graph
        .retrieve(&NodeKey::variable("latest"))
        .unwrap()
        .dependencies;
    assert!(!dependencies.contains(&NodeKey::property(left, "reading")));
    assert!(dependencies.contains(&NodeKey::property(right, "reading")));
    checked(
        &mut runtime,
        "assert(original@0, 1)\nassert(original@1, 10)\nassert(original@2, null)",
    );
}

#[test]
fn initial_maps_constructors_parameters_this_self_and_class_rule_receivers_record_real_values() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
plain = {amount: 3}
assert(plain.amount@first, 3)
class Meter(seed):
    this.seed = seed
    self.seed = seed + 1
    this.parameterFirst = seed@first
    seed = seed + 5
    this.parameterPrevious = seed@previous
    this.parameterLast = seed@last
    def increase():
        this.seed = this.seed + 1
        return self.seed@previous
$Meter.initial = $Meter.seed@first
$Meter.latest = $Meter.seed@last
first = Meter(2)
second = Meter(10)
assert(first.seed@0, 2)
assert(first.seed@1, 3)
assert(first.seed@2, null)
assert(first.initial, 2)
assert(first.latest, 3)
assert(second.initial, 10)
assert(second.latest, 11)
assert(first.parameterFirst, 2)
assert(first.parameterPrevious, 2)
assert(first.parameterLast, 7)
before = first.increase()
assert(before, 3)
assert(first.seed@2, 4)
assert(first.initial, 2)
assert(first.latest, 4)
assert(second.latest, 11)
assert(first.id@first, "first")
"#,
    );
}

#[test]
fn local_assignment_expressions_parameters_missing_arguments_and_call_isolation_keep_histories() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
value = 99
def capture(value) => [value@first, value = value + 1, value@previous, value@last, value@2]
one = capture(2)
two = capture(10)
assert(one, [2, 3, 2, 3, null])
assert(two, [10, 11, 10, 11, null])
assert(value@first, 99)
def local() => [temporary = 4, temporary = 5, temporary@first, temporary@last, temporary@previous]
assert(local(), [4, 5, 4, 5, 4])
assert(local(), [4, 5, 4, 5, 4])
assert(temporary@first, null)
def omitted(value) => [value@first, value@previous]
assert(omitted(), [null, null])
"#,
    );
}

#[test]
fn loop_and_catch_bindings_shadow_and_expire_using_existing_frame_lifetimes() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
character = "outer"
error = "outer error"
seen = []
for character of "ab":
    original = character@first.value
    character = character + "!"
    seen.push([original, character@last, character@previous, character@2])
assert(seen, [["a", "a!", "a", null], ["b", "b!", "b", null]])
assert(character@first, "outer")
try:
    throw "caught"
catch error:
    assert(error@first, "caught")
    error = "handled"
    assert(error@previous, "caught")
    assert(error@last, "handled")
assert(error@first, "outer error")
assert(error@previous, null)
"#,
    );
}

#[test]
fn scope_clones_and_innermost_slots_are_independent_and_end_on_pop() {
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.declare("value", Value::Number(1.0));
    scope.assign("value", Value::Number(2.0));
    scope.push();
    scope.declare("value", Value::Number(10.0));
    scope.assign("value", Value::Number(11.0));
    let read = |runtime: &mut Runtime, scope: &mut Scope, source| {
        Ast::convert(&parse_expression(source).unwrap())
            .resolve(runtime, scope)
            .unwrap()
    };
    assert_eq!(
        read(&mut runtime, &mut scope, "value@first"),
        Value::Number(10.0)
    );
    let mut cloned = scope.clone();
    cloned.assign("value", Value::Number(12.0));
    assert_eq!(
        read(&mut runtime, &mut cloned, "value@last"),
        Value::Number(12.0)
    );
    assert_eq!(
        read(&mut runtime, &mut scope, "value@last"),
        Value::Number(11.0)
    );
    scope.pop();
    assert_eq!(
        read(&mut runtime, &mut scope, "value@first"),
        Value::Number(1.0)
    );
    assert_eq!(
        read(&mut runtime, &mut scope, "value@last"),
        Value::Number(2.0)
    );
}

#[test]
fn history_follows_existing_nontransactional_local_values_without_promoting_them_to_state() {
    // Accepted Try rolls back State/Graph, not Scope locals. Keep that language
    // behavior: a local's history and current value must not disagree.
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
def keep(value):
    try:
        value = value + 1
        throw "stop"
    catch error:
        assert(error, "stop")
    return [value, value@first, value@last, value@previous]
assert(keep(3), [4, 3, 4, 3])
"#,
    );
}

#[test]
fn list_writebacks_record_value_copies_and_equal_mutations_add_nothing() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
items = [1]
items.push(2)
items.append()
items.pop()
items.shift()
items.pop()
assert(items@0, [1])
assert(items@1, [1, 2])
assert(items@2, [1])
assert(items@3, [])
assert(items@4, null)
holder = {items: [3]}
holder.items.push(4)
assert(holder.items@first, [3])
assert(holder.items@last, [3, 4])
def local(items) => [items.push(8), items@first, items@last]
assert(local([7]), [2, [7], [7, 8]])
copy = holder.items@first.value
copy.push(9)
assert(copy, [3, 9])
assert(holder.items@first, [3])
assert(holder.items@last, [3, 4])
assert(holder.items@2, null)
"#,
    );
}

#[test]
fn historical_list_queries_capture_receivers_before_arguments_and_preserve_retained_lists() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
items = [[1], 2]
items.push(3)
def append():
    items.push(4)
    return [5]
assert(items@last.concat(append()), [[1], 2, 3, 5])
assert(items@last, [[1], 2, 3, 4])
assert(items@previous, [[1], 2, 3])
assert(items@first.reverse(), [2, [1]])
assert(items@first, [[1], 2])
assert(items@3, null)
"#,
    );
}

#[test]
fn returned_object_identity_is_writable_but_history_slots_and_historical_lists_are_not() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
first = {label: "old", items: [1], count: 1}
second = {label: "new"}
selected = first
selected = second
selected@first.label = "updated"
selected@first.count += 2
selected@first.items.push(2)
selected@first["items"].push(3)
assert(first.label, "updated")
assert(first.count, 3)
assert(first.items, [1, 2, 3])
assert(first.items@first, [1])
assert(first.items@last, [1, 2, 3])
assert(selected@0, first)
assert(selected@previous, first)
assert(selected@last, second)
delete selected@first.label
assert(first.label, null)
items = [[1], 2]
items.push(3)
"#,
    );
    for source in [
        "selected@0 = second",
        "selected@last += 1",
        "selected@first &&= true",
        "delete selected@first",
        "[delete selected@0]",
        "items@first.push(4)",
        "items@last.append(4)",
        "items@0.pop()",
        "items@0.shift()",
        "items@0[0].push(4)",
        "items@0[:].push(4)",
        "items@first.value.push(4)",
    ] {
        error(
            &mut runtime,
            source,
            ErrorKind::Type,
            "State versions are read-only",
        );
    }
    checked(
        &mut runtime,
        "assert(items@0, [[1], 2])\nassert(items@1, [[1], 2, 3])\nassert(items@2, null)",
    );
    let json = runtime
        .serialize_json(runtime.state.variable("first").unwrap())
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).unwrap(),
        serde_json::json!({"items":[1.0,2.0,3.0],"count":3.0})
    );
}

#[test]
fn side_effecting_receivers_and_computed_keys_run_once_including_real_property_list_writes() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
calls = 0
keys = 0
holder = {reading: 4, stored: {items: [1]}}
holder.reading = 5
def receiver():
    calls = calls + 1
    return holder
def key():
    keys = keys + 1
    return "reading"
"#,
    );
    assert_eq!(
        checked(&mut runtime, "receiver().reading@first"),
        Value::Number(4.0)
    );
    assert_eq!(runtime.state.variable("calls"), Some(&Value::Number(1.0)));
    assert_eq!(
        checked(&mut runtime, "receiver()[key()]@previous"),
        Value::Number(4.0)
    );
    assert_eq!(runtime.state.variable("calls"), Some(&Value::Number(2.0)));
    assert_eq!(runtime.state.variable("keys"), Some(&Value::Number(1.0)));
    checked(&mut runtime, "receiver().stored@first.items.push(2)");
    assert_eq!(runtime.state.variable("calls"), Some(&Value::Number(3.0)));
    checked(
        &mut runtime,
        "assert(holder.stored.items, [1, 2])\nassert(holder.stored.items@first, [1])",
    );
}

#[test]
fn same_value_semantics_cover_nan_signed_zero_lists_dates_regexes_and_object_identity() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
number = 0 / 0
number = 0 / 0
assert(Number.isNaN(number@first), true)
assert(number@1, null)
zero = 0
zero = -0
zero = -0
assert(zero@2, null)
list = [0 / 0, -0]
list = [0 / 0, -0]
assert(list@1, null)
list = [0 / 0, 0]
assert(list@2, null)
date = Date(0)
date = Date(0)
assert(date@1, null)
regex = /a/
regex = /a/
assert(regex@1, null)
first = {same: true}
second = {same: true}
chosen = first
chosen = first
chosen = second
assert(chosen@0, first)
assert(chosen@1, second)
assert(chosen@2, null)
"#,
    );
    let Value::Number(first) = checked(&mut runtime, "zero@0") else {
        panic!()
    };
    let Value::Number(last) = checked(&mut runtime, "zero@1") else {
        panic!()
    };
    assert_eq!(first.to_bits(), 0.0_f64.to_bits());
    assert_eq!(last.to_bits(), (-0.0_f64).to_bits());
    let old = checked(&mut runtime, "chosen@0");
    assert_ne!(&old, runtime.state.variable("second").unwrap());
}

#[test]
fn functions_and_classes_stored_as_values_use_normal_semantics_and_namespaces_have_no_fake_slots() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
callable = value => value + 1
callable = value => value + 2
assert(callable@first(3), 4)
assert(callable@last(3), 5)
assert(callable@previous(3), 4)
class Record
stored = Record
assert(stored@first, Record)
Math = 7
assert(Math@first, 7)
def named(value) => value
"#,
    );
    for source in [
        "Record@first",
        "named@0",
        "Number@0",
        "List@last",
        "1@0",
        "[1]@0",
        "({p:1})@0",
        "callable()@0",
        "[1][0]@0",
        "\"x\".length@0",
        "stored.length@0",
        "stored.value@0",
    ] {
        error(
            &mut runtime,
            source,
            ErrorKind::Type,
            "State versions require a variable or object property",
        );
    }
    error(
        &mut runtime,
        "stored@first()",
        ErrorKind::Type,
        "stored@first is not a function",
    );
    checked(
        &mut runtime,
        "callable = callable.value\nassert(callable@2, null)",
    );
}

#[test]
fn failed_runs_and_nested_savepoints_undo_appends_removals_and_delete_recreate_resets() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
value = "A"
value = "B"
holder = {reading: 1, items: [1]}
holder.reading = 2
first = value@first
latest = value@last
"#,
    );
    error(
        &mut runtime,
        r#"
value = "discarded"
holder.items.push(2)
delete value
value = "new lifetime"
delete holder.reading
holder.reading = 99
throw "fail"
"#,
        ErrorKind::Thrown,
        "fail",
    );
    checked(
        &mut runtime,
        r#"
assert(value@0, "A")
assert(value@1, "B")
assert(value@2, null)
assert(holder.reading@0, 1)
assert(holder.reading@1, 2)
assert(holder.reading@2, null)
assert(holder.items@0, [1])
assert(holder.items@1, null)
assert(first, "A")
assert(latest, "B")
try:
    value = "outer"
    try:
        delete value
        value = "inner"
        holder.reading = 3
        throw "inner failure"
    catch inner:
        assert(inner, "inner failure")
    assert(value@last, "outer")
    assert(value@previous, "B")
    holder.reading = 4
    throw "outer failure"
catch outer:
    assert(outer, "outer failure")
assert(value@last, "B")
assert(value@2, null)
assert(holder.reading@last, 2)
assert(holder.reading@2, null)
value = "C"
assert(value@2, "C")
"#,
    );
}

#[test]
fn function_namespace_transitions_and_object_recreation_restore_histories_on_failure() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
callable = value => value + 1
class Record
record = Record()
record.label = "old"
record.label = "current"
try:
    def callable(value) => value + 100
    record = Record()
    record.label = "discarded"
    throw "reset"
catch error:
    assert(error, "reset")
assert(callable@0(2), 3)
assert(callable@1, null)
assert(record.label@first, "old")
assert(record.label@last, "current")
assert(record.label@2, null)
"#,
    );
    checked(&mut runtime, "def callable(value) => value + 10");
    error(
        &mut runtime,
        "callable@0",
        ErrorKind::Type,
        "State versions require a variable or object property",
    );
    checked(
        &mut runtime,
        "callable = value => value + 20\nassert(callable@0(2), 22)\nassert(callable@previous, null)",
    );
    checked(
        &mut runtime,
        "delete record.label\ndelete record\nrecord = Record()\nrecord.label = \"fresh\"\nassert(record.label@first, \"fresh\")\nassert(record.label@previous, null)\nassert(record@previous, null)",
    );
}

#[test]
fn retention_has_no_sliding_window_and_both_clear_operations_reset_it() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, "value = 0");
    for value in 1..=300 {
        checked(&mut runtime, &format!("value = {value}"));
    }
    checked(
        &mut runtime,
        "assert(value@0, 0)\nassert(value@1, 1)\nassert(value@150, 150)\nassert(value@300, 300)\nassert(value@301, null)",
    );
    runtime.state.clear();
    checked(
        &mut runtime,
        "value = 400\nassert(value@first, 400)\nassert(value@previous, null)",
    );
    runtime.clear();
    checked(
        &mut runtime,
        "value = 500\nassert(value@first, 500)\nassert(value@previous, null)\nassert(value@1, null)",
    );
}

#[test]
fn read_only_failures_are_catchable_and_restore_all_attempted_state_writes() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "value = 1\nvalue = 2\nitems = [1]\neffects = 0",
    );
    for invalid in [
        "value@first = 99",
        "value@0 += 1",
        "value@previous ||= 1",
        "[value@last = 9]",
        "delete value@0",
        "items@first.push(2)",
    ] {
        let source = format!(
            "try:\n    value = 3\n    effects = 1\n    {invalid}\ncatch error:\n    \
             assert(error, TypeError(\"State versions are read-only\"))\n\
             assert(value, 2)\nassert(value@last, 2)\nassert(value@previous, 1)\n\
             assert(value@2, null)\nassert(effects, 0)\nassert(effects@1, null)\n\
             assert(items@0, [1])\nassert(items@1, null)"
        );
        checked(&mut runtime, &source);
    }
}

#[test]
fn successful_inner_delete_recreate_is_still_undone_by_enclosing_failure() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
score = 1
score = 2
holder = {label: "A"}
holder.label = "B"
try:
    score = 3
    try:
        delete score
        score = 9
        delete holder.label
        holder.label = "new lifetime"
    catch unused:
        pass
    assert(score@first, 9)
    assert(score@previous, null)
    assert(holder.label@first, "new lifetime")
    assert(holder.label@previous, null)
    throw "outer"
catch error:
    assert(error, "outer")
assert(score@0, 1)
assert(score@1, 2)
assert(score@2, null)
assert(holder.label@0, "A")
assert(holder.label@1, "B")
assert(holder.label@2, null)
"#,
    );
}

#[test]
fn live_history_readers_defer_when_a_named_function_replaces_variable_storage_and_recover() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
callable = value => value + 1
oldest = callable@first
result = callable@first(3)
assert(result, 4)
try:
    def callable(value) => value + 10
    assert(oldest, null)
    assert(result, null)
    throw "restore"
catch error:
    assert(error, "restore")
assert(result, 4)
assert(oldest(3), 4)
assert(callable@1, null)
def callable(value) => value + 10
assert(oldest, null)
assert(result, null)
assert(callable(3), 13)
callable = value => value + 20
assert(oldest(3), 23)
assert(result, 23)
assert(callable@previous, null)
"#,
    );
}

#[test]
fn historical_object_calls_and_nested_property_versions_keep_the_real_receiver() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Counter(seed):
    this.count = seed
    def increment():
        this.count += 1
        return this.count
first = Counter(1)
second = Counter(10)
selected = first
selected = second
"#,
    );
    assert_eq!(
        checked(&mut runtime, "selected@first.increment()"),
        Value::Number(2.0)
    );
    checked(
        &mut runtime,
        r#"
assert(first.count, 2)
assert(second.count, 10)
assert(selected@first.count@first, 1)
assert(selected@first.count@last, 2)
selected = null
assert(selected@first.count, 2)
assert(selected@previous.count, 10)
assert(selected@first.count@previous, 1)
"#,
    );
}

#[test]
fn getters_do_not_invent_successful_histories_for_untracked_embedding_writes() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, "holder = {}");
    let Value::Object(id) = runtime.state.variable("holder").unwrap().clone() else {
        panic!()
    };
    // This existing embedding escape hatch bypasses Runtime writes. A getter
    // must not turn the current property into a fabricated retained version.
    runtime
        .state
        .object_mut(&id)
        .unwrap()
        .properties
        .insert("untracked".to_string(), Value::Number(42.0));
    assert_eq!(
        checked(&mut runtime, "holder.untracked"),
        Value::Number(42.0)
    );
    assert_eq!(
        checked(&mut runtime, "holder.untracked@first"),
        Value::Undefined
    );
    assert_eq!(
        checked(&mut runtime, "holder.untracked@last"),
        Value::Undefined
    );
    assert_eq!(
        runtime.state.property(&id, "untracked"),
        Some(&Value::Number(42.0))
    );
}

#[test]
fn local_delete_keeps_accepted_global_targeting_and_does_not_reset_the_parameter_slot() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
value = 99
def keep(value):
    delete value
    return [value, value@first, value@last, value@previous]
result = keep(2)
assert(result, [2, 2, 2, null])
assert(value@first, null)
value = 100
assert(value@0, 100)
assert(value@previous, null)
"#,
    );
}

#[test]
fn cycles_still_fail_and_rollback_history_while_one_shot_reads_do_not_replay() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, "source = 1\nfirst = source@first");
    error(
        &mut runtime,
        "source = first + 1",
        ErrorKind::Type,
        "Circular Dependency",
    );
    checked(
        &mut runtime,
        "assert(source@first, 1)\nassert(source@1, null)\nassert(first@1, null)",
    );
    checked(
        &mut runtime,
        r#"
calls = 0
holder = {reading: 4}
def probe():
    calls = calls + 1
    return holder
"#,
    );
    assert_eq!(
        checked(&mut runtime, "probe().reading@first"),
        Value::Number(4.0)
    );
    checked(
        &mut runtime,
        "holder.reading = 5\nassert(calls, 1)\nassert(holder.reading@previous, 4)",
    );
}

#[test]
fn class_rule_reference_checks_allow_deferred_slots_but_keep_strict_receivers_and_cycles() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Meter
$Meter.saved = future@first
first = Meter()
second = Meter()
assert(first.saved, null)
assert(second.saved, null)
future = 7
assert(first.saved, 7)
assert(second.saved, 7)
future = 8
assert(first.saved, 7)
assert(second.saved, 7)
"#,
    );
    error(
        &mut runtime,
        "$Meter.bad = missing.property@first",
        ErrorKind::Reference,
        "missing is not defined",
    );
    error(
        &mut runtime,
        "$Meter.bad = future@first + missing",
        ErrorKind::Reference,
        "missing is not defined",
    );
    error(
        &mut runtime,
        "$Meter.loop = $Meter.loop@first",
        ErrorKind::Type,
        "Circular Dependency",
    );
}

#[test]
fn expression_visitors_generation_freezing_limits_and_cli_diagnostics_include_versions() {
    let expression = parse_expression("student.name@previous").unwrap();
    assert_eq!(expression.to_string(), "student.name@previous");
    assert_eq!(
        parse_expression(&expression.to_string()).unwrap(),
        expression
    );
    assert_eq!(expression.path(), None, "not an ordinary dot property");
    assert_eq!(expression.last(), None);
    assert!(matches!(expression.first(), Some(Expr::Identifier(name)) if name == "student"));
    assert!(matches!(expression.object(), Some(Expr::Member { .. })));
    assert_eq!(Expression::new(&expression).roots(), ["student"]);
    let class = parse_expression("$Meter.total@first").unwrap();
    assert_eq!(
        Expression::new(&class).class_reference().as_deref(),
        Some("Meter")
    );
    assert_eq!(Expression::new(&class).class_properties("Meter"), ["total"]);
    let reasoning = parse_expression("(amount |> why).holds@first").unwrap();
    assert!(reasoning.contains_reasoning());
    let nested = parse("record[assert(1, 1)]@0").unwrap();
    assert_eq!(
        common::assertions::protected_assertions(&nested),
        ["assert(1,1)"]
    );
    let too_many = format!("value{}", "@first".repeat(129));
    assert_eq!(
        Runtime::check(&too_many).unwrap_err().message(),
        "Expression has too many parts"
    );

    use std::io::Write as _;
    use std::process::{Command, Stdio};
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
        .write_all(b"value = 1\nvalue = 2\nvalue@first\nvalue@last\nvalue@-1\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .replace("\r\n", "\n");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stdout.contains("1\n") && stdout.contains("2\n"), "{stdout}");
    assert!(
        stderr.contains("Expected a non-negative integer"),
        "{stderr}"
    );
    assert!(stderr.contains("line 1, column 7"), "{stderr}");
}
