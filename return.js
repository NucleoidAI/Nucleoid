const Node = require("./node");
const state = require("./state");

class RETURN extends Node {
  run(scope) {
    let value = this.expression.run(scope);
    let result = state.run(scope, value);
    return result;
  }
}
module.exports = RETURN;
