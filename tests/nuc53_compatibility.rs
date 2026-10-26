//! Ignored full development replay of pinned accepted sources and candidate
//! forms. No token counts, evaluation gate, report, or acceptance decision.

mod common;
#[path = "support/state_version_cases.rs"]
mod version_cases;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use nucleoid::Runtime;
use nucleoid::lang::ast::parse_program;
use serde::Deserialize;

const ACCEPTED: &str = "8f6b18ba6195401f31352432e4ae8f6615e35094";
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

// Token fields are intentionally not deserialized. This is only a behavior
// replay; the independent evaluator owns all before/after token comparisons.
#[derive(Deserialize)]
struct ProtectedCase {
    id: Identity,
    assertions: Vec<String>,
    expected: Option<String>,
}

#[derive(Deserialize)]
struct Extra {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

#[derive(Deserialize)]
struct Selection {
    proposal: String,
    accepted_revision: String,
    category: String,
    selection_method: String,
    requested_change: String,
    rationale: String,
    candidate_worktree: String,
    new_measured_cases: usize,
    synthesized_record_file: String,
}

fn artifact<T: serde::de::DeserializeOwned>(variable: &str, filename: &str) -> T {
    let path = std::env::var_os(variable).unwrap_or_else(|| panic!("{variable} is required"));
    let path = Path::new(&path);
    // Exactly named JSON inputs only: never an environment file, recursive
    // directory read, unrelated worktree, or fallback artifact.
    assert_eq!(path.file_name().unwrap(), filename);
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn identity(document: &str, title: &str) -> Identity {
    Identity {
        document: document.to_string(),
        title: title.to_string(),
    }
}

fn accepted_sources() -> Vec<(Identity, common::Case)> {
    let mut result = Vec::new();
    for &(document, path) in DOCUMENTS {
        // These are Git object names, not filesystem reads of the main tree.
        let output = Command::new("git")
            .args(["--no-pager", "show", &format!("{ACCEPTED}:{path}")])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let source = String::from_utf8(output.stdout).unwrap();
        let cases = if matches!(document, "spec" | "reference") {
            common::cases(&source)
        } else {
            common::snippets(&source, path)
        };
        result.extend(
            cases
                .into_iter()
                .map(|case| (identity(document, &case.title), case)),
        );
    }
    assert_eq!(result.len(), 651);
    result
}

#[test]
#[ignore = "development replay needs NUC53_BASELINE, NUC53_SELECTION, NUC53_EXTRA_BASELINES and pinned accepted Git blobs"]
fn all_651_byte_exact_programs_and_nine_sealed_history_pairs_preserve_2996_assertions_53_returns() {
    let baseline: Baseline = artifact("NUC53_BASELINE", "nuc-0053-baseline.json");
    assert_eq!(baseline.metric, "nucleoid-lexer-with-layout-v1");
    assert_eq!(baseline.revision, ACCEPTED);
    assert_eq!(baseline.cases.len(), 660);
    let selection: Selection = artifact("NUC53_SELECTION", "nuc-0053-selection.json");
    assert_eq!(selection.proposal, r"docs\nuc-0053.md");
    assert_eq!(selection.accepted_revision, ACCEPTED);
    assert_eq!(selection.category, "Runtime features");
    assert_eq!(selection.selection_method, "explicit_user_feature");
    assert!(selection.requested_change.contains("@0"));
    assert!(!selection.rationale.is_empty());
    assert_eq!(selection.new_measured_cases, 9);
    assert_eq!(
        selection.synthesized_record_file,
        r"dataset\nucleoid.synth.33.jsonl"
    );
    assert_eq!(
        Path::new(&selection.candidate_worktree)
            .canonicalize()
            .unwrap(),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .canonicalize()
            .unwrap()
    );
    let extras: Vec<Extra> = artifact("NUC53_EXTRA_BASELINES", "nuc-0053-extra-baselines.json");
    assert_eq!(extras.len(), version_cases::SEALED.len());
    let mut accepted = accepted_sources();
    let historical = accepted
        .iter()
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    for (extra, sealed) in extras.into_iter().zip(version_cases::SEALED) {
        assert_eq!(extra.document, sealed.document);
        assert_eq!(extra.title, sealed.title);
        assert_eq!(
            extra.source, sealed.accepted,
            "complete pre-executed counterpart"
        );
        assert_eq!(extra.expected.as_deref(), sealed.expected);
        accepted.push((
            identity(&extra.document, &extra.title),
            common::Case {
                title: extra.title,
                source: extra.source,
                expected: extra.expected,
            },
        ));
    }
    let mut candidate = BTreeMap::new();
    for (document, cases) in version_cases::documents() {
        let original_order = accepted
            .iter()
            .filter(|(id, _)| id.document == document)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        let current_order = cases
            .iter()
            .map(|case| identity(document, &case.title))
            .collect::<Vec<_>>();
        assert_eq!(
            current_order, original_order,
            "{document}: identities/order"
        );
        for case in cases {
            assert!(
                candidate
                    .insert(identity(document, &case.title), case)
                    .is_none()
            );
        }
    }
    assert_eq!(candidate.len(), baseline.cases.len());
    let mut seen = BTreeSet::new();
    let mut totals = (0, 0, 0, 0);
    let mut actual = [0, 0];
    let mut new_actual = [0, 0];
    for (protected, (id, original)) in baseline.cases.iter().zip(&accepted) {
        assert_eq!(&protected.id, id, "baseline identities/order");
        assert!(seen.insert(id.clone()));
        let current = candidate.get(id).unwrap();
        assert_eq!(original.expected, protected.expected, "{id:?}");
        assert_eq!(current.expected, protected.expected, "{id:?}");
        let old = parse_program(&original.source).unwrap_or_else(|error| panic!("{id:?}: {error}"));
        let new = parse_program(&current.source).unwrap_or_else(|error| panic!("{id:?}: {error}"));
        let sealed = version_cases::SEALED
            .iter()
            .find(|sealed| sealed.document == id.document && sealed.title == id.title);
        if historical.contains(id) {
            assert!(sealed.is_none());
            assert_eq!(current.source, original.source, "{id:?}: historical bytes");
            assert_eq!(
                new.statements, old.statements,
                "{id:?}: complete historical AST"
            );
            assert_eq!(new.positions, old.positions, "{id:?}: historical positions");
            totals.0 += 1;
        } else {
            let sealed = sealed.unwrap();
            let documented = if matches!(sealed.document, "spec" | "reference") {
                format!("# {}\n\n{}", sealed.title, sealed.candidate)
            } else {
                sealed.candidate.to_string()
            };
            assert_eq!(
                current.source.trim(),
                documented,
                "{id:?}: only supplied lowering"
            );
            assert_ne!(
                new.statements, old.statements,
                "history mirrors intentionally differ"
            );
            totals.1 += 1;
        }
        for statements in [&old.statements, &new.statements] {
            assert_eq!(
                common::assertions::protected_assertions(statements),
                protected.assertions,
                "{id:?}: canonical assertions/order"
            );
        }
        let mut runtimes = [Runtime::new(), Runtime::new()];
        let mut returned = Vec::new();
        for (index, (runtime, case)) in runtimes.iter_mut().zip([original, current]).enumerate() {
            returned.push(
                common::check_with_count(runtime, case, protected.assertions.len())
                    .unwrap_or_else(|reason| panic!("{id:?}: {reason}")),
            );
            actual[index] += runtime.assertions_run();
        }
        assert_eq!(
            runtimes[0].assertions_run(),
            runtimes[1].assertions_run(),
            "{id:?}: actual coverage"
        );
        if let Some(sealed) = sealed {
            for (index, runtime) in runtimes.iter().enumerate() {
                assert_eq!(runtime.assertions_run(), sealed.assertions, "{id:?}");
                new_actual[index] += runtime.assertions_run();
            }
            version_cases::assert_observations(&runtimes[1], &runtimes[0], sealed);
            if sealed.expected.is_some() {
                assert_eq!(returned[0], returned[1], "{id:?}: explicit return");
            }
        }
        // Historical Date samples and fresh anonymous IDs are not cross-run
        // identity guarantees. Each corpus independently checks all assertions
        // and explicit JSON returns through the common harness.
        totals.2 += protected.assertions.len();
        totals.3 += usize::from(protected.expected.is_some());
    }
    assert_eq!(totals, (651, 9, 2996, 53));
    assert_eq!(
        actual,
        [2996, 2996],
        "actual executed assertions per corpus"
    );
    assert_eq!(new_actual, [69, 69]);
    println!(
        "Development compatibility: all 660 accepted/candidate pairs executed; \
        651 byte-exact historical sources with equal full ASTs and positions; nine sealed \
        history-emulation controls and candidate forms preserve canonical assertion order, \
        69 new assertions and meaningful exposed values. Exactly 2996 actual assertions and \
        53 expected returns checked in EACH corpus. No token comparison, evaluation, report or verdict."
    );
}
