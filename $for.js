var $ = require("./$");
var FOR = require("./for");
var Instruction = require("./instruction");

module.exports = function(variable, array, statements) {
  let statement = new $FOR();
  statement.variable = variable;
  statement.array = array;
  statement.statements = statements;
  return statement;
};

class $FOR extends $ {
  run(scope) {
    let statement = new FOR();
    statement.variable = this.variable;
    statement.array = this.array;
    statement.statements = this.statements;

    return new Instruction(scope, statement, false, true, false);
  }
}
