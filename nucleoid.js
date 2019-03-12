var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var ControlFlow = require("./controlflow");
var VARIABLE = require("./variable").VARIABLE;
var ASSIGNMENT = require("./assignment").ASSIGNMENT;
var IF = require("./if").IF;
var CLASS = require("./class").CLASS;
var EXPRESSION = require("./expression").EXPRESSION;

module.exports.run = function(string) {
  let callStack = ControlFlow.extract(string);
  let result;

  while (callStack.length != 0) {
    let statement = callStack.shift();

    switch (statement.constructor) {
      case VARIABLE:
      case ASSIGNMENT: {
        result = eval(statement.assignment);
        break;
      }

      case EXPRESSION: {
        result = eval(statement.expression);
        break;
      }

      case IF: {
        result = eval(statement.expression);

        if (result) {
          let list = ControlFlow.extract(statement.true);
          callStack = list.concat(callStack);
        }

        break;
      }

      case CLASS: {
        result = eval(statement.definition);
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
