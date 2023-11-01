class Array {
  constructor(node) {
    this.node = node;
  }

  resolve() {
    return `[${this.node.elements.map((el) => el.resolve()).join(",")}]`;
  }
}

module.exports = Array;
