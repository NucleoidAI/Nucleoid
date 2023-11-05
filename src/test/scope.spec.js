const Scope = require("../scope");
const Evaluation = require("../lang/ast/Evaluation");
const { equal } = require("assert");
const Identifier = require("../lang/ast/Identifier");

describe("Scope", () => {
  it("retrieves local variable", () => {
    const scope = new Scope();

    const variable = new Identifier("a");

    scope.assign(variable, new Evaluation("1"));
    const identifier = scope.retrieve(variable);
    equal(identifier.toString(), "scope.local.a");
  });

  it("retrieves local variable from prior scope", () => {
    const scope1 = new Scope();
    const scope2 = new Scope(scope1);
    const scope3 = new Scope(scope2);

    const variable = new Identifier("a");

    scope1.assign(variable, new Evaluation("1"));
    const identifier = scope3.retrieve(variable);
    equal(identifier.toString(), "scope.prior.prior.local.a");
  });

  it("retrieves local property from prior scope", () => {
    const scope1 = new Scope();
    const scope2 = new Scope(scope1);
    const scope3 = new Scope(scope2);

    const variable1 = new Identifier("a");
    const variable2 = new Identifier("a.b.c");

    scope1.assign(variable1, new Evaluation("{ b: { c: 1 } }"));
    const identifier = scope3.retrieve(variable2);
    equal(identifier.toString(), "scope.prior.prior.local.a.b.c");
  });
});
