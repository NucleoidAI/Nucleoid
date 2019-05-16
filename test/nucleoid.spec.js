var assert = require("assert");
var nucleoid = require("../nucleoid");

describe("Nucleoid", function() {
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

  it("runs block statement of variable", function() {
    nucleoid.run("h = 1");
    nucleoid.run("j = undefined");
    nucleoid.run("{ let value = h * 2 ; j = value * 2 }");
    assert.equal(nucleoid.run("j"), 4);

    nucleoid.run("h = 2");
    assert.equal(nucleoid.run("j"), 8);
  });

  it("defines class in the state", function() {
    nucleoid.run("class Entity { }");
    nucleoid.run("e = new Entity ( )");
    assert.equal(nucleoid.run("e.constructor == Entity"), true);
  });

  it("creates property assignment", function() {
    nucleoid.run("class User { }");
    nucleoid.run("user = new User ( )");
    nucleoid.run("user.name = 'sample'");
    nucleoid.run("user.email = user.name + '@example.com'");
    assert.equal(nucleoid.run("user.email"), "sample@example.com");

    nucleoid.run("user.name = 'samplex'");
    assert.equal(nucleoid.run("user.email"), "samplex@example.com");
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

  it("creates class assignment", function() {
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

  it("creates if statement of class", function() {
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
});
