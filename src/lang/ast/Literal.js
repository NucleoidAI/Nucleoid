class Literal {
  constructor(node) {
    this.node = node;
  }

  resolve() {
    return this.node.raw;
  }
}

module.exports = Literal;
