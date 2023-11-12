const Node = require("./Node");

class Literal extends Node {
  generate() {
    return this.node.raw;
  }
}

module.exports = Literal;
