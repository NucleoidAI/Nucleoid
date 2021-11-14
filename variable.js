const state = require("./state");
const Node = require("./node");

module.exports = class VARIABLE extends Node {
  before(scope) {
    this.key = this.name;
    this.value.before(scope, this.key);
  }

  run(scope) {
    const value = this.value.run(scope);
    state.assign(scope, this.name, value);
    return { value };
  }

  graph(scope) {
    return this.value.graph(scope);
  }
};
