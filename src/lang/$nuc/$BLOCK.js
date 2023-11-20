const BLOCK = require("../../nuc/BLOCK");
const BLOCK$CLASS = require("../../nuc/BLOCK$CLASS");
const $ = require("./$");
const Instruction = require("../../instruction");
const Scope = require("../../Scope");
const { v4: uuid } = require("uuid");

function build(statements, skip) {
  let statement = new $BLOCK();
  statement.statements = statements;
  statement.skip = skip;
  return statement;
}

class $BLOCK extends $ {
  run(scope) {
    let test = new Scope(scope);
    test.object = scope.object;
    let cls = this.class;

    /*
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
    */

    if (cls) {
      let statement = new BLOCK$CLASS(uuid());
      statement.class = cls;
      statement.statements = this.statements;
      return [
        new Instruction(scope, statement, true, true, false),
        new Instruction(scope, statement, false, false, true),
      ];
    } else {
      let statement = new BLOCK(uuid());
      statement.statements = this.statements;
      statement.skip = this.skip;

      return [
        new Instruction(scope, statement, true, true, false),
        new Instruction(scope, statement, false, false, true),
      ];
    }
  }
}

module.exports = build;
