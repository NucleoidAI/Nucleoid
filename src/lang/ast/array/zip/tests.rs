use super::*;
use crate::value::ObjectId;

fn numbers(values: &[f64]) -> Value {
    Value::List(values.iter().copied().map(Value::Number).collect())
}

fn instances(names: &[&str]) -> Vec<Value> {
    names
        .iter()
        .map(|name| Value::Object(ObjectId::from(*name)))
        .collect()
}

#[test]
fn a_borrowed_list_lends_only_the_items_the_receiver_can_pair() {
    let mut runtime = Runtime::new();
    let other = numbers(&[1.0, 2.0, 3.0]);
    let partners = runtime.zip_partners(Some(&other), 2).unwrap();
    assert_eq!(partners, vec![Value::Number(1.0), Value::Number(2.0)]);
    let all = runtime.zip_partners(Some(&other), 5).unwrap();
    assert_eq!(
        all,
        vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
    );
    let none = runtime.zip_partners(Some(&other), 0).unwrap();
    assert_eq!(none, Vec::<Value>::new());
}

#[test]
fn a_declared_class_lends_only_the_instances_the_receiver_can_pair() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Truck\na = Truck()\nb = Truck()\nc = Truck()")
        .unwrap();
    let truck = Value::Class("Truck".to_string());
    let partners = runtime.zip_partners(Some(&truck), 2).unwrap();
    assert_eq!(partners, instances(&["a", "b"]));
    let all = runtime.zip_partners(Some(&truck), 5).unwrap();
    assert_eq!(all, instances(&["a", "b", "c"]));
    let none = runtime.zip_partners(Some(&truck), 0).unwrap();
    assert_eq!(none, Vec::<Value>::new());
}

#[test]
fn anything_else_is_not_a_partner() {
    let mut runtime = Runtime::new();
    let function = runtime.run("x -> x").unwrap();
    let pattern = runtime.run("/a/").unwrap();
    assert!(matches!(function, Value::Function(_)), "{function:?}");
    assert!(matches!(pattern, Value::Regex(_)), "{pattern:?}");
    for argument in [
        None,
        Some(Value::Null),
        Some(Value::Undefined),
        Some(Value::Number(5.0)),
        Some(Value::Bool(true)),
        Some(Value::String("ab".to_string())),
        Some(Value::Class("Math".to_string())),
        Some(Value::Class("Missing".to_string())),
        Some(Value::Object(ObjectId::from("o"))),
        Some(Value::Date(0)),
        Some(pattern),
        Some(function),
    ] {
        let error = runtime.zip_partners(argument.as_ref(), 3).unwrap_err();
        assert_eq!(error.message(), "zip expects a list", "{argument:?}");
    }
}
