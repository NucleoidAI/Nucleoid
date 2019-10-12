var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");

module.exports = class DELETE {
  run() {
    eval("delete state." + this.variable);
  }

  graph() {
    for (let node in graph.node[this.variable].previous)
      delete graph.node[node].next[this.variable];

    delete graph.node[this.variable];
  }
};
