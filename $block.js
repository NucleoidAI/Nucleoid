var BLOCK = require("./block");
var BLOCK$CLASS = require("./block$class");
var LET$CLASS = require("./let$class");
var PROPERTY$CLASS = require("./property$class");
var $ = require("./$");
var Instruction = require("./instruction");

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

    if (dependent instanceof LET$CLASS) {
      let statement = new BLOCK$CLASS();
      statement.statements = list;
      statement.class = dependent.class;
      return [
        new Instruction(scope, statement, true, true, false),
        new Instruction(scope, statement, false, false, true)
      ];
    } else if (dependent instanceof PROPERTY$CLASS) {
      let statement = new BLOCK$CLASS();
      statement.statements = list;
      statement.class = dependent.object;
      return [
        new Instruction(scope, statement, true, true, false),
        new Instruction(scope, statement, false, false, true)
      ];
    } else {
      let statement = new BLOCK();
      statement.statements = list;

      return [
        new Instruction(scope, statement, true, true, false),
        new Instruction(scope, statement, false, false, true)
      ];
    }
  }
}
