const random = require("../random");
const { equal } = require("assert");

describe("Random", () => {
  it("returns random number with default length", () => {
    const string = random();
    equal(string.length, 16);
  });

  it("returns random number with given length", () => {
    const string = random(32);
    equal(string.length, 32);
  });
});
