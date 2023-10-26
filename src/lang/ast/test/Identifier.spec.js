const acorn = require("acorn");
const Identifier = require("../Identifier");
const { equal } = require("assert");
describe("Identifier", () => {
  it("serializes identifier expression", () => {
    const program = acorn.parse("var a;", { ecmaVersion: 2020 });
    const {
      declarations: [{ id }],
    } = program.body[0];

    const identifier = new Identifier(id);
    const serialize = identifier.resolve();
    equal(serialize, "a");
  });

  it("serializes member expression", () => {
    const program = acorn.parse("a.b = 1;", { ecmaVersion: 2020 });
    const {
      expression: { left },
    } = program.body[0];

    const identifier = new Identifier(left);
    const serialize = identifier.resolve();
    equal(serialize, "a.b");
  });
});
