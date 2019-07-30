var graph = require("./graph");
var IF = require("./if");
var BLOCK$INSTANCE = require("./block$instance");
var IF$INSTANCE = require("./if$instance");

module.exports = class IF$CLASS extends IF {
  run(scope) {
    let list = [];

    if (scope.instance[this.class.name]) {
      let statement = new IF$INSTANCE();
      statement.class = this.class;
      statement.instance = scope.instance[this.class.name];
      statement.condition = this.condition;

      statement.true = new BLOCK$INSTANCE();
      statement.true.class = this.class;
      statement.true.instance = statement.instance;
      statement.true.statements = this.true.statements;
      list.push(statement);
    }

    for (let node in this.class.instance) {
      let statement = new IF$INSTANCE();
      statement.class = this.class;
      statement.instance = this.class.instance[node];
      statement.condition = this.condition;

      statement.true = new BLOCK$INSTANCE();
      statement.true.statements = this.true.statements;
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
