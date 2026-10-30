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
    let mut run = runner();
    run(r#"try:
    # t is e plus 1
    t = e + 1
catch error:
    assert(error, ReferenceError("e is not defined"))"#);
}

/// Nucleoid throws an error inside a block
#[rustfmt::skip]
#[test]
fn throws_an_error_inside_a_block() {
    let mut run = runner();
    run(r#"# k is 99
k = 99

try:
    # if k is greater than or equal to 99, then throw "INVALID"
    if k >= 99:
        throw "INVALID"
catch error:
    assert(error, "INVALID")"#);
}

/// Nucleoid throws an error as a variable
#[rustfmt::skip]
#[test]
fn throws_an_error_as_a_variable() {
    let mut run = runner();
    run(r#"# length is 0.1
length = 0.1

try:
    # if length is less than 1, then throw length
    if length < 1:
        throw length
catch error:
    assert(error, 0.1)

try:
    # if length is less than 1.1, then throw 'length'
    if length < 1.1:
        throw 'length'
catch error:
    assert(error, "length")"#);
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
    def init name school:
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
    run(r#"def multiply first_factor second_factor:
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
    let mut run = runner();
    run(r#"try:
    # whether e is equal to 2.71828
    e == 2.71828
catch error:
    assert(error, ReferenceError("e is not defined"))"#);
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
    let mut run = runner();
    run(r#"# There is a Ticket type
class Ticket

try:
    # while in the block, ticket is a local Ticket,
    # and ticket's event's group is "ENTERTAINMENT"
    {
        ticket = Ticket()
        ticket.event.group = "ENTERTAINMENT"
    }
catch error:
    assert(error, ReferenceError("ticket.event is not defined"))"#);
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
    let mut run = runner();
    run(r#"# There is a Device type,
# which has a code as a string
class Device(code: str):
    this.code = code

# device1 is a Device whose code is "A0"
device1 = Device("A0")

# device2 is a Device whose code is "B1"
device2 = Device("B1")

try:
    # While in the block, device is a local variable that is the Device whose code is "A1",
    # and if there is no device, then throw "INVALID_DEVICE"
    {
        device = Device.find(d => d.code == "A1")
        if not device:
            throw "INVALID_DEVICE"
        return device
    }
catch error:
    assert(error, "INVALID_DEVICE")"#);
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
    let mut run = runner();
    run(r#"# There is a User type
class User

# If any user's password does not match /.{4,8}/, then throw 'INVALID_PASSWORD'
if not /.{4,8}/.test($User.password):
    throw 'INVALID_PASSWORD'

# user1 is a User
user1 = User()

assert(user1.password, null)

try:
    # user1's password is 'PAS'
    user1.password = 'PAS'
catch error:
    assert(error, "INVALID_PASSWORD")"#);
}

/// Nucleoid rejects defining a class declaration in a non-class block
#[rustfmt::skip]
#[test]
fn rejects_defining_a_class_declaration_in_a_non_class_block() {
    let mut run = runner();
    run(r#"# There is a Person type
class Person

# person1 is a Person whose weight is 90 and whose height is 1.8
person1 = Person()
person1.weight = 90
person1.height = 1.8

try:
    # while in the block, weight is a local variable that is person1's weight,
    # and height is a local variable that is person1's height,
    # and any person's bmi is weight divided by height squared
    {
        weight = person1.weight
        height = person1.height
        $Person.bmi = weight / (height * height)
    }
catch error:
    assert(error, SyntaxError("Cannot define class declaration in non-class block"))"#);
}

/// Nucleoid detects a circular dependency
#[rustfmt::skip]
#[test]
fn detects_a_circular_dependency() {
    let mut run = runner();
    run(r#"# number1 is 10
number1 = 10

# number2 is number1 times 10
number2 = number1 * 10

assert(number2, 100)

try:
    # number1 is number2 times 10
    number1 = number2 * 10
catch error:
    assert(error, TypeError("Circular Dependency"))"#);
}

/// Nucleoid rolls back a variable if an exception is thrown
#[rustfmt::skip]
#[test]
fn rolls_back_a_variable_if_an_exception_is_thrown() {
    let mut run = runner();
    run(r#"# a is 5
a = 5

# if a is greater than 5, then throw 'INVALID_VALUE'
if a > 5:
    throw 'INVALID_VALUE'

try:
    # a is 6
    a = 6
catch error:
    assert(error, "INVALID_VALUE")

assert(a, 5)"#);
}

/// Nucleoid rolls back a property if an exception is thrown
#[rustfmt::skip]
#[test]
fn rolls_back_a_property_if_an_exception_is_thrown() {
    let mut run = runner();
    run(r#"# There is an Item type
class Item

# If any item's sku is 'A', then throw 'INVALID_SKU'
if $Item.sku == 'A':
    throw 'INVALID_SKU'

# item1 is an Item
item1 = Item()

try:
    # item1's sku is 'A'
    item1.sku = 'A'
catch error:
    assert(error, "INVALID_SKU")

assert(item1.sku, null)"#);
}

/// Nucleoid rolls back an instance if an exception is thrown
#[rustfmt::skip]
#[test]
fn rolls_back_an_instance_if_an_exception_is_thrown() {
    let mut run = runner();
    run(r#"# There is a User type,
# which has a first as a string
# and a last as a string
class User(first: str, last: str):
    this.first = first
    this.last = last

# If any user's first is shorter than 3 characters, then throw 'INVALID_USER'
if $User.first.length < 3:
    throw 'INVALID_USER'

try:
    # user1 is a User whose first is 'F' and whose last is 'L'
    user1 = User('F', 'L')
catch error:
    assert(error, "INVALID_USER")

assert(User.length, 0)

try:
    # user1
    user1
catch error:
    assert(error, ReferenceError("user1 is not defined"))"#);
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
    let mut run = runner();
    run(r#"# t is 1
t = 1

# q is t plus 1
q = t + 1

assert(q, 2)

# q is deleted
delete q

# t is 2
t = 2

try:
    # q
    q
catch error:
    assert(error, ReferenceError("q is not defined"))"#);
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
    let mut run = runner();
    run(r#"try:
    # chart1 is a Chart
    chart1 = Chart()
catch error:
    assert(error, ReferenceError("Chart is not defined"))

# There is a Chart type
class Chart

# chart1 is a Chart
chart1 = Chart()

try:
    # chart1's plot is a Plot
    chart1.plot = Plot()
catch error:
    assert(error, ReferenceError("Plot is not defined"))

try:
    # any chart's plot is a Plot
    $Chart.plot = Plot()
catch error:
    assert(error, ReferenceError("Plot is not defined"))"#);
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
    let mut run = runner();
    run(r#"# There is a Channel type
class Channel

# channel1 is a Channel
channel1 = Channel()

try:
    # channel1's frequency's type is "ANGULAR"
    channel1.frequency.type = "ANGULAR"
catch error:
    assert(error, ReferenceError("channel1.frequency is not defined"))"#);
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
    let mut run = runner();
    run(r#"# There is a Worker type
class Worker

# There is a Schedule type
class Schedule

# worker1 is a Worker
worker1 = Worker()

try:
    # worker1's duty's schedule is a Schedule
    worker1.duty.schedule = Schedule()
catch error:
    assert(error, ReferenceError("worker1.duty is not defined"))"#);
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
    let mut run = runner();
    run(r#"# There is a Schedule type
class Schedule

# There is a Place type
class Place

# value is a Schedule
value = Schedule()

assert(value, { "id": "value" })

try:
    # value's value is a Place
    value.value = Place()
catch error:
    assert(error, TypeError("Cannot use 'value' as a property"))"#);
}

/// Nucleoid rejects value as a property name in a value assignment
#[rustfmt::skip]
#[test]
fn rejects_value_as_a_property_name_in_a_value_assignment() {
    let mut run = runner();
    run(r#"# There is a Value type
class Value

# value is a Value
value = Value()

assert(value, { "id": "value" })

try:
    # value's value is 2147483647
    value.value = 2147483647
catch error:
    assert(error, TypeError("Cannot use 'value' as a property"))"#);
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
    let mut run = runner();
    run(r#"# There is a Question type
class Question

# question1 is a Question whose text is "How was the service?"
question1 = Question()
question1.text = "How was the service?"

# if question1's text is not question1's text's value, then throw "QUESTION_ARCHIVED"
if question1.text != question1.text.value:
    throw "QUESTION_ARCHIVED"

assert(question1.text, "How was the service?")

try:
    # question1's text is "How would you rate us?"
    question1.text = "How would you rate us?"
catch error:
    assert(error, "QUESTION_ARCHIVED")"#);
}

/// Nucleoid rejects value of a property if the property is not defined
#[rustfmt::skip]
#[test]
fn rejects_value_of_a_property_if_the_property_is_not_defined() {
    let mut run = runner();
    run(r#"# There is a Travel type
class Travel

# travel1 is a Travel whose speed is 65
travel1 = Travel()
travel1.speed = 65

# travel1's duration is travel1's distance divided by travel1's speed
travel1.duration = travel1.distance / travel1.speed

assert(travel1.duration, null)

try:
    # travel1's time is travel1's distance's value divided by travel1's speed
    travel1.time = travel1.distance.value / travel1.speed
catch error:
    assert(error, ReferenceError("travel1.distance is not defined"))"#);
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
    let mut run = runner();
    run(r#"# There is an Alarm type
class Alarm

try:
    # while in the block, value is a local Alarm,
    # and value's value is "22:00"
    {
        value = Alarm()
        value.value = "22:00"
    }
catch error:
    assert(error, TypeError("Cannot use 'value' as a property"))"#);
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
    let mut run = runner();
    run(r#"# There is a Circle type
class Circle

# circle1 is a Circle
circle1 = Circle()

# circle1 is deleted
delete circle1

assert(Circle["circle1"], null)
assert(Circle.find(circle => circle.id == "circle1"), null)

try:
    # circle1
    circle1
catch error:
    assert(error, ReferenceError("circle1 is not defined"))"#);
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
    let mut run = runner();
    run(r#"# There is a Channel type
class Channel

# channel1 is a Channel
channel1 = Channel()

# channel1's frequency is 440
channel1.frequency = 440

try:
    # channel1 is deleted
    delete channel1
catch error:
    assert(error, TypeError("Cannot delete object 'channel1'"))

assert(channel1.frequency, 440)

# channel1's frequency is deleted
delete channel1.frequency

# channel1 is deleted
delete channel1

assert(Channel["channel1"], null)"#);
}

/// Nucleoid rejects deleting an instance if it has an object as a property
#[rustfmt::skip]
#[test]
fn rejects_deleting_an_instance_if_it_has_an_object_as_a_property() {
    let mut run = runner();
    run(r#"# There is a Shape type
class Shape

# There is a Type type
class Type

# shape1 is a Shape
shape1 = Shape()

# shape1's type is a Type
shape1.type = Type()

try:
    # shape1 is deleted
    delete shape1
catch error:
    assert(error, TypeError("Cannot delete object 'shape1'"))

# shape1's type is deleted
delete shape1.type

# shape1 is deleted
delete shape1

assert(Shape["shape1"], null)"#);
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
    let mut run = runner();
    run(r#"try:
    # a is declared as a number but not defined
    a: int
catch error:
    assert(error, ReferenceError("Missing definition"))"#);
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
    let mut run = runner();
    run(r#"# date1 is the current date
date1 = Date()

# date2 is a date whose time is date1's time
date2 = Date(date1.getTime())

assert(date1.getTime() == date2.getTime(), true)

# date3 is the parsed date of "04 Dec 1995 00:12:00 GMT"
date3 = Date.parse("04 Dec 1995 00:12:00 GMT")

assert(date3, 818035920000)

try:
    # date4 is the wrong date
    date4 = Date.wrong()
catch error:
    assert(error, TypeError("Date.wrong is not a function"))"#);
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
    run(r#"# samples is a list of 38.2, 39.1, 38.8 and 39
samples = [38.2, 39.1, 38.8, 39]

# ratio is 2.1
ratio = 2.1

# element is 38.5
element = 38.5

# The element parameter shadows the outer element in each function
assert(samples.find(function(element) { result = element * ratio; return result == 81.48 }), 38.8)
assert(samples.find(element => { result = element * ratio; return result == 81.48 }), 38.8)
assert(samples.find(element => element == 38.8), 38.8)
assert(samples.find(element => (element == 38.8)), 38.8)"#);
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
    let mut run = runner();
    run(r#"# name is "AbCDE"
name = "AbCDE"

# pointer is 0
pointer = 0

# if the character of name at pointer is not an uppercase letter,
# then throw "INVALID_FIRST_CHARACTER"
if not /[A-Z]/.test(name.charAt(pointer)):
    throw "INVALID_FIRST_CHARACTER"

try:
    # name is "bbCDE"
    name = "bbCDE"
catch error:
    assert(error, "INVALID_FIRST_CHARACTER")

# name is "CbCDE"
name = "CbCDE"

try:
    # pointer is 1
    pointer = 1
catch error:
    assert(error, "INVALID_FIRST_CHARACTER")"#);
}

/// Nucleoid supports a property of chained functions in an expression
#[rustfmt::skip]
#[test]
fn supports_a_property_of_chained_functions_in_an_expression() {
    let mut run = runner();
    run(r#"# There is a User type
class User

# There is a Registration type
class Registration

# user1 is a User
user1 = User()

# registration1 is a Registration whose user is user1
registration1 = Registration()
registration1.user = user1

# registration2 is a Registration whose user is user1
registration2 = Registration()
registration2.user = user1

try:
    # if the number of registrations whose user is any user is more than one,
    # then throw "USER_ALREADY_REGISTERED"
    if Registration.filter(r => r.user == $User).length > 1:
        throw "USER_ALREADY_REGISTERED"
catch error:
    assert(error, "USER_ALREADY_REGISTERED")"#);
}

/// Nucleoid throws an error as a string
#[rustfmt::skip]
#[test]
fn throws_an_error_as_a_string() {
    let mut run = runner();
    run(r#"try:
    # throw 'INVALID'
    throw 'INVALID'
catch error:
    assert(error, "INVALID")

try:
    # throw "INVALID"
    throw "INVALID"
catch error:
    assert(error, "INVALID")"#);
}

/// Nucleoid throws an error as an integer
#[rustfmt::skip]
#[test]
fn throws_an_error_as_an_integer() {
    let mut run = runner();
    run(r#"try:
    # throw 123
    throw 123
catch error:
    assert(error, 123)"#);
}

/// Nucleoid throws a reference error if the thrown value is not defined
#[rustfmt::skip]
#[test]
fn throws_a_reference_error_if_the_thrown_value_is_not_defined() {
    let mut run = runner();
    run(r#"try:
    # throw abc
    throw abc
catch error:
    assert(error, ReferenceError("abc is not defined"))"#);
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
    let mut run = runner();
    run(r#"# There is a Phone type
class Phone

try:
    # any phone's line's wired is true
    $Phone.line.wired = true
catch error:
    assert(error, ReferenceError("Phone.line is not defined"))"#);
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
    let mut run = runner();
    run(r#"try:
    # explain nothing
    why nothing
catch error:
    assert(error, ReferenceError("nothing is not defined"))"#);
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
    let mut run = runner();
    run(r#"class Parent

class Child: Parent
    pass

parent = Parent()
parent.score = 0
child = Child()
child.score = 2

try:
    if $Parent.score > 1:
        throw "LIMIT"
catch error:
    assert(error, "LIMIT")

child.score = 3
parent.score = 3
another = Child()
another.score = 4

assert(child.score, 3)
assert(parent.score, 3)
assert(another.score, 4)"#);
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
    let mut run = runner();
    run(r#"try:
    class Loop: Loop
        pass
catch error:
    assert(error, TypeError("Circular Inheritance"))

assert(Class.length, 0)

class Loop

loop = Loop()

assert(loop.id, "loop")"#);
}

/// Nucleoid rejects an inheritance cycle introduced by redeclaration
#[rustfmt::skip]
#[test]
fn rejects_an_inheritance_cycle_introduced_by_redeclaration() {
    let mut run = runner();
    run(r#"class Root

$Root.flag = true

class Child: Root
    pass

try:
    class Root: Child
        pass
catch error:
    assert(error, TypeError("Circular Inheritance"))

child = Child()

assert(child.flag, true)
assert(Class.length, 2)"#);
}

/// Nucleoid rejects class dependency cycles inside compound expressions
#[rustfmt::skip]
#[test]
fn rejects_class_dependency_cycles_inside_compound_expressions() {
    let mut run = runner();
    run(r#"class Item

$Item.list = [$Item.source]

try:
    $Item.source = $Item.list
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.record = { "source": $Item.source }

try:
    $Item.source = $Item.record
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.label = `${$Item.source}`

try:
    $Item.source = $Item.label
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.part = "abcd"[$Item.source:]

try:
    $Item.source = $Item.part.length
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.head = "abcd"[:$Item.source]

try:
    $Item.source = $Item.head.length
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.copy = $Item.list[:]

try:
    $Item.source = $Item.copy[0]
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.indexed = [$Item["source"]]

try:
    $Item.source = $Item.indexed
catch error:
    assert(error, TypeError("Circular Dependency"))

$Item.source = 1
item = Item()

assert(item.list, [1])
assert(item.label, "1")
assert(item.part, "bcd")

$Item.snapshot = [$Item.source.value]
$Item.source = $Item.snapshot[0] + 1

assert(item.source, 2)
assert(item.snapshot, [1])
assert(item.list, [2])"#);
}

/// Nucleoid checks both slice bounds when declaring a class rule
#[rustfmt::skip]
#[test]
fn checks_both_slice_bounds_when_declaring_a_class_rule() {
    let mut run = runner();
    run(r#"class Item

try:
    $Item.part = "abcd"[missing:]
catch error:
    assert(error, ReferenceError("missing is not defined"))

try:
    $Item.part = "abcd"[:missing]
catch error:
    assert(error, ReferenceError("missing is not defined"))

start = 1
end = 3
$Item.part = "abcd"[start:end]
item = Item()

assert(item.part, "bc")

end = 4

assert(item.part, "bcd")"#);
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
    let mut run = runner();
    run(r#"count = Class.length
doubled = count * 2

class First

assert(count, 1)
assert(doubled, 2)

class Second

assert(count, 2)
assert(doubled, 4)

class First

assert(count, 2)

try:
    class Temporary
    throw "ABORT"
catch error:
    assert(error, "ABORT")

assert(Class.length, 2)
assert(count, 2)
assert(doubled, 4)

class Third

assert(count, 3)
assert(doubled, 6)

if count > 3:
    throw "TOO_MANY_TYPES"

try:
    class Fourth
catch error:
    assert(error, "TOO_MANY_TYPES")

assert(Class.length, 3)
assert(count, 3)
assert(doubled, 6)"#);
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
    throw "EXPECTED"

def recover local value:
    try:
        local = value + fail()
    catch error:
        if error != "EXPECTED":
            throw error
    return local

input = null
result = input + recover(1, 0)

assert(result, null)

input = 2

assert(result, 3)

result = 5 + recover(1, null)

assert(result, 6)"#);
}

/// Nucleoid restores enclosing null tracking after failed property assignments
#[rustfmt::skip]
#[test]
fn restores_enclosing_null_tracking_after_failed_property_assignments() {
    let mut run = runner();
    run(r#"class Item

item = Item()
item.amount = 1

def fail():
    throw "EXPECTED"

def recover value:
    try:
        item.amount = value + fail()
    catch error:
        if error != "EXPECTED":
            throw error
    return item.amount

input = null
result = input + recover(0)

assert(result, null)
assert(item.amount, 1)

input = 2

assert(result, 3)

result = 5 + recover(null)

assert(result, 6)
assert(item.amount, 1)"#);
}

/// Nucleoid repeats strings with catchable allocation errors
#[rustfmt::skip]
#[test]
fn repeats_strings_with_catchable_allocation_errors() {
    let mut run = runner();
    run(r#"assert("ab".repeat(2), "abab")
assert("ab".repeat(2.9), "abab")
assert("ab".repeat(0), "")
assert("ab".repeat(-1), "")
assert("ab".repeat(), "")
assert("ab".repeat(null), "")
assert("ab".repeat(Number.NaN), "")
assert("ab".repeat(Number.NEGATIVE_INFINITY), "")
assert("".repeat(Number.MAX_VALUE), "")
assert("".repeat(Number.POSITIVE_INFINITY), "")

symbol = String.fromCharCode(128512)

assert(symbol.repeat(3), symbol + symbol + symbol)

try:
    "ab".repeat(Number.MAX_VALUE)
catch error:
    assert(error, TypeError("Repeated string is too large"))

try:
    "a".repeat(Number.POSITIVE_INFINITY)
catch error:
    assert(error, TypeError("Repeated string is too large"))

count = 2
repeated = "ab".repeat(count)

try:
    count = Number.POSITIVE_INFINITY
catch error:
    assert(error, TypeError("Repeated string is too large"))

assert(count, 2)
assert(repeated, "abab")

count = 3

assert(repeated, "ababab")"#);
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
    let mut run = runner();
    run(r#"first = 0
second = 0
total = first + second

if first > 0 and first < 4:
    second = first + 1

if second > 0 and second < 4:
    first = second + 1

first = 1

assert(first, 3)
assert(second, 4)
assert(total, 7)

if second == 3:
    throw "INVALID_SECOND"

try:
    first = 2
catch error:
    assert(error, "INVALID_SECOND")

assert(first, 3)
assert(second, 4)
assert(total, 7)

# The failed propagation leaves the runtime ready for another update
if second == 3:
    pass

first = 2

assert(first, 4)
assert(second, 3)
assert(total, 7)"#);
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
    run(r#"trigger = 0
source = 1
observations = 0

if source > 0:
    observations += 1

if trigger > 0:
    try:
        source = 2
        throw "ABORT"
    catch error:
        if error != "ABORT":
            throw error

# This dependent is already queued when the failing body starts
remaining = trigger + 10

trigger = 1

assert(source, 1)
assert(observations, 1)
assert(remaining, 11)

source = 3

assert(observations, 2)"#);
}

/// Nucleoid preserves earlier queued work across nested caught failures
#[rustfmt::skip]
#[test]
fn preserves_earlier_queued_work_across_nested_caught_failures() {
    let mut run = runner();
    run(r#"trigger = 0
source = 0
observations = 0

if source > 0:
    observations += 1

if trigger > 0:
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
            throw outer

trigger = 1

assert(source, 1)
assert(observations, 1)"#);
}

/// Nucleoid restores deletion tracking before entering a catch body
#[rustfmt::skip]
#[test]
fn restores_deletion_tracking_before_entering_a_catch_body() {
    let mut run = runner();
    run(r#"trigger = 0
missing = null

if trigger > 0:
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
            missing = absent

trigger = 1

assert(missing, ReferenceError("temporary is not defined"))"#);
}

/// Nucleoid ends temporary undefined reads when a deletion cascade finishes
#[rustfmt::skip]
#[test]
fn ends_temporary_undefined_reads_when_a_deletion_cascade_finishes() {
    let mut run = runner();
    run(r#"trigger = 0
source = 1
dependent = source + 1

if trigger > 0:
    delete source

def read_after_update():
    trigger = 1
    try:
        source
    catch error:
        return error
    return "NO_ERROR"

assert(read_after_update(), ReferenceError("source is not defined"))
assert(dependent, null)

source = 3

assert(dependent, 4)"#);
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
    run(r#"callback = null
input = 3
offset = 0
result = callback(input) + offset

assert(result, null)

callback = number => number * 2

assert(result, 6)

callback = null

assert(result, null)

callback = number => number * 3

assert(result, 9)

delete callback

assert(result, null)

offset = 1
input = 4

assert(result, null)

missing = null

try:
    callback(3)
catch error:
    missing = error

assert(missing, ReferenceError("callback is not defined"))

trigger = 0

if trigger > 0:
    callback(3)

try:
    trigger = 1
catch error:
    missing = error

assert(missing, ReferenceError("callback is not defined"))
assert(trigger, 0)

callback = number => number * 4

assert(result, 17)

input = 5

assert(result, 21)"#);
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
    run(r#"def operation number => 99

def apply operation number => operation(number)

callback = null
result = apply(callback, 3)

assert(result, null)

callback = number => number * 2

assert(result, 6)

callback = number => number * 3

assert(result, 9)

failure = null

try:
    apply(7, 3)
catch error:
    failure = error

assert(failure, TypeError("operation is not a function"))
assert(operation(3), 99)"#);
}

/// Nucleoid rolls back non callable replacements used by existing callers
#[rustfmt::skip]
#[test]
fn rolls_back_non_callable_replacements_used_by_existing_callers() {
    let mut run = runner();
    run(r#"callback = number => number * 2
result = callback(3)
failure = null

try:
    callback = 7
catch error:
    failure = error

assert(failure, TypeError("callback is not a function"))
assert(result, 6)
assert(callback(4), 8)

class Handler

handler = Handler()
handler.operation = number => number + 1
propertyResult = handler.operation(3)

try:
    handler.operation = 7
catch error:
    failure = error

assert(failure, TypeError("handler.operation is not a function"))
assert(propertyResult, 4)
assert(handler.operation(4), 5)

handler.operation = number => number + 2

assert(propertyResult, 5)"#);
}

/// Nucleoid rolls back failed redefinitions of named callbacks
#[rustfmt::skip]
#[test]
fn rolls_back_failed_redefinitions_of_named_callbacks() {
    let mut run = runner();
    run(r#"def transform number => number * 2

numbers = [1, 2]
mapped = numbers.map(transform)
failure = null

try:
    def transform number:
        if number == 2:
            throw "REJECTED_CALLBACK"
        return number * 3
catch error:
    failure = error

assert(failure, "REJECTED_CALLBACK")
assert(mapped, [2, 4])
assert(transform(3), 6)

def transform number => number * 4

assert(mapped, [4, 8])"#);
}

/// Nucleoid replaces functions and callable variables under the same name
#[rustfmt::skip]
#[test]
fn replaces_functions_and_callable_variables_under_the_same_name() {
    let mut run = runner();
    run(r#"def callback number => number * 2

forwarded = callback
direct = callback(3)
indirect = forwarded(3)

callback = number => number * 3

assert(direct, 9)
assert(indirect, 9)

failure = null

try:
    def callback number:
        throw "REJECTED_BINDING"
catch error:
    failure = error

assert(failure, "REJECTED_BINDING")
assert(direct, 9)
assert(indirect, 9)
assert(callback(4), 12)

def callback number => number * 4

assert(direct, 12)
assert(indirect, 12)

try:
    callback = 7
catch error:
    failure = error

assert(failure, TypeError("callback is not a function"))
assert(direct, 12)
assert(indirect, 12)
assert(callback(4), 16)

callback = null

assert(direct, null)
assert(indirect, null)

def callback number => number * 5

assert(direct, 15)
assert(indirect, 15)

callback = number => number * 6

assert(direct, 18)
assert(indirect, 18)

delete callback

assert(direct, null)
assert(indirect, null)

try:
    callback(3)
catch error:
    failure = error

assert(failure, ReferenceError("callback is not defined"))

def callback number => number * 7

assert(direct, 21)
assert(indirect, 21)"#);
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
    let mut run = runner();
    run(r#"a = 1
alternative = 1
b = a + 1
trace = why b
exposure = affects a

try:
    b = alternative + 1
    throw "REJECTED_REWRITE"
catch error:
    assert(error, "REJECTED_REWRITE")

assert(trace[0].from, ["a"])
assert(exposure.map(step => step.node), ["b"])

alternative = 4

assert(trace[0].holds, 2)

a = 3

assert(trace[0].holds, 4)

b = alternative + 1

assert(trace[0].from, ["alternative"])
assert(exposure, [])"#);
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
    let mut run = runner();
    run(r#"source = 1
limit = 1

if (affects source).length > limit:
    throw "TOO_MANY_DEPENDENTS"

first = source + 1

try:
    second = source + 2
catch error:
    assert(error, "TOO_MANY_DEPENDENTS")

assert((affects source).map(step => step.node), ["first"])

limit = 2
second = source + 2

assert((affects source).map(step => step.node), ["first", "second"])"#);
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
    run(r#"trigger = 0
inventory = model |> why

if trigger > 0:
    try:
        class Temporary
        throw "ABORT"
    catch error:
        if error != "ABORT":
            throw error

remaining = trigger + 10
trigger = 1

assert(inventory.some(step => step.node == "$Temporary"), false)
assert(inventory.find(step => step.node == "trigger").holds, 1)
assert(inventory.find(step => step.node == "remaining").holds, 11)"#);
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
    let mut run = runner();
    run(r#"try:
    this
catch error:
    assert(error, ReferenceError("this is not defined"))

try:
    self
catch error:
    assert(error, ReferenceError("this is not defined"))

try:
    this = 1
catch error:
    assert(error, TypeError("Cannot assign to 'this'"))

try:
    self = 1
catch error:
    assert(error, TypeError("Cannot assign to 'this'"))"#);
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
    let mut run = runner();
    run(r#"class Meter(reading: int):
    this.reading = reading
    def adjust(delta: int):
        delta *= 2
        this.reading += delta
        return this.reading
meter1 = Meter(10)
meter1.reading += 2
assert(meter1.reading, 12)
assert(meter1.adjust(3), 18)
assert(meter1.reading, 18)
class Scale
scale1 = Scale()
$Scale.factor = 2
try:
    $Scale.factor *= 3
catch error:
    assert(error, TypeError("Circular Dependency"))
assert(scale1.factor, 2)"#);
}

/// Nucleoid rolls back failed compound assignments
#[rustfmt::skip]
#[test]
fn rolls_back_failed_compound_assignments() {
    let mut run = runner();
    run(r#"balance = 10
step = 2
if balance > 15:
    throw "LIMIT"
try:
    balance += step * 4
catch error:
    assert(error, "LIMIT")
assert(balance, 10)
step = 3
assert(balance, 10)
balance += step
assert(balance, 13)
first = 2
second = first * 2
try:
    first += second
catch error:
    assert(error, TypeError("Circular Dependency"))
assert(first, 2)
assert(second, 4)
try:
    missing += 1
catch error:
    assert(error, ReferenceError("missing is not defined"))"#);
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
    let mut run = runner();
    run(r#"class Limited
count = Limited.length
if count > 1:
    throw "LIMIT"
limited1 = Limited()
try:
    limited2 = Limited()
catch error:
    assert(error, "LIMIT")
assert(Limited.length, 1)
assert(count, 1)
assert(limited1, {id: "limited1"})
try:
    limited2
catch error:
    assert(error, ReferenceError("limited2 is not defined"))"#);
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
    let mut run = runner();
    run(r#"class Parent:
    def init():
        this.amount = 3

class Child: Parent
    def init() => super
    def reset() => super

child = Child()

assert(child.amount, 3)
child.amount = 9
assert(child.reset(), null)
assert(child.amount, 3)

class Root:
    def init():
        this.result = super

root = Root()

assert(root.result, null)

try:
    super
catch error:
    assert(error, ReferenceError("super is not defined"))"#);
}

/// Nucleoid rolls back a failing parent constructor called with bare super
#[rustfmt::skip]
#[test]
fn rolls_back_a_failing_parent_constructor_called_with_bare_super() {
    let mut run = runner();
    run(r#"kept = { amount: 5 }

class Parent:
    def init():
        kept.amount = 9
        this.amount = 7
        throw "BLOCKED"

class Child: Parent
    def init():
        super

try:
    failed = Child()
catch error:
    assert(error, "BLOCKED")

assert(kept.amount, 5)
assert(Parent.length, 0)
assert(Child.length, 0)

try:
    failed
catch error:
    assert(error, ReferenceError("failed is not defined"))"#);
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
    let mut run = runner();
    run(r#"amount = 2
live = { amount }
frozen = { amount: amount.value }

assert(live.amount, 2)
assert(frozen.amount, 2)

amount = 5

assert(live.amount, 5)
assert(frozen.amount, 2)

try:
    pending = { missing }
catch error:
    assert(error, ReferenceError("missing is not defined"))

missing = 8
pending = { missing }

assert(pending.missing, 8)"#);
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
    let mut run = runner();
    run(r#"amount = 3
alternate = 7
record = { amount }
derived = record.amount * 2

try:
    record = { alternate }
    assert(record.alternate, 7)
    throw "ABORT"
catch error:
    assert(error, "ABORT")

assert(record.amount, 3)
assert(derived, 6)

alternate = 9

assert(record.amount, 3)

amount = 5

assert(record.amount, 5)
assert(derived, 10)"#);
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
    let mut run = runner();
    run(r#"source = 3

def reading amount => source + amount

total = reading(2)

assert(total, 5)

try:
    def reading amount => missing + amount
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(total, 5)

source = 4

assert(total, 6)"#);
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
    let mut run = runner();
    run(r#"source = 3

def reading amount =>
    source + amount

total = reading(2)

assert(total, 5)

try:
    def reading amount =>
        missing + amount
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(total, 5)

source = 4

assert(total, 6)"#);
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
    let mut run = runner();
    run(r#"source = 3

def reading amount: int => source + amount

total = reading(2)

try:
    def reading amount: int => missing + amount
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(total, 5)

source = 4

assert(total, 6)
assert(reading(3), 7)"#);
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
    let mut run = runner();
    run(r#"class Sensor
sensor = Sensor()

observed = Math.clamp(sensor.amount, 0, 10)

assert(observed, null)

sensor.amount = 12

assert(observed, 10)

delete sensor.amount

assert(observed, null)

sensor.amount = 3

assert(observed, 3)

try:
    observed = Math.clamp(missing, 0, 10)
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(observed, 3)

sensor.amount = 6

assert(observed, 6)"#);
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
    let mut run = runner();
    run(r#"class Sensor
sensor = Sensor()
end = 10
weight = 0.5

position = Math.lerp(sensor.start, end, weight)

assert(position, null)

sensor.start = 2

assert(position, 6)

delete sensor.start

assert(position, null)

sensor.start = 4

assert(position, 7)

try:
    position = Math.lerp(missing, end, weight)
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(position, 7)

end = 20

assert(position, 12)"#);
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
    let mut run = runner();
    run(r#"class Sensor
sensor = Sensor()

bounded = Math.saturate(sensor.reading)

assert(bounded, null)

sensor.reading = -2

assert(bounded, 0)

delete sensor.reading

assert(bounded, null)

sensor.reading = 0.75

assert(bounded, 0.75)

try:
    bounded = Math.saturate(missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(bounded, 0.75)

sensor.reading = 2

assert(bounded, 1)"#);
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
    let mut run = runner();
    run(r#"source = 3

def reading amount: int =>
    source + amount

total = reading(2)

assert(total, 5)

try:
    def reading amount: int =>
        missing + amount
catch error:
    assert(error, ReferenceError("missing is not defined"))

assert(total, 5)

source = 4

assert(total, 6)
assert(reading(3), 7)"#);
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
    run(r#"enabled = false
if enabled:
    throw "BLOCKED"
caught = null
try:
    enabled ||= true
catch error:
    caught = error
assert(caught, "BLOCKED")
assert(enabled, false)
ready = true
if !ready:
    throw "REQUIRED"
caught = null
try:
    ready &&= false
catch error:
    caught = error
assert(caught, "REQUIRED")
assert(ready, true)"#);
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
    run(r#"def add left right:
    total = left + right
    return total"#);
    run(r#"def scaled left right factor =>
    (left + right) * factor"#);
    run(r#"def largest left right:
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
    def init left right:
        this.amount = left + right
    def reading offset scale =>
        self.amount * scale + offset"#);
    run(r#"class Child: Parent
    def init left right:
        super(left, right)
    def reading offset scale =>
        self.amount * scale + offset
    def increase delta extra:
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
    run(r#"def add left right:
    left += right
    return left"#);
    run(r#"def noReturn left right:
    left + right"#);
    run(r#"def bareReturn left right:
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
    let mut run = runner();
    run(r#"source = 3
def reading left right =>
    source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading left right =>
        missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)
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
    let mut run = runner();
    run(r#"left = 10
right = 20
def pair left: int, right: int => [left, right]
def invoke operation: Callback, amount: int => operation(amount)
def increment(amount: int) => amount + 1
def operation(amount) => amount + 100
def outer(initial: int):
    def combine left: int, right, scale: int => (left + right) * scale
    return combine(initial, 2, 3)
assert(pair(3, 4), [3, 4])
assert(pair(3), [3, null])
assert(pair(), [null, null])
assert(pair(3, 4, 5), [3, 4])
assert(pair("three", true), ["three", true])
assert(invoke(increment, 4), 5)
assert(invoke(null, 3), null)
try:
    invoke(7, 3)
catch error:
    assert(error, TypeError("operation is not a function"))
assert(outer(4), 18)
assert(left, 10)
assert(right, 20)"#);
}

/// Nucleoid rolls back failed bare typed multiple parameter arrow redefinitions
#[rustfmt::skip]
#[test]
fn rolls_back_failed_bare_typed_multiple_parameter_arrow_redefinitions() {
    let mut run = runner();
    run(r#"source = 3
def reading left: int, right: int => source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading left: int, right: int => missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)
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
    let mut run = runner();
    run(r#"source = 3
def reading left: int, right: int =>
    source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading left: int, right =>
        missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)
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
    let mut run = runner();
    run(r#"class Entry
first = Entry()
first.amount = 2
second = Entry()
second.amount = 5
values = [first, second]
saved = 7
try:
    for values:
        saved += it.amount
        it.amount += 1
        if it.amount > 5:
            throw "stop"
catch error:
    assert(error, "stop")
assert(first.amount, 2)
assert(second.amount, 5)
assert(saved, 7)
broken = null
try:
    for broken:
        first.amount = 99
catch error:
    assert(error, TypeError("broken is not iterable"))
assert(first.amount, 2)
for values:
    it.score = it.amount * 2
assert(first.score, 4)
assert(second.score, 10)
first.score + second.score
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
    let mut run = runner();
    run(r#"source = 3
def reading left: int, right: int =>
    source + left + right
total = reading(2, 4)
assert(total, 9)
try:
    def reading left: int, right =>
        missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(total, 9)
source = 4
assert(total, 10)
assert(reading(8, 9), 21)
reading(8, 9)
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
    let mut run = runner();
    run(r#"class Sensor
sensor = Sensor()
lower = 0
upper = 8
value = sensor.reading
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smoothstep(lower, upper, value)
assert(value, null)
assert(unit, null)
assert(eased, null)
sensor.reading = 2
assert(unit, 0.25)
assert(eased, 0.15625)
delete sensor.reading
assert(value, null)
assert(eased, null)
sensor.reading = 4
assert(unit, 0.5)
assert(eased, 0.5)
try:
    sensor.reading = 6
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 4)
assert(unit, 0.5)
assert(eased, 0.5)
try:
    lower = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(lower, 0)
sensor.reading = 2
assert(unit, 0.25)
assert(eased, 0.15625)
eased
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
    let mut run = runner();
    run(r#"marker = 7
try:
    if false:
        marker = 8
    else
        marker = 9
        throw "ABORT"
catch error:
    assert(error, "ABORT")
assert(marker, 7)
gate = false
if gate:
    result = 1
else
    result = 2
assert(result, 2)
gate = true
assert(result, 1)"#);
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
    let mut run = runner();
    run(r#"class Sensor
sensor = Sensor()
lower = 0
upper = 8
value = sensor.reading
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smootherstep(lower, upper, value)
assert(value, null)
assert(unit, null)
assert(eased, null)
sensor.reading = 2
assert(unit, 0.25)
assert(eased, 0.103515625)
delete sensor.reading
assert(value, null)
assert(eased, null)
sensor.reading = 4
assert(unit, 0.5)
assert(eased, 0.5)
try:
    sensor.reading = 6
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 4)
assert(unit, 0.5)
assert(eased, 0.5)
try:
    lower = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(lower, 0)
sensor.reading = 2
assert(unit, 0.25)
assert(eased, 0.103515625)
eased
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
    let mut run = runner();
    run(r#"class Sensor
sensor = Sensor()
value = sensor.reading
whole = Math.floor(value)
fraction = Math.fract(value)
withDeferredExtra = Math.fract(value, sensor.optional)
assert(value, null)
assert(whole, null)
assert(fraction, null)
assert(withDeferredExtra, null)
sensor.reading = 2.25
assert(whole, 2)
assert(fraction, 0.25)
assert(withDeferredExtra, null)
delete sensor.reading
assert(value, null)
assert(fraction, null)
sensor.reading = 3.75
assert(whole, 3)
assert(fraction, 0.75)
sensor.optional = 99
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = 5.5
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 3.75)
assert(whole, 3)
assert(fraction, 0.75)
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(sensor.reading, 3.75)
delete sensor.optional
assert(withDeferredExtra, null)
assert(fraction, 0.75)
sensor.optional = [1, 2]
assert(withDeferredExtra, 0.75)
sensor.reading = 2.25
assert(whole, 2)
assert(fraction, 0.25)
assert(withDeferredExtra, 0.25)
fraction
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
    let mut run = runner();
    run(r#"source = 3
def reading left right => source + left + right
result = reading(2, 4)
assert(result, 9)
try:
    def reading left right => missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(result, 9)
source = 5
assert(result, 11)
def prefer current fallback => current ||= fallback
def combined first second scale => (first + second) * scale
assert(prefer(0, 9), 9)
assert(prefer(3, 9), 3)
assert(combined(2, 5, 3), 21)
combined(2, 5, 3)
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
    let mut run = runner();
    run(r#"class Sensor
sensor = Sensor()
start = sensor.lower
end = sensor.upper
sample = sensor.reading
weight = Math.inverseLerp(start, end, sample)
withDeferredExtra = Math.inverseLerp(start, end, sample, sensor.optional)
assert(weight, null)
assert(withDeferredExtra, null)
sensor.lower = 0
sensor.upper = 8
sensor.reading = 2
assert(weight, 0.25)
assert(withDeferredExtra, null)
sensor.optional = 99
assert(withDeferredExtra, 0.25)
delete sensor.reading
assert(weight, null)
assert(withDeferredExtra, null)
sensor.reading = 6
assert(weight, 0.75)
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = 4
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 6)
assert(weight, 0.75)
assert(withDeferredExtra, 0.75)
try:
    sensor.reading = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(sensor.reading, 6)
delete sensor.optional
assert(withDeferredExtra, null)
assert(weight, 0.75)
sensor.optional = [1, 2]
assert(withDeferredExtra, 0.75)
sensor.reading = 2
assert(weight, 0.25)
assert(withDeferredExtra, 0.25)
weight
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
    let mut run = runner();
    run(r#"class Sensor
sensor = Sensor()
start = sensor.lower
end = sensor.upper
targetStart = sensor.outputLower
targetEnd = sensor.outputUpper
sample = sensor.reading
mapped = Math.remap(start, end, targetStart, targetEnd, sample)
withDeferredExtra = Math.remap(start, end, targetStart, targetEnd, sample, sensor.optional)
assert(mapped, null)
assert(withDeferredExtra, null)
sensor.lower = 0
sensor.upper = 8
sensor.outputLower = 10
sensor.outputUpper = 30
sensor.reading = 2
assert(mapped, 15)
assert(withDeferredExtra, null)
sensor.optional = 99
assert(withDeferredExtra, 15)
delete sensor.reading
assert(mapped, null)
assert(withDeferredExtra, null)
sensor.reading = 6
assert(mapped, 25)
assert(withDeferredExtra, 25)
try:
    sensor.reading = 4
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(sensor.reading, 6)
assert(mapped, 25)
assert(withDeferredExtra, 25)
try:
    sensor.reading = missing
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(sensor.reading, 6)
delete sensor.optional
assert(withDeferredExtra, null)
assert(mapped, 25)
sensor.optional = [1, 2]
assert(withDeferredExtra, 25)
sensor.reading = 2
assert(mapped, 15)
assert(withDeferredExtra, 15)
mapped
# return: 15"#);
}

/// Nucleoid defines comma-free bare indented blocks
#[rustfmt::skip]
#[test]
fn defines_comma_free_bare_indented_blocks() {
    let mut run = runner();
    run(r#"def add left right:
    total = left + right
    return total"#);
    run(r#"def scaled left right factor:
    return (left + right) * factor"#);
    run(r#"def largest left right:
    if left > right:
        return left
    return right"#);
    run(r#"def pair left right:
    return [left, right]"#);
    run(r#"def coordinates x y:
    return {x, y}"#);
    run(r#"def noReturn left right:
    left + right"#);
    assert_eq!(run("add(2, 3)"), 5);
    assert_eq!(run("scaled(2, 3, 4)"), 20);
    assert_eq!(run("largest(7, 3)"), 7);
    assert_eq!(run("largest(2, 9)"), 9);
    assert_eq!(run(r#"pair("left", "right")"#), serde_json::json!(["left", "right"]));
    run("point = coordinates(2, 3)");
    assert_eq!(run("point.x"), 2);
    assert_eq!(run("point.y"), 3);
    assert_eq!(run("noReturn(2, 3)"), serde_json::Value::Null);
}

/// Nucleoid keeps comma-free bare indented blocks reactive
#[rustfmt::skip]
#[test]
fn keeps_comma_free_bare_indented_blocks_reactive() {
    let mut run = runner();
    run("offset = 1");
    run(r#"def adjusted left right:
    return left + right + offset"#);
    run("alias = adjusted");
    run("frozen = adjusted.value");
    run("left = 2");
    run("right = 3");
    run("total = alias(left, right)");
    run("snapshot = total.value");
    assert_eq!(run("total"), 6);
    assert_eq!(run("snapshot"), 6);
    run("offset = 4");
    assert_eq!(run("total"), 9);
    run("left = 5");
    assert_eq!(run("total"), 12);
    run(r#"def adjusted left right:
    return left * right + offset"#);
    assert_eq!(run("total"), 19);
    assert_eq!(run("frozen(2, 3)"), 9);
    assert_eq!(run("snapshot"), 6);
    run("right = 2");
    assert_eq!(run("total"), 14);
}

/// Nucleoid preserves comma-free block methods and initializers
#[rustfmt::skip]
#[test]
fn preserves_comma_free_block_methods_and_initializers() {
    let mut run = runner();
    run(r#"class Parent:
    def init left right:
        this.left = left
        this.right = right
    def total left right:
        return this.left + this.right + left + right"#);
    run(r#"class Child: Parent
    def init left right:
        super(left, right)
        self.extra = 2
    def total left right:
        return this.left + this.right + left + right"#);
    run("child = Child(2, 3)");
    assert_eq!(run("child.left"), 2);
    assert_eq!(run("child.right"), 3);
    assert_eq!(run("child.extra"), 2);
    run("observed = child.total(4, 5)");
    assert_eq!(run("observed"), 14);
    run("child.left = 7");
    assert_eq!(run("observed"), 19);
    run("result = child.total(1, 2)");
    assert_eq!(run("result"), 13);
    run("child.right = 4");
    assert_eq!(run("observed"), 20);
    assert_eq!(run("result"), 14);
    assert_eq!(run("Parent.length"), 0);
    assert_eq!(run("Child.length"), 1);
}

/// Nucleoid preserves comma-free block bindings and explicit returns
#[rustfmt::skip]
#[test]
fn preserves_comma_free_block_bindings_and_explicit_returns() {
    let mut run = runner();
    run("left = 10");
    run("right = 20");
    run(r#"def pair first second:
    return [first, second]"#);
    run(r#"def add first second:
    first += second
    return first"#);
    run(r#"def outer initial extra:
    def combine first second scale:
        return (first + second) * scale
    return combine(initial, extra, 3)"#);
    run(r#"def factory ignored other:
    return (first, second) => first + second"#);
    run(r#"def bareReturn first second:
    return"#);
    assert_eq!(run("pair(3, 4)"), serde_json::json!([3, 4]));
    assert_eq!(run("pair(3)"), serde_json::json!([3, null]));
    assert_eq!(run("pair()"), serde_json::json!([null, null]));
    assert_eq!(run("pair(3, 4, 99)"), serde_json::json!([3, 4]));
    assert_eq!(run("add(3, 4)"), 7);
    assert_eq!(run("left"), 10);
    assert_eq!(run("right"), 20);
    assert_eq!(run("outer(4, 2)"), 18);
    run("callback = factory(10, 20)");
    assert_eq!(run("callback(2, 3)"), 5);
    assert_eq!(run("bareReturn(3, 4)"), serde_json::Value::Null);
}

/// Nucleoid rolls back comma-free block redefinitions
#[rustfmt::skip]
#[test]
fn rolls_back_comma_free_block_redefinitions() {
    let mut run = runner();
    run(r#"source = 3
def reading left right:
    return source + left + right
result = reading(2, 4)
assert(result, 9)
try:
    def reading left right:
        return missing + left + right
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(result, 9)
source = 5
assert(result, 11)
def combined first second scale:
    return (first + second) * scale
def prefer current fallback:
    current ||= fallback
    return current
assert(prefer(0, 9), 9)
assert(prefer(3, 9), 3)
assert(combined(2, 5, 3), 21)
combined(2, 5, 3)
# return: 21"#);
}

/// Nucleoid flat maps lists by one level
#[rustfmt::skip]
#[test]
fn flat_maps_lists_by_one_level() {
    let mut run = runner();
    run("numbers = [1, 2, 3]");
    run("def expand(value) => [value, value * 10]");
    run("expanded = numbers.flatMap(expand)");
    assert_eq!(run("expanded"), serde_json::json!([1, 10, 2, 20, 3, 30]));
    assert_eq!(run("numbers"), serde_json::json!([1, 2, 3]));
    run(r#"mixed = [[[1]], [2, 3], 4, "xy", []]"#);
    run("flattened = mixed.flatMap(item => item)");
    assert_eq!(run("flattened"), serde_json::json!([[1], 2, 3, 4, "xy"]));
    assert_eq!(run("mixed[0]"), serde_json::json!([[1]]));
    run("nullable = numbers.flatMap(item => [item, null])");
    assert_eq!(run("nullable"), serde_json::json!([1, null, 2, null, 3, null]));
    run("empty = [].flatMap(item => missing)");
    assert_eq!(run("empty"), serde_json::json!([]));
}

/// Nucleoid keeps flat mapped lists reactive
#[rustfmt::skip]
#[test]
fn keeps_flat_mapped_lists_reactive() {
    let mut run = runner();
    run("numbers = [1, 2]");
    run("factor = 2");
    run("def expand(value) => [value, value * factor]");
    run("expanded = numbers.flatMap(expand)");
    assert_eq!(run("expanded"), serde_json::json!([1, 2, 2, 4]));
    run("factor = 3");
    assert_eq!(run("expanded"), serde_json::json!([1, 3, 2, 6]));
    run("numbers.push(4)");
    assert_eq!(run("expanded"), serde_json::json!([1, 3, 2, 6, 4, 12]));
    run("numbers.pop()");
    assert_eq!(run("expanded"), serde_json::json!([1, 3, 2, 6]));
    run("def expand(value) => [value + factor]");
    assert_eq!(run("expanded"), serde_json::json!([4, 5]));
    assert_eq!(run("numbers"), serde_json::json!([1, 2]));
}

/// Nucleoid flat maps class populations
#[rustfmt::skip]
#[test]
fn flat_maps_class_populations() {
    let mut run = runner();
    run(r#"class Batch(items):
    this.items = items
batch1 = Batch([1, 2])
batch2 = Batch([3])
expanded = Batch.flatMap(batch => batch.items)
assert(expanded, [1, 2, 3])
batch1.items.push(4)
assert(expanded, [1, 2, 4, 3])
batch3 = Batch([5])
assert(expanded, [1, 2, 4, 3, 5])
try:
    delete batch2
catch error:
    assert(error, TypeError("Cannot delete object 'batch2'"))
assert(expanded, [1, 2, 4, 3, 5])
batch2.items = []
assert(expanded, [1, 2, 4, 5])
assert(Batch.length, 3)
assert(batch1.items, [1, 2, 4])"#);
}

/// Nucleoid preserves flat mapping callable snapshots
#[rustfmt::skip]
#[test]
fn preserves_flat_mapping_callable_snapshots() {
    let mut run = runner();
    run("numbers = [1, 2]");
    run("factor = 2");
    run("def expand(value) => [value * factor]");
    run("callback = expand");
    run("frozen = expand.value");
    run("live = numbers.flatMap(callback)");
    run("saved = numbers.flatMap(frozen)");
    run("snapshot = live.value");
    assert_eq!(run("live"), serde_json::json!([2, 4]));
    assert_eq!(run("saved"), serde_json::json!([2, 4]));
    assert_eq!(run("snapshot"), serde_json::json!([2, 4]));
    run("def expand(value) => [value + factor]");
    assert_eq!(run("live"), serde_json::json!([3, 4]));
    assert_eq!(run("saved"), serde_json::json!([2, 4]));
    run("factor = 3");
    assert_eq!(run("live"), serde_json::json!([4, 5]));
    assert_eq!(run("saved"), serde_json::json!([3, 6]));
    assert_eq!(run("snapshot"), serde_json::json!([2, 4]));
}

/// Nucleoid rolls back flat mapping callback failures
#[rustfmt::skip]
#[test]
fn rolls_back_flat_mapping_callback_failures() {
    let mut run = runner();
    run(r#"numbers = [1, 2]
def expand(value) => [value, value * 2]
expanded = numbers.flatMap(expand)
assert(expanded, [1, 2, 2, 4])
try:
    def expand(value):
        throw "FLAT_MAPPING_ABORT"
catch error:
    assert(error, "FLAT_MAPPING_ABORT")
assert(expanded, [1, 2, 2, 4])
assert(numbers, [1, 2])
numbers.push(3)
assert(expanded, [1, 2, 2, 4, 3, 6])
21
# return: 21"#);
}

/// Nucleoid replaces every literal string match
#[rustfmt::skip]
#[test]
fn replaces_every_literal_string_match() {
    let mut run = runner();
    run(r#"text = "one|two|one""#);
    run(r#"joined = text.replaceAll("|", "/")"#);
    run(r#"overlap = "aaaaa".replaceAll("aa", "X")"#);
    run(r#"literal = "a.b.a".replaceAll(".", "$&")"#);
    run(r#"noMatch = "abc".replaceAll("z", "")"#);
    assert_eq!(run("joined"), "one/two/one");
    assert_eq!(run("overlap"), "XXa");
    assert_eq!(run("literal"), "a$&b$&a");
    assert_eq!(run("noMatch"), "abc");
    assert_eq!(run(r#""a.a.a".replace(".", "/")"#), "a/a.a");
    assert_eq!(run("text"), "one|two|one");
    run(r#"joined

# return: "one/two/one""#);
}

/// Nucleoid replaces empty string matches at Unicode boundaries
#[rustfmt::skip]
#[test]
fn replaces_empty_string_matches_at_unicode_boundaries() {
    let mut run = runner();
    run(r#"def replaceEvery(text, search, replacement):
    if search == "":
        if text == "":
            return replacement
        return replacement + text.split("").join(replacement) + replacement
    return text.split(search).join(replacement)"#);
    run("accent = String.fromCharCode(233)");
    run("symbol = String.fromCharCode(128512)");
    run("text = accent + symbol");
    run(r#"empty = "".replaceAll("", "|")"#);
    run(r#"inserted = text.replaceAll("", "|")"#);
    run(r#"unchanged = text.replaceAll("", "")"#);
    run(r#"legacy = replaceEvery(text, "", "|")"#);
    assert_eq!(run("empty"), "|");
    {
        let actual = run("inserted");
        let expected = run(r#"("|" + accent + "|" + symbol + "|")"#);
        assert_eq!(actual, expected);
    }
    {
        let actual = run("inserted");
        let expected = run("(legacy)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("unchanged");
        let expected = run("(text)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("text");
        let expected = run("(accent + symbol)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid keeps literal replacement reactive
#[rustfmt::skip]
#[test]
fn keeps_literal_replacement_reactive() {
    let mut run = runner();
    run(r#"text = "red|blue|red""#);
    run(r#"search = "|""#);
    run(r#"replacement = "/""#);
    run("normalized = text.replaceAll(search, replacement)");
    run("frozen = normalized.value");
    assert_eq!(run("normalized"), "red/blue/red");
    run(r#"replacement = "-""#);
    assert_eq!(run("normalized"), "red-blue-red");
    run(r#"text = "blue|green""#);
    assert_eq!(run("normalized"), "blue-green");
    run(r#"search = "blue""#);
    assert_eq!(run("normalized"), "-|green");
    run(r#"replacement = "cyan""#);
    assert_eq!(run("normalized"), "cyan|green");
    assert_eq!(run("frozen"), "red/blue/red");
    assert_eq!(run("text"), "blue|green");
}

/// Nucleoid replaces literal strings in class rules
#[rustfmt::skip]
#[test]
fn replaces_literal_strings_in_class_rules() {
    let mut run = runner();
    run(r#"class Page(title: str):
    this.title = title"#);
    run(r#"$Page.slug = $Page.title.replaceAll(" ", "-")"#);
    run(r#"first = Page("One Two One")"#);
    run(r#"second = Page("Three Four")"#);
    run("slugs = Page.map(page => page.slug)");
    assert_eq!(run("first.slug"), "One-Two-One");
    assert_eq!(run("slugs"), serde_json::json!(["One-Two-One", "Three-Four"]));
    run(r#"first.title = "Fresh Page""#);
    assert_eq!(run("first.slug"), "Fresh-Page");
    assert_eq!(run("slugs"), serde_json::json!(["Fresh-Page", "Three-Four"]));
    run(r#"$Page.slug = $Page.title.lower().replaceAll(" ", "_")"#);
    assert_eq!(run("first.slug"), "fresh_page");
    assert_eq!(run("second.slug"), "three_four");
    assert_eq!(run("slugs"), serde_json::json!(["fresh_page", "three_four"]));
}

/// Nucleoid rolls back literal replacement argument failures
#[rustfmt::skip]
#[test]
fn rolls_back_literal_replacement_argument_failures() {
    let mut run = runner();
    run(r#"source = "a|b"
def fail():
    source = "changed|changed"
    throw "stop"
try:
    source.replaceAll("|", fail())
catch error:
    assert(error, "stop")
assert(source, "a|b")
recovered = source.replaceAll("|", "/")
assert(recovered, "a/b")"#);
}

/// Nucleoid selects direct conditional alternatives
#[rustfmt::skip]
#[test]
fn selects_direct_conditional_alternatives() {
    let mut run = runner();
    run(r#"def classify(state, urgent):
    if state == "blocked"
        return "hold"
    else urgent
        return "fast"
    elif state == "ready"
        return "send"
    else:
        return "wait""#);
    run(r#"state = "ready""#);
    run("urgent = false");
    run("label = classify(state, urgent)");
    assert_eq!(run("label"), "send");
    run("urgent = true");
    assert_eq!(run("label"), "fast");
    run(r#"state = "blocked""#);
    assert_eq!(run("label"), "hold");
    run("urgent = false");
    assert_eq!(run("label"), "hold");
    run(r#"state = "queued""#);
    assert_eq!(run("label"), "wait");
    run(r#"label

# return: "wait""#);
}

/// Nucleoid keeps direct conditional class alternatives reactive
#[rustfmt::skip]
#[test]
fn keeps_direct_conditional_class_alternatives_reactive() {
    let mut run = runner();
    run("class Ticket");
    run("first = Ticket()");
    run(r#"first.state = "new""#);
    run(r#"first.priority = "normal""#);
    run("second = Ticket()");
    run(r#"second.state = "review""#);
    run(r#"second.priority = "urgent""#);
    run(r#"if $Ticket.state == "blocked"
    $Ticket.queue = "hold"
else $Ticket.priority == "urgent"
    $Ticket.queue = "fast"
elif $Ticket.state == "review"
    $Ticket.queue = "review"
else:
    $Ticket.queue = "normal""#);
    assert_eq!(run("first.queue"), "normal");
    assert_eq!(run("second.queue"), "fast");
    run(r#"first.state = "blocked""#);
    assert_eq!(run("first.queue"), "hold");
    run(r#"second.priority = "normal""#);
    assert_eq!(run("second.queue"), "review");
    run(r#"first.priority = "urgent""#);
    assert_eq!(run("first.queue"), "hold");
    run(r#"first.state = "new""#);
    assert_eq!(run("first.queue"), "fast");
    assert_eq!(run("second.queue"), "review");
    run(r#"second.state = "done""#);
    assert_eq!(run("second.queue"), "normal");
}

/// Nucleoid preserves direct alternative evaluation order
#[rustfmt::skip]
#[test]
fn preserves_direct_alternative_evaluation_order() {
    let mut run = runner();
    run(r#"def decide(first, second, third):
    if first
        return "first"
    else second
        return "second"
    elif third
        return "third"
    else:
        return "none"
assert(decide(true, true, true), "first")
assert(decide(false, true, true), "second")
assert(decide(false, false, true), "third")
assert(decide(false, false, false), "none")
def guarded(value):
    if value
        return "ready"
    else missing
        return "bad"
    else:
        return "other"
assert(guarded(true), "ready")
try:
    guarded(false)
catch error:
    assert(error, ReferenceError("missing is not defined"))"#);
}

/// Nucleoid preserves direct alternative scope and return flow
#[rustfmt::skip]
#[test]
fn preserves_direct_alternative_scope_and_return_flow() {
    let mut run = runner();
    run(r#"class Router:
    def init(prefix):
        this.prefix = prefix
    def choose(status, fallback):
        if status == "blocked"
            return this.prefix + "hold"
        else status == "ready"
            return this.prefix + "send"
        elif fallback
            return this.prefix + "later"
        else:
            return this.prefix + "skip""#);
    run(r#"router = Router("a:")"#);
    run(r#"status = "ready""#);
    run("live = router.choose(status, false)");
    run("snapshot = live.value");
    assert_eq!(run("live"), "a:send");
    assert_eq!(run("snapshot"), "a:send");
    assert_eq!(run(r#"router.choose("unknown", false)"#), "a:skip");
    assert_eq!(run(r#"router.choose("unknown", true)"#), "a:later");
    assert_eq!(run(r#"router.choose("blocked", true)"#), "a:hold");
    assert_eq!(run(r#"router.choose("ready", true)"#), "a:send");
    assert_eq!(run("status"), "ready");
    assert_eq!(run("router.prefix"), "a:");
}

/// Nucleoid rolls back failed direct alternatives
#[rustfmt::skip]
#[test]
fn rolls_back_failed_direct_alternatives() {
    let mut run = runner();
    run(r#"source = "ready"
def route(status):
    if status == "ready"
        return "send"
    else status == "hold"
        return "wait"
    else:
        return "skip"
live = route(source)
assert(live, "send")
try:
    source = "hold"
    def route(status):
        if status == "ready"
            return "send"
        elif status == "hold"
            throw "blocked"
        else:
            return "skip"
catch error:
    assert(error, "blocked")
assert(source, "ready")
assert(live, "send")
source = "hold"
assert(live, "wait")
source = "unknown"
assert(live, "skip")
live
# return: "skip""#);
}

/// Nucleoid constructs comma-free explicit records
#[rustfmt::skip]
#[test]
fn constructs_comma_free_explicit_records() {
    let mut run = runner();
    run("amount = 4");
    run(r#"unit = "C""#);
    run("reading = {amount: amount unit: unit active: true}");
    assert_eq!(run("reading.amount"), 4);
    assert_eq!(run("reading.unit"), "C");
    assert_eq!(run("reading.active"), true);
    run("amount = 9");
    assert_eq!(run("reading.amount"), 9);
    run(r#"unit = "F""#);
    assert_eq!(run("reading.unit"), "F");
    run(r#"reading.unit

# return: "F""#);
}

/// Nucleoid preserves comma-free record keys and replacement order
#[rustfmt::skip]
#[test]
fn preserves_comma_free_record_keys_and_replacement_order() {
    let mut run = runner();
    run("seed = 3");
    run(r#"record = {"state": "open" 1: "first" "1": "latest" copy: seed copy: seed + 1}"#);
    assert_eq!(run("record.state"), "open");
    assert_eq!(run(r#"record["1"]"#), "latest");
    assert_eq!(run("record.copy"), 4);
    run("seed = 8");
    assert_eq!(run("record.copy"), 9);
    run(r#"record.state = "closed""#);
    assert_eq!(run("record.state"), "closed");
}

/// Nucleoid nests comma-free records without changing list or call separators
#[rustfmt::skip]
#[test]
fn nests_comma_free_records_without_changing_list_or_call_separators() {
    let mut run = runner();
    run("def pair(left, right) => left + right");
    run("first = 2");
    run(r#"packet = {meta: {kind: "sample" ready: true} data: [{reading: first unit: "C"}, {reading: pair(first, 3) unit: "K"}] tag: "batch"}"#);
    assert_eq!(run("packet.meta.kind"), "sample");
    assert_eq!(run("packet.data[0].reading"), 2);
    assert_eq!(run("packet.data[1].reading"), 5);
    assert_eq!(run("packet.data[1].unit"), "K");
    run("first = 7");
    assert_eq!(run("packet.data[0].reading"), 7);
    assert_eq!(run("packet.data[1].reading"), 10);
}

/// Nucleoid preserves scope and snapshots in comma-free records
#[rustfmt::skip]
#[test]
fn preserves_scope_and_snapshots_in_comma_free_records() {
    let mut run = runner();
    run(r#"def package(amount):
    return {reading: amount initial: amount.value label: "sample"}"#);
    run("source = 3");
    run("item = package(source)");
    run("view = {current: source saved: source.value packed: item.reading}");
    assert_eq!(run("item.reading"), 3);
    assert_eq!(run("view.current"), 3);
    assert_eq!(run("view.saved"), 3);
    run("source = 6");
    assert_eq!(run("item.reading"), 6);
    assert_eq!(run("view.current"), 6);
    assert_eq!(run("view.saved"), 3);
    assert_eq!(run("view.packed"), 6);
}

/// Nucleoid rolls back failed comma-free record values
#[rustfmt::skip]
#[test]
fn rolls_back_failed_comma_free_record_values() {
    let mut run = runner();
    run(r#"state = 1
record = {status: "old" marker: state}
message = ""
try:
    state = 2
    record = {status: "new" marker: missing status: "last"}
catch error:
    message = error
assert(message, ReferenceError("missing is not defined"))
assert(state, 1)
assert(record.status, "old")
assert(record.marker, 1)
state = 3
assert(record.marker, 3)
record.status
# return: "old""#);
}

/// Nucleoid checks values with colon-led assertion statements
#[rustfmt::skip]
#[test]
fn checks_values_with_colon_led_assertion_statements() {
    runner()("reading = {amount: 3, unit: \"C\", active: true, empty: null}\nlabel = \"sensor\"\nassert: reading.amount, 3\nassert: reading.unit, \"C\"\nassert: reading.active, true\nassert: reading.empty, null\nassert: [reading.amount, label], [3, \"sensor\"]\nassert: {kind: label, valid: reading.active}, {kind: \"sensor\", valid: true}");
    let mut run = runner();
    run(r#"reading = {amount: 3, unit: "C", active: true, empty: null}"#);
    run(r#"label = "sensor""#);
    assert_eq!(run("reading.amount"), 3);
    assert_eq!(run("reading.unit"), "C");
    assert_eq!(run("reading.active"), true);
    assert_eq!(run("reading.empty"), serde_json::Value::Null);
    assert_eq!(run("[reading.amount, label]"), serde_json::json!([3, "sensor"]));
    {
        let actual = run("({kind: label, valid: reading.active})");
        let expected = run(r#"({kind: "sensor", valid: true})"#);
        assert_eq!(actual, expected);
    }
}

/// Nucleoid keeps colon-led assertions out of reactive reasoning
#[rustfmt::skip]
#[test]
fn keeps_colon_led_assertions_out_of_reactive_reasoning() {
    runner()("source = 2\ntotal = source + 1\nassert: total, 3\nassert: (affects source).length, 1\nassert: (why total)[1].node, \"source\"\nsource = 6\nassert: total, 7\nassert: (why total)[1].holds, 6\ntotal\n# return: 7");
    let mut run = runner();
    run("source = 2");
    run("total = source + 1");
    assert_eq!(run("total"), 3);
    assert_eq!(run("(affects source).length"), 1);
    assert_eq!(run("(why total)[1].node"), "source");
    run("source = 6");
    assert_eq!(run("total"), 7);
    assert_eq!(run("(why total)[1].holds"), 6);
    run(r#"total

# return: 7"#);
}

/// Nucleoid preserves scope and return flow around colon-led assertions
#[rustfmt::skip]
#[test]
fn preserves_scope_and_return_flow_around_colon_led_assertions() {
    let mut run = runner();
    run(r#"def check_name(name):
    saved = name.value
    assert: saved, "draft"
    return {name: name, initial: saved}
packet = check_name("draft")
assert: packet.name, "draft"
assert: packet.initial, "draft"
assert: packet.name == packet.initial, true"#);
}

/// Nucleoid preserves colon-led assertion argument evaluation order
#[rustfmt::skip]
#[test]
fn preserves_colon_led_assertion_argument_evaluation_order() {
    runner()("log = []\ndef note(label):\n    log.push(label)\n    return 7\nassert: note(\"actual\"), note(\"expected\")\nassert: log, [\"actual\", \"expected\"]\nassert: log.length, 2");
    let mut run = runner();
    run("log = []");
    run(r#"def note(label):
    log.push(label)
    return 7"#);
    {
        let actual = run(r#"note("actual")"#);
        let expected = run(r#"(note("expected"))"#);
        assert_eq!(actual, expected);
    }
    assert_eq!(run("log"), serde_json::json!(["actual", "expected"]));
    assert_eq!(run("log.length"), 2);
}

/// Nucleoid checks caught errors and rollback with colon-led assertions
#[rustfmt::skip]
#[test]
fn checks_caught_errors_and_rollback_with_colon_led_assertions() {
    runner()("state = 1\nmessage = \"\"\ntry:\n    state = 2\n    failed = missing\ncatch error:\n    message = error\n    assert: error, ReferenceError(\"missing is not defined\")\nassert: state, 1\nassert: message, ReferenceError(\"missing is not defined\")");
    let mut run = runner();
    run(r#"state = 1
message = ""
try:
    state = 2
    failed = missing
catch error:
    message = error
    assert: error, ReferenceError("missing is not defined")
assert: state, 1
assert: message, ReferenceError("missing is not defined")"#);
}

/// Nucleoid catches thrown values with colon-free indented suites
#[rustfmt::skip]
#[test]
fn catches_thrown_values_with_colon_free_indented_suites() {
    let mut run = runner();
    run(r#"status = "ready"
try
    throw "STOP"
catch problem
    assert(problem, "STOP")
assert(status, "ready")
try
    throw 7
catch (issue)
    assert(issue, 7)
try
    missing
catch
    assert(error, ReferenceError("missing is not defined"))"#);
}

/// Nucleoid restores dependencies through colon-free exception suites
#[rustfmt::skip]
#[test]
fn restores_dependencies_through_colon_free_exception_suites() {
    let mut run = runner();
    run(r#"source = 2
derived = source * 3
try
    source = 9
    throw "ABORT"
catch failure
    assert(failure, "ABORT")
assert(source, 2)
assert(derived, 6)
source = 4
assert(derived, 12)
derived
# return: 12"#);
}

/// Nucleoid preserves nested catch association without suite colons
#[rustfmt::skip]
#[test]
fn preserves_nested_catch_association_without_suite_colons() {
    let mut run = runner();
    run(r#"state = "ready"
try
    try
        throw "INNER"
    catch inner
        throw inner + "-OUTER"
catch outer
    assert(outer, "INNER-OUTER")
assert(state, "ready")"#);
}

/// Nucleoid preserves function returns through compact exception suites
#[rustfmt::skip]
#[test]
fn preserves_function_returns_through_compact_exception_suites() {
    let mut run = runner();
    run(r#"def recover(flag):
    try
        if flag:
            throw "FALLBACK"
        return "OK"
    catch issue
        return issue
first = recover(false)
second = recover(true)
assert(first, "OK")
assert(second, "FALLBACK")"#);
}

/// Nucleoid rolls back class rules with colon-free catch suites
#[rustfmt::skip]
#[test]
fn rolls_back_class_rules_with_colon_free_catch_suites() {
    let mut run = runner();
    run(r#"class Parcel(state: str):
    this.state = state
parcel = Parcel("queued")
$Parcel.label = $Parcel.state + "!"
try
    parcel.state = "broken"
    throw "ABORT"
catch reason
    assert(reason, "ABORT")
assert(parcel.state, "queued")
assert(parcel.label, "queued!")
parcel.state = "sent"
assert(parcel.label, "sent!")"#);
}

/// Nucleoid binds opted-in constructor arguments by name
#[rustfmt::skip]
#[test]
fn binds_opted_in_constructor_arguments_by_name() {
    let mut run = runner();
    run(r#"a = 10
b = 20
class Pair(a, b):
    this.a = a
    this.b = b
pair = Pair(b=2, a=1)
assert(pair, {"id": "pair", "a": 1, "b": 2})
assert(a, 10)
assert(b, 20)"#);
}

/// Nucleoid evaluates named constructor values in source order
#[rustfmt::skip]
#[test]
fn evaluates_named_constructor_values_in_source_order() {
    let mut run = runner();
    run(r#"'use imperative'
class Pair(a, b):
    this.a = a
    this.b = b
counter = 0
pair = Pair(b=(counter = counter + 1), a=(counter = counter + 1))
assert(pair.a, 2)
assert(pair.b, 1)
assert(counter, 2)"#);
}

/// Nucleoid inherits opted-in named constructors
#[rustfmt::skip]
#[test]
fn inherits_opted_in_named_constructors() {
    let mut run = runner();
    run(r#"class Parent(a, b):
    this.a = a
    this.b = b
class Child: Parent
    pass
child = Child(b=2, a=1)
assert(child, {"id": "child", "a": 1, "b": 2})"#);
}

/// Nucleoid binds named parameters through opted-in initializers
#[rustfmt::skip]
#[test]
fn binds_named_parameters_through_opted_in_initializers() {
    let mut run = runner();
    run(r#"class Pair:
    def init(a, b):
        this.a = a
        this.b = b
pair = Pair(b=2, a=1)
assert(pair, {"id": "pair", "a": 1, "b": 2})"#);
}

/// Nucleoid keeps missing opted-in constructor parameters null
#[rustfmt::skip]
#[test]
fn keeps_missing_opted_in_constructor_parameters_null() {
    let mut run = runner();
    run(r#"class Pair(a, b):
    this.a = a
    this.b = b
pair = Pair(a=1)
assert(pair.a, 1)
assert(pair.b, null)"#);
}

/// Nucleoid snapshots named constructor argument values
#[rustfmt::skip]
#[test]
fn snapshots_named_constructor_argument_values() {
    let mut run = runner();
    run(r#"left = 1
right = 2
class Pair(a, b):
    this.a = a
    this.b = b
pair = Pair(b=right, a=left)
assert(pair.a, 1)
assert(pair.b, 2)
left = 3
assert(left, 3)
assert(pair.a, 1)
assert(pair.b, 2)"#);
}

/// Nucleoid preserves positional constructor assignment arguments
#[rustfmt::skip]
#[test]
fn preserves_positional_constructor_assignment_arguments() {
    let mut run = runner();
    run(r#"class Legacy(a, b):
    this.a = a
    this.b = b
a = b = 0
legacy = Legacy((b = 2), (a = 1))
assert(legacy.a, 2)
assert(legacy.b, 1)
assert(a, 1)
assert(b, 2)"#);
}

/// Nucleoid forwards named arguments to opted-in parent constructors
#[rustfmt::skip]
#[test]
fn forwards_named_arguments_to_opted_in_parent_constructors() {
    let mut run = runner();
    run(r#"class Parent(a, b):
    this.a = a
    this.b = b
class Child(first, second): Parent
    super(b=second, a=first)
child = Child(1, 2)
assert(child, {"id": "child", "a": 1, "b": 2})"#);
}

/// Nucleoid constructs students with positional and named arguments
#[rustfmt::skip]
#[test]
fn constructs_students_with_positional_and_named_arguments() {
    let mut run = runner();
    run(r#"class Student(name):
    this.name = name
positional = Student("Can")
named = Student(name="Can")
assert(positional.name, "Can")
assert(named.name, "Can")"#);
}

/// Nucleoid mixes positional and named constructor arguments
#[rustfmt::skip]
#[test]
fn mixes_positional_and_named_constructor_arguments() {
    let mut run = runner();
    run(r#"class Student(name, age):
    this.name = name
    this.age = age
ordered = Student(name="Can", age=20)
reordered = Student(age=20, name="Can")
mixed = Student("Can", age=20)
assert(ordered, {"id": "ordered", "name": "Can", "age": 20})
assert(reordered, {"id": "reordered", "name": "Can", "age": 20})
assert(mixed, {"id": "mixed", "name": "Can", "age": 20})"#);
}

/// Nucleoid validates constructor names without caller assignments
#[rustfmt::skip]
#[test]
fn validates_constructor_names_without_caller_assignments() {
    let mut run = runner();
    run(r#"class Student(name, age):
    this.name = name
    this.age = age
name = "Before"
student = Student(name="Can", age=20)
assert(name, "Before")
assert(student.name, "Can")
try:
    Student(nmae="Can", age=20)
catch error:
    assert(error, TypeError("Unknown constructor argument 'nmae' for Student"))
try:
    Student("Can", name="Other", age=20)
catch error:
    assert(error, TypeError("Duplicate constructor argument 'name' for Student"))"#);
}

/// Nucleoid preserves ordinary function argument assignments
#[rustfmt::skip]
#[test]
fn preserves_ordinary_function_argument_assignments() {
    let mut run = runner();
    run(r#"'use imperative'
def pair(a, b):
    return [a, b]
a = 0
b = 0
result = pair(b=2, a=1)
assert(result, [2, 1])
assert(a, 1)
assert(b, 2)"#);
}

/// Nucleoid filters reasoning selections by stated and derived state
#[rustfmt::skip]
#[test]
fn filters_reasoning_selections_by_stated_and_derived_state() {
    let mut run = runner();
    run("seed = 2");
    run("middle = seed + 3");
    run("result = middle * 2");
    run("facts = result |> why |> stated");
    run("conclusions = result |> why |> derived");
    assert_eq!(run("facts.map(step => step.node)"), serde_json::json!(["seed"]));
    assert_eq!(run("facts[0].holds"), 2);
    assert_eq!(run("facts[0].from"), serde_json::json!([]));
    assert_eq!(run("conclusions.map(step => step.node)"), serde_json::json!(["result", "middle"]));
    assert_eq!(run("conclusions[0].holds"), 10);
    assert_eq!(run("conclusions[0].from"), serde_json::json!(["middle"]));
    assert_eq!(run("conclusions[0].state"), "derived");
}

/// Nucleoid keeps reasoning state filters live through equal valued rewrites
#[rustfmt::skip]
#[test]
fn keeps_reasoning_state_filters_live_through_equal_valued_rewrites() {
    let mut run = runner();
    run("seed = 1");
    run("value = seed + 1");
    run("facts = value |> why |> stated");
    run("conclusions = value |> why |> derived");
    assert_eq!(run("facts.map(step => step.node)"), serde_json::json!(["seed"]));
    assert_eq!(run("conclusions.map(step => step.node)"), serde_json::json!(["value"]));
    run("value = 2");
    assert_eq!(run("facts.map(step => step.node)"), serde_json::json!(["value"]));
    assert_eq!(run("conclusions"), serde_json::json!([]));
    run("value = seed + 1");
    run("seed = 3");
    assert_eq!(run("facts.map(step => step.node)"), serde_json::json!(["seed"]));
    assert_eq!(run("facts[0].holds"), 3);
    assert_eq!(run("conclusions.map(step => step.node)"), serde_json::json!(["value"]));
    assert_eq!(run("conclusions[0].holds"), 4);
}

/// Nucleoid observes empty state filtered model selections
#[rustfmt::skip]
#[test]
fn observes_empty_state_filtered_model_selections() {
    let mut run = runner();
    run("facts = model |> stated");
    run("conclusions = model |> derived");
    assert_eq!(run("facts"), serde_json::json!([]));
    assert_eq!(run("conclusions"), serde_json::json!([]));
    run("seed = 1");
    run("value = seed + 1");
    assert_eq!(run("facts.map(step => step.node)"), serde_json::json!(["seed"]));
    assert_eq!(run("conclusions.map(step => step.node)"), serde_json::json!(["value"]));
    run("delete value");
    assert_eq!(run("conclusions"), serde_json::json!([]));
    run("seed = 2");
    assert_eq!(run("facts[0].holds"), 2);
}

/// Nucleoid retains derived class rules without input edges in state filters
#[rustfmt::skip]
#[test]
fn retains_derived_class_rules_without_input_edges_in_state_filters() {
    let mut run = runner();
    run("class Subject");
    run("$Subject.active = true");
    run("item = Subject()");
    run("facts = item.active |> why |> stated");
    run("conclusions = item.active |> why |> derived");
    assert_eq!(run("facts"), serde_json::json!([]));
    assert_eq!(run("conclusions.length"), 1);
    assert_eq!(run("conclusions[0].node"), "item.active");
    assert_eq!(run("conclusions[0].from"), serde_json::json!([]));
    assert_eq!(run("conclusions[0].rule"), "$Subject.active = true");
    assert_eq!(run("conclusions[0].holds"), true);
}

/// Nucleoid restores state filtered explanations and preserves snapshots
#[rustfmt::skip]
#[test]
fn restores_state_filtered_explanations_and_preserves_snapshots() {
    let mut run = runner();
    run(r#"seed = 1
value = seed + 1
facts = value |> why |> stated
conclusions = value |> why |> derived
frozen = facts.value
try:
    value = 2
    throw "ABORT_STATE_FILTER"
catch error:
    assert(error, "ABORT_STATE_FILTER")
assert(facts.map(step => step.node), ["seed"])
assert(conclusions.map(step => step.node), ["value"])
value = 2
seed = 5
assert(facts.map(step => step.node), ["value"])
assert(facts[0].holds, 2)
assert(conclusions, [])
assert(frozen[0].node, "seed")
assert(frozen[0].holds, 1)"#);
}

/// Nucleoid filters individual references without traversing their ancestors
#[rustfmt::skip]
#[test]
fn filters_individual_references_without_traversing_their_ancestors() {
    let mut run = runner();
    run("seed = 1");
    run("value = seed + 1");
    run("fact = stated seed");
    run("notFact = stated value");
    run("conclusion = derived value");
    assert_eq!(run("fact.map(step => step.node)"), serde_json::json!(["seed"]));
    assert_eq!(run("notFact"), serde_json::json!([]));
    assert_eq!(run("conclusion.map(step => step.node)"), serde_json::json!(["value"]));
}

/// Nucleoid preserves reference errors while filtering reasoning states
#[rustfmt::skip]
#[test]
fn preserves_reference_errors_while_filtering_reasoning_states() {
    let mut run = runner();
    run(r#"seed = 1
value = seed + 1
facts = value |> why |> stated
try:
    1 |> stated
catch error:
    assert(error, TypeError("Cannot explain 1"))
assert(facts.map(step => step.node), ["seed"])
try:
    stated missing
catch error:
    assert(error, ReferenceError("missing is not defined"))"#);
}

/// Nucleoid keeps degree to radian conversion reactive
#[rustfmt::skip]
#[test]
fn keeps_degree_to_radian_conversion_reactive() {
    let mut run = runner();
    run("degrees = 180");
    run("converted = Math.toRadians(degrees)");
    {
        let actual = run("converted");
        let expected = run("(Math.PI)");
        assert_eq!(actual, expected);
    }
    run("degrees = 90");
    {
        let actual = run("converted");
        let expected = run("(Math.PI / 2)");
        assert_eq!(actual, expected);
    }
    run("degrees = -180");
    {
        let actual = run("converted");
        let expected = run("(-Math.PI)");
        assert_eq!(actual, expected);
    }
    run("degrees = 360");
    {
        let actual = run("converted");
        let expected = run("(Math.PI * 2)");
        assert_eq!(actual, expected);
    }
    run("degrees = 0");
    assert_eq!(run("converted"), 0);
    run("degrees = -0");
    assert_eq!(run("converted"), 0);
    {
        let actual = run("1 / converted");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("degrees = 90");
    run(r#"converted

# return: 1.5707963267948966"#);
}

/// Nucleoid preserves numeric coercion while converting degrees to radians
#[rustfmt::skip]
#[test]
fn preserves_numeric_coercion_while_converting_degrees_to_radians() {
    let mut run = runner();
    run(r#"value = " 180 ""#);
    run("converted = Math.toRadians(value)");
    {
        let actual = run("converted");
        let expected = run("(Math.PI)");
        assert_eq!(actual, expected);
    }
    run("value = true");
    {
        let actual = run("converted");
        let expected = run("(Math.PI / 180)");
        assert_eq!(actual, expected);
    }
    run("value = false");
    assert_eq!(run("converted"), 0);
    run("value = []");
    assert_eq!(run("converted"), 0);
    run(r#"value = ["90"]"#);
    {
        let actual = run("converted");
        let expected = run("(Math.PI / 2)");
        assert_eq!(actual, expected);
    }
    run("value = Date(180)");
    {
        let actual = run("converted");
        let expected = run("(Math.PI)");
        assert_eq!(actual, expected);
    }
    run(r#"value = "not-a-number""#);
    assert_eq!(run("Number.isNaN(converted)"), true);
    run("literalNull = Math.toRadians(null)");
    assert_eq!(run("literalNull"), 0);
}

/// Nucleoid preserves radian conversion extremes omissions and signed zeros
#[rustfmt::skip]
#[test]
fn preserves_radian_conversion_extremes_omissions_and_signed_zeros() {
    let mut run = runner();
    run("finite = Math.toRadians(Number.MAX_VALUE)");
    assert_eq!(run("Number.isFinite(finite)"), true);
    run("negativeFinite = Math.toRadians(-Number.MAX_VALUE)");
    assert_eq!(run("Number.isFinite(negativeFinite)"), true);
    assert_eq!(run("negativeFinite < 0"), true);
    run("positive = Math.toRadians(Number.POSITIVE_INFINITY)");
    run("negative = Math.toRadians(Number.NEGATIVE_INFINITY)");
    run("invalid = Math.toRadians(Number.NaN)");
    run("omitted = Math.toRadians()");
    run("zero = Math.toRadians(-0)");
    run("subnormal = Math.toRadians(Number.MIN_VALUE)");
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
    assert_eq!(run("Number.isNaN(invalid)"), true);
    assert_eq!(run("Number.isNaN(omitted)"), true);
    {
        let actual = run("1 / zero");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("subnormal > 0"), true);
}

/// Nucleoid applies radian conversion to independent class rules
#[rustfmt::skip]
#[test]
fn applies_radian_conversion_to_independent_class_rules() {
    let mut run = runner();
    run(r#"class Angle(degrees):
    this.degrees = degrees"#);
    run("$Angle.radians = Math.toRadians($Angle.degrees)");
    run("first = Angle(180)");
    run("second = Angle(90)");
    {
        let actual = run("first.radians");
        let expected = run("(Math.PI)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("second.radians");
        let expected = run("(Math.PI / 2)");
        assert_eq!(actual, expected);
    }
    run("first.degrees = -180");
    {
        let actual = run("first.radians");
        let expected = run("(-Math.PI)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("second.radians");
        let expected = run("(Math.PI / 2)");
        assert_eq!(actual, expected);
    }
    run("delete first.degrees");
    assert_eq!(run("first.radians"), serde_json::Value::Null);
    run("first.degrees = 360");
    {
        let actual = run("first.radians");
        let expected = run("(Math.PI * 2)");
        assert_eq!(actual, expected);
    }
    run("second.degrees = 45");
    {
        let actual = run("second.radians");
        let expected = run("(Math.PI / 4)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid restores radian conversion relationships after failed transactions
#[rustfmt::skip]
#[test]
fn restores_radian_conversion_relationships_after_failed_transactions() {
    let mut run = runner();
    run(r#"class Dial
dial = Dial()
dial.degrees = 180
converted = Math.toRadians(dial.degrees)
assert(converted, Math.PI)
try:
    dial.degrees = 90
    throw "ANGLE_ROLLBACK"
catch error:
    assert(error, "ANGLE_ROLLBACK")
assert(dial.degrees, 180)
assert(converted, Math.PI)
try:
    converted = Math.toRadians(missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(converted, Math.PI)
dial.degrees = 90
assert(converted, Math.PI / 2)"#);
}

/// Nucleoid preserves ignored radian conversion reads and missing extras
#[rustfmt::skip]
#[test]
fn preserves_ignored_radian_conversion_reads_and_missing_extras() {
    let mut run = runner();
    run(r#"class Probe
probe = Probe()
angle = -180
extra = 1
converted = Math.toRadians(angle, extra)
assert(converted, -Math.PI)
extra = 2
assert(converted, -Math.PI)
withMissing = Math.toRadians(angle, probe.missing)
assert(withMissing, null)
probe.missing = 1
assert(withMissing, -Math.PI)
angle = 90
assert(converted, Math.PI / 2)
assert(withMissing, Math.PI / 2)
try:
    Math.toRadians(angle, missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))"#);
}

/// Nucleoid iterates string characters in source order
#[rustfmt::skip]
#[test]
fn iterates_string_characters_in_source_order() {
    let mut run = runner();
    run(r#"text = "cab""#);
    run("seen = []");
    run(r#"for character of text:
    seen.push(character)"#);
    assert_eq!(run("seen"), serde_json::json!(["c", "a", "b"]));
    assert_eq!(run("text"), "cab");
    run(r#"seen

# return: ["c","a","b"]"#);
}

/// Nucleoid preserves Unicode scalars while iterating strings
#[rustfmt::skip]
#[test]
fn preserves_unicode_scalars_while_iterating_strings() {
    let mut run = runner();
    run(r#"text = "Aé🙂é""#);
    run("seen = []");
    run(r#"for character of text:
    seen.push(character)"#);
    assert_eq!(run("seen"), serde_json::json!(["A", "é", "🙂", "e", "́"]));
    assert_eq!(run("seen.length"), 5);
    {
        let actual = run(r#"seen.join("")"#);
        let expected = run("(text)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid snapshots string sources before loop bodies
#[rustfmt::skip]
#[test]
fn snapshots_string_sources_before_loop_bodies() {
    let mut run = runner();
    run(r#"holder = {text: "ab"}"#);
    run("seen = []");
    run(r#"for character of holder.text:
    seen.push(character)
    holder.text = "z""#);
    assert_eq!(run("seen"), serde_json::json!(["a", "b"]));
    assert_eq!(run("holder.text"), "z");
    run(r#"holder.text = "cd""#);
    assert_eq!(run("seen"), serde_json::json!(["a", "b"]));
    run(r#"for character of holder.text:
    seen.push(character)"#);
    assert_eq!(run("seen"), serde_json::json!(["a", "b", "c", "d"]));
    assert_eq!(run("holder.text"), "cd");
}

/// Nucleoid keeps string iterator scopes and nested loops independent
#[rustfmt::skip]
#[test]
fn keeps_string_iterator_scopes_and_nested_loops_independent() {
    let mut run = runner();
    run(r#"it = "outer""#);
    run(r#"letters = "ab""#);
    run(r#"digits = "12""#);
    run("seen = []");
    run(r#"for letters:
    seen.push(it)
    for digits:
        seen.push(it)
    seen.push(it)"#);
    assert_eq!(run("seen"), serde_json::json!(["a", "1", "2", "a", "b", "1", "2", "b"]));
    assert_eq!(run("it"), "outer");
}

/// Nucleoid skips empty strings without executing loop bodies
#[rustfmt::skip]
#[test]
fn skips_empty_strings_without_executing_loop_bodies() {
    let mut run = runner();
    run(r#"empty = """#);
    run("seen = []");
    run(r#"for character in empty:
    seen.push(character)"#);
    assert_eq!(run("seen"), serde_json::json!([]));
    run(r#"full = "ab""#);
    run(r#"for character of full:
    seen.push(character)"#);
    assert_eq!(run("seen"), serde_json::json!(["a", "b"]));
    run(r#"for character in empty:
    seen.push(character)"#);
    assert_eq!(run("seen"), serde_json::json!(["a", "b"]));
}

/// Nucleoid rolls back failed string iteration and restores caller scope
#[rustfmt::skip]
#[test]
fn rolls_back_failed_string_iteration_and_restores_caller_scope() {
    let mut run = runner();
    run(r#"it = "outer"
seen = ["saved"]
try:
    for it of "abc":
        seen.push(it)
        if it == "b":
            throw "STOP_STRING_ITERATION"
catch error:
    assert(error, "STOP_STRING_ITERATION")
assert(seen, ["saved"])
assert(it, "outer")
for it of "c":
    seen.push(it)
assert(seen, ["saved", "c"])
assert(it, "outer")"#);
}

/// Nucleoid reads zero based and named versions of instance properties
#[rustfmt::skip]
#[test]
fn reads_zero_based_and_named_versions_of_instance_properties() {
    let mut run = runner();
    run("class Student");
    run("student1 = Student()");
    run(r#"student1.name = "Can""#);
    run(r#"student1.name = "Daphne""#);
    run(r#"student1.name = "Merve""#);
    run("version0 = student1.name@0");
    run("version1 = student1.name@1");
    run("version2 = student1.name@2");
    run("firstName = student1.name@first");
    run("lastName = student1.name@last");
    run("previousName = student1.name@previous");
    assert_eq!(run("version0"), "Can");
    assert_eq!(run("version1"), "Daphne");
    assert_eq!(run("version2"), "Merve");
    assert_eq!(run("firstName"), "Can");
    assert_eq!(run("lastName"), "Merve");
    assert_eq!(run("previousName"), "Daphne");
    assert_eq!(run("student1.name"), "Merve");
    run(r#"[version0, version1, version2, firstName, lastName, previousName]

# return: ["Can","Daphne","Merve","Can","Merve","Daphne"]"#);
}

/// Nucleoid tracks reactive variable versions without duplicate states
#[rustfmt::skip]
#[test]
fn tracks_reactive_variable_versions_without_duplicate_states() {
    let mut run = runner();
    run("input = 1");
    run("amount = input * 2");
    run("firstAmount = amount@first");
    run("lastAmount = amount@last");
    run("previousAmount = amount@previous");
    assert_eq!(run("firstAmount"), 2);
    assert_eq!(run("lastAmount"), 2);
    assert_eq!(run("previousAmount"), serde_json::Value::Null);
    run("input = 2");
    assert_eq!(run("amount"), 4);
    assert_eq!(run("lastAmount"), 4);
    assert_eq!(run("previousAmount"), 2);
    run("input = 2");
    assert_eq!(run("firstAmount"), 2);
    assert_eq!(run("lastAmount"), 4);
    assert_eq!(run("previousAmount"), 2);
    run("input = 3");
    assert_eq!(run("lastAmount"), 6);
    assert_eq!(run("previousAmount"), 4);
    assert_eq!(run("firstAmount"), 2);
    run("version0 = amount@0");
    run("version1 = amount@1");
    run("version2 = amount@2");
    run("version3 = amount@3");
    assert_eq!(run("version0"), 2);
    assert_eq!(run("version1"), 4);
    assert_eq!(run("version2"), 6);
    assert_eq!(run("version3"), serde_json::Value::Null);
}

/// Nucleoid reads versions through nulls and deferred indices
#[rustfmt::skip]
#[test]
fn reads_versions_through_nulls_and_deferred_indices() {
    let mut run = runner();
    run(r#"value = "ready""#);
    run("oldest = value@first");
    run("future = value@2");
    run("latest = value@last");
    run("previous = value@previous");
    assert_eq!(run("oldest"), "ready");
    assert_eq!(run("future"), serde_json::Value::Null);
    assert_eq!(run("previous"), serde_json::Value::Null);
    run("value = null");
    assert_eq!(run("oldest"), "ready");
    assert_eq!(run("latest"), serde_json::Value::Null);
    assert_eq!(run("previous"), "ready");
    assert_eq!(run("future"), serde_json::Value::Null);
    run(r#"value = "back""#);
    assert_eq!(run("oldest"), "ready");
    assert_eq!(run("latest"), "back");
    assert_eq!(run("previous"), serde_json::Value::Null);
    assert_eq!(run("future"), "back");
}

/// Nucleoid isolates local and parameter version histories between calls
#[rustfmt::skip]
#[test]
fn isolates_local_and_parameter_version_histories_between_calls() {
    let mut run = runner();
    run(r#"value = 99
def capture(value) => [value@first, value = value + 1, value@previous]
one = capture(2)
two = capture(10)
assert(one, [2, 3, 2])
assert(two, [10, 11, 10])
globalFirst = value.value
assert(globalFirst, 99)"#);
}

/// Nucleoid preserves list values object identity and ordinary fields in versions
#[rustfmt::skip]
#[test]
fn preserves_list_values_object_identity_and_ordinary_fields_in_versions() {
    let mut run = runner();
    run("items = [1]");
    run("items.push(2)");
    run("earlierList = items@first");
    run("latestList = items@last");
    run("previousList = items@previous");
    assert_eq!(run("earlierList"), serde_json::json!([1]));
    assert_eq!(run("latestList"), serde_json::json!([1, 2]));
    assert_eq!(run("previousList"), serde_json::json!([1]));
    run("class Record");
    run("first = Record()");
    run(r#"first.first = "given""#);
    run(r#"first.last = "family""#);
    run(r#"first.previous = "ordinary""#);
    run("second = Record()");
    run("selected = first");
    run("selected = second");
    run("earlierObject = selected@first");
    run("latestObject = selected@last");
    run("previousObject = selected@previous");
    run(r#"first.label = "updated""#);
    assert_eq!(run("earlierObject.label"), "updated");
    {
        let actual = run("latestObject");
        let expected = run("(second)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("previousObject");
        let expected = run("(first)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("first.first"), "given");
    assert_eq!(run("first.last"), "family");
    assert_eq!(run("first.previous"), "ordinary");
}

/// Nucleoid restores version histories on rollback and resets deleted bindings
#[rustfmt::skip]
#[test]
fn restores_version_histories_on_rollback_and_resets_deleted_bindings() {
    let mut run = runner();
    run(r#"value = "A"
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
assert(previousValue, null)"#);
}

/// Nucleoid fills omitted literal default parameters
#[rustfmt::skip]
#[test]
fn fills_omitted_literal_default_parameters() {
    let mut run = runner();
    run(r#"def price(quantity, unit = 10) => quantity * unit
one = price(1)
two = price(2)
three = price(3)
custom = price(2, 7)
assert(one, 10)
assert(two, 20)
assert(three, 30)
assert(custom, 14)"#);
}

/// Nucleoid preserves supplied falsy arguments when defaults exist
#[rustfmt::skip]
#[test]
fn preserves_supplied_falsy_arguments_when_defaults_exist() {
    let mut run = runner();
    run(r#"def pick(tag, fallback = 12) => [tag, fallback]
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
assert(nullC, ["c", null])"#);
}

/// Nucleoid applies literal defaults to typed functions and parenthesized arrows
#[rustfmt::skip]
#[test]
fn applies_literal_defaults_to_typed_functions_and_parenthesized_arrows() {
    let mut run = runner();
    run(r#"def scale(value: Number, factor: Number = 0.5) => value * factor
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
assert(six, 3)"#);
}

/// Nucleoid starts defaulted parameter histories at their effective values
#[rustfmt::skip]
#[test]
fn starts_defaulted_parameter_histories_at_their_effective_values() {
    let mut run = runner();
    run(r#"step = 100
def sample(seed, step = 4) => [seed, step = step + 1, step@first, step@last]
one = sample(1)
two = sample(2)
three = sample(3)
provided = sample(4, 9)
assert(one, [1, 5, 4, 5])
assert(two, [2, 5, 4, 5])
assert(three, [3, 5, 4, 5])
assert(provided, [4, 10, 9, 10])
assert(step, 100)"#);
}

/// Nucleoid binds independent literal constructor and method defaults
#[rustfmt::skip]
#[test]
fn binds_independent_literal_constructor_and_method_defaults() {
    let mut run = runner();
    run(r#"class Counter(seed = 1):
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
assert(custom, 9)"#);
}

/// Nucleoid restores defaulted function callers after failed redefinitions
#[rustfmt::skip]
#[test]
fn restores_defaulted_function_callers_after_failed_redefinitions() {
    let mut run = runner();
    run(r#"def price(quantity, unit = 10) => quantity * unit
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
assert(four, 40)"#);
}

/// Nucleoid anchors class conditionals to each instances first property values
#[rustfmt::skip]
#[test]
fn anchors_class_conditionals_to_each_instances_first_property_values() {
    let mut run = runner();
    run(r#"class Obj(x: int, y: int, orientation: int):
    this.x = x
    this.y = y
    this.orientation = orientation"#);
    run("first = Obj(10, 20, 180)");
    run("first.x = 70");
    run("first.y = 80");
    run(r#"if $Obj.orientation == 180:
    $Obj.tag = "l4214"
    $Obj.x = $Obj.x@first - 1
    $Obj.y = $Obj.y@first - 1"#);
    run("second = Obj(30, 40, 180)");
    run("waiting = Obj(50, 60, 0)");
    assert_eq!(run("[first.x, first.y, first.tag]"), serde_json::json!([9, 19, "l4214"]));
    assert_eq!(run("[second.x, second.y, second.tag]"), serde_json::json!([29, 39, "l4214"]));
    assert_eq!(run("[waiting.x, waiting.y, waiting.tag]"), serde_json::json!([50, 60, null]));
    run("waiting.x = 90");
    run("waiting.orientation = 180");
    assert_eq!(run("[waiting.x, waiting.y, waiting.tag]"), serde_json::json!([49, 59, "l4214"]));
    run("first.orientation = 0");
    run("first.orientation = 180");
    run("second.orientation = 180");
    assert_eq!(run("[first.x, first.y, second.x, second.y]"), serde_json::json!([9, 19, 29, 39]));
    assert_eq!(run("first.x@first"), 10);
    assert_eq!(run("first.x@0"), 10);
    assert_eq!(run("second.x@first"), 30);
    assert_eq!(run("second.x@2"), serde_json::Value::Null);
    run(r#"[first.x, first.y, second.x, second.y]

# return: [9,19,29,39]"#);
}

/// Nucleoid anchors ordinary class rules with fixed numeric and first selectors
#[rustfmt::skip]
#[test]
fn anchors_ordinary_class_rules_with_fixed_numeric_and_first_selectors() {
    let mut run = runner();
    run(r#"class Counter(seed):
    this.x = seed
    this.x = seed + 10
    this.y = seed * 2"#);
    run("before = Counter(10)");
    run("step = 1");
    run("$Counter.x = $Counter.x@first - step");
    run(r#"$Counter.y = $Counter["y"]@0 - step"#);
    run("$Counter.latest = $Counter.x@last");
    run("$Counter.prior = $Counter.x@previous");
    assert_eq!(run("[before.x, before.y, before.latest, before.prior]"), serde_json::json!([9, 19, 9, 20]));
    run("step = 2");
    assert_eq!(run("[before.x, before.y, before.latest, before.prior]"), serde_json::json!([8, 18, 8, 9]));
    run("$Counter.x = $Counter.x@1 - step");
    run("after = Counter(30)");
    assert_eq!(run("[before.x, after.x, after.y]"), serde_json::json!([18, 38, 58]));
    assert_eq!(run("[after.x@first, after.x@0, after.x@1]"), serde_json::json!([30, 30, 40]));
    run("step = 3");
    assert_eq!(run("[before.x, after.x, before.y, after.y]"), serde_json::json!([17, 37, 17, 57]));
    assert_eq!(run("[before.latest, after.latest, before.prior, after.prior]"), serde_json::json!([17, 37, 18, 38]));
    run(r#"[before.x, after.x, before.y, after.y]

# return: [17,37,17,57]"#);
}

/// Nucleoid defers missing self anchors and recovers recreated property slots
#[rustfmt::skip]
#[test]
fn defers_missing_self_anchors_and_recovers_recreated_property_slots() {
    let mut run = runner();
    run(r#"class Pending(orientation):
    this.orientation = orientation"#);
    run(r#"if $Pending.orientation == 180:
    $Pending.tag = "l4214"
    $Pending.x = $Pending.x@first - 1"#);
    run("item = Pending(180)");
    run("original = item.x@first");
    assert_eq!(run("item.tag"), "l4214");
    assert_eq!(run("item.x"), serde_json::Value::Null);
    assert_eq!(run("original"), serde_json::Value::Null);
    run("item.x = 10");
    assert_eq!(run("item.x"), 9);
    assert_eq!(run("item.x@first"), 10);
    assert_eq!(run("item.x@previous"), 10);
    assert_eq!(run("original"), 10);
    run("delete item.x");
    assert_eq!(run("item.x"), serde_json::Value::Null);
    assert_eq!(run("item.x@first"), serde_json::Value::Null);
    assert_eq!(run("original"), serde_json::Value::Null);
    run("item.x = 40");
    assert_eq!(run("item.x"), 39);
    assert_eq!(run("item.x@first"), 40);
    assert_eq!(run("item.x@0"), 40);
    assert_eq!(run("original"), 40);
    run("item.orientation = 180");
    assert_eq!(run("item.x"), 39);
    assert_eq!(run("item.x@2"), serde_json::Value::Null);
    run(r#"[item.x, item.x@first, original]

# return: [39,40,40]"#);
}

/// Nucleoid waits for actual history entries before using numeric self anchors
#[rustfmt::skip]
#[test]
fn waits_for_actual_history_entries_before_using_numeric_self_anchors() {
    let mut run = runner();
    run(r#"class Sample(seed):
    this.x = seed
    this.enabled = true"#);
    run(r#"if $Sample.enabled:
    $Sample.x = $Sample.x@1 - 1"#);
    run("item = Sample(10)");
    assert_eq!(run("item.x"), 10);
    assert_eq!(run("item.x@first"), 10);
    assert_eq!(run("item.x@1"), serde_json::Value::Null);
    run("item.enabled = true");
    assert_eq!(run("item.x"), 10);
    assert_eq!(run("item.x@1"), serde_json::Value::Null);
    run("item.x = 20");
    assert_eq!(run("item.x"), 19);
    assert_eq!(run("item.x@first"), 10);
    assert_eq!(run("item.x@1"), 20);
    run("item.enabled = false");
    run("item.enabled = true");
    assert_eq!(run("item.x"), 19);
    assert_eq!(run("item.x@3"), serde_json::Value::Null);
}

/// Nucleoid anchors inherited class receivers independently
#[rustfmt::skip]
#[test]
fn anchors_inherited_class_receivers_independently() {
    let mut run = runner();
    run(r#"class Base(seed):
    this.x = seed
    this.orientation = 180"#);
    run(r#"class Child: Base
    def init(seed):
        super(seed)"#);
    run("step = 1");
    run("$Base.x = $Base.x@0 - step");
    run(r#"if $Base.orientation == 180:
    $Base.tag = "l4214"
    $Base.x = $Base.x@first - step"#);
    run("parent = Base(10)");
    run("child = Child(30)");
    assert_eq!(run("[parent.x, child.x]"), serde_json::json!([9, 29]));
    assert_eq!(run("[parent.tag, child.tag]"), serde_json::json!(["l4214", "l4214"]));
    assert_eq!(run("[parent.x@first, child.x@first]"), serde_json::json!([10, 30]));
    run("step = 2");
    assert_eq!(run("[parent.x, child.x]"), serde_json::json!([8, 28]));
    run("child.orientation = 0");
    run("child.orientation = 180");
    assert_eq!(run("child.x"), 28);
    assert_eq!(run("child.x@3"), serde_json::Value::Null);
}

/// Nucleoid restores class anchors histories and declarations on caught rollback
#[rustfmt::skip]
#[test]
fn restores_class_anchors_histories_and_declarations_on_caught_rollback() {
    let mut run = runner();
    run(r#"class Sample(seed):
    this.x = seed
    this.enabled = true
if $Sample.enabled:
    $Sample.x = $Sample.x@first - 1
item = Sample(10)
original = item.x@first
try:
    delete item.x
    item.x = 50
    $Sample.extra = $Sample.x@first
    throw "RESTORE_ANCHOR"
catch error:
    assert(error, "RESTORE_ANCHOR")
assert(item.x, 9)
assert(item.x@first, 10)
assert(item.x@previous, 10)
assert(item.x@2, null)
assert(item.extra, null)
assert(original, 10)
item.enabled = true
assert(item.x, 9)
delete item.x
item.x = 30
assert(item.x, 29)
assert(item.x@first, 30)
assert(original, 30)
[item.x, original]
# return: [29,30]"#);
}

/// Nucleoid protects direct class history cycles and read only destinations
#[rustfmt::skip]
#[test]
fn protects_direct_class_history_cycles_and_read_only_destinations() {
    let mut run = runner();
    run(r#"class Sample:
    this.x = 10
    this.items = [1, 2]
    this.enabled = true
item = Sample()
try:
    $Sample.x = $Sample.x@last - 1
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.x@previous - 1
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.x@first + $Sample.x
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x@first = 4
catch error:
    assert(error, TypeError("State versions are read-only"))
try:
    delete $Sample.x@first
catch error:
    assert(error, TypeError("State versions are read-only"))
try:
    if $Sample.enabled:
        $Sample.items@first.push(3)
catch error:
    assert(error, TypeError("State versions are read-only"))
try:
    $Sample.bad = Math.PI@first
catch error:
    assert(error, TypeError("State versions require a variable or object property"))
class Empty
$Empty.x = $Empty.x@first.value
try:
    absent = Empty()
catch error:
    assert(error, ReferenceError("$Empty.x@first is not defined"))
assert(item.x, 10)
assert(item.x@first, 10)
assert(item.x@1, null)
assert(item.items, [1, 2])
assert(item.items@1, null)"#);
}

/// Nucleoid preserves moving class conditionals beside fixed first anchors
#[rustfmt::skip]
#[test]
fn preserves_moving_class_conditionals_beside_fixed_first_anchors() {
    let mut run = runner();
    run(r#"class Counter(seed):
    this.x = seed
    this.enabled = false"#);
    run(r#"if $Counter.enabled:
    $Counter.x = $Counter.x@last - 1"#);
    run("item = Counter(10)");
    run("item.enabled = true");
    assert_eq!(run("item.x"), 9);
    run("item.enabled = false");
    run("item.enabled = true");
    assert_eq!(run("item.x"), 8);
    assert_eq!(run("item.x@first"), 10);
    assert_eq!(run("item.x@previous"), 9);
    run(r#"class Anchor(seed):
    this.x = seed
    this.y = seed
    this.enabled = false"#);
    run(r#"if $Anchor.enabled:
    $Anchor.x = $Anchor.x@first - 1
    $Anchor.y = $Anchor.y@0 - 1"#);
    run("anchored = Anchor(10)");
    run("anchored.enabled = true");
    assert_eq!(run("[anchored.x, anchored.y]"), serde_json::json!([9, 9]));
    run("anchored.enabled = false");
    run("anchored.enabled = true");
    assert_eq!(run("[anchored.x, anchored.y]"), serde_json::json!([9, 9]));
    assert_eq!(run("[anchored.x@first, anchored.y@first]"), serde_json::json!([10, 10]));
    assert_eq!(run("[anchored.x@2, anchored.y@2]"), serde_json::json!([null, null]));
    run(r#"class Previous(seed):
    this.x = seed
    this.x = seed + 10
    this.enabled = false"#);
    run(r#"if $Previous.enabled:
    $Previous.x = $Previous.x@previous - 1"#);
    run("previous = Previous(10)");
    assert_eq!(run("previous.x@previous"), 10);
    run("previous.enabled = true");
    assert_eq!(run("previous.x"), 9);
    assert_eq!(run("previous.x@previous"), 20);
    run("previous.enabled = false");
    run("previous.enabled = true");
    assert_eq!(run("previous.x"), 19);
    assert_eq!(run("previous.x@first"), 10);
    assert_eq!(run("previous.x@1"), 20);
    assert_eq!(run("previous.x@2"), 9);
    assert_eq!(run("previous.x@previous"), 9);
    assert_eq!(run("previous.x@3"), 19);
    assert_eq!(run("previous.x@4"), serde_json::Value::Null);
    run(r#"[item.x, anchored.x, anchored.y, previous.x, previous.x@previous]

# return: [8,9,9,19,9]"#);
}

/// Nucleoid keeps base two exponentiation reactive
#[rustfmt::skip]
#[test]
fn keeps_base_two_exponentiation_reactive() {
    let mut run = runner();
    run("power = 2");
    run("converted = Math.exp2(power)");
    assert_eq!(run("converted"), 4);
    run("power = 3");
    assert_eq!(run("converted"), 8);
    run("power = -2");
    assert_eq!(run("converted"), 0.25);
    run("power = 0");
    assert_eq!(run("converted"), 1);
    run("power = -0");
    assert_eq!(run("converted"), 1);
    run("power = 4");
    assert_eq!(run("converted"), 16);
    run(r#"converted

# return: 16"#);
}

/// Nucleoid preserves base two exponential coercion and omissions
#[rustfmt::skip]
#[test]
fn preserves_base_two_exponential_coercion_and_omissions() {
    let mut run = runner();
    run(r#"text = " 3 ""#);
    run("converted = Math.exp2(text)");
    assert_eq!(run("converted"), 8);
    run("text = true");
    assert_eq!(run("converted"), 2);
    run("text = false");
    assert_eq!(run("converted"), 1);
    run("text = []");
    assert_eq!(run("converted"), 1);
    run(r#"text = ["-1"]"#);
    assert_eq!(run("converted"), 0.5);
    run("text = Date(2)");
    assert_eq!(run("converted"), 4);
    run("text = [1, 2]");
    assert_eq!(run("Number.isNaN(converted)"), true);
    run(r#"text = "invalid""#);
    assert_eq!(run("Number.isNaN(converted)"), true);
    run("literal = Math.exp2(null)");
    assert_eq!(run("literal"), 1);
    run("omitted = Math.exp2()");
    assert_eq!(run("Number.isNaN(omitted)"), true);
}

/// Nucleoid preserves base two exponential extremes and underflow
#[rustfmt::skip]
#[test]
fn preserves_base_two_exponential_extremes_and_underflow() {
    let mut run = runner();
    run("power = 1023");
    run("converted = Math.exp2(power)");
    assert_eq!(run("Number.isFinite(converted)"), true);
    run("power = 1024");
    {
        let actual = run("converted");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("power = -1074");
    assert_eq!(run("converted"), 5e-324);
    run("power = -1075");
    assert_eq!(run("converted"), 0);
    {
        let actual = run("1 / converted");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("power = Number.POSITIVE_INFINITY");
    {
        let actual = run("converted");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("power = Number.NEGATIVE_INFINITY");
    assert_eq!(run("converted"), 0);
    run("power = Number.NaN");
    assert_eq!(run("Number.isNaN(converted)"), true);
}

/// Nucleoid applies base two exponentiation to independent class rules
#[rustfmt::skip]
#[test]
fn applies_base_two_exponentiation_to_independent_class_rules() {
    let mut run = runner();
    run(r#"class Exponent(power):
    this.power = power"#);
    run("$Exponent.converted = Math.exp2($Exponent.power)");
    run("first = Exponent(2)");
    run("second = Exponent(-1)");
    assert_eq!(run("first.converted"), 4);
    assert_eq!(run("second.converted"), 0.5);
    run("first.power = 3");
    assert_eq!(run("first.converted"), 8);
    assert_eq!(run("second.converted"), 0.5);
    run("delete first.power");
    assert_eq!(run("first.converted"), serde_json::Value::Null);
    run("first.power = -2");
    assert_eq!(run("first.converted"), 0.25);
    run("second.power = 4");
    assert_eq!(run("second.converted"), 16);
}

/// Nucleoid restores base two exponential relationships after failed transactions
#[rustfmt::skip]
#[test]
fn restores_base_two_exponential_relationships_after_failed_transactions() {
    let mut run = runner();
    run(r#"class Dial
dial = Dial()
dial.power = 2
converted = Math.exp2(dial.power)
failure = ""
try:
    dial.power = 3
    throw "EXP2_ROLLBACK"
catch error:
    failure = error
assert(failure, "EXP2_ROLLBACK")
assert(dial.power, 2)
assert(converted, 4)
dial.power = -1
assert(converted, 0.5)"#);
}

/// Nucleoid preserves ignored exponential reads and missing extras
#[rustfmt::skip]
#[test]
fn preserves_ignored_exponential_reads_and_missing_extras() {
    let mut run = runner();
    run("class Probe");
    run("probe = Probe()");
    run("probe.power = 2");
    run("extra = 9");
    run("converted = Math.exp2(probe.power, extra)");
    run("withMissing = Math.exp2(probe.power, probe.missing)");
    assert_eq!(run("converted"), 4);
    assert_eq!(run("withMissing"), serde_json::Value::Null);
    run("probe.missing = 7");
    assert_eq!(run("withMissing"), 4);
    run(r#"extra = "ignored""#);
    assert_eq!(run("converted"), 4);
    run("delete probe.power");
    assert_eq!(run("converted"), serde_json::Value::Null);
    assert_eq!(run("withMissing"), serde_json::Value::Null);
    run("probe.power = -1");
    assert_eq!(run("converted"), 0.5);
    assert_eq!(run("withMissing"), 0.5);
    run("delete probe.missing");
    assert_eq!(run("withMissing"), serde_json::Value::Null);
    run("probe.missing = 2");
    assert_eq!(run("withMissing"), 0.5);
}

/// Nucleoid accepts thin arrows wherever fat arrows are accepted
#[rustfmt::skip]
#[test]
fn accepts_thin_arrows_wherever_fat_arrows_are_accepted() {
    let mut run = runner();
    run("offset = 1");
    run("items = [1, 2, 3]");
    run("doubled = items.map(item -> item * 2)");
    run("large = items.filter((item: int) -> item > offset)");
    run("found = items.find(item -> { return item == 2 })");
    run("def shift(value) -> value + offset");
    run("def double amount -> amount * 2");
    run("def increment amount: int -> amount + 1");
    run("def area width, height -> width * height");
    run("def mixed left: int, right -> left - right");
    run("def scaled left right factor -> (left + right) * factor");
    run(r#"def tripled value ->
    value * 3"#);
    assert_eq!(run("doubled"), serde_json::json!([2, 4, 6]));
    assert_eq!(run("large"), serde_json::json!([2, 3]));
    assert_eq!(run("found"), 2);
    assert_eq!(run("shift(2)"), 3);
    assert_eq!(run("double(4)"), 8);
    assert_eq!(run("increment(4)"), 5);
    assert_eq!(run("area(3, 4)"), 12);
    assert_eq!(run("mixed(5, 2)"), 3);
    assert_eq!(run("scaled(1, 2, 3)"), 9);
    assert_eq!(run("tripled(2)"), 6);
    run("offset = 2");
    assert_eq!(run("large"), serde_json::json!([3]));
    assert_eq!(run("shift(2)"), 4);
}

/// Nucleoid keeps thin arrow methods and class queries reactive
#[rustfmt::skip]
#[test]
fn keeps_thin_arrow_methods_and_class_queries_reactive() {
    let mut run = runner();
    run(r#"class Gauge:
    def init amount -> this.amount = amount
    def reading() -> self.amount
    def scaled factor -> self.amount * factor"#);
    run("gauge = Gauge(4)");
    assert_eq!(run("gauge.reading()"), 4);
    assert_eq!(run("gauge.scaled(3)"), 12);
    run(r#"class Reading(level: int):
    this.level = level"#);
    run("Reading(4); Reading(9); Reading(15)");
    run("threshold = 5");
    run("high = Reading.filter(reading -> reading.level > threshold)");
    assert_eq!(run("high.length"), 2);
    run("threshold = 10");
    assert_eq!(run("high.length"), 1);
    run("Reading(20)");
    assert_eq!(run("high.length"), 2);
}

/// Nucleoid defines Python style lambdas
#[rustfmt::skip]
#[test]
fn defines_python_style_lambdas() {
    let mut run = runner();
    run("scale = 3");
    run("items = [1, 2, 3]");
    run("tripled = items.map(lambda item: item * scale)");
    run("total = items.reduce(lambda sum, item: sum + item, 0)");
    run("zero = lambda: 0");
    run("add = lambda left, right: left + right");
    run("pair = lambda left, right: {left: left, right: right}");
    assert_eq!(run("tripled"), serde_json::json!([3, 6, 9]));
    assert_eq!(run("total"), 6);
    assert_eq!(run("zero()"), 0);
    assert_eq!(run("add(2, 3)"), 5);
    assert_eq!(run("pair(1, 2).right"), 2);
    assert_eq!(run("(lambda value: value * 2)(4)"), 8);
    run("scale = 2");
    assert_eq!(run("tripled"), serde_json::json!([2, 4, 6]));
    run("items.push(4)");
    assert_eq!(run("tripled"), serde_json::json!([2, 4, 6, 8]));
    assert_eq!(run("total"), 10);
}

/// Nucleoid keeps lambda an ordinary name where accepted syntax owns the colon
#[rustfmt::skip]
#[test]
fn keeps_lambda_an_ordinary_name_where_accepted_syntax_owns_the_colon() {
    let mut run = runner();
    run("lambda = 1");
    run("values = [10, 20, 30]");
    assert_eq!(run("values[lambda:]"), serde_json::json!([20, 30]));
    run("record = { first: lambda next: 2 }");
    assert_eq!(run("record.first"), 1);
    assert_eq!(run("record.next"), 2);
    run("keyed = { lambda: 5 }");
    assert_eq!(run("keyed.lambda"), 5);
    run("short = { lambda }");
    assert_eq!(run("short.lambda"), 1);
    run(r#"if lambda:
    state = "on""#);
    assert_eq!(run("state"), "on");
    run("alias = lambda assert: alias, 1");
    run("double = lambda -> lambda * 2");
    assert_eq!(run("double(4)"), 8);
    run("lambda = 2");
    assert_eq!(run("values[lambda:]"), serde_json::json!([30]));
    assert_eq!(run("record.first"), 2);
    assert_eq!(run("alias"), 2);
}

/// Nucleoid rolls back failed lambda and thin arrow updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_lambda_and_thin_arrow_updates() {
    let mut run = runner();
    run(r#"rate = 2
items = [1, 2]
scaled = items.map(lambda item: item * rate)
def fee amount -> amount * rate
charge = fee(10)
failure = ""
try:
    rate = 3
    throw "ROLLBACK"
catch error:
    failure = error
assert(failure, "ROLLBACK")
assert(rate, 2)
assert(scaled, [2, 4])
assert(charge, 20)
rate = 5
assert(scaled, [5, 10])
assert(charge, 50)"#);
}

/// Nucleoid sorts a list in place like JavaScript
#[rustfmt::skip]
#[test]
fn sorts_a_list_in_place_like_javascript() {
    let mut run = runner();
    run("scores = [30, 10, 20]");
    run("first = scores[0]");
    run("scores.sort((a, b) => a - b)");
    assert_eq!(run("scores"), serde_json::json!([10, 20, 30]));
    assert_eq!(run("first"), 10);
    assert_eq!(run("scores.sort((a, b) => b - a)"), serde_json::json!([30, 20, 10]));
    assert_eq!(run("scores"), serde_json::json!([30, 20, 10]));
    assert_eq!(run("first"), 30);
    run("codes = [10, 9, 1]");
    run("codes.sort()");
    assert_eq!(run("codes"), serde_json::json!([1, 10, 9]));
    run(r#"names = ["b", "a", "C"]"#);
    run("names.sort(null)");
    assert_eq!(run("names"), serde_json::json!(["C", "a", "b"]));
    run(r#"people = [{ name: "a", age: 30 }, { name: "b", age: 20 }, { name: "c", age: 30 }, { name: "d", age: 20 }]"#);
    run("people.sort((x, y) => x.age - y.age)");
    assert_eq!(run("people.map(person => person.name)"), serde_json::json!(["b", "d", "a", "c"]));
    run(r#"scores

# return: [30,20,10]"#);
}

/// Nucleoid sorts properties and instance members in place
#[rustfmt::skip]
#[test]
fn sorts_properties_and_instance_members_in_place() {
    let mut run = runner();
    run("board = { entries: [3, 1, 2] }");
    run("board.entries.sort((a, b) => b - a)");
    assert_eq!(run("board.entries"), serde_json::json!([3, 2, 1]));
    run("calls = []");
    run(r#"def pick():
    calls.push(1)
    return board"#);
    run("pick().entries.sort()");
    assert_eq!(run("calls.length"), 1);
    assert_eq!(run("board.entries"), serde_json::json!([1, 2, 3]));
    run(r#"class Team:
    def init(scores) => this.scores = scores
    def ranked() => self.scores.sort((a, b) => b - a)"#);
    run("team = Team([2, 9, 4])");
    assert_eq!(run("team.ranked()"), serde_json::json!([9, 4, 2]));
    assert_eq!(run("team.scores"), serde_json::json!([9, 4, 2]));
    run(r#"class Squad(scores):
    this.scores = scores"#);
    run("$Squad.best = $Squad.scores.sort((a, b) => b - a)[0]");
    run("red = Squad([3, 1, 2])");
    run("blue = Squad([5, 9])");
    assert_eq!(run("red.scores"), serde_json::json!([3, 2, 1]));
    assert_eq!(run("blue.scores"), serde_json::json!([9, 5]));
    run("red.scores.push(7)");
    assert_eq!(run("red.scores"), serde_json::json!([7, 3, 2, 1]));
    assert_eq!(run("red.best"), 7);
    run("older = { items: [3, 1, 2] }");
    run("newer = { items: [9, 8] }");
    run("picked = older");
    run("picked = newer");
    run("picked@first.items.sort()");
    assert_eq!(run("older.items"), serde_json::json!([1, 2, 3]));
    assert_eq!(run("newer.items"), serde_json::json!([9, 8]));
}

/// Nucleoid returns sorted copies from receivers without a list place
#[rustfmt::skip]
#[test]
fn returns_sorted_copies_from_receivers_without_a_list_place() {
    let mut run = runner();
    run(r#"scores = [30, 10, 20]
ranked = scores.slice().sort((a, b) => a - b)
assert(ranked, [10, 20, 30])
assert(scores, [30, 10, 20])
assert([3, 1, 2].sort(), [1, 2, 3])
grid = [[3, 1], [2, 0]]
assert(grid[0].sort(), [1, 3])
assert(grid, [[3, 1], [2, 0]])
record = { items: [2, 1] }
assert(record["items"].sort(), [1, 2])
assert(record.items, [2, 1])
snapshot = scores.value.sort()
assert(snapshot, [10, 20, 30])
scores = [5, 4]
oldest = scores@first.sort()
assert(oldest, [10, 20, 30])
assert(scores@first, [30, 10, 20])
assert(scores@last.sort(), [4, 5])
assert(scores, [5, 4])
assert(ranked, [4, 5])
assert(snapshot, [10, 20, 30])
try:
    scores@first.push(1)
catch error:
    assert(error, TypeError("State versions are read-only"))
class Item(size):
    this.size = size
Item(2); Item(1); Item(3)
smallest = Item.sort((x, y) => x.size - y.size)[0].size
assert(smallest, 1)
assert(Item.map(item => item.size), [2, 1, 3])
Item(0)
assert(smallest, 0)
assert(Item[0].size, 2)"#);
}

/// Nucleoid sorts only the local binding of a copied list
#[rustfmt::skip]
#[test]
fn sorts_only_the_local_binding_of_a_copied_list() {
    let mut run = runner();
    run(r#"def ordered(values):
    values.sort()
    return values"#);
    run("original = [3, 1, 2]");
    assert_eq!(run("ordered(original)"), serde_json::json!([1, 2, 3]));
    assert_eq!(run("original"), serde_json::json!([3, 1, 2]));
    run("rows = [[3, 1], [2, 0]]");
    run(r#"for row of rows:
    row.sort()"#);
    assert_eq!(run("rows"), serde_json::json!([[3, 1], [2, 0]]));
}

/// Nucleoid keeps a list sorted with a standing sort
#[rustfmt::skip]
#[test]
fn keeps_a_list_sorted_with_a_standing_sort() {
    let mut run = runner();
    run(r#"class Player(name, score):
    this.name = name
    this.score = score"#);
    run(r#"alice = Player("alice", 30)"#);
    run(r#"bob = Player("bob", 10)"#);
    run(r#"cara = Player("cara", 20)"#);
    run("players = [alice, bob, cara]");
    run("direction = 1");
    run("ranked = players.sort((a, b) => (a.score - b.score) * direction)");
    run("names = players.map(player => player.name)");
    assert_eq!(run("names"), serde_json::json!(["bob", "cara", "alice"]));
    {
        let actual = run("ranked");
        let expected = run("(players)");
        assert_eq!(actual, expected);
    }
    run(r#"dave = Player("dave", 5)"#);
    run("players.push(dave)");
    assert_eq!(run("names"), serde_json::json!(["dave", "bob", "cara", "alice"]));
    run("bob.score = 40");
    assert_eq!(run("names"), serde_json::json!(["dave", "cara", "alice", "bob"]));
    run("direction = -1");
    assert_eq!(run("names"), serde_json::json!(["bob", "alice", "cara", "dave"]));
    run("players = [cara, alice]");
    assert_eq!(run("names"), serde_json::json!(["alice", "cara"]));
    {
        let actual = run("ranked");
        let expected = run("(players)");
        assert_eq!(actual, expected);
    }
    run("values = [3, 1, 2]");
    run("values = values.sort()");
    run("values.push(0)");
    assert_eq!(run("values"), serde_json::json!([1, 2, 3, 0]));
    run(r#"names

# return: ["alice","cara"]"#);
}

/// Nucleoid records in place sorts as list versions
#[rustfmt::skip]
#[test]
fn records_in_place_sorts_as_list_versions() {
    let mut run = runner();
    run("scores = [3, 1, 2]");
    run("log = []");
    run(r#"watched = scores.length + log.push("run") * 0"#);
    assert_eq!(run("log.length"), 1);
    run("scores.sort()");
    assert_eq!(run("scores@0"), serde_json::json!([3, 1, 2]));
    assert_eq!(run("scores@1"), serde_json::json!([1, 2, 3]));
    assert_eq!(run("log.length"), 2);
    run("scores.sort()");
    assert_eq!(run("scores@2"), serde_json::Value::Null);
    assert_eq!(run("log.length"), 3);
    assert_eq!(run("watched"), 3);
}

/// Nucleoid rolls back failed in place sorts
#[rustfmt::skip]
#[test]
fn rolls_back_failed_in_place_sorts() {
    let mut run = runner();
    run(r#"scores = [3, 1, 2]
failure = ""
try:
    scores.sort((a, b) => a - b)
    throw "SORT_ROLLBACK"
catch error:
    failure = error
assert(failure, "SORT_ROLLBACK")
assert(scores, [3, 1, 2])
assert(scores@1, null)
try:
    scores.sort((a, b) => { throw "BAD_COMPARATOR" })
catch error:
    failure = error
assert(failure, "BAD_COMPARATOR")
assert(scores, [3, 1, 2])
def compare(a, b):
    if a > 100:
        throw "TOO_LARGE"
    return a - b
ranked = scores.sort(compare)
assert(scores, [1, 2, 3])
try:
    scores.push(500)
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(scores, [1, 2, 3])
assert(ranked, [1, 2, 3])
scores.push(0)
assert(scores, [0, 1, 2, 3])
assert(ranked, [0, 1, 2, 3])"#);
}

/// Nucleoid selects values with conditional expressions
#[rustfmt::skip]
#[test]
fn selects_values_with_conditional_expressions() {
    let mut run = runner();
    run("total = 80");
    run("member = false");
    run("shipping = 0 if total > 100 else 5");
    run("discount = member ? 10 : 0");
    run(r#"grade = total >= 90 ? "A" : total >= 70 ? "B" : "C""#);
    run(r#"level = "gold" if total > 150 else "silver" if total > 50 else "basic""#);
    run(r#"size = total > 100 ? "large" : "medium" if total > 50 else "small""#);
    run(r#"label = member || total > 100 ? "priority" : "standard""#);
    run("charge = total + (member ? 0 : shipping)");
    run(r#"vip = "yes" if member and total > 50 else "no""#);
    assert_eq!(run("shipping"), 5);
    assert_eq!(run("discount"), 0);
    assert_eq!(run("grade"), "B");
    assert_eq!(run("level"), "silver");
    assert_eq!(run("size"), "medium");
    assert_eq!(run("label"), "standard");
    assert_eq!(run("charge"), 85);
    assert_eq!(run("vip"), "no");
    run("total = 120");
    assert_eq!(run("shipping"), 0);
    assert_eq!(run("grade"), "A");
    assert_eq!(run("level"), "silver");
    assert_eq!(run("size"), "large");
    assert_eq!(run("label"), "priority");
    assert_eq!(run("charge"), 120);
    run("member = true");
    assert_eq!(run("discount"), 10);
    assert_eq!(run("vip"), "yes");
    run("total = 40");
    assert_eq!(run("shipping"), 5);
    assert_eq!(run("grade"), "C");
    assert_eq!(run("level"), "basic");
    assert_eq!(run("size"), "small");
    assert_eq!(run("label"), "priority");
    assert_eq!(run("charge"), 40);
    assert_eq!(run("vip"), "no");
}

/// Nucleoid evaluates only the selected conditional branch
#[rustfmt::skip]
#[test]
fn evaluates_only_the_selected_conditional_branch() {
    let mut run = runner();
    run(r#"log = []
def note(tag):
    log.push(tag)
    return tag
flag = true
other = 1
pick = flag ? note("yes") : note(other)
assert(pick, "yes")
assert(log, ["yes"])
other = 2
assert(pick, "yes")
assert(log, ["yes"])
flag = false
assert(pick, 2)
assert(log, ["yes", 2])
other = 3
assert(pick, 3)
assert(log, ["yes", 2, 3])
checks = []
def check(value):
    checks.push(value)
    return value > 1
size = "big" if check(other) else "small"
assert(size, "big")
assert(checks, [3])
ready = true
safe = 1 if ready else missing
assert(safe, 1)
failure = ""
try:
    ready = false
catch error:
    failure = error
assert(failure, ReferenceError("missing is not defined"))
assert(ready, true)
assert(safe, 1)
missing = 7
assert(safe, 1)
ready = false
assert(safe, 7)
toggle = null
state = toggle ? "on" : "off"
assert(state, null)
toggle = false
assert(state, "off")
toggle = 1
assert(state, "on")"#);
}

/// Nucleoid applies conditional expressions in class level rules
#[rustfmt::skip]
#[test]
fn applies_conditional_expressions_in_class_level_rules() {
    let mut run = runner();
    run(r#"class Order(total, color):
    this.total = total
    this.color = color
$Order.shipping = 0 if $Order.total > 100 else 5
$Order.tag = $Order.color == 1 ? "a" : "b"
$Order.rush = $Order.express ? 10 : 0
$Order.due = $Order.total + $Order.shipping + $Order.rush
small = Order(40, 1)
large = Order(250, 2)
assert(small.shipping, 5)
assert(large.shipping, 0)
assert(small.tag, "a")
assert(large.tag, "b")
assert(small.rush, 0)
assert(small.due, 45)
assert(large.due, 250)
small.total = 120
assert(small.shipping, 0)
assert(small.due, 120)
assert(large.shipping, 0)
large.color = 1
assert(large.tag, "a")
assert(small.tag, "a")
large.express = true
assert(large.rush, 10)
assert(small.rush, 0)
large.total = 30
assert(large.shipping, 5)
assert(large.due, 45)
failure = ""
try:
    $Order.note = $Order.total > 1000 ? vip : "regular"
catch error:
    failure = error
assert(failure, ReferenceError("vip is not defined"))
try:
    $Order.limit = $Order.express ? $Order.cap : 50
    $Order.cap = $Order.limit + 1
catch error:
    failure = error
assert(failure, TypeError("Circular Dependency"))"#);
}

/// Nucleoid uses conditional expressions in functions callbacks and literals
#[rustfmt::skip]
#[test]
fn uses_conditional_expressions_in_functions_callbacks_and_literals() {
    let mut run = runner();
    run("limit = 2");
    run("items = [1, 2, 3]");
    run(r#"def size(value) => value > limit ? "big" : "small""#);
    run(r#"def sign(value):
    return "neg" if value < 0 else "pos""#);
    run(r#"labels = items.map(item => item > limit ? "big" : "small")"#);
    run(r#"kinds = items.map(lambda item: "odd" if item % 2 else "even")"#);
    run("first = size(items[0])");
    run(r#"summary = { count: items.length > 2 ? "many" : "few", signs: [sign(-1), sign(limit)] }"#);
    run("pick = items[limit > 2 ? 0 : 1]");
    run(r#"banner = `${limit > 2 ? "wide" : "narrow"}`"#);
    run("scaled = Math.max(limit > 2 ? 10 : 20, 5)");
    assert_eq!(run("labels"), serde_json::json!(["small", "small", "big"]));
    assert_eq!(run("kinds"), serde_json::json!(["odd", "even", "odd"]));
    assert_eq!(run("first"), "small");
    assert_eq!(run("summary.count"), "many");
    assert_eq!(run("summary.signs"), serde_json::json!(["neg", "pos"]));
    assert_eq!(run("pick"), 2);
    assert_eq!(run("banner"), "narrow");
    assert_eq!(run("scaled"), 20);
    run("limit = 0");
    assert_eq!(run("labels"), serde_json::json!(["big", "big", "big"]));
    assert_eq!(run("first"), "big");
    assert_eq!(run("pick"), 2);
    run("limit = 5");
    assert_eq!(run("pick"), 1);
    assert_eq!(run("banner"), "wide");
    assert_eq!(run("scaled"), 10);
    run("items.push(6)");
    assert_eq!(run("labels"), serde_json::json!(["small", "small", "small", "big"]));
    assert_eq!(run("kinds"), serde_json::json!(["odd", "even", "odd", "even"]));
}

/// Nucleoid keeps if statements beside conditional expressions
#[rustfmt::skip]
#[test]
fn keeps_if_statements_beside_conditional_expressions() {
    let mut run = runner();
    run("ready = true");
    run("armed = true");
    run("lit = true");
    run("limit = 10");
    run(r#"mode = "on" if ready else "off""#);
    run(r#"x = 1 if armed:
    y = 2"#);
    run("z = 3 if lit: w = 4");
    run("if limit > 5: a = 1 else: a = 2");
    run("b = 5 c = 6");
    run(r#"if limit > (ready ? 5 : 20):
    band = "wide"
else:
    band = "narrow""#);
    run(r#"if armed and lit
    flow = "open"
elif limit > 5
    flow = "half"
else
    flow = "shut""#);
    run(r#"note = "ready? yes""#);
    run("pattern = /colou?r/");
    assert_eq!(run("mode"), "on");
    assert_eq!(run("[x, y, z, w, a, b, c]"), serde_json::json!([1, 2, 3, 4, 1, 5, 6]));
    assert_eq!(run("band"), "wide");
    assert_eq!(run("flow"), "open");
    assert_eq!(run("note"), "ready? yes");
    assert_eq!(run(r#"pattern.test("color")"#), true);
    run("ready = false");
    run("armed = false");
    assert_eq!(run("mode"), "off");
    assert_eq!(run("band"), "narrow");
    assert_eq!(run("flow"), "half");
    assert_eq!(run("[x, y, z, w, a]"), serde_json::json!([1, 2, 3, 4, 1]));
    run("limit = 1");
    assert_eq!(run("a"), 2);
    assert_eq!(run("flow"), "shut");
}

/// Nucleoid rolls back failed conditional expression updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_conditional_expression_updates() {
    let mut run = runner();
    run(r#"def check(amount):
    if amount > 100:
        throw "TOO_LARGE"
    return amount
rate = 2
amount = 5
fee = amount > 10 ? check(amount) * rate : rate
assert(fee, 2)
failure = ""
try:
    amount = 500
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(amount, 5)
assert(fee, 2)
try:
    amount = 20
    throw "ROLLBACK"
catch error:
    failure = error
assert(failure, "ROLLBACK")
assert(amount, 5)
assert(fee, 2)
amount = 20
assert(fee, 40)
rate = 3
assert(fee, 60)
base = 3
frozen = amount > 10 ? base.value : 0
spare = base.value if amount > 50 else 1
base = 4
assert(frozen, 3)
assert(spare, 1)
amount = 60
assert(spare, 3)
assert(fee, 180)
bonus = 7
reward = bonus if amount > 30 else 0
assert(reward, 7)
delete bonus
assert(reward, null)
bonus = 9
assert(reward, 9)
amount = 25
assert(reward, 0)"#);
}

/// Nucleoid defaults unknown values with nullish coalescing
#[rustfmt::skip]
#[test]
fn defaults_unknown_values_with_nullish_coalescing() {
    let mut run = runner();
    run("stock = null");
    run("count = stock ?? 0");
    run("discount = 0");
    run("rate = discount ?? 10");
    run(r#"page = { name: "", muted: false }"#);
    run(r#"page.title = page.name ?? "untitled""#);
    run("page.sound = page.muted ?? true");
    run("nickname = null");
    run(r#"first = "Ada""#);
    run(r#"display = nickname ?? first ?? "guest""#);
    run("other = null");
    run("fallback = other ?? nickname");
    run("plain = stock || 0");
    run("legacy = discount || 10");
    assert_eq!(run("count"), 0);
    assert_eq!(run("rate"), 0);
    assert_eq!(run("page.title"), "");
    assert_eq!(run("page.sound"), false);
    assert_eq!(run("display"), "Ada");
    assert_eq!(run("fallback"), serde_json::Value::Null);
    assert_eq!(run("plain"), serde_json::Value::Null);
    assert_eq!(run("legacy"), 10);
    run("stock = 5");
    run("discount = null");
    run("page.name = null");
    run(r#"nickname = "Lady A""#);
    assert_eq!(run("count"), 5);
    assert_eq!(run("rate"), 10);
    assert_eq!(run("page.title"), "untitled");
    assert_eq!(run("display"), "Lady A");
    assert_eq!(run("fallback"), "Lady A");
    assert_eq!(run("plain"), 5);
    assert_eq!(run("legacy"), serde_json::Value::Null);
    run("stock = 0");
    run("first = null");
    run("nickname = null");
    assert_eq!(run("count"), 0);
    assert_eq!(run("display"), "guest");
    assert_eq!(run("fallback"), serde_json::Value::Null);
}

/// Nucleoid evaluates the right operand of nullish coalescing only when needed
#[rustfmt::skip]
#[test]
fn evaluates_the_right_operand_of_nullish_coalescing_only_when_needed() {
    let mut run = runner();
    run(r#"log = []
def note(tag):
    log.push(tag)
    return tag
saved = "draft"
other = 1
pick = saved ?? note(other)
assert(pick, "draft")
assert(log, [])
other = 2
assert(pick, "draft")
assert(log, [])
saved = null
assert(pick, 2)
assert(log, [2])
other = 3
assert(pick, 3)
assert(log, [2, 3])
saved = 0
other = 4
assert(pick, 0)
assert(log, [2, 3])
ready = 1
safe = ready ?? missing
assert(safe, 1)
failure = ""
try:
    ready = null
catch error:
    failure = error
assert(failure, ReferenceError("missing is not defined"))
assert(ready, 1)
assert(safe, 1)
missing = 7
ready = null
assert(safe, 7)
try:
    loose = absent ?? 5
catch error:
    failure = error
assert(failure, ReferenceError("absent is not defined"))"#);
}

/// Nucleoid applies nullish coalescing in class level rules
#[rustfmt::skip]
#[test]
fn applies_nullish_coalescing_in_class_level_rules() {
    let mut run = runner();
    run(r#"class Order(total, coupon):
    this.total = total
    this.coupon = coupon
$Order.discount = $Order.coupon ?? 0
$Order.due = $Order.total - $Order.discount
if $Order.rush ?? false:
    $Order.lane = "fast"
else:
    $Order.lane = "normal"
small = Order(40, null)
large = Order(250, 25)
savings = Order.map(order => order.coupon ?? 0)
assert(small.discount, 0)
assert(large.discount, 25)
assert(small.due, 40)
assert(large.due, 225)
assert(small.lane, "normal")
assert(large.lane, "normal")
assert(savings, [0, 25])
small.coupon = 0
assert(small.discount, 0)
small.coupon = 5
assert(small.discount, 5)
assert(small.due, 35)
large.coupon = null
assert(large.discount, 0)
assert(large.due, 250)
assert(savings, [5, 0])
large.rush = true
assert(large.lane, "fast")
assert(small.lane, "normal")
large.rush = null
assert(large.lane, "normal")
failure = ""
try:
    $Order.note = $Order.memo ?? vip
catch error:
    failure = error
assert(failure, ReferenceError("vip is not defined"))
try:
    $Order.limit = $Order.cap ?? 50
    $Order.cap = $Order.limit + 1
catch error:
    failure = error
assert(failure, TypeError("Circular Dependency"))"#);
}

/// Nucleoid uses nullish coalescing in functions callbacks and literals
#[rustfmt::skip]
#[test]
fn uses_nullish_coalescing_in_functions_callbacks_and_literals() {
    let mut run = runner();
    run("settings = {}");
    run(r#"items = [{ name: "pen", size: 2 }, { name: "cup", size: 0 }, { name: "box" }]"#);
    run("def width(item) => item.size ?? settings.size ?? 1");
    run(r#"def label(item):
    return item.nick ?? item.name"#);
    run("sizes = items.map(item => item.size ?? -1)");
    run("names = items.map(lambda item: item.nick ?? item.name)");
    run("first = width(items[2])");
    run("last = label(items[2])");
    run("summary = { cap: settings.cap ?? 10, widths: [settings.size ?? 1, width(items[1])] }");
    run("pick = items[settings.index ?? 1].name");
    run(r#"banner = `${settings.title ?? "untitled"}`"#);
    run("scaled = Math.max(settings.cap ?? 0, 5)");
    run(r#"mode = settings.strict ? "strict" : settings.mode ?? "auto""#);
    assert_eq!(run("sizes"), serde_json::json!([2, 0, -1]));
    assert_eq!(run("names"), serde_json::json!(["pen", "cup", "box"]));
    assert_eq!(run("first"), 1);
    assert_eq!(run("last"), "box");
    assert_eq!(run("summary.cap"), 10);
    assert_eq!(run("summary.widths"), serde_json::json!([1, 0]));
    assert_eq!(run("pick"), "cup");
    assert_eq!(run("banner"), "untitled");
    assert_eq!(run("scaled"), 5);
    assert_eq!(run("mode"), "auto");
    run("settings.size = 3");
    run("settings.index = 0");
    run(r#"settings.title = "Sale""#);
    run("settings.cap = 0");
    run(r#"settings.mode = "manual""#);
    assert_eq!(run("first"), 3);
    assert_eq!(run("summary.cap"), 0);
    assert_eq!(run("summary.widths"), serde_json::json!([3, 0]));
    assert_eq!(run("pick"), "pen");
    assert_eq!(run("banner"), "Sale");
    assert_eq!(run("scaled"), 5);
    assert_eq!(run("mode"), "manual");
    run(r#"items.push({ name: "bag", nick: "tote" })"#);
    assert_eq!(run("sizes"), serde_json::json!([2, 0, -1, -1]));
    assert_eq!(run("names"), serde_json::json!(["pen", "cup", "box", "tote"]));
}

/// Nucleoid groups nullish coalescing between conditionals and logical operators
#[rustfmt::skip]
#[test]
fn groups_nullish_coalescing_between_conditionals_and_logical_operators() {
    let mut run = runner();
    run("page = { base: 0, level: 3, visible: true }");
    run(r#"state = page.custom ?? page.base ? "set" : "unset""#);
    run(r#"shown = page.visible ? page.label ?? "untitled" : "hidden""#);
    run(r#"spoken = page.label ?? "untitled" if page.visible else "hidden""#);
    run("first = page.flag || page.backup ?? 9");
    run("second = page.custom ?? page.base || page.level");
    run("same = page.custom ?? page.level == 3");
    run(r#"note = "why?? not""#);
    run("pattern = /colou??r/");
    run(r#"greeting = `who?? ${page.label ?? "you"}`"#);
    assert_eq!(run("state"), "unset");
    assert_eq!(run("shown"), "untitled");
    assert_eq!(run("spoken"), "untitled");
    assert_eq!(run("first"), 9);
    assert_eq!(run("second"), 3);
    assert_eq!(run("same"), true);
    assert_eq!(run("note"), "why?? not");
    assert_eq!(run(r#"pattern.test("color")"#), true);
    assert_eq!(run("greeting"), "who?? you");
    run("page.custom = 0");
    run("page.backup = 4");
    assert_eq!(run("state"), "unset");
    assert_eq!(run("first"), 4);
    assert_eq!(run("second"), 0);
    assert_eq!(run("same"), 0);
    run("page.custom = 2");
    run(r#"page.label = "Main""#);
    assert_eq!(run("state"), "set");
    assert_eq!(run("shown"), "Main");
    assert_eq!(run("spoken"), "Main");
    assert_eq!(run("second"), 2);
    assert_eq!(run("greeting"), "who?? Main");
    run("page.visible = false");
    assert_eq!(run("shown"), "hidden");
    assert_eq!(run("spoken"), "hidden");
}

/// Nucleoid rolls back failed nullish coalescing updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_nullish_coalescing_updates() {
    let mut run = runner();
    run(r#"def check(amount):
    if amount > 100:
        throw "TOO_LARGE"
    return amount
rate = 2
amount = 5
custom = null
fee = custom ?? check(amount) * rate
assert(fee, 10)
failure = ""
try:
    amount = 500
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(amount, 5)
assert(fee, 10)
try:
    custom = 1
    throw "ROLLBACK"
catch error:
    failure = error
assert(failure, "ROLLBACK")
assert(custom, null)
assert(fee, 10)
custom = 3
assert(fee, 3)
amount = 500
assert(fee, 3)
rate = 4
amount = 20
custom = null
assert(fee, 80)
base = 3
config = { size: 2 }
spare = config.size ?? base.value
base = 4
assert(spare, 2)
delete config.size
assert(spare, 3)
bonus = 7
reward = bonus ?? 0
assert(reward, 7)
delete bonus
assert(reward, 0)
bonus = 9
assert(reward, 9)"#);
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
