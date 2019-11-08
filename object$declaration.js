var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");
var Identifier = require("./identifier");
var graph = require("./graph");
var OBJECT$PROTOTYPE = require("./object$prototype");

module.exports = class OBJECT$DECLARATION extends Node {
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
      let statement = new OBJECT$PROTOTYPE();
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
};
