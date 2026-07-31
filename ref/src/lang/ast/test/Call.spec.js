const Call = require("../Call");
const { equal } = require("assert");
const Scope = require("../../../Scope");

describe("Call", () => {
  it("resolves single expression", () => {
    const call = new Call("filter(1, 2)");

    const scope = new Scope();
    scope.graph.filter = {};
    equal(call.first, "filter");
    equal(call.last, "filter");
    equal(call.object, null);
    equal(call.function, "filter");
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
    equal(call.function, "user.accounts.filter");
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
    equal(call.function, "user.accounts.filter");
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
    scope.graph.type = {};
    equal(call.first, "user");
    equal(call.last, "filter");
    equal(call.object, "user.accounts");
    equal(call.function, "user.accounts.filter");
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
    scope.graph.level = {};
    equal(call.first, "user");
    equal(call.last, "filter");
    equal(call.object, "user.accounts");
    equal(call.function, "user.accounts.filter");
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
    const call = new Call("user.filter('ABC' + name.charAt(first));");

    const scope = new Scope();
    scope.graph.first = {};
    equal(call.first, "user");
    equal(call.last, "filter");
    equal(call.object, "user");
    equal(call.function, "user.filter");
    equal(call, "user.filter('ABC'+name.charAt(first))");
    equal(
      call.generate(scope),
      "user.filter('ABC'+name.charAt(scope.local.first))"
    );
  });

  it("resolves literal expression", () => {
    const call = new Call("/[A-Z]/.test(user);");

    const scope = new Scope();
    scope.graph.user = {};
    equal(call.first, "/[A-Z]/");
    equal(call.last, "test");
    equal(call.object, "/[A-Z]/");
    equal(call.function, "/[A-Z]/.test");
    equal(call, "/[A-Z]/.test(user)");
    equal(call.generate(scope), "/[A-Z]/.test(scope.local.user)");
  });
});
