/* eslint-disable getter-return */
const ESTree = require("../estree/generator");

class Node {
  // Rename to Node
  constructor(node) {
    if (node === undefined) {
      this.node = {
        type: "Literal",
        value: null,
        raw: "null",
      };
    } else if (node.type) {
      this.node = node;
    } else {
      const { parse } = require("../estree/parser");
      const name = node;
      this.node = parse(name, false);
    }
  }

  get first() {}

  get object() {}

  get last() {}

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

module.exports = Node;
