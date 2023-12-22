const Node = require("./NODE");
const Id = require("../lib/identifier");
const OBJECT$INSTANCE = require("./OBJECT$INSTANCE");
const graph = require("../graph");
const Scope = require("../Scope");
const Instruction = require("../instruction");

class OBJECT$CLASS extends Node {
  before() {
    this.key = Id.serialize(this);
  }

  run(scope) {
    let instances;
    const statements = [];

    const instance = scope.$instance;

    if (instance) {
      instances = [instance];
    } else {
      instances = Object.values(this.object.instances);
    }

    for (let instance of instances) {
      const statement = new OBJECT$INSTANCE(`${instance.name}.${this.name}`);
      statement.class = this.class;
      statement.object = graph.retrieve(`${instance.name}`);
      statement.name = this.name;

      const instanceScope = new Scope(scope);
      instanceScope.$instance = instance;

      statements.push(
        new Instruction(instanceScope, statement, true, true, true, null, true)
      );
    }

    return { next: statements };
  }

  graph() {
    this.object.declarations[this.key] = this;
  }
}

OBJECT$CLASS.prototype.type = "CLASS";
module.exports = OBJECT$CLASS;
