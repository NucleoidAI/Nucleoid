const graph = require("./graph");
const BLOCK = require("./nuc/BLOCK");
const IF = require("./nuc/IF");
const Instruction = require("./instruction");
const Scope = require("./scope");
const Node = require("./nuc/Node");
const $ = require("./lang/$nuc/$");
const BREAK = require("./nuc/BREAK");
const EXPRESSION = require("./nuc/EXPRESSION");
const state = require("./state");
const RETURN = require("./nuc/RETURN");
const transaction = require("./transaction");

function process(statements, prior, options = {}) {
  const root = new Scope(prior);
  const { declarative } = options;

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

    switch (true) {
      case statement instanceof RETURN: {
        let scope = instruction.scope;
        return process(statement.statements, scope, options);
      }
      case statement instanceof BREAK: {
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

        break;
      }
      case statement instanceof EXPRESSION: {
        let scope = instruction.scope;

        const evaluation = statement.run(scope, false, false);
        transaction.push(evaluation.transactions);

        if (instruction.scope === root && !instruction.derivative) {
          result = state.expression(scope, evaluation);
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

        break;
      }
      case statement instanceof $: {
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
                : new Instruction(scope, statement, true, true, true, null); // root = null?
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

        break;
      }
      case statement instanceof Node: {
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
            const { destroyed } =
              statement.beforeGraph(instruction.scope) || {};

            if (destroyed) {
              continue;
            }

            if (statement instanceof Node) {
              if (graph[statement.key || statement.id]) {
                Node.replace(statement.key || statement.id, statement);
              } else {
                Node.register(statement);
              }
            }

            let list = statement.graph(instruction.scope);

            if (declarative) {
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

          break;
        }

        if (instruction.scope.block) {
          instruction.scope.block.stage(instruction);
          statement.block = instruction.scope.block;
        }
      }
    }
  }

  return result;
}

module.exports.process = process;
