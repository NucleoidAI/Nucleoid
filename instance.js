var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

module.exports = class INSTANCE extends Node {
  run() {
    state[this.variable] = eval("new state." + this.class.name + "()");
  }

  graph() {
    graph.node[this.variable] = this;
    this.class.instance[this.variable] = this;
  }
};
