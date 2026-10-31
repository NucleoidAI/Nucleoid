mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{
    BinaryOp, Expr, Quantifier, Stmt, TemplatePart, parse, parse_expression,
};
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};
use serde_json::json;

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "quantifier regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
}

fn run(runtime: &mut Runtime, source: &str) -> Value {
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
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

fn json_of(runtime: &Runtime, value: &Value) -> serde_json::Value {
    integral(serde_json::from_str(&runtime.serialize_json(value).unwrap()).unwrap())
}

fn anonymous(keys: Vec<String>) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    keys.into_iter()
        .map(|key| match key.strip_prefix("obj:") {
            Some(tail) => {
                let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
                let rank = match seen.iter().position(|known| *known == digits) {
                    Some(rank) => rank,
                    None => {
                        seen.push(digits.clone());
                        seen.len() - 1
                    }
                };
                format!("obj:#{rank}{}", &tail[digits.len()..])
            }
            None => key,
        })
        .collect()
}

fn read(runtime: &mut Runtime, source: &str) -> serde_json::Value {
    let value = run(runtime, source);
    json_of(runtime, &value)
}

fn evaluate(source: &str) -> serde_json::Value {
    read(&mut Runtime::new(), source)
}

fn unnamed(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(text) if text.starts_with("obj:") => json!("obj:#"),
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(unnamed).collect())
        }
        serde_json::Value::Object(fields) => serde_json::Value::Object(
            fields
                .into_iter()
                .map(|(key, value)| (key, unnamed(value)))
                .collect(),
        ),
        other => other,
    }
}

fn outcome(runtime: &mut Runtime, source: &str) -> Result<serde_json::Value, String> {
    match runtime.run(source) {
        Ok(value) => Ok(unnamed(json_of(runtime, &value))),
        Err(error) => Err(error.to_string()),
    }
}

fn failure(source: &str) -> (ErrorKind, String, Option<Position>) {
    let error = Runtime::new().run(source).unwrap_err();
    (error.kind(), error.message(), error.position())
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn equivalent(source: &str, expected: &str) -> Vec<Stmt> {
    let actual = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    let wanted = parse(expected).unwrap_or_else(|error| panic!("{expected}: {error}"));
    assert_eq!(actual, wanted, "{source}");
    assert_eq!(generate_all(&actual), generate_all(&wanted), "{source}");
    actual
}

fn dependencies(runtime: &Runtime, key: &NodeKey) -> Vec<String> {
    runtime
        .graph
        .retrieve(key)
        .unwrap_or_else(|| panic!("{key:?} is filed"))
        .dependencies
        .iter()
        .map(|key| key.as_str().to_string())
        .collect()
}

fn shapes(runtime: &Runtime, key: &NodeKey) -> Vec<String> {
    runtime
        .graph
        .shape_dependencies(key)
        .map(ToString::to_string)
        .collect()
}

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn quantified(quantifier: Quantifier, binder: &str, collection: Expr, condition: Expr) -> Expr {
    Expr::Quantified {
        quantifier,
        binder: binder.to_string(),
        collection: Box::new(collection),
        condition: Box::new(condition),
    }
}

fn mentions_quantifier(source: &str) -> bool {
    let tokens: Vec<Token> = tokenize(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    tokens
        .windows(3)
        .any(|window| Quantifier::starting(&window[0], &window[1], &window[2]).is_some())
}

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid quantifies over class instances with all and any",
    "Nucleoid quantifies over lists with all and any",
    "Nucleoid quantifies over reasoning selections",
    "Nucleoid nests quantifiers over related instances",
    "Nucleoid applies quantifiers in class level rules",
    "Nucleoid uses quantifiers in functions callbacks and conditionals",
    "Nucleoid keeps all and any as ordinary names beside quantifiers",
    "Nucleoid rolls back failed quantifier updates",
];

#[test]
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let readme = common::snippets(include_str!("../README.md"), "README.md");
    let examples = common::snippets(include_str!("../docs/examples.md"), "docs/examples.md");
    let appended = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (&reference, "Nucleoid states live facts with all and any"),
        (&readme, "Quantifiers keep claims about every instance live"),
        (
            &examples,
            "Quantifiers keep zone health true to every sensor",
        ),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(mentions_quantifier(&case.source), "{title} quantifies");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 77);
}

#[test]
fn quantifiers_build_one_node_with_a_grouped_canonical_rendering() {
    let expected = quantified(
        Quantifier::All,
        "v",
        name("xs"),
        Expr::Binary {
            operator: BinaryOp::Greater,
            left: Box::new(name("v")),
            right: Box::new(Expr::Number(0.0)),
        },
    );
    assert_eq!(parse_expression("all v in xs: v > 0").unwrap(), expected);
    assert_eq!(parse_expression("all v in xs:v>0").unwrap(), expected);
    assert_eq!(expected.to_string(), "(all v in xs:v>0)");
    assert_eq!(
        parse_expression("any w in Worker: w.ok").unwrap(),
        quantified(
            Quantifier::Any,
            "w",
            name("Worker"),
            Expr::Member {
                object: Box::new(name("w")),
                property: "ok".to_string(),
            },
        )
    );

    for (source, canonical) in [
        ("ok = all v in xs: v > 0", "ok=(all v in xs:v>0)"),
        ("ok = any w in Worker: w.ok", "ok=(any w in Worker:w.ok)"),
        (
            "covered = all t in Task: any w in Worker: w.skill == t.skill",
            "covered=(all t in Task:(any w in Worker:w.skill==t.skill))",
        ),
        (
            "ok = all s in why c: s.holds > 0",
            "ok=(all s in c|>why:s.holds>0)",
        ),
        (
            "ok = all s in c |> why |> stated: s.holds > 0",
            "ok=(all s in c|>why|>stated:s.holds>0)",
        ),
        (
            "ok = (all v in xs: v > 0) and ready",
            "ok=(all v in xs:v>0)&&ready",
        ),
        (
            "ok = all v in xs: v > 0 and ready",
            "ok=(all v in xs:v>0&&ready)",
        ),
        ("ok = all v in c ? a : b: v", "ok=(all v in (c?a:b):v)"),
        ("ok = all v in xs: v ?? 0", "ok=(all v in xs:(v??0))"),
        (
            "if all v in xs: v > 0:\n    pass",
            "if((all v in xs:v>0)){pass}",
        ),
        (
            "$Order.complete = all line in $Order.lines: line.sent",
            "$Order.complete=(all line in $Order.lines:line.sent)",
        ),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), canonical, "{source}");
    }

    for source in [
        "all v in xs: v > 0",
        "any w in Worker: w.skill == t.skill",
        "all t in Task: any w in Worker: w.skill == t.skill",
        "all s in c |> why |> stated: s.holds > 0",
        "all v in c ? a : b: v ? 1 : 2",
        "all v in a ?? b: v ?? 0",
        "(all v in xs: v) and (any w in ws: w)",
        "all row in grid: all v in row: v > row.length",
    ] {
        let expression = parse_expression(source).unwrap();
        assert_eq!(
            parse_expression(&expression.to_string()).unwrap(),
            expression,
            "{source}"
        );
    }
}

#[test]
fn quantifiers_extend_as_far_right_as_possible() {
    for (source, grouped) in [
        (
            "x = ready and all v in xs: v > 0",
            "x = ready and (all v in xs: v > 0)",
        ),
        ("x = not all v in xs: v > 0", "x = not (all v in xs: v > 0)"),
        (
            "x = all v in xs: v > 0 and ready",
            "x = (all v in xs: (v > 0 and ready))",
        ),
        (
            "x = all v in xs: v > 0 ? \"a\" : \"b\"",
            "x = (all v in xs: (v > 0 ? \"a\" : \"b\"))",
        ),
        (
            "x = all v in xs: v if ready else 0",
            "x = (all v in xs: (ready ? v : 0))",
        ),
        ("x = all v in xs: v ?? 0", "x = (all v in xs: (v ?? 0))"),
        ("x = all v in xs: v |> why", "x = (all v in xs: (v |> why))"),
        (
            "x = (all v in xs: v > 0) ? \"clean\" : \"dirty\"",
            "x = (all v in xs: v > 0) ? \"clean\" : \"dirty\"",
        ),
        (
            "x = \"clean\" if all v in xs: v > 0 else \"dirty\"",
            "x = (all v in xs: v > 0) ? \"clean\" : \"dirty\"",
        ),
        ("x = a + all v in xs: v", "x = a + (all v in xs: v)"),
        ("x = all v in a or b: v", "x = (all v in (a or b): v)"),
        ("x = all v in c ? a : b: v", "x = (all v in (c ? a : b): v)"),
        ("x = all v in a ?? b: v", "x = (all v in (a ?? b): v)"),
        (
            "x = all v in a if c else b: v",
            "x = (all v in (c ? a : b): v)",
        ),
        (
            "x = all s in c |> why |> stated: s",
            "x = (all s in (c |> why |> stated): s)",
        ),
        ("x = all s in why c: s", "x = (all s in (c |> why): s)"),
        (
            "x = all v in any w in ws: w: v",
            "x = (all v in (any w in ws: w): v)",
        ),
        ("f = v => all w in v: w", "f = v => (all w in v: w)"),
        ("f = v -> any w in v: w", "f = v => (any w in v: w)"),
        ("f = lambda v: all w in v: w", "f = (v) => (all w in v: w)"),
        (
            "def f(v) => all w in v: w > 0",
            "def f(v):\n    return all w in v: w > 0",
        ),
        (
            "r = {ok: all v in xs: v > 0, n: 1}",
            "r = {ok: (all v in xs: v > 0), n: 1}",
        ),
        (
            "r = {ok: all v in xs: v > 0 n: 1}",
            "r = {ok: (all v in xs: v > 0), n: 1}",
        ),
        ("r = [all v in xs: v, 1]", "r = [(all v in xs: v), 1]"),
        ("r = f(all v in xs: v, 2)", "r = f((all v in xs: v), 2)"),
        ("r = flags[all v in xs: v]", "r = flags[(all v in xs: v)]"),
        ("r = items[all v in xs: v:]", "r = items[(all v in xs: v):]"),
        ("r = items[:any v in xs: v]", "r = items[:(any v in xs: v)]"),
        (
            "if all v in xs: v ?? false:\n    ok = 1",
            "if (all v in xs: (v ?? false)):\n    ok = 1",
        ),
        (
            "r = items[0 if all v in xs: v else 1:]",
            "r = items[((all v in xs: v) ? 0 : 1):]",
        ),
        (
            "assert: all v in xs: v > 0, true",
            "assert((all v in xs: v > 0), true)",
        ),
        ("throw all v in xs: v", "throw (all v in xs: v)"),
        (
            "if all v in xs: v > 0:\n    ok = 1",
            "if (all v in xs: v > 0):\n    ok = 1",
        ),
        (
            "for v in all w in ws: w:\n    pass",
            "for v in (all w in ws: w):\n    pass",
        ),
        ("x = (all v in xs:\n    v > 0)", "x = (all v in xs: v > 0)"),
    ] {
        equivalent(source, grouped);
    }
}

#[test]
fn template_interpolations_keep_their_accepted_reading_of_a_bare_quantifier() {
    equivalent("t = `${all v in xs: v}`", "t = `${all}`");
    equivalent("t = `${x => any v in xs: v}`", "t = `${x => any}`");
    equivalent("t = `${c ? 1 : all v in xs: v}`", "t = `${c ? 1 : all}`");
    let statements = parse("t = `${(all v in xs: v > 1)}`").unwrap();
    let [Stmt::Assign { value, .. }] = statements.as_slice() else {
        panic!("one assignment");
    };
    let Expr::Template(parts) = value else {
        panic!("a template");
    };
    assert!(matches!(
        &parts[0],
        TemplatePart::Expression(Expr::Quantified { .. })
    ));

    let mut runtime = Runtime::new();
    run(&mut runtime, "all = 5\nxs = [1, 2]\ndef f(value) => value");
    for (source, expected) in [
        ("`${all v in xs: v > 1}`", "5"),
        ("`${(all v in xs: v > 1)}`", "false"),
        ("`${(any v in xs: v > 1)}`", "true"),
        ("`${f(all v in xs: v > 0)}`", "true"),
    ] {
        assert_eq!(read(&mut runtime, source), json!(expected), "{source}");
    }
}

#[test]
fn ordinary_names_keep_every_accepted_reading() {
    for (source, expected) in [
        (
            "all = [1, 2]\nany = 3\n[all, any, all.length + any]",
            json!([[1, 2], 3, 5]),
        ),
        (
            "def all(xs) => xs.length\ndef any(xs) => xs.length > 0\n[all([1, 2]), any([])]",
            json!([2, false]),
        ),
        (
            "class Member(name):\n    this.name = name\nada = Member(\"Ada\")\n[Member.all().length, Member.list().length]",
            json!([1, 1]),
        ),
        ("[1, 2, 3].any(x => x > 2)", json!(true)),
        ("o = {all: 1, any: 2}\n[o.all, o.any]", json!([1, 2])),
        (
            "all = 2\nb = 3\no = {a: all b: b}\n[o.a, o.b]",
            json!([2, 3]),
        ),
        ("c = 1\nall = 2\nx = all c\nx", json!(2)),
        (
            "f = all => all + 1\ng = (all, any) => all + any\n[f(1), g(1, 2)]",
            json!([2, 3]),
        ),
        ("def h all any => all * any\nh(2, 3)", json!(6)),
        ("def k(all) => all * 2\nk(4)", json!(8)),
        (
            "class Seen(size):\n    this.size = size\na = Seen(1)\nb = Seen(2)\nfor all in Seen:\n    all.doubled = all.size * 2\nfor any in \"ab\":\n    last = any\n[a.doubled, b.doubled, last]",
            json!([2, 4, "b"]),
        ),
        ("all = 5\nt = `${all x in xs}`\nt", json!("5")),
        (
            "class Box(all):\n    this.all = all\nbox = Box(5)\nbox.all",
            json!(5),
        ),
        ("all = true\nany = false\nall and not any", json!(true)),
        ("all = [3]\nall[0]", json!(3)),
        ("all = 7\nassert: all, 7\nall", json!(7)),
        ("all = null\nall ?? 9", json!(9)),
        (
            "all = 1\nany = 2\nr = {all, any}\n[r.all, r.any]",
            json!([1, 2]),
        ),
        ("all = [1, 2]\nall.every(v => v > 0)", json!(true)),
    ] {
        assert!(!mentions_quantifier(source), "{source}");
        assert_eq!(evaluate(source), expected, "{source}");
    }
    assert_eq!(
        parse("all: int").unwrap(),
        [Stmt::Declaration {
            name: "all".to_string(),
            type_name: "int".to_string(),
        }]
    );
    equivalent("x = all c", "x = all\nc");
    equivalent("x = any c", "x = any\nc");

    for (source, expected) in [
        (
            "all = [1, 2, 3]\nany = 2\nall n in all: n > any",
            json!(false),
        ),
        (
            "all = [1, 2, 3]\nany = 2\nany n in all: n > any",
            json!(true),
        ),
        ("all all in [1, 2]: all > 0", json!(true)),
        ("any any in [0, 2]: any", json!(true)),
    ] {
        assert!(mentions_quantifier(source), "{source}");
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn parse_expression_now_reads_a_quantifier_and_still_ignores_what_follows() {
    let expected = quantified(Quantifier::All, "x", name("xs"), name("x"));
    assert_eq!(parse_expression("all x in xs: x").unwrap(), expected);
    assert_eq!(parse_expression("all x in xs: x d").unwrap(), expected);
    assert_eq!(parse_expression("all x").unwrap(), name("all"));
    assert_eq!(parse_expression("all in xs").unwrap(), name("all"));
    assert_eq!(
        parse_expression("all(x) in xs").unwrap().to_string(),
        "all(x)"
    );
}

fn assert_parity(quantified: &str, spelled: &str, steps: &[&str], keys: &[NodeKey], probe: &str) {
    assert!(mentions_quantifier(quantified), "{quantified}");
    assert!(!mentions_quantifier(spelled), "{spelled}");
    let mut left = Runtime::new();
    let mut right = Runtime::new();
    assert_eq!(
        outcome(&mut left, quantified),
        outcome(&mut right, spelled),
        "{quantified}"
    );
    for step in std::iter::once(None).chain(steps.iter().map(Some)) {
        if let Some(step) = step {
            assert_eq!(
                outcome(&mut left, step),
                outcome(&mut right, step),
                "{step}"
            );
        }
        assert_eq!(
            unnamed(read(&mut left, probe)),
            unnamed(read(&mut right, probe)),
            "{step:?}"
        );
        for key in keys {
            assert_eq!(
                anonymous(dependencies(&left, key)),
                anonymous(dependencies(&right, key)),
                "{key:?} after {step:?}"
            );
            assert_eq!(
                anonymous(shapes(&left, key)),
                anonymous(shapes(&right, key)),
                "{key:?} after {step:?}"
            );
        }
    }
}

#[test]
fn quantifiers_evaluate_like_every_and_some() {
    let variables = |names: &[&str]| -> Vec<NodeKey> {
        names.iter().map(|name| NodeKey::variable(*name)).collect()
    };
    assert_parity(
        "numbers = [3, 5, 8]\nlimit = 2\nlarge = all n in numbers: n > limit\neven = any n in numbers: n % 2 == 0",
        "numbers = [3, 5, 8]\nlimit = 2\nlarge = numbers.every(n => n > limit)\neven = numbers.some(n => n % 2 == 0)",
        &[
            "limit = 4",
            "numbers = [5, 7]",
            "numbers.push(10)",
            "numbers = []",
            "numbers = null",
            "numbers = [null, 1]",
            "numbers = [1, null]",
            "limit = 0",
        ],
        &variables(&["large", "even"]),
        "[large, even]",
    );
    assert_parity(
        "class Check\nready = all c in Check: c.passed\nblocked = any c in Check: c.failed",
        "class Check\nready = Check.every(c => c.passed)\nblocked = Check.some(c => c.failed)",
        &[
            "first = Check()",
            "first.passed = true",
            "second = Check()",
            "second.passed = null",
            "second.passed = true",
            "second.failed = true",
            "delete second.failed",
            "third = Check()",
            "delete third",
        ],
        &variables(&["ready", "blocked"]),
        "[ready, blocked]",
    );
    assert_parity(
        "class Order\norder = Order()\nkind = Order\nfilled = all l in order.lines: l\nrushed = any o in kind: o.rush",
        "class Order\norder = Order()\nkind = Order\nfilled = order.lines.every(l => l)\nrushed = kind.some(o => o.rush)",
        &[
            "order.lines = [1, 2]",
            "order.lines = [1, 0]",
            "order.rush = true",
            "other = Order()",
        ],
        &variables(&["filled", "rushed"]),
        "[filled, rushed]",
    );
    assert_parity(
        "a = 1\nb = a + 2\nc = b * 2\npositive = all s in why c: s.holds > 0\ngrounded = all s in c |> why |> stated: s.holds > 0\nnamed = any s in why c: s.node == \"d\"",
        "a = 1\nb = a + 2\nc = b * 2\npositive = (why c).every(s => s.holds > 0)\ngrounded = (c |> why |> stated).every(s => s.holds > 0)\nnamed = (why c).some(s => s.node == \"d\")",
        &["d = -5", "b = a + d", "d = 5", "b = 2"],
        &variables(&["positive", "grounded", "named"]),
        "[positive, grounded, named]",
    );
    let order = ObjectId::named("order");
    assert_parity(
        "class Line(sent):\n    this.sent = sent\nclass Order(lines):\n    this.lines = lines\n$Order.complete = all line in $Order.lines: line.sent\n$Order.started = any line in $Order.lines: line.sent\nfirst = Line(true)\nsecond = Line(false)\norder = Order([first, second])",
        "class Line(sent):\n    this.sent = sent\nclass Order(lines):\n    this.lines = lines\n$Order.complete = $Order.lines.every(line => line.sent)\n$Order.started = $Order.lines.some(line => line.sent)\nfirst = Line(true)\nsecond = Line(false)\norder = Order([first, second])",
        &[
            "second.sent = true",
            "first.sent = false",
            "order.lines = []",
            "third = Line(true)",
            "order.lines = [third]",
        ],
        &[
            NodeKey::property(&order, "complete"),
            NodeKey::property(&order, "started"),
        ],
        "[order.complete, order.started]",
    );
    assert_parity(
        "def positive(values) => all v in values: v > 0\nscores = [2, 4]\nok = positive(scores)",
        "def positive(values) => values.every(v => v > 0)\nscores = [2, 4]\nok = positive(scores)",
        &["scores = [2, -1]", "scores = []"],
        &variables(&["ok"]),
        "ok",
    );
    assert_parity(
        "class Zone(name):\n    this.name = name\nclass Sensor(zone, ok):\n    this.zone = zone\n    this.ok = ok\nif all s in Sensor: s.zone != $Zone.name or s.ok:\n    $Zone.state = \"healthy\"\nelse:\n    $Zone.state = \"alert\"\nlab = Zone(\"lab\")\ndock = Zone(\"dock\")\ndoor = Sensor(\"dock\", false)",
        "class Zone(name):\n    this.name = name\nclass Sensor(zone, ok):\n    this.zone = zone\n    this.ok = ok\nif Sensor.every(s => s.zone != $Zone.name or s.ok):\n    $Zone.state = \"healthy\"\nelse:\n    $Zone.state = \"alert\"\nlab = Zone(\"lab\")\ndock = Zone(\"dock\")\ndoor = Sensor(\"dock\", false)",
        &[
            "door.ok = true",
            "Sensor(\"lab\", false)",
            "door.ok = false",
        ],
        &[],
        "[lab.state, dock.state]",
    );
    assert_parity(
        "limit = 1\nxs = [2, 3]\nok = all x in xs: x > limit.value\nsnapshot = all x in xs.value: x > 1",
        "limit = 1\nxs = [2, 3]\nok = xs.every(x => x > limit.value)\nsnapshot = xs.value.every(x => x > 1)",
        &["limit = 5", "xs = [3]", "xs = [9]", "limit = 0"],
        &variables(&["ok", "snapshot"]),
        "[ok, snapshot]",
    );
}

#[test]
fn quantifiers_stop_at_the_deciding_member_and_follow_only_visited_members() {
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "calls = []\ndef seen(v):\n    calls.push(v)\n    return v > 1\nxs = [3, 1, 5]",
    );
    for (source, result, calls) in [
        ("all v in xs: seen(v)", json!(false), json!([3, 1])),
        ("any v in xs: seen(v)", json!(true), json!([3])),
        ("all v in []: seen(v)", json!(true), json!([])),
        ("any v in []: seen(v)", json!(false), json!([])),
        ("any v in [1, 0]: seen(v)", json!(false), json!([1, 0])),
    ] {
        run(&mut runtime, "calls = []");
        assert_eq!(read(&mut runtime, source), result, "{source}");
        assert_eq!(read(&mut runtime, "calls"), calls, "{source}");
    }
    run(
        &mut runtime,
        "picks = []\ndef pick():\n    picks.push(1)\n    return [3, 1, 5]",
    );
    for (source, result) in [
        ("all v in pick(): v > 0", json!(true)),
        ("any v in pick(): v > 4", json!(true)),
        ("all v in pick(): v > 2", json!(false)),
    ] {
        run(&mut runtime, "picks = []");
        assert_eq!(read(&mut runtime, source), result, "{source}");
        assert_eq!(read(&mut runtime, "picks"), json!([1]), "{source}");
    }

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "class Flag(on):\n    this.on = on\na = Flag(true)\nb = Flag(false)\nc = Flag(true)\nevery_on = all f in Flag: f.on\nsome_on = any f in Flag: f.on\nassert(every_on, false)\nassert(some_on, true)",
    );
    assert_eq!(
        dependencies(&runtime, &NodeKey::variable("every_on")),
        ["$Flag", "a.on", "b.on"]
    );
    assert_eq!(
        dependencies(&runtime, &NodeKey::variable("some_on")),
        ["$Flag", "a.on"]
    );
    checked(
        &mut runtime,
        "b.on = true\nassert(every_on, true)\nc.on = false\nassert(every_on, false)\na.on = false\nassert(some_on, true)\nd = Flag(false)\nassert(some_on, true)",
    );
    assert_eq!(
        dependencies(&runtime, &NodeKey::variable("every_on")),
        ["$Flag", "a.on"]
    );
}

#[test]
fn null_collections_and_null_reads_settle_declarations_to_null() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "pending = null\nok = all x in pending: x > 0\nfound = any x in pending: x > 0\n[ok, found]",
            json!([null, null]),
        ),
        ("pending = [1, 2]\n[ok, found]", json!([true, true])),
        ("pending = [1, null]\n[ok, found]", json!([null, true])),
        ("pending = [null, 1]\n[ok, found]", json!([null, null])),
        ("all x in null: x", json!(null)),
        (
            "class C\nc1 = C()\nc1.passed = null\nready = all c in C: c.passed\nsome = any c in C: c.passed\n[ready, some]",
            json!([null, null]),
        ),
        ("c1.passed = true\n[ready, some]", json!([true, true])),
        (
            "class Order\norder = Order()\nfilled = all l in order.lines: l\nfilled",
            json!(null),
        ),
        ("order.lines = [1]\nfilled", json!(true)),
    ] {
        assert_eq!(read(&mut runtime, source), expected, "{source}");
    }
    assert_eq!(
        run(&mut runtime, "all x in null: x"),
        Value::Undefined,
        "an unstored null collection is undefined, as with every"
    );
}

#[test]
fn quantifiers_over_values_that_are_not_collections_raise_type_errors() {
    for (source, message, line) in [
        ("x = 5\nok = all v in x: v", "x is not a collection", 2),
        ("ok = all v in \"abc\": v", "\"abc\" is not a collection", 1),
        ("ok = any v in true: v", "true is not a collection", 1),
        ("o = {a: 1}\nok = all v in o: v", "o is not a collection", 2),
        (
            "class C\nc = C()\nok = all v in c: v",
            "c is not a collection",
            3,
        ),
        ("f = v => v\nok = any v in f: v", "f is not a collection", 2),
        ("ok = all v in Math: v", "Math is not a collection", 1),
        ("ok = all v in Class: v", "Class is not a collection", 1),
        ("ok = all v in 1 + 2: v", "1+2 is not a collection", 1),
        (
            "class Order\no = Order()\no.lines = 3\n$Order.ok = all l in $Order.lines: l",
            "Order.lines is not a collection",
            4,
        ),
    ] {
        assert_eq!(
            failure(source),
            (
                ErrorKind::Type,
                message.to_string(),
                Some(Position::new(line, 1))
            ),
            "{source}"
        );
    }
    assert_eq!(
        failure("x = 5\nok = x.every(v => v)").1,
        "number.every is not a function"
    );
    assert_eq!(
        evaluate("o = {a: 1}\nok = o.every(v => v)\nok"),
        json!(null)
    );
}

#[test]
fn binders_are_lexical_and_never_leak() {
    for (source, expected) in [
        (
            "xs = [1, 2]\nys = [10]\nm = xs.map(x => all y in ys: y > x)\nm",
            json!([true, true]),
        ),
        (
            "xs = [1, 2]\nys = [2, 3]\nok = all x in xs: any y in ys: y > x\nys = [0]\nok",
            json!(false),
        ),
        (
            "grid = [[1, 2], [3]]\nok = all row in grid: any v in row: v >= row.length\nok",
            json!(true),
        ),
        (
            "grid = [[1, 2], [3]]\nok = all row in grid: any v in row: v > row.length\nok",
            json!(false),
        ),
        (
            "class Team(min):\n    this.min = min\n    this.scores = [3, 4]\n    def ok() => all s in this.scores: s >= this.min\nteam = Team(3)\nteam.ok()",
            json!(true),
        ),
        (
            "def f(k) => all x in [1, 2]: x < k\n[f(2), f(3)]",
            json!([false, true]),
        ),
        (
            "def g():\n    k = 5\n    return all x in [1, 2]: x < k\ng()",
            json!(true),
        ),
        (
            "x = 100\nr = all x in [1, 2]: x < 5\n[r, x]",
            json!([true, 100]),
        ),
        (
            "v = 1\nr = all v in [5]: all v in [7]: v == 7\n[r, v]",
            json!([true, 1]),
        ),
        ("xs = [1, 2]\nok = all v in xs: v = 0\nxs", json!([1, 2])),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }

    let mut runtime = Runtime::new();
    run(&mut runtime, "xs = [1, 2]\nx = 100\nr = all x in xs: x < 5");
    assert_eq!(dependencies(&runtime, &NodeKey::variable("r")), ["xs"]);
    run(&mut runtime, "x = 0");
    assert_eq!(read(&mut runtime, "r"), json!(true));

    for (source, message) in [
        ("r = all x in [1]: x > 0\nx", "x is not defined"),
        (
            "ok = all x in [1, 2]: [5].some(y => y > x)",
            "x is not defined",
        ),
        (
            "def f(y) => y > x\nok = all x in [1]: f(x)",
            "x is not defined",
        ),
    ] {
        let (kind, actual, _) = failure(source);
        assert_eq!(
            (kind, actual.as_str()),
            (ErrorKind::Reference, message),
            "{source}"
        );
    }
    assert_eq!(
        evaluate("xs = [1, 2]\nall x in xs: any y in [5]: y > x"),
        json!(true)
    );
}

#[test]
fn if_and_alternative_headers_take_quantifier_conditions() {
    let mut runtime = Runtime::new();
    for (source, expected) in [
        (
            "xs = [1, 2]\nif all v in xs: v > 0:\n    state = \"positive\"\nelse:\n    state = \"mixed\"\nstate",
            "positive",
        ),
        ("xs = [1, -2]\nstate", "mixed"),
        (
            "ys = [1, -2]\nif all v in ys: v > 0\n    shape = \"positive\"\nelse\n    shape = \"mixed\"\nshape",
            "mixed",
        ),
        ("ys = [3]\nshape", "positive"),
        (
            "zs = [3]\nif false:\n    pick = \"none\"\nelse if any v in zs: v > 2:\n    pick = \"else if\"\nelse:\n    pick = \"other\"\npick",
            "else if",
        ),
        (
            "if false:\n    direct = \"none\"\nelse all v in zs: v > 2:\n    direct = \"direct\"\nelse:\n    direct = \"other\"\ndirect",
            "direct",
        ),
        (
            "if false:\n    bare = \"none\"\nelif all v in zs: v > 2\n    bare = \"elif\"\nelse\n    bare = \"other\"\nbare",
            "elif",
        ),
        (
            "if any v in zs: v > 2: same = \"same line\"\nsame",
            "same line",
        ),
        (
            "if all v in zs: v > 2 ? true : false:\n    pair = \"paired\"\npair",
            "paired",
        ),
    ] {
        assert_eq!(read(&mut runtime, source), json!(expected), "{source}");
    }
    run(
        &mut runtime,
        "class Zone(name):\n    this.name = name\nclass Sensor(zone, ok):\n    this.zone = zone\n    this.ok = ok\nif all s in Sensor: s.zone != $Zone.name or s.ok:\n    $Zone.state = \"healthy\"\nelse:\n    $Zone.state = \"alert\"\nlab = Zone(\"lab\")\ndock = Zone(\"dock\")\ndoor = Sensor(\"dock\", false)",
    );
    assert_eq!(
        read(&mut runtime, "[lab.state, dock.state]"),
        json!(["healthy", "alert"])
    );
    run(&mut runtime, "door.ok = true");
    assert_eq!(
        read(&mut runtime, "[lab.state, dock.state]"),
        json!(["healthy", "healthy"])
    );
    assert!(
        runtime
            .graph
            .retrieve(&NodeKey::conditional(
                &parse_expression("all v in xs: v > 0").unwrap(),
                None
            ))
            .is_some()
    );
}

#[test]
fn malformed_quantifiers_report_exact_errors() {
    for (source, message, line, column) in [
        (
            "ok = all v in xs v > 0",
            "Expected ':' but found identifier 'v'",
            1,
            18,
        ),
        (
            "ok = all v in xs",
            "Expected ':' but found end of input",
            1,
            17,
        ),
        (
            "ok = all v in xs\n",
            "Expected ':' but found end of line",
            1,
            15,
        ),
        ("ok = all v in xs:", "Unexpected end of input", 1, 18),
        ("ok = all v in [1]: ", "Unexpected end of input", 1, 20),
        ("ok = all v in : v", "Unexpected ':'", 1, 17),
        (
            "ok = all v in xs:\n    v > 1",
            "Unexpected end of line",
            2,
            5,
        ),
        (
            "ok = all v in xs: v > 0 if ready",
            "Expected ':' but found end of input",
            1,
            33,
        ),
        ("ok = all 1 in xs: v", "Unexpected keyword 'in'", 1, 15),
        ("ok = all (v) in xs: v", "Unexpected keyword 'in'", 1, 17),
        ("ok = all this in xs: 1", "Unexpected keyword 'in'", 1, 18),
        ("ok = all v of xs: v", "Unexpected keyword 'of'", 1, 15),
        ("ok = every v in xs: v", "Unexpected keyword 'in'", 1, 17),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    let nested = format!("ok = {}true", "all v in xs: ".repeat(200));
    assert_eq!(syntax_error(&nested).0, "Expressions are nested too deeply");
    assert_eq!(
        failure("ok = (all v in [1, 2]: v > 0) = 1"),
        (
            ErrorKind::Syntax,
            "Cannot assign to (all v in [1,2]:v>0)".to_string(),
            Some(Position::new(1, 1))
        )
    );
}

#[test]
fn class_level_rules_bind_the_quantifier_name_and_check_everything_else() {
    let mut runtime = Runtime::new();
    run(&mut runtime, "class Order\norder = Order()");
    for source in [
        "$Order.ok = all l in $Order.lines: l.sent and vip",
        "$Order.ok = all l in missing: l",
        "$Order.ok = any l in $Order.lines: other.ready",
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference, "{source}");
        assert!(error.message().ends_with(" is not defined"), "{source}");
    }
    run(
        &mut runtime,
        "$Order.ready = all l in $Order.lines: l.sent\n$Order.cap = all l in $Order.lines: l.size < $Order.limit",
    );
    for source in [
        "$Order.limit = any l in $Order.lines: l.ok and $Order.cap",
        "$Order.total = all l in $Order.lines: $Order.total",
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{source}");
        assert_eq!(error.message(), "Circular Dependency", "{source}");
    }
    checked(
        &mut runtime,
        "class Line(sent):\n    this.sent = sent\na = Line(true)\nb = Line(false)\norder.lines = [a]\nother = Order()\nother.lines = [a, b]\nassert(order.ready, true)\nassert(other.ready, false)\nb.sent = true\nassert(other.ready, true)",
    );
}

#[test]
fn failed_quantifier_updates_restore_state_and_graph() {
    let initial = r#"
def check(value):
    if value < 0:
        throw "NEGATIVE"
    return value > 0
xs = [1]
listed = all x in xs: check(x)
class Item(qty):
    this.qty = qty
item = Item(2)
valid = all i in Item: check(i.qty)
some = any i in Item: i.qty > 5
assert(listed, true)
assert(valid, true)
"#;
    let mut actual = Runtime::new();
    checked(&mut actual, initial);
    let mut control = Runtime::new();
    checked(&mut control, initial);
    for (source, thrown) in [
        ("xs = [1, -1]", "NEGATIVE"),
        ("item.qty = -1", "NEGATIVE"),
        ("other = Item(-3)", "NEGATIVE"),
        ("mid = Item(9)\nthrow \"ROLLBACK\"", "ROLLBACK"),
    ] {
        let error = actual.run(source).unwrap_err();
        assert_eq!(
            error.thrown_value(),
            Some(&Value::String(thrown.to_string())),
            "{source}"
        );
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    }
    let error = actual.run("broken = all x in missing: x").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    checked(
        &mut actual,
        "mid = Item(9)\nassert(some, true)\nitem.qty = 0\nassert(valid, false)",
    );
}

#[test]
fn quantifiers_over_selections_are_reasoning_observers() {
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "a = 1\nb = a + 2\nc = b * 2\npositive = all s in why c: s.holds > 0\nplain = all v in [a, b]: v > 0",
    );
    assert_eq!(
        read(&mut runtime, "(affects a).map(s => s.node)"),
        json!(["b", "plain", "c"])
    );
    assert_eq!(read(&mut runtime, "(why positive).length"), json!(0));
    assert_eq!(
        read(&mut runtime, "(why plain).map(s => s.node)"),
        json!(["plain", "a", "b"])
    );
    assert_eq!(
        read(&mut runtime, "(model |> stated).map(s => s.node)"),
        json!(["a"])
    );
    assert_eq!(
        read(&mut runtime, "(why plain)[0].rule"),
        json!("plain = (all v in [a,b]:v>0)")
    );
    assert_eq!(
        shapes(&runtime, &NodeKey::variable("positive")),
        ["c", "b", "a"]
    );
    run(&mut runtime, "d = -5\nb = a + d");
    assert_eq!(
        read(&mut runtime, "[positive, plain]"),
        json!([false, false])
    );
    assert_eq!(
        read(&mut runtime, "(why c).map(s => s.node)"),
        json!(["c", "b", "a", "d"])
    );
}

#[test]
fn why_and_affects_treat_quantifier_declarations_like_every_and_some() {
    let setup = "class Check\nc1 = Check()\nc1.passed = true\nc2 = Check()\nc2.passed = false\n";
    let mut quantified = Runtime::new();
    run(
        &mut quantified,
        &format!("{setup}ready = all c in Check: c.passed\nsome = any c in Check: c.passed"),
    );
    let mut spelled = Runtime::new();
    run(
        &mut spelled,
        &format!("{setup}ready = Check.every(c => c.passed)\nsome = Check.some(c => c.passed)"),
    );
    for probe in [
        "(why ready).map(s => [s.node, s.holds, s.state, s.from])",
        "(why some).map(s => [s.node, s.holds, s.state, s.from])",
        "(affects c1.passed).map(s => s.node)",
        "(affects c2.passed).map(s => s.node)",
        "(model |> derived).map(s => s.node)",
    ] {
        assert_eq!(
            read(&mut quantified, probe),
            read(&mut spelled, probe),
            "{probe}"
        );
    }
    assert_eq!(
        read(&mut quantified, "(affects c1.passed).map(s => s.node)"),
        json!(["ready", "some"])
    );
    assert_eq!(
        read(&mut quantified, "(affects c2.passed).map(s => s.node)"),
        json!(["ready"])
    );
    assert_eq!(
        read(&mut quantified, "[(why ready)[0].rule, (why some)[0].rule]"),
        json!([
            "ready = (all c in Check:c.passed)",
            "some = (any c in Check:c.passed)"
        ])
    );
    assert_eq!(
        read(&mut spelled, "(why ready)[0].rule"),
        json!("ready = Check.every(function())")
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
fn the_native_repl_buffers_colon_free_headers_containing_quantifiers() {
    for header in [
        "if all v in xs: v > 0",
        "if any v in xs: v > 1",
        "if (all v in xs: v > 0)",
        "if all v in xs: v > 0 ? true : false",
        "if all v in xs: v ?? false",
    ] {
        let output = repl(&format!(
            "xs = [1, 2]\n{header}\n    result = \"held\"\n\nresult\nif false\n    result = \"none\"\nelse all v in xs: v > 1\n    result = \"never\"\nelse\n    result = \"fallback\"\n\nresult\n"
        ));
        assert!(output.status.success(), "{header}\n{output:?}");
        assert_eq!(output.stderr, b"", "{header}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{BANNER}[1.0,2.0]\n\"held\"\n\"held\"\n\"fallback\"\n\"fallback\"\n"),
            "{header}"
        );
    }
}
