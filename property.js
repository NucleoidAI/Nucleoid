var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");
var graph = require("./graph");

class PROPERTY extends Node {
  run(scope) {
    let prefix = this.instance.identifier();
    this.id = prefix + "." + this.name;
    let instance = eval("state." + prefix);
    instance[this.name] = this.value.run(scope);
  }

  graph() {
    return this.value.graph().filter(token => {
      if (graph.node[token]) return true;
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        return true;
      }
    });
  }
}

PROPERTY.prototype.type = "REGULAR";
module.exports = PROPERTY;
