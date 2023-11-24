const state = require("./state");
const graph = require("./graph");
const BLOCK = require("./nuc/BLOCK");
const IF = require("./nuc/IF");
const Instruction = require("./instruction");
const Scope = require("./Scope");
const Node = require("./nuc/NODE");
const $ = require("./lang/$nuc/$");
const BREAK = require("./nuc/BREAK");
const EXPRESSION = require("./nuc/EXPRESSION");
const RETURN = require("./nuc/RETURN");

function process(statements, prior, options = {}) {
  const root = new Scope(prior);
  const { declarative } = options;

  let instructions = statements.map(
    (statement) => new Instruction(root, statement, true, true, false)
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

        const evaluation = statement.run(scope, false, false);

        if (
          evaluation &&
          instruction.scope === root &&
          !instruction.derivative
        ) {
          result.value = state.expression(scope, evaluation);
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

        if (!instruction.derivative) {
          // result.$nuc.push(statement);
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
                  : new Instruction(scope, statement, true, true, true);
              })
              .map((statement) => {
                statement.before = statement.before ?? instruction.before;
                statement.run = statement.run ?? instruction.run;
                statement.graph = statement.graph ?? instruction.graph;
                statement.derivative =
                  statement.derivative ?? instruction.derivative;
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
              if (graph.retrieve(statement.key)) {
                Node.replace(statement.key, statement);
              } else {
                Node.register(statement.key, statement);
              }
            }

            let list = statement.graph(instruction.scope);

            if (declarative) {
              if (list) {
                /*
                list.forEach((e) => {
                  if (graph[e].previous[statement.key] !== undefined) {
                    throw ReferenceError("Circular Dependency");
                  }
                });
                 */

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
                const targetKey = statement.key;
                Node.direct(source.key, targetKey, statement);
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
