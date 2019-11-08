var Node = require("./node");
var Instruction = require("./instruction");
var Scope = require("./scope");

module.exports = class IF extends Node {
  prepare() {
    this.key = "if(" + this.condition.tokens.join("") + ")";
  }

  run(scope) {
    let s = new Scope(scope);

    if (this.condition.run(scope)) {
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
