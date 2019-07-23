var Node = require("./node");
var LET$INSTANCE = require("./let$instance");

module.exports = class LET$CLASS extends Node {
  run(scope) {
    if (scope.instance[this.class.name]) {
      let statement = new LET$INSTANCE();
      statement.class = this.class;
      statement.instance = scope.instance[this.class.name];
      statement.variable = this.variable;
      statement.expression = this.expression;
      return statement;
    }
  }
};
