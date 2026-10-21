//! Opt-in named binding is a constructor capability, not a new function-call
//! convention. Edge/error cases stay out of the measured document corpus.

mod common;

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{ConstructorBinding, Expr, Stmt, parse, parse_expression, parse_program};
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn checked(source: &str) -> Runtime {
    let expected = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let mut runtime = Runtime::new();
    runtime.run(source).unwrap();
    assert_eq!(runtime.assertions_run(), expected, "{source}");
    let failures = runtime.take_assertions();
    assert!(failures.is_empty(), "{source}\n{failures:?}");
    runtime
}

#[test]
fn only_class_headers_opt_into_a_concrete_binding_mode() {
    for header in ["class Pair(*)", "class Pair(*,)", "class Pair(\n    *,\n)"] {
        let statements = parse(header).unwrap();
        let [Stmt::Class(class)] = statements.as_slice() else {
            panic!("expected a class");
        };
        assert_eq!(class.constructor_binding, ConstructorBinding::Named);
        assert!(class.parameters.is_empty());
        assert_eq!(generate_all(&statements), "class Pair(*)");
    }
    let program = parse_program("class Pair(*, a: int, b)\nnext = 1").unwrap();
    let Stmt::Class(class) = &program.statements[0] else {
        panic!("expected a class");
    };
    assert_eq!(class.parameters[0].name, "a");
    assert_eq!(class.parameters[0].type_name.as_deref(), Some("int"));
    assert_eq!(class.parameters[1].name, "b");
    assert_eq!(class.parameters[1].type_name, None);
    assert!(format!("{class:?}").contains("constructor_binding: Named"));
    assert_eq!(
        generate_all(&program.statements),
        "class Pair(*,a:int,b);next=1"
    );
    assert_eq!(
        program.positions,
        [Position::new(1, 1), Position::new(2, 1)]
    );

    for header in ["class Pair", "class Pair()", "class Pair(a, b: int)"] {
        let statements = parse(header).unwrap();
        let Stmt::Class(class) = &statements[0] else {
            panic!("expected a class");
        };
        assert_eq!(class.constructor_binding, ConstructorBinding::Positional);
        assert!(!format!("{class:?}").contains("constructor_binding"));
        assert_eq!(generate_all(&statements), "class Pair");
    }
    assert_eq!(
        nucleoid::state::ClassData::new("Pair").constructor_binding,
        ConstructorBinding::Positional
    );

    let call = parse_expression("Pair(b = 2, a = 1)").unwrap();
    let Expr::Call { arguments, .. } = &call else {
        panic!("expected the unchanged call AST");
    };
    assert!(
        arguments
            .iter()
            .all(|argument| matches!(argument, Expr::Assign { .. }))
    );
    assert_eq!(call.to_string(), "Pair(b=2,a=1)");
    assert_eq!(call, parse_expression("new Pair(b=2, a=1)").unwrap());
}

#[test]
fn malformed_new_headers_have_precise_errors_and_locations() {
    for (source, message, position) in [
        (
            "class Pair(* a)",
            "Expected ',' or ')' after constructor '*'",
            Position::new(1, 14),
        ),
        (
            "class Pair(*, *, a)",
            "Constructor '*' must appear once, before all parameters",
            Position::new(1, 15),
        ),
        (
            "class Pair(*, a, a)",
            "Duplicate named constructor parameter 'a'",
            Position::new(1, 18),
        ),
        (
            "class Pair(a, *)",
            "Expected a parameter but found '*'",
            Position::new(1, 16),
        ),
        (
            "class Pair(*, a = 1)",
            "Expected ')' but found '='",
            Position::new(1, 17),
        ),
        (
            "class Pair(*):\n    def init(a, a):\n        this.a = a",
            "Duplicate named constructor parameter 'a'",
            Position::new(2, 5),
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(position), "{source}");
    }
    for source in [
        "def build(*, a): pass",
        "def build(*): pass",
        "callback = function(*, a) { return a }",
        "callback = (*, a) => a",
    ] {
        assert_eq!(
            Runtime::check(source).unwrap_err().kind(),
            ErrorKind::Syntax
        );
    }
}

#[test]
fn typed_untyped_bodyless_and_missing_parameters_do_not_invent_properties() {
    checked(
        r#"
class Pair(*, a: int, b):
    this.a = a
    this.b = b
class Shell(*, code: int)
pair = Pair(b=2, a=1)
missing = Pair(a=3)
empty = Pair()
shell = Shell(code=7)
assert(pair, {id: "pair", a: 1, b: 2})
assert(missing.a, 3)
assert(missing.b, null)
assert(empty.a, null)
assert(empty.b, null)
assert(shell, {id: "shell"})
assert(shell.code, null)
assert(Pair.length, 3)
assert(Shell.length, 1)
"#,
    );
}

#[test]
fn owned_constructor_values_preserve_composites_nulls_and_legacy_surplus() {
    checked(
        r#"
class Pair(*, a, b, missing):
    this.a = a
    this.b = b
    this.missing = missing
class Legacy(a, a, missing):
    this.a = a
    this.missing = missing
events = {count: 0}
def payload():
    events.count = events.count + 1
    return [[1, 2], [3]]
pair = Pair(b=payload(), a=[[4], [5, 6]])
legacy = Legacy([0], payload())
surplus = Legacy([0], [[7, 8]], null, payload())
assert(pair.a, [[4], [5, 6]])
assert(pair.b, [[1, 2], [3]])
assert(pair.missing, null)
assert(legacy.a, [[1, 2], [3]])
assert(legacy.missing, null)
assert(surplus.a, [[7, 8]])
assert(surplus.missing, null)
assert(events.count, 3)
"#,
    );
}

#[test]
fn assignment_expression_new_nested_property_and_collection_routes_bind_names() {
    checked(
        r#"
class Pair(*, a, b):
    this.a = a
    this.b = b
pair = new Pair(b=2, a=1)
holder = {}
holder.pair = Pair(b=4, a=3)
items = [Pair(b=6, a=5)]
record = {pair: Pair(b=8, a=7)}
assert(pair.a, 1)
assert(pair.b, 2)
assert(holder.pair.a, 3)
assert(holder.pair.b, 4)
assert(items[0].a, 5)
assert(items[0].b, 6)
assert(record.pair.a, 7)
assert(record.pair.b, 8)
assert(Pair(b=10, a=9).a, 9)
assert(Pair.length, 5)
"#,
    );
}

#[test]
fn every_binding_is_validated_before_any_value_even_value_snapshots() {
    let mut runtime = checked(
        r#"
events = {count: 0}
class Pair(*, a, b):
    this.a = a
    this.b = b
"#,
    );
    for (call, message) in [
        (
            "Pair(a=missing, a=(events.count=9))",
            "Duplicate constructor argument 'a' for Pair",
        ),
        (
            "Pair(unknown=(events.count=9), a=missing)",
            "Unknown constructor argument 'unknown' for Pair",
        ),
        (
            "Pair(a=(events.count=9), missing.value)",
            "Constructor Pair requires named arguments",
        ),
        (
            "Pair(1, a=(events.count=9))",
            "Constructor Pair requires named arguments",
        ),
        (
            "Pair(events.count=9)",
            "Constructor Pair requires named arguments",
        ),
        (
            "Pair(a=(events.count=9), b=1, unknown=missing)",
            "Unknown constructor argument 'unknown' for Pair",
        ),
    ] {
        let error = runtime.run(&format!("failed = {call}")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{call}");
        assert_eq!(error.message(), message, "{call}");
        assert_eq!(error.position(), Some(Position::new(1, 1)), "{call}");
        assert_eq!(runtime.run("events.count").unwrap(), Value::Number(0.0));
        assert_eq!(runtime.run("Pair.length").unwrap(), Value::Number(0.0));
        assert!(!runtime.state.has_object(&ObjectId::named("failed")));
        assert!(!runtime.state.has_variable("failed"));
        assert!(!runtime.graph.contains(&NodeKey::variable("a")));
        assert!(!runtime.graph.contains(&NodeKey::variable("unknown")));
    }
}

#[test]
fn values_run_once_in_source_order_in_the_callers_scope() {
    checked(
        r#"
'use imperative'
class Pair(*, a, b):
    this.a = a
    this.b = b
events = {count: 0, order: ""}
def mark(label):
    events.count = events.count + 1
    events.order = events.order + label
    return events.count
pair = Pair(b=mark("B"), a=mark("A"))
assert(pair.a, 2)
assert(pair.b, 1)
assert(events.count, 2)
assert(events.order, "BA")
snapshot = Pair(b=(events.count=3), a=events.count.value)
assert(snapshot.a, 3)
assert(snapshot.b, 3)
def create(a, b):
    result = Pair(b=a + 1, a=b + 1)
    assert(a, 10)
    assert(b, 20)
    return result
local = create(10, 20)
assert(local.a, 21)
assert(local.b, 11)
"#,
    );
}

#[test]
fn snapshot_instances_keep_legacy_dependency_and_reasoning_behavior() {
    let runtime = checked(
        r#"
left = 1
right = 2
class Pair(*, a, b):
    this.a = a
    this.b = b
pair = Pair(b=right, a=left)
collection = [Pair(b=right, a=left)]
why = pair |> why
assert(why[0].from, [])
assert(why[0].rule, "pair = Pair(b=right,a=left)")
left = 3
assert(pair.a, 1)
assert(pair.b, 2)
assert(collection[0].a, 3)
assert(collection[0].b, 2)
"#,
    );
    for label in ["a", "b"] {
        assert!(!runtime.state.has_variable(label));
        assert!(!runtime.graph.contains(&NodeKey::variable(label)));
        assert!(
            runtime
                .graph
                .nodes()
                .all(|node| !node.dependencies.contains(&NodeKey::variable(label)))
        );
    }
}

#[test]
fn class_rules_ignore_labels_but_check_rhs_references_and_cycles() {
    checked(
        r#"
class Pair(*, a, b):
    this.a = a
    this.b = b
class Holder
$Holder.shared = Pair(b=2, a=1)
first = Holder()
second = Holder()
assert(first.shared, second.shared)
assert(first.shared.a, 1)
assert(first.shared.b, 2)
a = {unrelated: true}
{
    $Holder.nested = Pair(a=3, b=4)
}
assert(first.nested.a, 3)
assert(second.nested.b, 4)
$Holder.seed = 5
$Holder.record = {pair: Pair(b=6, a=$Holder.seed)}
assert(first.record.pair.a, 5)
first.seed = 7
assert(first.record.pair.a, 7)
assert(second.record.pair.a, 5)
try:
    $Holder.missing = Pair(a=absent)
catch error:
    assert(error, ReferenceError("absent is not defined"))
try:
    $Holder.seed = Pair(a=$Holder.seed).a
catch error:
    assert(error, TypeError("Circular Dependency"))
assert(first.seed, 7)
"#,
    );
}

#[test]
fn shared_named_construction_uses_caller_parameters_without_repairing_legacy_scope() {
    checked(
        r#"
class Pair(*, a, b):
    this.a = a
    this.b = b
class Legacy(a):
    this.a = a
class Holder
first = Holder()
def install(a, b):
    $Holder.shared = Pair(b=b, a=a)
    return [a, b]
installed = install(3, 4)
second = Holder()
assert(installed, [3, 4])
assert(first.shared.a, 3)
assert(second.shared.b, 4)
assert(first.shared, second.shared)
assert(Pair.length, 1)
def legacyInstall(a):
    $Holder.legacy = Legacy(a)
try:
    legacyInstall(7)
catch error:
    assert(error, ReferenceError("a is not defined"))
assert(Legacy.length, 0)
assert(first.shared.a, 3)
"#,
    );
}

#[test]
fn init_adoption_keeps_class_binding_but_does_not_change_direct_method_calls() {
    checked(
        r#"
'use imperative'
class Pair(*):
    def init(a: int, b):
        this.a = a
        this.b = b
pair = Pair(b=2, a=1)
assert(pair.a, 1)
assert(pair.b, 2)
a = 10
b = 20
pair.init(b=4, a=3)
assert(pair.a, 4)
assert(pair.b, 3)
assert(a, 3)
assert(b, 4)
class Adopted(*, ignored):
    def init(actual):
        this.actual = actual
adopted = Adopted(actual=9)
assert(adopted.actual, 9)
try:
    invalid = Adopted(ignored=1)
catch error:
    assert(error, TypeError("Unknown constructor argument 'ignored' for Adopted"))
"#,
    );
}

#[test]
fn inherited_effective_constructors_carry_their_own_binding_modes() {
    checked(
        r#"
class Parent(*, a, b):
    this.a = a
    this.b = b
class Child: Parent
    pass
class Bodyless(ignored): Parent
    pass
child = Child(b=2, a=1)
bodyless = Bodyless(b=4, a=3)
assert(child.a, 1)
assert(child.b, 2)
assert(bodyless.a, 3)
assert(bodyless.b, 4)
class Legacy(a, b):
    this.a = a
    this.b = b
class MarkedButEmpty(*, ignored): Legacy
    pass
legacy = MarkedButEmpty(6, 5)
assert(legacy.a, 6)
assert(legacy.b, 5)
assert(Parent.length, 0)
assert(Child.length, 1)
"#,
    );
}

#[test]
fn named_and_positional_super_forwarding_preserve_receiver_and_constructor_context() {
    checked(
        r#"
class Base(*, a, b):
    this.a = a
    this.b = b
class Middle(first, second): Base
    super(b=second, a=first)
class Leaf: Middle
    pass
leaf = Leaf(1, 2)
assert(leaf.a, 1)
assert(leaf.b, 2)
class Empty: Base
    def init():
        this.result = super
empty = Empty()
assert(empty.a, null)
assert(empty.b, null)
assert(empty.result, null)
class OldBase(a, b):
    this.a = a
    this.b = b
class NamedChild(*, first, second): OldBase
    super(b=second, a=first)
named = NamedChild(second=4, first=3)
assert(named.a, 4)
assert(named.b, 3)
assert(Base.length, 0)
assert(Middle.length, 0)
assert(Leaf.length, 1)
"#,
    );
}

#[test]
fn inherited_binding_errors_and_method_super_use_the_effective_parent() {
    checked(
        r#"
events = {count: 0}
class Parent(*, a, b):
    this.a = a
    this.b = b
class Child: Parent
    def reset(x, y) => super(b=y, a=x)
    def invalid() => super(a=missing, a=(events.count=9))
    def clear() => super
child = Child(b=2, a=1)
assert(child.reset(3, 4), null)
assert(child.a, 3)
assert(child.b, 4)
try:
    child.invalid()
catch error:
    assert(error, TypeError("Duplicate constructor argument 'a' for Parent"))
try:
    invalid = Child(unknown=missing)
catch error:
    assert(error, TypeError("Unknown constructor argument 'unknown' for Parent"))
assert(events.count, 0)
assert(child.a, 3)
assert(child.clear(), null)
assert(child.a, null)
assert(child.b, null)
assert(Parent.length, 0)
assert(Child.length, 1)
"#,
    );
}

#[test]
fn grouping_and_existing_assignment_lowering_keep_the_documented_ast_interpretation() {
    let lowered = parse_expression("One(amount += 2)").unwrap();
    assert_eq!(lowered.to_string(), "One(amount=amount+2)");
    checked(
        r#"
class One(*, amount):
    this.amount = amount
class Legacy(amount):
    this.amount = amount
amount = 10
grouped = One((amount=3))
assert(grouped.amount, 3)
assert(amount, 10)
compound = One(amount += 2)
assert(compound.amount, 12)
assert(amount, 10)
legacy = Legacy(amount += 2)
assert(legacy.amount, 12)
assert(amount, 12)
"#,
    );
}

#[test]
fn ordinary_functions_methods_builtins_and_duplicate_legacy_formals_remain_positional() {
    checked(
        r#"
'use imperative'
def combine(a, b) => a * 10 + b
a = 0
b = 0
result = combine(b=2, a=1)
assert(result, 21)
assert(a, 1)
assert(b, 2)
class Legacy(a, b):
    this.a = a
    this.b = b
    def combine(a, b) => a * 10 + b
legacy = Legacy(b=4, a=3)
assert(legacy.a, 4)
assert(legacy.b, 3)
assert(legacy.combine(b=6, a=5), 65)
assert(a, 5)
assert(b, 6)
assert(Math.max(b=8, a=7), 8)
assert(a, 7)
assert(b, 8)
class Repeated(a, a):
    this.a = a
repeated = Repeated(1, 2)
assert(repeated.a, 2)
class RepeatedInit:
    def init(a, a):
        this.a = a
repeatedInit = RepeatedInit(3, 4)
assert(repeatedInit.a, 4)
"#,
    );
}

#[test]
fn opting_in_does_not_add_callable_class_aliases() {
    let mut runtime = checked(
        r#"
class Pair(*, a):
    this.a = a
alias = $Pair
holder = {factory: $Pair}
"#,
    );
    let Some(Value::Object(holder)) = runtime.state.variable("holder") else {
        panic!("holder must be an object");
    };
    let holder = holder.clone();
    for (source, message) in [
        ("alias(a=1)", "alias is not a function".to_string()),
        (
            "holder.factory(a=1)",
            format!("{holder}.factory is not a function"),
        ),
        (
            "[$Pair][0](a=1)",
            "[$Pair][0] is not a function".to_string(),
        ),
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{source}");
        assert_eq!(error.message(), message, "{source}");
    }
    assert_eq!(runtime.run("Pair.length").unwrap(), Value::Number(0.0));
}

#[test]
fn class_and_function_name_collisions_keep_the_existing_distinct_call_routes() {
    checked(
        r#"
class Factory(*, a):
    this.a = a
def Factory(a, b) => a * 10 + b
a = 0
b = 0
class Holder
$Holder.instance = Factory(a=3)
$Holder.result = {number: Factory(b=2, a=1)}
holder = Holder()
assert(holder.instance.a, 3)
assert(holder.result.number, 21)
assert(Factory(b=4, a=3), 43)
def call(Factory):
    return Factory(b=6, a=5)
assert(call((a, b) => a * 10 + b), 65)
"#,
    );
}

#[test]
fn failed_constructors_restore_instances_properties_dependencies_and_binding_metadata() {
    let mut runtime = checked(
        r#"
saved = {amount: 5}
derived = saved.amount * 2
class Pair(*, a):
    saved.amount = 9
    this.a = a
    throw "ABORT"
"#,
    );
    let before: Vec<_> = runtime
        .graph
        .nodes()
        .map(|node| (node.key.clone(), node.dependencies.clone()))
        .collect();
    let error = runtime.run("failed = Pair(a=3)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Thrown);
    assert_eq!(error.message(), "ABORT");
    let after: Vec<_> = runtime
        .graph
        .nodes()
        .map(|node| (node.key.clone(), node.dependencies.clone()))
        .collect();
    assert_eq!(before, after);
    assert_eq!(runtime.run("saved.amount").unwrap(), Value::Number(5.0));
    assert_eq!(runtime.run("derived").unwrap(), Value::Number(10.0));
    assert!(runtime.state.class("Pair").unwrap().instances.is_empty());
    assert!(!runtime.state.has_object(&ObjectId::named("failed")));
    assert!(!runtime.state.has_variable("failed"));

    runtime
        .run(
            r#"
try:
    class Pair(a):
        this.a = a
    failed = Pair(7)
    throw "UNDO"
catch error:
    assert(error, "UNDO")
"#,
        )
        .unwrap();
    assert!(runtime.take_assertions().is_empty());
    assert_eq!(
        runtime.state.class("Pair").unwrap().constructor_binding,
        ConstructorBinding::Named
    );
    assert!(runtime.state.class("Pair").unwrap().instances.is_empty());
    runtime.run("saved.amount = 6").unwrap();
    assert_eq!(runtime.run("derived").unwrap(), Value::Number(12.0));
}

#[test]
fn caught_parent_and_rhs_failures_leave_no_partial_objects_or_rules() {
    checked(
        r#"
events = {count: 0}
doubled = events.count * 2
class Parent(*, a):
    this.a = a
    events.count = 9
    throw "PARENT"
class Child(a): Parent
    super(a=a)
try:
    failed = Child(3)
catch error:
    assert(error, "PARENT")
assert(events.count, 0)
assert(doubled, 0)
assert(Parent.length, 0)
assert(Child.length, 0)
class Good(*, a, b):
    this.a = a
try:
    failed = Good(a=(events.count=4), b=missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(events.count, 0)
assert(doubled, 0)
assert(Good.length, 0)
events.count = 5
assert(doubled, 10)
"#,
    );
}

#[test]
fn redeclaration_resets_mode_without_rewriting_instances_or_class_rules() {
    checked(
        r#"
class Pair(*, a):
    this.a = a
first = Pair(a=1)
$Pair.ready = true
class Pair(a):
    this.a = a
second = Pair(2)
assert(first.a, 1)
assert(second.a, 2)
assert(first.ready, true)
assert(second.ready, true)
class Pair(*):
    def init(a):
        this.a = a
third = Pair(a=3)
assert(third.a, 3)
assert(third.ready, true)
assert(Pair.length, 3)
"#,
    );
}

#[test]
fn named_calls_pin_the_validated_constructor_but_legacy_calls_still_select_after_rhs() {
    checked(
        r#"
class Named(*, a):
    this.original = a
def replaceNamed():
    class Named(a):
        this.replacement = a
    return 7
first = Named(a=replaceNamed())
second = Named(8)
assert(first.original, 7)
assert(first.replacement, null)
assert(second.replacement, 8)
class Legacy(a):
    this.original = a
def replaceLegacy():
    class Legacy(a):
        this.replacement = a
    return 9
legacy = Legacy(replaceLegacy())
assert(legacy.original, null)
assert(legacy.replacement, 9)
"#,
    );
}

#[test]
fn existing_header_and_expression_resource_limits_remain_in_force() {
    let names: Vec<_> = (0..256).map(|index| format!("p{index}")).collect();
    checked(&format!(
        "class Wide(*, {}):\n    this.last = p255\nwide = Wide(p255=7)\nassert(wide.last, 7)",
        names.join(", ")
    ));
    let source = format!(
        "class Pair(*, a):\n    this.a = a\npair = Pair(a={}1{})",
        "(".repeat(80),
        ")".repeat(80)
    );
    let error = Runtime::check(&source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(error.message(), "Expressions are nested too deeply");
}

#[test]
fn actual_cli_serializes_named_instances_and_has_no_label_graph_nodes() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/named-constructors.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let value: serde_json::Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(value, serde_json::json!({"id": "pair", "a": 1.0, "b": 2.0}));
    assert!(stdout.contains("pair.a [property]"), "{stdout}");
    assert!(
        !stdout.lines().any(|line| line.starts_with("  a [")),
        "{stdout}"
    );
    assert!(
        !stdout.lines().any(|line| line.starts_with("  b [")),
        "{stdout}"
    );
}

#[test]
fn actual_cli_reports_invalid_bindings_and_repl_handles_the_new_class_header() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/named-constructors-invalid.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg(fixture)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        "TypeError: Constructor Pair requires named arguments (line 2, column 1)"
    );

    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(
        b"class Pair(*):\n    def init(a, b):\n        this.a = a\n        this.b = b\n\npair = Pair(b=2, a=1)\nassert(pair.a, 1)\nassert(pair.b, 2)\n.json on\npair.a + pair.b\n.exit\n",
    ).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.lines().any(|line| line == "3.0"), "{stdout}");
}
