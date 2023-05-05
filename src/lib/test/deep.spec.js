const assert = require("assert");
const { deepEqual, deepMerge } = require("../deep");
describe("Deep lib", () => {
  it("compares two objects", () => {
    const obj1 = { a: 1, b: 2, c: { d: 3, e: 4 } };
    const obj2 = { a: 1, b: 2, c: { d: 3, e: 4 } };
    const obj3 = { a: 1, b: 2, c: { d: 3, e: 5 } };

    assert.equal(deepEqual(obj1, obj2), true);
    assert.equal(deepEqual(obj1, obj3), false);
  });

  it("merges two objects", () => {
    const obj1 = { a: 1, b: 2, c: { d: 3, e: 4 } };
    const obj2 = { b: 3, c: { d: 4, f: 5 }, g: 6 };
    const obj3 = { a: 1, b: 3, c: { d: 4, e: 4, f: 5 }, g: 6 };

    assert.deepEqual(deepMerge(obj1, obj2), obj3);
  });
});
