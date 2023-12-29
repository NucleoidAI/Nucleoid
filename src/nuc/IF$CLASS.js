const IF$INSTANCE = require("./IF$INSTANCE");
const $BLOCK = require("../lang/$nuc/$BLOCK");
const NODE = require("./NODE");
const _ = require("lodash");
const { v4: uuid } = require("uuid");
const Scope = require("../Scope");
const Instruction = require("../instruction");

class IF$CLASS extends NODE {
  run(scope) {
    let instances;
    let statements = [];

    let instance = scope.$instance;

    if (instance) {
      instances = [instance];
    } else {
      instances = Object.values(this.class.instances);
    }

    for (let instance of instances) {
      const statement = new IF$INSTANCE(uuid());
      statement.condition = _.cloneDeep(this.condition);
      statement.true = $BLOCK(_.cloneDeep(this.true.stms));

      if (this.false?.iof === "$IF") {
        statement.false = _.cloneDeep(this.false);
      } else if (this.false?.iof === "$BLOCK") {
        statement.false = $BLOCK(_.cloneDeep(this.false.stms));
      }

      const instanceScope = new Scope(scope);
      instanceScope.$instance = instance;

      statements.push(
        new Instruction(instanceScope, statement, true, true, true, null, true)
      );
    }

    return { next: statements };
  }

  graph() {
    this.class.declarations[this.key] = this;
  }
}

IF$CLASS.prototype.type = "CLASS";
module.exports = IF$CLASS;
