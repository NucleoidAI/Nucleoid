//! Behavior-only replay of NUC 47's immutable accepted counterparts.
//!
//! No token measurement, report writing, baseline recording or verdict lives
//! here. Run the ignored replay with NUC47_BASELINE and NUC47_EXTRA_BASELINES
//! pointing to the caller's pre-recorded JSON artifacts.

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use nucleoid::Runtime;
use nucleoid::lang::ast::parse;
use serde::Deserialize;

const ACCEPTED: &str = "e9b275615f52a36bb818e3a90cb96fe78b1241d7";
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

fn document_cases(document: &str, path: &str, source: &str) -> Vec<common::Case> {
    match document {
        "spec" | "reference" => common::cases(source),
        _ => common::snippets(source, path),
    }
}

fn insert(cases: &mut BTreeMap<Identity, common::Case>, document: &str, case: common::Case) {
    let id = Identity {
        document: document.to_string(),
        title: case.title.clone(),
    };
    assert!(cases.insert(id.clone(), case).is_none(), "duplicate {id:?}");
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
#[ignore = "requires the caller's immutable NUC47_BASELINE and NUC47_EXTRA_BASELINES artifacts"]
fn accepted_sources_and_all_paired_candidate_cases_preserve_behavior() {
    let baseline: Baseline = serde_json::from_str(
        &fs::read_to_string(
            std::env::var_os("NUC47_BASELINE").expect("NUC47_BASELINE is required"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(baseline.revision, ACCEPTED);
    assert_eq!(baseline.cases.len(), 620);

    let mut accepted = BTreeMap::new();
    let mut candidate = BTreeMap::new();
    for &(document, path) in DOCUMENTS {
        // Read only the four explicit executable documents at the pinned
        // revision, never the main worktree or an environment file.
        let output = Command::new("git")
            .args(["--no-pager", "show", &format!("{ACCEPTED}:{path}")])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let original = String::from_utf8(output.stdout).unwrap();
        for case in document_cases(document, path, &original) {
            insert(&mut accepted, document, case);
        }
        let current = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        for case in document_cases(document, path, &current) {
            insert(&mut candidate, document, case);
        }
    }
    assert_eq!(accepted.len(), 611);
    assert_eq!(candidate.len(), 620);
    for (id, original) in &accepted {
        let current = candidate
            .get(id)
            .expect("every historical case must remain");
        assert_eq!(original.source.trim(), current.source.trim(), "{id:?}");
        assert_eq!(original.expected, current.expected, "{id:?}");
    }

    let extras: Vec<ExtraSource> = serde_json::from_str(
        &fs::read_to_string(
            std::env::var_os("NUC47_EXTRA_BASELINES").expect("NUC47_EXTRA_BASELINES is required"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(extras.len(), 9);
    for extra in extras {
        insert(
            &mut accepted,
            &extra.document,
            common::Case {
                title: extra.title,
                source: extra.source,
                expected: extra.expected,
            },
        );
    }
    assert_eq!(accepted.len(), 620);
    let mut total_assertions = 0;
    let mut identities = std::collections::BTreeSet::new();
    for protected in &baseline.cases {
        assert!(identities.insert(protected.id.clone()));
        let original = accepted
            .get(&protected.id)
            .expect("an accepted counterpart is required");
        let current = candidate
            .get(&protected.id)
            .expect("a candidate case is required");
        let before_coverage = replay(original, protected);
        let after_coverage = replay(current, protected);
        assert_eq!(before_coverage, after_coverage, "{:?}", protected.id);
        total_assertions += after_coverage;
    }
    println!(
        "Behavior-only replay: 611 unchanged historical sources + 9 accepted extras; \
         all 620 candidate counterparts; {total_assertions} executed assertions on each side; \
         canonical assertions, expected returns and execution coverage preserved."
    );
}
