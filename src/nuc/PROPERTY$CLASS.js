const NODE = require("./NODE");
const PROPERTY$INSTANCE = require("./PROPERTY$INSTANCE");
const _ = require("lodash");
const Instruction = require("../instruction");
const Scope = require("../Scope");

class PROPERTY$CLASS extends NODE {
  run(scope) {
    let instances;
    let statements = [];

    const instance = scope.$instance;

    if (instance) {
      instances = [instance];
    } else {
      instances = Object.values(this.class.instances);
    }

    for (let instance of instances) {
      let statement = new PROPERTY$INSTANCE(`${instance.name}.${this.name}`);
      statement.object = instance;
      statement.name = this.name;
      statement.value = _.cloneDeep(this.value);

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

PROPERTY$CLASS.prototype.type = "CLASS";
module.exports = PROPERTY$CLASS;
