var Node = require("./node");
var LET$INSTANCE = require("./let$instance");

module.exports = class LET$CLASS extends Node {
  run() {
    if (this.instance) {
      let statement = new LET$INSTANCE();
      statement.class = this.class;
      statement.instance = this.instance;
      statement.variable = this.variable;
      statement.expression = this.expression;
      return statement;
    }
  }
};
