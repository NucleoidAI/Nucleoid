const walk = require("acorn-walk");
const _ = require("lodash");
const AST = require("./AST");
const Identifier = require("./Identifier");

class Object extends AST {
  resolve(scope) {
    if (scope) {
      const cloned = _.cloneDeep(this.node);

      walk.simple(cloned, {
        Property(node) {
          if (Identifier.types.includes(node.value.type)) {
            const identifier = new Identifier(node.value);
            node.value = identifier.resolve(scope);
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
