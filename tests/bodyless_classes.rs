//! Empty class headers use the existing class AST and execution path.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{Stmt, parse, parse_program};
use nucleoid::{ErrorKind, Position, Runtime};

fn equivalent(compact: &str, expanded: &str) {
    let compact_ast = parse(compact).unwrap();
    let expanded_ast = parse(expanded).unwrap();
    assert_eq!(compact_ast, expanded_ast, "{compact}");
    assert_eq!(
        generate_all(&compact_ast),
        generate_all(&expanded_ast),
        "{compact}"
    );
}

#[test]
fn bodyless_headers_lower_to_the_existing_empty_class_ast() {
    for header in [
        "class Entity",
        "class Entity()",
        "class Shell(code: int)",
        "class Shell(code: int, label: str, raw)",
        "class Shell(\n    code: int,\n    label: str,\n    raw,\n)",
    ] {
        equivalent(header, &format!("{header}:\n    pass"));

        let statements = parse(header).unwrap();
        let [Stmt::Class(class)] = statements.as_slice() else {
            panic!("expected one class declaration");
        };
        assert!(class.parent.is_none());
        assert!(class.fields.is_empty());
        assert!(class.constructor.is_empty());
        assert!(class.methods.is_empty());
        assert_eq!(generate_all(&statements), format!("class {}", class.name));
    }
}

#[test]
fn statement_boundaries_keep_following_statements_outside_the_class() {
    for (compact, expanded) in [
        (
            "class Entity\nnext = 1",
            "class Entity:\n    pass\nnext = 1",
        ),
        (
            "class Entity # empty\n\n# next statement\nnext = 1",
            "class Entity:\n    pass\nnext = 1",
        ),
        (
            "class Entity; next = 1",
            "class Entity:\n    pass\nnext = 1",
        ),
        ("{ class Entity }", "{\n    class Entity:\n        pass\n}"),
        (
            "{ class Entity; next = 1 }",
            "{\n    class Entity:\n        pass\n    next = 1\n}",
        ),
        (
            "if active:\n    class Entity",
            "if active:\n    class Entity:\n        pass",
        ),
        (
            "if active:\n    class Entity\nelse:\n    next = 1",
            "if active:\n    class Entity:\n        pass\nelse:\n    next = 1",
        ),
        (
            "if active: class Entity; next = 1",
            "if active:\n    class Entity:\n        pass\n    next = 1",
        ),
        (
            "for item of items:\n    class Entity\nnext = 1",
            "for item of items:\n    class Entity:\n        pass\nnext = 1",
        ),
        (
            "try:\n    class Entity\ncatch error:\n    pass\nnext = 1",
            "try:\n    class Entity:\n        pass\ncatch error:\n    pass\nnext = 1",
        ),
        (
            "def setup():\n    class Entity\n    return 1",
            "def setup():\n    class Entity:\n        pass\n    return 1",
        ),
        (
            "class Outer:\n    class Inner\n    this.count = 1",
            "class Outer:\n    class Inner:\n        pass\n    this.count = 1",
        ),
    ] {
        equivalent(compact, expanded);
    }
}

#[test]
fn bodyless_headers_preserve_top_level_positions() {
    let source = "class Entity\nnext = missing";
    let program = parse_program(source).unwrap();
    assert_eq!(
        program.positions,
        [Position::new(1, 1), Position::new(2, 1)]
    );

    let error = Runtime::new().run(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
}

#[test]
fn malformed_headers_keep_exact_existing_syntax_diagnostics() {
    for (source, message) in [
        ("class", "Expected a name but found end of input"),
        ("class 123", "Expected a name but found number 123"),
        ("class Entity =", "Expected ':' but found '='"),
        (
            "class Entity other",
            "Expected ':' but found identifier 'other'",
        ),
        ("class Entity {}", "Expected ':' but found '{'"),
        ("class Entity => 1", "Expected ':' but found '=>'"),
        ("class Entity.code", "Expected ':' but found '.'"),
        ("class Entity)", "Expected ':' but found ')'"),
        ("class Entity]", "Expected ':' but found ']'"),
        ("class Entity, next", "Expected ':' but found ','"),
        ("class Entity(code:)", "Expected a type but found ')'"),
        ("class Entity(1)", "Expected a parameter but found number 1"),
        ("class Entity(code", "Expected ')' but found end of input"),
        (
            "class Entity(code = [])",
            "Expected a scalar literal parameter default",
        ),
        (
            "class Entity\n    this.code = 1",
            "Expected ':' but found end of line",
        ),
        (
            "class Entity\n\n    # still an indented body\n    pass",
            "Expected ':' but found end of line",
        ),
        (
            "class Entity;\n    this.code = 1",
            "Expected ':' but found ';'",
        ),
        ("class Entity; ;\n    pass", "Expected ':' but found ';'"),
        (
            "{\n    class Entity\n        code: int\n}",
            "Expected ':' but found end of line",
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert!(error.position().is_some(), "{source}");
    }

    let error = Runtime::check("class Entity\n    this.code = 1").unwrap_err();
    assert_eq!(error.position(), Some(Position::new(1, 7)));
    let error = Runtime::check("class Entity = 1").unwrap_err();
    assert_eq!(error.position(), Some(Position::new(1, 14)));
}

#[test]
fn malformed_headers_do_not_apply_earlier_statements_or_create_classes() {
    let mut runtime = Runtime::new();
    runtime.run("saved = 7").unwrap();

    for invalid in [
        "class Entity other",
        "class Entity\n    this.code = 1",
        "class Entity;\n    pass",
    ] {
        let error = runtime.run(&format!("saved = 3\n{invalid}")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(runtime.run("saved").unwrap().to_string(), "7");
        assert_eq!(runtime.run("Class.length").unwrap().to_string(), "0");
    }
}

#[test]
fn optional_parameters_do_not_implicitly_create_properties() {
    let mut runtime = Runtime::new();
    runtime
        .run(
            r#"
class Shell(code: int, label: str, raw)
first = Shell(5, "A", 9)
second = Shell()
assert(first, {id: "first"})
assert(first.code, null)
assert(first.label, null)
assert(first.raw, null)
assert(second.code, null)
assert(Shell.length, 2)
{
    class Token
    token1 = Token()
    assert(Token[0], token1)
}
assert(Token.length, 1)
assert(Class.length, 2)
"#,
        )
        .unwrap();
    assert_eq!(runtime.assertions_run(), 9);
    assert!(runtime.take_assertions().is_empty());
}

#[test]
fn explicit_fields_constructors_methods_and_inheritance_remain_unchanged() {
    let mut runtime = Runtime::new();
    runtime
        .run(
            r#"
class Parent(code: int):
    this.code = code
    def reading() => self.code
class Child: Parent
    label: str
    def init(code: int):
        super(code)
        this.label = "child"
    def reading() => self.code
parent1 = Parent(4)
child1 = Child(7)
$Parent.enabled = true
assert(parent1.code, 4)
assert(parent1.reading(), 4)
assert(child1.code, 7)
assert(child1.label, "child")
assert(child1.reading(), 7)
assert(parent1.enabled, true)
assert(child1.enabled, true)
assert(Parent.length, 1)
assert(Child.length, 1)
"#,
        )
        .unwrap();
    assert_eq!(runtime.assertions_run(), 9);
    let failures = runtime.take_assertions();
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
fn bodyless_and_explicit_classes_keep_identical_graphs_returns_and_rollback() {
    let mut compact = Runtime::new();
    let mut expanded = Runtime::new();
    for (compact_source, expanded_source) in [
        ("class Item", "class Item:\n    pass"),
        (
            "rate = 2\n$Item.price = $Item.quantity * rate",
            "rate = 2\n$Item.price = $Item.quantity * rate",
        ),
        ("item1 = Item()", "item1 = Item()"),
        ("item1.quantity = 3", "item1.quantity = 3"),
        ("class Item", "class Item:\n    pass"),
        ("item2 = Item()", "item2 = Item()"),
        (
            "item2.quantity = 4\nrate = 3",
            "item2.quantity = 4\nrate = 3",
        ),
        ("class Child: Item\n    pass", "class Child: Item\n    pass"),
        ("child1 = Child()", "child1 = Child()"),
        ("child1.quantity = 5", "child1.quantity = 5"),
        (
            "count = Item.length\nif count > 2:\n    throw \"LIMIT\"",
            "count = Item.length\nif count > 2:\n    throw \"LIMIT\"",
        ),
        ("item3 = Item()", "item3 = Item()"),
        ("item3", "item3"),
        ("Item.length", "Item.length"),
        ("Child.length", "Child.length"),
        ("count", "count"),
        ("why item1.price", "why item1.price"),
        ("model |> why", "model |> why"),
    ] {
        match (compact.run(compact_source), expanded.run(expanded_source)) {
            (Ok(compact_value), Ok(expanded_value)) => assert_eq!(
                compact.serialize_json(&compact_value).unwrap(),
                expanded.serialize_json(&expanded_value).unwrap(),
                "{compact_source}"
            ),
            (Err(compact_error), Err(expanded_error)) => {
                assert_eq!(compact_error, expanded_error, "{compact_source}");
                match compact_source {
                    "item3 = Item()" => {
                        assert_eq!(compact_error.kind(), ErrorKind::Thrown);
                        assert_eq!(compact_error.message(), "LIMIT");
                    }
                    "item3" => {
                        assert_eq!(compact_error.kind(), ErrorKind::Reference);
                        assert_eq!(compact_error.message(), "item3 is not defined");
                    }
                    _ => panic!("unexpected error: {compact_error}"),
                }
            }
            (compact_result, expanded_result) => {
                panic!(
                    "different results for {compact_source}: {compact_result:?}, {expanded_result:?}"
                )
            }
        }
        assert!(
            compact.graph.keys().eq(expanded.graph.keys()),
            "{compact_source}"
        );
    }
    compact
        .run("assert(Item.length, 2)\nassert(Child.length, 1)\nassert(count, 2)\nassert(item1.price, 9)\nassert(item2.price, 12)\nassert(child1.price, 15)")
        .unwrap();
    assert_eq!(compact.assertions_run(), 6);
    assert!(compact.take_assertions().is_empty());
}

#[test]
fn bodyless_classes_still_obey_statement_and_expression_limits() {
    let allowed = format!("{}class Entity{}", "{".repeat(63), "}".repeat(63));
    assert!(Runtime::check(&allowed).is_ok());

    for depth in [64, 4096] {
        let source = format!("{}class Entity{}", "{".repeat(depth), "}".repeat(depth));
        let error = Runtime::check(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), "Statements are nested too deeply");
    }

    let source = format!("class Entity\nresult = {}1", "-".repeat(4096));
    assert_eq!(
        Runtime::check(&source).unwrap_err().message(),
        "Expressions are nested too deeply"
    );
    let source = format!("class Entity\nresult = 1{}", " + 1".repeat(4096));
    assert_eq!(
        Runtime::check(&source).unwrap_err().message(),
        "Expression has too many operands"
    );
}

#[test]
fn cli_runs_bodyless_classes_in_files() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("bodyless-classes.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().next(), Some("1.0"));
    assert!(stdout.contains("$Entity"), "{stdout}");
}

#[test]
fn repl_executes_bare_headers_without_waiting_for_a_body() {
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
            b"class Entity\n.json on\nclass Shell(code: int)\nfirst = Shell(5)\nassert(first.code, null)\nassert(Class.length, 2)\nassert(Shell.length, 1)\nShell.length\n.exit\n",
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("json on."), "{stdout}");
    assert!(stdout.lines().any(|line| line == "1.0"), "{stdout}");
}
