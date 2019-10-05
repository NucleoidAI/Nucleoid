var state = require("./state");
var Node = require("./node");

class INSTANCE extends Node {
  run(scope) {
    this.id = this.name;

    state[this.name] = eval("new state." + this.class.name + "()");
    scope.instance[this.class.name] = this;

    let list = [];

    for (let node in this.class.declaration)
      list.push(this.class.declaration[node]);

    return list;
  }

  graph() {
    this.class.instance[this.id] = this;
  }
}

INSTANCE.prototype.type = "REGULAR";
module.exports = INSTANCE;
