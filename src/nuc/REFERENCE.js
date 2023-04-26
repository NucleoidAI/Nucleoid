const Id = require("../lib/identifier");
const EXPRESSION = require("./EXPRESSION");
const Local = require("../lib/local");

class REFERENCE extends EXPRESSION {
  before() {}

  run(scope) {
    if (this.link.name) {
      let local = Local.retrieve(scope, this.link.name);

      if (local) return local;
    }

    return "state." + Id.serialize(this.link);
  }

  next() {}

  graph() {
    return [Id.serialize(this.link)];
  }
}

REFERENCE.prototype.instanceof = "REFERENCE";
module.exports = REFERENCE;
