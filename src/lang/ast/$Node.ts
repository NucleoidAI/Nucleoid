import ESTree from "../estree/generator";
import { parse } from "../estree/parser";
import $Identifier from "./$Identifier";
import { Node, Literal } from "acorn";

class $Node {
  iof: string;
  node: Node;

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

  get first(): $Identifier | null {
    return null;
  }

  get object(): $Identifier | null {
    return null;
  }

  get last(): $Identifier | null {
    return null;
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  resolve(scope): Node {
    return this.node;
  }

  generate(scope) {
    const resolved = this.resolve(scope);
    return ESTree.generate(resolved);
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  graph(scope): $Identifier[] {
    return [];
  }

  walk(): $Node[] {
    return [];
  }

  toString(scope) {
    return this.generate(scope);
  }
}

export default $Node;
