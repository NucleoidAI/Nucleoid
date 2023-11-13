const Node = require("./NODE");
const Instruction = require("../instruction");
const Scope = require("../scope");
const state = require("../state");

class IF extends Node {
  before(scope) {
    this.condition.before(scope);
  }

  run(scope) {
    let local = new Scope(scope);
    let condition;

    if (scope.block?.skip) {
      condition = this.condition.run(scope, true);
    } else {
      condition = this.condition.run(scope);
    }

    if (state.expression(scope, condition)) {
      return {
        next: [
          new Instruction(local, this.true, true, false),
          new Instruction(local, this.true, false, true),
        ],
      };
    } else if (this.false && this.false instanceof IF) {
      return { next: this.false.run(scope) };
    } else if (this.false) {
      return {
        next: [
          new Instruction(local, this.false, true, false),
          new Instruction(local, this.false, false, true),
        ],
      };
    }
  }

  graph(scope) {
    return this.condition.graph(scope);
  }
}

module.exports = IF;
