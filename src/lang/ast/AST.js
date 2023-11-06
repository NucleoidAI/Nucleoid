const ESTree = require("../estree/generator");

class AST {
  constructor(node) {
    if (typeof node === "string") {
      const { parse } = require("../estree/parser");
      const name = node;
      this.node = parse(name, false);
    } else {
      this.node = node;
    }
  }

  first() {}

  object() {}

  last() {}

  // resolve() returns ESTree
  resolve() {
    return this.node;
  }

  generate(scope) {
    const resolved = this.resolve(scope);
    return ESTree.generate(resolved);
  }

  toString() {
    return this.generate();
  }
}

module.exports = AST;
