const state = require("../state");
const PROPERTY = require("./PROPERTY");
const Id = require("../lib/identifier");

class PROPERTY$REFERENCE extends PROPERTY {
  constructor() {
    super();
    this.properties = {};
  }

  before(scope) {
    this.key = Id.serialize(this, false);
    this.value.before(scope, this.key);
  }

  run(scope) {
    let object = Id.serialize(this.object, true);
    const value = state.assign(scope, object + "." + this.name, this.value);
    return { value };
  }
}

module.exports = PROPERTY$REFERENCE;
