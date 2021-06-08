const Node = require("./node");
const Instruction = require("./instruction");
const Scope = require("./scope");
const BREAK = require("./break");
const state = require("./state");

module.exports = class IF extends Node {
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

      if (condition === "undefined") {
        return new BREAK(scope.block);
      }
    }

    if (state.run(scope, condition)) {
      return [
        new Instruction(s, this.true, true, false),
        new Instruction(s, this.true, false, true),
      ];
    } else if (this.false && this.false instanceof IF) {
      return this.false.run(scope);
    } else if (this.false) {
      return [
        new Instruction(s, this.false, true, false),
        new Instruction(s, this.false, false, true),
      ];
    }
  }

  graph(scope) {
    return this.condition.graph(scope);
  }
};
