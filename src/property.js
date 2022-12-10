const state = require("./state");
const Node = require("./node");
const Id = require("./utils/identifier");

class PROPERTY extends Node {
  before(scope) {
    this.key = Id.serialize(this, true);
    this.value.before(scope, this.key);
  }

  run(scope) {
    let object = Id.serialize(this.object, true);
    const run = this.value.run(scope);
    const value = state.assign(scope, object + "." + this.name, run, true);
    return { value };
  }

  graph(scope) {
    this.object.properties[this.name] = this;
    return this.value.graph(scope);
  }
}

module.exports = PROPERTY;
