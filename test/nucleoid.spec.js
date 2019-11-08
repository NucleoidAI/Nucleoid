var assert = require("assert");
var nucleoid = require("../nucleoid");

describe("Nucleoid", function() {
  this.slow(1);

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

  it("supports standard built-in objects", function() {
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

  it("uses value property to indicate using value only", function() {
    nucleoid.run("goldenRatio = 1.618");
    nucleoid.run("altitude = 10");

    nucleoid.run("width = goldenRatio.value * altitude");
    assert.equal(nucleoid.run("width"), 16.18);

    nucleoid.run("goldenRatio = 1.62");
    assert.equal(nucleoid.run("width"), 16.18);

    nucleoid.run("altitude = 100");
    assert.equal(nucleoid.run("width"), 161.8);
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

  it("deletes variable assignment", function() {
    nucleoid.run("t = 1");
    nucleoid.run("q = t + 1");
    nucleoid.run("delete q");
    nucleoid.run("t = 2");
    assert.throws(function() {
      nucleoid.run("q");
    }, ReferenceError);
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

  it("runs let statement as an object", function() {
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

  it("reassigns let statement", function() {
    nucleoid.run("class Stock { }");
    nucleoid.run("class Trade { }");
    nucleoid.run("stock1 = new Stock ( )");
    nucleoid.run("stock1.price = 100");
    nucleoid.run("trade1 = new Trade ( )");
    nucleoid.run("trade1.quantity = 1");
    nucleoid.run("stock1.trade = trade1");
    nucleoid.run(
      "{ let trade = Stock.trade ; trade.value = Stock.price * trade.quantity }"
    );
    assert.equal(nucleoid.run("trade1.value"), 100);
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

  it("assigns object to property", function() {
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

  it("deletes property assignment", function() {
    nucleoid.run("class Agent { }");
    nucleoid.run("agent = new Agent ( )");
    nucleoid.run("agent.time = 52926163455");
    nucleoid.run("agent.location = 'CITY'");
    nucleoid.run("agent.report = agent.time + '@' + agent.location");
    assert.equal(nucleoid.run("agent.report"), "52926163455@CITY");

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

  it("creates property assignment with multiple properties", function() {
    nucleoid.run("class Person { }");
    nucleoid.run("person1 = new Person ( )");
    nucleoid.run("class Address { }");
    nucleoid.run("address1 = new Address ( )");
    nucleoid.run("Address.print = Address.city + ',' + Address.state");
    nucleoid.run("person1.address = new Address ( )");
    nucleoid.run("person1.address.city = 'Syracuse'");
    nucleoid.run("person1.address.state = 'NY'");
    assert.equal(nucleoid.run("person1.address.print"), "Syracuse,NY");
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
      "{ let question = vote1.question ; question.rate = ( question.rate * question.count + vote1.rate ) / ( question.count + 1 ) ; question.count++ }"
    );
    assert.equal(nucleoid.run("question1.rate"), 4);
    assert.equal(nucleoid.run("question1.count"), 1);

    nucleoid.run("vote1.rate = 5");
    assert.equal(nucleoid.run("question1.rate"), 4.5);
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

  it("runs block statement of class", function() {
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

  it("runs nested if statement of class", function() {
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
});
