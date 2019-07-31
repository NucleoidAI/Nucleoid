var Node = require("./node");

module.exports = class LET extends Node {
  run(scope) {
    scope.local[this.variable] = this.expression.run(scope);
  }
};
