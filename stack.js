var graph = require("./graph");
var EXPRESSION = require("./expression");
var BLOCK = require("./block");
var Instruction = require("./instruction");
var Scope = require("./scope");

module.exports.process = function(statements) {
  let root = new Scope("ROOT");
  let instructions = statements.map(
    statement => new Instruction(root, statement)
  );

  let result;

  while (instructions.length != 0) {
    let instruction = instructions.shift();
    let statement = instruction.statement;
    let scope = instruction.scope;

    if (statement instanceof EXPRESSION) {
      result = statement.run(scope);
    } else {
      result = statement.run(scope);

      if (result && !Array.isArray(result)) {
        result = [result];
      }

      if (result) {
        if (statement instanceof BLOCK) {
          let nestedScope = new Scope(statement.constructor);
          instructions = result
            .map(statement => new Instruction(nestedScope, statement))
            .concat(instructions);
        } else {
          instructions = result
            .map(statement => new Instruction(scope, statement))
            .concat(instructions);
        }
      }

      if (instruction.graph) {
        statement.graph();
        Object.freeze(statement);
      }
    }

    for (let node in statement.next) {
      instructions.unshift(new Instruction(scope, graph.node[node], false));
    }
  }

  return result;
};
