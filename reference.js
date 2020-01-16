var state = require("./state"); // eslint-disable-line no-unused-vars
var Identifier = require("./identifier");
var EXPRESSION = require("./expression");

class REFERENCE extends EXPRESSION {
  before() {}
  run() {
    return eval("state." + Identifier.serialize(this.link));
  }

  next() {}
  graph() {
    return [Identifier.serialize(this.link)];
  }
}

REFERENCE.prototype.instanceof = "REFERENCE";
module.exports = REFERENCE;
