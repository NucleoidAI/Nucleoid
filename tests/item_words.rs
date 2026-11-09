mod common;

use nucleoid::graph::{NodeKey, NodeKind};
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{
    Expr, FunctionBody, Stmt, UnaryOp, parse, parse_expression, parse_program,
};
use nucleoid::{ErrorKind, Position, Runtime, Value};
use regex::Regex;

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn word(name: &str) -> Expr {
    Expr::Unary {
        operator: UnaryOp::Word,
        operand: Box::new(Expr::Identifier(name.to_string())),
    }
}

fn assign(target: Expr, value: Expr) -> Stmt {
    Stmt::Assign { target, value }
}

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn tree(source: &str) -> String {
    format!("{:?}", statements(source))
}

fn marker() -> Regex {
    Regex::new(r#"Unary \{ operator: Word, operand: Identifier\("([^"]*)"\) \}"#).unwrap()
}

fn words_in(tree: &str) -> Vec<String> {
    marker()
        .captures_iter(tree)
        .map(|captures| captures[1].to_string())
        .collect()
}

fn unmarked(tree: &str) -> String {
    marker()
        .replace_all(tree, r#"Identifier("$1")"#)
        .into_owned()
}

fn words(source: &str) -> Vec<String> {
    words_in(&tree(source))
}

fn equals(source: &str) -> String {
    source.replacen(" is ", " = ", 1)
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn assert_diagnostics_of_equals(source: &str) {
    let (message, position) = syntax_error(source);
    let (expected, at) = syntax_error(&equals(source));
    assert_eq!(message, expected, "{source}");
    assert_eq!(position.line, at.line, "{source}");
    assert!(
        position.column == at.column + 1 || (position.line > 1 && position.column == at.column),
        "{source}: {position:?} against {at:?}"
    );
}

fn expect(runtime: &mut Runtime, source: &str, expected: &str) {
    let parsed = statements(source);
    let assertions = common::assertions::protected_assertions(&parsed).len();
    let case = common::Case {
        title: "item word regression".to_string(),
        source: source.to_string(),
        expected: Some(expected.to_string()),
    };
    common::check_with_count(runtime, &case, assertions)
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn fresh(source: &str, expected: &str) {
    expect(&mut Runtime::new(), source, expected);
}

fn failure(runtime: &mut Runtime, source: &str) -> String {
    runtime.run(source).unwrap_err().to_string()
}

fn json(runtime: &mut Runtime, source: &str) -> String {
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    runtime.serialize_json(&value).unwrap()
}

fn run_ok(runtime: &mut Runtime, source: &str) -> Value {
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn kind(runtime: &Runtime, name: &str) -> Option<NodeKind> {
    runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .map(|node| node.kind)
}

#[test]
fn items_of_one_identifier_token_are_words_wherever_their_literal_or_call_stands() {
    let label = |value: Expr| Expr::Call {
        callee: Box::new(name("Light")),
        arguments: vec![Expr::Assign {
            target: Box::new(name("color")),
            value: Box::new(value),
        }],
    };
    for (source, expected) in [
        (
            "a is [A, B]",
            assign(name("a"), Expr::List(vec![word("A"), word("B")])),
        ),
        (
            "a is {k: A}",
            assign(
                name("a"),
                Expr::ObjectLiteral(vec![("k".to_string(), word("A"))]),
            ),
        ),
        ("a is Light(color=A)", assign(name("a"), label(word("A")))),
        ("a is Light(color=(A))", assign(name("a"), label(name("A")))),
    ] {
        assert_eq!(statements(source), vec![expected], "{source}");
    }
    for (source, expected) in [
        (
            "$Light.partner is [[RED, GREEN], [BLUE, AMBER]].find(pair => pair.includes($Light.color@first)).find(other => other is not $Light.color@first)",
            &["RED", "GREEN", "BLUE", "AMBER"][..],
        ),
        (
            "$Light.glow is {small: DIM, large: BRIGHT, s7: OFF}[$Light.size@first]",
            &["DIM", "BRIGHT", "OFF"],
        ),
        (
            "$Light.shade is [AMBER, RED, GREEN][$Light.x@first]",
            &["AMBER", "RED", "GREEN"],
        ),
        (
            "$Light.corners is [[RED, BLUE], [GREEN, AMBER]]",
            &["RED", "BLUE", "GREEN", "AMBER"],
        ),
        ("first is Light(x=0, size=4, color=RED, period=4)", &["RED"]),
        (
            "second is Light(x=0, parts=[RED, RED, RED])",
            &["RED", "RED", "RED"],
        ),
        ("third is Light(x=0, tag=s7)", &["s7"]),
        ("a is [A, 1, \"B\", 'C', (D), E.x, F(), G[0], -H]", &["A"]),
        ("a is {k: A, 'q': B, 7: C, D}", &["A", "B", "C"]),
        ("a is {k: A j: B}", &["A", "B"]),
        ("a is {...o, k: A}", &["A"]),
        ("a is Light(A, color=B)", &["B"]),
        ("a is new Light(color=A)", &["A"]),
        ("a is pick(color=A)", &["A"]),
        ("a is (Light)(color=A)", &["A"]),
        ("a is super(color=A)", &["A"]),
        ("a is xs.concat([A])", &["A"]),
        ("a is pick([A], {k: B})", &["A", "B"]),
        ("a is [A] + [B]", &["A", "B"]),
        ("a is not [A]", &["A"]),
        ("a is ([A, B])", &["A", "B"]),
        ("a is [[A], {k: [B]}]", &["A", "B"]),
        ("a is {k: {j: A}}", &["A"]),
        ("a is [A].length", &["A"]),
        ("a is [A][0]", &["A"]),
        ("a is {k: A}.k", &["A"]),
        ("a is xs[[A].length]", &["A"]),
        ("a is xs[[A][0]:[B][0]]", &["A", "B"]),
        ("a is X if [A].includes(c) else Y", &["A", "X", "Y"]),
        ("a is [A] ?? [B]", &["A", "B"]),
        ("a is all x in [A]: x", &["A"]),
        ("a is any x in xs: [A].includes(x)", &["A"]),
        ("a is [x for x in [A]]", &["A"]),
        ("a is xs |> depth([A].length)", &["A"]),
        ("a is Light(color=[A, {j: B}])", &["A", "B"]),
        ("a is [Light(color=A)]", &["A"]),
        ("a is Light(color=Light(color=A))", &["A"]),
        ("a is {...{k: A}}", &["A"]),
        (
            "a is [is, suppose, lambda, model, why]",
            &["is", "suppose", "lambda", "model", "why"],
        ),
        (
            "a is [true, null, this, self, None, undefined, $Light, A@first]",
            &[],
        ),
        ("o.x is [A]", &["A"]),
        ("this.p is {k: A}", &["A"]),
        ("(a) is [A]", &["A"]),
    ] {
        let expected: Vec<String> = expected.iter().map(ToString::to_string).collect();
        assert_eq!(words(source), expected, "{source}");
        assert_eq!(unmarked(&tree(source)), tree(&equals(source)), "{source}");
    }
    assert_eq!(
        statements("a is [A] b is [B]"),
        vec![
            assign(name("a"), Expr::List(vec![word("A")])),
            assign(name("b"), Expr::List(vec![word("B")])),
        ]
    );
}

#[test]
fn bodies_interpolations_inner_assignments_and_targets_hold_no_item_words() {
    for source in [
        "a is (A)",
        "a is [(A)]",
        "a is [((A))]",
        "a is {k: (A)}",
        "a is Light(color=(A))",
        "a is pick(A)",
        "a is pick(A, B)",
        "a is o.m(color=A)",
        "a is o.m(color=[A])",
        "a is pick()(color=A)",
        "a is xs[0](color=A)",
        "a is Light((color=A))",
        "a is Light(o.k=A)",
        "a is Light(color+=A)",
        "a is {A}",
        "a is {A, B}",
        "a is {...A}",
        "a is xs[A]",
        "a is xs[A:B]",
        "a is [A.x, A(), A[0], A@first, -A, !A, not A, typeof A, A + 1, A ?? B]",
        "a is [\"A\", 'B', 1, true, null, this]",
        "a is x => [A]",
        "a is x => { b = [A] }",
        "a is lambda x: [A]",
        "a is function () { return [A] }",
        "a is `${[A]}`",
        "a is `${pick(color=A)}`",
        "a is b = [A]",
        "a is b = Light(color=A)",
        "a is b += [A]",
        "a is [A][0] = 1",
        "a is Light(color=A).x = 1",
        "a is X if c else [A] = 5",
        "a is [x for x in xs]",
        "a is [A for x in xs]",
        "a is [[A] for x in xs]",
        "a is [x for x in xs if [A].includes(x)]",
        "a is [A if c else B for c in cs]",
        "a = [A]",
        "a = {k: A}",
        "a = Light(color=A)",
        "a := [A]",
        "a += [A]",
        "[A]",
        "Light(color=A)",
        "x = [A] if c else [B]",
        "if c:\n    a = [A]",
        "def f() => [A]",
        "throw [A]",
        "assert: [A], [A]",
        "{a, b} = [A, B]",
        "a, b = [A, B]",
        "while [A]:\n    pass",
    ] {
        assert!(words(source).is_empty(), "{source}");
        if source.starts_with("a is ") {
            assert_eq!(statements(source), statements(&equals(source)), "{source}");
        }
    }
    assert_eq!(
        parse_expression("[A, Light(color=B), {k: C}]").unwrap(),
        Expr::List(vec![
            name("A"),
            Expr::Call {
                callee: Box::new(name("Light")),
                arguments: vec![Expr::Assign {
                    target: Box::new(name("color")),
                    value: Box::new(name("B")),
                }],
            },
            Expr::ObjectLiteral(vec![("k".to_string(), name("C"))]),
        ])
    );
}

#[test]
fn item_positions_resume_after_every_excluded_part() {
    for (source, expected) in [
        ("a is [x => x, A]", &["A"][..]),
        ("a is [x => { b = [B] }, A]", &["A"]),
        ("a is [lambda x: [B], A]", &["A"]),
        ("a is [function () { return [B] }, A]", &["A"]),
        ("a is [b = [B], A]", &["A"]),
        ("a is [b += 1, A]", &["A"]),
        ("a is [[B][0] = 1, A]", &["A"]),
        ("a is [[v for v in xs if [B].includes(v)], A]", &["A"]),
        ("a is [[[C] for v in [B]], A]", &["B", "A"]),
        ("a is [`${[B]}`, A]", &["A"]),
        ("a is {k: x => [B], j: A}", &["A"]),
        ("a is pick(k=x => [B], j=A)", &["A"]),
        ("a is Light(color=(b = [B]), size=A)", &["A"]),
        ("a is X if [x => B].length else [A]", &["X", "A"]),
    ] {
        let expected: Vec<String> = expected.iter().map(ToString::to_string).collect();
        assert_eq!(words(source), expected, "{source}");
        assert_eq!(unmarked(&tree(source)), tree(&equals(source)), "{source}");
    }
}

#[test]
fn every_value_without_item_words_builds_the_tree_of_its_equals_spelling() {
    let targets = ["a", "o.x", "this.p", "$Order.p", "(a)", "a@first"];
    let values = [
        "[1, \"x\"]",
        "[(good)]",
        "{k: (good)}",
        "{good}",
        "{k: 1}",
        "{...good}",
        "[[1], {k: [\"x\"]}]",
        "x => [x]",
        "lambda x: [x]",
        "Order(total=1)",
        "Order(total=(good))",
        "pick(good)",
        "o.m(k=good)",
        "(c ? x : y)",
        "good ?? 1",
        "good |> why",
        "[x * 2 for x in xs]",
        "b = [c]",
        "b += 1",
        "all x in xs: x > 0",
        "sum of xs",
        "`${[good]}`",
        "super(1)",
        "new Date(0)",
    ];
    for target in targets {
        for value in values {
            let spelled = format!("{target} is {value}");
            assert_eq!(
                statements(&spelled),
                statements(&format!("{target} = {value}")),
                "{spelled}"
            );
        }
    }
}

#[test]
fn item_positions_are_word_positions_for_conditional_words() {
    for (source, expected) in [
        ("a is [A]", &["A"][..]),
        ("a is A if c else B", &["A", "B"]),
        ("a is [A if c else B]", &["A", "B"]),
        ("a is [A] if c else B", &["A", "B"]),
        ("a is [A] if c else {k: B}", &["A", "B"]),
        ("a is Light(color=A) if c else [B]", &["A", "B"]),
        ("a is c ? pick(k=A) : {j: [B]}", &["A", "B"]),
        ("a is {k: c ? A : B}", &["A", "B"]),
        ("a is Light(color=A if c else B)", &["A", "B"]),
        ("a is [c ? [A] : B]", &["A", "B"]),
        ("a is [(A if c else B)]", &[]),
        ("a is [A if c else (B)]", &["A"]),
        ("a is Light(color=c ? (A) : B)", &["B"]),
        ("a is {k: A if c1 else B if c2 else C}", &["A", "B", "C"]),
        ("a is [A] if [B].includes(c) else C", &["B", "A", "C"]),
        ("a is X if c else [A if d else B]", &["X", "A", "B"]),
        ("a is ([A] if c else B)", &["A"]),
    ] {
        let expected: Vec<String> = expected.iter().map(ToString::to_string).collect();
        assert_eq!(words(source), expected, "{source}");
        assert_eq!(unmarked(&tree(source)), tree(&equals(source)), "{source}");
    }
}

#[test]
fn item_words_render_with_is_and_read_back_as_the_same_statement() {
    for (source, rendered) in [
        ("a is [A, B]", "a is [A,B]"),
        ("a is [A, (B)]", "a is [A,(B)]"),
        ("a is [(A)]", "a=[A]"),
        ("a is {k: A}", "a is {\"k\":A}"),
        ("a is {k: A, B}", "a is {\"k\":A,\"B\":(B)}"),
        ("a is {...o, k: A}", "a is {...o,\"k\":A}"),
        ("a is Light(color=A)", "a is Light(color=A)"),
        (
            "a is Light(B, color=(A), size=C)",
            "a is Light(B,color=(A),size=C)",
        ),
        (
            "a is Light((color=A), size=B)",
            "a is Light((color=A),size=B)",
        ),
        ("a is o.m(k=A, [B])", "a is o.m(k=A,[B])"),
        ("a is [AMBER, RED, GREEN][x]", "a is [AMBER,RED,GREEN][x]"),
        (
            "a is {small: DIM, s7: OFF}[x]",
            "a is {\"small\":DIM,\"s7\":OFF}[x]",
        ),
        (
            "a is [[RED, BLUE], [GREEN, AMBER]]",
            "a is [[RED,BLUE],[GREEN,AMBER]]",
        ),
        ("a is [A] if c else B", "a is c?[A]:B"),
        ("a is [A] if c else 1", "a is (c?[A]:1)"),
        (
            "a is X if [A].includes(c) else Y",
            "a is [A].includes(c)?X:Y",
        ),
        ("a is [A if c else B]", "a is [c?A:B]"),
        ("a is [c ? (A) : B]", "a is [c?(A):B]"),
        ("a is [(A if c else B), D]", "a is [(c?A:B),D]"),
        ("a is Light(color=A if c else B)", "a is Light(color=c?A:B)"),
        ("a is [A, b = B]", "a is [A,b=B]"),
        ("a is xs[[A][0]]", "a is xs[[A][0]]"),
        ("a is [A] + [1]", "a is [A]+[1]"),
        ("a is good", "a is good"),
        ("{ total is [A, (B)] }", "{total is [A,(B)]}"),
        ("if c:\n    a is [A]", "if(c){a is [A]}"),
        ("a is suppose:\n    b is [B]", "a=suppose{b is [B]}"),
    ] {
        let parsed = statements(source);
        assert_eq!(generate_all(&parsed), rendered, "{source}");
        if !rendered.starts_with("if(") && !rendered.contains("suppose") {
            assert_eq!(statements(rendered), parsed, "{rendered}");
        }
    }
    assert_ne!(
        generate_all(&statements("{ a is [A] }")),
        generate_all(&statements("{ a = [A] }"))
    );
    assert_eq!(
        generate_all(&statements("{ a is [(A)] }")),
        generate_all(&statements("{ a = [A] }"))
    );
}

#[test]
fn values_whose_words_are_all_branch_words_keep_the_rendering_of_conditional_words() {
    for (source, rendered) in [
        ("a is [(A)] if c else B", "a is c?[A]:B"),
        ("a is {A} if c else B", "a is c?{\"A\":A}:B"),
        (
            "a is X if [(A)].includes(c) else Y",
            "a is [A].includes(c)?X:Y",
        ),
        (
            "a is Light(color=(A)) if c else B",
            "a is c?Light(color=A):B",
        ),
    ] {
        let parsed = statements(source);
        assert!(!words(source).contains(&"A".to_string()), "{source}");
        assert!(!words(source).is_empty(), "{source}");
        assert_eq!(generate_all(&parsed), rendered, "{source}");
        assert!(words(rendered).contains(&"A".to_string()), "{rendered}");
        assert_eq!(
            unmarked(&tree(rendered)),
            unmarked(&tree(source)),
            "{rendered}"
        );
    }
}

#[test]
fn an_item_word_reads_its_variable_once_defined_and_its_text_otherwise() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "xs is [A, B]\nm is {k: C, j: [D]}\nq is ['A', \"B\"]\nassert([xs, m, q], [[\"A\", \"B\"], {\"k\": \"C\", \"j\": [\"D\"]}, [\"A\", \"B\"]])\nxs",
        "[\"A\",\"B\"]",
    );
    for (word, dependent) in [("A", "xs"), ("B", "xs"), ("C", "m"), ("D", "m")] {
        let node = runtime
            .graph
            .retrieve(&NodeKey::variable(word))
            .expect("an undefined item word is a pending node");
        assert_eq!(node.kind, NodeKind::Pending, "{word}");
        assert!(
            node.dependents.contains(&NodeKey::variable(dependent)),
            "{word}"
        );
    }
    assert!(
        !runtime
            .graph
            .retrieve(&NodeKey::variable("A"))
            .unwrap()
            .dependents
            .contains(&NodeKey::variable("q"))
    );
    expect(
        &mut runtime,
        "A = 1\nassert(xs, [1, \"B\"])\nxs",
        "[1,\"B\"]",
    );
    expect(&mut runtime, "A = [2]\nxs", "[[2],\"B\"]");
    expect(&mut runtime, "C = 3\nm", "{\"k\":3,\"j\":[\"D\"]}");
    expect(&mut runtime, "D = null\nm", "null");
    assert_eq!(
        json(&mut Runtime::new(), "D = null\nm = {k: 3, j: [D]}\nm"),
        "null"
    );
    expect(
        &mut runtime,
        "delete A\ndelete C\ndelete D\n[xs, m]",
        "[[\"A\",\"B\"],{\"k\":\"C\",\"j\":[\"D\"]}]",
    );
    assert_eq!(kind(&runtime, "A"), Some(NodeKind::Pending));
    fresh(
        "A = 2\nxs is [A, B]\nassert(xs, [2, \"B\"])\nA = 5\nassert(xs, [5, \"B\"])\nxs[0]",
        "5",
    );
    fresh(
        "s7 = 7\nm is {s7: C, 'q': s7}\nassert(m, {\"s7\": \"C\", \"q\": 7})\nm.s7",
        "\"C\"",
    );
    for (source, missing) in [("z = [A, B]", "A"), ("m = {s7: C}", "C")] {
        assert_eq!(
            failure(&mut Runtime::new(), source),
            format!("ReferenceError: {missing} is not defined (line 1, column 1)"),
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "good = 3\nxs is [1, good]\nys = [1, good]\nassert([xs, ys], [[1, 3], [1, 3]])\nxs",
        "[1,3]",
    );
    let why = json(&mut runtime, "xs |> why");
    assert!(why.contains("\"rule\":\"xs is [1,good]\""), "{why}");
    let why = json(&mut runtime, "ys |> why");
    assert!(why.contains("\"rule\":\"ys = [1,good]\""), "{why}");
    expect(
        &mut runtime,
        "delete good\nassert(xs, [1, \"good\"])\nassert(typeof ys[1], \"undefined\")\nxs",
        "[1,\"good\"]",
    );
}

#[test]
fn only_items_that_are_evaluated_are_read() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "c = true\na is [A] if c else [Z]\nb is false && [D]\ne is null ?? [E]\nf is 1 ?? [F]\nassert([a, b, e, f], [[\"A\"], false, [\"E\"], 1])\na",
        "[\"A\"]",
    );
    assert_eq!(kind(&runtime, "A"), Some(NodeKind::Pending));
    assert_eq!(kind(&runtime, "E"), Some(NodeKind::Pending));
    for unread in ["Z", "D", "F"] {
        assert_eq!(kind(&runtime, unread), None, "{unread}");
    }
    expect(&mut runtime, "c = false\na", "[\"Z\"]");
    assert_eq!(kind(&runtime, "Z"), Some(NodeKind::Pending));
    let mut runtime = Runtime::new();
    run_ok(&mut runtime, "class Light(color):\n    this.color is color");
    assert_eq!(
        failure(&mut runtime, "first is Light(colour=GREY)"),
        "TypeError: Unknown constructor argument 'colour' for Light (line 1, column 1)"
    );
    assert_eq!(kind(&runtime, "GREY"), None);
}

#[test]
fn constructors_and_class_level_rules_take_item_words_per_instance() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "class Light(size, x):\n    this.size is size\n    this.x is x\n    this.corners is [[RED, BLUE], [GREEN, AMBER]]\n$Light.glow is {small: BLUE, large: GREEN}[$Light.size@first]\n$Light.shade is [RED, BLUE, GREEN][$Light.x@first]\nfirst = Light(\"small\", 0)\nsecond is Light(size=large, x=2)\nassert([first.glow, second.glow], [\"BLUE\", \"GREEN\"])\nassert([first.shade, second.shade], [\"RED\", \"GREEN\"])\nassert(first.corners, [[\"RED\", \"BLUE\"], [\"GREEN\", \"AMBER\"]])\nsecond.size",
        "\"large\"",
    );
    expect(
        &mut runtime,
        "GREEN = \"green\"\nassert([second.glow, second.shade], [\"green\", \"green\"])\nfirst.corners",
        "[[\"RED\",\"BLUE\"],[\"green\",\"AMBER\"]]",
    );
    expect(
        &mut runtime,
        "delete GREEN\n[second.glow, first.corners[1][0]]",
        "[\"GREEN\",\"GREEN\"]",
    );
    for (rule, missing) in [
        ("$Light.bad is [(MISSING)]", "MISSING"),
        ("$Light.bad is {k: MISSING.x}", "MISSING"),
        ("$Light.bad is [RED, OTHER](MISSING)", "MISSING"),
    ] {
        assert_eq!(
            failure(&mut runtime, rule),
            format!("ReferenceError: {missing} is not defined (line 1, column 1)"),
            "{rule}"
        );
    }
    assert_eq!(
        failure(
            &mut Runtime::new(),
            "class Light(size):\n    this.size = size\n$Light.bad = [MISSING]"
        ),
        "ReferenceError: MISSING is not defined (line 3, column 1)"
    );
    expect(
        &mut runtime,
        "$Light.ok is [MISSING, {k: OTHER}]\n[first.ok, second.ok[1].k]",
        "[[\"MISSING\",{\"k\":\"OTHER\"}],\"OTHER\"]",
    );
    fresh(
        "class Light(color, parts):\n    this.color is color\n    this.parts is parts\nnamed is Light(color=BLUE, parts=[BLUE, GREEN])\nassert([named.color, named.parts], [\"BLUE\", [\"BLUE\", \"GREEN\"]])\nBLUE = \"blue\"\nassert([named.color, named.parts], [\"BLUE\", [\"BLUE\", \"GREEN\"]])\nnamed.color",
        "\"BLUE\"",
    );
    fresh(
        "class Light(color):\n    this.color is color\nlights is [Light(color=BLUE)]\nassert(lights[0].color, \"BLUE\")\nBLUE = \"blue\"\nlights[0].color",
        "\"blue\"",
    );
}

#[test]
fn words_that_name_a_local_class_function_or_global_read_them() {
    fresh(
        "def pick(EXACT):\n    found is [EXACT, OTHER]\n    return found\nassert(pick(5), [5, \"OTHER\"])\npick(1)[1]",
        "\"OTHER\"",
    );
    fresh(
        "class Order:\n    pass\ndef helper() => 7\nk is [Order, helper, {m: Math}]\nassert(k[0] == Order, true)\nassert(k[1](), 7)\nassert(k[2].m.PI, Math.PI)\ntypeof k[1]",
        "\"Function\"",
    );
    fresh(
        "h is [later]\nassert(h, [\"later\"])\ndef later() => 8\nassert(h[0](), 8)\nh[0]()",
        "8",
    );
    fresh(
        "def pick(color) => color\nr is pick(color=A)\nassert([r, color], [\"A\", \"A\"])\nr",
        "\"A\"",
    );
    assert_eq!(
        failure(&mut Runtime::new(), "o = {m: (v) => v}\nr is o.m(color=A)"),
        "ReferenceError: A is not defined (line 2, column 1)"
    );
}

#[test]
fn hypotheticals_and_rollback_restore_item_words() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "xs is [A, B]\nsuppose A = 9:\n    assert(xs, [9, \"B\"])\nassert(xs, [\"A\", \"B\"])\nxs",
        "[\"A\",\"B\"]",
    );
    expect(
        &mut runtime,
        "result is suppose A = 2:\n    xs[0] * 10\nassert(result, 20)\nresult",
        "20",
    );
    assert!(!runtime.state.has_variable("A"));
    expect(
        &mut runtime,
        "def check(v):\n    if v == 5:\n        throw \"FIVE\"\n    return v\nguarded is {k: A, v: check(xs[0])}\nguarded.v",
        "\"A\"",
    );
    assert_eq!(
        failure(&mut runtime, "A = 5\nguarded"),
        "FIVE (line 1, column 1)"
    );
    assert!(!runtime.state.has_variable("A"));
    expect(&mut runtime, "[xs, guarded.k]", "[[\"A\",\"B\"],\"A\"]");
}

#[test]
fn item_words_apply_in_every_statement_position_of_an_is_assignment() {
    for (spelled, symbol) in [
        ("a is [A, B]", "a = [A, B]"),
        ("x = 0; a is [A, B]", "x = 0; a = [A, B]"),
        ("x = 0 a is [A, B]", "x = 0 a = [A, B]"),
        ("if d:\n    a is [A, B]", "if d:\n    a = [A, B]"),
        ("if d: a is {k: A, j: B}", "if d: a = {k: A, j: B}"),
        ("if d\n    a is [A, B]", "if d\n    a = [A, B]"),
        (
            "if d:\n    pass\nelif e:\n    a is [A, B]\nelse:\n    a is [A, B]",
            "if d:\n    pass\nelif e:\n    a = [A, B]\nelse:\n    a = [A, B]",
        ),
        ("{ a is [A, B] }", "{ a = [A, B] }"),
        ("def f():\n    a is [A, B]", "def f():\n    a = [A, B]"),
        ("def f(): a is [A, B]", "def f(): a = [A, B]"),
        (
            "function f() { a is [A, B] }",
            "function f() { a = [A, B] }",
        ),
        ("f = () => { a is [A, B] }", "f = () => { a = [A, B] }"),
        (
            "class C(p):\n    this.p is [A, B]",
            "class C(p):\n    this.p = [A, B]",
        ),
        (
            "class C: this.p is Light(color=A, size=B)",
            "class C: this.p = Light(color=A, size=B)",
        ),
        (
            "class C:\n    def init(p):\n        self.p is [A, B]",
            "class C:\n    def init(p):\n        self.p = [A, B]",
        ),
        (
            "try:\n    a is [A, B]\ncatch e:\n    b = e",
            "try:\n    a = [A, B]\ncatch e:\n    b = e",
        ),
        (
            "for x in xs:\n    a is [A, B]",
            "for x in xs:\n    a = [A, B]",
        ),
        ("while d:\n    a is [A, B]", "while d:\n    a = [A, B]"),
        ("suppose:\n    a is [A, B]", "suppose:\n    a = [A, B]"),
        (
            "suppose d > 1:\n    a is [A, B]",
            "suppose d > 1:\n    a = [A, B]",
        ),
        (
            "x = {k: function () { a is [A, B] }}",
            "x = {k: function () { a = [A, B] }}",
        ),
        (
            "t = `${(() => { a is [A, B] })()}`",
            "t = `${(() => { a = [A, B] })()}`",
        ),
        ("a is x => { b is [A, B] }", "a = x => { b = [A, B] }"),
    ] {
        let tree = tree(spelled);
        let statements =
            spelled.matches(" is ").count() - usize::from(spelled.starts_with("a is x"));
        assert_eq!(
            words_in(&tree),
            ["A", "B"].repeat(statements),
            "{spelled}: {tree}"
        );
        assert_eq!(
            unmarked(&tree),
            format!("{:?}", parse(symbol).unwrap()),
            "{spelled}"
        );
    }
    let Expr::Function(function) = parse_expression("() => { a is [A, B] }").unwrap() else {
        panic!("a function literal");
    };
    let FunctionBody::Block(body) = &function.body else {
        panic!("a block body");
    };
    assert_eq!(*body, statements("a is [A, B]"));
}

#[test]
fn every_statement_position_runs_item_words() {
    fresh(
        "c = true\nif true: one is [ONE]\nif false:\n    two is [0]\nelse:\n    two is [TWO]\n{ three is {k: THREE} }\ntry:\n    throw \"four\"\ncatch issue:\n    four is [FOUR]\nn = 0\nwhile n < 1:\n    n is n + 1\n    five is [FIVE]\ndef six():\n    local is [SIX]\n    return local\nf = () => { seven is [SEVEN] }\nf()\nr = `${[8].map(v => { eight is [EIGHT]; return eight })}`\nclass K:\n    this.p is [TEN]\nsuppose c = false:\n    eleven is [ELEVEN]\n    assert(eleven, [\"ELEVEN\"])\nassert([one, two, three.k, four, five, six(), seven, eight, K().p], [[\"ONE\"], [\"TWO\"], \"THREE\", [\"FOUR\"], [\"FIVE\"], [\"SIX\"], [\"SEVEN\"], [\"EIGHT\"], [\"TEN\"]])\nr",
        "\"EIGHT\"",
    );
}

#[test]
fn malformed_values_report_the_diagnostics_of_equals() {
    for value in [
        "[A, ",
        "[A B]",
        "[A,, B]",
        "{k: A",
        "{k: }",
        "{k: A,, j: B}",
        "Light(color=A",
        "Light(color=)",
        "Light(color=A,,)",
        "[A] [",
        "{k: A}.",
        "Light(color=A)(",
        "[(A]",
        "[A if c]",
        "Light(color=A if c)",
        "[A] = ",
        "[A for x in]",
    ] {
        assert_diagnostics_of_equals(&format!("a is {value}"));
    }
    assert_eq!(
        syntax_error("a is [A, "),
        ("Unexpected end of input".to_string(), Position::new(1, 10))
    );
    assert_eq!(
        syntax_error("a is Light(color=A"),
        (
            "Expected ')' but found end of input".to_string(),
            Position::new(1, 19)
        )
    );
    let mut runtime = Runtime::new();
    for (source, missing) in [
        ("a is [(A)]", "A"),
        ("a is [A.x]", "A"),
        ("a is [() => A][0]()", "A"),
        ("a is [A for x in [1]]", "A"),
        ("a is pick(color=(A))", "A"),
    ] {
        assert_eq!(
            failure(&mut runtime, &format!("def pick(color) => color\n{source}")),
            format!("ReferenceError: {missing} is not defined (line 2, column 1)"),
            "{source}"
        );
    }
    assert!(!runtime.state.has_variable("a"));
}

fn nested(levels: usize, open: &str, close: &str, assign: &str) -> String {
    format!(
        "x {assign} {}B{}",
        open.repeat(levels),
        close.repeat(levels)
    )
}

#[test]
fn nested_items_stop_where_the_values_of_equals_stop_and_read_back_at_their_deepest() {
    for (open, close) in [
        ("[", "]"),
        ("{a: ", "}"),
        ("Light(color=", ")"),
        ("[{a: ", "}]"),
    ] {
        let limit = (1..128)
            .find(|&levels| Runtime::check(&nested(levels, open, close, "=")).is_err())
            .expect("nested items stop at a parser limit");
        for levels in 1..limit {
            let source = nested(levels, open, close, "is");
            assert!(Runtime::check(&source).is_ok(), "{source}");
        }
        assert_diagnostics_of_equals(&nested(limit, open, close, "is"));
        assert_eq!(
            syntax_error(&nested(limit, open, close, "is")).0,
            "Expressions are nested too deeply"
        );
        let deepest = nested(limit - 1, open, close, "is");
        let parsed = statements(&deepest);
        assert_eq!(words(&deepest), ["B"], "{deepest}");
        assert_eq!(statements(&generate_all(&parsed)), parsed, "{deepest}");
    }
    let lists = nested(40, "[", "]", "is");
    let mut runtime = Runtime::new();
    run_ok(&mut runtime, &lists);
    assert_eq!(
        json(&mut runtime, "x"),
        format!("{}\"B\"{}", "[".repeat(40), "]".repeat(40))
    );
}

#[test]
fn nuc86_comparisons_and_conditional_words_work_beside_item_words() {
    fresh(
        "value is EXACT\nlabel is [MATCH if value is EXACT else MISS, OTHER]\nfound is [FOUND] if [EXACT, B].includes(value) else [NONE]\npaired is {hit: value is EXACT, word: EXACT}\nassert([label, found, paired], [[\"MATCH\", \"OTHER\"], [\"FOUND\"], {\"hit\": true, \"word\": \"EXACT\"}])\nEXACT = \"B\"\nassert([value, label, found, paired], [\"B\", [\"MATCH\", \"OTHER\"], [\"FOUND\"], {\"hit\": true, \"word\": \"B\"}])\nlabel",
        "[\"MATCH\",\"OTHER\"]",
    );
}

#[test]
fn explanations_and_graph_keys_show_item_words() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "c = true\nxs is [A, (c)]\nn is [1, 2]\no = {x: 1}\no.x is {k: A}\nclass Order:\n    pass\n$Order.tags is [OPEN, c]\nfirst = Order()\n{ total is [A, (c)] }\nxs",
        "[\"A\",true]",
    );
    let why = json(&mut runtime, "xs |> why");
    assert!(why.contains("\"rule\":\"xs is [A,(c)]\""), "{why}");
    assert!(why.contains("\"from\":[\"A\",\"c\"]"), "{why}");
    let why = json(&mut runtime, "n |> why");
    assert!(why.contains("\"rule\":\"n = [1,2]\""), "{why}");
    let why = json(&mut runtime, "o.x |> why");
    assert!(why.contains(".x is {\\\"k\\\":A}\""), "{why}");
    let why = json(&mut runtime, "first.tags |> why");
    assert!(
        why.contains("\"rule\":\"$Order.tags is [OPEN,c]\""),
        "{why}"
    );
    assert_eq!(
        kind(&runtime, "block(total is [A,(c)])"),
        Some(NodeKind::Block)
    );
    assert_eq!(json(&mut runtime, "A |> affects"), "[]");
    run_ok(&mut runtime, "A = 1");
    let affected = json(&mut runtime, "A |> affects");
    assert!(affected.contains("\"node\":\"xs\""), "{affected}");
    assert!(
        affected.contains("\"rule\":\"xs is [A,(c)]\""),
        "{affected}"
    );
}

#[test]
fn item_words_need_no_option_and_reach_every_parse_entry_point() {
    for mut runtime in [Runtime::new(), Runtime::default()] {
        assert_eq!(
            run_ok(&mut runtime, "xs is [A, B]\nxs[1]"),
            Value::string("B")
        );
        runtime.clear();
        assert_eq!(
            run_ok(&mut runtime, "m is {k: YES}\nm.k"),
            Value::string("YES")
        );
    }
    assert!(Runtime::check("a is [A, B]").is_ok());
    assert!(Runtime::check("a = [A, B]").is_ok());
    let program = parse_program("x = 1\n  \na is [A, B]\nb is {k: C}").unwrap();
    assert_eq!(program.statements.len(), 3);
    assert_eq!(
        program.positions,
        [
            Position::new(1, 1),
            Position::new(3, 1),
            Position::new(4, 1)
        ]
    );
    assert_eq!(
        nucleoid::statement::compile("a is [A, B]")
            .unwrap()
            .statements,
        statements("a is [A, B]")
    );
}

#[test]
fn item_words_combine_with_c_style_comments() {
    fresh(
        "c = true // a flag\nxs is [A /* first */, B] // items\nm is {k: /* value */ C}\nx is [6 / 2]\nassert([xs, m, x], [[\"A\", \"B\"], {\"k\": \"C\"}, [3]])\nxs",
        "[\"A\",\"B\"]",
    );
    assert_eq!(
        statements("xs is [A /* x */, /* y */ B] // z"),
        statements("xs is [A, B]")
    );
    assert_eq!(
        failure(&mut Runtime::new(), "c = true\nxs = [A, B] // items"),
        "ReferenceError: A is not defined (line 2, column 1)"
    );
}

#[test]
fn the_documented_examples_run_as_written() {
    let lights = "class Light(color, size, x, parts):\n    this.color is color\n    this.size is size\n    this.x is x\n    this.parts is parts\n";
    fresh(
        &format!(
            "{lights}$Light.partner is [['RED', 'GREEN'], ['BLUE', 'AMBER']].find(pair => pair.includes($Light.color@first)).find(other => other is not $Light.color@first)\n$Light.glow is {{small: 'DIM', large: 'BRIGHT'}}[$Light.size@first]\n$Light.shade is ['RED', 'GREEN', 'BLUE'][$Light.x@first]\nfirst is Light(color='RED', size='small', x=2, parts=['GREEN', 'GREEN'])\n[first.partner, first.glow, first.shade, first.parts]"
        ),
        "[\"GREEN\",\"DIM\",\"BLUE\",[\"GREEN\",\"GREEN\"]]",
    );
    let rules = "$Light.partner is [[RED, GREEN], [BLUE, AMBER]].find(pair => pair.includes($Light.color@first)).find(other => other is not $Light.color@first)\n$Light.glow is {small: DIM, large: BRIGHT}[$Light.size@first]\n$Light.shade is [RED, GREEN, BLUE][$Light.x@first]\nfirst is Light(color=RED, size=small, x=2, parts=[GREEN, GREEN])\n";
    fresh(
        &format!(
            "{lights}{rules}assert([first.partner, first.glow, first.shade, first.parts], [\"GREEN\", \"DIM\", \"BLUE\", [\"GREEN\", \"GREEN\"]])\nDIM = \"dim\"\nGREEN = \"green\"\nassert([first.partner, first.glow, first.shade, first.parts], [\"green\", \"dim\", \"BLUE\", [\"GREEN\", \"GREEN\"]])\nfirst.glow"
        ),
        "\"dim\"",
    );
    assert_eq!(
        failure(
            &mut Runtime::new(),
            &format!("{lights}{}", rules.replace(" is ", " = "))
        ),
        "ReferenceError: RED is not defined (line 6, column 1)"
    );
    fresh(
        "colors is [RED, GREEN]\npair is {left: RED, right: BLUE}\nassert([colors, pair], [[\"RED\", \"GREEN\"], {\"left\": \"RED\", \"right\": \"BLUE\"}])\nRED = \"red\"\nassert([colors[0], pair.left], [\"red\", \"red\"])\ncolors[0]",
        "\"red\"",
    );
    fresh(
        "class Light(size, x):\n    this.size is size\n    this.x is x\n    this.corners is [[RED, BLUE], [GREEN, AMBER]]\n$Light.glow is {small: BLUE, large: GREEN}[$Light.size@first]\n$Light.shade is [RED, BLUE, GREEN][$Light.x@first]\nfirst = Light(\"small\", 0)\nsecond is Light(size=large, x=2)\nassert([first.glow, second.glow], [\"BLUE\", \"GREEN\"])\nassert([first.shade, second.shade], [\"RED\", \"GREEN\"])\nassert(first.corners, [[\"RED\", \"BLUE\"], [\"GREEN\", \"AMBER\"]])\nGREEN = \"green\"\nassert([second.glow, second.shade], [\"green\", \"green\"])\nsecond.glow",
        "\"green\"",
    );
    fresh(
        "c = true\nxs is [A, (c)]\nassert(xs, [\"A\", true])\nxs",
        "[\"A\",true]",
    );
    fresh(
        "c = true\nlabel is X if [A, B].includes(c) else Y\npicked is [A if c else B]\nassert([label, picked], [\"Y\", [\"A\"]])\nlabel",
        "\"Y\"",
    );
    let mut runtime = Runtime::new();
    runtime.run("colors is [RED, GREEN]").unwrap();
    assert_eq!(runtime.run("colors[1]").unwrap().to_string(), "GREEN");
    runtime.run("GREEN = \"green\"").unwrap();
    assert_eq!(runtime.run("colors[1]").unwrap().to_string(), "green");
}
