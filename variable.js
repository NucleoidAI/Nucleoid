var state = require("./state");
var Node = require("./node");

class VARIABLE extends Node {
  prepare() {
    this.id = this.name;
  }

  run(scope) {
    state[this.name] = this.value.run(scope);
  }

  graph() {
    return this.value.graph();
  }
}

VARIABLE.prototype.type = "REGULAR";
module.exports = VARIABLE;
