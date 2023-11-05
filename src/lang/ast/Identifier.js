const graph = require("../../graph");
const { root } = require("../estree/estree");
const _ = require("lodash");
const AST = require("./AST");

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
          type: "MemberExpression",
          computed: false,
          optional: false,
          object: {
            type: "Identifier",
            name: "state",
          },
        };

        if (this.node.type === "Identifier") {
          state.property = this.node;
          return state;
        } else if (this.node.type === "MemberExpression") {
          const cloned = _.cloneDeep(this.node);

          const firstNode = root(cloned);
          state.property = firstNode.object;
          firstNode.object = state;

          return cloned;
        }

        state.property = this.node;

        return state;
      } else {
        return this.node;
      }
    } else {
      return this.node;
    }
  }
}

module.exports = Identifier;
