mod common;

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Expr, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime};

const SPEC_TITLES: [&str; 8] = [
    "Nucleoid builds lists with comprehensions",
    "Nucleoid filters comprehensions with an if clause",
    "Nucleoid keeps comprehensions over class populations live",
    "Nucleoid accepts of in comprehensions over lists and populations",
    "Nucleoid writes comprehensions across lines",
    "Nucleoid nests comprehensions that read only their own names",
    "Nucleoid writes comprehensions wherever a list may start",
    "Nucleoid keeps comprehensions identical to their map and filter spellings",
];
const REFERENCE_TITLE: &str = "Nucleoid reads list comprehensions as map and filter";
const README_TITLE: &str = "List comprehensions keep Python-style lists live";
const EXAMPLES_TITLE: &str = "List comprehensions keep a class's shipping labels live";

const COMPREHENSION: &str = "[x * 2 for x in xs if x > 1]";
const SPELLED: &str = "xs.filter(x => x > 1).map(x => x * 2)";

fn uses_comprehension(source: &str) -> bool {
    let mut brackets = Vec::new();
    for spanned in tokenize(source).unwrap() {
        match spanned.token {
            Token::LeftBracket => brackets.push(true),
            Token::LeftParen | Token::LeftBrace => brackets.push(false),
            Token::RightBracket | Token::RightParen | Token::RightBrace => {
                brackets.pop();
            }
            Token::Keyword(Keyword::For) if brackets.last() == Some(&true) => return true,
            _ => {}
        }
    }
    false
}

fn equivalent(comprehension: &str, spelled: &str) {
    let left = parse(comprehension).unwrap_or_else(|error| panic!("{comprehension}: {error}"));
    let right = parse(spelled).unwrap_or_else(|error| panic!("{spelled}: {error}"));
    assert_eq!(left, right, "{comprehension}");
    assert_eq!(generate_all(&left), generate_all(&right), "{comprehension}");
}

fn syntax_error(source: &str, message: &str, line: usize, column: usize) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    assert_eq!(error.message(), message, "{source}");
    assert_eq!(
        error.position(),
        Some(Position::new(line, column)),
        "{source}"
    );
}

fn failure(source: &str) -> (ErrorKind, String) {
    let error = Runtime::new().run(source).unwrap_err();
    (error.kind(), error.message().to_string())
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

fn normalized(text: &str) -> String {
    let mut seen: Vec<&str> = Vec::new();
    let mut result = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("obj:") {
        let digits = after
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(after.len());
        let (identifier, tail) = after.split_at(digits);
        let ordinal = seen
            .iter()
            .position(|seen| *seen == identifier)
            .unwrap_or_else(|| {
                seen.push(identifier);
                seen.len() - 1
            });
        result.push_str(before);
        result.push_str("obj:");
        result.push_str(&ordinal.to_string());
        rest = tail;
    }
    result.push_str(rest);
    result
}

fn outcome(source: &str) -> String {
    let mut runtime = Runtime::new();
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    let json = runtime.serialize_json(&value).unwrap();
    assert!(runtime.take_assertions().is_empty(), "{source}");
    normalized(&format!("{json} {:?}", graph_shape(&runtime)))
}

fn nested(depth: usize) -> String {
    let mut expression = "1".to_string();
    for level in 0..depth {
        expression = format!("[{expression} for v{level} in xs]");
    }
    format!("xs = [1]\n{expression}")
}

#[test]
fn comprehensions_build_the_map_and_filter_trees() {
    for (comprehension, spelled) in [
        ("[x * 2 for x in xs]", "xs.map(x => x * 2)"),
        ("[x * 2 for x of xs]", "xs.map(x => x * 2)"),
        (
            "[x for x in xs if x > 1]",
            "xs.filter(x => x > 1).map(x => x)",
        ),
        (
            "[x for x of xs if x > 1]",
            "xs.filter(x => x > 1).map(x => x)",
        ),
        (
            "[item.name for item in Item if item.price <= budget]",
            "Item.filter(item => item.price <= budget).map(item => item.name)",
        ),
        (
            "[[cell * 10 for cell in row] for row in grid]",
            "grid.map(row => row.map(cell => cell * 10))",
        ),
        (
            "[x for x in [y for y in ys if y]]",
            "ys.filter(y => y).map(y => y).map(x => x)",
        ),
        (
            "[\"high\" if s > 80 else \"low\" for s in scores]",
            "scores.map(s => \"high\" if s > 80 else \"low\")",
        ),
        (
            "[s > 80 ? 1 : 0 for s in scores if s ?? false]",
            "scores.filter(s => s ?? false).map(s => s > 80 ? 1 : 0)",
        ),
        ("[f => f(x) for x in xs]", "xs.map(x => f => f(x))"),
        ("[lambda: x for x in xs]", "xs.map(x => lambda: x)"),
        ("[x for x in a.b(c)[0]]", "a.b(c)[0].map(x => x)"),
        ("[x for x in c ? a : b]", "(c ? a : b).map(x => x)"),
        (
            "[x for x in (a if c else b)]",
            "(a if c else b).map(x => x)",
        ),
        (
            "[x for x in xs if a if b else c]",
            "xs.filter(x => a if b else c).map(x => x)",
        ),
        ("[x for x in [1, 2]]", "[1, 2].map(x => x)"),
        ("[x for x in sum of xs]", "(sum of xs).map(x => x)"),
        (
            "[x for x in all y in ys: y > 0]",
            "(all y in ys: y > 0).map(x => x)",
        ),
        (
            "[x is 3 for x in xs if x is not 4]",
            "xs.filter(x => x is not 4).map(x => x is 3)",
        ),
        ("[x for lambda in xs]", "xs.map(lambda => x)"),
        (
            "[\n  x\n    for x in xs\n  if x\n]",
            "xs.filter(x => x).map(x => x)",
        ),
        (
            "[x for x in xs if x].length",
            "xs.filter(x => x).map(x => x).length",
        ),
        ("y = [x for x in xs]", "y = xs.map(x => x)"),
    ] {
        equivalent(comprehension, spelled);
    }
    assert_eq!(
        generate_all(&parse(COMPREHENSION).unwrap()),
        "xs.filter(function()).map(function())"
    );
}

#[test]
fn comprehensions_stand_wherever_a_list_literal_may() {
    for context in [
        "@",
        "y = @",
        "y = (@)",
        "f(@)",
        "f(1, @)",
        "Item(@)",
        "y = {k: @, j: 1}",
        "y = {k: 1, j: @}",
        "y = [@, 1]",
        "y = [1, @]",
        "y = `${@}`",
        "y = `a ${@} b ${1}`",
        "def g(xs):\n    return @",
        "def g(xs) => @",
        "if @:\n    pass",
        "if c:\n    pass\nelse if @.length:\n    pass",
        "for v in @:\n    pass",
        "for v of @: pass",
        "while @.length > 3:\n    pass",
        "y = c ? @ : 1",
        "y = c ? 1 : @",
        "y = @ if c else 1",
        "y = 1 if @ else 2",
        "y = a => @",
        "y = lambda: @",
        "y = all v in @: v > 0",
        "y = @ ?? []",
        "y = @.length",
        "y = typeof @",
        "assert(@, 1)",
        "assert: @, 1",
        "throw @",
        "a, b = @",
        "{a, b} = @",
        "x = 1; @",
        "x = 1\n@",
        "{ @ }",
        "y = suppose: @",
        "y = function (xs) { return @ }",
    ] {
        equivalent(
            &context.replace('@', COMPREHENSION),
            &context.replace('@', SPELLED),
        );
    }
    for (comprehension, spelled) in [
        (
            "[x /* each */ for x in xs // all\n  if x > 1]",
            "xs.filter(x => x > 1).map(x => x)",
        ),
        (
            "t = `${[x for /* x */ x in xs]}`",
            "t = `${xs.map(x => x)}`",
        ),
    ] {
        assert_eq!(
            parse(comprehension).unwrap(),
            parse(spelled).unwrap(),
            "{comprehension}"
        );
    }
    assert_eq!(
        parse_expression(COMPREHENSION).unwrap(),
        parse_expression(SPELLED).unwrap()
    );
}

#[test]
fn malformed_comprehensions_report_exact_syntax_errors() {
    for (source, message, line, column) in [
        ("[x for x in xs, y]", "Expected ']' but found ','", 1, 15),
        ("[x for x in xs,]", "Expected ']' but found ','", 1, 15),
        (
            "[x for y in ys for x in y]",
            "Expected ']' but found keyword 'for'",
            1,
            16,
        ),
        (
            "[x for x in xs if x > 1 for y in ys]",
            "Expected ']' but found keyword 'for'",
            1,
            25,
        ),
        (
            "[x for x in xs if a if b]",
            "Expected ']' but found keyword 'if'",
            1,
            21,
        ),
        (
            "[x for x in a if c else b]",
            "Expected ']' but found keyword 'else'",
            1,
            20,
        ),
        (
            "[a for a, b in pairs]",
            "Expected 'in' or 'of' but found ','",
            1,
            9,
        ),
        (
            "[a for (a, b) in pairs]",
            "Expected a name but found '('",
            1,
            8,
        ),
        (
            "[x for this in xs]",
            "Expected a name but found keyword 'this'",
            1,
            8,
        ),
        (
            "[x for self in xs]",
            "Expected a name but found keyword 'this'",
            1,
            8,
        ),
        (
            "[x for $Item in xs]",
            "Expected a name but found '$Item'",
            1,
            8,
        ),
        (
            "[x for 1 in xs]",
            "Expected a name but found number 1",
            1,
            8,
        ),
        ("[x for]", "Expected a name but found ']'", 1, 7),
        (
            "[x for x xs]",
            "Expected 'in' or 'of' but found identifier 'xs'",
            1,
            10,
        ),
        ("[x for x]", "Expected 'in' or 'of' but found ']'", 1, 9),
        (
            "[x for x = xs]",
            "Expected 'in' or 'of' but found '='",
            1,
            10,
        ),
        ("[x for x in]", "Unexpected ']'", 1, 13),
        ("[x for x in xs if]", "Unexpected ']'", 1, 19),
        (
            "[x for x in xs",
            "Expected ']' but found end of input",
            1,
            15,
        ),
        (
            "[x for x in xs\ny = 1",
            "Expected ']' but found identifier 'y'",
            2,
            1,
        ),
        (
            "[x\n for 1 in xs]",
            "Expected a name but found number 1",
            2,
            6,
        ),
    ] {
        syntax_error(source, message, line, column);
    }
    for (comprehension, spelled, message) in [
        (
            "xs = [1]\n[x for x in xs] = 1",
            "xs = [1]\nxs.map(x => x) = 1",
            "Cannot assign to xs.map(function())",
        ),
        (
            "xs = [1]\n[x for x in xs if x] = 1",
            "xs = [1]\nxs.filter(x => x).map(x => x) = 1",
            "Cannot assign to xs.filter(function()).map(function())",
        ),
    ] {
        assert!(Runtime::check(comprehension).is_ok(), "{comprehension}");
        let mut runtime = Runtime::new();
        let error = runtime.run(comprehension).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{comprehension}");
        assert_eq!(error.message(), message, "{comprehension}");
        assert_eq!(
            error.position(),
            Some(Position::new(2, 1)),
            "{comprehension}"
        );
        assert_eq!(failure(spelled), (ErrorKind::Syntax, message.to_string()));
        assert_eq!(
            runtime.run("xs").unwrap_err().message(),
            "xs is not defined"
        );
    }
}

#[test]
fn other_brackets_and_generators_keep_their_accepted_errors() {
    for (source, message, line, column) in [
        (
            "(x for x in xs)",
            "Expected ')' but found keyword 'for'",
            1,
            4,
        ),
        (
            "f(x for x in xs)",
            "Expected ')' but found keyword 'for'",
            1,
            5,
        ),
        (
            "xs[x for x in ys]",
            "Expected ']' but found keyword 'for'",
            1,
            6,
        ),
        (
            "Item[x for x in ys]",
            "Expected ']' but found keyword 'for'",
            1,
            8,
        ),
        (
            "xs[1:x for x in ys]",
            "Expected ']' but found keyword 'for'",
            1,
            8,
        ),
        (
            "y = {x for x in xs}",
            "Expected ':' but found keyword 'for'",
            1,
            8,
        ),
        (
            "y = {k: v for k in ks}",
            "Expected '}' but found keyword 'for'",
            1,
            11,
        ),
        ("{x for x in xs}", "Expected ':' but found '}'", 1, 15),
        (
            "[x, y for y in ys]",
            "Expected ']' but found keyword 'for'",
            1,
            7,
        ),
        ("[for x in xs]", "Unexpected keyword 'for'", 1, 6),
        (
            "x = 1 [y for y in ys]",
            "Expected ']' but found keyword 'for'",
            1,
            10,
        ),
        ("z = )\nys = [x for x in xs]", "Unexpected ')'", 1, 5),
    ] {
        syntax_error(source, message, line, column);
    }
}

#[test]
fn accepted_inputs_keep_their_readings_and_ignored_tails() {
    let x = || Expr::Identifier("x".to_string());
    for source in [
        "x for x in xs",
        "x if [y for y in ys] else z",
        "x if [y for y in [1]] else 0",
        "x of xs",
        "x in xs",
    ] {
        assert_eq!(parse_expression(source).unwrap(), x(), "{source}");
    }
    assert_eq!(
        parse_expression("x |> why for x in xs").unwrap(),
        parse_expression("x |> why").unwrap()
    );
    for (template, expected) in [
        ("`${x for x in xs}`", "5"),
        ("`${x if [y for y in ys] else z}`", "5"),
        ("`${x ?? 1 for y in [1]}`", "5"),
        ("`${x ? 1 : 2 if [y for y in ys] else 3}`", "1"),
        ("`${[x] for x in xs}`", "5"),
        ("`${[x][0] if [y for y in ys] else 0}`", "5"),
        ("`${(x) in xs}`", "5"),
        ("`${x of xs}`", "5"),
        ("`${x if [y for y in [1]] else 0}`", "5"),
    ] {
        let mut runtime = Runtime::new();
        let value = runtime
            .run(&format!("x = 5\nt = {template}\nt"))
            .unwrap_or_else(|error| panic!("{template}: {error}"));
        assert_eq!(value.to_string(), expected, "{template}");
    }
    let mut runtime = Runtime::new();
    let value = runtime
        .run("x = 5\nt = `${(x if [y for y in [1]] else 0)} ${[y * 2 for y in [1, 2]]}`\nt")
        .unwrap();
    assert_eq!(value.to_string(), "5 2,4");
}

#[test]
fn comprehensions_parse_again_only_after_the_accepted_parse_fails() {
    equivalent(
        "[x for x in xs if x is 3]",
        "xs.filter(x => x is 3).map(x => x)",
    );
    let error = Runtime::check("xs = [1]\nys = [x for x in xs]\nz = a |> foo").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "foo is not an operation");
    syntax_error("ys = [x for x in xs]\nz = )", "Unexpected ')'", 2, 6);
    for later in [
        "z = a |> foo",
        "z = )",
        "t = `${a +}`",
        "t = `${[y for 1 in ys]}`",
    ] {
        let comprehension = Runtime::check(&format!("ys = [x for x in xs]\n{later}")).unwrap_err();
        let spelled = Runtime::check(&format!("ys = xs.map(x => x)\n{later}")).unwrap_err();
        assert_eq!(
            (
                comprehension.kind(),
                comprehension.message(),
                comprehension.position()
            ),
            (spelled.kind(), spelled.message(), spelled.position()),
            "{later}"
        );
    }
}

#[test]
fn comprehension_names_are_local_and_sources_run_once() {
    let mut runtime = Runtime::new();
    let value = runtime
        .run("x = 99\nxs = [1, 2]\nys = [x * 2 for x in xs]\n[x, ys]")
        .unwrap();
    assert_eq!(runtime.serialize_json(&value).unwrap(), "[99.0,[2.0,4.0]]");
    let mut runtime = Runtime::new();
    let value = runtime
        .run(
            "calls = []\ndef source():\n    calls.push(1)\n    return [1, 2, 3]\n[n for n in source() if n > 1]",
        )
        .unwrap();
    assert_eq!(runtime.serialize_json(&value).unwrap(), "[2.0,3.0]");
    let calls = runtime.run("calls.length").unwrap();
    assert_eq!(calls.to_string(), "1");
}

#[test]
fn inherited_lambda_limits_raise_the_arrow_spelling_errors() {
    for (comprehension, spelled, kind, message) in [
        (
            "def scale(xs, k): return [x * k for x in xs]\nscale([1, 2], 3)",
            "def scale(xs, k): return xs.map(x => x * k)\nscale([1, 2], 3)",
            ErrorKind::Reference,
            "k is not defined",
        ),
        (
            "class Box(k):\n    this.k = k\n    def scaled(xs):\n        return [x * this.k for x in xs]\nBox(2).scaled([1, 2])",
            "class Box(k):\n    this.k = k\n    def scaled(xs):\n        return xs.map(x => x * this.k)\nBox(2).scaled([1, 2])",
            ErrorKind::Reference,
            "this is not defined",
        ),
        (
            "grid = [[1, 2]]\n[[cell + row.length for cell in row] for row in grid]",
            "grid = [[1, 2]]\ngrid.map(row => row.map(cell => cell + row.length))",
            ErrorKind::Reference,
            "row is not defined",
        ),
        (
            "[c for c in \"abc\"]",
            "\"abc\".map(c => c)",
            ErrorKind::Type,
            "abc.map is not a function",
        ),
        (
            "[x for x in 5]",
            "(5).map(x => x)",
            ErrorKind::Type,
            "number.map is not a function",
        ),
        (
            "d = [x for x in missing]",
            "d = missing.map(x => x)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
    ] {
        assert_eq!(
            failure(comprehension),
            (kind, message.to_string()),
            "{comprehension}"
        );
        assert_eq!(failure(spelled), failure(comprehension), "{spelled}");
    }
}

#[test]
fn comprehension_and_arrow_runtimes_keep_identical_results_graphs_and_explanations() {
    for (comprehension, spelled) in [
        (
            "xs = [1, 2, 3]\nlimit = 1\nd = [x * 2 for x in xs if x > limit]\nxs.push(4)\nlimit = 2\n[d, (why d).map(s -> s.rule), (affects limit).map(s -> s.node)]",
            "xs = [1, 2, 3]\nlimit = 1\nd = xs.filter(x => x > limit).map(x => x * 2)\nxs.push(4)\nlimit = 2\n[d, (why d).map(s -> s.rule), (affects limit).map(s -> s.node)]",
        ),
        (
            "class O(t):\n    this.t = t\nlimit = 1\nd = [o.t for o of O if o.t > limit]\nO(1)\nk = O(2)\nO(3)\nk.t = 0\nlimit = 0\n[d, (why d).map(s -> s.rule)]",
            "class O(t):\n    this.t = t\nlimit = 1\nd = O.filter(o => o.t > limit).map(o => o.t)\nO(1)\nk = O(2)\nO(3)\nk.t = 0\nlimit = 0\n[d, (why d).map(s -> s.rule)]",
        ),
        (
            "grid = [[1, 2], [3]]\nd = [[c * 10 for c in row] for row in grid]\ngrid.push([4])\nd",
            "grid = [[1, 2], [3]]\nd = grid.map(row => row.map(c => c * 10))\ngrid.push([4])\nd",
        ),
        (
            "xs = [1, 2]\nd = [x + 1 for x in xs]\ntry:\n    xs.push(9)\n    throw \"UNDO\"\ncatch error:\n    reason = error\n[d, reason]",
            "xs = [1, 2]\nd = xs.map(x => x + 1)\ntry:\n    xs.push(9)\n    throw \"UNDO\"\ncatch error:\n    reason = error\n[d, reason]",
        ),
    ] {
        assert_eq!(outcome(comprehension), outcome(spelled), "{comprehension}");
    }
}

#[test]
fn nested_comprehensions_obey_the_nesting_limit_of_list_literals() {
    let lists = |depth: usize| format!("{}1{}", "[".repeat(depth), "]".repeat(depth));
    let mut runtime = Runtime::new();
    let value = runtime.run(&nested(62)).unwrap();
    assert_eq!(
        runtime.serialize_json(&value).unwrap(),
        format!("{}1.0{}", "[".repeat(62), "]".repeat(62))
    );
    assert!(Runtime::check(&lists(62)).is_ok());
    for depth in [63, 64, 4096] {
        let error = Runtime::check(&nested(depth)).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{depth}");
        assert_eq!(
            error.message(),
            "Expressions are nested too deeply",
            "{depth}"
        );
        let literal = Runtime::check(&lists(depth)).unwrap_err();
        assert_eq!(literal.message(), error.message(), "{depth}");
    }
    let assigned = |source: &str| {
        Runtime::check(&format!(
            "nested = {}",
            source.trim_start_matches("xs = [1]\n")
        ))
        .map_err(|error| error.message().to_string())
    };
    for depth in [61, 62] {
        assert_eq!(assigned(&nested(depth)), assigned(&lists(depth)), "{depth}");
    }
    assert!(assigned(&nested(61)).is_ok());
}

#[test]
fn cli_runs_comprehensions_in_files() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("list-comprehensions.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.lines().next(),
        Some("[[24.0,60.0],[\"#12\",\"#30\",\"#8\"]]")
    );
    assert!(
        stdout.contains("picked [variable]\n    reads    prices, limit"),
        "{stdout}"
    );
    assert!(
        stdout.contains("labels [variable]\n    reads    prices"),
        "{stdout}"
    );
}

#[test]
fn repl_runs_one_line_comprehensions_as_it_runs_list_literals() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            b".json on\nxs = [1, 2, 3]\nys = [x * 2 for x in xs if x > 1]\nxs.push(4)\nys\nif [x for x in xs if x > 3].length > 0:\n    big = 1\n\nbig\n.exit\n",
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let results: Vec<&str> = stdout.lines().skip(4).collect();
    assert_eq!(
        results,
        [
            "[1.0,2.0,3.0]",
            "[4.0,6.0]",
            "4.0",
            "[4.0,6.0,8.0]",
            "1.0",
            "1.0"
        ],
        "{stdout}"
    );
}

#[test]
fn only_the_appended_cases_use_comprehensions_and_run_every_assertion() {
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
        assert_eq!(uses_comprehension(&case.source), new, "{}", case.title);
        if new {
            let count =
                common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
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
    assert_eq!(assertions, 46);
    assert_eq!(cases, 1044);
    assert!(uses_comprehension("y = f([x for x in xs])"));
    assert!(!uses_comprehension("y = f(x for x in xs)"));
    assert!(!uses_comprehension("for x in xs:\n    pass"));
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### List comprehensions\n\n```\n")
        .expect("the language reference documents list comprehensions");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert!(reference.contains("[NUC 92](nuc-0092.md)"));
    assert_eq!(case.expected.as_deref(), Some("[24,60,16]"));
}
