const ESTree = require("../estree/generator");
const walk = require("acorn-walk");
const _ = require("lodash");
const { root } = require("../estree/estree");
const graph = require("../../graph");
const AST = require("./AST");

class Object extends AST {
  resolve(scope) {
    if (scope) {
      const cloned = _.cloneDeep(this.node);

      walk.simple(cloned, {
        Property(node) {
          if (!["Identifier", "MemberExpression"].includes(node.value.type)) {
            return;
          }

          const name = ESTree.generate(node.value);

          const scoped = scope.retrieve(name);

          if (scoped) {
            node.value = scoped;
            return;
          }

          if (graph[name]) {
            const state = {
              type: "MemberExpression",
              computed: false,
              optional: false,
              object: {
                type: "Identifier",
                name: "state",
              },
            };

            if (node.value.type === "Identifier") {
              state.property = node.value;
              node.value = state;
            } else if (node.value.type === "MemberExpression") {
              const firstNode = root(node.value);
              state.property = firstNode.object;
              firstNode.object = state;
              console.log("test");
            }
          }
        },
      });
      return cloned;
    } else {
      return this.node;
    }
  }
}

module.exports = Object;
