var Node = require("./node");
var PROPERTY$INSTANCE = require("./property$instance");

class PROPERTY$CLASS extends Node {
  run(scope) {
    this.id = this.class.name + "." + this.name;

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
    this.class.declaration[this.id] = this;
  }
}

PROPERTY$CLASS.prototype.type = "CLASS";
module.exports = PROPERTY$CLASS;
