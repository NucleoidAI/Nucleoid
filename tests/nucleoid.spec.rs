// @generated from dataset JSONL by build.rs — do not edit.

#![allow(clippy::approx_constant)]

//! Individually named executable behaviors from `nucleoid.spec.md`.

mod common;

use common::runner;

/// Nucleoid runs a statement in the state
#[rustfmt::skip]
#[test]
fn runs_a_statement_in_the_state() {
    let mut run = runner();
    run("i = 1");
    assert_eq!(run("i == 1"), true);
}

/// Nucleoid runs a expression statement
#[rustfmt::skip]
#[test]
fn runs_a_expression_statement() {
    let mut run = runner();
    run("j = 1");
    assert_eq!(run("j + 2"), 3);
}

/// Nucleoid returns value of variable
#[rustfmt::skip]
#[test]
fn returns_value_of_variable() {
    let mut run = runner();
    run("k = 1");
    run(r#"k

# return: 1"#);
}

/// Nucleoid throws an error if variable is not defined
#[rustfmt::skip]
#[test]
fn throws_an_error_if_variable_is_not_defined() {
    let (_run, mut run_error) = crate::common::runners();
    assert_eq!(run_error("t = e + 1"), "ReferenceError: e is not defined");
}

/// Nucleoid throws an error inside a block
#[rustfmt::skip]
#[test]
fn throws_an_error_inside_a_block() {
    let (mut run, mut run_error) = crate::common::runners();
    run("k = 99");
    assert_eq!(run_error(r#"if k >= 99:
    throw "INVALID""#), "INVALID");
}

/// Nucleoid throws an error as a variable
#[rustfmt::skip]
#[test]
fn throws_an_error_as_a_variable() {
    let (mut run, mut run_error) = crate::common::runners();
    run("length = 0.1");
    assert_eq!(run_error(r#"if length < 1:
    throw length"#), 0.1);
    assert_eq!(run_error(r#"if length < 1.1:
    throw 'length'"#), "length");
}

/// Nucleoid creates a class with constructor
#[rustfmt::skip]
#[test]
fn creates_a_class_with_constructor() {
    let mut run = runner();
    run(r#"class Shape(type: str):
    this.type = type"#);
    run(r#"shape1 = Shape("Square")"#);
    assert_eq!(run("shape1"), serde_json::json!({ "id": "shape1", "type": "Square" }));
}

/// Nucleoid creates a class with a constructor and a typed attribute
#[rustfmt::skip]
#[test]
fn creates_a_class_with_a_constructor_and_a_typed_attribute() {
    let mut run = runner();
    run(r#"class Shape:
    type: str

    def init(type: str):
        this.type = type"#);
    run(r#"shape1 = Shape("Rectangle")"#);
    assert_eq!(run("shape1"), serde_json::json!({ "id": "shape1", "type": "Rectangle" }));
}

/// Nucleoid adds an object to the class's object list
#[rustfmt::skip]
#[test]
fn adds_an_object_to_the_class_s_object_list() {
    let mut run = runner();
    run(r#"class Student:
    pass"#);
    run("user0 = Student()");
    assert_eq!(run(r#"Student.find(student => student.id == "user0")"#), serde_json::json!({ "id": "user0" }));
    assert_eq!(run(r#"Student["user0"]"#), serde_json::json!({ "id": "user0" }));
}

/// Nucleoid preserves class and object lists when a class is updated
#[rustfmt::skip]
#[test]
fn preserves_class_and_object_lists_when_a_class_is_updated() {
    let mut run = runner();
    run(r#"class User:
    pass"#);
    run("User()");
    assert_eq!(run("Class.length"), 1);
    assert_eq!(run("User.length"), 1);
    run(r#"class User:
    pass"#);
    assert_eq!(run("Class.length"), 1);
    assert_eq!(run("User.length"), 1);
    run("User()");
    assert_eq!(run("Class.length"), 1);
    assert_eq!(run("User.length"), 2);
}

/// Nucleoid places an instance in the list of the class when created
#[rustfmt::skip]
#[test]
fn places_an_instance_in_the_list_of_the_class_when_created() {
    let mut run = runner();
    run(r#"class Student:
    pass"#);
    {
        let actual = run("typeof Student");
        let expected = run("(List)");
        assert_eq!(actual, expected);
    }
    run("student1 = Student()");
    assert_eq!(run("Student.length"), 1);
}

/// Nucleoid creates a class and a subclass
#[rustfmt::skip]
#[test]
fn creates_a_class_and_a_subclass() {
    let mut run = runner();
    run(r#"class Person(name: str):
    this.name = name"#);
    run(r#"class Student: Person
    def init(name, school):
        super(name)
        this.name = name
        this.school = school"#);
    run(r#"student1 = Student("Emma", "Riverside High")"#);
    assert_eq!(run("student1"), serde_json::json!({ "id": "student1", "name": "Emma", "school": "Riverside High" }));
}

/// Nucleoid runs a class-level property assignment
#[rustfmt::skip]
#[test]
fn runs_a_class_level_property_assignment() {
    let mut run = runner();
    run(r#"class Human(name: str):
    this.name = name"#);
    run("$Human.mortal = true");
    run(r#"human1 = Human("Socrates")"#);
    assert_eq!(run("human1.mortal"), true);
}

/// Nucleoid runs a class-level conditional
#[rustfmt::skip]
#[test]
fn runs_a_class_level_conditional() {
    let mut run = runner();
    run(r#"class Device(profile: str):
    this.profile = profile"#);
    run(r#"if $Device.profile:
    $Device.active = true"#);
    run("device1 = Device()");
    run(r#"device2 = Device("PROFILE-1")"#);
    assert_eq!(run("device1.active"), serde_json::Value::Null);
    assert_eq!(run("device2.active"), true);
}

/// Nucleoid creates an instance in a block and assigns it to a property
#[rustfmt::skip]
#[test]
fn creates_an_instance_in_a_block_and_assigns_it_to_a_property() {
    let mut run = runner();
    run(r#"class Room:
    pass"#);
    run(r#"class Meeting:
    pass"#);
    run("room1 = Room()");
    run(r#"$Meeting.time = Date.now() + " @ " + $Meeting.date.toDateString()"#);
    run(r#"{
    meeting = Meeting()
    meeting.date = Date("2020-1-1")
    room1.meeting = meeting
}"#);
    assert_eq!(run("room1.meeting.date.toDateString()"), "Wed Jan 01 2020");
    assert_eq!(run("room1.meeting.time[-17:]"), "@ Wed Jan 01 2020");
}

/// Nucleoid creates nested instances in a block and assigns them to a property
#[rustfmt::skip]
#[test]
fn creates_nested_instances_in_a_block_and_assigns_them_to_a_property() {
    let mut run = runner();
    run(r#"class Timesheet:
    pass"#);
    run(r#"class Task:
    pass"#);
    run(r#"class Project:
    pass"#);
    run(r#"$Project.code = "N-" + $Project.number"#);
    run("timesheet1 = Timesheet()");
    run(r#"{
    task = Task()
    task.project = Project()
    task.project.number = 3668347
    timesheet1.task = task
}"#);
    assert_eq!(run("timesheet1.task.project.number"), 3668347);
    assert_eq!(run("timesheet1.task.project.code"), "N-3668347");
}

/// Nucleoid creates a local variable in a block and uses in assignment
#[rustfmt::skip]
#[test]
fn creates_a_local_variable_in_a_block_and_uses_in_assignment() {
    let mut run = runner();
    run("integer = 30");
    run("equivalency = null");
    run(r#"{
    division = integer / 10
    equivalency = division * 10
}"#);
    assert_eq!(run("equivalency"), 30);
    run("integer = 40");
    assert_eq!(run("equivalency"), 40);
}

/// Nucleoid creates a standard built-in object as a local variable inside a block
#[rustfmt::skip]
#[test]
fn creates_a_standard_built_in_object_as_a_local_variable_inside_a_block() {
    let mut run = runner();
    run(r#"{
    f = Boolean(false)
    condition = f
}"#);
    assert_eq!(run("condition"), false);
}

/// Nucleoid creates and assigns an instance to a local variable inside a block
#[rustfmt::skip]
#[test]
fn creates_and_assigns_an_instance_to_a_local_variable_inside_a_block() {
    let mut run = runner();
    run(r#"class Device:
    pass"#);
    run("$Device.renew = $Device.created + 604800000");
    run(r#"{
    device = Device()
    device.created = Date.now()
}"#);
    assert_eq!(run("Device[0].renew - Device[0].created"), 604800000);
}

/// Nucleoid creates and assigns an instance with a constructor to a local variable inside a block
#[rustfmt::skip]
#[test]
fn creates_and_assigns_an_instance_with_a_constructor_to_a_local_variable_inside_a_block() {
    let mut run = runner();
    run(r#"class Member(first: str, last: str):
    this.first = first
    this.last = last"#);
    run(r#"$Member.display = $Member.last + ", " + $Member.first"#);
    run(r#"{
    member = Member("First", "Last")
}"#);
    assert_eq!(run("Member[0].display"), "Last, First");
}

/// Nucleoid creates an object in a block and assigns it to a class-level property before instantiation
#[rustfmt::skip]
#[test]
fn creates_an_object_in_a_block_and_assigns_it_to_a_class_level_property_before_instantiation() {
    let mut run = runner();
    run(r#"class Member:
    pass"#);
    run(r#"{
    registration = Object()
    registration.date = Date("2019-1-2")
    $Member.registration = registration
}"#);
    run("member1 = Member()");
    assert_eq!(run("member1.registration.date.toDateString()"), "Wed Jan 02 2019");
    assert_eq!(run("member1.registration.age"), serde_json::Value::Null);
}

/// Nucleoid creates an object in a block and assigns it to a class-level property after instantiation
#[rustfmt::skip]
#[test]
fn creates_an_object_in_a_block_and_assigns_it_to_a_class_level_property_after_instantiation() {
    let mut run = runner();
    run(r#"class Distance:
    pass"#);
    run("distance1 = Distance()");
    run(r#"{
    location = Object()
    location.coordinates = "40.6976701,-74.2598779"
    $Distance.startingPoint = location
}"#);
    assert_eq!(run("distance1.startingPoint.coordinates"), "40.6976701,-74.2598779");
    assert_eq!(run("distance1.startingPoint.print"), serde_json::Value::Null);
}

/// Nucleoid calls function in an assignment
#[rustfmt::skip]
#[test]
fn calls_function_in_an_assignment() {
    let mut run = runner();
    run(r#"def multiply(first_factor, second_factor):
    product = first_factor * second_factor
    return product"#);
    run("x = 1");
    run("y = 2");
    run("z = multiply(x, y) + 1");
    assert_eq!(run("z"), 3);
}

/// Nucleoid assigns a block in a function as a dependency
#[rustfmt::skip]
#[test]
fn assigns_a_block_in_a_function_as_a_dependency() {
    let mut run = runner();
    run(r#"class Student:
    pass"#);
    run("student1 = Student()");
    run("student1.age = 7");
    run("student2 = Student()");
    run("student2.age = 8");
    run("student3 = Student()");
    run("student3.age = 9");
    run("age = 8");
    run("student = Student.find(s => s.age == age)");
    {
        let actual = run("student");
        let expected = run("(student2)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("student"), serde_json::json!({ "id": "student2", "age": 8 }));
    run("age = 9");
    {
        let actual = run("student");
        let expected = run("(student3)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("student"), serde_json::json!({ "id": "student3", "age": 9 }));
}

/// Nucleoid supports chained functions with a parameter in an expression
#[rustfmt::skip]
#[test]
fn supports_chained_functions_with_a_parameter_in_an_expression() {
    let mut run = runner();
    run(r#"class Result(score: int):
    this.score = score"#);
    run("Result(10); Result(15); Result(20)");
    run("upperThreshold = 18");
    run("lowerThreshold = 12");
    run("list = Result.filter(r => r.score > lowerThreshold).filter(r => r.score < upperThreshold)");
    assert_eq!(run("list.length"), 1);
    assert_eq!(run("list[0].score"), 15);
    run("lowerThreshold = 7");
    assert_eq!(run("list.length"), 2);
    assert_eq!(run("list[0].score"), 10);
    assert_eq!(run("list[1].score"), 15);
    run("upperThreshold = 14");
    assert_eq!(run("list.length"), 1);
    assert_eq!(run("list[0].score"), 10);
}

/// Nucleoid supports an array with brackets
#[rustfmt::skip]
#[test]
fn supports_an_array_with_brackets() {
    let mut run = runner();
    run(r#"states = ["NY", "GA", "CT", "MI"]"#);
    run(r#"states[2]

# return: "CT""#);
}

/// Nucleoid throws an error if a variable in an expression is not defined
#[rustfmt::skip]
#[test]
fn throws_an_error_if_a_variable_in_an_expression_is_not_defined() {
    let (_run, mut run_error) = crate::common::runners();
    assert_eq!(run_error("e == 2.71828"), "ReferenceError: e is not defined");
}

/// Nucleoid retrieves the value of a variable
#[rustfmt::skip]
#[test]
fn retrieves_the_value_of_a_variable() {
    let mut run = runner();
    run("number = -1");
    run(r#"number

# return: -1"#);
}

/// Nucleoid creates a property assignment on a local variable only if the instance is defined
#[rustfmt::skip]
#[test]
fn creates_a_property_assignment_on_a_local_variable_only_if_the_instance_is_defined() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Ticket:
    pass"#);
    assert_eq!(run_error(r#"{
    ticket = Ticket()
    ticket.event.group = "ENTERTAINMENT"
}"#), "ReferenceError: ticket.event is not defined");
}

/// Nucleoid declares a local variable as undefined
#[rustfmt::skip]
#[test]
fn declares_a_local_variable_as_undefined() {
    let mut run = runner();
    run(r#"class Device(code: str):
    this.code = code"#);
    run(r#"device1 = Device("A0")"#);
    run(r#"device2 = Device("B1")"#);
    run(r#"{
    device = Device.find(d => d.code == "A0")
    if not device:
        throw "INVALID_DEVICE"
    return device
}

# return: { "id": "device1", "code": "A0" }"#);
}

/// Nucleoid rejects a local variable declared as undefined
#[rustfmt::skip]
#[test]
fn rejects_a_local_variable_declared_as_undefined() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Device(code: str):
    this.code = code"#);
    run(r#"device1 = Device("A0")"#);
    run(r#"device2 = Device("B1")"#);
    assert_eq!(run_error(r#"{
    device = Device.find(d => d.code == "A1")
    if not device:
        throw "INVALID_DEVICE"
    return device
}"#), "INVALID_DEVICE");
}

/// Nucleoid creates a standard built-in object as a property of a local variable
#[rustfmt::skip]
#[test]
fn creates_a_standard_built_in_object_as_a_property_of_a_local_variable() {
    let mut run = runner();
    run(r#"class Shipment:
    pass"#);
    run(r#"{
    shipment = Shipment()
    shipment.date = Date("2019-1-3")
    shipment1 = shipment
}"#);
    assert_eq!(run("shipment1.date.toDateString()"), "Thu Jan 03 2019");
}

/// Nucleoid creates a property of a local variable in a different scope
#[rustfmt::skip]
#[test]
fn creates_a_property_of_a_local_variable_in_a_different_scope() {
    let mut run = runner();
    run(r#"class User:
    pass"#);
    run("user0 = User()");
    run(r#"{
    user = User["user0"]
    if user:
        user.name = "TEST"
}"#);
    assert_eq!(run("user0.name"), "TEST");
}

/// Nucleoid assigns a variable declaratively
#[rustfmt::skip]
#[test]
fn assigns_a_variable_declaratively() {
    let mut run = runner();
    run("a = 1");
    run("b = 2");
    run("c = a + b");
    assert_eq!(run("c"), 3);
    run("a = 2");
    assert_eq!(run("c"), 4);
}

/// Nucleoid creates if statement of variable
#[rustfmt::skip]
#[test]
fn creates_if_statement_of_variable() {
    let mut run = runner();
    run("m = false");
    run("n = false");
    run(r#"if m == true:
    n = m and true"#);
    assert_eq!(run("n"), false);
    run("m = true");
    assert_eq!(run("n"), true);
}

/// Nucleoid updates if block of variable
#[rustfmt::skip]
#[test]
fn updates_if_block_of_variable() {
    let mut run = runner();
    run("p = 0.01");
    run("s = 0.02");
    run(r#"if p < 1:
    r = p * 10"#);
    run(r#"if p < 1:
    r = s * 10"#);
    assert_eq!(run("r"), 0.2);
    run("s = 0.03");
    assert_eq!(run("r"), 0.3);
}

/// Nucleoid creates else if statement of variable
#[rustfmt::skip]
#[test]
fn creates_else_if_statement_of_variable() {
    let mut run = runner();
    run("g = 11");
    run("earth = 9.8");
    run("mars = 3.71");
    run("mass = 10");
    run(r#"if g > 9:
    weight = earth * mass
else if g > 3:
    weight = mars * mass"#);
    run("g = 5");
    assert_eq!(run("weight"), 37.1);
    run("mars = 3.72");
    assert_eq!(run("weight"), 37.2);
}

/// Nucleoid creates multiple else if statement of variable
#[rustfmt::skip]
#[test]
fn creates_multiple_else_if_statement_of_variable() {
    let mut run = runner();
    run("fraction = -0.1");
    run("point = 1");
    run(r#"if fraction > 1:
    score = fraction * point * 3
else if fraction > 0:
    score = fraction * point * 2
else:
    score = fraction * point"#);
    assert_eq!(run("score"), -0.1);
    run("point = 2");
    assert_eq!(run("score"), -0.2);
}

/// Nucleoid runs dependent statements in the same transaction
#[rustfmt::skip]
#[test]
fn runs_dependent_statements_in_the_same_transaction() {
    let mut run = runner();
    run(r#"class Vehicle:
    pass"#);
    run(r#"$Vehicle.tag = "US-" + $Vehicle.plate"#);
    run("vehicle1 = Vehicle()");
    run(r#"vehicle1.plate = "XSJ422""#);
    assert_eq!(run("vehicle1.tag"), "US-XSJ422");
}

/// Nucleoid runs dependencies in order as received
#[rustfmt::skip]
#[test]
fn runs_dependencies_in_order_as_received() {
    let mut run = runner();
    run("any = 0");
    run(r#"if any > 1:
    result = 1"#);
    run(r#"if any > 2:
    result = 2"#);
    run(r#"if any > 3:
    result = 3"#);
    run(r#"if any > 2:
    result = 4"#);
    run(r#"if any > 1:
    result = 5"#);
    run("any = 4");
    assert_eq!(run("result"), 5);
}

/// Nucleoid searches a variable in scope before the state
#[rustfmt::skip]
#[test]
fn searches_a_variable_in_scope_before_the_state() {
    let mut run = runner();
    run("e = 2.71828");
    run("number = null");
    run(r#"{
    e = 3
    number = e
}"#);
    assert_eq!(run("number"), 3);
}

/// Nucleoid uses local variable at lowest scope as priority
#[rustfmt::skip]
#[test]
fn uses_local_variable_at_lowest_scope_as_priority() {
    let mut run = runner();
    run("pi = 3.14");
    run("number = pi");
    run(r#"{
    pi = 3.141
    number = pi
}"#);
    assert_eq!(run("number"), 3.141);
}

/// Nucleoid assigns undefined if any dependency in expression is undefined
#[rustfmt::skip]
#[test]
fn assigns_undefined_if_any_dependency_in_expression_is_undefined() {
    let mut run = runner();
    run(r#"class Person:
    pass"#);
    run("person1 = Person()");
    run(r#"person1.lastName = "Brown""#);
    run(r#"person1.fullName = person1.firstName + " " + person1.lastName"#);
    assert_eq!(run("person1.fullName"), serde_json::Value::Null);
}

/// Nucleoid keeps as null if any dependencies as in local is null
#[rustfmt::skip]
#[test]
fn keeps_as_null_if_any_dependencies_as_in_local_is_null() {
    let mut run = runner();
    run("a = 1");
    run("c = null");
    run(r#"{
    b = null
    c = b / a
}"#);
    assert_eq!(run("c"), serde_json::Value::Null);
}

/// Nucleoid keeps as null if any dependencies in expression is null
#[rustfmt::skip]
#[test]
fn keeps_as_null_if_any_dependencies_in_expression_is_null() {
    let mut run = runner();
    run(r#"class Schedule:
    pass"#);
    run("schedule1 = Schedule()");
    run(r#"schedule1.expression = "0 */2 * * *""#);
    run("schedule1.script = null");
    run(r#"schedule1.run = schedule1.expression + " " + schedule1.script"#);
    assert_eq!(run("schedule1.run"), serde_json::Value::Null);
}

/// Nucleoid assigns null if there is null pointer in expression
#[rustfmt::skip]
#[test]
fn assigns_null_if_there_is_null_pointer_in_expression() {
    let mut run = runner();
    run(r#"class Product:
    pass"#);
    run("product1 = Product()");
    run("score = product1.quality.score");
    assert_eq!(run("score"), serde_json::Value::Null);
}

/// Nucleoid assigns a unique variable for an instance without a variable name
#[rustfmt::skip]
#[test]
fn assigns_a_unique_variable_for_an_instance_without_a_variable_name() {
    let mut run = runner();
    run(r#"class Vehicle:
    pass"#);
    run("Vehicle()");
    assert_eq!(run("Vehicle.length"), 1);
    assert_eq!(run("Vehicle[0].id != null"), true);
}

/// Nucleoid creates a function in state
#[rustfmt::skip]
#[test]
fn creates_a_function_in_state() {
    let mut run = runner();
    run(r#"def generate(number):
    return number * 10"#);
    run("random = 10");
    run("number = generate(random)");
    assert_eq!(run("number"), 100);
    run("random = 20");
    assert_eq!(run("number"), 200);
}

/// Nucleoid assigns a function as a dependency
#[rustfmt::skip]
#[test]
fn assigns_a_function_as_a_dependency() {
    let mut run = runner();
    run("list = []");
    run("count = list.filter(n => n % 2)");
    run("list.push(1)");
    assert_eq!(run("count.length"), 1);
    run("list.push(2)");
    assert_eq!(run("count.length"), 1);
    run("list.push(3)");
    assert_eq!(run("count.length"), 2);
    run("list.pop()");
    assert_eq!(run("count.length"), 1);
}

/// Nucleoid supports a regular expression literal
#[rustfmt::skip]
#[test]
fn supports_a_regular_expression_literal() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class User:
    pass"#);
    run(r#"if not /.{4,8}/.test($User.password):
    throw 'INVALID_PASSWORD'"#);
    run("user1 = User()");
    assert_eq!(run("user1.password"), serde_json::Value::Null);
    assert_eq!(run_error("user1.password = 'PAS'"), "INVALID_PASSWORD");
}

/// Nucleoid rejects defining a class declaration in a non-class block
#[rustfmt::skip]
#[test]
fn rejects_defining_a_class_declaration_in_a_non_class_block() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Person:
    pass"#);
    run("person1 = Person()");
    run("person1.weight = 90");
    run("person1.height = 1.8");
    assert_eq!(run_error(r#"{
    weight = person1.weight
    height = person1.height
    $Person.bmi = weight / (height * height)
}"#), "SyntaxError: Cannot define class declaration in non-class block");
}

/// Nucleoid detects a circular dependency
#[rustfmt::skip]
#[test]
fn detects_a_circular_dependency() {
    let (mut run, mut run_error) = crate::common::runners();
    run("number1 = 10");
    run("number2 = number1 * 10");
    assert_eq!(run("number2"), 100);
    assert_eq!(run_error("number1 = number2 * 10"), "TypeError: Circular Dependency");
}

/// Nucleoid rolls back a variable if an exception is thrown
#[rustfmt::skip]
#[test]
fn rolls_back_a_variable_if_an_exception_is_thrown() {
    let (mut run, mut run_error) = crate::common::runners();
    run("a = 5");
    run(r#"if a > 5:
    throw 'INVALID_VALUE'"#);
    assert_eq!(run_error("a = 6"), "INVALID_VALUE");
    assert_eq!(run("a"), 5);
}

/// Nucleoid rolls back a property if an exception is thrown
#[rustfmt::skip]
#[test]
fn rolls_back_a_property_if_an_exception_is_thrown() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Item:
    pass"#);
    run(r#"if $Item.sku == 'A':
    throw 'INVALID_SKU'"#);
    run("item1 = Item()");
    assert_eq!(run_error("item1.sku = 'A'"), "INVALID_SKU");
    assert_eq!(run("item1.sku"), serde_json::Value::Null);
}

/// Nucleoid rolls back an instance if an exception is thrown
#[rustfmt::skip]
#[test]
fn rolls_back_an_instance_if_an_exception_is_thrown() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class User(first: str, last: str):
    this.first = first
    this.last = last"#);
    run(r#"if $User.first.length < 3:
    throw 'INVALID_USER'"#);
    assert_eq!(run_error("user1 = User('F', 'L')"), "INVALID_USER");
    assert_eq!(run("User.length"), 0);
    assert_eq!(run_error("user1"), "ReferenceError: user1 is not defined");
}

/// Nucleoid updates a variable assignment
#[rustfmt::skip]
#[test]
fn updates_a_variable_assignment() {
    let mut run = runner();
    run("a = 1");
    run("b = 2");
    run("c = a + 3");
    assert_eq!(run("c"), 4);
    run("c = b + 3");
    assert_eq!(run("c"), 5);
    run("b = 4");
    assert_eq!(run("c"), 7);
}

/// Nucleoid uses only the value when a variable references itself
#[rustfmt::skip]
#[test]
fn uses_only_the_value_when_a_variable_references_itself() {
    let mut run = runner();
    run("radius = 10");
    run("radius = radius + 10");
    assert_eq!(run("radius"), 20);
}

/// Nucleoid deletes a variable assignment
#[rustfmt::skip]
#[test]
fn deletes_a_variable_assignment() {
    let (mut run, mut run_error) = crate::common::runners();
    run("t = 1");
    run("q = t + 1");
    assert_eq!(run("q"), 2);
    run("delete q");
    run("t = 2");
    assert_eq!(run_error("q"), "ReferenceError: q is not defined");
}

/// Nucleoid returns the assigned value in a variable assignment
#[rustfmt::skip]
#[test]
fn returns_the_assigned_value_in_a_variable_assignment() {
    let mut run = runner();
    run(r#"x = 1

# return: 1"#);
}

/// Nucleoid assigns a parameter in a function as a dependency
#[rustfmt::skip]
#[test]
fn assigns_a_parameter_in_a_function_as_a_dependency() {
    let mut run = runner();
    run(r#"str1 = "ABC""#);
    run(r#"str2 = str1.lower() + "d""#);
    run("str3 = str2 + str1");
    assert_eq!(run("str2"), "abcd");
    assert_eq!(run("str3"), "abcdABC");
    run(r#"str1 = "AAA""#);
    assert_eq!(run("str2"), "aaad");
    assert_eq!(run("str3"), "aaadAAA");
}

/// Nucleoid uses value property to indicate using only value of variable
#[rustfmt::skip]
#[test]
fn uses_value_property_to_indicate_using_only_value_of_variable() {
    let mut run = runner();
    run("goldenRatio = 1.618");
    run("altitude = 10");
    run("width = goldenRatio.value * altitude");
    run("depth = goldenRatio.value * altitude");
    assert_eq!(run("width"), 16.18);
    assert_eq!(run("depth"), 16.18);
    run("goldenRatio = 1.62");
    assert_eq!(run("width"), 16.18);
    assert_eq!(run("depth"), 16.18);
    run("altitude = 100");
    assert_eq!(run("width"), 161.8);
    assert_eq!(run("depth"), 161.8);
}

/// Nucleoid creates a nested object in a block and assigns it to a class-level property before instantiation
#[rustfmt::skip]
#[test]
fn creates_a_nested_object_in_a_block_and_assigns_it_to_a_class_level_property_before_instantiation() {
    let mut run = runner();
    run(r#"class Account:
    pass"#);
    run(r#"class Currency:
    pass"#);
    run(r#"$Currency.description = "Code:" + $Currency.code"#);
    run(r#"{
    balance = Object()
    balance.currency = Object()
    balance.currency.code = "USD"
    $Account.balance = balance
}"#);
    run("account1 = Account()");
    assert_eq!(run("account1.balance.currency.code"), "USD");
    assert_eq!(run("account1.balance.currency.description"), serde_json::Value::Null);
}

/// Nucleoid creates a nested object in a block and assigns it to a class-level property after instantiation
#[rustfmt::skip]
#[test]
fn creates_a_nested_object_in_a_block_and_assigns_it_to_a_class_level_property_after_instantiation() {
    let mut run = runner();
    run(r#"class Warehouse:
    pass"#);
    run("warehouse1 = Warehouse()");
    run(r#"{
    inventory = Object()
    inventory.item = Object()
    inventory.item.sku = "699546085767"
    $Warehouse.inventory = inventory
}"#);
    assert_eq!(run("warehouse1.inventory.item.sku"), "699546085767");
    assert_eq!(run("warehouse1.inventory.item.description"), serde_json::Value::Null);
}

/// Nucleoid creates an instance inside a block
#[rustfmt::skip]
#[test]
fn creates_an_instance_inside_a_block() {
    let mut run = runner();
    run(r#"class Device(name: str):
    this.name = name"#);
    run("$Device.deleted = false");
    run(r#"$Device.key = "X-" + $Device.name"#);
    run(r#"{
    name = "Hall"
    device1 = Device(name)
}"#);
    assert_eq!(run("device1.name"), "Hall");
    assert_eq!(run("device1.key"), "X-Hall");
    assert_eq!(run("device1.deleted"), false);
}

/// Nucleoid creates an instance inside a block without a variable name
#[rustfmt::skip]
#[test]
fn creates_an_instance_inside_a_block_without_a_variable_name() {
    let mut run = runner();
    run(r#"class Summary(rate: int):
    this.rate = rate"#);
    run("$Summary.score = $Summary.rate * 100");
    run(r#"{
    rate = 4
    Summary(rate)
}"#);
    assert_eq!(run("Summary[0].rate"), 4);
    assert_eq!(run("Summary[0].score"), 400);
}

/// Nucleoid creates a local variable inside a block
#[rustfmt::skip]
#[test]
fn creates_a_local_variable_inside_a_block() {
    let mut run = runner();
    run("a = 5");
    run("b = 10");
    run(r#"if a > 9:
    c = a + b
    d = c * 10"#);
    run("a = 10");
    assert_eq!(run("d"), 200);
    run("a = 15");
    assert_eq!(run("d"), 250);
    run("b = 20");
    assert_eq!(run("d"), 350);
}

/// Nucleoid runs a local variable as an object before declaration
#[rustfmt::skip]
#[test]
fn runs_a_local_variable_as_an_object_before_declaration() {
    let mut run = runner();
    run(r#"class Plane:
    pass"#);
    run(r#"class Trip:
    pass"#);
    run("plane1 = Plane()");
    run("plane1.speed = 903");
    run("trip1 = Trip()");
    run("trip1.distance = 5540");
    run(r#"{
    trip = $Plane.trip
    $Plane.time = trip.distance / $Plane.speed
}"#);
    run("plane1.trip = trip1");
    assert_eq!(run("plane1.time"), 6.135105204872647);
}

/// Nucleoid runs a local variable as an object after declaration
#[rustfmt::skip]
#[test]
fn runs_a_local_variable_as_an_object_after_declaration() {
    let mut run = runner();
    run(r#"class Seller:
    pass"#);
    run(r#"class Commission:
    pass"#);
    run("seller1 = Seller()");
    run("seller1.sales = 1000000");
    run("comm1 = Commission()");
    run("comm1.rate = 0.05");
    run("seller1.commission = comm1");
    run(r#"{
    commission = $Seller.commission
    $Seller.pay = $Seller.sales * commission.rate
}"#);
    assert_eq!(run("seller1.pay"), 50000);
}

/// Nucleoid assigns a property on a local variable after initialization
#[rustfmt::skip]
#[test]
fn assigns_a_property_on_a_local_variable_after_initialization() {
    let mut run = runner();
    run(r#"class Stock:
    pass"#);
    run(r#"class Trade:
    pass"#);
    run("stock1 = Stock()");
    run("stock1.price = 100");
    run("trade1 = Trade()");
    run("trade1.quantity = 1");
    run("stock1.trade = trade1");
    run(r#"{
    trade = $Stock.trade
    trade.worth = $Stock.price * trade.quantity
}"#);
    assert_eq!(run("trade1.worth"), 100);
}

/// Nucleoid reassigns a shadowing local variable in a nested block
#[rustfmt::skip]
#[test]
fn reassigns_a_shadowing_local_variable_in_a_nested_block() {
    let mut run = runner();
    run(r#"barcode = "barcode""#);
    assert_eq!(run(r#"{
    barcode = "barcode"
    {
        barcode = "barcode2"
        {
            barcode
        }
    }
}"#), "barcode2");
    assert_eq!(run("barcode"), "barcode");
}

/// Nucleoid holds the result of a function in a local variable
#[rustfmt::skip]
#[test]
fn holds_the_result_of_a_function_in_a_local_variable() {
    let mut run = runner();
    run("bugs = []");
    run("ticket = 1");
    run(r#"class Bug:
    pass"#);
    run("bug1 = Bug()");
    run("bug1.ticket = 1");
    run(r#"bug1.priority = "LOW""#);
    run("bugs.push(bug1)");
    run("bug2 = Bug()");
    run("bug2.ticket = 2");
    run(r#"bug2.priority = "MEDIUM""#);
    run("bugs.push(bug2)");
    run(r#"{
    bug = bugs.find(b => b.ticket == ticket)
    bug.selected = true
}"#);
    assert_eq!(run("bug1.selected"), true);
    assert_eq!(run("bug2.selected"), serde_json::Value::Null);
    run("ticket = 2");
    assert_eq!(run("bug2.selected"), true);
}

/// Nucleoid runs a block statement of variable
#[rustfmt::skip]
#[test]
fn runs_a_block_statement_of_variable() {
    let mut run = runner();
    run("h = 1");
    run(r#"{
    value = h * 2
    j = value * 2
}"#);
    assert_eq!(run("j"), 4);
    run("h = 2");
    assert_eq!(run("j"), 8);
}

/// Nucleoid runs a nested block statement of variable
#[rustfmt::skip]
#[test]
fn runs_a_nested_block_statement_of_variable() {
    let mut run = runner();
    run("radius = 10");
    run(r#"{
    area = Math.pow(radius, 2) * 3.14
    {
        volume = area * 5
    }
}"#);
    assert_eq!(run("volume"), 1570);
}

/// Nucleoid runs a nested if statement of variable
#[rustfmt::skip]
#[test]
fn runs_a_nested_if_statement_of_variable() {
    let mut run = runner();
    run("gravity = 9.8");
    run("time = 10");
    run("distance = 480");
    run("target = true");
    run(r#"{
    dist = 1 / 2 * gravity * time * time
    if dist > distance:
        hit = target
}"#);
    assert_eq!(run("hit"), true);
    run("target = false");
    assert_eq!(run("hit"), false);
}

/// Nucleoid runs a nested else statement of variable
#[rustfmt::skip]
#[test]
fn runs_a_nested_else_statement_of_variable() {
    let mut run = runner();
    run("percentage = 28");
    run("density = 0.899");
    run(r#"substance = "NH3""#);
    run("molarConcentration = null");
    run("fallback = 0");
    run(r#"{
    concentration = percentage * density / 100 * 1000
    if substance == "NH3":
        molarConcentration = concentration / 17.04
    else:
        molarConcentration = fallback
}"#);
    run(r#"substance = "NH16""#);
    run("fallback = 1");
    assert_eq!(run("molarConcentration"), 1);
}

/// Nucleoid assigns a variable to a reference
#[rustfmt::skip]
#[test]
fn assigns_a_variable_to_a_reference() {
    let mut run = runner();
    run("a = 1");
    run("b = a");
    assert_eq!(run("b"), 1);
    run("a = 2");
    assert_eq!(run("b"), 2);
}

/// Nucleoid assigns an object to a variable
#[rustfmt::skip]
#[test]
fn assigns_an_object_to_a_variable() {
    let mut run = runner();
    run(r#"class Model:
    pass"#);
    run("model1 = Model()");
    {
        let actual = run("typeof model1");
        let expected = run("(Object)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid defines a class in the state
#[rustfmt::skip]
#[test]
fn defines_a_class_in_the_state() {
    let mut run = runner();
    run(r#"class Entity:
    pass"#);
    {
        let actual = run("typeof $Entity");
        let expected = run("(Class)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid rejects creating an instance if the class does not exist
#[rustfmt::skip]
#[test]
fn rejects_creating_an_instance_if_the_class_does_not_exist() {
    let (mut run, mut run_error) = crate::common::runners();
    assert_eq!(run_error("chart1 = Chart()"), "ReferenceError: Chart is not defined");
    run(r#"class Chart:
    pass"#);
    run("chart1 = Chart()");
    assert_eq!(run_error("chart1.plot = Plot()"), "ReferenceError: Plot is not defined");
    assert_eq!(run_error("$Chart.plot = Plot()"), "ReferenceError: Plot is not defined");
}

/// Nucleoid creates a property assignment before declaration
#[rustfmt::skip]
#[test]
fn creates_a_property_assignment_before_declaration() {
    let mut run = runner();
    run(r#"class Order:
    pass"#);
    run("order1 = Order()");
    run(r#"order1.upc = "04061" + order1.barcode"#);
    assert_eq!(run("order1.upc"), serde_json::Value::Null);
    run(r#"order1.barcode = "94067""#);
    assert_eq!(run("order1.upc"), "0406194067");
}

/// Nucleoid creates a property assignment after declaration
#[rustfmt::skip]
#[test]
fn creates_a_property_assignment_after_declaration() {
    let mut run = runner();
    run(r#"class User:
    pass"#);
    run("user1 = User()");
    run(r#"user1.name = "sample""#);
    run(r#"user1.email = user1.name + "@example.com""#);
    assert_eq!(run("user1.email"), "sample@example.com");
    run(r#"user1.name = "samplex""#);
    assert_eq!(run("user1.email"), "samplex@example.com");
}

/// Nucleoid creates a property assignment only if the instance is defined
#[rustfmt::skip]
#[test]
fn creates_a_property_assignment_only_if_the_instance_is_defined() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Channel:
    pass"#);
    run("channel1 = Channel()");
    assert_eq!(run_error(r#"channel1.frequency.type = "ANGULAR""#), "ReferenceError: channel1.frequency is not defined");
}

/// Nucleoid creates an object and assigns it to a variable
#[rustfmt::skip]
#[test]
fn creates_an_object_and_assigns_it_to_a_variable() {
    let mut run = runner();
    run(r#"class Item(name: str):
    this.name = name"#);
    run(r#"item1 = Item("NAME-1")"#);
    assert_eq!(run("item1"), serde_json::json!({ "id": "item1", "name": "NAME-1" }));
    run("item2 = Item()");
    assert_eq!(run("item2"), serde_json::json!({ "id": "item2", "name": null }));
}

/// Nucleoid creates an object assignment as a property only if the instance is defined
#[rustfmt::skip]
#[test]
fn creates_an_object_assignment_as_a_property_only_if_the_instance_is_defined() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Worker:
    pass"#);
    run(r#"class Schedule:
    pass"#);
    run("worker1 = Worker()");
    assert_eq!(run_error("worker1.duty.schedule = Schedule()"), "ReferenceError: worker1.duty is not defined");
}

/// Nucleoid uses only the value when a property references itself
#[rustfmt::skip]
#[test]
fn uses_only_the_value_when_a_property_references_itself() {
    let mut run = runner();
    run(r#"class Construction:
    pass"#);
    run("construction1 = Construction()");
    run("construction1.timeline = 120");
    run("construction1.timeline = 2 * construction1.timeline");
    assert_eq!(run("construction1.timeline"), 240);
}

/// Nucleoid assigns an object to a property before initialization
#[rustfmt::skip]
#[test]
fn assigns_an_object_to_a_property_before_initialization() {
    let mut run = runner();
    run(r#"class Agent:
    pass"#);
    run(r#"class Distance:
    pass"#);
    run("$Distance.total = Math.sqrt($Distance.x * $Distance.x + $Distance.y * $Distance.y)");
    run("agent1 = Agent()");
    run("agent1.distance = Distance()");
    assert_eq!(run("agent1.distance.total"), serde_json::Value::Null);
    run("agent1.distance.x = 3");
    run("agent1.distance.y = 4");
    assert_eq!(run("agent1.distance.total"), 5);
}

/// Nucleoid assigns an object to a property after initialization
#[rustfmt::skip]
#[test]
fn assigns_an_object_to_a_property_after_initialization() {
    let mut run = runner();
    run(r#"class Product:
    pass"#);
    run("product1 = Product()");
    run(r#"class Quality:
    pass"#);
    run("product1.quality = Quality()");
    run("product1.quality.score = 15");
    assert_eq!(run("product1.quality.class"), serde_json::Value::Null);
    run("$Quality.class = String.fromCharCode(65 + Math.floor($Quality.score / 10))");
    assert_eq!(run("product1.quality.class"), "B");
}

/// Nucleoid rejects value as a property name
#[rustfmt::skip]
#[test]
fn rejects_value_as_a_property_name() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Schedule:
    pass"#);
    run(r#"class Place:
    pass"#);
    run("value = Schedule()");
    assert_eq!(run("value"), serde_json::json!({ "id": "value" }));
    assert_eq!(run_error("value.value = Place()"), "TypeError: Cannot use 'value' as a property");
}

/// Nucleoid rejects value as a property name in a value assignment
#[rustfmt::skip]
#[test]
fn rejects_value_as_a_property_name_in_a_value_assignment() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Value:
    pass"#);
    run("value = Value()");
    assert_eq!(run("value"), serde_json::json!({ "id": "value" }));
    assert_eq!(run_error("value.value = 2147483647"), "TypeError: Cannot use 'value' as a property");
}

/// Nucleoid uses value property to indicate using only value of property
#[rustfmt::skip]
#[test]
fn uses_value_property_to_indicate_using_only_value_of_property() {
    let mut run = runner();
    run(r#"class Weight:
    pass"#);
    run("weight1 = Weight()");
    run("weight1.gravity = 1.352");
    run("weight1.mass = 1000");
    run("weight1.force = weight1.gravity * weight1.mass.value");
    assert_eq!(run("weight1.force"), 1352);
    run("weight1.mass = 2000");
    assert_eq!(run("weight1.force"), 1352);
    run("weight1.gravity = 2");
    assert_eq!(run("weight1.force"), 2000);
}

/// Nucleoid uses value property in an if condition to indicate using only value of property
#[rustfmt::skip]
#[test]
fn uses_value_property_in_an_if_condition_to_indicate_using_only_value_of_property() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Question:
    pass"#);
    run("question1 = Question()");
    run(r#"question1.text = "How was the service?""#);
    run(r#"if question1.text != question1.text.value:
    throw "QUESTION_ARCHIVED""#);
    assert_eq!(run("question1.text"), "How was the service?");
    assert_eq!(run_error(r#"question1.text = "How would you rate us?""#), "QUESTION_ARCHIVED");
}

/// Nucleoid rejects value of a property if the property is not defined
#[rustfmt::skip]
#[test]
fn rejects_value_of_a_property_if_the_property_is_not_defined() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Travel:
    pass"#);
    run("travel1 = Travel()");
    run("travel1.speed = 65");
    run("travel1.duration = travel1.distance / travel1.speed");
    assert_eq!(run("travel1.duration"), serde_json::Value::Null);
    assert_eq!(run_error("travel1.time = travel1.distance.value / travel1.speed"), "ReferenceError: travel1.distance is not defined");
}

/// Nucleoid uses the value of a null property as zero
#[rustfmt::skip]
#[test]
fn uses_the_value_of_a_null_property_as_zero() {
    let mut run = runner();
    run(r#"class Interest:
    pass"#);
    run("interest1 = Interest()");
    run("interest1.rate = 3");
    run("interest1.amount = null");
    run("interest1.annual = interest1.rate * interest1.amount.value / 100");
    assert_eq!(run("interest1.annual"), 0);
    run("interest1.amount = 10000");
    assert_eq!(run("interest1.annual"), 0);
}

/// Nucleoid rejects value as a property name in a block
#[rustfmt::skip]
#[test]
fn rejects_value_as_a_property_name_in_a_block() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Alarm:
    pass"#);
    assert_eq!(run_error(r#"{
    value = Alarm()
    value.value = "22:00"
}"#), "TypeError: Cannot use 'value' as a property");
}

/// Nucleoid keeps same as its value when the value property is used for a local
#[rustfmt::skip]
#[test]
fn keeps_same_as_its_value_when_the_value_property_is_used_for_a_local() {
    let mut run = runner();
    run("speedOfLight = 299792");
    run("roundTrip = null");
    run(r#"{
    time = speedOfLight / 225623
    roundTrip = time.value * 2
}"#);
    assert_eq!(run("roundTrip"), 2.6574595675086314);
}

/// Nucleoid uses value property in a class-level assignment
#[rustfmt::skip]
#[test]
fn uses_value_property_in_a_class_level_assignment() {
    let mut run = runner();
    run("count = 0");
    run(r#"class Device:
    pass"#);
    run("device1 = Device()");
    run(r#"{
    $Device.code = "A" + count.value
    count = count + 1
}"#);
    assert_eq!(run("device1.code"), "A0");
}

/// Nucleoid uses value property on a class-level property chain
#[rustfmt::skip]
#[test]
fn uses_value_property_on_a_class_level_property_chain() {
    let mut run = runner();
    run(r#"class Summary(question):
    this.question = question"#);
    run(r#"class Question:
    pass"#);
    run("$Summary.count = $Summary.question.count.value");
    run("question1 = Question()");
    run("question1.count = 10");
    run("summary1 = Summary(question1)");
    assert_eq!(run("summary1.count"), 10);
    run("question1.count = 11");
    assert_eq!(run("question1.count"), 11);
    assert_eq!(run("summary1.count"), 10);
}

/// Nucleoid updates if block of property
#[rustfmt::skip]
#[test]
fn updates_if_block_of_property() {
    let mut run = runner();
    run(r#"class Account:
    pass"#);
    run("account1 = Account()");
    run("account1.balance = 1000");
    run(r#"if account1.balance < 1500:
    account1.status = "OK""#);
    assert_eq!(run("account1.status"), "OK");
    run(r#"if account1.balance < 1500:
    account1.status = "LOW""#);
    assert_eq!(run("account1.status"), "LOW");
}

/// Nucleoid creates an else statement of variable
#[rustfmt::skip]
#[test]
fn creates_an_else_statement_of_variable() {
    let mut run = runner();
    run("compound = 0.0001");
    run("acidic = 'ACIDIC'");
    run("basic = 'BASIC'");
    run(r#"if compound > 0.0000001:
    pH = acidic
else:
    pH = basic"#);
    assert_eq!(run("pH"), "ACIDIC");
    run("compound = 0.000000001");
    assert_eq!(run("pH"), "BASIC");
    run("basic = '+7'");
    assert_eq!(run("pH"), "+7");
}

/// Nucleoid creates if statement of property
#[rustfmt::skip]
#[test]
fn creates_if_statement_of_property() {
    let mut run = runner();
    run(r#"class Toy:
    pass"#);
    run("toy1 = Toy()");
    run(r#"toy1.color = "BLUE""#);
    run(r#"if toy1.color == "RED":
    toy1.shape = "CIRCLE""#);
    assert_eq!(run("toy1.shape"), serde_json::Value::Null);
    run(r#"toy1.color = "RED""#);
    assert_eq!(run("toy1.shape"), "CIRCLE");
}

/// Nucleoid creates else statement of property
#[rustfmt::skip]
#[test]
fn creates_else_statement_of_property() {
    let mut run = runner();
    run(r#"class Engine:
    pass"#);
    run("engine1 = Engine()");
    run(r#"engine1.type = "V8""#);
    run(r#"mpl = "MPL""#);
    run(r#"bsd = "BSD""#);
    run(r#"if engine1.type == "Gecko":
    engine1.license = mpl
else:
    engine1.license = bsd"#);
    assert_eq!(run("engine1.license"), "BSD");
    run(r#"bsd = "Berkeley Software Distribution""#);
    assert_eq!(run("engine1.license"), "Berkeley Software Distribution");
    run(r#"engine1.type = "Gecko""#);
    assert_eq!(run("engine1.license"), "MPL");
}

/// Nucleoid creates else statement of property with property dependencies
#[rustfmt::skip]
#[test]
fn creates_else_statement_of_property_with_property_dependencies() {
    let mut run = runner();
    run(r#"class Contact:
    pass"#);
    run("contact1 = Contact()");
    run(r#"contact1.type = "PERSON""#);
    run(r#"contact1.first = "First""#);
    run(r#"contact1.last = "Last""#);
    run(r#"if contact1.type == "BUSINESS":
    contact1.full = "B" + contact1.first
else:
    contact1.full = contact1.first + " " + contact1.last"#);
    assert_eq!(run("contact1.full"), "First Last");
    run(r#"contact1.first = "F""#);
    run(r#"contact1.last = "L""#);
    assert_eq!(run("contact1.full"), "F L");
    run(r#"contact1.type = "BUSINESS""#);
    assert_eq!(run("contact1.full"), "BF");
}

/// Nucleoid creates multiple else if statement of property
#[rustfmt::skip]
#[test]
fn creates_multiple_else_if_statement_of_property() {
    let mut run = runner();
    run(r#"class Taxpayer:
    pass"#);
    run("taxpayer1 = Taxpayer()");
    run("taxpayer1.income = 60000");
    run("taxpayer1.member = 1");
    run("rate = 22");
    run(r#"if taxpayer1.member > 4:
    taxpayer1.tax = taxpayer1.income * rate / 100 - 2000
else if taxpayer1.member > 2:
    taxpayer1.tax = taxpayer1.income * rate / 100 - 1000
else:
    taxpayer1.tax = taxpayer1.income * rate / 100"#);
    assert_eq!(run("taxpayer1.tax"), 13200);
    run("rate = 23");
    assert_eq!(run("taxpayer1.tax"), 13800);
    run("taxpayer1.member = 3");
    assert_eq!(run("taxpayer1.tax"), 12800);
    run("taxpayer1.member = 5");
    assert_eq!(run("taxpayer1.tax"), 11800);
}

/// Nucleoid updates property assignment
#[rustfmt::skip]
#[test]
fn updates_property_assignment() {
    let mut run = runner();
    run(r#"class Matter:
    pass"#);
    run("matter1 = Matter()");
    run("matter1.mass = 10");
    run("matter1.weight = matter1.mass * 9.8");
    assert_eq!(run("matter1.weight"), 98);
    run("matter1.weight = matter1.mass * 3.7");
    assert_eq!(run("matter1.weight"), 37);
    run("matter1.mass = 20");
    assert_eq!(run("matter1.weight"), 74);
}

/// Nucleoid deletes an instance
#[rustfmt::skip]
#[test]
fn deletes_an_instance() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Circle:
    pass"#);
    run("circle1 = Circle()");
    run("delete circle1");
    assert_eq!(run(r#"Circle["circle1"]"#), serde_json::Value::Null);
    assert_eq!(run(r#"Circle.find(circle => circle.id == "circle1")"#), serde_json::Value::Null);
    assert_eq!(run_error("circle1"), "ReferenceError: circle1 is not defined");
}

/// Nucleoid deletes an instance by reference
#[rustfmt::skip]
#[test]
fn deletes_an_instance_by_reference() {
    let mut run = runner();
    run(r#"class Item:
    pass"#);
    run("item1 = Item()");
    run("item2 = Item()");
    assert_eq!(run(r#"Item["item1"]"#), serde_json::json!({ "id": "item1" }));
    run(r#"delete Item["item1"]"#);
    assert_eq!(run(r#"Item["item1"]"#), serde_json::Value::Null);
    assert_eq!(run(r#"Item["item2"]"#), serde_json::json!({ "id": "item2" }));
    run(r#"{
    item = "item2"
    delete Item[item]
}"#);
    assert_eq!(run(r#"Item["item2"]"#), serde_json::Value::Null);
}

/// Nucleoid returns a boolean when deleting an object
#[rustfmt::skip]
#[test]
fn returns_a_boolean_when_deleting_an_object() {
    let mut run = runner();
    run(r#"class Location:
    pass"#);
    run("location1 = Location()");
    assert_eq!(run("delete location1"), true);
    assert_eq!(run("delete location2"), false);
}

/// Nucleoid rejects deleting an instance if it has any properties
#[rustfmt::skip]
#[test]
fn rejects_deleting_an_instance_if_it_has_any_properties() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Channel:
    pass"#);
    run("channel1 = Channel()");
    run("channel1.frequency = 440");
    assert_eq!(run_error("delete channel1"), "TypeError: Cannot delete object 'channel1'");
    assert_eq!(run("channel1.frequency"), 440);
    run("delete channel1.frequency");
    run("delete channel1");
    assert_eq!(run(r#"Channel["channel1"]"#), serde_json::Value::Null);
}

/// Nucleoid rejects deleting an instance if it has an object as a property
#[rustfmt::skip]
#[test]
fn rejects_deleting_an_instance_if_it_has_an_object_as_a_property() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Shape:
    pass"#);
    run(r#"class Type:
    pass"#);
    run("shape1 = Shape()");
    run("shape1.type = Type()");
    assert_eq!(run_error("delete shape1"), "TypeError: Cannot delete object 'shape1'");
    run("delete shape1.type");
    run("delete shape1");
    assert_eq!(run(r#"Shape["shape1"]"#), serde_json::Value::Null);
}

/// Nucleoid deletes a property assignment
#[rustfmt::skip]
#[test]
fn deletes_a_property_assignment() {
    let mut run = runner();
    run(r#"class Agent:
    pass"#);
    run("agent = Agent()");
    run("agent.time = 52926163455");
    run(r#"agent.location = "CITY""#);
    run(r#"agent.report = agent.time + "@" + agent.location"#);
    assert_eq!(run("agent.report"), "52926163455@CITY");
    run("delete agent.time");
    assert_eq!(run("agent.report"), serde_json::Value::Null);
    run("delete agent.report");
    assert_eq!(run("agent.report"), serde_json::Value::Null);
}

/// Nucleoid runs a block statement of property
#[rustfmt::skip]
#[test]
fn runs_a_block_statement_of_property() {
    let mut run = runner();
    run(r#"class Item:
    pass"#);
    run("item1 = Item()");
    run(r#"item1.sku = "0000001""#);
    run(r#"{
    custom = "US" + item1.sku
    item1.custom = custom
}"#);
    assert_eq!(run("item1.custom"), "US0000001");
    run(r#"item1.sku = "0000002""#);
    assert_eq!(run("item1.custom"), "US0000002");
}

/// Nucleoid runs a nested block statement of property
#[rustfmt::skip]
#[test]
fn runs_a_nested_block_statement_of_property() {
    let mut run = runner();
    run(r#"class Figure:
    pass"#);
    run("figure1 = Figure()");
    run("figure1.width = 9");
    run("figure1.height = 10");
    run(r#"{
    base = Math.pow(figure1.width, 2)
    {
        figure1.volume = base * figure1.height
    }
}"#);
    assert_eq!(run("figure1.volume"), 810);
    run("figure1.height = 9");
    assert_eq!(run("figure1.volume"), 729);
}

/// Nucleoid runs a nested if statement of property
#[rustfmt::skip]
#[test]
fn runs_a_nested_if_statement_of_property() {
    let mut run = runner();
    run(r#"class Sale:
    pass"#);
    run("sale1 = Sale()");
    run("sale1.price = 50");
    run("sale1.quantity = 2");
    run(r#"{
    amount = sale1.price * sale1.quantity
    if amount > 100:
        sale1.tax = amount * 10 / 100
}"#);
    assert_eq!(run("sale1.tax"), serde_json::Value::Null);
    run("sale1.quantity = 3");
    assert_eq!(run("sale1.tax"), 15);
}

/// Nucleoid creates a nested else statement of property
#[rustfmt::skip]
#[test]
fn creates_a_nested_else_statement_of_property() {
    let mut run = runner();
    run(r#"class Chart:
    pass"#);
    run("chart1 = Chart()");
    run("chart1.percentage = 1");
    run(r#"invalid = "INVALID""#);
    run(r#"valid = "VALID""#);
    run(r#"{
    ratio = chart1.percentage / 100
    if ratio > 1:
        chart1.status = invalid
    else:
        chart1.status = valid
}"#);
    assert_eq!(run("chart1.status"), "VALID");
    run(r#"valid = "V""#);
    assert_eq!(run("chart1.status"), "V");
}

/// Nucleoid creates a property assignment with multiple properties
#[rustfmt::skip]
#[test]
fn creates_a_property_assignment_with_multiple_properties() {
    let mut run = runner();
    run(r#"class Person:
    pass"#);
    run("person1 = Person()");
    run(r#"class Address:
    pass"#);
    run("address1 = Address()");
    run(r#"$Address.print = $Address.city + ", " + $Address.state"#);
    run("person1.address = Address()");
    run(r#"person1.address.city = "Syracuse""#);
    run(r#"person1.address.state = "NY""#);
    assert_eq!(run("person1.address.print"), "Syracuse, NY");
}

/// Nucleoid creates a property assignment with multiple properties as part of a declaration
#[rustfmt::skip]
#[test]
fn creates_a_property_assignment_with_multiple_properties_as_part_of_a_declaration() {
    let mut run = runner();
    run(r#"class Server:
    pass"#);
    run("server1 = Server()");
    run(r#"server1.name = "HOST1""#);
    run(r#"class IP:
    pass"#);
    run("ip1 = IP()");
    run("server1.ip = ip1");
    run(r#"ip1.address = "10.0.0.1""#);
    run(r#"server1.summary = server1.name + "@" + server1.ip.address"#);
    assert_eq!(run("server1.summary"), "HOST1@10.0.0.1");
    run(r#"ip1.address = "10.0.0.2""#);
    assert_eq!(run("server1.summary"), "HOST1@10.0.0.2");
}

/// Nucleoid creates a dependency on behalf if a property has a reference
#[rustfmt::skip]
#[test]
fn creates_a_dependency_on_behalf_if_a_property_has_a_reference() {
    let mut run = runner();
    run(r#"class Schedule:
    pass"#);
    run("schedule1 = Schedule()");
    run(r#"class Template:
    pass"#);
    run("template1 = Template()");
    run(r#"template1.type = "W""#);
    run("schedule1.template = template1");
    run(r#"schedule1.template.name = schedule1.template.type + "-0001""#);
    assert_eq!(run("template1.name"), "W-0001");
    assert_eq!(run("schedule1.template.name"), "W-0001");
    run(r#"template1.type = "D""#);
    assert_eq!(run("template1.name"), "D-0001");
    run(r#"template1.shape = template1.type + "-Form""#);
    assert_eq!(run("template1.shape"), "D-Form");
    assert_eq!(run("schedule1.template.shape"), "D-Form");
    run(r#"template1.type = "C""#);
    assert_eq!(run("template1.shape"), "C-Form");
    assert_eq!(run("schedule1.template.shape"), "C-Form");
}

/// Nucleoid creates a dependency on behalf if a local variable has a reference
#[rustfmt::skip]
#[test]
fn creates_a_dependency_on_behalf_if_a_local_variable_has_a_reference() {
    let mut run = runner();
    run(r#"class Vote:
    pass"#);
    run("vote1 = Vote()");
    run("vote1.rate = 4");
    run(r#"class Question:
    pass"#);
    run("$Question.rate = 0");
    run("$Question.count = 0");
    run("question1 = Question()");
    run("vote1.question = question1");
    run(r#"{
    question = vote1.question
    question.rate = (question.rate * question.count + vote1.rate) / (question.count + 1)
    question.count = question.count + 1
}"#);
    assert_eq!(run("question1.rate"), 4);
    assert_eq!(run("question1.count"), 1);
    run("vote1.rate = 5");
    assert_eq!(run("question1.rate"), 4.5);
}

/// Nucleoid runs an expression statement of class
#[rustfmt::skip]
#[test]
fn runs_an_expression_statement_of_class() {
    let mut run = runner();
    run(r#"class Element:
    pass"#);
    run("alkalis = []");
    run("element1 = Element()");
    run("element1.number = 3");
    run(r#"{
    number = $Element.number
    if number == 3:
        alkalis.push($Element)
}"#);
    {
        let actual = run("alkalis.pop()");
        let expected = run("(element1)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid rejects a variable declaration without definition
#[rustfmt::skip]
#[test]
fn rejects_a_variable_declaration_without_definition() {
    let (_run, mut run_error) = crate::common::runners();
    assert_eq!(run_error("a: int"), "ReferenceError: Missing definition");
}

/// Nucleoid creates a dependency based on the length of an identifier
#[rustfmt::skip]
#[test]
fn creates_a_dependency_based_on_the_length_of_an_identifier() {
    let mut run = runner();
    run(r#"str1 = "ABC""#);
    run("i1 = str1.length + 1");
    assert_eq!(run("i1"), 4);
    run(r#"str1 = "ABCD""#);
    assert_eq!(run("i1"), 5);
    run(r#"if str1.length > 5:
    i2 = i1"#);
    run(r#"str1 = "ABCDEF""#);
    assert_eq!(run("i2"), 7);
}

/// Nucleoid adds a created class to the class list
#[rustfmt::skip]
#[test]
fn adds_a_created_class_to_the_class_list() {
    let mut run = runner();
    assert_eq!(run("Class.length"), 0);
    run(r#"class Student:
    pass"#);
    assert_eq!(run("Class.length"), 1);
    run(r#"class User:
    pass"#);
    assert_eq!(run("Class.length"), 2);
}

/// Nucleoid updates a class definition
#[rustfmt::skip]
#[test]
fn updates_a_class_definition() {
    let mut run = runner();
    run(r#"class Message:
    pass"#);
    run("$Message.read = false");
    run("message1 = Message()");
    run(r#"class Message(payload: str):
    this.payload = payload"#);
    assert_eq!(run("message1.read"), false);
    assert_eq!(run("message1.payload"), serde_json::Value::Null);
    run(r#"message2 = Message("MESSAGE")"#);
    assert_eq!(run("message2.read"), false);
    assert_eq!(run("message2.payload"), "MESSAGE");
}

/// Nucleoid supports a string in an expression
#[rustfmt::skip]
#[test]
fn supports_a_string_in_an_expression() {
    let mut run = runner();
    assert_eq!(run("'New String'"), "New String");
    assert_eq!(run(r#""New String""#), "New String");
    assert_eq!(run("`New String`"), "New String");
    run("a = 123");
    assert_eq!(run("`New ${a} String`"), "New 123 String");
}

/// Nucleoid supports logical operators
#[rustfmt::skip]
#[test]
fn supports_logical_operators() {
    let mut run = runner();
    run("condition = false");
    assert_eq!(run("condition or true"), true);
    assert_eq!(run("condition || true"), true);
    assert_eq!(run("not condition and true"), true);
    assert_eq!(run("!condition && true"), true);
}

/// Nucleoid supports standard built-in objects
#[rustfmt::skip]
#[test]
fn supports_standard_built_in_objects() {
    let mut run = runner();
    run("max = Number.MAX_INTEGER");
    assert_eq!(run("max"), 9007199254740991.0);
    run("now = Date.now()");
    assert_eq!(run("now > 0"), true);
}

/// Nucleoid supports creating standard built-in objects
#[rustfmt::skip]
#[test]
fn supports_creating_standard_built_in_objects() {
    let mut run = runner();
    run(r#"date = Date("2019-7-24")"#);
    assert_eq!(run("date.getYear()"), 119);
}

/// Nucleoid supports built-in objects
#[rustfmt::skip]
#[test]
fn supports_built_in_objects() {
    let (mut run, mut run_error) = crate::common::runners();
    run("date1 = Date()");
    run("date2 = Date(date1.getTime())");
    assert_eq!(run("date1.getTime() == date2.getTime()"), true);
    run(r#"date3 = Date.parse("04 Dec 1995 00:12:00 GMT")"#);
    assert_eq!(run("date3"), 818035920000.0);
    assert_eq!(run_error("date4 = Date.wrong()"), "TypeError: Date.wrong is not a function");
}

/// Nucleoid calls a function with no return
#[rustfmt::skip]
#[test]
fn calls_a_function_with_no_return() {
    let mut run = runner();
    run("a = 1");
    run(r#"def copy(val):
    b = val"#);
    run(r#"copy(a)

# return: null"#);
}

/// Nucleoid calls a function with a return value
#[rustfmt::skip]
#[test]
fn calls_a_function_with_a_return_value() {
    let mut run = runner();
    run("a = 1");
    run(r#"def copy(val):
    b = val
    return val"#);
    run(r#"copy(a)

# return: 1"#);
}

/// Nucleoid supports a function in an expression
#[rustfmt::skip]
#[test]
fn supports_a_function_in_an_expression() {
    let mut run = runner();
    run("list = [1, 2, 3]");
    assert_eq!(run("list.find(function(element) { return element == 3 })"), 3);
    assert_eq!(run("list.find(element => { return element == 2 })"), 2);
    assert_eq!(run("list.find(element => element == 1)"), 1);
    assert_eq!(run("list.find(element => (element == 1))"), 1);
}

/// Nucleoid supports a function with a parameter in an expression
#[rustfmt::skip]
#[test]
fn supports_a_function_with_a_parameter_in_an_expression() {
    let mut run = runner();
    run("samples = [38.2, 39.1, 38.8, 39]");
    run("ratio = 2.1");
    run("element = 38.5");
    assert_eq!(run("samples.find(function(element) { result = element * ratio; return result == 81.48 })"), 38.8);
    assert_eq!(run("samples.find(element => { result = element * ratio; return result == 81.48 })"), 38.8);
    assert_eq!(run("samples.find(element => element == 38.8)"), 38.8);
    assert_eq!(run("samples.find(element => (element == 38.8))"), 38.8);
}

/// Nucleoid creates a variable statement with JSON
#[rustfmt::skip]
#[test]
fn creates_a_variable_statement_with_json() {
    let mut run = runner();
    assert_eq!(run(r#"{
    payload = { "data": "TEST", "nested": { "data": "NESTED_TEST" } }
    [payload.data, payload.nested.data]
}"#), serde_json::json!(["TEST", "NESTED_TEST"]));
    run(r#"message = { "pid": 1200 }"#);
    assert_eq!(run("message.pid"), 1200);
    assert_eq!(run(r#"{
    scope = { "query": "test" }
    i = { "test": scope.query }
    i.test
}"#), "test");
}

/// Nucleoid returns an inline JSON object
#[rustfmt::skip]
#[test]
fn returns_an_inline_json_object() {
    let mut run = runner();
    run(r#"{
    return { "number": 123, "string": "ABC", "bool": true }
}

# return: { "number": 123, "string": "ABC", "bool": true }"#);
}

/// Nucleoid returns an inline JSON array
#[rustfmt::skip]
#[test]
fn returns_an_inline_json_array() {
    let mut run = runner();
    run(r#"{
    return [{ "number": 123, "string": "ABC", "bool": true }]
}

# return: [{ "number": 123, "string": "ABC", "bool": true }]"#);
}

/// Nucleoid returns an inline object
#[rustfmt::skip]
#[test]
fn returns_an_inline_object() {
    let mut run = runner();
    run(r#"{
    return { number: 123, string: "ABC", bool: true }
}

# return: { "number": 123, "string": "ABC", "bool": true }"#);
}

/// Nucleoid returns an inline array
#[rustfmt::skip]
#[test]
fn returns_an_inline_array() {
    let mut run = runner();
    run(r#"{
    return [{ number: 123, string: "ABC", bool: true }]
}

# return: [{ "number": 123, "string": "ABC", "bool": true }]"#);
}

/// Nucleoid supports nested functions as a parameter in an expression
#[rustfmt::skip]
#[test]
fn supports_nested_functions_as_a_parameter_in_an_expression() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"name = "AbCDE""#);
    run("pointer = 0");
    run(r#"if not /[A-Z]/.test(name.charAt(pointer)):
    throw "INVALID_FIRST_CHARACTER""#);
    assert_eq!(run_error(r#"name = "bbCDE""#), "INVALID_FIRST_CHARACTER");
    run(r#"name = "CbCDE""#);
    assert_eq!(run_error("pointer = 1"), "INVALID_FIRST_CHARACTER");
}

/// Nucleoid supports a property of chained functions in an expression
#[rustfmt::skip]
#[test]
fn supports_a_property_of_chained_functions_in_an_expression() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class User:
    pass"#);
    run(r#"class Registration:
    pass"#);
    run("user1 = User()");
    run("registration1 = Registration()");
    run("registration1.user = user1");
    run("registration2 = Registration()");
    run("registration2.user = user1");
    assert_eq!(run_error(r#"if Registration.filter(r => r.user == $User).length > 1:
    throw "USER_ALREADY_REGISTERED""#), "USER_ALREADY_REGISTERED");
}

/// Nucleoid throws an error as a string
#[rustfmt::skip]
#[test]
fn throws_an_error_as_a_string() {
    let (_run, mut run_error) = crate::common::runners();
    assert_eq!(run_error("throw 'INVALID'"), "INVALID");
    assert_eq!(run_error(r#"throw "INVALID""#), "INVALID");
}

/// Nucleoid throws an error as an integer
#[rustfmt::skip]
#[test]
fn throws_an_error_as_an_integer() {
    let (_run, mut run_error) = crate::common::runners();
    assert_eq!(run_error("throw 123"), 123);
}

/// Nucleoid throws a reference error if the thrown value is not defined
#[rustfmt::skip]
#[test]
fn throws_a_reference_error_if_the_thrown_value_is_not_defined() {
    let (_run, mut run_error) = crate::common::runners();
    assert_eq!(run_error("throw abc"), "ReferenceError: abc is not defined");
}

/// Nucleoid creates a class assignment before initialization
#[rustfmt::skip]
#[test]
fn creates_a_class_assignment_before_initialization() {
    let mut run = runner();
    run(r#"class Review:
    pass"#);
    run("$Review.rate = $Review.sum / 10");
    run("review1 = Review()");
    assert_eq!(run("review1.rate"), serde_json::Value::Null);
    run("review1.sum = 42");
    assert_eq!(run("review1.rate"), 4.2);
}

/// Nucleoid creates a class assignment after initialization
#[rustfmt::skip]
#[test]
fn creates_a_class_assignment_after_initialization() {
    let mut run = runner();
    run(r#"class Shape:
    pass"#);
    run("shape1 = Shape()");
    run("shape1.edge = 3");
    run("shape2 = Shape()");
    run("shape2.edge = 3");
    run("$Shape.angle = ($Shape.edge - 2) * 180");
    assert_eq!(run("shape1.angle"), 180);
    assert_eq!(run("shape2.angle"), 180);
    run("shape1.edge = 4");
    assert_eq!(run("shape1.angle"), 360);
    assert_eq!(run("shape2.angle"), 180);
}

/// Nucleoid updates a class assignment
#[rustfmt::skip]
#[test]
fn updates_a_class_assignment() {
    let mut run = runner();
    run(r#"class Employee:
    pass"#);
    run("employee1 = Employee()");
    run("employee1.id = 1");
    run(r#"$Employee.username = "E" + $Employee.id"#);
    assert_eq!(run("employee1.username"), "E1");
    run(r#"$Employee.username = "F" + $Employee.id"#);
    assert_eq!(run("employee1.username"), "F1");
    run("employee1.id = 2");
    assert_eq!(run("employee1.username"), "F2");
}

/// Nucleoid creates an if statement of class before initialization
#[rustfmt::skip]
#[test]
fn creates_an_if_statement_of_class_before_initialization() {
    let mut run = runner();
    run(r#"class Ticket:
    pass"#);
    run(r#"if $Ticket.date > Date("1993-1-1"):
    $Ticket.status = "EXPIRED""#);
    run("ticket1 = Ticket()");
    assert_eq!(run("ticket1.status"), serde_json::Value::Null);
    run(r#"ticket1.date = Date("1993-2-1")"#);
    assert_eq!(run("ticket1.status"), "EXPIRED");
    run("ticket2 = Ticket()");
    assert_eq!(run("ticket2.status"), serde_json::Value::Null);
}

/// Nucleoid creates an if statement of class after initialization
#[rustfmt::skip]
#[test]
fn creates_an_if_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Student:
    pass"#);
    run("student1 = Student()");
    run("student1.age = 2");
    run(r#"student1.class = "Daycare""#);
    run("student2 = Student()");
    run("student2.age = 2");
    run(r#"student2.class = "Daycare""#);
    run(r#"if $Student.age == 3:
    $Student.class = "Preschool""#);
    assert_eq!(run("student1.class"), "Daycare");
    assert_eq!(run("student2.class"), "Daycare");
    run("student1.age = 3");
    assert_eq!(run("student1.class"), "Preschool");
    assert_eq!(run("student2.class"), "Daycare");
}

/// Nucleoid updates an if block of class
#[rustfmt::skip]
#[test]
fn updates_an_if_block_of_class() {
    let mut run = runner();
    run(r#"class Inventory:
    pass"#);
    run("inventory1 = Inventory()");
    run("inventory1.quantity = 0");
    run("inventory2 = Inventory()");
    run("inventory2.quantity = 1000");
    run(r#"if $Inventory.quantity == 0:
    $Inventory.replenishment = true"#);
    assert_eq!(run("inventory1.replenishment"), true);
    assert_eq!(run("inventory2.replenishment"), serde_json::Value::Null);
    run(r#"if $Inventory.quantity == 0:
    $Inventory.replenishment = false"#);
    assert_eq!(run("inventory1.replenishment"), false);
    assert_eq!(run("inventory2.replenishment"), serde_json::Value::Null);
}

/// Nucleoid creates an else statement of class before initialization
#[rustfmt::skip]
#[test]
fn creates_an_else_statement_of_class_before_initialization() {
    let mut run = runner();
    run(r#"class Count:
    pass"#);
    run(r#"if $Count.max > 1000:
    $Count.reset = urgent
else:
    $Count.reset = regular"#);
    run(r#"urgent = "URGENT""#);
    run(r#"regular = "REGULAR""#);
    run("count1 = Count()");
    run("count1.max = 850");
    assert_eq!(run("count1.reset"), "REGULAR");
    run(r#"regular = "R""#);
    assert_eq!(run("count1.reset"), "R");
}

/// Nucleoid creates an else statement of class after initialization
#[rustfmt::skip]
#[test]
fn creates_an_else_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Concentration:
    pass"#);
    run(r#"serialDilution = "(c1V1+c2V2)/(V1+V2)""#);
    run(r#"directDilution = "c1/V1""#);
    run("concentration1 = Concentration()");
    run("concentration1.substances = 2");
    run(r#"if $Concentration.substances == 1:
    $Concentration.formula = directDilution
else:
    $Concentration.formula = serialDilution"#);
    assert_eq!(run("concentration1.formula"), "(c1V1+c2V2)/(V1+V2)");
    run(r#"serialDilution = "(c1V1+c2V2+c3V3)/(V1+V2+V3)""#);
    assert_eq!(run("concentration1.formula"), "(c1V1+c2V2+c3V3)/(V1+V2+V3)");
}

/// Nucleoid creates an else if statement of class before initialization
#[rustfmt::skip]
#[test]
fn creates_an_else_if_statement_of_class_before_initialization() {
    let mut run = runner();
    run(r#"class Storage:
    pass"#);
    run(r#"normal = "NORMAL"; low = "LOW"; empty = "EMPTY""#);
    run(r#"if $Storage.capacity > 25:
    $Storage.status = normal
else if $Storage.capacity > 0:
    $Storage.status = low
else:
    $Storage.status = empty"#);
    run("storage1 = Storage()");
    run("storage1.capacity = 23");
    assert_eq!(run("storage1.status"), "LOW");
    run(r#"low = "L""#);
    assert_eq!(run("storage1.status"), "L");
}

/// Nucleoid creates an else if statement of class after initialization
#[rustfmt::skip]
#[test]
fn creates_an_else_if_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Registration:
    pass"#);
    run(r#"yes = "YES"; pending = "PENDING"; no = "NO""#);
    run("registration1 = Registration()");
    run("registration1.available = 0");
    run(r#"if $Registration.available > 10:
    $Registration.accepted = yes
else if $Registration.available > 0:
    $Registration.accepted = pending
else:
    $Registration.accepted = no"#);
    assert_eq!(run("registration1.accepted"), "NO");
    run("yes = true; no = false");
    assert_eq!(run("registration1.accepted"), false);
}

/// Nucleoid creates multiple else if statement of class before initialization
#[rustfmt::skip]
#[test]
fn creates_multiple_else_if_statement_of_class_before_initialization() {
    let mut run = runner();
    run(r#"class Capacity:
    pass"#);
    run(r#"if $Capacity.spare / $Capacity.available > 0.5:
    $Capacity.total = $Capacity.available + $Capacity.spare
else if $Capacity.spare / $Capacity.available > 0.1:
    $Capacity.total = $Capacity.available + $Capacity.spare * 2
else:
    $Capacity.total = $Capacity.available + $Capacity.spare * 3"#);
    run("capacity1 = Capacity()");
    run("capacity1.available = 100");
    run("capacity1.spare = 5");
    assert_eq!(run("capacity1.total"), 115);
    run("capacity1.spare = 1");
    assert_eq!(run("capacity1.total"), 103);
}

/// Nucleoid creates multiple else if statement of class after initialization
#[rustfmt::skip]
#[test]
fn creates_multiple_else_if_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Shape:
    pass"#);
    run("shape1 = Shape()");
    run(r#"shape1.type = "RECTANGLE""#);
    run("shape1.x = 5");
    run("shape1.y = 6");
    run(r#"if $Shape.type == "SQUARE":
    $Shape.area = Math.pow($Shape.x, 2)
else if $Shape.type == "TRIANGLE":
    $Shape.area = $Shape.x * $Shape.y / 2
else:
    $Shape.area = $Shape.x * $Shape.y"#);
    assert_eq!(run("shape1.area"), 30);
    run("shape1.x = 7");
    assert_eq!(run("shape1.area"), 42);
}

/// Nucleoid runs a block statement of class before initialization
#[rustfmt::skip]
#[test]
fn runs_a_block_statement_of_class_before_initialization() {
    let mut run = runner();
    run(r#"class Stock:
    pass"#);
    run(r#"{
    change = $Stock.before * 4 / 100
    $Stock.after = $Stock.before + change
}"#);
    run("stock1 = Stock()");
    assert_eq!(run("stock1.after"), serde_json::Value::Null);
    run("stock1.before = 57.25");
    assert_eq!(run("stock1.after"), 59.54);
    run("stock1.before = 59.5");
    assert_eq!(run("stock1.after"), 61.88);
}

/// Nucleoid runs a block statement of class after initialization
#[rustfmt::skip]
#[test]
fn runs_a_block_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Purchase:
    pass"#);
    run("purchase1 = Purchase()");
    run("purchase1.price = 99");
    run(r#"{
    retail = $Purchase.price * 1.15
    $Purchase.retailPrice = retail
}"#);
    assert_eq!(run("purchase1.retailPrice"), 113.85);
    run("purchase1.price = 199");
    assert_eq!(run("purchase1.retailPrice"), 228.85);
}

/// Nucleoid runs a nested block statement of class before initialization
#[rustfmt::skip]
#[test]
fn runs_a_nested_block_statement_of_class_before_initialization() {
    let mut run = runner();
    run(r#"class Compound:
    pass"#);
    run(r#"{
    mol = 69.94 / $Compound.substance
    {
        $Compound.sample = Math.floor(mol * $Compound.mol)
    }
}"#);
    run("compound1 = Compound()");
    run("compound1.substance = 55.85");
    run("compound1.mol = 1000");
    assert_eq!(run("compound1.sample"), 1252);
}

/// Nucleoid runs a nested block statement of class after initialization
#[rustfmt::skip]
#[test]
fn runs_a_nested_block_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Bug:
    pass"#);
    run("bug1 = Bug()");
    run("bug1.initialScore = 1000");
    run("bug1.aging = 24");
    run(r#"{
    score = $Bug.aging * 10
    {
        $Bug.priorityScore = score + $Bug.initialScore
    }
}"#);
    assert_eq!(run("bug1.priorityScore"), 1240);
}

/// Nucleoid runs a nested if statement of class before initialization
#[rustfmt::skip]
#[test]
fn runs_a_nested_if_statement_of_class_before_initialization() {
    let mut run = runner();
    run(r#"class Mortgage:
    pass"#);
    run(r#"rate1 = "EXCEPTIONAL""#);
    run(r#"{
    interest = $Mortgage.annual / 12
    if interest < 4:
        $Mortgage.rate = rate1
}"#);
    run("mortgage1 = Mortgage()");
    run("mortgage1.annual = 46");
    assert_eq!(run("mortgage1.rate"), "EXCEPTIONAL");
    run(r#"rate1 = "E""#);
    assert_eq!(run("mortgage1.rate"), "E");
}

/// Nucleoid runs a nested if statement of class after initialization
#[rustfmt::skip]
#[test]
fn runs_a_nested_if_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Building:
    pass"#);
    run(r#"buildingType1 = "SKYSCRAPER""#);
    run("building1 = Building()");
    run("building1.floors = 20");
    run(r#"{
    height = $Building.floors * 14
    if height > 330:
        $Building.type = buildingType1
}"#);
    assert_eq!(run("building1.type"), serde_json::Value::Null);
    run("building1.floors = 25");
    assert_eq!(run("building1.type"), "SKYSCRAPER");
    run(r#"buildingType1 = "S""#);
    assert_eq!(run("building1.type"), "S");
}

/// Nucleoid creates a nested else statement of class before initialization
#[rustfmt::skip]
#[test]
fn creates_a_nested_else_statement_of_class_before_initialization() {
    let mut run = runner();
    run(r#"class Account:
    pass"#);
    run(r#"noAlert = "NO_ALERT""#);
    run(r#"lowAlert = "LOW_ALERT""#);
    run(r#"{
    balance = $Account.balance
    if balance > 1000:
        $Account.alert = noAlert
    else:
        $Account.alert = lowAlert
}"#);
    run("account1 = Account()");
    run("account1.balance = 950");
    assert_eq!(run("account1.alert"), "LOW_ALERT");
    run(r#"lowAlert = "L""#);
    assert_eq!(run("account1.alert"), "L");
}

/// Nucleoid creates a nested else statement of class after initialization
#[rustfmt::skip]
#[test]
fn creates_a_nested_else_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Question:
    pass"#);
    run(r#"high = "HIGH""#);
    run(r#"low = "LOW""#);
    run("question1 = Question()");
    run("question1.count = 1");
    run(r#"{
    score = $Question.count * 10
    if score > 100:
        $Question.type = high
    else:
        $Question.type = low
}"#);
    assert_eq!(run("question1.type"), "LOW");
    run(r#"low = "L""#);
    assert_eq!(run("question1.type"), "L");
    run("question1.count = 11");
    assert_eq!(run("question1.type"), "HIGH");
}

/// Nucleoid creates a class assignment with multiple properties before declaration
#[rustfmt::skip]
#[test]
fn creates_a_class_assignment_with_multiple_properties_before_declaration() {
    let mut run = runner();
    run(r#"class Room:
    pass"#);
    run("$Room.level = $Room.number / 10");
    run(r#"class Guest:
    pass"#);
    run("$Guest.room = Room()");
    run("guest1 = Guest()");
    run("guest1.room.number = 30");
    assert_eq!(run("guest1.room.level"), 3);
    run("guest2 = Guest()");
    assert_eq!(run("guest2.room.number"), 30);
    assert_eq!(run("guest2.room.level"), 3);
}

/// Nucleoid creates a class assignment with multiple properties after declaration
#[rustfmt::skip]
#[test]
fn creates_a_class_assignment_with_multiple_properties_after_declaration() {
    let mut run = runner();
    run(r#"class Channel:
    pass"#);
    run(r#"class Frequency:
    pass"#);
    run("channel1 = Channel()");
    run("$Channel.frequency = Frequency()");
    run("$Frequency.hertz = 1 / $Frequency.period");
    assert_eq!(run("channel1.frequency.hertz"), serde_json::Value::Null);
    run("channel1.frequency.period = 0.0025");
    assert_eq!(run("channel1.frequency.hertz"), 400);
    run("channel2 = Channel()");
    assert_eq!(run("channel2.frequency.period"), 0.0025);
    assert_eq!(run("channel2.frequency.hertz"), 400);
}

/// Nucleoid creates a class assignment as multiple properties as part of a declaration before initialization
#[rustfmt::skip]
#[test]
fn creates_a_class_assignment_as_multiple_properties_as_part_of_a_declaration_before_initialization() {
    let mut run = runner();
    run(r#"class Hospital:
    pass"#);
    run(r#"class Clinic:
    pass"#);
    run("$Hospital.clinic = Clinic()");
    run("$Hospital.patients = $Hospital.clinic.beds * 746");
    run("hospital1 = Hospital()");
    assert_eq!(run("hospital1.patients"), serde_json::Value::Null);
    run("hospital1.clinic.beds = 2678");
    assert_eq!(run("hospital1.patients"), 1997788);
    run("hospital1.clinic.beds = 3000");
    assert_eq!(run("hospital1.patients"), 2238000);
}

/// Nucleoid creates a class assignment as multiple properties as part of a declaration after initialization
#[rustfmt::skip]
#[test]
fn creates_a_class_assignment_as_multiple_properties_as_part_of_a_declaration_after_initialization() {
    let mut run = runner();
    run(r#"class Server:
    pass"#);
    run(r#"class OS:
    pass"#);
    run("$Server.os = OS()");
    run("server1 = Server()");
    run("server1.os.version = 14");
    run(r#"$Server.build = $Server.os.version + ".526291""#);
    assert_eq!(run("server1.build"), "14.526291");
    run("server1.os.version = 15");
    assert_eq!(run("server1.build"), "15.526291");
}

/// Nucleoid creates a class assignment only if the instance is defined
#[rustfmt::skip]
#[test]
fn creates_a_class_assignment_only_if_the_instance_is_defined() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Phone:
    pass"#);
    assert_eq!(run_error("$Phone.line.wired = true"), "ReferenceError: Phone.line is not defined");
}

/// Nucleoid creates a for of statement
#[rustfmt::skip]
#[test]
fn creates_a_for_of_statement() {
    let mut run = runner();
    run(r#"class Question(rate: int):
    this.rate = rate"#);
    run("question1 = Question(4)");
    run("question2 = Question(5)");
    run(r#"class Summary(question):
    this.question = question"#);
    run("$Summary.rate = $Summary.question.rate.value");
    run(r#"for question of Question:
    Summary(question)"#);
    assert_eq!(run("Summary[0].rate"), 4);
    assert_eq!(run("Summary[1].rate"), 5);
}

/// Nucleoid creates a block of for statement without dependencies
#[rustfmt::skip]
#[test]
fn creates_a_block_of_for_statement_without_dependencies() {
    let mut run = runner();
    run(r#"class Item:
    pass"#);
    run("item1 = Item()");
    run("item2 = Item()");
    run("VALUE = 10");
    run(r#"for item of Item:
    i = 10 * VALUE
    item.score = i"#);
    run("VALUE = 20");
    assert_eq!(run("item1.score"), 100);
    assert_eq!(run("item2.score"), 100);
    run(r#"for item of Item:
    i = 10 * VALUE
    item.score = i"#);
    assert_eq!(run("item1.score"), 200);
    assert_eq!(run("item2.score"), 200);
    run("item3 = Item()");
    assert_eq!(run("item3.score"), serde_json::Value::Null);
}

/// Nucleoid loops through only defined objects in a for of statement
#[rustfmt::skip]
#[test]
fn loops_through_only_defined_objects_in_a_for_of_statement() {
    let mut run = runner();
    run("array = []");
    run(r#"class Item:
    pass"#);
    run("item1 = Object()");
    run("array.push(item1)");
    run(r#"item2 = { "id": "item3" }"#);
    run("array.push(item2)");
    run("item4 = Item()");
    run("array.push(item4)");
    run(r#"item5 = { "id": "item4" }"#);
    run("array.push(item5)");
    run("count = 0");
    run("items = []");
    run(r#"for item of array:
    count = count + 1
    items.push(item)"#);
    assert_eq!(run("count"), 1);
    assert_eq!(run("items.length"), 1);
    {
        let actual = run("items[0]");
        let expected = run("(item4)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid supports an if statement in a for of statement
#[rustfmt::skip]
#[test]
fn supports_an_if_statement_in_a_for_of_statement() {
    let mut run = runner();
    run(r#"class Question:
    pass"#);
    run("question1 = Question()");
    run("question2 = Question()");
    run("question2.archived = true");
    run("question3 = Question()");
    run(r#"class Summary(question):
    this.question = question"#);
    run(r#"$Summary.type = "DAILY""#);
    run(r#"for question of Question:
    if not question.archived:
        Summary(question)"#);
    assert_eq!(run("Summary.length"), 2);
    assert_eq!(run("Summary[0].question.id"), "question1");
    assert_eq!(run("Summary[1].question.id"), "question3");
    assert_eq!(run("Summary[0].type"), "DAILY");
    assert_eq!(run("Summary[1].type"), "DAILY");
    run(r#"$Summary.type = "WEEKLY""#);
    assert_eq!(run("Summary[0].type"), "WEEKLY");
    assert_eq!(run("Summary[1].type"), "WEEKLY");
}

/// Nucleoid returns an integer in variable assignment
#[rustfmt::skip]
#[test]
fn returns_an_integer_in_variable_assignment() {
    let mut run = runner();
    run(r#"def test(a):
    return a = 2"#);
    run("b = 1");
    run(r#"test(b)

# return: 2"#);
}

/// Nucleoid returns the reference of a function call
#[rustfmt::skip]
#[test]
fn returns_the_reference_of_a_function_call() {
    let mut run = runner();
    run("a = Object()");
    run("c = 1");
    run(r#"def test(b):
    return b = a"#);
    assert_eq!(run("test(c)"), serde_json::json!({}));
    assert_eq!(run("c"), 1);
}

/// Nucleoid returns a string value of a function call
#[rustfmt::skip]
#[test]
fn returns_a_string_value_of_a_function_call() {
    let mut run = runner();
    run(r#"def test(a):
    return a = "abc""#);
    run("b = 1");
    run(r#"test(b)

# return: "abc""#);
}

/// Nucleoid returns an object value of a function call
#[rustfmt::skip]
#[test]
fn returns_an_object_value_of_a_function_call() {
    let mut run = runner();
    run(r#"def test(a):
    return a = Object()"#);
    run("b = 1");
    run(r#"test(b)

# return: {}"#);
}

/// Nucleoid runs a function with a variable
#[rustfmt::skip]
#[test]
fn runs_a_function_with_a_variable() {
    let mut run = runner();
    run(r#"def test(a):
    return a + 23"#);
    run(r#"data = "UUID-1""#);
    run(r#"test(data)

# return: "UUID-123""#);
}

/// Nucleoid returns the first return statement in a block
#[rustfmt::skip]
#[test]
fn returns_the_first_return_statement_in_a_block() {
    let mut run = runner();
    run(r#"{
    return 123
    return "abc"
}

# return: 123"#);
}

/// Nucleoid returns the instance itself in instance creation
#[rustfmt::skip]
#[test]
fn returns_the_instance_itself_in_instance_creation() {
    let mut run = runner();
    run(r#"class Test(prop: int):
    this.prop = prop"#);
    run(r#"Test(123)

# return: { "id": "[UUID]", "prop": 123 }"#);
    assert_eq!(run("Test[0].prop"), 123);
    assert_eq!(run("Test[0].id != null"), true);
}

/// Nucleoid explains how a value was derived
#[rustfmt::skip]
#[test]
fn explains_how_a_value_was_derived() {
    let mut run = runner();
    run("a = 1");
    run("b = a + 2");
    run("c = b * 2");
    assert_eq!(run("(why c).length"), 3);
    assert_eq!(run("(why c)[0].node"), "c");
    assert_eq!(run("(why c)[0].holds"), 6);
    assert_eq!(run("(why c)[0].rule"), "c = b*2");
    assert_eq!(run("(why c)[0].state"), "derived");
    assert_eq!(run("(why c)[0].from"), serde_json::json!(["b"]));
}

/// Nucleoid states a fact that follows from nothing else
#[rustfmt::skip]
#[test]
fn states_a_fact_that_follows_from_nothing_else() {
    let mut run = runner();
    run("a = 1");
    run("b = a + 2");
    assert_eq!(run("(why b)[1].node"), "a");
    assert_eq!(run("(why b)[1].state"), "stated");
    assert_eq!(run("(why b)[1].from"), serde_json::json!([]));
}

/// Nucleoid names the class-level rule a property was derived from
#[rustfmt::skip]
#[test]
fn names_the_class_level_rule_a_property_was_derived_from() {
    let mut run = runner();
    run(r#"class Human(name: str):
    this.name = name"#);
    run("$Human.mortal = true");
    run(r#"socrates = Human("Socrates")"#);
    assert_eq!(run("(why socrates.mortal).length"), 1);
    assert_eq!(run("(why socrates.mortal)[0].rule"), "$Human.mortal = true");
    assert_eq!(run("(why socrates.mortal)[0].state"), "derived");
}

/// Nucleoid reports what a value affects
#[rustfmt::skip]
#[test]
fn reports_what_a_value_affects() {
    let mut run = runner();
    run("a = 1");
    run("b = a + 2");
    run("c = b * 2");
    assert_eq!(run("(affects a).length"), 2);
    assert_eq!(run("(affects a)[0].node"), "b");
    assert_eq!(run("(affects a)[1].node"), "c");
}

/// Nucleoid chains reasoning operations
#[rustfmt::skip]
#[test]
fn chains_reasoning_operations() {
    let mut run = runner();
    run("a = 1");
    run("b = a + 2");
    run("c = b * 2");
    assert_eq!(run("(c |> why).length"), 3);
    assert_eq!(run("(a |> affects |> why).length"), 3);
    {
        let actual = run("(why c).length");
        let expected = run("((c |> why).length)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid keeps an explanation up to date
#[rustfmt::skip]
#[test]
fn keeps_an_explanation_up_to_date() {
    let mut run = runner();
    run("a = 1");
    run("b = a + 2");
    run("c = b * 2");
    run("trace = why c");
    assert_eq!(run("trace[0].holds"), 6);
    run("a = 5");
    assert_eq!(run("trace[0].holds"), 14);
}

/// Nucleoid does not select a reasoning statement
#[rustfmt::skip]
#[test]
fn does_not_select_a_reasoning_statement() {
    let mut run = runner();
    run("a = 1");
    run("b = a + 2");
    run("trace = why b");
    assert_eq!(run("(affects a).length"), 1);
    assert_eq!(run("(affects a)[0].node"), "b");
}

/// Nucleoid selects the whole model
#[rustfmt::skip]
#[test]
fn selects_the_whole_model() {
    let mut run = runner();
    run("a = 1");
    run("b = a + 2");
    assert_eq!(run("(model |> why).length"), 2);
}

/// Nucleoid throws an error when explaining something that is not defined
#[rustfmt::skip]
#[test]
fn throws_an_error_when_explaining_something_that_is_not_defined() {
    let (_run, mut run_error) = crate::common::runners();
    assert_eq!(run_error("why nothing"), "ReferenceError: nothing is not defined");
}

/// Nucleoid treats a reasoning name as a variable when one is defined
#[rustfmt::skip]
#[test]
fn treats_a_reasoning_name_as_a_variable_when_one_is_defined() {
    let mut run = runner();
    run("why = 1");
    assert_eq!(run("why"), 1);
}

/// Nucleoid freezes value reads inside compound expressions
#[rustfmt::skip]
#[test]
fn freezes_value_reads_inside_compound_expressions() {
    let mut run = runner();
    run("seed = 1");
    run("live = 10");
    run("values = [seed.value, live]");
    run(r#"record = { "frozen": seed.value, "live": live }"#);
    run("label = `${seed.value}-${live}`");
    run(r#"text = "abcd""#);
    run("start = 1");
    run("end = 3");
    run("part = text[start.value:end]");
    run("tail = text[start.value:]");
    run("head = text[:end.value]");
    run("seed = 2");
    run("live = 20");
    run("start = 2");
    run("end = 4");
    run(r#"text = "wxyz""#);
    assert_eq!(run("values"), serde_json::json!([1, 20]));
    assert_eq!(run("record"), serde_json::json!({ "frozen": 1, "live": 20 }));
    assert_eq!(run("label"), "1-20");
    assert_eq!(run("part"), "xyz");
    assert_eq!(run("tail"), "xyz");
    assert_eq!(run("head"), "wxy");
}

/// Nucleoid updates inherited rules on existing subtype instances
#[rustfmt::skip]
#[test]
fn updates_inherited_rules_on_existing_subtype_instances() {
    let mut run = runner();
    run(r#"class Parent:
    pass"#);
    run("$Parent.flag = 1");
    run(r#"class Child: Parent
    pass"#);
    run(r#"class Grandchild: Child
    pass"#);
    run("child = Child()");
    run("grandchild = Grandchild()");
    run("$Parent.flag = 2");
    run(r#"$Parent.note = "NEW""#);
    assert_eq!(run("child.flag"), 2);
    assert_eq!(run("grandchild.flag"), 2);
    assert_eq!(run("child.note"), "NEW");
    assert_eq!(run("grandchild.note"), "NEW");
    assert_eq!(run("Parent.length"), 0);
    assert_eq!(run("Child.length"), 1);
    run(r#"if $Parent.flag == 2:
    $Parent.active = true"#);
    assert_eq!(run("child.active"), true);
    assert_eq!(run("grandchild.active"), true);
    run("delete $Parent.note");
    assert_eq!(run("child.note"), serde_json::Value::Null);
    assert_eq!(run("grandchild.note"), serde_json::Value::Null);
}

/// Nucleoid preserves subtype property rules when a parent rule changes
#[rustfmt::skip]
#[test]
fn preserves_subtype_property_rules_when_a_parent_rule_changes() {
    let mut run = runner();
    run(r#"class Parent:
    pass"#);
    run(r#"class Child: Parent
    pass"#);
    run("$Child.flag = 7");
    run("child = Child()");
    run("$Parent.flag = 2");
    assert_eq!(run("child.flag"), 7);
    run(r#"if $Parent.id != "":
    $Parent.flag = 3"#);
    run(r#"{
    $Parent.flag = 4
}"#);
    assert_eq!(run("child.flag"), 7);
    run("another = Child()");
    assert_eq!(run("another.flag"), 7);
    run("delete $Parent.flag");
    assert_eq!(run("child.flag"), 7);
    assert_eq!(run("another.flag"), 7);
}

/// Nucleoid rolls back a parent rule that fails for a subtype instance
#[rustfmt::skip]
#[test]
fn rolls_back_a_parent_rule_that_fails_for_a_subtype_instance() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Parent:
    pass"#);
    run(r#"class Child: Parent
    pass"#);
    run("parent = Parent()");
    run("parent.score = 0");
    run("child = Child()");
    run("child.score = 2");
    assert_eq!(run_error(r#"if $Parent.score > 1:
    throw "LIMIT""#), "LIMIT");
    run("child.score = 3");
    run("parent.score = 3");
    run("another = Child()");
    run("another.score = 4");
    assert_eq!(run("child.score"), 3);
    assert_eq!(run("parent.score"), 3);
    assert_eq!(run("another.score"), 4);
}

/// Nucleoid resolves super from the constructor currently executing
#[rustfmt::skip]
#[test]
fn resolves_super_from_the_constructor_currently_executing() {
    let mut run = runner();
    run(r#"class Base(amount):
    this.amount = amount"#);
    run(r#"class Middle: Base
    def init(amount):
        super(amount + 1)
        this.middle = true"#);
    run(r#"class Leaf: Middle
    def init(amount):
        super(amount * 2)
        this.leaf = true"#);
    run("leaf = Leaf(3)");
    assert_eq!(run("leaf.amount"), 7);
    assert_eq!(run("leaf.middle"), true);
    assert_eq!(run("leaf.leaf"), true);
    run(r#"class Passive: Middle
    pass"#);
    run("passive = Passive(4)");
    assert_eq!(run("passive.amount"), 5);
    run(r#"class Final: Passive
    def init(amount):
        super(amount + 2)"#);
    run("final = Final(4)");
    assert_eq!(run("final.amount"), 7);
}

/// Nucleoid rejects a class inheriting from itself
#[rustfmt::skip]
#[test]
fn rejects_a_class_inheriting_from_itself() {
    let (mut run, mut run_error) = crate::common::runners();
    assert_eq!(run_error(r#"class Loop: Loop
    pass"#), "TypeError: Circular Inheritance");
    assert_eq!(run("Class.length"), 0);
    run(r#"class Loop:
    pass"#);
    run("loop = Loop()");
    assert_eq!(run("loop.id"), "loop");
}

/// Nucleoid rejects an inheritance cycle introduced by redeclaration
#[rustfmt::skip]
#[test]
fn rejects_an_inheritance_cycle_introduced_by_redeclaration() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Root:
    pass"#);
    run("$Root.flag = true");
    run(r#"class Child: Root
    pass"#);
    assert_eq!(run_error(r#"class Root: Child
    pass"#), "TypeError: Circular Inheritance");
    run("child = Child()");
    assert_eq!(run("child.flag"), true);
    assert_eq!(run("Class.length"), 2);
}

/// Nucleoid rejects class dependency cycles inside compound expressions
#[rustfmt::skip]
#[test]
fn rejects_class_dependency_cycles_inside_compound_expressions() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Item:
    pass"#);
    run("$Item.list = [$Item.source]");
    assert_eq!(run_error("$Item.source = $Item.list"), "TypeError: Circular Dependency");
    run(r#"$Item.record = { "source": $Item.source }"#);
    assert_eq!(run_error("$Item.source = $Item.record"), "TypeError: Circular Dependency");
    run("$Item.label = `${$Item.source}`");
    assert_eq!(run_error("$Item.source = $Item.label"), "TypeError: Circular Dependency");
    run(r#"$Item.part = "abcd"[$Item.source:]"#);
    assert_eq!(run_error("$Item.source = $Item.part.length"), "TypeError: Circular Dependency");
    run(r#"$Item.head = "abcd"[:$Item.source]"#);
    assert_eq!(run_error("$Item.source = $Item.head.length"), "TypeError: Circular Dependency");
    run("$Item.copy = $Item.list[:]");
    assert_eq!(run_error("$Item.source = $Item.copy[0]"), "TypeError: Circular Dependency");
    run(r#"$Item.indexed = [$Item["source"]]"#);
    assert_eq!(run_error("$Item.source = $Item.indexed"), "TypeError: Circular Dependency");
    run("$Item.source = 1");
    run("item = Item()");
    assert_eq!(run("item.list"), serde_json::json!([1]));
    assert_eq!(run("item.label"), "1");
    assert_eq!(run("item.part"), "bcd");
    run("$Item.snapshot = [$Item.source.value]");
    run("$Item.source = $Item.snapshot[0] + 1");
    assert_eq!(run("item.source"), 2);
    assert_eq!(run("item.snapshot"), serde_json::json!([1]));
    assert_eq!(run("item.list"), serde_json::json!([2]));
}

/// Nucleoid checks both slice bounds when declaring a class rule
#[rustfmt::skip]
#[test]
fn checks_both_slice_bounds_when_declaring_a_class_rule() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Item:
    pass"#);
    assert_eq!(run_error(r#"$Item.part = "abcd"[missing:]"#), "ReferenceError: missing is not defined");
    assert_eq!(run_error(r#"$Item.part = "abcd"[:missing]"#), "ReferenceError: missing is not defined");
    run("start = 1");
    run("end = 3");
    run(r#"$Item.part = "abcd"[start:end]"#);
    run("item = Item()");
    assert_eq!(run("item.part"), "bc");
    run("end = 4");
    assert_eq!(run("item.part"), "bcd");
}

/// Nucleoid propagates null through indexed object properties
#[rustfmt::skip]
#[test]
fn propagates_null_through_indexed_object_properties() {
    let mut run = runner();
    run(r#"class Item:
    pass"#);
    run("item = Item()");
    run("item.amount = null");
    run(r#"field = "amount""#);
    run("direct = item.amount + 1");
    run("indexed = item[field] + 1");
    assert_eq!(run("direct"), serde_json::Value::Null);
    assert_eq!(run("indexed"), serde_json::Value::Null);
    run("item.amount = 4");
    assert_eq!(run("direct"), 5);
    assert_eq!(run("indexed"), 5);
    run("delete item.amount");
    assert_eq!(run("direct"), serde_json::Value::Null);
    assert_eq!(run("indexed"), serde_json::Value::Null);
}

/// Nucleoid defers a class conditional reading an undefined indexed property
#[rustfmt::skip]
#[test]
fn defers_a_class_conditional_reading_an_undefined_indexed_property() {
    let mut run = runner();
    run(r#"class Item:
    pass"#);
    run(r#"if $Item["score"] == null:
    $Item.active = true"#);
    run("item = Item()");
    assert_eq!(run("item.active"), serde_json::Value::Null);
    run("item.score = null");
    assert_eq!(run("item.active"), true);
}

/// Nucleoid tracks the number of declared classes as a dependency
#[rustfmt::skip]
#[test]
fn tracks_the_number_of_declared_classes_as_a_dependency() {
    let (mut run, mut run_error) = crate::common::runners();
    run("count = Class.length");
    run("doubled = count * 2");
    run(r#"class First:
    pass"#);
    assert_eq!(run("count"), 1);
    assert_eq!(run("doubled"), 2);
    run(r#"class Second:
    pass"#);
    assert_eq!(run("count"), 2);
    assert_eq!(run("doubled"), 4);
    run(r#"class First:
    pass"#);
    assert_eq!(run("count"), 2);
    assert_eq!(run_error(r#"class Temporary:
    pass
throw "ABORT""#), "ABORT");
    assert_eq!(run("Class.length"), 2);
    assert_eq!(run("count"), 2);
    assert_eq!(run("doubled"), 4);
    run(r#"class Third:
    pass"#);
    assert_eq!(run("count"), 3);
    assert_eq!(run("doubled"), 6);
    run(r#"if count > 3:
    throw "TOO_MANY_TYPES""#);
    assert_eq!(run_error(r#"class Fourth:
    pass"#), "TOO_MANY_TYPES");
    assert_eq!(run("Class.length"), 3);
    assert_eq!(run("count"), 3);
    assert_eq!(run("doubled"), 6);
}

/// Nucleoid rounds numbers without losing precision at integer and half boundaries
#[rustfmt::skip]
#[test]
fn rounds_numbers_without_losing_precision_at_integer_and_half_boundaries() {
    let mut run = runner();
    assert_eq!(run("Math.round(0.49999999999999994)"), 0);
    assert_eq!(run("Math.round(0.5)"), 1);
    assert_eq!(run("Math.round(0.5000000000000001)"), 1);
    assert_eq!(run("Math.round(-0.49999999999999994)"), 0);
    assert_eq!(run("Math.round(-0.5)"), 0);
    assert_eq!(run("Math.round(-0.5000000000000001)"), -1);
    assert_eq!(run("Math.round(1.5)"), 2);
    assert_eq!(run("Math.round(-1.5)"), -1);
    assert_eq!(run("Math.round(4503599627370497)"), 4503599627370497.0);
    assert_eq!(run("Math.round(-4503599627370497)"), -4503599627370497.0);
    {
        let actual = run("Math.round(Number.MAX_INTEGER)");
        let expected = run("(Number.MAX_INTEGER)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("Math.round(Number.MIN_INTEGER)");
        let expected = run("(Number.MIN_INTEGER)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("1 / Math.round(-0.5)");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("1 / Math.round(-0.1)");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("1 / Math.round(-0.0)");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("1 / Math.round(0.0)");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("Math.round(Number.POSITIVE_INFINITY)");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("Math.round(Number.NEGATIVE_INFINITY)");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("String(Math.round(Number.NaN))"), "NaN");
    run("input = 4503599627370497");
    run("rounded = Math.round(input)");
    assert_eq!(run("rounded"), 4503599627370497.0);
    run("input = 0.49999999999999994");
    assert_eq!(run("rounded"), 0);
}

/// The committed tests must not drift from their generated JSONL export.
#[rustfmt::skip]
#[test]
fn committed_tests_match_the_dataset() {
    assert_eq!(
        include_str!("nucleoid.spec.rs").replace("\r\n", "\n"),
        include_str!(concat!(env!("OUT_DIR"), "/nucleoid.spec.rs")),
        "tests/nucleoid.spec.rs is stale; regenerate with \
         UPDATE_SPEC_TESTS=1 cargo build"
    );
}
