class Array {
  constructor(tokens) {
    this.tokens = tokens;
  }

  resolve() {
    return `[${this.tokens.elements.map((el) => el.resolve()).join(", ")}]`;
  }
}

module.exports = Array;
