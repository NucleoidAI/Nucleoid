class Identifier {
  constructor(node) {
    this.node = node;

    if (node.type === "MemberExpression") {
      this.first = new Identifier(root(node));
      this.last = new Identifier(node.property);
    }
  }

  resolve() {
    if (this.node.type === "Identifier") {
      return this.node.name;
    } else if (this.node.type === "MemberExpression") {
      return traverse(this.node);
    }
  }
}

function root(node) {
  let curr = node;

  while (curr.object.type !== "Identifier") {
    curr = curr.object;
  }

  return curr.object;
}

function traverse(node) {
  if (node.type === "MemberExpression") {
    return traverse(node.object) + "." + node.property.name;
  } else {
    return node.name;
  }
}

module.exports = Identifier;
