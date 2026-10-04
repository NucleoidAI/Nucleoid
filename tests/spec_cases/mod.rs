// @generated from dataset JSONL by build.rs — do not edit.

/// Nucleoid runs a statement in the state
#[test]
fn runs_a_statement_in_the_state() {
    let mut run = runner();
    run(r#"i = 1"#);
    assert_eq!(run(r#"i == 1"#), run(r#"(true)"#));
}

/// Nucleoid runs a expression statement
#[test]
fn runs_a_expression_statement() {
    let mut run = runner();
    run(r#"j = 1"#);
    assert_eq!(run(r#"j + 2"#), run(r#"(3)"#));
}

/// Nucleoid returns value of variable
#[test]
fn returns_value_of_variable() {
    let mut run = runner();
    run(r#"k = 1"#);
    run(r#"k

# return: 1"#);
}

/// Nucleoid throws an error if variable is not defined
#[test]
fn throws_an_error_if_variable_is_not_defined() {
    let mut run = runner();
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    t = e + 1
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("e is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid throws an error inside a block
#[test]
fn throws_an_error_inside_a_block() {
    let mut run = runner();
    run(r#"k = 99"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    if k >= 99:
        throw "INVALID"
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = "INVALID"
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid throws an error as a variable
#[test]
fn throws_an_error_as_a_variable() {
    let mut run = runner();
    run(r#"length = 0.1"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    if length < 1:
        throw length
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = 0.1
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
    run("__nucleoid_test_assertion_1_actual = null");
    run("__nucleoid_test_assertion_1_expected = null");
    run("__nucleoid_test_assertion_1_ran = false");
    run(r#"try:
    if length < 1.1:
        throw 'length'
catch error:
    __nucleoid_test_assertion_1_actual = error
    __nucleoid_test_assertion_1_expected = "length"
    __nucleoid_test_assertion_1_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_1_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_1_actual"), run("(__nucleoid_test_assertion_1_expected)"));
}

/// Nucleoid creates a class with constructor
#[test]
fn creates_a_class_with_constructor() {
    let mut run = runner();
    run(r#"class Shape(type: str):
    this.type = type"#);
    run(r#"shape1 = Shape("Square")"#);
    assert_eq!(run(r#"shape1"#), run(r#"({ "id": "shape1", "type": "Square" })"#));
}

/// Nucleoid creates a class with a constructor and a typed attribute
#[test]
fn creates_a_class_with_a_constructor_and_a_typed_attribute() {
    let mut run = runner();
    run(r#"class Shape:
    type: str

    def init(type: str):
        this.type = type"#);
    run(r#"shape1 = Shape("Rectangle")"#);
    assert_eq!(run(r#"shape1"#), run(r#"({ "id": "shape1", "type": "Rectangle" })"#));
}

/// Nucleoid adds an object to the class's object list
#[test]
fn adds_an_object_to_the_class_s_object_list() {
    let mut run = runner();
    run(r#"class Student:
    pass"#);
    run(r#"user0 = Student()"#);
    assert_eq!(run(r#"Student.find(student => student.id == "user0")"#), run(r#"({ "id": "user0" })"#));
    assert_eq!(run(r#"Student["user0"]"#), run(r#"({ "id": "user0" })"#));
}

/// Nucleoid preserves class and object lists when a class is updated
#[test]
fn preserves_class_and_object_lists_when_a_class_is_updated() {
    let mut run = runner();
    run(r#"class User:
    pass"#);
    run(r#"User()"#);
    assert_eq!(run(r#"Class.length"#), run(r#"(1)"#));
    assert_eq!(run(r#"User.length"#), run(r#"(1)"#));
    run(r#"class User:
    pass"#);
    assert_eq!(run(r#"Class.length"#), run(r#"(1)"#));
    assert_eq!(run(r#"User.length"#), run(r#"(1)"#));
    run(r#"User()"#);
    assert_eq!(run(r#"Class.length"#), run(r#"(1)"#));
    assert_eq!(run(r#"User.length"#), run(r#"(2)"#));
}

/// Nucleoid places an instance in the list of the class when created
#[test]
fn places_an_instance_in_the_list_of_the_class_when_created() {
    let mut run = runner();
    run(r#"class Student:
    pass"#);
    assert_eq!(run(r#"typeof Student"#), run(r#"(List)"#));
    run(r#"student1 = Student()"#);
    assert_eq!(run(r#"Student.length"#), run(r#"(1)"#));
}

/// Nucleoid creates a class and a subclass
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
    assert_eq!(run(r#"student1"#), run(r#"({ "id": "student1", "name": "Emma", "school": "Riverside High" })"#));
}

/// Nucleoid runs a class-level property assignment
#[test]
fn runs_a_class_level_property_assignment() {
    let mut run = runner();
    run(r#"class Human(name: str):
    this.name = name"#);
    run(r#"$Human.mortal = true"#);
    run(r#"human1 = Human("Socrates")"#);
    assert_eq!(run(r#"human1.mortal"#), run(r#"(true)"#));
}

/// Nucleoid runs a class-level conditional
#[test]
fn runs_a_class_level_conditional() {
    let mut run = runner();
    run(r#"class Device(profile: str):
    this.profile = profile"#);
    run(r#"if $Device.profile:
    $Device.active = true"#);
    run(r#"device1 = Device()"#);
    run(r#"device2 = Device("PROFILE-1")"#);
    assert_eq!(run(r#"device1.active"#), run(r#"(null)"#));
    assert_eq!(run(r#"device2.active"#), run(r#"(true)"#));
}

/// Nucleoid creates an instance in a block and assigns it to a property
#[test]
fn creates_an_instance_in_a_block_and_assigns_it_to_a_property() {
    let mut run = runner();
    run(r#"class Room:
    pass"#);
    run(r#"class Meeting:
    pass"#);
    run(r#"room1 = Room()"#);
    run(r#"$Meeting.time = Date.now() + " @ " + $Meeting.date.toDateString()"#);
    run(r#"{
    meeting = Meeting()
    meeting.date = Date("2020-1-1")
    room1.meeting = meeting
}"#);
    assert_eq!(run(r#"room1.meeting.date.toDateString()"#), run(r#"("Wed Jan 01 2020")"#));
    assert_eq!(run(r#"room1.meeting.time[-17:]"#), run(r#"("@ Wed Jan 01 2020")"#));
}

/// Nucleoid creates nested instances in a block and assigns them to a property
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
    run(r#"timesheet1 = Timesheet()"#);
    run(r#"{
    task = Task()
    task.project = Project()
    task.project.number = 3668347
    timesheet1.task = task
}"#);
    assert_eq!(run(r#"timesheet1.task.project.number"#), run(r#"(3668347)"#));
    assert_eq!(run(r#"timesheet1.task.project.code"#), run(r#"("N-3668347")"#));
}

/// Nucleoid creates a local variable in a block and uses in assignment
#[test]
fn creates_a_local_variable_in_a_block_and_uses_in_assignment() {
    let mut run = runner();
    run(r#"integer = 30"#);
    run(r#"equivalency = null"#);
    run(r#"{
    division = integer / 10
    equivalency = division * 10
}"#);
    assert_eq!(run(r#"equivalency"#), run(r#"(30)"#));
    run(r#"integer = 40"#);
    assert_eq!(run(r#"equivalency"#), run(r#"(40)"#));
}

/// Nucleoid creates a standard built-in object as a local variable inside a block
#[test]
fn creates_a_standard_built_in_object_as_a_local_variable_inside_a_block() {
    let mut run = runner();
    run(r#"{
    f = Boolean(false)
    condition = f
}"#);
    assert_eq!(run(r#"condition"#), run(r#"(false)"#));
}

/// Nucleoid creates and assigns an instance to a local variable inside a block
#[test]
fn creates_and_assigns_an_instance_to_a_local_variable_inside_a_block() {
    let mut run = runner();
    run(r#"class Device:
    pass"#);
    run(r#"$Device.renew = $Device.created + 604800000"#);
    run(r#"{
    device = Device()
    device.created = Date.now()
}"#);
    assert_eq!(run(r#"Device[0].renew - Device[0].created"#), run(r#"(604800000)"#));
}

/// Nucleoid creates and assigns an instance with a constructor to a local variable inside a block
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
    assert_eq!(run(r#"Member[0].display"#), run(r#"("Last, First")"#));
}

/// Nucleoid creates an object in a block and assigns it to a class-level property before instantiation
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
    run(r#"member1 = Member()"#);
    assert_eq!(run(r#"member1.registration.date.toDateString()"#), run(r#"("Wed Jan 02 2019")"#));
    assert_eq!(run(r#"member1.registration.age"#), run(r#"(null)"#));
}

/// Nucleoid creates an object in a block and assigns it to a class-level property after instantiation
#[test]
fn creates_an_object_in_a_block_and_assigns_it_to_a_class_level_property_after_instantiation() {
    let mut run = runner();
    run(r#"class Distance:
    pass"#);
    run(r#"distance1 = Distance()"#);
    run(r#"{
    location = Object()
    location.coordinates = "40.6976701,-74.2598779"
    $Distance.startingPoint = location
}"#);
    assert_eq!(run(r#"distance1.startingPoint.coordinates"#), run(r#"("40.6976701,-74.2598779")"#));
    assert_eq!(run(r#"distance1.startingPoint.print"#), run(r#"(null)"#));
}

/// Nucleoid calls function in an assignment
#[test]
fn calls_function_in_an_assignment() {
    let mut run = runner();
    run(r#"def multiply(first_factor, second_factor):
    product = first_factor * second_factor
    return product"#);
    run(r#"x = 1"#);
    run(r#"y = 2"#);
    run(r#"z = multiply(x, y) + 1"#);
    assert_eq!(run(r#"z"#), run(r#"(3)"#));
}

/// Nucleoid assigns a block in a function as a dependency
#[test]
fn assigns_a_block_in_a_function_as_a_dependency() {
    let mut run = runner();
    run(r#"class Student:
    pass"#);
    run(r#"student1 = Student()"#);
    run(r#"student1.age = 7"#);
    run(r#"student2 = Student()"#);
    run(r#"student2.age = 8"#);
    run(r#"student3 = Student()"#);
    run(r#"student3.age = 9"#);
    run(r#"age = 8"#);
    run(r#"student = Student.find(s => s.age == age)"#);
    assert_eq!(run(r#"student"#), run(r#"(student2)"#));
    assert_eq!(run(r#"student"#), run(r#"({ "id": "student2", "age": 8 })"#));
    run(r#"age = 9"#);
    assert_eq!(run(r#"student"#), run(r#"(student3)"#));
    assert_eq!(run(r#"student"#), run(r#"({ "id": "student3", "age": 9 })"#));
}

/// Nucleoid supports chained functions with a parameter in an expression
#[test]
fn supports_chained_functions_with_a_parameter_in_an_expression() {
    let mut run = runner();
    run(r#"class Result(score: int):
    this.score = score"#);
    run(r#"Result(10); Result(15); Result(20)"#);
    run(r#"upperThreshold = 18"#);
    run(r#"lowerThreshold = 12"#);
    run(r#"list = Result.filter(r => r.score > lowerThreshold).filter(r => r.score < upperThreshold)"#);
    assert_eq!(run(r#"list.length"#), run(r#"(1)"#));
    assert_eq!(run(r#"list[0].score"#), run(r#"(15)"#));
    run(r#"lowerThreshold = 7"#);
    assert_eq!(run(r#"list.length"#), run(r#"(2)"#));
    assert_eq!(run(r#"list[0].score"#), run(r#"(10)"#));
    assert_eq!(run(r#"list[1].score"#), run(r#"(15)"#));
    run(r#"upperThreshold = 14"#);
    assert_eq!(run(r#"list.length"#), run(r#"(1)"#));
    assert_eq!(run(r#"list[0].score"#), run(r#"(10)"#));
}

/// Nucleoid supports an array with brackets
#[test]
fn supports_an_array_with_brackets() {
    let mut run = runner();
    run(r#"states = ["NY", "GA", "CT", "MI"]"#);
    run(r#"states[2]

# return: "CT""#);
}

/// Nucleoid throws an error if a variable in an expression is not defined
#[test]
fn throws_an_error_if_a_variable_in_an_expression_is_not_defined() {
    let mut run = runner();
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    e == 2.71828
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("e is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid retrieves the value of a variable
#[test]
fn retrieves_the_value_of_a_variable() {
    let mut run = runner();
    run(r#"number = -1"#);
    run(r#"number

# return: -1"#);
}

/// Nucleoid creates a property assignment on a local variable only if the instance is defined
#[test]
fn creates_a_property_assignment_on_a_local_variable_only_if_the_instance_is_defined() {
    let mut run = runner();
    run(r#"class Ticket:
    pass"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    {
        ticket = Ticket()
        ticket.event.group = "ENTERTAINMENT"
    }
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("ticket.event is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid declares a local variable as undefined
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
#[test]
fn rejects_a_local_variable_declared_as_undefined() {
    let mut run = runner();
    run(r#"class Device(code: str):
    this.code = code"#);
    run(r#"device1 = Device("A0")"#);
    run(r#"device2 = Device("B1")"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    {
        device = Device.find(d => d.code == "A1")
        if not device:
            throw "INVALID_DEVICE"
        return device
    }
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = "INVALID_DEVICE"
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid creates a standard built-in object as a property of a local variable
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
    assert_eq!(run(r#"shipment1.date.toDateString()"#), run(r#"("Thu Jan 03 2019")"#));
}

/// Nucleoid creates a property of a local variable in a different scope
#[test]
fn creates_a_property_of_a_local_variable_in_a_different_scope() {
    let mut run = runner();
    run(r#"class User:
    pass"#);
    run(r#"user0 = User()"#);
    run(r#"{
    user = User["user0"]
    if user:
        user.name = "TEST"
}"#);
    assert_eq!(run(r#"user0.name"#), run(r#"("TEST")"#));
}

/// Nucleoid assigns a variable declaratively
#[test]
fn assigns_a_variable_declaratively() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"b = 2"#);
    run(r#"c = a + b"#);
    assert_eq!(run(r#"c"#), run(r#"(3)"#));
    run(r#"a = 2"#);
    assert_eq!(run(r#"c"#), run(r#"(4)"#));
}

/// Nucleoid creates if statement of variable
#[test]
fn creates_if_statement_of_variable() {
    let mut run = runner();
    run(r#"m = false"#);
    run(r#"n = false"#);
    run(r#"if m == true:
    n = m and true"#);
    assert_eq!(run(r#"n"#), run(r#"(false)"#));
    run(r#"m = true"#);
    assert_eq!(run(r#"n"#), run(r#"(true)"#));
}

/// Nucleoid updates if block of variable
#[test]
fn updates_if_block_of_variable() {
    let mut run = runner();
    run(r#"p = 0.01"#);
    run(r#"s = 0.02"#);
    run(r#"if p < 1:
    r = p * 10"#);
    run(r#"if p < 1:
    r = s * 10"#);
    assert_eq!(run(r#"r"#), run(r#"(0.2)"#));
    run(r#"s = 0.03"#);
    assert_eq!(run(r#"r"#), run(r#"(0.3)"#));
}

/// Nucleoid creates else if statement of variable
#[test]
fn creates_else_if_statement_of_variable() {
    let mut run = runner();
    run(r#"g = 11"#);
    run(r#"earth = 9.8"#);
    run(r#"mars = 3.71"#);
    run(r#"mass = 10"#);
    run(r#"if g > 9:
    weight = earth * mass
else if g > 3:
    weight = mars * mass"#);
    run(r#"g = 5"#);
    assert_eq!(run(r#"weight"#), run(r#"(37.1)"#));
    run(r#"mars = 3.72"#);
    assert_eq!(run(r#"weight"#), run(r#"(37.2)"#));
}

/// Nucleoid creates multiple else if statement of variable
#[test]
fn creates_multiple_else_if_statement_of_variable() {
    let mut run = runner();
    run(r#"fraction = -0.1"#);
    run(r#"point = 1"#);
    run(r#"if fraction > 1:
    score = fraction * point * 3
else if fraction > 0:
    score = fraction * point * 2
else:
    score = fraction * point"#);
    assert_eq!(run(r#"score"#), run(r#"(-0.1)"#));
    run(r#"point = 2"#);
    assert_eq!(run(r#"score"#), run(r#"(-0.2)"#));
}

/// Nucleoid runs dependent statements in the same transaction
#[test]
fn runs_dependent_statements_in_the_same_transaction() {
    let mut run = runner();
    run(r#"class Vehicle:
    pass"#);
    run(r#"$Vehicle.tag = "US-" + $Vehicle.plate"#);
    run(r#"vehicle1 = Vehicle()"#);
    run(r#"vehicle1.plate = "XSJ422""#);
    assert_eq!(run(r#"vehicle1.tag"#), run(r#"("US-XSJ422")"#));
}

/// Nucleoid runs dependencies in order as received
#[test]
fn runs_dependencies_in_order_as_received() {
    let mut run = runner();
    run(r#"any = 0"#);
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
    run(r#"any = 4"#);
    assert_eq!(run(r#"result"#), run(r#"(5)"#));
}

/// Nucleoid searches a variable in scope before the state
#[test]
fn searches_a_variable_in_scope_before_the_state() {
    let mut run = runner();
    run(r#"e = 2.71828"#);
    run(r#"number = null"#);
    run(r#"{
    e = 3
    number = e
}"#);
    assert_eq!(run(r#"number"#), run(r#"(3)"#));
}

/// Nucleoid uses local variable at lowest scope as priority
#[test]
fn uses_local_variable_at_lowest_scope_as_priority() {
    let mut run = runner();
    run(r#"pi = 3.14"#);
    run(r#"number = pi"#);
    run(r#"{
    pi = 3.141
    number = pi
}"#);
    assert_eq!(run(r#"number"#), run(r#"(3.141)"#));
}

/// Nucleoid assigns undefined if any dependency in expression is undefined
#[test]
fn assigns_undefined_if_any_dependency_in_expression_is_undefined() {
    let mut run = runner();
    run(r#"class Person:
    pass"#);
    run(r#"person1 = Person()"#);
    run(r#"person1.lastName = "Brown""#);
    run(r#"person1.fullName = person1.firstName + " " + person1.lastName"#);
    assert_eq!(run(r#"person1.fullName"#), run(r#"(null)"#));
}

/// Nucleoid keeps as null if any dependencies as in local is null
#[test]
fn keeps_as_null_if_any_dependencies_as_in_local_is_null() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"c = null"#);
    run(r#"{
    b = null
    c = b / a
}"#);
    assert_eq!(run(r#"c"#), run(r#"(null)"#));
}

/// Nucleoid keeps as null if any dependencies in expression is null
#[test]
fn keeps_as_null_if_any_dependencies_in_expression_is_null() {
    let mut run = runner();
    run(r#"class Schedule:
    pass"#);
    run(r#"schedule1 = Schedule()"#);
    run(r#"schedule1.expression = "0 */2 * * *""#);
    run(r#"schedule1.script = null"#);
    run(r#"schedule1.run = schedule1.expression + " " + schedule1.script"#);
    assert_eq!(run(r#"schedule1.run"#), run(r#"(null)"#));
}

/// Nucleoid assigns null if there is null pointer in expression
#[test]
fn assigns_null_if_there_is_null_pointer_in_expression() {
    let mut run = runner();
    run(r#"class Product:
    pass"#);
    run(r#"product1 = Product()"#);
    run(r#"score = product1.quality.score"#);
    assert_eq!(run(r#"score"#), run(r#"(null)"#));
}

/// Nucleoid assigns a unique variable for an instance without a variable name
#[test]
fn assigns_a_unique_variable_for_an_instance_without_a_variable_name() {
    let mut run = runner();
    run(r#"class Vehicle:
    pass"#);
    run(r#"Vehicle()"#);
    assert_eq!(run(r#"Vehicle.length"#), run(r#"(1)"#));
    assert_eq!(run(r#"Vehicle[0].id != null"#), run(r#"(true)"#));
}

/// Nucleoid creates a function in state
#[test]
fn creates_a_function_in_state() {
    let mut run = runner();
    run(r#"def generate(number):
    return number * 10"#);
    run(r#"random = 10"#);
    run(r#"number = generate(random)"#);
    assert_eq!(run(r#"number"#), run(r#"(100)"#));
    run(r#"random = 20"#);
    assert_eq!(run(r#"number"#), run(r#"(200)"#));
}

/// Nucleoid assigns a function as a dependency
#[test]
fn assigns_a_function_as_a_dependency() {
    let mut run = runner();
    run(r#"list = []"#);
    run(r#"count = list.filter(n => n % 2)"#);
    run(r#"list.push(1)"#);
    assert_eq!(run(r#"count.length"#), run(r#"(1)"#));
    run(r#"list.push(2)"#);
    assert_eq!(run(r#"count.length"#), run(r#"(1)"#));
    run(r#"list.push(3)"#);
    assert_eq!(run(r#"count.length"#), run(r#"(2)"#));
    run(r#"list.pop()"#);
    assert_eq!(run(r#"count.length"#), run(r#"(1)"#));
}

/// Nucleoid supports a regular expression literal
#[test]
fn supports_a_regular_expression_literal() {
    let mut run = runner();
    run(r#"class User:
    pass"#);
    run(r#"if not /.{4,8}/.test($User.password):
    throw 'INVALID_PASSWORD'"#);
    run(r#"user1 = User()"#);
    assert_eq!(run(r#"user1.password"#), run(r#"(null)"#));
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    user1.password = 'PAS'
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = "INVALID_PASSWORD"
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid rejects defining a class declaration in a non-class block
#[test]
fn rejects_defining_a_class_declaration_in_a_non_class_block() {
    let mut run = runner();
    run(r#"class Person:
    pass"#);
    run(r#"person1 = Person()"#);
    run(r#"person1.weight = 90"#);
    run(r#"person1.height = 1.8"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    {
        weight = person1.weight
        height = person1.height
        $Person.bmi = weight / (height * height)
    }
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = SyntaxError("Cannot define class declaration in non-class block")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid detects a circular dependency
#[test]
fn detects_a_circular_dependency() {
    let mut run = runner();
    run(r#"number1 = 10"#);
    run(r#"number2 = number1 * 10"#);
    assert_eq!(run(r#"number2"#), run(r#"(100)"#));
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    number1 = number2 * 10
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = TypeError("Circular Dependency")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid rolls back a variable if an exception is thrown
#[test]
fn rolls_back_a_variable_if_an_exception_is_thrown() {
    let mut run = runner();
    run(r#"a = 5"#);
    run(r#"if a > 5:
    throw 'INVALID_VALUE'"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    a = 6
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = "INVALID_VALUE"
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
    assert_eq!(run(r#"a"#), run(r#"(5)"#));
}

/// Nucleoid rolls back a property if an exception is thrown
#[test]
fn rolls_back_a_property_if_an_exception_is_thrown() {
    let mut run = runner();
    run(r#"class Item:
    pass"#);
    run(r#"if $Item.sku == 'A':
    throw 'INVALID_SKU'"#);
    run(r#"item1 = Item()"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    item1.sku = 'A'
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = "INVALID_SKU"
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
    assert_eq!(run(r#"item1.sku"#), run(r#"(null)"#));
}

/// Nucleoid rolls back an instance if an exception is thrown
#[test]
fn rolls_back_an_instance_if_an_exception_is_thrown() {
    let mut run = runner();
    run(r#"class User(first: str, last: str):
    this.first = first
    this.last = last"#);
    run(r#"if $User.first.length < 3:
    throw 'INVALID_USER'"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    user1 = User('F', 'L')
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = "INVALID_USER"
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
    assert_eq!(run(r#"User.length"#), run(r#"(0)"#));
    run("__nucleoid_test_assertion_1_actual = null");
    run("__nucleoid_test_assertion_1_expected = null");
    run("__nucleoid_test_assertion_1_ran = false");
    run(r#"try:
    user1
catch error:
    __nucleoid_test_assertion_1_actual = error
    __nucleoid_test_assertion_1_expected = ReferenceError("user1 is not defined")
    __nucleoid_test_assertion_1_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_1_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_1_actual"), run("(__nucleoid_test_assertion_1_expected)"));
}

/// Nucleoid updates a variable assignment
#[test]
fn updates_a_variable_assignment() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"b = 2"#);
    run(r#"c = a + 3"#);
    assert_eq!(run(r#"c"#), run(r#"(4)"#));
    run(r#"c = b + 3"#);
    assert_eq!(run(r#"c"#), run(r#"(5)"#));
    run(r#"b = 4"#);
    assert_eq!(run(r#"c"#), run(r#"(7)"#));
}

/// Nucleoid uses only the value when a variable references itself
#[test]
fn uses_only_the_value_when_a_variable_references_itself() {
    let mut run = runner();
    run(r#"radius = 10"#);
    run(r#"radius = radius + 10"#);
    assert_eq!(run(r#"radius"#), run(r#"(20)"#));
}

/// Nucleoid deletes a variable assignment
#[test]
fn deletes_a_variable_assignment() {
    let mut run = runner();
    run(r#"t = 1"#);
    run(r#"q = t + 1"#);
    assert_eq!(run(r#"q"#), run(r#"(2)"#));
    run(r#"delete q"#);
    run(r#"t = 2"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    q
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("q is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid returns the assigned value in a variable assignment
#[test]
fn returns_the_assigned_value_in_a_variable_assignment() {
    let mut run = runner();
    run(r#"x = 1

# return: 1"#);
}

/// Nucleoid assigns a parameter in a function as a dependency
#[test]
fn assigns_a_parameter_in_a_function_as_a_dependency() {
    let mut run = runner();
    run(r#"str1 = "ABC""#);
    run(r#"str2 = str1.lower() + "d""#);
    run(r#"str3 = str2 + str1"#);
    assert_eq!(run(r#"str2"#), run(r#"("abcd")"#));
    assert_eq!(run(r#"str3"#), run(r#"("abcdABC")"#));
    run(r#"str1 = "AAA""#);
    assert_eq!(run(r#"str2"#), run(r#"("aaad")"#));
    assert_eq!(run(r#"str3"#), run(r#"("aaadAAA")"#));
}

/// Nucleoid uses value property to indicate using only value of variable
#[test]
fn uses_value_property_to_indicate_using_only_value_of_variable() {
    let mut run = runner();
    run(r#"goldenRatio = 1.618"#);
    run(r#"altitude = 10"#);
    run(r#"width = goldenRatio.value * altitude"#);
    run(r#"depth = goldenRatio.value * altitude"#);
    assert_eq!(run(r#"width"#), run(r#"(16.18)"#));
    assert_eq!(run(r#"depth"#), run(r#"(16.18)"#));
    run(r#"goldenRatio = 1.62"#);
    assert_eq!(run(r#"width"#), run(r#"(16.18)"#));
    assert_eq!(run(r#"depth"#), run(r#"(16.18)"#));
    run(r#"altitude = 100"#);
    assert_eq!(run(r#"width"#), run(r#"(161.8)"#));
    assert_eq!(run(r#"depth"#), run(r#"(161.8)"#));
}

/// Nucleoid creates a nested object in a block and assigns it to a class-level property before instantiation
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
    run(r#"account1 = Account()"#);
    assert_eq!(run(r#"account1.balance.currency.code"#), run(r#"("USD")"#));
    assert_eq!(run(r#"account1.balance.currency.description"#), run(r#"(null)"#));
}

/// Nucleoid creates a nested object in a block and assigns it to a class-level property after instantiation
#[test]
fn creates_a_nested_object_in_a_block_and_assigns_it_to_a_class_level_property_after_instantiation() {
    let mut run = runner();
    run(r#"class Warehouse:
    pass"#);
    run(r#"warehouse1 = Warehouse()"#);
    run(r#"{
    inventory = Object()
    inventory.item = Object()
    inventory.item.sku = "699546085767"
    $Warehouse.inventory = inventory
}"#);
    assert_eq!(run(r#"warehouse1.inventory.item.sku"#), run(r#"("699546085767")"#));
    assert_eq!(run(r#"warehouse1.inventory.item.description"#), run(r#"(null)"#));
}

/// Nucleoid creates an instance inside a block
#[test]
fn creates_an_instance_inside_a_block() {
    let mut run = runner();
    run(r#"class Device(name: str):
    this.name = name"#);
    run(r#"$Device.deleted = false"#);
    run(r#"$Device.key = "X-" + $Device.name"#);
    run(r#"{
    name = "Hall"
    device1 = Device(name)
}"#);
    assert_eq!(run(r#"device1.name"#), run(r#"("Hall")"#));
    assert_eq!(run(r#"device1.key"#), run(r#"("X-Hall")"#));
    assert_eq!(run(r#"device1.deleted"#), run(r#"(false)"#));
}

/// Nucleoid creates an instance inside a block without a variable name
#[test]
fn creates_an_instance_inside_a_block_without_a_variable_name() {
    let mut run = runner();
    run(r#"class Summary(rate: int):
    this.rate = rate"#);
    run(r#"$Summary.score = $Summary.rate * 100"#);
    run(r#"{
    rate = 4
    Summary(rate)
}"#);
    assert_eq!(run(r#"Summary[0].rate"#), run(r#"(4)"#));
    assert_eq!(run(r#"Summary[0].score"#), run(r#"(400)"#));
}

/// Nucleoid creates a local variable inside a block
#[test]
fn creates_a_local_variable_inside_a_block() {
    let mut run = runner();
    run(r#"a = 5"#);
    run(r#"b = 10"#);
    run(r#"if a > 9:
    c = a + b
    d = c * 10"#);
    run(r#"a = 10"#);
    assert_eq!(run(r#"d"#), run(r#"(200)"#));
    run(r#"a = 15"#);
    assert_eq!(run(r#"d"#), run(r#"(250)"#));
    run(r#"b = 20"#);
    assert_eq!(run(r#"d"#), run(r#"(350)"#));
}

/// Nucleoid runs a local variable as an object before declaration
#[test]
fn runs_a_local_variable_as_an_object_before_declaration() {
    let mut run = runner();
    run(r#"class Plane:
    pass"#);
    run(r#"class Trip:
    pass"#);
    run(r#"plane1 = Plane()"#);
    run(r#"plane1.speed = 903"#);
    run(r#"trip1 = Trip()"#);
    run(r#"trip1.distance = 5540"#);
    run(r#"{
    trip = $Plane.trip
    $Plane.time = trip.distance / $Plane.speed
}"#);
    run(r#"plane1.trip = trip1"#);
    assert_eq!(run(r#"plane1.time"#), run(r#"(6.135105204872647)"#));
}

/// Nucleoid runs a local variable as an object after declaration
#[test]
fn runs_a_local_variable_as_an_object_after_declaration() {
    let mut run = runner();
    run(r#"class Seller:
    pass"#);
    run(r#"class Commission:
    pass"#);
    run(r#"seller1 = Seller()"#);
    run(r#"seller1.sales = 1000000"#);
    run(r#"comm1 = Commission()"#);
    run(r#"comm1.rate = 0.05"#);
    run(r#"seller1.commission = comm1"#);
    run(r#"{
    commission = $Seller.commission
    $Seller.pay = $Seller.sales * commission.rate
}"#);
    assert_eq!(run(r#"seller1.pay"#), run(r#"(50000)"#));
}

/// Nucleoid assigns a property on a local variable after initialization
#[test]
fn assigns_a_property_on_a_local_variable_after_initialization() {
    let mut run = runner();
    run(r#"class Stock:
    pass"#);
    run(r#"class Trade:
    pass"#);
    run(r#"stock1 = Stock()"#);
    run(r#"stock1.price = 100"#);
    run(r#"trade1 = Trade()"#);
    run(r#"trade1.quantity = 1"#);
    run(r#"stock1.trade = trade1"#);
    run(r#"{
    trade = $Stock.trade
    trade.worth = $Stock.price * trade.quantity
}"#);
    assert_eq!(run(r#"trade1.worth"#), run(r#"(100)"#));
}

/// Nucleoid reassigns a shadowing local variable in a nested block
#[test]
fn reassigns_a_shadowing_local_variable_in_a_nested_block() {
    let mut run = runner();
    run(r#"barcode = "barcode""#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"{
    barcode = "barcode"
    {
        barcode = "barcode2"
        {
            __nucleoid_test_assertion_0_actual = barcode
            __nucleoid_test_assertion_0_expected = "barcode2"
            __nucleoid_test_assertion_0_ran = true
        }
    }
}"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
    assert_eq!(run(r#"barcode"#), run(r#"("barcode")"#));
}

/// Nucleoid holds the result of a function in a local variable
#[test]
fn holds_the_result_of_a_function_in_a_local_variable() {
    let mut run = runner();
    run(r#"bugs = []"#);
    run(r#"ticket = 1"#);
    run(r#"class Bug:
    pass"#);
    run(r#"bug1 = Bug()"#);
    run(r#"bug1.ticket = 1"#);
    run(r#"bug1.priority = "LOW""#);
    run(r#"bugs.push(bug1)"#);
    run(r#"bug2 = Bug()"#);
    run(r#"bug2.ticket = 2"#);
    run(r#"bug2.priority = "MEDIUM""#);
    run(r#"bugs.push(bug2)"#);
    run(r#"{
    bug = bugs.find(b => b.ticket == ticket)
    bug.selected = true
}"#);
    assert_eq!(run(r#"bug1.selected"#), run(r#"(true)"#));
    assert_eq!(run(r#"bug2.selected"#), run(r#"(null)"#));
    run(r#"ticket = 2"#);
    assert_eq!(run(r#"bug2.selected"#), run(r#"(true)"#));
}

/// Nucleoid runs a block statement of variable
#[test]
fn runs_a_block_statement_of_variable() {
    let mut run = runner();
    run(r#"h = 1"#);
    run(r#"{
    value = h * 2
    j = value * 2
}"#);
    assert_eq!(run(r#"j"#), run(r#"(4)"#));
    run(r#"h = 2"#);
    assert_eq!(run(r#"j"#), run(r#"(8)"#));
}

/// Nucleoid runs a nested block statement of variable
#[test]
fn runs_a_nested_block_statement_of_variable() {
    let mut run = runner();
    run(r#"radius = 10"#);
    run(r#"{
    area = Math.pow(radius, 2) * 3.14
    {
        volume = area * 5
    }
}"#);
    assert_eq!(run(r#"volume"#), run(r#"(1570)"#));
}

/// Nucleoid runs a nested if statement of variable
#[test]
fn runs_a_nested_if_statement_of_variable() {
    let mut run = runner();
    run(r#"gravity = 9.8"#);
    run(r#"time = 10"#);
    run(r#"distance = 480"#);
    run(r#"target = true"#);
    run(r#"{
    dist = 1 / 2 * gravity * time * time
    if dist > distance:
        hit = target
}"#);
    assert_eq!(run(r#"hit"#), run(r#"(true)"#));
    run(r#"target = false"#);
    assert_eq!(run(r#"hit"#), run(r#"(false)"#));
}

/// Nucleoid runs a nested else statement of variable
#[test]
fn runs_a_nested_else_statement_of_variable() {
    let mut run = runner();
    run(r#"percentage = 28"#);
    run(r#"density = 0.899"#);
    run(r#"substance = "NH3""#);
    run(r#"molarConcentration = null"#);
    run(r#"fallback = 0"#);
    run(r#"{
    concentration = percentage * density / 100 * 1000
    if substance == "NH3":
        molarConcentration = concentration / 17.04
    else:
        molarConcentration = fallback
}"#);
    run(r#"substance = "NH16""#);
    run(r#"fallback = 1"#);
    assert_eq!(run(r#"molarConcentration"#), run(r#"(1)"#));
}

/// Nucleoid assigns a variable to a reference
#[test]
fn assigns_a_variable_to_a_reference() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"b = a"#);
    assert_eq!(run(r#"b"#), run(r#"(1)"#));
    run(r#"a = 2"#);
    assert_eq!(run(r#"b"#), run(r#"(2)"#));
}

/// Nucleoid assigns an object to a variable
#[test]
fn assigns_an_object_to_a_variable() {
    let mut run = runner();
    run(r#"class Model:
    pass"#);
    run(r#"model1 = Model()"#);
    assert_eq!(run(r#"typeof model1"#), run(r#"(Object)"#));
}

/// Nucleoid defines a class in the state
#[test]
fn defines_a_class_in_the_state() {
    let mut run = runner();
    run(r#"class Entity:
    pass"#);
    assert_eq!(run(r#"typeof $Entity"#), run(r#"(Class)"#));
}

/// Nucleoid rejects creating an instance if the class does not exist
#[test]
fn rejects_creating_an_instance_if_the_class_does_not_exist() {
    let mut run = runner();
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    chart1 = Chart()
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("Chart is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
    run(r#"class Chart:
    pass"#);
    run(r#"chart1 = Chart()"#);
    run("__nucleoid_test_assertion_1_actual = null");
    run("__nucleoid_test_assertion_1_expected = null");
    run("__nucleoid_test_assertion_1_ran = false");
    run(r#"try:
    chart1.plot = Plot()
catch error:
    __nucleoid_test_assertion_1_actual = error
    __nucleoid_test_assertion_1_expected = ReferenceError("Plot is not defined")
    __nucleoid_test_assertion_1_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_1_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_1_actual"), run("(__nucleoid_test_assertion_1_expected)"));
    run("__nucleoid_test_assertion_2_actual = null");
    run("__nucleoid_test_assertion_2_expected = null");
    run("__nucleoid_test_assertion_2_ran = false");
    run(r#"try:
    $Chart.plot = Plot()
catch error:
    __nucleoid_test_assertion_2_actual = error
    __nucleoid_test_assertion_2_expected = ReferenceError("Plot is not defined")
    __nucleoid_test_assertion_2_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_2_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_2_actual"), run("(__nucleoid_test_assertion_2_expected)"));
}

/// Nucleoid creates a property assignment before declaration
#[test]
fn creates_a_property_assignment_before_declaration() {
    let mut run = runner();
    run(r#"class Order:
    pass"#);
    run(r#"order1 = Order()"#);
    run(r#"order1.upc = "04061" + order1.barcode"#);
    assert_eq!(run(r#"order1.upc"#), run(r#"(null)"#));
    run(r#"order1.barcode = "94067""#);
    assert_eq!(run(r#"order1.upc"#), run(r#"("0406194067")"#));
}

/// Nucleoid creates a property assignment after declaration
#[test]
fn creates_a_property_assignment_after_declaration() {
    let mut run = runner();
    run(r#"class User:
    pass"#);
    run(r#"user1 = User()"#);
    run(r#"user1.name = "sample""#);
    run(r#"user1.email = user1.name + "@example.com""#);
    assert_eq!(run(r#"user1.email"#), run(r#"("sample@example.com")"#));
    run(r#"user1.name = "samplex""#);
    assert_eq!(run(r#"user1.email"#), run(r#"("samplex@example.com")"#));
}

/// Nucleoid creates a property assignment only if the instance is defined
#[test]
fn creates_a_property_assignment_only_if_the_instance_is_defined() {
    let mut run = runner();
    run(r#"class Channel:
    pass"#);
    run(r#"channel1 = Channel()"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    channel1.frequency.type = "ANGULAR"
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("channel1.frequency is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid creates an object and assigns it to a variable
#[test]
fn creates_an_object_and_assigns_it_to_a_variable() {
    let mut run = runner();
    run(r#"class Item(name: str):
    this.name = name"#);
    run(r#"item1 = Item("NAME-1")"#);
    assert_eq!(run(r#"item1"#), run(r#"({ "id": "item1", "name": "NAME-1" })"#));
    run(r#"item2 = Item()"#);
    assert_eq!(run(r#"item2"#), run(r#"({ "id": "item2", "name": null })"#));
}

/// Nucleoid creates an object assignment as a property only if the instance is defined
#[test]
fn creates_an_object_assignment_as_a_property_only_if_the_instance_is_defined() {
    let mut run = runner();
    run(r#"class Worker:
    pass"#);
    run(r#"class Schedule:
    pass"#);
    run(r#"worker1 = Worker()"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    worker1.duty.schedule = Schedule()
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("worker1.duty is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid uses only the value when a property references itself
#[test]
fn uses_only_the_value_when_a_property_references_itself() {
    let mut run = runner();
    run(r#"class Construction:
    pass"#);
    run(r#"construction1 = Construction()"#);
    run(r#"construction1.timeline = 120"#);
    run(r#"construction1.timeline = 2 * construction1.timeline"#);
    assert_eq!(run(r#"construction1.timeline"#), run(r#"(240)"#));
}

/// Nucleoid assigns an object to a property before initialization
#[test]
fn assigns_an_object_to_a_property_before_initialization() {
    let mut run = runner();
    run(r#"class Agent:
    pass"#);
    run(r#"class Distance:
    pass"#);
    run(r#"$Distance.total = Math.sqrt($Distance.x * $Distance.x + $Distance.y * $Distance.y)"#);
    run(r#"agent1 = Agent()"#);
    run(r#"agent1.distance = Distance()"#);
    assert_eq!(run(r#"agent1.distance.total"#), run(r#"(null)"#));
    run(r#"agent1.distance.x = 3"#);
    run(r#"agent1.distance.y = 4"#);
    assert_eq!(run(r#"agent1.distance.total"#), run(r#"(5)"#));
}

/// Nucleoid assigns an object to a property after initialization
#[test]
fn assigns_an_object_to_a_property_after_initialization() {
    let mut run = runner();
    run(r#"class Product:
    pass"#);
    run(r#"product1 = Product()"#);
    run(r#"class Quality:
    pass"#);
    run(r#"product1.quality = Quality()"#);
    run(r#"product1.quality.score = 15"#);
    assert_eq!(run(r#"product1.quality.class"#), run(r#"(null)"#));
    run(r#"$Quality.class = String.fromCharCode(65 + Math.floor($Quality.score / 10))"#);
    assert_eq!(run(r#"product1.quality.class"#), run(r#"("B")"#));
}

/// Nucleoid rejects value as a property name
#[test]
fn rejects_value_as_a_property_name() {
    let mut run = runner();
    run(r#"class Schedule:
    pass"#);
    run(r#"class Place:
    pass"#);
    run(r#"value = Schedule()"#);
    assert_eq!(run(r#"value"#), run(r#"({ "id": "value" })"#));
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    value.value = Place()
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = TypeError("Cannot use 'value' as a property")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid rejects value as a property name in a value assignment
#[test]
fn rejects_value_as_a_property_name_in_a_value_assignment() {
    let mut run = runner();
    run(r#"class Value:
    pass"#);
    run(r#"value = Value()"#);
    assert_eq!(run(r#"value"#), run(r#"({ "id": "value" })"#));
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    value.value = 2147483647
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = TypeError("Cannot use 'value' as a property")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid uses value property to indicate using only value of property
#[test]
fn uses_value_property_to_indicate_using_only_value_of_property() {
    let mut run = runner();
    run(r#"class Weight:
    pass"#);
    run(r#"weight1 = Weight()"#);
    run(r#"weight1.gravity = 1.352"#);
    run(r#"weight1.mass = 1000"#);
    run(r#"weight1.force = weight1.gravity * weight1.mass.value"#);
    assert_eq!(run(r#"weight1.force"#), run(r#"(1352)"#));
    run(r#"weight1.mass = 2000"#);
    assert_eq!(run(r#"weight1.force"#), run(r#"(1352)"#));
    run(r#"weight1.gravity = 2"#);
    assert_eq!(run(r#"weight1.force"#), run(r#"(2000)"#));
}

/// Nucleoid uses value property in an if condition to indicate using only value of property
#[test]
fn uses_value_property_in_an_if_condition_to_indicate_using_only_value_of_property() {
    let mut run = runner();
    run(r#"class Question:
    pass"#);
    run(r#"question1 = Question()"#);
    run(r#"question1.text = "How was the service?""#);
    run(r#"if question1.text != question1.text.value:
    throw "QUESTION_ARCHIVED""#);
    assert_eq!(run(r#"question1.text"#), run(r#"("How was the service?")"#));
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    question1.text = "How would you rate us?"
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = "QUESTION_ARCHIVED"
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid rejects value of a property if the property is not defined
#[test]
fn rejects_value_of_a_property_if_the_property_is_not_defined() {
    let mut run = runner();
    run(r#"class Travel:
    pass"#);
    run(r#"travel1 = Travel()"#);
    run(r#"travel1.speed = 65"#);
    run(r#"travel1.duration = travel1.distance / travel1.speed"#);
    assert_eq!(run(r#"travel1.duration"#), run(r#"(null)"#));
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    travel1.time = travel1.distance.value / travel1.speed
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("travel1.distance is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid uses the value of a null property as zero
#[test]
fn uses_the_value_of_a_null_property_as_zero() {
    let mut run = runner();
    run(r#"class Interest:
    pass"#);
    run(r#"interest1 = Interest()"#);
    run(r#"interest1.rate = 3"#);
    run(r#"interest1.amount = null"#);
    run(r#"interest1.annual = interest1.rate * interest1.amount.value / 100"#);
    assert_eq!(run(r#"interest1.annual"#), run(r#"(0)"#));
    run(r#"interest1.amount = 10000"#);
    assert_eq!(run(r#"interest1.annual"#), run(r#"(0)"#));
}

/// Nucleoid rejects value as a property name in a block
#[test]
fn rejects_value_as_a_property_name_in_a_block() {
    let mut run = runner();
    run(r#"class Alarm:
    pass"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    {
        value = Alarm()
        value.value = "22:00"
    }
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = TypeError("Cannot use 'value' as a property")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid keeps same as its value when the value property is used for a local
#[test]
fn keeps_same_as_its_value_when_the_value_property_is_used_for_a_local() {
    let mut run = runner();
    run(r#"speedOfLight = 299792"#);
    run(r#"roundTrip = null"#);
    run(r#"{
    time = speedOfLight / 225623
    roundTrip = time.value * 2
}"#);
    assert_eq!(run(r#"roundTrip"#), run(r#"(2.6574595675086314)"#));
}

/// Nucleoid uses value property in a class-level assignment
#[test]
fn uses_value_property_in_a_class_level_assignment() {
    let mut run = runner();
    run(r#"count = 0"#);
    run(r#"class Device:
    pass"#);
    run(r#"device1 = Device()"#);
    run(r#"{
    $Device.code = "A" + count.value
    count = count + 1
}"#);
    assert_eq!(run(r#"device1.code"#), run(r#"("A0")"#));
}

/// Nucleoid uses value property on a class-level property chain
#[test]
fn uses_value_property_on_a_class_level_property_chain() {
    let mut run = runner();
    run(r#"class Summary(question):
    this.question = question"#);
    run(r#"class Question:
    pass"#);
    run(r#"$Summary.count = $Summary.question.count.value"#);
    run(r#"question1 = Question()"#);
    run(r#"question1.count = 10"#);
    run(r#"summary1 = Summary(question1)"#);
    assert_eq!(run(r#"summary1.count"#), run(r#"(10)"#));
    run(r#"question1.count = 11"#);
    assert_eq!(run(r#"question1.count"#), run(r#"(11)"#));
    assert_eq!(run(r#"summary1.count"#), run(r#"(10)"#));
}

/// Nucleoid updates if block of property
#[test]
fn updates_if_block_of_property() {
    let mut run = runner();
    run(r#"class Account:
    pass"#);
    run(r#"account1 = Account()"#);
    run(r#"account1.balance = 1000"#);
    run(r#"if account1.balance < 1500:
    account1.status = "OK""#);
    assert_eq!(run(r#"account1.status"#), run(r#"("OK")"#));
    run(r#"if account1.balance < 1500:
    account1.status = "LOW""#);
    assert_eq!(run(r#"account1.status"#), run(r#"("LOW")"#));
}

/// Nucleoid creates an else statement of variable
#[test]
fn creates_an_else_statement_of_variable() {
    let mut run = runner();
    run(r#"compound = 0.0001"#);
    run(r#"acidic = 'ACIDIC'"#);
    run(r#"basic = 'BASIC'"#);
    run(r#"if compound > 0.0000001:
    pH = acidic
else:
    pH = basic"#);
    assert_eq!(run(r#"pH"#), run(r#"("ACIDIC")"#));
    run(r#"compound = 0.000000001"#);
    assert_eq!(run(r#"pH"#), run(r#"("BASIC")"#));
    run(r#"basic = '+7'"#);
    assert_eq!(run(r#"pH"#), run(r#"("+7")"#));
}

/// Nucleoid creates if statement of property
#[test]
fn creates_if_statement_of_property() {
    let mut run = runner();
    run(r#"class Toy:
    pass"#);
    run(r#"toy1 = Toy()"#);
    run(r#"toy1.color = "BLUE""#);
    run(r#"if toy1.color == "RED":
    toy1.shape = "CIRCLE""#);
    assert_eq!(run(r#"toy1.shape"#), run(r#"(null)"#));
    run(r#"toy1.color = "RED""#);
    assert_eq!(run(r#"toy1.shape"#), run(r#"("CIRCLE")"#));
}

/// Nucleoid creates else statement of property
#[test]
fn creates_else_statement_of_property() {
    let mut run = runner();
    run(r#"class Engine:
    pass"#);
    run(r#"engine1 = Engine()"#);
    run(r#"engine1.type = "V8""#);
    run(r#"mpl = "MPL""#);
    run(r#"bsd = "BSD""#);
    run(r#"if engine1.type == "Gecko":
    engine1.license = mpl
else:
    engine1.license = bsd"#);
    assert_eq!(run(r#"engine1.license"#), run(r#"("BSD")"#));
    run(r#"bsd = "Berkeley Software Distribution""#);
    assert_eq!(run(r#"engine1.license"#), run(r#"("Berkeley Software Distribution")"#));
    run(r#"engine1.type = "Gecko""#);
    assert_eq!(run(r#"engine1.license"#), run(r#"("MPL")"#));
}

/// Nucleoid creates else statement of property with property dependencies
#[test]
fn creates_else_statement_of_property_with_property_dependencies() {
    let mut run = runner();
    run(r#"class Contact:
    pass"#);
    run(r#"contact1 = Contact()"#);
    run(r#"contact1.type = "PERSON""#);
    run(r#"contact1.first = "First""#);
    run(r#"contact1.last = "Last""#);
    run(r#"if contact1.type == "BUSINESS":
    contact1.full = "B" + contact1.first
else:
    contact1.full = contact1.first + " " + contact1.last"#);
    assert_eq!(run(r#"contact1.full"#), run(r#"("First Last")"#));
    run(r#"contact1.first = "F""#);
    run(r#"contact1.last = "L""#);
    assert_eq!(run(r#"contact1.full"#), run(r#"("F L")"#));
    run(r#"contact1.type = "BUSINESS""#);
    assert_eq!(run(r#"contact1.full"#), run(r#"("BF")"#));
}

/// Nucleoid creates multiple else if statement of property
#[test]
fn creates_multiple_else_if_statement_of_property() {
    let mut run = runner();
    run(r#"class Taxpayer:
    pass"#);
    run(r#"taxpayer1 = Taxpayer()"#);
    run(r#"taxpayer1.income = 60000"#);
    run(r#"taxpayer1.member = 1"#);
    run(r#"rate = 22"#);
    run(r#"if taxpayer1.member > 4:
    taxpayer1.tax = taxpayer1.income * rate / 100 - 2000
else if taxpayer1.member > 2:
    taxpayer1.tax = taxpayer1.income * rate / 100 - 1000
else:
    taxpayer1.tax = taxpayer1.income * rate / 100"#);
    assert_eq!(run(r#"taxpayer1.tax"#), run(r#"(13200)"#));
    run(r#"rate = 23"#);
    assert_eq!(run(r#"taxpayer1.tax"#), run(r#"(13800)"#));
    run(r#"taxpayer1.member = 3"#);
    assert_eq!(run(r#"taxpayer1.tax"#), run(r#"(12800)"#));
    run(r#"taxpayer1.member = 5"#);
    assert_eq!(run(r#"taxpayer1.tax"#), run(r#"(11800)"#));
}

/// Nucleoid updates property assignment
#[test]
fn updates_property_assignment() {
    let mut run = runner();
    run(r#"class Matter:
    pass"#);
    run(r#"matter1 = Matter()"#);
    run(r#"matter1.mass = 10"#);
    run(r#"matter1.weight = matter1.mass * 9.8"#);
    assert_eq!(run(r#"matter1.weight"#), run(r#"(98)"#));
    run(r#"matter1.weight = matter1.mass * 3.7"#);
    assert_eq!(run(r#"matter1.weight"#), run(r#"(37)"#));
    run(r#"matter1.mass = 20"#);
    assert_eq!(run(r#"matter1.weight"#), run(r#"(74)"#));
}

/// Nucleoid deletes an instance
#[test]
fn deletes_an_instance() {
    let mut run = runner();
    run(r#"class Circle:
    pass"#);
    run(r#"circle1 = Circle()"#);
    run(r#"delete circle1"#);
    assert_eq!(run(r#"Circle["circle1"]"#), run(r#"(null)"#));
    assert_eq!(run(r#"Circle.find(circle => circle.id == "circle1")"#), run(r#"(null)"#));
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    circle1
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("circle1 is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid deletes an instance by reference
#[test]
fn deletes_an_instance_by_reference() {
    let mut run = runner();
    run(r#"class Item:
    pass"#);
    run(r#"item1 = Item()"#);
    run(r#"item2 = Item()"#);
    assert_eq!(run(r#"Item["item1"]"#), run(r#"({ "id": "item1" })"#));
    run(r#"delete Item["item1"]"#);
    assert_eq!(run(r#"Item["item1"]"#), run(r#"(null)"#));
    assert_eq!(run(r#"Item["item2"]"#), run(r#"({ "id": "item2" })"#));
    run(r#"{
    item = "item2"
    delete Item[item]
}"#);
    assert_eq!(run(r#"Item["item2"]"#), run(r#"(null)"#));
}

/// Nucleoid returns a boolean when deleting an object
#[test]
fn returns_a_boolean_when_deleting_an_object() {
    let mut run = runner();
    run(r#"class Location:
    pass"#);
    run(r#"location1 = Location()"#);
    assert_eq!(run(r#"delete location1"#), run(r#"(true)"#));
    assert_eq!(run(r#"delete location2"#), run(r#"(false)"#));
}

/// Nucleoid rejects deleting an instance if it has any properties
#[test]
fn rejects_deleting_an_instance_if_it_has_any_properties() {
    let mut run = runner();
    run(r#"class Channel:
    pass"#);
    run(r#"channel1 = Channel()"#);
    run(r#"channel1.frequency = 440"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    delete channel1
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = TypeError("Cannot delete object 'channel1'")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
    assert_eq!(run(r#"channel1.frequency"#), run(r#"(440)"#));
    run(r#"delete channel1.frequency"#);
    run(r#"delete channel1"#);
    assert_eq!(run(r#"Channel["channel1"]"#), run(r#"(null)"#));
}

/// Nucleoid rejects deleting an instance if it has an object as a property
#[test]
fn rejects_deleting_an_instance_if_it_has_an_object_as_a_property() {
    let mut run = runner();
    run(r#"class Shape:
    pass"#);
    run(r#"class Type:
    pass"#);
    run(r#"shape1 = Shape()"#);
    run(r#"shape1.type = Type()"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    delete shape1
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = TypeError("Cannot delete object 'shape1'")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
    run(r#"delete shape1.type"#);
    run(r#"delete shape1"#);
    assert_eq!(run(r#"Shape["shape1"]"#), run(r#"(null)"#));
}

/// Nucleoid deletes a property assignment
#[test]
fn deletes_a_property_assignment() {
    let mut run = runner();
    run(r#"class Agent:
    pass"#);
    run(r#"agent = Agent()"#);
    run(r#"agent.time = 52926163455"#);
    run(r#"agent.location = "CITY""#);
    run(r#"agent.report = agent.time + "@" + agent.location"#);
    assert_eq!(run(r#"agent.report"#), run(r#"("52926163455@CITY")"#));
    run(r#"delete agent.time"#);
    assert_eq!(run(r#"agent.report"#), run(r#"(null)"#));
    run(r#"delete agent.report"#);
    assert_eq!(run(r#"agent.report"#), run(r#"(null)"#));
}

/// Nucleoid runs a block statement of property
#[test]
fn runs_a_block_statement_of_property() {
    let mut run = runner();
    run(r#"class Item:
    pass"#);
    run(r#"item1 = Item()"#);
    run(r#"item1.sku = "0000001""#);
    run(r#"{
    custom = "US" + item1.sku
    item1.custom = custom
}"#);
    assert_eq!(run(r#"item1.custom"#), run(r#"("US0000001")"#));
    run(r#"item1.sku = "0000002""#);
    assert_eq!(run(r#"item1.custom"#), run(r#"("US0000002")"#));
}

/// Nucleoid runs a nested block statement of property
#[test]
fn runs_a_nested_block_statement_of_property() {
    let mut run = runner();
    run(r#"class Figure:
    pass"#);
    run(r#"figure1 = Figure()"#);
    run(r#"figure1.width = 9"#);
    run(r#"figure1.height = 10"#);
    run(r#"{
    base = Math.pow(figure1.width, 2)
    {
        figure1.volume = base * figure1.height
    }
}"#);
    assert_eq!(run(r#"figure1.volume"#), run(r#"(810)"#));
    run(r#"figure1.height = 9"#);
    assert_eq!(run(r#"figure1.volume"#), run(r#"(729)"#));
}

/// Nucleoid runs a nested if statement of property
#[test]
fn runs_a_nested_if_statement_of_property() {
    let mut run = runner();
    run(r#"class Sale:
    pass"#);
    run(r#"sale1 = Sale()"#);
    run(r#"sale1.price = 50"#);
    run(r#"sale1.quantity = 2"#);
    run(r#"{
    amount = sale1.price * sale1.quantity
    if amount > 100:
        sale1.tax = amount * 10 / 100
}"#);
    assert_eq!(run(r#"sale1.tax"#), run(r#"(null)"#));
    run(r#"sale1.quantity = 3"#);
    assert_eq!(run(r#"sale1.tax"#), run(r#"(15)"#));
}

/// Nucleoid creates a nested else statement of property
#[test]
fn creates_a_nested_else_statement_of_property() {
    let mut run = runner();
    run(r#"class Chart:
    pass"#);
    run(r#"chart1 = Chart()"#);
    run(r#"chart1.percentage = 1"#);
    run(r#"invalid = "INVALID""#);
    run(r#"valid = "VALID""#);
    run(r#"{
    ratio = chart1.percentage / 100
    if ratio > 1:
        chart1.status = invalid
    else:
        chart1.status = valid
}"#);
    assert_eq!(run(r#"chart1.status"#), run(r#"("VALID")"#));
    run(r#"valid = "V""#);
    assert_eq!(run(r#"chart1.status"#), run(r#"("V")"#));
}

/// Nucleoid creates a property assignment with multiple properties
#[test]
fn creates_a_property_assignment_with_multiple_properties() {
    let mut run = runner();
    run(r#"class Person:
    pass"#);
    run(r#"person1 = Person()"#);
    run(r#"class Address:
    pass"#);
    run(r#"address1 = Address()"#);
    run(r#"$Address.print = $Address.city + ", " + $Address.state"#);
    run(r#"person1.address = Address()"#);
    run(r#"person1.address.city = "Syracuse""#);
    run(r#"person1.address.state = "NY""#);
    assert_eq!(run(r#"person1.address.print"#), run(r#"("Syracuse, NY")"#));
}

/// Nucleoid creates a property assignment with multiple properties as part of a declaration
#[test]
fn creates_a_property_assignment_with_multiple_properties_as_part_of_a_declaration() {
    let mut run = runner();
    run(r#"class Server:
    pass"#);
    run(r#"server1 = Server()"#);
    run(r#"server1.name = "HOST1""#);
    run(r#"class IP:
    pass"#);
    run(r#"ip1 = IP()"#);
    run(r#"server1.ip = ip1"#);
    run(r#"ip1.address = "10.0.0.1""#);
    run(r#"server1.summary = server1.name + "@" + server1.ip.address"#);
    assert_eq!(run(r#"server1.summary"#), run(r#"("HOST1@10.0.0.1")"#));
    run(r#"ip1.address = "10.0.0.2""#);
    assert_eq!(run(r#"server1.summary"#), run(r#"("HOST1@10.0.0.2")"#));
}

/// Nucleoid creates a dependency on behalf if a property has a reference
#[test]
fn creates_a_dependency_on_behalf_if_a_property_has_a_reference() {
    let mut run = runner();
    run(r#"class Schedule:
    pass"#);
    run(r#"schedule1 = Schedule()"#);
    run(r#"class Template:
    pass"#);
    run(r#"template1 = Template()"#);
    run(r#"template1.type = "W""#);
    run(r#"schedule1.template = template1"#);
    run(r#"schedule1.template.name = schedule1.template.type + "-0001""#);
    assert_eq!(run(r#"template1.name"#), run(r#"("W-0001")"#));
    assert_eq!(run(r#"schedule1.template.name"#), run(r#"("W-0001")"#));
    run(r#"template1.type = "D""#);
    assert_eq!(run(r#"template1.name"#), run(r#"("D-0001")"#));
    run(r#"template1.shape = template1.type + "-Form""#);
    assert_eq!(run(r#"template1.shape"#), run(r#"("D-Form")"#));
    assert_eq!(run(r#"schedule1.template.shape"#), run(r#"("D-Form")"#));
    run(r#"template1.type = "C""#);
    assert_eq!(run(r#"template1.shape"#), run(r#"("C-Form")"#));
    assert_eq!(run(r#"schedule1.template.shape"#), run(r#"("C-Form")"#));
}

/// Nucleoid creates a dependency on behalf if a local variable has a reference
#[test]
fn creates_a_dependency_on_behalf_if_a_local_variable_has_a_reference() {
    let mut run = runner();
    run(r#"class Vote:
    pass"#);
    run(r#"vote1 = Vote()"#);
    run(r#"vote1.rate = 4"#);
    run(r#"class Question:
    pass"#);
    run(r#"$Question.rate = 0"#);
    run(r#"$Question.count = 0"#);
    run(r#"question1 = Question()"#);
    run(r#"vote1.question = question1"#);
    run(r#"{
    question = vote1.question
    question.rate = (question.rate * question.count + vote1.rate) / (question.count + 1)
    question.count = question.count + 1
}"#);
    assert_eq!(run(r#"question1.rate"#), run(r#"(4)"#));
    assert_eq!(run(r#"question1.count"#), run(r#"(1)"#));
    run(r#"vote1.rate = 5"#);
    assert_eq!(run(r#"question1.rate"#), run(r#"(4.5)"#));
}

/// Nucleoid runs an expression statement of class
#[test]
fn runs_an_expression_statement_of_class() {
    let mut run = runner();
    run(r#"class Element:
    pass"#);
    run(r#"alkalis = []"#);
    run(r#"element1 = Element()"#);
    run(r#"element1.number = 3"#);
    run(r#"{
    number = $Element.number
    if number == 3:
        alkalis.push($Element)
}"#);
    assert_eq!(run(r#"alkalis.pop()"#), run(r#"(element1)"#));
}

/// Nucleoid rejects a variable declaration without definition
#[test]
fn rejects_a_variable_declaration_without_definition() {
    let mut run = runner();
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    a: int
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("Missing definition")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid creates a dependency based on the length of an identifier
#[test]
fn creates_a_dependency_based_on_the_length_of_an_identifier() {
    let mut run = runner();
    run(r#"str1 = "ABC""#);
    run(r#"i1 = str1.length + 1"#);
    assert_eq!(run(r#"i1"#), run(r#"(4)"#));
    run(r#"str1 = "ABCD""#);
    assert_eq!(run(r#"i1"#), run(r#"(5)"#));
    run(r#"if str1.length > 5:
    i2 = i1"#);
    run(r#"str1 = "ABCDEF""#);
    assert_eq!(run(r#"i2"#), run(r#"(7)"#));
}

/// Nucleoid adds a created class to the class list
#[test]
fn adds_a_created_class_to_the_class_list() {
    let mut run = runner();
    assert_eq!(run(r#"Class.length"#), run(r#"(0)"#));
    run(r#"class Student:
    pass"#);
    assert_eq!(run(r#"Class.length"#), run(r#"(1)"#));
    run(r#"class User:
    pass"#);
    assert_eq!(run(r#"Class.length"#), run(r#"(2)"#));
}

/// Nucleoid updates a class definition
#[test]
fn updates_a_class_definition() {
    let mut run = runner();
    run(r#"class Message:
    pass"#);
    run(r#"$Message.read = false"#);
    run(r#"message1 = Message()"#);
    run(r#"class Message(payload: str):
    this.payload = payload"#);
    assert_eq!(run(r#"message1.read"#), run(r#"(false)"#));
    assert_eq!(run(r#"message1.payload"#), run(r#"(null)"#));
    run(r#"message2 = Message("MESSAGE")"#);
    assert_eq!(run(r#"message2.read"#), run(r#"(false)"#));
    assert_eq!(run(r#"message2.payload"#), run(r#"("MESSAGE")"#));
}

/// Nucleoid supports a string in an expression
#[test]
fn supports_a_string_in_an_expression() {
    let mut run = runner();
    assert_eq!(run(r#"'New String'"#), run(r#"("New String")"#));
    assert_eq!(run(r#""New String""#), run(r#"("New String")"#));
    assert_eq!(run(r#"`New String`"#), run(r#"("New String")"#));
    run(r#"a = 123"#);
    assert_eq!(run(r#"`New ${a} String`"#), run(r#"("New 123 String")"#));
}

/// Nucleoid supports logical operators
#[test]
fn supports_logical_operators() {
    let mut run = runner();
    run(r#"condition = false"#);
    assert_eq!(run(r#"condition or true"#), run(r#"(true)"#));
    assert_eq!(run(r#"condition || true"#), run(r#"(true)"#));
    assert_eq!(run(r#"not condition and true"#), run(r#"(true)"#));
    assert_eq!(run(r#"!condition && true"#), run(r#"(true)"#));
}

/// Nucleoid supports standard built-in objects
#[test]
fn supports_standard_built_in_objects() {
    let mut run = runner();
    run(r#"max = Number.MAX_INTEGER"#);
    assert_eq!(run(r#"max"#), run(r#"(9007199254740991)"#));
    run(r#"now = Date.now()"#);
    assert_eq!(run(r#"now > 0"#), run(r#"(true)"#));
}

/// Nucleoid supports creating standard built-in objects
#[test]
fn supports_creating_standard_built_in_objects() {
    let mut run = runner();
    run(r#"date = Date("2019-7-24")"#);
    assert_eq!(run(r#"date.getYear()"#), run(r#"(119)"#));
}

/// Nucleoid supports built-in objects
#[test]
fn supports_built_in_objects() {
    let mut run = runner();
    run(r#"date1 = Date()"#);
    run(r#"date2 = Date(date1.getTime())"#);
    assert_eq!(run(r#"date1.getTime() == date2.getTime()"#), run(r#"(true)"#));
    run(r#"date3 = Date.parse("04 Dec 1995 00:12:00 GMT")"#);
    assert_eq!(run(r#"date3"#), run(r#"(818035920000)"#));
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    date4 = Date.wrong()
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = TypeError("Date.wrong is not a function")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid calls a function with no return
#[test]
fn calls_a_function_with_no_return() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"def copy(val):
    b = val"#);
    run(r#"copy(a)

# return: null"#);
}

/// Nucleoid calls a function with a return value
#[test]
fn calls_a_function_with_a_return_value() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"def copy(val):
    b = val
    return val"#);
    run(r#"copy(a)

# return: 1"#);
}

/// Nucleoid supports a function in an expression
#[test]
fn supports_a_function_in_an_expression() {
    let mut run = runner();
    run(r#"list = [1, 2, 3]"#);
    assert_eq!(run(r#"list.find(function(element) { return element == 3 })"#), run(r#"(3)"#));
    assert_eq!(run(r#"list.find(element => { return element == 2 })"#), run(r#"(2)"#));
    assert_eq!(run(r#"list.find(element => element == 1)"#), run(r#"(1)"#));
    assert_eq!(run(r#"list.find(element => (element == 1))"#), run(r#"(1)"#));
}

/// Nucleoid supports a function with a parameter in an expression
#[test]
fn supports_a_function_with_a_parameter_in_an_expression() {
    let mut run = runner();
    run(r#"samples = [38.2, 39.1, 38.8, 39]"#);
    run(r#"ratio = 2.1"#);
    run(r#"element = 38.5"#);
    assert_eq!(run(r#"samples.find(function(element) { result = element * ratio; return result == 81.48 })"#), run(r#"(38.8)"#));
    assert_eq!(run(r#"samples.find(element => { result = element * ratio; return result == 81.48 })"#), run(r#"(38.8)"#));
    assert_eq!(run(r#"samples.find(element => element == 38.8)"#), run(r#"(38.8)"#));
    assert_eq!(run(r#"samples.find(element => (element == 38.8))"#), run(r#"(38.8)"#));
}

/// Nucleoid creates a variable statement with JSON
#[test]
fn creates_a_variable_statement_with_json() {
    let mut run = runner();
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run("__nucleoid_test_assertion_1_actual = null");
    run("__nucleoid_test_assertion_1_expected = null");
    run("__nucleoid_test_assertion_1_ran = false");
    run(r#"{
    payload = { "data": "TEST", "nested": { "data": "NESTED_TEST" } }
    __nucleoid_test_assertion_0_actual = payload.data
    __nucleoid_test_assertion_0_expected = "TEST"
    __nucleoid_test_assertion_0_ran = true
    __nucleoid_test_assertion_1_actual = payload.nested.data
    __nucleoid_test_assertion_1_expected = "NESTED_TEST"
    __nucleoid_test_assertion_1_ran = true
}"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
    assert_eq!(run("__nucleoid_test_assertion_1_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_1_actual"), run("(__nucleoid_test_assertion_1_expected)"));
    run(r#"message = { "pid": 1200 }"#);
    assert_eq!(run(r#"message.pid"#), run(r#"(1200)"#));
    run("__nucleoid_test_assertion_2_actual = null");
    run("__nucleoid_test_assertion_2_expected = null");
    run("__nucleoid_test_assertion_2_ran = false");
    run(r#"{
    scope = { "query": "test" }
    i = { "test": scope.query }
    __nucleoid_test_assertion_2_actual = i.test
    __nucleoid_test_assertion_2_expected = "test"
    __nucleoid_test_assertion_2_ran = true
}"#);
    assert_eq!(run("__nucleoid_test_assertion_2_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_2_actual"), run("(__nucleoid_test_assertion_2_expected)"));
}

/// Nucleoid returns an inline JSON object
#[test]
fn returns_an_inline_json_object() {
    let mut run = runner();
    run(r#"{
    return { "number": 123, "string": "ABC", "bool": true }
}

# return: { "number": 123, "string": "ABC", "bool": true }"#);
}

/// Nucleoid returns an inline JSON array
#[test]
fn returns_an_inline_json_array() {
    let mut run = runner();
    run(r#"{
    return [{ "number": 123, "string": "ABC", "bool": true }]
}

# return: [{ "number": 123, "string": "ABC", "bool": true }]"#);
}

/// Nucleoid returns an inline object
#[test]
fn returns_an_inline_object() {
    let mut run = runner();
    run(r#"{
    return { number: 123, string: "ABC", bool: true }
}

# return: { "number": 123, "string": "ABC", "bool": true }"#);
}

/// Nucleoid returns an inline array
#[test]
fn returns_an_inline_array() {
    let mut run = runner();
    run(r#"{
    return [{ number: 123, string: "ABC", bool: true }]
}

# return: [{ "number": 123, "string": "ABC", "bool": true }]"#);
}

/// Nucleoid supports nested functions as a parameter in an expression
#[test]
fn supports_nested_functions_as_a_parameter_in_an_expression() {
    let mut run = runner();
    run(r#"name = "AbCDE""#);
    run(r#"pointer = 0"#);
    run(r#"if not /[A-Z]/.test(name.charAt(pointer)):
    throw "INVALID_FIRST_CHARACTER""#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    name = "bbCDE"
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = "INVALID_FIRST_CHARACTER"
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
    run(r#"name = "CbCDE""#);
    run("__nucleoid_test_assertion_1_actual = null");
    run("__nucleoid_test_assertion_1_expected = null");
    run("__nucleoid_test_assertion_1_ran = false");
    run(r#"try:
    pointer = 1
catch error:
    __nucleoid_test_assertion_1_actual = error
    __nucleoid_test_assertion_1_expected = "INVALID_FIRST_CHARACTER"
    __nucleoid_test_assertion_1_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_1_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_1_actual"), run("(__nucleoid_test_assertion_1_expected)"));
}

/// Nucleoid supports a property of chained functions in an expression
#[test]
fn supports_a_property_of_chained_functions_in_an_expression() {
    let mut run = runner();
    run(r#"class User:
    pass"#);
    run(r#"class Registration:
    pass"#);
    run(r#"user1 = User()"#);
    run(r#"registration1 = Registration()"#);
    run(r#"registration1.user = user1"#);
    run(r#"registration2 = Registration()"#);
    run(r#"registration2.user = user1"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    if Registration.filter(r => r.user == $User).length > 1:
        throw "USER_ALREADY_REGISTERED"
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = "USER_ALREADY_REGISTERED"
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid throws an error as a string
#[test]
fn throws_an_error_as_a_string() {
    let mut run = runner();
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    throw 'INVALID'
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = "INVALID"
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
    run("__nucleoid_test_assertion_1_actual = null");
    run("__nucleoid_test_assertion_1_expected = null");
    run("__nucleoid_test_assertion_1_ran = false");
    run(r#"try:
    throw "INVALID"
catch error:
    __nucleoid_test_assertion_1_actual = error
    __nucleoid_test_assertion_1_expected = "INVALID"
    __nucleoid_test_assertion_1_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_1_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_1_actual"), run("(__nucleoid_test_assertion_1_expected)"));
}

/// Nucleoid throws an error as an integer
#[test]
fn throws_an_error_as_an_integer() {
    let mut run = runner();
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    throw 123
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = 123
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid throws a reference error if the thrown value is not defined
#[test]
fn throws_a_reference_error_if_the_thrown_value_is_not_defined() {
    let mut run = runner();
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    throw abc
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("abc is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid creates a class assignment before initialization
#[test]
fn creates_a_class_assignment_before_initialization() {
    let mut run = runner();
    run(r#"class Review:
    pass"#);
    run(r#"$Review.rate = $Review.sum / 10"#);
    run(r#"review1 = Review()"#);
    assert_eq!(run(r#"review1.rate"#), run(r#"(null)"#));
    run(r#"review1.sum = 42"#);
    assert_eq!(run(r#"review1.rate"#), run(r#"(4.2)"#));
}

/// Nucleoid creates a class assignment after initialization
#[test]
fn creates_a_class_assignment_after_initialization() {
    let mut run = runner();
    run(r#"class Shape:
    pass"#);
    run(r#"shape1 = Shape()"#);
    run(r#"shape1.edge = 3"#);
    run(r#"shape2 = Shape()"#);
    run(r#"shape2.edge = 3"#);
    run(r#"$Shape.angle = ($Shape.edge - 2) * 180"#);
    assert_eq!(run(r#"shape1.angle"#), run(r#"(180)"#));
    assert_eq!(run(r#"shape2.angle"#), run(r#"(180)"#));
    run(r#"shape1.edge = 4"#);
    assert_eq!(run(r#"shape1.angle"#), run(r#"(360)"#));
    assert_eq!(run(r#"shape2.angle"#), run(r#"(180)"#));
}

/// Nucleoid updates a class assignment
#[test]
fn updates_a_class_assignment() {
    let mut run = runner();
    run(r#"class Employee:
    pass"#);
    run(r#"employee1 = Employee()"#);
    run(r#"employee1.id = 1"#);
    run(r#"$Employee.username = "E" + $Employee.id"#);
    assert_eq!(run(r#"employee1.username"#), run(r#"("E1")"#));
    run(r#"$Employee.username = "F" + $Employee.id"#);
    assert_eq!(run(r#"employee1.username"#), run(r#"("F1")"#));
    run(r#"employee1.id = 2"#);
    assert_eq!(run(r#"employee1.username"#), run(r#"("F2")"#));
}

/// Nucleoid creates an if statement of class before initialization
#[test]
fn creates_an_if_statement_of_class_before_initialization() {
    let mut run = runner();
    run(r#"class Ticket:
    pass"#);
    run(r#"if $Ticket.date > Date("1993-1-1"):
    $Ticket.status = "EXPIRED""#);
    run(r#"ticket1 = Ticket()"#);
    assert_eq!(run(r#"ticket1.status"#), run(r#"(null)"#));
    run(r#"ticket1.date = Date("1993-2-1")"#);
    assert_eq!(run(r#"ticket1.status"#), run(r#"("EXPIRED")"#));
    run(r#"ticket2 = Ticket()"#);
    assert_eq!(run(r#"ticket2.status"#), run(r#"(null)"#));
}

/// Nucleoid creates an if statement of class after initialization
#[test]
fn creates_an_if_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Student:
    pass"#);
    run(r#"student1 = Student()"#);
    run(r#"student1.age = 2"#);
    run(r#"student1.class = "Daycare""#);
    run(r#"student2 = Student()"#);
    run(r#"student2.age = 2"#);
    run(r#"student2.class = "Daycare""#);
    run(r#"if $Student.age == 3:
    $Student.class = "Preschool""#);
    assert_eq!(run(r#"student1.class"#), run(r#"("Daycare")"#));
    assert_eq!(run(r#"student2.class"#), run(r#"("Daycare")"#));
    run(r#"student1.age = 3"#);
    assert_eq!(run(r#"student1.class"#), run(r#"("Preschool")"#));
    assert_eq!(run(r#"student2.class"#), run(r#"("Daycare")"#));
}

/// Nucleoid updates an if block of class
#[test]
fn updates_an_if_block_of_class() {
    let mut run = runner();
    run(r#"class Inventory:
    pass"#);
    run(r#"inventory1 = Inventory()"#);
    run(r#"inventory1.quantity = 0"#);
    run(r#"inventory2 = Inventory()"#);
    run(r#"inventory2.quantity = 1000"#);
    run(r#"if $Inventory.quantity == 0:
    $Inventory.replenishment = true"#);
    assert_eq!(run(r#"inventory1.replenishment"#), run(r#"(true)"#));
    assert_eq!(run(r#"inventory2.replenishment"#), run(r#"(null)"#));
    run(r#"if $Inventory.quantity == 0:
    $Inventory.replenishment = false"#);
    assert_eq!(run(r#"inventory1.replenishment"#), run(r#"(false)"#));
    assert_eq!(run(r#"inventory2.replenishment"#), run(r#"(null)"#));
}

/// Nucleoid creates an else statement of class before initialization
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
    run(r#"count1 = Count()"#);
    run(r#"count1.max = 850"#);
    assert_eq!(run(r#"count1.reset"#), run(r#"("REGULAR")"#));
    run(r#"regular = "R""#);
    assert_eq!(run(r#"count1.reset"#), run(r#"("R")"#));
}

/// Nucleoid creates an else statement of class after initialization
#[test]
fn creates_an_else_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Concentration:
    pass"#);
    run(r#"serialDilution = "(c1V1+c2V2)/(V1+V2)""#);
    run(r#"directDilution = "c1/V1""#);
    run(r#"concentration1 = Concentration()"#);
    run(r#"concentration1.substances = 2"#);
    run(r#"if $Concentration.substances == 1:
    $Concentration.formula = directDilution
else:
    $Concentration.formula = serialDilution"#);
    assert_eq!(run(r#"concentration1.formula"#), run(r#"("(c1V1+c2V2)/(V1+V2)")"#));
    run(r#"serialDilution = "(c1V1+c2V2+c3V3)/(V1+V2+V3)""#);
    assert_eq!(run(r#"concentration1.formula"#), run(r#"("(c1V1+c2V2+c3V3)/(V1+V2+V3)")"#));
}

/// Nucleoid creates an else if statement of class before initialization
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
    run(r#"storage1 = Storage()"#);
    run(r#"storage1.capacity = 23"#);
    assert_eq!(run(r#"storage1.status"#), run(r#"("LOW")"#));
    run(r#"low = "L""#);
    assert_eq!(run(r#"storage1.status"#), run(r#"("L")"#));
}

/// Nucleoid creates an else if statement of class after initialization
#[test]
fn creates_an_else_if_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Registration:
    pass"#);
    run(r#"yes = "YES"; pending = "PENDING"; no = "NO""#);
    run(r#"registration1 = Registration()"#);
    run(r#"registration1.available = 0"#);
    run(r#"if $Registration.available > 10:
    $Registration.accepted = yes
else if $Registration.available > 0:
    $Registration.accepted = pending
else:
    $Registration.accepted = no"#);
    assert_eq!(run(r#"registration1.accepted"#), run(r#"("NO")"#));
    run(r#"yes = true; no = false"#);
    assert_eq!(run(r#"registration1.accepted"#), run(r#"(false)"#));
}

/// Nucleoid creates multiple else if statement of class before initialization
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
    run(r#"capacity1 = Capacity()"#);
    run(r#"capacity1.available = 100"#);
    run(r#"capacity1.spare = 5"#);
    assert_eq!(run(r#"capacity1.total"#), run(r#"(115)"#));
    run(r#"capacity1.spare = 1"#);
    assert_eq!(run(r#"capacity1.total"#), run(r#"(103)"#));
}

/// Nucleoid creates multiple else if statement of class after initialization
#[test]
fn creates_multiple_else_if_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Shape:
    pass"#);
    run(r#"shape1 = Shape()"#);
    run(r#"shape1.type = "RECTANGLE""#);
    run(r#"shape1.x = 5"#);
    run(r#"shape1.y = 6"#);
    run(r#"if $Shape.type == "SQUARE":
    $Shape.area = Math.pow($Shape.x, 2)
else if $Shape.type == "TRIANGLE":
    $Shape.area = $Shape.x * $Shape.y / 2
else:
    $Shape.area = $Shape.x * $Shape.y"#);
    assert_eq!(run(r#"shape1.area"#), run(r#"(30)"#));
    run(r#"shape1.x = 7"#);
    assert_eq!(run(r#"shape1.area"#), run(r#"(42)"#));
}

/// Nucleoid runs a block statement of class before initialization
#[test]
fn runs_a_block_statement_of_class_before_initialization() {
    let mut run = runner();
    run(r#"class Stock:
    pass"#);
    run(r#"{
    change = $Stock.before * 4 / 100
    $Stock.after = $Stock.before + change
}"#);
    run(r#"stock1 = Stock()"#);
    assert_eq!(run(r#"stock1.after"#), run(r#"(null)"#));
    run(r#"stock1.before = 57.25"#);
    assert_eq!(run(r#"stock1.after"#), run(r#"(59.54)"#));
    run(r#"stock1.before = 59.5"#);
    assert_eq!(run(r#"stock1.after"#), run(r#"(61.88)"#));
}

/// Nucleoid runs a block statement of class after initialization
#[test]
fn runs_a_block_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Purchase:
    pass"#);
    run(r#"purchase1 = Purchase()"#);
    run(r#"purchase1.price = 99"#);
    run(r#"{
    retail = $Purchase.price * 1.15
    $Purchase.retailPrice = retail
}"#);
    assert_eq!(run(r#"purchase1.retailPrice"#), run(r#"(113.85)"#));
    run(r#"purchase1.price = 199"#);
    assert_eq!(run(r#"purchase1.retailPrice"#), run(r#"(228.85)"#));
}

/// Nucleoid runs a nested block statement of class before initialization
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
    run(r#"compound1 = Compound()"#);
    run(r#"compound1.substance = 55.85"#);
    run(r#"compound1.mol = 1000"#);
    assert_eq!(run(r#"compound1.sample"#), run(r#"(1252)"#));
}

/// Nucleoid runs a nested block statement of class after initialization
#[test]
fn runs_a_nested_block_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Bug:
    pass"#);
    run(r#"bug1 = Bug()"#);
    run(r#"bug1.initialScore = 1000"#);
    run(r#"bug1.aging = 24"#);
    run(r#"{
    score = $Bug.aging * 10
    {
        $Bug.priorityScore = score + $Bug.initialScore
    }
}"#);
    assert_eq!(run(r#"bug1.priorityScore"#), run(r#"(1240)"#));
}

/// Nucleoid runs a nested if statement of class before initialization
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
    run(r#"mortgage1 = Mortgage()"#);
    run(r#"mortgage1.annual = 46"#);
    assert_eq!(run(r#"mortgage1.rate"#), run(r#"("EXCEPTIONAL")"#));
    run(r#"rate1 = "E""#);
    assert_eq!(run(r#"mortgage1.rate"#), run(r#"("E")"#));
}

/// Nucleoid runs a nested if statement of class after initialization
#[test]
fn runs_a_nested_if_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Building:
    pass"#);
    run(r#"buildingType1 = "SKYSCRAPER""#);
    run(r#"building1 = Building()"#);
    run(r#"building1.floors = 20"#);
    run(r#"{
    height = $Building.floors * 14
    if height > 330:
        $Building.type = buildingType1
}"#);
    assert_eq!(run(r#"building1.type"#), run(r#"(null)"#));
    run(r#"building1.floors = 25"#);
    assert_eq!(run(r#"building1.type"#), run(r#"("SKYSCRAPER")"#));
    run(r#"buildingType1 = "S""#);
    assert_eq!(run(r#"building1.type"#), run(r#"("S")"#));
}

/// Nucleoid creates a nested else statement of class before initialization
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
    run(r#"account1 = Account()"#);
    run(r#"account1.balance = 950"#);
    assert_eq!(run(r#"account1.alert"#), run(r#"("LOW_ALERT")"#));
    run(r#"lowAlert = "L""#);
    assert_eq!(run(r#"account1.alert"#), run(r#"("L")"#));
}

/// Nucleoid creates a nested else statement of class after initialization
#[test]
fn creates_a_nested_else_statement_of_class_after_initialization() {
    let mut run = runner();
    run(r#"class Question:
    pass"#);
    run(r#"high = "HIGH""#);
    run(r#"low = "LOW""#);
    run(r#"question1 = Question()"#);
    run(r#"question1.count = 1"#);
    run(r#"{
    score = $Question.count * 10
    if score > 100:
        $Question.type = high
    else:
        $Question.type = low
}"#);
    assert_eq!(run(r#"question1.type"#), run(r#"("LOW")"#));
    run(r#"low = "L""#);
    assert_eq!(run(r#"question1.type"#), run(r#"("L")"#));
    run(r#"question1.count = 11"#);
    assert_eq!(run(r#"question1.type"#), run(r#"("HIGH")"#));
}

/// Nucleoid creates a class assignment with multiple properties before declaration
#[test]
fn creates_a_class_assignment_with_multiple_properties_before_declaration() {
    let mut run = runner();
    run(r#"class Room:
    pass"#);
    run(r#"$Room.level = $Room.number / 10"#);
    run(r#"class Guest:
    pass"#);
    run(r#"$Guest.room = Room()"#);
    run(r#"guest1 = Guest()"#);
    run(r#"guest1.room.number = 30"#);
    assert_eq!(run(r#"guest1.room.level"#), run(r#"(3)"#));
    run(r#"guest2 = Guest()"#);
    assert_eq!(run(r#"guest2.room.number"#), run(r#"(30)"#));
    assert_eq!(run(r#"guest2.room.level"#), run(r#"(3)"#));
}

/// Nucleoid creates a class assignment with multiple properties after declaration
#[test]
fn creates_a_class_assignment_with_multiple_properties_after_declaration() {
    let mut run = runner();
    run(r#"class Channel:
    pass"#);
    run(r#"class Frequency:
    pass"#);
    run(r#"channel1 = Channel()"#);
    run(r#"$Channel.frequency = Frequency()"#);
    run(r#"$Frequency.hertz = 1 / $Frequency.period"#);
    assert_eq!(run(r#"channel1.frequency.hertz"#), run(r#"(null)"#));
    run(r#"channel1.frequency.period = 0.0025"#);
    assert_eq!(run(r#"channel1.frequency.hertz"#), run(r#"(400)"#));
    run(r#"channel2 = Channel()"#);
    assert_eq!(run(r#"channel2.frequency.period"#), run(r#"(0.0025)"#));
    assert_eq!(run(r#"channel2.frequency.hertz"#), run(r#"(400)"#));
}

/// Nucleoid creates a class assignment as multiple properties as part of a declaration before initialization
#[test]
fn creates_a_class_assignment_as_multiple_properties_as_part_of_a_declaration_before_initialization() {
    let mut run = runner();
    run(r#"class Hospital:
    pass"#);
    run(r#"class Clinic:
    pass"#);
    run(r#"$Hospital.clinic = Clinic()"#);
    run(r#"$Hospital.patients = $Hospital.clinic.beds * 746"#);
    run(r#"hospital1 = Hospital()"#);
    assert_eq!(run(r#"hospital1.patients"#), run(r#"(null)"#));
    run(r#"hospital1.clinic.beds = 2678"#);
    assert_eq!(run(r#"hospital1.patients"#), run(r#"(1997788)"#));
    run(r#"hospital1.clinic.beds = 3000"#);
    assert_eq!(run(r#"hospital1.patients"#), run(r#"(2238000)"#));
}

/// Nucleoid creates a class assignment as multiple properties as part of a declaration after initialization
#[test]
fn creates_a_class_assignment_as_multiple_properties_as_part_of_a_declaration_after_initialization() {
    let mut run = runner();
    run(r#"class Server:
    pass"#);
    run(r#"class OS:
    pass"#);
    run(r#"$Server.os = OS()"#);
    run(r#"server1 = Server()"#);
    run(r#"server1.os.version = 14"#);
    run(r#"$Server.build = $Server.os.version + ".526291""#);
    assert_eq!(run(r#"server1.build"#), run(r#"("14.526291")"#));
    run(r#"server1.os.version = 15"#);
    assert_eq!(run(r#"server1.build"#), run(r#"("15.526291")"#));
}

/// Nucleoid creates a class assignment only if the instance is defined
#[test]
fn creates_a_class_assignment_only_if_the_instance_is_defined() {
    let mut run = runner();
    run(r#"class Phone:
    pass"#);
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    $Phone.line.wired = true
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("Phone.line is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid creates a for of statement
#[test]
fn creates_a_for_of_statement() {
    let mut run = runner();
    run(r#"class Question(rate: int):
    this.rate = rate"#);
    run(r#"question1 = Question(4)"#);
    run(r#"question2 = Question(5)"#);
    run(r#"class Summary(question):
    this.question = question"#);
    run(r#"$Summary.rate = $Summary.question.rate.value"#);
    run(r#"for question of Question:
    Summary(question)"#);
    assert_eq!(run(r#"Summary[0].rate"#), run(r#"(4)"#));
    assert_eq!(run(r#"Summary[1].rate"#), run(r#"(5)"#));
}

/// Nucleoid creates a block of for statement without dependencies
#[test]
fn creates_a_block_of_for_statement_without_dependencies() {
    let mut run = runner();
    run(r#"class Item:
    pass"#);
    run(r#"item1 = Item()"#);
    run(r#"item2 = Item()"#);
    run(r#"VALUE = 10"#);
    run(r#"for item of Item:
    i = 10 * VALUE
    item.score = i"#);
    run(r#"VALUE = 20"#);
    assert_eq!(run(r#"item1.score"#), run(r#"(100)"#));
    assert_eq!(run(r#"item2.score"#), run(r#"(100)"#));
    run(r#"for item of Item:
    i = 10 * VALUE
    item.score = i"#);
    assert_eq!(run(r#"item1.score"#), run(r#"(200)"#));
    assert_eq!(run(r#"item2.score"#), run(r#"(200)"#));
    run(r#"item3 = Item()"#);
    assert_eq!(run(r#"item3.score"#), run(r#"(null)"#));
}

/// Nucleoid loops through only defined objects in a for of statement
#[test]
fn loops_through_only_defined_objects_in_a_for_of_statement() {
    let mut run = runner();
    run(r#"array = []"#);
    run(r#"class Item:
    pass"#);
    run(r#"item1 = Object()"#);
    run(r#"array.push(item1)"#);
    run(r#"item2 = { "id": "item3" }"#);
    run(r#"array.push(item2)"#);
    run(r#"item4 = Item()"#);
    run(r#"array.push(item4)"#);
    run(r#"item5 = { "id": "item4" }"#);
    run(r#"array.push(item5)"#);
    run(r#"count = 0"#);
    run(r#"items = []"#);
    run(r#"for item of array:
    count = count + 1
    items.push(item)"#);
    assert_eq!(run(r#"count"#), run(r#"(1)"#));
    assert_eq!(run(r#"items.length"#), run(r#"(1)"#));
    assert_eq!(run(r#"items[0]"#), run(r#"(item4)"#));
}

/// Nucleoid supports an if statement in a for of statement
#[test]
fn supports_an_if_statement_in_a_for_of_statement() {
    let mut run = runner();
    run(r#"class Question:
    pass"#);
    run(r#"question1 = Question()"#);
    run(r#"question2 = Question()"#);
    run(r#"question2.archived = true"#);
    run(r#"question3 = Question()"#);
    run(r#"class Summary(question):
    this.question = question"#);
    run(r#"$Summary.type = "DAILY""#);
    run(r#"for question of Question:
    if not question.archived:
        Summary(question)"#);
    assert_eq!(run(r#"Summary.length"#), run(r#"(2)"#));
    assert_eq!(run(r#"Summary[0].question.id"#), run(r#"("question1")"#));
    assert_eq!(run(r#"Summary[1].question.id"#), run(r#"("question3")"#));
    assert_eq!(run(r#"Summary[0].type"#), run(r#"("DAILY")"#));
    assert_eq!(run(r#"Summary[1].type"#), run(r#"("DAILY")"#));
    run(r#"$Summary.type = "WEEKLY""#);
    assert_eq!(run(r#"Summary[0].type"#), run(r#"("WEEKLY")"#));
    assert_eq!(run(r#"Summary[1].type"#), run(r#"("WEEKLY")"#));
}

/// Nucleoid returns an integer in variable assignment
#[test]
fn returns_an_integer_in_variable_assignment() {
    let mut run = runner();
    run(r#"def test(a):
    return a = 2"#);
    run(r#"b = 1"#);
    run(r#"test(b)

# return: 2"#);
}

/// Nucleoid returns the reference of a function call
#[test]
fn returns_the_reference_of_a_function_call() {
    let mut run = runner();
    run(r#"a = Object()"#);
    run(r#"c = 1"#);
    run(r#"def test(b):
    return b = a"#);
    assert_eq!(run(r#"test(c)"#), run(r#"({})"#));
    assert_eq!(run(r#"c"#), run(r#"(1)"#));
}

/// Nucleoid returns a string value of a function call
#[test]
fn returns_a_string_value_of_a_function_call() {
    let mut run = runner();
    run(r#"def test(a):
    return a = "abc""#);
    run(r#"b = 1"#);
    run(r#"test(b)

# return: "abc""#);
}

/// Nucleoid returns an object value of a function call
#[test]
fn returns_an_object_value_of_a_function_call() {
    let mut run = runner();
    run(r#"def test(a):
    return a = Object()"#);
    run(r#"b = 1"#);
    run(r#"test(b)

# return: {}"#);
}

/// Nucleoid runs a function with a variable
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
#[test]
fn returns_the_instance_itself_in_instance_creation() {
    let mut run = runner();
    run(r#"class Test(prop: int):
    this.prop = prop"#);
    run(r#"Test(123)

# return: { "id": "[UUID]", "prop": 123 }"#);
    assert_eq!(run(r#"Test[0].prop"#), run(r#"(123)"#));
    assert_eq!(run(r#"Test[0].id != null"#), run(r#"(true)"#));
}

/// Nucleoid explains how a value was derived
#[test]
fn explains_how_a_value_was_derived() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"b = a + 2"#);
    run(r#"c = b * 2"#);
    assert_eq!(run(r#"(why c).length"#), run(r#"(3)"#));
    assert_eq!(run(r#"(why c)[0].node"#), run(r#"("c")"#));
    assert_eq!(run(r#"(why c)[0].holds"#), run(r#"(6)"#));
    assert_eq!(run(r#"(why c)[0].rule"#), run(r#"("c = b*2")"#));
    assert_eq!(run(r#"(why c)[0].state"#), run(r#"("derived")"#));
    assert_eq!(run(r#"(why c)[0].from"#), run(r#"(["b"])"#));
}

/// Nucleoid states a fact that follows from nothing else
#[test]
fn states_a_fact_that_follows_from_nothing_else() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"b = a + 2"#);
    assert_eq!(run(r#"(why b)[1].node"#), run(r#"("a")"#));
    assert_eq!(run(r#"(why b)[1].state"#), run(r#"("stated")"#));
    assert_eq!(run(r#"(why b)[1].from"#), run(r#"([])"#));
}

/// Nucleoid names the class-level rule a property was derived from
#[test]
fn names_the_class_level_rule_a_property_was_derived_from() {
    let mut run = runner();
    run(r#"class Human(name: str):
    this.name = name"#);
    run(r#"$Human.mortal = true"#);
    run(r#"socrates = Human("Socrates")"#);
    assert_eq!(run(r#"(why socrates.mortal).length"#), run(r#"(1)"#));
    assert_eq!(run(r#"(why socrates.mortal)[0].rule"#), run(r#"("$Human.mortal = true")"#));
    assert_eq!(run(r#"(why socrates.mortal)[0].state"#), run(r#"("derived")"#));
}

/// Nucleoid reports what a value affects
#[test]
fn reports_what_a_value_affects() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"b = a + 2"#);
    run(r#"c = b * 2"#);
    assert_eq!(run(r#"(affects a).length"#), run(r#"(2)"#));
    assert_eq!(run(r#"(affects a)[0].node"#), run(r#"("b")"#));
    assert_eq!(run(r#"(affects a)[1].node"#), run(r#"("c")"#));
}

/// Nucleoid chains reasoning operations
#[test]
fn chains_reasoning_operations() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"b = a + 2"#);
    run(r#"c = b * 2"#);
    assert_eq!(run(r#"(c |> why).length"#), run(r#"(3)"#));
    assert_eq!(run(r#"(a |> affects |> why).length"#), run(r#"(3)"#));
    assert_eq!(run(r#"(why c).length"#), run(r#"((c |> why).length)"#));
}

/// Nucleoid keeps an explanation up to date
#[test]
fn keeps_an_explanation_up_to_date() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"b = a + 2"#);
    run(r#"c = b * 2"#);
    run(r#"trace = why c"#);
    assert_eq!(run(r#"trace[0].holds"#), run(r#"(6)"#));
    run(r#"a = 5"#);
    assert_eq!(run(r#"trace[0].holds"#), run(r#"(14)"#));
}

/// Nucleoid does not select a reasoning statement
#[test]
fn does_not_select_a_reasoning_statement() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"b = a + 2"#);
    run(r#"trace = why b"#);
    assert_eq!(run(r#"(affects a).length"#), run(r#"(1)"#));
    assert_eq!(run(r#"(affects a)[0].node"#), run(r#"("b")"#));
}

/// Nucleoid selects the whole model
#[test]
fn selects_the_whole_model() {
    let mut run = runner();
    run(r#"a = 1"#);
    run(r#"b = a + 2"#);
    assert_eq!(run(r#"(model |> why).length"#), run(r#"(2)"#));
}

/// Nucleoid throws an error when explaining something that is not defined
#[test]
fn throws_an_error_when_explaining_something_that_is_not_defined() {
    let mut run = runner();
    run("__nucleoid_test_assertion_0_actual = null");
    run("__nucleoid_test_assertion_0_expected = null");
    run("__nucleoid_test_assertion_0_ran = false");
    run(r#"try:
    why nothing
catch error:
    __nucleoid_test_assertion_0_actual = error
    __nucleoid_test_assertion_0_expected = ReferenceError("nothing is not defined")
    __nucleoid_test_assertion_0_ran = true"#);
    assert_eq!(run("__nucleoid_test_assertion_0_ran"), run("(true)"));
    assert_eq!(run("__nucleoid_test_assertion_0_actual"), run("(__nucleoid_test_assertion_0_expected)"));
}

/// Nucleoid treats a reasoning name as a variable when one is defined
#[test]
fn treats_a_reasoning_name_as_a_variable_when_one_is_defined() {
    let mut run = runner();
    run(r#"why = 1"#);
    assert_eq!(run(r#"why"#), run(r#"(1)"#));
}

