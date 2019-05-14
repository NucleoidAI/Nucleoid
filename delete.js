var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");

module.exports = class DELETE {
  run() {
    eval("delete state." + this.variable);
  }

  graph() {
    const variable = this.variable;
    graph.index[variable].forEach(i => delete graph.node[i].edge[variable]);
    delete graph.node[variable];
    delete graph.index[variable];
  }
};
