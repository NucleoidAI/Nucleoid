var state = require("./state"); // eslint-disable-line no-unused-vars
var Node = require("./node");

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
  }
};
