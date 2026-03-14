const BLOCK = require("../../nuc/BLOCK");
const BLOCK$CLASS = require("../../nuc/BLOCK$CLASS");
const $ = require("./$");
const Instruction = require("../../Instruction");
const Scope = require("../../Scope");
const { v4: uuid } = require("uuid");
const _ = require("lodash");
const LET = require("../../nuc/LET");
const REFERENCE = require("../../nuc/REFERENCE");

function build(statements, skip) {
  let statement = new $BLOCK();
  statement.stms = statements;
  statement.skp = skip;
  return statement;
}

class $BLOCK extends $ {
  run(scope) {
    let test = new Scope(scope);
    test.object = scope.object;

    let $class;

    test: for (let statement of _.cloneDeep(this.stms)) {
      while (statement instanceof $) {
        if (statement.iof === "$ASSIGNMENT") {
          if (!statement.prepared) {
            statement.before(test);
            statement.prepared = true;
          }
          statement.graph(test);
          statement = statement.run(test);
          statement = statement.statement;
        }

        if (!statement.prepared) {
          statement.before(test);
          statement.prepared = true;
        }
        statement = statement.run(test);
      }

      const [result] = [statement]
        .flat(Infinity)
        .map((statement) =>
          statement instanceof Instruction ? statement.statement : statement
        );

      if (result instanceof LET && !(result.value instanceof REFERENCE)) {
        result.before(test);
        result.run(test);
        statement.beforeGraph(test);
        statement.graph(test);
        continue;
      } else if (result.type === "CLASS") {
        const OBJECT$CLASS = require("../../nuc/OBJECT$CLASS");
        const PROPERTY$CLASS = require("../../nuc/PROPERTY$CLASS");

        if (
          result instanceof PROPERTY$CLASS ||
          statement instanceof OBJECT$CLASS
        ) {
          $class = result.object;
        } else {
          $class = result.class;
        }

        break test;
      } else {
        break test;
      }
    }

    if ($class) {
      let statement = new BLOCK$CLASS(uuid());
      statement.class = $class;
      statement.statements = this.stms;
      return [
        new Instruction(scope, statement, true, true, false, false),
        new Instruction(scope, statement, false, false, true, true),
      ];
    } else {
      let statement = new BLOCK(uuid());
      statement.statements = this.stms;
      statement.skip = this.skp;

      return [
        new Instruction(scope, statement, true, true, false, false),
        new Instruction(scope, statement, false, false, true, true),
      ];
    }
  }
}

module.exports = build;
