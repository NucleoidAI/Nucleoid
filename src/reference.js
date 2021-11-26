const Identifier = require("./identifier");
const EXPRESSION = require("./expression");

class REFERENCE extends EXPRESSION {
  before() {}

  run() {
    let reference = "state." + Identifier.serialize(this.link);
    return reference;
  }

  next() {}

  graph() {
    return [Identifier.serialize(this.link)];
  }
}

REFERENCE.prototype.instanceof = "REFERENCE";
module.exports = REFERENCE;
