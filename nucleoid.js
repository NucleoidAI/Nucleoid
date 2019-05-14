var graph = require("./graph");
var Statement = require("./statement");
var EXPRESSION = require("./expression");

module.exports.run = function(string) {
  let callStack = Statement.compile(string);
  let result;

  while (callStack.length != 0) {
    let statement = callStack.shift();

    switch (statement.constructor) {
      case EXPRESSION: {
        result = statement.run();
        break;
      }

      default: {
        result = statement.run();

        if (result && !Array.isArray(result)) {
          result = [result];
        }

        if (result) {
          callStack = result.concat(callStack);
        }
      }
    }

    if (statement && statement.variable && !statement.instance) {
      for (let n in graph.node[statement.variable].edge) {
        callStack.push(graph.node[n].statement);
      }
    } else if (statement && statement.instance) {
      for (let n in graph.node[statement.instance].edge) {
        callStack.push(graph.node[n].statement);
      }
    }
  }

  return result;
};
