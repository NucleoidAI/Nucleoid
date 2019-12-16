var DELETE = require("./delete");
var graph = require("./graph");

module.exports = class DELETE$OBJECT extends DELETE {
  run() {
    let name = graph[this.key].name;

    if (graph[this.key].object !== undefined)
      delete graph[this.key].object.property[name];

    if (Object.keys(graph[this.key].property).length > 0)
      throw ReferenceError(`Cannot delete object '${this.key}'`);

    super.run();
  }
  graph() {
    for (let node in graph[this.key].previous)
      delete graph[node].next[this.key];

    delete graph[this.key];
  }
};
