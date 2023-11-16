const Function = require("../Function");
const Scope = require("../../../scope");
const { equal } = require("assert");

describe("Function", () => {
  it("resolves identifiers", () => {
    const func = new Function("(user) => user.age === age");

    const scope = new Scope();
    scope.local.age = 10;
    equal(func.generate(scope), "user=>user.age===scope.local.age");
  });

  it("resolves identifiers with return", () => {
    const func = new Function("(user) => { user.age = age ; return user.age }");

    const scope = new Scope();
    scope.local.age = 10;
    equal(
      func.generate(scope),
      "user=>{user.age=scope.local.age;return user.age}"
    );
  });

  it("resolves identifiers with if", () => {
    const func = new Function("(user) => { if ( age ) { return age } }");

    const scope = new Scope();
    scope.local.age = 10;
    equal(
      func.generate(scope),
      "user=>{if(scope.local.age){return scope.local.age}}"
    );
  });
});
