mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::{generate, generate_all};
use nucleoid::lang::ast::{
    BinaryOp, Expr, Function, FunctionBody, Parameter, ParameterDefault, Stmt, parse,
    parse_expression,
};
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde_json::json;

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "suppose regression".to_string(),
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

fn json_of(runtime: &Runtime, value: &Value) -> serde_json::Value {
    integral(serde_json::from_str(&runtime.serialize_json(value).unwrap()).unwrap())
}

fn read(runtime: &mut Runtime, source: &str) -> serde_json::Value {
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    json_of(runtime, &value)
}

fn evaluate(source: &str) -> serde_json::Value {
    read(&mut Runtime::new(), source)
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

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn number(value: f64) -> Expr {
    Expr::Number(value)
}

fn assign(target: &str, value: Expr) -> Stmt {
    Stmt::Assign {
        target: name(target),
        value,
    }
}

fn suppose(target: Option<&str>, body: Vec<Stmt>) -> Stmt {
    Stmt::Suppose {
        target: target.map(name),
        body,
    }
}

fn dependencies(runtime: &Runtime, key: &str) -> Vec<String> {
    runtime
        .graph
        .retrieve(&NodeKey::variable(key))
        .unwrap_or_else(|| panic!("{key} is filed"))
        .dependencies
        .iter()
        .map(|key| key.as_str().to_string())
        .collect()
}

fn contains_suppose(statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| match statement {
        Stmt::Suppose { .. } => true,
        Stmt::If {
            consequent,
            alternate,
            ..
        } => {
            contains_suppose(consequent)
                || alternate
                    .as_deref()
                    .is_some_and(|alternate| contains_suppose(std::slice::from_ref(alternate)))
        }
        Stmt::Block(body) | Stmt::For { body, .. } => contains_suppose(body),
        Stmt::Try { body, catch, .. } => contains_suppose(body) || contains_suppose(catch),
        Stmt::Function(function) => function_supposes(function),
        Stmt::Class(class) => {
            contains_suppose(&class.constructor)
                || class.methods.iter().any(|method| function_supposes(method))
        }
        _ => false,
    })
}

fn function_supposes(function: &Function) -> bool {
    match &function.body {
        FunctionBody::Block(body) => contains_suppose(body),
        FunctionBody::Expression(_) => false,
    }
}

fn mentions_suppose(source: &str) -> bool {
    source.match_indices("suppose").any(|(index, word)| {
        let boundary = |character: char| !(character.is_ascii_alphanumeric() || character == '_');
        source[..index].chars().next_back().is_none_or(boundary)
            && source[index + word.len()..]
                .chars()
                .next()
                .is_none_or(boundary)
    })
}

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid runs a suppose block in a hypothetical world",
    "Nucleoid discards instances created in a suppose block",
    "Nucleoid keeps the value of a suppose block as a snapshot",
    "Nucleoid nests suppose blocks and discards the inner one first",
    "Nucleoid runs header-less suppose blocks over whole populations",
    "Nucleoid returns hypothetical answers from functions and methods",
    "Nucleoid keeps suppose as an ordinary name beside suppose blocks",
    "Nucleoid supposes a changed class level rule",
];

#[test]
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let readme = common::snippets(include_str!("../README.md"), "README.md");
    let examples = common::snippets(include_str!("../docs/examples.md"), "docs/examples.md");
    let appended = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (
            &reference,
            "Nucleoid answers what if questions with suppose",
        ),
        (
            &readme,
            "Suppose blocks answer what-if questions without changing the model",
        ),
        (
            &examples,
            "Suppose blocks test restocking plans without committing them",
        ),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        let statements = parse(&case.source).unwrap();
        assert!(contains_suppose(&statements), "{title} supposes");
        let count = common::assertions::protected_assertions(&statements).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 73);
}

#[test]
fn suppose_forms_parse_into_one_statement_with_the_premise_first() {
    let premise = assign("reading", number(95.0));
    let value = Stmt::Expression(name("alarm"));
    let expected = suppose(None, vec![premise.clone(), value.clone()]);
    for source in [
        "suppose reading = 95: alarm",
        "suppose reading = 95:\n    alarm",
        "suppose reading = 95:\n\n    alarm\n",
        "suppose: reading = 95; alarm",
        "suppose:\n    reading = 95\n    alarm",
    ] {
        assert_eq!(
            parse(source).unwrap(),
            std::slice::from_ref(&expected),
            "{source:?}"
        );
    }
    for source in [
        "peak = suppose reading = 95: alarm",
        "peak = suppose:\n    reading = 95\n    alarm",
    ] {
        assert_eq!(
            parse(source).unwrap(),
            [suppose(Some("peak"), vec![premise.clone(), value.clone()])],
            "{source:?}"
        );
    }
    assert_eq!(
        parse("suppose: alarm()").unwrap(),
        [suppose(
            None,
            vec![Stmt::Expression(Expr::Call {
                callee: Box::new(name("alarm")),
                arguments: Vec::new(),
            })]
        )]
    );
    assert_eq!(
        parse("suppose reading = 95: a; b\nc").unwrap(),
        [
            suppose(
                None,
                vec![
                    premise.clone(),
                    Stmt::Expression(name("a")),
                    Stmt::Expression(name("b")),
                ],
            ),
            Stmt::Expression(name("c")),
        ]
    );
    assert_eq!(
        parse("suppose reading = 95: a c").unwrap(),
        [
            suppose(None, vec![premise.clone(), Stmt::Expression(name("a"))]),
            Stmt::Expression(name("c")),
        ]
    );
    assert_eq!(
        parse("box.limit = suppose x += 1: x").unwrap(),
        [Stmt::Suppose {
            target: Some(Expr::Member {
                object: Box::new(name("box")),
                property: "limit".to_string(),
            }),
            body: vec![
                assign(
                    "x",
                    Expr::Binary {
                        operator: BinaryOp::Add,
                        left: Box::new(name("x")),
                        right: Box::new(number(1.0)),
                    },
                ),
                Stmt::Expression(name("x")),
            ],
        }]
    );
    assert_eq!(
        parse("outer = suppose a = 1:\n    inner = suppose b = 2: c\n    inner").unwrap(),
        [suppose(
            Some("outer"),
            vec![
                assign("a", number(1.0)),
                suppose(
                    Some("inner"),
                    vec![assign("b", number(2.0)), Stmt::Expression(name("c"))]
                ),
                Stmt::Expression(name("inner")),
            ],
        )]
    );

    for (source, canonical) in [
        (
            "suppose Item(5): Item.length",
            "suppose{Item(5);Item.length}",
        ),
        ("suppose delete x: y", "suppose{delete x;y}"),
        ("suppose not ready: y", "suppose{!ready;y}"),
        ("suppose x = c ? 1 : 2: x", "suppose{x=(c?1:2);x}"),
        (
            "suppose ok = all v in xs: v > 0: ok",
            "suppose{ok=(all v in xs:v>0);ok}",
        ),
        ("suppose f = x => x + 1: f", "suppose{f=function();f}"),
        ("suppose $Order.fee = 0: fees", "suppose{$Order.fee=0;fees}"),
        ("if ready: suppose x = 1: y", "if(ready){suppose{x=1;y}}"),
        ("{ suppose x = 1: y }", "{suppose{x=1;y}}"),
        ("{ suppose: x }", "{suppose{x}}"),
        ("x = suppose: int", "x=suppose{int}"),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), canonical, "{source}");
    }
}

#[test]
fn canonical_rendering_folds_the_premise_and_reads_back() {
    for (source, canonical) in [
        ("suppose reading = 95: alarm", "suppose{reading=95;alarm}"),
        ("suppose: reading = 95; alarm", "suppose{reading=95;alarm}"),
        (
            "peak = suppose reading = 95: alarm",
            "peak=suppose{reading=95;alarm}",
        ),
        (
            "o.p = suppose: suppose a = 1: a",
            "o.p=suppose{suppose{a=1;a}}",
        ),
    ] {
        let statements = parse(source).unwrap();
        assert_eq!(generate_all(&statements), canonical, "{source}");
    }

    let body =
        parse("peak = suppose reading = level: alarm\nsuppose: reading = 1\nreturn peak").unwrap();
    let function = Stmt::Function(Arc::new(Function {
        name: Some("probe".to_string()),
        parameters: vec![Parameter {
            name: "level".to_string(),
            type_name: None,
            default: Some(ParameterDefault::Number(95.0)),
            rest: false,
        }],
        body: FunctionBody::Block(body.clone()),
    }));
    let rendered = generate(&function);
    assert_eq!(
        rendered,
        "def probe(level=95):\n    peak=suppose:\n        reading=level\n        alarm\n    suppose:\n        reading=1\n    return peak"
    );
    let parsed = parse(&rendered).unwrap();
    let [Stmt::Function(read_back)] = parsed.as_slice() else {
        panic!("one function");
    };
    assert_eq!(read_back.body, FunctionBody::Block(body));
}

#[test]
fn accepted_readings_of_the_name_suppose_are_unchanged() {
    for (source, expected) in [
        ("suppose = 3\nx = suppose + 1\nx", json!(4)),
        ("def suppose(v) => v * 2\nsuppose(4)", json!(8)),
        (
            "suppose = 1\nreading = 40\nsuppose reading = 95\n    reading = 50\nreading",
            json!(50),
        ),
        (
            "suppose = 1\nreading = 40\nsuppose { reading = 50 }\nreading",
            json!(50),
        ),
        (
            "suppose = 1\nreading = 40\nsuppose reading = 95 { reading = 50 }\nreading",
            json!(50),
        ),
        ("suppose = 1\nsuppose x = 1 { y = 2 }", json!(2)),
        ("suppose = 1\nsuppose assert: suppose, 1", json!(1)),
        ("suppose = 1\nx = suppose assert: suppose, 1\nx", json!(1)),
        ("suppose = 1\nsuppose x = lambda: 2\nx()", json!(2)),
        ("suppose = 1\nsuppose x = lambda a: a * 3\nx(2)", json!(6)),
        ("suppose = 1\nsuppose -1", json!(0)),
        ("suppose = 1\nx = 0\nsuppose if x > 5: x = 9\nx", json!(0)),
        (
            "suppose = 1\nclass A\nA()\nn = 0\nsuppose for a of A: n = 7\nn",
            json!(7),
        ),
        (
            "suppose = 1\nsuppose try: throw 1 catch: e = 2\ne",
            json!(2),
        ),
        ("suppose = 1\nsuppose def f(): return 3\nf()", json!(3)),
        ("suppose = 1\nsuppose class B\nB.length", json!(0)),
        (
            "suppose = 1\nx = 4\nsuppose delete x\ntypeof suppose",
            json!("number"),
        ),
        (
            "suppose = 1\nx = suppose\ny = suppose + 1\n[x, y]",
            json!([1, 2]),
        ),
        ("suppose = 1\nx = 1\nsuppose x", json!(1)),
        ("x = suppose = 3\n[x, suppose]", json!([3, 3])),
        ("o = {suppose: 3}\no.suppose = 5\no.suppose", json!(5)),
        ("def f(suppose): return suppose + 1\nf(1)", json!(2)),
        (
            "class C(suppose): this.suppose = suppose\nc = C(4)\nc.suppose",
            json!(4),
        ),
        ("n = 0\nfor suppose of \"ab\": n = n + 1\nn", json!(2)),
        ("suppose = 3\nt = `${suppose} items`\nt", json!("3 items")),
        ("suppose = 2\nbox = {suppose}\nbox.suppose", json!(2)),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }

    for (source, expected) in [
        (
            "suppose = 1\nsuppose: int",
            (ErrorKind::Reference, "Missing definition", 2, 1),
        ),
        (
            "suppose = 1\nsuppose x: int",
            (ErrorKind::Reference, "Missing definition", 2, 9),
        ),
        (
            "suppose = 1\npeak = suppose x: int",
            (ErrorKind::Reference, "Missing definition", 2, 16),
        ),
        (
            "suppose = 1\nsuppose: x; y = 2",
            (ErrorKind::Reference, "Missing definition", 2, 1),
        ),
    ] {
        let (kind, message, line, column) = expected;
        assert_eq!(
            failure(source),
            (kind, message.to_string(), Some(Position::new(line, column))),
            "{source}"
        );
    }

    for (source, line, column) in [
        ("suppose = 1\nsuppose x.lambda: 1", 2, 19),
        ("suppose(1): 2", 1, 13),
        ("suppose.x: 1", 1, 12),
        ("suppose + 1: 2", 1, 14),
        ("suppose = 1\nsuppose f = lambda: 1: 2", 2, 24),
    ] {
        assert_eq!(
            syntax_error(source),
            ("Unexpected ':'".to_string(), Position::new(line, column)),
            "{source}"
        );
    }

    assert_eq!(
        parse("suppose x: int").unwrap(),
        [
            Stmt::Expression(name("suppose")),
            Stmt::Declaration {
                name: "x".to_string(),
                type_name: "int".to_string(),
            },
        ]
    );
    assert_eq!(
        parse("suppose: int").unwrap(),
        [Stmt::Declaration {
            name: "suppose".to_string(),
            type_name: "int".to_string(),
        }]
    );
}

#[test]
fn parse_expression_still_reads_the_name_suppose() {
    assert_eq!(
        parse_expression("suppose x = 1: y").unwrap(),
        name("suppose")
    );
    assert_eq!(parse_expression("suppose: y").unwrap(), name("suppose"));
    assert_eq!(
        parse_expression("peak = suppose x = 1: y").unwrap(),
        Expr::Assign {
            target: Box::new(name("peak")),
            value: Box::new(name("suppose")),
        }
    );
}

#[test]
fn failed_premise_attempts_are_cached_so_nested_juxtapositions_parse_quickly() {
    let mut source = String::from("0");
    for _ in 0..20 {
        source = format!("function() {{ suppose = 1\nsuppose f = {source} }}");
    }
    let source = format!("suppose = 1\nsuppose g = {source}\ng");
    let started = Instant::now();
    let statements = parse(&source).unwrap();
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_eq!(statements.len(), 4);
    let deeper = source.replacen("0", "function() { suppose = 1\nsuppose f = 0 }", 1);
    assert_eq!(syntax_error(&deeper).0, "Expressions are nested too deeply");
    assert!(
        statements
            .iter()
            .all(|statement| !matches!(statement, Stmt::Suppose { .. }))
    );
}

#[test]
fn blocks_roll_back_every_effect() {
    let initial = r#"
class Account(balance): this.balance = balance
$Account.rich = $Account.balance > 100
total = Account.reduce(lambda sum, account: sum + account.balance, 0)
acct = Account(50)
a = 1
b = a + 1
trace = why b
exposure = affects a
"#;
    let mut actual = Runtime::new();
    checked(&mut actual, initial);
    let mut control = Runtime::new();
    checked(&mut control, initial);

    for (source, body, expected) in [
        ("suppose a = 5: b", "a = 5; b", json!(6)),
        (
            "suppose acct.balance = 500: [acct.rich, total]",
            "acct.balance = 500; [acct.rich, total]",
            json!([true, 500]),
        ),
        (
            "suppose Account(200): [Account.length, total]",
            "Account(200); [Account.length, total]",
            json!([2, 250]),
        ),
        (
            "suppose named = Account(300): named.rich",
            "named = Account(300); named.rich",
            json!(true),
        ),
        (
            "suppose c = b * 2: [c, exposure.length]",
            "c = b * 2; [c, exposure.length]",
            json!([4, 2]),
        ),
        (
            "suppose $Account.rich = $Account.balance > 10: acct.rich",
            "$Account.rich = $Account.balance > 10; acct.rich",
            json!(true),
        ),
        (
            "suppose:\n    class Temp(v): this.v = v\n    t = Temp(1)\n    Temp.length",
            "class Temp(v): this.v = v; t = Temp(1); Temp.length",
            json!(1),
        ),
        (
            "suppose: def helper(x) => x + a; helper(1)",
            "def helper(x) => x + a; helper(1)",
            json!(2),
        ),
        (
            "suppose delete b: trace.length",
            "delete b; trace.length",
            json!(0),
        ),
        (
            "suppose b = 10: [trace[0].holds, trace.length]",
            "b = 10; [trace[0].holds, trace.length]",
            json!([10, 1]),
        ),
        (
            "suppose:\n    for account of Account: account.balance *= 3\n    total",
            "for account of Account: account.balance *= 3\n    total",
            json!(150),
        ),
        (
            "suppose a = 2:\n    inner = suppose b = 7: b\n    [inner, b]",
            "a = 2; inner = b; [inner, b]",
            json!([7, 3]),
        ),
        (
            "suppose a = 2: a = 3; a = 4; b",
            "a = 2; a = 3; a = 4; b",
            json!(5),
        ),
        (
            "suppose acct.extra = 1: delete acct.extra",
            "acct.extra = 1; delete acct.extra",
            json!(null),
        ),
        (
            "suppose exposure = 0: exposure",
            "exposure = 0; exposure",
            json!(0),
        ),
        ("suppose: shape = {n: 1}", "shape = {n: 1}", json!(null)),
        (
            "suppose: shape = {n: 1}; assert(shape.n, 1)",
            "shape = {n: 1}; assert(shape.n, 1)",
            json!(null),
        ),
    ] {
        assert_eq!(read(&mut actual, source), expected, "{source}");
        let caught = if body.contains('\n') {
            format!("try:\n    {body}\n    throw 0\ncatch:\n    pass")
        } else {
            format!("try: {body}; throw 0 catch: pass")
        };
        read(&mut control, &caught);
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    }

    checked(
        &mut actual,
        "a = 3\nassert(b, 4)\nassert(trace[0].holds, 4)\nacct.balance = 200\nassert(acct.rich, true)\nassert(total, 200)",
    );
}

#[test]
fn assertions_inside_a_block_stay_counted_and_recorded() {
    let mut runtime = Runtime::new();
    runtime.run("x = 1").unwrap();
    runtime
        .run("suppose x = 2: assert(x, 2); assert(x, 3)")
        .unwrap();
    assert_eq!(runtime.assertions_run(), 2);
    let failures = runtime.take_assertions();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].actual, Value::Number(2.0));
    assert_eq!(failures[0].expected, Value::Number(3.0));
    assert_eq!(runtime.run("x").unwrap(), Value::Number(1.0));
}

#[test]
fn effects_inside_a_failed_run_roll_back_with_the_run() {
    let mut runtime = Runtime::new();
    checked(&mut runtime, "a = 1\nb = a + 1");
    let mut control = Runtime::new();
    checked(&mut control, "a = 1\nb = a + 1");
    let error = runtime
        .run("kept = suppose a = 5: b\nlist = [kept]\nthrow \"STOP\"")
        .unwrap_err();
    assert_eq!(
        error.thrown_value(),
        Some(&Value::String("STOP".to_string()))
    );
    runtime_equivalence::assert_equivalent(&runtime, None, &control, None);
}

#[test]
fn values_leave_a_block_as_plain_data() {
    for (source, expected) in [
        ("x = 1\nsuppose x = 5: x * 2", json!(10)),
        (
            "x = 1\nv = suppose x = 5: [x, \"s\", true, null]\nv",
            json!([5, "s", true, null]),
        ),
        (
            "class Account(balance): this.balance = balance\nacct = Account(10)\ncopy = suppose acct.balance = 99: acct\n[copy.balance, acct.balance, copy == acct, Account.length, typeof copy]",
            json!([99, 10, false, 1, "Object"]),
        ),
        (
            "class Account(balance): this.balance = balance\nmade = suppose: Account(7)\n[made.balance, Account.length, Account.filter(a -> a.balance == 7).length]",
            json!([7, 0, 0]),
        ),
        (
            "class Account(balance):\n    this.balance = balance\n    def twice() => this.balance * 2\nmade = suppose: Account(7)\n[made.balance, made.twice]",
            json!([7, null]),
        ),
        (
            "class Account(balance): this.balance = balance\nlisted = suppose Account(3): Account.filter(a -> a.balance > 0)\nn = 0\nfor a of listed: n = n + 1\n[listed.length, n, listed[0].balance]",
            json!([1, 0, 3]),
        ),
        (
            "a = 1\nb = a * 2\ntrace = suppose a = 5: why b\n[trace.length, trace[0].node, trace[0].holds, trace[1].holds, trace[0].state, trace[0].from]",
            json!([2, "b", 10, 5, "derived", ["a"]]),
        ),
        (
            "a = 1\nb = a * 2\ntrace = suppose a = 5: why b\na = 7\n[trace[0].holds, trace.length]",
            json!([10, 2]),
        ),
        (
            "a = 1\nb = a * 2\ntrace = suppose a = 5: why b\nchained = trace |> why\n[chained.length, chained[0].node, chained[0].state]",
            json!([1, "trace", "stated"]),
        ),
        (
            "pair = suppose: shared = {n: 1}; [shared, shared]\npair[0].n = 2\n[pair[1].n, pair[0] == pair[1]]",
            json!([2, true]),
        ),
        (
            "loop = suppose: node = {name: \"a\"}; node.self = node; node\n[loop.self.self.name, loop.self == loop]",
            json!(["a", true]),
        ),
        (
            "outer = {inner: {n: 1}}\ncopy = suppose outer.inner.n = 2: outer\n[copy.inner.n, outer.inner.n, copy.inner == outer.inner]",
            json!([2, 1, false]),
        ),
        (
            "when = suppose: Date(86400000)\n[when.getTime(), typeof when]",
            json!([86400000, "Date"]),
        ),
        (
            "pattern = suppose: /ab+/\npattern.test(\"abbb\")",
            json!(true),
        ),
        ("m = suppose: Math\nm.max(1, 2)", json!(2)),
        (
            "x = suppose: missing_property_owner = {}; missing_property_owner.none\nx",
            json!(null),
        ),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn an_object_missing_when_the_block_ends_leaves_as_an_empty_plain_object() {
    let mut runtime = Runtime::new();
    let value = checked(
        &mut runtime,
        "class Box\nkept = suppose:\n    try:\n        made = Box()\n        made.v = 1\n        throw made\n    catch error:\n        caught = error\n    [caught, caught.v]\nassert(kept[0], {})\nassert(kept[1], null)\n[typeof kept[0], Box.length]",
    );
    assert_eq!(json_of(&runtime, &value), json!(["Object", 0]));
}

#[test]
fn a_class_declared_inside_a_block_leaves_as_null() {
    for (source, expected) in [
        ("kind = suppose: class Temp; Temp\nkind", json!(null)),
        ("suppose: class Temp; Temp", json!(null)),
        (
            "kind = suppose: class Temp; Temp\n[kind, kind.length, kind ?? \"gone\"]",
            json!([null, null, "gone"]),
        ),
        (
            "kind = suppose: class Temp; [Temp, 1]\nkind",
            json!([null, 1]),
        ),
        (
            "class Kept\nkind = suppose: Kept\n[kind, typeof kind]",
            json!(["Kept", "List"]),
        ),
        (
            "class Kept\nkind = suppose: class Kept(v): this.v = v; Kept\nKept(1)\n[kind, Kept.length]",
            json!(["Kept", 1]),
        ),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
    for source in [
        "kind = suppose: class Temp; Temp\nTemp.length",
        "kind = suppose: class Temp; Temp\nTemp()",
    ] {
        assert_eq!(
            failure(source),
            (
                ErrorKind::Reference,
                "Temp is not defined".to_string(),
                Some(Position::new(2, 1))
            ),
            "{source}"
        );
    }
}

#[test]
fn a_function_declared_inside_a_block_stays_callable_without_its_name() {
    for (source, expected) in [
        (
            "twice = suppose: def double(x) => x * 2; double\n[twice(4), typeof twice]",
            json!([8, "Function"]),
        ),
        ("twice = suppose: x -> x * 2\ntwice(5)", json!(10)),
        (
            "scale = 3\nscaled = suppose scale = 10: def apply(x) => x * scale; apply\nscaled(2)",
            json!(6),
        ),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
    for (source, missing) in [
        (
            "twice = suppose: def double(x) => x * 2; double\ndouble(4)",
            "double",
        ),
        (
            "reader = suppose: helper = 3; def scaled(x) => x * helper; scaled\nreader(2)",
            "helper",
        ),
    ] {
        assert_eq!(
            failure(source),
            (
                ErrorKind::Reference,
                format!("{missing} is not defined"),
                Some(Position::new(2, 1))
            ),
            "{source}"
        );
    }
}

#[test]
fn throws_and_errors_leave_after_the_rollback() {
    for (source, expected) in [
        (
            "reading = 40\nalarm = reading > 90\ntry:\n    suppose reading = 95: throw alarm\ncatch error:\n    caught = error\n[caught, reading, alarm]",
            json!([true, 40, false]),
        ),
        (
            "x = 1\ntry:\n    suppose x = 5: throw [x, \"over\"]\ncatch error:\n    caught = error\n[caught, x]",
            json!([[5, "over"], 1]),
        ),
        (
            "class Item(price): this.price = price\ntry:\n    suppose: throw Item(5)\ncatch error:\n    caught = error\n[caught.price, Item.length]",
            json!([null, 0]),
        ),
        (
            "class Item(price): this.price = price\nitem = Item(2)\ntry:\n    suppose item.price = 9: throw item\ncatch error:\n    caught = error\n[caught.price, item.price, caught == item]",
            json!([null, 2, false]),
        ),
        (
            "reading = 40\ntry:\n    suppose reading = 95: missing\ncatch error:\n    failure = error\n[failure, reading]",
            json!(["ReferenceError: missing is not defined", 40]),
        ),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }

    let mut runtime = Runtime::new();
    checked(&mut runtime, "x = 1\ny = x + 1");
    let mut control = Runtime::new();
    checked(&mut control, "x = 1\ny = x + 1");
    for (source, kind, message, line) in [
        (
            "suppose x = 5: throw \"STOP\"",
            ErrorKind::Thrown,
            "STOP",
            1,
        ),
        (
            "z = 0\nsuppose x = 5: missing",
            ErrorKind::Reference,
            "missing is not defined",
            2,
        ),
        (
            "suppose x = 5: suppose y = 1: throw y * 2",
            ErrorKind::Thrown,
            "2",
            1,
        ),
        (
            "suppose x = 5:\n    y = 0\n    $Nope.p = 1",
            ErrorKind::Reference,
            "Nope is not defined",
            1,
        ),
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), kind, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(Position::new(line, 1)), "{source}");
        runtime_equivalence::assert_equivalent(&runtime, None, &control, None);
    }
    assert_eq!(
        runtime
            .run("suppose x = 5: throw y")
            .unwrap_err()
            .thrown_value(),
        Some(&Value::Number(6.0))
    );
}

#[test]
fn an_assigned_block_without_a_value_raises_before_it_runs() {
    for body in [
        "z = 1",
        "assert(x, 2)",
        "pass",
        "if x: 1",
        "for c of \"ab\": n = 1",
        "def f() => 1",
        "class Temp",
        "w = suppose x = 3: x",
        "suppose x = 3: y = x",
        "return 1",
    ] {
        let source = format!("x = 1\ncount = 0\ny = suppose x = 2: assert(x, 2); {body}");
        let mut runtime = Runtime::new();
        let error = runtime.run(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{source}");
        assert_eq!(error.message(), "suppose has no value", "{source}");
        assert_eq!(error.position(), Some(Position::new(3, 1)), "{source}");
        assert_eq!(runtime.assertions_run(), 0, "{source}");
    }
    assert_eq!(
        evaluate(
            "x = 1\ntry:\n    y = suppose x = 2: z = 1\ncatch error:\n    failure = error\n[failure, x]"
        ),
        json!(["TypeError: suppose has no value", 1])
    );
    assert_eq!(
        evaluate("x = 1\ny = suppose x = 5: suppose x = 7: x * 2\ny"),
        json!(14)
    );
    assert_eq!(evaluate("y = suppose: x = 1; assert(x, 1); x\ny"), json!(1));
}

#[test]
fn an_unassigned_block_evaluates_to_its_value_or_null() {
    for (source, expected) in [
        ("x = 1\nsuppose x = 5: x", json!(5)),
        ("x = 1\nsuppose x = 5: y = x", json!(null)),
        ("x = 1\nsuppose x = 5: assert(x, 5)", json!(null)),
        ("x = 1\nsuppose x = 5: x\nx", json!(1)),
        ("x = 1\nsuppose x = 5: suppose x = 7: x * 2", json!(14)),
        ("x = 1\nsuppose x = 5: suppose x = 7: y = x", json!(null)),
        ("x = 1\nsuppose: x + 1", json!(2)),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn return_inside_a_block_leaves_after_the_rollback() {
    for (source, expected) in [
        ("x = 1\nsuppose x = 5: return x * 3\nx = 100", json!(15)),
        (
            "x = 1\ndef f():\n    suppose x = 5: return x * 3\n    return 0\n[f(), x]",
            json!([15, 1]),
        ),
        (
            "class Box(v):\n    this.v = v\n    def peek(n): suppose this.v = n: return this.v * 2\nb = Box(1)\n[b.peek(4), b.v]",
            json!([8, 1]),
        ),
        (
            "x = 1\ndef f():\n    suppose x = 5: return {seen: x}\n    return 0\nr = f()\n[r.seen, x]",
            json!([5, 1]),
        ),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn local_bindings_are_restored() {
    for (source, expected) in [
        (
            "def f(n):\n    suppose n = 10: m = n\n    return n\nf(3)",
            json!(3),
        ),
        (
            "def f(n):\n    seen = suppose n = 10: n\n    return [n, seen]\nf(3)",
            json!([3, 10]),
        ),
        (
            "class Item(v): this.v = v\nItem(1)\nout = []\nfor item of Item:\n    suppose item = 5: out.push(item)\n    out.push(item.v)\nout",
            json!([1]),
        ),
        (
            "def g(n):\n    try:\n        n = 10\n        throw 0\n    catch:\n        pass\n    return n\ng(3)",
            json!(10),
        ),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn blocks_add_no_dependency_to_what_runs_them() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "w = 1\nx = 0\ndef f():\n    v = suppose x = 5: x + w\n    return v\nz = f()\nassert(z, 6)\nw = 100\nassert(z, 6)",
    );
    assert_eq!(dependencies(&runtime, "z"), ["f", "v"]);
    assert!(dependencies(&runtime, "v").is_empty());

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "w = 1\nx = 0\ndef f():\n    try:\n        x = 5\n        throw x + w\n    catch:\n        return error\nz = f()\nassert(z, 6)\nw = 100\nassert(z, 105)",
    );

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = 1\nwatch = 0\n{\n    seen = suppose a = 9: a + 1\n    watch = seen\n}\nassert(watch, 10)\na = 3\nassert(watch, 10)",
    );

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = 1\nb = a + 1\ntrace = suppose: why b\nassert(trace.length, 2)\nc = a * 3\nb = c + 1\nassert(trace.length, 2)",
    );
    assert!(dependencies(&runtime, "trace").is_empty());
    assert_eq!(
        runtime
            .graph
            .shape_dependencies(&NodeKey::variable("trace"))
            .count(),
        0
    );
}

#[test]
fn blocks_finish_their_own_propagation_while_other_changes_propagate() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = 1\nb = a * 2\ndef g(n):\n    v = suppose a = n: b\n    return v\nc = g(a + 10)\nassert(c, 22)\na = 2\nassert(c, 24)\nassert(b, 4)",
    );

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = 1\nb = a * 2\ndef g(n):\n    try:\n        a = n\n        throw b\n    catch:\n        return error\nc = g(a + 10)\nassert(c, 22)\na = 2\nassert(c, 4)",
    );

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = 1\nb = a + 1\nc = 0\nif b > 2:\n    c = suppose a = 10: b\nassert(c, 0)\na = 5\nassert(c, 11)\nassert(b, 6)",
    );

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "a = 1\ncalls = 0\ndef bump(n):\n    suppose a = n: calls = a\n    return n\nc = bump(a)\na = 4\nassert(c, 4)\nassert(calls, 0)",
    );
}

#[test]
fn blocks_run_whenever_the_statement_holding_them_runs() {
    for (source, expected) in [
        (
            "class Tank(level): this.level = level\nresults = []\nfor t of [Tank(1), Tank(2)]:\n    v = suppose t.level = t.level * 10: t.level\n    results.push(v)\n[results, Tank[0].level]",
            json!([[10, 20], 1]),
        ),
        (
            "x = 1\ndef f(n):\n    v = suppose x = n: x * 2\n    return v\n[f(2), f(3), x]",
            json!([4, 6, 1]),
        ),
        (
            "class Point(x):\n    this.x = x\n    this.doubled = suppose this.x = x * 2: this.x\np = Point(4)\n[p.x, p.doubled]",
            json!([4, 8]),
        ),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn class_level_rules_hold_snapshots_and_run_blocks_once_per_instance() {
    for (source, expected) in [
        (
            "class Tank(level): this.level = level\n$Tank.peak = suppose: 7\nt = Tank(1)\nu = Tank(2)\n[t.peak, u.peak]",
            json!([7, 7]),
        ),
        (
            "class Tank(level): this.level = level\ncap = 5\n$Tank.limit = suppose cap = 9: cap * 2\nt = Tank(1)\ncap = 1\n[t.limit, Tank(3).limit]",
            json!([18, 18]),
        ),
        (
            "class Tank(level): this.level = level\nif $Tank.level > 0:\n    $Tank.doubled = suppose $Tank.level = $Tank.level * 2: $Tank.level\nt = Tank(3)\nu = Tank(5)\n[t.doubled, u.doubled, t.level, u.level]",
            json!([6, 10, 3, 5]),
        ),
        (
            "class Order(amount): this.amount = amount\n$Order.fee = $Order.amount * 0.02\no = Order(100)\nseen = 0\n{\n    seen = suppose $Order.fee = 1: o.fee\n}\n[seen, o.fee, Order.length]",
            json!([1, 2, 1]),
        ),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn assignment_targets_follow_the_ordinary_assignment() {
    for (source, expected) in [
        ("box = {}\nbox.limit = suppose: 4\nbox.limit", json!(4)),
        (
            "class Box(v):\n    this.v = v\n    def keep(n): this.seen = suppose this.v = n: this.v\nb = Box(1)\nb.keep(5)\n[b.seen, b.v]",
            json!([5, 1]),
        ),
        (
            "def f(n):\n    n = suppose: n * 2\n    return n\nf(4)",
            json!(8),
        ),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
    for (source, kind, message) in [
        (
            "a = 1\n[1][0] = suppose a = 2: a",
            ErrorKind::Syntax,
            "Cannot assign to [1][0]",
        ),
        (
            "missing.x = suppose: 3",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "this = suppose: 3",
            ErrorKind::Type,
            "Cannot assign to 'this'",
        ),
    ] {
        let (actual_kind, actual_message, _) = failure(source);
        assert_eq!(
            (actual_kind, actual_message.as_str()),
            (kind, message),
            "{source}"
        );
    }
}

#[test]
fn out_of_scope_spellings_keep_their_current_readings() {
    for (source, message, line, column) in [
        (
            "a = 1\nb = 1\na = b = suppose a = 2: a",
            "Unexpected ':'",
            3,
            24,
        ),
        ("a = 1\na += suppose a = 2: a", "Unexpected ':'", 2, 21),
        (
            "def f():\n    return suppose a = 2: a\nf()",
            "Unexpected ':'",
            2,
            27,
        ),
        (
            "x = (suppose a = 2: a)",
            "Expected ')' but found identifier 'a'",
            1,
            14,
        ),
        ("x = [suppose: 1]", "Expected ']' but found ':'", 1, 13),
        ("f(suppose: 1)", "Expected ')' but found ':'", 1, 10),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    assert_eq!(
        failure("a = 1\nsuppose a = 2: diff"),
        (
            ErrorKind::Reference,
            "diff is not defined".to_string(),
            Some(Position::new(2, 1))
        )
    );
}

#[test]
fn malformed_blocks_report_exact_errors() {
    for (source, message, line, column) in [
        (
            "reading = 40\npeak = suppose reading = 95:",
            "Unexpected end of input",
            2,
            29,
        ),
        (
            "reading = 40\npeak = suppose reading = 95:\n",
            "Expected an indented block but found end of input",
            3,
            1,
        ),
        (
            "reading = 40\nsuppose reading = 95:",
            "Unexpected end of input",
            2,
            22,
        ),
        (
            "reading = 40\nsuppose reading = 95:\nreading = 1",
            "Expected an indented block but found identifier 'reading'",
            3,
            1,
        ),
        (
            "reading = 40\nsuppose reading = 95:\n    # only a comment\n",
            "Expected an indented block but found end of input",
            4,
            1,
        ),
        (
            "reading = 40\nsuppose reading = 95:;",
            "Unexpected ';'",
            2,
            23,
        ),
        (
            "reading = 40\nsuppose reading = 95:: pass",
            "Unexpected ':'",
            2,
            24,
        ),
        ("suppose:", "Unexpected end of input", 1, 9),
        (
            "suppose:\n",
            "Expected an indented block but found end of input",
            2,
            1,
        ),
        ("x = suppose:", "Unexpected end of input", 1, 13),
        ("x = suppose: ;", "Unexpected ';'", 1, 15),
        (
            "suppose reading = 95: assert: 1",
            "Expected ',' but found end of input",
            1,
            32,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source:?}"
        );
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
fn the_native_repl_buffers_suppose_headers_and_prints_block_values() {
    let output = repl(
        "reading = 40\nalarm = reading > 90\nsuppose reading = 95: alarm\npeak = suppose reading = 95:\n    alarm\n\npeak\nsuppose:\n    reading = 99\n    alarm\n\nalarm\nsuppose = 3\nsuppose reading = 50\nreading\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stderr, b"");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{BANNER}40.0\nfalse\ntrue\ntrue\ntrue\ntrue\nfalse\n3.0\n50.0\n50.0\n")
    );
}

#[test]
fn every_case_mentioning_suppose_is_exported_as_one_program() {
    let generated = include_str!("nucleoid.spec.rs");
    for title in SPEC_TITLES {
        let header = format!("/// {title}\n");
        let start = generated
            .find(&header)
            .unwrap_or_else(|| panic!("{title} is exported"));
        let rest = &generated[start + header.len()..];
        let end = rest.find("\n}\n").unwrap();
        let test = &rest[..end];
        assert_eq!(test.matches("run(").count(), 1, "{title}\n{test}");
        assert!(test.contains("let mut run = runner();"), "{title}\n{test}");
        assert!(mentions_suppose(test), "{title}");
    }
}
