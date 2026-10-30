mod common;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::{Expr, LogicalOp, Stmt, UnaryOp, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn equivalent(source: &str, expected: &str) -> Vec<Stmt> {
    let expected = parse(expected).unwrap_or_else(|error| panic!("{expected}: {error}"));
    let actual = parse(source).unwrap_or_else(|error| panic!("{source}: {error}"));
    assert_eq!(actual, expected, "{source}");
    assert_eq!(generate_all(&actual), generate_all(&expected), "{source}");
    actual
}

fn coalesce(left: Expr, right: Expr) -> Expr {
    Expr::Logical {
        operator: LogicalOp::Coalesce,
        left: Box::new(left),
        right: Box::new(right),
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

fn expect(runtime: &mut Runtime, source: &str, expected: &str) {
    let assertions = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let case = common::Case {
        title: "nullish coalescing regression".to_string(),
        source: source.to_string(),
        expected: Some(expected.to_string()),
    };
    common::check_with_count(runtime, &case, assertions)
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn fresh(source: &str, expected: &str) {
    expect(&mut Runtime::new(), source, expected);
}

fn text(value: &str) -> Value {
    Value::String(value.to_string())
}

fn dependencies(runtime: &Runtime, variable: &str) -> Vec<String> {
    let mut found: Vec<String> = runtime
        .graph
        .retrieve(&NodeKey::variable(variable))
        .unwrap_or_else(|| panic!("{variable} is not in the graph"))
        .dependencies
        .iter()
        .map(NodeKey::to_string)
        .collect();
    found.sort();
    found
}

#[test]
fn nullish_coalescing_lowers_to_one_logical_node_with_grouped_canonical_source() {
    let expected = coalesce(name("a"), name("b"));
    assert_eq!(parse_expression("a ?? b").unwrap(), expected);
    assert_eq!(parse_expression("a??b").unwrap(), expected);
    assert_eq!(parse_expression("(a ?? b)").unwrap(), expected);
    assert_eq!(expected.to_string(), "(a??b)");

    for (source, canonical) in [
        ("x = a ?? b", "x=(a??b)"),
        ("x = a ?? b ?? c", "x=(a??(b??c))"),
        ("x = (a ?? b) ?? c", "x=((a??b)??c)"),
        ("x = (a ?? 0) > 5", "x=(a??0)>5"),
        ("x = a ?? 0 > 5", "x=(a??0>5)"),
        ("x = a || b ?? c", "x=(a||b??c)"),
        ("x = a || (b ?? c)", "x=a||(b??c)"),
        ("x = a ?? b ? c : d", "x=((a??b)?c:d)"),
        ("x = c ? a : b ?? d", "x=(c?a:(b??d))"),
        ("x ||= a ?? b", "x=x||(a??b)"),
        ("if (x ?? 0) > 5:\n    pass", "if((x??0)>5){pass}"),
        ("if x ?? 0 > 5:\n    pass", "if((x??0>5)){pass}"),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), canonical, "{source}");
    }
}

#[test]
fn nullish_coalescing_binds_between_conditionals_and_reasoning_pipelines() {
    for (source, grouped) in [
        ("x = a ?? b", "x = (a ?? b)"),
        ("x = a ?? b ?? c", "x = a ?? (b ?? c)"),
        ("x = a || b ?? c", "x = (a || b) ?? c"),
        ("x = a or b ?? c", "x = (a || b) ?? c"),
        ("x = a ?? b || c", "x = a ?? (b || c)"),
        ("x = a ?? b or c", "x = a ?? (b || c)"),
        ("x = a && b ?? c", "x = (a && b) ?? c"),
        ("x = a ?? b and c", "x = a ?? (b && c)"),
        ("x = a ?? b == c", "x = a ?? (b == c)"),
        ("x = a ?? b > c", "x = a ?? (b > c)"),
        ("x = a ?? b + 1", "x = a ?? (b + 1)"),
        ("x = a ?? -b", "x = a ?? (-b)"),
        ("x = not a ?? b", "x = (!a) ?? b"),
        ("x = !a ?? b", "x = (!a) ?? b"),
        ("x = typeof a ?? b", "x = (typeof a) ?? b"),
        ("x = a.b ?? c[0]", "x = (a.b) ?? (c[0])"),
        ("x = a ?? b |> why", "x = a ?? (b |> why)"),
        ("x = a |> why ?? b", "x = (a |> why) ?? b"),
        ("x = a ?? b ? c : d", "x = (a ?? b) ? c : d"),
        ("x = c ? a ?? b : d", "x = c ? (a ?? b) : d"),
        ("x = c ? a : b ?? d", "x = c ? a : (b ?? d)"),
        (
            "x = c ? a : b ?? d ? e : f",
            "x = c ? a : ((b ?? d) ? e : f)",
        ),
        ("x = a ?? b if c else d", "x = c ? (a ?? b) : d"),
        ("x = d if a ?? b else e", "x = (a ?? b) ? d : e"),
        ("x = d if c else a ?? b", "x = c ? d : (a ?? b)"),
        ("x = y = a ?? b", "x = (y = (a ?? b))"),
        ("x += a ?? 1", "x = x + (a ?? 1)"),
        ("x ||= a ?? b", "x = x || (a ?? b)"),
        ("x &&= a ?? b", "x = x && (a ?? b)"),
        ("f = v => v ?? 0", "f = v => (v ?? 0)"),
        ("f = v -> v ?? 0", "f = v => (v ?? 0)"),
        ("f = lambda v: v ?? 0", "f = (v) => (v ?? 0)"),
        ("def f(v) => v ?? 0", "def f(v):\n    return v ?? 0"),
        ("x = [a ?? 1, b]", "x = [(a ?? 1), b]"),
        ("x = { k: a ?? 1 b: 2 }", "x = { k: (a ?? 1), b: 2 }"),
        ("x = f(a ?? 1, b)", "x = f((a ?? 1), b)"),
        ("x = items[i ?? 0]", "x = items[(i ?? 0)]"),
        ("x = items[i ?? 0 : j ?? 2]", "x = items[(i ?? 0):(j ?? 2)]"),
        ("x = `${a ?? b}`", "x = `${(a ?? b)}`"),
    ] {
        equivalent(source, grouped);
    }
    assert_eq!(
        parse_expression("not a ?? b").unwrap(),
        coalesce(
            Expr::Unary {
                operator: UnaryOp::Not,
                operand: Box::new(name("a")),
            },
            name("b"),
        )
    );
}

#[test]
fn coalescing_chains_respect_the_existing_operator_limit() {
    let chain = |operators: usize| format!("x = {}7\nx", "null ?? ".repeat(operators));
    let mut runtime = Runtime::new();
    assert_eq!(run(&mut runtime, &chain(128)), Value::Number(7.0));
    assert_eq!(
        syntax_error(&chain(129)),
        (
            "Expression has too many operands".to_string(),
            Position::new(1, 1037)
        )
    );
    let deep = format!("x = {}1{}", "(null ?? ".repeat(60), ")".repeat(60));
    assert_eq!(run(&mut Runtime::new(), &deep), Value::Number(1.0));
    let too_deep = format!("x = {}1{}", "(null ?? ".repeat(70), ")".repeat(70));
    assert_eq!(
        syntax_error(&too_deep).0,
        "Expressions are nested too deeply"
    );
}

#[test]
fn the_left_operand_is_null_exactly_when_its_declaration_would_store_null() {
    for (source, expected) in [
        ("x = null\ny = x ?? 5\ny", "5"),
        ("y = null ?? 5\ny", "5"),
        ("y = undefined ?? 5\ny", "5"),
        ("y = None ?? 5\ny", "5"),
        ("n = null\ny = (n + 1) ?? 0\ny", "0"),
        ("n = null\ny = (n * 2) ?? 3\ny", "3"),
        ("x = null\ny = (x == null) ?? \"unknown\"\ny", "\"unknown\""),
        ("x = null\ny = (typeof x) ?? \"unknown\"\ny", "\"unknown\""),
        ("flag = 0\nbackup = null\ny = flag || backup ?? 9\ny", "9"),
        ("flag = 0\nbackup = 4\ny = flag || backup ?? 9\ny", "4"),
        ("p = {}\ny = p.name ?? \"anon\"\ny", "\"anon\""),
        (
            "p = {}\np.nick = null\ny = p.nick ?? \"anon\"\ny",
            "\"anon\"",
        ),
        ("y = [1][5] ?? 4\ny", "4"),
        ("record = {}\ny = record[\"key\"] ?? 6\ny", "6"),
        ("def f():\n    return null\ny = f() ?? 2\ny", "2"),
        ("def g():\n    pass\ny = g() ?? 3\ny", "3"),
        ("x = null\ndef h():\n    return x\ny = h() ?? 8\ny", "8"),
        ("x = 0\ny = x ?? 5\ny", "0"),
        ("x = \"\"\ny = x ?? 5\ny", "\"\""),
        ("x = false\ny = x ?? 5\ny", "false"),
        ("x = []\ny = x ?? 5\ny", "[]"),
        ("x = [null]\ny = x ?? 5\ny", "[null]"),
        ("a = null\nb = null\ny = a ?? b\ny", "null"),
        ("a = null\nb = null\ny = a ?? b ?? 1\ny", "1"),
        ("y = [1, null, 3].map(v => v ?? 0)\ny", "[1,0,3]"),
        ("y = [1, null, 3].map(v => v || 0)\ny", "null"),
        ("y = [1, null, 3].filter(v => (v ?? 0) > 0)\ny", "[1,3]"),
        ("x = null\nx ?? 7", "7"),
    ] {
        fresh(source, expected);
    }
    let nan = run(&mut Runtime::new(), "y = (0 / 0) ?? 1\ny");
    assert!(matches!(nan, Value::Number(number) if number.is_nan()));
    fresh("class Box\nbox = Box()\ny = box ?? 1\ny == box", "true");
}

#[test]
fn nullishness_differs_from_truthiness_only_for_falsy_values_that_are_not_null() {
    for (literal, nullish) in [
        ("0", false),
        ("1", false),
        ("-1", false),
        ("\"\"", false),
        ("\"a\"", false),
        ("[]", false),
        ("[0]", false),
        ("{}", false),
        ("true", false),
        ("false", false),
        ("null", true),
        ("undefined", true),
        ("box.missing", true),
        ("box", false),
        ("Box", false),
        ("x => x", false),
        ("/a/", false),
    ] {
        let mut runtime = Runtime::new();
        run(&mut runtime, "class Box\nbox = Box()");
        let falsy = run(&mut runtime, &format!("not ({literal})")) == Value::Bool(true);
        for (operator, defaulted) in [("??", nullish), ("||", falsy)] {
            let picked = run(
                &mut runtime,
                &format!("picked = ({literal}) {operator} \"d\"\npicked == \"d\""),
            );
            assert_eq!(picked, Value::Bool(defaulted), "{literal} {operator}");
        }
        assert!(falsy || !nullish, "{literal}");
    }
}

#[test]
fn the_right_operand_runs_only_when_needed_and_replaces_obsolete_dependencies() {
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "calls = []\ndef note(tag):\n    calls.push(tag)\n    return tag\nsaved = \"draft\"\nother = 1\npick = saved ?? note(other)",
    );
    assert_eq!(dependencies(&runtime, "pick"), ["saved"]);
    expect(&mut runtime, "other = 2\n[pick, calls]", "[\"draft\",[]]");
    run(&mut runtime, "saved = null");
    expect(&mut runtime, "[pick, calls]", "[2,[2]]");
    assert_eq!(
        dependencies(&runtime, "pick"),
        ["calls", "note", "other", "saved"]
    );
    run(&mut runtime, "saved = 0");
    assert_eq!(dependencies(&runtime, "pick"), ["saved"]);
    expect(&mut runtime, "other = 5\n[pick, calls]", "[0,[2]]");

    for (source, expected) in [
        ("y = 5 ?? missing\ny", "5"),
        ("y = 5 ?? missing.deeper\ny", "5"),
        ("y = 5 ?? (1 / 0)\ny", "5"),
        ("y = 0 ?? missing\ny", "0"),
    ] {
        fresh(source, expected);
    }
    for source in ["y = missing ?? 5", "y = null ?? missing", "missing ?? 5"] {
        let error = Runtime::new().run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference, "{source}");
        assert_eq!(error.message(), "missing is not defined", "{source}");
    }

    let mut runtime = Runtime::new();
    run(&mut runtime, "ready = 1\nsafe = ready ?? missing");
    let error = runtime.run("ready = null").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    expect(&mut runtime, "[ready, safe]", "[1,1]");
    run(&mut runtime, "missing = 7\nready = null");
    expect(&mut runtime, "safe", "7");
}

#[test]
fn null_reads_in_the_left_operand_are_absorbed_and_right_reads_still_count() {
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "first = null\nsecond = null\npicked = first ?? second\nlabel = (first ?? second) ?? \"none\"",
    );
    expect(&mut runtime, "[picked, label]", "[null,\"none\"]");
    assert_eq!(dependencies(&runtime, "picked"), ["first", "second"]);
    run(&mut runtime, "second = \"b\"");
    expect(&mut runtime, "[picked, label]", "[\"b\",\"b\"]");
    run(&mut runtime, "first = \"a\"");
    expect(&mut runtime, "[picked, label]", "[\"a\",\"a\"]");
    assert_eq!(dependencies(&runtime, "picked"), ["first"]);

    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "stock = null\nother = null\ntotal = (stock ?? 0) + 1\nsum = other + (stock ?? 0)",
    );
    expect(&mut runtime, "[total, sum]", "[1,null]");
    run(&mut runtime, "other = 2");
    expect(&mut runtime, "[total, sum]", "[1,2]");
}

#[test]
fn class_level_rules_validate_both_operands_and_evaluate_each_instance() {
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "class Order(coupon):\n    this.coupon = coupon\n$Order.discount = $Order.coupon ?? 0\n$Order.legacy = $Order.coupon || 0\nsmall = Order(null)\nlarge = Order(25)\nzero = Order(0)",
    );
    expect(
        &mut runtime,
        "[small.discount, large.discount, zero.discount, small.legacy]",
        "[0,25,0,null]",
    );
    run(&mut runtime, "small.coupon = 5\nlarge.coupon = null");
    expect(&mut runtime, "[small.discount, large.discount]", "[5,0]");

    for source in [
        "$Order.note = $Order.memo ?? vip",
        "$Order.note = vip ?? $Order.memo",
        "$Order.note = $Order.memo || vip",
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference, "{source}");
        assert_eq!(error.message(), "vip is not defined", "{source}");
    }
    run(&mut runtime, "$Order.limit = $Order.cap ?? 50");
    let error = runtime.run("$Order.cap = $Order.limit + 1").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "Circular Dependency");
    expect(&mut runtime, "small.limit", "50");
}

#[test]
fn class_level_if_conditions_decide_once_a_default_replaces_an_undefined_read() {
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "class Device\nif $Device.profile ?? false:\n    $Device.mode = \"managed\"\nelse:\n    $Device.mode = \"open\"\nif $Device.profile:\n    $Device.state = \"on\"\nelse:\n    $Device.state = \"off\"\nif ($Device.profile || false) ?? true:\n    $Device.plain = \"yes\"\nelse:\n    $Device.plain = \"no\"\nphone = Device()",
    );
    expect(
        &mut runtime,
        "[phone.mode, phone.state, phone.plain]",
        "[\"open\",null,null]",
    );
    run(&mut runtime, "phone.profile = \"P\"");
    expect(
        &mut runtime,
        "[phone.mode, phone.state, phone.plain]",
        "[\"managed\",\"on\",\"yes\"]",
    );
    run(&mut runtime, "phone.profile = null");
    expect(
        &mut runtime,
        "[phone.mode, phone.state, phone.plain]",
        "[\"open\",\"off\",\"yes\"]",
    );
}

#[test]
fn statement_conditions_and_suites_accept_coalescing_headers() {
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "stock = null\nif (stock ?? 0) > 3:\n    level = \"high\"\nelse:\n    level = \"low\"\nif stock ?? false\n    flag = \"set\"\nelse stock === 0\n    flag = \"zero\"\nelse\n    flag = \"unset\"",
    );
    expect(&mut runtime, "[level, flag]", "[\"low\",\"unset\"]");
    run(&mut runtime, "stock = 5");
    expect(&mut runtime, "[level, flag]", "[\"high\",\"set\"]");
    run(&mut runtime, "stock = 0");
    expect(&mut runtime, "[level, flag]", "[\"low\",\"zero\"]");
}

#[test]
fn nullish_coalescing_keeps_the_existing_assignment_machinery() {
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "base = 3\nconfig = { size: 2 }\nspare = config.size ?? base.value\npicked = null\nfrozen = picked ?? base.value\nbase = 4",
    );
    expect(&mut runtime, "[spare, frozen]", "[2,3]");
    run(&mut runtime, "delete config.size\npicked = 9");
    expect(&mut runtime, "[spare, frozen]", "[3,9]");
    run(&mut runtime, "picked = null");
    expect(&mut runtime, "frozen", "3");

    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "bonus = 7\nreward = bonus ?? 0\nchain = reward + 1",
    );
    run(&mut runtime, "delete bonus");
    expect(&mut runtime, "[reward, chain]", "[0,1]");
    run(&mut runtime, "bonus = 9");
    expect(&mut runtime, "[reward, chain]", "[9,10]");

    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "def check(amount):\n    if amount > 100:\n        throw \"TOO_LARGE\"\n    return amount\ncustom = null\namount = 5\nfee = custom ?? check(amount)",
    );
    let error = runtime.run("amount = 500").unwrap_err();
    assert_eq!(error.thrown_value(), Some(&text("TOO_LARGE")));
    expect(&mut runtime, "[amount, fee]", "[5,5]");
    run(&mut runtime, "custom = 1\namount = 500");
    expect(&mut runtime, "[amount, fee]", "[500,1]");

    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "class Account\n$Account.opening = $Account.opening@first ?? 0\naccount = Account()",
    );
    expect(&mut runtime, "account.opening", "null");
    run(&mut runtime, "account.opening = 7");
    expect(&mut runtime, "account.opening", "7");

    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "price = 10\nchange = price - (price@previous ?? price)",
    );
    expect(&mut runtime, "change", "0");
    run(&mut runtime, "price = 14");
    expect(&mut runtime, "change", "4");
}

#[test]
fn template_interpolations_subscripts_and_arguments_stay_live() {
    let mut runtime = Runtime::new();
    run(
        &mut runtime,
        "user = {}\nitems = [\"a\", \"b\", \"c\"]\ngreeting = `Hi ${user.name ?? \"guest\"}`\npicked = items[user.index ?? 2]\nwidest = Math.max(user.width ?? 0, 4)\nlabel = `${user.name ?? \"guest\" if user.active else \"away\"}`",
    );
    expect(
        &mut runtime,
        "[greeting, picked, widest, label]",
        "[\"Hi guest\",\"c\",4,\"guest\"]",
    );
    run(
        &mut runtime,
        "user.name = \"Ann\"\nuser.index = 0\nuser.width = 9\nuser.active = false",
    );
    expect(
        &mut runtime,
        "[greeting, picked, widest, label]",
        "[\"Hi Ann\",\"a\",9,\"Ann\"]",
    );
}

#[test]
fn literal_question_marks_and_conditional_expressions_are_unchanged() {
    let tokens = |source: &str| -> Vec<Token> {
        tokenize(source)
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect()
    };
    assert_eq!(
        tokens("note = \"ready??\" # why ??"),
        [
            Token::Identifier("note".into()),
            Token::Assign,
            Token::Str("ready??".into()),
            Token::Eof
        ]
    );
    assert_eq!(
        tokens("a?.5:1"),
        [
            Token::Identifier("a".into()),
            Token::Question,
            Token::Number(0.5),
            Token::Colon,
            Token::Number(1.0),
            Token::Eof
        ]
    );
    let mut runtime = Runtime::new();
    for (source, expected) in [
        ("\"why??\"", text("why??")),
        ("'why??'", text("why??")),
        ("/colou??r/.test(\"color\")", Value::Bool(true)),
        ("ready = true # ready??\nready ? 1 : 2", Value::Number(1.0)),
        ("`why?? ${ready ? \"yes\" : \"no\"}`", text("why?? yes")),
        ("ready ? null ?? 3 : 4", Value::Number(3.0)),
        ("a = null\na ?? /x?/.test(\"\")", Value::Bool(true)),
    ] {
        assert_eq!(run(&mut runtime, source), expected, "{source}");
    }
}

#[test]
fn malformed_coalescing_reports_exact_errors() {
    for (source, message, line, column) in [
        ("x = c ?? 1 : 2", "Unexpected ':'", 1, 14),
        ("x = ?? b", "Unexpected '??'", 1, 8),
        ("x ??= 1", "Unexpected '='", 1, 7),
        ("x = a ??= b", "Unexpected '='", 1, 11),
        ("x = a ??? b", "Unexpected '?'", 1, 11),
        ("x = a ??", "Unexpected end of input", 1, 9),
        ("x = a ??\n", "Unexpected end of line", 2, 1),
        ("x = a ?? ? b : c", "Unexpected '?'", 1, 12),
        ("x = c ? a ?? : b", "Unexpected ':'", 1, 16),
        ("x = (a ??)", "Unexpected ')'", 1, 11),
        ("a = 1\nb = ??", "Unexpected '??'", 2, 7),
        ("x = a?? .b", "Unexpected '.'", 1, 10),
        ("x = `${a ?? }`", "Unexpected end of input", 1, 6),
        ("a ?? b += 1", "Invalid compound assignment target", 1, 8),
        ("x = a ? ? b", "Unexpected '?'", 1, 11),
        ("x = a?.b", "Unexpected '.'", 1, 8),
        ("x = a ?: b", "Unexpected ':'", 1, 10),
        (
            "def f left: int??, right => left",
            "Expected '(' but found identifier 'left'",
            1,
            7,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    let error = Runtime::new().run("x = a ?? b = 1").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(error.message(), "Cannot assign to (a??b)");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
    assert_eq!(
        Runtime::new().run("x = a ??.5").unwrap_err().message(),
        "a is not defined"
    );
}

#[test]
fn parse_expression_reads_coalescing_and_ignores_what_follows() {
    let expected = coalesce(name("a"), name("b"));
    assert_eq!(parse_expression("a ?? b").unwrap(), expected);
    assert_eq!(parse_expression("a ?? b c").unwrap(), expected);
    assert_eq!(parse_expression("a b ?? c").unwrap(), name("a"));
    assert_eq!(
        parse_expression("a ?? b ?? c").unwrap(),
        coalesce(name("a"), coalesce(name("b"), name("c")))
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
fn the_native_repl_buffers_colon_free_headers_containing_coalescing() {
    let output = repl(
        "ready = null\nif ready ?? true\n    result = \"defaulted\"\n\nresult\nlabel = ready ?? \"off\"\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stderr, b"");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{BANNER}null\n\"defaulted\"\n\"defaulted\"\n\"off\"\n")
    );
}
