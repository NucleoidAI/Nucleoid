mod common;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::{NodeKey, NodeKind};
use nucleoid::lang::ast::generator::{generate, generate_all};
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{BinaryOp, Expr, Stmt, parse, parse_expression};
use nucleoid::nuc::Nuc;
use nucleoid::value::ObjectId;
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde_json::json;

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid applies a value assignment once",
    "Nucleoid replaces a standing relationship with a value assignment",
    "Nucleoid assigns instance properties once with value assignments",
    "Nucleoid records no dependency through calls in a value assignment",
    "Nucleoid runs a value assignment whenever its conditional branch runs",
    "Nucleoid uses value assignments in functions blocks and loops",
    "Nucleoid keeps colon readings beside value assignments",
    "Nucleoid rolls back failed value assignments",
];

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "value assignment regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
    value
}

fn integral(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Number(number) => match number.as_f64() {
            Some(float) if float.fract() == 0.0 && float.abs() < 1e15 => json!(float as i64),
            _ => serde_json::Value::Number(number),
        },
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(integral).collect())
        }
        serde_json::Value::Object(fields) => serde_json::Value::Object(
            fields
                .into_iter()
                .map(|(key, value)| (key, integral(value)))
                .collect(),
        ),
        other => other,
    }
}

fn read(runtime: &mut Runtime, source: &str) -> serde_json::Value {
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    integral(serde_json::from_str(&runtime.serialize_json(&value).unwrap()).unwrap())
}

fn evaluate(source: &str) -> serde_json::Value {
    read(&mut Runtime::new(), source)
}

fn failure(runtime: &mut Runtime, source: &str) -> (ErrorKind, String, Option<Position>) {
    let error = runtime.run(source).unwrap_err();
    (error.kind(), error.message(), error.position())
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn member(object: Expr, property: &str) -> Expr {
    Expr::Member {
        object: Box::new(object),
        property: property.to_string(),
    }
}

fn value_assign(target: Expr, value: Expr) -> Stmt {
    Stmt::ValueAssign { target, value }
}

fn node<'a>(runtime: &'a Runtime, key: &NodeKey) -> &'a nucleoid::graph::GraphNode {
    runtime
        .graph
        .retrieve(key)
        .unwrap_or_else(|| panic!("{key:?} is filed"))
}

fn stored(runtime: &Runtime, key: &NodeKey) -> Expr {
    match node(runtime, key).node.as_ref() {
        Some(Nuc::Variable(variable)) => variable.value.clone(),
        Some(Nuc::Property(property)) => property.value.clone(),
        other => panic!("{key:?} holds {other:?}"),
    }
}

fn contains_value_assignment(statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| match statement {
        Stmt::ValueAssign { .. } => true,
        Stmt::If {
            consequent,
            alternate,
            ..
        } => {
            contains_value_assignment(consequent)
                || alternate.as_deref().is_some_and(|statement| {
                    contains_value_assignment(std::slice::from_ref(statement))
                })
        }
        Stmt::Block(body) | Stmt::For { body, .. } => contains_value_assignment(body),
        Stmt::Class(class) => {
            contains_value_assignment(&class.constructor)
                || class.methods.iter().any(|method| match &method.body {
                    nucleoid::lang::ast::FunctionBody::Block(body) => {
                        contains_value_assignment(body)
                    }
                    nucleoid::lang::ast::FunctionBody::Expression(_) => false,
                })
        }
        Stmt::Function(function) => match &function.body {
            nucleoid::lang::ast::FunctionBody::Block(body) => contains_value_assignment(body),
            nucleoid::lang::ast::FunctionBody::Expression(_) => false,
        },
        Stmt::Try { body, catch, .. } => {
            contains_value_assignment(body) || contains_value_assignment(catch)
        }
        _ => false,
    })
}

#[test]
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let readme = common::snippets(include_str!("../README.md"), "README.md");
    let examples = common::snippets(include_str!("../docs/examples.md"), "docs/examples.md");
    let appended = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (
            &reference,
            "Nucleoid separates one-shot actions from standing laws",
        ),
        (
            &readme,
            "Value assignments take an action once while declarations stay live",
        ),
        (
            &examples,
            "Value assignments lock deposit rates while interest stays live",
        ),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        let statements = parse(&case.source).unwrap();
        assert!(contains_value_assignment(&statements), "{title} uses :=");
        let count = common::assertions::protected_assertions(&statements).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 80);
}

#[test]
fn accepted_documents_use_value_assignments_only_in_the_appended_cases() {
    let documents = [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ];
    let appended = [
        "Nucleoid separates one-shot actions from standing laws",
        "Value assignments take an action once while declarations stay live",
        "Value assignments lock deposit rates while interest stays live",
    ];
    let mut with_operator = 0;
    let mut cases = 0;
    for case in documents.iter().flatten() {
        cases += 1;
        let uses = tokenize(case.source.trim())
            .unwrap()
            .iter()
            .any(|spanned| spanned.token == Token::ColonAssign);
        let new =
            SPEC_TITLES.contains(&case.title.as_str()) || appended.contains(&case.title.as_str());
        if uses {
            with_operator += 1;
        }
        assert_eq!(uses, new, "{}", case.title);
    }
    assert_eq!(with_operator, 11);
    assert_eq!(cases, 780);
}

#[test]
fn value_assignments_parse_into_one_statement() {
    let sum = Expr::Binary {
        operator: BinaryOp::Add,
        left: Box::new(name("a")),
        right: Box::new(Expr::Number(1.0)),
    };
    for (source, expected) in [
        ("x := a + 1", vec![value_assign(name("x"), sum.clone())]),
        ("x:=a+1", vec![value_assign(name("x"), sum.clone())]),
        ("(x) := a + 1", vec![value_assign(name("x"), sum.clone())]),
        (
            "box.size := a + 1",
            vec![value_assign(member(name("box"), "size"), sum.clone())],
        ),
        (
            "a.b.c := 2",
            vec![value_assign(
                member(member(name("a"), "b"), "c"),
                Expr::Number(2.0),
            )],
        ),
        (
            "this.price := 3",
            vec![value_assign(member(Expr::This, "price"), Expr::Number(3.0))],
        ),
        (
            "$Order.taken := 4",
            vec![value_assign(
                member(Expr::ClassRef("Order".into()), "taken"),
                Expr::Number(4.0),
            )],
        ),
        (
            "x := 1; y := 2",
            vec![
                value_assign(name("x"), Expr::Number(1.0)),
                value_assign(name("y"), Expr::Number(2.0)),
            ],
        ),
        (
            "x := y = 3",
            vec![value_assign(
                name("x"),
                Expr::Assign {
                    target: Box::new(name("y")),
                    value: Box::new(Expr::Number(3.0)),
                },
            )],
        ),
        (
            "lambda := 1",
            vec![value_assign(name("lambda"), Expr::Number(1.0))],
        ),
        (
            "assert := 1",
            vec![value_assign(name("assert"), Expr::Number(1.0))],
        ),
        (
            "suppose := 1",
            vec![value_assign(name("suppose"), Expr::Number(1.0))],
        ),
        (
            "x := suppose",
            vec![value_assign(name("x"), name("suppose"))],
        ),
    ] {
        assert_eq!(parse(source).unwrap(), expected, "{source}");
    }

    let statements = parse("if ready: total := 1\nfor t of \"ab\": n := t\n{ x := 2 }").unwrap();
    let Stmt::If { consequent, .. } = &statements[0] else {
        panic!("{statements:?}");
    };
    assert_eq!(
        consequent,
        &vec![value_assign(name("total"), Expr::Number(1.0))]
    );
    let Stmt::For { body, .. } = &statements[1] else {
        panic!("{statements:?}");
    };
    assert_eq!(body, &vec![value_assign(name("n"), name("t"))]);
    assert_eq!(
        statements[2],
        Stmt::Block(vec![value_assign(name("x"), Expr::Number(2.0))])
    );
    assert_eq!(parse_expression("x := 1").unwrap(), name("x"));
    assert_eq!(
        parse_expression("x + 1 := 2").unwrap(),
        parse_expression("x + 1").unwrap()
    );
}

#[test]
fn canonical_rendering_keeps_the_operator_and_reads_back() {
    for (source, rendered) in [
        ("x := a + 1", "x:=a+1"),
        ("box.size := box.width * 2", "box.size:=box.width*2"),
        ("x := c ? 1 : 2", "x:=(c?1:2)"),
        ("x := y = 3", "x:=y=3"),
    ] {
        let statements = parse(source).unwrap();
        assert_eq!(generate(&statements[0]), rendered, "{source}");
        assert_eq!(parse(rendered).unwrap(), statements, "{source}");
    }
    let assigned = parse("{ total = a + b }").unwrap();
    let valued = parse("{ total := a + b }").unwrap();
    let (Stmt::Block(assigned), Stmt::Block(valued)) = (&assigned[0], &valued[0]) else {
        panic!("blocks");
    };
    assert_eq!(generate_all(assigned), "total=a+b");
    assert_eq!(generate_all(valued), "total:=a+b");
    assert_ne!(NodeKey::block(assigned, None), NodeKey::block(valued, None));

    let source = "def award(points = 1):\n    granted := points * rate\n    return granted";
    let statements = parse(source).unwrap();
    let rendered = generate(&statements[0]);
    assert_eq!(
        rendered,
        "def award(points=1):\n    granted:=points*rate\n    return granted"
    );
    assert_eq!(parse(&rendered).unwrap(), statements);
    assert_eq!(evaluate(&format!("rate = 2\n{source}\naward()")), json!(2));
}

#[test]
fn invalid_targets_are_syntax_errors_at_the_operator() {
    let long_path = format!("x{} := 1", ".p".repeat(129));
    let grouped_path = format!("(x{}){} := 1", ".p".repeat(100), ".p".repeat(29));
    let operator = grouped_path.find(":=").unwrap() + 1;
    for (source, message, line, column) in [
        ("xs[0] := 1", "Invalid value assignment target", 1, 7),
        ("xs[0:1] := 1", "Invalid value assignment target", 1, 9),
        ("f() := 1", "Invalid value assignment target", 1, 5),
        ("this := 1", "Invalid value assignment target", 1, 6),
        ("$Order := 1", "Invalid value assignment target", 1, 8),
        ("(a + b) := 1", "Invalid value assignment target", 1, 9),
        ("y = x := 1", "Invalid value assignment target", 1, 7),
        ("x += 1 := 2", "Invalid value assignment target", 1, 8),
        ("f = lambda:=1", "Invalid value assignment target", 1, 11),
        ("\"text\" := 1", "Invalid value assignment target", 1, 8),
        (
            "x.value.y := 1\nx := [y := 1]",
            "Expected ']' but found ':='",
            2,
            9,
        ),
        (long_path.as_str(), "Expression has too many parts", 1, 258),
        (
            grouped_path.as_str(),
            "Expression has too many parts",
            1,
            operator,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    let mut runtime = Runtime::new();
    runtime.run("x = 1").unwrap();
    assert_eq!(
        failure(&mut runtime, "x@previous := 2"),
        (
            ErrorKind::Type,
            "State versions are read-only".to_string(),
            Some(Position::new(1, 1))
        )
    );
    assert_eq!(read(&mut runtime, "x"), json!(1));
}

#[test]
fn expression_positions_keep_existing_diagnostics_naming_the_operator() {
    for (source, message, line, column) in [
        ("x = 1\ny = [x := 2]", "Expected ']' but found ':='", 2, 8),
        (
            "def f(a) => a\nf(x := 1)",
            "Expected ')' but found ':='",
            2,
            5,
        ),
        ("o = {a:=1}", "Expected ':' but found ':='", 1, 7),
        ("x = 1\nif x:=2: pass", "Expected ':' but found ':='", 2, 5),
        (
            "xs = [1, 2]\nok = all v in xs:=v",
            "Expected ':' but found ':='",
            2,
            17,
        ),
        (
            "xs = [1,2,3]\nxs[1:=2]",
            "Expected ']' but found ':='",
            2,
            5,
        ),
        ("def f():\n    return x := 1", "Unexpected ':='", 2, 17),
        ("x := y := 1", "Unexpected ':='", 1, 11),
        ("x = 2\nx ?:= 3", "Unexpected ':='", 2, 7),
        ("x :=", "Unexpected end of input", 1, 5),
        ("x :=\ny = 1", "Unexpected end of line", 2, 1),
        ("x := 1 + ", "Unexpected end of input", 1, 10),
        ("x = 1\nx :== 2", "Unexpected '='", 2, 7),
        ("x = 1\nx :=> 2", "Unexpected '>'", 2, 7),
        ("class A := 1", "Expected ':' but found ':='", 1, 9),
        ("x := suppose: 1", "Unexpected ':'", 1, 15),
        ("suppose x := 1: x", "Unexpected ':'", 1, 17),
        ("x: =1", "Unexpected ':'", 1, 4),
        (
            "def f x := 1",
            "Expected '(' but found identifier 'x'",
            1,
            7,
        ),
        (
            "for t := \"ab\": pass",
            "Expected 'of' in for statement",
            1,
            7,
        ),
        (
            "x = 1\nif x > 5:\n    pass\nelse x := 1:\n    pass",
            "Expected ':' but found identifier 'x'",
            4,
            6,
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
fn colon_readings_and_template_trailing_tokens_are_unchanged() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "x = 1\nt = `${x:=2}`\nassert: t, \"1\"\nclass Meter:\n    reading: int\n    def init(reading): this.reading = reading\nm = Meter(4)\nassert: m.reading, 4\nhalf = lambda value: value / 2\nassert: half(8), 4\nassert: [1, 2, 3][1:3], [2, 3]\nassert: {a: 1, b: 2}.b, 2\nassert: (x > 0 ? \"pos\" : \"neg\"), \"pos\"\nassert: (all v in [1, 2]: v > 0), true\nif x: y := 5\nassert: y, 5",
    );
    assert_eq!(
        failure(&mut Runtime::new(), "a: int"),
        (
            ErrorKind::Reference,
            "Missing definition".to_string(),
            Some(Position::new(1, 1))
        )
    );
    assert_eq!(evaluate("lambda := 3\nlambda"), json!(3));
    assert_eq!(evaluate("assert := 2\nassert"), json!(2));
    assert_eq!(evaluate("suppose = 4\nx := suppose\nx"), json!(4));
}

#[test]
fn value_assignments_file_stated_literals_without_dependencies() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = 2\nb = 3\nlabel = \"n\"\ny := a * b\nassert: y, 6\nassert: (why y)[0].rule, \"y = 6\"\nassert: (why y)[0].state, \"stated\"\nassert: (why y)[0].from, []\nassert: (affects a).length, 0\na = 10\nassert: y, 6\nassert: y.value, 6\nnames := [label, label]\nassert: (why names)[0].rule, \"names = [\\\"n\\\",\\\"n\\\"]\"",
    );
    let key = NodeKey::variable("y");
    assert_eq!(node(&runtime, &key).kind, NodeKind::Variable);
    assert!(node(&runtime, &key).dependencies.is_empty());
    assert_eq!(runtime.graph.shape_dependencies(&key).count(), 0);
    assert_eq!(stored(&runtime, &key), Expr::Number(6.0));
    assert!(
        node(&runtime, &NodeKey::variable("a"))
            .dependents
            .is_empty()
    );

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "rate = 2\ndef fee(amount) => amount * rate\nclass Item(price):\n    this.price = price\n    def taxed() => this.price * rate\npen = Item(3)\nlive = pen.price\nviews := (why live).length\ncallbacks := Item.map(item -> item.taxed()).length\ncharged := fee(10) + pen.taxed()\nassert: [charged, views, callbacks], [26, 3, 1]\nrate = 5\npen.price = 7\nItem(1)\nlive = pen.price * 2 + rate\nassert: (why live).length, 4\nassert: [charged, views, callbacks], [26, 3, 1]",
    );
    for name in ["views", "callbacks", "charged"] {
        let key = NodeKey::variable(name);
        assert!(node(&runtime, &key).dependencies.is_empty(), "{name}");
        assert_eq!(runtime.graph.shape_dependencies(&key).count(), 0, "{name}");
    }
}

#[test]
fn value_assignments_store_what_an_assignment_would_store_now() {
    assert_eq!(
        evaluate(
            "n = null\nsettled := n + 1\nplain = n + 1\nfrozen = (n + 1).value\nsupposed = suppose: n + 1\n[settled, plain, frozen, supposed]"
        ),
        json!([null, null, 1, 1])
    );
    assert_eq!(
        evaluate("box = {width: 2}\nmissing := box.height\nmissing"),
        json!(null)
    );
    assert_eq!(
        evaluate("xs = [1, 2]\nys := xs\nxs.push(3)\n[ys, xs]"),
        json!([[1, 2], [1, 2, 3]])
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Item(n): this.n = n\npen = Item(1)\npicked := Item.find(item -> item.n == 1)\ncopied = suppose: Item.find(item -> item.n == 1)\npicked.n = 5\nassert: pen.n, 5\ncopied.n = 9\nassert: pen.n, 5\nassert: picked == pen, true\nassert: copied == pen, false\nwhen := Date(0)\nassert: when.getTime(), 0\ntwice := x -> x * 2\nassert: twice(4), 8\nkind := Item\nassert: kind.length, 1",
    );
}

#[test]
fn value_assignments_keep_effects_that_suppose_discards() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "count = 0\ndef bump():\n    count := count + 1\n    return count\nkept := bump()\nassert: kept, 1\nassert: count, 1\ndiscarded = suppose: bump()\nassert: discarded, 2\nassert: count, 1\nclass Order(qty): this.qty = qty\nfirst := Order(2)\nassert: Order.length, 1\nghost = suppose: Order(3)\nassert: Order.length, 1\nassert: ghost.qty, 3\nassert: first.id, \"first\"",
    );
    assert_eq!(
        read(&mut runtime, "Order.list().map(order -> order.qty)"),
        json!([2])
    );
}

#[test]
fn instantiation_names_the_instance_after_the_target() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Item(n): this.n = n\nclass Order(qty): this.qty = qty\nclass Box\nq = 3\nx := Order(q)\nq = 9\nassert: x.qty, 3\nassert: x.id, \"x\"\nholder = Box()\nholder.item := Item(q)\nq = 1\nassert: holder.item.n, 9\nassert: holder.item.id, \"holder.item\"\nassert: Item.length, 1",
    );
    assert_eq!(read(&mut runtime, "(why x)[0].state"), json!("stated"));
    assert!(
        node(&runtime, &NodeKey::variable("x"))
            .dependencies
            .is_empty()
    );
}

#[test]
fn imperative_contexts_carry_value_assignments_out_once() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "base = 1\nx = base * 2\nfor t of \"a\": x := 7\nassert: x, 7\nbase = 5\nassert: x, 10\nfor t of \"ab\": last := t\nassert: last, \"b\"",
    );
    assert!(!runtime.graph.contains(&NodeKey::variable("last")));

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "paid = false\nstock = 10\nqty = 3\nif paid:\n    stock := stock - qty\n    for unit of \"ab\": served := unit\npaid = true\nassert: stock, 7\nassert: served, \"b\"\nqty = 4\nstock = 20\nassert: stock, 20\npaid = false\npaid = true\nassert: stock, 16",
    );
    let condition = parse_expression("paid").unwrap();
    let rule = node(&runtime, &NodeKey::conditional(&condition, None));
    assert_eq!(
        rule.dependencies
            .iter()
            .map(|key| key.as_str().to_string())
            .collect::<Vec<_>>(),
        vec!["paid".to_string()]
    );

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = 1\nb = 2\n{\n    sum := a + b\n    double = sum * 2\n}\na = 10\nassert: double, 6\nassert: sum, 3",
    );
}

#[test]
fn target_paths_are_located_without_dependency() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Order(qty): this.qty = qty\nfirst = Order(1)\nsecond = Order(2)\norder = first\npaid = true\nif paid: order.status := \"shipped\"\norder = second\nassert: first.status, \"shipped\"\nassert: second.status == \"shipped\", false\nfor each of Order: each.checked := true\nOrder(3)\nassert: Order.list().map(o -> o.checked == true), [true, true, false]",
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Order(qty): this.qty = qty\nfirst = Order(1)\nsecond = Order(2)\norder = first\npaid = true\nif paid: order.status = \"shipped\"\norder = second\nassert: [first.status, second.status], [\"shipped\", \"shipped\"]",
    );
}

#[test]
fn functions_methods_and_constructors_use_value_assignments() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "bonus = 5\ndef award(points):\n    points := points * bonus\n    return points\nprize = award(2)\nbonus = 7\nassert: prize, 10\nassert: award(2), 14\ncatalog = {price: 4}\nclass Order(qty):\n    this.qty = qty\n    this.price := catalog.price\n    def reprice(): this.price := catalog.price; return this.price\norder = Order(2)\ncatalog.price = 6\nassert: order.price, 4\nassert: order.reprice(), 6\ncatalog.price = 8\nassert: order.price, 6",
    );
    assert!(
        node(&runtime, &NodeKey::variable("prize"))
            .dependencies
            .iter()
            .all(|key| key.as_str() != "bonus")
    );
    let price = NodeKey::property(&ObjectId::named("order"), "price");
    assert_eq!(node(&runtime, &price).kind, NodeKind::Property);
    assert!(node(&runtime, &price).dependencies.is_empty());
    assert_eq!(stored(&runtime, &price), Expr::Number(6.0));
}

#[test]
fn class_level_rules_apply_value_assignments_per_instance() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "stock = 10\nclass Order(qty, paid):\n    this.qty = qty\n    this.paid = paid\nif $Order.paid:\n    $Order.taken := $Order.qty\n    stock := stock - $Order.qty\nsmall = Order(2, true)\nlarge = Order(5, false)\nassert: stock, 8\nlarge.paid = true\nassert: stock, 3\nlarge.qty = 6\nassert: large.taken, 5\nassert: stock, 3\ncount := $Order.length\nassert: count, 2\nready = true\nif ready: $Order.double := $Order.qty * 2\nassert: [small.double, large.double], [4, 12]\nlarge.qty = 1\nassert: large.double, 12\nOrder(7, false)\nassert: Order.list().map(order -> order.double), [4, 12, 14]",
    );
    for (source, kind, message) in [
        (
            "$Order.status := \"new\"",
            ErrorKind::Type,
            "Cannot use ':=' on a class-level property",
        ),
        (
            "Order.status := \"new\"",
            ErrorKind::Type,
            "Cannot use ':=' on a class-level property",
        ),
        (
            "$Missing.status := 1",
            ErrorKind::Reference,
            "Missing is not defined",
        ),
    ] {
        assert_eq!(
            failure(&mut runtime, source),
            (kind, message.to_string(), Some(Position::new(1, 1))),
            "{source}"
        );
    }
    assert_eq!(read(&mut runtime, "small.status"), json!(null));
    assert_eq!(
        read(
            &mut runtime,
            "failure = \"\"\ntry:\n    $Order.status := 1\ncatch error:\n    failure = error\nfailure"
        ),
        json!("TypeError: Cannot use ':=' on a class-level property")
    );
}

#[test]
fn failed_value_assignments_roll_back() {
    let mut runtime = Runtime::new();
    runtime
        .run("def checked(amount):\n    if amount > 100: throw \"TOO_LARGE\"\n    return amount\nstock = 10\nlow = stock < 20\nif stock > 500: throw \"LIMIT\"")
        .unwrap();
    for (source, kind, message) in [
        ("stock := checked(400)", ErrorKind::Thrown, "TOO_LARGE"),
        ("stock := 900", ErrorKind::Thrown, "LIMIT"),
        (
            "stock := missing",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "missing.total := 1",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "stock.total := 1",
            ErrorKind::Type,
            "Cannot assign 'total' on number",
        ),
        (
            "box = {}\nbox.value := 1",
            ErrorKind::Type,
            "Cannot use 'value' as a property",
        ),
        (
            "box = {}\nbox.inner.total := 1",
            ErrorKind::Reference,
            "box.inner is not defined",
        ),
    ] {
        let (found, text, _) = failure(&mut runtime, source);
        assert_eq!((found, text.as_str()), (kind, message), "{source}");
        assert_eq!(
            read(&mut runtime, "[stock, low]"),
            json!([10, true]),
            "{source}"
        );
        assert!(
            node(&runtime, &NodeKey::variable("stock"))
                .dependencies
                .is_empty()
        );
    }
    assert_eq!(read(&mut runtime, "stock := checked(40)"), json!(40));
    assert_eq!(read(&mut runtime, "low"), json!(false));
}

#[test]
fn deep_literals_raise_the_expression_depth_error_instead_of_overflowing() {
    let wrapped = |count: usize| format!("a = 0\nfor c of \"{}\": a = [a]\n", "c".repeat(count));
    let mut runtime = Runtime::new();
    runtime.run(&wrapped(191)).unwrap();
    assert_eq!(read(&mut runtime, "x := a\n(why x).length"), json!(1));
    assert_eq!(
        read(
            &mut runtime,
            "holder = {items: 1}\nholder.items := a\n(why holder.items).length"
        ),
        json!(1)
    );

    for count in [192, 400] {
        let mut runtime = Runtime::new();
        runtime.run(&wrapped(count)).unwrap();
        runtime.run("x = 1\nholder = {items: 1}").unwrap();
        assert_eq!(
            failure(&mut runtime, "y = 2\nx := a"),
            (
                ErrorKind::Type,
                "Maximum expression depth exceeded".to_string(),
                Some(Position::new(2, 1))
            ),
            "{count}"
        );
        assert_eq!(read(&mut runtime, "x"), json!(1), "{count}");
        assert!(!runtime.graph.contains(&NodeKey::variable("y")), "{count}");
        assert_eq!(
            failure(&mut runtime, "holder.items := a"),
            (
                ErrorKind::Type,
                "Maximum expression depth exceeded".to_string(),
                Some(Position::new(1, 1))
            ),
            "{count}"
        );
        assert_eq!(read(&mut runtime, "holder.items"), json!(1), "{count}");
        assert_eq!(
            read(&mut runtime, "for t of \"a\": z := a\nz.length"),
            json!(1),
            "{count}"
        );
        assert_eq!(read(&mut runtime, "w = a\nw.length"), json!(1), "{count}");
    }
}

#[test]
fn suppose_blocks_discard_value_assignments_made_inside_them() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "stock = 10\nqty = 3\nr = suppose:\n    stock := stock - qty\n    stock\nassert: r, 7\nassert: stock, 10\nsuppose qty = 5: stock := stock - qty; assert: stock, 5\nassert: stock, 10",
    );
}

#[test]
fn version_history_records_each_value_assignment() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "stock = 10\nqty = 3\nstock := stock - qty\nstock := stock - qty\nstock := stock - 0\nassert: stock@0, 10\nassert: stock@1, 7\nassert: stock@previous, 7\nassert: stock@last, 4\ndelete stock\nstock := 1\nassert: stock@first, 1",
    );
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
fn the_native_repl_runs_value_assignments_at_once() {
    let output =
        repl("stock = 10\nqty = 3\nstock := stock - qty\nqty = 5\nstock\nif stock := 1\nstock\n");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{BANNER}10.0\n3.0\n7.0\n5.0\n7.0\n7.0\n")
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "SyntaxError: Expected ':' but found ':=' (line 1, column 10)\n"
    );
}
