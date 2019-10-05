var Node = require("./node");
var LET$INSTANCE = require("./let$instance");

module.exports = class LET$CLASS extends Node {
  run(scope) {
    let instance = scope.retrieve(scope, this.class.name);

    if (instance) {
      let statement = new LET$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.name = this.name;
      statement.value = this.value;
      return statement;
    }
  }
};
