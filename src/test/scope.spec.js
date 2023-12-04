const Scope = require("../Scope");
const { equal } = require("assert");
const Identifier = require("../lang/ast/Identifier");

describe("Scope", () => {
  it("retrieves local variable", () => {
    const scope = new Scope();

    const variable = new Identifier("a");

    scope.graph.a = {};
    const identifier = scope.retrieve(variable);
    equal(identifier.toString(), "scope.local.a");
  });

  it("retrieves local variable from prior scope", () => {
    const scope1 = new Scope();
    const scope2 = new Scope(scope1);
    const scope3 = new Scope(scope2);

    const variable = new Identifier("a");

    scope1.graph.a = {};
    const identifier = scope3.retrieve(variable);
    equal(identifier.toString(), "scope.prior.prior.local.a");
  });

  it("retrieves local property from prior scope", () => {
    const scope1 = new Scope();
    const scope2 = new Scope(scope1);
    const scope3 = new Scope(scope2);

    scope1.graph.a = {};
    const variable = new Identifier("a.b.c");
    const identifier = scope3.retrieve(variable);
    equal(identifier.toString(), "scope.prior.prior.local.a.b.c");
  });
});
