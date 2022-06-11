const graph = require("./graph");
const BLOCK = require("./block");
const IF = require("./if");
const Instruction = require("./instruction");
const Scope = require("./scope");
const Node = require("./node");
const $ = require("./lang/$nuc/$");
const BREAK = require("./break");
const EXPRESSION = require("./expression");
const state = require("./state");
const RETURN = require("./return");

let runtime;

setImmediate(() => (runtime = require("./runtime")));

module.exports.process = function process(statements, prior) {
  const root = new Scope(prior);
  const { declarative } = runtime.options();

  let instructions = statements.map(
    (statement) => new Instruction(root, statement, true, true, false)
  );

  let result;
  let dependencies = [];
  let dependents = [];
  let priorities = [];

  while (instructions.length) {
    let instruction = instructions.shift();
    let statement = instruction.statement;

    if (statement instanceof RETURN) {
      let scope = instruction.scope;
      return process(statement.statements, scope);
    } else if (statement instanceof BREAK) {
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
      let scope = instruction.scope;

      const expression = statement.run(scope);
      const value = state.run(scope, expression);

      if (instruction.scope === root && !instruction.derivative) {
        result = value;
      }

      let list = statement.next(scope);

      if (list) {
        instructions = list
          .map((statement) => {
            if (statement instanceof Instruction) {
              return statement;
            } else {
              let scope = instruction.scope;
              return new Instruction(
                scope,
                statement,
                false,
                true,
                false,
                null,
                declarative
              );
            }
          })
          .concat(instructions);
      }
    } else if (statement instanceof $) {
      if (instruction.before) {
        statement.before(instruction.scope);
      }

      if (instruction.run) {
        let next = statement.run(instruction.scope);
        next = Array.isArray(next) ? next : [next];

        const scope = instruction.scope;
        const { derivative } = instruction;

        next = next
          .map((statement) => {
            return statement instanceof Instruction
              ? statement
              : new Instruction(scope, statement, true, true, true, null);
          })
          .map((instruction) => {
            if (instruction.derivative === undefined) {
              instruction.derivative = derivative;
            }
            return instruction;
          });

        instructions = next.filter((i) => !i.root).concat(instructions);
        priorities = next.filter((i) => i.root).concat(priorities);
      }
    } else {
      if (instruction.before) {
        statement.before(instruction.scope);
      }

      if (instruction.run) {
        let { value, next } = statement.run(instruction.scope) || {};

        if (instruction.scope === root && !instruction.derivative) {
          result = value;
        }

        if (next) {
          let scope = instruction.scope;
          next = Array.isArray(next) ? next : [next];

          if (statement instanceof BLOCK) {
            scope = new Scope(scope);
            scope.block = statement;
          }

          if (statement instanceof IF) {
            scope = new Scope(scope);
          }

          next = next
            .map((statement) => {
              return statement instanceof Instruction
                ? statement
                : new Instruction(scope, statement, true, true, true);
            })
            .map((statement) => {
              if (statement.derivative === undefined) {
                statement.derivative = true;
              }
              return statement;
            });

          instructions = next.filter((i) => !i.root).concat(instructions);
          priorities = next.filter((i) => i.root).concat(priorities);
        }
      }

      skip: if (!(statement instanceof $)) {
        if (statement.type === "CLASS" && instruction.scope.prior) {
          break skip;
        }

        if (instruction.graph) {
          statement.beforeGraph(instruction.scope);

          if (statement instanceof Node) {
            if (statement.key && graph[statement.key]) {
              Node.replace(statement.key, statement);
            } else if (graph[statement.id]) {
              Node.replace(statement.id, statement);
            } else if (statement.key) {
              Node.register(statement.key, statement);
            } else {
              Node.register(statement.id, statement);
            }
          }

          if (declarative) {
            let list = statement.graph(instruction.scope);
            if (list) {
              list.forEach((e) => {
                if (graph[e].previous[statement.key] !== undefined) {
                  throw ReferenceError("Circular Dependency");
                }
              });

              dependencies = dependencies.concat(
                list.filter((e) => !dependencies.includes(e))
              );
            }
          }
        }

        if (statement.next) {
          Object.values(statement.next)
            .sort((a, b) => a.sequence - b.sequence)
            .forEach((n) => {
              let s = instruction.scope;

              if (n instanceof BLOCK || n instanceof IF) {
                let scope = new Scope();
                dependents.push(
                  new Instruction(scope, n, false, true, false, null, true)
                );
                dependents.push(
                  new Instruction(scope, n, false, false, true, null, true)
                );
              } else {
                dependents.push(
                  new Instruction(s, n, false, true, false, null, true)
                );
              }
            });
        }

        // Root scope is a scope, which does not have any prior
        if (!instruction.scope.prior) {
          if (!instruction.statement.skip) {
            dependencies.forEach((source) => {
              let targetKey = statement.key ? statement.key : statement.id;
              Node.direct(source, targetKey, statement);
            });
          }

          instructions = instructions.concat(dependents);
          instructions = instructions.concat(priorities);

          dependencies = [];
          dependents = [];
          priorities = [];
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
