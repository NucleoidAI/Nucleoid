var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");

module.exports = class DELETE {
  prepare() {}
  run() {
    eval("delete state." + this.name);
  }

  graph() {
    for (let node in graph[this.name].previous)
      delete graph[node].next[this.name];

    delete graph[this.name];
  }
};
