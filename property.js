var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");
var Identifier = require("./identifier");

module.exports = class PROPERTY extends Node {
  prepare() {
    this.key = Identifier.serialize(this);
  }

  run(scope) {
    let object = eval("state." + Identifier.serialize(this.object, true));
    object[this.name] = this.value.run(scope);
  }

  graph(scope) {
    return this.value.graph(scope);
  }
};
