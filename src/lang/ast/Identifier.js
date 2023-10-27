class Identifier {
  constructor(node) {
    this.node = node;

    if (node.type === "MemberExpression") {
      this.first = new Identifier(first(node));
      this.object = new Identifier(node.object);
      this.last = new Identifier(node.property);
    }
  }

  resolve() {
    if (this.node.type === "Identifier") {
      return this.node.name;
    } else if (this.node.type === "MemberExpression") {
      return resolveTraverse(this.node);
    }
  }
}

function first(node) {
  let current = node;

  while (current.object.type !== "Identifier") {
    current = current.object;
  }

  return current.object;
}

function resolveTraverse(node) {
  if (node.type === "MemberExpression") {
    return resolveTraverse(node.object) + "." + node.property.name;
  } else {
    return node.name;
  }
}

module.exports = Identifier;
