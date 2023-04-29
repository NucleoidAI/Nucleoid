const DELETE = require("./DELETE");
const graph = require("../graph");
const { state } = require("../state");

class DELETE$OBJECT extends DELETE {
  run() {
    let name = graph[this.key].name;

    if (Object.keys(graph[this.key].properties).length > 0)
      throw ReferenceError(`Cannot delete object '${this.key}'`);

    if (graph[this.key].object) {
      delete graph[this.key].object.properties[name];
    } else {
      const list = graph[this.key].class.name.substring(1);
      delete state[list][this.key];

      const index = state[list].findIndex((object) => object.id === this.key);
      state[list].splice(index, 1);
    }

    return super.run();
  }

  graph() {
    for (let node in graph[this.key].previous)
      delete graph[node].next[this.key];

    delete graph[this.key];
  }
}

module.exports = DELETE$OBJECT;
