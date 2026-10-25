//! Development compatibility replay, not a token gate, report or verdict.
//! Only this ignored audit needs immutable session artifacts and pinned Git.

mod common;
#[path = "support/radian_cases.rs"]
mod radian_cases;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use nucleoid::lang::ast::parse_program;
use nucleoid::{Runtime, Value};
use serde::Deserialize;

const ACCEPTED: &str = "cd1f7fdb288e6d2e1c9febbabcf280976cce4169";
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

// Deliberately do not deserialize or compare token counts. The independent
// evaluator, not a development replay, owns the per-case token decision.
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
    eligible_categories: Vec<String>,
    selected_index: usize,
    category: String,
    candidate_worktree: String,
    new_measured_cases: usize,
}

fn artifact<T: serde::de::DeserializeOwned>(variable: &str, filename: &str) -> T {
    let path = std::env::var_os(variable).unwrap_or_else(|| panic!("{variable} is required"));
    let path = Path::new(&path);
    // Only the three named JSON inputs may be opened, never an environment file.
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
        // Read exactly the four accepted corpus blobs; never main-worktree
        // sources, other repository paths or environment files.
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
    assert_eq!(result.len(), 633);
    result
}

#[test]
#[ignore = "development replay needs NUC50_BASELINE, NUC50_SELECTION, NUC50_EXTRA_BASELINES and pinned Git blobs"]
fn all_633_unchanged_sources_and_nine_accepted_counterparts_preserve_candidate_behavior() {
    let baseline: Baseline = artifact("NUC50_BASELINE", "nuc-0050-baseline.json");
    assert_eq!(baseline.metric, "nucleoid-lexer-with-layout-v1");
    assert_eq!(baseline.revision, ACCEPTED);
    assert_eq!(baseline.cases.len(), 642);
    let selection: Selection = artifact("NUC50_SELECTION", "nuc-0050-selection.json");
    assert_eq!(selection.proposal.replace('\\', "/"), "docs/nuc-0050.md");
    assert_eq!(selection.accepted_revision, ACCEPTED);
    assert_eq!(selection.category, "Utility enhancement");
    assert_eq!(selection.selected_index, 2);
    assert_eq!(
        selection.eligible_categories[selection.selected_index],
        selection.category
    );
    assert_eq!(selection.new_measured_cases, 9);
    assert_eq!(
        Path::new(&selection.candidate_worktree)
            .canonicalize()
            .unwrap(),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .canonicalize()
            .unwrap()
    );
    let extras: Vec<Extra> = artifact("NUC50_EXTRA_BASELINES", "nuc-0050-extra-baselines.json");
    assert_eq!(extras.len(), radian_cases::SEALED.len());
    let mut accepted = accepted_sources();
    let historical = accepted
        .iter()
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    for (extra, sealed) in extras.iter().zip(radian_cases::SEALED) {
        assert_eq!(extra.document, sealed.document);
        assert_eq!(extra.title, sealed.title);
        assert_eq!(
            extra.source, sealed.accepted,
            "immutable accepted counterpart"
        );
        assert_eq!(extra.expected.as_deref(), sealed.expected);
        accepted.push((
            identity(&extra.document, &extra.title),
            common::Case {
                title: extra.title.clone(),
                source: extra.source.clone(),
                expected: extra.expected.clone(),
            },
        ));
    }
    let mut candidate = BTreeMap::new();
    for (document, cases) in radian_cases::documents() {
        let original_order = accepted
            .iter()
            .filter(|(id, _)| id.document == document)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        let current_order = cases
            .iter()
            .map(|case| identity(document, &case.title))
            .collect::<Vec<_>>();
        assert_eq!(current_order, original_order, "{document}: identity/order");
        for case in cases {
            assert!(
                candidate
                    .insert(identity(document, &case.title), case)
                    .is_none(),
                "duplicate candidate identity"
            );
        }
    }
    assert_eq!(candidate.len(), baseline.cases.len());
    let mut seen = BTreeSet::new();
    let mut unchanged = 0;
    let mut rewritten = 0;
    let mut edits = 0;
    let mut assertions = 0;
    let mut new_assertions = 0;
    let mut returns = 0;
    for (protected, (id, original)) in baseline.cases.iter().zip(&accepted) {
        assert_eq!(&protected.id, id, "baseline identity/order");
        assert!(seen.insert(id.clone()), "duplicate baseline identity");
        let current = candidate.get(id).unwrap();
        assert_eq!(original.expected, protected.expected, "{id:?}");
        assert_eq!(current.expected, protected.expected, "{id:?}");
        let old = parse_program(&original.source).unwrap_or_else(|error| panic!("{id:?}: {error}"));
        let new = parse_program(&current.source).unwrap_or_else(|error| panic!("{id:?}: {error}"));
        let sealed = radian_cases::SEALED
            .iter()
            .find(|sealed| sealed.document == id.document && sealed.title == id.title);
        if historical.contains(id) {
            assert!(sealed.is_none());
            assert_eq!(current.source, original.source, "{id:?}: byte-exact legacy");
            assert_eq!(
                new.statements, old.statements,
                "{id:?}: complete legacy AST"
            );
            assert_eq!(new.positions, old.positions, "{id:?}: legacy positions");
            unchanged += 1;
        } else {
            let sealed = sealed.unwrap();
            let (expected, changes) =
                radian_cases::rewrite(&radian_cases::documented_source(sealed));
            assert_eq!(current.source.trim(), expected, "{id:?}: RHS-only rewrite");
            assert_eq!(changes, sealed.rewrites);
            assert_ne!(new.statements, old.statements, "new RHS ASTs differ");
            edits += changes;
            rewritten += 1;
        }
        for statements in [&old.statements, &new.statements] {
            assert_eq!(
                common::assertions::protected_assertions(statements),
                protected.assertions,
                "{id:?}: canonical assertion expressions/order"
            );
        }
        let mut runtimes = [Runtime::new(), Runtime::new()];
        let mut results = Vec::new();
        for (runtime, case) in runtimes.iter_mut().zip([original, current]) {
            results.push(
                common::check_with_count(runtime, case, protected.assertions.len())
                    .unwrap_or_else(|reason| panic!("{id:?}: {reason}")),
            );
        }
        assert_eq!(
            runtimes[0].assertions_run(),
            runtimes[1].assertions_run(),
            "{id:?}: actual executed coverage"
        );
        if let Some(sealed) = sealed {
            assert_eq!(runtimes[0].assertions_run(), sealed.assertions);
            radian_cases::assert_observations(&runtimes[1], &runtimes[0], sealed);
            if matches!(
                (&results[0], &results[1]),
                (Value::Number(_), Value::Number(_))
            ) {
                radian_cases::assert_numeric_same(&results[1], &results[0]);
            } else {
                assert_eq!(results[1], results[0]);
            }
            new_assertions += runtimes[1].assertions_run();
        }
        // Historical complete ASTs and each execution's protected observations
        // are equal. Do not require independently sampled Date.now() values or
        // fresh object identities to be byte-equal across separate executions.
        // Focused rollback regressions separately compare deterministic full
        // runtime snapshots through support/runtime_equivalence.rs.
        assertions += runtimes[1].assertions_run();
        returns += usize::from(protected.expected.is_some());
    }
    assert_eq!(unchanged, 633);
    assert_eq!(rewritten, 9);
    assert_eq!(edits, 21);
    assert_eq!(new_assertions, 55);
    assert_eq!(returns, 51);
    println!(
        "Development compatibility: 633 byte-exact pinned programs plus nine immutable accepted \
         arithmetic counterparts and all 642 current forms executed. Complete historical ASTs and \
         positions agree. Nine exact RHS-only rewrites ({edits} substitutions) \
         preserve 55 new protected assertions, numeric bits/NaN classification and class reachability. \
         All {assertions} actual assertions and {returns} explicit expected returns match; \
         no new-call AST/rule-source equality, token comparison, gate, report or verdict is claimed."
    );
}
