//! Parser-lowered logical assignments keep the expanded runtime behavior.

mod common;

use nucleoid::Runtime;
use nucleoid::graph::NodeKey;

fn checked(source: &str, assertions: usize, expected_return: Option<&str>) -> Runtime {
    let mut runtime = Runtime::new();
    let case = common::Case {
        title: "logical compound regression".into(),
        source: source.into(),
        expected: expected_return.map(str::to_owned),
    };
    common::check_with_count(&mut runtime, &case, assertions).unwrap();
    assert_eq!(runtime.assertions_run(), assertions);
    runtime
}

#[test]
fn logical_compounds_short_circuit_and_return_operands_without_boolean_coercion() {
    checked(
        r#"
def fail():
    throw "EVALUATED"
retained = "ready"
retained ||= fail()
assert(retained, "ready")
stopped = 0
stopped &&= fail()
assert(stopped, 0)
chosen = false
chosen ||= 12
assert(chosen, 12)
chosen &&= 4
assert(chosen, 4)
text = ""
text ||= "ready"
assert(text, "ready")
text &&= "done"
assert(text, "done")
caught = null
try:
    stopped ||= fail()
catch error:
    caught = error
assert(caught, "EVALUATED")
assert(stopped, 0)
caught = null
try:
    retained &&= fail()
catch error:
    caught = error
assert(caught, "EVALUATED")
assert(retained, "ready")
empty = null
empty ||= 12
assert(empty, null)
empty &&= 12
assert(empty, null)
"#,
        12,
        None,
    );
}

#[test]
fn even_short_circuited_compounds_replace_the_old_standing_relationship() {
    let runtime = checked(
        r#"
source = true
enabled = source
enabled ||= false
source = false
assert(enabled, true)
other = false
disabled = other
disabled &&= true
other = true
assert(disabled, false)
"#,
        2,
        None,
    );
    for (target, discarded) in [("enabled", "source"), ("disabled", "other")] {
        let node = runtime.graph.retrieve(&NodeKey::variable(target)).unwrap();
        assert!(!node.dependencies.contains(&NodeKey::variable(discarded)));
    }
}

#[test]
fn logical_compounds_keep_snapshot_freezing_even_in_a_skipped_logical_operand() {
    checked(
        r#"
snapshot = false
source = true
snapshot ||= source.value
source = false
assert(snapshot, true)
retained = true
caught = null
try:
    retained ||= missing.value
catch error:
    caught = error
assert(caught, ReferenceError("missing is not defined"))
assert(retained, true)
stopped = false
caught = null
try:
    stopped &&= missing.value
catch error:
    caught = error
assert(caught, ReferenceError("missing is not defined"))
assert(stopped, false)
"#,
        5,
        None,
    );
}

#[test]
fn logical_compounds_preserve_receivers_locals_nested_returns_and_class_cycles() {
    checked(
        r#"
class Gate:
    def allow(fallback):
        this.enabled ||= fallback
        return this.enabled
    def require(allowed):
        self.enabled &&= allowed
        return self.enabled
gate = Gate()
gate.enabled = false
assert(gate.allow(true), true)
assert(gate.require(false), false)
assert(gate.enabled, false)
def combined(a, b) => a ||= b &&= 9
assert(combined(false, 2), 9)
assert(combined(true, 2), true)
assert(combined(false, 0), 0)
class Shared
shared = Shared()
$Shared.enabled = true
try:
    $Shared.enabled ||= false
catch error:
    assert(error, TypeError("Circular Dependency"))
assert(shared.enabled, true)
$Shared.enabled = false
try:
    $Shared.enabled &&= true
catch error:
    assert(error, TypeError("Circular Dependency"))
assert(shared.enabled, false)
combined(false, 2)
"#,
        10,
        Some("9"),
    );
}

#[test]
fn failed_logical_compounds_restore_the_previous_dependency_and_queued_work() {
    checked(
        r#"
source = false
enabled = source
dependent = enabled || false
enforce = true
if enforce && enabled:
    throw "BLOCKED"
caught = null
try:
    enabled ||= true
catch error:
    caught = error
assert(caught, "BLOCKED")
assert(enabled, false)
assert(dependent, false)
enforce = false
source = true
assert(enabled, true)
assert(dependent, true)
limit = true
ready = limit
required = ready && true
enforceRequired = true
if enforceRequired && !ready:
    throw "REQUIRED"
caught = null
try:
    ready &&= false
catch error:
    caught = error
assert(caught, "REQUIRED")
assert(ready, true)
assert(required, true)
enforceRequired = false
limit = false
assert(ready, false)
assert(required, false)
"#,
        10,
        None,
    );
}

#[test]
fn compact_and_expanded_logical_relationships_have_identical_graphs_and_returns() {
    let mut compact = Runtime::new();
    let mut expanded = Runtime::new();
    for (compact_source, expanded_source) in [
        (
            "enabled = false\nfallback = false",
            "enabled = false\nfallback = false",
        ),
        (
            "if true:\n    enabled ||= fallback",
            "if true:\n    enabled = enabled || fallback",
        ),
        ("fallback = true", "fallback = true"),
        ("fallback = false", "fallback = false"),
        ("allowed = true", "allowed = true"),
        ("allowed &&= enabled", "allowed = allowed && enabled"),
        ("why enabled", "why enabled"),
        ("model |> why", "model |> why"),
        ("enabled = false", "enabled = false"),
        ("why allowed", "why allowed"),
        (
            "enabled ||= fallback.value",
            "enabled = enabled || fallback.value",
        ),
        ("fallback = true", "fallback = true"),
        ("why enabled", "why enabled"),
    ] {
        let compact_value = compact.run(compact_source).unwrap();
        let expanded_value = expanded.run(expanded_source).unwrap();
        assert_eq!(
            compact.serialize_json(&compact_value).unwrap(),
            expanded.serialize_json(&expanded_value).unwrap(),
            "{compact_source}"
        );
        assert!(
            compact.graph.keys().eq(expanded.graph.keys()),
            "{compact_source}"
        );
        for key in compact.graph.keys() {
            let before = expanded.graph.retrieve(key).unwrap();
            let after = compact.graph.retrieve(key).unwrap();
            assert_eq!(before.kind, after.kind, "{key}");
            assert_eq!(before.dependencies, after.dependencies, "{key}");
            assert_eq!(before.dependents, after.dependents, "{key}");
        }
    }
}

#[test]
fn legacy_assignments_word_logicals_and_arithmetic_compounds_remain_supported() {
    checked(
        r#"
value = 0
value = value || 7
assert(value, 7)
value = value && 9
assert(value, 9)
word = 0
word = word or 8
word = word and 11
assert(word, 11)
amount = 20
amount += 4
amount -= 6
amount *= 3
amount /= 3
amount %= 5
assert(amount, 3)
"#,
        4,
        None,
    );
}
