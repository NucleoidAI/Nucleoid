var Node = require("./node");
var PROPERTY$INSTANCE = require("./property$instance");
var graph = require("./graph");
var Identifier = require("./identifier");

class PROPERTY$CLASS extends Node {
  run(scope) {
    this.id = this.class.name + "." + this.name;

    let instances;
    let statements = [];

    let instance = scope.retrieve(this.class.name);

    if (instance) instances = [instance];
    else instances = Object.keys(this.class.instance).map(i => graph.node[i]);

    for (let instance of instances) {
      let statement = new PROPERTY$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      let parts = this.object.split(".");
      parts[0] = Identifier.serialize(instance);

      statement.object = graph.node[parts.join(".")];
      statement.name = this.name;
      statement.declaration = this;
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
