var graph = require("./graph");
var uuidv4 = require("uuid/v4");

module.exports = class Node {
  constructor() {
    this.next = {};
    this.previous = {};
    this.id = uuidv4();
  }

  run() {}
  graph() {}

  static replace(sourceKey, targetNode) {
    targetNode.block = graph.node[sourceKey].block;

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
