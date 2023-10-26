class Identifier {
  constructor(node) {
    this.node = node;

    if (node.type === "MemberExpression") {
      this.first = node.object.name;
      this.last = node.property.name;
    }
  }

  resolve() {
    if (this.node.type === "Identifier") {
      return this.node.name;
    } else if (this.node.type === "MemberExpression") {
      return this.first + "." + this.last;
    }
  }
}

module.exports = Identifier;
