var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

class PROPERTY extends Node {
  run(scope) {
    this.id = this.instance.name + "." + this.name;
    let instance = state[this.instance.name];
    instance[this.name] = this.value.run(scope);
  }

  graph() {
    return this.value.tokens.filter(token => {
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
