//! Ignored development replay of pinned accepted programs and current forms.
//! No token measurement, acceptance gate, report or outcome is produced here.

mod common;
#[path = "support/string_iteration_cases.rs"]
mod string_cases;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use nucleoid::Runtime;
use nucleoid::lang::ast::parse_program;
use serde::Deserialize;

const ACCEPTED: &str = "2e0816ca677868d2716d2b53d317111d03c5619a";
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

// Deliberately ignore the artifact's token counts. Only the independent
// evaluator owns token comparison and the proposal decision.
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
struct Draw {
    accepted_revision: String,
    registry_blob: String,
    eligible_categories: Vec<String>,
    selected_index: usize,
    category: String,
}

#[derive(Deserialize)]
struct Selection {
    proposal: String,
    accepted_revision: String,
    draw_artifact: String,
    draw_sha256: String,
    selected_index: usize,
    category: String,
    requested_change: String,
    rationale: String,
    candidate_worktree: String,
    new_measured_cases: usize,
}

fn artifact<T: serde::de::DeserializeOwned>(variable: &str, filename: &str) -> T {
    let path = std::env::var_os(variable).unwrap_or_else(|| panic!("{variable} is required"));
    let path = Path::new(&path);
    // Only named immutable JSON inputs may be opened, never environment files.
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
        // Exactly four accepted corpus blobs; no main-worktree source reads.
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
    assert_eq!(result.len(), 642);
    result
}

#[test]
#[ignore = "development replay needs NUC51_BASELINE, NUC51_DRAW, NUC51_SELECTION, NUC51_EXTRA_BASELINES and pinned Git blobs"]
fn all_642_unchanged_sources_and_nine_split_map_counterparts_preserve_final_observations() {
    let baseline: Baseline = artifact("NUC51_BASELINE", "nuc-0051-baseline.json");
    assert_eq!(baseline.metric, "nucleoid-lexer-with-layout-v1");
    assert_eq!(baseline.revision, ACCEPTED);
    assert_eq!(baseline.cases.len(), 651);
    let draw: Draw = artifact("NUC51_DRAW", "nuc-0051-draw.json");
    assert_eq!(draw.accepted_revision, ACCEPTED);
    assert_eq!(
        draw.registry_blob,
        "7d06c2d45e5aac3f21c10d624b837490b727044f"
    );
    assert_eq!(draw.selected_index, 1);
    assert_eq!(draw.category, "Runtime features");
    assert_eq!(
        draw.eligible_categories,
        [
            "Syntax enhancement",
            "Runtime features",
            "Utility enhancement",
            "Reasoning enhancement",
        ]
    );
    let selection: Selection = artifact("NUC51_SELECTION", "nuc-0051-selection.json");
    assert_eq!(selection.proposal.replace('\\', "/"), "docs/nuc-0051.md");
    assert_eq!(selection.accepted_revision, ACCEPTED);
    assert_eq!(selection.draw_artifact, "nuc-0051-draw.json");
    assert_eq!(
        selection.draw_sha256,
        "1B3F4DCC5AD7E2A5B7999040FC1823D16748B10874C41D7CCBFE9EBE6149B791"
    );
    assert_eq!(selection.category, draw.category);
    assert_eq!(selection.selected_index, draw.selected_index);
    assert_eq!(
        draw.eligible_categories[selection.selected_index],
        selection.category
    );
    assert!(!selection.requested_change.is_empty());
    assert!(!selection.rationale.is_empty());
    assert_eq!(selection.new_measured_cases, 9);
    assert_eq!(
        Path::new(&selection.candidate_worktree)
            .canonicalize()
            .unwrap(),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .canonicalize()
            .unwrap()
    );
    let extras: Vec<Extra> = artifact("NUC51_EXTRA_BASELINES", "nuc-0051-extra-baselines.json");
    assert_eq!(extras.len(), string_cases::SEALED.len());
    let mut accepted = accepted_sources();
    let historical = accepted
        .iter()
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    for (extra, sealed) in extras.iter().zip(string_cases::SEALED) {
        assert_eq!(extra.document, sealed.document);
        assert_eq!(extra.title, sealed.title);
        assert_eq!(
            extra.source, sealed.accepted,
            "complete accepted counterpart"
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
    for (document, cases) in string_cases::documents() {
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
    let mut traversals = 0;
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
        let sealed = string_cases::SEALED
            .iter()
            .find(|sealed| sealed.document == id.document && sealed.title == id.title);
        if historical.contains(id) {
            assert!(sealed.is_none());
            assert_eq!(
                current.source, original.source,
                "{id:?}: byte-exact history"
            );
            assert_eq!(
                new.statements, old.statements,
                "{id:?}: full historical AST"
            );
            assert_eq!(new.positions, old.positions, "{id:?}: historical positions");
            unchanged += 1;
        } else {
            let sealed = sealed.unwrap();
            let (expected, changes) =
                string_cases::rewrite(&string_cases::documented_source(sealed));
            assert_eq!(
                current.source.trim(),
                expected,
                "{id:?}: loop lowering only"
            );
            assert_eq!(changes, sealed.rewrites);
            assert_ne!(new.statements, old.statements, "map/For ASTs differ");
            traversals += changes;
            rewritten += 1;
        }
        for statements in [&old.statements, &new.statements] {
            assert_eq!(
                common::assertions::protected_assertions(statements),
                protected.assertions,
                "{id:?}: canonical assertions/order"
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
            string_cases::assert_observations(&runtimes[1], &runtimes[0], sealed);
            if sealed.expected.is_some() {
                assert_eq!(results[1], results[0]);
            }
            new_assertions += runtimes[1].assertions_run();
        }
        // Do not compare discarded map results, callback ASTs or rule-source
        // strings to For. Historical nondeterministic Date samples and fresh
        // object IDs are likewise not cross-run identity guarantees.
        assertions += runtimes[1].assertions_run();
        returns += usize::from(protected.expected.is_some());
    }
    assert_eq!(unchanged, 642);
    assert_eq!(rewritten, 9);
    assert_eq!(traversals, 16);
    assert_eq!(new_assertions, 27);
    assert_eq!(returns, 52);
    println!(
        "Development compatibility: 642 byte-exact pinned programs plus nine immutable accepted \
         split/map counterparts and all 651 current forms executed. Historical full ASTs/positions \
         agree. Exactly 16 loop lowerings preserve 27 new protected assertions and one explicit \
         return; final arrays, strings and receiver properties agree. All {assertions} actual \
         assertions and {returns} expected returns checked per corpus. No map/For whole-AST, \
         whole-graph or discarded-result equality, token comparison, gate, report or verdict."
    );
}
