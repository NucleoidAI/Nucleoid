const state = require("../state");
const graph = require("../graph");
const Instruction = require("../instruction");
const Node = require("./NODE");
const { v4: uuid } = require("uuid");

class DELETE {
  before() {}

  run(scope) {
    if (this.variable) {
      const key = this.variable.key;
      state.delete(scope, key);

      let list = [];

      for (let node in graph.retrieve(key).next) {
        const statement = graph.retrieve(node);
        list.push(new Instruction(scope.root, statement, false, true, false));
      }

      return { next: list, value: true };
    } else {
      return { value: false };
    }
  }

  beforeGraph() {}

  graph() {
    if (!this.variable) {
      return;
    }

    const node = graph.retrieve(this.variable.key);

    for (const key in node.previous) {
      delete node.next[key];
    }

    const empty = new Node(node.key);

    for (const key in node.next) {
      empty.next[key] = node.next[key];
      delete node.next[key];
    }

    const name = node.name;
    delete node.object.properties[name];
    delete graph.graph[node.key];
    graph.graph[node.key] = empty;
  }
}

module.exports = DELETE;
