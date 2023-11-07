const AST = require("./AST");

class Array extends AST {
  constructor(elements) {
    super();
    this.elements = elements;
  }

  generate(scope) {
    return `[${this.elements.map((el) => el.generate(scope)).join(",")}]`;
  }
}

module.exports = Array;
