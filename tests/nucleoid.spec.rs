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

/// Nucleoid slices class instance lists with ranges
#[rustfmt::skip]
#[test]
fn slices_class_instance_lists_with_ranges() {
    let mut run = runner();
    run(r#"class User(name):
    this.name = name"#);
    run(r#"ada = User("Ada")"#);
    run(r#"bob = User("Bob")"#);
    run(r#"cy = User("Cy")"#);
    {
        let actual = run("User[:]");
        let expected = run("([ada, bob, cy])");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("User[1:]");
        let expected = run("([bob, cy])");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("User[:2]");
        let expected = run("([ada, bob])");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("User[-2:]");
        let expected = run("([bob, cy])");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("User[1:-1]");
        let expected = run("([bob])");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("User[0:9]");
        let expected = run("([ada, bob, cy])");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("User[-9:1]");
        let expected = run("([ada])");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("User[2:1]"), serde_json::json!([]));
    {
        let actual = run("User[1:]");
        let expected = run("(User.slice(1))");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("User[-2:-1]");
        let expected = run("(User.slice(-2, -1))");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("User[:].length");
        let expected = run("(User.length)");
        assert_eq!(actual, expected);
    }
    run("start = 1");
    {
        let actual = run("User[start:start + 1]");
        let expected = run("([bob])");
        assert_eq!(actual, expected);
    }
    run("class Empty");
    assert_eq!(run("Empty[:]"), serde_json::json!([]));
    {
        let actual = run("User[0]");
        let expected = run("(ada)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run(r#"User["bob"]"#);
        let expected = run("(bob)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("[1, 2, 3][1:]"), serde_json::json!([2, 3]));
    assert_eq!(run(r#""Nucleoid"[-4:]"#), "eoid");
    run(r#"User[1:].map(user => user.name)

# return: ["Bob","Cy"]"#);
}

/// Nucleoid keeps instance list ranges live
#[rustfmt::skip]
#[test]
fn keeps_instance_list_ranges_live() {
    let mut run = runner();
    run(r#"class Task(title):
    this.title = title"#);
    run(r#"first = Task("plan")"#);
    run(r#"second = Task("build")"#);
    run("count = 1");
    run("recent = Task[-2:]");
    run("opening = Task[:count]");
    run("pinned = Task[:count.value]");
    run("titles = Task[1:].map(task => task.title)");
    {
        let actual = run("recent");
        let expected = run("([first, second])");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("opening");
        let expected = run("([first])");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("pinned");
        let expected = run("([first])");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("titles"), serde_json::json!(["build"]));
    run(r#"third = Task("ship")"#);
    {
        let actual = run("recent");
        let expected = run("([second, third])");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("titles"), serde_json::json!(["build", "ship"]));
    run("count = 2");
    {
        let actual = run("opening");
        let expected = run("([first, second])");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("pinned");
        let expected = run("([first])");
        assert_eq!(actual, expected);
    }
    run(r#"second.title = "test""#);
    assert_eq!(run("titles"), serde_json::json!(["test", "ship"]));
    run("class Ticket");
    run("older = Ticket()");
    run("newer = Ticket()");
    run("latest = Ticket[-1:]");
    run("extra = Ticket()");
    {
        let actual = run("latest");
        let expected = run("([extra])");
        assert_eq!(actual, expected);
    }
    run("delete extra");
    {
        let actual = run("latest");
        let expected = run("([newer])");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("Ticket[:]");
        let expected = run("([older, newer])");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid lists every instance with list and all
#[rustfmt::skip]
#[test]
fn lists_every_instance_with_list_and_all() {
    let mut run = runner();
    run(r#"class Member(name):
    this.name = name
ana = Member("Ana")
ben = Member("Ben")
everyone = Member.list()
roster = Member.all()
names = Member.list().map(member => member.name)
assert(everyone, [ana, ben])
assert(roster, everyone)
assert(everyone, Member[:])
assert(everyone, Member.slice())
cal = Member("Cal")
assert(everyone, [ana, ben, cal])
assert(roster, [ana, ben, cal])
assert(names, ["Ana", "Ben", "Cal"])
class Guest
dee = Guest()
party = Guest.concat(Member.list())
assert(party, [dee, ana, ben, cal])
assert(Guest.concat(Member[1:]), [dee, ben, cal])
calls = []
def note(tag):
    calls.push(tag)
    return tag
assert(Member.all(note(1), note(2)), [ana, ben, cal])
assert(calls, [1, 2])
plain = [1, 2]
failure = ""
try:
    plain.list()
catch error:
    failure = error
assert(failure, TypeError("plain.list is not a function"))
try:
    plain.all()
catch error:
    failure = error
assert(failure, TypeError("plain.all is not a function"))"#);
}

/// Nucleoid compares list members with deep equality
#[rustfmt::skip]
#[test]
fn compares_list_members_with_deep_equality() {
    let mut run = runner();
    run(r#"tags = ["red", "blue", "red"]"#);
    assert_eq!(run(r#"tags.includes("blue")"#), true);
    assert_eq!(run(r#"tags.includes("green")"#), false);
    assert_eq!(run(r#"tags.indexOf("red")"#), 0);
    assert_eq!(run(r#"tags.indexOf("green")"#), -1);
    assert_eq!(run(r#"tags.indexOf("red", 1)"#), 0);
    assert_eq!(run(r#"tags.includes("red", 5)"#), true);
    run("pairs = [[1, 2], { x: 1 }, null]");
    assert_eq!(run("pairs.indexOf([1, 2])"), 0);
    assert_eq!(run("pairs.includes({ x: 1 })"), true);
    assert_eq!(run("pairs.includes({ x: 2 })"), false);
    assert_eq!(run("pairs.indexOf()"), 2);
    assert_eq!(run("pairs.includes()"), true);
    assert_eq!(run("[Number.NaN].includes(Number.NaN)"), false);
    assert_eq!(run("[Number.NaN].indexOf(Number.NaN)"), -1);
    run(r#"class Seat(row):
    this.row = row"#);
    run("front = Seat(1)");
    run("twin = Seat(1)");
    assert_eq!(run("Seat.indexOf(twin)"), 1);
    assert_eq!(run("Seat.indexOf(front)"), 0);
    assert_eq!(run("Seat.includes(twin)"), true);
    assert_eq!(run("Seat.includes({ row: 1 })"), false);
    assert_eq!(run(r#"Seat.includes({ id: "twin", row: 1 })"#), true);
    assert_eq!(run("Seat.indexOf(5)"), -1);
    run(r#"where = tags.indexOf("green")"#);
    run(r#"known = tags.includes("green")"#);
    assert_eq!(run("where"), -1);
    run(r#"tags.push("green")"#);
    assert_eq!(run("where"), 3);
    assert_eq!(run("known"), true);
    run("back = Seat(2)");
    run("spot = Seat.indexOf(back)");
    assert_eq!(run("spot"), 2);
}

/// Nucleoid concatenates lists and instance lists into new lists
#[rustfmt::skip]
#[test]
fn concatenates_lists_and_instance_lists_into_new_lists() {
    let mut run = runner();
    run("base = [1, 2]");
    run("more = base.concat(3, [4, [5]])");
    assert_eq!(run("more"), serde_json::json!([1, 2, 3, 4, [5]]));
    assert_eq!(run("base"), serde_json::json!([1, 2]));
    assert_eq!(run("base.concat()"), serde_json::json!([1, 2]));
    assert_eq!(run("base.concat(null)"), serde_json::json!([1, 2, null]));
    assert_eq!(run("base.concat([])"), serde_json::json!([1, 2]));
    run("base.push(9)");
    assert_eq!(run("more"), serde_json::json!([1, 2, 9, 3, 4, [5]]));
    run("class Guest");
    run("amy = Guest()");
    run("ben = Guest()");
    run("class Host");
    run("cara = Host()");
    run("dan = Host()");
    run("party = Guest.concat([cara, dan])");
    {
        let actual = run("party");
        let expected = run("([amy, ben, cara, dan])");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("Guest.concat(Host).length"), 3);
    {
        let actual = run("Guest.concat(Host)[2]");
        let expected = run("(Host)");
        assert_eq!(actual, expected);
    }
    run("eve = Guest()");
    {
        let actual = run("party");
        let expected = run("([amy, ben, eve, cara, dan])");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("Guest.length"), 3);
}

/// Nucleoid reads positions from either end with at and lastIndexOf
#[rustfmt::skip]
#[test]
fn reads_positions_from_either_end_with_at_and_lastindexof() {
    let mut run = runner();
    run("scores = [10, 20, 30]");
    assert_eq!(run("scores.at(0)"), 10);
    assert_eq!(run("scores.at(-1)"), 30);
    assert_eq!(run("scores.at(1.9)"), 20);
    assert_eq!(run("scores.at(-1.9)"), 30);
    assert_eq!(run("scores.at()"), 10);
    assert_eq!(run(r#"scores.at("2")"#), 30);
    assert_eq!(run("scores.at(3)"), serde_json::Value::Null);
    assert_eq!(run("scores.at(-4)"), serde_json::Value::Null);
    assert_eq!(run("typeof scores.at(3)"), "undefined");
    run("last = scores.at(-1)");
    run("scores.push(40)");
    assert_eq!(run("last"), 40);
    run(r#"label = "top " + scores.at(9)"#);
    assert_eq!(run("label"), serde_json::Value::Null);
    run(r#"tags = ["red", "blue", "red"]"#);
    assert_eq!(run(r#"tags.lastIndexOf("red")"#), 2);
    assert_eq!(run(r#"tags.lastIndexOf("green")"#), -1);
    assert_eq!(run(r#"tags.lastIndexOf("red", 0)"#), 2);
    assert_eq!(run("[[1], [1]].lastIndexOf([1])"), 1);
    assert_eq!(run("[null, 1].lastIndexOf()"), 0);
    assert_eq!(run("[Number.NaN].lastIndexOf(Number.NaN)"), -1);
    run(r#"where = tags.lastIndexOf("blue")"#);
    run(r#"tags.push("blue")"#);
    assert_eq!(run("where"), 3);
    run(r#"class Lap(time):
    this.time = time"#);
    run("one = Lap(61)");
    run("two = Lap(58)");
    {
        let actual = run("Lap.at(0)");
        let expected = run("(one)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("Lap.at(-1)");
        let expected = run("(two)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("Lap.at(2)"), serde_json::Value::Null);
    assert_eq!(run("typeof Lap.at(2)"), "null");
    assert_eq!(run(r#""lap " + Lap.at(5)"#), "lap null");
    assert_eq!(run("Lap.lastIndexOf(one)"), 0);
    run("latest = Lap.at(-1).time");
    run("three = Lap(57)");
    assert_eq!(run("latest"), 57);
}

/// Nucleoid searches from either end with findIndex findLast and findLastIndex
#[rustfmt::skip]
#[test]
fn searches_from_either_end_with_findindex_findlast_and_findlastindex() {
    let mut run = runner();
    run(r#"visits = []
def big(value):
    visits.push(value)
    return value > 15
readings = [10, 20, 30, 5]
assert(readings.findIndex(big), 1)
assert(visits, [10, 20])
visits = []
assert(readings.findLast(big), 30)
assert(visits, [5, 30])
visits = []
assert(readings.findLastIndex(big), 2)
assert(visits, [5, 30])
assert(readings.find(big), 20)
assert(readings.findIndex(value => value > 99), -1)
assert(readings.findLast(value => value > 99), null)
assert(readings.findLastIndex(value => value > 99), -1)
assert(readings.findLastIndex((value, position) => position == null), 3)
assert([].findLast(value => true), null)
failure = ""
try:
    readings.findIndex(15)
catch error:
    failure = error
assert(failure, TypeError("findIndex expects a function"))
try:
    [].findLast()
catch error:
    failure = error
assert(failure, TypeError("findLast expects a function"))
try:
    [].findLastIndex(null)
catch error:
    failure = error
assert(failure, TypeError("findLastIndex expects a function"))"#);
}

/// Nucleoid keeps list queries live
#[rustfmt::skip]
#[test]
fn keeps_list_queries_live() {
    let mut run = runner();
    run(r#"class Order(total, rush):
    this.total = total
    this.rush = rush"#);
    run("first = Order(20, false)");
    run("second = Order(45, true)");
    run("third = Order(30, false)");
    run("limit = 25");
    run("lastRush = Order.findLast(order => order.rush)");
    run("lastBig = Order.findLastIndex(order => order.total > limit)");
    run("firstBig = Order.findIndex(order => order.total > limit)");
    run("newest = Order.at(-1)");
    {
        let actual = run("lastRush");
        let expected = run("(second)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("lastBig"), 2);
    assert_eq!(run("firstBig"), 1);
    {
        let actual = run("newest");
        let expected = run("(third)");
        assert_eq!(actual, expected);
    }
    run("third.rush = true");
    {
        let actual = run("lastRush");
        let expected = run("(third)");
        assert_eq!(actual, expected);
    }
    run("limit = 40");
    assert_eq!(run("lastBig"), 1);
    assert_eq!(run("firstBig"), 1);
    run("second.total = 10");
    assert_eq!(run("lastBig"), -1);
    assert_eq!(run("firstBig"), -1);
    run("fourth = Order(50, false)");
    {
        let actual = run("newest");
        let expected = run("(fourth)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("lastBig"), 3);
    assert_eq!(run("firstBig"), 3);
    {
        let actual = run("lastRush");
        let expected = run("(third)");
        assert_eq!(actual, expected);
    }
    run("queue = [3, 8, 1]");
    run("peak = queue.findLastIndex(value => value > limit / 10)");
    run("tail = queue.at(-1)");
    run("where = queue.lastIndexOf(8)");
    assert_eq!(run("peak"), 1);
    run("queue.push(9)");
    assert_eq!(run("peak"), 3);
    assert_eq!(run("tail"), 9);
    assert_eq!(run("where"), 1);
    run("queue.pop()");
    assert_eq!(run("tail"), 1);
    assert_eq!(run("peak"), 1);
}

/// Nucleoid applies instance list ranges and queries in class level rules
#[rustfmt::skip]
#[test]
fn applies_instance_list_ranges_and_queries_in_class_level_rules() {
    let mut run = runner();
    run(r#"class Player(name):
    this.name = name
class Team(size):
    this.size = size
$Team.roster = Player[0:$Team.size]
$Player.rank = Player.indexOf($Player)
ana = Player("Ana")
ben = Player("Ben")
red = Team(1)
blue = Team(3)
assert(red.roster, [ana])
assert(blue.roster, [ana, ben])
assert(ana.rank, 0)
assert(ben.rank, 1)
cal = Player("Cal")
assert(blue.roster, [ana, ben, cal])
assert(cal.rank, 2)
red.size = 2
assert(red.roster, [ana, ben])
failure = ""
try:
    $Team.bench = Player[$Team.size:limit]
catch error:
    failure = error
assert(failure, ReferenceError("limit is not defined"))"#);
}

/// Nucleoid rolls back failed list query updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_list_query_updates() {
    let mut run = runner();
    run(r#"def check(value):
    if value > 100:
        throw "TOO_LARGE"
    return value > 2
values = [1, 3]
spot = values.findLastIndex(check)
early = values.findIndex(check)
assert(spot, 1)
assert(early, 1)
failure = ""
try:
    values.push(500)
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(values, [1, 3])
assert(spot, 1)
values.push(5)
assert(spot, 2)
assert(early, 1)
class Item(size):
    this.size = size
small = Item(1)
pick = Item.findLast(item => check(item.size))
assert(pick, null)
try:
    big = Item(500)
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(Item.length, 1)
assert(pick, null)
try:
    mid = Item(3)
    throw "ROLLBACK"
catch error:
    failure = error
assert(failure, "ROLLBACK")
assert(Item.length, 1)
assert(pick, null)
mid = Item(3)
assert(pick, mid)"#);
}

/// Nucleoid quantifies over class instances with all and any
#[rustfmt::skip]
#[test]
fn quantifies_over_class_instances_with_all_and_any() {
    let mut run = runner();
    run("class Check");
    run("ready = all c in Check: c.passed");
    run("blocked = any c in Check: c.failed");
    assert_eq!(run("ready"), true);
    assert_eq!(run("blocked"), false);
    run("first = Check()");
    run("second = Check()");
    run("first.passed = true");
    run("second.passed = true");
    assert_eq!(run("ready"), true);
    run("second.passed = false");
    run("second.failed = true");
    assert_eq!(run("ready"), false);
    assert_eq!(run("blocked"), true);
    run("second.passed = true");
    run("second.failed = false");
    assert_eq!(run("ready"), true);
    assert_eq!(run("blocked"), false);
    run("third = Check()");
    assert_eq!(run("ready"), false);
    run("third.passed = true");
    assert_eq!(run("ready"), true);
}

/// Nucleoid quantifies over lists with all and any
#[rustfmt::skip]
#[test]
fn quantifies_over_lists_with_all_and_any() {
    let mut run = runner();
    run("numbers = [3, 5, 8]");
    run("limit = 2");
    run("large = all n in numbers: n > limit");
    run("even = any n in numbers: n % 2 == 0");
    assert_eq!(run("large"), true);
    assert_eq!(run("even"), true);
    run("limit = 4");
    assert_eq!(run("large"), false);
    run("numbers = [5, 7]");
    assert_eq!(run("large"), true);
    assert_eq!(run("even"), false);
    run("numbers.push(10)");
    assert_eq!(run("even"), true);
    run("empty = []");
    run("vacuous = all n in empty: n > 0");
    run("none = any n in empty: n > 0");
    assert_eq!(run("vacuous"), true);
    assert_eq!(run("none"), false);
}

/// Nucleoid quantifies over reasoning selections
#[rustfmt::skip]
#[test]
fn quantifies_over_reasoning_selections() {
    let mut run = runner();
    run("a = 1");
    run("b = a + 2");
    run("c = b * 2");
    run("positive = all s in why c: s.holds > 0");
    run("grounded = all s in c |> why |> stated: s.holds > 0");
    run(r#"mentions = any s in why c: s.node == "d""#);
    assert_eq!(run("positive"), true);
    assert_eq!(run("grounded"), true);
    assert_eq!(run("mentions"), false);
    run("d = -5");
    run("b = a + d");
    assert_eq!(run("positive"), false);
    assert_eq!(run("grounded"), false);
    assert_eq!(run("mentions"), true);
    run("d = 5");
    assert_eq!(run("positive"), true);
    assert_eq!(run("grounded"), true);
}

/// Nucleoid nests quantifiers over related instances
#[rustfmt::skip]
#[test]
fn nests_quantifiers_over_related_instances() {
    let mut run = runner();
    run(r#"class Task(skill):
    this.skill = skill"#);
    run(r#"class Worker(skill):
    this.skill = skill"#);
    run("covered = all t in Task: any w in Worker: w.skill == t.skill");
    run("spare = any w in Worker: all t in Task: t.skill != w.skill");
    assert_eq!(run("covered"), true);
    assert_eq!(run("spare"), false);
    run(r#"Task("rust")"#);
    run(r#"Task("docs")"#);
    assert_eq!(run("covered"), false);
    run(r#"Worker("rust")"#);
    run(r#"writer = Worker("docs")"#);
    assert_eq!(run("covered"), true);
    assert_eq!(run("spare"), false);
    run(r#"Worker("ops")"#);
    assert_eq!(run("spare"), true);
    run(r#"writer.skill = "ops""#);
    assert_eq!(run("covered"), false);
    run(r#"writer.skill = "docs""#);
    assert_eq!(run("covered"), true);
}

/// Nucleoid applies quantifiers in class level rules
#[rustfmt::skip]
#[test]
fn applies_quantifiers_in_class_level_rules() {
    let mut run = runner();
    run(r#"class Line(sent):
    this.sent = sent"#);
    run(r#"class Order(lines):
    this.lines = lines"#);
    run("$Order.complete = all line in $Order.lines: line.sent");
    run("$Order.started = any line in $Order.lines: line.sent");
    run("first = Line(true)");
    run("second = Line(false)");
    run("order = Order([first, second])");
    run("empty = Order([])");
    assert_eq!(run("order.complete"), false);
    assert_eq!(run("order.started"), true);
    assert_eq!(run("empty.complete"), true);
    assert_eq!(run("empty.started"), false);
    run("second.sent = true");
    assert_eq!(run("order.complete"), true);
    run("first.sent = false");
    run("second.sent = false");
    assert_eq!(run("order.complete"), false);
    assert_eq!(run("order.started"), false);
}

/// Nucleoid uses quantifiers in functions callbacks and conditionals
#[rustfmt::skip]
#[test]
fn uses_quantifiers_in_functions_callbacks_and_conditionals() {
    let mut run = runner();
    run("def positive(values) => all v in values: v > 0");
    run("scores = [2, 4]");
    run("ok = positive(scores)");
    run(r#"label = (all v in scores: v > 0) ? "clean" : "dirty""#);
    assert_eq!(run("ok"), true);
    assert_eq!(run("label"), "clean");
    run("scores = [2, -1]");
    assert_eq!(run("ok"), false);
    assert_eq!(run("label"), "dirty");
    run("groups = [[1, 2], [3, -4], []]");
    run("whole = groups.filter(g => all v in g: v > 0)");
    run("mixed = groups.map(g => any v in g: v < 0)");
    assert_eq!(run("whole"), serde_json::json!([[1, 2], []]));
    assert_eq!(run("mixed"), serde_json::json!([false, true, false]));
    run("groups = [[-1]]");
    assert_eq!(run("whole"), serde_json::json!([]));
    assert_eq!(run("mixed"), serde_json::json!([true]));
}

/// Nucleoid keeps all and any as ordinary names beside quantifiers
#[rustfmt::skip]
#[test]
fn keeps_all_and_any_as_ordinary_names_beside_quantifiers() {
    let mut run = runner();
    run("all = [1, 2, 3]");
    run("any = 0");
    run("large = all n in all: n > any");
    run("some = any n in all: n > 2");
    assert_eq!(run("large"), true);
    assert_eq!(run("some"), true);
    run("any = 2");
    assert_eq!(run("large"), false);
    assert_eq!(run("some"), true);
    run("n = 10");
    run("doubled = all n in all: n * 2 > n");
    assert_eq!(run("doubled"), true);
    assert_eq!(run("n"), 10);
}

/// Nucleoid rolls back failed quantifier updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_quantifier_updates() {
    let mut run = runner();
    run(r#"def checked(value):
    if value < 0:
        throw "NEGATIVE"
    return value > 0
class Item(qty):
    this.qty = qty
item = Item(2)
valid = all i in Item: checked(i.qty)
assert(valid, true)
failure = ""
try:
    item.qty = -1
catch error:
    failure = error
assert(failure, "NEGATIVE")
assert(item.qty, 2)
assert(valid, true)
failure = ""
try:
    Item(-3)
catch error:
    failure = error
assert(failure, "NEGATIVE")
assert(Item.length, 1)
item.qty = 0
assert(valid, false)
try:
    broken = all x in missing: x
catch error:
    failure = error
assert(failure, ReferenceError("missing is not defined"))
assert(valid, false)"#);
}

/// Nucleoid raises numbers to powers with the exponentiation operator
#[rustfmt::skip]
#[test]
fn raises_numbers_to_powers_with_the_exponentiation_operator() {
    let mut run = runner();
    run("base = 2");
    run("exponent = 3");
    run("tower = base ** exponent ** 2");
    run("grouped = (base ** exponent) ** 2");
    run("scaled = 3 * base ** exponent + 1");
    assert_eq!(run("tower"), 512);
    assert_eq!(run("grouped"), 64);
    assert_eq!(run("scaled"), 25);
    run("base = 3");
    run("exponent = 2");
    assert_eq!(run("tower"), 81);
    assert_eq!(run("grouped"), 81);
    assert_eq!(run("scaled"), 28);
    run("base = 16");
    run("exponent = 0.5");
    assert_eq!(run("tower"), 2);
    assert_eq!(run("grouped"), 16);
    assert_eq!(run("scaled"), 13);
    run(r#"tower

# return: 2"#);
}

/// Nucleoid binds prefix operators around the exponentiation operator
#[rustfmt::skip]
#[test]
fn binds_prefix_operators_around_the_exponentiation_operator() {
    let mut run = runner();
    run("x = 2");
    run("n = 2");
    run("negated = -x ** n");
    run("grouped = (-x) ** n");
    run("inverse = x ** -n");
    assert_eq!(run("negated"), -4);
    assert_eq!(run("grouped"), 4);
    assert_eq!(run("inverse"), 0.25);
    run("n = 3");
    assert_eq!(run("negated"), -8);
    assert_eq!(run("grouped"), -8);
    assert_eq!(run("inverse"), 0.125);
    run("x = 0.5");
    assert_eq!(run("negated"), -0.125);
    assert_eq!(run("grouped"), -0.125);
    assert_eq!(run("inverse"), 8);
}

/// Nucleoid gives the exponentiation operator the results of Math.pow
#[rustfmt::skip]
#[test]
fn gives_the_exponentiation_operator_the_results_of_math_pow() {
    let mut run = runner();
    run(r#"base = "3""#);
    run("power = 2");
    run("result = base ** power");
    run("literal = null ** power");
    assert_eq!(run("result"), 9);
    assert_eq!(run("literal"), 0);
    run("base = [2]");
    assert_eq!(run("result"), 4);
    run("base = true");
    run("power = 3");
    assert_eq!(run("result"), 1);
    run(r#"base = "a""#);
    assert_eq!(run("Number.isNaN(result)"), true);
    run("base = 0");
    run("power = -1");
    {
        let actual = run("result");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("literal");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("base = -0");
    {
        let actual = run("result");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("base = -8");
    run("power = 1 / 3");
    assert_eq!(run("Number.isNaN(result)"), true);
    run("base = Number.NaN");
    run("power = 0");
    assert_eq!(run("result"), 1);
    assert_eq!(run("literal"), 1);
    run("base = null");
    assert_eq!(run("result"), serde_json::Value::Null);
}

/// Nucleoid keeps exponentiation relationships live in class level rules
#[rustfmt::skip]
#[test]
fn keeps_exponentiation_relationships_live_in_class_level_rules() {
    let mut run = runner();
    run(r#"class Tank(side, depth):
    this.side = side
    this.depth = depth"#);
    run("$Tank.volume = $Tank.side ** 2 * $Tank.depth");
    run("$Tank.cube = $Tank.side ** 3");
    run("small = Tank(2, 3)");
    run("large = Tank(3, 1)");
    assert_eq!(run("small.volume"), 12);
    assert_eq!(run("large.volume"), 9);
    assert_eq!(run("small.cube"), 8);
    run("small.side = 4");
    assert_eq!(run("small.volume"), 48);
    assert_eq!(run("small.cube"), 64);
    assert_eq!(run("large.cube"), 27);
    run("small.depth = 0.5");
    assert_eq!(run("small.volume"), 8);
    run("delete small.side");
    assert_eq!(run("small.volume"), serde_json::Value::Null);
    assert_eq!(run("small.cube"), serde_json::Value::Null);
}

/// Nucleoid keeps compound exponentiation assignments equivalent
#[rustfmt::skip]
#[test]
fn keeps_compound_exponentiation_assignments_equivalent() {
    let mut run = runner();
    run(r#"value = 4
step = 2
value **= step
assert(value, 16)
step = 0.5
assert(value, 4)
class Meter(reading, power):
    this.reading = reading
    this.power = power
    def root(degree):
        degree **= -1
        this.reading **= degree
        return this.reading
meter = Meter(2, 2)
meter.reading **= meter.power
assert(meter.reading, 4)
meter.power = 3
assert(meter.reading, 64)
assert(meter.root(2), 8)
assert(meter.reading, 8)
class Scale
scale = Scale()
$Scale.factor = 2
try:
    $Scale.factor **= 3
catch error:
    assert(error, TypeError("Circular Dependency"))
assert(scale.factor, 2)"#);
}

/// Nucleoid uses the exponentiation operator in functions callbacks and literals
#[rustfmt::skip]
#[test]
fn uses_the_exponentiation_operator_in_functions_callbacks_and_literals() {
    let mut run = runner();
    run("def hypotenuse(a, b) => (a ** 2 + b ** 2) ** 0.5");
    run("def grow current rate => current **= rate");
    run("sides = [3, 4]");
    run("squares = sides.map(side -> side ** 2)");
    run("box = {volume: sides[1] ** 3}");
    run("label = `${sides[0]} cubed is ${sides[0] ** 3}`");
    assert_eq!(run("hypotenuse(3, 4)"), 5);
    assert_eq!(run("hypotenuse(8, 15)"), 17);
    assert_eq!(run("grow(4, 1.5)"), 8);
    assert_eq!(run("grow(16, 0.25)"), 2);
    assert_eq!(run("squares"), serde_json::json!([9, 16]));
    assert_eq!(run("box.volume"), 64);
    assert_eq!(run("label"), "3 cubed is 27");
    run("sides = [2, 5]");
    assert_eq!(run("squares"), serde_json::json!([4, 25]));
    assert_eq!(run("box.volume"), 125);
    assert_eq!(run("label"), "2 cubed is 8");
}

/// Nucleoid keeps double stars in strings and regular expressions beside the exponentiation operator
#[rustfmt::skip]
#[test]
fn keeps_double_stars_in_strings_and_regular_expressions_beside_the_exponentiation_operator() {
    let mut run = runner();
    run("bits = 8");
    run("states = 2 ** bits");
    run("note = `2**${bits} is ${states}`");
    run("pattern = /x**y/ # x**y stays a pattern");
    assert_eq!(run("states"), 256);
    assert_eq!(run("note"), "2**8 is 256");
    assert_eq!(run(r#"note.split("**")"#), serde_json::json!(["2", "8 is 256"]));
    assert_eq!(run(r#"pattern.test("xxy")"#), true);
    assert_eq!(run(r#"pattern.test("xx")"#), false);
    run("bits = 10");
    assert_eq!(run("states"), 1024);
    assert_eq!(run("note"), "2**10 is 1024");
}

/// Nucleoid rolls back failed exponentiation updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_exponentiation_updates() {
    let mut run = runner();
    run(r#"def checked(value):
    if value > 1000:
        throw "TOO_LARGE"
    return value
base = 2
power = 3
result = checked(base ** power)
assert(result, 8)
failure = ""
try:
    power = 11
catch error:
    failure = error
assert(failure, "TOO_LARGE")
assert(power, 3)
assert(result, 8)
base = 3
power = 5
assert(result, 243)
balance = 3
if balance > 100:
    throw "LIMIT"
try:
    balance **= power
catch error:
    failure = error
assert(failure, "LIMIT")
assert(balance, 3)
power = 2
assert(balance, 3)
assert(result, 9)"#);
}

/// Nucleoid runs a suppose block in a hypothetical world
#[rustfmt::skip]
#[test]
fn runs_a_suppose_block_in_a_hypothetical_world() {
    runner()("threshold = 90\nreading = 40\nalarm = reading > threshold\nstatus = alarm ? \"ALERT\" : \"OK\"\nsuppose reading = 95:\n    assert: status, \"ALERT\"\n    threshold = 99\n    assert: alarm, false\n    margin = threshold - reading\n    assert: margin, 4\n    assert: (affects reading).length, 3\nassert: reading, 40\nassert: threshold, 90\nassert: status, \"OK\"\nassert: (affects reading).length, 2\nsuppose threshold = 30: assert: status, \"ALERT\"\nassert: threshold, 90\nreading = 95\nassert: status, \"ALERT\"");
    let mut run = runner();
    run(r#"threshold = 90
reading = 40
alarm = reading > threshold
status = alarm ? "ALERT" : "OK"
suppose reading = 95:
    assert: status, "ALERT"
    threshold = 99
    assert: alarm, false
    margin = threshold - reading
    assert: margin, 4
    assert: (affects reading).length, 3
assert: reading, 40
assert: threshold, 90
assert: status, "OK"
assert: (affects reading).length, 2
suppose threshold = 30: assert: status, "ALERT"
assert: threshold, 90
reading = 95
assert: status, "ALERT""#);
}

/// Nucleoid discards instances created in a suppose block
#[rustfmt::skip]
#[test]
fn discards_instances_created_in_a_suppose_block() {
    runner()("class Account(balance): this.balance = balance\n$Account.rich = $Account.balance > 100\ntotal = Account.reduce(lambda sum, account: sum + account.balance, 0)\nrich = Account.filter(account -> account.rich).length\nchecking = Account(50)\nsavings = Account(150)\nsuppose Account(500):\n    assert: Account.length, 3\n    assert: total, 700\n    checking.balance = 300\n    assert: checking.rich, true\n    assert: rich, 3\nassert: Account.length, 2\nassert: total, 200\nassert: rich, 1\nassert: checking.rich, false\nsuppose savings.balance = 20: assert: rich, 0\nAccount(400)\nassert: Account.length, 3\nassert: total, 600\nassert: rich, 2");
    let mut run = runner();
    run(r#"class Account(balance): this.balance = balance
$Account.rich = $Account.balance > 100
total = Account.reduce(lambda sum, account: sum + account.balance, 0)
rich = Account.filter(account -> account.rich).length
checking = Account(50)
savings = Account(150)
suppose Account(500):
    assert: Account.length, 3
    assert: total, 700
    checking.balance = 300
    assert: checking.rich, true
    assert: rich, 3
assert: Account.length, 2
assert: total, 200
assert: rich, 1
assert: checking.rich, false
suppose savings.balance = 20: assert: rich, 0
Account(400)
assert: Account.length, 3
assert: total, 600
assert: rich, 2"#);
}

/// Nucleoid keeps the value of a suppose block as a snapshot
#[rustfmt::skip]
#[test]
fn keeps_the_value_of_a_suppose_block_as_a_snapshot() {
    runner()("class Line(price, quantity):\n    this.price = price\n    this.quantity = quantity\n$Line.cost = $Line.price * $Line.quantity\nsubtotal = Line.reduce(lambda sum, line: sum + line.cost, 0)\nshipping = subtotal >= 100 ? 0 : 15\ndue = subtotal + shipping\npens = Line(2, 10)\nink = Line(30, 2)\nbulk = suppose pens.quantity = 30: due\nquote = suppose ink.price = 45: pens.quantity = 5; [subtotal, shipping]\nassert: bulk, 120\nassert: quote, [100, 0]\nassert: due, 95\nink.quantity = 3\nassert: due, 110\nassert: bulk, 120\nassert: quote, [100, 0]");
    let mut run = runner();
    run(r#"class Line(price, quantity):
    this.price = price
    this.quantity = quantity
$Line.cost = $Line.price * $Line.quantity
subtotal = Line.reduce(lambda sum, line: sum + line.cost, 0)
shipping = subtotal >= 100 ? 0 : 15
due = subtotal + shipping
pens = Line(2, 10)
ink = Line(30, 2)
bulk = suppose pens.quantity = 30: due
quote = suppose ink.price = 45: pens.quantity = 5; [subtotal, shipping]
assert: bulk, 120
assert: quote, [100, 0]
assert: due, 95
ink.quantity = 3
assert: due, 110
assert: bulk, 120
assert: quote, [100, 0]"#);
}

/// Nucleoid nests suppose blocks and discards the inner one first
#[rustfmt::skip]
#[test]
fn nests_suppose_blocks_and_discards_the_inner_one_first() {
    runner()("class Seat(taken): this.taken = taken\nfree = Seat.filter(seat -> not seat.taken).length\nfull = not free\nfront = Seat(false)\nback = Seat(true)\nouter = suppose front.taken = true:\n    assert: full, true\n    inner = suppose back.taken = false: [free, full]\n    assert: inner, [1, false]\n    assert: free, 0\n    [inner, full]\nassert: outer, [[1, false], true]\nassert: free, 1\nback.taken = false\nassert: free, 2\nassert: outer, [[1, false], true]");
    let mut run = runner();
    run(r#"class Seat(taken): this.taken = taken
free = Seat.filter(seat -> not seat.taken).length
full = not free
front = Seat(false)
back = Seat(true)
outer = suppose front.taken = true:
    assert: full, true
    inner = suppose back.taken = false: [free, full]
    assert: inner, [1, false]
    assert: free, 0
    [inner, full]
assert: outer, [[1, false], true]
assert: free, 1
back.taken = false
assert: free, 2
assert: outer, [[1, false], true]"#);
}

/// Nucleoid runs header-less suppose blocks over whole populations
#[rustfmt::skip]
#[test]
fn runs_header_less_suppose_blocks_over_whole_populations() {
    runner()("class Account(balance):\n    this.balance = balance\n    def withdraw(amount): this.balance -= amount; return this.balance >= 0\n$Account.interest = $Account.balance * 0.05\ntotal = Account.reduce(lambda sum, account: sum + account.interest, 0)\ncash = Account(100)\nAccount(300)\nsuppose:\n    for account of Account: account.balance *= 2\n    assert: total, 40\ncovered = suppose: cash.withdraw(150)\nassert: covered, false\nassert: cash.balance, 100\ncash.withdraw(50)\nassert: total, 17.5\nassert: covered, false");
    let mut run = runner();
    run(r#"class Account(balance):
    this.balance = balance
    def withdraw(amount): this.balance -= amount; return this.balance >= 0
$Account.interest = $Account.balance * 0.05
total = Account.reduce(lambda sum, account: sum + account.interest, 0)
cash = Account(100)
Account(300)
suppose:
    for account of Account: account.balance *= 2
    assert: total, 40
covered = suppose: cash.withdraw(150)
assert: covered, false
assert: cash.balance, 100
cash.withdraw(50)
assert: total, 17.5
assert: covered, false"#);
}

/// Nucleoid returns hypothetical answers from functions and methods
#[rustfmt::skip]
#[test]
fn returns_hypothetical_answers_from_functions_and_methods() {
    runner()("class Account(balance):\n    this.balance = balance\n    def projected(deposit): suppose this.balance += deposit: return this.tier\n$Account.tier = $Account.balance >= 1000 ? \"gold\" : \"basic\"\ndef tier_at(account, balance): suppose account.balance = balance: return account.tier\nacct = Account(600)\nassert: acct.projected(500), \"gold\"\nassert: acct.projected(100), \"basic\"\nassert: tier_at(acct, 2000), \"gold\"\nassert: acct.balance, 600\nassert: acct.tier, \"basic\"\n$Account.tier = $Account.balance >= 650 ? \"silver\" : \"basic\"\nassert: acct.projected(100), \"silver\"\nassert: tier_at(acct, 600), \"basic\"");
    let mut run = runner();
    run(r#"class Account(balance):
    this.balance = balance
    def projected(deposit): suppose this.balance += deposit: return this.tier
$Account.tier = $Account.balance >= 1000 ? "gold" : "basic"
def tier_at(account, balance): suppose account.balance = balance: return account.tier
acct = Account(600)
assert: acct.projected(500), "gold"
assert: acct.projected(100), "basic"
assert: tier_at(acct, 2000), "gold"
assert: acct.balance, 600
assert: acct.tier, "basic"
$Account.tier = $Account.balance >= 650 ? "silver" : "basic"
assert: acct.projected(100), "silver"
assert: tier_at(acct, 600), "basic""#);
}

/// Nucleoid keeps suppose as an ordinary name beside suppose blocks
#[rustfmt::skip]
#[test]
fn keeps_suppose_as_an_ordinary_name_beside_suppose_blocks() {
    runner()("suppose = 2\nreading = 40\nscaled = reading * suppose\nsuppose reading = 95: assert: scaled, 190\nsuppose suppose = 3: assert: scaled, 120\nassert: scaled, 80\ndef twice suppose => suppose * 2\nassert: twice(suppose), 4\nbox = {suppose}\nassert: box.suppose, 2\nsuppose reading = 50\nassert: scaled, 100");
    let mut run = runner();
    run(r#"suppose = 2
reading = 40
scaled = reading * suppose
suppose reading = 95: assert: scaled, 190
suppose suppose = 3: assert: scaled, 120
assert: scaled, 80
def twice suppose => suppose * 2
assert: twice(suppose), 4
box = {suppose}
assert: box.suppose, 2
suppose reading = 50
assert: scaled, 100"#);
}

/// Nucleoid supposes a changed class level rule
#[rustfmt::skip]
#[test]
fn supposes_a_changed_class_level_rule() {
    runner()("class Order(amount): this.amount = amount\n$Order.fee = $Order.amount * 0.02\nOrder(100)\nlarge = Order(400)\nfees = Order.reduce(lambda sum, order: sum + order.fee, 0)\nfloor = suppose $Order.fee = Math.max($Order.amount * 0.02, 5): fees\nwaived = suppose $Order.fee = 0: [fees, large.fee]\nassert: floor, 13\nassert: waived, [0, 0]\nassert: fees, 10\nassert: large.fee, 8\nOrder(500)\nassert: fees, 20\nassert: floor, 13");
    let mut run = runner();
    run(r#"class Order(amount): this.amount = amount
$Order.fee = $Order.amount * 0.02
Order(100)
large = Order(400)
fees = Order.reduce(lambda sum, order: sum + order.fee, 0)
floor = suppose $Order.fee = Math.max($Order.amount * 0.02, 5): fees
waived = suppose $Order.fee = 0: [fees, large.fee]
assert: floor, 13
assert: waived, [0, 0]
assert: fees, 10
assert: large.fee, 8
Order(500)
assert: fees, 20
assert: floor, 13"#);
}

/// Nucleoid applies a value assignment once
#[rustfmt::skip]
#[test]
fn applies_a_value_assignment_once() {
    runner()("stock = 10\nqty = 3\nlow = stock < 8\nstock := stock - qty\nassert: stock, 7\nassert: low, true\nqty = 5\nassert: stock, 7\nstock := stock - qty\nassert: stock, 2\nassert: stock@previous, 7\nassert: (why stock)[0].rule, \"stock = 2\"\nassert: (why stock)[0].state, \"stated\"\nassert: (affects qty).length, 0\nstock\n# return: 2");
    let mut run = runner();
    run("stock = 10");
    run("qty = 3");
    run("low = stock < 8");
    run("stock := stock - qty");
    assert_eq!(run("stock"), 7);
    assert_eq!(run("low"), true);
    run("qty = 5");
    assert_eq!(run("stock"), 7);
    run("stock := stock - qty");
    assert_eq!(run("stock"), 2);
    assert_eq!(run("stock@previous"), 7);
    assert_eq!(run("(why stock)[0].rule"), "stock = 2");
    assert_eq!(run("(why stock)[0].state"), "stated");
    assert_eq!(run("(affects qty).length"), 0);
    run(r#"stock

# return: 2"#);
}

/// Nucleoid replaces a standing relationship with a value assignment
#[rustfmt::skip]
#[test]
fn replaces_a_standing_relationship_with_a_value_assignment() {
    runner()("price = 4\ncount = 3\ntotal = price * count\ndue = total + 1\nassert: due, 13\ntotal := price * 2\nassert: total, 8\nassert: due, 9\nprice = 10\ncount = 5\nassert: total, 8\nassert: due, 9\nassert: (why total)[0].from, []\nassert: (affects price).length, 0\nbox = {width: 2, height: 3}\nbox.area = box.width * box.height\nbox.area := box.width\nbox.width = 7\nassert: box.area, 2\ntotal = price * count\nassert: due, 51\nassert: (affects price).length, 2\ntotal := price * count\n# return: 50");
    let mut run = runner();
    run("price = 4");
    run("count = 3");
    run("total = price * count");
    run("due = total + 1");
    assert_eq!(run("due"), 13);
    run("total := price * 2");
    assert_eq!(run("total"), 8);
    assert_eq!(run("due"), 9);
    run("price = 10");
    run("count = 5");
    assert_eq!(run("total"), 8);
    assert_eq!(run("due"), 9);
    assert_eq!(run("(why total)[0].from"), serde_json::json!([]));
    assert_eq!(run("(affects price).length"), 0);
    run("box = {width: 2, height: 3}");
    run("box.area = box.width * box.height");
    run("box.area := box.width");
    run("box.width = 7");
    assert_eq!(run("box.area"), 2);
    run("total = price * count");
    assert_eq!(run("due"), 51);
    assert_eq!(run("(affects price).length"), 2);
    run(r#"total := price * count

# return: 50"#);
}

/// Nucleoid assigns instance properties once with value assignments
#[rustfmt::skip]
#[test]
fn assigns_instance_properties_once_with_value_assignments() {
    runner()("catalog = {price: 4}\nclass Order(qty):\n    this.qty = qty\n    this.price := catalog.price\n    this.total = this.qty * this.price\n    def reprice(): this.price := catalog.price\nfirst = Order(3)\ncatalog.price = 9\nsecond = Order(1)\nassert: first.price, 4\nassert: second.price, 9\nassert: first.total, 12\nfirst.qty = 5\nassert: first.total, 20\nfirst.reprice()\nassert: first.total, 45\ncatalog.price = 2\nassert: first.price, 9\nassert: (why first.price)[0].from, []\nsecond.price := catalog.price + second.qty\ncatalog.price = 6\nassert: second.price, 3\nassert: second.total, 3");
    let mut run = runner();
    run("catalog = {price: 4}");
    run(r#"class Order(qty):
    this.qty = qty
    this.price := catalog.price
    this.total = this.qty * this.price
    def reprice(): this.price := catalog.price"#);
    run("first = Order(3)");
    run("catalog.price = 9");
    run("second = Order(1)");
    assert_eq!(run("first.price"), 4);
    assert_eq!(run("second.price"), 9);
    assert_eq!(run("first.total"), 12);
    run("first.qty = 5");
    assert_eq!(run("first.total"), 20);
    run("first.reprice()");
    assert_eq!(run("first.total"), 45);
    run("catalog.price = 2");
    assert_eq!(run("first.price"), 9);
    assert_eq!(run("(why first.price)[0].from"), serde_json::json!([]));
    run("second.price := catalog.price + second.qty");
    run("catalog.price = 6");
    assert_eq!(run("second.price"), 3);
    assert_eq!(run("second.total"), 3);
}

/// Nucleoid records no dependency through calls in a value assignment
#[rustfmt::skip]
#[test]
fn records_no_dependency_through_calls_in_a_value_assignment() {
    runner()("rate = 2\ndef fee(amount) => amount * rate\nclass Item(price):\n    this.price = price\n    def taxed() => this.price * rate\npen = Item(3)\nbook = Item(5)\ncharged := fee(10)\nquote := pen.taxed()\nspent := Item.reduce(lambda sum, item: sum + item.price, 0)\ncount := Item.length\nlive = Item.length\nrate = 3\npen.price = 4\nItem(7)\nassert: charged, 20\nassert: quote, 6\nassert: spent, 8\nassert: count, 2\nassert: live, 3\nassert: fee(10), 30\nassert: pen.taxed(), 12");
    let mut run = runner();
    run("rate = 2");
    run("def fee(amount) => amount * rate");
    run(r#"class Item(price):
    this.price = price
    def taxed() => this.price * rate"#);
    run("pen = Item(3)");
    run("book = Item(5)");
    run("charged := fee(10)");
    run("quote := pen.taxed()");
    run("spent := Item.reduce(lambda sum, item: sum + item.price, 0)");
    run("count := Item.length");
    run("live = Item.length");
    run("rate = 3");
    run("pen.price = 4");
    run("Item(7)");
    assert_eq!(run("charged"), 20);
    assert_eq!(run("quote"), 6);
    assert_eq!(run("spent"), 8);
    assert_eq!(run("count"), 2);
    assert_eq!(run("live"), 3);
    assert_eq!(run("fee(10)"), 30);
    assert_eq!(run("pen.taxed()"), 12);
}

/// Nucleoid runs a value assignment whenever its conditional branch runs
#[rustfmt::skip]
#[test]
fn runs_a_value_assignment_whenever_its_conditional_branch_runs() {
    runner()("paid = false\nstock = 10\nqty = 3\nif paid:\n    shipped := qty\n    stock := stock - qty\npaid = true\nassert: stock, 7\nassert: shipped, 3\nqty = 4\nstock = 20\nassert: stock, 20\nassert: shipped, 3\npaid = false\npaid = true\nassert: stock, 16\nassert: shipped, 4\nclass Order(qty, paid):\n    this.qty = qty\n    this.paid = paid\nif $Order.paid:\n    $Order.taken := $Order.qty\n    stock := stock - $Order.qty\nsmall = Order(2, true)\nlarge = Order(5, false)\nassert: stock, 14\nlarge.paid = true\nassert: stock, 9\nlarge.qty = 6\nassert: large.taken, 5\nassert: stock, 9\nassert: small.taken, 2");
    let mut run = runner();
    run("paid = false");
    run("stock = 10");
    run("qty = 3");
    run(r#"if paid:
    shipped := qty
    stock := stock - qty"#);
    run("paid = true");
    assert_eq!(run("stock"), 7);
    assert_eq!(run("shipped"), 3);
    run("qty = 4");
    run("stock = 20");
    assert_eq!(run("stock"), 20);
    assert_eq!(run("shipped"), 3);
    run("paid = false");
    run("paid = true");
    assert_eq!(run("stock"), 16);
    assert_eq!(run("shipped"), 4);
    run(r#"class Order(qty, paid):
    this.qty = qty
    this.paid = paid"#);
    run(r#"if $Order.paid:
    $Order.taken := $Order.qty
    stock := stock - $Order.qty"#);
    run("small = Order(2, true)");
    run("large = Order(5, false)");
    assert_eq!(run("stock"), 14);
    run("large.paid = true");
    assert_eq!(run("stock"), 9);
    run("large.qty = 6");
    assert_eq!(run("large.taken"), 5);
    assert_eq!(run("stock"), 9);
    assert_eq!(run("small.taken"), 2);
}

/// Nucleoid uses value assignments in functions blocks and loops
#[rustfmt::skip]
#[test]
fn uses_value_assignments_in_functions_blocks_and_loops() {
    runner()("bonus = 5\ndef award(points):\n    granted := points * bonus\n    return granted\nprize = award(2)\nclass Player(name): this.name = name\nada = Player(\"Ada\")\nbo = Player(\"Bo\")\ndef roster():\n    for player of Player: last := player.name\n    return last\nfinal = roster()\na = 1\nb = 2\n{\n    sum := a + b\n    double = sum * 2\n}\nbonus = 7\nbo.name = \"Ben\"\na = 10\nassert: prize, 10\nassert: granted, 10\nassert: final, \"Bo\"\nassert: double, 6\nassert: sum, 3\nassert: award(2), 14\nassert: roster(), \"Ben\"");
    let mut run = runner();
    run("bonus = 5");
    run(r#"def award(points):
    granted := points * bonus
    return granted"#);
    run("prize = award(2)");
    run("class Player(name): this.name = name");
    run(r#"ada = Player("Ada")"#);
    run(r#"bo = Player("Bo")"#);
    run(r#"def roster():
    for player of Player: last := player.name
    return last"#);
    run("final = roster()");
    run("a = 1");
    run("b = 2");
    run(r#"{
    sum := a + b
    double = sum * 2
}"#);
    run("bonus = 7");
    run(r#"bo.name = "Ben""#);
    run("a = 10");
    assert_eq!(run("prize"), 10);
    assert_eq!(run("granted"), 10);
    assert_eq!(run("final"), "Bo");
    assert_eq!(run("double"), 6);
    assert_eq!(run("sum"), 3);
    assert_eq!(run("award(2)"), 14);
    assert_eq!(run("roster()"), "Ben");
}

/// Nucleoid keeps colon readings beside value assignments
#[rustfmt::skip]
#[test]
fn keeps_colon_readings_beside_value_assignments() {
    let mut run = runner();
    run(r#"class Meter:
    reading: int
    def init(reading): this.reading = reading
meter = Meter(4)
scale = 2
label := `${meter.reading:=scale}`
level := meter.reading > 3 ? "high" : "low"
box := {size: meter.reading * scale}
double = lambda value: value * scale
parts = [1, 2, 3][1:3]
ready := all part in parts: part > 1
if ready: total := double(meter.reading)
meter.reading = 1
scale = 5
parts = [0]
assert: label, "4"
assert: level, "high"
assert: box.size, 8
assert: ready, true
assert: total, 8
assert: double(2), 10"#);
}

/// Nucleoid rolls back failed value assignments
#[rustfmt::skip]
#[test]
fn rolls_back_failed_value_assignments() {
    runner()("def checked(amount):\n    if amount > 100: throw \"TOO_LARGE\"\n    return amount\nlimit = 50\nstock = 10\nstock := checked(limit)\nassert: stock, 50\nfailure = \"\"\ntry:\n    stock := checked(limit * 3)\ncatch error:\n    failure = error\nassert: failure, \"TOO_LARGE\"\nassert: stock, 50\nbalance = 3\nif balance > 100: throw \"LIMIT\"\ntry:\n    balance := stock * 4\ncatch error:\n    failure = error\nassert: failure, \"LIMIT\"\nassert: balance, 3\ntry:\n    missing.total := stock\ncatch error:\n    failure = error\nassert: failure, ReferenceError(\"missing is not defined\")\nlimit = 1000\nbalance := stock + 40\nstock = 70\nassert: stock, 70\nassert: balance, 90");
    let mut run = runner();
    run(r#"def checked(amount):
    if amount > 100: throw "TOO_LARGE"
    return amount
limit = 50
stock = 10
stock := checked(limit)
assert: stock, 50
failure = ""
try:
    stock := checked(limit * 3)
catch error:
    failure = error
assert: failure, "TOO_LARGE"
assert: stock, 50
balance = 3
if balance > 100: throw "LIMIT"
try:
    balance := stock * 4
catch error:
    failure = error
assert: failure, "LIMIT"
assert: balance, 3
try:
    missing.total := stock
catch error:
    failure = error
assert: failure, ReferenceError("missing is not defined")
limit = 1000
balance := stock + 40
stock = 70
assert: stock, 70
assert: balance, 90"#);
}

/// Nucleoid totals lists with sum
#[rustfmt::skip]
#[test]
fn totals_lists_with_sum() {
    runner()("moves = [4, -6, 10]\nnet = moves.sum()\ntravelled = moves.sum(move -> Math.abs(move))\nassert: net, 8\nassert: travelled, 20\nmoves.push(-5)\nassert: net, 3\nassert: travelled, 25\nmoves.pop()\nmoves.pop()\nassert: net, -2\nassert: travelled, 10\nmoves = []\nassert: net, 0\nassert: travelled, 0\ncart = {items: [0.1, 0.2]}\npaid = cart.items.sum()\nsmall = cart.items.filter(item -> item < 0.25).sum()\nassert: paid, 0.30000000000000004\ncart.items.push(0.3)\nassert: paid, 0.6000000000000001\nassert: small, 0.30000000000000004");
    let mut run = runner();
    run("moves = [4, -6, 10]");
    run("net = moves.sum()");
    run("travelled = moves.sum(move -> Math.abs(move))");
    assert_eq!(run("net"), 8);
    assert_eq!(run("travelled"), 20);
    run("moves.push(-5)");
    assert_eq!(run("net"), 3);
    assert_eq!(run("travelled"), 25);
    run("moves.pop()");
    run("moves.pop()");
    assert_eq!(run("net"), -2);
    assert_eq!(run("travelled"), 10);
    run("moves = []");
    assert_eq!(run("net"), 0);
    assert_eq!(run("travelled"), 0);
    run("cart = {items: [0.1, 0.2]}");
    run("paid = cart.items.sum()");
    run("small = cart.items.filter(item -> item < 0.25).sum()");
    assert_eq!(run("paid"), 0.30000000000000004);
    run("cart.items.push(0.3)");
    assert_eq!(run("paid"), 0.6000000000000001);
    assert_eq!(run("small"), 0.30000000000000004);
}

/// Nucleoid totals class instance populations with sum
#[rustfmt::skip]
#[test]
fn totals_class_instance_populations_with_sum() {
    runner()("class Order(total, rush):\n    this.total = total\n    this.rush = rush\nfirst = Order(20, false)\nsecond = Order(45, true)\nrevenue = Order.sum(order -> order.total)\nrushed = Order.filter(order -> order.rush).sum(order -> order.total)\nrecent = Order[-2:].sum(order -> order.total)\nassert: revenue, 65\nassert: rushed, 45\nassert: recent, 65\nthird = Order(30, true)\nassert: revenue, 95\nassert: rushed, 75\nassert: recent, 75\nfirst.total = 50\nassert: revenue, 125\ndelete third.total\nassert: revenue, null\ndelete third.rush\ndelete third\nassert: revenue, 95\nassert: rushed, 45\nclass Refund(amount): this.amount = amount\nrefunds = Refund.sum(refund -> refund.amount)\nassert: refunds, 0\nRefund(5)\nassert: refunds, 5");
    let mut run = runner();
    run(r#"class Order(total, rush):
    this.total = total
    this.rush = rush"#);
    run("first = Order(20, false)");
    run("second = Order(45, true)");
    run("revenue = Order.sum(order -> order.total)");
    run("rushed = Order.filter(order -> order.rush).sum(order -> order.total)");
    run("recent = Order[-2:].sum(order -> order.total)");
    assert_eq!(run("revenue"), 65);
    assert_eq!(run("rushed"), 45);
    assert_eq!(run("recent"), 65);
    run("third = Order(30, true)");
    assert_eq!(run("revenue"), 95);
    assert_eq!(run("rushed"), 75);
    assert_eq!(run("recent"), 75);
    run("first.total = 50");
    assert_eq!(run("revenue"), 125);
    run("delete third.total");
    assert_eq!(run("revenue"), serde_json::Value::Null);
    run("delete third.rush");
    run("delete third");
    assert_eq!(run("revenue"), 95);
    assert_eq!(run("rushed"), 45);
    run("class Refund(amount): this.amount = amount");
    run("refunds = Refund.sum(refund -> refund.amount)");
    assert_eq!(run("refunds"), 0);
    run("Refund(5)");
    assert_eq!(run("refunds"), 5);
}

/// Nucleoid converts summed values to numbers
#[rustfmt::skip]
#[test]
fn converts_summed_values_to_numbers() {
    runner()("flags = [true, false, true]\ncount = flags.sum()\nentries = [2, \"3\", \" 4 \", \"\", null, [5], []]\ntotal = entries.sum()\nassert: count, 2\nassert: total, 14\nflags.push(true)\nentries.push(\"x\")\nassert: count, 3\nassert: Number.isNaN(total), true\nentries.pop()\nentries.push(Number.POSITIVE_INFINITY)\nassert: total, Number.POSITIVE_INFINITY\nclass Item(price): this.price = price\npen = Item(3)\nbook = Item(12)\nspent = Item.sum(item -> item.price)\nassert: spent, 15\ngift = Item(null)\nassert: spent, null\ngift.price = 5\nassert: spent, 20");
    let mut run = runner();
    run("flags = [true, false, true]");
    run("count = flags.sum()");
    run(r#"entries = [2, "3", " 4 ", "", null, [5], []]"#);
    run("total = entries.sum()");
    assert_eq!(run("count"), 2);
    assert_eq!(run("total"), 14);
    run("flags.push(true)");
    run(r#"entries.push("x")"#);
    assert_eq!(run("count"), 3);
    assert_eq!(run("Number.isNaN(total)"), true);
    run("entries.pop()");
    run("entries.push(Number.POSITIVE_INFINITY)");
    {
        let actual = run("total");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("class Item(price): this.price = price");
    run("pen = Item(3)");
    run("book = Item(12)");
    run("spent = Item.sum(item -> item.price)");
    assert_eq!(run("spent"), 15);
    run("gift = Item(null)");
    assert_eq!(run("spent"), serde_json::Value::Null);
    run("gift.price = 5");
    assert_eq!(run("spent"), 20);
}

/// Nucleoid calls the sum selector once per item with only the item
#[rustfmt::skip]
#[test]
fn calls_the_sum_selector_once_per_item_with_only_the_item() {
    runner()("seen = []\ndef weigh(item, position):\n    seen.push(position ?? item)\n    return item * item\nitems = [3, 1, 2]\nweighed := items.sum(weigh)\nassert: weighed, 14\nassert: seen, [3, 1, 2]\nfloor = 2\ndef price item => Math.max(item, floor)\ncost = items.sum(price)\nassert: cost, 7\nfloor = 3\nassert: cost, 9\nitems.push(4)\nassert: cost, 13\nassert: seen, [3, 1, 2]\nmissing = null\nskipped = missing.sum(weigh(9))\nassert: skipped, null\nassert: seen, [3, 1, 2]");
    let mut run = runner();
    run("seen = []");
    run(r#"def weigh(item, position):
    seen.push(position ?? item)
    return item * item"#);
    run("items = [3, 1, 2]");
    run("weighed := items.sum(weigh)");
    assert_eq!(run("weighed"), 14);
    assert_eq!(run("seen"), serde_json::json!([3, 1, 2]));
    run("floor = 2");
    run("def price item => Math.max(item, floor)");
    run("cost = items.sum(price)");
    assert_eq!(run("cost"), 7);
    run("floor = 3");
    assert_eq!(run("cost"), 9);
    run("items.push(4)");
    assert_eq!(run("cost"), 13);
    assert_eq!(run("seen"), serde_json::json!([3, 1, 2]));
    run("missing = null");
    run("skipped = missing.sum(weigh(9))");
    assert_eq!(run("skipped"), serde_json::Value::Null);
    assert_eq!(run("seen"), serde_json::json!([3, 1, 2]));
}

/// Nucleoid keeps sums live in class level rules
#[rustfmt::skip]
#[test]
fn keeps_sums_live_in_class_level_rules() {
    runner()("class Order(name): this.name = name\nclass Line(order, price, qty):\n    this.order = order\n    this.price = price\n    this.qty = qty\n$Line.cost = $Line.price * $Line.qty\n$Order.total = Line.filter(line -> line.order == $Order).sum(line -> line.cost)\n$Order.units = Line.filter(line -> line.order == $Order).sum(line -> line.qty)\nnorth = Order(\"north\")\nsouth = Order(\"south\")\nLine(north, 2, 3)\npen = Line(south, 4, 2)\nLine(north, 5, 1)\nassert: north.total, 11\nassert: south.total, 8\nassert: north.units, 4\npen.qty = 5\nassert: south.total, 20\nassert: south.units, 5\ngrand = Order.sum(order -> order.total)\nassert: grand, 31\nwest = Order(\"west\")\nassert: west.total, 0\nLine(west, 1, 1)\nassert: grand, 32\npen.price = 1\nassert: grand, 17");
    let mut run = runner();
    run("class Order(name): this.name = name");
    run(r#"class Line(order, price, qty):
    this.order = order
    this.price = price
    this.qty = qty"#);
    run("$Line.cost = $Line.price * $Line.qty");
    run("$Order.total = Line.filter(line -> line.order == $Order).sum(line -> line.cost)");
    run("$Order.units = Line.filter(line -> line.order == $Order).sum(line -> line.qty)");
    run(r#"north = Order("north")"#);
    run(r#"south = Order("south")"#);
    run("Line(north, 2, 3)");
    run("pen = Line(south, 4, 2)");
    run("Line(north, 5, 1)");
    assert_eq!(run("north.total"), 11);
    assert_eq!(run("south.total"), 8);
    assert_eq!(run("north.units"), 4);
    run("pen.qty = 5");
    assert_eq!(run("south.total"), 20);
    assert_eq!(run("south.units"), 5);
    run("grand = Order.sum(order -> order.total)");
    assert_eq!(run("grand"), 31);
    run(r#"west = Order("west")"#);
    assert_eq!(run("west.total"), 0);
    run("Line(west, 1, 1)");
    assert_eq!(run("grand"), 32);
    run("pen.price = 1");
    assert_eq!(run("grand"), 17);
}

/// Nucleoid uses sums in functions methods and conditional expressions
#[rustfmt::skip]
#[test]
fn uses_sums_in_functions_methods_and_conditional_expressions() {
    runner()("class Team(name, scores):\n    this.name = name\n    this.scores = scores\n    def points(): return this.scores.sum()\ndef squares(team) => team.scores.sum(score -> score * score)\nred = Team(\"red\", [3, 4])\nblue = Team(\"blue\", [5])\nleague = Team.sum(team -> team.points())\nleader = red.points() > blue.points() ? red.name : blue.name\nspread = squares(red)\nassert: league, 12\nassert: leader, \"red\"\nassert: spread, 25\nblue.scores.push(4)\nred.scores.push(1)\nTeam(\"green\", [2])\nassert: league, 19\nassert: leader, \"blue\"\nassert: spread, 26");
    let mut run = runner();
    run(r#"class Team(name, scores):
    this.name = name
    this.scores = scores
    def points(): return this.scores.sum()"#);
    run("def squares(team) => team.scores.sum(score -> score * score)");
    run(r#"red = Team("red", [3, 4])"#);
    run(r#"blue = Team("blue", [5])"#);
    run("league = Team.sum(team -> team.points())");
    run("leader = red.points() > blue.points() ? red.name : blue.name");
    run("spread = squares(red)");
    assert_eq!(run("league"), 12);
    assert_eq!(run("leader"), "red");
    assert_eq!(run("spread"), 25);
    run("blue.scores.push(4)");
    run("red.scores.push(1)");
    run(r#"Team("green", [2])"#);
    assert_eq!(run("league"), 19);
    assert_eq!(run("leader"), "blue");
    assert_eq!(run("spread"), 26);
}

/// Nucleoid keeps sum as an ordinary name beside the sum query
#[rustfmt::skip]
#[test]
fn keeps_sum_as_an_ordinary_name_beside_the_sum_query() {
    runner()("sum = 2\nreport = {sum: 5}\ndef double sum => sum * 2\nclass Bag(items):\n    this.items = items\n    def sum(): return this.items.sum() + 1\nbag = Bag([1, 2])\ntotal = bag.items.sum() + sum + report.sum\nbags = Bag.sum(each -> each.sum())\nassert: total, 10\nassert: bag.sum(), 4\nassert: bags, 4\nassert: double(sum), 4\nassert: bag.items.sum, null\nbag.items.push(3)\nsum = 4\nBag([5])\nassert: total, 15\nassert: bags, 13");
    let mut run = runner();
    run("sum = 2");
    run("report = {sum: 5}");
    run("def double sum => sum * 2");
    run(r#"class Bag(items):
    this.items = items
    def sum(): return this.items.sum() + 1"#);
    run("bag = Bag([1, 2])");
    run("total = bag.items.sum() + sum + report.sum");
    run("bags = Bag.sum(each -> each.sum())");
    assert_eq!(run("total"), 10);
    assert_eq!(run("bag.sum()"), 4);
    assert_eq!(run("bags"), 4);
    assert_eq!(run("double(sum)"), 4);
    assert_eq!(run("bag.items.sum"), serde_json::Value::Null);
    run("bag.items.push(3)");
    run("sum = 4");
    run("Bag([5])");
    assert_eq!(run("total"), 15);
    assert_eq!(run("bags"), 13);
}

/// Nucleoid rolls back failed sum updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_sum_updates() {
    runner()("def checked(value):\n    if value > 100:\n        throw \"TOO_LARGE\"\n    return value\namounts = [10, 20]\ntotal = amounts.sum(checked)\nassert: total, 30\nfailure = \"\"\ntry:\n    amounts.push(500)\ncatch error:\n    failure = error\nassert: failure, \"TOO_LARGE\"\nassert: amounts, [10, 20]\nassert: total, 30\namounts.push(5)\nassert: total, 35\nclass Item(price): this.price = price\npen = Item(3)\nspent = Item.sum(item -> checked(item.price))\nfailure = \"\"\ntry:\n    Item(500)\ncatch error:\n    failure = error\nassert: failure, \"TOO_LARGE\"\nassert: Item.length, 1\nassert: spent, 3\nItem(4)\nassert: spent, 7");
    let mut run = runner();
    run(r#"def checked(value):
    if value > 100:
        throw "TOO_LARGE"
    return value
amounts = [10, 20]
total = amounts.sum(checked)
assert: total, 30
failure = ""
try:
    amounts.push(500)
catch error:
    failure = error
assert: failure, "TOO_LARGE"
assert: amounts, [10, 20]
assert: total, 30
amounts.push(5)
assert: total, 35
class Item(price): this.price = price
pen = Item(3)
spent = Item.sum(item -> checked(item.price))
failure = ""
try:
    Item(500)
catch error:
    failure = error
assert: failure, "TOO_LARGE"
assert: Item.length, 1
assert: spent, 3
Item(4)
assert: spent, 7"#);
}

/// Nucleoid finds list extremes with max and min
#[rustfmt::skip]
#[test]
fn finds_list_extremes_with_max_and_min() {
    runner()("temps = [18, 24, 9]\nhigh = temps.max()\nlow = temps.min()\nassert: high, 24\nassert: low, 9\ntemps.push(30)\nassert: high, 30\ntemps.pop()\ntemps.pop()\nassert: high, 24\nassert: low, 18\ntemps = []\nassert: high, Number.NEGATIVE_INFINITY\nassert: low, Number.POSITIVE_INFINITY\ntrip = {legs: [{km: 5, mins: 10}, {km: 12, mins: 30}]}\nfastest = trip.legs.max(leg -> leg.km / leg.mins)\nshortest = trip.legs.min(leg -> leg.km)\nassert: fastest, 0.5\nassert: shortest, 5\ntrip.legs.push({km: 3, mins: 4})\nassert: fastest, 0.75\nassert: shortest, 3");
    let mut run = runner();
    run("temps = [18, 24, 9]");
    run("high = temps.max()");
    run("low = temps.min()");
    assert_eq!(run("high"), 24);
    assert_eq!(run("low"), 9);
    run("temps.push(30)");
    assert_eq!(run("high"), 30);
    run("temps.pop()");
    run("temps.pop()");
    assert_eq!(run("high"), 24);
    assert_eq!(run("low"), 18);
    run("temps = []");
    {
        let actual = run("high");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("low");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("trip = {legs: [{km: 5, mins: 10}, {km: 12, mins: 30}]}");
    run("fastest = trip.legs.max(leg -> leg.km / leg.mins)");
    run("shortest = trip.legs.min(leg -> leg.km)");
    assert_eq!(run("fastest"), 0.5);
    assert_eq!(run("shortest"), 5);
    run("trip.legs.push({km: 3, mins: 4})");
    assert_eq!(run("fastest"), 0.75);
    assert_eq!(run("shortest"), 3);
}

/// Nucleoid finds extremes of class instance populations with max and min
#[rustfmt::skip]
#[test]
fn finds_extremes_of_class_instance_populations_with_max_and_min() {
    runner()("class Bid(amount, live):\n    this.amount = amount\n    this.live = live\nhighest = Bid.max(bid -> bid.amount)\nlowest = Bid.min(bid -> bid.amount)\nassert: highest, Number.NEGATIVE_INFINITY\nassert: lowest, Number.POSITIVE_INFINITY\nfirst = Bid(120, true)\nsecond = Bid(90, false)\nthird = Bid(150, true)\nopen = Bid.filter(bid -> bid.live).min(bid -> bid.amount)\nrecent = Bid[-2:].min(bid -> bid.amount)\nassert: highest, 150\nassert: lowest, 90\nassert: open, 120\nassert: recent, 90\nsecond.amount = 200\nfirst.live = false\nassert: highest, 200\nassert: lowest, 120\nassert: open, 150\nassert: recent, 150\ndelete third.amount\nassert: highest, null\ndelete third.live\ndelete third\nassert: highest, 200\nassert: open, Number.POSITIVE_INFINITY\nassert: recent, 120");
    let mut run = runner();
    run(r#"class Bid(amount, live):
    this.amount = amount
    this.live = live"#);
    run("highest = Bid.max(bid -> bid.amount)");
    run("lowest = Bid.min(bid -> bid.amount)");
    {
        let actual = run("highest");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("lowest");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("first = Bid(120, true)");
    run("second = Bid(90, false)");
    run("third = Bid(150, true)");
    run("open = Bid.filter(bid -> bid.live).min(bid -> bid.amount)");
    run("recent = Bid[-2:].min(bid -> bid.amount)");
    assert_eq!(run("highest"), 150);
    assert_eq!(run("lowest"), 90);
    assert_eq!(run("open"), 120);
    assert_eq!(run("recent"), 90);
    run("second.amount = 200");
    run("first.live = false");
    assert_eq!(run("highest"), 200);
    assert_eq!(run("lowest"), 120);
    assert_eq!(run("open"), 150);
    assert_eq!(run("recent"), 150);
    run("delete third.amount");
    assert_eq!(run("highest"), serde_json::Value::Null);
    run("delete third.live");
    run("delete third");
    assert_eq!(run("highest"), 200);
    {
        let actual = run("open");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("recent"), 120);
}

/// Nucleoid compares extreme values as numbers
#[rustfmt::skip]
#[test]
fn compares_extreme_values_as_numbers() {
    runner()("inputs = [\"10\", \" 7 \", \"9\"]\nlargest = inputs.max()\nsmallest = inputs.min()\nassert: largest, 10\nassert: smallest, 7\ninputs.push(true)\nassert: smallest, 1\ninputs.push(null)\nassert: smallest, 0\nassert: largest, 10\ninputs.push(\"n/a\")\nassert: Number.isNaN(largest), true\nassert: Number.isNaN(smallest), true\ninputs.pop()\ninputs.push(Number.POSITIVE_INFINITY)\nassert: largest, Number.POSITIVE_INFINITY\nassert: smallest, 0\nclass Gauge(level): this.level = level\npump = Gauge(55)\ntank = Gauge(40)\npeak = Gauge.max(gauge -> gauge.level)\nassert: peak, 55\nidle = Gauge(null)\nassert: peak, null\nidle.level = 70\nassert: peak, 70");
    let mut run = runner();
    run(r#"inputs = ["10", " 7 ", "9"]"#);
    run("largest = inputs.max()");
    run("smallest = inputs.min()");
    assert_eq!(run("largest"), 10);
    assert_eq!(run("smallest"), 7);
    run("inputs.push(true)");
    assert_eq!(run("smallest"), 1);
    run("inputs.push(null)");
    assert_eq!(run("smallest"), 0);
    assert_eq!(run("largest"), 10);
    run(r#"inputs.push("n/a")"#);
    assert_eq!(run("Number.isNaN(largest)"), true);
    assert_eq!(run("Number.isNaN(smallest)"), true);
    run("inputs.pop()");
    run("inputs.push(Number.POSITIVE_INFINITY)");
    {
        let actual = run("largest");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("smallest"), 0);
    run("class Gauge(level): this.level = level");
    run("pump = Gauge(55)");
    run("tank = Gauge(40)");
    run("peak = Gauge.max(gauge -> gauge.level)");
    assert_eq!(run("peak"), 55);
    run("idle = Gauge(null)");
    assert_eq!(run("peak"), serde_json::Value::Null);
    run("idle.level = 70");
    assert_eq!(run("peak"), 70);
}

/// Nucleoid calls the extreme selector once per item with only the item
#[rustfmt::skip]
#[test]
fn calls_the_extreme_selector_once_per_item_with_only_the_item() {
    runner()("seen = []\ndef weigh(item, position):\n    seen.push(position ?? item)\n    return item * item\nitems = [3, -4, 2]\ndef heaviest() => items.max(weigh)\nassert: heaviest(), 16\nassert: seen, [3, -4, 2]\nitems.push(5)\nassert: heaviest(), 25\nassert: seen, [3, -4, 2, 3, -4, 2, 5]\ntarget = 4\ndef gap item => Math.abs(item - target)\nspread = items.max(gap)\nassert: spread, 8\ntarget = -1\nassert: spread, 6\nmissing = null\nskipped = missing.min(weigh(9))\nassert: skipped, null\nassert: seen, [3, -4, 2, 3, -4, 2, 5]");
    let mut run = runner();
    run("seen = []");
    run(r#"def weigh(item, position):
    seen.push(position ?? item)
    return item * item"#);
    run("items = [3, -4, 2]");
    run("def heaviest() => items.max(weigh)");
    assert_eq!(run("heaviest()"), 16);
    assert_eq!(run("seen"), serde_json::json!([3, -4, 2]));
    run("items.push(5)");
    assert_eq!(run("heaviest()"), 25);
    assert_eq!(run("seen"), serde_json::json!([3, -4, 2, 3, -4, 2, 5]));
    run("target = 4");
    run("def gap item => Math.abs(item - target)");
    run("spread = items.max(gap)");
    assert_eq!(run("spread"), 8);
    run("target = -1");
    assert_eq!(run("spread"), 6);
    run("missing = null");
    run("skipped = missing.min(weigh(9))");
    assert_eq!(run("skipped"), serde_json::Value::Null);
    assert_eq!(run("seen"), serde_json::json!([3, -4, 2, 3, -4, 2, 5]));
}

/// Nucleoid keeps extremes live in class level rules
#[rustfmt::skip]
#[test]
fn keeps_extremes_live_in_class_level_rules() {
    runner()("class Room(name): this.name = name\nclass Sensor(room, temp):\n    this.room = room\n    this.temp = temp\n$Room.high = Sensor.filter(sensor -> sensor.room == $Room).max(sensor -> sensor.temp)\n$Room.low = Sensor.filter(sensor -> sensor.room == $Room).min(sensor -> sensor.temp)\n$Room.swing = $Room.high - $Room.low\nlab = Room(\"lab\")\nden = Room(\"den\")\nSensor(lab, 21)\nprobe = Sensor(lab, 25)\nSensor(den, 19)\nassert: lab.high, 25\nassert: lab.swing, 4\nassert: den.swing, 0\npeak = Room.max(room -> room.high)\nassert: peak, 25\nprobe.temp = 18\nassert: lab.high, 21\nassert: lab.low, 18\nassert: lab.swing, 3\nattic = Room(\"attic\")\nassert: attic.high, Number.NEGATIVE_INFINITY\nSensor(attic, 30)\nassert: peak, 30");
    let mut run = runner();
    run("class Room(name): this.name = name");
    run(r#"class Sensor(room, temp):
    this.room = room
    this.temp = temp"#);
    run("$Room.high = Sensor.filter(sensor -> sensor.room == $Room).max(sensor -> sensor.temp)");
    run("$Room.low = Sensor.filter(sensor -> sensor.room == $Room).min(sensor -> sensor.temp)");
    run("$Room.swing = $Room.high - $Room.low");
    run(r#"lab = Room("lab")"#);
    run(r#"den = Room("den")"#);
    run("Sensor(lab, 21)");
    run("probe = Sensor(lab, 25)");
    run("Sensor(den, 19)");
    assert_eq!(run("lab.high"), 25);
    assert_eq!(run("lab.swing"), 4);
    assert_eq!(run("den.swing"), 0);
    run("peak = Room.max(room -> room.high)");
    assert_eq!(run("peak"), 25);
    run("probe.temp = 18");
    assert_eq!(run("lab.high"), 21);
    assert_eq!(run("lab.low"), 18);
    assert_eq!(run("lab.swing"), 3);
    run(r#"attic = Room("attic")"#);
    {
        let actual = run("attic.high");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run("Sensor(attic, 30)");
    assert_eq!(run("peak"), 30);
}

/// Nucleoid uses extremes in functions methods and conditional expressions
#[rustfmt::skip]
#[test]
fn uses_extremes_in_functions_methods_and_conditional_expressions() {
    runner()("class Team(name, scores):\n    this.name = name\n    this.scores = scores\n    def best(): return this.scores.max()\ndef worst(team) => team.scores.min()\nred = Team(\"red\", [4, 7, 9])\nblue = Team(\"blue\", [12, 6, 8])\nleader = red.best() > blue.best() ? red.name : blue.name\nchampion = Team.max(team -> team.best())\nsteady = Team.max(team -> team.scores.min())\nfloor = worst(red)\nassert: leader, \"blue\"\nassert: champion, 12\nassert: steady, 6\nassert: floor, 4\nblue.scores.push(3)\nred.scores = [9, 15]\nTeam(\"green\", [5, 13])\nassert: leader, \"red\"\nassert: champion, 15\nassert: steady, 9\nassert: floor, 9");
    let mut run = runner();
    run(r#"class Team(name, scores):
    this.name = name
    this.scores = scores
    def best(): return this.scores.max()"#);
    run("def worst(team) => team.scores.min()");
    run(r#"red = Team("red", [4, 7, 9])"#);
    run(r#"blue = Team("blue", [12, 6, 8])"#);
    run("leader = red.best() > blue.best() ? red.name : blue.name");
    run("champion = Team.max(team -> team.best())");
    run("steady = Team.max(team -> team.scores.min())");
    run("floor = worst(red)");
    assert_eq!(run("leader"), "blue");
    assert_eq!(run("champion"), 12);
    assert_eq!(run("steady"), 6);
    assert_eq!(run("floor"), 4);
    run("blue.scores.push(3)");
    run("red.scores = [9, 15]");
    run(r#"Team("green", [5, 13])"#);
    assert_eq!(run("leader"), "red");
    assert_eq!(run("champion"), 15);
    assert_eq!(run("steady"), 9);
    assert_eq!(run("floor"), 9);
}

/// Nucleoid keeps max and min as ordinary names beside the extreme queries
#[rustfmt::skip]
#[test]
fn keeps_max_and_min_as_ordinary_names_beside_the_extreme_queries() {
    runner()("max = 3\nmin = 1\nlimits = {max: 10, min: 0}\ndef scale min => min * 2\nclass Gauge(readings):\n    this.readings = readings\n    def max(): return this.readings.max() + 1\ngauge = Gauge([4, 6])\nspare = Gauge([2])\ntop = gauge.readings.max() + max + limits.max\nlow = gauge.readings.min() - min - limits.min\npeaks = Gauge.max(each -> each.max())\nassert: top, 19\nassert: low, 3\nassert: gauge.max(), 7\nassert: peaks, 7\nassert: scale(min), 2\nassert: gauge.readings.max, null\nassert: Math.max(max, min), 3\ngauge.readings = [8, 5, 2]\nmax = 5\nGauge([12])\nassert: top, 23\nassert: low, 1\nassert: gauge.max(), 9\nassert: peaks, 13");
    let mut run = runner();
    run("max = 3");
    run("min = 1");
    run("limits = {max: 10, min: 0}");
    run("def scale min => min * 2");
    run(r#"class Gauge(readings):
    this.readings = readings
    def max(): return this.readings.max() + 1"#);
    run("gauge = Gauge([4, 6])");
    run("spare = Gauge([2])");
    run("top = gauge.readings.max() + max + limits.max");
    run("low = gauge.readings.min() - min - limits.min");
    run("peaks = Gauge.max(each -> each.max())");
    assert_eq!(run("top"), 19);
    assert_eq!(run("low"), 3);
    assert_eq!(run("gauge.max()"), 7);
    assert_eq!(run("peaks"), 7);
    assert_eq!(run("scale(min)"), 2);
    assert_eq!(run("gauge.readings.max"), serde_json::Value::Null);
    assert_eq!(run("Math.max(max, min)"), 3);
    run("gauge.readings = [8, 5, 2]");
    run("max = 5");
    run("Gauge([12])");
    assert_eq!(run("top"), 23);
    assert_eq!(run("low"), 1);
    assert_eq!(run("gauge.max()"), 9);
    assert_eq!(run("peaks"), 13);
}

/// Nucleoid rolls back failed extreme updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_extreme_updates() {
    runner()("def checked(value):\n    if value > 100:\n        throw \"TOO_HOT\"\n    return value\ntemps = [35, 20]\nhottest = temps.max(checked)\nassert: hottest, 35\nfailure = \"\"\ntry:\n    temps.push(500)\ncatch error:\n    failure = error\nassert: failure, \"TOO_HOT\"\nassert: temps, [35, 20]\nassert: hottest, 35\ntemps.push(40)\nassert: hottest, 40\nclass Probe(temp): this.temp = temp\nProbe(12)\nProbe(30)\ncoldest = Probe.min(probe -> checked(probe.temp))\nfailure = \"\"\ntry:\n    Probe(500)\ncatch error:\n    failure = error\nassert: failure, \"TOO_HOT\"\nassert: Probe.length, 2\nassert: coldest, 12\nProbe(9)\nProbe(4)\nassert: coldest, 4");
    let mut run = runner();
    run(r#"def checked(value):
    if value > 100:
        throw "TOO_HOT"
    return value
temps = [35, 20]
hottest = temps.max(checked)
assert: hottest, 35
failure = ""
try:
    temps.push(500)
catch error:
    failure = error
assert: failure, "TOO_HOT"
assert: temps, [35, 20]
assert: hottest, 35
temps.push(40)
assert: hottest, 40
class Probe(temp): this.temp = temp
Probe(12)
Probe(30)
coldest = Probe.min(probe -> checked(probe.temp))
failure = ""
try:
    Probe(500)
catch error:
    failure = error
assert: failure, "TOO_HOT"
assert: Probe.length, 2
assert: coldest, 12
Probe(9)
Probe(4)
assert: coldest, 4"#);
}

/// Nucleoid removes duplicate list items with distinct
#[rustfmt::skip]
#[test]
fn removes_duplicate_list_items_with_distinct() {
    runner()("visits = [\"lab\", \"den\", \"lab\", \"hall\", \"den\"]\nrooms = visits.distinct()\nassert: rooms, [\"lab\", \"den\", \"hall\"]\nvisits.push(\"attic\")\nvisits.push(\"hall\")\nassert: rooms, [\"lab\", \"den\", \"hall\", \"attic\"]\nvisits.shift()\nassert: rooms, [\"den\", \"lab\", \"hall\", \"attic\"]\nvisits = []\nassert: rooms, []\ncart = {lines: [{sku: \"pen\", qty: 2}, {sku: \"ink\", qty: 1}, {sku: \"pen\", qty: 5}]}\nskus = cart.lines.distinct(line -> line.sku)\nbulk = cart.lines.filter(line -> line.qty > 1).distinct(line -> line.sku)\nassert: skus, [\"pen\", \"ink\"]\nassert: bulk, [\"pen\"]\ncart.lines.push({sku: \"cap\", qty: 3})\nassert: skus, [\"pen\", \"ink\", \"cap\"]\nassert: bulk, [\"pen\", \"cap\"]");
    let mut run = runner();
    run(r#"visits = ["lab", "den", "lab", "hall", "den"]"#);
    run("rooms = visits.distinct()");
    assert_eq!(run("rooms"), serde_json::json!(["lab", "den", "hall"]));
    run(r#"visits.push("attic")"#);
    run(r#"visits.push("hall")"#);
    assert_eq!(run("rooms"), serde_json::json!(["lab", "den", "hall", "attic"]));
    run("visits.shift()");
    assert_eq!(run("rooms"), serde_json::json!(["den", "lab", "hall", "attic"]));
    run("visits = []");
    assert_eq!(run("rooms"), serde_json::json!([]));
    run(r#"cart = {lines: [{sku: "pen", qty: 2}, {sku: "ink", qty: 1}, {sku: "pen", qty: 5}]}"#);
    run("skus = cart.lines.distinct(line -> line.sku)");
    run("bulk = cart.lines.filter(line -> line.qty > 1).distinct(line -> line.sku)");
    assert_eq!(run("skus"), serde_json::json!(["pen", "ink"]));
    assert_eq!(run("bulk"), serde_json::json!(["pen"]));
    run(r#"cart.lines.push({sku: "cap", qty: 3})"#);
    assert_eq!(run("skus"), serde_json::json!(["pen", "ink", "cap"]));
    assert_eq!(run("bulk"), serde_json::json!(["pen", "cap"]));
}

/// Nucleoid lists the distinct values of class instance populations
#[rustfmt::skip]
#[test]
fn lists_the_distinct_values_of_class_instance_populations() {
    runner()("class Order(region, rush):\n    this.region = region\n    this.rush = rush\nregions = Order.distinct(order -> order.region)\nassert: regions, []\nfirst = Order(\"north\", false)\nsecond = Order(\"south\", true)\nthird = Order(\"north\", true)\nrushed = Order.filter(order -> order.rush).distinct(order -> order.region)\nrecent = Order[-3:].distinct(order -> order.region)\nassert: regions, [\"north\", \"south\"]\nassert: rushed, [\"south\", \"north\"]\nassert: recent, [\"north\", \"south\"]\nfirst.region = \"east\"\nOrder(\"south\", true)\nassert: regions, [\"east\", \"south\", \"north\"]\nassert: rushed, [\"south\", \"north\"]\nassert: recent, [\"south\", \"north\"]\ndelete third.region\nassert: regions, [\"east\", \"south\", null]\ndelete third.rush\ndelete third\nassert: regions, [\"east\", \"south\"]\nassert: rushed, [\"south\"]\nassert: recent, [\"east\", \"south\"]\nOrder(\"east\", false)\nassert: recent, [\"south\", \"east\"]");
    let mut run = runner();
    run(r#"class Order(region, rush):
    this.region = region
    this.rush = rush"#);
    run("regions = Order.distinct(order -> order.region)");
    assert_eq!(run("regions"), serde_json::json!([]));
    run(r#"first = Order("north", false)"#);
    run(r#"second = Order("south", true)"#);
    run(r#"third = Order("north", true)"#);
    run("rushed = Order.filter(order -> order.rush).distinct(order -> order.region)");
    run("recent = Order[-3:].distinct(order -> order.region)");
    assert_eq!(run("regions"), serde_json::json!(["north", "south"]));
    assert_eq!(run("rushed"), serde_json::json!(["south", "north"]));
    assert_eq!(run("recent"), serde_json::json!(["north", "south"]));
    run(r#"first.region = "east""#);
    run(r#"Order("south", true)"#);
    assert_eq!(run("regions"), serde_json::json!(["east", "south", "north"]));
    assert_eq!(run("rushed"), serde_json::json!(["south", "north"]));
    assert_eq!(run("recent"), serde_json::json!(["south", "north"]));
    run("delete third.region");
    assert_eq!(run("regions"), serde_json::json!(["east", "south", null]));
    run("delete third.rush");
    run("delete third");
    assert_eq!(run("regions"), serde_json::json!(["east", "south"]));
    assert_eq!(run("rushed"), serde_json::json!(["south"]));
    assert_eq!(run("recent"), serde_json::json!(["east", "south"]));
    run(r#"Order("east", false)"#);
    assert_eq!(run("recent"), serde_json::json!(["south", "east"]));
}

/// Nucleoid compares distinct values with deep equality
#[rustfmt::skip]
#[test]
fn compares_distinct_values_with_deep_equality() {
    runner()("inputs = [2, \"2\", true, 2, null, \"2\", null]\nkinds = inputs.distinct()\nassert: kinds, [2, \"2\", true, null]\ninputs.push(1)\ninputs.push(-0)\ninputs.push(0)\nassert: kinds, [2, \"2\", true, null, 1, 0]\nassert: String(1 / kinds[5]), \"-Infinity\"\ninputs.push(0 / 0)\ninputs.push(0 / 0)\nassert: kinds.length, 8\nplaces = [{x: 1, y: 2}, {x: 1, y: 2}, {x: 2, y: 1}]\nspots = places.distinct()\nassert: spots, [{x: 1, y: 2}, {x: 2, y: 1}]\nplaces.push({y: 2, x: 1})\nplaces.push({x: 1, y: 2, z: 0})\nassert: spots.length, 3\npaths = [[1, 2], [2, 1], [1, 2]]\nroutes = paths.distinct()\nassert: routes, [[1, 2], [2, 1]]\npaths.push([1, [2]])\npaths.push([1, [2]])\nassert: routes, [[1, 2], [2, 1], [1, [2]]]\nclass Robot(name): this.name = name\nfirst = Robot(\"ada\")\nsecond = Robot(\"ada\")\ncrew = [first, first]\nteam = crew.distinct()\nassert: team.length, 1\ncrew.push(second)\nassert: team.length, 2");
    let mut run = runner();
    run(r#"inputs = [2, "2", true, 2, null, "2", null]"#);
    run("kinds = inputs.distinct()");
    assert_eq!(run("kinds"), serde_json::json!([2, "2", true, null]));
    run("inputs.push(1)");
    run("inputs.push(-0)");
    run("inputs.push(0)");
    assert_eq!(run("kinds"), serde_json::json!([2, "2", true, null, 1, 0]));
    assert_eq!(run("String(1 / kinds[5])"), "-Infinity");
    run("inputs.push(0 / 0)");
    run("inputs.push(0 / 0)");
    assert_eq!(run("kinds.length"), 8);
    run("places = [{x: 1, y: 2}, {x: 1, y: 2}, {x: 2, y: 1}]");
    run("spots = places.distinct()");
    {
        let actual = run("spots");
        let expected = run("([{x: 1, y: 2}, {x: 2, y: 1}])");
        assert_eq!(actual, expected);
    }
    run("places.push({y: 2, x: 1})");
    run("places.push({x: 1, y: 2, z: 0})");
    assert_eq!(run("spots.length"), 3);
    run("paths = [[1, 2], [2, 1], [1, 2]]");
    run("routes = paths.distinct()");
    assert_eq!(run("routes"), serde_json::json!([[1, 2], [2, 1]]));
    run("paths.push([1, [2]])");
    run("paths.push([1, [2]])");
    assert_eq!(run("routes"), serde_json::json!([[1, 2], [2, 1], [1, [2]]]));
    run("class Robot(name): this.name = name");
    run(r#"first = Robot("ada")"#);
    run(r#"second = Robot("ada")"#);
    run("crew = [first, first]");
    run("team = crew.distinct()");
    assert_eq!(run("team.length"), 1);
    run("crew.push(second)");
    assert_eq!(run("team.length"), 2);
}

/// Nucleoid calls the distinct selector once per item with only the item
#[rustfmt::skip]
#[test]
fn calls_the_distinct_selector_once_per_item_with_only_the_item() {
    runner()("seen = []\ndef zone(stop, position):\n    seen.push(position ?? stop.city)\n    return stop.zone\nstops = [{city: \"Oslo\", zone: 1}, {city: \"Rome\", zone: 3}, {city: \"Bergen\", zone: 1}]\ndef zones() => stops.distinct(zone)\nassert: zones(), [1, 3]\nassert: seen, [\"Oslo\", \"Rome\", \"Bergen\"]\nstops.push({city: \"Nice\", zone: 2})\nassert: zones(), [1, 3, 2]\nassert: seen, [\"Oslo\", \"Rome\", \"Bergen\", \"Oslo\", \"Rome\", \"Bergen\", \"Nice\"]\nlimit = 2\ndef band stop => stop.zone > limit ? \"far\" : \"near\"\nbands = stops.distinct(band)\nassert: bands, [\"near\", \"far\"]\nlimit = 0\nassert: bands, [\"far\"]\nmissing = null\nskipped = missing.distinct(zone(9))\nassert: skipped, null\nassert: seen, [\"Oslo\", \"Rome\", \"Bergen\", \"Oslo\", \"Rome\", \"Bergen\", \"Nice\"]");
    let mut run = runner();
    run("seen = []");
    run(r#"def zone(stop, position):
    seen.push(position ?? stop.city)
    return stop.zone"#);
    run(r#"stops = [{city: "Oslo", zone: 1}, {city: "Rome", zone: 3}, {city: "Bergen", zone: 1}]"#);
    run("def zones() => stops.distinct(zone)");
    assert_eq!(run("zones()"), serde_json::json!([1, 3]));
    assert_eq!(run("seen"), serde_json::json!(["Oslo", "Rome", "Bergen"]));
    run(r#"stops.push({city: "Nice", zone: 2})"#);
    assert_eq!(run("zones()"), serde_json::json!([1, 3, 2]));
    assert_eq!(run("seen"), serde_json::json!(["Oslo", "Rome", "Bergen", "Oslo", "Rome", "Bergen", "Nice"]));
    run("limit = 2");
    run(r#"def band stop => stop.zone > limit ? "far" : "near""#);
    run("bands = stops.distinct(band)");
    assert_eq!(run("bands"), serde_json::json!(["near", "far"]));
    run("limit = 0");
    assert_eq!(run("bands"), serde_json::json!(["far"]));
    run("missing = null");
    run("skipped = missing.distinct(zone(9))");
    assert_eq!(run("skipped"), serde_json::Value::Null);
    assert_eq!(run("seen"), serde_json::json!(["Oslo", "Rome", "Bergen", "Oslo", "Rome", "Bergen", "Nice"]));
}

/// Nucleoid keeps distinct values live in class level rules
#[rustfmt::skip]
#[test]
fn keeps_distinct_values_live_in_class_level_rules() {
    runner()("class Customer(name): this.name = name\nclass Order(customer, region, total):\n    this.customer = customer\n    this.region = region\n    this.total = total\n$Order.size = $Order.total > 100 ? \"large\" : \"small\"\n$Customer.regions = Order.filter(order -> order.customer == $Customer).distinct(order -> order.region)\n$Customer.sizes = Order.filter(order -> order.customer == $Customer).distinct(order -> order.size)\n$Customer.spread = $Customer.regions.length\nada = Customer(\"ada\")\nbo = Customer(\"bo\")\nOrder(ada, \"north\", 50)\nbig = Order(ada, \"south\", 150)\nOrder(bo, \"north\", 20)\nOrder(ada, \"north\", 80)\nsizes = Order.distinct(order -> order.size)\nassert: ada.regions, [\"north\", \"south\"]\nassert: ada.sizes, [\"small\", \"large\"]\nassert: bo.spread, 1\nassert: sizes, [\"small\", \"large\"]\nbig.total = 90\nassert: ada.sizes, [\"small\"]\nassert: sizes, [\"small\"]\ncy = Customer(\"cy\")\nassert: cy.regions, []\nassert: cy.spread, 0\nOrder(cy, \"east\", 300)\nOrder(cy, \"east\", 60)\nassert: cy.regions, [\"east\"]\nassert: sizes, [\"small\", \"large\"]");
    let mut run = runner();
    run("class Customer(name): this.name = name");
    run(r#"class Order(customer, region, total):
    this.customer = customer
    this.region = region
    this.total = total"#);
    run(r#"$Order.size = $Order.total > 100 ? "large" : "small""#);
    run("$Customer.regions = Order.filter(order -> order.customer == $Customer).distinct(order -> order.region)");
    run("$Customer.sizes = Order.filter(order -> order.customer == $Customer).distinct(order -> order.size)");
    run("$Customer.spread = $Customer.regions.length");
    run(r#"ada = Customer("ada")"#);
    run(r#"bo = Customer("bo")"#);
    run(r#"Order(ada, "north", 50)"#);
    run(r#"big = Order(ada, "south", 150)"#);
    run(r#"Order(bo, "north", 20)"#);
    run(r#"Order(ada, "north", 80)"#);
    run("sizes = Order.distinct(order -> order.size)");
    assert_eq!(run("ada.regions"), serde_json::json!(["north", "south"]));
    assert_eq!(run("ada.sizes"), serde_json::json!(["small", "large"]));
    assert_eq!(run("bo.spread"), 1);
    assert_eq!(run("sizes"), serde_json::json!(["small", "large"]));
    run("big.total = 90");
    assert_eq!(run("ada.sizes"), serde_json::json!(["small"]));
    assert_eq!(run("sizes"), serde_json::json!(["small"]));
    run(r#"cy = Customer("cy")"#);
    assert_eq!(run("cy.regions"), serde_json::json!([]));
    assert_eq!(run("cy.spread"), 0);
    run(r#"Order(cy, "east", 300)"#);
    run(r#"Order(cy, "east", 60)"#);
    assert_eq!(run("cy.regions"), serde_json::json!(["east"]));
    assert_eq!(run("sizes"), serde_json::json!(["small", "large"]));
}

/// Nucleoid uses distinct values in functions methods and conditional expressions
#[rustfmt::skip]
#[test]
fn uses_distinct_values_in_functions_methods_and_conditional_expressions() {
    runner()("class Post(author, tags):\n    this.author = author\n    this.tags = tags\n    def topics(): return this.tags.distinct()\ndef sizes(post) => post.tags.distinct(tag -> tag.length)\nfirst = Post(\"ada\", [\"rust\", \"logic\", \"rust\"])\nsecond = Post(\"bo\", [\"logic\", \"go\"])\ninuse = Post.flatMap(post -> post.topics()).distinct()\nbroader = first.topics().length > second.topics().length ? first.author : second.author\nlengths = sizes(first)\nassert: inuse, [\"rust\", \"logic\", \"go\"]\nassert: broader, \"bo\"\nassert: lengths, [4, 5]\nsecond.tags.pop()\nfirst.tags.push(\"graphs\")\nPost(\"cy\", [\"go\", \"sql\"])\nassert: inuse, [\"rust\", \"logic\", \"graphs\", \"go\", \"sql\"]\nassert: broader, \"ada\"\nassert: lengths, [4, 5, 6]");
    let mut run = runner();
    run(r#"class Post(author, tags):
    this.author = author
    this.tags = tags
    def topics(): return this.tags.distinct()"#);
    run("def sizes(post) => post.tags.distinct(tag -> tag.length)");
    run(r#"first = Post("ada", ["rust", "logic", "rust"])"#);
    run(r#"second = Post("bo", ["logic", "go"])"#);
    run("inuse = Post.flatMap(post -> post.topics()).distinct()");
    run("broader = first.topics().length > second.topics().length ? first.author : second.author");
    run("lengths = sizes(first)");
    assert_eq!(run("inuse"), serde_json::json!(["rust", "logic", "go"]));
    assert_eq!(run("broader"), "bo");
    assert_eq!(run("lengths"), serde_json::json!([4, 5]));
    run("second.tags.pop()");
    run(r#"first.tags.push("graphs")"#);
    run(r#"Post("cy", ["go", "sql"])"#);
    assert_eq!(run("inuse"), serde_json::json!(["rust", "logic", "graphs", "go", "sql"]));
    assert_eq!(run("broader"), "ada");
    assert_eq!(run("lengths"), serde_json::json!([4, 5, 6]));
}

/// Nucleoid keeps distinct as an ordinary name beside the distinct query
#[rustfmt::skip]
#[test]
fn keeps_distinct_as_an_ordinary_name_beside_the_distinct_query() {
    runner()("distinct = 2\nreport = {distinct: 3}\ndef twice distinct => distinct * 2\nclass Shelf(items):\n    this.items = items\n    def distinct(): return this.items.distinct().length\nshelf = Shelf([\"pen\", \"ink\", \"pen\"])\nspare = Shelf([\"cap\", \"pen\"])\nkinds = shelf.items.distinct()\ncount = shelf.distinct() + distinct + report.distinct\nshelves = Shelf.distinct(each -> each.distinct())\nassert: kinds, [\"pen\", \"ink\"]\nassert: count, 7\nassert: shelves, [2]\nassert: twice(distinct), 4\nassert: shelf.items.distinct, null\nshelf.items.push(\"cap\")\ndistinct = 4\nShelf([\"ink\", \"ink\"])\nassert: kinds, [\"pen\", \"ink\", \"cap\"]\nassert: count, 10\nassert: shelves, [3, 2, 1]");
    let mut run = runner();
    run("distinct = 2");
    run("report = {distinct: 3}");
    run("def twice distinct => distinct * 2");
    run(r#"class Shelf(items):
    this.items = items
    def distinct(): return this.items.distinct().length"#);
    run(r#"shelf = Shelf(["pen", "ink", "pen"])"#);
    run(r#"spare = Shelf(["cap", "pen"])"#);
    run("kinds = shelf.items.distinct()");
    run("count = shelf.distinct() + distinct + report.distinct");
    run("shelves = Shelf.distinct(each -> each.distinct())");
    assert_eq!(run("kinds"), serde_json::json!(["pen", "ink"]));
    assert_eq!(run("count"), 7);
    assert_eq!(run("shelves"), serde_json::json!([2]));
    assert_eq!(run("twice(distinct)"), 4);
    assert_eq!(run("shelf.items.distinct"), serde_json::Value::Null);
    run(r#"shelf.items.push("cap")"#);
    run("distinct = 4");
    run(r#"Shelf(["ink", "ink"])"#);
    assert_eq!(run("kinds"), serde_json::json!(["pen", "ink", "cap"]));
    assert_eq!(run("count"), 10);
    assert_eq!(run("shelves"), serde_json::json!([3, 2, 1]));
}

/// Nucleoid rolls back failed distinct updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_distinct_updates() {
    runner()("def checked(zone):\n    if zone == \"void\":\n        throw \"BAD_ZONE\"\n    return zone\nzones = [\"a\", \"b\", \"a\"]\nseen = zones.distinct(checked)\nassert: seen, [\"a\", \"b\"]\nfailure = \"\"\ntry:\n    zones.shift()\n    zones.push(\"void\")\ncatch error:\n    failure = error\nassert: failure, \"BAD_ZONE\"\nassert: zones, [\"a\", \"b\", \"a\"]\nassert: seen, [\"a\", \"b\"]\nzones.push(\"c\")\nassert: seen, [\"a\", \"b\", \"c\"]\nclass Parcel(zone): this.zone = zone\nParcel(\"b\")\nParcel(\"a\")\nroutes = Parcel.distinct(parcel -> checked(parcel.zone))\nfailure = \"\"\ntry:\n    Parcel(\"void\")\ncatch error:\n    failure = error\nassert: failure, \"BAD_ZONE\"\nassert: Parcel.length, 2\nassert: routes, [\"b\", \"a\"]\nParcel(\"a\")\nParcel(\"c\")\nassert: routes, [\"b\", \"a\", \"c\"]");
    let mut run = runner();
    run(r#"def checked(zone):
    if zone == "void":
        throw "BAD_ZONE"
    return zone
zones = ["a", "b", "a"]
seen = zones.distinct(checked)
assert: seen, ["a", "b"]
failure = ""
try:
    zones.shift()
    zones.push("void")
catch error:
    failure = error
assert: failure, "BAD_ZONE"
assert: zones, ["a", "b", "a"]
assert: seen, ["a", "b"]
zones.push("c")
assert: seen, ["a", "b", "c"]
class Parcel(zone): this.zone = zone
Parcel("b")
Parcel("a")
routes = Parcel.distinct(parcel -> checked(parcel.zone))
failure = ""
try:
    Parcel("void")
catch error:
    failure = error
assert: failure, "BAD_ZONE"
assert: Parcel.length, 2
assert: routes, ["b", "a"]
Parcel("a")
Parcel("c")
assert: routes, ["b", "a", "c"]"#);
}

/// Nucleoid loops over object entries in insertion order
#[rustfmt::skip]
#[test]
fn loops_over_object_entries_in_insertion_order() {
    runner()("class Line(shelf, item, count):\n    this.shelf = shelf\n    this.item = item\n    this.count = count\nclass Shelf(name):\n    this.name = name\n    this.units = 0\n    def stock(items):\n        for item, count in items:\n            Line(this.name, item, count)\n            this.units += count ?? 0\n        return this.units\nnone = {}\ndry = {rice: 4, beans: 1, oil: null, tea: none.tea}\ndry.flour = 6\ndelete dry.beans\ndry.beans = 2\npantry = Shelf(\"pantry\")\nassert: [pantry.stock(dry), Shelf(\"fridge\").stock({milk: 2, eggs: 12}), Shelf(\"cellar\").stock({wine: 3, cider: 1})], [12, 14, 4]\nassert: Line.map(line -> line.item).join(\" \"), \"rice oil flour beans milk eggs wine cider\"\nassert: Line.filter(line -> line.count == null).map(line -> line.shelf + \":\" + line.item), [\"pantry:oil\"]");
    let mut run = runner();
    run(r#"class Line(shelf, item, count):
    this.shelf = shelf
    this.item = item
    this.count = count"#);
    run(r#"class Shelf(name):
    this.name = name
    this.units = 0
    def stock(items):
        for item, count in items:
            Line(this.name, item, count)
            this.units += count ?? 0
        return this.units"#);
    run("none = {}");
    run("dry = {rice: 4, beans: 1, oil: null, tea: none.tea}");
    run("dry.flour = 6");
    run("delete dry.beans");
    run("dry.beans = 2");
    run(r#"pantry = Shelf("pantry")"#);
    assert_eq!(run(r#"[pantry.stock(dry), Shelf("fridge").stock({milk: 2, eggs: 12}), Shelf("cellar").stock({wine: 3, cider: 1})]"#), serde_json::json!([12, 14, 4]));
    assert_eq!(run(r#"Line.map(line -> line.item).join(" ")"#), "rice oil flour beans milk eggs wine cider");
    assert_eq!(run(r#"Line.filter(line -> line.count == null).map(line -> line.shelf + ":" + line.item)"#), serde_json::json!(["pantry:oil"]));
}

/// Nucleoid loops over the entries an instance holds
#[rustfmt::skip]
#[test]
fn loops_over_the_entries_an_instance_holds() {
    runner()("class Room(name, width, depth):\n    this.name = name\n    this.width = width\n    this.depth = depth\n$Room.area = $Room.width * $Room.depth\nclass Lamp(watts): this.watts = watts\nclass Fact(subject, field, data):\n    this.subject = subject\n    this.field = field\n    this.data = data\ndef record(thing):\n    for field, value of thing: Fact(thing.id, field, value)\n    return Fact.length\nlab = Room(\"lab\", 3, 4)\nden = Room(\"den\", 2, 2)\ndesk = Lamp(60)\ndesk.on = true\nlab.width = 5\ndelete lab.name\nlab.name = \"studio\"\ntry:\n    delete den.width\n    den.width = 7\n    throw \"BUSY\"\ncatch error:\n    pass\nassert: [record(lab), record(den), record(desk)], [5, 10, 13]\nassert: Fact.filter(fact -> fact.subject == \"lab\").map(fact -> fact.field).join(\" \"), \"id width depth area name\"\nassert: Fact.filter(fact -> fact.subject == \"den\").map(fact -> fact.field + \"=\" + fact.data).join(\" \"), \"id=den name=den width=2 depth=2 area=4\"\nassert: Fact.filter(fact -> fact.data == 20 or fact.data == true).map(fact -> fact.subject + \".\" + fact.field), [\"lab.area\", \"desk.on\"]");
    let mut run = runner();
    run(r#"class Room(name, width, depth):
    this.name = name
    this.width = width
    this.depth = depth
$Room.area = $Room.width * $Room.depth
class Lamp(watts): this.watts = watts
class Fact(subject, field, data):
    this.subject = subject
    this.field = field
    this.data = data
def record(thing):
    for field, value of thing: Fact(thing.id, field, value)
    return Fact.length
lab = Room("lab", 3, 4)
den = Room("den", 2, 2)
desk = Lamp(60)
desk.on = true
lab.width = 5
delete lab.name
lab.name = "studio"
try:
    delete den.width
    den.width = 7
    throw "BUSY"
catch error:
    pass
assert: [record(lab), record(den), record(desk)], [5, 10, 13]
assert: Fact.filter(fact -> fact.subject == "lab").map(fact -> fact.field).join(" "), "id width depth area name"
assert: Fact.filter(fact -> fact.subject == "den").map(fact -> fact.field + "=" + fact.data).join(" "), "id=den name=den width=2 depth=2 area=4"
assert: Fact.filter(fact -> fact.data == 20 or fact.data == true).map(fact -> fact.subject + "." + fact.field), ["lab.area", "desk.on"]"#);
}

/// Nucleoid snapshots object entries before loop bodies
#[rustfmt::skip]
#[test]
fn snapshots_object_entries_before_loop_bodies() {
    runner()("bills = {rent: 900, food: 300, travel: 200, gym: 50}\nseen = []\nfor bill, amount of bills:\n    seen.push(bill + \":\" + amount + \":\" + (bills[bill] ?? \"gone\"))\n    bills.food = 0\n    delete bills.travel\n    bills.fee = (bills.fee ?? 0) + 5\nassert: seen, [\"rent:900:900\", \"food:300:0\", \"travel:200:gone\", \"gym:50:50\"]\nassert: [bills.food, bills.travel, bills.fee], [0, null, 20]\nowed = {total: 0}\nfor bill, amount of bills: owed.total += amount\nassert: owed.total, 970");
    let mut run = runner();
    run("bills = {rent: 900, food: 300, travel: 200, gym: 50}");
    run("seen = []");
    run(r#"for bill, amount of bills:
    seen.push(bill + ":" + amount + ":" + (bills[bill] ?? "gone"))
    bills.food = 0
    delete bills.travel
    bills.fee = (bills.fee ?? 0) + 5"#);
    assert_eq!(run("seen"), serde_json::json!(["rent:900:900", "food:300:0", "travel:200:gone", "gym:50:50"]));
    assert_eq!(run("[bills.food, bills.travel, bills.fee]"), serde_json::json!([0, null, 20]));
    run("owed = {total: 0}");
    run("for bill, amount of bills: owed.total += amount");
    assert_eq!(run("owed.total"), 970);
}

/// Nucleoid nests object entry loops over nested records
#[rustfmt::skip]
#[test]
fn nests_object_entry_loops_over_nested_records() {
    runner()("class Store(region, city, units):\n    this.region = region\n    this.city = city\n    this.units = units\nclass Region(name, stores):\n    this.name = name\n    this.stores = stores\nregions = {north: {oslo: 4, bergen: 2}, south: {rome: 5, bari: 1, napoli: 3}, east: {riga: 6, vilnius: 2}}\nfor region, cities of regions:\n    for city, units of cities: Store(region, city, units)\n    Region(region, Store.length)\nregions.south.bari = 7\nassert: Store.map(store -> store.city).join(\" \"), \"oslo bergen rome bari napoli riga vilnius\"\nassert: Region.map(each -> each.name + each.stores).join(\" \"), \"north2 south5 east7\"\nassert: [Store.filter(store -> store.region == \"south\").sum(store -> store.units), regions.south.bari], [9, 7]");
    let mut run = runner();
    run(r#"class Store(region, city, units):
    this.region = region
    this.city = city
    this.units = units"#);
    run(r#"class Region(name, stores):
    this.name = name
    this.stores = stores"#);
    run("regions = {north: {oslo: 4, bergen: 2}, south: {rome: 5, bari: 1, napoli: 3}, east: {riga: 6, vilnius: 2}}");
    run(r#"for region, cities of regions:
    for city, units of cities: Store(region, city, units)
    Region(region, Store.length)"#);
    run("regions.south.bari = 7");
    assert_eq!(run(r#"Store.map(store -> store.city).join(" ")"#), "oslo bergen rome bari napoli riga vilnius");
    assert_eq!(run(r#"Region.map(each -> each.name + each.stores).join(" ")"#), "north2 south5 east7");
    assert_eq!(run(r#"[Store.filter(store -> store.region == "south").sum(store -> store.units), regions.south.bari]"#), serde_json::json!([9, 7]));
}

/// Nucleoid returns from object entry loops in functions and methods
#[rustfmt::skip]
#[test]
fn returns_from_object_entry_loops_in_functions_and_methods() {
    runner()("class Pick(item, qty):\n    this.item = item\n    this.qty = qty\nclass Store(stock):\n    this.stock = stock\n    def pick(order):\n        for item, qty of order:\n            if (this.stock[item] ?? 0) < qty:\n                return item\n            Pick(item, qty)\n        return \"done\"\ndef load(truck, crates):\n    for crate, kg of crates:\n        if truck.kg + kg > truck.limit:\n            return\n        truck.kg += kg\nshop = Store({nuts: 5, bolts: 2, gears: 7})\nassert: shop.pick({nuts: 3, gears: 2}), \"done\"\nassert: shop.pick({gears: 1, bolts: 4, nuts: 1}), \"bolts\"\nassert: Pick.map(pick -> pick.item).join(\" \"), \"nuts gears gears\"\nvan = {kg: 0, limit: 10}\nload(van, {tools: 4, tiles: 9, paint: 2})\nassert: van.kg, 4\nload(van, {paint: 2, rope: 3})\nassert: van.kg, 9");
    let mut run = runner();
    run(r#"class Pick(item, qty):
    this.item = item
    this.qty = qty"#);
    run(r#"class Store(stock):
    this.stock = stock
    def pick(order):
        for item, qty of order:
            if (this.stock[item] ?? 0) < qty:
                return item
            Pick(item, qty)
        return "done""#);
    run(r#"def load(truck, crates):
    for crate, kg of crates:
        if truck.kg + kg > truck.limit:
            return
        truck.kg += kg"#);
    run("shop = Store({nuts: 5, bolts: 2, gears: 7})");
    assert_eq!(run("shop.pick({nuts: 3, gears: 2})"), "done");
    assert_eq!(run("shop.pick({gears: 1, bolts: 4, nuts: 1})"), "bolts");
    assert_eq!(run(r#"Pick.map(pick -> pick.item).join(" ")"#), "nuts gears gears");
    run("van = {kg: 0, limit: 10}");
    run("load(van, {tools: 4, tiles: 9, paint: 2})");
    assert_eq!(run("van.kg"), 4);
    run("load(van, {paint: 2, rope: 3})");
    assert_eq!(run("van.kg"), 9);
}

/// Nucleoid keeps object entry loops one shot
#[rustfmt::skip]
#[test]
fn keeps_object_entry_loops_one_shot() {
    runner()("class Lamp(room, watts):\n    this.room = room\n    this.watts = watts\ndef light(rooms):\n    for room, watts of rooms: Lamp(room, watts)\n    return Lamp.length\nload = Lamp.sum(lamp -> lamp.watts)\nlights = {hall: 40, desk: 60, porch: 25}\nnight = false\nif night: light(lights)\nassert: Lamp.length, 0\nnight = true\nlights.desk = 100\nlights.shed = 30\nassert: [Lamp.length, load], [3, 125]\nlights = {shed: 30, gate: 10, pool: 20, yard: 5}\nassert: [Lamp.length, load], [7, 190]\nbill = {watts: 0}\nfor room, watts of lights: bill.watts += watts\nlights.gate = 50\nassert: [Lamp.map(lamp -> lamp.room).join(\" \"), bill.watts], [\"hall desk porch shed gate pool yard\", 65]");
    let mut run = runner();
    run(r#"class Lamp(room, watts):
    this.room = room
    this.watts = watts"#);
    run(r#"def light(rooms):
    for room, watts of rooms: Lamp(room, watts)
    return Lamp.length"#);
    run("load = Lamp.sum(lamp -> lamp.watts)");
    run("lights = {hall: 40, desk: 60, porch: 25}");
    run("night = false");
    run("if night: light(lights)");
    assert_eq!(run("Lamp.length"), 0);
    run("night = true");
    run("lights.desk = 100");
    run("lights.shed = 30");
    assert_eq!(run("[Lamp.length, load]"), serde_json::json!([3, 125]));
    run("lights = {shed: 30, gate: 10, pool: 20, yard: 5}");
    assert_eq!(run("[Lamp.length, load]"), serde_json::json!([7, 190]));
    run("bill = {watts: 0}");
    run("for room, watts of lights: bill.watts += watts");
    run("lights.gate = 50");
    assert_eq!(run(r#"[Lamp.map(lamp -> lamp.room).join(" "), bill.watts]"#), serde_json::json!(["hall desk porch shed gate pool yard", 65]));
}

/// Nucleoid restores object entries at nested savepoints
#[rustfmt::skip]
#[test]
fn restores_object_entries_at_nested_savepoints() {
    runner()("class Seat(row, guest, price):\n    this.row = row\n    this.guest = guest\n    this.price = price\ndef seat(row, guests):\n    for guest, price of guests:\n        if price > 50: throw guest + \" is over budget\"\n        Seat(row, guest, price)\n    return Seat.length\nparty = {ann: 30, bob: 40, cy: 20}\ntry:\n    delete party.ann\n    try:\n        delete party.bob\n        party.bob = 60\n        seat(\"a\", party)\n    catch error:\n        assert: [error, Seat.length], [\"bob is over budget\", 0]\n    party.ann = 35\n    assert: seat(\"b\", party), 3\n    assert: Seat.map(each -> each.guest + each.price).join(\" \"), \"bob40 cy20 ann35\"\n    throw \"CANCELLED\"\ncatch error:\n    assert: [error, Seat.length], [\"CANCELLED\", 0]\nassert: seat(\"c\", party), 3\nassert: Seat.map(each -> each.row + each.guest + each.price).join(\" \"), \"cann30 cbob40 ccy20\"");
    let mut run = runner();
    run(r#"class Seat(row, guest, price):
    this.row = row
    this.guest = guest
    this.price = price
def seat(row, guests):
    for guest, price of guests:
        if price > 50: throw guest + " is over budget"
        Seat(row, guest, price)
    return Seat.length
party = {ann: 30, bob: 40, cy: 20}
try:
    delete party.ann
    try:
        delete party.bob
        party.bob = 60
        seat("a", party)
    catch error:
        assert: [error, Seat.length], ["bob is over budget", 0]
    party.ann = 35
    assert: seat("b", party), 3
    assert: Seat.map(each -> each.guest + each.price).join(" "), "bob40 cy20 ann35"
    throw "CANCELLED"
catch error:
    assert: [error, Seat.length], ["CANCELLED", 0]
assert: seat("c", party), 3
assert: Seat.map(each -> each.row + each.guest + each.price).join(" "), "cann30 cbob40 ccy20""#);
}

/// Nucleoid restores object entries in place when it rolls back
#[rustfmt::skip]
#[test]
fn restores_object_entries_in_place_when_it_rolls_back() {
    runner()("class Line(order, item, qty):\n    this.order = order\n    this.item = item\n    this.qty = qty\nclass Order(name):\n    this.name = name\n    this.items = []\n    def fill(stock):\n        for item, qty of stock:\n            if qty > 5: throw item + \" is short\"\n            Line(this.name, item, qty)\n            this.items.push(item + qty)\n        return this.items\nbasket = {tea: 4, jam: 3, oats: 2}\nfirst = Order(\"first\")\ntry:\n    basket.tea = 9\n    delete basket.jam\n    delete basket.tea\n    basket.tea = 6\n    first.fill(basket)\ncatch error:\n    assert: [error, first.items, Line.length], [\"tea is short\", [], 0]\nsecond = Order(\"second\")\nassert: second.fill(basket), [\"tea4\", \"jam3\", \"oats2\"]\nplan = suppose:\n    delete basket.jam\n    basket.rye = 5\n    Order(\"third\").fill(basket)\nassert: plan, [\"tea4\", \"oats2\", \"rye5\"]\nbasket.salt = 1\nfourth = Order(\"fourth\")\nassert: fourth.fill(basket), [\"tea4\", \"jam3\", \"oats2\", \"salt1\"]\nassert: Line.map(line -> line.order + \":\" + line.item).join(\" \"), \"second:tea second:jam second:oats fourth:tea fourth:jam fourth:oats fourth:salt\"");
    let mut run = runner();
    run(r#"class Line(order, item, qty):
    this.order = order
    this.item = item
    this.qty = qty
class Order(name):
    this.name = name
    this.items = []
    def fill(stock):
        for item, qty of stock:
            if qty > 5: throw item + " is short"
            Line(this.name, item, qty)
            this.items.push(item + qty)
        return this.items
basket = {tea: 4, jam: 3, oats: 2}
first = Order("first")
try:
    basket.tea = 9
    delete basket.jam
    delete basket.tea
    basket.tea = 6
    first.fill(basket)
catch error:
    assert: [error, first.items, Line.length], ["tea is short", [], 0]
second = Order("second")
assert: second.fill(basket), ["tea4", "jam3", "oats2"]
plan = suppose:
    delete basket.jam
    basket.rye = 5
    Order("third").fill(basket)
assert: plan, ["tea4", "oats2", "rye5"]
basket.salt = 1
fourth = Order("fourth")
assert: fourth.fill(basket), ["tea4", "jam3", "oats2", "salt1"]
assert: Line.map(line -> line.order + ":" + line.item).join(" "), "second:tea second:jam second:oats fourth:tea fourth:jam fourth:oats fourth:salt""#);
}

/// Nucleoid repeats a while loop until its condition fails
#[rustfmt::skip]
#[test]
fn repeats_a_while_loop_until_its_condition_fails() {
    let mut run = runner();
    run("level = 0");
    run("full = level >= 10");
    run("steps = 0");
    run(r#"while not full:
    level = level + 3
    steps = steps + 1"#);
    assert_eq!(run("level"), 12);
    assert_eq!(run("steps"), 4);
    assert_eq!(run("full"), true);
    run("level = 4");
    assert_eq!(run("full"), false);
    assert_eq!(run("steps"), 4);
}

/// Nucleoid skips a while loop whose condition starts falsy
#[rustfmt::skip]
#[test]
fn skips_a_while_loop_whose_condition_starts_falsy() {
    let mut run = runner();
    run(r#"class Counter(count):
    this.count = count
    this.ticks = 0"#);
    run("busy = Counter(3)");
    run("idle = Counter(0)");
    run(r#"blank = Counter("")"#);
    run("unset = Counter(null)");
    run(r#"for counter of Counter:
    while counter.count:
        counter.count = counter.count - 1
        counter.ticks = counter.ticks + 1"#);
    assert_eq!(run("busy.count"), 0);
    assert_eq!(run("busy.ticks"), 3);
    assert_eq!(run("idle.ticks"), 0);
    assert_eq!(run("blank.ticks"), 0);
    assert_eq!(run("unset.ticks"), 0);
    assert_eq!(run("blank.count"), "");
}

/// Nucleoid runs while loops in methods with parameters and return
#[rustfmt::skip]
#[test]
fn runs_while_loops_in_methods_with_parameters_and_return() {
    let mut run = runner();
    run(r#"class Tank(level):
    this.level = level
    this.cycles = 0
    def drain(rate):
        while this.level > 0:
            if this.level < rate:
                return "partial"
            this.level = this.level - rate
            this.cycles = this.cycles + 1
        return "empty""#);
    run("big = Tank(9)");
    run("small = Tank(7)");
    assert_eq!(run("big.drain(3)"), "empty");
    assert_eq!(run("big.level"), 0);
    assert_eq!(run("big.cycles"), 3);
    assert_eq!(run("small.drain(3)"), "partial");
    assert_eq!(run("small.level"), 1);
    assert_eq!(run("small.cycles"), 2);
    assert_eq!(run("Tank(0).drain(3)"), "empty");
}

/// Nucleoid nests while loops with conditionals and for loops
#[rustfmt::skip]
#[test]
fn nests_while_loops_with_conditionals_and_for_loops() {
    let mut run = runner();
    run(r#"class Shelf(stock):
    this.stock = stock"#);
    run("a = Shelf(0)");
    run("b = Shelf(3)");
    run("low = Shelf.filter(s => s.stock < 3).length");
    run("rounds = 0");
    run("moves = 0");
    run(r#"while low > 0:
    for s of Shelf:
        if s.stock < 3:
            while s.stock < 3:
                s.stock = s.stock + 2
                moves = moves + 1
        else:
            s.stock = s.stock - 1
    rounds = rounds + 1"#);
    assert_eq!(run("rounds"), 2);
    assert_eq!(run("moves"), 3);
    assert_eq!(run("a.stock"), 3);
    assert_eq!(run("b.stock"), 4);
    assert_eq!(run("low"), 0);
    run("a.stock = 0");
    assert_eq!(run("low"), 1);
    assert_eq!(run("moves"), 3);
}

/// Nucleoid creates instances in a while loop until a class rule holds
#[rustfmt::skip]
#[test]
fn creates_instances_in_a_while_loop_until_a_class_rule_holds() {
    let mut run = runner();
    run(r#"class Cell(size):
    this.size = size"#);
    run("$Cell.big = $Cell.size > 4");
    run("grown = Cell.filter(c => c.big).length");
    run(r#"while grown < 2:
    Cell(Cell.length * 2)"#);
    assert_eq!(run("Cell.length"), 5);
    assert_eq!(run("grown"), 2);
    assert_eq!(run("Cell.map(c => c.size)"), serde_json::json!([0, 2, 4, 6, 8]));
    assert_eq!(run("Cell.map(c => c.big)"), serde_json::json!([false, false, false, true, true]));
}

/// Nucleoid stops a runaway while loop and rolls it back
#[rustfmt::skip]
#[test]
fn stops_a_runaway_while_loop_and_rolls_it_back() {
    let mut run = runner();
    run(r#"level = 0
while level < 12:
    level = level + 3
assert(level, 12)
try:
    while level != 20: level = level + 3
catch error:
    failure = error
assert(failure, TypeError("Loop did not terminate"))
assert(level, 12)"#);
}

/// Nucleoid keeps while as an ordinary name beside while loops
#[rustfmt::skip]
#[test]
fn keeps_while_as_an_ordinary_name_beside_while_loops() {
    let mut run = runner();
    run("while = 3");
    run("count = 0");
    run(r#"while while > 0:
    while = while - 1
    count = count + 1"#);
    assert_eq!(run("while"), 0);
    assert_eq!(run("count"), 3);
    run("def twice(while) => while * 2");
    assert_eq!(run("twice(4)"), 8);
    run("point = {while: 1}");
    assert_eq!(run("point.while"), 1);
}

/// Nucleoid rolls back a while loop that fails during an iteration
#[rustfmt::skip]
#[test]
fn rolls_back_a_while_loop_that_fails_during_an_iteration() {
    let mut run = runner();
    run(r#"class Parcel
stock = 16
shipped = 0
left = stock - shipped
try:
    while shipped < 20:
        if left < 3:
            throw "SHORT"
        Parcel()
        shipped = shipped + 3
catch error:
    failure = error
assert(failure, "SHORT")
assert(shipped, 0)
assert(left, 16)
assert(Parcel.length, 0)
try:
    while missing > 0:
        shipped = 1
catch error:
    failure = error
assert(failure, ReferenceError("missing is not defined"))
assert(shipped, 0)"#);
}

/// Nucleoid destructures object fields into standing declarations
#[rustfmt::skip]
#[test]
fn destructures_object_fields_into_standing_declarations() {
    runner()("point = {x: 3, y: 4, z: 0}\n{x, y, z} = point\nassert: [x, y, z], [3, 4, 0]\npoint.x = 6\npoint.z = 2\nassert: [x, z], [6, 2]\npoint = {x: 1, y: 2}\nassert: [x, y, z], [1, 2, null]\nsize = {w: 5}\n{w, h,} = size\nassert: [w, h], [5, null]\nsize.h = 4\nsize.w = 3\nassert: [w, h], [3, 4]\nx + y + w * h\n# return: 15");
    let mut run = runner();
    run("point = {x: 3, y: 4, z: 0}");
    run("{x, y, z} = point");
    assert_eq!(run("[x, y, z]"), serde_json::json!([3, 4, 0]));
    run("point.x = 6");
    run("point.z = 2");
    assert_eq!(run("[x, z]"), serde_json::json!([6, 2]));
    run("point = {x: 1, y: 2}");
    assert_eq!(run("[x, y, z]"), serde_json::json!([1, 2, null]));
    run("size = {w: 5}");
    run("{w, h,} = size");
    assert_eq!(run("[w, h]"), serde_json::json!([5, null]));
    run("size.h = 4");
    run("size.w = 3");
    assert_eq!(run("[w, h]"), serde_json::json!([3, 4]));
    run(r#"x + y + w * h

# return: 15"#);
}

/// Nucleoid destructures renamed fields of instances
#[rustfmt::skip]
#[test]
fn destructures_renamed_fields_of_instances() {
    runner()("class Room(name, width, depth):\n    this.name = name\n    this.width = width\n    this.depth = depth\n$Room.area = $Room.width * $Room.depth\nhall = Room(\"hall\", 2, 5)\nden = Room(\"den\", 4, 4)\n{name, area, width} = hall\n{name: other area: larger, depth} = den\nassert: [name, area, width], [\"hall\", 10, 2]\nassert: [other, larger, depth], [\"den\", 16, 4]\nhall.depth = 9\nden.width = 5\nassert: [area, larger], [18, 20]\nhall.width = 3\nden.depth = 2\nhall.name = \"atrium\"\nassert: [name, width, depth, area, larger], [\"atrium\", 3, 2, 27, 10]\nRoom.map(room -> room.area)\n# return: [27,10]");
    let mut run = runner();
    run(r#"class Room(name, width, depth):
    this.name = name
    this.width = width
    this.depth = depth"#);
    run("$Room.area = $Room.width * $Room.depth");
    run(r#"hall = Room("hall", 2, 5)"#);
    run(r#"den = Room("den", 4, 4)"#);
    run("{name, area, width} = hall");
    run("{name: other area: larger, depth} = den");
    assert_eq!(run("[name, area, width]"), serde_json::json!(["hall", 10, 2]));
    assert_eq!(run("[other, larger, depth]"), serde_json::json!(["den", 16, 4]));
    run("hall.depth = 9");
    run("den.width = 5");
    assert_eq!(run("[area, larger]"), serde_json::json!([18, 20]));
    run("hall.width = 3");
    run("den.depth = 2");
    run(r#"hall.name = "atrium""#);
    assert_eq!(run("[name, width, depth, area, larger]"), serde_json::json!(["atrium", 3, 2, 27, 10]));
    run(r#"Room.map(room -> room.area)

# return: [27,10]"#);
}

/// Nucleoid unpacks lists and strings into comma separated names
#[rustfmt::skip]
#[test]
fn unpacks_lists_and_strings_into_comma_separated_names() {
    runner()("size = [4, 3]\nwidth, height = size\narea = width * height\nassert: area, 12\nsize = [5, 2]\nassert: [width, height, area], [5, 2, 10]\norigin = {at: [0, 3]}\nrow, column, = origin.at\norigin.at = [4, 1]\nassert: [row, column], [4, 1]\nword = \"ok\"\nfirst, = word\nword = \"no\"\nassert: first, \"n\"\ndims = [2, 7]\ndeep, wide = dims\ndims = [6]\nassert: [deep, wide], [6, null]\narea + row\n# return: 14");
    let mut run = runner();
    run("size = [4, 3]");
    run("width, height = size");
    run("area = width * height");
    assert_eq!(run("area"), 12);
    run("size = [5, 2]");
    assert_eq!(run("[width, height, area]"), serde_json::json!([5, 2, 10]));
    run("origin = {at: [0, 3]}");
    run("row, column, = origin.at");
    run("origin.at = [4, 1]");
    assert_eq!(run("[row, column]"), serde_json::json!([4, 1]));
    run(r#"word = "ok""#);
    run("first, = word");
    run(r#"word = "no""#);
    assert_eq!(run("first"), "n");
    run("dims = [2, 7]");
    run("deep, wide = dims");
    run("dims = [6]");
    assert_eq!(run("[deep, wide]"), serde_json::json!([6, null]));
    run(r#"area + row

# return: 14"#);
}

/// Nucleoid destructures inside functions blocks and loops
#[rustfmt::skip]
#[test]
fn destructures_inside_functions_blocks_and_loops() {
    runner()("class Leg(span):\n    this.span = span\nLeg([0, 4])\nLeg([4, 10])\ndef length(leg):\n    start, end = leg.span\n    return end - start\ncovered = 0\nfor leg of Leg:\n    covered = covered + length(leg)\nassert: [covered, start, end], [10, 4, 10]\npoint = {x: 1, y: 2, z: 3}\n{\n    {x, y, z} = point\n    norm = x * x + y * y + z * z\n}\nassert: [x, y, z, norm], [1, 2, 3, 14]\npoint.z = 5\nassert: [z, norm], [5, 30]");
    let mut run = runner();
    run(r#"class Leg(span):
    this.span = span"#);
    run("Leg([0, 4])");
    run("Leg([4, 10])");
    run(r#"def length(leg):
    start, end = leg.span
    return end - start"#);
    run("covered = 0");
    run(r#"for leg of Leg:
    covered = covered + length(leg)"#);
    assert_eq!(run("[covered, start, end]"), serde_json::json!([10, 4, 10]));
    run("point = {x: 1, y: 2, z: 3}");
    run(r#"{
    {x, y, z} = point
    norm = x * x + y * y + z * z
}"#);
    assert_eq!(run("[x, y, z, norm]"), serde_json::json!([1, 2, 3, 14]));
    run("point.z = 5");
    assert_eq!(run("[z, norm]"), serde_json::json!([5, 30]));
}

/// Nucleoid destructures in conditional branches and hypotheticals
#[rustfmt::skip]
#[test]
fn destructures_in_conditional_branches_and_hypotheticals() {
    runner()("pair = [3, 4]\nready = false\nif ready: low, high = pair\nready = true\nassert: [low, high], [3, 4]\npair = [5, 6]\nassert: [low, high], [5, 6]\nready = false\npair = [7, 8]\nassert: [low, high], [5, 6]\nspread = suppose ready = true: high - low\nassert: [spread, low, high], [1, 5, 6]\npoint = {x: 1, y: 2}\nsuppose point.y = 9:\n    {x, y} = point\n    assert: [x, y], [1, 9]\nassert: point.y, 2");
    let mut run = runner();
    run(r#"pair = [3, 4]
ready = false
if ready: low, high = pair
ready = true
assert: [low, high], [3, 4]
pair = [5, 6]
assert: [low, high], [5, 6]
ready = false
pair = [7, 8]
assert: [low, high], [5, 6]
spread = suppose ready = true: high - low
assert: [spread, low, high], [1, 5, 6]
point = {x: 1, y: 2}
suppose point.y = 9:
    {x, y} = point
    assert: [x, y], [1, 9]
assert: point.y, 2"#);
}

/// Nucleoid explains destructured names as their declarations
#[rustfmt::skip]
#[test]
fn explains_destructured_names_as_their_declarations() {
    runner()("point = {x: 3, y: 4}\npair = [5, 6]\n{x, y} = point\nfirst, second = pair\nassert: (affects point).map(step -> step.rule), [\"x = point.x\", \"y = point.y\"]\nassert: (affects pair).map(step -> step.rule), [\"first = pair[0]\", \"second = pair[1]\"]\nassert: (why second)[0].from, [\"pair\"]\nclass Point(x, y):\n    this.x = x\n    this.y = y\n{x, y} = Point(1, 2)\nassert: [x, y, Point.length], [1, 2, 2]\nassert: (why y)[0].rule, \"y = Point(1,2).y\"\nassert: (affects point).length, 0");
    let mut run = runner();
    run("point = {x: 3, y: 4}");
    run("pair = [5, 6]");
    run("{x, y} = point");
    run("first, second = pair");
    assert_eq!(run("(affects point).map(step -> step.rule)"), serde_json::json!(["x = point.x", "y = point.y"]));
    assert_eq!(run("(affects pair).map(step -> step.rule)"), serde_json::json!(["first = pair[0]", "second = pair[1]"]));
    assert_eq!(run("(why second)[0].from"), serde_json::json!(["pair"]));
    run(r#"class Point(x, y):
    this.x = x
    this.y = y"#);
    run("{x, y} = Point(1, 2)");
    assert_eq!(run("[x, y, Point.length]"), serde_json::json!([1, 2, 2]));
    assert_eq!(run("(why y)[0].rule"), "y = Point(1,2).y");
    assert_eq!(run("(affects point).length"), 0);
}

/// Nucleoid keeps block object and list readings beside destructuring
#[rustfmt::skip]
#[test]
fn keeps_block_object_and_list_readings_beside_destructuring() {
    runner()("point = {x: 1, y: 2}\n{x, y} = point\n{ total = x + y }\nbox = {x, y}\nrecord = {left: x right: y}\npair = [3, 4]\nlow, high = pair\nadd = lambda a, b: a + b\npositive = all v in pair: v > 0\nfailure = \"\"\ntry:\n    [a, b] = pair\ncatch error:\n    failure = error\nassert: failure, SyntaxError(\"Cannot assign to [a,b]\")\nassert: [total, box, record], [3, {x: 1, y: 2}, {left: 1, right: 2}]\nassert: [add(low, high), positive], [7, true]\npoint.x = 5\npair = [-1, 6]\nassert: [total, box.x, record.left, low, high, positive], [7, 5, 5, -1, 6, false]\n{ x }\n# return: 5");
    let mut run = runner();
    run(r#"point = {x: 1, y: 2}
{x, y} = point
{ total = x + y }
box = {x, y}
record = {left: x right: y}
pair = [3, 4]
low, high = pair
add = lambda a, b: a + b
positive = all v in pair: v > 0
failure = ""
try:
    [a, b] = pair
catch error:
    failure = error
assert: failure, SyntaxError("Cannot assign to [a,b]")
assert: [total, box, record], [3, {x: 1, y: 2}, {left: 1, right: 2}]
assert: [add(low, high), positive], [7, true]
point.x = 5
pair = [-1, 6]
assert: [total, box.x, record.left, low, high, positive], [7, 5, 5, -1, 6, false]
{ x }
# return: 5"#);
}

/// Nucleoid rolls back failed destructuring
#[rustfmt::skip]
#[test]
fn rolls_back_failed_destructuring() {
    runner()("limits = [4, 50]\nlow, high = limits\nif high - low > 100: throw \"WIDE\"\nwidened = \"\"\ntry: limits = [0, 500]\ncatch error: widened = error\nwide = [9, 300]\nmoved = \"\"\ntry: low, high = wide\ncatch error: moved = error\nassert: [widened, moved], [\"WIDE\", \"WIDE\"]\nassert: [low, high, limits], [4, 50, [4, 50]]\norder = {qty: 2, price: 5}\n{qty, price} = order\nif qty * price > 50: throw \"COSTLY\"\ncostly = \"\"\ntry: order.price = 40\ncatch error: costly = error\nassert: [costly, order.price, qty, price], [\"COSTLY\", 5, 2, 5]\norder.qty = 3\norder.price = 7\nlimits = [3, 60]\nassert: [low, high, qty, price], [3, 60, 3, 7]");
    let mut run = runner();
    run(r#"limits = [4, 50]
low, high = limits
if high - low > 100: throw "WIDE"
widened = ""
try: limits = [0, 500]
catch error: widened = error
wide = [9, 300]
moved = ""
try: low, high = wide
catch error: moved = error
assert: [widened, moved], ["WIDE", "WIDE"]
assert: [low, high, limits], [4, 50, [4, 50]]
order = {qty: 2, price: 5}
{qty, price} = order
if qty * price > 50: throw "COSTLY"
costly = ""
try: order.price = 40
catch error: costly = error
assert: [costly, order.price, qty, price], ["COSTLY", 5, 2, 5]
order.qty = 3
order.price = 7
limits = [3, 60]
assert: [low, high, qty, price], [3, 60, 3, 7]"#);
}

/// Nucleoid counts matching list items
#[rustfmt::skip]
#[test]
fn counts_matching_list_items() {
    runner()("readings = [2, 5, 8, 11]\nthreshold = 6\nhigh = readings.count(reading -> reading > threshold)\nassert: high, 2\nreadings.push(14)\nassert: high, 3\nthreshold = 10\nassert: high, 2\nreadings.shift()\nassert: high, 2\nreadings = []\nassert: high, 0\ncart = {items: [0, 1, 2, 0, 3]}\nnonzero = cart.items.count(item -> item != 0)\nassert: nonzero, 3\ncart.items.push(4)\nassert: nonzero, 4");
    let mut run = runner();
    run("readings = [2, 5, 8, 11]");
    run("threshold = 6");
    run("high = readings.count(reading -> reading > threshold)");
    assert_eq!(run("high"), 2);
    run("readings.push(14)");
    assert_eq!(run("high"), 3);
    run("threshold = 10");
    assert_eq!(run("high"), 2);
    run("readings.shift()");
    assert_eq!(run("high"), 2);
    run("readings = []");
    assert_eq!(run("high"), 0);
    run("cart = {items: [0, 1, 2, 0, 3]}");
    run("nonzero = cart.items.count(item -> item != 0)");
    assert_eq!(run("nonzero"), 3);
    run("cart.items.push(4)");
    assert_eq!(run("nonzero"), 4);
}

/// Nucleoid counts matching class instances
#[rustfmt::skip]
#[test]
fn counts_matching_class_instances() {
    runner()("class Ticket(open, priority):\n    this.open = open\n    this.priority = priority\npending = Ticket.count(ticket -> ticket.open)\nassert: pending, 0\nfirst = Ticket(true, 2)\nsecond = Ticket(false, 5)\nthird = Ticket(true, 9)\nimportant = Ticket.count(ticket -> ticket.open and ticket.priority >= 5)\nassert: pending, 2\nassert: important, 1\nsecond.open = true\nassert: pending, 3\nassert: important, 2\nthird.priority = 1\nassert: important, 1\ndelete first.open\ndelete first.priority\ndelete first\nassert: pending, 2\nthird.open = false\nassert: pending, 1\nassert: important, 1");
    let mut run = runner();
    run(r#"class Ticket(open, priority):
    this.open = open
    this.priority = priority"#);
    run("pending = Ticket.count(ticket -> ticket.open)");
    assert_eq!(run("pending"), 0);
    run("first = Ticket(true, 2)");
    run("second = Ticket(false, 5)");
    run("third = Ticket(true, 9)");
    run("important = Ticket.count(ticket -> ticket.open and ticket.priority >= 5)");
    assert_eq!(run("pending"), 2);
    assert_eq!(run("important"), 1);
    run("second.open = true");
    assert_eq!(run("pending"), 3);
    assert_eq!(run("important"), 2);
    run("third.priority = 1");
    assert_eq!(run("important"), 1);
    run("delete first.open");
    run("delete first.priority");
    run("delete first");
    assert_eq!(run("pending"), 2);
    run("third.open = false");
    assert_eq!(run("pending"), 1);
    assert_eq!(run("important"), 1);
}

/// Nucleoid keeps predicate counts live in class level rules
#[rustfmt::skip]
#[test]
fn keeps_predicate_counts_live_in_class_level_rules() {
    runner()("class Team(name): this.name = name\nclass Task(team, done):\n    this.team = team\n    this.done = done\n$Team.pending = Task.count(task -> task.team == $Team and not task.done)\nalpha = Team(\"alpha\")\nbeta = Team(\"beta\")\nassert: alpha.pending, 0\nassert: beta.pending, 0\nfirst = Task(alpha, false)\nsecond = Task(alpha, true)\nthird = Task(beta, false)\nassert: alpha.pending, 1\nassert: beta.pending, 1\nfirst.team = beta\nassert: alpha.pending, 0\nassert: beta.pending, 2\nsecond.done = false\nassert: alpha.pending, 1\ndelete third.team\ndelete third.done\ndelete third\nassert: beta.pending, 1");
    let mut run = runner();
    run("class Team(name): this.name = name");
    run(r#"class Task(team, done):
    this.team = team
    this.done = done"#);
    run("$Team.pending = Task.count(task -> task.team == $Team and not task.done)");
    run(r#"alpha = Team("alpha")"#);
    run(r#"beta = Team("beta")"#);
    assert_eq!(run("alpha.pending"), 0);
    assert_eq!(run("beta.pending"), 0);
    run("first = Task(alpha, false)");
    run("second = Task(alpha, true)");
    run("third = Task(beta, false)");
    assert_eq!(run("alpha.pending"), 1);
    assert_eq!(run("beta.pending"), 1);
    run("first.team = beta");
    assert_eq!(run("alpha.pending"), 0);
    assert_eq!(run("beta.pending"), 2);
    run("second.done = false");
    assert_eq!(run("alpha.pending"), 1);
    run("delete third.team");
    run("delete third.done");
    run("delete third");
    assert_eq!(run("beta.pending"), 1);
}

/// Nucleoid uses predicate counts beside ordinary count names
#[rustfmt::skip]
#[test]
fn uses_predicate_counts_beside_ordinary_count_names() {
    runner()("cutoff = 3\ncount = 7\nreport = {count: 9}\nclass Bag(items):\n    this.items = items\n    def count(): return this.items.count(item -> item > cutoff)\nbag = Bag([1, 4, 8])\ndef matching(items) => items.count(item -> item > cutoff)\nassert: bag.count(), 2\nassert: matching([2, 5, 9]), 2\nassert: Bag.filter(each -> each.count() > 1).length, 1\nkind = Bag\nassert: kind.filter(each -> each.count() > 1).length, 1\nassert: [count, report.count], [7, 9]\ncutoff = 6\nassert: bag.count(), 1\nassert: matching([2, 5, 9]), 1\nassert: Bag.filter(each -> each.count() > 1).length, 0");
    let mut run = runner();
    run("cutoff = 3");
    run("count = 7");
    run("report = {count: 9}");
    run(r#"class Bag(items):
    this.items = items
    def count(): return this.items.count(item -> item > cutoff)"#);
    run("bag = Bag([1, 4, 8])");
    run("def matching(items) => items.count(item -> item > cutoff)");
    assert_eq!(run("bag.count()"), 2);
    assert_eq!(run("matching([2, 5, 9])"), 2);
    assert_eq!(run("Bag.filter(each -> each.count() > 1).length"), 1);
    run("kind = Bag");
    assert_eq!(run("kind.filter(each -> each.count() > 1).length"), 1);
    assert_eq!(run("[count, report.count]"), serde_json::json!([7, 9]));
    run("cutoff = 6");
    assert_eq!(run("bag.count()"), 1);
    assert_eq!(run("matching([2, 5, 9])"), 1);
    assert_eq!(run("Bag.filter(each -> each.count() > 1).length"), 0);
}

/// Nucleoid collects surplus positional arguments in fresh rest lists
#[rustfmt::skip]
#[test]
fn collects_surplus_positional_arguments_in_fresh_rest_lists() {
    let mut run = runner();
    run(r#"def collect(head = 1, ...tail) => [head, tail]
empty = collect()
one = collect(0)
many = collect(2, 3, 4)
nested = collect(false, "", [5, 6])
assert(empty, [1, []])
assert(one, [0, []])
assert(many, [2, [3, 4]])
assert(nested, [false, ["", [5, 6]]])
assert(collect(null).value, [null, []])
many
# return: [2,[3,4]]"#);
}

/// Nucleoid shares rest binding across anonymous functions callbacks and methods
#[rustfmt::skip]
#[test]
fn shares_rest_binding_across_anonymous_functions_callbacks_and_methods() {
    let mut run = runner();
    run("arrow = (...items) -> items");
    run("blocked = function(head, ...tail) { return [head, tail] }");
    run("def collect(...items) => items");
    run("alias = collect");
    run("record = {call: collect}");
    run("callbacks = [collect]");
    assert_eq!(run("arrow(1, 2)"), serde_json::json!([1, 2]));
    assert_eq!(run("blocked(1, 2, 3)"), serde_json::json!([1, [2, 3]]));
    assert_eq!(run("alias()"), serde_json::json!([]));
    assert_eq!(run("record.call(3, 4)"), serde_json::json!([3, 4]));
    assert_eq!(run("callbacks[0](5)"), serde_json::json!([5]));
    assert_eq!(run("[2, 3].map((item, ...extras) => [item, extras])"), serde_json::json!([[2, []], [3, []]]));
    run(r#"class Meter(seed):
    this.seed = seed
    def read(...items) => [self.seed, items]"#);
    run("meter = Meter(7)");
    assert_eq!(run("meter.read(8, 9)"), serde_json::json!([7, [8, 9]]));
    assert_eq!(run("meter.read()"), serde_json::json!([7, []]));
}

/// Nucleoid keeps fixed constructor labels separate from positional rest values
#[rustfmt::skip]
#[test]
fn keeps_fixed_constructor_labels_separate_from_positional_rest_values() {
    let mut run = runner();
    run(r#"class Packet(head = 1, ...tail):
    this.head = head
    this.tail = tail
empty = Packet()
many = Packet(2, 3, 4)
named = Packet(head=5)
nested = Packet(6, [7, 8])
assert(empty.tail, [])
assert(many.tail, [3, 4])
assert(named.head, 5)
assert(named.tail, [])
assert(nested.tail, [[7, 8]])
class Pair(first = 1, second = 2, ...tail):
    this.values = [first, second, tail]
mixed = Pair(4, second=7)
assert(mixed.values, [4, 7, []])
try:
    rejected = Packet(tail=[9])
catch error:
    assert(error, TypeError("Rest parameter 'tail' cannot be a named constructor argument for Packet"))
class Base:
    def init(...items):
        this.items = items
class Inherited: Base
    pass
class Child: Base
    def init(...items):
        super(1, 2)
        this.own = items
inherited = Inherited(3, 4)
child = Child(5, 6)
assert(inherited.items, [3, 4])
assert(child.items, [1, 2])
assert(child.own, [5, 6])"#);
}

/// Nucleoid initializes rest histories without capture or shared list mutation
#[rustfmt::skip]
#[test]
fn initializes_rest_histories_without_capture_or_shared_list_mutation() {
    let mut run = runner();
    run(r#"tail = [100]
def history(...tail):
    tail.push(9)
    return [tail@0, tail@last]
one = history(1, 2)
two = history()
three = history()
assert(one, [[1, 2], [1, 2, 9]])
assert(two, [[], [9]])
assert(three, [[], [9]])
assert(tail, [100])
def factory(...tail) => () => tail
callback = factory(7, 8)
assert(callback(), [100])
state = 1
def attempt(...tail):
    try:
        tail.push(8)
        state = 2
        throw "LOCAL_REST"
    catch error:
        state = state
    return [tail@0, tail, state]
assert(attempt(1), [[1], [1, 8], 1])
assert(state, 1)"#);
}

/// Nucleoid restores variadic definitions and their reactive callers on rollback
#[rustfmt::skip]
#[test]
fn restores_variadic_definitions_and_their_reactive_callers_on_rollback() {
    let mut run = runner();
    run(r#"unit = 2
def total(...items) => items.sum() * unit
live = total
frozen = total.value
observed = live(1, 2)
assert(observed, 6)
def total(head = 10, ...tail) => (head + tail.length) * unit
assert(observed, 4)
assert(frozen(1, 2), 6)
try:
    def total(...items) => 99
    assert(observed, 99)
    throw "RESTORE_REST"
catch error:
    assert(error, "RESTORE_REST")
assert(observed, 4)
unit = 3
assert(observed, 6)
assert(frozen(1, 2), 9)
observed
# return: 6"#);
}

/// Nucleoid finds the most frequent list item with mode
#[rustfmt::skip]
#[test]
fn finds_the_most_frequent_list_item_with_mode() {
    runner()("sizes = [\"M\", \"S\", \"M\", \"L\"]\ntop = sizes.mode()\nassert: top, \"M\"\nsizes.push(\"L\")\nsizes.push(\"L\")\nassert: top, \"L\"\nsizes.shift()\nassert: top, \"L\"\nsizes = []\nassert: top, null\nsizes = [3, 4, 4, 3]\nassert: top, 3\ncart = {lines: [{sku: \"pen\", qty: 2}, {sku: \"ink\", qty: 1}, {sku: \"ink\", qty: 5}]}\ncommon = cart.lines.mode(line -> line.sku)\nbulk = cart.lines.filter(line -> line.qty > 1).mode(line -> line.sku)\nassert: common, \"ink\"\nassert: bulk, \"pen\"\ncart.lines.push({sku: \"ink\", qty: 4})\nassert: bulk, \"ink\"\ncart.lines.push({sku: \"pen\", qty: 6})\ncart.lines.push({sku: \"pen\", qty: 7})\nassert: common, \"pen\"\nassert: bulk, \"pen\"");
    let mut run = runner();
    run(r#"sizes = ["M", "S", "M", "L"]"#);
    run("top = sizes.mode()");
    assert_eq!(run("top"), "M");
    run(r#"sizes.push("L")"#);
    run(r#"sizes.push("L")"#);
    assert_eq!(run("top"), "L");
    run("sizes.shift()");
    assert_eq!(run("top"), "L");
    run("sizes = []");
    assert_eq!(run("top"), serde_json::Value::Null);
    run("sizes = [3, 4, 4, 3]");
    assert_eq!(run("top"), 3);
    run(r#"cart = {lines: [{sku: "pen", qty: 2}, {sku: "ink", qty: 1}, {sku: "ink", qty: 5}]}"#);
    run("common = cart.lines.mode(line -> line.sku)");
    run("bulk = cart.lines.filter(line -> line.qty > 1).mode(line -> line.sku)");
    assert_eq!(run("common"), "ink");
    assert_eq!(run("bulk"), "pen");
    run(r#"cart.lines.push({sku: "ink", qty: 4})"#);
    assert_eq!(run("bulk"), "ink");
    run(r#"cart.lines.push({sku: "pen", qty: 6})"#);
    run(r#"cart.lines.push({sku: "pen", qty: 7})"#);
    assert_eq!(run("common"), "pen");
    assert_eq!(run("bulk"), "pen");
}

/// Nucleoid finds the most frequent value of class instance populations
#[rustfmt::skip]
#[test]
fn finds_the_most_frequent_value_of_class_instance_populations() {
    runner()("class Vote(choice, round):\n    this.choice = choice\n    this.round = round\nfavorite = Vote.mode(vote -> vote.choice)\nassert: favorite, null\nfirst = Vote(\"tea\", 1)\nVote(\"coffee\", 1)\nsecond = Vote(\"coffee\", 2)\nlate = Vote.filter(vote -> vote.round > 1).mode(vote -> vote.choice)\nrecent = Vote[-2:].mode(vote -> vote.choice)\nassert: favorite, \"coffee\"\nassert: late, \"coffee\"\nassert: recent, \"coffee\"\nsecond.choice = \"tea\"\nassert: favorite, \"tea\"\nassert: late, \"tea\"\nassert: recent, \"coffee\"\nVote(\"juice\", 2)\nVote(\"juice\", 3)\nassert: favorite, \"tea\"\nassert: late, \"juice\"\nassert: recent, \"juice\"\ndelete second.choice\ndelete second.round\ndelete second\nassert: favorite, \"juice\"\nkind = Vote\nassert: kind.mode(vote -> vote.round), 1");
    let mut run = runner();
    run(r#"class Vote(choice, round):
    this.choice = choice
    this.round = round"#);
    run("favorite = Vote.mode(vote -> vote.choice)");
    assert_eq!(run("favorite"), serde_json::Value::Null);
    run(r#"first = Vote("tea", 1)"#);
    run(r#"Vote("coffee", 1)"#);
    run(r#"second = Vote("coffee", 2)"#);
    run("late = Vote.filter(vote -> vote.round > 1).mode(vote -> vote.choice)");
    run("recent = Vote[-2:].mode(vote -> vote.choice)");
    assert_eq!(run("favorite"), "coffee");
    assert_eq!(run("late"), "coffee");
    assert_eq!(run("recent"), "coffee");
    run(r#"second.choice = "tea""#);
    assert_eq!(run("favorite"), "tea");
    assert_eq!(run("late"), "tea");
    assert_eq!(run("recent"), "coffee");
    run(r#"Vote("juice", 2)"#);
    run(r#"Vote("juice", 3)"#);
    assert_eq!(run("favorite"), "tea");
    assert_eq!(run("late"), "juice");
    assert_eq!(run("recent"), "juice");
    run("delete second.choice");
    run("delete second.round");
    run("delete second");
    assert_eq!(run("favorite"), "juice");
    run("kind = Vote");
    assert_eq!(run("kind.mode(vote -> vote.round)"), 1);
}

/// Nucleoid compares mode items with deep equality
#[rustfmt::skip]
#[test]
fn compares_mode_items_with_deep_equality() {
    runner()("inputs = [2, \"2\", true, \"2\", 2]\ncommon = inputs.mode()\nassert: common, 2\ninputs.push(\"2\")\nassert: common, \"2\"\ninputs.push(null)\ninputs.push(inputs[99])\ninputs.push(null)\ninputs.push(null)\nassert: common, null\nsigns = [-0, 1, 0, 1]\nzero = signs.mode()\nassert: String(1 / zero), \"-Infinity\"\nreadings = [0 / 0, 0 / 0, 5]\nlevel = readings.mode()\nassert: String(level), \"NaN\"\nreadings.push(5)\nassert: level, 5\nfirst = {x: 1, y: 2}\nplaces = [{x: 2, y: 1}, first, {y: 2, x: 1}]\nspot = places.mode()\nassert: spot == first, true\nheld = {k: null, m: 1}\ngap = {k: {}.none, m: 1, z: 5}\nrecords = [held, gap]\nkept = records.mode()\nassert: kept == held, true\nrecords = [gap, held]\nassert: kept == gap, true\nclass Robot(name): this.name = name\nada = Robot(\"ada\")\ntwin = Robot(\"ada\")\ncrew = [ada, twin, twin]\nlead = crew.mode()\nassert: lead == twin, true\ncrew.push(ada)\nassert: lead == ada, true");
    let mut run = runner();
    run(r#"inputs = [2, "2", true, "2", 2]"#);
    run("common = inputs.mode()");
    assert_eq!(run("common"), 2);
    run(r#"inputs.push("2")"#);
    assert_eq!(run("common"), "2");
    run("inputs.push(null)");
    run("inputs.push(inputs[99])");
    run("inputs.push(null)");
    run("inputs.push(null)");
    assert_eq!(run("common"), serde_json::Value::Null);
    run("signs = [-0, 1, 0, 1]");
    run("zero = signs.mode()");
    assert_eq!(run("String(1 / zero)"), "-Infinity");
    run("readings = [0 / 0, 0 / 0, 5]");
    run("level = readings.mode()");
    assert_eq!(run("String(level)"), "NaN");
    run("readings.push(5)");
    assert_eq!(run("level"), 5);
    run("first = {x: 1, y: 2}");
    run("places = [{x: 2, y: 1}, first, {y: 2, x: 1}]");
    run("spot = places.mode()");
    assert_eq!(run("spot == first"), true);
    run("held = {k: null, m: 1}");
    run("gap = {k: {}.none, m: 1, z: 5}");
    run("records = [held, gap]");
    run("kept = records.mode()");
    assert_eq!(run("kept == held"), true);
    run("records = [gap, held]");
    assert_eq!(run("kept == gap"), true);
    run("class Robot(name): this.name = name");
    run(r#"ada = Robot("ada")"#);
    run(r#"twin = Robot("ada")"#);
    run("crew = [ada, twin, twin]");
    run("lead = crew.mode()");
    assert_eq!(run("lead == twin"), true);
    run("crew.push(ada)");
    assert_eq!(run("lead == ada"), true);
}

/// Nucleoid calls the mode selector once per item with only the item
#[rustfmt::skip]
#[test]
fn calls_the_mode_selector_once_per_item_with_only_the_item() {
    runner()("seen = []\ndef size(order, position):\n    seen.push(position ?? order.id)\n    return order.size\norders = [{id: 1, size: \"M\"}, {id: 2, size: \"S\"}, {id: 3, size: \"S\"}]\ndef common() => orders.mode(size)\nassert: common(), \"S\"\nassert: seen, [1, 2, 3]\norders.push({id: 4, size: \"M\"})\nassert: common(), \"M\"\nassert: seen, [1, 2, 3, 1, 2, 3, 4]\nlimit = 2\ndef band order => order.id > limit ? \"late\" : \"early\"\nphase = orders.mode(band)\nassert: phase, \"early\"\nlimit = 0\nassert: phase, \"late\"\nmissing = null\nskipped = missing.mode(size(9))\nassert: skipped, null\nassert: seen, [1, 2, 3, 1, 2, 3, 4]");
    let mut run = runner();
    run("seen = []");
    run(r#"def size(order, position):
    seen.push(position ?? order.id)
    return order.size"#);
    run(r#"orders = [{id: 1, size: "M"}, {id: 2, size: "S"}, {id: 3, size: "S"}]"#);
    run("def common() => orders.mode(size)");
    assert_eq!(run("common()"), "S");
    assert_eq!(run("seen"), serde_json::json!([1, 2, 3]));
    run(r#"orders.push({id: 4, size: "M"})"#);
    assert_eq!(run("common()"), "M");
    assert_eq!(run("seen"), serde_json::json!([1, 2, 3, 1, 2, 3, 4]));
    run("limit = 2");
    run(r#"def band order => order.id > limit ? "late" : "early""#);
    run("phase = orders.mode(band)");
    assert_eq!(run("phase"), "early");
    run("limit = 0");
    assert_eq!(run("phase"), "late");
    run("missing = null");
    run("skipped = missing.mode(size(9))");
    assert_eq!(run("skipped"), serde_json::Value::Null);
    assert_eq!(run("seen"), serde_json::json!([1, 2, 3, 1, 2, 3, 4]));
}

/// Nucleoid keeps modes live in class level rules
#[rustfmt::skip]
#[test]
fn keeps_modes_live_in_class_level_rules() {
    runner()("class Team(name): this.name = name\nclass Vote(team, choice):\n    this.team = team\n    this.choice = choice\n$Team.favorite = Vote.filter(vote -> vote.team == $Team).mode(vote -> vote.choice)\nred = Team(\"red\")\nblue = Team(\"blue\")\nVote(red, \"tea\")\nswing = Vote(red, \"coffee\")\nVote(red, \"coffee\")\nVote(blue, \"juice\")\noverall = Vote.mode(vote -> vote.choice)\nassert: red.favorite, \"coffee\"\nassert: blue.favorite, \"juice\"\nassert: overall, \"coffee\"\nswing.choice = \"tea\"\nassert: red.favorite, \"tea\"\nassert: overall, \"tea\"\ngreen = Team(\"green\")\nassert: green.favorite, null\nVote(green, \"juice\")\nVote(green, \"juice\")\nassert: green.favorite, \"juice\"\nassert: overall, \"juice\"");
    let mut run = runner();
    run("class Team(name): this.name = name");
    run(r#"class Vote(team, choice):
    this.team = team
    this.choice = choice"#);
    run("$Team.favorite = Vote.filter(vote -> vote.team == $Team).mode(vote -> vote.choice)");
    run(r#"red = Team("red")"#);
    run(r#"blue = Team("blue")"#);
    run(r#"Vote(red, "tea")"#);
    run(r#"swing = Vote(red, "coffee")"#);
    run(r#"Vote(red, "coffee")"#);
    run(r#"Vote(blue, "juice")"#);
    run("overall = Vote.mode(vote -> vote.choice)");
    assert_eq!(run("red.favorite"), "coffee");
    assert_eq!(run("blue.favorite"), "juice");
    assert_eq!(run("overall"), "coffee");
    run(r#"swing.choice = "tea""#);
    assert_eq!(run("red.favorite"), "tea");
    assert_eq!(run("overall"), "tea");
    run(r#"green = Team("green")"#);
    assert_eq!(run("green.favorite"), serde_json::Value::Null);
    run(r#"Vote(green, "juice")"#);
    run(r#"Vote(green, "juice")"#);
    assert_eq!(run("green.favorite"), "juice");
    assert_eq!(run("overall"), "juice");
}

/// Nucleoid uses modes in functions methods and conditional expressions
#[rustfmt::skip]
#[test]
fn uses_modes_in_functions_methods_and_conditional_expressions() {
    runner()("class Shop(name, sales):\n    this.name = name\n    this.sales = sales\n    def bestseller(): return this.sales.mode()\ndef typical(shop) => shop.sales.mode(sale -> sale.length)\nnorth = Shop(\"north\", [\"tea\", \"cake\", \"tea\"])\nsouth = Shop(\"south\", [\"pie\", \"cake\", \"cake\"])\nhit = north.bestseller() == south.bestseller() ? \"shared\" : north.bestseller()\nsize = typical(north)\ntrend = Shop.mode(shop -> shop.bestseller())\nassert: hit, \"tea\"\nassert: size, 3\nassert: trend, \"tea\"\nnorth.sales.push(\"cake\")\nnorth.sales.push(\"cake\")\nassert: hit, \"shared\"\nassert: size, 4\nassert: trend, \"cake\"");
    let mut run = runner();
    run(r#"class Shop(name, sales):
    this.name = name
    this.sales = sales
    def bestseller(): return this.sales.mode()"#);
    run("def typical(shop) => shop.sales.mode(sale -> sale.length)");
    run(r#"north = Shop("north", ["tea", "cake", "tea"])"#);
    run(r#"south = Shop("south", ["pie", "cake", "cake"])"#);
    run(r#"hit = north.bestseller() == south.bestseller() ? "shared" : north.bestseller()"#);
    run("size = typical(north)");
    run("trend = Shop.mode(shop -> shop.bestseller())");
    assert_eq!(run("hit"), "tea");
    assert_eq!(run("size"), 3);
    assert_eq!(run("trend"), "tea");
    run(r#"north.sales.push("cake")"#);
    run(r#"north.sales.push("cake")"#);
    assert_eq!(run("hit"), "shared");
    assert_eq!(run("size"), 4);
    assert_eq!(run("trend"), "cake");
}

/// Nucleoid keeps mode as an ordinary name beside the mode query
#[rustfmt::skip]
#[test]
fn keeps_mode_as_an_ordinary_name_beside_the_mode_query() {
    runner()("mode = 2\nreport = {mode: 3}\ndef twice mode => mode * 2\nclass Shelf(items):\n    this.items = items\n    def mode(): return this.items.mode()\nshelf = Shelf([\"pen\", \"ink\", \"pen\"])\nspare = Shelf([\"cap\", \"cap\"])\ncommon = shelf.items.mode()\nlabel = shelf.mode() + \"/\" + spare.mode()\nshelves = Shelf.mode(each -> each.mode())\nassert: common, \"pen\"\nassert: label, \"pen/cap\"\nassert: shelves, \"pen\"\nassert: [mode, report.mode, twice(mode)], [2, 3, 4]\nassert: shelf.items.mode, null\nassert: Shelf.mode() == shelf, true\nspare.items.push(\"ink\")\nShelf([\"cap\"])\nassert: shelves, \"cap\"\nshelf.items.push(\"ink\")\nshelf.items.push(\"ink\")\nassert: common, \"ink\"\nassert: label, \"ink/cap\"");
    let mut run = runner();
    run("mode = 2");
    run("report = {mode: 3}");
    run("def twice mode => mode * 2");
    run(r#"class Shelf(items):
    this.items = items
    def mode(): return this.items.mode()"#);
    run(r#"shelf = Shelf(["pen", "ink", "pen"])"#);
    run(r#"spare = Shelf(["cap", "cap"])"#);
    run("common = shelf.items.mode()");
    run(r#"label = shelf.mode() + "/" + spare.mode()"#);
    run("shelves = Shelf.mode(each -> each.mode())");
    assert_eq!(run("common"), "pen");
    assert_eq!(run("label"), "pen/cap");
    assert_eq!(run("shelves"), "pen");
    assert_eq!(run("[mode, report.mode, twice(mode)]"), serde_json::json!([2, 3, 4]));
    assert_eq!(run("shelf.items.mode"), serde_json::Value::Null);
    assert_eq!(run("Shelf.mode() == shelf"), true);
    run(r#"spare.items.push("ink")"#);
    run(r#"Shelf(["cap"])"#);
    assert_eq!(run("shelves"), "cap");
    run(r#"shelf.items.push("ink")"#);
    run(r#"shelf.items.push("ink")"#);
    assert_eq!(run("common"), "ink");
    assert_eq!(run("label"), "ink/cap");
}

/// Nucleoid rolls back failed mode updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_mode_updates() {
    runner()("def checked(zone):\n    if zone == \"void\":\n        throw \"BAD_ZONE\"\n    return zone\nzones = [\"a\", \"b\", \"b\"]\nbusiest = zones.mode(checked)\nassert: busiest, \"b\"\nfailure = \"\"\ntry:\n    zones.shift()\n    zones.push(\"void\")\ncatch error:\n    failure = error\nassert: failure, \"BAD_ZONE\"\nassert: zones, [\"a\", \"b\", \"b\"]\nassert: busiest, \"b\"\nzones.push(\"a\")\nzones.push(\"a\")\nassert: busiest, \"a\"\nclass Parcel(zone): this.zone = zone\nParcel(\"b\")\nParcel(\"a\")\nParcel(\"a\")\nroutes = Parcel.mode(parcel -> checked(parcel.zone))\nfailure = \"\"\ntry:\n    Parcel(\"void\")\ncatch error:\n    failure = error\nassert: failure, \"BAD_ZONE\"\nassert: Parcel.length, 3\nassert: routes, \"a\"\nParcel(\"b\")\nParcel(\"b\")\nassert: routes, \"b\"");
    let mut run = runner();
    run(r#"def checked(zone):
    if zone == "void":
        throw "BAD_ZONE"
    return zone
zones = ["a", "b", "b"]
busiest = zones.mode(checked)
assert: busiest, "b"
failure = ""
try:
    zones.shift()
    zones.push("void")
catch error:
    failure = error
assert: failure, "BAD_ZONE"
assert: zones, ["a", "b", "b"]
assert: busiest, "b"
zones.push("a")
zones.push("a")
assert: busiest, "a"
class Parcel(zone): this.zone = zone
Parcel("b")
Parcel("a")
Parcel("a")
routes = Parcel.mode(parcel -> checked(parcel.zone))
failure = ""
try:
    Parcel("void")
catch error:
    failure = error
assert: failure, "BAD_ZONE"
assert: Parcel.length, 3
assert: routes, "a"
Parcel("b")
Parcel("b")
assert: routes, "b""#);
}

/// Nucleoid reads noun list queries with of
#[rustfmt::skip]
#[test]
fn reads_noun_list_queries_with_of() {
    runner()("prices = [4, 9, 2, 9]\ntotal = sum of prices\ntop = max of prices\nlow = min of prices\ncommon = mode of prices\nassert: [total, top, low, common], [24, 9, 2, 9]\nprices.push(2)\nprices.push(2)\nassert: [total, top, low, common], [28, 9, 2, 2]\nprices.shift()\nassert: [total, common], [24, 2]\nprices = [1, 3, 3]\nassert: [total, top, low, common], [7, 3, 1, 3]\nassert: sum of prices, prices.sum()\nassert: max of prices, prices.max()\nassert: min of prices, prices.min()\ncart = {lines: [{sku: \"pen\", qty: 2}, {sku: \"ink\", qty: 5}]}\nunits = sum of cart.lines.map(line -> line.qty)\nfavorite = mode of cart.lines.map(line -> line.sku)\nassert: units, cart.lines.sum(line -> line.qty)\nassert: favorite, \"pen\"\ncart.lines.push({sku: \"ink\", qty: 1})\nassert: [units, favorite], [8, \"ink\"]");
    let mut run = runner();
    run("prices = [4, 9, 2, 9]");
    run("total = sum of prices");
    run("top = max of prices");
    run("low = min of prices");
    run("common = mode of prices");
    assert_eq!(run("[total, top, low, common]"), serde_json::json!([24, 9, 2, 9]));
    run("prices.push(2)");
    run("prices.push(2)");
    assert_eq!(run("[total, top, low, common]"), serde_json::json!([28, 9, 2, 2]));
    run("prices.shift()");
    assert_eq!(run("[total, common]"), serde_json::json!([24, 2]));
    run("prices = [1, 3, 3]");
    assert_eq!(run("[total, top, low, common]"), serde_json::json!([7, 3, 1, 3]));
    {
        let actual = run("sum of prices");
        let expected = run("(prices.sum())");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("max of prices");
        let expected = run("(prices.max())");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("min of prices");
        let expected = run("(prices.min())");
        assert_eq!(actual, expected);
    }
    run(r#"cart = {lines: [{sku: "pen", qty: 2}, {sku: "ink", qty: 5}]}"#);
    run("units = sum of cart.lines.map(line -> line.qty)");
    run("favorite = mode of cart.lines.map(line -> line.sku)");
    {
        let actual = run("units");
        let expected = run("(cart.lines.sum(line -> line.qty))");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("favorite"), "pen");
    run(r#"cart.lines.push({sku: "ink", qty: 1})"#);
    assert_eq!(run("[units, favorite]"), serde_json::json!([8, "ink"]));
}

/// Nucleoid reads noun queries of class instance populations with of
#[rustfmt::skip]
#[test]
fn reads_noun_queries_of_class_instance_populations_with_of() {
    runner()("class Ticket(queue, minutes):\n    this.queue = queue\n    this.minutes = minutes\nbusiest = mode of Ticket.map(ticket -> ticket.queue)\nlongest = max of Ticket.map(ticket -> ticket.minutes)\nassert: busiest, null\nassert: longest, Number.NEGATIVE_INFINITY\nfirst = Ticket(\"billing\", 30)\nTicket(\"support\", 5)\nsecond = Ticket(\"support\", 12)\nassert: [busiest, longest], [\"support\", 30]\nrecent = min of Ticket[1:].map(ticket -> ticket.minutes)\nkind = Ticket\nworkload = sum of kind.map(ticket -> ticket.minutes)\nlisted = sum of Ticket.list().map(ticket -> ticket.minutes)\nassert: [recent, workload, listed], [5, 47, 47]\nfirst.minutes = 8\nsecond.queue = \"billing\"\nassert: [busiest, longest, workload], [\"billing\", 12, 25]\nassert: mode of Ticket, first\ndelete second.queue\ndelete second.minutes\ndelete second\nassert: [busiest, longest, recent, workload], [\"billing\", 8, 5, 13]");
    let mut run = runner();
    run(r#"class Ticket(queue, minutes):
    this.queue = queue
    this.minutes = minutes"#);
    run("busiest = mode of Ticket.map(ticket -> ticket.queue)");
    run("longest = max of Ticket.map(ticket -> ticket.minutes)");
    assert_eq!(run("busiest"), serde_json::Value::Null);
    {
        let actual = run("longest");
        let expected = run("(Number.NEGATIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run(r#"first = Ticket("billing", 30)"#);
    run(r#"Ticket("support", 5)"#);
    run(r#"second = Ticket("support", 12)"#);
    assert_eq!(run("[busiest, longest]"), serde_json::json!(["support", 30]));
    run("recent = min of Ticket[1:].map(ticket -> ticket.minutes)");
    run("kind = Ticket");
    run("workload = sum of kind.map(ticket -> ticket.minutes)");
    run("listed = sum of Ticket.list().map(ticket -> ticket.minutes)");
    assert_eq!(run("[recent, workload, listed]"), serde_json::json!([5, 47, 47]));
    run("first.minutes = 8");
    run(r#"second.queue = "billing""#);
    assert_eq!(run("[busiest, longest, workload]"), serde_json::json!(["billing", 12, 25]));
    {
        let actual = run("mode of Ticket");
        let expected = run("(first)");
        assert_eq!(actual, expected);
    }
    run("delete second.queue");
    run("delete second.minutes");
    run("delete second");
    assert_eq!(run("[busiest, longest, recent, workload]"), serde_json::json!(["billing", 8, 5, 13]));
}

/// Nucleoid keeps noun queries live in class level rules
#[rustfmt::skip]
#[test]
fn keeps_noun_queries_live_in_class_level_rules() {
    runner()("class Obj(parts): this.parts = parts\n$Obj.fill_color = mode of $Obj.parts\n$Obj.brightest = max of $Obj.parts\nfirst = Obj([3, 1, 3])\nsecond = Obj([2, 2, 5])\nassert: [first.fill_color, first.brightest], [3, 3]\nassert: [second.fill_color, second.brightest], [2, 5]\nfirst.parts.push(1)\nfirst.parts.push(1)\nsecond.parts = [7]\nassert: [first.fill_color, second.fill_color, second.brightest], [1, 7, 7]\nclass Team(name): this.name = name\nclass Job(team, hours):\n    this.team = team\n    this.hours = hours\n$Team.load = sum of Job.filter(job -> job.team == $Team).map(job -> job.hours)\nred = Team(\"red\")\nblue = Team(\"blue\")\nJob(red, 3)\nlong = Job(blue, 8)\nJob(red, 2)\nassert: [red.load, blue.load], [5, 8]\nlong.team = red\nassert: [red.load, blue.load], [13, 0]\nlong.hours = 1\nassert: red.load, 6");
    let mut run = runner();
    run("class Obj(parts): this.parts = parts");
    run("$Obj.fill_color = mode of $Obj.parts");
    run("$Obj.brightest = max of $Obj.parts");
    run("first = Obj([3, 1, 3])");
    run("second = Obj([2, 2, 5])");
    assert_eq!(run("[first.fill_color, first.brightest]"), serde_json::json!([3, 3]));
    assert_eq!(run("[second.fill_color, second.brightest]"), serde_json::json!([2, 5]));
    run("first.parts.push(1)");
    run("first.parts.push(1)");
    run("second.parts = [7]");
    assert_eq!(run("[first.fill_color, second.fill_color, second.brightest]"), serde_json::json!([1, 7, 7]));
    run("class Team(name): this.name = name");
    run(r#"class Job(team, hours):
    this.team = team
    this.hours = hours"#);
    run("$Team.load = sum of Job.filter(job -> job.team == $Team).map(job -> job.hours)");
    run(r#"red = Team("red")"#);
    run(r#"blue = Team("blue")"#);
    run("Job(red, 3)");
    run("long = Job(blue, 8)");
    run("Job(red, 2)");
    assert_eq!(run("[red.load, blue.load]"), serde_json::json!([5, 8]));
    run("long.team = red");
    assert_eq!(run("[red.load, blue.load]"), serde_json::json!([13, 0]));
    run("long.hours = 1");
    assert_eq!(run("red.load"), 6);
}

/// Nucleoid binds noun queries as prefix operators
#[rustfmt::skip]
#[test]
fn binds_noun_queries_as_prefix_operators() {
    runner()("xs = [1, 2, 3]\nassert: sum of xs + 1, 7\nassert: 1 + sum of xs * 2, 13\nassert: sum of xs.map(x -> x * 10), 60\nassert: (sum of xs) ** 2, 36\nassert: 2 ** max of xs, 8\nassert: -min of xs, -1\nassert: typeof mode of xs, \"number\"\nassert: max of xs > min of xs, true\nrows = [[1, 2], [1, 2], [5]]\nassert: sum of mode of rows, 3\nassert: max of rows.map(row -> sum of row), 5\nmissing = null\nassert: sum of missing ?? 0, 0\nready = false\nassert: sum of xs if ready else max of xs, 3\nfailure = \"\"\ntry:\n    sum of xs ** 2\ncatch error:\n    failure = error\nassert: failure, TypeError(\"number.sum is not a function\")");
    let mut run = runner();
    run(r#"xs = [1, 2, 3]
assert: sum of xs + 1, 7
assert: 1 + sum of xs * 2, 13
assert: sum of xs.map(x -> x * 10), 60
assert: (sum of xs) ** 2, 36
assert: 2 ** max of xs, 8
assert: -min of xs, -1
assert: typeof mode of xs, "number"
assert: max of xs > min of xs, true
rows = [[1, 2], [1, 2], [5]]
assert: sum of mode of rows, 3
assert: max of rows.map(row -> sum of row), 5
missing = null
assert: sum of missing ?? 0, 0
ready = false
assert: sum of xs if ready else max of xs, 3
failure = ""
try:
    sum of xs ** 2
catch error:
    failure = error
assert: failure, TypeError("number.sum is not a function")"#);
}

/// Nucleoid keeps names and loop variables named like noun queries
#[rustfmt::skip]
#[test]
fn keeps_names_and_loop_variables_named_like_noun_queries() {
    runner()("def sum(values):\n    return 0\nmax = 9\nrecords = [{mode: 4}, {mode: 4}, {mode: 6}]\nassert: sum([1, 2]), 0\nassert: sum of [1, 2], 3\nassert: max, 9\nassert: max of [1, 2], 2\nassert: mode of records.map(record -> record.mode), 4\nclass Item(size): this.size = size\nItem(4)\nItem(6)\nsizes = []\nfor mode of Item:\n    sizes.push(mode.size)\nassert: sizes, [4, 6]\nfor min in Item: sizes.push(min.size * 10)\nassert: sizes, [4, 6, 40, 60]\ntotal = [1, 2, 3].reduce(lambda sum, x: sum + x, 0)\nassert: total, 6\nlabel = `${max of [1, 2]}`\ngrouped = `${(max of [1, 2])}`\nassert: [label, grouped], [\"9\", \"2\"]");
    let mut run = runner();
    run(r#"def sum(values):
    return 0"#);
    run("max = 9");
    run("records = [{mode: 4}, {mode: 4}, {mode: 6}]");
    assert_eq!(run("sum([1, 2])"), 0);
    assert_eq!(run("sum of [1, 2]"), 3);
    assert_eq!(run("max"), 9);
    assert_eq!(run("max of [1, 2]"), 2);
    assert_eq!(run("mode of records.map(record -> record.mode)"), 4);
    run("class Item(size): this.size = size");
    run("Item(4)");
    run("Item(6)");
    run("sizes = []");
    run(r#"for mode of Item:
    sizes.push(mode.size)"#);
    assert_eq!(run("sizes"), serde_json::json!([4, 6]));
    run("for min in Item: sizes.push(min.size * 10)");
    assert_eq!(run("sizes"), serde_json::json!([4, 6, 40, 60]));
    run("total = [1, 2, 3].reduce(lambda sum, x: sum + x, 0)");
    assert_eq!(run("total"), 6);
    run("label = `${max of [1, 2]}`");
    run("grouped = `${(max of [1, 2])}`");
    assert_eq!(run("[label, grouped]"), serde_json::json!(["9", "2"]));
}

/// Nucleoid reads noun queries in headers and nested expressions
#[rustfmt::skip]
#[test]
fn reads_noun_queries_in_headers_and_nested_expressions() {
    runner()("class Item(size): this.size = size\nsmall = Item(1)\nlarge = Item(2)\ngroups = [[small, large], [small, large], [large]]\nsizes = []\nfor item of mode of groups:\n    sizes.push(item.size)\nassert: sizes, [1, 2]\nqueue = [1]\nwhile sum of queue < 4:\n    queue.push(1)\nassert: queue.length, 4\nalert = false\nif max of queue > 0:\n    alert = true\nassert: alert, true\nassert: all group in groups: sum of group.map(item -> item.size) > 1, true\ndef spread(values) => max of values - min of values\nassert: spread([4, 9, 1]), 8\npick = values -> mode of values\nassert: pick([\"a\", \"b\", \"b\"]), \"b\"\nreport = {total: sum of queue, top: max of queue}\nassert: [report.total, report.top], [4, 1]\nassert: [max of queue, min of queue], [1, 1]\nassert: String(sum of queue), \"4\"\nready = true\nassert: ready ? sum of queue : 0, 4\nassert: [10, 20, 30][max of queue], 20");
    let mut run = runner();
    run("class Item(size): this.size = size");
    run("small = Item(1)");
    run("large = Item(2)");
    run("groups = [[small, large], [small, large], [large]]");
    run("sizes = []");
    run(r#"for item of mode of groups:
    sizes.push(item.size)"#);
    assert_eq!(run("sizes"), serde_json::json!([1, 2]));
    run("queue = [1]");
    run(r#"while sum of queue < 4:
    queue.push(1)"#);
    assert_eq!(run("queue.length"), 4);
    run("alert = false");
    run(r#"if max of queue > 0:
    alert = true"#);
    assert_eq!(run("alert"), true);
    assert_eq!(run("all group in groups: sum of group.map(item -> item.size) > 1"), true);
    run("def spread(values) => max of values - min of values");
    assert_eq!(run("spread([4, 9, 1])"), 8);
    run("pick = values -> mode of values");
    assert_eq!(run(r#"pick(["a", "b", "b"])"#), "b");
    run("report = {total: sum of queue, top: max of queue}");
    assert_eq!(run("[report.total, report.top]"), serde_json::json!([4, 1]));
    assert_eq!(run("[max of queue, min of queue]"), serde_json::json!([1, 1]));
    assert_eq!(run("String(sum of queue)"), "4");
    run("ready = true");
    assert_eq!(run("ready ? sum of queue : 0"), 4);
    assert_eq!(run("[10, 20, 30][max of queue]"), 20);
}

/// Nucleoid explains noun queries in their written form
#[rustfmt::skip]
#[test]
fn explains_noun_queries_in_their_written_form() {
    runner()("prices = [4, 9, 9]\ntop = max of prices\nwords = [\"ab\", \"c\", \"ab\"]\nwidth = (mode of words).length\nsquared = (sum of prices) ** 2\nassert: (why top)[0].rule, \"top = max of prices\"\nassert: (why top)[0].from, [\"prices\"]\nassert: (why width)[0].rule, \"width = (mode of words).length\"\nassert: (why squared)[0].rule, \"squared = (sum of prices)**2\"\nassert: [top, width, squared], [9, 2, 484]");
    let mut run = runner();
    run("prices = [4, 9, 9]");
    run("top = max of prices");
    run(r#"words = ["ab", "c", "ab"]"#);
    run("width = (mode of words).length");
    run("squared = (sum of prices) ** 2");
    assert_eq!(run("(why top)[0].rule"), "top = max of prices");
    assert_eq!(run("(why top)[0].from"), serde_json::json!(["prices"]));
    assert_eq!(run("(why width)[0].rule"), "width = (mode of words).length");
    assert_eq!(run("(why squared)[0].rule"), "squared = (sum of prices)**2");
    assert_eq!(run("[top, width, squared]"), serde_json::json!([9, 2, 484]));
}

/// Nucleoid rolls back failed noun query updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_noun_query_updates() {
    runner()("def checked(zone):\n    if zone == \"void\":\n        throw \"BAD_ZONE\"\n    return zone\nzones = [\"a\", \"b\", \"b\"]\nbusiest = mode of zones.map(checked)\nassert: busiest, \"b\"\nfailure = \"\"\ntry:\n    zones.shift()\n    zones.push(\"void\")\ncatch error:\n    failure = error\nassert: failure, \"BAD_ZONE\"\nassert: zones, [\"a\", \"b\", \"b\"]\nassert: busiest, \"b\"\nzones.push(\"a\")\nzones.push(\"a\")\nassert: busiest, \"a\"");
    let mut run = runner();
    run(r#"def checked(zone):
    if zone == "void":
        throw "BAD_ZONE"
    return zone
zones = ["a", "b", "b"]
busiest = mode of zones.map(checked)
assert: busiest, "b"
failure = ""
try:
    zones.shift()
    zones.push("void")
catch error:
    failure = error
assert: failure, "BAD_ZONE"
assert: zones, ["a", "b", "b"]
assert: busiest, "b"
zones.push("a")
zones.push("a")
assert: busiest, "a""#);
}

/// Nucleoid reads noun queries of snapshots versions and other receivers
#[rustfmt::skip]
#[test]
fn reads_noun_queries_of_snapshots_versions_and_other_receivers() {
    runner()("history = [1]\nhistory.push(4)\nfirst = sum of history@first\nfrozen = sum of history.value\nhistory.push(5)\nassert: [first, frozen, sum of history], [1, 5, 10]\nnothing = null\nassert: sum of nothing, null\nassert: mode of [], null\nassert: [max of [], min of []], [Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY]\nclass Shelf:\n    def mode(): return 9\nshelf = Shelf()\nassert: mode of shelf, 9\nassert: mode of Shelf, shelf\nrecord = {sum: () -> 5}\nassert: sum of record, 5\nfailure = \"\"\ntry:\n    sum of \"abc\"\ncatch error:\n    failure = error\nassert: failure, TypeError(\"abc.sum is not a function\")");
    let mut run = runner();
    run(r#"history = [1]
history.push(4)
first = sum of history@first
frozen = sum of history.value
history.push(5)
assert: [first, frozen, sum of history], [1, 5, 10]
nothing = null
assert: sum of nothing, null
assert: mode of [], null
assert: [max of [], min of []], [Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY]
class Shelf:
    def mode(): return 9
shelf = Shelf()
assert: mode of shelf, 9
assert: mode of Shelf, shelf
record = {sum: () -> 5}
assert: sum of record, 5
failure = ""
try:
    sum of "abc"
catch error:
    failure = error
assert: failure, TypeError("abc.sum is not a function")"#);
}

/// Nucleoid finds the extreme items of lists with maxBy and minBy
#[rustfmt::skip]
#[test]
fn finds_the_extreme_items_of_lists_with_maxby_and_minby() {
    runner()("readings = [{at: 9, c: 18}, {at: 12, c: 24}, {at: 15, c: 24}, {at: 18, c: 9}]\nwarmest = readings.maxBy(reading -> reading.c)\ncoldest = readings.minBy(reading -> reading.c)\nassert: warmest.at, 12\nassert: coldest.at, 18\nassert: warmest == readings[1], true\nreadings.push({at: 21, c: 30})\nreadings.shift()\nassert: warmest.at, 21\nassert: coldest.at, 18\nreadings.pop()\nreadings.pop()\nassert: warmest.at, 12\nassert: coldest.at, 12\nreadings = []\nassert: warmest, null\nassert: coldest, null\ntrip = {legs: [{to: \"oslo\", km: 410, mins: 300}, {to: \"voss\", km: 110, mins: 75}]}\nfastest = trip.legs.maxBy(leg -> leg.km / leg.mins)\nshortest = trip.legs.minBy(leg -> leg.km)\nassert: fastest.to, \"voss\"\nassert: shortest.to, \"voss\"\ntrip.legs.push({to: \"lom\", km: 60, mins: 90})\ntrip.legs.push({to: \"alta\", km: 900, mins: 480})\ntrip.legs.push({to: \"bodo\", km: 300, mins: 400})\nassert: fastest.to, \"alta\"\nassert: shortest.to, \"lom\"");
    let mut run = runner();
    run("readings = [{at: 9, c: 18}, {at: 12, c: 24}, {at: 15, c: 24}, {at: 18, c: 9}]");
    run("warmest = readings.maxBy(reading -> reading.c)");
    run("coldest = readings.minBy(reading -> reading.c)");
    assert_eq!(run("warmest.at"), 12);
    assert_eq!(run("coldest.at"), 18);
    assert_eq!(run("warmest == readings[1]"), true);
    run("readings.push({at: 21, c: 30})");
    run("readings.shift()");
    assert_eq!(run("warmest.at"), 21);
    assert_eq!(run("coldest.at"), 18);
    run("readings.pop()");
    run("readings.pop()");
    assert_eq!(run("warmest.at"), 12);
    assert_eq!(run("coldest.at"), 12);
    run("readings = []");
    assert_eq!(run("warmest"), serde_json::Value::Null);
    assert_eq!(run("coldest"), serde_json::Value::Null);
    run(r#"trip = {legs: [{to: "oslo", km: 410, mins: 300}, {to: "voss", km: 110, mins: 75}]}"#);
    run("fastest = trip.legs.maxBy(leg -> leg.km / leg.mins)");
    run("shortest = trip.legs.minBy(leg -> leg.km)");
    assert_eq!(run("fastest.to"), "voss");
    assert_eq!(run("shortest.to"), "voss");
    run(r#"trip.legs.push({to: "lom", km: 60, mins: 90})"#);
    run(r#"trip.legs.push({to: "alta", km: 900, mins: 480})"#);
    run(r#"trip.legs.push({to: "bodo", km: 300, mins: 400})"#);
    assert_eq!(run("fastest.to"), "alta");
    assert_eq!(run("shortest.to"), "lom");
}

/// Nucleoid finds the extreme instances of class populations with maxBy and minBy
#[rustfmt::skip]
#[test]
fn finds_the_extreme_instances_of_class_populations_with_maxby_and_minby() {
    runner()("class Bid(bidder, amount, open):\n    this.bidder = bidder\n    this.amount = amount\n    this.open = open\nbest = Bid.maxBy(bid -> bid.amount)\ncheapest = Bid.minBy(bid -> bid.amount)\nassert: best, null\nassert: cheapest, null\nada = Bid(\"ada\", 120, true)\nbo = Bid(\"bo\", 90, false)\ncy = Bid(\"cy\", 150, true)\nleading = Bid.filter(bid -> bid.open).minBy(bid -> bid.amount)\nrecent = Bid[-2:].maxBy(bid -> bid.amount)\nassert: best, cy\nassert: cheapest, bo\nassert: leading, ada\nassert: recent, cy\nbo.amount = 200\nada.open = false\nassert: best, bo\nassert: cheapest, ada\nassert: leading, cy\nassert: recent, bo\ndelete cy.amount\nassert: best, null\nassert: leading, null\ndelete cy.open\ndelete cy.bidder\ndelete cy\nassert: best, bo\nassert: cheapest, ada\nassert: leading, null\nassert: recent, bo");
    let mut run = runner();
    run(r#"class Bid(bidder, amount, open):
    this.bidder = bidder
    this.amount = amount
    this.open = open"#);
    run("best = Bid.maxBy(bid -> bid.amount)");
    run("cheapest = Bid.minBy(bid -> bid.amount)");
    assert_eq!(run("best"), serde_json::Value::Null);
    assert_eq!(run("cheapest"), serde_json::Value::Null);
    run(r#"ada = Bid("ada", 120, true)"#);
    run(r#"bo = Bid("bo", 90, false)"#);
    run(r#"cy = Bid("cy", 150, true)"#);
    run("leading = Bid.filter(bid -> bid.open).minBy(bid -> bid.amount)");
    run("recent = Bid[-2:].maxBy(bid -> bid.amount)");
    {
        let actual = run("best");
        let expected = run("(cy)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("cheapest");
        let expected = run("(bo)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("leading");
        let expected = run("(ada)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("recent");
        let expected = run("(cy)");
        assert_eq!(actual, expected);
    }
    run("bo.amount = 200");
    run("ada.open = false");
    {
        let actual = run("best");
        let expected = run("(bo)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("cheapest");
        let expected = run("(ada)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("leading");
        let expected = run("(cy)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("recent");
        let expected = run("(bo)");
        assert_eq!(actual, expected);
    }
    run("delete cy.amount");
    assert_eq!(run("best"), serde_json::Value::Null);
    assert_eq!(run("leading"), serde_json::Value::Null);
    run("delete cy.open");
    run("delete cy.bidder");
    run("delete cy");
    {
        let actual = run("best");
        let expected = run("(bo)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("cheapest");
        let expected = run("(ada)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("leading"), serde_json::Value::Null);
    {
        let actual = run("recent");
        let expected = run("(bo)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid compares maxBy and minBy keys as the comparison operators do
#[rustfmt::skip]
#[test]
fn compares_maxby_and_minby_keys_as_the_comparison_operators_do() {
    runner()("class Job(name, due, kg):\n    this.name = name\n    this.due = due\n    this.kg = kg\nalpha = Job.minBy(job -> job.name)\nsoonest = Job.minBy(job -> job.due)\nlatest = Job.maxBy(job -> job.due)\nheaviest = Job.maxBy(job -> job.kg)\npress = Job(\"press\", new Date(\"2026-12-01\"), 40)\ncoat = Job(\"Coat\", new Date(\"2026-11-20\"), \"9\")\nweld = Job(\"weld\", new Date(\"2026-11-28\"), 40)\nassert: alpha, coat\nassert: soonest, coat\nassert: latest, press\nassert: heaviest, press\ncoat.kg = \"75\"\nweld.due = new Date(\"2026-11-09\")\nJob(\"Anvil\", new Date(\"2026-12-24\"), \"100\")\nassert: alpha.name, \"Anvil\"\nassert: soonest, weld\nassert: latest.name, \"Anvil\"\nassert: heaviest, coat\npress.kg = 0 / 0\nassert: heaviest, null\npress.kg = 300\ndelete weld.due\nassert: heaviest, press\nassert: soonest, null\nassert: latest, null");
    let mut run = runner();
    run(r#"class Job(name, due, kg):
    this.name = name
    this.due = due
    this.kg = kg"#);
    run("alpha = Job.minBy(job -> job.name)");
    run("soonest = Job.minBy(job -> job.due)");
    run("latest = Job.maxBy(job -> job.due)");
    run("heaviest = Job.maxBy(job -> job.kg)");
    run(r#"press = Job("press", new Date("2026-12-01"), 40)"#);
    run(r#"coat = Job("Coat", new Date("2026-11-20"), "9")"#);
    run(r#"weld = Job("weld", new Date("2026-11-28"), 40)"#);
    {
        let actual = run("alpha");
        let expected = run("(coat)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("soonest");
        let expected = run("(coat)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("latest");
        let expected = run("(press)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("heaviest");
        let expected = run("(press)");
        assert_eq!(actual, expected);
    }
    run(r#"coat.kg = "75""#);
    run(r#"weld.due = new Date("2026-11-09")"#);
    run(r#"Job("Anvil", new Date("2026-12-24"), "100")"#);
    assert_eq!(run("alpha.name"), "Anvil");
    {
        let actual = run("soonest");
        let expected = run("(weld)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("latest.name"), "Anvil");
    {
        let actual = run("heaviest");
        let expected = run("(coat)");
        assert_eq!(actual, expected);
    }
    run("press.kg = 0 / 0");
    assert_eq!(run("heaviest"), serde_json::Value::Null);
    run("press.kg = 300");
    run("delete weld.due");
    {
        let actual = run("heaviest");
        let expected = run("(press)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("soonest"), serde_json::Value::Null);
    assert_eq!(run("latest"), serde_json::Value::Null);
}

/// Nucleoid calls the extreme item selector once per item with only the item
#[rustfmt::skip]
#[test]
fn calls_the_extreme_item_selector_once_per_item_with_only_the_item() {
    runner()("seen = []\ndef weigh(parcel, position):\n    seen.push(position ?? parcel.id)\n    return parcel.kg\nparcels = [{id: 1, kg: 4}, {id: 2, kg: 9}, {id: 3, kg: 2}]\ndef heaviest() => parcels.maxBy(weigh)\nassert: heaviest().id, 2\nassert: seen, [1, 2, 3]\nparcels.push({id: 4, kg: 12})\nassert: heaviest().id, 4\nassert: seen, [1, 2, 3, 1, 2, 3, 4]\ndepot = 5\ndef gap parcel => Math.abs(parcel.kg - depot)\nclosest = parcels.minBy(gap)\nassert: closest.id, 1\ndepot = 10\nassert: closest.id, 2\nmissing = null\nskipped = missing.minBy(weigh(9))\nassert: skipped, null\nassert: seen, [1, 2, 3, 1, 2, 3, 4]");
    let mut run = runner();
    run("seen = []");
    run(r#"def weigh(parcel, position):
    seen.push(position ?? parcel.id)
    return parcel.kg"#);
    run("parcels = [{id: 1, kg: 4}, {id: 2, kg: 9}, {id: 3, kg: 2}]");
    run("def heaviest() => parcels.maxBy(weigh)");
    assert_eq!(run("heaviest().id"), 2);
    assert_eq!(run("seen"), serde_json::json!([1, 2, 3]));
    run("parcels.push({id: 4, kg: 12})");
    assert_eq!(run("heaviest().id"), 4);
    assert_eq!(run("seen"), serde_json::json!([1, 2, 3, 1, 2, 3, 4]));
    run("depot = 5");
    run("def gap parcel => Math.abs(parcel.kg - depot)");
    run("closest = parcels.minBy(gap)");
    assert_eq!(run("closest.id"), 1);
    run("depot = 10");
    assert_eq!(run("closest.id"), 2);
    run("missing = null");
    run("skipped = missing.minBy(weigh(9))");
    assert_eq!(run("skipped"), serde_json::Value::Null);
    assert_eq!(run("seen"), serde_json::json!([1, 2, 3, 1, 2, 3, 4]));
}

/// Nucleoid keeps extreme items live in class level rules
#[rustfmt::skip]
#[test]
fn keeps_extreme_items_live_in_class_level_rules() {
    runner()("class Team(name, target):\n    this.name = name\n    this.target = target\nclass Player(team, name, goals):\n    this.team = team\n    this.name = name\n    this.goals = goals\n$Team.star = Player.filter(player -> player.team == $Team.name).maxBy(player -> player.goals)\n$Team.total = Player.filter(player -> player.team == $Team.name).sum(player -> player.goals)\n$Team.fit = Player.minBy(player -> Math.abs(player.goals - $Team.target))\nreds = Team(\"reds\", 4)\nblues = Team(\"blues\", 10)\nleader = Team.maxBy(team -> team.total)\nana = Player(\"reds\", \"ana\", 7)\nPlayer(\"reds\", \"ben\", 3)\ncal = Player(\"blues\", \"cal\", 5)\nassert: reds.star, ana\nassert: blues.star, cal\nassert: reds.fit.name, \"ben\"\nassert: blues.fit, ana\nassert: leader, reds\ncal.goals = 12\nassert: blues.star, cal\nassert: blues.fit, cal\nassert: leader, blues\ncal.team = \"reds\"\nassert: reds.star, cal\nassert: blues.star, null\nassert: leader, reds\ngreens = Team(\"greens\", 6)\nassert: greens.star, null\nassert: greens.fit, ana");
    let mut run = runner();
    run(r#"class Team(name, target):
    this.name = name
    this.target = target"#);
    run(r#"class Player(team, name, goals):
    this.team = team
    this.name = name
    this.goals = goals"#);
    run("$Team.star = Player.filter(player -> player.team == $Team.name).maxBy(player -> player.goals)");
    run("$Team.total = Player.filter(player -> player.team == $Team.name).sum(player -> player.goals)");
    run("$Team.fit = Player.minBy(player -> Math.abs(player.goals - $Team.target))");
    run(r#"reds = Team("reds", 4)"#);
    run(r#"blues = Team("blues", 10)"#);
    run("leader = Team.maxBy(team -> team.total)");
    run(r#"ana = Player("reds", "ana", 7)"#);
    run(r#"Player("reds", "ben", 3)"#);
    run(r#"cal = Player("blues", "cal", 5)"#);
    {
        let actual = run("reds.star");
        let expected = run("(ana)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("blues.star");
        let expected = run("(cal)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("reds.fit.name"), "ben");
    {
        let actual = run("blues.fit");
        let expected = run("(ana)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("leader");
        let expected = run("(reds)");
        assert_eq!(actual, expected);
    }
    run("cal.goals = 12");
    {
        let actual = run("blues.star");
        let expected = run("(cal)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("blues.fit");
        let expected = run("(cal)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("leader");
        let expected = run("(blues)");
        assert_eq!(actual, expected);
    }
    run(r#"cal.team = "reds""#);
    {
        let actual = run("reds.star");
        let expected = run("(cal)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("blues.star"), serde_json::Value::Null);
    {
        let actual = run("leader");
        let expected = run("(reds)");
        assert_eq!(actual, expected);
    }
    run(r#"greens = Team("greens", 6)"#);
    assert_eq!(run("greens.star"), serde_json::Value::Null);
    {
        let actual = run("greens.fit");
        let expected = run("(ana)");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid uses extreme items in functions methods and conditional expressions
#[rustfmt::skip]
#[test]
fn uses_extreme_items_in_functions_methods_and_conditional_expressions() {
    runner()("class Shelf(name, items):\n    this.name = name\n    this.items = items\n    def priciest(): return this.items.maxBy(item -> item.price)\ndef cheapest(shelf) => shelf.items.minBy(item -> item.price)\nleft = Shelf(\"left\", [{sku: \"pen\", price: 2}, {sku: \"ink\", price: 9}])\nright = Shelf(\"right\", [{sku: \"pad\", price: 4}, {sku: \"cap\", price: 1}])\ndearer = left.priciest().price > right.priciest().price ? left.name : right.name\ntop = Shelf.maxBy(shelf -> shelf.priciest().price)\nbargain = Shelf.minBy(shelf -> cheapest(shelf).price)\nassert: dearer, \"left\"\nassert: top, left\nassert: bargain, right\nassert: cheapest(left).sku, \"pen\"\nassert: right.priciest().sku, \"pad\"\nright.items.push({sku: \"tape\", price: 15})\nleft.items = [{sku: \"clip\", price: 0.5}]\nassert: dearer, \"right\"\nassert: top, right\nassert: bargain, left\nassert: cheapest(left).sku, \"clip\"");
    let mut run = runner();
    run(r#"class Shelf(name, items):
    this.name = name
    this.items = items
    def priciest(): return this.items.maxBy(item -> item.price)"#);
    run("def cheapest(shelf) => shelf.items.minBy(item -> item.price)");
    run(r#"left = Shelf("left", [{sku: "pen", price: 2}, {sku: "ink", price: 9}])"#);
    run(r#"right = Shelf("right", [{sku: "pad", price: 4}, {sku: "cap", price: 1}])"#);
    run("dearer = left.priciest().price > right.priciest().price ? left.name : right.name");
    run("top = Shelf.maxBy(shelf -> shelf.priciest().price)");
    run("bargain = Shelf.minBy(shelf -> cheapest(shelf).price)");
    assert_eq!(run("dearer"), "left");
    {
        let actual = run("top");
        let expected = run("(left)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("bargain");
        let expected = run("(right)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("cheapest(left).sku"), "pen");
    assert_eq!(run("right.priciest().sku"), "pad");
    run(r#"right.items.push({sku: "tape", price: 15})"#);
    run(r#"left.items = [{sku: "clip", price: 0.5}]"#);
    assert_eq!(run("dearer"), "right");
    {
        let actual = run("top");
        let expected = run("(right)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("bargain");
        let expected = run("(left)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("cheapest(left).sku"), "clip");
}

/// Nucleoid keeps maxBy and minBy as ordinary names beside the extreme item queries
#[rustfmt::skip]
#[test]
fn keeps_maxby_and_minby_as_ordinary_names_beside_the_extreme_item_queries() {
    runner()("maxBy = 3\nminBy = 1\nlimits = {maxBy: size -> size * 10, minBy: 2}\ndef scale minBy => minBy * 2\nclass Crate(items):\n    this.items = items\n    def maxBy(key): return key(this.items.length)\ncrate = Crate([{kg: 6}, {kg: 4}])\nspare = Crate([{kg: 9}])\nheavy = crate.items.maxBy(box -> box.kg)\nfullest = Crate.maxBy(each -> each.maxBy(count -> count))\nlightest = Crate.minBy(each -> each.items.min(box -> box.kg))\nassert: heavy.kg, 6\nassert: fullest, crate\nassert: lightest, crate\nassert: crate.maxBy(count -> count + maxBy), 5\nassert: limits.maxBy(minBy), 10\nassert: scale(minBy), 2\nassert: crate.items.maxBy, null\nspare.items = [{kg: 1}, {kg: 2}, {kg: 3}, {kg: 9}]\ncrate.items.push({kg: 7})\nmaxBy = 5\nassert: heavy.kg, 7\nassert: fullest, spare\nassert: lightest, spare\nassert: crate.maxBy(count -> count + maxBy), 8");
    let mut run = runner();
    run("maxBy = 3");
    run("minBy = 1");
    run("limits = {maxBy: size -> size * 10, minBy: 2}");
    run("def scale minBy => minBy * 2");
    run(r#"class Crate(items):
    this.items = items
    def maxBy(key): return key(this.items.length)"#);
    run("crate = Crate([{kg: 6}, {kg: 4}])");
    run("spare = Crate([{kg: 9}])");
    run("heavy = crate.items.maxBy(box -> box.kg)");
    run("fullest = Crate.maxBy(each -> each.maxBy(count -> count))");
    run("lightest = Crate.minBy(each -> each.items.min(box -> box.kg))");
    assert_eq!(run("heavy.kg"), 6);
    {
        let actual = run("fullest");
        let expected = run("(crate)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("lightest");
        let expected = run("(crate)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("crate.maxBy(count -> count + maxBy)"), 5);
    assert_eq!(run("limits.maxBy(minBy)"), 10);
    assert_eq!(run("scale(minBy)"), 2);
    assert_eq!(run("crate.items.maxBy"), serde_json::Value::Null);
    run("spare.items = [{kg: 1}, {kg: 2}, {kg: 3}, {kg: 9}]");
    run("crate.items.push({kg: 7})");
    run("maxBy = 5");
    assert_eq!(run("heavy.kg"), 7);
    {
        let actual = run("fullest");
        let expected = run("(spare)");
        assert_eq!(actual, expected);
    }
    {
        let actual = run("lightest");
        let expected = run("(spare)");
        assert_eq!(actual, expected);
    }
    assert_eq!(run("crate.maxBy(count -> count + maxBy)"), 8);
}

/// Nucleoid rolls back failed extreme item updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_extreme_item_updates() {
    runner()("def checked(value):\n    if value > 100:\n        throw \"TOO_MANY\"\n    return value\norders = [{id: 1, qty: 30}, {id: 2, qty: 50}]\nlargest = orders.maxBy(order -> checked(order.qty))\nassert: largest.id, 2\nfailure = \"\"\ntry:\n    orders.push({id: 3, qty: 500})\ncatch error:\n    failure = error\nassert: failure, \"TOO_MANY\"\nassert: orders.length, 2\nassert: largest.id, 2\norders.push({id: 4, qty: 80})\nassert: largest.id, 4\nclass Probe(name, temp):\n    this.name = name\n    this.temp = temp\nProbe(\"a\", 12)\nProbe(\"b\", 30)\ncoldest = Probe.minBy(probe -> checked(probe.temp))\nfailure = \"\"\ntry:\n    Probe(\"c\", 500)\ncatch error:\n    failure = error\nassert: failure, \"TOO_MANY\"\nassert: Probe.length, 2\nassert: coldest.name, \"a\"\nProbe(\"d\", 9)\nassert: coldest.name, \"d\"");
    let mut run = runner();
    run(r#"def checked(value):
    if value > 100:
        throw "TOO_MANY"
    return value
orders = [{id: 1, qty: 30}, {id: 2, qty: 50}]
largest = orders.maxBy(order -> checked(order.qty))
assert: largest.id, 2
failure = ""
try:
    orders.push({id: 3, qty: 500})
catch error:
    failure = error
assert: failure, "TOO_MANY"
assert: orders.length, 2
assert: largest.id, 2
orders.push({id: 4, qty: 80})
assert: largest.id, 4
class Probe(name, temp):
    this.name = name
    this.temp = temp
Probe("a", 12)
Probe("b", 30)
coldest = Probe.minBy(probe -> checked(probe.temp))
failure = ""
try:
    Probe("c", 500)
catch error:
    failure = error
assert: failure, "TOO_MANY"
assert: Probe.length, 2
assert: coldest.name, "a"
Probe("d", 9)
assert: coldest.name, "d""#);
}

/// Nucleoid declares subclasses with extends clauses
#[rustfmt::skip]
#[test]
fn declares_subclasses_with_extends_clauses() {
    let mut run = runner();
    run(r#"class Animal(name):
    this.name = name"#);
    run("$Animal.legs = 4");
    run(r#"class Dog(name, breed) extends Animal:
    this.name = name
    this.breed = breed
    def speak():
        return this.name + " barks""#);
    run(r#"class Cat extends Animal:
    def speak() => this.name + " meows""#);
    run(r#"rex = Dog("Rex", "lab")"#);
    run(r#"tom = Cat("Tom")"#);
    assert_eq!(run("rex.breed"), "lab");
    assert_eq!(run("rex.speak()"), "Rex barks");
    assert_eq!(run("tom.name"), "Tom");
    assert_eq!(run("tom.speak()"), "Tom meows");
    assert_eq!(run("rex.legs"), 4);
    assert_eq!(run("tom.legs"), 4);
    assert_eq!(run("Animal.length"), 0);
    assert_eq!(run("Dog.length"), 1);
    assert_eq!(run("Cat.length"), 1);
    run("$Animal.legs = 3");
    assert_eq!(run("rex.legs"), 3);
    assert_eq!(run("tom.legs"), 3);
}

/// Nucleoid declares bodyless subclasses with extends clauses
#[rustfmt::skip]
#[test]
fn declares_bodyless_subclasses_with_extends_clauses() {
    let mut run = runner();
    run(r#"class Shape(sides):
    this.sides = sides"#);
    run("$Shape.polygon = $Shape.sides > 2");
    run("class Square extends Shape");
    run("square = Square(4)");
    run("class Triangle(sides) extends Shape; triangle = Triangle(3)");
    run("{ class Segment extends Shape }");
    run("segment = Segment(1)");
    assert_eq!(run("square.sides"), 4);
    assert_eq!(run("triangle.sides"), 3);
    assert_eq!(run("square.polygon"), true);
    assert_eq!(run("segment.polygon"), false);
    assert_eq!(run("Shape.length"), 0);
    assert_eq!(run("Square.length"), 1);
    assert_eq!(run("Triangle.length"), 1);
    assert_eq!(run("Segment.length"), 1);
    run("segment.sides = 5");
    assert_eq!(run("segment.polygon"), true);
}

/// Nucleoid keeps extends subclasses equivalent to colon subclasses
#[rustfmt::skip]
#[test]
fn keeps_extends_subclasses_equivalent_to_colon_subclasses() {
    let mut run = runner();
    run(r#"class Account(owner, balance):
    this.owner = owner
    this.balance = balance
$Account.overdrawn = $Account.balance < 0
class Savings: Account
    def init(owner, balance, rate):
        super(owner, balance)
        this.rate = rate
class Deposit extends Account:
    def init(owner, balance, rate):
        super(owner, balance)
        this.rate = rate
saving = Savings("ada", 100, 2)
deposit = Deposit(rate=2, owner="ada", balance=100)
assert(deposit.owner, saving.owner)
assert(deposit.rate, saving.rate)
assert(deposit.overdrawn, false)
deposit.balance = -5
saving.balance = -5
assert(deposit.overdrawn, true)
assert(saving.overdrawn, true)
assert(Account.length, 0)
assert(Deposit.length, Savings.length)"#);
}

/// Nucleoid resolves super and inherited rules through extends chains
#[rustfmt::skip]
#[test]
fn resolves_super_and_inherited_rules_through_extends_chains() {
    let mut run = runner();
    run(r#"class Base:
    def init():
        this.trace = "A""#);
    run(r#"class Middle extends Base:
    def init():
        super
        this.trace = this.trace + "B""#);
    run(r#"class Leaf: Middle
    def init():
        super
        this.trace = this.trace + "C""#);
    run("class Passive extends Leaf");
    run("$Base.ready = true");
    run("leaf = Leaf()");
    run("passive = Passive()");
    assert_eq!(run("leaf.trace"), "ABC");
    assert_eq!(run("passive.trace"), "ABC");
    assert_eq!(run("passive.ready"), true);
    assert_eq!(run("Base.length"), 0);
    assert_eq!(run("Middle.length"), 0);
    assert_eq!(run("Leaf.length"), 1);
    assert_eq!(run("Passive.length"), 1);
}

/// Nucleoid keeps extends an ordinary name
#[rustfmt::skip]
#[test]
fn keeps_extends_an_ordinary_name() {
    let mut run = runner();
    run("extends = 2");
    run("limits = {extends: 3}");
    run("def scale(extends) => extends * 10");
    run(r#"class Gate:
    def extends(step) => step + 1"#);
    run("gate = Gate()");
    run("class extends");
    run("class Probe extends extends");
    run("$extends.ready = true");
    run("probe = Probe()");
    assert_eq!(run("extends + limits.extends"), 5);
    assert_eq!(run("scale(extends)"), 20);
    assert_eq!(run("gate.extends(4)"), 5);
    assert_eq!(run("probe.ready"), true);
    assert_eq!(run("Probe.length"), 1);
    run("extends = 4");
    assert_eq!(run("scale(extends)"), 40);
}

/// Nucleoid rolls back extends class declarations
#[rustfmt::skip]
#[test]
fn rolls_back_extends_class_declarations() {
    let mut run = runner();
    run(r#"class Base
try:
    class Loop extends Loop
catch error:
    assert(error, TypeError("Circular Inheritance"))
class Child extends Base
try:
    class Base extends Child
catch error:
    assert(error, TypeError("Circular Inheritance"))
assert(Class.length, 2)
class Limited extends Base:
    def init(amount):
        if amount > 10:
            throw "LIMIT"
        this.amount = amount
small = Limited(5)
try:
    large = Limited(20)
catch error:
    assert(error, "LIMIT")
assert(Limited.length, 1)
assert(small.amount, 5)"#);
}

/// Nucleoid reads list item properties with selector arguments
#[rustfmt::skip]
#[test]
fn reads_list_item_properties_with_selector_arguments() {
    runner()("pen = {item: \"pen\", qty: 2, paid: true}\nink = {item: \"ink\", qty: 5, paid: false}\norders = [pen, ink]\nitems = orders.map(.item)\nunits = orders.sum(.qty)\nsettled = orders.count(.paid)\nlargest = orders.maxBy(.qty)\nassert: [items, units, settled, largest], [[\"pen\", \"ink\"], 7, 1, ink]\nink.paid = true\npen.qty = 8\norders.push({item: \"cap\", qty: 1, paid: false})\nassert: [items, units, settled, largest], [[\"pen\", \"ink\", \"cap\"], 14, 2, pen]\norders.shift()\nassert: [items, units, settled, largest], [[\"ink\", \"cap\"], 6, 1, ink]");
    let mut run = runner();
    run(r#"pen = {item: "pen", qty: 2, paid: true}"#);
    run(r#"ink = {item: "ink", qty: 5, paid: false}"#);
    run("orders = [pen, ink]");
    run("items = orders.map(.item)");
    run("units = orders.sum(.qty)");
    run("settled = orders.count(.paid)");
    run("largest = orders.maxBy(.qty)");
    {
        let actual = run("[items, units, settled, largest]");
        let expected = run(r#"([["pen", "ink"], 7, 1, ink])"#);
        assert_eq!(actual, expected);
    }
    run("ink.paid = true");
    run("pen.qty = 8");
    run(r#"orders.push({item: "cap", qty: 1, paid: false})"#);
    {
        let actual = run("[items, units, settled, largest]");
        let expected = run(r#"([["pen", "ink", "cap"], 14, 2, pen])"#);
        assert_eq!(actual, expected);
    }
    run("orders.shift()");
    {
        let actual = run("[items, units, settled, largest]");
        let expected = run(r#"([["ink", "cap"], 6, 1, ink])"#);
        assert_eq!(actual, expected);
    }
}

/// Nucleoid reads nested property paths with selector arguments
#[rustfmt::skip]
#[test]
fn reads_nested_property_paths_with_selector_arguments() {
    runner()("oslo = {to: {city: \"Oslo\", zip: \"0150\"}, tags: [\"cold\", \"fragile\"], \"class\": \"air\"}\nbergen = {to: {city: \"Bergen\"}, tags: [], \"class\": \"sea\"}\nshipments = [oslo, bergen]\ncities = shipments.map(.to.city)\nzips = shipments.map(.to.zip)\ncounts = shipments.map(.tags.length)\nmodes = shipments.map(.class)\nassert: [cities, zips, counts, modes], [[\"Oslo\", \"Bergen\"], [\"0150\", null], [2, 0], [\"air\", \"sea\"]]\nbergen.to.zip = \"5003\"\nbergen.tags.push(\"bulk\")\nshipments.push({tags: [], \"class\": \"rail\"})\nassert: [cities, zips, counts, modes], [[\"Oslo\", \"Bergen\", null], [\"0150\", \"5003\", null], [2, 1, 0], [\"air\", \"sea\", \"rail\"]]\noslo.to = null\nassert: [cities, zips, counts], [null, null, [2, 1, 0]]\noslo.to = {city: \"Tromso\"}\nassert: [cities, zips], [[\"Tromso\", \"Bergen\", null], [null, \"5003\", null]]");
    let mut run = runner();
    run(r#"oslo = {to: {city: "Oslo", zip: "0150"}, tags: ["cold", "fragile"], "class": "air"}"#);
    run(r#"bergen = {to: {city: "Bergen"}, tags: [], "class": "sea"}"#);
    run("shipments = [oslo, bergen]");
    run("cities = shipments.map(.to.city)");
    run("zips = shipments.map(.to.zip)");
    run("counts = shipments.map(.tags.length)");
    run("modes = shipments.map(.class)");
    assert_eq!(run("[cities, zips, counts, modes]"), serde_json::json!([["Oslo", "Bergen"], ["0150", null], [2, 0], ["air", "sea"]]));
    run(r#"bergen.to.zip = "5003""#);
    run(r#"bergen.tags.push("bulk")"#);
    run(r#"shipments.push({tags: [], "class": "rail"})"#);
    assert_eq!(run("[cities, zips, counts, modes]"), serde_json::json!([["Oslo", "Bergen", null], ["0150", "5003", null], [2, 1, 0], ["air", "sea", "rail"]]));
    run("oslo.to = null");
    assert_eq!(run("[cities, zips, counts]"), serde_json::json!([null, null, [2, 1, 0]]));
    run(r#"oslo.to = {city: "Tromso"}"#);
    assert_eq!(run("[cities, zips]"), serde_json::json!([["Tromso", "Bergen", null], [null, "5003", null]]));
}

/// Nucleoid reads class instance properties with selector arguments
#[rustfmt::skip]
#[test]
fn reads_class_instance_properties_with_selector_arguments() {
    runner()("class Bid(bidder, amount, open):\n    this.bidder = bidder\n    this.amount = amount\n    this.open = open\nlow = Bid.min(.amount)\nleader = Bid.maxBy(.amount)\nbidding = Bid.filter(.open).map(.bidder)\nrecent = Bid[-1:].map(.bidder)\nassert: [low, leader, bidding, recent], [Number.POSITIVE_INFINITY, null, [], []]\nada = Bid(\"ada\", 120, true)\nbo = Bid(\"bo\", 90, false)\nassert: [low, leader, bidding, recent], [90, ada, [\"ada\"], [\"bo\"]]\nbo.amount = 150\nbo.open = true\nassert: [low, leader, bidding], [120, bo, [\"ada\", \"bo\"]]\ndelete bo.open\ndelete bo.bidder\ndelete bo.amount\ndelete bo\nassert: [low, leader, bidding, recent], [120, ada, [\"ada\"], [\"ada\"]]");
    let mut run = runner();
    run(r#"class Bid(bidder, amount, open):
    this.bidder = bidder
    this.amount = amount
    this.open = open"#);
    run("low = Bid.min(.amount)");
    run("leader = Bid.maxBy(.amount)");
    run("bidding = Bid.filter(.open).map(.bidder)");
    run("recent = Bid[-1:].map(.bidder)");
    {
        let actual = run("[low, leader, bidding, recent]");
        let expected = run("([Number.POSITIVE_INFINITY, null, [], []])");
        assert_eq!(actual, expected);
    }
    run(r#"ada = Bid("ada", 120, true)"#);
    run(r#"bo = Bid("bo", 90, false)"#);
    {
        let actual = run("[low, leader, bidding, recent]");
        let expected = run(r#"([90, ada, ["ada"], ["bo"]])"#);
        assert_eq!(actual, expected);
    }
    run("bo.amount = 150");
    run("bo.open = true");
    {
        let actual = run("[low, leader, bidding]");
        let expected = run(r#"([120, bo, ["ada", "bo"]])"#);
        assert_eq!(actual, expected);
    }
    run("delete bo.open");
    run("delete bo.bidder");
    run("delete bo.amount");
    run("delete bo");
    {
        let actual = run("[low, leader, bidding, recent]");
        let expected = run(r#"([120, ada, ["ada"], ["ada"]])"#);
        assert_eq!(actual, expected);
    }
}

/// Nucleoid keeps selector arguments live in class level rules
#[rustfmt::skip]
#[test]
fn keeps_selector_arguments_live_in_class_level_rules() {
    runner()("class Order(lines): this.lines = lines\n$Order.units = $Order.lines.sum(.qty)\n$Order.skus = $Order.lines.map(.sku)\n$Order.bulky = $Order.lines.count(.bulk)\nsmall = Order([{sku: \"pen\", qty: 2, bulk: false}])\nlarge = Order([{sku: \"ink\", qty: 50, bulk: true}, {sku: \"cap\", qty: 1, bulk: false}])\nbusiest = Order.maxBy(.units)\nassert: [small.units, small.skus, small.bulky, large.units, large.bulky], [2, [\"pen\"], 0, 51, 1]\nassert: busiest, large\nsmall.lines.push({sku: \"pad\", qty: 80, bulk: true})\nassert: [small.units, small.skus, small.bulky, busiest], [82, [\"pen\", \"pad\"], 1, small]\nlarge.lines = [{sku: \"box\", qty: 99, bulk: true}]\nassert: [large.units, large.skus, large.bulky, busiest], [99, [\"box\"], 1, large]");
    let mut run = runner();
    run("class Order(lines): this.lines = lines");
    run("$Order.units = $Order.lines.sum(.qty)");
    run("$Order.skus = $Order.lines.map(.sku)");
    run("$Order.bulky = $Order.lines.count(.bulk)");
    run(r#"small = Order([{sku: "pen", qty: 2, bulk: false}])"#);
    run(r#"large = Order([{sku: "ink", qty: 50, bulk: true}, {sku: "cap", qty: 1, bulk: false}])"#);
    run("busiest = Order.maxBy(.units)");
    assert_eq!(run("[small.units, small.skus, small.bulky, large.units, large.bulky]"), serde_json::json!([2, ["pen"], 0, 51, 1]));
    {
        let actual = run("busiest");
        let expected = run("(large)");
        assert_eq!(actual, expected);
    }
    run(r#"small.lines.push({sku: "pad", qty: 80, bulk: true})"#);
    {
        let actual = run("[small.units, small.skus, small.bulky, busiest]");
        let expected = run(r#"([82, ["pen", "pad"], 1, small])"#);
        assert_eq!(actual, expected);
    }
    run(r#"large.lines = [{sku: "box", qty: 99, bulk: true}]"#);
    {
        let actual = run("[large.units, large.skus, large.bulky, busiest]");
        let expected = run(r#"([99, ["box"], 1, large])"#);
        assert_eq!(actual, expected);
    }
}

/// Nucleoid passes selector arguments as ordinary functions
#[rustfmt::skip]
#[test]
fn passes_selector_arguments_as_ordinary_functions() {
    runner()("def spread(list, key) => list.max(key) - list.min(key)\ndef column(list, key) => list.map(key).join(\"/\")\nboxes = [{w: 2, h: 9, tag: \"a\"}, {w: 5, h: 4, tag: \"b\"}]\nwide = spread(boxes, .w)\ntall = spread(boxes, .h)\nwidths = column(boxes, .w)\ntags = column(\n    boxes,\n    .tag\n)\nchain = {at: 1, next: {at: 2, next: {at: 3, next: null}}}\nsteps = [\"x\", \"y\"]\nhop = steps.reduce(.next, chain)\nassert: [wide, tall, widths, tags, hop.at], [3, 5, \"2/5\", \"a/b\", 3]\nboxes.push({w: 11, h: 1, tag: \"c\"})\nsteps.pop()\nassert: [wide, tall, widths, tags, hop.at], [9, 8, \"2/5/11\", \"a/b/c\", 2]");
    let mut run = runner();
    run("def spread(list, key) => list.max(key) - list.min(key)");
    run(r#"def column(list, key) => list.map(key).join("/")"#);
    run(r#"boxes = [{w: 2, h: 9, tag: "a"}, {w: 5, h: 4, tag: "b"}]"#);
    run("wide = spread(boxes, .w)");
    run("tall = spread(boxes, .h)");
    run("widths = column(boxes, .w)");
    run(r#"tags = column(
    boxes,
    .tag
)"#);
    run("chain = {at: 1, next: {at: 2, next: {at: 3, next: null}}}");
    run(r#"steps = ["x", "y"]"#);
    run("hop = steps.reduce(.next, chain)");
    assert_eq!(run("[wide, tall, widths, tags, hop.at]"), serde_json::json!([3, 5, "2/5", "a/b", 3]));
    run(r#"boxes.push({w: 11, h: 1, tag: "c"})"#);
    run("steps.pop()");
    assert_eq!(run("[wide, tall, widths, tags, hop.at]"), serde_json::json!([9, 8, "2/5/11", "a/b/c", 2]));
}

/// Nucleoid uses selector arguments in headers methods lambdas and templates
#[rustfmt::skip]
#[test]
fn uses_selector_arguments_in_headers_methods_lambdas_and_templates() {
    runner()("class Part(sku, kg, fragile, cost):\n    this.sku = sku\n    this.kg = kg\n    this.fragile = fragile\n    this.cost = cost\nclass Shelf(parts):\n    this.parts = parts\n    def weight(): return this.parts.sum(.kg)\npen = Part(\"pen\", 2, false, 4)\nink = Part(\"ink\", 5, true, 9)\nleft = Shelf([pen, ink])\ncare = left.parts.some(.fragile) ? \"careful\" : \"plain\"\nsize = \"big\" if left.parts.max(.kg) > 4 else \"small\"\nnote = `${left.parts.map(.sku).join(\"+\")}`\nprice = shelf -> shelf.parts.sum(.cost)\nwrapped = []\nfor part of Part.filter(.fragile):\n    wrapped.push(part.sku)\npending = [{open: true}, {open: true}, {open: false}]\nwhile pending.count(.open) > 1:\n    pending.shift()\nassert: [left.weight(), care, size, note, price(left)], [7, \"careful\", \"big\", \"pen+ink\", 13]\nassert: [wrapped, pending.length], [[\"ink\"], 2]\nink.fragile = false\nink.kg = 3\nassert: [left.weight(), care, size], [5, \"plain\", \"small\"]\nleft.parts.push(Part(\"cap\", 2, true, 1))\nassert: [left.weight(), care, note, price(left)], [7, \"careful\", \"pen+ink+cap\", 14]");
    let mut run = runner();
    run(r#"class Part(sku, kg, fragile, cost):
    this.sku = sku
    this.kg = kg
    this.fragile = fragile
    this.cost = cost"#);
    run(r#"class Shelf(parts):
    this.parts = parts
    def weight(): return this.parts.sum(.kg)"#);
    run(r#"pen = Part("pen", 2, false, 4)"#);
    run(r#"ink = Part("ink", 5, true, 9)"#);
    run("left = Shelf([pen, ink])");
    run(r#"care = left.parts.some(.fragile) ? "careful" : "plain""#);
    run(r#"size = "big" if left.parts.max(.kg) > 4 else "small""#);
    run(r#"note = `${left.parts.map(.sku).join("+")}`"#);
    run("price = shelf -> shelf.parts.sum(.cost)");
    run("wrapped = []");
    run(r#"for part of Part.filter(.fragile):
    wrapped.push(part.sku)"#);
    run("pending = [{open: true}, {open: true}, {open: false}]");
    run(r#"while pending.count(.open) > 1:
    pending.shift()"#);
    assert_eq!(run("[left.weight(), care, size, note, price(left)]"), serde_json::json!([7, "careful", "big", "pen+ink", 13]));
    assert_eq!(run("[wrapped, pending.length]"), serde_json::json!([["ink"], 2]));
    run("ink.fragile = false");
    run("ink.kg = 3");
    assert_eq!(run("[left.weight(), care, size]"), serde_json::json!([5, "plain", "small"]));
    run(r#"left.parts.push(Part("cap", 2, true, 1))"#);
    assert_eq!(run("[left.weight(), care, note, price(left)]"), serde_json::json!([7, "careful", "pen+ink+cap", 14]));
}

/// Nucleoid rolls back failed selector argument updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_selector_argument_updates() {
    runner()("def checked(total):\n    if total > 100:\n        throw \"TOO_MANY\"\n    return total\ncart = [{sku: \"pen\", qty: 2, price: 3}, {sku: \"ink\", qty: 5, price: 9}]\nunits = cart.sum(.qty)\nskus = cart.map(.sku)\ndearest = cart.maxBy(.price)\ncheapest = cart.minBy(.price)\ncap = checked(units)\nfailure = \"\"\ntry:\n    cart.push({sku: \"box\", qty: 500, price: 1})\ncatch error:\n    failure = error\nassert: [failure, units, skus, cap, cart.length], [\"TOO_MANY\", 7, [\"pen\", \"ink\"], 7, 2]\nsuppose:\n    cart.push({sku: \"cap\", qty: 30, price: 12})\n    assert: [units, skus, dearest.sku, cap], [37, [\"pen\", \"ink\", \"cap\"], \"cap\", 37]\nassert: [units, skus, dearest.sku, cheapest.sku, cart.length], [7, [\"pen\", \"ink\"], \"ink\", \"pen\", 2]\ncart.push({sku: \"pad\", qty: 9, price: 1})\nassert: [units, skus, dearest.sku, cheapest.sku, cap], [16, [\"pen\", \"ink\", \"pad\"], \"ink\", \"pad\", 16]");
    let mut run = runner();
    run(r#"def checked(total):
    if total > 100:
        throw "TOO_MANY"
    return total
cart = [{sku: "pen", qty: 2, price: 3}, {sku: "ink", qty: 5, price: 9}]
units = cart.sum(.qty)
skus = cart.map(.sku)
dearest = cart.maxBy(.price)
cheapest = cart.minBy(.price)
cap = checked(units)
failure = ""
try:
    cart.push({sku: "box", qty: 500, price: 1})
catch error:
    failure = error
assert: [failure, units, skus, cap, cart.length], ["TOO_MANY", 7, ["pen", "ink"], 7, 2]
suppose:
    cart.push({sku: "cap", qty: 30, price: 12})
    assert: [units, skus, dearest.sku, cap], [37, ["pen", "ink", "cap"], "cap", 37]
assert: [units, skus, dearest.sku, cheapest.sku, cart.length], [7, ["pen", "ink"], "ink", "pen", 2]
cart.push({sku: "pad", qty: 9, price: 1})
assert: [units, skus, dearest.sku, cheapest.sku, cap], [16, ["pen", "ink", "pad"], "ink", "pad", 16]"#);
}

/// Nucleoid tracks only the properties that selector arguments read
#[rustfmt::skip]
#[test]
fn tracks_only_the_properties_that_selector_arguments_read() {
    runner()("class Sensor(room, temp):\n    this.room = room\n    this.temp = temp\nhottest = Sensor.maxBy(.temp)\nrooms = Sensor.map(.room)\nlab = Sensor(\"lab\", 21)\nhall = Sensor(\"hall\", 25)\nreaders = (affects hall.temp).map(.node)\nlisters = (affects hall.room).map(.node)\nassert: [readers, listers, hottest, rooms], [[\"hottest\"], [\"rooms\"], hall, [\"lab\", \"hall\"]]\ncoolest = Sensor.minBy(.temp)\nassert: [readers, listers, coolest], [[\"hottest\", \"coolest\"], [\"rooms\"], lab]\nlab.temp = 30\nhall.room = \"foyer\"\nassert: [hottest, coolest, rooms], [lab, hall, [\"lab\", \"foyer\"]]");
    let mut run = runner();
    run(r#"class Sensor(room, temp):
    this.room = room
    this.temp = temp"#);
    run("hottest = Sensor.maxBy(.temp)");
    run("rooms = Sensor.map(.room)");
    run(r#"lab = Sensor("lab", 21)"#);
    run(r#"hall = Sensor("hall", 25)"#);
    run("readers = (affects hall.temp).map(.node)");
    run("listers = (affects hall.room).map(.node)");
    {
        let actual = run("[readers, listers, hottest, rooms]");
        let expected = run(r#"([["hottest"], ["rooms"], hall, ["lab", "hall"]])"#);
        assert_eq!(actual, expected);
    }
    run("coolest = Sensor.minBy(.temp)");
    {
        let actual = run("[readers, listers, coolest]");
        let expected = run(r#"([["hottest", "coolest"], ["rooms"], lab])"#);
        assert_eq!(actual, expected);
    }
    run("lab.temp = 30");
    run(r#"hall.room = "foyer""#);
    {
        let actual = run("[hottest, coolest, rooms]");
        let expected = run(r#"([lab, hall, ["lab", "foyer"]])"#);
        assert_eq!(actual, expected);
    }
}

/// Nucleoid averages lists with average
#[rustfmt::skip]
#[test]
fn averages_lists_with_average() {
    runner()("temps = [18, 24, 9]\nmean = temps.average()\ngap = temps.average(temp -> Math.abs(temp - 20))\nassert: [mean, gap, (why mean)[0].from], [17, 5.666666666666667, [\"temps\"]]\ntemps.push(29)\nassert: [mean, gap], [20, 6.5]\ntemps.pop()\ntemps.pop()\nassert: [mean, gap], [21, 3]\ntemps = []\nassert: [Number.isNaN(mean), Number.isNaN(gap)], [true, true]\ncart = {items: [0.1, 0.2]}\npaid = cart.items.average()\nsmall = cart.items.filter(item -> item < 0.25).average()\ncart.items.push(0.3)\nassert: [paid, small], [0.20000000000000004, 0.15000000000000002]");
    let mut run = runner();
    run("temps = [18, 24, 9]");
    run("mean = temps.average()");
    run("gap = temps.average(temp -> Math.abs(temp - 20))");
    assert_eq!(run("[mean, gap, (why mean)[0].from]"), serde_json::json!([17, 5.666666666666667, ["temps"]]));
    run("temps.push(29)");
    assert_eq!(run("[mean, gap]"), serde_json::json!([20, 6.5]));
    run("temps.pop()");
    run("temps.pop()");
    assert_eq!(run("[mean, gap]"), serde_json::json!([21, 3]));
    run("temps = []");
    assert_eq!(run("[Number.isNaN(mean), Number.isNaN(gap)]"), serde_json::json!([true, true]));
    run("cart = {items: [0.1, 0.2]}");
    run("paid = cart.items.average()");
    run("small = cart.items.filter(item -> item < 0.25).average()");
    run("cart.items.push(0.3)");
    assert_eq!(run("[paid, small]"), serde_json::json!([0.20000000000000004, 0.15000000000000002]));
}

/// Nucleoid averages class instance populations with average
#[rustfmt::skip]
#[test]
fn averages_class_instance_populations_with_average() {
    runner()("class Trip(km, hours):\n    this.km = km\n    this.hours = hours\ndef speed trip => trip.km / trip.hours\nfirst = Trip(120, 2)\nsecond = Trip(45, 1)\npace = Trip.average(speed)\nfar = Trip.filter(trip -> trip.km > 100).average(speed)\nrecent = Trip[-2:].average(trip -> trip.hours * 60)\nassert: [pace, far, recent], [52.5, 60, 90]\nthird = Trip(300, 4)\nassert: [pace, far, recent], [60, 67.5, 150]\nfirst.hours = 3\nassert: [pace, far], [53.333333333333336, 57.5]\ndelete third.hours\nassert: [pace, far, recent], [null, null, null]\ndelete third.km\ndelete third\nassert: [pace, far, recent], [42.5, 40, 120]\nclass Refund(cents): this.cents = cents\nrefunds = Refund.average(refund -> refund.cents / 100)\nassert: Number.isNaN(refunds), true\nRefund(250)\nassert: refunds, 2.5");
    let mut run = runner();
    run(r#"class Trip(km, hours):
    this.km = km
    this.hours = hours"#);
    run("def speed trip => trip.km / trip.hours");
    run("first = Trip(120, 2)");
    run("second = Trip(45, 1)");
    run("pace = Trip.average(speed)");
    run("far = Trip.filter(trip -> trip.km > 100).average(speed)");
    run("recent = Trip[-2:].average(trip -> trip.hours * 60)");
    assert_eq!(run("[pace, far, recent]"), serde_json::json!([52.5, 60, 90]));
    run("third = Trip(300, 4)");
    assert_eq!(run("[pace, far, recent]"), serde_json::json!([60, 67.5, 150]));
    run("first.hours = 3");
    assert_eq!(run("[pace, far]"), serde_json::json!([53.333333333333336, 57.5]));
    run("delete third.hours");
    assert_eq!(run("[pace, far, recent]"), serde_json::json!([null, null, null]));
    run("delete third.km");
    run("delete third");
    assert_eq!(run("[pace, far, recent]"), serde_json::json!([42.5, 40, 120]));
    run("class Refund(cents): this.cents = cents");
    run("refunds = Refund.average(refund -> refund.cents / 100)");
    assert_eq!(run("Number.isNaN(refunds)"), true);
    run("Refund(250)");
    assert_eq!(run("refunds"), 2.5);
}

/// Nucleoid converts averaged values to numbers
#[rustfmt::skip]
#[test]
fn converts_averaged_values_to_numbers() {
    runner()("flags = [true, false, true, true]\nshare = flags.average()\nentries = [2, \"3\", \" 4 \", \"\", null, [5], []]\nmean = entries.average()\nassert: [share, mean], [0.75, 2]\nflags.push(false)\nentries.push(\"x\")\nassert: [share, Number.isNaN(mean)], [0.6, true]\nentries.pop()\nentries.push(Number.POSITIVE_INFINITY)\nassert: mean, Number.POSITIVE_INFINITY\nclass Line(price, qty):\n    this.price = price\n    this.qty = qty\npen = Line(3, 2)\nbook = Line(12, 1)\ntypical = Line.average(line -> line.price * line.qty)\nassert: typical, 9\ngift = Line(null, 1)\nassert: typical, null\ngift.price = 6\nassert: typical, 8");
    let mut run = runner();
    run("flags = [true, false, true, true]");
    run("share = flags.average()");
    run(r#"entries = [2, "3", " 4 ", "", null, [5], []]"#);
    run("mean = entries.average()");
    assert_eq!(run("[share, mean]"), serde_json::json!([0.75, 2]));
    run("flags.push(false)");
    run(r#"entries.push("x")"#);
    assert_eq!(run("[share, Number.isNaN(mean)]"), serde_json::json!([0.6, true]));
    run("entries.pop()");
    run("entries.push(Number.POSITIVE_INFINITY)");
    {
        let actual = run("mean");
        let expected = run("(Number.POSITIVE_INFINITY)");
        assert_eq!(actual, expected);
    }
    run(r#"class Line(price, qty):
    this.price = price
    this.qty = qty"#);
    run("pen = Line(3, 2)");
    run("book = Line(12, 1)");
    run("typical = Line.average(line -> line.price * line.qty)");
    assert_eq!(run("typical"), 9);
    run("gift = Line(null, 1)");
    assert_eq!(run("typical"), serde_json::Value::Null);
    run("gift.price = 6");
    assert_eq!(run("typical"), 8);
}

/// Nucleoid calls the average selector once per item with only the item
#[rustfmt::skip]
#[test]
fn calls_the_average_selector_once_per_item_with_only_the_item() {
    runner()("seen = []\ndef weigh(item, position):\n    seen.push(position ?? item)\n    return item * item\nitems = [3, 1, 2]\ndef spread() => items.average(weigh)\nassert: [spread(), seen], [4.666666666666667, [3, 1, 2]]\nfloor = 2\ndef price item => Math.max(item, floor)\ncost = items.average(price)\nassert: cost, 2.3333333333333335\nfloor = 3\nitems.push(5)\nassert: [cost, seen], [3.5, [3, 1, 2]]\nmissing = null\nskipped = missing.average(weigh(9))\nassert: [skipped, seen], [null, [3, 1, 2]]");
    let mut run = runner();
    run("seen = []");
    run(r#"def weigh(item, position):
    seen.push(position ?? item)
    return item * item"#);
    run("items = [3, 1, 2]");
    run("def spread() => items.average(weigh)");
    assert_eq!(run("[spread(), seen]"), serde_json::json!([4.666666666666667, [3, 1, 2]]));
    run("floor = 2");
    run("def price item => Math.max(item, floor)");
    run("cost = items.average(price)");
    assert_eq!(run("cost"), 2.3333333333333335);
    run("floor = 3");
    run("items.push(5)");
    assert_eq!(run("[cost, seen]"), serde_json::json!([3.5, [3, 1, 2]]));
    run("missing = null");
    run("skipped = missing.average(weigh(9))");
    assert_eq!(run("[skipped, seen]"), serde_json::json!([null, [3, 1, 2]]));
}

/// Nucleoid keeps averages live in class level rules
#[rustfmt::skip]
#[test]
fn keeps_averages_live_in_class_level_rules() {
    runner()("class Room(name): this.name = name\nclass Sensor(room, temp):\n    this.room = room\n    this.temp = temp\n$Room.mean = Sensor.filter(sensor -> sensor.room == $Room).average(sensor -> sensor.temp)\n$Room.warm = $Room.mean > 21\nlab = Room(\"lab\")\nhall = Room(\"hall\")\nSensor(lab, 20)\nprobe = Sensor(lab, 25)\nSensor(hall, 18)\ncomfort = Room.average(room -> Math.abs(room.mean - 21))\nassert: [lab.mean, lab.warm, hall.mean, hall.warm, comfort], [22.5, true, 18, false, 2.25]\nprobe.temp = 21\nassert: [lab.mean, lab.warm, comfort], [20.5, false, 1.75]\nprobe.room = hall\nassert: [lab.mean, hall.mean, comfort], [20, 19.5, 1.25]\nattic = Room(\"attic\")\nassert: [Number.isNaN(attic.mean), attic.warm, Number.isNaN(comfort)], [true, false, true]");
    let mut run = runner();
    run("class Room(name): this.name = name");
    run(r#"class Sensor(room, temp):
    this.room = room
    this.temp = temp"#);
    run("$Room.mean = Sensor.filter(sensor -> sensor.room == $Room).average(sensor -> sensor.temp)");
    run("$Room.warm = $Room.mean > 21");
    run(r#"lab = Room("lab")"#);
    run(r#"hall = Room("hall")"#);
    run("Sensor(lab, 20)");
    run("probe = Sensor(lab, 25)");
    run("Sensor(hall, 18)");
    run("comfort = Room.average(room -> Math.abs(room.mean - 21))");
    assert_eq!(run("[lab.mean, lab.warm, hall.mean, hall.warm, comfort]"), serde_json::json!([22.5, true, 18, false, 2.25]));
    run("probe.temp = 21");
    assert_eq!(run("[lab.mean, lab.warm, comfort]"), serde_json::json!([20.5, false, 1.75]));
    run("probe.room = hall");
    assert_eq!(run("[lab.mean, hall.mean, comfort]"), serde_json::json!([20, 19.5, 1.25]));
    run(r#"attic = Room("attic")"#);
    assert_eq!(run("[Number.isNaN(attic.mean), attic.warm, Number.isNaN(comfort)]"), serde_json::json!([true, false, true]));
}

/// Nucleoid uses averages in functions methods and conditional expressions
#[rustfmt::skip]
#[test]
fn uses_averages_in_functions_methods_and_conditional_expressions() {
    runner()("class Team(name, scores):\n    this.name = name\n    this.scores = scores\n    def mean(): return this.scores.average()\ndef variance(team) => team.scores.average(score -> score * score) - team.mean() ** 2\nred = Team(\"red\", [3, 5])\nblue = Team(\"blue\", [6])\nleague = Team.average(team -> team.mean())\nleader = red.mean() > blue.mean() ? red.name : blue.name\nspread = variance(red)\nassert: [league, leader, spread], [5, \"blue\", 1]\nblue.scores.push(2)\nred.scores.push(7)\nTeam(\"green\", [1])\nassert: [league, leader, spread], [3.3333333333333335, \"red\", 2.666666666666668]");
    let mut run = runner();
    run(r#"class Team(name, scores):
    this.name = name
    this.scores = scores
    def mean(): return this.scores.average()"#);
    run("def variance(team) => team.scores.average(score -> score * score) - team.mean() ** 2");
    run(r#"red = Team("red", [3, 5])"#);
    run(r#"blue = Team("blue", [6])"#);
    run("league = Team.average(team -> team.mean())");
    run("leader = red.mean() > blue.mean() ? red.name : blue.name");
    run("spread = variance(red)");
    assert_eq!(run("[league, leader, spread]"), serde_json::json!([5, "blue", 1]));
    run("blue.scores.push(2)");
    run("red.scores.push(7)");
    run(r#"Team("green", [1])"#);
    assert_eq!(run("[league, leader, spread]"), serde_json::json!([3.3333333333333335, "red", 2.666666666666668]));
}

/// Nucleoid keeps average as an ordinary name beside the average query
#[rustfmt::skip]
#[test]
fn keeps_average_as_an_ordinary_name_beside_the_average_query() {
    runner()("average = 2\nreport = {average: 5}\ndef double average => average * 2\nclass Bag(items):\n    this.items = items\n    def average(): return this.items.average(item -> item * item)\nbag = Bag([1, 3])\nlevel = bag.items.average() + average + report.average\nbags = Bag.average(each -> each.average())\nassert: [level, bag.average(), bags, double(average)], [9, 5, 5, 4]\nassert: bag.items.average, null\nbag.items.push(5)\naverage = 4\nBag([6])\nassert: [level, bags], [12, 23.833333333333332]");
    let mut run = runner();
    run("average = 2");
    run("report = {average: 5}");
    run("def double average => average * 2");
    run(r#"class Bag(items):
    this.items = items
    def average(): return this.items.average(item -> item * item)"#);
    run("bag = Bag([1, 3])");
    run("level = bag.items.average() + average + report.average");
    run("bags = Bag.average(each -> each.average())");
    assert_eq!(run("[level, bag.average(), bags, double(average)]"), serde_json::json!([9, 5, 5, 4]));
    assert_eq!(run("bag.items.average"), serde_json::Value::Null);
    run("bag.items.push(5)");
    run("average = 4");
    run("Bag([6])");
    assert_eq!(run("[level, bags]"), serde_json::json!([12, 23.833333333333332]));
}

/// Nucleoid rolls back failed average updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_average_updates() {
    runner()("def checked(value):\n    if value > 100:\n        throw \"TOO_LARGE\"\n    return value\namounts = [10, 20]\nmean = amounts.average(checked)\nfailure = \"\"\ntry:\n    amounts.push(500)\ncatch error:\n    failure = error\nassert: [failure, amounts, mean], [\"TOO_LARGE\", [10, 20], 15]\namounts.push(30)\nassert: mean, 20\nclass Item(price): this.price = price\npen = Item(3)\ntypical = Item.average(item -> checked(item.price))\nfailure = \"\"\ntry:\n    Item(500)\ncatch error:\n    failure = error\nassert: [failure, Item.length, typical], [\"TOO_LARGE\", 1, 3]\nsuppose:\n    Item(5)\n    assert: typical, 4\nassert: [Item.length, typical], [1, 3]");
    let mut run = runner();
    run(r#"def checked(value):
    if value > 100:
        throw "TOO_LARGE"
    return value
amounts = [10, 20]
mean = amounts.average(checked)
failure = ""
try:
    amounts.push(500)
catch error:
    failure = error
assert: [failure, amounts, mean], ["TOO_LARGE", [10, 20], 15]
amounts.push(30)
assert: mean, 20
class Item(price): this.price = price
pen = Item(3)
typical = Item.average(item -> checked(item.price))
failure = ""
try:
    Item(500)
catch error:
    failure = error
assert: [failure, Item.length, typical], ["TOO_LARGE", 1, 3]
suppose:
    Item(5)
    assert: typical, 4
assert: [Item.length, typical], [1, 3]"#);
}

/// Nucleoid copies object entries into records with spread entries
#[rustfmt::skip]
#[test]
fn copies_object_entries_into_records_with_spread_entries() {
    runner()("base = {name: \"tea\", price: 4, stock: 10}\nsale = {...base, price: 3, tag: \"sale\"}\nfloor = {price: 9, ...base}\nassert: [sale, floor], [{name: \"tea\", price: 3, stock: 10, tag: \"sale\"}, {price: 4, name: \"tea\", stock: 10}]\nassert: [sale == base, floor == base, base.tag], [false, false, null]\nbase.name = \"oolong\"\nbase.price = 6\nbase.size = 2\ndelete base.stock\nassert: [sale, floor], [{name: \"oolong\", price: 3, size: 2, tag: \"sale\"}, {price: 6, name: \"oolong\", size: 2}]\ntag = \"new\"\nfresh = {tag, ...base price: 5,}\nassert: fresh, {tag: \"new\", name: \"oolong\", price: 5, size: 2}");
    let mut run = runner();
    run(r#"base = {name: "tea", price: 4, stock: 10}"#);
    run(r#"sale = {...base, price: 3, tag: "sale"}"#);
    run("floor = {price: 9, ...base}");
    {
        let actual = run("[sale, floor]");
        let expected = run(r#"([{name: "tea", price: 3, stock: 10, tag: "sale"}, {price: 4, name: "tea", stock: 10}])"#);
        assert_eq!(actual, expected);
    }
    assert_eq!(run("[sale == base, floor == base, base.tag]"), serde_json::json!([false, false, null]));
    run(r#"base.name = "oolong""#);
    run("base.price = 6");
    run("base.size = 2");
    run("delete base.stock");
    {
        let actual = run("[sale, floor]");
        let expected = run(r#"([{name: "oolong", price: 3, size: 2, tag: "sale"}, {price: 6, name: "oolong", size: 2}])"#);
        assert_eq!(actual, expected);
    }
    run(r#"tag = "new""#);
    run("fresh = {tag, ...base price: 5,}");
    {
        let actual = run("fresh");
        let expected = run(r#"({tag: "new", name: "oolong", price: 5, size: 2})"#);
        assert_eq!(actual, expected);
    }
}

/// Nucleoid keeps spread entries live as their sources change
#[rustfmt::skip]
#[test]
fn keeps_spread_entries_live_as_their_sources_change() {
    runner()("base = {name: \"tea\", price: 4}\nitem = {...base, qty: 2}\nline = {...item, total: item.price * item.qty}\nassert: line, {name: \"tea\", price: 4, qty: 2, total: 8}\nbase.price = 5\nbase.size = \"large\"\ndelete base.name\nassert: line, {price: 5, size: \"large\", qty: 2, total: 10}\nbase = {name: \"jam\", price: 3}\nassert: [item, line], [{name: \"jam\", price: 3, qty: 2}, {name: \"jam\", price: 3, qty: 2, total: 6}]");
    let mut run = runner();
    run(r#"base = {name: "tea", price: 4}"#);
    run("item = {...base, qty: 2}");
    run("line = {...item, total: item.price * item.qty}");
    {
        let actual = run("line");
        let expected = run(r#"({name: "tea", price: 4, qty: 2, total: 8})"#);
        assert_eq!(actual, expected);
    }
    run("base.price = 5");
    run(r#"base.size = "large""#);
    run("delete base.name");
    {
        let actual = run("line");
        let expected = run(r#"({price: 5, size: "large", qty: 2, total: 10})"#);
        assert_eq!(actual, expected);
    }
    run(r#"base = {name: "jam", price: 3}"#);
    {
        let actual = run("[item, line]");
        let expected = run(r#"([{name: "jam", price: 3, qty: 2}, {name: "jam", price: 3, qty: 2, total: 6}])"#);
        assert_eq!(actual, expected);
    }
}

/// Nucleoid merges records with several spread entries
#[rustfmt::skip]
#[test]
fn merges_records_with_several_spread_entries() {
    runner()("defaults = {color: \"red\", size: 2, gift: false}\ncustom = {size: 3}\norder = {...defaults, ...custom, id: 7}\nassert: order, {color: \"red\", size: 3, gift: false, id: 7}\ncustom.gift = true\ncustom.note = \"rush\"\nassert: order, {color: \"red\", size: 3, gift: true, note: \"rush\", id: 7}\ndelete custom.size\ndefaults.color = \"blue\"\nassert: order, {color: \"blue\", size: 2, gift: true, note: \"rush\", id: 7}");
    let mut run = runner();
    run(r#"defaults = {color: "red", size: 2, gift: false}"#);
    run("custom = {size: 3}");
    run("order = {...defaults, ...custom, id: 7}");
    {
        let actual = run("order");
        let expected = run(r#"({color: "red", size: 3, gift: false, id: 7})"#);
        assert_eq!(actual, expected);
    }
    run("custom.gift = true");
    run(r#"custom.note = "rush""#);
    {
        let actual = run("order");
        let expected = run(r#"({color: "red", size: 3, gift: true, note: "rush", id: 7})"#);
        assert_eq!(actual, expected);
    }
    run("delete custom.size");
    run(r#"defaults.color = "blue""#);
    {
        let actual = run("order");
        let expected = run(r#"({color: "blue", size: 2, gift: true, note: "rush", id: 7})"#);
        assert_eq!(actual, expected);
    }
}

/// Nucleoid spreads instances into records and class level rules
#[rustfmt::skip]
#[test]
fn spreads_instances_into_records_and_class_level_rules() {
    runner()("class Room(name, temp):\n    this.name = name\n    this.temp = temp\n$Room.card = {...$Room, unit: \"C\"}\nlab = Room(\"lab\", 20)\nlab.temp = 25\nlab.floor = 2\nassert: lab.card, {id: \"lab\", name: \"lab\", temp: 25, floor: 2, unit: \"C\"}\nhall = Room(\"hall\", 18)\ncopy = {...hall, temp: 19}\nassert: (why copy)[0].from, [\"hall\", \"hall.id\", \"hall.name\", \"hall.card\"]\nhall.name = \"foyer\"\nassert: copy, {id: \"hall\", name: \"foyer\", temp: 19, card: {id: \"hall\", name: \"foyer\", temp: 18, unit: \"C\"}}");
    let mut run = runner();
    run(r#"class Room(name, temp):
    this.name = name
    this.temp = temp"#);
    run(r#"$Room.card = {...$Room, unit: "C"}"#);
    run(r#"lab = Room("lab", 20)"#);
    run("lab.temp = 25");
    run("lab.floor = 2");
    {
        let actual = run("lab.card");
        let expected = run(r#"({id: "lab", name: "lab", temp: 25, floor: 2, unit: "C"})"#);
        assert_eq!(actual, expected);
    }
    run(r#"hall = Room("hall", 18)"#);
    run("copy = {...hall, temp: 19}");
    assert_eq!(run("(why copy)[0].from"), serde_json::json!(["hall", "hall.id", "hall.name", "hall.card"]));
    run(r#"hall.name = "foyer""#);
    {
        let actual = run("copy");
        let expected = run(r#"({id: "hall", name: "foyer", temp: 19, card: {id: "hall", name: "foyer", temp: 18, unit: "C"}})"#);
        assert_eq!(actual, expected);
    }
}

/// Nucleoid spreads entries in functions methods and lambdas
#[rustfmt::skip]
#[test]
fn spreads_entries_in_functions_methods_and_lambdas() {
    runner()("class Point(x, y):\n    this.x = x\n    this.y = y\n    def shifted dx => {...this, x: this.x + dx}\ndef flat p => {...p, z: 0}\np = Point(1, 2)\nq = Point(3, 4)\nup = flat(p)\nfar = p.shifted(10)\ngrid = [p, q].map(lambda pt: {...pt, area: pt.x * pt.y})\nassert: [up, far], [{id: \"p\", x: 1, y: 2, z: 0}, {id: \"p\", x: 11, y: 2}]\np.y = 5\nassert: [up, far, grid], [{id: \"p\", x: 1, y: 5, z: 0}, {id: \"p\", x: 11, y: 5}, [{id: \"p\", x: 1, y: 5, area: 5}, {id: \"q\", x: 3, y: 4, area: 12}]]");
    let mut run = runner();
    run(r#"class Point(x, y):
    this.x = x
    this.y = y
    def shifted dx => {...this, x: this.x + dx}"#);
    run("def flat p => {...p, z: 0}");
    run("p = Point(1, 2)");
    run("q = Point(3, 4)");
    run("up = flat(p)");
    run("far = p.shifted(10)");
    run("grid = [p, q].map(lambda pt: {...pt, area: pt.x * pt.y})");
    {
        let actual = run("[up, far]");
        let expected = run(r#"([{id: "p", x: 1, y: 2, z: 0}, {id: "p", x: 11, y: 2}])"#);
        assert_eq!(actual, expected);
    }
    run("p.y = 5");
    {
        let actual = run("[up, far, grid]");
        let expected = run(r#"([{id: "p", x: 1, y: 5, z: 0}, {id: "p", x: 11, y: 5}, [{id: "p", x: 1, y: 5, area: 5}, {id: "q", x: 3, y: 4, area: 12}]])"#);
        assert_eq!(actual, expected);
    }
}

/// Nucleoid spreads only objects and evaluates each source once
#[rustfmt::skip]
#[test]
fn spreads_only_objects_and_evaluates_each_source_once() {
    runner()("owner = null\ncard = {...owner, seen: true}\nassert: card, null\nowner = {name: \"ada\", role: \"dev\"}\nassert: card, {name: \"ada\", role: \"dev\", seen: true}\nbox = {}\npacked = {...box.lid, open: false}\nassert: packed, {open: false}\nbox.lid = {color: \"tan\", size: 3}\nassert: packed, {color: \"tan\", size: 3, open: false}\nclass Tag(label, uses):\n    this.label = label\n    this.uses = uses\nmade = {...Tag(\"new\", 0), qty: 1}\nassert: [made.label, made.uses, made.qty, Tag.length], [\"new\", 0, 1, 1]\nsizes = [1, 2]\ntry:\n    broken = {...sizes}\ncatch error:\n    assert: error, TypeError(\"sizes is not an object\")");
    let mut run = runner();
    run(r#"owner = null
card = {...owner, seen: true}
assert: card, null
owner = {name: "ada", role: "dev"}
assert: card, {name: "ada", role: "dev", seen: true}
box = {}
packed = {...box.lid, open: false}
assert: packed, {open: false}
box.lid = {color: "tan", size: 3}
assert: packed, {color: "tan", size: 3, open: false}
class Tag(label, uses):
    this.label = label
    this.uses = uses
made = {...Tag("new", 0), qty: 1}
assert: [made.label, made.uses, made.qty, Tag.length], ["new", 0, 1, 1]
sizes = [1, 2]
try:
    broken = {...sizes}
catch error:
    assert: error, TypeError("sizes is not an object")"#);
}

/// Nucleoid leaves the declared property out of spreads of its own object
#[rustfmt::skip]
#[test]
fn leaves_the_declared_property_out_of_spreads_of_its_own_object() {
    runner()("class Order(total, items):\n    this.total = total\n    this.items = items\n$Order.view = {...$Order, paid: false}\norder = Order(10, 2)\norder.total = 12\norder.total = 14\nassert: order.view, {id: \"order\", total: 14, items: 2, paid: false}\ncart = {tea: 4, jam: 2}\ncart.copy = {...cart, n: 1}\ncart.tea = 5\ncart.oat = 1\nassert: cart.copy, {tea: 5, jam: 2, oat: 1, n: 1}");
    let mut run = runner();
    run(r#"class Order(total, items):
    this.total = total
    this.items = items"#);
    run("$Order.view = {...$Order, paid: false}");
    run("order = Order(10, 2)");
    run("order.total = 12");
    run("order.total = 14");
    {
        let actual = run("order.view");
        let expected = run(r#"({id: "order", total: 14, items: 2, paid: false})"#);
        assert_eq!(actual, expected);
    }
    run("cart = {tea: 4, jam: 2}");
    run("cart.copy = {...cart, n: 1}");
    run("cart.tea = 5");
    run("cart.oat = 1");
    {
        let actual = run("cart.copy");
        let expected = run("({tea: 5, jam: 2, oat: 1, n: 1})");
        assert_eq!(actual, expected);
    }
}

/// Nucleoid rolls back failed spread updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_spread_updates() {
    runner()("def checked(count):\n    if count > 9:\n        throw \"TOO_MANY\"\n    return count\ncart = {tea: 4, jam: 3}\norder = {...cart, count: checked(cart.tea)}\nfailure = \"\"\ntry:\n    cart.tea = 50\ncatch error:\n    failure = error\nassert: [failure, cart.tea, order], [\"TOO_MANY\", 4, {tea: 4, jam: 3, count: 4}]\ntry:\n    cart.oats = 2\n    delete cart.jam\n    assert: order, {tea: 4, oats: 2, count: 4}\n    throw \"DECLINED\"\ncatch error:\n    pass\nassert: order, {tea: 4, jam: 3, count: 4}\nplan = suppose:\n    cart.rye = 5\n    order.rye\nassert: [plan, order.rye], [5, null]\ncart.rye = 1\nassert: order, {tea: 4, jam: 3, rye: 1, count: 4}");
    let mut run = runner();
    run(r#"def checked(count):
    if count > 9:
        throw "TOO_MANY"
    return count
cart = {tea: 4, jam: 3}
order = {...cart, count: checked(cart.tea)}
failure = ""
try:
    cart.tea = 50
catch error:
    failure = error
assert: [failure, cart.tea, order], ["TOO_MANY", 4, {tea: 4, jam: 3, count: 4}]
try:
    cart.oats = 2
    delete cart.jam
    assert: order, {tea: 4, oats: 2, count: 4}
    throw "DECLINED"
catch error:
    pass
assert: order, {tea: 4, jam: 3, count: 4}
plan = suppose:
    cart.rye = 5
    order.rye
assert: [plan, order.rye], [5, null]
cart.rye = 1
assert: order, {tea: 4, jam: 3, rye: 1, count: 4}"#);
}

/// Nucleoid narrows explanations to a reasoning depth
#[rustfmt::skip]
#[test]
fn narrows_explanations_to_a_reasoning_depth() {
    let mut run = runner();
    run("base = 1");
    run("seed = base + 1");
    run("step = seed + 1");
    run("next = step * 2");
    run("result = next + step");
    run("own = result |> why |> depth(0)");
    run("near = result |> why |> depth(1)");
    run("wide = result |> why |> depth(2)");
    assert_eq!(run("own.map(s -> s.node)"), serde_json::json!(["result"]));
    assert_eq!(run("near.map(s -> s.node)"), serde_json::json!(["result", "next", "step"]));
    assert_eq!(run("near[0].from"), serde_json::json!(["next", "step"]));
    assert_eq!(run("wide.map(s -> s.node)"), serde_json::json!(["result", "next", "step", "seed"]));
    run("base = 5");
    run("step = base * 3");
    assert_eq!(run("near.map(s -> s.holds)"), serde_json::json!([45, 30, 15]));
    assert_eq!(run("wide.map(s -> s.node)"), serde_json::json!(["result", "next", "step", "base"]));
    run("result = 7");
    assert_eq!(run("own.map(s -> s.state)"), serde_json::json!(["stated"]));
}

/// Nucleoid measures reasoning depth along the shortest path
#[rustfmt::skip]
#[test]
fn measures_reasoning_depth_along_the_shortest_path() {
    let mut run = runner();
    run("rate = 2");
    run("fee = rate + 1");
    run("charge = fee * 2");
    run("base = rate * 3");
    run("total = base + charge");
    run("two = total |> why |> depth(2)");
    assert_eq!(run("two.map(s -> s.node)"), serde_json::json!(["total", "base", "charge", "fee", "rate"]));
    run("base = 10");
    assert_eq!(run("two.map(s -> s.node)"), serde_json::json!(["total", "base", "charge", "fee"]));
    assert_eq!(run("(why total).length"), 5);
    run("rate = 4");
    assert_eq!(run("two.map(s -> s.holds)"), serde_json::json!([20, 10, 10, 5]));
}

/// Nucleoid narrows impact selections to direct dependents
#[rustfmt::skip]
#[test]
fn narrows_impact_selections_to_direct_dependents() {
    let mut run = runner();
    run("price = 10");
    run("tax = price * 2");
    run("total = price + tax");
    run("label = total * 3");
    run("direct = price |> affects |> depth(1)");
    run("again = tax |> why |> affects |> depth(1)");
    assert_eq!(run("(affects price).map(s -> s.node)"), serde_json::json!(["tax", "total", "label"]));
    assert_eq!(run("direct.map(s -> s.node)"), serde_json::json!(["tax", "total"]));
    assert_eq!(run("again.map(s -> s.node)"), serde_json::json!(["tax", "total"]));
    run("label = price + 1");
    assert_eq!(run("direct.map(s -> s.node)"), serde_json::json!(["tax", "total", "label"]));
    assert_eq!(run("again.map(s -> s.node)"), serde_json::json!(["tax", "total", "label"]));
    run("price = 4");
    assert_eq!(run("direct.map(s -> s.holds)"), serde_json::json!([8, 12, 5]));
}

/// Nucleoid keeps depth selections live as limits and rules change
#[rustfmt::skip]
#[test]
fn keeps_depth_selections_live_as_limits_and_rules_change() {
    let mut run = runner();
    run("base = 1");
    run("mid = base + 1");
    run("top = mid * 2");
    run("limit = 1");
    run("near = top |> why |> depth(limit)");
    assert_eq!(run("near.map(s -> s.node)"), serde_json::json!(["top", "mid"]));
    run("limit = 2");
    assert_eq!(run("near.map(s -> s.node)"), serde_json::json!(["top", "mid", "base"]));
    run("extra = 5");
    run("mid = extra + base");
    assert_eq!(run("near.map(s -> s.node)"), serde_json::json!(["top", "mid", "extra", "base"]));
    assert_eq!(run("near[0].holds"), 12);
    run("limit = 0");
    assert_eq!(run("near.map(s -> s.node)"), serde_json::json!(["top"]));
    run("top = 3");
    run("limit = 1");
    assert_eq!(run("near.map(s -> s.state)"), serde_json::json!(["stated"]));
}

/// Nucleoid composes depth with state filters and later walks
#[rustfmt::skip]
#[test]
fn composes_depth_with_state_filters_and_later_walks() {
    let mut run = runner();
    run("base = 2");
    run("rate = base * 2");
    run("fee = 1");
    run("cost = rate * 4");
    run("price = cost + fee");
    run("facts = price |> why |> depth(2) |> stated");
    run("touched = price |> why |> stated |> affects |> depth(1)");
    run("alone = price |> depth(2)");
    assert_eq!(run("facts.map(s -> s.node)"), serde_json::json!(["fee"]));
    assert_eq!(run("touched.map(s -> s.node)"), serde_json::json!(["rate", "price"]));
    assert_eq!(run("alone.map(s -> s.node)"), serde_json::json!(["price"]));
    run("fee = base + 1");
    assert_eq!(run("facts.map(s -> s.node)"), serde_json::json!(["base"]));
    assert_eq!(run("touched.map(s -> s.node)"), serde_json::json!(["rate", "fee"]));
    run("price = 9");
    assert_eq!(run("alone.map(s -> s.state)"), serde_json::json!(["stated"]));
}

/// Nucleoid waits for unknown depth limits
#[rustfmt::skip]
#[test]
fn waits_for_unknown_depth_limits() {
    let mut run = runner();
    run("cause = 4");
    run("effect = cause * 2");
    run("class Plan");
    run("plan = Plan()");
    run("known = null");
    run("pending = effect |> why |> depth(plan.depth)");
    run("waiting = effect |> why |> depth(known)");
    run("ready = all s in waiting: s.holds > 0");
    assert_eq!(run("pending"), serde_json::Value::Null);
    assert_eq!(run("waiting"), serde_json::Value::Null);
    assert_eq!(run("ready"), serde_json::Value::Null);
    run("plan.depth = 0");
    run("known = 1");
    assert_eq!(run("pending.map(s -> s.node)"), serde_json::json!(["effect"]));
    assert_eq!(run("waiting.map(s -> s.node)"), serde_json::json!(["effect", "cause"]));
    assert_eq!(run("ready"), true);
    run("cause = -1");
    assert_eq!(run("ready"), false);
}

/// Nucleoid applies depth in class level rules methods and quantifiers
#[rustfmt::skip]
#[test]
fn applies_depth_in_class_level_rules_methods_and_quantifiers() {
    let mut run = runner();
    run(r#"class Order(qty, price):
    this.qty = qty
    this.price = price
    def direct() => this.total |> why |> depth(1)"#);
    run("$Order.units = $Order.qty * 10");
    run("$Order.total = $Order.units * $Order.price");
    run("$Order.causes = $Order.total |> why |> depth(1)");
    run("pen = Order(2, 3)");
    run("ok = all s in pen.total |> why |> depth(1): s.holds > 5");
    assert_eq!(run("pen.causes.map(s -> s.node)"), serde_json::json!(["pen.total", "pen.units", "pen.price"]));
    assert_eq!(run("pen.direct().map(s -> s.holds)"), serde_json::json!([60, 20, 3]));
    assert_eq!(run("ok"), false);
    run("pen.price = 7");
    assert_eq!(run("pen.causes[0].holds"), 140);
    assert_eq!(run("ok"), true);
    run("cup = Order(1, 9)");
    assert_eq!(run("cup.causes.map(s -> s.holds)"), serde_json::json!([90, 10, 9]));
}

/// Nucleoid restores depth selections after failed changes
#[rustfmt::skip]
#[test]
fn restores_depth_selections_after_failed_changes() {
    let mut run = runner();
    run(r#"seed = 1
mid = seed + 1
top = mid * 2
limit = 1
near = top |> why |> depth(limit)
frozen = near.value
try:
    limit = 2
    mid = 5
    throw "UNDO"
catch error:
    assert(error, "UNDO")
assert(near.map(s -> s.node), ["top", "mid"])
plan = suppose:
    limit = 2
    near.length
assert(plan, 3)
assert(near.length, 2)
limit = 2
seed = 4
assert(near.map(s -> s.holds), [10, 5, 4])
assert(frozen.map(s -> s.holds), [4, 2])"#);
}

/// Nucleoid lists what a supposed premise changes
#[rustfmt::skip]
#[test]
fn lists_what_a_supposed_premise_changes() {
    let mut run = runner();
    run(r#"threshold = 90
reading = 40
alarm = reading > threshold
level = reading > 10
suppose reading = 95:
    moved = model |> diff
    assert(moved.map(s -> s.node), ["reading", "alarm"])
    assert(moved.map(s -> s.holds), [95, true])
    assert(moved[1].from, ["reading", "threshold"])
    threshold = 99
    assert(moved.map(s -> s.node), ["threshold", "reading"])
assert(alarm, false)"#);
}

/// Nucleoid lists what changed in an explanation or an impact
#[rustfmt::skip]
#[test]
fn lists_what_changed_in_an_explanation_or_an_impact() {
    let mut run = runner();
    run(r#"base = 10
rate = 2
fee = 3
cost = base * rate
total = cost + fee
label = total > 40
charge = fee * 2
suppose rate = 3:
    causes = total |> why |> diff
    effects = base |> affects |> diff
    reach = model |> diff |> affects
    assert(causes.map(s -> s.node), ["total", "cost", "rate"])
    assert(effects.map(s -> s.node), ["cost", "total"])
    assert(reach.map(s -> s.node).sort(), ["cost", "label", "total"])
    fee = 30
    assert(causes.map(s -> s.node), ["total", "cost", "fee", "rate"])
    assert(effects.map(s -> s.node), ["cost", "total", "label"])
    assert(reach.map(s -> s.node).sort(), ["charge", "cost", "label", "total"])"#);
}

/// Nucleoid compares instances and class level rules in a hypothetical
#[rustfmt::skip]
#[test]
fn compares_instances_and_class_level_rules_in_a_hypothetical() {
    let mut run = runner();
    run(r#"class Item
$Item.premium = $Item.price > 10
pen = Item()
pen.price = 5
book = Item()
book.price = 20
suppose pen.price = 20:
    moved = model |> diff
    assert(moved.map(s -> s.node), ["pen.premium", "pen.price"])
    cup = Item()
    cup.price = 3
    assert(moved.map(s -> s.node), ["pen.premium", "pen.price", "cup", "cup.premium", "cup.price"])
    $Item.premium = $Item.price > 25
    assert(book.premium, false)
    assert(moved.map(s -> s.node), ["pen.price", "book.premium", "cup", "cup.premium", "cup.price"])
assert(book.premium, true)"#);
}

/// Nucleoid compares lists and objects in a hypothetical
#[rustfmt::skip]
#[test]
fn compares_lists_and_objects_in_a_hypothetical() {
    let mut run = runner();
    run(r#"tags = ["pen"]
count = tags.length
limits = {max: 3}
full = count >= limits.max
suppose tags.push("ink"):
    moved = model |> diff
    assert(moved.map(s -> s.node), ["tags", "count"])
    assert(moved[0].holds, ["pen", "ink"])
    tags.pop()
    assert(moved, [])
    limits = {max: 3}
    assert(moved.map(s -> s.node), ["limits"])
assert(tags, ["pen"])"#);
}

/// Nucleoid leaves equal values and undone writes out of a diff
#[rustfmt::skip]
#[test]
fn leaves_equal_values_and_undone_writes_out_of_a_diff() {
    let mut run = runner();
    run(r#"a = 1
b = a + 1
c = b * 2
suppose a = 1:
    moved = model |> diff
    assert(moved, [])
    b = 1 + a
    delete c
    c = b * 2
    assert(moved, [])
    try:
        a = 5
        throw "UNDO"
    catch error:
        assert(error, "UNDO")
    assert(moved, [])
    a = 3
    assert(moved.map(s -> s.node), ["a", "b", "c"])
    assert(moved[1].rule, "b = 1+a")"#);
}

/// Nucleoid compares nested hypotheticals with the innermost block
#[rustfmt::skip]
#[test]
fn compares_nested_hypotheticals_with_the_innermost_block() {
    let mut run = runner();
    run(r#"price = 10
qty = 2
total = price * qty
suppose price = 12:
    outer = model |> diff
    assert(outer.map(s -> s.node), ["price", "total"])
    inner = suppose qty = 3:
        nested = model |> diff
        assert(nested.map(s -> s.node), ["qty", "total"])
        price = 10
        assert(nested.map(s -> s.node), ["price", "qty", "total"])
        nested.map(s -> s.holds)
    assert(inner, [10, 3, 30])
    assert(outer.map(s -> s.node), ["price", "total", "inner"])
assert(total, 20)"#);
}

/// Nucleoid finds hypothetical changes in functions methods and quantifiers
#[rustfmt::skip]
#[test]
fn finds_hypothetical_changes_in_functions_methods_and_quantifiers() {
    let mut run = runner();
    run(r#"class Tank(level):
    this.level = level
    def changed() => (model |> diff).map(s -> s.node)
low = 10
tank = Tank(5)
empty = tank.level < low
def touches(name) => any s in model |> diff: s.node == name
suppose tank.level = 20:
    stable = all s in model |> diff: s.node != "low"
    assert(tank.changed(), ["tank.level", "empty"])
    assert(touches("empty"), true)
    assert(stable, true)
    low = 30
    assert(stable, false)
    assert(touches("empty"), false)
    assert(tank.changed(), ["low", "tank.level"])"#);
}

/// Nucleoid composes hypothetical differences with state filters
#[rustfmt::skip]
#[test]
fn composes_hypothetical_differences_with_state_filters() {
    let mut run = runner();
    run(r#"price = 10
discount = 0
net = price - discount
taxed = net * 2
suppose discount = 3:
    facts = model |> diff |> stated
    effects = model |> derived |> diff
    assert(facts.map(s -> s.node), ["discount"])
    assert(effects.map(s -> s.node), ["net", "taxed"])
    assert(effects[0].from, ["price", "discount"])
    price = 13
    assert(facts.map(s -> s.node), ["price", "discount"])
    assert(effects, [])"#);
}

/// Nucleoid finds the middle values of lists with median
#[rustfmt::skip]
#[test]
fn finds_the_middle_values_of_lists_with_median() {
    runner()("waits = [12, 3, 45]\ntypical = waits.median()\nassert: [typical, waits, (why typical)[0].from], [12, [12, 3, 45], [\"waits\"]]\nwaits.push(7)\nassert: typical, 9.5\nwaits.push(300)\nassert: [typical, waits.average()], [12, 73.4]\nwaits.pop()\nwaits.shift()\nassert: typical, 7\ncourier = {stops: [12, 2, 45, 31]}\nusual = courier.stops.median()\nshort = courier.stops.filter(stop -> stop < 30).median()\nassert: [usual, short], [21.5, 7]\ncourier.stops.push(5)\nassert: [usual, short], [12, 5]");
    let mut run = runner();
    run("waits = [12, 3, 45]");
    run("typical = waits.median()");
    assert_eq!(run("[typical, waits, (why typical)[0].from]"), serde_json::json!([12, [12, 3, 45], ["waits"]]));
    run("waits.push(7)");
    assert_eq!(run("typical"), 9.5);
    run("waits.push(300)");
    assert_eq!(run("[typical, waits.average()]"), serde_json::json!([12, 73.4]));
    run("waits.pop()");
    run("waits.shift()");
    assert_eq!(run("typical"), 7);
    run("courier = {stops: [12, 2, 45, 31]}");
    run("usual = courier.stops.median()");
    run("short = courier.stops.filter(stop -> stop < 30).median()");
    assert_eq!(run("[usual, short]"), serde_json::json!([21.5, 7]));
    run("courier.stops.push(5)");
    assert_eq!(run("[usual, short]"), serde_json::json!([12, 5]));
}

/// Nucleoid finds the medians of class instance populations
#[rustfmt::skip]
#[test]
fn finds_the_medians_of_class_instance_populations() {
    runner()("class Listing(area, price):\n    this.area = area\n    this.price = price\ndef perMeter listing => listing.price / listing.area\nListing(50, 200000)\nloft = Listing(40, 400000)\ntypical = Listing.median(perMeter)\nroomy = Listing.filter(listing -> listing.area > 45).median(perMeter)\nrecent = Listing[-3:].median(listing -> listing.area)\nassert: [typical, roomy, recent], [7000, 4000, 45]\nListing(80, 72000)\nassert: [typical, roomy, recent], [4000, 2450, 50]\nListing(100, 450000)\nassert: [typical, roomy, recent], [4250, 4000, 80]\nloft.price = 120000\nassert: [typical, roomy], [3500, 4000]");
    let mut run = runner();
    run(r#"class Listing(area, price):
    this.area = area
    this.price = price"#);
    run("def perMeter listing => listing.price / listing.area");
    run("Listing(50, 200000)");
    run("loft = Listing(40, 400000)");
    run("typical = Listing.median(perMeter)");
    run("roomy = Listing.filter(listing -> listing.area > 45).median(perMeter)");
    run("recent = Listing[-3:].median(listing -> listing.area)");
    assert_eq!(run("[typical, roomy, recent]"), serde_json::json!([7000, 4000, 45]));
    run("Listing(80, 72000)");
    assert_eq!(run("[typical, roomy, recent]"), serde_json::json!([4000, 2450, 50]));
    run("Listing(100, 450000)");
    assert_eq!(run("[typical, roomy, recent]"), serde_json::json!([4250, 4000, 80]));
    run("loft.price = 120000");
    assert_eq!(run("[typical, roomy]"), serde_json::json!([3500, 4000]));
}

/// Nucleoid orders values as numbers to find medians
#[rustfmt::skip]
#[test]
fn orders_values_as_numbers_to_find_medians() {
    runner()("north = [21, 9]\nsouth = [-3, 7.5]\nmiddle = north.concat(south).median()\nwarm = north.filter(temp -> temp > 0).median()\nassert: [middle, warm], [8.25, 15]\nnorth.push(-8)\nassert: [middle, warm, north], [7.5, 15, [21, 9, -8]]\nnorth.push(4)\nassert: [middle, warm], [5.75, 9]");
    let mut run = runner();
    run("north = [21, 9]");
    run("south = [-3, 7.5]");
    run("middle = north.concat(south).median()");
    run("warm = north.filter(temp -> temp > 0).median()");
    assert_eq!(run("[middle, warm]"), serde_json::json!([8.25, 15]));
    run("north.push(-8)");
    assert_eq!(run("[middle, warm, north]"), serde_json::json!([7.5, 15, [21, 9, -8]]));
    run("north.push(4)");
    assert_eq!(run("[middle, warm]"), serde_json::json!([5.75, 9]));
}

/// Nucleoid calls the median selector once per item with only the item
#[rustfmt::skip]
#[test]
fn calls_the_median_selector_once_per_item_with_only_the_item() {
    runner()("seen = []\ndef weigh(item, position):\n    seen.push(position ?? item)\n    return item * item\nitems = [4, 1]\ndef spread() => items.median(weigh)\nassert: [spread(), seen], [8.5, [4, 1]]\nfloor = 2\ndef price item => Math.max(item, floor)\ncost = items.median(price)\nassert: cost, 3\nfloor = 3\nitems.push(12)\nassert: [cost, spread(), seen], [4, 16, [4, 1, 4, 1, 12]]\nmissing = null\nskipped = missing.median(weigh(9))\nassert: [skipped, seen], [null, [4, 1, 4, 1, 12]]");
    let mut run = runner();
    run("seen = []");
    run(r#"def weigh(item, position):
    seen.push(position ?? item)
    return item * item"#);
    run("items = [4, 1]");
    run("def spread() => items.median(weigh)");
    assert_eq!(run("[spread(), seen]"), serde_json::json!([8.5, [4, 1]]));
    run("floor = 2");
    run("def price item => Math.max(item, floor)");
    run("cost = items.median(price)");
    assert_eq!(run("cost"), 3);
    run("floor = 3");
    run("items.push(12)");
    assert_eq!(run("[cost, spread(), seen]"), serde_json::json!([4, 16, [4, 1, 4, 1, 12]]));
    run("missing = null");
    run("skipped = missing.median(weigh(9))");
    assert_eq!(run("[skipped, seen]"), serde_json::json!([null, [4, 1, 4, 1, 12]]));
}

/// Nucleoid keeps medians live in class level rules
#[rustfmt::skip]
#[test]
fn keeps_medians_live_in_class_level_rules() {
    runner()("class Room(name): this.name = name\nclass Sensor(room, temp):\n    this.room = room\n    this.temp = temp\n$Room.mid = Sensor.filter(sensor -> sensor.room == $Room).median(sensor -> sensor.temp)\n$Room.warm = $Room.mid > 21\nlab = Room(\"lab\")\nhall = Room(\"hall\")\nSensor(lab, 8)\nprobe = Sensor(lab, 25)\nSensor(lab, 30)\nSensor(hall, 18)\nSensor(hall, 23)\nspread = Room.median(room -> room.mid)\nassert: [lab.mid, lab.warm, hall.mid, hall.warm, spread], [25, true, 20.5, false, 22.75]\nprobe.temp = 21\nassert: [lab.mid, lab.warm, spread], [21, false, 20.75]\nprobe.room = hall\nporch = Room(\"porch\")\nSensor(porch, 5)\nassert: [lab.mid, hall.mid, porch.mid, spread], [19, 21, 5, 19]");
    let mut run = runner();
    run("class Room(name): this.name = name");
    run(r#"class Sensor(room, temp):
    this.room = room
    this.temp = temp"#);
    run("$Room.mid = Sensor.filter(sensor -> sensor.room == $Room).median(sensor -> sensor.temp)");
    run("$Room.warm = $Room.mid > 21");
    run(r#"lab = Room("lab")"#);
    run(r#"hall = Room("hall")"#);
    run("Sensor(lab, 8)");
    run("probe = Sensor(lab, 25)");
    run("Sensor(lab, 30)");
    run("Sensor(hall, 18)");
    run("Sensor(hall, 23)");
    run("spread = Room.median(room -> room.mid)");
    assert_eq!(run("[lab.mid, lab.warm, hall.mid, hall.warm, spread]"), serde_json::json!([25, true, 20.5, false, 22.75]));
    run("probe.temp = 21");
    assert_eq!(run("[lab.mid, lab.warm, spread]"), serde_json::json!([21, false, 20.75]));
    run("probe.room = hall");
    run(r#"porch = Room("porch")"#);
    run("Sensor(porch, 5)");
    assert_eq!(run("[lab.mid, hall.mid, porch.mid, spread]"), serde_json::json!([19, 21, 5, 19]));
}

/// Nucleoid uses medians in functions methods and conditional expressions
#[rustfmt::skip]
#[test]
fn uses_medians_in_functions_methods_and_conditional_expressions() {
    runner()("class Team(name, scores):\n    this.name = name\n    this.scores = scores\n    def middle(): return this.scores.median()\ndef lift(team) => team.scores.max() - team.scores.median()\nred = Team(\"red\", [3, 9])\nblue = Team(\"blue\", [8, 5, 7])\nleague = Team.median(team -> team.middle())\nleader = red.middle() > blue.middle() ? red.name : blue.name\nassert: [league, leader, lift(red)], [6.5, \"blue\", 3]\nred.scores.push(10)\nblue.scores.push(2)\nTeam(\"green\", [13])\nassert: [league, leader, lift(red)], [9, \"red\", 1]");
    let mut run = runner();
    run(r#"class Team(name, scores):
    this.name = name
    this.scores = scores
    def middle(): return this.scores.median()"#);
    run("def lift(team) => team.scores.max() - team.scores.median()");
    run(r#"red = Team("red", [3, 9])"#);
    run(r#"blue = Team("blue", [8, 5, 7])"#);
    run("league = Team.median(team -> team.middle())");
    run("leader = red.middle() > blue.middle() ? red.name : blue.name");
    assert_eq!(run("[league, leader, lift(red)]"), serde_json::json!([6.5, "blue", 3]));
    run("red.scores.push(10)");
    run("blue.scores.push(2)");
    run(r#"Team("green", [13])"#);
    assert_eq!(run("[league, leader, lift(red)]"), serde_json::json!([9, "red", 1]));
}

/// Nucleoid keeps median as an ordinary name beside the median query
#[rustfmt::skip]
#[test]
fn keeps_median_as_an_ordinary_name_beside_the_median_query() {
    runner()("median = 2\nreport = {median: 5}\ndef double median => median * 2\nclass Bag(items):\n    this.items = items\n    def median(): return this.items.median(item -> item * item)\nbag = Bag([1, 3])\nBag([4])\nlevel = bag.items.median() + median + report.median\nbags = Bag.median(each -> each.median())\nassert: [level, bag.median(), bags, double(median)], [9, 5, 10.5, 4]\nassert: bag.items.median, null\nbag.items.push(10)\nmedian = 4\nBag([6])\nassert: [level, bag.median(), bags], [12, 9, 16]");
    let mut run = runner();
    run("median = 2");
    run("report = {median: 5}");
    run("def double median => median * 2");
    run(r#"class Bag(items):
    this.items = items
    def median(): return this.items.median(item -> item * item)"#);
    run("bag = Bag([1, 3])");
    run("Bag([4])");
    run("level = bag.items.median() + median + report.median");
    run("bags = Bag.median(each -> each.median())");
    assert_eq!(run("[level, bag.median(), bags, double(median)]"), serde_json::json!([9, 5, 10.5, 4]));
    assert_eq!(run("bag.items.median"), serde_json::Value::Null);
    run("bag.items.push(10)");
    run("median = 4");
    run("Bag([6])");
    assert_eq!(run("[level, bag.median(), bags]"), serde_json::json!([12, 9, 16]));
}

/// Nucleoid rolls back failed median updates
#[rustfmt::skip]
#[test]
fn rolls_back_failed_median_updates() {
    runner()("def checked(value):\n    if value > 100:\n        throw \"TOO_LARGE\"\n    return value\namounts = [10, 40]\nmid = amounts.median(checked)\nfailure = \"\"\ntry:\n    amounts.push(500)\ncatch error:\n    failure = error\nassert: [failure, amounts, mid], [\"TOO_LARGE\", [10, 40], 25]\namounts.push(5)\nassert: mid, 10\nclass Item(price): this.price = price\nItem(3)\nItem(8)\ntypical = Item.median(item -> checked(item.price))\nfailure = \"\"\ntry:\n    Item(500)\ncatch error:\n    failure = error\nassert: [failure, Item.length, typical], [\"TOO_LARGE\", 2, 5.5]\nsuppose:\n    Item(12)\n    assert: typical, 8\nassert: [Item.length, typical], [2, 5.5]");
    let mut run = runner();
    run(r#"def checked(value):
    if value > 100:
        throw "TOO_LARGE"
    return value
amounts = [10, 40]
mid = amounts.median(checked)
failure = ""
try:
    amounts.push(500)
catch error:
    failure = error
assert: [failure, amounts, mid], ["TOO_LARGE", [10, 40], 25]
amounts.push(5)
assert: mid, 10
class Item(price): this.price = price
Item(3)
Item(8)
typical = Item.median(item -> checked(item.price))
failure = ""
try:
    Item(500)
catch error:
    failure = error
assert: [failure, Item.length, typical], ["TOO_LARGE", 2, 5.5]
suppose:
    Item(12)
    assert: typical, 8
assert: [Item.length, typical], [2, 5.5]"#);
}

/// Nucleoid swaps two class properties through fixed cross anchors
#[rustfmt::skip]
#[test]
fn swaps_two_class_properties_through_fixed_cross_anchors() {
    let mut run = runner();
    run(r#"class Box(width, height):
    this.width = width
    this.height = height"#);
    run("early = Box(4, 1)");
    run("$Box.width = $Box.height@first");
    run("$Box.height = $Box.width@first");
    run("box = Box(3, 2)");
    assert_eq!(run("[box.width, box.height]"), serde_json::json!([2, 3]));
    assert_eq!(run("[early.width, early.height]"), serde_json::json!([1, 4]));
    assert_eq!(run("[box.width@first, box.height@first]"), serde_json::json!([3, 2]));
    assert_eq!(run("[box.width@1, box.height@1, box.width@2]"), serde_json::json!([2, 3, null]));
    assert_eq!(run("[early.width@1, early.height@1, early.width@2]"), serde_json::json!([1, 4, null]));
    run(r#"[box.width, box.height, early.width, early.height]

# return: [2,3,1,4]"#);
}

/// Nucleoid exchanges coordinates and turns rectangles through fixed cross anchors
#[rustfmt::skip]
#[test]
fn exchanges_coordinates_and_turns_rectangles_through_fixed_cross_anchors() {
    let mut run = runner();
    run(r#"class Point(x, y):
    this.x = x
    this.y = y"#);
    run("$Point.x = $Point.y@first");
    run("$Point.y = $Point.x@first");
    run("p = Point(1, 5)");
    run("q = Point(-2, 4)");
    assert_eq!(run("[p.x, p.y, q.x, q.y]"), serde_json::json!([5, 1, 4, -2]));
    run("size = 10");
    run(r#"class Rect(x, y, width, height):
    this.x = x
    this.y = y
    this.width = width
    this.height = height"#);
    run("$Rect.x = $Rect.y@first");
    run("$Rect.y = size - $Rect.x@first - $Rect.width@first");
    run("$Rect.width = $Rect.height@first");
    run("$Rect.height = $Rect.width@first");
    run("r = Rect(1, 2, 4, 3)");
    assert_eq!(run("[r.x, r.y, r.width, r.height]"), serde_json::json!([2, 5, 3, 4]));
    run("size = 20");
    assert_eq!(run("[r.x, r.y, r.width, r.height]"), serde_json::json!([2, 15, 3, 4]));
    assert_eq!(run("[r.x@first, r.y@first, r.width@first, r.height@first]"), serde_json::json!([1, 2, 4, 3]));
    run(r#"[p.x, p.y, r.x, r.y]

# return: [5,1,2,15]"#);
}

/// Nucleoid rotates class properties with numeric first and indexed anchors
#[rustfmt::skip]
#[test]
fn rotates_class_properties_with_numeric_first_and_indexed_anchors() {
    let mut run = runner();
    run(r#"class Triple(a, b, c):
    this.a = a
    this.b = b
    this.c = c"#);
    run("$Triple.a = $Triple.b@0");
    run(r#"$Triple.b = $Triple["c"]@first"#);
    run("$Triple.c = $Triple.a@0");
    run("t = Triple(1, 2, 3)");
    assert_eq!(run("[t.a, t.b, t.c]"), serde_json::json!([2, 3, 1]));
    run(r#"class Pair(a, b):
    this.a = a
    this.b = b"#);
    run("$Pair.a = $Pair.a@first + $Pair.b@first");
    run("$Pair.b = $Pair.a@first");
    run("pair = Pair(1, 2)");
    assert_eq!(run("[pair.a, pair.b]"), serde_json::json!([3, 1]));
    run(r#"class Base(a, b):
    this.a = a
    this.b = b"#);
    run(r#"class Child: Base
    def init(a, b):
        super(a, b)"#);
    run("$Base.a = $Base.b@first");
    run("$Child.b = $Child.a@first");
    run("child = Child(1, 2)");
    run("parent = Base(5, 6)");
    assert_eq!(run("[child.a, child.b]"), serde_json::json!([2, 1]));
    assert_eq!(run("[parent.a, parent.b]"), serde_json::json!([6, 6]));
    run(r#"[t.a, t.b, t.c]

# return: [2,3,1]"#);
}

/// Nucleoid keeps fixed cross anchors tracked through slot deletion and recreation
#[rustfmt::skip]
#[test]
fn keeps_fixed_cross_anchors_tracked_through_slot_deletion_and_recreation() {
    let mut run = runner();
    run(r#"class Box(width, height):
    this.width = width
    this.height = height"#);
    run("$Box.width = $Box.height@first");
    run("$Box.height = $Box.width@first");
    run(r#"class Frozen(width, height):
    this.width = width
    this.height = height"#);
    run("$Frozen.width = $Frozen.height@first.value");
    run("$Frozen.height = $Frozen.width@first.value");
    run("box = Box(3, 2)");
    run("frozen = Frozen(3, 2)");
    assert_eq!(run("[box.width, box.height, frozen.width, frozen.height]"), serde_json::json!([2, 3, 2, 3]));
    run("delete box.height");
    run("delete frozen.height");
    assert_eq!(run("box.width"), serde_json::Value::Null);
    assert_eq!(run("frozen.width"), 2);
    run("box.height = 7");
    run("frozen.height = 7");
    assert_eq!(run("[box.width, box.height]"), serde_json::json!([7, 7]));
    assert_eq!(run("[frozen.width, frozen.height]"), serde_json::json!([2, 7]));
    assert_eq!(run("[box.width@first, box.width@1, box.width@2, box.width@3]"), serde_json::json!([3, 2, null, 7]));
    assert_eq!(run("box.height@first"), 7);
    run(r#"[box.width, frozen.width]

# return: [7,2]"#);
}

/// Nucleoid settles fixed cross anchor cycles beside live inputs and readers
#[rustfmt::skip]
#[test]
fn settles_fixed_cross_anchor_cycles_beside_live_inputs_and_readers() {
    let mut run = runner();
    run("k = 1");
    run(r#"class Pair(a, b):
    this.a = a
    this.b = b"#);
    run("$Pair.a = $Pair.b@first + k");
    run("$Pair.b = $Pair.a@first + k");
    run("$Pair.sum = $Pair.a + $Pair.b");
    run("pair = Pair(1, 2)");
    assert_eq!(run("[pair.a, pair.b, pair.sum]"), serde_json::json!([3, 2, 5]));
    run("k = 10");
    assert_eq!(run("[pair.a, pair.b, pair.sum]"), serde_json::json!([12, 11, 23]));
    assert_eq!(run("[pair.a@first, pair.b@first, pair.a@3]"), serde_json::json!([1, 2, null]));
    run(r#"class Late(a, b):
    this.a = a
    this.b = b"#);
    run("$Late.a = $Late.b@1 ?? -1");
    run("$Late.b = $Late.a@first");
    run("late = Late(1, 2)");
    assert_eq!(run("[late.a, late.b]"), serde_json::json!([1, 1]));
    assert_eq!(run("[late.a@1, late.a@2, late.b@1]"), serde_json::json!([-1, 1, 1]));
    run(r#"class Missing(width):
    this.width = width"#);
    run("$Missing.width = $Missing.height@first");
    run("$Missing.height = $Missing.width@first");
    run("missing = Missing(3)");
    assert_eq!(run("[missing.width, missing.height]"), serde_json::json!([3, 3]));
    assert_eq!(run("[missing.width@1, missing.width@2]"), serde_json::json!([null, 3]));
    run(r#"[pair.a, pair.b, pair.sum]

# return: [12,11,23]"#);
}

/// Nucleoid keeps live and moving reads in class cycles as circular dependencies
#[rustfmt::skip]
#[test]
fn keeps_live_and_moving_reads_in_class_cycles_as_circular_dependencies() {
    let mut run = runner();
    run(r#"class Sample(x, y):
    this.x = x
    this.y = y
def latest(sample) => sample.y@last
item = Sample(1, 2)
try:
    $Sample.x = $Sample.y@first
    $Sample.y = $Sample.x + 1
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.x@first + $Sample.y
    $Sample.y = $Sample.x
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.y@first
    $Sample.y = $Sample.x@last
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.y@previous
    $Sample.y = $Sample.x@first
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.y@first + $Sample.y
    $Sample.y = $Sample.x@first
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.y@first + latest($Sample) * 0
    $Sample.y = $Sample.x@first
catch error:
    assert(error, TypeError("Circular Dependency"))
try:
    $Sample.x = $Sample.y@first
    $Sample.y = $Sample.x@first
    $Sample.y += 1
catch error:
    assert(error, TypeError("Circular Dependency"))
assert([item.x, item.y], [1, 2])
assert([item.x@1, item.y@1], [null, null])
$Sample.x = $Sample.y@first
$Sample.y = $Sample.x@first
assert([item.x, item.y], [2, 1])
[item.x, item.y]
# return: [2,1]"#);
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
