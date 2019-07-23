var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

module.exports = class INSTANCE extends Node {
  run(scope) {
    state[this.variable] = eval("new state." + this.class.name + "()");
    scope.instance[this.class.name] = this;

    let list = [];

    for (let node in this.class.declaration)
      list.push(this.class.declaration[node]);

    return list;
  }

  graph() {
    graph.node[this.variable] = this;
    this.class.instance[this.variable] = this;
  }
};
