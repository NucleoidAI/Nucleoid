/* eslint-disable getter-return */
import * as ESTree from "../estree/generator";

interface BaseNode {
  type: string;
  [key: string]: any;
}

class Node {
  node: BaseNode;
  iof: string;

  constructor(node?: BaseNode | string) {
    this.iof = this.constructor.name;

    if (node === undefined) {
      this.node = {
        type: "Literal",
        value: null,
        raw: "null",
      };
    } else if (typeof node === "object" && node.type) {
      this.node = node;
    } else {
      const { parse } = require("../estree/parser");
      const name = node as string;
      this.node = parse(name, false);
    }
  }

  get type(): string {
    return this.node.type;
  }

  get first(): Node | null {
    return null;
  }

  get object(): Node | null {
    return null;
  }

  get last(): Node | null {
    return null;
  }

  resolve(scope?: any): any {
    return this.node;
  }

  generate(scope?: any): string {
    const resolved = this.resolve(scope);
    return ESTree.generate(resolved);
  }

  graph(scope?: any): Node | null {
    return null;
  }

  walk(): Node[] {
    return [];
  }

  toString(): string {
    return this.generate();
  }

  static convert(node: BaseNode): Node {
    const Literal = require("./Literal");
    const Identifier = require("./Identifier");
    const Array = require("./Array");
    const New = require("./New");
    const Object = require("./Object");
    const Function = require("./Function");
    const Operator = require("./Operator");
    const Call = require("./Call");
    const Template = require("./Template");

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
      case "TemplateLiteral": {
        return new Template(node);
      }
      default: {
        return new Operator(node);
      }
    }
  }
}

export default Node;
