var state = require("./state");
var Node = require("./node");

module.exports = class VARIABLE extends Node {
  before(scope) {
    this.key = this.name;
    this.value.before(scope, this.key);
  }

  run(scope) {
    state[this.name] = this.value.run(scope);
  }

  graph(scope) {
    return this.value.graph(scope);
  }
};
