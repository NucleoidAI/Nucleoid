const NODE = require("./NODE");
const BLOCK$INSTANCE = require("./BLOCK$INSTANCE");
const Instruction = require("../Instruction");
const { v4: uuid } = require("uuid");
const Scope = require("../Scope");

class BLOCK$CLASS extends NODE {
  constructor(key) {
    super(key);
    this.statements = [];
  }

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
      let statement = new BLOCK$INSTANCE(uuid());
      statement.class = this.class;
      statement.instance = instance;
      statement.statements = this.statements;
      statement.declaration = this;

      const instanceScope = new Scope(scope.block ? scope : null);
      instanceScope.$instance = this;

      statements.push(
        new Instruction(instanceScope, statement, true, true, false, false)
      );
      statements.push(
        new Instruction(instanceScope, statement, false, false, true, true)
      );
    }

    return { next: statements };
  }

  graph() {
    this.class.declarations[this.id] = this;
  }
}

BLOCK$CLASS.prototype.type = "CLASS";
module.exports = BLOCK$CLASS;
