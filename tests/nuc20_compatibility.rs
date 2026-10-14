//! Read-only NUC20 preparation audit: behavior and syntax, never token acceptance.

mod common;

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use indexmap::IndexMap;
use nucleoid::Runtime;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::parse;
use serde::Deserialize;

const ACCEPTED_REVISION: &str = "2c852701db9c946e42c24c1e87f52b8b4d0308d0";
const DOCUMENTS: [(Document, usize); 4] = [
    (Document::Spec, 283),
    (Document::Reference, 57),
    (Document::Readme, 8),
    (Document::Examples, 63),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Document {
    Spec,
    Reference,
    Readme,
    Examples,
}

impl Document {
    fn path(self) -> &'static str {
        match self {
            Self::Spec => "nucleoid.spec.md",
            Self::Reference => "tests\\reference.md",
            Self::Readme => "README.md",
            Self::Examples => "docs\\examples.md",
        }
    }

    fn cases(self, source: &str) -> Vec<common::Case> {
        match self {
            Self::Spec | Self::Reference => common::cases(source),
            Self::Readme | Self::Examples => common::snippets(source, self.path()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct CaseId {
    document: Document,
    title: String,
}

#[derive(Deserialize)]
struct ProtectedCase {
    id: CaseId,
    assertions: Vec<String>,
    expected: Option<String>,
}

#[derive(Deserialize)]
struct Baseline {
    metric: String,
    revision: String,
    cases: Vec<ProtectedCase>,
}

#[derive(Deserialize)]
struct Extra {
    document: Document,
    title: String,
    source: String,
    expected: Option<String>,
}

fn git(root: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("--no-pager")
        .args(arguments)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn collect_cases(
    destination: &mut IndexMap<CaseId, common::Case>,
    document: Document,
    source: &str,
) {
    for case in document.cases(source) {
        let id = CaseId {
            document,
            title: case.title.clone(),
        };
        assert!(destination.insert(id, case).is_none(), "duplicate identity");
    }
}

#[test]
fn document_metadata_preserves_wire_names_paths_counts_and_order() {
    let documents = [
        (Document::Spec, "spec", "nucleoid.spec.md", 283),
        (Document::Reference, "reference", "tests\\reference.md", 57),
        (Document::Readme, "readme", "README.md", 8),
        (Document::Examples, "examples", "docs\\examples.md", 63),
    ];
    assert_eq!(
        DOCUMENTS,
        documents.map(|(document, _, _, count)| (document, count))
    );
    for (document, wire_name, path, _) in documents {
        assert_eq!(document.path(), path);
        let id: CaseId = serde_json::from_value(serde_json::json!({
            "document": wire_name,
            "title": "protected title",
        }))
        .unwrap();
        assert_eq!(
            id,
            CaseId {
                document,
                title: "protected title".to_string(),
            }
        );
        let extra: Extra = serde_json::from_value(serde_json::json!({
            "document": wire_name,
            "title": "protected title",
            "source": "42",
            "expected": "42",
        }))
        .unwrap();
        assert_eq!(extra.document, document);
        assert_eq!(extra.title, "protected title");
        assert_eq!(extra.source, "42");
        assert_eq!(extra.expected.as_deref(), Some("42"));
    }
}

#[test]
fn unknown_document_metadata_is_rejected_before_collecting_cases() {
    for document in ["unknown", "Spec", "docs/examples.md"] {
        let metadata = format!(
            r#"{{"document":"{document}","title":"protected title","source":"42","expected":"42"}}"#
        );
        let id_error = serde_json::from_str::<CaseId>(&metadata).unwrap_err();
        let Err(extra_error) = serde_json::from_str::<Extra>(&metadata) else {
            panic!("unknown extra document {document:?} must be rejected");
        };
        for error in [id_error, extra_error] {
            assert!(
                error
                    .to_string()
                    .contains(&format!("unknown variant `{document}`")),
                "{error}"
            );
        }
    }
}

/// Only parentheses on immediate, single-identifier typed named arrows may change.
/// Source positions exclude matches inside comments, strings and template text.
fn delimiter_only_changes(legacy: &str, candidate: &str) -> usize {
    let legacy_lines: Vec<_> = legacy.trim().lines().collect();
    let candidate_lines: Vec<_> = candidate.trim().lines().collect();
    assert_eq!(
        legacy_lines.len(),
        candidate_lines.len(),
        "body/layout changed"
    );
    let tokens = tokenize(legacy.trim()).unwrap();
    let mut eligible: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
    for window in tokens.windows(8) {
        let [
            definition,
            name,
            left,
            parameter,
            colon,
            annotation,
            right,
            arrow,
        ] = window
        else {
            unreachable!("eight-token window")
        };
        if definition.token == Token::Keyword(Keyword::Def)
            && matches!(name.token, Token::Identifier(_))
            && left.token == Token::LeftParen
            && matches!(parameter.token, Token::Identifier(_))
            && colon.token == Token::Colon
            && matches!(annotation.token, Token::Identifier(_))
            && right.token == Token::RightParen
            && arrow.token == Token::Arrow
            && window
                .iter()
                .all(|token| token.position.line == definition.position.line)
        {
            eligible
                .entry(definition.position.line - 1)
                .or_default()
                .push((left.position.column - 1, right.position.column - 1));
        }
    }
    let mut changed = 0;
    for (line, (before, after)) in legacy_lines.iter().zip(&candidate_lines).enumerate() {
        if before == after {
            continue;
        }
        let pairs = eligible
            .get(&line)
            .expect("only eligible typed headers may change");
        let mut spelling: Vec<char> = before.chars().collect();
        for &(left, right) in pairs.iter().rev() {
            assert_eq!(spelling[left], '(');
            assert_eq!(spelling[right], ')');
            spelling[left] = ' ';
            spelling.remove(right);
        }
        assert_eq!(
            spelling.into_iter().collect::<String>(),
            *after,
            "non-delimiter edit at line {}",
            line + 1
        );
        changed += pairs.len();
    }
    changed
}

#[test]
fn delimiter_audit_leaves_comments_strings_templates_and_legacy_signatures_intact() {
    let legacy = r#"
text = "def f(amount: int) => amount"
template = `def f(amount: int) => amount`
# def f(amount: int) => amount
def f(amount: int) => amount
def block(amount: int):
    return amount
def multiline(
    amount: int
) => amount
class Gauge(amount: int):
    this.amount = amount
def untyped amount => amount
def indented amount:
    return amount
callback = (amount: int) => amount
"#;
    let candidate = legacy.replace(
        "\ndef f(amount: int) => amount\n",
        "\ndef f amount: int => amount\n",
    );
    assert_eq!(delimiter_only_changes(legacy, &candidate), 1);
    assert_eq!(parse(legacy).unwrap(), parse(&candidate).unwrap());
}

#[test]
fn shared_assertion_walk_preserves_nested_return_template_and_catch_order() {
    let statements = parse(
        "def f amount: int => assert(amount, amount)\nclass Gauge:\n    def reading amount: int => `n=${assert(amount, amount)}`\ntry:\n    assert(1, 1)\ncatch error:\n    assert(error, error)",
    )
    .unwrap();
    assert_eq!(
        common::assertions::protected_assertions(&statements),
        [
            "assert(amount,amount)",
            "assert(amount,amount)",
            "assert(1,1)",
            "assert(error,error)",
        ]
    );
}

fn checked_execution(case: &common::Case, protected: usize) -> usize {
    let mut runtime = Runtime::new();
    common::check_with_count(&mut runtime, case, protected)
        .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
    runtime.assertions_run()
}

#[test]
#[ignore = "requires caller-sealed NUC20_COMPAT_BASELINE and NUC20_COMPAT_EXTRAS"]
fn pinned_corpus_preserves_ast_assertions_expected_returns_and_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let baseline: Baseline = serde_json::from_str(
        &fs::read_to_string(env::var_os("NUC20_COMPAT_BASELINE").expect("sealed baseline path"))
            .unwrap(),
    )
    .unwrap();
    let extras: Vec<Extra> = serde_json::from_str(
        &fs::read_to_string(env::var_os("NUC20_COMPAT_EXTRAS").expect("sealed extras path"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(baseline.metric, "nucleoid-lexer-with-layout-v1");
    assert_eq!(baseline.revision, ACCEPTED_REVISION);
    assert_eq!(git(root, &["rev-parse", "HEAD"]).trim(), ACCEPTED_REVISION);
    assert_eq!(extras.len(), 8);
    let mut legacy = IndexMap::new();
    let mut candidate = IndexMap::new();
    for (document, _) in DOCUMENTS {
        let path = document.path();
        let accepted_path = format!("{ACCEPTED_REVISION}:{}", path.replace('\\', "/"));
        collect_cases(&mut legacy, document, &git(root, &["show", &accepted_path]));
        collect_cases(
            &mut candidate,
            document,
            &fs::read_to_string(root.join(path)).unwrap(),
        );
    }
    assert_eq!(
        legacy.len(),
        403,
        "all unchanged accepted cases are required"
    );
    for extra in extras {
        assert!(
            legacy
                .insert(
                    CaseId {
                        document: extra.document,
                        title: extra.title.clone(),
                    },
                    common::Case {
                        title: extra.title,
                        source: extra.source,
                        expected: extra.expected,
                    }
                )
                .is_none(),
            "duplicate extra identity"
        );
    }
    assert_eq!(baseline.cases.len(), 411);
    assert_eq!(legacy.len(), baseline.cases.len());
    assert_eq!(candidate.len(), baseline.cases.len());
    for (document, count) in DOCUMENTS {
        let expected: Vec<_> = baseline
            .cases
            .iter()
            .filter(|case| case.id.document == document)
            .map(|case| &case.id)
            .collect();
        assert_eq!(expected.len(), count);
        for sources in [&legacy, &candidate] {
            assert_eq!(
                sources
                    .keys()
                    .filter(|id| id.document == document)
                    .collect::<Vec<_>>(),
                expected,
                "{document:?}: case identity/order changed"
            );
        }
    }
    let mut protected = 0;
    let mut expected_returns = 0;
    let mut executed = [0, 0];
    let mut shortened_headers = 0;
    for case in &baseline.cases {
        let legacy = legacy.get(&case.id).unwrap();
        let candidate = candidate.get(&case.id).unwrap();
        let legacy_ast = parse(&legacy.source).unwrap();
        let candidate_ast = parse(&candidate.source).unwrap();
        assert_eq!(legacy_ast, candidate_ast, "{:?}", case.id);
        assert_eq!(
            generate_all(&legacy_ast),
            generate_all(&candidate_ast),
            "{:?}",
            case.id
        );
        for (source, statements) in [(legacy, &legacy_ast), (candidate, &candidate_ast)] {
            assert_eq!(source.expected, case.expected, "{:?}", case.id);
            assert_eq!(
                common::assertions::protected_assertions(statements),
                case.assertions,
                "{:?}: ordered canonical assertions changed",
                case.id
            );
        }
        shortened_headers += delimiter_only_changes(&legacy.source, &candidate.source);
        let pair = [
            checked_execution(legacy, case.assertions.len()),
            checked_execution(candidate, case.assertions.len()),
        ];
        assert_eq!(pair[0], pair[1], "{:?}: executed coverage changed", case.id);
        for (total, count) in executed.iter_mut().zip(pair) {
            *total += count;
        }
        protected += case.assertions.len();
        expected_returns += usize::from(case.expected.is_some());
    }
    assert_eq!(protected, 1469);
    assert_eq!(expected_returns, 17);
    println!(
        "{} paired programs (403 pinned originals + 8 sealed extras) preserve document/title/order, complete ASTs, canonical rendering, {protected} ordered protected assertions, {expected_returns} explicit expected returns, and executed coverage {executed:?}. Only {shortened_headers} eligible typed named-arrow delimiter pairs changed; body/layout/calls/type names/comments stayed intact. No candidate token counts or acceptance evaluation ran.",
        baseline.cases.len()
    );
}
