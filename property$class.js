var graph = require("./graph");
var Node = require("./node");
var PROPERTY$INSTANCE = require("./property$instance");

module.exports = class PROPERTY$CLASS extends Node {
  run(scope) {
    let statements = [];

    let instance = scope.retrieve(scope, this.class.name);

    if (instance) {
      let statement = new PROPERTY$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.name = this.name;
      statement.value = this.value;
      return statement;
    }

    for (let node in this.class.instance) {
      let statement = new PROPERTY$INSTANCE();
      statement.class = this.class;
      statement.instance = this.class.instance[node];
      statement.name = this.name;
      statement.value = this.value;
      statements.push(statement);
    }

    return statements;
  }

  graph() {
    let key = this.class.name + "." + this.name;
    graph.node[key] = this;
    this.class.declaration[key] = this;
  }
};
