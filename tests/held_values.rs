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
    "Nucleoid gives the values of an explanation with holds",
    "Nucleoid gives the values that a change reaches and the model with holds",
    "Nucleoid gives the values behind class level rules with holds",
    "Nucleoid rolls back failed updates through held values",
];
const REFERENCE_TITLE: &str = "Nucleoid gives the values of selections with holds";
const README_TITLE: &str =
    "Holds lists the readings behind a frost warning and what the wind moves";
const EXAMPLES_TITLE: &str = "Holds shows the figures behind an invoice and the totals on the way";

const CHAIN: &str = "a = 1\nb = a + 2\nc = b * 2\n";
const SIX: &str = "a = 1\nb = a + 1\nc = b * 2\nd = a + 3\ne = 7\nf = e + d\n";
const KINDS: &str = "class P(v): this.v = v\np = P(5)\ndef f(x) => x\ng = x -> x\nk = 1\nif k > 0:\n    m = 2\ns = \"hi\"\nxs = [1, [2]]\nnone = null\nflag = true\no = {k: 1}\nr = o\n";
const CLASSY: &str =
    "rate = 2\nclass M(load):\n    this.load = load\n$M.wear = $M.load * rate\nm = M(3)\n";

type Rewrite = (&'static str, &'static str);

struct Counterpart {
    title: &'static str,
    rewrites: &'static [Rewrite],
    before: usize,
    after: usize,
}

const SPELLED: [Counterpart; 7] = [
    Counterpart {
        title: "Nucleoid gives the values of an explanation with holds",
        rewrites: &[
            ("price = 10", "h = s -> s.map(.holds)\nprice = 10"),
            ("total |> why |> holds", "h(why total)"),
            ("total |> why |> stated |> holds", "h(stated why total)"),
            ("total |> why |> derived |> holds", "h(derived why total)"),
        ],
        before: 170,
        after: 160,
    },
    Counterpart {
        title: "Nucleoid gives the values that a change reaches and the model with holds",
        rewrites: &[
            ("price |> affects |> holds", "(affects price).map(.holds)"),
            ("model |> holds", "(model |> why).map(.holds)"),
        ],
        before: 153,
        after: 140,
    },
    Counterpart {
        title: "Nucleoid gives the values behind class level rules with holds",
        rewrites: &[
            ("fee = 2", "h = s -> s.map(.holds)\nfee = 2"),
            ("$Order.total |> why |> holds", "h(why $Order.total)"),
            ("fee |> affects |> holds", "h(affects fee)"),
            (
                "first.total |> why |> stated |> holds",
                "h(stated why first.total)",
            ),
        ],
        before: 228,
        after: 217,
    },
    Counterpart {
        title: "Nucleoid rolls back failed updates through held values",
        rewrites: &[
            ("subtotal |> why |> holds", "(why subtotal).map(.holds)"),
            ("1 |> holds", "1 |> why"),
            (
                "subtotal |> why |> stated |> holds",
                "(stated why subtotal).map(.holds)",
            ),
        ],
        before: 159,
        after: 150,
    },
    Counterpart {
        title: "Nucleoid gives the values of selections with holds",
        rewrites: &[
            (
                "total |> why |> stated |> holds",
                "(stated why total).map(.holds)",
            ),
            ("price |> affects |> holds", "(affects price).map(.holds)"),
        ],
        before: 94,
        after: 85,
    },
    Counterpart {
        title: "Holds lists the readings behind a frost warning and what the wind moves",
        rewrites: &[
            (
                "frost |> why |> stated |> holds",
                "(stated why frost).map(.holds)",
            ),
            ("wind |> affects |> holds", "(affects wind).map(.holds)"),
        ],
        before: 110,
        after: 101,
    },
    Counterpart {
        title: "Holds shows the figures behind an invoice and the totals on the way",
        rewrites: &[
            (
                "invoice |> why |> stated |> holds",
                "(stated why invoice).map(.holds)",
            ),
            (
                "invoice |> why |> derived |> holds",
                "(derived why invoice).map(.holds)",
            ),
        ],
        before: 98,
        after: 90,
    },
];

const FORMS: [(&str, &str); 23] = [
    ("X |> holds", "X |> depth(0)"),
    ("X |> why |> holds", "X |> why"),
    ("why X |> holds", "why X"),
    ("X |> affects |> holds", "X |> affects"),
    ("affects X |> holds", "affects X"),
    ("X |> why |> stated |> holds", "X |> why |> stated"),
    ("X |> why |> derived |> holds", "X |> why |> derived"),
    ("stated X |> holds", "stated X"),
    ("derived X |> holds", "derived X"),
    ("stated why X |> holds", "stated why X"),
    ("X |> why |> depth(1) |> holds", "X |> why |> depth(1)"),
    (
        "X |> affects |> depth(1) |> holds",
        "X |> affects |> depth(1)",
    ),
    ("X |> why |> path(Y) |> holds", "X |> why |> path(Y)"),
    (
        "Y |> affects |> path(X) |> holds",
        "Y |> affects |> path(X)",
    ),
    (
        "X |> why |> intersect(Y |> why) |> holds",
        "X |> why |> intersect(Y |> why)",
    ),
    (
        "X |> why |> except(Y |> why) |> holds",
        "X |> why |> except(Y |> why)",
    ),
    (
        "X |> why |> union(Y |> why) |> holds",
        "X |> why |> union(Y |> why)",
    ),
    (
        "X |> affects |> union(Y |> why) |> depth(1) |> holds",
        "X |> affects |> union(Y |> why) |> depth(1)",
    ),
    (
        "X |> why |> stated |> except(Y |> why |> intersect(Y |> why)) |> holds",
        "X |> why |> stated |> except(Y |> why |> intersect(Y |> why))",
    ),
    (
        "X |> why |> depth(null) |> holds",
        "X |> why |> depth(null)",
    ),
    ("model |> holds", "model |> depth(0)"),
    ("model |> stated |> holds", "model |> stated"),
    (
        "model |> intersect(X |> why) |> holds",
        "model |> intersect(X |> why)",
    ),
];

const GRAPHS: [(&str, &[&str], &[&str]); 6] = [
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
        "s = \"ab\"\nflag = true\nnone = null\nxs = [1, [2]]\no = {k: 1}\nref = o\nlabel = s + xs.length\nok = flag && none == null\nnest = [xs, label, ok]\n",
        &["label", "ok", "nest", "ref"],
        &["s = \"cd\"", "xs = [3]", "flag = false", "o = {k: 2}"],
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

fn writes_holds(source: &str) -> bool {
    tokenize(source).unwrap().windows(2).any(|window| {
        window[0].token == Token::Pipe
            && matches!(&window[1].token, Token::Identifier(name) if name == "holds")
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

fn exact(runtime: &mut Runtime, source: &str) -> String {
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
            title: "held values regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn accepted(selection: &str) -> String {
    format!("({selection}).map(step => step.holds)")
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
        for name in ["values", "facts", "inner", "names"] {
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
fn only_the_appended_cases_write_holds_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(writes_holds(&case.source), new, "{}", case.title);
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
    assert_eq!(assertions, 27);
    assert_eq!(cases, 1280);
}

#[test]
fn the_audit_finds_every_holds_stage_and_nothing_else() {
    for source in [
        "c |> holds",
        "c |> why |> holds",
        "why c |> stated |> holds",
        "x = model|>holds",
        "f(c |> why |> holds)",
        "$P.basis = $P.v |> why |> holds",
        "c |> why |> holds |> why",
        "c |> why |> holds(1)",
    ] {
        assert!(writes_holds(source), "{source}");
    }
    for source in [
        "holds",
        "holds c",
        "holds(c |> why)",
        "holds = [1, 2]",
        "o = {holds: 1}",
        "o.holds",
        "(why c).map(.holds)",
        "[s.holds for s in why c]",
        "def holds(x) => x",
        "c |> why |> Holds",
        "c |> why |> hold",
        "c |> why |>\nholds",
        "c |> why |> nodes",
        "\"|> holds\"",
    ] {
        assert!(!writes_holds(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_after_nuc_120_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(titles[796], "Nucleoid restores unions after failed changes");
    assert_eq!(titles[797..801], SPEC_TITLES);
    assert_eq!(titles[836], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid joins two explanations with union",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Union lists every fact behind either of two alarms",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Union shows every figure that a change of rent or of wages moves",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(
            index,
            cases.len() - 15 - usize::from(title == REFERENCE_TITLE),
            "{title}"
        );
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 10].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Held values of reasoning selections\n\n```\n")
        .expect("the language reference documents the holds stage");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(case.expected.as_deref(), Some("[[4,20,3],[60,65]]"));
    for line in [
        "[NUC 121](nuc-0121.md)",
        "reference |> why |> holds             the values of an explanation's steps, in its order (NUC 121)",
        "model |> holds                        the values of the current model's nodes (NUC 121)",
        "selection |> holds                    the list (selection).map(step => step.holds); ends the pipeline (NUC 121)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    assert!(include_str!("../README.md").contains("[NUC 121](docs/nuc-0121.md)"));
    assert!(include_str!("../docs/examples.md").contains("[NUC 121](nuc-0121.md)"));
    assert!(
        include_str!("../nucleoid.spec.md").contains("\n## Held values of reasoning selections\n")
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
        assert!(writes_holds(&case.source), "{title}");
        assert!(!writes_holds(&counterpart), "{title}");
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
fn holds_parses_as_a_final_stage_and_stays_an_ordinary_name() {
    let why_c = Expr::Reason {
        stage: Stage::Why,
        source: Box::new(Expr::Identifier("c".to_string())),
    };
    let values = Expr::Reason {
        stage: Stage::Holds,
        source: Box::new(why_c.clone()),
    };
    assert_eq!(parse_expression("c |> why |> holds").unwrap(), values);
    assert_eq!(parse_expression("why c |> holds").unwrap(), values);
    assert_eq!(parse_expression("c|>why|>holds").unwrap(), values);
    assert_eq!(parse_expression("(c |>\nwhy |>\nholds)").unwrap(), values);
    assert_eq!(
        parse_expression("c |> holds").unwrap(),
        Expr::Reason {
            stage: Stage::Holds,
            source: Box::new(Expr::Identifier("c".to_string())),
        }
    );
    assert_eq!(
        parse_expression("model |> holds").unwrap(),
        Expr::Reason {
            stage: Stage::Holds,
            source: Box::new(Expr::Model),
        }
    );
    assert_eq!(
        parse_expression("c |> why |> holds |> why").unwrap(),
        Expr::Reason {
            stage: Stage::Why,
            source: Box::new(values.clone()),
        }
    );
    assert_eq!(
        parse_expression("c |> why |> nodes |> holds").unwrap(),
        Expr::Reason {
            stage: Stage::Holds,
            source: Box::new(Expr::Reason {
                stage: Stage::Nodes,
                source: Box::new(why_c),
            }),
        }
    );
    assert_eq!(Stage::Holds.as_str(), "holds");
    assert_eq!(Stage::Nodes.as_str(), "nodes");
    assert_eq!(Stage::from_name("holds"), None);
    assert_eq!(Stage::from_name("why"), Some(Stage::Why));
    for (source, expected) in [
        (
            format!("{CHAIN}holds c"),
            "ReferenceError: holds is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}holds"),
            "ReferenceError: holds is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}holds(c |> why)"),
            "ReferenceError: holds is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> Holds"),
            "TypeError: Holds is not an operation",
        ),
        (
            format!("{CHAIN}c |> why |> hold"),
            "TypeError: hold is not an operation",
        ),
        (
            format!("{CHAIN}c |> why |>\nholds"),
            "SyntaxError: Expected an operation (line 4, column 10)",
        ),
        (format!("{CHAIN}holds = [1, 2]\nholds"), "[1.0,2.0]"),
        (format!("{CHAIN}x = {{holds: 1}}\nx.holds"), "1.0"),
        (format!("{CHAIN}def holds(x) => x * 2\nholds(3)"), "6.0"),
        (format!("{CHAIN}(c |> why).holds"), "null"),
        (format!("{CHAIN}(c |> why)[0].holds"), "6.0"),
        (format!("{CHAIN}xs = [1]\nxs.holds"), "null"),
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
    assert_eq!(compared, 1702);
    assert_eq!(
        result(&format!(
            "{CHAIN}[c |> why |> holds, model |> holds, a |> affects |> holds]"
        )),
        "[[6.0,3.0,1.0],[1.0,3.0,6.0],[3.0,6.0]]"
    );
    assert_eq!(result("model |> holds"), "[]");
    assert_eq!(result(&format!("{CHAIN}a |> derived |> holds")), "[]");
    assert_eq!(result(&format!("{CHAIN}o = {{}}\no.gone |> holds")), "[]");
}

#[test]
fn values_are_given_as_their_records_hold_them() {
    for selection in [
        "model |> depth(0)",
        "model |> stated",
        "r |> why",
        "p |> why |> depth(0)",
        "p.v |> affects",
        "s |> why |> depth(0)",
        "g |> why |> depth(0)",
        "f |> why |> depth(0)",
        "xs |> why",
    ] {
        let written = format!("{KINDS}{selection} |> holds");
        let spelling = format!("{KINDS}{}", accepted(selection));
        assert_eq!(fresh(&written), fresh(&spelling), "{selection}");
    }
    assert_eq!(
        result(&format!("{KINDS}model |> holds")),
        "[null,{\"id\":\"p\",\"v\":5.0},5.0,null,\"[function]\",1.0,null,\"hi\",[1.0,[2.0]],null,true,{\"k\":1.0},{\"k\":1.0}]"
    );
    assert_eq!(
        result(&format!(
            "{KINDS}[r |> why |> holds, p |> holds, s |> holds, g |> holds, f |> holds]"
        )),
        "[[{\"k\":1.0},{\"k\":1.0}],[{\"id\":\"p\",\"v\":5.0}],[\"hi\"],[\"[function]\"],[null]]"
    );
    assert_eq!(result(&format!("{KINDS}(p |> holds)[0].v")), "5.0");
    assert_eq!(
        result(&format!("{KINDS}(r |> why |> holds)[0] == o")),
        "true"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}xs = [1, 2]\nn = xs.length\nlabel = \"n\" + n\nlabel |> why |> holds"
        )),
        "[\"n2\",2.0,[1.0,2.0]]"
    );
}

#[test]
fn a_reference_alone_gives_its_own_value() {
    assert_eq!(result(&format!("{CHAIN}c |> holds")), "[6.0]");
    assert_eq!(
        result(&format!("{CHAIN}{}", accepted("c |> depth(0)"))),
        "[6.0]"
    );
    assert_eq!(
        result(&format!("{CHAIN}{}", accepted("c |> why |> depth(0)"))),
        "[6.0]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}[a |> holds, a |> stated |> holds, c |> stated |> holds, c |> derived |> holds]"
        )),
        "[[1.0],[1.0],[],[6.0]]"
    );
    assert_eq!(
        failure(&format!("{CHAIN}(c).map(step => step.holds)")),
        "TypeError: number.map is not a function (line 4, column 1)"
    );
    for (source, expected) in [
        (
            format!("{CHAIN}missing |> holds"),
            "ReferenceError: missing is not defined (line 4, column 1)",
        ),
        (
            format!("{CHAIN}1 |> holds"),
            "TypeError: Cannot explain 1 (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c + 1 |> holds"),
            "TypeError: Cannot explain c+1 (line 4, column 1)",
        ),
        (
            "n = null\nn.p |> holds".to_string(),
            "ReferenceError: n is not defined (line 2, column 1)",
        ),
        (
            "k = 5\nk.p |> holds".to_string(),
            "TypeError: Cannot explain k.p (line 2, column 1)",
        ),
        (
            "u = {}\nu.q.p |> holds".to_string(),
            "ReferenceError: u.q is not defined (line 2, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> depth(-1) |> holds"),
            "TypeError: -1 is not a non-negative integer (line 4, column 1)",
        ),
        (
            format!("{CHAIN}c |> why |> diff |> holds"),
            "TypeError: Cannot use 'diff' outside suppose (line 4, column 1)",
        ),
    ] {
        assert_eq!(failure(&source), expected, "{source}");
    }
}

#[test]
fn stages_after_holds_and_holds_arguments_raise_their_accepted_errors() {
    let prelude = format!("{CHAIN}d = a + 10\n");
    for (written, spelling, expected, spelled_error) in [
        (
            "c |> why |> holds |> why",
            "(c |> why).map(step => step.holds) |> why",
            "TypeError: Cannot explain c|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> holds |> depth(1)",
            "(c |> why).map(step => step.holds) |> depth(1)",
            "TypeError: Cannot explain c|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> holds |> nodes",
            "(c |> why).map(step => step.holds) |> nodes",
            "TypeError: Cannot explain c|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> holds |> holds",
            "(c |> why).map(step => step.holds) |> depth(0)",
            "TypeError: Cannot explain c|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> nodes |> holds",
            "(c |> why).map(step => step.node) |> depth(0)",
            "TypeError: Cannot explain c|>why|>nodes (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> holds |> affects |> path(a)",
            "(c |> why).map(step => step.holds) |> affects |> path(a)",
            "TypeError: Cannot explain c|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> holds |> union(c |> why)",
            "(c |> why).map(step => step.holds) |> union(c |> why)",
            "TypeError: Cannot explain c|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain c|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> intersect(d |> why |> holds)",
            "c |> why |> intersect((d |> why).map(step => step.holds))",
            "TypeError: Cannot explain d|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain d|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> except(d |> why |> holds)",
            "c |> why |> except((d |> why).map(step => step.holds))",
            "TypeError: Cannot explain d|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain d|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> union(d |> why |> holds)",
            "c |> why |> union((d |> why).map(step => step.holds))",
            "TypeError: Cannot explain d|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain d|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> depth(2) |> union(d |> why |> holds)",
            "c |> why |> depth(2) |> union((d |> why).map(step => step.holds))",
            "TypeError: Cannot explain d|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain d|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> path(a |> why |> holds)",
            "c |> why |> path((a |> why).map(step => step.holds))",
            "TypeError: Cannot explain a|>why|>holds (line 5, column 1)",
            "TypeError: Cannot explain a|>why.map(function()) (line 5, column 1)",
        ),
        (
            "c |> why |> depth(m.value) |> holds |> why",
            "(c |> why |> depth(m.value)).map(step => step.holds) |> why",
            "TypeError: Cannot explain c|>why|>depth(m.value)|>holds (line 5, column 1)",
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
        failure(&format!("{prelude}stated = 1\nc |> why |> holds |> stated")),
        "TypeError: stated is not an operation (line 6, column 1)"
    );
    assert_eq!(
        result(&format!(
            "{prelude}other = d |> why |> holds\n[c |> why |> intersect(other) |> holds, c |> why |> except(other) |> holds, c |> why |> union(other) |> holds]"
        )),
        "[[],[6.0,3.0,1.0],[6.0,3.0,1.0]]"
    );
    assert_eq!(
        result(&format!(
            "{prelude}other = (d |> why).map(step => step.holds)\n[(c |> why |> intersect(other)).map(step => step.holds), (c |> why |> except(other)).map(step => step.holds), (c |> why |> union(other)).map(step => step.holds)]"
        )),
        "[[],[6.0,3.0,1.0],[6.0,3.0,1.0]]"
    );
}

#[test]
fn every_binding_named_holds_shadows_the_stage_at_run_time() {
    let bindings = [
        ("holds = 1\nc |> why |> holds", 5),
        ("def holds(x) => x\nc |> why |> holds", 5),
        ("class holds\nc |> why |> holds", 5),
        ("holds = 1\nmissing |> why |> holds", 5),
        ("holds = 1\n1 |> holds", 5),
        ("stated = 1\nholds = 2\nc |> why |> stated |> holds", 6),
        ("def f(holds) => c |> why |> holds\nf(1)", 5),
        ("def g holds => c |> why |> holds\ng(1)", 5),
        ("f = holds -> c |> why |> holds\nf(1)", 5),
        ("f = (holds) => c |> why |> holds\nf(1)", 5),
        ("f = lambda holds: c |> why |> holds\nf(1)", 5),
        ("[1].map(holds -> c |> why |> holds)", 4),
        ("for holds of \"x\":\n    y = c |> why |> holds", 4),
        ("ok = all holds in [1]: (c |> why |> holds).length > 0", 4),
        ("{\n    holds = 5\n    y = c |> why |> holds\n}", 4),
        (
            "class R:\n    def m(holds) => c |> why |> holds\nR().m(1)",
            6,
        ),
        (
            "class Q(holds):\n    this.v = (c |> why |> holds).length\nQ(1)",
            6,
        ),
        (
            "try:\n    throw 1\ncatch holds:\n    y = c |> why |> holds",
            4,
        ),
        ("{holds} = {holds: 1}\nc |> why |> holds", 5),
    ];
    for (binding, line) in bindings {
        assert_eq!(
            failure(&format!("{CHAIN}{binding}")),
            format!("TypeError: holds is not an operation (line {line}, column 1)"),
            "{binding}"
        );
    }
    assert_eq!(
        failure(&format!("{CHAIN}stated = 1\nc |> why |> stated |> holds")),
        "TypeError: stated is not an operation (line 5, column 1)"
    );
    for (binding, expected) in [
        (
            "holds = 1\nc |> why |> holds |> why",
            "TypeError: Cannot explain c|>why|>holds (line 5, column 1)",
        ),
        (
            "nodes = 1\nc |> why |> nodes |> holds",
            "TypeError: Cannot explain c|>why|>nodes (line 5, column 1)",
        ),
        (
            "holds = 1\nd = a + 10\nc |> why |> intersect(d |> why |> holds)",
            "TypeError: Cannot explain d|>why|>holds (line 6, column 1)",
        ),
    ] {
        assert_eq!(failure(&format!("{CHAIN}{binding}")), expected, "{binding}");
    }
    assert_eq!(
        result(&format!(
            "{CHAIN}class K:\n    def holds() => 1\no = {{holds: 1}}\nq = {{}}\nq.holds = 2\n[c |> why |> holds, o.holds, q.holds, K().holds(), (c |> why)[0].holds]"
        )),
        "[[6.0,3.0,1.0],1.0,2.0,1.0,6.0]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}why = 1\naffects = 2\n[c |> why |> holds, a |> affects |> holds]"
        )),
        "[[6.0,3.0,1.0],[3.0,6.0]]"
    );
    assert_eq!(
        result(&format!("{CHAIN}nodes = 1\nc |> why |> holds")),
        "[6.0,3.0,1.0]"
    );
    assert_eq!(
        result(&format!("{CHAIN}holds = 1\nc |> why |> nodes")),
        "[\"c\",\"b\",\"a\"]"
    );
    assert_eq!(
        result(&format!("{CHAIN}holds = 1\n{}", accepted("c |> why"))),
        "[6.0,3.0,1.0]"
    );
}

#[test]
fn a_try_now_catches_what_was_a_parse_error() {
    assert_eq!(
        result(&format!(
            "{CHAIN}holds = 5\nr = \"none\"\ntry:\n    r = c |> why |> holds\ncatch error:\n    r = error\nr"
        )),
        "\"TypeError: holds is not an operation\""
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}try:\n    r = c |> why |> holds\ncatch e:\n    r = \"caught\"\nr"
        )),
        "[6.0,3.0,1.0]"
    );
    assert_eq!(
        failure(&format!("{CHAIN}missing\nholds = 1\nc |> why |> holds")),
        "ReferenceError: missing is not defined (line 4, column 1)"
    );
    assert_eq!(
        failure(&format!("{CHAIN}holds = 1\nc |> why |> holds\nmissing")),
        "TypeError: holds is not an operation (line 5, column 1)"
    );
    let session = repl(
        "x = 5\nholds = 1\nx |> holds\nx\n.exit\n",
        &["--plain", "--json"],
    );
    assert!(session.status.success(), "{session:?}");
    assert_eq!(
        String::from_utf8(session.stderr).unwrap(),
        "TypeError: holds is not an operation (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(session.stdout).unwrap();
    assert!(stdout.ends_with("5.0\n1.0\n5.0\n"), "{stdout}");
}

#[test]
fn a_binding_declared_after_a_standing_declaration_fails_its_refresh() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{CHAIN}values = c |> why |> holds"));
    checked(&mut runtime, "holds = 5");
    assert_eq!(outcome(&mut runtime, "values"), "[6.0,3.0,1.0]");
    assert_eq!(
        outcome(&mut runtime, "a = 3"),
        "TypeError: holds is not an operation (line 1, column 1)"
    );
    assert_eq!(outcome(&mut runtime, "a"), "1.0");
    assert_eq!(outcome(&mut runtime, "values"), "[6.0,3.0,1.0]");
    checked(&mut runtime, "delete holds\nx = 3\nb = x + 1");
    assert_eq!(outcome(&mut runtime, "values"), "[8.0,4.0,3.0]");
}

#[test]
fn the_backtracking_forms_now_destructure_the_values() {
    for (written, expected) in [
        ("x, y = c |> why |> holds\n[x, y]", "[6.0,3.0]"),
        (
            "x, y = [c |> why |> holds, 1]\n[x, y]",
            "[[6.0,3.0,1.0],1.0]",
        ),
        ("{length} = c |> why |> holds\nlength", "3.0"),
        ("{x, y} = c |> why |> holds\n[x, y]", "[null,null]"),
        ("x, = c |> why |> holds\nx", "6.0"),
    ] {
        let spelling = written.replace("c |> why |> holds", &accepted("c |> why"));
        assert_eq!(result(&format!("{CHAIN}{written}")), expected, "{written}");
        assert_eq!(
            result(&format!("{CHAIN}{spelling}")),
            expected,
            "{spelling}"
        );
    }
    behaves_as_spelled(&[
        (
            "a = 1\nb = a + 2\nc = b * 2\nx, y = c |> why |> holds\n[x, y]",
            "a = 1\nb = a + 2\nc = b * 2\nx, y = (c |> why).map(step => step.holds)\n[x, y]",
        ),
        ("z = 9\nb = z + 1\n[x, y]", "z = 9\nb = z + 1\n[x, y]"),
        ("z = 2\n[x, y]", "z = 2\n[x, y]"),
    ]);
}

#[test]
fn standing_declarations_follow_the_graph_as_their_accepted_spelling_does() {
    let mut compared = 0;
    for (setup, names, writes) in GRAPHS {
        for x in names {
            for (form, selection) in FORMS {
                let y = names[0];
                let written = format!("{setup}values = {}\nvalues", spelled(form, x, y));
                let spelling = format!(
                    "{setup}values = {}\nvalues",
                    accepted(&spelled(selection, x, y))
                );
                let mut steps = vec![(written, spelling)];
                for write in writes {
                    steps.push((format!("{write}\nvalues"), format!("{write}\nvalues")));
                }
                behaves_as_spelled(&steps);
                compared += 1;
            }
        }
    }
    assert_eq!(compared, 460);
}

#[test]
fn values_follow_every_change_their_selection_observes() {
    behaves_as_spelled(&[
        (
            "a = 1\nb = a + 2\nc = b * 2\nvalues = c |> why |> holds\nfacts = c |> why |> stated |> holds\nvalues",
            "a = 1\nb = a + 2\nc = b * 2\nvalues = (c |> why).map(step => step.holds)\nfacts = (c |> why |> stated).map(step => step.holds)\nvalues",
        ),
        ("a = 5\n[values, facts]", "a = 5\n[values, facts]"),
        ("c = b * 3\n[values, facts]", "c = b * 3\n[values, facts]"),
        (
            "x = 2\nb = a + x\n[values, facts]",
            "x = 2\nb = a + x\n[values, facts]",
        ),
        ("x = 4\n[values, facts]", "x = 4\n[values, facts]"),
        ("delete x\n[values, facts]", "delete x\n[values, facts]"),
    ]);
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}values = c |> why |> holds\nfacts = c |> why |> stated |> holds"),
    );
    for (write, expected) in [
        ("a = 5", "[[14.0,7.0,5.0],[5.0]]"),
        ("c = b + 1", "[[8.0,7.0,5.0],[5.0]]"),
        ("x = 2\nb = a + x", "[[8.0,7.0,5.0,2.0],[5.0,2.0]]"),
        ("x = 3", "[[9.0,8.0,5.0,3.0],[5.0,3.0]]"),
    ] {
        checked(&mut runtime, write);
        assert_eq!(
            outcome(&mut runtime, "[values, facts]"),
            expected,
            "{write}"
        );
    }
}

#[test]
fn value_snapshots_suppose_and_rollback_behave_as_the_accepted_spelling() {
    behaves_as_spelled(&[
        (
            "a = 1\nb = a + 2\nc = b * 2\nvalues = c |> why |> holds\nsnap = values.value\nx = 5\nb = x + 1\n[values, snap]",
            "a = 1\nb = a + 2\nc = b * 2\nvalues = (c |> why).map(step => step.holds)\nsnap = values.value\nx = 5\nb = x + 1\n[values, snap]",
        ),
        (
            "frozen = (c |> why |> stated |> holds).value\nx = 8\n[frozen, values]",
            "frozen = (c |> why |> stated).map(step => step.holds).value\nx = 8\n[frozen, values]",
        ),
        (
            "suppose x = 5:\n    inner = c |> why |> holds\n    [inner, c |> why |> diff |> holds]",
            "suppose x = 5:\n    inner = (c |> why).map(step => step.holds)\n    [inner, (c |> why |> diff).map(step => step.holds)]",
        ),
        ("values", "values"),
        (
            "try:\n    y = 4\n    b = y + a\n    assert(values, [10, 5, 4, 1])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nvalues",
            "try:\n    y = 4\n    b = y + a\n    assert(values, [10, 5, 4, 1])\n    throw \"UNDO\"\ncatch error:\n    assert(error, \"UNDO\")\nvalues",
        ),
        ("b = a + missing", "b = a + missing"),
        ("values", "values"),
        (
            "facts = c |> why |> stated |> holds\ntry:\n    facts = 1 |> holds\ncatch error:\n    failure = error\n[failure, facts]",
            "facts = (c |> why |> stated).map(step => step.holds)\ntry:\n    facts = (1 |> why |> depth(0)).map(step => step.holds)\ncatch error:\n    failure = error\n[failure, facts]",
        ),
        ("x = 6\n[values, facts]", "x = 6\n[values, facts]"),
    ]);
    assert_eq!(
        result(&format!(
            "{CHAIN}suppose a = 5:\n    inner = c |> why |> holds\n    [inner, c |> why |> diff |> holds]"
        )),
        "[[14.0,7.0,5.0],[14.0,7.0,5.0]]"
    );
}

#[test]
fn declarations_that_hold_the_stage_are_reasoning_observers() {
    behaves_as_spelled(&[
        (
            "a = 1\nb = a + 2\nc = b * 2\nvalues = c |> why |> holds\n[values |> why |> holds, a |> affects |> holds, model |> holds]",
            "a = 1\nb = a + 2\nc = b * 2\nvalues = (c |> why).map(step => step.holds)\n[(values |> why).map(step => step.holds), (a |> affects).map(step => step.holds), (model |> depth(0)).map(step => step.holds)]",
        ),
        (
            "if (c |> why |> holds).length > 2:\n    flag = true\n[flag, model |> holds]",
            "if (c |> why).map(step => step.holds).length > 2:\n    flag = true\n[flag, (model |> depth(0)).map(step => step.holds)]",
        ),
        (
            "class P(v):\n    this.v = v\n$P.basis = $P.v |> why |> holds\np = P(1)\n[model |> holds, p.basis]",
            "class P(v):\n    this.v = v\n$P.basis = ($P.v |> why).map(step => step.holds)\np = P(1)\n[(model |> depth(0)).map(step => step.holds), p.basis]",
        ),
        (
            "def f() => c |> why |> holds\nlist = f()\n[list, f()]",
            "def f() => (c |> why).map(step => step.holds)\nlist = f()\n[list, f()]",
        ),
        (
            "x = 8\nb = x + 1\n[list, f()]",
            "x = 8\nb = x + 1\n[list, f()]",
        ),
        (
            "q = P(4)\np.v = 7\n[q.basis, p.basis]",
            "q = P(4)\np.v = 7\n[q.basis, p.basis]",
        ),
    ]);
    assert_eq!(
        result(&format!(
            "{CHAIN}values = c |> why |> holds\n[values |> why |> holds, a |> affects |> holds, model |> holds]"
        )),
        "[[],[3.0,6.0],[1.0,3.0,6.0]]"
    );
}

#[test]
fn undefined_selections_stay_undefined() {
    behaves_as_spelled(&[
        (
            "a = 1\nb = a + 2\nc = b * 2\nn = null\nvalues = c |> why |> depth(n) |> holds\n[values, (c |> why |> depth(n) |> holds).length]",
            "a = 1\nb = a + 2\nc = b * 2\nn = null\nvalues = (c |> why |> depth(n)).map(step => step.holds)\n[values, (c |> why |> depth(n)).map(step => step.holds).length]",
        ),
        ("n = 1\nvalues", "n = 1\nvalues"),
        ("n = undefined\nvalues", "n = undefined\nvalues"),
    ]);
    assert_eq!(
        result(&format!("{CHAIN}c |> why |> depth(null) |> holds")),
        "null"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}[`v ${{c |> why |> depth(null) |> holds}}`, typeof (c |> why |> depth(null) |> holds)]"
        )),
        "[\"v undefined\",\"undefined\"]"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}[`v ${{(c |> why |> depth(null)).map(step => step.holds)}}`, typeof (c |> why |> depth(null)).map(step => step.holds)]"
        )),
        "[\"v undefined\",\"undefined\"]"
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}n = null\nfor s in c |> why |> depth(n) |> holds:\n    r = s"
        )),
        "TypeError: c|>why|>depth(n)|>holds is not iterable (line 5, column 1)"
    );
    assert_eq!(
        failure(&format!(
            "{CHAIN}n = null\nfor s in (c |> why |> depth(n)).map(step => step.holds):\n    r = s"
        )),
        "TypeError: c|>why|>depth(n).map(function()) is not iterable (line 5, column 1)"
    );
    assert_eq!(
        result(&format!(
            "{CHAIN}d = a + 10\nc |> why |> union(d |> why |> depth(null)) |> holds"
        )),
        "null"
    );
}

#[test]
fn limits_in_the_source_freeze_and_arguments_that_end_with_holds_stay_as_written() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}n = 1\nvalues = c |> why |> depth(n.value) |> holds\nn = 2"),
    );
    assert_eq!(outcome(&mut runtime, "values"), "[6.0,3.0]");
    assert_eq!(stored(&runtime, "values"), "c|>why|>depth(1)|>holds");
    checked(&mut runtime, "a = 4");
    assert_eq!(outcome(&mut runtime, "values"), "[12.0,6.0]");
    let mut spelling = Runtime::new();
    checked(
        &mut spelling,
        &format!(
            "{CHAIN}n = 1\nvalues = (c |> why |> depth(n.value)).map(step => step.holds)\nn = 2"
        ),
    );
    assert_eq!(outcome(&mut spelling, "values"), "[6.0,3.0]");
    for (written, expected) in [
        (
            "d = a + 10\nv = c |> why |> intersect(d |> why |> depth(m.value) |> holds)",
            "TypeError: Cannot explain d|>why|>depth(m.value)|>holds (line 5, column 1)",
        ),
        (
            "d = a + 10\nv = c |> why |> union(d |> why |> depth(m.value) |> holds)",
            "TypeError: Cannot explain d|>why|>depth(m.value)|>holds (line 5, column 1)",
        ),
        (
            "d = a + 10\nv = c |> why |> intersect((d |> why |> depth(m.value)).map(step => step.holds))",
            "TypeError: Cannot explain d|>why|>depth(m.value).map(function()) (line 5, column 1)",
        ),
        (
            "v = c |> why |> depth(m.value) |> holds |> depth(1)",
            "TypeError: Cannot explain c|>why|>depth(m.value)|>holds (line 4, column 1)",
        ),
        (
            "v = (c |> why |> depth(m.value)).map(step => step.holds) |> depth(1)",
            "TypeError: Cannot explain c|>why|>depth(m.value).map(function()) (line 4, column 1)",
        ),
        (
            "v = c |> why |> depth(m.value) |> holds |> holds",
            "TypeError: Cannot explain c|>why|>depth(m.value)|>holds (line 4, column 1)",
        ),
        (
            "v = c |> why |> depth(m.value) |> holds",
            "ReferenceError: m is not defined (line 4, column 1)",
        ),
        (
            "v = (c |> why |> depth(m.value)).map(step => step.holds)",
            "ReferenceError: m is not defined (line 4, column 1)",
        ),
    ] {
        assert_eq!(failure(&format!("{CHAIN}{written}")), expected, "{written}");
    }
    let mut combined = Runtime::new();
    checked(
        &mut combined,
        &format!(
            "{CHAIN}d = a + 10\nk = 1\nv = c |> why |> intersect(d |> why |> depth(k.value)) |> holds\nk = 0"
        ),
    );
    assert_eq!(
        stored(&combined, "v"),
        "c|>why|>intersect(d|>why|>depth(1))|>holds"
    );
    assert_eq!(outcome(&mut combined, "v"), "[1.0]");
    let mut combined_spelling = Runtime::new();
    checked(
        &mut combined_spelling,
        &format!(
            "{CHAIN}d = a + 10\nk = 1\nv = (c |> why |> intersect(d |> why |> depth(k.value))).map(step => step.holds)\nk = 0"
        ),
    );
    assert_eq!(outcome(&mut combined_spelling, "v"), "[1.0]");
}

#[test]
fn the_graph_stays_bounded_where_the_accepted_spelling_grows() {
    let mut stage = Runtime::new();
    let mut spelling = Runtime::new();
    checked(&mut stage, &format!("{CHAIN}values = c |> why |> holds"));
    checked(
        &mut spelling,
        &format!("{CHAIN}values = (why c).map(.holds)"),
    );
    assert_eq!((stage.graph.len(), spelling.graph.len()), (4, 7));
    for write in 0..50 {
        let source = format!("a = {}", write + 2);
        checked(&mut stage, &source);
        checked(&mut spelling, &source);
        assert_eq!(stage.graph.len(), 4);
        assert_eq!(spelling.graph.len(), 7 + 3 * (write + 1));
        assert_eq!(pending(&stage), 0);
        assert_eq!(pending(&spelling), 3 * (write + 2));
        assert_eq!(reads(&stage, "values"), ["a", "b", "c"]);
        assert_eq!(reads(&stage, "values"), reads(&spelling, "values"));
        assert_eq!(observes(&stage, "values"), observes(&spelling, "values"));
        assert_eq!(
            outcome(&mut stage, "values"),
            outcome(&mut spelling, "values")
        );
    }
    assert_eq!(outcome(&mut stage, "values"), "[106.0,53.0,51.0]");
}

#[test]
fn the_records_number_later_anonymous_objects_as_the_accepted_spelling_does() {
    let file = SourceFile::new("numbering");
    for (prelude, written, spelling, expected) in [
        (
            CHAIN,
            "x = c |> why |> holds\np = {k: 1}\nq = p.value\n[String(p), `${p}`, (why q)[0].rule]",
            "x = (c |> why).map(step => step.holds)\np = {k: 1}\nq = p.value\n[String(p), `${p}`, (why q)[0].rule]",
            "[\"[object obj:3]\",\"[object obj:3]\",\"q = obj:3\"]",
        ),
        (
            CHAIN,
            "x = model |> holds\np = {k: 1}\nString(p)",
            "x = (model |> depth(0)).map(step => step.holds)\np = {k: 1}\nString(p)",
            "\"[object obj:3]\"",
        ),
        (
            CHAIN,
            "x = c |> why |> stated |> holds\ny = c |> why |> nodes\np = {k: 1}\nString(p)",
            "x = (c |> why |> stated).map(step => step.holds)\ny = (c |> why).map(step => step.node)\np = {k: 1}\nString(p)",
            "\"[object obj:4]\"",
        ),
        (
            SIX,
            "x = c |> why |> holds\np = {k: 1}\nString(p)",
            "x = (c |> why).map(step => step.holds)\np = {k: 1}\nString(p)",
            "\"[object obj:3]\"",
        ),
        (
            "",
            "p = {k: 1}\nString(p)",
            "p = {k: 1}\nString(p)",
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
fn declarations_that_read_a_null_value_settle_to_null_as_the_accepted_spelling_does() {
    let model = accepted("model |> depth(0)");
    let steps = [
        (
            format!("{CLASSY}values = model |> holds\n[values, model |> holds]"),
            format!("{CLASSY}values = {model}\n[values, {model}]"),
        ),
        (
            "j = M(null)\n[values, j.wear, j |> holds]".to_string(),
            format!(
                "j = M(null)\n[values, j.wear, {}]",
                accepted("j |> why |> depth(0)")
            ),
        ),
        (
            "$M.basis = $M.load |> why |> holds\nk = M(null)\n[m.basis, k.basis]".to_string(),
            format!(
                "$M.basis = {}\nk = M(null)\n[m.basis, k.basis]",
                accepted("$M.load |> why")
            ),
        ),
        (
            format!(
                "{CHAIN}x = 2\nb = a + x\nfacts = c |> why |> stated |> holds\nsums = c |> why |> holds\n[facts, sums]"
            ),
            format!(
                "{CHAIN}x = 2\nb = a + x\nfacts = {}\nsums = {}\n[facts, sums]",
                accepted("c |> why |> stated"),
                accepted("c |> why")
            ),
        ),
        (
            "delete x\n[facts, sums, c |> why |> holds]".to_string(),
            format!("delete x\n[facts, sums, {}]", accepted("c |> why")),
        ),
        (
            "x = 3\n[facts, sums]".to_string(),
            "x = 3\n[facts, sums]".to_string(),
        ),
    ];
    behaves_as_spelled(&steps);
    let mut runtime = Runtime::new();
    assert_eq!(
        outcome(
            &mut runtime,
            &format!("{CLASSY}values = model |> holds\n[values, model |> holds]")
        ),
        "[null,[2.0,null,{\"id\":\"m\",\"load\":3.0,\"wear\":6.0},3.0,6.0]]"
    );
    checked(&mut runtime, "values = m |> why |> holds");
    assert_eq!(
        outcome(&mut runtime, "values"),
        "[{\"id\":\"m\",\"load\":3.0,\"wear\":6.0}]"
    );
    assert_eq!(
        outcome(
            &mut runtime,
            "n = null\nlist = n |> holds\n[list, n |> holds]"
        ),
        "[null,[null]]"
    );
}

#[test]
fn rendering_shows_the_stage_as_written() {
    for (source, rendered) in [
        ("c |> why |> holds", "c|>why|>holds"),
        ("why c |> stated |> holds", "c|>why|>stated|>holds"),
        ("model |> holds", "model|>holds"),
        (
            "c |> why |> intersect(d |> why) |> holds",
            "c|>why|>intersect(d|>why)|>holds",
        ),
        (
            "c |> why |> union(d |> why) |> holds",
            "c|>why|>union(d|>why)|>holds",
        ),
        ("c |> why |> holds |> why", "c|>why|>holds|>why"),
        ("c |> why |> nodes |> holds", "c|>why|>nodes|>holds"),
        ("(c |> why |> holds).length", "c|>why|>holds.length"),
        ("(c |> why |> nodes).length", "c|>why|>nodes.length"),
    ] {
        let parsed = parse_expression(source).unwrap();
        assert_eq!(parsed.to_string(), rendered, "{source}");
    }
    for source in [
        "c |> why |> holds",
        "c |> holds",
        "c |> why |> depth(1) |> holds",
        "c |> why |> holds |> why",
        "model |> holds",
    ] {
        let parsed = parse_expression(source).unwrap();
        assert_eq!(
            parse_expression(&parsed.to_string()).unwrap(),
            parsed,
            "{source}"
        );
    }
    assert_eq!(
        generate(&parse("def f(x = 1) => c |> why |> holds").unwrap()[0]),
        "def f(x=1):\n    return c|>why|>holds"
    );
    let mut runtime = Runtime::new();
    checked(&mut runtime, &format!("{CHAIN}values = c |> why |> holds"));
    assert_eq!(stored(&runtime, "values"), "c|>why|>holds");
    assert_eq!(
        failure(&format!("{CHAIN}delete c |> why |> holds")),
        "SyntaxError: Cannot delete c|>why|>holds (line 4, column 1)"
    );
    assert_eq!(
        failure(&format!("{CHAIN}delete (c |> why).map(.holds)")),
        "SyntaxError: Cannot delete c|>why.map(.holds) (line 4, column 1)"
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-held-values-{label}-{}.nuc",
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

const PROGRAM: &str = "a = 1\nb = a + 2\nc = b * 2\nvalues = c |> why |> holds\nfacts = c |> why |> stated |> holds\nx = 5\nb = x + 1\n[values, facts, a |> affects |> holds]";
const PROGRAM_SPELLED: &str = "a = 1\nb = a + 2\nc = b * 2\nvalues = (c |> why).map(step => step.holds)\nfacts = (c |> why |> stated).map(step => step.holds)\nx = 5\nb = x + 1\n[values, facts, (a |> affects).map(step => step.holds)]";

#[test]
fn files_graphs_and_repl_sessions_give_values() {
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
        "[[12.0,6.0,5.0],[5.0],[]]\n"
    );
    assert_eq!(
        String::from_utf8(file.run(PROGRAM, &[]).stdout).unwrap(),
        "12,6,5,5,\n"
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
            stage.contains("  values [variable]\n    reads    c, b, x\n    observes c, b, x"),
            "{stage}"
        );
    }
    for (source, message) in [
        (
            "a = 1\nholds = 2\na |> holds\n",
            "TypeError: holds is not an operation (line 3, column 1)\n",
        ),
        (
            "a = 1\na |> holds |> why\n",
            "TypeError: Cannot explain a|>holds (line 2, column 1)\n",
        ),
    ] {
        let failed = file.run(source, &["--json"]);
        assert!(!failed.status.success(), "{failed:?}");
        assert_eq!(String::from_utf8(failed.stderr).unwrap(), message);
    }
    let session = "a = 1\nb = a + 2\nc = b * 2\nc |> why |> holds\nvalues = c |> why |> stated |> holds\nvalues\nx = 5\nb = x + 1\nvalues\n.exit\n";
    let session_spelled = session
        .replace("c |> why |> holds", &accepted("c |> why"))
        .replace(
            "c |> why |> stated |> holds",
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
                "[6.0,3.0,1.0]\n[1.0]\n"
            } else {
                "6,3,1\n1\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
        }
    }
}

#[test]
fn long_chains_of_holds_stages_raise_without_recursing() {
    let stages = " |> holds".repeat(120);
    let source = format!("{CHAIN}c |> why{stages}");
    let quoted = format!("c|>why{}", "|>holds".repeat(119));
    assert_eq!(
        failure(&source),
        format!("TypeError: Cannot explain {quoted} (line 4, column 1)")
    );
    let declared = format!("{CHAIN}v = c |> why{stages}");
    assert_eq!(
        failure(&declared),
        format!("TypeError: Cannot explain {quoted} (line 4, column 1)")
    );
    let alternating = " |> nodes |> holds".repeat(60);
    let mixed = format!("{CHAIN}c |> why{alternating}");
    let mixed_quoted = format!("c|>why{}|>nodes", "|>nodes|>holds".repeat(59));
    assert_eq!(
        failure(&mixed),
        format!("TypeError: Cannot explain {mixed_quoted} (line 4, column 1)")
    );
}

#[test]
fn the_nodes_stage_keeps_its_accepted_behavior() {
    assert_eq!(
        result(&format!(
            "{CHAIN}[c |> why |> nodes, c |> nodes, model |> nodes, a |> affects |> nodes]"
        )),
        "[[\"c\",\"b\",\"a\"],[\"c\"],[\"a\",\"b\",\"c\"],[\"b\",\"c\"]]"
    );
    assert_eq!(
        failure(&format!("{CHAIN}nodes = 1\nc |> why |> nodes")),
        "TypeError: nodes is not an operation (line 5, column 1)"
    );
    assert_eq!(
        failure(&format!("{CHAIN}c |> why |> nodes |> why")),
        "TypeError: Cannot explain c|>why|>nodes (line 4, column 1)"
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!(
            "{CHAIN}n = 1\nnames = c |> why |> depth(n.value) |> nodes\nn = 2\nvalues = c |> why |> holds"
        ),
    );
    assert_eq!(stored(&runtime, "names"), "c|>why|>depth(1)|>nodes");
    assert_eq!(
        outcome(&mut runtime, "[names, values]"),
        "[[\"c\",\"b\"],[6.0,3.0,1.0]]"
    );
    let mut stage = Runtime::new();
    let mut spelling = Runtime::new();
    checked(&mut stage, &format!("{CHAIN}names = c |> why |> nodes"));
    checked(&mut spelling, &format!("{CHAIN}names = (why c).map(.node)"));
    checked(&mut stage, "a = 2");
    checked(&mut spelling, "a = 2");
    assert_eq!((stage.graph.len(), spelling.graph.len()), (4, 10));
    assert_eq!(
        outcome(&mut stage, "names"),
        outcome(&mut spelling, "names")
    );
    let nulls = format!("{CLASSY}none = null\n");
    behaves_as_spelled(&[
        (
            format!("{nulls}names = model |> nodes\nfacts = none |> nodes\n[names, facts]"),
            format!(
                "{nulls}names = (model |> depth(0)).map(step => step.node)\nfacts = (none |> depth(0)).map(step => step.node)\n[names, facts]"
            ),
        ),
        (
            "none = 2\nj = M(null)\n[names, facts]".to_string(),
            "none = 2\nj = M(null)\n[names, facts]".to_string(),
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
        ("c |> why |> holds", "[4.0,2.0,1.0]"),
        ("why c |> holds", "[4.0,2.0,1.0]"),
        ("c |> holds", "[4.0]"),
        ("model |> holds", "[1.0,2.0,4.0,4.0,7.0,11.0]"),
        ("a |> affects |> holds", "[2.0,4.0,11.0,4.0]"),
        ("affects a |> holds", "[2.0,4.0,11.0,4.0]"),
        ("c |> why |> stated |> holds", "[1.0]"),
        ("c |> why |> derived |> holds", "[4.0,2.0]"),
        ("c |> why |> depth(1) |> holds", "[4.0,2.0]"),
        ("c |> why |> path(a) |> holds", "[4.0,2.0,1.0]"),
        ("c |> why |> intersect(f |> why) |> holds", "[1.0]"),
        ("c |> why |> except(f |> why) |> holds", "[4.0,2.0]"),
        (
            "c |> why |> union(f |> why) |> holds",
            "[4.0,2.0,1.0,11.0,7.0,4.0]",
        ),
        ("c |> why |> depth(null) |> holds", "null"),
        (
            "1 |> holds",
            "TypeError: Cannot explain 1 (line 7, column 1)",
        ),
        (
            "missing |> why |> holds",
            "ReferenceError: missing is not defined (line 7, column 1)",
        ),
        (
            "c |> why |> diff |> holds",
            "TypeError: Cannot use 'diff' outside suppose (line 7, column 1)",
        ),
        (
            "c |> why |> holds |> why",
            "TypeError: Cannot explain c|>why|>holds (line 7, column 1)",
        ),
        (
            "c |> why |> holds |> depth(1)",
            "TypeError: Cannot explain c|>why|>holds (line 7, column 1)",
        ),
        (
            "c |> why |> holds |> holds",
            "TypeError: Cannot explain c|>why|>holds (line 7, column 1)",
        ),
        (
            "c |> why |> holds |> nodes",
            "TypeError: Cannot explain c|>why|>holds (line 7, column 1)",
        ),
        (
            "c |> why |> nodes |> holds",
            "TypeError: Cannot explain c|>why|>nodes (line 7, column 1)",
        ),
        (
            "stated = 1\nc |> why |> holds |> stated",
            "TypeError: stated is not an operation (line 8, column 1)",
        ),
        (
            "c |> why |> intersect(f |> why |> holds)",
            "TypeError: Cannot explain f|>why|>holds (line 7, column 1)",
        ),
        (
            "c |> why |> except(f |> why |> holds)",
            "TypeError: Cannot explain f|>why|>holds (line 7, column 1)",
        ),
        (
            "c |> why |> union(f |> why |> holds)",
            "TypeError: Cannot explain f|>why|>holds (line 7, column 1)",
        ),
        (
            "c |> why |> path(a |> why |> holds)",
            "TypeError: Cannot explain a|>why|>holds (line 7, column 1)",
        ),
        (
            "other = f |> why |> holds\nc |> why |> intersect(other) |> holds",
            "[]",
        ),
        (
            "holds = 1\nc |> why |> holds",
            "TypeError: holds is not an operation (line 8, column 1)",
        ),
        (
            "def holds(x) => x\nc |> why |> holds",
            "TypeError: holds is not an operation (line 8, column 1)",
        ),
        (
            "class holds\nc |> why |> holds",
            "TypeError: holds is not an operation (line 8, column 1)",
        ),
        (
            "try:\n    r = c |> why |> holds\ncatch e:\n    r = \"caught\"\nr",
            "[4.0,2.0,1.0]",
        ),
        (
            "holds = 1\ntry:\n    r = c |> why |> holds\ncatch e:\n    r = \"caught\"\nr",
            "\"caught\"",
        ),
        ("vals = c |> why |> holds\na = 5\nvals", "[12.0,6.0,5.0]"),
        (
            "vals = c |> why |> holds\nx = 5\nb = x + 1\nvals",
            "[12.0,6.0,5.0]",
        ),
        ("vals = c |> why |> holds\nwhy vals", "[]"),
        ("`n ${c |> why |> holds}`", "\"n 4,2,1\""),
        (
            "suppose a = 5:\n    inner = c |> why |> holds\n    inner",
            "[12.0,6.0,5.0]",
        ),
        ("x, y = c |> why |> holds\n[x, y]", "[4.0,2.0]"),
        (
            "x, y = [c |> why |> holds, 1]\n[x, y]",
            "[[4.0,2.0,1.0],1.0]",
        ),
        ("{length} = c |> why |> holds\nlength", "3.0"),
        ("{x, y} = c |> why |> holds\n[x, y]", "[null,null]"),
        ("x, = c |> why |> holds\nx", "4.0"),
        (
            "[p] = c |> why |> holds",
            "SyntaxError: Cannot assign to [p] (line 7, column 1)",
        ),
        (
            "delete c |> why |> holds",
            "SyntaxError: Cannot delete c|>why|>holds (line 7, column 1)",
        ),
        (
            "not c |> why |> holds",
            "TypeError: Cannot explain !c (line 7, column 1)",
        ),
        ("c |> why |> holds(1)", "1.0"),
        ("c |> why |> holds[0]", "[0.0]"),
        (
            "c |> why |> holds.length",
            "SyntaxError: Unexpected '.' (line 7, column 19)",
        ),
        ("(c |>\nwhy |>\nholds)", "[4.0,2.0,1.0]"),
        ("c |> why |> holds; a", "1.0"),
        (
            "vals = c |> why |> holds\nvals.push(\"z\")\nvals",
            "[4.0,2.0,1.0,\"z\"]",
        ),
        (
            "n = null\nfor s in c |> why |> depth(n) |> holds:\n    r = s",
            "TypeError: c|>why|>depth(n)|>holds is not iterable (line 8, column 1)",
        ),
    ];
    let unchanged = [
        ("c |> why |> Holds", "TypeError: Holds is not an operation"),
        ("c |> why |> hold", "TypeError: hold is not an operation"),
        (
            "c |> why |>\nholds",
            "SyntaxError: Expected an operation (line 7, column 10)",
        ),
        (
            "holds c",
            "ReferenceError: holds is not defined (line 7, column 1)",
        ),
        (
            "holds",
            "ReferenceError: holds is not defined (line 7, column 1)",
        ),
        (
            "holds(c |> why)",
            "ReferenceError: holds is not defined (line 7, column 1)",
        ),
        ("holds = [1, 2]\nholds", "[1.0,2.0]"),
        ("x = {holds: 1}\nx.holds", "1.0"),
        ("def holds(x) => x * 2\nholds(3)", "6.0"),
        (
            "class Tank(holds): this.holds = holds\nt = Tank(2)\nt.holds",
            "2.0",
        ),
        ("(c |> why).holds", "null"),
        ("(c |> why)[0].holds", "4.0"),
        (
            "xs = [1]\nxs.holds([2])",
            "TypeError: xs.holds is not a function (line 8, column 1)",
        ),
        ("`v ${c why c |> holds}`", "\"v 4\""),
        (
            "def g(x = c |> why |> holds) => x",
            "SyntaxError: Expected a scalar literal parameter default (line 7, column 11)",
        ),
        (
            "{x: c |> why |> holds}",
            "SyntaxError: Unexpected ':' (line 7, column 5)",
        ),
        (
            "(c).map(step => step.holds)",
            "TypeError: number.map is not a function (line 7, column 1)",
        ),
        ("(c |> depth(0)).map(step => step.holds)", "[4.0]"),
        ("(c |> why |> depth(0)).map(step => step.holds)", "[4.0]"),
        ("c |> why |> nodes", "[\"c\",\"b\",\"a\"]"),
    ];
    for (last, expected) in changed.into_iter().chain(unchanged) {
        assert_eq!(
            exact(&mut Runtime::new(), &format!("{SIX}{last}")),
            expected,
            "{last}"
        );
    }
    let part = "class Part(cost): this.cost = cost\n$Part.price = $Part.cost * 2\nbolt = Part(3)\nnut = Part(1)\n";
    assert_eq!(
        result(&format!("{part}bolt.price |> why |> holds")),
        "[6.0,3.0]"
    );
    assert_eq!(
        result(&format!("{part}bolt |> holds")),
        "[{\"cost\":3.0,\"id\":\"bolt\",\"price\":6.0}]"
    );
    assert_eq!(result(&format!("{part}(bolt |> holds)[0] == bolt")), "true");
    assert_eq!(
        result(&format!(
            "{part}(bolt |> depth(0)).map(step => step.holds)[0] == bolt"
        )),
        "true"
    );
}
