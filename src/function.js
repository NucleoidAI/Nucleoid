const Node = require("./node");

class FUNCTION extends Node {
  before() {
    this.key = this.name;
  }
}

module.exports = FUNCTION;
