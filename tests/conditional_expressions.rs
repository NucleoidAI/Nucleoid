//! NUC 59: conditional expressions. `condition ? consequent : alternate` and
//! Python's `consequent if condition else alternate` build one expression node
//! that evaluates its condition once and then only the selected branch.

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, Stmt, TemplatePart, parse, parse_expression, parse_program};
use nucleoid::{ErrorKind, Position, Runtime, Value};

/// Both sources parse to the same complete AST and canonical source.
fn equivalent(source: &str, expected: &str) -> Vec<Stmt> {
    let expected = parse(expected).unwrap_or_else(|error| panic!("{expected}: {error}"));
    let actual = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    assert_eq!(actual, expected, "{source}");
    assert_eq!(generate_all(&actual), generate_all(&expected), "{source}");
    actual
}

fn conditional(condition: Expr, consequent: Expr, alternate: Expr) -> Expr {
    Expr::Conditional {
        condition: Box::new(condition),
        consequent: Box::new(consequent),
        alternate: Box::new(alternate),
    }
}

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn run(runtime: &mut Runtime, source: &str) -> Value {
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn text(value: &str) -> Value {
    Value::String(value.to_string())
}

#[test]
fn both_spellings_build_one_node_with_one_grouped_canonical_source() {
    let expected = conditional(name("c"), name("a"), name("b"));
    assert_eq!(parse_expression("c ? a : b").unwrap(), expected);
    assert_eq!(parse_expression("a if c else b").unwrap(), expected);
    assert_eq!(parse_expression("c?a:b").unwrap(), expected);
    assert_eq!(expected.to_string(), "(c?a:b)");

    for source in [
        "x = c ? a : b",
        "x = a if c else b",
        "x = (c ? a : b)",
        "x = (a if c else b)",
    ] {
        assert_eq!(
            generate_all(&parse(source).unwrap()),
            "x=(c?a:b)",
            "{source}"
        );
    }
    for (source, canonical) in [
        ("x = c ? a : d ? e : f", "x=(c?a:(d?e:f))"),
        ("x = a if c else e if d else f", "x=(c?a:(d?e:f))"),
        ("x = (c ? a : d) ? e : f", "x=((c?a:d)?e:f)"),
        ("x = (c ? a : b) + 1", "x=(c?a:b)+1"),
        ("x = c ? a : b + 1", "x=(c?a:b+1)"),
        ("x = c ? a : b |> why", "x=(c?a:b|>why)"),
        ("x = !c ? a : b", "x=(!c?a:b)"),
        ("if c ? a : b:\n    pass", "if((c?a:b)){pass}"),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), canonical, "{source}");
    }
}

#[test]
fn conditionals_bind_above_assignment_and_below_the_reasoning_pipeline() {
    for (source, grouped) in [
        ("x = c ? a : b", "x = (c ? a : b)"),
        ("x = a || b ? c : d", "x = (a || b) ? c : d"),
        ("x = a or b ? c : d", "x = (a || b) ? c : d"),
        ("x = a && b ? c : d", "x = (a && b) ? c : d"),
        ("x = a == b ? c : d", "x = (a == b) ? c : d"),
        ("x = a + 1 > b ? c : d", "x = ((a + 1) > b) ? c : d"),
        ("x = !c ? a : b", "x = (!c) ? a : b"),
        ("x = not c ? a : b", "x = (!c) ? a : b"),
        (
            "x = typeof v == \"number\" ? v : 0",
            "x = (typeof v == \"number\") ? v : 0",
        ),
        ("x = items |> why ? a : b", "x = (items |> why) ? a : b"),
        ("x = c ? items |> why : b", "x = c ? (items |> why) : b"),
        ("x = c ? a : items |> why", "x = c ? a : (items |> why)"),
        ("x = a ? b : c ? d : e", "x = a ? b : (c ? d : e)"),
        ("x = a ? b ? c : d : e", "x = a ? (b ? c : d) : e"),
        ("x = a if b else c if d else e", "x = b ? a : (d ? c : e)"),
        ("x = a ? b : c if d else e", "x = a ? b : (d ? c : e)"),
        ("x = a if b else c ? d : e", "x = b ? a : (c ? d : e)"),
        ("x = a if not b else c", "x = (!b) ? a : c"),
        ("x = a if b or c else d", "x = (b || c) ? a : d"),
        ("x = a if b and c else d", "x = (b && c) ? a : d"),
        ("x = a + 1 if b else c * 2", "x = b ? (a + 1) : (c * 2)"),
        ("x = (a if b else c) if d else e", "x = d ? (b ? a : c) : e"),
        ("x = a if (b if c else d) else e", "x = (c ? b : d) ? a : e"),
        ("x = c ? y = 1 : z", "x = c ? (y = 1) : z"),
        ("x = y = c ? 1 : 2", "x = (y = (c ? 1 : 2))"),
        ("x += c ? 1 : 2", "x = x + (c ? 1 : 2)"),
        ("x ||= 1 if c else 2", "x = x || (c ? 1 : 2)"),
        ("f = v => v ? 1 : 2", "f = v => (v ? 1 : 2)"),
        ("f = v -> 1 if v else 2", "f = v => (v ? 1 : 2)"),
        ("f = lambda v: 1 if v else 2", "f = (v) => (v ? 1 : 2)"),
        ("def f(v) => v ? 1 : 2", "def f(v):\n    return v ? 1 : 2"),
        (
            "def f v => 1 if v else 2",
            "def f(v):\n    return v ? 1 : 2",
        ),
    ] {
        equivalent(source, grouped);
    }
}

#[test]
fn conditionals_are_operands_in_every_expression_position() {
    for (source, expected) in [
        (
            "record = {a: c ? 1 : 2, b: 3}",
            "record = {a: (c ? 1 : 2), b: 3}",
        ),
        (
            "record = {a: c ? 1 : 2 b: 3}",
            "record = {a: (c ? 1 : 2), b: 3}",
        ),
        (
            "record = {a: 1 if c else 2 b: 3}",
            "record = {a: (c ? 1 : 2), b: 3}",
        ),
        (
            "values = [c ? 1 : 2, 3 if d else 4]",
            "values = [(c ? 1 : 2), (d ? 3 : 4)]",
        ),
        (
            "total = Math.max(c ? 1 : 2, 3)",
            "total = Math.max((c ? 1 : 2), 3)",
        ),
        ("total = sum(1 if c else 2)", "total = sum((c ? 1 : 2))"),
        (
            "box = Box(size = c ? 1 : 2)",
            "box = Box(size = (c ? 1 : 2))",
        ),
        ("pick = items[c ? 0 : 1]", "pick = items[(c ? 0 : 1)]"),
        ("pick = items[0 if c else 1]", "pick = items[(c ? 0 : 1)]"),
        ("part = items[c ? 0 : 1 : 3]", "part = items[(c ? 0 : 1):3]"),
        ("part = items[:c ? 1 : 2]", "part = items[:(c ? 1 : 2)]"),
        ("part = items[1 if c else 0:]", "part = items[(c ? 1 : 0):]"),
        (
            "text = `${c ? \"a\" : \"b\"}`",
            "text = `${(c ? \"a\" : \"b\")}`",
        ),
        ("if c ? a : b:\n    pass", "if (c ? a : b):\n    pass"),
        ("if c ? a : b\n    pass", "if (c ? a : b):\n    pass"),
        ("if a if b else c:\n    pass", "if (b ? a : c):\n    pass"),
        (
            "if ready: pass\nelse c ? a : b:\n    pass",
            "if ready: pass\nelse if (c ? a : b):\n    pass",
        ),
        (
            "if ready: pass\nelse c ? a : b\n    pass",
            "if ready: pass\nelse if (c ? a : b):\n    pass",
        ),
        (
            "if ready: pass\nelif a if b else c:\n    pass",
            "if ready: pass\nelse if (b ? a : c):\n    pass",
        ),
        (
            "for item of c ? left : right:\n    total = item",
            "for item of (c ? left : right):\n    total = item",
        ),
        (
            "def f(v):\n    return v if v else 0",
            "def f(v):\n    return (v ? v : 0)",
        ),
        ("throw c ? \"a\" : \"b\"", "throw (c ? \"a\" : \"b\")"),
        ("assert: c ? 1 : 2, 1", "assert((c ? 1 : 2), 1)"),
        ("assert(1 if c else 2, 1)", "assert((c ? 1 : 2), 1)"),
        (
            "class Item:\n    this.size = this.n > 2 ? \"big\" : \"small\"",
            "class Item:\n    this.size = (this.n > 2 ? \"big\" : \"small\")",
        ),
        (
            "$Item.size = \"big\" if $Item.n > 2 else \"small\"",
            "$Item.size = ($Item.n > 2 ? \"big\" : \"small\")",
        ),
    ] {
        equivalent(source, expected);
    }
    let statements = parse("text = `a${c ? 1 : 2}b`").unwrap();
    let [Stmt::Assign { value, .. }] = statements.as_slice() else {
        panic!("one assignment");
    };
    let Expr::Template(parts) = value else {
        panic!("a template");
    };
    assert_eq!(
        parts[1],
        TemplatePart::Expression(conditional(name("c"), Expr::Number(1.0), Expr::Number(2.0)))
    );
}

#[test]
fn lambda_stays_a_name_where_the_conditional_owns_the_colon() {
    assert_eq!(
        parse_expression("c ? lambda : d").unwrap(),
        conditional(name("c"), name("lambda"), name("d"))
    );
    assert_eq!(
        parse_expression("a if lambda else b").unwrap(),
        conditional(name("lambda"), name("a"), name("b"))
    );
    equivalent("x = c ? (lambda: 1) : d", "x = c ? (() => (1)) : d");
    equivalent("x = c ? d : lambda: 1", "x = c ? d : () => (1)");
    equivalent(
        "kinds = items.map(lambda v: \"odd\" if v % 2 else \"even\")",
        "kinds = items.map((v) => (v % 2 ? \"odd\" : \"even\"))",
    );
    assert_eq!(
        syntax_error("x = c ? lambda x: x : d"),
        (
            "Expected ':' but found identifier 'x'".to_string(),
            Position::new(1, 16)
        )
    );
}

#[test]
fn an_if_statement_after_an_expression_keeps_its_accepted_reading() {
    for (source, separated) in [
        ("x = 1 if ready:\n    y = 2", "x = 1\nif ready:\n    y = 2"),
        ("x = 1 if ready\n    y = 2", "x = 1\nif ready\n    y = 2"),
        ("z = 3 if lit: w = 4", "z = 3\nif lit: w = 4"),
        (
            "x = 1 if a: y = 1 else: y = 2",
            "x = 1\nif a: y = 1 else: y = 2",
        ),
        (
            "x = 1 if ready:\n    y = 2\nelse:\n    y = 3",
            "x = 1\nif ready:\n    y = 2\nelse:\n    y = 3",
        ),
        (
            "x = 1 if ready:\n    y = 2\nelse other:\n    y = 3",
            "x = 1\nif ready:\n    y = 2\nelse other:\n    y = 3",
        ),
        (
            "x = 1 if ready:\n    y = 2\nelif other\n    y = 3\nelse\n    y = 4",
            "x = 1\nif ready:\n    y = 2\nelif other\n    y = 3\nelse\n    y = 4",
        ),
        ("x = f() if ready: y = 2", "x = f()\nif ready: y = 2"),
        ("x = 1 if (ready): y = 2", "x = 1\nif (ready): y = 2"),
        (
            "x = 1 if items[0:1]:\n    y = 2",
            "x = 1\nif items[0:1]:\n    y = 2",
        ),
        (
            "x = 1 if {a: 1}.a:\n    y = 2",
            "x = 1\nif {a: 1}.a:\n    y = 2",
        ),
        ("x = 1 if lambda: y = 2", "x = 1\nif lambda: y = 2"),
        (
            "x = 1 if v => v:\n    y = 2",
            "x = 1\nif v => v:\n    y = 2",
        ),
        (
            "x = 1 if ready: y = 2; z = 3",
            "x = 1\nif ready: y = 2; z = 3",
        ),
        (
            "class A:\n    this.x = 1 if this.ready:\n        this.y = 2",
            "class A:\n    this.x = 1\n    if this.ready:\n        this.y = 2",
        ),
        (
            "def f(v):\n    return v if v:\n        pass",
            "def f(v):\n    return v\n    if v:\n        pass",
        ),
    ] {
        equivalent(source, separated);
    }
    let program = parse_program("x = 1 if ready:\n    y = 2").unwrap();
    assert_eq!(
        program.positions,
        [Position::new(1, 1), Position::new(1, 7)]
    );

    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "ready = true\nx = 1 if ready:\n    y = 2\nx",
            Value::Number(1.0),
        ),
        ("y", Value::Number(2.0)),
        ("x = 1 y = 2\ny", Value::Number(2.0)),
        ("if true: y = 1 else: y = 2\ny", Value::Number(1.0)),
        ("mode = \"on\" if ready else \"off\"\nmode", text("on")),
        ("ready = false\nmode", text("off")),
        (
            "[x, y]",
            Value::List(vec![Value::Number(1.0), Value::Number(1.0)]),
        ),
    ] {
        assert_eq!(run(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn nonqualifying_python_forms_keep_the_statement_diagnostics() {
    for (source, message, line, column) in [
        ("x = 1 if c", "Expected ':' but found end of input", 1, 11),
        ("x = 1 if c\n", "Expected ':' but found end of line", 1, 10),
        ("x = 1 if c;", "Expected ':' but found ';'", 1, 11),
        (
            "x = 1 if a b else 2",
            "Expected ':' but found identifier 'b'",
            1,
            12,
        ),
        (
            "x = 1 if lambda: 2 else 3",
            "Expected ':' but found number 3",
            1,
            25,
        ),
        (
            "x = 1 if c ? a : b else 2",
            "Expected ':' but found keyword 'else'",
            1,
            20,
        ),
        (
            "x = 1 if c else 2 if d",
            "Expected ':' but found end of input",
            1,
            23,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn malformed_conditionals_report_exact_errors() {
    for (source, message, line, column) in [
        ("x = c ? 1", "Expected ':' but found end of input", 1, 10),
        ("x = c ? 1\n", "Expected ':' but found end of line", 1, 9),
        ("x = c ? : 1", "Unexpected ':'", 1, 11),
        ("x = ? 1 : 2", "Unexpected '?'", 1, 7),
        ("x = c ? ? 1 : 2", "Unexpected '?'", 1, 11),
        ("x = c ?? 1 : 2", "Unexpected ':'", 1, 14),
        ("x = a?.b", "Unexpected '.'", 1, 8),
        ("x = a ?: b", "Unexpected ':'", 1, 10),
        ("x = c ? 1 : 2 : 3", "Unexpected ':'", 1, 17),
        ("x = c ?\n1 : 2", "Unexpected end of line", 2, 1),
        ("x = 1 if c else", "Unexpected end of input", 1, 16),
        (
            "x = 1 if c else 2 else 3",
            "Unexpected keyword 'else'",
            1,
            24,
        ),
        (
            "x = c ? 1 : 2 += 1",
            "Invalid compound assignment target",
            1,
            15,
        ),
        // Before NUC 59 these were `Unexpected character '?'` lexer errors.
        ("a = 1\nb = ?\n", "Unexpected '?'", 2, 5),
        ("value = 1 ?", "Unexpected end of input", 1, 12),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    let error = Runtime::new().run("x = a ? b : c = 1").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(error.message(), "Cannot assign to (a?b:c)");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn template_interpolations_keep_ignoring_a_trailing_python_if() {
    equivalent("t = `${a if b else c}`", "t = `${a}`");
    equivalent("t = `${x => x if b else c}`", "t = `${x => x}`");
    equivalent("t = `${(a if b else c)}`", "t = `${b ? a : c}`");
    equivalent("t = `${f(a if b else c)}`", "t = `${f(b ? a : c)}`");
    let mut runtime = Runtime::new();
    run(&mut runtime, "a = 1\nb = false\nc = 3");
    for (source, expected) in [
        ("`${a if b else c}`", "1"),
        ("`${(a if b else c)}`", "3"),
        ("`${b ? a : c}`", "3"),
        ("`${a ? \"x\" : \"y\"}-${b ? \"x\" : \"y\"}`", "x-y"),
    ] {
        assert_eq!(run(&mut runtime, source), text(expected), "{source}");
    }
}

#[test]
fn question_marks_in_literals_comments_and_regular_expressions_are_unchanged() {
    let tokens = |source: &str| -> Vec<Token> {
        tokenize(source)
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect()
    };
    assert_eq!(
        tokens("note = \"ready?\" # why?"),
        [
            Token::Identifier("note".into()),
            Token::Assign,
            Token::Str("ready?".into()),
            Token::Eof
        ]
    );
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("\"why?\"", text("why?")),
        ("'why?'", text("why?")),
        ("/colou?r/.test(\"color\")", Value::Bool(true)),
        (
            "ready = true # ready?\nready ? /a?/.test(\"\") : false",
            Value::Bool(true),
        ),
        ("`why? ${ready ? \"yes\" : \"no\"}`", text("why? yes")),
        ("ready ? 10 / 2 : 0", Value::Number(5.0)),
    ] {
        assert_eq!(run(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn conditions_use_the_existing_truthiness_of_if_not_and_logical_operators() {
    for literal in [
        "0",
        "1",
        "-1",
        "0.5",
        "0 / 0",
        "\"\"",
        "\"a\"",
        "[]",
        "[0]",
        "{}",
        "true",
        "false",
        "null",
        "undefined",
        "box.missing",
        "box",
        "Box",
        "x => x",
        "/a/",
    ] {
        let mut runtime = Runtime::new();
        run(&mut runtime, "class Box\nbox = Box()");
        let symbol = run(&mut runtime, &format!("({literal}) ? \"t\" : \"f\""));
        let python = run(&mut runtime, &format!("\"t\" if ({literal}) else \"f\""));
        let logical = run(&mut runtime, &format!("({literal}) && \"t\" || \"f\""));
        run(
            &mut runtime,
            &format!("if ({literal}):\n    branch = \"t\"\nelse:\n    branch = \"f\""),
        );
        let statement = run(&mut runtime, "branch");
        let negated = run(&mut runtime, &format!("not ({literal})"));
        assert_eq!(symbol, statement, "{literal}");
        assert_eq!(python, statement, "{literal}");
        assert_eq!(logical, statement, "{literal}");
        assert_eq!(
            negated == Value::Bool(false),
            symbol == text("t"),
            "{literal}"
        );
    }
}

#[test]
fn unselected_branches_are_neither_evaluated_nor_dependencies() {
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "calls = []\ndef note(tag):\n    calls.push(tag)\n    return tag\nflag = true\nother = 1",
    );
    for (source, expected) in [
        ("safe = flag ? 1 : missing\nsafe", Value::Number(1.0)),
        (
            "deep = 2 if flag else missing.deeper\ndeep",
            Value::Number(2.0),
        ),
        (
            "pick = flag ? note(\"yes\") : note(other)\ncalls",
            Value::List(vec![text("yes")]),
        ),
        ("other = 2\ncalls", Value::List(vec![text("yes")])),
    ] {
        assert_eq!(run(&mut runtime, source), expected, "{source}");
    }
    let error = runtime.run("flag = false").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(
        run(&mut runtime, "[flag, safe, deep, pick, calls]"),
        Value::List(vec![
            Value::Bool(true),
            Value::Number(1.0),
            Value::Number(2.0),
            text("yes"),
            Value::List(vec![text("yes")]),
        ])
    );
    run(&mut runtime, "missing = {deeper: 3}\nflag = false");
    assert_eq!(
        run(&mut runtime, "[safe.deeper, deep, pick, calls]"),
        Value::List(vec![
            Value::Number(3.0),
            Value::Number(3.0),
            Value::Number(2.0),
            Value::List(vec![text("yes"), Value::Number(2.0)]),
        ])
    );
}

#[test]
fn conditional_expressions_differ_from_statement_conditionals_only_as_specified() {
    // A conditional expression belongs to its own declaration. Two `if`
    // statements with the same condition text still replace each other.
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "total = 80\nshipping = 0 if total > 100 else 5\nsize = \"large\" if total > 100 else \"small\"\ntotal = 120",
    );
    assert_eq!(
        run(&mut runtime, "[shipping, size]"),
        Value::List(vec![Value::Number(0.0), text("large")])
    );
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "total = 80\nif total > 100:\n    shipping = 0\nelse:\n    shipping = 5\nif total > 100:\n    size = \"large\"\nelse:\n    size = \"small\"\ntotal = 120",
    );
    assert_eq!(
        run(&mut runtime, "[shipping, size]"),
        Value::List(vec![Value::Number(5.0), text("large")])
    );

    // An undefined class-rule condition selects the alternate and stays live;
    // a class-level `if` statement stands aside until the property exists.
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "class Box\n$Box.state = $Box.ready ? \"on\" : \"off\"\nif $Box.ready:\n    $Box.mode = \"on\"\nelse:\n    $Box.mode = \"off\"\nbox = Box()",
    );
    assert_eq!(run(&mut runtime, "box.state"), text("off"));
    assert_eq!(run(&mut runtime, "box.mode == null"), Value::Bool(true));
    run(&mut runtime, "box.ready = true");
    assert_eq!(
        run(&mut runtime, "[box.state, box.mode]"),
        Value::List(vec![text("on"), text("on")])
    );

    // `.value` is frozen in every operand when the declaration is stored,
    // as in a skipped logical operand; a statement branch reads it when it runs.
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "flag = false\nbase = 1\nfrozen = flag ? base.value : 0\nif flag:\n    late = base.value\nelse:\n    late = 0\nbase = 2\nflag = true",
    );
    assert_eq!(
        run(&mut runtime, "[frozen, late]"),
        Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
    );

    // A stored declaration that read a null condition settles to null, as
    // with `&&` and `||`; an expression statement still selects a branch.
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "toggle = null\nstate = toggle ? \"on\" : \"off\"\nlogical = toggle && \"on\" || \"off\"",
    );
    assert_eq!(
        run(&mut runtime, "[state, logical]"),
        Value::List(vec![Value::Null, Value::Null])
    );
    assert_eq!(run(&mut runtime, "toggle ? \"on\" : \"off\""), text("off"));
    run(&mut runtime, "toggle = false");
    assert_eq!(run(&mut runtime, "state"), text("off"));

    // A conditional expression in a function body adds no statement node, so
    // a later input change cannot re-run it outside the call.
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "limit = 2\ndef size(value) => value > limit ? \"big\" : \"small\"\nsize(1)\nlimit = 0",
    );
    assert_eq!(run(&mut runtime, "size(3)"), text("big"));
}

#[test]
fn class_rule_validation_inspects_every_branch_at_declaration() {
    let mut runtime = Runtime::new();
    run(&mut runtime, "class Order\norder = Order()");
    for source in [
        "$Order.note = $Order.total > 1000 ? vip : \"regular\"",
        "$Order.note = \"regular\" if $Order.total < 1000 else vip",
        "$Order.note = $Order.total > 1000 && vip || \"regular\"",
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference, "{source}");
        assert_eq!(error.message(), "vip is not defined", "{source}");
    }
    run(
        &mut runtime,
        "$Order.limit = $Order.express ? $Order.cap : 50",
    );
    let error = runtime.run("$Order.cap = $Order.limit + 1").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Circular Dependency");
    assert_eq!(run(&mut runtime, "order.limit"), Value::Number(50.0));
}

#[test]
fn parse_expression_reads_either_spelling_and_ignores_what_follows() {
    let expected = conditional(name("b"), name("a"), name("c"));
    assert_eq!(parse_expression("a if b else c").unwrap(), expected);
    assert_eq!(parse_expression("b ? a : c").unwrap(), expected);
    assert_eq!(parse_expression("a if b else c d").unwrap(), expected);
    assert_eq!(parse_expression("a if b").unwrap(), name("a"));
    assert_eq!(parse_expression("a if b: c").unwrap(), name("a"));
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
fn the_native_repl_buffers_colon_free_headers_containing_conditionals() {
    for header in ["if ready ? false : true", "if (false if ready else true)"] {
        let output = repl(&format!(
            "ready = false\n{header}\n    result = \"flipped\"\n\nresult\nlabel = \"on\" if ready else \"off\"\n"
        ));
        assert!(output.status.success(), "{header}\n{output:?}");
        assert_eq!(output.stderr, b"", "{header}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{BANNER}false\n\"flipped\"\n\"flipped\"\n\"off\"\n"),
            "{header}"
        );
    }
}
