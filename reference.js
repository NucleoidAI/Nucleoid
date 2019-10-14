var state = require("./state"); // eslint-disable-line no-unused-vars
var Value = require("./value");
var Identifier = require("./identifier");

module.exports = class REFERENCE extends Value {
  run() {
    return eval("state." + Identifier.serialize(this.link));
  }

  graph() {
    return [Identifier.serialize(this.link)];
  }
};
