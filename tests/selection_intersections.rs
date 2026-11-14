mod common;

use std::collections::HashSet;
use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate;
use nucleoid::lang::ast::{Ast, Expr, SetOperation, Stmt, parse, parse_expression};
use nucleoid::nuc::Nuc;
use nucleoid::reasoning::Stage;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde_json::{Value as Json, json};

const SPEC_TITLES: [&str; 6] = [
    "Nucleoid finds the causes two explanations share with intersect",
    "Nucleoid tells two explanations apart with except",
    "Nucleoid finds what two inputs both move with intersect",
    "Nucleoid composes intersect and except with state filters",
    "Nucleoid fills an empty intersection when two explanations meet",
    "Nucleoid restores intersections after failed changes",
];

const REFERENCE_TITLE: &str = "Nucleoid compares two explanations with intersect and except";
const README_TITLE: &str = "Intersect and except tell apart what two symptoms rest on";
const EXAMPLES_TITLE: &str = "Except shows what a steel delay moves and a glass delay does not";

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

const CHAIN: &str = "a = 1\nb = a + 2\nc = b * 2\n";

const ORDER: &str = "price = 10\nquantity = 3\nrate = 0.2\nsubtotal = price * quantity\ntax = subtotal * rate\nshipping = quantity * 2\n";

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "selection intersection regression".to_string(),
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

fn combines(source: &str) -> usize {
    let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    format!("{statements:?}").matches("Combine {").count()
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

fn identifier(name: &str) -> Box<Expr> {
    Box::new(Expr::Identifier(name.to_string()))
}

fn reason(stage: Stage, source: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Reason { stage, source })
}

fn combine(operation: SetOperation, source: Box<Expr>, other: Box<Expr>) -> Expr {
    Expr::Combine {
        operation,
        source,
        other,
    }
}

fn chain(levels: usize) -> Expr {
    let mut tree = *reason(Stage::Why, identifier("c"));
    for _ in 0..levels {
        tree = combine(
            SetOperation::Intersect,
            reason(Stage::Why, identifier("c")),
            Box::new(tree),
        );
    }
    tree
}

fn resolved(
    runtime: &mut Runtime,
    tree: &Expr,
) -> Result<Json, (ErrorKind, String, Option<Position>)> {
    match Ast::convert(tree).resolve(runtime, &mut Scope::new()) {
        Ok(value) => Ok(json_of(runtime, &value)),
        Err(error) => Err((error.kind(), error.message(), error.position())),
    }
}

fn cli(label: &str, source: &str) -> Output {
    let directory =
        std::env::temp_dir().join(format!("nucleoid-nuc106-{label}-{}", std::process::id()));
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

fn repl(arguments: &[&str], source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(arguments)
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
            if (state >> 33) % 3 == 0 {
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
            (
                &reference,
                REFERENCE_TITLE,
                Some("[\"price\",\"quantity\"]"),
            ),
            (&readme, README_TITLE, None),
            (&examples, EXAMPLES_TITLE, Some("[[\"frame\"],[15,19]]")),
        ]);

    let mut assertions = 0;
    for (cases, title, expected) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(combines(&case.source) > 0, "{title} writes a set stage");
        assert_eq!(case.expected.as_deref(), expected, "{title}");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 39);
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Intersect and except selections\n\n```\n")
        .expect("the language reference documents intersect and except");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 106](nuc-0106.md)"));
}

#[test]
fn no_earlier_document_case_writes_a_set_stage() {
    let mut cases = 0;
    let mut with_stage = 0;
    for case in documents().iter().flatten() {
        cases += 1;
        let uses = combines(&case.source) > 0;
        assert_eq!(uses, is_appended(&case.title), "{}", case.title);
        with_stage += usize::from(uses);
    }
    assert_eq!(with_stage, 9);
    assert_eq!(cases, 1105);
}

#[test]
fn set_stages_parse_into_one_variant() {
    let tree = combine(
        SetOperation::Intersect,
        reason(Stage::Why, identifier("c")),
        reason(Stage::Why, identifier("b")),
    );
    assert_eq!(value_of("x = c |> why |> intersect(b |> why)"), tree);
    assert_eq!(value_of("x = why c |> intersect(why b)"), tree);
    assert_eq!(value_of("x = why c |> intersect (why b)"), tree);
    assert_eq!(value_of("x = c|>why|>intersect(b|>why)"), tree);
    assert_eq!(value_of("x = (why c |> intersect(\n    why b\n))"), tree);
    assert_eq!(parse_expression("why c |> intersect(why b)").unwrap(), tree);
    assert_eq!(
        value_of("x = why c |> except(b)"),
        combine(
            SetOperation::Except,
            reason(Stage::Why, identifier("c")),
            identifier("b")
        )
    );
    assert_eq!(
        value_of("x = why c |> intersect(model)"),
        combine(
            SetOperation::Intersect,
            reason(Stage::Why, identifier("c")),
            Box::new(Expr::Model)
        )
    );
    assert_eq!(
        value_of("x = why c |> intersect((model))"),
        combine(
            SetOperation::Intersect,
            reason(Stage::Why, identifier("c")),
            Box::new(Expr::Model)
        )
    );
    assert_eq!(
        value_of("x = why c |> except(model.p)"),
        combine(
            SetOperation::Except,
            reason(Stage::Why, identifier("c")),
            Box::new(Expr::Member {
                object: identifier("model"),
                property: "p".to_string(),
            })
        )
    );
    assert_eq!(
        value_of("x = why c |> intersect(why b) |> stated"),
        *reason(Stage::Stated, Box::new(tree.clone()))
    );
    assert_eq!(
        value_of("x = intersect(c, b)"),
        Expr::Call {
            callee: identifier("intersect"),
            arguments: vec![
                Expr::Identifier("c".to_string()),
                Expr::Identifier("b".to_string()),
            ],
        }
    );
    assert_eq!(parse("intersect c b").unwrap().len(), 3);
    assert_eq!(Stage::from_name("intersect"), None);
    assert_eq!(Stage::from_name("except"), None);
    assert_eq!(SetOperation::Intersect.as_str(), "intersect");
    assert_eq!(SetOperation::Except.as_str(), "except");
    let operations: HashSet<SetOperation> = [
        SetOperation::Intersect,
        SetOperation::Except,
        SetOperation::Intersect,
    ]
    .into();
    assert_eq!(operations.len(), 2);
    assert_eq!(combines("x = why c |> depth(1)\ny = intersect(c)"), 0);
    assert_eq!(
        combines("x = why c |> intersect(why b |> except(a)) |> except(why a)"),
        3
    );
}

#[test]
fn ordinary_names_called_intersect_and_except_keep_their_meaning() {
    assert_eq!(
        result("intersect = 1\nexcept = 2\n[intersect, except]"),
        json!([1.0, 2.0])
    );
    assert_eq!(
        result("def intersect(x, y) => x + y\nintersect(4, 1)"),
        json!(5.0)
    );
    assert_eq!(result("def except(x) => x * 2\nexcept(4)"), json!(8.0));
    assert_eq!(
        result("x = {intersect: 1, except: 2}\n[x.intersect, x.except]"),
        json!([1.0, 2.0])
    );
    assert_eq!(result("o = {}\no.except = 3\no.except"), json!(3.0));
    assert_eq!(
        result("[1, 2].map(intersect -> intersect * 2)"),
        json!([2.0, 4.0])
    );
    assert_eq!(
        result("o = {f: x -> x + 1}\no.intersect = o.f\no.intersect(2)"),
        json!(3.0)
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}class Sets:\n    def intersect(x) => x + 1\n    def except() => 7\ns = Sets()\n[(why c |> intersect(why b)).length, s.intersect(1), s.except()]"
        )),
        json!([2.0, 2.0, 7.0])
    );
    assert_eq!(
        failure(&format!("{CHAIN}except")),
        error(ErrorKind::Reference, "except is not defined", 4, 1)
    );
    assert_eq!(
        failure(&format!("{CHAIN}intersect(1, 2)")),
        error(ErrorKind::Reference, "intersect is not defined", 4, 1)
    );
    assert_eq!(
        failure(&format!("{CHAIN}intersect a b")),
        error(ErrorKind::Reference, "intersect is not defined", 4, 1)
    );
    assert_eq!(
        failure("try:\n    y = 1\nexcept e:\n    y = 2"),
        error(ErrorKind::Syntax, "Expected 'catch' after 'try'", 3, 1)
    );
}

#[test]
fn forms_without_parentheses_keep_their_parse_time_error() {
    for (stage, name) in [
        ("why c |> intersect", "intersect"),
        ("why c |> except", "except"),
        ("why c |> intersect\n(why b)", "intersect"),
        ("why c |> except\n(why b)", "except"),
        ("why c |> intersect[b]", "intersect"),
        ("why c |> intersect.x", "intersect"),
        ("why c |> intersect{1}", "intersect"),
        ("why c |> Intersect(why b)", "Intersect"),
        ("why c |> intersection(why b)", "intersection"),
        ("why c |> excepts(why b)", "excepts"),
        ("first = 1\nwhy c |> intersect", "intersect"),
    ] {
        assert_eq!(
            failure(&format!("{CHAIN}{stage}")),
            unpositioned(ErrorKind::Type, &format!("{name} is not an operation")),
            "{stage}"
        );
    }
    assert_eq!(
        failure(&format!("{CHAIN}why c |> |> intersect(why b)")),
        error(ErrorKind::Syntax, "Expected an operation", 4, 10)
    );
}

#[test]
fn intersect_and_except_keep_what_the_node_name_spelling_keeps() {
    for seed in 1..=6 {
        let mut runtime = Runtime::new();
        checked(&mut runtime, &graph(seed, 10));
        for x in 0..10 {
            for y in 0..10 {
                for walk in ["why", "affects"] {
                    for (operation, test) in [("intersect", "in"), ("except", "not in")] {
                        let stage = nodes(
                            &mut runtime,
                            &format!("n{x} |> {walk} |> {operation}(n{y} |> {walk})"),
                        );
                        let spelled = nodes(
                            &mut runtime,
                            &format!(
                                "(n{x} |> {walk}).filter(s -> s.node {test} (n{y} |> {walk}).map(t -> t.node))"
                            ),
                        );
                        assert_eq!(stage, spelled, "seed {seed}: n{x} {walk} {operation} n{y}");
                    }
                }
            }
        }
    }
}

#[test]
fn the_filters_select_the_same_steps_in_any_order() {
    for seed in 1..=3 {
        let mut runtime = Runtime::new();
        checked(&mut runtime, &graph(seed, 9));
        for x in 0..9 {
            for y in [0, 4, 8] {
                for operation in ["intersect", "except"] {
                    let other = format!("{operation}(n{y} |> why)");
                    for filter in [
                        "stated", "derived", "depth(1)", "depth(2)", "path(n1)", "path(n0)",
                    ] {
                        let before =
                            nodes(&mut runtime, &format!("n{x} |> why |> {filter} |> {other}"));
                        let after =
                            nodes(&mut runtime, &format!("n{x} |> why |> {other} |> {filter}"));
                        assert_eq!(before, after, "seed {seed}: n{x} {filter} {other}");
                    }
                    let inside = evaluate(
                        &mut runtime,
                        &format!(
                            "suppose n1 = 7:\n    [(n{x} |> why |> diff |> {other}).map(s -> s.node), (n{x} |> why |> {other} |> diff).map(s -> s.node)]"
                        ),
                    );
                    assert_eq!(inside[0], inside[1], "seed {seed}: n{x} diff {other}");
                }
            }
        }
    }
}

#[test]
fn identities_hold_for_the_same_selection_model_and_repeated_stages() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{ORDER}total = subtotal + tax + shipping\n"),
    );
    for source in [
        "why tax",
        "affects quantity",
        "tax |> why |> stated",
        "model |> derived",
    ] {
        let whole = nodes(&mut runtime, source);
        assert_eq!(
            nodes(&mut runtime, &format!("{source} |> intersect({source})")),
            whole,
            "{source}"
        );
        assert_eq!(
            nodes(&mut runtime, &format!("{source} |> except({source})")),
            json!([]),
            "{source}"
        );
        assert_eq!(
            nodes(&mut runtime, &format!("{source} |> intersect(model)")),
            whole,
            "{source}"
        );
        assert_eq!(
            nodes(&mut runtime, &format!("{source} |> except(model)")),
            json!([]),
            "{source}"
        );
        for other in ["why shipping", "affects price"] {
            let once = nodes(&mut runtime, &format!("{source} |> intersect({other})"));
            let twice = nodes(
                &mut runtime,
                &format!("{source} |> intersect({other}) |> intersect({other})"),
            );
            assert_eq!(once, twice, "{source} {other}");
            let removed = nodes(&mut runtime, &format!("{source} |> except({other})"));
            let removed_twice = nodes(
                &mut runtime,
                &format!("{source} |> except({other}) |> except({other})"),
            );
            assert_eq!(removed, removed_twice, "{source} {other}");
            assert_eq!(
                nodes(
                    &mut runtime,
                    &format!("{source} |> intersect({other}) |> except({other})")
                ),
                json!([])
            );
        }
    }
    assert_eq!(
        nodes(
            &mut runtime,
            "why total |> intersect(why tax) |> intersect(why shipping)"
        ),
        nodes(
            &mut runtime,
            "why total |> intersect(why shipping) |> intersect(why tax)"
        )
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why total |> intersect(why tax) |> intersect(why shipping)"
        ),
        json!(["quantity"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why total |> except(why tax) |> except(why shipping)"
        ),
        json!(["total"])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> intersect(why total)"),
        json!(["tax", "subtotal", "rate", "price", "quantity"])
    );
    assert_eq!(
        nodes(&mut runtime, "why total |> intersect(why tax)"),
        json!(["subtotal", "tax", "quantity", "rate", "price"])
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> intersect(why tax)"),
        json!(["quantity"])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> intersect(why shipping)"),
        json!(["quantity"])
    );
}

#[test]
fn kept_steps_keep_the_direction_and_distances_of_their_source() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{ORDER}total = subtotal + tax + shipping\nlabel = shipping + 1\n"),
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> intersect(why subtotal)"),
        json!(["subtotal", "price", "quantity"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why tax |> intersect(why subtotal) |> depth(1)"
        ),
        json!(["subtotal"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why tax |> depth(1) |> intersect(why subtotal)"
        ),
        json!(["subtotal"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why subtotal |> intersect(why tax) |> depth(1)"
        ),
        json!(["subtotal", "price", "quantity"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "affects price |> intersect(affects quantity) |> depth(1)"
        ),
        json!(["subtotal"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "affects price |> intersect(affects quantity) |> depth(2)"
        ),
        json!(["subtotal", "tax", "total"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "affects price |> intersect(affects quantity) |> path(total)"
        ),
        json!(["subtotal", "tax", "total"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "affects price |> path(total) |> intersect(affects quantity)"
        ),
        json!(["subtotal", "tax", "total"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why total |> intersect(why tax) |> path(price)"
        ),
        json!(["subtotal", "tax", "price"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why total |> path(price) |> intersect(why tax)"
        ),
        json!(["subtotal", "tax", "price"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why tax |> intersect(why shipping) |> affects"
        ),
        json!(["subtotal", "shipping", "total", "label", "tax"])
    );
    assert_eq!(
        nodes(&mut runtime, "affects quantity"),
        json!(["subtotal", "shipping", "total", "label", "tax"])
    );
    assert_eq!(
        nodes(&mut runtime, "affects rate |> except(affects price) |> why"),
        json!([])
    );
    assert_eq!(
        nodes(&mut runtime, "why label |> except(why total)"),
        json!(["label"])
    );
    assert_eq!(
        nodes(&mut runtime, "why label |> except(why total) |> why"),
        json!(["label", "shipping", "quantity"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why tax |> intersect(why shipping) |> why |> stated"
        ),
        json!(["quantity"])
    );
    assert_eq!(
        nodes(&mut runtime, "affects quantity |> except(affects price)"),
        json!(["shipping", "label"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "affects quantity |> except(affects price) |> affects"
        ),
        json!(["total", "label"])
    );
}

#[test]
fn every_kind_of_other_selection_selects_as_a_pipeline_source() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}order = {{fee: 2}}\norder.cost = order.fee + quantity\nclass Item(price, qty):\n    this.price = price\n    this.qty = qty\n    def common() => (this.total |> why |> intersect(this.qty)).map(s -> s.node)\n$Item.total = $Item.price * $Item.qty\n$Item.shared = (why $Item.total |> intersect($Item.qty)).map(s -> s.node)\npen = Item(2, 5)\n"
        ),
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> intersect(quantity)"),
        json!(["quantity"])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> except(tax)"),
        json!(["subtotal", "rate", "price", "quantity"])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> intersect(shipping)"),
        json!([])
    );
    let cost = evaluate(&mut runtime, "(order.cost |> why)[0].node");
    assert_eq!(
        nodes(&mut runtime, "why order.cost"),
        json!([cost.clone(), "order", "quantity"])
    );
    assert_eq!(
        nodes(&mut runtime, "why order.cost |> intersect(order.cost)"),
        json!([cost])
    );
    assert_eq!(
        nodes(&mut runtime, "why order.cost |> except(order.cost)"),
        json!(["order", "quantity"])
    );
    assert_eq!(
        nodes(&mut runtime, "why order.cost |> intersect(order.fee)"),
        json!([])
    );
    assert_eq!(
        nodes(&mut runtime, "why order.cost |> intersect(why tax)"),
        json!(["quantity"])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> intersect(model |> stated)"),
        json!(["rate", "price", "quantity"])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> except(model |> stated)"),
        json!(["tax", "subtotal"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why tax |> intersect(shipping |> why |> stated)"
        ),
        json!(["quantity"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why tax |> intersect(why subtotal |> except(why shipping))"
        ),
        json!(["subtotal", "price"])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> except(affects price)"),
        json!(["rate", "price", "quantity"])
    );
    assert_eq!(
        evaluate(&mut runtime, "[pen.common(), pen.shared]"),
        json!([["pen.qty"], ["pen.qty"]])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> intersect(order.missing)"),
        json!([])
    );
    let mut filled = Runtime::new();
    checked(
        &mut filled,
        &format!("{CHAIN}o = {{}}\nkept = why c |> intersect(o.p)\nrest = why c |> except(o.p)\n"),
    );
    assert_eq!(
        evaluate(&mut filled, "[kept, rest.map(s -> s.node)]"),
        json!([[], ["c", "b", "a"]])
    );
    checked(&mut filled, "o.p = 1");
    assert_eq!(
        evaluate(&mut filled, "[kept, rest.map(s -> s.node)]"),
        json!([[], ["c", "b", "a"]])
    );
    checked(&mut filled, "c = o.p * 2");
    let property = evaluate(&mut filled, "(o.p |> why)[0].node");
    assert_eq!(
        nodes(&mut filled, "why c"),
        json!(["c", "o", property.clone()])
    );
    assert_eq!(nodes(&mut filled, "kept"), json!([property]));
    assert_eq!(nodes(&mut filled, "rest"), json!(["c", "o"]));
}

#[test]
fn a_name_that_holds_a_selection_selects_nothing() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{ORDER}causes = why shipping\n"));
    assert_eq!(
        evaluate(&mut runtime, "(causes |> stated).length"),
        json!(0.0)
    );
    assert_eq!(evaluate(&mut runtime, "(why causes).length"), json!(0.0));
    assert_eq!(
        nodes(&mut runtime, "why tax |> intersect(causes)"),
        json!([])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> except(causes)"),
        json!(["tax", "subtotal", "rate", "price", "quantity"])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> intersect(why shipping)"),
        json!(["quantity"])
    );
    assert_eq!(evaluate(&mut runtime, "causes.length"), json!(2.0));
}

#[test]
fn invalid_other_selections_keep_the_diagnostics_of_a_pipeline_source() {
    for (other, expected) in [
        ("1", error(ErrorKind::Type, "Cannot explain 1", 7, 1)),
        (
            "\"price\"",
            error(ErrorKind::Type, "Cannot explain \"price\"", 7, 1),
        ),
        (
            "price + 1",
            error(ErrorKind::Type, "Cannot explain price+1", 7, 1),
        ),
        (
            "price.x",
            error(ErrorKind::Type, "Cannot explain price.x", 7, 1),
        ),
        ("this", error(ErrorKind::Type, "Cannot explain this", 7, 1)),
        (
            "[price]",
            error(ErrorKind::Type, "Cannot explain [price]", 7, 1),
        ),
        (
            "(why price).filter(s -> true)",
            error(
                ErrorKind::Type,
                "Cannot explain price|>why.filter(function())",
                7,
                1,
            ),
        ),
        (
            "missing",
            error(ErrorKind::Reference, "missing is not defined", 7, 1),
        ),
        (
            "ghost.x",
            error(ErrorKind::Reference, "ghost is not defined", 7, 1),
        ),
        (
            "why missing",
            error(ErrorKind::Reference, "missing is not defined", 7, 1),
        ),
        ("1 |> why", error(ErrorKind::Type, "Cannot explain 1", 7, 1)),
        (
            "why price |> depth(-1)",
            error(ErrorKind::Type, "-1 is not a non-negative integer", 7, 1),
        ),
        (
            "why price |> diff",
            error(ErrorKind::Type, "Cannot use 'diff' outside suppose", 7, 1),
        ),
    ] {
        assert_eq!(
            failure(&format!("{ORDER}why tax |> intersect({other})")),
            expected,
            "{other}"
        );
        assert_eq!(
            failure(&format!("{ORDER}why tax |> except({other})")),
            expected,
            "{other}"
        );
    }
    assert_eq!(
        failure(&format!(
            "{ORDER}class Item(price):\n    this.price = price\nwhy tax |> intersect($Item.price)"
        )),
        error(ErrorKind::Type, "Cannot explain $Item.price", 9, 1)
    );
    assert_eq!(
        failure(&format!(
            "{ORDER}o = {{p: null}}\nwhy tax |> intersect(o.p.q)"
        )),
        error(ErrorKind::Reference, "o.p is not defined", 8, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}1 |> intersect(missing)")),
        error(ErrorKind::Type, "Cannot explain 1", 7, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}missing |> except(1)")),
        error(ErrorKind::Reference, "missing is not defined", 7, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}x = why tax |> intersect(1)")),
        error(ErrorKind::Type, "Cannot explain 1", 7, 1)
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{ORDER}kept = why tax |> intersect(why shipping)\n"),
    );
    assert_eq!(
        failure_in(&mut runtime, "kept = why tax |> intersect(missing)"),
        error(ErrorKind::Reference, "missing is not defined", 1, 1)
    );
    assert_eq!(nodes(&mut runtime, "kept"), json!(["quantity"]));
}

#[test]
fn zero_or_two_arguments_report_the_errors_of_parentheses() {
    assert_eq!(
        failure(&format!("{ORDER}why tax |> intersect()\n")),
        error(ErrorKind::Syntax, "Unexpected ')'", 7, 22)
    );
    assert_eq!(
        failure(&format!("{ORDER}why tax |> except()\n")),
        error(ErrorKind::Syntax, "Unexpected ')'", 7, 19)
    );
    assert_eq!(
        failure(&format!(
            "{ORDER}why tax |> intersect(why shipping, why price)\n"
        )),
        error(ErrorKind::Syntax, "Expected ')' but found ','", 7, 34)
    );
    assert_eq!(
        failure(&format!("{ORDER}why tax |> intersect(why shipping,)\n")),
        error(ErrorKind::Syntax, "Expected ')' but found ','", 7, 34)
    );
    assert_eq!(
        failure(&format!("{ORDER}why tax |> intersect(why shipping\n")),
        error(
            ErrorKind::Syntax,
            "Expected ')' but found end of input",
            8,
            1
        )
    );
    assert_eq!(
        failure(&format!("{ORDER}why tax |> intersect(\n")),
        error(ErrorKind::Syntax, "Unexpected end of input", 8, 1)
    );
    let mut deep = String::from("why c");
    for _ in 0..70 {
        deep = format!("why c |> intersect({deep})");
    }
    let (kind, message, _) = failure(&format!("{CHAIN}x = {deep}"));
    assert_eq!(
        (kind, message.as_str()),
        (ErrorKind::Syntax, "Expressions are nested too deeply")
    );
    let mut nested = String::from("why c");
    for _ in 0..30 {
        nested = format!("why c |> intersect({nested})");
    }
    assert_eq!(result(&format!("{CHAIN}({nested}).length")), json!(3.0));
}

#[test]
fn other_selections_built_past_the_parser_limit_raise_the_depth_error() {
    let mut source = String::from("why c");
    let mut parsed = 0;
    for levels in 1..=100 {
        let deeper = format!("why c |> intersect({source})");
        match parse(&deeper) {
            Ok(statements) => {
                assert_eq!(statements, [Stmt::Expression(chain(levels))], "{levels}");
                source = deeper;
                parsed = levels;
            }
            Err(error) => {
                assert_eq!(
                    error.message(),
                    "Expressions are nested too deeply",
                    "{levels}"
                );
                break;
            }
        }
    }
    assert!(parsed > 0 && parsed < 100, "{parsed}");

    let mut runtime = Runtime::new();
    checked(&mut runtime, CHAIN);
    let steps = resolved(&mut runtime, &chain(0)).unwrap();
    assert_eq!(steps.as_array().map(Vec::len), Some(3));
    assert_eq!(evaluate(&mut runtime, &source), steps);
    assert_eq!(resolved(&mut runtime, &chain(parsed)), Ok(steps.clone()));
    let exceeded = unpositioned(ErrorKind::Type, "Maximum expression depth exceeded");
    let mut limit = None;
    for levels in parsed + 1..=1_000 {
        match resolved(&mut runtime, &chain(levels)) {
            Ok(selection) => assert_eq!(selection, steps, "{levels}"),
            Err(error) => {
                assert_eq!(error, exceeded, "{levels}");
                limit = Some(levels - 1);
                break;
            }
        }
    }
    let limit = limit.expect("a chain of 1,000 levels meets the depth limit");
    assert!(limit > parsed, "{limit}");
    for levels in [limit + 1, limit + 2, 1_000] {
        assert_eq!(
            resolved(&mut runtime, &chain(levels)),
            Err(exceeded.clone()),
            "{levels}"
        );
    }
    assert_eq!(resolved(&mut runtime, &chain(limit)), Ok(steps));
    checked(&mut runtime, "d = c + 1");
    assert_eq!(
        nodes(&mut runtime, "why d |> intersect(why b)"),
        json!(["b", "a"])
    );
}

#[test]
fn every_binding_named_intersect_or_except_shadows_its_stage() {
    for name in ["intersect", "except"] {
        let bindings = [
            (format!("{name} = 1\nwhy c |> {name}(why b)"), 5),
            (format!("def {name}(x) => x\nwhy c |> {name}(why b)"), 5),
            (format!("class {name}\nwhy c |> {name}(why b)"), 5),
            (format!("{name} = 1\nwhy c |> {name}(missing)"), 5),
            (format!("{name} = 1\nmissing |> {name}(why b)"), 5),
            (format!("{name} = 1\n1 |> {name}(why b)"), 5),
            (format!("def f({name}) => why c |> {name}(why b)\nf(1)"), 5),
            (format!("def g {name} => why c |> {name}(why b)\ng(1)"), 5),
            (format!("f = {name} -> why c |> {name}(why b)\nf(1)"), 5),
            (format!("f = ({name}) => why c |> {name}(why b)\nf(1)"), 5),
            (
                format!("f = lambda {name}: why c |> {name}(why b)\nf(1)"),
                5,
            ),
            (format!("[1].map({name} -> why c |> {name}(why b))"), 4),
            (
                format!("for {name} of \"x\":\n    y = why c |> {name}(why b)"),
                4,
            ),
            (
                format!("ok = all {name} in [1]: (why c |> {name}(why b)).length > 0"),
                4,
            ),
            (
                format!("{{\n    {name} = 5\n    y = why c |> {name}(why b)\n}}"),
                4,
            ),
            (
                format!("class R:\n    def m({name}) => why c |> {name}(why b)\nR().m(1)"),
                6,
            ),
            (
                format!("class Q({name}):\n    this.v = (why c |> {name}(why b)).length\nQ(1)"),
                6,
            ),
            (
                format!("try:\n    throw 1\ncatch {name}:\n    y = why c |> {name}(why b)"),
                4,
            ),
            (
                format!("{{{name}}} = {{{name}: 1}}\nwhy c |> {name}(why b)"),
                5,
            ),
        ];
        for (binding, line) in bindings {
            assert_eq!(
                failure(&format!("{CHAIN}{binding}")),
                error(
                    ErrorKind::Type,
                    &format!("{name} is not an operation"),
                    line,
                    1
                ),
                "{binding}"
            );
        }
    }
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\nwhy c |> intersect(why b |> stated)"
        )),
        error(ErrorKind::Type, "stated is not an operation", 5, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\nmissing |> intersect(why b |> stated)"
        )),
        error(ErrorKind::Reference, "missing is not defined", 5, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\nintersect = 2\nwhy c |> intersect(why b) |> stated"
        )),
        error(ErrorKind::Type, "stated is not an operation", 6, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\nexcept = 2\nwhy c |> stated |> except(why b)"
        )),
        error(ErrorKind::Type, "except is not an operation", 6, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}except = 1\nwhy c |> intersect(why b |> except(a))"
        )),
        error(ErrorKind::Type, "except is not an operation", 5, 1)
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}intersect = 1\n[(why c |> except(why a)).length, (why c |> stated).length]"
        )),
        json!([2.0, 1.0])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}why = 1\naffects = 2\n(why c |> intersect(why b)).length"
        )),
        json!(2.0)
    );
}

#[test]
fn a_binding_declared_after_a_standing_selection_fails_its_refresh() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}kept = why c |> intersect(why b)\nexcept = 5"),
    );
    assert_eq!(nodes(&mut runtime, "kept"), json!(["b", "a"]));
    checked(&mut runtime, "intersect = 5");
    assert_eq!(
        failure_in(&mut runtime, "a = 3"),
        error(ErrorKind::Type, "intersect is not an operation", 1, 1)
    );
    assert_eq!(evaluate(&mut runtime, "a"), json!(1.0));
    checked(&mut runtime, "delete intersect\na = 3");
    assert_eq!(
        evaluate(&mut runtime, "kept.map(s -> s.holds)"),
        json!([5.0, 3.0])
    );
}

#[test]
fn an_undefined_side_makes_the_selection_undefined() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}n = null\nkept = why c |> intersect(why b |> depth(n))\n"),
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "[kept, why c |> intersect(why b |> depth(n)), why c |> depth(n) |> intersect(missing), why c |> intersect(why b |> depth(n)) |> intersect(missing), why c |> intersect(why b |> depth(n)) |> depth(missing)]"
        ),
        json!([null, null, null, null, null])
    );
    assert_eq!(evaluate(&mut runtime, "kept.length"), Json::Null);
    assert_eq!(
        evaluate(
            &mut runtime,
            "all s in why c |> intersect(why b |> depth(n)): s.holds > 0"
        ),
        Json::Null
    );
    checked(&mut runtime, "n = 0");
    assert_eq!(nodes(&mut runtime, "kept"), json!(["b"]));
    assert_eq!(
        failure_in(&mut runtime, "why c |> depth(n) |> intersect(missing)"),
        error(ErrorKind::Reference, "missing is not defined", 1, 1)
    );
    checked(&mut runtime, "n = 1");
    assert_eq!(nodes(&mut runtime, "kept"), json!(["b", "a"]));
    checked(&mut runtime, "n = undefined");
    assert_eq!(evaluate(&mut runtime, "kept"), Json::Null);

    let mut waiting = Runtime::new();
    checked(
        &mut waiting,
        &format!("{CHAIN}n = null\nskipped = why c |> depth(n) |> intersect(missing)\n"),
    );
    assert_eq!(evaluate(&mut waiting, "skipped"), Json::Null);
    assert_eq!(
        failure_in(&mut waiting, "n = 1"),
        error(ErrorKind::Reference, "missing is not defined", 1, 1)
    );
    assert_eq!(evaluate(&mut waiting, "[n, skipped]"), json!([null, null]));
}

#[test]
fn standing_selections_follow_both_sides_and_the_steps_they_drop() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}fee = 1\nkept = why tax |> intersect(why shipping)\nfacts = why tax |> stated |> except(why shipping |> stated)\nsplit = why subtotal |> intersect(why shipping |> derived |> why)\n"
        ),
    );
    assert_eq!(nodes(&mut runtime, "kept"), json!(["quantity"]));
    assert_eq!(nodes(&mut runtime, "facts"), json!(["rate", "price"]));
    assert_eq!(nodes(&mut runtime, "split"), json!(["quantity"]));
    checked(&mut runtime, "shipping = quantity * 2 + fee");
    assert_eq!(nodes(&mut runtime, "kept"), json!(["quantity"]));
    checked(&mut runtime, "rate = fee / 5");
    assert_eq!(
        nodes(&mut runtime, "why tax"),
        json!(["tax", "subtotal", "rate", "fee", "price", "quantity"])
    );
    assert_eq!(nodes(&mut runtime, "kept"), json!(["fee", "quantity"]));
    assert_eq!(nodes(&mut runtime, "facts"), json!(["price"]));
    checked(&mut runtime, "shipping = price + 1");
    assert_eq!(nodes(&mut runtime, "kept"), json!(["price"]));
    assert_eq!(nodes(&mut runtime, "facts"), json!(["fee", "quantity"]));
    assert_eq!(nodes(&mut runtime, "split"), json!(["price"]));
    checked(&mut runtime, "price = 12\nquantity = 5");
    assert_eq!(
        evaluate(&mut runtime, "kept.map(s -> s.holds)"),
        json!([12.0])
    );
    checked(&mut runtime, "delete shipping");
    assert_eq!(nodes(&mut runtime, "kept"), json!([]));
    assert_eq!(
        nodes(&mut runtime, "facts"),
        json!(["fee", "price", "quantity"])
    );
    assert_eq!(nodes(&mut runtime, "split"), json!([]));
    checked(&mut runtime, "shipping = fee");
    assert_eq!(nodes(&mut runtime, "kept"), json!(["fee"]));
}

#[test]
fn standing_selections_observe_both_walks_and_model_membership() {
    let output = cli(
        "observes",
        &format!(
            "{CHAIN}d = 4\nx = why c |> intersect(model)\ny = why b |> except(why d)\nz = why d |> intersect(why c |> stated)\n1"
        ),
    );
    assert!(output.status.success(), "{output:?}");
    let listing = String::from_utf8(output.stdout).unwrap();
    assert!(
        listing
            .contains("  x [variable]\n    reads    c, b, a, d\n    observes c, b, a, model, d\n"),
        "{listing}"
    );
    assert!(
        listing.contains("  y [variable]\n    reads    b, a, d\n    observes b, a, d\n"),
        "{listing}"
    );
    assert!(
        listing.contains("  z [variable]\n    reads    d, c, b, a\n    observes d, c, b, a\n"),
        "{listing}"
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{CHAIN}held = why c |> except(model |> stated)\nempty = why a |> except(why c)\n"
        ),
    );
    assert_eq!(nodes(&mut runtime, "held"), json!(["c", "b"]));
    assert_eq!(nodes(&mut runtime, "empty"), json!([]));
    checked(&mut runtime, "b = 7");
    assert_eq!(nodes(&mut runtime, "held"), json!(["c"]));
    checked(&mut runtime, "c = a + 1\na = b");
    assert_eq!(nodes(&mut runtime, "held"), json!(["c", "a"]));
    assert_eq!(nodes(&mut runtime, "empty"), json!([]));
    checked(&mut runtime, "c = 5");
    assert_eq!(nodes(&mut runtime, "empty"), json!(["a", "b"]));
}

#[test]
fn class_rules_functions_methods_lambdas_quantifiers_and_conditions_hold_the_stages() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Order(qty, price, fee):\n    this.qty = qty\n    this.price = price\n    this.fee = fee\n    def own() => (this.total |> why |> except(this.cost |> why)).map(s -> s.node)\n$Order.total = $Order.qty * $Order.price\n$Order.cost = $Order.qty + $Order.fee\n$Order.shared = why $Order.total |> intersect(why $Order.cost)\n$Order.big = any s in why $Order.total |> intersect(why $Order.cost): s.holds > 3\nif (why $Order.total |> intersect(why $Order.cost)).length > 0:\n    $Order.linked = true\npen = Order(2, 5, 1)\ncup = Order(4, 3, 2)\ndef common() => (why pen.total |> intersect(why pen.cost)).length\npick = [1].map(v -> (why pen.total |> intersect(why cup.total)).length)\n",
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "[pen.shared.map(s -> s.node), cup.shared.map(s -> s.node), pen.big, cup.big, pen.linked, pen.own()]"
        ),
        json!([
            ["pen.qty"],
            ["cup.qty"],
            false,
            true,
            true,
            ["pen.total", "pen.price"]
        ])
    );
    assert_eq!(
        evaluate(&mut runtime, "[common(), pick]"),
        json!([1.0, [0.0]])
    );
    checked(&mut runtime, "pen.qty = 9");
    assert_eq!(
        evaluate(&mut runtime, "[pen.big, cup.big]"),
        json!([true, true])
    );
    checked(&mut runtime, "$Order.cost = $Order.price + $Order.fee");
    assert_eq!(
        evaluate(
            &mut runtime,
            "[pen.shared.map(s -> s.node), cup.shared.map(s -> s.node)]"
        ),
        json!([["pen.price"], ["cup.price"]])
    );
    checked(&mut runtime, "jar = Order(1, 2, 3)");
    assert_eq!(nodes(&mut runtime, "jar.shared"), json!(["jar.price"]));
    let text = result(&format!(
        "{CHAIN}`${{(why c |> intersect(why b)).length}} shared`"
    ));
    assert_eq!(text, json!("2 shared"));
}

#[test]
fn class_rules_check_the_names_and_cycles_in_other() {
    let class = "class Order(x):\n    this.x = x\n";
    assert_eq!(
        failure(&format!(
            "{class}$Order.k = why $Order.x |> intersect(why missing)\n1"
        )),
        error(ErrorKind::Reference, "missing is not defined", 3, 1)
    );
    assert_eq!(
        failure(&format!(
            "{class}$Order.y = (why $Order.x |> intersect(why $Order.y)).length\no = Order(1)\no.y"
        )),
        error(ErrorKind::Type, "Circular Dependency", 3, 1)
    );
    assert_eq!(
        failure(&format!(
            "{class}$Order.a = (why $Order.x |> intersect(why $Order.b)).length\n$Order.b = (why $Order.x |> except(why $Order.a)).length\no = Order(1)\n[o.a, o.b]"
        )),
        error(ErrorKind::Type, "Circular Dependency", 4, 1)
    );
    assert_eq!(
        result(&format!(
            "{class}$Order.k = (why $Order.x |> intersect(why $Order.z)).length\no = Order(1)\no.z = o.x + 1\no.k"
        )),
        json!(1.0)
    );
}

#[test]
fn a_declaration_holding_a_stage_stays_out_of_its_own_model() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = 1\nb = a + 1\nrest = model |> except(why a)\nwhole = model |> intersect(model)\n",
    );
    assert_eq!(nodes(&mut runtime, "rest"), json!(["b"]));
    assert_eq!(nodes(&mut runtime, "whole"), json!(["a", "b"]));
    assert_eq!(nodes(&mut runtime, "model |> why"), json!(["a", "b"]));
    checked(&mut runtime, "c = b + 1");
    assert_eq!(nodes(&mut runtime, "rest"), json!(["b", "c"]));
    checked(
        &mut runtime,
        "held = 1 if true else (why c |> intersect(why b))\nleft = 1 if true else (why c |> except(why b))\nwalked = 1 if true else (why c)\nplain = 1 if true else c\n",
    );
    assert_eq!(
        nodes(&mut runtime, "model |> stated"),
        json!(["a", "plain"])
    );
    assert_eq!(nodes(&mut runtime, "rest"), json!(["b", "c", "plain"]));
    assert_eq!(
        evaluate(
            &mut runtime,
            "[held, left, walked, plain, (why held).length, (why left).length]"
        ),
        json!([1.0, 1.0, 1.0, 1.0, 0.0, 0.0])
    );
}

#[test]
fn declarations_that_call_a_function_holding_a_stage_stay_live() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}def counted() => (why tax |> intersect(why shipping)).length\nheld = counted()\n"
        ),
    );
    assert_eq!(evaluate(&mut runtime, "held"), json!(1.0));
    checked(&mut runtime, "shipping = price + quantity");
    assert_eq!(evaluate(&mut runtime, "held"), json!(2.0));
    checked(&mut runtime, "subtotal = quantity * 2");
    assert_eq!(evaluate(&mut runtime, "held"), json!(1.0));
}

#[test]
fn failed_changes_suppose_blocks_and_diff_restore_intersections() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{ORDER}kept = why tax |> intersect(why shipping)\n"),
    );
    checked(
        &mut runtime,
        "try:\n    shipping = price + quantity\n    assert(kept.map(s -> s.node), [\"price\", \"quantity\"])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert(kept.map(s -> s.node), [\"quantity\"])",
    );
    assert_eq!(
        failure_in(
            &mut runtime,
            "shipping = price + quantity\nshipping = missing"
        ),
        error(ErrorKind::Reference, "missing is not defined", 2, 1)
    );
    assert_eq!(nodes(&mut runtime, "kept"), json!(["quantity"]));
    let plan = evaluate(
        &mut runtime,
        "suppose shipping = price + 1:\n    [kept.map(s -> s.node), (why tax |> diff |> intersect(why shipping)).map(s -> s.node), (why tax |> intersect(why shipping) |> diff).map(s -> s.node)]",
    );
    assert_eq!(plan, json!([["price"], [], []]));
    let moved = evaluate(
        &mut runtime,
        "suppose price = 20:\n    [(why tax |> diff |> intersect(why shipping |> why)).map(s -> s.node), (why tax |> diff |> except(why shipping)).map(s -> s.node)]",
    );
    assert_eq!(moved, json!([[], ["tax", "subtotal", "price"]]));
    assert_eq!(nodes(&mut runtime, "kept"), json!(["quantity"]));
    assert_eq!(evaluate(&mut runtime, "price"), json!(10.0));
}

#[test]
fn snapshots_freeze_selections_and_the_limits_inside_other() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}n = 1\nsnap = (why tax |> intersect(why shipping)).value\nnear = why tax |> intersect(why subtotal |> depth(n.value))\nlive = why tax |> intersect(why subtotal |> depth(n))\n"
        ),
    );
    assert_eq!(
        stored(&runtime, "near"),
        "tax|>why|>intersect(subtotal|>why|>depth(1))"
    );
    assert_eq!(
        stored(&runtime, "live"),
        "tax|>why|>intersect(subtotal|>why|>depth(n))"
    );
    checked(&mut runtime, "n = 0\nshipping = price + 1");
    assert_eq!(nodes(&mut runtime, "snap"), json!(["quantity"]));
    assert_eq!(
        nodes(&mut runtime, "near"),
        json!(["subtotal", "price", "quantity"])
    );
    assert_eq!(nodes(&mut runtime, "live"), json!(["subtotal"]));
    checked(&mut runtime, "subtotal = price + 1");
    assert_eq!(nodes(&mut runtime, "near"), json!(["subtotal", "price"]));
}

#[test]
fn steps_are_matched_by_node_where_the_value_spelling_misses_nan() {
    assert_eq!(
        result(
            "x = 0 / 0\ny = x + 1\n[(why y |> intersect(why x)).map(s -> s.node), [s for s in why y if s in why x].map(s -> s.node), (why y |> except(why x)).map(s -> s.node), [s for s in why y if s not in why x].map(s -> s.node)]"
        ),
        json!([["x"], [], ["y"], ["y", "x"]])
    );
    assert_eq!(
        result(
            "x = 0 / 0\ny = x + 1\n(why y).filter(s -> s.node in (why x).map(t -> t.node)).map(s -> s.node)"
        ),
        json!(["x"])
    );
}

#[test]
fn set_stages_render_once_and_parse_back() {
    for source in [
        "why c |> intersect(why b)",
        "c |> why |> except(b |> why |> stated)",
        "model |> intersect(model) |> stated",
        "a |> affects |> intersect(b |> affects) |> why |> except(a)",
        "why c |> intersect(o.p)",
        "why c |> intersect(why b |> depth(n + 1)) |> depth(1)",
        "why c |> path(a) |> except(why b |> path(a))",
        "a || b |> intersect(c)",
        "why c |> intersect(why b |> intersect(why a))",
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
        parse_expression("why tax |> intersect(why shipping) |> derived")
            .unwrap()
            .to_string(),
        "tax|>why|>intersect(shipping|>why)|>derived"
    );
    assert_eq!(
        parse_expression("why tax |> except(model)")
            .unwrap()
            .to_string(),
        "tax|>why|>except(model)"
    );
    let mut chained = String::from("c");
    for level in 1..=6 {
        chained.push_str(" |> intersect(a)");
        let rendered = parse_expression(&chained).unwrap().to_string();
        assert_eq!(rendered.len(), 1 + 14 * level, "{rendered}");
    }
    assert_eq!(
        generate(&parse("def f(x = 1) => why c |> except(why b)").unwrap()[0]),
        "def f(x=1):\n    return c|>why|>except(b|>why)"
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}kept = why c |> intersect(why b)\n"),
    );
    assert_eq!(stored(&runtime, "kept"), "c|>why|>intersect(b|>why)");
    assert_eq!(nodes(&mut runtime, "why c"), json!(["c", "b", "a"]));
    assert_eq!(nodes(&mut runtime, "affects a"), json!(["b", "c"]));
    assert_eq!(nodes(&mut runtime, "model |> why"), json!(["a", "b", "c"]));
    let output = cli(
        "keys",
        &format!(
            "{CHAIN}if (why c |> intersect(why b)).length > 1:\n    flag = 1\nkept = why c |> except(why b)\n1"
        ),
    );
    assert!(output.status.success(), "{output:?}");
    let listing = String::from_utf8(output.stdout).unwrap();
    assert!(
        listing.contains("  if(c|>why|>intersect(b|>why).length>1) [if]\n"),
        "{listing}"
    );
    assert!(
        listing.contains("  kept [variable]\n    reads    c, b, a\n    observes c, b, a\n"),
        "{listing}"
    );
}

#[test]
fn set_stages_handle_large_graphs() {
    let size = 20_000;
    let mut source = String::from("v0 = 1\n");
    for index in 1..size {
        source.push_str(&format!("v{index} = v{} + 1\n", index - 1));
    }
    source.push_str("w0 = v0 + 1\n");
    for index in 1..size {
        source.push_str(&format!("w{index} = w{} + v{index}\n", index - 1));
    }
    let mut runtime = Runtime::new();
    checked(&mut runtime, &source);
    let last = size - 1;
    assert_eq!(
        evaluate(
            &mut runtime,
            &format!("(why w{last} |> intersect(why v{last})).length")
        ),
        json!(20_000.0)
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            &format!("(why w{last} |> except(why v{last})).length")
        ),
        json!(20_000.0)
    );
    assert_eq!(
        evaluate(&mut runtime, "(affects v0 |> intersect(affects w0)).length"),
        json!(19_999.0)
    );
}

#[test]
fn near_misses_keep_their_accepted_results() {
    assert_eq!(
        result(&format!(
            "{ORDER}`v ${{quantity why tax |> intersect(why shipping)}}`"
        )),
        json!("v 3")
    );
    assert_eq!(
        failure(&format!("{ORDER}why tax | > intersect(why shipping)")),
        error(ErrorKind::Syntax, "Unexpected character '|'", 7, 9)
    );
    assert_eq!(
        failure(&format!("{ORDER}why tax ||> intersect(why shipping)")),
        error(ErrorKind::Syntax, "Unexpected '>'", 7, 13)
    );
    assert_eq!(
        failure(&format!("{ORDER}tax.intersect(why shipping)")),
        error(ErrorKind::Type, "number.intersect is not a function", 7, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}why intersect(shipping)")),
        error(ErrorKind::Type, "Cannot explain intersect(shipping)", 7, 1)
    );
}

#[test]
fn inputs_that_were_errors_now_give_what_the_proposal_lists() {
    assert_eq!(
        result(&format!(
            "{ORDER}a, b = tax |> why |> intersect(why shipping)\n[a.node, b]"
        )),
        json!(["quantity", null])
    );
    assert_eq!(
        result(&format!(
            "{ORDER}{{node}} = why tax |> except(why shipping)\nnode"
        )),
        Json::Null
    );
    assert_eq!(
        failure(&format!(
            "{ORDER}[p] = why tax |> intersect(why shipping)\n"
        )),
        error(ErrorKind::Syntax, "Cannot assign to [p]", 7, 1)
    );
    assert_eq!(
        failure(&format!(
            "{ORDER}delete why tax |> intersect(why shipping)\n"
        )),
        error(
            ErrorKind::Syntax,
            "Cannot delete tax|>why|>intersect(shipping|>why)",
            7,
            1
        )
    );
    assert_eq!(
        result(&format!(
            "{ORDER}y = 1 if why tax |> intersect(why shipping) else 2\ny"
        )),
        json!(1.0)
    );
    assert_eq!(
        result(&format!(
            "{ORDER}x = suppose: (why tax |> intersect(why shipping)).length\nx"
        )),
        json!(1.0)
    );
    assert_eq!(
        result(&format!(
            "{ORDER}first = 1\n(why tax |> intersect(why shipping)).length + first"
        )),
        json!(2.0)
    );
    assert_eq!(
        result(&format!("{ORDER}why tax |> intersect(why shipping)(2)")),
        json!(2.0)
    );
}

#[test]
fn the_binary_runs_set_stages_from_files_and_the_line_repl() {
    let output = cli(
        "file",
        &format!("{CHAIN}(why c |> intersect(why b)).map(s -> s.node)\n"),
    );
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("[\"b\",\"a\"]\n")
    );
    for ending in ["\n", "\r\n"] {
        let lines = [
            "a = 1",
            "b = a + 2",
            "c = b * 2",
            "(why c |> intersect(why b)).length",
            "why c |> except(",
            "(why c |> except(why b)).length",
            "why c |> intersect",
            "intersect = 1",
            "why c |> intersect(why b)",
        ];
        let output = repl(&["--plain", "--json"], &(lines.join(ending) + ending));
        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{BANNER}1.0\n3.0\n6.0\n2.0\n1.0\n1.0\n")
        );
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "SyntaxError: Unexpected end of input (line 2, column 1)\nTypeError: intersect is not an operation\nTypeError: intersect is not an operation (line 1, column 1)\n"
        );
        let plain = repl(&["--plain"], &(lines[..4].join(ending) + ending));
        assert!(plain.status.success(), "{plain:?}");
        assert_eq!(
            String::from_utf8(plain.stdout).unwrap(),
            format!("{BANNER}1\n3\n6\n2\n")
        );
    }
}
