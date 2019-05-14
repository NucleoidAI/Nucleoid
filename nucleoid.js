var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./state").state.graph;
var ControlFlow = require("./controlflow");
var Variable = require("./variable");
var If = require("./if");

module.exports.run = function(string) {
  let controlFlow = new ControlFlow(string);
  let stack = controlFlow.extract();

  while (stack.length != 0) {
    let statement = stack.shift();

    switch (statement.constructor) {
      case Variable: {
        eval(statement.tokens);
        break;
      }

      case If: {
        let result = eval(statement.expression);

        if (result) {
          let cf = new ControlFlow(statement.trueBlock);
          let list = cf.extract();
          stack = list.concat(stack);
        }

        break;
      }

      default: {
        return eval(statement.tokens);
      }
    }

    for (let n in graph[statement.key].nodes) {
      stack.push(graph[n].data.statement);
    }
  }
};
