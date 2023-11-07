const graph = require("../../graph");

const { root, append } = require("../estree/estree");
const AST = require("./AST");

class Identifier extends AST {
  static get types() {
    return ["Identifier", "MemberExpression"];
  }

  // TODO Convert to getter
  first() {
    if (this.node.type === "Identifier") {
      return new Identifier(this.node);
    } else if (this.node.type === "MemberExpression") {
      return new Identifier(root(this.node).object);
    }
  }

  last() {
    if (this.node.type === "Identifier") {
      return new Identifier(this.node);
    } else if (this.node.type === "MemberExpression") {
      return new Identifier(this.node.property);
    }
  }

  resolve(scope) {
    if (scope) {
      const first = this.first();

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

  object() {
    if (this.node.type === "Identifier") {
      return new Identifier(this.node);
    } else if (this.node.type === "MemberExpression") {
      return new Identifier(this.node.object);
    }
  }
}

module.exports = Identifier;
