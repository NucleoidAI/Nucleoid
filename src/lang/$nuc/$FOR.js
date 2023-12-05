const $ = require("./$");
const FOR = require("../../nuc/FOR");
const Instruction = require("../../instruction");
const Identifier = require("../ast/Identifier");

function build(variable, array, statements) {
  let statement = new $FOR();
  statement.variable = variable;
  statement.array = array;
  statement.statements = statements;
  return statement;
}

class $FOR extends $ {
  run(scope) {
    let statement = new FOR();
    statement.variable = new Identifier(this.variable);
    statement.array = new Identifier(this.array);
    statement.statements = this.statements;

    return new Instruction(scope, statement, false, true, false);
  }
}

module.exports = build;
