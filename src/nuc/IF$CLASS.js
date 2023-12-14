const IF$INSTANCE = require("./IF$INSTANCE");
const $BLOCK = require("../lang/$nuc/$BLOCK");
const Node = require("./NODE");
const _ = require("lodash");
const { v4: uuid } = require("uuid");

class IF$CLASS extends Node {
  run(scope) {
    let instances;
    let statements = [];

    let instance = scope.instance(this.class.name);

    if (instance) {
      instances = [instance];
    } else {
      instances = Object.values(this.class.instances);
    }

    for (let instance of instances) {
      const statement = new IF$INSTANCE(uuid());
      statement.class = this.class;
      statement.instance = instance;
      statement.condition = _.cloneDeep(this.condition);
      statement.true = $BLOCK(_.cloneDeep(this.true.statements));

      if (this.false?.iof === "$IF") {
        statement.false = _.cloneDeep(this.false);
      } else if (this.false?.iof === "$BLOCK") {
        statement.false = $BLOCK(_.cloneDeep(this.false.statements));
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
