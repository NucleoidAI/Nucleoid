mod common;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate;
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_expression};
use nucleoid::nuc::Nuc;
use nucleoid::reasoning::Stage;
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde_json::{Value as Json, json};

const SPEC_TITLES: [&str; 10] = [
    "Nucleoid narrows an explanation to the paths from one source",
    "Nucleoid narrows an impact to the paths toward one target",
    "Nucleoid keeps every path through a diamond in walk order",
    "Nucleoid selects no step until a path reaches the target",
    "Nucleoid keeps path selections live as rules and targets change",
    "Nucleoid finds derivation paths in class level rules",
    "Nucleoid composes path selections with state filters",
    "Nucleoid uses path selections in functions and quantifiers",
    "Nucleoid keeps path an ordinary name beside path stages",
    "Nucleoid restores path selections after failed changes and hypotheticals",
];

const REFERENCE_TITLE: &str = "Nucleoid explains how one value reaches another with path";
const README_TITLE: &str = "Path shows how a discount tier reaches an order total";
const EXAMPLES_TITLE: &str = "Paths show which steps a late supplier pushes back";

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

const CHAIN: &str = "a = 1\nb = a + 2\nc = b * 2\n";

const ORDER: &str =
    "price = 10\nqty = 3\nshipping = 5\nsubtotal = price * qty\ntotal = subtotal + shipping\n";

const SHORTCUT: &str = "base = 1\nu = base + 1\ns = u + 1\nq = s * 2\np = u * 3\nt = p + q\n";

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "derivation path regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
    value
}

fn json_of(runtime: &Runtime, value: &Value) -> Json {
    serde_json::from_str(&runtime.serialize_json(value).unwrap()).unwrap()
}

fn evaluate(runtime: &mut Runtime, source: &str) -> Json {
    let value = checked(runtime, source);
    json_of(runtime, &value)
}

fn result(source: &str) -> Json {
    evaluate(&mut Runtime::new(), source)
}

fn nodes(runtime: &mut Runtime, selection: &str) -> Json {
    evaluate(runtime, &format!("({selection}).map(step -> step.node)"))
}

fn failure(source: &str) -> (ErrorKind, String, Option<Position>) {
    failure_in(&mut Runtime::new(), source)
}

fn failure_in(runtime: &mut Runtime, source: &str) -> (ErrorKind, String, Option<Position>) {
    let error = runtime.run(source).expect_err("the program must fail");
    (error.kind(), error.message(), error.position())
}

fn error(
    kind: ErrorKind,
    message: &str,
    line: usize,
    column: usize,
) -> (ErrorKind, String, Option<Position>) {
    (kind, message.to_string(), Some(Position::new(line, column)))
}

fn unpositioned(kind: ErrorKind, message: &str) -> (ErrorKind, String, Option<Position>) {
    (kind, message.to_string(), None)
}

fn paths(source: &str) -> usize {
    let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    format!("{statements:?}").matches("Path { source").count()
}

fn value_of(source: &str) -> Expr {
    match parse(source).unwrap().pop() {
        Some(Stmt::Assign { value, .. }) => value,
        other => panic!("{source} ends with {other:?}"),
    }
}

fn stored(runtime: &Runtime, name: &str) -> String {
    match runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .and_then(|node| node.node.as_ref())
    {
        Some(Nuc::Variable(variable)) => variable.value.to_string(),
        other => panic!("{name} is {other:?}"),
    }
}

fn graph_line(runtime: &Runtime, name: &str) -> String {
    let key = NodeKey::variable(name);
    let node = runtime.graph.retrieve(&key).unwrap();
    let mut reads: Vec<String> = node.dependencies.iter().map(ToString::to_string).collect();
    reads.sort();
    let mut observes: Vec<String> = runtime
        .graph
        .shape_dependencies(&key)
        .map(ToString::to_string)
        .collect();
    observes.sort();
    format!("{name} reads {reads:?} observes {observes:?}")
}

fn identifier(name: &str) -> Box<Expr> {
    Box::new(Expr::Identifier(name.to_string()))
}

fn reason(stage: Stage, source: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Reason { stage, source })
}

fn path(source: Box<Expr>, target: Box<Expr>) -> Expr {
    Expr::Path { source, target }
}

fn cli(label: &str, source: &str) -> Output {
    let directory =
        std::env::temp_dir().join(format!("nucleoid-nuc88-{label}-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let file = directory.join("program.nuc");
    fs::write(&file, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(&file)
        .output()
        .unwrap();
    fs::remove_dir_all(&directory).unwrap();
    output
}

fn repl(source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--plain", "--json"])
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

fn documents() -> [Vec<common::Case>; 4] {
    [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ]
}

fn is_appended(title: &str) -> bool {
    SPEC_TITLES.contains(&title) || [REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE].contains(&title)
}

fn graph(seed: u64, size: usize) -> String {
    let mut state = seed;
    let mut source = String::from("n0 = 1\n");
    for index in 1..size {
        let mut reads = Vec::new();
        for earlier in 0..index {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            if (state >> 33) % 4 == 0 {
                reads.push(format!("n{earlier}"));
            }
        }
        if reads.is_empty() {
            source.push_str(&format!("n{index} = {index}\n"));
        } else {
            source.push_str(&format!("n{index} = {} + 1\n", reads.join(" + ")));
        }
    }
    source
}

#[test]
fn appended_document_cases_run_every_assertion() {
    let [spec, reference, readme, examples] = documents();
    let appended = SPEC_TITLES
        .iter()
        .map(|title| (&spec, *title, None))
        .chain([
            (&reference, REFERENCE_TITLE, Some("[41,36,12]")),
            (&readme, README_TITLE, None),
            (&examples, EXAMPLES_TITLE, Some("[11,15,19]")),
        ]);

    let mut assertions = 0;
    for (cases, title, expected) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(paths(&case.source) > 0, "{title} writes a path stage");
        assert_eq!(case.expected.as_deref(), expected, "{title}");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 62);
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Derivation path selections\n\n```\n")
        .expect("the language reference documents path stages");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 88](nuc-0088.md)"));
}

#[test]
fn no_earlier_document_case_writes_a_path_stage() {
    let mut cases = 0;
    let mut with_path = 0;
    for case in documents().iter().flatten() {
        cases += 1;
        let uses = paths(&case.source) > 0;
        assert_eq!(uses, is_appended(&case.title), "{}", case.title);
        with_path += usize::from(uses);
    }
    assert_eq!(with_path, 13);
    assert_eq!(cases, 1072);
}

#[test]
fn path_stages_parse_into_their_own_variant() {
    let tree = path(reason(Stage::Why, identifier("c")), identifier("a"));
    assert_eq!(value_of("x = c |> why |> path(a)"), tree);
    assert_eq!(value_of("x = c |> why |> path (a)"), tree);
    assert_eq!(value_of("x = c|>why|>path(a)"), tree);
    assert_eq!(value_of("x = (c |> why |> path(\n    a\n))"), tree);
    assert_eq!(value_of("x = why c |> path(a)"), tree);
    assert_eq!(parse_expression("c |> why |> path(a)").unwrap(), tree);
    assert_eq!(
        value_of("x = model |> path(a)"),
        path(Box::new(Expr::Model), identifier("a"))
    );
    assert_eq!(
        value_of("x = c |> why |> path(a) |> stated"),
        *reason(Stage::Stated, Box::new(tree.clone()))
    );
    assert_eq!(
        value_of("x = c |> affects |> path(o.p)"),
        path(
            reason(Stage::Affects, identifier("c")),
            Box::new(Expr::Member {
                object: identifier("o"),
                property: "p".to_string(),
            })
        )
    );
    assert_eq!(
        value_of("x = c |> why |> depth(1)"),
        Expr::Depth {
            source: reason(Stage::Why, identifier("c")),
            limit: Box::new(Expr::Number(1.0)),
        }
    );
    assert_eq!(
        value_of("x = path(c, a)"),
        Expr::Call {
            callee: identifier("path"),
            arguments: vec![
                Expr::Identifier("c".to_string()),
                Expr::Identifier("a".to_string()),
            ],
        }
    );
    assert_eq!(parse("path c").unwrap().len(), 2);
    assert_eq!(Stage::from_name("path"), None);
    assert_eq!(paths("x = c |> why |> depth(1)\ny = path(c)"), 0);
}

#[test]
fn ordinary_names_called_path_keep_their_meaning() {
    assert_eq!(result("path = 2\npath + 1"), json!(3.0));
    assert_eq!(result("def path(x, y) => y\npath(4, 1)"), json!(1.0));
    assert_eq!(result("x = {path: 1}\nx.path"), json!(1.0));
    assert_eq!(result("o = {}\no.path = 3\no.path"), json!(3.0));
    assert_eq!(result("[1, 2].map(path -> path * 2)"), json!([2.0, 4.0]));
    assert_eq!(
        result(&format!(
            "{CHAIN}class Route:\n    def path() => 7\ndef hops(path) => path + 1\nx = {{path: 1}}\nr = Route()\n[(c |> why |> path(a)).length, r.path(), hops(1), x.path]"
        )),
        json!([3.0, 7.0, 2.0, 1.0])
    );
    assert_eq!(
        result(
            "class Route(stops):\n    this.stops = stops\n    def path() => this.stops.length\n    def via() => (this.legs |> why |> path(this.stops)).map(s -> s.node)\ntrip = Route([1, 2])\ntrip.legs = trip.stops.length\n[trip.via(), trip.path()]"
        ),
        json!([["trip.legs", "trip.stops"], 2.0])
    );
}

#[test]
fn explanation_paths_keep_the_steps_that_affects_of_the_target_selects() {
    for (seed, size) in [(1, 12), (7, 12), (42, 10), (2026, 14)] {
        let mut runtime = Runtime::new();
        checked(&mut runtime, &graph(seed, size));
        for source in 0..size {
            for target in 0..size {
                assert_eq!(
                    nodes(
                        &mut runtime,
                        &format!("n{source} |> why |> path(n{target})")
                    ),
                    nodes(
                        &mut runtime,
                        &format!(
                            "(why n{source}).filter(s -> s.node == \"n{target}\" || any d in affects n{target}: d.node == s.node)"
                        )
                    ),
                    "seed {seed}: n{source} to n{target}"
                );
            }
        }
    }
}

#[test]
fn impact_paths_keep_the_steps_that_why_of_the_target_selects() {
    for (seed, size) in [(3, 12), (11, 12), (99, 10), (2027, 14)] {
        let mut runtime = Runtime::new();
        checked(&mut runtime, &graph(seed, size));
        for source in 0..size {
            for target in 0..size {
                assert_eq!(
                    nodes(
                        &mut runtime,
                        &format!("n{source} |> affects |> path(n{target})")
                    ),
                    nodes(
                        &mut runtime,
                        &format!(
                            "(affects n{source}).filter(s -> any d in why n{target}: d.node == s.node)"
                        )
                    ),
                    "seed {seed}: n{source} to n{target}"
                );
            }
        }
    }
}

#[test]
fn deleted_nodes_and_observers_keep_the_definitional_equivalence() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{}view = why n5\nn12 = view.length + n9\n", graph(5, 12)),
    );
    checked(&mut runtime, "delete n2");
    for source in ["n12", "n11", "n9", "n5"] {
        for target in ["n0", "n2", "n3", "n5", "n9", "view"] {
            assert_eq!(
                nodes(&mut runtime, &format!("{source} |> why |> path({target})")),
                nodes(
                    &mut runtime,
                    &format!(
                        "(why {source}).filter(s -> s.node == \"{target}\" || any d in affects {target}: d.node == s.node)"
                    )
                ),
                "{source} to {target}"
            );
            assert_eq!(
                nodes(
                    &mut runtime,
                    &format!("{target} |> affects |> path({source})")
                ),
                nodes(
                    &mut runtime,
                    &format!(
                        "(affects {target}).filter(s -> any d in why {source}: d.node == s.node)"
                    )
                ),
                "{target} to {source}"
            );
        }
    }
    assert_eq!(nodes(&mut runtime, "n12 |> why |> path(view)"), json!([]));
    assert_eq!(nodes(&mut runtime, "n12 |> why |> path(n2)"), json!([]));
}

#[test]
fn selections_that_no_walk_produced_use_the_direction_of_why() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, ORDER);
    assert_eq!(
        nodes(&mut runtime, "price |> path(price)"),
        json!(["price"])
    );
    assert_eq!(
        nodes(&mut runtime, "total |> path(price)"),
        json!(["total"])
    );
    assert_eq!(nodes(&mut runtime, "price |> path(total)"), json!([]));
    assert_eq!(
        nodes(&mut runtime, "model |> path(price)"),
        json!(["price", "subtotal", "total"])
    );
    assert_eq!(
        nodes(&mut runtime, "model |> path(shipping)"),
        json!(["shipping", "total"])
    );
    assert_eq!(
        nodes(&mut runtime, "model |> stated |> path(price)"),
        json!(["price"])
    );
    assert_eq!(
        nodes(&mut runtime, "model |> derived |> path(price)"),
        json!(["subtotal", "total"])
    );
    assert_eq!(
        nodes(&mut runtime, "stated price |> path(price)"),
        json!(["price"])
    );
    assert_eq!(
        nodes(&mut runtime, "derived total |> path(qty)"),
        json!(["total"])
    );
    assert_eq!(
        nodes(&mut runtime, "total |> why |> path(total)"),
        json!(["total"])
    );
    assert_eq!(
        nodes(&mut runtime, "price |> affects |> path(price)"),
        json!([])
    );
    assert_eq!(
        nodes(&mut runtime, "model |> depth(0) |> path(price)"),
        json!(["price", "subtotal", "total"])
    );
    assert_eq!(
        nodes(&mut runtime, "total |> path(price) |> depth(0)"),
        json!(["total"])
    );
}

#[test]
fn every_path_through_a_diamond_is_kept_in_walk_order() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, SHORTCUT);
    assert_eq!(
        nodes(&mut runtime, "why t"),
        json!(["t", "p", "q", "s", "u", "base"])
    );
    assert_eq!(
        nodes(&mut runtime, "t |> why |> path(u)"),
        json!(["t", "p", "q", "s", "u"])
    );
    assert_eq!(
        nodes(&mut runtime, "t |> why |> path(s)"),
        json!(["t", "q", "s"])
    );
    assert_eq!(
        nodes(&mut runtime, "t |> why |> path(p)"),
        json!(["t", "p"])
    );
    assert_eq!(
        nodes(&mut runtime, "t |> why |> path(base)"),
        json!(["t", "p", "q", "s", "u", "base"])
    );
    assert_eq!(
        nodes(&mut runtime, "affects base"),
        json!(["u", "s", "p", "t", "q"])
    );
    assert_eq!(
        nodes(&mut runtime, "base |> affects |> path(t)"),
        json!(["u", "s", "p", "t", "q"])
    );
    assert_eq!(
        nodes(&mut runtime, "base |> affects |> path(q)"),
        json!(["u", "s", "q"])
    );
    assert_eq!(nodes(&mut runtime, "u |> affects |> path(p)"), json!(["p"]));
    assert_eq!(
        evaluate(&mut runtime, "(t |> why |> path(s))[0].from"),
        json!(["p", "q"])
    );
}

#[test]
fn filters_keep_the_direction_and_later_walks_set_their_own() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, SHORTCUT);
    assert_eq!(
        nodes(&mut runtime, "base |> affects |> derived |> path(q)"),
        json!(["u", "s", "q"])
    );
    assert_eq!(
        nodes(&mut runtime, "base |> affects |> path(q) |> derived"),
        json!(["u", "s", "q"])
    );
    assert_eq!(
        nodes(&mut runtime, "t |> why |> stated |> path(base)"),
        json!(["base"])
    );
    assert_eq!(
        nodes(&mut runtime, "t |> why |> path(base) |> stated"),
        json!(["base"])
    );
    assert_eq!(
        nodes(&mut runtime, "base |> affects |> path(q) |> why |> path(s)"),
        json!(["s", "q"])
    );
    assert_eq!(
        nodes(&mut runtime, "t |> why |> path(s) |> path(u)"),
        json!(["t", "q", "s"])
    );
    assert_eq!(
        nodes(&mut runtime, "t |> why |> path(s) |> path(p)"),
        json!(["t"])
    );
}

#[test]
fn depth_distances_survive_path_stages() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, SHORTCUT);
    for limit in 0..4 {
        assert_eq!(
            nodes(
                &mut runtime,
                &format!("t |> why |> path(u) |> depth({limit})")
            ),
            nodes(
                &mut runtime,
                &format!("t |> why |> depth({limit}) |> path(u)")
            ),
            "limit {limit}"
        );
    }
    assert_eq!(
        nodes(&mut runtime, "t |> why |> path(u) |> depth(1)"),
        json!(["t", "p", "q"])
    );
    assert_eq!(
        nodes(&mut runtime, "base |> affects |> path(t) |> depth(1)"),
        json!(["u"])
    );
    assert_eq!(
        nodes(&mut runtime, "base |> affects |> depth(2) |> path(q)"),
        json!(["u", "s"])
    );
    assert_eq!(
        evaluate(&mut runtime, "x = t |> why |> path(u) |> depth(null)\nx"),
        json!(null)
    );
}

#[test]
fn path_and_diff_compose_inside_suppose() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, "a = 1\nb = a + 2\nd = 4\nc = b * d\n");
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose d = 5: [(c |> why |> path(a) |> diff).map(s -> s.node), (c |> why |> diff |> path(a)).map(s -> s.node), (model |> diff |> path(d)).map(s -> s.node)]"
        ),
        json!([["c"], ["c"], ["d", "c"]])
    );
    assert_eq!(
        failure(&format!("{CHAIN}c |> why |> path(a) |> diff")),
        error(ErrorKind::Type, "Cannot use 'diff' outside suppose", 4, 1)
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose a = 2: [(a |> affects |> diff |> path(c)).map(s -> s.node), (a |> affects |> diff |> depth(1) |> path(c)).map(s -> s.node)]"
        ),
        json!([["b", "c"], ["b"]])
    );
}

#[test]
fn targets_resolve_as_pipeline_sources_do() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Order(qty, price):\n    this.qty = qty\n    this.price = price\n    def priced() => this.total |> why |> path(this.price)\n$Order.total = $Order.qty * $Order.price + 1\n$Order.via = $Order.total |> why |> path($Order.qty)\npen = Order(2, 3)\ncup = Order(4, 5)\ndef hops(n) => n + 1\nlegs = hops(pen.qty)",
    );
    assert_eq!(
        nodes(&mut runtime, "pen.total |> why |> path(pen.price)"),
        json!(["pen.total", "pen.price"])
    );
    assert_eq!(
        nodes(&mut runtime, "pen.priced()"),
        json!(["pen.total", "pen.price"])
    );
    assert_eq!(
        nodes(&mut runtime, "pen.via"),
        json!(["pen.total", "pen.qty"])
    );
    assert_eq!(
        nodes(&mut runtime, "cup.via"),
        json!(["cup.total", "cup.qty"])
    );
    assert_eq!(
        nodes(&mut runtime, "legs |> why |> path(hops)"),
        json!(["legs", "hops"])
    );
    assert_eq!(
        nodes(&mut runtime, "legs |> why |> path(pen.qty)"),
        json!(["legs", "pen.qty"])
    );
    assert_eq!(
        nodes(&mut runtime, "pen |> why |> path(pen)"),
        json!(["pen"])
    );
    assert_eq!(
        evaluate(&mut runtime, "[pen.priced()[1].holds, cup.via[0].holds]"),
        json!([3.0, 21.0])
    );
}

#[test]
fn invalid_targets_keep_the_diagnostics_of_a_pipeline_source() {
    for (line, expected) in [
        (
            "c |> why |> path(1)",
            error(ErrorKind::Type, "Cannot explain 1", 4, 1),
        ),
        (
            "c |> why |> path(\"a\")",
            error(ErrorKind::Type, "Cannot explain \"a\"", 4, 1),
        ),
        (
            "c |> why |> path(a + 1)",
            error(ErrorKind::Type, "Cannot explain a+1", 4, 1),
        ),
        (
            "c |> why |> path(a.x)",
            error(ErrorKind::Type, "Cannot explain a.x", 4, 1),
        ),
        (
            "c |> why |> path(this)",
            error(ErrorKind::Type, "Cannot explain this", 4, 1),
        ),
        (
            "c |> why |> path([a])",
            error(ErrorKind::Type, "Cannot explain [a]", 4, 1),
        ),
        (
            "c |> why |> path(a |> why)",
            error(ErrorKind::Type, "Cannot explain a|>why", 4, 1),
        ),
        (
            "c |> why |> path(why a)",
            error(ErrorKind::Type, "Cannot explain a|>why", 4, 1),
        ),
        (
            "c |> why |> path(a |> why |> path(a))",
            error(ErrorKind::Type, "Cannot explain a|>why|>path(a)", 4, 1),
        ),
        (
            "class K\nc |> why |> path($K.p)",
            error(ErrorKind::Type, "Cannot explain $K.p", 5, 1),
        ),
        (
            "class K\nc |> why |> path($K)",
            error(ErrorKind::Type, "Cannot explain $K", 5, 1),
        ),
        (
            "c |> why |> path(ghost)",
            error(ErrorKind::Reference, "ghost is not defined", 4, 1),
        ),
        (
            "c |> why |> path(ghost.x)",
            error(ErrorKind::Reference, "ghost is not defined", 4, 1),
        ),
        (
            "c |> why |> path(model)",
            error(ErrorKind::Reference, "model is not defined", 4, 1),
        ),
        (
            "missing |> path(ghost)",
            error(ErrorKind::Reference, "missing is not defined", 4, 1),
        ),
        (
            "1 |> path(ghost)",
            error(ErrorKind::Type, "Cannot explain 1", 4, 1),
        ),
        (
            "c |> why |> depth(-1) |> path(ghost)",
            error(ErrorKind::Type, "-1 is not a non-negative integer", 4, 1),
        ),
        (
            "c |> why |> path(ghost) |> depth(-1)",
            error(ErrorKind::Reference, "ghost is not defined", 4, 1),
        ),
    ] {
        assert_eq!(failure(&format!("{CHAIN}{line}")), expected, "{line}");
    }
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{CHAIN}x = 5"));
    assert_eq!(
        failure_in(&mut runtime, "x = c |> why |> path(ghost)"),
        error(ErrorKind::Reference, "ghost is not defined", 1, 1)
    );
    assert_eq!(evaluate(&mut runtime, "x"), json!(5.0));
    assert_eq!(
        evaluate(
            &mut runtime,
            "late = c |> why |> depth(null) |> path(ghost)\nlate"
        ),
        json!(null)
    );
    checked(&mut runtime, "model = 4\nc = b + model");
    assert_eq!(
        nodes(&mut runtime, "c |> why |> path(model)"),
        json!(["c", "model"])
    );
}

#[test]
fn zero_or_two_arguments_report_the_errors_of_parentheses() {
    for (line, expected) in [
        (
            "c |> why |> path()",
            error(ErrorKind::Syntax, "Unexpected ')'", 4, 19),
        ),
        (
            "c |> why |> path(a, b)",
            error(ErrorKind::Syntax, "Expected ')' but found ','", 4, 19),
        ),
        (
            "c |> why |> path(a,)",
            error(ErrorKind::Syntax, "Expected ')' but found ','", 4, 19),
        ),
        (
            "c |> why |> path(a",
            error(
                ErrorKind::Syntax,
                "Expected ')' but found end of input",
                4,
                19,
            ),
        ),
        (
            "c |> why |> path(",
            error(ErrorKind::Syntax, "Unexpected end of input", 4, 18),
        ),
    ] {
        assert_eq!(failure(&format!("{CHAIN}{line}")), expected, "{line}");
    }
}

#[test]
fn targets_outside_the_model_select_nothing_until_they_are_defined() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{CHAIN}class Box\nbox = Box()\nfar = c |> why |> path(box.size)\nview = why a\nseen = c |> why |> path(view)\nnear = c |> why |> path(a)"
        ),
    );
    assert_eq!(evaluate(&mut runtime, "[far, seen]"), json!([[], []]));
    checked(&mut runtime, "box.size = 2");
    assert_eq!(evaluate(&mut runtime, "far"), json!([]));
    checked(&mut runtime, "a = box.size + 1");
    assert_eq!(
        nodes(&mut runtime, "far"),
        json!(["c", "b", "a", "box.size"])
    );
    checked(&mut runtime, "delete box.size");
    assert_eq!(evaluate(&mut runtime, "far"), json!([]));
    checked(&mut runtime, "box.size = 4");
    assert_eq!(
        evaluate(&mut runtime, "far.map(s -> s.holds)"),
        json!([14.0, 7.0, 5.0, 4.0])
    );
    checked(&mut runtime, "b = a + view.length");
    assert_eq!(evaluate(&mut runtime, "seen"), json!([]));
    assert_eq!(nodes(&mut runtime, "near"), json!(["c", "b", "a"]));
    checked(&mut runtime, "view = 3");
    assert_eq!(nodes(&mut runtime, "seen"), json!(["c", "b", "view"]));
    let mut replaced = Runtime::new();
    checked(
        &mut replaced,
        "a = 1\nview = why a\nflag = 1 if view else 1\nheld = flag |> path(view)",
    );
    assert_eq!(evaluate(&mut replaced, "held"), json!([]));
    checked(&mut replaced, "view = 3");
    assert_eq!(
        evaluate(&mut replaced, "[held.map(s -> s.node), flag]"),
        json!([["flag"], 1.0])
    );
}

#[test]
fn every_binding_named_path_shadows_the_stage() {
    let bindings = [
        ("path = 1\nc |> why |> path(a)", 5),
        ("def path(x) => x\nc |> why |> path(a)", 5),
        ("class path\nc |> why |> path(a)", 5),
        ("path = 1\nmissing |> why |> path(a)", 5),
        ("path = 1\n1 |> path(a)", 5),
        ("def f(path) => c |> why |> path(a)\nf(1)", 5),
        ("def g path => c |> why |> path(a)\ng(1)", 5),
        ("f = path -> c |> why |> path(a)\nf(1)", 5),
        ("f = (path) => c |> why |> path(a)\nf(1)", 5),
        ("f = lambda path: c |> why |> path(a)\nf(1)", 5),
        ("[1].map(path -> c |> why |> path(a))", 4),
        ("for path of \"x\":\n    y = c |> why |> path(a)", 4),
        ("ok = all path in [1]: (c |> why |> path(a)).length > 0", 4),
        ("{\n    path = 5\n    y = c |> why |> path(a)\n}", 4),
        (
            "class R:\n    def m(path) => c |> why |> path(a)\nR().m(1)",
            6,
        ),
        (
            "class Q(path):\n    this.v = (c |> why |> path(a)).length\nQ(1)",
            6,
        ),
        (
            "try:\n    throw 1\ncatch path:\n    y = c |> why |> path(a)",
            4,
        ),
        ("{path} = {path: 1}\nc |> why |> path(a)", 5),
    ];
    for (binding, line) in bindings {
        assert_eq!(
            failure(&format!("{CHAIN}{binding}")),
            error(ErrorKind::Type, "path is not an operation", line, 1),
            "{binding}"
        );
    }
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\npath = 2\nc |> why |> path(a) |> stated"
        )),
        error(ErrorKind::Type, "stated is not an operation", 6, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\npath = 2\nc |> why |> stated |> path(a)"
        )),
        error(ErrorKind::Type, "path is not an operation", 6, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}depth = 1\nc |> why |> path(a) |> depth(1)"
        )),
        error(ErrorKind::Type, "depth is not an operation", 5, 1)
    );
}

#[test]
fn a_binding_declared_after_a_standing_selection_fails_its_refresh() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}link = c |> why |> path(a)\npath = 5"),
    );
    assert_eq!(
        failure_in(&mut runtime, "a = 3"),
        error(ErrorKind::Type, "path is not an operation", 1, 1)
    );
    assert_eq!(evaluate(&mut runtime, "a"), json!(1.0));
    checked(&mut runtime, "delete path\na = 3");
    assert_eq!(
        evaluate(&mut runtime, "link.map(s -> s.holds)"),
        json!([10.0, 5.0, 3.0])
    );
}

#[test]
fn why_and_affects_keep_ignoring_bindings_and_other_stages_are_unchanged() {
    assert_eq!(
        result(&format!(
            "{CHAIN}path = 1\n[(why c).length, (affects a).length, (c |> why |> depth(1)).length, (c |> why |> stated).length]"
        )),
        json!([3.0, 2.0, 2.0, 1.0])
    );
    assert_eq!(
        result(&format!("{CHAIN}why = 1\n(c |> why |> path(a)).length")),
        json!(3.0)
    );
}

#[test]
fn standing_paths_follow_their_source_and_target_structure() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{SHORTCUT}z = 9\nlink = t |> why |> path(s)\nback = base |> affects |> path(q)\nlone = q |> path(u)\nfar = q |> why |> path(z)"
        ),
    );
    assert_eq!(nodes(&mut runtime, "link"), json!(["t", "q", "s"]));
    assert_eq!(nodes(&mut runtime, "back"), json!(["u", "s", "q"]));
    assert_eq!(nodes(&mut runtime, "lone"), json!(["q"]));
    assert_eq!(
        graph_line(&runtime, "lone"),
        r#"lone reads ["q"] observes ["base", "q", "s", "u"]"#
    );
    assert_eq!(
        graph_line(&runtime, "far"),
        r#"far reads ["base", "q", "s", "u"] observes ["base", "q", "s", "u", "z"]"#
    );
    checked(&mut runtime, "p = s * 3");
    assert_eq!(nodes(&mut runtime, "link"), json!(["t", "p", "q", "s"]));
    checked(&mut runtime, "s = 2");
    assert_eq!(evaluate(&mut runtime, "[lone, back]"), json!([[], []]));
    assert_eq!(nodes(&mut runtime, "link"), json!(["t", "p", "q", "s"]));
    checked(&mut runtime, "delete s");
    assert_eq!(evaluate(&mut runtime, "link"), json!([]));
    checked(&mut runtime, "s = u + 5");
    assert_eq!(nodes(&mut runtime, "link"), json!(["t", "p", "q", "s"]));
    assert_eq!(nodes(&mut runtime, "lone"), json!(["q"]));
    assert_eq!(nodes(&mut runtime, "back"), json!(["u", "s", "q"]));
    checked(&mut runtime, "s = z + 1");
    assert_eq!(nodes(&mut runtime, "far"), json!(["q", "s", "z"]));
    assert_eq!(
        evaluate(&mut runtime, "far.map(step -> step.holds)"),
        json!([20.0, 10.0, 9.0])
    );
    assert_eq!(
        nodes(&mut runtime, "model |> why"),
        json!(["base", "u", "s", "q", "p", "t", "z"])
    );
}

#[test]
fn class_level_rewiring_refreshes_every_instance() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Item(price, qty):\n    this.price = price\n    this.qty = qty\n$Item.subtotal = $Item.price * $Item.qty\n$Item.total = $Item.subtotal + 1\n$Item.via = $Item.total |> why |> path($Item.price)\npen = Item(2, 5)\ncup = Item(3, 1)",
    );
    assert_eq!(
        evaluate(&mut runtime, "[pen.via.length, cup.via.length]"),
        json!([3.0, 3.0])
    );
    checked(&mut runtime, "$Item.total = $Item.qty + 1");
    assert_eq!(
        evaluate(&mut runtime, "[pen.via, cup.via]"),
        json!([[], []])
    );
    checked(&mut runtime, "$Item.total = $Item.price * 2");
    assert_eq!(
        nodes(&mut runtime, "pen.via"),
        json!(["pen.total", "pen.price"])
    );
    checked(&mut runtime, "jar = Item(7, 1)");
    assert_eq!(
        evaluate(&mut runtime, "jar.via.map(s -> s.holds)"),
        json!([14.0, 7.0])
    );
}

#[test]
fn failed_changes_roll_back_path_selections() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}z = 9\nlink = c |> why |> path(z)"),
    );
    checked(
        &mut runtime,
        "try:\n    a = z + 1\n    assert(link.map(s -> s.node), [\"c\", \"b\", \"a\", \"z\"])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
    );
    assert_eq!(evaluate(&mut runtime, "[link, a]"), json!([[], 1.0]));
    assert_eq!(
        evaluate(
            &mut runtime,
            "plan = suppose a = z: link.map(s -> s.holds)\n[plan, link]"
        ),
        json!([[22.0, 11.0, 9.0, 9.0], []])
    );
    assert_eq!(
        failure_in(&mut runtime, "a = z + 1\nb = ghost"),
        error(ErrorKind::Reference, "ghost is not defined", 2, 1)
    );
    assert_eq!(evaluate(&mut runtime, "[link, a]"), json!([[], 1.0]));
    checked(&mut runtime, "a = z + 1");
    assert_eq!(nodes(&mut runtime, "link"), json!(["c", "b", "a", "z"]));
    checked(&mut runtime, "a = 2");
    assert_eq!(evaluate(&mut runtime, "link"), json!([]));
}

#[test]
fn snapshots_freeze_selections_but_not_targets() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{CHAIN}n = 1\nlink = c |> why |> path(a)\nfrozen = link.value\nkept = (c |> why |> path(a)).value\nnear = c |> why |> path(a) |> depth(n.value)"
        ),
    );
    assert_eq!(stored(&runtime, "link"), "c|>why|>path(a)");
    assert_eq!(stored(&runtime, "near"), "c|>why|>path(a)|>depth(1)");
    checked(&mut runtime, "a = 5\nn = 2");
    assert_eq!(
        evaluate(
            &mut runtime,
            "[link.map(s -> s.holds), frozen.map(s -> s.holds), kept.map(s -> s.holds), near.length]"
        ),
        json!([[14.0, 7.0, 5.0], [6.0, 3.0, 1.0], [6.0, 3.0, 1.0], 2.0])
    );
    assert_eq!(
        failure_in(&mut runtime, "x = c |> why |> path(a.value)"),
        error(ErrorKind::Type, "Cannot explain a.value", 1, 1)
    );
    assert_eq!(
        failure(&format!("{CHAIN}x = why a.value")),
        error(ErrorKind::Type, "Cannot explain a.value", 4, 1)
    );
}

#[test]
fn functions_methods_lambdas_quantifiers_conditions_and_templates_hold_paths() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{CHAIN}def via() => (c |> why |> path(a)).length\nok = all s in c |> why |> path(a): s.holds > 0\nwide = 0\nif (c |> why |> path(a)).length > 2:\n    wide = 1\nnote = `${{(c |> why |> path(a)).length}}`\nclass Gauge(level):\n    this.level = level\n    def trail() => this.total |> why |> path(this.level)\n$Gauge.total = $Gauge.level * 2 + a\n$Gauge.via = $Gauge.total |> why |> path($Gauge.level)\nlow = Gauge(1)\nhigh = Gauge(3)\nlengths = [1, 2].map(v -> (c |> why |> path(a)).length + v)"
        ),
    );
    assert_eq!(
        evaluate(&mut runtime, "[via(), ok, wide, note, lengths]"),
        json!([3.0, true, 1.0, "3", [4.0, 5.0]])
    );
    assert_eq!(
        nodes(&mut runtime, "low.via"),
        json!(["low.total", "low.level"])
    );
    assert_eq!(
        evaluate(&mut runtime, "high.trail().map(s -> s.holds)"),
        json!([7.0, 3.0])
    );
    checked(&mut runtime, "b = 4\na = -9");
    assert_eq!(
        evaluate(&mut runtime, "[via(), ok, note, lengths]"),
        json!([0.0, true, "0", [1.0, 2.0]])
    );
    assert_eq!(
        evaluate(&mut runtime, "high.via.map(s -> s.holds)"),
        json!([-3.0, 3.0])
    );
}

#[test]
fn path_stages_render_once_and_parse_back() {
    for source in [
        "c |> why |> path(a)",
        "why c |> path(a)",
        "model |> path(a) |> stated",
        "a |> affects |> path(c) |> why |> path(a)",
        "c |> why |> path(o.p)",
        "c |> why |> path(a) |> depth(n + 1)",
        "c |> why |> depth(1) |> path(a)",
        "a || b |> path(c)",
    ] {
        let parsed = parse_expression(source).unwrap();
        let rendered = parsed.to_string();
        assert_eq!(
            parse_expression(&rendered).unwrap(),
            parsed,
            "{source} => {rendered}"
        );
    }
    assert_eq!(
        parse_expression("total |> why |> path(price) |> derived")
            .unwrap()
            .to_string(),
        "total|>why|>path(price)|>derived"
    );
    let mut chained = String::from("c");
    for level in 1..=6 {
        chained.push_str(" |> path(a)");
        let rendered = parse_expression(&chained).unwrap().to_string();
        assert_eq!(rendered.len(), 1 + 9 * level, "{rendered}");
    }
    assert_eq!(
        generate(&parse("def f(x = 1) => c |> why |> path(a)").unwrap()[0]),
        "def f(x=1):
    return c|>why|>path(a)"
    );
    let output = cli(
        "keys",
        &format!(
            "{CHAIN}if (c |> why |> path(a)).length > 1:\n    flag = 1\nlink = c |> why |> path(a)\n1"
        ),
    );
    assert!(output.status.success(), "{output:?}");
    let listing = String::from_utf8(output.stdout).unwrap();
    assert!(
        listing.contains("  if(c|>why|>path(a).length>1) [if]\n"),
        "{listing}"
    );
    assert!(
        listing.contains("  link [variable]\n    reads    c, b, a\n    observes c, b, a\n"),
        "{listing}"
    );
}

#[test]
fn path_stages_handle_large_graphs() {
    let size = 20_000;
    let mut source = String::from("v0 = 1\n");
    for index in 1..size {
        source.push_str(&format!("v{index} = v{} + 1\n", index - 1));
    }
    let mut runtime = Runtime::new();
    checked(&mut runtime, &source);
    let last = size - 1;
    assert_eq!(
        evaluate(
            &mut runtime,
            &format!("(v{last} |> why |> path(v0)).length")
        ),
        json!(20_000.0)
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            &format!("(v0 |> affects |> path(v{last})).length")
        ),
        json!(19_999.0)
    );
    assert_eq!(
        nodes(
            &mut runtime,
            &format!("v{last} |> why |> path(v{}) |> depth(5)", last - 2)
        ),
        json!([
            format!("v{last}"),
            format!("v{}", last - 1),
            format!("v{}", last - 2)
        ])
    );
}

#[test]
fn near_misses_keep_their_accepted_results() {
    let parse_time = unpositioned(ErrorKind::Type, "path is not an operation");
    for line in [
        "total |> why |> path",
        "total |> why |> path\n(price)",
        "total |> why |> path[price]",
        "total |> why |> path.x",
        "total |> why |> path{1}",
    ] {
        assert_eq!(failure(&format!("{ORDER}{line}")), parse_time, "{line}");
    }
    for (line, expected) in [
        (
            "total |> why |> Path(price)",
            unpositioned(ErrorKind::Type, "Path is not an operation"),
        ),
        (
            "total |> why |> paths(price)",
            unpositioned(ErrorKind::Type, "paths is not an operation"),
        ),
        (
            "path(total, price)",
            error(ErrorKind::Reference, "path is not defined", 6, 1),
        ),
        (
            "path total",
            error(ErrorKind::Reference, "path is not defined", 6, 1),
        ),
        (
            "why path(price)",
            error(ErrorKind::Type, "Cannot explain path(price)", 6, 1),
        ),
        (
            "total.path(price)",
            error(ErrorKind::Type, "number.path is not a function", 6, 1),
        ),
        (
            "total | > path(price)",
            error(ErrorKind::Syntax, "Unexpected character '|'", 6, 7),
        ),
        (
            "total ||> path(price)",
            error(ErrorKind::Syntax, "Unexpected '>'", 6, 11),
        ),
        (
            "total |> |> path(price)",
            error(ErrorKind::Syntax, "Expected an operation", 6, 10),
        ),
    ] {
        assert_eq!(failure(&format!("{ORDER}{line}")), expected, "{line}");
    }
    assert_eq!(result(&format!("{ORDER}path = 2\npath + 1")), json!(3.0));
    assert_eq!(
        result(&format!("{ORDER}def path(x, y) => y\npath(total, 1)")),
        json!(1.0)
    );
    assert_eq!(
        result(&format!("{ORDER}x = {{path: 1}}\nx.path")),
        json!(1.0)
    );
}

#[test]
fn inputs_that_were_errors_now_give_what_the_proposal_lists() {
    for (line, expected) in [
        (
            "(total |> why |> path(price)).map(s -> s.node)",
            json!(["total", "subtotal", "price"]),
        ),
        ("(total |> why |> path (price)).length", json!(3.0)),
        ("(total|>why|>path(price)).length", json!(3.0)),
        (
            "(price |> affects |> path(total)).map(s -> s.node)",
            json!(["subtotal", "total"]),
        ),
        ("(total |> path(price)).map(s -> s.node)", json!(["total"])),
        (
            "(model |> path(price)).map(s -> s.node)",
            json!(["price", "subtotal", "total"]),
        ),
        ("(price |> affects |> path(price)).length", json!(0.0)),
        ("total |> why |> path(price)(2)", json!(2.0)),
        ("a, b = total |> why |> path(price)\na.node", json!("total")),
        ("{node} = total |> why |> path(price)\nnode", json!(null)),
        ("y = 1 if total |> why |> path(price) else 2\ny", json!(1.0)),
        (
            "x = suppose: (total |> why |> path(price)).length\nx",
            json!(3.0),
        ),
    ] {
        assert_eq!(result(&format!("{ORDER}{line}")), expected, "{line}");
    }
    for (line, expected) in [
        (
            "total |> why |> path()",
            error(ErrorKind::Syntax, "Unexpected ')'", 6, 23),
        ),
        (
            "total |> why |> path(price, qty)",
            error(ErrorKind::Syntax, "Expected ')' but found ','", 6, 27),
        ),
        (
            "total |> why |> path(price).length",
            error(ErrorKind::Syntax, "Unexpected '.'", 6, 29),
        ),
        (
            "total |> why |> path(1)",
            error(ErrorKind::Type, "Cannot explain 1", 6, 1),
        ),
        (
            "total |> why |> path(ghost)",
            error(ErrorKind::Reference, "ghost is not defined", 6, 1),
        ),
        (
            "total |> why |> path(price |> why)",
            error(ErrorKind::Type, "Cannot explain price|>why", 6, 1),
        ),
        (
            "1 |> path(price)",
            error(ErrorKind::Type, "Cannot explain 1", 6, 1),
        ),
        (
            "missing |> path(ghost)",
            error(ErrorKind::Reference, "missing is not defined", 6, 1),
        ),
        (
            "path = 2\ntotal |> why |> path(price)",
            error(ErrorKind::Type, "path is not an operation", 7, 1),
        ),
        (
            "[p] = total |> why |> path(price)",
            error(ErrorKind::Syntax, "Cannot assign to [p]", 6, 1),
        ),
        (
            "delete total |> why |> path(price)",
            error(
                ErrorKind::Syntax,
                "Cannot delete total|>why|>path(price)",
                6,
                1,
            ),
        ),
    ] {
        assert_eq!(failure(&format!("{ORDER}{line}")), expected, "{line}");
    }
}

#[test]
fn tentative_contexts_read_the_stage() {
    assert_eq!(
        result(&format!(
            "{CHAIN}n = 0\nwhile n < (c |> why |> path(a)).length:\n    n += 1\nn"
        )),
        json!(3.0)
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose k = (c |> why |> path(a)).length:\n    assert(k, 3)\nk = 0\nk"
        )),
        json!(0.0)
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}v = \"X\"\nw = (v is X)\nx = (c |> why |> path(a)).length\n[w, x]"
        )),
        json!([true, 3.0])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}t = `${{(c |> why |> path(a)).length}} steps`\nt"
        )),
        json!("3 steps")
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}first, second = c |> why |> path(a)\n[first.node, second.holds]"
        )),
        json!(["c", 3.0])
    );
}

#[test]
fn the_binary_runs_path_stages_from_files_and_the_line_repl() {
    let output = cli(
        "file",
        &format!("{CHAIN}(c |> why |> path(a)).map(s -> s.node)\n"),
    );
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("[\"c\",\"b\",\"a\"]\n")
    );
    let output = repl(&format!(
        "{CHAIN}(c |> why |> path(a)).length\nc |> why |> path(\n(c |> why |> path(c)).length\nc |> why |> path\n"
    ));
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{BANNER}1.0\n3.0\n6.0\n3.0\n1.0\n")
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "SyntaxError: Unexpected end of input (line 2, column 1)\nTypeError: path is not an operation\n"
    );
}
