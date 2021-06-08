const state = require("./state");
const Node = require("./node");
const Identifier = require("./identifier");

module.exports = class PROPERTY extends Node {
  before(scope) {
    this.key = Identifier.serialize(this, true);
    this.value.before(scope, this.key);
  }

  run(scope) {
    let object = Identifier.serialize(this.object, true);
    let value = this.value.run(scope);
    state.assign(scope, object + "." + this.name, value);
  }

  graph(scope) {
    this.object.properties[this.name] = this;
    return this.value.graph(scope);
  }
};
