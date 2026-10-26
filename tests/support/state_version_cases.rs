//! Complete, pre-executed accepted counterparts. No session artifacts or Git
//! are needed by ordinary tests; only the manual history mirrors are lowered.

use nucleoid::{Runtime, Value};

use crate::common;

pub struct SealedCase {
    pub document: &'static str,
    pub title: &'static str,
    pub accepted: &'static str,
    pub candidate: &'static str,
    pub assertions: usize,
    pub expected: Option<&'static str>,
    observations: &'static [&'static str],
}

pub const SEALED: &[SealedCase] = &[
    SealedCase {
        document: "spec",
        title: "Nucleoid reads zero based and named versions of instance properties",
        accepted: r#"class Student
student1 = Student()
student1.name = "Can"
nameHistory = [student1.name.value]
student1.name = "Daphne"
nameHistory.push(student1.name.value)
student1.name = "Merve"
nameHistory.push(student1.name.value)
version0 = nameHistory[0]
version1 = nameHistory[1]
version2 = nameHistory[2]
firstName = nameHistory[0]
lastName = nameHistory[nameHistory.length - 1]
previousName = nameHistory[nameHistory.length - 2]
assert(version0, "Can")
assert(version1, "Daphne")
assert(version2, "Merve")
assert(firstName, "Can")
assert(lastName, "Merve")
assert(previousName, "Daphne")
assert(student1.name, "Merve")
[version0, version1, version2, firstName, lastName, previousName]"#,
        candidate: r#"class Student
student1 = Student()
student1.name = "Can"
student1.name = "Daphne"
student1.name = "Merve"
version0 = student1.name@0
version1 = student1.name@1
version2 = student1.name@2
firstName = student1.name@first
lastName = student1.name@last
previousName = student1.name@previous
assert(version0, "Can")
assert(version1, "Daphne")
assert(version2, "Merve")
assert(firstName, "Can")
assert(lastName, "Merve")
assert(previousName, "Daphne")
assert(student1.name, "Merve")
[version0, version1, version2, firstName, lastName, previousName]"#,
        assertions: 7,
        expected: Some(r#"["Can","Daphne","Merve","Can","Merve","Daphne"]"#),
        observations: &[
            "student1.name",
            "version0",
            "version1",
            "version2",
            "firstName",
            "lastName",
            "previousName",
        ],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid tracks reactive variable versions without duplicate states",
        accepted: r#"input = 1
amount = input * 2
amountHistory = [amount.value]
firstAmount = amountHistory[0]
lastAmount = amountHistory[amountHistory.length - 1]
previousAmount = amountHistory.slice(-2, -1)[0]
assert(firstAmount, 2)
assert(lastAmount, 2)
assert(previousAmount, null)
input = 2
amountHistory.push(amount.value)
assert(amount, 4)
assert(lastAmount, 4)
assert(previousAmount, 2)
input = 2
assert(firstAmount, 2)
assert(lastAmount, 4)
assert(previousAmount, 2)
input = 3
amountHistory.push(amount.value)
assert(lastAmount, 6)
assert(previousAmount, 4)
assert(firstAmount, 2)
version0 = amountHistory[0]
version1 = amountHistory[1]
version2 = amountHistory[2]
version3 = amountHistory[3]
assert(version0, 2)
assert(version1, 4)
assert(version2, 6)
assert(version3, null)"#,
        candidate: r#"input = 1
amount = input * 2
firstAmount = amount@first
lastAmount = amount@last
previousAmount = amount@previous
assert(firstAmount, 2)
assert(lastAmount, 2)
assert(previousAmount, null)
input = 2
assert(amount, 4)
assert(lastAmount, 4)
assert(previousAmount, 2)
input = 2
assert(firstAmount, 2)
assert(lastAmount, 4)
assert(previousAmount, 2)
input = 3
assert(lastAmount, 6)
assert(previousAmount, 4)
assert(firstAmount, 2)
version0 = amount@0
version1 = amount@1
version2 = amount@2
version3 = amount@3
assert(version0, 2)
assert(version1, 4)
assert(version2, 6)
assert(version3, null)"#,
        assertions: 16,
        expected: None,
        observations: &[
            "input",
            "amount",
            "firstAmount",
            "lastAmount",
            "previousAmount",
            "version0",
            "version1",
            "version2",
            "version3",
        ],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid reads versions through nulls and deferred indices",
        accepted: r#"value = "ready"
valueHistory = [value.value]
oldest = valueHistory[0]
future = valueHistory[2]
latest = valueHistory[valueHistory.length - 1]
previous = valueHistory.slice(-2, -1)[0]
assert(oldest, "ready")
assert(future, null)
assert(previous, null)
value = null
valueHistory.push(value.value)
assert(oldest, "ready")
assert(latest, null)
assert(previous, "ready")
assert(future, null)
value = "back"
valueHistory.push(value.value)
assert(oldest, "ready")
assert(latest, "back")
assert(previous, null)
assert(future, "back")"#,
        candidate: r#"value = "ready"
oldest = value@first
future = value@2
latest = value@last
previous = value@previous
assert(oldest, "ready")
assert(future, null)
assert(previous, null)
value = null
assert(oldest, "ready")
assert(latest, null)
assert(previous, "ready")
assert(future, null)
value = "back"
assert(oldest, "ready")
assert(latest, "back")
assert(previous, null)
assert(future, "back")"#,
        assertions: 11,
        expected: None,
        observations: &["value", "oldest", "future", "latest", "previous"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid isolates local and parameter version histories between calls",
        accepted: r#"value = 99
def capture(value) => [(history = [value])[0], value = value + 1, [history.push(value), history[0]][1]]
one = capture(2)
two = capture(10)
assert(one, [2, 3, 2])
assert(two, [10, 11, 10])
globalFirst = value.value
assert(globalFirst, 99)"#,
        candidate: r#"value = 99
def capture(value) => [value@first, value = value + 1, value@previous]
one = capture(2)
two = capture(10)
assert(one, [2, 3, 2])
assert(two, [10, 11, 10])
globalFirst = value.value
assert(globalFirst, 99)"#,
        assertions: 3,
        expected: None,
        observations: &["value", "one", "two", "globalFirst"],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid preserves list values object identity and ordinary fields in versions",
        accepted: r#"items = [1]
itemHistory = [items.value]
items.push(2)
itemHistory.push(items.value)
earlierList = itemHistory[0]
latestList = itemHistory[1]
previousList = itemHistory[0]
assert(earlierList, [1])
assert(latestList, [1, 2])
assert(previousList, [1])
class Record
first = Record()
first.first = "given"
first.last = "family"
first.previous = "ordinary"
second = Record()
selected = first
objectHistory = [selected.value]
selected = second
objectHistory.push(selected.value)
earlierObject = objectHistory[0]
latestObject = objectHistory[1]
previousObject = objectHistory[0]
first.label = "updated"
assert(earlierObject.label, "updated")
assert(latestObject, second)
assert(previousObject, first)
assert(first.first, "given")
assert(first.last, "family")
assert(first.previous, "ordinary")"#,
        candidate: r#"items = [1]
items.push(2)
earlierList = items@first
latestList = items@last
previousList = items@previous
assert(earlierList, [1])
assert(latestList, [1, 2])
assert(previousList, [1])
class Record
first = Record()
first.first = "given"
first.last = "family"
first.previous = "ordinary"
second = Record()
selected = first
selected = second
earlierObject = selected@first
latestObject = selected@last
previousObject = selected@previous
first.label = "updated"
assert(earlierObject.label, "updated")
assert(latestObject, second)
assert(previousObject, first)
assert(first.first, "given")
assert(first.last, "family")
assert(first.previous, "ordinary")"#,
        assertions: 9,
        expected: None,
        observations: &[
            "items",
            "earlierList",
            "latestList",
            "previousList",
            "first",
            "second",
            "selected",
            "earlierObject",
            "latestObject",
            "previousObject",
        ],
    },
    SealedCase {
        document: "spec",
        title: "Nucleoid restores version histories on rollback and resets deleted bindings",
        accepted: r#"value = "A"
valueHistory = [value.value]
try:
    value = "discarded"
    valueHistory.push(value.value)
    throw "ABORT_VERSIONS"
catch error:
    assert(error, "ABORT_VERSIONS")
value = "B"
valueHistory.push(value.value)
firstValue = valueHistory[0]
lastValue = valueHistory[valueHistory.length - 1]
previousValue = valueHistory.slice(-2, -1)[0]
assert(firstValue, "A")
assert(lastValue, "B")
assert(previousValue, "A")
delete value
delete valueHistory
value = "C"
valueHistory = [value.value]
resetFirst = valueHistory[0]
resetPrevious = valueHistory.slice(-2, -1)[0]
assert(resetFirst, "C")
assert(resetPrevious, null)
assert(firstValue, "C")
assert(lastValue, "C")
assert(previousValue, null)"#,
        candidate: r#"value = "A"
try:
    value = "discarded"
    throw "ABORT_VERSIONS"
catch error:
    assert(error, "ABORT_VERSIONS")
value = "B"
firstValue = value@first
lastValue = value@last
previousValue = value@previous
assert(firstValue, "A")
assert(lastValue, "B")
assert(previousValue, "A")
delete value
value = "C"
resetFirst = value@first
resetPrevious = value@previous
assert(resetFirst, "C")
assert(resetPrevious, null)
assert(firstValue, "C")
assert(lastValue, "C")
assert(previousValue, null)"#,
        assertions: 9,
        expected: None,
        observations: &[
            "value",
            "firstValue",
            "lastValue",
            "previousValue",
            "resetFirst",
            "resetPrevious",
        ],
    },
    SealedCase {
        document: "reference",
        title: "State versions expose zero based variable history",
        accepted: r#"score = 10
scoreHistory = [score.value]
score = 20
scoreHistory.push(score.value)
firstScore = scoreHistory[0]
lastScore = scoreHistory[scoreHistory.length - 1]
previousScore = scoreHistory[scoreHistory.length - 2]
assert(firstScore, 10)
assert(lastScore, 20)
assert(previousScore, 10)
score = 20
version1 = scoreHistory[1]
version2 = scoreHistory[2]
assert(version1, 20)
assert(version2, null)"#,
        candidate: r#"score = 10
score = 20
firstScore = score@first
lastScore = score@last
previousScore = score@previous
assert(firstScore, 10)
assert(lastScore, 20)
assert(previousScore, 10)
score = 20
version1 = score@1
version2 = score@2
assert(version1, 20)
assert(version2, null)"#,
        assertions: 5,
        expected: None,
        observations: &[
            "score",
            "firstScore",
            "lastScore",
            "previousScore",
            "version1",
            "version2",
        ],
    },
    SealedCase {
        document: "readme",
        title: "Named state versions read earlier variable values",
        accepted: r#"name = "Can"
nameHistory = [name.value]
name = "Daphne"
nameHistory.push(name.value)
firstName = nameHistory[0]
lastName = nameHistory[nameHistory.length - 1]
previousName = nameHistory[nameHistory.length - 2]
assert(firstName, "Can")
assert(lastName, "Daphne")
assert(previousName, "Can")"#,
        candidate: r#"name = "Can"
name = "Daphne"
firstName = name@first
lastName = name@last
previousName = name@previous
assert(firstName, "Can")
assert(lastName, "Daphne")
assert(previousName, "Can")"#,
        assertions: 3,
        expected: None,
        observations: &["name", "firstName", "lastName", "previousName"],
    },
    SealedCase {
        document: "examples",
        title: "Reactive property versions remain independent between instances",
        accepted: r#"class Meter(seed):
    this.seed = seed
$Meter.total = $Meter.seed * 2
first = Meter(2)
second = Meter(3)
firstTotals = [first.total.value]
secondTotals = [second.total.value]
first.seed = 4
firstTotals.push(first.total.value)
firstOriginal = firstTotals[0]
firstLatest = firstTotals[firstTotals.length - 1]
firstPrevious = firstTotals[firstTotals.length - 2]
secondOriginal = secondTotals[0]
secondLatest = secondTotals[secondTotals.length - 1]
secondPrevious = secondTotals.slice(-2, -1)[0]
assert(firstOriginal, 4)
assert(firstLatest, 8)
assert(firstPrevious, 4)
assert(secondOriginal, 6)
assert(secondLatest, 6)
assert(secondPrevious, null)"#,
        candidate: r#"class Meter(seed):
    this.seed = seed
$Meter.total = $Meter.seed * 2
first = Meter(2)
second = Meter(3)
first.seed = 4
firstOriginal = first.total@first
firstLatest = first.total@last
firstPrevious = first.total@previous
secondOriginal = second.total@first
secondLatest = second.total@last
secondPrevious = second.total@previous
assert(firstOriginal, 4)
assert(firstLatest, 8)
assert(firstPrevious, 4)
assert(secondOriginal, 6)
assert(secondLatest, 6)
assert(secondPrevious, null)"#,
        assertions: 6,
        expected: None,
        observations: &[
            "first",
            "second",
            "firstOriginal",
            "firstLatest",
            "firstPrevious",
            "secondOriginal",
            "secondLatest",
            "secondPrevious",
        ],
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

/// Compare meaningful exposed results, not removed mirrors, graph source
/// strings, or the entire state (which intentionally contains new histories).
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
        // These fixtures deliberately use named, stable instance identities.
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
