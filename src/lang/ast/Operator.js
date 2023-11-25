const Node = require("./Node");
const _ = require("lodash");
const Identifier = require("./Identifier");

class Operator extends Node {
  resolve(scope) {
    if (scope) {
      const cloned = _.cloneDeep(this.node);

      traverseResolve(scope, cloned);

      return cloned;
    } else {
      return this.node;
    }
  }

  walk() {
    return traverseWalk(this.node);
  }

  graph(scope) {
    return traverseGraph(scope, this.node);
  }
}

function traverseWalk(node, acc = []) {
  if (["BinaryExpression", "LogicalExpression"].includes(node.type)) {
    traverseWalk(node.left, acc);
    traverseWalk(node.right, acc);
  } else if (node.type === "UnaryExpression") {
    traverseWalk(node.argument, acc);
  } else {
    acc.push(Node.convert(node).walk());
  }

  return acc;
}

function traverseResolve(scope, node) {
  if (["BinaryExpression", "LogicalExpression"].includes(node.type)) {
    traverseResolve(scope, node.left);
    traverseResolve(scope, node.right);
  } else if (node.type === "UnaryExpression") {
    traverseResolve(scope, node.argument);
  } else {
    const ast = Node.convert(node);

    if (Identifier.types.includes(ast.type)) {
      const resolved = ast.resolve(scope);
      Object.assign(node, resolved);
    } else {
      ast.resolve(scope);
    }
  }
}

function traverseGraph(scope, node, acc = []) {
  if (["BinaryExpression", "LogicalExpression"].includes(node.type)) {
    traverseGraph(scope, node.left, acc);
    traverseGraph(scope, node.right, acc);
  } else if (node.type === "UnaryExpression") {
    traverseGraph(scope, node.argument, acc);
  } else {
    acc.push(Node.convert(node).graph(scope));
  }

  return acc;
}

module.exports = Operator;
