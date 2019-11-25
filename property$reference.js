var PROPERTY = require("./property");
var Identifier = require("./identifier");

module.exports = class PROPERTY$REFERENCE extends PROPERTY {
  constructor() {
    super();
    this.property = {};
  }

  prepare(scope) {
    this.key = Identifier.serialize(this, false);
    this.value.prepare(scope, this.key);
  }
};
