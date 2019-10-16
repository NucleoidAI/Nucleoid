var state = require("./state");
var Node = require("./node");

class VARIABLE extends Node {
  prepare() {
    this.id = this.name;
  }

  run(scope) {
    state[this.name] = this.value.run(scope);
  }

  graph(scope) {
    return this.value.graph(scope);
  }
}

VARIABLE.prototype.type = "REGULAR";
module.exports = VARIABLE;
