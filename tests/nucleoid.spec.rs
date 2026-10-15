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
    run("class Student");
    run("user0 = Student()");
    assert_eq!(run(r#"Student.find(student => student.id == "user0")"#), serde_json::json!({ "id": "user0" }));
    assert_eq!(run(r#"Student["user0"]"#), serde_json::json!({ "id": "user0" }));
}

/// Nucleoid preserves class and object lists when a class is updated
#[rustfmt::skip]
#[test]
fn preserves_class_and_object_lists_when_a_class_is_updated() {
    let mut run = runner();
    run("class User");
    run("User()");
    assert_eq!(run("Class.length"), 1);
    assert_eq!(run("User.length"), 1);
    run("class User");
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
    run("class Student");
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
    def init name, school:
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
    run("class Room");
    run("class Meeting");
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
    run("class Timesheet");
    run("class Task");
    run("class Project");
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
    run("class Device");
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
    run("class Member");
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
    run("class Distance");
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
    run(r#"def multiply first_factor, second_factor:
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
    run("class Student");
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
    run("class Ticket");
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
    run("class Shipment");
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
    run("class User");
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
else
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
    run("class Vehicle");
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
    run("class Person");
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
    run("class Schedule");
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
    run("class Product");
    run("product1 = Product()");
    run("score = product1.quality.score");
    assert_eq!(run("score"), serde_json::Value::Null);
}

/// Nucleoid assigns a unique variable for an instance without a variable name
#[rustfmt::skip]
#[test]
fn assigns_a_unique_variable_for_an_instance_without_a_variable_name() {
    let mut run = runner();
    run("class Vehicle");
    run("Vehicle()");
    assert_eq!(run("Vehicle.length"), 1);
    assert_eq!(run("Vehicle[0].id != null"), true);
}

/// Nucleoid creates a function in state
#[rustfmt::skip]
#[test]
fn creates_a_function_in_state() {
    let mut run = runner();
    run("def generate number => number * 10");
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
    run("class User");
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
    run("class Person");
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
    run("class Item");
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
    run("radius += 10");
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
    run("class Account");
    run("class Currency");
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
    run("class Warehouse");
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
    run("class Plane");
    run("class Trip");
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
    run("class Seller");
    run("class Commission");
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
    run("class Stock");
    run("class Trade");
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
    run("class Bug");
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
    else
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
    run("class Model");
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
    run("class Entity");
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
    run("class Chart");
    run("chart1 = Chart()");
    assert_eq!(run_error("chart1.plot = Plot()"), "ReferenceError: Plot is not defined");
    assert_eq!(run_error("$Chart.plot = Plot()"), "ReferenceError: Plot is not defined");
}

/// Nucleoid creates a property assignment before declaration
#[rustfmt::skip]
#[test]
fn creates_a_property_assignment_before_declaration() {
    let mut run = runner();
    run("class Order");
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
    run("class User");
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
    run("class Channel");
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
    run("class Worker");
    run("class Schedule");
    run("worker1 = Worker()");
    assert_eq!(run_error("worker1.duty.schedule = Schedule()"), "ReferenceError: worker1.duty is not defined");
}

/// Nucleoid uses only the value when a property references itself
#[rustfmt::skip]
#[test]
fn uses_only_the_value_when_a_property_references_itself() {
    let mut run = runner();
    run("class Construction");
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
    run("class Agent");
    run("class Distance");
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
    run("class Product");
    run("product1 = Product()");
    run("class Quality");
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
    run("class Schedule");
    run("class Place");
    run("value = Schedule()");
    assert_eq!(run("value"), serde_json::json!({ "id": "value" }));
    assert_eq!(run_error("value.value = Place()"), "TypeError: Cannot use 'value' as a property");
}

/// Nucleoid rejects value as a property name in a value assignment
#[rustfmt::skip]
#[test]
fn rejects_value_as_a_property_name_in_a_value_assignment() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Value");
    run("value = Value()");
    assert_eq!(run("value"), serde_json::json!({ "id": "value" }));
    assert_eq!(run_error("value.value = 2147483647"), "TypeError: Cannot use 'value' as a property");
}

/// Nucleoid uses value property to indicate using only value of property
#[rustfmt::skip]
#[test]
fn uses_value_property_to_indicate_using_only_value_of_property() {
    let mut run = runner();
    run("class Weight");
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
    run("class Question");
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
    run("class Travel");
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
    run("class Interest");
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
    run("class Alarm");
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
    run("class Device");
    run("device1 = Device()");
    run(r#"{
    $Device.code = "A" + count.value
    count += 1
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
    run("class Question");
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
    run("class Account");
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
else
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
    run("class Toy");
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
    run("class Engine");
    run("engine1 = Engine()");
    run(r#"engine1.type = "V8""#);
    run(r#"mpl = "MPL""#);
    run(r#"bsd = "BSD""#);
    run(r#"if engine1.type == "Gecko":
    engine1.license = mpl
else
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
    run("class Contact");
    run("contact1 = Contact()");
    run(r#"contact1.type = "PERSON""#);
    run(r#"contact1.first = "First""#);
    run(r#"contact1.last = "Last""#);
    run(r#"if contact1.type == "BUSINESS":
    contact1.full = "B" + contact1.first
else
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
    run("class Taxpayer");
    run("taxpayer1 = Taxpayer()");
    run("taxpayer1.income = 60000");
    run("taxpayer1.member = 1");
    run("rate = 22");
    run(r#"if taxpayer1.member > 4:
    taxpayer1.tax = taxpayer1.income * rate / 100 - 2000
else if taxpayer1.member > 2:
    taxpayer1.tax = taxpayer1.income * rate / 100 - 1000
else
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
    run("class Matter");
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
    run("class Circle");
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
    run("class Item");
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
    run("class Location");
    run("location1 = Location()");
    assert_eq!(run("delete location1"), true);
    assert_eq!(run("delete location2"), false);
}

/// Nucleoid rejects deleting an instance if it has any properties
#[rustfmt::skip]
#[test]
fn rejects_deleting_an_instance_if_it_has_any_properties() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Channel");
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
    run("class Shape");
    run("class Type");
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
    run("class Agent");
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
    run("class Item");
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
    run("class Figure");
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
    run("class Sale");
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
    run("class Chart");
    run("chart1 = Chart()");
    run("chart1.percentage = 1");
    run(r#"invalid = "INVALID""#);
    run(r#"valid = "VALID""#);
    run(r#"{
    ratio = chart1.percentage / 100
    if ratio > 1:
        chart1.status = invalid
    else
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
    run("class Person");
    run("person1 = Person()");
    run("class Address");
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
    run("class Server");
    run("server1 = Server()");
    run(r#"server1.name = "HOST1""#);
    run("class IP");
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
    run("class Schedule");
    run("schedule1 = Schedule()");
    run("class Template");
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
    run("class Vote");
    run("vote1 = Vote()");
    run("vote1.rate = 4");
    run("class Question");
    run("$Question.rate = 0");
    run("$Question.count = 0");
    run("question1 = Question()");
    run("vote1.question = question1");
    run(r#"{
    question = vote1.question
    question.rate = (question.rate * question.count + vote1.rate) / (question.count + 1)
    question.count += 1
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
    run("class Element");
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
    run("class Student");
    assert_eq!(run("Class.length"), 1);
    run("class User");
    assert_eq!(run("Class.length"), 2);
}

/// Nucleoid updates a class definition
#[rustfmt::skip]
#[test]
fn updates_a_class_definition() {
    let mut run = runner();
    run("class Message");
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
    assert_eq!(run("String(Math.max(Number.NaN))"), "NaN");
    assert_eq!(run("String(Math.min(Number.NaN))"), "NaN");
    assert_eq!(run("String(Math.max(Number.NaN, 1))"), "NaN");
    assert_eq!(run("String(Math.min(Number.NaN, 1))"), "NaN");
    assert_eq!(run("String(Math.max(1, Number.NaN))"), "NaN");
    assert_eq!(run("String(Math.min(1, Number.NaN))"), "NaN");
    assert_eq!(run("String(Math.max(1, Number.NaN, 2))"), "NaN");
    assert_eq!(run("String(Math.min(1, Number.NaN, 2))"), "NaN");
    assert_eq!(run("String(Math.max(Number.POSITIVE_INFINITY, Number.NaN))"), "NaN");
    assert_eq!(run("String(Math.min(Number.NEGATIVE_INFINITY, Number.NaN))"), "NaN");
    assert_eq!(run(r#"String(Math.max(1, "invalid"))"#), "NaN");
    assert_eq!(run(r#"String(Math.min(1, "invalid"))"#), "NaN");
    assert_eq!(run("String(Math.max(1, Math.sqrt(-1)))"), "NaN");
    assert_eq!(run("String(Math.min(1, Math.sqrt(-1)))"), "NaN");
    {
        let actual = run("Math.max()");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("Math.min()");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run(r#"Math.max("7", 5)"#), 7);
    assert_eq!(run(r#"Math.min("7", 5)"#), 5);
    assert_eq!(run("Math.max(Number.NEGATIVE_INFINITY, 5)"), 5);
    assert_eq!(run("Math.min(Number.POSITIVE_INFINITY, 5)"), 5);
    run("a = 5");
    run("b = 9");
    run("peak = Math.max(a, b)");
    run("floor = Math.min(a, b)");
    assert_eq!(run("peak"), 9);
    assert_eq!(run("floor"), 5);
    run("b = Number.NaN");
    assert_eq!(run("String(peak)"), "NaN");
    assert_eq!(run("String(floor)"), "NaN");
    run("b = 3");
    assert_eq!(run("peak"), 5);
    assert_eq!(run("floor"), 3);
    run("b = null");
    assert_eq!(run("peak"), serde_json::Value::Null);
    assert_eq!(run("floor"), serde_json::Value::Null);
    run("b = 2");
    assert_eq!(run("peak"), 5);
    assert_eq!(run("floor"), 2);
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
    run(r#"def copy val:
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
    run(r#"def copy val:
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
    run("class User");
    run("class Registration");
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
    run("class Review");
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
    run("class Shape");
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
    run("class Employee");
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
    run("class Ticket");
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
    run("class Student");
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
    run("class Inventory");
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
    run("class Count");
    run(r#"if $Count.max > 1000:
    $Count.reset = urgent
else
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
    run("class Concentration");
    run(r#"serialDilution = "(c1V1+c2V2)/(V1+V2)""#);
    run(r#"directDilution = "c1/V1""#);
    run("concentration1 = Concentration()");
    run("concentration1.substances = 2");
    run(r#"if $Concentration.substances == 1:
    $Concentration.formula = directDilution
else
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
    run("class Storage");
    run(r#"normal = "NORMAL"; low = "LOW"; empty = "EMPTY""#);
    run(r#"if $Storage.capacity > 25:
    $Storage.status = normal
else if $Storage.capacity > 0:
    $Storage.status = low
else
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
    run("class Registration");
    run(r#"yes = "YES"; pending = "PENDING"; no = "NO""#);
    run("registration1 = Registration()");
    run("registration1.available = 0");
    run(r#"if $Registration.available > 10:
    $Registration.accepted = yes
else if $Registration.available > 0:
    $Registration.accepted = pending
else
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
    run("class Capacity");
    run(r#"if $Capacity.spare / $Capacity.available > 0.5:
    $Capacity.total = $Capacity.available + $Capacity.spare
else if $Capacity.spare / $Capacity.available > 0.1:
    $Capacity.total = $Capacity.available + $Capacity.spare * 2
else
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
    run("class Shape");
    run("shape1 = Shape()");
    run(r#"shape1.type = "RECTANGLE""#);
    run("shape1.x = 5");
    run("shape1.y = 6");
    run(r#"if $Shape.type == "SQUARE":
    $Shape.area = Math.pow($Shape.x, 2)
else if $Shape.type == "TRIANGLE":
    $Shape.area = $Shape.x * $Shape.y / 2
else
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
    run("class Stock");
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
    run("class Purchase");
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
    run("class Compound");
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
    run("class Bug");
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
    run("class Mortgage");
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
    run("class Building");
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
    run("class Account");
    run(r#"noAlert = "NO_ALERT""#);
    run(r#"lowAlert = "LOW_ALERT""#);
    run(r#"{
    balance = $Account.balance
    if balance > 1000:
        $Account.alert = noAlert
    else
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
    run("class Question");
    run(r#"high = "HIGH""#);
    run(r#"low = "LOW""#);
    run("question1 = Question()");
    run("question1.count = 1");
    run(r#"{
    score = $Question.count * 10
    if score > 100:
        $Question.type = high
    else
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
    run("class Room");
    run("$Room.level = $Room.number / 10");
    run("class Guest");
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
    run("class Channel");
    run("class Frequency");
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
    run("class Hospital");
    run("class Clinic");
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
    run("class Server");
    run("class OS");
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
    run("class Phone");
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
    run("class Item");
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
    run("class Item");
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
    count += 1
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
    run("class Question");
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
    run("def test a => a = 2");
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
    run("def test b => b = a");
    assert_eq!(run("test(c)"), serde_json::json!({}));
    assert_eq!(run("c"), 1);
}

/// Nucleoid returns a string value of a function call
#[rustfmt::skip]
#[test]
fn returns_a_string_value_of_a_function_call() {
    let mut run = runner();
    run(r#"def test a => a = "abc""#);
    run("b = 1");
    run(r#"test(b)

# return: "abc""#);
}

/// Nucleoid returns an object value of a function call
#[rustfmt::skip]
#[test]
fn returns_an_object_value_of_a_function_call() {
    let mut run = runner();
    run("def test a => a = Object()");
    run("b = 1");
    run(r#"test(b)

# return: {}"#);
}

/// Nucleoid runs a function with a variable
#[rustfmt::skip]
#[test]
fn runs_a_function_with_a_variable() {
    let mut run = runner();
    run("def test a => a + 23");
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
    run(r#"record = { "frozen": seed.value, live }"#);
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
    run("class Parent");
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
    run("class Parent");
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
    run("class Parent");
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
    def init amount:
        super(amount + 1)
        this.middle = true"#);
    run(r#"class Leaf: Middle
    def init amount:
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
    def init amount:
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
    run("class Loop");
    run("loop = Loop()");
    assert_eq!(run("loop.id"), "loop");
}

/// Nucleoid rejects an inheritance cycle introduced by redeclaration
#[rustfmt::skip]
#[test]
fn rejects_an_inheritance_cycle_introduced_by_redeclaration() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Root");
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
    run("class Item");
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
    run("class Item");
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
    run("class Item");
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
    run("class Item");
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
    run("class First");
    assert_eq!(run("count"), 1);
    assert_eq!(run("doubled"), 2);
    run("class Second");
    assert_eq!(run("count"), 2);
    assert_eq!(run("doubled"), 4);
    run("class First");
    assert_eq!(run("count"), 2);
    assert_eq!(run_error(r#"class Temporary
throw "ABORT""#), "ABORT");
    assert_eq!(run("Class.length"), 2);
    assert_eq!(run("count"), 2);
    assert_eq!(run("doubled"), 4);
    run("class Third");
    assert_eq!(run("count"), 3);
    assert_eq!(run("doubled"), 6);
    run(r#"if count > 3:
    throw "TOO_MANY_TYPES""#);
    assert_eq!(run_error("class Fourth"), "TOO_MANY_TYPES");
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
    assert_eq!(run("Math.round(1.4999999999999998)"), 1);
    assert_eq!(run("Math.round(1.5)"), 2);
    assert_eq!(run("Math.round(1.5000000000000002)"), 2);
    assert_eq!(run("Math.round(-1.4999999999999998)"), -1);
    assert_eq!(run("Math.round(-1.5)"), -1);
    assert_eq!(run("Math.round(-1.5000000000000002)"), -2);
    assert_eq!(run("Math.round(4503599627370495.5)"), 4503599627370496.0);
    assert_eq!(run("Math.round(-4503599627370495.5)"), -4503599627370495.0);
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
        let actual = run("Math.round(Number.MAX_VALUE)");
        let expected = run("(Number.MAX_VALUE)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("Math.round(-Number.MAX_VALUE)");
        let expected = run("(-Number.MAX_VALUE)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("1 / Math.round(-0.49999999999999994)");
        let expected = run("(Number.NEGATIVE_INFINITY)");
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
        let actual = run("1 / Math.round(5e-324)");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("1 / Math.round(-5e-324)");
        let expected = run("(Number.NEGATIVE_INFINITY)");
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
    run(r#"class Reading(amount):
    this.amount = amount"#);
    run("$Reading.rounded = Math.round($Reading.amount)");
    run("$Reading.inverse = 1 / $Reading.rounded");
    run("reading = Reading(-0.49999999999999994)");
    assert_eq!(run("reading.rounded"), 0);
    {
        let actual = run("reading.inverse");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("reading.amount = 0.49999999999999994");
    assert_eq!(run("reading.rounded"), 0);
    {
        let actual = run("reading.inverse");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("reading.amount = -0.5000000000000001");
    assert_eq!(run("reading.rounded"), -1);
    assert_eq!(run("reading.inverse"), -1);
}

/// Nucleoid indexes and slices strings by Unicode scalar values
#[rustfmt::skip]
#[test]
fn indexes_and_slices_strings_by_unicode_scalar_values() {
    let mut run = runner();
    run("accent = String.fromCharCode(233)");
    run("symbol = String.fromCharCode(128512)");
    run(r#"text = "A" + accent + symbol + "Z""#);
    assert_eq!(run("text.length"), 4);
    {
        let actual = run("text[1]");
        let expected = run("(accent)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("text[2]");
        let expected = run("(symbol)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("text[-1]"), "Z");
    assert_eq!(run("text.charAt()"), "A");
    {
        let actual = run("text.charAt(1)");
        let expected = run("(accent)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("text.charAt(2)");
        let expected = run("(symbol)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("text.charAt(4)"), "");
    assert_eq!(run("text.charCodeAt(1)"), 233);
    assert_eq!(run("text.charCodeAt(2)"), 128512);
    assert_eq!(run("Number.isNaN(text.charCodeAt(4))"), true);
    {
        let actual = run("text[1:3]");
        let expected = run("(accent + symbol)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("text[-3:-1]");
        let expected = run("(accent + symbol)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("text[3:1]"), "");
    {
        let actual = run("text[:]");
        let expected = run("(text)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("text.slice()");
        let expected = run("(text)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("text.slice(1, 3)");
        let expected = run("(accent + symbol)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("text.substring(1, 3)");
        let expected = run("(accent + symbol)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("text.substr(1, 3)");
        let expected = run("(accent + symbol)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("text.slice(-3, -1)");
        let expected = run("(accent + symbol)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("text.slice(3, 1)"), "");
    assert_eq!(run("text.slice(100)"), "");
    {
        let actual = run("text.slice(-100, 100)");
        let expected = run("(text)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run(r#""".slice()"#), "");
    assert_eq!(run(r#""".charAt(0)"#), "");
    run("index = 1");
    run("character = text.charAt(index)");
    run("part = text[index:3]");
    {
        let actual = run("character");
        let expected = run("(accent)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("part");
        let expected = run("(accent + symbol)");
        assert_eq!(actual, expected);
    }
    run("index = 2");
    {
        let actual = run("character");
        let expected = run("(symbol)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("part");
        let expected = run("(symbol)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid restores enclosing null tracking after failed local assignments
#[rustfmt::skip]
#[test]
fn restores_enclosing_null_tracking_after_failed_local_assignments() {
    let mut run = runner();
    run(r#"def fail():
    throw "EXPECTED""#);
    run(r#"def recover local, value:
    try:
        local = value + fail()
    catch error:
        if error != "EXPECTED":
            throw error
    return local"#);
    run("input = null");
    run("result = input + recover(1, 0)");
    assert_eq!(run("result"), serde_json::Value::Null);
    run("input = 2");
    assert_eq!(run("result"), 3);
    run("result = 5 + recover(1, null)");
    assert_eq!(run("result"), 6);
}

/// Nucleoid restores enclosing null tracking after failed property assignments
#[rustfmt::skip]
#[test]
fn restores_enclosing_null_tracking_after_failed_property_assignments() {
    let mut run = runner();
    run("class Item");
    run("item = Item()");
    run("item.amount = 1");
    run(r#"def fail():
    throw "EXPECTED""#);
    run(r#"def recover value:
    try:
        item.amount = value + fail()
    catch error:
        if error != "EXPECTED":
            throw error
    return item.amount"#);
    run("input = null");
    run("result = input + recover(0)");
    assert_eq!(run("result"), serde_json::Value::Null);
    assert_eq!(run("item.amount"), 1);
    run("input = 2");
    assert_eq!(run("result"), 3);
    run("result = 5 + recover(null)");
    assert_eq!(run("result"), 6);
    assert_eq!(run("item.amount"), 1);
}

/// Nucleoid repeats strings with catchable allocation errors
#[rustfmt::skip]
#[test]
fn repeats_strings_with_catchable_allocation_errors() {
    let (mut run, mut run_error) = crate::common::runners();
    assert_eq!(run(r#""ab".repeat(2)"#), "abab");
    assert_eq!(run(r#""ab".repeat(2.9)"#), "abab");
    assert_eq!(run(r#""ab".repeat(0)"#), "");
    assert_eq!(run(r#""ab".repeat(-1)"#), "");
    assert_eq!(run(r#""ab".repeat()"#), "");
    assert_eq!(run(r#""ab".repeat(null)"#), "");
    assert_eq!(run(r#""ab".repeat(Number.NaN)"#), "");
    assert_eq!(run(r#""ab".repeat(Number.NEGATIVE_INFINITY)"#), "");
    assert_eq!(run(r#""".repeat(Number.MAX_VALUE)"#), "");
    assert_eq!(run(r#""".repeat(Number.POSITIVE_INFINITY)"#), "");
    run("symbol = String.fromCharCode(128512)");
    {
        let actual = run("symbol.repeat(3)");
        let expected = run("(symbol + symbol + symbol)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run_error(r#""ab".repeat(Number.MAX_VALUE)"#), "TypeError: Repeated string is too large");
    assert_eq!(run_error(r#""a".repeat(Number.POSITIVE_INFINITY)"#), "TypeError: Repeated string is too large");
    run("count = 2");
    run(r#"repeated = "ab".repeat(count)"#);
    assert_eq!(run_error("count = Number.POSITIVE_INFINITY"), "TypeError: Repeated string is too large");
    assert_eq!(run("count"), 2);
    assert_eq!(run("repeated"), "abab");
    run("count = 3");
    assert_eq!(run("repeated"), "ababab");
}

/// Nucleoid preserves declaration order when standing rules are re-evaluated
#[rustfmt::skip]
#[test]
fn preserves_declaration_order_when_standing_rules_are_re_evaluated() {
    let mut run = runner();
    run("trigger = 0");
    run("enabled = false");
    run("result = 0");
    run(r#"if enabled and trigger > 0:
    result = 1"#);
    run(r#"if trigger > 0:
    result = 2"#);
    run("enabled = true");
    run("trigger = 1");
    assert_eq!(run("result"), 2);
    run("trigger = 0");
    run("enabled = false");
    run("enabled = true");
    run("trigger = 1");
    assert_eq!(run("result"), 2);
    run(r#"if enabled and trigger > 0:
    result = 3"#);
    run("trigger = 2");
    assert_eq!(run("result"), 3);
}

/// Nucleoid settles queued upstream dependencies before their dependents
#[rustfmt::skip]
#[test]
fn settles_queued_upstream_dependencies_before_their_dependents() {
    let mut run = runner();
    run("source = 1");
    run("middle = source + 1");
    run("last = middle + 1");
    run("total = source + last");
    assert_eq!(run("total"), 4);
    run("middle = source + 2");
    assert_eq!(run("total"), 5);
    run("source = 2");
    assert_eq!(run("middle"), 4);
    assert_eq!(run("last"), 5);
    assert_eq!(run("total"), 7);
    run("source = 3");
    assert_eq!(run("total"), 9);
}

/// Nucleoid settles class property dependencies before validation rules
#[rustfmt::skip]
#[test]
fn settles_class_property_dependencies_before_validation_rules() {
    let mut run = runner();
    run(r#"class Reading(amount):
    this.amount = amount"#);
    run("$Reading.doubled = $Reading.amount * 2");
    run("$Reading.margin = $Reading.doubled - $Reading.amount");
    run(r#"if $Reading.margin < 0:
    throw "NEGATIVE_MARGIN""#);
    run("reading = Reading(1)");
    assert_eq!(run("reading.margin"), 1);
    run("$Reading.doubled = $Reading.amount * 3");
    assert_eq!(run("reading.margin"), 2);
    run("reading.amount = 10");
    assert_eq!(run("reading.doubled"), 30);
    assert_eq!(run("reading.margin"), 20);
}

/// Nucleoid revisits dependencies changed later in the same propagation
#[rustfmt::skip]
#[test]
fn revisits_dependencies_changed_later_in_the_same_propagation() {
    let (mut run, mut run_error) = crate::common::runners();
    run("first = 0");
    run("second = 0");
    run("total = first + second");
    run(r#"if first > 0 and first < 4:
    second = first + 1"#);
    run(r#"if second > 0 and second < 4:
    first = second + 1"#);
    run("first = 1");
    assert_eq!(run("first"), 3);
    assert_eq!(run("second"), 4);
    assert_eq!(run("total"), 7);
    run(r#"if second == 3:
    throw "INVALID_SECOND""#);
    assert_eq!(run_error("first = 2"), "INVALID_SECOND");
    assert_eq!(run("first"), 3);
    assert_eq!(run("second"), 4);
    assert_eq!(run("total"), 7);
    run(r#"if second == 3:
    pass"#);
    run("first = 2");
    assert_eq!(run("first"), 4);
    assert_eq!(run("second"), 3);
    assert_eq!(run("total"), 7);
}

/// Nucleoid stops propagation feedback when assignments keep the same value
#[rustfmt::skip]
#[test]
fn stops_propagation_feedback_when_assignments_keep_the_same_value() {
    let mut run = runner();
    run("first = 0");
    run("second = 0");
    run(r#"if first > 0:
    second = 1"#);
    run(r#"if second > 0:
    first = 1"#);
    run("first = 1");
    assert_eq!(run("first"), 1);
    assert_eq!(run("second"), 1);
    run("count = 0");
    run("values = [0]");
    run(r#"if count > 0 and count < 4:
    values = [Number.NaN]"#);
    run(r#"if Number.isNaN(values[0]):
    count += 1"#);
    run("count = 1");
    assert_eq!(run("count"), 2);
    assert_eq!(run("Number.isNaN(values[0])"), true);
}

/// Nucleoid propagates changes between positive and negative zero
#[rustfmt::skip]
#[test]
fn propagates_changes_between_positive_and_negative_zero() {
    let mut run = runner();
    run("negative = false");
    run("zero = 0.0");
    run("inverse = 1 / zero");
    run(r#"if negative:
    zero = -0.0
else
    zero = 0.0"#);
    {
        let actual = run("inverse");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("negative = true");
    {
        let actual = run("inverse");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("negative = false");
    {
        let actual = run("inverse");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid keeps conditional loop bodies imperative
#[rustfmt::skip]
#[test]
fn keeps_conditional_loop_bodies_imperative() {
    let mut run = runner();
    run("class Item");
    run("item = Item()");
    run("item.amount = 0");
    run("limit = 0");
    run(r#"for current of Item:
    if limit > 0:
        current.amount = limit"#);
    run("limit = 2");
    assert_eq!(run("item.amount"), 0);
    run(r#"for current of Item:
    if limit > 0:
        current.amount = limit"#);
    assert_eq!(run("item.amount"), 2);
    run("limit = 3");
    assert_eq!(run("item.amount"), 2);
}

/// Nucleoid cancels propagation queued by a caught failure
#[rustfmt::skip]
#[test]
fn cancels_propagation_queued_by_a_caught_failure() {
    let mut run = runner();
    run("trigger = 0");
    run("source = 1");
    run("observations = 0");
    run(r#"if source > 0:
    observations += 1"#);
    run(r#"if trigger > 0:
    try:
        source = 2
        throw "ABORT"
    catch error:
        if error != "ABORT":
            throw error"#);
    run("remaining = trigger + 10");
    run("trigger = 1");
    assert_eq!(run("source"), 1);
    assert_eq!(run("observations"), 1);
    assert_eq!(run("remaining"), 11);
    run("source = 3");
    assert_eq!(run("observations"), 2);
}

/// Nucleoid preserves earlier queued work across nested caught failures
#[rustfmt::skip]
#[test]
fn preserves_earlier_queued_work_across_nested_caught_failures() {
    let mut run = runner();
    run("trigger = 0");
    run("source = 0");
    run("observations = 0");
    run(r#"if source > 0:
    observations += 1"#);
    run(r#"if trigger > 0:
    source = 1
    try:
        source = 2
        try:
            source = 3
            throw "INNER"
        catch inner:
            if inner != "INNER":
                throw inner
        throw "OUTER"
    catch outer:
        if outer != "OUTER":
            throw outer"#);
    run("trigger = 1");
    assert_eq!(run("source"), 1);
    assert_eq!(run("observations"), 1);
}

/// Nucleoid restores deletion tracking before entering a catch body
#[rustfmt::skip]
#[test]
fn restores_deletion_tracking_before_entering_a_catch_body() {
    let mut run = runner();
    run("trigger = 0");
    run("missing = null");
    run(r#"if trigger > 0:
    try:
        temporary = 1
        delete temporary
        throw "ABORT"
    catch error:
        if error != "ABORT":
            throw error
        try:
            temporary
        catch absent:
            missing = absent"#);
    run("trigger = 1");
    {
        let actual = run("missing");
        let expected = run(r#"(ReferenceError("temporary is not defined"))"#);
        assert_eq!(actual, expected);
    }
}

/// Nucleoid ends temporary undefined reads when a deletion cascade finishes
#[rustfmt::skip]
#[test]
fn ends_temporary_undefined_reads_when_a_deletion_cascade_finishes() {
    let mut run = runner();
    run("trigger = 0");
    run("source = 1");
    run("dependent = source + 1");
    run(r#"if trigger > 0:
    delete source"#);
    run(r#"def read_after_update():
    trigger = 1
    try:
        source
    catch error:
        return error
    return "NO_ERROR""#);
    {
        let actual = run("read_after_update()");
        let expected = run(r#"(ReferenceError("source is not defined"))"#);
        assert_eq!(actual, expected);
    }
    assert_eq!(run("dependent"), serde_json::Value::Null);
    run("source = 3");
    assert_eq!(run("dependent"), 4);
}

/// Nucleoid tracks named functions forwarded as values
#[rustfmt::skip]
#[test]
fn tracks_named_functions_forwarded_as_values() {
    let mut run = runner();
    run("def transform number => number * 2");
    run("def choose() => transform");
    run("callback = transform");
    run("forwarded = callback");
    run("returned = choose()");
    run("direct = forwarded(3)");
    run("indirect = returned(3)");
    assert_eq!(run("direct"), 6);
    assert_eq!(run("indirect"), 6);
    run("def transform number => number * 3");
    assert_eq!(run("direct"), 9);
    assert_eq!(run("indirect"), 9);
}

/// Nucleoid tracks named callbacks over lists and class populations
#[rustfmt::skip]
#[test]
fn tracks_named_callbacks_over_lists_and_class_populations() {
    let mut run = runner();
    run("numbers = [1, 2, 3]");
    run("def transform number => number * 2");
    run("def matches number => number > 0");
    run("def combine total number => total + number");
    run("mapped = numbers.map(transform)");
    run("filtered = numbers.filter(matches)");
    run("found = numbers.find(matches)");
    run("all = numbers.every(matches)");
    run("any = numbers.some(matches)");
    run("total = numbers.reduce(combine, 1)");
    assert_eq!(run("mapped"), serde_json::json!([2, 4, 6]));
    assert_eq!(run("filtered"), serde_json::json!([1, 2, 3]));
    assert_eq!(run("found"), 1);
    assert_eq!(run("all"), true);
    assert_eq!(run("any"), true);
    assert_eq!(run("total"), 7);
    run("def transform number => number * 3");
    run("def matches number => number > 3");
    run("def combine total number => total * number");
    assert_eq!(run("mapped"), serde_json::json!([3, 6, 9]));
    assert_eq!(run("filtered"), serde_json::json!([]));
    assert_eq!(run("found"), serde_json::Value::Null);
    assert_eq!(run("all"), false);
    assert_eq!(run("any"), false);
    assert_eq!(run("total"), 6);
    run(r#"class Sample(amount):
    this.amount = amount"#);
    run("def reading sample => sample.amount * 2");
    run("sample = Sample(2)");
    run("readings = Sample.map(reading)");
    assert_eq!(run("readings"), serde_json::json!([4]));
    run("def reading sample => sample.amount * 3");
    assert_eq!(run("readings"), serde_json::json!([6]));
    run("sample.amount = 3");
    run("Sample(4)");
    assert_eq!(run("readings"), serde_json::json!([9, 12]));
}

/// Nucleoid tracks named functions passed to higher order calls
#[rustfmt::skip]
#[test]
fn tracks_named_functions_passed_to_higher_order_calls() {
    let mut run = runner();
    run("def transform number => number * 2");
    run("def apply operation number => operation(number)");
    run("input = 3");
    run("result = apply(transform, input)");
    assert_eq!(run("result"), 6);
    run("def transform number => number * 3");
    assert_eq!(run("result"), 9);
    run("input = 4");
    assert_eq!(run("result"), 12);
}

/// Nucleoid tracks calls through reassigned lambda variables
#[rustfmt::skip]
#[test]
fn tracks_calls_through_reassigned_lambda_variables() {
    let mut run = runner();
    run("input = 3");
    run("callback = number => number * 2");
    run("result = callback(input)");
    run("callbacks = [callback]");
    run("indexed = callbacks[0](input)");
    assert_eq!(run("result"), 6);
    assert_eq!(run("indexed"), 6);
    run("callback = number => number * 3");
    assert_eq!(run("result"), 9);
    assert_eq!(run("indexed"), 9);
    run("input = 4");
    assert_eq!(run("result"), 12);
    assert_eq!(run("indexed"), 12);
}

/// Nucleoid tracks dotted and indexed callable properties consistently
#[rustfmt::skip]
#[test]
fn tracks_dotted_and_indexed_callable_properties_consistently() {
    let mut run = runner();
    run("class Handler");
    run("handler = Handler()");
    run("input = 2");
    run("dotted = handler.operation(input)");
    run(r#"indexed = handler["operation"](input)"#);
    assert_eq!(run("dotted"), serde_json::Value::Null);
    assert_eq!(run("indexed"), serde_json::Value::Null);
    run("handler.operation = number => number + 1");
    assert_eq!(run("dotted"), 3);
    assert_eq!(run("indexed"), 3);
    run("handler.operation = number => number + 2");
    assert_eq!(run("dotted"), 4);
    assert_eq!(run("indexed"), 4);
    run("input = 3");
    assert_eq!(run("dotted"), 5);
    assert_eq!(run("indexed"), 5);
}

/// Nucleoid freezes a function definition without freezing its body dependencies
#[rustfmt::skip]
#[test]
fn freezes_a_function_definition_without_freezing_its_body_dependencies() {
    let mut run = runner();
    run("factor = 2");
    run("def transform number => number * factor");
    run("snapshot = transform.value");
    run("frozen = snapshot(3)");
    run("live = transform(3)");
    assert_eq!(run("frozen"), 6);
    assert_eq!(run("live"), 6);
    run("def transform number => number * factor * 10");
    assert_eq!(run("frozen"), 6);
    assert_eq!(run("live"), 60);
    run("factor = 4");
    assert_eq!(run("frozen"), 12);
    assert_eq!(run("live"), 120);
}

/// Nucleoid defers null function aliases and restores callers after deletion
#[rustfmt::skip]
#[test]
fn defers_null_function_aliases_and_restores_callers_after_deletion() {
    let mut run = runner();
    run("callback = null");
    run("input = 3");
    run("offset = 0");
    run("result = callback(input) + offset");
    assert_eq!(run("result"), serde_json::Value::Null);
    run("callback = number => number * 2");
    assert_eq!(run("result"), 6);
    run("callback = null");
    assert_eq!(run("result"), serde_json::Value::Null);
    run("callback = number => number * 3");
    assert_eq!(run("result"), 9);
    run("delete callback");
    assert_eq!(run("result"), serde_json::Value::Null);
    run("offset = 1");
    run("input = 4");
    assert_eq!(run("result"), serde_json::Value::Null);
    run("missing = null");
    run(r#"try:
    callback(3)
catch error:
    missing = error"#);
    {
        let actual = run("missing");
        let expected = run(r#"(ReferenceError("callback is not defined"))"#);
        assert_eq!(actual, expected);
    }
    run("trigger = 0");
    run(r#"if trigger > 0:
    callback(3)"#);
    run(r#"try:
    trigger = 1
catch error:
    missing = error"#);
    {
        let actual = run("missing");
        let expected = run(r#"(ReferenceError("callback is not defined"))"#);
        assert_eq!(actual, expected);
    }
    assert_eq!(run("trigger"), 0);
    run("callback = number => number * 4");
    assert_eq!(run("result"), 17);
    run("input = 5");
    assert_eq!(run("result"), 21);
}

/// Nucleoid restores nullable callable properties without changing frozen references
#[rustfmt::skip]
#[test]
fn restores_nullable_callable_properties_without_changing_frozen_references() {
    let mut run = runner();
    run("class Handler");
    run("handler = Handler()");
    run("handler.operation = number => number + 1");
    run("snapshot = handler.operation.value");
    run("frozen = snapshot(2)");
    run("input = 2");
    run("offset = 0");
    run("dotted = handler.operation(input) + offset");
    run(r#"indexed = handler["operation"](input) + offset"#);
    run("handler.operation = null");
    assert_eq!(run("dotted"), serde_json::Value::Null);
    assert_eq!(run("indexed"), serde_json::Value::Null);
    assert_eq!(run("frozen"), 3);
    run("handler.operation = number => number + 2");
    assert_eq!(run("dotted"), 4);
    assert_eq!(run("indexed"), 4);
    run("delete handler.operation");
    assert_eq!(run("dotted"), serde_json::Value::Null);
    assert_eq!(run("indexed"), serde_json::Value::Null);
    assert_eq!(run("frozen"), 3);
    run("offset = 1");
    run("input = 3");
    assert_eq!(run("dotted"), serde_json::Value::Null);
    assert_eq!(run("indexed"), serde_json::Value::Null);
    run("handler.operation = number => number + 3");
    assert_eq!(run("dotted"), 7);
    assert_eq!(run("indexed"), 7);
    assert_eq!(run("frozen"), 3);
}

/// Nucleoid lets null and non callable parameters shadow outer functions
#[rustfmt::skip]
#[test]
fn lets_null_and_non_callable_parameters_shadow_outer_functions() {
    let mut run = runner();
    run("def operation number => 99");
    run("def apply operation number => operation(number)");
    run("callback = null");
    run("result = apply(callback, 3)");
    assert_eq!(run("result"), serde_json::Value::Null);
    run("callback = number => number * 2");
    assert_eq!(run("result"), 6);
    run("callback = number => number * 3");
    assert_eq!(run("result"), 9);
    run("failure = null");
    run(r#"try:
    apply(7, 3)
catch error:
    failure = error"#);
    {
        let actual = run("failure");
        let expected = run(r#"(TypeError("operation is not a function"))"#);
        assert_eq!(actual, expected);
    }
    assert_eq!(run("operation(3)"), 99);
}

/// Nucleoid rolls back non callable replacements used by existing callers
#[rustfmt::skip]
#[test]
fn rolls_back_non_callable_replacements_used_by_existing_callers() {
    let mut run = runner();
    run("callback = number => number * 2");
    run("result = callback(3)");
    run("failure = null");
    run(r#"try:
    callback = 7
catch error:
    failure = error"#);
    {
        let actual = run("failure");
        let expected = run(r#"(TypeError("callback is not a function"))"#);
        assert_eq!(actual, expected);
    }
    assert_eq!(run("result"), 6);
    assert_eq!(run("callback(4)"), 8);
    run("class Handler");
    run("handler = Handler()");
    run("handler.operation = number => number + 1");
    run("propertyResult = handler.operation(3)");
    run(r#"try:
    handler.operation = 7
catch error:
    failure = error"#);
    {
        let actual = run("failure");
        let expected = run(r#"(TypeError("handler.operation is not a function"))"#);
        assert_eq!(actual, expected);
    }
    assert_eq!(run("propertyResult"), 4);
    assert_eq!(run("handler.operation(4)"), 5);
    run("handler.operation = number => number + 2");
    assert_eq!(run("propertyResult"), 5);
}

/// Nucleoid rolls back failed redefinitions of named callbacks
#[rustfmt::skip]
#[test]
fn rolls_back_failed_redefinitions_of_named_callbacks() {
    let mut run = runner();
    run("def transform number => number * 2");
    run("numbers = [1, 2]");
    run("mapped = numbers.map(transform)");
    run("failure = null");
    run(r#"try:
    def transform number:
        if number == 2:
            throw "REJECTED_CALLBACK"
        return number * 3
catch error:
    failure = error"#);
    assert_eq!(run("failure"), "REJECTED_CALLBACK");
    assert_eq!(run("mapped"), serde_json::json!([2, 4]));
    assert_eq!(run("transform(3)"), 6);
    run("def transform number => number * 4");
    assert_eq!(run("mapped"), serde_json::json!([4, 8]));
}

/// Nucleoid replaces functions and callable variables under the same name
#[rustfmt::skip]
#[test]
fn replaces_functions_and_callable_variables_under_the_same_name() {
    let mut run = runner();
    run("def callback number => number * 2");
    run("forwarded = callback");
    run("direct = callback(3)");
    run("indirect = forwarded(3)");
    run("callback = number => number * 3");
    assert_eq!(run("direct"), 9);
    assert_eq!(run("indirect"), 9);
    run("failure = null");
    run(r#"try:
    def callback number:
        throw "REJECTED_BINDING"
catch error:
    failure = error"#);
    assert_eq!(run("failure"), "REJECTED_BINDING");
    assert_eq!(run("direct"), 9);
    assert_eq!(run("indirect"), 9);
    assert_eq!(run("callback(4)"), 12);
    run("def callback number => number * 4");
    assert_eq!(run("direct"), 12);
    assert_eq!(run("indirect"), 12);
    run(r#"try:
    callback = 7
catch error:
    failure = error"#);
    {
        let actual = run("failure");
        let expected = run(r#"(TypeError("callback is not a function"))"#);
        assert_eq!(actual, expected);
    }
    assert_eq!(run("direct"), 12);
    assert_eq!(run("indirect"), 12);
    assert_eq!(run("callback(4)"), 16);
    run("callback = null");
    assert_eq!(run("direct"), serde_json::Value::Null);
    assert_eq!(run("indirect"), serde_json::Value::Null);
    run("def callback number => number * 5");
    assert_eq!(run("direct"), 15);
    assert_eq!(run("indirect"), 15);
    run("callback = number => number * 6");
    assert_eq!(run("direct"), 18);
    assert_eq!(run("indirect"), 18);
    run("delete callback");
    assert_eq!(run("direct"), serde_json::Value::Null);
    assert_eq!(run("indirect"), serde_json::Value::Null);
    run(r#"try:
    callback(3)
catch error:
    failure = error"#);
    {
        let actual = run("failure");
        let expected = run(r#"(ReferenceError("callback is not defined"))"#);
        assert_eq!(actual, expected);
    }
    run("def callback number => number * 7");
    assert_eq!(run("direct"), 21);
    assert_eq!(run("indirect"), 21);
}

/// Nucleoid keeps live reasoning impact selections current
#[rustfmt::skip]
#[test]
fn keeps_live_reasoning_impact_selections_current() {
    let mut run = runner();
    run("a = 1");
    run("exposure = affects a");
    assert_eq!(run("exposure"), serde_json::json!([]));
    run("b = a + 1");
    run("c = b * 2");
    assert_eq!(run("exposure.map(step => step.node)"), serde_json::json!(["b", "c"]));
    run("b = 0");
    assert_eq!(run("exposure"), serde_json::json!([]));
    run("b = a + 1");
    assert_eq!(run("exposure.map(step => step.node)"), serde_json::json!(["b", "c"]));
}

/// Nucleoid keeps live reasoning current through equal valued reference changes
#[rustfmt::skip]
#[test]
fn keeps_live_reasoning_current_through_equal_valued_reference_changes() {
    let mut run = runner();
    run(r#"class Plan(rate):
    this.rate = rate"#);
    run("class User");
    run("basic = Plan(10)");
    run("pro = Plan(10)");
    run("user = User()");
    run("user.plan = basic");
    run("user.bill = user.plan.rate * 2");
    run("trace = why user.bill");
    run("user.plan = pro");
    assert_eq!(run("trace[0].holds"), 20);
    assert_eq!(run("trace[0].from"), serde_json::json!(["user", "user.plan", "pro.rate"]));
    assert_eq!(run(r#"trace.some(step => step.node == "pro.rate")"#), true);
    assert_eq!(run(r#"trace.some(step => step.node == "basic.rate")"#), false);
    run("pro.rate = 15");
    assert_eq!(run("trace[0].holds"), 30);
}

/// Nucleoid keeps live reasoning model selections current
#[rustfmt::skip]
#[test]
fn keeps_live_reasoning_model_selections_current() {
    let mut run = runner();
    run("inventory = model |> why");
    assert_eq!(run("inventory"), serde_json::json!([]));
    run("a = 1");
    run("b = a + 1");
    assert_eq!(run("inventory.map(step => step.node)"), serde_json::json!(["a", "b"]));
    run("delete b");
    assert_eq!(run("inventory.map(step => step.node)"), serde_json::json!(["a"]));
    run("b = a + 2");
    run("trace = why b");
    assert_eq!(run("inventory.map(step => step.node)"), serde_json::json!(["a", "b"]));
    assert_eq!(run("inventory[1].holds"), 3);
    run("trace = a + 0");
    assert_eq!(run("inventory.map(step => step.node)"), serde_json::json!(["a", "b", "trace"]));
    assert_eq!(run("(affects a).map(step => step.node)"), serde_json::json!(["b", "trace"]));
    run("trace = why b");
    assert_eq!(run("inventory.map(step => step.node)"), serde_json::json!(["a", "b"]));
    assert_eq!(run("(affects a).map(step => step.node)"), serde_json::json!(["b"]));
}

/// Nucleoid keeps live reasoning subscribed to an absent property
#[rustfmt::skip]
#[test]
fn keeps_live_reasoning_subscribed_to_an_absent_property() {
    let mut run = runner();
    run("class Item");
    run("item = Item()");
    run("trace = why item.amount");
    assert_eq!(run("trace"), serde_json::json!([]));
    run("item.amount = 3");
    assert_eq!(run("trace[0].holds"), 3);
    assert_eq!(run("trace[0].node"), "item.amount");
    run("delete item.amount");
    assert_eq!(run("trace"), serde_json::json!([]));
    run("item.amount = 4");
    assert_eq!(run("trace[0].holds"), 4);
}

/// Nucleoid keeps live reasoning observers outside impact walks
#[rustfmt::skip]
#[test]
fn keeps_live_reasoning_observers_outside_impact_walks() {
    let mut run = runner();
    run("a = 1");
    run("b = a + 1");
    run("trace = why b");
    run("size = trace.length");
    run("exposure = affects a");
    assert_eq!(run("exposure.map(step => step.node)"), serde_json::json!(["b"]));
    run("secondTrace = why b");
    run("secondSize = secondTrace.length");
    assert_eq!(run("exposure.map(step => step.node)"), serde_json::json!(["b"]));
}

/// Nucleoid keeps live reasoning snapshots fixed
#[rustfmt::skip]
#[test]
fn keeps_live_reasoning_snapshots_fixed() {
    let mut run = runner();
    run("a = 1");
    run("exposure = affects a");
    run("frozen = exposure.value");
    run("b = a + 1");
    assert_eq!(run("exposure.map(step => step.node)"), serde_json::json!(["b"]));
    assert_eq!(run("frozen"), serde_json::json!([]));
    run("a = 2");
    assert_eq!(run("exposure[0].holds"), 3);
    assert_eq!(run("frozen"), serde_json::json!([]));
}

/// Nucleoid restores live reasoning subscriptions after a caught failure
#[rustfmt::skip]
#[test]
fn restores_live_reasoning_subscriptions_after_a_caught_failure() {
    let (mut run, mut run_error) = crate::common::runners();
    run("a = 1");
    run("alternative = 1");
    run("b = a + 1");
    run("trace = why b");
    run("exposure = affects a");
    assert_eq!(run_error(r#"b = alternative + 1
throw "REJECTED_REWRITE""#), "REJECTED_REWRITE");
    assert_eq!(run("trace[0].from"), serde_json::json!(["a"]));
    assert_eq!(run("exposure.map(step => step.node)"), serde_json::json!(["b"]));
    run("alternative = 4");
    assert_eq!(run("trace[0].holds"), 2);
    run("a = 3");
    assert_eq!(run("trace[0].holds"), 4);
    run("b = alternative + 1");
    assert_eq!(run("trace[0].from"), serde_json::json!(["alternative"]));
    assert_eq!(run("exposure"), serde_json::json!([]));
}

/// Nucleoid refreshes live reasoning declaration metadata without a value change
#[rustfmt::skip]
#[test]
fn refreshes_live_reasoning_declaration_metadata_without_a_value_change() {
    let mut run = runner();
    run("a = 1");
    run("b = a + 2");
    run("trace = why b");
    run("b = a + 1 + 1");
    assert_eq!(run("trace[0].holds"), 3);
    assert_eq!(run("trace[0].rule"), "b = a+1+1");
    assert_eq!(run("trace[0].from"), serde_json::json!(["a"]));
    run("b = 3");
    assert_eq!(run("trace[0].state"), "stated");
    assert_eq!(run("trace.map(step => step.node)"), serde_json::json!(["b"]));
    run("b = a + 2");
    assert_eq!(run("trace[0].state"), "derived");
    assert_eq!(run("trace.map(step => step.node)"), serde_json::json!(["b", "a"]));
}

/// Nucleoid replaces live reasoning ancestry and its subscriptions
#[rustfmt::skip]
#[test]
fn replaces_live_reasoning_ancestry_and_its_subscriptions() {
    let mut run = runner();
    run("a = 1");
    run("b = a + 2");
    run("c = b * 2");
    run("trace = why c");
    run("d = 0");
    run("b = a + d + 2");
    assert_eq!(run("trace.length"), 4);
    assert_eq!(run("trace[0].holds"), 6);
    assert_eq!(run("trace[1].from"), serde_json::json!(["a", "d"]));
    run("d = 5");
    assert_eq!(run("trace[0].holds"), 16);
    run("b = a + 2");
    run("d = 10");
    assert_eq!(run("trace.length"), 3);
    assert_eq!(run("trace[0].holds"), 6);
    assert_eq!(run(r#"trace.some(step => step.node == "d")"#), false);
}

/// Nucleoid preserves declaration order in live reasoning impact selections
#[rustfmt::skip]
#[test]
fn preserves_declaration_order_in_live_reasoning_impact_selections() {
    let mut run = runner();
    run("source = 1");
    run("left = source + 1");
    run("right = source + 2");
    run("exposure = affects source");
    assert_eq!(run("exposure.map(step => step.node)"), serde_json::json!(["left", "right"]));
    run("left = source + 1");
    assert_eq!(run("exposure.map(step => step.node)"), serde_json::json!(["right", "left"]));
    run("source = 2");
    assert_eq!(run("exposure.map(step => step.node)"), serde_json::json!(["right", "left"]));
    assert_eq!(run("exposure[0].holds"), 4);
    assert_eq!(run("exposure[1].holds"), 3);
}

/// Nucleoid keeps live reasoning current through function calls
#[rustfmt::skip]
#[test]
fn keeps_live_reasoning_current_through_function_calls() {
    let mut run = runner();
    run("a = 1");
    run("def explain() => affects a");
    run("exposure = explain()");
    run("b = a + 1");
    assert_eq!(run("exposure.map(step => step.node)"), serde_json::json!(["b"]));
    assert_eq!(run("(affects a).map(step => step.node)"), serde_json::json!(["b"]));
    run("b = 0");
    assert_eq!(run("exposure"), serde_json::json!([]));
}

/// Nucleoid keeps class level reasoning properties live
#[rustfmt::skip]
#[test]
fn keeps_class_level_reasoning_properties_live() {
    let mut run = runner();
    run(r#"class Reading(level):
    this.level = level"#);
    run(r#"class Audit(target):
    this.target = target"#);
    run("reading = Reading(1)");
    run("audit = Audit(reading)");
    run("$Audit.exposure = affects $Audit.target.level");
    assert_eq!(run("audit.exposure"), serde_json::json!([]));
    run("reading.double = reading.level * 2");
    assert_eq!(run("audit.exposure.map(step => step.node)"), serde_json::json!(["reading.double"]));
    assert_eq!(run("audit.exposure[0].holds"), 2);
    run("reading.level = 3");
    assert_eq!(run("audit.exposure[0].holds"), 6);
    run("reading.double = 6");
    assert_eq!(run("audit.exposure"), serde_json::json!([]));
}

/// Nucleoid enforces live reasoning guards on structural changes
#[rustfmt::skip]
#[test]
fn enforces_live_reasoning_guards_on_structural_changes() {
    let (mut run, mut run_error) = crate::common::runners();
    run("source = 1");
    run("limit = 1");
    run(r#"if (affects source).length > limit:
    throw "TOO_MANY_DEPENDENTS""#);
    run("first = source + 1");
    assert_eq!(run_error("second = source + 2"), "TOO_MANY_DEPENDENTS");
    assert_eq!(run("(affects source).map(step => step.node)"), serde_json::json!(["first"]));
    run("limit = 2");
    run("second = source + 2");
    assert_eq!(run("(affects source).map(step => step.node)"), serde_json::json!(["first", "second"]));
}

/// Nucleoid clears and restores a live explanation when its source is deleted
#[rustfmt::skip]
#[test]
fn clears_and_restores_a_live_explanation_when_its_source_is_deleted() {
    let mut run = runner();
    run("a = 1");
    run("trace = why a");
    run("delete a");
    assert_eq!(run("trace"), serde_json::json!([]));
    run("a = 2");
    assert_eq!(run("trace[0].holds"), 2);
    assert_eq!(run("trace[0].node"), "a");
    run("trace = 99");
    run("a = 3");
    assert_eq!(run("trace"), 99);
    assert_eq!(run("(affects a)"), serde_json::json!([]));
}

/// Nucleoid preserves queued live reasoning across a caught structural failure
#[rustfmt::skip]
#[test]
fn preserves_queued_live_reasoning_across_a_caught_structural_failure() {
    let mut run = runner();
    run("trigger = 0");
    run("inventory = model |> why");
    run(r#"if trigger > 0:
    try:
        class Temporary
        throw "ABORT"
    catch error:
        if error != "ABORT":
            throw error"#);
    run("remaining = trigger + 10");
    run("trigger = 1");
    assert_eq!(run(r#"inventory.some(step => step.node == "$Temporary")"#), false);
    assert_eq!(run(r#"inventory.find(step => step.node == "trigger").holds"#), 1);
    assert_eq!(run(r#"inventory.find(step => step.node == "remaining").holds"#), 11);
}

/// Nucleoid treats this and self as interchangeable instance keywords
#[rustfmt::skip]
#[test]
fn treats_this_and_self_as_interchangeable_instance_keywords() {
    let mut run = runner();
    run(r#"class Counter(count: int):
    self.count = count

    def same_receiver() => this == self

    def set_this(count: int):
        this.count = count
        return self.count

    def set_self(count: int):
        self.count = count
        return this.count"#);
    run(r#"class SpecialCounter: Counter
    def init(count: int):
        super(count)
        this.kind = "SPECIAL"
        self.active = true"#);
    run("$Counter.doubled = this.count + self.count");
    run("counter = Counter(2)");
    run("other = Counter(5)");
    run("special = SpecialCounter(7)");
    assert_eq!(run("counter.count"), 2);
    assert_eq!(run("other.count"), 5);
    assert_eq!(run("special.count"), 7);
    assert_eq!(run("special.kind"), "SPECIAL");
    assert_eq!(run("special.active"), true);
    assert_eq!(run("counter.same_receiver()"), true);
    assert_eq!(run("other.same_receiver()"), true);
    assert_eq!(run("counter.doubled"), 4);
    assert_eq!(run("other.doubled"), 10);
    assert_eq!(run("special.doubled"), 14);
    assert_eq!(run("counter.set_self(3)"), 3);
    assert_eq!(run("counter.count"), 3);
    assert_eq!(run("counter.doubled"), 6);
    assert_eq!(run("other.count"), 5);
    assert_eq!(run("other.set_this(7)"), 7);
    assert_eq!(run("other.count"), 7);
    assert_eq!(run("other.doubled"), 14);
    assert_eq!(run("counter.count"), 3);
}

/// Nucleoid rejects this and self outside an instance or as assignment targets
#[rustfmt::skip]
#[test]
fn rejects_this_and_self_outside_an_instance_or_as_assignment_targets() {
    let (_run, mut run_error) = crate::common::runners();
    assert_eq!(run_error("this"), "ReferenceError: this is not defined");
    assert_eq!(run_error("self"), "ReferenceError: this is not defined");
    assert_eq!(run_error("this = 1"), "TypeError: Cannot assign to 'this'");
    assert_eq!(run_error("self = 1"), "TypeError: Cannot assign to 'this'");
}

/// Nucleoid keeps expression bodied named functions reactive
#[rustfmt::skip]
#[test]
fn keeps_expression_bodied_named_functions_reactive() {
    let mut run = runner();
    run("source = 3");
    run("def next number: int => number + 1");
    run("def twice number: int => next(number) * 2");
    run("result = twice(source)");
    assert_eq!(run("result"), 8);
    run("source = 5");
    assert_eq!(run("result"), 12);
    run("def next number: int => number + 2");
    assert_eq!(run("result"), 14);
}

/// Nucleoid keeps expression bodied methods and initializers equivalent
#[rustfmt::skip]
#[test]
fn keeps_expression_bodied_methods_and_initializers_equivalent() {
    let mut run = runner();
    run(r#"class Gauge:
    def init amount: int => this.amount = amount

    def reading() => self.amount

    def set amount: int => this.amount = amount"#);
    run("gauge = Gauge(4)");
    run("observed = gauge.reading()");
    assert_eq!(run("gauge.id"), "gauge");
    assert_eq!(run("gauge.amount"), 4);
    assert_eq!(run("observed"), 4);
    assert_eq!(run("gauge.set(7)"), 7);
    assert_eq!(run("gauge.amount"), 7);
    assert_eq!(run("observed"), 7);
}

/// Nucleoid preserves expression body returns and callable references
#[rustfmt::skip]
#[test]
fn preserves_expression_body_returns_and_callable_references() {
    let mut run = runner();
    run("def literal() => 42");
    run("reference = literal");
    run("frozen = literal.value");
    assert_eq!(run("reference()"), 42);
    run("def literal() => 43");
    assert_eq!(run("reference()"), 43);
    assert_eq!(run("frozen()"), 42);
    run("def assigned value => value = 2");
    assert_eq!(run("assigned(99)"), 2);
    run("def mapper() => value => value * 2");
    run("callback = mapper()");
    assert_eq!(run("callback(3)"), 6);
    run(r#"def no_return():
    42"#);
    assert_eq!(run("no_return()"), serde_json::Value::Null);
}

/// Nucleoid keeps compound arithmetic assignments equivalent
#[rustfmt::skip]
#[test]
fn keeps_compound_arithmetic_assignments_equivalent() {
    let mut run = runner();
    run("value = 20");
    run("value += 4");
    assert_eq!(run("value"), 24);
    run("value -= 6");
    assert_eq!(run("value"), 18);
    run("value *= 2 + 1");
    assert_eq!(run("value"), 54);
    run("value /= 3");
    assert_eq!(run("value"), 18);
    run("value %= 5");
    assert_eq!(run("value"), 3);
    run(r#"text = "A""#);
    run(r#"text += "B""#);
    assert_eq!(run("text"), "AB");
    run("ratio = 2");
    run("ratio *= -3");
    assert_eq!(run("ratio"), -6);
    run("empty = null");
    run("empty += 1");
    assert_eq!(run("empty"), serde_json::Value::Null);
}

/// Nucleoid keeps compound assignments reactive
#[rustfmt::skip]
#[test]
fn keeps_compound_assignments_reactive() {
    let mut run = runner();
    run("amount = 10");
    run("step = 2");
    run("amount += step");
    run("doubled = amount * 2");
    assert_eq!(run("amount"), 12);
    assert_eq!(run("doubled"), 24);
    run("step = 3");
    assert_eq!(run("amount"), 15);
    assert_eq!(run("doubled"), 30);
    run("amount += step.value");
    assert_eq!(run("amount"), 18);
    run("step = 5");
    assert_eq!(run("amount"), 18);
    assert_eq!(run("doubled"), 36);
}

/// Nucleoid keeps compound property and local assignments equivalent
#[rustfmt::skip]
#[test]
fn keeps_compound_property_and_local_assignments_equivalent() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Meter(reading: int):
    this.reading = reading
    def adjust(delta: int):
        delta *= 2
        this.reading += delta
        return this.reading"#);
    run("meter1 = Meter(10)");
    run("meter1.reading += 2");
    assert_eq!(run("meter1.reading"), 12);
    assert_eq!(run("meter1.adjust(3)"), 18);
    assert_eq!(run("meter1.reading"), 18);
    run("class Scale");
    run("scale1 = Scale()");
    run("$Scale.factor = 2");
    assert_eq!(run_error("$Scale.factor *= 3"), "TypeError: Circular Dependency");
    assert_eq!(run("scale1.factor"), 2);
}

/// Nucleoid rolls back failed compound assignments
#[rustfmt::skip]
#[test]
fn rolls_back_failed_compound_assignments() {
    let (mut run, mut run_error) = crate::common::runners();
    run("balance = 10");
    run("step = 2");
    run(r#"if balance > 15:
    throw "LIMIT""#);
    assert_eq!(run_error("balance += step * 4"), "LIMIT");
    assert_eq!(run("balance"), 10);
    run("step = 3");
    assert_eq!(run("balance"), 10);
    run("balance += step");
    assert_eq!(run("balance"), 13);
    run("first = 2");
    run("second = first * 2");
    assert_eq!(run_error("first += second"), "TypeError: Circular Dependency");
    assert_eq!(run("first"), 2);
    assert_eq!(run("second"), 4);
    assert_eq!(run_error("missing += 1"), "ReferenceError: missing is not defined");
}

/// Nucleoid returns compound assignment expressions
#[rustfmt::skip]
#[test]
fn returns_compound_assignment_expressions() {
    let mut run = runner();
    run("def add current delta => current += delta");
    run("def subtract current delta => current -= delta");
    run("def multiply current delta => current *= delta");
    run("def divide current delta => current /= delta");
    run("def remainder current delta => current %= delta");
    run("def combined a b => a += b *= 2");
    assert_eq!(run("add(5, 3)"), 8);
    assert_eq!(run("subtract(5, 3)"), 2);
    assert_eq!(run("multiply(5, 3)"), 15);
    assert_eq!(run("divide(6, 3)"), 2);
    assert_eq!(run("remainder(5, 3)"), 2);
    assert_eq!(run("combined(1, 3)"), 7);
}

/// Nucleoid keeps bodyless class declarations equivalent
#[rustfmt::skip]
#[test]
fn keeps_bodyless_class_declarations_equivalent() {
    let mut run = runner();
    run("class Entity");
    {
        let actual = run("typeof $Entity");
        let expected = run("(Class)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("typeof Entity");
        let expected = run("(List)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("Class.length"), 1);
    assert_eq!(run("Entity.length"), 0);
    run("entity1 = Entity()");
    {
        let actual = run("entity1");
        let expected = run(r#"({id: "entity1"})"#);
        assert_eq!(actual, expected);
    }
    assert_eq!(run("Entity.length"), 1);
}

/// Nucleoid keeps rules reactive on bodyless classes
#[rustfmt::skip]
#[test]
fn keeps_rules_reactive_on_bodyless_classes() {
    let mut run = runner();
    run("class Item");
    run("rate = 2");
    run("$Item.price = $Item.quantity * rate");
    run("item1 = Item()");
    assert_eq!(run("item1.price"), serde_json::Value::Null);
    run("item1.quantity = 3");
    assert_eq!(run("item1.price"), 6);
    run("item2 = Item()");
    run("item2.quantity = 4");
    assert_eq!(run("item2.price"), 8);
    run("rate = 3");
    assert_eq!(run("item1.price"), 9);
    assert_eq!(run("item2.price"), 12);
    run("item1.quantity = 5");
    assert_eq!(run("item1.price"), 15);
    assert_eq!(run("item2.price"), 12);
    assert_eq!(run("Item.length"), 2);
}

/// Nucleoid preserves parameters and scope in bodyless classes
#[rustfmt::skip]
#[test]
fn preserves_parameters_and_scope_in_bodyless_classes() {
    let mut run = runner();
    run("class Shell(code: int)");
    run("first = Shell(5)");
    run("second = Shell()");
    {
        let actual = run("first");
        let expected = run(r#"({id: "first"})"#);
        assert_eq!(actual, expected);
    }
    assert_eq!(run("first.code"), serde_json::Value::Null);
    assert_eq!(run("second.code"), serde_json::Value::Null);
    assert_eq!(run("Shell.length"), 2);
    run(r#"{
    class Token
    token1 = Token()
    assert(Token[0], token1)
}"#);
    assert_eq!(run("Token.length"), 1);
    assert_eq!(run("Class.length"), 2);
}

/// Nucleoid preserves bodyless class populations and inheritance
#[rustfmt::skip]
#[test]
fn preserves_bodyless_class_populations_and_inheritance() {
    let mut run = runner();
    run("class Parent");
    run("first = Parent()");
    assert_eq!(run("Parent.length"), 1);
    run("class Parent");
    assert_eq!(run("Class.length"), 1);
    assert_eq!(run("Parent.length"), 1);
    run(r#"$Parent.label = "BASE""#);
    run(r#"class Child: Parent
    pass"#);
    run("child1 = Child()");
    assert_eq!(run("Class.length"), 2);
    assert_eq!(run("Parent.length"), 1);
    assert_eq!(run("Child.length"), 1);
    assert_eq!(run("first.label"), "BASE");
    assert_eq!(run("child1.label"), "BASE");
}

/// Nucleoid rolls back creation of bodyless class instances
#[rustfmt::skip]
#[test]
fn rolls_back_creation_of_bodyless_class_instances() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Limited");
    run("count = Limited.length");
    run(r#"if count > 1:
    throw "LIMIT""#);
    run("limited1 = Limited()");
    assert_eq!(run_error("limited2 = Limited()"), "LIMIT");
    assert_eq!(run("Limited.length"), 1);
    assert_eq!(run("count"), 1);
    {
        let actual = run("limited1");
        let expected = run(r#"({id: "limited1"})"#);
        assert_eq!(actual, expected);
    }
    assert_eq!(run_error("limited2"), "ReferenceError: limited2 is not defined");
}

/// Nucleoid invokes a parent constructor with bare super on the same instance
#[rustfmt::skip]
#[test]
fn invokes_a_parent_constructor_with_bare_super_on_the_same_instance() {
    let mut run = runner();
    run(r#"class Parent:
    def init():
        this.amount = 4"#);
    run(r#"class Child: Parent
    def init():
        super
        this.amount = this.amount + 1"#);
    run("child = Child()");
    assert_eq!(run("child.amount"), 5);
    assert_eq!(run("child.id"), "child");
    assert_eq!(run("Parent.length"), 0);
    assert_eq!(run("Child.length"), 1);
}

/// Nucleoid resolves bare super through the executing constructor chain
#[rustfmt::skip]
#[test]
fn resolves_bare_super_through_the_executing_constructor_chain() {
    let mut run = runner();
    run(r#"class Base:
    def init():
        this.trace = "A""#);
    run(r#"class Middle: Base
    def init():
        super
        this.trace = this.trace + "B""#);
    run(r#"class Leaf: Middle
    def init():
        super
        this.trace = this.trace + "C""#);
    run(r#"class Passive: Leaf
    pass"#);
    run("leaf = Leaf()");
    run("passive = Passive()");
    assert_eq!(run("leaf.trace"), "ABC");
    assert_eq!(run("passive.trace"), "ABC");
    assert_eq!(run("Base.length"), 0);
    assert_eq!(run("Middle.length"), 0);
    assert_eq!(run("Leaf.length"), 1);
    assert_eq!(run("Passive.length"), 1);
}

/// Nucleoid preserves omitted parent arguments with bare super
#[rustfmt::skip]
#[test]
fn preserves_omitted_parent_arguments_with_bare_super() {
    let mut run = runner();
    run(r#"class Parent:
    def init amount:
        this.amount = amount"#);
    run(r#"class Child: Parent
    def init():
        super"#);
    run(r#"class Explicit: Parent
    def init amount:
        super(amount)"#);
    run("child = Child()");
    run("explicit = Explicit(9)");
    assert_eq!(run("child.amount"), serde_json::Value::Null);
    assert_eq!(run("explicit.amount"), 9);
}

/// Nucleoid preserves bare super expression and receiver behavior
#[rustfmt::skip]
#[test]
fn preserves_bare_super_expression_and_receiver_behavior() {
    let (mut run, mut run_error) = crate::common::runners();
    run(r#"class Parent:
    def init():
        this.amount = 3"#);
    run(r#"class Child: Parent
    def init() => super
    def reset() => super"#);
    run("child = Child()");
    assert_eq!(run("child.amount"), 3);
    run("child.amount = 9");
    assert_eq!(run("child.reset()"), serde_json::Value::Null);
    assert_eq!(run("child.amount"), 3);
    run(r#"class Root:
    def init():
        this.result = super"#);
    run("root = Root()");
    assert_eq!(run("root.result"), serde_json::Value::Null);
    assert_eq!(run_error("super"), "ReferenceError: super is not defined");
}

/// Nucleoid rolls back a failing parent constructor called with bare super
#[rustfmt::skip]
#[test]
fn rolls_back_a_failing_parent_constructor_called_with_bare_super() {
    let (mut run, mut run_error) = crate::common::runners();
    run("kept = { amount: 5 }");
    run(r#"class Parent:
    def init():
        kept.amount = 9
        this.amount = 7
        throw "BLOCKED""#);
    run(r#"class Child: Parent
    def init():
        super"#);
    assert_eq!(run_error("failed = Child()"), "BLOCKED");
    assert_eq!(run("kept.amount"), 5);
    assert_eq!(run("Parent.length"), 0);
    assert_eq!(run("Child.length"), 0);
    assert_eq!(run_error("failed"), "ReferenceError: failed is not defined");
}

/// Nucleoid keeps shorthand object fields reactive
#[rustfmt::skip]
#[test]
fn keeps_shorthand_object_fields_reactive() {
    let mut run = runner();
    run("price = 4");
    run("quantity = 3");
    run("order = { price, quantity }");
    run("total = order.price * order.quantity");
    assert_eq!(run("order.price"), 4);
    assert_eq!(run("order.quantity"), 3);
    assert_eq!(run("total"), 12);
    run("price = 5");
    assert_eq!(run("order.price"), 5);
    assert_eq!(run("total"), 15);
    run("quantity = 2");
    assert_eq!(run("order.quantity"), 2);
    assert_eq!(run("total"), 10);
}

/// Nucleoid resolves shorthand object fields in their lexical scope
#[rustfmt::skip]
#[test]
fn resolves_shorthand_object_fields_in_their_lexical_scope() {
    let mut run = runner();
    run(r#"label = "outer""#);
    run(r#"def pack label =>
    { label }"#);
    run(r#"local = pack("inner")"#);
    assert_eq!(run("local.label"), "inner");
    run(r#"label = "changed""#);
    assert_eq!(run("local.label"), "inner");
    run("builder = amount => ({ amount })");
    run("built = builder(7)");
    assert_eq!(run("built.amount"), 7);
    run("def wrap amount => { amount }");
    run("wrapped = wrap(5)");
    assert_eq!(run("wrapped.amount"), 5);
    run(r#"class Parcel:
    def init amount:
        this.payload = { amount }"#);
    run("parcel = Parcel(8)");
    assert_eq!(run("parcel.payload.amount"), 8);
}

/// Nucleoid preserves snapshots and undefined reads in shorthand objects
#[rustfmt::skip]
#[test]
fn preserves_snapshots_and_undefined_reads_in_shorthand_objects() {
    let (mut run, mut run_error) = crate::common::runners();
    run("amount = 2");
    run("live = { amount }");
    run("frozen = { amount: amount.value }");
    assert_eq!(run("live.amount"), 2);
    assert_eq!(run("frozen.amount"), 2);
    run("amount = 5");
    assert_eq!(run("live.amount"), 5);
    assert_eq!(run("frozen.amount"), 2);
    assert_eq!(run_error("pending = { missing }"), "ReferenceError: missing is not defined");
    run("missing = 8");
    run("pending = { missing }");
    assert_eq!(run("pending.missing"), 8);
}

/// Nucleoid preserves mixed nested and duplicate shorthand object fields
#[rustfmt::skip]
#[test]
fn preserves_mixed_nested_and_duplicate_shorthand_object_fields() {
    let mut run = runner();
    run("amount = 3");
    run(r#"label = "A""#);
    run("other = 7");
    run(r#"mixed = ({
    amount,
    label,
    "quoted-key": label,
    1: amount,
    amount: other,
})"#);
    assert_eq!(run("mixed.amount"), 7);
    assert_eq!(run("mixed.label"), "A");
    assert_eq!(run(r#"mixed["quoted-key"]"#), "A");
    assert_eq!(run("mixed[1]"), 3);
    run("amount = 5");
    assert_eq!(run("mixed.amount"), 7);
    assert_eq!(run("mixed[1]"), 5);
    run("other = 9");
    assert_eq!(run("mixed.amount"), 9);
    run("nested = { mixed, items: [{ amount }] }");
    assert_eq!(run("nested.mixed.amount"), 9);
    assert_eq!(run("nested.items[0].amount"), 5);
}

/// Nucleoid restores shorthand object dependencies after a failed transaction
#[rustfmt::skip]
#[test]
fn restores_shorthand_object_dependencies_after_a_failed_transaction() {
    let (mut run, mut run_error) = crate::common::runners();
    run("amount = 3");
    run("alternate = 7");
    run("record = { amount }");
    run("derived = record.amount * 2");
    assert_eq!(run_error(r#"record = { alternate }
assert(record.alternate, 7)
throw "ABORT""#), "ABORT");
    assert_eq!(run("record.amount"), 3);
    assert_eq!(run("derived"), 6);
    run("alternate = 9");
    assert_eq!(run("record.amount"), 3);
    run("amount = 5");
    assert_eq!(run("record.amount"), 5);
    assert_eq!(run("derived"), 10);
}

/// Nucleoid keeps bare single parameter named arrows reactive
#[rustfmt::skip]
#[test]
fn keeps_bare_single_parameter_named_arrows_reactive() {
    let mut run = runner();
    run("base = 2");
    run("def scaled amount => amount + base");
    run("result = scaled(3)");
    assert_eq!(run("result"), 5);
    run("base = 4");
    assert_eq!(run("result"), 7);
    run("def scaled amount => amount * base");
    assert_eq!(run("result"), 12);
}

/// Nucleoid preserves callable references for bare single parameter named arrows
#[rustfmt::skip]
#[test]
fn preserves_callable_references_for_bare_single_parameter_named_arrows() {
    let mut run = runner();
    run("def change amount => amount + 1");
    run("live = change");
    run("snapshot = change.value");
    assert_eq!(run("live(2)"), 3);
    assert_eq!(run("snapshot(2)"), 3);
    run("def change amount => amount * 3");
    assert_eq!(run("live(2)"), 6);
    assert_eq!(run("snapshot(2)"), 3);
}

/// Nucleoid preserves bare single parameter methods and initializers
#[rustfmt::skip]
#[test]
fn preserves_bare_single_parameter_methods_and_initializers() {
    let mut run = runner();
    run(r#"class Gauge:
    def init amount => this.amount = amount
    def reading offset => self.amount + offset"#);
    run("gauge = Gauge(4)");
    assert_eq!(run("gauge.amount"), 4);
    assert_eq!(run("gauge.reading(3)"), 7);
    run("gauge.amount = 6");
    assert_eq!(run("gauge.reading(3)"), 9);
}

/// Nucleoid keeps bare single parameter binding and explicit alternatives equivalent
#[rustfmt::skip]
#[test]
fn keeps_bare_single_parameter_binding_and_explicit_alternatives_equivalent() {
    let mut run = runner();
    run("amount = 10");
    run("def identity amount => amount");
    assert_eq!(run("identity(3)"), 3);
    assert_eq!(run("identity()"), serde_json::Value::Null);
    assert_eq!(run("identity(3, 4)"), 3);
    assert_eq!(run("amount"), 10);
    run("def typed(amount: int) => amount + 1");
    run(r#"def block(amount) =>
    amount + 2"#);
    assert_eq!(run("typed(4)"), 5);
    assert_eq!(run("block(4)"), 6);
}

/// Nucleoid rolls back failed bare single parameter arrow redefinitions
#[rustfmt::skip]
#[test]
fn rolls_back_failed_bare_single_parameter_arrow_redefinitions() {
    let (mut run, mut run_error) = crate::common::runners();
    run("source = 3");
    run("def reading amount => source + amount");
    run("total = reading(2)");
    assert_eq!(run("total"), 5);
    assert_eq!(run_error("def reading amount => missing + amount"), "ReferenceError: missing is not defined");
    assert_eq!(run("total"), 5);
    run("source = 4");
    assert_eq!(run("total"), 6);
}

/// Nucleoid keeps bare single parameter block definitions reactive
#[rustfmt::skip]
#[test]
fn keeps_bare_single_parameter_block_definitions_reactive() {
    let mut run = runner();
    run("base = 2");
    run(r#"def scaled amount =>
    amount + base"#);
    run("result = scaled(3)");
    assert_eq!(run("result"), 5);
    run("base = 4");
    assert_eq!(run("result"), 7);
    run(r#"def scaled amount =>
    amount * base"#);
    assert_eq!(run("result"), 12);
}

/// Nucleoid preserves callable references for bare single parameter block definitions
#[rustfmt::skip]
#[test]
fn preserves_callable_references_for_bare_single_parameter_block_definitions() {
    let mut run = runner();
    run(r#"def change amount =>
    amount + 1"#);
    run("live = change");
    run("snapshot = change.value");
    assert_eq!(run("live(2)"), 3);
    assert_eq!(run("snapshot(2)"), 3);
    run(r#"def change amount =>
    amount * 3"#);
    assert_eq!(run("live(2)"), 6);
    assert_eq!(run("snapshot(2)"), 3);
}

/// Nucleoid preserves bare single parameter block methods and initializers
#[rustfmt::skip]
#[test]
fn preserves_bare_single_parameter_block_methods_and_initializers() {
    let mut run = runner();
    run(r#"class Gauge:
    def init amount:
        this.amount = amount
    def reading offset =>
        self.amount + offset"#);
    run("gauge = Gauge(4)");
    assert_eq!(run("gauge.amount"), 4);
    assert_eq!(run("gauge.reading(3)"), 7);
    run("gauge.amount = 6");
    assert_eq!(run("gauge.reading(3)"), 9);
}

/// Nucleoid keeps bare single parameter block binding and explicit alternatives equivalent
#[rustfmt::skip]
#[test]
fn keeps_bare_single_parameter_block_binding_and_explicit_alternatives_equivalent() {
    let mut run = runner();
    run("amount = 10");
    run(r#"def identity amount =>
    amount"#);
    run(r#"def ignored amount:
    amount + 100"#);
    run(r#"def typed(amount: int) =>
    amount + 1"#);
    run(r#"def pair left right =>
    left + right"#);
    run(r#"def unit() =>
    42"#);
    run("def arrow amount => amount + 3");
    assert_eq!(run("identity(3)"), 3);
    assert_eq!(run("identity()"), serde_json::Value::Null);
    assert_eq!(run("identity(3, 4)"), 3);
    assert_eq!(run("amount"), 10);
    assert_eq!(run("ignored(5)"), serde_json::Value::Null);
    assert_eq!(run("typed(4)"), 5);
    assert_eq!(run("pair(2, 3)"), 5);
    assert_eq!(run("unit()"), 42);
    assert_eq!(run("arrow(4)"), 7);
    assert_eq!(run("amount"), 10);
}

/// Nucleoid rolls back failed bare single parameter block redefinitions
#[rustfmt::skip]
#[test]
fn rolls_back_failed_bare_single_parameter_block_redefinitions() {
    let (mut run, mut run_error) = crate::common::runners();
    run("source = 3");
    run(r#"def reading amount =>
    source + amount"#);
    run("total = reading(2)");
    assert_eq!(run("total"), 5);
    assert_eq!(run_error(r#"def reading amount =>
    missing + amount"#), "ReferenceError: missing is not defined");
    assert_eq!(run("total"), 5);
    run("source = 4");
    assert_eq!(run("total"), 6);
}

/// Nucleoid keeps bare typed single parameter named arrows reactive
#[rustfmt::skip]
#[test]
fn keeps_bare_typed_single_parameter_named_arrows_reactive() {
    let mut run = runner();
    run("base = 2");
    run("def scaled amount: int => amount + base");
    run("result = scaled(3)");
    assert_eq!(run("result"), 5);
    run("base = 4");
    assert_eq!(run("result"), 7);
    run("def scaled amount: int => amount * base");
    assert_eq!(run("result"), 12);
}

/// Nucleoid preserves callable references for bare typed single parameter named arrows
#[rustfmt::skip]
#[test]
fn preserves_callable_references_for_bare_typed_single_parameter_named_arrows() {
    let mut run = runner();
    run("def change amount: int => amount + 1");
    run("live = change");
    run("snapshot = change.value");
    run("first = live(2)");
    run("old = snapshot(2)");
    assert_eq!(run("first"), 3);
    assert_eq!(run("old"), 3);
    run("def change amount: int => amount * 2");
    assert_eq!(run("first"), 4);
    assert_eq!(run("old"), 3);
}

/// Nucleoid preserves bare typed single parameter methods and initializers
#[rustfmt::skip]
#[test]
fn preserves_bare_typed_single_parameter_methods_and_initializers() {
    let mut run = runner();
    run(r#"class Gauge:
    def init amount: int => this.amount = amount
    def reading offset: int => self.amount + offset"#);
    run("gauge = Gauge(4)");
    assert_eq!(run("gauge.amount"), 4);
    assert_eq!(run("gauge.reading(3)"), 7);
    run("gauge.amount = 6");
    assert_eq!(run("gauge.reading(3)"), 9);
}

/// Nucleoid keeps bare typed single parameter binding and explicit alternatives equivalent
#[rustfmt::skip]
#[test]
fn keeps_bare_typed_single_parameter_binding_and_explicit_alternatives_equivalent() {
    let mut run = runner();
    run("amount = 10");
    run("def identity amount: ArbitraryType => amount");
    run("def text label: str => label");
    run("def bare value => value + 1");
    run(r#"def block(amount: int) =>
    amount + 2"#);
    run("def pair left: int, right: int => left + right");
    run("def unit() => 42");
    assert_eq!(run("identity(3)"), 3);
    assert_eq!(run("identity()"), serde_json::Value::Null);
    assert_eq!(run("identity(3, 4)"), 3);
    assert_eq!(run(r#"identity("text")"#), "text");
    assert_eq!(run(r#"text("hello")"#), "hello");
    assert_eq!(run("amount"), 10);
    assert_eq!(run("bare(4)"), 5);
    assert_eq!(run("block(4)"), 6);
    assert_eq!(run("pair(2, 3)"), 5);
    assert_eq!(run("unit()"), 42);
}

/// Nucleoid rolls back failed bare typed single parameter arrow redefinitions
#[rustfmt::skip]
#[test]
fn rolls_back_failed_bare_typed_single_parameter_arrow_redefinitions() {
    let (mut run, mut run_error) = crate::common::runners();
    run("source = 3");
    run("def reading amount: int => source + amount");
    run("total = reading(2)");
    assert_eq!(run_error("def reading amount: int => missing + amount"), "ReferenceError: missing is not defined");
    assert_eq!(run("total"), 5);
    run("source = 4");
    assert_eq!(run("total"), 6);
    assert_eq!(run("reading(3)"), 7);
}

/// Nucleoid keeps numeric clamping reactive to all three inputs
#[rustfmt::skip]
#[test]
fn keeps_numeric_clamping_reactive_to_all_three_inputs() {
    let mut run = runner();
    run("value = 12");
    run("lower = 0");
    run("upper = 10");
    run("clipped = Math.clamp(value, lower, upper)");
    assert_eq!(run("clipped"), 10);
    run("value = 5");
    assert_eq!(run("clipped"), 5);
    run("lower = 7");
    assert_eq!(run("clipped"), 7);
    run("upper = 6");
    assert_eq!(run("clipped"), 6);
    run("lower = 0");
    run("value = -3");
    assert_eq!(run("clipped"), 0);
    run("upper = 20");
    run("value = 17");
    assert_eq!(run("clipped"), 17);
}

/// Nucleoid preserves numeric coercion while clamping values
#[rustfmt::skip]
#[test]
fn preserves_numeric_coercion_while_clamping_values() {
    let mut run = runner();
    run(r#"numeric = Math.clamp("7", 0, 5)"#);
    run("boolean = Math.clamp(true, 0, 2)");
    run("nil = Math.clamp(null, -1, 1)");
    run("empty = Math.clamp([], 1, 3)");
    run(r#"singleton = Math.clamp(["4"], 0, 3)"#);
    run(r#"invalid = Math.clamp("invalid", 0, 5)"#);
    assert_eq!(run("numeric"), 5);
    assert_eq!(run("boolean"), 1);
    assert_eq!(run("nil"), 0);
    assert_eq!(run("empty"), 1);
    assert_eq!(run("singleton"), 3);
    assert_eq!(run("String(invalid)"), "NaN");
}

/// Nucleoid preserves nonfinite equal reversed and omitted clamp bounds
#[rustfmt::skip]
#[test]
fn preserves_nonfinite_equal_reversed_and_omitted_clamp_bounds() {
    let mut run = runner();
    run("positive = Math.clamp(Number.POSITIVE_INFINITY, -10, 10)");
    run("negative = Math.clamp(Number.NEGATIVE_INFINITY, -10, 10)");
    run("unbounded = Math.clamp(Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY)");
    run("equal = Math.clamp(3, 4, 4)");
    run("reversed = Math.clamp(5, 10, 0)");
    run("nan_input = Math.clamp(Number.NaN, 0, 10)");
    run("nan_lower = Math.clamp(2, Number.NaN, 10)");
    run("nan_upper = Math.clamp(2, 0, Number.NaN)");
    run("missing_all = Math.clamp()");
    run("missing_lower = Math.clamp(2)");
    run("missing_upper = Math.clamp(2, 0)");
    run("extra = Math.clamp(3, 0, 5, 99)");
    assert_eq!(run("positive"), 10);
    assert_eq!(run("negative"), -10);
    {
        let actual = run("unbounded");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("equal"), 4);
    assert_eq!(run("reversed"), 0);
    assert_eq!(run("String(nan_input)"), "NaN");
    assert_eq!(run("String(nan_lower)"), "NaN");
    assert_eq!(run("String(nan_upper)"), "NaN");
    assert_eq!(run("String(missing_all)"), "NaN");
    assert_eq!(run("String(missing_lower)"), "NaN");
    assert_eq!(run("String(missing_upper)"), "NaN");
    assert_eq!(run("extra"), 3);
}

/// Nucleoid applies numeric clamping to live class rules
#[rustfmt::skip]
#[test]
fn applies_numeric_clamping_to_live_class_rules() {
    let mut run = runner();
    run(r#"class Reading(amount):
    this.amount = amount"#);
    run("$Reading.bounded = Math.clamp($Reading.amount, 0, 10)");
    run("first = Reading(12)");
    run("second = Reading(-3)");
    assert_eq!(run("first.bounded"), 10);
    assert_eq!(run("second.bounded"), 0);
    run("first.amount = 4");
    assert_eq!(run("first.bounded"), 4);
    run("second.amount = 8");
    assert_eq!(run("second.bounded"), 8);
}

/// Nucleoid preserves deferred reads and rollback for numeric clamping
#[rustfmt::skip]
#[test]
fn preserves_deferred_reads_and_rollback_for_numeric_clamping() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Sensor");
    run("sensor = Sensor()");
    run("observed = Math.clamp(sensor.amount, 0, 10)");
    assert_eq!(run("observed"), serde_json::Value::Null);
    run("sensor.amount = 12");
    assert_eq!(run("observed"), 10);
    run("delete sensor.amount");
    assert_eq!(run("observed"), serde_json::Value::Null);
    run("sensor.amount = 3");
    assert_eq!(run("observed"), 3);
    assert_eq!(run_error("observed = Math.clamp(missing, 0, 10)"), "ReferenceError: missing is not defined");
    assert_eq!(run("observed"), 3);
    run("sensor.amount = 6");
    assert_eq!(run("observed"), 6);
}

/// Nucleoid keeps numeric interpolation reactive to all three inputs
#[rustfmt::skip]
#[test]
fn keeps_numeric_interpolation_reactive_to_all_three_inputs() {
    let mut run = runner();
    run("start = 0");
    run("end = 10");
    run("weight = 0.25");
    run("position = Math.lerp(start, end, weight)");
    assert_eq!(run("position"), 2.5);
    run("start = 2");
    assert_eq!(run("position"), 4);
    run("end = 18");
    assert_eq!(run("position"), 6);
    run("weight = 0.5");
    assert_eq!(run("position"), 10);
    run("weight = -0.5");
    assert_eq!(run("position"), -6);
    run("weight = 1.5");
    assert_eq!(run("position"), 26);
}

/// Nucleoid preserves numeric coercion while interpolating values
#[rustfmt::skip]
#[test]
fn preserves_numeric_coercion_while_interpolating_values() {
    let mut run = runner();
    run(r#"numeric = Math.lerp(" 2 ", "10", "0.25")"#);
    run("boolean = Math.lerp(false, true, true)");
    run("nil = Math.lerp(null, 4, 0.5)");
    run("empty = Math.lerp([], 4, 0.5)");
    run(r#"singleton = Math.lerp(["2"], ["10"], ["0.25"])"#);
    run("date = Math.lerp(Date(0), Date(10), 0.5)");
    run(r#"invalid = Math.lerp("invalid", 10, 0.5)"#);
    assert_eq!(run("numeric"), 4);
    assert_eq!(run("boolean"), 1);
    assert_eq!(run("nil"), 2);
    assert_eq!(run("empty"), 2);
    assert_eq!(run("singleton"), 4);
    assert_eq!(run("date"), 5);
    assert_eq!(run("String(invalid)"), "NaN");
}

/// Nucleoid preserves ordered special values and omitted interpolation inputs
#[rustfmt::skip]
#[test]
fn preserves_ordered_special_values_and_omitted_interpolation_inputs() {
    let mut run = runner();
    run("left = Math.lerp(2, 10, 0)");
    run("right = Math.lerp(2, 10, 1)");
    run("equal = Math.lerp(4, 4, 2)");
    run("reversed = Math.lerp(10, 2, 0.25)");
    run("extrapolated = Math.lerp(2, 10, 2)");
    run("positive = Math.lerp(0, Number.POSITIVE_INFINITY, 0.5)");
    run("negative = Math.lerp(0, Number.NEGATIVE_INFINITY, 0.5)");
    run("zero_infinite = Math.lerp(0, Number.POSITIVE_INFINITY, 0)");
    run("equal_infinite = Math.lerp(Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY, 0.5)");
    run("nan_start = Math.lerp(Number.NaN, 10, 0.5)");
    run("nan_end = Math.lerp(2, Number.NaN, 0.5)");
    run("nan_weight = Math.lerp(2, 10, Number.NaN)");
    run("missing_all = Math.lerp()");
    run("missing_end = Math.lerp(2)");
    run("missing_weight = Math.lerp(2, 10)");
    run("extra = Math.lerp(2, 10, 0.5, 99)");
    assert_eq!(run("left"), 2);
    assert_eq!(run("right"), 10);
    assert_eq!(run("equal"), 4);
    assert_eq!(run("reversed"), 8);
    assert_eq!(run("extrapolated"), 18);
    {
        let actual = run("positive");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("negative");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("String(zero_infinite)"), "NaN");
    assert_eq!(run("String(equal_infinite)"), "NaN");
    assert_eq!(run("String(nan_start)"), "NaN");
    assert_eq!(run("String(nan_end)"), "NaN");
    assert_eq!(run("String(nan_weight)"), "NaN");
    assert_eq!(run("String(missing_all)"), "NaN");
    assert_eq!(run("String(missing_end)"), "NaN");
    assert_eq!(run("String(missing_weight)"), "NaN");
    assert_eq!(run("extra"), 6);
}

/// Nucleoid applies numeric interpolation to live class rules
#[rustfmt::skip]
#[test]
fn applies_numeric_interpolation_to_live_class_rules() {
    let mut run = runner();
    run(r#"class Blend(start, end, weight):
    this.start = start
    this.end = end
    this.weight = weight"#);
    run("$Blend.position = Math.lerp($Blend.start, $Blend.end, $Blend.weight)");
    run("first = Blend(0, 10, 0.25)");
    run("second = Blend(10, 0, 0.5)");
    assert_eq!(run("first.position"), 2.5);
    assert_eq!(run("second.position"), 5);
    run("first.weight = 0.75");
    assert_eq!(run("first.position"), 7.5);
    run("second.end = 20");
    assert_eq!(run("second.position"), 15);
}

/// Nucleoid preserves deferred reads and rollback for numeric interpolation
#[rustfmt::skip]
#[test]
fn preserves_deferred_reads_and_rollback_for_numeric_interpolation() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Sensor");
    run("sensor = Sensor()");
    run("end = 10");
    run("weight = 0.5");
    run("position = Math.lerp(sensor.start, end, weight)");
    assert_eq!(run("position"), serde_json::Value::Null);
    run("sensor.start = 2");
    assert_eq!(run("position"), 6);
    run("delete sensor.start");
    assert_eq!(run("position"), serde_json::Value::Null);
    run("sensor.start = 4");
    assert_eq!(run("position"), 7);
    assert_eq!(run_error("position = Math.lerp(missing, end, weight)"), "ReferenceError: missing is not defined");
    assert_eq!(run("position"), 7);
    run("end = 20");
    assert_eq!(run("position"), 12);
}

/// Nucleoid keeps unit-interval saturation reactive to its input
#[rustfmt::skip]
#[test]
fn keeps_unit_interval_saturation_reactive_to_its_input() {
    let mut run = runner();
    run("reading = -2");
    run("bounded = Math.saturate(reading)");
    assert_eq!(run("bounded"), 0);
    run("reading = 0.25");
    assert_eq!(run("bounded"), 0.25);
    run("reading = 3");
    assert_eq!(run("bounded"), 1);
    run("reading = 0.5");
    assert_eq!(run("bounded"), 0.5);
    run("reading = 1");
    assert_eq!(run("bounded"), 1);
    run("reading = -1");
    assert_eq!(run("bounded"), 0);
}

/// Nucleoid preserves numeric coercion while saturating values
#[rustfmt::skip]
#[test]
fn preserves_numeric_coercion_while_saturating_values() {
    let mut run = runner();
    run(r#"text = " 0.375 ""#);
    run("from_text = Math.saturate(text)");
    assert_eq!(run("from_text"), 0.375);
    run(r#"text = "-3""#);
    assert_eq!(run("from_text"), 0);
    run(r#"text = "2""#);
    assert_eq!(run("from_text"), 1);
    run(r#"empty = Math.saturate("")"#);
    run("from_false = Math.saturate(false)");
    run("from_true = Math.saturate(true)");
    run("from_empty_list = Math.saturate([])");
    run("from_list = Math.saturate([0.625])");
    run("bad_list = Math.saturate([0, 1])");
    run(r#"bad_text = Math.saturate("not a number")"#);
    run("literal_null = Math.saturate(null)");
    run("from_date = Math.saturate(Date(1))");
    assert_eq!(run("empty"), 0);
    assert_eq!(run("from_false"), 0);
    assert_eq!(run("from_true"), 1);
    assert_eq!(run("from_empty_list"), 0);
    assert_eq!(run("from_list"), 0.625);
    assert_eq!(run("Number.isNaN(bad_list)"), true);
    assert_eq!(run("Number.isNaN(bad_text)"), true);
    assert_eq!(run("literal_null"), 0);
    assert_eq!(run("from_date"), 1);
}

/// Nucleoid preserves special values and omitted unit saturation inputs
#[rustfmt::skip]
#[test]
fn preserves_special_values_and_omitted_unit_saturation_inputs() {
    let mut run = runner();
    run("positive = Math.saturate(1 / 0)");
    run("negative = Math.saturate(-1 / 0)");
    run("invalid = Math.saturate(0 / 0)");
    run("positive_zero = Math.saturate(0)");
    run("negative_zero = Math.saturate(-0)");
    run("omitted = Math.saturate()");
    run("extra = Math.saturate(0.5, 99)");
    run("tiny = Math.saturate(Number.MIN_VALUE)");
    assert_eq!(run("positive"), 1);
    assert_eq!(run("negative"), 0);
    assert_eq!(run("Number.isNaN(invalid)"), true);
    {
        let actual = run("1 / positive_zero");
        let expected = run("(1 / Math.min(Math.max(0, 0), 1))");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("1 / negative_zero");
        let expected = run("(1 / Math.min(Math.max(-0, 0), 1))");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("Number.isNaN(omitted)"), true);
    assert_eq!(run("extra"), 0.5);
    {
        let actual = run("tiny");
        let expected = run("(Number.MIN_VALUE)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid applies unit-interval saturation to live class rules
#[rustfmt::skip]
#[test]
fn applies_unit_interval_saturation_to_live_class_rules() {
    let mut run = runner();
    run(r#"class Level(reading):
    this.reading = reading"#);
    run("$Level.bounded = Math.saturate($Level.reading)");
    run("first = Level(-2)");
    run("second = Level(0.4)");
    assert_eq!(run("first.bounded"), 0);
    assert_eq!(run("second.bounded"), 0.4);
    run("first.reading = 0.8");
    assert_eq!(run("first.bounded"), 0.8);
    run("second.reading = 2");
    assert_eq!(run("second.bounded"), 1);
}

/// Nucleoid preserves deferred reads and rollback for unit saturation
#[rustfmt::skip]
#[test]
fn preserves_deferred_reads_and_rollback_for_unit_saturation() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Sensor");
    run("sensor = Sensor()");
    run("bounded = Math.saturate(sensor.reading)");
    assert_eq!(run("bounded"), serde_json::Value::Null);
    run("sensor.reading = -2");
    assert_eq!(run("bounded"), 0);
    run("delete sensor.reading");
    assert_eq!(run("bounded"), serde_json::Value::Null);
    run("sensor.reading = 0.75");
    assert_eq!(run("bounded"), 0.75);
    assert_eq!(run_error("bounded = Math.saturate(missing)"), "ReferenceError: missing is not defined");
    assert_eq!(run("bounded"), 0.75);
    run("sensor.reading = 2");
    assert_eq!(run("bounded"), 1);
}

/// Nucleoid keeps bare typed single parameter block definitions reactive
#[rustfmt::skip]
#[test]
fn keeps_bare_typed_single_parameter_block_definitions_reactive() {
    let mut run = runner();
    run("base = 2");
    run(r#"def scaled amount: int =>
    amount + base"#);
    run("result = scaled(3)");
    assert_eq!(run("result"), 5);
    run("base = 4");
    assert_eq!(run("result"), 7);
    run(r#"def scaled amount: int =>
    amount * base"#);
    assert_eq!(run("result"), 12);
}

/// Nucleoid preserves callable references for bare typed single parameter block definitions
#[rustfmt::skip]
#[test]
fn preserves_callable_references_for_bare_typed_single_parameter_block_definitions() {
    let mut run = runner();
    run("base = 1");
    run(r#"def change amount: ArbitraryType =>
    amount + base"#);
    run("live = change");
    run("snapshot = change.value");
    run("first = live(2)");
    run("old = snapshot(2)");
    assert_eq!(run("first"), 3);
    assert_eq!(run("old"), 3);
    run("base = 3");
    assert_eq!(run("first"), 5);
    assert_eq!(run("old"), 5);
    run(r#"def change amount: int =>
    amount * base"#);
    assert_eq!(run("first"), 6);
    assert_eq!(run("old"), 5);
    run("base = 4");
    assert_eq!(run("first"), 8);
    assert_eq!(run("old"), 6);
}

/// Nucleoid preserves bare typed single parameter block methods and initializers
#[rustfmt::skip]
#[test]
fn preserves_bare_typed_single_parameter_block_methods_and_initializers() {
    let mut run = runner();
    run(r#"class Gauge:
    def init amount: int:
        this.amount = amount
    def reading offset: int =>
        self.amount + offset"#);
    run("gauge = Gauge(4)");
    assert_eq!(run("gauge.amount"), 4);
    assert_eq!(run("gauge.reading(3)"), 7);
    run("gauge.amount = 6");
    assert_eq!(run("gauge.reading(3)"), 9);
}

/// Nucleoid keeps bare typed single parameter block binding and explicit alternatives equivalent
#[rustfmt::skip]
#[test]
fn keeps_bare_typed_single_parameter_block_binding_and_explicit_alternatives_equivalent() {
    let mut run = runner();
    run("amount = 10");
    run(r#"def identity amount: ArbitraryType =>
    amount"#);
    run(r#"def text label: str =>
    label"#);
    run(r#"def ignored amount: int:
    amount + 100"#);
    run(r#"def bare value =>
    value + 1"#);
    run(r#"def typed(amount: int) =>
    amount + 2"#);
    run("def arrow value: int => value + 3");
    run(r#"def pair(left: int, right: int) =>
    left + right"#);
    run(r#"def unit() =>
    42"#);
    assert_eq!(run("identity(3)"), 3);
    assert_eq!(run("identity()"), serde_json::Value::Null);
    assert_eq!(run("identity(3, 4)"), 3);
    assert_eq!(run(r#"identity("text")"#), "text");
    assert_eq!(run(r#"text("hello")"#), "hello");
    assert_eq!(run("ignored(5)"), serde_json::Value::Null);
    assert_eq!(run("bare(4)"), 5);
    assert_eq!(run("typed(4)"), 6);
    assert_eq!(run("arrow(4)"), 7);
    assert_eq!(run("pair(2, 3)"), 5);
    assert_eq!(run("unit()"), 42);
    assert_eq!(run("amount"), 10);
}

/// Nucleoid rolls back failed bare typed single parameter block redefinitions
#[rustfmt::skip]
#[test]
fn rolls_back_failed_bare_typed_single_parameter_block_redefinitions() {
    let (mut run, mut run_error) = crate::common::runners();
    run("source = 3");
    run(r#"def reading amount: int =>
    source + amount"#);
    run("total = reading(2)");
    assert_eq!(run("total"), 5);
    assert_eq!(run_error(r#"def reading amount: int =>
    missing + amount"#), "ReferenceError: missing is not defined");
    assert_eq!(run("total"), 5);
    run("source = 4");
    assert_eq!(run("total"), 6);
    assert_eq!(run("reading(3)"), 7);
}

/// Nucleoid keeps logical compound assignments equivalent
#[rustfmt::skip]
#[test]
fn keeps_logical_compound_assignments_equivalent() {
    let mut run = runner();
    run("value = 0");
    run("value ||= 7");
    assert_eq!(run("value"), 7);
    run("value &&= 9");
    assert_eq!(run("value"), 9);
    run(r#"label = """#);
    run(r#"label ||= "ready""#);
    assert_eq!(run("label"), "ready");
    run(r#"label &&= "done""#);
    assert_eq!(run("label"), "done");
    run("off = false");
    run(r#"off &&= "unused""#);
    assert_eq!(run("off"), false);
    run("on = true");
    run(r#"on ||= "unused""#);
    assert_eq!(run("on"), true);
    run("grouped = false");
    run("grouped ||= false && true");
    assert_eq!(run("grouped"), false);
    run("value &&= 0 || 3");
    assert_eq!(run("value"), 3);
}

/// Nucleoid keeps logical compound assignments reactive
#[rustfmt::skip]
#[test]
fn keeps_logical_compound_assignments_reactive() {
    let mut run = runner();
    run("enabled = false");
    run("fallback = false");
    run("enabled ||= fallback");
    assert_eq!(run("enabled"), false);
    run("fallback = true");
    assert_eq!(run("enabled"), true);
    run("fallback = false");
    assert_eq!(run("enabled"), true);
    run("ready = true");
    run("limit = true");
    run("ready &&= limit");
    assert_eq!(run("ready"), true);
    run("limit = false");
    assert_eq!(run("ready"), false);
    run("limit = true");
    assert_eq!(run("ready"), false);
    run("snapshot = false");
    run("source = true");
    run("snapshot ||= source.value");
    assert_eq!(run("snapshot"), true);
    run("source = false");
    assert_eq!(run("snapshot"), true);
}

/// Nucleoid short circuits logical compound operands
#[rustfmt::skip]
#[test]
fn short_circuits_logical_compound_operands() {
    let mut run = runner();
    run(r#"def fail():
    throw "UNREACHABLE""#);
    run(r#"retained = "ready""#);
    run("retained ||= fail()");
    assert_eq!(run("retained"), "ready");
    run("stopped = false");
    run("stopped &&= fail()");
    assert_eq!(run("stopped"), false);
    run("chosen = false");
    run("chosen ||= 12");
    assert_eq!(run("chosen"), 12);
    run("chosen &&= 4");
    assert_eq!(run("chosen"), 4);
}

/// Nucleoid preserves logical compound property and local assignments
#[rustfmt::skip]
#[test]
fn preserves_logical_compound_property_and_local_assignments() {
    let mut run = runner();
    run(r#"class Gate:
    def allow(fallback):
        this.enabled ||= fallback
        return this.enabled
    def require(allowed):
        self.enabled &&= allowed
        return self.enabled"#);
    run("gate = Gate()");
    run("gate.enabled = false");
    assert_eq!(run("gate.allow(true)"), true);
    assert_eq!(run("gate.require(false)"), false);
    assert_eq!(run("gate.enabled"), false);
    run("def prefer value fallback => value ||= fallback");
    run("def requireBoth value allowed => value &&= allowed");
    assert_eq!(run("prefer(0, 8)"), 8);
    assert_eq!(run("prefer(3, 8)"), 3);
    assert_eq!(run("requireBoth(2, 7)"), 7);
    assert_eq!(run("requireBoth(0, 7)"), 0);
    run("def combined a b => a ||= b &&= 9");
    assert_eq!(run("combined(false, 2)"), 9);
    assert_eq!(run("combined(true, 2)"), true);
    run(r#"combined(false, 2)

# return: 9"#);
}

/// Nucleoid rolls back failed logical compound assignments
#[rustfmt::skip]
#[test]
fn rolls_back_failed_logical_compound_assignments() {
    let mut run = runner();
    run("enabled = false");
    run(r#"if enabled:
    throw "BLOCKED""#);
    run("caught = null");
    run(r#"try:
    enabled ||= true
catch error:
    caught = error"#);
    assert_eq!(run("caught"), "BLOCKED");
    assert_eq!(run("enabled"), false);
    run("ready = true");
    run(r#"if !ready:
    throw "REQUIRED""#);
    run("caught = null");
    run(r#"try:
    ready &&= false
catch error:
    caught = error"#);
    assert_eq!(run("caught"), "REQUIRED");
    assert_eq!(run("ready"), true);
}

/// Nucleoid defines bare multiple parameter named arrows
#[rustfmt::skip]
#[test]
fn defines_bare_multiple_parameter_named_arrows() {
    let mut run = runner();
    run("def add left right => left + right");
    run("def scaled left right factor => (left + right) * factor");
    run("def pair left right => [left, right]");
    run("def coordinates x y => {x, y}");
    assert_eq!(run("add(2, 3)"), 5);
    assert_eq!(run("scaled(2, 3, 4)"), 20);
    assert_eq!(run(r#"pair("left", "right")"#), serde_json::json!(["left", "right"]));
    run("point = coordinates(2, 3)");
    assert_eq!(run("point.x"), 2);
    assert_eq!(run("point.y"), 3);
}

/// Nucleoid keeps bare multiple parameter arrows reactive
#[rustfmt::skip]
#[test]
fn keeps_bare_multiple_parameter_arrows_reactive() {
    let mut run = runner();
    run("bias = 1");
    run("def total left right => left + right + bias");
    run("tracked = total(2, 3)");
    assert_eq!(run("tracked"), 6);
    run("bias = 4");
    assert_eq!(run("tracked"), 9);
    run("def projected first second third => first * second + third");
    run("left = 2");
    run("right = 3");
    run("offset = 4");
    run("result = projected(left, right, offset)");
    assert_eq!(run("result"), 10);
    run("left = 5");
    assert_eq!(run("result"), 19);
    run("right = 2");
    assert_eq!(run("result"), 14);
}

/// Nucleoid preserves bare multiple parameter methods
#[rustfmt::skip]
#[test]
fn preserves_bare_multiple_parameter_methods() {
    let mut run = runner();
    run(r#"class Calculator:
    def combine left right => this.base + left + right
    def product left right scale => self.base + left * right * scale"#);
    run("calculator = Calculator()");
    run("calculator.base = 2");
    run("result = calculator.combine(3, 4)");
    assert_eq!(run("result"), 9);
    run("calculator.base = 5");
    assert_eq!(run("result"), 12);
    assert_eq!(run("calculator.product(2, 3, 4)"), 29);
}

/// Nucleoid preserves nested bare multiple parameter arrows
#[rustfmt::skip]
#[test]
fn preserves_nested_bare_multiple_parameter_arrows() {
    let mut run = runner();
    run(r#"def outer(initial):
    def combine left right scale => (left + right) * scale
    return combine(initial, 2, 3)"#);
    assert_eq!(run("outer(4)"), 18);
    run("def factory(ignored) => (left, right) => left + right");
    run("custom = factory(10)");
    assert_eq!(run("custom(2, 3)"), 5);
}

/// Nucleoid returns bare multiple parameter named arrows
#[rustfmt::skip]
#[test]
fn returns_bare_multiple_parameter_named_arrows() {
    let mut run = runner();
    run("def prefer current fallback => current ||= fallback");
    run("def requireBoth current condition => current &&= condition");
    run("def combined first second scale => (first + second) * scale");
    assert_eq!(run("prefer(0, 9)"), 9);
    assert_eq!(run("prefer(3, 9)"), 3);
    assert_eq!(run("requireBoth(2, 7)"), 7);
    assert_eq!(run("requireBoth(0, 7)"), 0);
    assert_eq!(run("combined(2, 5, 3)"), 21);
    run(r#"combined(2, 5, 3)

# return: 21"#);
}

/// Nucleoid defines bare multiple parameter indented blocks
#[rustfmt::skip]
#[test]
fn defines_bare_multiple_parameter_indented_blocks() {
    let mut run = runner();
    run(r#"def add left, right:
    total = left + right
    return total"#);
    run(r#"def scaled left right factor =>
    (left + right) * factor"#);
    run(r#"def largest left, right:
    if left > right:
        return left
    return right"#);
    assert_eq!(run("add(2, 3)"), 5);
    assert_eq!(run("scaled(2, 3, 4)"), 20);
    assert_eq!(run("largest(7, 3)"), 7);
    assert_eq!(run("largest(2, 9)"), 9);
}

/// Nucleoid keeps bare multiple parameter blocks reactive
#[rustfmt::skip]
#[test]
fn keeps_bare_multiple_parameter_blocks_reactive() {
    let mut run = runner();
    run("bias = 2");
    run(r#"def scaled left right =>
    left * right + bias"#);
    run("live = scaled");
    run("snapshot = scaled.value");
    run("result = live(3, 4)");
    run("frozen = snapshot(3, 4)");
    assert_eq!(run("result"), 14);
    assert_eq!(run("frozen"), 14);
    run(r#"def scaled left right =>
    left + right + bias"#);
    assert_eq!(run("result"), 9);
    assert_eq!(run("frozen"), 14);
    run("bias = 4");
    assert_eq!(run("result"), 11);
    assert_eq!(run("frozen"), 16);
}

/// Nucleoid preserves bare multiple parameter block methods and initializers
#[rustfmt::skip]
#[test]
fn preserves_bare_multiple_parameter_block_methods_and_initializers() {
    let mut run = runner();
    run(r#"class Parent:
    def init left, right:
        this.amount = left + right
    def reading offset scale =>
        self.amount * scale + offset"#);
    run(r#"class Child: Parent
    def init left, right:
        super(left, right)
    def reading offset scale =>
        self.amount * scale + offset
    def increase delta, extra:
        this.amount += delta + extra
        return this.amount"#);
    run("child = Child(2, 3)");
    run("observed = child.reading(1, 2)");
    assert_eq!(run("child.id"), "child");
    assert_eq!(run("child.amount"), 5);
    assert_eq!(run("observed"), 11);
    assert_eq!(run("child.increase(2, 1)"), 8);
    assert_eq!(run("child.amount"), 8);
    assert_eq!(run("observed"), 17);
    assert_eq!(run("Parent.length"), 0);
    assert_eq!(run("Child.length"), 1);
}

/// Nucleoid preserves bare multiple parameter block bindings and returns
#[rustfmt::skip]
#[test]
fn preserves_bare_multiple_parameter_block_bindings_and_returns() {
    let mut run = runner();
    run("left = 10");
    run("right = 20");
    run(r#"def pair left right =>
    [left, right]"#);
    run(r#"def add left, right:
    left += right
    return left"#);
    run(r#"def noReturn left, right:
    left + right"#);
    run(r#"def bareReturn left, right:
    return"#);
    run(r#"def outer(initial):
    def combine left right scale =>
        (left + right) * scale
    return combine(initial, 2, 3)"#);
    assert_eq!(run("pair(3, 4)"), serde_json::json!([3, 4]));
    assert_eq!(run("pair(3)"), serde_json::json!([3, null]));
    assert_eq!(run("pair()"), serde_json::json!([null, null]));
    assert_eq!(run("pair(3, 4, 5)"), serde_json::json!([3, 4]));
    assert_eq!(run("add(3, 4)"), 7);
    assert_eq!(run("left"), 10);
    assert_eq!(run("right"), 20);
    assert_eq!(run("noReturn(2, 3)"), serde_json::Value::Null);
    assert_eq!(run("bareReturn(2, 3)"), serde_json::Value::Null);
    assert_eq!(run("outer(4)"), 18);
}

/// Nucleoid rolls back failed bare multiple parameter block redefinitions
#[rustfmt::skip]
#[test]
fn rolls_back_failed_bare_multiple_parameter_block_redefinitions() {
    let (mut run, mut run_error) = crate::common::runners();
    run("source = 3");
    run(r#"def reading left right =>
    source + left + right"#);
    run("total = reading(2, 4)");
    assert_eq!(run("total"), 9);
    assert_eq!(run_error(r#"def reading left right =>
    missing + left + right"#), "ReferenceError: missing is not defined");
    assert_eq!(run("total"), 9);
    run("source = 4");
    assert_eq!(run("total"), 10);
    assert_eq!(run("reading(8, 9)"), 21);
    run(r#"reading(8, 9)

# return: 21"#);
}

/// Nucleoid defines bare typed multiple parameter named arrows
#[rustfmt::skip]
#[test]
fn defines_bare_typed_multiple_parameter_named_arrows() {
    let mut run = runner();
    run("def add left: int, right: int => left + right");
    run("def scaled left: number, right, factor: number => (left + right) * factor");
    run("def typedLast left, right: number => left + right");
    run("def typedFirst left: number, right => left + right");
    assert_eq!(run("add(2, 3)"), 5);
    assert_eq!(run("scaled(2, 3, 4)"), 20);
    assert_eq!(run("typedLast(4, 5)"), 9);
    assert_eq!(run("typedFirst(6, 7)"), 13);
}

/// Nucleoid keeps bare typed multiple parameter arrows reactive
#[rustfmt::skip]
#[test]
fn keeps_bare_typed_multiple_parameter_arrows_reactive() {
    let mut run = runner();
    run("bias = 2");
    run("def scaled left: int, right: int => left * right + bias");
    run("live = scaled");
    run("snapshot = scaled.value");
    run("result = live(3, 4)");
    run("frozen = snapshot(3, 4)");
    assert_eq!(run("result"), 14);
    assert_eq!(run("frozen"), 14);
    run("def scaled left: int, right: int => left + right + bias");
    assert_eq!(run("result"), 9);
    assert_eq!(run("frozen"), 14);
    run("bias = 4");
    assert_eq!(run("result"), 11);
    assert_eq!(run("frozen"), 16);
}

/// Nucleoid preserves bare typed multiple parameter methods and initializers
#[rustfmt::skip]
#[test]
fn preserves_bare_typed_multiple_parameter_methods_and_initializers() {
    let mut run = runner();
    run(r#"class Parent:
    def init left: int, right: int => this.amount = left + right
    def reading offset: int, scale: int => self.amount * scale + offset"#);
    run(r#"class Child: Parent
    def init left: int, right => super(left, right)
    def reading offset, scale: int => self.amount * scale + offset"#);
    run("child = Child(2, 3)");
    run("observed = child.reading(1, 2)");
    assert_eq!(run("child.id"), "child");
    assert_eq!(run("child.amount"), 5);
    assert_eq!(run("observed"), 11);
    run("child.amount = 8");
    assert_eq!(run("observed"), 17);
    assert_eq!(run("Parent.length"), 0);
    assert_eq!(run("Child.length"), 1);
}

/// Nucleoid preserves bare typed multiple parameter arrow bindings and scopes
#[rustfmt::skip]
#[test]
fn preserves_bare_typed_multiple_parameter_arrow_bindings_and_scopes() {
    let (mut run, mut run_error) = crate::common::runners();
    run("left = 10");
    run("right = 20");
    run("def pair left: int, right: int => [left, right]");
    run("def invoke operation: Callback, amount: int => operation(amount)");
    run("def increment(amount: int) => amount + 1");
    run("def operation(amount) => amount + 100");
    run(r#"def outer(initial: int):
    def combine left: int, right, scale: int => (left + right) * scale
    return combine(initial, 2, 3)"#);
    assert_eq!(run("pair(3, 4)"), serde_json::json!([3, 4]));
    assert_eq!(run("pair(3)"), serde_json::json!([3, null]));
    assert_eq!(run("pair()"), serde_json::json!([null, null]));
    assert_eq!(run("pair(3, 4, 5)"), serde_json::json!([3, 4]));
    assert_eq!(run(r#"pair("three", true)"#), serde_json::json!(["three", true]));
    assert_eq!(run("invoke(increment, 4)"), 5);
    assert_eq!(run("invoke(null, 3)"), serde_json::Value::Null);
    assert_eq!(run_error("invoke(7, 3)"), "TypeError: operation is not a function");
    assert_eq!(run("outer(4)"), 18);
    assert_eq!(run("left"), 10);
    assert_eq!(run("right"), 20);
}

/// Nucleoid rolls back failed bare typed multiple parameter arrow redefinitions
#[rustfmt::skip]
#[test]
fn rolls_back_failed_bare_typed_multiple_parameter_arrow_redefinitions() {
    let (mut run, mut run_error) = crate::common::runners();
    run("source = 3");
    run("def reading left: int, right: int => source + left + right");
    run("total = reading(2, 4)");
    assert_eq!(run("total"), 9);
    assert_eq!(run_error("def reading left: int, right: int => missing + left + right"), "ReferenceError: missing is not defined");
    assert_eq!(run("total"), 9);
    run("source = 4");
    assert_eq!(run("total"), 10);
    assert_eq!(run("reading(8, 9)"), 21);
    run(r#"reading(8, 9)

# return: 21"#);
}

/// Nucleoid defines bare typed multiple parameter indented blocks
#[rustfmt::skip]
#[test]
fn defines_bare_typed_multiple_parameter_indented_blocks() {
    let mut run = runner();
    run(r#"def add left: int, right: int:
    total = left + right
    return total"#);
    run(r#"def scaled left: number, right, factor: number =>
    (left + right) * factor"#);
    run(r#"def largest left, right: number:
    if left > right:
        return left
    return right"#);
    assert_eq!(run("add(2, 3)"), 5);
    assert_eq!(run("scaled(2, 3, 4)"), 20);
    assert_eq!(run("largest(7, 3)"), 7);
    assert_eq!(run("largest(2, 9)"), 9);
}

/// Nucleoid keeps bare typed multiple parameter blocks reactive
#[rustfmt::skip]
#[test]
fn keeps_bare_typed_multiple_parameter_blocks_reactive() {
    let mut run = runner();
    run("bias = 2");
    run(r#"def scaled left: int, right =>
    left * right + bias"#);
    run("live = scaled");
    run("snapshot = scaled.value");
    run("result = live(3, 4)");
    run("frozen = snapshot(3, 4)");
    assert_eq!(run("result"), 14);
    assert_eq!(run("frozen"), 14);
    run(r#"def scaled left: int, right: int =>
    left + right + bias"#);
    assert_eq!(run("result"), 9);
    assert_eq!(run("frozen"), 14);
    run("bias = 4");
    assert_eq!(run("result"), 11);
    assert_eq!(run("frozen"), 16);
}

/// Nucleoid preserves bare typed multiple parameter block methods and initializers
#[rustfmt::skip]
#[test]
fn preserves_bare_typed_multiple_parameter_block_methods_and_initializers() {
    let mut run = runner();
    run(r#"class Parent:
    def init left: int, right: int:
        this.amount = left + right
    def reading offset: int, scale =>
        self.amount * scale + offset"#);
    run(r#"class Child: Parent
    def init left: int, right:
        super(left, right)
    def reading offset, scale: int =>
        self.amount * scale + offset
    def increase delta: int, extra:
        this.amount += delta + extra
        return this.amount"#);
    run("child = Child(2, 3)");
    run("observed = child.reading(1, 2)");
    assert_eq!(run("child.id"), "child");
    assert_eq!(run("child.amount"), 5);
    assert_eq!(run("observed"), 11);
    assert_eq!(run("child.increase(2, 1)"), 8);
    assert_eq!(run("child.amount"), 8);
    assert_eq!(run("observed"), 17);
    assert_eq!(run("Parent.length"), 0);
    assert_eq!(run("Child.length"), 1);
}

/// Nucleoid preserves bare typed multiple parameter block bindings and returns
#[rustfmt::skip]
#[test]
fn preserves_bare_typed_multiple_parameter_block_bindings_and_returns() {
    let mut run = runner();
    run("left = 10");
    run("right = 20");
    run(r#"def pair left: int, right: int =>
    [left, right]"#);
    run(r#"def add left: int, right:
    left += right
    return left"#);
    run(r#"def noReturn left: int, right: int:
    left + right"#);
    run(r#"def bareReturn left, right: int:
    return"#);
    run(r#"def outer(initial: int):
    def combine left: int, right, scale: int =>
        (left + right) * scale
    return combine(initial, 2, 3)"#);
    assert_eq!(run("pair(3, 4)"), serde_json::json!([3, 4]));
    assert_eq!(run("pair(3)"), serde_json::json!([3, null]));
    assert_eq!(run("pair()"), serde_json::json!([null, null]));
    assert_eq!(run("pair(3, 4, 5)"), serde_json::json!([3, 4]));
    assert_eq!(run(r#"pair("three", true)"#), serde_json::json!(["three", true]));
    assert_eq!(run("add(3, 4)"), 7);
    assert_eq!(run("left"), 10);
    assert_eq!(run("right"), 20);
    assert_eq!(run("noReturn(2, 3)"), serde_json::Value::Null);
    assert_eq!(run("bareReturn(2, 3)"), serde_json::Value::Null);
    assert_eq!(run("outer(4)"), 18);
}

/// Nucleoid rolls back failed bare typed multiple parameter block redefinitions
#[rustfmt::skip]
#[test]
fn rolls_back_failed_bare_typed_multiple_parameter_block_redefinitions() {
    let (mut run, mut run_error) = crate::common::runners();
    run("source = 3");
    run(r#"def reading left: int, right: int =>
    source + left + right"#);
    run("total = reading(2, 4)");
    assert_eq!(run("total"), 9);
    assert_eq!(run_error(r#"def reading left: int, right =>
    missing + left + right"#), "ReferenceError: missing is not defined");
    assert_eq!(run("total"), 9);
    run("source = 4");
    assert_eq!(run("total"), 10);
    assert_eq!(run("reading(8, 9)"), 21);
    run(r#"reading(8, 9)

# return: 21"#);
}

/// Nucleoid defines implicit iterator indented loops
#[rustfmt::skip]
#[test]
fn defines_implicit_iterator_indented_loops() {
    let mut run = runner();
    run("class Entry");
    run("first = Entry()");
    run("first.amount = 2");
    run("second = Entry()");
    run("second.amount = 5");
    run("values = [first, 99, second, null, { amount: 9 }]");
    run("seen = []");
    run("total = 0");
    run(r#"for values:
    seen.push(it.amount)
    total += it.amount"#);
    assert_eq!(run("seen"), serde_json::json!([2, 5]));
    assert_eq!(run("total"), 7);
    run("empty = []");
    run(r#"for empty:
    total += 100"#);
    assert_eq!(run("total"), 7);
}

/// Nucleoid preserves implicit iterator loop scopes
#[rustfmt::skip]
#[test]
fn preserves_implicit_iterator_loop_scopes() {
    let mut run = runner();
    run("class Entry");
    run("first = Entry()");
    run("first.amount = 2");
    run("second = Entry()");
    run("second.amount = 5");
    run("outer = Entry()");
    run("outer.amount = 100");
    run("groups = [outer]");
    run("values = [first, second]");
    run("it = 50");
    run("seen = []");
    run(r#"for groups:
    seen.push(it.amount)
    for values:
        seen.push(it.amount)
    seen.push(it.amount)"#);
    assert_eq!(run("seen"), serde_json::json!([100, 2, 5, 100]));
    assert_eq!(run("it"), 50);
    run("it = values");
    run("nested = []");
    run(r#"for it:
    nested.push(it.amount)"#);
    assert_eq!(run("nested"), serde_json::json!([2, 5]));
    assert_eq!(run("it.length"), 2);
}

/// Nucleoid preserves implicit iterator loop population snapshots
#[rustfmt::skip]
#[test]
fn preserves_implicit_iterator_loop_population_snapshots() {
    let mut run = runner();
    run("class Reading");
    run("first = Reading()");
    run("first.amount = 2");
    run("second = Reading()");
    run("second.amount = 5");
    run("scale = 2");
    run(r#"for Reading:
    it.score = it.amount * scale"#);
    assert_eq!(run("first.score"), 4);
    assert_eq!(run("second.score"), 10);
    run("scale = 3");
    assert_eq!(run("first.score"), 4);
    assert_eq!(run("second.score"), 10);
    run("third = Reading()");
    run("third.amount = 7");
    assert_eq!(run("third.score"), serde_json::Value::Null);
    run(r#"for Reading:
    it.score = it.amount * scale"#);
    assert_eq!(run("first.score"), 6);
    assert_eq!(run("second.score"), 15);
    assert_eq!(run("third.score"), 21);
}

/// Nucleoid preserves implicit iterator loop return flow
#[rustfmt::skip]
#[test]
fn preserves_implicit_iterator_loop_return_flow() {
    let mut run = runner();
    run("class Entry");
    run("first = Entry()");
    run("first.amount = 2");
    run("second = Entry()");
    run("second.amount = 5");
    run("values = [first, second]");
    run(r#"def firstAmount(records):
    for records:
        return it.amount
    return -1"#);
    run(r#"def noResult(records):
    for records:
        pass"#);
    run(r#"class Reader:
    def first(records):
        for records:
            return it.amount
        return -1"#);
    run("reader = Reader()");
    assert_eq!(run("firstAmount(values)"), 2);
    assert_eq!(run("firstAmount([])"), -1);
    assert_eq!(run("noResult(values)"), serde_json::Value::Null);
    assert_eq!(run("reader.first(values)"), 2);
    assert_eq!(run("reader.first([])"), -1);
    run(r#"reader.first(values)

# return: 2"#);
}

/// Nucleoid rolls back failed implicit iterator loops
#[rustfmt::skip]
#[test]
fn rolls_back_failed_implicit_iterator_loops() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Entry");
    run("first = Entry()");
    run("first.amount = 2");
    run("second = Entry()");
    run("second.amount = 5");
    run("values = [first, second]");
    run("saved = 7");
    assert_eq!(run_error(r#"for values:
    saved += it.amount
    it.amount += 1
    if it.amount > 5:
        throw "stop""#), "stop");
    assert_eq!(run("first.amount"), 2);
    assert_eq!(run("second.amount"), 5);
    assert_eq!(run("saved"), 7);
    run("broken = null");
    assert_eq!(run_error(r#"for broken:
    first.amount = 99"#), "TypeError: broken is not iterable");
    assert_eq!(run("first.amount"), 2);
    run(r#"for values:
    it.score = it.amount * 2"#);
    assert_eq!(run("first.score"), 4);
    assert_eq!(run("second.score"), 10);
    run(r#"first.score + second.score

# return: 14"#);
}

/// Nucleoid defines indented named arrow expressions
#[rustfmt::skip]
#[test]
fn defines_indented_named_arrow_expressions() {
    let mut run = runner();
    run(r#"def literal() =>
    4"#);
    run(r#"def twice amount =>
    amount * 2"#);
    run(r#"def typed amount: int =>
    amount + 1"#);
    run(r#"def add left right =>
    left + right"#);
    run(r#"def scaled left: int, right, factor: number =>
    (left + right) * factor"#);
    assert_eq!(run("literal()"), 4);
    assert_eq!(run("twice(3)"), 6);
    assert_eq!(run("typed(4)"), 5);
    assert_eq!(run("add(2, 3)"), 5);
    assert_eq!(run("scaled(2, 3, 4)"), 20);
    run(r#"scaled(2, 3, 4)

# return: 20"#);
}

/// Nucleoid keeps indented named arrows reactive
#[rustfmt::skip]
#[test]
fn keeps_indented_named_arrows_reactive() {
    let mut run = runner();
    run("offset = 2");
    run(r#"def adjusted number =>
    number + offset"#);
    run("live = adjusted");
    run("snapshot = adjusted.value");
    run("result = live(3)");
    run("frozen = snapshot(3)");
    assert_eq!(run("result"), 5);
    assert_eq!(run("frozen"), 5);
    run(r#"def adjusted number =>
    number * offset"#);
    assert_eq!(run("result"), 6);
    assert_eq!(run("frozen"), 5);
    run("offset = 4");
    assert_eq!(run("result"), 12);
    assert_eq!(run("frozen"), 7);
}

/// Nucleoid preserves indented named arrow methods and initializers
#[rustfmt::skip]
#[test]
fn preserves_indented_named_arrow_methods_and_initializers() {
    let mut run = runner();
    run(r#"class Parent:
    def init value =>
        this.amount = value
    def reading offset: int =>
        self.amount + offset"#);
    run(r#"class Child: Parent
    def init value =>
        super(value)
    def reading offset: int =>
        this.amount * 2 + offset
    def change delta extra =>
        this.amount += delta + extra"#);
    run("child = Child(3)");
    run("observed = child.reading(1)");
    assert_eq!(run("child.id"), "child");
    assert_eq!(run("child.amount"), 3);
    assert_eq!(run("observed"), 7);
    assert_eq!(run("child.change(2, 1)"), 6);
    assert_eq!(run("child.amount"), 6);
    assert_eq!(run("observed"), 13);
    assert_eq!(run("Parent.length"), 0);
    assert_eq!(run("Child.length"), 1);
}

/// Nucleoid preserves indented named arrow bindings and returned values
#[rustfmt::skip]
#[test]
fn preserves_indented_named_arrow_bindings_and_returned_values() {
    let mut run = runner();
    run("left = 10");
    run("right = 20");
    run(r#"def pair left: int, right =>
    [left, right]"#);
    run(r#"def point x y =>
    {x, y}"#);
    run(r#"def text left right =>
    `${left},${right}`"#);
    run(r#"def factory ignored =>
    value => value + 1"#);
    run(r#"def assign left right =>
    left += right"#);
    run(r#"def outer initial:
    def local amount =>
        amount * 2
    return local(initial)"#);
    assert_eq!(run("pair(3, 4)"), serde_json::json!([3, 4]));
    assert_eq!(run("pair(3)"), serde_json::json!([3, null]));
    assert_eq!(run("pair()"), serde_json::json!([null, null]));
    assert_eq!(run(r#"pair("three", true)"#), serde_json::json!(["three", true]));
    run("p = point(2, 3)");
    assert_eq!(run("p.x"), 2);
    assert_eq!(run("p.y"), 3);
    assert_eq!(run("text(2, 3)"), "2,3");
    run("callback = factory(0)");
    assert_eq!(run("callback(5)"), 6);
    assert_eq!(run("assign(3, 4)"), 7);
    assert_eq!(run("left"), 10);
    assert_eq!(run("right"), 20);
    assert_eq!(run("outer(4)"), 8);
}

/// Nucleoid rolls back failed indented named arrow redefinitions
#[rustfmt::skip]
#[test]
fn rolls_back_failed_indented_named_arrow_redefinitions() {
    let (mut run, mut run_error) = crate::common::runners();
    run("source = 3");
    run(r#"def reading left: int, right: int =>
    source + left + right"#);
    run("total = reading(2, 4)");
    assert_eq!(run("total"), 9);
    assert_eq!(run_error(r#"def reading left: int, right =>
    missing + left + right"#), "ReferenceError: missing is not defined");
    assert_eq!(run("total"), 9);
    run("source = 4");
    assert_eq!(run("total"), 10);
    assert_eq!(run("reading(8, 9)"), 21);
    run(r#"reading(8, 9)

# return: 21"#);
}

/// Nucleoid keeps cubic smoothstep reactive to all three inputs
#[rustfmt::skip]
#[test]
fn keeps_cubic_smoothstep_reactive_to_all_three_inputs() {
    let mut run = runner();
    run("lower = 0");
    run("upper = 8");
    run("value = 2");
    run("unit = Math.saturate((value - lower) / (upper - lower))");
    run("eased = Math.smoothstep(lower, upper, value)");
    assert_eq!(run("unit"), 0.25);
    assert_eq!(run("eased"), 0.15625);
    run("value = 4");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    run("upper = 16");
    assert_eq!(run("unit"), 0.25);
    assert_eq!(run("eased"), 0.15625);
    run("lower = -8");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    run("value = -16");
    assert_eq!(run("unit"), 0);
    assert_eq!(run("eased"), 0);
    run("value = 24");
    assert_eq!(run("unit"), 1);
    assert_eq!(run("eased"), 1);
    run("upper = -32");
    assert_eq!(run("unit"), 0);
    assert_eq!(run("eased"), 0);
    run("value = -20");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    run(r#"eased

# return: 0.5"#);
}

/// Nucleoid preserves numeric coercion while smoothing values
#[rustfmt::skip]
#[test]
fn preserves_numeric_coercion_while_smoothing_values() {
    let mut run = runner();
    run(r#"lower = " 0 ""#);
    run(r#"upper = "8""#);
    run(r#"value = "2""#);
    run("unit = Math.saturate((value - lower) / (upper - lower))");
    run("eased = Math.smoothstep(lower, upper, value)");
    assert_eq!(run("unit"), 0.25);
    assert_eq!(run("eased"), 0.15625);
    run("lower = false");
    run("upper = true");
    run("value = true");
    assert_eq!(run("unit"), 1);
    assert_eq!(run("eased"), 1);
    run("lower = []");
    run(r#"upper = ["8"]"#);
    run(r#"value = ["6"]"#);
    assert_eq!(run("unit"), 0.75);
    assert_eq!(run("eased"), 0.84375);
    run("lower = Date(0)");
    run("upper = Date(8)");
    run("value = Date(4)");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    run("value = [1, 2]");
    assert_eq!(run("Number.isNaN(unit)"), true);
    assert_eq!(run("Number.isNaN(eased)"), true);
}

/// Nucleoid preserves special values and omitted smoothstep inputs
#[rustfmt::skip]
#[test]
fn preserves_special_values_and_omitted_smoothstep_inputs() {
    let mut run = runner();
    run("lower = 2");
    run("upper = 2");
    run("value = 2");
    run("unit = Math.saturate((value - lower) / (upper - lower))");
    run("eased = Math.smoothstep(lower, upper, value)");
    assert_eq!(run("Number.isNaN(unit)"), true);
    assert_eq!(run("Number.isNaN(eased)"), true);
    run("value = 1");
    assert_eq!(run("unit"), 0);
    assert_eq!(run("eased"), 0);
    run("value = 3");
    assert_eq!(run("unit"), 1);
    assert_eq!(run("eased"), 1);
    run("upper = 0");
    assert_eq!(run("unit"), 0);
    assert_eq!(run("eased"), 0);
    run("value = 1");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    run("upper = Number.POSITIVE_INFINITY");
    assert_eq!(run("unit"), 0);
    assert_eq!(run("eased"), 0);
    run("value = Number.POSITIVE_INFINITY");
    assert_eq!(run("Number.isNaN(unit)"), true);
    assert_eq!(run("Number.isNaN(eased)"), true);
    run("nan = Number.NaN");
    run("value = nan");
    assert_eq!(run("Number.isNaN(unit)"), true);
    assert_eq!(run("Number.isNaN(eased)"), true);
    run("lower = Number.NEGATIVE_INFINITY");
    run("upper = Number.POSITIVE_INFINITY");
    run("value = 0");
    assert_eq!(run("Number.isNaN(unit)"), true);
    assert_eq!(run("Number.isNaN(eased)"), true);
    run("empty = Math.smoothstep()");
    run("one = Math.smoothstep(0)");
    run("two = Math.smoothstep(0, 1)");
    assert_eq!(run("Number.isNaN(empty)"), true);
    assert_eq!(run("Number.isNaN(one)"), true);
    assert_eq!(run("Number.isNaN(two)"), true);
    run("half = Math.saturate(0.5)");
    run("withExtra = Math.smoothstep(0, 1, 0.5, 99)");
    run("ignored = Number.NaN");
    run("withIgnored = Math.smoothstep(0, 1, 0.5, ignored)");
    assert_eq!(run("half"), 0.5);
    assert_eq!(run("withExtra"), 0.5);
    assert_eq!(run("withIgnored"), 0.5);
}

/// Nucleoid applies cubic smoothstep to live class rules
#[rustfmt::skip]
#[test]
fn applies_cubic_smoothstep_to_live_class_rules() {
    let mut run = runner();
    run("class Curve");
    run("lower = 0");
    run("upper = 8");
    run("$Curve.unit = Math.saturate(($Curve.sample - lower) / (upper - lower))");
    run("$Curve.eased = Math.smoothstep(lower, upper, $Curve.sample)");
    run("first = Curve()");
    run("second = Curve()");
    run("first.sample = 2");
    run("second.sample = 6");
    assert_eq!(run("first.unit"), 0.25);
    assert_eq!(run("first.eased"), 0.15625);
    assert_eq!(run("second.unit"), 0.75);
    assert_eq!(run("second.eased"), 0.84375);
    run("upper = 16");
    assert_eq!(run("first.unit"), 0.125);
    assert_eq!(run("first.eased"), 0.04296875);
    assert_eq!(run("second.unit"), 0.375);
    assert_eq!(run("second.eased"), 0.31640625);
    run("first.sample = 8");
    assert_eq!(run("first.unit"), 0.5);
    assert_eq!(run("first.eased"), 0.5);
    run("third = Curve()");
    assert_eq!(run("third.unit"), serde_json::Value::Null);
    assert_eq!(run("third.eased"), serde_json::Value::Null);
    run("third.sample = 4");
    assert_eq!(run("third.unit"), 0.25);
    assert_eq!(run("third.eased"), 0.15625);
    run("delete first.sample");
    assert_eq!(run("first.unit"), serde_json::Value::Null);
    assert_eq!(run("first.eased"), serde_json::Value::Null);
    run("first.sample = 16");
    assert_eq!(run("first.unit"), 1);
    assert_eq!(run("first.eased"), 1);
    assert_eq!(run("Curve.length"), 3);
}

/// Nucleoid preserves deferred reads and rollback for cubic smoothstep
#[rustfmt::skip]
#[test]
fn preserves_deferred_reads_and_rollback_for_cubic_smoothstep() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Sensor");
    run("sensor = Sensor()");
    run("lower = 0");
    run("upper = 8");
    run("value = sensor.reading");
    run("unit = Math.saturate((value - lower) / (upper - lower))");
    run("eased = Math.smoothstep(lower, upper, value)");
    assert_eq!(run("value"), serde_json::Value::Null);
    assert_eq!(run("unit"), serde_json::Value::Null);
    assert_eq!(run("eased"), serde_json::Value::Null);
    run("sensor.reading = 2");
    assert_eq!(run("unit"), 0.25);
    assert_eq!(run("eased"), 0.15625);
    run("delete sensor.reading");
    assert_eq!(run("value"), serde_json::Value::Null);
    assert_eq!(run("eased"), serde_json::Value::Null);
    run("sensor.reading = 4");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    assert_eq!(run_error(r#"sensor.reading = 6
throw "ROLLBACK""#), "ROLLBACK");
    assert_eq!(run("sensor.reading"), 4);
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    assert_eq!(run_error("lower = missing"), "ReferenceError: missing is not defined");
    assert_eq!(run("lower"), 0);
    run("sensor.reading = 2");
    assert_eq!(run("unit"), 0.25);
    assert_eq!(run("eased"), 0.15625);
    run(r#"eased

# return: 0.15625"#);
}

/// Nucleoid associates nested indented else suites with their conditions
#[rustfmt::skip]
#[test]
fn associates_nested_indented_else_suites_with_their_conditions() {
    let mut run = runner();
    run("class Gauge");
    run("gauge = Gauge()");
    run("gauge.outer = false");
    run("gauge.inner = false");
    run("first = 3");
    run("second = 5");
    run("third = 7");
    run(r#"if gauge.outer:
    gauge.result = first
else
    if gauge.inner:
        gauge.result = second
    else
        gauge.result = third"#);
    assert_eq!(run("gauge.result"), 7);
    run("gauge.inner = true");
    assert_eq!(run("gauge.result"), 5);
    run("gauge.outer = true");
    assert_eq!(run("gauge.result"), 3);
    run("first = 11");
    assert_eq!(run("gauge.result"), 11);
    run("gauge.outer = false");
    assert_eq!(run("gauge.result"), 5);
    run("second = 13");
    assert_eq!(run("gauge.result"), 13);
    run("gauge.inner = false");
    assert_eq!(run("gauge.result"), 7);
    run("third = 17");
    assert_eq!(run("gauge.result"), 17);
    run(r#"gauge.result

# return: 17"#);
}

/// Nucleoid keeps indented else class rules independent per instance
#[rustfmt::skip]
#[test]
fn keeps_indented_else_class_rules_independent_per_instance() {
    let mut run = runner();
    run("class Tier");
    run("first = Tier()");
    run("first.score = 1");
    run("second = Tier()");
    run("second.score = 9");
    run("threshold = 5");
    run(r#"low = "LOW""#);
    run(r#"high = "HIGH""#);
    run(r#"if $Tier.score >= threshold:
    $Tier.label = high
else
    $Tier.label = low"#);
    assert_eq!(run("first.label"), "LOW");
    assert_eq!(run("second.label"), "HIGH");
    run(r#"low = "L""#);
    assert_eq!(run("first.label"), "L");
    assert_eq!(run("second.label"), "HIGH");
    run("first.score = 8");
    assert_eq!(run("first.label"), "HIGH");
    assert_eq!(run("second.label"), "HIGH");
    run("threshold = 10");
    assert_eq!(run("first.label"), "L");
    assert_eq!(run("second.label"), "L");
    run("second.score = 12");
    assert_eq!(run("first.label"), "L");
    assert_eq!(run("second.label"), "HIGH");
}

/// Nucleoid preserves returns from indented else function branches
#[rustfmt::skip]
#[test]
fn preserves_returns_from_indented_else_function_branches() {
    let mut run = runner();
    run(r#"def choose(flag, value):
    if flag:
        return value
    else
        return value + 1"#);
    run("enabled = false");
    run("amount = 2");
    run("chosen = choose(enabled, amount)");
    assert_eq!(run("chosen"), 3);
    run("enabled = true");
    assert_eq!(run("chosen"), 2);
    run("amount = 5");
    assert_eq!(run("chosen"), 5);
    run("enabled = false");
    assert_eq!(run("chosen"), 6);
    run(r#"chosen

# return: 6"#);
}

/// Nucleoid rolls back throws from indented else suites
#[rustfmt::skip]
#[test]
fn rolls_back_throws_from_indented_else_suites() {
    let (mut run, mut run_error) = crate::common::runners();
    run("marker = 7");
    assert_eq!(run_error(r#"if false:
    marker = 8
else
    marker = 9
    throw "ABORT""#), "ABORT");
    assert_eq!(run("marker"), 7);
    run("gate = false");
    run(r#"if gate:
    result = 1
else
    result = 2"#);
    assert_eq!(run("result"), 2);
    run("gate = true");
    assert_eq!(run("result"), 1);
}

/// Nucleoid preserves legacy suites beside indented else bodies
#[rustfmt::skip]
#[test]
fn preserves_legacy_suites_beside_indented_else_bodies() {
    let mut run = runner();
    run("result = 0");
    run(r#"if true: result = 1
else: result = 2"#);
    assert_eq!(run("result"), 1);
    run(r#"if false:
    result = 3
else
    result = 4"#);
    assert_eq!(run("result"), 4);
    run(r#"if true: { result = 5 }
else: { result = 6 }"#);
    assert_eq!(run("result"), 5);
    run(r#"text = "else:""#);
    assert_eq!(run("text"), "else:");
    run("flag = false");
    run("value = 2");
    run(r#"if flag:
    answer = value
else # else: in a comment is not a delimiter
    answer = value + 10"#);
    assert_eq!(run("answer"), 12);
    run("flag = true");
    assert_eq!(run("answer"), 2);
    run("value = 6");
    assert_eq!(run("answer"), 6);
}

/// Nucleoid keeps quintic smootherstep reactive to all three inputs
#[rustfmt::skip]
#[test]
fn keeps_quintic_smootherstep_reactive_to_all_three_inputs() {
    let mut run = runner();
    run("lower = 0");
    run("upper = 8");
    run("value = 2");
    run("unit = Math.saturate((value - lower) / (upper - lower))");
    run("eased = Math.smootherstep(lower, upper, value)");
    assert_eq!(run("unit"), 0.25);
    assert_eq!(run("eased"), 0.103515625);
    run("value = 4");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    run("upper = 16");
    assert_eq!(run("unit"), 0.25);
    assert_eq!(run("eased"), 0.103515625);
    run("lower = -8");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    run("value = -16");
    assert_eq!(run("unit"), 0);
    assert_eq!(run("eased"), 0);
    run("value = 24");
    assert_eq!(run("unit"), 1);
    assert_eq!(run("eased"), 1);
    run("upper = -32");
    assert_eq!(run("unit"), 0);
    assert_eq!(run("eased"), 0);
    run("value = -20");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    run(r#"eased

# return: 0.5"#);
}

/// Nucleoid preserves numeric coercion while applying quintic smootherstep
#[rustfmt::skip]
#[test]
fn preserves_numeric_coercion_while_applying_quintic_smootherstep() {
    let mut run = runner();
    run(r#"lower = " 0 ""#);
    run(r#"upper = "8""#);
    run(r#"value = "2""#);
    run("unit = Math.saturate((value - lower) / (upper - lower))");
    run("eased = Math.smootherstep(lower, upper, value)");
    assert_eq!(run("unit"), 0.25);
    assert_eq!(run("eased"), 0.103515625);
    run("lower = false");
    run("upper = true");
    run("value = true");
    assert_eq!(run("unit"), 1);
    assert_eq!(run("eased"), 1);
    run("lower = []");
    run(r#"upper = ["8"]"#);
    run(r#"value = ["6"]"#);
    assert_eq!(run("unit"), 0.75);
    assert_eq!(run("eased"), 0.896484375);
    run("lower = Date(0)");
    run("upper = Date(8)");
    run("value = Date(4)");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    run("value = [1, 2]");
    assert_eq!(run("Number.isNaN(unit)"), true);
    assert_eq!(run("Number.isNaN(eased)"), true);
}

/// Nucleoid preserves special values and omitted smootherstep inputs
#[rustfmt::skip]
#[test]
fn preserves_special_values_and_omitted_smootherstep_inputs() {
    let mut run = runner();
    run("lower = 2");
    run("upper = 2");
    run("value = 2");
    run("unit = Math.saturate((value - lower) / (upper - lower))");
    run("eased = Math.smootherstep(lower, upper, value)");
    assert_eq!(run("Number.isNaN(unit)"), true);
    assert_eq!(run("Number.isNaN(eased)"), true);
    run("value = 1");
    assert_eq!(run("unit"), 0);
    assert_eq!(run("eased"), 0);
    run("value = 3");
    assert_eq!(run("unit"), 1);
    assert_eq!(run("eased"), 1);
    run("upper = 0");
    assert_eq!(run("unit"), 0);
    assert_eq!(run("eased"), 0);
    run("value = 1");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    run("upper = Number.POSITIVE_INFINITY");
    assert_eq!(run("unit"), 0);
    assert_eq!(run("eased"), 0);
    run("value = Number.POSITIVE_INFINITY");
    assert_eq!(run("Number.isNaN(unit)"), true);
    assert_eq!(run("Number.isNaN(eased)"), true);
    run("nan = Number.NaN");
    run("value = nan");
    assert_eq!(run("Number.isNaN(unit)"), true);
    assert_eq!(run("Number.isNaN(eased)"), true);
    run("lower = Number.NEGATIVE_INFINITY");
    run("upper = Number.POSITIVE_INFINITY");
    run("value = 0");
    assert_eq!(run("Number.isNaN(unit)"), true);
    assert_eq!(run("Number.isNaN(eased)"), true);
    run("empty = Math.smootherstep()");
    run("one = Math.smootherstep(0)");
    run("two = Math.smootherstep(0, 1)");
    assert_eq!(run("Number.isNaN(empty)"), true);
    assert_eq!(run("Number.isNaN(one)"), true);
    assert_eq!(run("Number.isNaN(two)"), true);
    run("half = Math.saturate(0.5)");
    run("withExtra = Math.smootherstep(0, 1, 0.5, 99)");
    run("ignored = Number.NaN");
    run("withIgnored = Math.smootherstep(0, 1, 0.5, ignored)");
    assert_eq!(run("half"), 0.5);
    assert_eq!(run("withExtra"), 0.5);
    assert_eq!(run("withIgnored"), 0.5);
}

/// Nucleoid applies quintic smootherstep to live class rules
#[rustfmt::skip]
#[test]
fn applies_quintic_smootherstep_to_live_class_rules() {
    let mut run = runner();
    run("class Curve");
    run("lower = 0");
    run("upper = 8");
    run("$Curve.unit = Math.saturate(($Curve.sample - lower) / (upper - lower))");
    run("$Curve.eased = Math.smootherstep(lower, upper, $Curve.sample)");
    run("first = Curve()");
    run("second = Curve()");
    run("first.sample = 2");
    run("second.sample = 6");
    assert_eq!(run("first.unit"), 0.25);
    assert_eq!(run("first.eased"), 0.103515625);
    assert_eq!(run("second.unit"), 0.75);
    assert_eq!(run("second.eased"), 0.896484375);
    run("upper = 16");
    assert_eq!(run("first.unit"), 0.125);
    assert_eq!(run("first.eased"), 0.01605224609375);
    assert_eq!(run("second.unit"), 0.375);
    assert_eq!(run("second.eased"), 0.27520751953125);
    run("first.sample = 8");
    assert_eq!(run("first.unit"), 0.5);
    assert_eq!(run("first.eased"), 0.5);
    run("third = Curve()");
    assert_eq!(run("third.unit"), serde_json::Value::Null);
    assert_eq!(run("third.eased"), serde_json::Value::Null);
    run("third.sample = 4");
    assert_eq!(run("third.unit"), 0.25);
    assert_eq!(run("third.eased"), 0.103515625);
    run("delete first.sample");
    assert_eq!(run("first.unit"), serde_json::Value::Null);
    assert_eq!(run("first.eased"), serde_json::Value::Null);
    run("first.sample = 16");
    assert_eq!(run("first.unit"), 1);
    assert_eq!(run("first.eased"), 1);
    assert_eq!(run("Curve.length"), 3);
}

/// Nucleoid preserves deferred reads and rollback for quintic smootherstep
#[rustfmt::skip]
#[test]
fn preserves_deferred_reads_and_rollback_for_quintic_smootherstep() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Sensor");
    run("sensor = Sensor()");
    run("lower = 0");
    run("upper = 8");
    run("value = sensor.reading");
    run("unit = Math.saturate((value - lower) / (upper - lower))");
    run("eased = Math.smootherstep(lower, upper, value)");
    assert_eq!(run("value"), serde_json::Value::Null);
    assert_eq!(run("unit"), serde_json::Value::Null);
    assert_eq!(run("eased"), serde_json::Value::Null);
    run("sensor.reading = 2");
    assert_eq!(run("unit"), 0.25);
    assert_eq!(run("eased"), 0.103515625);
    run("delete sensor.reading");
    assert_eq!(run("value"), serde_json::Value::Null);
    assert_eq!(run("eased"), serde_json::Value::Null);
    run("sensor.reading = 4");
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    assert_eq!(run_error(r#"sensor.reading = 6
throw "ROLLBACK""#), "ROLLBACK");
    assert_eq!(run("sensor.reading"), 4);
    assert_eq!(run("unit"), 0.5);
    assert_eq!(run("eased"), 0.5);
    assert_eq!(run_error("lower = missing"), "ReferenceError: missing is not defined");
    assert_eq!(run("lower"), 0);
    run("sensor.reading = 2");
    assert_eq!(run("unit"), 0.25);
    assert_eq!(run("eased"), 0.103515625);
    run(r#"eased

# return: 0.103515625"#);
}

/// Nucleoid keeps floor fractional parts reactive
#[rustfmt::skip]
#[test]
fn keeps_floor_fractional_parts_reactive() {
    let mut run = runner();
    run("value = 2.75");
    run("whole = Math.floor(value)");
    run("fraction = Math.fract(value)");
    assert_eq!(run("whole"), 2);
    assert_eq!(run("fraction"), 0.75);
    run("value = -1.25");
    assert_eq!(run("whole"), -2);
    assert_eq!(run("fraction"), 0.75);
    run("value = 4.125");
    assert_eq!(run("whole"), 4);
    assert_eq!(run("fraction"), 0.125);
    run("value = -2");
    assert_eq!(run("whole"), -2);
    assert_eq!(run("fraction"), 0);
    run("value = 0");
    assert_eq!(run("whole"), 0);
    assert_eq!(run("fraction"), 0);
    run("value = -0");
    assert_eq!(run("whole"), 0);
    assert_eq!(run("fraction"), 0);
    run("value = 1.5");
    assert_eq!(run("whole"), 1);
    assert_eq!(run("fraction"), 0.5);
    run(r#"fraction

# return: 0.5"#);
}

/// Nucleoid preserves numeric coercion while computing floor fractional parts
#[rustfmt::skip]
#[test]
fn preserves_numeric_coercion_while_computing_floor_fractional_parts() {
    let mut run = runner();
    run(r#"value = " 2.75 ""#);
    run("whole = Math.floor(value)");
    run("fraction = Math.fract(value)");
    assert_eq!(run("whole"), 2);
    assert_eq!(run("fraction"), 0.75);
    run("value = true");
    assert_eq!(run("whole"), 1);
    assert_eq!(run("fraction"), 0);
    run("value = false");
    assert_eq!(run("whole"), 0);
    assert_eq!(run("fraction"), 0);
    run("value = []");
    assert_eq!(run("whole"), 0);
    assert_eq!(run("fraction"), 0);
    run(r#"value = ["3.125"]"#);
    assert_eq!(run("whole"), 3);
    assert_eq!(run("fraction"), 0.125);
    run("value = Date(1234)");
    assert_eq!(run("whole"), 1234);
    assert_eq!(run("fraction"), 0);
    run("value = [1, 2]");
    assert_eq!(run("Number.isNaN(whole)"), true);
    assert_eq!(run("Number.isNaN(fraction)"), true);
    run(r#"value = "not-a-number""#);
    assert_eq!(run("Number.isNaN(whole)"), true);
    assert_eq!(run("Number.isNaN(fraction)"), true);
}

/// Nucleoid preserves nonfinite fractional parts omissions and rounded endpoints
#[rustfmt::skip]
#[test]
fn preserves_nonfinite_fractional_parts_omissions_and_rounded_endpoints() {
    let mut run = runner();
    run("value = -0.00000000000000000001");
    run("whole = Math.floor(value)");
    run("fraction = Math.fract(value)");
    assert_eq!(run("whole"), -1);
    assert_eq!(run("fraction"), 1);
    run("value = Number.POSITIVE_INFINITY");
    {
        let actual = run("whole");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("Number.isNaN(fraction)"), true);
    run("value = Number.NEGATIVE_INFINITY");
    {
        let actual = run("whole");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("Number.isNaN(fraction)"), true);
    run("value = Number.NaN");
    assert_eq!(run("Number.isNaN(whole)"), true);
    assert_eq!(run("Number.isNaN(fraction)"), true);
    run("value = Number.MAX_VALUE");
    {
        let actual = run("whole");
        let expected = run("(Number.MAX_VALUE)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("fraction"), 0);
    run("value = -Number.MAX_VALUE");
    {
        let actual = run("whole");
        let expected = run("(-Number.MAX_VALUE)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("fraction"), 0);
    run("empty = Math.fract()");
    assert_eq!(run("Number.isNaN(empty)"), true);
    run("half = 0.5");
    run("withExtra = Math.fract(half, 99)");
    run("ignored = Number.NaN");
    run("withIgnored = Math.fract(half, ignored)");
    assert_eq!(run("half"), 0.5);
    assert_eq!(run("withExtra"), 0.5);
    assert_eq!(run("withIgnored"), 0.5);
    run(r#"ignored = "not-a-number""#);
    assert_eq!(run("withIgnored"), 0.5);
}

/// Nucleoid applies floor fractional parts to live class rules
#[rustfmt::skip]
#[test]
fn applies_floor_fractional_parts_to_live_class_rules() {
    let mut run = runner();
    run("class Reading");
    run("$Reading.whole = Math.floor($Reading.sample)");
    run("$Reading.fraction = Math.fract($Reading.sample)");
    run("first = Reading()");
    run("second = Reading()");
    run("first.sample = 2.25");
    run("second.sample = -1.75");
    assert_eq!(run("first.whole"), 2);
    assert_eq!(run("first.fraction"), 0.25);
    assert_eq!(run("second.whole"), -2);
    assert_eq!(run("second.fraction"), 0.25);
    run("first.sample = -1.25");
    assert_eq!(run("first.whole"), -2);
    assert_eq!(run("first.fraction"), 0.75);
    assert_eq!(run("second.fraction"), 0.25);
    run("third = Reading()");
    assert_eq!(run("third.whole"), serde_json::Value::Null);
    assert_eq!(run("third.fraction"), serde_json::Value::Null);
    run("third.sample = 8.125");
    assert_eq!(run("third.whole"), 8);
    assert_eq!(run("third.fraction"), 0.125);
    run("delete first.sample");
    assert_eq!(run("first.whole"), serde_json::Value::Null);
    assert_eq!(run("first.fraction"), serde_json::Value::Null);
    run("first.sample = -3.5");
    assert_eq!(run("first.whole"), -4);
    assert_eq!(run("first.fraction"), 0.5);
    assert_eq!(run("Reading.length"), 3);
}

/// Nucleoid preserves deferred fractional parts and rollback
#[rustfmt::skip]
#[test]
fn preserves_deferred_fractional_parts_and_rollback() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Sensor");
    run("sensor = Sensor()");
    run("value = sensor.reading");
    run("whole = Math.floor(value)");
    run("fraction = Math.fract(value)");
    run("withDeferredExtra = Math.fract(value, sensor.optional)");
    assert_eq!(run("value"), serde_json::Value::Null);
    assert_eq!(run("whole"), serde_json::Value::Null);
    assert_eq!(run("fraction"), serde_json::Value::Null);
    assert_eq!(run("withDeferredExtra"), serde_json::Value::Null);
    run("sensor.reading = 2.25");
    assert_eq!(run("whole"), 2);
    assert_eq!(run("fraction"), 0.25);
    assert_eq!(run("withDeferredExtra"), serde_json::Value::Null);
    run("delete sensor.reading");
    assert_eq!(run("value"), serde_json::Value::Null);
    assert_eq!(run("fraction"), serde_json::Value::Null);
    run("sensor.reading = 3.75");
    assert_eq!(run("whole"), 3);
    assert_eq!(run("fraction"), 0.75);
    run("sensor.optional = 99");
    assert_eq!(run("withDeferredExtra"), 0.75);
    assert_eq!(run_error(r#"sensor.reading = 5.5
throw "ROLLBACK""#), "ROLLBACK");
    assert_eq!(run("sensor.reading"), 3.75);
    assert_eq!(run("whole"), 3);
    assert_eq!(run("fraction"), 0.75);
    assert_eq!(run("withDeferredExtra"), 0.75);
    assert_eq!(run_error("sensor.reading = missing"), "ReferenceError: missing is not defined");
    assert_eq!(run("sensor.reading"), 3.75);
    run("delete sensor.optional");
    assert_eq!(run("withDeferredExtra"), serde_json::Value::Null);
    assert_eq!(run("fraction"), 0.75);
    run("sensor.optional = [1, 2]");
    assert_eq!(run("withDeferredExtra"), 0.75);
    run("sensor.reading = 2.25");
    assert_eq!(run("whole"), 2);
    assert_eq!(run("fraction"), 0.25);
    assert_eq!(run("withDeferredExtra"), 0.25);
    run(r#"fraction

# return: 0.25"#);
}

/// Nucleoid defines comma-free bare named arrows
#[rustfmt::skip]
#[test]
fn defines_comma_free_bare_named_arrows() {
    let mut run = runner();
    run("def add left right => left + right");
    run("def scaled left right factor => (left + right) * factor");
    run("def pair left right => [left, right]");
    run("def coordinates x y => {x, y}");
    run(r#"def indented left right =>
    left + right"#);
    assert_eq!(run("add(2, 3)"), 5);
    assert_eq!(run("scaled(2, 3, 4)"), 20);
    assert_eq!(run(r#"pair("left", "right")"#), serde_json::json!(["left", "right"]));
    run("point = coordinates(2, 3)");
    assert_eq!(run("point.x"), 2);
    assert_eq!(run("point.y"), 3);
    assert_eq!(run("indented(2, 3)"), 5);
}

/// Nucleoid keeps comma-free bare named arrows reactive
#[rustfmt::skip]
#[test]
fn keeps_comma_free_bare_named_arrows_reactive() {
    let mut run = runner();
    run("bias = 1");
    run("def total left right => left + right + bias");
    run("tracked = total(2, 3)");
    assert_eq!(run("tracked"), 6);
    run("bias = 4");
    assert_eq!(run("tracked"), 9);
    run("def projected first second third => first * second + third");
    run("left = 2");
    run("right = 3");
    run("offset = 4");
    run("result = projected(left, right, offset)");
    assert_eq!(run("result"), 10);
    run("left = 5");
    assert_eq!(run("result"), 19);
    run("right = 2");
    assert_eq!(run("result"), 14);
}

/// Nucleoid preserves comma-free methods and initializers
#[rustfmt::skip]
#[test]
fn preserves_comma_free_methods_and_initializers() {
    let mut run = runner();
    run(r#"class Calculator:
    def init base scale => this.base = base * scale
    def combine left right => this.base + left + right
    def product left right scale => self.base + left * right * scale"#);
    run("calculator = Calculator(1, 2)");
    run("result = calculator.combine(3, 4)");
    assert_eq!(run("result"), 9);
    run("calculator.base = 5");
    assert_eq!(run("result"), 12);
    assert_eq!(run("calculator.product(2, 3, 4)"), 29);
}

/// Nucleoid preserves nested comma-free bare named arrows
#[rustfmt::skip]
#[test]
fn preserves_nested_comma_free_bare_named_arrows() {
    let mut run = runner();
    run(r#"def outer(initial):
    def combine left right scale => (left + right) * scale
    return combine(initial, 2, 3)"#);
    assert_eq!(run("outer(4)"), 18);
    run("def factory(ignored) => (left, right) => left + right");
    run("custom = factory(10)");
    assert_eq!(run("custom(2, 3)"), 5);
}

/// Nucleoid rolls back comma-free named arrow redefinitions
#[rustfmt::skip]
#[test]
fn rolls_back_comma_free_named_arrow_redefinitions() {
    let (mut run, mut run_error) = crate::common::runners();
    run("source = 3");
    run("def reading left right => source + left + right");
    run("result = reading(2, 4)");
    assert_eq!(run("result"), 9);
    assert_eq!(run_error("def reading left right => missing + left + right"), "ReferenceError: missing is not defined");
    assert_eq!(run("result"), 9);
    run("source = 5");
    assert_eq!(run("result"), 11);
    run("def prefer current fallback => current ||= fallback");
    run("def combined first second scale => (first + second) * scale");
    assert_eq!(run("prefer(0, 9)"), 9);
    assert_eq!(run("prefer(3, 9)"), 3);
    assert_eq!(run("combined(2, 5, 3)"), 21);
    run(r#"combined(2, 5, 3)

# return: 21"#);
}

/// Nucleoid keeps inverse interpolation reactive
#[rustfmt::skip]
#[test]
fn keeps_inverse_interpolation_reactive() {
    let mut run = runner();
    run("start = 2");
    run("end = 10");
    run("sample = 4");
    run("weight = Math.inverseLerp(start, end, sample)");
    run("reconstructed = Math.lerp(start, end, weight)");
    assert_eq!(run("weight"), 0.25);
    assert_eq!(run("reconstructed"), 4);
    run("sample = 6");
    assert_eq!(run("weight"), 0.5);
    assert_eq!(run("reconstructed"), 6);
    run("sample = 14");
    assert_eq!(run("weight"), 1.5);
    assert_eq!(run("reconstructed"), 14);
    run("sample = -2");
    assert_eq!(run("weight"), -0.5);
    assert_eq!(run("reconstructed"), -2);
    run("start = 10");
    run("end = 2");
    run("sample = 8");
    assert_eq!(run("weight"), 0.25);
    assert_eq!(run("reconstructed"), 8);
    run("sample = 12");
    assert_eq!(run("weight"), -0.25);
    assert_eq!(run("reconstructed"), 12);
    run("sample = 6");
    assert_eq!(run("weight"), 0.5);
    assert_eq!(run("reconstructed"), 6);
    run(r#"weight

# return: 0.5"#);
}

/// Nucleoid preserves intrinsic conversion during inverse interpolation
#[rustfmt::skip]
#[test]
fn preserves_intrinsic_conversion_during_inverse_interpolation() {
    let mut run = runner();
    run(r#"start = " 2 ""#);
    run(r#"end = "10""#);
    run(r#"sample = "4""#);
    run("weight = Math.inverseLerp(start, end, sample)");
    assert_eq!(run("weight"), 0.25);
    assert_eq!(run("start"), " 2 ");
    run("start = false");
    run("end = true");
    run("sample = false");
    assert_eq!(run("weight"), 0);
    run("sample = true");
    assert_eq!(run("weight"), 1);
    run("sample = [0.5]");
    assert_eq!(run("weight"), 0.5);
    run("sample = []");
    assert_eq!(run("weight"), 0);
    run(r#"sample = "not-a-number""#);
    assert_eq!(run("Number.isNaN(weight)"), true);
    run("sample = [1, 2]");
    assert_eq!(run("Number.isNaN(weight)"), true);
    run("start = Date(0)");
    run("end = Date(8)");
    run("sample = Date(2)");
    assert_eq!(run("weight"), 0.25);
    run("sample = Date(6)");
    assert_eq!(run("weight"), 0.75);
    run("start = []");
    run(r#"end = ["8"]"#);
    run(r#"sample = ["2"]"#);
    assert_eq!(run("weight"), 0.25);
    run("sample = false");
    assert_eq!(run("weight"), 0);
}

/// Nucleoid preserves inverse interpolation endpoints nonfinite inputs and omissions
#[rustfmt::skip]
#[test]
fn preserves_inverse_interpolation_endpoints_nonfinite_inputs_and_omissions() {
    let mut run = runner();
    run("start = 0");
    run("end = 8");
    run("sample = 2");
    run("weight = Math.inverseLerp(start, end, sample)");
    assert_eq!(run("weight"), 0.25);
    run("sample = start");
    assert_eq!(run("weight"), 0);
    run("sample = end");
    assert_eq!(run("weight"), 1);
    run("sample = 16");
    assert_eq!(run("weight"), 2);
    run("sample = -8");
    assert_eq!(run("weight"), -1);
    run("start = 2");
    run("end = 2");
    run("sample = 2");
    assert_eq!(run("Number.isNaN(weight)"), true);
    run("sample = 3");
    {
        let actual = run("weight");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("sample = 1");
    {
        let actual = run("weight");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("start = 0");
    run("end = Number.POSITIVE_INFINITY");
    run("sample = 8");
    assert_eq!(run("weight"), 0);
    run("sample = Number.POSITIVE_INFINITY");
    assert_eq!(run("Number.isNaN(weight)"), true);
    run("end = 8");
    run("sample = Number.NEGATIVE_INFINITY");
    {
        let actual = run("weight");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("start = Number.NEGATIVE_INFINITY");
    run("end = Number.POSITIVE_INFINITY");
    run("sample = 0");
    assert_eq!(run("Number.isNaN(weight)"), true);
    run("start = -Number.MAX_VALUE");
    run("end = Number.MAX_VALUE");
    run("sample = 0");
    assert_eq!(run("weight"), 0);
    run("start = 0");
    run("end = 8");
    run("sample = Number.NaN");
    assert_eq!(run("Number.isNaN(weight)"), true);
    run("empty = Math.inverseLerp()");
    assert_eq!(run("Number.isNaN(empty)"), true);
    run("one = Math.inverseLerp(0)");
    assert_eq!(run("Number.isNaN(one)"), true);
    run("two = Math.inverseLerp(0, 8)");
    assert_eq!(run("Number.isNaN(two)"), true);
    run("literal = Math.inverseLerp(false, true, null)");
    assert_eq!(run("literal"), 0);
    run("withExtra = Math.inverseLerp(0, 8, 2, 99)");
    assert_eq!(run("withExtra"), 0.25);
    run("ignored = Number.NaN");
    run("withIgnored = Math.inverseLerp(0, 8, 2, ignored)");
    assert_eq!(run("withIgnored"), 0.25);
    run(r#"ignored = "not-a-number""#);
    assert_eq!(run("withIgnored"), 0.25);
    run("ignored = null");
    assert_eq!(run("withIgnored"), serde_json::Value::Null);
    run("ignored = []");
    assert_eq!(run("withIgnored"), 0.25);
}

/// Nucleoid applies inverse interpolation to live class rules
#[rustfmt::skip]
#[test]
fn applies_inverse_interpolation_to_live_class_rules() {
    let mut run = runner();
    run("class Gauge");
    run("$Gauge.weight = Math.inverseLerp($Gauge.start, $Gauge.end, $Gauge.sample)");
    run("first = Gauge()");
    run("second = Gauge()");
    run("first.start = 0");
    run("first.end = 8");
    run("first.sample = 2");
    run("second.start = 10");
    run("second.end = 2");
    run("second.sample = 8");
    assert_eq!(run("first.weight"), 0.25);
    assert_eq!(run("second.weight"), 0.25);
    run("first.sample = 12");
    assert_eq!(run("first.weight"), 1.5);
    assert_eq!(run("second.weight"), 0.25);
    run("first.end = 16");
    assert_eq!(run("first.weight"), 0.75);
    run("third = Gauge()");
    assert_eq!(run("third.weight"), serde_json::Value::Null);
    run("third.start = 0");
    run("third.end = 4");
    run("third.sample = 1");
    assert_eq!(run("third.weight"), 0.25);
    run("delete first.sample");
    assert_eq!(run("first.weight"), serde_json::Value::Null);
    run("first.sample = -4");
    assert_eq!(run("first.weight"), -0.25);
    assert_eq!(run("Gauge.length"), 3);
}

/// Nucleoid preserves deferred inverse interpolation and rollback
#[rustfmt::skip]
#[test]
fn preserves_deferred_inverse_interpolation_and_rollback() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Sensor");
    run("sensor = Sensor()");
    run("start = sensor.lower");
    run("end = sensor.upper");
    run("sample = sensor.reading");
    run("weight = Math.inverseLerp(start, end, sample)");
    run("withDeferredExtra = Math.inverseLerp(start, end, sample, sensor.optional)");
    assert_eq!(run("weight"), serde_json::Value::Null);
    assert_eq!(run("withDeferredExtra"), serde_json::Value::Null);
    run("sensor.lower = 0");
    run("sensor.upper = 8");
    run("sensor.reading = 2");
    assert_eq!(run("weight"), 0.25);
    assert_eq!(run("withDeferredExtra"), serde_json::Value::Null);
    run("sensor.optional = 99");
    assert_eq!(run("withDeferredExtra"), 0.25);
    run("delete sensor.reading");
    assert_eq!(run("weight"), serde_json::Value::Null);
    assert_eq!(run("withDeferredExtra"), serde_json::Value::Null);
    run("sensor.reading = 6");
    assert_eq!(run("weight"), 0.75);
    assert_eq!(run("withDeferredExtra"), 0.75);
    assert_eq!(run_error(r#"sensor.reading = 4
throw "ROLLBACK""#), "ROLLBACK");
    assert_eq!(run("sensor.reading"), 6);
    assert_eq!(run("weight"), 0.75);
    assert_eq!(run("withDeferredExtra"), 0.75);
    assert_eq!(run_error("sensor.reading = missing"), "ReferenceError: missing is not defined");
    assert_eq!(run("sensor.reading"), 6);
    run("delete sensor.optional");
    assert_eq!(run("withDeferredExtra"), serde_json::Value::Null);
    assert_eq!(run("weight"), 0.75);
    run("sensor.optional = [1, 2]");
    assert_eq!(run("withDeferredExtra"), 0.75);
    run("sensor.reading = 2");
    assert_eq!(run("weight"), 0.25);
    assert_eq!(run("withDeferredExtra"), 0.25);
    run(r#"weight

# return: 0.25"#);
}

/// Nucleoid keeps range remapping reactive
#[rustfmt::skip]
#[test]
fn keeps_range_remapping_reactive() {
    let mut run = runner();
    run("start = 2");
    run("end = 10");
    run("targetStart = 10");
    run("targetEnd = 30");
    run("sample = 4");
    run("mapped = Math.remap(start, end, targetStart, targetEnd, sample)");
    run("unit = Math.inverseLerp(start, end, sample)");
    assert_eq!(run("mapped"), 15);
    assert_eq!(run("unit"), 0.25);
    run("sample = 6");
    assert_eq!(run("mapped"), 20);
    assert_eq!(run("unit"), 0.5);
    run("sample = 14");
    assert_eq!(run("mapped"), 40);
    assert_eq!(run("unit"), 1.5);
    run("sample = -2");
    assert_eq!(run("mapped"), 0);
    assert_eq!(run("unit"), -0.5);
    run("start = 10");
    run("end = 2");
    run("sample = 8");
    assert_eq!(run("mapped"), 15);
    assert_eq!(run("unit"), 0.25);
    run("targetStart = 30");
    run("targetEnd = 10");
    assert_eq!(run("mapped"), 25);
    assert_eq!(run("unit"), 0.25);
    run("sample = 6");
    assert_eq!(run("mapped"), 20);
    assert_eq!(run("unit"), 0.5);
    run(r#"mapped

# return: 20"#);
}

/// Nucleoid preserves intrinsic conversion during range remapping
#[rustfmt::skip]
#[test]
fn preserves_intrinsic_conversion_during_range_remapping() {
    let mut run = runner();
    run(r#"start = " 2 ""#);
    run(r#"end = "10""#);
    run(r#"targetStart = "10""#);
    run(r#"targetEnd = "30""#);
    run(r#"sample = "4""#);
    run("mapped = Math.remap(start, end, targetStart, targetEnd, sample)");
    assert_eq!(run("mapped"), 15);
    assert_eq!(run("start"), " 2 ");
    run("start = false");
    run("end = true");
    run("targetStart = false");
    run("targetEnd = true");
    run("sample = false");
    assert_eq!(run("mapped"), 0);
    run("sample = true");
    assert_eq!(run("mapped"), 1);
    run("sample = [0.5]");
    assert_eq!(run("mapped"), 0.5);
    run("sample = []");
    assert_eq!(run("mapped"), 0);
    run(r#"sample = "not-a-number""#);
    assert_eq!(run("Number.isNaN(mapped)"), true);
    run("sample = [1, 2]");
    assert_eq!(run("Number.isNaN(mapped)"), true);
    run("start = Date(0)");
    run("end = Date(8)");
    run("targetStart = Date(10)");
    run("targetEnd = Date(30)");
    run("sample = Date(2)");
    assert_eq!(run("mapped"), 15);
    run("sample = Date(6)");
    assert_eq!(run("mapped"), 25);
    run("start = []");
    run(r#"end = ["8"]"#);
    run("targetStart = []");
    run(r#"targetEnd = ["8"]"#);
    run(r#"sample = ["2"]"#);
    assert_eq!(run("mapped"), 2);
    run("sample = false");
    assert_eq!(run("mapped"), 0);
}

/// Nucleoid preserves remapping endpoints nonfinite inputs and omissions
#[rustfmt::skip]
#[test]
fn preserves_remapping_endpoints_nonfinite_inputs_and_omissions() {
    let mut run = runner();
    run("start = 0");
    run("end = 8");
    run("targetStart = 10");
    run("targetEnd = 30");
    run("sample = 2");
    run("mapped = Math.remap(start, end, targetStart, targetEnd, sample)");
    assert_eq!(run("mapped"), 15);
    run("sample = start");
    assert_eq!(run("mapped"), 10);
    run("sample = end");
    assert_eq!(run("mapped"), 30);
    run("sample = 16");
    assert_eq!(run("mapped"), 50);
    run("sample = -8");
    assert_eq!(run("mapped"), -10);
    run("start = 2");
    run("end = 2");
    run("sample = 2");
    assert_eq!(run("Number.isNaN(mapped)"), true);
    run("sample = 3");
    {
        let actual = run("mapped");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("sample = 1");
    {
        let actual = run("mapped");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("start = 0");
    run("end = Number.POSITIVE_INFINITY");
    run("sample = 8");
    assert_eq!(run("mapped"), 10);
    run("sample = Number.POSITIVE_INFINITY");
    assert_eq!(run("Number.isNaN(mapped)"), true);
    run("end = 8");
    run("sample = Number.NEGATIVE_INFINITY");
    {
        let actual = run("mapped");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("start = Number.NEGATIVE_INFINITY");
    run("end = Number.POSITIVE_INFINITY");
    run("sample = 0");
    assert_eq!(run("Number.isNaN(mapped)"), true);
    run("start = -Number.MAX_VALUE");
    run("end = Number.MAX_VALUE");
    run("sample = 0");
    assert_eq!(run("mapped"), 10);
    run("start = 0");
    run("end = 8");
    run("sample = Number.NaN");
    assert_eq!(run("Number.isNaN(mapped)"), true);
    run("targetStart = 4");
    run("targetEnd = 4");
    run("sample = 2");
    assert_eq!(run("mapped"), 4);
    run("sample = Number.POSITIVE_INFINITY");
    assert_eq!(run("Number.isNaN(mapped)"), true);
    run("targetStart = -Number.MAX_VALUE");
    run("targetEnd = Number.MAX_VALUE");
    run("sample = 2");
    {
        let actual = run("mapped");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("targetStart = 0");
    run("targetEnd = 1");
    assert_eq!(run("mapped"), 0.25);
    run("empty = Math.remap()");
    assert_eq!(run("Number.isNaN(empty)"), true);
    run("one = Math.remap(0)");
    assert_eq!(run("Number.isNaN(one)"), true);
    run("two = Math.remap(0, 8)");
    assert_eq!(run("Number.isNaN(two)"), true);
    run("three = Math.remap(0, 8, 0)");
    assert_eq!(run("Number.isNaN(three)"), true);
    run("four = Math.remap(0, 8, 0, 1)");
    assert_eq!(run("Number.isNaN(four)"), true);
    run("literal = Math.remap(false, true, 0, 1, null)");
    assert_eq!(run("literal"), 0);
    run("withExtra = Math.remap(0, 8, 10, 30, 2, 99)");
    assert_eq!(run("withExtra"), 15);
    run("ignored = Number.NaN");
    run("withIgnored = Math.remap(0, 8, 10, 30, 2, ignored)");
    assert_eq!(run("withIgnored"), 15);
    run(r#"ignored = "not-a-number""#);
    assert_eq!(run("withIgnored"), 15);
    run("ignored = null");
    assert_eq!(run("withIgnored"), serde_json::Value::Null);
    run("ignored = []");
    assert_eq!(run("withIgnored"), 15);
}

/// Nucleoid applies range remapping to live class rules
#[rustfmt::skip]
#[test]
fn applies_range_remapping_to_live_class_rules() {
    let mut run = runner();
    run("class Gauge");
    run("$Gauge.mapped = Math.remap($Gauge.start, $Gauge.end, $Gauge.targetStart, $Gauge.targetEnd, $Gauge.sample)");
    run("first = Gauge()");
    run("second = Gauge()");
    run("first.start = 0");
    run("first.end = 8");
    run("first.targetStart = 10");
    run("first.targetEnd = 30");
    run("first.sample = 2");
    run("second.start = 10");
    run("second.end = 2");
    run("second.targetStart = 30");
    run("second.targetEnd = 10");
    run("second.sample = 8");
    assert_eq!(run("first.mapped"), 15);
    assert_eq!(run("second.mapped"), 25);
    run("first.sample = 12");
    assert_eq!(run("first.mapped"), 40);
    assert_eq!(run("second.mapped"), 25);
    run("first.targetEnd = 50");
    assert_eq!(run("first.mapped"), 70);
    run("third = Gauge()");
    assert_eq!(run("third.mapped"), serde_json::Value::Null);
    run("third.start = 0");
    run("third.end = 4");
    run("third.targetStart = 0");
    run("third.targetEnd = 100");
    run("third.sample = 1");
    assert_eq!(run("third.mapped"), 25);
    run("delete first.sample");
    assert_eq!(run("first.mapped"), serde_json::Value::Null);
    run("first.sample = -4");
    assert_eq!(run("first.mapped"), -10);
    assert_eq!(run("Gauge.length"), 3);
}

/// Nucleoid preserves deferred range remapping and rollback
#[rustfmt::skip]
#[test]
fn preserves_deferred_range_remapping_and_rollback() {
    let (mut run, mut run_error) = crate::common::runners();
    run("class Sensor");
    run("sensor = Sensor()");
    run("start = sensor.lower");
    run("end = sensor.upper");
    run("targetStart = sensor.outputLower");
    run("targetEnd = sensor.outputUpper");
    run("sample = sensor.reading");
    run("mapped = Math.remap(start, end, targetStart, targetEnd, sample)");
    run("withDeferredExtra = Math.remap(start, end, targetStart, targetEnd, sample, sensor.optional)");
    assert_eq!(run("mapped"), serde_json::Value::Null);
    assert_eq!(run("withDeferredExtra"), serde_json::Value::Null);
    run("sensor.lower = 0");
    run("sensor.upper = 8");
    run("sensor.outputLower = 10");
    run("sensor.outputUpper = 30");
    run("sensor.reading = 2");
    assert_eq!(run("mapped"), 15);
    assert_eq!(run("withDeferredExtra"), serde_json::Value::Null);
    run("sensor.optional = 99");
    assert_eq!(run("withDeferredExtra"), 15);
    run("delete sensor.reading");
    assert_eq!(run("mapped"), serde_json::Value::Null);
    assert_eq!(run("withDeferredExtra"), serde_json::Value::Null);
    run("sensor.reading = 6");
    assert_eq!(run("mapped"), 25);
    assert_eq!(run("withDeferredExtra"), 25);
    assert_eq!(run_error(r#"sensor.reading = 4
throw "ROLLBACK""#), "ROLLBACK");
    assert_eq!(run("sensor.reading"), 6);
    assert_eq!(run("mapped"), 25);
    assert_eq!(run("withDeferredExtra"), 25);
    assert_eq!(run_error("sensor.reading = missing"), "ReferenceError: missing is not defined");
    assert_eq!(run("sensor.reading"), 6);
    run("delete sensor.optional");
    assert_eq!(run("withDeferredExtra"), serde_json::Value::Null);
    assert_eq!(run("mapped"), 25);
    run("sensor.optional = [1, 2]");
    assert_eq!(run("withDeferredExtra"), 25);
    run("sensor.reading = 2");
    assert_eq!(run("mapped"), 15);
    assert_eq!(run("withDeferredExtra"), 15);
    run(r#"mapped

# return: 15"#);
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
