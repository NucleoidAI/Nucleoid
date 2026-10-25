//! Optional named-source loops lower to the existing explicit-it For node.
//! Every ordinary regression is self-contained: no Git or session inputs.

mod common;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{Expr, Stmt, parse, parse_program};
use nucleoid::lang::evaluation::Flow;
use nucleoid::nuc::Nuc;
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value, statement};
use regex::Regex;

fn equivalent(implicit: &str, explicit: &str) -> Vec<Stmt> {
    let implicit_ast = parse(implicit).unwrap_or_else(|error| panic!("{implicit}\n{error}"));
    let explicit_ast = parse(explicit).unwrap_or_else(|error| panic!("{explicit}\n{error}"));
    assert_eq!(implicit_ast, explicit_ast, "{implicit}");
    assert_eq!(
        generate_all(&implicit_ast),
        generate_all(&explicit_ast),
        "{implicit}"
    );
    implicit_ast
}

/// Change only complete ordinary-identifier explicit-it indented headers.
fn implicit_loops(explicit: &str) -> String {
    Regex::new(r"(?m)^([ \t]*)for it of ([A-Za-z_][A-Za-z_0-9]*):$")
        .unwrap()
        .replace_all(explicit, "${1}for ${2}:")
        .into_owned()
}

fn explicit_loops(implicit: &str) -> String {
    Regex::new(r"(?m)^([ \t]*)for ([A-Za-z_][A-Za-z_0-9]*):$")
        .unwrap()
        .replace_all(implicit, "${1}for it of ${2}:")
        .into_owned()
}

fn check_forms(
    implicit: &str,
    explicit: &str,
    assertions: usize,
    expected: Option<&str>,
) -> [Runtime; 2] {
    assert_ne!(
        implicit, explicit,
        "a runtime pair must exercise distinct loop headers"
    );
    let statements = equivalent(implicit, explicit);
    assert_eq!(
        common::assertions::protected_assertions(&statements).len(),
        assertions
    );
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let mut returns = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([implicit, explicit]) {
        let value = common::check_with_count(
            runtime,
            &common::Case {
                title: "implicit iterator loop pair".to_string(),
                source: source.to_string(),
                expected: expected.map(str::to_string),
            },
            assertions,
        )
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
        assert_eq!(runtime.assertions_run(), assertions, "{source}");
        returns.push(runtime.serialize_json(&value).unwrap());
    }
    assert_eq!(returns[0], returns[1]);
    runtimes
}

fn graphs_match(implicit: &Runtime, explicit: &Runtime) {
    assert!(implicit.graph.keys().eq(explicit.graph.keys()));
    for node in implicit.graph.nodes() {
        let other = explicit.graph.retrieve(&node.key).unwrap();
        assert_eq!(node.kind, other.kind, "{}", node.key);
        assert_eq!(node.dependencies, other.dependencies, "{}", node.key);
        assert_eq!(node.dependents, other.dependents, "{}", node.key);
        assert!(
            implicit
                .graph
                .shape_dependencies(&node.key)
                .eq(explicit.graph.shape_dependencies(&node.key)),
            "{}",
            node.key
        );
    }
}

#[test]
fn ordinary_identifier_sources_preserve_full_for_metadata_body_and_canonical_keys() {
    for name in [
        "values", "Entry", "it", "_private", "A1", "type", "value", "why", "affects",
    ] {
        let implicit =
            format!("for {name}:\n    it.amount += 2\n    if it.amount > 3:\n        pass");
        let explicit =
            format!("for it of {name}:\n    it.amount += 2\n    if it.amount > 3:\n        pass");
        let statements = equivalent(&implicit, &explicit);
        let [
            Stmt::For {
                variable,
                iterable,
                body,
            },
        ] = statements.as_slice()
        else {
            panic!("expected exactly one existing For node");
        };
        assert_eq!(variable, "it");
        assert_eq!(*iterable, Expr::Identifier(name.to_string()));
        assert_eq!(
            *body,
            parse("it.amount += 2\nif it.amount > 3:\n    pass").unwrap()
        );
        assert_eq!(generate_all(&statements), format!("for it of {name}"));
        assert_eq!(
            NodeKey::block(&statements, None).as_str(),
            format!("block(for it of {name})")
        );
    }
}

#[test]
fn existing_tokens_and_real_suite_layout_are_left_to_the_existing_parser() {
    assert_eq!(
        tokenize("for values:\n    pass")
            .unwrap()
            .into_iter()
            .map(|spanned| spanned.token)
            .collect::<Vec<_>>(),
        [
            Token::Keyword(Keyword::For),
            Token::Identifier("values".to_string()),
            Token::Colon,
            Token::Newline,
            Token::Indent,
            Token::Keyword(Keyword::Pass),
            Token::Dedent,
            Token::Eof,
        ]
    );
    for layout in [
        "\n    ",
        "\r\n    ",
        "\n\t",
        " # suite\n\n# blank\n    # body\n    ",
        "\n;\n    ",
        "\n    ;\n    ",
    ] {
        equivalent(
            &format!("for values:{layout}pass"),
            "for it of values:\n    pass",
        );
    }
    for suffix in ["", "\nnext = 3", "\n# boundary\nnext = 3", "\n; next = 3"] {
        equivalent(
            &format!("for\tvalues:\n    pass{suffix}"),
            &format!("for it of values:\n    pass{suffix}"),
        );
    }
}

#[test]
fn formerly_rejected_named_headers_have_distinct_nonvacuous_positive_pairs() {
    for name in ["values", "it", "_private", "A1", "type", "why"] {
        let explicit = format!(
            "class Entry\nfirst = Entry()\nfirst.amount = 2\n{name} = [first]\n\
             seen = []\nfor it of {name}:\n    seen.push(it.amount)\nassert(seen, [2])"
        );
        check_forms(&implicit_loops(&explicit), &explicit, 1, None);
    }
    let explicit = "class Entry\nfirst = Entry()\nfirst.amount = 2\nseen = []\nfor it of Entry:\n    seen.push(it.amount)\nassert(seen, [2])";
    check_forms(&implicit_loops(explicit), explicit, 1, None);
}

#[test]
fn unsupported_named_prefixes_keep_exact_old_messages_and_positions() {
    for (suffix, column) in [
        (":", 11),
        (":\n", 11),
        (":\n# no body", 11),
        (":\npass", 11),
        (":\n;\npass", 11),
        (": pass", 11),
        (": { pass }", 11),
        (":;\n    pass", 11),
        (": ;\n    pass", 11),
        (".items:\n    pass", 11),
        ("[0]:\n    pass", 11),
        ("():\n    pass", 11),
        (" + other:\n    pass", 12),
        (", other:\n    pass", 11),
        (" => it:\n    pass", 12),
        (";:\n    pass", 11),
    ] {
        let source = format!("for values{suffix}");
        let error = Runtime::check(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(
            error.message(),
            "Expected 'of' in for statement",
            "{source}"
        );
        assert_eq!(error.position(), Some(Position::new(1, column)), "{source}");
    }
    for source in [
        "for values\n:\n    pass",
        "for values # header\n:\n    pass",
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), "Expected 'of' in for statement");
        assert_eq!(error.position(), Some(Position::new(1, 5)));
    }
}

#[test]
fn shorthand_requires_one_ordinary_identifier_without_header_separator_skipping() {
    for (source, message, line, column) in [
        ("for :", "Expected a name but found ':'", 1, 6),
        ("for", "Expected a name but found end of input", 1, 4),
        (
            "for (values):\n    pass",
            "Expected a name but found '('",
            1,
            6,
        ),
        (
            "for [values]:\n    pass",
            "Expected a name but found '['",
            1,
            6,
        ),
        (
            "for { values }:\n    pass",
            "Expected a name but found '{'",
            1,
            7,
        ),
        (
            "for \"values\":\n    pass",
            "Expected a name but found a string",
            1,
            13,
        ),
        (
            "for 1:\n    pass",
            "Expected a name but found number 1",
            1,
            6,
        ),
        (
            "for null:\n    pass",
            "Expected a name but found keyword 'null'",
            1,
            9,
        ),
        (
            "for this:\n    pass",
            "Expected a name but found keyword 'this'",
            1,
            9,
        ),
        (
            "for $Entry:\n    pass",
            "Expected a name but found '$Entry'",
            1,
            11,
        ),
        (
            "for\nvalues:\n    pass",
            "Expected a name but found end of line",
            2,
            1,
        ),
        (
            "for; values:\n    pass",
            "Expected a name but found ';'",
            1,
            6,
        ),
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
}

#[test]
fn qualifying_headers_do_not_hide_body_or_lexical_errors() {
    for (body, message, column) in [
        ("it.amount +)", "Unexpected ')'", 17),
        ("it.amount = @", "Unexpected character '@'", 17),
        (
            "it.amount = { value = 1 }",
            "Expected ':' but found '='",
            25,
        ),
    ] {
        let implicit = format!("for values:\n    {body}");
        let explicit = format!("for it of values:\n    {body}");
        let error = Runtime::check(&implicit).unwrap_err();
        assert_eq!(error, Runtime::check(&explicit).unwrap_err());
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(2, column)));
    }
}

#[test]
fn explicit_of_and_in_keep_arbitrary_expressions_and_all_existing_suite_forms() {
    for source in [
        "values",
        "(values)",
        "container.items",
        "values[0]",
        "supply()",
        "[first, second, 99]",
        "[]",
        "$Entry",
        "null",
        "{ one: first }",
        "this.values",
        "left || right",
    ] {
        for body in [": pass", ": { pass }", ":\n    pass"] {
            equivalent(
                &format!("for current of {source}{body}"),
                &format!("for current in {source}{body}"),
            );
        }
    }
    // Identical legacy inputs are legitimate AST comparisons, not runtime pairs.
    equivalent("for current of values: pass", "for current of values: pass");
    for (source, message, column) in [
        ("for current of", "Unexpected end of input", 15),
        ("for current in", "Unexpected end of input", 15),
        (
            "for current of values:\npass",
            "Expected an indented block but found keyword 'pass'",
            1,
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        let line = if source.contains('\n') { 2 } else { 1 };
        assert_eq!(
            error.position(),
            Some(Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn legacy_explicit_expression_sources_and_inline_braced_bodies_still_execute() {
    for source in [
        "values",
        "(values)",
        "container.items",
        "supply()",
        "[first, second, 99]",
        "Entry",
    ] {
        for keyword in ["of", "in"] {
            for body in [
                ": seen.push(current.amount)",
                ": { seen.push(current.amount) }",
                ":\n    seen.push(current.amount)",
            ] {
                let code = format!(
                    "class Entry\nfirst = Entry()\nfirst.amount = 2\nsecond = Entry()\n\
                     second.amount = 5\nvalues = [first, second]\ncontainer = {{ items: values }}\n\
                     def supply() => values\nseen = []\nfor current {keyword} {source}{body}\n\
                     assert(seen, [2, 5])"
                );
                let mut runtime = Runtime::new();
                common::check_with_count(
                    &mut runtime,
                    &common::Case {
                        title: "legacy explicit loop".to_string(),
                        source: code,
                        expected: None,
                    },
                    1,
                )
                .unwrap();
                assert_eq!(runtime.assertions_run(), 1);
            }
        }
    }
}

#[test]
fn registered_instances_filter_lists_in_order_with_duplicate_visits_and_empty_sources() {
    let explicit = r#"
class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
plain = Object()
values = [plain, second, 99, first, null, { amount: 9 }, second]
seen = []
total = 0
for it of values:
    seen.push(it.amount)
    total += it.amount
assert(seen, [5, 2, 5])
assert(total, 12)
empty = []
for it of empty:
    throw "empty body executed"
primitives = [1, null, true, "text", { amount: 20 }, plain]
for it of primitives:
    throw "primitive body executed"
assert(total, 12)
total
"#;
    check_forms(&implicit_loops(explicit), explicit, 3, Some("12"));
}

#[test]
fn class_populations_are_ordered_one_shot_snapshots_not_standing_declarations() {
    let explicit = r#"
class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
scale = 2
population = Entry
seen = []
for it of population:
    seen.push(it.amount)
    it.score = it.amount * scale
assert(seen, [2, 5])
assert(first.score, 4)
assert(second.score, 10)
scale = 3
assert(first.score, 4)
assert(second.score, 10)
third = Entry()
third.amount = 7
assert(third.score, null)
for it of Entry:
    it.score = it.amount * scale
assert(first.score, 6)
assert(second.score, 15)
assert(third.score, 21)
third.score
"#;
    check_forms(&implicit_loops(explicit), explicit, 9, Some("21"));
}

#[test]
fn mutating_a_source_list_or_creating_instances_cannot_extend_the_current_snapshot() {
    let explicit = r#"
class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
later = Entry()
later.amount = 7
values = [first, second]
seen = []
for it of values:
    seen.push(it.amount)
    values.push(later)
assert(seen, [2, 5])
assert(values.length, 4)
replayed = []
for it of values:
    replayed.push(it.amount)
assert(replayed, [2, 5, 7, 7])
population = []
for it of Entry:
    population.push(it.amount)
    Entry()
assert(population, [2, 5, 7])
assert(Entry.length, 6)
Entry.length
"#;
    check_forms(&implicit_loops(explicit), explicit, 5, Some("6"));
}

#[test]
fn nested_global_and_same_name_source_bindings_shadow_and_restore_it() {
    let explicit = r#"
class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
outer = Entry()
outer.amount = 100
groups = [outer]
values = [first, second]
it = 50
seen = []
for it of groups:
    seen.push(it.amount)
    for it of values:
        seen.push(it.amount)
    seen.push(it.amount)
assert(seen, [100, 2, 5, 100])
assert(it, 50)
it = values
nested = []
for it of it:
    nested.push(it.amount)
assert(nested, [2, 5])
assert(it.length, 2)
def read(it):
    for it of it:
        return it.amount
    return -1
assert(read(values), 2)
assert(read([]), -1)
assert(it.length, 2)
read(values)
"#;
    check_forms(&implicit_loops(explicit), explicit, 7, Some("2"));
}

#[test]
fn return_bare_return_no_return_empty_sources_and_nested_control_flow_stay_explicit() {
    let explicit = r#"
class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
values = [first, second]
def choose(records):
    for it of records:
        if it.amount > 2:
            return it.amount
    return -1
def bare(records):
    for it of records:
        return
    return -1
def noResult(records):
    for it of records:
        it.amount
def nested(records):
    for it of records:
        for it of records:
            if it.amount > 2:
                return it.amount
    return -1
assert(choose(values), 5)
assert(choose([]), -1)
assert(bare(values), null)
assert(bare([]), -1)
assert(noResult(values), null)
assert(noResult([]), null)
assert(nested(values), 5)
assert(nested([]), -1)
choose(values)
"#;
    check_forms(&implicit_loops(explicit), explicit, 8, Some("5"));
    let explicit = "class Empty\nfor it of Empty:\n    throw \"empty class body executed\"";
    check_forms(&implicit_loops(explicit), explicit, 0, Some("null"));
}

#[test]
fn methods_initializers_receivers_and_inheritance_keep_their_enclosing_instance() {
    let explicit = r#"
class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
values = [first, second]
class Reader:
    def init(records):
        this.total = 0
        for it of records:
            this.total += it.amount
            it.owner = self
    def first(records):
        for it of records:
            return it.amount + self.total
        return -1
class Child: Reader
    def init(records):
        super(records)
    def first(records):
        for it of records:
            return it.amount + this.total
        return -1
base = Reader(values)
reader = Child(values)
assert(base.first(values), 9)
assert(base.first([]), -1)
assert(reader.total, 7)
assert(first.owner, reader)
assert(second.owner, reader)
assert(reader.first(values), 9)
assert(reader.first([]), -1)
assert(Child.length, 1)
reader.first(values)
"#;
    check_forms(&implicit_loops(explicit), explicit, 8, Some("9"));
}

#[test]
fn caught_errors_restore_state_and_scopes_and_both_catches_really_run() {
    let explicit = r#"
class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
values = [first, second]
it = 50
saved = 7
try:
    for it of values:
        saved += it.amount
        it.amount += 1
        if it.amount > 5:
            throw "stop"
catch error:
    assert(error, "stop")
assert(first.amount, 2)
assert(second.amount, 5)
assert(saved, 7)
assert(it, 50)
broken = null
try:
    for it of broken:
        first.amount = 99
catch error:
    assert(error, TypeError("broken is not iterable"))
assert(first.amount, 2)
for it of values:
    it.score = it.amount * 2
assert(first.score, 4)
assert(second.score, 10)
first.score + second.score
"#;
    check_forms(&implicit_loops(explicit), explicit, 9, Some("14"));
}

#[test]
fn uncaught_body_errors_roll_back_the_whole_run_then_restore_imperative_tracking() {
    let explicit = "saved = 99\nfor it of values:\n    saved += it.amount\n    it.amount += 1\n    throw \"stop\"";
    let implicit = implicit_loops(explicit);
    equivalent(&implicit, explicit);
    let mut runtimes = [Runtime::new(), Runtime::new()];
    let mut errors = Vec::new();
    for (runtime, source) in runtimes.iter_mut().zip([implicit.as_str(), explicit]) {
        runtime
            .run("class Entry\nfirst = Entry()\nfirst.amount = 2\nvalues = [first]\nit = 50\nsaved = 7")
            .unwrap();
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Thrown);
        assert_eq!(error.thrown_value(), Some(&Value::string("stop")));
        assert_eq!(error.position(), Some(Position::new(2, 1)));
        errors.push(error);
        runtime
            .run(
                "assert(saved, 7)\nassert(first.amount, 2)\nassert(it, 50)\n\
                 input = 2\noutput = input * 3\ninput = 4\nassert(output, 12)",
            )
            .unwrap();
        assert_eq!(runtime.assertions_run(), 4);
        assert!(runtime.take_assertions().is_empty());
    }
    assert_eq!(errors[0], errors[1]);
    graphs_match(&runtimes[0], &runtimes[1]);
}

#[test]
fn named_strings_now_iterate_scalars_in_both_forms_without_leaking_it() {
    let explicit = r#"
broken = "text"
it = "outer"
seen = []
for it of broken:
    seen.push(it)
assert(seen, ["t", "e", "x", "t"])
assert(broken, "text")
assert(it, "outer")
"#;
    check_forms(&implicit_loops(explicit), explicit, 3, None);
}

#[test]
fn named_noniterables_undefined_sources_and_callables_keep_errors_without_auto_invocation() {
    for setup in [
        "broken = null",
        "broken = 7",
        "broken = true",
        "broken = { amount: 2 }",
        "broken = Object()",
        "broken = Entry()",
        "broken = value => value",
        "def broken():\n    Entry()\n    return []",
    ] {
        let explicit = "saved = 99\nfor it of broken:\n    saved = 100";
        let implicit = implicit_loops(explicit);
        equivalent(&implicit, explicit);
        for source in [implicit.as_str(), explicit] {
            let mut runtime = Runtime::new();
            runtime
                .run(&format!("class Entry\nsaved = 7\n{setup}"))
                .unwrap();
            let initial = runtime.run("Entry.length").unwrap();
            let error = runtime.run(source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Type, "{setup}");
            assert_eq!(error.message(), "broken is not iterable", "{setup}");
            assert_eq!(error.position(), Some(Position::new(2, 1)), "{setup}");
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
            assert_eq!(runtime.run("Entry.length").unwrap(), initial);
        }
    }
    for (body, kind, message) in [
        (
            "for missing:\n    pass",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "for values:\n    missing",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "for values:\n    this.amount",
            ErrorKind::Reference,
            "this is not defined",
        ),
        (
            "for values:\n    self.amount",
            ErrorKind::Reference,
            "this is not defined",
        ),
    ] {
        let explicit = explicit_loops(body);
        equivalent(body, &explicit);
        for source in [body, explicit.as_str()] {
            let mut runtime = Runtime::new();
            runtime
                .run("class Entry\nfirst = Entry()\nvalues = [first]")
                .unwrap();
            let error = runtime.run(source).unwrap_err();
            assert_eq!(error.kind(), kind);
            assert_eq!(error.message(), message);
            assert_eq!(error.position(), Some(Position::new(1, 1)));
        }
    }
}

#[test]
fn one_shot_graphs_shapes_and_reasoning_match_without_leaking_iteration_bindings() {
    let explicit = r#"
class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
values = [first, second]
scale = 2
for it of values:
    it.score = it.amount * scale
observed = first.score + second.score
assert(observed, 14)
scale = 3
assert(observed, 14)
first.amount = 4
assert(first.score, 4)
assert(observed, 14)
for it of values:
    it.score = it.amount * scale
assert(observed, 27)
observed
"#;
    let [mut implicit, mut explicit] =
        check_forms(&implicit_loops(explicit), explicit, 5, Some("27"));
    graphs_match(&implicit, &explicit);
    for runtime in [&implicit, &explicit] {
        assert!(!runtime.graph.contains(&NodeKey::variable("it")));
        assert!(
            runtime
                .graph
                .keys()
                .all(|key| !key.as_str().starts_with("for "))
        );
    }
    for source in ["why observed", "model |> why", "model |> affects"] {
        let actual = implicit.run(source).unwrap();
        let expected = explicit.run(source).unwrap();
        assert_eq!(
            implicit.serialize_json(&actual).unwrap(),
            explicit.serialize_json(&expected).unwrap()
        );
    }
}

#[test]
fn compile_check_and_program_positions_share_the_unchanged_source_entry_points() {
    let implicit = "before = 1\nfor values:\n    pass\nafter = 2";
    let explicit = "before = 1\nfor it of values:\n    pass\nafter = 2";
    Runtime::check(implicit).unwrap();
    Runtime::check(explicit).unwrap();
    let implicit_program = parse_program(implicit).unwrap();
    let explicit_program = statement::compile(explicit).unwrap();
    assert_eq!(implicit_program.statements, explicit_program.statements);
    assert_eq!(implicit_program.positions, explicit_program.positions);
    assert_eq!(
        implicit_program.positions,
        [
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(4, 1)
        ]
    );
    assert_eq!(
        statement::compile(implicit).unwrap().statements,
        parse(implicit).unwrap()
    );
}

#[test]
fn direct_for_node_api_restores_frames_receivers_and_imperative_mode_on_every_exit() {
    for body in [
        "pass",
        "return it.amount",
        "return",
        "missing",
        "throw \"stop\"",
    ] {
        for header in ["for values:", "for it of values:"] {
            let mut runtime = Runtime::new();
            runtime
                .run("class Entry\nfirst = Entry()\nfirst.amount = 2\nplain = Object()")
                .unwrap();
            let first = runtime.run("first").unwrap();
            let plain = runtime.run("plain").unwrap();
            let mut scope = Scope::new();
            scope.push();
            scope.declare("it", Value::Number(77.0));
            scope.declare(
                "values",
                Value::List(vec![
                    Value::Object(ObjectId::named("unregistered")),
                    Value::Number(9.0),
                    Value::Null,
                    plain,
                    first,
                ]),
            );
            let receiver = ObjectId::named("first");
            scope.set_this(Some(receiver.clone()));
            scope.set_instance(Some(receiver.clone()));
            let statements = parse(&format!("{header}\n    {body}")).unwrap();
            let mut node = Nuc::convert(&mut runtime, &mut scope, &statements[0]).unwrap();
            let result = node.run(&mut runtime, &mut scope);
            match (body, result) {
                ("pass", Ok(outcome)) => assert!(matches!(outcome.flow, Flow::Normal(Value::Null))),
                ("return it.amount", Ok(outcome)) => {
                    assert!(matches!(outcome.flow, Flow::Return(Value::Number(2.0))));
                }
                ("return", Ok(outcome)) => {
                    assert!(matches!(outcome.flow, Flow::Return(Value::Null)))
                }
                ("missing", Err(error)) => {
                    assert_eq!(error.kind(), ErrorKind::Reference);
                    assert_eq!(error.message(), "missing is not defined");
                }
                ("throw \"stop\"", Err(error)) => {
                    assert_eq!(error.thrown_value(), Some(&Value::string("stop")));
                }
                _ => panic!("unexpected direct-node outcome for {header} {body}"),
            }
            assert_eq!(scope.depth(), 2);
            assert_eq!(scope.retrieve("it"), Some(&Value::Number(77.0)));
            assert_eq!(scope.this(), Some(&receiver));
            assert_eq!(scope.instance(), Some(&receiver));
            runtime
                .run("input = 2\noutput = input * 3\ninput = 4")
                .unwrap();
            assert_eq!(runtime.run("output").unwrap(), Value::Number(12.0));
        }
    }
}

#[test]
fn direct_for_node_filters_unregistered_objects_and_restores_empty_and_invalid_source_frames() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Entry\nfirst = Entry()\nfirst.amount = 2")
        .unwrap();
    let first = runtime.run("first").unwrap();
    let mut scope = Scope::new();
    scope.declare("it", Value::Number(77.0));
    scope.declare("total", Value::Number(0.0));
    scope.declare(
        "values",
        Value::List(vec![
            Value::Object(ObjectId::named("unregistered")),
            first.clone(),
            Value::Undefined,
            first,
        ]),
    );
    let statements = equivalent(
        "for values:\n    total += it.amount",
        "for it of values:\n    total += it.amount",
    );
    let mut node = Nuc::convert(&mut runtime, &mut scope, &statements[0]).unwrap();
    assert!(matches!(
        node.run(&mut runtime, &mut scope).unwrap().flow,
        Flow::Normal(Value::Null)
    ));
    assert_eq!(scope.retrieve("total"), Some(&Value::Number(4.0)));
    for source in [Value::List(vec![]), Value::Null, Value::Undefined] {
        scope.declare("values", source.clone());
        let result = node.run(&mut runtime, &mut scope);
        if matches!(source, Value::List(_)) {
            assert!(matches!(result.unwrap().flow, Flow::Normal(Value::Null)));
        } else {
            let error = result.err().expect("noniterable sources must fail");
            assert_eq!(error.kind(), ErrorKind::Type);
            assert_eq!(error.message(), "values is not iterable");
        }
        assert_eq!(scope.depth(), 1);
        assert_eq!(scope.retrieve("it"), Some(&Value::Number(77.0)));
        assert_eq!(scope.retrieve("total"), Some(&Value::Number(4.0)));
    }
}

#[test]
fn shared_guard_keeps_single_typed_and_multiple_function_blocks_and_arrows_exact() {
    for (bare, parenthesized) in [
        ("amount", "(amount)"),
        ("amount: int", "(amount: int)"),
        ("left, right", "(left, right)"),
        ("left: int, right", "(left: int, right)"),
        ("left, right: int", "(left, right: int)"),
        ("left: int, right: int", "(left: int, right: int)"),
    ] {
        for layout in ["\n    ", " # suite\n\n    ", "\n;\n    ", "\n    ;\n    "] {
            equivalent(
                &format!("def f {bare}:{layout}return 1"),
                &format!("def f{parenthesized}:\n    return 1"),
            );
        }
        equivalent(
            &format!("def f {bare} => 1"),
            &format!("def f{parenthesized} => 1"),
        );
        for suffix in [
            ":",
            ":\n# no body",
            ": pass",
            ": { pass }",
            ":;\n    pass",
            ":\npass",
        ] {
            let source = format!("def f {bare}{suffix}");
            let error = Runtime::check(&source).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
            let first = bare.split([':', ',']).next().unwrap();
            assert_eq!(
                error.message(),
                format!("Expected '(' but found identifier '{first}'"),
                "{source}"
            );
            assert_eq!(error.position(), Some(Position::new(1, 7)), "{source}");
        }
    }
}

#[test]
fn inherited_nesting_operand_postfix_and_statement_limits_fail_atomically_then_recover() {
    let mut runtime = Runtime::new();
    runtime.run("saved = 7").unwrap();
    for (body, message) in [
        (
            format!("{}it", "-".repeat(200)),
            "Expressions are nested too deeply",
        ),
        (
            format!("it{}", " + 1".repeat(200)),
            "Expression has too many operands",
        ),
        (
            format!("root{}", ".next".repeat(200)),
            "Expression has too many parts",
        ),
        (
            format!("{}it", "saved = ".repeat(200)),
            "Expressions are nested too deeply",
        ),
        (
            format!("{}pass{}", "{ ".repeat(65), " }".repeat(65)),
            "Statements are nested too deeply",
        ),
    ] {
        let mut errors = Vec::new();
        for header in ["for values:", "for it of values:"] {
            let error = runtime
                .run(&format!("saved = 99\n{header}\n    {body}"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Syntax);
            assert_eq!(error.message(), message);
            assert!(error.position().is_some());
            assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
            errors.push(error);
        }
        assert_eq!(errors[0], errors[1]);
    }
    runtime
        .run("input = 2\noutput = input * 3\ninput = 4")
        .unwrap();
    assert_eq!(runtime.run("output").unwrap(), Value::Number(12.0));
}

#[test]
fn all_eight_own_document_cases_execute_their_protected_assertions_and_returns_in_both_forms() {
    let documents = [
        (
            common::cases(include_str!("../nucleoid.spec.md")),
            vec![
                ("Nucleoid defines implicit iterator indented loops", 3, None),
                ("Nucleoid preserves implicit iterator loop scopes", 4, None),
                (
                    "Nucleoid preserves implicit iterator loop population snapshots",
                    8,
                    None,
                ),
                (
                    "Nucleoid preserves implicit iterator loop return flow",
                    5,
                    Some("2"),
                ),
                (
                    "Nucleoid rolls back failed implicit iterator loops",
                    8,
                    Some("14"),
                ),
            ],
        ),
        (
            common::cases(include_str!("reference.md")),
            vec![(
                "Nucleoid preserves implicit iterator loop class populations",
                2,
                None,
            )],
        ),
        (
            common::snippets(include_str!("../README.md"), "README.md"),
            vec![("Named source loops can bind it implicitly", 1, None)],
        ),
        (
            common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
            vec![("Implicit iterator loops keep one shot updates", 4, None)],
        ),
    ];
    let mut protected = 0;
    for (cases, own) in documents {
        let mut previous = None;
        for (title, assertions, expected) in own {
            let matching = cases
                .iter()
                .enumerate()
                .filter(|(_, case)| case.title == title)
                .collect::<Vec<_>>();
            let [(index, case)] = matching.as_slice() else {
                panic!("expected one own case titled {title}");
            };
            assert!(previous.is_none_or(|previous| previous < *index));
            previous = Some(*index);
            assert_eq!(case.expected.as_deref(), expected);
            let explicit = explicit_loops(&case.source);
            check_forms(&case.source, &explicit, assertions, expected);
            protected += assertions;
        }
    }
    assert_eq!(
        protected, 35,
        "protect this proposal's eight additions, not future corpus size"
    );
}

fn repl(source: &str) -> Output {
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
        .write_all(source.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn native_stdin_keeps_identical_results_graphs_and_flushes_real_bodies_at_eof() {
    let setup =
        ".json on\n.graph on\nclass Entry\nfirst = Entry()\nfirst.amount = 2\nvalues = [first]\n";
    for suffix in ["\n\nassert(first.score, 4)\nfirst.score\n.exit\n", ""] {
        let explicit = format!(
            "{setup}for it of values:\n    it.score = it.amount * 2\n    return it.score{suffix}"
        );
        let implicit = implicit_loops(&explicit);
        assert_ne!(implicit, explicit);
        let outputs = [repl(&implicit), repl(&explicit)];
        for output in &outputs {
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("values [variable]"), "{stdout}");
            assert!(!stdout.contains("first.score [property]"), "{stdout}");
            assert!(stdout.lines().any(|line| line == "4.0"), "{stdout}");
        }
        assert_eq!(outputs[0].stdout, outputs[1].stdout);
    }
}

#[test]
fn native_stdin_recovers_after_same_prefix_body_limit_and_named_null_errors() {
    let source = format!(
        ".json on\nclass Entry\nfirst = Entry()\nfirst.amount = 2\nvalues = [first]\n\
         for values.items:\n    pass\n\n\
         for values:\n    it.amount{}\n\n\
         broken = null\nfor broken:\n    first.amount = 99\n\n\
         for values:\n    it.score = it.amount * 2\n\n\
         assert(first.amount, 2)\nassert(first.score, 4)\nfirst.score",
        " + 1".repeat(200)
    );
    let output = repl(&source);
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 3, "{stderr}");
    assert!(
        stderr.contains("SyntaxError: Expected 'of' in for statement (line 1, column 11)"),
        "{stderr}"
    );
    assert!(
        stderr.contains("SyntaxError: Expression has too many operands"),
        "{stderr}"
    );
    assert!(
        stderr.contains("TypeError: broken is not iterable (line 1, column 1)"),
        "{stderr}"
    );
    assert!(!stderr.contains("assertion failed"), "{stderr}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.lines().any(|line| line == "4.0"), "{stdout}");
}
