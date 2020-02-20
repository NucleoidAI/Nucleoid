var Node = require("./node");
var PROPERTY$INSTANCE = require("./property$instance");
var graph = require("./graph");
var Identifier = require("./identifier");
var Instance = require("./instance");

class PROPERTY$CLASS extends Node {
  before() {
    this.key = Identifier.serialize(this);
  }

  run(scope) {
    let instances;
    let statements = [];

    let instance = Instance.retrieve(scope, Identifier.root(this).name);

    if (instance) instances = [instance];
    else
      instances = Object.keys(Identifier.root(this).instance).map(
        i => graph[i]
      );

    for (let instance of instances) {
      let statement = new PROPERTY$INSTANCE();
      statement.instance = instance;
      statement.name = this.name;
      statement.declaration = this;
      statements.push(statement);
    }

    return statements;
  }

  graph() {
    Identifier.root(this).declaration[this.key] = this;
  }
}

PROPERTY$CLASS.prototype.type = "CLASS";
module.exports = PROPERTY$CLASS;
