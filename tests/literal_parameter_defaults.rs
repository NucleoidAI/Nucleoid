//! Bounded scalar defaults, with complete accepted controls and no dependency
//! on Git or session artifacts in the ordinary feature checks.

mod common;
#[path = "support/literal_default_cases.rs"]
mod default_cases;

use nucleoid::expression::Expression;
use nucleoid::lang::ast::generator::{generate, generate_all};
use nucleoid::lang::ast::{
    Expr, FunctionBody, ParameterDefault, Stmt, parse, parse_expression, parse_program,
};
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn checked(source: &str) -> Runtime {
    let expected = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let mut runtime = Runtime::new();
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}\n{error}"));
    assert_eq!(runtime.assertions_run(), expected, "{source}");
    let failures = runtime.take_assertions();
    assert!(failures.is_empty(), "{source}\n{failures:?}");
    runtime
}

#[test]
fn nine_complete_controls_and_exact_candidate_lowerings_preserve_61_assertions_and_one_return() {
    let documents = default_cases::documents();
    let mut totals = [0, 0, 0];
    for sealed in default_cases::SEALED {
        default_cases::assert_exact_lowering(sealed);
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == sealed.document)
            .unwrap()
            .1;
        let matches = cases
            .iter()
            .filter(|case| case.title == sealed.title)
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1);
        let documented = if matches!(sealed.document, "spec" | "reference") {
            format!("# {}\n\n{}", sealed.title, sealed.candidate)
        } else {
            sealed.candidate.to_string()
        };
        assert_eq!(matches[0].source.trim(), documented, "{}", sealed.title);
        assert_eq!(matches[0].expected.as_deref(), sealed.expected);
        let old = parse(sealed.accepted).unwrap();
        let new = parse(sealed.candidate).unwrap();
        assert_ne!(old, new, "signatures/calls intentionally change");
        let assertions = common::assertions::protected_assertions(&old);
        assert_eq!(assertions.len(), sealed.assertions);
        assert_eq!(common::assertions::protected_assertions(&new), assertions);
        let mut runtimes = [Runtime::new(), Runtime::new()];
        for (runtime, source) in runtimes.iter_mut().zip([sealed.accepted, sealed.candidate]) {
            let case = common::Case {
                title: sealed.title.to_string(),
                source: source.to_string(),
                expected: sealed.expected.map(str::to_string),
            };
            common::check_with_count(runtime, &case, sealed.assertions)
                .unwrap_or_else(|reason| panic!("{}: {reason}", sealed.title));
            assert_eq!(runtime.assertions_run(), sealed.assertions);
        }
        default_cases::assert_observations(&runtimes[1], &runtimes[0], sealed);
        totals[0] += 1;
        totals[1] += sealed.assertions;
        totals[2] += usize::from(sealed.expected.is_some());
    }
    assert_eq!(totals, [9, 61, 1]);
}

#[test]
fn every_parenthesized_function_body_and_multiline_form_retains_typed_default_data() {
    for source in [
        "def sample(value: Marker = 1) => value",
        "def sample(value: Marker = 1) =>\n    value",
        "def sample(value: Marker = 1): return value",
        "def sample(value: Marker = 1):\n    return value",
        "callback = function(value: Marker = 1) { return value }",
        "callback = (value: Marker = 1) => value",
        "callback = (value: Marker = 1) => { return value }",
    ] {
        let statements = parse(source).unwrap();
        let function = match &statements[0] {
            Stmt::Function(function)
            | Stmt::Assign {
                value: Expr::Function(function),
                ..
            } => function,
            other => panic!("{other:?}"),
        };
        assert_eq!(function.parameters.len(), 1);
        let parameter = &function.parameters[0];
        assert_eq!(parameter.name, "value");
        assert_eq!(parameter.type_name.as_deref(), Some("Marker"));
        assert_eq!(parameter.default, Some(ParameterDefault::Number(1.0)));
        let call = if function.name.is_some() {
            "sample"
        } else {
            "callback"
        };
        checked(&format!("{source}\nassert({call}(), 1)"));
    }
    checked(
        r#"
def mixed(
    first = 1,
    second: UnresolvedAnnotation = "x",
    third = false,
):
    return [first, second, third]
assert(mixed(), [1, "x", false])
"#,
    );
}

#[test]
fn scalar_kinds_signs_scientific_notation_and_exact_signed_zero_are_preserved() {
    for (literal, expected) in [
        ("0", 0.0_f64),
        ("+0", 0.0),
        ("-0", -0.0),
        ("-0.0", -0.0),
        ("+1.25", 1.25),
        ("-.5", -0.5),
        ("+2e-3", 0.002),
        ("-1e3", -1000.0),
        ("1e308", 1e308),
        ("-1e-999", -0.0),
    ] {
        let source = format!("def sample(value = {literal}) => value\nsample()");
        let mut runtime = Runtime::new();
        let Value::Number(actual) = runtime.run(&source).unwrap() else {
            panic!("{literal}")
        };
        assert_eq!(actual.to_bits(), expected.to_bits(), "{literal}");
        let Stmt::Function(function) = &parse(&source).unwrap()[0] else {
            panic!()
        };
        let parameter = &function.parameters[0];
        let canonical = format!("def sample({parameter}) => value");
        let Stmt::Function(reparsed) = &parse(&canonical).unwrap()[0] else {
            panic!()
        };
        assert_eq!(function.parameters, reparsed.parameters);
    }
    checked(
        r#"
def scalars(a = "text", b = '', c = true, d = false, e = null):
    return [a, b, c, d, e]
result = scalars().value
assert(result, ["text", "", true, false, null])
"#,
    );
    let mut runtime = Runtime::new();
    let result = runtime
        .run("def signed(value = -0) => [value@0, value = 0, value@previous]\nsigned().value")
        .unwrap();
    let Value::List(values) = result else {
        panic!()
    };
    assert_eq!(values.len(), 3);
    for (value, negative) in values.iter().zip([true, false, true]) {
        let Value::Number(number) = value else {
            panic!("{value:?}")
        };
        assert_eq!(*number, 0.0);
        assert_eq!(number.is_sign_negative(), negative);
    }
}

#[test]
fn defaults_reject_unsupported_missing_compound_and_nonfinite_values_explicitly() {
    for (literal, message) in [
        ("[]", "Expected a scalar literal parameter default"),
        ("{}", "Expected a scalar literal parameter default"),
        ("/x/", "Expected a scalar literal parameter default"),
        ("`plain`", "Expected a scalar literal parameter default"),
        (
            "`value:${1}`",
            "Expected a scalar literal parameter default",
        ),
        ("other", "Expected a scalar literal parameter default"),
        ("other()", "Expected a scalar literal parameter default"),
        ("Infinity", "Expected a scalar literal parameter default"),
        ("NaN", "Expected a scalar literal parameter default"),
        ("(1)", "Expected a scalar literal parameter default"),
        ("--1", "Expected a scalar literal parameter default"),
        ("++1", "Expected a scalar literal parameter default"),
        ("-+1", "Expected a scalar literal parameter default"),
        ("+false", "Expected a scalar literal parameter default"),
        ("-null", "Expected a scalar literal parameter default"),
        ("", "Expected a scalar literal parameter default"),
        ("1e999", "Parameter default must be a finite number"),
        ("-1e999", "Parameter default must be a finite number"),
        ("+1e999", "Parameter default must be a finite number"),
        ("1 + 2", "Expected ',' or ')' after parameter default"),
        (
            "true && false",
            "Expected ',' or ')' after parameter default",
        ),
        ("null.member", "Expected ',' or ')' after parameter default"),
        ("1()", "Expected ',' or ')' after parameter default"),
        ("1 = 2", "Expected ',' or ')' after parameter default"),
    ] {
        for source in [
            format!("def sample(value = {literal}) => value"),
            format!("callback = (value = {literal}) => value"),
            format!("callback = function(value = {literal}) {{ return value }}"),
            format!("class Sample(value = {literal}):\n    this.value = value"),
            format!("class Sample:\n    def method(value = {literal}) => value"),
        ] {
            let error = Runtime::check(&source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            assert_eq!(error.message(), message, "{source}");
        }
    }
    let error = Runtime::check("def sample(value = []) => value").unwrap_err();
    assert_eq!(error.position(), Some(Position::new(1, 20)));
    let mut runtime = Runtime::new();
    runtime.run("saved = 1").unwrap();
    assert!(
        runtime
            .run("saved = 2\ndef f(value = unknown()) => value")
            .is_err()
    );
    assert_eq!(runtime.run("saved").unwrap(), Value::Number(1.0));
}

#[test]
fn parentheses_do_not_reinterpret_grouped_assignments_or_legacy_bare_signatures() {
    let grouped = parse_expression("(value = 4)").unwrap();
    assert!(matches!(grouped, Expr::Assign { .. }));
    assert_eq!(grouped, parse_expression("((value = 4))").unwrap());
    let arrow = parse_expression("(value = 4) => value").unwrap();
    assert!(matches!(arrow, Expr::Function(_)));
    for signature in [
        "def f value => value",
        "def f value: int => value",
        "def f left, right => left",
        "def f left right => left",
        "def f left: Type, right => left",
        "def f value:\n    return value",
        "def f value: Type:\n    return value",
        "def f left, right:\n    return left",
        "def f left right:\n    return left",
        "def f left: Type, right:\n    return left",
    ] {
        let statements = parse(signature).unwrap();
        let Stmt::Function(function) = &statements[0] else {
            panic!()
        };
        assert!(
            function
                .parameters
                .iter()
                .all(|parameter| parameter.default.is_none())
        );
        checked(&format!(
            "{signature}\nassert(f(3), 3)\nassert(f(), null)\nassert(f(5, 6, 7), 5)"
        ));
    }
    for source in [
        "def f value = 1 => value",
        "def f value: Type = 1 => value",
        "def f left, right = 1 => left",
        "def f left right = 1 => left",
        "def f value = 1:\n    return value",
        "def f value: Type = 1:\n    return value",
        "def f left, right = 1:\n    return left",
        "def f left right = 1:\n    return left",
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert!(
            error
                .message()
                .starts_with("Expected '(' but found identifier"),
            "{source}: {error}"
        );
    }
}

#[test]
fn defaults_can_precede_required_slots_and_annotations_do_not_enforce_types_or_arity() {
    checked(
        r#"
def slots(first: Unresolved = 1, required, last = 3) => [first, required, last]
empty = slots().value
one = slots(0).value
two = slots(false, "").value
extra = slots("text", 2, true, 99).value
assert(empty, [1, null, 3])
assert(one, [0, null, 3])
assert(two, [false, "", 3])
assert(extra, ["text", 2, true])
def repeated(value = 1, value = 2) => [value@0, value@1, value]
assert(repeated(), [1, 2, 2])
"#,
    );
}

#[test]
fn explicitly_supplied_undefined_and_other_values_do_not_use_defaults() {
    checked(
        r#"
def pick(value = 12) => value
assert(pick(), 12)
assert(pick(0), 0)
assert(pick(false), false)
assert(pick(""), "")
assert(pick(null), null)
assert(pick([]), [])
assert(pick([1, 2]), [1, 2])
"#,
    );
    let mut runtime = Runtime::new();
    runtime
        .run("class Box\nbox = Box()\ndef observe(value = 12) => typeof value")
        .unwrap();
    assert_eq!(
        runtime.run("observe(box.missing)").unwrap(),
        Value::String("undefined".to_string())
    );
    assert_eq!(
        runtime.run("observe(null)").unwrap(),
        Value::String("null".to_string())
    );
    assert_eq!(
        runtime.run("observe()").unwrap(),
        Value::String("number".to_string())
    );
}

#[test]
fn ordinary_alias_callback_property_list_and_returned_function_routes_share_binding() {
    checked(
        r#"
base = 2
def operation(value = 3, offset = 4) => value + offset + base
alias = operation
frozen = operation.value
holder = {call: operation}
callbacks = [operation]
def apply(callback) => callback()
def factory(ignored = 100) => (value = 5) => value + base
returned = factory()
anonymous = function(value = 6) { return value + base }
blocked = (value = 7) => { return value + base }
assert(operation(), 9)
assert(alias(), 9)
assert(frozen(), 9)
assert(holder.call(), 9)
assert(callbacks[0](), 9)
assert(apply(operation), 9)
assert(returned(), 7)
assert(anonymous(), 8)
assert(blocked(), 9)
assert([1, 2].map((value, offset = 2) => value + offset), [3, 4])
assert([1, 2].reduce((sum, value, offset = 3) => sum + value + offset, 0), 9)
class Meter(seed = 1):
    this.seed = seed
    def read(offset = 2) => self.seed + offset
first = Meter()
second = Meter(5)
$Meter.total = operation($Meter.seed)
assert(first.read(), 3)
assert(second.read(), 7)
assert(first.total, 7)
assert(second.total, 11)
"#,
    );
}

#[test]
fn function_argument_assignments_remain_positional_expressions_not_labels() {
    checked(
        r#"
'use imperative'
a = b = 0
def combine(a = 1, b = 2) => a * 10 + b
result = combine(b=4, a=3)
assert(result, 43)
assert(a, 3)
assert(b, 4)
single = combine(b=8)
assert(single, 82)
assert(a, 3)
assert(b, 8)
class Meter(seed = 1):
    this.seed = seed
    def combine(a = 1, b = 2) => a * 10 + b
meter = Meter()
assert(meter.combine(b=6, a=5), 65)
assert(a, 5)
assert(b, 6)
assert(Math.max(b=8, a=7), 8)
assert(a, 7)
assert(b, 8)
"#,
    );
}

#[test]
fn positional_named_mixed_and_inherited_constructor_and_super_routes_use_selected_defaults() {
    checked(
        r#"
class Pair(first = 1, second: Marker = 2):
    this.first = first
    this.second = second
positional = Pair()
named = Pair(second=8)
mixed = Pair(0, second=0)
supplied = Pair(first=null, second=false)
assert(positional.first, 1)
assert(positional.second, 2)
assert(named.first, 1)
assert(named.second, 8)
assert(mixed.first, 0)
assert(mixed.second, 0)
assert(supplied.first, null)
assert(supplied.second, false)
class Parent:
    def init(seed = 3):
        this.seed = seed
class Inherited: Parent
    pass
class Skipped(seed = 99): Parent
    pass
class Bare: Parent
    super
class Empty: Parent
    super()
class Named: Parent
    super(seed=0)
class Child(seed = 4): Parent
    super()
    this.childSeed = seed
inherited = Inherited()
skipped = Skipped()
bare = Bare()
empty = Empty()
namedSuper = Named()
child = Child()
assert(inherited.seed, 3)
assert(skipped.seed, 3)
assert(bare.seed, 3)
assert(empty.seed, 3)
assert(namedSuper.seed, 0)
assert(child.seed, 3)
assert(child.childSeed, 4)
class Shell(code = "x")
shell = Shell()
assert(shell.code, null)
assert(shell, {id: "shell"})
"#,
    );
}

#[test]
fn automatic_no_formal_constructors_and_strict_label_validation_are_not_relaxed() {
    let mut runtime = checked(
        r#"
class Automatic
class Pair(first = 1, second = 2):
    this.first = first
def fail() => missing
"#,
    );
    for (source, message) in [
        (
            "Automatic(first=fail())",
            "Unknown constructor argument 'first' for Automatic",
        ),
        (
            "Pair(unknown=fail())",
            "Unknown constructor argument 'unknown' for Pair",
        ),
        (
            "Pair(first=1, first=fail())",
            "Duplicate constructor argument 'first' for Pair",
        ),
        (
            "Pair(1, first=fail())",
            "Duplicate constructor argument 'first' for Pair",
        ),
        (
            "Pair(first=1, fail())",
            "Positional constructor argument after named arguments for Pair",
        ),
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{source}");
        assert_eq!(error.message(), message, "{source}");
    }
    assert_eq!(runtime.run("Pair.length").unwrap(), Value::Number(0.0));
}

#[test]
fn new_nested_property_collection_anonymous_and_shared_constructor_routes_fill_absent_keys() {
    checked(
        r#"
class Pair(first = 1, second = 2):
    this.first = first
    this.second = second
pair = new Pair()
holder = {}
holder.pair = Pair(second=4)
items = [Pair(second=5)]
record = {pair: Pair(second=6)}
assert(pair.first, 1)
assert(pair.second, 2)
assert(holder.pair.first, 1)
assert(holder.pair.second, 4)
assert(items[0].first, 1)
assert(items[0].second, 5)
assert(record.pair.first, 1)
assert(record.pair.second, 6)
assert(Pair(second=7).first, 1)
class Holder
owner = Holder()
$Holder.pair = Pair(second=8)
assert(owner.pair.first, 1)
assert(owner.pair.second, 8)
assert(Pair.length, 6)
"#,
    );
}

#[test]
fn arguments_run_once_in_source_order_and_surplus_values_still_run() {
    let mut runtime = checked(
        r#"
events = {trace: []}
def mark(value):
    events.trace.push(value)
    return value
def pair(first = 10, second = 20) => [first, second]
class Pair(first = 10, second = 20):
    this.first = first
    this.second = second
"#,
    );
    // Direct invocation avoids changing the accepted snapshot/freeze behavior
    // of effectful list-valued assignments; each actual call evaluates once.
    assert_eq!(
        runtime.run("pair(mark(1), mark(2), mark(3))").unwrap(),
        Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
    );
    let case = common::Case {
        title: "named constructor argument source order".to_string(),
        source: r#"
named = Pair(second=mark(4), first=mark(5))
missing = Pair(second=mark(6))
assert(named.first, 5)
assert(named.second, 4)
assert(missing.first, 10)
assert(missing.second, 6)
assert(events.trace, [1, 2, 3, 4, 5, 6])
"#
        .to_string(),
        expected: None,
    };
    common::check_with_count(&mut runtime, &case, 5).unwrap();
    assert_eq!(runtime.assertions_run(), 5);
}

#[test]
fn constructor_default_selection_keeps_named_pinning_and_positional_delay() {
    for (call, expected) in [("Pair(value=replace())", 10), ("Pair(replace())", 20)] {
        checked(&format!(
            r#"
class Pair(value, unit = 10):
    this.amount = value
    this.unit = unit
def replace():
    class Pair(value, unit = 20):
        this.amount = value
        this.unit = unit
    return 1
first = {call}
second = Pair(2)
assert(first.amount, 1)
assert(first.unit, {expected})
assert(second.unit, 20)
"#
        ));
    }
    checked(
        r#"
def choose(value, unit = 10) => value + unit
def replace():
    def choose(value, unit = 20) => value + unit
    return 1
first = choose(replace()).value
second = choose(1).value
assert(first, 11)
assert(second, 21)
"#,
    );
}

#[test]
fn super_keeps_parent_resolution_timing_and_same_receiver_with_defaulted_holes() {
    for (call, expected) in [("super(reparent())", 20), ("super(value=reparent())", 10)] {
        for (body, invocation) in [
            (call.to_string(), "child = Child()"),
            (
                format!("def reset() => {call}"),
                "child = Child()\nchild.reset()",
            ),
        ] {
            checked(&format!(
                r#"
class Original(value, unit = 10):
    this.amount = value
    this.unit = unit
class Replacement(value, unit = 20):
    this.amount = value
    this.unit = unit
def reparent():
    class Child: Replacement
        pass
    return 1
class Child: Original
    {body}
{invocation}
assert(child.amount, 1)
assert(child.unit, {expected})
assert(Child.length, 1)
assert(Original.length, 0)
assert(Replacement.length, 0)
"#
            ));
        }
    }
}

#[test]
fn method_receiver_is_captured_before_argument_side_rebinding() {
    checked(
        r#"
class Meter(seed = 3):
    this.seed = seed
    def read(ignored, offset = 2) => this.seed + offset
first = Meter()
second = Meter(10)
def swap():
    first = second
    return 0
result = first.read(swap()).value
assert(result, 5)
assert(first, second)
"#,
    );
}

#[test]
fn effective_defaults_start_at_version_zero_in_fresh_function_constructor_and_method_frames() {
    checked(
        r#"
value = 100
def history(value = 4) => [value@0, value = value + 1, value@0, value@1]
one = history()
two = history()
supplied = history(9)
assert(one, [4, 5, 4, 5])
assert(two, [4, 5, 4, 5])
assert(supplied, [9, 10, 9, 10])
assert(value, 100)
class Sample(value = 4):
    this.initial = value@0
    value += 1
    this.previous = value@previous
    this.latest = value@last
    def read(value = 7) => [value@0, value = value + 1, value@previous, value@last]
first = Sample()
second = Sample(value=9)
assert(first.initial, 4)
assert(first.previous, 4)
assert(first.latest, 5)
assert(second.initial, 9)
assert(second.previous, 9)
assert(second.latest, 10)
assert(first.read(), [7, 8, 7, 8])
assert(first.read(), [7, 8, 7, 8])
assert(second.read(10), [10, 11, 10, 11])
"#,
    );
}

#[test]
fn caught_failures_do_not_make_defaulted_locals_transactional_or_capturing() {
    checked(
        r#"
state = 1
value = 100
def attempt(value = 4):
    try:
        value = 9
        state = 2
        throw "LOCAL"
    catch error:
        state = state
    return [value, value@0, value@last, state]
result = attempt().value
assert(result, [9, 4, 9, 1])
assert(state, 1)
assert(value, 100)
class Sample(value = 4):
    this.before = value
    try:
        value = 9
        this.before = 99
        throw "LOCAL"
    catch error:
        this.after = value
    this.initial = value@first
sample = Sample()
assert(sample.before, 4)
assert(sample.after, 9)
assert(sample.initial, 4)
def factory(value = 10) => (offset = 2) => value + offset
callback = factory()
assert(callback(), 102)
"#,
    );
}

#[test]
fn live_frozen_and_historical_function_values_retain_their_own_default_metadata() {
    let mut runtime = checked(
        r#"
base = 2
def operation(value = 3) => value + base
live = operation
frozen = operation.value
stored = operation.value
stored = (value = 7) => value * 2
old = stored@first
observed = live()
assert(observed, 5)
assert(frozen(), 5)
assert(old(), 5)
assert(stored(), 14)
def operation(value = 5) => value * 10 + base
assert(observed, 52)
assert(frozen(), 5)
assert(old(), 5)
base = 4
assert(observed, 54)
assert(frozen(), 7)
assert(old(), 7)
"#,
    );
    let before = runtime
        .graph
        .nodes()
        .map(|node| (node.key.clone(), node.dependencies.clone()))
        .collect::<Vec<_>>();
    let before_function = runtime.state.function("operation").unwrap().clone();
    let error = runtime
        .run("def operation(value = 9) => missing + value")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(
        runtime.state.function("operation").unwrap(),
        &before_function
    );
    let after = runtime
        .graph
        .nodes()
        .map(|node| (node.key.clone(), node.dependencies.clone()))
        .collect::<Vec<_>>();
    assert_eq!(before, after);
    assert_eq!(runtime.run("observed").unwrap(), Value::Number(54.0));
    assert_eq!(runtime.run("old()").unwrap(), Value::Number(7.0));
}

#[test]
fn class_redefinition_and_failed_construction_restore_defaults_instances_state_and_graph() {
    let mut runtime = checked(
        r#"
saved = {amount: 5}
derived = saved.amount * 2
class Pair(value = 3):
    this.amount = value
first = Pair()
$Pair.ready = true
class Pair(value = 7):
    this.amount = value
second = Pair()
assert(first.amount, 3)
assert(second.amount, 7)
assert(first.ready, true)
assert(second.ready, true)
try:
    class Pair(value = 99):
        this.amount = value
    temporary = Pair()
    throw "RESTORE"
catch error:
    assert(error, "RESTORE")
third = Pair()
assert(third.amount, 7)
assert(Pair.length, 3)
class Failing(value = 11):
    saved.amount = value
    this.amount = value
    throw "ABORT"
"#,
    );
    assert_eq!(
        runtime.state.class("Pair").unwrap().parameters[0].default,
        Some(ParameterDefault::Number(7.0))
    );
    let before = runtime
        .graph
        .nodes()
        .map(|node| (node.key.clone(), node.dependencies.clone()))
        .collect::<Vec<_>>();
    let error = runtime.run("failed = Failing()").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Thrown, "{error}");
    assert_eq!(error.message(), "ABORT");
    let after = runtime
        .graph
        .nodes()
        .map(|node| (node.key.clone(), node.dependencies.clone()))
        .collect::<Vec<_>>();
    assert_eq!(before, after);
    assert!(!runtime.state.has_object(&ObjectId::named("failed")));
    assert!(runtime.state.class("Failing").unwrap().instances.is_empty());
    assert_eq!(
        runtime.state.class("Failing").unwrap().parameters[0].default,
        Some(ParameterDefault::Number(11.0))
    );
    assert_eq!(runtime.run("saved.amount").unwrap(), Value::Number(5.0));
    assert_eq!(runtime.run("derived").unwrap(), Value::Number(10.0));
}

#[test]
fn class_method_default_redefinition_and_rollback_preserve_existing_instances() {
    let runtime = checked(
        r#"
class Meter(seed = 1):
    this.amount = seed
    def read(offset = 2) => this.amount + offset
meter = Meter()
assert(meter.read(), 3)
class Meter(seed = 5):
    this.amount = seed
    def read(offset = 4) => this.amount + offset
assert(meter.amount, 1)
assert(meter.read(), 5)
try:
    class Meter(seed = 99):
        this.amount = seed
        def read(offset = 99) => this.amount + offset
    assert(meter.read(), 100)
    throw "METHOD_DEFAULT"
catch error:
    assert(error, "METHOD_DEFAULT")
assert(meter.read(), 5)
assert(meter.amount, 1)
assert(Meter.length, 1)
"#,
    );
    let class = runtime.state.class("Meter").unwrap();
    assert_eq!(
        class.parameters[0].default,
        Some(ParameterDefault::Number(5.0))
    );
    assert_eq!(
        class.methods["read"].parameters[0].default,
        Some(ParameterDefault::Number(4.0))
    );
}

#[test]
fn canonical_generation_keeps_default_signatures_classes_escapes_and_legacy_labels() {
    for source in [
        r#"(value = -0, suffix: Text = "a\"b\\c\n\t\r\0λ") => value"#,
        r#"function(value = false) { return value }"#,
    ] {
        let expression = parse_expression(source).unwrap();
        let canonical = expression.to_string();
        assert_eq!(
            parse_expression(&canonical).unwrap(),
            expression,
            "{canonical}"
        );
        assert!(Expression::new(&expression).roots().is_empty());
        assert!(Expression::new(&expression).class_reference().is_none());
    }
    for source in [
        "def sample(value: Type = -0):\n    return value",
        "class Sample(value = 3):\n    this.value = value\n    def read(offset = 2):\n        return this.value + offset",
        "class Sample:\n    def init(value = 3):\n        this.value = value",
        "class Sample(value = 3): Parent\n    super(value)\n    def read(offset = 2) => this.value + offset",
        "class Sample(value = 3)",
    ] {
        let statements = parse(source).unwrap();
        let canonical = generate(&statements[0]);
        assert!(canonical.contains("=3") || canonical.contains("=-0"));
        assert_eq!(parse(&canonical).unwrap(), statements, "{canonical}");
    }
    assert_eq!(
        generate_all(&parse("def legacy(value) => value\nclass Legacy(value)").unwrap()),
        "def legacy;class Legacy"
    );
    let defaulted = parse_program("def f(value = 1) => value\nnext = 2").unwrap();
    assert_eq!(
        defaulted.positions,
        [Position::new(1, 1), Position::new(2, 1)]
    );
    let Stmt::Function(function) = &defaulted.statements[0] else {
        panic!()
    };
    assert!(matches!(&function.body, FunctionBody::Block(body) if body.len() == 1));
}

#[test]
fn defaults_do_not_change_json_function_snapshots_or_version_namespace_restrictions() {
    let mut runtime = checked(
        r#"
class Sample(value = 3):
    this.amount = value
sample = Sample()
snapshot = sample.amount.value
sample.amount = 4
assert(snapshot, 3)
assert(sample.amount@first, 3)
assert(sample.amount@last, 4)
callable = (value = 7) => value
"#,
    );
    let object = runtime.state.variable("sample").unwrap();
    assert_eq!(
        runtime.serialize_json(object).unwrap(),
        r#"{"amount":4.0,"id":"sample"}"#
    );
    let function = runtime.state.variable("callable").unwrap();
    assert_eq!(runtime.serialize_json(function).unwrap(), r#""[function]""#);
    for source in ["Sample@first", "Math@first"] {
        assert_eq!(
            runtime.run(source).unwrap_err().message(),
            "State versions require a variable or object property"
        );
    }
    assert_eq!(
        Runtime::check("sample.value@-1").unwrap_err().message(),
        "Expected a non-negative integer, 'first', 'last', or 'previous' after '@'"
    );
}
