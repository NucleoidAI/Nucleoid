var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");
var Identifier = require("./identifier");
var $VALUE = require("./$value");

module.exports = class OBJECT extends Node {
  constructor() {
    super();
    this.property = {};
  }

  prepare() {
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
      let context = $VALUE(this.class.name + "s.push ( " + this.name + " )", 0);
      list.push(context.statement);
    }

    return list;
  }

  graph() {
    this.class.instance[this.key] = this;
  }
};
