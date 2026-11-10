mod common;

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{ClassDecl, Expr, FunctionBody, Stmt, parse};
use nucleoid::{ErrorKind, Position, Runtime};

const SPEC_TITLES: [&str; 10] = [
    "Nucleoid assigns the properties that a constructor header names with this",
    "Nucleoid initializes self parameters before the constructor body reads them",
    "Nucleoid fills typed initializing parameters from literal defaults and labels",
    "Nucleoid leaves missing initializing parameters null",
    "Nucleoid collects final rest arguments into an initializing parameter",
    "Nucleoid initializes properties from the parameter list of init",
    "Nucleoid initializes subclass parameters before the parent constructor runs",
    "Nucleoid applies class level rules to initialized properties",
    "Nucleoid explains initialized properties by their constructor assignments",
    "Nucleoid rolls back an instance whose initializing constructor fails",
];
const REFERENCE_TITLE: &str = "Nucleoid initializes properties from constructor parameters";
const README_TITLE: &str = "Constructor parameters can assign the properties they name";
const EXAMPLES_TITLE: &str = "Initializing parameters keep a fleet's service schedule live";

const PARAMETER: &str = "Expected a parameter but found keyword 'this'";
const BARE: &str = "Expected '(' but found keyword 'this'";

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn single_class(source: &str) -> ClassDecl {
    match statements(source).as_slice() {
        [Stmt::Class(class)] => class.clone(),
        other => panic!("{source} parses to {other:?}"),
    }
}

fn equivalent(prefixed: &str, written: &str) {
    let prefixed_tree = statements(prefixed);
    let written_tree = statements(written);
    assert_eq!(prefixed_tree, written_tree, "{prefixed}");
    assert_eq!(
        generate_all(&prefixed_tree),
        generate_all(&written_tree),
        "{prefixed}"
    );
}

fn graph_shape(runtime: &Runtime) -> Vec<String> {
    runtime
        .graph
        .nodes()
        .map(|node| {
            let reads: Vec<String> = node.dependencies.iter().map(ToString::to_string).collect();
            let updates: Vec<String> = node.dependents.iter().map(ToString::to_string).collect();
            format!("{} [{}] {reads:?} {updates:?}", node.key, node.kind)
        })
        .collect()
}

fn expect(runtime: &mut Runtime, source: &str, expected: &str) {
    let assertions = common::assertions::protected_assertions(&statements(source)).len();
    let case = common::Case {
        title: "initializing parameter regression".to_string(),
        source: source.to_string(),
        expected: Some(expected.to_string()),
    };
    common::check_with_count(runtime, &case, assertions)
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn fresh(source: &str, expected: &str) {
    expect(&mut Runtime::new(), source, expected);
}

fn result(runtime: &mut Runtime, source: &str) -> Result<String, String> {
    match runtime.run(source) {
        Ok(value) => Ok(runtime.serialize_json(&value).unwrap()),
        Err(error) => Err(error.to_string()),
    }
}

fn same_runs(prefixed: &str, written: &str, tail: &str) -> Result<String, String> {
    let mut prefixed_runtime = Runtime::new();
    let mut written_runtime = Runtime::new();
    assert_eq!(
        result(&mut prefixed_runtime, prefixed),
        result(&mut written_runtime, written),
        "{prefixed}"
    );
    let prefixed_result = result(&mut prefixed_runtime, tail);
    let written_result = result(&mut written_runtime, tail);
    assert_eq!(prefixed_result, written_result, "{prefixed}\n{tail}");
    assert_eq!(
        graph_shape(&prefixed_runtime),
        graph_shape(&written_runtime),
        "{prefixed}\n{tail}"
    );
    prefixed_result
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn assignment(name: &str) -> Stmt {
    Stmt::Assign {
        target: Expr::Member {
            object: Box::new(Expr::This),
            property: name.to_string(),
        },
        value: Expr::Identifier(name.to_string()),
    }
}

fn repl(input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--plain", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn starts_parameter(tokens: &[Token], index: usize) -> bool {
    matches!(
        index
            .checked_sub(1)
            .and_then(|previous| tokens.get(previous)),
        Some(Token::LeftParen | Token::Comma | Token::Dot)
    )
}

fn constructor_lists(tokens: &[Token]) -> Vec<usize> {
    tokens
        .windows(3)
        .enumerate()
        .filter(|(_, window)| {
            matches!(
                window,
                [
                    Token::Keyword(Keyword::Class),
                    Token::Identifier(_),
                    Token::LeftParen
                ]
            ) || matches!(
                window,
                [Token::Keyword(Keyword::Def), Token::Identifier(name), Token::LeftParen]
                    if name == "init"
            )
        })
        .map(|(index, _)| index + 2)
        .collect()
}

fn uses_prefix(source: &str) -> bool {
    let tokens: Vec<Token> = tokenize(source.trim())
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    constructor_lists(&tokens).into_iter().any(|open| {
        let mut depth = 0usize;
        for (index, token) in tokens.iter().enumerate().skip(open) {
            match token {
                Token::LeftParen | Token::LeftBracket | Token::LeftBrace => depth += 1,
                Token::RightParen | Token::RightBracket | Token::RightBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return false;
                    }
                }
                Token::Keyword(Keyword::This)
                    if depth == 1
                        && starts_parameter(&tokens, index)
                        && tokens.get(index + 1) == Some(&Token::Dot) =>
                {
                    return true;
                }
                _ => {}
            }
        }
        false
    })
}

#[test]
fn every_header_form_parses_to_the_written_out_class() {
    for (prefixed, written) in [
        (
            "class Point(this.x, this.y)",
            "class Point(x, y):\n    this.x = x\n    this.y = y",
        ),
        (
            "class Point(self.x, self.y)",
            "class Point(x, y):\n    self.x = x\n    self.y = y",
        ),
        (
            "class Point(self.x, this.y)",
            "class Point(x, y):\n    this.x = x\n    this.y = y",
        ),
        (
            "class Point(this.x):\n    this.double = x * 2",
            "class Point(x):\n    this.x = x\n    this.double = x * 2",
        ),
        (
            "class Point(this.x): this.double = x * 2",
            "class Point(x):\n    this.x = x\n    this.double = x * 2",
        ),
        (
            "class Point(this.x): Base\n    super(x)",
            "class Point(x): Base\n    this.x = x\n    super(x)",
        ),
        (
            "class Point(this.x) extends Base",
            "class Point(x): Base\n    this.x = x",
        ),
        (
            "class Point(this.x) extends Base:\n    super(x)",
            "class Point(x): Base\n    this.x = x\n    super(x)",
        ),
        (
            "class Point(this.x) extends Base: super(x)",
            "class Point(x): Base\n    this.x = x\n    super(x)",
        ),
        (
            "class Point(a, this.b, c)",
            "class Point(a, b, c):\n    this.b = b",
        ),
        (
            "class Point(this.a: int, this.b = 1, this.c: str = \"x\", ...this.d)",
            "class Point(a: int, b = 1, c: str = \"x\", ...d):\n    this.a = a\n    this.b = b\n    this.c = c\n    this.d = d",
        ),
        (
            "class Point(this.x, this.x)",
            "class Point(x, x):\n    this.x = x\n    this.x = x",
        ),
        (
            "class Point(this.x, x)",
            "class Point(x, x):\n    this.x = x",
        ),
        (
            "class Point(\n    this.x,\n    self . y,\n)",
            "class Point(x, y):\n    this.x = x\n    this.y = y",
        ),
        (
            "class Point(this.x):\n    y: int\n    def area() => this.x",
            "class Point(x):\n    this.x = x\n    y: int\n    def area() => this.x",
        ),
        (
            "class Point(this.x); next = 1",
            "class Point(x):\n    this.x = x\nnext = 1",
        ),
        (
            "class Outer:\n    class Inner(this.v)",
            "class Outer:\n    class Inner(v):\n        this.v = v",
        ),
        (
            "if ready:\n    class Point(this.x)",
            "if ready:\n    class Point(x):\n        this.x = x",
        ),
        (
            "{ class Point(this.x) }",
            "{\n    class Point(x):\n        this.x = x\n}",
        ),
        (
            "def make():\n    class Point(this.x)\n    return Point(1)",
            "def make():\n    class Point(x):\n        this.x = x\n    return Point(1)",
        ),
        (
            "make = function() { class Point(this.x) }",
            "make = function() { class Point(x): this.x = x }",
        ),
    ] {
        equivalent(prefixed, written);
    }
    let class = single_class("class Point(this.x, self.y)");
    assert_eq!(class.constructor, [assignment("x"), assignment("y")]);
    assert!(class.fields.is_empty() && class.methods.is_empty());
    assert!(
        single_class("class Shell(code: int)")
            .constructor
            .is_empty()
    );
}

#[test]
fn every_init_body_form_parses_to_the_written_out_init() {
    let written = "class Point:\n    def init(x, y):\n        this.x = x\n        this.y = y\n        this.sum = x + y";
    equivalent(
        "class Point:\n    def init(this.x, this.y):\n        this.sum = x + y",
        written,
    );
    equivalent(
        "class Point:\n    def init(this.x, this.y): this.sum = x + y",
        written,
    );
    let returns = "class Point:\n    def init(x):\n        this.x = x\n        return 5";
    for arrow in [
        "class Point:\n    def init(this.x) => 5",
        "class Point:\n    def init(this.x) -> 5",
        "class Point:\n    def init(this.x) =>\n        5",
    ] {
        equivalent(arrow, returns);
    }
    for (prefixed, written) in [
        (
            "class Point:\n    def init(this.a: int, this.b = 2, ...this.c):\n        pass",
            "class Point:\n    def init(a: int, b = 2, ...c):\n        this.a = a\n        this.b = b\n        this.c = c\n        pass",
        ),
        (
            "class Point: Base\n    def init(this.x):\n        super(x)",
            "class Point: Base\n    def init(x):\n        this.x = x\n        super(x)",
        ),
        (
            "class Point extends Base:\n    def init(this.x):\n        pass",
            "class Point: Base\n    def init(x):\n        this.x = x\n        pass",
        ),
        (
            "class Point:\n    x: int\n    def init(this.x):\n        pass\n    def double() => this.x * 2",
            "class Point:\n    x: int\n    def init(x):\n        this.x = x\n        pass\n    def double() => this.x * 2",
        ),
        (
            "class Point(this.a):\n    def init(this.b):\n        pass",
            "class Point(a):\n    this.a = a\n    def init(b):\n        this.b = b\n        pass",
        ),
    ] {
        equivalent(prefixed, written);
    }
    let class = single_class("class Point:\n    def init(this.x) => 5");
    let [method] = class.methods.as_slice() else {
        panic!("one method");
    };
    assert_eq!(method.name.as_deref(), Some("init"));
    assert_eq!(
        method.body,
        FunctionBody::Block(vec![assignment("x"), Stmt::Return(Some(Expr::Number(5.0)))])
    );
}

#[test]
fn binding_follows_the_written_out_spelling() {
    let header = "class P(this.x, this.y = 7, ...this.rest)";
    let written =
        "class P(x, y = 7, ...rest):\n    this.x = x\n    this.y = y\n    this.rest = rest";
    for (call, expected) in [
        ("P(1)", "[1.0,7.0,[]]"),
        ("P(1, 2, 3, 4)", "[1.0,2.0,[3.0,4.0]]"),
        ("P(y=2, x=1)", "[1.0,2.0,[]]"),
        ("P(5, y=6)", "[5.0,6.0,[]]"),
        ("P()", "[null,7.0,[]]"),
        ("P(null, null)", "[null,null,[]]"),
    ] {
        assert_eq!(
            same_runs(header, written, &format!("p = {call}\n[p.x, p.y, p.rest]")),
            Ok(expected.to_string()),
            "{call}"
        );
    }
    for (call, error) in [
        (
            "P(z=1)",
            "TypeError: Unknown constructor argument 'z' for P",
        ),
        (
            "P(rest=[1])",
            "TypeError: Rest parameter 'rest' cannot be a named constructor argument for P",
        ),
        (
            "P(1, x=2)",
            "TypeError: Duplicate constructor argument 'x' for P",
        ),
        (
            "P(x=1, 2)",
            "TypeError: Positional constructor argument after named arguments for P",
        ),
    ] {
        assert_eq!(
            same_runs(header, written, &format!("p = {call}")),
            Err(format!("{error} (line 1, column 1)")),
            "{call}"
        );
    }
    for (prefixed, written, tail, expected) in [
        (
            "class P(this.x, this.x)",
            "class P(x, x):\n    this.x = x\n    this.x = x",
            "p = P(1, 2)\np.x",
            "2.0",
        ),
        (
            "class P(this.x, x)",
            "class P(x, x):\n    this.x = x",
            "p = P(1, 2)\np.x",
            "2.0",
        ),
        (
            "class P(x, this.x)",
            "class P(x, x):\n    this.x = x",
            "p = P(1, 2)\np.x",
            "2.0",
        ),
        (
            "class P(this.label: str = \"none\", this.done = false, this.count = -0)",
            "class P(label: str = \"none\", done = false, count = -0):\n    this.label = label\n    this.done = done\n    this.count = count",
            "p = P()\n[p.label, p.done, p.count]",
            "[\"none\",false,-0.0]",
        ),
        (
            "class P(x, this.y)",
            "class P(x, y):\n    this.y = y",
            "p = P(1, 2)\n[p.x, p.y]",
            "[null,2.0]",
        ),
    ] {
        assert_eq!(
            same_runs(prefixed, written, tail),
            Ok(expected.to_string()),
            "{prefixed}"
        );
    }
    fresh(
        "class Plain(x, x):\n    this.x = x\nplain = Plain(1, 2)\nassert(plain.x, 2)\nclass Account(this.owner: str, this.balance: int = 0)\na = Account(balance=50, owner=\"ana\")\nb = Account(\"bo\")\nassert([a.owner, a.balance, b.owner, b.balance], [\"ana\", 50, \"bo\", 0])\nassert(Account.length, 2)\nb.balance",
        "0",
    );
}

#[test]
fn assignments_run_before_written_statements_and_a_leading_super() {
    fresh(
        "class Box(this.width, self.height):\n    this.area = this.width * this.height\n    this.wide = width > height\nbox = Box(4, 2)\nassert([box.area, box.wide], [8, true])\nbox.width = 1\nassert([box.area, box.wide], [2, true])\nbox.area",
        "2",
    );
    fresh(
        "class Scaled(this.x):\n    this.x = x * 10\nassert(Scaled(2).x, 20)\nclass Seen(this.x):\n    this.copy = this.x\nassert(Seen(3).copy, 3)\nSeen.length",
        "1",
    );
    fresh(
        "class Base(this.level)\nclass Boosted(this.level) extends Base:\n    super(level * 10)\nb = Boosted(2)\nassert(b.level, 20)\nclass Kept(this.level, this.bonus) extends Base:\n    super(level)\n    this.level = level + bonus\nk = Kept(2, 3)\nassert(k.level, 5)\n[Base.length, Boosted.length, Kept.length]",
        "[0,1,1]",
    );
    fresh(
        "class Base(level):\n    this.level = level * 100\nclass Child(this.level, this.tag): Base\n    super(level)\nc = Child(1, \"c\")\nassert([c.level, c.tag], [100, \"c\"])\nc.tag",
        "\"c\"",
    );
    fresh(
        "class Sensor:\n    def init(this.room, this.reading):\n        this.alarm = this.reading > 30\nattic = Sensor(\"attic\", 35)\nassert([attic.room, attic.reading, attic.alarm], [\"attic\", 35, true])\nattic.reading = 20\nassert(attic.alarm, false)\nattic.alarm",
        "false",
    );
}

#[test]
fn the_nearest_nonempty_constructor_counts_prefixed_statements() {
    fresh(
        "class Base(this.a)\nclass Plain(b) extends Base\nclass Own(this.b) extends Base\nplain = Plain(7)\nown = Own(7)\nassert([plain.a, plain.b], [7, null])\nassert([own.a, own.b], [null, 7])\nassert(Plain(a=3).a, 3)\nPlain.length",
        "2",
    );
    let error = Runtime::new()
        .run("class Base(this.a)\nclass Own(this.b) extends Base\nOwn(a=3)")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Unknown constructor argument 'a' for Own");
    fresh(
        "class Adopted(a):\n    def init(this.b):\n        pass\nadopted = Adopted(1)\nassert([adopted.a, adopted.b], [null, 1])\nclass Kept(this.a):\n    def init(this.b):\n        pass\nkept = Kept(1)\nassert([kept.a, kept.b], [1, null])\nkept.init(2)\nassert([kept.a, kept.b], [1, 2])\nkept.b",
        "2",
    );
    assert_eq!(
        same_runs(
            "class Shell(this.code: int)\nclass Inner(this.v) extends Shell",
            "class Shell(code: int):\n    this.code = code\nclass Inner(v): Shell\n    this.v = v",
            "inner = Inner(4)\n[inner.v, inner.code]"
        ),
        Ok("[4.0,null]".to_string())
    );
}

#[test]
fn attributes_redeclaration_rollback_rules_and_populations_are_unchanged() {
    fresh(
        "class Point(this.x):\n    y: int\np = Point(1)\nassert(p, {\"id\": \"p\", \"x\": 1})\nclass Point(this.x, this.y)\nq = Point(2, 3)\nassert([Point.length, p.y, q.y], [2, null, 3])\nclass Point(x, y):\n    this.x = x\n    this.y = y\nr = Point(4, 5)\nassert([Point.length, r.x, r.y], [3, 4, 5])\nPoint.map(point -> point.x)",
        "[1,2,4]",
    );
    fresh(
        "class Ticket(this.code, this.seat):\n    if seat > 100:\n        throw \"no such seat\"\nfirst = Ticket(\"A1\", 4)\nreason = null\ntry:\n    Ticket(\"Z9\", 400)\ncatch error:\n    reason = error\nassert([Ticket.length, reason, first.seat], [1, \"no such seat\", 4])\nTicket.length",
        "1",
    );
    fresh(
        "class Order(this.qty, this.price)\n$Order.total = $Order.qty * $Order.price\n$Order.bulk = $Order.qty >= 10\na = Order(2, 3)\nb = Order(12, 1)\nrevenue = Order.sum(order -> order.total)\nassert([a.total, b.bulk, revenue], [6, true, 18])\na.qty = 10\nassert([a.total, a.bulk, revenue], [30, true, 42])\n[typeof $Order, typeof Order, Order.length]",
        "[\"Class\",\"List\",2]",
    );
}

#[test]
fn explanations_graph_keys_and_rendering_match_the_written_out_spelling() {
    let mut prefixed = Runtime::new();
    let mut written = Runtime::new();
    for (prefixed_source, written_source) in [
        (
            "class Shipment(this.origin, this.weight = 1)",
            "class Shipment(origin, weight = 1):\n    this.origin = origin\n    this.weight = weight",
        ),
        (
            "$Shipment.heavy = $Shipment.weight > 10",
            "$Shipment.heavy = $Shipment.weight > 10",
        ),
        (
            "parcel = Shipment(\"oslo\", 3)",
            "parcel = Shipment(\"oslo\", 3)",
        ),
        ("why parcel.weight", "why parcel.weight"),
        ("why parcel.heavy", "why parcel.heavy"),
        ("affects parcel.weight", "affects parcel.weight"),
        ("parcel.weight = 12", "parcel.weight = 12"),
        (
            "[parcel.heavy, (why parcel.weight)[0].rule]",
            "[parcel.heavy, (why parcel.weight)[0].rule]",
        ),
        ("model |> why", "model |> why"),
        ("empty = Shipment()", "empty = Shipment()"),
        (
            "[empty.origin, empty.weight, empty.heavy]",
            "[empty.origin, empty.weight, empty.heavy]",
        ),
        ("Shipment.length", "Shipment.length"),
    ] {
        let prefixed_result = result(&mut prefixed, prefixed_source);
        let written_result = result(&mut written, written_source);
        assert_eq!(prefixed_result, written_result, "{prefixed_source}");
        assert_eq!(
            graph_shape(&prefixed),
            graph_shape(&written),
            "{prefixed_source}"
        );
    }
    assert_eq!(
        result(&mut prefixed, "(why parcel.origin)[0].rule"),
        Ok("\"$Shipment.origin = origin\"".to_string())
    );
    assert_eq!(
        generate_all(&statements("class Shipment(this.weight = 1)")),
        "class Shipment(weight=1):\n    this.weight=weight"
    );
    assert_eq!(
        generate_all(&statements("class Shipment(this.weight)")),
        "class Shipment"
    );
    assert_eq!(
        generate_all(&statements(
            "class Route(this.name, ...this.stops) extends Path"
        )),
        "class Route(name,...stops): Path\n    this.name=name\n    this.stops=stops"
    );
}

#[test]
fn the_cli_runs_a_file_of_prefixed_headers() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("initializing-parameters.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().next(), Some("[12.0,true,\"van\"]"));
    assert!(stdout.contains("$Truck [class]"), "{stdout}");
    assert!(stdout.contains("hauler.load [property]"), "{stdout}");
}

#[test]
fn the_native_repl_buffers_prefixed_headers_as_plain_headers() {
    let output = repl(
        "class P(this.x, self.y)\np = P(3, 4)\n[p.x, p.y]\nclass Q(this.a):\n    this.b = a * 2\n\nq = Q(5)\n[q.a, q.b]\n.exit\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "{BANNER}null\n{{\"id\":\"p\",\"x\":3.0,\"y\":4.0}}\n[3.0,4.0]\nnull\n{{\"a\":5.0,\"b\":10.0,\"id\":\"q\"}}\n[5.0,10.0]\n"
        )
    );
    let output = repl("def f(this.x) => 1\nclass P(this.a.b)\n.exit\n");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        format!(
            "SyntaxError: {PARAMETER} (line 1, column 11)\nSyntaxError: {PARAMETER} (line 1, column 13)\n"
        )
    );
}

#[test]
fn every_other_parameter_list_keeps_its_accepted_diagnostic() {
    for (source, message, line, column) in [
        ("def f(this.x) => 1", PARAMETER, 1, 11),
        ("def f(this.x):\n    return 1", PARAMETER, 1, 11),
        ("def f(a, self.b) => 1", PARAMETER, 1, 14),
        (
            "class P:\n    def m(this.x):\n        pass",
            PARAMETER,
            2,
            15,
        ),
        ("class P(x):\n    def m(this.x) => 1", PARAMETER, 2, 15),
        (
            "class P:\n    def initialize(this.x):\n        pass",
            PARAMETER,
            2,
            24,
        ),
        ("f = (this.x) => 1", PARAMETER, 1, 10),
        ("f = (this.x) -> 1", PARAMETER, 1, 10),
        ("f = (this.x) => {\n    return 1\n}", PARAMETER, 1, 10),
        ("f = function(this.x) { return 1 }", PARAMETER, 1, 18),
        ("f = lambda this.x: 1", "Unexpected ':'", 1, 20),
        ("def init(this.x):\n    pass", PARAMETER, 1, 14),
        ("class P: def init(this.x): pass", PARAMETER, 1, 23),
        (
            "class P:\n    if true:\n        def init(this.x):\n            pass",
            PARAMETER,
            3,
            22,
        ),
        (
            "class P:\n    {\n        def init(this.x):\n            pass\n    }",
            PARAMETER,
            3,
            22,
        ),
        (
            "def outer():\n    def init(this.x):\n        pass",
            PARAMETER,
            2,
            18,
        ),
        ("def f this.x => 1", BARE, 1, 7),
        ("def f this.x:\n    return 1", BARE, 1, 7),
        ("def f self.x => 1", BARE, 1, 7),
        ("class P:\n    def init this.x:\n        pass", BARE, 2, 14),
        ("class P:\n    def init this.x => 1", BARE, 2, 14),
        (
            "class P:\n    def init this.x, y:\n        pass",
            BARE,
            2,
            14,
        ),
        (
            "class P(x: this.y)",
            "Expected a type but found keyword 'this'",
            1,
            16,
        ),
        (
            "class P(x = this.y)",
            "Expected a scalar literal parameter default",
            1,
            13,
        ),
        (
            "class P(super.x)",
            "Expected a parameter but found keyword 'super'",
            1,
            14,
        ),
        ("class P(.x)", "Expected a parameter but found '.'", 1, 10),
        ("class P(@x)", "Expected a parameter but found '@'", 1, 10),
        (
            "class P(val x)",
            "Expected ')' but found identifier 'x'",
            1,
            13,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    let error = Runtime::new()
        .run("class P:\n    def init(x):\n        pass\np = P(this.x)")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "this is not defined");
}

#[test]
fn unqualified_prefixes_keep_the_accepted_diagnostic() {
    for (source, line, column) in [
        ("class P(this.a.b)", 1, 13),
        ("class P(this[\"x\"])", 1, 13),
        ("class P(this)", 1, 13),
        ("class P(this.class)", 1, 13),
        ("class P(this.5)", 1, 13),
        ("class P(this.x this.y)", 1, 13),
        ("class P(this.$X)", 1, 13),
        ("class P(self.self)", 1, 13),
        ("class P(this.this)", 1, 13),
        ("class P(this.x())", 1, 13),
        ("class P(this.x@1)", 1, 13),
        ("class P(this?.x)", 1, 13),
        ("class P(this..x)", 1, 13),
        ("class P(this....x)", 1, 13),
        ("class P(this.\"x\")", 1, 13),
        ("class P(this.", 1, 13),
        ("class P(this.x", 1, 13),
        ("class P(self)", 1, 13),
        ("class P(this.a.b) extends B", 1, 13),
        ("class P:\n    def init(this.a.b):\n        pass", 2, 18),
        ("class P(\n    this.x\n", 2, 9),
    ] {
        assert_eq!(
            syntax_error(source),
            (PARAMETER.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn former_errors_report_the_diagnostics_of_their_plain_spelling() {
    for (source, message, line, column) in [
        (
            "class P(this.x: 5)\n",
            "Expected a type but found number 5",
            1,
            18,
        ),
        (
            "class P(this.x = y)\n",
            "Expected a scalar literal parameter default",
            1,
            18,
        ),
        (
            "class P(this.x = 1e999)\n",
            "Parameter default must be a finite number",
            1,
            18,
        ),
        (
            "class P(this.x = 1 2)\n",
            "Expected ',' or ')' after parameter default",
            1,
            20,
        ),
        (
            "class P(this.x: int int)\n",
            "Expected ')' but found identifier 'int'",
            1,
            21,
        ),
        (
            "class P(...this.x = 1)\n",
            "Rest parameter cannot have a default",
            1,
            21,
        ),
        (
            "class P(this.x, ...this.x)\n",
            "Rest parameter 'x' duplicates a fixed parameter",
            1,
            26,
        ),
        (
            "class P(...this.x, this.y)\n",
            "Rest parameter must be last",
            1,
            20,
        ),
        (
            "class P(this.x)\n    this.y = 1\n",
            "Expected ':' but found end of line",
            1,
            15,
        ),
        (
            "class P(this.x) extend B\n",
            "Expected ':' but found identifier 'extend'",
            1,
            17,
        ),
        (
            "class P(this.x) extends\n",
            "Expected a parent class name but found end of line",
            1,
            17,
        ),
        (
            "class P(this.x) extends A: B\n",
            "Class 'P' already extends 'A'",
            1,
            28,
        ),
        (
            "class P(this.x) 5\n",
            "Expected ':' but found number 5",
            1,
            17,
        ),
        (
            "class P:\n    def init(this.x)\n",
            "Expected ':' but found end of line",
            2,
            20,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
        let plain = source.replace("this.", "");
        assert_eq!(syntax_error(&plain).0, message, "{plain}");
    }
    for (source, column) in [
        ("class P(this.x, this.y\n", 21),
        ("class P(this.x, this.this.y)\n", 21),
    ] {
        assert_eq!(
            syntax_error(source),
            (PARAMETER.to_string(), Position::new(1, column)),
            "{source}"
        );
    }
    let error = Runtime::new().run("class P(this.value)\nP(5)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Cannot use 'value' as a property");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
}

#[test]
fn tentative_statements_take_the_reading_that_failed_before() {
    for (source, expected) in [
        (
            "a, b = [function() { class P(this.x) }, 1]\na()\n[P(4).x, b]",
            "[4,1]",
        ),
        (
            "{k} = {k: function() { class P(this.x) }}\nk()\nP(4).x",
            "4",
        ),
        (
            "{k: v} = {k: function() { class P(this.x) }}\nv()\nP(4).x",
            "4",
        ),
        (
            "suppose f = function() { class P(this.x) }:\n    f()\n    v = P(3).x\n    assert(v, 3)\n1",
            "1",
        ),
        (
            "n = 0\nwhile n < (function() { class P(this.x); return 2 })():\n    n += 1\n[n, P(1).x]",
            "[2,1]",
        ),
        (
            "if (function() { class P(this.x); return true })():\n    r = P(5).x\nr",
            "5",
        ),
        (
            "x = 1 if (function() { class P(this.x); return true })() else 2\n[x, P(3).x]",
            "[1,3]",
        ),
        (
            "s = `${(function() { class P(this.x); return 1 })()}`\n[s, P(9).x]",
            "[\"1\",9]",
        ),
        (
            "class P(this.status)\np = P(\"DONE\")\nif p.status is DONE:\n    r = \"yes\"\nr",
            "\"yes\"",
        ),
        (
            "class P(this.n)\np = P(0)\nwhile p.n is not 3:\n    p.n += 1\np.n",
            "3",
        ),
        (
            "v = 1\nwhile v is () => { class P(this.x) }:\n    v = 2\nv",
            "1",
        ),
    ] {
        fresh(source, expected);
    }
    let error = Runtime::new()
        .run("n = 0\nwhile () => { class P(this.x) }:\n    n = 1\nn")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Loop did not terminate");
}

#[test]
fn a_colon_parent_still_adopts_an_unindented_next_statement() {
    let source = "class Base(this.a)\nclass Sub(this.b): Base\nx = 5\ns = Sub(1)\n[s.b, x]";
    let tree = statements(source);
    let Some(Stmt::Class(sub)) = tree.get(1) else {
        panic!("a second class");
    };
    assert_eq!(sub.parent.as_deref(), Some("Base"));
    assert_eq!(sub.constructor, statements("this.b = b\nx = 5"));
    assert_eq!(tree.len(), 4);
    fresh(source, "[1,5]");
    for source in [
        "class Base(a):\n    this.a = a\nclass Sub(b): Base\nx = 5\nx",
        "class Base(this.a)\nclass Sub(this.b): Base\nx = 5\nx",
    ] {
        let error = Runtime::new().run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference, "{source}");
        assert_eq!(error.message(), "x is not defined", "{source}");
    }
    fresh(
        "class Base(this.a)\nclass Sub(this.b) extends Base\nx = 5\nx",
        "5",
    );
}

#[test]
fn many_prefixed_parameters_parse_and_construct_in_one_pass() {
    let names: Vec<String> = (0..1000).map(|index| format!("p{index}")).collect();
    let header = names
        .iter()
        .map(|name| format!("this.{name}"))
        .collect::<Vec<_>>()
        .join(", ");
    let values = (0..1000)
        .map(|index| index.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!("class Wide({header})\nw = Wide({values})\n[w.p0, w.p500, w.p999]");
    let tree = statements(&source);
    let Some(Stmt::Class(class)) = tree.first() else {
        panic!("a class");
    };
    assert_eq!(class.parameters.len(), 1000);
    assert_eq!(class.constructor.len(), 1000);
    assert_eq!(class.constructor.last(), Some(&assignment("p999")));
    fresh(&source, "[0,500,999]");
}

#[test]
fn only_the_appended_cases_use_initializing_parameters_and_run_every_assertion() {
    let documents = [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ];
    let appended: Vec<&str> = SPEC_TITLES
        .into_iter()
        .chain([REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE])
        .collect();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flatten() {
        cases += 1;
        let new = appended.contains(&case.title.as_str());
        assert_eq!(uses_prefix(&case.source), new, "{}", case.title);
        if new {
            let count = common::assertions::protected_assertions(&statements(&case.source)).len();
            common::check_with_assertions(case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assertions += count;
        }
    }
    for title in &appended {
        let matching = documents
            .iter()
            .flatten()
            .filter(|case| case.title == *title)
            .count();
        assert_eq!(matching, 1, "{title}");
    }
    assert_eq!(assertions, 36);
    assert_eq!(cases, 1072);
    assert!(uses_prefix("class P(a, ...this.b)"));
    assert!(uses_prefix("class P:\n    def init(self.a):\n        pass"));
    assert!(!uses_prefix("class P(a):\n    this.a = f(this.b)"));
    assert!(!uses_prefix("def f(x) => g(this.x)"));
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Initializing constructor parameters\n")
        .expect("the language reference documents initializing parameters");
    let (_, example) = section
        .split_once("\n```\n")
        .expect("the section shows an example");
    let (example, _) = example.split_once("\n```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 87](nuc-0087.md)"));
}
