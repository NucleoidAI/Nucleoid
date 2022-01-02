const state = require("./state");
const graph = require("./graph");
const Instruction = require("./instruction");
const Node = require("./node");

module.exports = class DELETE {
  before() {}

  run(scope) {
    if (graph[this.key]) {
      state.run(scope, "delete state." + this.key);

      let list = [];

      for (let node in graph[this.key].next) {
        list.push(new Instruction(scope.root, graph[node], false, true, false));
      }

      return { next: list, value: true };
    } else {
      return { value: false };
    }
  }

  beforeGraph() {}

  graph() {
    if (!graph[this.key]) return;

    for (let node in graph[this.key].previous)
      delete graph[node].next[this.key];

    let empty = new Node();

    for (let node in graph[this.key].next) {
      empty.next[node] = graph[this.key].next[node];
      delete graph[this.key].next[node];
    }

    let name = graph[this.key].name;
    delete graph[this.key].object.properties[name];
    delete graph[this.key];
    graph[this.key] = empty;
  }
};
