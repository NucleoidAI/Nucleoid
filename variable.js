var state = require("./state");
var Node = require("./node");

module.exports = class VARIABLE extends Node {
  prepare(scope) {
    this.key = this.name;
    this.value.prepare(scope);
  }

  run(scope) {
    state[this.name] = this.value.run(scope);
  }

  graph(scope) {
    return this.value.graph(scope);
  }
};
