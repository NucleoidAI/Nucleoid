const Function = require("../Function");
const Scope = require("../../../scope");
const { equal } = require("assert");

describe("Function", () => {
  it("resolves identifiers", () => {
    const func = new Function("(user) => user.age === age");

    const scope = new Scope();
    scope.graph.age = {};
    equal(func.first, null);
    equal(func.object, null);
    equal(func.last, null);
    equal(func, "user=>user.age===age");
    equal(func.generate(scope), "user=>user.age===scope.local.age");
  });

  it("resolves identifiers with return statement", () => {
    const func = new Function("(user) => { user.age = age ; return user.age }");

    const scope = new Scope();
    scope.graph.age = {};
    equal(func.first, null);
    equal(func.object, null);
    equal(func.last, null);
    equal(func, "user=>{user.age=age;return user.age}");
    equal(
      func.generate(scope),
      "user=>{user.age=scope.local.age;return user.age}"
    );
  });

  it("resolves identifiers with if statement", () => {
    const func = new Function("(user) => { if ( age ) { return age } }");

    const scope = new Scope();
    scope.graph.age = {};
    equal(func.first, null);
    equal(func.object, null);
    equal(func.last, null);
    equal(func, "user=>{if(age){return age}}");
    equal(
      func.generate(scope),
      "user=>{if(scope.local.age){return scope.local.age}}"
    );
  });
});
