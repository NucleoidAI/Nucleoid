const test = require("../lib/test");
const { equal, deepEqual } = require("assert");
const nucleoid = require("../../");
const stack = require("../stack");

describe("Nucleoid", () => {
  before(() => nucleoid.start({ test: true, options: { declarative: true } }));
  beforeEach(() => test.clear());

  it("stores $nuc", () => {
    nucleoid.run("a = 1");
    nucleoid.run("b = a + 2");
    nucleoid.run("a = 2");

    nucleoid.run("arr  = [ 1, 2, 3 ]");
    nucleoid.run("arr.push ( 4 )");

    nucleoid.run(
      "class User { constructor ( name, createdAt ) { this.name = name; this.createdAt = createdAt } }"
    );
    nucleoid.run("$User.active = true");
    nucleoid.run("if ( $User.name === 'Test' ) { $User.mode = 'TEST' }");
    nucleoid.run("new User ( 'Test', Date.now() )");

    const expectedUserList = nucleoid.run("User");

    const statements = nucleoid.datastore
      .read()
      .flatMap((statement) => statement.$)
      .filter((value) => value);

    test.clear();

    stack.process(statements, null, { declarative: true });

    equal(nucleoid.run("a"), 2);
    equal(nucleoid.run("b"), 4);

    nucleoid.run("c = b + 3");
    nucleoid.run("a = 3");

    equal(nucleoid.run("b"), 5);
    equal(nucleoid.run("c"), 8);

    deepEqual(nucleoid.run("arr"), [1, 2, 3, 4]);

    const actualUserList = nucleoid.run("User");
    deepEqual(actualUserList, expectedUserList);
  });
});
