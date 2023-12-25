const test = require("../lib/test");
const { deepEqual } = require("assert");
const nucleoid = require("../../");
const stack = require("../stack");

describe("Nucleoid", () => {
  before(() => nucleoid.start({ declarative: false, test: true }));
  beforeEach(() => test.clear());

  it("stores $nuc", () => {
    nucleoid.run(
      "class User { constructor ( name, createdAt ) { this.name = name; this.createdAt = createdAt } }"
    );
    nucleoid.run("$User.active = true");
    nucleoid.run("if ( $User.name === 'Test' ) { $User.mode = 'TEST' }");
    nucleoid.run("new User ( 'Test', Date.now() )");

    const actualList = nucleoid.run("User");

    const statements = [];
    nucleoid.datastore
      .read()
      .forEach((statement) =>
        statement.$.forEach((item) => statements.push(item))
      );

    test.clear();

    stack.process(statements, null, { load: true });

    const expectedList = nucleoid.run("User");
    deepEqual(actualList, expectedList);
  });
});
