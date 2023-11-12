const IF$INSTANCE = require("./IF$INSTANCE");
const $BLOCK = require("../lang/$nuc/$BLOCK");
const Node = require("./NODE");
const graph = require("../graph");
const _ = require("lodash");
const { v4: uuid } = require("uuid");

class IF$CLASS extends Node {
  before() {
    this.key = `if(${this.condition.tokens})`;
  }

  run(scope) {
    let instances;
    let statements = [];

    let instance = scope.instance(this.class.name);

    if (instance) {
      instances = [instance];
    } else {
      instances = Object.keys(this.class.instances).map((i) => graph[i]);
    }

    for (let instance of instances) {
      const statement = new IF$INSTANCE(uuid());
      statement.class = this.class;
      statement.instance = instance;
      statement.condition = _.cloneDeep(this.condition);

      if (this.true) {
        statement.true = $BLOCK(_.cloneDeep(this.true.statements));
        statement.true.class = this.class;
        statement.true.instance = statement.instance;
      }

      if (this.false) {
        statement.false = $BLOCK(_.cloneDeep(this.false.statements));
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
