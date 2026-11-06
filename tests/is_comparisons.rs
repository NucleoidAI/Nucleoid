mod common;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::{NodeKey, NodeKind};
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{BinaryOp, Expr, LogicalOp, Stmt, UnaryOp, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn binary(operator: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::Binary {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn expression(source: &str) -> Expr {
    parse_expression(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn compares_words(statements: &[Stmt]) -> bool {
    let tree = format!("{statements:?}");
    tree.contains("operator: Is,") || tree.contains("operator: IsNot,")
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn expect(runtime: &mut Runtime, source: &str, expected: &str) {
    let assertions = common::assertions::protected_assertions(&statements(source)).len();
    let case = common::Case {
        title: "is comparison regression".to_string(),
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

#[test]
fn a_bare_word_is_a_word_comparison_and_every_other_operand_is_equality() {
    assert_eq!(
        expression("(value is EXACT)"),
        binary(BinaryOp::Is, name("value"), name("EXACT"))
    );
    assert_eq!(
        expression("(value is not EXACT)"),
        binary(BinaryOp::IsNot, name("value"), name("EXACT"))
    );
    assert_eq!(
        expression("(value is is)"),
        binary(BinaryOp::Is, name("value"), name("is"))
    );
    for (word, symbol) in [
        ("(value is \"EXACT\")", "(value == \"EXACT\")"),
        ("(value is 'EXACT')", "(value == \"EXACT\")"),
        ("(value is not \"EXACT\")", "(value != \"EXACT\")"),
        ("(value is (EXACT))", "(value == EXACT)"),
        ("(value is not (EXACT))", "(value != EXACT)"),
        ("(value is EXACT.x)", "(value == EXACT.x)"),
        ("(value is EXACT())", "(value == EXACT())"),
        ("(value is EXACT[0])", "(value == EXACT[0])"),
        ("(value is EXACT@1)", "(value == EXACT@1)"),
        ("(value is EXACT + 1)", "(value == EXACT + 1)"),
        ("(value is -EXACT)", "(value == -EXACT)"),
        ("(value is $Order)", "(value == $Order)"),
        ("(value is true)", "(value == true)"),
        ("(value is false)", "(value == false)"),
        ("(value is null)", "(value == null)"),
        ("(value is None)", "(value == null)"),
        ("(value is undefined)", "(value == null)"),
        ("(value is this)", "(value == this)"),
        ("(value is self)", "(value == this)"),
        ("(value is 3)", "(value == 3)"),
        ("(value is [1, 2])", "(value == [1, 2])"),
        ("(value is {k: 1})", "(value == {k: 1})"),
        ("(value is not not ready)", "(value != not ready)"),
        ("(value is (not ready))", "(value == (not ready))"),
        ("(value is typeof ready)", "(value == typeof ready)"),
        (
            "(order.status is order.goal)",
            "(order.status == order.goal)",
        ),
    ] {
        assert_eq!(expression(word), expression(symbol), "{word}");
    }
}

#[test]
fn is_shares_the_level_and_associativity_of_equality() {
    assert_eq!(
        expression("(a is b == c)"),
        binary(
            BinaryOp::Equal,
            binary(BinaryOp::Is, name("a"), name("b")),
            name("c")
        )
    );
    assert_eq!(
        expression("(a == b is c)"),
        binary(
            BinaryOp::Is,
            binary(BinaryOp::Equal, name("a"), name("b")),
            name("c")
        )
    );
    assert_eq!(
        expression("(a is b is not c)"),
        binary(
            BinaryOp::IsNot,
            binary(BinaryOp::Is, name("a"), name("b")),
            name("c")
        )
    );
    assert_eq!(expression("(a is b < c)"), expression("(a == (b < c))"));
    assert_eq!(
        expression("(a < b is c)"),
        binary(
            BinaryOp::Is,
            binary(BinaryOp::Less, name("a"), name("b")),
            name("c")
        )
    );
    assert_eq!(
        expression("(not a is b)"),
        binary(
            BinaryOp::Is,
            Expr::Unary {
                operator: UnaryOp::Not,
                operand: Box::new(name("a")),
            },
            name("b")
        )
    );
    assert_eq!(
        expression("(a is b and c is not d)"),
        Expr::Logical {
            operator: LogicalOp::And,
            left: Box::new(binary(BinaryOp::Is, name("a"), name("b"))),
            right: Box::new(binary(BinaryOp::IsNot, name("c"), name("d"))),
        }
    );
    assert_eq!(
        expression("(a is b ? c : d)"),
        Expr::Conditional {
            condition: Box::new(binary(BinaryOp::Is, name("a"), name("b"))),
            consequent: Box::new(name("c")),
            alternate: Box::new(name("d")),
        }
    );
}

#[test]
fn word_comparisons_render_grouped_and_other_comparisons_render_as_equality() {
    for (source, rendered) in [
        ("if value is EXACT:\n    pass", "if((value is EXACT)){pass}"),
        (
            "if value is not EXACT:\n    pass",
            "if((value is not EXACT)){pass}",
        ),
        (
            "if value is \"EXACT\":\n    pass",
            "if(value==\"EXACT\"){pass}",
        ),
        (
            "if value is 'EXACT':\n    pass",
            "if(value==\"EXACT\"){pass}",
        ),
        ("if value is not 3:\n    pass", "if(value!=3){pass}"),
        ("flag = (value is EXACT)", "flag=(value is EXACT)"),
        ("flag = (a is b == c)", "flag=(a is b)==c"),
        ("flag = (a is b) ** 2", "flag=(a is b)**2"),
        ("flag = 2 ** (a is b)", "flag=2**(a is b)"),
        ("flag = (a is b).length", "flag=(a is b).length"),
    ] {
        assert_eq!(generate_all(&statements(source)), rendered, "{source}");
    }
    assert_ne!(
        generate_all(&statements("if value is EXACT:\n    pass")),
        generate_all(&statements("if value == EXACT:\n    pass"))
    );
}

#[test]
fn is_compares_in_every_enclosed_position() {
    for source in [
        "if x is Y:\n    pass",
        "if x is Y\n    pass",
        "if a:\n    pass\nelif x is Y:\n    pass",
        "if a:\n    pass\nelse if x is Y:\n    pass",
        "if a:\n    pass\nelse x is Y:\n    pass",
        "if a:\n    pass\nelse x is Y\n    pass",
        "while x is not Y:\n    pass",
        "while x is not Y: pass",
        "for item in xs if x is Y else ys:\n    pass",
        "ok = all v in (xs if x is Y else ys): v",
        "ok = c ? x is Y : 0",
        "ok = 1 if x is Y else 0",
        "ok = xs[x is Y ? 0 : 1]",
        "ok = xs[x is Y:]",
        "ok = xs[:x is Y]",
        "ok = (x is Y)",
        "ok = f(x is Y)",
        "ok = f(1, x is Y)",
        "ok = [x is Y]",
        "ok = {k: x is Y}",
        "ok = {k: 1, j: x is Y}",
        "ok = {...(x is Y)}",
        "ok = {k: v => v is Y}",
        "ok = xs.filter(v => v is Y)",
        "ok = xs.filter(v -> v is not Y)",
        "ok = xs.filter(lambda v: v is Y)",
        "ok = xs.some(v => v is Y ? true : false)",
        "ok = (c ? 0 : x is Y)",
        "ok = !(x is Y)",
        "ok = `${(x is Y)}`",
        "ok = x |> depth(x is Y)",
        "if all v in xs: v is Y:\n    pass",
        "def same(v) =>\n    (v is Y)",
        "assert: x is Y, true",
        "assert(x is Y, true)",
        "suppose x is Y:\n    pass",
        "ok = super(x is Y)",
        "a, b = [x is Y, 1]",
        "{a, b} = f(x is Y)",
    ] {
        assert!(compares_words(&statements(source)), "{source}");
    }
}

#[test]
fn is_stays_a_name_where_a_statement_can_follow() {
    for (source, separated) in [
        ("value is \"EXACT\"", "value\nis\n\"EXACT\""),
        ("flag = value is \"EXACT\"", "flag = value\nis\n\"EXACT\""),
        ("flag = value is not EXACT", "flag = value\nis\nnot EXACT"),
        ("f = x => x is 3", "f = x => x\nis\n3"),
        ("f = x -> x is 3", "f = x -> x\nis\n3"),
        ("f = lambda x: x is 3", "f = lambda x: (x)\nis\n3"),
        ("a is 1 ? 2 : 3", "a\nis\n1 ? 2 : 3"),
        ("x = c ? 1 : 2 is 3", "x = c ? 1 : 2\nis\n3"),
        ("x = 1 if c else 2 is 3", "x = 1 if c else 2\nis\n3"),
        ("x = a is b if c else d", "x = a\nis\nb if c else d"),
        ("ok = all x in xs: x is 1", "ok = all x in xs: x\nis\n1"),
        ("x = not a is true", "x = not a\nis\ntrue"),
        ("x := y is 1", "x := y\nis\n1"),
        ("x += y is 1", "x += y\nis\n1"),
        ("assert: 1, x is 1", "assert: 1, x\nis\n1"),
        ("def f(x) => x is 1", "def f(x) => x\nis\n1"),
        (
            "def f(x):\n    return x is 1",
            "def f(x):\n    return x\n    is\n    1",
        ),
        ("throw x is 1", "throw x\nis\n1"),
        ("delete x is 1", "delete x\nis\n1"),
        ("{ a is b }", "{ a\nis\nb }"),
        ("while v is x: T", "while v\nis\nx: T"),
        ("while v is assert: a, b", "while v\nis\nassert: a, b"),
        (
            "while v is suppose:\n    a = 1",
            "while v\nis\nsuppose:\n    a = 1",
        ),
        ("suppose v is x: T", "suppose v\nis\nx: T"),
        ("x = {k: 1 is: 2}", "x = {k: 1, is: 2}"),
        ("t = `${a is 1}`", "t = `${a}`"),
        ("t = `${a is}`", "t = `${a}`"),
    ] {
        let read = statements(source);
        assert_eq!(read, statements(separated), "{source}");
        assert!(!compares_words(&read), "{source}");
    }
    for source in [
        "is = 5",
        "is",
        "is(4)",
        "x = {is: 1}",
        "x = {is}",
        "x = a.is",
        "def is(x) => x",
        "def f(is) => is",
        "is: int",
        "class A:\n    is: int",
        "if is:\n    pass",
        "x = (is)",
        "x = [is, is]",
        "for is in xs:\n    pass",
        "xs.map(is => is)",
    ] {
        assert!(!compares_words(&statements(source)), "{source}");
    }
    assert_eq!(expression("a is b"), name("a"));
    assert_eq!(
        expression("(a is b)"),
        binary(BinaryOp::Is, name("a"), name("b"))
    );
}

#[test]
fn equivalent_spellings_keep_one_syntax_tree_with_is() {
    for (spelling, equivalent) in [
        (
            "def same(v) =>\n    (v is Y)",
            "def same(v):\n    return (v is Y)",
        ),
        ("if x is Y\n    pass", "if x is Y:\n    pass"),
        (
            "if a:\n    pass\nelse x is Y:\n    pass",
            "if a:\n    pass\nelse if x is Y:\n    pass",
        ),
        (
            "if a:\n    pass\nelif x is Y:\n    pass",
            "if a:\n    pass\nelse if x is Y:\n    pass",
        ),
        ("assert: x is Y, true", "assert(x is Y, true)"),
        ("ok = xs.map(v -> v is Y)", "ok = xs.map(v => v is Y)"),
        (
            "ok = xs.map(lambda v: v is Y)",
            "ok = xs.map((v) => (v is Y))",
        ),
        ("while x is Y: pass", "while x is Y:\n    pass"),
        ("ok = {a: x is Y b: 1}", "ok = {a: x is Y, b: 1}"),
    ] {
        assert_eq!(statements(spelling), statements(equivalent), "{spelling}");
    }
    assert_eq!(
        syntax_error("def same(v) =>\n    v is Y"),
        (
            "Expected the end of a block but found identifier 'is'".to_string(),
            Position::new(2, 7)
        )
    );
    assert_eq!(
        statements("def same(v):\n    return v is Y"),
        statements("def same(v):\n    return v\n    is\n    Y")
    );
}

#[test]
fn a_program_read_with_the_operator_keeps_headers_that_also_read_as_statements() {
    let read = statements("while v is x: T\nif a is b:\n    c = 1");
    assert_eq!(read.get(..4), statements("while\nv\nis\nx: T").get(..));
    assert!(compares_words(&read[4..]));
    let read = statements("suppose v is x: T\nif a is b:\n    c = 1");
    assert_eq!(read.get(..4), statements("suppose\nv\nis\nx: T").get(..));
    let read = statements("v = 1\nwhile v is suppose:\n    a = 1\nif v is 1:\n    b = 2");
    assert!(matches!(read.get(3), Some(Stmt::Expression(Expr::Identifier(word))) if word == "is"));
    assert!(matches!(read.get(4), Some(Stmt::Suppose { .. })));
    let read = statements("n = 0\nwhile n < 3:\n    n = n + 1\nif n is DONE:\n    pass");
    assert!(matches!(read.get(1), Some(Stmt::While { .. })));
    let read = statements("n = 0\nwhile n is not LIMIT: n = n + 1");
    assert!(matches!(read.get(1), Some(Stmt::While { .. })));
    assert!(compares_words(&read));
}

#[test]
fn malformed_is_forms_report_the_diagnostics_of_equality() {
    for (word, symbol) in [
        ("if value is:\n    a = 1", "if value ==:\n    a = 1"),
        ("if value is not:\n    a = 1", "if value !=:\n    a = 1"),
        ("if value is", "if value =="),
        ("if value is EXACT", "if value == EXACT"),
        ("x = (value is)", "x = (value ==)"),
        ("x = (value is not)", "x = (value !=)"),
        ("x = [value is]", "x = [value ==]"),
        ("x = {k: value is}", "x = {k: value ==}"),
        ("x = (a is b c)", "x = (a == b c)"),
        ("x = (a is =)", "x = (a == =)"),
        ("x = (a is .)", "x = (a == .)"),
        ("f(a is, b)", "f(a ==, b)"),
        (
            "if value is EXACT then:\n    a = 1",
            "if value == EXACT then:\n    a = 1",
        ),
    ] {
        let (message, position) = syntax_error(word);
        let (expected, at) = syntax_error(symbol);
        assert_eq!(message, expected, "{word}");
        assert_eq!(position.line, at.line, "{word}");
        let shift = word.len().abs_diff(symbol.len());
        assert!(
            position.column == at.column || position.column == at.column + shift,
            "{word}: {position:?} against {at:?}"
        );
    }
    assert_eq!(
        syntax_error("if value is:\n    a = 1"),
        ("Unexpected ':'".to_string(), Position::new(1, 12))
    );
    assert_eq!(
        syntax_error("if value is"),
        ("Unexpected end of input".to_string(), Position::new(1, 12))
    );
    assert_eq!(
        syntax_error("x = (value is)"),
        ("Unexpected ')'".to_string(), Position::new(1, 15))
    );
    assert_eq!(
        syntax_error("x = (value is)\n"),
        ("Unexpected ')'".to_string(), Position::new(1, 14))
    );
}

#[test]
fn a_failing_program_reports_the_accepted_error_unless_the_operator_reads_further() {
    for (source, message, position) in [
        ("x = (1 + )", "Unexpected ')'", Position::new(1, 11)),
        (
            "f(a b)",
            "Expected ')' but found identifier 'b'",
            Position::new(1, 5),
        ),
        (
            "value = 1\nflag = value is 2 b c: d e",
            "Unexpected ':'",
            Position::new(2, 24),
        ),
        (
            "p = [1, 2]\na, b = p is q\n[a, b]\n",
            "Unexpected ','",
            Position::new(2, 4),
        ),
        (
            "x = (1 + )\nif v is X:\n    a = 1",
            "Unexpected ')'",
            Position::new(1, 10),
        ),
        (
            "while v is X:\n    a = 1\nx = (1 + )",
            "Unexpected ')'",
            Position::new(3, 11),
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), position),
            "{source}"
        );
    }
}

#[test]
fn one_chain_holds_at_most_128_comparison_operators() {
    let chain = |count: usize| format!("x = (a{})", " is a".repeat(count));
    assert!(Runtime::check(&chain(128)).is_ok());
    assert_eq!(
        syntax_error(&chain(129)).0,
        "Expression has too many operands"
    );
    let mixed = format!("x = (a{} is not a)", " == a".repeat(127));
    assert!(Runtime::check(&mixed).is_ok());
    let mixed = format!("x = (a{} is not a)", " == a".repeat(128));
    assert_eq!(syntax_error(&mixed).0, "Expression has too many operands");
}

#[test]
fn a_bare_word_reads_its_variable_once_defined_and_its_text_otherwise() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "value = \"EXACT\"\nflag = (value is EXACT)\nmiss = (value is not EXACT)\nassert(flag, true)\nassert(miss, false)\n[flag, miss]",
        "[true,false]",
    );
    let pending = runtime
        .graph
        .retrieve(&NodeKey::variable("EXACT"))
        .expect("the word is a pending node");
    assert_eq!(pending.kind, NodeKind::Pending);
    assert!(pending.dependents.contains(&NodeKey::variable("flag")));
    assert!(pending.dependents.contains(&NodeKey::variable("miss")));
    expect(
        &mut runtime,
        "EXACT = \"OTHER\"\n[flag, miss]",
        "[false,true]",
    );
    expect(
        &mut runtime,
        "value = \"OTHER\"\n[flag, miss]",
        "[true,false]",
    );
    expect(&mut runtime, "delete EXACT\n[flag, miss]", "[false,true]");
    expect(
        &mut runtime,
        "value = \"EXACT\"\n[flag, miss]",
        "[true,false]",
    );
    expect(
        &mut runtime,
        "EXACT = 7\nvalue = 7\n[flag, miss]",
        "[true,false]",
    );
    expect(&mut runtime, "value = \"7\"\n[flag, miss]", "[true,false]");
    expect(&mut runtime, "value = \"EXACT\"\nflag", "false");
}

#[test]
fn standing_rules_follow_a_word_through_definition_change_and_deletion() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "status = \"SHIPPED\"\nif status is SHIPPED:\n    stage = \"done\"\nelse:\n    stage = \"open\"\nstage",
        "\"done\"",
    );
    expect(&mut runtime, "SHIPPED = \"SENT\"\nstage", "\"open\"");
    expect(&mut runtime, "status = \"SENT\"\nstage", "\"done\"");
    expect(&mut runtime, "delete SHIPPED\nstage", "\"open\"");
    expect(&mut runtime, "status = \"SHIPPED\"\nstage", "\"done\"");
    assert!(
        runtime
            .graph
            .retrieve(&NodeKey::variable("SHIPPED"))
            .is_some_and(|node| node.kind == NodeKind::Pending)
    );
}

#[test]
fn names_that_exist_are_read_as_names_and_quotes_or_grouping_choose_explicitly() {
    fresh(
        "class Order:\n    pass\ndef helper() => 1\nkind = \"Order\"\nfound = [(kind is Order), (kind is \"Order\"), (kind is Math), (kind is helper), (kind is ORDER)]\nfound",
        "[false,true,false,false,false]",
    );
    fresh(
        "def pick(EXACT, candidate) => (candidate is EXACT)\n[pick(\"A\", \"A\"), pick(\"A\", \"EXACT\")]",
        "[true,false]",
    );
    fresh(
        "xs = [\"x\", \"y\"]\nfound = xs.filter(item => item is x)\nfound",
        "[\"x\"]",
    );
    let mut runtime = Runtime::new();
    expect(&mut runtime, "value = \"EXACT\"\nvalue", "\"EXACT\"");
    assert_eq!(
        failure(&mut runtime, "flag = (value is (EXACT))"),
        "ReferenceError: EXACT is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "flag = (EXACT is value)"),
        "ReferenceError: EXACT is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "flag = (value is EXACT.size)"),
        "ReferenceError: EXACT is not defined (line 1, column 1)"
    );
}

#[test]
fn class_rules_accept_undefined_words_but_still_validate_names() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "class Order(status):\n    this.status = status\n$Order.shipped = ($Order.status is SHIPPED)\nif $Order.status is not SHIPPED:\n    $Order.waiting = true\nelse:\n    $Order.waiting = false\nfirst = Order(\"SHIPPED\")\nsecond = Order(\"OPEN\")\n[first.shipped, second.shipped, first.waiting, second.waiting]",
        "[true,false,false,true]",
    );
    expect(
        &mut runtime,
        "SHIPPED = \"OPEN\"\n[first.shipped, second.shipped, first.waiting, second.waiting]",
        "[false,true,true,false]",
    );
    assert_eq!(
        failure(&mut runtime, "$Order.bad = ($Order.status is (MISSING))"),
        "ReferenceError: MISSING is not defined (line 1, column 1)"
    );
    assert_eq!(
        failure(&mut runtime, "$Order.bad = ($Order.status is MISSING.x)"),
        "ReferenceError: MISSING is not defined (line 1, column 1)"
    );
}

#[test]
fn word_comparisons_explain_their_pending_and_defined_sources() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "value = \"EXACT\"\nflag = (value is EXACT)\nflag",
        "true",
    );
    let explained = runtime.run("flag |> why").unwrap();
    let rendered = runtime.serialize_json(&explained).unwrap();
    assert!(
        rendered.contains("\"from\":[\"value\",\"EXACT\"]"),
        "{rendered}"
    );
    assert!(
        rendered.contains("\"rule\":\"flag = (value is EXACT)\""),
        "{rendered}"
    );
    let affected = runtime.run("EXACT |> affects").unwrap();
    assert_eq!(runtime.serialize_json(&affected).unwrap(), "[]");
    run_ok(&mut runtime, "EXACT = \"OTHER\"");
    let affected = runtime.run("EXACT |> affects").unwrap();
    let rendered = runtime.serialize_json(&affected).unwrap();
    assert!(rendered.contains("\"node\":\"flag\""), "{rendered}");
    assert!(rendered.contains("\"holds\":false"), "{rendered}");
}

fn run_ok(runtime: &mut Runtime, source: &str) -> Value {
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
}

#[test]
fn hypotheticals_and_rollback_restore_word_comparisons() {
    let mut runtime = Runtime::new();
    expect(
        &mut runtime,
        "value = \"EXACT\"\nflag = (value is EXACT)\nsuppose EXACT = \"OTHER\":\n    assert(flag, false)\nflag",
        "true",
    );
    expect(
        &mut runtime,
        "def check(blocked):\n    if blocked:\n        throw \"BLOCKED\"\n    return \"ok\"\nstate = check(value is not EXACT)\nstate",
        "\"ok\"",
    );
    assert_eq!(
        failure(&mut runtime, "EXACT = \"OTHER\""),
        "BLOCKED (line 1, column 1)"
    );
    expect(&mut runtime, "[state, flag]", "[\"ok\",true]");
    assert_eq!(
        failure(&mut runtime, "EXACT"),
        "ReferenceError: EXACT is not defined (line 1, column 1)"
    );
}

#[test]
fn bound_words_literals_and_properties_compare_exactly_as_with_equality() {
    let names = [
        "zero", "one", "three", "digit", "empty", "letter", "yes", "no", "nothing", "single",
        "none", "order",
    ];
    let lefts = [
        "0",
        "1",
        "3",
        "\"3\"",
        "\"\"",
        "\"a\"",
        "true",
        "false",
        "null",
        "[1]",
        "[]",
        "order.total",
        "order.paid",
        "order.status",
        "order.missing",
        "zero",
        "nothing",
    ];
    let mut runtime = Runtime::new();
    run_ok(
        &mut runtime,
        "zero = 0\none = 1\nthree = 3\ndigit = \"3\"\nempty = \"\"\nletter = \"a\"\nyes = true\nno = false\nnothing = null\nsingle = [1]\nnone = []\norder = {total: 3, paid: true, status: \"OPEN\"}",
    );
    for left in lefts {
        for right in names.iter().copied().chain(lefts) {
            let word = run_ok(
                &mut runtime,
                &format!("[({left} is {right}), ({left} is not {right})]"),
            );
            let symbol = run_ok(
                &mut runtime,
                &format!("[({left} == {right}), ({left} != {right})]"),
            );
            assert_eq!(word, symbol, "{left} is {right}");
        }
    }
}

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

fn repl(source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
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

#[test]
fn the_native_repl_buffers_is_headers_and_keeps_statement_readings() {
    let output = repl(
        "value = \"EXACT\"\nif value is EXACT\n    result = \"matched\"\n\nresult\nlate = (value is not EXACT)\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stderr, b"");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{BANNER}\"EXACT\"\n\"matched\"\n\"matched\"\nfalse\n")
    );
    let output = repl("value = \"EXACT\"\nlate = value is EXACT\n");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "ReferenceError: is is not defined (line 1, column 14)\n"
    );
}
