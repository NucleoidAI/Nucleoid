import { append, root } from "../estree/estree";
import Node from "./Node";
import _ from "lodash";
import { retrieve } from "../../graph";

interface IdentifierNode {
  type: string;
  [key: string]: any;
}

export default class Identifier extends Node {
  declare node: IdentifierNode;

  constructor(node: IdentifierNode | string) {
    super(node);
  }

  static get types(): string[] {
    return ["Identifier", "MemberExpression", "ThisExpression"];
  }

  override get first(): Identifier | null {
    const n = this.node;
    if (["Identifier", "ThisExpression", "Literal"].includes(n.type)) {
      return new Identifier(n);
    } else if (n.type === "MemberExpression") {
      const r = root(n);
      if (!r.object) return null;
      return new Identifier(r.object);
    }
    return null;
  }

  // Add a getter that returns a string representation of first
  get firstAsString(): string {
    const first = this.first;
    return first ? first.toString() : "";
  }

  // Make toString() explicit for TypeScript
  override toString(): string {
    if (this.node.type === "Identifier") {
      return this.node.name;
    } else if (this.node.type === "ThisExpression") {
      return "this";
    } else if (this.node.type === "MemberExpression") {
      const object = new Identifier(this.node.object);
      const property = new Identifier(this.node.property);
      return `${object.toString()}.${property.toString()}`;
    } else if (this.node.type === "Literal") {
      return String(this.node.value);
    }
    return "";
  }

  override set first(first: Identifier) {
    const f = this.first;
    if (!f) throw new Error("Cannot set first on non-MemberExpression");
    Object.keys(this.node).forEach((key) => delete (this.node as any)[key]);
    Object.assign(this.node, first.node);
  }

  override get object(): Identifier | null {
    if (["Identifier", "ThisExpression", "Literal"].includes(this.node.type)) {
      return null;
    } else if (this.node.type === "MemberExpression") {
      return new Identifier((this.node as any).object);
    }
    return null;
  }

  override set object(object: Identifier) {
    const obj = this.object;
    if (!obj) throw new Error("Cannot set object on non-MemberExpression");
    (this.node as any).object = object.node;
  }

  override get last(): Identifier | null {
    if (["Identifier", "ThisExpression", "Literal"].includes(this.node.type)) {
      return new Identifier(this.node);
    } else if (this.node.type === "MemberExpression") {
      return new Identifier((this.node as any).property);
    }
    return null;
  }

  override set last(last: Identifier) {
    const l = this.last;
    if (!l) throw new Error("Cannot set last on non-MemberExpression");
    if (this.node.type === "MemberExpression") {
      (this.node as any).property = last.node;
    } else {
      Object.keys(this.node).forEach((key) => delete (this.node as any)[key]);
      Object.assign(this.node, last.node);
    }
  }

  override resolve(scope?: any): any {
    if (!scope) return this.node;

    const first = this.first;
    if (!first) return null;

    if (
      scope.callback
        .map((arg: any) => arg.toString())
        .includes(first.toString())
    ) {
      return this.node;
    }

    const scoped = scope.retrieve(this);
    if (scoped) {
      return scoped.resolve();
    }

    let node: any = this.node;
    if ((this.node as any).computed) {
      node = _.cloneDeep(this.node);
      const property = new Identifier(_.cloneDeep((node as any).property));
      const resolved = property.resolve(scope);
      if (resolved) {
        Object.keys((node as any).property).forEach(
          (key) => delete ((node as any).property as any)[key]
        );
        Object.assign((node as any).property, resolved);
      }
    }

    if (retrieve(first)) {
      const state: IdentifierNode = { type: "Identifier", name: "state" };
      return append(state, node);
    }
    return node;
  }

  override graph(scope?: any): Identifier | null {
    const first = this.first;
    if (!first) return null;
    if (
      scope.callback
        .map((arg: any) => arg.toString())
        .includes(first.toString())
    ) {
      return null;
    }
    const rec = retrieve(first);
    if (rec) {
      return removeBuiltins(this);
    }
    return null;
  }

  override walk(): Node[] {
    return [this];
  }

  [Symbol.iterator](): Iterator<{
    left: Identifier;
    right: Identifier | null;
  }> {
    const list: { left: Identifier; right: Identifier | null }[] = [];
    let left: Identifier = this;
    let right: Identifier | null = null;

    while (left.object) {
      const last = left.last!;

      // Handle the case when right is null
      if (right === null) {
        right = new Identifier(last.node);
      } else {
        // When right is not null, append last.node to right.node
        const rightNode = right.node;
        right = new Identifier(append(last.node, rightNode));
      }

      left = left.object!;
      list.push({ left, right });
    }

    let index = list.length - 1;
    return {
      next(): IteratorResult<{ left: Identifier; right: Identifier | null }> {
        if (index >= 0) {
          return { done: false, value: list[index--] };
        }
        return { done: true, value: undefined as any };
      },
    };
  }
}

function removeBuiltins(identifier: Identifier): Identifier {
  let current: any = _.cloneDeep(identifier.node);
  while (current.property?.name === "length") {
    current = current.object;
  }
  return new Identifier(current);
}
