mod common;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::{NodeKey, ShapeKey};
use nucleoid::lang::ast::generator::generate;
use nucleoid::lang::ast::{Expr, ObjectEntry, Stmt, parse};
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};
use serde_json::{Value as Json, json};

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid copies object entries into records with spread entries",
    "Nucleoid keeps spread entries live as their sources change",
    "Nucleoid merges records with several spread entries",
    "Nucleoid spreads instances into records and class level rules",
    "Nucleoid spreads entries in functions methods and lambdas",
    "Nucleoid spreads only objects and evaluates each source once",
    "Nucleoid leaves the declared property out of spreads of its own object",
    "Nucleoid rolls back failed spread updates",
];

const REFERENCE_TITLE: &str = "Nucleoid derives records with spread entries";
const README_TITLE: &str = "Spread entries keep derived listings in step with their products";
const EXAMPLES_TITLE: &str = "Spread entries keep shipping labels in step with their customers";

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

const ENTRIES: &str =
    "def entries(o):\n    out = []\n    for k, v of o: out.push(k + \"=\" + v)\n    return out";

const COUNTER: &str = "calls = []\ndef note(o):\n    calls.push(1)\n    return o";

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "object spread entry regression".to_string(),
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

fn failure(source: &str) -> (ErrorKind, String, Position) {
    failure_in(&mut Runtime::new(), source)
}

fn failure_in(runtime: &mut Runtime, source: &str) -> (ErrorKind, String, Position) {
    let error = runtime.run(source).expect_err("the program must fail");
    (
        error.kind(),
        error.message(),
        error.position().expect("errors are positioned"),
    )
}

fn error(
    kind: ErrorKind,
    message: &str,
    line: usize,
    column: usize,
) -> (ErrorKind, String, Position) {
    (kind, message.to_string(), Position::new(line, column))
}

fn spreads(source: &str) -> usize {
    let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    format!("{statements:?}").matches("Spread(").count()
}

fn value_of(source: &str) -> Expr {
    match parse(source).unwrap().pop() {
        Some(Stmt::Assign { value, .. }) => value,
        other => panic!("{source} ends with {other:?}"),
    }
}

fn single(source: &str) -> Stmt {
    let [statement]: [Stmt; 1] = parse(source)
        .unwrap()
        .try_into()
        .unwrap_or_else(|statements| panic!("{source} parses to {statements:?}"));
    statement
}

fn object_id(runtime: &mut Runtime, name: &str) -> ObjectId {
    match runtime.run(name).unwrap() {
        Value::Object(id) => id,
        other => panic!("{name} is {other:?}"),
    }
}

fn observations(runtime: &Runtime, key: &NodeKey) -> Vec<ShapeKey> {
    runtime.graph.shape_dependencies(key).cloned().collect()
}

fn entry_observers(runtime: &Runtime) -> Vec<String> {
    runtime
        .graph
        .nodes()
        .flat_map(|node| {
            runtime
                .graph
                .shape_dependencies(&node.key)
                .filter(|shape| matches!(shape, ShapeKey::Entries(_)))
                .map(move |shape| format!("{} {shape}", node.key))
        })
        .collect()
}

fn sorted(items: impl Iterator<Item = String>) -> Vec<String> {
    let mut items: Vec<String> = items.collect();
    items.sort();
    items
}

fn graph_listing(runtime: &Runtime) -> Vec<String> {
    sorted(runtime.graph.nodes().map(|node| {
        let reads = sorted(node.dependencies.iter().map(ToString::to_string));
        let observes = sorted(
            runtime
                .graph
                .shape_dependencies(&node.key)
                .map(ToString::to_string),
        );
        let updates = sorted(node.dependents.iter().map(ToString::to_string));
        format!(
            "{} [{}] reads {reads:?} observes {observes:?} updates {updates:?}",
            node.key, node.kind
        )
    }))
}

fn why_from(runtime: &mut Runtime, name: &str) -> Json {
    evaluate(runtime, &format!("(why {name})[0].from"))
}

fn named_from(runtime: &mut Runtime, name: &str, objects: &[&str]) -> Vec<String> {
    let prefixes: Vec<(String, &str)> = objects
        .iter()
        .map(|object| (format!("{}.", object_id(runtime, object)), *object))
        .collect();
    why_from(runtime, name)
        .as_array()
        .unwrap()
        .iter()
        .map(|key| {
            let key = key.as_str().unwrap();
            prefixes
                .iter()
                .find_map(|(prefix, object)| {
                    key.strip_prefix(prefix.as_str())
                        .map(|rest| format!("{object}.{rest}"))
                })
                .unwrap_or_else(|| key.to_string())
        })
        .collect()
}

fn cli(label: &str, source: &str) -> Output {
    let directory =
        std::env::temp_dir().join(format!("nucleoid-nuc81-{label}-{}", std::process::id()));
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
            (&reference, REFERENCE_TITLE, Some("12")),
            (&readme, README_TITLE, None),
            (&examples, EXAMPLES_TITLE, Some(r#"["Lyon","69001",3]"#)),
        ]);

    let mut assertions = 0;
    for (cases, title, expected) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(spreads(&case.source) > 0, "{title} writes a spread entry");
        assert_eq!(case.expected.as_deref(), expected, "{title}");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 36);
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Object spread entries\n\n```\n")
        .expect("the language reference documents spread entries");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 81](nuc-0081.md)"));
}

#[test]
fn no_earlier_document_case_writes_a_spread_entry() {
    let mut cases = 0;
    let mut with_spreads = 0;
    for case in documents().iter().flatten() {
        cases += 1;
        let uses = spreads(&case.source) > 0;
        assert_eq!(uses, is_appended(&case.title), "{}", case.title);
        with_spreads += usize::from(uses);
    }
    assert_eq!(with_spreads, 11);
    assert_eq!(cases, 1072);
}

#[test]
fn programs_without_spreads_file_no_entry_observations() {
    let mut programs = 0;
    for case in documents()
        .iter()
        .flatten()
        .filter(|case| !is_appended(&case.title))
    {
        assert_eq!(spreads(&case.source), 0, "{}", case.title);
        let mut runtime = Runtime::new();
        runtime
            .run(&case.source)
            .unwrap_or_else(|error| panic!("{}: {error}", case.title));
        assert_eq!(
            entry_observers(&runtime),
            Vec::<String>::new(),
            "{}",
            case.title
        );
        programs += 1;
    }
    assert_eq!(programs, 1061);
}

#[test]
fn spread_entries_parse_into_their_own_variant() {
    let spread = |name: &str| ObjectEntry::Spread(Expr::Identifier(name.to_string()));
    let property = |key: &str, value: Expr| ObjectEntry::Property(key.to_string(), value);
    let name = |name: &str| Expr::Identifier(name.to_string());

    assert_eq!(
        value_of("b = {...base, price: 3, tag}"),
        Expr::SpreadObject(vec![
            spread("base"),
            property("price", Expr::Number(3.0)),
            property("tag", name("tag")),
        ])
    );
    let expected = Expr::SpreadObject(vec![spread("base"), property("qty", Expr::Number(1.0))]);
    for source in [
        "b = {...base, qty: 1}",
        "b = {...base qty: 1}",
        "b = {...base, qty: 1,}",
        "b = {... base, qty: 1}",
        "b = {\n...base\nqty: 1\n}",
        "b = {...base,\nqty: 1}",
    ] {
        assert_eq!(value_of(source), expected, "{source:?}");
    }
    assert_eq!(
        value_of("b = {tag, ...base stock: 0,}"),
        Expr::SpreadObject(vec![
            property("tag", name("tag")),
            spread("base"),
            property("stock", Expr::Number(0.0)),
        ])
    );
    assert_eq!(
        value_of("b = {...a, ...b}"),
        Expr::SpreadObject(vec![spread("a"), spread("b")])
    );

    let plain = value_of("b = {x: 1, y}");
    assert_eq!(
        plain,
        Expr::ObjectLiteral(vec![
            ("x".to_string(), Expr::Number(1.0)),
            ("y".to_string(), name("y")),
        ])
    );
    assert!(format!("{plain:?}").starts_with("ObjectLiteral("));
    assert_eq!(value_of("b = {}"), Expr::ObjectLiteral(Vec::new()));
    assert_eq!(
        value_of("b = {\"...\": 1}"),
        Expr::ObjectLiteral(vec![("...".to_string(), Expr::Number(1.0))])
    );

    let Expr::SpreadObject(entries) =
        value_of("b = {...shop.items(), ...a.b ?? c, ...{...d}, k: 1}")
    else {
        panic!("a literal with spreads is a spread object");
    };
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.value().to_string())
            .collect::<Vec<_>>(),
        ["shop.items()", "(a.b??c)", "{...d}", "1"]
    );
}

#[test]
fn spreads_write_objects_wherever_an_expression_can() {
    assert_eq!(
        result("a = {x: 1}\n({...a, y: 2})"),
        json!({"x": 1.0, "y": 2.0})
    );
    assert_eq!(
        result("a = {x: 1}\ndef f p => {...p, y: 2}\nf(a)"),
        json!({"x": 1.0, "y": 2.0})
    );
    assert_eq!(
        result("ps = [{x: 1}]\nps.map(lambda p: {...p, z: 0})"),
        json!([{"x": 1.0, "z": 0.0}])
    );
    assert_eq!(
        result("ps = [{x: 1}]\nps.map(p -> ({...p, z: 0}))"),
        json!([{"x": 1.0, "z": 0.0}])
    );
    assert_eq!(
        result("a = {x: 1}\n[{...a}, {...a, n: 2}]"),
        json!([{"x": 1.0}, {"x": 1.0, "n": 2.0}])
    );
    assert_eq!(result("a = {x: 1}\n`v=${({...a, y: 2}).y}`"), json!("v=2"));
    assert_eq!(
        result("a = {x: 1}\nb = {inner: {...a, y: 1}}\nb.inner"),
        json!({"x": 1.0, "y": 1.0})
    );
    assert_eq!(
        result("def f(o) => o\na = {x: 1}\nf({...a})"),
        json!({"x": 1.0})
    );
    assert_eq!(
        result("def f(o):\n    return {...o, done: true}\nf({n: 1})"),
        json!({"n": 1.0, "done": true})
    );
    assert_eq!(
        result(
            "a = {x: 1}\nout = 0\ntry:\n    out = {...a, ok: true}\ncatch error:\n    out = 1\nout"
        ),
        json!({"x": 1.0, "ok": true})
    );
}

#[test]
fn entries_copy_in_insertion_order_and_later_entries_win_in_place() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ENTRIES}\npoint = {{x: 1, y: 2}}\nmoved = {{...point, x: 5}}\nfloor = {{x: 9, ...point}}\nwide = {{z: 0, ...point, x: 7, w: 1}}\nmerged = {{...point, ...{{y: 3, q: 4}}}}"
        ),
    );
    for (name, expected) in [
        ("moved", json!(["x=5", "y=2"])),
        ("floor", json!(["x=1", "y=2"])),
        ("wide", json!(["z=0", "x=7", "y=2", "w=1"])),
        ("merged", json!(["x=1", "y=3", "q=4"])),
    ] {
        assert_eq!(
            evaluate(&mut runtime, &format!("entries({name})")),
            expected,
            "{name}"
        );
    }

    assert_eq!(
        result(&format!(
            "{ENTRIES}\nnone = {{}}\nrec = {{a: null, b: none.missing, c: 1}}\ncopy = {{...rec}}\nentries(copy)"
        )),
        json!(["a=null", "c=1"])
    );
    assert_eq!(
        result(&format!(
            "{ENTRIES}\nclass Room(name, width):\n    this.name = name\n    this.width = width\nlab = Room(\"lab\", 3)\nlab.lamp = \"on\"\ndelete lab.name\nlab.name = \"studio\"\ncopy = {{...lab}}\nentries(copy)"
        )),
        json!(["id=lab", "width=3", "lamp=on", "name=studio"])
    );
    assert_eq!(
        result(
            "inner = {n: 1}\nouter = {inner: inner, k: 1}\ncopy = {...outer}\ncopy.inner.n = 5\n[inner.n, outer.inner.n]"
        ),
        json!([5.0, 5.0])
    );
}

#[test]
fn copied_nulls_stay_values_while_null_sources_wait() {
    assert_eq!(
        result(
            "defaults = {size: null, gift: false}\norder = {...defaults, size: 3}\nplain = {...defaults}\nexplicit = {size: defaults.size, gift: defaults.gift}\n[order, plain, explicit]"
        ),
        json!([{"size": 3.0, "gift": false}, {"size": null, "gift": false}, null])
    );
    assert_eq!(
        result(
            "owner = null\ncard = {...owner, seen: true}\nfirst := card\nowner = {name: \"ada\"}\n[first, card]"
        ),
        json!([null, {"name": "ada", "seen": true}])
    );
    assert_eq!(result("b = {...null, y: 1}\nb"), json!({"y": 1.0}));
    assert_eq!(
        result(
            "box = {}\npacked = {...box.lid, open: false}\nfirst := packed\nbox.lid = {color: \"tan\"}\nsecond := packed\ndelete box.lid\n[first, second, packed]"
        ),
        json!([
            {"open": false},
            {"color": "tan", "open": false},
            {"open": false}
        ])
    );
}

#[test]
fn other_sources_raise_type_errors_at_the_running_statement() {
    for (setup, source, message) in [
        ("sizes = [1, 2]", "sizes", "sizes is not an object"),
        ("x = 1", "\"ab\"", "\"ab\" is not an object"),
        ("text = \"ab\"", "text", "text is not an object"),
        ("n = 3", "n", "n is not an object"),
        ("flag = true", "flag", "flag is not an object"),
        ("class Item", "Item", "Item is not an object"),
        ("def f() => 1", "f", "f is not an object"),
        ("d = Date(0)", "d", "d is not an object"),
        ("r = /a/", "r", "r is not an object"),
        ("x = 1", "[1, 2]", "[1,2] is not an object"),
        ("a = {x: 1}", "a.x", "a.x is not an object"),
        ("class Item", "Item.list()", "Item.list() is not an object"),
    ] {
        let program = format!("{setup}\nb = {{...{source}, k: 1}}");
        assert_eq!(
            failure(&program),
            error(ErrorKind::Type, message, 2, 1),
            "{program}"
        );
    }
    assert_eq!(
        failure("def f(o) => {...o}\nb = f([1])"),
        error(ErrorKind::Type, "o is not an object", 2, 1)
    );
    assert_eq!(
        failure("b = {...missing}"),
        error(ErrorKind::Reference, "missing is not defined", 1, 1)
    );
    assert_eq!(
        result(
            "xs = [1]\nfailure = \"\"\ntry:\n    b = {...xs}\ncatch error:\n    failure = error\nfailure"
        ),
        json!("TypeError: xs is not an object")
    );

    let mut runtime = Runtime::new();
    runtime.run("count = 1\nlist = [1]").unwrap();
    assert_eq!(
        failure_in(&mut runtime, "count = 2\nb = {...list}"),
        error(ErrorKind::Type, "list is not an object", 2, 1)
    );
    assert_eq!(runtime.run("count").unwrap(), Value::Number(1.0));
    assert_eq!(entry_observers(&runtime), Vec::<String>::new());
}

#[test]
fn every_object_is_a_source_and_class_rules_spread_their_instance() {
    assert_eq!(
        result("def table() => {a: 1, b: 2}\n({...table(), c: 3})"),
        json!({"a": 1.0, "b": 2.0, "c": 3.0})
    );
    assert_eq!(
        result(
            "class Shop(tea):\n    this.tea = tea\n    def stock():\n        return {tea: this.tea}\nshop = Shop(4)\n({...shop.stock(), jam: 2})"
        ),
        json!({"tea": 4.0, "jam": 2.0})
    );
    assert_eq!(
        result(
            "rec = {a: 1}\nstep = {...(why rec)[0], extra: 1}\nkeys = []\nfor k, v of step: keys.push(k)\nkeys"
        ),
        json!(["node", "holds", "rule", "state", "from", "extra"])
    );
    assert_eq!(
        result("a = {x: 1}\ncopy = suppose:\n    a.x = 2\n    a\nb = {...copy, y: 1}\n[b, a]"),
        json!([{"x": 2.0, "y": 1.0}, {"x": 1.0}])
    );
    assert_eq!(
        result(
            "class Lab(name):\n    this.name = name\n$Lab.card = {...$Lab, unit: \"C\"}\nlab = Lab(\"lab\")\nlab.floor = 2\nlab.card"
        ),
        json!({"id": "lab", "name": "lab", "floor": 2.0, "unit": "C"})
    );
}

#[test]
fn each_source_is_evaluated_once_from_left_to_right() {
    assert_eq!(
        result(
            "calls = []\ndef table(tag):\n    calls.push(tag)\n    return {a: tag}\nb = {...table(\"x\"), ...table(\"y\"), n: 1}\nseen := calls\n[seen, b]"
        ),
        json!([["x", "y"], {"a": "y", "n": 1.0}])
    );
    assert_eq!(
        result(
            "class Tag(label):\n    this.label = label\nmade = {...Tag(\"new\"), qty: 1}\n[Tag.length, made.label, made.qty]"
        ),
        json!([1.0, "new", 1.0])
    );
}

#[test]
fn a_declaration_leaves_its_own_property_out_of_spreads_of_its_object() {
    let program = |body: &str| result(&format!("{ENTRIES}\n{body}"));
    assert_eq!(
        program(
            "cart = {tea: 1, jam: 2}\ncart.copy = {...cart, n: 1}\ncart.oat = 3\nentries(cart.copy)"
        ),
        json!(["tea=1", "jam=2", "oat=3", "n=1"])
    );
    assert_eq!(
        program(
            "class Order(total, items):\n    this.total = total\n    this.items = items\n$Order.view = {...$Order, paid: false}\norder = Order(10, 2)\norder.total = 12\norder.total = 14\nentries(order.view)"
        ),
        json!(["id=order", "total=14", "items=2", "paid=false"])
    );
    assert_eq!(
        result("class Box(w):\n    this.w = w\n    this.copy = {...this}\nbox = Box(3)\nbox.copy"),
        json!({"id": "box", "w": 3.0})
    );
    for (suite, expected) in [
        (
            "{\n    cart.copy = {...cart, n: 1}\n}\ncart.oat = 3",
            json!({"tea": 1.0, "jam": 2.0, "oat": 3.0, "n": 1.0}),
        ),
        (
            "if cart.tea > 0:\n    cart.copy = {...cart, n: 1}\ncart.oat = 3",
            json!({"tea": 1.0, "jam": 2.0, "oat": 3.0, "n": 1.0}),
        ),
        (
            "for c of \"a\":\n    cart.copy = {...cart, n: c}\nfor c of \"b\":\n    cart.copy = {...cart, n: c}",
            json!({"tea": 1.0, "jam": 2.0, "n": "b"}),
        ),
        (
            "n = 0\nwhile n < 2:\n    n += 1\n    cart.copy = {...cart, n: n}",
            json!({"tea": 1.0, "jam": 2.0, "n": 2.0}),
        ),
        (
            "cart.copy := {...cart}\ncart.copy := {...cart, n: 1}",
            json!({"tea": 1.0, "jam": 2.0, "n": 1.0}),
        ),
    ] {
        assert_eq!(
            result(&format!("cart = {{tea: 1, jam: 2}}\n{suite}\ncart.copy")),
            expected,
            "{suite}"
        );
    }
    assert_eq!(
        result(
            "class Order(total):\n    this.total = total\ndef view o => {...o, paid: false}\n$Order.view = view($Order)\norder = Order(10)\norder.total = 12\norder.view"
        ),
        json!({"id": "order", "total": 12.0, "paid": false})
    );
    assert_eq!(
        result(
            "class Cart(tea):\n    this.tea = tea\n    def snapshot():\n        return {...this}\ncart = Cart(1)\ncart.copy = cart.snapshot()\ncart.tea = 2\ncart.copy"
        ),
        json!({"id": "cart", "tea": 2.0})
    );
    assert_eq!(
        result("o = {n: 1}\nset = v -> (o.v = {...o})\nset(1)\nset(2)\no.v"),
        json!({"n": 1.0, "v": {"n": 1.0}})
    );
}

#[test]
fn spreads_depend_on_the_entries_the_explicit_record_names() {
    let mut spread = Runtime::new();
    checked(
        &mut spread,
        "base = {name: \"tea\", price: 4}\nitem = {...base, qty: 2}\nline = {...item, total: item.price * item.qty}\nmoved = {...base, price: 9}",
    );
    let mut explicit = Runtime::new();
    checked(
        &mut explicit,
        "base = {name: \"tea\", price: 4}\nitem = {name: base.name, price: base.price, qty: 2}\nline = {name: item.name, price: item.price, qty: item.qty, total: item.price * item.qty}\nmoved = {name: base.name, price: 9}",
    );
    for name in ["item", "line", "moved"] {
        assert_eq!(
            named_from(&mut spread, name, &["base", "item"]),
            named_from(&mut explicit, name, &["base", "item"]),
            "{name}"
        );
    }
    assert_eq!(
        named_from(&mut spread, "line", &["base", "item"]),
        ["item", "item.name", "item.price", "item.qty"]
    );
    let base = object_id(&mut spread, "base");
    assert_eq!(
        why_from(&mut spread, "moved"),
        json!(["base", NodeKey::property(&base, "name").as_str()])
    );
    assert_eq!(
        observations(&spread, &NodeKey::variable("item")),
        [ShapeKey::Entries(base.clone())]
    );
    for source in [
        "(affects base).map(step -> step.node)",
        "(model |> why).map(step -> step.node)",
    ] {
        assert_eq!(
            evaluate(&mut spread, source),
            evaluate(&mut explicit, source),
            "{source}"
        );
    }

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "base = {name: \"tea\", price: 4}\nitem = {...base, qty: 2}",
    );
    for (change, expected) in [
        (
            "base.price = 5",
            json!({"name": "tea", "price": 5.0, "qty": 2.0}),
        ),
        (
            "base.size = \"L\"",
            json!({"name": "tea", "price": 5.0, "size": "L", "qty": 2.0}),
        ),
        (
            "delete base.name",
            json!({"price": 5.0, "size": "L", "qty": 2.0}),
        ),
        (
            "base = {name: \"jam\", price: 3}",
            json!({"name": "jam", "price": 3.0, "qty": 2.0}),
        ),
    ] {
        checked(&mut runtime, change);
        assert_eq!(evaluate(&mut runtime, "item"), expected, "{change}");
    }
    let rebound = object_id(&mut runtime, "base");
    assert_eq!(
        observations(&runtime, &NodeKey::variable("item")),
        [ShapeKey::Entries(rebound)]
    );

    for program in [
        "a = {x: 1}\nb = {...a, y: 2}\nb.extra = 1\na.x = 2\nb.extra",
        "a = {x: 1}\nb = {x: a.x, y: 2}\nb.extra = 1\na.x = 2\nb.extra",
    ] {
        assert_eq!(result(program), json!(null), "{program}");
    }
    assert_eq!(
        result(
            "a = {x: 1}\nb = {...a.value, y: 2}\na.x = 5\na.z = 3\nfirst := b\na = {q: 9}\n[first, b]"
        ),
        json!([
            {"x": 5.0, "z": 3.0, "y": 2.0},
            {"x": 5.0, "z": 3.0, "y": 2.0}
        ])
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = {x: 1}\nb := {...a, y: 2}\na.x = 5\na.z = 3",
    );
    assert_eq!(evaluate(&mut runtime, "b"), json!({"x": 1.0, "y": 2.0}));
    assert_eq!(entry_observers(&runtime), Vec::<String>::new());
}

#[test]
fn a_replaced_entry_is_not_a_dependency_and_wakes_nothing() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{COUNTER}\na = {{x: 1, y: 2}}\nb = {{...note(a), x: 5}}"),
    );
    let a = object_id(&mut runtime, "a");
    let from = why_from(&mut runtime, "b");
    assert!(
        !from
            .as_array()
            .unwrap()
            .contains(&json!(NodeKey::property(&a, "x").as_str())),
        "{from}"
    );
    checked(&mut runtime, "a.x = 9");
    assert_eq!(
        evaluate(&mut runtime, "[calls.length, b]"),
        json!([1.0, {"x": 5.0, "y": 2.0}])
    );
    checked(&mut runtime, "a.y = 3");
    assert_eq!(
        evaluate(&mut runtime, "[calls.length, b]"),
        json!([2.0, {"x": 5.0, "y": 3.0}])
    );
}

#[test]
fn a_spread_of_its_own_object_observes_it_without_waking_itself() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, "a = {x: 1}\na.b = {...a}\na.y = 2");
    let a = object_id(&mut runtime, "a");
    let b = NodeKey::property(&a, "b");
    assert_eq!(
        evaluate(
            &mut runtime,
            "names = []\nfor k, v of a: names.push(k)\n[names, a.b]"
        ),
        json!([["x", "b", "y"], {"x": 1.0, "y": 2.0}])
    );
    assert_eq!(
        why_from(&mut runtime, "a.b"),
        json!([
            "a",
            NodeKey::property(&a, "x").as_str(),
            NodeKey::property(&a, "y").as_str()
        ])
    );
    assert_eq!(observations(&runtime, &b), [ShapeKey::Entries(a.clone())]);
    assert_eq!(entry_observers(&runtime), [format!("{b} {{...{a}}}")]);

    let steps = [
        ("a.y = 2", 2.0),
        ("a.x = 5", 3.0),
        ("a.y = 2", 4.0),
        ("delete a.y", 5.0),
        ("a.z = 3", 6.0),
    ];
    for declaration in ["a.b = {...note(a)}", "a.b = {x: note(a).x, y: a.y, z: a.z}"] {
        let mut runtime = Runtime::new();
        checked(
            &mut runtime,
            &format!("{COUNTER}\na = {{x: 1}}\n{declaration}"),
        );
        assert_eq!(
            evaluate(&mut runtime, "calls.length"),
            json!(1.0),
            "{declaration}"
        );
        for (step, calls) in steps {
            checked(&mut runtime, step);
            assert_eq!(
                evaluate(&mut runtime, "calls.length"),
                json!(calls),
                "{declaration}: {step}"
            );
        }
        assert_eq!(
            evaluate(&mut runtime, "a.b"),
            json!({"x": 5.0, "z": 3.0}),
            "{declaration}"
        );
    }

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{COUNTER}\na = {{x: 1}}\na.b = {{...note(a)}}"),
    );
    let additions: Vec<String> = (0..100)
        .map(|index| format!("a.k{index} = {index}"))
        .collect();
    checked(&mut runtime, &additions.join("\n"));
    assert_eq!(
        evaluate(&mut runtime, "[calls.length, a.b.k99]"),
        json!([101.0, 99.0])
    );
    checked(&mut runtime, "delete a.b\na.w = 1");
    assert_eq!(evaluate(&mut runtime, "calls.length"), json!(101.0));
    assert_eq!(entry_observers(&runtime), Vec::<String>::new());
}

#[test]
fn the_graph_listing_shows_each_entry_observation() {
    let output = cli("self", "a = {x: 1}\na.b = {...a}\na.y = 2\na.b");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "{\"x\":1.0,\"y\":2.0}\n\nlogic graph (4 nodes)\n  a [variable]\n    updates  obj:0.b\n  obj:0.b [property]\n    reads    a, obj:0.x, obj:0.y\n    observes {...obj:0}\n  obj:0.x [pending]\n    updates  obj:0.b\n  obj:0.y [property]\n    updates  obj:0.b\n"
    );

    let output = cli("plain", "a = {x: 1}\na.b = {x: a.x, y: a.y}\na.y = 2\na.b");
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("observes"), "{stdout}");
}

#[test]
fn rules_that_spread_one_receiver_are_circular_and_roll_back() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class P(n):\n    this.n = n\n$P.a = {...$P}\n$P.b = {...$P}",
    );
    let before = graph_listing(&runtime);
    assert_eq!(
        failure_in(&mut runtime, "p = P(1)"),
        error(ErrorKind::Type, "Circular Dependency", 1, 1)
    );
    assert_eq!(graph_listing(&runtime), before);
    assert_eq!(evaluate(&mut runtime, "P.length"), json!(0.0));
    assert_eq!(failure_in(&mut runtime, "p").0, ErrorKind::Reference);
    assert_eq!(entry_observers(&runtime), Vec::<String>::new());

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class P(n):\n    this.n = n\np = P(1)\n$P.a = {...$P, k: 1}",
    );
    let before = graph_listing(&runtime);
    let instance = evaluate(&mut runtime, "p");
    assert_eq!(
        failure_in(&mut runtime, "$P.b = {...$P}"),
        error(ErrorKind::Type, "Circular Dependency", 1, 1)
    );
    assert_eq!(graph_listing(&runtime), before);
    assert_eq!(evaluate(&mut runtime, "p"), instance);
    checked(&mut runtime, "p.n = 2\np.m = 3");
    assert_eq!(
        evaluate(&mut runtime, "p.a"),
        json!({"id": "p", "n": 2.0, "m": 3.0, "k": 1.0})
    );

    assert_eq!(
        result(
            "class P(n):\n    this.n = n\n$P.a = {...$P}\n$P.b = {...$P}\nfailure = \"\"\ntry:\n    p = P(1)\ncatch error:\n    failure = error\n[failure, P.length]"
        ),
        json!(["TypeError: Circular Dependency", 0.0])
    );
    assert_eq!(
        failure(
            "class P(n):\n    this.n = n\n$P.a = {n: $P.n, b: $P.b}\n$P.b = {n: $P.n, a: $P.a}"
        ),
        error(ErrorKind::Type, "Circular Dependency", 4, 1)
    );
}

#[test]
fn spreads_render_once_in_keys_conditions_and_rules() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = {x: 1}\nb = {...{...{...a, k: 1}, j: 2}, i: 3}\n{\n    d = {...a, y: a.x}\n}\nif ({...a, w: 2}).w > 1:\n    m = {...a, big: true}\nh = (o, n = 1) => ({...o, n: n})",
    );
    assert_eq!(
        evaluate(&mut runtime, "(why b)[0].rule"),
        json!("b = {...{...{...a,\"k\":1},\"j\":2},\"i\":3}")
    );
    assert_eq!(
        evaluate(&mut runtime, "(why h)[0].rule"),
        json!("h = (o,n=1)=>{...o,\"n\":n}")
    );
    let keys: Vec<&str> = runtime
        .graph
        .nodes()
        .map(|node| node.key.as_str())
        .collect();
    for key in ["block(d={...a,\"y\":a.x})", "if({...a,\"w\":2}.w>1)"] {
        assert!(keys.contains(&key), "{key} in {keys:?}");
    }
    assert_eq!(
        generate(&single("def f(o, n = 1) => {...o, n: n}")),
        "def f(o,n=1):\n    return {...o,\"n\":n}"
    );

    for source in [
        "{...{...{...a, k: 1}, j: 2}, i: 3}",
        "{...a, \"y\": a.x}",
        "{tag, ...base stock: 0,}",
        "{...shop.items(), ...a.b ?? c, k: [1, {...d}]}",
    ] {
        let literal = value_of(&format!("b = {source}"));
        let rendered = literal.to_string();
        assert_eq!(value_of(&format!("b = {rendered}")), literal, "{rendered}");
    }
}

#[test]
fn rendered_size_grows_linearly_with_nested_spreads() {
    let nested = |depth: usize, inner: &str| {
        (0..depth).fold(inner.to_string(), |source, _| {
            format!("{{...{source}, k: 1}}")
        })
    };
    let mut sizes: [Vec<usize>; 4] = Default::default();
    for depth in 1..=40 {
        let literal = nested(depth, "a");
        let mut runtime = Runtime::new();
        checked(
            &mut runtime,
            &format!(
                "a = {{x: 1}}\nb = {literal}\n{{\n    c = {literal}\n}}\nif ({literal}).k > 0:\n    e = 1"
            ),
        );
        let rule = evaluate(&mut runtime, "(why b)[0].rule");
        sizes[0].push(rule.as_str().unwrap().len());
        let keys: Vec<usize> = runtime
            .graph
            .nodes()
            .map(|node| node.key.as_str())
            .filter(|key| key.starts_with("block(") || key.starts_with("if("))
            .map(str::len)
            .collect();
        let [block, condition] = keys[..] else {
            panic!("one block and one condition are filed: {keys:?}");
        };
        sizes[1].push(block);
        sizes[2].push(condition);
        let function = single(&format!("def f(o, n = 1) => {}", nested(depth, "o")));
        sizes[3].push(generate(&function).len());
    }
    for lengths in &sizes {
        let step = lengths[1] - lengths[0];
        assert_eq!(step, 11, "{lengths:?}");
        for pair in lengths.windows(2) {
            assert_eq!(pair[1] - pair[0], step, "{lengths:?}");
        }
    }
}

#[test]
fn deeply_nested_spreads_stop_at_the_nesting_guard() {
    let literal = (0..10_000).fold("a".to_string(), |source, _| format!("{{...{source}}}"));
    let error = parse(&format!("b = {literal}")).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(error.message(), "Expressions are nested too deeply");
}

#[test]
fn key_set_changes_wake_only_the_spreads_that_observe_them() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "spreads = []\nkeyed = []\nothers = []\ndef spread_tick(o):\n    spreads.push(1)\n    return o\ndef keyed_tick(o):\n    keyed.push(1)\n    return o\ndef other_tick(o):\n    others.push(1)\n    return o\nrec = {a: 1}\nother = {z: 1}\ns = {...spread_tick(rec)}\nk = {a: keyed_tick(rec).a}\nt = {...other_tick(other)}",
    );
    let counts = "[spreads.length, keyed.length, others.length]";
    assert_eq!(evaluate(&mut runtime, counts), json!([1.0, 1.0, 1.0]));
    for (change, expected) in [
        ("rec.b = 2", json!([2.0, 1.0, 1.0])),
        ("delete rec.b", json!([3.0, 1.0, 1.0])),
        ("other.y = 1", json!([3.0, 1.0, 2.0])),
        ("rec.a = 5", json!([4.0, 2.0, 2.0])),
        ("other.y = 2", json!([4.0, 2.0, 3.0])),
    ] {
        checked(&mut runtime, change);
        assert_eq!(evaluate(&mut runtime, counts), expected, "{change}");
    }
    let rec = object_id(&mut runtime, "rec");
    let other = object_id(&mut runtime, "other");
    assert_eq!(
        entry_observers(&runtime),
        [format!("s {{...{rec}}}"), format!("t {{...{other}}}")]
    );
    assert_eq!(
        evaluate(&mut runtime, "(model |> why).map(step -> step.node)")
            .as_array()
            .unwrap()
            .iter()
            .filter(|node| *node == "s" || *node == "t")
            .count(),
        2
    );
}

#[test]
fn chains_and_fan_outs_re_evaluate_each_spread_once() {
    let declarations = 50;
    let lengths = format!(
        "[{}]",
        (0..declarations)
            .map(|index| format!("l{index}.length"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let counted = |index: usize, value: &str| {
        format!(
            "\nl{index} = []\ndef t{index}(o):\n    l{index}.push(1)\n    return o\nc{index} = {value}"
        )
    };
    let chain: String = (0..declarations)
        .map(|index| {
            let source = if index == 0 {
                "a".to_string()
            } else {
                format!("c{}", index - 1)
            };
            counted(index, &format!("{{...t{index}({source})}}"))
        })
        .collect();
    let fan: String = (0..declarations)
        .map(|index| counted(index, &format!("{{...t{index}(a), n: {index}}}")))
        .collect();

    for (declared, last) in [
        (chain, json!({"x": 1.0, "y": 2.0})),
        (fan, json!({"x": 1.0, "y": 2.0, "n": 49.0})),
    ] {
        let mut runtime = Runtime::new();
        checked(&mut runtime, &format!("a = {{x: 1}}{declared}"));
        assert_eq!(
            evaluate(&mut runtime, &lengths),
            json!(vec![1.0; declarations])
        );
        checked(&mut runtime, "a.y = 2");
        assert_eq!(
            evaluate(&mut runtime, &lengths),
            json!(vec![2.0; declarations])
        );
        assert_eq!(evaluate(&mut runtime, "c49"), last);
    }
}

#[test]
fn spread_cost_stays_linear() {
    let entries = 20_000;
    let declarations = (0..entries)
        .map(|index| format!("big.k{index} = {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!(
        "big = Object()\n{declarations}\ncopy = {{...big, extra: 1}}\nbig.k0 = -1\nbig.last = 1\n[copy.k0, copy.k19999, copy.last, copy.extra]"
    );
    assert_eq!(result(&source), json!([-1.0, 19_999.0, 1.0, 1.0]));
}

#[test]
fn rollback_restores_entries_order_and_observations_together() {
    const SETUP: &str = "src = {a: 1, b: 2, c: 3}\nview = {...src, z: 0}\nother = {q: 1}";
    const CHANGES: &str = "src.a = 9\ndelete src.a\ndelete src.b\nsrc.b = 7\nsrc.d = 4";
    let indented: Vec<String> = CHANGES.lines().map(|line| format!("    {line}")).collect();
    let indented = indented.join("\n");
    let view = NodeKey::variable("view");

    let restored = |runtime: &mut Runtime| {
        let src = object_id(runtime, "src");
        assert_eq!(
            evaluate(runtime, "[entries(view), entries(src)]"),
            json!([["a=1", "b=2", "c=3", "z=0"], ["a=1", "b=2", "c=3"]])
        );
        assert_eq!(observations(runtime, &view), [ShapeKey::Entries(src)]);
        assert_eq!(entry_observers(runtime).len(), 1);
    };

    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{ENTRIES}\n{SETUP}"));
    checked(
        &mut runtime,
        &format!("try:\n{indented}\n    throw \"STOP\"\ncatch error:\n    pass"),
    );
    restored(&mut runtime);

    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{ENTRIES}\n{SETUP}"));
    let error = runtime
        .run(&format!("{CHANGES}\nthrow \"STOP\""))
        .unwrap_err();
    assert_eq!(error.thrown_value(), Some(&Value::String("STOP".into())));
    restored(&mut runtime);

    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{ENTRIES}\n{SETUP}"));
    assert_eq!(
        evaluate(
            &mut runtime,
            &format!("seen = suppose:\n{indented}\n    view\nentries(seen)")
        ),
        json!(["c=3", "b=7", "d=4", "z=0"])
    );
    restored(&mut runtime);

    assert_eq!(
        failure_in(&mut runtime, "view = {...other}\nthrow \"STOP\"").0,
        ErrorKind::Thrown
    );
    restored(&mut runtime);
    assert_eq!(
        failure_in(&mut runtime, "fresh = {...src}\nsrc.e = 5\nthrow \"STOP\"").0,
        ErrorKind::Thrown
    );
    restored(&mut runtime);
    assert_eq!(failure_in(&mut runtime, "fresh").0, ErrorKind::Reference);

    checked(&mut runtime, "other.r = 1\nsrc.e = 5");
    assert_eq!(
        evaluate(&mut runtime, "entries(view)"),
        json!(["a=1", "b=2", "c=3", "e=5", "z=0"])
    );
}

#[test]
fn failed_re_evaluations_roll_back_with_the_change_that_caused_them() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ENTRIES}\ncart = {{tea: 4, jam: 3}}\ndef count(o):\n    if o.tea > 9:\n        throw \"TOO_MANY\"\n    return o.tea\norder = {{...cart, count: count(cart)}}"
        ),
    );
    let before = graph_listing(&runtime);
    let error = runtime.run("cart.oats = 2\ncart.tea = 10").unwrap_err();
    assert_eq!(
        error.thrown_value(),
        Some(&Value::String("TOO_MANY".into()))
    );
    assert_eq!(graph_listing(&runtime), before);
    assert_eq!(
        evaluate(&mut runtime, "[entries(cart), entries(order)]"),
        json!([["tea=4", "jam=3"], ["tea=4", "jam=3", "count=4"]])
    );
}

#[test]
fn near_misses_keep_their_accepted_diagnostics() {
    for (line, kind, message, row, column) in [
        (
            "b = {..a}",
            ErrorKind::Syntax,
            "Expected a key but found '.'",
            2,
            7,
        ),
        (
            "b = {. . .a}",
            ErrorKind::Syntax,
            "Expected a key but found '.'",
            2,
            8,
        ),
        (
            "b = {...5}",
            ErrorKind::Syntax,
            "Expected a key but found '.'",
            2,
            7,
        ),
        (
            "b = {y: 2 ...a}",
            ErrorKind::Syntax,
            "Expected a property but found '.'",
            2,
            13,
        ),
        (
            "b = {y: 2...a}",
            ErrorKind::Syntax,
            "Expected a property but found '.'",
            2,
            12,
        ),
        (
            "b = {y: 2; ...a}",
            ErrorKind::Syntax,
            "Expected '}' but found '.'",
            2,
            12,
        ),
        ("{...a, y: 2}", ErrorKind::Syntax, "Unexpected '.'", 2, 3),
        (
            "b = {\n  ...a\n}",
            ErrorKind::Syntax,
            "Expected a key but found an indented block",
            3,
            3,
        ),
        (
            "b = {x...}",
            ErrorKind::Syntax,
            "Expected ':' but found '.'",
            2,
            7,
        ),
        ("b = {x: ...a}", ErrorKind::Syntax, "Unexpected '.'", 2, 10),
    ] {
        let source = format!("a = {{x: 1}}\n{line}");
        assert_eq!(
            failure(&source),
            error(kind, message, row, column),
            "{source:?}"
        );
    }
    assert_eq!(
        result("a = {x: 1}\nb = {\"...\": 1}\nb"),
        json!({"...": 1.0})
    );
    for (source, message, row, column) in [
        ("xs = [1]\nys = [...xs, 3]", "Unexpected '.'", 2, 8),
        (
            "ps = [{x: 1, y: 2}]\nps.map(p -> {...p, z: 0})",
            "Unexpected '.'",
            2,
            15,
        ),
        ("{x, ...rest} = p", "Unexpected ','", 1, 5),
        ("xs = [1]\nf(...xs)", "Unexpected '.'", 2, 4),
    ] {
        assert_eq!(
            failure(source),
            error(ErrorKind::Syntax, message, row, column),
            "{source:?}"
        );
    }
}

#[test]
fn inputs_that_were_errors_now_report_what_the_proposal_lists() {
    for (line, expected) in [
        ("b = {...a}", json!({"x": 1.0})),
        ("b = {y: 2, ...a}", json!({"y": 2.0, "x": 1.0})),
        ("b = {...a y: 2}", json!({"x": 1.0, "y": 2.0})),
        ("b = {...a,}", json!({"x": 1.0})),
        ("b = {... a}", json!({"x": 1.0})),
        ("b = {...a\n}", json!({"x": 1.0})),
        ("b = ({...a, y: 2})", json!({"x": 1.0, "y": 2.0})),
    ] {
        let source = format!("a = {{x: 1}}\n{line}\nb");
        assert_eq!(result(&source), expected, "{source:?}");
    }
    assert_eq!(
        result("def f(o) => o\na = {x: 1}\nf({...a})"),
        json!({"x": 1.0})
    );
    assert_eq!(
        result("class P(n):\n    this.n = n\np = P(1)\nb = {...p}\nb"),
        json!({"id": "p", "n": 1.0})
    );
    for (line, kind, message, row, column) in [
        ("b = {....a}", ErrorKind::Syntax, "Unexpected '.'", 2, 10),
        ("b = {...}", ErrorKind::Syntax, "Unexpected '}'", 2, 10),
        (
            "b = {..., y: 2}",
            ErrorKind::Syntax,
            "Unexpected ','",
            2,
            11,
        ),
        ("b = {...: 1}", ErrorKind::Syntax, "Unexpected ':'", 2, 11),
        (
            "b = {...a ...a}",
            ErrorKind::Syntax,
            "Expected a property but found '.'",
            2,
            13,
        ),
        (
            "b = {...a y}",
            ErrorKind::Syntax,
            "Expected '}' but found identifier 'y'",
            2,
            11,
        ),
        (
            "b = {...a: 1}",
            ErrorKind::Syntax,
            "Expected '}' but found ':'",
            2,
            10,
        ),
        (
            "b = {...a,,}",
            ErrorKind::Syntax,
            "Expected a key but found ','",
            2,
            12,
        ),
        (
            "b = {...\na}",
            ErrorKind::Syntax,
            "Unexpected end of line",
            3,
            1,
        ),
        (
            "b = {...a, y}",
            ErrorKind::Reference,
            "y is not defined",
            2,
            1,
        ),
        (
            "b = {...a.x}",
            ErrorKind::Type,
            "a.x is not an object",
            2,
            1,
        ),
    ] {
        let source = format!("a = {{x: 1}}\n{line}");
        assert_eq!(
            failure(&source),
            error(kind, message, row, column),
            "{source:?}"
        );
    }
}

#[test]
fn the_native_repl_runs_spreads_line_by_line() {
    let output = repl("a = {x: 1}\nb = {...a, y: 2}\nb\nb = {...a,\nb = {..a}\na.z = 3\nb\n");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "{BANNER}{{\"x\":1.0}}\n{{\"x\":1.0,\"y\":2.0}}\n{{\"x\":1.0,\"y\":2.0}}\n3.0\n{{\"x\":1.0,\"y\":2.0,\"z\":3.0}}\n"
        )
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "SyntaxError: Expected a key but found end of input (line 2, column 1)\nSyntaxError: Expected a key but found '.' (line 1, column 7)\n"
    );
}

#[test]
fn object_entries_expose_their_expressions() {
    let key = ObjectEntry::Property("k".to_string(), Expr::Number(1.0));
    let spread = ObjectEntry::Spread(Expr::Identifier("a".to_string()));
    assert_eq!(key.value(), &Expr::Number(1.0));
    assert_eq!(spread.value(), &Expr::Identifier("a".to_string()));
    assert_eq!(
        Expr::SpreadObject(vec![spread, key]).to_string(),
        "{...a,\"k\":1}"
    );
}
