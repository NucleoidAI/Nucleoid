//! One-level reactive projection and unchanged accepted-syntax controls.
//! Ordinary tests use embedded sources, not Git or session discovery.

mod common;

use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Ast, parse, parse_expression};
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};
use serde::Deserialize;

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "flatMap regression".to_string(),
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

fn dependencies<'a>(runtime: &'a Runtime, name: &str) -> Vec<&'a str> {
    runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap()
        .dependencies
        .iter()
        .map(NodeKey::as_str)
        .collect()
}

fn documents() -> [(&'static str, Vec<common::Case>); 4] {
    [
        ("spec", common::cases(include_str!("../nucleoid.spec.md"))),
        ("reference", common::cases(include_str!("reference.md"))),
        (
            "readme",
            common::snippets(include_str!("../README.md"), "README.md"),
        ),
        (
            "examples",
            common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
        ),
    ]
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedCase {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

// Exact complete V2 accepted counterparts, already executed before the
// candidate existed. This is embedded test data, not a session dependency.
const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid flat maps lists by one level",
    "source": "# Nucleoid flat maps lists by one level\n\nnumbers = [1, 2, 3]\ndef expand(value) => [value, value * 10]\nexpanded = numbers.map(expand).reduce((joined, item) => joined.concat(item), [])\nassert(expanded, [1, 10, 2, 20, 3, 30])\nassert(numbers, [1, 2, 3])\nmixed = [[[1]], [2, 3], 4, \"xy\", []]\nflattened = mixed.map(item => item).reduce((joined, item) => joined.concat(item), [])\nassert(flattened, [[1], 2, 3, 4, \"xy\"])\nassert(mixed[0], [[1]])\nnullable = numbers.map(item => [item, null]).reduce((joined, item) => joined.concat(item), [])\nassert(nullable, [1, null, 2, null, 3, null])\nempty = [].map(item => missing).reduce((joined, item) => joined.concat(item), [])\nassert(empty, [])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid keeps flat mapped lists reactive",
    "source": "# Nucleoid keeps flat mapped lists reactive\n\nnumbers = [1, 2]\nfactor = 2\ndef expand(value) => [value, value * factor]\nexpanded = numbers.map(expand).reduce((joined, item) => joined.concat(item), [])\nassert(expanded, [1, 2, 2, 4])\nfactor = 3\nassert(expanded, [1, 3, 2, 6])\nnumbers.push(4)\nassert(expanded, [1, 3, 2, 6, 4, 12])\nnumbers.pop()\nassert(expanded, [1, 3, 2, 6])\ndef expand(value) => [value + factor]\nassert(expanded, [4, 5])\nassert(numbers, [1, 2])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid flat maps class populations",
    "source": "# Nucleoid flat maps class populations\n\nclass Batch(items):\n    this.items = items\nbatch1 = Batch([1, 2])\nbatch2 = Batch([3])\nexpanded = Batch.map(batch => batch.items).reduce((joined, item) => joined.concat(item), [])\nassert(expanded, [1, 2, 3])\nbatch1.items.push(4)\nassert(expanded, [1, 2, 4, 3])\nbatch3 = Batch([5])\nassert(expanded, [1, 2, 4, 3, 5])\ntry:\n    delete batch2\ncatch error:\n    assert(error, TypeError(\"Cannot delete object 'batch2'\"))\nassert(expanded, [1, 2, 4, 3, 5])\nbatch2.items = []\nassert(expanded, [1, 2, 4, 5])\nassert(Batch.length, 3)\nassert(batch1.items, [1, 2, 4])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid preserves flat mapping callable snapshots",
    "source": "# Nucleoid preserves flat mapping callable snapshots\n\nnumbers = [1, 2]\nfactor = 2\ndef expand(value) => [value * factor]\ncallback = expand\nfrozen = expand.value\nlive = numbers.map(callback).reduce((joined, item) => joined.concat(item), [])\nsaved = numbers.map(frozen).reduce((joined, item) => joined.concat(item), [])\nsnapshot = live.value\nassert(live, [2, 4])\nassert(saved, [2, 4])\nassert(snapshot, [2, 4])\ndef expand(value) => [value + factor]\nassert(live, [3, 4])\nassert(saved, [2, 4])\nfactor = 3\nassert(live, [4, 5])\nassert(saved, [3, 6])\nassert(snapshot, [2, 4])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back flat mapping callback failures",
    "source": "# Nucleoid rolls back flat mapping callback failures\n\nnumbers = [1, 2]\ndef expand(value) => [value, value * 2]\nexpanded = numbers.map(expand).reduce((joined, item) => joined.concat(item), [])\nassert(expanded, [1, 2, 2, 4])\ntry:\n    def expand(value):\n        throw \"FLAT_MAPPING_ABORT\"\ncatch error:\n    assert(error, \"FLAT_MAPPING_ABORT\")\nassert(expanded, [1, 2, 2, 4])\nassert(numbers, [1, 2])\nnumbers.push(3)\nassert(expanded, [1, 2, 2, 4, 3, 6])\n21",
    "expected": "21"
  },
  {
    "document": "reference",
    "title": "Nucleoid flat maps live related collections",
    "source": "# Nucleoid flat maps live related collections\n\nclass Team(tags):\n    this.tags = tags\nteam1 = Team([\"red\", \"blue\"])\nteam2 = Team([\"green\"])\ntags = Team.map(team => team.tags).reduce((joined, item) => joined.concat(item), [])\nassert(tags, [\"red\", \"blue\", \"green\"])\nteam1.tags = [\"gold\"]\nassert(tags, [\"gold\", \"green\"])\nteam2.tags.push(\"silver\")\nassert(tags, [\"gold\", \"green\", \"silver\"])",
    "expected": null
  },
  {
    "document": "readme",
    "title": "Flat mapping produces zero or many results per input",
    "source": "numbers = [1, 2, 3]\ndef copies(value):\n    if value > 1:\n        return [value, value * 10]\n    return []\nexpanded = numbers.map(copies).reduce((joined, item) => joined.concat(item), [])\nassert(expanded, [2, 20, 3, 30])\nassert(numbers, [1, 2, 3])\nnumbers.push(4)\nassert(expanded, [2, 20, 3, 30, 4, 40])",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Flattened batches retain live object identities",
    "source": "class Task(label):\n    this.label = label\ntask1 = Task(\"alpha\")\ntask2 = Task(\"beta\")\nbatches = [[task1], [task2, task1]]\nflattened = batches.map(batch => batch).reduce((joined, item) => joined.concat(item), [])\nlabels = flattened.map(task => task.label)\nassert(labels, [\"alpha\", \"beta\", \"alpha\"])\ntask1.label = \"gamma\"\nassert(labels, [\"gamma\", \"beta\", \"gamma\"])\nfirst = flattened[0]\nfirst.label = \"delta\"\nassert(task1.label, \"delta\")\nassert(labels, [\"delta\", \"beta\", \"delta\"])\ntask3 = Task(\"third\")\nbatches.push([task3])\nassert(labels, [\"delta\", \"beta\", \"delta\", \"third\"])\nassert(batches.length, 3)",
    "expected": null
  }
]"###;

const REWRITES: &[(&str, &str)] = &[
    (
        "numbers.map(expand).reduce((joined, item) => joined.concat(item), [])",
        "numbers.flatMap(expand)",
    ),
    (
        "mixed.map(item => item).reduce((joined, item) => joined.concat(item), [])",
        "mixed.flatMap(item => item)",
    ),
    (
        "numbers.map(item => [item, null]).reduce((joined, item) => joined.concat(item), [])",
        "numbers.flatMap(item => [item, null])",
    ),
    (
        "[].map(item => missing).reduce((joined, item) => joined.concat(item), [])",
        "[].flatMap(item => missing)",
    ),
    (
        "Batch.map(batch => batch.items).reduce((joined, item) => joined.concat(item), [])",
        "Batch.flatMap(batch => batch.items)",
    ),
    (
        "numbers.map(callback).reduce((joined, item) => joined.concat(item), [])",
        "numbers.flatMap(callback)",
    ),
    (
        "numbers.map(frozen).reduce((joined, item) => joined.concat(item), [])",
        "numbers.flatMap(frozen)",
    ),
    (
        "Team.map(team => team.tags).reduce((joined, item) => joined.concat(item), [])",
        "Team.flatMap(team => team.tags)",
    ),
    (
        "numbers.map(copies).reduce((joined, item) => joined.concat(item), [])",
        "numbers.flatMap(copies)",
    ),
    (
        "batches.map(batch => batch).reduce((joined, item) => joined.concat(item), [])",
        "batches.flatMap(batch => batch)",
    ),
];

#[test]
fn all_eight_exact_accepted_counterparts_keep_only_twelve_operation_rewrites_and_actual_coverage() {
    let sealed: Vec<SealedCase> = serde_json::from_str(ACCEPTED).unwrap();
    let documents = documents();
    assert_eq!(sealed.len(), 8);
    let mut totals = (0, 0, 0);
    for (index, sealed) in sealed.iter().enumerate() {
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == sealed.document)
            .unwrap()
            .1;
        let matches = cases
            .iter()
            .enumerate()
            .filter(|(_, case)| case.title == sealed.title)
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "{}", sealed.title);
        let (position, case) = matches[0];
        assert_eq!(position, [378, 379, 380, 381, 382, 76, 27, 82][index]);
        assert_eq!(case.expected, sealed.expected);
        let mut source = sealed.source.clone();
        let mut changes = 0;
        for &(old, new) in REWRITES {
            changes += source.matches(old).count();
            source = source.replace(old, new);
        }
        assert_eq!(changes, [4, 1, 1, 2, 1, 1, 1, 1][index]);
        assert_eq!(case.source.trim(), source, "all non-operation source bytes");
        let control_ast = parse(&sealed.source).unwrap();
        let actual_ast = parse(&case.source).unwrap();
        assert_ne!(
            control_ast, actual_ast,
            "runtime-call RHSs intentionally differ"
        );
        let assertions = common::assertions::protected_assertions(&control_ast);
        assert_eq!(assertions.len(), [6, 6, 8, 8, 5, 3, 3, 6][index]);
        assert_eq!(
            common::assertions::protected_assertions(&actual_ast),
            assertions
        );
        let control = common::Case {
            title: sealed.title.clone(),
            source: sealed.source.clone(),
            expected: sealed.expected.clone(),
        };
        let mut runtimes = [Runtime::new(), Runtime::new()];
        for (runtime, form) in runtimes.iter_mut().zip([case, &control]) {
            common::check_with_count(runtime, form, assertions.len())
                .unwrap_or_else(|reason| panic!("{}: {reason}", sealed.title));
            assert_eq!(runtime.assertions_run(), assertions.len());
        }
        let outputs: &[&str] = match index {
            0 => &[
                "numbers",
                "expanded",
                "mixed",
                "flattened",
                "nullable",
                "empty",
            ],
            1 | 4 | 6 => &["numbers", "expanded"],
            2 => &["expanded"],
            3 => &["numbers", "factor", "live", "saved", "snapshot"],
            5 => &["tags"],
            7 => &["batches", "flattened", "labels"],
            _ => unreachable!(),
        };
        // Compare observable named outputs, not definitions/graph source of
        // the deliberately rewritten runtime-call RHSs.
        for name in outputs {
            assert_eq!(
                runtimes[0]
                    .serialize_json(runtimes[0].state.variable(name).unwrap())
                    .unwrap(),
                runtimes[1]
                    .serialize_json(runtimes[1].state.variable(name).unwrap())
                    .unwrap(),
                "{name}",
            );
        }
        totals.0 += assertions.len();
        totals.1 += usize::from(sealed.expected.is_some());
        totals.2 += changes;
    }
    assert_eq!(totals, (45, 1, 12));
}

// Length-delimited FNV-1a regression seals from a read-only audit using the
// actual common extractors and pinned 1d416f7 snapshot. These are not lexer
// token counts or security digests. Future case tails remain extendable.
fn seal_field(hash: &mut u64, bytes: &[u8]) {
    for byte in (bytes.len() as u64).to_le_bytes().iter().chain(bytes) {
        *hash = (*hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
}

fn history_signatures(cases: &[common::Case]) -> [u64; 3] {
    let mut hashes = [0xcbf2_9ce4_8422_2325; 3];
    for case in cases {
        for text in [&case.title, &case.source, &format!("{:?}", case.expected)] {
            seal_field(&mut hashes[0], text.as_bytes());
        }
        let ast = parse(&case.source).unwrap();
        seal_field(&mut hashes[1], format!("{ast:?}").as_bytes());
        for assertion in common::assertions::protected_assertions(&ast) {
            seal_field(&mut hashes[2], assertion.as_bytes());
        }
    }
    hashes
}

#[test]
fn all_563_historical_sources_keep_exact_boundaries_order_comments_asts_assertions_and_returns() {
    let expected = [
        (
            378,
            [
                2_566_519_894_396_707_985,
                11_086_541_957_152_234_801,
                13_192_226_042_441_415_281,
            ],
        ),
        (
            76,
            [
                7_588_065_465_599_684_719,
                4_233_585_456_964_045_452,
                11_825_209_885_931_979_957,
            ],
        ),
        (
            27,
            [
                6_085_232_326_361_796_888,
                8_308_489_451_378_322_214,
                8_511_435_508_874_224_710,
            ],
        ),
        (
            82,
            [
                14_507_179_690_735_124_957,
                11_589_942_919_484_399_866,
                5_998_861_743_361_075_651,
            ],
        ),
    ];
    let mut total = 0;
    for ((document, cases), (count, signatures)) in documents().into_iter().zip(expected) {
        assert!(cases.len() >= count, "{document}: missing history");
        assert_eq!(
            history_signatures(&cases[..count]),
            signatures,
            "{document}"
        );
        total += count;
    }
    assert_eq!(total, 563);
}

#[test]
fn one_level_projection_preserves_scalars_nested_lists_duplicates_and_read_only_input() {
    run(r#"
values = [[[1]], [], [2, 2], 3, "ab", true, false]
expanded = values.flatMap(item => item)
assert(expanded, [[1], 2, 2, 3, "ab", true, false])
assert(values, [[[1]], [], [2, 2], 3, "ab", true, false])
assert(values.length, 7)
assert([1, 2].flatMap(item => [[item]]), [[1], [2]])
assert([1, 2].flatMap(item => []), [])
assert([1, 2].flatMap(item => item * 3), [3, 6])
assert([1, 2].flatMap(function(item) { return [item, item] }), [1, 1, 2, 2])
assert([1, 2].flatMap(item => { return [item] }), [1, 2])
empty = [].flatMap(item => missing)
assert(empty, [])
"#);
}

#[test]
fn callbacks_run_once_per_item_in_order_without_an_extra_receiver_or_index_argument() {
    let runtime = run(r#"
log = Object()
log.order = 0
expanded = [1, 2, 3].flatMap(item => [log.order = log.order.value * 10 + item, item])
assert(log.order, 123)
assert(expanded, [1, 1, 12, 2, 123, 3])
item = 99
assert([1, 2].flatMap(item => [item]), [1, 2])
assert(item, 99)
def unused(item, index, receiver) => [item]
assert([1, 2].flatMap(unused), [1, 2])
"#);
    assert!(!runtime.graph.contains(&NodeKey::variable("index")));
    assert!(!runtime.graph.contains(&NodeKey::variable("receiver")));
    let mut runtime = Runtime::new();
    let actual = checked(
        &mut runtime,
        "[1, 2].flatMap((item, index, receiver) => [item, index, receiver])",
        None,
    );
    assert_eq!(
        actual,
        Value::List(vec![
            Value::number(1),
            Value::Null,
            Value::Null,
            Value::number(2),
            Value::Null,
            Value::Null,
        ])
    );
    checked(
        &mut runtime,
        "assigned = [1].flatMap((item, extra) => [item, extra])",
        None,
    );
    assert_eq!(runtime.state.variable("assigned"), Some(&Value::Null));
}

#[test]
fn object_function_class_date_and_regex_values_keep_their_actual_identities() {
    let runtime = run(r#"
class Entry
entry = Entry()
def callback(item) => item * 2
values = [entry, callback, Entry, Date(3), /same/, [[1]]]
expanded = values.flatMap(item => [item, item])
assert(expanded.length, 12)
first = expanded[0]
first.label = "live"
assert(entry.label, "live")
assert(expanded[1].label, "live")
assert(expanded[2](3), 6)
assert(expanded[3](4), 8)
assert(values.length, 6)
"#);
    let Value::List(input) = runtime.state.variable("values").unwrap() else {
        panic!("list")
    };
    let Value::List(output) = runtime.state.variable("expanded").unwrap() else {
        panic!("list")
    };
    for (item, copies) in input.iter().zip(output.chunks_exact(2)) {
        for copy in copies {
            assert_eq!(copy, item);
            match (item, copy) {
                (Value::Function(left), Value::Function(right)) => {
                    assert!(Arc::ptr_eq(left, right))
                }
                (Value::Regex(left), Value::Regex(right)) => assert!(Arc::ptr_eq(left, right)),
                _ => {}
            }
        }
    }
}

#[test]
fn returned_null_no_return_and_undefined_are_not_artificially_read_or_settled() {
    let runtime = run(r#"
literal = [1, 2].flatMap(item => null)
withNull = [1, 2].flatMap(item => [item, null])
def noReturn(item):
    pass
implicit = [1, 2].flatMap(noReturn)
def bareReturn(item):
    return
bare = [1, 2].flatMap(bareReturn)
undefinedItems = [1, 2].flatMap(item => [][0])
assert(literal, [null, null])
assert(withNull, [1, null, 2, null])
assert(implicit, [null, null])
assert(bare, [null, null])
"#);
    for name in ["literal", "implicit", "bare"] {
        assert_eq!(
            runtime.state.variable(name),
            Some(&Value::List(vec![Value::Null, Value::Null]))
        );
    }
    assert_eq!(
        runtime.state.variable("undefinedItems"),
        Some(&Value::List(vec![Value::Undefined, Value::Undefined,]))
    );
    assert_eq!(
        runtime
            .serialize_json(runtime.state.variable("undefinedItems").unwrap())
            .unwrap(),
        "[null,null]"
    );
}

#[test]
fn actual_null_reads_keep_assignment_tracking_instead_of_claiming_reducer_equivalence() {
    let mut runtime = run(r#"
nullable = null
literal = [1, 2].flatMap(item => null)
read = [1, 2].flatMap(item => nullable)
parameterRead = [null].flatMap(item => [item])
legacy = [1, 2].map(item => null).reduce((joined, item) => joined.concat(item), [])
assert(literal, [null, null])
assert(read, null)
assert(parameterRead, null)
assert(legacy, null)
"#);
    assert_eq!(
        runtime.state.variable("literal"),
        Some(&Value::List(vec![Value::Null, Value::Null]))
    );
    assert_eq!(runtime.state.variable("legacy"), Some(&Value::Null));
    assert!(dependencies(&runtime, "read").contains(&"nullable"));
    checked(&mut runtime, "nullable = 7\nassert(read, [7, 7])", None);
    checked(&mut runtime, "nullable = null\nassert(read, null)", None);
    checked(&mut runtime, "nullable = []\nassert(read, [])", None);
}

#[test]
fn undefined_member_results_real_null_members_deletion_and_recovery_follow_existing_reads() {
    let mut runtime = run(r#"
class Probe
probe = Probe()
expanded = [1, 2].flatMap(item => probe.items)
"#);
    assert_eq!(
        runtime.state.variable("expanded"),
        Some(&Value::List(vec![Value::Undefined, Value::Undefined,]))
    );
    let property = NodeKey::property(&ObjectId::named("probe"), "items");
    assert!(
        runtime
            .graph
            .reaches(&property, &NodeKey::variable("expanded"))
    );
    checked(
        &mut runtime,
        "probe.items = [3]\nassert(expanded, [3, 3])",
        None,
    );
    checked(
        &mut runtime,
        "probe.items = null\nassert(expanded, null)",
        None,
    );
    checked(&mut runtime, "delete probe.items", None);
    assert_eq!(
        runtime.state.variable("expanded"),
        Some(&Value::List(vec![Value::Undefined, Value::Undefined,]))
    );
    checked(&mut runtime, "probe.items = []\nassert(expanded, [])", None);
    checked(
        &mut runtime,
        "probe.items = 4\nassert(expanded, [4, 4])",
        None,
    );
    assert!(
        runtime
            .graph
            .reaches(&property, &NodeKey::variable("expanded"))
    );
}

#[test]
fn undefined_or_null_receivers_skip_arguments_then_recover_through_the_existing_route() {
    run(r#"
class Probe
probe = Probe()
log = Object()
log.calls = 0
expanded = probe.items.flatMap(item => [item], log.calls = log.calls.value + 1)
assert(expanded, null)
assert(log.calls, 0)
probe.items = [1, 2]
assert(expanded, [1, 2])
assert(log.calls, 1)
delete probe.items
assert(expanded, null)
assert(log.calls, 1)
probe.items = [3]
assert(expanded, [3])
assert(log.calls, 2)
nullable = null
skipped = nullable.flatMap(missing, log.calls = 99)
assert(skipped, null)
assert(log.calls, 2)
"#);
}

#[test]
fn callback_validation_is_typed_even_for_empty_lists_and_empty_populations() {
    for receiver in ["[]", "[1]", "Entry"] {
        for callback in [
            "",
            "0",
            "false",
            "\"function\"",
            "null",
            "[]",
            "{}",
            "Date(0)",
            "/callback/",
            "Entry",
            "probe.absent",
        ] {
            let mut runtime = run("class Entry\nclass Probe\nprobe = Probe()");
            let error = runtime
                .run(&format!("{receiver}.flatMap({callback})"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Type, "{receiver}/{callback}");
            assert_eq!(error.message(), "flatMap expects a function");
            assert_eq!(error.position(), Some(Position::new(1, 1)));
        }
    }
    run(r#"
class Empty
assert(Empty.flatMap(item => missing), [])
assert([].flatMap(item => missing), [])
"#);
}

#[test]
fn supplied_arguments_precede_validation_and_callbacks_including_unused_extras() {
    run(r#"
log = Object()
log.order = 0
expanded = [1, 2].flatMap(
    item => [log.order = log.order.value * 10 + item],
    log.order = 7,
    log.order = log.order.value * 10 + 8
)
assert(expanded, [781, 7812])
assert(log.order, 7812)
empty = [].flatMap(item => missing, log.order = 9)
assert(empty, [])
assert(log.order, 9)
"#);
    for receiver in ["[]", "[1]", "Entry"] {
        for (arguments, kind, message) in [
            ("0, missing", ErrorKind::Reference, "missing is not defined"),
            (
                "0, Math.random()",
                ErrorKind::Type,
                "Math.random is not supported",
            ),
            (
                "missing, Math.random()",
                ErrorKind::Reference,
                "missing is not defined",
            ),
            (
                "item => missing, Math.random()",
                ErrorKind::Type,
                "Math.random is not supported",
            ),
        ] {
            let mut runtime = run("class Entry");
            let error = runtime
                .run(&format!("\nanswer = {receiver}.flatMap({arguments})"))
                .unwrap_err();
            assert_eq!(error.kind(), kind);
            assert_eq!(error.message(), message);
            assert_eq!(error.position(), Some(Position::new(2, 1)));
            assert!(runtime.state.variable("answer").is_none());
        }
    }
}

#[test]
fn unused_extra_reads_retain_real_dependencies_null_tracking_and_errors() {
    let mut runtime = run(r#"
extra = 1
expanded = [1, 2].flatMap(item => [item], extra)
assert(expanded, [1, 2])
extra = null
assert(expanded, null)
extra = 4
assert(expanded, [1, 2])
class Probe
probe = Probe()
withMissingExtra = [1].flatMap(item => [item], probe.optional)
assert(withMissingExtra, [1])
probe.optional = null
assert(withMissingExtra, null)
probe.optional = []
assert(withMissingExtra, [1])
"#);
    assert!(dependencies(&runtime, "expanded").contains(&"extra"));
    assert!(dependencies(&runtime, "withMissingExtra").contains(&"probe.optional"));
    let error = runtime
        .run("[].flatMap(item => [item], missing)")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
}

#[test]
fn receiver_and_callback_values_are_snapshotted_before_later_argument_effects() {
    run(r#"
values = [1, 2]
expanded = values.flatMap(item => [item], values.push(3))
assert(values, [1, 2, 3])
assert(expanded, [1, 2])
other = [4, 5]
beforeReplacement = other.flatMap(item => [item], other = [9])
assert(other, [9])
assert(beforeReplacement, [4, 5])
callback = item => [item * 2]
savedArgument = [2, 3].flatMap(callback, callback = item => [99])
assert(savedArgument, [4, 6])
assert(callback(2), [99])
log = Object()
log.calls = 0
def receiver() => [log.calls = log.calls.value + 1, 2]
result = receiver().flatMap(item => [item])
assert(result, [1, 2])
assert(log.calls, 1)
class Entry
first = Entry()
population = Entry.flatMap(item => [1], Entry())
assert(population, [1])
assert(Entry.length, 2)
"#);
}

#[test]
fn callback_mutation_visits_only_the_original_receiver_snapshot() {
    let mut runtime = run("values = [1, 2]");
    let result = checked(
        &mut runtime,
        "values.flatMap(item => [values.push(9), item])",
        None,
    );
    assert_eq!(
        result,
        Value::List(vec![
            Value::number(3),
            Value::number(1),
            Value::number(4),
            Value::number(2),
        ])
    );
    assert_eq!(
        runtime.state.variable("values"),
        Some(&Value::List(vec![
            Value::number(1),
            Value::number(2),
            Value::number(9),
            Value::number(9),
        ]))
    );
}

#[test]
fn a_receiver_snapshot_preserves_object_handles_not_frozen_member_values() {
    run(r#"
class Entry(items):
    this.items = items
entry = Entry([1])
source = [entry]
expanded = source.flatMap(item => item.items, entry.items = [7, 8])
assert(expanded, [7, 8])
assert(source.length, 1)
assert(source[0], entry)
assert(entry.items, [7, 8])
"#);
}

#[test]
fn re_evaluation_replaces_obsolete_member_dependencies_and_empty_inputs_remove_reads() {
    let mut runtime = run(r#"
class Batch(items):
    this.items = items
first = Batch([1])
second = Batch([2])
source = [first]
expanded = source.flatMap(item => item.items)
assert(expanded, [1])
"#);
    assert_eq!(
        dependencies(&runtime, "expanded"),
        ["source", "first.items"]
    );
    checked(
        &mut runtime,
        "source = [second]\nassert(expanded, [2])",
        None,
    );
    assert_eq!(
        dependencies(&runtime, "expanded"),
        ["source", "second.items"]
    );
    assert!(!runtime.graph.reaches(
        &NodeKey::property(&ObjectId::named("first"), "items"),
        &NodeKey::variable("expanded"),
    ));
    checked(
        &mut runtime,
        r#"
first.items.push(9)
assert(expanded, [2])
second.items.push(3)
assert(expanded, [2, 3])
source = []
assert(expanded, [])
"#,
        None,
    );
    assert_eq!(dependencies(&runtime, "expanded"), ["source"]);
    checked(
        &mut runtime,
        "second.items = [4]\nassert(expanded, [])",
        None,
    );
    checked(
        &mut runtime,
        "source = [first, second]\nassert(expanded, [1, 9, 4])",
        None,
    );
    assert_eq!(
        dependencies(&runtime, "expanded"),
        ["source", "first.items", "second.items"]
    );
}

#[test]
fn empty_class_instances_can_leave_and_rejoin_the_live_population_without_changing_deletion_rules()
{
    let runtime = run(r#"
class Marker
first = Marker()
second = Marker()
expanded = Marker.flatMap(item => [1])
assert(expanded, [1, 1])
delete second
assert(expanded, [1])
third = Marker()
assert(expanded, [1, 1])
assert(Marker.length, 2)
"#);
    assert_eq!(dependencies(&runtime, "expanded"), ["$Marker"]);
    assert!(
        runtime
            .graph
            .reaches(&NodeKey::class("Marker"), &NodeKey::variable("expanded"))
    );
}

#[test]
fn class_member_rules_keep_instance_binding_missing_receiver_deletion_and_recovery() {
    let runtime = run(r#"
class Batch
$Batch.expanded = $Batch.items.flatMap(item => [item * $Batch.factor])
first = Batch()
second = Batch()
assert(first.expanded, null)
assert(second.expanded, null)
first.factor = 2
first.items = [1, 2]
second.factor = 3
second.items = [4]
assert(first.expanded, [2, 4])
assert(second.expanded, [12])
first.factor = 5
assert(first.expanded, [5, 10])
assert(second.expanded, [12])
delete first.items
assert(first.expanded, null)
first.items = [3]
assert(first.expanded, [15])
assert(second.expanded, [12])
"#);
    for name in ["first", "second"] {
        let id = ObjectId::named(name);
        for property in ["items", "factor"] {
            assert!(runtime.graph.reaches(
                &NodeKey::property(&id, property),
                &NodeKey::property(&id, "expanded"),
            ));
        }
    }
}

#[test]
fn live_named_callbacks_frozen_definitions_and_result_snapshots_keep_distinct_dependencies() {
    let runtime = run(r#"
values = [2, 3]
factor = 2
def expand(item) => [item * factor]
alias = expand
frozen = expand.value
live = values.flatMap(alias)
saved = values.flatMap(frozen)
snapshot = live.value
def expand(item) => [item + factor]
assert(live, [4, 5])
assert(saved, [4, 6])
assert(snapshot, [4, 6])
factor = 3
assert(live, [5, 6])
assert(saved, [6, 9])
assert(snapshot, [4, 6])
values.push(4)
assert(live, [5, 6, 7])
assert(saved, [6, 9, 12])
assert(snapshot, [4, 6])
"#);
    assert_eq!(
        dependencies(&runtime, "live"),
        ["values", "alias", "factor"]
    );
    assert_eq!(
        dependencies(&runtime, "saved"),
        ["values", "frozen", "factor"]
    );
    assert!(dependencies(&runtime, "snapshot").is_empty());
    assert!(
        runtime
            .graph
            .reaches(&NodeKey::function("expand"), &NodeKey::variable("live"))
    );
    assert!(
        !runtime
            .graph
            .reaches(&NodeKey::function("expand"), &NodeKey::variable("saved"))
    );
}

#[test]
fn flat_mapping_reasoning_steps_tracks_real_member_reads_and_live_structure() {
    run(r#"
source = 1
middle = source + 1
result = middle * 2
trace = why result
inputs = trace.flatMap(step => step.from)
assert(inputs, ["middle", "source"])
middle = 0
assert(inputs, ["middle"])
source = 4
assert(inputs, ["middle"])
middle = source + 2
assert(inputs, ["middle", "source"])
"#);
}

#[test]
fn function_scopes_non_capture_and_caller_receiver_remain_the_same_as_map() {
    for method in ["map", "flatMap"] {
        let mut runtime = Runtime::new();
        let error = runtime
            .run(&format!(
                "def factory(offset) => item => [item + offset]\n[1].{method}(factory(10))"
            ))
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "offset is not defined");
    }
    let expression =
        parse_expression("values.flatMap((item, extra) => [item, extra], 99)").unwrap();
    let Ast::Call(call) = Ast::convert(&expression) else {
        panic!("ordinary call AST")
    };
    assert_eq!(call.arguments().len(), 2);
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.push();
    scope.declare(
        "values",
        Value::List(vec![Value::number(1), Value::number(2)]),
    );
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    assert_eq!(
        call.resolve(&mut runtime, &mut scope).unwrap(),
        Value::List(vec![
            Value::number(1),
            Value::Null,
            Value::number(2),
            Value::Null,
        ])
    );
    let expression = parse_expression("values.flatMap(item => this)").unwrap();
    let error = Ast::convert(&expression)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "this is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(
        scope.retrieve("values"),
        Some(&Value::List(vec![Value::number(1), Value::number(2)]))
    );
}

#[test]
fn callback_failure_stops_iteration_and_rolls_back_effects_state_and_graph_before_recovery() {
    let mut runtime = run(r#"
values = [1, 2, 3]
log = Object()
log.count = 0
def abort(item):
    log.count = log.count.value + 1
    if item == 2:
        throw "CALLBACK_ABORT"
    return [item]
"#);
    let state = format!("{:?}", runtime.state);
    let keys = runtime.graph.keys().cloned().collect::<Vec<_>>();
    let error = runtime.run("expanded = values.flatMap(abort)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Thrown);
    assert_eq!(error.thrown_value(), Some(&Value::string("CALLBACK_ABORT")));
    assert_eq!(error.position(), Some(Position::new(1, 1)));
    assert_eq!(format!("{:?}", runtime.state), state);
    assert_eq!(runtime.graph.keys().cloned().collect::<Vec<_>>(), keys);
    checked(
        &mut runtime,
        r#"
assert(log.count, 0)
assert(values, [1, 2, 3])
def abort(item) => [item, item * 2]
expanded = values.flatMap(abort)
assert(expanded, [1, 2, 2, 4, 3, 6])
values.push(4)
assert(expanded, [1, 2, 2, 4, 3, 6, 4, 8])
"#,
        None,
    );
}

#[test]
fn failed_relationship_replacement_and_redefinition_restore_dependencies_then_recover() {
    let runtime = run(r#"
values = [1, 2]
factor = 2
alternative = [9]
def expand(item) => [item * factor]
expanded = values.flatMap(expand)
try:
    expanded = alternative.flatMap(0)
catch error:
    assert(error, TypeError("flatMap expects a function"))
assert(expanded, [2, 4])
try:
    def expand(item) => missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(expanded, [2, 4])
alternative.push(10)
assert(expanded, [2, 4])
factor = 3
assert(expanded, [3, 6])
try:
    values.push(4)
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(values, [1, 2])
assert(expanded, [3, 6])
values.push(3)
assert(expanded, [3, 6, 9])
"#);
    assert_eq!(
        dependencies(&runtime, "expanded"),
        ["values", "expand", "factor"]
    );
    assert!(!runtime.graph.reaches(
        &NodeKey::variable("alternative"),
        &NodeKey::variable("expanded")
    ));
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
}

#[test]
fn a_throwing_callback_preserves_its_exact_value_and_earlier_argument_effects_are_transactional() {
    let mut runtime =
        run("class Failure\nfailure = Failure()\nfailure.code = 7\nlog = Object()\nlog.calls = 0");
    let Value::Object(log) = runtime.state.variable("log").unwrap().clone() else {
        panic!("ordinary Object() identity");
    };
    let state = format!("{:?}", runtime.state);
    let keys = runtime.graph.keys().cloned().collect::<Vec<_>>();
    let error = runtime
        .run(
            r#"
def abort(item):
    throw failure
[1].flatMap(abort, log.calls = 99)
"#,
        )
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Thrown);
    assert_eq!(
        error.thrown_value(),
        Some(&Value::Object(ObjectId::named("failure")))
    );
    assert_eq!(
        runtime.state.property(&log, "calls"),
        Some(&Value::number(0))
    );
    assert_eq!(format!("{:?}", runtime.state), state);
    assert_eq!(runtime.graph.keys().cloned().collect::<Vec<_>>(), keys);
    assert!(runtime.state.function("abort").is_none());
    checked(
        &mut runtime,
        "result = [1].flatMap(item => [item], log.calls = 1)\nassert(result, [1])\nassert(log.calls, 1)",
        None,
    );
}

#[test]
fn custom_global_local_object_and_instance_flat_map_names_keep_ordinary_dispatch() {
    let mut runtime = run(r#"
def flatMap(item) => item + 1
assert(flatMap(2), 3)
namespace = Object()
namespace.flatMap = flatMap
assert(namespace.flatMap(2), 3)
class Handler:
    def init(bias):
        this.bias = bias
    def flatMap(item) => self.bias + item
handler = Handler(10)
assert(handler.flatMap(2), 12)
def apply(flatMap) => flatMap(2)
assert(apply(item => item * 3), 6)
namespace.flatMap = 9
assert(namespace.flatMap, 9)
flatMap = 7
assert(flatMap, 7)
assert([2].flatMap(item => [item]), [2])
nativeProperty = [1, 2].flatMap
assert(nativeProperty, null)
"#);
    let Value::Object(namespace) = runtime.state.variable("namespace").unwrap() else {
        panic!("ordinary Object() identity");
    };
    let message = format!("{namespace}.flatMap is not a function");
    let error = runtime.run("namespace.flatMap(2)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), message);
    let error = runtime.run("flatMap(2)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "flatMap is not a function");
}

#[test]
fn existing_map_reduce_concat_and_other_queries_remain_available_without_aliases() {
    run(r#"
values = [1, 2, 3]
assert(values.map(item => [item]), [[1], [2], [3]])
assert(values.reduce((sum, item) => sum + item, 0), 6)
assert(values.concat([4], [5, 6], 7), [1, 2, 3, 4, 5, 6, 7])
assert(values.filter(item => item > 1), [2, 3])
assert(values.find(item => item > 1), 2)
assert(values.some(item => item == 2), true)
assert(values.every(item => item > 0), true)
assert(values, [1, 2, 3])
"#);
    for (method, message) in [
        ("map", "map expects a function"),
        ("reduce", "reduce expects a function"),
        ("flatmap", "values.flatmap is not a function"),
        ("flatten", "values.flatten is not a function"),
    ] {
        let mut runtime = run("values = [1, 2]");
        let error = runtime.run(&format!("values.{method}(0)")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), message);
    }
}

#[test]
fn native_cli_files_use_literal_bytes_at_eof_and_exact_success_and_error_output() {
    for (index, (source, success, stdout, stderr)) in [
        ("[1, 2].flatMap(item => [item, item * 10])", true, "[1.0,10.0,2.0,20.0]\n", ""),
        ("[[[1]], [], [2, 2], \"ab\"].flatMap(item => item)", true, "[[1.0],2.0,2.0,\"ab\"]\n", ""),
        ("expanded = [1, 2].flatMap(item => null)\nexpanded", true, "[null,null]\n", ""),
        ("expanded = [1, 2].flatMap(item => [][0])\nexpanded", true, "[null,null]\n", ""),
        ("[].flatMap(item => missing)", true, "[]\n", ""),
        ("class Batch(items):\n    this.items = items\nfirst = Batch([1, 2])\nsecond = Batch([3])\nBatch.flatMap(batch => batch.items)", true, "[1.0,2.0,3.0]\n", ""),
        ("[].flatMap()", false, "", "TypeError: flatMap expects a function (line 1, column 1)\n"),
        ("[1].flatMap(0)", false, "", "TypeError: flatMap expects a function (line 1, column 1)\n"),
        ("[].flatMap(0, missing)", false, "", "ReferenceError: missing is not defined (line 1, column 1)\n"),
        ("def abort(item):\n    throw \"CALLBACK_ABORT\"\n[1].flatMap(abort)", false, "", "CALLBACK_ABORT (line 3, column 1)\n"),
        ("[1].flatMap(", false, "", "SyntaxError: Unexpected end of input (line 1, column 13)\n"),
        ("[1].flatMap(\n", false, "", "SyntaxError: Unexpected end of input (line 2, column 1)\n"),
    ].into_iter().enumerate() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join(format!(
            "nuc41-flatmap-{}-{index}.nuc", std::process::id()
        ));
        fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap()
            .write_all(source.as_bytes()).unwrap();
        assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg("--json").arg(&path).stdin(Stdio::null()).output();
        let cleanup = fs::remove_file(&path);
        let output = output.unwrap();
        cleanup.unwrap();
        assert_eq!(output.status.success(), success, "{source}: {output:?}");
        assert_eq!(output.stdout, stdout.as_bytes(), "{source}");
        assert_eq!(output.stderr, stderr.as_bytes(), "{source}");
    }
}

fn stdin_cli(source: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn native_stdin_recovers_from_callback_validation_and_keeps_existing_eof_framing() {
    let banner = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";
    let source = "values = [1, 2]\nexpanded = values.flatMap(item => [item, item * 10])\n[].flatMap(0)\nvalues.push(3)\nexpanded";
    assert!(!source.ends_with('\n'));
    let output = stdin_cli(source);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stderr,
        b"TypeError: flatMap expects a function (line 1, column 1)\n"
    );
    assert_eq!(
        output.stdout,
        format!("{banner}[1.0,2.0]\n[1.0,10.0,2.0,20.0]\n3.0\n[1.0,10.0,2.0,20.0,3.0,30.0]\n")
            .as_bytes()
    );
    // The inherited REPL appends a newline; files above separately protect
    // true unterminated EOF. This feature does not change REPL framing.
    for source in ["[1].flatMap(", "[1].flatMap(\n"] {
        let output = stdin_cli(source);
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, banner.as_bytes());
        assert_eq!(
            output.stderr,
            b"SyntaxError: Unexpected end of input (line 2, column 1)\n"
        );
    }
}
