const DELETE = require("./DELETE");
const graph = require("../graph");

class DELETE$VARIABLE extends DELETE {
  graph() {
    const node = graph.retrieve(this.variable.key);

    for (const key in node.previous) {
      delete node.next[key];
    }

    delete graph.graph[node.key];
  }
}

module.exports = DELETE$VARIABLE;
