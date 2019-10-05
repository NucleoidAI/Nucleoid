var Node = require("./node");
var $ = require("./$");

class BLOCK extends Node {
  constructor() {
    super();
    this.statements = [];
  }

  run() {
    let list = this.statements;
    this.statements = [];
    return list;
  }

  stage(instruction) {
    if (
      instruction.run &&
      !(instruction.statement instanceof $) &&
      instruction.statement.type != "CLASS"
    ) {
      let statement = instruction.statement;
      let block = statement.block;

      if (block && statement.id) {
        block.statements = block.statements.filter(s => s.id != statement.id);
      }

      this.statements.push(instruction.statement);
    }
  }
}

BLOCK.prototype.type = "REGULAR";
module.exports = BLOCK;
