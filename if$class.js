var IF$INSTANCE = require("./if$instance");
var $BLOCK = require("./$block");
var Node = require("./node");

class IF$CLASS extends Node {
  run(scope) {
    this.id = "if(" + this.condition.tokens.join("") + ")";

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
    this.class.declaration[this.id] = this;
  }
}

IF$CLASS.prototype.type = "CLASS";
module.exports = IF$CLASS;
