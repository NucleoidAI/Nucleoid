const Id = require("./utils/identifier");
const EXPRESSION = require("./expression");
const Local = require("./utils/local");

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
