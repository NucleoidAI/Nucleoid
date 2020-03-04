var graph = require("./graph");
var uuidv4 = require("uuid/v4");

var sequence = 0;

module.exports = class Node {
  constructor() {
    this.next = {};
    this.previous = {};
    this.id = uuidv4();
    this.sequence = sequence++;
  }

  before() {}
  run() {}
  beforeGraph() {}
  graph() {}

  static replace(sourceKey, targetNode) {
    targetNode.block = graph[sourceKey].block;

    for (let node in graph[sourceKey].next) {
      targetNode.next[node] = graph[sourceKey].next[node];
      delete graph[sourceKey].next[node];
    }

    for (let node in graph[sourceKey].previous) {
      delete graph[node].next[sourceKey];
    }

    graph[sourceKey] = targetNode;
  }

  static direct(sourceKey, targetKey, targetNode) {
    graph[sourceKey].next[targetKey] = targetNode;
    targetNode.previous[sourceKey] = graph[targetKey];
  }
};
