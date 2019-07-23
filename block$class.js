var graph = require("./graph");
var Node = require("./node");
var BLOCK$INSTANCE = require("./block$instance");

module.exports = class BLOCK$CLASS extends Node {
  constructor() {
    super();
    this.statements = [];
  }
  run(scope) {
    let list = [];

    if (scope.instance[this.class.name]) {
      let statement = new BLOCK$INSTANCE();
      statement.class = this.class;
      statement.instance = scope.instance[this.class.name];
      statement.statements = this.statements;
      return statement;
    }

    for (let node in this.class.instance) {
      let statement = new BLOCK$INSTANCE();
      statement.class = this.class;
      statement.instance = this.class.instance[node];
      statement.statements = this.statements;
      list.push(statement);
    }

    return list;
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
