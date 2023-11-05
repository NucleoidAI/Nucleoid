const AST = require("./AST");

class Literal extends AST {
  generate() {
    return this.node.raw;
  }
}

module.exports = Literal;
