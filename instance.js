var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");

class INSTANCE extends Node {
  run(scope) {
    let name;
    if (this.instance) {
      name = this.instance.name + "." + this.name;
    } else {
      name = this.name;
    }

    this.id = name;

    eval("state." + name + " = new state." + this.class.name + "()");
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
