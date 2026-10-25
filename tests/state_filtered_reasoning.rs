//! Additive state-filter stages and unchanged accepted-syntax replay.
//! This suite checks behavior, never token acceptance or proposal outcomes.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use nucleoid::graph::{NodeKey, ShapeKey};
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime};
use serde::Deserialize;

fn run(source: &str) -> Runtime {
    let mut runtime = Runtime::new();
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    common::check_with_count(
        &mut runtime,
        &common::Case {
            title: "state-filter regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|error| panic!("{source}\n{error}"));
    assert_eq!(runtime.assertions_run(), count);
    runtime
}

#[test]
fn ordinary_names_bindings_calls_methods_and_properties_keep_their_meaning() {
    let tokens = tokenize("stated derived").unwrap();
    assert_eq!(tokens[0].token, Token::Identifier("stated".to_string()));
    assert_eq!(tokens[1].token, Token::Identifier("derived".to_string()));
    run(r#"
stated = 4
derived = 7
assert(stated, 4)
assert(derived, 7)
assert(stated + derived, 11)
compact = {stated: stated derived: derived}
assert(compact.stated, 4)
assert(compact.derived, 7)
compound = {sum: stated + derived next: stated}
assert(compound.sum, 11)
assert(compound.next, 4)
unary = {first: -stated next: derived}
assert(unary.first, -4)
assert(unary.next, 7)
def stated(value) => value + 1
def derived(value) => value * 2
assert(stated(3), 4)
assert(derived(3), 6)
record = {stated: "fact", derived: "conclusion"}
assert(record.stated, "fact")
assert(record.derived, "conclusion")
class Operations:
    def stated(value) => value + 3
    def derived(value) => value * 3
operations = Operations()
assert(operations.stated(2), 5)
assert(operations.derived(2), 6)
def identity(stated, derived) => [stated, derived]
assert(identity(2, 3), [2, 3])
assert([1, 2].map(stated => stated + 1), [2, 3])
assert([1, 2].map(derived => derived * 2), [2, 4])
def invoke(stated, derived) => [stated(2), derived(3)]
assert(invoke(stated, derived), [3, 6])
def package(stated, derived) => {fact: stated conclusion: derived}
assert(package(5, 6), {fact: 5, conclusion: 6})
"#);
}

#[test]
fn new_names_preserve_complete_comma_free_object_values_at_every_expression_boundary() {
    for (without_comma, with_comma) in [
        (
            "record = {first: stated next: derived}",
            "record = {first: stated, next: derived}",
        ),
        (
            "record = {first: seed + stated next: derived}",
            "record = {first: seed + stated, next: derived}",
        ),
        (
            "record = {first: -derived next: stated}",
            "record = {first: -derived, next: stated}",
        ),
        (
            "record = {first: copy = stated next: derived}",
            "record = {first: copy = stated, next: derived}",
        ),
        (
            "record = {first: () => derived next: stated}",
            "record = {first: () => derived, next: stated}",
        ),
        (
            "record = {first: item => item + stated next: derived}",
            "record = {first: item => item + stated, next: derived}",
        ),
    ] {
        assert_eq!(
            parse(without_comma).unwrap(),
            parse(with_comma).unwrap(),
            "{without_comma}"
        );
    }
    run(r#"
seed = 1
value = seed + 1
record = {facts: stated seed conclusions: derived value}
assert(record.facts[0].node, "seed")
assert(record.conclusions[0].node, "value")
if stated seed:
    assert(seed, 1)
"#);
}

#[test]
fn bound_new_stage_names_retain_the_not_an_operation_diagnostic() {
    for name in ["stated", "derived"] {
        for declaration in [
            format!("{name} = 0"),
            format!("{name} = null"),
            format!("{name} = value => value + 1"),
            format!("def {name}(value) => value + 1"),
            format!("class {name}"),
        ] {
            let mut runtime = run(&format!("seed = 1\n{declaration}"));
            for source in ["seed", "missing"] {
                let error = runtime.run(&format!("{source} |> {name}")).unwrap_err();
                assert_eq!(error.kind(), ErrorKind::Type);
                assert_eq!(error.message(), format!("{name} is not an operation"));
            }
            assert_eq!(runtime.run("seed").unwrap().to_string(), "1");
        }
    }
    run(r#"
seed = 1
def facts(stated):
    return seed |> stated
def conclusions(derived):
    return seed |> derived
try:
    facts(null)
catch error:
    assert(error, TypeError("stated is not an operation"))
try:
    conclusions(null)
catch error:
    assert(error, TypeError("derived is not an operation"))
{
    stated = 5
    derived = 6
    assert(stated, 5)
    assert(derived, 6)
    try:
        seed |> stated
    catch error:
        assert(error, TypeError("stated is not an operation"))
    try:
        seed |> derived
    catch error:
        assert(error, TypeError("derived is not an operation"))
}
"#);
}

#[test]
fn accepted_why_and_affects_shadowing_behavior_is_not_changed() {
    run(r#"
why = 4
affects = 5
seed = 1
value = seed + 1
assert(why, 4)
assert(affects, 5)
assert((why value).map(step => step.node), ["value", "seed"])
assert((seed |> affects).map(step => step.node), ["value"])
"#);
}

#[test]
fn nested_filters_compose_as_graph_selections_without_implicit_why() {
    run(r#"
seed = 1
middle = seed + 1
result = middle * 2
assert((stated seed).map(step => step.node), ["seed"])
assert(stated result, [])
assert((derived result).map(step => step.node), ["result"])
assert(((result |> why) |> stated).map(step => step.node), ["seed"])
assert(((result |> why) |> derived).map(step => step.node), ["result", "middle"])
assert(((result |> why |> stated) |> affects |> derived).map(step => step.node), ["middle", "result"])
assert((result |> why |> stated |> derived), [])
assert((result |> why |> derived |> why |> stated).map(step => step.node), ["seed"])
assert((why result).filter(step => step.state == "stated").map(step => step.node), ["seed"])
"#);
}

#[test]
fn excluded_steps_retain_value_and_shape_observations() {
    let runtime = run(r#"
seed = 1
value = seed + 1
facts = value |> why |> stated
conclusions = value |> why |> derived
assert(facts.map(step => step.node), ["seed"])
assert(conclusions.map(step => step.node), ["value"])
"#);
    for observer in ["facts", "conclusions"] {
        let observer = NodeKey::variable(observer);
        let node = runtime.graph.retrieve(&observer).unwrap();
        for source in ["seed", "value"] {
            assert!(node.dependencies.contains(&NodeKey::variable(source)));
            assert!(
                runtime
                    .graph
                    .shape_dependencies(&observer)
                    .any(|shape| shape == &ShapeKey::Node(NodeKey::variable(source)))
            );
        }
    }
}

#[test]
fn initially_excluded_references_and_equal_value_ancestry_stay_live() {
    run(r#"
left = 1
right = 1
value = 2
facts = value |> why |> stated
conclusions = derived value
assert(facts.map(step => step.node), ["value"])
assert(conclusions, [])
value = left + 1
assert(facts.map(step => step.node), ["left"])
assert(conclusions[0].from, ["left"])
value = right + 1
assert(facts.map(step => step.node), ["right"])
assert(conclusions[0].from, ["right"])
left = 5
assert(conclusions[0].holds, 2)
right = 3
assert(facts[0].holds, 3)
assert(conclusions[0].holds, 4)
value = 4
assert(facts.map(step => step.node), ["value"])
assert(conclusions, [])
"#);
}

#[test]
fn empty_property_filters_follow_deletion_redefinition_and_reference_replacement() {
    run(r#"
class Item
first = Item()
second = Item()
selected = first
facts = selected.amount |> stated
conclusions = selected.amount |> derived
assert(facts, [])
assert(conclusions, [])
first.amount = 2
second.amount = 2
assert(facts, [])
assert(conclusions[0].node, "first.amount")
assert(conclusions[0].state, "derived")
assert(conclusions[0].from, [])
selected = second
assert(conclusions[0].node, "second.amount")
seed = 1
second.amount = seed + 1
assert(facts, [])
assert(conclusions[0].holds, 2)
delete second.amount
assert(facts, [])
assert(conclusions, [])
second.amount = 3
assert(facts, [])
assert(conclusions[0].holds, 3)
"#);
}

#[test]
fn model_filters_keep_membership_order_and_observers_excluded() {
    run(r#"
facts = model |> stated
conclusions = model |> derived
assert(facts, [])
assert(conclusions, [])
seed = 1
value = seed + 1
assert(facts.map(step => step.node), ["seed"])
assert(conclusions.map(step => step.node), ["value"])
trace = value |> why |> stated
assert(facts.map(step => step.node), ["seed"])
trace = seed + 1
assert(conclusions.map(step => step.node), ["value", "trace"])
trace = value |> why |> stated
assert(conclusions.map(step => step.node), ["value"])
value = 2
assert(facts.map(step => step.node), ["seed", "value"])
assert(conclusions, [])
delete value
assert(facts.map(step => step.node), ["seed"])
"#);
}

#[test]
fn functions_class_rules_and_snapshots_keep_source_observations_after_rollback() {
    run(r#"
seed = 1
other = 1
value = seed + 1
def supporting() => value |> why |> stated
facts = supporting()
class Report
report = Report()
$Report.facts = value |> why |> stated
frozen = facts.value
assert(facts[0].node, "seed")
assert(report.facts[0].node, "seed")
try:
    value = other + 1
    throw "ABORT_FILTER"
catch error:
    assert(error, "ABORT_FILTER")
assert(facts[0].node, "seed")
assert(report.facts[0].node, "seed")
other = 5
assert(facts[0].holds, 1)
value = 2
assert(facts[0].node, "value")
assert(report.facts[0].node, "value")
value = other + 1
assert(facts[0].node, "other")
assert(facts[0].holds, 5)
assert(report.facts[0].holds, 5)
assert(frozen[0].node, "seed")
assert(frozen[0].holds, 1)
assert((model |> why).some(step => step.node == "facts"), false)
assert((model |> why).some(step => step.node == "report.facts"), false)
"#);
}

#[test]
fn existing_reference_type_and_call_errors_remain_visible_and_transaction_safe() {
    let mut runtime = run(r#"
seed = 1
value = seed + 1
facts = value |> why |> stated
try:
    value = 2
    1 |> stated
catch error:
    assert(error, TypeError("Cannot explain 1"))
try:
    1 |> derived
catch error:
    assert(error, TypeError("Cannot explain 1"))
assert(facts[0].node, "seed")
try:
    stated missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
try:
    derived missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
try:
    stated(1)
catch error:
    assert(error, ReferenceError("stated is not defined"))
try:
    derived(1)
catch error:
    assert(error, ReferenceError("derived is not defined"))
seed = 3
assert(facts[0].holds, 3)
"#);
    let error = runtime.run("value |> unknown").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "unknown is not an operation");
    assert_eq!(runtime.run("facts[0].holds").unwrap().to_string(), "3");
}

const ACCEPTED: &str = "63ada8be51e92e0d09f5b37d2855104840a970f6";
const DOCUMENTS: &[(&str, &str)] = &[
    ("spec", "nucleoid.spec.md"),
    ("reference", "tests/reference.md"),
    ("readme", "README.md"),
    ("examples", "docs/examples.md"),
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
struct Identity {
    document: String,
    title: String,
}

#[derive(Deserialize)]
struct Baseline {
    metric: String,
    revision: String,
    cases: Vec<ProtectedCase>,
}

#[derive(Deserialize)]
struct ProtectedCase {
    id: Identity,
    assertions: Vec<String>,
    expected: Option<String>,
}

#[derive(Deserialize)]
struct ExtraSource {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

fn artifact<T: serde::de::DeserializeOwned>(variable: &str) -> T {
    let path = std::env::var_os(variable).unwrap_or_else(|| panic!("{variable} is required"));
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn insert(cases: &mut BTreeMap<Identity, common::Case>, document: &str, case: common::Case) {
    let id = Identity {
        document: document.to_string(),
        title: case.title.clone(),
    };
    assert!(cases.insert(id.clone(), case).is_none(), "duplicate {id:?}");
}

fn sources(accepted: bool) -> BTreeMap<Identity, common::Case> {
    let mut cases = BTreeMap::new();
    for &(document, path) in DOCUMENTS {
        let source = if accepted {
            // Read only the four exact pinned Git documents, never a worktree
            // from another task or any environment file.
            let output = Command::new("git")
                .args(["--no-pager", "show", &format!("{ACCEPTED}:{path}")])
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            String::from_utf8(output.stdout).unwrap()
        } else {
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
        };
        let parsed = match document {
            "spec" | "reference" => common::cases(&source),
            _ => common::snippets(&source, path),
        };
        for case in parsed {
            insert(&mut cases, document, case);
        }
    }
    if accepted {
        assert_eq!(cases.len(), 624);
        let extras: Vec<ExtraSource> = artifact("NUC48_EXTRA_BASELINES");
        assert_eq!(extras.len(), 9);
        for extra in extras {
            insert(
                &mut cases,
                &extra.document,
                common::Case {
                    title: extra.title,
                    source: extra.source,
                    expected: extra.expected,
                },
            );
        }
    }
    assert_eq!(cases.len(), 633);
    cases
}

fn program(case: &common::Case) -> &str {
    let source = case.source.trim();
    source
        .strip_prefix(&format!("# {}", case.title))
        .unwrap_or(source)
        .trim()
}

fn replay(case: &common::Case, protected: &ProtectedCase) -> usize {
    assert_eq!(case.expected, protected.expected, "{:?}", protected.id);
    let assertions = common::assertions::protected_assertions(&parse(&case.source).unwrap());
    assert_eq!(assertions, protected.assertions, "{:?}", protected.id);
    let mut runtime = Runtime::new();
    common::check_with_count(&mut runtime, case, assertions.len())
        .unwrap_or_else(|error| panic!("{:?}: {error}", protected.id));
    runtime.assertions_run()
}

#[test]
#[ignore = "requires immutable NUC48_BASELINE and NUC48_EXTRA_BASELINES"]
fn all_633_unchanged_accepted_sources_and_candidate_forms_preserve_behavior() {
    let baseline: Baseline = artifact("NUC48_BASELINE");
    assert_eq!(baseline.revision, ACCEPTED);
    assert_eq!(baseline.metric, "nucleoid-lexer-with-layout-v1");
    assert_eq!(baseline.cases.len(), 633);
    let accepted = sources(true);
    let candidate = sources(false);
    assert!(accepted.keys().eq(candidate.keys()));
    let extras: Vec<ExtraSource> = artifact("NUC48_EXTRA_BASELINES");
    let rewritten: BTreeSet<_> = extras
        .into_iter()
        .map(|extra| Identity {
            document: extra.document,
            title: extra.title,
        })
        .collect();
    let mut identities = BTreeSet::new();
    let mut total_assertions = 0;
    let mut returns = 0;
    let mut changed = 0;
    for protected in &baseline.cases {
        assert!(identities.insert(protected.id.clone()));
        let original = accepted.get(&protected.id).unwrap();
        let current = candidate.get(&protected.id).unwrap();
        let before_coverage = replay(original, protected);
        let coverage = replay(current, protected);
        assert_eq!(before_coverage, coverage, "{:?}", protected.id);
        if rewritten.contains(&protected.id) {
            assert_ne!(program(original), program(current), "{:?}", protected.id);
            changed += 1;
        } else {
            assert_eq!(program(original), program(current), "{:?}", protected.id);
        }
        total_assertions += coverage;
        returns += usize::from(protected.expected.is_some());
    }
    assert_eq!(changed, 9);
    println!(
        "Compatibility replay: all 624 pinned sources plus 9 exact extra accepted-syntax \
         sources executed unchanged under the candidate; all 633 candidate cases also ran. \
         Each side executed {total_assertions} assertions and matched {returns} explicit \
         returns; identities, canonical assertions/order and actual coverage match."
    );
}
