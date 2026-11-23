mod common;

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Expr, parse, parse_expression};
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};

fn checked(source: &str) -> Runtime {
    let expected = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let mut runtime = Runtime::new();
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}\n{error}"));
    assert_eq!(runtime.assertions_run(), expected, "{source}");
    let failures = runtime.take_assertions();
    assert!(failures.is_empty(), "{source}\n{failures:?}");
    runtime
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

fn result(runtime: &mut Runtime, source: &str) -> Result<String, String> {
    match runtime.run(source) {
        Ok(value) => Ok(runtime.serialize_json(&value).unwrap()),
        Err(error) => Err(error.to_string()),
    }
}

fn same_runs(unknown: &str, written: &str, tail: &str) -> Result<String, String> {
    let mut unknown_runtime = Runtime::new();
    let mut written_runtime = Runtime::new();
    assert_eq!(
        result(&mut unknown_runtime, unknown),
        result(&mut written_runtime, written),
        "{unknown}"
    );
    let unknown_result = result(&mut unknown_runtime, tail);
    let written_result = result(&mut written_runtime, tail);
    assert_eq!(unknown_result, written_result, "{unknown}\n{tail}");
    assert_eq!(
        graph_shape(&unknown_runtime),
        graph_shape(&written_runtime),
        "{unknown}\n{tail}"
    );
    unknown_result
}

fn run_repl(source: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(source).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn the_requested_shape_builds_the_instance_of_its_written_out_class() {
    assert_eq!(
        same_runs(
            "class Shape(type):\n    this.type = type",
            "class Shape(type, color):\n    this.type = type\n    this.color = color",
            "shape1 = Shape(type='SQUARE', color='RED')\nshape1",
        ),
        Ok(r#"{"color":"RED","id":"shape1","type":"SQUARE"}"#.to_string())
    );
    checked(
        r#"
class Shape(type):
    this.type = type
shape1 = Shape(type='SQUARE', color='RED')
assert(shape1, {"id": "shape1", "type": "SQUARE", "color": "RED"})
assert([shape1.type, shape1.color, Shape.length], ["SQUARE", "RED", 1])
"#,
    );
}

#[test]
fn every_class_header_form_takes_unknown_labels() {
    for (unknown, written, tail, expected) in [
        (
            "class Shape",
            "class Shape(color):\n    this.color = color",
            "s = Shape(color=\"RED\")\n[s.color, Shape.length]",
            r#"["RED",1.0]"#,
        ),
        (
            "class Shape()",
            "class Shape(color):\n    this.color = color",
            "s = Shape(color=\"RED\")\n[s.color, Shape.length]",
            r#"["RED",1.0]"#,
        ),
        (
            "class Shape(type: str)",
            "class Shape(type: str, color):\n    this.color = color",
            "s = Shape(type=\"SQUARE\", color=\"RED\")\n[s.type, s.color]",
            r#"[null,"RED"]"#,
        ),
        (
            "class Shape(type):\n    pass",
            "class Shape(type, color):\n    this.color = color",
            "s = Shape(\"SQUARE\", color=\"RED\")\n[s.type, s.color]",
            r#"[null,"RED"]"#,
        ),
        (
            "class Base(type):\n    this.type = type\nclass Square: Base\n    pass",
            "class Base(type, color):\n    this.type = type\n    this.color = color\nclass Square: Base\n    pass",
            "s = Square(type=\"SQUARE\", color=\"RED\")\n[s.type, s.color, Square.length, Base.length]",
            r#"["SQUARE","RED",1.0,0.0]"#,
        ),
        (
            "class Base(type):\n    this.type = type\nclass Square extends Base",
            "class Base(type, color):\n    this.type = type\n    this.color = color\nclass Square extends Base",
            "s = Square(type=\"SQUARE\", color=\"RED\")\n[s.type, s.color]",
            r#"["SQUARE","RED"]"#,
        ),
        (
            "class Base(type):\n    this.type = type\nclass Square(side): Base\n    pass",
            "class Base(type, side):\n    this.type = type\n    this.side = side\nclass Square(side): Base\n    pass",
            "s = Square(type=\"SQUARE\", side=2)\n[s.type, s.side]",
            r#"["SQUARE",2.0]"#,
        ),
        (
            "class Base(type):\n    this.type = type\nclass Square(side) extends Base:\n    this.side = side",
            "class Base(type):\n    this.type = type\nclass Square(side, color) extends Base:\n    this.side = side\n    this.color = color",
            "s = Square(side=2, color=\"RED\")\n[s.type, s.side, s.color]",
            r#"[null,2.0,"RED"]"#,
        ),
        (
            "class Base\nclass Square: Base\n    pass",
            "class Base(color):\n    this.color = color\nclass Square: Base\n    pass",
            "s = Square(color=\"RED\")\n[s.color, Square.length]",
            r#"["RED",1.0]"#,
        ),
    ] {
        assert_eq!(
            same_runs(unknown, written, tail),
            Ok(expected.to_string()),
            "{unknown}"
        );
    }
}

#[test]
fn adopted_initializers_take_unknown_labels() {
    for (unknown, written) in [
        (
            "class Point:\n    def init(x):\n        this.x = x",
            "class Point:\n    def init(x, y):\n        this.x = x\n        this.y = y",
        ),
        (
            "class Point:\n    def __init__(self, x):\n        this.x = x",
            "class Point:\n    def __init__(self, x, y):\n        this.x = x\n        this.y = y",
        ),
        (
            "class Point:\n    def init(self.x):\n        pass",
            "class Point:\n    def init(self.x, y):\n        this.y = y",
        ),
        (
            "class Point(ignored):\n    def init(x):\n        this.x = x",
            "class Point(ignored):\n    def init(x, y):\n        this.x = x\n        this.y = y",
        ),
    ] {
        assert_eq!(
            same_runs(unknown, written, "p = Point(x=1, y=2)\n[p.x, p.y]"),
            Ok("[1.0,2.0]".to_string()),
            "{unknown}"
        );
    }
    checked(
        r#"
class Adopted(ignored):
    def init(actual):
        this.actual = actual
adopted = Adopted(actual=1, ignored=2)
assert(adopted, {"id": "adopted", "actual": 1, "ignored": 2})
"#,
    );
}

#[test]
fn initializing_parameters_defaults_and_rest_formals_keep_their_binding() {
    checked(
        r#"
class P(this.x, this.y = 7, ...this.rest)
first = P(z=1)
second = P(1, 2, 3, 4, z=5)
assert(first, {"id": "first", "x": null, "y": 7, "rest": [], "z": 1})
assert(second, {"id": "second", "x": 1, "y": 2, "rest": [3, 4], "z": 5})
class Pair(first = 1, second = 2):
    this.first = first
    this.second = second
pair = Pair(second=4, third=3)
assert(pair, {"id": "pair", "first": 1, "second": 4, "third": 3})
class Packet(head = 1, ...tail):
    this.tail = tail
packet = Packet(1, 2, 3, kind="x")
assert(packet, {"id": "packet", "tail": [2, 3], "kind": "x"})
"#,
    );
}

#[test]
fn every_construction_route_takes_unknown_labels() {
    checked(
        r#"
class Tag(name):
    this.name = name
holder = {}
first = Tag(name="a", color="RED")
second = new Tag(name="b", color="GREEN")
holder.tag = Tag(name="c", color="BLUE")
items = [Tag(name="d", color="GREY")]
record = {tag: Tag(name="e", color="PINK")}
assert(first.color, "RED")
assert(second.color, "GREEN")
assert(holder.tag.color, "BLUE")
assert(items[0].color, "GREY")
assert(record.tag.color, "PINK")
assert(Tag(name="f", color="TEAL").color, "TEAL")
class Holder
$Holder.shared = Tag(name="g", color="GOLD")
one = Holder()
two = Holder()
assert(one.shared.color, "GOLD")
assert(one.shared, two.shared)
assert(Tag.length, 7)
"#,
    );
    checked(
        r#"
class Light(color):
    this.color is color
first is Light(colour=GREY)
assert(first, {"id": "first", "color": null, "colour": "GREY"})
GREY = "grey"
assert(first.colour, "GREY")
lights is [Light(colour=GREY)]
assert(lights[0].colour, "grey")
GREY = "silver"
assert(lights[0].colour, "silver")
assert(first.colour, "GREY")
"#,
    );
}

#[test]
fn super_assigns_unknown_labels_on_the_same_receiver_after_the_parent_statements() {
    let unknown = "class Base(level):\n    this.level = level\n    this.color = \"BASE\"\nclass Child(level): Base\n    super(level, color=\"RED\")\n    this.tag = color";
    let written = "class Base(level, color):\n    this.level = level\n    this.color = \"BASE\"\n    this.color = color\nclass Child(level): Base\n    super(level, color=\"RED\")\n    this.tag = color";
    assert_eq!(
        same_runs(
            &format!("color = \"GLOBAL\"\n{unknown}"),
            &format!("color = \"GLOBAL\"\n{written}"),
            "child = Child(2)\n[child.level, child.color, child.tag, Child.length, Base.length]"
        ),
        Ok(r#"[2.0,"RED","GLOBAL",1.0,0.0]"#.to_string())
    );
    checked(
        r#"
class Parent(a):
    this.a = a
class Child: Parent
    super(a=1, color="RED")
class Repainted: Parent
    def paint(c) => super(a=2, color=c)
child = Child(color="BLUE")
assert(child, {"id": "child", "a": 1, "color": "BLUE"})
repainted = Repainted(a=0)
assert(repainted.paint("GREEN"), null)
assert(repainted, {"id": "repainted", "a": 2, "color": "GREEN"})
assert(Parent.length, 0)
class Base(level):
    this.level = level
    return
class Glazed(level): Base
    super(level, color="RED")
    return
glazed = Glazed(2, glaze="MATTE")
assert(glazed, {"id": "glazed", "level": 2, "color": "RED", "glaze": "MATTE"})
"#,
    );
}

#[test]
fn unknown_labels_are_assigned_after_the_constructor_statements() {
    checked(
        r#"
'use imperative'
color = "GLOBAL"
class Shape(type):
    this.type = type
    this.size = 1
    this.label = color
plain = Shape("CIRCLE")
assert(plain, {"id": "plain", "type": "CIRCLE", "size": 1, "label": "GLOBAL"})
shape = Shape(type="SQUARE", color="RED", size=2)
assert(shape, {"id": "shape", "type": "SQUARE", "size": 2, "label": "RED", "color": "RED"})
keys = []
for key, value of shape:
    keys.push(key)
assert(keys, ["id", "type", "size", "label", "color"])
assert(color, "GLOBAL")
class Early(a):
    this.a = a
    if a:
        return
    this.after = true
early = Early(1, extra=2)
late = Early(0, extra=3)
assert(early, {"id": "early", "a": 1, "extra": 2})
assert(late, {"id": "late", "a": 0, "after": true, "extra": 3})
class Arrow:
    def init(a) => this.a = a
arrow = Arrow(a=1, b=2)
assert(arrow, {"id": "arrow", "a": 1, "b": 2})
class Point:
    def init(this.x) => 5
point = Point(x=1, y=2)
assert(point, {"id": "point", "x": 1, "y": 2})
"#,
    );
    let tail = "arrow = Arrow(a=1, b=2)\narrow";
    let mut literal = Runtime::new();
    result(
        &mut literal,
        "class Arrow:\n    def init(a, b):\n        return this.a = a\n        this.b = b",
    )
    .unwrap();
    assert_eq!(
        result(&mut literal, tail),
        Ok(r#"{"a":1.0,"id":"arrow"}"#.to_string())
    );
    let mut unknown = Runtime::new();
    result(&mut unknown, "class Arrow:\n    def init(a) => this.a = a").unwrap();
    assert_eq!(
        result(&mut unknown, tail),
        Ok(r#"{"a":1.0,"b":2.0,"id":"arrow"}"#.to_string())
    );
}

#[test]
fn positional_values_never_fill_unknown_labels_and_surplus_values_still_run() {
    checked(
        r#"
'use imperative'
events = {order: ""}
def mark(label):
    events.order = events.order + label
    return label
class Pair(a):
    this.a = a
pair = Pair(mark("A"), mark("S"), b=mark("B"))
assert(pair, {"id": "pair", "a": "A", "b": "B"})
assert(events.order, "ASB")
class Empty
empty = Empty(mark("X"), b=mark("Y"))
assert(empty, {"id": "empty", "b": "Y"})
assert(events.order, "ASBXY")
"#,
    );
}

#[test]
fn every_binding_is_still_validated_before_any_value_runs() {
    let mut runtime = checked(
        r#"
events = {count: 0}
class Pair(a):
    this.a = a
class Packet(...tail):
    this.tail = tail
class Child: Pair
    pass
"#,
    );
    for (call, message) in [
        (
            "Pair(color=(events.count=9), color=missing)",
            "Duplicate constructor argument 'color' for Pair",
        ),
        (
            "Pair(color=missing, a=(events.count=9), color=1)",
            "Duplicate constructor argument 'color' for Pair",
        ),
        (
            "Pair(color=(events.count=9), missing.value)",
            "Positional constructor argument after named arguments for Pair",
        ),
        (
            "Packet(color=(events.count=9), tail=missing)",
            "Rest parameter 'tail' cannot be a named constructor argument for Packet",
        ),
        (
            "Child(color=missing, color=(events.count=9))",
            "Duplicate constructor argument 'color' for Pair",
        ),
    ] {
        let error = runtime.run(&format!("failed = {call}")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{call}");
        assert_eq!(error.message(), message, "{call}");
        assert_eq!(error.position(), Some(Position::new(1, 1)), "{call}");
        assert_eq!(runtime.run("events.count").unwrap(), Value::Number(0.0));
        assert_eq!(runtime.run("Pair.length").unwrap(), Value::Number(0.0));
        assert!(!runtime.state.has_object(&ObjectId::named("failed")));
        assert!(!runtime.state.has_variable("failed"));
        assert!(!runtime.graph.contains(&NodeKey::variable("color")));
    }
    checked(
        r#"
events = {count: 0}
class Parent(a):
    this.a = a
class Child: Parent
    def invalid() => super(color=missing, color=(events.count=9))
child = Child(a=1)
try:
    child.invalid()
catch error:
    assert(error, TypeError("Duplicate constructor argument 'color' for Parent"))
assert(events.count, 0)
assert(child, {"id": "child", "a": 1})
"#,
    );
}

#[test]
fn values_run_once_in_source_order_in_the_callers_scope_and_stay_snapshots() {
    let runtime = checked(
        r#"
'use imperative'
class Shape
events = {count: 0, order: ""}
def mark(label):
    events.count = events.count + 1
    events.order = events.order + label
    return events.count
shape = Shape(second=mark("B"), first=mark("A"))
assert(shape, {"id": "shape", "second": 1, "first": 2})
assert(events.order, "BA")
def create(color):
    result = Shape(color=color + "!", shade=color.value)
    assert(color, "RED")
    return result
local = create("RED")
assert([local.color, local.shade], ["RED!", "RED"])
"#,
    );
    assert!(!runtime.graph.contains(&NodeKey::variable("second")));
    assert!(!runtime.graph.contains(&NodeKey::variable("first")));
    assert!(!runtime.graph.contains(&NodeKey::variable("shade")));
    let runtime = checked(
        r#"
class Shape
c = "RED"
named = Shape(color=c)
frozen = Shape(color=c.value)
c = "GREEN"
assert(named.color, "RED")
assert(frozen.color, "RED")
assert(Shape.length, 2)
"#,
    );
    assert!(!runtime.graph.contains(&NodeKey::variable("color")));
}

#[test]
fn standing_declarations_rebuild_and_class_rules_still_apply_after_construction() {
    checked(
        r#"
class Rect
$Rect.area = $Rect.width * $Rect.height
rect = Rect(width=2, height=3)
assert(rect.area, 6)
rect.width = 4
assert(rect.area, 12)
c = "RED"
shapes = [Rect(color=c)]
assert(shapes[0].color, "RED")
c = "GREEN"
assert(shapes[0].color, "GREEN")
assert(Rect.length, 3)
"#,
    );
    assert_eq!(
        same_runs(
            "class Shape",
            "class Shape(color):\n    this.color = color",
            "$Shape.color = \"BLUE\"\nfirst = Shape(color=\"RED\")\n$Shape.size = 1\nsecond = Shape(color=\"GREEN\", size=2)\n[first.color, second.color, first.size, second.size]"
        ),
        Ok(r#"["BLUE","BLUE",1.0,1.0]"#.to_string())
    );
}

#[test]
fn failures_roll_back_as_their_written_out_spellings() {
    for (unknown, written, tail) in [
        (
            "class Shape",
            "class Shape(color):\n    this.color = color",
            "s = Shape(color=missing)",
        ),
        (
            "class Shape:\n    throw \"no\"",
            "class Shape(color):\n    throw \"no\"\n    this.color = color",
            "s = Shape(color=\"RED\")",
        ),
        (
            "class Shape",
            "class Shape(value):\n    this.value = value",
            "s = Shape(value=1)",
        ),
        (
            "class Shape(type):\n    this.type = type",
            "class Shape(type, value):\n    this.type = type\n    this.value = value",
            "s = Shape(type=\"SQUARE\", value=1)",
        ),
    ] {
        let mut unknown_runtime = Runtime::new();
        let mut written_runtime = Runtime::new();
        result(&mut unknown_runtime, unknown).unwrap();
        result(&mut written_runtime, written).unwrap();
        let before = graph_shape(&unknown_runtime);
        let failure = result(&mut unknown_runtime, tail).unwrap_err();
        assert_eq!(result(&mut written_runtime, tail), Err(failure.clone()));
        assert!(failure.ends_with("(line 1, column 1)"), "{failure}");
        assert_eq!(graph_shape(&unknown_runtime), before, "{tail}");
        assert_eq!(graph_shape(&written_runtime), before, "{tail}");
        for runtime in [&mut unknown_runtime, &mut written_runtime] {
            assert_eq!(result(runtime, "Shape.length"), Ok("0.0".to_string()));
            assert!(!runtime.state.has_object(&ObjectId::named("s")));
        }
    }
    let runtime = checked(
        r#"
class Shape(type):
    this.type = type
reason = null
try:
    Shape(type="SQUARE", value=1)
catch error:
    reason = error
assert(reason, TypeError("Cannot use 'value' as a property"))
try:
    s = Shape(type="SQUARE", color=missing)
catch error:
    reason = error
assert(reason, ReferenceError("missing is not defined"))
assert(Shape.length, 0)
seen = suppose s = Shape(type="SQUARE", color="RED"): [s.color, Shape.length]
assert(seen, ["RED", 1])
assert(Shape.length, 0)
"#,
    );
    assert!(!runtime.state.has_object(&ObjectId::named("s")));
}

#[test]
fn a_validated_plan_stays_pinned_through_redeclaration_during_values() {
    checked(
        r#"
class Named(a):
    this.original = a
def replace():
    class Named(a, b):
        this.replacement = a
    return 7
first = Named(a=1, b=replace())
second = Named(2, 3)
assert(first, {"id": "first", "original": 1, "b": 7})
assert(second, {"id": "second", "replacement": 2})
"#,
    );
}

#[test]
fn labels_named_id_or_like_class_rules_follow_the_written_out_spelling() {
    assert_eq!(
        same_runs(
            "class Shape(type):\n    this.type = type",
            "class Shape(type, id):\n    this.type = type\n    this.id = id",
            "shape1 = Shape(type=\"SQUARE\", id=5)\n[shape1.id, shape1.type, Shape.length]"
        ),
        Ok(r#"[5.0,"SQUARE",1.0]"#.to_string())
    );
    assert_eq!(
        same_runs(
            "class Shape",
            "class Shape(width, area):\n    this.width = width\n    this.area = area",
            "$Shape.area = $Shape.width * 2\nshape = Shape(width=3, area=1)\n[shape.area, shape.width]"
        ),
        Ok("[6.0,3.0]".to_string())
    );
}

#[test]
fn class_rule_checks_accept_unknown_labels_and_keep_every_other_check() {
    checked(
        r#"
class Tag
class Holder
def install(color):
    $Holder.tag = Tag(color=color)
install("RED")
holder = Holder()
assert(holder.tag.color, "RED")
try:
    $Holder.other = Tag(color=missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
try:
    $Holder.twice = Tag(color=1, color=2)
catch error:
    assert(error, TypeError("Duplicate constructor argument 'color' for Tag"))
assert(Tag.length, 1)
"#,
    );
}

#[test]
fn functions_methods_built_ins_and_direct_init_keep_assignment_arguments() {
    checked(
        r#"
'use imperative'
def pair(a, b):
    return [a, b]
color = 0
result = pair(color=2, shade=1)
assert(result, [2, 1])
assert(color, 2)
assert(shade, 1)
class Point:
    def init(x):
        this.x = x
point = Point(x=1)
point.init(tone=5)
assert(point, {"id": "point", "x": 5})
assert(tone, 5)
items = List(width=3)
assert(items, [3])
assert(width, 3)
"#,
    );
}

#[test]
fn calls_parse_and_render_as_before() {
    let call = parse_expression("Shape(type='SQUARE', color='RED')").unwrap();
    let Expr::Call { arguments, .. } = &call else {
        panic!("expected the unchanged call AST");
    };
    assert!(
        arguments
            .iter()
            .all(|argument| matches!(argument, Expr::Assign { .. }))
    );
    assert_eq!(call.to_string(), r#"Shape(type="SQUARE",color="RED")"#);
    assert_eq!(call, parse_expression(&call.to_string()).unwrap());
}

#[test]
fn explanations_and_graphs_show_the_written_out_assignment() {
    let mut runtime = checked(
        r#"
class Shape(type):
    this.type = type
c = "RED"
shape1 = Shape(type="SQUARE", color=c)
$Shape.label = $Shape.type + "/" + $Shape.color
assert(shape1.label, "SQUARE/RED")
"#,
    );
    let why = runtime.run("shape1.color |> why").unwrap();
    let why = runtime.serialize_json(&why).unwrap();
    assert_eq!(
        why,
        r#"[{"from":[],"holds":"RED","node":"shape1.color","rule":"$Shape.color = color","state":"derived"}]"#
    );
    let shape = graph_shape(&runtime);
    assert!(
        shape.contains(&"shape1.color [property] [] [\"shape1.label\"]".to_string()),
        "{shape:?}"
    );
    assert!(!runtime.graph.contains(&NodeKey::variable("color")));
}

#[test]
fn the_binary_builds_instances_from_unknown_labels() {
    let output = run_repl(
        b"class Shape(type):\n    this.type = type\n\nshape1 = Shape(type='SQUARE', color='RED')\n.json on\nshape1\n.exit\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let value: serde_json::Value =
        serde_json::from_str(stdout.lines().next_back().unwrap()).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"id": "shape1", "type": "SQUARE", "color": "RED"}),
        "{stdout}"
    );
}
