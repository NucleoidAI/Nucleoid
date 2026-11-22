mod common;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::nuc::Nuc;
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "Ruby collection aliases".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count);
    value
}

#[test]
fn appended_cases_execute_every_assertion_and_expected_return() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let reference = common::cases(include_str!("reference.md"));
    for (cases, title) in [
        (
            &spec,
            "Nucleoid collects and selects lists with Ruby method names",
        ),
        (
            &spec,
            "Nucleoid collects and selects live class populations",
        ),
        (
            &spec,
            "Nucleoid preserves collection alias errors and rollback",
        ),
        (
            &reference,
            "Nucleoid collects and selects live collections with Ruby spellings",
        ),
    ] {
        let matching: Vec<_> = cases.iter().filter(|case| case.title == title).collect();
        assert_eq!(matching.len(), 1, "{title}");
        let case = matching.first().unwrap();
        assert!(case.expected.is_some(), "{title}");
        for source in [
            case.source.clone(),
            case.source
                .replace(".collect(", ".map(")
                .replace(".select(", ".filter("),
        ] {
            checked(&mut Runtime::new(), &source, case.expected.as_deref());
        }
    }
}

#[test]
fn aliases_share_eager_receiver_argument_and_callback_snapshots() {
    for (alias, canonical, expected) in [
        ("collect", "map", "[781,7812]"),
        ("select", "filter", "[1,2]"),
    ] {
        for method in [alias, canonical] {
            let mut runtime = Runtime::new();
            let source = format!(
                r#"
values = [1, 2]
log = Object()
log.order = 0
callback = item -> (log.order = log.order.value * 10 + item)
result = values.{method}(callback, log.order = 7, log.order = log.order.value * 10 + 8, values.push(3), callback = item -> 0)
assert(log.order, 7812)
assert(values, [1, 2, 3])
assert(callback(1), 0)
class Entry
first = Entry()
population = Entry.{method}(item -> item, Entry())
assert(population, [first])
assert(Entry.length, 2)
result
"#
            );
            checked(&mut runtime, &source, Some(expected));
        }
    }
}

#[test]
fn callbacks_receive_only_the_item_and_visit_the_original_snapshot() {
    checked(
        &mut Runtime::new(),
        "[1, 2].collect((item, index, receiver) -> [item, index, receiver])",
        Some("[[1,null,null],[2,null,null]]"),
    );
    checked(
        &mut Runtime::new(),
        "[1, 2].select((item, index, receiver) -> index == null && receiver == null)",
        Some("[1,2]"),
    );
    for (method, expected) in [("collect", "[3,4]"), ("select", "[1,2]")] {
        let mut runtime = Runtime::new();
        checked(&mut runtime, "values = [1, 2]", None);
        checked(
            &mut runtime,
            &format!("values.{method}(item -> values.push(9))"),
            Some(expected),
        );
        assert_eq!(
            runtime.state.variable("values"),
            Some(&Value::List(vec![
                Value::number(1),
                Value::number(2),
                Value::number(9),
                Value::number(9),
            ]))
        );
    }
}

#[test]
fn callback_errors_keep_canonical_labels_precedence_and_rollback() {
    for (method, label) in [("collect", "map"), ("select", "filter")] {
        let mut runtime = Runtime::new();
        checked(&mut runtime, "class Empty", None);
        for receiver in ["[]", "Empty"] {
            for argument in ["", "0"] {
                let error = runtime
                    .run(&format!("{receiver}.{method}({argument})"))
                    .unwrap_err();
                assert_eq!(error.kind(), ErrorKind::Type);
                assert_eq!(error.message(), format!("{label} expects a function"));
                assert_eq!(error.position(), Some(Position::new(1, 1)));
            }
        }
        let error = runtime
            .run(&format!("Empty.{method}(0, Math.random())"))
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), "Math.random is not supported");
        let source = format!(
            r#"
log = []
def abort(item):
    log.push(item)
    throw "ALIAS_ABORT"
try:
    [1, 2].{method}(abort)
catch error:
    assert(error, "ALIAS_ABORT")
assert(log, [])
"#
        );
        checked(&mut runtime, &source, None);
    }
}

#[test]
fn ordinary_names_properties_methods_and_other_receivers_keep_dispatch() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
def collect(value) => value + 1
def select(value) => value * 2
fields = {collect: "own", select: 7}
callable = {collect: collect, select: select}
assert([fields.collect, fields.select], ["own", 7])
assert([collect(3), select(3)], [4, 6])
assert([callable.collect(3), callable.select(3)], [4, 6])
class Tool(number):
    this.number = number
    def collect() => this.number + 1
    def select() => this.number * 2
tool = Tool(4)
assert([tool.collect(), tool.select()], [5, 8])
class Holder(items):
    this.items = items
$Holder.collected = .items.collect(.number)
$Holder.selected = .items.select(.number)
holder = Holder([{number: 2}, {number: 0}])
assert([holder.collected, holder.selected], [[2, 0], [holder.items[0]]])
log = Object()
log.count = 0
none = null
none.collect(log.count = 1)
none.select(log.count = 2)
callable.missing.collect(log.count = 3)
callable.missing.select(log.count = 4)
assert(log.count, 0)
"#,
        None,
    );
    for (source, control) in [("[1].collect", "[1].map"), ("Tool.select", "Tool.filter")] {
        assert_eq!(runtime.run(source).unwrap(), Value::Undefined);
        assert_eq!(runtime.run(control).unwrap(), Value::Undefined);
    }
    for method in ["collect", "select"] {
        for (receiver, label) in [(r#""abc""#, "abc"), ("(1)", "number")] {
            let error = runtime
                .run(&format!("{receiver}.{method}(item -> item)"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Type);
            assert_eq!(
                error.message(),
                format!("{label}.{method} is not a function")
            );
        }
    }
}

#[test]
fn written_calls_and_graph_nodes_preserve_names_and_dependencies() {
    assert_eq!(
        tokenize("collect select")
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        vec![
            Token::Identifier("collect".to_string()),
            Token::Identifier("select".to_string()),
            Token::Eof,
        ]
    );
    let source = "collected = values.collect(project)\nselected = values.select(keep)";
    assert_eq!(
        generate_all(&parse(source).unwrap()),
        "collected=values.collect(project);selected=values.select(keep)"
    );
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
values = [1, 2]
factor = 2
def project(value) => value * factor
def keep(value) => value > 1
collected = values.collect(project)
mapped = values.map(project)
selected = values.select(keep)
filtered = values.filter(keep)
assert(collected, mapped)
assert(selected, filtered)
"#,
        None,
    );
    for (alias, control, written) in [
        ("collected", "mapped", "values.collect(project)"),
        ("selected", "filtered", "values.select(keep)"),
    ] {
        let node = runtime.graph.retrieve(&NodeKey::variable(alias)).unwrap();
        let canonical = runtime.graph.retrieve(&NodeKey::variable(control)).unwrap();
        assert_eq!(node.dependencies, canonical.dependencies);
        let Some(Nuc::Variable(variable)) = &node.node else {
            panic!("{alias} is a variable declaration");
        };
        assert_eq!(variable.value.to_string(), written);
    }
}
