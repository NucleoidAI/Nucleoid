const PROPERTY = require("./property");
const Identifier = require("./identifier");

module.exports = class PROPERTY$REFERENCE extends PROPERTY {
  constructor() {
    super();
    this.properties = {};
  }

  before(scope) {
    this.key = Identifier.serialize(this, false);
    this.value.before(scope, this.key);
  }
};
