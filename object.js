var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");
var Identifier = require("./identifier");
var $EXP = require("./$expression");

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

    let list = [];

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
