var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");

module.exports = class CLASS {
  run() {
    let definition = this.definition;

    graph.node[this.class] = new Node(this);
    eval(definition);
  }
};
