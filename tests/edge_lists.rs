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
    "Nucleoid lists the edges of an explanation with edges",
    "Nucleoid lists the edges that a change travels and the model with edges",
    "Nucleoid lists the edges behind class level rules with edges",
    "Nucleoid rolls back failed updates through edge lists",
];
const REFERENCE_TITLE: &str = "Nucleoid lists the edges of selections with edges";
const README_TITLE: &str = "Edges draws the wiring behind a heat alarm";
const EXAMPLES_TITLE: &str =
    "Edges traces where a supplier's delay travels through a build schedule";

const CHAIN: &str = "a = 1\nb = a + 2\nc = b * 2\n";
const SIX: &str = "a = 1\nb = a + 1\nc = b * 2\nd = a + 3\ne = 7\nf = e + d\n";
const KINDS: &str = "class P(v): this.v = v\np = P(5)\ndef f(x) => x\ng = x -> x\nk = 1\nif k > 0:\n    m = 2\ns = \"hi\"\nxs = [1, [2]]\nnone = null\nflag = true\no = {k: 1}\nr = o\nw = why k\nn = w.length\nu = {}\ny = u.gone\nt = k + k\n";
const CLASSY: &str =
    "rate = 2\nclass M(load):\n    this.load = load\n$M.wear = $M.load * rate\nm = M(3)\n";
const HELPER: &str =
    "e = sel -> sel.flatMap(s -> s.from.zip((s.node + \"|\").repeat(s.from.length).split(\"|\")))";
const DETOUR: &str = "s.from.zip((s.node + \"|\").repeat(s.from.length).split(\"|\"))";

type Rewrite = (&'static str, &'static str);

struct Counterpart {
    title: &'static str,
    rewrites: &'static [Rewrite],
    before: usize,
    after: usize,
}

const SPELLED: [Counterpart; 7] = [
    Counterpart {
        title: "Nucleoid lists the edges of an explanation with edges",
        rewrites: &[
            ("price = 10", "{helper}\nprice = 10"),
            ("total |> why |> edges", "e(why total)"),
            ("tax |> why |> edges", "e(why tax)"),
        ],
        before: 263,
        after: 223,
    },
    Counterpart {
        title: "Nucleoid lists the edges that a change travels and the model with edges",
        rewrites: &[
            ("rate = 0.2", "{helper}\nrate = 0.2"),
            ("price |> affects |> edges", "e(affects price)"),
            ("model |> edges", "e(model |> why)"),
        ],
        before: 281,
        after: 238,
    },
    Counterpart {
        title: "Nucleoid lists the edges behind class level rules with edges",
        rewrites: &[
            ("markup = 2", "{helper}\nmarkup = 2"),
            ("$Part.price |> why |> edges", "e(why $Part.price)"),
            ("fee |> affects |> edges", "e(affects fee)"),
        ],
        before: 227,
        after: 187,
    },
    Counterpart {
        title: "Nucleoid rolls back failed updates through edge lists",
        rewrites: &[
            (
                "subtotal |> why |> edges",
                "(why subtotal).flatMap(s -> {detour})",
            ),
            ("1 |> edges", "1 |> why"),
        ],
        before: 199,
        after: 166,
    },
    Counterpart {
        title: "Nucleoid lists the edges of selections with edges",
        rewrites: &[
            ("price = 10", "{helper}\nprice = 10"),
            ("total |> why |> edges", "e(why total)"),
            ("price |> affects |> edges", "e(affects price)"),
        ],
        before: 153,
        after: 113,
    },
    Counterpart {
        title: "Edges draws the wiring behind a heat alarm",
        rewrites: &[(
            "(alarm |> why |> edges)",
            "(why alarm).flatMap(s -> {detour})",
        )],
        before: 124,
        after: 93,
    },
    Counterpart {
        title: "Edges traces where a supplier's delay travels through a build schedule",
        rewrites: &[(
            "delay |> affects |> edges",
            "(affects delay).flatMap(s -> {detour})",
        )],
        before: 111,
        after: 78,
    },
];

const FORMS: [(&str, &str); 26] = [
    ("X |> edges", "X |> depth(0)"),
    ("X |> why |> edges", "X |> why"),
    ("why X |> edges", "why X"),
    ("X |> affects |> edges", "X |> affects"),
    ("affects X |> edges", "affects X"),
    ("X |> why |> stated |> edges", "X |> why |> stated"),
    ("X |> why |> derived |> edges", "X |> why |> derived"),
    ("stated X |> edges", "stated X"),
    ("derived X |> edges", "derived X"),
    ("stated why X |> edges", "stated why X"),
    ("X |> why |> depth(1) |> edges", "X |> why |> depth(1)"),
    (
        "X |> affects |> depth(1) |> edges",
        "X |> affects |> depth(1)",
    ),
    ("X |> why |> path(Y) |> edges", "X |> why |> path(Y)"),
    (
        "Y |> affects |> path(X) |> edges",
        "Y |> affects |> path(X)",
    ),
    (
        "X |> why |> intersect(Y |> why) |> edges",
        "X |> why |> intersect(Y |> why)",
    ),
    (
        "X |> why |> except(Y |> why) |> edges",
        "X |> why |> except(Y |> why)",
    ),
    (
        "X |> why |> union(Y |> why) |> edges",
        "X |> why |> union(Y |> why)",
    ),
    (
        "X |> affects |> union(Y |> why) |> depth(1) |> edges",
        "X |> affects |> union(Y |> why) |> depth(1)",
    ),
    (
        "X |> why |> stated |> except(Y |> why |> intersect(Y |> why)) |> edges",
        "X |> why |> stated |> except(Y |> why |> intersect(Y |> why))",
    ),
    (
        "X |> why |> depth(null) |> edges",
        "X |> why |> depth(null)",
    ),
    ("X |> why |> origins |> edges", "X |> why |> origins"),
    (
        "X |> affects |> outcomes |> edges",
        "X |> affects |> outcomes",
    ),
    ("model |> edges", "model |> depth(0)"),
    ("model |> stated |> edges", "model |> stated"),
    (
        "model |> intersect(X |> why) |> edges",
        "model |> intersect(X |> why)",
    ),
    ("model |> outcomes |> edges", "model |> outcomes"),
];

const GRAPHS: [(&str, &[&str], &[&str]); 8] = [
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
        &["rate", "m.wear", "m", "t"],
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
    (
        "a = 2\ng = a + a * a\nif g > 3:\n    big = true\nh = g + a\n",
        &["a", "g", "h"],
        &["a = 1", "g = a * 3", "h = g"],
    ),
    (
        "class Box(width, height):\n    this.width = width\n    this.height = height\nbox = Box(4, 1)\n$Box.width = $Box.height@first\n$Box.height = $Box.width@first\n",
        &["box.width", "box.height", "box"],
        &["late = Box(2, 3)"],
    ),
    (
        "u = {}\ny = u.gone\nk = 1\nw = why k\nn = w.length + k\ndef f(x) => x\nclass Q\n",
        &["y", "n", "k"],
        &["u.gone = 3", "z = n + y", "k = 2"],
    ),
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

fn writes_edges(source: &str) -> bool {
    tokenize(source).unwrap().windows(2).any(|window| {
        window[0].token == Token::Pipe
            && matches!(&window[1].token, Token::Identifier(name) if name == "edges")
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

fn object_ids(text: &str) -> Vec<u64> {
    let mut ids = Vec::new();
    let mut rest = text;
    while let Some((_, after)) = rest.split_once("obj:") {
        let digits = after
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(after.len());
        if let Ok(id) = after[..digits].parse() {
            ids.push(id);
        }
        rest = &after[digits..];
    }
    ids
}

fn relabeled(text: &str, order: &[u64]) -> String {
    let mut result = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("obj:") {
        let digits = after
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(after.len());
        let (identifier, tail) = after.split_at(digits);
        result.push_str(before);
        result.push_str("obj:#");
        match identifier
            .parse::<u64>()
            .ok()
            .and_then(|id| order.iter().position(|known| *known == id))
        {
            Some(index) => result.push_str(&index.to_string()),
            None => result.push_str(identifier),
        }
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
            title: "edge lists regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn accepted(selection: &str) -> String {
    format!(
        "({selection}).flatMap(step => step.from.reduce((pairs, input) => pairs.concat([[input, pairs[0]]]), [step.node])[1:])"
    )
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
    let shapes: Vec<String> = runtime
        .graph
        .shape_dependencies(&NodeKey::variable(name))
        .map(|shape| format!("{shape:?}"))
        .collect();
    let mut order: Vec<u64> = shapes.iter().flat_map(|shape| object_ids(shape)).collect();
    order.sort_unstable();
    order.dedup();
    let mut canonical: Vec<String> = shapes
        .iter()
        .map(|shape| relabeled(shape, &order))
        .collect();
    canonical.sort();
    canonical
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

fn behaves_as_spelled<S: AsRef<str> + std::fmt::Debug>(steps: &[(S, S)]) {
    let mut stage = Runtime::new();
    let mut spelling = Runtime::new();
    for (written, accepted) in steps {
        let (written, accepted) = (written.as_ref(), accepted.as_ref());
        assert_eq!(
            outcome(&mut stage, written),
            outcome(&mut spelling, accepted),
            "{written}"
        );
        for name in ["wiring", "facts", "inner", "names", "values"] {
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
fn only_the_appended_cases_write_edges_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(writes_edges(&case.source), new, "{}", case.title);
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
    assert_eq!(assertions, 22);
    assert_eq!(cases, 1333);
}

#[test]
fn the_audit_finds_every_edges_stage_and_nothing_else() {
    for source in [
        "c |> edges",
        "c |> why |> edges",
        "why c |> stated |> edges",
        "x = model|>edges",
        "f(c |> why |> edges)",
        "$P.links = $P.v |> why |> edges",
        "c |> why |> edges |> why",
        "c |> why |> edges(1)",
        "(c |> why |> edges).map(edge -> edge.join(\" -> \"))",
    ] {
        assert!(writes_edges(source), "{source}");
    }
    for source in [
        "edges",
        "edges c",
        "edges(c |> why)",
        "edges = [1, 2]",
        "o = {edges: 1}",
        "o.edges",
        "(why c).flatMap(s -> s.from)",
        "def edges(x) => x",
        "c |> why |> Edges",
        "c |> why |> edge",
        "c |> why |>\nedges",
        "c |> why |> nodes",
        "c |> why |> holds",
        "\"|> edges\"",
    ] {
        assert!(!writes_edges(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_131_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[831],
        "Nucleoid rolls back failed updates through reversal aliases"
    );
    assert_eq!(titles[832..836], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid reverses lists, populations and strings as Kotlin, Swift and JavaScript do",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Reversal methods keep an undo history newest first",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Reversal methods trace every delivery route back to its depot",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 10 - 12 * usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 1].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Edge lists of reasoning selections\n\n```\n")
        .expect("the language reference documents the edges stage");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(
        case.expected.as_deref(),
        Some(
            "[[[\"subtotal\",\"total\"],[\"shipping\",\"total\"],[\"price\",\"subtotal\"],[\"qty\",\"subtotal\"],[\"price\",\"shipping\"]],[[\"price\",\"subtotal\"],[\"qty\",\"subtotal\"],[\"price\",\"shipping\"],[\"subtotal\",\"total\"],[\"shipping\",\"total\"]]]"
        )
    );
    for line in [
        "[NUC 133](nuc-0133.md)",
        "reference |> why |> edges             the [input, node] edges of an explanation's steps, in its order (NUC 133)",
        "model |> edges                        the edges of the current model's nodes (NUC 133)",
        "selection |> edges                    one [input, step.node] pair per name of each step's from; ends the pipeline (NUC 133)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    assert!(include_str!("../README.md").contains("[NUC 133](docs/nuc-0133.md)"));
    assert!(include_str!("../docs/examples.md").contains("[NUC 133](nuc-0133.md)"));
    assert!(
        include_str!("../nucleoid.spec.md").contains("\n## Edge lists of reasoning selections\n")
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
            let spelling = spelling
                .replace("{helper}", HELPER)
                .replace("{detour}", DETOUR);
            assert_eq!(
                counterpart.matches(written).count(),
                1,
                "{title}: {written}"
            );
            counterpart = counterpart.replacen(written, &spelling, 1);
        }
        assert!(writes_edges(&case.source), "{title}");
        assert!(!writes_edges(&counterpart), "{title}");
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
fn edges_parses_as_a_final_stage_and_stays_an_ordinary_name() {
    let why_c = Expr::Reason {
        stage: Stage::Why,
        source: Box::new(Expr::Identifier("c".to_string())),
    };
    let wiring = Expr::Reason {
        stage: Stage::Edges,
        source: Box::new(why_c.clone()),
    };
    assert_eq!(parse_expression("c |> why |> edges").unwrap(), wiring);
    assert_eq!(parse_expression("why c |> edges").unwrap(), wiring);
    assert_eq!(parse_expression("c|>why|>edges").unwrap(), wiring);
    assert_eq!(parse_expression("(c |>\nwhy |>\nedges)").unwrap(), wiring);
    assert_eq!(
        parse_expression("c |> edges").unwrap(),
        Expr::Reason {
            stage: Stage::Edges,
            source: Box::new(Expr::Identifier("c".to_string())),
        }
    );
    assert_eq!(
        parse_expression("model |> edges").unwrap(),
        Expr::Reason {
            stage: Stage::Edges,
            source: Box::new(Expr::Model),
        }
    );
    assert_eq!(
        parse_expression("c |> why |> edges |> why").unwrap(),
        Expr::Reason {
            stage: Stage::Why,
            source: Box::new(wiring.clone()),
        }
    );
    assert_eq!(
        parse_expression("c |> why |> holds |> edges").unwrap(),
        Expr::Reason {
            stage: Stage::Edges,
            source: Box::new(Expr::Reason {
                stage: Stage::Holds,
                source: Box::new(why_c),
            }),
        }
    );
    assert_eq!(Stage::Edges.as_str(), "edges");
    assert_eq!(Stage::Holds.as_str(), "holds");
    assert_eq!(Stage::Nodes.as_str(), "nodes");
    assert_eq!(Stage::from_name("edges"), None);
    assert_eq!(Stage::from_name("why"), Some(Stage::Why));
    for (source, expected) in [
        (
            format!("{CHAIN}edges c"),
            "ReferenceError: edges is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}edges"),
            "ReferenceError: edges is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}edges(c |> why)"),
            "ReferenceError: edges is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> Edges"),
            "TypeError: Edges is not an operation",
        ),
        (
            format!("{CHAIN}c |> why |> edge"),
            "TypeError: edge is not an operation",
        ),
        (
            format!("{CHAIN}c |> why |>\nedges"),
            "SyntaxError: Expected an operation (line 4, column 10)",
        ),
        (format!("{CHAIN}edges = [1, 2]\nedges"), "[1.0,2.0]"),
        (format!("{CHAIN}x = {{edges: 1}}\nx.edges"), "1.0"),
        (format!("{CHAIN}def edges(x) => x * 2\nedges(3)"), "6.0"),
        (format!("{CHAIN}(c |> why).edges"), "null"),
        (format!("{CHAIN}(c |> why)[0].from"), "[\"b\"]"),
        (format!("{CHAIN}xs = [1]\nxs.edges"), "null"),
        (format!("{CHAIN}c |> why |> edges(1)"), "1.0"),
        (format!("{CHAIN}c |> why |> edges[0]"), "[0.0]"),
        (
            format!("{CHAIN}c |> why |> edges.length"),
            "SyntaxError: Unexpected '.' (line 4, column 19)",
        ),
        (
            format!("{CHAIN}c |> edges edges"),
            "ReferenceError: edges is not defined (line 4, column 12)",
        ),
        (format!("{CHAIN}(c |> why |> edges).length"), "2.0"),
    ] {
        assert_eq!(fresh(&source), expected, "{source}");
    }
}

#[test]
fn every_source_and_stage_gives_what_its_accepted_spelling_gives() {
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
    assert_eq!(compared, 2210);
    assert_eq!(
        result(&format!(
            "{CHAIN}[c |> why |> edges, model |> edges, a |> affects |> edges]"
        )),
        "[[[\"b\",\"c\"],[\"a\",\"b\"]],[[\"a\",\"b\"],[\"b\",\"c\"]],[[\"a\",\"b\"],[\"b\",\"c\"]]]"
    );
    assert_eq!(result("model |> edges"), "[]");
    assert_eq!(result(&format!("{CHAIN}a |> derived |> edges")), "[]");
    assert_eq!(result(&format!("{CHAIN}o = {{}}\no.gone |> edges")), "[]");
    let suppose =
        format!("{CHAIN}suppose a = 5:\n    [c |> why |> diff |> edges, model |> diff |> edges]");
    let suppose_spelled = format!(
        "{CHAIN}suppose a = 5:\n    [{}, {}]",
        accepted("c |> why |> diff"),
        accepted("model |> diff")
    );
    assert_eq!(fresh(&suppose), fresh(&suppose_spelled));
    assert_eq!(
        result(&suppose),
        "[[[\"b\",\"c\"],[\"a\",\"b\"]],[[\"a\",\"b\"],[\"b\",\"c\"]]]"
    );
}

#[test]
fn stated_steps_have_no_edges_and_derived_steps_have_them_all() {
    let mut checked_pairs = 0;
    for (setup, names, writes) in GRAPHS {
        let mut prefixes = vec![setup.to_string()];
        for write in writes {
            let previous = prefixes.last().unwrap().clone();
            prefixes.push(format!("{previous}{write}\n"));
        }
        for prefix in &prefixes {
            for selection in names
                .iter()
                .map(|name| format!("{name} |> why"))
                .chain(names.iter().map(|name| format!("{name} |> affects")))
                .chain(["model".to_string()])
            {
                let all = fresh(&format!("{prefix}{selection} |> edges"));
                let derived = fresh(&format!("{prefix}{selection} |> derived |> edges"));
                let stated = fresh(&format!("{prefix}{selection} |> stated |> edges"));
                assert_eq!(derived, all, "{prefix}{selection}");
                if all.starts_with('[') {
                    assert_eq!(stated, "[]", "{prefix}{selection}");
                } else {
                    assert_eq!(stated, all, "{prefix}{selection}");
                }
                checked_pairs += 1;
            }
        }
    }
    assert_eq!(checked_pairs, 243);
}

#[test]
fn pairs_name_their_inputs_as_the_records_name_them() {
    for selection in [
        "model |> depth(0)",
        "model |> stated",
        "r |> why",
        "p |> why |> depth(0)",
        "p.v |> affects",
        "n |> why",
        "y |> why",
        "t |> why",
        "xs |> why",
        "u |> affects",
    ] {
        let written = format!("{KINDS}{selection} |> edges");
        let spelling = format!("{KINDS}{}", accepted(selection));
        assert_eq!(fresh(&written), fresh(&spelling), "{selection}");
    }
    assert_eq!(
        result(&format!("{KINDS}model |> edges")),
        "[[\"k\",\"if(k>0)\"],[\"o\",\"r\"],[\"w\",\"n\"],[\"u\",\"y\"],[\"obj:0.gone\",\"y\"],[\"k\",\"t\"]]"
    );
    assert_eq!(
        result(&format!(
            "{KINDS}[n |> why |> edges, y |> edges, t |> edges, p.v |> why |> edges]"
        )),
        "[[[\"w\",\"n\"]],[[\"u\",\"y\"],[\"obj:0.gone\",\"y\"]],[[\"k\",\"t\"]],[]]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}g = a + a * a\nh = b + a + b\n[g |> edges, h |> edges]"
        )),
        "[[[\"a\",\"g\"]],[[\"b\",\"h\"],[\"a\",\"h\"]]]"
    );
    assert_eq!(
        fresh(&format!(
            "{CHAIN}g = a + a * a\nh = b + a + b\n[{}, {}]",
            accepted("g |> depth(0)"),
            accepted("h |> depth(0)")
        )),
        "[[[\"a\",\"g\"]],[[\"b\",\"h\"],[\"a\",\"h\"]]]"
    );
    assert_eq!(
        result(&format!(
            "{CLASSY}k = M(4)\n[m.wear |> why |> edges, rate |> affects |> edges]"
        )),
        "[[[\"m.load\",\"m.wear\"],[\"rate\",\"m.wear\"]],[[\"m.load\",\"m.wear\"],[\"rate\",\"m.wear\"],[\"k.load\",\"k.wear\"],[\"rate\",\"k.wear\"]]]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}(c |> why |> edges).map(edge -> edge.join(\" -> \"))"
        )),
        "[\"b -> c\",\"a -> b\"]"
    );
}

#[test]
fn a_reference_alone_gives_the_edges_into_it() {
    assert_eq!(result(&format!("{CHAIN}c |> edges")), "[[\"b\",\"c\"]]");
    assert_eq!(
        result(&format!("{CHAIN}{}", accepted("c |> depth(0)"))),
        "[[\"b\",\"c\"]]"
    );
    assert_eq!(
        result(&format!("{CHAIN}{}", accepted("c |> why |> depth(0)"))),
        "[[\"b\",\"c\"]]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}[a |> edges, a |> stated |> edges, c |> stated |> edges, c |> derived |> edges]"
        )),
        "[[],[],[],[[\"b\",\"c\"]]]"
    );
    assert_eq!(
        failure(&format!("{CHAIN}(c).flatMap(step => step.from)")),
        "TypeError: number.flatMap is not a function (line 4, column 1)"
    );
    for (source, expected) in [
        (
            format!("{CHAIN}missing |> edges"),
            "ReferenceError: missing is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}1 |> edges"),
            "TypeError: Cannot explain 1 (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c + 1 |> edges"),
            "TypeError: Cannot explain c+1 (line 4, column 1)",
        ),
        (
            "n = null\nn.p |> edges".to_string(),
            "ReferenceError: n is not defined (line 2, column 1)",
        ),
        (
            "k = 5\nk.p |> edges".to_string(),
            "TypeError: Cannot explain k.p (line 2, column 1)",
        ),
        (
            "s = \"ab\"\ns.p |> edges".to_string(),
            "TypeError: Cannot explain s.p (line 2, column 1)",
        ),
        (
            "u = {}\nu.q.p |> edges".to_string(),
            "ReferenceError: u.q is not defined (line 2, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> depth(-1) |> edges"),
            "TypeError: -1 is not a non-negative integer (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> diff |> edges"),
            "TypeError: Cannot use 'diff' outside suppose (line 4, column 1)",
        ),
    ] {
        assert_eq!(failure(&source), expected, "{source}");
    }
}

#[test]
fn stages_after_edges_and_edges_arguments_raise_their_accepted_errors() {
    let prelude = format!("{CHAIN}d = a + 10\n");
    let spelled_c = accepted("c |> why");
    let spelled_d = accepted("d |> why");
    let spelled_a = accepted("a |> why");
    for (written, spelling, expected, spelled_error) in [
        (
            "c |> why |> edges |> why".to_string(),
            format!("{spelled_c} |> why"),
            "TypeError: Cannot explain c|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain c|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> edges |> depth(1)".to_string(),
            format!("{spelled_c} |> depth(1)"),
            "TypeError: Cannot explain c|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain c|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> edges |> nodes".to_string(),
            format!("{spelled_c} |> nodes"),
            "TypeError: Cannot explain c|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain c|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> edges |> holds".to_string(),
            format!("{spelled_c} |> holds"),
            "TypeError: Cannot explain c|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain c|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> edges |> edges".to_string(),
            format!("{spelled_c} |> depth(0)"),
            "TypeError: Cannot explain c|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain c|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> nodes |> edges".to_string(),
            "(c |> why).map(step => step.node) |> depth(0)".to_string(),
            "TypeError: Cannot explain c|>why|>nodes (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> holds |> edges".to_string(),
            "(c |> why).map(step => step.holds) |> depth(0)".to_string(),
            "TypeError: Cannot explain c|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> edges |> origins".to_string(),
            format!("{spelled_c} |> origins"),
            "TypeError: Cannot explain c|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain c|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> edges |> affects |> path(a)".to_string(),
            format!("{spelled_c} |> affects |> path(a)"),
            "TypeError: Cannot explain c|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain c|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> edges |> union(c |> why)".to_string(),
            format!("{spelled_c} |> union(c |> why)"),
            "TypeError: Cannot explain c|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain c|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> intersect(d |> why |> edges)".to_string(),
            format!("c |> why |> intersect({spelled_d})"),
            "TypeError: Cannot explain d|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain d|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> except(d |> why |> edges)".to_string(),
            format!("c |> why |> except({spelled_d})"),
            "TypeError: Cannot explain d|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain d|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> union(d |> why |> edges)".to_string(),
            format!("c |> why |> union({spelled_d})"),
            "TypeError: Cannot explain d|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain d|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> depth(2) |> union(d |> why |> edges)".to_string(),
            format!("c |> why |> depth(2) |> union({spelled_d})"),
            "TypeError: Cannot explain d|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain d|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> path(a |> why |> edges)".to_string(),
            format!("c |> why |> path({spelled_a})"),
            "TypeError: Cannot explain a|>why|>edges (line 5, column 1)",
            "TypeError: Cannot explain a|>why.flatMap(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> depth(m.value) |> edges |> why".to_string(),
            format!("{} |> why", accepted("c |> why |> depth(m.value)")),
            "TypeError: Cannot explain c|>why|>depth(m.value)|>edges (line 5, column 1)",
            "TypeError: Cannot explain c|>why|>depth(m.value).flatMap(function()) (line 5, column 1)",
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
        failure(&format!("{prelude}stated = 1\nc |> why |> edges |> stated")),
        "TypeError: stated is not an operation (line 6, column 1)"
    );
    assert_eq!(
        result(&format!(
            "{prelude}other = d |> why |> edges\n[c |> why |> intersect(other) |> edges, c |> why |> except(other) |> edges, c |> why |> union(other) |> edges]"
        )),
        "[[],[[\"b\",\"c\"],[\"a\",\"b\"]],[[\"b\",\"c\"],[\"a\",\"b\"]]]"
    );
    assert_eq!(
        result(&format!(
            "{prelude}other = {spelled_d}\n[{}, {}, {}]",
            accepted("c |> why |> intersect(other)"),
            accepted("c |> why |> except(other)"),
            accepted("c |> why |> union(other)")
        )),
        "[[],[[\"b\",\"c\"],[\"a\",\"b\"]],[[\"b\",\"c\"],[\"a\",\"b\"]]]"
    );
}

#[test]
fn every_binding_named_edges_shadows_the_stage_at_run_time() {
    let bindings = [
        ("edges = 1\nc |> why |> edges", 5),
        ("def edges(x) => x\nc |> why |> edges", 5),
        ("class edges\nc |> why |> edges", 5),
        ("edges = 1\nmissing |> why |> edges", 5),
        ("edges = 1\n1 |> edges", 5),
        ("stated = 1\nedges = 2\nc |> why |> stated |> edges", 6),
        ("def f(edges) => c |> why |> edges\nf(1)", 5),
        ("def g edges => c |> why |> edges\ng(1)", 5),
        ("f = edges -> c |> why |> edges\nf(1)", 5),
        ("f = (edges) => c |> why |> edges\nf(1)", 5),
        ("f = lambda edges: c |> why |> edges\nf(1)", 5),
        ("[1].map(edges -> c |> why |> edges)", 4),
        ("for edges of \"x\":\n    y = c |> why |> edges", 4),
        ("ok = all edges in [1]: (c |> why |> edges).length > 0", 4),
        ("{\n    edges = 5\n    y = c |> why |> edges\n}", 4),
        (
            "class R:\n    def m(edges) => c |> why |> edges\nR().m(1)",
            6,
        ),
        (
            "class Q(edges):\n    this.v = (c |> why |> edges).length\nQ(1)",
            6,
        ),
        (
            "try:\n    throw 1\ncatch edges:\n    y = c |> why |> edges",
            4,
        ),
        ("{edges} = {edges: 1}\nc |> why |> edges", 5),
    ];
    for (binding, line) in bindings {
        assert_eq!(
            failure(&format!("{CHAIN}{binding}")),
            format!("TypeError: edges is not an operation (line {line}, column 1)"),
            "{binding}"
        );
    }
    assert_eq!(
        failure(&format!("{CHAIN}stated = 1\nc |> why |> stated |> edges")),
        "TypeError: stated is not an operation (line 5, column 1)"
    );
    for (binding, expected) in [
        (
            "edges = 1\nc |> why |> edges |> why",
            "TypeError: Cannot explain c|>why|>edges (line 5, column 1)",
        ),
        (
            "holds = 1\nc |> why |> holds |> edges",
            "TypeError: Cannot explain c|>why|>holds (line 5, column 1)",
        ),
        (
            "edges = 1\nd = a + 10\nc |> why |> intersect(d |> why |> edges)",
            "TypeError: Cannot explain d|>why|>edges (line 6, column 1)",
        ),
    ] {
        assert_eq!(failure(&format!("{CHAIN}{binding}")), expected, "{binding}");
    }
    assert_eq!(
        result(&format!(
            "{CHAIN}class K:\n    def edges() => 1\no = {{edges: 1}}\nq = {{}}\nq.edges = 2\n[c |> why |> edges, o.edges, q.edges, K().edges(), (c |> why)[0].from]"
        )),
        "[[[\"b\",\"c\"],[\"a\",\"b\"]],1.0,2.0,1.0,[\"b\"]]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}why = 1\naffects = 2\n[c |> why |> edges, a |> affects |> edges]"
        )),
        "[[[\"b\",\"c\"],[\"a\",\"b\"]],[[\"a\",\"b\"],[\"b\",\"c\"]]]"
    );
    for (binding, expected) in [
        (
            "nodes = 1\nc |> why |> edges",
            "[[\"b\",\"c\"],[\"a\",\"b\"]]",
        ),
        (
            "holds = 1\nc |> why |> edges",
            "[[\"b\",\"c\"],[\"a\",\"b\"]]",
        ),
        ("edges = 1\nc |> why |> nodes", "[\"c\",\"b\",\"a\"]"),
        ("edges = 1\nc |> why |> holds", "[6.0,3.0,1.0]"),
    ] {
        assert_eq!(result(&format!("{CHAIN}{binding}")), expected, "{binding}");
    }
    assert_eq!(
        result(&format!("{CHAIN}edges = 1\n{}", accepted("c |> why"))),
        "[[\"b\",\"c\"],[\"a\",\"b\"]]"
    );
}

#[test]
fn a_try_now_catches_what_was_a_parse_error() {
    assert_eq!(
        result(&format!(
            "{CHAIN}edges = 5\nr = \"none\"\ntry:\n    r = c |> why |> edges\ncatch error:\n    r = error\nr"
        )),
        "\"TypeError: edges is not an operation\""
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}try:\n    r = c |> why |> edges\ncatch e:\n    r = \"caught\"\nr"
        )),
        "[[\"b\",\"c\"],[\"a\",\"b\"]]"
    );
    assert_eq!(
        failure(&format!("{CHAIN}missing\nedges = 1\nc |> why |> edges")),
        "ReferenceError: missing is not defined (line 4, column 1)"
    );
    assert_eq!(
        failure(&format!("{CHAIN}edges = 1\nc |> why |> edges\nmissing")),
        "TypeError: edges is not an operation (line 5, column 1)"
    );
    let session = repl(
        "x = 5\nedges = 1\nx |> edges\nx\n.exit\n",
        &["--plain", "--json"],
    );
    assert!(session.status.success(), "{session:?}");
    assert_eq!(
        String::from_utf8(session.stderr).unwrap(),
        "TypeError: edges is not an operation (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(session.stdout).unwrap();
    assert!(stdout.ends_with("5.0\n1.0\n5.0\n"), "{stdout}");
}

#[test]
fn a_binding_declared_after_a_standing_declaration_fails_its_refresh() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{CHAIN}wiring = c |> why |> edges"));
    checked(&mut runtime, "edges = 5");
    assert_eq!(
        outcome(&mut runtime, "wiring"),
        "[[\"b\",\"c\"],[\"a\",\"b\"]]"
    );
    assert_eq!(
        outcome(&mut runtime, "a = 3"),
        "TypeError: edges is not an operation (line 1, column 1)"
    );
    assert_eq!(outcome(&mut runtime, "a"), "1.0");
    assert_eq!(
        outcome(&mut runtime, "wiring"),
        "[[\"b\",\"c\"],[\"a\",\"b\"]]"
    );
    checked(&mut runtime, "delete edges\nx = 3\nb = x + 1");
    assert_eq!(
        outcome(&mut runtime, "wiring"),
        "[[\"b\",\"c\"],[\"x\",\"b\"]]"
    );
}

#[test]
fn the_backtracking_forms_now_destructure_the_edges() {
    for (written, expected) in [
        (
            "x, y = c |> why |> edges\n[x, y]",
            "[[\"b\",\"c\"],[\"a\",\"b\"]]",
        ),
        (
            "x, y = [c |> why |> edges, 1]\n[x, y]",
            "[[[\"b\",\"c\"],[\"a\",\"b\"]],1.0]",
        ),
        ("{length} = c |> why |> edges\nlength", "2.0"),
        ("{x, y} = c |> why |> edges\n[x, y]", "[null,null]"),
        ("x, = c |> why |> edges\nx", "[\"b\",\"c\"]"),
    ] {
        let spelling = written.replace("c |> why |> edges", &accepted("c |> why"));
        assert_eq!(result(&format!("{CHAIN}{written}")), expected, "{written}");
        assert_eq!(
            result(&format!("{CHAIN}{spelling}")),
            expected,
            "{spelling}"
        );
    }
    behaves_as_spelled(&[
        (
            format!("{CHAIN}x, y = c |> why |> edges\n[x, y]"),
            format!("{CHAIN}x, y = {}\n[x, y]", accepted("c |> why")),
        ),
        (
            "z = 9\nb = z + 1\n[x, y]".to_string(),
            "z = 9\nb = z + 1\n[x, y]".to_string(),
        ),
        ("z = 2\n[x, y]".to_string(), "z = 2\n[x, y]".to_string()),
    ]);
}

#[test]
fn standing_declarations_follow_the_graph_as_their_accepted_spelling_does() {
    let mut compared = 0;
    for (setup, names, writes) in GRAPHS {
        for x in names {
            for (form, selection) in FORMS {
                let y = names[0];
                let written = format!("{setup}wiring = {}\nwiring", spelled(form, x, y));
                let spelling = format!(
                    "{setup}wiring = {}\nwiring",
                    accepted(&spelled(selection, x, y))
                );
                let mut steps = vec![(written, spelling)];
                for write in writes {
                    steps.push((format!("{write}\nwiring"), format!("{write}\nwiring")));
                }
                behaves_as_spelled(&steps);
                compared += 1;
            }
        }
    }
    assert_eq!(compared, 650);
}

#[test]
fn edges_follow_rewirings_and_stay_through_value_changes() {
    behaves_as_spelled(&[
        (
            format!(
                "{CHAIN}wiring = c |> why |> edges\nfacts = c |> why |> stated |> edges\nwiring"
            ),
            format!(
                "{CHAIN}wiring = {}\nfacts = {}\nwiring",
                accepted("c |> why"),
                accepted("c |> why |> stated")
            ),
        ),
        (
            "a = 5\n[wiring, facts]".to_string(),
            "a = 5\n[wiring, facts]".to_string(),
        ),
        (
            "c = b * 3\n[wiring, facts]".to_string(),
            "c = b * 3\n[wiring, facts]".to_string(),
        ),
        (
            "x = 2\nb = a + x\n[wiring, facts]".to_string(),
            "x = 2\nb = a + x\n[wiring, facts]".to_string(),
        ),
        (
            "x = 4\n[wiring, facts]".to_string(),
            "x = 4\n[wiring, facts]".to_string(),
        ),
        (
            "delete x\n[wiring, facts]".to_string(),
            "delete x\n[wiring, facts]".to_string(),
        ),
    ]);
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}wiring = c |> why |> edges\nfacts = c |> why |> stated |> edges"),
    );
    for (write, expected) in [
        ("a = 5", "[[[\"b\",\"c\"],[\"a\",\"b\"]],[]]"),
        (
            "c = b + a",
            "[[[\"b\",\"c\"],[\"a\",\"c\"],[\"a\",\"b\"]],[]]",
        ),
        (
            "x = 2\nb = a + x",
            "[[[\"b\",\"c\"],[\"a\",\"c\"],[\"a\",\"b\"],[\"x\",\"b\"]],[]]",
        ),
        (
            "x = 3",
            "[[[\"b\",\"c\"],[\"a\",\"c\"],[\"a\",\"b\"],[\"x\",\"b\"]],[]]",
        ),
        (
            "b = x * 2",
            "[[[\"b\",\"c\"],[\"a\",\"c\"],[\"x\",\"b\"]],[]]",
        ),
        ("delete b", "[[[\"b\",\"c\"],[\"a\",\"c\"]],[]]"),
    ] {
        checked(&mut runtime, write);
        assert_eq!(
            outcome(&mut runtime, "[wiring, facts]"),
            expected,
            "{write}"
        );
    }
}

#[test]
fn value_snapshots_suppose_and_rollback_behave_as_the_accepted_spelling() {
    behaves_as_spelled(&[
        (
            format!(
                "{CHAIN}wiring = c |> why |> edges\nsnap = wiring.value\nx = 5\nb = x + 1\n[wiring, snap]"
            ),
            format!(
                "{CHAIN}wiring = {}\nsnap = wiring.value\nx = 5\nb = x + 1\n[wiring, snap]",
                accepted("c |> why")
            ),
        ),
        (
            "frozen = (c |> why |> edges).value\ny = 8\nb = x + y\n[frozen, wiring]".to_string(),
            format!(
                "frozen = {}.value\ny = 8\nb = x + y\n[frozen, wiring]",
                accepted("c |> why")
            ),
        ),
        (
            "suppose x = 5:\n    inner = c |> why |> edges\n    [inner, c |> why |> diff |> edges]"
                .to_string(),
            format!(
                "suppose x = 5:\n    inner = {}\n    [inner, {}]",
                accepted("c |> why"),
                accepted("c |> why |> diff")
            ),
        ),
        ("wiring".to_string(), "wiring".to_string()),
        (
            "try:\n    z = 4\n    b = z + a\n    assert(wiring, [[\"b\", \"c\"], [\"z\", \"b\"], [\"a\", \"b\"]])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nwiring"
                .to_string(),
            "try:\n    z = 4\n    b = z + a\n    assert(wiring, [[\"b\", \"c\"], [\"z\", \"b\"], [\"a\", \"b\"]])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nwiring"
                .to_string(),
        ),
        ("b = a + missing".to_string(), "b = a + missing".to_string()),
        ("wiring".to_string(), "wiring".to_string()),
        (
            "facts = c |> why |> stated |> edges\ntry:\n    facts = 1 |> edges\ncatch error:\n    failure = error\n[failure, facts]"
                .to_string(),
            format!(
                "facts = {}\ntry:\n    facts = {}\ncatch error:\n    failure = error\n[failure, facts]",
                accepted("c |> why |> stated"),
                accepted("1 |> why |> depth(0)")
            ),
        ),
        (
            "x = 6\nb = x + 2\n[wiring, facts]".to_string(),
            "x = 6\nb = x + 2\n[wiring, facts]".to_string(),
        ),
    ]);
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose b = a * 4:\n    inner = c |> why |> edges\n    [inner, c |> why |> diff |> edges]"
        )),
        "[[[\"b\",\"c\"],[\"a\",\"b\"]],[[\"b\",\"c\"],[\"a\",\"b\"]]]"
    );
}

#[test]
fn declarations_that_hold_the_stage_are_reasoning_observers() {
    behaves_as_spelled(&[
        (
            format!(
                "{CHAIN}wiring = c |> why |> edges\n[wiring |> why |> edges, a |> affects |> edges, model |> edges]"
            ),
            format!(
                "{CHAIN}wiring = {}\n[{}, {}, {}]",
                accepted("c |> why"),
                accepted("wiring |> why"),
                accepted("a |> affects"),
                accepted("model |> depth(0)")
            ),
        ),
        (
            "count = wiring.length\n[model |> edges, count |> why |> edges]".to_string(),
            format!(
                "count = wiring.length\n[{}, {}]",
                accepted("model |> depth(0)"),
                accepted("count |> why")
            ),
        ),
        (
            "if (c |> why |> edges).length > 1:\n    flag = true\n[flag, model |> edges]"
                .to_string(),
            format!(
                "if {}.length > 1:\n    flag = true\n[flag, {}]",
                accepted("c |> why"),
                accepted("model |> depth(0)")
            ),
        ),
        (
            "class P(v):\n    this.v = v\n$P.basis = $P.v |> why |> edges\n$P.w = $P.v * a\n$P.links = $P.w |> why |> edges\np = P(1)\n[model |> edges, p.basis, p.links]"
                .to_string(),
            format!(
                "class P(v):\n    this.v = v\n$P.basis = {}\n$P.w = $P.v * a\n$P.links = {}\np = P(1)\n[{}, p.basis, p.links]",
                accepted("$P.v |> why"),
                accepted("$P.w |> why"),
                accepted("model |> depth(0)")
            ),
        ),
        (
            "def f() => c |> why |> edges\nlist = f()\n[list, f()]".to_string(),
            format!(
                "def f() => {}\nlist = f()\n[list, f()]",
                accepted("c |> why")
            ),
        ),
        (
            "x = 8\nb = x + 1\n[list, f()]".to_string(),
            "x = 8\nb = x + 1\n[list, f()]".to_string(),
        ),
        (
            "q = P(4)\na = 3\n[q.links, p.links]".to_string(),
            "q = P(4)\na = 3\n[q.links, p.links]".to_string(),
        ),
        (
            "class W(v):\n    this.v = v\n    def links() => this.v |> why |> edges\nw = W(2)\nw.links()"
                .to_string(),
            format!(
                "class W(v):\n    this.v = v\n    def links() => {}\nw = W(2)\nw.links()",
                accepted("this.v |> why")
            ),
        ),
        (
            "g = s -> (s |> why |> edges).length\ng(c)".to_string(),
            format!("g = s -> {}.length\ng(c)", accepted("s |> why")),
        ),
    ]);
    assert_eq!(
        result(&format!(
            "{CHAIN}wiring = c |> why |> edges\ncount = wiring.length\n[wiring |> why |> edges, model |> edges]"
        )),
        "[[],[[\"a\",\"b\"],[\"b\",\"c\"],[\"wiring\",\"count\"]]]"
    );
}

#[test]
fn undefined_selections_stay_undefined() {
    behaves_as_spelled(&[
        (
            format!(
                "{CHAIN}n = null\nwiring = c |> why |> depth(n) |> edges\n[wiring, (c |> why |> depth(n) |> edges).length]"
            ),
            format!(
                "{CHAIN}n = null\nwiring = {}\n[wiring, {}.length]",
                accepted("c |> why |> depth(n)"),
                accepted("c |> why |> depth(n)")
            ),
        ),
        ("n = 1\nwiring".to_string(), "n = 1\nwiring".to_string()),
        (
            "n = undefined\nwiring".to_string(),
            "n = undefined\nwiring".to_string(),
        ),
    ]);
    assert_eq!(
        result(&format!("{CHAIN}c |> why |> depth(null) |> edges")),
        "null"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}[`v ${{c |> why |> depth(null) |> edges}}`, typeof (c |> why |> depth(null) |> edges)]"
        )),
        "[\"v undefined\",\"undefined\"]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}[`v ${{{}}}`, typeof {}]",
            accepted("c |> why |> depth(null)"),
            accepted("c |> why |> depth(null)")
        )),
        "[\"v undefined\",\"undefined\"]"
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}n = null\nfor s in c |> why |> depth(n) |> edges:\n    r = s"
        )),
        "TypeError: c|>why|>depth(n)|>edges is not iterable (line 5, column 1)"
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}n = null\nfor s in {}:\n    r = s",
            accepted("c |> why |> depth(n)")
        )),
        "TypeError: c|>why|>depth(n).flatMap(function()) is not iterable (line 5, column 1)"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}d = a + 10\nc |> why |> union(d |> why |> depth(null)) |> edges"
        )),
        "null"
    );
}

#[test]
fn limits_in_the_source_freeze_and_arguments_that_end_with_edges_stay_as_written() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}n = 1\nwiring = c |> why |> depth(n.value) |> edges\nn = 0"),
    );
    assert_eq!(
        outcome(&mut runtime, "wiring"),
        "[[\"b\",\"c\"],[\"a\",\"b\"]]"
    );
    assert_eq!(stored(&runtime, "wiring"), "c|>why|>depth(1)|>edges");
    checked(&mut runtime, "x = 4\nb = x + 1");
    assert_eq!(
        outcome(&mut runtime, "wiring"),
        "[[\"b\",\"c\"],[\"x\",\"b\"]]"
    );
    let mut spelling = Runtime::new();
    checked(
        &mut spelling,
        &format!(
            "{CHAIN}n = 1\nwiring = {}\nn = 0",
            accepted("c |> why |> depth(n.value)")
        ),
    );
    assert_eq!(
        outcome(&mut spelling, "wiring"),
        "[[\"b\",\"c\"],[\"a\",\"b\"]]"
    );
    for (written, expected) in [
        (
            "d = a + 10\nv = c |> why |> intersect(d |> why |> depth(m.value) |> edges)"
                .to_string(),
            "TypeError: Cannot explain d|>why|>depth(m.value)|>edges (line 5, column 1)",
        ),
        (
            "d = a + 10\nv = c |> why |> union(d |> why |> depth(m.value) |> edges)".to_string(),
            "TypeError: Cannot explain d|>why|>depth(m.value)|>edges (line 5, column 1)",
        ),
        (
            format!(
                "d = a + 10\nv = c |> why |> intersect({})",
                accepted("d |> why |> depth(m.value)")
            ),
            "TypeError: Cannot explain d|>why|>depth(m.value).flatMap(function()) (line 5, column 1)",
        ),
        (
            "v = c |> why |> depth(m.value) |> edges |> depth(1)".to_string(),
            "TypeError: Cannot explain c|>why|>depth(m.value)|>edges (line 4, column 1)",
        ),
        (
            "v = c |> why |> depth(m.value) |> edges |> edges".to_string(),
            "TypeError: Cannot explain c|>why|>depth(m.value)|>edges (line 4, column 1)",
        ),
        (
            "v = c |> why |> depth(m.value) |> edges".to_string(),
            "ReferenceError: m is not defined (line 4, column 1)",
        ),
        (
            format!("v = {}", accepted("c |> why |> depth(m.value)")),
            "ReferenceError: m is not defined (line 4, column 1)",
        ),
    ] {
        assert_eq!(failure(&format!("{CHAIN}{written}")), expected, "{written}");
    }
    let mut combined = Runtime::new();
    checked(
        &mut combined,
        &format!(
            "{CHAIN}d = a + 10\nk = 1\nv = c |> why |> intersect(d |> why |> depth(k.value)) |> edges\nk = 0"
        ),
    );
    assert_eq!(
        stored(&combined, "v"),
        "c|>why|>intersect(d|>why|>depth(1))|>edges"
    );
    assert_eq!(outcome(&mut combined, "v"), "[]");
    let mut joined = Runtime::new();
    checked(
        &mut joined,
        &format!(
            "{CHAIN}d = a + 10\nk = 1\nv = c |> why |> union(d |> why |> depth(k.value)) |> edges\nk = 0"
        ),
    );
    assert_eq!(
        stored(&joined, "v"),
        "c|>why|>union(d|>why|>depth(1))|>edges"
    );
    assert_eq!(
        outcome(&mut joined, "v"),
        "[[\"b\",\"c\"],[\"a\",\"b\"],[\"a\",\"d\"]]"
    );
}

#[test]
fn the_graph_stays_bounded_where_the_accepted_spelling_grows() {
    let mut stage = Runtime::new();
    let mut spelling = Runtime::new();
    checked(&mut stage, &format!("{CHAIN}wiring = c |> why |> edges"));
    checked(
        &mut spelling,
        &format!("{CHAIN}wiring = (why c).flatMap(s -> {DETOUR})"),
    );
    assert_eq!((stage.graph.len(), spelling.graph.len()), (4, 10));
    for write in 0..50 {
        let source = format!("a = {}", write + 2);
        checked(&mut stage, &source);
        checked(&mut spelling, &source);
        assert_eq!(stage.graph.len(), 4);
        assert_eq!(spelling.graph.len(), 10 + 6 * (write + 1));
        assert_eq!(pending(&stage), 0);
        assert_eq!(pending(&spelling), 6 * (write + 2));
        assert_eq!(reads(&stage, "wiring"), ["a", "b", "c"]);
        assert_eq!(reads(&stage, "wiring"), reads(&spelling, "wiring"));
        assert_eq!(observes(&stage, "wiring"), observes(&spelling, "wiring"));
        assert_eq!(
            outcome(&mut stage, "wiring"),
            outcome(&mut spelling, "wiring")
        );
    }
    let mut rewired = Runtime::new();
    checked(&mut rewired, &format!("{CHAIN}wiring = c |> why |> edges"));
    for write in 0..20 {
        let source = format!("x{write} = {write}\nb = a + x{write}");
        checked(&mut rewired, &source);
        assert_eq!(rewired.graph.len(), 5 + write);
        assert_eq!(pending(&rewired), 0);
    }
    assert_eq!(
        outcome(&mut rewired, "wiring"),
        "[[\"b\",\"c\"],[\"a\",\"b\"],[\"x19\",\"b\"]]"
    );
}

#[test]
fn the_records_number_later_anonymous_objects_as_the_accepted_spelling_does() {
    let file = SourceFile::new("numbering");
    for (prelude, written, spelling, expected) in [
        (
            CHAIN.to_string(),
            "x = c |> why |> edges\np = {k: 1}\nq = p.value\n[String(p), `${p}`, (why q)[0].rule]"
                .to_string(),
            format!(
                "x = {}\np = {{k: 1}}\nq = p.value\n[String(p), `${{p}}`, (why q)[0].rule]",
                accepted("c |> why")
            ),
            "[\"[object obj:3]\",\"[object obj:3]\",\"q = obj:3\"]",
        ),
        (
            CHAIN.to_string(),
            "x = model |> edges\np = {k: 1}\nString(p)".to_string(),
            format!(
                "x = {}\np = {{k: 1}}\nString(p)",
                accepted("model |> depth(0)")
            ),
            "\"[object obj:3]\"",
        ),
        (
            CHAIN.to_string(),
            "x = c |> why |> stated |> edges\ny = c |> why |> nodes\np = {k: 1}\nString(p)"
                .to_string(),
            format!(
                "x = {}\ny = (c |> why).map(step => step.node)\np = {{k: 1}}\nString(p)",
                accepted("c |> why |> stated")
            ),
            "\"[object obj:4]\"",
        ),
        (
            SIX.to_string(),
            "x = c |> why |> edges\np = {k: 1}\nString(p)".to_string(),
            format!("x = {}\np = {{k: 1}}\nString(p)", accepted("c |> why")),
            "\"[object obj:3]\"",
        ),
        (
            String::new(),
            "p = {k: 1}\nString(p)".to_string(),
            "p = {k: 1}\nString(p)".to_string(),
            "\"[object obj:0]\"",
        ),
    ] {
        let stage = file.run(&format!("{prelude}{written}"), &["--json"]);
        let accepted = file.run(&format!("{prelude}{spelling}"), &["--json"]);
        assert!(stage.status.success(), "{written} {stage:?}");
        assert_eq!(stage.stdout, accepted.stdout, "{written}");
        assert_eq!(
            String::from_utf8(stage.stdout).unwrap(),
            format!("{expected}\n"),
            "{written}"
        );
    }
}

#[test]
fn no_null_or_undefined_value_reaches_the_pairs() {
    let model = accepted("model |> depth(0)");
    behaves_as_spelled(&[
        (
            format!("{CLASSY}none = null\nwiring = model |> edges\n[wiring, model |> edges]"),
            format!("{CLASSY}none = null\nwiring = {model}\n[wiring, {model}]"),
        ),
        (
            "j = M(null)\n[wiring, j.wear, j |> edges]".to_string(),
            format!(
                "j = M(null)\n[wiring, j.wear, {}]",
                accepted("j |> why |> depth(0)")
            ),
        ),
        (
            "$M.basis = $M.load |> why |> edges\nk = M(null)\n[m.basis, k.basis]".to_string(),
            format!(
                "$M.basis = {}\nk = M(null)\n[m.basis, k.basis]",
                accepted("$M.load |> why")
            ),
        ),
        (
            format!(
                "{CHAIN}x = 2\nb = a + x\nfacts = c |> why |> stated |> edges\nvalues = c |> why |> edges\n[facts, values]"
            ),
            format!(
                "{CHAIN}x = 2\nb = a + x\nfacts = {}\nvalues = {}\n[facts, values]",
                accepted("c |> why |> stated"),
                accepted("c |> why")
            ),
        ),
        (
            "delete x\n[facts, values, c |> why |> edges]".to_string(),
            format!("delete x\n[facts, values, {}]", accepted("c |> why")),
        ),
        (
            "x = 3\n[facts, values]".to_string(),
            "x = 3\n[facts, values]".to_string(),
        ),
    ]);
    assert_eq!(
        result(&format!(
            "{CLASSY}none = null\nwiring = model |> edges\nheld = model |> holds\n[wiring, held]"
        )),
        "[[[\"m.load\",\"m.wear\"],[\"rate\",\"m.wear\"]],null]"
    );
    let undefined = "o = {k: 1}\np = {x: 0}\np.x = 1\nf = v -> p.x = v\nf(o.missing)\n";
    behaves_as_spelled(&[
        (
            format!("{undefined}names = p.x |> edges\n[names, p.x |> edges, p.x |> holds]"),
            format!(
                "{undefined}names = {}\n[names, {}, (p.x |> depth(0)).map(step => step.holds)]",
                accepted("p.x |> depth(0)"),
                accepted("p.x |> depth(0)")
            ),
        ),
        (
            "class T(n):\n    this.n = n\nif $T.n > 0 and (p.x |> edges).length == 0:\n    $T.flag = true\nt = T(1)\nt.flag"
                .to_string(),
            format!(
                "class T(n):\n    this.n = n\nif $T.n > 0 and {}.length == 0:\n    $T.flag = true\nt = T(1)\nt.flag",
                accepted("p.x |> depth(0)")
            ),
        ),
    ]);
    assert_eq!(
        result(&format!(
            "{undefined}class T(n):\n    this.n = n\nif $T.n > 0 and (p.x |> edges).length == 0:\n    $T.flag = true\nt = T(1)\n[t.flag, p.x |> edges]"
        )),
        "[true,[]]"
    );
}

#[test]
fn rendering_shows_the_stage_as_written() {
    for (source, rendered) in [
        ("c |> why |> edges", "c|>why|>edges"),
        ("why c |> stated |> edges", "c|>why|>stated|>edges"),
        ("model |> edges", "model|>edges"),
        (
            "c |> why |> intersect(d |> why) |> edges",
            "c|>why|>intersect(d|>why)|>edges",
        ),
        (
            "c |> why |> union(d |> why) |> edges",
            "c|>why|>union(d|>why)|>edges",
        ),
        ("c |> why |> origins |> edges", "c|>why|>origins|>edges"),
        ("c |> why |> edges |> why", "c|>why|>edges|>why"),
        ("c |> why |> holds |> edges", "c|>why|>holds|>edges"),
        ("(c |> why |> edges).length", "c|>why|>edges.length"),
        ("(c |> why |> nodes).length", "c|>why|>nodes.length"),
    ] {
        let parsed = parse_expression(source).unwrap();
        assert_eq!(parsed.to_string(), rendered, "{source}");
    }
    for source in [
        "c |> why |> edges",
        "c |> edges",
        "c |> why |> depth(1) |> edges",
        "c |> why |> edges |> why",
        "model |> edges",
        "c |> affects |> outcomes |> edges",
    ] {
        let parsed = parse_expression(source).unwrap();
        assert_eq!(
            parse_expression(&parsed.to_string()).unwrap(),
            parsed,
            "{source}"
        );
    }
    assert_eq!(
        generate(&parse("def f(x = 1) => c |> why |> edges").unwrap()[0]),
        "def f(x=1):\n    return c|>why|>edges"
    );
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{CHAIN}wiring = c |> why |> edges"));
    assert_eq!(stored(&runtime, "wiring"), "c|>why|>edges");
    checked(&mut runtime, "count = wiring.length");
    assert_eq!(outcome(&mut runtime, "(why count)[0].from"), "[\"wiring\"]");
    assert_eq!(
        failure(&format!("{CHAIN}delete c |> why |> edges")),
        "SyntaxError: Cannot delete c|>why|>edges (line 4, column 1)"
    );
    assert_eq!(
        failure(&format!("{CHAIN}delete (c |> why).flatMap(s -> s.from)")),
        "SyntaxError: Cannot delete c|>why.flatMap(function()) (line 4, column 1)"
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-edge-lists-{label}-{}.nuc",
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

const PROGRAM: &str = "a = 1\nb = a + 2\nc = b * 2\nwiring = c |> why |> edges\nfacts = c |> why |> stated |> edges\nx = 5\nb = x + 1\n[wiring, facts, a |> affects |> edges]";

fn program_spelled() -> String {
    format!(
        "a = 1\nb = a + 2\nc = b * 2\nwiring = {}\nfacts = {}\nx = 5\nb = x + 1\n[wiring, facts, {}]",
        accepted("c |> why"),
        accepted("c |> why |> stated"),
        accepted("a |> affects")
    )
}

#[test]
fn files_graphs_and_repl_sessions_give_edges() {
    let file = SourceFile::new("file");
    let spelled_program = program_spelled();
    for options in [&[][..], &["--json"][..]] {
        let stage = file.run(PROGRAM, options);
        let spelling = file.run(&spelled_program, options);
        assert!(stage.status.success(), "{options:?} {stage:?}");
        assert!(stage.stderr.is_empty(), "{options:?} {stage:?}");
        assert_eq!(stage.stdout, spelling.stdout, "{options:?}");
    }
    assert_eq!(
        String::from_utf8(file.run(PROGRAM, &["--json"]).stdout).unwrap(),
        "[[[\"b\",\"c\"],[\"x\",\"b\"]],[],[]]\n"
    );
    assert_eq!(
        String::from_utf8(file.run(PROGRAM, &[]).stdout).unwrap(),
        "b,c,x,b,,\n"
    );
    for options in [&["--graph"][..], &["--json", "--graph"][..]] {
        let stage = String::from_utf8(file.run(PROGRAM, options).stdout).unwrap();
        let spelling = String::from_utf8(file.run(&spelled_program, options).stdout).unwrap();
        assert!(!stage.contains("obj:"), "{stage}");
        assert!(spelling.contains("obj:"), "{spelling}");
        assert_eq!(
            without_record_reads(&stage),
            without_record_reads(&spelling),
            "{options:?}"
        );
        assert!(
            stage.contains("  wiring [variable]\n    reads    c, b, x\n    observes c, b, x"),
            "{stage}"
        );
    }
    for (source, message) in [
        (
            "a = 1\nedges = 2\na |> edges\n",
            "TypeError: edges is not an operation (line 3, column 1)\n",
        ),
        (
            "a = 1\na |> edges |> why\n",
            "TypeError: Cannot explain a|>edges (line 2, column 1)\n",
        ),
    ] {
        let failed = file.run(source, &["--json"]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(String::from_utf8(failed.stderr).unwrap(), message);
    }
    let session = "a = 1\nb = a + 2\nc = b * 2\nc |> why |> edges\nwiring = c |> why |> derived |> edges\nwiring\nx = 5\nb = x + 1\nwiring\n.exit\n";
    let session_spelled = session
        .replace("c |> why |> edges", &accepted("c |> why"))
        .replace(
            "c |> why |> derived |> edges",
            &accepted("c |> why |> derived"),
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
                "[[\"b\",\"c\"],[\"a\",\"b\"]]\n[[\"b\",\"c\"],[\"a\",\"b\"]]\n"
            } else {
                "b,c,a,b\nb,c,a,b\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
        }
    }
}

#[test]
fn long_chains_of_edges_stages_raise_without_recursing() {
    let stages = " |> edges".repeat(120);
    let source = format!("{CHAIN}c |> why{stages}");
    let quoted = format!("c|>why{}", "|>edges".repeat(119));
    assert_eq!(
        failure(&source),
        format!("TypeError: Cannot explain {quoted} (line 4, column 1)")
    );
    let declared = format!("{CHAIN}v = c |> why{stages}");
    assert_eq!(
        failure(&declared),
        format!("TypeError: Cannot explain {quoted} (line 4, column 1)")
    );
    let alternating = " |> nodes |> holds |> edges".repeat(40);
    let mixed = format!("{CHAIN}c |> why{alternating}");
    let mixed_quoted = format!("c|>why{}|>nodes|>holds", "|>nodes|>holds|>edges".repeat(39));
    assert_eq!(
        failure(&mixed),
        format!("TypeError: Cannot explain {mixed_quoted} (line 4, column 1)")
    );
}

#[test]
fn the_nodes_and_holds_stages_keep_their_accepted_behavior() {
    assert_eq!(
        result(&format!(
            "{CHAIN}[c |> why |> nodes, c |> nodes, model |> nodes, a |> affects |> nodes]"
        )),
        "[[\"c\",\"b\",\"a\"],[\"c\"],[\"a\",\"b\",\"c\"],[\"b\",\"c\"]]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}[c |> why |> holds, c |> holds, model |> holds, a |> affects |> holds]"
        )),
        "[[6.0,3.0,1.0],[6.0],[1.0,3.0,6.0],[3.0,6.0]]"
    );
    for (source, expected) in [
        (
            "nodes = 1\nc |> why |> nodes",
            "TypeError: nodes is not an operation (line 5, column 1)",
        ),
        (
            "holds = 1\nc |> why |> holds",
            "TypeError: holds is not an operation (line 5, column 1)",
        ),
        (
            "c |> why |> nodes |> why",
            "TypeError: Cannot explain c|>why|>nodes (line 4, column 1)",
        ),
        (
            "c |> why |> holds |> why",
            "TypeError: Cannot explain c|>why|>holds (line 4, column 1)",
        ),
    ] {
        assert_eq!(failure(&format!("{CHAIN}{source}")), expected, "{source}");
    }
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{CHAIN}n = 1\nnames = c |> why |> depth(n.value) |> nodes\nvalues = c |> why |> depth(n.value) |> holds\nn = 2"
        ),
    );
    assert_eq!(stored(&runtime, "names"), "c|>why|>depth(1)|>nodes");
    assert_eq!(stored(&runtime, "values"), "c|>why|>depth(1)|>holds");
    assert_eq!(
        outcome(&mut runtime, "[names, values]"),
        "[[\"c\",\"b\"],[6.0,3.0]]"
    );
    let mut stage = Runtime::new();
    let mut spelling = Runtime::new();
    checked(
        &mut stage,
        &format!("{CHAIN}names = c |> why |> nodes\nvalues = c |> why |> holds"),
    );
    checked(
        &mut spelling,
        &format!("{CHAIN}names = (why c).map(.node)\nvalues = (why c).map(.holds)"),
    );
    checked(&mut stage, "a = 2");
    checked(&mut spelling, "a = 2");
    assert_eq!((stage.graph.len(), spelling.graph.len()), (5, 17));
    assert_eq!(
        outcome(&mut stage, "[names, values]"),
        outcome(&mut spelling, "[names, values]")
    );
    let nulls = format!("{CLASSY}none = null\n");
    behaves_as_spelled(&[
        (
            format!(
                "{nulls}names = model |> nodes\nfacts = none |> nodes\nvalues = model |> holds\ninner = none |> holds\n[names, facts, values, inner]"
            ),
            format!(
                "{nulls}names = (model |> depth(0)).map(step => step.node)\nfacts = (none |> depth(0)).map(step => step.node)\nvalues = (model |> depth(0)).map(step => step.holds)\ninner = (none |> depth(0)).map(step => step.holds)\n[names, facts, values, inner]"
            ),
        ),
        (
            "none = 2\nj = M(null)\n[names, facts, values, inner]".to_string(),
            "none = 2\nj = M(null)\n[names, facts, values, inner]".to_string(),
        ),
    ]);
    assert_eq!(
        result(&format!(
            "{nulls}names = model |> nodes\nfacts = none |> nodes\nvalues = model |> holds\nheld = none |> holds\n[names, facts, values, held]"
        )),
        "[[\"rate\",\"$M\",\"m\",\"m.load\",\"m.wear\",\"none\"],[\"none\"],null,null]"
    );
}

#[test]
fn the_compatibility_rows_give_their_documented_results() {
    let changed = [
        ("c |> why |> edges", "[[\"b\",\"c\"],[\"a\",\"b\"]]"),
        ("why c |> edges", "[[\"b\",\"c\"],[\"a\",\"b\"]]"),
        ("c |> edges", "[[\"b\",\"c\"]]"),
        (
            "model |> edges",
            "[[\"a\",\"b\"],[\"b\",\"c\"],[\"a\",\"d\"],[\"e\",\"f\"],[\"d\",\"f\"]]",
        ),
        (
            "a |> affects |> edges",
            "[[\"a\",\"b\"],[\"a\",\"d\"],[\"e\",\"f\"],[\"d\",\"f\"],[\"b\",\"c\"]]",
        ),
        (
            "affects a |> edges",
            "[[\"a\",\"b\"],[\"a\",\"d\"],[\"e\",\"f\"],[\"d\",\"f\"],[\"b\",\"c\"]]",
        ),
        ("c |> why |> stated |> edges", "[]"),
        (
            "c |> why |> derived |> edges",
            "[[\"b\",\"c\"],[\"a\",\"b\"]]",
        ),
        (
            "c |> why |> depth(1) |> edges",
            "[[\"b\",\"c\"],[\"a\",\"b\"]]",
        ),
        (
            "c |> why |> path(a) |> edges",
            "[[\"b\",\"c\"],[\"a\",\"b\"]]",
        ),
        ("c |> why |> intersect(f |> why) |> edges", "[]"),
        (
            "c |> why |> except(f |> why) |> edges",
            "[[\"b\",\"c\"],[\"a\",\"b\"]]",
        ),
        (
            "c |> why |> union(f |> why) |> edges",
            "[[\"b\",\"c\"],[\"a\",\"b\"],[\"e\",\"f\"],[\"d\",\"f\"],[\"a\",\"d\"]]",
        ),
        ("c |> why |> origins |> edges", "[]"),
        ("c |> why |> outcomes |> edges", "[[\"b\",\"c\"]]"),
        ("c |> why |> depth(null) |> edges", "null"),
        ("g = a + a\ng |> edges", "[[\"a\",\"g\"]]"),
        (
            "1 |> edges",
            "TypeError: Cannot explain 1 (line 7, column 1)",
        ),
        (
            "missing |> why |> edges",
            "ReferenceError: missing is not defined (line 7, column 1)",
        ),
        (
            "c |> why |> diff |> edges",
            "TypeError: Cannot use 'diff' outside suppose (line 7, column 1)",
        ),
        (
            "n = null\nn.p |> edges",
            "ReferenceError: n is not defined (line 8, column 1)",
        ),
        (
            "k = 5\nk.p |> edges",
            "TypeError: Cannot explain k.p (line 8, column 1)",
        ),
        (
            "c |> why |> edges |> why",
            "TypeError: Cannot explain c|>why|>edges (line 7, column 1)",
        ),
        (
            "c |> why |> edges |> depth(1)",
            "TypeError: Cannot explain c|>why|>edges (line 7, column 1)",
        ),
        (
            "c |> why |> edges |> edges",
            "TypeError: Cannot explain c|>why|>edges (line 7, column 1)",
        ),
        (
            "c |> why |> edges |> nodes",
            "TypeError: Cannot explain c|>why|>edges (line 7, column 1)",
        ),
        (
            "c |> why |> nodes |> edges",
            "TypeError: Cannot explain c|>why|>nodes (line 7, column 1)",
        ),
        (
            "c |> why |> holds |> edges",
            "TypeError: Cannot explain c|>why|>holds (line 7, column 1)",
        ),
        (
            "stated = 1\nc |> why |> edges |> stated",
            "TypeError: stated is not an operation (line 8, column 1)",
        ),
        (
            "c |> why |> intersect(f |> why |> edges)",
            "TypeError: Cannot explain f|>why|>edges (line 7, column 1)",
        ),
        (
            "c |> why |> except(f |> why |> edges)",
            "TypeError: Cannot explain f|>why|>edges (line 7, column 1)",
        ),
        (
            "c |> why |> union(f |> why |> edges)",
            "TypeError: Cannot explain f|>why|>edges (line 7, column 1)",
        ),
        (
            "c |> why |> path(a |> why |> edges)",
            "TypeError: Cannot explain a|>why|>edges (line 7, column 1)",
        ),
        (
            "other = f |> why |> edges\nc |> why |> intersect(other) |> edges",
            "[]",
        ),
        (
            "edges = 1\nc |> why |> edges",
            "TypeError: edges is not an operation (line 8, column 1)",
        ),
        (
            "def edges(x) => x\nc |> why |> edges",
            "TypeError: edges is not an operation (line 8, column 1)",
        ),
        (
            "class edges\nc |> why |> edges",
            "TypeError: edges is not an operation (line 8, column 1)",
        ),
        (
            "try:\n    r = c |> why |> edges\ncatch err:\n    r = \"caught\"\nr",
            "[[\"b\",\"c\"],[\"a\",\"b\"]]",
        ),
        (
            "edges = 1\ntry:\n    r = c |> why |> edges\ncatch err:\n    r = \"caught\"\nr",
            "\"caught\"",
        ),
        (
            "vals = c |> why |> edges\na = 5\nvals",
            "[[\"b\",\"c\"],[\"a\",\"b\"]]",
        ),
        (
            "vals = c |> why |> edges\nx = 5\nb = x + 1\nvals",
            "[[\"b\",\"c\"],[\"x\",\"b\"]]",
        ),
        ("vals = c |> why |> edges\nwhy vals", "[]"),
        ("`n ${c |> why |> edges}`", "\"n b,c,a,b\""),
        (
            "suppose a = 5:\n    inner = c |> why |> edges\n    inner",
            "[[\"b\",\"c\"],[\"a\",\"b\"]]",
        ),
        (
            "x, y = c |> why |> edges\n[x, y]",
            "[[\"b\",\"c\"],[\"a\",\"b\"]]",
        ),
        (
            "x, y = [c |> why |> edges, 1]\n[x, y]",
            "[[[\"b\",\"c\"],[\"a\",\"b\"]],1.0]",
        ),
        ("{length} = c |> why |> edges\nlength", "2.0"),
        ("{x, y} = c |> why |> edges\n[x, y]", "[null,null]"),
        ("x, = c |> why |> edges\nx", "[\"b\",\"c\"]"),
        (
            "[p] = c |> why |> edges",
            "SyntaxError: Cannot assign to [p] (line 7, column 1)",
        ),
        (
            "delete c |> why |> edges",
            "SyntaxError: Cannot delete c|>why|>edges (line 7, column 1)",
        ),
        (
            "not c |> why |> edges",
            "TypeError: Cannot explain !c (line 7, column 1)",
        ),
        ("c |> why |> edges(1)", "1.0"),
        ("c |> why |> edges[0]", "[0.0]"),
        (
            "c |> why |> edges.length",
            "SyntaxError: Unexpected '.' (line 7, column 19)",
        ),
        ("(c |>\nwhy |>\nedges)", "[[\"b\",\"c\"],[\"a\",\"b\"]]"),
        ("c |> why |> edges; a", "1.0"),
        (
            "vals = c |> why |> edges\nvals.push(\"z\")\nvals",
            "[[\"b\",\"c\"],[\"a\",\"b\"],\"z\"]",
        ),
        (
            "n = null\nfor s in c |> why |> depth(n) |> edges:\n    r = s",
            "TypeError: c|>why|>depth(n)|>edges is not iterable (line 8, column 1)",
        ),
        (
            "c |> edges edges",
            "ReferenceError: edges is not defined (line 7, column 12)",
        ),
    ];
    let unchanged = [
        ("c |> why |> Edges", "TypeError: Edges is not an operation"),
        ("c |> why |> edge", "TypeError: edge is not an operation"),
        (
            "c |> why |>\nedges",
            "SyntaxError: Expected an operation (line 7, column 10)",
        ),
        (
            "edges c",
            "ReferenceError: edges is not defined (line 7, column 1)",
        ),
        (
            "edges",
            "ReferenceError: edges is not defined (line 7, column 1)",
        ),
        (
            "edges(c |> why)",
            "ReferenceError: edges is not defined (line 7, column 1)",
        ),
        ("edges = [1, 2]\nedges", "[1.0,2.0]"),
        ("x = {edges: 1}\nx.edges", "1.0"),
        ("def edges(x) => x * 2\nedges(3)", "6.0"),
        (
            "class Graph(edges): this.edges = edges\ng = Graph(2)\ng.edges",
            "2.0",
        ),
        ("(c |> why).edges", "null"),
        ("(c |> why)[0].from", "[\"b\"]"),
        (
            "xs = [1]\nxs.edges([2])",
            "TypeError: xs.edges is not a function (line 8, column 1)",
        ),
        ("`v ${c why c |> edges}`", "\"v 4\""),
        (
            "def g(x = c |> why |> edges) => x",
            "SyntaxError: Expected a scalar literal parameter default (line 7, column 11)",
        ),
        (
            "{x: c |> why |> edges}",
            "SyntaxError: Unexpected ':' (line 7, column 5)",
        ),
        (
            "(c).flatMap(step => step.from)",
            "TypeError: number.flatMap is not a function (line 7, column 1)",
        ),
        ("c |> why |> nodes", "[\"c\",\"b\",\"a\"]"),
        ("c |> why |> holds", "[4.0,2.0,1.0]"),
        ("s = \"a |> edges\"\ns", "\"a |> edges\""),
    ];
    for (rows, label) in [(&changed[..], "changed"), (&unchanged[..], "unchanged")] {
        for (lines, expected) in rows {
            let source = format!("{SIX}{lines}\n");
            assert_eq!(fresh(&source), *expected, "{label}: {lines}");
        }
    }
    assert_eq!(
        fresh(&format!("{SIX}{}\n", accepted("c |> depth(0)"))),
        "[[\"b\",\"c\"]]"
    );
    assert_eq!(changed.len() + unchanged.len(), 80);
}
