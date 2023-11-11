const graph = require("../../graph");

const { root, append } = require("../estree/estree");
const AST = require("./AST");

class Identifier extends AST {
  static get types() {
    return ["Identifier", "MemberExpression"];
  }

  get first() {
    if (this.node.type === "Identifier") {
      return new Identifier(this.node);
    } else if (this.node.type === "MemberExpression") {
      return new Identifier(root(this.node).object);
    } else {
      throw new Error(`Unknown identifier type ${this.node.type}`);
    }
  }

  set first(first) {
    if (this.node.type === "Identifier") {
      this.node = first.node;
    } else if (this.node.type === "MemberExpression") {
      this.node.object = first.node;
    } else {
      throw new Error(`Unknown identifier type ${this.node.type}`);
    }
  }

  get object() {
    if (this.node.type === "Identifier") {
      return null;
    } else if (this.node.type === "MemberExpression") {
      return new Identifier(this.node.object);
    } else {
      throw new Error(`Unknown identifier type ${this.node.type}`);
    }
  }

  set object(object) {
    if (this.node.type === "Identifier") {
      this.node = object.node;
    } else if (this.node.type === "MemberExpression") {
      this.node.object = object.node;
    } else {
      throw new Error(`Unknown identifier type ${this.node.type}`);
    }
  }

  get last() {
    if (this.node.type === "Identifier") {
      return new Identifier(this.node);
    } else if (this.node.type === "MemberExpression") {
      return new Identifier(this.node.property);
    } else {
      throw new Error(`Unknown identifier type ${this.node.type}`);
    }
  }

  set last(last) {
    if (this.node.type === "Identifier") {
      this.node = last.node;
    } else if (this.node.type === "MemberExpression") {
      this.node.property = last.node;
    } else {
      throw new Error(`Unknown identifier type ${this.node.type}`);
    }
  }

  resolve(scope) {
    if (scope) {
      const first = this.first;

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
}

module.exports = Identifier;
