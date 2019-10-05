var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");

class DELETE {
  run() {
    eval("delete state." + this.name);
  }

  graph() {
    for (let node in graph.node[this.name].previous)
      delete graph.node[node].next[this.name];

    delete graph.node[this.name];
  }
}

DELETE.prototype.type = "REGULAR";
module.exports = DELETE;
