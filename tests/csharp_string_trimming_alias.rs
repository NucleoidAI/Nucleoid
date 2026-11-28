mod common;

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Position, Runtime, Value};

fn checked(runtime: &mut Runtime, source: &str, expected: &str) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "C# string trimming alias".to_string(),
            source: source.to_string(),
            expected: Some(expected.to_string()),
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    let executed = runtime.assertions_run() - before;
    assert_eq!(executed, count, "{source}");
    println!("Trim coverage: canonical={count}, executed={executed}, expected_return={expected}");
    value
}

#[test]
fn trim_preserves_unicode_boundaries_internal_text_and_legacy_spellings() {
    checked(
        &mut Runtime::new(),
        r#"
outer = String.fromCharCode(160, 8195, 9)
middle = "Zoë  😀\t x"
text = outer + middle + outer
assert([text.Trim(), text.trim(), text.strip()], [middle, middle, middle])
assert(["".Trim(), " \t\r\n".Trim(), outer.Trim()], ["", "", ""])
edge = String.fromCharCode(8203, 65279)
assert((edge + " x " + edge).Trim(), edge + " x " + edge)
record = {name: "  input  "}
items = ["  item\t"]
def supplied() => "  call\r"
assert([record.name.Trim(), items[0].Trim(), supplied().Trim().PadLeft(6, ".")], ["input", "item", "..call"])
text.Trim()
"#,
        r#""Zoë  😀\t x""#,
    );
}

#[test]
fn trim_tracks_standing_receiver_arguments_class_rules_and_written_names() {
    let mut runtime = Runtime::new();
    checked(
        &mut runtime,
        r#"
text = "  first  "
ignored = 1
clean = text.Trim(ignored)
legacy = text.trim(ignored)
assert([clean, legacy], ["first", "first"])
assert((why clean)[0].rule, "clean = text.Trim(ignored)")
assert((why legacy)[0].rule, "legacy = text.trim(ignored)")
class Label(text):
    this.text = text
$Label.clean = $Label.text.Trim()
label = Label("  own  ")
assert(label.clean, "own")
[clean, legacy, label.clean]
"#,
        r#"["first","first","own"]"#,
    );
    let dependencies: Vec<String> = runtime
        .graph
        .retrieve(&NodeKey::variable("clean"))
        .unwrap()
        .dependencies
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(dependencies, ["text", "ignored"]);
    for (source, rendered) in [
        ("clean = text.Trim(ignored)", "clean=text.Trim(ignored)"),
        (
            "$Label.clean = $Label.text.Trim()",
            "$Label.clean=$Label.text.Trim()",
        ),
    ] {
        assert_eq!(generate_all(&parse(source).unwrap()), rendered);
    }
    checked(
        &mut runtime,
        r#"
ignored = null
assert([clean, legacy], [null, null])
ignored = "x"
text = " \tsecond \n"
label.text = "  changed  "
assert([clean, legacy, label.clean], ["second", "second", "changed"])
[clean, legacy, label.clean]
"#,
        r#"["second","second","changed"]"#,
    );
    let error = runtime.run("text = 5").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.Trim is not a function");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
    checked(
        &mut runtime,
        r#"
assert([text, clean, legacy, label.clean], [" \tsecond \n", "second", "second", "changed"])
clean
"#,
        r#""second""#,
    );
}

#[test]
fn trim_evaluates_receiver_and_ignored_arguments_once_and_rolls_back_errors() {
    checked(
        &mut Runtime::new(),
        r#"
log = []
def receiver():
    log.push("receiver")
    return "  trimmed  "
def argument(value):
    log.push(value)
    return value
assert(receiver().Trim(argument("first"), argument("second")), "trimmed")
assert(log, ["receiver", "first", "second"])
log = []
assert(receiver().strip(argument("first"), argument("second")), "trimmed")
assert(log, ["receiver", "first", "second"])
log = []
failure = ""
try:
    receiver().Trim(argument("before"), missing, argument("after"))
catch error:
    failure = error
assert(failure, ReferenceError("missing is not defined"))
assert(log, [])
assert("  xoxo  ".Trim("x"), "xoxo")
log
"#,
        "[]",
    );
}

#[test]
fn trim_keeps_nullish_member_user_method_static_and_nonstring_routes() {
    checked(
        &mut Runtime::new(),
        r#"
assert("  alias  ".Trim(), "alias")
log = []
nullish = null
empty = {}
assert([nullish.Trim(log.push(1), missing), empty.absent.Trim(log.push(2), missing), empty.Trim(missing)], [undefined, undefined, undefined])
assert(log, [])
def Trim(value) => value + 1
record = {Trim: value -> value + 2}
class Marker:
    def Trim(value) => value + 3
marker = Marker()
assert([Trim(1), record.Trim(1), marker.Trim(1)], [2, 3, 4])
field = {Trim: 7}
assert(field.Trim, 7)
assert(["x".Trim, "x".trim, "x".strip], [undefined, undefined, undefined])
[record.Trim(2), marker.Trim(2)]
"#,
        "[4,5]",
    );
    for (source, message, line) in [
        ("(5).Trim()", "number.Trim is not a function", 1),
        (
            "items = [\"x\"]\nitems.Trim()",
            "items.Trim is not a function",
            2,
        ),
        ("String.Trim(\" x \")", "String.Trim is not a function", 1),
        (
            "text = \"x\"\ntext.Trim = 1",
            "Cannot assign 'Trim' on string",
            2,
        ),
        ("\"x\".TrimStart()", "x.TrimStart is not a function", 1),
        ("\"x\".TrimEnd()", "x.TrimEnd is not a function", 1),
        ("\"x\".TRIM()", "x.TRIM is not a function", 1),
    ] {
        let error = Runtime::new().run(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(error.position(), Some(Position::new(line, 1)), "{source}");
    }
}
