const state = require("./state");

module.exports = class LET {
  before() {}

  run(scope) {
    let value = this.value.run(scope);
    let expression = `scope.local.${this.name}=${value}`;
    state.run(scope, expression);
  }

  beforeGraph() {}

  graph(scope) {
    scope.graph[this.name] = this;

    if (scope.block !== undefined) {
      return this.value.graph(scope);
    }
  }
};
