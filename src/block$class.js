const Node = require("./node");
const BLOCK$INSTANCE = require("./block$instance");
const Instruction = require("./instruction");
const graph = require("./graph");
const Instance = require("./utils/instance");

class BLOCK$CLASS extends Node {
  constructor() {
    super();
    this.statements = [];
  }

  run(scope) {
    let instances;
    let statements = [];

    let instance = Instance.retrieve(scope, this.class.name);

    if (instance) instances = [instance];
    else instances = Object.keys(this.class.instances).map((i) => graph[i]);

    for (let instance of instances) {
      let statement = new BLOCK$INSTANCE();
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
