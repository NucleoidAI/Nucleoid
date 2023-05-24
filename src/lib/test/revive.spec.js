const revive = require("../revive");
const { equal } = require("assert");
describe("Revive lib", () => {
  it("revives a node", () => {
    const node = {
      id: "abc123",
      instanceof: "VARIABLE",
      name: "a",
      test: null,
    };
    revive(node);
  });

  it("revives a node with reference", () => {
    const graph = require("../../graph");
    graph["12345"] = { instanceof: "PROPERTY", name: "b" };
    const node = {
      id: "abc123",
      instanceof: "VARIABLE",
      name: "a",
      test: { $ref: { id: "12345", source: "graph" } },
    };
    revive(node);
    equal(node.test.instanceof, "PROPERTY");
  });
});
