const DELETE = require("./DELETE");
const graph = require("../graph");
const state = require("../state");

class DELETE$OBJECT extends DELETE {
  run(scope) {
    const node = graph.retrieve(this.variable.key);
    let name = node.name;

    if (Object.keys(node.properties).length > 0)
      throw ReferenceError(`Cannot delete object '${this.variable.key}'`);

    if (node.object) {
      delete node.object.properties[name];
    } else {
      const list = node.class.list.toString();
      state.delete(scope, `${list}.${name}`);

      const index = state.$[list].findIndex((object) => object.id === node.key);
      state.$[list].splice(index, 1);
    }

    return super.run();
  }

  graph() {
    const node = graph.retrieve(this.variable.key);

    for (const key in node.previous) {
      delete node.next[key];
    }

    delete graph.$[node.key];
  }
}

module.exports = DELETE$OBJECT;
