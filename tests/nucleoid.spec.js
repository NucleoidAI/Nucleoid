var assert = require("assert");
var nucleoid = require("../nucleoid");
var state = require("../state");
var graph = require("../graph");

function validate(error, expectedError, expectedMessage) {
  return error instanceof expectedError && error.message === expectedMessage;
}

describe("Nucleoid", function() {
  this.slow(1);

  beforeEach(function() {
    for (let property in state) delete state[property];
    for (let property in graph) delete graph[property];
  });

  it("runs statements in the state", function() {
    nucleoid.run("var i = 1 ;");
    assert.equal(nucleoid.run("i == 1"), true);
  });

  it("runs expression statement", function() {
    nucleoid.run("var j = 1 ;");
    assert.equal(nucleoid.run("j + 2"), 3);
  });

  it("allows variable declaration without var keyword", function() {
    nucleoid.run("pi = 3.14 ;");
    assert.equal(nucleoid.run("pi == 3.14"), true);
  });

  it("allows statements without semicolon at the end", function() {
    nucleoid.run("au = 149597870700");
    assert.equal(nucleoid.run("au == 149597870700"), true);
  });

  it("creates dependency based on length of identifier", function() {
    nucleoid.run("str1 = 'ABC'");
    nucleoid.run("i1 = str1.length + 1");
    assert.equal(nucleoid.run("i1"), 4);

    nucleoid.run("str1 = 'ABCD'");
    assert.equal(nucleoid.run("i1"), 5);
  });

  it("validates syntax of class", function() {
    assert.throws(
      function() {
        nucleoid.run("class Ratio ( }");
      },
      error => validate(error, SyntaxError, "Unexpected token (")
    );

    assert.throws(
      function() {
        nucleoid.run("class Ratio { calculate() }");
      },
      error => validate(error, SyntaxError, "Methods are not supported.")
    );

    assert.throws(
      function() {
        nucleoid.run("class Ratio { calculate() )");
      },
      error => validate(error, SyntaxError, "Unexpected token )")
    );
  });

  it("supports string in expression", function() {
    assert.equal(nucleoid.run("'New String'"), "New String");
  });

  it("supports standard built-in functions", function() {
    nucleoid.run("max = Number.MAX_SAFE_INTEGER");
    nucleoid.run("now = Date.now ( )");
  });

  it("supports creating standard built-in objects", function() {
    nucleoid.run("date = new Date ( '2019-7-24' )");
    assert.equal(nucleoid.run("date.getYear()"), 119);
  });

  it("supports function in expression", function() {
    nucleoid.run("list = [1, 2, 3]");
    assert.equal(
      nucleoid.run("list.find ( ( element ) => { return element == 2 } )"),
      2
    );
  });

  it("retrieves value by variable", function() {
    nucleoid.run("number = -1");
    assert.equal(nucleoid.run("number"), -1);
  });

  it("rejects variable if not declared", function() {
    assert.throws(function() {
      nucleoid.run("e == 2.71828");
    }, ReferenceError);
  });

  it("runs multiple statements in the state", function() {
    nucleoid.run("k = 1 ; l = k + 1 ; k = 2");
    assert.equal(nucleoid.run("l == 3"), true);
  });

  it("runs dependent statements in the same transaction", function() {
    nucleoid.run("class Vehicle { } ; Vehicle.tag = 'US-' + Vehicle.plate ");
    nucleoid.run("vehicle1 = new Vehicle ( )");
    nucleoid.run("vehicle1.plate = 'XSJ422'");
    assert.equal(nucleoid.run("vehicle1.tag"), "US-XSJ422");
  });

  it("runs dependencies in order as received", function() {
    nucleoid.run("any = 0");
    nucleoid.run("if ( any > 1 ) { result = 1 }");
    nucleoid.run("if ( any > 2 ) { result = 2 }");
    nucleoid.run("if ( any > 3 ) { result = 3 }");
    nucleoid.run("if ( any > 2 ) { result = 4 }");
    nucleoid.run("if ( any > 1 ) { result = 5 }");

    nucleoid.run("any = 4");
    assert.equal(nucleoid.run("result"), 5);
  });

  it("searches variable in scope before state", function() {
    nucleoid.run("e = 2.71828");
    nucleoid.run("{ let e = 3 ; number = e }");
    assert.equal(nucleoid.run("number"), 3);
  });

  it("uses local variable at lowest scope as priority", function() {
    nucleoid.run("pi = 3.14");
    nucleoid.run("number = pi");

    nucleoid.run("{ let pi = 3.141 ; { number = null ; { number = pi } } }");
    assert.equal(nucleoid.run("number"), 3.141);

    nucleoid.run(
      "{ let pi = 3.141 ; { let number = pi ; { let pi = 3.1415 ; number = pi } } }"
    );
    assert.equal(nucleoid.run("number"), 3.1415);

    nucleoid.run(
      "{ let pi = 3.141 ; number = pi ; { let pi = 3.1415 ; let number = pi ; { let pi = 3.14159 ; number = pi } } }"
    );
    assert.equal(nucleoid.run("number"), 3.14159);
  });

  it("assigns null if any dependencies in expression is undefined", function() {
    nucleoid.run("class Person { }");
    nucleoid.run("person1 = new Person ( )");
    nucleoid.run("person1.lastName = 'Brown'");
    nucleoid.run(
      "person1.fullName = person1.firstName + ' ' + person1.lastName"
    );
    assert.equal(nucleoid.run("person1.fullName"), null);
  });

  it("keeps as null if any dependencies as in local is null", function() {
    nucleoid.run("a = 1");
    nucleoid.run("{ let b = null ; c = b / a }");
    assert.equal(nucleoid.run("c"), 0);
  });

  it("keeps as null if any dependencies in expression is null", function() {
    nucleoid.run("class Schedule { }");
    nucleoid.run("schedule1 = new Schedule ( )");
    nucleoid.run("schedule1.expression = '0 */2 * * *'");
    nucleoid.run("schedule1.script = null");
    nucleoid.run(
      "schedule1.run = schedule1.expression + '\t' + schedule1.script"
    );
    assert.equal(nucleoid.run("schedule1.run"), "0 */2 * * *\tnull");
  });

  it("assigns null if there is null pointer in expression", function() {
    nucleoid.run("class Product { }");
    nucleoid.run("product1 = new Product ( )");
    nucleoid.run("score = product1.quality.score");
    assert.equal(nucleoid.run("score"), null);
  });

  it("places instance in the list of class when created", function() {
    nucleoid.run("class Student { }");
    assert.equal(nucleoid.run("Array.isArray ( Students )"), true);

    nucleoid.run("student1 = new Student ( )");
    assert.equal(nucleoid.run("Students.length"), 1);
  });

  it("assigns unique variable for instance without variable name defined", function() {
    nucleoid.run("class Vehicle { }");
    nucleoid.run("new Vehicle ( )");
    assert.equal(nucleoid.run("Vehicles.length"), 1);
  });

  it("throws error as a string", function() {
    nucleoid.run("k = 99");
    assert.throws(function() {
      nucleoid.run("if ( k >= 99 ) { throw 'INVALID' }");
    }, "INVALID");
  });

  it("throws error as a variable", function() {
    nucleoid.run("length = 0.1");
    assert.throws(
      function() {
        nucleoid.run("if ( length < 1 ) { throw length }");
      },
      error => error === 0.1
    );

    assert.throws(function() {
      nucleoid.run("if ( length < 1.1 ) { throw 'length' }");
    }, "length");
  });

  it("creates variable assignment", function() {
    nucleoid.run("x = 1");
    nucleoid.run("y = x + 2");
    nucleoid.run("x = 2");
    assert.equal(nucleoid.run("y == 4"), true);
  });

  it("updates variable assignment", function() {
    nucleoid.run("a = 1");
    nucleoid.run("b = 2");
    nucleoid.run("c = a + 3");
    nucleoid.run("c = b + 3");
    assert.equal(nucleoid.run("c"), 5);

    nucleoid.run("b = 4");
    assert.equal(nucleoid.run("c"), 7);
  });

  it("uses its value when self variable used", function() {
    nucleoid.run("radius = 10");
    nucleoid.run("radius = radius + 10");
    assert.equal(nucleoid.run("radius"), 20);
  });

  it("deletes variable assignment", function() {
    nucleoid.run("t = 1");
    nucleoid.run("q = t + 1");
    nucleoid.run("delete q");
    nucleoid.run("t = 2");
    assert.throws(function() {
      nucleoid.run("q");
    }, ReferenceError);
  });

  it("uses value property to indicate using only value of variable", function() {
    nucleoid.run("goldenRatio = 1.618");
    nucleoid.run("altitude = 10");

    nucleoid.run("width = goldenRatio.value * altitude");
    assert.equal(nucleoid.run("width"), 16.18);

    nucleoid.run("goldenRatio = 1.62");
    assert.equal(nucleoid.run("width"), 16.18);

    nucleoid.run("altitude = 100");
    assert.equal(nucleoid.run("width"), 161.8);
  });

  it("creates if statement of variable", function() {
    nucleoid.run("m = false");
    nucleoid.run("n = false");
    nucleoid.run("if ( m == true ) { n = m && true }");
    assert.equal(nucleoid.run("n == false"), true);

    nucleoid.run("m = true");
    assert.equal(nucleoid.run("n == true"), true);
  });

  it("updates if block of variable", function() {
    nucleoid.run("p = 0.01");
    nucleoid.run("s = 0.02");
    nucleoid.run("if ( p < 1 ) { r = p * 10 }");
    nucleoid.run("if ( p < 1 ) { r = s * 10 }");
    assert.equal(nucleoid.run("r"), 0.2);

    nucleoid.run("s = 0.03");
    assert.equal(nucleoid.run("r"), 0.3);
  });

  it("creates else statement of variable", function() {
    nucleoid.run("compound = 0.0001");
    nucleoid.run("acidic = 'ACIDIC'");
    nucleoid.run("basic = 'BASIC'");
    nucleoid.run(
      "if ( compound > 0.0000001 ) { pH = acidic } else { pH = basic }"
    );
    nucleoid.run("compound = 0.000000001");
    assert.equal(nucleoid.run("pH"), "BASIC");

    nucleoid.run("basic = '+7'");
    assert.equal(nucleoid.run("pH"), "+7");
  });

  it("creates else if statement of variable", function() {
    nucleoid.run("g = 11");
    nucleoid.run("earth = 9.8");
    nucleoid.run("mars = 3.71");
    nucleoid.run("mass = 10");
    nucleoid.run(
      "if ( g > 9 ) { weight = earth * mass } else if ( g > 3 ) { weight = mars * mass }"
    );
    nucleoid.run("g = 5");
    assert.equal(nucleoid.run("weight"), 37.1);

    nucleoid.run("mars = 3.72");
    assert.equal(nucleoid.run("weight"), 37.2);
  });

  it("runs let statement as a variable", function() {
    nucleoid.run("integer = 30");
    nucleoid.run(
      "{ let division = integer / 10 ; equivalency = division * 10}"
    );
    assert.equal(nucleoid.run("equivalency"), 30);

    nucleoid.run("integer = 40");
    assert.equal(nucleoid.run("equivalency"), 40);
  });

  it("runs let statement of class as new instance before initialization", function() {
    nucleoid.run("class Member { }");
    nucleoid.run("class Registration { }");
    nucleoid.run(
      "{ let registration = new Registration ( ) ; registration.date = new Date ( '2019-1-2' ) ; Member.registration = registration }"
    );

    nucleoid.run("member1 = new Member ( )");
    assert.equal(
      nucleoid.run("member1.registration.date.toDateString()"),
      "Wed Jan 02 2019"
    );
  });

  it("runs let statement of class as new instance after initialization", function() {
    nucleoid.run("class Distance { }");
    nucleoid.run("class Location { }");
    nucleoid.run("distance1 = new Distance ( )");
    nucleoid.run(
      "{ let location = new Location ( ) ; location.coordinates = '40.6976701,-74.2598779' ; Distance.startingPoint = location }"
    );
    assert.equal(
      nucleoid.run("distance1.startingPoint.coordinates"),
      "40.6976701,-74.2598779"
    );
  });

  it("runs let statement as an object before declaration", function() {
    nucleoid.run("class Plane { }");
    nucleoid.run("class Trip { }");
    nucleoid.run("plane1 = new Plane ( )");
    nucleoid.run("plane1.speed = 903");
    nucleoid.run("trip1 = new Trip ( )");
    nucleoid.run("trip1.distance = 5540");
    nucleoid.run(
      "{ let trip = Plane.trip ; Plane.time = trip.distance / Plane.speed }"
    );
    nucleoid.run("plane1.trip = trip1");
    assert.equal(nucleoid.run("plane1.time"), 6.135105204872647);
  });

  it("runs let statement as an object after declaration", function() {
    nucleoid.run("class Seller { }");
    nucleoid.run("class Commission { }");
    nucleoid.run("seller1 = new Seller ( )");
    nucleoid.run("seller1.sales = 1000000");
    nucleoid.run("comm1 = new Commission ( )");
    nucleoid.run("comm1.rate = 0.05");
    nucleoid.run("seller1.commission = comm1");
    nucleoid.run(
      "{ let commission = Seller.commission ; Seller.pay = Seller.sales * commission.rate }"
    );
    assert.equal(nucleoid.run("seller1.pay"), 50000);
  });

  it("reassigns let statement before initialization", function() {
    nucleoid.run("class Order { }");
    nucleoid.run("class Sale { }");
    nucleoid.run(
      "{ let sale = Order.sale ; sale.amount = sale.percentage / Order.amount * 100 }"
    );
    nucleoid.run("order1 = new Order ( )");
    nucleoid.run("order1.amount = 100");
    nucleoid.run("sale1 = new Sale ( )");
    nucleoid.run("sale1.percentage = 10");
    nucleoid.run("order1.sale = sale1");
    assert.equal(nucleoid.run("sale1.amount"), 10);
  });

  it("reassigns let statement after initialization", function() {
    nucleoid.run("class Stock { }");
    nucleoid.run("class Trade { }");
    nucleoid.run("stock1 = new Stock ( )");
    nucleoid.run("stock1.price = 100");
    nucleoid.run("trade1 = new Trade ( )");
    nucleoid.run("trade1.quantity = 1");
    nucleoid.run("stock1.trade = trade1");
    nucleoid.run(
      "{ let trade = Stock.trade ; trade.worth = Stock.price * trade.quantity }"
    );
    assert.equal(nucleoid.run("trade1.worth"), 100);
  });

  it("holds result of function in let", function() {
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
    assert.equal(nucleoid.run("bug1.selected"), true);
    assert.equal(nucleoid.run("bug2.selected"), undefined);

    nucleoid.run("ticket = 2");
    assert.equal(nucleoid.run("bug2.selected"), true);
  });

  it("skips if block is empty", function() {
    nucleoid.run("{ }");
  });

  it("runs block statement of variable", function() {
    nucleoid.run("h = 1");
    nucleoid.run("{ let value = h * 2 ; j = value * 2 }");
    assert.equal(nucleoid.run("j"), 4);

    nucleoid.run("h = 2");
    assert.equal(nucleoid.run("j"), 8);
  });

  it("runs nested block statement of variable", function() {
    nucleoid.run("radius = 10");
    nucleoid.run(
      "{ let area = Math.pow ( radius , 2 ) * 3.14 ; { volume = area * 5 } }"
    );
    assert.equal(nucleoid.run("volume"), 1570);
  });

  it("runs nested if statement of variable", function() {
    nucleoid.run("gravity = 9.8");
    nucleoid.run("time = 10");
    nucleoid.run("distance = 480");
    nucleoid.run("target = true");
    nucleoid.run(
      "{ let dist = 1 / 2 * gravity * time * time ; if ( dist > distance ) { hit = target } }"
    );
    assert.equal(nucleoid.run("hit"), true);

    nucleoid.run("target = false");
    assert.equal(nucleoid.run("hit"), false);
  });

  it("runs nested else statement of variable", function() {
    nucleoid.run("percentage = 28");
    nucleoid.run("density = 0.899");
    nucleoid.run("substance = 'NH3'");
    nucleoid.run("molarConcentration = null");
    nucleoid.run("default = 0");
    nucleoid.run(
      "{ let concentration = percentage * density / 100 * 1000 ; if ( substance == 'NH3' ) { molarConcentration = concentration / 17.04 } else { molarConcentration = default } }"
    );
    nucleoid.run("substance = 'NH16'");
    nucleoid.run("default = 1");
    assert.equal(nucleoid.run("molarConcentration"), 1);
  });

  it("assigns object to variable", function() {
    nucleoid.run("class Model { }");
    nucleoid.run("model1 = new Model ( )");
    assert.equal(nucleoid.run("typeof model1"), "object");
  });

  it("defines class in the state", function() {
    nucleoid.run("class Entity { }");
    assert.equal(nucleoid.run("typeof Entity"), "function");
  });

  it("creates property assignment before declaration", function() {
    nucleoid.run("class Order { }");
    nucleoid.run("var order1 = new Order ( )");
    nucleoid.run("order1.upc = '04061' + order1.barcode");
    nucleoid.run("order1.barcode = '94067'");
    assert.equal(nucleoid.run("order1.upc"), "0406194067");
  });

  it("creates property assignment after declaration", function() {
    nucleoid.run("class User { }");
    nucleoid.run("user = new User ( )");
    nucleoid.run("user.name = 'sample'");
    nucleoid.run("user.email = user.name + '@example.com'");
    assert.equal(nucleoid.run("user.email"), "sample@example.com");

    nucleoid.run("user.name = 'samplex'");
    assert.equal(nucleoid.run("user.email"), "samplex@example.com");
  });

  it("creates property assignment only if instance is defined", function() {
    nucleoid.run("class Channel { }");
    nucleoid.run("channel1 = new Channel ( )");
    assert.throws(function() {
      nucleoid.run("channel1.frequency.type = 'ANGULAR'");
    }, TypeError);
  });

  it("uses its value when self property used", function() {
    nucleoid.run("class Construction { }");
    nucleoid.run("construction1 = new Construction ( ) ");
    nucleoid.run("construction1.timeline = 120");
    nucleoid.run("construction1.timeline = 2 * construction1.timeline");
    assert.equal(nucleoid.run("construction1.timeline"), 240);
  });

  it("assigns object to property before initialization", function() {
    nucleoid.run("class Agent { }");
    nucleoid.run("class Distance { }");
    nucleoid.run(
      "Distance.total = Math.sqrt ( Distance.x * Distance.x + Distance.y * Distance.y )"
    );
    nucleoid.run("agent1 = new Agent ( )");
    nucleoid.run("agent1.distance = new Distance ( )");
    nucleoid.run("agent1.distance.x = 3");
    nucleoid.run("agent1.distance.y = 4");
    assert.equal(nucleoid.run("agent1.distance.total"), 5);
  });

  it("assigns object to property after initialization", function() {
    nucleoid.run("class Product { }");
    nucleoid.run("product1 = new Product ( )");
    nucleoid.run("class Quality { }");
    nucleoid.run("product1.quality = new Quality ( )");
    nucleoid.run("product1.quality.score = 15");
    nucleoid.run(
      "Quality.class = String.fromCharCode ( 65 + Math.floor ( Quality.score / 10 ) )"
    );
    assert.equal(nucleoid.run("product1.quality.class"), "B");
  });

  it("rejects if property name is value", function() {
    nucleoid.run("class Number { }");
    nucleoid.run("value = new Number ( )");
    assert.throws(function() {
      nucleoid.run("value.value = 2147483647");
    }, TypeError);
  });

  it("uses value property to indicate using only value of property", function() {
    nucleoid.run("class Weight { }");
    nucleoid.run("weight1 = new Weight ( )");
    nucleoid.run("weight1.gravity = 1.352");
    nucleoid.run("weight1.mass = 1000");
    nucleoid.run("weight1.force = weight1.gravity * weight1.mass.value");
    assert.equal(nucleoid.run("weight1.force"), 1352);

    nucleoid.run("weight1.mass = 2000");
    assert.equal(nucleoid.run("weight1.force"), 1352);
  });

  it("rejects value of property if property is not defined", function() {
    nucleoid.run("class Travel { }");
    nucleoid.run("travel1 = new Travel ( )");
    nucleoid.run("travel1.speed = 65");
    assert.throws(function() {
      nucleoid.run("travel1.time = travel1.distance.value / travel1.speed");
    }, TypeError);
  });

  it("keeps as null if value of property is null", function() {
    nucleoid.run("class Interest { }");
    nucleoid.run("interest1 = new Interest ( )");
    nucleoid.run("interest1.rate = 3");
    nucleoid.run("interest1.amount = null");
    nucleoid.run(
      "interest1.annual = interest1.rate * interest1.amount.value / 100"
    );
    assert.equal(nucleoid.run("interest1.annual"), 0);

    nucleoid.run("interest1.amount = 10000");
    assert.equal(nucleoid.run("interest1.annual"), 0);
  });

  it("rejects if property of local name is value", function() {
    nucleoid.run("class Alarm { }");
    assert.throws(function() {
      nucleoid.run("{ let value = new Alarm ( ) ; value.value = '22:00' }");
    }, TypeError);
  });

  it("keeps same as its value when value property used for local", function() {
    nucleoid.run("speedOfLight = 299792");
    nucleoid.run(
      "{ let time = speedOfLight / 225623 ; roundTrip = time.value * 2 }"
    );
    assert.equal(nucleoid.run("roundTrip"), 2.6574595675086314);
  });

  it("updates if block of property", function() {
    nucleoid.run("class Account { }");
    nucleoid.run("account = new Account ( )");
    nucleoid.run("account.balance = 1000");
    nucleoid.run("if ( account.balance < 1500 ) { account.status = 'OK' }");
    assert.equal(nucleoid.run("account.status"), "OK");

    nucleoid.run("if ( account.balance < 1500 ) { account.status = 'LOW' }");
    assert.equal(nucleoid.run("account.status"), "LOW");
  });

  it("creates if statement of property", function() {
    nucleoid.run("class Toy { }");
    nucleoid.run("toy = new Toy ( )");
    nucleoid.run("toy.color = 'BLUE'");
    nucleoid.run("if ( toy.color == 'RED' ) { toy.shape = 'CIRCLE' }");
    nucleoid.run("toy.color = 'RED'");
    assert.equal(nucleoid.run("toy.shape"), "CIRCLE");
  });

  it("creates else statement of property", function() {
    nucleoid.run("class Engine { }");
    nucleoid.run("engine1 = new Engine ( )");
    nucleoid.run("engine1.type = 'V8'");
    nucleoid.run("mpl = 'MPL'");
    nucleoid.run("bsd = 'BSD'");
    nucleoid.run(
      "if ( engine1.type == 'Gecko' ) { engine1.license = mpl } else { engine1.license = bsd }"
    );
    assert.equal(nucleoid.run("engine1.license"), "BSD");

    nucleoid.run("bsd = 'Berkeley Software Distribution'");
    assert.equal(
      nucleoid.run("engine1.license"),
      "Berkeley Software Distribution"
    );
  });

  it("updates property assignment", function() {
    nucleoid.run("class Matter { }");
    nucleoid.run("matter1 = new Matter ( )");
    nucleoid.run("matter1.mass = 10");
    nucleoid.run("matter1.weight = matter1.mass * 9.8");
    assert.equal(nucleoid.run("matter1.weight"), 98);

    nucleoid.run("matter1.weight = matter1.mass * 3.7");
    assert.equal(nucleoid.run("matter1.weight"), 37);

    nucleoid.run("matter1.mass = 20");
    assert.equal(nucleoid.run("matter1.weight"), 74);
  });

  it("deletes instance", function() {
    nucleoid.run("class Circle { }");
    nucleoid.run("circle1 = new Circle ( )");
    nucleoid.run("delete circle1");
    assert.throws(
      function() {
        nucleoid.run("circle1");
      },
      error => validate(error, ReferenceError, "circle1 is not defined")
    );
  });

  it("rejects deleting instance if it has any properties", function() {
    nucleoid.run("class Channel { }");
    nucleoid.run("channel1 = new Channel ( )");
    nucleoid.run("channel1.frequency = 440");
    assert.throws(
      function() {
        nucleoid.run("delete channel1");
      },
      error =>
        validate(error, ReferenceError, "Cannot delete object 'channel1'")
    );
    assert.equal(nucleoid.run("channel1.frequency "), 440);

    nucleoid.run("delete channel1.frequency");
    nucleoid.run("delete channel1");
  });

  it("rejects deleting instance if it has object as a property", function() {
    nucleoid.run("class Shape { }");
    nucleoid.run("class Type { }");
    nucleoid.run("shape1 = new Shape ( )");
    nucleoid.run("shape1.type = new Type ( )");
    assert.throws(
      function() {
        nucleoid.run("delete shape1");
      },
      error => validate(error, ReferenceError, "Cannot delete object 'shape1'")
    );

    nucleoid.run("delete shape1.type");
    nucleoid.run("delete shape1");
  });

  it("deletes property assignment", function() {
    nucleoid.run("class Agent { }");
    nucleoid.run("agent = new Agent ( )");
    nucleoid.run("agent.time = 52926163455");
    nucleoid.run("agent.location = 'CITY'");
    nucleoid.run("agent.report = agent.time + '@' + agent.location");
    assert.equal(nucleoid.run("agent.report"), "52926163455@CITY");

    nucleoid.run("delete agent.time");
    assert.equal(nucleoid.run("agent.report"), undefined);

    nucleoid.run("delete agent.report");
    assert.equal(nucleoid.run("agent.report"), undefined);
  });

  it("runs block statement of property", function() {
    nucleoid.run("class Item { }");
    nucleoid.run("item1 = new Item ( )");
    nucleoid.run("item1.sku = '0000001' ");
    nucleoid.run("{ let custom = 'US' + item1.sku ; item1.custom = custom }");
    assert.equal(nucleoid.run("item1.custom"), "US0000001");

    nucleoid.run("item1.sku = '0000002' ");
    assert.equal(nucleoid.run("item1.custom"), "US0000002");
  });

  it("runs nested if statement of property", function() {
    nucleoid.run("class Sale { }");
    nucleoid.run("sale1 = new Sale ( )");
    nucleoid.run("sale1.price = 50");
    nucleoid.run("sale1.quantity = 2");
    nucleoid.run(
      "{ let amount = sale1.price * sale1.quantity ; if ( amount > 100 ) { sale1.tax = amount * 10 / 100 } }"
    );
    assert.equal(nucleoid.run("sale1.tax"), undefined);

    nucleoid.run("sale1.quantity = 3");
    assert.equal(nucleoid.run("sale1.tax"), 15);
  });

  it("creates nested else statement of property", function() {
    nucleoid.run("class Chart { }");
    nucleoid.run("chart1 = new Chart ( )");
    nucleoid.run("invalid = 'INVALID'");
    nucleoid.run("valid = 'VALID'");
    nucleoid.run(
      "{ let ratio = chart1.percentage / 100 ; if ( ratio > 1 ) { chart1.status = invalid } else { chart1.status = valid } }"
    );
    assert.equal(nucleoid.run("chart1.status"), "VALID");

    nucleoid.run("valid = 'V'");
    assert.equal(nucleoid.run("chart1.status"), "V");
  });

  it("creates property assignment with multiple properties", function() {
    nucleoid.run("class Person { }");
    nucleoid.run("person1 = new Person ( )");
    nucleoid.run("class Address { }");
    nucleoid.run("address1 = new Address ( )");
    nucleoid.run("Address.print = Address.city + ', ' + Address.state");
    nucleoid.run("person1.address = new Address ( )");
    nucleoid.run("person1.address.city = 'Syracuse'");
    nucleoid.run("person1.address.state = 'NY'");
    assert.equal(nucleoid.run("person1.address.print"), "Syracuse, NY");
  });

  it("creates property assignment as multiple properties as part of declaration", function() {
    nucleoid.run("class Server { }");
    nucleoid.run("server1 = new Server ( )");
    nucleoid.run("server1.name = 'HOST1'");
    nucleoid.run("class IP { }");
    nucleoid.run("ip1 = new IP ( )");
    nucleoid.run("server1.ip = ip1");
    nucleoid.run("ip1.address = '10.0.0.1'");
    nucleoid.run("server1.summary = server1.name + '@' + server1.ip.address");
    assert.equal(nucleoid.run("server1.summary"), "HOST1@10.0.0.1");

    nucleoid.run("ip1.address = '10.0.0.2'");
    assert.equal(nucleoid.run("server1.summary"), "HOST1@10.0.0.2");
  });

  it("creates dependency behalf if property has reference", function() {
    nucleoid.run("class Schedule { }");
    nucleoid.run("schedule1 = new Schedule ( )");
    nucleoid.run("class Template { }");
    nucleoid.run("template1 = new Template ( )");
    nucleoid.run("template1.type = 'W'");
    nucleoid.run("schedule1.template = template1");
    nucleoid.run("schedule1.template.name = schedule1.template.type + '-0001'");
    assert.equal(nucleoid.run("template1.name"), "W-0001");

    nucleoid.run("template1.type = 'D'");
    assert.equal(nucleoid.run("template1.name"), "D-0001");
  });

  it("creates dependency behalf if let has reference", function() {
    nucleoid.run("class Vote { }");
    nucleoid.run("vote1 = new Vote ( )");
    nucleoid.run("vote1.rate = 4");
    nucleoid.run("class Question { }");
    nucleoid.run("Question.rate = 0");
    nucleoid.run("Question.count = 0");
    nucleoid.run("question1 = new Question ( )");
    nucleoid.run("vote1.question = question1");
    nucleoid.run(
      "{ let question = vote1.question ; question.rate = ( question.rate * question.count + vote1.rate ) / ( question.count + 1 ) ; question.count =  question.count + 1}"
    );
    assert.equal(nucleoid.run("question1.rate"), 4);
    assert.equal(nucleoid.run("question1.count"), 1);

    nucleoid.run("vote1.rate = 5");
    assert.equal(nucleoid.run("question1.rate"), 4.5);
  });

  it("runs expression statement of class", function() {
    nucleoid.run("class Element { }");
    nucleoid.run("alkalis = [ ]");
    nucleoid.run("element1 = new Element ( )");
    nucleoid.run("element1.number = 3");
    nucleoid.run(
      "{ let number = Element.number ; if ( number == 3 ) { alkalis.push ( Element ) } }"
    );
    assert.equal(nucleoid.run("alkalis.pop (  )"), nucleoid.run("element1"));
  });

  it("creates class assignment before initialization", function() {
    nucleoid.run("class Review { }");
    nucleoid.run("Review.rate = Review.sum / 10");
    nucleoid.run("review1 = new Review ( )");
    nucleoid.run("review1.sum = 42");
    assert.equal(nucleoid.run("review1.rate"), 4.2);
  });

  it("creates class assignment after initialization", function() {
    nucleoid.run("class Shape { }");
    nucleoid.run("s1 = new Shape ( )");
    nucleoid.run("s1.edge = 3");
    nucleoid.run("s2 = new Shape ( )");
    nucleoid.run("s2.edge = 3");
    nucleoid.run("Shape.angle = ( Shape.edge - 2 ) * 180");
    nucleoid.run("s1.edge = 4");
    assert.equal(nucleoid.run("s1.angle"), 360);
    assert.equal(nucleoid.run("s2.angle"), 180);
  });

  it("updates class assignment", function() {
    nucleoid.run("class Employee { }");
    nucleoid.run("employee = new Employee ( )");
    nucleoid.run("employee.id = 1");
    nucleoid.run("Employee.username = 'E' + Employee.id");
    assert.equal(nucleoid.run("employee.username"), "E1");
    nucleoid.run("Employee.username = 'F' + Employee.id");
    nucleoid.run("employee.id = 2");
    assert.equal(nucleoid.run("employee.username"), "F2");
  });

  it("creates if statement of class before initialization", function() {
    nucleoid.run("class Ticket { }");
    nucleoid.run(
      "if ( Ticket.date > new Date ( '1993-1-1' ) ) { Ticket.status = 'EXPIRED' }"
    );
    nucleoid.run("ticket1 = new Ticket ( )");
    assert.equal(nucleoid.run("ticket1.status"), undefined);
    nucleoid.run("ticket1.date = new Date ( '1993-2-1' ) ");
    assert.equal(nucleoid.run("ticket1.status"), "EXPIRED");
  });

  it("creates if statement of class after initialization", function() {
    nucleoid.run("class Student { }");
    nucleoid.run("s1 = new Student ( )");
    nucleoid.run("s1.age = 2");
    nucleoid.run("s1.class = 'Daycare'");
    nucleoid.run("s2 = new Student ( )");
    nucleoid.run("s2.age = 2");
    nucleoid.run("s2.class = 'Daycare'");
    nucleoid.run("if ( Student.age == 3 ) { Student.class = 'Preschool' }");
    nucleoid.run("s1.age = 3");
    assert.equal(nucleoid.run("s1.class"), "Preschool");
    assert.equal(nucleoid.run("s2.class"), "Daycare");
  });

  it("updates if block of class", function() {
    nucleoid.run("class Inventory { }");
    nucleoid.run("i1 = new Inventory ( )");
    nucleoid.run("i1.quantity = 0");

    nucleoid.run("i2 = new Inventory ( )");
    nucleoid.run("i2.quantity = 1000");

    nucleoid.run(
      "if ( Inventory.quantity == 0 ) { Inventory.replenishment = true }"
    );
    assert.equal(nucleoid.run("i1.replenishment"), true);
    assert.equal(nucleoid.run("i2.replenishment"), undefined);
    nucleoid.run(
      "if ( Inventory.quantity == 0 ) { Inventory.replenishment = false }"
    );

    assert.equal(nucleoid.run("i1.replenishment"), false);
    assert.equal(nucleoid.run("i2.replenishment"), undefined);
  });

  it("creates else statement of class before initialization", function() {
    nucleoid.run("class Count { }");
    nucleoid.run(
      "if ( Count.max > 1000 ) { Count.reset = urgent } else { Count.reset = regular }"
    );
    nucleoid.run("urgent = 'URGENT'");
    nucleoid.run("regular = 'REGULAR'");
    nucleoid.run("count1 = new Count ( )");
    nucleoid.run("count1.max = 850");
    assert.equal(nucleoid.run("count1.reset"), "REGULAR");

    nucleoid.run("regular = 'R'");
    assert.equal(nucleoid.run("count1.reset"), "R");
  });

  it("creates else statement of class after initialization", function() {
    nucleoid.run("class Concentration { }");
    nucleoid.run("serialDilution = '(c1V1+c2V2)/(V1+V2)'");
    nucleoid.run("directDilution = 'c1/V1'");
    nucleoid.run("concentration1 = new Concentration ( )");
    nucleoid.run("concentration1.substances = 2");
    nucleoid.run(
      "if ( Concentration.substances == 1 ) { Concentration.formula = directDilution } else { Concentration.formula = serialDilution }"
    );
    assert.equal(nucleoid.run("concentration1.formula"), "(c1V1+c2V2)/(V1+V2)");

    nucleoid.run("serialDilution = '(c1V1+c2V2+c3V3)/(V1+V2+V3)'");
    assert.equal(
      nucleoid.run("concentration1.formula"),
      "(c1V1+c2V2+c3V3)/(V1+V2+V3)"
    );
  });

  it("runs block statement of class before initialization", function() {
    nucleoid.run("class Stock { }"); //Stock
    nucleoid.run(
      "{ let change = Stock.before * 4 / 100 ; Stock.after = Stock.before + change }"
    );
    nucleoid.run("stock1 = new Stock ( )");
    nucleoid.run("stock1.before = 57.25");
    assert.equal(nucleoid.run("stock1.after"), 59.54);

    nucleoid.run("stock1.before = 59.50");
    assert.equal(nucleoid.run("stock1.after"), 61.88);
  });

  it("runs block statement of class after initialization", function() {
    nucleoid.run("class Purchase { }");
    nucleoid.run("purchase = new Purchase ( )");
    nucleoid.run("purchase.price = 99");
    nucleoid.run(
      "{ let retailPrice = Purchase.price * 1.15 ; Purchase.retailPrice = retailPrice }"
    );
    assert.equal(nucleoid.run("purchase.retailPrice"), 113.85);

    nucleoid.run("purchase.price = 199");
    assert.equal(nucleoid.run("purchase.retailPrice"), 228.85);
  });

  it("runs nested block statement of class before initialization", function() {
    nucleoid.run("class Compound { }");
    nucleoid.run(
      "{ let mol = 69.94 / Compound.substance ; { Compound.sample = Math.floor ( mol * Compound.mol ) } }"
    );
    nucleoid.run("compound1 = new Compound ( )");
    nucleoid.run("compound1.substance = 55.85");
    nucleoid.run("compound1.mol = 1000");
    assert.equal(nucleoid.run("compound1.sample"), 1252);
  });

  it("runs nested block statement of class after initialization", function() {
    nucleoid.run("class Bug { }");
    nucleoid.run("bug1 = new Bug ( )");
    nucleoid.run("bug1.initialScore = 1000");
    nucleoid.run("bug1.aging = 24");
    nucleoid.run(
      "{ let score = Bug.aging * 10 ; { Bug.priorityScore = score + Bug.initialScore } }"
    );
    assert.equal(nucleoid.run("bug1.priorityScore"), 1240);
  });

  it("runs nested if statement of class before initialization", function() {
    nucleoid.run("class Mortgage { }");
    nucleoid.run(
      "{ let interest = Mortgage.annual / 12 ; if ( interest < 4 ) { Mortgage.rate = rate1 } }"
    );
    nucleoid.run("rate1 = 'EXCEPTIONAL'");
    nucleoid.run("mortgage1 = new Mortgage ( )");
    nucleoid.run("mortgage1.annual = 46");
    assert.equal(nucleoid.run("mortgage1.rate"), "EXCEPTIONAL");

    nucleoid.run("rate1 = 'E'");
    assert.equal(nucleoid.run("mortgage1.rate"), "E");
  });

  it("runs nested if statement of class after initialization", function() {
    nucleoid.run("class Building { }");
    nucleoid.run("buildingType1 = 'SKYSCRAPER'");
    nucleoid.run("building1 = new Building ( )");
    nucleoid.run("building1.floors = 20");
    nucleoid.run(
      "{ let height = Building.floors * 14 ; if ( height > 330 ) { Building.type = buildingType1 } }"
    );
    assert.equal(nucleoid.run("building1.type"), undefined);

    nucleoid.run("building1.floors = 25");
    assert.equal(nucleoid.run("building1.type"), "SKYSCRAPER");

    nucleoid.run("buildingType1 = 'S'");
    assert.equal(nucleoid.run("building1.type"), "S");
  });

  it("creates nested else statement of class before initialization", function() {
    nucleoid.run("class Account { }");
    nucleoid.run("noAlert = 'NO_ALERT'");
    nucleoid.run("lowAlert = 'LOW_ALERT'");
    nucleoid.run(
      "{ let balance = Account.balance ; if ( balance > 1000 ) { Account.alert = noAlert } else { Account.alert = lowAlert } }"
    );
    nucleoid.run("account1 = new Account ( )");
    nucleoid.run("account1.balance = 950");
    assert.equal(nucleoid.run("account1.alert"), "LOW_ALERT");

    nucleoid.run("lowAlert = 'L'");
    assert.equal(nucleoid.run("account1.alert"), "L");
  });

  it("creates nested else statement of class after initialization", function() {
    nucleoid.run("class Question { }");
    nucleoid.run("high = 'HIGH'");
    nucleoid.run("low = 'LOW'");
    nucleoid.run("question1 = new Question ( )");
    nucleoid.run("question1.count = 1");
    nucleoid.run(
      "{ let score = Question.count * 10 ; if ( score > 100 ) { Question.type = high } else { Question.type = low } }"
    );
    assert.equal(nucleoid.run("question1.type"), "LOW");

    nucleoid.run("low = 'L'");
    assert.equal(nucleoid.run("question1.type"), "L");
  });

  it("creates class assignment with multiple properties before declaration", function() {
    nucleoid.run("class Room { }");
    nucleoid.run("Room.level = Room.number / 10");
    nucleoid.run("class Guest { }");
    nucleoid.run("Guest.room = new Room ( )");
    nucleoid.run("guest1 = new Guest ( )");
    nucleoid.run("guest1.room.number = 30");
    assert.equal(nucleoid.run("guest1.room.level"), 3);
  });

  it("creates class assignment with multiple properties after declaration", function() {
    nucleoid.run("class Channel { }");
    nucleoid.run("class Frequency { }");
    nucleoid.run("channel1 = new Channel ( )");
    nucleoid.run("Channel.frequency = new Frequency ( )");
    nucleoid.run("Frequency.hertz = 1 / Frequency.period");
    nucleoid.run("channel1.frequency.period = 0.0025");
    assert.equal(nucleoid.run("channel1.frequency.hertz"), 400);
  });

  it("creates class assignment as multiple properties as part of declaration before initialization", function() {
    nucleoid.run("class Hospital { }");
    nucleoid.run("class Clinic { }");
    nucleoid.run("Hospital.clinic = new Clinic ( )");
    nucleoid.run("Hospital.patients = Hospital.clinic.beds * 746");
    nucleoid.run("hospital1 = new Hospital ( )");
    nucleoid.run("hospital1.clinic.beds = 2678");
    assert.equal(nucleoid.run("hospital1.patients"), 1997788);
  });

  it("creates class assignment as multiple properties as part of declaration after initialization", function() {
    nucleoid.run("class Server { }");
    nucleoid.run("class OS { }");
    nucleoid.run("Server.os = new OS ( )");
    nucleoid.run("server1 = new Server ( )");
    nucleoid.run("server1.os.version = 14");
    nucleoid.run("Server.build = Server.os.version + '.526291'");
    assert.equal(nucleoid.run("server1.build"), "14.526291");
  });

  it("creates class assignment only if instance is defined", function() {
    nucleoid.run("class Phone { }");
    assert.throws(function() {
      nucleoid.run("Phone.line.wired = true");
    }, TypeError);
  });

  it("rejects using value of class", function() {
    nucleoid.run("class Employee { }");
    assert.throws(function() {
      nucleoid.run("Employee.annual = Employee.biweekly.value * 52");
    }, TypeError);
  });
});
