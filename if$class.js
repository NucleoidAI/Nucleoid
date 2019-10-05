var graph = require("./graph");
var IF$INSTANCE = require("./if$instance");
var $BLOCK = require("./$block");
var Node = require("./node");

module.exports = class IF$CLASS extends Node {
  run(scope) {
    let list = [];

    if (scope.instance[this.class.name]) {
      let statement = new IF$INSTANCE();
      statement.class = this.class;
      statement.instance = scope.instance[this.class.name];
      statement.condition = this.condition;

      statement.true = $BLOCK(this.true.statements);
      statement.true.class = this.class;
      statement.true.instance = statement.instance;
      list.push(statement);
    }

    for (let node in this.class.instance) {
      let statement = new IF$INSTANCE();
      statement.class = this.class;
      statement.instance = this.class.instance[node];
      statement.condition = this.condition;

      statement.true = $BLOCK(this.true.statements);
      statement.true.class = this.class;
      statement.true.instance = statement.instance;
      list.push(statement);
    }

    return list;
  }

  graph() {
    let key = "if(" + this.condition.tokens.join("") + ")";
    graph.node[key] = this;
    this.class.declaration[key] = this;
  }
};
