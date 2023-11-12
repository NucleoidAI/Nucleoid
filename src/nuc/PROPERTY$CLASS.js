const Node = require("./NODE");
const PROPERTY$INSTANCE = require("./PROPERTY$INSTANCE");
const graph = require("../graph");
const _ = require("lodash");

class PROPERTY$CLASS extends Node {
  run(scope) {
    let instances;
    let statements = [];

    const instance = scope.instance(this.class.name);

    if (instance) {
      instances = [instance];
    } else {
      instances = Object.keys(this.class.instances).map((i) => graph[i]);
    }

    for (let instance of instances) {
      let statement = new PROPERTY$INSTANCE(`${instance.name}.${this.name}`);
      statement.class = this.class;
      statement.instance = instance;
      statement.object = instance;
      statement.name = this.name;
      statement.value = _.cloneDeep(this.value);
      statements.push(statement);
    }

    return { next: statements };
  }

  graph() {
    this.class.declarations[this.key] = this;
  }
}

PROPERTY$CLASS.prototype.type = "CLASS";
module.exports = PROPERTY$CLASS;
