//! Replay the accepted NUC47 baseline with an exact, authorized migration set.
//!
//! The old pre-NUC47 replay remains historical evidence in Git and the caller's
//! session. This revision pins the accepted NUC47 runtime, not that older base.
//! No baseline recording, acceptance report or verdict lives here.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use nucleoid::Runtime;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use serde::Deserialize;

const ACCEPTED: &str = "835c1ac3087a1aee3cc5cc578f02a5784a8a0c3b";
const DOCUMENTS: &[(&str, &str)] = &[
    ("spec", "nucleoid.spec.md"),
    ("reference", "tests/reference.md"),
    ("readme", "README.md"),
    ("examples", "docs/examples.md"),
];

#[derive(Debug, Clone, Copy)]
enum Migration {
    StarHeader,
    GroupedAssignments,
    StudentPositional,
    StudentMixed,
    StudentValidation,
}

// Enumerate identities, not broad syntax/classes/error filters. The three
// Student cases also exercise the explicitly requested positional/mixed forms.
const MIGRATIONS: &[(&str, &str, Migration)] = &[
    (
        "spec",
        "Nucleoid binds opted-in constructor arguments by name",
        Migration::StarHeader,
    ),
    (
        "spec",
        "Nucleoid evaluates named constructor values in source order",
        Migration::StarHeader,
    ),
    (
        "spec",
        "Nucleoid inherits opted-in named constructors",
        Migration::StarHeader,
    ),
    (
        "spec",
        "Nucleoid binds named parameters through opted-in initializers",
        Migration::StarHeader,
    ),
    (
        "spec",
        "Nucleoid keeps missing opted-in constructor parameters null",
        Migration::StarHeader,
    ),
    (
        "spec",
        "Nucleoid snapshots named constructor argument values",
        Migration::StarHeader,
    ),
    (
        "spec",
        "Nucleoid preserves positional constructor assignment arguments",
        Migration::GroupedAssignments,
    ),
    (
        "spec",
        "Nucleoid forwards named arguments to opted-in parent constructors",
        Migration::StarHeader,
    ),
    (
        "reference",
        "Nucleoid constructs an object with named parameters",
        Migration::StarHeader,
    ),
    (
        "spec",
        "Nucleoid constructs students with positional and named arguments",
        Migration::StudentPositional,
    ),
    (
        "spec",
        "Nucleoid mixes positional and named constructor arguments",
        Migration::StudentMixed,
    ),
    (
        "spec",
        "Nucleoid validates constructor names without caller assignments",
        Migration::StudentValidation,
    ),
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
    tokens: usize,
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

#[derive(Deserialize)]
struct Authorization {
    requested_change: String,
    approved_user_instruction: String,
    accepted_revision: String,
    approved_compatibility_exceptions: Vec<String>,
    legacy_assignment_migration: LegacyMigration,
}

#[derive(Deserialize)]
struct LegacyMigration {
    document: String,
    title: String,
}

fn artifact<T: serde::de::DeserializeOwned>(variable: &str) -> T {
    let path = std::env::var_os(variable).unwrap_or_else(|| panic!("{variable} is required"));
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn baseline() -> Baseline {
    let baseline: Baseline = artifact("NUC47_BASELINE");
    assert_eq!(baseline.revision, ACCEPTED);
    assert_eq!(baseline.metric, "nucleoid-lexer-with-layout-v1");
    assert_eq!(baseline.cases.len(), 624);
    baseline
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

fn sources(accepted: bool) -> BTreeMap<Identity, common::Case> {
    let mut cases = BTreeMap::new();
    for &(document, path) in DOCUMENTS {
        let source = if accepted {
            // Four explicit documents in Git, never the main worktree or an
            // environment file. The immutable baseline was executed separately
            // by this accepted revision before candidate runtime edits.
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
        for case in document_cases(document, path, &source) {
            insert(&mut cases, document, case);
        }
    }
    if accepted {
        assert_eq!(cases.len(), 620);
        let extras: Vec<ExtraSource> = artifact("NUC47_EXTRA_BASELINES");
        assert_eq!(extras.len(), 4);
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
    assert_eq!(cases.len(), 624);
    cases
}

fn program(case: &common::Case) -> &str {
    let source = case.source.trim();
    source
        .strip_prefix(&format!("# {}", case.title))
        .unwrap_or(source)
        .trim()
}

fn migration(id: &Identity) -> Option<Migration> {
    MIGRATIONS
        .iter()
        .find(|(document, title, _)| *document == id.document && *title == id.title)
        .map(|(_, _, migration)| *migration)
}

fn migrated_program(original: &common::Case, migration: Migration) -> String {
    let original = program(original);
    if matches!(migration, Migration::GroupedAssignments) {
        return original
            .strip_prefix("'use imperative'\n")
            .expect("the protected legacy case has its original directive")
            .replace("a = 0\nb = 0", "a = b = 0")
            .replace(
                "legacy = Legacy(b = 2, a = 1)",
                "legacy = Legacy((b = 2), (a = 1))",
            );
    }
    let unmarked = original
        .lines()
        .map(|line| {
            if line.starts_with("class ") {
                line.replace("(*, ", "(").replace("(*)", "")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(
        unmarked, original,
        "a listed star-header migration must remove a star"
    );
    match migration {
        Migration::StarHeader => unmarked,
        Migration::StudentPositional => unmarked.replace(
            "positional = Student(name=\"Can\")",
            "positional = Student(\"Can\")",
        ),
        Migration::StudentMixed => unmarked.replace(
            "mixed = Student(name=\"Can\", age=20)",
            "mixed = Student(\"Can\", age=20)",
        ),
        Migration::StudentValidation => unmarked.replace(
            "Student(name=\"Can\", name=\"Other\", age=20)",
            "Student(\"Can\", name=\"Other\", age=20)",
        ),
        Migration::GroupedAssignments => unreachable!("handled above"),
    }
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
#[ignore = "requires immutable NUC47_BASELINE, NUC47_EXTRA_BASELINES and NUC47_APPROVED_MIGRATION"]
fn accepted_sources_and_exact_authorized_migrations_preserve_behavior() {
    let authorization: Authorization = artifact("NUC47_APPROVED_MIGRATION");
    assert_eq!(authorization.accepted_revision, ACCEPTED);
    assert_eq!(
        authorization.requested_change,
        "default_positional_and_named_constructor_arguments"
    );
    assert_eq!(
        authorization.approved_user_instruction,
        "I don't want * at all but I agreed with the table"
    );
    assert_eq!(
        authorization.approved_compatibility_exceptions,
        [
            "remove_constructor_star_header_syntax",
            "interpret_plain_constructor_name_equals_value_arguments_as_labels_not_caller_assignments",
        ]
    );
    assert_eq!(authorization.legacy_assignment_migration.document, "spec");
    assert_eq!(
        authorization.legacy_assignment_migration.title,
        "Nucleoid preserves positional constructor assignment arguments"
    );
    let baseline = baseline();
    let accepted = sources(true);
    let candidate = sources(false);
    assert!(accepted.keys().eq(candidate.keys()));
    let mut identities = BTreeSet::new();
    let mut unchanged = 0;
    let mut migrated = 0;
    let mut total_assertions = 0;
    let mut migrated_assertions = 0;
    let mut returns = 0;
    for protected in &baseline.cases {
        assert!(identities.insert(protected.id.clone()));
        let original = accepted
            .get(&protected.id)
            .expect("an accepted counterpart is required");
        let current = candidate
            .get(&protected.id)
            .expect("a candidate case is required");
        assert_eq!(original.expected, protected.expected, "{:?}", protected.id);
        let coverage = replay(current, protected);
        if let Some(migration) = migration(&protected.id) {
            assert_eq!(
                program(current),
                migrated_program(original, migration),
                "{:?}",
                protected.id
            );
            // All twelve exact migration cases are straight-line or use catch
            // assertions once. Their pre-executed baseline coverage is the
            // canonical count, not an inferred candidate before measurement.
            assert_eq!(coverage, protected.assertions.len(), "{:?}", protected.id);
            println!(
                "migrated {migration:?}: {:?}; {coverage} assertions",
                protected.id
            );
            migrated += 1;
            migrated_assertions += coverage;
        } else {
            assert_eq!(program(original), program(current), "{:?}", protected.id);
            let before_coverage = replay(original, protected);
            assert_eq!(before_coverage, coverage, "{:?}", protected.id);
            unchanged += 1;
        }
        total_assertions += coverage;
        returns += usize::from(protected.expected.is_some());
    }
    assert_eq!(unchanged, 612);
    assert_eq!(migrated, MIGRATIONS.len());
    assert_eq!(migrated, 12);
    println!(
        "Compatibility replay: {unchanged} unchanged accepted sources executed unchanged; \
         {migrated} exact authorized migrations, {migrated_assertions} migrated assertions; \
         all 624 candidate programs, {total_assertions} executed assertions and {returns} \
         explicit returns. Canonical assertions/order, expected returns and coverage preserved. \
         Removed star syntax and bare constructor assignments are intentional breaks, \
         not unchanged-source compatibility claims."
    );
}

#[test]
#[ignore = "development lexer measurements only; requires the immutable NUC47_BASELINE"]
fn development_lexer_counts_without_acceptance_entry_point() {
    let baseline = baseline();
    let candidate = sources(false);
    let mut less = 0;
    let mut same = 0;
    let mut more = 0;
    for protected in &baseline.cases {
        let current = candidate.get(&protected.id).unwrap();
        let after = tokenize(current.source.trim())
            .unwrap()
            .iter()
            .filter(|spanned| spanned.token != Token::Eof)
            .count();
        match after.cmp(&protected.tokens) {
            std::cmp::Ordering::Less => less += 1,
            std::cmp::Ordering::Equal => same += 1,
            std::cmp::Ordering::Greater => more += 1,
        }
        if after != protected.tokens || migration(&protected.id).is_some() {
            println!(
                "{:?}: before={}, after={after}",
                protected.id, protected.tokens
            );
        }
    }
    println!("Development lexer counts only: {less} less, {same} same, {more} more; no verdict.");
}
