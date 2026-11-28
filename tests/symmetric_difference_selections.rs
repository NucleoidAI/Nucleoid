mod common;

use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Ast, Expr, Stmt, parse, parse_expression};
use nucleoid::nuc::Nuc;
use nucleoid::reasoning::Stage;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde_json::{Value as Json, json};

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid tells two explanations apart with xor",
    "Nucleoid finds what a change of only one of two inputs reaches with xor",
    "Nucleoid separates what two changes of each instance reach with xor",
    "Nucleoid restores symmetric differences after failed changes",
];

const REFERENCE_TITLE: &str = "Nucleoid separates two explanations with xor";
const README_TITLE: &str = "Xor tells two alarms apart";
const EXAMPLES_TITLE: &str = "Xor lists what tells two diagnoses apart";

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

const CHAIN: &str = "a = 1\nb = a + 2\nc = b * 2\n";

const SIX: &str = "a = 1\nb = a + 1\nc = b * 2\nd = a + 3\ne = 7\nf = e + d\n";

const ORDER: &str = "price = 10\nquantity = 3\nrate = 0.2\nsubtotal = price * quantity\ntax = subtotal * rate\nshipping = quantity * 2\ntotal = subtotal + tax\nfreight = shipping + rate\n";

struct Counterpart {
    title: &'static str,
    rewrites: &'static [(&'static str, &'static str)],
    before: usize,
    after: usize,
}

const SPELLED: [Counterpart; 7] = [
    Counterpart {
        title: SPEC_TITLES[0],
        rewrites: &[
            (
                "total |> why |> xor(freight |> why)",
                "why total |> except(why freight) |> union(why freight |> except(why total))",
            ),
            (
                "why freight |> xor(why total)",
                "why freight |> except(why total) |> union(why total |> except(why freight))",
            ),
        ],
        before: 237,
        after: 215,
    },
    Counterpart {
        title: SPEC_TITLES[1],
        rewrites: &[
            (
                "price |> affects |> xor(rate |> affects)",
                "affects price |> except(affects rate) |> union(affects rate |> except(affects price))",
            ),
            (
                "affects quantity |> xor(affects price)",
                "affects quantity |> except(affects price) |> union(affects price |> except(affects quantity))",
            ),
        ],
        before: 197,
        after: 175,
    },
    Counterpart {
        title: SPEC_TITLES[2],
        rewrites: &[(
            "$Part.cost |> affects |> xor($Part.weight |> affects)",
            "affects $Part.cost |> except(affects $Part.weight) |> union(affects $Part.weight |> except(affects $Part.cost))",
        )],
        before: 253,
        after: 239,
    },
    Counterpart {
        title: SPEC_TITLES[3],
        rewrites: &[
            (
                "why subtotal |> xor(why shipping)",
                "why subtotal |> except(why shipping) |> union(why shipping |> except(why subtotal))",
            ),
            ("why subtotal |> xor(1)", "1 |> why"),
        ],
        before: 196,
        after: 188,
    },
    Counterpart {
        title: REFERENCE_TITLE,
        rewrites: &[
            (
                "apart = tax |> why |> xor(shipping |> why)\n",
                "apart = why tax |> except(why shipping) |> union(why shipping |> except(why tax))\n",
            ),
            (
                "facts = tax |> why |> xor(shipping |> why) |> stated",
                "facts = why tax |> except(why shipping) |> union(why shipping |> except(why tax)) |> stated",
            ),
        ],
        before: 146,
        after: 126,
    },
    Counterpart {
        title: README_TITLE,
        rewrites: &[(
            "why fire |> xor(why fog)",
            "why fire |> except(why fog) |> union(why fog |> except(why fire))",
        )],
        before: 114,
        after: 102,
    },
    Counterpart {
        title: EXAMPLES_TITLE,
        rewrites: &[(
            "why flu |> xor(why infection)",
            "why flu |> except(why infection) |> union(why infection |> except(why flu))",
        )],
        before: 101,
        after: 89,
    },
];

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "symmetric difference regression".to_string(),
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

fn sorted(value: &Json) -> Vec<String> {
    let mut names: Vec<String> = value
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap().to_string())
        .collect();
    names.sort();
    names
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

fn syntax(source: &str) -> String {
    let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    format!("{statements:?}")
}

fn xors(source: &str) -> usize {
    syntax(source).matches("Xor {").count()
}

fn unions(source: &str) -> usize {
    syntax(source).matches("Union {").count()
}

fn combines(source: &str) -> usize {
    syntax(source).matches("Combine {").count()
}

fn tokens(source: &str) -> usize {
    tokenize(source.trim())
        .unwrap()
        .iter()
        .filter(|spanned| spanned.token != Token::Eof)
        .count()
}

fn writes_bang_in(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(2).any(|pair| {
            pair[0].token == Token::Bang
                && pair[1].token == Token::Keyword(Keyword::In)
                && pair[1].position.line == pair[0].position.line
                && pair[1].position.column == pair[0].position.column + 1
        })
    })
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

fn xor(source: Box<Expr>, other: Box<Expr>) -> Expr {
    Expr::Xor { source, other }
}

fn chain(levels: usize) -> Expr {
    let mut tree = *reason(Stage::Why, identifier("c"));
    for _ in 0..levels {
        tree = xor(reason(Stage::Why, identifier("c")), Box::new(tree));
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
        std::env::temp_dir().join(format!("nucleoid-nuc139-{label}-{}", std::process::id()));
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
    format!("{selection} |> except({other}) |> union({other} |> except({selection})){stage}")
}

#[test]
fn appended_document_cases_run_every_assertion() {
    let [spec, reference, readme, examples] = documents();
    let appended = SPEC_TITLES
        .iter()
        .zip([None, Some("1"), Some("2"), None])
        .map(|(title, expected)| (&spec, *title, expected))
        .chain([
            (
                &reference,
                REFERENCE_TITLE,
                Some("[\"rate\",\"price\",\"fee\"]"),
            ),
            (&readme, README_TITLE, None),
            (&examples, EXAMPLES_TITLE, Some("[[\"white_cells\"],true]")),
        ]);

    let mut assertions = 0;
    for (cases, title, expected) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(xors(&case.source) > 0, "{title} writes an xor stage");
        assert_eq!(unions(&case.source), 0, "{title} writes no union stage");
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
        .split_once("### Symmetric difference selections\n\n```\n")
        .expect("the language reference documents xor");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(
        case.expected.as_deref(),
        Some("[\"rate\",\"price\",\"fee\"]")
    );
    for line in [
        "[NUC 139](nuc-0139.md)",
        "why a |> xor(why b)                   what one of a and b rests on and the other does not (NUC 139)",
        "affects a |> xor(affects b)           what a change of exactly one of a and b reaches (NUC 139)",
        "selection |> xor(other)               steps whose node other lacks, then other's steps whose node they lack (NUC 139)",
        "selection |> xor(other) |> stated     the stated steps that tell two selections apart (NUC 139)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    let union = reference.find("### Union selections").unwrap();
    let apart = reference
        .find("### Symmetric difference selections")
        .unwrap();
    let ends = reference
        .find("### Origins and outcomes of reasoning selections")
        .unwrap();
    assert!(union < apart && apart < ends);
    assert!(include_str!("../README.md").contains("[NUC 139](docs/nuc-0139.md)"));
    assert!(include_str!("../docs/examples.md").contains("[NUC 139](nuc-0139.md)"));
    let spec = include_str!("../nucleoid.spec.md").replace("\r\n", "\n");
    let contract = spec
        .find("\n## Symmetric difference reasoning selections\n")
        .unwrap();
    assert!(spec.find("\n## Union reasoning selections\n").unwrap() < contract);
    assert!(
        contract
            < spec
                .find("\n## Origins and outcomes of reasoning selections\n")
                .unwrap()
    );
}

#[test]
fn no_earlier_document_case_writes_a_xor_stage() {
    let mut cases = 0;
    let mut with_stage = 0;
    for case in documents().iter().flatten() {
        cases += 1;
        let uses = xors(&case.source) > 0;
        assert_eq!(uses, is_appended(&case.title), "{}", case.title);
        with_stage += usize::from(uses);
    }
    assert_eq!(with_stage, 7);
    assert_eq!(cases, 1333);
}

#[test]
fn the_audit_counts_xor_stages_and_nothing_else() {
    assert_eq!(xors("x = why c |> xor(why b)"), 1);
    assert_eq!(xors("x = c|>why|>xor(b|>why)"), 1);
    assert_eq!(xors("x = why c |> xor(why b |> xor(a)) |> xor(why a)"), 3);
    assert_eq!(xors("$P.k = $P.v |> affects |> xor($P.w |> affects)"), 1);
    assert_eq!(xors("x = [why c |> xor (why b)]"), 1);
    for source in [
        "xor = 1",
        "x = xor(c, b)",
        "x = {xor: 1}",
        "x = o.xor",
        "x = \"|> xor(b)\"",
        "x = why c |> union(why b)",
        "x = why c |> except(why b) |> union(why b |> except(why c))",
        "def xor(a, b) => a",
    ] {
        assert_eq!(xors(source), 0, "{source}");
    }
}

#[test]
fn the_appended_cases_stand_together_right_before_the_negated_membership_cases() {
    let [spec, reference, readme, examples] = documents();
    for (cases, own) in [
        (&spec, &SPEC_TITLES[..]),
        (&reference, &[REFERENCE_TITLE][..]),
        (&readme, &[README_TITLE][..]),
        (&examples, &[EXAMPLES_TITLE][..]),
    ] {
        let first = cases
            .iter()
            .position(|case| case.title == own[0])
            .unwrap_or_else(|| panic!("{} is appended", own[0]));
        for (offset, title) in own.iter().enumerate() {
            assert_eq!(
                cases.get(first + offset).map(|case| case.title.as_str()),
                Some(*title),
                "{title} stands with the other appended cases"
            );
        }
        let following = cases
            .iter()
            .position(|case| writes_bang_in(&case.source))
            .expect("the negated membership cases follow");
        assert_eq!(first + own.len(), following, "{}", own[0]);
        assert!(
            cases[..following]
                .iter()
                .all(|case| !writes_bang_in(&case.source))
        );
    }
}

#[test]
fn every_case_is_its_frozen_counterpart_with_only_the_stages_written_out() {
    let documents = documents();
    let mut before = 0;
    let mut after = 0;
    for Counterpart {
        title,
        rewrites,
        before: spelled_tokens,
        after: stage_tokens,
    } in SPELLED
    {
        let case = documents
            .iter()
            .flatten()
            .find(|case| case.title == title)
            .unwrap();
        let mut counterpart = case.source.clone();
        for (written, spelling) in rewrites {
            assert_eq!(
                counterpart.matches(written).count(),
                1,
                "{title}: {written}"
            );
            counterpart = counterpart.replacen(written, spelling, 1);
        }
        assert!(xors(&case.source) > 0, "{title}");
        assert_eq!(xors(&counterpart), 0, "{title}");
        assert_eq!(tokens(&case.source), stage_tokens, "{title}");
        assert_eq!(tokens(&counterpart), spelled_tokens, "{title}");
        assert!(stage_tokens < spelled_tokens, "{title}");
        let assertions = common::assertions::protected_assertions(&parse(&case.source).unwrap());
        assert_eq!(
            assertions,
            common::assertions::protected_assertions(&parse(&counterpart).unwrap()),
            "{title}"
        );
        for source in [&case.source, &counterpart] {
            common::check_with_assertions(
                &common::Case {
                    title: title.to_string(),
                    source: source.clone(),
                    expected: case.expected.clone(),
                },
                assertions.len(),
            )
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        }
        before += spelled_tokens;
        after += stage_tokens;
    }
    assert_eq!((before, after), (1_244, 1_134));
}

#[test]
fn the_xor_stage_parses_into_its_own_variant() {
    let tree = xor(
        reason(Stage::Why, identifier("c")),
        reason(Stage::Why, identifier("b")),
    );
    assert_eq!(value_of("x = c |> why |> xor(b |> why)"), tree);
    assert_eq!(value_of("x = why c |> xor(why b)"), tree);
    assert_eq!(value_of("x = why c |> xor (why b)"), tree);
    assert_eq!(value_of("x = c|>why|>xor(b|>why)"), tree);
    assert_eq!(value_of("x = (why c |> xor(\n    why b\n))"), tree);
    assert_eq!(parse_expression("why c |> xor(why b)").unwrap(), tree);
    assert_eq!(
        value_of("x = why c |> xor(b)"),
        xor(reason(Stage::Why, identifier("c")), identifier("b"))
    );
    for other in ["model", "(model)"] {
        assert_eq!(
            value_of(&format!("x = why c |> xor({other})")),
            xor(reason(Stage::Why, identifier("c")), Box::new(Expr::Model)),
            "{other}"
        );
    }
    assert_eq!(
        value_of("x = model |> xor(why c)"),
        xor(Box::new(Expr::Model), reason(Stage::Why, identifier("c")))
    );
    assert_eq!(
        value_of("x = why c |> xor(model.p)"),
        xor(
            reason(Stage::Why, identifier("c")),
            Box::new(Expr::Member {
                object: identifier("model"),
                property: "p".to_string(),
            })
        )
    );
    assert_eq!(
        value_of("x = why c |> xor(why b) |> stated"),
        *reason(Stage::Stated, Box::new(tree.clone()))
    );
    assert_eq!(
        value_of("x = why c |> xor(why b) |> union(a)"),
        Expr::Union {
            source: Box::new(tree.clone()),
            other: identifier("a"),
        }
    );
    assert_eq!(
        value_of("x = why c |> union(a) |> xor(why b)"),
        xor(
            Box::new(Expr::Union {
                source: reason(Stage::Why, identifier("c")),
                other: identifier("a"),
            }),
            reason(Stage::Why, identifier("b"))
        )
    );
    assert_eq!(
        value_of("x = xor(c, b)"),
        Expr::Call {
            callee: identifier("xor"),
            arguments: vec![
                Expr::Identifier("c".to_string()),
                Expr::Identifier("b".to_string()),
            ],
        }
    );
    assert_eq!(parse("xor c b").unwrap().len(), 3);
    assert_eq!(Stage::from_name("xor"), None);
    assert_eq!(unions("x = why c |> xor(why b)"), 0);
    assert_eq!(combines("x = why c |> xor(why b)"), 0);
    assert_eq!(combines("x = why c |> xor(why b |> intersect(a))"), 1);
    assert_eq!(unions("x = why c |> xor(why b |> union(a))"), 1);
    for source in [
        "p is [why c |> xor(why g([w])) for s in [1]]",
        "p is [x = why c |> xor(g([w])) for s in [1]]",
    ] {
        let tree = syntax(source);
        assert!(tree.contains("Xor {"), "{tree}");
        assert!(!tree.contains("Word"), "{tree}");
    }
    let marked = syntax("p is [w, why c |> xor(w)]");
    assert_eq!(marked.matches("Word").count(), 1, "{marked}");
}

#[test]
fn ordinary_names_called_xor_keep_their_meaning() {
    assert_eq!(result("xor = 1\n[xor, xor + 1]"), json!([1.0, 2.0]));
    assert_eq!(result("def xor(x, y) => x + y\nxor(4, 1)"), json!(5.0));
    assert_eq!(result("x = {xor: 1}\nx.xor"), json!(1.0));
    assert_eq!(result("o = {}\no.xor = 3\no.xor"), json!(3.0));
    assert_eq!(result("[1, 2].map(xor -> xor * 2)"), json!([2.0, 4.0]));
    assert_eq!(
        result("o = {f: x -> x + 1}\no.xor = o.f\no.xor(2)"),
        json!(3.0)
    );
    assert_eq!(
        result("enum Gate: xor, nand\n[Gate.xor, Gate.nand]"),
        json!(["xor", "nand"])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}enum Gate: xor, nand\n[Gate.xor, (why c |> xor(why b)).map(s -> s.node)]"
        )),
        json!(["xor", ["c"]])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}class Sets:\n    def xor(x) => x + 1\ns = Sets()\n[(why b |> xor(why c)).length, s.xor(1)]"
        )),
        json!([1.0, 2.0])
    );
    assert_eq!(
        failure(&format!("{CHAIN}xor")),
        error(ErrorKind::Reference, "xor is not defined", 4, 1)
    );
    assert_eq!(
        failure(&format!("{CHAIN}xor(1, 2)")),
        error(ErrorKind::Reference, "xor is not defined", 4, 1)
    );
    assert_eq!(
        failure(&format!("{CHAIN}xor a b")),
        error(ErrorKind::Reference, "xor is not defined", 4, 1)
    );
    assert_eq!(
        failure(&format!("{CHAIN}true xor false")),
        error(ErrorKind::Reference, "xor is not defined", 4, 6)
    );
    assert_eq!(
        failure(&format!("{CHAIN}xs = [1]\nxs.xor([2])")),
        error(ErrorKind::Type, "xs.xor is not a function", 5, 1)
    );
    assert_eq!(
        result("x = \"why c |> xor(why b)\"\nx"),
        json!("why c |> xor(why b)")
    );
}

#[test]
fn forms_without_parentheses_keep_their_parse_time_error() {
    for (stage, name) in [
        ("why c |> xor", "xor"),
        ("why c |> xor\n(why b)", "xor"),
        ("why c |> xor[b]", "xor"),
        ("why c |> xor.x", "xor"),
        ("why c |> xor{1}", "xor"),
        ("why c |> Xor(why b)", "Xor"),
        ("why c |> XOR(why b)", "XOR"),
        ("why c |> xors(why b)", "xors"),
        ("why c |> exclusive(why b)", "exclusive"),
        ("first = 1\nwhy c |> xor", "xor"),
        ("x = [why c |> xor]", "xor"),
        ("delete why c |> xor", "xor"),
    ] {
        assert_eq!(
            failure(&format!("{CHAIN}{stage}")),
            unpositioned(ErrorKind::Type, &format!("{name} is not an operation")),
            "{stage}"
        );
    }
    assert_eq!(
        failure(&format!("{CHAIN}why c |> |> xor(why b)")),
        error(ErrorKind::Syntax, "Expected an operation", 4, 10)
    );
    assert_eq!(
        failure(&format!("{CHAIN}1 ^ 2")),
        error(ErrorKind::Syntax, "Unexpected character '^'", 4, 3)
    );
}

#[test]
fn xor_selects_what_the_accepted_spelling_selects() {
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
                    let stage = evaluate(&mut runtime, &format!("{selection} |> xor({other})"));
                    let accepted = evaluate(&mut runtime, &spelled(&selection, &other, ""));
                    assert_eq!(stage, accepted, "seed {seed}: {selection} xor {other}");
                }
            }
        }
    }
}

#[test]
fn stages_after_xor_keep_what_they_keep_after_the_accepted_spelling() {
    for seed in 1..=3 {
        let mut runtime = Runtime::new();
        checked(&mut runtime, &graph(seed, 9));
        for x in 0..9 {
            for y in [0, 4, 8] {
                let selection = format!("n{x} |> why");
                let other = format!("n{y} |> affects");
                for later in [
                    "",
                    " |> stated",
                    " |> derived",
                    " |> depth(0)",
                    " |> depth(1)",
                    " |> depth(2)",
                    " |> path(n1)",
                    " |> path(n0)",
                    " |> intersect(n5 |> why)",
                    " |> except(n3 |> why)",
                    " |> union(n4 |> why)",
                    " |> xor(n2 |> why)",
                    " |> origins",
                    " |> outcomes",
                    " |> why",
                    " |> affects",
                    " |> nodes",
                    " |> holds",
                    " |> edges",
                    " |> depth(1) |> path(n0)",
                    " |> path(n0) |> depth(1)",
                ] {
                    let stage =
                        evaluate(&mut runtime, &format!("{selection} |> xor({other}){later}"));
                    let accepted = evaluate(&mut runtime, &spelled(&selection, &other, later));
                    assert_eq!(stage, accepted, "seed {seed}: n{x} xor n{y}{later}");
                }
                let changed = evaluate(
                    &mut runtime,
                    &format!(
                        "suppose n1 = 7:\n    [{selection} |> xor({other}) |> diff, {}]",
                        spelled(&selection, &other, " |> diff")
                    ),
                );
                assert_eq!(changed[0], changed[1], "seed {seed}: n{x} xor n{y} diff");
            }
        }
    }
}

#[test]
fn distances_and_direction_follow_the_walk_that_selected_each_step() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, SIX);
    for (selection, expected) in [
        ("why c |> xor(why f)", json!(["c", "b", "f", "e", "d"])),
        (
            "why c |> xor(why f) |> depth(1)",
            json!(["c", "b", "f", "e", "d"]),
        ),
        (
            "why f |> xor(why c) |> depth(1)",
            json!(["f", "e", "d", "c", "b"]),
        ),
        ("why c |> xor(why d)", json!(["c", "b", "d"])),
        ("why c |> xor(why d) |> depth(1)", json!(["c", "b", "d"])),
        (
            "why c |> depth(1) |> xor(why d |> depth(1))",
            json!(["c", "b", "d", "a"]),
        ),
        ("why c |> xor(why d) |> depth(0)", json!(["c", "d"])),
        ("why c |> xor(model) |> depth(0)", json!(["d", "e", "f"])),
        ("c |> xor(why f) |> depth(1)", json!(["c", "f", "e", "d"])),
        ("why c |> xor(affects a)", json!(["a", "d", "f"])),
        ("why c |> xor(affects a) |> path(d)", json!(["d", "f"])),
        ("affects a |> xor(why c)", json!(["d", "f", "a"])),
        ("affects a |> xor(why c) |> path(d)", json!(["d", "a"])),
        ("affects a |> xor(why c) |> depth(1)", json!(["d"])),
        (
            "affects a |> xor(why c) |> depth(2)",
            json!(["d", "f", "a"]),
        ),
        ("affects e |> xor(why c)", json!(["f", "c", "b", "a"])),
        ("affects f |> xor(why a) |> path(a)", json!(["a"])),
        ("why b |> xor(why d) |> affects", json!(["f", "c"])),
        ("b |> xor(d) |> affects", json!(["f", "c"])),
        (
            "affects b |> xor(affects d) |> why |> stated",
            json!(["e", "a"]),
        ),
    ] {
        assert_eq!(nodes(&mut runtime, selection), expected, "{selection}");
    }
    for (selection, path) in [
        ("why c |> xor(affects a)", json!(["d", "f"])),
        ("affects a |> xor(why c)", json!(["d", "a"])),
    ] {
        assert_eq!(
            nodes(&mut runtime, &format!("{selection} |> depth(9) |> path(d)")),
            path,
            "{selection} ranked"
        );
        assert_eq!(
            nodes(&mut runtime, &format!("{selection} |> path(d) |> depth(9)")),
            path,
            "{selection} ranked after path"
        );
    }
    for (first, second) in [
        ("why c", "affects a"),
        ("why f", "affects a"),
        ("why c", "affects e"),
        ("affects a", "why f"),
    ] {
        for target in ["a", "b", "d"] {
            for later in [
                format!(" |> path({target})"),
                format!(" |> depth(1) |> path({target})"),
            ] {
                assert_eq!(
                    nodes(&mut runtime, &format!("{first} |> xor({second}){later}")),
                    nodes(&mut runtime, &spelled(first, second, &later)),
                    "{first} xor {second}{later}"
                );
            }
        }
    }
}

#[test]
fn empty_disjoint_equal_and_nested_selections_combine_in_order() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, SIX);
    for (selection, expected) in [
        ("why e |> xor(why c)", json!(["e", "c", "b", "a"])),
        ("why c |> xor(why e)", json!(["c", "b", "a", "e"])),
        ("why c |> derived |> stated |> xor(why e)", json!(["e"])),
        ("why c |> xor(why e |> derived)", json!(["c", "b", "a"])),
        ("why f |> xor(why c)", json!(["f", "e", "d", "c", "b"])),
        ("why c |> xor(c)", json!(["b", "a"])),
        ("c |> xor(why c)", json!(["b", "a"])),
        ("why c |> xor(why c)", json!([])),
        ("why e |> derived |> xor(why a |> derived)", json!([])),
        ("affects f |> xor(affects c)", json!([])),
        ("affects a |> xor(affects a)", json!([])),
    ] {
        assert_eq!(nodes(&mut runtime, selection), expected, "{selection}");
    }
    checked(
        &mut runtime,
        "held = why e |> derived |> xor(why a |> derived)",
    );
    assert_eq!(evaluate(&mut runtime, "held"), json!([]));
    checked(&mut runtime, "e = a + 1");
    assert_eq!(nodes(&mut runtime, "held"), json!(["e"]));
    checked(&mut runtime, "e = 7");
    assert_eq!(evaluate(&mut runtime, "held"), json!([]));

    let mut follows = Runtime::new();
    checked(
        &mut follows,
        &format!("{SIX}held = why e |> derived |> xor(why c)\n"),
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
    checked(&mut runtime, ORDER);
    for source in [
        "why tax",
        "affects quantity",
        "tax |> why |> stated",
        "model |> derived",
        "shipping",
    ] {
        assert_eq!(
            evaluate(&mut runtime, &format!("{source} |> xor({source})")),
            json!([]),
            "{source}"
        );
        let outside = evaluate(&mut runtime, &format!("model |> except({source})"));
        assert_eq!(
            evaluate(&mut runtime, &format!("{source} |> xor(model)")),
            outside,
            "{source}"
        );
        assert_eq!(
            evaluate(&mut runtime, &format!("model |> xor({source})")),
            outside,
            "{source}"
        );
        for other in ["why shipping", "affects price", "rate", "why freight"] {
            let once = evaluate(&mut runtime, &format!("{source} |> xor({other})"));
            assert_eq!(
                evaluate(
                    &mut runtime,
                    &format!("{source} |> xor({other}) |> xor({other})")
                ),
                evaluate(
                    &mut runtime,
                    &format!(
                        "{source} |> except({other}) |> union({other} |> intersect({source}))"
                    )
                ),
                "{source} {other}"
            );
            assert_eq!(
                sorted(&nodes(
                    &mut runtime,
                    &format!("{source} |> xor({other}) |> xor({other})")
                )),
                sorted(&nodes(
                    &mut runtime,
                    &format!("{source} |> intersect(model)")
                )),
                "{source} {other}"
            );
            assert_eq!(
                evaluate(
                    &mut runtime,
                    &format!("{source} |> xor({other}) |> except({other})")
                ),
                evaluate(&mut runtime, &format!("{source} |> except({other})")),
                "{source} {other}"
            );
            assert_eq!(
                evaluate(
                    &mut runtime,
                    &format!("{source} |> xor({other}) |> intersect({source})")
                ),
                evaluate(&mut runtime, &format!("{source} |> except({other})")),
                "{source} {other}"
            );
            assert_eq!(
                evaluate(
                    &mut runtime,
                    &format!("{source} |> xor({other}) |> except({source})")
                ),
                evaluate(&mut runtime, &format!("{other} |> except({source})")),
                "{source} {other}"
            );
            assert_eq!(
                evaluate(
                    &mut runtime,
                    &format!(
                        "{source} |> union({other}) |> except({source} |> intersect({other}))"
                    )
                ),
                once,
                "{source} {other}"
            );
            assert_eq!(
                sorted(&nodes(&mut runtime, &format!("{other} |> xor({source})"))),
                sorted(&nodes(&mut runtime, &format!("{source} |> xor({other})"))),
                "{source} {other}"
            );
            assert_eq!(
                sorted(&nodes(
                    &mut runtime,
                    &format!("{source} |> xor({other}) |> xor(why total)")
                )),
                sorted(&nodes(
                    &mut runtime,
                    &format!("{source} |> xor({other} |> xor(why total))")
                )),
                "{source} {other}"
            );
        }
    }
    assert_eq!(
        nodes(&mut runtime, "why total |> xor(why freight)"),
        json!(["total", "subtotal", "tax", "price", "freight", "shipping"])
    );
    assert_eq!(
        nodes(&mut runtime, "why freight |> xor(why total)"),
        json!(["freight", "shipping", "total", "subtotal", "tax", "price"])
    );
    assert_eq!(
        nodes(&mut runtime, "why total |> xor(why tax)"),
        json!(["total"])
    );
    assert_eq!(
        nodes(&mut runtime, "why tax |> xor(why total)"),
        json!(["total"])
    );
    assert_eq!(
        nodes(&mut runtime, "why total |> stated |> xor(why freight)"),
        json!(["price", "freight", "shipping"])
    );
    assert_eq!(
        nodes(&mut runtime, "why total |> xor(why freight) |> stated"),
        json!(["price"])
    );
    assert_eq!(
        nodes(&mut runtime, "why total |> xor(why freight) |> stated"),
        nodes(
            &mut runtime,
            "why total |> stated |> xor(why freight |> stated)"
        )
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why total |> xor(why freight) |> intersect(why subtotal)"
        ),
        nodes(
            &mut runtime,
            "why total |> intersect(why subtotal) |> xor(why freight |> intersect(why subtotal))"
        )
    );
    let mut order = Runtime::new();
    checked(
        &mut order,
        "x = 1\np = x + 1\nq = p + x\ny = 2\nr = q + y\n",
    );
    assert_eq!(nodes(&mut order, "why r"), json!(["r", "q", "y", "p", "x"]));
    assert_eq!(
        nodes(&mut order, "why r |> xor(why q) |> xor(why q)"),
        json!(["r", "y", "q", "p", "x"])
    );
    let mut regrouped = Runtime::new();
    checked(
        &mut regrouped,
        "x = 1\na1 = x + 1\nb1 = x + 2\nc1 = x + 3\n",
    );
    assert_eq!(
        nodes(&mut regrouped, "why a1 |> xor(why b1) |> xor(why c1)"),
        json!(["a1", "b1", "c1", "x"])
    );
    assert_eq!(
        nodes(&mut regrouped, "why a1 |> xor(why b1 |> xor(why c1))"),
        json!(["a1", "x", "b1", "c1"])
    );
}

#[test]
fn every_kind_of_other_selection_selects_as_a_pipeline_source() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}order = {{fee: 2}}\norder.cost = order.fee + quantity\nclass Item(price, qty):\n    this.price = price\n    this.qty = qty\n    def apart() => (this.total |> why |> xor(this.qty)).map(s -> s.node)\n$Item.total = $Item.price * $Item.qty\n$Item.apart = (why $Item.price |> xor($Item.total)).map(s -> s.node)\npen = Item(2, 5)\n"
        ),
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> xor(price)"),
        json!(["shipping", "quantity", "price"])
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> xor(quantity)"),
        json!(["shipping"])
    );
    let cost = evaluate(&mut runtime, "(order.cost |> why)[0].node");
    assert_eq!(
        nodes(&mut runtime, "why shipping |> xor(order.cost)"),
        json!(["shipping", "quantity", cost.clone()])
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> xor(why order.cost)"),
        json!(["shipping", cost, "order"])
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> xor(model |> stated)"),
        nodes(
            &mut runtime,
            &spelled("why shipping", "model |> stated", "")
        )
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why shipping |> xor(subtotal |> why |> derived)"
        ),
        json!(["shipping", "quantity", "subtotal"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why shipping |> xor(why tax |> intersect(why subtotal))"
        ),
        json!(["shipping", "subtotal", "price"])
    );
    assert_eq!(
        nodes(
            &mut runtime,
            "why shipping |> xor(why tax |> xor(why order.fee))"
        ),
        nodes(
            &mut runtime,
            &spelled("why shipping", "why tax |> xor(why order.fee)", "")
        )
    );
    assert_eq!(
        evaluate(&mut runtime, "[pen.apart(), pen.apart]"),
        json!([["pen.total", "pen.price"], ["pen.price", "pen.total"]])
    );
    assert_eq!(
        nodes(&mut runtime, "why shipping |> xor(order.missing)"),
        json!(["shipping", "quantity"])
    );
    let mut filled = Runtime::new();
    checked(
        &mut filled,
        &format!("{CHAIN}o = {{}}\nheld = why b |> xor(o.p)\n"),
    );
    assert_eq!(nodes(&mut filled, "held"), json!(["b", "a"]));
    checked(&mut filled, "o.p = 1");
    let property = evaluate(&mut filled, "(o.p |> why)[0].node");
    assert_eq!(nodes(&mut filled, "held"), json!(["b", "a", property]));
    checked(&mut filled, "delete o.p");
    assert_eq!(nodes(&mut filled, "held"), json!(["b", "a"]));
}

#[test]
fn names_that_hold_a_reasoning_expression_select_nothing() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}causes = why freight\nnames = why freight |> nodes\nlisted = (why freight).map(s -> s.node)\nxs = [1, 2]\n"
        ),
    );
    let total = nodes(&mut runtime, "why total");
    for name in ["causes", "names", "listed"] {
        assert_eq!(
            nodes(&mut runtime, &format!("why total |> xor({name})")),
            total,
            "{name}"
        );
    }
    assert_eq!(
        nodes(&mut runtime, "why total |> xor(xs)"),
        json!([
            "total", "subtotal", "tax", "rate", "price", "quantity", "xs"
        ])
    );
    assert_eq!(
        nodes(&mut runtime, "causes |> xor(why tax)"),
        nodes(&mut runtime, "why tax")
    );
    assert_eq!(evaluate(&mut runtime, "causes.length"), json!(4.0));
}

#[test]
fn invalid_other_selections_keep_the_diagnostics_of_a_pipeline_source() {
    for (other, expected) in [
        ("1", error(ErrorKind::Type, "Cannot explain 1", 9, 1)),
        (
            "\"price\"",
            error(ErrorKind::Type, "Cannot explain \"price\"", 9, 1),
        ),
        (
            "price + 1",
            error(ErrorKind::Type, "Cannot explain price+1", 9, 1),
        ),
        (
            "price.x",
            error(ErrorKind::Type, "Cannot explain price.x", 9, 1),
        ),
        ("this", error(ErrorKind::Type, "Cannot explain this", 9, 1)),
        (
            "[price]",
            error(ErrorKind::Type, "Cannot explain [price]", 9, 1),
        ),
        (
            "(why price).filter(s -> true)",
            error(
                ErrorKind::Type,
                "Cannot explain price|>why.filter(function())",
                9,
                1,
            ),
        ),
        (
            "missing",
            error(ErrorKind::Reference, "missing is not defined", 9, 1),
        ),
        (
            "ghost.x",
            error(ErrorKind::Reference, "ghost is not defined", 9, 1),
        ),
        (
            "why missing",
            error(ErrorKind::Reference, "missing is not defined", 9, 1),
        ),
        ("1 |> why", error(ErrorKind::Type, "Cannot explain 1", 9, 1)),
        (
            "why price |> depth(-1)",
            error(ErrorKind::Type, "-1 is not a non-negative integer", 9, 1),
        ),
        (
            "why price |> diff",
            error(ErrorKind::Type, "Cannot use 'diff' outside suppose", 9, 1),
        ),
        (
            "freight |> why |> nodes",
            error(ErrorKind::Type, "Cannot explain freight|>why|>nodes", 9, 1),
        ),
        (
            "freight |> why |> holds",
            error(ErrorKind::Type, "Cannot explain freight|>why|>holds", 9, 1),
        ),
        (
            "freight |> why |> edges",
            error(ErrorKind::Type, "Cannot explain freight|>why|>edges", 9, 1),
        ),
    ] {
        assert_eq!(
            failure(&format!("{ORDER}why total |> xor({other})")),
            expected,
            "{other}"
        );
        assert_eq!(
            failure(&format!("{ORDER}why total |> xor({other}) |> depth(1)")),
            expected,
            "{other} ranked"
        );
        assert_eq!(
            failure(&format!(
                "{ORDER}why total |> except({other}) |> union({other} |> except(why total))"
            )),
            expected,
            "{other} spelled"
        );
    }
    assert_eq!(
        failure(&format!(
            "{ORDER}class Item(price):\n    this.price = price\nwhy total |> xor($Item.price)"
        )),
        error(ErrorKind::Type, "Cannot explain $Item.price", 11, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}o = {{p: null}}\nwhy total |> xor(o.p.q)")),
        error(ErrorKind::Reference, "o.p is not defined", 10, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}1 |> xor(missing)")),
        error(ErrorKind::Type, "Cannot explain 1", 9, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}missing |> xor(1)")),
        error(ErrorKind::Reference, "missing is not defined", 9, 1)
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{ORDER}held = why total |> xor(why freight)\n"),
    );
    assert_eq!(
        failure_in(&mut runtime, "held = why total |> xor(missing)"),
        error(ErrorKind::Reference, "missing is not defined", 1, 1)
    );
    assert_eq!(
        nodes(&mut runtime, "held"),
        json!(["total", "subtotal", "tax", "price", "freight", "shipping"])
    );
}

#[test]
fn zero_or_two_arguments_report_the_errors_of_parentheses() {
    assert_eq!(
        failure(&format!("{ORDER}why total |> xor()\n")),
        error(ErrorKind::Syntax, "Unexpected ')'", 9, 18)
    );
    assert_eq!(
        failure(&format!(
            "{ORDER}why total |> xor(why freight, why price)\n"
        )),
        error(ErrorKind::Syntax, "Expected ')' but found ','", 9, 29)
    );
    assert_eq!(
        failure(&format!("{ORDER}why total |> xor(why freight,)\n")),
        error(ErrorKind::Syntax, "Expected ')' but found ','", 9, 29)
    );
    assert_eq!(
        failure(&format!("{ORDER}why total |> xor(why freight\n")),
        error(
            ErrorKind::Syntax,
            "Expected ')' but found end of input",
            10,
            1
        )
    );
    assert_eq!(
        failure(&format!("{ORDER}why total |> xor(\n")),
        error(ErrorKind::Syntax, "Unexpected end of input", 10, 1)
    );
    let mut deep = String::from("why c");
    for _ in 0..70 {
        deep = format!("why c |> xor({deep})");
    }
    let (kind, message, _) = failure(&format!("{CHAIN}x = {deep}"));
    assert_eq!(
        (kind, message.as_str()),
        (ErrorKind::Syntax, "Expressions are nested too deeply")
    );
    let mut nested = String::from("why c");
    for _ in 0..30 {
        nested = format!("why c |> xor({nested})");
    }
    assert_eq!(result(&format!("{CHAIN}({nested}).length")), json!(3.0));
    assert_eq!(
        result(&format!("{CHAIN}(why c |> xor({nested})).length")),
        json!(0.0)
    );
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
        let deeper = format!("why c |> xor({source})");
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
        let expected = |levels: usize| {
            if levels % 2 == 0 {
                steps.clone()
            } else {
                json!([])
            }
        };
        assert_eq!(resolved(&mut runtime, &build(1)), Ok(json!([])));
        assert_eq!(resolved(&mut runtime, &build(parsed)), Ok(expected(parsed)));
        let mut limit = None;
        for levels in parsed + 1..=1_000 {
            match resolved(&mut runtime, &build(levels)) {
                Ok(selection) => assert_eq!(selection, expected(levels), "{levels}"),
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
        assert_eq!(resolved(&mut runtime, &build(limit)), Ok(expected(limit)));
        checked(&mut runtime, "d = c + 1");
        assert_eq!(
            nodes(&mut runtime, "why b |> xor(why d)"),
            json!(["d", "c"])
        );
    }
}

#[test]
fn every_binding_named_xor_shadows_the_stage() {
    let bindings = [
        ("xor = 1\nwhy c |> xor(why b)", 5),
        ("def xor(x) => x\nwhy c |> xor(why b)", 5),
        ("class xor\nwhy c |> xor(why b)", 5),
        ("enum xor: A, B\nwhy c |> xor(why b)", 5),
        ("xor = 1\nwhy c |> xor(missing)", 5),
        ("xor = 1\nmissing |> xor(why b)", 5),
        ("xor = 1\n1 |> xor(why b)", 5),
        ("xor = 1\nwhy c |> xor(why b) |> depth(1)", 5),
        ("def f(xor) => why c |> xor(why b)\nf(1)", 5),
        ("def g xor => why c |> xor(why b)\ng(1)", 5),
        ("f = xor -> why c |> xor(why b)\nf(1)", 5),
        ("f = (xor) => why c |> xor(why b)\nf(1)", 5),
        ("f = lambda xor: why c |> xor(why b)\nf(1)", 5),
        ("[1].map(xor -> why c |> xor(why b))", 4),
        ("for xor of \"x\":\n    y = why c |> xor(why b)", 4),
        ("ok = all xor in [1]: (why c |> xor(why b)).length > 0", 4),
        ("{\n    xor = 5\n    y = why c |> xor(why b)\n}", 4),
        (
            "class R:\n    def m(xor) => why c |> xor(why b)\nR().m(1)",
            6,
        ),
        (
            "class Q(xor):\n    this.v = (why c |> xor(why b)).length\nQ(1)",
            6,
        ),
        (
            "try:\n    throw 1\ncatch xor:\n    y = why c |> xor(why b)",
            4,
        ),
        ("{xor} = {xor: 1}\nwhy c |> xor(why b)", 5),
    ];
    for (binding, line) in bindings {
        assert_eq!(
            failure(&format!("{CHAIN}{binding}")),
            error(ErrorKind::Type, "xor is not an operation", line, 1),
            "{binding}"
        );
    }
    assert_eq!(
        result(&format!(
            "{CHAIN}xor = 1\ncaught = \"\"\ntry:\n    why c |> xor(why b)\ncatch error:\n    caught = error\n[xor, caught]"
        )),
        json!([1.0, "TypeError: xor is not an operation"])
    );
    let mut runtime = Runtime::new();
    checked(&mut runtime, CHAIN);
    assert_eq!(
        failure_in(&mut runtime, "x = 5\nxor = 1\ny = why c |> xor(why b)"),
        error(ErrorKind::Type, "xor is not an operation", 3, 1)
    );
    assert_eq!(
        failure(&format!("{CHAIN}stated = 1\nwhy c |> xor(why b |> stated)")),
        error(ErrorKind::Type, "stated is not an operation", 5, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\nmissing |> xor(why b |> stated)"
        )),
        error(ErrorKind::Reference, "missing is not defined", 5, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\nxor = 2\nwhy c |> xor(why b) |> stated"
        )),
        error(ErrorKind::Type, "stated is not an operation", 6, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}stated = 1\nxor = 2\nwhy c |> stated |> xor(why b)"
        )),
        error(ErrorKind::Type, "xor is not an operation", 6, 1)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}except = 1\nwhy c |> xor(why b |> except(a))"
        )),
        error(ErrorKind::Type, "except is not an operation", 5, 1)
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}except = 1\nunion = 2\nintersect = 3\nnodes = 4\n(why b |> xor(why c)).length"
        )),
        json!(1.0)
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}union = 1\nwhy b |> except(why c) |> union(why c |> except(why b))"
        )),
        error(ErrorKind::Type, "union is not an operation", 5, 1)
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}xor = 1\n[(why c |> except(why a)).length, (why c |> union(why b)).length, (why c |> stated).length]"
        )),
        json!([2.0, 3.0, 1.0])
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}why = 1\naffects = 2\n(why b |> xor(why c)).length"
        )),
        json!(1.0)
    );
}

#[test]
fn a_binding_declared_after_a_standing_xor_fails_its_refresh() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}held = why b |> xor(why c)\nexcept = 5\nunion = 6"),
    );
    assert_eq!(nodes(&mut runtime, "held"), json!(["c"]));
    checked(&mut runtime, "a = 2");
    assert_eq!(
        evaluate(&mut runtime, "held.map(s -> s.holds)"),
        json!([8.0])
    );
    checked(&mut runtime, "xor = 5");
    assert_eq!(
        failure_in(&mut runtime, "a = 3"),
        error(ErrorKind::Type, "xor is not an operation", 1, 1)
    );
    assert_eq!(evaluate(&mut runtime, "a"), json!(2.0));
    checked(&mut runtime, "delete xor\na = 3");
    assert_eq!(
        evaluate(&mut runtime, "held.map(s -> s.holds)"),
        json!([10.0])
    );
    let mut spelled_runtime = Runtime::new();
    checked(
        &mut spelled_runtime,
        &format!("{CHAIN}held = {}\n", spelled("why b", "why c", "")),
    );
    checked(&mut spelled_runtime, "union = 6");
    assert_eq!(
        failure_in(&mut spelled_runtime, "a = 3"),
        error(ErrorKind::Type, "union is not an operation", 1, 1)
    );
}

#[test]
fn an_undefined_side_makes_the_symmetric_difference_undefined() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}n = null\nheld = why c |> xor(why b |> depth(n))\n"),
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "[held, why c |> xor(why b |> depth(n)), why c |> depth(n) |> xor(missing), why c |> xor(why b |> depth(n)) |> xor(missing), why c |> xor(why b |> depth(n)) |> depth(missing), why c |> xor(why b |> depth(n)) |> nodes, why b |> depth(n) |> xor(why c)]"
        ),
        json!([null, null, null, null, null, null, null])
    );
    assert_eq!(evaluate(&mut runtime, "held.length"), Json::Null);
    assert_eq!(
        evaluate(
            &mut runtime,
            "all s in why c |> xor(why b |> depth(n)): s.holds > 0"
        ),
        Json::Null
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "[why c |> except(why b |> depth(n)) |> union(why b |> depth(n) |> except(why c)), why c |> depth(n) |> except(missing) |> union(missing |> except(why c |> depth(n)))]"
        ),
        json!([null, null])
    );
    checked(&mut runtime, "n = 0");
    assert_eq!(nodes(&mut runtime, "held"), json!(["c", "a"]));
    assert_eq!(
        failure_in(&mut runtime, "why c |> depth(n) |> xor(missing)"),
        error(ErrorKind::Reference, "missing is not defined", 1, 1)
    );
    checked(&mut runtime, "n = undefined");
    assert_eq!(evaluate(&mut runtime, "held"), Json::Null);

    let mut waiting = Runtime::new();
    checked(
        &mut waiting,
        &format!("{CHAIN}n = null\nskipped = why c |> depth(n) |> xor(missing)\n"),
    );
    assert_eq!(evaluate(&mut waiting, "skipped"), Json::Null);
    assert_eq!(
        failure_in(&mut waiting, "n = 1"),
        error(ErrorKind::Reference, "missing is not defined", 1, 1)
    );
    assert_eq!(evaluate(&mut waiting, "[n, skipped]"), json!([null, null]));
}

#[test]
fn standing_symmetric_differences_follow_both_sides_and_the_steps_they_leave_out() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}fee = 1\napart = why total |> xor(why freight)\nfacts = why total |> xor(why freight) |> stated\nfirst = why total |> stated |> xor(why freight |> stated)\n"
        ),
    );
    assert_eq!(
        nodes(&mut runtime, "apart"),
        json!(["total", "subtotal", "tax", "price", "freight", "shipping"])
    );
    assert_eq!(nodes(&mut runtime, "facts"), json!(["price"]));
    checked(&mut runtime, "freight = shipping + rate + fee");
    assert_eq!(
        nodes(&mut runtime, "apart"),
        json!([
            "total", "subtotal", "tax", "price", "freight", "shipping", "fee"
        ])
    );
    assert_eq!(nodes(&mut runtime, "facts"), json!(["price", "fee"]));
    assert_eq!(nodes(&mut runtime, "first"), nodes(&mut runtime, "facts"));
    checked(&mut runtime, "rate = fee / 5");
    assert_eq!(
        nodes(&mut runtime, "apart"),
        json!(["total", "subtotal", "tax", "price", "freight", "shipping"])
    );
    assert_eq!(nodes(&mut runtime, "facts"), json!(["price"]));
    checked(&mut runtime, "subtotal = 30");
    assert_eq!(
        nodes(&mut runtime, "apart"),
        json!([
            "total", "subtotal", "tax", "freight", "shipping", "quantity"
        ])
    );
    assert_eq!(
        nodes(&mut runtime, "facts"),
        json!(["subtotal", "quantity"])
    );
    assert_eq!(nodes(&mut runtime, "first"), nodes(&mut runtime, "facts"));

    let mut values = Runtime::new();
    checked(
        &mut values,
        &format!("{ORDER}apart = why total |> xor(why freight)\n"),
    );
    checked(&mut values, "price = 12\nquantity = 5");
    assert_eq!(
        evaluate(&mut values, "apart.map(s -> s.holds)"),
        json!([72.0, 60.0, 12.0, 12.0, 10.2, 10.0])
    );
    checked(&mut values, "delete shipping");
    assert_eq!(
        nodes(&mut values, "apart"),
        json!(["total", "subtotal", "tax", "price", "quantity", "freight"])
    );
    checked(&mut values, "shipping = price");
    assert_eq!(
        nodes(&mut values, "apart"),
        json!([
            "total", "subtotal", "tax", "quantity", "freight", "shipping"
        ])
    );
}

#[test]
fn standing_symmetric_differences_observe_both_walks_and_model_membership() {
    let output = cli(
        "observes",
        &format!(
            "{CHAIN}d = 4\nx = why b |> xor(model)\ny = why b |> xor(why d)\nz = why d |> xor(why c |> stated)\nw = {}\n1",
            spelled("why b", "why d", "")
        ),
    );
    assert!(output.status.success(), "{output:?}");
    let listing = String::from_utf8(output.stdout).unwrap();
    for entry in [
        "  x [variable]\n    reads    b, a, c, d\n    observes b, a, model, c, d\n",
        "  y [variable]\n    reads    b, a, d\n    observes b, a, d\n",
        "  z [variable]\n    reads    d, c, b, a\n    observes d, c, b, a\n",
        "  w [variable]\n    reads    b, a, d\n    observes b, a, d\n",
    ] {
        assert!(listing.contains(entry), "{entry}\n{listing}");
    }
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{CHAIN}held = why a |> xor(model |> derived)\nempty = a |> stated |> xor(b |> stated)\n"
        ),
    );
    assert_eq!(nodes(&mut runtime, "held"), json!(["a", "b", "c"]));
    assert_eq!(nodes(&mut runtime, "empty"), json!(["a"]));
    checked(&mut runtime, "b = 7");
    assert_eq!(nodes(&mut runtime, "held"), json!(["a", "c"]));
    assert_eq!(nodes(&mut runtime, "empty"), json!(["a", "b"]));
    checked(&mut runtime, "a = b + 1");
    assert_eq!(nodes(&mut runtime, "held"), json!(["b", "c"]));
    assert_eq!(nodes(&mut runtime, "empty"), json!(["b"]));
    checked(&mut runtime, "e = c + 1");
    assert_eq!(nodes(&mut runtime, "held"), json!(["b", "c", "e"]));
}

#[test]
fn class_rules_functions_methods_lambdas_quantifiers_and_conditions_hold_the_stage() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Order(qty, price, fee):\n    this.qty = qty\n    this.price = price\n    this.fee = fee\n    def own() => (this.cost |> why |> xor(this.total |> why)).map(s -> s.node)\n$Order.total = $Order.qty * $Order.price\n$Order.cost = $Order.qty + $Order.fee\n$Order.apart = why $Order.total |> xor(why $Order.cost)\n$Order.big = any s in why $Order.total |> xor(why $Order.cost): s.holds > 9\nif (why $Order.total |> xor(why $Order.cost)).length > 3:\n    $Order.linked = true\npen = Order(2, 5, 1)\ncup = Order(4, 3, 2)\ndef counted() => (why pen.total |> xor(why pen.cost)).length\npick = [1].map(v -> (why pen.total |> xor(why cup.total)).length)\n",
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            "[pen.apart.map(s -> s.node), cup.apart.map(s -> s.node), pen.big, cup.big, pen.linked, pen.own()]"
        ),
        json!([
            ["pen.total", "pen.price", "pen.cost", "pen.fee"],
            ["cup.total", "cup.price", "cup.cost", "cup.fee"],
            true,
            true,
            true,
            ["pen.cost", "pen.fee", "pen.total", "pen.price"]
        ])
    );
    assert_eq!(
        evaluate(&mut runtime, "[counted(), pick]"),
        json!([4.0, [6.0]])
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
            "[pen.apart.map(s -> s.node), cup.apart.map(s -> s.node)]"
        ),
        json!([
            ["pen.total", "pen.qty", "pen.price", "pen.cost", "pen.fee"],
            ["cup.total", "cup.qty", "cup.price", "cup.cost", "cup.fee"]
        ])
    );
    checked(&mut runtime, "jar = Order(1, 2, 3)");
    assert_eq!(
        nodes(&mut runtime, "jar.apart"),
        json!(["jar.total", "jar.qty", "jar.price", "jar.cost", "jar.fee"])
    );
    let text = result(&format!("{CHAIN}`${{(why b |> xor(why c)).length}} apart`"));
    assert_eq!(text, json!("1 apart"));
}

#[test]
fn class_rules_check_the_names_and_cycles_in_other() {
    let class = "class Order(x):\n    this.x = x\n";
    assert_eq!(
        failure(&format!(
            "{class}$Order.k = why $Order.x |> xor(why missing)\n1"
        )),
        error(ErrorKind::Reference, "missing is not defined", 3, 1)
    );
    assert_eq!(
        failure(&format!(
            "{class}$Order.y = (why $Order.x |> xor(why $Order.y)).length\no = Order(1)\no.y"
        )),
        error(ErrorKind::Type, "Circular Dependency", 3, 1)
    );
    assert_eq!(
        failure(&format!(
            "{class}$Order.a = (why $Order.x |> xor(why $Order.b)).length\n$Order.b = (why $Order.x |> xor(why $Order.a)).length\no = Order(1)\n[o.a, o.b]"
        )),
        error(ErrorKind::Type, "Circular Dependency", 4, 1)
    );
    assert_eq!(
        result(&format!(
            "{class}$Order.k = (why $Order.x |> xor(why $Order.z)).map(s -> s.node)\no = Order(1)\no.z = o.x + 1\n[o.k, (why o.x |> except(why o.z) |> union(why o.z |> except(why o.x))).map(s -> s.node)]"
        )),
        json!([["o.z", "o"], ["o.z", "o"]])
    );
}

#[test]
fn a_declaration_holding_the_stage_stays_out_of_its_own_model() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = 1\nb = a + 1\nall_of = model |> xor(why b)\nmore = why b |> xor(model)\n",
    );
    assert_eq!(evaluate(&mut runtime, "all_of"), json!([]));
    assert_eq!(evaluate(&mut runtime, "more"), json!([]));
    assert_eq!(nodes(&mut runtime, "model |> why"), json!(["a", "b"]));
    assert_eq!(evaluate(&mut runtime, "(why all_of).length"), json!(0.0));
    assert_eq!(nodes(&mut runtime, "affects a"), json!(["b"]));
    checked(&mut runtime, "c = b + 1");
    assert_eq!(nodes(&mut runtime, "all_of"), json!(["c"]));
    assert_eq!(nodes(&mut runtime, "more"), json!(["c"]));
    checked(
        &mut runtime,
        "held = 1 if true else (why c |> xor(why b))\nwalked = 1 if true else (why c)\nplain = 1 if true else c\n",
    );
    assert_eq!(
        nodes(&mut runtime, "model |> stated"),
        json!(["a", "plain"])
    );
    assert_eq!(nodes(&mut runtime, "all_of"), json!(["c", "plain"]));
    assert_eq!(
        evaluate(
            &mut runtime,
            "[held, walked, plain, (why held).length, (why walked).length]"
        ),
        json!([1.0, 1.0, 1.0, 0.0, 0.0])
    );
}

#[test]
fn declarations_that_call_a_function_holding_the_stage_stay_live() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}def counted() => (why total |> xor(why freight)).length\nheld = counted()\n"
        ),
    );
    assert_eq!(evaluate(&mut runtime, "held"), json!(6.0));
    checked(&mut runtime, "fee = 1\nfreight = shipping + fee");
    assert_eq!(evaluate(&mut runtime, "held"), json!(8.0));
    checked(&mut runtime, "subtotal = quantity * 2");
    assert_eq!(evaluate(&mut runtime, "held"), json!(7.0));
    checked(&mut runtime, "freight = 1");
    assert_eq!(evaluate(&mut runtime, "held"), json!(6.0));
}

#[test]
fn failed_changes_suppose_blocks_and_diff_restore_symmetric_differences() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{ORDER}apart = why subtotal |> xor(why shipping)\n"),
    );
    let initial = json!(["subtotal", "price", "shipping"]);
    checked(
        &mut runtime,
        "try:\n    shipping = price / 2\n    assert(apart.map(s -> s.node), [\"subtotal\", \"quantity\", \"shipping\"])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nassert(apart.map(s -> s.node), [\"subtotal\", \"price\", \"shipping\"])",
    );
    assert_eq!(
        failure_in(
            &mut runtime,
            "shipping = price + quantity\nshipping = missing"
        ),
        error(ErrorKind::Reference, "missing is not defined", 2, 1)
    );
    assert_eq!(nodes(&mut runtime, "apart"), initial);
    assert_eq!(
        failure_in(&mut runtime, "apart = why subtotal |> xor(1)"),
        error(ErrorKind::Type, "Cannot explain 1", 1, 1)
    );
    assert_eq!(nodes(&mut runtime, "apart"), initial);
    let plan = evaluate(
        &mut runtime,
        "suppose shipping = rate + 1:\n    [apart.map(s -> s.node), (why tax |> diff |> xor(why shipping |> diff)).map(s -> s.node), (why tax |> xor(why shipping) |> diff).map(s -> s.node)]",
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
        "suppose price = 20:\n    [(why tax |> xor(why shipping) |> diff).map(s -> s.node), (why shipping |> xor(why tax) |> diff).map(s -> s.node)]",
    );
    assert_eq!(
        moved,
        json!([["tax", "subtotal", "price"], ["tax", "subtotal", "price"]])
    );
    assert_eq!(nodes(&mut runtime, "apart"), initial);
    assert_eq!(evaluate(&mut runtime, "price"), json!(10.0));
}

#[test]
fn snapshots_freeze_the_stage_and_the_limits_inside_other() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{ORDER}n = 0\nsnap = (why tax |> xor(why shipping)).value\nnear = why rate |> xor(why tax |> depth(n.value))\nlive = why rate |> xor(why tax |> depth(n))\nnested = why rate |> xor(why price |> xor(why tax |> depth(n.value)))\nshared = why rate |> intersect(why tax |> xor(why subtotal |> depth(n.value)))\njoined = why rate |> union(why tax |> xor(why subtotal |> depth(n.value)))\n"
        ),
    );
    assert_eq!(
        stored(&runtime, "near"),
        "rate|>why|>xor(tax|>why|>depth(0))"
    );
    assert_eq!(
        stored(&runtime, "nested"),
        "rate|>why|>xor(price|>why|>xor(tax|>why|>depth(0)))"
    );
    assert_eq!(
        stored(&runtime, "shared"),
        "rate|>why|>intersect(tax|>why|>xor(subtotal|>why|>depth(0)))"
    );
    assert_eq!(
        stored(&runtime, "joined"),
        "rate|>why|>union(tax|>why|>xor(subtotal|>why|>depth(0)))"
    );
    assert_eq!(
        stored(&runtime, "live"),
        "rate|>why|>xor(tax|>why|>depth(n))"
    );
    assert_eq!(nodes(&mut runtime, "near"), json!(["rate", "tax"]));
    assert_eq!(nodes(&mut runtime, "live"), json!(["rate", "tax"]));
    checked(&mut runtime, "n = 1\nshipping = price + 1");
    assert_eq!(
        nodes(&mut runtime, "snap"),
        json!(["tax", "subtotal", "rate", "price", "shipping"])
    );
    assert_eq!(nodes(&mut runtime, "near"), json!(["rate", "tax"]));
    assert_eq!(nodes(&mut runtime, "live"), json!(["tax", "subtotal"]));
    checked(&mut runtime, "quantity = 4");
    assert_eq!(
        evaluate(&mut runtime, "snap.map(s -> s.holds)"),
        json!([6.0, 30.0, 0.2, 10.0, 6.0])
    );
}

#[test]
fn steps_are_matched_by_node_even_when_they_hold_nan() {
    assert_eq!(
        result(
            "x = 0 / 0\ny = x + 1\nz = x * 2\n[(why y |> xor(why z)).map(s -> s.node), (why y).concat(why z).filter(s -> !(why y).includes(s) || !(why z).includes(s)).map(s -> s.node), (why y |> except(why z) |> union(why z |> except(why y))).map(s -> s.node)]"
        ),
        json!([["y", "z"], ["y", "x", "z", "x"], ["y", "z"]])
    );
}

#[test]
fn symmetric_differences_render_once_and_parse_back() {
    for source in [
        "why c |> xor(why b)",
        "c |> why |> xor(b |> why |> stated)",
        "model |> xor(model) |> stated",
        "a |> affects |> xor(b |> affects) |> why |> xor(a)",
        "why c |> xor(o.p)",
        "why c |> xor(why b |> depth(n + 1)) |> depth(1)",
        "why c |> path(a) |> xor(why b |> path(a))",
        "a || b |> xor(c)",
        "why c |> xor(why b |> xor(why a))",
        "why c |> xor(why b |> union(why a)) |> except(a)",
        "why c |> union(why b |> xor(why a)) |> intersect(a)",
        "2 ** (why c |> xor(why b))",
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
        parse_expression("why tax |> xor(why shipping) |> derived")
            .unwrap()
            .to_string(),
        "tax|>why|>xor(shipping|>why)|>derived"
    );
    assert_eq!(
        parse_expression("why tax |> xor(model)")
            .unwrap()
            .to_string(),
        "tax|>why|>xor(model)"
    );
    assert_eq!(
        parse_expression("2 ** (why c |> xor(why b))")
            .unwrap()
            .to_string(),
        "2**(c|>why|>xor(b|>why))"
    );
    let mut chained = String::from("c");
    for level in 1..=6 {
        chained.push_str(" |> xor(a)");
        let rendered = parse_expression(&chained).unwrap().to_string();
        assert_eq!(rendered.len(), 1 + 8 * level, "{rendered}");
    }
    assert_eq!(
        generate(&parse("def f(x = 1) => why c |> xor(why b)").unwrap()[0]),
        "def f(x=1):\n    return c|>why|>xor(b|>why)"
    );
    assert_eq!(
        failure(&format!("{ORDER}delete why total |> xor(why freight)\n")),
        error(
            ErrorKind::Syntax,
            "Cannot delete total|>why|>xor(freight|>why)",
            9,
            1
        )
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}held = why b |> xor(why c)\n"),
    );
    assert_eq!(stored(&runtime, "held"), "b|>why|>xor(c|>why)");
    assert_eq!(nodes(&mut runtime, "why c"), json!(["c", "b", "a"]));
    assert_eq!(nodes(&mut runtime, "affects a"), json!(["b", "c"]));
    assert_eq!(nodes(&mut runtime, "model |> why"), json!(["a", "b", "c"]));
    let output = cli(
        "keys",
        &format!(
            "{CHAIN}if (why b |> xor(why c)).length > 0:\n    flag = 1\nheld = why b |> xor(why c)\n1"
        ),
    );
    assert!(output.status.success(), "{output:?}");
    let listing = String::from_utf8(output.stdout).unwrap();
    assert!(
        listing.contains("  if(b|>why|>xor(c|>why).length>0) [if]\n"),
        "{listing}"
    );
    assert!(
        listing.contains("  held [variable]\n    reads    b, a, c\n    observes b, a, c\n"),
        "{listing}"
    );
}

#[test]
fn symmetric_differences_handle_large_graphs() {
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
            &format!("(why w{last} |> xor(why v{last})).length")
        ),
        json!(20_000.0)
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            &format!("(why v{last} |> xor(why w0)).length")
        ),
        json!(20_000.0)
    );
    assert_eq!(
        evaluate(&mut runtime, "(affects v0 |> xor(affects w0)).length"),
        json!(20_000.0)
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            &format!("(why v{last} |> xor(why w{last}) |> depth(1)).length")
        ),
        json!(2.0)
    );
}

#[test]
fn near_misses_keep_their_accepted_results() {
    assert_eq!(
        result(&format!(
            "{ORDER}`v ${{quantity why total |> xor(why freight)}}`"
        )),
        json!("v 3")
    );
    assert_eq!(
        failure(&format!("{ORDER}why total | > xor(why freight)")),
        error(ErrorKind::Syntax, "Unexpected character '|'", 9, 11)
    );
    assert_eq!(
        failure(&format!("{ORDER}why total ||> xor(why freight)")),
        error(ErrorKind::Syntax, "Unexpected '>'", 9, 15)
    );
    assert_eq!(
        failure(&format!("{ORDER}tax.xor(why freight)")),
        error(ErrorKind::Type, "number.xor is not a function", 9, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}why xor(freight)")),
        error(ErrorKind::Type, "Cannot explain xor(freight)", 9, 1)
    );
    assert_eq!(
        result(&format!(
            "{ORDER}(why total |> except(why freight) |> union(why freight |> except(why total))).map(s -> s.node)"
        )),
        json!(["total", "subtotal", "tax", "price", "freight", "shipping"])
    );
}

#[test]
fn inputs_that_were_errors_now_give_what_the_proposal_lists() {
    assert_eq!(
        result(&format!(
            "{ORDER}apart = why total |> xor(why freight)\napart.map(s -> s.node)"
        )),
        json!(["total", "subtotal", "tax", "price", "freight", "shipping"])
    );
    assert_eq!(
        failure(&format!("{ORDER}xor = 1\nwhy total |> xor(why freight)")),
        error(ErrorKind::Type, "xor is not an operation", 10, 1)
    );
    assert_eq!(
        failure(&format!(
            "{ORDER}def xor(x) => x\nwhy total |> xor(why freight)"
        )),
        error(ErrorKind::Type, "xor is not an operation", 10, 1)
    );
    assert_eq!(
        result(&format!(
            "{ORDER}why total |> xor(why freight |> depth(null))"
        )),
        Json::Null
    );
    assert_eq!(
        result(&format!("{ORDER}(why total |> xor (why freight)).length")),
        json!(6.0)
    );
    assert_eq!(
        result(&format!("{ORDER}(why total |> xor(\nwhy freight)).length")),
        json!(6.0)
    );
    assert_eq!(
        result(&format!(
            "{ORDER}(why total |> xor(model)).map(s -> s.node)"
        )),
        json!(["shipping", "freight"])
    );
    assert_eq!(
        result(&format!(
            "{SIX}x, y = c |> why |> xor(f |> why)\n[x.node, y.node]"
        )),
        json!(["c", "b"])
    );
    assert_eq!(
        result(&format!(
            "{SIX}{{length}} = c |> why |> xor(f |> why)\nlength"
        )),
        json!(5.0)
    );
    assert_eq!(
        failure(&format!("{ORDER}[p] = why total |> xor(why freight)\n")),
        error(ErrorKind::Syntax, "Cannot assign to [p]", 9, 1)
    );
    assert_eq!(
        failure(&format!("{ORDER}not why total |> xor(why freight)\n")),
        error(ErrorKind::Type, "Cannot explain !total|>why", 9, 1)
    );
    assert_eq!(
        failure(&format!(
            "{ORDER}`v ${{quantity (why total |> xor(why freight))}}`\n"
        )),
        error(ErrorKind::Type, "quantity is not a function", 9, 1)
    );
    assert_eq!(
        failure(&format!(
            "{ORDER}n = null\nfor s in why total |> xor(why freight |> depth(n)):\n    s\n"
        )),
        error(
            ErrorKind::Type,
            "total|>why|>xor(freight|>why|>depth(n)) is not iterable",
            10,
            1
        )
    );
    assert_eq!(
        result(&format!(
            "{ORDER}big = [1, 5, 9].filter(> 4)\napart = why total |> xor(why freight)\n[big, apart.length]"
        )),
        json!([[5.0, 9.0], 6.0])
    );
    assert_eq!(
        result(&format!(
            "{ORDER}class Box:\n    def size(self):\n        return 1\napart = why total |> xor(why freight)\n[Box().size(), apart.length]"
        )),
        json!([1.0, 6.0])
    );
    assert_eq!(
        result(&format!(
            "{ORDER}y = 1 if why total |> xor(why freight) else 2\ny"
        )),
        json!(1.0)
    );
    assert_eq!(
        result(&format!(
            "{ORDER}x = suppose: (why total |> xor(why freight)).length\nx"
        )),
        json!(6.0)
    );
    assert_eq!(
        result(&format!(
            "{ORDER}first = 1\n(why total |> xor(why freight)).length + first"
        )),
        json!(7.0)
    );
    assert_eq!(
        result(&format!("{ORDER}why total |> xor(why freight)(2)")),
        json!(2.0)
    );
}

#[test]
fn the_binary_runs_the_stage_from_files_and_the_line_repl() {
    let output = cli(
        "file",
        &format!("{CHAIN}(why b |> xor(why c)).map(s -> s.node)\n"),
    );
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("[\"c\"]\n")
    );
    for ending in ["\n", "\r\n"] {
        let lines = [
            "a = 1",
            "b = a + 2",
            "c = b * 2",
            "(why c |> xor(why b)).length",
            "why c |> xor(",
            "(why a |> xor(why b)).length",
            "why c |> xor",
            "xor = 1",
            "why c |> xor(why b)",
        ];
        let output = repl(&["--plain", "--json"], &(lines.join(ending) + ending));
        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{BANNER}1.0\n3.0\n6.0\n1.0\n1.0\n1.0\n")
        );
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "SyntaxError: Unexpected end of input (line 2, column 1)\nTypeError: xor is not an operation\nTypeError: xor is not an operation (line 1, column 1)\n"
        );
        let plain = repl(&["--plain"], &(lines.join(ending) + ending));
        assert!(plain.status.success(), "{plain:?}");
        assert_eq!(
            String::from_utf8(plain.stdout).unwrap(),
            format!("{BANNER}1\n3\n6\n1\n1\n1\n")
        );
    }
}
