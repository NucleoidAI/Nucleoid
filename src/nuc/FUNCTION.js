const Node = require("./Node");

class FUNCTION extends Node {
  before() {
    this.key = this.name;
  }
}

module.exports = FUNCTION;
