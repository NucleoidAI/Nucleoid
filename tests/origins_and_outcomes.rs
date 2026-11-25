mod common;

use std::collections::HashSet;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::Runtime;
use nucleoid::graph::{NodeKey, ShapeKey};
use nucleoid::lang::ast::generator::generate;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, parse, parse_expression};
use nucleoid::nuc::Nuc;
use nucleoid::reasoning::{Stage, is_reasoning};

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid finds where a change enters and where it ends with origins and outcomes",
    "Nucleoid finds the inputs and the results of the model with origins and outcomes",
    "Nucleoid finds the inputs and the results behind class level rules with origins and outcomes",
    "Nucleoid rolls back failed updates through origins and outcomes",
];
const REFERENCE_TITLE: &str = "Nucleoid keeps the ends of selections with origins and outcomes";
const README_TITLE: &str =
    "Origins and outcomes show where a change of pressure enters and which alarms it ends at";
const EXAMPLES_TITLE: &str =
    "Origins and outcomes show where more nights enter a trip budget and the figures they end at";

const CHAIN: &str = "a = 1\nb = a + 1\nc = b * 2\n";
const INVOICE: &str = "price = 10\nquantity = 3\nrate = 0.2\nsubtotal = price * quantity\ntax = subtotal * rate\ntotal = subtotal + tax\nunit = total / quantity\nshipping = quantity * 2\n";
const DIAMOND: &str = "a = 1\nb = a + 1\nc = a * 2\nd = b + c\ne = d + a\n";
const OBSERVED: &str =
    "a = 1\nb = a + 1\nreach = b |> why\nn = reach.length\nm = n * 2\nk = a + 5\n";
const DELETED: &str = "x = 1\ny = x * 2\nz = y + 1\nw = 3\nv = w + z\ndelete x\n";
const CLASSY: &str = "fee = 2\nclass Order(price, qty):\n    this.price = price\n    this.qty = qty\n$Order.subtotal = $Order.price * $Order.qty\n$Order.total = $Order.subtotal + fee\nfirst = Order(10, 2)\nsecond = Order(4, 5)\n";
const CYCLE: &str = "class Box(width, height):\n    this.width = width\n    this.height = height\n$Box.width = $Box.height@first\n$Box.height = $Box.width@first\nbox = Box(3, 2)\n";
const CYCLIC: &str = "class Box(width, height):\n    this.width = width\n    this.height = height\n$Box.width = $Box.height@first\n$Box.height = $Box.width@first\n$Box.area = $Box.width * $Box.height\nbox = Box(3, 2)\nprice = 10\ntotal = price + box.area\n";
const ANCHORED: &str = "class Box(width, height):\n    this.width = width\n    this.height = height\n$Box.width = $Box.width@first\nbox = Box(3, 2)\n";

type Graph = (
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
);

const GRAPHS: [Graph; 6] = [
    (
        INVOICE,
        &["price", "quantity", "subtotal", "tax", "total", "unit"],
        &[
            "fee = 2",
            "total = subtotal + tax + fee",
            "grand = total * 2",
            "quantity = 5",
            "delete shipping",
            "shipping = quantity * 3",
            "tax = price * rate",
            "delete grand",
        ],
    ),
    (
        DIAMOND,
        &["a", "b", "c", "d", "e"],
        &["f = d * 2", "c = 5", "b = c + 1", "delete d", "d = a + b"],
    ),
    (
        OBSERVED,
        &["a", "b", "n", "m", "k"],
        &["j = m + 1", "b = 7", "reach = k |> why", "delete n"],
    ),
    (
        DELETED,
        &["y", "z", "w", "v"],
        &["x = 4", "u = v + y", "delete w", "w = 9"],
    ),
    (
        CLASSY,
        &[
            "fee",
            "first.total",
            "first.subtotal",
            "first.price",
            "second.total",
        ],
        &[
            "third = Order(1, 1)",
            "$Order.due = $Order.total * 0.9",
            "revenue = first.total + second.total",
            "first.qty = 7",
        ],
    ),
    (
        CYCLIC,
        &["box.width", "box.height", "box.area", "price", "total"],
        &[
            "$Box.height = 5",
            "$Box.height = $Box.width@first",
            "price = box.width + 1",
            "spare = Box(1, 4)",
            "$Box.width = 4",
            "$Box.width = $Box.height@first",
        ],
    ),
];

const SOURCES: [&str; 20] = [
    "X",
    "X |> why",
    "why X",
    "X |> affects",
    "affects X",
    "model",
    "X |> why |> stated",
    "X |> why |> derived",
    "model |> derived",
    "X |> why |> depth(1)",
    "X |> affects |> depth(1)",
    "X |> why |> path(Y)",
    "X |> affects |> path(Y)",
    "X |> why |> except(Y)",
    "X |> why |> except(Y |> why)",
    "X |> why |> intersect(Y |> why)",
    "X |> why |> union(Y |> why)",
    "X |> affects |> union(Y |> affects)",
    "model |> except(X |> why)",
    "X |> affects |> why",
];

type Rewrite = (&'static str, &'static str);

struct Counterpart {
    title: &'static str,
    rewrites: &'static [Rewrite],
    before: usize,
    after: usize,
}

const SPELLED: [Counterpart; 7] = [
    Counterpart {
        title: "Nucleoid finds where a change enters and where it ends with origins and outcomes",
        rewrites: &[
            (
                "quantity |> affects |> origins",
                "affects quantity |> except(affects affects quantity)",
            ),
            (
                "quantity |> affects |> outcomes",
                "[s for s in affects quantity if not any t in affects quantity: s.node in t.from]",
            ),
        ],
        before: 214,
        after: 192,
    },
    Counterpart {
        title: "Nucleoid finds the inputs and the results of the model with origins and outcomes",
        rewrites: &[
            ("model |> origins", "model |> except(model |> affects)"),
            (
                "model |> outcomes",
                "[s for s in model |> why if not any t in model |> why: s.node in t.from]",
            ),
        ],
        before: 221,
        after: 194,
    },
    Counterpart {
        title: "Nucleoid finds the inputs and the results behind class level rules with origins and outcomes",
        rewrites: &[
            (
                "$Order.total |> why |> origins",
                "why $Order.total |> except(affects why $Order.total)",
            ),
            (
                "fee |> affects |> outcomes",
                "[s for s in affects fee if not any t in affects fee: s.node in t.from]",
            ),
        ],
        before: 243,
        after: 219,
    },
    Counterpart {
        title: "Nucleoid rolls back failed updates through origins and outcomes",
        rewrites: &[
            (
                "qty |> affects |> origins",
                "affects qty |> except(affects affects qty)",
            ),
            (
                "qty |> affects |> outcomes",
                "[s for s in affects qty if not any t in affects qty: s.node in t.from]",
            ),
            ("1 |> outcomes", "1 |> why"),
        ],
        before: 268,
        after: 246,
    },
    Counterpart {
        title: "Nucleoid keeps the ends of selections with origins and outcomes",
        rewrites: &[
            (
                "qty |> affects |> origins",
                "affects qty |> except(affects affects qty)",
            ),
            (
                "model |> outcomes",
                "[s for s in model |> why if not any t in model |> why: s.node in t.from]",
            ),
        ],
        before: 135,
        after: 109,
    },
    Counterpart {
        title: "Origins and outcomes show where a change of pressure enters and which alarms it ends at",
        rewrites: &[
            (
                "pressure |> affects |> origins",
                "affects pressure |> except(affects affects pressure)",
            ),
            (
                "pressure |> affects |> outcomes",
                "[s for s in affects pressure if not any t in affects pressure: s.node in t.from]",
            ),
        ],
        before: 176,
        after: 154,
    },
    Counterpart {
        title: "Origins and outcomes show where more nights enter a trip budget and the figures they end at",
        rewrites: &[
            (
                "nights |> affects |> origins",
                "affects nights |> except(affects affects nights)",
            ),
            (
                "nights |> affects |> outcomes",
                "[s for s in affects nights if not any t in affects nights: s.node in t.from]",
            ),
        ],
        before: 174,
        after: 152,
    },
];

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

fn is_appended(title: &str) -> bool {
    SPEC_TITLES.contains(&title) || [REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE].contains(&title)
}

fn writes_ends(source: &str) -> bool {
    tokenize(source).unwrap().windows(2).any(|window| {
        window[0].token == Token::Pipe
            && matches!(&window[1].token, Token::Identifier(name) if name == "origins" || name == "outcomes")
    })
}

fn tokens(source: &str) -> usize {
    tokenize(source.trim())
        .unwrap()
        .iter()
        .filter(|spanned| spanned.token != Token::Eof)
        .count()
}

fn outcome(runtime: &mut Runtime, source: &str) -> String {
    match runtime.run(source) {
        Ok(value) => runtime.serialize_json(&value).unwrap(),
        Err(error) => error.to_string(),
    }
}

fn fresh(source: &str) -> String {
    outcome(&mut Runtime::new(), source)
}

fn result(source: &str) -> String {
    let mut runtime = Runtime::new();
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    runtime.serialize_json(&value).unwrap()
}

fn failure(source: &str) -> String {
    Runtime::new()
        .run(source)
        .expect_err("the program raises an error")
        .to_string()
}

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "origins and outcomes regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn spelled(form: &str, x: &str, y: &str) -> String {
    form.replace('X', x).replace('Y', y)
}

fn named(selection: &str) -> String {
    format!("({selection}).map(step => step.node)")
}

fn selected(selection: &str) -> String {
    format!("({selection} |> intersect(model)).map(step => step.node)")
}

fn names(runtime: &mut Runtime, selection: &str) -> Option<Vec<String>> {
    let value = runtime.run(&selected(selection)).ok()?;
    serde_json::from_str(&runtime.serialize_json(&value).unwrap()).ok()
}

fn observer(runtime: &Runtime, key: &NodeKey) -> bool {
    runtime
        .graph
        .shape_dependencies(key)
        .any(|shape| matches!(shape, ShapeKey::Node(_) | ShapeKey::Model))
        || runtime
            .graph
            .retrieve(key)
            .and_then(|node| node.node.as_ref())
            .is_some_and(is_reasoning)
}

fn rests_on(runtime: &Runtime, step: &str, other: &str) -> bool {
    let target = NodeKey::variable(other);
    let mut seen: HashSet<NodeKey> = HashSet::new();
    let mut pending = vec![NodeKey::variable(step)];
    while let Some(current) = pending.pop() {
        let Some(node) = runtime.graph.retrieve(&current) else {
            continue;
        };
        for dependency in &node.dependencies {
            if observer(runtime, dependency) {
                continue;
            }
            if *dependency == target {
                return true;
            }
            if seen.insert(dependency.clone()) {
                pending.push(dependency.clone());
            }
        }
    }
    false
}

fn ends(runtime: &Runtime, steps: &[String]) -> (Vec<String>, Vec<String>) {
    let origins = steps
        .iter()
        .filter(|step| !steps.iter().any(|other| rests_on(runtime, step, other)))
        .cloned()
        .collect();
    let outcomes = steps
        .iter()
        .filter(|step| !steps.iter().any(|other| rests_on(runtime, other, step)))
        .cloned()
        .collect();
    (origins, outcomes)
}

fn json(names: &[String]) -> String {
    serde_json::to_string(names).unwrap()
}

fn observes(runtime: &Runtime, name: &str) -> Vec<String> {
    let mut shapes: Vec<String> = runtime
        .graph
        .shape_dependencies(&NodeKey::variable(name))
        .map(|shape| format!("{shape:?}"))
        .collect();
    shapes.sort();
    shapes
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

fn reason(stage: Stage, source: Expr) -> Expr {
    Expr::Reason {
        stage,
        source: Box::new(source),
    }
}

fn identifier(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

#[test]
fn only_the_appended_cases_write_origins_or_outcomes_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(writes_ends(&case.source), new, "{}", case.title);
        if new {
            let count =
                common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            common::check_with_assertions(case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assertions += count;
        }
    }
    for title in SPEC_TITLES
        .into_iter()
        .chain([REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE])
    {
        let matching = documents
            .iter()
            .flat_map(|(_, cases)| cases)
            .filter(|case| case.title == title)
            .count();
        assert_eq!(matching, 1, "{title}");
    }
    assert_eq!(assertions, 28);
    assert_eq!(cases, 1316);
}

#[test]
fn the_audit_finds_every_end_stage_and_nothing_else() {
    for source in [
        "c |> origins",
        "c |> why |> outcomes",
        "why c |> stated |> origins",
        "x = model|>outcomes",
        "f(c |> why |> origins)",
        "$P.ends = $P.v |> affects |> outcomes",
        "c |> why |> origins |> why",
        "c |> why |> outcomes(1)",
    ] {
        assert!(writes_ends(source), "{source}");
    }
    for source in [
        "origins",
        "origins c",
        "outcomes(c |> why)",
        "origins = [1, 2]",
        "o = {outcomes: 1}",
        "o.origins",
        "def outcomes(x) => x",
        "c |> why |> Origins",
        "c |> why |> outcome",
        "c |> why |>\norigins",
        "c |> why |> holds",
        "\"|> origins\"",
    ] {
        assert!(!writes_ends(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_125_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[815],
        "Nucleoid rolls back failed updates through stepped slices"
    );
    assert_eq!(titles[820..824], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    assert_eq!(titles.len(), 881);
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid walks lists from a position with stepped slices",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Stepped slices deal the chores round-robin as roommates move in",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Stepped slices mirror the rows of every sprite",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 12 - 7 * usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 2].title, previous);
        assert_eq!(cases[index + 4].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Origins and outcomes of reasoning selections\n\n```\n")
        .expect("the language reference documents origins and outcomes");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(
        case.expected.as_deref(),
        Some("[[\"subtotal\",\"fee\"],[\"unit\",\"fee\"]]")
    );
    let summary = reference.find("reference |> affects |> origins").unwrap();
    assert!(summary < reference.find("reference |> why |> nodes").unwrap());
    for line in [
        "[NUC 128](nuc-0128.md)",
        "reference |> affects |> origins       where a change of the reference enters the model (NUC 128)",
        "reference |> affects |> outcomes      where a change of the reference ends (NUC 128)",
        "model |> origins                      the model's sources, which rest on no node (NUC 128)",
        "model |> outcomes                     the model's results, which no node rests on (NUC 128)",
        "selection |> origins                  steps that rest on no step of the selection (NUC 128)",
        "selection |> outcomes                 steps that no step of the selection rests on (NUC 128)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    let sections = reference.find("### Union selections").unwrap();
    let ends = reference
        .find("### Origins and outcomes of reasoning selections")
        .unwrap();
    assert!(sections < ends && ends < reference.find("### Node names").unwrap());
    assert!(include_str!("../README.md").contains("[NUC 128](docs/nuc-0128.md)"));
    assert!(include_str!("../docs/examples.md").contains("[NUC 128](nuc-0128.md)"));
    let spec = include_str!("../nucleoid.spec.md").replace("\r\n", "\n");
    let contract = spec
        .find("\n## Origins and outcomes of reasoning selections\n")
        .unwrap();
    assert!(spec.find("\n## Union reasoning selections\n").unwrap() < contract);
    assert!(
        contract
            < spec
                .find("\n## Node names of reasoning selections\n")
                .unwrap()
    );
}

#[test]
fn every_case_is_its_frozen_counterpart_with_only_the_stages_written_out() {
    let documents = documents();
    for Counterpart {
        title,
        rewrites,
        before,
        after,
    } in SPELLED
    {
        let case = documents
            .iter()
            .flat_map(|(_, cases)| cases)
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
        assert!(writes_ends(&case.source), "{title}");
        assert!(!writes_ends(&counterpart), "{title}");
        assert_eq!(tokens(&case.source), after, "{title}");
        assert_eq!(tokens(&counterpart), before, "{title}");
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
    }
}

#[test]
fn the_stages_parse_as_filters_and_stay_ordinary_names() {
    let why_c = reason(Stage::Why, identifier("c"));
    let origins = reason(Stage::Origins, why_c.clone());
    let outcomes = reason(Stage::Outcomes, why_c.clone());
    assert_eq!(parse_expression("c |> why |> origins").unwrap(), origins);
    assert_eq!(parse_expression("why c |> origins").unwrap(), origins);
    assert_eq!(parse_expression("c|>why|>origins").unwrap(), origins);
    assert_eq!(
        parse_expression("(c |>\nwhy |>\norigins)").unwrap(),
        origins
    );
    assert_eq!(parse_expression("c |> why |> outcomes").unwrap(), outcomes);
    assert_eq!(
        parse_expression("model |> outcomes").unwrap(),
        reason(Stage::Outcomes, Expr::Model)
    );
    assert_eq!(
        parse_expression("c |> origins").unwrap(),
        reason(Stage::Origins, identifier("c"))
    );
    assert_eq!(
        parse_expression("c |> why |> origins |> outcomes |> why").unwrap(),
        reason(Stage::Why, reason(Stage::Outcomes, origins.clone()))
    );
    assert_eq!(
        parse_expression("c |> why |> origins |> nodes").unwrap(),
        reason(Stage::Nodes, origins)
    );
    assert_eq!(Stage::Origins.as_str(), "origins");
    assert_eq!(Stage::Outcomes.as_str(), "outcomes");
    assert_eq!(Stage::from_name("origins"), None);
    assert_eq!(Stage::from_name("outcomes"), None);
    assert_eq!(Stage::from_name("stated"), Some(Stage::Stated));
    for (source, expected) in [
        (
            format!("{CHAIN}origins c"),
            "ReferenceError: origins is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}outcomes"),
            "ReferenceError: outcomes is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}origins(c |> why)"),
            "ReferenceError: origins is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> Origins"),
            "TypeError: Origins is not an operation",
        ),
        (
            format!("{CHAIN}c |> why |> outcome"),
            "TypeError: outcome is not an operation",
        ),
        (
            format!("{CHAIN}c |> why |>\norigins"),
            "SyntaxError: Expected an operation (line 4, column 10)",
        ),
        (
            format!("{CHAIN}c |> why |> origins.length"),
            "SyntaxError: Unexpected '.' (line 4, column 21)",
        ),
        (format!("{CHAIN}c |> why |> origins(1)"), "1.0"),
        (format!("{CHAIN}c |> why |> outcomes[0]"), "[0.0]"),
        (format!("{CHAIN}(c |> why |> origins).length"), "1.0"),
        (format!("{CHAIN}origins = [1, 2]\norigins"), "[1.0,2.0]"),
        (format!("{CHAIN}x = {{outcomes: 1}}\nx.outcomes"), "1.0"),
        (format!("{CHAIN}def origins(x) => x * 2\norigins(3)"), "6.0"),
        (format!("{CHAIN}(c |> why).origins"), "null"),
        (format!("{CHAIN}xs = [1]\nxs.outcomes"), "null"),
    ] {
        assert_eq!(fresh(&source), expected, "{source}");
    }
}

#[test]
fn origins_keep_what_except_affects_keeps_and_both_match_an_independent_walk() {
    let mut compared = 0;
    for (setup, nodes, _) in GRAPHS {
        let mut oracle = Runtime::new();
        checked(&mut oracle, setup);
        let mut seen = HashSet::new();
        for x in nodes {
            for y in nodes {
                for form in SOURCES {
                    let selection = spelled(form, x, y);
                    if !seen.insert(selection.clone()) {
                        continue;
                    }
                    let origins = fresh(&format!(
                        "{setup}{}",
                        named(&format!("{selection} |> origins"))
                    ));
                    let accepted = fresh(&format!(
                        "{setup}{}",
                        named(&format!("{selection} |> except({selection} |> affects)"))
                    ));
                    assert_eq!(origins, accepted, "{setup}{selection}");
                    let outcomes = fresh(&format!(
                        "{setup}{}",
                        named(&format!("{selection} |> outcomes"))
                    ));
                    match names(&mut oracle, &selection) {
                        Some(steps) => {
                            let (first, last) = ends(&oracle, &steps);
                            assert_eq!(origins, json(&first), "{setup}{selection}");
                            assert_eq!(outcomes, json(&last), "{setup}{selection}");
                        }
                        None => {
                            assert!(origins.contains("Error"), "{setup}{selection}");
                            assert_eq!(outcomes, origins, "{setup}{selection}");
                        }
                    }
                    compared += 1;
                }
            }
        }
    }
    assert_eq!(compared, 1406);
}

#[test]
fn outcomes_over_walked_selections_keep_the_steps_that_no_from_names() {
    for (setup, nodes, _) in GRAPHS {
        for x in nodes {
            for walk in ["affects X", "why X", "X |> affects |> why"] {
                let selection = spelled(walk, x, x);
                let stage = fresh(&format!(
                    "{setup}{}",
                    named(&format!("{selection} |> outcomes"))
                ));
                let filter = fresh(&format!(
                    "{setup}[s for s in {selection} if not any t in {selection}: s.node in t.from].map(step => step.node)"
                ));
                assert_eq!(stage, filter, "{setup}{selection}");
            }
        }
        let stage = fresh(&format!("{setup}{}", named("model |> outcomes")));
        let filter = fresh(&format!(
            "{setup}[s for s in model |> why if not any t in model |> why: s.node in t.from].map(step => step.node)"
        ));
        assert_eq!(stage, filter, "{setup}");
    }
}

#[test]
fn thinned_selections_follow_the_chains_through_the_steps_they_drop() {
    let invoice = "price = 10\nquantity = 3\nrate = 0.2\nsubtotal = price * quantity\ntax = subtotal * rate\ntotal = subtotal + tax\nshipping = quantity * 2\n";
    for (selection, origins, outcomes) in [
        (
            "total |> why |> except(subtotal)",
            "[\"rate\",\"price\",\"quantity\"]",
            "[\"total\"]",
        ),
        ("total |> why |> depth(1)", "[\"subtotal\"]", "[\"total\"]"),
        ("total |> why |> derived", "[\"subtotal\"]", "[\"total\"]"),
        (
            "total |> why |> stated",
            "[\"rate\",\"price\",\"quantity\"]",
            "[\"rate\",\"price\",\"quantity\"]",
        ),
        (
            "total |> why |> intersect(price |> affects |> union(price))",
            "[\"price\"]",
            "[\"total\"]",
        ),
        (
            "tax |> why |> union(shipping |> why)",
            "[\"rate\",\"price\",\"quantity\"]",
            "[\"tax\",\"shipping\"]",
        ),
        (
            "model |> except(subtotal |> union(tax))",
            "[\"price\",\"quantity\",\"rate\"]",
            "[\"total\",\"shipping\"]",
        ),
        (
            "price |> affects |> except(tax)",
            "[\"subtotal\"]",
            "[\"total\"]",
        ),
    ] {
        assert_eq!(
            result(&format!(
                "{invoice}{}",
                named(&format!("{selection} |> origins"))
            )),
            origins,
            "{selection}"
        );
        assert_eq!(
            result(&format!(
                "{invoice}{}",
                named(&format!("{selection} |> outcomes"))
            )),
            outcomes,
            "{selection}"
        );
    }
    assert_eq!(
        result(&format!(
            "{invoice}[s for s in total |> why |> except(subtotal) if not any t in total |> why |> except(subtotal): s.node in t.from].map(step => step.node)"
        )),
        "[\"total\",\"price\",\"quantity\"]"
    );
    assert_eq!(
        result(&format!(
            "{invoice}suppose price = 20:\n    [(total |> why |> diff |> origins).map(step => step.node), (total |> why |> diff |> outcomes).map(step => step.node)]"
        )),
        "[[\"price\"],[\"total\"]]"
    );
}

#[test]
fn reasoning_observers_break_chains_and_pending_nodes_end_them() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, OBSERVED);
    assert_eq!(
        outcome(&mut runtime, &named("model |> origins")),
        "[\"a\",\"n\"]"
    );
    assert_eq!(
        outcome(&mut runtime, &named("model |> outcomes")),
        "[\"b\",\"m\",\"k\"]"
    );
    assert_eq!(
        outcome(&mut runtime, &named("m |> why |> origins")),
        "[\"n\"]"
    );
    let mut deleted = Runtime::new();
    checked(&mut deleted, DELETED);
    assert_eq!(
        outcome(&mut deleted, &named("model |> origins")),
        "[\"y\",\"w\"]"
    );
    assert_eq!(
        outcome(&mut deleted, &named("v |> why |> origins")),
        "[\"w\",\"y\"]"
    );
    assert_eq!(outcome(&mut deleted, &named("model |> stated")), "[\"w\"]");
    checked(&mut deleted, "x = 4");
    assert_eq!(
        outcome(&mut deleted, &named("v |> why |> origins")),
        "[\"w\",\"x\"]"
    );
    assert_eq!(
        outcome(&mut deleted, &named("x |> affects |> outcomes")),
        "[\"v\"]"
    );
}

#[test]
fn steps_on_a_cycle_of_fixed_anchors_rest_on_themselves_and_are_no_ends() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, CYCLE);
    assert!(rests_on(&runtime, "box.width", "box.width"));
    assert!(rests_on(&runtime, "box.height", "box.height"));
    assert_eq!(
        outcome(&mut runtime, "box.width |> affects |> nodes"),
        "[\"box.height\",\"box.width\"]"
    );
    assert_eq!(
        outcome(&mut runtime, "box.width |> why |> nodes"),
        "[\"box.width\",\"box.height\"]"
    );
    for (selection, kept) in [
        ("box.width", "[]"),
        ("box.height", "[]"),
        ("box.width |> why", "[]"),
        ("box.width |> affects", "[]"),
        ("box.width |> union(box.height)", "[]"),
        ("box.width |> why |> depth(1)", "[]"),
        ("model", "[\"$Box\",\"box\"]"),
        ("model |> except(box.height)", "[\"$Box\",\"box\"]"),
    ] {
        for stage in ["origins", "outcomes"] {
            assert_eq!(
                outcome(&mut runtime, &named(&format!("{selection} |> {stage}"))),
                kept,
                "{selection} |> {stage}"
            );
        }
        assert_eq!(
            outcome(
                &mut runtime,
                &named(&format!("{selection} |> except({selection} |> affects)"))
            ),
            kept,
            "{selection}"
        );
        let steps = names(&mut runtime, selection).unwrap();
        let (first, last) = ends(&runtime, &steps);
        assert_eq!(json(&first), kept, "{selection}");
        assert_eq!(json(&last), kept, "{selection}");
    }
}

#[test]
fn a_cycle_beside_acyclic_steps_ends_where_its_chains_leave_it() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, CYCLIC);
    for (selection, origins, outcomes) in [
        (
            "model",
            "[\"$Box\",\"box\",\"price\"]",
            "[\"$Box\",\"total\"]",
        ),
        ("total |> why", "[\"price\",\"box\"]", "[\"total\"]"),
        ("box.area |> why", "[]", "[\"box.area\"]"),
        ("box.width |> affects", "[]", "[\"total\"]"),
        ("box.width |> affects |> depth(1)", "[]", "[\"box.area\"]"),
        ("box.width |> union(price)", "[\"price\"]", "[\"price\"]"),
        ("box.width |> union(box.area)", "[]", "[\"box.area\"]"),
        (
            "total |> why |> except(box.area)",
            "[\"price\",\"box\"]",
            "[\"total\"]",
        ),
    ] {
        assert_eq!(
            outcome(&mut runtime, &named(&format!("{selection} |> origins"))),
            origins,
            "{selection}"
        );
        assert_eq!(
            outcome(&mut runtime, &named(&format!("{selection} |> outcomes"))),
            outcomes,
            "{selection}"
        );
        assert_eq!(
            outcome(
                &mut runtime,
                &named(&format!("{selection} |> except({selection} |> affects)"))
            ),
            origins,
            "{selection}"
        );
        let steps = names(&mut runtime, selection).unwrap();
        let (first, last) = ends(&runtime, &steps);
        assert_eq!(json(&first), origins, "{selection}");
        assert_eq!(json(&last), outcomes, "{selection}");
    }
}

type Standing = (
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
);

#[test]
fn standing_ends_follow_a_cycle_that_is_broken_and_restored() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, CYCLIC);
    checked(
        &mut runtime,
        "starts = model |> origins |> nodes\nspelled = model |> except(model |> affects) |> nodes\nsinks = model |> outcomes |> nodes\nlone = box.width |> origins |> nodes\nlonely = box.width |> except(box.width |> affects) |> nodes\nlast = box.width |> outcomes |> nodes",
    );
    let states: [Standing; 7] = [
        ("", &["$Box", "box", "price"], &["$Box", "total"], &[]),
        (
            "$Box.height = 5",
            &["$Box", "box", "box.height", "price"],
            &["$Box", "total"],
            &["box.width"],
        ),
        (
            "$Box.height = $Box.width@first",
            &["$Box", "box", "price"],
            &["$Box", "total"],
            &[],
        ),
        (
            "price = box.width + 1",
            &["$Box", "box"],
            &["$Box", "total"],
            &[],
        ),
        (
            "spare = Box(1, 4)",
            &["$Box", "box", "spare"],
            &["$Box", "total", "spare", "spare.area"],
            &[],
        ),
        (
            "$Box.width = 4",
            &["$Box", "box", "box.width", "spare", "spare.width"],
            &["$Box", "total", "spare", "spare.area"],
            &["box.width"],
        ),
        (
            "$Box.width = $Box.height@first",
            &["$Box", "box", "spare"],
            &["$Box", "total", "spare", "spare.area"],
            &[],
        ),
    ];
    for (write, starts, sinks, lone) in states {
        if !write.is_empty() {
            checked(&mut runtime, write);
        }
        assert_eq!(
            outcome(&mut runtime, "[starts, spelled, sinks, lone, lonely, last]"),
            serde_json::to_string(&[starts, starts, sinks, lone, lone, lone]).unwrap(),
            "{write}"
        );
    }
}

#[test]
fn a_property_anchored_only_on_itself_stays_an_ordinary_step() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, ANCHORED);
    assert!(!rests_on(&runtime, "box.width", "box.width"));
    assert_eq!(outcome(&mut runtime, "box.width"), "3.0");
    assert_eq!(
        outcome(&mut runtime, "(box.width |> why).map(step => step.from)"),
        "[[]]"
    );
    assert_eq!(outcome(&mut runtime, "box.width |> affects |> nodes"), "[]");
    for (selection, kept) in [
        ("box.width", "[\"box.width\"]"),
        ("box.width |> why", "[\"box.width\"]"),
        ("model", "[\"$Box\",\"box\",\"box.width\",\"box.height\"]"),
    ] {
        for stage in ["origins", "outcomes"] {
            assert_eq!(
                outcome(&mut runtime, &named(&format!("{selection} |> {stage}"))),
                kept,
                "{selection} |> {stage}"
            );
        }
        assert_eq!(
            outcome(
                &mut runtime,
                &named(&format!("{selection} |> except({selection} |> affects)"))
            ),
            kept,
            "{selection}"
        );
    }
}

#[test]
fn the_kept_steps_keep_their_fields_order_distances_and_direction() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, INVOICE);
    let records = outcome(&mut runtime, "quantity |> affects |> origins");
    let spelled = outcome(
        &mut runtime,
        "affects quantity |> except(affects affects quantity)",
    );
    assert_eq!(records, spelled);
    assert_eq!(
        records,
        "[{\"from\":[\"price\",\"quantity\"],\"holds\":30.0,\"node\":\"subtotal\",\"rule\":\"subtotal = price*quantity\",\"state\":\"derived\"},{\"from\":[\"quantity\"],\"holds\":6.0,\"node\":\"shipping\",\"rule\":\"shipping = quantity*2\",\"state\":\"derived\"}]"
    );
    for (selection, expected) in [
        (
            "quantity |> affects |> outcomes |> depth(1)",
            "[\"unit\",\"shipping\"]",
        ),
        ("quantity |> affects |> outcomes |> depth(0)", "[]"),
        ("total |> why |> origins |> depth(1)", "[]"),
        (
            "total |> why |> origins |> depth(2)",
            "[\"rate\",\"price\",\"quantity\"]",
        ),
        ("total |> why |> origins |> path(price)", "[\"price\"]"),
        ("price |> affects |> outcomes |> path(subtotal)", "[]"),
        ("price |> affects |> outcomes |> path(unit)", "[\"unit\"]"),
        ("model |> origins |> path(price)", "[\"price\"]"),
        (
            "price |> affects |> origins |> why",
            "[\"subtotal\",\"price\",\"quantity\"]",
        ),
        (
            "price |> affects |> origins |> affects",
            "[\"tax\",\"total\",\"unit\"]",
        ),
        ("model |> outcomes |> stated", "[]"),
        (
            "model |> origins |> outcomes",
            "[\"price\",\"quantity\",\"rate\"]",
        ),
        (
            "quantity |> affects |> outcomes |> origins",
            "[\"unit\",\"shipping\"]",
        ),
        ("price |> origins", "[\"price\"]"),
        ("price |> outcomes", "[\"price\"]"),
        ("model |> except(model) |> origins", "[]"),
        (
            "total |> why |> intersect(quantity |> affects |> outcomes)",
            "[]",
        ),
        (
            "unit |> why |> intersect(quantity |> affects |> outcomes)",
            "[\"unit\"]",
        ),
        (
            "tax |> why |> except(total |> why |> origins)",
            "[\"tax\",\"subtotal\"]",
        ),
        (
            "tax |> why |> union(quantity |> affects |> outcomes)",
            "[\"tax\",\"subtotal\",\"rate\",\"price\",\"quantity\",\"unit\",\"shipping\"]",
        ),
    ] {
        assert_eq!(
            outcome(&mut runtime, &named(selection)),
            expected,
            "{selection}"
        );
    }
    assert_eq!(
        outcome(&mut runtime, "quantity |> affects |> outcomes |> nodes"),
        "[\"unit\",\"shipping\"]"
    );
    assert_eq!(
        outcome(&mut runtime, "quantity |> affects |> outcomes |> holds"),
        "[12.0,6.0]"
    );
}

#[test]
fn undefined_selections_stay_undefined() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{CHAIN}n = null"));
    assert_eq!(
        outcome(&mut runtime, "c |> why |> depth(n) |> origins"),
        "null"
    );
    assert_eq!(
        outcome(&mut runtime, "c |> why |> depth(n) |> outcomes"),
        "null"
    );
    assert_eq!(
        outcome(&mut runtime, "c |> why |> origins |> depth(n) |> outcomes"),
        "null"
    );
    checked(&mut runtime, "held = c |> why |> depth(n) |> origins");
    assert_eq!(outcome(&mut runtime, "held"), "null");
    assert_eq!(
        outcome(&mut runtime, "typeof (c |> why |> depth(n) |> outcomes)"),
        "\"undefined\""
    );
    assert_eq!(
        outcome(
            &mut runtime,
            "for s in c |> why |> depth(n) |> origins:\n    t = s"
        ),
        "TypeError: c|>why|>depth(n)|>origins is not iterable (line 1, column 1)"
    );
    checked(&mut runtime, "n = 1");
    assert_eq!(outcome(&mut runtime, &named("held")), "[\"b\"]");
}

#[test]
fn sources_and_targets_keep_their_accepted_diagnostics() {
    for (source, expected) in [
        (
            format!("{CHAIN}1 |> origins"),
            "TypeError: Cannot explain 1 (line 4, column 1)",
        ),
        (
            format!("{CHAIN}missing |> outcomes"),
            "ReferenceError: missing is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c + 1 |> outcomes"),
            "TypeError: Cannot explain c+1 (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> diff |> origins"),
            "TypeError: Cannot use 'diff' outside suppose (line 4, column 1)",
        ),
        (
            format!("{CHAIN}n = null\nn.p |> origins"),
            "ReferenceError: n is not defined (line 5, column 1)",
        ),
        (
            format!("{CHAIN}k = 5\nk.p |> outcomes"),
            "TypeError: Cannot explain k.p (line 5, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> depth(-1) |> origins"),
            "TypeError: -1 is not a non-negative integer (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> path(a |> why |> origins)"),
            "TypeError: Cannot explain a|>why|>origins (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> path(a |> affects |> outcomes)"),
            "TypeError: Cannot explain a|>affects|>outcomes (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> intersect(1 |> origins)"),
            "TypeError: Cannot explain 1 (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> union(missing |> outcomes)"),
            "ReferenceError: missing is not defined (line 4, column 1)",
        ),
    ] {
        assert_eq!(failure(&source), expected, "{source}");
    }
    assert_eq!(
        result(&format!("{CHAIN}o = {{}}\n{}", named("o.gone |> origins"))),
        "[]"
    );
    assert_eq!(result(&named("model |> outcomes")), "[]");
}

#[test]
fn every_binding_of_either_name_shadows_its_stage_at_run_time() {
    for name in ["origins", "outcomes"] {
        let bindings = [
            (format!("{name} = 1\nc |> why |> {name}"), 5),
            (format!("def {name}(x) => x\nc |> why |> {name}"), 5),
            (format!("class {name}\nc |> why |> {name}"), 5),
            (format!("{name} = 1\nmissing |> why |> {name}"), 5),
            (format!("{name} = 1\n1 |> {name}"), 5),
            (format!("def f({name}) => c |> why |> {name}\nf(1)"), 5),
            (format!("f = {name} -> c |> why |> {name}\nf(1)"), 5),
            (format!("f = lambda {name}: c |> why |> {name}\nf(1)"), 5),
            (format!("[1].map({name} -> c |> why |> {name})"), 4),
            (
                format!("for {name} of \"x\":\n    y = c |> why |> {name}"),
                4,
            ),
            (
                format!("ok = all {name} in [1]: (c |> why |> {name}).length > 0"),
                4,
            ),
            (
                format!("{{\n    {name} = 5\n    y = c |> why |> {name}\n}}"),
                4,
            ),
            (
                format!("class R:\n    def m({name}) => c |> why |> {name}\nR().m(1)"),
                6,
            ),
            (
                format!("try:\n    throw 1\ncatch {name}:\n    y = c |> why |> {name}"),
                4,
            ),
            (
                format!("stated = 1\n{name} = 2\nc |> why |> stated |> {name}"),
                6,
            ),
        ];
        for (binding, line) in bindings {
            assert_eq!(
                failure(&format!("{CHAIN}{binding}")),
                format!("TypeError: {name} is not an operation (line {line}, column 1)"),
                "{binding}"
            );
        }
    }
    assert_eq!(
        failure(&format!("{CHAIN}stated = 1\nc |> why |> stated |> origins")),
        "TypeError: stated is not an operation (line 5, column 1)"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}origins = 1\n{}",
            named("c |> why |> outcomes")
        )),
        "[\"c\"]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}outcomes = 1\n{}",
            named("c |> why |> origins")
        )),
        "[\"a\"]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}class K:\n    def origins() => 1\no = {{outcomes: 1}}\nq = {{}}\nq.origins = 2\n[(c |> why |> origins |> outcomes).length, o.outcomes, q.origins, K().origins()]"
        )),
        "[1.0,1.0,2.0,1.0]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}why = 1\naffects = 2\n[c |> why |> origins |> nodes, a |> affects |> outcomes |> nodes]"
        )),
        "[[\"a\"],[\"c\"]]"
    );
}

#[test]
fn a_try_now_catches_what_was_a_parse_error() {
    assert_eq!(
        result(&format!(
            "{CHAIN}outcomes = 5\nr = \"none\"\ntry:\n    r = c |> why |> outcomes\ncatch error:\n    r = error\nr"
        )),
        "\"TypeError: outcomes is not an operation\""
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}try:\n    r = c |> why |> origins |> nodes\ncatch e:\n    r = \"caught\"\nr"
        )),
        "[\"a\"]"
    );
    assert_eq!(
        failure(&format!("{CHAIN}missing\norigins = 1\nc |> why |> origins")),
        "ReferenceError: missing is not defined (line 4, column 1)"
    );
    assert_eq!(
        failure(&format!("{CHAIN}origins = 1\nc |> why |> origins\nmissing")),
        "TypeError: origins is not an operation (line 5, column 1)"
    );
}

#[test]
fn a_binding_declared_after_a_standing_declaration_fails_its_refresh() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}ends = a |> affects |> outcomes"),
    );
    checked(&mut runtime, "outcomes = 5");
    assert_eq!(outcome(&mut runtime, &named("ends")), "[\"c\"]");
    assert_eq!(
        outcome(&mut runtime, "d = c + 1"),
        "TypeError: outcomes is not an operation (line 1, column 1)"
    );
    assert_eq!(
        outcome(&mut runtime, "d"),
        "ReferenceError: d is not defined (line 1, column 1)"
    );
    assert_eq!(outcome(&mut runtime, &named("ends")), "[\"c\"]");
    checked(&mut runtime, "delete outcomes\nd = c + 1");
    assert_eq!(outcome(&mut runtime, &named("ends")), "[\"d\"]");
    let mut model = Runtime::new();
    checked(&mut model, &format!("{CHAIN}starts = model |> origins"));
    assert_eq!(
        outcome(&mut model, "origins = 3"),
        "TypeError: origins is not an operation (line 1, column 1)"
    );
    assert_eq!(
        outcome(&mut model, "origins"),
        "ReferenceError: origins is not defined (line 1, column 1)"
    );
    assert_eq!(outcome(&mut model, &named("starts")), "[\"a\"]");
}

#[test]
fn the_backtracking_forms_now_parse_and_templates_keep_their_reading() {
    for (written, expected) in [
        ("x, y = c |> why |> outcomes\n[x.node, y]", "[\"c\",null]"),
        ("{length} = c |> why |> origins\nlength", "1.0"),
        ("x, = c |> why |> origins\nx.node", "\"a\""),
        ("`v ${c why c |> origins}`", "\"v 4\""),
        ("`n ${(c |> why |> origins).length}`", "\"n 1\""),
        (
            "[p] = c |> why |> origins",
            "SyntaxError: Cannot assign to [p] (line 4, column 1)",
        ),
        (
            "delete c |> why |> outcomes",
            "SyntaxError: Cannot delete c|>why|>outcomes (line 4, column 1)",
        ),
        (
            "not c |> why |> origins",
            "TypeError: Cannot explain !c (line 4, column 1)",
        ),
    ] {
        assert_eq!(fresh(&format!("{CHAIN}{written}")), expected, "{written}");
    }
}

#[test]
fn standing_declarations_follow_the_graph_as_their_accepted_spellings_do() {
    let mut compared = 0;
    for (setup, nodes, writes) in GRAPHS {
        for x in nodes {
            for form in ["X |> affects", "X |> why", "model", "X |> why |> derived"] {
                let selection = spelled(form, x, x);
                let mut stage = Runtime::new();
                let mut spelling = Runtime::new();
                checked(&mut stage, setup);
                checked(&mut spelling, setup);
                let declared =
                    format!("starts = {selection} |> origins\nends = {selection} |> outcomes");
                let accepted = format!(
                    "starts = {selection} |> except({selection} |> affects)\nnames = {}",
                    selected(&selection)
                );
                assert_eq!(outcome(&mut stage, &declared), outcome(&mut stage, "ends"));
                outcome(&mut spelling, &accepted);
                for write in writes {
                    let left = outcome(&mut stage, write);
                    let right = outcome(&mut spelling, write);
                    assert_eq!(left, right, "{setup}{selection}\n{write}");
                    assert_eq!(
                        outcome(&mut stage, &named("starts")),
                        outcome(&mut spelling, &named("starts")),
                        "{setup}{selection}\n{write}"
                    );
                    let steps: Option<Vec<String>> =
                        serde_json::from_str(&outcome(&mut spelling, "names")).ok();
                    if let Some(steps) = steps {
                        let (_, last) = ends(&spelling, &steps);
                        assert_eq!(
                            outcome(&mut stage, &named("ends")),
                            json(&last),
                            "{setup}{selection}\n{write}"
                        );
                    }
                    compared += 1;
                }
            }
        }
    }
    assert_eq!(compared, 636);
}

#[test]
fn the_walk_observes_the_structure_of_every_node_it_visits() {
    let mut stage = Runtime::new();
    let mut spelling = Runtime::new();
    checked(&mut stage, INVOICE);
    checked(&mut spelling, INVOICE);
    checked(&mut stage, "starts = subtotal |> why |> origins");
    checked(
        &mut spelling,
        "starts = subtotal |> why |> except(subtotal |> why |> affects)",
    );
    assert_eq!(observes(&stage, "starts"), observes(&spelling, "starts"));
    let shapes = observes(&stage, "starts");
    for key in [
        "subtotal", "price", "quantity", "tax", "total", "unit", "shipping",
    ] {
        assert!(
            shapes.contains(&format!("Node(NodeKey(\"{key}\"))")),
            "{key}: {shapes:?}"
        );
    }
    checked(&mut stage, "ends = quantity |> affects |> outcomes");
    let ends = observes(&stage, "ends");
    for key in ["quantity", "subtotal", "tax", "total", "unit", "shipping"] {
        assert!(
            ends.contains(&format!("Node(NodeKey(\"{key}\"))")),
            "{key}: {ends:?}"
        );
    }
    assert!(
        !ends.contains(&"Node(NodeKey(\"price\"))".to_string()),
        "{ends:?}"
    );
    checked(&mut stage, "grand = unit * 2");
    assert_eq!(
        outcome(&mut stage, &named("ends")),
        "[\"shipping\",\"grand\"]"
    );
}

#[test]
fn declarations_that_hold_the_stages_are_reasoning_observers() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}starts = model |> origins\nends = a |> affects |> outcomes\nd = c + 1"),
    );
    assert_eq!(
        outcome(&mut runtime, "model |> nodes"),
        "[\"a\",\"b\",\"c\",\"d\"]"
    );
    assert_eq!(
        outcome(&mut runtime, "a |> affects |> nodes"),
        "[\"b\",\"c\",\"d\"]"
    );
    assert_eq!(outcome(&mut runtime, "why starts"), "[]");
    assert_eq!(outcome(&mut runtime, "ends |> nodes"), "[]");
    assert_eq!(outcome(&mut runtime, &named("ends")), "[\"d\"]");
    assert_eq!(outcome(&mut runtime, &named("starts")), "[\"a\"]");
    assert_eq!(stored(&runtime, "starts"), "model|>origins");
    assert_eq!(stored(&runtime, "ends"), "a|>affects|>outcomes");
}

#[test]
fn class_level_rules_functions_methods_conditions_and_quantifiers_evaluate_the_stages() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "fee = 2\nclass Order(price, qty):\n    this.price = price\n    this.qty = qty\n    def ends():\n        return (this.price |> affects |> outcomes).map(step -> step.node)\n$Order.subtotal = $Order.price * $Order.qty\n$Order.total = $Order.subtotal + fee\n$Order.inputs = $Order.total |> why |> origins\n$Order.big = any s in $Order.price |> affects |> outcomes: s.holds > 25\nfirst = Order(10, 2)\nsecond = Order(4, 5)",
    );
    assert_eq!(
        outcome(
            &mut runtime,
            "[first.inputs.map(step -> step.node), second.inputs.map(step -> step.node)]"
        ),
        "[[\"fee\",\"first.price\",\"first.qty\"],[\"fee\",\"second.price\",\"second.qty\"]]"
    );
    assert_eq!(
        outcome(&mut runtime, "[first.big, second.big]"),
        "[false,false]"
    );
    assert_eq!(outcome(&mut runtime, "first.ends()"), "[\"first.total\"]");
    checked(&mut runtime, "first.qty = 3\n$Order.due = $Order.total * 2");
    assert_eq!(
        outcome(&mut runtime, "[first.big, second.big, first.ends()]"),
        "[true,true,[\"first.due\"]]"
    );
    checked(
        &mut runtime,
        "def roots() => (fee |> affects |> origins).map(step -> step.node)\nif (fee |> affects |> outcomes).length > 1:\n    flag = true",
    );
    assert_eq!(
        outcome(&mut runtime, "[roots(), flag]"),
        "[[\"first.total\",\"second.total\"],true]"
    );
    assert_eq!(
        outcome(&mut runtime, "first.total |> why |> stated |> nodes"),
        "[\"fee\"]"
    );
    assert_eq!(
        outcome(
            &mut runtime,
            "all s in fee |> affects |> outcomes: s.state == \"derived\""
        ),
        "true"
    );
}

#[test]
fn value_snapshots_suppose_and_rollback_behave_as_for_the_accepted_stages() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}ends = a |> affects |> outcomes\nsnap = ends.value"),
    );
    checked(&mut runtime, "d = c + 1");
    assert_eq!(
        outcome(
            &mut runtime,
            &format!("[{}, {}]", named("ends"), named("snap"))
        ),
        "[[\"d\"],[\"c\"]]"
    );
    assert_eq!(
        outcome(
            &mut runtime,
            "suppose e = d * 2:\n    ends.map(step => step.node)"
        ),
        "[\"e\"]"
    );
    assert_eq!(outcome(&mut runtime, &named("ends")), "[\"d\"]");
    let rolled = outcome(
        &mut runtime,
        "try:\n    e = d * 2\n    assert(ends.map(step => step.node), [\"e\"])\n    throw \"UNDO\"\ncatch error:\n    caught = error\ncaught",
    );
    assert_eq!(rolled, "\"UNDO\"");
    assert_eq!(outcome(&mut runtime, &named("ends")), "[\"d\"]");
    assert!(runtime.take_assertions().is_empty());
    assert_eq!(
        outcome(&mut runtime, "ends = 1 |> origins"),
        "TypeError: Cannot explain 1 (line 1, column 1)"
    );
    assert_eq!(outcome(&mut runtime, &named("ends")), "[\"d\"]");
}

#[test]
fn rendering_shows_the_stages_as_written() {
    for (source, rendered) in [
        ("c |> why |> origins", "c|>why|>origins"),
        ("why c |> stated |> outcomes", "c|>why|>stated|>outcomes"),
        ("model |> origins", "model|>origins"),
        (
            "c |> why |> intersect(d |> why |> outcomes)",
            "c|>why|>intersect(d|>why|>outcomes)",
        ),
        (
            "c |> why |> origins |> depth(1) |> outcomes",
            "c|>why|>origins|>depth(1)|>outcomes",
        ),
        ("(c |> why |> origins).length", "c|>why|>origins.length"),
    ] {
        let parsed = parse_expression(source).unwrap();
        assert_eq!(parsed.to_string(), rendered, "{source}");
    }
    for source in [
        "c |> why |> origins",
        "c |> outcomes",
        "c |> why |> depth(1) |> origins",
        "c |> why |> origins |> why",
        "model |> outcomes |> nodes",
    ] {
        let parsed = parse_expression(source).unwrap();
        assert_eq!(
            parse_expression(&parsed.to_string()).unwrap(),
            parsed,
            "{source}"
        );
    }
    assert_eq!(
        generate(&parse("def f(x = 1) => c |> why |> outcomes").unwrap()[0]),
        "def f(x=1):\n    return c|>why|>outcomes"
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}starts = c |> why |> origins"),
    );
    assert_eq!(stored(&runtime, "starts"), "c|>why|>origins");
    assert_eq!(
        failure(&format!("{CHAIN}delete c |> why |> origins")),
        "SyntaxError: Cannot delete c|>why|>origins (line 4, column 1)"
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-origins-and-outcomes-{label}-{}.nuc",
            std::process::id()
        ));
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        Self(path)
    }

    fn run(&self, source: &str, options: &[&str]) -> Output {
        std::fs::write(&self.0, source).unwrap();
        Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .args(options)
            .arg(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for SourceFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).unwrap();
    }
}

fn repl(source: &str, options: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(options)
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

const PROGRAM: &str = "price = 10\nqty = 3\nsubtotal = price * qty\nunit = subtotal / qty\nstarts = qty |> affects |> origins\nends = qty |> affects |> outcomes\nshipping = qty * 2\n[starts |> nodes, ends.map(step => step.node), model |> outcomes |> nodes]";
const PROGRAM_SPELLED: &str = "price = 10\nqty = 3\nsubtotal = price * qty\nunit = subtotal / qty\nstarts = affects qty |> except(affects affects qty)\nends = [s for s in affects qty if not any t in affects qty: s.node in t.from]\nshipping = qty * 2\n[starts |> nodes, ends.map(step => step.node), [s for s in model |> why if not any t in model |> why: s.node in t.from].map(step => step.node)]";

#[test]
fn files_and_repl_sessions_give_the_stages_results() {
    let file = SourceFile::new("file");
    for options in [&[][..], &["--json"][..]] {
        let stage = file.run(PROGRAM, options);
        let spelling = file.run(PROGRAM_SPELLED, options);
        assert!(stage.status.success(), "{options:?} {stage:?}");
        assert!(stage.stderr.is_empty(), "{options:?} {stage:?}");
        assert_eq!(stage.stdout, spelling.stdout, "{options:?}");
    }
    assert_eq!(
        String::from_utf8(file.run(PROGRAM, &["--json"]).stdout).unwrap(),
        "[[],[\"unit\",\"shipping\"],[\"unit\",\"shipping\"]]\n"
    );
    let graph = String::from_utf8(file.run(PROGRAM, &["--graph"]).stdout).unwrap();
    assert!(graph.contains("  ends [variable]"), "{graph}");
    assert!(graph.contains("observes"), "{graph}");
    for (source, message) in [
        (
            "a = 1\norigins = 2\na |> origins\n",
            "TypeError: origins is not an operation (line 3, column 1)\n",
        ),
        (
            "a = 1\na |> why |> path(a |> outcomes)\n",
            "TypeError: Cannot explain a|>outcomes (line 2, column 1)\n",
        ),
    ] {
        let failed = file.run(source, &["--json"]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(String::from_utf8(failed.stderr).unwrap(), message);
    }
    let session = "a = 1\nb = a + 2\nc = b * 2\nc |> why |> origins |> nodes\nends = a |> affects |> outcomes\nends |> nodes\nd = c + 1\nends.map(step => step.node)\n.exit\n";
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let stage = repl(&session.replace('\n', newline), options);
            assert!(stage.status.success(), "{options:?} {stage:?}");
            assert!(stage.stderr.is_empty(), "{options:?} {stage:?}");
            let stdout = String::from_utf8(stage.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "[\"a\"]\n"
            } else {
                "a\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
            assert!(stdout.contains("d"), "{options:?} {stdout}");
        }
    }
}

#[test]
fn long_chains_of_end_stages_evaluate_without_recursing() {
    let stages = " |> origins |> outcomes".repeat(60);
    assert_eq!(
        result(&format!("{CHAIN}{}", named(&format!("c |> why{stages}")))),
        "[\"a\"]"
    );
    assert_eq!(
        result(&format!("{CHAIN}v = c |> why{stages}\n{}", named("v"))),
        "[\"a\"]"
    );
    for stage in ["stated", "origins", "outcomes"] {
        let chained = format!(" |> {stage}").repeat(1000);
        let error = failure(&format!("{CHAIN}c |> why{chained}"));
        assert!(
            error.starts_with("SyntaxError: Expression has too many operands (line 4, column "),
            "{stage}: {error}"
        );
    }
}

#[test]
fn the_compatibility_rows_give_their_documented_results() {
    let six = "a = 1\nb = a + 1\nc = b * 2\nd = a + 3\ne = 7\nf = e + d\n";
    for (written, expected) in [
        (
            "c |> why |> Origins",
            "TypeError: Origins is not an operation",
        ),
        (
            "c |> why |> outcome",
            "TypeError: outcome is not an operation",
        ),
        (
            "origins c",
            "ReferenceError: origins is not defined (line 7, column 1)",
        ),
        ("outcomes = [1, 2]\noutcomes", "[1.0,2.0]"),
        ("x = {origins: 1}\nx.origins", "1.0"),
        (
            "class Tank(outcomes): this.outcomes = outcomes\nt = Tank(2)\nt.outcomes",
            "2.0",
        ),
        ("(c |> why).origins", "null"),
        ("`v ${c why c |> origins}`", "\"v 4\""),
        (
            "(c |> why |> except(c |> why |> affects)).map(step => step.node)",
            "[\"a\"]",
        ),
        ("c |> why |> origins |> nodes", "[\"a\"]"),
        ("c |> why |> outcomes |> nodes", "[\"c\"]"),
        ("model |> origins |> nodes", "[\"a\",\"e\"]"),
        ("model |> outcomes |> nodes", "[\"c\",\"f\"]"),
        ("a |> affects |> origins |> nodes", "[\"b\",\"d\"]"),
        ("a |> affects |> outcomes |> nodes", "[\"f\",\"c\"]"),
        (
            "f |> why |> union(c |> why) |> outcomes |> nodes",
            "[\"f\",\"c\"]",
        ),
        ("f |> why |> except(d) |> outcomes |> nodes", "[\"f\"]"),
        ("f |> why |> except(d) |> origins |> nodes", "[\"e\",\"a\"]"),
        (
            "vals = a |> affects |> outcomes\ng = c + 1\nvals |> nodes",
            "[]",
        ),
        (
            "vals = a |> affects |> outcomes\ng = c + 1\nvals.map(step => step.node)",
            "[\"f\",\"g\"]",
        ),
    ] {
        assert_eq!(fresh(&format!("{six}{written}")), expected, "{written}");
    }
}
