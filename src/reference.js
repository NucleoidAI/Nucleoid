const Id = require("./utils/identifier");
const EXPRESSION = require("./expression");

class REFERENCE extends EXPRESSION {
  before() {}

  run() {
    let reference = "state." + Id.serialize(this.link);
    return reference;
  }

  next() {}

  graph() {
    return [Id.serialize(this.link)];
  }
}

REFERENCE.prototype.instanceof = "REFERENCE";
module.exports = REFERENCE;
