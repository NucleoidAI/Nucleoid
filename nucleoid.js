var graph = require("./graph");
var Statement = require("./statement");
var EXPRESSION = require("./expression");
var BLOCK = require("./block");
var Instruction = require("./instruction");
var Scope = require("./scope");

module.exports.run = function(string) {
  let statements = Statement.compile(string);
  let root = new Scope("ROOT");
  let callStack = statements.map(statement => new Instruction(root, statement));

  let result;

  while (callStack.length != 0) {
    let instruction = callStack.shift();
    let statement = instruction.statement;
    let scope = instruction.scope;

    switch (statement.constructor) {
      case EXPRESSION: {
        result = statement.run(scope.local);
        break;
      }

      default: {
        if (instruction.run) {
          result = statement.run(scope.local);

          if (result && !Array.isArray(result)) {
            result = [result];
          }

          if (result) {
            if (statement instanceof BLOCK) {
              let nestedScope = new Scope(statement.constructor);
              callStack = result
                .map(statement => new Instruction(nestedScope, statement))
                .concat(callStack);
            } else {
              callStack = result
                .map(statement => new Instruction(scope, statement))
                .concat(callStack);
            }
          }
        }

        if (instruction.graph) {
          statement.graph();
        }
      }
    }

    if (
      statement &&
      statement.variable &&
      graph.node[statement.variable] &&
      !statement.instance
    ) {
      for (let n in graph.node[statement.variable].edge) {
        callStack.unshift(
          new Instruction(scope, graph.node[n].statement, true, false)
        );
      }
    } else if (statement && statement.instance) {
      for (let n in graph.node[statement.instance].edge) {
        callStack.unshift(
          new Instruction(scope, graph.node[n].statement, true, false)
        );
      }
    }
  }

  return result;
};
