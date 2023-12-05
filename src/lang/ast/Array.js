const Node = require("./Node");

class Array extends Node {
  constructor(elements) {
    super();
    this.elements = elements;
  }

  generate(scope) {
    return `[${this.elements.map((el) => el.generate(scope)).join(",")}]`;
  }
}

module.exports = Array;
