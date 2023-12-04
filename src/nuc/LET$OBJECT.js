const LET = require("./LET");
const Evaluation = require("../lang/Evaluation");

class LET$OBJECT extends LET {
  before() {}

  run(scope) {
    const evaluation = new Evaluation(this.object.key);
    const value = scope.assign(this.name, `state.${evaluation}`);
    return { value };
  }

  graph() {}
}

module.exports = LET$OBJECT;
