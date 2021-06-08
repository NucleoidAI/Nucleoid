var Node = require("./node");
var Identifier = require("./identifier");
var graph = require("./graph");
var OBJECT$INSTANCE = require("./object$instance");
var Instance = require("./instance");

class OBJECT$CLASS extends Node {
  before() {
    this.key = Identifier.serialize(this);
  }

  run(scope) {
    let instances;
    let statements = [];

    let instance = Instance.retrieve(scope, Identifier.root(this).name);

    if (instance) instances = [instance];
    else
      instances = Object.keys(Identifier.root(this).instances).map(
        (i) => graph[i]
      );

    for (let instance of instances) {
      let statement = new OBJECT$INSTANCE();
      statement.instance = instance;
      statement.name = this.name;
      statement.declaration = this;
      statements.push(statement);
    }

    return statements;
  }

  graph() {
    Identifier.root(this).declarations[this.key] = this;
  }
}

OBJECT$CLASS.prototype.type = "CLASS";
module.exports = OBJECT$CLASS;
