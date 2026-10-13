//! Bare `super` uses the existing zero-argument AST and parent-call runtime.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{Ast, Expr, Stmt, parse, parse_expression, parse_program};
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn equivalent(bare: &str, explicit: &str) {
    let bare_ast = parse(bare).unwrap();
    let explicit_ast = parse(explicit).unwrap();
    assert_eq!(bare_ast, explicit_ast, "{bare}");
    assert_eq!(
        generate_all(&bare_ast),
        generate_all(&explicit_ast),
        "{bare}"
    );
}

/// Both forms must preserve the program's return and run every assertion,
/// including those in catch blocks.
fn check_forms(explicit: &str, assertions: usize) {
    let bare = explicit.replace("super()", "super");
    equivalent(&bare, explicit);
    let mut returns = Vec::new();
    for source in [bare.as_str(), explicit] {
        let mut runtime = Runtime::new();
        let value = runtime.run(source).unwrap();
        returns.push(runtime.serialize_json(&value).unwrap());
        assert_eq!(runtime.assertions_run(), assertions, "{source}");
        let failures = runtime.take_assertions();
        assert!(failures.is_empty(), "{source}\n{failures:?}");
    }
    assert_eq!(returns[0], returns[1]);
}

#[test]
fn bare_super_lowers_to_the_existing_call_ast_and_canonical_source() {
    let expression = parse_expression("super").unwrap();
    assert_eq!(expression, Expr::Super(Vec::new()));
    assert_eq!(expression.to_string(), "super()");
    let Ast::Call(call) = Ast::convert(&expression) else {
        panic!("super must use the existing call dispatch");
    };
    assert!(call.arguments().is_empty());
    assert_eq!(parse("super").unwrap(), [Stmt::Expression(expression)]);

    // A normal identifier is still a reference, not an automatic nullary call.
    assert_eq!(
        parse_expression("parent").unwrap(),
        Expr::Identifier("parent".to_string())
    );
    let error = Runtime::check("def init => super").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
    assert_eq!(error.message(), "Expected '(' but found '=>'");
}

#[test]
fn bare_super_is_a_primary_expression_not_just_a_statement() {
    for (bare, explicit) in [
        ("answer = super", "answer = super()"),
        ("return super", "return super()"),
        ("def init() => super", "def init() => super()"),
        ("callback = () => super", "callback = () => super()"),
        ("items = [super, super(7)]", "items = [super(), super(7)]"),
        ("record = {result: super}", "record = {result: super()}"),
        ("assert(super, null)", "assert(super(), null)"),
        ("(super)", "(super())"),
        ("!super", "!super()"),
        ("typeof super", "typeof super()"),
        ("super - 1", "super() - 1"),
        ("super == null", "super() == null"),
        ("super and true", "super() and true"),
        ("super or 7", "super() or 7"),
    ] {
        equivalent(bare, explicit);
    }
}

#[test]
fn existing_postfix_grammar_applies_to_the_super_result() {
    for (bare, explicit) in [
        ("super.amount", "super().amount"),
        ("super[0]", "super()[0]"),
        ("super[1:3]", "super()[1:3]"),
        ("(super)()", "(super())()"),
        ("(super)(7)", "(super())(7)"),
        ("super.amount(7)[0]", "super().amount(7)[0]"),
    ] {
        equivalent(bare, explicit);
    }

    assert_eq!(
        parse_expression("(super)()").unwrap(),
        Expr::Call {
            callee: Box::new(Expr::Super(Vec::new())),
            arguments: Vec::new(),
        }
    );
    assert_eq!(
        parse_expression("(super)()").unwrap().to_string(),
        "super()()"
    );
}

#[test]
fn immediate_parentheses_keep_explicit_arguments_and_their_rendering() {
    let expected = Expr::Super(vec![
        Expr::Number(1.0),
        Expr::String("A".to_string()),
        Expr::This,
    ]);
    for source in [
        "super(1, \"A\", this)",
        "super (1, \"A\", this,)",
        "super(\n    1,\n    \"A\",\n    this,\n)",
        "(super\n(1, \"A\", this))",
        "[super\n(1, \"A\", this)][0]",
    ] {
        let expression = parse_expression(source).unwrap();
        if source.starts_with('[') {
            equivalent(source, "[super(1, \"A\", this)][0]");
        } else {
            assert_eq!(expression, expected, "{source}");
            assert_eq!(expression.to_string(), "super(1,\"A\",this)", "{source}");
        }
    }

    equivalent("super()", "super");
    equivalent("super ()", "super");
    equivalent("(super # logical line continues\n(7))", "super(7)");
}

#[test]
fn separators_and_adjacent_statements_do_not_become_implicit_arguments() {
    for (bare, explicit) in [
        ("super\n(7)", "super()\n(7)"),
        ("super;\n(7)", "super();\n(7)"),
        ("super; (7)", "super(); (7)"),
        ("super # empty call\n\n(7)", "super() # empty call\n\n(7)"),
        ("super\n    (7)", "super()\n    (7)"),
        ("super amount", "super() amount"),
        ("super 7", "super() 7"),
        ("super next = 1", "super() next = 1"),
        ("{ super }", "{ super() }"),
        (
            "def init():\n    super\n    (7)",
            "def init():\n    super()\n    (7)",
        ),
        (
            "if active:\n    super\nelse:\n    pass",
            "if active:\n    super()\nelse:\n    pass",
        ),
        (
            "try:\n    super\ncatch error:\n    pass",
            "try:\n    super()\ncatch error:\n    pass",
        ),
    ] {
        equivalent(bare, explicit);
    }

    let program = parse_program("super\n(7)\nnext = 1").unwrap();
    assert_eq!(
        program.statements,
        [
            Stmt::Expression(Expr::Super(Vec::new())),
            Stmt::Expression(Expr::Number(7.0)),
            Stmt::Assign {
                target: Expr::Identifier("next".to_string()),
                value: Expr::Number(1.0),
            },
        ]
    );
    assert_eq!(
        program.positions,
        [
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(3, 1),
        ]
    );
}

#[test]
fn malformed_explicit_calls_retain_accepted_errors_and_exact_positions() {
    // Captured with the unchanged accepted runtime before the parser edit.
    for (source, message, line, column) in [
        ("super(", "Unexpected end of input", 1, 7),
        ("super(1", "Expected ')' but found end of input", 1, 8),
        ("super(,)", "Unexpected ','", 1, 8),
        ("super(1 2)", "Expected ')' but found number 2", 1, 9),
        ("super(1 +)", "Unexpected ')'", 1, 11),
        ("super(;)", "Unexpected ';'", 1, 8),
        ("super(1,,2)", "Unexpected ','", 1, 10),
        ("super(\n1 +\n)", "Unexpected ')'", 3, 2),
        ("super(1;", "Expected ')' but found ';'", 1, 8),
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

    let mut runtime = Runtime::new();
    runtime.run("saved = 7").unwrap();
    let error = runtime.run("saved = 3\nsuper(1 +)").unwrap_err();
    assert_eq!(error.message(), "Unexpected ')'");
    assert_eq!(error.position(), Some(Position::new(2, 11)));
    assert_eq!(runtime.run("saved").unwrap(), Value::Number(7.0));
}

#[test]
fn parent_initialization_uses_the_same_object_and_returns_null() {
    check_forms(
        r#"
class Parent:
    def init():
        this.amount = 4
        this.parent_id = this.id
        return 99
class Child: Parent
    def init():
        this.result = super()
        this.amount = this.amount + 1
child = Child()
assert(child.amount, 5)
assert(child.parent_id, "child")
assert(child.id, "child")
assert(child.result, null)
assert(Parent.length, 0)
assert(Child.length, 1)
"#,
        6,
    );
}

#[test]
fn constructor_context_advances_the_chain_even_for_passive_subclasses() {
    check_forms(
        r#"
class Base:
    def init():
        this.trace = "A"
class Middle: Base
    def init():
        super()
        this.trace = this.trace + "B"
class Leaf: Middle
    def init():
        super()
        this.trace = this.trace + "C"
class Passive: Leaf
    pass
leaf = Leaf()
passive = Passive()
assert(leaf.trace, "ABC")
assert(passive.trace, "ABC")
assert(Base.length, 0)
assert(Middle.length, 0)
assert(Leaf.length, 1)
assert(Passive.length, 1)
"#,
        6,
    );
}

#[test]
fn bare_calls_pass_no_locals_or_following_expressions_as_parent_arguments() {
    check_forms(
        r#"
class Parent:
    def init(amount):
        this.amount = amount
class Bare: Parent
    def init(amount):
        super() amount
class Boundary: Parent
    def init():
        super()
        (9)
class Explicit: Parent
    def init(amount):
        super(amount,)
bare = Bare(8)
boundary = Boundary()
explicit = Explicit(9)
assert(bare.amount, null)
assert(boundary.amount, null)
assert(explicit.amount, 9)
assert(Parent.length, 0)
assert(Bare.length, 1)
"#,
        5,
    );
}

#[test]
fn methods_use_receiver_class_fallback_and_the_same_object() {
    check_forms(
        r#"
class Base:
    def init():
        this.base = true
class Middle: Base
    def init():
        super()
        this.middle = true
    def reset() => super()
class Leaf: Middle
    def init():
        this.leaf = true
    def reset() => super()
middle = Middle()
leaf = Leaf()
assert(leaf.base, null)
assert(leaf.middle, null)
assert(leaf.reset(), null)
assert(leaf.base, true)
assert(leaf.middle, true)
assert(leaf.leaf, true)
assert(middle.reset(), null)
assert(middle.base, true)
assert(Base.length, 0)
assert(Middle.length, 1)
assert(Leaf.length, 1)
"#,
        11,
    );
}

#[test]
fn a_receiver_without_a_parent_keeps_null_and_existing_operator_behavior() {
    check_forms(
        r#"
class Root:
    def init():
        this.result = super()
    def bare() => super()
    def grouped() => (super())
    def member() => super().amount
    def indexed() => super()[0]
    def postfix() => (super())()
    def negated() => !super()
    def compared() => super() == null
    def subtracted() => super() - 1
    def chosen() => super() or 7
root = Root()
assert(root.result, null)
assert(root.bare(), null)
assert(root.grouped(), null)
assert(root.member(), null)
assert(root.indexed(), null)
assert(root.postfix(), null)
assert(root.negated(), true)
assert(root.compared(), true)
assert(root.subtracted(), -1)
assert(root.chosen(), 7)
assert(Root.length, 1)
"#,
        11,
    );
}

#[test]
fn explicit_arguments_still_evaluate_in_order_including_without_a_parent() {
    check_forms(
        r#"
events = {trace: ""}
def mark(label):
    events.trace = events.trace + label
    return label
class Parent:
    def init(first, second):
        this.first = first
        this.second = second
class Child: Parent
    def init():
        super(mark("A"), mark("B"))
class Root:
    def init():
        this.result = super(mark("C"), mark("D"))
child = Child()
root = Root()
assert(child.first, "A")
assert(child.second, "B")
assert(events.trace, "ABCD")
assert(root.result, null)
"#,
        4,
    );
}

#[test]
fn missing_receivers_and_argument_errors_keep_the_existing_diagnostics() {
    for (source, line) in [
        ("super", 1),
        ("saved = 7\nsuper", 2),
        ("def probe() => super\nprobe()", 2),
        (
            "def probe() => super\nclass Root:\n    def run() => probe()\nroot = Root()\nroot.run()",
            5,
        ),
    ] {
        let bare = Runtime::new().run(source).unwrap_err();
        let explicit = Runtime::new()
            .run(&source.replace("super", "super()"))
            .unwrap_err();
        assert_eq!(bare, explicit, "{source}");
        assert_eq!(bare.kind(), ErrorKind::Reference);
        assert_eq!(bare.message(), "super is not defined");
        assert_eq!(bare.position(), Some(Position::new(line, 1)));
    }

    // Explicit arguments are evaluated before the receiver is checked.
    let error = Runtime::new().run("super(missing)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn failed_parent_calls_roll_back_properties_dependencies_and_populations() {
    check_forms(
        r#"
kept = {amount: 5}
doubled = kept.amount * 2
class Parent:
    def init():
        kept.amount = 9
        this.amount = 7
        throw "BLOCKED"
class Child: Parent
    def init() => super()
try:
    failed = Child()
catch error:
    assert(error, "BLOCKED")
assert(kept.amount, 5)
assert(doubled, 10)
assert(Parent.length, 0)
assert(Child.length, 0)
try:
    failed
catch error:
    assert(error, ReferenceError("failed is not defined"))
kept.amount = 6
assert(doubled, 12)
"#,
        7,
    );
}

#[test]
fn inherited_rules_and_graph_source_stay_canonical_for_both_spellings() {
    let explicit = r#"
seed = 4
class Parent:
    def init():
        this.ready = true
$Parent.amount = seed
class Child: Parent
    def init() => super()
child = Child()
enabled = false
if enabled and super() == null:
    flag = 1
assert(child.amount, 4)
seed = 6
assert(child.amount, 6)
"#;
    let bare = explicit.replace("super()", "super");
    let mut bare_runtime = Runtime::new();
    let mut explicit_runtime = Runtime::new();
    for (runtime, source) in [
        (&mut bare_runtime, bare.as_str()),
        (&mut explicit_runtime, explicit),
    ] {
        runtime.run(source).unwrap();
        assert_eq!(runtime.assertions_run(), 2);
        let failures = runtime.take_assertions();
        assert!(failures.is_empty(), "{failures:?}");
        assert!(
            runtime
                .graph
                .keys()
                .any(|key| key.as_str().contains("super()")),
            "the standing condition must keep canonical super() source"
        );
    }
    assert!(bare_runtime.graph.keys().eq(explicit_runtime.graph.keys()));
    assert_eq!(
        NodeKey::block(&parse("{ super }").unwrap(), None),
        NodeKey::block(&parse("{ super() }").unwrap(), None)
    );
    let bare_trace = bare_runtime.run("model |> why").unwrap();
    let explicit_trace = explicit_runtime.run("model |> why").unwrap();
    assert_eq!(
        bare_runtime.serialize_json(&bare_trace).unwrap(),
        explicit_runtime.serialize_json(&explicit_trace).unwrap()
    );
    // Short-circuiting kept this top-level call dormant. Triggering it must
    // still report the missing receiver and roll back the variable update.
    for runtime in [&mut bare_runtime, &mut explicit_runtime] {
        let error = runtime.run("enabled = true").unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "super is not defined");
        assert_eq!(runtime.run("enabled").unwrap(), Value::Bool(false));
    }
}

#[test]
fn cli_runs_bare_and_argument_bearing_super_calls_in_a_file() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("bare-super.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().next(), Some("6.0"));
    assert!(stdout.contains("$Base"), "{stdout}");
    assert!(stdout.contains("leaf.amount"), "{stdout}");
}

#[test]
fn cli_reports_a_bare_call_without_a_receiver_at_its_statement() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("bare-super-no-receiver.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg(fixture)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        "ReferenceError: super is not defined (line 2, column 1)"
    );
}

#[test]
fn repl_keeps_existing_multiline_class_entry_and_executes_bare_super() {
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
            b"class Parent:\n    def init():\n        this.ready = true\n\nclass Child: Parent\n    def init() => super\n    def reset() => super\n\nchild = Child()\nassert(child.ready, true)\nassert(child.reset(), null)\n.json on\nChild.length\n.exit\n",
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("json on."), "{stdout}");
    assert!(stdout.lines().any(|line| line == "1.0"), "{stdout}");
}
