var BLOCK = require("./block");
var BLOCK$CLASS = require("./block$class");
var LET$CLASS = require("./let$class");
var PROPERTY$CLASS = require("./property$class");
var $ = require("./$");

module.exports = function(statements) {
  let statement = new $BLOCK();
  statement.statements = statements;
  return statement;
};

class $BLOCK extends $ {
  run(scope) {
    let dependent = this.statements[0];
    while (dependent instanceof $) dependent = dependent.run(scope);
    let list = this.statements;
    list[0] = dependent;

    if (dependent instanceof LET$CLASS || dependent instanceof PROPERTY$CLASS) {
      let statement = new BLOCK$CLASS();
      statement.statements = list;
      statement.class = dependent.class;
      return statement;
    } else {
      let statement = new BLOCK();
      statement.statements = list;
      return statement;
    }
  }

  graph() {}
}
