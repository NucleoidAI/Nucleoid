var DELETE = require("./delete");
var graph = require("./graph");

module.exports = class DELETE$OBJECT extends DELETE {
  graph() {
    for (let node in graph[this.key].previous)
      delete graph[node].next[this.key];

    delete graph[this.key];
  }
};
