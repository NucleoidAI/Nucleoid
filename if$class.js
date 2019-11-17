var IF$INSTANCE = require("./if$instance");
var $BLOCK = require("./$block");
var Node = require("./node");
var graph = require("./graph");
var Instance = require("./instance");

class IF$CLASS extends Node {
  prepare() {
    this.key = "if(" + this.condition.tokens.join("") + ")";
  }

  run(scope) {
    let instances;
    let statements = [];

    let instance = Instance.retrieve(scope, this.class.name);

    if (instance) instances = [instance];
    else instances = Object.keys(this.class.instance).map(i => graph[i]);

    for (let instance of instances) {
      let statement = new IF$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.declaration = this;

      statement.true = $BLOCK(this.true.statements);
      statement.true.class = this.class;
      statement.true.instance = statement.instance;

      if (this.false) {
        statement.false = $BLOCK(this.false.statements);
        statement.false.class = this.class;
        statement.false.instance = statement.instance;
      }

      statements.push(statement);
    }

    return statements;
  }

  graph() {
    this.class.declaration[this.key] = this;
  }
}

IF$CLASS.prototype.type = "CLASS";
module.exports = IF$CLASS;
