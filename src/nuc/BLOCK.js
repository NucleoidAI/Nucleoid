const Node = require("./NODE");
const $ = require("../lang/$nuc/$");
const Instruction = require("../instruction");
const Scope = require("../Scope");

class BLOCK extends Node {
  constructor(key) {
    super(key);
    this.statements = [];
  }

  run(scope) {
    const newScope = new Scope(scope);
    newScope.block = this;
    return {
      next: this.statements.map(
        (statement) => new Instruction(newScope, statement)
      ),
    };
  }

  stage(instruction) {
    if (
      instruction.scope.block.type === undefined &&
      instruction.statement.type === "CLASS"
    ) {
      throw SyntaxError("Cannot define class declaration in non-class block");
    }

    if (
      instruction.run &&
      !(instruction.statement instanceof $) &&
      instruction.statement.type !== "CLASS"
    ) {
      let statement = instruction.statement;
      let block = statement.block;

      if (block && statement.id) {
        block.statements = block.statements.filter(
          (s) => s.id !== statement.id
        );
      }

      this.statements.push(instruction.statement);
    }
  }
}

module.exports = BLOCK;
