const DELETE = require("./DELETE");
const graph = require("../graph");

class DELETE$VARIABLE extends DELETE {
  graph() {
    for (let node in graph[this.key].previous)
      delete graph[node].next[this.key];

    delete graph[this.key];
  }
}

module.exports = DELETE$VARIABLE;
