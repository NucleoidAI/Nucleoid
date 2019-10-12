var graph = require("./graph");
var Node = require("./node");

module.exports = class BLOCK extends Node {
  constructor() {
    super();
    this.statements = [];
  }

  run() {
    return this.statements;
  }

  graph() {
    let statement = this.statements[0];
    let expression = statement.expression;

    let key = Date.now();
    graph.node[key] = this;

    expression.tokens.forEach(token => {
      if (graph.node[token]) Node.direct(token, key, this);
    });
  }
};
