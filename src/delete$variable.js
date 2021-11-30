const DELETE = require("./delete");
const graph = require("./graph");

module.exports = class DELETE$VARIABLE extends DELETE {
  graph() {
    for (let node in graph[this.key].previous)
      delete graph[node].next[this.key];

    delete graph[this.key];
  }
};
