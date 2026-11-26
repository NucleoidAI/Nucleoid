mod common;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate;
use nucleoid::lang::ast::{Ast, Expr, Stmt, parse, parse_expression};
use nucleoid::nuc::Nuc;
use nucleoid::reasoning::Stage;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde_json::{Value as Json, json};

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid joins what two explanations rest on with union",
    "Nucleoid narrows a union of explanations to its facts and conclusions",
    "Nucleoid joins what two changes of each instance reach with union",
    "Nucleoid restores unions after failed changes",
];

const REFERENCE_TITLE: &str = "Nucleoid joins two explanations with union";
const README_TITLE: &str = "Union lists every fact behind either of two alarms";
const EXAMPLES_TITLE: &str = "Union shows every figure that a change of rent or of wages moves";

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

const CHAIN: &str = "a = 1\nb = a + 2\nc = b * 2\n";

const SIX: &str = "a = 1\nb = a + 1\nc = b * 2\nd = a + 3\ne = 7\nf = e + d\n";

const ORDER: &str = "price = 10\nquantity = 3\nrate = 0.2\nsubtotal = price * quantity\ntax = subtotal * rate\nshipping = quantity * 2\n";

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "union selection regression".to_string(),
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

fn unions(source: &str) -> usize {
    let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    format!("{statements:?}").matches("Union {").count()
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

fn union(source: Box<Expr>, other: Box<Expr>) -> Expr {
    Expr::Union { source, other }
}

fn chain(levels: usize) -> Expr {
    let mut tree = *reason(Stage::Why, identifier("c"));
    for _ in 0..levels {
        tree = union(reason(Stage::Why, identifier("c")), Box::new(tree));
    }
    tree
}

fn ranked_chain(levels: usize) -> Expr {
    Expr::Depth {
        source: Box::new(chain(levels)),
        limit: Box::new(Expr::Number(5.0)),
    }
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
        std::env::temp_dir().join(format!("nucleoid-nuc120-{label}-{}", std::process::id()));
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

fn spelled(selection: &str, other: &str, stage: &str) -> String {
    format!("({selection}{stage}).concat({other} |> except({selection}){stage})")
}

#[test]
fn appended_document_cases_run_every_assertion() {
    let [spec, reference, readme, examples] = documents();
    let appended = SPEC_TITLES
        .iter()
        .map(|title| {
            let expected = (*title == SPEC_TITLES[2]).then_some("4");
            (&spec, *title, expected)
        })
        .chain([
            (
                &reference,
                REFERENCE_TITLE,
                Some("[\"rate\",\"price\",\"quantity\",\"fee\"]"),
            ),
            (&readme, README_TITLE, None),
            (
                &examples,
                EXAMPLES_TITLE,
                Some("[[\"overhead\",\"power\",\"costs\",\"price\",\"payroll\"],55.5]"),
            ),
        ]);

    let mut assertions = 0;
    for (cases, title, expected) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(unions(&case.source) > 0, "{title} writes a union stage");
        assert_eq!(combines(&case.source), 0, "{title} writes no set stage");
        assert_eq!(case.expected.as_deref(), expected, "{title}");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 26);
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Union selections\n\n```\n")
        .expect("the language reference documents union");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 120](nuc-0120.md)"));
    assert!(include_str!("../README.md").contains("[NUC 120](docs/nuc-0120.md)"));
    assert!(include_str!("../docs/examples.md").contains("[NUC 120](nuc-0120.md)"));
    assert!(include_str!("../nucleoid.spec.md").contains("\n## Union reasoning selections\n"));
}

#[test]
fn no_earlier_document_case_writes_a_union_stage() {
    let mut cases = 0;
    let mut with_stage = 0;
    for case in documents().iter().flatten() {
        cases += 1;
        let uses = unions(&case.source) > 0;
        assert_eq!(uses, is_appended(&case.title), "{}", case.title);
        with_stage += usize::from(uses);
    }
    assert_eq!(with_stage, 7);
    assert_eq!(cases, 1326);
}

#[test]
fn the_appended_cases_stand_after_nuc_119_and_before_nuc_103() {
    let [spec, reference, readme, examples] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[792],
        "Nucleoid runs a finally clause before an error or a return leaves"
    );
    assert_eq!(titles[793..797], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid runs finally clauses as Python, JavaScript and Java do",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Finally clauses record every attempt to unlock a door",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Finally clauses log every dispatch of a driver",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 19 - 12 * usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 11].title, following);
    }
}

#[test]
fn the_union_stage_parses_into_its_own_variant() {
    let tree = union(
        reason(Stage::Why, identifier("c")),
        reason(Stage::Why, identifier("b")),
    );
    assert_eq!(value_of("x = c |> why |> union(b |> why)"), tree);
    assert_eq!(value_of("x = why c |> union(why b)"), tree);
    assert_eq!(value_of("x = why c |> union (why b)"), tree);
    assert_eq!(value_of("x = c|>why|>union(b|>why)"), tree);
    assert_eq!(value_of("x = (why c |> union(\n    why b\n))"), tree);
    assert_eq!(parse_expression("why c |> union(why b)").unwrap(), tree);
    assert_eq!(
        value_of("x = why c |> union(b)"),
        union(reason(Stage::Why, identifier("c")), identifier("b"))
    );
    for other in ["model", "(model)"] {
        assert_eq!(
            value_of(&format!("x = why c |> union({other})")),
            union(reason(Stage::Why, identifier("c")), Box::new(Expr::Model)),
            "{other}"
        );
    }
    assert_eq!(
        value_of("x = model |> union(why c)"),
        union(Box::new(Expr::Model), reason(Stage::Why, identifier("c")))
    );
    assert_eq!(
        value_of("x = why c |> union(model.p)"),
        union(
            reason(Stage::Why, identifier("c")),
            Box::new(Expr::Member {
                object: identifier("model"),
                property: "p".to_string(),
            })
        )
    );
    assert_eq!(
        value_of("x = why c |> union(why b) |> stated"),
        *reason(Stage::Stated, Box::new(tree.clone()))
    );
    assert_eq!(
        value_of("x = union(c, b)"),
        Expr::Call {
            callee: identifier("union"),
            arguments: vec![
                Expr::Identifier("c".to_string()),
                Expr::Identifier("b".to_string()),
            ],
        }
    );
    assert_eq!(parse("union c b").unwrap().len(), 3);
    assert_eq!(Stage::from_name("union"), None);
    assert_eq!(
        unions("x = why c |> union(why b |> union(a)) |> union(why a)"),
        3
    );
    assert_eq!(unions("x = why c |> depth(1)\ny = union(c)"), 0);
    assert_eq!(combines("x = why c |> union(why b)"), 0);
    assert_eq!(combines("x = why c |> union(why b |> intersect(a))"), 1);
    for source in [
        "p is [why c |> union(why g([w])) for s in [1]]",
        "p is [x = why c |> union(g([w])) for s in [1]]",
    ] {
        let tree = format!("{:?}", parse(source).unwrap());
        assert!(tree.contains("Union {"), "{tree}");
        assert!(!tree.contains("Word"), "{tree}");
    }
    let marked = format!("{:?}", parse("p is [w, why c |> union(w)]").unwrap());
    assert_eq!(marked.matches("Word").count(), 1, "{marked}");
}

#[test]
fn ordinary_names_called_union_keep_their_meaning() {
    assert_eq!(result("union = 1\n[union, union + 1]"), json!([1.0, 2.0]));
    assert_eq!(result("def union(x, y) => x + y\nunion(4, 1)"), json!(5.0));
    assert_eq!(result("x = {union: 1}\nx.union"), json!(1.0));
    assert_eq!(result("o = {}\no.union = 3\no.union"), json!(3.0));
    assert_eq!(result("[1, 2].map(union -> union * 2)"), json!([2.0, 4.0]));
    assert_eq!(
        result("o = {f: x -> x + 1}\no.union = o.f\no.union(2)"),
        json!(3.0)
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}class Sets:\n    def union(x) => x + 1\ns = Sets()\n[(why b |> union(why c)).length, s.union(1)]"
        )),
        json!([3.0, 2.0])
    );
    assert_eq!(
        failure(&format!("{CHAIN}union")),
        error(ErrorKind::Reference, "union is not defined", 4, 1)
    );
    assert_eq!(
        failure(&format!("{CHAIN}union(1, 2)")),
        error(ErrorKind::Reference, "union is not defined", 4, 1)
    );
    assert_eq!(
        failure(&format!("{CHAIN}union a b")),
        error(ErrorKind::Reference, "union is not defined", 4, 1)
    );
    assert_eq!(
        failure(&format!("{CHAIN}xs = [1]\nxs.union([2])")),
        error(ErrorKind::Type, "xs.union is not a function", 5, 1)
    );
}

#[test]
fn forms_without_parentheses_keep_their_parse_time_error() {
    for (stage, name) in [
        ("why c |> union", "union"),
        ("why c |> union\n(why b)", "union"),
        ("why c |> union[b]", "union"),
        ("why c |> union.x", "union"),
        ("why c |> union{1}", "union"),
        ("why c |> Union(why b)", "Union"),
        ("why c |> unions(why b)", "unions"),
        ("why c |> unite(why b)", "unite"),
        ("first = 1\nwhy c |> union", "union"),
        ("x = [why c |> union]", "union"),
    ] {
        assert_eq!(
            failure(&format!("{CHAIN}{stage}")),
            unpositioned(ErrorKind::Type, &format!("{name} is not an operation")),
            "{stage}"
        );
    }
    assert_eq!(
        failure(&format!("{CHAIN}why c |> |> union(why b)")),
        error(ErrorKind::Syntax, "Expected an operation", 4, 10)
    );
}

#[test]
fn union_selects_what_the_accepted_spelling_selects() {
    for seed in 1..=6 {
        let mut runtime = Runtime::new();
        checked(&mut runtime, &graph(seed, 10));
        for x in 0..10 {
            for y in 0..10 {
                for (first, second) in [
                    ("why", "why"),
                    ("affects", "affects"),
                    ("why", "affects"),
                    ("affects", "why"),
                ] {
                    let selection = format!("n{x} |> {first}");
                    let other = format!("n{y} |> {second}");
                    let stage = evaluate(&mut runtime, &format!("{selection} |> union({other})"));
                    let accepted = evaluate(&mut runtime, &spelled(&selection, &other, ""));
                    assert_eq!(stage, accepted, "seed {seed}: {selection} union {other}");
                }
            }
        }
    }
}

#[test]
fn stages_after_a_union_keep_what_they_keep_of_each_side() {
    for seed in 1..=3 {
        let mut runtime = Runtime::new();
        checked(&mut runtime, &graph(seed, 9));
        for x in 0..9 {
            for y in [0, 4, 8] {
                let selection = format!("n{x} |> why");
                let other = format!("n{y} |> why");
                for filter in [
                    " |> stated",
                    " |> derived",
                    " |> depth(0)",
                    " |> depth(1)",
                    " |> depth(2)",
                    " |> path(n1)",
                    " |> path(n0)",
                    " |> intersect(n5 |> why)",
                    " |> except(n3 |> why)",
                ] {
                    let stage = evaluate(
                        &mut runtime,
                        &format!("{selection} |> union({other}){filter}"),
                    );
                    let accepted = evaluate(&mut runtime, &spelled(&selection, &other, filter));
                    assert_eq!(stage, accepted, "seed {seed}: n{x} union n{y}{filter}");
                }
                let names = evaluate(
                    &mut runtime,
                    &format!("{selection} |> union({other}) |> nodes"),
                );
                assert_eq!(
                    names,
                    nodes(&mut runtime, &spelled(&selection, &other, "")),
                    "seed {seed}: n{x} union n{y} nodes"
                );
                let walked = evaluate(
                    &mut runtime,
                    &format!("{selection} |> union({other}) |> why"),
                );
                assert_eq!(
                    walked,
                    evaluate(&mut runtime, &format!("{selection} |> union({other})")),
                    "seed {seed}: a why walk is closed under why"
                );
                let changed = evaluate(
                    &mut runtime,
                    &format!(
                        "suppose n1 = 7:\n    [{selection} |> union({other}) |> diff, {}]",
                        spelled(&selection, &other, " |> diff")
                    ),
                );
                assert_eq!(changed[0], changed[1], "seed {seed}: n{x} union n{y} diff");
            }
        }
    }
}

#[test]
fn distances_and_direction_follow_the_walk_that_selected_each_step() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, SIX);
    assert_eq!(
        nodes(&mut runtime, "why c |> union(why f) |> depth(1)"),
        json!(["c", "b", "f", "e", "d"])
    );
    assert_eq!(
        nodes(&mut runtime, "why c |> union(why d)"),
        json!(["c", "b", "a", "d"])
    );
    assert_eq!(
        nodes(&mut runtime, "why c |> union(why d) |> depth(1)"),
        json!(["c", "b", "d"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why c |> depth(1) |> union(why d |> depth(1))"
        ),
        json!(["c", "b", "d", "a"])
    );
    assert_eq!(
        nodes(&mut runtime, "why c |> union(why d) |> depth(0)"),
        json!(["c", "d"])
    );
    assert_eq!(
        nodes(&mut runtime, "why c |> union(model) |> depth(0)"),
        json!(["c", "d", "e", "f"])
    );
    assert_eq!(
        nodes(&mut runtime, "c |> union(why f) |> depth(1)"),
        json!(["c", "f", "e", "d"])
    );
    assert_eq!(
        nodes(&mut runtime, "why c |> union(affects a)"),
        json!(["c", "b", "a", "d", "f"])
    );
    assert_eq!(
        nodes(&mut runtime, "why c |> union(affects a) |> path(b)"),
        json!(["c", "b"])
    );
    assert_eq!(
        nodes(&mut runtime, "affects a |> union(why c)"),
        json!(["b", "d", "f", "c", "a"])
    );
    assert_eq!(
        nodes(&mut runtime, "affects a |> union(why c) |> path(b)"),
        json!(["b", "a"])
    );
    assert_eq!(
        nodes(&mut runtime, "affects a |> union(why c) |> depth(1)"),
        json!(["b", "d"])
    );
    assert_eq!(
        nodes(&mut runtime, "affects a |> union(why c) |> depth(2)"),
        json!(["b", "d", "f", "c", "a"])
    );
    for (union, path) in [
        ("why c |> union(affects a)", json!(["c", "b"])),
        ("affects a |> union(why c)", json!(["b", "a"])),
    ] {
        assert_eq!(nodes(&mut runtime, &format!("{union} |> path(b)")), path);
        assert_eq!(
            nodes(&mut runtime, &format!("{union} |> depth(9) |> path(b)")),
            path,
            "{union} ranked"
        );
        assert_eq!(
            nodes(&mut runtime, &format!("{union} |> path(b) |> depth(9)")),
            path,
            "{union} ranked after path"
        );
    }
    for (first, second, target) in [
        ("why c", "affects a", "b"),
        ("why f", "affects a", "d"),
        ("why c", "affects e", "a"),
    ] {
        assert_eq!(
            nodes(
                &mut runtime,
                &format!("{first} |> union({second}) |> path({target})")
            ),
            nodes(
                &mut runtime,
                &format!(
                    "({first} |> path({target})).concat({second} |> except({first}) |> intersect(model |> path({target})))"
                )
            ),
            "{first} union {second} path {target}"
        );
        assert_eq!(
            nodes(
                &mut runtime,
                &format!("{second} |> union({first}) |> path({target})")
            ),
            nodes(
                &mut runtime,
                &format!(
                    "({second} |> path({target})).concat({first} |> except({second}) |> intersect({target} |> why))"
                )
            ),
            "{second} union {first} path {target}"
        );
    }
    assert_eq!(
        nodes(&mut runtime, "why b |> union(why d) |> affects"),
        json!(["f", "b", "d", "c"])
    );
    assert_eq!(
        nodes(&mut runtime, "b |> union(d) |> affects"),
        json!(["f", "c"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "affects b |> union(affects d) |> why |> stated"
        ),
        json!(["e", "a"])
    );
}

#[test]
fn empty_and_disjoint_selections_join_in_order() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, SIX);
    assert_eq!(
        nodes(&mut runtime, "why e |> union(why c)"),
        json!(["e", "c", "b", "a"])
    );
    assert_eq!(
        nodes(&mut runtime, "why c |> union(why e)"),
        json!(["c", "b", "a", "e"])
    );
    assert_eq!(
        nodes(&mut runtime, "why c |> derived |> stated |> union(why e)"),
        json!(["e"])
    );
    assert_eq!(
        nodes(&mut runtime, "why c |> union(why e |> derived)"),
        json!(["c", "b", "a"])
    );
    assert_eq!(
        evaluate(&mut runtime, "why e |> derived |> union(why a |> derived)"),
        json!([])
    );
    assert_eq!(
        evaluate(&mut runtime, "affects f |> union(affects c)"),
        json!([])
    );
    checked(
        &mut runtime,
        "held = why e |> derived |> union(why a |> derived)",
    );
    assert_eq!(evaluate(&mut runtime, "held"), json!([]));
    checked(&mut runtime, "e = a + 1");
    assert_eq!(nodes(&mut runtime, "held"), json!(["e"]));
    checked(&mut runtime, "e = 7");
    assert_eq!(evaluate(&mut runtime, "held"), json!([]));

    let mut follows = Runtime::new();
    checked(
        &mut follows,
        &format!("{SIX}held = why e |> derived |> union(why c)\n"),
    );
    assert_eq!(nodes(&mut follows, "held"), json!(["c", "b", "a"]));
    checked(&mut follows, "c = b * 3 + e");
    assert_eq!(nodes(&mut follows, "held"), json!(["c", "b", "e", "a"]));
    checked(&mut follows, "e = 9");
    assert_eq!(
        evaluate(&mut follows, "held.map(s -> s.holds)"),
        json!([15.0, 2.0, 9.0, 1.0])
    );
}

#[test]
fn identities_hold_for_the_same_selection_model_and_repeated_stages() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{ORDER}total = subtotal + tax + shipping\n"),
    );
    let model = evaluate(&mut runtime, "model |> intersect(model)");
    for source in [
        "why tax",
        "affects quantity",
        "tax |> why |> stated",
        "model |> derived",
        "shipping",
    ] {
        let selection = format!("{source} |> intersect(model)");
        let whole = evaluate(&mut runtime, &selection);
        assert_eq!(
            evaluate(&mut runtime, &format!("{source} |> union({source})")),
            whole,
            "{source}"
        );
        assert_eq!(
            evaluate(&mut runtime, &format!("{source} |> union(model)")),
            evaluate(&mut runtime, &spelled(&selection, "model", "")),
            "{source}"
        );
        assert_eq!(
            evaluate(&mut runtime, &format!("model |> union({source})")),
            model,
            "{source}"
        );
        for other in ["why shipping", "affects price", "rate"] {
            let once = evaluate(&mut runtime, &format!("{source} |> union({other})"));
            assert_eq!(
                evaluate(
                    &mut runtime,
                    &format!("{source} |> union({other}) |> union({other})")
                ),
                once,
                "{source} {other}"
            );
            assert_eq!(
                evaluate(
                    &mut runtime,
                    &format!("{source} |> union({other}) |> except({other})")
                ),
                evaluate(&mut runtime, &format!("{source} |> except({other})")),
                "{source} {other}"
            );
            assert_eq!(
                evaluate(
                    &mut runtime,
                    &format!("{source} |> union({other}) |> intersect({source})")
                ),
                whole,
                "{source} {other}"
            );
            assert_eq!(
                evaluate(
                    &mut runtime,
                    &format!("{source} |> union({other}) |> union(why total)")
                ),
                evaluate(
                    &mut runtime,
                    &format!("{source} |> union({other} |> union(why total))")
                ),
                "{source} {other}"
            );
        }
    }
    assert_eq!(
        nodes(&mut runtime, "why tax |> union(why shipping)"),
        json!(["tax", "subtotal", "rate", "price", "quantity", "shipping"])
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> union(why tax)"),
        json!(["shipping", "quantity", "tax", "subtotal", "rate", "price"])
    );
    assert_eq!(
        nodes(&mut runtime, "why total |> union(why tax)"),
        nodes(&mut runtime, "why total")
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> union(why total)"),
        json!([
            "tax", "subtotal", "rate", "price", "quantity", "total", "shipping"
        ])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> stated |> union(why shipping)"),
        json!(["rate", "price", "quantity", "shipping"])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> union(why shipping) |> stated"),
        json!(["rate", "price", "quantity"])
    );
}

#[test]
fn every_kind_of_other_selection_selects_as_a_pipeline_source() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}order = {{fee: 2}}\norder.cost = order.fee + quantity\nclass Item(price, qty):\n    this.price = price\n    this.qty = qty\n    def joined() => (this.total |> why |> union(this.qty)).map(s -> s.node)\n$Item.total = $Item.price * $Item.qty\n$Item.joined = (why $Item.price |> union($Item.total)).map(s -> s.node)\npen = Item(2, 5)\n"
        ),
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> union(price)"),
        json!(["shipping", "quantity", "price"])
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> union(quantity)"),
        json!(["shipping", "quantity"])
    );
    let cost = evaluate(&mut runtime, "(order.cost |> why)[0].node");
    assert_eq!(
        nodes(&mut runtime, "why shipping |> union(order.cost)"),
        json!(["shipping", "quantity", cost.clone()])
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> union(why order.cost)"),
        json!(["shipping", "quantity", cost, "order"])
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> union(model |> stated)"),
        json!([
            "shipping", "quantity", "price", "rate", "order", "$Item", "pen"
        ])
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> union(model |> stated)"),
        nodes(
            &mut runtime,
            &spelled("why shipping", "model |> stated", "")
        )
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why shipping |> union(subtotal |> why |> derived)"
        ),
        json!(["shipping", "quantity", "subtotal"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why shipping |> union(why tax |> union(why order.fee))"
        ),
        nodes(
            &mut runtime,
            "why shipping |> union(why tax) |> union(why order.fee)"
        )
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why shipping |> union(why tax |> intersect(why subtotal))"
        ),
        json!(["shipping", "quantity", "subtotal", "price"])
    );
    assert_eq!(
        evaluate(&mut runtime, "[pen.joined(), pen.joined]"),
        json!([
            ["pen.total", "pen.price", "pen.qty"],
            ["pen.price", "pen.total"]
        ])
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> union(order.missing)"),
        json!(["shipping", "quantity"])
    );
    let mut filled = Runtime::new();
    checked(
        &mut filled,
        &format!("{CHAIN}o = {{}}\nheld = why b |> union(o.p)\n"),
    );
    assert_eq!(nodes(&mut filled, "held"), json!(["b", "a"]));
    checked(&mut filled, "o.p = 1");
    let property = evaluate(&mut filled, "(o.p |> why)[0].node");
    assert_eq!(nodes(&mut filled, "held"), json!(["b", "a", property]));
    checked(&mut filled, "delete o.p");
    assert_eq!(nodes(&mut filled, "held"), json!(["b", "a"]));
}

#[test]
fn a_name_that_holds_a_selection_adds_nothing() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{ORDER}causes = why shipping\n"));
    assert_eq!(
        nodes(&mut runtime, "why tax |> union(causes)"),
        nodes(&mut runtime, "why tax")
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> union(why shipping)"),
        json!(["tax", "subtotal", "rate", "price", "quantity", "shipping"])
    );
    assert_eq!(
        nodes(&mut runtime, "causes |> union(why tax)"),
        nodes(&mut runtime, "why tax")
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
        (
            "shipping |> why |> nodes",
            error(ErrorKind::Type, "Cannot explain shipping|>why|>nodes", 7, 1),
        ),
    ] {
        assert_eq!(
            failure(&format!("{ORDER}why tax |> union({other})")),
            expected,
            "{other}"
        );
        assert_eq!(
            failure(&format!("{ORDER}why tax |> union({other}) |> depth(1)")),
            expected,
            "{other} ranked"
        );
    }
    assert_eq!(
        failure(&format!(
            "{ORDER}class Item(price):\n    this.price = price\nwhy tax |> union($Item.price)"
        )),
        error(ErrorKind::Type, "Cannot explain $Item.price", 9, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}o = {{p: null}}\nwhy tax |> union(o.p.q)")),
        error(ErrorKind::Reference, "o.p is not defined", 8, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}1 |> union(missing)")),
        error(ErrorKind::Type, "Cannot explain 1", 7, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}missing |> union(1)")),
        error(ErrorKind::Reference, "missing is not defined", 7, 1)
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{ORDER}held = why tax |> union(why shipping)\n"),
    );
    assert_eq!(
        failure_in(&mut runtime, "held = why tax |> union(missing)"),
        error(ErrorKind::Reference, "missing is not defined", 1, 1)
    );
    assert_eq!(
        nodes(&mut runtime, "held"),
        json!(["tax", "subtotal", "rate", "price", "quantity", "shipping"])
    );
}

#[test]
fn zero_or_two_arguments_report_the_errors_of_parentheses() {
    assert_eq!(
        failure(&format!("{ORDER}why tax |> union()\n")),
        error(ErrorKind::Syntax, "Unexpected ')'", 7, 18)
    );
    assert_eq!(
        failure(&format!(
            "{ORDER}why tax |> union(why shipping, why price)\n"
        )),
        error(ErrorKind::Syntax, "Expected ')' but found ','", 7, 30)
    );
    assert_eq!(
        failure(&format!("{ORDER}why tax |> union(why shipping,)\n")),
        error(ErrorKind::Syntax, "Expected ')' but found ','", 7, 30)
    );
    assert_eq!(
        failure(&format!("{ORDER}why tax |> union(why shipping\n")),
        error(
            ErrorKind::Syntax,
            "Expected ')' but found end of input",
            8,
            1
        )
    );
    assert_eq!(
        failure(&format!("{ORDER}why tax |> union(\n")),
        error(ErrorKind::Syntax, "Unexpected end of input", 8, 1)
    );
    let mut deep = String::from("why c");
    for _ in 0..70 {
        deep = format!("why c |> union({deep})");
    }
    let (kind, message, _) = failure(&format!("{CHAIN}x = {deep}"));
    assert_eq!(
        (kind, message.as_str()),
        (ErrorKind::Syntax, "Expressions are nested too deeply")
    );
    let mut nested = String::from("why c");
    for _ in 0..30 {
        nested = format!("why c |> union({nested})");
    }
    assert_eq!(result(&format!("{CHAIN}({nested}).length")), json!(3.0));
    assert_eq!(
        result(&format!("{CHAIN}({nested} |> depth(1)).length")),
        json!(2.0)
    );
}

#[test]
fn other_selections_built_past_the_parser_limit_raise_the_depth_error() {
    let mut source = String::from("why c");
    let mut parsed = 0;
    for levels in 1..=100 {
        let deeper = format!("why c |> union({source})");
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

    let exceeded = unpositioned(ErrorKind::Type, "Maximum expression depth exceeded");
    for build in [chain, ranked_chain] {
        let mut runtime = Runtime::new();
        checked(&mut runtime, CHAIN);
        let steps = resolved(&mut runtime, &build(0)).unwrap();
        assert_eq!(steps.as_array().map(Vec::len), Some(3));
        assert_eq!(resolved(&mut runtime, &build(parsed)), Ok(steps.clone()));
        let mut limit = None;
        for levels in parsed + 1..=1_000 {
            match resolved(&mut runtime, &build(levels)) {
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
                resolved(&mut runtime, &build(levels)),
                Err(exceeded.clone()),
                "{levels}"
            );
        }
        assert_eq!(resolved(&mut runtime, &build(limit)), Ok(steps));
        checked(&mut runtime, "d = c + 1");
        assert_eq!(
            nodes(&mut runtime, "why b |> union(why d)"),
            json!(["b", "a", "d", "c"])
        );
    }
}

#[test]
fn every_binding_named_union_shadows_the_stage() {
    let bindings = [
        ("union = 1\nwhy c |> union(why b)", 5),
        ("def union(x) => x\nwhy c |> union(why b)", 5),
        ("class union\nwhy c |> union(why b)", 5),
        ("union = 1\nwhy c |> union(missing)", 5),
        ("union = 1\nmissing |> union(why b)", 5),
        ("union = 1\n1 |> union(why b)", 5),
        ("union = 1\nwhy c |> union(why b) |> depth(1)", 5),
        ("def f(union) => why c |> union(why b)\nf(1)", 5),
        ("def g union => why c |> union(why b)\ng(1)", 5),
        ("f = union -> why c |> union(why b)\nf(1)", 5),
        ("f = (union) => why c |> union(why b)\nf(1)", 5),
        ("f = lambda union: why c |> union(why b)\nf(1)", 5),
        ("[1].map(union -> why c |> union(why b))", 4),
        ("for union of \"x\":\n    y = why c |> union(why b)", 4),
        (
            "ok = all union in [1]: (why c |> union(why b)).length > 0",
            4,
        ),
        ("{\n    union = 5\n    y = why c |> union(why b)\n}", 4),
        (
            "class R:\n    def m(union) => why c |> union(why b)\nR().m(1)",
            6,
        ),
        (
            "class Q(union):\n    this.v = (why c |> union(why b)).length\nQ(1)",
            6,
        ),
        (
            "try:\n    throw 1\ncatch union:\n    y = why c |> union(why b)",
            4,
        ),
        ("{union} = {union: 1}\nwhy c |> union(why b)", 5),
    ];
    for (binding, line) in bindings {
        assert_eq!(
            failure(&format!("{CHAIN}{binding}")),
            error(ErrorKind::Type, "union is not an operation", line, 1),
            "{binding}"
        );
    }
    assert_eq!(
        result(&format!(
            "{CHAIN}union = 1\ncaught = \"\"\ntry:\n    why c |> union(why b)\ncatch error:\n    caught = error\n[union, caught]"
        )),
        json!([1.0, "TypeError: union is not an operation"])
    );
    let mut runtime = Runtime::new();
    checked(&mut runtime, CHAIN);
    assert_eq!(
        failure_in(&mut runtime, "x = 5\nunion = 1\ny = why c |> union(why b)"),
        error(ErrorKind::Type, "union is not an operation", 3, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\nwhy c |> union(why b |> stated)"
        )),
        error(ErrorKind::Type, "stated is not an operation", 5, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\nmissing |> union(why b |> stated)"
        )),
        error(ErrorKind::Reference, "missing is not defined", 5, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\nunion = 2\nwhy c |> union(why b) |> stated"
        )),
        error(ErrorKind::Type, "stated is not an operation", 6, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\nunion = 2\nwhy c |> stated |> union(why b)"
        )),
        error(ErrorKind::Type, "union is not an operation", 6, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}except = 1\nwhy c |> union(why b |> except(a))"
        )),
        error(ErrorKind::Type, "except is not an operation", 5, 1)
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}except = 1\nintersect = 2\nnodes = 3\n(why b |> union(why c)).length"
        )),
        json!(3.0)
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}union = 1\n[(why c |> except(why a)).length, (why c |> stated).length]"
        )),
        json!([2.0, 1.0])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}why = 1\naffects = 2\n(why b |> union(why c)).length"
        )),
        json!(3.0)
    );
}

#[test]
fn a_binding_declared_after_a_standing_union_fails_its_refresh() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}held = why b |> union(why c)\nexcept = 5"),
    );
    assert_eq!(nodes(&mut runtime, "held"), json!(["b", "a", "c"]));
    checked(&mut runtime, "union = 5");
    assert_eq!(
        failure_in(&mut runtime, "a = 3"),
        error(ErrorKind::Type, "union is not an operation", 1, 1)
    );
    assert_eq!(evaluate(&mut runtime, "a"), json!(1.0));
    checked(&mut runtime, "delete union\na = 3");
    assert_eq!(
        evaluate(&mut runtime, "held.map(s -> s.holds)"),
        json!([5.0, 3.0, 10.0])
    );
}

#[test]
fn an_undefined_side_makes_the_union_undefined() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}n = null\nheld = why c |> union(why b |> depth(n))\n"),
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "[held, why c |> union(why b |> depth(n)), why c |> depth(n) |> union(missing), why c |> union(why b |> depth(n)) |> union(missing), why c |> union(why b |> depth(n)) |> depth(missing), why c |> union(why b |> depth(n)) |> nodes]"
        ),
        json!([null, null, null, null, null, null])
    );
    assert_eq!(evaluate(&mut runtime, "held.length"), Json::Null);
    assert_eq!(
        evaluate(
            &mut runtime,
            "all s in why c |> union(why b |> depth(n)): s.holds > 0"
        ),
        Json::Null
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "(why c).concat(why b |> depth(n) |> except(why c)).map(s -> s == undefined)"
        ),
        json!([false, false, false, true])
    );
    checked(&mut runtime, "n = 0");
    assert_eq!(nodes(&mut runtime, "held"), json!(["c", "b", "a"]));
    assert_eq!(
        failure_in(&mut runtime, "why c |> depth(n) |> union(missing)"),
        error(ErrorKind::Reference, "missing is not defined", 1, 1)
    );
    checked(&mut runtime, "n = undefined");
    assert_eq!(evaluate(&mut runtime, "held"), Json::Null);

    let mut waiting = Runtime::new();
    checked(
        &mut waiting,
        &format!("{CHAIN}n = null\nskipped = why c |> depth(n) |> union(missing)\n"),
    );
    assert_eq!(evaluate(&mut waiting, "skipped"), Json::Null);
    assert_eq!(
        failure_in(&mut waiting, "n = 1"),
        error(ErrorKind::Reference, "missing is not defined", 1, 1)
    );
    assert_eq!(evaluate(&mut waiting, "[n, skipped]"), json!([null, null]));
}

#[test]
fn standing_unions_follow_both_sides_and_the_steps_they_drop() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}fee = 1\neither = why tax |> union(why shipping)\nfacts = why tax |> union(why shipping) |> stated\nfirst = why tax |> stated |> union(why shipping |> stated)\n"
        ),
    );
    assert_eq!(
        nodes(&mut runtime, "either"),
        json!(["tax", "subtotal", "rate", "price", "quantity", "shipping"])
    );
    assert_eq!(
        nodes(&mut runtime, "facts"),
        json!(["rate", "price", "quantity"])
    );
    checked(&mut runtime, "shipping = quantity * 2 + fee");
    assert_eq!(
        nodes(&mut runtime, "either"),
        json!([
            "tax", "subtotal", "rate", "price", "quantity", "shipping", "fee"
        ])
    );
    assert_eq!(
        nodes(&mut runtime, "facts"),
        json!(["rate", "price", "quantity", "fee"])
    );
    checked(&mut runtime, "rate = fee / 5");
    assert_eq!(
        nodes(&mut runtime, "either"),
        json!([
            "tax", "subtotal", "rate", "fee", "price", "quantity", "shipping"
        ])
    );
    assert_eq!(
        nodes(&mut runtime, "facts"),
        json!(["fee", "price", "quantity"])
    );
    assert_eq!(nodes(&mut runtime, "first"), nodes(&mut runtime, "facts"));
    checked(&mut runtime, "subtotal = 30");
    assert_eq!(
        nodes(&mut runtime, "facts"),
        json!(["subtotal", "fee", "quantity"])
    );
    assert_eq!(
        nodes(&mut runtime, "either"),
        json!(["tax", "subtotal", "rate", "fee", "shipping", "quantity"])
    );
    checked(&mut runtime, "price = 12\nquantity = 5");
    assert_eq!(
        evaluate(&mut runtime, "either.map(s -> s.holds)"),
        json!([6.0, 30.0, 0.2, 1.0, 11.0, 5.0])
    );
    checked(&mut runtime, "delete shipping");
    assert_eq!(
        nodes(&mut runtime, "either"),
        json!(["tax", "subtotal", "rate", "fee"])
    );
    checked(&mut runtime, "shipping = price");
    assert_eq!(
        nodes(&mut runtime, "either"),
        json!(["tax", "subtotal", "rate", "fee", "shipping", "price"])
    );
}

#[test]
fn standing_unions_observe_both_walks_and_model_membership() {
    let output = cli(
        "observes",
        &format!(
            "{CHAIN}d = 4\nx = why b |> union(model)\ny = why b |> union(why d)\nz = why d |> union(why c |> stated)\n1"
        ),
    );
    assert!(output.status.success(), "{output:?}");
    let listing = String::from_utf8(output.stdout).unwrap();
    assert!(
        listing
            .contains("  x [variable]\n    reads    b, a, c, d\n    observes b, a, model, c, d\n"),
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
            "{CHAIN}held = why a |> union(model |> derived)\nempty = a |> stated |> union(b |> stated)\n"
        ),
    );
    assert_eq!(nodes(&mut runtime, "held"), json!(["a", "b", "c"]));
    assert_eq!(nodes(&mut runtime, "empty"), json!(["a"]));
    checked(&mut runtime, "b = 7");
    assert_eq!(nodes(&mut runtime, "held"), json!(["a", "c"]));
    assert_eq!(nodes(&mut runtime, "empty"), json!(["a", "b"]));
    checked(&mut runtime, "a = b + 1");
    assert_eq!(nodes(&mut runtime, "held"), json!(["a", "b", "c"]));
    assert_eq!(nodes(&mut runtime, "empty"), json!(["b"]));
    checked(&mut runtime, "e = c + 1");
    assert_eq!(nodes(&mut runtime, "held"), json!(["a", "b", "c", "e"]));
}

#[test]
fn class_rules_functions_methods_lambdas_quantifiers_and_conditions_hold_the_stage() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Order(qty, price, fee):\n    this.qty = qty\n    this.price = price\n    this.fee = fee\n    def own() => (this.cost |> why |> union(this.total |> why)).map(s -> s.node)\n$Order.total = $Order.qty * $Order.price\n$Order.cost = $Order.qty + $Order.fee\n$Order.either = why $Order.total |> union(why $Order.cost)\n$Order.big = any s in why $Order.total |> union(why $Order.cost): s.holds > 9\nif (why $Order.total |> union(why $Order.cost)).length > 4:\n    $Order.linked = true\npen = Order(2, 5, 1)\ncup = Order(4, 3, 2)\ndef joined() => (why pen.total |> union(why pen.cost)).length\npick = [1].map(v -> (why pen.total |> union(why cup.total)).length)\n",
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "[pen.either.map(s -> s.node), cup.either.map(s -> s.node), pen.big, cup.big, pen.linked, pen.own()]"
        ),
        json!([
            ["pen.total", "pen.qty", "pen.price", "pen.cost", "pen.fee"],
            ["cup.total", "cup.qty", "cup.price", "cup.cost", "cup.fee"],
            true,
            true,
            true,
            ["pen.cost", "pen.qty", "pen.fee", "pen.total", "pen.price"]
        ])
    );
    assert_eq!(
        evaluate(&mut runtime, "[joined(), pick]"),
        json!([5.0, [6.0]])
    );
    checked(&mut runtime, "pen.price = 1");
    assert_eq!(
        evaluate(&mut runtime, "[pen.big, cup.big]"),
        json!([false, true])
    );
    checked(&mut runtime, "$Order.cost = $Order.fee * 2");
    assert_eq!(
        evaluate(
            &mut runtime,
            "[pen.either.map(s -> s.node), cup.either.map(s -> s.node)]"
        ),
        json!([
            ["pen.total", "pen.qty", "pen.price", "pen.cost", "pen.fee"],
            ["cup.total", "cup.qty", "cup.price", "cup.cost", "cup.fee"]
        ])
    );
    checked(&mut runtime, "jar = Order(1, 2, 3)");
    assert_eq!(
        nodes(&mut runtime, "jar.either"),
        json!(["jar.total", "jar.qty", "jar.price", "jar.cost", "jar.fee"])
    );
    let text = result(&format!(
        "{CHAIN}`${{(why b |> union(why c)).length}} joined`"
    ));
    assert_eq!(text, json!("3 joined"));
}

#[test]
fn class_rules_check_the_names_and_cycles_in_other() {
    let class = "class Order(x):\n    this.x = x\n";
    assert_eq!(
        failure(&format!(
            "{class}$Order.k = why $Order.x |> union(why missing)\n1"
        )),
        error(ErrorKind::Reference, "missing is not defined", 3, 1)
    );
    assert_eq!(
        failure(&format!(
            "{class}$Order.y = (why $Order.x |> union(why $Order.y)).length\no = Order(1)\no.y"
        )),
        error(ErrorKind::Type, "Circular Dependency", 3, 1)
    );
    assert_eq!(
        failure(&format!(
            "{class}$Order.a = (why $Order.x |> union(why $Order.b)).length\n$Order.b = (why $Order.x |> union(why $Order.a)).length\no = Order(1)\n[o.a, o.b]"
        )),
        error(ErrorKind::Type, "Circular Dependency", 4, 1)
    );
    assert_eq!(
        result(&format!(
            "{class}$Order.k = (why $Order.x |> union(why $Order.z)).map(s -> s.node)\no = Order(1)\no.z = o.x + 1\n[o.k, ((why o.x).concat(why o.z |> except(why o.x))).map(s -> s.node)]"
        )),
        json!([["o.x", "o.z", "o"], ["o.x", "o.z", "o"]])
    );
}

#[test]
fn a_declaration_holding_a_union_stays_out_of_its_own_model() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = 1\nb = a + 1\nall_of = model |> union(why b)\nmore = why b |> union(model)\n",
    );
    assert_eq!(nodes(&mut runtime, "all_of"), json!(["a", "b"]));
    assert_eq!(nodes(&mut runtime, "more"), json!(["b", "a"]));
    assert_eq!(nodes(&mut runtime, "model |> why"), json!(["a", "b"]));
    assert_eq!(evaluate(&mut runtime, "(why all_of).length"), json!(0.0));
    assert_eq!(nodes(&mut runtime, "affects a"), json!(["b"]));
    checked(&mut runtime, "c = b + 1");
    assert_eq!(nodes(&mut runtime, "all_of"), json!(["a", "b", "c"]));
    assert_eq!(nodes(&mut runtime, "more"), json!(["b", "a", "c"]));
    checked(
        &mut runtime,
        "held = 1 if true else (why c |> union(why b))\nwalked = 1 if true else (why c)\nplain = 1 if true else c\n",
    );
    assert_eq!(
        nodes(&mut runtime, "model |> stated"),
        json!(["a", "plain"])
    );
    assert_eq!(
        nodes(&mut runtime, "all_of"),
        json!(["a", "b", "c", "plain"])
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "[held, walked, plain, (why held).length, (why walked).length]"
        ),
        json!([1.0, 1.0, 1.0, 0.0, 0.0])
    );
}

#[test]
fn declarations_that_call_a_function_holding_a_union_stay_live() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}def counted() => (why tax |> union(why shipping)).length\nheld = counted()\n"
        ),
    );
    assert_eq!(evaluate(&mut runtime, "held"), json!(6.0));
    checked(&mut runtime, "fee = 1\nshipping = price + fee");
    assert_eq!(evaluate(&mut runtime, "held"), json!(7.0));
    checked(&mut runtime, "subtotal = quantity * 2");
    assert_eq!(evaluate(&mut runtime, "held"), json!(7.0));
    checked(&mut runtime, "shipping = 1");
    assert_eq!(evaluate(&mut runtime, "held"), json!(5.0));
}

#[test]
fn failed_changes_suppose_blocks_and_diff_restore_unions() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{ORDER}either = why subtotal |> union(why shipping)\n"),
    );
    let initial = json!(["subtotal", "price", "quantity", "shipping"]);
    checked(
        &mut runtime,
        "try:\n    fee = 2\n    shipping = quantity * 2 + fee\n    assert(either.map(s -> s.node), [\"subtotal\", \"price\", \"quantity\", \"shipping\", \"fee\"])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert(either.map(s -> s.node), [\"subtotal\", \"price\", \"quantity\", \"shipping\"])",
    );
    assert_eq!(
        failure_in(
            &mut runtime,
            "shipping = price + quantity\nshipping = missing"
        ),
        error(ErrorKind::Reference, "missing is not defined", 2, 1)
    );
    assert_eq!(nodes(&mut runtime, "either"), initial);
    let plan = evaluate(
        &mut runtime,
        "suppose shipping = rate + 1:\n    [either.map(s -> s.node), (why tax |> diff |> union(why shipping |> diff)).map(s -> s.node), (why tax |> union(why shipping) |> diff).map(s -> s.node)]",
    );
    assert_eq!(
        plan,
        json!([
            ["subtotal", "price", "quantity", "shipping", "rate"],
            ["shipping"],
            ["shipping"]
        ])
    );
    let moved = evaluate(
        &mut runtime,
        "suppose price = 20:\n    [(why tax |> union(why shipping) |> diff).map(s -> s.node), (why shipping |> union(why tax) |> diff).map(s -> s.node)]",
    );
    assert_eq!(
        moved,
        json!([["tax", "subtotal", "price"], ["tax", "subtotal", "price"]])
    );
    assert_eq!(nodes(&mut runtime, "either"), initial);
    assert_eq!(evaluate(&mut runtime, "price"), json!(10.0));
}

#[test]
fn snapshots_freeze_unions_and_the_limits_inside_other() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}n = 0\nsnap = (why tax |> union(why shipping)).value\nnear = why rate |> union(why tax |> depth(n.value))\nlive = why rate |> union(why tax |> depth(n))\nnested = why rate |> union(why price |> union(why tax |> depth(n.value)))\nshared = why rate |> intersect(why tax |> union(why subtotal |> depth(n.value)))\n"
        ),
    );
    assert_eq!(
        stored(&runtime, "near"),
        "rate|>why|>union(tax|>why|>depth(0))"
    );
    assert_eq!(
        stored(&runtime, "nested"),
        "rate|>why|>union(price|>why|>union(tax|>why|>depth(0)))"
    );
    assert_eq!(
        stored(&runtime, "shared"),
        "rate|>why|>intersect(tax|>why|>union(subtotal|>why|>depth(0)))"
    );
    assert_eq!(
        stored(&runtime, "live"),
        "rate|>why|>union(tax|>why|>depth(n))"
    );
    assert_eq!(nodes(&mut runtime, "near"), json!(["rate", "tax"]));
    checked(&mut runtime, "n = 1\nshipping = price + 1");
    assert_eq!(
        nodes(&mut runtime, "snap"),
        json!(["tax", "subtotal", "rate", "price", "quantity", "shipping"])
    );
    assert_eq!(nodes(&mut runtime, "near"), json!(["rate", "tax"]));
    assert_eq!(
        nodes(&mut runtime, "live"),
        json!(["rate", "tax", "subtotal"])
    );
    checked(&mut runtime, "quantity = 4");
    assert_eq!(
        evaluate(&mut runtime, "snap.map(s -> s.holds)"),
        json!([6.0, 30.0, 0.2, 10.0, 3.0, 6.0])
    );
}

#[test]
fn steps_are_matched_by_node_where_distinct_compares_values() {
    assert_eq!(
        result(
            "x = 0 / 0\ny = x + 1\nz = x * 2\n[(why y |> union(why z)).map(s -> s.node), (why y).concat(why z).distinct().map(s -> s.node), (why y).concat(why z |> except(why y)).map(s -> s.node)]"
        ),
        json!([["y", "x", "z"], ["y", "x", "z", "x"], ["y", "x", "z"]])
    );
    assert_eq!(
        result(&format!(
            "{ORDER}[(why tax |> union(why shipping)).map(s -> s.node), (why tax).concat(why shipping).distinct().map(s -> s.node)]"
        )),
        json!([
            ["tax", "subtotal", "rate", "price", "quantity", "shipping"],
            ["tax", "subtotal", "rate", "price", "quantity", "shipping"]
        ])
    );
}

#[test]
fn unions_render_once_and_parse_back() {
    for source in [
        "why c |> union(why b)",
        "c |> why |> union(b |> why |> stated)",
        "model |> union(model) |> stated",
        "a |> affects |> union(b |> affects) |> why |> union(a)",
        "why c |> union(o.p)",
        "why c |> union(why b |> depth(n + 1)) |> depth(1)",
        "why c |> path(a) |> union(why b |> path(a))",
        "a || b |> union(c)",
        "why c |> union(why b |> union(why a))",
        "why c |> union(why b |> intersect(why a)) |> except(a)",
        "2 ** (why c |> union(why b))",
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
        parse_expression("why tax |> union(why shipping) |> derived")
            .unwrap()
            .to_string(),
        "tax|>why|>union(shipping|>why)|>derived"
    );
    assert_eq!(
        parse_expression("why tax |> union(model)")
            .unwrap()
            .to_string(),
        "tax|>why|>union(model)"
    );
    assert_eq!(
        parse_expression("2 ** (why c |> union(why b))")
            .unwrap()
            .to_string(),
        "2**(c|>why|>union(b|>why))"
    );
    let mut chained = String::from("c");
    for level in 1..=6 {
        chained.push_str(" |> union(a)");
        let rendered = parse_expression(&chained).unwrap().to_string();
        assert_eq!(rendered.len(), 1 + 10 * level, "{rendered}");
    }
    assert_eq!(
        generate(&parse("def f(x = 1) => why c |> union(why b)").unwrap()[0]),
        "def f(x=1):\n    return c|>why|>union(b|>why)"
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}held = why b |> union(why c)\n"),
    );
    assert_eq!(stored(&runtime, "held"), "b|>why|>union(c|>why)");
    assert_eq!(nodes(&mut runtime, "why c"), json!(["c", "b", "a"]));
    assert_eq!(nodes(&mut runtime, "affects a"), json!(["b", "c"]));
    assert_eq!(nodes(&mut runtime, "model |> why"), json!(["a", "b", "c"]));
    let output = cli(
        "keys",
        &format!(
            "{CHAIN}if (why b |> union(why c)).length > 2:\n    flag = 1\nheld = why b |> union(why c)\n1"
        ),
    );
    assert!(output.status.success(), "{output:?}");
    let listing = String::from_utf8(output.stdout).unwrap();
    assert!(
        listing.contains("  if(b|>why|>union(c|>why).length>2) [if]\n"),
        "{listing}"
    );
    assert!(
        listing.contains("  held [variable]\n    reads    b, a, c\n    observes b, a, c\n"),
        "{listing}"
    );
}

#[test]
fn unions_handle_large_graphs() {
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
            &format!("(why w{last} |> union(why v{last})).length")
        ),
        json!(40_000.0)
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            &format!("(why v{last} |> union(why w0)).length")
        ),
        json!(20_001.0)
    );
    assert_eq!(
        evaluate(&mut runtime, "(affects v0 |> union(affects w0)).length"),
        json!(39_999.0)
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            &format!("(why v{last} |> union(why w{last}) |> depth(1)).length")
        ),
        json!(4.0)
    );
}

#[test]
fn near_misses_keep_their_accepted_results() {
    assert_eq!(
        result(&format!(
            "{ORDER}`v ${{quantity why tax |> union(why shipping)}}`"
        )),
        json!("v 3")
    );
    assert_eq!(
        failure(&format!("{ORDER}why tax | > union(why shipping)")),
        error(ErrorKind::Syntax, "Unexpected character '|'", 7, 9)
    );
    assert_eq!(
        failure(&format!("{ORDER}why tax ||> union(why shipping)")),
        error(ErrorKind::Syntax, "Unexpected '>'", 7, 13)
    );
    assert_eq!(
        failure(&format!("{ORDER}tax.union(why shipping)")),
        error(ErrorKind::Type, "number.union is not a function", 7, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}why union(shipping)")),
        error(ErrorKind::Type, "Cannot explain union(shipping)", 7, 1)
    );
    assert_eq!(
        result(&format!(
            "{ORDER}(tax |> why).concat(shipping |> why |> except(tax |> why)).map(s -> s.node)"
        )),
        json!(["tax", "subtotal", "rate", "price", "quantity", "shipping"])
    );
}

#[test]
fn inputs_that_were_errors_now_give_what_the_proposal_lists() {
    assert_eq!(
        result(&format!(
            "{ORDER}either = why tax |> union(why shipping)\neither.map(s -> s.node)"
        )),
        json!(["tax", "subtotal", "rate", "price", "quantity", "shipping"])
    );
    assert_eq!(
        failure(&format!("{ORDER}union = 1\nwhy tax |> union(why shipping)")),
        error(ErrorKind::Type, "union is not an operation", 8, 1)
    );
    assert_eq!(
        result(&format!(
            "{ORDER}why tax |> union(why shipping |> depth(null))"
        )),
        Json::Null
    );
    assert_eq!(
        result(&format!(
            "{SIX}x, y = c |> why |> union(f |> why)\n[x.node, y.node]"
        )),
        json!(["c", "b"])
    );
    assert_eq!(
        result(&format!(
            "{SIX}{{length}} = c |> why |> union(f |> why)\nlength"
        )),
        json!(6.0)
    );
    assert_eq!(
        failure(&format!("{ORDER}[p] = why tax |> union(why shipping)\n")),
        error(ErrorKind::Syntax, "Cannot assign to [p]", 7, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}delete why tax |> union(why shipping)\n")),
        error(
            ErrorKind::Syntax,
            "Cannot delete tax|>why|>union(shipping|>why)",
            7,
            1
        )
    );
    assert_eq!(
        failure(&format!("{ORDER}not why tax |> union(why shipping)\n")),
        error(ErrorKind::Type, "Cannot explain !tax|>why", 7, 1)
    );
    assert_eq!(
        failure(&format!(
            "{ORDER}`v ${{quantity (why tax |> union(why shipping))}}`\n"
        )),
        error(ErrorKind::Type, "quantity is not a function", 7, 1)
    );
    assert_eq!(
        failure(&format!(
            "{ORDER}n = null\nfor s in why tax |> union(why shipping |> depth(n)):\n    s\n"
        )),
        error(
            ErrorKind::Type,
            "tax|>why|>union(shipping|>why|>depth(n)) is not iterable",
            8,
            1
        )
    );
    assert_eq!(
        result(&format!(
            "{ORDER}y = 1 if why tax |> union(why shipping) else 2\ny"
        )),
        json!(1.0)
    );
    assert_eq!(
        result(&format!(
            "{ORDER}x = suppose: (why tax |> union(why shipping)).length\nx"
        )),
        json!(6.0)
    );
    assert_eq!(
        result(&format!(
            "{ORDER}first = 1\n(why tax |> union(why shipping)).length + first"
        )),
        json!(7.0)
    );
    assert_eq!(
        result(&format!("{ORDER}why tax |> union(why shipping)(2)")),
        json!(2.0)
    );
}

#[test]
fn the_binary_runs_unions_from_files_and_the_line_repl() {
    let output = cli(
        "file",
        &format!("{CHAIN}(why b |> union(why c)).map(s -> s.node)\n"),
    );
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("[\"b\",\"a\",\"c\"]\n")
    );
    for ending in ["\n", "\r\n"] {
        let lines = [
            "a = 1",
            "b = a + 2",
            "c = b * 2",
            "(why b |> union(why c)).length",
            "why c |> union(",
            "(why a |> union(why b)).length",
            "why c |> union",
            "union = 1",
            "why c |> union(why b)",
        ];
        let output = repl(&["--plain", "--json"], &(lines.join(ending) + ending));
        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{BANNER}1.0\n3.0\n6.0\n3.0\n2.0\n1.0\n")
        );
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "SyntaxError: Unexpected end of input (line 2, column 1)\nTypeError: union is not an operation\nTypeError: union is not an operation (line 1, column 1)\n"
        );
        let plain = repl(&["--plain"], &(lines[..4].join(ending) + ending));
        assert!(plain.status.success(), "{plain:?}");
        assert_eq!(
            String::from_utf8(plain.stdout).unwrap(),
            format!("{BANNER}1\n3\n6\n3\n")
        );
    }
}
