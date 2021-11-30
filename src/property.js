const state = require("./state");
const Node = require("./node");
const Id = require("./utils/identifier");

module.exports = class PROPERTY extends Node {
  before(scope) {
    this.key = Id.serialize(this, true);
    this.value.before(scope, this.key);
  }

  run(scope) {
    let object = Id.serialize(this.object, true);
    let value = this.value.run(scope);
    state.assign(scope, object + "." + this.name, value);
  }

  graph(scope) {
    this.object.properties[this.name] = this;
    return this.value.graph(scope);
  }
};
