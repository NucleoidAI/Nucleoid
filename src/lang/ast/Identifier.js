const graph = require("../../graph");
const { root } = require("../estree/estree");
const _ = require("lodash");
const AST = require("./AST");
const { append } = require("../estree/estree");

class Identifier extends AST {
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

  object() {
    if (this.node.type === "Identifier") {
      return new Identifier(this.node);
    } else if (this.node.type === "MemberExpression") {
      return new Identifier(this.node.object);
    }
  }

  resolve(scope) {
    if (scope) {
      const first = this.first();
      const scoped = scope.retrieve(first);

      if (scoped) {
        return scoped;
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
