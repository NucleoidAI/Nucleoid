var graph = require("./graph");
var Node = require("./node");
var Instruction = require("./instruction");
var Scope = require("./scope");

class IF extends Node {
  run(scope) {
    this.id = "if(" + this.condition.tokens.join("") + ")";
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

  graph() {
    return this.condition.tokens.filter(token => graph.node[token]);
  }
}

IF.prototype.type = "REGULAR";
module.exports = IF;
