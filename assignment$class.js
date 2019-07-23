var graph = require("./graph");
var Node = require("./node");
var ASSIGNMENT$INSTANCE = require("./assignment$instance");

module.exports = class ASSIGNMENT$CLASS extends Node {
  run(scope) {
    let statements = [];

    if (scope.instance[this.class.name]) {
      let statement = new ASSIGNMENT$INSTANCE();
      statement.class = this.class;
      statement.instance = scope.instance[this.class.name];
      statement.property = this.property;
      statement.expression = this.expression;
      return statement;
    }

    for (let node in this.class.instance) {
      let statement = new ASSIGNMENT$INSTANCE();
      statement.class = this.class;
      statement.instance = this.class.instance[node];
      statement.property = this.property;
      statement.expression = this.expression;
      statements.push(statement);
    }

    return statements;
  }

  graph() {
    let key = this.class.name + "." + this.property;
    graph.node[key] = this;
    this.class.declaration[key] = this;
  }
};
