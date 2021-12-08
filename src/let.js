const state = require("./state");

module.exports = class LET {
  constructor(name, value) {
    this.name = name;
    this.value = value;
  }
  before() {}

  run(scope) {
    let value = this.value.run(scope);
    let expression = `scope.local.${this.name}=${value}`;
    state.run(scope, expression);
  }

  beforeGraph(scope) {
    scope.graph[this.name] = this;
  }

  graph(scope) {
    if (scope.block !== undefined) {
      return this.value.graph(scope);
    }
  }
};
