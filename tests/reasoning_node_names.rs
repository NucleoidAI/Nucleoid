mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::Runtime;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, parse, parse_expression};
use nucleoid::nuc::Nuc;
use nucleoid::reasoning::Stage;

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid names the nodes of an explanation with nodes",
    "Nucleoid names what a change reaches and the model with nodes",
    "Nucleoid names the nodes behind class level rules with nodes",
    "Nucleoid rolls back failed updates through node names",
];
const REFERENCE_TITLE: &str = "Nucleoid names the nodes of selections with nodes";
const README_TITLE: &str = "Nodes names the facts a heater rests on and what insulation moves";
const EXAMPLES_TITLE: &str = "Nodes names the facts and rules behind a loan decision";

const CHAIN: &str = "a = 1\nb = a + 2\nc = b * 2\n";

type Rewrite = (&'static str, &'static str);

struct Counterpart {
    title: &'static str,
    rewrites: &'static [Rewrite],
    before: usize,
    after: usize,
}

const SPELLED: [Counterpart; 7] = [
    Counterpart {
        title: "Nucleoid names the nodes of an explanation with nodes",
        rewrites: &[
            ("price = 10", "n = s -> s.map(.node)\nprice = 10"),
            ("total |> why |> nodes", "n(why total)"),
            ("total |> why |> stated |> nodes", "n(why total |> stated)"),
            (
                "total |> why |> derived |> nodes",
                "n(why total |> derived)",
            ),
        ],
        before: 149,
        after: 137,
    },
    Counterpart {
        title: "Nucleoid names what a change reaches and the model with nodes",
        rewrites: &[
            ("price |> affects |> nodes", "(affects price).map(.node)"),
            ("model |> nodes", "(model |> why).map(.node)"),
        ],
        before: 155,
        after: 142,
    },
    Counterpart {
        title: "Nucleoid names the nodes behind class level rules with nodes",
        rewrites: &[
            ("fee = 2", "n = s -> s.map(.node)\nfee = 2"),
            ("$Order.total |> why |> nodes", "n(why $Order.total)"),
            ("fee |> affects |> nodes", "n(affects fee)"),
            (
                "first.total |> why |> stated |> nodes",
                "n(why first.total |> stated)",
            ),
        ],
        before: 172,
        after: 160,
    },
    Counterpart {
        title: "Nucleoid rolls back failed updates through node names",
        rewrites: &[
            ("price = 10", "n = s -> s.map(.node)\nprice = 10"),
            ("subtotal |> why |> nodes", "n(why subtotal)"),
            ("1 |> why |> nodes", "n(1 |> why)"),
            (
                "subtotal |> why |> stated |> nodes",
                "n(why subtotal |> stated)",
            ),
        ],
        before: 161,
        after: 148,
    },
    Counterpart {
        title: "Nucleoid names the nodes of selections with nodes",
        rewrites: &[
            (
                "total |> why |> stated |> nodes",
                "(why total |> stated).map(.node)",
            ),
            ("price |> affects |> nodes", "(affects price).map(.node)"),
        ],
        before: 87,
        after: 77,
    },
    Counterpart {
        title: "Nodes names the facts a heater rests on and what insulation moves",
        rewrites: &[
            (
                "heater |> why |> stated |> nodes",
                "(why heater |> stated).map(.node)",
            ),
            (
                "insulation |> affects |> nodes",
                "(affects insulation).map(.node)",
            ),
        ],
        before: 106,
        after: 96,
    },
    Counterpart {
        title: "Nodes names the facts and rules behind a loan decision",
        rewrites: &[
            ("income = 5000", "n = s -> s.map(.node)\nincome = 5000"),
            (
                "approved |> why |> stated |> nodes",
                "n(why approved |> stated)",
            ),
            (
                "approved |> why |> derived |> nodes",
                "n(why approved |> derived)",
            ),
            ("income |> affects |> nodes", "n(affects income)"),
        ],
        before: 127,
        after: 115,
    },
];

const FORMS: [(&str, &str); 20] = [
    ("X |> nodes", "X |> depth(0)"),
    ("X |> why |> nodes", "X |> why"),
    ("why X |> nodes", "why X"),
    ("X |> affects |> nodes", "X |> affects"),
    ("affects X |> nodes", "affects X"),
    ("X |> why |> stated |> nodes", "X |> why |> stated"),
    ("X |> why |> derived |> nodes", "X |> why |> derived"),
    ("stated X |> nodes", "stated X"),
    ("derived X |> nodes", "derived X"),
    ("X |> why |> depth(1) |> nodes", "X |> why |> depth(1)"),
    (
        "X |> affects |> depth(1) |> nodes",
        "X |> affects |> depth(1)",
    ),
    ("X |> why |> path(Y) |> nodes", "X |> why |> path(Y)"),
    (
        "Y |> affects |> path(X) |> nodes",
        "Y |> affects |> path(X)",
    ),
    (
        "X |> why |> intersect(Y |> why) |> nodes",
        "X |> why |> intersect(Y |> why)",
    ),
    (
        "X |> why |> except(Y |> why) |> nodes",
        "X |> why |> except(Y |> why)",
    ),
    (
        "X |> why |> stated |> except(Y |> why |> intersect(Y |> why)) |> nodes",
        "X |> why |> stated |> except(Y |> why |> intersect(Y |> why))",
    ),
    (
        "X |> why |> depth(null) |> nodes",
        "X |> why |> depth(null)",
    ),
    ("model |> nodes", "model |> depth(0)"),
    ("model |> stated |> nodes", "model |> stated"),
    (
        "model |> intersect(X |> why) |> nodes",
        "model |> intersect(X |> why)",
    ),
];

const GRAPHS: [(&str, &[&str], &[&str]); 5] = [
    (
        "a = 1\nb = a + 2\nc = b * 2\n",
        &["a", "b", "c"],
        &["a = 4", "x = 7\nb = x + 1", "b = a + 3", "c = a * 5"],
    ),
    (
        "a = 1\nb = a + 1\nc = a * 2\nd = b + c\ne = d + b\n",
        &["a", "b", "d", "e"],
        &["a = 2", "c = b * 3", "f = e + 1", "delete f"],
    ),
    (
        "rate = 2\nclass M(load):\n    this.load = load\n$M.wear = $M.load * rate\nm = M(3)\nk = M(4)\nt = m.wear + k.wear\n",
        &["rate", "m.wear", "k.load", "t"],
        &[
            "rate = 3",
            "j = M(5)",
            "$M.wear = $M.load * rate + 1",
            "m.load = 9",
        ],
    ),
    (
        "a = 1\nb = a + 1\nc = b + 1\nd = 4\ndelete c\n",
        &["a", "b", "c", "d"],
        &["c = b * 2", "a = 3", "delete c"],
    ),
    ("", &["x"], &["x = 1", "y = x + 1", "delete y"]),
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

fn writes_nodes(source: &str) -> bool {
    tokenize(source).unwrap().windows(2).any(|window| {
        window[0].token == Token::Pipe
            && matches!(&window[1].token, Token::Identifier(name) if name == "nodes")
    })
}

fn tokens(source: &str) -> usize {
    tokenize(source.trim())
        .unwrap()
        .iter()
        .filter(|spanned| spanned.token != Token::Eof)
        .count()
}

fn renumbered(text: &str) -> String {
    let mut seen: Vec<&str> = Vec::new();
    let mut result = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("obj:") {
        let digits = after
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(after.len());
        let (identifier, tail) = after.split_at(digits);
        let ordinal = match seen.iter().position(|known| *known == identifier) {
            Some(ordinal) => ordinal,
            None => {
                seen.push(identifier);
                seen.len() - 1
            }
        };
        result.push_str(before);
        result.push_str("obj:");
        result.push_str(&ordinal.to_string());
        rest = tail;
    }
    result.push_str(rest);
    result
}

fn outcome(runtime: &mut Runtime, source: &str) -> String {
    match runtime.run(source) {
        Ok(value) => renumbered(&runtime.serialize_json(&value).unwrap()),
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
    renumbered(&runtime.serialize_json(&value).unwrap())
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
            title: "node names regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn accepted(selection: &str) -> String {
    format!("({selection}).map(step => step.node)")
}

fn spelled(form: &str, x: &str, y: &str) -> String {
    form.replace('X', x).replace('Y', y)
}

fn reads(runtime: &Runtime, name: &str) -> Vec<String> {
    let mut keys: Vec<String> = runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .map(|node| {
            node.dependencies
                .iter()
                .map(ToString::to_string)
                .filter(|key| !key.starts_with("obj:"))
                .collect()
        })
        .unwrap_or_default();
    keys.sort();
    keys
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

fn pending(runtime: &Runtime) -> usize {
    runtime
        .graph
        .keys()
        .filter(|key| key.to_string().starts_with("obj:"))
        .count()
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

fn behaves_as_spelled(steps: &[(&str, &str)]) {
    let mut stage = Runtime::new();
    let mut spelling = Runtime::new();
    for (written, accepted) in steps {
        assert_eq!(
            outcome(&mut stage, written),
            outcome(&mut spelling, accepted),
            "{written}"
        );
        for name in ["names", "facts", "inner"] {
            assert_eq!(reads(&stage, name), reads(&spelling, name), "{written}");
            assert_eq!(
                observes(&stage, name),
                observes(&spelling, name),
                "{written}"
            );
        }
    }
    assert_eq!(stage.assertions_run(), spelling.assertions_run());
    assert!(stage.take_assertions().is_empty(), "{steps:?}");
    assert!(spelling.take_assertions().is_empty(), "{steps:?}");
}

#[test]
fn only_the_appended_cases_write_nodes_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(writes_nodes(&case.source), new, "{}", case.title);
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
    assert_eq!(assertions, 26);
    assert_eq!(cases, 1280);
}

#[test]
fn the_audit_finds_every_nodes_stage_and_nothing_else() {
    for source in [
        "c |> nodes",
        "c |> why |> nodes",
        "why c |> stated |> nodes",
        "x = model|>nodes",
        "f(c |> why |> nodes)",
        "$P.basis = $P.v |> why |> nodes",
        "c |> why |> nodes |> why",
        "c |> why |> nodes(1)",
    ] {
        assert!(writes_nodes(source), "{source}");
    }
    for source in [
        "nodes",
        "nodes c",
        "nodes(c |> why)",
        "nodes = [1, 2]",
        "o = {nodes: 1}",
        "o.nodes",
        "def nodes(x) => x",
        "c |> why |> Nodes",
        "c |> why |> node",
        "c |> why |>\nnodes",
        "\"|> nodes\"",
        "(why c).map(.node)",
    ] {
        assert!(!writes_nodes(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_111_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[768],
        "Nucleoid rolls back failed updates through padded strings"
    );
    assert_eq!(titles[769..773], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid pads strings with padStart and padEnd",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Padding keeps a price list aligned as it changes",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Padded labels follow every bin's aisle and slot",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 23 - usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 18].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Node names of reasoning selections\n\n```\n")
        .expect("the language reference documents the nodes stage");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(
        case.expected.as_deref(),
        Some("[\"base\",\"price\",\"qty\"]")
    );
    for line in [
        "[NUC 112](nuc-0112.md)",
        "reference |> why |> nodes             the node names of an explanation, in its order (NUC 112)",
        "model |> nodes                        the names of the current model's nodes (NUC 112)",
        "selection |> nodes                    the list (selection).map(step => step.node); ends the pipeline (NUC 112)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
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
        assert!(writes_nodes(&case.source), "{title}");
        assert!(!writes_nodes(&counterpart), "{title}");
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
fn nodes_parses_as_a_final_stage_and_stays_an_ordinary_name() {
    let why_c = Expr::Reason {
        stage: Stage::Why,
        source: Box::new(Expr::Identifier("c".to_string())),
    };
    let names = Expr::Reason {
        stage: Stage::Nodes,
        source: Box::new(why_c.clone()),
    };
    assert_eq!(parse_expression("c |> why |> nodes").unwrap(), names);
    assert_eq!(parse_expression("why c |> nodes").unwrap(), names);
    assert_eq!(parse_expression("c|>why|>nodes").unwrap(), names);
    assert_eq!(
        parse_expression("c |> nodes").unwrap(),
        Expr::Reason {
            stage: Stage::Nodes,
            source: Box::new(Expr::Identifier("c".to_string())),
        }
    );
    assert_eq!(
        parse_expression("model |> nodes").unwrap(),
        Expr::Reason {
            stage: Stage::Nodes,
            source: Box::new(Expr::Model),
        }
    );
    assert_eq!(
        parse_expression("c |> why |> nodes |> why").unwrap(),
        Expr::Reason {
            stage: Stage::Why,
            source: Box::new(names),
        }
    );
    assert_eq!(Stage::Nodes.as_str(), "nodes");
    assert_eq!(Stage::from_name("nodes"), None);
    assert_eq!(Stage::from_name("why"), Some(Stage::Why));
    for (source, expected) in [
        (
            format!("{CHAIN}nodes c"),
            "ReferenceError: nodes is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}nodes"),
            "ReferenceError: nodes is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}nodes(c |> why)"),
            "ReferenceError: nodes is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> Nodes"),
            "TypeError: Nodes is not an operation",
        ),
        (
            format!("{CHAIN}c |> why |> node"),
            "TypeError: node is not an operation",
        ),
        (
            format!("{CHAIN}c |> why |>\nnodes"),
            "SyntaxError: Expected an operation (line 4, column 10)",
        ),
        (format!("{CHAIN}nodes = [1, 2]\nnodes"), "[1.0,2.0]"),
        (format!("{CHAIN}x = {{nodes: 1}}\nx.nodes"), "1.0"),
        (format!("{CHAIN}def nodes(x) => x * 2\nnodes(3)"), "6.0"),
        (format!("{CHAIN}(c |> why).nodes"), "null"),
    ] {
        assert_eq!(fresh(&source), expected, "{source}");
    }
}

#[test]
fn every_source_and_stage_names_what_its_accepted_spelling_names() {
    let mut compared = 0;
    for (setup, names, _) in GRAPHS {
        for x in names {
            for y in names {
                for (form, selection) in FORMS {
                    let written = format!("{setup}{}", spelled(form, x, y));
                    let spelling = format!("{setup}{}", accepted(&spelled(selection, x, y)));
                    assert_eq!(fresh(&written), fresh(&spelling), "{written}");
                    compared += 1;
                }
            }
        }
    }
    assert_eq!(compared, 1160);
    assert_eq!(
        result(&format!(
            "{CHAIN}[c |> why |> nodes, model |> nodes, a |> affects |> nodes]"
        )),
        "[[\"c\",\"b\",\"a\"],[\"a\",\"b\",\"c\"],[\"b\",\"c\"]]"
    );
    assert_eq!(result("model |> nodes"), "[]");
    assert_eq!(result(&format!("{CHAIN}a |> derived |> nodes")), "[]");
}

#[test]
fn a_reference_alone_names_its_own_node() {
    assert_eq!(result(&format!("{CHAIN}c |> nodes")), "[\"c\"]");
    assert_eq!(
        result(&format!("{CHAIN}{}", accepted("c |> why |> depth(0)"))),
        "[\"c\"]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}[a |> nodes, a |> stated |> nodes, c |> stated |> nodes, c |> derived |> nodes]"
        )),
        "[[\"a\"],[\"a\"],[],[\"c\"]]"
    );
    assert_eq!(
        failure(&format!("{CHAIN}(c).map(step => step.node)")),
        "TypeError: number.map is not a function (line 4, column 1)"
    );
    assert_eq!(
        failure(&format!("{CHAIN}missing |> nodes")),
        "ReferenceError: missing is not defined (line 4, column 1)"
    );
}

#[test]
fn stages_after_nodes_and_nodes_arguments_raise_their_accepted_errors() {
    let prelude = format!("{CHAIN}d = a + 10\n");
    for (written, spelling, expected, spelled_error) in [
        (
            "c |> why |> nodes |> why",
            "(c |> why).map(step => step.node) |> why",
            "TypeError: Cannot explain c|>why|>nodes (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> nodes |> depth(1)",
            "(c |> why).map(step => step.node) |> depth(1)",
            "TypeError: Cannot explain c|>why|>nodes (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> nodes |> nodes",
            "(c |> why).map(step => step.node) |> depth(0)",
            "TypeError: Cannot explain c|>why|>nodes (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> nodes |> affects |> path(a)",
            "(c |> why).map(step => step.node) |> affects |> path(a)",
            "TypeError: Cannot explain c|>why|>nodes (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> nodes |> intersect(c |> why)",
            "(c |> why).map(step => step.node) |> intersect(c |> why)",
            "TypeError: Cannot explain c|>why|>nodes (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> intersect(d |> why |> nodes)",
            "c |> why |> intersect((d |> why).map(step => step.node))",
            "TypeError: Cannot explain d|>why|>nodes (line 5, column 1)",
            "TypeError: Cannot explain d|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> except(d |> why |> nodes)",
            "c |> why |> except((d |> why).map(step => step.node))",
            "TypeError: Cannot explain d|>why|>nodes (line 5, column 1)",
            "TypeError: Cannot explain d|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> path(a |> why |> nodes)",
            "c |> why |> path((a |> why).map(step => step.node))",
            "TypeError: Cannot explain a|>why|>nodes (line 5, column 1)",
            "TypeError: Cannot explain a|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> depth(m.value) |> nodes |> why",
            "(c |> why |> depth(m.value)).map(step => step.node) |> why",
            "TypeError: Cannot explain c|>why|>depth(m.value)|>nodes (line 5, column 1)",
            "TypeError: Cannot explain c|>why|>depth(m.value).map(function()) (line 5, column 1)",
        ),
    ] {
        assert_eq!(
            failure(&format!("{prelude}{written}")),
            expected,
            "{written}"
        );
        assert_eq!(
            failure(&format!("{prelude}{spelling}")),
            spelled_error,
            "{spelling}"
        );
    }
    assert_eq!(
        failure(&format!("{prelude}stated = 1\nc |> why |> nodes |> stated")),
        "TypeError: stated is not an operation (line 6, column 1)"
    );
    assert_eq!(
        result(&format!(
            "{prelude}other = d |> why |> nodes\n[c |> why |> intersect(other) |> nodes, c |> why |> except(other) |> nodes]"
        )),
        "[[],[\"c\",\"b\",\"a\"]]"
    );
    assert_eq!(
        result(&format!(
            "{prelude}other = (d |> why).map(step => step.node)\n[(c |> why |> intersect(other)).map(step => step.node), (c |> why |> except(other)).map(step => step.node)]"
        )),
        "[[],[\"c\",\"b\",\"a\"]]"
    );
}

#[test]
fn every_binding_named_nodes_shadows_the_stage_at_run_time() {
    let bindings = [
        ("nodes = 1\nc |> why |> nodes", 5),
        ("def nodes(x) => x\nc |> why |> nodes", 5),
        ("class nodes\nc |> why |> nodes", 5),
        ("nodes = 1\nmissing |> why |> nodes", 5),
        ("nodes = 1\n1 |> nodes", 5),
        ("stated = 1\nnodes = 2\nc |> why |> stated |> nodes", 6),
        ("def f(nodes) => c |> why |> nodes\nf(1)", 5),
        ("def g nodes => c |> why |> nodes\ng(1)", 5),
        ("f = nodes -> c |> why |> nodes\nf(1)", 5),
        ("f = (nodes) => c |> why |> nodes\nf(1)", 5),
        ("f = lambda nodes: c |> why |> nodes\nf(1)", 5),
        ("[1].map(nodes -> c |> why |> nodes)", 4),
        ("for nodes of \"x\":\n    y = c |> why |> nodes", 4),
        ("ok = all nodes in [1]: (c |> why |> nodes).length > 0", 4),
        ("{\n    nodes = 5\n    y = c |> why |> nodes\n}", 4),
        (
            "class R:\n    def m(nodes) => c |> why |> nodes\nR().m(1)",
            6,
        ),
        (
            "class Q(nodes):\n    this.v = (c |> why |> nodes).length\nQ(1)",
            6,
        ),
        (
            "try:\n    throw 1\ncatch nodes:\n    y = c |> why |> nodes",
            4,
        ),
        ("{nodes} = {nodes: 1}\nc |> why |> nodes", 5),
    ];
    for (binding, line) in bindings {
        assert_eq!(
            failure(&format!("{CHAIN}{binding}")),
            format!("TypeError: nodes is not an operation (line {line}, column 1)"),
            "{binding}"
        );
    }
    assert_eq!(
        failure(&format!("{CHAIN}stated = 1\nc |> why |> stated |> nodes")),
        "TypeError: stated is not an operation (line 5, column 1)"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}class K:\n    def nodes() => 1\no = {{nodes: 1}}\nq = {{}}\nq.nodes = 2\n[c |> why |> nodes, o.nodes, q.nodes, K().nodes()]"
        )),
        "[[\"c\",\"b\",\"a\"],1.0,2.0,1.0]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}why = 1\naffects = 2\n[c |> why |> nodes, a |> affects |> nodes]"
        )),
        "[[\"c\",\"b\",\"a\"],[\"b\",\"c\"]]"
    );
    assert_eq!(
        result(&format!("{CHAIN}nodes = 1\n{}", accepted("c |> why"))),
        "[\"c\",\"b\",\"a\"]"
    );
}

#[test]
fn a_try_now_catches_what_was_a_parse_error() {
    assert_eq!(
        result(&format!(
            "{CHAIN}nodes = 5\nr = \"none\"\ntry:\n    r = c |> why |> nodes\ncatch error:\n    r = error\nr"
        )),
        "\"TypeError: nodes is not an operation\""
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}try:\n    r = c |> why |> nodes\ncatch e:\n    r = \"caught\"\nr"
        )),
        "[\"c\",\"b\",\"a\"]"
    );
    assert_eq!(
        failure(&format!("{CHAIN}missing\nnodes = 1\nc |> why |> nodes")),
        "ReferenceError: missing is not defined (line 4, column 1)"
    );
    assert_eq!(
        failure(&format!("{CHAIN}nodes = 1\nc |> why |> nodes\nmissing")),
        "TypeError: nodes is not an operation (line 5, column 1)"
    );
    let session = repl(
        "x = 5\nnodes = 1\nx |> nodes\nx\n.exit\n",
        &["--plain", "--json"],
    );
    assert!(session.status.success(), "{session:?}");
    assert_eq!(
        String::from_utf8(session.stderr).unwrap(),
        "TypeError: nodes is not an operation (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(session.stdout).unwrap();
    assert!(stdout.ends_with("5.0\n1.0\n5.0\n"), "{stdout}");
}

#[test]
fn a_binding_declared_after_a_standing_declaration_fails_its_refresh() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{CHAIN}names = c |> why |> nodes"));
    checked(&mut runtime, "nodes = 5");
    assert_eq!(outcome(&mut runtime, "names"), "[\"c\",\"b\",\"a\"]");
    assert_eq!(
        outcome(&mut runtime, "a = 3"),
        "TypeError: nodes is not an operation (line 1, column 1)"
    );
    assert_eq!(outcome(&mut runtime, "a"), "1.0");
    checked(&mut runtime, "delete nodes\nx = 3\nb = x + 1");
    assert_eq!(outcome(&mut runtime, "names"), "[\"c\",\"b\",\"x\"]");
}

#[test]
fn the_backtracking_forms_now_destructure_the_names() {
    for (written, expected) in [
        ("x, y = c |> why |> nodes\n[x, y]", "[\"c\",\"b\"]"),
        (
            "x, y = [c |> why |> nodes, 1]\n[x, y]",
            "[[\"c\",\"b\",\"a\"],1.0]",
        ),
        ("{length} = c |> why |> nodes\nlength", "3.0"),
        ("{x, y} = c |> why |> nodes\n[x, y]", "[null,null]"),
        ("x, = c |> why |> nodes\nx", "\"c\""),
    ] {
        let spelling = written.replace("c |> why |> nodes", &accepted("c |> why"));
        assert_eq!(result(&format!("{CHAIN}{written}")), expected, "{written}");
        assert_eq!(
            result(&format!("{CHAIN}{spelling}")),
            expected,
            "{spelling}"
        );
    }
    behaves_as_spelled(&[
        (
            "a = 1\nb = a + 2\nc = b * 2\nx, y = c |> why |> nodes\n[x, y]",
            "a = 1\nb = a + 2\nc = b * 2\nx, y = (c |> why).map(step => step.node)\n[x, y]",
        ),
        ("z = 9\nb = z + 1\n[x, y]", "z = 9\nb = z + 1\n[x, y]"),
    ]);
}

#[test]
fn standing_declarations_follow_the_graph_as_their_accepted_spelling_does() {
    let mut compared = 0;
    for (setup, names, writes) in GRAPHS {
        for x in names {
            for (form, selection) in FORMS {
                let y = names[0];
                let written = format!("{setup}names = {}\nnames", spelled(form, x, y));
                let spelling = format!(
                    "{setup}names = {}\nnames",
                    accepted(&spelled(selection, x, y))
                );
                let mut steps = vec![(written, spelling)];
                for write in writes {
                    steps.push((format!("{write}\nnames"), format!("{write}\nnames")));
                }
                let borrowed: Vec<(&str, &str)> = steps
                    .iter()
                    .map(|(written, spelling)| (written.as_str(), spelling.as_str()))
                    .collect();
                behaves_as_spelled(&borrowed);
                compared += 1;
            }
        }
    }
    assert_eq!(compared, 320);
}

#[test]
fn value_snapshots_suppose_and_rollback_behave_as_the_accepted_spelling() {
    behaves_as_spelled(&[
        (
            "a = 1\nb = a + 2\nc = b * 2\nnames = c |> why |> nodes\nsnap = names.value\nx = 5\nb = x + 1\n[names, snap]",
            "a = 1\nb = a + 2\nc = b * 2\nnames = (c |> why).map(step => step.node)\nsnap = names.value\nx = 5\nb = x + 1\n[names, snap]",
        ),
        (
            "frozen = (c |> why |> stated |> nodes).value\nb = a + x\n[frozen, names]",
            "frozen = (c |> why |> stated).map(step => step.node).value\nb = a + x\n[frozen, names]",
        ),
        (
            "suppose a = 5:\n    inner = c |> why |> nodes\n    [inner, c |> why |> diff |> nodes]",
            "suppose a = 5:\n    inner = (c |> why).map(step => step.node)\n    [inner, (c |> why |> diff).map(step => step.node)]",
        ),
        ("names", "names"),
        (
            "try:\n    y = 4\n    b = y + a\n    assert(names, [\"c\", \"b\", \"y\", \"a\"])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nnames",
            "try:\n    y = 4\n    b = y + a\n    assert(names, [\"c\", \"b\", \"y\", \"a\"])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nnames",
        ),
        ("b = a + missing", "b = a + missing"),
        ("names", "names"),
        (
            "facts = c |> why |> stated |> nodes\ntry:\n    facts = 1 |> why |> nodes\ncatch error:\n    failure = error\n[failure, facts]",
            "facts = (c |> why |> stated).map(step => step.node)\ntry:\n    facts = (1 |> why).map(step => step.node)\ncatch error:\n    failure = error\n[failure, facts]",
        ),
    ]);
}

#[test]
fn declarations_that_hold_the_stage_are_reasoning_observers() {
    behaves_as_spelled(&[
        (
            "a = 1\nb = a + 2\nc = b * 2\nnames = c |> why |> nodes\n[names |> why |> nodes, a |> affects |> nodes, model |> nodes]",
            "a = 1\nb = a + 2\nc = b * 2\nnames = (c |> why).map(step => step.node)\n[(names |> why).map(step => step.node), (a |> affects).map(step => step.node), (model |> depth(0)).map(step => step.node)]",
        ),
        (
            "if (c |> why |> nodes).length > 2:\n    flag = true\n[flag, model |> nodes]",
            "if (c |> why).map(step => step.node).length > 2:\n    flag = true\n[flag, (model |> depth(0)).map(step => step.node)]",
        ),
        (
            "class P(v):\n    this.v = v\n$P.basis = $P.v |> why |> nodes\np = P(1)\n[model |> nodes, p.basis]",
            "class P(v):\n    this.v = v\n$P.basis = ($P.v |> why).map(step => step.node)\np = P(1)\n[(model |> depth(0)).map(step => step.node), p.basis]",
        ),
        (
            "def f() => c |> why |> nodes\nlist = f()\n[list, f()]",
            "def f() => (c |> why).map(step => step.node)\nlist = f()\n[list, f()]",
        ),
        (
            "x = 8\nb = x + 1\n[list, f()]",
            "x = 8\nb = x + 1\n[list, f()]",
        ),
    ]);
    assert_eq!(
        result(&format!(
            "{CHAIN}names = c |> why |> nodes\n[names |> why |> nodes, a |> affects |> nodes, model |> nodes]"
        )),
        "[[],[\"b\",\"c\"],[\"a\",\"b\",\"c\"]]"
    );
}

#[test]
fn undefined_selections_stay_undefined() {
    behaves_as_spelled(&[
        (
            "a = 1\nb = a + 2\nc = b * 2\nn = null\nnames = c |> why |> depth(n) |> nodes\n[names, (c |> why |> depth(n) |> nodes).length]",
            "a = 1\nb = a + 2\nc = b * 2\nn = null\nnames = (c |> why |> depth(n)).map(step => step.node)\n[names, (c |> why |> depth(n)).map(step => step.node).length]",
        ),
        ("n = 1\nnames", "n = 1\nnames"),
        ("n = undefined\nnames", "n = undefined\nnames"),
    ]);
    assert_eq!(
        result(&format!("{CHAIN}c |> why |> depth(null) |> nodes")),
        "null"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}[`v ${{c |> why |> depth(null) |> nodes}}`, typeof (c |> why |> depth(null) |> nodes)]"
        )),
        "[\"v undefined\",\"undefined\"]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}[`v ${{(c |> why |> depth(null)).map(step => step.node)}}`, typeof (c |> why |> depth(null)).map(step => step.node)]"
        )),
        "[\"v undefined\",\"undefined\"]"
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}n = null\nfor s in c |> why |> depth(n) |> nodes:\n    r = s"
        )),
        "TypeError: c|>why|>depth(n)|>nodes is not iterable (line 5, column 1)"
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}n = null\nfor s in (c |> why |> depth(n)).map(step => step.node):\n    r = s"
        )),
        "TypeError: c|>why|>depth(n).map(function()) is not iterable (line 5, column 1)"
    );
}

#[test]
fn limits_in_the_source_freeze_and_arguments_that_end_with_nodes_stay_as_written() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}n = 1\nnames = c |> why |> depth(n.value) |> nodes\nn = 2"),
    );
    assert_eq!(outcome(&mut runtime, "names"), "[\"c\",\"b\"]");
    assert_eq!(stored(&runtime, "names"), "c|>why|>depth(1)|>nodes");
    let mut spelling = Runtime::new();
    checked(
        &mut spelling,
        &format!(
            "{CHAIN}n = 1\nnames = (c |> why |> depth(n.value)).map(step => step.node)\nn = 2"
        ),
    );
    assert_eq!(outcome(&mut spelling, "names"), "[\"c\",\"b\"]");
    for (written, expected) in [
        (
            "d = a + 10\nv = c |> why |> intersect(d |> why |> depth(m.value) |> nodes)",
            "TypeError: Cannot explain d|>why|>depth(m.value)|>nodes (line 5, column 1)",
        ),
        (
            "d = a + 10\nv = c |> why |> intersect((d |> why |> depth(m.value)).map(step => step.node))",
            "TypeError: Cannot explain d|>why|>depth(m.value).map(function()) (line 5, column 1)",
        ),
        (
            "v = c |> why |> depth(m.value) |> nodes |> depth(1)",
            "TypeError: Cannot explain c|>why|>depth(m.value)|>nodes (line 4, column 1)",
        ),
        (
            "v = (c |> why |> depth(m.value)).map(step => step.node) |> depth(1)",
            "TypeError: Cannot explain c|>why|>depth(m.value).map(function()) (line 4, column 1)",
        ),
        (
            "v = c |> why |> depth(m.value) |> nodes |> nodes",
            "TypeError: Cannot explain c|>why|>depth(m.value)|>nodes (line 4, column 1)",
        ),
        (
            "v = c |> why |> depth(m.value) |> nodes",
            "ReferenceError: m is not defined (line 4, column 1)",
        ),
        (
            "v = (c |> why |> depth(m.value)).map(step => step.node)",
            "ReferenceError: m is not defined (line 4, column 1)",
        ),
    ] {
        assert_eq!(failure(&format!("{CHAIN}{written}")), expected, "{written}");
    }
    let mut combined = Runtime::new();
    checked(
        &mut combined,
        &format!(
            "{CHAIN}d = a + 10\nk = 1\nv = c |> why |> intersect(d |> why |> depth(k.value)) |> nodes\nk = 0"
        ),
    );
    assert_eq!(
        stored(&combined, "v"),
        "c|>why|>intersect(d|>why|>depth(1))|>nodes"
    );
    assert_eq!(outcome(&mut combined, "v"), "[\"a\"]");
    let mut combined_spelling = Runtime::new();
    checked(
        &mut combined_spelling,
        &format!(
            "{CHAIN}d = a + 10\nk = 1\nv = (c |> why |> intersect(d |> why |> depth(k.value))).map(step => step.node)\nk = 0"
        ),
    );
    assert_eq!(outcome(&mut combined_spelling, "v"), "[\"a\"]");
}

#[test]
fn the_graph_stays_bounded_where_the_accepted_spelling_grows() {
    let mut stage = Runtime::new();
    let mut spelling = Runtime::new();
    checked(&mut stage, &format!("{CHAIN}names = c |> why |> nodes"));
    checked(&mut spelling, &format!("{CHAIN}names = (why c).map(.node)"));
    assert_eq!((stage.graph.len(), spelling.graph.len()), (4, 7));
    for write in 0..50 {
        let source = format!("a = {}", write + 2);
        checked(&mut stage, &source);
        checked(&mut spelling, &source);
        assert_eq!(stage.graph.len(), 4);
        assert_eq!(spelling.graph.len(), 7 + 3 * (write + 1));
        assert_eq!(pending(&stage), 0);
        assert_eq!(pending(&spelling), 3 * (write + 2));
        assert_eq!(reads(&stage, "names"), ["a", "b", "c"]);
        assert_eq!(reads(&stage, "names"), reads(&spelling, "names"));
        assert_eq!(observes(&stage, "names"), observes(&spelling, "names"));
    }
    assert_eq!(
        outcome(&mut stage, "names"),
        outcome(&mut spelling, "names")
    );
}

#[test]
fn rendering_shows_the_stage_as_written() {
    for (source, rendered) in [
        ("c |> why |> nodes", "c|>why|>nodes"),
        ("why c |> stated |> nodes", "c|>why|>stated|>nodes"),
        ("model |> nodes", "model|>nodes"),
        (
            "c |> why |> intersect(d |> why) |> nodes",
            "c|>why|>intersect(d|>why)|>nodes",
        ),
        ("c |> why |> nodes |> why", "c|>why|>nodes|>why"),
        ("(c |> why |> nodes).length", "c|>why|>nodes.length"),
        ("(c |> why |> stated).length", "c|>why|>stated.length"),
    ] {
        let parsed = parse_expression(source).unwrap();
        assert_eq!(parsed.to_string(), rendered, "{source}");
    }
    for source in [
        "c |> why |> nodes",
        "c |> nodes",
        "c |> why |> depth(1) |> nodes",
        "c |> why |> nodes |> why",
    ] {
        let parsed = parse_expression(source).unwrap();
        assert_eq!(
            parse_expression(&parsed.to_string()).unwrap(),
            parsed,
            "{source}"
        );
    }
    assert_eq!(
        generate(&parse("def f(x = 1) => c |> why |> nodes").unwrap()[0]),
        "def f(x=1):\n    return c|>why|>nodes"
    );
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{CHAIN}names = c |> why |> nodes"));
    assert_eq!(stored(&runtime, "names"), "c|>why|>nodes");
    assert_eq!(
        failure(&format!("{CHAIN}delete c |> why |> nodes")),
        "SyntaxError: Cannot delete c|>why|>nodes (line 4, column 1)"
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-node-names-{label}-{}.nuc",
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

fn without_record_reads(graph: &str) -> String {
    let mut kept = Vec::new();
    let mut record = false;
    for line in graph.lines() {
        if line.starts_with("  ") && !line.starts_with("    ") {
            record = line.trim_start().starts_with("obj:");
        }
        if record && line.starts_with("  ") {
            continue;
        }
        if line.starts_with("logic graph (") {
            kept.push("logic graph".to_string());
        } else if line.trim_start().starts_with("reads") {
            let (head, rest) = line.split_once("reads").unwrap();
            let keys: Vec<&str> = rest
                .split(", ")
                .map(str::trim)
                .filter(|key| !key.starts_with("obj:"))
                .collect();
            kept.push(format!("{head}reads    {}", keys.join(", ")));
        } else {
            kept.push(line.to_string());
        }
    }
    kept.join("\n")
}

const PROGRAM: &str = "a = 1\nb = a + 2\nc = b * 2\nnames = c |> why |> nodes\nfacts = c |> why |> stated |> nodes\nx = 5\nb = x + 1\n[names, facts, a |> affects |> nodes]";
const PROGRAM_SPELLED: &str = "a = 1\nb = a + 2\nc = b * 2\nnames = (c |> why).map(step => step.node)\nfacts = (c |> why |> stated).map(step => step.node)\nx = 5\nb = x + 1\n[names, facts, (a |> affects).map(step => step.node)]";

#[test]
fn files_graphs_and_repl_sessions_name_nodes() {
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
        "[[\"c\",\"b\",\"x\"],[\"x\"],[]]\n"
    );
    for options in [&["--graph"][..], &["--json", "--graph"][..]] {
        let stage = String::from_utf8(file.run(PROGRAM, options).stdout).unwrap();
        let spelling = String::from_utf8(file.run(PROGRAM_SPELLED, options).stdout).unwrap();
        assert!(!stage.contains("obj:"), "{stage}");
        assert!(spelling.contains("obj:"), "{spelling}");
        assert_eq!(
            without_record_reads(&stage),
            without_record_reads(&spelling),
            "{options:?}"
        );
        assert!(
            stage.contains("  names [variable]\n    reads    c, b, x\n    observes c, b, x"),
            "{stage}"
        );
    }
    let numbered =
        "a = 1\nb = a + 2\nc = b * 2\nx = c |> why |> nodes\np = {k: 1}\n[String(p), `${p}`]";
    let numbered_spelled = "a = 1\nb = a + 2\nc = b * 2\nx = (c |> why).map(step => step.node)\np = {k: 1}\n[String(p), `${p}`]";
    for options in [&[][..], &["--json"][..]] {
        assert_eq!(
            file.run(numbered, options).stdout,
            file.run(numbered_spelled, options).stdout,
            "{options:?}"
        );
    }
    assert_eq!(
        String::from_utf8(file.run(numbered, &["--json"]).stdout).unwrap(),
        "[\"[object obj:3]\",\"[object obj:3]\"]\n"
    );
    for (source, message) in [
        (
            "a = 1\nnodes = 2\na |> nodes\n",
            "TypeError: nodes is not an operation (line 3, column 1)\n",
        ),
        (
            "a = 1\na |> nodes |> why\n",
            "TypeError: Cannot explain a|>nodes (line 2, column 1)\n",
        ),
    ] {
        let failed = file.run(source, &["--json"]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(String::from_utf8(failed.stderr).unwrap(), message);
    }
    let session = "a = 1\nb = a + 2\nc = b * 2\nc |> why |> nodes\nnames = c |> why |> stated |> nodes\nnames\nx = 5\nb = x + 1\nnames\n.exit\n";
    let session_spelled = session
        .replace("c |> why |> nodes", &accepted("c |> why"))
        .replace(
            "c |> why |> stated |> nodes",
            &accepted("c |> why |> stated"),
        );
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let stage = repl(&session.replace('\n', newline), options);
            let spelling = repl(&session_spelled.replace('\n', newline), options);
            assert!(stage.status.success(), "{options:?} {stage:?}");
            assert_eq!(stage.stdout, spelling.stdout, "{options:?}");
            assert_eq!(stage.stderr, spelling.stderr, "{options:?}");
            let stdout = String::from_utf8(stage.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "[\"c\",\"b\",\"a\"]\n[\"a\"]\n"
            } else {
                "c,b,a\na\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
        }
    }
}

#[test]
fn long_chains_of_nodes_stages_raise_without_recursing() {
    let stages = " |> nodes".repeat(120);
    let source = format!("{CHAIN}c |> why{stages}");
    let quoted = format!("c|>why{}", "|>nodes".repeat(119));
    assert_eq!(
        failure(&source),
        format!("TypeError: Cannot explain {quoted} (line 4, column 1)")
    );
    let declared = format!("{CHAIN}v = c |> why{stages}");
    assert_eq!(
        failure(&declared),
        format!("TypeError: Cannot explain {quoted} (line 4, column 1)")
    );
}

#[test]
fn the_compatibility_rows_give_their_documented_results() {
    let changed = [
        ("c |> why |> nodes", "[\"c\",\"b\",\"a\"]"),
        ("c |> nodes", "[\"c\"]"),
        ("model |> nodes", "[\"a\",\"b\",\"c\"]"),
        ("a |> affects |> nodes", "[\"b\",\"c\"]"),
        ("affects a |> nodes", "[\"b\",\"c\"]"),
        ("c |> why |> stated |> nodes", "[\"a\"]"),
        ("c |> why |> derived |> nodes", "[\"c\",\"b\"]"),
        ("c |> why |> depth(1) |> nodes", "[\"c\",\"b\"]"),
        ("c |> why |> path(a) |> nodes", "[\"c\",\"b\",\"a\"]"),
        (
            "d = a + 10\nc |> why |> intersect(d |> why) |> nodes",
            "[\"a\"]",
        ),
        (
            "d = a + 10\nc |> why |> except(d |> why) |> nodes",
            "[\"c\",\"b\"]",
        ),
        ("c |> why |> depth(null) |> nodes", "null"),
        (
            "1 |> nodes",
            "TypeError: Cannot explain 1 (line 4, column 1)",
        ),
        (
            "missing |> why |> nodes",
            "ReferenceError: missing is not defined (line 4, column 1)",
        ),
        (
            "c |> why |> diff |> nodes",
            "TypeError: Cannot use 'diff' outside suppose (line 4, column 1)",
        ),
        (
            "c |> why |> nodes |> why",
            "TypeError: Cannot explain c|>why|>nodes (line 4, column 1)",
        ),
        (
            "c |> why |> nodes |> depth(1)",
            "TypeError: Cannot explain c|>why|>nodes (line 4, column 1)",
        ),
        (
            "c |> why |> nodes |> nodes",
            "TypeError: Cannot explain c|>why|>nodes (line 4, column 1)",
        ),
        (
            "stated = 1\nc |> why |> nodes |> stated",
            "TypeError: stated is not an operation (line 5, column 1)",
        ),
        (
            "d = a + 10\nc |> why |> intersect(d |> why |> nodes)",
            "TypeError: Cannot explain d|>why|>nodes (line 5, column 1)",
        ),
        (
            "c |> why |> path(a |> why |> nodes)",
            "TypeError: Cannot explain a|>why|>nodes (line 4, column 1)",
        ),
        (
            "d = a + 10\nother = d |> why |> nodes\nc |> why |> intersect(other) |> nodes",
            "[]",
        ),
        (
            "nodes = 1\nc |> why |> nodes",
            "TypeError: nodes is not an operation (line 5, column 1)",
        ),
        (
            "def nodes(x) => x\nc |> why |> nodes",
            "TypeError: nodes is not an operation (line 5, column 1)",
        ),
        (
            "class nodes\nc |> why |> nodes",
            "TypeError: nodes is not an operation (line 5, column 1)",
        ),
        (
            "try:\n    r = c |> why |> nodes\ncatch e:\n    r = \"caught\"\nr",
            "[\"c\",\"b\",\"a\"]",
        ),
        (
            "nodes = 1\ntry:\n    r = c |> why |> nodes\ncatch e:\n    r = \"caught\"\nr",
            "\"caught\"",
        ),
        (
            "names = c |> why |> nodes\nx = 5\nb = x + 1\nnames",
            "[\"c\",\"b\",\"x\"]",
        ),
        ("names = c |> why |> nodes\nwhy names", "[]"),
        ("`n ${c |> why |> nodes}`", "\"n c,b,a\""),
        (
            "suppose a = 5:\n    inner = c |> why |> nodes\n    inner",
            "[\"c\",\"b\",\"a\"]",
        ),
        ("x, y = c |> why |> nodes\n[x, y]", "[\"c\",\"b\"]"),
        (
            "x, y = [c |> why |> nodes, 1]\n[x, y]",
            "[[\"c\",\"b\",\"a\"],1.0]",
        ),
        ("{length} = c |> why |> nodes\nlength", "3.0"),
        ("{x, y} = c |> why |> nodes\n[x, y]", "[null,null]"),
        ("x, = c |> why |> nodes\nx", "\"c\""),
        (
            "[p] = c |> why |> nodes",
            "SyntaxError: Cannot assign to [p] (line 4, column 1)",
        ),
        (
            "delete c |> why |> nodes",
            "SyntaxError: Cannot delete c|>why|>nodes (line 4, column 1)",
        ),
        (
            "not c |> why |> nodes",
            "TypeError: Cannot explain !c (line 4, column 1)",
        ),
        ("c |> why |> nodes(1)", "1.0"),
        ("c |> why |> nodes[0]", "[0.0]"),
        (
            "c |> why |> nodes.length",
            "SyntaxError: Unexpected '.' (line 4, column 19)",
        ),
        ("(c |>\nwhy |>\nnodes)", "[\"c\",\"b\",\"a\"]"),
        ("c |> why |> nodes; a", "1.0"),
        (
            "names = c |> why |> nodes\nnames.push(\"z\")\nnames",
            "[\"c\",\"b\",\"a\",\"z\"]",
        ),
        (
            "n = null\nfor s in c |> why |> depth(n) |> nodes:\n    r = s",
            "TypeError: c|>why|>depth(n)|>nodes is not iterable (line 5, column 1)",
        ),
    ];
    let unchanged = [
        ("c |> why |> Nodes", "TypeError: Nodes is not an operation"),
        ("c |> why |> node", "TypeError: node is not an operation"),
        (
            "c |> why |>\nnodes",
            "SyntaxError: Expected an operation (line 4, column 10)",
        ),
        (
            "nodes c",
            "ReferenceError: nodes is not defined (line 4, column 1)",
        ),
        (
            "nodes",
            "ReferenceError: nodes is not defined (line 4, column 1)",
        ),
        (
            "nodes(c |> why)",
            "ReferenceError: nodes is not defined (line 4, column 1)",
        ),
        ("nodes = [1, 2]\nnodes", "[1.0,2.0]"),
        ("x = {nodes: 1}\nx.nodes", "1.0"),
        ("def nodes(x) => x * 2\nnodes(3)", "6.0"),
        ("(c |> why).nodes", "null"),
        ("`v ${c why c |> nodes}`", "\"v 6\""),
        (
            "def f(x = c |> why |> nodes) => x",
            "SyntaxError: Expected a scalar literal parameter default (line 4, column 11)",
        ),
        (
            "{x: c |> why |> nodes}",
            "SyntaxError: Unexpected ':' (line 4, column 5)",
        ),
        (
            "(c).map(step => step.node)",
            "TypeError: number.map is not a function (line 4, column 1)",
        ),
        ("(c |> why |> depth(0)).map(step => step.node)", "[\"c\"]"),
    ];
    for (last, expected) in changed.into_iter().chain(unchanged) {
        assert_eq!(fresh(&format!("{CHAIN}{last}")), expected, "{last}");
    }
}
