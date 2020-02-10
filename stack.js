var graph = require("./graph");
var BLOCK = require("./block");
var IF = require("./if");
var Instruction = require("./instruction");
var Scope = require("./scope");
var Node = require("./node");
var $ = require("./$");
var BREAK = require("./break");
var EXPRESSION = require("./expression");

module.exports.process = function(statements) {
  let root = new Scope();

  let instructions = statements.map(
    statement => new Instruction(root, statement, true, true, false)
  );

  let result;
  let dependencies = [];
  let dependents = [];

  while (instructions.length !== 0) {
    let instruction = instructions.shift();
    let statement = instruction.statement;

    if (statement instanceof BREAK) {
      let inst = instructions[0];

      while (
        inst !== undefined &&
        statement.block !== undefined &&
        inst.scope.block === statement.block
      ) {
        instructions.shift();
        inst = instructions[0];
        statement.block.break = true;
      }
    } else if (statement instanceof EXPRESSION) {
      result = statement.run(instruction.scope);
      let list = statement.next(instruction.scope);

      if (list) {
        instructions = list
          .map(statement => {
            if (statement instanceof Instruction) {
              return statement;
            } else {
              let scope = instruction.scope;
              return new Instruction(scope, statement, false, true, false);
            }
          })
          .concat(instructions);
      }
    } else {
      if (instruction.before) {
        statement.before(instruction.scope);
      }

      if (instruction.run) {
        let list = statement.run(instruction.scope);

        if (list) {
          let scope = instruction.scope;
          list = Array.isArray(list) ? list : [list];

          if (statement instanceof BLOCK) {
            scope = new Scope(scope, statement);
          }

          if (statement instanceof IF) {
            scope = new Scope(scope);
          }

          instructions = list
            .map(statement => {
              return statement instanceof Instruction
                ? statement
                : new Instruction(scope, statement, true, true, true);
            })
            .filter(instruction => instruction.statement !== statement)
            .concat(instructions);
        }
      }

      skip: if (!(statement instanceof $)) {
        if (statement.type === "CLASS" && instruction.scope.prior) {
          break skip;
        }

        if (instruction.graph) {
          if (statement instanceof Node) {
            if (statement.key && graph[statement.key]) {
              Node.replace(statement.key, statement);
            } else if (graph[statement.id]) {
              Node.replace(statement.id, statement);
            } else if (statement.key) {
              graph[statement.key] = statement;
            } else {
              graph[statement.id] = statement;
            }
          }

          let list = statement.graph(instruction.scope);
          if (list) {
            list.forEach(e => {
              if (graph[e].previous[statement.key] !== undefined) {
                throw ReferenceError("Circular Dependency");
              }
            });

            dependencies = dependencies
              .concat(list.filter(e => !dependencies.includes(e)))
              .sort((a, b) => graph[a].sequence - graph[b].sequence);
          }
        }

        for (let node in statement.next) {
          let s = instruction.scope;
          let n = graph[node];

          if (n instanceof BLOCK || n instanceof IF) {
            let scope = new Scope();
            dependents.push(new Instruction(scope, n, false, true, false));
            dependents.push(new Instruction(scope, n, false, false, true));
          } else {
            dependents.push(new Instruction(s, n, false, true, false));
          }
        }

        // Root scope is a scope, which does not have any prior.
        if (!instruction.scope.prior) {
          dependencies.forEach(source => {
            let targetKey = statement.key ? statement.key : statement.id;
            Node.direct(source, targetKey, statement);
          });
          dependencies = [];

          instructions = dependents.concat(instructions);
          dependents = [];
        }
      }

      if (instruction.scope.block) {
        instruction.scope.block.stage(instruction);
        statement.block = instruction.scope.block;
      }
    }
  }

  return result;
};
