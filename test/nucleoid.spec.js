var assert = require("assert");
var nucleoid = require("../nucleoid");

describe("Nucleoid", function() {
  it("runs statements in the state", function() {
    nucleoid.run("var i = 1 ;");
    assert.equal(nucleoid.run("i == 1"), true);
  });
});
