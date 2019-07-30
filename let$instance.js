var LET = require("./let");

module.exports = class LET$INSTANCE extends LET {
  run(scope) {
    scope.local[this.variable] = this.expression.run(
      scope.local,
      this.instance
    );
  }
};
