var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");

class CLASS extends Node {
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
  }
}

CLASS.prototype.type = "REGULAR";
module.exports = CLASS;
