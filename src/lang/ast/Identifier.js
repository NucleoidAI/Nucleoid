const graph = require("../../graph");

class Identifier {
  constructor(node) {
    this.node = node;

    if (node.type === "MemberExpression") {
      this.first = new Identifier(first(node));
      this.object = new Identifier(node.object);
      this.last = new Identifier(node.property);
    }
  }

  resolve(path = false) {
    let identifier;
    if (this.node.type === "Identifier") {
      identifier = this.node.name;
    } else if (this.node.type === "MemberExpression") {
      identifier = traverse(this.node);
    } else {
      throw new Error("Unknown identifier type");
    }

    if (path) {
      const name = identifier;
      return graph[name] ? `state.${name}` : name;
    } else {
      return identifier;
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

function traverse(node) {
  if (node.type === "MemberExpression") {
    return traverse(node.object) + "." + node.property.name;
  } else {
    return node.name;
  }
}

module.exports = Identifier;
