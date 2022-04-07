const parser = require("../parser");
const { equal, deepEqual } = require("assert");

describe("Parser", () => {
  it("parses arrow function with block", () => {
    const { args, fn } = parser.fn(
      "( req ) => { return User[req.query.user] }"
    );
    equal(fn, "{return User[req.query.user]}");
    deepEqual(args, ["req"]);
  });

  it("parses arrow function without block", () => {
    const { args, fn } = parser.fn("( req ) => User[req.query.user]");
    equal(fn, "User[req.query.user]");
    deepEqual(args, ["req"]);
  });
});
