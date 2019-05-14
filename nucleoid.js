var graph = require("./graph");
var Statement = require("./statement");
var EXPRESSION = require("./expression");

module.exports.run = function(string) {
  let statements = Statement.compile(string);
  let callStack = statements.map(statement => {
    return { statement: statement, run: true, graph: true };
  });

  let result;

  while (callStack.length != 0) {
    let procedure = callStack.shift();
    let statement = procedure.statement;

    switch (statement.constructor) {
      case EXPRESSION: {
        result = statement.run();
        break;
      }

      default: {
        if (procedure.run) {
          result = statement.run();

          if (result && !Array.isArray(result)) {
            result = [result];
          }

          if (result) {
            callStack = result
              .map(statement => {
                return { statement: statement, run: true, graph: true };
              })
              .concat(callStack);
          }
        }

        if (procedure.graph) {
          statement.graph();
        }
      }
    }

    if (statement && statement.variable && !statement.instance) {
      for (let n in graph.node[statement.variable].edge) {
        callStack.push({
          statement: graph.node[n].statement,
          run: true,
          graph: false
        });
      }
    } else if (statement && statement.instance) {
      for (let n in graph.node[statement.instance].edge) {
        callStack.push({
          statement: graph.node[n].statement,
          run: true,
          graph: false
        });
      }
    }
  }

  return result;
};
