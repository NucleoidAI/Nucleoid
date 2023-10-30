const graph = require("../../graph");
const { generate } = require("../estree/generator");

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
    const name = generate(this.node);

    if (path) {
      return graph[name] ? `state.${name}` : name;
    } else {
      return name;
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

module.exports = Identifier;
