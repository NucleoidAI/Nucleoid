const Node = require("./NODE");
const BLOCK$INSTANCE = require("./BLOCK$INSTANCE");
const Instruction = require("../instruction");
const { v4: uuid } = require("uuid");

class BLOCK$CLASS extends Node {
  constructor(key) {
    super(key);
    this.statements = [];
  }

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
      let statement = new BLOCK$INSTANCE(uuid());
      statement.class = this.class;
      statement.instance = instance;
      statement.statements = this.statements;
      statement.declaration = this;
      statements.push(new Instruction(scope, statement, true, true, false));
      statements.push(new Instruction(scope, statement, false, false, true));
    }

    return { next: statements };
  }

  graph() {
    this.class.declarations[this.id] = this;
  }
}

BLOCK$CLASS.prototype.type = "CLASS";
module.exports = BLOCK$CLASS;
