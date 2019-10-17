var graph = require("./graph");
var Node = require("./node");
var Instruction = require("./instruction");
var Scope = require("./scope");

class IF extends Node {
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
    return this.condition.graph(scope).filter(token => {
      if (graph.node[token]) return true;
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        return true;
      }
    });
  }
}

IF.prototype.type = "REGULAR";
module.exports = IF;
