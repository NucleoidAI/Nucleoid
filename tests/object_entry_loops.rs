mod common;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Expr, Stmt, parse};
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};
use serde_json::{Value as Json, json};

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid loops over object entries in insertion order",
    "Nucleoid loops over the entries an instance holds",
    "Nucleoid snapshots object entries before loop bodies",
    "Nucleoid nests object entry loops over nested records",
    "Nucleoid returns from object entry loops in functions and methods",
    "Nucleoid keeps object entry loops one shot",
    "Nucleoid restores object entries at nested savepoints",
    "Nucleoid restores object entries in place when it rolls back",
];

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

fn checked(runtime: &mut Runtime, source: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "object entry loop regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
    value
}

fn json_of(runtime: &Runtime, value: &Value) -> Json {
    serde_json::from_str(&runtime.serialize_json(value).unwrap()).unwrap()
}

fn evaluate(runtime: &mut Runtime, source: &str) -> Json {
    let value = checked(runtime, source);
    json_of(runtime, &value)
}

fn result(source: &str) -> Json {
    evaluate(&mut Runtime::new(), source)
}

fn failure(source: &str) -> (ErrorKind, String, Position) {
    failure_in(&mut Runtime::new(), source)
}

fn failure_in(runtime: &mut Runtime, source: &str) -> (ErrorKind, String, Position) {
    let error = runtime.run(source).expect_err("the program must fail");
    (
        error.kind(),
        error.message(),
        error.position().expect("errors are positioned"),
    )
}

fn syntax(source: &str) -> (String, Position) {
    let error = parse(source).expect_err("the program must not parse");
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn single(source: &str) -> Stmt {
    let [statement]: [Stmt; 1] = parse(source)
        .unwrap()
        .try_into()
        .unwrap_or_else(|statements| panic!("{source} parses to {statements:?}"));
    statement
}

fn has_two_name_header(source: &str) -> bool {
    let tokens: Vec<Token> = tokenize(source)
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    tokens.windows(5).any(|window| {
        matches!(
            window,
            [
                Token::Keyword(Keyword::For),
                Token::Identifier(_),
                Token::Comma,
                Token::Identifier(_),
                Token::Keyword(Keyword::Of | Keyword::In),
            ]
        )
    })
}

fn object_id(runtime: &mut Runtime, name: &str) -> ObjectId {
    match runtime.run(name).unwrap() {
        Value::Object(id) => id,
        other => panic!("{name} is {other:?}"),
    }
}

fn dependencies(runtime: &Runtime, prefix: &str) -> Vec<String> {
    let node = runtime
        .graph
        .nodes()
        .find(|node| node.key.as_str().starts_with(prefix))
        .unwrap_or_else(|| panic!("a node starting with {prefix} is filed"));
    node.dependencies
        .iter()
        .map(|key| key.as_str().to_string())
        .collect()
}

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
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let readme = common::snippets(include_str!("../README.md"), "README.md");
    let examples = common::snippets(include_str!("../docs/examples.md"), "docs/examples.md");
    let appended = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (
            &reference,
            "Nucleoid loops over the keys and values of an object",
        ),
        (&readme, "Entry loops read the keys and values of a record"),
        (
            &examples,
            "Entry loops stock warehouses from delivery manifests",
        ),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(
            has_two_name_header(&case.source),
            "{title} uses an entry loop"
        );
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 38);
}

#[test]
fn no_earlier_document_case_uses_a_two_name_header() {
    let documents = [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ];
    let appended = [
        "Nucleoid loops over the keys and values of an object",
        "Entry loops read the keys and values of a record",
        "Entry loops stock warehouses from delivery manifests",
    ];
    let mut cases = 0;
    let mut with_entries = 0;
    for case in documents.iter().flatten() {
        cases += 1;
        let uses = has_two_name_header(&case.source);
        let new =
            SPEC_TITLES.contains(&case.title.as_str()) || appended.contains(&case.title.as_str());
        assert_eq!(uses, new, "{}", case.title);
        with_entries += usize::from(uses);
    }
    assert_eq!(with_entries, 11);
    assert_eq!(cases, 1041);
}

#[test]
fn two_name_headers_parse_into_one_entry_loop_statement() {
    let body = parse("log.push(key + amount)").unwrap();
    for source in [
        "for key, amount of prices:\n    log.push(key + amount)",
        "for key, amount in prices:\n    log.push(key + amount)",
        "for key,amount of prices: log.push(key + amount)",
        "for key , amount in prices: log.push(key + amount)",
    ] {
        assert_eq!(
            parse(source).unwrap(),
            vec![Stmt::ForEntries {
                key: "key".to_string(),
                value: "amount".to_string(),
                iterable: Expr::Identifier("prices".to_string()),
                body: body.clone(),
            }],
            "{source}"
        );
    }

    let Stmt::ForEntries { body, .. } =
        single("for key, amount of prices: { a = key; b = amount }")
    else {
        panic!("a braced suite is one statement");
    };
    assert_eq!(body, parse("{ a = key; b = amount }").unwrap());

    let Stmt::ForEntries { body, .. } = single("for key, amount of prices: a = key; b = amount")
    else {
        panic!("an inline suite is one statement");
    };
    assert_eq!(body, parse("a = key\nb = amount").unwrap());

    let Stmt::ForEntries {
        iterable, value, ..
    } = single("for key, amount of shop.prices():\n    pass")
    else {
        panic!("an expression source parses");
    };
    assert_eq!(value, "amount");
    assert_eq!(iterable.to_string(), "shop.prices()");

    assert_eq!(
        single("for key of prices:\n    pass"),
        Stmt::For {
            variable: "key".to_string(),
            iterable: Expr::Identifier("prices".to_string()),
            body: vec![Stmt::Pass],
        }
    );
}

#[test]
fn two_name_loops_run_every_accepted_suite_form() {
    assert_eq!(
        result(
            "prices = {apple: 3, pear: 2}\nlog = []\nfor fruit, price of prices:\n    log.push(fruit)\n    log.push(price)\nlog"
        ),
        json!(["apple", 3.0, "pear", 2.0])
    );
    assert_eq!(
        result(
            "prices = {apple: 3, pear: 2}\nlog = []\nfor fruit, price in prices: log.push(fruit); log.push(price)\nlog"
        ),
        json!(["apple", 3.0, "pear", 2.0])
    );
    assert_eq!(
        result(
            "prices = {apple: 3, pear: 2}\nlog = []\nfor fruit, price of prices: { log.push(fruit + price) }\nlog"
        ),
        json!(["apple3", "pear2"])
    );
    assert_eq!(
        result("log = []\nfor fruit, price in {fig: 1}: log.push(fruit + price)\nlog"),
        json!(["fig1"])
    );
}

#[test]
fn entries_follow_insertion_order_and_skip_undefined_literal_slots() {
    assert_eq!(
        result(
            "none = {}\nrec = {b: 2, a: null, c: none.c, d: 4}\nrec.e = 5\ndelete rec.b\nrec.b = 6\nrec.d = 7\nseen = []\nfor key, value of rec: seen.push(key + \"=\" + value)\nseen"
        ),
        json!(["a=null", "d=7", "e=5", "b=6"])
    );
    assert_eq!(
        result(
            "count = {n: 0}\nfor key, value of {}: count.n += 1\nfor key, value of Object(): count.n += 1\ncount.n"
        ),
        json!(0.0)
    );
    assert_eq!(
        result("rec = {a: 1}\nkeys = []\nfor key, value of (why rec)[0]: keys.push(key)\nkeys"),
        json!(["node", "holds", "rule", "state", "from"])
    );
    assert_eq!(
        result(
            "rec = {a: {x: 1}}\nseen = []\nfor key, value of rec:\n    value.x = 9\n    seen.push(key)\n[seen, rec.a.x]"
        ),
        json!([["a"], 9.0])
    );
}

#[test]
fn instances_list_id_first_and_then_their_stored_fields() {
    let mut runtime = Runtime::new();
    let source = "class Room(name, width, depth):\n    this.name = name\n    this.width = width\n    this.depth = depth\n$Room.area = $Room.width * $Room.depth\nlab = Room(\"lab\", 3, 4)\nlab.lamp = \"on\"\nlab.width = 5\ndelete lab.name\nlab.name = \"studio\"\nfields = []\nfor field, value of lab: fields.push(field + \"=\" + value)\nfields";
    assert_eq!(
        evaluate(&mut runtime, source),
        json!([
            "id=lab",
            "width=5",
            "depth=4",
            "area=20",
            "lamp=on",
            "name=studio"
        ])
    );

    let id = object_id(&mut runtime, "lab");
    let Json::Object(serialized) = json_of(&runtime, &Value::Object(id)) else {
        panic!("an instance serializes as an object");
    };
    let mut keys: Vec<&str> = serialized.keys().map(String::as_str).collect();
    keys.sort_unstable();
    let mut listed = ["id", "width", "depth", "area", "lamp", "name"];
    listed.sort_unstable();
    assert_eq!(keys, listed);

    assert_eq!(
        result(
            "class Kit(parts):\n    this.count = 0\n    for name, qty of parts:\n        Part(name, qty)\n        this.count += qty\nclass Part(name, qty):\n    this.name = name\n    this.qty = qty\nkit = Kit({bolt: 4, nut: 6})\nfields = []\nfor field, value of kit: fields.push(field)\n[kit.count, Part.map(part -> part.name), fields]"
        ),
        json!([10.0, ["bolt", "nut"], ["id", "count"]])
    );
}

#[test]
fn entries_are_snapshotted_before_the_first_body_runs() {
    assert_eq!(
        result(
            "bills = {rent: 900, food: 300, travel: 200}\nseen = []\nfor bill, amount of bills:\n    seen.push(bill + \":\" + amount + \":\" + (bills[bill] ?? \"gone\"))\n    bills.food = 0\n    delete bills.travel\n    bills.fee = 25\nlater = []\nfor bill, amount of bills: later.push(bill + \":\" + amount)\n[seen, later]"
        ),
        json!([
            ["rent:900:900", "food:300:0", "travel:200:gone"],
            ["rent:900", "food:0", "fee:25"]
        ])
    );
    assert_eq!(
        result(
            "calls = {n: 0}\ndef table():\n    calls.n += 1\n    return {a: 1, b: 2}\ntotal = {sum: 0}\nfor key, value of table(): total.sum += value\n[calls.n, total.sum]"
        ),
        json!([1.0, 3.0])
    );
    assert_eq!(
        result(
            "key = \"outer\"\nseen = []\nfor key, value of {key: key}: seen.push(value)\n[seen, key]"
        ),
        json!([["outer"], "outer"])
    );
}

#[test]
fn every_other_source_raises_a_type_error_at_the_running_statement() {
    for (setup, source, message) in [
        ("xs = [1, 2]", "xs", "xs is not an object"),
        ("class Item", "Item", "Item is not an object"),
        ("text = \"ab\"", "text", "text is not an object"),
        ("n = 3", "n", "n is not an object"),
        ("flag = true", "flag", "flag is not an object"),
        ("broken = null", "broken", "broken is not an object"),
        ("rec = {}", "rec.missing", "rec.missing is not an object"),
        ("def f() => 1", "f", "f is not an object"),
        ("d = Date(0)", "d", "d is not an object"),
        ("r = /a/", "r", "r is not an object"),
        ("x = 1", "[1, 2]", "[1,2] is not an object"),
        ("class Item", "Item.list()", "Item.list() is not an object"),
    ] {
        let program = format!("{setup}\nfor key, value of {source}: pass");
        assert_eq!(
            failure(&program),
            (ErrorKind::Type, message.to_string(), Position::new(2, 1)),
            "{program}"
        );
    }
    assert_eq!(
        failure("for key, value of missing: pass"),
        (
            ErrorKind::Reference,
            "missing is not defined".to_string(),
            Position::new(1, 1)
        )
    );
    assert_eq!(
        result(
            "xs = [1]\nfailure = \"\"\ntry:\n    for key, value in xs: pass\ncatch error:\n    failure = error\nfailure"
        ),
        json!("TypeError: xs is not an object")
    );

    let mut runtime = Runtime::new();
    runtime.run("count = 1\nlist = [1]").unwrap();
    assert_eq!(
        failure_in(&mut runtime, "count = 2\nfor key, value of list: pass"),
        (
            ErrorKind::Type,
            "list is not an object".to_string(),
            Position::new(2, 1)
        )
    );
    assert_eq!(runtime.run("count").unwrap(), Value::Number(1.0));
}

#[test]
fn single_name_loops_keep_their_sources_and_diagnostics() {
    for program in [
        "rec = {a: 1}\nfor it of rec: pass",
        "rec = {a: 1}\nfor it in rec: pass",
        "rec = {a: 1}\nfor rec:\n    pass",
    ] {
        assert_eq!(
            failure(program),
            (
                ErrorKind::Type,
                "rec is not iterable".to_string(),
                Position::new(2, 1)
            ),
            "{program}"
        );
    }
    assert_eq!(
        result(
            "class T\na = T()\nseen = []\nfor item of [1, a, \"x\", null, {b: 1}]: seen.push(item.id)\nfor character of \"ab\": seen.push(character)\nseen"
        ),
        json!(["a", "a", "b"])
    );
}

#[test]
fn near_miss_headers_keep_their_accepted_diagnostics() {
    for (source, message, line, column) in [
        (
            "for values, other:\n    pass\n",
            "Expected 'of' in for statement",
            1,
            11,
        ),
        (
            "for t := \"ab\": pass",
            "Expected 'of' in for statement",
            1,
            7,
        ),
        (
            "for a, b, c of rec: pass",
            "Expected 'of' in for statement",
            1,
            6,
        ),
        (
            "for (k, v) of rec: pass",
            "Expected a name but found '('",
            1,
            6,
        ),
        (
            "for k, of rec: pass",
            "Expected 'of' in for statement",
            1,
            6,
        ),
        (
            "for k, this of rec: pass",
            "Expected 'of' in for statement",
            1,
            6,
        ),
        (
            "for 1, v of rec: pass",
            "Expected a name but found number 1",
            1,
            6,
        ),
        (
            "for k, 1 of rec: pass",
            "Expected 'of' in for statement",
            1,
            6,
        ),
        (
            "for k v of rec: pass",
            "Expected 'of' in for statement",
            1,
            7,
        ),
        (
            "for k, v, of rec: pass",
            "Expected 'of' in for statement",
            1,
            6,
        ),
        ("for k, v rec: pass", "Expected 'of' in for statement", 1, 6),
        ("for k, v: pass", "Expected 'of' in for statement", 1, 6),
        ("for k,: pass", "Expected 'of' in for statement", 1, 6),
        (
            "for , v of rec: pass",
            "Expected a name but found ','",
            1,
            7,
        ),
        (
            "for k, v = rec: pass",
            "Expected 'of' in for statement",
            1,
            6,
        ),
        (
            "for k.a, v of rec: pass",
            "Expected 'of' in for statement",
            1,
            6,
        ),
        ("for k of: pass", "Unexpected ':'", 1, 11),
    ] {
        assert_eq!(
            syntax(source),
            (message.to_string(), Position::new(line, column)),
            "{source:?}"
        );
    }
}

#[test]
fn malformed_two_name_headers_report_errors_where_they_occur() {
    for (source, message, line, column) in [
        ("for k, v of", "Unexpected end of input", 1, 12),
        ("for k, v of\n", "Unexpected end of line", 2, 1),
        (
            "for k, v of rec",
            "Expected ':' but found end of input",
            1,
            16,
        ),
        (
            "rec = {a: 1}\nfor k, v of rec\n    pass\n",
            "Expected ':' but found end of line",
            2,
            13,
        ),
        ("for k, v of rec:", "Unexpected end of input", 1, 17),
        (
            "for k, v of rec:\n",
            "Expected an indented block but found end of input",
            2,
            1,
        ),
        (
            "rec = {a: 1}\nfor k, v of rec:\npass\n",
            "Expected an indented block but found keyword 'pass'",
            3,
            1,
        ),
        ("for k, v of rec:;", "Unexpected ';'", 1, 18),
        (
            "for k, v of rec { pass }",
            "Expected ':' but found '{'",
            1,
            17,
        ),
        ("for k, v of rec: :", "Unexpected ':'", 1, 19),
        ("for k, v of :", "Unexpected ':'", 1, 14),
        ("for k, v in", "Unexpected end of input", 1, 12),
        (
            "rec = {a: 1}\nfor k, v in rec\n    pass\n",
            "Expected ':' but found end of line",
            2,
            13,
        ),
        (
            "for k, v of rec, other: pass",
            "Expected ':' but found ','",
            1,
            16,
        ),
        (
            "rec = {a: 1}\nfor k, v of rec:\n    x = )\n",
            "Unexpected ')'",
            3,
            9,
        ),
        ("for k, v of rec: pass\nx = )\n", "Unexpected ')'", 2, 5),
        (
            "for k, v of (rec: pass",
            "Expected ')' but found ':'",
            1,
            17,
        ),
        (
            "for k,v of rec",
            "Expected ':' but found end of input",
            1,
            15,
        ),
        (
            "rec = {a: 1}\nfor k, v of rec:\n    # only a comment\n",
            "Expected an indented block but found end of input",
            4,
            1,
        ),
        ("for k, v of rec:: pass", "Unexpected ':'", 1, 19),
        (
            "for k, v of rec: pass else: pass",
            "Unexpected keyword 'else'",
            1,
            27,
        ),
        ("for k, v of of: pass", "Unexpected keyword 'of'", 1, 15),
    ] {
        assert_eq!(
            syntax(source),
            (message.to_string(), Position::new(line, column)),
            "{source:?}"
        );
    }
}

#[test]
fn two_name_loops_render_once_and_round_trip() {
    let statement = single("for key, value in prices:\n    log.push(key)");
    assert_eq!(generate(&statement), "for key,value of prices");

    let source = "def total(rate = 1):\n    for key, value of prices:\n        rate = rate + value\n    return rate";
    let function = single(source);
    let rendered = generate(&function);
    assert_eq!(
        rendered,
        "def total(rate=1):\n    for key,value of prices:\n        rate=rate+value\n    return rate"
    );
    assert_eq!(parse(&rendered).unwrap(), vec![function]);

    assert_eq!(
        result(
            "prices = {a: 1, b: 2}\ntotal = (rate = 1) => {\n    for key, value of prices: rate = rate + value\n    return rate\n}\n(why total)[0].rule"
        ),
        json!(
            "total = function(rate=1){\n    for key,value of prices:\n        rate=rate+value\n    return rate\n}"
        )
    );

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "rec = {a: 1}\nout = {n: 0}\n{\n    for key, value of rec: out.n += value\n}",
    );
    assert!(
        runtime
            .graph
            .nodes()
            .any(|node| node.key.as_str() == "block(for key,value of rec)")
    );
}

#[test]
fn loops_run_in_functions_methods_constructors_and_class_rules() {
    assert_eq!(
        result(
            "def label(prefix, rec):\n    out = []\n    for key, value of rec:\n        out.push(prefix + key + value)\n    return out\nlabel(\"x\", {a: 1, b: 2})"
        ),
        json!(["xa1", "xb2"])
    );
    assert_eq!(
        result(
            "class Shelf(items):\n    this.items = items\n    this.total = 0\n    def count():\n        for key, value of this.items:\n            this.total += value\n        return this.total\nshelf = Shelf({a: 2, b: 3})\nshelf.count()"
        ),
        json!(5.0)
    );
    assert_eq!(
        result(
            "class Room(name, on):\n    this.name = name\n    this.on = on\nlog = []\nsizes = {s: 1, m: 2}\nif $Room.on:\n    for size, rank of sizes: log.push($Room.name + size + rank)\nRoom(\"a\", true)\nRoom(\"b\", false)\nlog"
        ),
        json!(["as1", "am2"])
    );
    assert_eq!(
        result(
            "def first(rec, limit):\n    for key, value of rec:\n        if value > limit:\n            return key\n    return \"none\"\n[first({a: 1, b: 5}, 2), first({a: 1}, 2)]"
        ),
        json!(["b", "none"])
    );
    assert_eq!(
        result(
            "def scan(rec):\n    for key, value of rec:\n        if value == 0:\n            return\n        hits.push(key)\n    hits.push(\"end\")\nhits = []\nscan({a: 1, b: 0, c: 2})\nscan({d: 3})\nhits"
        ),
        json!(["a", "d", "end"])
    );
    assert_eq!(
        result(
            "def find(nested, wanted):\n    for group, items of nested:\n        for item, qty of items:\n            if item == wanted:\n                return group + qty\n    return null\nfind({x: {a: 1}, y: {b: 2, c: 3}}, \"c\")"
        ),
        json!("y3")
    );
    assert_eq!(
        result("for key, value of {a: 1, b: 2}:\n    if value == 2: return key\n\"none\""),
        json!("b")
    );
    assert_eq!(
        result("result = {v: 1}\nfor key, value of {}: result.v = 2\nresult.v"),
        json!(1.0)
    );
}

#[test]
fn class_level_sources_fail_when_declared_as_single_name_loops_do() {
    assert_eq!(
        failure(
            "class Box(parts): this.parts = parts\nb = Box({x: 1, y: 2})\n{\n    for key, value of $Box.parts: $Box.count = 1\n}"
        ),
        (
            ErrorKind::Type,
            "$Box.parts is not an object".to_string(),
            Position::new(3, 1)
        )
    );
    assert_eq!(
        failure(
            "class Box(parts): this.parts = parts\nb = Box([1, 2])\n{\n    for part of $Box.parts: $Box.count = 1\n}"
        ),
        (
            ErrorKind::Type,
            "$Box.parts is not iterable".to_string(),
            Position::new(3, 1)
        )
    );
}

#[test]
fn loops_run_in_blocks_suppose_and_try_with_rollback() {
    assert_eq!(
        result(
            "class Part(name): this.name = name\nrec = {a: 1, b: 2}\nseen = suppose:\n    for key, value of rec: Part(key)\n    Part.length\n[seen, Part.length]"
        ),
        json!([2.0, 0.0])
    );
    assert_eq!(
        result(
            "class Part(name): this.name = name\norder = {bolts: 5, nuts: 0, gears: 2}\npacked = {units: 0}\nfailure = \"\"\ntry:\n    for item, qty of order:\n        packed.units += qty\n        if qty == 0:\n            throw \"EMPTY_\" + item\n        Part(item)\ncatch error:\n    failure = error\nfirst := [failure, Part.length, packed.units]\norder.nuts = 3\nfor item, qty of order:\n    packed.units += qty\n    Part(item)\n[first, Part.length, packed.units]"
        ),
        json!([["EMPTY_nuts", 0.0, 0.0], 3.0, 10.0])
    );

    let mut runtime = Runtime::new();
    runtime
        .run("class Part(name): this.name = name\norder = {a: 1, b: 0}\ntally = {n: 0}")
        .unwrap();
    let error = runtime
        .run("for item, qty of order:\n    tally.n += 1\n    Part(item)\n    if qty == 0: throw \"STOP\"")
        .unwrap_err();
    assert_eq!(error.thrown_value(), Some(&Value::String("STOP".into())));
    assert_eq!(error.position(), Some(Position::new(1, 1)));
    assert_eq!(runtime.run("tally.n").unwrap(), Value::Number(0.0));
    assert_eq!(runtime.run("Part.length").unwrap(), Value::Number(0.0));
}

#[test]
fn names_shadow_and_restore_outer_bindings() {
    assert_eq!(
        result(
            "key = \"k\"\nvalue = \"v\"\nseen = []\nfor key, value of {a: 1}:\n    for key, value of {b: 2}: seen.push(key + value)\n    seen.push(key + value)\n[seen, key, value]"
        ),
        json!([["b2", "a1"], "k", "v"])
    );
    assert_eq!(
        result("seen = []\nfor name, name of {a: 1, b: 2}: seen.push(name)\nseen"),
        json!([1.0, 2.0])
    );
    assert_eq!(
        failure("for fresh, other of {a: 1}: pass\nfresh"),
        (
            ErrorKind::Reference,
            "fresh is not defined".to_string(),
            Position::new(2, 1)
        )
    );
    assert_eq!(
        result(
            "for key, value of {a: 1}:\n    key = \"z\"\n    value = 9\n    out = key + value\nout"
        ),
        json!("z9")
    );
}

#[test]
fn the_snapshot_records_no_dependency_of_its_own() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Lamp(room, watts):\n    this.room = room\n    this.watts = watts\nlights = {hall: 40, desk: 60}\nnight = false\nif night:\n    for room, watts of lights: Lamp(room, watts)")
        .unwrap();
    assert_eq!(runtime.run("Lamp.length").unwrap(), Value::Number(0.0));
    runtime.run("night = true").unwrap();
    assert_eq!(runtime.run("Lamp.length").unwrap(), Value::Number(2.0));
    runtime.run("lights.desk = 100").unwrap();
    runtime.run("lights.attic = 15").unwrap();
    assert_eq!(runtime.run("Lamp.length").unwrap(), Value::Number(2.0));
    runtime.run("lights = {shed: 5}").unwrap();
    assert_eq!(runtime.run("Lamp.length").unwrap(), Value::Number(3.0));

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "rec = {a: 1, b: 2}\nout = {n: 0}\n{\n    for key, value of rec: out.n += value\n}",
    );
    let rec = object_id(&mut runtime, "rec");
    let block = dependencies(&runtime, "block(");
    assert!(block.contains(&"rec".to_string()), "{block:?}");
    for property in ["a", "b"] {
        let key = NodeKey::property(&rec, property);
        assert!(!block.contains(&key.as_str().to_string()), "{block:?}");
    }

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "rec = {a: 1}\nout = {n: 0}\n{\n    for key, value of rec: out.n += rec[key]\n}",
    );
    let rec = object_id(&mut runtime, "rec");
    let block = dependencies(&runtime, "block(");
    assert!(
        block.contains(&NodeKey::property(&rec, "a").as_str().to_string()),
        "{block:?}"
    );
}

#[test]
fn protected_assertions_inside_two_name_bodies_are_counted() {
    let source = "rec = {a: 1, b: 2}\nfor key, value of rec:\n    assert(value > 0, true)";
    assert_eq!(
        common::assertions::protected_assertions(&parse(source).unwrap()),
        vec!["assert(value>0,true)".to_string()]
    );
    let mut runtime = Runtime::new();
    runtime.run(source).unwrap();
    assert_eq!(runtime.assertions_run(), 2);
    assert!(runtime.take_assertions().is_empty());
}

#[test]
fn entry_loops_cost_stays_linear() {
    let entries = 20_000;
    let declarations = (0..entries)
        .map(|index| format!("big.k{index} = {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!(
        "big = Object()\n{declarations}\ncalls = {{n: 0}}\ndef table():\n    calls.n += 1\n    return big\ntotal = {{sum: 0, count: 0}}\nfor key, value of table():\n    total.sum += value\n    total.count += 1\n[calls.n, total.count, total.sum]"
    );
    let sum: i32 = (0..entries).sum();
    assert_eq!(
        result(&source),
        json!([1.0, f64::from(entries), f64::from(sum)])
    );
}

#[test]
fn the_native_repl_buffers_indented_two_name_headers_and_runs_inline_ones() {
    let output = repl(
        "rec = {a: 1, b: 2}\nseen = []\nfor key, value of rec:\n    seen.push(key + value)\n\nseen\nfor key, value in rec: seen.push(value)\nseen\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "{BANNER}{{\"a\":1.0,\"b\":2.0}}\n[]\nnull\n[\"a1\",\"b2\"]\nnull\n[\"a1\",\"b2\",1.0,2.0]\n"
        )
    );
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}
