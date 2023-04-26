const state = require("../state");
const LET = require("./LET");

class LET$OBJECT extends LET {
  before() {}

  run(scope) {
    state.run(scope, `scope.local.${this.name}=state.${this.object.name}`);
  }

  graph() {}
}

module.exports = LET$OBJECT;
