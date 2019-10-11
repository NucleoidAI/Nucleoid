var Node = require("./node");
var BLOCK$INSTANCE = require("./block$instance");
var Instruction = require("./instruction");

class BLOCK$CLASS extends Node {
  constructor() {
    super();
    this.statements = [];
  }

  run(scope) {
    let list = [];

    let instance = scope.retrieve(this.class.name);

    if (instance) {
      let statement = new BLOCK$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.statements = this.statements;
      return [
        new Instruction(scope, statement, true, false),
        new Instruction(scope, statement, false, true)
      ];
    }

    for (let node in this.class.instance) {
      let statement = new BLOCK$INSTANCE();
      statement.class = this.class;
      statement.instance = this.class.instance[node];
      statement.statements = this.statements;
      list.push(new Instruction(scope, statement, true, false));
      list.push(new Instruction(scope, statement, false, true));
    }

    return list;
  }
}

BLOCK$CLASS.prototype.type = "CLASS";
module.exports = BLOCK$CLASS;
