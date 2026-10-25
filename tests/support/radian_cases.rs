//! Sealed NUC50 accepted-arithmetic controls, independent of Git/session files.
//! Keep the complete nine inputs, their assertion order and explicit return.

use nucleoid::graph::NodeKey;
use nucleoid::{ObjectId, Runtime, Value};

use crate::common;

pub struct SealedCase {
    pub document: &'static str,
    pub title: &'static str,
    pub accepted: &'static str,
    pub assertions: usize,
    pub expected: Option<&'static str>,
    pub rewrites: usize,
    numbers: &'static [&'static str],
}

pub const SEALED: &[SealedCase] = &[
    SealedCase {
        document: "spec",
        title: "Nucleoid keeps degree to radian conversion reactive",
        accepted: r#"degrees = 180
converted = degrees * (Math.PI / 180)
assert(converted, Math.PI)
degrees = 90
assert(converted, Math.PI / 2)
degrees = -180
assert(converted, -Math.PI)
degrees = 360
assert(converted, Math.PI * 2)
degrees = 0
assert(converted, 0)
degrees = -0
assert(converted, 0)
assert(1 / converted, Number.NEGATIVE_INFINITY)
degrees = 90
converted"#,
        assertions: 7,
        expected: Some("1.5707963267948966"),
        rewrites: 1,
        numbers: &["converted"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves numeric coercion while converting degrees to radians",
        accepted: r#"value = " 180 "
converted = value * (Math.PI / 180)
assert(converted, Math.PI)
value = true
assert(converted, Math.PI / 180)
value = false
assert(converted, 0)
value = []
assert(converted, 0)
value = ["90"]
assert(converted, Math.PI / 2)
value = Date(180)
assert(converted, Math.PI)
value = "not-a-number"
assert(Number.isNaN(converted), true)
literalNull = null * (Math.PI / 180)
assert(literalNull, 0)"#,
        assertions: 8,
        expected: None,
        rewrites: 2,
        numbers: &["converted", "literalNull"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves radian conversion extremes omissions and signed zeros",
        accepted: r#"finite = Number.MAX_VALUE * (Math.PI / 180)
assert(Number.isFinite(finite), true)
negativeFinite = -Number.MAX_VALUE * (Math.PI / 180)
assert(Number.isFinite(negativeFinite), true)
assert(negativeFinite < 0, true)
positive = Number.POSITIVE_INFINITY * (Math.PI / 180)
negative = Number.NEGATIVE_INFINITY * (Math.PI / 180)
invalid = Number.NaN * (Math.PI / 180)
omitted = Number.NaN * (Math.PI / 180)
zero = -0 * (Math.PI / 180)
subnormal = Number.MIN_VALUE * (Math.PI / 180)
assert(positive, Number.POSITIVE_INFINITY)
assert(negative, Number.NEGATIVE_INFINITY)
assert(Number.isNaN(invalid), true)
assert(Number.isNaN(omitted), true)
assert(1 / zero, Number.NEGATIVE_INFINITY)
assert(subnormal > 0, true)"#,
        assertions: 9,
        expected: None,
        rewrites: 8,
        numbers: &[
            "finite",
            "negativeFinite",
            "positive",
            "negative",
            "invalid",
            "omitted",
            "zero",
            "subnormal",
        ],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid applies radian conversion to independent class rules",
        accepted: r#"class Angle(degrees):
    this.degrees = degrees
$Angle.radians = $Angle.degrees * (Math.PI / 180)
first = Angle(180)
second = Angle(90)
assert(first.radians, Math.PI)
assert(second.radians, Math.PI / 2)
first.degrees = -180
assert(first.radians, -Math.PI)
assert(second.radians, Math.PI / 2)
delete first.degrees
assert(first.radians, null)
first.degrees = 360
assert(first.radians, Math.PI * 2)
second.degrees = 45
assert(second.radians, Math.PI / 4)"#,
        assertions: 7,
        expected: None,
        rewrites: 1,
        numbers: &[],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid restores radian conversion relationships after failed transactions",
        accepted: r#"class Dial
dial = Dial()
dial.degrees = 180
converted = dial.degrees * (Math.PI / 180)
assert(converted, Math.PI)
try:
    dial.degrees = 90
    throw "ANGLE_ROLLBACK"
catch error:
    assert(error, "ANGLE_ROLLBACK")
assert(dial.degrees, 180)
assert(converted, Math.PI)
try:
    converted = missing * (Math.PI / 180)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(converted, Math.PI)
dial.degrees = 90
assert(converted, Math.PI / 2)"#,
        assertions: 7,
        expected: None,
        rewrites: 2,
        numbers: &["converted"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves ignored radian conversion reads and missing extras",
        accepted: r#"class Probe
probe = Probe()
angle = -180
extra = 1
converted = Math.clamp(angle, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY, extra) * (Math.PI / 180)
assert(converted, -Math.PI)
extra = 2
assert(converted, -Math.PI)
withMissing = Math.clamp(angle, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY, probe.missing) * (Math.PI / 180)
assert(withMissing, null)
probe.missing = 1
assert(withMissing, -Math.PI)
angle = 90
assert(converted, Math.PI / 2)
assert(withMissing, Math.PI / 2)
try:
    Math.clamp(angle, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY, missing) * (Math.PI / 180)
catch error:
    assert(error, ReferenceError("missing is not defined"))"#,
        assertions: 7,
        expected: None,
        rewrites: 3,
        numbers: &["converted", "withMissing"],
    },
    SealedCase {
        document: "reference",
        title: "Degree to radian conversion keeps angles live",
        accepted: r#"degrees = 90
radians = degrees * (Math.PI / 180)
assert(radians, Math.PI / 2)
degrees = 180
assert(radians, Math.PI)
degrees = -90
assert(radians, -Math.PI / 2)"#,
        assertions: 3,
        expected: None,
        rewrites: 1,
        numbers: &["radians"],
    },
    SealedCase {
        document: "readme",
        title: "Degree to radian conversion follows live inputs",
        accepted: r#"degrees = 180
radians = degrees * (Math.PI / 180)
assert(radians, Math.PI)
degrees = 90
assert(radians, Math.PI / 2)"#,
        assertions: 2,
        expected: None,
        rewrites: 1,
        numbers: &["radians"],
    },
    SealedCase {
        document: "examples",
        title: "Radian conversion keeps independent headings reactive",
        accepted: r#"heading = 180
target = 90
radians = heading * (Math.PI / 180)
targetRadians = target * (Math.PI / 180)
assert(radians, Math.PI)
assert(targetRadians, Math.PI / 2)
heading = 360
assert(radians, Math.PI * 2)
assert(targetRadians, Math.PI / 2)
target = -90
assert(targetRadians, -Math.PI / 2)"#,
        assertions: 5,
        expected: None,
        rewrites: 2,
        numbers: &["radians", "targetRadians"],
    },
];

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

pub fn documented_source(sealed: &SealedCase) -> String {
    if matches!(sealed.document, "spec" | "reference") {
        format!("# {}\n\n{}", sealed.title, sealed.accepted)
    } else {
        sealed.accepted.to_string()
    }
}

pub fn rewrite(source: &str) -> (String, usize) {
    // Whole omitted/extra expressions and qualified or negative inputs precede
    // their substrings. Nothing outside the intended conversion RHS changes.
    let replacements = [
        (
            "omitted = Number.NaN * (Math.PI / 180)",
            "omitted = Math.toRadians()",
        ),
        (
            "Math.clamp(angle, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY, extra) * (Math.PI / 180)",
            "Math.toRadians(angle, extra)",
        ),
        (
            "Math.clamp(angle, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY, probe.missing) * (Math.PI / 180)",
            "Math.toRadians(angle, probe.missing)",
        ),
        (
            "Math.clamp(angle, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY, missing) * (Math.PI / 180)",
            "Math.toRadians(angle, missing)",
        ),
        (
            "$Angle.degrees * (Math.PI / 180)",
            "Math.toRadians($Angle.degrees)",
        ),
        (
            "dial.degrees * (Math.PI / 180)",
            "Math.toRadians(dial.degrees)",
        ),
        (
            "-Number.MAX_VALUE * (Math.PI / 180)",
            "Math.toRadians(-Number.MAX_VALUE)",
        ),
        (
            "Number.MAX_VALUE * (Math.PI / 180)",
            "Math.toRadians(Number.MAX_VALUE)",
        ),
        (
            "Number.POSITIVE_INFINITY * (Math.PI / 180)",
            "Math.toRadians(Number.POSITIVE_INFINITY)",
        ),
        (
            "Number.NEGATIVE_INFINITY * (Math.PI / 180)",
            "Math.toRadians(Number.NEGATIVE_INFINITY)",
        ),
        ("Number.NaN * (Math.PI / 180)", "Math.toRadians(Number.NaN)"),
        ("-0 * (Math.PI / 180)", "Math.toRadians(-0)"),
        (
            "Number.MIN_VALUE * (Math.PI / 180)",
            "Math.toRadians(Number.MIN_VALUE)",
        ),
        ("degrees * (Math.PI / 180)", "Math.toRadians(degrees)"),
        ("value * (Math.PI / 180)", "Math.toRadians(value)"),
        ("null * (Math.PI / 180)", "Math.toRadians(null)"),
        ("missing * (Math.PI / 180)", "Math.toRadians(missing)"),
        ("heading * (Math.PI / 180)", "Math.toRadians(heading)"),
        ("target * (Math.PI / 180)", "Math.toRadians(target)"),
    ];
    let mut rewritten = source.to_string();
    let mut count = 0;
    for (accepted, candidate) in replacements {
        count += rewritten.matches(accepted).count();
        rewritten = rewritten.replace(accepted, candidate);
    }
    (rewritten, count)
}

pub fn assert_numeric_same(actual: &Value, expected: &Value) {
    let (Value::Number(actual), Value::Number(expected)) = (actual, expected) else {
        panic!("numeric controls required: {actual:?}, {expected:?}");
    };
    if expected.is_nan() {
        assert!(actual.is_nan(), "{actual} is not NaN");
    } else {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
}

/// Compare observable conversion results and real input reachability, not the
/// intentionally different arithmetic/call rule ASTs or their source strings.
pub fn assert_observations(actual: &Runtime, expected: &Runtime, sealed: &SealedCase) {
    for name in sealed.numbers {
        assert_numeric_same(
            actual.state.variable(name).unwrap(),
            expected.state.variable(name).unwrap(),
        );
    }
    if sealed.title == "Nucleoid applies radian conversion to independent class rules" {
        for name in ["first", "second"] {
            let id = ObjectId::named(name);
            for property in ["degrees", "radians"] {
                assert_numeric_same(
                    actual.state.property(&id, property).unwrap(),
                    expected.state.property(&id, property).unwrap(),
                );
            }
            for runtime in [actual, expected] {
                assert!(runtime.graph.reaches(
                    &NodeKey::property(&id, "degrees"),
                    &NodeKey::property(&id, "radians"),
                ));
            }
        }
    }
}
