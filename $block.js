var BLOCK = require("./block");
var BLOCK$DECLARATION = require("./block$declaration");
var LET$DECLARATION = require("./let$declaration");
var PROPERTY$DECLARATION = require("./property$declaration");
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

    if (dependent instanceof LET$DECLARATION) {
      let statement = new BLOCK$DECLARATION();
      statement.statements = list;
      statement.class = dependent.class;
      return [
        new Instruction(scope, statement, true, true, false),
        new Instruction(scope, statement, false, false, true)
      ];
    } else if (dependent instanceof PROPERTY$DECLARATION) {
      let statement = new BLOCK$DECLARATION();
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
