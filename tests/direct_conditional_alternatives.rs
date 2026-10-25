//! Optional direct alternatives, with embedded accepted-syntax controls.
//! Ordinary tests depend on neither Git nor a proposal-session directory.

mod common;
#[allow(dead_code)]
#[path = "support/conditional_headers.rs"]
mod conditional_headers;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};
use std::sync::Arc;

use nucleoid::graph::{NodeKey, NodeKind};
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_program};
use nucleoid::lang::evaluation::Flow;
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};
use serde::Deserialize;

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

/// The sealed fixtures have only these line-initial, real-indented keyword
/// sites. This is deliberately not a general source-to-source rewriter.
fn direct_counterpart(source: &str) -> (String, usize) {
    let mut sites = 0;
    let rewritten = source
        .split_inclusive('\n')
        .map(|line| {
            let body = line.trim_start_matches([' ', '\t']);
            if let Some(condition) = body.strip_prefix("else if ") {
                let prefix = &line[..line.len() - body.len()];
                let spelling = if sites % 2 != 0 { "elif" } else { "else" };
                sites += 1;
                format!("{prefix}{spelling} {condition}")
            } else {
                line.to_string()
            }
        })
        .collect();
    (rewritten, sites)
}

fn equivalent(actual: &str, explicit: &str) -> Vec<Stmt> {
    let actual = parse_program(actual).unwrap_or_else(|error| panic!("{actual}\n{error}"));
    let explicit = parse_program(explicit).unwrap_or_else(|error| panic!("{explicit}\n{error}"));
    assert_eq!(actual.statements, explicit.statements);
    assert_eq!(actual.positions, explicit.positions);
    assert_eq!(
        generate_all(&actual.statements),
        generate_all(&explicit.statements)
    );
    actual.statements
}

fn checked(runtime: &mut Runtime, source: &str, count: usize, expected: Option<&str>) -> Value {
    assert_eq!(
        common::assertions::protected_assertions(&parse(source).unwrap()).len(),
        count
    );
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "direct conditional regression".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count);
    value
}

fn graphs_match(actual: &Runtime, explicit: &Runtime) {
    assert!(actual.graph.keys().eq(explicit.graph.keys()));
    for node in actual.graph.nodes() {
        let other = explicit.graph.retrieve(&node.key).unwrap();
        assert_eq!(node.kind, other.kind, "{}", node.key);
        assert_eq!(node.dependencies, other.dependencies, "{}", node.key);
        assert_eq!(node.dependents, other.dependents, "{}", node.key);
        assert!(
            actual
                .graph
                .shape_dependencies(&node.key)
                .eq(explicit.graph.shape_dependencies(&node.key))
        );
    }
}

fn check_pair(actual: &str, explicit: &str, count: usize, expected: Option<&str>) {
    let ast = equivalent(actual, explicit);
    assert_eq!(
        common::assertions::protected_assertions(&ast),
        common::assertions::protected_assertions(&parse(explicit).unwrap())
    );
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, source) in runtimes.iter_mut().zip([actual, explicit]) {
        checked(runtime, source, count, expected);
    }
    // Do not compare allocated function values or clock results across
    // runtimes. Assertions and explicit returns are checked in each runtime.
    graphs_match(&runtimes[0], &runtimes[1]);
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Counterpart {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

// Exact V2 counterparts pre-executed by the accepted runtime before the
// immutable baseline. The unsupported V1 receiver-update case is not here.
const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid selects direct conditional alternatives",
    "source": "# Nucleoid selects direct conditional alternatives\n\ndef classify(state, urgent):\n    if state == \"blocked\":\n        return \"hold\"\n    else if urgent:\n        return \"fast\"\n    else if state == \"ready\":\n        return \"send\"\n    else:\n        return \"wait\"\nstate = \"ready\"\nurgent = false\nlabel = classify(state, urgent)\nassert(label, \"send\")\nurgent = true\nassert(label, \"fast\")\nstate = \"blocked\"\nassert(label, \"hold\")\nurgent = false\nassert(label, \"hold\")\nstate = \"queued\"\nassert(label, \"wait\")\nlabel\n",
    "expected": "\"wait\""
  },
  {
    "document": "spec",
    "title": "Nucleoid keeps direct conditional class alternatives reactive",
    "source": "# Nucleoid keeps direct conditional class alternatives reactive\n\nclass Ticket\nfirst = Ticket()\nfirst.state = \"new\"\nfirst.priority = \"normal\"\nsecond = Ticket()\nsecond.state = \"review\"\nsecond.priority = \"urgent\"\nif $Ticket.state == \"blocked\":\n    $Ticket.queue = \"hold\"\nelse if $Ticket.priority == \"urgent\":\n    $Ticket.queue = \"fast\"\nelse if $Ticket.state == \"review\":\n    $Ticket.queue = \"review\"\nelse:\n    $Ticket.queue = \"normal\"\nassert(first.queue, \"normal\")\nassert(second.queue, \"fast\")\nfirst.state = \"blocked\"\nassert(first.queue, \"hold\")\nsecond.priority = \"normal\"\nassert(second.queue, \"review\")\nfirst.priority = \"urgent\"\nassert(first.queue, \"hold\")\nfirst.state = \"new\"\nassert(first.queue, \"fast\")\nassert(second.queue, \"review\")\nsecond.state = \"done\"\nassert(second.queue, \"normal\")\n",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid preserves direct alternative evaluation order",
    "source": "# Nucleoid preserves direct alternative evaluation order\n\ndef decide(first, second, third):\n    if first:\n        return \"first\"\n    else if second:\n        return \"second\"\n    else if third:\n        return \"third\"\n    else:\n        return \"none\"\nassert(decide(true, true, true), \"first\")\nassert(decide(false, true, true), \"second\")\nassert(decide(false, false, true), \"third\")\nassert(decide(false, false, false), \"none\")\ndef guarded(value):\n    if value:\n        return \"ready\"\n    else if missing:\n        return \"bad\"\n    else:\n        return \"other\"\nassert(guarded(true), \"ready\")\ntry:\n    guarded(false)\ncatch error:\n    assert(error, ReferenceError(\"missing is not defined\"))\n",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid preserves direct alternative scope and return flow",
    "source": "# Nucleoid preserves direct alternative scope and return flow\n\nclass Router:\n    def init(prefix):\n        this.prefix = prefix\n    def choose(status, fallback):\n        if status == \"blocked\":\n            return this.prefix + \"hold\"\n        else if status == \"ready\":\n            return this.prefix + \"send\"\n        else if fallback:\n            return this.prefix + \"later\"\n        else:\n            return this.prefix + \"skip\"\nrouter = Router(\"a:\")\nstatus = \"ready\"\nlive = router.choose(status, false)\nsnapshot = live.value\nassert(live, \"a:send\")\nassert(snapshot, \"a:send\")\nassert(router.choose(\"unknown\", false), \"a:skip\")\nassert(router.choose(\"unknown\", true), \"a:later\")\nassert(router.choose(\"blocked\", true), \"a:hold\")\nassert(router.choose(\"ready\", true), \"a:send\")\nassert(status, \"ready\")\nassert(router.prefix, \"a:\")\n",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid rolls back failed direct alternatives",
    "source": "# Nucleoid rolls back failed direct alternatives\n\nsource = \"ready\"\ndef route(status):\n    if status == \"ready\":\n        return \"send\"\n    else if status == \"hold\":\n        return \"wait\"\n    else:\n        return \"skip\"\nlive = route(source)\nassert(live, \"send\")\ntry:\n    source = \"hold\"\n    def route(status):\n        if status == \"ready\":\n            return \"send\"\n        else if status == \"hold\":\n            throw \"blocked\"\n        else:\n            return \"skip\"\ncatch error:\n    assert(error, \"blocked\")\nassert(source, \"ready\")\nassert(live, \"send\")\nsource = \"hold\"\nassert(live, \"wait\")\nsource = \"unknown\"\nassert(live, \"skip\")\nlive\n",
    "expected": "\"skip\""
  },
  {
    "document": "reference",
    "title": "Nucleoid routes records through direct alternatives",
    "source": "# Nucleoid routes records through direct alternatives\n\nclass Job\nurgent = Job()\nurgent.state = \"ready\"\nurgent.force = true\nnormal = Job()\nnormal.state = \"queued\"\nnormal.force = false\nif $Job.state == \"blocked\":\n    $Job.lane = \"hold\"\nelse if $Job.force:\n    $Job.lane = \"fast\"\nelse if $Job.state == \"ready\":\n    $Job.lane = \"send\"\nelse:\n    $Job.lane = \"wait\"\nassert(urgent.lane, \"fast\")\nassert(normal.lane, \"wait\")\nnormal.state = \"ready\"\nassert(normal.lane, \"send\")\nurgent.state = \"blocked\"\nassert(urgent.lane, \"hold\")\nnormal.force = true\nassert(normal.lane, \"fast\")\n",
    "expected": null
  },
  {
    "document": "readme",
    "title": "Conditional alternatives omit the second keyword",
    "source": "def label(state):\n    if state == \"done\":\n        return \"archive\"\n    else if state == \"ready\":\n        return \"send\"\n    else:\n        return \"wait\"\nstate = \"ready\"\nresult = label(state)\nassert(result, \"send\")\nstate = \"done\"\nassert(result, \"archive\")\nstate = \"queued\"\nassert(result, \"wait\")\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Conditional alternatives keep frozen classifiers and live inputs separate",
    "source": "def classify(state, urgent):\n    if state == \"blocked\":\n        return \"hold\"\n    else if urgent:\n        return \"fast\"\n    else if state == \"ready\":\n        return \"send\"\n    else:\n        return \"wait\"\nstate = \"ready\"\nurgent = false\nlive = classify(state, urgent)\nfrozen = classify.value\nusingFrozen = frozen(state, urgent)\nsnapshot = usingFrozen.value\nassert(live, \"send\")\nassert(usingFrozen, \"send\")\nurgent = true\nassert(live, \"fast\")\nassert(usingFrozen, \"fast\")\ndef classify(state, urgent):\n    if state == \"blocked\":\n        return \"pause\"\n    else if urgent:\n        return \"rush\"\n    else if state == \"ready\":\n        return \"go\"\n    else:\n        return \"queue\"\nassert(live, \"rush\")\nassert(usingFrozen, \"fast\")\nassert(snapshot, \"send\")\nstate = \"blocked\"\nassert(live, \"pause\")\nassert(usingFrozen, \"hold\")\n",
    "expected": null
  }
]"###;

#[test]
fn exact_eight_v2_pairs_preserve_eighteen_keyword_sites_and_execute_fifty_assertions() {
    let fixtures: Vec<Counterpart> = serde_json::from_str(ACCEPTED).unwrap();
    let documents = documents();
    let mut totals = (0, 0, 0, 0);
    for (index, fixture) in fixtures.iter().enumerate() {
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == fixture.document)
            .unwrap()
            .1;
        let matching = cases
            .iter()
            .enumerate()
            .filter(|(_, case)| case.title == fixture.title)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "{}", fixture.title);
        let (position, case) = matching[0];
        assert_eq!(position, [388, 389, 390, 391, 392, 78, 29, 84][index]);
        assert_eq!(case.expected, fixture.expected);
        // Include direct `else` at the first top-level class-rule site.
        // The production exporter must retain it with the whole chain.
        let (rewritten, sites) = direct_counterpart(&fixture.source);
        assert_eq!(sites, [2, 2, 3, 2, 2, 2, 1, 4][index]);
        // NUC49 optionally removes only genuinely indented header colons at
        // these same eight identities. Retain every original keyword-site,
        // source, assertion, order, position and return protection.
        let (colon_free, colons) = conditional_headers::omit(&rewritten);
        assert_eq!(colons, [3, 3, 5, 3, 4, 3, 2, 6][index]);
        assert_eq!(case.source.trim(), colon_free.trim(), "{}", fixture.title);
        let count = [5, 8, 6, 8, 6, 5, 3, 9][index];
        // Outer case-container whitespace is not an AST source position.
        // The complete actual executable program is otherwise unchanged.
        check_pair(
            case.source.trim(),
            fixture.source.trim(),
            count,
            fixture.expected.as_deref(),
        );
        totals.0 += 1;
        totals.1 += sites;
        totals.2 += count;
        totals.3 += usize::from(fixture.expected.is_some());
    }
    assert_eq!(totals, (8, 18, 50, 2));
}

#[test]
fn all_579_historical_source_boundaries_comments_order_and_returns_remain_exact() {
    for ((name, cases), (count, expected)) in documents().into_iter().zip([
        (388, 0xad2a77d7bd1b981a_u64),
        (78, 0xe0f48098c2cead7a),
        (29, 0x9cfbdb03631274e8),
        (84, 0x2e531e2d5e46bcd2),
    ]) {
        assert!(cases.len() >= count);
        let mut seal = 0xcbf29ce484222325_u64;
        for case in &cases[..count] {
            for field in [
                &case.title,
                &case.source,
                case.expected.as_deref().unwrap_or(""),
            ] {
                for byte in field.bytes().chain([0xff]) {
                    seal = (seal ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                }
            }
        }
        assert_eq!(
            seal, expected,
            "{name}: historical explicit-syntax controls"
        );
    }
}

#[test]
fn balanced_conditions_and_genuine_layout_keep_whole_asts_canonical_source_and_positions() {
    for condition in [
        "ready",
        "not ready and available",
        "first = second",
        "items[1:3] == [2, 3]",
        "items[:].length > 0",
        "({ready: {nested: true}}).ready.nested",
        "check({ready: value}, items[:], (item: int) => item)",
        "(item: int) => { return item }",
        "(\n    ready\n)",
        "({\n    ready: true\n}).ready",
        "\"else if :\" == \"elif :\"",
        "/[:{}]/.test(text)",
        "`else if ${value}:`",
    ] {
        for layout in [
            "\n    pass",
            "\r\n\tpass",
            " # comment\n\n# blank\n    # body\n    pass",
            "\n;\n    pass",
            "\n    ;\n    pass;",
        ] {
            for spelling in ["else", "elif"] {
                let actual = format!("if false: pass\n{spelling} {condition}:{layout}\nnext = 3");
                let explicit = format!("if false: pass\nelse if {condition}:{layout}\nnext = 3");
                let statements = equivalent(&actual, &explicit);
                let Stmt::If { alternate, .. } = &statements[0] else {
                    panic!("outer conditional");
                };
                assert!(matches!(alternate.as_deref(), Some(Stmt::If { .. })));
            }
        }
    }
}

#[test]
fn keyword_aliases_final_alternatives_and_explicit_if_priority_remain_compatible() {
    assert_eq!(
        tokenize("else").unwrap()[0].token,
        Token::Keyword(Keyword::Else)
    );
    assert_eq!(
        tokenize("elif").unwrap()[0].token,
        Token::Keyword(Keyword::Else)
    );
    for spelling in ["else:", "elif:", "else", "elif"] {
        let actual =
            format!("if false:\n    result = 1\n{spelling}\n    result = 2\nassert(result, 2)");
        let explicit = "if false:\n    result = 1\nelse:\n    result = 2\nassert(result, 2)";
        check_pair(&actual, explicit, 1, None);
    }
    for body in ["result = 2", "{ result = 2 }", "result = 1; result = 2"] {
        let explicit = format!(
            "if false: result = 0\nelse if true: {body}\nelse: result = 3\nassert(result, 2)"
        );
        let alias = explicit.replace("else if", "elif if");
        check_pair(&alias, &explicit, 1, None);
    }
}

#[test]
fn nested_mixed_suites_and_dangling_else_association_use_the_same_recursive_parser() {
    for explicit in [
        "if true:\n    if false: result = 1\n    else if true:\n        result = 2\n    else: result = 3\nelse if true:\n    result = 4\nassert(result, 2)",
        "if false: { result = 1 }\nelse if true:\n    if false:\n        result = 2\n    else if true:\n        result = 3\n    elif\n        result = 4\nelse: { result = 5 }\nassert(result, 3)",
        "if false: result = 1\nelse if false: { result = 2 }\nelse if true:\n    result = 3\n    if false: result = 4 else: result = 5\nelif: result = 6\nassert(result, 5)",
        "if true: if false: result = 1 else if true:\n    result = 2\nelse: result = 3\nassert(result, 2)",
    ] {
        let (actual, _) = direct_counterpart(explicit);
        // Inline explicit clauses are deliberately not new direct forms.
        let actual = actual.replace(
            "else false: { result = 2 }",
            "else if false: { result = 2 }",
        );
        check_pair(&actual, explicit, 1, None);
    }
}

fn syntax(source: &str, message: &str, line: usize, column: usize) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    assert_eq!(error.message(), message, "{source}");
    assert_eq!(
        error.position(),
        Some(Position::new(line, column)),
        "{source}"
    );
}

#[test]
fn nonqualifying_headers_keep_original_final_else_diagnostics_and_positions() {
    for spelling in ["else", "elif"] {
        for tail in [
            "ready: pass",
            "ready: { pass }",
            "ready:;\n    pass",
            "ready:\npass",
            "ready:\n    # no body",
            "ready:\n",
            "ready:",
            "ready\n    # no body",
            "ready;\n    pass",
            "ready\nother:\n    pass",
            "ready):\n    pass",
            "ready[):\n    pass",
        ] {
            syntax(
                &format!("if false: pass\n{spelling} {tail}"),
                "Expected ':' but found identifier 'ready'",
                2,
                6,
            );
        }
        syntax(
            &format!("if false: pass\n{spelling}"),
            "Expected ':' but found end of input",
            2,
            5,
        );
        syntax(
            &format!("if false: pass\n{spelling}\n    # no body"),
            "Expected ':' but found end of line",
            2,
            1,
        );
        syntax(
            &format!("if false: pass\n{spelling};\n    pass"),
            "Expected ':' but found ';'",
            2,
            5,
        );
        syntax(
            &format!("{spelling} true:\n    pass"),
            "Unexpected keyword 'else'",
            1,
            6,
        );
    }
}

#[test]
fn qualified_malformed_conditions_and_bodies_report_real_errors_without_fallback() {
    for spelling in ["else", "elif"] {
        for (tail, message, line, column) in [
            ("true +:\n    pass", "Unexpected ':'", 2, 12),
            ("(true +):\n    pass", "Unexpected ')'", 2, 14),
            ("true:\n    value = (1 +)", "Unexpected ')'", 3, 18),
            ("true:\n    value = @", "Unexpected character '@'", 3, 13),
        ] {
            syntax(
                &format!("if false: pass\n{spelling} {tail}"),
                message,
                line,
                column,
            );
        }
    }
    for spelling in ["else if", "elif if"] {
        syntax(
            &format!("if false: pass\n{spelling} true\npass"),
            "Expected ':' but found end of line",
            2,
            9,
        );
    }
}

fn rule_dependencies(runtime: &Runtime) -> Vec<&str> {
    runtime
        .graph
        .retrieve(&NodeKey::conditional(
            &Expr::Identifier("first".into()),
            None,
        ))
        .unwrap()
        .dependencies
        .iter()
        .map(NodeKey::as_str)
        .collect()
}

#[test]
fn first_match_short_circuit_actual_reads_dependency_replacement_and_recovery_stay_identical() {
    let setup = "first = true\nsecond = false\nactive = \"a\"\nfallback = \"b\"";
    let explicit = "if first:\n    selected = active\nelse if second and missing:\n    selected = \"bad\"\nelse if true:\n    selected = fallback";
    let (actual, _) = direct_counterpart(explicit);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, source) in runtimes.iter_mut().zip([actual.as_str(), explicit]) {
        checked(runtime, setup, 0, None);
        checked(runtime, source, 0, None);
        assert_eq!(runtime.run("selected").unwrap(), Value::string("a"));
        assert_eq!(rule_dependencies(runtime), ["first", "active"]);
        runtime.run("first = false").unwrap();
        assert_eq!(runtime.run("selected").unwrap(), Value::string("b"));
        assert_eq!(rule_dependencies(runtime), ["first", "second", "fallback"]);
        let error = runtime.run("second = true").unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(runtime.run("second").unwrap(), Value::Bool(false));
        assert_eq!(runtime.run("selected").unwrap(), Value::string("b"));
        assert_eq!(rule_dependencies(runtime), ["first", "second", "fallback"]);
        runtime.run("fallback = \"c\"").unwrap();
        assert_eq!(runtime.run("selected").unwrap(), Value::string("c"));
        assert_eq!(
            runtime
                .graph
                .nodes()
                .filter(|node| node.kind == NodeKind::If)
                .count(),
            1
        );
    }
    graphs_match(&runtimes[0], &runtimes[1]);
    // Replacing the same root condition changes the branch reads, not the
    // declaration's graph identity or declaration order.
    let replacement = "if first:\n    selected = fallback\nelse if true:\n    selected = active";
    let (actual, _) = direct_counterpart(replacement);
    for (runtime, source) in runtimes.iter_mut().zip([actual.as_str(), replacement]) {
        checked(runtime, source, 0, None);
        assert_eq!(runtime.run("selected").unwrap(), Value::string("a"));
        assert_eq!(rule_dependencies(runtime), ["first", "active"]);
    }
    graphs_match(&runtimes[0], &runtimes[1]);
}

#[test]
fn independent_class_rules_and_existing_truthiness_do_not_gain_a_new_evaluation_path() {
    let explicit = r#"class Item
first = Item()
first.status = null
second = Item()
second.status = "ready"
if $Item.status == "blocked":
    $Item.lane = "hold"
else if $Item.status:
    $Item.lane = "send"
else:
    $Item.lane = "wait"
assert(first.lane, "wait")
assert(second.lane, "send")
first.status = "blocked"
assert(first.lane, "hold")
second.status = ""
assert(second.lane, "wait")
first.status = "ready"
assert(first.lane, "send")"#;
    check_pair(&direct_counterpart(explicit).0, explicit, 5, None);
}

#[test]
fn live_frozen_functions_and_snapshot_identities_are_checked_within_each_runtime() {
    let explicit = r#"def classify(value):
    if value == "closed":
        return "archive"
    else if value == "open":
        return "work"
    else:
        return "wait"
state = "open"
saved = classify.value
alias = saved.value
live = classify(state)
frozen = saved(state)
snapshot = frozen.value
assert(live, "work")
assert(frozen, "work")
def classify(value):
    if value == "closed":
        return "file"
    else if value == "open":
        return "review"
    else:
        return "queue"
assert(live, "review")
assert(frozen, "work")
state = "closed"
assert(live, "file")
assert(frozen, "archive")
assert(snapshot, "work")"#;
    let (actual, _) = direct_counterpart(explicit);
    equivalent(&actual, explicit);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, source) in runtimes.iter_mut().zip([actual.as_str(), explicit]) {
        checked(runtime, source, 7, None);
        let (Value::Function(saved), Value::Function(alias)) = (
            runtime.state.variable("saved").unwrap(),
            runtime.state.variable("alias").unwrap(),
        ) else {
            panic!("function identities");
        };
        let current = runtime.state.function("classify").unwrap();
        assert!(Arc::ptr_eq(saved, alias));
        assert!(!Arc::ptr_eq(saved, current));
        assert!(
            runtime
                .graph
                .reaches(&NodeKey::function("classify"), &NodeKey::variable("live"))
        );
        assert!(
            !runtime
                .graph
                .reaches(&NodeKey::function("classify"), &NodeKey::variable("frozen"))
        );
        assert!(
            runtime
                .graph
                .retrieve(&NodeKey::variable("snapshot"))
                .unwrap()
                .dependencies
                .is_empty()
        );
    }
    graphs_match(&runtimes[0], &runtimes[1]);
}

#[test]
fn direct_return_flow_preserves_caller_scope_receivers_frames_and_noncapture() {
    for expression in ["value + 1", "missing"] {
        let explicit = format!("if false: pass\nelse if true:\n    return {expression}");
        let (actual, _) = direct_counterpart(&explicit);
        let ast = equivalent(&actual, &explicit);
        for source in [&actual, &explicit] {
            let mut runtime = Runtime::new();
            let mut scope = Scope::new();
            scope.push();
            scope.declare("value", Value::Number(99.0));
            let receiver = ObjectId::named("caller");
            scope.set_this(Some(receiver.clone()));
            scope.set_instance(Some(receiver.clone()));
            assert_eq!(parse(source).unwrap(), ast);
            let mut node = Nuc::convert(&mut runtime, &mut scope, &ast[0]).unwrap();
            let result = node.run(&mut runtime, &mut scope);
            if expression == "missing" {
                let error = result.err().expect("missing read must fail");
                assert_eq!(error.kind(), ErrorKind::Reference);
                assert_eq!(error.message(), "missing is not defined");
            } else {
                assert!(matches!(
                    result.unwrap().flow,
                    Flow::Return(Value::Number(100.0))
                ));
            }
            assert_eq!(scope.depth(), 2);
            assert_eq!(scope.retrieve("value"), Some(&Value::Number(99.0)));
            assert_eq!(scope.this(), Some(&receiver));
            assert_eq!(scope.instance(), Some(&receiver));
            runtime
                .run("input = 2\noutput = input * 3\ninput = 4")
                .unwrap();
            assert_eq!(runtime.run("output").unwrap(), Value::Number(12.0));
        }
    }
    let explicit = "def factory(hidden):\n    if false:\n        return null\n    else if true:\n        return value => hidden + value\nfactory(\"x\")(\"y\")";
    let (actual, _) = direct_counterpart(explicit);
    equivalent(&actual, explicit);
    for source in [&actual, explicit] {
        let error = Runtime::new().run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "hidden is not defined");
        assert_eq!(error.position(), Some(Position::new(6, 1)));
    }
}

// Retained V1 source, including the unsupported expectations after the actual
// failure. It is an error/rollback fixture, NEVER a measured successful case.
const INHERITED_RECEIVER_FAILURE: &str = r#"# Nucleoid preserves direct alternative scope and return flow

class Router:
    def init(prefix):
        this.prefix = prefix
    def choose(status, fallback):
        if status == "blocked":
            return this.prefix + "hold"
        else if status == "ready":
            return this.prefix + "send"
        else if fallback:
            return this.prefix + "later"
        else:
            return this.prefix + "skip"
router = Router("a:")
status = "ready"
live = router.choose(status, false)
snapshot = live.value
assert(live, "a:send")
assert(snapshot, "a:send")
router.prefix = "b:"
assert(live, "b:send")
assert(snapshot, "a:send")
status = "unknown"
assert(live, "b:skip")
assert(router.choose("unknown", true), "b:later")
assert(router.choose("blocked", true), "b:hold")
assert(status, "unknown")
"#;

#[test]
fn retained_v1_receiver_failure_keeps_exact_error_position_and_atomic_rollback() {
    let explicit = INHERITED_RECEIVER_FAILURE;
    let (actual, _) = direct_counterpart(explicit);
    equivalent(&actual, explicit);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    for (runtime, source) in runtimes.iter_mut().zip([actual.as_str(), explicit]) {
        runtime.run("kept = 7").unwrap();
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "this is not defined");
        assert_eq!(error.position(), Some(Position::new(21, 1)));
        // Only the two real assertions BEFORE the inherited failure ran.
        assert_eq!(runtime.assertions_run(), 2);
        assert!(runtime.take_assertions().is_empty());
        assert_eq!(runtime.state.variable("kept"), Some(&Value::Number(7.0)));
        assert!(runtime.state.class("Router").is_none());
        for name in ["router", "status", "live", "snapshot"] {
            assert!(runtime.state.variable(name).is_none());
        }
        assert_eq!(runtime.graph.len(), 1);
        runtime
            .run("input = 2\noutput = input * 3\ninput = 4")
            .unwrap();
        assert_eq!(runtime.run("output").unwrap(), Value::Number(12.0));
    }
    graphs_match(&runtimes[0], &runtimes[1]);
}

#[test]
fn uncaught_direct_branch_errors_restore_state_graph_queue_and_later_transactions() {
    let explicit = "if false: pass\nelse if true:\n    marker = 9\n    throw \"STOP\"";
    let (actual, _) = direct_counterpart(explicit);
    equivalent(&actual, explicit);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let mut errors = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([actual.as_str(), explicit]) {
        runtime
            .run("marker = 7\ninput = 2\noutput = input * 3")
            .unwrap();
        let error = runtime.run(source).unwrap_err();
        errors.push(error);
        assert_eq!(runtime.run("marker").unwrap(), Value::Number(7.0));
        runtime.run("input = 4").unwrap();
        assert_eq!(runtime.run("output").unwrap(), Value::Number(12.0));
    }
    assert_eq!(errors[0], errors[1]);
    assert_eq!(errors[0].kind(), ErrorKind::Thrown);
    assert_eq!(errors[0].message(), "STOP");
    assert_eq!(errors[0].position(), Some(Position::new(1, 1)));
    graphs_match(&runtimes[0], &runtimes[1]);
}

#[test]
fn inherited_expression_and_statement_resource_bounds_are_not_relaxed() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            for count in [127, 128, 129] {
                for (condition, allowed) in [
                    (format!("value{}", " + 1".repeat(count)), count <= 128),
                    // The inherited postfix loop also counts its terminal
                    // probe; preserve that boundary rather than repairing it.
                    (format!("root{}", ".next".repeat(count)), count < 128),
                ] {
                    let explicit = format!("if false: pass\nelse if {condition}:\n    pass");
                    let (actual, _) = direct_counterpart(&explicit);
                    if allowed {
                        equivalent(&actual, &explicit);
                    } else {
                        for source in [&actual, &explicit] {
                            let error = Runtime::check(source).unwrap_err();
                            assert_eq!(error.kind(), ErrorKind::Syntax);
                            assert!(matches!(
                                error.message().as_str(),
                                "Expression has too many operands"
                                    | "Expression has too many parts"
                            ));
                            let mut runtime = Runtime::new();
                            runtime.run("kept = 7").unwrap();
                            assert!(runtime.run(&format!("kept = 3\n{source}")).is_err());
                            assert_eq!(runtime.state.variable("kept"), Some(&Value::Number(7.0)));
                        }
                    }
                }
            }
            for expression in [
                format!("{}value", "-".repeat(80)),
                format!("{}true{}", "(".repeat(80), ")".repeat(80)),
            ] {
                let explicit = format!("if false: pass\nelse if {expression}:\n    pass");
                let (actual, _) = direct_counterpart(&explicit);
                for source in [&actual, &explicit] {
                    let error = Runtime::check(source).unwrap_err();
                    assert_eq!(error.message(), "Expressions are nested too deeply");
                }
            }
            let mut body = String::new();
            for depth in 1..80 {
                body.push_str(&format!("{}if true:\n", "    ".repeat(depth)));
            }
            body.push_str(&format!("{}pass", "    ".repeat(80)));
            let explicit = format!("if false: pass\nelse if true:\n{body}");
            let (actual, _) = direct_counterpart(&explicit);
            let errors = [
                Runtime::check(&actual).unwrap_err(),
                Runtime::check(&explicit).unwrap_err(),
            ];
            assert_eq!(errors[0], errors[1]);
            assert_eq!(errors[0].message(), "Expressions are nested too deeply");
        })
        .unwrap()
        .join()
        .unwrap();
}

fn stdin(source: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(source).unwrap();
    child.wait_with_output().unwrap()
}

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

#[test]
fn native_stdin_preserves_utf8_nul_f64_framing_and_initial_if_buffering_at_eof() {
    for newline in ["\n", "\r\n"] {
        for ending in ["", "\n", "\n\n"] {
            for literal in [r#""café😀\0""#, "\"café😀\0\""] {
                let explicit = format!("if false:\n\tresult = \"hold\"\nelse if true:\n\tresult = {literal}\nelse:\n\tresult = \"wait\"\nresult{ending}").replace('\n', newline);
                let (actual, _) = direct_counterpart(&explicit);
                for source in [&actual, &explicit] {
                    let output = stdin(source.as_bytes());
                    assert!(output.status.success(), "{output:?}");
                    assert_eq!(output.stderr, b"");
                    assert_eq!(
                        output.stdout,
                        format!("{BANNER}\"café😀\\u0000\"\n").as_bytes()
                    );
                }
            }
        }
    }
    let output = stdin(
        b"if false:\n    result = 1\nelif true:\n    result = 2\nelse\n    result = 3\nresult\n\n7",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stderr, b"");
    assert_eq!(output.stdout, format!("{BANNER}2.0\n7.0\n").as_bytes());
}

#[test]
fn native_files_keep_json_bytes_physical_eof_error_positions_and_no_banner() {
    let directory = std::env::temp_dir().join(format!(
        "nucleoid-direct-alternatives-{}",
        std::process::id()
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("program.nuc");
    for newline in ["\n", "\r\n"] {
        for ending in ["", "\n"] {
            let explicit = format!("if false:\n    result = \"hold\"\nelse if true:\n    result = \"café😀\\0\"\nelse:\n    result = \"wait\"\nresult{ending}").replace('\n', newline);
            let (actual, _) = direct_counterpart(&explicit);
            for source in [&actual, &explicit] {
                fs::write(&path, source).unwrap();
                let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
                    .arg("--json")
                    .arg(&path)
                    .output()
                    .unwrap();
                assert!(output.status.success(), "{output:?}");
                assert_eq!(output.stderr, b"");
                assert_eq!(output.stdout, "\"café😀\\u0000\"\n".as_bytes());
            }
        }
    }
    for (source, message) in [
        (
            "if false: pass\nelse",
            "SyntaxError: Expected ':' but found end of input (line 2, column 5)",
        ),
        (
            "if false: pass\nelif",
            "SyntaxError: Expected ':' but found end of input (line 2, column 5)",
        ),
        (
            "if false: pass\nelse ready: pass",
            "SyntaxError: Expected ':' but found identifier 'ready' (line 2, column 6)",
        ),
        (
            "if false: pass\nelif true:\n    value = (1 +)",
            "SyntaxError: Unexpected ')' (line 3, column 18)",
        ),
        (
            "if false: pass\nelse true:\n    missing",
            "ReferenceError: missing is not defined (line 1, column 1)",
        ),
        (
            "if false: pass\nelse true:\n    \0",
            "SyntaxError: Unexpected character '\0' (line 3, column 5)",
        ),
    ] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg("--json")
            .arg(&path)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{output:?}");
        assert_eq!(output.stdout, b"");
        assert_eq!(output.stderr, format!("{message}\n").as_bytes());
    }
    fs::remove_file(path).unwrap();
    fs::remove_dir(directory).unwrap();
}

#[test]
fn native_stdin_keeps_inherited_appended_newline_diagnostics_and_recovers_after_error() {
    let output = stdin(b"saved = 7\nif false:\n    pass\nelse ready:;\n    saved = 1\n\nsaved\nif false:\n    pass\nelif true:\n    saved = 9\n\nsaved");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stderr,
        b"SyntaxError: Expected ':' but found identifier 'ready' (line 3, column 6)\n"
    );
    assert_eq!(
        output.stdout,
        format!("{BANNER}7.0\n7.0\n9.0\n9.0\n").as_bytes()
    );
    let output = stdin(b"if false:\n    pass\nelse");
    assert!(output.status.success(), "{output:?}");
    // The REPL's inherited newline differs intentionally from literal file EOF.
    assert_eq!(
        output.stderr,
        b"SyntaxError: Expected ':' but found end of line (line 3, column 1)\n"
    );
    assert_eq!(output.stdout, BANNER.as_bytes());
}
