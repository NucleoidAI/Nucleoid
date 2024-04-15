const state = require("./state");
const graph = require("./graph");
const NODE = require("./nuc/NODE");
const BLOCK = require("./nuc/BLOCK");
const IF = require("./nuc/IF");
const Instruction = require("./Instruction");
const Scope = require("./Scope");
const Node = require("./nuc/NODE");
const $ = require("./lang/$nuc/$");
const BREAK = require("./nuc/BREAK");
const EXPRESSION = require("./nuc/EXPRESSION");
const RETURN = require("./nuc/RETURN");

function process(statements, prior, options = {}) {
  const root = new Scope(prior);

  let instructions = statements.map(
    (statement) =>
      new Instruction(root, statement, true, true, false, false, false) // TODO Confirm graph is false
  );

  let result = { value: undefined, $nuc: [] };
  let dependencies = [];
  let dependents = [];
  let priorities = [];

  while (instructions.length) {
    let instruction = instructions.shift();
    let statement = instruction.statement;

    switch (true) {
      case statement instanceof RETURN: {
        let scope = instruction.scope;
        return process([statement.statement], scope, options);
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

        statement.before(scope);
        const evaluation = statement.run(scope, false, false);

        let value;

        if (evaluation) {
          value = state.expression(scope, evaluation);
        }

        if (value === "use declarative") {
          options.declarative = true;
        }

        if (value === "use imperative") {
          options.declarative = false;
        }

        if (instruction.scope === root && !instruction.derivative) {
          result.value = value;
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
                  false,
                  instruction.derivative
                );
              }
            })
            .concat(instructions);
        }

        break;
      }
      case statement instanceof $: {
        if (instruction.before && !statement.prepared) {
          statement.before(instruction.scope);
          statement.prepared = true;
        }

        if (instruction.run) {
          let next = statement.run(instruction.scope);
          next = Array.isArray(next) ? next : [next];
          next.push(
            new Instruction(
              instruction.scope,
              statement,
              false,
              false,
              true,
              true,
              null
            )
          );

          const scope = instruction.scope;

          next = next
            .map((statement) => {
              return statement instanceof Instruction
                ? statement
                : new Instruction(
                    scope,
                    statement,
                    true,
                    true,
                    true,
                    true,
                    null
                  );
            })
            .map((statement) => {
              statement.before = statement.before ?? instruction.before;
              statement.run = statement.run ?? instruction.run;
              statement.graph = statement.graph ?? instruction.graph;
              statement.after = statement.after ?? instruction.after;
              statement.derivative =
                statement.derivative ?? instruction.derivative;
              return statement;
            });

          instructions = next.concat(instructions);
        }

        if (instruction.graph) {
          statement.graph(instruction.scope);
        }

        if (instruction.after) {
          statement.after(instruction.scope);

          if (!instruction.derivative && !statement.asg) {
            if (statement.iof === "$EXPRESSION") {
              if (statement.tkns.wrt) {
                result.$nuc.push(statement);
              }
            } else {
              result.$nuc.push(statement);
            }
          }
        }

        break;
      }
      default: {
        if (instruction.before) {
          statement.before(instruction.scope);
        }

        if (instruction.run) {
          const { scope } = instruction;
          let { value, next } = statement.run(instruction.scope) || {};

          if (instruction.scope === root && !instruction.derivative) {
            result.value = value;
          }

          if (next) {
            next = Array.isArray(next) ? next : [next];

            next = next
              .map((statement) => {
                return statement instanceof Instruction
                  ? statement
                  : new Instruction(scope, statement, true, true, true, true);
              })
              .map((statement) => {
                statement.before = statement.before ?? instruction.before;
                statement.run = statement.run ?? instruction.run;
                statement.graph = statement.graph ?? instruction.graph;
                return statement;
              });

            instructions = next.filter((i) => !i.priority).concat(instructions);
            priorities = next.filter((i) => i.priority).concat(priorities);
          }
        }

        skip: {
          if (statement.type === "CLASS" && instruction.scope.prior) {
            break skip;
          }

          if (instruction.graph) {
            const { destroyed } =
              statement.beforeGraph(instruction.scope) || {};

            if (destroyed) {
              continue;
            }

            if (statement instanceof NODE) {
              if (graph.retrieve(statement.key)) {
                Node.replace(statement.key, statement);
              } else {
                Node.register(statement.key, statement);
              }
            }

            let list = statement.graph(instruction.scope);

            if (options.declarative) {
              if (list) {
                list.forEach((target) => {
                  if (target.previous[statement.key] !== undefined) {
                    throw ReferenceError("Circular Dependency");
                  }
                });

                dependencies = dependencies.concat(list);
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
                    new Instruction(scope, n, false, true, false, false)
                  );
                  dependents.push(
                    new Instruction(scope, n, false, false, true, true)
                  );
                } else {
                  dependents.push(
                    new Instruction(s, n, false, true, false, false)
                  );
                }
              });
          }

          // Root scope is a scope, which does not have any prior
          if (!instruction.scope.prior) {
            if (!instruction.statement.skip) {
              dependencies.forEach((source) => {
                const targetKey = statement.key;
                NODE.direct(source.key, targetKey, statement);
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
