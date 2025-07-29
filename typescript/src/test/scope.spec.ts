import Identifier from "../lang/ast/Identifier";
import Scope from "../Scope";
import { equal } from "assert";

describe("Scope", () => {
  it("retrieves local variable", () => {
    const scope = new Scope(null, {});

    const variable = new Identifier("a");

    scope.graph.a = {};
    const identifier = scope.retrieve(variable);

    if (identifier === null) {
      throw new Error("Identifier is null");
    }

    equal(identifier.toString(), "scope.local.a");
  });

  it("retrieves local variable from prior scope", () => {
    const scope = new Scope(null, {});
    const scope2 = new Scope(scope, {});
    const scope3 = new Scope(scope2, {});

    const variable = new Identifier("a");

    scope.graph.a = {};
    const identifier = scope3.retrieve(variable);
    if (identifier === null) {
      throw new Error("Identifier is null");
    }
    equal(identifier.toString(), "scope.prior.prior.local.a");
  });

  it("retrieves local property from prior scope", () => {
    const scope1 = new Scope(null, {});
    const scope2 = new Scope(scope1, {});
    const scope3 = new Scope(scope2, {});

    scope1.graph.a = {};
    const variable = new Identifier("a.b.c");
    const identifier = scope3.retrieve(variable);

    if (identifier === null) {
      throw new Error("Identifier is null");
    }

    equal(identifier.toString(), "scope.prior.prior.local.a.b.c");
  });
});

