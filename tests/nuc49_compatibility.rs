//! Development replay only: no token comparison, gate, report or verdict.
//! Ordinary suites remain independent of session artifacts and Git history.

mod common;
#[allow(dead_code)]
#[path = "support/conditional_headers.rs"]
mod conditional_headers;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use nucleoid::Runtime;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::parse_program;
use serde::Deserialize;

const ACCEPTED: &str = "30f8db484567aa1a1fc6cc681b3ac00d55632a3a";
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

// Intentionally do not deserialize/count/compare the baseline's token metric.
// Only the independent evaluator owns the per-case token acceptance decision.
#[derive(Deserialize)]
struct ProtectedCase {
    id: Identity,
    assertions: Vec<String>,
    expected: Option<String>,
}

#[derive(Deserialize)]
struct Selection {
    accepted_revision: String,
    category: String,
    new_measured_cases: bool,
    measured_rewrite_scope: Vec<Identity>,
}

fn artifact<T: serde::de::DeserializeOwned>(variable: &str) -> T {
    let path = std::env::var_os(variable).unwrap_or_else(|| panic!("{variable} is required"));
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn sources(accepted: bool) -> Vec<(Identity, common::Case)> {
    let mut result = Vec::new();
    for &(document, path) in DOCUMENTS {
        let source = if accepted {
            // Read exactly four pinned corpus blobs, never another worktree
            // or an environment file. No main-worktree source is consulted.
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
        let cases = if matches!(document, "spec" | "reference") {
            common::cases(&source)
        } else {
            common::snippets(&source, path)
        };
        for case in cases {
            result.push((
                Identity {
                    document: document.to_string(),
                    title: case.title.clone(),
                },
                case,
            ));
        }
    }
    assert_eq!(result.len(), 633);
    result
}

#[test]
#[ignore = "development replay requires immutable NUC49_BASELINE, NUC49_SELECTION and pinned Git blobs"]
fn all_633_pinned_sources_and_candidate_forms_preserve_complete_syntax_and_executed_behavior() {
    let baseline: Baseline = artifact("NUC49_BASELINE");
    assert_eq!(baseline.metric, "nucleoid-lexer-with-layout-v1");
    assert_eq!(baseline.revision, ACCEPTED);
    assert_eq!(baseline.cases.len(), 633);
    let selection: Selection = artifact("NUC49_SELECTION");
    assert_eq!(selection.accepted_revision, ACCEPTED);
    assert_eq!(selection.category, "Syntax enhancement");
    assert!(!selection.new_measured_cases);
    assert_eq!(
        selection.measured_rewrite_scope,
        conditional_headers::REWRITES
            .iter()
            .map(|(document, title)| Identity {
                document: (*document).to_string(),
                title: (*title).to_string(),
            })
            .collect::<Vec<_>>()
    );

    let accepted = sources(true);
    let candidate = sources(false);
    let mut identities = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut unchanged = 0;
    let mut rewritten = 0;
    let mut edits = 0;
    let mut assertions = 0;
    let mut rewritten_assertions = 0;
    let mut returns = 0;
    for ((protected, (id, original)), (candidate_id, current)) in
        baseline.cases.iter().zip(&accepted).zip(&candidate)
    {
        assert_eq!(&protected.id, id, "accepted identity and order");
        assert_eq!(candidate_id, id, "candidate identity and order");
        assert!(identities.insert(id.clone()), "duplicate {id:?}");
        assert_eq!(original.expected, protected.expected, "{id:?}");
        assert_eq!(current.expected, protected.expected, "{id:?}");
        if let Some(index) = selection
            .measured_rewrite_scope
            .iter()
            .position(|selected| selected == id)
        {
            assert!(seen.insert(id.clone()));
            let (expected_source, count) = conditional_headers::omit(&original.source);
            assert_eq!(
                current.source, expected_source,
                "{id:?}: only qualifying colon bytes may change"
            );
            assert_eq!(count, [3, 3, 5, 3, 4, 3, 2, 6][index], "{id:?}");
            edits += count;
            rewritten += 1;
        } else {
            assert_eq!(
                original.source, current.source,
                "{id:?}: historical legacy control"
            );
            unchanged += 1;
        }

        let old = parse_program(&original.source).unwrap_or_else(|error| panic!("{id:?}: {error}"));
        let new = parse_program(&current.source).unwrap_or_else(|error| panic!("{id:?}: {error}"));
        assert_eq!(new.statements, old.statements, "{id:?}: complete AST");
        assert_eq!(new.positions, old.positions, "{id:?}: statement positions");
        assert_eq!(
            generate_all(&new.statements),
            generate_all(&old.statements),
            "{id:?}: canonical"
        );
        for statements in [&old.statements, &new.statements] {
            assert_eq!(
                common::assertions::protected_assertions(statements),
                protected.assertions,
                "{id:?}: protected assertion expressions and order"
            );
        }
        let mut runtimes = [Runtime::new(), Runtime::new()];
        for (runtime, case) in runtimes.iter_mut().zip([original, current]) {
            common::check_with_count(runtime, case, protected.assertions.len())
                .unwrap_or_else(|error| panic!("{id:?}: {error}"));
        }
        assert_eq!(
            runtimes[0].assertions_run(),
            runtimes[1].assertions_run(),
            "{id:?}: executed coverage"
        );
        assertions += runtimes[1].assertions_run();
        if seen.contains(id) {
            rewritten_assertions += runtimes[1].assertions_run();
        }
        returns += usize::from(protected.expected.is_some());
    }
    assert_eq!(unchanged, 625);
    assert_eq!(rewritten, 8);
    assert_eq!(seen.len(), selection.measured_rewrite_scope.len());
    assert_eq!(edits, 29);
    assert_eq!(rewritten_assertions, 50);
    assert_eq!(returns, 50);
    println!(
        "Development compatibility: all 633 pinned Git programs and all 633 candidate forms executed; \
         {unchanged} byte-exact legacy controls; {rewritten} exact condition-header-colon-only rewrites, \
         {edits} omissions with final Else colons retained; \
         {rewritten_assertions} rewritten assertions; {assertions} executed assertions per corpus, \
         {returns} explicit returns. Complete ASTs, canonical source, statement positions, \
         protected assertions/order, expected returns and coverage match. No token decision."
    );
}
