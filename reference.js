var state = require("./state"); // eslint-disable-line no-unused-vars
var Identifier = require("./identifier");
var Type = require("./type");

class REFERENCE {
  prepare() {}
  run() {
    return eval("state." + Identifier.serialize(this.link));
  }

  next() {}
  graph() {
    return [Identifier.serialize(this.link)];
  }
}

REFERENCE.prototype.type = Type.EXPRESSION;
module.exports = REFERENCE;
