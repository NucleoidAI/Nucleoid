var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");
var $VAR = require("./$variable");
var $VALUE = require("./$value");

module.exports = class CLASS extends Node {
  constructor() {
    super();
    this.instance = {};
    this.declaration = {};
  }

  prepare() {
    this.key = this.name;
  }

  run() {
    eval("state." + this.name + "=" + "class" + "{}");
    let context = $VALUE("[]", 0);
    return $VAR(this.name + "s", context.statement);
  }
};
