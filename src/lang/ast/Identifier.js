const graph = require("../../graph");

const { root, append } = require("../estree/estree");
const Node = require("./Node");
const _ = require("lodash");

class Identifier extends Node {
  static get types() {
    return ["Identifier", "MemberExpression", "ThisExpression"];
  }

  get first() {
    if (["Identifier", "ThisExpression", "Literal"].includes(this.node.type)) {
      return new Identifier(this.node);
    } else if (this.node.type === "MemberExpression") {
      return new Identifier(root(this.node).object);
    } else {
      return null;
    }
  }

  set first(first) {
    if (["Identifier", "ThisExpression"].includes(this.node.type)) {
      Object.keys(this.node).forEach((key) => delete this.node[key]);
      Object.assign(this.node, first.node);
    } else if (this.node.type === "MemberExpression") {
      root(this.node).object = first.node;
    }
  }

  get object() {
    if (["Identifier", "ThisExpression", "Literal"].includes(this.node.type)) {
      return null;
    } else if (this.node.type === "MemberExpression") {
      return new Identifier(this.node.object);
    } else {
      return null;
    }
  }

  set object(object) {
    if (["Identifier", "ThisExpression"].includes(this.node.type)) {
      Object.keys(this.node).forEach((key) => delete this.node[key]);
      Object.assign(this.node, object.node);
    } else if (this.node.type === "MemberExpression") {
      this.node.object = object.node;
    }
  }

  get last() {
    if (["Identifier", "ThisExpression", "Literal"].includes(this.node.type)) {
      return new Identifier(this.node);
    } else if (this.node.type === "MemberExpression") {
      return new Identifier(this.node.property);
    } else {
      return null;
    }
  }

  set last(last) {
    if (["Identifier", "ThisExpression"].includes(this.node.type)) {
      Object.keys(this.node).forEach((key) => delete this.node[key]);
      Object.assign(this.node, last.node);
    } else if (this.node.type === "MemberExpression") {
      this.node.property = last.node;
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
          .includes(this.first.toString())
      ) {
        return this.node;
      }

      const scoped = scope.retrieve(this);

      if (scoped) {
        return scoped.resolve();
      }

      let node = this.node;

      // TODO Loop through all computed identifiers
      if (this.node.computed) {
        node = _.cloneDeep(this.node);
        const property = new Identifier(_.cloneDeep(node.property));
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

  graph(scope) {
    if (
      scope.callback
        .map((arg) => arg.toString())
        .includes(this.first.toString())
    ) {
      return null;
    }

    const first = graph.retrieve(this.first);

    if (first) {
      return removeBuiltins(this);
    }

    return null;
  }

  walk() {
    return [this];
  }

  [Symbol.iterator]() {
    const list = [];

    let left = this;
    let right = null;

    while (left.object) {
      // list.push(left.object);
      right = new Identifier(append(left.last.node, right?.node));
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

  return new Identifier(current);
}

module.exports = Identifier;
