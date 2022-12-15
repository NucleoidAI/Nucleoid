const Node = require("./node");
const Instruction = require("./instruction");
const Scope = require("./scope");
const state = require("./state");

class IF extends Node {
  before(scope) {
    this.key = "if(" + this.condition.tokens.construct() + ")";
    this.condition.before(scope);
  }

  run(scope) {
    let s = new Scope(scope);
    let condition;

    if (scope.block && scope.block.skip) {
      condition = this.condition.run(scope, true);
    } else {
      condition = this.condition.run(scope);
    }

    if (state.run(scope, condition)) {
      return {
        next: [
          new Instruction(s, this.true, true, false),
          new Instruction(s, this.true, false, true),
        ],
      };
    } else if (this.false && this.false instanceof IF) {
      return { next: this.false.run(scope) };
    } else if (this.false) {
      return {
        next: [
          new Instruction(s, this.false, true, false),
          new Instruction(s, this.false, false, true),
        ],
      };
    }
  }

  graph(scope) {
    return this.condition.graph(scope);
  }
}

module.exports = IF;
