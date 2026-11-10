use super::*;
use std::thread;

const DEEP: usize = 25_000;
const SMALL_STACK: usize = 128 * 1024;

fn number(value: f64) -> Value {
    Value::Number(value)
}

fn list(items: Vec<Value>) -> Value {
    Value::List(items)
}

fn text(value: &str) -> Value {
    Value::String(value.to_string())
}

fn identity_step(items: Vec<Value>) -> Vec<Value> {
    let mut stepped = Vec::new();
    for item in items {
        match item {
            Value::List(inner) => stepped.extend(inner),
            other => stepped.push(other),
        }
    }
    stepped
}

fn identity_steps(items: Vec<Value>, levels: usize) -> (Vec<Value>, bool) {
    let mut current = items;
    let mut read_null = current.iter().any(Value::is_null);
    for _ in 1..levels {
        current = identity_step(current);
        read_null |= current.iter().any(Value::is_null);
    }
    if levels > 0 {
        current = identity_step(current);
    }
    (current, read_null)
}

fn flattened(items: Vec<Value>, argument: Option<&Value>) -> (Vec<Value>, bool) {
    let mut runtime = Runtime::new();
    let result = runtime.flatten_items(items, argument);
    (result, runtime.null_read)
}

fn shapes() -> Vec<Vec<Value>> {
    vec![
        vec![],
        vec![number(1.0), Value::Null, text("ab")],
        vec![
            list(vec![number(1.0), number(2.0)]),
            number(2.0),
            list(vec![list(vec![number(3.0), list(vec![number(4.0)])])]),
            list(vec![]),
            text("ab"),
        ],
        vec![
            list(vec![number(1.0)]),
            Value::Null,
            list(vec![Value::Null, list(vec![Value::Null])]),
            list(vec![list(vec![list(vec![number(5.0)])])]),
        ],
        vec![list(vec![list(vec![list(vec![list(vec![Value::Null])])])])],
        vec![
            Value::Bool(true),
            list(vec![Value::Undefined, list(vec![])]),
            list(vec![list(vec![]), list(vec![list(vec![])])]),
        ],
    ]
}

fn deep_chain(levels: usize) -> Value {
    let mut value = number(1.0);
    for _ in 0..levels {
        value = list(vec![value]);
    }
    value
}

#[test]
fn an_omitted_depth_is_one_and_a_supplied_one_converts_as_the_index_of_at() {
    assert_eq!(Depth::new(None).levels, 1);
    let rows = [
        (number(1.9), 1),
        (number(2.9), 2),
        (Value::Bool(true), 1),
        (Value::Bool(false), 0),
        (text("2"), 2),
        (text(" 2 "), 2),
        (text(""), 0),
        (text("a"), 0),
        (text("0x10"), 0),
        (list(vec![number(2.0)]), 2),
        (list(vec![]), 0),
        (list(vec![number(1.0), number(2.0)]), 0),
        (Value::Null, 0),
        (Value::Undefined, 0),
        (number(f64::NAN), 0),
        (number(-0.0), 0),
        (number(0.5), 0),
        (number(-1.0), 0),
        (number(-0.5), 0),
        (number(f64::NEG_INFINITY), 0),
        (number(f64::INFINITY), usize::MAX),
        (text("Infinity"), usize::MAX),
        (number(1e300), usize::MAX),
        (number(f64::MAX), usize::MAX),
        (Value::Date(5), 5),
    ];
    for (value, levels) in rows {
        assert_eq!(Depth::new(Some(&value)).levels, levels, "{value:?}");
    }
}

#[test]
fn every_depth_is_its_identity_steps_in_results_and_null_reads() {
    for shape in shapes() {
        assert_eq!(
            flattened(shape.clone(), None),
            identity_steps(shape.clone(), 1)
        );
        for levels in 0..7_u8 {
            assert_eq!(
                flattened(shape.clone(), Some(&number(f64::from(levels)))),
                identity_steps(shape.clone(), usize::from(levels)),
                "{shape:?} at {levels}"
            );
        }
        assert_eq!(
            flattened(shape.clone(), Some(&number(f64::INFINITY))),
            identity_steps(shape, 8)
        );
    }
}

#[test]
fn null_items_are_read_only_at_the_levels_a_call_flattens() {
    let nested_null = || vec![list(vec![Value::Null])];
    assert!(flattened(vec![list(vec![number(1.0)]), Value::Null], None).1);
    assert!(!flattened(vec![list(vec![number(1.0), Value::Null])], None).1);
    assert!(!flattened(nested_null(), None).1);
    assert!(flattened(nested_null(), Some(&number(2.0))).1);
    assert!(flattened(vec![number(1.0), Value::Null], Some(&number(0.0))).1);
    assert!(!flattened(vec![list(vec![Value::Null])], Some(&number(0.0))).1);
    assert!(flattened(nested_null(), Some(&number(f64::INFINITY))).1);
    assert!(!flattened(vec![Value::Undefined], None).1);
}

#[test]
fn the_walk_takes_no_stack_for_a_level() {
    let worker = thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(|| {
            let every = flattened(vec![deep_chain(DEEP)], Some(&number(f64::INFINITY)));
            assert_eq!(every, (vec![number(1.0)], false));
            let all_but_one = flattened(vec![deep_chain(DEEP)], Some(&number(24_999.0)));
            assert_eq!(all_but_one, (vec![list(vec![number(1.0)])], false));
            let wide: Vec<Value> = (0..DEEP).map(|_| deep_chain(3)).collect();
            let (items, read_null) = flattened(wide, Some(&number(f64::INFINITY)));
            assert_eq!((items.len(), read_null), (DEEP, false));
        })
        .unwrap();
    worker.join().unwrap();
}
