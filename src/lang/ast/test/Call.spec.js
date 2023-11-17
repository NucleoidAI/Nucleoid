const Call = require("../Call");
const { equal } = require("assert");
const Scope = require("../../../scope");

describe("Call", () => {
  it("resolves single expression", () => {
    const call = new Call("filter(1, 2)");

    const scope = new Scope();
    scope.local.filter = {};
    equal(call.first.toString(), "filter");
    equal(call.object, null);
    equal(call.last, null);
    equal(call.toString(), "filter(1,2)");
    equal(call.generate(scope), "scope.local.filter(1,2)");
  });

  it("resolves member expression", () => {
    const call = new Call("user.accounts.filter(1, 2)");

    equal(call.first.toString(), "user");
    equal(call.object, null);
    equal(call.last, null);
    equal(call.toString(), "user.accounts.filter(1,2)");
  });

  it("resolves chained expression", () => {
    const call = new Call("user.accounts.filter(1, 2).type.add(1, 2)");

    const scope = new Scope();
    scope.local.user = {};
    equal(call.first.toString(), "user");
    equal(call.object, null);
    equal(call.last, null);
    equal(call.toString(), "user.accounts.filter(1,2).type.add(1,2)");
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
    scope.local.type = "TEST";
    equal(call.first.toString(), "user");
    equal(call.object, null);
    equal(call.last, null);
    equal(
      call.toString(),
      "user.accounts.filter(account=>account.type===type)"
    );
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
    scope.local.type = "TEST";
    scope.local.level = 10;
    equal(call.first.toString(), "user");
    equal(call.object, null);
    equal(call.last, null);
    equal(
      call.toString(),
      "user.accounts.filter(account=>account.type===type).type.find(type=>type.level>level)"
    );
    equal(
      call.generate(scope),
      "user.accounts.filter(account=>account.type===scope.local.type).type.find(type=>type.level>scope.local.level)"
    );
  });
});
