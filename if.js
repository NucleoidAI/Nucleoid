var Node = require("./node");
var Instruction = require("./instruction");
var Scope = require("./scope");
var BREAK = require("./break");

module.exports = class IF extends Node {
  prepare() {
    this.key = "if(" + this.condition.tokens.construct() + ")";
  }

  run(scope) {
    let s = new Scope(scope);
    let condition = this.condition.run(scope);

    if (condition === undefined) {
      return new BREAK(scope.block);
    }

    if (condition) {
      return [
        new Instruction(s, this.true, true, false),
        new Instruction(s, this.true, false, true)
      ];
    } else if (this.false && this.false instanceof IF) {
      return this.false.run(scope);
    } else if (this.false) {
      return [
        new Instruction(s, this.false, true, false),
        new Instruction(s, this.false, false, true)
      ];
    }
  }

  graph(scope) {
    return this.condition.graph(scope);
  }
};
