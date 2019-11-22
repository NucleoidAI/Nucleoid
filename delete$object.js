var DELETE = require("./delete");
var graph = require("./graph");

module.exports = class DELETE$OBJECT extends DELETE {
  run() {
    if (Object.keys(graph[this.key].property).length > 0) {
      throw new ReferenceError(`Cannot delete object '${this.key}'`);
    }

    super.run();
  }
  graph() {
    for (let node in graph[this.key].previous)
      delete graph[node].next[this.key];

    delete graph[this.key];
  }
};
