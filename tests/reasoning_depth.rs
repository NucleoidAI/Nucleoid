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

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid narrows explanations to a reasoning depth",
    "Nucleoid measures reasoning depth along the shortest path",
    "Nucleoid narrows impact selections to direct dependents",
    "Nucleoid keeps depth selections live as limits and rules change",
    "Nucleoid composes depth with state filters and later walks",
    "Nucleoid waits for unknown depth limits",
    "Nucleoid applies depth in class level rules methods and quantifiers",
    "Nucleoid restores depth selections after failed changes",
];

const REFERENCE_TITLE: &str = "Nucleoid limits explanations to direct causes with depth";
const README_TITLE: &str = "Depth keeps an alarm's explanation to its direct causes";
const EXAMPLES_TITLE: &str = "Depth keeps the direct effects of a part's cost in view";

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

const PRELUDE: &str = "a = 1\nc = a + 1\n";

const CHAIN: &str = "a = 1\nb = a + 2\nc = b * 2\n";

const SHORTCUT: &str = "base = 1\nu = base + 1\ns = u + 1\nq = s * 2\np = u * 3\nt = p + q\n";

const PRICES: &str = "price = 10\ntax = price * 2\ntotal = price + tax\nlabel = total * 3\n";

const DETOUR: &str = "w = 1\nm = w + 1\nu = m * 2\nx = u + w\n";

const COUNTER: &str = "calls = []\ndef note(n):\n    calls.push(n)\n    return n\n";

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "reasoning depth regression".to_string(),
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

fn depths(source: &str) -> usize {
    let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    format!("{statements:?}").matches("Depth {").count()
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

fn number(value: f64) -> Box<Expr> {
    Box::new(Expr::Number(value))
}

fn reason(stage: Stage, source: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Reason { stage, source })
}

fn depth(source: Box<Expr>, limit: Box<Expr>) -> Expr {
    Expr::Depth { source, limit }
}

fn cli(label: &str, source: &str) -> Output {
    let directory =
        std::env::temp_dir().join(format!("nucleoid-nuc82-{label}-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("program.nuc");
    fs::write(&path, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(&path)
        .output()
        .unwrap();
    fs::remove_dir_all(&directory).unwrap();
    output
}

fn repl(source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
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

#[test]
fn appended_document_cases_run_every_assertion() {
    let [spec, reference, readme, examples] = documents();
    let appended = SPEC_TITLES
        .iter()
        .map(|title| (&spec, *title, None))
        .chain([
            (&reference, REFERENCE_TITLE, Some("[41,36,5]")),
            (&readme, README_TITLE, None),
            (&examples, EXAMPLES_TITLE, Some("[2,17,30]")),
        ]);

    let mut assertions = 0;
    for (cases, title, expected) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(depths(&case.source) > 0, "{title} writes a depth stage");
        assert_eq!(case.expected.as_deref(), expected, "{title}");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 55);
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Depth-limited selections\n\n```\n")
        .expect("the language reference documents depth stages");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 82](nuc-0082.md)"));
}

#[test]
fn no_earlier_document_case_writes_a_depth_stage() {
    let mut cases = 0;
    let mut with_depth = 0;
    for case in documents().iter().flatten() {
        cases += 1;
        let uses = depths(&case.source) > 0;
        assert_eq!(uses, is_appended(&case.title), "{}", case.title);
        with_depth += usize::from(uses);
    }
    assert_eq!(with_depth, 11);
    assert_eq!(cases, 1030);
}

#[test]
fn depth_stages_parse_into_their_own_variant() {
    let why_c = || reason(Stage::Why, identifier("c"));
    assert_eq!(
        value_of("x = c |> why |> depth(1)"),
        depth(why_c(), number(1.0))
    );
    assert_eq!(
        value_of("x = why c |> depth(n)"),
        depth(why_c(), identifier("n"))
    );
    assert_eq!(
        value_of("x = c |> depth (1)"),
        depth(identifier("c"), number(1.0))
    );
    assert_eq!(
        value_of("x = c|>why|>depth(1)"),
        depth(why_c(), number(1.0))
    );
    assert_eq!(
        value_of("x = (c |> why |> depth(\n    1\n))"),
        depth(why_c(), number(1.0))
    );
    assert_eq!(
        value_of("x = model |> depth(0)"),
        depth(Box::new(Expr::Model), number(0.0))
    );
    assert_eq!(
        value_of("x = c |> why |> depth(1) |> stated |> depth(n)"),
        depth(
            reason(Stage::Stated, Box::new(depth(why_c(), number(1.0)))),
            identifier("n")
        )
    );
    assert_eq!(
        value_of("x = c |> why |> stated"),
        *reason(Stage::Stated, why_c())
    );
    assert_eq!(
        value_of("x = stated c"),
        *reason(Stage::Stated, identifier("c"))
    );
    assert_eq!(
        value_of("x = depth(c, 1)"),
        Expr::Call {
            callee: identifier("depth"),
            arguments: vec![Expr::Identifier("c".to_string()), Expr::Number(1.0)],
        }
    );
    assert_eq!(
        parse("depth c").unwrap(),
        vec![
            Stmt::Expression(Expr::Identifier("depth".to_string())),
            Stmt::Expression(Expr::Identifier("c".to_string())),
        ]
    );
    assert!(
        Expr::Depth {
            source: identifier("c"),
            limit: number(1.0),
        }
        .contains_reasoning()
    );
}

#[test]
fn ordinary_names_called_depth_keep_their_meaning() {
    checked(
        &mut Runtime::new(),
        r#"
depth = 3
record = {depth: depth, deeper: depth + 1}
def deeper(depth) => depth + 1
class Hole(depth):
    this.depth = depth
    def doubled() => this.depth * 2
hole = Hole(4)
assert(depth, 3)
assert(record.depth, 3)
assert(record.deeper, 4)
assert(deeper(1), 2)
assert(hole.depth, 4)
assert(hole.doubled(), 8)
assert([1, 2].map(depth -> depth * 10), [10, 20])
"#,
    );
    assert_eq!(
        failure(&format!("{PRELUDE}depth(c, 1)")),
        error(ErrorKind::Reference, "depth is not defined", 3, 1)
    );
}

#[test]
fn depth_keeps_the_steps_within_the_limit_in_walk_order() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{CHAIN}d = c + a"));
    assert_eq!(nodes(&mut runtime, "why d"), json!(["d", "c", "a", "b"]));
    assert_eq!(nodes(&mut runtime, "d |> why |> depth(0)"), json!(["d"]));
    assert_eq!(
        nodes(&mut runtime, "d |> why |> depth(1)"),
        json!(["d", "c", "a"])
    );
    assert_eq!(
        nodes(&mut runtime, "d |> why |> depth(2)"),
        json!(["d", "c", "a", "b"])
    );
    assert_eq!(
        evaluate(&mut runtime, "d |> why |> depth(1)"),
        evaluate(&mut runtime, "(why d).slice(0, 3)")
    );
    assert_eq!(
        evaluate(&mut runtime, "(d |> why |> depth(1))[0].from"),
        json!(["c", "a"])
    );
    assert_eq!(
        evaluate(&mut runtime, "d |> why |> depth(10)"),
        evaluate(&mut runtime, "why d")
    );
}

#[test]
fn distances_are_shortest_paths_not_the_depth_of_the_walk() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, SHORTCUT);
    assert_eq!(
        nodes(&mut runtime, "why t"),
        json!(["t", "p", "q", "s", "u", "base"])
    );
    assert_eq!(
        nodes(&mut runtime, "t |> why |> depth(1)"),
        json!(["t", "p", "q"])
    );
    assert_eq!(
        nodes(&mut runtime, "t |> why |> depth(2)"),
        json!(["t", "p", "q", "s", "u"])
    );
    assert_eq!(
        nodes(&mut runtime, "t |> why |> depth(3)"),
        json!(["t", "p", "q", "s", "u", "base"])
    );
    checked(&mut runtime, "p = 2");
    assert_eq!(
        nodes(&mut runtime, "t |> why |> depth(3)"),
        json!(["t", "p", "q", "s", "u"])
    );
}

#[test]
fn a_two_link_lattice_keeps_every_node_within_its_shortest_distance() {
    let size: usize = 40;
    let mut source = String::from("v0 = 1\nv1 = 1\n");
    for index in 2..size {
        source.push_str(&format!("v{index} = v{} + v{}\n", index - 1, index - 2));
    }
    let mut runtime = Runtime::new();
    checked(&mut runtime, &source);
    let last = size - 1;
    let walked = nodes(&mut runtime, &format!("why v{last}"));
    for limit in [0usize, 1, 2, 5, 19, 20] {
        let expected: Vec<Json> = walked
            .as_array()
            .unwrap()
            .iter()
            .filter(|node| {
                let index: usize = node.as_str().unwrap()[1..].parse().unwrap();
                (last - index).div_ceil(2) <= limit
            })
            .cloned()
            .collect();
        assert_eq!(
            nodes(&mut runtime, &format!("v{last} |> why |> depth({limit})")),
            Json::Array(expected),
            "{limit}"
        );
    }
    assert_eq!(
        nodes(&mut runtime, "v0 |> affects |> depth(1)"),
        json!(["v2"])
    );
    assert_eq!(
        nodes(&mut runtime, "v1 |> affects |> depth(1)"),
        json!(["v2", "v3"])
    );
}

#[test]
fn impact_distances_start_at_one() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, PRICES);
    assert_eq!(
        nodes(&mut runtime, "affects price"),
        json!(["tax", "total", "label"])
    );
    assert_eq!(
        nodes(&mut runtime, "price |> affects |> depth(1)"),
        json!(["tax", "total"])
    );
    assert_eq!(
        evaluate(&mut runtime, "price |> affects |> depth(0)"),
        json!([])
    );
    assert_eq!(
        nodes(&mut runtime, "tax |> why |> affects |> depth(1)"),
        json!(["tax", "total"])
    );
    assert_eq!(
        nodes(&mut runtime, "tax |> why |> affects |> depth(0)"),
        json!([])
    );
    assert_eq!(
        nodes(&mut runtime, "price |> affects |> depth(2)"),
        json!(["tax", "total", "label"])
    );
    let mut detour = Runtime::new();
    checked(&mut detour, DETOUR);
    assert_eq!(
        nodes(&mut detour, "x |> why |> depth(1)"),
        json!(["x", "u", "w"])
    );
    assert_eq!(
        nodes(&mut detour, "x |> why |> depth(1) |> affects"),
        json!(["m", "x", "u"])
    );
    assert_eq!(
        nodes(&mut detour, "x |> why |> depth(1) |> affects |> depth(1)"),
        json!(["m", "x"])
    );
    assert_eq!(
        nodes(&mut detour, "x |> why |> depth(1) |> affects |> depth(2)"),
        json!(["m", "x", "u"])
    );
}

#[test]
fn steps_that_no_walk_produced_are_at_distance_zero() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, CHAIN);
    assert_eq!(nodes(&mut runtime, "c |> depth(0)"), json!(["c"]));
    assert_eq!(nodes(&mut runtime, "c |> depth(5)"), json!(["c"]));
    assert_eq!(
        nodes(&mut runtime, "model |> depth(0)"),
        json!(["a", "b", "c"])
    );
    assert_eq!(nodes(&mut runtime, "stated a |> depth(0)"), json!(["a"]));
    assert_eq!(evaluate(&mut runtime, "stated c |> depth(0)"), json!([]));
    assert_eq!(
        nodes(&mut runtime, "model |> why |> depth(0)"),
        json!(["a", "b", "c"])
    );
}

#[test]
fn filters_keep_distances_and_later_walks_measure_from_their_own_input() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "base = 2\nrate = base * 2\nfee = 1\ncost = rate * 4\nprice = cost + fee",
    );
    assert_eq!(
        nodes(&mut runtime, "price |> why |> depth(2) |> stated"),
        json!(["fee"])
    );
    assert_eq!(
        nodes(&mut runtime, "price |> why |> stated |> depth(2)"),
        json!(["fee"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "price |> why |> depth(3) |> derived |> depth(1)"
        ),
        json!(["price", "cost"])
    );
    assert_eq!(
        nodes(&mut runtime, "price |> why |> depth(0) |> why |> depth(1)"),
        json!(["price", "cost", "fee"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "price |> why |> stated |> affects |> depth(1)"
        ),
        json!(["rate", "price"])
    );
    assert_eq!(
        nodes(&mut runtime, "price |> why |> depth(1) |> depth(0)"),
        json!(["price"])
    );
}

#[test]
fn limits_must_be_non_negative_integer_numbers() {
    let cases = [
        ("-1", "-1"),
        ("0.5", "0.5"),
        ("n", "n"),
        ("0/0", "0/0"),
        ("1/0", "1/0"),
        ("-1/0", "-1/0"),
        ("Number.POSITIVE_INFINITY", "Number.POSITIVE_INFINITY"),
        ("\"1\"", "\"1\""),
        ("true", "true"),
        ("[1]", "[1]"),
        ("{k: 1}", "{\"k\":1}"),
        ("K", "K"),
        ("k", "k"),
        ("f", "f"),
        ("v -> 1", "function()"),
        ("/a/", "/a/"),
        ("Date(0)", "Date(0)"),
        ("box.level", "box.level"),
    ];
    let setup = format!("{PRELUDE}n = 1.5\nclass K\nk = K()\ndef f(v) => v\nbox = {{level: -2}}\n");
    for (limit, named) in cases {
        let mut runtime = Runtime::new();
        checked(&mut runtime, &setup);
        assert_eq!(
            failure_in(&mut runtime, &format!("x = c |> why |> depth({limit})")),
            error(
                ErrorKind::Type,
                &format!("{named} is not a non-negative integer"),
                1,
                1
            ),
            "{limit}"
        );
        assert_eq!(
            evaluate(&mut runtime, "[n, k.id, box.level]"),
            json!([1.5, "k", -2.0])
        );
    }
    let mut runtime = Runtime::new();
    checked(&mut runtime, PRELUDE);
    for (limit, count) in [("0", 1), ("-0", 1), ("1", 2), ("2.0", 2), ("1e300", 2)] {
        assert_eq!(
            evaluate(
                &mut runtime,
                &format!("(c |> why |> depth({limit})).length")
            ),
            json!(f64::from(count)),
            "{limit}"
        );
    }
    assert_eq!(
        failure(&format!(
            "{PRELUDE}class Plan(level):\n    this.level = level\n$Plan.near = c |> why |> depth($Plan.level)\nplan = Plan(1.5)"
        )),
        error(
            ErrorKind::Type,
            "Plan.level is not a non-negative integer",
            6,
            1
        )
    );
}

#[test]
fn unknown_limits_make_the_selection_undefined() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}class Plan\nplan = Plan()\nknown = null"),
    );
    assert!(matches!(
        runtime.run("c |> why |> depth(null)").unwrap(),
        Value::Undefined
    ));
    assert!(matches!(
        runtime.run("c |> why |> depth(plan.depth)").unwrap(),
        Value::Undefined
    ));
    checked(
        &mut runtime,
        r#"
pending = c |> why |> depth(plan.depth)
waiting = c |> why |> depth(known)
ready = all s in c |> why |> depth(known): s.holds > 0
size = (c |> why |> depth(known)).length
names = (c |> why |> depth(known)).map(s -> s.node)
skipped = c |> why |> depth(null) |> depth(missing) |> stated
assert(pending, null)
assert(waiting, null)
assert(ready, null)
assert(size, null)
assert(names, null)
assert(skipped, null)
plan.depth = 0
known = 1
assert(pending.map(s -> s.node), ["c"])
assert(waiting.map(s -> s.node), ["c", "b"])
assert(ready, true)
assert(size, 2)
assert(names, ["c", "b"])
known = null
assert(waiting, null)
"#,
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}def near(limit):\n    return (c |> why |> depth(limit)).length\n[near(null), near(1)]"
        )),
        json!([null, 2.0])
    );
}

#[test]
fn limits_are_evaluated_in_place_once_after_the_source() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}{COUNTER}near = c |> why |> depth(note(1))"),
    );
    assert_eq!(
        evaluate(&mut runtime, "[calls.length, near.length]"),
        json!([1.0, 2.0])
    );
    checked(&mut runtime, "a = 5");
    assert_eq!(
        evaluate(&mut runtime, "[calls.length, near[1].holds]"),
        json!([2.0, 7.0])
    );
    assert_eq!(
        failure(&format!("{PRELUDE}1 |> depth(zz)")),
        error(ErrorKind::Type, "Cannot explain 1", 3, 1)
    );
    assert_eq!(
        failure(&format!("{PRELUDE}missing |> depth(-1)")),
        error(ErrorKind::Reference, "missing is not defined", 3, 1)
    );
    assert_eq!(
        failure(&format!("{PRELUDE}c |> why |> depth(zz)")),
        error(ErrorKind::Reference, "zz is not defined", 3, 1)
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}def within(levels):\n    near = c |> why |> depth(levels)\n    return near.map(s -> s.node)\n[within(0), within(2)]"
        )),
        json!([["c"], ["c", "b", "a"]])
    );
}

#[test]
fn every_binding_named_depth_shadows_the_stage() {
    let bindings = [
        ("depth = 0\nx = c |> why |> depth(1)", 4),
        ("depth = null\nx = c |> why |> depth(1)", 4),
        ("depth = v => v\nx = c |> why |> depth(1)", 4),
        ("def depth(v) => v\nx = c |> why |> depth(1)", 4),
        ("class depth\nx = c |> why |> depth(1)", 4),
        ("depth = 2\nmissing |> depth(1)", 4),
        ("def f(depth) => c |> why |> depth(1)\nf(1)", 4),
        (
            "class M:\n    def m(depth) => c |> why |> depth(1)\nM().m(1)",
            5,
        ),
        ("f = lambda depth: c |> why |> depth(1)\nf(1)", 4),
        ("f = depth -> c |> why |> depth(1)\nf(1)", 4),
        ("f = (depth) => c |> why |> depth(1)\nf(1)", 4),
        ("[1].map(depth -> c |> why |> depth(1))", 3),
        ("for depth of \"x\":\n    y = c |> why |> depth(1)", 3),
        (
            "ok = all depth in [1]: (c |> why |> depth(1)).length > 0",
            3,
        ),
        ("{\n    depth = 5\n    y = c |> why |> depth(1)\n}", 3),
    ];
    for (binding, line) in bindings {
        assert_eq!(
            failure(&format!("{PRELUDE}{binding}")),
            error(ErrorKind::Type, "depth is not an operation", line, 1),
            "{binding}"
        );
    }
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{PRELUDE}depth = 2"));
    assert_eq!(
        failure_in(&mut runtime, "c |> why |> depth(1)"),
        error(ErrorKind::Type, "depth is not an operation", 1, 1)
    );
    checked(&mut runtime, "delete depth");
    assert_eq!(nodes(&mut runtime, "c |> why |> depth(0)"), json!(["c"]));
    checked(
        &mut runtime,
        "{\n    depth = 5\n    try:\n        y = c |> why |> depth(1)\n    catch error:\n        assert(error, TypeError(\"depth is not an operation\"))\n}",
    );
}

#[test]
fn a_binding_declared_after_a_standing_selection_fails_its_refresh() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}near = c |> why |> depth(1)\ndepth = 5"),
    );
    assert_eq!(
        evaluate(&mut runtime, "near.map(s -> s.holds)"),
        json!([6.0, 3.0])
    );
    assert_eq!(
        failure_in(&mut runtime, "a = 2"),
        error(ErrorKind::Type, "depth is not an operation", 1, 1)
    );
    assert_eq!(evaluate(&mut runtime, "[a, b, c]"), json!([1.0, 3.0, 6.0]));
    assert_eq!(
        evaluate(&mut runtime, "near.map(s -> s.holds)"),
        json!([6.0, 3.0])
    );
    checked(&mut runtime, "delete depth\na = 2");
    assert_eq!(
        evaluate(&mut runtime, "near.map(s -> s.holds)"),
        json!([8.0, 4.0])
    );
}

#[test]
fn depth_is_shadowed_exactly_as_the_stated_filter_is() {
    let situations = [
        "{NAME} = 0\nx = c |> why |> {STAGE}",
        "def {NAME}(v) => v\nx = c |> why |> {STAGE}",
        "class {NAME}\nx = c |> why |> {STAGE}",
        "{NAME} = 0\nmissing |> {STAGE}",
        "def f({NAME}) => c |> why |> {STAGE}\nf(null)",
        "f = {NAME} -> c |> why |> {STAGE}\nf(1)",
        "ok = all {NAME} in [1]: (c |> why |> {STAGE}).length > 0",
        "{\n    {NAME} = 5\n    y = c |> why |> {STAGE}\n}",
        "x = c |> why |> {STAGE}\n{NAME} = 5\na = 2",
        "x = c |> why |> {STAGE}\n{NAME} = 5\ndelete {NAME}\na = 2\nx.length",
        "{NAME} = 5\ndelete {NAME}\n(c |> why |> {STAGE}).length",
    ];
    for situation in situations {
        let program = |name: &str, stage: &str| {
            format!(
                "{PRELUDE}{}",
                situation.replace("{NAME}", name).replace("{STAGE}", stage)
            )
        };
        let outcome =
            |source: String| -> Result<(Json, Json), (ErrorKind, String, Option<Position>)> {
                let mut runtime = Runtime::new();
                match runtime.run(&source) {
                    Ok(value) => {
                        let state = runtime.run("[a, c]").unwrap();
                        Ok((
                            json_of(&runtime, &value).is_null().into(),
                            json_of(&runtime, &state),
                        ))
                    }
                    Err(error) => Err((error.kind(), error.message(), error.position())),
                }
            };
        let filtered = outcome(program("stated", "stated"));
        let deep = outcome(program("depth", "depth(1)"));
        match (&filtered, &deep) {
            (Err(stated), Err(depth)) => {
                assert_eq!(stated.0, depth.0, "{situation}");
                assert_eq!(stated.1.replace("stated", "depth"), depth.1, "{situation}");
                assert_eq!(stated.2, depth.2, "{situation}");
            }
            (Ok(stated), Ok(depth)) => assert_eq!(stated.1, depth.1, "{situation}"),
            _ => panic!("{situation}: {filtered:?} against {deep:?}"),
        }
    }
}

#[test]
fn why_and_affects_keep_ignoring_bindings() {
    assert_eq!(
        result(&format!(
            "{CHAIN}why = 5\naffects = 6\n[(c |> why |> depth(1)).length, (a |> affects |> depth(1)).length, why, affects]"
        )),
        json!([2.0, 1.0, 5.0, 6.0])
    );
}

#[test]
fn depth_selections_stay_live_through_limits_values_and_structure() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{CHAIN}limit = 1\nnear = c |> why |> depth(limit)\nfar = a |> affects |> depth(limit)"
        ),
    );
    assert_eq!(nodes(&mut runtime, "near"), json!(["c", "b"]));
    assert_eq!(nodes(&mut runtime, "far"), json!(["b"]));
    checked(&mut runtime, "limit = 2");
    assert_eq!(nodes(&mut runtime, "near"), json!(["c", "b", "a"]));
    assert_eq!(nodes(&mut runtime, "far"), json!(["b", "c"]));
    checked(&mut runtime, "limit = 1\nx = 7\nb = x + a");
    assert_eq!(nodes(&mut runtime, "near"), json!(["c", "b"]));
    checked(&mut runtime, "limit = 2");
    assert_eq!(nodes(&mut runtime, "near"), json!(["c", "b", "x", "a"]));
    assert_eq!(
        evaluate(&mut runtime, "near.map(s -> s.holds)"),
        json!([16.0, 8.0, 7.0, 1.0])
    );
    checked(&mut runtime, "x = 9");
    assert_eq!(evaluate(&mut runtime, "near[2].holds"), json!(9.0));
    checked(&mut runtime, "c = a + 1");
    assert_eq!(nodes(&mut runtime, "near"), json!(["c", "a"]));
    assert_eq!(nodes(&mut runtime, "far"), json!(["b", "c"]));
    checked(&mut runtime, "delete c");
    assert_eq!(evaluate(&mut runtime, "near"), json!([]));
    checked(&mut runtime, "c = b * 3");
    assert_eq!(nodes(&mut runtime, "near"), json!(["c", "b", "x", "a"]));
    assert_eq!(
        nodes(&mut runtime, "model |> why"),
        json!(["a", "b", "c", "limit", "x"])
    );
    assert_eq!(nodes(&mut runtime, "affects limit"), json!([]));
    assert_eq!(nodes(&mut runtime, "affects a"), json!(["b", "c"]));
}

#[test]
fn the_graph_records_limit_reads_and_walk_observations() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}n = 1\nnear = c |> why |> depth(n)"),
    );
    assert_eq!(
        graph_line(&runtime, "near"),
        r#"near reads ["a", "b", "c", "n"] observes ["a", "b", "c"]"#
    );
    let output = cli(
        "graph",
        &format!("{CHAIN}n = 1\nnear = c |> why |> depth(n)\n1"),
    );
    let listing = String::from_utf8(output.stdout).unwrap();
    assert!(listing.contains("  near [variable]\n    reads    c, b, a, n\n    observes c, b, a\n"));
    assert!(listing.contains("  n [variable]\n    updates  near\n"));
}

#[test]
fn snapshots_freeze_selections_and_limits() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{CHAIN}n = 1\nnear = c |> why |> depth(n)\nfixed = c |> why |> depth(n.value)\nfrozen = near.value\nlater = c |> why |> stated |> depth(n.value + 1)"
        ),
    );
    assert_eq!(stored(&runtime, "near"), "c|>why|>depth(n)");
    assert_eq!(stored(&runtime, "fixed"), "c|>why|>depth(1)");
    assert_eq!(stored(&runtime, "later"), "c|>why|>stated|>depth(1+1)");
    checked(&mut runtime, "n = 2\na = 4");
    assert_eq!(nodes(&mut runtime, "near"), json!(["c", "b", "a"]));
    assert_eq!(nodes(&mut runtime, "fixed"), json!(["c", "b"]));
    assert_eq!(
        evaluate(&mut runtime, "fixed.map(s -> s.holds)"),
        json!([12.0, 6.0])
    );
    assert_eq!(
        evaluate(&mut runtime, "frozen.map(s -> s.holds)"),
        json!([6.0, 3.0])
    );
    assert_eq!(nodes(&mut runtime, "later"), json!(["a"]));
    checked(&mut runtime, "trace = c |> why\nsnap = trace.value");
    assert_eq!(stored(&runtime, "trace"), "c|>why");
}

#[test]
fn failed_changes_roll_back_depth_selections() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}n = 1\nnear = c |> why |> depth(n)\nk = c |> why |> depth(1) |> stated"),
    );
    checked(
        &mut runtime,
        "try:\n    n = 2\n    b = 9\n    assert(near.map(s -> s.node), [\"c\", \"b\"])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")",
    );
    assert_eq!(nodes(&mut runtime, "near"), json!(["c", "b"]));
    assert_eq!(
        evaluate(&mut runtime, "[n, b, near[1].holds]"),
        json!([1.0, 3.0, 3.0])
    );
    assert_eq!(
        failure_in(&mut runtime, "n = -1"),
        error(ErrorKind::Type, "n is not a non-negative integer", 1, 1)
    );
    assert_eq!(evaluate(&mut runtime, "n"), json!(1.0));
    assert_eq!(nodes(&mut runtime, "near"), json!(["c", "b"]));
    assert_eq!(
        evaluate(
            &mut runtime,
            "plan = suppose:\n    n = 2\n    near.length\n[plan, near.length]"
        ),
        json!([3.0, 2.0])
    );
    checked(&mut runtime, "n = 2");
    assert_eq!(nodes(&mut runtime, "near"), json!(["c", "b", "a"]));
    assert_eq!(evaluate(&mut runtime, "k"), json!([]));
}

#[test]
fn quantifiers_conditions_and_class_rules_hold_depth_stages() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{CHAIN}ok = all s in c |> why |> depth(1): s.holds > 2\nwide = 0\nif (c |> why |> depth(1)).length > 1:\n    wide = 1\nclass Gauge(level):\n    this.level = level\n$Gauge.near = c |> why |> depth($Gauge.level)\nlow = Gauge(0)\nhigh = Gauge(2)"
        ),
    );
    assert_eq!(evaluate(&mut runtime, "[ok, wide]"), json!([true, 1.0]));
    assert_eq!(
        evaluate(&mut runtime, "[low.near.length, high.near.length]"),
        json!([1.0, 3.0])
    );
    checked(&mut runtime, "b = 1\nlow.level = 1");
    assert_eq!(
        evaluate(&mut runtime, "[ok, low.near.length]"),
        json!([false, 2.0])
    );
    assert_eq!(
        nodes(&mut runtime, "model |> why |> depth(0) |> stated"),
        nodes(&mut runtime, "model |> stated")
    );
}

#[test]
fn depth_renders_once_and_parses_back() {
    for source in [
        "c |> why |> depth(1)",
        "why c |> depth(0)",
        "model |> depth(0) |> stated",
        "a |> affects |> depth(n + 1) |> why |> depth(2)",
        "c |> why |> depth(n.value)",
        "c |> why |> depth(plan.depth ?? 1)",
        "c |> why |> depth(n > 0 ? 1 : 0)",
        "a || b |> depth(1)",
        "c |> why |> depth(all v in xs: v > 0)",
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
        parse_expression("c |> why |> depth(n > 0 ? 1 : 0)")
            .unwrap()
            .to_string(),
        "c|>why|>depth((n>0?1:0))"
    );
    let mut chained = String::from("c");
    let mut nested = String::from("1");
    for level in 1..=6 {
        chained.push_str(" |> depth(1)");
        nested = format!("c |> depth({nested})");
        let chain = parse_expression(&chained).unwrap().to_string();
        let limit = parse_expression(&nested).unwrap().to_string();
        assert_eq!(chain.len(), 1 + 10 * level, "{chain}");
        assert_eq!(limit.len(), 1 + 10 * level, "{limit}");
    }
    assert_eq!(
        generate(&parse("def f(x = 1) => c |> why |> depth(x)").unwrap()[0]),
        "def f(x=1):
    return c|>why|>depth(x)"
    );
    let output = cli(
        "keys",
        &format!("{CHAIN}if (c |> why |> depth(1)).length > 1:\n    flag = 1\n1"),
    );
    let listing = String::from_utf8(output.stdout).unwrap();
    assert!(
        listing.contains("  if(c|>why|>depth(1).length>1) [if]\n"),
        "{listing}"
    );
}

#[test]
fn depth_stages_handle_large_graphs() {
    let size = 20_000;
    let mut source = String::from("v0 = 1\n");
    for index in 1..size {
        source.push_str(&format!("v{index} = v{} + 1\n", index - 1));
    }
    let mut runtime = Runtime::new();
    checked(&mut runtime, &source);
    let last = size - 1;
    assert_eq!(
        nodes(&mut runtime, &format!("v{last} |> why |> depth(3)")),
        json!([
            format!("v{last}"),
            format!("v{}", last - 1),
            format!("v{}", last - 2),
            format!("v{}", last - 3)
        ])
    );
    assert_eq!(
        nodes(&mut runtime, "v0 |> affects |> depth(2)"),
        json!(["v1", "v2"])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            &format!("(v{last} |> why |> depth({size})).length")
        ),
        json!(20_000.0)
    );
}

#[test]
fn near_misses_keep_their_accepted_results() {
    let parse_time = unpositioned(ErrorKind::Type, "depth is not an operation");
    for line in [
        "c |> why |> depth",
        "c |> why |> depth\n(1)",
        "c |> why |> depth[1]",
        "c |> why |> depth.x",
        "c |> why |> depth{1}",
    ] {
        assert_eq!(failure(&format!("{PRELUDE}{line}")), parse_time, "{line}");
    }
    assert_eq!(
        failure(&format!("{PRELUDE}c |> why |> Depth(1)")),
        unpositioned(ErrorKind::Type, "Depth is not an operation")
    );
    assert_eq!(
        failure(&format!("{PRELUDE}c |> why |> depths(1)")),
        unpositioned(ErrorKind::Type, "depths is not an operation")
    );
    for (line, expected) in [
        (
            "depth c",
            error(ErrorKind::Reference, "depth is not defined", 3, 1),
        ),
        (
            "c.depth(1)",
            error(ErrorKind::Type, "number.depth is not a function", 3, 1),
        ),
        (
            "c | > depth(1)",
            error(ErrorKind::Syntax, "Unexpected character '|'", 3, 3),
        ),
        (
            "c ||> depth(1)",
            error(ErrorKind::Syntax, "Unexpected '>'", 3, 7),
        ),
        (
            "c |> |> depth(1)",
            error(ErrorKind::Syntax, "Expected an operation", 3, 6),
        ),
    ] {
        assert_eq!(failure(&format!("{PRELUDE}{line}")), expected, "{line}");
    }
    assert_eq!(
        result(&format!("{PRELUDE}depth = 2\ndepth + 1")),
        json!(3.0)
    );
    assert_eq!(
        result(&format!("{PRELUDE}def depth(x, y) => y\ndepth(c, 1)")),
        json!(1.0)
    );
    assert_eq!(
        result(&format!("{PRELUDE}x = {{depth: 1}}\nx.depth")),
        json!(1.0)
    );
}

#[test]
fn inputs_that_were_errors_now_report_what_the_proposal_lists() {
    let both = json!([
        {"from": ["a"], "holds": 2.0, "node": "c", "rule": "c = a+1", "state": "derived"},
        {"from": [], "holds": 1.0, "node": "a", "rule": "a = 1", "state": "stated"}
    ]);
    for line in [
        "c |> why |> depth(1)",
        "c |> why |> depth (1)",
        "c|>why|>depth(1)",
    ] {
        assert_eq!(result(&format!("{PRELUDE}{line}")), both, "{line}");
    }
    for line in ["c |> depth(5)", "why c |> depth(0)"] {
        assert_eq!(
            result(&format!("{PRELUDE}{line}")),
            json!([both[0]]),
            "{line}"
        );
    }
    assert_eq!(
        result(&format!("{PRELUDE}model |> depth(0)")),
        json!([both[1], both[0]])
    );
    assert_eq!(
        result(&format!("{PRELUDE}a |> affects |> depth(0)")),
        json!([])
    );
    assert_eq!(
        result(&format!("{PRELUDE}c |> why |> depth(null)")),
        Json::Null
    );
    assert_eq!(
        result(&format!("{PRELUDE}c |> why |> depth(1)(2)")),
        json!(2.0)
    );
    assert_eq!(
        result(&format!("{PRELUDE}q, r = c |> why |> depth(1)\n[q, r]")),
        both
    );
    for (line, expected) in [
        (
            "c |> why |> depth()",
            error(ErrorKind::Syntax, "Unexpected ')'", 3, 20),
        ),
        (
            "c |> why |> depth(1, 2)",
            error(ErrorKind::Syntax, "Expected ')' but found ','", 3, 20),
        ),
        (
            "c |> why |> depth(1,)",
            error(ErrorKind::Syntax, "Expected ')' but found ','", 3, 20),
        ),
        (
            "c |> why |> depth(1",
            error(
                ErrorKind::Syntax,
                "Expected ')' but found end of input",
                3,
                20,
            ),
        ),
        (
            "c |> why |> depth(",
            error(ErrorKind::Syntax, "Unexpected end of input", 3, 19),
        ),
        (
            "c |> why |> depth(1).length",
            error(ErrorKind::Syntax, "Unexpected '.'", 3, 22),
        ),
        (
            "c |> why |> depth(-1)",
            error(ErrorKind::Type, "-1 is not a non-negative integer", 3, 1),
        ),
        (
            "depth = 2\nc |> why |> depth(1)",
            error(ErrorKind::Type, "depth is not an operation", 4, 1),
        ),
        (
            "1 |> depth(1)",
            error(ErrorKind::Type, "Cannot explain 1", 3, 1),
        ),
        (
            "missing |> depth(-1)",
            error(ErrorKind::Reference, "missing is not defined", 3, 1),
        ),
    ] {
        assert_eq!(failure(&format!("{PRELUDE}{line}")), expected, "{line}");
    }
}

#[test]
fn the_native_repl_runs_depth_stages_line_by_line() {
    let output = repl(&format!(
        "{PRELUDE}(c |> why |> depth(1)).length\nc |> why |> depth(\n(c |> why |> depth(0)).length\nc |> why |> depth\n"
    ));
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{BANNER}1.0\n2.0\n2.0\n1.0\n")
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "SyntaxError: Unexpected end of input (line 2, column 1)\nTypeError: depth is not an operation\n"
    );
}
