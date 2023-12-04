const test = require("../lib/test");
const { equal, notEqual, deepEqual, throws } = require("assert");
const nucleoid = require("../../");

const validate = (error, expectedError, expectedMessage) => {
  return error instanceof expectedError && error.message === expectedMessage;
};

describe("Nucleoid", () => {
  before(() => nucleoid.start({ declarative: true, test: true }));
  beforeEach(() => test.clear());

  describe("in declarative mode", () => {
    const details = { details: true };

    it("runs statements in the state", () => {
      nucleoid.run("var i = 1 ;");
      equal(nucleoid.run("i == 1"), true);
    });

    it("runs expression statement", () => {
      nucleoid.run("var j = 1 ;");
      equal(nucleoid.run("j + 2"), 3);
    });

    it("rejects variable declaration without definition", () => {
      throws(
        () => nucleoid.run("var a"),
        (err) => validate(err, SyntaxError, "Missing definition")
      );
    });

    it("allows variable declaration without var keyword", () => {
      nucleoid.run("pi = 3.14 ;");
      equal(nucleoid.run("pi == 3.14"), true);
    });

    it("allows statements without semicolon at the end", () => {
      nucleoid.run("au = 149597870700");
      equal(nucleoid.run("au == 149597870700"), true);
    });

    it("creates dependency based on length of identifier", () => {
      nucleoid.run("str1 = 'ABC'");
      nucleoid.run("i1 = str1.length + 1");
      equal(nucleoid.run("i1"), 4);

      nucleoid.run("str1 = 'ABCD'");
      equal(nucleoid.run("i1"), 5);

      nucleoid.run("if ( str1.length > 5 ) { i2 = i1 }");
      nucleoid.run("str1 = 'ABCDEF'");
      equal(nucleoid.run("i2"), 7);
    });

    it("creates class with constructor", () => {
      nucleoid.run(
        "class Device { constructor ( name ) { this.name = name } }"
      );
      nucleoid.run("$Device.active = false");
      nucleoid.run("if ( $Device.name ) { $Device.active = true }");

      nucleoid.run("device1 = new Device ( 'Entrance' )");
      equal(nucleoid.run("device1.name"), "Entrance");
      equal(nucleoid.run("device1.active"), true);

      let device2 = nucleoid.run("new Device ( 'Hall' )");
      nucleoid.run(`${device2.id}.name`, "Hall");
      nucleoid.run(`${device2.id}.active`, true);

      nucleoid.run("device3 = new Device ( )");
      equal(nucleoid.run("device3.name"), undefined);
      equal(nucleoid.run("device3.active"), false);

      let device4 = nucleoid.run("new Device ( )");
      equal(nucleoid.run(`${device4.id}.name`), undefined);
      equal(nucleoid.run(`${device4.id}.active`), false);
    });

    it("adds object in object list", () => {
      nucleoid.run("class Student { }");
      nucleoid.run("user0 = new Student ( )");
      deepEqual(
        nucleoid.run("Student.find ( student => student.id === 'user0')"),
        { id: "user0" }
      );
      deepEqual(nucleoid.run("Student['user0']"), { id: "user0" });
    });

    it("prevents class and object lists when class is updated", () => {
      nucleoid.run("class User { }");
      nucleoid.run("new User ( )");
      equal(nucleoid.run("classes.length"), 1);
      equal(nucleoid.run("User.length"), 1);

      nucleoid.run("class User { }");
      nucleoid.run("new User ( )");
      equal(nucleoid.run("classes.length"), 1);
      equal(nucleoid.run("User.length"), 2);
    });

    it("adds created class in class list", () => {
      equal(nucleoid.run("classes.length"), 0);

      nucleoid.run("class Student { }");
      equal(nucleoid.run("classes.length"), 1);

      nucleoid.run("class User { }");
      equal(nucleoid.run("classes.length"), 2);
    });

    it("updates class definition", () => {
      nucleoid.run("class Message { }");
      nucleoid.run("$Message.read = false");
      nucleoid.run("message1 = new Message ( )");
      nucleoid.run(
        "class Message { constructor ( payload ) { this.payload = payload } }"
      );
      equal(nucleoid.run("message1.read"), false);
      equal(nucleoid.run("message1.payload"), undefined);

      nucleoid.run("message2 = new Message('MESSAGE')");
      equal(nucleoid.run("message2.read"), false);
      equal(nucleoid.run("message2.payload"), "MESSAGE");
    });

    it("supports new line as replacing with space", () => {
      nucleoid.run("a = 1 ; \n b = 2");
      equal(nucleoid.run("a"), 1);
      equal(nucleoid.run("b"), 2);

      nucleoid.run("a = 3 ; \r b = 4");
      equal(nucleoid.run("a"), 3);
      equal(nucleoid.run("b"), 4);

      nucleoid.run("a = 5 ; \r\n b = 6");
      equal(nucleoid.run("a"), 5);
      equal(nucleoid.run("b"), 6);
    });

    it("supports string in expression", () => {
      equal(nucleoid.run("'New String'"), "New String");
      equal(nucleoid.run('"New String"'), "New String");
      equal(nucleoid.run("`New String`"), "New String");

      nucleoid.run("a = 123");
      equal(nucleoid.run("`New ${a} String`"), "New 123 String");
    });

    it("supports logical operators", () => {
      nucleoid.run("condition = false");
      equal(nucleoid.run("condition || true"), true);
      equal(nucleoid.run("!condition && true"), true);
    });

    it("supports standard built-in functions", () => {
      nucleoid.run("max = Number.MAX_SAFE_INTEGER");
      nucleoid.run("now = Date.now ( )");
    });

    it("supports creating standard built-in objects", () => {
      nucleoid.run("date = new Date ( '2019-7-24' )");
      equal(nucleoid.run("date.getYear()"), 119);
    });

    it("supports built-in objects", () => {
      const result = nucleoid.run("date1 = new Date ( )");
      nucleoid.run(`date2 = new Date ( ${result.getTime()} )`);
      equal(nucleoid.run("date1.getTime() == date2.getTime()"), true);

      nucleoid.run("date3 = Date.parse ( '04 Dec 1995 00:12:00 GMT' )");
      equal(nucleoid.run("date3"), 818035920000);

      throws(
        () => {
          nucleoid.run("date4 = Date.wrong ( )");
        },
        (error) => validate(error, TypeError, "Date.wrong is not a function")
      );
    });

    it("calls function with no return", () => {
      nucleoid.run("a = 1");
      nucleoid.run("function copy ( val ) { b = val }");
      const result = nucleoid.run("copy ( a )");
      equal(result, undefined);
    });

    it("calls function with returning variable", () => {
      nucleoid.run("a = 1");
      nucleoid.run("function copy ( val ) { b = val; return b; }");
      const result = nucleoid.run("copy ( a )");
      equal(result, 1);
    });

    it("calls function with returning value", () => {
      nucleoid.run("a = 1");
      nucleoid.run("function copy ( val ) { b = val; return val; }");
      const result = nucleoid.run("copy ( a )");
      equal(result, 1);
    });

    it("supports function in expression", () => {
      nucleoid.run("list = [1, 2, 3]");
      equal(
        nucleoid.run(
          "list.find ( function ( element ) { return element == 3 } )"
        ),
        3
      );

      equal(
        nucleoid.run("list.find ( element => { return element == 2 } )"),
        2
      );

      equal(nucleoid.run("list.find ( element => element == 1 )"), 1);
      equal(nucleoid.run("list.find ( element => ( element == 1 ) )"), 1);
    });

    it("supports function with parameter in expression", () => {
      nucleoid.run("samples = [ 38.2 , 39.1 , 38.8 , 39 ]");
      nucleoid.run("ratio = 2.1");
      nucleoid.run("element = 38.5");
      equal(
        nucleoid.run(
          "samples.find ( function ( element ) { let result = element * ratio ; return result == 81.48 } )"
        ),
        38.8
      );

      equal(
        nucleoid.run(
          "samples.find ( element => { let result = element * ratio ; return result == 81.48 } )"
        ),
        38.8
      );

      equal(nucleoid.run("samples.find ( element => element == 38.8 )"), 38.8);
      equal(
        nucleoid.run("samples.find ( element => ( element == 38.8 ) )"),
        38.8
      );
    });

    it("creates let statement with JSON", () => {
      let payload = nucleoid.run(
        '{ let payload = { "data" : "TEST" , "nested" : { "data" : "NESTED_TEST" } } ; return payload }'
      );
      equal(payload.data, "TEST");
      equal(payload.nested.data, "NESTED_TEST");

      nucleoid.run('message = { "pid" : 1200 }');
      equal(nucleoid.run("message.pid"), 1200);

      const i = nucleoid.run(
        '{ let scope = { query : "test" } ; let i = { test : scope.query } ; return i ; }'
      );
      equal(i.test, "test");
    });

    it("returns inline JSON object", () => {
      const json = nucleoid.run(
        '{ return { "number" : 123 , "string" : "ABC" , "bool" : true } }'
      );
      deepEqual(json, { number: 123, string: "ABC", bool: true });
    });

    it("returns inline JSON array", () => {
      const json = nucleoid.run(
        '{ return [ { "number" : 123 , "string" : "ABC" , "bool" : true } ] }'
      );
      deepEqual(json, [{ number: 123, string: "ABC", bool: true }]);
    });

    it("returns inline object", () => {
      const object = nucleoid.run(
        '{ return { number : 123 , string : "ABC" , bool : true } }'
      );
      deepEqual(object, { number: 123, string: "ABC", bool: true });
    });

    it("returns inline array", () => {
      const object = nucleoid.run(
        '{ return [ { number : 123 , string : "ABC" , bool : true } ] }'
      );
      deepEqual(object, [{ number: 123, string: "ABC", bool: true }]);
    });

    it("assigns block in function as dependency", () => {
      nucleoid.run("class Student { }");
      nucleoid.run("student1 = new Student ( )");
      nucleoid.run("student1.age = 7");
      nucleoid.run("student2 = new Student ( )");
      nucleoid.run("student2.age = 8");
      nucleoid.run("student3 = new Student ( )");
      nucleoid.run("student3.age = 9");

      nucleoid.run("age = 8");
      nucleoid.run("student = Student.find ( s => s.age == age )");
      deepEqual(nucleoid.run("student"), nucleoid.run("student2"));

      nucleoid.run("age = 9");
      deepEqual(nucleoid.run("student"), nucleoid.run("student3"));
    });

    it("supports chained functions with parameter in expression", () => {
      nucleoid.run(
        "class Result { constructor ( score ) { this.score = score } }"
      );
      nucleoid.run("new Result ( 10 ) ; new Result ( 15 ) ; new Result ( 20 )");

      nucleoid.run("upperThreshold = 18");
      nucleoid.run("lowerThreshold = 12");
      nucleoid.run(
        "list = Result.filter ( r => r.score > lowerThreshold ) .filter ( r => r.score < upperThreshold )"
      );
      let list = nucleoid.run("list");
      equal(list[0].score, 15);

      nucleoid.run("lowerThreshold = 7");
      list = nucleoid.run("list");
      equal(list[0].score, 10);
      equal(list[1].score, 15);

      nucleoid.run("upperThreshold = 14");
      list = nucleoid.run("list");
      equal(list[0].score, 10);
    });

    it("supports nested functions as parameter in expression", () => {
      nucleoid.run("name = 'AbCDE'");
      nucleoid.run("pointer = 0");
      nucleoid.run(
        "if ( ! /[A-Z]/.test ( name.charAt ( pointer ) ) ) { throw 'INVALID_FIRST_CHARACTER' }"
      );

      throws(
        () => {
          nucleoid.run("name = 'bbCDE'");
        },
        (error) => error === "INVALID_FIRST_CHARACTER"
      );

      nucleoid.run("name = 'CbCDE'");

      throws(
        () => {
          nucleoid.run("pointer = 1");
        },
        (error) => error === "INVALID_FIRST_CHARACTER"
      );
    });

    it("supports property of chained functions in expression", () => {
      nucleoid.run("class User { }");
      nucleoid.run("class Registration { }");
      nucleoid.run("user1 = new User ( )");

      nucleoid.run("registration1 = new Registration ( )");
      nucleoid.run("registration1.user = user1");

      nucleoid.run("registration2 = new Registration ( )");
      nucleoid.run("registration2.user = user1");

      throws(() => {
        nucleoid.run(
          "if ( Registrations.filter ( r => r.user == User ) .length > 1 ) { throw 'USER_ALREADY_REGISTERED' }"
        );
      }, "USER_ALREADY_REGISTERED");
    });

    it("supports array with brackets", () => {
      nucleoid.run("states = [ 'NY' , 'GA' , 'CT' , 'MI' ]");
      equal(nucleoid.run("states [ 2 ]"), "CT");
    });

    it("retrieves value by variable", () => {
      nucleoid.run("number = -1");
      equal(nucleoid.run("number"), -1);
    });

    it("rejects variable if not declared", () => {
      throws(
        () => {
          nucleoid.run("e == 2.71828");
        },
        (error) => validate(error, ReferenceError, "e is not defined")
      );
    });

    it("runs multiple statements in the state", () => {
      nucleoid.run("k = 1 ; l = k + 1 ; k = 2");
      equal(nucleoid.run("l == 3"), true);
    });

    it("runs dependent statements in the same transaction", () => {
      nucleoid.run(
        "class Vehicle { } ; $Vehicle.tag = 'US-' + $Vehicle.plate "
      );
      nucleoid.run("vehicle1 = new Vehicle ( )");
      nucleoid.run("vehicle1.plate = 'XSJ422'");
      equal(nucleoid.run("vehicle1.tag"), "US-XSJ422");
    });

    it("runs dependencies in order as received", () => {
      nucleoid.run("any = 0");
      nucleoid.run("if ( any > 1 ) { result = 1 }");
      nucleoid.run("if ( any > 2 ) { result = 2 }");
      nucleoid.run("if ( any > 3 ) { result = 3 }");
      nucleoid.run("if ( any > 2 ) { result = 4 }");
      nucleoid.run("if ( any > 1 ) { result = 5 }");

      nucleoid.run("any = 4");
      equal(nucleoid.run("result"), 5);
    });

    it("runs let at root scope", () => {
      nucleoid.run("number = 13");
      equal(nucleoid.run("let i = number + 4; i;"), 17);
      nucleoid.run("number = 14");
      throws(
        () => {
          nucleoid.run("i");
        },
        (error) => validate(error, ReferenceError, "i is not defined")
      );
    });

    it("searches variable in scope before state", () => {
      nucleoid.run("e = 2.71828");
      nucleoid.run("{ let e = 3 ; number = e }");
      equal(nucleoid.run("number"), 3);
    });

    it("uses local variable at lowest scope as priority", () => {
      nucleoid.run("pi = 3.14");
      nucleoid.run("number = pi");

      nucleoid.run("{ let pi = 3.141 ; { number = pi } }");
      equal(nucleoid.run("number"), 3.141);

      equal(
        nucleoid.run(
          "{ let pi = 3.1415 ; { let number = pi ; { let pi = 3.14159 ; number = pi ; return number } } }"
        ),
        3.14159
      );

      nucleoid.run(
        "{ let pi = 3.14159 ; number = pi ; { let pi = 3.141592 ; let number = pi ; { let pi = 3.1415926 ; number = pi } } }"
      );
      equal(nucleoid.run("number"), 3.14159);
    });

    it("creates let statement if its instance is defined", () => {
      nucleoid.run("class Ticket { }");
      throws(
        () => {
          nucleoid.run(
            "{ let ticket = new Ticket ( ) ; ticket.event.group = 'ENTERTAINMENT' }"
          );
        },
        (error) =>
          validate(error, ReferenceError, "ticket.event is not defined")
      );
    });

    it("declares let statement with undefined", () => {
      nucleoid.run(
        "class Device { constructor ( code ) { this.code = code } }"
      );
      nucleoid.run("device1 = new Device ( 'A0' )");
      nucleoid.run("device2 = new Device ( 'B1' )");

      equal(
        nucleoid.run("device1"),
        nucleoid.run(
          "let device = Device.find ( d => d.code == 'A0' ) ; if ( ! device ) { throw 'INVALID_DEVICE' }  device"
        )
      );

      throws(
        () => {
          nucleoid.run(
            "let device = Device.find ( d => d.code == 'A1' ) ; if ( ! device ) { throw 'INVALID_DEVICE' } device"
          );
        },
        (error) => error === "INVALID_DEVICE"
      );
    });

    it("creates standard built-in object of let statement as property", () => {
      nucleoid.run("class Shipment { }");
      nucleoid.run(
        "{ let shipment = new Shipment ( ) ; shipment.date = new Date ( '2019-1-3' ) ; shipment1 = shipment }"
      );
      equal(nucleoid.run("shipment1.date.toDateString ( )"), "Thu Jan 03 2019");
    });

    it("creates property of let statement in different scope", () => {
      nucleoid.run("class User { }");
      nucleoid.run("user0 = new User ( )");
      nucleoid.run(
        "let user = User['user0'] ; if ( user ) { user.name = 'TEST' }"
      );
      equal(nucleoid.run("user0.name"), "TEST");
    });

    it("assigns undefined if any dependencies in expression is undefined", () => {
      nucleoid.run("class Person { }");
      nucleoid.run("person1 = new Person ( )");
      nucleoid.run("person1.lastName = 'Brown'");
      nucleoid.run(
        "person1.fullName = person1.firstName + ' ' + person1.lastName"
      );
      equal(nucleoid.run("person1.fullName") === undefined, true);
    });

    it("keeps as null if any dependencies as in local is null", () => {
      nucleoid.run("a = 1");
      nucleoid.run("{ let b = null ; c = b / a }");
      equal(nucleoid.run("c"), 0);
    });

    it("keeps as null if any dependencies in expression is null", () => {
      nucleoid.run("class Schedule { }");
      nucleoid.run("schedule1 = new Schedule ( )");
      nucleoid.run("schedule1.expression = '0 */2 * * *'");
      nucleoid.run("schedule1.script = null");
      nucleoid.run(
        "schedule1.run = schedule1.expression + ' ' + schedule1.script"
      );
      equal(nucleoid.run("schedule1.run"), "0 */2 * * * null");
    });

    it("assigns null if there is null pointer in expression", () => {
      nucleoid.run("class Product { }");
      nucleoid.run("product1 = new Product ( )");
      nucleoid.run("score = product1.quality.score");
      equal(nucleoid.run("score"), null);
    });

    it("places instance in the list of class when created", () => {
      nucleoid.run("class Student { }");
      equal(nucleoid.run("Array.isArray ( Student )"), true);

      nucleoid.run("student1 = new Student ( )");
      equal(nucleoid.run("Student.length"), 1);
    });

    it("assigns unique variable for instance without variable name defined", () => {
      nucleoid.run("class Vehicle { }");
      nucleoid.run("new Vehicle ( )");
      equal(nucleoid.run("Vehicle.length"), 1);
    });

    it("throws error as a string", () => {
      throws(
        () => {
          nucleoid.run("throw 'INVALID'");
        },
        (error) => error === "INVALID"
      );

      throws(
        () => {
          nucleoid.run('throw "INVALID"');
        },
        (error) => error === "INVALID"
      );
    });

    it("throws error as an integer", () => {
      throws(
        () => {
          nucleoid.run("throw 123");
        },
        (error) => error === 123
      );
    });

    it("throws reference error if invalid in throw", () => {
      throws(
        () => {
          nucleoid.run("throw abc");
        },
        (error) => validate(error, ReferenceError, "abc is not defined")
      );
    });

    it("throws error inside block", () => {
      nucleoid.run("k = 99");
      throws(
        () => {
          nucleoid.run("if ( k >= 99 ) { throw 'INVALID' }");
        },
        (error) => error === "INVALID"
      );
    });

    it("throws error as a variable", () => {
      nucleoid.run("length = 0.1");
      throws(
        () => {
          nucleoid.run("if ( length < 1 ) { throw length }");
        },
        (error) => error === 0.1
      );

      throws(
        () => {
          nucleoid.run("if ( length < 1.1 ) { throw 'length' }");
        },
        (error) => error === "length"
      );
    });

    it("assigns function as dependency", () => {
      nucleoid.run("list = [ ]");
      nucleoid.run("count = list.filter ( n => n % 2 )");
      nucleoid.run("list.push ( 1 )");
      equal(nucleoid.run("count.length"), 1);

      nucleoid.run("list.push ( 2 )");
      equal(nucleoid.run("count.length"), 1);

      nucleoid.run("list.push ( 3 )");
      equal(nucleoid.run("count.length"), 2);

      nucleoid.run("list.pop ( )");
      equal(nucleoid.run("count.length"), 1);
    });

    it("assigns parameter in function as dependency", () => {
      nucleoid.run("str1 = 'ABC'");
      nucleoid.run("str2 = str1.toLowerCase ( ) + 'd'");
      nucleoid.run("str3 = str2.concat ( str1 )");
      equal(nucleoid.run("str2"), "abcd");
      equal(nucleoid.run("str3"), "abcdABC");

      nucleoid.run("str1 = 'AAA'");
      equal(nucleoid.run("str2"), "aaad");
      equal(nucleoid.run("str3"), "aaadAAA");
    });

    it("supports regular expression literal", () => {
      nucleoid.run("class User { }");
      nucleoid.run(
        "if ( ! /.{4,8}/.test ( $User.password ) ) { throw 'INVALID_PASSWORD' }"
      );
      nucleoid.run("user1 = new User ( )");
      throws(
        () => {
          nucleoid.run("user1.password = 'PAS'");
        },
        (error) => error === "INVALID_PASSWORD"
      );
    });

    it("rejects defining class declaration in non-class declaration block", () => {
      nucleoid.run("class Person { }");
      nucleoid.run("person1 = new Person ( )");
      nucleoid.run("person1.weight = 90");
      nucleoid.run("person1.height = 1.8");
      throws(
        () => {
          nucleoid.run(
            "{ let weight = person1.weight ; let height = person1.height ; $Person.bmi = weight / ( height * height ) }"
          );
        },
        (error) =>
          validate(
            error,
            SyntaxError,
            "Cannot define class declaration in non-class block"
          )
      );
    });

    it("detects circular dependency", () => {
      nucleoid.run("number1 = 10");
      nucleoid.run("number2 = number1 * 10");
      throws(
        () => {
          nucleoid.run("number1 = number2 * 10");
        },
        (error) => validate(error, ReferenceError, "Circular Dependency")
      );
    });

    it("creates function in state", () => {
      nucleoid.run("function generate ( number ) { return number * 10 }");
      nucleoid.run("random = 10");
      nucleoid.run("number = generate ( random )");
      equal(nucleoid.run("number"), 100);

      nucleoid.run("random = 20");
      equal(nucleoid.run("number"), 200);
    });

    it("publishes event", () => {
      nucleoid.run(
        "class Task { constructor ( ) { event ( 'TASK_CREATED', 'TASK_DATA' ) } }"
      );

      let result = nucleoid.run("task1 = new Task ( )", details);
      equal(result.events[0].name, "TASK_CREATED");
      equal(result.events[0].data, '"TASK_DATA"');
    });

    it("rollbacks variable if exception is thrown", () => {
      nucleoid.run("a = 5");
      nucleoid.run("if ( a > 5 ) { throw 'INVALID_VALUE' }");

      throws(
        () => {
          nucleoid.run("a = 6");
        },
        (error) => error === "INVALID_VALUE"
      );
      equal(nucleoid.run("a"), 5);
    });

    it("rollbacks property if exception is thrown", () => {
      nucleoid.run("class Item { }");
      nucleoid.run("if ( $Item.sku == 'A' ) { throw 'INVALID_SKU' }");
      nucleoid.run("item1 = new Item ( )");

      throws(
        () => {
          nucleoid.run("item1.sku = 'A'");
        },
        (error) => error === "INVALID_SKU"
      );
      equal(nucleoid.run("item1.sku"), undefined);
    });

    it("rollbacks instance if exception is thrown", () => {
      nucleoid.run(
        "class User { constructor ( first , last ) { this.first = first ; this.last = last } }"
      );
      nucleoid.run("if ( $User.first.length < 3 ) { throw 'INVALID_USER' }");

      throws(
        () => {
          nucleoid.run("user1 = new User ( 'F' , 'L' )");
        },
        (error) => error === "INVALID_USER"
      );
      throws(
        () => {
          nucleoid.run("user1");
        },
        (error) => validate(error, ReferenceError, "user1 is not defined")
      );
    });

    it("creates variable assignment", () => {
      nucleoid.run("x = 1");
      nucleoid.run("y = x + 2");
      nucleoid.run("x = 2");
      equal(nucleoid.run("y == 4"), true);
    });

    it("return assigned value while variable assignment", () => {
      const result = nucleoid.run("x = 1");
      equal(result, 1);
    });

    it("updates variable assignment", () => {
      nucleoid.run("a = 1");
      nucleoid.run("b = 2");
      nucleoid.run("c = a + 3");
      nucleoid.run("c = b + 3");
      equal(nucleoid.run("c"), 5);

      nucleoid.run("b = 4");
      equal(nucleoid.run("c"), 7);
    });

    it("uses its value when self variable used", () => {
      nucleoid.run("radius = 10");
      nucleoid.run("radius = radius + 10");
      equal(nucleoid.run("radius"), 20);
    });

    it("deletes variable assignment", () => {
      nucleoid.run("t = 1");
      nucleoid.run("q = t + 1");
      nucleoid.run("delete q");
      nucleoid.run("t = 2");
      throws(
        () => {
          nucleoid.run("q");
        },
        (error) => validate(error, ReferenceError, "q is not defined")
      );
    });

    it("uses value property to indicate using only value of variable", () => {
      nucleoid.run("goldenRatio = 1.618");
      nucleoid.run("altitude = 10");

      nucleoid.run("width = goldenRatio.value * altitude");
      equal(nucleoid.run("width"), 16.18);

      nucleoid.run("goldenRatio = 1.62");
      equal(nucleoid.run("width"), 16.18);

      nucleoid.run("altitude = 100");
      equal(nucleoid.run("width"), 161.8);
    });

    it("creates if statement of variable", () => {
      nucleoid.run("m = false");
      nucleoid.run("n = false");
      nucleoid.run("if ( m == true ) { n = m && true }");
      equal(nucleoid.run("n == false"), true);

      nucleoid.run("m = true");
      equal(nucleoid.run("n == true"), true);
    });

    it("updates if block of variable", () => {
      nucleoid.run("p = 0.01");
      nucleoid.run("s = 0.02");
      nucleoid.run("if ( p < 1 ) { r = p * 10 }");
      nucleoid.run("if ( p < 1 ) { r = s * 10 }");
      equal(nucleoid.run("r"), 0.2);

      nucleoid.run("s = 0.03");
      equal(nucleoid.run("r"), 0.3);
    });

    it("creates else statement of variable", () => {
      nucleoid.run("compound = 0.0001");
      nucleoid.run("acidic = 'ACIDIC'");
      nucleoid.run("basic = 'BASIC'");
      nucleoid.run(
        "if ( compound > 0.0000001 ) { pH = acidic } else { pH = basic }"
      );
      nucleoid.run("compound = 0.000000001");
      equal(nucleoid.run("pH"), "BASIC");

      nucleoid.run("basic = '+7'");
      equal(nucleoid.run("pH"), "+7");
    });

    it("creates else if statement of variable", () => {
      nucleoid.run("g = 11");
      nucleoid.run("earth = 9.8");
      nucleoid.run("mars = 3.71");
      nucleoid.run("mass = 10");
      nucleoid.run(
        "if ( g > 9 ) { weight = earth * mass } else if ( g > 3 ) { weight = mars * mass }"
      );
      nucleoid.run("g = 5");
      equal(nucleoid.run("weight"), 37.1);

      nucleoid.run("mars = 3.72");
      equal(nucleoid.run("weight"), 37.2);
    });

    it("creates multiple else if statement of variable", () => {
      nucleoid.run("fraction = -0.1");
      nucleoid.run("point = 1");
      nucleoid.run(
        "if ( fraction > 1 ) { score = fraction * point * 3 } else if ( fraction > 0 ) { score = fraction * point * 2 } else { score = fraction * point }"
      );
      equal(nucleoid.run("score"), -0.1);

      nucleoid.run("point = 2");
      equal(nucleoid.run("score"), -0.2);
    });

    it("runs let statement as a variable", () => {
      nucleoid.run("integer = 30");
      nucleoid.run(
        "{ let division = integer / 10 ; equivalency = division * 10 }"
      );
      equal(nucleoid.run("equivalency"), 30);

      nucleoid.run("integer = 40");
      equal(nucleoid.run("equivalency"), 40);
    });

    it("runs let statement as standard built-in object", () => {
      nucleoid.run("{ let f = new Boolean ( false ) ; condition = f }");
      equal(nucleoid.run("condition"), false);
    });

    it("runs const statement as a variable", () => {
      nucleoid.run("number = 3");
      throws(
        () => {
          nucleoid.run(
            "{ const percentage = number / 100 ; percentage = number / 1000 }"
          );
        },
        (error) =>
          validate(error, TypeError, "Assignment to constant variable.")
      );
    });

    it("creates and assigns instance to let variable inside block", () => {
      nucleoid.run("class Device { }");
      nucleoid.run("$Device.renew = $Device.created + 604800000");

      nucleoid.run(
        "{ let device = new Device ( ) ; device.created = Date.now ( ) }"
      );

      let id = nucleoid.run("Device[0].id");
      equal(nucleoid.run(`${id}.renew - ${id}.created`), 604800000);
    });

    it("creates and assigns instance with constructor to let variable inside block", () => {
      nucleoid.run(
        "class Member { constructor ( first , last ) { this.first = first ; this.last = last } } "
      );
      nucleoid.run("$Member.display = $Member.last + ', ' + $Member.first");
      nucleoid.run("{ let member = new Member ( 'First', 'Last' ) }");
      equal(nucleoid.run("Member[0].display"), "Last, First");
    });

    it("runs new instance of let statement of property", () => {
      nucleoid.run("class Room { }");
      nucleoid.run("class Meeting { }");
      nucleoid.run("room1 = new Room ( )");
      nucleoid.run(
        "$Meeting.time = Date.now ( ) + ' @ ' + $Meeting.date.toDateString()"
      );
      nucleoid.run(
        "{ let meeting = new Meeting ( ) ; meeting.date = new Date ( '2020-1-1' ) ; room1.meeting = meeting }"
      );
      equal(
        nucleoid.run("room1.meeting.date.toDateString()"),
        "Wed Jan 01 2020"
      );
      equal(
        nucleoid.run("room1.meeting.time").substr(-17),
        "@ Wed Jan 01 2020"
      );
    });

    it("runs multiple instance of let statement of property", () => {
      nucleoid.run("class Timesheet { }");
      nucleoid.run("class Task { }");
      nucleoid.run("class Project { }");
      nucleoid.run("$Project.code = 'N-' + $Project.number");
      nucleoid.run("timesheet1 = new Timesheet ( )");
      nucleoid.run(
        "{ let task = new Task ( ) ; task.project = new Project ( ) ; task.project.number = 3668347 ; timesheet1.task = task }"
      );
      equal(nucleoid.run("timesheet1.task.project.number"), 3668347);
      equal(nucleoid.run("timesheet1.task.project.code"), "N-3668347");
    });

    it("creates new object of let statement of class as value before initialization", () => {
      nucleoid.run("class Member { }");
      nucleoid.run(
        "{ let registration = new Object ( ) ; registration.date = new Date ( '2019-1-2' ) ; $Member.registration = registration }"
      );

      nucleoid.run("member1 = new Member ( )");
      equal(
        nucleoid.run("member1.registration.date.toDateString()"),
        "Wed Jan 02 2019"
      );
      equal(nucleoid.run("member1.registration.age"), undefined);
    });

    it("creates new object of let statement of class as value after initialization", () => {
      nucleoid.run("class Distance { }");
      nucleoid.run("distance1 = new Distance ( )");
      nucleoid.run(
        "{ let location = new Object ( ) ; location.coordinates = '40.6976701,-74.2598779' ; $Distance.startingPoint = location }"
      );
      equal(
        nucleoid.run("distance1.startingPoint.coordinates"),
        "40.6976701,-74.2598779"
      );
      equal(nucleoid.run("distance1.startingPoint.print"), undefined);
    });

    it("creates multiple object of let statement of class as value before initialization", () => {
      nucleoid.run("class Account { }");
      nucleoid.run("class Balance { }");
      nucleoid.run("class Currency { }");
      nucleoid.run("$Currency.description = 'Code:' + $Currency.code");
      nucleoid.run(
        "{ let balance = new Object ( ) ; balance.currency = new Object ( ) ; balance.currency.code = 'USD' ; $Account.balance = balance }"
      );
      nucleoid.run("account1 = new Account ( )");
      equal(nucleoid.run("account1.balance.currency.code "), "USD");
      equal(nucleoid.run("account1.balance.currency.description "), undefined);
    });

    it("creates multiple object of let statement of class as value after initialization", () => {
      nucleoid.run("class Warehouse { }");
      nucleoid.run("warehouse1 = new Warehouse ( )");
      nucleoid.run(
        "{ let inventory = new Object ( ) ; inventory.item = new Object ( ) ; inventory.item.sku = '699546085767' ; $Warehouse.inventory = inventory }"
      );
      equal(nucleoid.run("warehouse1.inventory.item.sku"), "699546085767");
      equal(nucleoid.run("warehouse1.inventory.item.description"), undefined);
    });

    it("creates instance inside block", () => {
      nucleoid.run(
        "class Device { constructor ( name ) { this.name = name } }"
      );
      nucleoid.run("$Device.deleted = false");
      nucleoid.run("$Device.key = 'X-' + $Device.name");
      nucleoid.run("{ let name = 'Hall' ; device1 = new Device ( name ) }");

      equal(nucleoid.run("device1.name"), "Hall");
      equal(nucleoid.run("device1.key"), "X-Hall");
      equal(nucleoid.run("device1.deleted"), false);
    });

    it("creates instance inside block without variable name defined", () => {
      nucleoid.run(
        "class Summary { constructor ( rate ) { this.rate = rate } }"
      );
      nucleoid.run("$Summary.score = $Summary.rate * 100");
      nucleoid.run("{ let rate = 4 ; new Summary ( rate ) }");

      equal(nucleoid.run("Summary[0].rate"), 4);
      equal(nucleoid.run("Summary[0].score"), 400);
    });

    it("creates variable inside block", () => {
      nucleoid.run("a = 5 ; b = 10");
      nucleoid.run("if ( a > 9 ) { let c = a + b ; d = c * 10 }");
      nucleoid.run("a = 10");
      equal(nucleoid.run("d"), 200);

      nucleoid.run("a = 15");
      equal(nucleoid.run("d"), 250);

      nucleoid.run("b = 20");
      equal(nucleoid.run("d"), 350);
    });

    it("runs let statement as an object before declaration", () => {
      nucleoid.run("class Plane { }");
      nucleoid.run("class Trip { }");
      nucleoid.run("plane1 = new Plane ( )");
      nucleoid.run("plane1.speed = 903");
      nucleoid.run("trip1 = new Trip ( )");
      nucleoid.run("trip1.distance = 5540");
      nucleoid.run(
        "{ let trip = $Plane.trip ; $Plane.time = trip.distance / $Plane.speed }"
      );
      nucleoid.run("plane1.trip = trip1");
      equal(nucleoid.run("plane1.time"), 6.135105204872647);
    });

    it("runs let statement as an object after declaration", () => {
      nucleoid.run("class Seller { }");
      nucleoid.run("class Commission { }");
      nucleoid.run("seller1 = new Seller ( )");
      nucleoid.run("seller1.sales = 1000000");
      nucleoid.run("comm1 = new Commission ( )");
      nucleoid.run("comm1.rate = 0.05");
      nucleoid.run("seller1.commission = comm1");
      nucleoid.run(
        "{ let commission = $Seller.commission ; $Seller.pay = $Seller.sales * commission.rate }"
      );
      equal(nucleoid.run("seller1.pay"), 50000);
    });

    it.skip("reassigns let statement before initialization", () => {
      nucleoid.run("class Order { }");
      nucleoid.run("class Sale { }");
      nucleoid.run(
        "{ let sale = $Order.sale ; sale.amount = sale.percentage / $Order.amount * 100 }"
      );
      nucleoid.run("order1 = new Order ( )");
      nucleoid.run("order1.amount = 100");
      nucleoid.run("sale1 = new Sale ( )");
      nucleoid.run("sale1.percentage = 10");
      nucleoid.run("order1.sale = sale1");
      equal(nucleoid.run("sale1.amount"), 10);
    });

    it("reassigns let statement after initialization", () => {
      nucleoid.run("class Stock { }");
      nucleoid.run("class Trade { }");
      nucleoid.run("stock1 = new Stock ( )");
      nucleoid.run("stock1.price = 100");
      nucleoid.run("trade1 = new Trade ( )");
      nucleoid.run("trade1.quantity = 1");
      nucleoid.run("stock1.trade = trade1");
      nucleoid.run(
        "{ let trade = $Stock.trade ; trade.worth = $Stock.price * trade.quantity }"
      );
      equal(nucleoid.run("trade1.worth"), 100);
    });

    it("holds result of function in let", () => {
      nucleoid.run("bugs = [ ]");
      nucleoid.run("ticket = 1");
      nucleoid.run("class Bug { }");
      nucleoid.run("bug1 = new Bug ( )");
      nucleoid.run("bug1.ticket = 1");
      nucleoid.run("bug1.priority = 'LOW'");
      nucleoid.run("bugs.push ( bug1 )");
      nucleoid.run("bug2 = new Bug ( )");
      nucleoid.run("bug2.ticket = 2");
      nucleoid.run("bug2.priority = 'MEDIUM'");
      nucleoid.run("bugs.push ( bug2 )");
      nucleoid.run(
        "{ let bug = bugs.find ( it => it.ticket == ticket ) ; bug.selected = true }"
      );
      equal(nucleoid.run("bug1.selected"), true);
      equal(nucleoid.run("bug2.selected"), undefined);

      nucleoid.run("ticket = 2");
      equal(nucleoid.run("bug2.selected"), true);
    });

    it("skips if block is empty", () => {
      nucleoid.run("{ }");
    });

    it("runs block statement of variable", () => {
      nucleoid.run("h = 1");
      nucleoid.run("{ let value = h * 2 ; j = value * 2 }");
      equal(nucleoid.run("j"), 4);

      nucleoid.run("h = 2");
      equal(nucleoid.run("j"), 8);
    });

    it("runs nested block statement of variable", () => {
      nucleoid.run("radius = 10");
      nucleoid.run(
        "{ let area = Math.pow ( radius , 2 ) * 3.14 ; { volume = area * 5 } }"
      );
      equal(nucleoid.run("volume"), 1570);
    });

    it("runs nested if statement of variable", () => {
      nucleoid.run("gravity = 9.8");
      nucleoid.run("time = 10");
      nucleoid.run("distance = 480");
      nucleoid.run("target = true");
      nucleoid.run(
        "{ let dist = 1 / 2 * gravity * time * time ; if ( dist > distance ) { hit = target } }"
      );
      equal(nucleoid.run("hit"), true);

      nucleoid.run("target = false");
      equal(nucleoid.run("hit"), false);
    });

    it("runs nested else statement of variable", () => {
      nucleoid.run("percentage = 28");
      nucleoid.run("density = 0.899");
      nucleoid.run("substance = 'NH3'");
      nucleoid.run("molarConcentration = null");
      nucleoid.run("def = 0");
      nucleoid.run(
        "{ let concentration = percentage * density / 100 * 1000 ; if ( substance == 'NH3' ) { molarConcentration = concentration / 17.04 } else { molarConcentration = def } }"
      );
      nucleoid.run("substance = 'NH16'");
      nucleoid.run("def = 1");
      equal(nucleoid.run("molarConcentration"), 1);
    });

    it("assigns variable to reference", () => {
      nucleoid.run("a = 1");
      nucleoid.run("b = a");
      equal(nucleoid.run("b"), 1);

      nucleoid.run("a = 2");
      equal(nucleoid.run("b"), 2);
    });

    it("assigns object to variable", () => {
      nucleoid.run("class Model { }");
      nucleoid.run("model1 = new Model ( )");
      equal(nucleoid.run("typeof model1"), "object");
    });

    it("defines class in the state", () => {
      nucleoid.run("class Entity { }");
      equal(nucleoid.run("typeof $Entity"), "function");
    });

    it("rejects creating instance if the class does not exist", () => {
      throws(
        () => {
          nucleoid.run("chart1 = new Chart ( )");
        },
        (error) => validate(error, ReferenceError, "Chart is not defined")
      );

      nucleoid.run("class Chart { }");
      nucleoid.run("chart1 = new Chart ( )");
      throws(
        () => {
          nucleoid.run("chart1.plot = new Plot ( )");
        },
        (error) => validate(error, ReferenceError, "Plot is not defined")
      );

      throws(
        () => {
          nucleoid.run("$Chart.plot = new Plot ( )");
        },
        (error) => validate(error, ReferenceError, "Plot is not defined")
      );
    });

    it("creates property assignment before declaration", () => {
      nucleoid.run("class Order { }");
      nucleoid.run("var order1 = new Order ( )");
      nucleoid.run("order1.upc = '04061' + order1.barcode");
      nucleoid.run("order1.barcode = '94067'");
      equal(nucleoid.run("order1.upc"), "0406194067");
    });

    it("creates property assignment after declaration", () => {
      nucleoid.run("class User { }");
      nucleoid.run("user = new User ( )");
      nucleoid.run("user.name = 'sample'");
      nucleoid.run("user.email = user.name + '@example.com'");
      equal(nucleoid.run("user.email"), "sample@example.com");

      nucleoid.run("user.name = 'samplex'");
      equal(nucleoid.run("user.email"), "samplex@example.com");
    });

    it("creates property assignment only if instance is defined", () => {
      nucleoid.run("class Channel { }");
      nucleoid.run("channel1 = new Channel ( )");
      throws(
        () => {
          nucleoid.run("channel1.frequency.type = 'ANGULAR'");
        },
        (error) =>
          validate(error, ReferenceError, "channel1.frequency is not defined")
      );
    });

    it("creates object with var statement", () => {
      nucleoid.run("class Item { constructor ( name ) { this.name  = name } }");

      nucleoid.run("var item1 = new Item ( 'NAME-1' )");
      deepEqual(nucleoid.run("item1"), { id: "item1", name: "NAME-1" });

      nucleoid.run("var item2 = new Item ( )");
      deepEqual(nucleoid.run("item2"), { id: "item2", name: undefined });
    });

    it("creates object assignment as property only if instance is defined", () => {
      nucleoid.run("class Worker { }");
      nucleoid.run("class Schedule { }");
      nucleoid.run("worker1 = new Worker ( )");
      throws(
        () => {
          nucleoid.run("worker1.duty.schedule = new Schedule ( )");
        },

        (error) =>
          validate(error, ReferenceError, "worker1.duty is not defined")
      );
    });

    it("uses its value when self property used", () => {
      nucleoid.run("class Construction { }");
      nucleoid.run("construction1 = new Construction ( ) ");
      nucleoid.run("construction1.timeline = 120");
      nucleoid.run("construction1.timeline = 2 * construction1.timeline");
      equal(nucleoid.run("construction1.timeline"), 240);
    });

    it("assigns object to property before initialization", () => {
      nucleoid.run("class Agent { }");
      nucleoid.run("class Distance { }");
      nucleoid.run(
        "$Distance.total = Math.sqrt ( $Distance.x * $Distance.x + $Distance.y * $Distance.y )"
      );
      nucleoid.run("agent1 = new Agent ( )");
      nucleoid.run("agent1.distance = new Distance ( )");
      nucleoid.run("agent1.distance.x = 3");
      nucleoid.run("agent1.distance.y = 4");
      equal(nucleoid.run("agent1.distance.total"), 5);
    });

    it("assigns object to property after initialization", () => {
      nucleoid.run("class Product { }");
      nucleoid.run("product1 = new Product ( )");
      nucleoid.run("class Quality { }");
      nucleoid.run("product1.quality = new Quality ( )");
      nucleoid.run("product1.quality.score = 15");
      nucleoid.run(
        "$Quality.class = String.fromCharCode ( 65 + Math.floor ( $Quality.score / 10 ) )"
      );
      equal(nucleoid.run("product1.quality.class"), "B");
    });

    it("rejects if name of instance as property is value", () => {
      nucleoid.run("class Schedule { }");
      nucleoid.run("class Place { }");
      nucleoid.run("value = new Schedule ( )");
      throws(
        () => {
          nucleoid.run("value.value = new Place ( )");
        },
        (error) =>
          validate(error, TypeError, "Cannot use 'value' as a property")
      );
    });

    it("rejects if property name is value", () => {
      nucleoid.run("class Value { }");
      nucleoid.run("value = new Value ( )");
      throws(
        () => {
          nucleoid.run("value.value = 2147483647");
        },
        (error) => validate(error, TypeError, "Cannot use 'value' as a name")
      );
    });

    it("uses value property to indicate using only value of property", () => {
      nucleoid.run("class Weight { }");
      nucleoid.run("weight1 = new Weight ( )");
      nucleoid.run("weight1.gravity = 1.352");
      nucleoid.run("weight1.mass = 1000");
      nucleoid.run("weight1.force = weight1.gravity * weight1.mass.value");
      equal(nucleoid.run("weight1.force"), 1352);

      nucleoid.run("weight1.mass = 2000");
      equal(nucleoid.run("weight1.force"), 1352);
    });

    it("uses value property in if condition to indicate using only value of variable", () => {
      nucleoid.run("class Question { }");
      nucleoid.run("question1 = new Question ( )");
      nucleoid.run("question1.text = 'How was the service?'");
      nucleoid.run(
        "if ( question1.text != question1.text.value ) { throw 'QUESTION_ARCHIVED' }"
      );
      throws(
        () => {
          nucleoid.run("question1.text = 'How would you rate us?'");
        },
        (error) => error === "QUESTION_ARCHIVED"
      );
    });

    it.skip("rejects value of property if property is not defined", () => {
      nucleoid.run("class Travel { }");
      nucleoid.run("travel1 = new Travel ( )");
      nucleoid.run("travel1.speed = 65");
      throws(
        () => {
          nucleoid.run("travel1.time = travel1.distance.value / travel1.speed");
        },
        (error) =>
          validate(error, ReferenceError, "travel1.distance is not defined")
      );
    });

    it("keeps as null if value of property is null", () => {
      nucleoid.run("class Interest { }");
      nucleoid.run("interest1 = new Interest ( )");
      nucleoid.run("interest1.rate = 3");
      nucleoid.run("interest1.amount = null");
      nucleoid.run(
        "interest1.annual = interest1.rate * interest1.amount.value / 100"
      );
      equal(nucleoid.run("interest1.annual"), 0);

      nucleoid.run("interest1.amount = 10000");
      equal(nucleoid.run("interest1.annual"), 0);
    });

    it("rejects if property of local name is value", () => {
      nucleoid.run("class Alarm { }");
      throws(
        () => {
          nucleoid.run("{ let value = new Alarm ( ) ; value.value = '22:00' }");
        },
        (error) => validate(error, TypeError, "Cannot use 'value' in local")
      );
    });

    it("keeps same as its value when value property used for local", () => {
      nucleoid.run("speedOfLight = 299792");
      nucleoid.run(
        "{ let time = speedOfLight / 225623 ; roundTrip = time.value * 2 }"
      );
      equal(nucleoid.run("roundTrip"), 2.6574595675086314);
    });

    it("uses value property as part of class declaration", () => {
      nucleoid.run("count = 0");
      nucleoid.run("class Device { }");
      nucleoid.run("device1 = new Device ( )");
      nucleoid.run("{ $Device.code = 'A' + count.value ; count = count + 1 }");
      equal(nucleoid.run("device1.code"), "A0");
    });

    it("uses value property of class declaration", () => {
      nucleoid.run(
        "class Summary { constructor ( question ) { this.question = question } }"
      );
      nucleoid.run("class Question { }");
      nucleoid.run("$Summary.count = $Summary.question.count.value");
      nucleoid.run("question1 = new Question ( )");
      nucleoid.run("question1.count = 10");
      nucleoid.run("summary1 = new Summary ( question1 )");
      equal(nucleoid.run("summary1.count"), 10);

      nucleoid.run("question1.count = 11");
      equal(nucleoid.run("summary1.count"), 10);
    });

    it("updates if block of property", () => {
      nucleoid.run("class Account { }");
      nucleoid.run("account = new Account ( )");
      nucleoid.run("account.balance = 1000");
      nucleoid.run("if ( account.balance < 1500 ) { account.status = 'OK' }");
      equal(nucleoid.run("account.status"), "OK");

      nucleoid.run("if ( account.balance < 1500 ) { account.status = 'LOW' }");
      equal(nucleoid.run("account.status"), "LOW");
    });

    it("creates if statement of property", () => {
      nucleoid.run("class Toy { }");
      nucleoid.run("toy = new Toy ( )");
      nucleoid.run("toy.color = 'BLUE'");
      nucleoid.run("if ( toy.color == 'RED' ) { toy.shape = 'CIRCLE' }");
      nucleoid.run("toy.color = 'RED'");
      equal(nucleoid.run("toy.shape"), "CIRCLE");
    });

    it("creates else statement of property", () => {
      nucleoid.run("class Engine { }");
      nucleoid.run("engine1 = new Engine ( )");
      nucleoid.run("engine1.type = 'V8'");
      nucleoid.run("mpl = 'MPL'");
      nucleoid.run("bsd = 'BSD'");
      nucleoid.run(
        "if ( engine1.type == 'Gecko' ) { engine1.license = mpl } else { engine1.license = bsd }"
      );
      equal(nucleoid.run("engine1.license"), "BSD");

      nucleoid.run("bsd = 'Berkeley Software Distribution'");
      equal(nucleoid.run("engine1.license"), "Berkeley Software Distribution");
    });

    it("creates else if statement of property", () => {
      nucleoid.run("class Contact { }");
      nucleoid.run("contact1 = new Contact ( )");
      nucleoid.run("contact1.type = 'PERSON'");
      nucleoid.run("contact1.first = 'First'");
      nucleoid.run("contact1.last = 'Last'");
      nucleoid.run(
        "if ( contact1.type == 'BUSINESS' ) { contact1.full = 'B' + contact1.first } else { contact1.full = contact1.first + ' ' + contact1.last }"
      );
      equal(nucleoid.run("contact1.full"), "First Last");

      nucleoid.run("contact1.first = 'F' ; contact1.last = 'L'");
      equal(nucleoid.run("contact1.full"), "F L");
    });

    it("creates multiple else if statement of property", () => {
      nucleoid.run("class Taxpayer { }");
      nucleoid.run("taxpayer1 = new Taxpayer ( )");
      nucleoid.run("taxpayer1.income = 60000");
      nucleoid.run("taxpayer1.member = 1");
      nucleoid.run("rate = 22");
      nucleoid.run(
        "if ( taxpayer1.member > 4 ) { taxpayer1.tax = taxpayer1.income * rate / 100 - 2000 } else if ( taxpayer1.member > 2 ) { taxpayer1.tax = taxpayer1.income * rate / 100 - 1000 } else { taxpayer1.tax = taxpayer1.income * rate / 100 }"
      );
      equal(nucleoid.run("taxpayer1.tax"), 13200);

      nucleoid.run("rate = 23");
      equal(nucleoid.run("taxpayer1.tax"), 13800);
    });

    it("updates property assignment", () => {
      nucleoid.run("class Matter { }");
      nucleoid.run("matter1 = new Matter ( )");
      nucleoid.run("matter1.mass = 10");
      nucleoid.run("matter1.weight = matter1.mass * 9.8");
      equal(nucleoid.run("matter1.weight"), 98);

      nucleoid.run("matter1.weight = matter1.mass * 3.7");
      equal(nucleoid.run("matter1.weight"), 37);

      nucleoid.run("matter1.mass = 20");
      equal(nucleoid.run("matter1.weight"), 74);
    });

    it("deletes instance", () => {
      nucleoid.run("class Circle { }");
      nucleoid.run("circle1 = new Circle ( )");
      nucleoid.run("delete circle1");
      equal(nucleoid.run("Circle['circle1']"), undefined);
      equal(
        nucleoid.run("Circle.find ( circle => circle.id === 'circle1' )"),
        undefined
      );

      throws(
        () => {
          nucleoid.run("circle1");
        },
        (error) => validate(error, ReferenceError, "circle1 is not defined")
      );
    });

    it("deletes instance by reference", () => {
      nucleoid.run("class Item { }");
      nucleoid.run("item1 = new Item ( )");
      nucleoid.run("item2 = new Item ( )");
      deepEqual(nucleoid.run("Item['item1']"), { id: "item1" });

      nucleoid.run("delete Item['item1']");
      equal(nucleoid.run("Item['item1']"), undefined);

      deepEqual(nucleoid.run("Item['item2']"), { id: "item2" });
      nucleoid.run("let item = 'item2' ; delete Item[item]");
      equal(nucleoid.run("Item['item2']"), undefined);
    });

    it("returns boolean when deleting object", () => {
      nucleoid.run("class Location { }");
      nucleoid.run("location1 = new Location ( )");
      equal(nucleoid.run("delete location1"), true);
      equal(nucleoid.run("delete location2"), false);
    });

    it("rejects deleting instance if it has any properties", () => {
      nucleoid.run("class Channel { }");
      nucleoid.run("channel1 = new Channel ( )");
      nucleoid.run("channel1.frequency = 440");
      throws(
        () => {
          nucleoid.run("delete channel1");
        },
        (error) =>
          validate(error, ReferenceError, "Cannot delete object 'channel1'")
      );
      equal(nucleoid.run("channel1.frequency "), 440);

      nucleoid.run("delete channel1.frequency");
      nucleoid.run("delete channel1");
    });

    it("rejects deleting instance if it has object as a property", () => {
      nucleoid.run("class Shape { }");
      nucleoid.run("class Type { }");
      nucleoid.run("shape1 = new Shape ( )");
      nucleoid.run("shape1.type = new Type ( )");
      throws(
        () => {
          nucleoid.run("delete shape1");
        },
        (error) =>
          validate(error, ReferenceError, "Cannot delete object 'shape1'")
      );

      nucleoid.run("delete shape1.type");
      nucleoid.run("delete shape1");
    });

    it("deletes property assignment", () => {
      nucleoid.run("class Agent { }");
      nucleoid.run("agent = new Agent ( )");
      nucleoid.run("agent.time = 52926163455");
      nucleoid.run("agent.location = 'CITY'");
      nucleoid.run("agent.report = agent.time + '@' + agent.location");
      equal(nucleoid.run("agent.report"), "52926163455@CITY");

      nucleoid.run("delete agent.time");
      equal(nucleoid.run("agent.report"), undefined);

      nucleoid.run("delete agent.report");
      equal(nucleoid.run("agent.report"), undefined);
    });

    it("runs block statement of property", () => {
      nucleoid.run("class Item { }");
      nucleoid.run("item1 = new Item ( )");
      nucleoid.run("item1.sku = '0000001' ");
      nucleoid.run("{ let custom = 'US' + item1.sku ; item1.custom = custom }");
      equal(nucleoid.run("item1.custom"), "US0000001");

      nucleoid.run("item1.sku = '0000002' ");
      equal(nucleoid.run("item1.custom"), "US0000002");
    });

    it.skip("runs nested block statement of property", () => {
      nucleoid.run("class Figure { }");
      nucleoid.run("figure1 = new Figure ( )");
      nucleoid.run("figure1.width = 9");
      nucleoid.run("figure1.height = 10");
      nucleoid.run(
        "{ let base = Math.pow ( figure1.width , 2 ) ; { figure1.volume = base * figure1.height } }"
      );
      equal(nucleoid.run("figure1.volume"), 810);

      nucleoid.run("figure1.height = 9");
      equal(nucleoid.run("figure1.volume"), 729);
    });

    it("runs nested if statement of property", () => {
      nucleoid.run("class Sale { }");
      nucleoid.run("sale1 = new Sale ( )");
      nucleoid.run("sale1.price = 50");
      nucleoid.run("sale1.quantity = 2");
      nucleoid.run(
        "{ let amount = sale1.price * sale1.quantity ; if ( amount > 100 ) { sale1.tax = amount * 10 / 100 } }"
      );
      equal(nucleoid.run("sale1.tax"), undefined);

      nucleoid.run("sale1.quantity = 3");
      equal(nucleoid.run("sale1.tax"), 15);
    });

    it("creates nested else statement of property", () => {
      nucleoid.run("class Chart { }");
      nucleoid.run("chart1 = new Chart ( )");
      nucleoid.run("chart1.percentage = 1");
      nucleoid.run("invalid = 'INVALID'");
      nucleoid.run("valid = 'VALID'");
      nucleoid.run(
        "{ let ratio = chart1.percentage / 100 ; if ( ratio > 1 ) { chart1.status = invalid } else { chart1.status = valid } }"
      );
      equal(nucleoid.run("chart1.status"), "VALID");

      nucleoid.run("valid = 'V'");
      equal(nucleoid.run("chart1.status"), "V");
    });

    it("creates property assignment with multiple properties", () => {
      nucleoid.run("class Person { }");
      nucleoid.run("person1 = new Person ( )");
      nucleoid.run("class Address { }");
      nucleoid.run("address1 = new Address ( )");
      nucleoid.run("$Address.print = $Address.city + ', ' + $Address.state");
      nucleoid.run("person1.address = new Address ( )");
      nucleoid.run("person1.address.city = 'Syracuse'");
      nucleoid.run("person1.address.state = 'NY'");
      equal(nucleoid.run("person1.address.print"), "Syracuse, NY");
    });

    it("creates property assignment as multiple properties as part of declaration", () => {
      nucleoid.run("class Server { }");
      nucleoid.run("server1 = new Server ( )");
      nucleoid.run("server1.name = 'HOST1'");
      nucleoid.run("class IP { }");
      nucleoid.run("ip1 = new IP ( )");
      nucleoid.run("server1.ip = ip1");
      nucleoid.run("ip1.address = '10.0.0.1'");
      nucleoid.run("server1.summary = server1.name + '@' + server1.ip.address");
      equal(nucleoid.run("server1.summary"), "HOST1@10.0.0.1");

      nucleoid.run("ip1.address = '10.0.0.2'");
      equal(nucleoid.run("server1.summary"), "HOST1@10.0.0.2");
    });

    it("creates dependency behalf if property has reference", () => {
      nucleoid.run("class Schedule { }");
      nucleoid.run("schedule1 = new Schedule ( )");

      nucleoid.run("class Template { }");
      nucleoid.run("template1 = new Template ( )");
      nucleoid.run("template1.type = 'W'");

      nucleoid.run("schedule1.template = template1");
      nucleoid.run(
        "schedule1.template.name = schedule1.template.type + '-0001'"
      );
      equal(nucleoid.run("template1.name"), "W-0001");
      equal(nucleoid.run("schedule1.template.name"), "W-0001");

      nucleoid.run("template1.type = 'D'");
      equal(nucleoid.run("template1.name"), "D-0001");

      nucleoid.run("template1.shape = template1.type + '-Form'");
      equal(nucleoid.run("template1.shape"), "D-Form");
      equal(nucleoid.run("schedule1.template.shape"), "D-Form");

      nucleoid.run("template1.type = 'C'");
      equal(nucleoid.run("template1.shape"), "C-Form");
      equal(nucleoid.run("schedule1.template.shape"), "C-Form");
    });

    it("creates dependency behalf if let has reference", () => {
      nucleoid.run("class Vote { }");
      nucleoid.run("vote1 = new Vote ( )");
      nucleoid.run("vote1.rate = 4");
      nucleoid.run("class Question { }");
      nucleoid.run("$Question.rate = 0");
      nucleoid.run("$Question.count = 0");
      nucleoid.run("question1 = new Question ( )");
      nucleoid.run("vote1.question = question1");
      nucleoid.run(
        "{ let question = vote1.question ; question.rate = ( question.rate * question.count + vote1.rate ) / ( question.count + 1 ) ; question.count =  question.count + 1}"
      );
      equal(nucleoid.run("question1.rate"), 4);
      equal(nucleoid.run("question1.count"), 1);

      nucleoid.run("vote1.rate = 5");
      equal(nucleoid.run("question1.rate"), 4.5);
    });

    it("runs expression statement of class", () => {
      nucleoid.run("class Element { }");
      nucleoid.run("alkalis = [ ]");
      nucleoid.run("element1 = new Element ( )");
      nucleoid.run("element1.number = 3");
      nucleoid.run(
        "{ let number = $Element.number ; if ( number == 3 ) { alkalis.push ( $Element ) } }"
      );
      equal(nucleoid.run("alkalis.pop ( )"), nucleoid.run("element1"));
    });

    it("creates class assignment before initialization", () => {
      nucleoid.run("class Review { }");
      nucleoid.run("$Review.rate = $Review.sum / 10");
      nucleoid.run("review1 = new Review ( )");
      nucleoid.run("review1.sum = 42");
      equal(nucleoid.run("review1.rate"), 4.2);
    });

    it("creates class assignment after initialization", () => {
      nucleoid.run("class Shape { }");
      nucleoid.run("s1 = new Shape ( )");
      nucleoid.run("s1.edge = 3");
      nucleoid.run("s2 = new Shape ( )");
      nucleoid.run("s2.edge = 3");
      nucleoid.run("$Shape.angle = ( $Shape.edge - 2 ) * 180");
      nucleoid.run("s1.edge = 4");
      equal(nucleoid.run("s1.angle"), 360);
      equal(nucleoid.run("s2.angle"), 180);
    });

    it("updates class assignment", () => {
      nucleoid.run("class Employee { }");
      nucleoid.run("employee = new Employee ( )");
      nucleoid.run("employee.id = 1");
      nucleoid.run("$Employee.username = 'E' + $Employee.id");
      equal(nucleoid.run("employee.username"), "E1");
      nucleoid.run("$Employee.username = 'F' + $Employee.id");
      nucleoid.run("employee.id = 2");
      equal(nucleoid.run("employee.username"), "F2");
    });

    it("creates if statement of class before initialization", () => {
      nucleoid.run("class Ticket { }");
      nucleoid.run(
        "if ( $Ticket.date > new Date ( '1993-1-1' ) ) { $Ticket.status = 'EXPIRED' }"
      );
      nucleoid.run("ticket1 = new Ticket ( )");

      equal(nucleoid.run("ticket1.status"), undefined);
      nucleoid.run("ticket1.date = new Date ( '1993-2-1' ) ");
      equal(nucleoid.run("ticket1.status"), "EXPIRED");

      nucleoid.run("ticket2 = new Ticket ( )");
      equal(nucleoid.run("ticket2.status"), undefined);
    });

    it("creates if statement of class after initialization", () => {
      nucleoid.run("class Student { }");
      nucleoid.run("s1 = new Student ( )");
      nucleoid.run("s1.age = 2");
      nucleoid.run("s1.class = 'Daycare'");
      nucleoid.run("s2 = new Student ( )");
      nucleoid.run("s2.age = 2");
      nucleoid.run("s2.class = 'Daycare'");
      nucleoid.run("if ( $Student.age == 3 ) { $Student.class = 'Preschool' }");
      nucleoid.run("s1.age = 3");
      equal(nucleoid.run("s1.class"), "Preschool");
      equal(nucleoid.run("s2.class"), "Daycare");
    });

    it("updates if block of class", () => {
      nucleoid.run("class Inventory { }");
      nucleoid.run("i1 = new Inventory ( )");
      nucleoid.run("i1.quantity = 0");

      nucleoid.run("i2 = new Inventory ( )");
      nucleoid.run("i2.quantity = 1000");

      nucleoid.run(
        "if ( $Inventory.quantity == 0 ) { $Inventory.replenishment = true }"
      );
      equal(nucleoid.run("i1.replenishment"), true);
      equal(nucleoid.run("i2.replenishment"), undefined);
      nucleoid.run(
        "if ( $Inventory.quantity == 0 ) { $Inventory.replenishment = false }"
      );

      equal(nucleoid.run("i1.replenishment"), false);
      equal(nucleoid.run("i2.replenishment"), undefined);
    });

    it("creates else statement of class before initialization", () => {
      nucleoid.run("class Count { }");
      nucleoid.run(
        "if ( $Count.max > 1000 ) { $Count.reset = urgent } else { $Count.reset = regular }"
      );
      nucleoid.run("urgent = 'URGENT'");
      nucleoid.run("regular = 'REGULAR'");
      nucleoid.run("count1 = new Count ( )");
      nucleoid.run("count1.max = 850");
      equal(nucleoid.run("count1.reset"), "REGULAR");

      nucleoid.run("regular = 'R'");
      equal(nucleoid.run("count1.reset"), "R");
    });

    it("creates else statement of class after initialization", () => {
      nucleoid.run("class Concentration { }");
      nucleoid.run("serialDilution = '(c1V1+c2V2)/(V1+V2)'");
      nucleoid.run("directDilution = 'c1/V1'");
      nucleoid.run("concentration1 = new Concentration ( )");
      nucleoid.run("concentration1.substances = 2");
      nucleoid.run(
        "if ( $Concentration.substances == 1 ) { $Concentration.formula = directDilution } else { $Concentration.formula = serialDilution }"
      );
      equal(nucleoid.run("concentration1.formula"), "(c1V1+c2V2)/(V1+V2)");

      nucleoid.run("serialDilution = '(c1V1+c2V2+c3V3)/(V1+V2+V3)'");
      equal(
        nucleoid.run("concentration1.formula"),
        "(c1V1+c2V2+c3V3)/(V1+V2+V3)"
      );
    });

    it("creates else if statement of class before initialization", () => {
      nucleoid.run("class Storage { }");
      nucleoid.run("normal = 'NORMAL' ; low = 'LOW'");
      nucleoid.run(
        "if ( $Storage.capacity > 25 ) { $Storage.status = normal } else { $Storage.status = low }"
      );
      nucleoid.run("storage1 = new Storage ( )");
      nucleoid.run("storage1.capacity = 23");
      equal(nucleoid.run("storage1.status"), "LOW");

      nucleoid.run("low = 'L'");
      equal(nucleoid.run("storage1.status"), "L");
    });

    it("creates else if statement of class after initialization", () => {
      nucleoid.run("class Registration { }");
      nucleoid.run("yes = 'YES' ; no = 'NO'");
      nucleoid.run("registration1 = new Registration ( )");
      nucleoid.run("registration1.available = 0");
      nucleoid.run(
        "if ( $Registration.available > 0 ) { $Registration.accepted = yes } else { $Registration.accepted = no }"
      );
      equal(nucleoid.run("registration1.accepted"), "NO");

      nucleoid.run("yes = true ; no = false");
      equal(nucleoid.run("registration1.accepted"), false);
    });

    it("creates multiple else if statement of class before initialization", () => {
      nucleoid.run("class Capacity { }");
      nucleoid.run(
        "if ( $Capacity.spare / $Capacity.available > 0.5 ) { $Capacity.total = $Capacity.available + $Capacity.spare } else if ( $Capacity.spare / $Capacity.available > 0.1 ) { $Capacity.total = $Capacity.available + $Capacity.spare * 2 } else { $Capacity.total = $Capacity.available + $Capacity.spare * 3 }"
      );
      nucleoid.run("capacity1 = new Capacity ( )");
      nucleoid.run("capacity1.available = 100");
      nucleoid.run("capacity1.spare = 5");
      equal(nucleoid.run("capacity1.total"), 115);

      nucleoid.run("capacity1.spare = 1");
      equal(nucleoid.run("capacity1.total"), 103);
    });

    it("creates multiple else if statement of class after initialization", () => {
      nucleoid.run("class Shape { }");
      nucleoid.run("shape1 = new Shape ( )");
      nucleoid.run("shape1.type = 'RECTANGLE'");
      nucleoid.run("shape1.x = 5");
      nucleoid.run("shape1.y = 6");
      nucleoid.run(
        "if ( $Shape.type == 'SQUARE' ) { $Shape.area = Math.pow( Shape.x, 2 ) } else if ( $Shape.type == 'TRIANGLE' ) { $Shape.area = $Shape.x * $Shape.y / 2 } else { $Shape.area = $Shape.x * $Shape.y }"
      );
      equal(nucleoid.run("shape1.area"), 30);

      nucleoid.run("shape1.x = 7");
      equal(nucleoid.run("shape1.area"), 42);
    });

    it("runs block statement of class before initialization", () => {
      nucleoid.run("class Stock { }");
      nucleoid.run(
        "{ let change = $Stock.before * 4 / 100 ; $Stock.after = $Stock.before + change }"
      );
      nucleoid.run("stock1 = new Stock ( )");
      nucleoid.run("stock1.before = 57.25");
      equal(nucleoid.run("stock1.after"), 59.54);

      nucleoid.run("stock1.before = 59.50");
      equal(nucleoid.run("stock1.after"), 61.88);
    });

    it("runs block statement of class after initialization", () => {
      nucleoid.run("class Purchase { }");
      nucleoid.run("purchase = new Purchase ( )");
      nucleoid.run("purchase.price = 99");
      nucleoid.run(
        "{ let retailPrice = $Purchase.price * 1.15 ; $Purchase.retailPrice = retailPrice }"
      );
      equal(nucleoid.run("purchase.retailPrice"), 113.85);

      nucleoid.run("purchase.price = 199");
      equal(nucleoid.run("purchase.retailPrice"), 228.85);
    });

    // TODO It will be fixed with new writing strategy
    it.skip("runs nested block statement of class before initialization", () => {
      nucleoid.run("class Compound { }");
      nucleoid.run(
        "{ let mol = 69.94 / $Compound.substance ; { $Compound.sample = Math.floor ( mol * $Compound.mol ) } }"
      );
      nucleoid.run("compound1 = new Compound ( )");
      nucleoid.run("compound1.substance = 55.85");
      nucleoid.run("compound1.mol = 1000");
      equal(nucleoid.run("compound1.sample"), 1252);
    });

    it("runs nested block statement of class after initialization", () => {
      nucleoid.run("class Bug { }");
      nucleoid.run("bug1 = new Bug ( )");
      nucleoid.run("bug1.initialScore = 1000");
      nucleoid.run("bug1.aging = 24");
      nucleoid.run(
        "{ let score = $Bug.aging * 10 ; { $Bug.priorityScore = score + $Bug.initialScore } }"
      );
      equal(nucleoid.run("bug1.priorityScore"), 1240);
    });

    it("runs nested if statement of class before initialization", () => {
      nucleoid.run("class Mortgage { }");
      nucleoid.run(
        "{ let interest = $Mortgage.annual / 12 ; if ( interest < 4 ) { $Mortgage.rate = rate1 } }"
      );
      nucleoid.run("rate1 = 'EXCEPTIONAL'");
      nucleoid.run("mortgage1 = new Mortgage ( )");
      nucleoid.run("mortgage1.annual = 46");
      equal(nucleoid.run("mortgage1.rate"), "EXCEPTIONAL");

      nucleoid.run("rate1 = 'E'");
      equal(nucleoid.run("mortgage1.rate"), "E");
    });

    it("runs nested if statement of class after initialization", () => {
      nucleoid.run("class Building { }");
      nucleoid.run("buildingType1 = 'SKYSCRAPER'");
      nucleoid.run("building1 = new Building ( )");
      nucleoid.run("building1.floors = 20");
      nucleoid.run(
        "{ let height = $Building.floors * 14 ; if ( height > 330 ) { $Building.type = buildingType1 } }"
      );
      equal(nucleoid.run("building1.type"), undefined);

      nucleoid.run("building1.floors = 25");
      equal(nucleoid.run("building1.type"), "SKYSCRAPER");

      nucleoid.run("buildingType1 = 'S'");
      equal(nucleoid.run("building1.type"), "S");
    });

    it("creates nested else statement of class before initialization", () => {
      nucleoid.run("class Account { }");
      nucleoid.run("noAlert = 'NO_ALERT'");
      nucleoid.run("lowAlert = 'LOW_ALERT'");
      nucleoid.run(
        "{ let balance = $Account.balance ; if ( balance > 1000 ) { $Account.alert = noAlert } else { $Account.alert = lowAlert } }"
      );
      nucleoid.run("account1 = new Account ( )");
      nucleoid.run("account1.balance = 950");
      equal(nucleoid.run("account1.alert"), "LOW_ALERT");

      nucleoid.run("lowAlert = 'L'");
      equal(nucleoid.run("account1.alert"), "L");
    });

    it("creates nested else statement of class after initialization", () => {
      nucleoid.run("class Question { }");
      nucleoid.run("high = 'HIGH'");
      nucleoid.run("low = 'LOW'");
      nucleoid.run("question1 = new Question ( )");
      nucleoid.run("question1.count = 1");
      nucleoid.run(
        "{ let score = $Question.count * 10 ; if ( score > 100 ) { $Question.type = high } else { $Question.type = low } }"
      );
      equal(nucleoid.run("question1.type"), "LOW");

      nucleoid.run("low = 'L'");
      equal(nucleoid.run("question1.type"), "L");
    });

    it("creates class assignment with multiple properties before declaration", () => {
      nucleoid.run("class Room { }");
      nucleoid.run("$Room.level = $Room.number / 10");
      nucleoid.run("class Guest { }");
      nucleoid.run("$Guest.room = new Room ( )");
      nucleoid.run("guest1 = new Guest ( )");
      nucleoid.run("guest1.room.number = 30");
      equal(nucleoid.run("guest1.room.level"), 3);
    });

    it("creates class assignment with multiple properties after declaration", () => {
      nucleoid.run("class Channel { }");
      nucleoid.run("class Frequency { }");
      nucleoid.run("channel1 = new Channel ( )");
      nucleoid.run("$Channel.frequency = new Frequency ( )");
      nucleoid.run("$Frequency.hertz = 1 / $Frequency.period");
      nucleoid.run("channel1.frequency.period = 0.0025");
      equal(nucleoid.run("channel1.frequency.hertz"), 400);
    });

    it("creates class assignment as multiple properties as part of declaration before initialization", () => {
      nucleoid.run("class Hospital { }");
      nucleoid.run("class Clinic { }");
      nucleoid.run("$Hospital.clinic = new Clinic ( )");
      nucleoid.run("$Hospital.patients = $Hospital.clinic.beds * 746");
      nucleoid.run("hospital1 = new Hospital ( )");
      nucleoid.run("hospital1.clinic.beds = 2678");
      equal(nucleoid.run("hospital1.patients"), 1997788);
    });

    it("creates class assignment as multiple properties as part of declaration after initialization", () => {
      nucleoid.run("class Server { }");
      nucleoid.run("class OS { }");
      nucleoid.run("$Server.os = new OS ( )");
      nucleoid.run("server1 = new Server ( )");
      nucleoid.run("server1.os.version = 14");
      nucleoid.run("$Server.build = $Server.os.version + '.526291'");
      equal(nucleoid.run("server1.build"), "14.526291");
    });

    it("creates class assignment only if instance is defined", () => {
      nucleoid.run("class Phone { }");
      throws(
        () => {
          nucleoid.run("Phone.line.wired = true");
        },
        (error) => validate(error, ReferenceError, "Phone.line is not defined")
      );
    });

    it("creates for of statement", () => {
      nucleoid.run(
        "class Question { constructor ( rate ) { this.rate = rate } }"
      );
      nucleoid.run("question1 = new Question ( 4 )");
      nucleoid.run("question2 = new Question ( 5 )");
      nucleoid.run(
        "class Summary { constructor ( question ) { this.question = question } }"
      );
      nucleoid.run("$Summary.rate = $Summary.question.rate.value");

      nucleoid.run("for ( question of Question ) { new Summary ( question ) }");
      equal(nucleoid.run("Summary[0]").rate, 4);
      equal(nucleoid.run("Summary[1]").rate, 5);
    });

    it("creates block of for statement without dependencies", () => {
      nucleoid.run("class Item { }");
      nucleoid.run("item1 = new Item ( )");
      nucleoid.run("item2 = new Item ( )");
      nucleoid.run("VALUE = 10");
      nucleoid.run(
        "for ( item of Item ) { let i = 10 * VALUE ; item.score = i }"
      );

      nucleoid.run("VALUE = 20");
      equal(nucleoid.run("item1.score"), 100);
      equal(nucleoid.run("item2.score"), 100);

      nucleoid.run(
        "for ( item of Item ) { let i = 10 * VALUE ; item.score = i }"
      );
      equal(nucleoid.run("item1.score"), 200);
      equal(nucleoid.run("item2.score"), 200);
    });

    it("loops through only defined objects in for of statement", () => {
      nucleoid.run("array = [ ]");
      nucleoid.run("class Item { }");

      nucleoid.run("item1 = new Object ( ) ;  array.push ( item1 )");
      nucleoid.run("item2 = { id: 'item3' } ;   array.push ( item2 )");
      nucleoid.run("item4 = new Item ( );   array.push ( item4 )");
      nucleoid.run("count = 0;");

      nucleoid.run("for ( item of array ) { count++ }");
      equal(nucleoid.run("count"), 1);
    });

    it("supports if statement in for of statement", () => {
      nucleoid.run("class Question { }");
      nucleoid.run("question1 = new Question ( )");
      nucleoid.run("question2 = new Question ( )");
      nucleoid.run("question2.archived = true");
      nucleoid.run("question3 = new Question ( )");
      nucleoid.run(
        "class Summary { constructor ( question ) { this.question = question } }"
      );
      nucleoid.run("$Summary.type = 'DAILY'");
      nucleoid.run(
        "for ( question of Question ) { if ( ! question.archived ) { new Summary ( question ) } }"
      );

      equal(nucleoid.run("Summary.length"), 2);
      equal(nucleoid.run("Summary[0].question.id"), "question1");
      equal(nucleoid.run("Summary[1].question.id"), "question3");
      equal(nucleoid.run("Summary[0].type"), "DAILY");
      equal(nucleoid.run("Summary[1].type"), "DAILY");
    });

    it("returns integer in variable assignment", () => {
      nucleoid.run("function test ( a ) { return a = 2 }");
      equal(nucleoid.run("b = 1 ; test ( b )"), 2);
    });

    it("returns reference in variable assignment", () => {
      nucleoid.run("a = new Object ( )");
      nucleoid.run("c = 1");
      nucleoid.run("function test ( b ) { return b = a }");
      deepEqual(nucleoid.run("test ( c )"), {});
      deepEqual(nucleoid.run("c"), 1);
    });

    it("returns string in variable assignment", () => {
      nucleoid.run("function test ( a ) { return a = 'abc' }");
      equal(nucleoid.run("b = 1 ; test ( b )"), "abc");
    });

    it("returns object in variable assignment", () => {
      nucleoid.run("function test ( a ) { return a = new Object ( ) }");
      deepEqual(nucleoid.run("b = 1 ; test ( b )"), {});
    });

    it("runs function with variable", () => {
      nucleoid.run("function test ( a ) { return a + 23 }");
      equal(nucleoid.run("let data = 'UUID-1' ; test ( data )"), "UUID-123");
    });

    it("returns first return statement", () => {
      equal(nucleoid.run("{ return 123 ; return 'abc' }"), 123);
    });

    it("returns undefined in class creation", () => {
      equal(nucleoid.run("class Test { }"), undefined);
    });

    it("returns object itself in object creation", () => {
      nucleoid.run("class Test { constructor ( prop ) { this.prop = prop } }");
      const object = nucleoid.run("new Test ( 123 )");
      notEqual(object.id, null);
      equal(object.prop, 123);
    });

    it("accepts JS function", () => {
      equal(
        nucleoid.run(() => {
          "ABC";
        }),
        null
      );
      equal(
        nucleoid.run(() => {
          return 123;
        }),
        123
      );
      equal(
        nucleoid.run(() => true),
        true
      );
    });

    it("accepts JS function with scope", () => {
      const scope = { test: true };
      equal(
        nucleoid.run((scope) => scope.test, scope),
        true
      );
    });
  });

  describe("in imperative mode", () => {
    const imperative = { declarative: false };

    it("creates variable assignment", () => {
      nucleoid.run("x = 1", imperative);
      nucleoid.run("y = x + 2", imperative);
      nucleoid.run("x = 2", imperative);
      equal(nucleoid.run("y", imperative), 3);
    });

    it("creates if statement of variable", () => {
      nucleoid.run("m = false", imperative);
      nucleoid.run("n = false", imperative);
      nucleoid.run("if ( m == true ) { n = m && true }", imperative);
      equal(nucleoid.run("n", imperative), false);

      nucleoid.run("m = true", imperative);
      equal(nucleoid.run("n", imperative), false);
    });

    it("creates property assignment", () => {
      nucleoid.run("class Order { }", imperative);
      nucleoid.run("var order1 = new Order ( )", imperative);
      nucleoid.run("order1.upc = '04061' + order1.barcode", imperative);
      nucleoid.run("order1.barcode = '94067'", imperative);
      equal(nucleoid.run("order1.upc", imperative), undefined);

      nucleoid.run("order1.upc = '04061' + order1.barcode", imperative);
      equal(nucleoid.run("order1.upc", imperative), "0406194067");
    });

    it("retrieves object through let variable", () => {
      nucleoid.run("class User { }");
      nucleoid.run("user0 = new User ( )");
      deepEqual(nucleoid.run("let user = 'user0' ; User[user]"), {
        id: "user0",
      });
    });
  });
});
