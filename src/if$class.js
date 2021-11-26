const IF$INSTANCE = require("./if$instance");
const $BLOCK = require("./lang/$/$block");
const Node = require("./node");
const graph = require("./graph");
const Instance = require("./instance");

class IF$CLASS extends Node {
  before() {
    this.key = "if(" + this.condition.tokens.construct() + ")";
  }

  run(scope) {
    let instances;
    let statements = [];

    let instance = Instance.retrieve(scope, this.class.name);

    if (instance) instances = [instance];
    else instances = Object.keys(this.class.instances).map((i) => graph[i]);

    for (let instance of instances) {
      let statement = new IF$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.declaration = this;

      statement.true = $BLOCK(this.true.statements);
      statement.true.class = this.class;
      statement.true.instance = statement.instance;

      if (this.false !== undefined) {
        if (this.false instanceof $BLOCK) {
          statement.false = $BLOCK(this.false.statements);
        } else {
          statement.false = this.false;
        }

        statement.false.class = this.class;
        statement.false.instance = statement.instance;
      }

      statements.push(statement);
    }

    return { next: statements };
  }

  graph() {
    this.class.declarations[this.key] = this;
  }
}

IF$CLASS.prototype.type = "CLASS";
module.exports = IF$CLASS;
