var state = require("./state");
var LET = require("./let");

module.exports = class LET$OBJECT extends LET {
  before() {}

  run(scope) {
    state.run(scope, `scope.local.${this.name}=state.${this.object.name}`);
  }

  graph(scope) {
    scope.graph[this.name] = this;
  }
};
