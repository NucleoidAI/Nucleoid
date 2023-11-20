const Node = require("./lang/ast/Node");

// TODO Organize expression structure
class Expression extends Node {
  map(fn) {
    return mapReduce(this.node, fn);
  }

  find(fn) {
    // TODO Optimize this with own reduce
    const [first] = mapReduce(this.node, fn);
    return first;
  }

  resolve(scope) {
    return Node.convert(this.node).resolve(scope);
  }

  traverse(fn) {
    return traverseReduce(this.node, fn);
  }

  graph(scope, fn) {
    return graphReduce(scope, this.node, fn);
  }
}

// TODO Traverse with different types
function traverseReduce(exp, fn, acc = []) {
  if (["BinaryExpression", "LogicalExpression"].includes(exp.type)) {
    traverseReduce(exp.left, fn, acc);
    acc.push(exp.operator);
    traverseReduce(exp.right, fn, acc);
  } else if (exp.type === "UnaryExpression") {
    acc.push(exp.operator);
    traverseReduce(exp.argument, fn, acc);
  } else {
    const ast = Node.convert(exp);
    const curr = fn(ast);

    if (curr) {
      acc.push(curr);
    }
  }

  return acc;
}

function mapReduce(exp, fn, acc = []) {
  if (["BinaryExpression", "LogicalExpression"].includes(exp.type)) {
    mapReduce(exp.left, fn, acc);
    mapReduce(exp.right, fn, acc);
  } else if (exp.type === "UnaryExpression") {
    mapReduce(exp.argument, fn, acc);
  } else {
    const ast = Node.convert(exp);
    const curr = fn(ast);

    // Filter out undefined values
    if (curr !== undefined) {
      acc.push(curr);
    }
  }

  return acc;
}

function graphReduce(scope, exp, fn, acc = []) {
  if (["BinaryExpression", "LogicalExpression"].includes(exp.type)) {
    graphReduce(scope, exp.left, fn, acc);
    graphReduce(scope, exp.right, fn, acc);
  } else if (exp.type === "UnaryExpression") {
    graphReduce(scope, exp.argument, fn, acc);
  } else {
    const ast = Node.convert(exp);

    const graphed = [ast.graph(scope)];
    graphed.flat(Infinity).forEach((item) => {
      if (item) {
        const curr = fn(item);

        if (curr !== undefined) {
          acc.push(curr);
        }
      }
    });
  }

  return acc;
}

module.exports = Expression;
