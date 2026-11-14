mod common;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate;
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_expression};
use nucleoid::reasoning::Stage;
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde_json::{Value as Json, json};

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid lists what a supposed premise changes",
    "Nucleoid lists what changed in an explanation or an impact",
    "Nucleoid compares instances and class level rules in a hypothetical",
    "Nucleoid compares lists and objects in a hypothetical",
    "Nucleoid leaves equal values and undone writes out of a diff",
    "Nucleoid compares nested hypotheticals with the innermost block",
    "Nucleoid finds hypothetical changes in functions methods and quantifiers",
    "Nucleoid composes hypothetical differences with state filters",
];

const REFERENCE_TITLE: &str = "Nucleoid lists what a hypothetical changes with diff";
const README_TITLE: &str = "Diff lists what a warmer room would change in a heating plan";
const EXAMPLES_TITLE: &str = "Diff shows which orders a delayed shipment would make late";

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

const OUTSIDE: &str = "Cannot use 'diff' outside suppose";

const PRELUDE: &str = "x = 1\nc = x + 1\n";

const CHAIN: &str = "a = 1\nb = a + 1\nc = b * 2\n";

const CHANGED: &str = "(model |> diff).map(s -> s.node)";

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "hypothetical difference regression".to_string(),
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

fn changed(prelude: &str, header: &str) -> Json {
    result(&format!("{prelude}suppose {header}: {CHANGED}"))
}

fn changed_by(prelude: &str, statement: &str) -> Json {
    result(&format!(
        "{prelude}suppose:\n    {statement}\n    {CHANGED}"
    ))
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

fn outside(line: usize) -> (ErrorKind, String, Option<Position>) {
    error(ErrorKind::Type, OUTSIDE, line, 1)
}

fn shadowed(line: usize) -> (ErrorKind, String, Option<Position>) {
    error(ErrorKind::Type, "diff is not an operation", line, 1)
}

fn diffs(source: &str) -> usize {
    let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    format!("{statements:?}").matches("stage: Diff").count()
}

fn value_of(source: &str) -> Expr {
    match parse(source).unwrap().pop() {
        Some(Stmt::Assign { value, .. }) => value,
        other => panic!("{source} ends with {other:?}"),
    }
}

fn identifier(name: &str) -> Box<Expr> {
    Box::new(Expr::Identifier(name.to_string()))
}

fn reason(stage: Stage, source: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Reason { stage, source })
}

fn cli(label: &str, source: &str) -> Output {
    let directory =
        std::env::temp_dir().join(format!("nucleoid-nuc83-{label}-{}", std::process::id()));
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
            (
                &reference,
                REFERENCE_TITLE,
                Some("[\"qty\",\"subtotal\",\"total\",\"free\"]"),
            ),
            (&readme, README_TITLE, None),
            (&examples, EXAMPLES_TITLE, Some("[\"lamp.days\"]")),
        ]);

    let mut assertions = 0;
    for (cases, title, expected) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(diffs(&case.source) > 0, "{title} writes a diff stage");
        assert_eq!(case.expected.as_deref(), expected, "{title}");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 51);
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Hypothetical differences\n\n```\n")
        .expect("the language reference documents the diff stage");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 83](nuc-0083.md)"));
}

#[test]
fn no_earlier_document_case_writes_a_diff_stage() {
    let mut cases = 0;
    let mut with_diff = 0;
    for case in documents().iter().flatten() {
        cases += 1;
        let uses = diffs(&case.source) > 0;
        assert_eq!(uses, is_appended(&case.title), "{}", case.title);
        with_diff += usize::from(uses);
    }
    assert_eq!(with_diff, 11);
    assert_eq!(cases, 1105);
}

#[test]
fn diff_parses_into_a_reasoning_stage() {
    assert_eq!(
        value_of("y = c |> diff"),
        *reason(Stage::Diff, identifier("c"))
    );
    assert_eq!(
        value_of("y = model |> diff"),
        *reason(Stage::Diff, Box::new(Expr::Model))
    );
    assert_eq!(
        value_of("y = c |> why |> diff"),
        *reason(Stage::Diff, reason(Stage::Why, identifier("c")))
    );
    assert_eq!(
        value_of("y = why c |> diff"),
        value_of("y = c |> why |> diff")
    );
    assert_eq!(value_of("y = c|>diff"), value_of("y = c |> diff"));
    assert_eq!(
        value_of("y = c |> diff |> depth(1)"),
        Expr::Depth {
            source: reason(Stage::Diff, identifier("c")),
            limit: Box::new(Expr::Number(1.0)),
        }
    );
    assert_eq!(Stage::Diff.as_str(), "diff");
    assert_eq!(Stage::from_name("diff"), None);
    for (source, count) in [
        ("diff c", 2),
        ("diff(c)", 1),
        ("c.diff", 1),
        ("y = {diff: 1}", 1),
        ("c |> diff(1)", 2),
        ("c |> diff x", 2),
    ] {
        assert_eq!(parse(source).unwrap().len(), count, "{source}");
    }
    assert_eq!(diffs("diff c\ndiff(c)\nc.diff\ny = {diff: 1}"), 0);
    assert_eq!(diffs("c |> diff(1)"), 1);
    assert!(parse_expression("c |> diff").unwrap().contains_reasoning());
}

#[test]
fn names_keys_properties_and_functions_called_diff_keep_their_meaning() {
    assert_eq!(result(&format!("{PRELUDE}diff = 2\ndiff + 1")), json!(3.0));
    assert_eq!(
        result(&format!("{PRELUDE}def diff(v) => v\ndiff(c)")),
        json!(2.0)
    );
    assert_eq!(
        result(&format!("{PRELUDE}y = {{diff: 1}}\ny.diff")),
        json!(1.0)
    );
    assert_eq!(result(&format!("{PRELUDE}c.diff")), Json::Null);
    assert_eq!(
        result(
            "class Pair(left, right):\n    this.left = left\n    this.right = right\n    def diff() => this.right - this.left\npair = Pair(2, 7)\npair.diff()"
        ),
        json!(5.0)
    );
    assert_eq!(
        result(&format!("{PRELUDE}def gap(diff) => diff * 2\ngap(c)")),
        json!(4.0)
    );
    assert_eq!(
        result(&format!(
            "{PRELUDE}suppose x = 2:\n    diff = c * 10\n    diff"
        )),
        json!(30.0)
    );
    assert_eq!(
        failure(&format!("{PRELUDE}suppose x = 2: diff")),
        error(ErrorKind::Reference, "diff is not defined", 3, 1)
    );
}

#[test]
fn every_binding_named_diff_shadows_the_stage() {
    let bindings = [
        ("diff = 0\nsuppose x = 2: c |> diff", 4),
        ("diff = null\nsuppose x = 2: c |> diff", 4),
        ("diff = v => v\nsuppose x = 2: c |> diff", 4),
        ("def diff(v) => v\nsuppose x = 2: c |> diff", 4),
        ("class diff\nsuppose x = 2: c |> diff", 4),
        ("diff = 2\nsuppose x = 2: missing |> diff", 4),
        ("diff = 2\nc |> diff", 4),
        ("def f(diff):\n    suppose x = 2: return c |> diff\nf(1)", 5),
        (
            "class M:\n    def m(diff):\n        suppose x = 2: return c |> diff\nM().m(1)",
            6,
        ),
        ("f = lambda diff: c |> diff\nsuppose x = 2: f(1)", 4),
        ("f = diff -> c |> diff\nsuppose x = 2: f(1)", 4),
        ("f = (diff) => c |> diff\nsuppose x = 2: f(1)", 4),
        ("suppose x = 2: [1].map(diff -> c |> diff)", 3),
        (
            "suppose x = 2:\n    for diff of \"x\":\n        y = c |> diff",
            3,
        ),
        ("suppose x = 2: any diff in [1]: (c |> diff).length > 0", 3),
        (
            "suppose x = 2:\n    {\n        diff = 5\n        y = c |> diff\n    }",
            3,
        ),
    ];
    for (binding, line) in bindings {
        assert_eq!(
            failure(&format!("{PRELUDE}{binding}")),
            shadowed(line),
            "{binding}"
        );
    }
    let mut runtime = Runtime::new();
    checked(&mut runtime, PRELUDE);
    assert_eq!(
        failure_in(
            &mut runtime,
            "suppose x = 2:\n    m = c |> diff\n    diff = 3\n    x = 4"
        ),
        shadowed(1)
    );
    checked(&mut runtime, "diff = 3");
    assert_eq!(
        failure_in(&mut runtime, "suppose x = 2: c |> diff"),
        shadowed(1)
    );
    checked(&mut runtime, "delete diff");
    assert_eq!(
        evaluate(&mut runtime, "suppose x = 2: (c |> diff).map(s -> s.holds)"),
        json!([3.0])
    );
    for name in ["why", "affects"] {
        assert_eq!(
            result(&format!(
                "{PRELUDE}{name} = 5\nsuppose x = 2: (c |> {name} |> diff).length"
            )),
            json!(if name == "why" { 2.0 } else { 0.0 }),
            "{name}"
        );
    }
    for name in ["stated", "derived", "depth"] {
        let stage = if name == "depth" { "depth(1)" } else { name };
        assert_eq!(
            failure(&format!(
                "{PRELUDE}{name} = 5\nsuppose x = 2: c |> diff |> {stage}"
            )),
            error(
                ErrorKind::Type,
                &format!("{name} is not an operation"),
                4,
                1
            ),
            "{name}"
        );
    }
}

#[test]
fn outside_a_block_the_stage_raises_wherever_it_runs() {
    for line in [
        "c |> diff",
        "c|>diff",
        "model |> diff",
        "c |> why |> diff",
        "m = model |> diff",
        "n = (model |> diff).length",
        "ok = any s in model |> diff: true",
        "if (model |> diff).length > 0:\n    z = 1",
        "{node} = c |> diff",
        "q, r = c |> diff",
        "c |> diff(1)",
        "c |> diff[0]",
        "c |> diff x",
        "c |> why |> diff |> depth(-1)",
    ] {
        assert_eq!(failure(&format!("{PRELUDE}{line}")), outside(3), "{line}");
    }
    assert_eq!(
        failure(&format!("{PRELUDE}c |> why |> depth(-1) |> diff")),
        error(ErrorKind::Type, "-1 is not a non-negative integer", 3, 1)
    );
    assert_eq!(
        failure(&format!("{PRELUDE}def f() => (model |> diff).length\nf()")),
        outside(4)
    );
    assert_eq!(
        failure(&format!(
            "{PRELUDE}class Item\n$Item.seen = ($Item.size |> diff).length\nitem = Item()"
        )),
        outside(5)
    );
    assert_eq!(
        failure(&format!(
            "{PRELUDE}escaped = suppose x = 2: v -> (model |> diff).length\nescaped(1)"
        )),
        outside(4)
    );
    assert_eq!(
        result(&format!(
            "{PRELUDE}escaped = suppose x = 2: v -> (model |> diff).length\nsuppose x = 3: escaped(1)"
        )),
        json!(2.0)
    );
    assert_eq!(
        result(&format!(
            "{PRELUDE}try:\n    c |> diff\ncatch error:\n    out = error\nout"
        )),
        json!(format!("TypeError: {OUTSIDE}"))
    );
    let mut runtime = Runtime::new();
    checked(&mut runtime, PRELUDE);
    assert_eq!(failure_in(&mut runtime, "x = 5\nc |> diff"), outside(2));
    assert_eq!(evaluate(&mut runtime, "[x, c]"), json!([1.0, 2.0]));
    checked(&mut runtime, "suppose x = 4: 1");
    assert_eq!(failure_in(&mut runtime, "x |> diff"), outside(1));
    checked(
        &mut runtime,
        "class Item\n$Item.seen = ($Item.size |> diff).length",
    );
    assert_eq!(failure_in(&mut runtime, "item = Item()"), outside(1));
    assert_eq!(
        evaluate(
            &mut runtime,
            "r = suppose:\n    item = Item()\n    item.size = 3\n    item.seen\n[r, Item.length]"
        ),
        json!([1.0, 0.0])
    );
}

#[test]
fn every_way_out_of_a_block_leaves_no_running_block_behind() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, CHAIN);
    for (source, fails) in [
        ("suppose a = missing: 1", true),
        ("suppose a = 2: missing", true),
        ("suppose a = 2: throw \"STOP\"", true),
        ("suppose a = 2:\n    suppose b = 7: missing", true),
        ("def f(v):\n    suppose a = v: return v\nf(3)", false),
        ("suppose a = 2: (model |> diff).length", false),
    ] {
        assert_eq!(runtime.run(source).is_err(), fails, "{source}");
        assert_eq!(
            failure_in(&mut runtime, "a |> diff"),
            outside(1),
            "{source}"
        );
    }
    assert_eq!(
        evaluate(
            &mut runtime,
            &format!(
                "suppose a = 5:\n    try:\n        suppose b = 7: throw \"INNER\"\n    catch error:\n        pass\n    {CHANGED}"
            )
        ),
        json!(["a", "b", "c"])
    );
    checked(
        &mut runtime,
        "def g(v):\n    suppose a = v: return (model |> diff).length\nlast = g(a)",
    );
    assert_eq!(evaluate(&mut runtime, "last"), json!(0.0));
    checked(&mut runtime, "a = 4");
    assert_eq!(evaluate(&mut runtime, "[a, last]"), json!([4.0, 0.0]));
    checked(
        &mut runtime,
        "def h(v):\n    suppose a = v:\n        if v == 9:\n            throw \"NINE\"\n        return a\nseen = h(a)",
    );
    assert_eq!(failure_in(&mut runtime, "a = 9").1, "NINE");
    assert_eq!(evaluate(&mut runtime, "[a, seen]"), json!([4.0, 4.0]));
    assert_eq!(failure_in(&mut runtime, "a |> diff"), outside(1));
}

#[test]
fn values_compare_by_the_rule_that_decides_propagation() {
    for (prelude, header, expected) in [
        ("n = 1\n", "n = 1.0", json!([])),
        ("n = 1\n", "n = 2", json!(["n"])),
        ("n = 1\n", "n = \"1\"", json!(["n"])),
        ("z = 0\n", "z = -0", json!(["z"])),
        ("z = -0\n", "z = 0", json!(["z"])),
        ("q = 0 / 0\n", "q = 0 / 0", json!([])),
        ("s = \"a\"\n", "s = \"a\"", json!([])),
        ("s = \"a\"\n", "s = \"b\"", json!(["s"])),
        ("f = false\n", "f = false", json!([])),
        ("f = false\n", "f = 0", json!(["f"])),
        ("e = null\n", "e = null", json!([])),
        ("e = null\n", "e = 0", json!(["e"])),
        ("l = [5]\n", "l = 5", json!(["l"])),
        ("l = [1, [2]]\n", "l = [1, [2]]", json!([])),
        ("l = [0 / 0]\n", "l = [0 / 0]", json!([])),
        ("l = [0]\n", "l = [-0]", json!(["l"])),
        ("l = [1, 2]\n", "l.push(3)", json!(["l"])),
        ("d = Date(1000)\n", "d = Date(1000)", json!([])),
        ("d = Date(1000)\n", "d = Date(2000)", json!(["d"])),
        ("r = /ab/\n", "r = /ab/", json!([])),
        ("r = /ab/\n", "r = /ac/", json!(["r"])),
        ("class Item\nk = Item\n", "k = Item", json!([])),
        ("o = {k: 1}\n", "o = {k: 1}", json!(["o"])),
        ("g = v -> v\nn = 1\n", "n = 1", json!([])),
        ("g = v -> v\n", "g = v -> v", json!(["g"])),
        ("g = v -> v\nh = g\n", "h = g", json!([])),
    ] {
        assert_eq!(changed(prelude, header), expected, "{prelude}{header}");
    }
    assert_eq!(
        changed("class Item\nclass Box\nk = Item\n", "k = Box"),
        json!(["k"])
    );
    assert_eq!(
        result("o = {k: 1}\nsuppose o.k = 2: (model |> diff).map(s -> s.holds)"),
        json!([2.0])
    );
    assert_eq!(
        result("o = {k: 1}\nsuppose o.k = 1: (model |> diff).map(s -> s.holds)"),
        json!([1.0])
    );
    assert_eq!(
        result(
            "class Item\npen = Item()\npen.price = 5\nsuppose pen.price = 6: (model |> diff).map(s -> [s.node, s.holds])"
        ),
        json!([["pen.price", 6.0]])
    );
}

#[test]
fn new_deleted_recreated_and_redeclared_nodes() {
    for (prelude, header, expected) in [
        (CHAIN, "fresh = 1", json!(["fresh"])),
        (CHAIN, "delete c", json!([])),
        (CHAIN, "b = 1 + a", json!([])),
        (CHAIN, "b = a + 2", json!(["b", "c"])),
        ("a = 1\nb = a + 1\n", "class Item", json!(["$Item"])),
        ("class Item\n", "class Item", json!([])),
        ("f = 5\n", "def f() => 2", json!(["f"])),
        ("def h() => 1\n", "h = 3", json!(["h"])),
        (
            "def h(v) => v + 1\nk = h(1)\n",
            "def h(v) => v + 1",
            json!([]),
        ),
        (
            "def h(v) => v + 1\nk = h(1)\n",
            "def h(v) => v * 5",
            json!(["k"]),
        ),
        ("a = 1\nt = why a\n", "t = t.value", json!(["t"])),
        ("a = 1\nt = why a\n", "t = 5", json!(["t"])),
        ("a = 1\ndef f() => why a\nt = f()\n", "t = 5", json!(["t"])),
        ("a = 1\nt = why a\n", "u = 1", json!(["u"])),
    ] {
        assert_eq!(changed_by(prelude, header), expected, "{prelude}{header}");
    }
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose:\n    delete c\n    c = b * 2\n    {CHANGED}"
        )),
        json!([])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose:\n    delete c\n    c = 7\n    {CHANGED}"
        )),
        json!(["c"])
    );
    assert_eq!(
        result(&format!(
            "class Item\n$Item.total = $Item.price * 2\npen = Item()\nsuppose pen.price = 4: {CHANGED}"
        )),
        json!(["pen.total", "pen.price"])
    );
    assert_eq!(
        result(
            "class Item\npen = Item()\npen.price = 5\npen.tag = \"a\"\nsuppose pen = Item(): (model |> diff).map(s -> [s.node, s.holds])"
        ),
        json!([["pen.price", null], ["pen.tag", null]])
    );
    assert_eq!(
        result(
            "class Item\npen = Item()\npen.price = 5\nsuppose:\n    pen.price = 6\n    pen = Item()\n    pen.price = 6\n    (model |> diff).map(s -> [s.node, s.holds])"
        ),
        json!([["pen.price", 6.0]])
    );
    assert_eq!(
        result(&format!(
            "class Item\npen = Item()\npen.price = 5\nsuppose:\n    pen = Item()\n    pen.price = 5\n    {CHANGED}"
        )),
        json!([])
    );
    assert_eq!(
        result(&format!(
            "class Item\npen = Item()\npen.price = 5\nsuppose:\n    delete pen.price\n    first = {CHANGED}.value\n    pen.price = 5\n    [first, {CHANGED}]"
        )),
        json!([[], []])
    );
    assert_eq!(
        result(
            "class Maker\nclass Item\npen = Item()\npen.maker = Maker()\npen.maker.name = \"ACME\"\nsuppose pen.maker = Maker(): (model |> diff).map(s -> [s.node, s.holds])"
        ),
        json!([["pen.maker.name", null]])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}flag = 0\nsuppose:\n    if a > 0:\n        flag = 1\n    {CHANGED}"
        )),
        json!(["flag", "if(a>0)"])
    );
}

#[test]
fn nested_blocks_try_savepoints_and_caught_failures() {
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose a = 2:\n    inner = suppose b = 9: {CHANGED}\n    [inner, {CHANGED}]"
        )),
        json!([["b", "c"], ["a", "b", "c", "inner"]])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose a = 2:\n    suppose a = 1: {CHANGED}"
        )),
        json!(["a", "b", "c"])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose a = 2:\n    try:\n        b = 7\n    catch error:\n        pass\n    {CHANGED}"
        )),
        json!(["a", "b", "c"])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose:\n    try:\n        a = 5\n        throw \"UNDO\"\n    catch error:\n        pass\n    {CHANGED}"
        )),
        json!([])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose:\n    a = 5\n    try:\n        a = 1\n        throw \"UNDO\"\n    catch error:\n        pass\n    {CHANGED}"
        )),
        json!(["a", "b", "c"])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose:\n    try:\n        a = 5\n        b = a\n    catch error:\n        pass\n    a = 1\n    {CHANGED}"
        )),
        json!(["b", "c"])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose a = 3:\n    try:\n        suppose b = 1:\n            throw (model |> diff).length\n    catch error:\n        out = error\n    [out, {CHANGED}]"
        )),
        json!([2.0, ["a", "b", "c", "out"]])
    );
}

#[test]
fn standing_selections_follow_the_running_block() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{CHAIN}k = 5"));
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose a = 2:\n    moved = model |> diff\n    only = a |> diff\n    first = moved.map(s -> s.node).value\n    inner = suppose k = 6:\n        [moved.map(s -> s.node), only.map(s -> s.node)]\n    [first, inner, moved.map(s -> s.node), only.map(s -> s.node)]"
        ),
        json!([
            ["a", "b", "c"],
            [["k"], ["a"]],
            ["a", "b", "c", "first", "inner"],
            ["a"]
        ])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose a = 2:\n    only = a |> diff\n    inner = suppose b = 9: only.map(s -> s.node)\n    [inner, only.map(s -> s.node)]"
        ),
        json!([[], ["a"]])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose a = 5:\n    level = 0\n    if (model |> diff).length > 4:\n        level = 1\n    one = level.value\n    extra = 1\n    two = level.value\n    [one, two]"
        ),
        json!([0.0, 1.0])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose a = 5:\n    gone = any s in model |> diff: s.node == \"c\"\n    one = gone.value\n    a = 1\n    [one, gone]"
        ),
        json!([true, false])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose a = 1:\n    moved = c |> why |> diff\n    one = moved.length.value\n    b = 5\n    [one, moved.map(s -> s.node)]"
        ),
        json!([0.0, ["c", "b"]])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "kept = suppose a = 2: model |> diff\na = 8\nkept.map(s -> s.holds)"
        ),
        json!([2.0, 3.0, 6.0])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose a = 3:\n    snap = (model |> diff).value\n    a = 5\n    [snap.map(s -> s.holds), (model |> diff).map(s -> s.holds)]"
        ),
        json!([[3.0, 4.0, 8.0], [5.0, 6.0, 12.0]])
    );
}

#[test]
fn functions_methods_rules_and_quantifiers_use_the_running_block() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{CHAIN}class Tank(level):\n    this.level = level\n    def changed() => (model |> diff).map(s -> s.node)\ntank = Tank(5)\ndef touches(name) => any s in model |> diff: s.node == name\ncount = v -> (model |> diff).length"
        ),
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose tank.level = 7: [tank.changed(), touches(\"tank.level\"), touches(\"a\"), count(0)]"
        ),
        json!([["tank.level"], true, false, 1.0])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose a = 3:\n    $Tank.seen = (model |> diff).length\n    first = tank.seen.value\n    other = Tank(1)\n    [first, tank.seen, other.seen]"
        ),
        json!([3.0, 6.0, 6.0])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose:\n    ok = all s in model |> diff: s.state == \"stated\"\n    first = ok.value\n    a = 2\n    [first, ok]"
        ),
        json!([true, false])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "def plan(v):\n    suppose a = v: return (model |> diff).map(s -> s.node)\n[plan(1), plan(4)]"
        ),
        json!([[], ["a", "b", "c"]])
    );
}

#[test]
fn diff_composes_with_depth_filters_and_walks() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{CHAIN}d = c + 1\nk = 5"));
    let both = |runtime: &mut Runtime, left: &str, right: &str| {
        let first = evaluate(
            runtime,
            &format!("suppose a = 2: ({left}).map(s -> s.node)"),
        );
        let second = evaluate(
            runtime,
            &format!("suppose a = 2: ({right}).map(s -> s.node)"),
        );
        assert_eq!(first, second, "{left} against {right}");
        first
    };
    assert_eq!(
        both(
            &mut runtime,
            "d |> why |> diff |> depth(1)",
            "d |> why |> depth(1) |> diff"
        ),
        json!(["d", "c"])
    );
    assert_eq!(
        both(
            &mut runtime,
            "a |> affects |> diff |> depth(1)",
            "a |> affects |> depth(1) |> diff"
        ),
        json!(["b"])
    );
    assert_eq!(
        both(
            &mut runtime,
            "model |> diff |> stated",
            "model |> stated |> diff"
        ),
        json!(["a"])
    );
    assert_eq!(
        both(
            &mut runtime,
            "model |> diff |> derived",
            "model |> derived |> diff"
        ),
        json!(["b", "c", "d"])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose a = 2: [(d |> why |> diff).map(s -> s.node), (a |> affects |> diff).map(s -> s.node), (model |> diff |> why).map(s -> s.node), (model |> diff |> affects).map(s -> s.node).sort()]"
        ),
        json!([
            ["d", "c", "b", "a"],
            ["b", "c", "d"],
            ["a", "b", "c", "d"],
            ["b", "c", "d"]
        ])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose a = 2: (model |> diff |> depth(0)).length"
        ),
        json!(4.0)
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "suppose k = 6: (model |> diff |> diff).map(s -> s.node)"
        ),
        json!(["k"])
    );
    assert_eq!(
        result(
            "a = 1\nb = a * 0\nc = b + 1\nsuppose a = 2: [(c |> why |> diff |> depth(1)).map(s -> s.node), (c |> why |> depth(1) |> diff).map(s -> s.node), (c |> why |> diff |> depth(2)).map(s -> s.node)]"
        ),
        json!([[], [], ["a"]])
    );
}

#[test]
fn diff_renders_once_and_parses_back() {
    for source in [
        "model |> diff",
        "c |> why |> diff",
        "why c |> diff",
        "c |> diff |> affects |> diff",
        "c |> why |> diff |> depth(1) |> diff",
        "c |> why |> depth(n + 1) |> diff",
        "a || b |> diff",
        "(any s in model |> diff: s.node == \"a\")",
        "model |> diff |> stated",
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
        parse_expression("c |> why |> diff |> depth(1)")
            .unwrap()
            .to_string(),
        "c|>why|>diff|>depth(1)"
    );
    let mut chained = String::from("c");
    for level in 1..=6 {
        chained.push_str(" |> diff");
        let rendered = parse_expression(&chained).unwrap().to_string();
        assert_eq!(rendered.len(), 1 + 6 * level, "{rendered}");
    }
    assert_eq!(
        parse_expression("(c |> diff).length").unwrap().to_string(),
        "c|>diff.length"
    );
    assert_eq!(
        generate(&parse("def f(x = 1) => c |> diff").unwrap()[0]),
        "def f(x=1):
    return c|>diff"
    );
    let condition = parse("if (model |> diff).length > 0:\n    flag = 1").unwrap();
    let Some(Stmt::If {
        condition: test, ..
    }) = condition.first()
    else {
        panic!("{condition:?}");
    };
    assert_eq!(
        NodeKey::conditional(test, None).to_string(),
        "if(model|>diff.length>0)"
    );
    assert_eq!(
        generate(&parse("r = suppose a = 2: model |> diff").unwrap()[0]),
        "r=suppose{a=2;model|>diff}"
    );
    let output = cli(
        "graph",
        &format!("{PRELUDE}suppose x = 2:\n    seen = x |> diff\n    (seen).length"),
    );
    assert!(output.status.success(), "{output:?}");
    let listing = String::from_utf8(output.stdout).unwrap();
    assert!(listing.starts_with("1.0\n"), "{listing}");
    assert!(!listing.contains("seen"), "{listing}");
}

#[test]
fn diff_reads_the_undo_log_once_for_large_blocks() {
    let size = 20_000;
    let mut source = String::new();
    for index in 0..size {
        source.push_str(&format!("v{index} = 0\n"));
    }
    source.push_str("suppose:\n");
    for index in 0..size {
        source.push_str(&format!("    v{index} = {}\n", index % 2));
    }
    source.push_str("    (model |> diff).length\n");
    assert_eq!(result(&source), json!(10_000.0));
}

#[test]
fn near_misses_keep_their_accepted_results() {
    for (line, expected) in [
        (
            "c |> Diff",
            unpositioned(ErrorKind::Type, "Diff is not an operation"),
        ),
        (
            "c |> diffs",
            unpositioned(ErrorKind::Type, "diffs is not an operation"),
        ),
        (
            "c |> dif",
            unpositioned(ErrorKind::Type, "dif is not an operation"),
        ),
        (
            "c | > diff",
            error(ErrorKind::Syntax, "Unexpected character '|'", 3, 3),
        ),
        (
            "c ||> diff",
            error(ErrorKind::Syntax, "Unexpected '>'", 3, 7),
        ),
        (
            "c |> |> diff",
            error(ErrorKind::Syntax, "Expected an operation", 3, 6),
        ),
        (
            "diff c",
            error(ErrorKind::Reference, "diff is not defined", 3, 1),
        ),
        (
            "diff(c)",
            error(ErrorKind::Reference, "diff is not defined", 3, 1),
        ),
        (
            "diff",
            error(ErrorKind::Reference, "diff is not defined", 3, 1),
        ),
        (
            "why diff",
            error(ErrorKind::Reference, "diff is not defined", 3, 1),
        ),
        (
            "c |> why diff",
            error(ErrorKind::Reference, "diff is not defined", 3, 10),
        ),
        (
            "suppose x = 2: diff",
            error(ErrorKind::Reference, "diff is not defined", 3, 1),
        ),
    ] {
        assert_eq!(failure(&format!("{PRELUDE}{line}")), expected, "{line}");
    }
    assert_eq!(result(&format!("{PRELUDE}c.diff")), Json::Null);
    assert_eq!(result(&format!("{PRELUDE}diff = 2\ndiff + 1")), json!(3.0));
    assert_eq!(
        result(&format!("{PRELUDE}def diff(v) => v\ndiff(c)")),
        json!(2.0)
    );
    assert_eq!(
        result(&format!("{PRELUDE}y = {{diff: 1}}\ny.diff")),
        json!(1.0)
    );
}

#[test]
fn inputs_that_were_errors_now_report_what_the_proposal_lists() {
    let step =
        json!({"from": ["x"], "holds": 6.0, "node": "c", "rule": "c = x+1", "state": "derived"});
    assert_eq!(
        result(&format!("{PRELUDE}suppose x = 5: c |> diff")),
        json!([step])
    );
    assert_eq!(
        result(&format!("{PRELUDE}suppose c |> diff:\n    1")),
        json!(1.0)
    );
    assert_eq!(
        result(&format!("{PRELUDE}suppose x = 5:\n    q, r = c |> diff")),
        Json::Null
    );
    for (line, expected) in [
        ("c |> diff", outside(3)),
        ("c|>diff", outside(3)),
        ("model |> diff", outside(3)),
        ("c |> why |> diff", outside(3)),
        ("c |> diff(1)", outside(3)),
        ("c |> diff[0]", outside(3)),
        ("c |> diff x", outside(3)),
        (
            "c |> diff()",
            error(ErrorKind::Syntax, "Unexpected ')'", 3, 12),
        ),
        (
            "c |> diff.length",
            error(ErrorKind::Syntax, "Unexpected '.'", 3, 11),
        ),
        (
            "1 |> diff",
            error(ErrorKind::Type, "Cannot explain 1", 3, 1),
        ),
        (
            "missing |> diff",
            error(ErrorKind::Reference, "missing is not defined", 3, 1),
        ),
        ("diff = 2\nc |> diff", shadowed(4)),
        ("{node} = c |> diff", outside(3)),
        ("q, r = c |> diff", outside(3)),
        (
            "[p] = c |> diff",
            error(ErrorKind::Syntax, "Cannot assign to [p]", 3, 1),
        ),
        (
            "delete c |> diff",
            error(ErrorKind::Syntax, "Cannot delete c|>diff", 3, 1),
        ),
    ] {
        assert_eq!(failure(&format!("{PRELUDE}{line}")), expected, "{line}");
    }
}

#[test]
fn the_native_repl_runs_diff_stages_line_by_line() {
    let output = repl(&format!(
        "{PRELUDE}suppose x = 2: (c |> diff).length\nc |> diff\nsuppose x = 5: (model |> diff).length\n"
    ));
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{BANNER}1.0\n2.0\n1.0\n2.0\n")
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        format!("TypeError: {OUTSIDE} (line 1, column 1)\n")
    );
}
