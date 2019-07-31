var LET = require("./let");
var LET$INSTANCE = require("./let$instance");

module.exports = class LET$CLASS extends LET {
  run(scope) {
    let instance = scope.retrieve(scope, this.class.name);

    if (instance) {
      let statement = new LET$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.variable = this.variable;
      statement.expression = this.expression;
      return statement;
    }
  }
};
