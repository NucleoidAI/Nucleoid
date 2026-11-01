mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::collections::BTreeSet;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::{generate, generate_all};
use nucleoid::lang::ast::{BinaryOp, Expr, FunctionBody, Stmt, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "while loop regression".to_string(),
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

fn json(runtime: &mut Runtime, source: &str) -> String {
    let value = run(runtime, source);
    runtime.serialize_json(&value).unwrap()
}

fn evaluate(source: &str) -> String {
    json(&mut Runtime::new(), source)
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

fn name(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn contains_while(statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| match statement {
        Stmt::While { .. } => true,
        Stmt::If {
            consequent,
            alternate,
            ..
        } => {
            contains_while(consequent)
                || alternate
                    .as_ref()
                    .is_some_and(|statement| contains_while(std::slice::from_ref(statement)))
        }
        Stmt::Block(body)
        | Stmt::For { body, .. }
        | Stmt::ForEntries { body, .. }
        | Stmt::Suppose { body, .. } => contains_while(body),
        Stmt::Try { body, catch, .. } => contains_while(body) || contains_while(catch),
        Stmt::Function(function) => {
            matches!(&function.body, FunctionBody::Block(body) if contains_while(body))
        }
        Stmt::Class(class) => contains_while(&class.constructor)
            || class.methods.iter().any(
                |method| matches!(&method.body, FunctionBody::Block(body) if contains_while(body)),
            ),
        _ => false,
    })
}

fn keys(runtime: &Runtime) -> BTreeSet<String> {
    runtime
        .graph
        .keys()
        .map(|key| key.as_str().to_string())
        .collect()
}

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid repeats a while loop until its condition fails",
    "Nucleoid skips a while loop whose condition starts falsy",
    "Nucleoid runs while loops in methods with parameters and return",
    "Nucleoid nests while loops with conditionals and for loops",
    "Nucleoid creates instances in a while loop until a class rule holds",
    "Nucleoid stops a runaway while loop and rolls it back",
    "Nucleoid keeps while as an ordinary name beside while loops",
    "Nucleoid rolls back a while loop that fails during an iteration",
];

#[test]
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let readme = common::snippets(include_str!("../README.md"), "README.md");
    let examples = common::snippets(include_str!("../docs/examples.md"), "docs/examples.md");
    let appended = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (&reference, "Nucleoid repeats statements with a while loop"),
        (&readme, "While loops step a world model until a goal holds"),
        (
            &examples,
            "While loops run packing shifts until every order is done",
        ),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        let statements = parse(&case.source).unwrap();
        assert!(contains_while(&statements), "{title} loops");
        let count = common::assertions::protected_assertions(&statements).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 49);
}

#[test]
fn while_loops_build_one_statement_with_canonical_renderings() {
    let statements = parse("while n < 3:\n    n = n + 1").unwrap();
    assert_eq!(
        statements,
        [Stmt::While {
            condition: Expr::Binary {
                operator: BinaryOp::Less,
                left: Box::new(name("n")),
                right: Box::new(Expr::Number(3.0)),
            },
            body: vec![Stmt::Assign {
                target: name("n"),
                value: Expr::Binary {
                    operator: BinaryOp::Add,
                    left: Box::new(name("n")),
                    right: Box::new(Expr::Number(1.0)),
                },
            }],
        }]
    );
    assert_eq!(generate_all(&statements), "while(n<3){n=n+1}");

    equivalent("while n < 3: n = n + 1", "while n < 3:\n    n = n + 1");
    equivalent(
        "while n < 3: n = n + 1; seen = n",
        "while n < 3:\n    n = n + 1\n    seen = n",
    );
    equivalent(
        "while (n < 3): n = n + 1",
        "while n < 3:\n\n    # comment\n    n = n + 1",
    );
    assert_eq!(
        generate_all(&parse("while a:\n    while b: c()\n    d = 1").unwrap()),
        "while(a){while(b){c()};d=1}"
    );
    assert_eq!(
        generate_all(&parse("while ready ? on : off: tick()").unwrap()),
        "while((ready?on:off)){tick()}"
    );
    assert_eq!(
        NodeKey::block(&parse("while n < 3: n = n + 1").unwrap(), None).as_str(),
        "block(while(n<3){n=n+1})"
    );

    let source = "def step(n = 3):\n    while n > 0:\n        n = n - 1\n    return n";
    let function = parse(source).unwrap();
    let rendered = generate(&function[0]);
    assert_eq!(
        rendered,
        "def step(n=3):\n    while n>0:\n        n=n-1\n    return n"
    );
    assert_eq!(parse(&rendered).unwrap(), function);
}

#[test]
fn while_keeps_every_accepted_reading_of_its_name() {
    equivalent("while ready: tick", "while\nready: tick");
    equivalent("while ready: tick;", "while\nready: tick;");
    equivalent("while assert: ready, true", "while\nassert: ready, true");
    equivalent("while lambda: 1", "while\nlambda: 1");
    equivalent("while ready + lambda: 1", "while\nready + lambda: 1");
    equivalent("while not lambda: 1", "while\nnot lambda: 1");
    equivalent("while x => lambda: 1", "while\nx => lambda: 1");
    equivalent("while lambda x: x", "while\nlambda x: x");
    equivalent(
        "while ready if done: tick()",
        "while\nready\nif done: tick()",
    );
    equivalent("while all v in xs: v", "while\nall v in xs: v");
    equivalent(
        "while n < 3 {\n    n = n + 1\n}",
        "while\nn < 3\n{\n    n = n + 1\n}",
    );
    equivalent("while n < 3\n    n = n + 1", "while\nn < 3\nn = n + 1");
    equivalent("while (n) b: int", "while(n)\nb: int");
    equivalent(
        "class Gate:\n    while open: int",
        "class Gate:\n    while\n    open: int",
    );
    equivalent("if ready: while open: int", "if ready: while\nopen: int");

    assert_eq!(
        parse("while(4)").unwrap(),
        [Stmt::Expression(Expr::Call {
            callee: Box::new(name("while")),
            arguments: vec![Expr::Number(4.0)],
        })]
    );
    assert_eq!(
        parse("while: int").unwrap(),
        [Stmt::Declaration {
            name: "while".to_string(),
            type_name: "int".to_string(),
        }]
    );
    assert_eq!(
        parse_expression("while ready: tick()").unwrap(),
        name("while")
    );
    assert!(!contains_while(
        &parse("for while of items:\n    point = {while: while}\n    point.while = 1").unwrap()
    ));

    assert_eq!(evaluate("while = 2\nwhile"), "2.0");
    assert_eq!(evaluate("def while(x) => x + 1\nwhile(4)"), "5.0");
    assert_eq!(evaluate("while = 3\nn = while + 1\nn"), "4.0");
    assert_eq!(
        evaluate("while = 1\nwhile assert: while, 1\nwhile lambda: 2"),
        "\"[function]\""
    );
    assert_eq!(
        failure("while ready: tick"),
        (
            ErrorKind::Reference,
            "while is not defined".to_string(),
            Some(Position::new(1, 1))
        )
    );
    assert_eq!(
        failure("while = 1\nwhile ready: tick"),
        (
            ErrorKind::Reference,
            "Missing definition".to_string(),
            Some(Position::new(2, 7))
        )
    );
    assert_eq!(
        failure("n = 0\nwhile n < 3 {\n    n = n + 1\n}"),
        (
            ErrorKind::Reference,
            "while is not defined".to_string(),
            Some(Position::new(2, 1))
        )
    );
}

#[test]
fn conditions_are_tested_before_each_iteration_with_if_truthiness() {
    for falsy in ["0", "\"\"", "null", "false", "undefined"] {
        assert_eq!(
            evaluate(&format!(
                "flag = {falsy}\nran = 0\nwhile flag:\n    ran = ran + 1\nran"
            )),
            "0.0",
            "{falsy}"
        );
    }
    assert_eq!(
        evaluate("class Box\nbox = Box()\nran = 0\nwhile box.open: ran = 1\nran"),
        "0.0"
    );
    assert_eq!(
        evaluate(
            "countdown = 3\nticks = 0\nwhile countdown:\n    countdown = countdown - 1\n    ticks = ticks + 1\n[countdown, ticks]"
        ),
        "[0.0,3.0]"
    );
    assert_eq!(
        evaluate(
            "name = \"abc\"\nsteps = 0\nwhile name:\n    name = name.slice(1)\n    steps = steps + 1\nsteps"
        ),
        "3.0"
    );
    assert_eq!(evaluate("n = 0\nwhile n < 3: n = n + 1\nn"), "3.0");
    assert_eq!(evaluate("n = 0\nwhile n < 3: n = n + 1"), "null");
}

#[test]
fn each_iteration_sees_the_propagation_its_assignments_cause() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
level = 0
doubled = level * 2
seen = []
while doubled < 6:
    level = level + 1
    seen.push(doubled)
assert(level, 3)
assert(seen, [2, 4, 6])
"#,
    );
    checked(
        &mut runtime,
        r#"
class Cell(size):
    this.size = size
$Cell.big = $Cell.size > 1
big = Cell.filter(cell => cell.big).length
while big < 2:
    Cell(Cell.length)
assert(Cell.length, 4)
assert(big, 2)
"#,
    );
}

#[test]
fn loops_leave_no_standing_rule_and_never_rerun() {
    let mut runtime = Runtime::new();
    run(&mut runtime, "limit = 3\ncount = 0\ntrace = []");
    let before = keys(&runtime);
    run(
        &mut runtime,
        "while count < limit:\n    count = count + 1\n    if count > 1:\n        trace.push(count)\n    {\n        seen = count\n    }",
    );
    assert_eq!(keys(&runtime), before);
    checked(
        &mut runtime,
        "assert(count, 3)\nassert(trace, [2, 3])\nassert(seen, 3)\nlimit = 10\ncount = 0\nassert(count, 0)\nassert(trace, [2, 3])\nassert(seen, 3)",
    );
}

#[test]
fn bodies_run_in_the_enclosing_scope_like_if_branches() {
    assert_eq!(
        evaluate(
            "def total(n):\n    sum = 0\n    while n > 0:\n        sum = sum + n\n        n = n - 1\n    return sum\ntotal(4)"
        ),
        "10.0"
    );
    assert_eq!(
        evaluate(
            "class Item\nItem()\nfor item of Item:\n    total = 0\n    while total < 3:\n        total = total + 1\ntotal"
        ),
        "3.0"
    );
    assert_eq!(
        evaluate(
            "class Item\nItem()\nItem()\ndef counted():\n    sum = 0\n    for item of Item:\n        sum = sum + 1\n    return sum\ncounted()"
        ),
        "0.0"
    );
    assert_eq!(
        evaluate("n = 0\nwhile n < 1:\n    n = n + 1\n    {\n        n = 5\n    }\nn"),
        "5.0"
    );
    assert_eq!(
        evaluate(
            "def countdown(n):\n    while n > 0: n = n - 1\n    return n\n[countdown(3), countdown(0)]"
        ),
        "[0.0,0.0]"
    );
    assert_eq!(
        evaluate(
            "class Counter:\n    def tick(times):\n        while times > 0:\n            self.count = (this.count ?? 0) + 1\n            times = times - 1\n        return this.count\ncounter = Counter()\n[counter.tick(3), counter.tick(2)]"
        ),
        "[3.0,5.0]"
    );
    assert_eq!(
        evaluate(
            "class Ladder(height):\n    this.rungs = []\n    while this.rungs.length < height:\n        this.rungs.push(this.rungs.length)\nshort = Ladder(2)\ntall = Ladder(4)\n[short.rungs, tall.rungs]"
        ),
        "[[0.0,1.0],[0.0,1.0,2.0,3.0]]"
    );
    assert_eq!(
        failure("while this.ready: pass"),
        (
            ErrorKind::Reference,
            "this is not defined".to_string(),
            Some(Position::new(1, 1))
        )
    );
}

#[test]
fn returns_leave_the_enclosing_function_block_loop_or_program() {
    assert_eq!(
        evaluate(
            "def root(limit, k = 1):\n    while true:\n        if k * k > limit:\n            return k\n        k = k + 1\nroot(50)"
        ),
        "8.0"
    );
    let mut runtime = Runtime::new();
    assert_eq!(
        json(
            &mut runtime,
            "n = 0\nwhile true:\n    n = n + 1\n    if n == 3: return n\nn = 99"
        ),
        "3.0"
    );
    assert_eq!(json(&mut runtime, "n"), "3.0");
    assert_eq!(evaluate("{\n    while true: return 7\n}"), "7.0");
    assert_eq!(
        evaluate("class Task\nTask()\nfor task of Task:\n    while true: return \"out\""),
        "\"out\""
    );
}

#[test]
fn value_reads_are_current_at_each_test() {
    assert_eq!(evaluate("a = 1\nwhile a.value < 3: a = a + 1\na"), "3.0");
}

#[test]
fn loops_inside_standing_rules_run_whenever_those_rules_run() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
class Gauge
gauge = Gauge()
gauge.target = 2
gauge.level = 0
if gauge.target > 0:
    while gauge.level < gauge.target: gauge.level = gauge.level + 1
assert(gauge.level, 2)
gauge.target = 5
assert(gauge.level, 5)
def fill(n, level = 0):
    while level < n: level = level + 2
    return level
target = 3
filled = fill(target)
assert(filled, 4)
target = 7
assert(filled, 8)
"#,
    );
}

#[test]
fn class_references_follow_for_statements() {
    assert_eq!(
        evaluate("class Item\nItem()\nn = 0\nwhile $Item.length > n: n = n + 1\nn"),
        "1.0"
    );
    assert_eq!(
        evaluate("class Item\nn = 0\nwhile $Item.length > n: n = n + 1\nItem()\nItem()\nn"),
        "0.0"
    );
    assert_eq!(
        evaluate(
            "class Item\nfirst = Item()\nn = 0\nwhile n < 1:\n    $Item.flag = true\n    n = n + 1\nsecond = Item()\n[first.flag, second.flag]"
        ),
        "[true,true]"
    );
    assert_eq!(
        evaluate(
            "class Tank(level):\n    this.level = level\nif $Tank.level < 3:\n    while $Tank.level < 3: $Tank.level = $Tank.level + 1\nlow = Tank(0)\nhigh = Tank(5)\n[low.level, high.level]"
        ),
        "[3.0,5.0]"
    );
    assert_eq!(
        failure("while $Missing.ready: pass"),
        (
            ErrorKind::Reference,
            "Missing is not defined".to_string(),
            Some(Position::new(1, 1))
        )
    );
}

#[test]
fn the_iteration_bound_is_per_run_and_rolls_back() {
    assert_eq!(
        evaluate("n = 0\nwhile n < 100000: n = n + 1\nn"),
        "100000.0"
    );

    let mut runtime = Runtime::new();
    run(&mut runtime, "n = 5\ndoubled = n * 2");
    let mut control = Runtime::new();
    run(&mut control, "n = 5\ndoubled = n * 2");
    let error = runtime
        .run("n = 0\nwhile n < 100001: n = n + 1\nn")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Loop did not terminate");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    runtime_equivalence::assert_equivalent(&runtime, None, &control, None);

    assert_eq!(
        evaluate(
            "outer = 0\ninner = 0\nwhile outer < 2:\n    inner = 0\n    while inner < 50001: inner = inner + 1\n    outer = outer + 1\n[outer, inner]"
        ),
        "[2.0,50001.0]"
    );
    assert_eq!(
        failure("def spin():\n    while true: pass\nspin()"),
        (
            ErrorKind::Type,
            "Loop did not terminate".to_string(),
            Some(Position::new(3, 1))
        )
    );
    assert_eq!(
        evaluate(
            "level = 12\ntry:\n    while level != 20: level = level + 3\ncatch error:\n    failure = error\n[failure, level]"
        ),
        "[\"TypeError: Loop did not terminate\",12.0]"
    );
}

#[test]
fn malformed_loops_report_exact_errors() {
    for (source, message, line, column) in [
        ("n = 0\nwhile n < 3:", "Unexpected end of input", 2, 13),
        (
            "n = 0\nwhile n < 3:\n",
            "Expected an indented block but found end of input",
            3,
            1,
        ),
        (
            "n = 0\nwhile n < 3:\nn = n + 1",
            "Expected an indented block but found identifier 'n'",
            3,
            1,
        ),
        (
            "n = 0\nwhile n < 3:\n    # only a comment\n",
            "Expected an indented block but found end of input",
            4,
            1,
        ),
        ("n = 0\nwhile n < 3:;", "Unexpected ';'", 2, 14),
        ("n = 0\nwhile n < 3:: pass", "Unexpected ':'", 2, 15),
        ("n = 0\nwhile n < 3 n = 1: pass", "Unexpected ':'", 2, 20),
        ("while x.lambda: tick()", "Unexpected ':'", 1, 17),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    assert_eq!(
        failure("while missing: pass"),
        (
            ErrorKind::Reference,
            "missing is not defined".to_string(),
            Some(Position::new(1, 1))
        )
    );
    let error = Runtime::new()
        .run("n = 0\nwhile true:\n    n = n + 1\n    throw \"STOP\"")
        .unwrap_err();
    assert_eq!(
        error.thrown_value(),
        Some(&Value::String("STOP".to_string()))
    );
    assert_eq!(error.position(), Some(Position::new(2, 1)));
}

#[test]
fn failed_iterations_restore_state_and_graph() {
    let initial = r#"
class Parcel
stock = 16
shipped = 0
left = stock - shipped
assert(left, 16)
"#;
    let mut actual = Runtime::new();
    checked(&mut actual, initial);
    let mut control = Runtime::new();
    checked(&mut control, initial);
    for (source, thrown) in [
        (
            "while shipped < 20:\n    if left < 3:\n        throw \"SHORT\"\n    Parcel()\n    shipped = shipped + 3",
            Some("SHORT"),
        ),
        ("while missing > 0: shipped = 1", None),
        ("while shipped >= 0: shipped = shipped + 1", None),
    ] {
        let error = actual.run(source).unwrap_err();
        assert_eq!(
            error.thrown_value(),
            thrown
                .map(|value| Value::String(value.to_string()))
                .as_ref(),
            "{source}"
        );
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    }
    checked(
        &mut actual,
        "while shipped < 6:\n    Parcel()\n    shipped = shipped + 3\nassert(Parcel.length, 2)\nassert(left, 10)",
    );
}

#[test]
fn newer_statements_keep_their_readings_of_while() {
    equivalent("while suppose: x", "while\nsuppose: x");
    equivalent("while suppose:\n    x = 1", "while\nsuppose:\n    x = 1");
    equivalent("while x = suppose: x", "while\nx = suppose: x");
    equivalent("while x = (suppose): x", "while\nx = suppose: x");
    equivalent("while suppose x > 1: x", "while\nsuppose x > 1: x");
    equivalent("while x := 1", "while\nx := 1");
    assert_eq!(
        parse("while := 1").unwrap(),
        [Stmt::ValueAssign {
            target: name("while"),
            value: Expr::Number(1.0),
        }]
    );
    assert!(matches!(
        parse("for while, value of point:\n    while = value").unwrap().as_slice(),
        [Stmt::ForEntries { key, body, .. }] if key == "while" && !contains_while(body)
    ));
    assert!(!contains_while(
        &parse("suppose:\n    while = 1\n    while").unwrap()
    ));

    assert_eq!(evaluate("while = 5\nwhile suppose: 3"), "3.0");
    assert_eq!(evaluate("while = 5\nwhile x = suppose: 4\nx"), "4.0");
    assert_eq!(evaluate("while = 5\nwhile x = (suppose): 8\nx"), "8.0");
    let mut runtime = Runtime::new();
    assert_eq!(
        json(&mut runtime, "while = 5\nx = 1\nwhile suppose x = 9: x"),
        "9.0"
    );
    assert_eq!(json(&mut runtime, "x"), "1.0");
    assert_eq!(evaluate("while := 3\nwhile"), "3.0");
    assert_eq!(
        evaluate("while = 0\nseen = suppose:\n    while = while + 4\n    while\n[seen, while]"),
        "[4.0,0.0]"
    );
    assert_eq!(
        evaluate(
            "point = {while: 1, b: 2}\nseen = []\nfor while, value of point: seen.push(while + value)\nseen"
        ),
        "[\"while1\",\"b2\"]"
    );
    for (source, column) in [
        ("while (suppose): pass", 18),
        ("while (x = suppose): pass", 22),
    ] {
        assert_eq!(
            syntax_error(source),
            ("Unexpected ':'".to_string(), Position::new(1, column)),
            "{source}"
        );
    }
    assert_eq!(
        failure("while (a) = suppose: 6"),
        (
            ErrorKind::Syntax,
            "Cannot assign to while(a)".to_string(),
            Some(Position::new(1, 1))
        )
    );
}

#[test]
fn loops_run_inside_and_around_newer_statements() {
    assert_eq!(
        evaluate("n = 0\nm = suppose:\n    while n < 3: n = n + 1\n    n\n[m, n]"),
        "[3.0,0.0]"
    );
    assert_eq!(
        evaluate(
            "n = 0\ntry:\n    suppose:\n        while true: n = n + 1\ncatch error:\n    failure = error\n[failure, n]"
        ),
        "[\"TypeError: Loop did not terminate\",0.0]"
    );
    assert_eq!(
        evaluate(
            "n = 0\nseen = []\nwhile n < 2:\n    n = n + 1\n    v = suppose:\n        n = n * 10\n        n\n    seen.push(v)\n[seen, n]"
        ),
        "[[10.0,20.0],2.0]"
    );
    assert_eq!(evaluate("n = 0\nwhile n < 3: n := n + 1\nn"), "3.0");
    assert_eq!(
        evaluate(
            "o = {a: 1, b: 2}\ntotal = 0\nwhile total < 9:\n    for k, v of o: total = total + v\ntotal"
        ),
        "9.0"
    );
    assert_eq!(evaluate("n = 2\nwhile n < 100: n **= 2\nn"), "256.0");
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
fn the_native_repl_buffers_while_headers_until_a_blank_line() {
    let output = repl(
        "count = 0\nwhile count < 3:\n    count = count + 1\n\ncount\nwhile count < 6: count = count + 1\ncount\nwhile ready: tick\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{BANNER}0.0\nnull\n3.0\nnull\n6.0\n")
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "ReferenceError: while is not defined (line 1, column 1)\n"
    );
}
