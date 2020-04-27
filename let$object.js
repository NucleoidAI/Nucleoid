var state = require("./state");
var LET = require("./let");

module.exports = class LET$OBJECT extends LET {
  before() {}

  run(scope) {
    state.run(
      scope,
      "scope.local." + this.name + "= new state." + this.class.name + "()"
    );
  }

  graph(scope) {
    scope.graph[this.name] = this;
  }
};
