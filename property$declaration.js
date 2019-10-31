var Node = require("./node");
var PROPERTY$PROTOTYPE = require("./property$prototype");
var graph = require("./graph");
var Identifier = require("./identifier");

class PROPERTY$DECLARATION extends Node {
  prepare() {
    this.key = Identifier.serialize(this);
  }

  run(scope) {
    let instances;
    let statements = [];

    let instance = scope.retrieve(Identifier.root(this).name);

    if (instance) instances = [instance];
    else
      instances = Object.keys(Identifier.root(this).instance).map(
        i => graph[i]
      );

    for (let instance of instances) {
      let statement = new PROPERTY$PROTOTYPE();
      statement.object = instance;
      statement.template = instance;
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

PROPERTY$DECLARATION.prototype.type = "DECLARATION";
module.exports = PROPERTY$DECLARATION;
