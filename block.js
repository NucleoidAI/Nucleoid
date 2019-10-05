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
    let dependent = this.statements[0];

    let key = Date.now();
    graph.node[key] = this;

    dependent.value.tokens.forEach(token => {
      if (graph.node[token]) Node.direct(token, key, this);
    });
  }
};
