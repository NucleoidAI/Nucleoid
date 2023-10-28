const Identifier = require("./Identifier");
const Literal = require("./Literal");
const Call = require("./Call");
const Array = require("./Array");

class Expression {
  constructor(tokens) {
    this.tokens = tokens;
  }
  map(fn) {
    return mapReduce(this.tokens, fn);
  }

  traverse(fn) {
    return traverseReduce(this.tokens, fn);
  }
}

function convertToAST(token) {
  switch (token.type) {
    case "Literal": {
      return new Literal(token);
    }
    case "Identifier":
    case "MemberExpression": {
      return new Identifier(token);
    }
    case "CallExpression": {
      return new Call(token);
    }
    case "ArrayExpression": {
      return new Array(token);
    }
    default: {
      throw new Error(`ParserError: Unknown node type '${token.type}'`);
    }
  }
}

function traverseReduce(exp, fn, acc = []) {
  if (exp.type === "BinaryExpression") {
    traverseReduce(exp.left, fn, acc);
    acc.push(exp.operator);
    traverseReduce(exp.right, fn, acc);
  } else {
    const ast = convertToAST(exp);
    const cur = fn(ast);

    if (cur) {
      acc.push(cur);
    }
  }

  return acc;
}

function mapReduce(exp, fn, acc = []) {
  if (exp.type === "BinaryExpression") {
    mapReduce(exp.left, fn, acc);
    mapReduce(exp.right, fn, acc);
  } else {
    const ast = convertToAST(exp);
    const cur = fn(ast);

    // Filter out undefined values
    if (cur !== undefined) {
      acc.push(cur);
    }
  }

  return acc;
}

module.exports = Expression;
