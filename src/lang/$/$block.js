const BLOCK = require("../../block");
const BLOCK$CLASS = require("../../block$class");
const PROPERTY$CLASS = require("../../property$class");
const $ = require("./$");
const Instruction = require("../../instruction");
const Scope = require("../../scope");
const LET = require("../../let");
const OBJECT$CLASS = require("../../object$class");
const REFERENCE = require("../../reference");
const $LET = require("./$let");
const $EXP = require("./$expression");

module.exports = function (statements, skip, args) {
  let statement = new $BLOCK();
  statement.statements = statements;
  statement.skip = skip;
  statement.args = args;
  return statement;
};

class $BLOCK extends $ {
  run(scope) {
    let test = new Scope(scope);
    test.object = scope.object;
    let cls = null;

    test: for (let statement of this.statements) {
      while (statement instanceof $) {
        statement = statement.run(test);
      }

      if (statement instanceof LET && !(statement.value instanceof REFERENCE)) {
        statement.run(test);
        statement.beforeGraph(test);
        statement.graph(test);
        continue;
      } else if (statement.type === "CLASS") {
        if (
          statement instanceof PROPERTY$CLASS ||
          statement instanceof OBJECT$CLASS
        ) {
          cls = statement.object;
        } else {
          cls = statement.class;
        }

        break test;
      } else {
        break test;
      }
    }

    if (cls) {
      let statement = new BLOCK$CLASS();
      statement.statements = this.statements;
      statement.class = cls;
      return [
        new Instruction(scope, statement, true, true, false),
        new Instruction(scope, statement, false, false, true),
      ];
    } else {
      let statement = new BLOCK();
      statement.statements = this.statements;
      statement.skip = this.skip;

      if (this.args) {
        statement.statements = Object.entries(this.args)
          .map(([name, value]) => $LET(name, $EXP(value, 0).statement))
          .concat(statement.statements);
      }

      return [
        new Instruction(scope, statement, true, true, false),
        new Instruction(scope, statement, false, false, true),
      ];
    }
  }
}
