const state = require("./state");
const Node = require("./node");

module.exports = class VARIABLE extends Node {
  before(scope) {
    this.key = this.name;
    this.value.before(scope, this.key);
  }

  run(scope) {
    state.assign(scope, this.name, this.value.run(scope));
  }

  graph(scope) {
    return this.value.graph(scope);
  }
};
