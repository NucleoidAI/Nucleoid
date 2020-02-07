var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");
var Identifier = require("./identifier");
var $EXP = require("./$expression");
var Instruction = require("./instruction");
var LET = require("./let");

module.exports = class OBJECT extends Node {
  constructor() {
    super();
    this.property = {};
  }

  before() {
    if (this.name === undefined && this.object === undefined) {
      this.key = this.class.name.toLowerCase() + this.sequence;
      this.name = this.key;
    } else {
      this.key = Identifier.serialize(this);
    }
  }

  run(scope) {
    let name = this.key;

    eval("state." + name + " = new state." + this.class.name + "()");
    scope.instance[this.class.name] = this;
    scope.object = this;

    let list = [];

    for (let i = 0; i < this.class.args.length; i++) {
      let local = new LET();
      local.name = this.class.args[i];

      if (this.args[i] !== undefined) {
        let context = $EXP(this.args[i], 0);
        local.value = context.statement.run();
        list.push(local);
      } else {
        let context = $EXP("undefined", 0);
        local.value = context.statement.run();
        list.push(local);
      }
    }

    if (this.class.construct !== undefined) {
      let construct = this.class.construct;
      let instruction = new Instruction(scope, construct, false, true, false);
      list.push(instruction);
    }

    for (let node in this.class.declaration)
      list.push(this.class.declaration[node]);

    if (this.object === undefined) {
      let context = $EXP(this.class.name + "s.push ( " + this.name + " )", 0);
      list.push(context.statement);

      eval(`state.${name}.id="${name}"`);

      context = $EXP(this.name, 0);
      list.push(context.statement);
    }

    return list;
  }

  graph() {
    if (this.object !== undefined) this.object.property[this.name] = this;
    this.class.instance[this.key] = this;
  }
};
