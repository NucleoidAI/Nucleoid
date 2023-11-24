const Node = require("./NODE");
const Instruction = require("../instruction");
const Scope = require("../Scope");
const state = require("../state");

class IF extends Node {
  before(scope) {
    this.condition.before(scope);
  }

  run(scope) {
    let condition;

    if (scope.block?.skip) {
      condition = this.condition.run(scope, true);
    } else {
      condition = this.condition.run(scope);
    }

    if (!condition) {
      return;
    }

    if (state.expression(scope, condition)) {
      return {
        next: [
          new Instruction(scope, this.true, true, false),
          new Instruction(scope, this.true, false, true),
        ],
      };
    } else if (this.false && this.false instanceof IF) {
      return { next: this.false.run(scope) };
    } else if (this.false) {
      return {
        next: [
          new Instruction(scope, this.false, true, false),
          new Instruction(scope, this.false, false, true),
        ],
      };
    }
  }

  graph(scope) {
    return this.condition.graph(scope);
  }
}

module.exports = IF;
