const $ = require("./$");
const FOR = require("../../nuc/FOR");
const Instruction = require("../../instruction");
const Identifier = require("../ast/Identifier");

function build(variable, array, statements) {
  let statement = new $FOR();
  statement.var = variable;
  statement.arr = array;
  statement.stms = statements;
  return statement;
}

class $FOR extends $ {
  run(scope) {
    let statement = new FOR();
    statement.variable = new Identifier(this.var);
    statement.array = new Identifier(this.arr);
    statement.statements = this.stms;

    return new Instruction(scope, statement, false, true, false);
  }
}

module.exports = build;
