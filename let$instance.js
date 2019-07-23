var Node = require("./node");

module.exports = class LET$INSTANCE extends Node {
  run(scope) {
    scope.local[this.variable] = this.expression.run(
      scope.local,
      this.instance
    );
  }
};
