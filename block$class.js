var Node = require("./node");
var BLOCK$INSTANCE = require("./block$instance");
var Instruction = require("./instruction");
var graph = require("./graph");

class BLOCK$CLASS extends Node {
  constructor() {
    super();
    this.statements = [];
  }

  run(scope) {
    let instances;
    let statements = [];

    let instance = scope.retrieve(this.class.name);

    if (instance) instances = [instance];
    else instances = Object.keys(this.class.instance).map(i => graph.node[i]);

    for (let instance of instances) {
      let statement = new BLOCK$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.statements = this.statements;
      statements.push(new Instruction(scope, statement, true, true, false));
      statements.push(new Instruction(scope, statement, false, false, true));
    }

    return statements;
  }
}

BLOCK$CLASS.prototype.type = "CLASS";
module.exports = BLOCK$CLASS;
