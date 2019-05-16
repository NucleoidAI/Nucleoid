var graph = require("./graph");
var Node = require("./node");

module.exports = class BLOCK {
  constructor() {
    this.statements = [];
  }

  run() {
    return this.statements;
  }

  graph() {
    let statement = this.statements[0];
    let expression = statement.expression;

    const id = Date.now();
    graph.node[id] = new Node(this);
    graph.index[id] = [];

    expression.tokens.forEach(token => {
      if (graph.node[token]) {
        graph.node[token].edge[id] = graph.node[id];
        graph.index[id].push(token);
      }
    });
  }
};
