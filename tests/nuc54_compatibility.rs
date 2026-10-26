//! Ignored full development replay: pinned accepted Git documents, sealed
//! controls and candidate forms. No token measurement, gate, report or verdict.

mod common;
#[path = "support/literal_default_cases.rs"]
mod default_cases;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use nucleoid::Runtime;
use nucleoid::lang::ast::parse_program;
use serde::Deserialize;

const ACCEPTED: &str = "a9144c162ce23e6cc8407b5640c9a6327dfe6f25";
const REGISTRY_BLOB: &str = "7d06c2d45e5aac3f21c10d624b837490b727044f";
const BASELINE_SHA256: &str = "B31087B1F3C892F958F4C4773086777CD2ECF8ECBB8A793AF5095D813EF61538";
const EXTRAS_SHA256: &str = "91FEAD3E8C568432BC8B653CA12C8F595526A828757EF38C855A00CE242BBBA1";
const SELECTION_SHA256: &str = "3C07C01FDD3381DF209791EF7D8591A1E54B4E6531033E1F117AA1BF8F14EC5E";
const DRAW_SHA256: &str = "1E83DB657FE93D2C6773F482705B49AE69007147E93F50CC862504DEC81397C5";
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

// Intentionally do not deserialize the token fields: those belong solely to
// the independent evaluator, not this behavior/compatibility replay.
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
    proposal: String,
    accepted_revision: String,
    registry: String,
    registry_git_blob: String,
    eligible_categories: Vec<String>,
    selection_method: String,
    selected_index: usize,
    selected_category: String,
    probability_numerator: usize,
    probability_denominator: usize,
}

#[derive(Deserialize)]
struct Selection {
    proposal: String,
    title: String,
    accepted_revision: String,
    category: String,
    selection_method: String,
    requested_change: String,
    rationale: String,
    candidate_worktree: String,
    baseline_path: String,
    baseline_sha256: String,
    extra_baselines_path: String,
    extra_baselines_sha256: String,
    draw_path: String,
    draw_sha256: String,
    historical_measured_cases: usize,
    new_measured_cases: usize,
    total_canonical_assertions: usize,
    new_canonical_assertions: usize,
    explicit_expected_returns: usize,
    new_expected_returns: usize,
    synthesized_record_file: String,
    signature_scope: String,
    bare_parameter_forms: String,
    omission_policy: String,
}

fn artifact<T: serde::de::DeserializeOwned>(variable: &str, filename: &str, sha256: &str) -> T {
    let path = std::env::var_os(variable).unwrap_or_else(|| panic!("{variable} is required"));
    let path = Path::new(&path);
    // Only the four exactly named JSON inputs are ever opened. No environment
    // files, directory traversal, main-tree sources or fallback artifacts.
    assert_eq!(path.file_name().unwrap(), filename);
    // This explicitly Windows development replay uses the platform hasher,
    // without adding a runtime or development dependency for scalar defaults.
    let script = format!(
        "$ErrorActionPreference = 'Stop'; \
         $hash = [System.Security.Cryptography.SHA256]::Create(); \
         try {{ [System.BitConverter]::ToString($hash.ComputeHash(\
         [System.IO.File]::ReadAllBytes('{}'))).Replace('-', '') }} \
         finally {{ $hash.Dispose() }}",
        path.to_str().unwrap().replace('\'', "''")
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        sha256,
        "{filename}"
    );
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn same_artifact_path(variable: &str, selected: &str) {
    let supplied = std::env::var_os(variable).unwrap();
    assert_eq!(
        Path::new(&supplied).canonicalize().unwrap(),
        Path::new(selected).canonicalize().unwrap()
    );
}

fn git(arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("--no-pager")
        .args(arguments)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
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
        // These are exactly four pinned Git object names, not filesystem
        // paths into the main tree or any other proposal's worktree.
        let source = git(&["show", &format!("{ACCEPTED}:{path}")]);
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
    assert_eq!(result.len(), 660);
    result
}

#[test]
#[ignore = "development replay needs NUC54_BASELINE, NUC54_SELECTION, NUC54_DRAW, NUC54_EXTRA_BASELINES and pinned accepted Git blobs"]
fn all_660_byte_exact_programs_and_nine_literal_lowerings_preserve_3057_assertions_54_returns() {
    assert_eq!(git(&["rev-parse", "HEAD"]).trim(), ACCEPTED);
    assert_eq!(
        git(&["rev-parse", &format!("{ACCEPTED}:docs/nuc-0001.md")]).trim(),
        REGISTRY_BLOB
    );
    let baseline: Baseline = artifact("NUC54_BASELINE", "nuc-0054-baseline.json", BASELINE_SHA256);
    assert_eq!(baseline.metric, "nucleoid-lexer-with-layout-v1");
    assert_eq!(baseline.revision, ACCEPTED);
    assert_eq!(baseline.cases.len(), 669);
    let draw: Draw = artifact("NUC54_DRAW", "nuc-0054-category-draw.json", DRAW_SHA256);
    assert_eq!(draw.proposal, r"docs\nuc-0054.md");
    assert_eq!(draw.accepted_revision, ACCEPTED);
    assert_eq!(draw.registry, r"docs\nuc-0001.md");
    assert_eq!(draw.registry_git_blob, REGISTRY_BLOB);
    assert_eq!(draw.selection_method, "uniform_single_category_draw");
    assert_eq!(draw.selected_index, 1);
    assert_eq!(draw.selected_category, "Runtime features");
    assert_eq!(
        (draw.probability_numerator, draw.probability_denominator),
        (1, 4)
    );
    assert_eq!(
        draw.eligible_categories,
        [
            "Syntax enhancement",
            "Runtime features",
            "Utility enhancement",
            "Reasoning enhancement",
        ]
    );
    let selection: Selection = artifact(
        "NUC54_SELECTION",
        "nuc-0054-selection.json",
        SELECTION_SHA256,
    );
    assert_eq!(selection.proposal, r"docs\nuc-0054.md");
    assert_eq!(selection.title, "Literal Parameter Defaults");
    assert_eq!(selection.accepted_revision, ACCEPTED);
    assert_eq!(selection.category, draw.selected_category);
    assert_eq!(selection.selection_method, draw.selection_method);
    assert_eq!(
        draw.eligible_categories[draw.selected_index],
        selection.category
    );
    assert_eq!(selection.baseline_sha256, BASELINE_SHA256);
    assert_eq!(selection.extra_baselines_sha256, EXTRAS_SHA256);
    assert_eq!(selection.draw_sha256, DRAW_SHA256);
    same_artifact_path("NUC54_BASELINE", &selection.baseline_path);
    same_artifact_path("NUC54_EXTRA_BASELINES", &selection.extra_baselines_path);
    same_artifact_path("NUC54_DRAW", &selection.draw_path);
    assert!(selection.requested_change.contains("parenthesized"));
    assert!(selection.rationale.contains("argument-binding"));
    assert_eq!(selection.historical_measured_cases, 660);
    assert_eq!(selection.new_measured_cases, 9);
    assert_eq!(selection.total_canonical_assertions, 3057);
    assert_eq!(selection.new_canonical_assertions, 61);
    assert_eq!(selection.explicit_expected_returns, 54);
    assert_eq!(selection.new_expected_returns, 1);
    assert_eq!(
        selection.synthesized_record_file,
        r"dataset\nucleoid.synth.34.jsonl"
    );
    assert_eq!(
        selection.signature_scope,
        "parenthesized_user_formal_parameter_lists"
    );
    assert_eq!(selection.bare_parameter_forms, "unchanged_no_new_defaults");
    assert_eq!(selection.omission_policy, "absent_argument_position_only");
    assert_eq!(
        Path::new(&selection.candidate_worktree)
            .canonicalize()
            .unwrap(),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .canonicalize()
            .unwrap()
    );
    let extras: Vec<Extra> = artifact(
        "NUC54_EXTRA_BASELINES",
        "nuc-0054-extra-baselines.json",
        EXTRAS_SHA256,
    );
    assert_eq!(extras.len(), default_cases::SEALED.len());
    let mut accepted = accepted_sources();
    let historical = accepted
        .iter()
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    for (extra, sealed) in extras.into_iter().zip(default_cases::SEALED) {
        assert_eq!(extra.document, sealed.document);
        assert_eq!(extra.title, sealed.title);
        assert_eq!(
            extra.source, sealed.accepted,
            "complete pre-executed counterpart"
        );
        assert_eq!(extra.expected.as_deref(), sealed.expected);
        default_cases::assert_exact_lowering(sealed);
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
    for (document, cases) in default_cases::documents() {
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
    assert_eq!(candidate.len(), 669);
    let mut seen = BTreeSet::new();
    let mut paired = [0, 0];
    let mut actual_assertions = [0, 0];
    let mut actual_returns = [0, 0];
    let mut new_assertions = [0, 0];
    let mut new_returns = [0, 0];
    for (protected, (id, original)) in baseline.cases.iter().zip(&accepted) {
        assert_eq!(&protected.id, id, "baseline identities/order");
        assert!(seen.insert(id.clone()));
        let current = candidate.get(id).unwrap();
        assert_eq!(original.expected, protected.expected, "{id:?}");
        assert_eq!(current.expected, protected.expected, "{id:?}");
        let old = parse_program(&original.source).unwrap_or_else(|error| panic!("{id:?}: {error}"));
        let new = parse_program(&current.source).unwrap_or_else(|error| panic!("{id:?}: {error}"));
        let sealed = default_cases::SEALED
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
            paired[0] += 1;
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
                "{id:?}: only authorized lowering"
            );
            assert_ne!(
                new.statements, old.statements,
                "{id:?}: signatures/calls intentionally differ"
            );
            paired[1] += 1;
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
            assert_eq!(
                runtime.assertions_run(),
                protected.assertions.len(),
                "{id:?}: actual coverage"
            );
            actual_assertions[index] += runtime.assertions_run();
            actual_returns[index] += usize::from(case.expected.is_some());
        }
        if let Some(sealed) = sealed {
            for (index, runtime) in runtimes.iter().enumerate() {
                assert_eq!(runtime.assertions_run(), sealed.assertions, "{id:?}");
                new_assertions[index] += runtime.assertions_run();
                new_returns[index] += usize::from(sealed.expected.is_some());
            }
            default_cases::assert_observations(&runtimes[1], &runtimes[0], sealed);
            if sealed.expected.is_some() {
                assert_eq!(returned[0], returned[1], "{id:?}: explicit return");
            }
        }
        // Dates and fresh anonymous object IDs are not cross-run identities.
        // Each corpus independently checks the original assertions/returns.
    }
    assert_eq!(seen.len(), 669);
    assert_eq!(paired, [660, 9]);
    assert_eq!(actual_assertions, [3057, 3057]);
    assert_eq!(actual_returns, [54, 54]);
    assert_eq!(new_assertions, [61, 61]);
    assert_eq!(new_returns, [1, 1]);
    println!(
        "Development replay: all 669 accepted/candidate pairs executed; 660 byte-exact historical \
         programs with equal complete ASTs and positions; nine exact literal-only lowerings \
         preserve canonical assertions/order and meaningful exposed observations. Exactly 3057 \
         actual assertions and 54 explicit expected returns checked in EACH corpus, including \
         61 new assertions and one new return. No token comparison, evaluation, report or verdict."
    );
}
