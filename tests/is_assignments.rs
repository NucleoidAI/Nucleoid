mod common;

use nucleoid::graph::{NodeKey, NodeKind};
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{
    BinaryOp, Expr, Stmt, UnaryOp, parse, parse_expression, parse_expression_with_options,
    parse_program_with_options, parse_with_options,
};
use nucleoid::{ErrorKind, Position, Runtime, SyntaxOptions, Value};

const IS: SyntaxOptions = SyntaxOptions {
    c_style_comments: false,
    is_assignment: true,
};

const BOTH: SyntaxOptions = SyntaxOptions {
    c_style_comments: true,
    is_assignment: true,
};

const COMMENTS: SyntaxOptions = SyntaxOptions {
    c_style_comments: true,
    is_assignment: false,
};

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

fn member(object: Expr, property: &str) -> Expr {
    Expr::Member {
        object: Box::new(object),
        property: property.to_string(),
    }
}

fn statements(source: &str) -> Vec<Stmt> {
    parse_with_options(source, IS).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn accepted(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn syntax_error(source: &str, options: SyntaxOptions) -> (String, Position) {
    let error = Runtime::check_with_options(source, options).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn expect(runtime: &mut Runtime, source: &str, expected: &str) {
    let parsed = parse_with_options(source, runtime.options())
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    let assertions = common::assertions::protected_assertions(&parsed).len();
    let case = common::Case {
        title: "is assignment regression".to_string(),
        source: source.to_string(),
        expected: Some(expected.to_string()),
    };
    common::check_with_count(runtime, &case, assertions)
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn fresh(source: &str, expected: &str) {
    expect(&mut Runtime::with_options(IS), source, expected);
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

#[test]
fn every_value_but_a_bare_word_builds_the_tree_of_its_equals_spelling() {
    let targets = [
        "a", "o.x", "o.x.y", "this.p", "self.p", "$Order.p", "(a)", "(o.x)", "a@first",
    ];
    let values = [
        "5",
        "-1",
        "\"good\"",
        "'good'",
        "`text ${good}`",
        "true",
        "false",
        "null",
        "None",
        "undefined",
        "this",
        "self",
        "$Order",
        "(good)",
        "good.x",
        "good()",
        "good[0]",
        "good[1:]",
        "good@first",
        "good + 1",
        "good == 1",
        "not good",
        "!good",
        "typeof good",
        "+good",
        "[1, good]",
        "{k: good}",
        "{...good}",
        "x => x * 2",
        "(x, y) -> x + y",
        "lambda x: x + 1",
        "function (x) { return x }",
        "Order(1)",
        "Order(total=1)",
        "new Date(0)",
        "delete good",
        "super",
        "super(1)",
        "all x in xs: x > 0",
        "sum of xs",
        "c ? 1 : 2",
        "1 if c else 2",
        "good ?? 1",
        "good |> why",
        "[x * 2 for x in xs]",
        "b = c",
        "b += 1",
        "b = c = d",
        "1 + (2 * 3) ** 2",
    ];
    for target in targets {
        for value in values {
            let spelled = format!("{target} is {value}");
            assert_eq!(
                statements(&spelled),
                accepted(&format!("{target} = {value}")),
                "{spelled}"
            );
            assert_eq!(
                parse_with_options(&spelled, BOTH).unwrap(),
                statements(&spelled),
                "{spelled}"
            );
        }
    }
}

#[test]
fn a_value_of_exactly_one_identifier_token_is_a_word() {
    for (source, expected) in [
        ("a is good", vec![assign(name("a"), word("good"))]),
        (
            "o.x is good",
            vec![assign(member(name("o"), "x"), word("good"))],
        ),
        (
            "this.p is good",
            vec![assign(member(Expr::This, "p"), word("good"))],
        ),
        (
            "$Order.state is OPEN",
            vec![assign(
                member(Expr::ClassRef("Order".to_string()), "state"),
                word("OPEN"),
            )],
        ),
        ("(a) is good", vec![assign(name("a"), word("good"))]),
        ("a is is", vec![assign(name("a"), word("is"))]),
        ("a is suppose", vec![assign(name("a"), word("suppose"))]),
        ("a is model", vec![assign(name("a"), word("model"))]),
        ("a is why", vec![assign(name("a"), word("why"))]),
        ("a is lambda", vec![assign(name("a"), word("lambda"))]),
        (
            "a is good b",
            vec![assign(name("a"), word("good")), Stmt::Expression(name("b"))],
        ),
        (
            "a is good; b is 2",
            vec![
                assign(name("a"), word("good")),
                assign(name("b"), Expr::Number(2.0)),
            ],
        ),
        (
            "a is 1 b is good",
            vec![
                assign(name("a"), Expr::Number(1.0)),
                assign(name("b"), word("good")),
            ],
        ),
    ] {
        assert_eq!(statements(source), expected, "{source}");
    }
    assert_ne!(statements("a is good"), accepted("a = good"));
    assert_eq!(statements("a is (good)"), accepted("a = good"));
}

#[test]
fn values_are_parsed_as_the_value_of_equals_at_statement_level() {
    for (source, separated) in [
        ("a is b is c", "a is b\nis\nc"),
        ("a is not b is c", "a is not b\nis\nc"),
        ("a is 1 ? 2 : 3", "a = 1 ? 2 : 3"),
        ("a is x => x is 3", "a = x => x\nis\n3"),
        ("a is lambda x: x is 3", "a = lambda x: (x)\nis\n3"),
        ("a is 1 if c else 2 is 3", "a = 1 if c else 2\nis\n3"),
        ("x is y = 4", "x = y = 4"),
        ("t is `${a is 1}`", "t = `${a}`"),
    ] {
        assert_eq!(statements(source), statements(separated), "{source}");
    }
    assert_eq!(
        statements("a is suppose:\n    b = 1"),
        accepted("a = suppose:\n    b = 1")
    );
    assert_eq!(
        statements("a is suppose b > 1:\n    c = 2"),
        accepted("a = suppose b > 1:\n    c = 2")
    );
    assert!(matches!(
        statements("a is suppose:\n    b = 1").first(),
        Some(Stmt::Suppose {
            target: Some(Expr::Identifier(target)),
            ..
        }) if target == "a"
    ));
}

#[test]
fn guard_tokens_and_other_leading_expressions_keep_the_default_reading() {
    for source in [
        "a is: T",
        "a is = 5",
        "a is += 1",
        "a is -= 1",
        "a is *= 2",
        "a is **= 2",
        "a is /= 2",
        "a is %= 2",
        "a is &&= b",
        "a is ||= b",
        "a is := 1",
        "a is",
        "a is;",
        "a is\nb",
        "a is => 1",
        "a is -> 1",
        "a is == b",
        "a is != b",
        "a is === b",
        "a is < b",
        "a is >= b",
        "a is * 2",
        "a is ** 2",
        "a is / 2",
        "a is % 2",
        "a is && b",
        "a is || b",
        "a is and b",
        "a is or b",
        "a is ?? b",
        "a is |> why",
        "a is . b",
        "a is @1",
        "a is if c else d",
        "a is if c:\n    d = 1",
        "f() is 5",
        "1 is 2",
        "\"s\" is 1",
        "a[0] is 1",
        "a[0:1] is 1",
        "this is 5",
        "self is 5",
        "$Order is 5",
        "a + b is 5",
        "(a + b) is 5",
        "-a is 5",
        "not a is 5",
        "[a] is 5",
        "a = b is c",
        "a := b is c",
        "a += b is c",
        "x => x is 5",
        "a if c else b is 5",
        "c ? a : b is 5",
        "def f(x) => x is 1",
        "return a is 1",
        "throw a is 1",
        "delete a is 1",
        "assert: 1, a is 1",
        "a: int",
        "is = 5",
        "is",
        "is(4)",
        "x = {is: 1}",
        "def is(x) => x",
        "for is in xs:\n    pass",
        "while a:\n    is",
    ] {
        assert_eq!(statements(source), accepted(source), "{source}");
    }
    assert_eq!(
        parse_expression_with_options("a is b", IS).unwrap(),
        name("a")
    );
    assert_eq!(
        parse_expression_with_options("(a is b)", IS).unwrap(),
        parse_expression("(a is b)").unwrap()
    );
}

#[test]
fn targets_past_the_chain_limit_keep_the_default_reading() {
    let chain = |links: usize| format!("(root{}){}", ".next".repeat(64), ".next".repeat(links));
    let longest = chain(64);
    assert_eq!(
        statements(&format!("{longest} is 1")),
        accepted(&format!("{longest} = 1"))
    );
    let excessive = format!("{} is 1", chain(65));
    assert_eq!(statements(&excessive), accepted(&excessive));
    assert_eq!(
        syntax_error(&format!("{} := 1", chain(65)), IS).0,
        "Expression has too many parts"
    );
}

#[test]
fn is_assigns_in_every_statement_position() {
    for (spelled, symbol) in [
        ("a is 1", "a = 1"),
        ("x = 0; a is 1", "x = 0; a = 1"),
        ("x = 0 a is 1", "x = 0 a = 1"),
        ("if c:\n    a is 1", "if c:\n    a = 1"),
        ("if c: a is 1", "if c: a = 1"),
        ("if c\n    a is 1", "if c\n    a = 1"),
        (
            "if c:\n    pass\nelif d:\n    a is 1\nelse:\n    a is 2",
            "if c:\n    pass\nelif d:\n    a = 1\nelse:\n    a = 2",
        ),
        ("{ a is 1 }", "{ a = 1 }"),
        ("{\n    a is 1\n    b is 2\n}", "{\n    a = 1\n    b = 2\n}"),
        ("def f():\n    a is 1", "def f():\n    a = 1"),
        ("def f(): a is 1", "def f(): a = 1"),
        ("def f x:\n    a is x + 1", "def f x:\n    a = x + 1"),
        ("function f() { a is 1 }", "function f() { a = 1 }"),
        ("f = () => { a is 1 }", "f = () => { a = 1 }"),
        ("f = function () { a is 1 }", "f = function () { a = 1 }"),
        (
            "class C(p):\n    this.p is p + 1",
            "class C(p):\n    this.p = p + 1",
        ),
        ("class C: this.p is 1", "class C: this.p = 1"),
        (
            "class C:\n    def init(p):\n        self.p is p + 1\n    def m():\n        this.q is 2",
            "class C:\n    def init(p):\n        self.p = p + 1\n    def m():\n        this.q = 2",
        ),
        (
            "try:\n    a is 1\ncatch e:\n    b is e.x",
            "try:\n    a = 1\ncatch e:\n    b = e.x",
        ),
        (
            "try\n    a is 1\ncatch\n    b is 2",
            "try\n    a = 1\ncatch\n    b = 2",
        ),
        (
            "for x in xs:\n    a is x + 1",
            "for x in xs:\n    a = x + 1",
        ),
        ("while c:\n    a is 1", "while c:\n    a = 1"),
        ("while c: a is 1", "while c: a = 1"),
        ("suppose:\n    a is 1", "suppose:\n    a = 1"),
        ("suppose c > 1:\n    a is 1", "suppose c > 1:\n    a = 1"),
        (
            "x = {k: function () { a is 1 }}",
            "x = {k: function () { a = 1 }}",
        ),
        (
            "t = `${(() => { a is 1 })()}`",
            "t = `${(() => { a = 1 })()}`",
        ),
    ] {
        assert_eq!(statements(spelled), accepted(symbol), "{spelled}");
    }
    assert_eq!(
        parse_expression_with_options("() => { a is 1 }", IS).unwrap(),
        parse_expression("() => { a = 1 }").unwrap()
    );
}

#[test]
fn word_assignments_render_with_is_and_every_other_value_renders_as_equals() {
    for (source, rendered) in [
        ("a is good", "a is good"),
        ("o.x is good", "o.x is good"),
        ("$Order.state is OPEN", "$Order.state is OPEN"),
        ("a is \"good\"", "a=\"good\""),
        ("a is 'good'", "a=\"good\""),
        ("a is (good)", "a=good"),
        ("a is 5", "a=5"),
        ("a is not good", "a=!good"),
        ("{ a is good }", "{a is good}"),
        ("if c:\n    a is good", "if(c){a is good}"),
        ("a is suppose:\n    b is good", "a=suppose{b is good}"),
    ] {
        assert_eq!(generate_all(&statements(source)), rendered, "{source}");
    }
    assert_ne!(
        generate_all(&statements("{ a is good }")),
        generate_all(&accepted("{ a = good }"))
    );
}

#[test]
fn a_bare_word_reads_its_variable_once_defined_and_its_text_otherwise() {
    let mut runtime = Runtime::with_options(IS);
    expect(
        &mut runtime,
        "a is good\nquoted is \"good\"\nsingle is 'good'\nassert([a, quoted, single], [\"good\", \"good\", \"good\"])\na",
        "\"good\"",
    );
    let pending = runtime
        .graph
        .retrieve(&NodeKey::variable("good"))
        .expect("the word is a pending node");
    assert_eq!(pending.kind, NodeKind::Pending);
    assert!(pending.dependents.contains(&NodeKey::variable("a")));
    assert!(!pending.dependents.contains(&NodeKey::variable("quoted")));
    expect(
        &mut runtime,
        "good = 3\nassert([a, quoted, single], [3, \"good\", \"good\"])\na",
        "3",
    );
    expect(&mut runtime, "good = 4\na", "4");
    expect(&mut runtime, "delete good\na", "\"good\"");
    expect(&mut runtime, "good = [1, 2]\na", "[1,2]");
    expect(&mut runtime, "good = null\na", "null");
    expect(&mut runtime, "delete good\na", "\"good\"");

    fresh(
        "good = 2\na is good\nassert(a, 2)\ngood = 5\nassert(a, 5)\na",
        "5",
    );
    fresh(
        "o = {x: 1}\no.x is good\nassert(o.x, \"good\")\ngood = 7\nassert(o.x, 7)\ndelete good\no.x",
        "\"good\"",
    );
}

#[test]
fn constructors_and_class_level_rules_assign_with_is_per_instance() {
    let mut runtime = Runtime::with_options(IS);
    expect(
        &mut runtime,
        "class Order(status, total):\n    this.status is status\n    this.total is total\n    this.stage is NEW\n$Order.state is OPEN\n$Order.big is $Order.total > 10\nfirst = Order(\"PAID\", 5)\nsecond = Order(\"SENT\", 20)\nassert([first.status, second.status], [\"PAID\", \"SENT\"])\nassert([first.stage, second.stage], [\"NEW\", \"NEW\"])\nassert([first.state, second.state], [\"OPEN\", \"OPEN\"])\nassert([first.big, second.big], [false, true])\nfirst.state",
        "\"OPEN\"",
    );
    expect(
        &mut runtime,
        "OPEN = \"open\"\nNEW = 0\nassert([first.state, second.state, first.stage], [\"open\", \"open\", 0])\nsecond.state",
        "\"open\"",
    );
    expect(
        &mut runtime,
        "delete OPEN\n[first.state, second.state]",
        "[\"OPEN\",\"OPEN\"]",
    );
    assert_eq!(
        failure(&mut runtime, "$Order.bad is (MISSING)"),
        "ReferenceError: MISSING is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "$Order.bad is MISSING.x"),
        "ReferenceError: MISSING is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "$Order.bad = MISSING"),
        "ReferenceError: MISSING is not defined (line 1, column 1)"
    );
    expect(
        &mut runtime,
        "$Order.ok is MISSING\nfirst.ok",
        "\"MISSING\"",
    );
}

#[test]
fn words_that_name_a_local_class_function_or_global_read_it() {
    fresh(
        "def pick(EXACT):\n    found is EXACT\n    return found\nassert(pick(5), 5)\npick(\"x\")",
        "\"x\"",
    );
    fresh(
        "class Order:\n    pass\ndef helper() => 7\nk is Order\nh is helper\nm is Math\nassert(k == Order, true)\nassert(h(), 7)\nassert(m.PI, Math.PI)\ntypeof h",
        "\"Function\"",
    );
    fresh(
        "h is helper\nassert(h, \"helper\")\ndef helper() => 7\nassert(h(), 7)\nh()",
        "7",
    );
}

#[test]
fn hypotheticals_and_rollback_restore_word_assignments() {
    let mut runtime = Runtime::with_options(IS);
    expect(
        &mut runtime,
        "a is good\nsuppose good = 9:\n    assert(a, 9)\nassert(a, \"good\")\na",
        "\"good\"",
    );
    expect(
        &mut runtime,
        "result is suppose good = 2:\n    a * 10\nassert(result, 20)\nresult",
        "20",
    );
    assert!(!runtime.state.has_variable("good"));
    expect(
        &mut runtime,
        "def check(v):\n    if v == 5:\n        throw \"FIVE\"\n    return v\nguarded is check(a)\nguarded",
        "\"good\"",
    );
    assert_eq!(
        failure(&mut runtime, "a is 5\nguarded"),
        "FIVE (line 1, column 1)"
    );
    expect(&mut runtime, "[a, guarded]", "[\"good\",\"good\"]");
}

#[test]
fn every_statement_position_runs_is_assignments() {
    fresh(
        "if true: one is 1\nif false:\n    two is 0\nelse:\n    two is 2\n{ three is 3 }\ntry:\n    throw \"four\"\ncatch issue:\n    four is issue\nn = 0\nwhile n < 5:\n    n is n + 1\ndef six():\n    local is 6\n    return local\nf = () => { seven is 7 }\nf()\nr = `${[8].map(v => { eight is v; return eight })}`\nassert([one, two, three, four, n, six(), seven, eight], [1, 2, 3, \"four\", 5, 6, 7, 8])\nr",
        "\"8\"",
    );
}

#[test]
fn non_word_values_behave_as_their_equals_spelling() {
    fresh(
        "ready = false\nsum is 1 + 2\nlist is [1, sum]\nobject is {k: sum}\ndouble is x => x * 2\nnext is lambda x: x + 1\nnegated is not ready\nmaybe is ready ?? 1\nassert([sum, list, object.k, double(4), next(4), negated, maybe], [3, [1, 3], 3, 8, 5, true, false])\nready = true\nassert(negated, false)\nsum",
        "3",
    );
    let mut runtime = Runtime::with_options(IS);
    assert_eq!(
        failure(&mut runtime, "a is (good)"),
        "ReferenceError: good is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "a is good.x"),
        "ReferenceError: good is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "a is not good"),
        "ReferenceError: good is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "a@first is 1"),
        "TypeError: State versions are read-only (line 1, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "a = 1\na is b is c"),
        "ReferenceError: is is not defined (line 2, column 8)"
    );
    assert!(!runtime.state.has_variable("a"));
}

#[test]
fn malformed_values_report_the_diagnostics_of_equals() {
    for value in [
        "(1 + )",
        "(1 + )\n",
        "[1, ",
        "{k: }",
        "x =>",
        "1 +",
        "1 + \n",
        "f(",
        "good.",
        "good@",
        "lambda x:",
        "c ? 1",
        "b := 1",
        "suppose 1 >",
    ] {
        let (message, position) = syntax_error(&format!("a is {value}"), IS);
        let (expected, at) = syntax_error(&format!("a = {value}"), IS);
        assert_eq!(message, expected, "a is {value}");
        assert_eq!(position.line, at.line, "a is {value}");
        assert!(
            position.column == at.column + 1 || (position.line > 1 && position.column == at.column),
            "a is {value}: {position:?} against {at:?}"
        );
    }
    assert_eq!(
        syntax_error("a is (1 + )", IS),
        ("Unexpected ')'".to_string(), Position::new(1, 12))
    );
    assert_eq!(
        syntax_error("a is b := 1", IS),
        (
            "Invalid value assignment target".to_string(),
            Position::new(1, 8)
        )
    );
    assert_eq!(
        syntax_error("a is x: T", IS),
        ("Unexpected ':'".to_string(), Position::new(1, 9))
    );
    assert!(Runtime::check("a is x: T").is_ok());
}

#[test]
fn nuc86_comparisons_in_enclosed_positions_work_beside_is_assignments() {
    fresh(
        "value is EXACT\nif value is EXACT:\n    hit is \"yes\"\nflag is (value is EXACT)\nmiss is (value is not EXACT)\nxs = [\"EXACT\", \"B\"]\nfound is xs.filter(x => x is EXACT)\ncount is 0\nwhile count is not 2:\n    count is count + 1\nassert([hit, flag, miss, found, count], [\"yes\", true, false, [\"EXACT\"], 2])\nEXACT = \"B\"\nassert([value, flag, miss, found], [\"B\", true, false, [\"B\"]])\nflag",
        "true",
    );
}

#[test]
fn nuc86_protected_statement_forms_take_new_opt_in_readings() {
    for (source, opted) in [
        (
            "value is \"EXACT\"",
            vec![assign(name("value"), Expr::String("EXACT".to_string()))],
        ),
        (
            "{ a is b }",
            vec![Stmt::Block(vec![assign(name("a"), word("b"))])],
        ),
        ("a is 1 ? 2 : 3", accepted("a = 1 ? 2 : 3")),
    ] {
        assert_eq!(statements(source), opted, "{source}");
        assert_ne!(accepted(source), opted, "{source}");
    }
    assert_eq!(
        statements("while v is x: T"),
        vec![Stmt::While {
            condition: Expr::Binary {
                operator: BinaryOp::Is,
                left: Box::new(name("v")),
                right: Box::new(name("x")),
            },
            body: vec![Stmt::Expression(name("T"))],
        }]
    );
    assert_eq!(
        statements("suppose v is x: T"),
        vec![Stmt::Suppose {
            target: None,
            body: vec![
                Stmt::Expression(Expr::Binary {
                    operator: BinaryOp::Is,
                    left: Box::new(name("v")),
                    right: Box::new(name("x")),
                }),
                Stmt::Expression(name("T")),
            ],
        }]
    );
    assert_eq!(
        statements("while v is suppose:\n    a = 1"),
        accepted("while\nv = suppose:\n    a = 1")
    );
    assert_eq!(
        syntax_error("while v is assert: a, b", IS),
        ("Unexpected ','".to_string(), Position::new(1, 23))
    );
    for source in [
        "value is \"EXACT\"",
        "{ a is b }",
        "a is 1 ? 2 : 3",
        "while v is x: T",
        "suppose v is x: T",
        "while v is suppose:\n    a = 1",
        "while v is assert: a, b",
    ] {
        assert!(Runtime::check(source).is_ok(), "{source}");
        assert_eq!(
            parse_with_options(source, COMMENTS).unwrap(),
            accepted(source),
            "{source}"
        );
    }
}

#[test]
fn explanations_and_graph_keys_show_word_assignments() {
    let mut runtime = Runtime::with_options(IS);
    expect(
        &mut runtime,
        "a is good\nb is 5\no = {x: 1}\no.x is good\nclass Order:\n    pass\n$Order.state is OPEN\nfirst = Order()\n{ total is good }\na",
        "\"good\"",
    );
    let why = json(&mut runtime, "a |> why");
    assert!(why.contains("\"rule\":\"a is good\""), "{why}");
    assert!(why.contains("\"from\":[\"good\"]"), "{why}");
    let why = json(&mut runtime, "b |> why");
    assert!(why.contains("\"rule\":\"b = 5\""), "{why}");
    let why = json(&mut runtime, "o.x |> why");
    assert!(why.contains(".x is good\""), "{why}");
    assert!(why.contains("\"from\":[\"good\"]"), "{why}");
    let why = json(&mut runtime, "first.state |> why");
    assert!(why.contains("\"rule\":\"$Order.state is OPEN\""), "{why}");
    assert!(
        runtime
            .graph
            .retrieve(&NodeKey::variable("block(total is good)"))
            .is_some_and(|node| node.kind == NodeKind::Block)
    );
    assert_eq!(json(&mut runtime, "good |> affects"), "[]");
    run_ok(&mut runtime, "good = 1");
    let affected = json(&mut runtime, "good |> affects");
    assert!(affected.contains("\"node\":\"a\""), "{affected}");
    assert!(affected.contains("\"rule\":\"a is good\""), "{affected}");
    assert!(affected.contains("\"holds\":1"), "{affected}");
}

fn run_ok(runtime: &mut Runtime, source: &str) -> Value {
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
}

#[test]
fn the_option_is_explicit_per_runtime_and_survives_clear() {
    for runtime in [Runtime::new(), Runtime::default()] {
        assert_eq!(runtime.options(), SyntaxOptions::default());
        assert!(!runtime.options().is_assignment);
    }
    let mut runtime = Runtime::with_options(IS);
    assert_eq!(runtime.options(), IS);
    assert_eq!(run_ok(&mut runtime, "a is good\na"), Value::string("good"));
    runtime.clear();
    assert_eq!(runtime.options(), IS);
    assert_eq!(run_ok(&mut runtime, "b is 2\nb"), Value::Number(2.0));
    assert_eq!(
        Runtime::new().run("a is good").unwrap_err().to_string(),
        "ReferenceError: a is not defined (line 1, column 1)"
    );
    assert!(Runtime::check_with_options("a is good", IS).is_ok());
    let program = parse_program_with_options("x = 1\n  \na is good\nb is 2", IS).unwrap();
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
        nucleoid::statement::compile_with_options("a is good", IS)
            .unwrap()
            .statements,
        statements("a is good")
    );
}

#[test]
fn the_option_combines_with_c_style_comments() {
    let mut runtime = Runtime::with_options(BOTH);
    expect(
        &mut runtime,
        "a is good // the word\nb is /* quoted */ \"x\"\nc /* name */ is 6 / 2\nassert([a, b, c], [\"good\", \"x\", 3])\na",
        "\"good\"",
    );
    assert_eq!(
        parse_with_options("a /*\n*/ is 1", BOTH).unwrap(),
        accepted("a\nis 1")
    );
    let mut comments = Runtime::with_options(COMMENTS);
    assert_eq!(
        failure(&mut comments, "a is good // the word"),
        "ReferenceError: a is not defined (line 1, column 1)"
    );
}

#[test]
fn the_documented_examples_run_as_written() {
    fresh(
        "status is OPEN\nlabel is \"shipped\"\ntotal is 2 + 3\nassert([status, label, total], [\"OPEN\", \"shipped\", 5])\nOPEN = \"open\"\nassert(status, \"open\")\ndelete OPEN\nassert(status, \"OPEN\")\nstatus",
        "\"OPEN\"",
    );
    fresh(
        "class Order(total):\n    this.total is total\n    this.stage is NEW\n$Order.state is OPEN\n$Order.big is $Order.total > 10\nfirst = Order(5)\nsecond = Order(20)\nassert([first.state, second.state], [\"OPEN\", \"OPEN\"])\nassert([first.stage, second.big], [\"NEW\", true])\nOPEN = \"open\"\nassert([first.state, second.state], [\"open\", \"open\"])\nfirst.state",
        "\"open\"",
    );
    fresh(
        "value is EXACT\nif value is EXACT:\n    hit is \"yes\"\nflag is (value is EXACT)\nxs = [\"EXACT\", \"B\"]\nfound is xs.filter(x => x is EXACT)\nassert([hit, flag, found], [\"yes\", true, [\"EXACT\"]])\nEXACT = \"B\"\nassert([value, flag, found], [\"B\", true, [\"B\"]])\nfound",
        "[\"B\"]",
    );
    let mut runtime = Runtime::with_options(IS);
    expect(
        &mut runtime,
        "status is OPEN\nlabel is \"shipped\"\ncount is 2 + 3\nOPEN = \"open\"\n[status, label, count]",
        "[\"open\",\"shipped\",5]",
    );
    let mut runtime = Runtime::with_options(SyntaxOptions {
        is_assignment: true,
        ..SyntaxOptions::default()
    });
    runtime.run("status is OPEN").unwrap();
    assert_eq!(runtime.run("status").unwrap().to_string(), "OPEN");
    runtime.run("OPEN = \"open\"").unwrap();
    assert_eq!(runtime.run("status").unwrap().to_string(), "open");
    fresh(
        "pattern is (/colou?r/)\nassert(pattern.test(\"color\"), true)\npattern.test(\"colour\")",
        "true",
    );
    for source in [
        "pattern is /x/ 2",
        "why is 1",
        "affects is 1",
        "stated is 1",
        "derived is 1",
    ] {
        assert_eq!(statements(source), accepted(source), "{source}");
    }
}
