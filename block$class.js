var graph = require("./graph");
var Node = require("./node");
var BLOCK$INSTANCE = require("./block$instance");

module.exports = class BLOCK$CLASS {
  constructor() {
    this.statements = [];
  }

  run(scope) {
    let list = [];

    let instance = scope.retrieve(scope, this.class.name);

    if (instance) {
      let statement = new BLOCK$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
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
    let dependent = this.statements[0];

    let key = Date.now();
    graph.node[key] = this;

    dependent.value.tokens.forEach(token => {
      if (graph.node[token]) Node.direct(token, key, this);
    });
  }
};
