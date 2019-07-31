var graph = require("./graph");
var ASSIGNMENT = require("./assignment");
var ASSIGNMENT$INSTANCE = require("./assignment$instance");

module.exports = class ASSIGNMENT$CLASS extends ASSIGNMENT {
  run(scope) {
    let statements = [];

    let instance = scope.retrieve(scope, this.class.name);

    if (instance) {
      let statement = new ASSIGNMENT$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
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
