const PROPERTY = require("./property");
const Id = require("./utils/identifier");

class PROPERTY$REFERENCE extends PROPERTY {
  constructor() {
    super();
    this.properties = {};
  }

  before(scope) {
    this.key = Id.serialize(this, false);
    this.value.before(scope, this.key);
  }
}

module.exports = PROPERTY$REFERENCE;
