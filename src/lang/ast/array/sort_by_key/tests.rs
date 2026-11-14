use super::*;
use crate::error::ErrorKind;
use crate::number;
use crate::value::ObjectId;

struct Keys(u64);

impl Keys {
    fn pick(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from((self.0 >> 33) % u64::try_from(bound).unwrap()).unwrap()
    }

    fn list(&mut self, pool: &[Value], length: usize) -> Vec<(Value, Value)> {
        (0..length)
            .map(|position| (pool[self.pick(pool.len())].clone(), tag(position)))
            .collect()
    }
}

fn tag(position: usize) -> Value {
    Value::Number(number::from_usize(position))
}

fn tagged(keys: Vec<Value>) -> Vec<(Value, Value)> {
    keys.into_iter()
        .enumerate()
        .map(|(position, key)| (key, tag(position)))
        .collect()
}

fn text_key(value: &str) -> Value {
    Value::String(value.to_string())
}

fn number_pool() -> Vec<Value> {
    vec![
        Value::Number(0.0),
        Value::Number(-0.0),
        Value::Number(1.0),
        Value::Number(-1.5),
        Value::Number(2.0),
        Value::Number(f64::INFINITY),
        Value::Number(f64::NEG_INFINITY),
        Value::Bool(true),
        Value::Bool(false),
        Value::Null,
        Value::List(vec![]),
        Value::List(vec![Value::Number(2.0)]),
        Value::Date(5),
        Value::Date(-3),
    ]
}

fn text_pool() -> Vec<Value> {
    ["", "a", "B", "b", "10", "9", "é", "a b", "A"]
        .into_iter()
        .map(text_key)
        .collect()
}

fn unordered_pool() -> Vec<Value> {
    vec![
        Value::Undefined,
        Value::Number(f64::NAN),
        Value::Object(ObjectId::from("o")),
        Value::Class("C".to_string()),
        Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
    ]
}

fn positions(items: &[Value]) -> Vec<usize> {
    items
        .iter()
        .map(|item| number::to_usize(item.to_number()))
        .collect()
}

#[test]
fn number_keys_sort_stably_with_zero_and_negative_zero_tied() {
    let pattern = [
        (1.0, 2),
        (0.0, 1),
        (-0.0, 1),
        (-1.0, 0),
        (0.0, 1),
        (-0.0, 1),
    ];
    let keys: Vec<Value> = (0..48)
        .map(|position| Value::Number(pattern[position % pattern.len()].0))
        .collect();
    let keyed = tagged(keys);
    let sorted = ordered(keyed.clone());
    let expected: Vec<usize> = (0..3)
        .flat_map(|bucket| {
            (0..48).filter(move |position| pattern[position % pattern.len()].1 == bucket)
        })
        .collect();
    assert_eq!(positions(&sorted), expected);
    assert_eq!(sorted, by_placement(keyed));
}

#[test]
fn folding_negative_zero_changes_no_item() {
    let zeros: Vec<Value> = (0..30)
        .map(|position| Value::Number(if position % 3 == 0 { -0.0 } else { 0.0 }))
        .collect();
    let keyed: Vec<(Value, Value)> = zeros
        .iter()
        .map(|zero| (zero.clone(), zero.clone()))
        .collect();
    let signs = |items: &[Value]| -> Vec<bool> {
        items
            .iter()
            .map(|item| item.to_number().is_sign_negative())
            .collect()
    };
    assert_eq!(signs(&ordered(keyed)), signs(&zeros));
}

#[test]
fn text_keys_compare_by_code_point_and_keep_ties_in_order() {
    let pattern = ["b", "B", "a", "10", "9", "", "é", "a", "A"];
    let keys: Vec<Value> = (0..27)
        .map(|position| text_key(pattern[position % pattern.len()]))
        .collect();
    let keyed = tagged(keys);
    let sorted = ordered(keyed.clone());
    let mut expected = keyed.clone();
    expected.sort_by_key(|(key, _)| key.to_string());
    let expected: Vec<Value> = expected.into_iter().map(|(_, item)| item).collect();
    assert_eq!(sorted, expected);
    assert_eq!(sorted, by_placement(keyed));
}

#[test]
fn mixed_keys_follow_the_placement_rule() {
    let placed = |keys: Vec<Value>| positions(&ordered(tagged(keys)));
    assert_eq!(
        placed(vec![
            text_key("b"),
            Value::Number(5.0),
            text_key("a"),
            Value::Number(3.0),
        ]),
        [2, 0, 3, 1]
    );
    assert_eq!(
        placed(vec![text_key("10"), Value::Number(9.0), text_key("9")]),
        [1, 0, 2]
    );
    assert_eq!(
        placed(vec![Value::Number(9.5), text_key("10"), text_key("9")]),
        [2, 0, 1]
    );
}

#[test]
fn total_preorders_sort_exactly_as_the_placement_rule() {
    let mut keys = Keys(105);
    for pool in [number_pool(), text_pool()] {
        for length in 0..400 {
            let keyed = keys.list(&pool, length % 64);
            assert_eq!(
                ordered(keyed.clone()),
                by_placement(keyed.clone()),
                "{keyed:?}"
            );
        }
    }
}

#[test]
fn unordered_keys_are_the_keys_without_an_order_with_themselves() {
    for key in unordered_pool() {
        assert!(is_unordered(&key), "{key:?}");
    }
    for key in number_pool().into_iter().chain(text_pool()) {
        assert!(!is_unordered(&key), "{key:?}");
    }
}

#[test]
fn a_missing_or_non_function_selector_raises_even_without_items() {
    for predicate in [None, Some(Value::Null), Some(Value::Number(1.0))] {
        let error = Runtime::new().sort_by_key(vec![], predicate).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), "sortBy expects a function");
    }
}
