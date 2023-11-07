const Node = require("./NODE");

class FUNCTION extends Node {
  before() {
    this.key = this.name;
  }
}

module.exports = FUNCTION;
