const BLOCK = require("../../nuc/BLOCK");
const BLOCK$CLASS = require("../../nuc/BLOCK$CLASS");
const $ = require("./$");
const Instruction = require("../../instruction");
const Scope = require("../../Scope");
const { v4: uuid } = require("uuid");
const _ = require("lodash");
const LET = require("../../nuc/LET");
const REFERENCE = require("../../nuc/REFERENCE");
const $ASSIGNMENT = require("./$ASSIGNMENT");

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

    let $class;

    test: for (let statement of _.cloneDeep(this.statements)) {
      while (statement instanceof $) {
        if (statement.iof === "$ASSIGNMENT") {
          statement.before(test);
          statement.graph(test);
          statement = statement.run(test);
          statement = statement.statement;
        }

        statement.before(test);
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
