import { Literal, Node } from "acorn";

import ESTree from "../estree/generator";
import { parse } from "../estree/parser";

interface NodeConverter {
  convert(node: Node | string): $Node;
}

class $Node {
  iof: string;
  node: Node;
  static registry: Record<string, new (node?: Node | string) => $Node> = {};

  static convert(node: Node | string): $Node {
    if (!node) return new $Node();

    const nodeObj: Node =
      typeof node === "string" ? parse(node as string, false) : (node as Node);
    const type = nodeObj.type;

    const NodeClass = this.registry[type];
    if (NodeClass) {
      return new NodeClass(nodeObj);
    }

    return new $Node(nodeObj);
  }

  static register(
    type: string,
    NodeClass: new (node?: Node | string) => $Node
  ): void {
    this.registry[type] = NodeClass;
  }

  constructor(node?: Node | string) {
    this.iof = this.constructor.name;

    if (node === undefined) {
      this.node = {
        type: "Literal",
        value: null,
        raw: "null",
      } as Literal;
    } else if ((node as Node).type) {
      this.node = node as Node;
    } else {
      this.node = parse(node as string, false);
    }
  }

  get type(): string {
    return this.node.type;
  }

  get first(): $Node | null {
    return null;
  }

  get object(): $Node | null {
    return null;
  }

  get last(): $Node | null {
    return null;
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  resolve(scope: unknown): Node {
    return this.node;
  }

  generate(scope: unknown): string {
    const resolved = this.resolve(scope);
    return ESTree.generate(resolved);
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  graph(scope: unknown): $Node[] {
    return [];
  }

  walk(): $Node[] {
    return [];
  }

  toString(scope: unknown): string {
    return this.generate(scope);
  }
}

export default $Node;

