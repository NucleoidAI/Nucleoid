const AST = require("./AST");

class Array extends AST {
  generate(scope) {
    return `[${this.node.elements.map((el) => el.generate(scope)).join(",")}]`;
  }
}

module.exports = Array;
