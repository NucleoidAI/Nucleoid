const Call = require("../Call");
const { equal } = require("assert");
const Scope = require("../../../scope");

describe("Call", () => {
  it("resolves single expression", () => {
    const call = new Call("filter(1, 2)");

    const scope = new Scope();
    scope.graph.filter = {};
    equal(call.first, "filter");
    equal(call.last, "filter");
    equal(call.object, null);
    equal(call, "filter(1,2)");
    equal(call.generate(scope), "scope.local.filter(1,2)");
  });

  it("resolves member expression", () => {
    const call = new Call("user.accounts.filter(1, 2)");

    const scope = new Scope();
    scope.graph.user = {};
    equal(call.first, "user");
    equal(call.last, "filter");
    equal(call.object, "user.accounts");
    equal(call, "user.accounts.filter(1,2)");
    equal(call.generate(scope), "scope.local.user.accounts.filter(1,2)");
  });

  it("resolves chained expression", () => {
    const call = new Call("user.accounts.filter(1, 2).type.add(1, 2)");

    const scope = new Scope();
    scope.graph.user = {};
    equal(call.first, "user");
    equal(call.last, "filter");
    equal(call.object, "user.accounts");
    equal(call, "user.accounts.filter(1,2).type.add(1,2)");
    equal(
      call.generate(scope),
      "scope.local.user.accounts.filter(1,2).type.add(1,2)"
    );
  });

  it("resolves member expression with function", () => {
    const call = new Call(
      "user.accounts.filter((account) => account.type === type)"
    );

    const scope = new Scope();
    scope.graph.type = "TEST";
    equal(call.first, "user");
    equal(call.last, "filter");
    equal(call.object, "user.accounts");
    equal(call, "user.accounts.filter(account=>account.type===type)");
    equal(
      call.generate(scope),
      "user.accounts.filter(account=>account.type===scope.local.type)"
    );
  });

  it("resolves chained expression with function", () => {
    const call = new Call(
      "user.accounts.filter((account) => account.type === type).type.find((type) => type.level > level);"
    );

    const scope = new Scope();
    scope.graph.type = "TEST";
    scope.graph.level = 10;
    equal(call.first, "user");
    equal(call.object, "user.accounts");
    equal(call.last, "filter");
    equal(
      call,
      "user.accounts.filter(account=>account.type===type).type.find(type=>type.level>level)"
    );
    equal(
      call.generate(scope),
      "user.accounts.filter(account=>account.type===scope.local.type).type.find(type=>type.level>scope.local.level)"
    );
  });

  it("resolves member expression with another call", () => {
    const call = new Call("user.filter(name.charAt(first));");

    const scope = new Scope();
    scope.graph.first = 1;
    equal(call.first, "user");
    equal(call.last, "filter");
    equal(call.object, "user");
    equal(call, "user.filter(name.charAt(first))");
    equal(call.generate(scope), "user.filter(name.charAt(scope.local.first))");
  });

  it("resolves literal expression", () => {
    const call = new Call("/[A-Z]/.test(user);");

    const scope = new Scope();
    scope.graph.user = {};
    equal(call.first, "/[A-Z]/");
    equal(call.last, "test");
    equal(call.object, "/[A-Z]/");
    equal(call, "/[A-Z]/.test(user)");
    equal(call.generate(scope), "/[A-Z]/.test(scope.local.user)");
  });
});
