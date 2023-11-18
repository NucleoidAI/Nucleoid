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
      this.node = first.node;
    } else if (this.node.type === "MemberExpression") {
      this.node.object = first.node;
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
      this.node = object.node;
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
      this.node = last.node;
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
        return this;
      }

      const scoped = scope.retrieve(this);

      if (scoped) {
        return scoped.resolve();
      }

      if (graph.retrieve(first)) {
        const state = {
          type: "Identifier",
          name: "state",
        };

        return append(state, this.node);
      } else {
        return this.node;
      }
    } else {
      return this.node;
    }
  }

  // TODO Check scope callback
  graph() {
    const first = graph.retrieve(this.first);

    if (first) {
      return removeBuiltins(this);
    }

    return null;
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
