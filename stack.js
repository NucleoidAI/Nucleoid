var graph = require("./graph");
var EXPRESSION = require("./expression");
var BLOCK = require("./block");
var Instruction = require("./instruction");
var Scope = require("./scope");

module.exports.process = function(statements) {
  let root = new Scope();
  let instructions = statements.map(
    statement => new Instruction(root, statement)
  );
  let result;

  while (instructions.length != 0) {
    let instruction = instructions.shift();
    let statement = instruction.statement;

    if (statement instanceof EXPRESSION) {
      result = statement.run(instruction.scope);
    } else {
      result = statement.run(instruction.scope);

      if (result && !Array.isArray(result)) {
        result = [result];
      }

      if (result) {
        let scope = instruction.scope;

        if (statement instanceof BLOCK) {
          scope = new Scope(scope);
          scope.block = true;
        }

        instructions = result
          .map(statement => {
            return new Instruction(scope, statement);
          })
          .concat(instructions);
      }

      if (instruction.graph && !instruction.scope.block) {
        statement.graph();
        Object.freeze(statement);
      }
    }

    for (let node in statement.next) {
      instructions.unshift(
        new Instruction(instruction.scope, graph.node[node], false)
      );
    }
  }

  return result;
};
