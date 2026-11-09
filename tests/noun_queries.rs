//! NUC 75 revision 2: `NOUN of OPERAND` reads the list queries named by
//! nouns, `sum`, `max`, `min` and `mode`, as exactly `OPERAND.NOUN()`.

mod common;
#[path = "support/runtime_equivalence.rs"]
mod runtime_equivalence;

use std::sync::Arc;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Expr, Noun, Stmt, TemplatePart, parse, parse_expression};
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

const NOUNS: [&str; 4] = ["sum", "max", "min", "mode"];

const SPEC_TITLES: [&str; 9] = [
    "Nucleoid reads noun list queries with of",
    "Nucleoid reads noun queries of class instance populations with of",
    "Nucleoid keeps noun queries live in class level rules",
    "Nucleoid binds noun queries as prefix operators",
    "Nucleoid keeps names and loop variables named like noun queries",
    "Nucleoid reads noun queries in headers and nested expressions",
    "Nucleoid explains noun queries in their written form",
    "Nucleoid rolls back failed noun query updates",
    "Nucleoid reads noun queries of snapshots versions and other receivers",
];
const REFERENCE_TITLE: &str = "Nucleoid reads list and population queries named by nouns with of";
const README_TITLE: &str = "Noun queries read like the model they describe";
const EXAMPLES_TITLE: &str = "Noun queries keep the fill color and width of every shape live";

type Outcome = Result<Value, (ErrorKind, String, Option<Position>)>;

/// A Compatibility row's documented result: a value, or an error's kind,
/// message, line and column.
type Documented = Result<Value, (ErrorKind, &'static str, usize, usize)>;

fn outcome(runtime: &mut Runtime, source: &str) -> Outcome {
    runtime
        .run(source)
        .map_err(|error| (error.kind(), error.message(), error.position()))
}

fn run(source: &str) -> Runtime {
    let mut runtime = Runtime::new();
    checked(&mut runtime, source);
    runtime
}

/// Runs `source` and requires every assertion in it to run and hold.
fn checked(runtime: &mut Runtime, source: &str) {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    common::check_with_count(
        runtime,
        &common::Case {
            title: "noun query regression".to_string(),
            source: source.to_string(),
            expected: None,
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count, "{source}");
}

fn value(runtime: &mut Runtime, source: &str) -> Value {
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn failure(runtime: &mut Runtime, source: &str) -> (ErrorKind, String, Option<Position>) {
    match outcome(runtime, source) {
        Ok(value) => panic!("{source} returned {value:?}"),
        Err(error) => error,
    }
}

/// The same value, not merely an equal one: objects by identity, functions
/// and regular expressions by pointer, numbers by their bits with every NaN
/// alike.
fn identical(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Undefined, Value::Undefined) | (Value::Null, Value::Null) => true,
        (Value::Number(left), Value::Number(right)) => {
            left.to_bits() == right.to_bits() || (left.is_nan() && right.is_nan())
        }
        (Value::Bool(left), Value::Bool(right)) => left == right,
        (Value::String(left), Value::String(right)) => left == right,
        (Value::Date(left), Value::Date(right)) => left == right,
        (Value::Class(left), Value::Class(right)) => left == right,
        (Value::Object(left), Value::Object(right)) => left == right,
        (Value::Function(left), Value::Function(right)) => Arc::ptr_eq(left, right),
        (Value::Regex(left), Value::Regex(right)) => Arc::ptr_eq(left, right),
        (Value::List(left), Value::List(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| identical(left, right))
        }
        _ => false,
    }
}

fn same_outcome(left: &Outcome, right: &Outcome) -> bool {
    match (left, right) {
        (Ok(left), Ok(right)) => identical(left, right),
        (Err(left), Err(right)) => left == right,
        _ => false,
    }
}

fn dependencies(runtime: &Runtime, key: &NodeKey) -> Vec<String> {
    runtime
        .graph
        .retrieve(key)
        .unwrap_or_else(|| panic!("{key} is filed"))
        .dependencies
        .iter()
        .map(ToString::to_string)
        .collect()
}

fn object(runtime: &mut Runtime, name: &str) -> ObjectId {
    match value(runtime, name) {
        Value::Object(id) => id,
        other => panic!("{name} is {other:?}"),
    }
}

/// Whether a source writes `NOUN of` outside a loop name, read from tokens so
/// that template text and strings are left alone.
fn uses_noun_query(source: &str) -> bool {
    let tokens: Vec<Token> = tokenize(source.trim())
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    tokens.windows(2).enumerate().any(|(index, pair)| {
        matches!(
            pair,
            [Token::Identifier(name), Token::Keyword(Keyword::Of)]
                if NOUNS.contains(&name.as_str())
        ) && !loop_name(&tokens, index)
    })
}

/// `for NAME of` and `for KEY, NAME of` name loop variables.
fn loop_name(tokens: &[Token], index: usize) -> bool {
    let before = |offset: usize| index.checked_sub(offset).map(|at| &tokens[at]);
    matches!(before(1), Some(Token::Keyword(Keyword::For)))
        || matches!(
            (before(3), before(2), before(1)),
            (
                Some(Token::Keyword(Keyword::For)),
                Some(Token::Identifier(_)),
                Some(Token::Comma)
            )
        )
}

#[test]
fn appended_document_cases_run_every_assertion() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    let readme = common::snippets(include_str!("../README.md"), "README.md");
    let examples = common::snippets(include_str!("../docs/examples.md"), "docs/examples.md");
    let appended = SPEC_TITLES.iter().map(|title| (&spec, *title)).chain([
        (&reference, REFERENCE_TITLE),
        (&readme, README_TITLE),
        (&examples, EXAMPLES_TITLE),
    ]);

    let mut assertions = 0;
    for (cases, title) in appended {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title} is appended exactly once");
        let case = matching[0];
        assert!(uses_noun_query(&case.source), "{title} uses NOUN of");
        let count = common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
        assert!(count > 0, "{title} asserts its behavior");
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{title}: {reason}"));
        assertions += count;
    }
    assert_eq!(assertions, 87);
}

#[test]
fn no_earlier_document_case_writes_a_noun_query() {
    let documents = [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ];
    let appended = [REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE];
    let mut cases = 0;
    let mut with_query = 0;
    for case in documents.iter().flatten() {
        cases += 1;
        let uses = uses_noun_query(&case.source);
        let new =
            SPEC_TITLES.contains(&case.title.as_str()) || appended.contains(&case.title.as_str());
        assert_eq!(uses, new, "{}", case.title);
        with_query += usize::from(uses);
    }
    assert_eq!(with_query, 12);
    assert_eq!(cases, 1044);
}

#[test]
fn noun_queries_parse_to_their_own_node() {
    for (noun, expected) in [
        ("sum", Noun::Sum),
        ("max", Noun::Max),
        ("min", Noun::Min),
        ("mode", Noun::Mode),
    ] {
        assert_eq!(Noun::from_name(noun), Some(expected));
        assert_eq!(expected.as_str(), noun);
        assert_eq!(
            parse_expression(&format!("{noun} of list")).unwrap(),
            Expr::Of {
                noun: expected,
                operand: Box::new(Expr::Identifier("list".to_string())),
            }
        );
    }
    for name in [
        "count", "distinct", "length", "sort", "push", "list", "all", "Sum", "sums", "of",
    ] {
        assert_eq!(Noun::from_name(name), None, "{name}");
    }
}

#[test]
fn noun_queries_are_prefix_operators_over_unary_operands() {
    for (written, grouped) in [
        ("sum of xs + 1", "(sum of xs) + 1"),
        ("1 + sum of xs * 2", "1 + ((sum of xs) * 2)"),
        ("sum of xs > max of ys", "(sum of xs) > (max of ys)"),
        ("sum of xs == 3", "(sum of xs) == 3"),
        ("sum of xs and ready", "(sum of xs) and ready"),
        ("sum of xs ?? 0", "(sum of xs) ?? 0"),
        ("sum of xs |> why", "(sum of xs) |> why"),
        ("why sum of xs", "(sum of xs) |> why"),
        ("sum of xs if ready else 0", "ready ? (sum of xs) : 0"),
        (
            "ready ? max of xs : min of xs",
            "ready ? (max of xs) : (min of xs)",
        ),
        ("sum of xs.map(f)", "sum of (xs.map(f))"),
        ("sum of xs.map(f).slice(1)", "sum of (xs.map(f).slice(1))"),
        ("sum of xs[0:2]", "sum of (xs[0:2])"),
        ("sum of xs@first", "sum of (xs@first)"),
        ("sum of xs.value", "sum of (xs.value)"),
        ("mode of $Obj.parts", "mode of ($Obj.parts)"),
        ("sum of xs ** 2", "sum of (xs ** 2)"),
        ("2 ** sum of xs", "2 ** (sum of xs)"),
        ("-sum of xs", "-(sum of xs)"),
        ("not sum of xs", "!(sum of xs)"),
        ("typeof mode of xs", "typeof (mode of xs)"),
        ("sum of -xs", "sum of (-xs)"),
        ("sum of max of xs", "sum of (max of xs)"),
        ("x -> sum of x", "x -> (sum of x)"),
        ("lambda x: max of x", "(x) => (max of x)"),
        ("[max of xs, min of xs]", "[(max of xs), (min of xs)]"),
        ("f(sum of xs, 1)", "f((sum of xs), 1)"),
        ("total = sum of xs", "total = (sum of xs)"),
    ] {
        assert_eq!(
            parse_expression(written).unwrap(),
            parse_expression(grouped).unwrap(),
            "{written}"
        );
    }
}

#[test]
fn noun_queries_render_as_written_and_read_back() {
    for (source, rendered) in [
        ("sum of xs", "sum of xs"),
        (
            "mode of Order.map(order -> order.size)",
            "mode of Order.map(function())",
        ),
        ("sum of xs + 1", "sum of xs+1"),
        ("1 + sum of xs", "1+sum of xs"),
        ("sum of (a + b)", "sum of (a+b)"),
        ("sum of (a && b)", "sum of (a&&b)"),
        ("sum of (a ?? b)", "sum of (a??b)"),
        ("sum of (c ? a : b)", "sum of (c?a:b)"),
        ("sum of (x = xs)", "sum of (x=xs)"),
        ("sum of (xs |> why)", "sum of (xs|>why)"),
        ("sum of a ** b", "sum of a**b"),
        ("(sum of xs) ** 2", "(sum of xs)**2"),
        ("2 ** sum of xs", "2**sum of xs"),
        ("-sum of xs", "-sum of xs"),
        ("typeof mode of xs", "typeof mode of xs"),
        ("sum of -xs", "sum of -xs"),
        ("sum of max of xs", "sum of max of xs"),
        ("sum of xs@first", "sum of xs@first"),
        ("sum of xs.value", "sum of xs.value"),
        ("(mode of words).length", "(mode of words).length"),
        ("(max of xs)(1)", "(max of xs)(1)"),
        ("(mode of rows)[0]", "(mode of rows)[0]"),
        ("(mode of rows)[1:]", "(mode of rows)[1:]"),
        ("(mode of rows)@first", "(mode of rows)@first"),
        ("`${(sum of xs)}`", "`${(sum of xs)}`"),
        ("`a ${(max of xs)} b`", "`a ${(max of xs)} b`"),
        ("ready ? sum of xs : 0", "(ready?sum of xs:0)"),
        ("(all v in xs: sum of v > 1)", "(all v in xs:sum of v>1)"),
        ("{t: sum of xs}", "{\"t\":sum of xs}"),
        ("[max of xs, min of xs]", "[max of xs,min of xs]"),
        ("f(sum of xs)", "f(sum of xs)"),
    ] {
        let expression = parse_expression(source).unwrap();
        assert_eq!(expression.to_string(), rendered, "{source}");
        if !rendered.contains("function()") {
            assert_eq!(parse_expression(rendered).unwrap(), expression, "{source}");
        }
    }
    // Other operators add no parentheses, as `(-x).y` renders `-x.y`.
    for (source, rendered) in [
        ("(-sum of xs).y", "-sum of xs.y"),
        ("`${1 + (sum of xs)}`", "`${1+sum of xs}`"),
    ] {
        let expression = parse_expression(source).unwrap();
        assert_eq!(expression.to_string(), rendered, "{source}");
        assert_ne!(parse_expression(rendered).unwrap(), expression, "{source}");
    }
}

#[test]
fn accepted_renderings_and_graph_keys_do_not_change() {
    for (source, rendered) in [
        ("xs.sum()", "xs.sum()"),
        ("Order.mode(order -> order.size)", "Order.mode(function())"),
        ("(a + b).c", "a+b.c"),
        ("(-x).y", "-x.y"),
        ("(x ** 2).y", "x**2.y"),
        ("f(x)(y)", "f(x)(y)"),
        ("xs[0][1:]", "xs[0][1:]"),
        ("xs@first.sum()", "xs@first.sum()"),
        ("`${a} and ${b + 1}`", "`${a} and ${b+1}`"),
        ("(-x) ** 2", "(-x)**2"),
        ("sum + max", "sum+max"),
        ("mode(xs)", "mode(xs)"),
    ] {
        assert_eq!(
            parse_expression(source).unwrap().to_string(),
            rendered,
            "{source}"
        );
    }

    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        "xs = [1, 2, 3]\nflag = 0\nif xs.sum() > 3:\n    flag = 1\nif sum of xs > 3:\n    flag = 2\nassert(flag, 2)",
    );
    let keys: Vec<String> = runtime.graph.keys().map(ToString::to_string).collect();
    assert!(keys.contains(&"if(xs.sum()>3)".to_string()), "{keys:?}");
    assert!(keys.contains(&"if(sum of xs>3)".to_string()), "{keys:?}");
}

#[test]
fn why_explains_noun_queries_as_written() {
    let mut runtime = run(
        "prices = [4, 9, 9]\ntop = max of prices\nwords = [\"ab\", \"c\", \"ab\"]\nwidth = (mode of words).length\nclass Order(size): this.size = size\nOrder(3)\nlargest = max of Order.map(order -> order.size)\nlabel = `top ${(top)} of ${(sum of prices)}`",
    );
    for (query, rule) in [
        ("(why top)[0].rule", "top = max of prices"),
        ("(why width)[0].rule", "width = (mode of words).length"),
        (
            "(why largest)[0].rule",
            "largest = max of Order.map(function())",
        ),
        (
            "(why label)[0].rule",
            "label = `top ${top} of ${(sum of prices)}`",
        ),
    ] {
        assert_eq!(value(&mut runtime, query), Value::String(rule.to_string()));
    }
    assert_eq!(
        value(&mut runtime, "(why top)[0].from"),
        Value::List(vec![Value::String("prices".to_string())])
    );
    assert_eq!(
        failure(&mut runtime, "why sum of prices"),
        (
            ErrorKind::Type,
            "Cannot explain sum of prices".to_string(),
            Some(Position::new(1, 1))
        )
    );
}

/// Every receiver kind, in one runtime per receiver, for every noun: the
/// query and its method call give the same value, by identity, or the same
/// error with the same position.
#[test]
fn noun_queries_equal_their_method_calls_for_every_receiver() {
    let setup = r#"
xs = [2, "2", true, 2, null]
history = [1]
history.push(4)
holder = {items: [3, 3, 1]}
nested = [[1], [1], [2]]
records = [{a: 1}, {a: 1}, {a: 2}]
readings = [0 / 0, 1, 0 / 0]
gaps = [1, holder.missing, 3]
dates = [Date(5), 5]
class Order(region, size):
    this.region = region
    this.size = size
Order("n", 3)
Order("s", 5)
Order("n", 4)
kind = Order
class Empty
class Shelf:
    def sum(): return 1
    def max(): return 2
    def min(): return 3
    def mode(): return 4
shelf = Shelf()
calls = {sum: () -> 5, max: () -> 6, min: () -> 7, mode: () -> 8}
plain = {}
nothing = null
"#;
    let operands = [
        "[3, 1, 3]",
        "[]",
        "xs",
        "xs.value",
        "history",
        "history@first",
        "history@previous",
        "holder.items",
        "holder.items.filter(item -> item > 1)",
        "holder.items.map(item -> item * 2)",
        "holder.items.slice(1)",
        "holder.items[1:]",
        "holder.items.concat([9])",
        "nested",
        "records",
        "readings",
        "gaps",
        "dates",
        "[\"10\", \"9\"]",
        "Order",
        "kind",
        "Order.list()",
        "Order[1:]",
        "Order[0:0]",
        "Order.map(order -> order.size)",
        "Order.map(order -> order.region)",
        "Order.filter(order -> order.size > 3)",
        "Empty",
        "Shelf",
        "shelf",
        "calls",
        "plain",
        "nothing",
        "holder.missing",
        "\"abc\"",
        "5",
        "true",
        "Date(5)",
        "(item -> item)",
        "/ab/",
        "Math",
        "Number",
        "String",
        "Date",
        "undeclared",
    ];
    let mut compared = 0;
    for noun in NOUNS {
        for operand in operands {
            let mut runtime = Runtime::new();
            runtime.run(setup).unwrap();
            let query = outcome(&mut runtime, &format!("{noun} of {operand}"));
            let method = outcome(&mut runtime, &format!("{operand}.{noun}()"));
            assert!(
                same_outcome(&query, &method),
                "{noun} of {operand}: {query:?} against {method:?}"
            );
            compared += 1;
        }
    }
    assert_eq!(compared, 4 * operands.len());
}

#[test]
fn the_noun_names_the_query_never_a_binding() {
    let mut runtime = run(r#"
def sum(values):
    return 0
max = 9
min = x -> x
assert(sum([1, 2]), 0)
assert(sum of [1, 2], 3)
assert(max, 9)
assert(max of [1, 2], 2)
assert(min of [4, 3], 3)
assert(min(5), 5)
def pick(mode) => mode of mode
assert(pick([1, 2, 2]), 2)
lowest = lambda min: min of min
assert(lowest([4, 1]), 1)
o = {mode: 4}
assert(o.mode, 4)
assert(mode of [o.mode, o.mode, 1], 4)
"#);
    assert_eq!(
        failure(&mut runtime, "mode([1, 2])"),
        (
            ErrorKind::Reference,
            "mode is not defined".to_string(),
            Some(Position::new(1, 1))
        )
    );
    assert_eq!(value(&mut runtime, "Math.max(1, 5)"), Value::Number(5.0));
    assert_eq!(
        value(&mut runtime, "max of Math"),
        Value::Number(f64::NEG_INFINITY)
    );
}

#[test]
fn declarations_depend_on_what_the_method_call_depends_on() {
    let mut runtime = run(r#"
xs = [4, 9, 2, 9]
holder = {items: [1, 2]}
class Order(size): this.size = size
first = Order(3)
Order(5)
limit = 1
"#);
    let operands = [
        "xs",
        "holder.items",
        "xs.filter(x -> x > limit)",
        "Order.map(order -> order.size)",
        "Order",
        "holder.items.value",
    ];
    for (index, operand) in operands.iter().enumerate() {
        for noun in NOUNS {
            value(
                &mut runtime,
                &format!(
                    "q_{noun}_{index} = {noun} of {operand}\nm_{noun}_{index} = {operand}.{noun}()"
                ),
            );
            assert_eq!(
                dependencies(&runtime, &NodeKey::variable(format!("q_{noun}_{index}"))),
                dependencies(&runtime, &NodeKey::variable(format!("m_{noun}_{index}"))),
                "{noun} of {operand}"
            );
        }
    }
    let changes = [
        "xs.push(9)",
        "xs.shift()",
        "xs = [7, 7, 1]",
        "holder.items.push(8)",
        "holder.items = [5]",
        "limit = 6",
        "first.size = 11",
        "Order(1)",
        "delete first.size\ndelete first",
    ];
    for change in changes {
        value(&mut runtime, change);
        for (index, operand) in operands.iter().enumerate() {
            for noun in NOUNS {
                let query = value(&mut runtime, &format!("q_{noun}_{index}"));
                let method = value(&mut runtime, &format!("m_{noun}_{index}"));
                assert!(
                    identical(&query, &method),
                    "after {change}: {noun} of {operand}: {query:?} against {method:?}"
                );
            }
        }
    }
}

#[test]
fn class_level_rules_match_the_method_form_for_each_instance() {
    let mut runtime = run(r#"
class Obj(parts): this.parts = parts
$Obj.fill_color = mode of $Obj.parts
$Obj.fill_method = $Obj.parts.mode()
$Obj.brightest = max of $Obj.parts
$Obj.brightest_method = $Obj.parts.max()
first = Obj([3, 1, 3])
second = Obj([2, 2, 5])
class Team(name): this.name = name
class Job(team, hours):
    this.team = team
    this.hours = hours
$Team.load = sum of Job.filter(job -> job.team == $Team).map(job -> job.hours)
$Team.load_method = Job.filter(job -> job.team == $Team).map(job -> job.hours).sum()
red = Team("red")
blue = Team("blue")
Job(red, 3)
long = Job(blue, 8)
"#);
    let pairs = [
        ("first", "fill_color", "fill_method"),
        ("second", "fill_color", "fill_method"),
        ("first", "brightest", "brightest_method"),
        ("second", "brightest", "brightest_method"),
        ("red", "load", "load_method"),
        ("blue", "load", "load_method"),
    ];
    for (instance, query, method) in pairs {
        let id = object(&mut runtime, instance);
        let mut query_dependencies = dependencies(&runtime, &NodeKey::property(&id, query));
        let mut method_dependencies = dependencies(&runtime, &NodeKey::property(&id, method));
        query_dependencies.sort();
        method_dependencies.sort();
        assert_eq!(
            query_dependencies, method_dependencies,
            "{instance}.{query}"
        );
    }
    for change in [
        "first.parts.push(1)\nfirst.parts.push(1)",
        "second.parts = [7]",
        "Job(red, 2)",
        "long.team = red",
        "long.hours = 1",
        "third = Obj([4, 4])",
    ] {
        value(&mut runtime, change);
        for (instance, query, method) in pairs {
            let query_value = value(&mut runtime, &format!("{instance}.{query}"));
            let method_value = value(&mut runtime, &format!("{instance}.{method}"));
            assert!(
                identical(&query_value, &method_value),
                "after {change}: {instance}.{query} is {query_value:?}, not {method_value:?}"
            );
        }
    }
    checked(
        &mut runtime,
        "assert([first.fill_color, second.fill_color, third.fill_color], [1, 7, 4])\nassert([red.load, blue.load], [6, 0])",
    );
}

/// A class rule reads `NOUN of $Class` as the property `NOUN`, and
/// `NOUN of $Class.parts` as `parts`, as the method form's callee reads them.
#[test]
fn class_rules_reading_noun_queries_close_the_same_cycles() {
    for noun in NOUNS {
        for (rule, method) in [
            (
                format!("$Obj.{noun} = {noun} of $Obj"),
                format!("$Obj.{noun} = $Obj.{noun}()"),
            ),
            (
                format!("$Obj.total = {noun} of $Obj\n$Obj.{noun} = $Obj.total"),
                format!("$Obj.total = $Obj.{noun}()\n$Obj.{noun} = $Obj.total"),
            ),
            (
                format!("$Obj.total = {noun} of $Obj.parts\n$Obj.parts = $Obj.total"),
                format!("$Obj.total = $Obj.parts.{noun}()\n$Obj.parts = $Obj.total"),
            ),
        ] {
            let mut query = Runtime::new();
            let mut call = Runtime::new();
            query.run("class Obj").unwrap();
            call.run("class Obj").unwrap();
            let query = outcome(&mut query, &rule);
            let call = outcome(&mut call, &method);
            assert_eq!(
                query.as_ref().unwrap_err().1,
                "Circular Dependency",
                "{rule}"
            );
            assert!(
                same_outcome(&query, &call),
                "{rule}: {query:?} against {call:?}"
            );
        }
    }
}

#[test]
fn the_receiver_is_evaluated_once_in_order_and_a_null_receiver_is_undefined() {
    let mut runtime = run(r#"
calls = []
def note(label, result):
    calls.push(label)
    return result
holder = {}
nothing = null
"#);
    assert_eq!(
        value(
            &mut runtime,
            "[note(\"a\", 1), sum of note(\"b\", [2, 3]), note(\"c\", 4)]"
        ),
        Value::List(vec![
            Value::Number(1.0),
            Value::Number(5.0),
            Value::Number(4.0)
        ])
    );
    for noun in NOUNS {
        let query = value(&mut runtime, &format!("{noun} of note(\"{noun}\", [1, 2])"));
        let method = value(&mut runtime, &format!("note(\"{noun}\", [1, 2]).{noun}()"));
        assert!(identical(&query, &method), "{noun}");
        for receiver in ["nothing", "holder.missing", "null"] {
            assert_eq!(
                value(&mut runtime, &format!("{noun} of {receiver}")),
                Value::Undefined,
                "{noun} of {receiver}"
            );
        }
    }
    checked(
        &mut runtime,
        "assert(calls, [\"a\", \"b\", \"c\", \"sum\", \"sum\", \"max\", \"max\", \"min\", \"min\", \"mode\", \"mode\"])",
    );
}

#[test]
fn selectors_stay_method_arguments_that_equal_noun_queries_over_map() {
    let mut runtime = run(r#"
holder = {}
class Item(size, tag):
    this.size = size
    this.tag = tag
Item(3, "a")
Item(5, "b")
Item(5, "b")
xs = [1, 2, 2, 3]
"#);
    let selectors = [
        "x -> x * 2",
        "x -> x % 2",
        "x -> -x",
        "x -> 0 / 0",
        "x -> holder.missing",
        "x -> null",
        "x -> \"7\"",
        "x -> [x]",
        "item -> item",
    ];
    for noun in NOUNS {
        for selector in selectors {
            for receiver in ["xs", "[]"] {
                let method = outcome(&mut runtime, &format!("{receiver}.{noun}({selector})"));
                let query = outcome(
                    &mut runtime,
                    &format!("{noun} of {receiver}.map({selector})"),
                );
                assert!(
                    same_outcome(&query, &method),
                    "{noun} of {receiver}.map({selector}): {query:?} against {method:?}"
                );
            }
        }
        for selector in ["item -> item.size", "item -> item.tag"] {
            let method = outcome(&mut runtime, &format!("Item.{noun}({selector})"));
            let query = outcome(&mut runtime, &format!("{noun} of Item.map({selector})"));
            assert!(same_outcome(&query, &method), "{noun} {selector}");
        }
        let method = failure(&mut runtime, &format!("xs.{noun}(5)"));
        let query = failure(&mut runtime, &format!("{noun} of xs.map(5)"));
        assert_eq!(method.1, format!("{noun} expects a function"));
        assert_eq!(query.1, "map expects a function");
    }
    value(
        &mut runtime,
        "first = Item.list()[0]\nwith_method = Item.max(item -> item.size)\nwith_query = max of Item.map(item -> item.size)",
    );
    assert_eq!(
        dependencies(&runtime, &NodeKey::variable("with_query")),
        dependencies(&runtime, &NodeKey::variable("with_method"))
    );
    value(&mut runtime, "first.size = 9");
    assert_eq!(value(&mut runtime, "with_query"), Value::Number(9.0));
    assert_eq!(value(&mut runtime, "with_method"), Value::Number(9.0));
}

#[test]
fn failed_noun_query_updates_roll_back() {
    let initial = r#"
def check(value):
    if value == "void":
        throw "BAD_ZONE"
    return value
zones = ["a", "b", "b"]
busiest = mode of zones.map(check)
class Parcel(zone, weight):
    this.zone = zone
    this.weight = weight
Parcel("b", 2)
Parcel("a", 3)
heaviest = max of Parcel.map(parcel -> check(parcel.zone) == "a" ? parcel.weight : 0)
assert(busiest, "b")
assert(heaviest, 3)
"#;
    let mut actual = run(initial);
    let control = run(initial);
    for (source, thrown) in [
        ("zones.push(\"void\")", "BAD_ZONE"),
        ("zones.shift()\nzones.push(\"void\")", "BAD_ZONE"),
        ("zones = [\"void\"]", "BAD_ZONE"),
        ("bad = Parcel(\"void\", 1)", "BAD_ZONE"),
        ("Parcel(\"a\", 7)\nthrow \"ROLLBACK\"", "ROLLBACK"),
    ] {
        let error = actual.run(source).unwrap_err();
        assert_eq!(
            error.thrown_value(),
            Some(&Value::String(thrown.to_string())),
            "{source}"
        );
        runtime_equivalence::assert_equivalent(&actual, None, &control, None);
    }
    checked(
        &mut actual,
        "zones.push(\"a\")\nzones.push(\"a\")\nassert(busiest, \"a\")\nParcel(\"a\", 8)\nassert(heaviest, 8)",
    );
}

#[test]
fn suppose_blocks_discard_noun_query_changes() {
    run(r#"
sizes = [1, 2, 2]
common = mode of sizes
total = sum of sizes
suppose sizes.push(2): assert([common, total], [2, 7])
suppose:
    sizes.push(1)
    sizes.push(1)
    assert([common, total], [1, 7])
moved = suppose sizes = [9]: [common, total]
assert(moved, [9, 9])
assert([common, total, sizes], [2, 5, [1, 2, 2]])
"#);
}

#[test]
fn for_headers_keep_loop_variables_named_like_noun_queries() {
    run(r#"
class Item(size): this.size = size
small = Item(1)
large = Item(2)
seen = []
for sum of Item:
    seen.push(sum.size)
for max in Item: seen.push(max.size * 10)
for min of "ab":
    seen.push(min)
record = {a: 1, b: 2}
for key, mode of record:
    seen.push(mode)
for sum, value in record:
    seen.push(sum)
assert(seen, [1, 2, 10, 20, "a", "b", 1, 2, "a", "b"])
groups = [[small, large], [small, large], [large]]
sizes = []
for item of mode of groups:
    sizes.push(item.size)
assert(sizes, [1, 2])
"#);
    let statements = parse("for mode of rows:\n    pass").unwrap();
    let [
        Stmt::For {
            variable, iterable, ..
        },
    ] = statements.as_slice()
    else {
        panic!("one loop");
    };
    assert_eq!(variable, "mode");
    assert_eq!(iterable, &Expr::Identifier("rows".to_string()));
    let statements = parse("for sum, max of rec:\n    pass").unwrap();
    let [Stmt::ForEntries { key, value, .. }] = statements.as_slice() else {
        panic!("one entry loop");
    };
    assert_eq!((key.as_str(), value.as_str()), ("sum", "max"));
}

#[test]
fn template_interpolations_keep_reading_a_leading_noun_as_a_name() {
    assert_eq!(
        parse_expression("`${sum of xs}`").unwrap(),
        Expr::Template(vec![TemplatePart::Expression(Expr::Identifier(
            "sum".to_string()
        ))])
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
sum = 5
mode = "m"
xs = [1, 2]
assert(`${sum of xs}`, "5")
assert(`${1 + sum of xs}`, "6")
assert(`${mode of xs} and ${mode}`, "m and m")
assert(`${(sum of xs)}`, "3")
assert(`${String(max of xs)}`, "2")
assert(`${xs[min of xs]}`, "2")
"#,
    );
    assert_eq!(
        failure(&mut runtime, "`${max of xs}`"),
        (
            ErrorKind::Reference,
            "max is not defined".to_string(),
            Some(Position::new(1, 1))
        )
    );
}

#[test]
fn malformed_noun_queries_report_where_the_operand_should_start() {
    for (source, message, line, column) in [
        ("x = sum of", "Unexpected end of input", 1, 11),
        ("x = sum of\ny = 1", "Unexpected end of line", 2, 1),
        ("x = (sum of)", "Unexpected ')'", 1, 13),
        ("x = [max of, 1]", "Unexpected ','", 1, 14),
        ("x = sum of of xs", "Unexpected keyword 'of'", 1, 15),
        ("x = mode of :", "Unexpected ':'", 1, 14),
        ("for x of mode of:\n    pass", "Unexpected ':'", 1, 17),
        ("`${(sum of)}`", "Unexpected ')'", 1, 12),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(
            error.position(),
            Some(Position::new(line, column)),
            "{source}"
        );
    }
    let nested = |count: usize| format!("x = {}xs", "max of ".repeat(count));
    assert!(Runtime::check(&nested(61)).is_ok());
    let error = Runtime::check(&nested(62)).unwrap_err();
    assert_eq!(error.message(), "Expressions are nested too deeply");
    let prefixed = |count: usize| format!("x = {}xs", "-".repeat(count));
    assert!(Runtime::check(&prefixed(61)).is_ok());
    assert!(Runtime::check(&prefixed(62)).is_err());
}

#[test]
fn other_names_before_of_keep_their_accepted_errors() {
    for (source, column) in [
        ("xs = [1]\ncount of xs", 10),
        ("xs = [1]\ndistinct of xs", 13),
        ("xs = [1]\nlength of xs", 11),
        ("xs = [1]\nsort of xs", 9),
        ("xs = [1]\npush of xs", 9),
        ("xs = [1]\nreverse of xs", 12),
        ("xs = [1]\naverage of xs", 12),
        ("xs = [1]\nSum of xs", 8),
        ("xs = [1]\nsums of xs", 9),
        ("class Order\nlist of Order", 9),
        ("class Obj\n$sum of xs", 9),
        ("xs = [1]\n\"sum\" of xs", 10),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), "Unexpected keyword 'of'", "{source}");
        assert_eq!(error.position(), Some(Position::new(2, column)), "{source}");
    }
    for (source, message, line, column) in [
        (
            "o = {sum: 1}\nxs = [1]\no.sum of xs",
            "Unexpected keyword 'of'",
            3,
            10,
        ),
        ("ok = all v of xs: v", "Unexpected keyword 'of'", 1, 15),
        ("for k, v of of: pass", "Unexpected keyword 'of'", 1, 15),
        (
            "for of xs: pass",
            "Expected a name but found keyword 'of'",
            1,
            8,
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(
            (error.message(), error.position()),
            (message.to_string(), Some(Position::new(line, column))),
            "{source}"
        );
    }
}

#[test]
fn errors_that_quote_a_query_quote_it_as_written() {
    let mut runtime = run("xs = [1, 2]");
    for (source, kind, message) in [
        ("why sum of xs", ErrorKind::Type, "Cannot explain sum of xs"),
        (
            "sum of xs |> why",
            ErrorKind::Type,
            "Cannot explain sum of xs",
        ),
        (
            "delete sum of xs",
            ErrorKind::Syntax,
            "Cannot delete sum of xs",
        ),
        (
            "sum of xs = 1",
            ErrorKind::Syntax,
            "Cannot assign to sum of xs",
        ),
        (
            "sum of xs += 1",
            ErrorKind::Syntax,
            "Invalid compound assignment target",
        ),
        (
            "(sum of xs)@first",
            ErrorKind::Type,
            "State versions require a variable or object property",
        ),
        (
            "all v in sum of xs: v",
            ErrorKind::Type,
            "sum of xs is not a collection",
        ),
        (
            "(max of xs)()",
            ErrorKind::Type,
            "max of xs is not a function",
        ),
        (
            "sum of xs := 1",
            ErrorKind::Syntax,
            "Invalid value assignment target",
        ),
    ] {
        let (actual_kind, actual_message, _) = failure(&mut runtime, source);
        assert_eq!(
            (actual_kind, actual_message.as_str()),
            (kind, message),
            "{source}"
        );
    }
}

/// Storing a declaration freezes its `.value` reads. The query is frozen as
/// its method form, whose callee holds the operand one level deeper, so a
/// declaration reaches the depth limit at the same nesting either way.
#[test]
fn declarations_reach_the_depth_limit_at_the_same_nesting_as_the_method_form() {
    let mut outcomes = Vec::new();
    for operands in 40..=100 {
        let receiver = format!("[xs{}{}]", ".a".repeat(100), " + 0".repeat(operands));
        let query = outcome(
            &mut Runtime::new(),
            &format!("xs = {{}}\nt = sum of {receiver}"),
        );
        let method = outcome(
            &mut Runtime::new(),
            &format!("xs = {{}}\nt = {receiver}.sum()"),
        );
        assert!(
            same_outcome(&query, &method),
            "{operands}: {query:?} against {method:?}"
        );
        outcomes.push(method.is_ok());
    }
    assert!(outcomes.contains(&true) && outcomes.contains(&false));
}

/// Each row of NUC 75's revision 2 Compatibility table, in its order: every
/// source was a syntax error at the accepted revision.
#[test]
fn compatibility_rows_give_the_documented_results() {
    let orders = "class Order(region, size):\n    this.region = region\n    this.size = size\nOrder(\"n\", 3)\nOrder(\"s\", 5)\nOrder(\"n\", 4)\n";
    let shelf = "class Shelf:\n    def mode(): return 9\nshelf = Shelf()\n";
    let number = |number: f64| Ok(Value::Number(number));
    let text = |text: &str| Ok(Value::String(text.to_string()));
    let rows: Vec<(String, Documented)> = vec![
        ("xs = [1, 2, 3]\nsum of xs".into(), number(6.0)),
        ("xs = [1, 2, 3]\nmax of xs".into(), number(3.0)),
        ("xs = [1, 2, 3]\nmin of xs".into(), number(1.0)),
        ("list = [3, 4, 4]\nnum = mode of list\nnum".into(), number(4.0)),
        (format!("{orders}max of Order.map(o -> o.size)"), number(5.0)),
        (format!("{orders}mode of Order.map(o -> o.region)"), text("n")),
        (
            format!("{orders}kind = Order\nsum of kind.map(o -> o.size)"),
            number(12.0),
        ),
        (
            "class Obj(parts): this.parts = parts\n$Obj.fill_color = mode of $Obj.parts\nobj = Obj([1, 2, 2])\nobj.parts.push(1)\nobj.parts.push(1)\nobj.fill_color".into(),
            number(1.0),
        ),
        ("xs = [1, 2, 3]\nsum of xs + 1".into(), number(7.0)),
        ("xs = [1, 2, 3]\n(sum of xs) ** 2".into(), number(36.0)),
        (
            "xs = [1, 2, 3]\nsum of xs ** 2".into(),
            Err((ErrorKind::Type, "number.sum is not a function", 2, 1)),
        ),
        (
            "xs = [1, 2, 3]\n[max of xs, min of xs]".into(),
            Ok(Value::List(vec![Value::Number(3.0), Value::Number(1.0)])),
        ),
        ("xs = [1, 2, 3]\nString(sum of xs)".into(), text("6")),
        ("xs = [1, 2, 3]\n`total ${(sum of xs)}`".into(), text("total 6")),
        (
            "xs = [1, 2, 3]\nflag = false\nif sum of xs > 5:\n    flag = true\nflag".into(),
            Ok(Value::Bool(true)),
        ),
        (
            "words = [\"ab\", \"c\", \"ab\"]\nwidth = (mode of words).length\nwidth".into(),
            number(2.0),
        ),
        ("xs = null\nsum of xs".into(), Ok(Value::Undefined)),
        ("mode of []".into(), Ok(Value::Undefined)),
        (
            "sum of \"abc\"".into(),
            Err((ErrorKind::Type, "abc.sum is not a function", 1, 1)),
        ),
        ("def sum(values): return 0\nsum of [1, 2]".into(), number(3.0)),
        ("o = {sum: () -> 5}\nsum of o".into(), number(5.0)),
        (format!("{shelf}mode of shelf"), number(9.0)),
        (
            "xs = [4, 9]\ntop = max of xs\n(why top)[0].rule".into(),
            text("top = max of xs"),
        ),
        (
            "xs = [1, 2, 3]\nwhy sum of xs".into(),
            Err((ErrorKind::Type, "Cannot explain sum of xs", 2, 1)),
        ),
        (
            "xs = [1]\nx = sum of".into(),
            Err((ErrorKind::Syntax, "Unexpected end of input", 2, 11)),
        ),
        (
            "xs = [1]\nlambda sum of xs".into(),
            Err((ErrorKind::Reference, "lambda is not defined", 2, 1)),
        ),
    ];
    assert_eq!(rows.len(), 26);
    for (source, expected) in rows {
        let actual = outcome(&mut Runtime::new(), &source);
        match expected {
            Ok(expected) => {
                let actual = actual.unwrap_or_else(|error| panic!("{source}: {error:?}"));
                assert!(identical(&actual, &expected), "{source}: {actual:?}");
            }
            Err((kind, message, line, column)) => assert_eq!(
                actual.unwrap_err(),
                (kind, message.to_string(), Some(Position::new(line, column))),
                "{source}"
            ),
        }
    }
}

#[test]
fn noun_queries_take_fewer_tokens_than_method_calls_without_a_selector() {
    let count = |source: &str| {
        tokenize(source)
            .unwrap()
            .iter()
            .filter(|spanned| spanned.token != Token::Eof)
            .count()
    };
    for (query, method, tokens) in [
        ("num = mode of list", "num = list.mode()", (5, 7)),
        (
            "$Obj.fill_color = mode of $Obj.parts",
            "$Obj.fill_color = $Obj.parts.mode()",
            (9, 11),
        ),
        (
            "top = max of Order.map(o -> o.size)",
            "top = Order.max(o -> o.size)",
            (14, 12),
        ),
    ] {
        assert_eq!((count(query), count(method)), tokens, "{query}");
    }
}
