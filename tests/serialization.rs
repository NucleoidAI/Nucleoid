use nucleoid::Runtime;

#[test]
fn serializes_an_instance_from_the_state() {
    let mut runtime = Runtime::new();
    let value = runtime
        .run(
            r#"
class Human(name: str):
    this.name = name

$Human.mortal = true
socrates = Human("Socrates")
socrates
"#,
        )
        .unwrap();

    assert_eq!(
        runtime.serialize_json(&value).unwrap(),
        r#"{"id":"socrates","mortal":true,"name":"Socrates"}"#
    );
}

#[test]
fn serializes_scalar_and_list_values() {
    let runtime = Runtime::new();

    assert_eq!(
        runtime
            .serialize_json(&nucleoid::Value::List(vec![
                nucleoid::Value::number(1),
                nucleoid::Value::string("two"),
                nucleoid::Value::Bool(true),
            ]))
            .unwrap(),
        r#"[1.0,"two",true]"#
    );
}
