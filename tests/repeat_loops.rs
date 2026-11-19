mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::collections::BTreeSet;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::{generate, generate_all};
use nucleoid::lang::ast::{BinaryOp, Expr, FunctionBody, Stmt, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime, Value};
use serde::Deserialize;

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid repeats a block a counted number of times",
    "Nucleoid evaluates a repeat count once and truncates it",
    "Nucleoid runs repeat loops in constructors and methods with return",
    "Nucleoid nests repeat loops and rolls back a failed iteration",
];
const REFERENCE_TITLE: &str = "Nucleoid repeats statements a counted number of times";
const README_TITLE: &str = "Repeat loops advance a world model a counted number of steps";
const EXAMPLES_TITLE: &str = "Repeat loops build the cars of every train";

const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid repeats a block a counted number of times",
    "source": "# Nucleoid repeats a block a counted number of times\n\nrabbits = 2\nfood = rabbits * 3\ngenerations = 3\nwhile generations:\n    rabbits = rabbits * 2\n    generations -= 1\nassert(rabbits, 16)\nassert(food, 48)\ngenerations = 5\nassert(rabbits, 16)\nfor _ of \"aa\": rabbits = rabbits + food\nassert([rabbits, food], [256, 768])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid evaluates a repeat count once and truncates it",
    "source": "# Nucleoid evaluates a repeat count once and truncates it\n\nteam = [\"ana\", \"bo\"]\nsize = team.length\nfor _ of \"a\".repeat(size):\n    team.push(\"hire\")\nassert(team, [\"ana\", \"bo\", \"hire\", \"hire\"])\nassert(size, 4)\ndef culture(cells, hours):\n    total = cells\n    while hours >= 1:\n        total = total * 2\n        hours -= 1\n    return total\nassert([culture(1, 3), culture(1, 2.9), culture(1, \"3\"), culture(1, true)], [8, 4, 8, 2])\nassert([culture(5, 0), culture(5, -2), culture(5, null)], [5, 5, 5])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid runs repeat loops in constructors and methods with return",
    "source": "# Nucleoid runs repeat loops in constructors and methods with return\n\nclass Shelf(levels):\n    this.bins = []\n    while levels:\n        this.bins.push(this.bins.length * 10)\n        levels -= 1\n    def take(count):\n        weight = 0\n        while count:\n            if this.bins.length == 0:\n                return \"short\"\n            weight = weight + this.bins.pop()\n            count -= 1\n        return weight\nshelf = Shelf(3)\nassert(shelf.bins, [0, 10, 20])\nassert(shelf.take(2), 30)\nassert(shelf.take(2), \"short\")\nassert(shelf.bins, [])",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid nests repeat loops and rolls back a failed iteration",
    "source": "# Nucleoid nests repeat loops and rolls back a failed iteration\n\nclass Crate(row):\n    this.row = row\nrows = 2\nfor _ of \"a\".repeat(rows):\n    for _ of \"aaa\": Crate(rows)\n    rows = rows + 1\nassert(Crate.map(crate -> crate.row), [2, 2, 2, 3, 3, 3])\nloaded = Crate.length\ntry:\n    for _ of \"aa\":\n        Crate(0)\n        if loaded > 7:\n            throw \"FULL\"\ncatch error:\n    failure = error\nassert(failure, \"FULL\")\nassert([loaded, rows], [6, 4])",
    "expected": null
  },
  {
    "document": "reference",
    "title": "Nucleoid repeats statements a counted number of times",
    "source": "# Nucleoid repeats statements a counted number of times\n\nclass Account(balance):\n    this.balance = balance\nsavings = Account(100)\nyears = 3\nwhile years:\n    savings.balance = savings.balance * 1.5\n    years -= 1\nassert(savings.balance, 337.5)\nyears = 10\nsavings.balance",
    "expected": "337.5"
  },
  {
    "document": "readme",
    "title": "Repeat loops advance a world model a counted number of steps",
    "source": "class Cup(temp):\n    this.temp = temp\ntea = Cup(90)\nroom = 20\ncool = tea.temp < 40\nminutes = 3\nwhile minutes:\n    tea.temp = tea.temp - (tea.temp - room) / 2\n    minutes -= 1\nassert(tea.temp, 28.75)\nassert(cool, true)\n",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Repeat loops build the cars of every train",
    "source": "class Car(train):\n    this.train = train\n    this.seats = 40\nclass Train(cars):\n    while cars:\n        Car(this)\n        cars -= 1\n$Train.capacity = Car.filter(car -> car.train == $Train).sum(car -> car.seats)\nlocal = Train(2)\nexpress = Train(3)\nassert([local.capacity, express.capacity], [80, 120])\nCar(local)\n[local.capacity, express.capacity]\n",
    "expected": "[120,120]"
  }
]"###;

const RESPELLED: [(&str, &str); 12] = [
    (
        "repeat generations:\n    rabbits = rabbits * 2\n",
        "while generations:\n    rabbits = rabbits * 2\n    generations -= 1\n",
    ),
    (
        "repeat 2: rabbits = rabbits + food",
        "for _ of \"aa\": rabbits = rabbits + food",
    ),
    ("repeat size:", "for _ of \"a\".repeat(size):"),
    (
        "    repeat hours:\n        total = total * 2\n",
        "    while hours >= 1:\n        total = total * 2\n        hours -= 1\n",
    ),
    (
        "    repeat levels:\n        this.bins.push(this.bins.length * 10)\n",
        "    while levels:\n        this.bins.push(this.bins.length * 10)\n        levels -= 1\n",
    ),
    ("        repeat count:\n", "        while count:\n"),
    (
        "            weight = weight + this.bins.pop()\n",
        "            weight = weight + this.bins.pop()\n            count -= 1\n",
    ),
    (
        "repeat rows:\n    repeat 3: Crate(rows)",
        "for _ of \"a\".repeat(rows):\n    for _ of \"aaa\": Crate(rows)",
    ),
    (
        "    repeat 2:\n        Crate(0)",
        "    for _ of \"aa\":\n        Crate(0)",
    ),
    (
        "repeat years:\n    savings.balance = savings.balance * 1.5\n",
        "while years:\n    savings.balance = savings.balance * 1.5\n    years -= 1\n",
    ),
    (
        "repeat minutes:\n    tea.temp = tea.temp - (tea.temp - room) / 2\n",
        "while minutes:\n    tea.temp = tea.temp - (tea.temp - room) / 2\n    minutes -= 1\n",
    ),
    (
        "    repeat cars:\n        Car(this)\n",
        "    while cars:\n        Car(this)\n        cars -= 1\n",
    ),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedCase {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

fn documents() -> [(&'static str, Vec<common::Case>); 4] {
    [
        ("spec", common::cases(include_str!("../nucleoid.spec.md"))),
        ("reference", common::cases(include_str!("reference.md"))),
        (
            "readme",
            common::snippets(include_str!("../README.md"), "README.md"),
        ),
        (
            "examples",
            common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
        ),
    ]
}

fn is_appended(title: &str) -> bool {
    SPEC_TITLES.contains(&title) || [REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE].contains(&title)
}

fn contains_repeat(statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| match statement {
        Stmt::Repeat { .. } => true,
        Stmt::If {
            consequent,
            alternate,
            ..
        } => {
            contains_repeat(consequent)
                || alternate
                    .as_ref()
                    .is_some_and(|statement| contains_repeat(std::slice::from_ref(statement)))
        }
        Stmt::Block(body)
        | Stmt::For { body, .. }
        | Stmt::ForEntries { body, .. }
        | Stmt::While { body, .. }
        | Stmt::Suppose { body, .. } => contains_repeat(body),
        Stmt::Try { body, catch, .. } => contains_repeat(body) || contains_repeat(catch),
        Stmt::Function(function) => {
            matches!(&function.body, FunctionBody::Block(body) if contains_repeat(body))
        }
        Stmt::Class(class) => contains_repeat(&class.constructor)
            || class.methods.iter().any(
                |method| matches!(&method.body, FunctionBody::Block(body) if contains_repeat(body)),
            ),
        _ => false,
    })
}

fn repeats_anywhere(statements: &[Stmt]) -> bool {
    format!("{statements:?}").contains("Repeat { count: ")
}

fn respelled(case: &common::Case) -> (String, usize) {
    let mut written = case.source.clone();
    let mut rewrites = 0;
    for (repeat, accepted) in RESPELLED {
        rewrites += written.matches(repeat).count();
        written = written.replace(repeat, accepted);
    }
    (written, rewrites)
}

fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "repeat loop regression".to_string(),
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

fn keys(runtime: &Runtime) -> BTreeSet<String> {
    runtime
        .graph
        .keys()
        .map(|key| key.as_str().to_string())
        .collect()
}

#[test]
fn only_the_appended_cases_repeat_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let statements =
            parse(&case.source).unwrap_or_else(|error| panic!("{}: {error}", case.title));
        let new = is_appended(&case.title);
        assert_eq!(contains_repeat(&statements), new, "{}", case.title);
        assert_eq!(repeats_anywhere(&statements), new, "{}", case.title);
        if new {
            let count = common::assertions::protected_assertions(&statements).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            let mut runtime = Runtime::new();
            common::check_with_count(&mut runtime, case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assert_eq!(runtime.assertions_run(), count, "{}", case.title);
            assertions += count;
        }
    }
    for title in SPEC_TITLES
        .into_iter()
        .chain([REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE])
    {
        let matching = documents
            .iter()
            .flat_map(|(_, cases)| cases)
            .filter(|case| case.title == title)
            .count();
        assert_eq!(matching, 1, "{title}");
    }
    assert_eq!(assertions, 19);
    assert_eq!(cases, 1216);
}

#[test]
fn the_appended_cases_stand_after_nuc_114_and_before_nuc_103() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(titles[776], "Nucleoid rolls back failed product updates");
    assert_eq!(titles[777..781], SPEC_TITLES);
    assert_eq!(titles[808], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            &reference,
            REFERENCE_TITLE,
            "Nucleoid multiplies lists and populations with product",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            &readme,
            README_TITLE,
            "Products keep the chance that a trip connects live as flights change",
            "Flat turns nested lists into one live list",
        ),
        (
            &examples,
            EXAMPLES_TITLE,
            "Products keep every account's compounded balance live",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(index, cases.len() - 12, "{title}");
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 9].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### repeat\n\n```\n")
        .expect("the language reference documents repeat loops");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 115](nuc-0115.md)"));
    assert_eq!(case.expected.as_deref(), Some("337.5"));
}

#[test]
fn every_case_is_its_accepted_counterpart_with_only_the_loops_respelled() {
    let sealed: Vec<SealedCase> = serde_json::from_str(ACCEPTED).unwrap();
    let documents = documents();
    assert_eq!(sealed.len(), 7);
    let mut rewrites = 0;
    for sealed in &sealed {
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == sealed.document)
            .unwrap()
            .1;
        let matching: Vec<&common::Case> = cases
            .iter()
            .filter(|case| case.title == sealed.title)
            .collect();
        assert_eq!(matching.len(), 1, "{}", sealed.title);
        let case = matching[0];
        assert_eq!(case.expected, sealed.expected, "{}", sealed.title);
        let (written, count) = respelled(case);
        assert_eq!(written, sealed.source, "{}", sealed.title);
        assert!(
            !contains_repeat(&parse(&written).unwrap()),
            "{}",
            sealed.title
        );
        rewrites += count;
        let accepted = common::assertions::protected_assertions(&parse(&sealed.source).unwrap());
        let candidate = common::assertions::protected_assertions(&parse(&case.source).unwrap());
        assert_eq!(accepted, candidate, "{}", sealed.title);
        let control = common::Case {
            title: sealed.title.clone(),
            source: sealed.source.clone(),
            expected: sealed.expected.clone(),
        };
        for form in [case, &control] {
            let mut runtime = Runtime::new();
            common::check_with_count(&mut runtime, form, accepted.len())
                .unwrap_or_else(|reason| panic!("{}: {reason}", sealed.title));
            assert_eq!(runtime.assertions_run(), accepted.len(), "{}", sealed.title);
        }
    }
    assert_eq!(rewrites, 12);
}

#[test]
fn repeat_loops_build_one_statement_with_canonical_renderings() {
    let statements = parse("repeat 3:\n    n = n + 1").unwrap();
    assert_eq!(
        statements,
        [Stmt::Repeat {
            count: Expr::Number(3.0),
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
    assert_eq!(generate_all(&statements), "repeat(3){n=n+1}");

    equivalent("repeat 3: n = n + 1", "repeat 3:\n    n = n + 1");
    equivalent(
        "repeat 3: n = n + 1; seen = n",
        "repeat 3:\n    n = n + 1\n    seen = n",
    );
    equivalent(
        "repeat (3): n = n + 1",
        "repeat 3:\n\n    # comment\n    n = n + 1",
    );
    equivalent("repeat(3): n = n + 1", "repeat 3: n = n + 1");
    assert_eq!(
        generate_all(&parse("repeat a:\n    repeat b: c()\n    d = 1").unwrap()),
        "repeat(a){repeat(b){c()};d=1}"
    );
    assert_eq!(
        generate_all(&parse("repeat ready ? 2 : 3: tick()").unwrap()),
        "repeat((ready?2:3)){tick()}"
    );
    assert_eq!(
        generate_all(&parse("repeat n + 1: tick()").unwrap()),
        "repeat(n+1){tick()}"
    );
    assert_eq!(
        NodeKey::block(&parse("repeat 2: n = n + 1").unwrap(), None).as_str(),
        "block(repeat(2){n=n+1})"
    );

    let source = "def step(n = 3):\n    repeat n:\n        n = n - 1\n    return n";
    let function = parse(source).unwrap();
    let rendered = generate(&function[0]);
    assert_eq!(
        rendered,
        "def step(n=3):\n    repeat n:\n        n=n-1\n    return n"
    );
    assert_eq!(parse(&rendered).unwrap(), function);
}

#[test]
fn repeat_headers_follow_while_headers() {
    for source in [
        "while n < 3:\n    n = n + 1",
        "while n < 3: n = n + 1",
        "while (ready): pass",
        "while -n: pass",
        "while [n]: pass",
        "while x if y else z: pass",
        "while all v in xs: v > 0: pass",
        "while ready ? on : off: pass",
        "while [lambda: 1]: pass",
        "while ready: tick()",
        "while ready: tick",
        "while assert: ready, true",
        "while lambda: 1",
        "while ready + lambda: 1",
        "while all v in xs: v",
        "while n < 3 {\n    n = n + 1\n}",
        "while n < 3\n    n = n + 1",
        "while suppose: x",
        "while x = suppose: x",
        "while x = suppose + 1: pass",
        "while x := 1",
        "while x: y := 1",
        "if ready: while n < 3: n = n + 1",
        "def f(k):\n    while k: k = k - 1\n    return k",
        "class Box(size):\n    while size: size = size - 1",
        "{\n    while n: n = n - 1\n}",
        "v = 1\nwhile v is x: T",
        "def f() => 1\nwhile f() is x: T",
        "def f() => 1\nwhile f() is assert: a, b",
        "x = 3\nwhile x is 3: pass",
        "while v is suppose:\n    a = 1",
        "t = `${n}`\nwhile n: n = n - 1",
        "xs = [x for x in ys]\nwhile n: n = n - 1",
    ] {
        let spelled = source.replace("while", "repeat");
        let accepted = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        let repeated = parse(&spelled).unwrap_or_else(|error| panic!("{spelled}: {error}"));
        assert_eq!(
            format!("{repeated:?}"),
            format!("{accepted:?}")
                .replace("While { condition:", "Repeat { count:")
                .replace("\"while\"", "\"repeat\""),
            "{spelled}"
        );
        assert_eq!(
            generate_all(&repeated),
            generate_all(&accepted).replace("while", "repeat"),
            "{spelled}"
        );
    }
}

#[test]
fn repeat_keeps_every_accepted_reading_of_its_name() {
    equivalent("repeat ready: tick", "repeat\nready: tick");
    equivalent("repeat ready: tick;", "repeat\nready: tick;");
    equivalent("repeat assert: ready, true", "repeat\nassert: ready, true");
    equivalent("repeat lambda: 1", "repeat\nlambda: 1");
    equivalent("repeat ready + lambda: 1", "repeat\nready + lambda: 1");
    equivalent("repeat not lambda: 1", "repeat\nnot lambda: 1");
    equivalent("repeat x => lambda: 1", "repeat\nx => lambda: 1");
    equivalent("repeat lambda x: x", "repeat\nlambda x: x");
    equivalent(
        "repeat ready if done: tick()",
        "repeat\nready\nif done: tick()",
    );
    equivalent("repeat all v in xs: v", "repeat\nall v in xs: v");
    equivalent(
        "repeat n < 3 {\n    n = n + 1\n}",
        "repeat\nn < 3\n{\n    n = n + 1\n}",
    );
    equivalent("repeat n < 3\n    n = n + 1", "repeat\nn < 3\nn = n + 1");
    equivalent("repeat (n) b: int", "repeat(n)\nb: int");
    equivalent(
        "class Gate:\n    repeat open: int",
        "class Gate:\n    repeat\n    open: int",
    );
    equivalent("if ready: repeat open: int", "if ready: repeat\nopen: int");
    equivalent("repeat suppose: x", "repeat\nsuppose: x");
    equivalent("repeat suppose:\n    x = 1", "repeat\nsuppose:\n    x = 1");
    equivalent("repeat x = suppose: x", "repeat\nx = suppose: x");
    equivalent("repeat x = (suppose): x", "repeat\nx = suppose: x");
    equivalent("repeat suppose x > 1: x", "repeat\nsuppose x > 1: x");
    equivalent("repeat x := 1", "repeat\nx := 1");

    assert_eq!(
        parse("repeat(4)").unwrap(),
        [Stmt::Expression(Expr::Call {
            callee: Box::new(name("repeat")),
            arguments: vec![Expr::Number(4.0)],
        })]
    );
    assert_eq!(
        parse("repeat: int").unwrap(),
        [Stmt::Declaration {
            name: "repeat".to_string(),
            type_name: "int".to_string(),
        }]
    );
    assert_eq!(
        parse("repeat := 1").unwrap(),
        [Stmt::ValueAssign {
            target: name("repeat"),
            value: Expr::Number(1.0),
        }]
    );
    assert_eq!(
        parse_expression("repeat 3: tick()").unwrap(),
        name("repeat")
    );
    assert!(!contains_repeat(
        &parse(
            "for repeat of items:\n    point = {repeat: repeat}\n    point.repeat = \"ab\".repeat(2)"
        )
        .unwrap()
    ));
    assert!(matches!(
        parse("for repeat, value of point:\n    repeat = value").unwrap().as_slice(),
        [Stmt::ForEntries { key, body, .. }] if key == "repeat" && !contains_repeat(body)
    ));

    assert_eq!(evaluate("repeat = 2\nrepeat"), "2.0");
    assert_eq!(evaluate("def repeat(x) => x + 1\nrepeat(4)"), "5.0");
    assert_eq!(evaluate("def twice(repeat) => repeat * 2\ntwice(4)"), "8.0");
    assert_eq!(evaluate("repeat = 3\nn = repeat + 1\nn"), "4.0");
    assert_eq!(
        evaluate("repeat = 1\nrepeat assert: repeat, 1\nrepeat lambda: 2"),
        "\"[function]\""
    );
    assert_eq!(evaluate("\"ab\".repeat(2)"), "\"abab\"");
    assert_eq!(evaluate("o = {repeat: 2}\no.repeat"), "2.0");
    assert_eq!(evaluate("repeat = 2\nrepeat := 5\nrepeat"), "5.0");
    assert_eq!(evaluate("repeat = 5\nrepeat suppose: 3"), "3.0");
    assert_eq!(evaluate("repeat = 5\nrepeat x = suppose: 4\nx"), "4.0");
    assert_eq!(
        evaluate("repeat = 7\nn = 1\nrepeat n < 3 {\n    n = n + 1\n}\nn"),
        "2.0"
    );
    assert_eq!(
        evaluate("repeat = 7\nn = 1\nrepeat n < 3\n    n = n + 1\nn"),
        "2.0"
    );
    assert_eq!(
        evaluate("class Item\nItem()\nn = 0\nfor repeat of Item:\n    n = n + 1\nn"),
        "1.0"
    );
    assert_eq!(
        evaluate(
            "point = {a: 1}\nkeys = []\nfor repeat, value of point:\n    keys.push(repeat)\nkeys"
        ),
        "[\"a\"]"
    );
    assert_eq!(
        failure("repeat n: int"),
        (
            ErrorKind::Reference,
            "repeat is not defined".to_string(),
            Some(Position::new(1, 1))
        )
    );
}

#[test]
fn the_kept_readings_keep_their_exact_accepted_results() {
    for (source, kind, message, line, column) in [
        (
            "repeat ready: tick",
            ErrorKind::Reference,
            "repeat is not defined",
            1,
            1,
        ),
        (
            "repeat = 1\nrepeat ready: tick",
            ErrorKind::Reference,
            "Missing definition",
            2,
            8,
        ),
        (
            "n = 0\nrepeat n < 3 {\n    n = n + 1\n}",
            ErrorKind::Reference,
            "repeat is not defined",
            2,
            1,
        ),
        (
            "repeat = 1\nrepeat (a) = suppose: 6",
            ErrorKind::Syntax,
            "Cannot assign to repeat(a)",
            2,
            1,
        ),
        (
            "repeat: int",
            ErrorKind::Reference,
            "Missing definition",
            1,
            1,
        ),
        (
            "def f() => 1\nrepeat f() is x: T",
            ErrorKind::Reference,
            "repeat is not defined",
            2,
            1,
        ),
        (
            "def f() => 1\nrepeat f() is assert: a, b",
            ErrorKind::Reference,
            "repeat is not defined",
            2,
            1,
        ),
    ] {
        assert_eq!(
            failure(source),
            (kind, message.to_string(), Some(Position::new(line, column))),
            "{source}"
        );
    }
    for (source, line, column) in [
        ("repeat x.lambda: tick()", 1, 18),
        ("n = 0\nrepeat n < 3 n = 1: pass", 2, 21),
        ("repeat (suppose): pass", 1, 19),
        ("repeat (x = suppose): pass", 1, 23),
    ] {
        assert_eq!(
            syntax_error(source),
            ("Unexpected ':'".to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    assert_eq!(
        evaluate("repeat = 1\nx = 2\nrepeat x + lambda: 1"),
        evaluate("repeat = 1\nx = 2\nrepeat\nx + lambda: 1")
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
is = 4
value = 1
result = value is 4
assert(result, 1)
def increment(is) => is + 1
assert(increment(1), 2)
record = { k: value is: 3 }
assert(record.is, 3)
text = `${value is 4}`
assert(text, "1")
named = (value is is)
assert(named, false)
failure = ""
try:
    repeat value() is x: T
catch error:
    failure = error
assert(failure, ReferenceError("repeat is not defined"))
"#,
    );
    for (header, column) in [("repeat @a: a = a + 1", 9), ("while @a: a = a - 1", 8)] {
        assert_eq!(
            syntax_error(&format!("a = 2\n{header}")),
            (
                "Expected a non-negative integer, 'first', 'last', or 'previous' after '@'"
                    .to_string(),
                Position::new(2, column)
            ),
            "{header}"
        );
    }
}

#[test]
fn counts_convert_as_the_string_method_repeat_converts_them() {
    for (count, iterations) in [
        ("0", 0.0),
        ("1", 1.0),
        ("3", 3.0),
        ("2.9", 2.0),
        ("0.5", 0.0),
        ("-1", 0.0),
        ("-0.5", 0.0),
        ("\"3\"", 3.0),
        ("\" 3 \"", 3.0),
        ("\"abc\"", 0.0),
        ("\"\"", 0.0),
        ("true", 1.0),
        ("false", 0.0),
        ("null", 0.0),
        ("undefined", 0.0),
        ("holder.missing", 0.0),
        ("[2]", 2.0),
        ("[1, 2]", 0.0),
        ("[]", 0.0),
        ("{n: 2}", 0.0),
        ("Number.NaN", 0.0),
        ("0 / 0", 0.0),
        ("Number.NEGATIVE_INFINITY", 0.0),
        ("Date(5)", 5.0),
        ("100000", 100_000.0),
        ("100000.9", 100_000.0),
    ] {
        let mut runtime = Runtime::new();
        run(&mut runtime, "holder = {}\nn = 0");
        assert_eq!(
            run(&mut runtime, &format!("repeat {count}: n = n + 1")),
            Value::Null,
            "{count}"
        );
        assert_eq!(run(&mut runtime, "n"), Value::Number(iterations), "{count}");
        assert_eq!(
            run(&mut runtime, &format!("\"a\".repeat({count}).length")),
            Value::Number(iterations),
            "{count}"
        );
    }
}

#[test]
fn undefined_counts_leave_enclosing_conditions_settled_as_while_conditions_do() {
    for (statement, expected) in [
        ("repeat o.missing: n = n + 1", "true"),
        ("while o.missing: n = n + 1", "true"),
        ("n = o.missing", "null"),
    ] {
        assert_eq!(
            evaluate(&format!(
                "class Item\ndef check(o):\n    n = 0\n    {statement}\n    return true\nif check($Item):\n    $Item.ok = true\nitem = Item()\nitem.ok"
            )),
            expected,
            "{statement}"
        );
    }
    assert_eq!(
        evaluate("def f(o):\n    repeat o.missing: pass\n    return 3\nx = f({}) ?? 7\nx"),
        "3.0"
    );
}

#[test]
fn the_count_is_read_once_before_the_first_iteration() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
log = []
def next():
    log.push(log.length)
    return 3
n = 0
repeat next(): n = n + 1
assert(n, 3)
assert(log, [0])
queue = ["a"]
size = queue.length
repeat size + 1:
    queue.push(size)
assert(queue, ["a", 1, 2])
assert(size, 3)
box = {count: 2}
repeat box.count:
    box.count = box.count + 10
assert(box.count, 22)
k = 2
repeat k:
    k = k - 5
assert(k, -8)
"#,
    );
}

#[test]
fn the_bound_is_checked_before_the_first_iteration_and_rolls_back() {
    assert_eq!(evaluate("n = 0\nrepeat 100000: n = n + 1\nn"), "100000.0");
    assert_eq!(evaluate("n = 0\nrepeat 100000.9: n = n + 1\nn"), "100000.0");

    let mut runtime = Runtime::new();
    run(&mut runtime, "n = 5\ndoubled = n * 2\ntrace = []");
    let mut control = Runtime::new();
    run(&mut control, "n = 5\ndoubled = n * 2\ntrace = []");
    for (source, line) in [
        ("n = 0\nrepeat 100001: trace.push(n)\nn", 2),
        ("trace.push(1)\nrepeat 1 / 0: n = n + 1", 2),
        ("repeat Number.POSITIVE_INFINITY: n = 0", 1),
        ("repeat 1e300: n = 0", 1),
        ("repeat 100001: throw \"RAN\"", 1),
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{source}");
        assert_eq!(error.message(), "Repeat count is too large", "{source}");
        assert_eq!(error.position(), Some(Position::new(line, 1)), "{source}");
        assert_eq!(error.thrown_value(), None, "{source}");
        runtime_equivalence::assert_equivalent(&runtime, None, &control, None);
    }

    assert_eq!(
        evaluate("total = 0\nrepeat 2:\n    repeat 60000: total = total + 1\ntotal"),
        "120000.0"
    );
    assert_eq!(
        failure("def spin(k):\n    repeat k: pass\nspin(100001)"),
        (
            ErrorKind::Type,
            "Repeat count is too large".to_string(),
            Some(Position::new(3, 1))
        )
    );
    assert_eq!(
        evaluate(
            "level = 12\ntry:\n    repeat 100001: level = level + 3\ncatch error:\n    failure = error\n[failure, level]"
        ),
        "[\"TypeError: Repeat count is too large\",12.0]"
    );
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
repeat 3:
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
repeat 4:
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
        "repeat limit:\n    count = count + 1\n    if count > 1:\n        trace.push(count)\n    {\n        seen = count\n    }",
    );
    assert_eq!(keys(&runtime), before);
    checked(
        &mut runtime,
        "assert(count, 3)\nassert(trace, [2, 3])\nassert(seen, 3)\nlimit = 10\ncount = 0\nassert(count, 0)\nassert(trace, [2, 3])\nassert(seen, 3)",
    );
}

#[test]
fn bodies_run_in_the_enclosing_scope_like_while_bodies() {
    assert_eq!(
        evaluate(
            "def total(n):\n    sum = 0\n    repeat n:\n        sum = sum + n\n        n = n - 1\n    return sum\ntotal(4)"
        ),
        "10.0"
    );
    assert_eq!(
        evaluate(
            "def counted(k):\n    sum = 0\n    repeat k:\n        sum = sum + 1\n    return sum\ncounted(3)"
        ),
        "3.0"
    );
    assert_eq!(
        evaluate(
            "def counted(k):\n    sum = 0\n    for _ of \"a\".repeat(k):\n        sum = sum + 1\n    return sum\ncounted(3)"
        ),
        "0.0"
    );
    assert_eq!(
        evaluate(
            "class Item\nItem()\nfor item of Item:\n    total = 0\n    repeat 3:\n        total = total + 1\ntotal"
        ),
        "3.0"
    );
    assert_eq!(
        evaluate("n = 0\nrepeat 1:\n    n = n + 1\n    {\n        n = 5\n    }\nn"),
        "5.0"
    );
    assert_eq!(
        evaluate(
            "class Counter:\n    def tick(times):\n        repeat times:\n            self.count = (this.count ?? 0) + 1\n        return this.count\ncounter = Counter()\n[counter.tick(3), counter.tick(2)]"
        ),
        "[3.0,5.0]"
    );
    assert_eq!(
        evaluate(
            "class Ladder(height):\n    this.rungs = []\n    repeat height:\n        this.rungs.push(this.rungs.length)\nshort = Ladder(2)\ntall = Ladder(4)\n[short.rungs, tall.rungs]"
        ),
        "[[0.0,1.0],[0.0,1.0,2.0,3.0]]"
    );
    assert_eq!(
        failure("repeat this.ready: pass"),
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
            "def root(limit, k = 1):\n    repeat limit:\n        if k * k > limit:\n            return k\n        k = k + 1\nroot(50)"
        ),
        "8.0"
    );
    let mut runtime = Runtime::new();
    assert_eq!(
        json(
            &mut runtime,
            "n = 0\nrepeat 5:\n    n = n + 1\n    if n == 3: return n\nn = 99"
        ),
        "3.0"
    );
    assert_eq!(json(&mut runtime, "n"), "3.0");
    assert_eq!(evaluate("{\n    repeat 2: return 7\n}"), "7.0");
    assert_eq!(
        evaluate("class Task\nTask()\nfor task of Task:\n    repeat 3: return \"out\""),
        "\"out\""
    );
    assert_eq!(
        evaluate("def none():\n    repeat 0: return 1\n    return 2\nnone()"),
        "2.0"
    );
    assert_eq!(
        evaluate(
            "n = 0\nrepeat 2:\n    repeat 3:\n        n = n + 1\n        if n == 2: return n\nn"
        ),
        "2.0"
    );
}

#[test]
fn function_expression_bodies_run_repeat_loops_and_their_returns() {
    for (source, expected) in [
        (
            "count = (n) => {\n    total = 0\n    repeat n: total = total + 1\n    return total\n}\ncount(3)",
            "3.0",
        ),
        (
            "count = function(n) {\n    total = 0\n    repeat n:\n        total = total + 2\n    return total\n}\ncount(4)",
            "8.0",
        ),
        (
            "first = (xs) => {\n    repeat xs.length:\n        if xs[0] > 1: return xs[0]\n        xs.shift()\n    return null\n}\nfirst([1, 0, 5, 7])",
            "5.0",
        ),
        (
            "[1, 2, 3].map(k => {\n    total = 0\n    repeat k: total = total + 10\n    return total\n})",
            "[10.0,20.0,30.0]",
        ),
        (
            "[1, 2].map(k => {\n    n = k + 1\n    total = 0\n    repeat n: total = total + 1\n    return total\n})",
            "[2.0,3.0]",
        ),
        ("f = (n) => { repeat n: return n * 2 }\nf(4)", "8.0"),
        ("f = (n) => { repeat 0: return n * 2 }\nf(4)", "null"),
    ] {
        let statements = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        assert!(repeats_anywhere(&statements), "{source}");
        assert_eq!(evaluate(source), expected, "{source}");
    }
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
    repeat gauge.target: gauge.level = gauge.level + 1
assert(gauge.level, 2)
gauge.target = 3
assert(gauge.level, 5)
def fill(n):
    level = 0
    repeat n: level = level + 2
    return level
target = 3
filled = fill(target)
assert(filled, 6)
target = 5
assert(filled, 10)
"#,
    );
}

#[test]
fn class_references_follow_while_loops() {
    assert_eq!(
        evaluate("class Item\nItem()\nItem()\nn = 0\nrepeat $Item.length: n = n + 1\nn"),
        "2.0"
    );
    assert_eq!(
        evaluate("class Item\nn = 0\nrepeat $Item.length: n = n + 1\nItem()\nItem()\nn"),
        "0.0"
    );
    assert_eq!(
        evaluate(
            "class Item\nfirst = Item()\nrepeat 1: $Item.flag = true\nsecond = Item()\n[first.flag, second.flag]"
        ),
        "[true,true]"
    );
    assert_eq!(
        evaluate(
            "class Tank(level):\n    this.level = level\nif $Tank.level < 3:\n    repeat 2: $Tank.level = $Tank.level + 1\nlow = Tank(0)\nhigh = Tank(5)\n[low.level, high.level]"
        ),
        "[2.0,5.0]"
    );
    assert_eq!(
        failure("repeat $Missing.n: pass"),
        (
            ErrorKind::Reference,
            "Missing is not defined".to_string(),
            Some(Position::new(1, 1))
        )
    );
}

#[test]
fn malformed_loops_report_exact_errors() {
    for (source, message, line, column) in [
        ("n = 0\nrepeat 3:", "Unexpected end of input", 2, 10),
        (
            "n = 0\nrepeat 3:\n",
            "Expected an indented block but found end of input",
            3,
            1,
        ),
        (
            "n = 0\nrepeat 3:\nn = n + 1",
            "Expected an indented block but found identifier 'n'",
            3,
            1,
        ),
        (
            "n = 0\nrepeat 3:\n    # nothing here\n",
            "Expected an indented block but found end of input",
            4,
            1,
        ),
        (
            "n = 0\nrepeat 3:\n    # nothing here",
            "Expected an indented block but found end of input",
            3,
            19,
        ),
        ("n = 0\nrepeat 3:;", "Unexpected ';'", 2, 11),
        ("n = 0\nrepeat 3:: pass", "Unexpected ':'", 2, 12),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn runtime_errors_keep_the_position_of_the_running_statement() {
    for (source, kind, message, line) in [
        (
            "repeat missing: pass",
            ErrorKind::Reference,
            "missing is not defined",
            1,
        ),
        (
            "n = 0\nrepeat 100001: n = n + 1",
            ErrorKind::Type,
            "Repeat count is too large",
            2,
        ),
        (
            "def spin():\n    repeat 1 / 0: pass\nspin()",
            ErrorKind::Type,
            "Repeat count is too large",
            3,
        ),
        (
            "repeat $Missing.n: pass",
            ErrorKind::Reference,
            "Missing is not defined",
            1,
        ),
        (
            "repeat this.n: pass",
            ErrorKind::Reference,
            "this is not defined",
            1,
        ),
    ] {
        assert_eq!(
            failure(source),
            (kind, message.to_string(), Some(Position::new(line, 1))),
            "{source}"
        );
    }
    let error = Runtime::new().run("repeat 2: throw \"STOP\"").unwrap_err();
    assert_eq!(
        error.thrown_value(),
        Some(&Value::String("STOP".to_string()))
    );
    assert_eq!(error.position(), Some(Position::new(1, 1)));
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
            "repeat 7:\n    if left < 3:\n        throw \"SHORT\"\n    Parcel()\n    shipped = shipped + 3",
            Some("SHORT"),
        ),
        ("repeat missing: shipped = 1", None),
        ("repeat 100001: shipped = shipped + 1", None),
        (
            "repeat 3:\n    shipped = shipped + 1\n    missing.value = 1",
            None,
        ),
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
        "repeat 2:\n    Parcel()\n    shipped = shipped + 3\nassert(Parcel.length, 2)\nassert(left, 10)",
    );
}

#[test]
fn the_compatibility_rows_give_their_documented_results() {
    for (source, expected) in [
        ("n = 0\nrepeat 3:\n    n = n + 1\nn", "3.0"),
        (
            "years = 2\nbalance = 100\nrepeat years:\n    balance = balance * 1.5\nbalance",
            "225.0",
        ),
        ("n = 0\nrepeat 2: n = n + 1; n = n + 1\nn", "4.0"),
        ("n = 2\nrepeat (n):\n    n = n + 1\nn", "4.0"),
        ("n = 0\nrepeat 2.9:\n    n = n + 1\nn", "2.0"),
        (
            "class C:\n    def f(self, k):\n        repeat k: self.n = 1\n        return 1\nC().f(2)",
            "1.0",
        ),
        ("n = 0\nrepeat [1, 2].map(x -> x).length: n = n + 1", "null"),
        (
            "n = 0\nrepeat [1, 2].map(x -> x).length: n = n + 1\nn",
            "2.0",
        ),
        ("xs = [1]\nrepeat 1 in xs: xs.push(2)", "null"),
        ("xs = [1]\nrepeat 1 in xs: xs.push(2)\nxs", "[1.0,2.0]"),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
}

#[test]
fn loops_run_inside_and_around_newer_statements() {
    assert_eq!(
        evaluate("n = 0\nm = suppose:\n    repeat 3: n = n + 1\n    n\n[m, n]"),
        "[3.0,0.0]"
    );
    assert_eq!(
        evaluate(
            "n = 0\ntry:\n    suppose:\n        repeat 100001: n = n + 1\ncatch error:\n    failure = error\n[failure, n]"
        ),
        "[\"TypeError: Repeat count is too large\",0.0]"
    );
    assert_eq!(
        evaluate(
            "n = 0\nseen = []\nrepeat 2:\n    n = n + 1\n    v = suppose:\n        n = n * 10\n        n\n    seen.push(v)\n[seen, n]"
        ),
        "[[10.0,20.0],2.0]"
    );
    assert_eq!(evaluate("n = 0\nrepeat 3: n := n + 1\nn"), "3.0");
    assert_eq!(
        evaluate(
            "o = {a: 1, b: 2}\ntotal = 0\nrepeat 3:\n    for k, v of o: total = total + v\ntotal"
        ),
        "9.0"
    );
    assert_eq!(evaluate("n = 2\nrepeat 3: n **= 2\nn"), "256.0");
    assert_eq!(evaluate("x = 3\nn = 0\nrepeat x is 3: n = n + 1\nn"), "1.0");
    assert_eq!(
        evaluate("xs = [1, 5]\nys = xs.filter(> 4)\nrepeat 2: xs.push(9)\nys"),
        "[5.0,9.0,9.0]"
    );
    assert_eq!(
        evaluate("n = 2\nt = `${n}`\nrepeat n: n = n + 1\n[n, t]"),
        "[4.0,\"4\"]"
    );
    assert_eq!(
        evaluate(
            "price = 2\ntotal = price * 3\nsteps = total |> why\nrepeat 2: price = price + 1\ntotal"
        ),
        "12.0"
    );
    assert_eq!(
        evaluate("xs = [x for x in [1, 2]]\nrepeat 2: xs.push(0)\nxs"),
        "[1.0,2.0,0.0,0.0]"
    );
    assert_eq!(
        evaluate("{a, b} = {a: 2, b: 1}\nrepeat a: b = b + 1\nb"),
        "3.0"
    );
    assert_eq!(
        evaluate(
            "n = 0\nrepeat 2:\n    i = 0\n    while i < 3:\n        n = n + 1\n        i = i + 1\nn"
        ),
        "6.0"
    );
}

#[test]
fn why_reports_what_the_loop_left_as_its_while_spelling_does() {
    let looped = evaluate("base = 2\ntotal = 0\nrepeat 3:\n    total = total + base\nwhy total");
    let counted = evaluate(
        "base = 2\ntotal = 0\ni = 0\nwhile i < 3:\n    total = total + base\n    i = i + 1\nwhy total",
    );
    assert_eq!(looped, counted);
    assert_eq!(
        looped,
        "[{\"from\":[],\"holds\":6.0,\"node\":\"total\",\"rule\":\"total = 0\",\"state\":\"stated\"}]"
    );
}

#[test]
fn the_string_method_repeat_keeps_its_results() {
    for (source, expected) in [
        ("\"ab\".repeat(2)", "\"abab\""),
        ("\"ab\".repeat(2.9)", "\"abab\""),
        ("\"ab\".repeat(0)", "\"\""),
        ("\"ab\".repeat(-1)", "\"\""),
        ("\"ab\".repeat()", "\"\""),
        ("\"ab\".repeat(null)", "\"\""),
        ("\"ab\".repeat(\"3\")", "\"ababab\""),
        ("\"ab\".repeat(true)", "\"ab\""),
        ("\"ab\".repeat(Number.NaN)", "\"\""),
        ("\"\".repeat(Number.POSITIVE_INFINITY)", "\"\""),
        ("\"a\".repeat(100001).length", "100001.0"),
    ] {
        assert_eq!(evaluate(source), expected, "{source}");
    }
    assert_eq!(
        failure("\"a\".repeat(1 / 0)"),
        (
            ErrorKind::Type,
            "Repeated string is too large".to_string(),
            Some(Position::new(1, 1))
        )
    );
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-repeat-loops-{label}-{}.nuc",
            std::process::id()
        ));
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        Self(path)
    }

    fn run(&self, source: &str, options: &[&str]) -> Output {
        std::fs::write(&self.0, source).unwrap();
        Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .args(options)
            .arg(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for SourceFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).unwrap();
    }
}

fn repl(source: &str, options: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(options)
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
fn files_graphs_and_repl_sessions_run_repeat_loops() {
    let file = SourceFile::new("file");
    let program = "n = 0\nrepeat 3: n = n + 2\nn";
    let counted = file.run(program, &["--json"]);
    assert!(counted.status.success(), "{counted:?}");
    assert!(counted.stderr.is_empty(), "{counted:?}");
    assert_eq!(String::from_utf8(counted.stdout).unwrap(), "6.0\n");
    let plain = file.run(program, &[]);
    assert!(plain.status.success(), "{plain:?}");
    assert_eq!(String::from_utf8(plain.stdout).unwrap(), "6\n");
    let graph = String::from_utf8(file.run(program, &["--json", "--graph"]).stdout).unwrap();
    assert!(graph.contains("  n [variable]"), "{graph}");
    assert!(!graph.contains("repeat"), "{graph}");
    let block = String::from_utf8(
        file.run("n = 0\n{\n    repeat 2: n = n + 1\n}\nn", &["--graph"])
            .stdout,
    )
    .unwrap();
    assert!(
        block.contains(
            "  n [variable]\n    updates  block(repeat(2){n=n+1})\n  block(repeat(2){n=n+1}) [block]\n    reads    n"
        ),
        "{block}"
    );
    let failed = file.run("n = 0\nrepeat 100001: n = n + 1", &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "TypeError: Repeat count is too large (line 2, column 1)\n"
    );
    let session =
        "n = 0\nrepeat 3:\n    n = n + 1\n\nn\nrepeat 2: n = n + 1\nn\nrepeat ready: tick\n.exit\n";
    for (options, expected) in [
        (&["--plain"][..], "0\nnull\n3\nnull\n5\n"),
        (&["--plain", "--json"][..], "0.0\nnull\n3.0\nnull\n5.0\n"),
        (&["--json"][..], "0.0\nnull\n3.0\nnull\n5.0\n"),
    ] {
        for newline in ["\n", "\r\n"] {
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.ends_with(expected), "{options:?} {stdout}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                "ReferenceError: repeat is not defined (line 1, column 1)\n"
            );
        }
    }
}
