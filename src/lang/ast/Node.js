/* eslint-disable getter-return */
const ESTree = require("../estree/generator");

class Node {
  constructor(node) {
    if (node === undefined) {
      this.node = {
        type: "Literal",
        value: null,
        raw: "null",
      };
    } else if (node.type) {
      this.node = node;
    } else {
      const { parse } = require("../estree/parser");
      const name = node;
      this.node = parse(name, false);
    }
  }

  get type() {
    return this.node.type;
  }

  get first() {
    return null;
  }

  get object() {
    return null;
  }

  get last() {
    return null;
  }

  resolve() {
    return this.node;
  }

  generate(scope) {
    const resolved = this.resolve(scope);
    return ESTree.generate(resolved);
  }

  graph() {
    return null;
  }

  walk() {
    return [];
  }

  toString() {
    return this.generate();
  }

  static convert(node) {
    const Literal = require("./Literal");
    const Identifier = require("./Identifier");
    const Array = require("./Array");
    const New = require("./New");
    const Object = require("./Object");
    const Function = require("./Function");
    const Operator = require("./Operator");
    const Call = require("./Call");

    switch (node.type) {
      case "Literal": {
        return new Literal(node);
      }
      case "Identifier":
      case "MemberExpression": {
        return new Identifier(node);
      }
      case "ArrayExpression": {
        const elements = node.elements.map((el) => Node.convert(el));
        return new Array(elements);
      }
      case "NewExpression": {
        return new New(node);
      }
      case "ObjectExpression": {
        return new Object(node);
      }
      case "FunctionExpression":
      case "ArrowFunctionExpression": {
        return new Function(node);
      }
      case "CallExpression": {
        return new Call(node);
      }
      default: {
        return new Operator(node);
      }
    }
  }
}

module.exports = Node;
