//! Complete accepted controls and their exact literal-only lowerings. Ordinary
//! feature tests need neither Git nor proposal-session artifacts.

use nucleoid::{Runtime, Value};

use crate::common;

pub struct SealedCase {
    pub document: &'static str,
    pub title: &'static str,
    pub accepted: &'static str,
    pub candidate: &'static str,
    pub assertions: usize,
    pub expected: Option<&'static str>,
    edits: &'static [(&'static str, &'static str, usize)],
    observations: &'static [&'static str],
}

pub const SEALED: &[SealedCase] = &[
    SealedCase {
        document: "spec",
        title: "Nucleoid fills omitted literal default parameters",
        accepted: r#"def price(quantity, unit) => quantity * unit
one = price(1, 10)
two = price(2, 10)
three = price(3, 10)
custom = price(2, 7)
assert(one, 10)
assert(two, 20)
assert(three, 30)
assert(custom, 14)"#,
        candidate: r#"def price(quantity, unit = 10) => quantity * unit
one = price(1)
two = price(2)
three = price(3)
custom = price(2, 7)
assert(one, 10)
assert(two, 20)
assert(three, 30)
assert(custom, 14)"#,
        assertions: 4,
        expected: None,
        edits: &[
            (
                "def price(quantity, unit)",
                "def price(quantity, unit = 10)",
                1,
            ),
            ("price(1, 10)", "price(1)", 1),
            ("price(2, 10)", "price(2)", 1),
            ("price(3, 10)", "price(3)", 1),
        ],
        observations: &["one", "two", "three", "custom"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves supplied falsy arguments when defaults exist",
        accepted: r#"def pick(tag, fallback) => [tag, fallback]
one = pick("a", 12)
two = pick("b", 12)
three = pick("c", 12)
suppliedZero = pick("z", 0)
suppliedFalse = pick("f", false)
suppliedEmpty = pick("e", "")
suppliedNull = pick("n", null).value
assert(one, ["a", 12])
assert(two, ["b", 12])
assert(three, ["c", 12])
assert(suppliedZero, ["z", 0])
assert(suppliedFalse, ["f", false])
assert(suppliedEmpty, ["e", ""])
assert(suppliedNull, ["n", null])
def flag(tag, enabled) => [tag, enabled]
flagA = flag("a", false)
flagB = flag("b", false)
flagC = flag("c", false)
assert(flagA, ["a", false])
assert(flagB, ["b", false])
assert(flagC, ["c", false])
def absent(tag, missing) => [tag, missing]
nullA = absent("a", null).value
nullB = absent("b", null).value
nullC = absent("c", null).value
assert(nullA, ["a", null])
assert(nullB, ["b", null])
assert(nullC, ["c", null])"#,
        candidate: r#"def pick(tag, fallback = 12) => [tag, fallback]
one = pick("a")
two = pick("b")
three = pick("c")
suppliedZero = pick("z", 0)
suppliedFalse = pick("f", false)
suppliedEmpty = pick("e", "")
suppliedNull = pick("n", null).value
assert(one, ["a", 12])
assert(two, ["b", 12])
assert(three, ["c", 12])
assert(suppliedZero, ["z", 0])
assert(suppliedFalse, ["f", false])
assert(suppliedEmpty, ["e", ""])
assert(suppliedNull, ["n", null])
def flag(tag, enabled = false) => [tag, enabled]
flagA = flag("a")
flagB = flag("b")
flagC = flag("c")
assert(flagA, ["a", false])
assert(flagB, ["b", false])
assert(flagC, ["c", false])
def absent(tag, missing = null) => [tag, missing]
nullA = absent("a").value
nullB = absent("b").value
nullC = absent("c").value
assert(nullA, ["a", null])
assert(nullB, ["b", null])
assert(nullC, ["c", null])"#,
        assertions: 13,
        expected: None,
        edits: &[
            ("def pick(tag, fallback)", "def pick(tag, fallback = 12)", 1),
            ("pick(\"a\", 12)", "pick(\"a\")", 1),
            ("pick(\"b\", 12)", "pick(\"b\")", 1),
            ("pick(\"c\", 12)", "pick(\"c\")", 1),
            (
                "def flag(tag, enabled)",
                "def flag(tag, enabled = false)",
                1,
            ),
            ("flag(\"a\", false)", "flag(\"a\")", 1),
            ("flag(\"b\", false)", "flag(\"b\")", 1),
            ("flag(\"c\", false)", "flag(\"c\")", 1),
            (
                "def absent(tag, missing)",
                "def absent(tag, missing = null)",
                1,
            ),
            ("absent(\"a\", null)", "absent(\"a\")", 1),
            ("absent(\"b\", null)", "absent(\"b\")", 1),
            ("absent(\"c\", null)", "absent(\"c\")", 1),
        ],
        observations: &[
            "one",
            "two",
            "three",
            "suppliedZero",
            "suppliedFalse",
            "suppliedEmpty",
            "suppliedNull",
            "flagA",
            "flagB",
            "flagC",
            "nullA",
            "nullB",
            "nullC",
        ],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid applies literal defaults to typed functions and parenthesized arrows",
        accepted: r#"def scale(value: Number, factor: Number) => value * factor
adjust = (value, offset) => value + offset
one = scale(2, 0.5)
two = scale(4, 0.5)
three = scale(6, 0.5)
four = adjust(1, -2)
five = adjust(3, -2)
six = adjust(5, -2)
assert(one, 1)
assert(two, 2)
assert(three, 3)
assert(four, -1)
assert(five, 1)
assert(six, 3)"#,
        candidate: r#"def scale(value: Number, factor: Number = 0.5) => value * factor
adjust = (value, offset = -2) => value + offset
one = scale(2)
two = scale(4)
three = scale(6)
four = adjust(1)
five = adjust(3)
six = adjust(5)
assert(one, 1)
assert(two, 2)
assert(three, 3)
assert(four, -1)
assert(five, 1)
assert(six, 3)"#,
        assertions: 6,
        expected: None,
        edits: &[
            ("factor: Number)", "factor: Number = 0.5)", 1),
            ("(value, offset) =>", "(value, offset = -2) =>", 1),
            ("scale(2, 0.5)", "scale(2)", 1),
            ("scale(4, 0.5)", "scale(4)", 1),
            ("scale(6, 0.5)", "scale(6)", 1),
            ("adjust(1, -2)", "adjust(1)", 1),
            ("adjust(3, -2)", "adjust(3)", 1),
            ("adjust(5, -2)", "adjust(5)", 1),
        ],
        observations: &["one", "two", "three", "four", "five", "six"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid starts defaulted parameter histories at their effective values",
        accepted: r#"step = 100
def sample(seed, step) => [seed, step = step + 1, step@first, step@last]
one = sample(1, 4)
two = sample(2, 4)
three = sample(3, 4)
provided = sample(4, 9)
assert(one, [1, 5, 4, 5])
assert(two, [2, 5, 4, 5])
assert(three, [3, 5, 4, 5])
assert(provided, [4, 10, 9, 10])
assert(step, 100)"#,
        candidate: r#"step = 100
def sample(seed, step = 4) => [seed, step = step + 1, step@first, step@last]
one = sample(1)
two = sample(2)
three = sample(3)
provided = sample(4, 9)
assert(one, [1, 5, 4, 5])
assert(two, [2, 5, 4, 5])
assert(three, [3, 5, 4, 5])
assert(provided, [4, 10, 9, 10])
assert(step, 100)"#,
        assertions: 5,
        expected: None,
        edits: &[
            ("def sample(seed, step)", "def sample(seed, step = 4)", 1),
            ("sample(1, 4)", "sample(1)", 1),
            ("sample(2, 4)", "sample(2)", 1),
            ("sample(3, 4)", "sample(3)", 1),
        ],
        observations: &["step", "one", "two", "three", "provided"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid binds independent literal constructor and method defaults",
        accepted: r#"class Counter(seed):
    this.count = seed
    def advance(delta):
        this.count += delta
        return this.count
first = Counter(1)
second = Counter(1)
third = Counter(1)
one = first.advance(2).value
two = second.advance(2).value
three = third.advance(2).value
again = first.advance(2).value
supplied = Counter(5)
custom = supplied.advance(4).value
assert(first.count, 5)
assert(second.count, 3)
assert(third.count, 3)
assert(supplied.count, 9)
assert(one, 3)
assert(two, 3)
assert(three, 3)
assert(again, 5)
assert(custom, 9)"#,
        candidate: r#"class Counter(seed = 1):
    this.count = seed
    def advance(delta = 2):
        this.count += delta
        return this.count
first = Counter()
second = Counter()
third = Counter()
one = first.advance().value
two = second.advance().value
three = third.advance().value
again = first.advance().value
supplied = Counter(5)
custom = supplied.advance(4).value
assert(first.count, 5)
assert(second.count, 3)
assert(third.count, 3)
assert(supplied.count, 9)
assert(one, 3)
assert(two, 3)
assert(three, 3)
assert(again, 5)
assert(custom, 9)"#,
        assertions: 9,
        expected: None,
        edits: &[
            ("class Counter(seed)", "class Counter(seed = 1)", 1),
            ("def advance(delta)", "def advance(delta = 2)", 1),
            ("Counter(1)", "Counter()", 3),
            ("first.advance(2)", "first.advance()", 2),
            ("second.advance(2)", "second.advance()", 1),
            ("third.advance(2)", "third.advance()", 1),
        ],
        observations: &[
            "first.count",
            "second.count",
            "third.count",
            "supplied.count",
            "one",
            "two",
            "three",
            "again",
            "custom",
        ],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid restores defaulted function callers after failed redefinitions",
        accepted: r#"def price(quantity, unit) => quantity * unit
one = price(1, 10)
two = price(2, 10)
three = price(3, 10)
four = price(4, 10)
assert(one, 10)
assert(two, 20)
assert(three, 30)
assert(four, 40)
try:
    def price(quantity, unit) => quantity * unit + 1
    assert(one, 11)
    assert(two, 21)
    assert(three, 31)
    assert(four, 41)
    throw "RESTORE_LITERAL_DEFAULT"
catch error:
    assert(error, "RESTORE_LITERAL_DEFAULT")
assert(one, 10)
assert(two, 20)
assert(three, 30)
assert(four, 40)"#,
        candidate: r#"def price(quantity, unit = 10) => quantity * unit
one = price(1)
two = price(2)
three = price(3)
four = price(4)
assert(one, 10)
assert(two, 20)
assert(three, 30)
assert(four, 40)
try:
    def price(quantity, unit = 10) => quantity * unit + 1
    assert(one, 11)
    assert(two, 21)
    assert(three, 31)
    assert(four, 41)
    throw "RESTORE_LITERAL_DEFAULT"
catch error:
    assert(error, "RESTORE_LITERAL_DEFAULT")
assert(one, 10)
assert(two, 20)
assert(three, 30)
assert(four, 40)"#,
        assertions: 13,
        expected: None,
        edits: &[
            (
                "def price(quantity, unit)",
                "def price(quantity, unit = 10)",
                2,
            ),
            ("price(1, 10)", "price(1)", 1),
            ("price(2, 10)", "price(2)", 1),
            ("price(3, 10)", "price(3)", 1),
            ("price(4, 10)", "price(4)", 1),
        ],
        observations: &["one", "two", "three", "four"],
    },
    SealedCase {
        document: "reference",
        title: "Literal parameter defaults preserve supplied names",
        accepted: r#"def describe(name, suffix) => name + suffix
one = describe("Can", "!")
two = describe("Daphne", "!")
three = describe("Merve", "!")
assert(one, "Can!")
assert(two, "Daphne!")
assert(three, "Merve!")
[one, two, three]"#,
        candidate: r#"def describe(name, suffix = "!") => name + suffix
one = describe("Can")
two = describe("Daphne")
three = describe("Merve")
assert(one, "Can!")
assert(two, "Daphne!")
assert(three, "Merve!")
[one, two, three]"#,
        assertions: 3,
        expected: Some(r#"["Can!","Daphne!","Merve!"]"#),
        edits: &[
            (
                "def describe(name, suffix)",
                "def describe(name, suffix = \"!\")",
                1,
            ),
            ("describe(\"Can\", \"!\")", "describe(\"Can\")", 1),
            ("describe(\"Daphne\", \"!\")", "describe(\"Daphne\")", 1),
            ("describe(\"Merve\", \"!\")", "describe(\"Merve\")", 1),
        ],
        observations: &["one", "two", "three"],
    },
    SealedCase {
        document: "readme",
        title: "Literal defaults fill omitted trailing arguments",
        accepted: r#"def total(quantity, unit) => quantity * unit
one = total(1, 5)
two = total(2, 5)
three = total(3, 5)
assert(one, 5)
assert(two, 10)
assert(three, 15)"#,
        candidate: r#"def total(quantity, unit = 5) => quantity * unit
one = total(1)
two = total(2)
three = total(3)
assert(one, 5)
assert(two, 10)
assert(three, 15)"#,
        assertions: 3,
        expected: None,
        edits: &[
            (
                "def total(quantity, unit)",
                "def total(quantity, unit = 5)",
                1,
            ),
            ("total(1, 5)", "total(1)", 1),
            ("total(2, 5)", "total(2)", 1),
            ("total(3, 5)", "total(3)", 1),
        ],
        observations: &["one", "two", "three"],
    },
    SealedCase {
        document: "examples",
        title: "Defaulted calibration remains reactive to supplied arguments",
        accepted: r#"def calibrate(raw, offset) => raw + offset
sensor = 5
one = calibrate(sensor, 2)
two = calibrate(7, 2)
three = calibrate(9, 2)
untouched = calibrate(2, 0)
assert(one, 7)
assert(two, 9)
assert(three, 11)
assert(untouched, 2)
sensor = 6
assert(one, 8)"#,
        candidate: r#"def calibrate(raw, offset = 2) => raw + offset
sensor = 5
one = calibrate(sensor)
two = calibrate(7)
three = calibrate(9)
untouched = calibrate(2, 0)
assert(one, 7)
assert(two, 9)
assert(three, 11)
assert(untouched, 2)
sensor = 6
assert(one, 8)"#,
        assertions: 5,
        expected: None,
        edits: &[
            (
                "def calibrate(raw, offset)",
                "def calibrate(raw, offset = 2)",
                1,
            ),
            ("calibrate(sensor, 2)", "calibrate(sensor)", 1),
            ("calibrate(7, 2)", "calibrate(7)", 1),
            ("calibrate(9, 2)", "calibrate(9)", 1),
        ],
        observations: &["sensor", "one", "two", "three", "untouched"],
    },
];

/// Independently constrain the complete candidate to precisely the authorized
/// formal insertions and repeated-literal omissions, not arbitrary rewrites.
pub fn assert_exact_lowering(sealed: &SealedCase) {
    let mut source = sealed.accepted.to_string();
    for &(before, after, count) in sealed.edits {
        assert_eq!(source.matches(before).count(), count, "{}", sealed.title);
        source = source.replace(before, after);
    }
    assert_eq!(source, sealed.candidate, "{}", sealed.title);
}

pub fn documents() -> [(&'static str, Vec<common::Case>); 4] {
    [
        (
            "spec",
            common::cases(include_str!("../../nucleoid.spec.md")),
        ),
        ("reference", common::cases(include_str!("../reference.md"))),
        (
            "readme",
            common::snippets(include_str!("../../README.md"), "README.md"),
        ),
        (
            "examples",
            common::snippets(include_str!("../../docs/examples.md"), "docs/examples.md"),
        ),
    ]
}

/// Compare meaningful typed and serialized observations, not function metadata
/// or graph-source strings that necessarily differ between these new forms.
pub fn assert_observations(actual: &Runtime, expected: &Runtime, sealed: &SealedCase) {
    fn observation<'a>(runtime: &'a Runtime, path: &str) -> &'a Value {
        let mut parts = path.split('.');
        let name = parts.next().unwrap();
        let mut value = runtime
            .state
            .variable(name)
            .unwrap_or_else(|| panic!("{name}"));
        for property in parts {
            let Value::Object(id) = value else {
                panic!("{path}: expected object owner")
            };
            value = runtime
                .state
                .property(id, property)
                .unwrap_or_else(|| panic!("{path}"));
        }
        value
    }
    for path in sealed.observations {
        let actual_value = observation(actual, path);
        let expected_value = observation(expected, path);
        assert_eq!(
            actual_value, expected_value,
            "{}: typed {path}",
            sealed.title
        );
        assert_eq!(
            actual.serialize_json(actual_value).unwrap(),
            expected.serialize_json(expected_value).unwrap(),
            "{}: JSON {path}",
            sealed.title
        );
    }
}
