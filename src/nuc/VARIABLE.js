const state = require("../state");
const Node = require("./NODE");

class VARIABLE extends Node {
  before(scope) {
    this.value.before(scope);
  }

  run(scope) {
    const evaluation = this.value.run(scope);
    const value = state.assign(scope, this.name, evaluation);

    return { value };
  }

  graph(scope) {
    return this.value.graph(scope);
  }
}

module.exports = VARIABLE;
