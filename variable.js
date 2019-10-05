var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

class VARIABLE extends Node {
  run(scope) {
    this.id = this.name;
    state[this.name] = this.value.run(scope);
  }

  graph() {
    return this.value.tokens.filter(token => graph.node[token]);
  }
}

VARIABLE.prototype.type = "REGULAR";
module.exports = VARIABLE;
