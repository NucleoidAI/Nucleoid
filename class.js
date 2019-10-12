var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");

module.exports = class CLASS extends Node {
  constructor() {
    super();
    this.instance = {};
  }

  run() {
    eval("state." + this.name + "=" + "class" + "{}");
  }

  graph() {
    graph.node[this.name] = this;
  }
};
