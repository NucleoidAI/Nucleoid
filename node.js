var graph = require("./graph");

module.exports = class Node {
  constructor() {
    this.next = {};
    this.previous = {};
  }

  run() {}
  graph() {}

  static replace(sourceKey, targetNode) {
    for (let node in graph.node[sourceKey].next) {
      targetNode.next[node] = graph.node[sourceKey].next[node];
      delete graph.node[sourceKey].next[node];
    }

    for (let node in graph.node[sourceKey].previous) {
      delete graph.node[node].next[sourceKey];
    }

    graph.node[sourceKey] = targetNode;
  }

  static direct(sourceKey, targetKey, targetNode) {
    graph.node[sourceKey].next[targetKey] = targetNode;
    targetNode.previous[sourceKey] = graph.node[targetKey];
  }
};
