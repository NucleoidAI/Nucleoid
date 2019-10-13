var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");

class PROPERTY extends Node {
  run(scope) {
    let prefix = this.instance.identifier();
    this.id = prefix + "." + this.name;
    let instance = eval("state." + prefix);
    instance[this.name] = this.value.run(scope);
  }

  graph() {
    return this.value.graph();
  }
}

PROPERTY.prototype.type = "REGULAR";
module.exports = PROPERTY;
