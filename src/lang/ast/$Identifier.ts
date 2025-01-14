import graph from "../../graph";
import { root, append } from "../estree/estree";
import $Node from "./$Node";
import _ from "lodash";
import { Expression, MemberExpression } from "acorn";

class $Identifier extends $Node {
  static get types() {
    return ["Identifier", "MemberExpression", "ThisExpression"];
  }

  get first(): $Identifier | null {
    if (["Identifier", "ThisExpression", "Literal"].includes(this.node.type)) {
      return new $Identifier(this.node);
    } else if (this.node.type === "MemberExpression") {
      return new $Identifier(root(this.node).object);
    } else {
      return null;
    }
  }

  set first(first: $Identifier) {
    if (["Identifier", "ThisExpression"].includes(this.node.type)) {
      Object.keys(this.node).forEach((key) => delete this.node[key]);
      Object.assign(this.node, first.node);
    } else if (this.node.type === "MemberExpression") {
      root(this.node).object = first.node;
    }
  }

  get object(): $Identifier | null {
    if (["Identifier", "ThisExpression", "Literal"].includes(this.node.type)) {
      return null;
    } else if (this.node.type === "MemberExpression") {
      const node = this.node as MemberExpression;
      return new $Identifier(node.object);
    } else {
      return null;
    }
  }

  set object(object: $Identifier) {
    if (["Identifier", "ThisExpression"].includes(this.node.type)) {
      Object.keys(this.node).forEach((key) => delete this.node[key]);
      Object.assign(this.node, object.node);
    } else if (this.node.type === "MemberExpression") {
      const node = this.node as MemberExpression;
      node.object = object.node as Expression;
    }
  }

  get last(): $Identifier | null {
    if (["Identifier", "ThisExpression", "Literal"].includes(this.node.type)) {
      return new $Identifier(this.node);
    } else if (this.node.type === "MemberExpression") {
      const node = this.node as MemberExpression;
      return new $Identifier(node.property);
    } else {
      return null;
    }
  }

  set last(last: $Identifier) {
    if (["Identifier", "ThisExpression"].includes(this.node.type)) {
      Object.keys(this.node).forEach((key) => delete this.node[key]);
      Object.assign(this.node, last.node);
    } else if (this.node.type === "MemberExpression") {
      const node = this.node as MemberExpression;
      node.property = last.node as Expression;
    }
  }

  resolve(scope) {
    if (scope) {
      const first = this.first;

      if (!first) {
        return null;
      }

      if (
        scope.callback
          .map((arg) => arg.toString())
          .includes(first.toString(scope))
      ) {
        return this.node;
      }

      const scoped = scope.retrieve(this);

      if (scoped) {
        return scoped.resolve();
      }

      let node = this.node as MemberExpression;

      // TODO Loop through all computed identifiers
      if (node.computed) {
        node = _.cloneDeep(this.node);
        const property = new $Identifier(_.cloneDeep(node.property));
        const resolved = property.resolve(scope);

        if (resolved) {
          Object.keys(node.property).forEach(
            (key) => delete node.property[key]
          );
          Object.assign(node.property, resolved);
        }
      }

      if (graph.retrieve(first)) {
        const state = {
          type: "Identifier",
          name: "state",
        };

        return append(state, node);
      } else {
        return node;
      }
    } else {
      return this.node;
    }
  }

  graph(scope): $Identifier[] {
    if (
      this.first &&
      scope.callback
        .map((arg) => arg.toString())
        .includes(this.first.toString(scope))
    ) {
      return [];
    }

    const first = graph.retrieve(this.first);

    if (first) {
      return [removeBuiltins(this)];
    }

    return [];
  }

  walk(): $Node[] {
    return [this];
  }

  [Symbol.iterator]() {
    const list: { left: $Identifier; right: $Identifier }[] = [];

    // eslint-disable-next-line @typescript-eslint/no-this-alias
    let left: $Identifier = this;
    let right;

    while (left.object) {
      // list.push(left.object);
      right = new $Identifier(append(left?.last?.node, right?.node));
      left = left.object;
      list.push({ left, right });
    }

    let index = list.length - 1;

    return {
      next() {
        if (index >= 0) {
          return { done: false, value: list[index--] };
        } else {
          return { done: true };
        }
      },
    };
  }
}

function removeBuiltins(identifier) {
  let current = _.cloneDeep(identifier.node);

  // Skip `Identifier` type
  while (current?.property?.name === "length") {
    current = current.object;
  }

  return new $Identifier(current);
}

export default $Identifier;
