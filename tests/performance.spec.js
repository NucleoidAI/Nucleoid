const argv = require("yargs").argv;
const nucleoid = require("../nucleoid");

var n = argv.n ? argv.n : 2;
var c = argv.c ? argv.c : 0;

nucleoid.run("class Order { }");
nucleoid.run("class Shipment { }");

if (c > 1) nucleoid.run("Order.upc = '04061' + Order.barcode");
if (c > 2) nucleoid.run("Order.test = '04061' + Order.upc");
if (c > 3) nucleoid.run("Order.test2 = '04061' + Order.test");
if (c > 4) nucleoid.run("if ( Order.barcode ) { Order.test = Order.upc } ");

nucleoid.run("order = new Order ( )");
nucleoid.run("shipment = new Shipment ( )");

if (c > 1) nucleoid.run("shipment.test1 = shipment.test");
if (c > 2) nucleoid.run("shipment.test2 = shipment.test1");
if (c > 3) nucleoid.run("shipment.test3 = shipment.tes2");
if (c > 4)
  nucleoid.run("if ( shipment.test ) { shipment.test4 = shipment.test3 } ");

describe("Nucleoid", function() {
  this.timeout(0);
  this.slow(0);

  it("create entity", function() {
    for (let i = 0; i < n; i++) nucleoid.run(`order${i} = new Order ( )`);
  });

  it("updates all entities with class declaration", function() {
    for (let i = 0; i < n; i++) nucleoid.run(`order${i}.barcode = ${i}`);
  });

  it("updates the same entity with class declaration", function() {
    for (let i = 0; i < n; i++) nucleoid.run(`order.barcode = ${i}`);
  });

  it("updates the same entity with property declaration", function() {
    for (let i = 0; i < n; i++) nucleoid.run(`shipment.test = ${i}`);
  });

  it("delete entities", function() {
    for (let i = 0; i < n; i++) nucleoid.run(`delete order${i}`);
  });
});
