const state = require("../state");
const Node = require("./Node");
const Id = require("../lib/identifier");

class PROPERTY extends Node {
  before(scope) {
    this.key = Id.serialize(this, true);
    this.value.before(scope, this.key);
  }

  run(scope) {
    const evaluation = this.value.run(scope);
    const value = state.assign(
      scope,
      this.object.name + "." + this.name,
      evaluation
    );
    return { value };
  }

  graph(scope) {
    this.object.properties[this.name] = this;
    return this.value.graph(scope);
  }
}

module.exports = PROPERTY;
