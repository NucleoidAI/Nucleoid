const $ = require("./$");
const FOR = require("../../nuc/FOR");
const Instruction = require("../../instruction");

function construct(variable, array, statements) {
  let statement = new $FOR();
  statement.variable = variable;
  statement.array = array;
  statement.statements = statements;
  return statement;
}

class $FOR extends $ {
  run(scope) {
    let statement = new FOR();
    statement.variable = this.variable;
    statement.array = this.array;
    statement.statements = this.statements;

    return new Instruction(scope, statement, false, true, false);
  }
}

module.exports = construct;
