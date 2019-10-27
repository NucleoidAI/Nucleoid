var Node = require("./node");
var PROPERTY$INSTANCE = require("./property$instance");
var graph = require("./graph");

class PROPERTY$CLASS extends Node {
  prepare() {
    this.key = this.class.name + "." + this.name;
  }

  run(scope) {
    let instances;
    let statements = [];

    let instance = scope.retrieve(this.class.name);

    if (instance) instances = [instance];
    else instances = Object.keys(this.class.instance).map(i => graph[i]);

    for (let instance of instances) {
      let statement = new PROPERTY$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.name = this.name;
      statement.declaration = this;
      statements.push(statement);
    }

    return statements;
  }

  graph() {
    this.class.declaration[this.key] = this;
  }
}

PROPERTY$CLASS.prototype.type = "CLASS";
module.exports = PROPERTY$CLASS;
