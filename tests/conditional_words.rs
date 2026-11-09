mod common;

use nucleoid::graph::{NodeKey, NodeKind};
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{
    BinaryOp, Expr, FunctionBody, Stmt, UnaryOp, parse, parse_expression, parse_program,
};
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn word(name: &str) -> Expr {
    Expr::Unary {
        operator: UnaryOp::Word,
        operand: Box::new(Expr::Identifier(name.to_string())),
    }
}

fn conditional(condition: Expr, consequent: Expr, alternate: Expr) -> Expr {
    Expr::Conditional {
        condition: Box::new(condition),
        consequent: Box::new(consequent),
        alternate: Box::new(alternate),
    }
}

fn assign(target: Expr, value: Expr) -> Stmt {
    Stmt::Assign { target, value }
}

fn member(object: Expr, property: &str) -> Expr {
    Expr::Member {
        object: Box::new(object),
        property: property.to_string(),
    }
}

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn words(source: &str) -> usize {
    format!("{:?}", statements(source))
        .matches("operator: Word")
        .count()
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn equals_spelling(source: &str) -> Option<String> {
    source
        .strip_prefix("a is ")
        .map(|value| format!("a = {value}"))
}

fn assert_diagnostics_of_equals(source: &str) {
    let equals =
        equals_spelling(source).unwrap_or_else(|| panic!("{source} is not an is assignment to a"));
    let (message, position) = syntax_error(source);
    let (expected, at) = syntax_error(&equals);
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
        title: "conditional word regression".to_string(),
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

#[test]
fn branches_of_one_identifier_token_in_result_positions_are_words() {
    let both = || conditional(name("c"), word("A"), word("B"));
    let chain = || {
        conditional(
            name("c1"),
            word("A"),
            conditional(name("c2"), word("B"), word("C")),
        )
    };
    for (source, expected) in [
        ("a is A if c else B", vec![assign(name("a"), both())]),
        ("a is c ? A : B", vec![assign(name("a"), both())]),
        (
            "o.x is A if c else B",
            vec![assign(member(name("o"), "x"), both())],
        ),
        (
            "this.p is c ? A : B",
            vec![assign(member(Expr::This, "p"), both())],
        ),
        (
            "$Order.p is A if c else B",
            vec![assign(
                member(Expr::ClassRef("Order".to_string()), "p"),
                both(),
            )],
        ),
        ("(a) is A if c else B", vec![assign(name("a"), both())]),
        (
            "a is A if c else 1",
            vec![assign(
                name("a"),
                conditional(name("c"), word("A"), Expr::Number(1.0)),
            )],
        ),
        (
            "a is c ? 1 : B",
            vec![assign(
                name("a"),
                conditional(name("c"), Expr::Number(1.0), word("B")),
            )],
        ),
        (
            "a is A if c1 else B if c2 else C",
            vec![assign(name("a"), chain())],
        ),
        ("a is c1 ? A : c2 ? B : C", vec![assign(name("a"), chain())]),
        (
            "a is A if c1 else c2 ? B : C",
            vec![assign(name("a"), chain())],
        ),
        (
            "a is c1 ? A : B if c2 else C",
            vec![assign(name("a"), chain())],
        ),
        (
            "a is c1 ? c2 ? A : B : C",
            vec![assign(
                name("a"),
                conditional(
                    name("c1"),
                    conditional(name("c2"), word("A"), word("B")),
                    word("C"),
                ),
            )],
        ),
        (
            "a is is if c else suppose",
            vec![assign(
                name("a"),
                conditional(name("c"), word("is"), word("suppose")),
            )],
        ),
        (
            "a is model if c else why",
            vec![assign(
                name("a"),
                conditional(name("c"), word("model"), word("why")),
            )],
        ),
        (
            "a is A if v is X else B",
            vec![assign(
                name("a"),
                conditional(
                    Expr::Binary {
                        operator: BinaryOp::Is,
                        left: Box::new(name("v")),
                        right: Box::new(name("X")),
                    },
                    word("A"),
                    word("B"),
                ),
            )],
        ),
        (
            "a is (A) if c else B",
            vec![assign(
                name("a"),
                conditional(name("c"), name("A"), word("B")),
            )],
        ),
        (
            "a is A if c else (B)",
            vec![assign(
                name("a"),
                conditional(name("c"), word("A"), name("B")),
            )],
        ),
        (
            "a is c1 ? (c2 ? A : B) : C",
            vec![assign(
                name("a"),
                conditional(
                    name("c1"),
                    conditional(name("c2"), name("A"), name("B")),
                    word("C"),
                ),
            )],
        ),
        (
            "a is c ? b = A : B",
            vec![assign(
                name("a"),
                conditional(
                    name("c"),
                    Expr::Assign {
                        target: Box::new(name("b")),
                        value: Box::new(name("A")),
                    },
                    word("B"),
                ),
            )],
        ),
        (
            "a is c ? c2 ? A : B = 1 : C",
            vec![assign(
                name("a"),
                conditional(
                    name("c"),
                    Expr::Assign {
                        target: Box::new(conditional(name("c2"), name("A"), name("B"))),
                        value: Box::new(Expr::Number(1.0)),
                    },
                    word("C"),
                ),
            )],
        ),
        (
            "a is A if c else B b is 2",
            vec![
                assign(name("a"), both()),
                assign(name("b"), Expr::Number(2.0)),
            ],
        ),
    ] {
        assert_eq!(statements(source), expected, "{source}");
    }
    assert_ne!(
        statements("a is A if c else B"),
        statements("a = A if c else B")
    );
    assert_eq!(
        statements("a is (A if c else B)"),
        statements("a = A if c else B")
    );
}

#[test]
fn conditions_parentheses_and_other_operands_hold_no_word_branches() {
    for source in [
        "a is (A if c else B)",
        "a is (c ? A : B)",
        "a is ((A if c else B))",
        "a is (A) if c else (B)",
        "a is 1 if A else 2",
        "a is A ? 1 : 2",
        "a is A.x if c else B.y",
        "a is A() if c else B[0]",
        "a is f(A) if c else g(B)",
        "a is f(A if c else B)",
        "a is [(A if c else B)]",
        "a is {k: (A if c else B)}",
        "a is not A if c else not B",
        "a is -A if c else +B",
        "a is !A if c else typeof B",
        "a is A + 1 if c else B * 2",
        "a is [(A)] if c else {k: (B)}",
        "a is A ?? B if c else 1",
        "a is 1 if c else B ?? C",
        "a is A@first if c else B@last",
        "a is new A if c else super",
        "a is true if c else null",
        "a is this if c else self",
        "a is $Order if c else None",
        "a is 'A' if c else \"B\"",
        "a is 1 if c else 2",
        "a is c ? 1 : 2",
        "a is b = A if c else B",
        "a is A if c else B = 5",
        "a is c ? A : B = 5",
        "a is x => A if c else B",
        "a is lambda x: A if c else B",
        "a is `${c ? A : B}`",
        "a is `${(A if c else B)}`",
        "a is all x in xs: A if c else B",
        "a is (A)",
        "a = A if c else B",
        "a := A if c else B",
        "a += A if c else B",
        "A if c else B",
        "c ? A : B",
        "x = 1 if c else 2 is 3",
        "if c:\n    a = A if d else B",
        "def f() => A if c else B",
        "return A if c else B",
        "throw A if c else B",
        "assert: A if c else B, A",
        "{a, b} = A if c else B",
        "a, b = A if c else B",
        "suppose A if c else B:\n    pass",
        "while A if c else B:\n    pass",
    ] {
        assert_eq!(words(source), 0, "{source}");
        if let Some(equals) = equals_spelling(source) {
            assert_eq!(statements(source), statements(&equals), "{source}");
        }
    }
    assert_eq!(words("a is A"), 1);
    assert_eq!(
        parse_expression("A if c else B").unwrap(),
        conditional(name("c"), name("A"), name("B"))
    );
}

#[test]
fn every_is_assignment_value_without_word_branches_builds_the_tree_of_its_equals_spelling() {
    let targets = ["a", "o.x", "this.p", "$Order.p", "(a)", "a@first"];
    let values = [
        "5",
        "\"good\"",
        "(good)",
        "good.x",
        "good()",
        "good[0]",
        "good + 1",
        "not good",
        "[1, (good)]",
        "{k: (good)}",
        "x => x * 2",
        "lambda x: x + 1",
        "Order(total=1)",
        "c ? 1 : 2",
        "1 if c else 2",
        "(c ? x : y)",
        "f(c ? x : y)",
        "good ?? 1",
        "good |> why",
        "[x * 2 for x in xs]",
        "b = c",
        "b = c ? x : y",
        "b += 1",
        "all x in xs: x > 0",
        "sum of xs",
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
fn word_branches_render_with_is_and_read_back_as_the_same_statement() {
    for (source, rendered) in [
        ("a is A if c else B", "a is c?A:B"),
        ("a is c ? A : 1", "a is c?A:1"),
        ("a is A if c else (B)", "a is c?A:(B)"),
        ("a is A if c1 else B if c2 else C", "a is c1?A:c2?B:C"),
        ("a is c1 ? c2 ? A : B : C", "a is c1?c2?A:B:C"),
        ("a is c1 ? (c2 ? A : B) : C", "a is c1?(c2?A:B):C"),
        ("a is c1 ? (c2 ? 1 : 2) : C", "a is c1?(c2?1:2):C"),
        ("a is A if v is X else B", "a is (v is X)?A:B"),
        ("a is A if n > 1 else B", "a is n>1?A:B"),
        ("o.x is A if c else B", "o.x is c?A:B"),
        (
            "$Order.state is OPEN if $Order.total > 0 else EMPTY",
            "$Order.state is $Order.total>0?OPEN:EMPTY",
        ),
        ("a is c ? b = A : B", "a is c?b=A:B"),
        ("a is (A if c else B)", "a=(c?A:B)"),
        ("a is 1 if c else 2", "a=(c?1:2)"),
        ("a is good", "a is good"),
        ("a is (good)", "a=good"),
        ("{ a is A if c else B }", "{a is c?A:B}"),
        ("if c:\n    a is A if d else B", "if(c){a is d?A:B}"),
        (
            "a is suppose:\n    b is B if c else C",
            "a=suppose{b is c?B:C}",
        ),
    ] {
        let parsed = statements(source);
        assert_eq!(generate_all(&parsed), rendered, "{source}");
        if !rendered.starts_with("if(") && !rendered.contains("suppose") {
            assert_eq!(statements(rendered), parsed, "{rendered}");
        }
    }
    assert_ne!(
        generate_all(&statements("{ a is A if c else B }")),
        generate_all(&statements("{ a = A if c else B }"))
    );
    assert_eq!(
        generate_all(&statements("{ a is 1 if c else 2 }")),
        generate_all(&statements("{ a = 1 if c else 2 }"))
    );
}

#[test]
fn a_word_branch_reads_its_variable_once_defined_and_its_text_otherwise() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "c = true\na is A if c else B\nb is c ? A : B\nq is 'A' if c else \"B\"\nassert([a, b, q], [\"A\", \"A\", \"A\"])\na",
        "\"A\"",
    );
    let pending = runtime
        .graph
        .retrieve(&NodeKey::variable("A"))
        .expect("the selected word is a pending node");
    assert_eq!(pending.kind, NodeKind::Pending);
    assert!(pending.dependents.contains(&NodeKey::variable("a")));
    assert!(pending.dependents.contains(&NodeKey::variable("b")));
    assert!(!pending.dependents.contains(&NodeKey::variable("q")));
    assert!(runtime.graph.retrieve(&NodeKey::variable("B")).is_none());
    expect(
        &mut runtime,
        "A = 1\nassert([a, b, q], [1, 1, \"A\"])\na",
        "1",
    );
    expect(&mut runtime, "A = [2]\na", "[2]");
    expect(&mut runtime, "delete A\n[a, b]", "[\"A\",\"A\"]");
    expect(&mut runtime, "c = false\n[a, b, q]", "[\"B\",\"B\",\"B\"]");
    assert!(
        runtime
            .graph
            .retrieve(&NodeKey::variable("B"))
            .is_some_and(|node| node.kind == NodeKind::Pending)
    );
    expect(&mut runtime, "B = null\na", "null");
    expect(&mut runtime, "B = 3\nassert([a, b], [3, 3])\nb", "3");
    expect(&mut runtime, "delete B\na", "\"B\"");
    expect(&mut runtime, "c = true\na", "\"A\"");
    fresh(
        "A = 2\nc = true\na is A if c else B\nassert(a, 2)\nA = 5\nassert(a, 5)\na",
        "5",
    );
    fresh(
        "c1 = false\nc2 = false\na is c1 ? A : c2 ? B : C\nassert(a, \"C\")\nc2 = true\nassert(a, \"B\")\nB = 4\na",
        "4",
    );
}

#[test]
fn constructors_and_class_level_rules_take_word_branches_per_instance() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "class Order(total):\n    this.total = total\n    this.size is BIG if this.total > 10 else SMALL\n$Order.state is OPEN if $Order.total > 0 else EMPTY\n$Order.flag is $Order.total > 0 ? YES : NO\nfirst = Order(0)\nsecond = Order(20)\nassert([first.size, second.size], [\"SMALL\", \"BIG\"])\nassert([first.state, second.state], [\"EMPTY\", \"OPEN\"])\nassert([first.flag, second.flag], [\"NO\", \"YES\"])\nsecond.state",
        "\"OPEN\"",
    );
    expect(
        &mut runtime,
        "OPEN = \"open\"\nBIG = 1\nassert([first.state, second.state, first.size, second.size], [\"EMPTY\", \"open\", \"SMALL\", 1])\nsecond.state",
        "\"open\"",
    );
    expect(
        &mut runtime,
        "delete OPEN\n[first.state, second.state]",
        "[\"EMPTY\",\"OPEN\"]",
    );
    assert_eq!(
        failure(
            &mut runtime,
            "$Order.bad is (MISSING) if $Order.total > 0 else OTHER"
        ),
        "ReferenceError: MISSING is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure(
            &mut runtime,
            "$Order.bad is MISSING.x if $Order.total > 0 else OTHER"
        ),
        "ReferenceError: MISSING is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure(
            &mut runtime,
            "$Order.bad = MISSING if $Order.total > 0 else OTHER"
        ),
        "ReferenceError: MISSING is not defined (line 1, column 1)"
    );
    expect(
        &mut runtime,
        "$Order.ok is MISSING if $Order.total > 0 else OTHER\n[first.ok, second.ok]",
        "[\"OTHER\",\"MISSING\"]",
    );
}

#[test]
fn words_that_name_a_local_class_function_or_global_read_them() {
    fresh(
        "def pick(flag, EXACT):\n    found is EXACT if flag else OTHER\n    return found\nassert(pick(true, 5), 5)\npick(false, 5)",
        "\"OTHER\"",
    );
    fresh(
        "class Order:\n    pass\ndef helper() => 7\nk is Order if true else helper\nh is Order if false else helper\nm is true ? Math : Order\nassert(k == Order, true)\nassert(h(), 7)\nassert(m.PI, Math.PI)\ntypeof h",
        "\"Function\"",
    );
    fresh(
        "h is helper if true else other\nassert(h, \"helper\")\ndef helper() => 7\nassert(h(), 7)\nh()",
        "7",
    );
}

#[test]
fn hypotheticals_and_rollback_restore_word_branches() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "c = true\na is A if c else B\nsuppose A = 9:\n    assert(a, 9)\nsuppose c = false:\n    assert(a, \"B\")\nassert(a, \"A\")\na",
        "\"A\"",
    );
    expect(
        &mut runtime,
        "result is suppose A = 2:\n    a * 10\nassert(result, 20)\nresult",
        "20",
    );
    assert!(!runtime.state.has_variable("A"));
    expect(
        &mut runtime,
        "def check(v):\n    if v == 5:\n        throw \"FIVE\"\n    return v\nguarded is check(a) if c else NONE\nguarded",
        "\"A\"",
    );
    assert_eq!(
        failure(&mut runtime, "A = 5\nguarded"),
        "FIVE (line 1, column 1)"
    );
    assert!(!runtime.state.has_variable("A"));
    expect(&mut runtime, "[a, guarded]", "[\"A\",\"A\"]");
}

fn without_word_markers(tree: &str) -> String {
    tree.replace(
        "Unary { operator: Word, operand: Identifier(\"A\") }",
        "Identifier(\"A\")",
    )
    .replace(
        "Unary { operator: Word, operand: Identifier(\"B\") }",
        "Identifier(\"B\")",
    )
}

#[test]
fn word_branches_apply_in_every_statement_position_of_an_is_assignment() {
    for (spelled, symbol) in [
        ("a is A if c else B", "a = A if c else B"),
        ("x = 0; a is A if c else B", "x = 0; a = A if c else B"),
        ("x = 0 a is c ? A : B", "x = 0 a = c ? A : B"),
        (
            "if d:\n    a is A if c else B",
            "if d:\n    a = A if c else B",
        ),
        ("if d: a is A if c else B", "if d: a = A if c else B"),
        (
            "if d\n    a is A if c else B",
            "if d\n    a = A if c else B",
        ),
        (
            "if d:\n    pass\nelif e:\n    a is A if c else B\nelse:\n    a is c ? A : B",
            "if d:\n    pass\nelif e:\n    a = A if c else B\nelse:\n    a = c ? A : B",
        ),
        ("{ a is A if c else B }", "{ a = A if c else B }"),
        (
            "def f():\n    a is A if c else B",
            "def f():\n    a = A if c else B",
        ),
        ("def f(): a is A if c else B", "def f(): a = A if c else B"),
        (
            "function f() { a is A if c else B }",
            "function f() { a = A if c else B }",
        ),
        (
            "f = () => { a is A if c else B }",
            "f = () => { a = A if c else B }",
        ),
        (
            "class C(p):\n    this.p is A if p else B",
            "class C(p):\n    this.p = A if p else B",
        ),
        (
            "class C: this.p is A if c else B",
            "class C: this.p = A if c else B",
        ),
        (
            "class C:\n    def init(p):\n        self.p is A if p else B",
            "class C:\n    def init(p):\n        self.p = A if p else B",
        ),
        (
            "try:\n    a is A if c else B\ncatch e:\n    b = e",
            "try:\n    a = A if c else B\ncatch e:\n    b = e",
        ),
        (
            "for x in xs:\n    a is A if x else B",
            "for x in xs:\n    a = A if x else B",
        ),
        (
            "while d:\n    a is A if c else B",
            "while d:\n    a = A if c else B",
        ),
        (
            "suppose:\n    a is A if c else B",
            "suppose:\n    a = A if c else B",
        ),
        (
            "suppose d > 1:\n    a is A if c else B",
            "suppose d > 1:\n    a = A if c else B",
        ),
        (
            "x = {k: function () { a is A if c else B }}",
            "x = {k: function () { a = A if c else B }}",
        ),
        (
            "t = `${(() => { a is A if c else B })()}`",
            "t = `${(() => { a = A if c else B })()}`",
        ),
    ] {
        let tree = format!("{:?}", statements(spelled));
        assert_eq!(
            tree.matches("operator: Word").count(),
            2 * spelled.matches(" is ").count(),
            "{spelled}: {tree}"
        );
        assert_eq!(
            without_word_markers(&tree),
            format!("{:?}", statements(symbol)),
            "{spelled}"
        );
    }
}

#[test]
fn every_statement_position_takes_word_branches() {
    fresh(
        "c = true\nif true: one is ONE if c else NO\nif false:\n    two is 0\nelse:\n    two is TWO if c else NO\n{ three is THREE if c else NO }\ntry:\n    throw \"four\"\ncatch issue:\n    four is FOUR if issue is \"four\" else NO\nn = 0\nwhile n < 1:\n    n is n + 1\n    five is FIVE if n > 0 else NO\ndef six():\n    local is SIX if c else NO\n    return local\nf = () => { seven is SEVEN if c else NO }\nf()\nr = `${[8].map(v => { eight is c ? EIGHT : NO; return eight })}`\nclass K:\n    this.p is TEN if c else NO\nsuppose c = false:\n    eleven is ELEVEN if c else NO\n    assert(eleven, \"NO\")\nassert([one, two, three, four, five, six(), seven, eight, K().p], [\"ONE\", \"TWO\", \"THREE\", \"FOUR\", \"FIVE\", \"SIX\", \"SEVEN\", \"EIGHT\", \"TEN\"])\nr",
        "\"EIGHT\"",
    );
    let Expr::Function(function) = parse_expression("() => { a is A if c else B }").unwrap() else {
        panic!("a function literal");
    };
    let FunctionBody::Block(body) = &function.body else {
        panic!("a block body");
    };
    assert_eq!(*body, statements("a is A if c else B"));
}

#[test]
fn malformed_values_report_the_diagnostics_of_equals() {
    for value in [
        "A if c else",
        "A if c else\n",
        "c ? A",
        "c ? A :",
        "A if c else B +",
        "c ? A : B :",
        "(A if c else B",
        "A if c else (B",
        "c ? A : (1 + )",
        "A if c else B := 1",
        "c ? (A : B",
        "c ? A : B if",
        "A if c else B.",
        "A if c else B@",
        "c ? ? A : B",
    ] {
        assert_diagnostics_of_equals(&format!("a is {value}"));
    }
    assert_eq!(
        syntax_error("a is A if c else"),
        ("Unexpected end of input".to_string(), Position::new(1, 17))
    );
    assert_eq!(
        syntax_error("a is c ? A"),
        (
            "Expected ':' but found end of input".to_string(),
            Position::new(1, 11)
        )
    );
    assert_eq!(
        syntax_error("a is c ? A : (1 + )"),
        ("Unexpected ')'".to_string(), Position::new(1, 20))
    );
    let mut runtime = Runtime::new();
    assert_eq!(
        failure(&mut runtime, "c = true\na is (A) if c else B"),
        "ReferenceError: A is not defined (line 2, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "a is A if missing else B"),
        "ReferenceError: missing is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "c = true\na is A.x if c else B"),
        "ReferenceError: A is not defined (line 2, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "c = true\na is X if c else Y = 5"),
        "SyntaxError: Cannot assign to (c?X:Y) (line 2, column 1)"
    );
    assert!(!runtime.state.has_variable("c"));
}

fn python_chain(levels: usize) -> (String, String) {
    let words: Vec<String> = (0..=levels).map(|level| format!("A{level}")).collect();
    (
        format!("a is {}", words.join(" if c else ")),
        format!("A{levels}"),
    )
}

fn alternate_chain(levels: usize) -> (String, String) {
    let heads: String = (0..levels)
        .map(|level| format!("c ? A{level} : "))
        .collect();
    (format!("a is {heads}A{levels}"), format!("A{levels}"))
}

fn consequent_chain(levels: usize) -> (String, String) {
    let tails: String = (0..levels).map(|level| format!(" : B{level}")).collect();
    (
        format!("a is {}A{tails}", "c ? ".repeat(levels)),
        format!("B{}", levels - 1),
    )
}

#[test]
fn word_chains_stop_where_the_chains_of_equals_stop_and_read_back_at_their_deepest() {
    let chains: [fn(usize) -> (String, String); 3] =
        [python_chain, alternate_chain, consequent_chain];
    for chain in chains {
        let limit = (1..128)
            .find(|&levels| Runtime::check(&chain(levels).0).is_err())
            .expect("a chain of conditionals stops at a parser limit");
        for levels in 1..limit {
            let equals = equals_spelling(&chain(levels).0).unwrap();
            assert!(Runtime::check(&equals).is_ok(), "{equals}");
        }
        assert_diagnostics_of_equals(&chain(limit).0);
        let (source, selected) = chain(limit - 1);
        let parsed = statements(&source);
        assert_eq!(statements(&generate_all(&parsed)), parsed, "{source}");
        fresh(
            &format!("c = false\n{source}\na"),
            &format!("\"{selected}\""),
        );
    }
}

#[test]
fn nuc86_comparisons_in_conditions_work_beside_word_branches() {
    fresh(
        "value is EXACT\nlabel is MATCH if value is EXACT else MISS\nother is MATCH if value is not EXACT else MISS\nxs = [\"EXACT\", \"B\"]\nfound is FOUND if xs.some(x => x is EXACT) else NONE\nassert([label, other, found], [\"MATCH\", \"MISS\", \"FOUND\"])\nEXACT = \"B\"\nassert([value, label, other, found], [\"B\", \"MATCH\", \"MISS\", \"FOUND\"])\nlabel",
        "\"MATCH\"",
    );
}

#[test]
fn explanations_and_graph_keys_show_word_branches() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "c = true\na is A if c else B\nb is 1 if c else 2\no = {x: 1}\no.x is A if c else B\nclass Order:\n    pass\n$Order.state is OPEN if c else CLOSED\nfirst = Order()\n{ total is A if c else (B) }\na",
        "\"A\"",
    );
    let why = json(&mut runtime, "a |> why");
    assert!(why.contains("\"rule\":\"a is c?A:B\""), "{why}");
    assert!(why.contains("\"from\":[\"c\",\"A\"]"), "{why}");
    let why = json(&mut runtime, "b |> why");
    assert!(why.contains("\"rule\":\"b = (c?1:2)\""), "{why}");
    let why = json(&mut runtime, "o.x |> why");
    assert!(why.contains(".x is c?A:B\""), "{why}");
    let why = json(&mut runtime, "first.state |> why");
    assert!(
        why.contains("\"rule\":\"$Order.state is c?OPEN:CLOSED\""),
        "{why}"
    );
    assert!(
        runtime
            .graph
            .retrieve(&NodeKey::variable("block(total is c?A:(B))"))
            .is_some_and(|node| node.kind == NodeKind::Block)
    );
    assert_eq!(json(&mut runtime, "A |> affects"), "[]");
    run_ok(&mut runtime, "A = 1");
    let affected = json(&mut runtime, "A |> affects");
    assert!(affected.contains("\"node\":\"a\""), "{affected}");
    assert!(affected.contains("\"rule\":\"a is c?A:B\""), "{affected}");
}

#[test]
fn word_branches_survive_clear_and_reach_every_parse_entry_point() {
    for mut runtime in [Runtime::new(), Runtime::default()] {
        assert_eq!(
            run_ok(&mut runtime, "c = true\na is A if c else B\na"),
            Value::string("A")
        );
        runtime.clear();
        assert_eq!(
            run_ok(&mut runtime, "b is YES if true else NO\nb"),
            Value::string("YES")
        );
    }
    assert!(Runtime::check("a is A if c else B").is_ok());
    let program = parse_program("x = 1\n  \na is A if c else B\nb is 2").unwrap();
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
        nucleoid::statement::compile("a is A if c else B")
            .unwrap()
            .statements,
        statements("a is A if c else B")
    );
}

#[test]
fn word_branches_combine_with_c_style_comments() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "c = true // a flag\na is A /* first */ if c else B // second\nb is c ? /* yes */ YES : NO\nx is 6 / 2 if c else 0\nassert([a, b, x], [\"A\", \"YES\", 3])\na",
        "\"A\"",
    );
    assert_eq!(
        parse("a is A /* x */ if c else /* y */ B // z").unwrap(),
        statements("a is A if c else B")
    );
}

#[test]
fn the_documented_examples_run_as_written() {
    let tiles = "class Tile(symmetric, parts):\n    this.symmetric = symmetric\n    this.parts = parts\nfirst = Tile(true, [1, 1])\nsecond = Tile(false, [1, 2])\n";
    fresh(
        &format!(
            "{tiles}$Tile.fill is 'B' if $Tile.symmetric else 'C'\n$Tile.mark is 'A' if $Tile.parts.distinct().length is 1 else 0\n[first.fill, second.fill, first.mark, second.mark]"
        ),
        "[\"B\",\"C\",\"A\",0]",
    );
    fresh(
        &format!(
            "{tiles}$Tile.fill is B if $Tile.symmetric else C\n$Tile.mark is A if $Tile.parts.distinct().length is 1 else 0\n[first.fill, second.fill, first.mark, second.mark]"
        ),
        "[\"B\",\"C\",\"A\",0]",
    );
    fresh(
        "class Order(total):\n    this.total = total\n    this.size is BIG if this.total > 10 else SMALL\n$Order.state is OPEN if $Order.total > 0 else EMPTY\nfirst = Order(0)\nsecond = Order(20)\nassert([first.size, second.size], [\"SMALL\", \"BIG\"])\nassert([first.state, second.state], [\"EMPTY\", \"OPEN\"])\nOPEN = \"open\"\nassert(second.state, \"open\")\nsecond.state",
        "\"open\"",
    );
    fresh(
        "value is EXACT\nlabel is MATCH if value is EXACT else MISS\nother is MATCH if value is not EXACT else MISS\nassert([label, other], [\"MATCH\", \"MISS\"])\nEXACT = \"B\"\nassert([value, label, other], [\"B\", \"MATCH\", \"MISS\"])\nlabel",
        "\"MATCH\"",
    );
    fresh("c = false\na is c ? A : 1\na", "1");
    fresh(
        "shape is SQUARE\nsize is 12\nlabel is BIG if size > 10 else SMALL\nmark is CORNER if shape is SQUARE else EDGE\nassert([label, mark], [\"BIG\", \"CORNER\"])\nBIG = \"big\"\nsize = 3\nassert([label, mark], [\"SMALL\", \"CORNER\"])\nsize = 30\nassert(label, \"big\")\nlabel",
        "\"big\"",
    );
    fresh(
        "class Tile(symmetric, parts):\n    this.symmetric = symmetric\n    this.parts = parts\n$Tile.fill is B if $Tile.symmetric else C\n$Tile.mark is A if $Tile.parts.distinct().length is 1 else 0\nfirst = Tile(true, [1, 1])\nsecond = Tile(false, [1, 2])\nassert([first.fill, second.fill], [\"B\", \"C\"])\nassert([first.mark, second.mark], [\"A\", 0])\nC = \"c\"\nsecond.fill",
        "\"c\"",
    );
    fresh(
        "ready = false\nstatus is OPEN if ready else WAITING\nassert(status, \"WAITING\")\nready = true\nassert(status, \"OPEN\")\nstatus",
        "\"OPEN\"",
    );
    fresh(
        "shape is SQUARE\nmark is (shape is SQUARE) ? CORNER : EDGE\nother is CORNER if shape is ROUND else EDGE\nassert([mark, other], [\"CORNER\", \"EDGE\"])\nmark",
        "\"CORNER\"",
    );
    assert_eq!(
        failure(
            &mut Runtime::new(),
            "shape is SQUARE\nmark is shape is SQUARE ? CORNER : EDGE"
        ),
        "ReferenceError: is is not defined (line 2, column 15)"
    );
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "ready = false\nstatus is OPEN if ready else WAITING\nOPEN = \"open\"\nready = true\nstatus",
        "\"open\"",
    );
    assert_eq!(
        failure(
            &mut Runtime::new(),
            "ready = false\nlevel is (HIGH if ready else LOW)"
        ),
        "ReferenceError: LOW is not defined (line 2, column 1)"
    );
    let mut runtime = Runtime::new();
    runtime.run("ready = false").unwrap();
    runtime.run("status is OPEN if ready else WAITING").unwrap();
    assert_eq!(runtime.run("status").unwrap().to_string(), "WAITING");
    runtime.run("ready = true").unwrap();
    assert_eq!(runtime.run("status").unwrap().to_string(), "OPEN");
}
