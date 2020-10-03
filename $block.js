var BLOCK = require("./block");
var BLOCK$CLASS = require("./block$class");
var PROPERTY$CLASS = require("./property$class");
var $ = require("./$");
var Instruction = require("./instruction");
var Scope = require("./scope");
var LET = require("./let");
var OBJECT$CLASS = require("./object$class");
var REFERENCE = require("./reference");

module.exports = function(statements, skip) {
  let statement = new $BLOCK();
  statement.statements = statements;
  statement.skip = skip;
  return statement;
};

class $BLOCK extends $ {
  run(scope) {
    let test = new Scope();
    test.object = scope.object;
    let cls = null;

    test: for (let statement of this.statements) {
      while (statement instanceof $) {
        statement = statement.run(test);
      }

      if (statement instanceof LET && !(statement.value instanceof REFERENCE)) {
        statement.run(test);
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
        new Instruction(scope, statement, false, false, true)
      ];
    } else {
      let statement = new BLOCK();
      statement.statements = this.statements;
      statement.skip = this.skip;

      return [
        new Instruction(scope, statement, true, true, false),
        new Instruction(scope, statement, false, false, true)
      ];
    }
  }
}
