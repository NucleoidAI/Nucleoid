var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");

class CLASS extends Node {
  constructor() {
    super();
    this.instance = {};
    this.declaration = {};
  }

  run() {
    this.id = this.name;
    eval("state." + this.name + "=" + "class" + "{}");
  }
}

CLASS.prototype.type = "REGULAR";
module.exports = CLASS;
