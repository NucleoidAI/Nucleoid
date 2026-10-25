//! Complete NUC51 accepted split/map controls, independent of Git/session files.
//! Map results are discarded; compare final observations, not new For ASTs.

use nucleoid::{Runtime, Value};

use crate::common;

pub struct SealedCase {
    pub document: &'static str,
    pub title: &'static str,
    pub accepted: &'static str,
    pub assertions: usize,
    pub expected: Option<&'static str>,
    pub rewrites: usize,
    observations: &'static [&'static str],
}

pub const SEALED: &[SealedCase] = &[
    SealedCase {
        document: "spec",
        title: "Nucleoid iterates string characters in source order",
        accepted: r#"text = "cab"
seen = []
text.split("").map(character => seen.push(character))
assert(seen, ["c", "a", "b"])
assert(text, "cab")
seen"#,
        assertions: 2,
        expected: Some(r#"["c","a","b"]"#),
        rewrites: 1,
        observations: &["text", "seen"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves Unicode scalars while iterating strings",
        accepted: r#"text = "Aé🙂é"
seen = []
text.split("").map(character => seen.push(character))
assert(seen, ["A", "é", "🙂", "e", "́"])
assert(seen.length, 5)
assert(seen.join(""), text)"#,
        assertions: 3,
        expected: None,
        rewrites: 1,
        observations: &["text", "seen"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid snapshots string sources before loop bodies",
        accepted: r#"holder = {text: "ab"}
seen = []
holder.text.split("").map(character => {
    seen.push(character)
    holder.text = "z"
})
assert(seen, ["a", "b"])
assert(holder.text, "z")
holder.text = "cd"
assert(seen, ["a", "b"])
holder.text.split("").map(character => seen.push(character))
assert(seen, ["a", "b", "c", "d"])
assert(holder.text, "cd")"#,
        assertions: 5,
        expected: None,
        rewrites: 2,
        observations: &["holder.text", "seen"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid keeps string iterator scopes and nested loops independent",
        accepted: r#"it = "outer"
letters = "ab"
digits = "12"
seen = []
letters.split("").map(it => {
    seen.push(it)
    digits.split("").map(it => seen.push(it))
    seen.push(it)
})
assert(seen, ["a", "1", "2", "a", "b", "1", "2", "b"])
assert(it, "outer")"#,
        assertions: 2,
        expected: None,
        rewrites: 2,
        observations: &["it", "letters", "digits", "seen"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid skips empty strings without executing loop bodies",
        accepted: r#"empty = ""
seen = []
empty.split("").map(character => seen.push(character))
assert(seen, [])
full = "ab"
full.split("").map(character => seen.push(character))
assert(seen, ["a", "b"])
empty.split("").map(character => seen.push(character))
assert(seen, ["a", "b"])"#,
        assertions: 3,
        expected: None,
        rewrites: 3,
        observations: &["empty", "full", "seen"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid rolls back failed string iteration and restores caller scope",
        accepted: r#"it = "outer"
seen = ["saved"]
try:
    "abc".split("").map(it => {
        seen.push(it)
        if it == "b":
            throw "STOP_STRING_ITERATION"
    })
catch error:
    assert(error, "STOP_STRING_ITERATION")
assert(seen, ["saved"])
assert(it, "outer")
"c".split("").map(it => seen.push(it))
assert(seen, ["saved", "c"])
assert(it, "outer")"#,
        assertions: 5,
        expected: None,
        rewrites: 2,
        observations: &["it", "seen"],
    },
    SealedCase {
        document: "reference",
        title: "String iteration collects a snapshot of text",
        accepted: r#"text = "nuc"
letters = []
text.split("").map(letter => letters.push(letter))
assert(letters, ["n", "u", "c"])
text = "new"
assert(letters, ["n", "u", "c"])"#,
        assertions: 2,
        expected: None,
        rewrites: 1,
        observations: &["text", "letters"],
    },
    SealedCase {
        document: "readme",
        title: "String iteration reads characters directly",
        accepted: r#"word = "NUC"
letters = []
word.split("").map(character => letters.push(character))
assert(letters, ["N", "U", "C"])"#,
        assertions: 1,
        expected: None,
        rewrites: 1,
        observations: &["word", "letters"],
    },
    SealedCase {
        document: "examples",
        title: "String iteration keeps independent receiver data separate",
        accepted: r#"class Message(text):
    this.text = text
first = Message("ab")
second = Message("CD")
seen = []
first.text.split("").map(character => seen.push(character))
second.text.split("").map(character => seen.push(character))
assert(seen, ["a", "b", "C", "D"])
first.text = "xy"
assert(seen, ["a", "b", "C", "D"])
seen = []
first.text.split("").map(character => seen.push(character))
assert(seen, ["x", "y"])
assert(second.text, "CD")"#,
        assertions: 4,
        expected: None,
        rewrites: 3,
        observations: &["first.text", "second.text", "seen"],
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

/// Only the supplied standalone traversals lower to loops. Keep every source,
/// binding, body effect, condition, throw, assertion and final control intact.
pub fn rewrite(source: &str) -> (String, usize) {
    let replacements = [
        (
            r#"holder.text.split("").map(character => {
    seen.push(character)
    holder.text = "z"
})"#,
            r#"for character of holder.text:
    seen.push(character)
    holder.text = "z""#,
        ),
        (
            r#"holder.text.split("").map(character => seen.push(character))"#,
            "for character of holder.text:\n    seen.push(character)",
        ),
        (
            r#"letters.split("").map(it => {
    seen.push(it)
    digits.split("").map(it => seen.push(it))
    seen.push(it)
})"#,
            "for letters:\n    seen.push(it)\n    for digits:\n        seen.push(it)\n    seen.push(it)",
        ),
        (
            r#"empty.split("").map(character => seen.push(character))"#,
            "for character in empty:\n    seen.push(character)",
        ),
        (
            r#"full.split("").map(character => seen.push(character))"#,
            "for character of full:\n    seen.push(character)",
        ),
        (
            r#"    "abc".split("").map(it => {
        seen.push(it)
        if it == "b":
            throw "STOP_STRING_ITERATION"
    })"#,
            r#"    for it of "abc":
        seen.push(it)
        if it == "b":
            throw "STOP_STRING_ITERATION""#,
        ),
        (
            r#""c".split("").map(it => seen.push(it))"#,
            "for it of \"c\":\n    seen.push(it)",
        ),
        (
            r#"text.split("").map(letter => letters.push(letter))"#,
            "for letter in text:\n    letters.push(letter)",
        ),
        (
            r#"word.split("").map(character => letters.push(character))"#,
            "for character of word:\n    letters.push(character)",
        ),
        (
            r#"first.text.split("").map(character => seen.push(character))"#,
            "for character of first.text:\n    seen.push(character)",
        ),
        (
            r#"second.text.split("").map(character => seen.push(character))"#,
            "for character of second.text:\n    seen.push(character)",
        ),
        (
            r#"text.split("").map(character => seen.push(character))"#,
            "for character of text:\n    seen.push(character)",
        ),
    ];
    let mut rewritten = source.to_string();
    let mut loops = 0;
    for (accepted, candidate) in replacements {
        loops += rewritten.matches(accepted).count()
            * candidate
                .lines()
                .filter(|line| line.trim_start().starts_with("for "))
                .count();
        rewritten = rewritten.replace(accepted, candidate);
    }
    (rewritten, loops)
}

/// Intermediate map results, callback ASTs and graph source strings differ
/// deliberately. Compare the final arrays/strings/properties actually exposed.
pub fn assert_observations(actual: &Runtime, expected: &Runtime, sealed: &SealedCase) {
    fn observation(runtime: &Runtime, path: &str) -> String {
        let value = if let Some((name, property)) = path.split_once('.') {
            let Some(Value::Object(id)) = runtime.state.variable(name) else {
                panic!("{name}: expected an observable object");
            };
            runtime.state.property(id, property).unwrap()
        } else {
            runtime.state.variable(path).unwrap()
        };
        runtime.serialize_json(value).unwrap()
    }
    for path in sealed.observations {
        assert_eq!(
            observation(actual, path),
            observation(expected, path),
            "{}: {path}",
            sealed.title
        );
    }
}
