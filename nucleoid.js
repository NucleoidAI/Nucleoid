var graph = require("./graph");
var Statement = require("./statement");
var IF = require("./if").IF;
var EXPRESSION = require("./expression").EXPRESSION;

module.exports.run = function(string) {
  let callStack = Statement.compile(string);
  let result;

  while (callStack.length != 0) {
    let statement = callStack.shift();

    switch (statement.constructor) {
      case IF: {
        result = statement.run();

        if (result) {
          let list = Statement.compile(statement.true);
          callStack = list.concat(callStack);
        }

        break;
      }

      case EXPRESSION: {
        result = statement.run();
        break;
      }

      default: {
        result = statement.run();

        if (result) {
          callStack = result.concat(callStack);
        }
      }
    }

    if (statement && statement.variable) {
      for (let n in graph.node[statement.variable].edge) {
        callStack.push(graph.node[n].statement);
      }
    }
  }

  return result;
};
