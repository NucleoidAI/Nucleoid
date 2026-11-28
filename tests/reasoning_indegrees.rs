mod common;

use nucleoid::Runtime;
use nucleoid::error::{Error, ErrorKind, Position};
use nucleoid::graph::{NodeKey, ShapeKey};
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, parse, parse_expression};
use nucleoid::reasoning::Stage;

const CHAIN: &str = "a = 1\nb = a + 2\nc = b * 2\n";
const SPEC_TITLE: &str = "Nucleoid counts direct inputs in reasoning selections";
const REFERENCE_TITLE: &str = "Nucleoid keeps reasoning input counts reactive";

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "reasoning input counts".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn outcome(runtime: &mut Runtime, source: &str) -> String {
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    assert!(runtime.take_assertions().is_empty(), "{source}");
    runtime.serialize_json(&value).unwrap()
}

fn failure(source: &str) -> Error {
    Runtime::new()
        .run(source)
        .expect_err("the source must fail")
}

fn accepted(selection: &str) -> String {
    format!("({selection}).map(step -> step.from.length)")
}

fn structure(runtime: &Runtime, name: &str) -> Vec<String> {
    let mut shapes: Vec<String> = runtime
        .graph
        .shape_dependencies(&NodeKey::variable(name))
        .filter(|shape| match shape {
            ShapeKey::Node(key) => !key.as_str().starts_with("obj:"),
            ShapeKey::Model => true,
            ShapeKey::Entries(_) => false,
        })
        .map(|shape| format!("{shape:?}"))
        .collect();
    shapes.sort();
    shapes
}

fn reads(runtime: &Runtime, name: &str) -> Vec<String> {
    let mut keys: Vec<String> = runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap()
        .dependencies
        .iter()
        .filter(|key| !key.as_str().starts_with("obj:"))
        .map(ToString::to_string)
        .collect();
    keys.sort();
    keys
}

#[test]
fn the_final_stage_is_an_ordinary_identifier_with_no_prefix_lowering() {
    let expected = Expr::Reason {
        stage: Stage::Indegrees,
        source: Box::new(Expr::Reason {
            stage: Stage::Why,
            source: Box::new(Expr::Identifier("c".to_string())),
        }),
    };
    for source in [
        "c |> why |> indegrees",
        "why c |> indegrees",
        "c|>why|>indegrees",
        "(c |>\nwhy |>\nindegrees)",
    ] {
        assert_eq!(parse_expression(source).unwrap(), expected, "{source}");
    }
    assert_eq!(Stage::Indegrees.as_str(), "indegrees");
    assert_eq!(Stage::from_name("indegrees"), None);
    assert_eq!(
        tokenize("indegrees")
            .unwrap()
            .first()
            .map(|spanned| &spanned.token),
        Some(&Token::Identifier("indegrees".to_string()))
    );
    assert_eq!(
        parse_expression("indegrees").unwrap(),
        Expr::Identifier("indegrees".to_string())
    );
    assert_eq!(expected.to_string(), "c|>why|>indegrees");
    for source in ["indegrees", "indegrees c", "indegrees(c |> why)"] {
        let error = failure(&format!("{CHAIN}{source}"));
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "indegrees is not defined");
        assert_eq!(error.position(), Some(Position::new(4, 1)));
    }
}

#[test]
fn ordinary_names_members_and_calls_keep_their_meaning() {
    checked(
        &mut Runtime::new(),
        "indegrees = 7\no = {indegrees: 3}\nassert([indegrees, o.indegrees], [7, 3])",
    );
    checked(
        &mut Runtime::new(),
        "def indegrees(x) => x + 1\nclass K:\n    def indegrees() => 9\nassert([indegrees(2), K().indegrees()], [3, 9])",
    );
    assert_eq!(
        failure(&format!("{CHAIN}c |> why |> indegrees()")).kind(),
        ErrorKind::Syntax
    );
    assert_eq!(
        outcome(
            &mut Runtime::new(),
            &format!("{CHAIN}c |> why |> indegrees(1)")
        ),
        outcome(&mut Runtime::new(), &format!("{CHAIN}c |> why |> nodes(1)"))
    );
    assert_eq!(
        parse_expression("c |> why |> indegrees(1)").unwrap(),
        parse_expression("c |> why |> indegrees").unwrap()
    );
}

#[test]
fn counts_are_recorded_fan_in_in_selection_order_not_cardinality() {
    checked(
        &mut Runtime::new(),
        "a = 1\noutside = 2\nb = a + outside\nc = b * 2\nd = b + c\nassert(a |> affects |> indegrees, [2, 1, 2])\nassert(d |> why |> derived |> indegrees, [2, 2, 1])\nassert(d |> why |> stated |> indegrees, [0, 0])\nassert(b |> indegrees, [2])\nassert(d |> why |> depth(0) |> indegrees, [2])\nassert(b |> why |> except(outside |> why) |> indegrees, [2, 0])\nd = a + a\nassert(d |> indegrees, [1])",
    );
}

#[test]
fn empty_and_zero_input_steps_are_preserved() {
    checked(
        &mut Runtime::new(),
        "a = 1\nassert(a |> affects |> indegrees, [])\nassert(a |> indegrees, [0])\nassert(a |> why |> derived |> indegrees, [])\nclass K\n$K.flag = true\nk = K()\nassert((k.flag |> why)[0].state, \"derived\")\nassert((k.flag |> why)[0].from, [])\nassert(k.flag |> why |> derived |> indegrees, [0])",
    );
    checked(&mut Runtime::new(), "assert(model |> indegrees, [])");
}

#[test]
fn all_selection_stages_match_the_accepted_map_projection() {
    let setup = format!("{CHAIN}d = a + 10\n");
    for selection in [
        "c |> why",
        "why c",
        "a |> affects",
        "affects a",
        "stated a",
        "derived c",
        "c |> why |> stated",
        "c |> why |> derived",
        "c |> why |> depth(0)",
        "c |> why |> depth(1)",
        "a |> affects |> depth(1)",
        "c |> why |> path(a)",
        "a |> affects |> path(c)",
        "c |> why |> origins",
        "a |> affects |> outcomes",
        "c |> why |> intersect(d |> why)",
        "c |> why |> except(d |> why)",
        "c |> why |> union(d |> why)",
        "c |> why |> xor(d |> why)",
        "c |> why |> depth(1) |> union(d |> why) |> depth(1)",
        "c |> why |> depth(2) |> xor(d |> why) |> stated",
        "model |> depth(0)",
        "model |> stated",
        "model |> derived",
        "model |> origins",
        "model |> outcomes",
        "model |> intersect(c |> why)",
        "c |> why |> depth(null)",
    ] {
        let written = format!("{setup}{selection} |> indegrees");
        let control = format!("{setup}{}", accepted(selection));
        assert_eq!(
            outcome(&mut Runtime::new(), &written),
            outcome(&mut Runtime::new(), &control),
            "{selection}"
        );
    }
}

#[test]
fn exactly_two_paired_document_cases_keep_all_six_assertions_and_returns() {
    for (document, title, written, spelling, count) in [
        (
            include_str!("../nucleoid.spec.md"),
            SPEC_TITLE,
            "counts = total |> why |> indegrees",
            "counts = (total |> why).map(step -> step.from.length)",
            898,
        ),
        (
            include_str!("reference.md"),
            REFERENCE_TITLE,
            "impact = source |> affects |> indegrees",
            "impact = (source |> affects).map(step -> step.from.length)",
            175,
        ),
    ] {
        let cases = common::cases(document);
        assert_eq!(cases.len(), count);
        assert_eq!(cases.get(cases.len() - 5).unwrap().title, title);
        assert_eq!(cases.iter().filter(|case| case.title == title).count(), 1);
        let case = cases.get(cases.len() - 5).unwrap();
        assert_eq!(case.source.matches(written).count(), 1);
        assert_eq!(case.expected.as_deref(), Some("1"));
        let assertions = common::assertions::protected_assertions(&parse(&case.source).unwrap());
        assert_eq!(assertions.len(), 6);
        let control = common::Case {
            title: case.title.clone(),
            source: case.source.replacen(written, spelling, 1),
            expected: case.expected.clone(),
        };
        assert_eq!(
            common::assertions::protected_assertions(&parse(&control.source).unwrap()),
            assertions
        );
        common::check_with_assertions(case, 6).unwrap();
        common::check_with_assertions(&control, 6).unwrap();
    }
}

#[test]
fn equal_valued_rewiring_refreshes_counts_and_drops_old_subscriptions() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}counts = c |> why |> indegrees\nassert(counts, [1, 1, 0])"),
    );
    checked(
        &mut runtime,
        "c = a + b + 2\nassert(c, 6)\nassert(counts, [2, 0, 1])\nc = 6\nassert(c, 6)\nassert(counts, [0])\nb = a + 5\nassert(counts, [0])",
    );
    assert_eq!(
        runtime
            .graph
            .shape_dependencies(&NodeKey::variable("counts"))
            .collect::<Vec<_>>(),
        [&ShapeKey::Node(NodeKey::variable("c"))]
    );
}

#[test]
fn an_empty_impact_keeps_its_source_for_deletion_and_reattachment() {
    checked(
        &mut Runtime::new(),
        "a = 1\nb = a + 1\nimpact = a |> affects |> indegrees\nassert(impact, [1])\nb = 0\nassert(impact, [])\nb = a + 1\nassert(impact, [1])\ndelete b\nassert(impact, [])\nb = a * 2\nassert(impact, [1])",
    );
}

#[test]
fn structural_and_value_dependencies_match_the_accepted_control() {
    let mut written = Runtime::new();
    let mut control = Runtime::new();
    checked(
        &mut written,
        &format!("{CHAIN}counts = c |> why |> indegrees"),
    );
    checked(
        &mut control,
        &format!("{CHAIN}counts = {}", accepted("c |> why")),
    );
    for update in [
        "",
        "a = 4",
        "c = a + b",
        "d = 3\nb = a + d",
        "b = d",
        "c = 8",
    ] {
        checked(&mut written, update);
        checked(&mut control, update);
        assert_eq!(
            outcome(&mut written, "counts"),
            outcome(&mut control, "counts"),
            "{update}"
        );
        assert_eq!(
            structure(&written, "counts"),
            structure(&control, "counts"),
            "{update}"
        );
        assert_eq!(
            reads(&written, "counts"),
            reads(&control, "counts"),
            "{update}"
        );
    }
}

#[test]
fn projections_do_not_select_their_observers_or_change_step_metadata() {
    checked(
        &mut Runtime::new(),
        &format!(
            "{CHAIN}counts = c |> why |> indegrees\ntrace = c |> why\nassert(model |> nodes, [\"a\", \"b\", \"c\"])\nassert(a |> affects |> nodes, [\"b\", \"c\"])\nassert(counts |> why |> indegrees, [])\nassert(trace[0].node, \"c\")\nassert(trace[0].holds, 6)\nassert(trace[0].rule, \"c = b*2\")\nassert(trace[0].state, \"derived\")\nassert(trace[0].from, [\"b\"])\nassert((c |> why |> indegrees).length, 3)"
        ),
    );
}

#[test]
fn reference_coercion_never_consumes_a_stored_list_of_records() {
    checked(
        &mut Runtime::new(),
        &format!(
            "{CHAIN}steps = c |> why\nordinary = [1, 2, 3]\nassert(steps |> indegrees, [])\nassert(ordinary |> indegrees, [0])\nassert(c |> indegrees, [1])"
        ),
    );
}

#[test]
fn functions_methods_and_class_rules_share_the_existing_projection_context() {
    let mut written = Runtime::new();
    let mut control = Runtime::new();
    let setup = "rate = 2\nclass M(load):\n    this.load = load\n    def inputs() => this.wear |> why |> indegrees\n$M.wear = $M.load * rate\n$M.fan = $M.wear |> why |> indegrees\nm = M(3)\ndef inputs(o) => o.wear |> why |> indegrees\n";
    let spelling = setup
        .replace(
            "this.wear |> why |> indegrees",
            &accepted("this.wear |> why"),
        )
        .replace("$M.wear |> why |> indegrees", &accepted("$M.wear |> why"))
        .replace("o.wear |> why |> indegrees", &accepted("o.wear |> why"));
    checked(&mut written, setup);
    checked(&mut control, &spelling);
    for update in ["", "rate = 3", "$M.wear = $M.load + rate", "m.load = 9"] {
        checked(&mut written, update);
        checked(&mut control, update);
        assert_eq!(
            outcome(&mut written, "[inputs(m), m.fan, m.inputs()]"),
            outcome(&mut control, "[inputs(m), m.fan, m.inputs()]"),
            "{update}"
        );
    }
}

#[test]
fn snapshots_hypotheses_and_diff_use_the_accepted_selection_semantics() {
    let source = format!(
        "{CHAIN}counts = c |> why |> indegrees\nfrozen = counts.value\nc = a + b\nassert(counts, [2, 0, 1])\nassert(frozen, [1, 1, 0])\nsuppose b = a * 4:\n    assert(c |> why |> diff |> indegrees, [2, 1])\nassert(counts, [2, 0, 1])"
    );
    checked(&mut Runtime::new(), &source);
    let error = failure(&format!("{CHAIN}c |> why |> diff |> indegrees"));
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Cannot use 'diff' outside suppose");
}

#[test]
fn null_missing_and_undefined_sources_keep_projection_behavior() {
    checked(
        &mut Runtime::new(),
        &format!(
            "{CHAIN}n = null\no = {{}}\nassert(n |> indegrees, [0])\nassert(o.gone |> why |> indegrees, [])\nassert(typeof (c |> why |> depth(null) |> indegrees), \"undefined\")\nlimit = 1\ncounts = c |> why |> depth(limit) |> indegrees\nassert(counts, [1, 1])\nlimit = null\nassert(typeof counts, \"null\")\nlimit = 0\nassert(counts, [1])"
        ),
    );
    for (source, kind, message) in [
        (
            "missing |> indegrees",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        ("1 |> indegrees", ErrorKind::Type, "Cannot explain 1"),
        (
            "c |> why |> depth(1.5) |> indegrees",
            ErrorKind::Type,
            "1.5 is not a non-negative integer",
        ),
    ] {
        let error = failure(&format!("{CHAIN}{source}"));
        assert_eq!(error.kind(), kind, "{source}");
        assert_eq!(error.message(), message, "{source}");
    }
}

#[test]
fn every_binding_named_indegrees_shadows_the_final_stage() {
    for binding in [
        "indegrees = 1\nc |> why |> indegrees",
        "def indegrees(x) => x\nc |> why |> indegrees",
        "class indegrees\nc |> why |> indegrees",
        "indegrees = 1\nmissing |> indegrees",
        "indegrees = 1\n1 |> indegrees",
        "def f(indegrees) => c |> why |> indegrees\nf(1)",
        "f = indegrees -> c |> why |> indegrees\nf(1)",
        "f = lambda indegrees: c |> why |> indegrees\nf(1)",
        "[1].map(indegrees -> c |> why |> indegrees)",
        "class R:\n    def m(indegrees) => c |> why |> indegrees\nR().m(1)",
        "try:\n    throw 1\ncatch indegrees:\n    c |> why |> indegrees",
        "{indegrees} = {indegrees: 1}\nc |> why |> indegrees",
    ] {
        let error = failure(&format!("{CHAIN}{binding}"));
        assert_eq!(error.kind(), ErrorKind::Type, "{binding}");
        assert_eq!(
            error.message(),
            "indegrees is not an operation",
            "{binding}"
        );
    }
    checked(
        &mut Runtime::new(),
        &format!(
            "{CHAIN}why = 1\naffects = 2\nassert(c |> why |> indegrees, [1, 1, 0])\nassert(a |> affects |> indegrees, [1, 1])"
        ),
    );
}

#[test]
fn final_boundaries_and_shadow_error_precedence_match_other_projections() {
    for stage in [
        "why",
        "affects",
        "stated",
        "derived",
        "nodes",
        "holds",
        "edges",
        "indegrees",
        "origins",
        "outcomes",
        "depth(1)",
        "path(a)",
    ] {
        let error = failure(&format!("{CHAIN}c |> why |> indegrees |> {stage}"));
        assert_eq!(error.kind(), ErrorKind::Type, "{stage}");
        assert_eq!(
            error.message(),
            "Cannot explain c|>why|>indegrees",
            "{stage}"
        );
    }
    for operation in ["intersect", "except", "union", "xor", "path"] {
        let error = failure(&format!(
            "{CHAIN}c |> why |> {operation}(a |> why |> indegrees)"
        ));
        assert_eq!(error.kind(), ErrorKind::Type, "{operation}");
        assert_eq!(
            error.message(),
            "Cannot explain a|>why|>indegrees",
            "{operation}"
        );
    }
    let error = failure(&format!(
        "{CHAIN}stated = 1\nc |> why |> indegrees |> stated"
    ));
    assert_eq!(error.message(), "stated is not an operation");
    let error = failure(&format!(
        "{CHAIN}indegrees = 1\nc |> why |> indegrees |> why"
    ));
    assert_eq!(error.message(), "Cannot explain c|>why|>indegrees");
    for stage in ["nodes", "holds", "edges"] {
        let error = failure(&format!("{CHAIN}c |> why |> {stage} |> indegrees"));
        assert_eq!(error.message(), format!("Cannot explain c|>why|>{stage}"));
    }
}

#[test]
fn old_final_projections_and_map_spelling_work_even_with_an_indegrees_binding() {
    checked(
        &mut Runtime::new(),
        &format!(
            "{CHAIN}indegrees = 1\nassert(c |> why |> nodes, [\"c\", \"b\", \"a\"])\nassert(c |> why |> holds, [6, 3, 1])\nassert(c |> why |> edges, [[\"b\", \"c\"], [\"a\", \"b\"]])\nassert((c |> why).map(step -> step.from.length), [1, 1, 0])\nassert(`${{a 2 |> indegrees}}`, \"1\")"
        ),
    );
}

#[test]
fn a_failed_projection_assignment_rolls_back_and_chain_limits_stay_bounded() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        &format!("{CHAIN}counts = c |> why |> indegrees"),
    );
    let error = runtime.run("counts = 1 |> indegrees").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Cannot explain 1");
    checked(
        &mut runtime,
        "assert(counts, [1, 1, 0])\nc = a + b\nassert(counts, [2, 0, 1])",
    );
    let error = parse(&format!("a{}", " |> indegrees".repeat(2_000))).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(error.message(), "Expression has too many operands");
}
