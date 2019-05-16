var graph = require("./graph");
var Node = require("./node");

module.exports = class BLOCK$INSTANCE {
  constructor() {
    this.statements = [];
  }

  run() {
    let statements = [];

    this.statements.forEach(statement => {
      statement.class = this.class;
      statement.instance = this.instance;
      statements.push(statement);
    });

    return statements;
  }

  graph() {
    let statement = this.statements[0];
    let expression = statement.expression;

    const tokens = expression.tokens.map(token => {
      let parts = token.split(".");
      if (parts[0] == this.class) parts[0] = this.instance;
      return parts.join(".");
    });

    const id = Date.now();
    graph.node[id] = new Node(this);
    expression = { tokens: tokens };

    expression.tokens.forEach(token => {
      if (graph.node[token]) graph.node[token].edge[id] = graph.node[id];
    });
  }
};
